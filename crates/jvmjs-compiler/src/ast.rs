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
    /// `int a = 1, b;` — one declared type, one or more declarators.
    LocalDecl {
        ty: TypeRef,
        is_final: bool,
        declarators: Vec<LocalDeclarator>,
        span: SourceSpan,
    },
    /// `x = e;`, `x += e;` (op = `Some(Add)`), and `x++;` (lowered to
    /// `x += 1`). Plain assignment has `op = None`.
    Assign {
        name: String,
        op: Option<BinaryOp>,
        value: Expr,
        span: SourceSpan,
    },
    If {
        cond: Expr,
        then: Box<Stmt>,
        /// `else` branch; `else if` chains nest here naturally.
        els: Option<Box<Stmt>>,
        span: SourceSpan,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
        span: SourceSpan,
    },
    DoWhile {
        body: Box<Stmt>,
        cond: Expr,
        span: SourceSpan,
    },
    For {
        /// Declaration or simple statement; scoped to the loop.
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        /// Comma-separated statement expressions.
        update: Vec<Stmt>,
        body: Box<Stmt>,
        span: SourceSpan,
    },
    Break {
        span: SourceSpan,
    },
    Continue {
        span: SourceSpan,
    },
}

/// One `name = init` (or bare `name`) in a local declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalDeclarator {
    pub name: String,
    pub init: Option<Expr>,
    pub span: SourceSpan,
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    /// `&&` (short-circuit)
    And,
    /// `||` (short-circuit)
    Or,
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// `-x`
    Neg,
    /// `!x`
    Not,
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
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: SourceSpan,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
        span: SourceSpan,
    },
    /// A primitive cast: `(int) x`.
    Cast {
        ty: TypeRef,
        operand: Box<Expr>,
        span: SourceSpan,
    },
}

impl Expr {
    #[must_use]
    pub fn span(&self) -> SourceSpan {
        match self {
            Expr::Literal { span, .. }
            | Expr::Name { span, .. }
            | Expr::Call { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Cast { span, .. } => *span,
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
