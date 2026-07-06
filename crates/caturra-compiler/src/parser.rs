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
    Stmt, SwitchArm, TypeRef, UnaryOp,
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
        pending_annotations: Vec::new(),
    };
    let unit = parser.compilation_unit();
    (unit, parser.diagnostics)
}

/// The friendly message for statement-starting keywords caturra doesn't
/// support yet; `None` when the keyword can begin a real statement.
/// The source spelling of a primitive type keyword (for `int.class`).
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

fn unsupported_statement_keyword(keyword: Keyword) -> Option<&'static str> {
    match keyword {
        Keyword::Else => Some("'else' without a matching 'if'"),
        Keyword::Var => Some("'var' is not supported by caturra; write the type explicitly"),
        Keyword::New => Some("object creation with 'new' is not yet supported by caturra"),
        Keyword::Int
        | Keyword::Double
        | Keyword::Boolean
        | Keyword::Char
        | Keyword::Final
        | Keyword::This
        | Keyword::Long
        | Keyword::Float
        | Keyword::Byte
        | Keyword::Short => None,
        _ => Some("this statement is not yet supported by caturra"),
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
}

/// Parsed class-level modifiers.
#[derive(Debug, Default, Clone, Copy)]
struct ClassModifiers {
    is_abstract: bool,
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
    pending_annotations: Vec<Annotation>,
}

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
                        for class in &mut classes[first..] {
                            erase_type_vars(class);
                        }
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
        for class in anon {
            let first = classes.len();
            flatten_nested(class, &mut classes);
            for class in &mut classes[first..] {
                erase_type_vars(class);
            }
        }
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
                Some(TokenKind::Keyword(Keyword::Protected | Keyword::Default)) => {
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
        self.type_after_modifiers(start, modifiers.is_abstract)
    }

    /// Parse a class/interface/enum whose modifiers were already
    /// consumed (shared by top-level and nested declarations).
    fn type_after_modifiers(
        &mut self,
        start: SourceSpan,
        is_abstract_modifier: bool,
    ) -> Parsed<ClassDecl> {
        if self.at_keyword(Keyword::Enum) {
            return self.enum_decl(start);
        }
        let is_interface = self.eat_keyword(Keyword::Interface);
        if !is_interface && !self.eat_keyword(Keyword::Class) {
            self.error_here("expected a class declaration");
            return Err(Abort);
        }
        let (name, name_span) = self.expect_ident("for the class")?;
        let type_params = self.parse_type_params()?;

        let mut superclass = None;
        let mut interfaces = Vec::new();
        if self.eat_keyword(Keyword::Extends) {
            if is_interface {
                // Interfaces extend other interfaces (a list).
                loop {
                    let (parent, _) = self.expect_ident("after 'extends'")?;
                    self.skip_type_args();
                    interfaces.push(parent);
                    if !self.eat_symbol(",") {
                        break;
                    }
                }
            } else {
                let (parent, _) = self.expect_ident("after 'extends'")?;
                self.skip_type_args();
                superclass = Some(parent);
            }
        }
        if self.eat_keyword(Keyword::Implements) {
            if is_interface {
                self.error_here("interfaces cannot implement");
                return Err(Abort);
            }
            loop {
                let (parent, _) = self.expect_ident("after 'implements'")?;
                self.skip_type_args();
                interfaces.push(parent);
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }

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
            if let Ok(member) = self.member(&name) {
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

        Ok(ClassDecl {
            name,
            superclass,
            interfaces,
            is_abstract: is_abstract_modifier || is_interface,
            is_interface,
            is_enum: false,
            is_anonymous: false,
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
        let mut constants: Vec<(String, Vec<Expr>, SourceSpan)> = Vec::new();
        while let Some(TokenKind::Identifier(_)) = self.peek() {
            let (const_name, const_span) = self.expect_ident("for the enum constant")?;
            let args = if self.at_symbol("(") {
                self.arguments()?
            } else {
                Vec::new()
            };
            constants.push((const_name, args, const_span));
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
            if let Ok(member) = self.member(&name) {
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
                Some(TokenKind::Keyword(Keyword::Public | Keyword::Final)) => {
                    self.pos += 1;
                }
                Some(TokenKind::Symbol("@")) => self.skip_annotation(),
                _ => return modifiers,
            }
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per member kind
    fn member(&mut self, class_name: &str) -> Parsed<Member> {
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
            let nested = self.type_after_modifiers(start, modifiers.is_abstract)?;
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
            let (params, body) = self.method_rest(name_span, false)?;
            return Ok(Member::Method(MethodDecl {
                name,
                is_static: false,
                is_public: modifiers.is_public,
                is_private: modifiers.is_private,
                is_constructor: true,
                is_abstract: false,
                type_params: Vec::new(),
                return_type: TypeRef::Void,
                params,
                body: body.unwrap_or_default(),
                annotations: annotations.clone(),
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
                let init = if self.eat_symbol("=") {
                    if self.at_symbol("{") {
                        Some(self.array_literal()?)
                    } else {
                        Some(self.expression()?)
                    }
                } else {
                    None
                };
                fields.push(FieldDecl {
                    name: current.0,
                    ty: member_type.clone(),
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

        let (params, body) = self.method_rest(name_span, true)?;
        let is_abstract = body.is_none();
        Ok(Member::Method(MethodDecl {
            name,
            is_static: modifiers.is_static,
            is_public: modifiers.is_public,
            is_private: modifiers.is_private,
            is_constructor: false,
            is_abstract,
            type_params: method_type_params,
            return_type: member_type,
            params,
            body: body.unwrap_or_default(),
            annotations,
            span: SourceSpan {
                start: start.start,
                end: name_span.end,
            },
        }))
    }

    /// Parameter list and body, shared by methods and constructors.
    /// With `allow_abstract`, a `;` instead of a body yields `None`.
    fn method_rest(
        &mut self,
        name_span: SourceSpan,
        allow_abstract: bool,
    ) -> Parsed<(Vec<Param>, Option<Vec<Stmt>>)> {
        self.expect_symbol("(", "to open the parameter list")?;
        let mut params = Vec::new();
        if !self.at_symbol(")") {
            loop {
                let mut ty = self.type_ref()?;
                // Varargs: `Type... name` — the parameter is an array.
                let is_varargs = self.eat_symbol("...");
                if is_varargs {
                    ty = TypeRef::Array(Box::new(ty));
                }
                let (param_name, _) = self.expect_ident("for the parameter")?;
                params.push(Param {
                    ty,
                    name: param_name,
                    is_varargs,
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

        // `throws FileNotFoundException, ...` — accepted and ignored
        // (caturra does not enforce checked exceptions).
        if self.eat_keyword(Keyword::Throws) {
            loop {
                self.expect_ident("after 'throws'")?;
                // Qualified exception names: `throws java.io.IOException`.
                while self.at_symbol(".")
                    && matches!(self.peek_at(1), Some(TokenKind::Identifier(_)))
                {
                    self.pos += 1;
                    self.expect_ident("in the qualified exception")?;
                }
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
            return Ok((params, None));
        }
        self.expect_symbol("{", "to open the method body")?;
        let body = self.block_body();
        Ok((params, Some(body)))
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
                TokenKind::Symbol(">>>") => depth -= 3,
                _ => {}
            }
            self.pos += 1;
            if depth <= 0 {
                break;
            }
        }
    }

    /// Parse a `<T, U extends Bound, ...>` type-parameter list,
    /// returning the parameter names (bounds are erased to `Object`).
    fn parse_type_params(&mut self) -> Parsed<Vec<String>> {
        if !self.at_symbol("<") {
            return Ok(Vec::new());
        }
        self.pos += 1; // '<'
        let mut names = Vec::new();
        if !self.at_symbol(">") {
            loop {
                let (name, _) = self.expect_ident("for the type parameter")?;
                names.push(name);
                // `extends Bound & Other` — parsed and discarded.
                if self.eat_keyword(Keyword::Extends) {
                    self.type_ref()?;
                    while self.eat_symbol("&") {
                        self.type_ref()?;
                    }
                }
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }
        self.expect_symbol(">", "to close the type parameters")?;
        Ok(names)
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
                                // Wildcard `?` / `? extends T` / `? super T` —
                                // generics erase, so track it as Object.
                                self.pos += 1;
                                if matches!(
                                    self.peek(),
                                    Some(TokenKind::Keyword(Keyword::Extends | Keyword::Super))
                                ) {
                                    self.pos += 1;
                                    let _ = self.type_ref()?;
                                }
                                args.push(TypeRef::Named(String::from("Object")));
                            } else {
                                args.push(self.type_ref()?);
                            }
                            if !self.eat_symbol(",") {
                                break;
                            }
                        }
                    }
                    if self.at_symbol(">>") {
                        self.error_here(
                            "nested generic types (like ArrayList<ArrayList<...>>) are not \
                             supported by caturra",
                        );
                        return Err(Abort);
                    }
                    self.expect_symbol(">", "to close the type arguments")?;
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
        while !self.at_symbol("}") {
            if self.peek().is_none() {
                self.error_here("expected '}' to close the block");
                break;
            }
            if let Ok(Some(stmt)) = self.statement() {
                statements.push(stmt);
            } else {
                self.recover_to_statement_boundary();
            }
        }
        self.eat_symbol("}");
        statements
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
            && let Some(message) = unsupported_statement_keyword(*keyword)
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

        if let Some(op) = self.assignment_operator() {
            let Some(target) = assignment_target(&expr) else {
                self.error_at(
                    expr.span(),
                    "the left side of an assignment must be a variable or array element",
                );
                return Err(Abort);
            };
            let start = expr.span().start;
            self.pos += 1;
            let value = self.expression()?;
            let end = value.span().end;
            return Ok(Stmt::Assign {
                target,
                op,
                value,
                span: SourceSpan { start, end },
            });
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

        if !matches!(expr, Expr::Call { .. } | Expr::SuperMethodCall { .. }) {
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

    /// `( condition )` with a hint for the classic `=` vs `==` typo.
    fn paren_condition(&mut self, context: &str) -> Parsed<Expr> {
        self.expect_symbol("(", &format!("after '{context}'"))?;
        let cond = self.expression()?;
        if self.at_symbol("=") {
            self.error_here("assignment is not a condition — did you mean '=='?");
            return Err(Abort);
        }
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
        self.expect_symbol("{", "after 'try'")?;
        let body = self.block_body();

        let mut catches = Vec::new();
        while self.at_keyword(Keyword::Catch) {
            let clause_start = self.here();
            self.pos += 1;
            self.expect_symbol("(", "after 'catch'")?;
            let ty = self.type_ref()?;
            let (name, _) = self.expect_ident("for the caught exception")?;
            self.expect_symbol(")", "after the catch parameter")?;
            self.expect_symbol("{", "after the catch parameter")?;
            let catch_body = self.block_body();
            catches.push(CatchClause {
                ty,
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
        if catches.is_empty() && finally_body.is_none() {
            self.error_at(
                start,
                "'try' needs at least one 'catch' clause or a 'finally' block",
            );
            return Err(Abort);
        }
        Ok(Stmt::Try {
            body,
            catches,
            finally_body,
            span: SourceSpan {
                start: start.start,
                end: self.here().start,
            },
        })
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
            let ty = self.type_ref()?;
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
                // Only type names, commas, dots, and nested `<>` appear
                // in a type-argument list; anything else means this was
                // a comparison expression.
                // ...plus wildcards (`Class<?>`, `List<? extends T>`).
                TokenKind::Identifier(_)
                | TokenKind::Symbol("," | "." | "?")
                | TokenKind::Keyword(Keyword::Extends | Keyword::Super) => {}
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
        let ty = self.type_ref()?;

        let mut declarators = Vec::new();
        loop {
            let (name, name_span) = self.expect_ident("for the variable")?;
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
        self.ternary()
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
            LambdaBody::Expr(Box::new(self.expression()?))
        };
        let span = SourceSpan {
            start: start.start,
            end: self.here().start,
        };
        Ok(Expr::Lambda { params, body, span })
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
        // (`(a) - b` stays arithmetic).
        if self.at_symbol("(")
            && let Some(after) = self.scan_cast_type()
            && matches!(
                self.peek_at(after),
                Some(
                    TokenKind::Identifier(_)
                        | TokenKind::Keyword(Keyword::New | Keyword::This)
                        | TokenKind::StringLiteral(_)
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
    /// `new Anon$N()` expression. Constructor arguments and captured
    /// enclosing locals are not yet supported.
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    fn anonymous_class(
        &mut self,
        supertype: &str,
        args: Vec<Expr>,
        start: SourceSpan,
    ) -> Parsed<Expr> {
        if !args.is_empty() {
            self.error_at(
                start,
                "anonymous classes with constructor arguments are not yet supported by caturra",
            );
        }
        self.pos += 1; // '{'
        let mut methods = Vec::new();
        let mut fields = Vec::new();
        let mut init_blocks = Vec::new();
        let mut nested = Vec::new();
        let mut order = 0usize;
        while !self.at_symbol("}") && self.peek().is_some() {
            if let Ok(member) = self.member(supertype) {
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
        let mut anon = ClassDecl {
            name: name.clone(),
            // The supertype is resolved to extends/implements by the
            // compiler (it knows which names are interfaces).
            superclass: Some(String::from(supertype)),
            interfaces: Vec::new(),
            is_abstract: false,
            is_interface: false,
            is_enum: false,
            is_anonymous: true,
            type_params: Vec::new(),
            fields,
            methods,
            init_blocks,
            nested: Vec::new(),
            span,
        };
        anon.nested = nested;
        self.anon_classes.push(anon);
        Ok(Expr::NewObject {
            class: name,
            type_args: Vec::new(),
            args: Vec::new(),
            span,
        })
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
        if !self.at_symbol("}") {
            loop {
                if self.at_symbol("{") {
                    elements.push(self.array_literal()?);
                } else {
                    elements.push(self.expression()?);
                }
                if !self.eat_symbol(",") {
                    break;
                }
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
                    self.error_at(
                        method_span,
                        "super field access is not supported by caturra (call a method instead)",
                    );
                    return Err(Abort);
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
    interfaces: Vec<String>,
    constants: Vec<(String, Vec<Expr>, SourceSpan)>,
    mut fields: Vec<FieldDecl>,
    mut methods: Vec<MethodDecl>,
    init_blocks: Vec<InitBlock>,
    nested: Vec<ClassDecl>,
    span: SourceSpan,
) -> ClassDecl {
    let zero = SourceSpan {
        start: span.start,
        end: span.start,
    };
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

    // Shift user fields/blocks after the synthesized constant + array
    // initializers so those run first in <clinit>.
    let synth_order = constants.len() + 1;
    for field in &mut fields {
        field.order += synth_order;
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
    for (index, (const_name, args, const_span)) in constants.iter().enumerate() {
        let mut ctor_args = vec![
            lit_str(const_name),
            lit_int(i64::try_from(index).unwrap_or(i64::MAX)),
        ];
        ctor_args.extend(args.iter().cloned());
        synth_fields.push(FieldDecl {
            name: const_name.clone(),
            ty: enum_ty.clone(),
            is_static: true,
            is_private: false,
            is_final: true,
            init: Some(Expr::NewObject {
                class: name.clone(),
                type_args: Vec::new(),
                args: ctor_args,
                span: *const_span,
            }),
            order: index,
            span: *const_span,
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
            },
            Param {
                ty: TypeRef::Int,
                name: String::from("__ordinal"),
                is_varargs: false,
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
            is_constructor: true,
            is_abstract: false,
            type_params: Vec::new(),
            return_type: TypeRef::Void,
            params: lead_params(),
            body: store_stmts(),
            annotations: Vec::new(),
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

    // `static E[] values() { return new E[]{ A, B, ... }; }`
    if !defines(&methods, "values") {
        let elements: Vec<Expr> = constants.iter().map(|(n, _, _)| var(n)).collect();
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
            is_constructor: false,
            is_abstract: false,
            type_params: Vec::new(),
            return_type: TypeRef::Array(Box::new(enum_ty.clone())),
            params: Vec::new(),
            body: vec![Stmt::Return {
                value: Some(values_body),
                span: zero,
            }],
            annotations: Vec::new(),
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
                span: zero,
            },
            span: zero,
        };
        methods.push(MethodDecl {
            name: String::from("valueOf"),
            is_static: true,
            is_public: true,
            is_private: false,
            is_constructor: false,
            is_abstract: false,
            type_params: Vec::new(),
            return_type: enum_ty.clone(),
            params: vec![Param {
                ty: str_ty,
                name: String::from("__n"),
                is_varargs: false,
            }],
            body: vec![for_each, throw],
            annotations: Vec::new(),
            span: zero,
        });
    }

    ClassDecl {
        name,
        superclass: None,
        interfaces,
        is_abstract: false,
        is_interface: false,
        is_enum: true,
        is_anonymous: false,
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
fn flatten_nested(mut class: ClassDecl, out: &mut Vec<ClassDecl>) {
    let nested = std::mem::take(&mut class.nested);
    out.push(class);
    for inner in nested {
        flatten_nested(inner, out);
    }
}

/// Erase generic type parameters to `Object` within a class: every
/// `TypeRef` naming a class or method type parameter is rewritten to
/// `Named("Object")`. Runtime semantics are unchanged (erasure); this
/// keeps the rest of the compiler generics-unaware.
fn erase_type_vars(class: &mut ClassDecl) {
    use std::collections::HashSet;
    // A class with exactly one type parameter tracks it: that parameter
    // erases to the `TypeVar` sentinel (enabling cast-free reads);
    // every other type parameter (extra class params, method params)
    // erases straight to `Object`.
    let tracked: Option<String> = if class.type_params.len() == 1 {
        Some(class.type_params[0].clone())
    } else {
        None
    };
    let class_object: HashSet<String> = if tracked.is_some() {
        HashSet::new()
    } else {
        class.type_params.iter().cloned().collect()
    };
    let scope = |method: &MethodDecl| -> (HashSet<String>, Option<String>) {
        let mut to_object = class_object.clone();
        to_object.extend(method.type_params.iter().cloned());
        // A method type parameter shadowing the class one drops tracking.
        let tracked = tracked
            .clone()
            .filter(|name| !method.type_params.contains(name));
        (to_object, tracked)
    };
    for field in &mut class.fields {
        erase_in_type(&mut field.ty, &class_object, tracked.as_deref());
        if let Some(init) = &mut field.init {
            erase_in_expr(init, &class_object, tracked.as_deref());
        }
    }
    for method in &mut class.methods {
        let (to_object, tracked) = scope(method);
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
            erase_in_stmt(stmt, &class_object, tracked.as_deref());
        }
    }
}

/// The reserved type name that [`resolve_type`] maps to
/// [`JType::TypeVar`]; it cannot collide with a source identifier.
pub(crate) const TYPEVAR_SENTINEL: &str = "\u{0}TypeVar";

fn erase_in_type(
    ty: &mut TypeRef,
    to_object: &std::collections::HashSet<String>,
    tracked: Option<&str>,
) {
    match ty {
        TypeRef::Named(name) if Some(name.as_str()) == tracked => {
            *ty = TypeRef::Named(String::from(TYPEVAR_SENTINEL));
        }
        TypeRef::Named(name) if to_object.contains(name) => {
            *ty = TypeRef::Named(String::from("Object"));
        }
        TypeRef::Array(inner) => erase_in_type(inner, to_object, tracked),
        TypeRef::Generic { base, args } => {
            if Some(base.as_str()) == tracked {
                *ty = TypeRef::Named(String::from(TYPEVAR_SENTINEL));
            } else if to_object.contains(base) {
                *ty = TypeRef::Named(String::from("Object"));
            } else {
                for arg in args {
                    erase_in_type(arg, to_object, tracked);
                }
            }
        }
        _ => {}
    }
}

fn erase_in_stmt(
    stmt: &mut Stmt,
    to_object: &std::collections::HashSet<String>,
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
    to_object: &std::collections::HashSet<String>,
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
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } => {}
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
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        return_type,
        params: Vec::new(),
        body: vec![Stmt::Return {
            value: Some(value),
            span,
        }],
        annotations: Vec::new(),
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
        assert!(messages[0].contains("not yet supported"), "{messages:?}");
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
    fn assignment_in_condition_gets_a_hint() {
        let errors = parse_errors(r"class M { static void f() { int x = 1; if (x = 2) { } } }");
        assert!(
            errors[0].message.contains("did you mean '=='"),
            "{}",
            errors[0].message
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
