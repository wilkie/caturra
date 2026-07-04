//! Abstract syntax tree for the supported Java subset.
//!
//! Nodes carry the [`SourceSpan`] of the construct so every later phase
//! can report located diagnostics without re-scanning source.

use crate::diagnostics::SourceSpan;

/// One parsed source file.
#[derive(Debug, Clone, PartialEq)]
pub struct CompilationUnit {
    pub imports: Vec<ImportDecl>,
    pub classes: Vec<ClassDecl>,
}

/// An `import a.b.C;` or `import a.b.*;` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    /// Dotted segments, e.g. `["java", "util", "Scanner"]` (the last
    /// segment is `*` for wildcard imports... represented separately).
    pub path: Vec<String>,
    pub wildcard: bool,
    pub span: SourceSpan,
}

/// A class or interface declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    /// `extends` clause (classes only; single inheritance).
    pub superclass: Option<String>,
    /// `implements` clause (or `extends` list for interfaces).
    pub interfaces: Vec<String>,
    pub is_abstract: bool,
    pub is_interface: bool,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<MethodDecl>,
    pub span: SourceSpan,
}

/// A field declaration (one declarator; `int x, y;` produces two).
#[allow(clippy::struct_excessive_bools)] // mirrors Java modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub ty: TypeRef,
    pub is_static: bool,
    pub is_private: bool,
    pub is_final: bool,
    pub init: Option<Expr>,
    pub span: SourceSpan,
}

/// A method or constructor declaration. Constructors have
/// `is_constructor` set, `name` equal to the class name, and a `Void`
/// return type.
#[allow(clippy::struct_excessive_bools)] // mirrors Java modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub is_static: bool,
    pub is_public: bool,
    pub is_private: bool,
    pub is_constructor: bool,
    /// Abstract or interface method — no body; `body` is empty.
    pub is_abstract: bool,
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
    Long,
    Float,
    Short,
    Byte,
    /// A class type by simple name, e.g. `String`.
    Named(String),
    /// A generic type, e.g. `ArrayList<Integer>`.
    Generic {
        base: String,
        args: Vec<TypeRef>,
    },
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
    /// `x = e;`, `a[i] = e;`, `x += e;` (op = `Some(Add)`), and `x++;`
    /// (lowered to `x += 1`). Plain assignment has `op = None`.
    Assign {
        target: AssignTarget,
        op: Option<BinaryOp>,
        value: Expr,
        span: SourceSpan,
    },
    /// `for (Type name : iterable) body`.
    ForEach {
        ty: TypeRef,
        name: String,
        iterable: Expr,
        body: Box<Stmt>,
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
        /// The target label for `break label;`, if any.
        label: Option<String>,
        span: SourceSpan,
    },
    Continue {
        /// The target loop label for `continue label;`, if any.
        label: Option<String>,
        span: SourceSpan,
    },
    /// `label: statement` — a labeled statement (the label is
    /// meaningful for `break`/`continue` targeting an enclosing loop or
    /// block).
    Labeled {
        label: String,
        body: Box<Stmt>,
        span: SourceSpan,
    },
    Return {
        value: Option<Expr>,
        span: SourceSpan,
    },
    /// `super(args);` — must be the first statement of a constructor.
    SuperCall {
        args: Vec<Expr>,
        span: SourceSpan,
    },
    /// `this(args);` — constructor delegation, first statement only.
    ThisCall {
        args: Vec<Expr>,
        span: SourceSpan,
    },
    /// `try { ... } catch (Type name) { ... } ... finally { ... }`.
    Try {
        body: Vec<Stmt>,
        catches: Vec<CatchClause>,
        finally_body: Option<Vec<Stmt>>,
        span: SourceSpan,
    },
    /// `throw expr;`.
    Throw {
        value: Expr,
        span: SourceSpan,
    },
    /// `switch (selector) { case ...: ... default: ... }`.
    Switch {
        selector: Expr,
        arms: Vec<SwitchArm>,
        span: SourceSpan,
    },
}

