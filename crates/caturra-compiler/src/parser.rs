//! Recursive-descent parser for the supported Java subset.
//!
//! Diagnostics policy (see `specs/LANGUAGE.md`): constructs we plan to
//! support but haven't built yet produce a specific, friendly
//! "not yet supported" message with a source span — never a generic
//! syntax error — and the parser recovers so one file reports every
//! problem, not just the first.

use crate::ast::{
    Annotation, AssignTarget, BinaryOp, CatchClause, ClassDecl, CompilationUnit, Expr, FieldDecl,
    ImportDecl, InitBlock, LambdaBody, LambdaParam, Literal, LocalDeclarator, MethodDecl, Param,
    Stmt, SwitchArm, TypeParam, TypeRef, UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourcePosition, SourceSpan};
use crate::lexer::{Keyword, Token, TokenKind};

/// Parse a token stream into a compilation unit.
///
/// Always returns whatever could be parsed; problems are reported as
/// diagnostics alongside.
#[must_use]
pub fn parse(path: &str, tokens: Vec<Token>) -> (CompilationUnit, Vec<Diagnostic>) {
    let mut parser = Parser {
        path,
        tokens,
        pos: 0,
        diagnostics: Vec::new(),
        anon_classes: Vec::new(),
        anon_counter: 0,
        local_counter: 0,
        pending_annotations: Vec::new(),
    };
    let unit = parser.compilation_unit();
    (unit, parser.diagnostics)
}

/// The friendly message for statement-starting keywords caturra doesn't
/// support yet; `None` when the keyword can begin a real statement.
/// The source spelling of a primitive type keyword (for `int.class`).
/// One resource in a `try (...)` header: `Type name = init`.
struct Resource {
    ty: TypeRef,
    name: String,
    init: Expr,
    /// The Java 9 form `try (existing)`, naming an already-declared
    /// effectively-final variable. It is closed like any other resource but
    /// must NOT be re-declared.
    existing: bool,
    span: SourceSpan,
}

/// Desugar `try (resources) body catches finally` into the plain try/catch the
/// JLS §14.20.3 specifies, so codegen never sees a resource at all.
///
/// Per resource, innermost last, the translation is §14.20.3.1's:
///
/// ```text
/// {
///   RType r = init;                  // implicitly final
///   Throwable primary = null;
///   try { body }
///   catch (Throwable t) { primary = t; throw t; }
///   finally {
///     if (r != null) {
///       if (primary != null) {
///         try { r.close(); } catch (Throwable s) { primary.addSuppressed(s); }
///       } else {
///         r.close();
///       }
///     }
///   }
/// }
/// ```
///
/// Three things this buys that a bare `finally { r.close(); }` did not, each
/// of which was a confirmed divergence:
///
/// * **the body's exception wins.** If the body throws and `close()` throws
///   too, Java keeps the body's and attaches close's as SUPPRESSED. The old
///   shape let close's replace it, so the program caught the wrong exception
///   entirely.
/// * **an initializer failure is catchable.** The declarations now sit INSIDE
///   the outer try, as §14.20.3.2 requires, so `try (R r = new R()) {...}
///   catch (Exception e)` catches a throwing constructor — and any earlier
///   resource still closes. They used to sit outside, so the exception escaped
///   the statement's own catch and killed the program.
/// * **a null resource is skipped**, not dereferenced.
#[allow(clippy::too_many_lines)] // the JLS 14.20.3 translation, spelled out
fn desugar_try_with_resources(
    resources: Vec<Resource>,
    body: Vec<Stmt>,
    catches: Vec<CatchClause>,
    finally_body: Option<Vec<Stmt>>,
    span: SourceSpan,
) -> Stmt {
    let name_expr = |name: &str| Expr::Name {
        path: vec![name.to_owned()],
        span,
    };
    let null_literal = || Expr::Literal {
        value: Literal::Null,
        span,
    };
    let is_null = |name: &str, negated: bool| Expr::Binary {
        op: if negated { BinaryOp::Ne } else { BinaryOp::Eq },
        lhs: Box::new(name_expr(name)),
        rhs: Box::new(null_literal()),
        span,
    };
    let close_call = |name: &str| {
        Stmt::Expr(Expr::Call {
            receiver: Some(Box::new(name_expr(name))),
            method: String::from("close"),
            args: Vec::new(),
            span,
        })
    };

    // Innermost first, so the LAST resource declared is the FIRST closed.
    let mut inner = body;
    for (depth, resource) in resources.into_iter().enumerate().rev() {
        // Synthetic names cannot collide with a source identifier: `$` is not
        // in caturra's identifier set, and the depth keeps nested statements
        // apart.
        let primary = format!("__caturraPrimary${depth}");
        let thrown = format!("__caturraThrown${depth}");
        let closing = format!("__caturraClosing${depth}");

        // `try { r.close(); } catch (Throwable s) { primary.addSuppressed(s); }`
        let close_suppressing = Stmt::Try {
            body: vec![close_call(&resource.name)],
            catches: vec![CatchClause {
                types: vec![TypeRef::Named(String::from("Throwable"))],
                name: closing.clone(),
                body: vec![Stmt::Expr(Expr::Call {
                    receiver: Some(Box::new(name_expr(&primary))),
                    method: String::from("addSuppressed"),
                    args: vec![name_expr(&closing)],
                    span,
                })],
                span,
            }],
            finally_body: None,
            span,
        };

        // `if (primary != null) { close-suppressing } else { r.close(); }`
        let close_either_way = Stmt::If {
            cond: is_null(&primary, true),
            then: Box::new(Stmt::Block(vec![close_suppressing])),
            els: Some(Box::new(Stmt::Block(vec![close_call(&resource.name)]))),
            span,
        };

        // A null resource is not closed at all (JLS §14.20.3.1).
        let guarded_close = Stmt::If {
            cond: is_null(&resource.name, true),
            then: Box::new(Stmt::Block(vec![close_either_way])),
            els: None,
            span,
        };

        let attempt = Stmt::Try {
            body: inner,
            catches: vec![CatchClause {
                types: vec![TypeRef::Named(String::from("Throwable"))],
                name: thrown.clone(),
                body: vec![
                    Stmt::Assign {
                        target: AssignTarget::Var(primary.clone()),
                        op: None,
                        value: name_expr(&thrown),
                        span,
                    },
                    Stmt::Throw {
                        value: name_expr(&thrown),
                        span,
                    },
                ],
                span,
            }],
            finally_body: Some(vec![guarded_close]),
            span,
        };

        let mut block = Vec::with_capacity(3);
        if !resource.existing {
            block.push(Stmt::LocalDecl {
                ty: resource.ty,
                is_final: true,
                declarators: vec![LocalDeclarator {
                    name: resource.name,
                    init: Some(resource.init),
                    span: resource.span,
                    extra_dims: 0,
                }],
                span,
            });
        }
        block.push(Stmt::LocalDecl {
            ty: TypeRef::Named(String::from("Throwable")),
            is_final: false,
            declarators: vec![LocalDeclarator {
                name: primary,
                init: Some(null_literal()),
                span,
                extra_dims: 0,
            }],
            span,
        });
        block.push(attempt);
        inner = block;
    }

    // The statement's OWN catches and finally wrap the whole thing, including
    // the resource declarations — that is what makes an initializer failure
    // catchable here (JLS §14.20.3.2).
    if catches.is_empty() && finally_body.is_none() {
        Stmt::Block(inner)
    } else {
        Stmt::Try {
            body: inner,
            catches,
            finally_body,
            span,
        }
    }
}

fn primitive_type_name(keyword: Keyword) -> Option<&'static str> {
    Some(match keyword {
        Keyword::Int => "int",
        Keyword::Double => "double",
        Keyword::Boolean => "boolean",
        Keyword::Char => "char",
        Keyword::Long => "long",
        Keyword::Float => "float",
        Keyword::Short => "short",
        Keyword::Byte => "byte",
        _ => return None,
    })
}

/// The error for a keyword that cannot begin a statement, or `None` when it can.
///
/// Two different failures hide here, and conflating them is harmful. `public`
/// cannot start a statement in *any* Java compiler, so we answer with javac's
/// wording and never mention caturra — the old catch-all told students (and our
/// corpus tooling, which reads these strings) that broken source was our
/// limitation. Only valid Java that caturra does not implement says
/// "not supported by caturra".
fn statement_start_error(keyword: Keyword) -> Option<&'static str> {
    match keyword {
        // Valid Java that caturra does not implement. These must say so: the
        // level generator keys off "caturra"/"not supported" to tell an engine
        // gap apart from a student's mistake.
        // A local class in a block is handled before `statement`; reaching here
        // means a class in a non-block position (`if (c) class L {}`), which
        // javac rejects outright.
        Keyword::Class => Some("class, interface or enum declaration not allowed here"),
        Keyword::Synchronized => {
            Some("'synchronized' is not supported by caturra; programs run single-threaded")
        }

        // Invalid Java: javac's wording, with no mention of caturra.
        Keyword::Else => Some("'else' without a matching 'if'"),
        Keyword::Catch => Some("'catch' without 'try'"),
        Keyword::Finally => Some("'finally' without 'try'"),
        Keyword::Case => Some("orphaned case"),
        Keyword::Default => Some("orphaned default"),
        Keyword::Interface => Some("interface not allowed here"),
        Keyword::Enum => Some("enum types must not be local"),
        Keyword::Abstract | Keyword::Strictfp => Some("class, interface, or enum expected"),

        // `new` begins a class instance creation, which JLS §14.8 allows as a
        // statement expression. Types, `final` and `this` legitimately begin a
        // declaration or an expression. Let the normal parse continue.
        Keyword::New
        | Keyword::Int
        | Keyword::Double
        | Keyword::Boolean
        | Keyword::Char
        | Keyword::Final
        | Keyword::This
        | Keyword::Long
        | Keyword::Float
        | Keyword::Byte
        | Keyword::Short
        // `var x = e;` — a real declaration start, handled by local_declaration.
        | Keyword::Var => None,

        // Everything else — modifiers, `import`, `extends`, `instanceof`, the
        // reserved-but-unused `goto`/`const` — is javac's generic case.
        _ => Some("illegal start of expression"),
    }
}

/// Lower `x++` / `x--` (and `a[i]++`) to `+= 1` / `-= 1`.
fn increment_statement(
    target: AssignTarget,
    increment: bool,
    start: SourcePosition,
    end: SourcePosition,
) -> Stmt {
    let span = SourceSpan { start, end };
    Stmt::Assign {
        target,
        op: Some(if increment {
            BinaryOp::Add
        } else {
            BinaryOp::Sub
        }),
        value: Expr::Literal {
            value: Literal::Int(1),
            span,
        },
        span,
    }
}

/// Convert an expression to an assignment target, if it has the right
/// shape (`x`, `a[i]`, `p.x`, `this.x`).
fn assignment_target(expr: &Expr) -> Option<AssignTarget> {
    match expr {
        Expr::Name { path, .. } if path.len() == 1 => Some(AssignTarget::Var(path[0].clone())),
        Expr::Name { path, span } if path.len() > 1 => {
            // `p.x` / `ClassName.staticField`: peel the last segment.
            let (field, object_path) = path.split_last().expect("len > 1");
            Some(AssignTarget::Field {
                object: Box::new(Expr::Name {
                    path: object_path.to_vec(),
                    span: *span,
                }),
                name: field.clone(),
            })
        }
        Expr::Index { array, index, .. } => Some(AssignTarget::Index {
            array: array.clone(),
            index: index.clone(),
        }),
        Expr::Field { object, name, .. } => Some(AssignTarget::Field {
            object: object.clone(),
            name: name.clone(),
        }),
        _ => None,
    }
}

/// Parsed member modifiers.
#[allow(clippy::struct_excessive_bools)] // mirrors Java modifiers
#[derive(Debug, Default, Clone, Copy)]
struct Modifiers {
    is_public: bool,
    is_private: bool,
    is_static: bool,
    is_final: bool,
    is_abstract: bool,
    is_protected: bool,
    is_default: bool,
}

/// Parsed class-level modifiers.
#[derive(Debug, Default, Clone, Copy)]
struct ClassModifiers {
    is_abstract: bool,
    is_public: bool,
}

/// One parsed class member.
enum Member {
    Fields(Vec<FieldDecl>),
    Method(MethodDecl),
    Init(InitBlock),
    Nested(ClassDecl),
}

/// Internal marker: a construct failed to parse and a diagnostic was
/// already recorded; the caller should recover.
struct Abort;

type Parsed<T> = Result<T, Abort>;

struct Parser<'a> {
    path: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
    /// Synthesized anonymous-class declarations, hoisted to top level.
    anon_classes: Vec<ClassDecl>,
    anon_counter: usize,
    /// Distinguishes local classes so two methods can each declare a `class
    /// Local` without their hoisted names colliding.
    local_counter: usize,
    pending_annotations: Vec<Annotation>,
}

/// `(superclass, interfaces, type arguments written on each supertype)`.
type Supertypes = (Option<String>, Vec<String>, Vec<(String, Vec<TypeRef>)>);

