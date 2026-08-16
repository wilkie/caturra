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
    /// `import static X.Y.*` / `import static X.Y.member` — brings static
    /// members into scope unqualified (used for `JUnit` `Assertions`).
    pub is_static: bool,
    pub span: SourceSpan,
}

/// A class or interface declaration.
#[allow(clippy::struct_excessive_bools)] // mirrors Java's declaration modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    /// Declared `public`. A public top-level type must live in a file of
    /// its own name (JLS §7.6), which javac enforces and so do we.
    pub is_public: bool,
    /// Declared inside another type, then hoisted to the top level. The
    /// file-name rule applies to top-level types only, so a `public static
    /// class Inner` is exempt.
    pub is_nested: bool,
    /// For a synthesized anonymous/lambda class: the class whose method
    /// created it. Hoisting to the top level loses sight of that class's
    /// static fields, so name resolution falls back to them.
    pub enclosing: Option<String>,
    /// The JVM BINARY name of a hoisted nested class — `Outer$Inner`, and
    /// `A$B$C` for a deeper one. `name` stays the SIMPLE name, because that is
    /// how the source refers to it and how every pass here matches it; this is
    /// what the class file is called and therefore what `getClass().getName()`,
    /// a default `toString()` and a stack-trace frame report. `None` for a
    /// top-level class, whose binary name IS its name.
    pub binary_name: Option<String>,
    /// `extends` clause (classes only; single inheritance).
    pub superclass: Option<String>,
    /// `implements` clause (or `extends` list for interfaces).
    pub interfaces: Vec<String>,
    /// The TYPE ARGUMENTS written on each supertype: `extends Box<String>`
    /// records `("Box", [String])`. Erasure drops them everywhere else, but
    /// assigning a subclass to a parameterized supertype (`Box<String> b = new
    /// SBox()`) can only be CHECKED against what was written — without these
    /// the widening would have to be allowed blindly, accepting the mismatched
    /// `Box<String> b = new IntBox()` that javac refuses.
    pub supertype_args: Vec<(String, Vec<TypeRef>)>,
    pub is_abstract: bool,
    /// Declared `final` — a class that may not be extended (JLS §8.1.1.2).
    pub is_final: bool,
    pub is_interface: bool,
    /// Set for `enum` declarations (after desugaring to a class): the
    /// class has synthesized constant fields and `values`/`valueOf`;
    /// switch case labels are unqualified constant names.
    pub is_enum: bool,
    /// Set for a synthesized anonymous-class body. `superclass` holds
    /// the named supertype, which the compiler resolves to an
    /// `extends` (class) or `implements` (interface).
    pub is_anonymous: bool,
    /// Set for a local class (declared inside a method body) after it is
    /// mangled and hoisted to the top level. Like an anonymous class it can
    /// capture enclosing locals, so it joins the capture pass — but it keeps
    /// ordinary `extends`/`implements` resolution rather than the anonymous
    /// single-supertype form.
    pub is_local: bool,
    /// Set for a non-static nested class (an inner class), bound to an enclosing
    /// instance. A pass gives it a synthetic enclosing reference and threads
    /// that instance through its constructors and its `new` sites.
    pub is_inner: bool,
    /// Generic type parameter names (`<T, U>`); erased to `Object`.
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<MethodDecl>,
    /// `static { ... }` and instance `{ ... }` initializer blocks.
    pub init_blocks: Vec<InitBlock>,
    /// Nested type declarations, hoisted to top level after parsing.
    pub nested: Vec<ClassDecl>,
    pub span: SourceSpan,
}

/// A class-body initializer block (`static { ... }` or `{ ... }`).
#[derive(Debug, Clone, PartialEq)]
pub struct InitBlock {
    pub is_static: bool,
    pub body: Vec<Stmt>,
    /// Textual position among the class's fields and blocks, so
    /// initialization runs in source order (JLS §12.4.2 / §12.5).
    pub order: usize,
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
    /// Textual position among the class's fields and init blocks.
    pub order: usize,
    pub span: SourceSpan,
}

/// A method or constructor declaration. Constructors have
/// Where a generic method's parameters mention the type variable its return
/// names, for [`MethodDecl::infer_return`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferSource {
    /// The parameter IS the variable: `<T> T max(T a, T b)`.
    Direct(usize),
    /// The parameter is a container OF it: `<T> T max(List<T> xs)`. The
    /// argument's ELEMENT type is what pins `T`, which is the commoner shape
    /// of the two — every "biggest of a list" method is written this way.
    Element(usize),
}

