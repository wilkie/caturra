//! Recursive-descent parser for the supported Java subset.
//!
//! Diagnostics policy (see `specs/LANGUAGE.md`): constructs we plan to
//! support but haven't built yet produce a specific, friendly
//! "not yet supported" message with a source span — never a generic
//! syntax error — and the parser recovers so one file reports every
//! problem, not just the first.

use crate::ast::{ClassDecl, CompilationUnit, Expr, Literal, MethodDecl, Param, Stmt, TypeRef};
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

        if let Some(TokenKind::Keyword(keyword)) = self.peek() {
            let message = match keyword {
                Keyword::If | Keyword::Else => Some("if statements are not yet supported by jvmjs"),
                Keyword::While | Keyword::Do => Some("while loops are not yet supported by jvmjs"),
                Keyword::For => Some("for loops are not yet supported by jvmjs"),
                Keyword::Return => Some("return statements are not yet supported by jvmjs"),
                Keyword::Switch => Some("switch statements are not yet supported by jvmjs"),
                Keyword::Try | Keyword::Throw => {
                    Some("exception handling is not yet supported by jvmjs")
                }
                Keyword::Break | Keyword::Continue => {
                    Some("break/continue are not yet supported by jvmjs")
                }
                Keyword::Int
                | Keyword::Double
                | Keyword::Boolean
                | Keyword::Char
                | Keyword::Var
                | Keyword::Final => {
                    Some("local variable declarations are not yet supported by jvmjs")
                }
                Keyword::New => Some("object creation with 'new' is not yet supported by jvmjs"),
                _ => Some("this statement is not yet supported by jvmjs"),
            };
            if let Some(message) = message {
                self.error_here(message);
                return Err(Abort);
            }
        }

        // `String s = ...` — a declaration starting with a class type.
        if matches!(self.peek(), Some(TokenKind::Identifier(_)))
            && matches!(self.peek_at(1), Some(TokenKind::Identifier(_)))
        {
            self.error_here("local variable declarations are not yet supported by jvmjs");
            return Err(Abort);
        }

        let expr = self.expression()?;

        if self.at_symbol("=")
            || matches!(self.peek(), Some(TokenKind::Symbol(s)) if s.len() >= 2 && s.ends_with('=') && !matches!(*s, "==" | "!=" | "<=" | ">="))
        {
            self.error_here("assignment is not yet supported by jvmjs");
            return Err(Abort);
        }

        self.expect_symbol(";", "to end the statement")?;
        Ok(Some(Stmt::Expr(expr)))
    }

    fn expression(&mut self) -> Parsed<Expr> {
        let expr = self.postfix_expression()?;

        if let Some(TokenKind::Symbol(op)) = self.peek()
            && matches!(
                *op,
                "+" | "-"
                    | "*"
                    | "/"
                    | "%"
                    | "=="
                    | "!="
                    | "<"
                    | ">"
                    | "<="
                    | ">="
                    | "&&"
                    | "||"
                    | "++"
                    | "--"
            )
        {
            self.error_here(format!("the '{op}' operator is not yet supported by jvmjs"));
            return Err(Abort);
        }

        Ok(expr)
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
                    int x = 5;
                    if (true) { }
                    System.out.println("still parsed");
                }
            }
            "#,
        );
        let messages: Vec<&str> = errors.iter().map(|e| e.message.as_str()).collect();
        assert!(messages[0].contains("local variable declarations are not yet supported"));
        assert!(messages[1].contains("if statements are not yet supported"));
        // Recovery: only the two unsupported statements are reported.
        assert_eq!(errors.len(), 2, "{messages:?}");
    }

    #[test]
    fn operators_are_reported_not_yet_supported() {
        let errors =
            parse_errors(r"class Main { static void run() { System.out.println(1 + 2); } }");
        assert!(
            errors[0]
                .message
                .contains("'+' operator is not yet supported")
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