impl Parser<'_> {
    // ----- token helpers -----

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn peek_at(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|t| &t.kind)
    }

    fn here(&self) -> SourceSpan {
        self.tokens
            .get(self.pos)
            .map_or_else(|| self.eof_span(), |t| t.span)
    }

    fn eof_span(&self) -> SourceSpan {
        let end = self
            .tokens
            .last()
            .map_or(SourcePosition { line: 1, column: 1 }, |t| t.span.end);
        SourceSpan { start: end, end }
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn at_symbol(&self, symbol: &str) -> bool {
        matches!(self.peek(), Some(TokenKind::Symbol(s)) if *s == symbol)
    }

    fn eat_symbol(&mut self, symbol: &str) -> bool {
        if self.at_symbol(symbol) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Close one level of type arguments, consuming a single `>`.
    ///
    /// `ArrayList<ArrayList<Integer>>` ends in `>>`, which the lexer produces as
    /// ONE right-shift token — so a naive `expect('>')` at the inner level fails,
    /// and nested generics were rejected outright. The standard hand-parser trick
    /// (JLS calls it out): a `>>`/`>>>` seen where a single `>` is expected is
    /// SPLIT — one `>` closes this level, and the token is rewritten to the
    /// remaining `>`/`>>` for the enclosing level to close. `>` is never a shift
    /// operator inside a type, so this cannot mis-read `a >> b`.
    fn close_type_args(&mut self) -> Parsed<()> {
        let remainder = match self.peek() {
            Some(TokenKind::Symbol(">")) => {
                self.pos += 1;
                return Ok(());
            }
            Some(TokenKind::Symbol(">>")) => ">",
            Some(TokenKind::Symbol(">>>")) => ">>",
            _ => {
                self.error_here("expected '>' to close the type arguments");
                return Err(Abort);
            }
        };
        // Rewrite the shift token to its remaining `>`s, closing this level.
        self.tokens[self.pos].kind = TokenKind::Symbol(remainder);
        Ok(())
    }

    fn expect_symbol(&mut self, symbol: &str, context: &str) -> Parsed<()> {
        if self.eat_symbol(symbol) {
            Ok(())
        } else {
            self.error_here(format!("expected '{symbol}' {context}"));
            Err(Abort)
        }
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.peek(), Some(TokenKind::Keyword(k)) if *k == keyword)
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        if self.at_keyword(keyword) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self, context: &str) -> Parsed<(String, SourceSpan)> {
        if let Some(TokenKind::Identifier(_)) = self.peek() {
            let token = self.advance().expect("peeked");
            let TokenKind::Identifier(name) = token.kind else {
                unreachable!()
            };
            Ok((name, token.span))
        } else {
            self.error_here(format!("expected a name {context}"));
            Err(Abort)
        }
    }

    fn error_here(&mut self, message: impl Into<String>) {
        let span = self.here();
        self.diagnostics
            .push(Diagnostic::error(self.path, message, span));
    }

    fn error_at(&mut self, span: SourceSpan, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(self.path, message, span));
    }

    // ----- recovery -----

    /// Skip forward until just past a `;` or just before a `}` (or EOF),
    /// skipping over balanced `{ ... }` blocks whole.
    fn recover_to_statement_boundary(&mut self) {
        let mut depth = 0usize;
        while let Some(kind) = self.peek() {
            match kind {
                TokenKind::Symbol(";") if depth == 0 => {
                    self.pos += 1;
                    return;
                }
                TokenKind::Symbol("}") => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                    self.pos += 1;
                }
                TokenKind::Symbol("{") => {
                    depth += 1;
                    self.pos += 1;
                }
                _ => self.pos += 1,
            }
        }
    }

    // ----- grammar -----

    fn compilation_unit(&mut self) -> CompilationUnit {
        let mut imports = Vec::new();
        let mut classes = Vec::new();
        while let Some(kind) = self.peek() {
            match kind {
                TokenKind::Keyword(Keyword::Import) => {
                    if let Ok(import) = self.import_decl() {
                        imports.push(import);
                    } else {
                        self.recover_to_statement_boundary();
                    }
                }
                TokenKind::Keyword(Keyword::Package) => {
                    let span = self.here();
                    self.error_at(
                        span,
                        "package declarations are not supported by caturra; classes share one \
                         namespace",
                    );
                    self.recover_to_statement_boundary();
                }
                _ => {
                    if let Ok(class) = self.class_decl() {
                        let first = classes.len();
                        flatten_nested(class, &mut classes);
                        let mut synthesized = Vec::new();
                        for class in &mut classes[first..] {
                            for (message, span) in check_erasure_clashes(class) {
                                self.error_at(span, message);
                            }
                            for (name, span) in check_static_type_variable_use(class) {
                                self.error_at(
                                    span,
                                    format!(
                                        "non-static type variable {name} cannot be \
                                         referenced from a static context"
                                    ),
                                );
                            }
                            erase_type_vars(class, &mut synthesized);
                        }
                        // Interfaces synthesized for intersection bounds.
                        classes.extend(synthesized);
                    } else {
                        self.recover_to_statement_boundary();
                        // A stray `}` from a broken class body would stall
                        // the loop at top level; consume it and move on.
                        self.eat_symbol("}");
                    }
                }
            }
        }
        // Hoist synthesized anonymous classes to the top level.
        let anon = std::mem::take(&mut self.anon_classes);
        let mut synthesized = Vec::new();
        for class in anon {
            let first = classes.len();
            flatten_nested(class, &mut classes);
            for class in &mut classes[first..] {
                erase_type_vars(class, &mut synthesized);
            }
        }
        classes.extend(synthesized);
        CompilationUnit { imports, classes }
    }

    /// Whether the cursor sits on `Ident (. Ident)+` followed by an
    /// identifier or `<...>` — a fully qualified declaration.
    fn at_qualified_declaration(&self) -> bool {
        if !matches!(self.peek(), Some(TokenKind::Identifier(_))) {
            return false;
        }
        let mut i = 1;
        let mut segments = 1;
        while matches!(self.peek_at(i), Some(TokenKind::Symbol(".")))
            && matches!(self.peek_at(i + 1), Some(TokenKind::Identifier(_)))
        {
            i += 2;
            segments += 1;
        }
        if segments < 2 {
            return false;
        }
        matches!(self.peek_at(i), Some(TokenKind::Identifier(_)))
            || (matches!(self.peek_at(i), Some(TokenKind::Symbol("<")))
                && matches!(self.peek_at(i + 1), Some(TokenKind::Identifier(_))))
    }

    /// `import a.b.C;` or `import a.b.*;`.
    fn import_decl(&mut self) -> Parsed<ImportDecl> {
        let start = self.here();
        if !self.eat_keyword(Keyword::Import) {
            return Err(Abort);
        }
        let is_static = self.eat_keyword(Keyword::Static);
        let mut path = Vec::new();
        let mut wildcard = false;
        loop {
            if self.at_symbol("*") {
                self.pos += 1;
                wildcard = true;
                break;
            }
            let (segment, _) = self.expect_ident("in the import path")?;
            path.push(segment);
            if !self.eat_symbol(".") {
                break;
            }
        }
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        self.expect_symbol(";", "after the import")?;
        if path.is_empty() {
            self.error_at(span, "expected a class name after 'import'");
            return Err(Abort);
        }
        Ok(ImportDecl {
            path,
            wildcard,
            is_static,
            span,
        })
    }

    /// Modifier keywords before a class or member. Returns
    /// `(is_public, is_static)`; other modifiers parse and are ignored
    /// for now.
    fn modifiers(&mut self) -> Modifiers {
        let mut modifiers = Modifiers::default();
        self.pending_annotations.clear();
        loop {
            match self.peek() {
                Some(TokenKind::Keyword(Keyword::Public)) => {
                    modifiers.is_public = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Static)) => {
                    modifiers.is_static = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Private)) => {
                    modifiers.is_private = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Final)) => {
                    modifiers.is_final = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Abstract)) => {
                    modifiers.is_abstract = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Protected)) => {
                    modifiers.is_protected = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Default)) => {
                    modifiers.is_default = true;
                    self.pos += 1;
                }
                Some(TokenKind::Symbol("@")) => self.skip_annotation(),
                _ => return modifiers,
            }
        }
    }

    /// Skip an annotation (`@Override`, `@Deprecated`,
    /// `@SuppressWarnings("x")`). caturra does not act on annotations;
    /// this lets annotated code compile.
    fn skip_annotation(&mut self) {
        self.pos += 1; // '@'
        let mut name = String::new();
        while let Some(TokenKind::Identifier(segment)) = self.peek() {
            name.clone_from(segment);
            self.pos += 1;
            if !self.eat_symbol(".") {
                break;
            }
        }
        let mut int_arg = None;
        let mut str_arg = None;
        if self.at_symbol("(") {
            match self.peek_at(1) {
                Some(TokenKind::IntLiteral(value)) => int_arg = i32::try_from(*value).ok(),
                Some(TokenKind::StringLiteral(text)) => str_arg = Some(text.clone()),
                _ => {}
            }
            let mut depth = 0usize;
            loop {
                match self.peek() {
                    Some(TokenKind::Symbol("(")) => depth += 1,
                    Some(TokenKind::Symbol(")")) => {
                        depth -= 1;
                        if depth == 0 {
                            self.pos += 1;
                            break;
                        }
                    }
                    None => break,
                    _ => {}
                }
                self.pos += 1;
            }
        }
        if !name.is_empty() {
            self.pending_annotations.push(Annotation {
                name,
                int_arg,
                str_arg,
            });
        }
    }

    fn class_decl(&mut self) -> Parsed<ClassDecl> {
        let start = self.here();
        let modifiers = self.class_modifiers();
        self.type_after_modifiers(start, modifiers.is_abstract, modifiers.is_public)
    }

    /// The `extends` / `implements` clauses: `(superclass, interfaces)`.
    /// An interface's `extends` is a list of interfaces, and an interface
    /// may not `implement`.
    fn supertypes(&mut self, is_interface: bool) -> Parsed<Supertypes> {
        let mut superclass = None;
        let mut interfaces = Vec::new();
        // The type arguments written on each supertype are RECORDED rather
        // than skipped: `extends Box<String>` is what makes it checkable that
        // a subclass may stand in for `Box<String>` and not for `Box<Integer>`.
        let mut supertype_args: Vec<(String, Vec<TypeRef>)> = Vec::new();
        let record = |name: &str, args: Vec<TypeRef>, out: &mut Vec<(String, Vec<TypeRef>)>| {
            if !args.is_empty() {
                out.push((name.to_owned(), args));
            }
        };
        if self.eat_keyword(Keyword::Extends) {
            if is_interface {
                loop {
                    let (parent, args) = self.supertype_ref()?;
                    record(&parent, args, &mut supertype_args);
                    interfaces.push(parent);
                    if !self.eat_symbol(",") {
                        break;
                    }
                }
            } else {
                let (parent, args) = self.supertype_ref()?;
                record(&parent, args, &mut supertype_args);
                superclass = Some(parent);
            }
        }
        if self.eat_keyword(Keyword::Implements) {
            if is_interface {
                self.error_here("interfaces cannot implement");
                return Err(Abort);
            }
            loop {
                let (parent, args) = self.supertype_ref()?;
                record(&parent, args, &mut supertype_args);
                interfaces.push(parent);
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }
        Ok((superclass, interfaces, supertype_args))
    }

    /// A supertype in an `extends`/`implements` clause: its name and the type
    /// arguments written on it (empty when raw).
    fn supertype_ref(&mut self) -> Parsed<(String, Vec<TypeRef>)> {
        let start = self.pos;
        let (name, _) = self.expect_ident("after 'extends'")?;
        if !self.at_symbol("<") {
            return Ok((name, Vec::new()));
        }
        // Re-read the whole thing with the type parser, which already knows
        // the argument grammar (including wildcards).
        self.pos = start;
        if let Ok(TypeRef::Generic { base, args }) = self.type_ref() {
            Ok((base, args))
        } else {
            // A form the type parser does not model: skip the arguments, which
            // is what happened before they were recorded at all, and leave the
            // supertype raw (treated as unchecked, like javac).
            self.pos = start;
            let (name, _) = self.expect_ident("after 'extends'")?;
            self.skip_type_args();
            Ok((name, Vec::new()))
        }
    }

    /// Parse a class/interface/enum whose modifiers were already
    /// consumed (shared by top-level and nested declarations).
    fn type_after_modifiers(
        &mut self,
        start: SourceSpan,
        is_abstract_modifier: bool,
        is_public: bool,
    ) -> Parsed<ClassDecl> {
        if self.at_keyword(Keyword::Enum) {
            let mut decl = self.enum_decl(start)?;
            decl.is_public = is_public;
            return Ok(decl);
        }
        let is_interface = self.eat_keyword(Keyword::Interface);
        if !is_interface && !self.eat_keyword(Keyword::Class) {
            self.error_here("expected a class declaration");
            return Err(Abort);
        }
        let (name, name_span) = self.expect_ident("for the class")?;
        let type_params = self.parse_type_params()?;

        let (superclass, interfaces, supertype_args) = self.supertypes(is_interface)?;

        self.expect_symbol("{", "to open the class body")?;
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut init_blocks = Vec::new();
        let mut nested = Vec::new();
        // Monotonic source-order counter shared by fields and blocks so
        // initialization runs in textual order.
        let mut order = 0usize;
        while !self.at_symbol("}") {
            if self.peek().is_none() {
                self.error_at(
                    name_span,
                    format!("class '{name}' is missing its closing '}}'"),
                );
                break;
            }
            // A stray `;` between members is an empty declaration (JLS §8.1.6),
            // legal — e.g. after a nested `enum E { ... };`.
            if self.eat_symbol(";") {
                continue;
            }
            if let Ok(member) = self.member(&name, is_interface) {
                match member {
                    Member::Method(method) => methods.push(method),
                    Member::Fields(mut declared) => {
                        for field in &mut declared {
                            field.order = order;
                            order += 1;
                        }
                        fields.append(&mut declared);
                    }
                    Member::Init(mut block) => {
                        block.order = order;
                        order += 1;
                        init_blocks.push(block);
                    }
                    Member::Nested(decl) => nested.push(decl),
                }
            } else {
                self.recover_to_statement_boundary();
                self.eat_symbol(";");
            }
        }
        self.eat_symbol("}");

        // Every field declared in an interface is implicitly `public static
        // final` (JLS §9.3), so `Iface.CONST` reads a constant, not a
        // non-static field.
        if is_interface {
            for field in &mut fields {
                field.is_static = true;
                field.is_final = true;
                field.is_private = false;
            }
        }

        Ok(ClassDecl {
            name,
            is_public,
            is_nested: false,
            enclosing: None,
            superclass,
            interfaces,
            supertype_args,
            is_abstract: is_abstract_modifier || is_interface,
            is_interface,
            is_enum: false,
            is_anonymous: false,
            is_local: false,
            is_inner: false,
            type_params,
            fields,
            methods,
            init_blocks,
            nested,
            span: SourceSpan {
                start: start.start,
                end: name_span.end,
            },
        })
    }

    /// Parse an `enum` declaration and desugar it to an ordinary class
    /// with synthesized constant fields, a name/ordinal-storing
    /// constructor, and `values`/`valueOf`/`ordinal`/`name`/`toString`.
    #[allow(clippy::too_many_lines)] // one enum-body parse
    fn enum_decl(&mut self, start: SourceSpan) -> Parsed<ClassDecl> {
        self.pos += 1; // 'enum'
        let (name, name_span) = self.expect_ident("for the enum")?;

        let mut interfaces = Vec::new();
        if self.eat_keyword(Keyword::Implements) {
            loop {
                let (parent, _) = self.expect_ident("after 'implements'")?;
                self.skip_type_args();
                interfaces.push(parent);
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }

        self.expect_symbol("{", "to open the enum body")?;

        // Constants: `NAME`, `NAME(args)`, comma-separated, ended by
        // `;` (if members follow) or `}`.
        let mut constants: Vec<EnumConstant> = Vec::new();
        while let Some(TokenKind::Identifier(_)) = self.peek() {
            let (const_name, const_span) = self.expect_ident("for the enum constant")?;
            let args = if self.at_symbol("(") {
                self.arguments()?
            } else {
                Vec::new()
            };
            // A constant-specific body (`PLUS { int apply(...) {...} }`) becomes
            // a synthesized subclass of the enum; the constant is an instance of
            // that subclass rather than of the enum itself.
            let body = if self.at_symbol("{") {
                Some(self.synth_class_from_body(&name, const_span)?)
            } else {
                None
            };
            constants.push(EnumConstant {
                name: const_name,
                args,
                body,
                span: const_span,
            });
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.eat_symbol(";"); // optional separator before members

        // Ordinary members after the constants.
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut init_blocks = Vec::new();
        let mut nested = Vec::new();
        let mut order = 0usize;
        while !self.at_symbol("}") {
            if self.peek().is_none() {
                self.error_at(
                    name_span,
                    format!("enum '{name}' is missing its closing '}}'"),
                );
                break;
            }
            if let Ok(member) = self.member(&name, false) {
                match member {
                    Member::Method(method) => methods.push(method),
                    Member::Fields(mut declared) => {
                        for field in &mut declared {
                            field.order = order;
                            order += 1;
                        }
                        fields.append(&mut declared);
                    }
                    Member::Init(mut block) => {
                        block.order = order;
                        order += 1;
                        init_blocks.push(block);
                    }
                    Member::Nested(decl) => nested.push(decl),
                }
            } else {
                self.recover_to_statement_boundary();
                self.eat_symbol(";");
            }
        }
        self.eat_symbol("}");

        // `Enum`'s `name`/`ordinal`/`equals`/`hashCode`/`compareTo`/
        // `getDeclaringClass` are FINAL, so an enum may not override them
        // (JLS §8.9). caturra used the override silently.
        for method in &methods {
            if !method.is_static
                && matches!(
                    method.name.as_str(),
                    "name" | "ordinal" | "equals" | "hashCode" | "compareTo" | "getDeclaringClass"
                )
            {
                self.error_at(
                    method.span,
                    format!(
                        "{}() in {name} cannot override {}() in Enum",
                        method.name, method.name
                    ),
                );
            }
        }

        let span = SourceSpan {
            start: start.start,
            end: name_span.end,
        };
        Ok(desugar_enum(
            name,
            interfaces,
            constants,
            fields,
            methods,
            init_blocks,
            nested,
            span,
        ))
    }

    /// Class-level modifiers (public/abstract/final tracked loosely).
    fn class_modifiers(&mut self) -> ClassModifiers {
        let mut modifiers = ClassModifiers::default();
        loop {
            match self.peek() {
                Some(TokenKind::Keyword(Keyword::Abstract)) => {
                    modifiers.is_abstract = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Public)) => {
                    modifiers.is_public = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Final)) => {
                    self.pos += 1;
                }
                Some(TokenKind::Symbol("@")) => self.skip_annotation(),
                _ => return modifiers,
            }
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per member kind
    /// The modifier and body rules for an INTERFACE method (JLS §9.4): no
    /// `protected`/`final`; `static` and `default` are mutually exclusive; a
    /// `static`/`default`/`private` method must have a body while a plain
    /// (abstract) one must not; and a `default` method cannot override a member
    /// of `java.lang.Object`. Each was silently accepted before.
    fn validate_interface_method(
        &mut self,
        modifiers: Modifiers,
        iface_name: &str,
        name: &str,
        params_len: usize,
        has_body: bool,
        span: SourceSpan,
    ) {
        if modifiers.is_protected {
            self.error_at(span, "modifier protected not allowed here");
        }
        if modifiers.is_final {
            self.error_at(span, "modifier final not allowed here");
        }
        if modifiers.is_static && modifiers.is_default {
            self.error_at(span, "illegal combination of modifiers: static and default");
        }
        let concrete = modifiers.is_static || modifiers.is_default || modifiers.is_private;
        if has_body && !concrete {
            self.error_at(span, "interface abstract methods cannot have body");
        }
        if !has_body && concrete {
            self.error_at(span, "missing method body, or declare abstract");
        }
        // A `default` method cannot override `Object`'s `toString`/`hashCode`/
        // `equals` (JLS §9.4.1.2).
        let overrides_object = modifiers.is_default
            && matches!(
                (name, params_len),
                ("toString" | "hashCode", 0) | ("equals", 1)
            );
        if overrides_object {
            self.error_at(
                span,
                format!(
                    "default method {name} in interface {iface_name} \
                     overrides a member of java.lang.Object"
                ),
            );
        }
    }

    #[allow(clippy::too_many_lines)] // one member-declaration parse: field / method / ctor / nested
    fn member(&mut self, class_name: &str, is_interface: bool) -> Parsed<Member> {
        let start = self.here();
        let modifiers = self.modifiers();
        let annotations = std::mem::take(&mut self.pending_annotations);

        // Nested type declaration: `class`/`interface`/`enum`.
        if matches!(
            self.peek(),
            Some(TokenKind::Keyword(
                Keyword::Class | Keyword::Interface | Keyword::Enum
            ))
        ) {
            // A nested type may be `public`; the file-name rule is top-level
            // only (JLS §7.6), so this is recorded and never checked.
            let mut nested =
                self.type_after_modifiers(start, modifiers.is_abstract, modifiers.is_public)?;
            // A non-static nested CLASS is an inner class, bound to an enclosing
            // instance. Interfaces and enums are implicitly static.
            nested.is_inner = !modifiers.is_static && !nested.is_interface && !nested.is_enum;
            return Ok(Member::Nested(nested));
        }

        // Initializer block: `static { ... }` or a bare `{ ... }`.
        if self.at_symbol("{") {
            self.pos += 1; // '{'
            let body = self.block_body();
            let span = SourceSpan {
                start: start.start,
                end: self.here().start,
            };
            return Ok(Member::Init(InitBlock {
                is_static: modifiers.is_static,
                body,
                order: 0,
                span,
            }));
        }

        // Constructor: `ClassName(...)` with no return type.
        if let Some(TokenKind::Identifier(name)) = self.peek()
            && name == class_name
            && matches!(self.peek_at(1), Some(TokenKind::Symbol("(")))
        {
            let (name, name_span) = self.expect_ident("for the constructor")?;
            let (params, throws, body) = self.method_rest(name_span, false)?;
            return Ok(Member::Method(MethodDecl {
                name,
                is_static: false,
                is_public: modifiers.is_public,
                is_private: modifiers.is_private,
                is_final: modifiers.is_final,
                is_constructor: true,
                is_abstract: false,
                type_params: Vec::new(),
                infer_return: None,
                return_type: TypeRef::Void,
                params,
                body: body.unwrap_or_default(),
                annotations: annotations.clone(),
                throws,
                span: SourceSpan {
                    start: start.start,
                    end: name_span.end,
                },
            }));
        }

        // Generic method: `<T> ReturnType method(...)`.
        let method_type_params = self.parse_type_params()?;
        let member_type = self.type_ref()?;
        let (name, name_span) = self.expect_ident("for the class member")?;

        if !self.at_symbol("(") {
            if !method_type_params.is_empty() {
                self.error_at(name_span, "type parameters are only allowed on methods");
            }
            // Field declaration(s): `int x = 1, y;`.
            let mut fields = Vec::new();
            let mut current = (name, name_span);
            loop {
                // `private int f[];` — the brackets bind to this name only.
                let dims = self.trailing_array_dims();
                let init = if self.eat_symbol("=") {
                    if self.at_symbol("{") {
                        Some(self.array_literal()?)
                    } else {
                        Some(self.expression()?)
                    }
                } else {
                    // An interface field is implicitly `public static final`
                    // (JLS §9.3), so it MUST have an initializer — `interface F
                    // { int X; }` is javac's "= expected", not a valid field.
                    if is_interface {
                        self.error_at(current.1, "= expected");
                    }
                    None
                };
                fields.push(FieldDecl {
                    name: current.0,
                    ty: crate::ast::array_of(member_type.clone(), dims),
                    is_static: modifiers.is_static,
                    is_private: modifiers.is_private,
                    is_final: modifiers.is_final,
                    init,
                    order: 0,
                    span: current.1,
                });
                if !self.eat_symbol(",") {
                    break;
                }
                current = self.expect_ident("for the field")?;
            }
            self.expect_symbol(";", "to end the field declaration")?;
            return Ok(Member::Fields(fields));
        }

        let (params, throws, body) = self.method_rest(name_span, true)?;
        let is_abstract = body.is_none();
        if is_interface {
            self.validate_interface_method(
                modifiers,
                class_name,
                &name,
                params.len(),
                body.is_some(),
                name_span,
            );
        }
        Ok(Member::Method(MethodDecl {
            name,
            is_static: modifiers.is_static,
            is_public: modifiers.is_public,
            is_private: modifiers.is_private,
            is_final: modifiers.is_final,
            is_constructor: false,
            is_abstract,
            type_params: method_type_params,
            infer_return: None,
            return_type: member_type,
            params,
            body: body.unwrap_or_default(),
            annotations,
            throws,
            span: SourceSpan {
                start: start.start,
                end: name_span.end,
            },
        }))
    }

    /// C-style array brackets written *after* a declarator's name, as in
    /// `String args[]` or `int a[][]`. Each pair adds a dimension to that one
    /// declarator (JLS §10.2), which is why `int a[], b;` makes only `a` an
    /// array. Returns how many pairs were consumed.
    fn trailing_array_dims(&mut self) -> usize {
        let mut dims = 0;
        while self.at_symbol("[") && matches!(self.peek_at(1), Some(TokenKind::Symbol("]"))) {
            self.pos += 2;
            dims += 1;
        }
        dims
    }

    /// Parameter list and body, shared by methods and constructors.
    /// With `allow_abstract`, a `;` instead of a body yields `None`.
    #[allow(clippy::type_complexity)]
    fn method_rest(
        &mut self,
        name_span: SourceSpan,
        allow_abstract: bool,
    ) -> Parsed<(Vec<Param>, Vec<String>, Option<Vec<Stmt>>)> {
        self.expect_symbol("(", "to open the parameter list")?;
        let mut params = Vec::new();
        if !self.at_symbol(")") {
            loop {
                // `void m(final int v)` — legal on any parameter, and common in
                // code that leans on effectively-final capture.
                let is_final = self.eat_keyword(Keyword::Final);
                let mut ty = self.type_ref()?;
                // Varargs: `Type... name` — the parameter is an array.
                let is_varargs = self.eat_symbol("...");
                if is_varargs {
                    ty = TypeRef::Array(Box::new(ty));
                }
                let (param_name, param_span) = self.expect_ident("for the parameter")?;
                // `void m(String args[])` — the old C-style spelling of `String[]`.
                let dims = self.trailing_array_dims();
                if dims > 0 && is_varargs {
                    self.error_at(param_span, "a varargs parameter cannot also carry '[]'");
                }
                params.push(Param {
                    ty: crate::ast::array_of(ty, dims),
                    name: param_name,
                    is_varargs,
                    is_final,
                });
                if is_varargs {
                    // A varargs parameter must be last.
                    break;
                }
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }
        self.expect_symbol(")", "to close the parameter list")?;

        // `throws FileNotFoundException, ...` — recorded for JLS §11.2
        // checked-exception enforcement.
        let mut throws = Vec::new();
        if self.eat_keyword(Keyword::Throws) {
            loop {
                let (mut name, _) = self.expect_ident("after 'throws'")?;
                // Qualified exception names: `throws java.io.IOException`.
                while self.at_symbol(".")
                    && matches!(self.peek_at(1), Some(TokenKind::Identifier(_)))
                {
                    self.pos += 1;
                    let (segment, _) = self.expect_ident("in the qualified exception")?;
                    name.push('.');
                    name.push_str(&segment);
                }
                throws.push(name);
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }

        if self.eat_symbol(";") {
            if !allow_abstract {
                self.error_at(name_span, "constructors need a body");
                return Err(Abort);
            }
            return Ok((params, throws, None));
        }
        self.expect_symbol("{", "to open the method body")?;
        let body = self.block_body();
        Ok((params, throws, Some(body)))
    }

    /// Skip a `<...>` type-argument list on a supertype reference
    /// (`implements Comparable<Foo>`), balancing nested `<>`. Erased.
    fn skip_type_args(&mut self) {
        if !self.at_symbol("<") {
            return;
        }
        let mut depth = 0i32;
        while let Some(kind) = self.peek() {
            match kind {
                TokenKind::Symbol("<") => depth += 1,
                TokenKind::Symbol(">") => depth -= 1,
                TokenKind::Symbol(">>") => depth -= 2,
                // `>>>` closes three levels at once (`Map<K, List<Set<V>>>`).
                TokenKind::Symbol(">>>") => depth -= 3,
                _ => {}
            }
            self.pos += 1;
            if depth <= 0 {
                break;
            }
        }
    }

    /// Parse a `<T, U extends Bound, ...>` type-parameter list. Each
    /// parameter keeps its leftmost `extends` bound (further `& Other`
    /// bounds are parsed and discarded — erasure uses the leftmost, per
    /// JLS §4.6); an unbounded parameter erases to `Object`.
    fn parse_type_params(&mut self) -> Parsed<Vec<TypeParam>> {
        if !self.at_symbol("<") {
            return Ok(Vec::new());
        }
        self.pos += 1; // '<'
        let mut params = Vec::new();
        if !self.at_symbol(">") {
            loop {
                let (name, _) = self.expect_ident("for the type parameter")?;
                let mut bound = None;
                let mut extra_bounds = Vec::new();
                if self.eat_keyword(Keyword::Extends) {
                    bound = Some(self.type_ref()?);
                    // `& Other` intersection bounds (JLS §4.4).
                    while self.eat_symbol("&") {
                        extra_bounds.push(self.type_ref()?);
                    }
                }
                params.push(TypeParam {
                    name,
                    bound,
                    extra_bounds,
                });
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }
        self.expect_symbol(">", "to close the type parameters")?;
        Ok(params)
    }

    fn type_ref(&mut self) -> Parsed<TypeRef> {
        let mut ty = match self.peek() {
            Some(TokenKind::Keyword(Keyword::Void)) => {
                self.pos += 1;
                TypeRef::Void
            }
            Some(TokenKind::Keyword(Keyword::Int)) => {
                self.pos += 1;
                TypeRef::Int
            }
            Some(TokenKind::Keyword(Keyword::Double)) => {
                self.pos += 1;
                TypeRef::Double
            }
            Some(TokenKind::Keyword(Keyword::Boolean)) => {
                self.pos += 1;
                TypeRef::Boolean
            }
            Some(TokenKind::Keyword(Keyword::Char)) => {
                self.pos += 1;
                TypeRef::Char
            }
            Some(TokenKind::Keyword(Keyword::Long)) => {
                self.pos += 1;
                TypeRef::Long
            }
            Some(TokenKind::Keyword(Keyword::Float)) => {
                self.pos += 1;
                TypeRef::Float
            }
            Some(TokenKind::Keyword(Keyword::Short)) => {
                self.pos += 1;
                TypeRef::Short
            }
            Some(TokenKind::Keyword(Keyword::Byte)) => {
                self.pos += 1;
                TypeRef::Byte
            }
            Some(TokenKind::Identifier(_)) => {
                let (mut name, _) = self.expect_ident("for the type")?;
                // Fully qualified names: `java.util.Scanner`.
                while self.at_symbol(".")
                    && matches!(self.peek_at(1), Some(TokenKind::Identifier(_)))
                {
                    self.pos += 1;
                    let (segment, _) = self.expect_ident("in the qualified type")?;
                    name.push('.');
                    name.push_str(&segment);
                }
                if self.at_symbol("<") {
                    self.pos += 1;
                    let mut args = Vec::new();
                    if !self.at_symbol(">") {
                        loop {
                            if self.at_symbol("?") {
                                // Wildcard `?` / `? extends T` / `? super T`.
                                // Erasure keeps only the raw class, but the
                                // variance and bound decide argument
                                // applicability (`List<Integer>` matches
                                // `List<? extends Number>`), so preserve them in
                                // a sentinel name (see `ast::wildcard_type_name`).
                                self.pos += 1;
                                let (variance, bound) = match self.peek() {
                                    Some(TokenKind::Keyword(Keyword::Extends)) => {
                                        self.pos += 1;
                                        ('+', wildcard_bound_name(&self.type_ref()?))
                                    }
                                    Some(TokenKind::Keyword(Keyword::Super)) => {
                                        self.pos += 1;
                                        ('-', wildcard_bound_name(&self.type_ref()?))
                                    }
                                    _ => ('?', String::new()),
                                };
                                args.push(TypeRef::Named(crate::ast::wildcard_type_name(
                                    variance, &bound,
                                )));
                            } else {
                                args.push(self.type_ref()?);
                            }
                            if !self.eat_symbol(",") {
                                break;
                            }
                        }
                    }
                    self.close_type_args()?;
                    TypeRef::Generic { base: name, args }
                } else {
                    TypeRef::Named(name)
                }
            }
            _ => {
                self.error_here("expected a type");
                return Err(Abort);
            }
        };
        while self.at_symbol("[") {
            self.pos += 1;
            self.expect_symbol("]", "to complete the array type")?;
            ty = TypeRef::Array(Box::new(ty));
        }
        Ok(ty)
    }

    fn block_body(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        // Local classes declared in this block: (index in `statements` where
        // the declaration sat, its source name, its mangled hoisted decl).
        let mut locals: Vec<(usize, String, ClassDecl)> = Vec::new();
        while !self.at_symbol("}") {
            if self.peek().is_none() {
                self.error_here("expected '}' to close the block");
                break;
            }
            // A local class (`class C { ... }` in statement position) is not a
            // runtime statement: it is mangled, hoisted to the top level, and
            // references to it in the rest of this block are rewritten below.
            if self.at_local_class_start() {
                match self.local_class_decl() {
                    Ok((name, decl)) => locals.push((statements.len(), name, decl)),
                    Err(Abort) => self.recover_to_statement_boundary(),
                }
                continue;
            }
            match self.statement() {
                Ok(Some(stmt)) => statements.push(stmt),
                // An empty statement (`;`, JLS §14.6) parses to nothing. It
                // is not a parse error, and must not reach the recovery
                // below — that skips to the next `;`, which would silently
                // swallow the statement after it.
                Ok(None) => {}
                Err(Abort) => self.recover_to_statement_boundary(),
            }
        }
        self.eat_symbol("}");

        // A local class is in scope from its declaration to the end of the
        // block (JLS §6.3), so rewrite its source name to the mangled one in
        // the statements that follow it, in its own body (recursion), and in
        // any later local class's body — then hoist it to the top level.
        for k in 0..locals.len() {
            let (at, name, mangled) = (locals[k].0, locals[k].1.clone(), locals[k].2.name.clone());
            rename_class_in_stmts(&mut statements[at..], &name, &mangled);
            for later in &mut locals[k..] {
                rename_class_in_class(&mut later.2, &name, &mangled);
            }
        }
        for (_, _, decl) in locals {
            self.anon_classes.push(decl);
        }
        statements
    }

    /// Whether the upcoming tokens begin a local class: optional
    /// `final`/`abstract`/`strictfp` modifiers, then `class`. (A local
    /// `interface` or `enum` is not legal Java 11 and stays an error.)
    fn at_local_class_start(&self) -> bool {
        let mut i = self.pos;
        loop {
            match self.tokens.get(i).map(|t| &t.kind) {
                Some(TokenKind::Keyword(
                    Keyword::Final | Keyword::Abstract | Keyword::Strictfp,
                )) => i += 1,
                Some(TokenKind::Keyword(Keyword::Class)) => return true,
                _ => return false,
            }
        }
    }

    /// Parse a local class, giving it a mangled top-level name. Returns its
    /// source name (for rewriting references in the enclosing block) and the
    /// hoisted declaration.
    fn local_class_decl(&mut self) -> Parsed<(String, ClassDecl)> {
        let start = self.here();
        let mut is_abstract = false;
        while let Some(TokenKind::Keyword(kw)) = self.peek() {
            match kw {
                Keyword::Abstract => is_abstract = true,
                Keyword::Final | Keyword::Strictfp => {}
                _ => break,
            }
            self.pos += 1;
        }
        let mut decl = self.type_after_modifiers(start, is_abstract, false)?;
        let name = decl.name.clone();
        self.local_counter += 1;
        decl.name = format!("{name}$Local{}", self.local_counter);
        decl.is_local = true;
        Ok((name, decl))
    }

    /// Parse one statement. `Ok(None)` means an empty statement (`;`).
    fn statement(&mut self) -> Parsed<Option<Stmt>> {
        if self.eat_symbol(";") {
            return Ok(None);
        }
        // A label: `identifier : statement`. Unambiguous at statement
        // start — a bare expression there can't have a top-level `:`.
        if let Some(TokenKind::Identifier(name)) = self.peek()
            && self.peek_at(1) == Some(&TokenKind::Symbol(":"))
        {
            let label = name.clone();
            let start = self.here();
            self.pos += 2; // identifier + ':'
            let Some(body) = self.statement()? else {
                self.error_here("a label must be followed by a statement");
                return Err(Abort);
            };
            let span = SourceSpan {
                start: start.start,
                end: self.here().start,
            };
            return Ok(Some(Stmt::Labeled {
                label,
                body: Box::new(body),
                span,
            }));
        }
        if self.eat_symbol("{") {
            return Ok(Some(Stmt::Block(self.block_body())));
        }

        match self.peek() {
            Some(TokenKind::Keyword(Keyword::If)) => return self.if_statement().map(Some),
            Some(TokenKind::Keyword(Keyword::While)) => {
                return self.while_statement().map(Some);
            }
            Some(TokenKind::Keyword(Keyword::Do)) => {
                return self.do_while_statement().map(Some);
            }
            Some(TokenKind::Keyword(Keyword::For)) => return self.for_statement().map(Some),
            Some(TokenKind::Keyword(Keyword::Break | Keyword::Continue)) => {
                return self.break_or_continue().map(Some);
            }
            Some(TokenKind::Keyword(Keyword::Return)) => {
                return self.return_statement().map(Some);
            }
            Some(TokenKind::Keyword(Keyword::Try)) => return self.try_statement().map(Some),
            Some(TokenKind::Keyword(Keyword::Switch)) => {
                return self.switch_statement().map(Some);
            }
            Some(TokenKind::Keyword(Keyword::Throw)) => {
                return self.throw_statement().map(Some);
            }
            Some(TokenKind::Keyword(Keyword::Assert)) => {
                return self.assert_statement().map(Some);
            }
            _ => {}
        }

        // Constructor chaining: `super(args);` / `this(args);`.
        if matches!(
            self.peek(),
            Some(TokenKind::Keyword(Keyword::Super | Keyword::This))
        ) && matches!(self.peek_at(1), Some(TokenKind::Symbol("(")))
        {
            let span = self.here();
            let is_super = self.at_keyword(Keyword::Super);
            self.pos += 1;
            let args = self.arguments()?;
            self.expect_symbol(";", "to end the constructor call")?;
            return Ok(Some(if is_super {
                Stmt::SuperCall { args, span }
            } else {
                Stmt::ThisCall { args, span }
            }));
        }

        // `super.method(...)` / `this.field` etc. are expression
        // statements, not the unsupported bare keywords.
        let member_access = matches!(self.peek_at(1), Some(TokenKind::Symbol(".")));
        if let Some(TokenKind::Keyword(keyword)) = self.peek()
            && !((matches!(keyword, Keyword::Super | Keyword::This)) && member_access)
            && let Some(message) = statement_start_error(*keyword)
        {
            self.error_here(message);
            return Err(Abort);
        }

        if self.at_declaration_start() {
            return self.local_declaration().map(Some);
        }

        let stmt = self.simple_statement()?;
        self.expect_symbol(";", "to end the statement")?;
        Ok(Some(stmt))
    }

    /// An assignment, `++`/`--`, or call statement, WITHOUT the
    /// trailing `;` (shared by statements and `for` headers).
    fn simple_statement(&mut self) -> Parsed<Stmt> {
        // Prefix `++x` / `--x` (also `++a[i]`).
        if self.at_symbol("++") || self.at_symbol("--") {
            let start = self.here();
            let increment = self.at_symbol("++");
            self.pos += 1;
            let operand = self.postfix_expression()?;
            let Some(target) = assignment_target(&operand) else {
                self.error_at(operand.span(), "++/-- can only be applied to a variable");
                return Err(Abort);
            };
            return Ok(increment_statement(
                target,
                increment,
                start.start,
                operand.span().end,
            ));
        }

        let expr = self.expression()?;

        // A bare assignment statement (`x = e;`) stays a `Stmt::Assign` — the
        // expression parser produced an `Expr::Assign`, which is unwrapped here
        // so nothing downstream sees an assignment expression in statement
        // position and its definite-assignment analysis is unchanged.
        if let Expr::Assign {
            target,
            op,
            value,
            span,
        } = expr
        {
            // `simple_statement` never consumes the trailing `;` — its caller and
            // the `for` header do — so unwrap without eating one.
            return Ok(Stmt::Assign {
                target,
                op,
                value: *value,
                span,
            });
        }

        // `x++;` parses as a postfix expression; as a statement it
        // lowers to the compound assignment like before.
        if let Expr::IncDec {
            target,
            increment,
            span,
            ..
        } = &expr
        {
            if let Some(assign_target) = assignment_target(target) {
                return Ok(increment_statement(
                    assign_target,
                    *increment,
                    span.start,
                    span.end,
                ));
            }
            self.error_at(
                target.span(),
                "++/-- can only be applied to a variable or array element",
            );
            return Err(Abort);
        }

        if self.at_symbol("++") || self.at_symbol("--") {
            let increment = self.at_symbol("++");
            self.pos += 1;
            let Some(target) = assignment_target(&expr) else {
                self.error_at(
                    expr.span(),
                    "++/-- can only be applied to a variable or array element",
                );
                return Err(Abort);
            };
            return Ok(increment_statement(
                target,
                increment,
                expr.span().start,
                expr.span().end,
            ));
        }

        // JLS §14.8 statement expressions: calls, and class instance creation
        // (`new Foo();`). Array creation (`new int[3];`) is NOT one, and javac
        // rejects it — so `Expr::NewArray` still falls through to the error.
        if !matches!(
            expr,
            Expr::Call { .. } | Expr::SuperMethodCall { .. } | Expr::NewObject { .. }
        ) {
            self.error_at(expr.span(), "this expression is not a statement in Java");
            return Err(Abort);
        }
        Ok(Stmt::Expr(expr))
    }

    // ----- control flow -----

    /// The body of an `if`/loop: one statement. Bare declarations are
    /// illegal there in Java; empty statements become empty blocks.
    fn embedded_statement(&mut self, context: &str) -> Parsed<Stmt> {
        let start = self.here();
        match self.statement()? {
            None => Ok(Stmt::Block(Vec::new())),
            Some(Stmt::LocalDecl { .. }) => {
                self.error_at(
                    start,
                    format!("a variable declaration as the body of {context} needs braces {{ }}"),
                );
                Err(Abort)
            }
            Some(stmt) => Ok(stmt),
        }
    }

    /// `( condition )`. An assignment is a legal condition in Java
    /// (`while ((line = in.readLine()) != null)`), so the parse simply takes
    /// the expression; a non-boolean condition (`if (x = 2)`) is a type error
    /// codegen reports, exactly as javac does.
    fn paren_condition(&mut self, context: &str) -> Parsed<Expr> {
        self.expect_symbol("(", &format!("after '{context}'"))?;
        let cond = self.expression()?;
        self.expect_symbol(")", "to close the condition")?;
        Ok(cond)
    }

    fn if_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'if'
        let cond = self.paren_condition("if")?;
        let then = Box::new(self.embedded_statement("an if")?);
        let els = if self.eat_keyword(Keyword::Else) {
            Some(Box::new(self.embedded_statement("an else")?))
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then,
            els,
            span: start,
        })
    }

    /// `switch (selector) { case k: ... default: ... }`.
    #[allow(clippy::too_many_lines)] // one grammar production
    fn switch_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'switch'
        self.expect_symbol("(", "after 'switch'")?;
        let selector = self.expression()?;
        self.expect_symbol(")", "after the switch selector")?;
        self.expect_symbol("{", "to open the switch body")?;

        let mut arms: Vec<SwitchArm> = Vec::new();
        while !self.at_symbol("}") && self.peek().is_some() {
            // One arm: stacked labels, then statements.
            let arm_start = self.here();
            let mut labels = Vec::new();
            loop {
                if self.eat_keyword(Keyword::Case) {
                    let value = self.expression()?;
                    self.expect_symbol(":", "after the case value")?;
                    labels.push(Some(value));
                } else if self.eat_keyword(Keyword::Default) {
                    self.expect_symbol(":", "after 'default'")?;
                    labels.push(None);
                } else {
                    break;
                }
            }
            if labels.is_empty() {
                self.error_here("expected 'case' or 'default' in the switch body");
                return Err(Abort);
            }
            let mut body = Vec::new();
            while !self.at_symbol("}")
                && !self.at_keyword(Keyword::Case)
                && !self.at_keyword(Keyword::Default)
                && self.peek().is_some()
            {
                match self.statement() {
                    Ok(Some(stmt)) => body.push(stmt),
                    Ok(None) => {}
                    Err(Abort) => self.recover_to_statement_boundary(),
                }
            }
            arms.push(SwitchArm {
                labels,
                body,
                span: SourceSpan {
                    start: arm_start.start,
                    end: self.here().start,
                },
            });
        }
        self.expect_symbol("}", "to close the switch body")?;
        Ok(Stmt::Switch {
            selector,
            arms,
            span: SourceSpan {
                start: start.start,
                end: self.here().start,
            },
        })
    }

    /// `try { ... } catch (Type name) { ... } ...` — `finally` is
    /// recognized but not yet supported.
    fn try_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'try'
        // `try (Resource r = init; ...)` — try-with-resources. Parsed here and
        // desugared at the end into the plain try/finally the JLS specifies, so
        // codegen never sees it.
        let resources = if self.at_symbol("(") {
            self.resource_specification()?
        } else {
            Vec::new()
        };
        self.expect_symbol("{", "after 'try'")?;
        let body = self.block_body();

        let mut catches = Vec::new();
        while self.at_keyword(Keyword::Catch) {
            let clause_start = self.here();
            self.pos += 1;
            self.expect_symbol("(", "after 'catch'")?;
            // `catch (final IOException | SQLException e)` — the modifier is legal,
            // and so is a multi-catch: one handler, several alternatives.
            let _ = self.eat_keyword(Keyword::Final);
            let mut types = vec![self.type_ref()?];
            while self.eat_symbol("|") {
                types.push(self.type_ref()?);
            }
            let (name, _) = self.expect_ident("for the caught exception")?;
            self.expect_symbol(")", "after the catch parameter")?;
            self.expect_symbol("{", "after the catch parameter")?;
            let catch_body = self.block_body();
            catches.push(CatchClause {
                types,
                name,
                body: catch_body,
                span: SourceSpan {
                    start: clause_start.start,
                    end: self.here().start,
                },
            });
        }

        let finally_body = if self.at_keyword(Keyword::Finally) {
            self.pos += 1;
            self.expect_symbol("{", "after 'finally'")?;
            Some(self.block_body())
        } else {
            None
        };
        // A resource-less try still needs a catch or a finally; a
        // try-with-resources does not (its resources ARE the reason to try).
        if resources.is_empty() && catches.is_empty() && finally_body.is_none() {
            self.error_at(
                start,
                "'try' needs at least one 'catch' clause or a 'finally' block",
            );
            return Err(Abort);
        }
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        if resources.is_empty() {
            return Ok(Stmt::Try {
                body,
                catches,
                finally_body,
                span,
            });
        }
        Ok(desugar_try_with_resources(
            resources,
            body,
            catches,
            finally_body,
            span,
        ))
    }

    /// `(Type name = init; ...)` after `try`. Each resource is a declaration;
    /// caturra does not model the Java-9 "existing effectively-final variable"
    /// form, which is vanishingly rare. Implicitly `final`, as the JLS makes them.
    fn resource_specification(&mut self) -> Parsed<Vec<Resource>> {
        self.expect_symbol("(", "to open the resource list")?;
        let mut resources = Vec::new();
        loop {
            let _ = self.eat_keyword(Keyword::Final); // resources are final anyway
            // `try (r)` names an existing variable: one identifier, then `)`
            // or `;`. Anything else starts a declaration.
            let bare_existing = matches!(self.peek(), Some(TokenKind::Identifier(_)))
                && matches!(self.peek_at(1), Some(TokenKind::Symbol(")" | ";")));
            if bare_existing {
                let (name, name_span) = self.expect_ident("for the resource")?;
                resources.push(Resource {
                    ty: TypeRef::Var,
                    name: name.clone(),
                    init: Expr::Name {
                        path: vec![name],
                        span: name_span,
                    },
                    existing: true,
                    span: name_span,
                });
                if self.eat_symbol(";") {
                    if self.at_symbol(")") {
                        break;
                    }
                    continue;
                }
                break;
            }
            let ty = if self.eat_keyword(Keyword::Var) {
                TypeRef::Var
            } else {
                self.type_ref()?
            };
            let (name, name_span) = self.expect_ident("for the resource")?;
            // `try (r)` (Java 9): an existing effectively-final variable,
            // recognised by there being no `=` to follow.
            if !self.at_symbol("=") {
                resources.push(Resource {
                    ty,
                    name: name.clone(),
                    init: Expr::Name {
                        path: vec![name],
                        span: name_span,
                    },
                    existing: true,
                    span: name_span,
                });
                if self.eat_symbol(";") {
                    if self.at_symbol(")") {
                        break;
                    }
                    continue;
                }
                break;
            }
            self.expect_symbol("=", "after the resource name")?;
            let init = self.expression()?;
            resources.push(Resource {
                ty,
                name,
                init,
                existing: false,
                span: name_span,
            });
            // A trailing `;` before `)` is allowed and separates resources.
            if self.eat_symbol(";") {
                if self.at_symbol(")") {
                    break;
                }
                continue;
            }
            break;
        }
        self.expect_symbol(")", "to close the resource list")?;
        Ok(resources)
    }

    /// `throw expr;`.
    fn throw_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'throw'
        let value = self.expression()?;
        self.expect_symbol(";", "after the throw expression")?;
        Ok(Stmt::Throw {
            value,
            span: SourceSpan {
                start: start.start,
                end: self.here().start,
            },
        })
    }

    /// `assert cond;` / `assert cond : message;`. Assertions are DISABLED by
    /// default on a JVM (only `-ea` enables them), so the statement is a runtime
    /// no-op — but javac still type-checks the condition (boolean) and the
    /// message (any non-void). Desugar to a dead `if (false)` block carrying
    /// both: codegen type-checks a dead branch (a constant-`false` `if` is
    /// reachable per JLS §14.21) yet never runs it, matching `java Main` exactly.
    fn assert_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'assert'
        let cond = self.expression()?;
        let message = if self.eat_symbol(":") {
            Some(self.expression()?)
        } else {
            None
        };
        self.expect_symbol(";", "after the assert condition")?;
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        let decl = |name: &str, ty: TypeRef, init: Expr| Stmt::LocalDecl {
            ty,
            is_final: false,
            declarators: vec![LocalDeclarator {
                name: String::from(name),
                init: Some(init),
                span,
                extra_dims: 0,
            }],
            span,
        };
        // `boolean __caturraAssert = (cond);` forces the condition to be boolean.
        let mut body = vec![decl("__caturraAssert", TypeRef::Boolean, cond)];
        // `String __caturraAssertMsg = "" + (message);` forces a non-void message.
        if let Some(message) = message {
            let concat = Expr::Binary {
                op: BinaryOp::Add,
                lhs: Box::new(Expr::Literal {
                    value: Literal::Str(String::new()),
                    span,
                }),
                rhs: Box::new(message),
                span,
            };
            body.push(decl(
                "__caturraAssertMsg",
                TypeRef::Named(String::from("String")),
                concat,
            ));
        }
        Ok(Stmt::If {
            cond: Expr::Literal {
                value: Literal::Bool(false),
                span,
            },
            then: Box::new(Stmt::Block(body)),
            els: None,
            span,
        })
    }

    fn while_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'while'
        let cond = self.paren_condition("while")?;
        let body = Box::new(self.embedded_statement("a while loop")?);
        Ok(Stmt::While {
            cond,
            body,
            span: start,
        })
    }

    fn do_while_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'do'
        let body = Box::new(self.embedded_statement("a do-while loop")?);
        if !self.eat_keyword(Keyword::While) {
            self.error_here("expected 'while' after the do-while body");
            return Err(Abort);
        }
        let cond = self.paren_condition("while")?;
        self.expect_symbol(";", "to end the do-while statement")?;
        Ok(Stmt::DoWhile {
            body,
            cond,
            span: start,
        })
    }

    fn for_statement(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        self.pos += 1; // 'for'
        self.expect_symbol("(", "after 'for'")?;

        // `for (Type name : iterable) body` — the enhanced for.
        if self.header_contains_top_level_colon() {
            let ty = if self.eat_keyword(Keyword::Var) {
                TypeRef::Var
            } else {
                self.type_ref()?
            };
            let (name, _) = self.expect_ident("for the loop variable")?;
            self.expect_symbol(":", "in the for-each header")?;
            let iterable = self.expression()?;
            self.expect_symbol(")", "to close the for-each header")?;
            let body = Box::new(self.embedded_statement("a for-each loop")?);
            return Ok(Stmt::ForEach {
                ty,
                name,
                iterable,
                body,
                span: start,
            });
        }

        let init = if self.eat_symbol(";") {
            None
        } else if self.at_declaration_start() {
            // Consumes the `;` itself.
            Some(Box::new(self.local_declaration()?))
        } else {
            let stmt = self.simple_statement()?;
            self.expect_symbol(";", "after the for-loop initializer")?;
            Some(Box::new(stmt))
        };

        let cond = if self.at_symbol(";") {
            None
        } else {
            Some(self.expression()?)
        };
        self.expect_symbol(";", "after the for-loop condition")?;

        let mut update = Vec::new();
        if !self.at_symbol(")") {
            loop {
                update.push(self.simple_statement()?);
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }
        self.expect_symbol(")", "to close the for-loop header")?;

        let body = Box::new(self.embedded_statement("a for loop")?);
        Ok(Stmt::For {
            init,
            cond,
            update,
            body,
            span: start,
        })
    }

    fn return_statement(&mut self) -> Parsed<Stmt> {
        let span = self.here();
        self.pos += 1; // 'return'
        if self.at_symbol(";") {
            self.expect_symbol(";", "to end the return statement")?;
            return Ok(Stmt::Return { value: None, span });
        }
        let expr = self.expression()?;
        // `return x += y;` — an assignment expression. caturra models assignment
        // as a statement, so lower it to `x += y; return x;`.
        if let Some(op) = self.assignment_operator()
            && let Some(target) = assignment_target(&expr)
        {
            self.pos += 1;
            let rhs = self.expression()?;
            self.expect_symbol(";", "to end the return statement")?;
            return Ok(Stmt::Block(vec![
                Stmt::Assign {
                    target,
                    op,
                    value: rhs,
                    span,
                },
                Stmt::Return {
                    value: Some(expr),
                    span,
                },
            ]));
        }
        self.expect_symbol(";", "to end the return statement")?;
        Ok(Stmt::Return {
            value: Some(expr),
            span,
        })
    }

    fn break_or_continue(&mut self) -> Parsed<Stmt> {
        let span = self.here();
        let is_break = self.at_keyword(Keyword::Break);
        self.pos += 1;
        let label = if let Some(TokenKind::Identifier(name)) = self.peek() {
            let name = name.clone();
            self.pos += 1;
            Some(name)
        } else {
            None
        };
        self.expect_symbol(";", "to end the statement")?;
        Ok(if is_break {
            Stmt::Break { label, span }
        } else {
            Stmt::Continue { label, span }
        })
    }

    /// Whether the parenthesized header at the cursor contains a `:` at
    /// paren depth zero before any `;` (i.e. a for-each header). The
    /// cursor sits just past the opening `(`.
    fn header_contains_top_level_colon(&self) -> bool {
        let mut depth = 0usize;
        let mut offset = 0usize;
        while let Some(kind) = self.peek_at(offset) {
            match kind {
                TokenKind::Symbol("(") => depth += 1,
                TokenKind::Symbol(")") => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                TokenKind::Symbol(";") if depth == 0 => return false,
                TokenKind::Symbol(":") if depth == 0 => return true,
                _ => {}
            }
            offset += 1;
        }
        false
    }

    /// Whether the cursor starts a local declaration:
    /// `final? <type> name ...` — a primitive-type keyword, `final`, or
    /// a class type followed by a name (or `[]`).
    fn at_declaration_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(TokenKind::Keyword(
                Keyword::Int
                    | Keyword::Double
                    | Keyword::Boolean
                    | Keyword::Char
                    | Keyword::Final
                    | Keyword::Long
                    | Keyword::Float
                    | Keyword::Byte
                    | Keyword::Short
                    | Keyword::Var
            ))
        ) || (matches!(self.peek(), Some(TokenKind::Identifier(_)))
            && matches!(self.peek_at(1), Some(TokenKind::Identifier(_))))
            || (matches!(self.peek(), Some(TokenKind::Identifier(_)))
                && matches!(self.peek_at(1), Some(TokenKind::Symbol("[")))
                && matches!(self.peek_at(2), Some(TokenKind::Symbol("]"))))
            // `ArrayList<Integer> list = ...`, `Pair<A, B> p = ...` — a
            // generic declaration. Scan a balanced `<...>` and require a
            // following identifier. (`a < b;` alone is not a valid
            // statement, so this is safe.)
            || self.is_generic_declaration()
            // Fully qualified declarations: `java.util.Scanner sc = ...`
            // (scan `Ident (. Ident)+`, then an identifier or generic
            // arguments means a declaration, not an expression).
            || self.at_qualified_declaration()
    }

    /// Whether the cursor is `Ident < ... > Ident` — a generic local
    /// declaration. Scans balanced angle brackets from the `<`.
    fn is_generic_declaration(&self) -> bool {
        if !matches!(self.peek(), Some(TokenKind::Identifier(_)))
            || !matches!(self.peek_at(1), Some(TokenKind::Symbol("<")))
        {
            return false;
        }
        let mut depth = 0i32;
        let mut offset = 1usize;
        while let Some(kind) = self.peek_at(offset) {
            match kind {
                TokenKind::Symbol("<") => depth += 1,
                TokenKind::Symbol(">") => depth -= 1,
                TokenKind::Symbol(">>") => depth -= 2,
                // `>>>` closes three levels at once (`Map<K, List<Set<V>>>`).
                TokenKind::Symbol(">>>") => depth -= 3,
                // Only type names, commas, dots, and nested `<>` appear
                // in a type-argument list; anything else means this was
                // a comparison expression.
                // ...plus wildcards (`Class<?>`, `List<? extends T>`) and array
                // type arguments, incl. primitive ones (`List<int[]>`,
                // `List<String[]>`). A bare primitive (`List<int>`) parses here
                // and is rejected during type resolution, as javac does.
                TokenKind::Identifier(_)
                | TokenKind::Symbol("," | "." | "?" | "[" | "]")
                | TokenKind::Keyword(
                    Keyword::Extends
                    | Keyword::Super
                    | Keyword::Int
                    | Keyword::Double
                    | Keyword::Boolean
                    | Keyword::Char
                    | Keyword::Long
                    | Keyword::Float
                    | Keyword::Short
                    | Keyword::Byte,
                ) => {}
                _ => return false,
            }
            if depth <= 0 {
                // Allow a generic array type before the name: `Class<?>[] xs`.
                let mut next = offset + 1;
                while matches!(self.peek_at(next), Some(TokenKind::Symbol("[")))
                    && matches!(self.peek_at(next + 1), Some(TokenKind::Symbol("]")))
                {
                    next += 2;
                }
                return matches!(self.peek_at(next), Some(TokenKind::Identifier(_)));
            }
            offset += 1;
        }
        false
    }

    /// The assignment operator at the cursor, if any: `Some(None)` for
    /// `=`, `Some(Some(op))` for compound forms. Does not consume.
    #[allow(clippy::option_option)]
    fn assignment_operator(&self) -> Option<Option<BinaryOp>> {
        match self.peek() {
            Some(TokenKind::Symbol("=")) => Some(None),
            Some(TokenKind::Symbol("+=")) => Some(Some(BinaryOp::Add)),
            Some(TokenKind::Symbol("-=")) => Some(Some(BinaryOp::Sub)),
            Some(TokenKind::Symbol("*=")) => Some(Some(BinaryOp::Mul)),
            Some(TokenKind::Symbol("/=")) => Some(Some(BinaryOp::Div)),
            Some(TokenKind::Symbol("%=")) => Some(Some(BinaryOp::Rem)),
            Some(TokenKind::Symbol("&=")) => Some(Some(BinaryOp::BitAnd)),
            Some(TokenKind::Symbol("|=")) => Some(Some(BinaryOp::BitOr)),
            Some(TokenKind::Symbol("^=")) => Some(Some(BinaryOp::BitXor)),
            Some(TokenKind::Symbol("<<=")) => Some(Some(BinaryOp::Shl)),
            Some(TokenKind::Symbol(">>=")) => Some(Some(BinaryOp::Shr)),
            Some(TokenKind::Symbol(">>>=")) => Some(Some(BinaryOp::Ushr)),
            _ => None,
        }
    }

    fn local_declaration(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        let is_final = self.eat_keyword(Keyword::Final);
        let ty = if self.eat_keyword(Keyword::Var) {
            TypeRef::Var
        } else {
            self.type_ref()?
        };

        let mut declarators = Vec::new();
        loop {
            let (name, name_span) = self.expect_ident("for the variable")?;
            // `int a[] = {1, 2};` — the brackets belong to this declarator.
            let extra_dims = self.trailing_array_dims();
            let init = if self.eat_symbol("=") {
                // `int[] a = {1, 2, 3};` — the literal form is only
                // legal directly in a declaration.
                if self.at_symbol("{") {
                    Some(self.array_literal()?)
                } else {
                    Some(self.expression()?)
                }
            } else {
                None
            };
            declarators.push(LocalDeclarator {
                name,
                init,
                span: name_span,
                extra_dims,
            });
            if !self.eat_symbol(",") {
                break;
            }
        }
        let end = self.here().start;
        self.expect_symbol(";", "to end the declaration")?;
        Ok(Stmt::LocalDecl {
            ty,
            is_final,
            declarators,
            span: SourceSpan {
                start: start.start,
                end,
            },
        })
    }

    // ----- expressions, by descending precedence -----

    fn expression(&mut self) -> Parsed<Expr> {
        if let Some(lambda) = self.try_lambda()? {
            return Ok(lambda);
        }
        self.assignment()
    }

    /// Assignment is the lowest-precedence expression and right-associative
    /// (JLS §15.26): `a = b = c` is `a = (b = c)`, and its value is what was
    /// stored. The left side must be an assignable target. A bare `x = e;`
    /// statement is recognized here too and unwrapped to a `Stmt::Assign` by
    /// the statement parser, so its definite-assignment analysis is unchanged.
    fn assignment(&mut self) -> Parsed<Expr> {
        let lhs = self.ternary()?;
        let Some(op) = self.assignment_operator() else {
            return Ok(lhs);
        };
        let Some(target) = assignment_target(&lhs) else {
            self.error_at(
                lhs.span(),
                "the left side of an assignment must be a variable or array element",
            );
            return Err(Abort);
        };
        self.pos += 1; // the assignment operator
        let value = self.expression()?; // right-associative; a lambda may follow
        let span = SourceSpan {
            start: lhs.span().start,
            end: value.span().end,
        };
        Ok(Expr::Assign {
            target,
            op,
            value: Box::new(value),
            span,
        })
    }

    /// Detect and parse a lambda: `x -> body`, `(a, b) -> body`, or
    /// `(Type a) -> body`. Returns `None` if the cursor is not a lambda.
    fn try_lambda(&mut self) -> Parsed<Option<Expr>> {
        let start = self.here();
        // `identifier ->`
        if matches!(self.peek(), Some(TokenKind::Identifier(_)))
            && self.peek_at(1) == Some(&TokenKind::Symbol("->"))
        {
            let (name, _) = self.expect_ident("for the lambda parameter")?;
            self.pos += 1; // '->'
            let params = vec![LambdaParam { name, ty: None }];
            return Ok(Some(self.lambda_body(params, start)?));
        }
        // `( ... ) ->`
        if self.at_symbol("(") && self.parenthesized_is_lambda() {
            self.pos += 1; // '('
            let mut params = Vec::new();
            if !self.at_symbol(")") {
                loop {
                    // A parameter is `Type name` or just `name`.
                    // A parameter has an explicit type unless it is a
                    // bare name (an identifier followed by `,` or `)`).
                    let bare_name = matches!(self.peek(), Some(TokenKind::Identifier(_)))
                        && !matches!(self.peek_at(1), Some(TokenKind::Identifier(_)));
                    let ty = if bare_name {
                        None
                    } else {
                        Some(self.type_ref()?)
                    };
                    let (name, _) = self.expect_ident("for the lambda parameter")?;
                    params.push(LambdaParam { name, ty });
                    if !self.eat_symbol(",") {
                        break;
                    }
                }
            }
            self.expect_symbol(")", "to close the lambda parameters")?;
            self.expect_symbol("->", "in the lambda")?;
            return Ok(Some(self.lambda_body(params, start)?));
        }
        Ok(None)
    }

    /// Whether a `(...)` at the cursor is a lambda parameter list:
    /// scan to the matching `)` and check for a following `->`.
    fn parenthesized_is_lambda(&self) -> bool {
        let mut depth = 0i32;
        let mut offset = 0usize;
        while let Some(kind) = self.peek_at(offset) {
            match kind {
                TokenKind::Symbol("(") => depth += 1,
                TokenKind::Symbol(")") => {
                    depth -= 1;
                    if depth == 0 {
                        return self.peek_at(offset + 1) == Some(&TokenKind::Symbol("->"));
                    }
                }
                _ => {}
            }
            offset += 1;
        }
        false
    }

    fn lambda_body(&mut self, params: Vec<LambdaParam>, start: SourceSpan) -> Parsed<Expr> {
        let body = if self.at_symbol("{") {
            self.pos += 1;
            LambdaBody::Block(self.block_body())
        } else {
            self.lambda_expression_body()?
        };
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        Ok(Expr::Lambda { params, body, span })
    }

    /// A non-block lambda body. In Java this is any expression; assignment and
    /// `++`/`--` are expressions, but as a lambda body they run for effect, so
    /// they become a one-statement block (a `Stmt::Assign`) that reuses the
    /// statement-assignment path — the one that routes a bare name to an
    /// enclosing field. A plain value expression (`x -> x + 1`) stays an
    /// expression body.
    fn lambda_expression_body(&mut self) -> Parsed<LambdaBody> {
        // Prefix `++x` / `--x`.
        if self.at_symbol("++") || self.at_symbol("--") {
            let start = self.here();
            let increment = self.at_symbol("++");
            self.pos += 1;
            let operand = self.postfix_expression()?;
            let Some(target) = assignment_target(&operand) else {
                self.error_at(operand.span(), "++/-- can only be applied to a variable");
                return Err(Abort);
            };
            let stmt = increment_statement(target, increment, start.start, operand.span().end);
            return Ok(LambdaBody::Block(vec![stmt]));
        }

        let expr = self.expression()?;

        // `x -> count++`: a postfix increment used for effect.
        if let Expr::IncDec {
            target,
            increment,
            span,
            ..
        } = &expr
            && let Some(assign_target) = assignment_target(target)
        {
            let stmt = increment_statement(assign_target, *increment, span.start, span.end);
            return Ok(LambdaBody::Block(vec![stmt]));
        }

        // `n -> total[0] += n`: an assignment, unwrapped to a statement so it
        // takes the same code path a bare `total[0] += n;` would.
        if let Expr::Assign {
            target,
            op,
            value,
            span,
        } = expr
        {
            return Ok(LambdaBody::Block(vec![Stmt::Assign {
                target,
                op,
                value: *value,
                span,
            }]));
        }

        Ok(LambdaBody::Expr(Box::new(expr)))
    }

    /// `cond ? then : else` (right-associative, lowest precedence
    /// above assignment).
    fn ternary(&mut self) -> Parsed<Expr> {
        let cond = self.logical_or()?;
        if !self.eat_symbol("?") {
            return Ok(cond);
        }
        let then = self.ternary()?;
        self.expect_symbol(":", "in the conditional expression")?;
        let els = self.ternary()?;
        let span = SourceSpan {
            start: cond.span().start,
            end: els.span().end,
        };
        Ok(Expr::Ternary {
            cond: Box::new(cond),
            then: Box::new(then),
            els: Box::new(els),
            span,
        })
    }

    fn binary_level(
        &mut self,
        next: fn(&mut Self) -> Parsed<Expr>,
        table: &[(&str, BinaryOp)],
    ) -> Parsed<Expr> {
        let mut lhs = next(self)?;
        'outer: loop {
            for (symbol, op) in table {
                if self.at_symbol(symbol) {
                    self.pos += 1;
                    let rhs = next(self)?;
                    let span = SourceSpan {
                        start: lhs.span().start,
                        end: rhs.span().end,
                    };
                    lhs = Expr::Binary {
                        op: *op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        span,
                    };
                    continue 'outer;
                }
            }
            return Ok(lhs);
        }
    }

    fn logical_or(&mut self) -> Parsed<Expr> {
        self.binary_level(Self::logical_and, &[("||", BinaryOp::Or)])
    }

    fn logical_and(&mut self) -> Parsed<Expr> {
        self.binary_level(Self::bit_or, &[("&&", BinaryOp::And)])
    }

    fn bit_or(&mut self) -> Parsed<Expr> {
        self.binary_level(Self::bit_xor, &[("|", BinaryOp::BitOr)])
    }

    fn bit_xor(&mut self) -> Parsed<Expr> {
        self.binary_level(Self::bit_and, &[("^", BinaryOp::BitXor)])
    }

    fn bit_and(&mut self) -> Parsed<Expr> {
        self.binary_level(Self::equality, &[("&", BinaryOp::BitAnd)])
    }

    fn equality(&mut self) -> Parsed<Expr> {
        self.binary_level(
            Self::relational,
            &[("==", BinaryOp::Eq), ("!=", BinaryOp::Ne)],
        )
    }

    fn relational(&mut self) -> Parsed<Expr> {
        let mut expr = self.binary_level(
            Self::shift,
            &[
                ("<=", BinaryOp::Le),
                (">=", BinaryOp::Ge),
                ("<", BinaryOp::Lt),
                (">", BinaryOp::Gt),
            ],
        )?;
        while self.eat_keyword(Keyword::Instanceof) {
            let ty = self.type_ref()?;
            let span = SourceSpan {
                start: expr.span().start,
                end: self.here().start,
            };
            expr = Expr::InstanceOf {
                value: Box::new(expr),
                ty,
                span,
            };
        }
        Ok(expr)
    }

    fn shift(&mut self) -> Parsed<Expr> {
        self.binary_level(
            Self::additive,
            &[
                ("<<", BinaryOp::Shl),
                (">>>", BinaryOp::Ushr),
                (">>", BinaryOp::Shr),
            ],
        )
    }

    fn additive(&mut self) -> Parsed<Expr> {
        self.binary_level(
            Self::multiplicative,
            &[("+", BinaryOp::Add), ("-", BinaryOp::Sub)],
        )
    }

    fn multiplicative(&mut self) -> Parsed<Expr> {
        self.binary_level(
            Self::unary,
            &[
                ("*", BinaryOp::Mul),
                ("/", BinaryOp::Div),
                ("%", BinaryOp::Rem),
            ],
        )
    }

    #[allow(clippy::too_many_lines)] // one arm per prefix operator
    /// If a reference-cast type `(T)` begins at the current `(` — a
    /// (possibly dotted) name, optional generic arguments, optional `[]` — return
    /// the offset (from `self.pos`) of the token just past the closing `)`. Pure
    /// lookahead: it consumes nothing and never allocates, so `unary` can use it
    /// to disambiguate a cast from a parenthesized expression.
    fn scan_cast_type(&self) -> Option<usize> {
        // self.pos is at `(`.
        let mut i = 1;
        // A leading primitive keyword is an array-cast target here — a bare
        // `(int) x` is handled by the dedicated primitive-cast arm, so this
        // path exists for `(int[]) x`, `(double[][]) x`, and so on.
        if matches!(self.peek_at(i), Some(TokenKind::Keyword(k)) if primitive_type_name(*k).is_some())
        {
            i += 1;
            while matches!(self.peek_at(i), Some(TokenKind::Symbol(s)) if *s == "[")
                && matches!(self.peek_at(i + 1), Some(TokenKind::Symbol(s)) if *s == "]")
            {
                i += 2;
            }
            return matches!(self.peek_at(i), Some(TokenKind::Symbol(s)) if *s == ")")
                .then_some(i + 1);
        }
        // A dotted name: Ident ('.' Ident)*.
        if !matches!(self.peek_at(i), Some(TokenKind::Identifier(_))) {
            return None;
        }
        i += 1;
        while matches!(self.peek_at(i), Some(TokenKind::Symbol(s)) if *s == ".") {
            i += 1;
            if !matches!(self.peek_at(i), Some(TokenKind::Identifier(_))) {
                return None;
            }
            i += 1;
        }
        // Optional generic arguments, balanced across `<` … `>` (and `>>`).
        if matches!(self.peek_at(i), Some(TokenKind::Symbol(s)) if *s == "<") {
            let mut depth: i32 = 0;
            loop {
                match self.peek_at(i)? {
                    TokenKind::Symbol(s) => {
                        let opens = s.chars().filter(|c| *c == '<').count();
                        let closes = s.chars().filter(|c| *c == '>').count();
                        let allowed =
                            opens > 0 || closes > 0 || matches!(*s, "," | "." | "?" | "[" | "]");
                        if !allowed {
                            return None;
                        }
                        depth += i32::try_from(opens).ok()?;
                        depth -= i32::try_from(closes).ok()?;
                        i += 1;
                        if depth <= 0 {
                            break;
                        }
                    }
                    TokenKind::Identifier(_) | TokenKind::Keyword(_) => i += 1,
                    _ => return None,
                }
            }
        }
        // Optional array dimensions.
        while matches!(self.peek_at(i), Some(TokenKind::Symbol(s)) if *s == "[")
            && matches!(self.peek_at(i + 1), Some(TokenKind::Symbol(s)) if *s == "]")
        {
            i += 2;
        }
        if matches!(self.peek_at(i), Some(TokenKind::Symbol(s)) if *s == ")") {
            Some(i + 1)
        } else {
            None
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per prefix/cast form
    fn unary(&mut self) -> Parsed<Expr> {
        let start = self.here();
        if self.eat_symbol("~") {
            let operand = self.unary()?;
            let span = SourceSpan {
                start: start.start,
                end: operand.span().end,
            };
            return Ok(Expr::Unary {
                op: UnaryOp::BitNot,
                operand: Box::new(operand),
                span,
            });
        }
        // Prefix increment/decrement in expression position.
        if self.at_symbol("++") || self.at_symbol("--") {
            let increment = self.at_symbol("++");
            self.pos += 1;
            let operand = self.unary()?;
            let span = SourceSpan {
                start: start.start,
                end: operand.span().end,
            };
            return Ok(Expr::IncDec {
                target: Box::new(operand),
                increment,
                prefix: true,
                span,
            });
        }
        if self.eat_symbol("-") {
            let operand = self.unary()?;
            let span = SourceSpan {
                start: start.start,
                end: operand.span().end,
            };
            // Fold negated numeric literals so `-2147483648` (which
            // only exists as a negated literal) is representable.
            return Ok(match operand {
                Expr::Literal {
                    value: Literal::Int(v),
                    ..
                } => Expr::Literal {
                    value: Literal::Int(-v),
                    span,
                },
                Expr::Literal {
                    value: Literal::Double(v),
                    ..
                } => Expr::Literal {
                    value: Literal::Double(-v),
                    span,
                },
                operand => Expr::Unary {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span,
                },
            });
        }
        if self.eat_symbol("!") {
            let operand = self.unary()?;
            let span = SourceSpan {
                start: start.start,
                end: operand.span().end,
            };
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                span,
            });
        }
        if self.eat_symbol("+") {
            // Unary plus is a no-op.
            return self.unary();
        }
        if self.at_symbol("++") || self.at_symbol("--") {
            self.error_here("++/-- inside an expression is not yet supported by caturra");
            return Err(Abort);
        }
        // Primitive cast: `(int) x` — unambiguous because a primitive
        // keyword can't start a parenthesized expression.
        if self.at_symbol("(")
            && matches!(
                self.peek_at(1),
                Some(TokenKind::Keyword(
                    Keyword::Int
                        | Keyword::Double
                        | Keyword::Boolean
                        | Keyword::Char
                        | Keyword::Long
                        | Keyword::Float
                        | Keyword::Short
                        | Keyword::Byte
                ))
            )
            && matches!(self.peek_at(2), Some(TokenKind::Symbol(")")))
        {
            self.pos += 1;
            let ty = self.type_ref()?;
            self.expect_symbol(")", "to close the cast")?;
            let operand = self.unary()?;
            let span = SourceSpan {
                start: start.start,
                end: operand.span().end,
            };
            return Ok(Expr::Cast {
                ty,
                operand: Box::new(operand),
                span,
            });
        }
        // Class cast: `(Shape) x`, `(java.util.ArrayList) x`, or
        // `(ArrayList<Object>) x`. A `(type)` followed by a token that can
        // only start an operand is a cast, not a parenthesized expression
        // (`(a) - b` stays arithmetic). `super` starts an operand too, as in
        // `(JLabel) super.getListCellRendererComponent(...)`, and so does
        // `null`, as in `(String) null` to pick a varargs overload.
        //
        // A LITERAL is just as unambiguous: `(a) 5` cannot be a parenthesized
        // expression followed by anything, so it is a cast — which is what
        // makes the boxing cast `(Integer) 5` parse. Only the tokens that
        // could also be a binary operator (`-`, `+`, `(`) stay excluded.
        if self.at_symbol("(")
            && let Some(after) = self.scan_cast_type()
            && matches!(
                self.peek_at(after),
                Some(
                    TokenKind::Identifier(_)
                        | TokenKind::Keyword(Keyword::New | Keyword::This | Keyword::Super)
                        | TokenKind::StringLiteral(_)
                        | TokenKind::NullLiteral
                        | TokenKind::IntLiteral(_)
                        | TokenKind::LongLiteral(_)
                        | TokenKind::FloatLiteral(_)
                        | TokenKind::DoubleLiteral(_)
                        | TokenKind::CharLiteral(_)
                        | TokenKind::BooleanLiteral(_)
                        // `(Short) (short) 3` — the operand is itself
                        // parenthesized or another cast. Safe because
                        // `scan_cast_type` has already established that what
                        // precedes is a well-formed TYPE followed by `)`, and
                        // `(variable) (expr)` is not valid Java under any
                        // reading, so nothing legal is stolen.
                        | TokenKind::Symbol("(")
                )
            )
        {
            self.pos += 1;
            let ty = self.type_ref()?;
            self.expect_symbol(")", "to close the cast")?;
            let operand = self.unary()?;
            let span = SourceSpan {
                start: start.start,
                end: operand.span().end,
            };
            return Ok(Expr::Cast {
                ty,
                operand: Box::new(operand),
                span,
            });
        }
        self.postfix_expression()
    }

    #[allow(clippy::too_many_lines)] // one arm per postfix operator
    fn postfix_expression(&mut self) -> Parsed<Expr> {
        let mut expr = self.primary_expression()?;

        loop {
            // Method reference: `qualifier::method` or `Type::new`.
            if self.at_symbol("::") {
                self.pos += 1;
                let method = if self.eat_keyword(Keyword::New) {
                    String::from("new")
                } else {
                    self.expect_ident("after '::'")?.0
                };
                let span = SourceSpan {
                    start: expr.span().start,
                    end: self.here().start,
                };
                expr = Expr::MethodRef {
                    qualifier: Box::new(expr),
                    method,
                    span,
                };
                continue;
            }
            // Postfix increment/decrement in expression position. The
            // statement parser intercepts the statement-only form
            // before expressions are involved, so reaching here means
            // a value is wanted (`y = x++`, `a[i++]`).
            if self.at_symbol("++") || self.at_symbol("--") {
                let increment = self.at_symbol("++");
                self.pos += 1;
                let span = SourceSpan {
                    start: expr.span().start,
                    end: self.here().start,
                };
                expr = Expr::IncDec {
                    target: Box::new(expr),
                    increment,
                    prefix: false,
                    span,
                };
                continue;
            }
            if self.eat_symbol(".") {
                // `Type.class` class literal — modeled as a field access named
                // "class" (used by the EasyMock partial-mock rewrite).
                if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Class))) {
                    let end = self.here().end;
                    self.pos += 1;
                    let span = SourceSpan {
                        start: expr.span().start,
                        end,
                    };
                    expr = Expr::Field {
                        object: Box::new(expr),
                        name: String::from("class"),
                        span,
                    };
                    continue;
                }
                // Qualified inner-class creation: `outer.new Inner(args)`. The
                // enclosing instance rides in `outer`; a pass binds it.
                if self.eat_keyword(Keyword::New) {
                    let start = expr.span().start;
                    let (name, _) = self.expect_ident("for the inner class after '.new'")?;
                    self.skip_type_args();
                    let args = self.arguments()?;
                    let span = SourceSpan {
                        start,
                        end: self.here().start,
                    };
                    expr = Expr::NewObject {
                        class: name,
                        type_args: Vec::new(),
                        args,
                        outer: Some(Box::new(expr)),
                        span,
                    };
                    continue;
                }
                // `Outer.this` — qualified this (JLS §15.8.4), naming the
                // enclosing instance from inside an inner class. Encoded as a
                // name path ending in `this`, which cannot collide with a
                // field: `this` is a keyword, so no field can be called that.
                if self.at_keyword(Keyword::This)
                    && let Expr::Name { path, span } = &mut expr
                {
                    let end = self.here().end;
                    self.pos += 1;
                    path.push(String::from("this"));
                    span.end = end;
                    continue;
                }
                // An explicit type witness on a generic method call
                // (`Collections.<String>emptyList()`, `this.<T>id(x)`,
                // JLS §15.12): the arguments erase away, so skipping them is
                // exactly what the call needs. A `<` HERE — directly after the
                // `.` — can only be a witness; `a.b < c` puts the `<` after
                // the name, not before it.
                self.skip_type_args();
                let (segment, segment_span) = self.expect_ident("after '.'")?;
                if self.at_symbol("(") {
                    let args = self.arguments()?;
                    let span = SourceSpan {
                        start: expr.span().start,
                        end: segment_span.end,
                    };
                    expr = Expr::Call {
                        receiver: Some(Box::new(expr)),
                        method: segment,
                        args,
                        span,
                    };
                } else if let Expr::Name { path, span } = &mut expr {
                    path.push(segment);
                    span.end = segment_span.end;
                } else {
                    // Field access on a computed value: `m[i].length`.
                    let span = SourceSpan {
                        start: expr.span().start,
                        end: segment_span.end,
                    };
                    expr = Expr::Field {
                        object: Box::new(expr),
                        name: segment,
                        span,
                    };
                }
            } else if self.at_symbol("[") {
                self.pos += 1;
                let index = self.expression()?;
                self.expect_symbol("]", "to close the array index")?;
                let span = SourceSpan {
                    start: expr.span().start,
                    end: self.here().start,
                };
                expr = Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else if self.at_symbol("(") {
                if let Expr::Name { path, span } = &expr
                    && path.len() == 1
                {
                    let method = path[0].clone();
                    let start = span.start;
                    let args = self.arguments()?;
                    let span = SourceSpan {
                        start,
                        end: self.here().start,
                    };
                    expr = Expr::Call {
                        receiver: None,
                        method,
                        args,
                        span,
                    };
                } else {
                    self.error_here("this call expression is not yet supported by caturra");
                    return Err(Abort);
                }
            } else {
                return Ok(expr);
            }
        }
    }

    /// `new int[3]`, `new int[2][3]`, `new int[]{...}` — array
    /// creation. Constructor calls (`new Scanner(...)`) get a friendly
    /// not-yet message.
    #[allow(clippy::too_many_lines)] // one coherent grammar production
    /// Parse an anonymous class body and desugar it to a synthesized
    /// top-level class that extends/implements `supertype`. Returns a
    /// `new Anon$N(args)` expression; the capture pass synthesizes a
    /// constructor that forwards `args` to `super(...)` (and stores any
    /// captured enclosing locals).
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    fn anonymous_class(
        &mut self,
        supertype: &str,
        args: Vec<Expr>,
        start: SourceSpan,
    ) -> Parsed<Expr> {
        let name = self.synth_class_from_body(supertype, start)?;
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        Ok(Expr::NewObject {
            class: name,
            type_args: Vec::new(),
            // The super-constructor args ride along; the capture pass turns them
            // into a synthesized constructor that calls super(...).
            args,
            outer: None,
            span,
        })
    }

    /// Parse a `{ ...members... }` class body (positioned at the `{`) into a
    /// synthesized top-level class that extends/implements `supertype`, hoisted
    /// to top level like an anonymous class. Returns the synthesized name. Used
    /// both for anonymous classes (`new T(){...}`) and for enum constants that
    /// carry a body (`PLUS { int apply(...) {...} }`).
    #[allow(clippy::unnecessary_wraps)] // Parsed<_> for symmetry with the caller
    fn synth_class_from_body(&mut self, supertype: &str, start: SourceSpan) -> Parsed<String> {
        self.pos += 1; // '{'
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut init_blocks = Vec::new();
        let mut nested = Vec::new();
        let mut order = 0usize;
        while !self.at_symbol("}") && self.peek().is_some() {
            if let Ok(member) = self.member(supertype, false) {
                match member {
                    Member::Method(m) => methods.push(m),
                    Member::Fields(mut declared) => {
                        for f in &mut declared {
                            f.order = order;
                            order += 1;
                        }
                        fields.append(&mut declared);
                    }
                    Member::Init(mut b) => {
                        b.order = order;
                        order += 1;
                        init_blocks.push(b);
                    }
                    Member::Nested(decl) => nested.push(decl),
                }
            } else {
                self.recover_to_statement_boundary();
                self.eat_symbol(";");
            }
        }
        self.eat_symbol("}");

        self.anon_counter += 1;
        let name = format!("Anon${}", self.anon_counter);
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        self.anon_classes.push(ClassDecl {
            name: name.clone(),
            is_public: false,
            is_nested: false,
            enclosing: None,
            // The supertype is resolved to extends/implements by the
            // compiler (it knows which names are interfaces).
            superclass: Some(String::from(supertype)),
            interfaces: Vec::new(),
            supertype_args: Vec::new(),
            is_abstract: false,
            is_interface: false,
            is_enum: false,
            is_anonymous: true,
            is_local: false,
            is_inner: false,
            type_params: Vec::new(),
            fields,
            methods,
            init_blocks,
            nested,
            span,
        });
        Ok(name)
    }

    #[allow(clippy::too_many_lines)] // one arm per constructible form
    fn new_expression(&mut self) -> Parsed<Expr> {
        let start = self.here();
        self.pos += 1; // 'new'

        let base = match self.peek() {
            Some(TokenKind::Keyword(Keyword::Int)) => {
                self.pos += 1;
                TypeRef::Int
            }
            Some(TokenKind::Keyword(Keyword::Double)) => {
                self.pos += 1;
                TypeRef::Double
            }
            Some(TokenKind::Keyword(Keyword::Boolean)) => {
                self.pos += 1;
                TypeRef::Boolean
            }
            Some(TokenKind::Keyword(Keyword::Char)) => {
                self.pos += 1;
                TypeRef::Char
            }
            Some(TokenKind::Keyword(Keyword::Long)) => {
                self.pos += 1;
                TypeRef::Long
            }
            Some(TokenKind::Keyword(Keyword::Float)) => {
                self.pos += 1;
                TypeRef::Float
            }
            Some(TokenKind::Keyword(Keyword::Short)) => {
                self.pos += 1;
                TypeRef::Short
            }
            Some(TokenKind::Keyword(Keyword::Byte)) => {
                self.pos += 1;
                TypeRef::Byte
            }
            Some(TokenKind::Identifier(_)) => {
                let (mut name, _) = self.expect_ident("after 'new'")?;
                // `new java.util.Scanner(...)` — fully qualified.
                while self.at_symbol(".")
                    && matches!(self.peek_at(1), Some(TokenKind::Identifier(_)))
                {
                    self.pos += 1;
                    let (segment, _) = self.expect_ident("in the qualified type")?;
                    name.push('.');
                    name.push_str(&segment);
                }
                TypeRef::Named(name)
            }
            _ => {
                self.error_here("expected a type after 'new'");
                return Err(Abort);
            }
        };

        // Optional generic arguments: `new ArrayList<Integer>()` or the
        // diamond `new ArrayList<>()`.
        let mut type_args = Vec::new();
        if matches!(base, TypeRef::Named(_)) && self.at_symbol("<") {
            self.pos += 1;
            if !self.at_symbol(">") {
                loop {
                    if self.at_symbol("?") {
                        self.pos += 1;
                        if matches!(
                            self.peek(),
                            Some(TokenKind::Keyword(Keyword::Extends | Keyword::Super))
                        ) {
                            self.pos += 1;
                            let _ = self.type_ref()?;
                        }
                        type_args.push(TypeRef::Named(String::from("Object")));
                    } else {
                        type_args.push(self.type_ref()?);
                    }
                    if !self.eat_symbol(",") {
                        break;
                    }
                }
            }
            self.expect_symbol(">", "to close the type arguments")?;
        }

        if self.at_symbol("(") {
            // `new ClassName(args)` — object creation.
            let TypeRef::Named(class) = base else {
                self.error_at(start, "primitive types cannot be constructed with 'new'");
                return Err(Abort);
            };
            let args = self.arguments()?;
            // Anonymous class: `new Type(args) { members }`.
            if self.at_symbol("{") {
                return self.anonymous_class(&class, args, start);
            }
            let span = SourceSpan {
                start: start.start,
                end: self.here().start,
            };
            return Ok(Expr::NewObject {
                class,
                type_args,
                args,
                outer: None,
                span,
            });
        }
        // A generic array like `new Class<?>[n]` — the type arguments erase,
        // so keep going into the array dimensions.
        if !type_args.is_empty() && !self.at_symbol("[") {
            self.error_at(start, "expected '(' after the generic type");
            return Err(Abort);
        }

        let mut dims: Vec<Option<Expr>> = Vec::new();
        while self.eat_symbol("[") {
            if self.eat_symbol("]") {
                dims.push(None);
            } else {
                let size = self.expression()?;
                self.expect_symbol("]", "to close the array size")?;
                dims.push(Some(size));
            }
        }
        if dims.is_empty() {
            self.error_at(start, "expected '[' after the array type");
            return Err(Abort);
        }
        // Sized dimensions must come before empty ones (JLS §15.10.1).
        let first_empty = dims.iter().position(Option::is_none);
        if let Some(first_empty) = first_empty
            && dims[first_empty..].iter().any(Option::is_some)
        {
            self.error_at(
                start,
                "cannot specify an array dimension after an empty dimension",
            );
            return Err(Abort);
        }

        let init = if dims[0].is_none() {
            // `new int[] {...}` — all dims empty, initializer required.
            if !self.at_symbol("{") {
                self.error_at(
                    start,
                    "array creation with '[]' needs an initializer { ... }",
                );
                return Err(Abort);
            }
            let Expr::ArrayLiteral { elements, .. } = self.array_literal()? else {
                unreachable!("array_literal returns an ArrayLiteral");
            };
            Some(elements)
        } else {
            None
        };

        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        Ok(Expr::NewArray {
            elem: base,
            dims,
            init,
            span,
        })
    }

    /// `{ e1, e2, ... }` with nested literals for 2D arrays. Only
    /// called where an array initializer is legal.
    fn array_literal(&mut self) -> Parsed<Expr> {
        let start = self.here();
        self.expect_symbol("{", "to open the array initializer")?;
        let mut elements = Vec::new();
        // A trailing comma is legal here (JLS §10.6), so stop on `}` both
        // before an element and after a comma: `{1, 2,}` and `{}` both parse.
        while !self.at_symbol("}") {
            if self.at_symbol("{") {
                elements.push(self.array_literal()?);
            } else {
                elements.push(self.expression()?);
            }
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("}", "to close the array initializer")?;
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        Ok(Expr::ArrayLiteral { elements, span })
    }

    fn arguments(&mut self) -> Parsed<Vec<Expr>> {
        self.expect_symbol("(", "to open the argument list")?;
        let mut args = Vec::new();
        if !self.at_symbol(")") {
            loop {
                args.push(self.expression()?);
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }
        self.expect_symbol(")", "to close the argument list")?;
        Ok(args)
    }

    #[allow(clippy::too_many_lines)] // one grammar production
    fn primary_expression(&mut self) -> Parsed<Expr> {
        let span = self.here();
        match self.peek() {
            Some(TokenKind::IntLiteral(v)) => {
                let value = Literal::Int(*v);
                self.pos += 1;
                Ok(Expr::Literal { value, span })
            }
            Some(TokenKind::LongLiteral(v)) => {
                let value = Literal::Long(*v);
                self.pos += 1;
                Ok(Expr::Literal { value, span })
            }
            Some(TokenKind::FloatLiteral(v)) => {
                let value = Literal::Float(*v);
                self.pos += 1;
                Ok(Expr::Literal { value, span })
            }
            Some(TokenKind::DoubleLiteral(v)) => {
                let value = Literal::Double(*v);
                self.pos += 1;
                Ok(Expr::Literal { value, span })
            }
            Some(TokenKind::StringLiteral(v)) => {
                let value = Literal::Str(v.clone());
                self.pos += 1;
                Ok(Expr::Literal { value, span })
            }
            Some(TokenKind::CharLiteral(v)) => {
                let value = Literal::Char(*v);
                self.pos += 1;
                Ok(Expr::Literal { value, span })
            }
            Some(TokenKind::BooleanLiteral(v)) => {
                let value = Literal::Bool(*v);
                self.pos += 1;
                Ok(Expr::Literal { value, span })
            }
            Some(TokenKind::NullLiteral) => {
                self.pos += 1;
                Ok(Expr::Literal {
                    value: Literal::Null,
                    span,
                })
            }
            Some(TokenKind::Identifier(_)) => {
                let (name, name_span) = self.expect_ident("to start the expression")?;
                Ok(Expr::Name {
                    path: vec![name],
                    span: name_span,
                })
            }
            Some(TokenKind::Symbol("(")) => {
                self.pos += 1;
                let inner = self.expression()?;
                self.expect_symbol(")", "to close the parenthesized expression")?;
                Ok(inner)
            }
            Some(TokenKind::Keyword(Keyword::New)) => self.new_expression(),
            Some(TokenKind::Keyword(Keyword::This)) => {
                let span = self.here();
                self.pos += 1;
                if self.at_symbol("(") {
                    self.error_at(
                        span,
                        "constructor chaining with this(...) is not yet supported by caturra",
                    );
                    return Err(Abort);
                }
                Ok(Expr::This { span })
            }
            Some(TokenKind::Keyword(Keyword::Super)) => {
                let start = self.here();
                self.pos += 1;
                if !self.eat_symbol(".") {
                    self.error_at(start, "expected '.' after 'super' (super.method(...))");
                    return Err(Abort);
                }
                let (method, method_span) = self.expect_ident("after 'super.'")?;
                if !self.at_symbol("(") {
                    // `super.field` — a field access whose receiver is `this`
                    // but whose lookup starts at the superclass.
                    let span = SourceSpan {
                        start: start.start,
                        end: method_span.end,
                    };
                    return Ok(Expr::Field {
                        object: Box::new(Expr::Super { span: start }),
                        name: method,
                        span,
                    });
                }
                let args = self.arguments()?;
                Ok(Expr::SuperMethodCall {
                    method,
                    args,
                    span: SourceSpan {
                        start: start.start,
                        end: method_span.end,
                    },
                })
            }
            Some(TokenKind::Keyword(kw)) if primitive_type_name(*kw).is_some() => {
                // `int.class` / `int[].class` — a (possibly array) primitive
                // class literal.
                let name = primitive_type_name(*kw).expect("checked");
                let start = self.here();
                self.pos += 1;
                // Trailing `[]` pairs: `int[].class`, `int[][].class`.
                let mut dims = 0usize;
                while self.at_symbol("[") {
                    self.pos += 1;
                    if !self.eat_symbol("]") {
                        self.error_here("expected ']' in array class literal");
                        return Err(Abort);
                    }
                    dims += 1;
                }
                if self.eat_symbol(".")
                    && matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Class)))
                {
                    let end = self.here().end;
                    self.pos += 1;
                    let span = SourceSpan {
                        start: start.start,
                        end,
                    };
                    let full = format!("{name}{}", "[]".repeat(dims));
                    Ok(Expr::Field {
                        object: Box::new(Expr::Name {
                            path: vec![full],
                            span,
                        }),
                        name: String::from("class"),
                        span,
                    })
                } else {
                    self.error_at(start, "expected '.class' after a primitive type");
                    Err(Abort)
                }
            }
            _ => {
                self.error_here("expected an expression");
                Err(Abort)
            }
        }
    }
}

