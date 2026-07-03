//! Recursive-descent parser for the supported Java subset.
//!
//! Diagnostics policy (see `specs/LANGUAGE.md`): constructs we plan to
//! support but haven't built yet produce a specific, friendly
//! "not yet supported" message with a source span — never a generic
//! syntax error — and the parser recovers so one file reports every
//! problem, not just the first.

use crate::ast::{
    BinaryOp, ClassDecl, CompilationUnit, Expr, Literal, LocalDeclarator, MethodDecl, Param, Stmt,
    TypeRef, UnaryOp,
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
    };
    let unit = parser.compilation_unit();
    (unit, parser.diagnostics)
}

/// The friendly message for statement-starting keywords jvmjs doesn't
/// support yet; `None` when the keyword can begin a real statement.
fn unsupported_statement_keyword(keyword: Keyword) -> Option<&'static str> {
    match keyword {
        Keyword::Else => Some("'else' without a matching 'if'"),
        Keyword::Return => Some("return statements are not yet supported by jvmjs"),
        Keyword::Switch => Some("switch statements are not yet supported by jvmjs"),
        Keyword::Try | Keyword::Throw => Some("exception handling is not yet supported by jvmjs"),
        Keyword::Var => Some("'var' is not supported by jvmjs; write the type explicitly"),
        Keyword::New => Some("object creation with 'new' is not yet supported by jvmjs"),
        Keyword::Int | Keyword::Double | Keyword::Boolean | Keyword::Char | Keyword::Final => None,
        _ => Some("this statement is not yet supported by jvmjs"),
    }
}