/// `is_constructor` set, `name` equal to the class name, and a `Void`
/// return type.
#[allow(clippy::struct_excessive_bools)] // mirrors Java modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct MethodDecl {
    pub name: String,
    pub is_static: bool,
    pub is_public: bool,
    pub is_private: bool,
    /// Declared `protected` — an override may not weaken access below the
    /// overridden method's level (JLS §8.4.8.3).
    pub is_protected: bool,
    /// Declared `final` — an override of one is a compile-time error
    /// (JLS §8.4.3.3).
    pub is_final: bool,
    pub is_constructor: bool,
    /// Abstract or interface method — no body; `body` is empty.
    pub is_abstract: bool,
    /// Generic method type parameters (`<T> T identity(T x)`).
    pub type_params: Vec<TypeParam>,
    /// Return-type inference plan, filled in by `erase_type_vars`: `Some`
    /// when the declared return type is a bare type variable that the
    /// parameters also mention, holding where each mention is. The call's
    /// actual return type is the join of what those arguments pin —
    /// recovering the type argument erasure would otherwise drop. `None` when
    /// the return is not an inferable type variable.
    pub infer_return: Option<Vec<InferSource>>,
    /// The parameter types AS WRITTEN, before type variables erase. A lambda
    /// argument's target type is its declared parameter, and for a generic
    /// method that parameter mentions a type VARIABLE — `<T> int pick(T v,
    /// Box<T> f)` — which erasure replaces with a wildcard that says nothing
    /// about which variable it was. Kept so the call site can put the variable
    /// back; empty when the method declares no type parameters, since nothing
    /// is lost then.
    pub declared_params: Vec<TypeRef>,
    /// How to pin each of the method's own type variables from the ARGUMENTS
    /// at a call — the same plan `infer_return` holds for the return type,
    /// computed for every variable rather than just the returned one.
    pub type_var_sources: Vec<(String, Vec<InferSource>)>,
    pub return_type: TypeRef,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    /// Retained annotations (`@Test`, `@Order(1)`, `@BeforeEach`, …) with
    /// an optional integer argument — enough for the `JUnit` test runner.
    pub annotations: Vec<Annotation>,
    /// The `throws` clause's exception names, as written (`IOException`,
    /// `java.io.IOException`). Recorded for JLS §11.2 checked-exception
    /// enforcement; empty when absent.
    pub throws: Vec<String>,
    /// How many LEADING body statements a desugaring inserted to stand in for
    /// what a superclass constructor does, and so must run BEFORE the class's
    /// instance field initializers rather than after them (JLS §12.5 step 4).
    /// Two things need this: an enum constructor's `__name`/`__ordinal` stores
    /// (a real `java.lang.Enum` sets those in the super constructor, so a field
    /// initializer calling `name()` sees the name), and an anonymous or local
    /// class's captured-local stores (javac's `val$x = x`, which likewise
    /// precede the initializers so `int w = captured;` can read one).
    ///
    /// A USER constructor's own `this.x = x;` must NOT be counted: Java runs it
    /// after the initializers, so `int y = x;` in the same class reads 0.
    pub pre_init: usize,
    pub span: SourceSpan,
}

/// A retained annotation on a method (name plus an optional int arg).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub name: String,
    pub int_arg: Option<i32>,
    pub str_arg: Option<String>,
}

/// A declared type parameter: `T`, `T extends Bound`, or an INTERSECTION bound
/// `T extends A & B`. `bound` is the leftmost (what the JVM erasure is);
/// `extra_bounds` holds the rest, which still have to be visible — `t.b()` on a
/// `<T extends A & B>` is legal Java, and erasing to `A` alone hid it.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub bound: Option<TypeRef>,
    pub extra_bounds: Vec<TypeRef>,
}

/// The method name the try-with-resources desugaring gives ONE of the two
/// `close()` calls it generates per resource. Codegen turns it back into
/// `close`, having first checked that the resource really is an
/// `AutoCloseable` — a check the statement's own shape can no longer support,
/// because desugaring happens in the parser, before any type is known. Marking
/// one call and not both keeps the diagnostic from being reported twice.
pub const RESOURCE_CLOSE: &str = "\u{0}close\u{0}";

/// Reserved prefix that turns a wildcard type argument (`? extends Number`)
/// into an ordinary [`TypeRef::Named`], the way the type-variable sentinel
/// does — no dedicated variant, so the many `TypeRef` matches stay untouched.
/// It cannot collide with a source identifier (leading `NUL`).
const WILDCARD_PREFIX: &str = "\u{0}Wildcard\u{0}";

/// Encode a wildcard type argument as a reserved type name. `variance` is
/// `'?'` (unbounded), `'+'` (`extends`), `'-'` (`super`), or `'='` — the
/// erasure of a TYPE VARIABLE argument (`List<T>` in a generic method), which
/// accepts any element like `?` but, unlike `?`, may still be written to;
/// `bound` is the bound's simple name (empty when unbounded).
#[must_use]
pub fn wildcard_type_name(variance: char, bound: &str) -> String {
    format!("{WILDCARD_PREFIX}{variance}{bound}")
}