/// One parsed enum constant: its name, its constructor arguments, and — for a
/// constant with a body (`PLUS { ... }`) — the name of the synthesized subclass
/// the constant instantiates.
struct EnumConstant {
    name: String,
    args: Vec<Expr>,
    body: Option<String>,
    span: SourceSpan,
}

/// Build the synthesized class for an `enum`. Each constant becomes a
/// `static final E` field initialized with `new E("NAME", ordinal,
/// args...)`; the enum gets hidden `__name`/`__ordinal` instance
/// fields, a constructor that stores them (user constructors are
/// augmented with the two leading parameters), and the standard
/// `values`/`valueOf`/`ordinal`/`name`/`toString` members unless the
/// user supplied them.
#[allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments
)] // one desugaring plan
fn desugar_enum(
    name: String,
    mut interfaces: Vec<String>,
    constants: Vec<EnumConstant>,
    mut fields: Vec<FieldDecl>,
    mut methods: Vec<MethodDecl>,
    mut init_blocks: Vec<InitBlock>,
    nested: Vec<ClassDecl>,
    span: SourceSpan,
) -> ClassDecl {
    let zero = SourceSpan {
        start: span.start,
        end: span.start,
    };
    // Every enum extends `java.lang.Enum<E>`, which implements
    // `Comparable<E>`. Recording it here makes the enum a subtype of
    // Comparable (`Comparable<C> c = C.X`, `Collections.sort(enumList)`), which
    // the synthesized `compareTo` already backs.
    if !interfaces.iter().any(|i| i == "Comparable") {
        interfaces.push(String::from("Comparable"));
    }
    let str_ty = TypeRef::Named(String::from("String"));
    let enum_ty = TypeRef::Named(name.clone());

    let lit_int = |n: i64| Expr::Literal {
        value: Literal::Int(n),
        span: zero,
    };
    let lit_str = |s: &str| Expr::Literal {
        value: Literal::Str(String::from(s)),
        span: zero,
    };
    let var = |n: &str| Expr::Name {
        path: vec![String::from(n)],
        span: zero,
    };

    // Shift user fields AND static-initializer BLOCKS after the synthesized
    // constant initializers so every constant is constructed first in <clinit>
    // (JLS §8.9.2 — the enum constants are implicit `static final` fields
    // declared before any explicit static block). Only the fields were shifted,
    // so an explicit `static { ... }` ran BETWEEN two constant constructions.
    let synth_order = constants.len() + 1;
    for field in &mut fields {
        field.order += synth_order;
    }
    for block in &mut init_blocks {
        block.order += synth_order;
    }

    // Hidden instance fields (no initializer; set by the constructor).
    let mut synth_fields = vec![
        FieldDecl {
            name: String::from("__name"),
            ty: str_ty.clone(),
            is_static: false,
            is_private: true,
            is_final: true,
            init: None,
            order: 0,
            span: zero,
        },
        FieldDecl {
            name: String::from("__ordinal"),
            ty: TypeRef::Int,
            is_static: false,
            is_private: true,
            is_final: true,
            init: None,
            order: 0,
            span: zero,
        },
    ];

    // One `static final E NAME = new E("NAME", i, args...);` per
    // constant, in source order.
    for (index, constant) in constants.iter().enumerate() {
        let mut ctor_args = vec![
            lit_str(&constant.name),
            lit_int(i64::try_from(index).unwrap_or(i64::MAX)),
        ];
        ctor_args.extend(constant.args.iter().cloned());
        // A constant with a body is an instance of its synthesized subclass;
        // one without is an instance of the enum class directly.
        let class = constant.body.clone().unwrap_or_else(|| name.clone());
        synth_fields.push(FieldDecl {
            name: constant.name.clone(),
            ty: enum_ty.clone(),
            is_static: true,
            is_private: false,
            is_final: true,
            init: Some(Expr::NewObject {
                class,
                type_args: Vec::new(),
                args: ctor_args,
                outer: None,
                span: constant.span,
            }),
            order: index,
            span: constant.span,
        });
    }

    synth_fields.extend(fields);

    // Constructor: augment each user constructor with the two leading
    // parameters and the field stores; synthesize one if none exist.
    let store_stmts = || {
        vec![
            Stmt::Assign {
                target: AssignTarget::Field {
                    object: Box::new(Expr::This { span: zero }),
                    name: String::from("__name"),
                },
                op: None,
                value: var("__name"),
                span: zero,
            },
            Stmt::Assign {
                target: AssignTarget::Field {
                    object: Box::new(Expr::This { span: zero }),
                    name: String::from("__ordinal"),
                },
                op: None,
                value: var("__ordinal"),
                span: zero,
            },
        ]
    };
    let lead_params = || {
        vec![
            Param {
                ty: str_ty.clone(),
                name: String::from("__name"),
                is_varargs: false,
                is_final: false,
            },
            Param {
                ty: TypeRef::Int,
                name: String::from("__ordinal"),
                is_varargs: false,
                is_final: false,
            },
        ]
    };

    let has_ctor = methods.iter().any(|m| m.is_constructor);
    if has_ctor {
        for method in &mut methods {
            if method.is_constructor {
                let mut params = lead_params();
                params.append(&mut method.params);
                method.params = params;
                let mut body = store_stmts();
                body.append(&mut method.body);
                method.body = body;
                method.is_private = true;
            }
        }
    } else {
        methods.push(MethodDecl {
            name: name.clone(),
            is_static: false,
            is_public: false,
            is_private: true,
            is_final: false,
            is_constructor: true,
            is_abstract: false,
            type_params: Vec::new(),
            infer_return: None,
            return_type: TypeRef::Void,
            params: lead_params(),
            body: store_stmts(),
            annotations: Vec::new(),
            throws: Vec::new(),
            span: zero,
        });
    }

    let defines = |methods: &[MethodDecl], n: &str| methods.iter().any(|m| m.name == n);

    // `int ordinal() { return __ordinal; }`
    if !defines(&methods, "ordinal") {
        methods.push(simple_return_method(
            "ordinal",
            TypeRef::Int,
            var("__ordinal"),
            zero,
        ));
    }
    // `String name() { return __name; }`
    if !defines(&methods, "name") {
        methods.push(simple_return_method(
            "name",
            str_ty.clone(),
            var("__name"),
            zero,
        ));
    }
    // `String toString() { return __name; }`
    if !defines(&methods, "toString") {
        methods.push(simple_return_method(
            "toString",
            str_ty.clone(),
            var("__name"),
            zero,
        ));
    }

    // `Class getDeclaringClass() { return E.class; }` — the enum TYPE, which
    // for a constant with a body is not `getClass()`: that is the constant's
    // anonymous subclass. `Enum.getDeclaringClass` is final, so this is
    // unconditional (an override is already rejected above).
    methods.push(MethodDecl {
        name: String::from("getDeclaringClass"),
        is_static: false,
        is_public: true,
        is_private: false,
        is_final: false,
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        infer_return: None,
        return_type: TypeRef::Named(String::from("Class")),
        params: Vec::new(),
        body: vec![Stmt::Return {
            value: Some(Expr::Field {
                object: Box::new(Expr::Name {
                    path: vec![name.clone()],
                    span: zero,
                }),
                name: String::from("class"),
                span: zero,
            }),
            span: zero,
        }],
        annotations: Vec::new(),
        throws: Vec::new(),
        span: zero,
    });

    // `int compareTo(E __other) { return __ordinal - __other.__ordinal; }` —
    // Enum implements Comparable by declaration order, which is the ordinal.
    // (`__other.__ordinal` is a private field of the same class, so the access
    // is legal.) A user cannot override compareTo on an enum (javac forbids
    // it), so this is unconditional.
    methods.push(MethodDecl {
        name: String::from("compareTo"),
        is_static: false,
        is_public: true,
        is_private: false,
        is_final: false,
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        infer_return: None,
        return_type: TypeRef::Int,
        params: vec![Param {
            ty: enum_ty.clone(),
            name: String::from("__other"),
            is_varargs: false,
            is_final: false,
        }],
        body: vec![Stmt::Return {
            value: Some(Expr::Binary {
                op: BinaryOp::Sub,
                lhs: Box::new(var("__ordinal")),
                rhs: Box::new(Expr::Field {
                    object: Box::new(var("__other")),
                    name: String::from("__ordinal"),
                    span: zero,
                }),
                span: zero,
            }),
            span: zero,
        }],
        annotations: Vec::new(),
        throws: Vec::new(),
        span: zero,
    });

    // `static E[] values() { return new E[]{ A, B, ... }; }`
    if !defines(&methods, "values") {
        let elements: Vec<Expr> = constants.iter().map(|c| var(&c.name)).collect();
        let values_body = Expr::NewArray {
            elem: enum_ty.clone(),
            dims: vec![None],
            init: Some(elements),
            span: zero,
        };
        methods.push(MethodDecl {
            name: String::from("values"),
            is_static: true,
            is_public: true,
            is_private: false,
            is_final: false,
            is_constructor: false,
            is_abstract: false,
            type_params: Vec::new(),
            infer_return: None,
            return_type: TypeRef::Array(Box::new(enum_ty.clone())),
            params: Vec::new(),
            body: vec![Stmt::Return {
                value: Some(values_body),
                span: zero,
            }],
            annotations: Vec::new(),
            throws: Vec::new(),
            span: zero,
        });
    }

    // `static E valueOf(String __n) {
    //     for (E __e : values()) if (__e.__name.equals(__n)) return __e;
    //     throw new IllegalArgumentException("No enum constant E." + __n);
    // }`
    if !defines(&methods, "valueOf") {
        let match_test = Expr::Call {
            receiver: Some(Box::new(Expr::Field {
                object: Box::new(var("__e")),
                name: String::from("__name"),
                span: zero,
            })),
            method: String::from("equals"),
            args: vec![var("__n")],
            span: zero,
        };
        let loop_body = Stmt::If {
            cond: match_test,
            then: Box::new(Stmt::Return {
                value: Some(var("__e")),
                span: zero,
            }),
            els: None,
            span: zero,
        };
        let for_each = Stmt::ForEach {
            ty: enum_ty.clone(),
            name: String::from("__e"),
            iterable: Expr::Call {
                receiver: None,
                method: String::from("values"),
                args: Vec::new(),
                span: zero,
            },
            body: Box::new(loop_body),
            span: zero,
        };
        let null_check = Stmt::If {
            cond: Expr::Binary {
                op: BinaryOp::Eq,
                lhs: Box::new(var("__n")),
                rhs: Box::new(Expr::Literal {
                    value: Literal::Null,
                    span: zero,
                }),
                span: zero,
            },
            then: Box::new(Stmt::Throw {
                value: Expr::NewObject {
                    class: String::from("NullPointerException"),
                    type_args: Vec::new(),
                    args: vec![lit_str("Name is null")],
                    outer: None,
                    span: zero,
                },
                span: zero,
            }),
            els: None,
            span: zero,
        };
        let throw = Stmt::Throw {
            value: Expr::NewObject {
                class: String::from("IllegalArgumentException"),
                type_args: Vec::new(),
                args: vec![Expr::Binary {
                    op: BinaryOp::Add,
                    lhs: Box::new(lit_str(&format!("No enum constant {name}."))),
                    rhs: Box::new(var("__n")),
                    span: zero,
                }],
                outer: None,
                span: zero,
            },
            span: zero,
        };
        methods.push(MethodDecl {
            name: String::from("valueOf"),
            is_static: true,
            is_public: true,
            is_private: false,
            is_final: false,
            is_constructor: false,
            is_abstract: false,
            type_params: Vec::new(),
            infer_return: None,
            return_type: enum_ty.clone(),
            params: vec![Param {
                ty: str_ty,
                name: String::from("__n"),
                is_varargs: false,
                is_final: false,
            }],
            // `Enum.valueOf` null-checks the name FIRST (JDK:
            // `Objects.requireNonNull(name, "Name is null")`), so a null name
            // is a NullPointerException, not the not-found
            // IllegalArgumentException.
            body: vec![null_check, for_each, throw],
            annotations: Vec::new(),
            throws: Vec::new(),
            span: zero,
        });
    }

    // An enum is implicitly abstract when it declares an abstract method, OR
    // when EVERY constant has a body — the enum class is then never
    // instantiated directly (only its per-constant subclasses are), so it need
    // not implement an interface method itself; each constant's body supplies
    // it. `enum E implements I { A { m(){...} }, B { m(){...} } }` is the case
    // that was refused as "E is not abstract and does not override m".
    let all_constants_have_bodies =
        !constants.is_empty() && constants.iter().all(|c| c.body.is_some());
    let is_abstract = methods.iter().any(|m| m.is_abstract) || all_constants_have_bodies;

    ClassDecl {
        name,
        superclass: None,
        interfaces,
        supertype_args: Vec::new(),
        is_abstract,
        is_interface: false,
        is_enum: true,
        is_public: false,
        is_nested: false,
        enclosing: None,
        is_anonymous: false,
        is_local: false,
        is_inner: false,
        type_params: Vec::new(),
        fields: synth_fields,
        methods,
        init_blocks,
        nested,
        span,
    }
}