/// Lower `x++` / `x--` to `x += 1` / `x -= 1`.
fn increment_statement(
    name: String,
    increment: bool,
    start: SourcePosition,
    end: SourcePosition,
) -> Stmt {
    let span = SourceSpan { start, end };
    Stmt::Assign {
        name,
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

/// Internal marker: a construct failed to parse and a diagnostic was
/// already recorded; the caller should recover.
struct Abort;

type Parsed<T> = Result<T, Abort>;

struct Parser<'a> {
    path: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
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
        let mut classes = Vec::new();
        while let Some(kind) = self.peek() {
            match kind {
                TokenKind::Keyword(Keyword::Package | Keyword::Import) => {
                    let span = self.here();
                    let what = if self.at_keyword(Keyword::Package) {
                        "package declarations are"
                    } else {
                        "import is"
                    };
                    self.error_at(
                        span,
                        format!(
                            "{what} not yet supported by jvmjs; classes share one namespace for now"
                        ),
                    );
                    self.recover_to_statement_boundary();
                }
                _ => {
                    if let Ok(class) = self.class_decl() {
                        classes.push(class);
                    } else {
                        self.recover_to_statement_boundary();
                        // A stray `}` from a broken class body would stall
                        // the loop at top level; consume it and move on.
                        self.eat_symbol("}");
                    }
                }
            }
        }
        CompilationUnit { classes }
    }

    /// Modifier keywords before a class or member. Returns
    /// `(is_public, is_static)`; other modifiers parse and are ignored
    /// for now.
    fn modifiers(&mut self) -> (bool, bool) {
        let mut is_public = false;
        let mut is_static = false;
        loop {
            match self.peek() {
                Some(TokenKind::Keyword(Keyword::Public)) => {
                    is_public = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Static)) => {
                    is_static = true;
                    self.pos += 1;
                }
                Some(TokenKind::Keyword(
                    Keyword::Private | Keyword::Protected | Keyword::Final | Keyword::Abstract,
                )) => {
                    self.pos += 1;
                }
                _ => return (is_public, is_static),
            }
        }
    }

    fn class_decl(&mut self) -> Parsed<ClassDecl> {
        let start = self.here();
        self.modifiers();

        if self.at_keyword(Keyword::Interface) || self.at_keyword(Keyword::Enum) {
            let what = if self.at_keyword(Keyword::Interface) {
                "interfaces"
            } else {
                "enums"
            };
            self.error_here(format!("{what} are not yet supported by jvmjs"));
            return Err(Abort);
        }
        if !self.eat_keyword(Keyword::Class) {
            self.error_here("expected a class declaration");
            return Err(Abort);
        }
        let (name, name_span) = self.expect_ident("for the class")?;

        if self.at_keyword(Keyword::Extends) || self.at_keyword(Keyword::Implements) {
            let what = if self.at_keyword(Keyword::Extends) {
                "extends"
            } else {
                "implements"
            };
            self.error_here(format!("'{what}' is not yet supported by jvmjs"));
            // Skip the clause so the class body still parses.
            while !self.at_symbol("{") && self.peek().is_some() {
                self.pos += 1;
            }
        }

        self.expect_symbol("{", "to open the class body")?;
        let mut methods = Vec::new();
        while !self.at_symbol("}") {
            if self.peek().is_none() {
                self.error_at(
                    name_span,
                    format!("class '{name}' is missing its closing '}}'"),
                );
                break;
            }
            if let Ok(method) = self.member() {
                methods.push(method);
            } else {
                self.recover_to_statement_boundary();
                self.eat_symbol(";");
            }
        }
        self.eat_symbol("}");

        Ok(ClassDecl {
            name,
            methods,
            span: SourceSpan {
                start: start.start,
                end: name_span.end,
            },
        })
    }

    fn member(&mut self) -> Parsed<MethodDecl> {
        let start = self.here();
        let (is_public, is_static) = self.modifiers();
        let return_type = self.type_ref()?;
        let (name, name_span) = self.expect_ident("for the class member")?;

        if !self.at_symbol("(") {
            self.error_at(
                SourceSpan {
                    start: start.start,
                    end: name_span.end,
                },
                "field declarations are not yet supported by jvmjs",
            );
            return Err(Abort);
        }

        self.expect_symbol("(", "to open the parameter list")?;
        let mut params = Vec::new();
        if !self.at_symbol(")") {
            loop {
                let ty = self.type_ref()?;
                let (param_name, _) = self.expect_ident("for the parameter")?;
                params.push(Param {
                    ty,
                    name: param_name,
                });
                if !self.eat_symbol(",") {
                    break;
                }
            }
        }
        self.expect_symbol(")", "to close the parameter list")?;

        if self.at_symbol(";") {
            self.error_at(name_span, "abstract methods are not yet supported by jvmjs");
            return Err(Abort);
        }
        self.expect_symbol("{", "to open the method body")?;
        let body = self.block_body();

        Ok(MethodDecl {
            name,
            is_static,
            is_public,
            return_type,
            params,
            body,
            span: SourceSpan {
                start: start.start,
                end: name_span.end,
            },
        })
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
            Some(TokenKind::Keyword(
                Keyword::Long | Keyword::Float | Keyword::Byte | Keyword::Short,
            )) => {
                self.error_here(
                    "only int, double, boolean, and char primitives are supported by jvmjs \
                     (matching the CSA subset)",
                );
                return Err(Abort);
            }
            Some(TokenKind::Identifier(_)) => {
                let (name, _) = self.expect_ident("for the type")?;
                TypeRef::Named(name)
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
            _ => {}
        }

        if let Some(TokenKind::Keyword(keyword)) = self.peek()
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
        // Prefix `++x` / `--x`.
        if self.at_symbol("++") || self.at_symbol("--") {
            let start = self.here();
            let increment = self.at_symbol("++");
            self.pos += 1;
            let (name, name_span) = self.expect_ident("after the increment operator")?;
            return Ok(increment_statement(
                name,
                increment,
                start.start,
                name_span.end,
            ));
        }

        let expr = self.expression()?;

        if let Some(op) = self.assignment_operator() {
            let Expr::Name { path, span } = &expr else {
                self.error_at(
                    expr.span(),
                    "the left side of an assignment must be a variable",
                );
                return Err(Abort);
            };
            if path.len() != 1 {
                self.error_at(
                    *span,
                    "only simple variables can be assigned (fields come later)",
                );
                return Err(Abort);
            }
            let name = path[0].clone();
            let start = span.start;
            self.pos += 1;
            let value = self.expression()?;
            let end = value.span().end;
            return Ok(Stmt::Assign {
                name,
                op,
                value,
                span: SourceSpan { start, end },
            });
        }

        if self.at_symbol("++") || self.at_symbol("--") {
            let increment = self.at_symbol("++");
            self.pos += 1;
            let Expr::Name { path, span } = &expr else {
                self.error_at(expr.span(), "++/-- can only be applied to a variable");
                return Err(Abort);
            };
            if path.len() != 1 {
                self.error_at(*span, "++/-- can only be applied to a simple variable");
                return Err(Abort);
            }
            return Ok(increment_statement(
                path[0].clone(),
                increment,
                span.start,
                span.end,
            ));
        }

        if !matches!(expr, Expr::Call { .. }) {
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

        // `for (Type name : ...)` — for-each needs arrays/ArrayList.
        if self.header_contains_top_level_colon() {
            self.error_at(
                start,
                "for-each loops are not yet supported by jvmjs (they arrive with arrays)",
            );
            self.skip_balanced_parens();
            // Parse and discard the body so recovery is clean.
            let _ = self.statement();
            return Err(Abort);
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

    fn break_or_continue(&mut self) -> Parsed<Stmt> {
        let span = self.here();
        let is_break = self.at_keyword(Keyword::Break);
        self.pos += 1;
        if matches!(self.peek(), Some(TokenKind::Identifier(_))) {
            self.error_here("labeled break/continue is not supported by jvmjs");
            return Err(Abort);
        }
        self.expect_symbol(";", "to end the statement")?;
        Ok(if is_break {
            Stmt::Break { span }
        } else {
            Stmt::Continue { span }
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

    /// Skip forward past the closing `)` matching an already-consumed
    /// `(`.
    fn skip_balanced_parens(&mut self) {
        let mut depth = 1usize;
        while let Some(kind) = self.peek() {
            match kind {
                TokenKind::Symbol("(") => depth += 1,
                TokenKind::Symbol(")") => {
                    depth -= 1;
                    if depth == 0 {
                        self.pos += 1;
                        return;
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
    }

    /// Whether the cursor starts a local declaration:
    /// `final? <type> name ...` — a primitive-type keyword, `final`, or
    /// a class type followed by a name (or `[]`).
    fn at_declaration_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(TokenKind::Keyword(
                Keyword::Int | Keyword::Double | Keyword::Boolean | Keyword::Char | Keyword::Final
            ))
        ) || (matches!(self.peek(), Some(TokenKind::Identifier(_)))
            && matches!(self.peek_at(1), Some(TokenKind::Identifier(_))))
            || (matches!(self.peek(), Some(TokenKind::Identifier(_)))
                && matches!(self.peek_at(1), Some(TokenKind::Symbol("[")))
                && matches!(self.peek_at(2), Some(TokenKind::Symbol("]"))))
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
            _ => None,
        }
    }

    fn local_declaration(&mut self) -> Parsed<Stmt> {
        let start = self.here();
        let is_final = self.eat_keyword(Keyword::Final);
        let ty = self.type_ref()?;
        if matches!(ty, TypeRef::Array(_)) {
            self.error_at(start, "arrays are not yet supported by jvmjs");
            return Err(Abort);
        }

        let mut declarators = Vec::new();
        loop {
            let (name, name_span) = self.expect_ident("for the variable")?;
            let init = if self.eat_symbol("=") {
                Some(self.expression()?)
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
        self.logical_or()
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
        self.binary_level(Self::equality, &[("&&", BinaryOp::And)])
    }

    fn equality(&mut self) -> Parsed<Expr> {
        self.binary_level(
            Self::relational,
            &[("==", BinaryOp::Eq), ("!=", BinaryOp::Ne)],
        )
    }

    fn relational(&mut self) -> Parsed<Expr> {
        self.binary_level(
            Self::additive,
            &[
                ("<=", BinaryOp::Le),
                (">=", BinaryOp::Ge),
                ("<", BinaryOp::Lt),
                (">", BinaryOp::Gt),
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

    fn unary(&mut self) -> Parsed<Expr> {
        let start = self.here();
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
            self.error_here("++/-- inside an expression is not yet supported by jvmjs");
            return Err(Abort);
        }
        // Primitive cast: `(int) x` — unambiguous because a primitive
        // keyword can't start a parenthesized expression.
        if self.at_symbol("(")
            && matches!(
                self.peek_at(1),
                Some(TokenKind::Keyword(
                    Keyword::Int | Keyword::Double | Keyword::Boolean | Keyword::Char
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
        self.postfix_expression()
    }

    fn postfix_expression(&mut self) -> Parsed<Expr> {
        let mut expr = self.primary_expression()?;

        loop {
            if self.eat_symbol(".") {
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
                    self.error_at(
                        segment_span,
                        "field access on an expression is not yet supported by jvmjs",
                    );
                    return Err(Abort);
                }
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
                    self.error_here("this call expression is not yet supported by jvmjs");
                    return Err(Abort);
                }
            } else {
                return Ok(expr);
            }
        }
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

    fn primary_expression(&mut self) -> Parsed<Expr> {
        let span = self.here();
        match self.peek() {
            Some(TokenKind::IntLiteral(v)) => {
                let value = Literal::Int(*v);
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
            Some(TokenKind::Keyword(Keyword::New)) => {
                self.error_here("object creation with 'new' is not yet supported by jvmjs");
                Err(Abort)
            }
            Some(TokenKind::Keyword(Keyword::This | Keyword::Super)) => {
                self.error_here("'this' and 'super' are not yet supported by jvmjs");
                Err(Abort)
            }
            _ => {
                self.error_here("expected an expression");
                Err(Abort)
            }
        }
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
                    switch (1) { }
                    System.out.println("still parsed");
                }
            }
            "#,
        );
        let messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
        assert!(messages[0].contains("switch statements are not yet supported"));
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
    fn for_each_gets_a_friendly_message_and_recovers() {
        let errors = parse_errors(
            r#"
            class M {
                static void f() {
                    for (String s : names) { System.out.println(s); }
                    System.out.println("after");
                }
            }
            "#,
        );
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].message.contains("for-each"));
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
    fn labeled_break_is_rejected_kindly() {
        let errors = parse_errors(r"class M { static void f() { while (true) { break outer; } } }");
        assert!(errors[0].message.contains("labeled break"));
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
        let Stmt::Assign { name, op: None, .. } = &body[1] else {
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
    fn recovery_continues_after_bad_member() {
        let errors = parse_errors(
            r#"
            class Main {
                int field = 3;
                static void ok() { System.out.println("fine"); }
            }
            "#,
        );
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0]
                .message
                .contains("field declarations are not yet supported")
        );
    }

    #[test]
    fn import_and_package_are_skipped_with_messages() {
        let errors = parse_errors("package com.example;\nimport java.util.Scanner;\nclass A { }");
        assert_eq!(errors.len(), 2);
        assert!(errors[0].message.contains("package"));
        assert!(errors[1].message.contains("import"));
    }
}