/// Decode [`wildcard_type_name`]: `(variance, bound)`, or `None` when `name`
/// is not a wildcard sentinel.
#[must_use]
pub fn wildcard_parts(name: &str) -> Option<(char, &str)> {
    let rest = name.strip_prefix(WILDCARD_PREFIX)?;
    let variance = rest.chars().next()?;
    Some((variance, &rest[variance.len_utf8()..]))
}

/// A method parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub ty: TypeRef,
    pub name: String,
    /// `void m(final int v)` — a parameter that cannot be reassigned. Java allows
    /// it on every parameter, and javac rejects an assignment to one, so accepting
    /// the modifier without enforcing it would be worse than not taking it at all.
    pub is_final: bool,
    /// The trailing `Type... name` varargs parameter (`ty` is the
    /// array type). Only valid as the last parameter.
    pub is_varargs: bool,
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
    /// `var` (Java 10 local type inference): the real type is whatever the
    /// initializer produces, resolved during codegen — there is nothing to infer
    /// from at parse time.
    Var,
    /// A generic type, e.g. `ArrayList<Integer>`.
    Generic {
        base: String,
        args: Vec<TypeRef>,
    },
    Array(Box<TypeRef>),
}

/// Wrap `ty` in `dims` array levels (`array_of(Int, 2)` is `int[][]`).
#[must_use]
pub fn array_of(ty: TypeRef, dims: usize) -> TypeRef {
    let mut ty = ty;
    for _ in 0..dims {
        ty = TypeRef::Array(Box::new(ty));
    }
    ty
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
    /// The alternatives: one type, or several for a multi-catch
    /// (`catch (IOException | SQLException e)`).
    pub types: Vec<TypeRef>,
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
    /// Brackets written after this name, C-style: in `int a[], b;` the extra
    /// dimension belongs to `a` alone, so it cannot live on the shared type.
    pub extra_dims: usize,
}

/// A lambda parameter: a name, optionally with an explicit type.
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaParam {
    pub name: String,
    pub ty: Option<TypeRef>,
}

/// A lambda body: a single expression, or a statement block.
#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
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
    /// `+x` — numerically a no-op, but performs unary numeric promotion
    /// (JLS §15.15.3): `+aChar` is an int.
    Plus,
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
    /// The receiver of `super.field` — never a value on its own. Typed as
    /// the superclass, so a field resolves from there upward: with hiding,
    /// `super.n` is the field the superclass sees, not this class's.
    /// (`super.method(...)` is [`Expr::SuperMethodCall`]: methods dispatch,
    /// fields do not.)
    Super {
        span: SourceSpan,
    },
    /// `new ClassName(args)` / `new ArrayList<Integer>()`.
    NewObject {
        class: String,
        /// Generic type arguments (empty for the diamond `<>` or none).
        type_args: Vec<TypeRef>,
        args: Vec<Expr>,
        /// The enclosing instance for a qualified inner-class creation
        /// (`outer.new Inner()`); `None` for an ordinary `new`. A pass binds it
        /// as the inner class's synthetic enclosing reference.
        outer: Option<Box<Expr>>,
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
        /// The INTERFACE named in `Iface.super.m()` — the standard way to pick
        /// one of several inherited defaults. `None` for a plain `super.m()`,
        /// which looks up the superclass chain.
        owner: Option<String>,
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
    /// `Type::method`, `expr::method`, or `Type::new`. Target-typed
    /// against a functional interface and desugared to a lambda.
    MethodRef {
        /// The qualifier: a class name (`String`) or a value
        /// expression (`System.out`).
        qualifier: Box<Expr>,
        /// The referenced method, or `new` for a constructor reference.
        method: String,
        span: SourceSpan,
    },
    /// `x -> body` / `(a, b) -> { ... }`. Target-typed against a
    /// functional interface and desugared to an anonymous class.
    Lambda {
        params: Vec<LambdaParam>,
        body: LambdaBody,
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
    /// `x = e` / `a[i] += e` in expression position — assignment is an
    /// expression in Java (JLS §15.26) whose value is what was stored. A bare
    /// `x = e;` statement stays a [`Stmt::Assign`]; this is the nested use
    /// (`while ((s = in.readLine()) != null)`, `println(x = 7)`).
    Assign {
        target: AssignTarget,
        /// `None` for plain `=`, `Some(op)` for a compound form (`+=`).
        op: Option<BinaryOp>,
        value: Box<Expr>,
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
            | Expr::Super { span }
            | Expr::Ternary { span, .. }
            | Expr::Lambda { span, .. }
            | Expr::MethodRef { span, .. }
            | Expr::IncDec { span, .. }
            | Expr::Assign { span, .. } => *span,
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
    /// A `char` literal's UTF-16 code UNIT — not a Rust `char`, which cannot
    /// hold the unpaired surrogate `'\uD83D'` legally denotes.
    Char(u16),
    Bool(bool),
    Null,
}