/// Recursively hoist nested classes to the top level. Nested classes
/// become independent top-level classes with their simple name (JVM
/// `Outer$Inner` mangling is unnecessary — caturra shares one flat
/// namespace and rejects duplicate simple names).
fn flatten_nested(class: ClassDecl, out: &mut Vec<ClassDecl>) {
    flatten_nested_within(class, "", out);
}

/// `flatten_nested`, carrying the dotted chain of enclosing class names so a
/// nested enum can still name itself the way the JDK does.
fn flatten_nested_within(mut class: ClassDecl, enclosing: &str, out: &mut Vec<ClassDecl>) {
    if class.is_enum && !enclosing.is_empty() {
        qualify_enum_constant_message(&mut class, enclosing);
    }
    let nested = std::mem::take(&mut class.nested);
    let outer = class.name.clone();
    let inner_enclosing = if enclosing.is_empty() {
        outer.clone()
    } else {
        format!("{enclosing}.{outer}")
    };
    out.push(class);
    for mut inner in nested {
        // Hoisting loses the fact that it was nested; the file-name rule
        // needs it, because only a TOP-LEVEL public type is bound to the
        // file name.
        inner.is_nested = true;
        // It also loses sight of the enclosing class, whose STATIC members a
        // nested class names without qualifying (JLS §6.5.6.1) — `counter++`
        // inside `class Outer { static int counter; static class Inner {...} }`.
        // Codegen already falls back to an enclosing class's statics for the
        // anonymous classes it synthesizes; a nested class declared in source
        // is the same situation and was simply never told who enclosed it.
        // Only the *static* fallback opens up: the instance one needs a
        // captured `this`, which a nested class has not got.
        inner.enclosing.get_or_insert_with(|| outer.clone());
        flatten_nested_within(inner, &inner_enclosing, out);
    }
}

