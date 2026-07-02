//! Abstract syntax tree for the supported Java subset.
//!
//! Nodes carry the [`SourceSpan`] of the construct so every later phase
//! can report located diagnostics without re-scanning source.

use crate::diagnostics::SourceSpan;

/// One parsed source file.
#[derive(Debug, Clone, PartialEq)]
pub struct CompilationUnit {
    pub classes: Vec<ClassDecl>,
}

/// A class declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub methods: Vec<MethodDecl>,
    pub span: SourceSpan,
}

/// A method declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub is_static: bool,
    pub is_public: bool,
    pub return_type: TypeRef,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// A method parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub ty: TypeRef,
    pub name: String,
}

/// A type reference as written in source.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Void,
    Int,
    Double,
    Boolean,
    Char,
    /// A class type by simple name, e.g. `String`.
    Named(String),
    Array(Box<TypeRef>),
}

/// A statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: Literal,
        span: SourceSpan,
    },
    /// A dotted name that is not (yet) resolved: `args`, `System.out`.
    Name {
        path: Vec<String>,
        span: SourceSpan,
    },
    /// A method call: `receiver.method(args)` or a bare `method(args)`.
    Call {
        receiver: Option<Box<Expr>>,
        method: String,
        args: Vec<Expr>,
        span: SourceSpan,
    },
}

impl Expr {
    #[must_use]
    pub fn span(&self) -> SourceSpan {
        match self {
            Expr::Literal { span, .. } | Expr::Name { span, .. } | Expr::Call { span, .. } => *span,
        }
    }
}

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Double(f64),
    Str(String),
    Char(char),
    Bool(bool),
    Null,
}