/// One `case`/`default` group and the statements under it (which fall
/// through to the next group unless they break).
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchArm {
    /// The labels stacked on this arm; `None` is `default:`.
    pub labels: Vec<Option<Expr>>,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// One `catch (Type name) { ... }` clause.
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub ty: TypeRef,
    pub name: String,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}

/// One `name = init` (or bare `name`) in a local declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalDeclarator {
    pub name: String,
    pub init: Option<Expr>,
    pub span: SourceSpan,
}

/// The left side of an assignment.
#[derive(Debug, Clone, PartialEq)]
pub enum AssignTarget {
    /// `x = ...` — a local, an implicit `this` field, or a static
    /// field of the current class (resolved during codegen).
    Var(String),
    /// `a[i] = ...` (nested for `m[i][j]`: `array` is itself an index).
    Index { array: Box<Expr>, index: Box<Expr> },
    /// `p.x = ...`, `this.x = ...`, `ClassName.staticField = ...`.
    Field { object: Box<Expr>, name: String },
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
    /// `&` — bitwise on ints, non-short-circuit logical on booleans.
    BitAnd,
    /// `|`
    BitOr,
    /// `^`
    BitXor,
    /// `<<`
    Shl,
    /// `>>` (arithmetic)
    Shr,
    /// `>>>` (logical)
    Ushr,
}

/// A unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// `-x`
    Neg,
    /// `!x`
    Not,
    /// `~x`
    BitNot,
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
    /// Array element access: `a[i]`.
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
        span: SourceSpan,
    },
    /// Field access on a non-name expression: `m[i].length`. (Dotted
    /// identifier chains stay [`Expr::Name`].)
    Field {
        object: Box<Expr>,
        name: String,
        span: SourceSpan,
    },
    /// `new int[3]`, `new int[2][3]`, `new int[3][]`, or (with `init`)
    /// `new int[] {1, 2}`.
    NewArray {
        /// Element base type (`int` in `new int[2][3]`).
        elem: TypeRef,
        /// One entry per `[...]`: `Some(len)` or `None` for empty.
        dims: Vec<Option<Expr>>,
        init: Option<Vec<Expr>>,
        span: SourceSpan,
    },
    /// `{1, 2, 3}` — only valid as a declaration initializer or nested
    /// inside another array literal / `new T[] {...}`.
    ArrayLiteral {
        elements: Vec<Expr>,
        span: SourceSpan,
    },
    /// The `this` keyword (instance contexts only).
    This {
        span: SourceSpan,
    },
    /// `new ClassName(args)` / `new ArrayList<Integer>()`.
    NewObject {
        class: String,
        /// Generic type arguments (empty for the diamond `<>` or none).
        type_args: Vec<TypeRef>,
        args: Vec<Expr>,
        span: SourceSpan,
    },
    /// `expr instanceof Type`.
    InstanceOf {
        value: Box<Expr>,
        ty: TypeRef,
        span: SourceSpan,
    },
    /// `super.method(args)` — non-virtual call to the superclass.
    SuperMethodCall {
        method: String,
        args: Vec<Expr>,
        span: SourceSpan,
    },
    /// `cond ? then : else`.
    Ternary {
        cond: Box<Expr>,
        then: Box<Expr>,
        els: Box<Expr>,
        span: SourceSpan,
    },
    /// `x++` / `--a[i]` in expression position (statement-only forms
    /// still lower to compound assignments).
    IncDec {
        target: Box<Expr>,
        /// `++` or `--`.
        increment: bool,
        /// Prefix yields the new value, postfix the old.
        prefix: bool,
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
            | Expr::Cast { span, .. }
            | Expr::Index { span, .. }
            | Expr::Field { span, .. }
            | Expr::NewArray { span, .. }
            | Expr::ArrayLiteral { span, .. }
            | Expr::This { span }
            | Expr::NewObject { span, .. }
            | Expr::InstanceOf { span, .. }
            | Expr::SuperMethodCall { span, .. }
            | Expr::Ternary { span, .. }
            | Expr::IncDec { span, .. } => *span,
        }
    }
}

/// A literal value.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Long(i64),
    Float(f32),
    Double(f64),
    Str(String),
    Char(char),
    Bool(bool),
    Null,
}