/// `Enum.valueOf`'s "No enum constant" message names the enum the way
/// `Class.getCanonicalName()` does — `No enum constant Outer.Level.LOW`, not
/// `No enum constant Level.LOW`. The desugar (which runs while parsing, before
/// anything knows what encloses what) writes the simple name; hoisting is where
/// the enclosing chain becomes known, so the literal is qualified here.
///
/// Matched on the exact string the desugar wrote, so a user-written `valueOf`
/// is left alone.
fn qualify_enum_constant_message(class: &mut ClassDecl, enclosing: &str) {
    let written = format!("No enum constant {}.", class.name);
    let qualified = format!("No enum constant {enclosing}.{}.", class.name);
    for method in &mut class.methods {
        for stmt in &mut method.body {
            replace_string_literal(stmt, &written, &qualified);
        }
    }
}

fn replace_string_literal(stmt: &mut Stmt, from: &str, to: &str) {
    let Stmt::Throw { value, .. } = stmt else {
        return;
    };
    let Expr::NewObject { args, .. } = value else {
        return;
    };
    for arg in args {
        if let Expr::Binary { lhs, .. } = arg
            && let Expr::Literal {
                value: Literal::Str(text),
                ..
            } = lhs.as_mut()
            && text == from
        {
            *text = String::from(to);
        }
    }
}

/// Erase generic type parameters within a class: every `TypeRef` naming a
/// class or method type parameter is rewritten to its erasure — its bound's
/// raw base (`T extends Comparable<T>` → `Comparable`) or `Object` when
/// unbounded. Runtime semantics are unchanged (erasure); this keeps the rest
/// of the compiler generics-unaware while letting a bounded `T`'s methods
/// resolve.
/// JLS §8.4.1 / §6.5.5.1: a STATIC member may not use the class's type
/// parameters — there is no instance to have supplied them. javac: "non-static
/// type variable T cannot be referenced from a static context".
///
/// Must run BEFORE `erase_type_vars`, which replaces those names and leaves
/// nothing to detect. Method-level parameters (`static <T> T id(T)`) are fine
/// and are excluded.
fn check_static_type_variable_use(class: &ClassDecl) -> Vec<(String, SourceSpan)> {
    let mut found = Vec::new();
    if class.type_params.is_empty() {
        return found;
    }
    for method in class.methods.iter().filter(|m| m.is_static) {
        let shadowed: std::collections::HashSet<&str> =
            method.type_params.iter().map(|p| p.name.as_str()).collect();
        let mut check = |ty: &TypeRef| {
            if let Some(name) = names_class_type_param(ty, class, &shadowed) {
                found.push((name, method.span));
            }
        };
        check(&method.return_type);
        for param in &method.params {
            check(&param.ty);
        }
    }
    for field in class.fields.iter().filter(|f| f.is_static) {
        if let Some(name) =
            names_class_type_param(&field.ty, class, &std::collections::HashSet::new())
        {
            found.push((name, field.span));
        }
    }
    found
}

/// The class type parameter this type names, if any (looking through arrays
/// and type arguments).
fn names_class_type_param(
    ty: &TypeRef,
    class: &ClassDecl,
    shadowed: &std::collections::HashSet<&str>,
) -> Option<String> {
    match ty {
        TypeRef::Named(name) => (class.type_params.iter().any(|p| p.name == *name)
            && !shadowed.contains(name.as_str()))
        .then(|| name.clone()),
        TypeRef::Array(inner) => names_class_type_param(inner, class, shadowed),
        TypeRef::Generic { args, .. } => args
            .iter()
            .find_map(|a| names_class_type_param(a, class, shadowed)),
        _ => None,
    }
}

/// JLS §8.4.2: two methods of one class may not have the same ERASURE.
/// `m(List<String>)` and `m(List<Integer>)` are distinct in source and
/// identical on the JVM, so javac refuses the pair — "name clash". caturra
/// accepted both and silently kept whichever it resolved to.
///
/// Must run BEFORE erasure, which makes the two literally identical and so
/// indistinguishable from a plain duplicate method.
fn check_erasure_clashes(class: &ClassDecl) -> Vec<(String, SourceSpan)> {
    let mut clashes = Vec::new();
    let methods: Vec<&MethodDecl> = class.methods.iter().filter(|m| !m.is_abstract).collect();
    for (i, one) in methods.iter().enumerate() {
        for other in methods.iter().skip(i + 1) {
            if one.name != other.name || one.params.len() != other.params.len() {
                continue;
            }
            let (a, b) = (source_params(one), source_params(other));
            // Identical in source too: that is a duplicate method, a different
            // error reported elsewhere, not an erasure clash.
            if a == b {
                continue;
            }
            if erased_param_keys(one) == erased_param_keys(other) {
                clashes.push((
                    format!(
                        "name clash: {}({}) and {}({}) have the same erasure",
                        other.name,
                        b.join(", "),
                        one.name,
                        a.join(", ")
                    ),
                    other.span,
                ));
            }
        }
    }
    clashes
}

/// A parameter list as written, for the diagnostic.
fn source_params(method: &MethodDecl) -> Vec<String> {
    method
        .params
        .iter()
        .map(|p| type_source_text(&p.ty))
        .collect()
}

fn type_source_text(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Array(inner) => format!("{}[]", type_source_text(inner)),
        TypeRef::Generic { base, args } => {
            let args: Vec<String> = args.iter().map(type_source_text).collect();
            format!("{base}<{}>", args.join(", "))
        }
        other => format!("{other:?}").to_lowercase(),
    }
}

/// A parameter list after erasure — type arguments dropped.
fn erased_param_keys(method: &MethodDecl) -> Vec<String> {
    method
        .params
        .iter()
        .map(|p| erased_type_key(&p.ty))
        .collect()
}

fn erased_type_key(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Generic { base, .. } => base.clone(),
        TypeRef::Array(inner) => format!("{}[]", erased_type_key(inner)),
        other => format!("{other:?}"),
    }
}

fn erase_type_vars(class: &mut ClassDecl, synthesized: &mut Vec<ClassDecl>) {
    use std::collections::HashMap;
    let span = class.span;
    // A class with exactly one UNBOUNDED type parameter tracks it: that
    // parameter erases to the `TypeVar` sentinel (enabling cast-free reads).
    // Every other type parameter erases to its bound — `T extends Comparable`
    // becomes `Comparable`, so a bounded `T`'s methods resolve — or to
    // `Object` when unbounded.
    let tracked: Option<String> = match class.type_params.as_slice() {
        [tp] if tp.bound.is_none() => Some(tp.name.clone()),
        _ => None,
    };
    let class_erasures: HashMap<String, TypeRef> = class
        .type_params
        .iter()
        .filter(|tp| Some(&tp.name) != tracked.as_ref())
        .map(|tp| (tp.name.clone(), erasure_target(tp, span, synthesized)))
        .collect();
    let scope = |method: &MethodDecl,
                 synthesized: &mut Vec<ClassDecl>|
     -> (HashMap<String, TypeRef>, Option<String>) {
        let mut to_object = class_erasures.clone();
        for tp in &method.type_params {
            to_object.insert(tp.name.clone(), erasure_target(tp, span, synthesized));
        }
        // A method type parameter shadowing the class one drops tracking.
        let tracked = tracked
            .clone()
            .filter(|name| !method.type_params.iter().any(|tp| &tp.name == name));
        (to_object, tracked)
    };
    for field in &mut class.fields {
        erase_in_type(&mut field.ty, &class_erasures, tracked.as_deref());
        if let Some(init) = &mut field.init {
            erase_in_expr(init, &class_erasures, tracked.as_deref());
        }
    }
    for method in &mut class.methods {
        let (to_object, tracked) = scope(method, synthesized);
        // Record the return-type inference plan BEFORE erasing the types away:
        // if the return is a bare type variable that also names one or more
        // parameter types, the call site can recover the type argument as the
        // join of those arguments (see `MethodDecl::infer_return`).
        method.infer_return = infer_return_plan(method, &to_object);
        erase_in_type(&mut method.return_type, &to_object, tracked.as_deref());
        for param in &mut method.params {
            erase_in_type(&mut param.ty, &to_object, tracked.as_deref());
        }
        for stmt in &mut method.body {
            erase_in_stmt(stmt, &to_object, tracked.as_deref());
        }
    }
    for block in &mut class.init_blocks {
        for stmt in &mut block.body {
            erase_in_stmt(stmt, &class_erasures, tracked.as_deref());
        }
    }
}

/// The return-type inference plan for a method (see `MethodDecl::infer_return`):
/// `Some(indices)` when the declared return type is exactly a type variable
/// about to be erased (`erasures` names it) AND that same variable is the bare
/// type of the parameters at `indices`. The call site joins those arguments'
/// types to recover the type argument. A type variable that constrains no
/// parameter (`<T> T empty()`) cannot be inferred, so it yields `None`.
fn infer_return_plan(
    method: &MethodDecl,
    erasures: &std::collections::HashMap<String, TypeRef>,
) -> Option<Vec<usize>> {
    let TypeRef::Named(ret_var) = &method.return_type else {
        return None;
    };
    if !erasures.contains_key(ret_var) {
        return None;
    }
    let indices: Vec<usize> = method
        .params
        .iter()
        .enumerate()
        .filter(|(_, p)| matches!(&p.ty, TypeRef::Named(name) if name == ret_var))
        .map(|(index, _)| index)
        .collect();
    (!indices.is_empty()).then_some(indices)
}

/// The simple name of a wildcard's bound (`? extends Number` → `"Number"`),
/// or empty when the bound has no nameable base (an array or primitive, which
/// Java forbids anyway) — codegen then treats it as an unresolved bound.
fn wildcard_bound_name(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Generic { base, .. } => base.clone(),
        _ => String::new(),
    }
}

/// The erasure target of a type parameter: its bound's raw base type
/// (`T extends Comparable<T>` → `Comparable`), or `Object` when unbounded
/// or bounded by something without a nameable base (JLS §4.6).
///
/// An INTERSECTION bound (`T extends A & B`) erases to a synthesized interface
/// that extends every bound, appended to `synthesized`. The JVM erasure is the
/// leftmost bound, but using it here would lose `B`'s methods — `t.b()` is
/// legal on a `<T extends A & B>` and reported "cannot find symbol" — so the
/// compiler works against the intersection instead, which inherits both.
fn erasure_target(tp: &TypeParam, span: SourceSpan, synthesized: &mut Vec<ClassDecl>) -> TypeRef {
    let base_name = |ty: &TypeRef| match ty {
        TypeRef::Named(name) => Some(name.clone()),
        TypeRef::Generic { base, .. } => Some(base.clone()),
        _ => None,
    };
    let Some(leftmost) = tp.bound.as_ref().and_then(&base_name) else {
        return TypeRef::Named(String::from("Object"));
    };
    if tp.extra_bounds.is_empty() {
        return TypeRef::Named(leftmost);
    }
    let mut bounds = vec![leftmost];
    bounds.extend(tp.extra_bounds.iter().filter_map(&base_name));
    let name = format!("{INTERSECTION_PREFIX}{}", bounds.join("$"));
    if !synthesized.iter().any(|class| class.name == name) {
        synthesized.push(ClassDecl {
            name: name.clone(),
            enclosing: None,
            superclass: None,
            interfaces: bounds,
            supertype_args: Vec::new(),
            is_abstract: true,
            is_interface: true,
            is_enum: false,
            is_anonymous: false,
            is_local: false,
            is_inner: false,
            is_nested: false,
            is_public: false,
            type_params: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            init_blocks: Vec::new(),
            nested: Vec::new(),
            span,
        });
    }
    TypeRef::Named(name)
}

/// Name prefix of the interface synthesized for an intersection bound
/// (`<T extends Named & Aged>` → `__And$Named$Aged`). A class assigns to one
/// when it implements every bound — the rule that makes the synthesis sound,
/// since no class ever names it.
pub(crate) const INTERSECTION_PREFIX: &str = "__And$";

/// The reserved type name that [`resolve_type`] maps to
/// [`JType::TypeVar`]; it cannot collide with a source identifier.
pub(crate) const TYPEVAR_SENTINEL: &str = "\u{0}TypeVar";

fn erase_in_type(
    ty: &mut TypeRef,
    to_object: &std::collections::HashMap<String, TypeRef>,
    tracked: Option<&str>,
) {
    match ty {
        TypeRef::Named(name) if Some(name.as_str()) == tracked => {
            *ty = TypeRef::Named(String::from(TYPEVAR_SENTINEL));
        }
        TypeRef::Named(name) => {
            if let Some(target) = to_object.get(name) {
                *ty = target.clone();
            }
        }
        TypeRef::Array(inner) => erase_in_type(inner, to_object, tracked),
        TypeRef::Generic { base, args } => {
            if Some(base.as_str()) == tracked {
                *ty = TypeRef::Named(String::from(TYPEVAR_SENTINEL));
            } else if let Some(target) = to_object.get(base) {
                *ty = target.clone();
            } else {
                for arg in args {
                    erase_in_type_arg(arg, to_object, tracked);
                }
            }
        }
        _ => {}
    }
}

/// Erase a type in TYPE-ARGUMENT position (`List<T>`, `Map<String, T>`).
///
/// A bare type variable here does NOT erase to its bound: `void dump(List<T>)`
/// is applicable to a `List<String>`, which a `List<Object>` parameter is not,
/// so erasing the argument to `Object` (or `Comparable`, or `Number`) made
/// every generic method taking a collection unreachable. Instead it becomes a
/// type-variable wildcard — accepts any element, and unlike `? extends` it may
/// still be written to, because a `T` is a real type, not a capture.
///
/// Anything else erases normally, so `List<List<T>>` still works inside.
fn erase_in_type_arg(
    ty: &mut TypeRef,
    to_object: &std::collections::HashMap<String, TypeRef>,
    tracked: Option<&str>,
) {
    if let TypeRef::Named(name) = ty
        && (to_object.contains_key(name) || Some(name.as_str()) == tracked)
    {
        *ty = TypeRef::Named(crate::ast::wildcard_type_name('=', ""));
        return;
    }
    erase_in_type(ty, to_object, tracked);
}

fn erase_in_stmt(
    stmt: &mut Stmt,
    to_object: &std::collections::HashMap<String, TypeRef>,
    tracked: Option<&str>,
) {
    match stmt {
        Stmt::Block(stmts) => {
            for s in stmts {
                erase_in_stmt(s, to_object, tracked);
            }
        }
        Stmt::LocalDecl {
            ty, declarators, ..
        } => {
            erase_in_type(ty, to_object, tracked);
            for d in declarators {
                if let Some(init) = &mut d.init {
                    erase_in_expr(init, to_object, tracked);
                }
            }
        }
        Stmt::Expr(e)
        | Stmt::Throw { value: e, .. }
        | Stmt::Assign { value: e, .. }
        | Stmt::Return { value: Some(e), .. } => erase_in_expr(e, to_object, tracked),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            erase_in_expr(cond, to_object, tracked);
            erase_in_stmt(then, to_object, tracked);
            if let Some(e) = els {
                erase_in_stmt(e, to_object, tracked);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            erase_in_expr(cond, to_object, tracked);
            erase_in_stmt(body, to_object, tracked);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            if let Some(s) = init {
                erase_in_stmt(s, to_object, tracked);
            }
            if let Some(c) = cond {
                erase_in_expr(c, to_object, tracked);
            }
            for s in update {
                erase_in_stmt(s, to_object, tracked);
            }
            erase_in_stmt(body, to_object, tracked);
        }
        Stmt::ForEach {
            ty, iterable, body, ..
        } => {
            erase_in_type(ty, to_object, tracked);
            erase_in_expr(iterable, to_object, tracked);
            erase_in_stmt(body, to_object, tracked);
        }
        Stmt::Switch { selector, arms, .. } => {
            erase_in_expr(selector, to_object, tracked);
            for arm in arms {
                for label in arm.labels.iter_mut().flatten() {
                    erase_in_expr(label, to_object, tracked);
                }
                for s in &mut arm.body {
                    erase_in_stmt(s, to_object, tracked);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            for s in body {
                erase_in_stmt(s, to_object, tracked);
            }
            for c in catches {
                for s in &mut c.body {
                    erase_in_stmt(s, to_object, tracked);
                }
            }
            if let Some(fin) = finally_body {
                for s in fin {
                    erase_in_stmt(s, to_object, tracked);
                }
            }
        }
        Stmt::Labeled { body, .. } => erase_in_stmt(body, to_object, tracked),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                erase_in_expr(a, to_object, tracked);
            }
        }
    }
}

fn erase_in_expr(
    expr: &mut Expr,
    to_object: &std::collections::HashMap<String, TypeRef>,
    tracked: Option<&str>,
) {
    match expr {
        Expr::Cast { ty, operand, .. } => {
            erase_in_type(ty, to_object, tracked);
            erase_in_expr(operand, to_object, tracked);
        }
        Expr::InstanceOf { value, ty, .. } => {
            erase_in_type(ty, to_object, tracked);
            erase_in_expr(value, to_object, tracked);
        }
        Expr::NewArray {
            elem, dims, init, ..
        } => {
            erase_in_type(elem, to_object, tracked);
            for d in dims.iter_mut().flatten() {
                erase_in_expr(d, to_object, tracked);
            }
            if let Some(elements) = init {
                for e in elements {
                    erase_in_expr(e, to_object, tracked);
                }
            }
        }
        Expr::NewObject { args, .. } | Expr::SuperMethodCall { args, .. } => {
            for a in args {
                erase_in_expr(a, to_object, tracked);
            }
        }
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                erase_in_expr(r, to_object, tracked);
            }
            for a in args {
                erase_in_expr(a, to_object, tracked);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            erase_in_expr(lhs, to_object, tracked);
            erase_in_expr(rhs, to_object, tracked);
        }
        Expr::Unary { operand, .. }
        | Expr::Field {
            object: operand, ..
        } => {
            erase_in_expr(operand, to_object, tracked);
        }
        Expr::Index { array, index, .. } => {
            erase_in_expr(array, to_object, tracked);
            erase_in_expr(index, to_object, tracked);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            erase_in_expr(cond, to_object, tracked);
            erase_in_expr(then, to_object, tracked);
            erase_in_expr(els, to_object, tracked);
        }
        Expr::IncDec { target, .. } => erase_in_expr(target, to_object, tracked),
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                erase_in_expr(e, to_object, tracked);
            }
        }
        Expr::MethodRef { qualifier, .. } => erase_in_expr(qualifier, to_object, tracked),
        Expr::Lambda { params, body, .. } => {
            for p in params.iter_mut() {
                if let Some(ty) = &mut p.ty {
                    erase_in_type(ty, to_object, tracked);
                }
            }
            match body {
                LambdaBody::Expr(e) => erase_in_expr(e, to_object, tracked),
                LambdaBody::Block(stmts) => {
                    for s in stmts {
                        erase_in_stmt(s, to_object, tracked);
                    }
                }
            }
        }
        Expr::Assign { target, value, .. } => {
            match target {
                AssignTarget::Index { array, index } => {
                    erase_in_expr(array, to_object, tracked);
                    erase_in_expr(index, to_object, tracked);
                }
                AssignTarget::Field { object, .. } => erase_in_expr(object, to_object, tracked),
                AssignTarget::Var(_) => {}
            }
            erase_in_expr(value, to_object, tracked);
        }
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } | Expr::Super { .. } => {}
    }
}

/// Rewrite every reference to the simple type name `from` (a local class's
/// source name) to its mangled hoisted name `to`, across a run of statements —
/// the tail of the block the class was declared in.
fn rename_class_in_stmts(stmts: &mut [Stmt], from: &str, to: &str) {
    for s in stmts {
        rename_class_in_stmt(s, from, to);
    }
}

fn rename_class_in_type(ty: &mut TypeRef, from: &str, to: &str) {
    match ty {
        TypeRef::Named(name) if name == from => {
            to.clone_into(name);
        }
        TypeRef::Array(inner) => rename_class_in_type(inner, from, to),
        TypeRef::Generic { base, args } => {
            if base == from {
                to.clone_into(base);
            }
            for arg in args {
                rename_class_in_type(arg, from, to);
            }
        }
        _ => {}
    }
}

fn rename_class_in_stmt(stmt: &mut Stmt, from: &str, to: &str) {
    match stmt {
        Stmt::Block(stmts) => rename_class_in_stmts(stmts, from, to),
        Stmt::LocalDecl {
            ty, declarators, ..
        } => {
            rename_class_in_type(ty, from, to);
            for d in declarators {
                if let Some(init) = &mut d.init {
                    rename_class_in_expr(init, from, to);
                }
            }
        }
        Stmt::Expr(e)
        | Stmt::Throw { value: e, .. }
        | Stmt::Assign { value: e, .. }
        | Stmt::Return { value: Some(e), .. } => rename_class_in_expr(e, from, to),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            rename_class_in_expr(cond, from, to);
            rename_class_in_stmt(then, from, to);
            if let Some(e) = els {
                rename_class_in_stmt(e, from, to);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            rename_class_in_expr(cond, from, to);
            rename_class_in_stmt(body, from, to);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            if let Some(s) = init {
                rename_class_in_stmt(s, from, to);
            }
            if let Some(c) = cond {
                rename_class_in_expr(c, from, to);
            }
            for s in update {
                rename_class_in_stmt(s, from, to);
            }
            rename_class_in_stmt(body, from, to);
        }
        Stmt::ForEach {
            ty, iterable, body, ..
        } => {
            rename_class_in_type(ty, from, to);
            rename_class_in_expr(iterable, from, to);
            rename_class_in_stmt(body, from, to);
        }
        Stmt::Switch { selector, arms, .. } => {
            rename_class_in_expr(selector, from, to);
            for arm in arms {
                for label in arm.labels.iter_mut().flatten() {
                    rename_class_in_expr(label, from, to);
                }
                rename_class_in_stmts(&mut arm.body, from, to);
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            rename_class_in_stmts(body, from, to);
            for c in catches {
                for t in &mut c.types {
                    rename_class_in_type(t, from, to);
                }
                rename_class_in_stmts(&mut c.body, from, to);
            }
            if let Some(fin) = finally_body {
                rename_class_in_stmts(fin, from, to);
            }
        }
        Stmt::Labeled { body, .. } => rename_class_in_stmt(body, from, to),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                rename_class_in_expr(a, from, to);
            }
        }
    }
}

#[allow(clippy::too_many_lines)] // one arm per expression kind
fn rename_class_in_expr(expr: &mut Expr, from: &str, to: &str) {
    match expr {
        Expr::Cast { ty, operand, .. } => {
            rename_class_in_type(ty, from, to);
            rename_class_in_expr(operand, from, to);
        }
        Expr::InstanceOf { value, ty, .. } => {
            rename_class_in_type(ty, from, to);
            rename_class_in_expr(value, from, to);
        }
        Expr::NewArray {
            elem, dims, init, ..
        } => {
            rename_class_in_type(elem, from, to);
            for d in dims.iter_mut().flatten() {
                rename_class_in_expr(d, from, to);
            }
            if let Some(elements) = init {
                for e in elements {
                    rename_class_in_expr(e, from, to);
                }
            }
        }
        Expr::NewObject {
            class,
            type_args,
            args,
            ..
        } => {
            if class == from {
                to.clone_into(class);
            }
            for t in type_args {
                rename_class_in_type(t, from, to);
            }
            for a in args {
                rename_class_in_expr(a, from, to);
            }
        }
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                rename_class_in_expr(a, from, to);
            }
        }
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                rename_class_in_expr(r, from, to);
            }
            for a in args {
                rename_class_in_expr(a, from, to);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            rename_class_in_expr(lhs, from, to);
            rename_class_in_expr(rhs, from, to);
        }
        Expr::Unary { operand, .. }
        | Expr::Field {
            object: operand, ..
        } => {
            rename_class_in_expr(operand, from, to);
        }
        Expr::Index { array, index, .. } => {
            rename_class_in_expr(array, from, to);
            rename_class_in_expr(index, from, to);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            rename_class_in_expr(cond, from, to);
            rename_class_in_expr(then, from, to);
            rename_class_in_expr(els, from, to);
        }
        Expr::IncDec { target, .. } => rename_class_in_expr(target, from, to),
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                rename_class_in_expr(e, from, to);
            }
        }
        Expr::MethodRef { qualifier, .. } => rename_class_in_expr(qualifier, from, to),
        Expr::Lambda { params, body, .. } => {
            for p in params.iter_mut() {
                if let Some(ty) = &mut p.ty {
                    rename_class_in_type(ty, from, to);
                }
            }
            match body {
                LambdaBody::Expr(e) => rename_class_in_expr(e, from, to),
                LambdaBody::Block(stmts) => rename_class_in_stmts(stmts, from, to),
            }
        }
        Expr::Assign { target, value, .. } => {
            match target {
                AssignTarget::Index { array, index } => {
                    rename_class_in_expr(array, from, to);
                    rename_class_in_expr(index, from, to);
                }
                AssignTarget::Field { object, .. } => rename_class_in_expr(object, from, to),
                AssignTarget::Var(_) => {}
            }
            rename_class_in_expr(value, from, to);
        }
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } | Expr::Super { .. } => {}
    }
}

/// Rewrite `from` to `to` inside a class declaration — its supertypes, member
/// signatures, and method/initializer bodies — so a local class can name itself
/// (recursion) or an earlier sibling local class.
fn rename_class_in_class(class: &mut ClassDecl, from: &str, to: &str) {
    if class.superclass.as_deref() == Some(from) {
        class.superclass = Some(to.to_owned());
    }
    for iface in &mut class.interfaces {
        if iface == from {
            to.clone_into(iface);
        }
    }
    for f in &mut class.fields {
        rename_class_in_type(&mut f.ty, from, to);
        if let Some(init) = &mut f.init {
            rename_class_in_expr(init, from, to);
        }
    }
    for m in &mut class.methods {
        for p in &mut m.params {
            rename_class_in_type(&mut p.ty, from, to);
        }
        rename_class_in_type(&mut m.return_type, from, to);
        rename_class_in_stmts(&mut m.body, from, to);
    }
    for b in &mut class.init_blocks {
        rename_class_in_stmts(&mut b.body, from, to);
    }
    for n in &mut class.nested {
        rename_class_in_class(n, from, to);
    }
}

/// A `Type m() { return expr; }` helper for synthesized enum methods.
fn simple_return_method(
    name: &str,
    return_type: TypeRef,
    value: Expr,
    span: SourceSpan,
) -> MethodDecl {
    MethodDecl {
        name: String::from(name),
        is_static: false,
        is_public: true,
        is_private: false,
        is_final: false,
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        infer_return: None,
        return_type,
        params: Vec::new(),
        body: vec![Stmt::Return {
            value: Some(value),
            span,
        }],
        annotations: Vec::new(),
        throws: Vec::new(),
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_ok(source: &str) -> CompilationUnit {
        let (tokens, lex_errors) = lex("Test.java", source);
        assert!(lex_errors.is_empty(), "lex errors: {lex_errors:?}");
        let (unit, errors) = parse("Test.java", tokens);
        assert!(errors.is_empty(), "parse errors: {errors:?}");
        unit
    }

    fn parse_errors(source: &str) -> Vec<Diagnostic> {
        let (tokens, _) = lex("Test.java", source);
        let (_, errors) = parse("Test.java", tokens);
        errors
    }

    /// A keyword that cannot start a statement must be diagnosed as javac does,
    /// without blaming caturra — the level generator reads these strings to tell
    /// an engine gap apart from a student's mistake, so mislabelling invalid
    /// Java as "not supported by caturra" would silently drop the level.
    #[test]
    fn invalid_statement_keywords_use_javac_wording_and_never_blame_caturra() {
        let in_main = |body: &str| {
            format!("public class Main {{ public static void main(String[] a) {{ {body} }} }}")
        };
        // Wording checked against a real javac.
        let invalid = [
            ("public class Inner {}", "illegal start of expression"),
            ("static int q = 1;", "illegal start of expression"),
            ("private int q = 1;", "illegal start of expression"),
            ("void q;", "illegal start of expression"),
            ("import java.util.List;", "illegal start of expression"),
            ("extends Object", "illegal start of expression"),
            ("case 1:", "orphaned case"),
            ("default:", "orphaned default"),
            ("catch (Exception e) {}", "'catch' without 'try'"),
            ("finally { }", "'finally' without 'try'"),
            ("interface I {}", "interface not allowed here"),
            ("enum E { A }", "enum types must not be local"),
            ("abstract void q();", "class, interface, or enum expected"),
        ];
        for (body, want) in invalid {
            let errors = parse_errors(&in_main(body));
            let first = &errors.first().expect(body).message;
            assert_eq!(first, want, "wrong message for `{body}`");
            assert!(
                !first.contains("caturra") && !first.contains("not supported"),
                "invalid Java `{body}` must not be reported as a caturra limitation: {first}"
            );
        }

        // Valid Java that caturra does not implement DOES say so, so the corpus
        // tooling can recognise it as an engine gap.
        let sync = parse_errors(&in_main("synchronized (a) { }"));
        assert!(
            sync.first()
                .expect("synchronized")
                .message
                .contains("caturra"),
            "`synchronized` is valid Java we don't implement; say so"
        );

        // `assert` IS implemented now (a runtime no-op, assertions off) — it
        // parses cleanly, and its condition is still type-checked.
        assert!(parse_errors(&in_main("assert 1 > 0 : \"x\";")).is_empty());
        // A local class in a block IS implemented now — it parses cleanly.
        assert!(parse_errors(&in_main("class Inner {} new Inner();")).is_empty());
        // But a class in a non-block statement position is invalid Java, and the
        // message is javac's, with no mention of caturra.
        let bad_pos = parse_errors(&in_main("if (true) class L {}"));
        let first = &bad_pos.first().expect("if (true) class L {}").message;
        assert_eq!(
            first,
            "class, interface or enum declaration not allowed here"
        );
        assert!(!first.contains("caturra"));

        // `new Foo();` IS a statement expression (JLS 14.8); `new int[3];` is
        // not, and javac rejects it too.
        assert!(parse_errors(&in_main("new Object();")).is_empty());
        let array_new = parse_errors(&in_main("new int[3];"));
        assert_eq!(
            array_new.first().expect("new int[3];").message,
            "this expression is not a statement in Java"
        );
    }

    #[test]
    fn parses_hello_world() {
        let unit = parse_ok(
            r#"
            public class Main {
                public static void main(String[] args) {
                    System.out.println("Hello, World!");
                }
            }
            "#,
        );
        assert_eq!(unit.classes.len(), 1);
        let class = &unit.classes[0];
        assert_eq!(class.name, "Main");
        assert_eq!(class.methods.len(), 1);
        let main = &class.methods[0];
        assert_eq!(main.name, "main");
        assert!(main.is_static && main.is_public);
        assert_eq!(main.return_type, TypeRef::Void);
        assert_eq!(main.params.len(), 1);
        assert_eq!(
            main.params[0].ty,
            TypeRef::Array(Box::new(TypeRef::Named("String".into())))
        );

        let Stmt::Expr(Expr::Call {
            receiver,
            method,
            args,
            ..
        }) = &main.body[0]
        else {
            panic!("expected a call statement, got {:?}", main.body[0]);
        };
        assert_eq!(method, "println");
        assert_eq!(args.len(), 1);
        let Some(receiver) = receiver else {
            panic!("expected a receiver")
        };
        assert_eq!(
            **receiver,
            Expr::Name {
                path: vec!["System".into(), "out".into()],
                span: receiver.span(),
            }
        );
    }

    #[test]
    fn parses_multiple_statements_and_literals() {
        let unit = parse_ok(
            r#"
            class Demo {
                static void show() {
                    System.out.println(42);
                    System.out.println(3.5);
                    System.out.println(true);
                    System.out.println('x');
                    System.out.println();
                    System.err.println("uh oh");
                }
            }
            "#,
        );
        assert_eq!(unit.classes[0].methods[0].body.len(), 6);
    }

    #[test]
    fn unsupported_statements_get_friendly_messages() {
        let errors = parse_errors(
            r#"
            class Main {
                static void run() {
                    synchronized (this) { }
                    System.out.println("still parsed");
                }
            }
            "#,
        );
        let messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
        assert!(!messages.is_empty(), "{messages:?}");
        // `synchronized` is valid Java that caturra does not implement, so the
        // message names caturra — which is how the corpus tooling recognises an
        // engine gap rather than a mistake in the student's source.
        assert!(
            messages[0].contains("not supported by caturra"),
            "{messages:?}"
        );
    }

    #[test]
    fn parses_if_else_with_dangling_else() {
        let unit = parse_ok(
            r#"
            class Main {
                static void run() {
                    int x = 1;
                    if (x > 0)
                        if (x > 10) System.out.println("big");
                        else System.out.println("small");
                }
            }
            "#,
        );
        // The else must bind to the INNER if.
        let Stmt::If {
            then, els: None, ..
        } = &unit.classes[0].methods[0].body[1]
        else {
            panic!("outer if must have no else");
        };
        let Stmt::If { els: Some(_), .. } = &**then else {
            panic!("inner if must own the else");
        };
    }

    #[test]
    fn parses_loops_and_jumps() {
        let unit = parse_ok(
            r"
            class Main {
                static void run() {
                    int total = 0;
                    for (int i = 0, j = 10; i < j; i++, j--) {
                        total += i;
                        if (total > 5) break;
                    }
                    while (total > 0) total--;
                    do { total++; } while (total < 3);
                    for (;;) break;
                }
            }
            ",
        );
        let body = &unit.classes[0].methods[0].body;
        let Stmt::For {
            init: Some(_),
            cond: Some(_),
            update,
            ..
        } = &body[1]
        else {
            panic!("expected full for loop, got {:?}", body[1]);
        };
        assert_eq!(update.len(), 2);
        assert!(matches!(&body[2], Stmt::While { .. }));
        assert!(matches!(&body[3], Stmt::DoWhile { .. }));
        let Stmt::For {
            init: None,
            cond: None,
            update,
            ..
        } = &body[4]
        else {
            panic!("expected for(;;), got {:?}", body[4]);
        };
        assert!(update.is_empty());
    }

    #[test]
    fn assignment_is_an_expression() {
        // Assignment is an expression (JLS §15.26): it parses as an argument and
        // in a loop condition, and `a = b = c` is right-associative.
        parse_ok(
            r"class M {
                static void f() {
                    int a, b, c;
                    a = b = c = 5;
                    System.out.println(a = 7);
                    String s;
                    while ((s = next()) != null) { }
                }
                static String next() { return null; }
            }",
        );
    }

    #[test]
    fn parses_for_each_and_array_syntax() {
        let unit = parse_ok(
            r"
            class M {
                static void f() {
                    int[] a = {1, 2, 3};
                    int[][] grid = new int[2][3];
                    String[] names = new String[] {};
                    a[0] = 5;
                    a[1] += 2;
                    a[2]++;
                    grid[1][2] = a[0] + a.length;
                    int total = 0;
                    for (int x : a) total += x;
                }
            }
            ",
        );
        let body = &unit.classes[0].methods[0].body;
        let Stmt::LocalDecl { declarators, .. } = &body[0] else {
            panic!("expected declaration");
        };
        assert!(matches!(
            declarators[0].init,
            Some(Expr::ArrayLiteral { .. })
        ));
        let Stmt::Assign {
            target: AssignTarget::Index { .. },
            ..
        } = &body[3]
        else {
            panic!("expected element assignment, got {:?}", body[3]);
        };
        assert!(matches!(&body[8], Stmt::ForEach { name, .. } if name == "x"));
    }

    #[test]
    fn declaration_as_branch_body_needs_braces() {
        let errors = parse_errors(r"class M { static void f() { if (true) int x = 1; } }");
        assert!(
            errors[0].message.contains("needs braces"),
            "{}",
            errors[0].message
        );
    }

    #[test]
    fn enum_desugars_to_a_class_with_synthesized_members() {
        let unit = parse_ok(r"enum Suit { HEARTS, SPADES; int rank() { return 1; } }");
        let class = &unit.classes[0];
        assert!(class.is_enum);
        // Two constants as static fields, plus hidden __name/__ordinal.
        assert!(
            class
                .fields
                .iter()
                .any(|f| f.name == "HEARTS" && f.is_static)
        );
        assert!(
            class
                .fields
                .iter()
                .any(|f| f.name == "__ordinal" && !f.is_static)
        );
        // Synthesized accessors + the user method.
        for expected in ["values", "valueOf", "ordinal", "name", "toString", "rank"] {
            assert!(
                class.methods.iter().any(|m| m.name == expected),
                "missing {expected}"
            );
        }
    }

    #[test]
    fn labeled_break_and_continue_parse() {
        let unit = parse_ok(
            r"class M { static void f() {
                outer:
                for (int i = 0; i < 3; i++) {
                    for (int j = 0; j < 3; j++) {
                        if (j == 1) continue outer;
                        if (i == 2) break outer;
                    }
                }
            } }",
        );
        let body = &unit.classes[0].methods[0].body;
        assert!(matches!(body[0], Stmt::Labeled { .. }), "{:?}", body[0]);
    }

    #[test]
    fn parses_local_declarations() {
        let unit = parse_ok(
            r#"
            class Main {
                static void run() {
                    int a = 1, b;
                    final double d = 2.5;
                    String s = "hi";
                    char c = 'x';
                    boolean flag = true;
                }
            }
            "#,
        );
        let body = &unit.classes[0].methods[0].body;
        assert_eq!(body.len(), 5);
        let Stmt::LocalDecl {
            ty,
            is_final,
            declarators,
            ..
        } = &body[0]
        else {
            panic!("expected a declaration, got {:?}", body[0]);
        };
        assert_eq!(*ty, TypeRef::Int);
        assert!(!is_final);
        assert_eq!(declarators.len(), 2);
        assert_eq!(declarators[0].name, "a");
        assert!(declarators[0].init.is_some());
        assert!(declarators[1].init.is_none());
        let Stmt::LocalDecl { is_final: true, .. } = &body[1] else {
            panic!("expected final declaration");
        };
    }

    #[test]
    fn parses_assignments_and_increments() {
        let unit = parse_ok(
            r"
            class Main {
                static void run() {
                    int x = 0;
                    x = 5;
                    x += 2;
                    x++;
                    --x;
                }
            }
            ",
        );
        let body = &unit.classes[0].methods[0].body;
        let Stmt::Assign {
            target: AssignTarget::Var(name),
            op: None,
            ..
        } = &body[1]
        else {
            panic!("expected plain assignment");
        };
        assert_eq!(name, "x");
        let Stmt::Assign {
            op: Some(BinaryOp::Add),
            ..
        } = &body[2]
        else {
            panic!("expected compound assignment");
        };
        // x++ and --x both lower to compound assignments by 1.
        let Stmt::Assign {
            op: Some(BinaryOp::Add),
            value,
            ..
        } = &body[3]
        else {
            panic!("expected x++ lowering");
        };
        assert!(matches!(
            value,
            Expr::Literal {
                value: Literal::Int(1),
                ..
            }
        ));
        let Stmt::Assign {
            op: Some(BinaryOp::Sub),
            ..
        } = &body[4]
        else {
            panic!("expected --x lowering");
        };
    }

    #[test]
    fn precedence_binds_multiplication_tighter_than_addition() {
        let unit = parse_ok(r"class M { static void f() { System.out.println(1 + 2 * 3); } }");
        let Stmt::Expr(Expr::Call { args, .. }) = &unit.classes[0].methods[0].body[0] else {
            panic!("expected call");
        };
        let Expr::Binary {
            op: BinaryOp::Add,
            rhs,
            ..
        } = &args[0]
        else {
            panic!("expected + at the top, got {:?}", args[0]);
        };
        assert!(matches!(
            **rhs,
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn precedence_comparisons_and_logic() {
        let unit = parse_ok(
            r"class M { static void f() { System.out.println(1 < 2 && 3 >= 2 || !false); } }",
        );
        let Stmt::Expr(Expr::Call { args, .. }) = &unit.classes[0].methods[0].body[0] else {
            panic!("expected call");
        };
        // || at the top; && below it; comparisons below that.
        let Expr::Binary {
            op: BinaryOp::Or,
            lhs,
            rhs,
            ..
        } = &args[0]
        else {
            panic!("expected || at the top, got {:?}", args[0]);
        };
        assert!(matches!(
            **lhs,
            Expr::Binary {
                op: BinaryOp::And,
                ..
            }
        ));
        assert!(matches!(
            **rhs,
            Expr::Unary {
                op: UnaryOp::Not,
                ..
            }
        ));
    }

    #[test]
    fn negative_int_min_literal_folds() {
        let unit = parse_ok(r"class M { static void f() { System.out.println(-2147483648); } }");
        let Stmt::Expr(Expr::Call { args, .. }) = &unit.classes[0].methods[0].body[0] else {
            panic!("expected call");
        };
        assert!(
            matches!(&args[0], Expr::Literal { value: Literal::Int(v), .. } if *v == -2_147_483_648)
        );
    }

    #[test]
    fn parses_primitive_casts() {
        let unit = parse_ok(r"class M { static void f() { System.out.println((int) 2.9); } }");
        let Stmt::Expr(Expr::Call { args, .. }) = &unit.classes[0].methods[0].body[0] else {
            panic!("expected call");
        };
        let Expr::Cast {
            ty: TypeRef::Int, ..
        } = &args[0]
        else {
            panic!("expected cast, got {:?}", args[0]);
        };
    }

    #[test]
    fn parenthesized_expression_is_not_a_cast() {
        let unit = parse_ok(r"class M { static void f() { System.out.println((1 + 2) * 3); } }");
        let Stmt::Expr(Expr::Call { args, .. }) = &unit.classes[0].methods[0].body[0] else {
            panic!("expected call");
        };
        assert!(matches!(
            &args[0],
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn expression_statements_must_be_calls() {
        let errors = parse_errors(r"class M { static void f() { int x = 1; x + 1; } }");
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].message.contains("not a statement"),
            "{}",
            errors[0].message
        );
    }

    #[test]
    fn parses_fields_constructors_and_this() {
        let unit = parse_ok(
            r#"
            class Account {
                private double balance;
                private static int count = 0;
                public final String id = "A";

                public Account(double start) {
                    this.balance = start;
                }

                public double getBalance() { return this.balance; }
            }
            class Uses {
                static void f() {
                    Account a = new Account(100.0);
                    System.out.println(a.getBalance());
                }
            }
            "#,
        );
        let account = &unit.classes[0];
        assert_eq!(account.fields.len(), 3);
        assert!(account.fields[0].is_private && !account.fields[0].is_static);
        assert!(account.fields[1].is_static && account.fields[1].init.is_some());
        assert!(account.fields[2].is_final);
        let ctor = &account.methods[0];
        assert!(ctor.is_constructor);
        assert_eq!(ctor.params.len(), 1);
        // this.balance = start; parses as a field assignment on `this`.
        let Stmt::Assign {
            target: AssignTarget::Field { object, name },
            ..
        } = &ctor.body[0]
        else {
            panic!("expected field assignment, got {:?}", ctor.body[0]);
        };
        assert!(matches!(**object, Expr::This { .. }));
        assert_eq!(name, "balance");
        // new Account(100.0) in the second class.
        let uses_body = &unit.classes[1].methods[0].body;
        let Stmt::LocalDecl { declarators, .. } = &uses_body[0] else {
            panic!("expected declaration");
        };
        assert!(matches!(declarators[0].init, Some(Expr::NewObject { .. })));
    }

    #[test]
    fn imports_are_ignored_and_packages_rejected() {
        let unit = parse_ok("import java.util.Scanner;\nimport java.util.ArrayList;\nclass A { }");
        assert_eq!(unit.classes.len(), 1);

        let errors = parse_errors("package com.example;\nclass A { }");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("package"));
    }
}
