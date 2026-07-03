//! Bytecode generation: AST → [`ClassFile`].
//!
//! Scope matches `specs/LANGUAGE.md` stage 1: local variables,
//! assignment (plain, compound, `++`/`--`), arithmetic / comparison /
//! logical operators with Java's promotion and short-circuit rules,
//! primitive casts, string concatenation via `StringBuilder` chains,
//! and `System.out`/`System.err` `print`/`println` of any supported
//! expression. Unsupported constructs produce friendly diagnostics and
//! the classes are not emitted.
//!
//! Typing is done during emission ("type-directed emission"): each
//! expression emitter returns the [`JType`] it left on the operand
//! stack. `JType::Error` marks a subtree that already produced a
//! diagnostic, and silences follow-on errors.

use jvmjs_classfile::opcodes as op;
use jvmjs_classfile::{
    AttributeInfo, CODE_ATTRIBUTE, ClassFile, CodeAttribute, Constant, ConstantPool, CpIndex,
    MethodAccessFlags, MethodInfo, write_code_attribute,
};

use crate::CompiledClass;
use crate::ast::{
    AssignTarget, BinaryOp, ClassDecl, CompilationUnit, Expr, Literal, LocalDeclarator, MethodDecl,
    Stmt, TypeRef, UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourceSpan};

/// Generate class files for every class across all parsed units.
/// Method calls resolve against every class in the compilation, so the
/// signature table is built first. Diagnostics and classes are both
/// returned; callers treat any error diagnostic as failing the
/// compilation.
#[must_use]
pub fn generate(units: &[(String, CompilationUnit)]) -> (Vec<CompiledClass>, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let table = MethodTable::build(units, &mut diagnostics);

    let mut classes = Vec::new();
    for (path, unit) in units {
        for class in &unit.classes {
            classes.push(CompiledClass {
                binary_name: class.name.clone(),
                class_file: emit_class(path, &mut diagnostics, &table, class),
            });
        }
    }
    (classes, diagnostics)
}

fn emit_class(
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    table: &MethodTable,
    decl: &ClassDecl,
) -> ClassFile {
    let mut class = ClassFile::new_java11();
    class.this_class = intern_class(&mut class.constant_pool, &decl.name);
    class.super_class = intern_class(&mut class.constant_pool, "java/lang/Object");

    for method in &decl.methods {
        let compiled = emit_method(
            path,
            diagnostics,
            table,
            &decl.name,
            &mut class.constant_pool,
            method,
        );
        class.methods.push(compiled);
    }
    class
}

/// One method signature, as seen by call resolution.
#[derive(Debug, Clone)]
struct MethodSig {
    name: String,
    params: Vec<JType>,
    /// `None` is `void`.
    ret: Option<JType>,
    is_static: bool,
}

impl MethodSig {
    fn describe(&self) -> String {
        let params: Vec<String> = self.params.iter().map(|p| p.describe()).collect();
        format!("{}({})", self.name, params.join(","))
    }

    fn descriptor(&self) -> String {
        let mut out = String::from("(");
        for param in &self.params {
            out.push_str(&param.descriptor());
        }
        out.push(')');
        match self.ret {
            None => out.push('V'),
            Some(ty) => out.push_str(&ty.descriptor()),
        }
        out
    }
}

/// All method signatures in the compilation, keyed by class name.
struct MethodTable {
    classes: std::collections::HashMap<String, Vec<MethodSig>>,
}

/// Outcome of static overload resolution (JLS §15.12.2, without boxing
/// or varargs).
enum Resolution<'t> {
    Found(&'t MethodSig),
    /// No method of that name exists in the class.
    UnknownName,
    /// Methods of that name exist, but none accept these arguments.
    NoneApplicable,
    /// Several maximally specific methods match.
    Ambiguous(Vec<String>),
}

impl MethodTable {
    fn build(units: &[(String, CompilationUnit)], diagnostics: &mut Vec<Diagnostic>) -> Self {
        let mut classes: std::collections::HashMap<String, Vec<MethodSig>> =
            std::collections::HashMap::new();
        for (path, unit) in units {
            for class in &unit.classes {
                let methods = classes.entry(class.name.clone()).or_default();
                for method in &class.methods {
                    let sig = MethodSig {
                        name: method.name.clone(),
                        params: method
                            .params
                            .iter()
                            .map(|p| type_from_ref(&p.ty).unwrap_or(JType::Unsupported))
                            .collect(),
                        ret: match &method.return_type {
                            TypeRef::Void => None,
                            other => Some(type_from_ref(other).unwrap_or(JType::Unsupported)),
                        },
                        is_static: method.is_static,
                    };
                    if methods
                        .iter()
                        .any(|m| m.name == sig.name && m.params == sig.params)
                    {
                        diagnostics.push(Diagnostic::error(
                            path,
                            format!(
                                "method {} is already defined in class {}",
                                sig.describe(),
                                class.name
                            ),
                            method.span,
                        ));
                    } else {
                        methods.push(sig);
                    }
                }
            }
        }
        Self { classes }
    }

    fn has_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    /// Resolve `class.name(args)`: applicable-by-widening, exact match
    /// first, then the unique most-specific method.
    fn resolve(&self, class: &str, name: &str, args: &[JType]) -> Resolution<'_> {
        let Some(methods) = self.classes.get(class) else {
            return Resolution::UnknownName;
        };
        let named: Vec<&MethodSig> = methods.iter().filter(|m| m.name == name).collect();
        if named.is_empty() {
            return Resolution::UnknownName;
        }
        let applicable: Vec<&MethodSig> = named
            .iter()
            .copied()
            .filter(|m| {
                m.params.len() == args.len()
                    && m.params.iter().zip(args).all(|(p, a)| widens(*a, *p))
            })
            .collect();
        match applicable.len() {
            0 => Resolution::NoneApplicable,
            1 => Resolution::Found(applicable[0]),
            _ => {
                if let Some(exact) = applicable
                    .iter()
                    .find(|m| m.params.iter().zip(args).all(|(p, a)| p == a))
                {
                    return Resolution::Found(exact);
                }
                let most_specific: Vec<&MethodSig> = applicable
                    .iter()
                    .copied()
                    .filter(|m| {
                        applicable.iter().all(|other| {
                            m.params
                                .iter()
                                .zip(&other.params)
                                .all(|(a, b)| widens(*a, *b))
                        })
                    })
                    .collect();
                if most_specific.len() == 1 {
                    Resolution::Found(most_specific[0])
                } else {
                    Resolution::Ambiguous(applicable.iter().map(|m| m.describe()).collect())
                }
            }
        }
    }
}

/// The [`ElemType`] for a base (non-array) type, if it can be an array
/// element.
fn elem_type_of(ty: JType) -> Option<ElemType> {
    match ty {
        JType::Int => Some(ElemType::Int),
        JType::Double => Some(ElemType::Double),
        JType::Boolean => Some(ElemType::Boolean),
        JType::Char => Some(ElemType::Char),
        JType::Str => Some(ElemType::Str),
        _ => None,
    }
}

/// Method-invocation / assignment widening (JLS §5.3 without boxing).
fn widens(from: JType, to: JType) -> bool {
    from == to
        || matches!(
            (from, to),
            (JType::Char, JType::Int)
                | (JType::Int | JType::Char, JType::Double)
                | (JType::Null, JType::Str | JType::Array { .. })
        )
}

/// The element base type of an array (arrays of arrays are expressed
/// via [`JType::Array`]'s `dims`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElemType {
    Int,
    Double,
    Boolean,
    Char,
    Str,
}

impl ElemType {
    fn descriptor(self) -> &'static str {
        match self {
            ElemType::Int => "I",
            ElemType::Double => "D",
            ElemType::Boolean => "Z",
            ElemType::Char => "C",
            ElemType::Str => "Ljava/lang/String;",
        }
    }

    fn base_type(self) -> JType {
        match self {
            ElemType::Int => JType::Int,
            ElemType::Double => JType::Double,
            ElemType::Boolean => JType::Boolean,
            ElemType::Char => JType::Char,
            ElemType::Str => JType::Str,
        }
    }
}

/// The static type of an expression on the operand stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JType {
    Int,
    Double,
    Boolean,
    Char,
    /// `java.lang.String`.
    Str,
    /// The type of the `null` literal.
    Null,
    /// An array: `int[]` is `{elem: Int, dims: 1}`, `int[][]` has
    /// `dims: 2`, and so on.
    Array {
        elem: ElemType,
        dims: u8,
    },
    /// A type jvmjs doesn't handle yet.
    Unsupported,
    /// A diagnostic was already reported for this subtree.
    Error,
}

impl JType {
    fn describe(self) -> String {
        match self {
            JType::Int => String::from("int"),
            JType::Double => String::from("double"),
            JType::Boolean => String::from("boolean"),
            JType::Char => String::from("char"),
            JType::Str => String::from("String"),
            JType::Null => String::from("null"),
            JType::Array { elem, dims } => {
                let mut out = elem.base_type().describe();
                for _ in 0..dims {
                    out.push_str("[]");
                }
                out
            }
            JType::Unsupported => String::from("an unsupported type"),
            JType::Error => String::from("an unknown type"),
        }
    }

    fn is_numeric(self) -> bool {
        matches!(self, JType::Int | JType::Double | JType::Char)
    }

    fn is_reference(self) -> bool {
        matches!(self, JType::Str | JType::Null | JType::Array { .. })
    }

    /// Width in operand-stack slots (JVMS §2.6.2).
    fn width(self) -> u16 {
        if self == JType::Double { 2 } else { 1 }
    }

    /// The type of this array's elements (one dimension down).
    fn element_type(self) -> Option<JType> {
        match self {
            JType::Array { elem, dims: 1 } => Some(elem.base_type()),
            JType::Array { elem, dims } => Some(JType::Array {
                elem,
                dims: dims - 1,
            }),
            _ => None,
        }
    }

    /// The JVM field descriptor for this type.
    fn descriptor(self) -> String {
        match self {
            JType::Int => String::from("I"),
            JType::Double => String::from("D"),
            JType::Boolean => String::from("Z"),
            JType::Char => String::from("C"),
            JType::Str | JType::Null => String::from("Ljava/lang/String;"),
            JType::Array { elem, dims } => {
                let mut out = "[".repeat(usize::from(dims));
                out.push_str(elem.descriptor());
                out
            }
            // Only reachable for methods that already produced a
            // diagnostic; the descriptor keeps the class file coherent.
            JType::Unsupported | JType::Error => String::from("Ljava/lang/Object;"),
        }
    }
}

/// Binary numeric promotion (JLS §5.6.2), within the CSA type set.
fn promote(a: JType, b: JType) -> JType {
    if a == JType::Double || b == JType::Double {
        JType::Double
    } else {
        JType::Int
    }
}

fn type_from_ref(ty: &TypeRef) -> Option<JType> {
    match ty {
        TypeRef::Int => Some(JType::Int),
        TypeRef::Double => Some(JType::Double),
        TypeRef::Boolean => Some(JType::Boolean),
        TypeRef::Char => Some(JType::Char),
        TypeRef::Named(name) if name == "String" => Some(JType::Str),
        TypeRef::Array(inner) => {
            let mut dims: u8 = 1;
            let mut current = inner.as_ref();
            while let TypeRef::Array(next) = current {
                dims = dims.checked_add(1)?;
                current = next;
            }
            let elem = match current {
                TypeRef::Int => ElemType::Int,
                TypeRef::Double => ElemType::Double,
                TypeRef::Boolean => ElemType::Boolean,
                TypeRef::Char => ElemType::Char,
                TypeRef::Named(name) if name == "String" => ElemType::Str,
                _ => return None,
            };
            Some(JType::Array { elem, dims })
        }
        _ => None,
    }
}

struct LocalVar {
    slot: u16,
    ty: JType,
    is_final: bool,
    assigned: bool,
}

fn emit_method(
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    table: &MethodTable,
    current_class: &str,
    pool: &mut ConstantPool,
    decl: &MethodDecl,
) -> MethodInfo {
    let return_type = match &decl.return_type {
        TypeRef::Void => None,
        other => Some(type_from_ref(other).unwrap_or(JType::Unsupported)),
    };
    let mut body = BodyGen {
        path,
        diagnostics,
        pool,
        table,
        current_class,
        return_type,
        code: CodeBuilder::new(),
        scopes: vec![Vec::new()],
        next_slot: u16::from(!decl.is_static),
        loop_stack: Vec::new(),
    };

    for param in &decl.params {
        let ty = type_from_ref(&param.ty).unwrap_or(JType::Unsupported);
        let slot = body.next_slot;
        body.next_slot += ty.width();
        body.scopes[0].push((
            param.name.clone(),
            LocalVar {
                slot,
                ty,
                is_final: false,
                assigned: true,
            },
        ));
    }

    for stmt in &decl.body {
        body.statement(stmt);
    }
    // javac-style missing-return check: a non-void method whose body
    // can complete normally is an error (JLS §8.4.7).
    if !matches!(body.return_type, None | Some(JType::Error))
        && block_completes_normally(&decl.body)
    {
        body.error(decl.span, "missing return statement");
    }
    body.code.push_op(op::RETURN, 0);

    let max_locals = body.next_slot;
    let (bytecode, max_stack) = body.code.finish();

    let descriptor = method_descriptor(path, diagnostics, decl);
    let mut flags = 0;
    if decl.is_public {
        flags |= MethodAccessFlags::PUBLIC;
    }
    if decl.is_static {
        flags |= MethodAccessFlags::STATIC;
    }

    let code_attribute = CodeAttribute {
        max_stack,
        max_locals,
        code: bytecode,
        exception_table: Vec::new(),
        attributes: Vec::new(),
    };
    let name_index = pool.intern_utf8(&decl.name);
    let descriptor_index = pool.intern_utf8(&descriptor);
    let code_name_index = pool.intern_utf8(CODE_ATTRIBUTE);
    MethodInfo {
        access_flags: MethodAccessFlags(flags),
        name_index,
        descriptor_index,
        attributes: vec![AttributeInfo {
            name_index: code_name_index,
            info: write_code_attribute(&code_attribute),
        }],
    }
}

fn method_descriptor(path: &str, diagnostics: &mut Vec<Diagnostic>, decl: &MethodDecl) -> String {
    fn push_type(
        path: &str,
        diagnostics: &mut Vec<Diagnostic>,
        out: &mut String,
        ty: &TypeRef,
        span: SourceSpan,
    ) {
        match ty {
            TypeRef::Void => out.push('V'),
            TypeRef::Int => out.push('I'),
            TypeRef::Double => out.push('D'),
            TypeRef::Boolean => out.push('Z'),
            TypeRef::Char => out.push('C'),
            TypeRef::Array(inner) => {
                out.push('[');
                push_type(path, diagnostics, out, inner, span);
            }
            TypeRef::Named(name) => {
                if name == "String" {
                    out.push_str("Ljava/lang/String;");
                } else {
                    // TODO(classlib): resolve simple names against the
                    // compilation unit and the class library.
                    diagnostics.push(Diagnostic::error(
                        path,
                        format!("unknown type '{name}'"),
                        span,
                    ));
                    out.push_str("Ljava/lang/Object;");
                }
            }
        }
    }

    let mut descriptor = String::from("(");
    for param in &decl.params {
        push_type(path, diagnostics, &mut descriptor, &param.ty, decl.span);
    }
    descriptor.push(')');
    push_type(
        path,
        diagnostics,
        &mut descriptor,
        &decl.return_type,
        decl.span,
    );
    descriptor
}

/// Reachability-lite (JLS §14.21): whether a statement can complete
/// normally, used for the missing-return check. Conservative on
/// non-constant loop conditions, exact on the patterns students write
/// (`while (true)` without `break` does not complete).
fn stmt_completes_normally(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => false,
        Stmt::Block(statements) => block_completes_normally(statements),
        Stmt::If {
            then,
            els: Some(els),
            ..
        } => stmt_completes_normally(then) || stmt_completes_normally(els),
        // (An `if` without `else` falls through to `true` below: the
        // condition may be false.)
        // A constant-true loop only completes via `break`.
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            !is_true_literal(cond) || has_direct_break(body)
        }
        Stmt::For { cond, body, .. } => match cond {
            Some(cond) if !is_true_literal(cond) => true,
            // `for (;;)` or `for (; true;)`.
            _ => has_direct_break(body),
        },
        _ => true,
    }
}

fn block_completes_normally(statements: &[Stmt]) -> bool {
    statements.iter().all(stmt_completes_normally)
}

fn is_true_literal(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Literal {
            value: Literal::Bool(true),
            ..
        }
    )
}

/// Whether a loop body contains a `break` binding to THAT loop (nested
/// loops keep their own breaks).
fn has_direct_break(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Break { .. } => true,
        Stmt::Block(statements) => statements.iter().any(has_direct_break),
        Stmt::If { then, els, .. } => {
            has_direct_break(then) || els.as_deref().is_some_and(has_direct_break)
        }
        _ => false,
    }
}

/// What a method call's receiver refers to.
enum CallTarget {
    /// `System.out` / `System.err`.
    Stream(&'static str),
    /// Static methods of the named class (bare calls target the
    /// current class).
    Static(String),
}

fn describe_types(types: &[JType]) -> String {
    let names: Vec<String> = types.iter().map(|t| t.describe()).collect();
    names.join(",")
}

/// Break/continue targets for the innermost enclosing loop.
#[derive(Clone, Copy)]
struct LoopLabels {
    break_label: Label,
    continue_label: Label,
}

/// Per-method-body emission state.
struct BodyGen<'a> {
    path: &'a str,
    diagnostics: &'a mut Vec<Diagnostic>,
    pool: &'a mut ConstantPool,
    table: &'a MethodTable,
    current_class: &'a str,
    /// Declared return type; `None` is `void`.
    return_type: Option<JType>,
    code: CodeBuilder,
    /// Lexical scopes (innermost last); Java forbids shadowing locals,
    /// so declaration checks search all of them.
    scopes: Vec<Vec<(String, LocalVar)>>,
    next_slot: u16,
    /// Innermost-last enclosing loops, for `break`/`continue`.
    loop_stack: Vec<LoopLabels>,
}

impl BodyGen<'_> {
    fn error(&mut self, span: SourceSpan, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(self.path, message, span));
    }

    fn lookup(&mut self, name: &str) -> Option<&mut LocalVar> {
        self.scopes
            .iter_mut()
            .rev()
            .flat_map(|scope| scope.iter_mut())
            .find(|(n, _)| n == name)
            .map(|(_, var)| var)
    }

    // ----- statements -----

    fn statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                self.scopes.push(Vec::new());
                for inner in statements {
                    self.statement(inner);
                }
                self.scopes.pop();
            }
            Stmt::LocalDecl {
                ty,
                is_final,
                declarators,
                span,
            } => {
                self.local_decl(ty, *is_final, declarators, *span);
            }
            Stmt::Assign {
                target,
                op,
                value,
                span,
            } => match target {
                AssignTarget::Var(name) => self.assign(name, *op, value, *span),
                AssignTarget::Index { array, index } => {
                    self.assign_element(array, index, *op, value, *span);
                }
            },
            Stmt::ForEach {
                ty,
                name,
                iterable,
                body,
                span,
            } => self.for_each(ty, name, iterable, body, *span),
            Stmt::Expr(expr) => self.expression_statement(expr),
            Stmt::If {
                cond, then, els, ..
            } => self.if_statement(cond, then, els.as_deref()),
            Stmt::While { cond, body, .. } => self.while_statement(cond, body),
            Stmt::DoWhile { body, cond, .. } => self.do_while_statement(body, cond),
            Stmt::For {
                init,
                cond,
                update,
                body,
                ..
            } => self.for_statement(init.as_deref(), cond.as_ref(), update, body),
            Stmt::Break { span } => match self.loop_stack.last() {
                Some(labels) => {
                    let target = labels.break_label;
                    self.code.branch(op::GOTO, target, 0);
                }
                None => self.error(*span, "'break' can only be used inside a loop"),
            },
            Stmt::Continue { span } => match self.loop_stack.last() {
                Some(labels) => {
                    let target = labels.continue_label;
                    self.code.branch(op::GOTO, target, 0);
                }
                None => self.error(*span, "'continue' can only be used inside a loop"),
            },
            Stmt::Return { value, span } => self.return_statement(value.as_ref(), *span),
        }
    }

    fn return_statement(&mut self, value: Option<&Expr>, span: SourceSpan) {
        match (self.return_type, value) {
            (None, None) => self.code.push_op(op::RETURN, 0),
            (None, Some(value)) => {
                self.expr(value);
                self.error(
                    value.span(),
                    "incompatible types: unexpected return value (this method is void)",
                );
            }
            (Some(expected), None) => {
                if expected != JType::Error {
                    self.error(span, "incompatible types: missing return value");
                }
            }
            (Some(expected), Some(value)) => {
                let actual = self.expr(value);
                self.convert_for_assignment(actual, expected, value.span());
                let opcode = match expected {
                    JType::Double => op::DRETURN,
                    JType::Str | JType::Null => op::ARETURN,
                    _ => op::IRETURN,
                };
                self.code.push_op(opcode, 0);
                self.code.drop_stack(expected.width());
            }
        }
    }

    /// Emit a condition expression, requiring `boolean` as Java does.
    fn condition(&mut self, cond: &Expr, what: &str) {
        let ty = self.expr(cond);
        if ty != JType::Boolean && ty != JType::Error {
            self.error(
                cond.span(),
                format!(
                    "the {what} condition must be a boolean, got {}",
                    ty.describe()
                ),
            );
        }
    }

    fn if_statement(&mut self, cond: &Expr, then: &Stmt, els: Option<&Stmt>) {
        self.condition(cond, "if");
        let before = self.assigned_flags();
        if let Some(els) = els {
            let else_label = self.code.new_label();
            let end = self.code.new_label();
            self.code.branch(op::IFEQ, else_label, 1);
            self.statement(then);
            let after_then = self.assigned_flags();
            self.restore_assigned(&before);
            self.code.branch(op::GOTO, end, 0);
            self.code.bind(else_label);
            self.statement(els);
            // Definitely assigned only if both branches assign
            // (JLS §16 intersection rule).
            self.intersect_assigned(&after_then);
            self.code.bind(end);
        } else {
            let end = self.code.new_label();
            self.code.branch(op::IFEQ, end, 1);
            self.statement(then);
            // A lone if may not run: its assignments don't count.
            self.restore_assigned(&before);
            self.code.bind(end);
        }
    }

    fn while_statement(&mut self, cond: &Expr, body: &Stmt) {
        let start = self.code.new_label();
        let end = self.code.new_label();
        self.code.bind(start);
        self.condition(cond, "while");
        self.code.branch(op::IFEQ, end, 1);
        let before = self.assigned_flags();
        self.loop_stack.push(LoopLabels {
            break_label: end,
            continue_label: start,
        });
        self.statement(body);
        self.loop_stack.pop();
        self.restore_assigned(&before);
        self.code.branch(op::GOTO, start, 0);
        self.code.bind(end);
    }

    fn do_while_statement(&mut self, body: &Stmt, cond: &Expr) {
        let start = self.code.new_label();
        let continue_label = self.code.new_label();
        let end = self.code.new_label();
        self.code.bind(start);
        self.loop_stack.push(LoopLabels {
            break_label: end,
            continue_label,
        });
        // A do-while body always runs once, so its assignments stick.
        self.statement(body);
        self.loop_stack.pop();
        self.code.bind(continue_label);
        self.condition(cond, "do-while");
        self.code.branch(op::IFNE, start, 1);
        self.code.bind(end);
    }

    fn for_statement(
        &mut self,
        init: Option<&Stmt>,
        cond: Option<&Expr>,
        update: &[Stmt],
        body: &Stmt,
    ) {
        // The init declaration is scoped to the loop.
        self.scopes.push(Vec::new());
        if let Some(init) = init {
            self.statement(init);
        }
        let cond_label = self.code.new_label();
        let update_label = self.code.new_label();
        let end = self.code.new_label();
        self.code.bind(cond_label);
        if let Some(cond) = cond {
            self.condition(cond, "for");
            self.code.branch(op::IFEQ, end, 1);
        }
        let before = self.assigned_flags();
        self.loop_stack.push(LoopLabels {
            break_label: end,
            continue_label: update_label,
        });
        self.statement(body);
        self.loop_stack.pop();
        self.code.bind(update_label);
        for stmt in update {
            self.statement(stmt);
        }
        self.restore_assigned(&before);
        self.code.branch(op::GOTO, cond_label, 0);
        self.code.bind(end);
        self.scopes.pop();
    }

    // ----- branch-aware definite assignment -----
    //
    // The tracker is deliberately simple: per-variable `assigned` flags
    // snapshotted around branches. Both if-branches assigning counts;
    // loop bodies (which may run zero times) don't. This matches javac
    // on everything students write, minus constant-condition special
    // cases (`while (true)`), where we stay conservative.

    fn assigned_flags(&self) -> Vec<Vec<bool>> {
        self.scopes
            .iter()
            .map(|scope| scope.iter().map(|(_, var)| var.assigned).collect())
            .collect()
    }

    /// Reset flags to a snapshot (branch bodies whose execution isn't
    /// guaranteed). Scopes pushed since the snapshot are untouched —
    /// they can only be the current statement's own, already popped.
    fn restore_assigned(&mut self, snapshot: &[Vec<bool>]) {
        for (scope, flags) in self.scopes.iter_mut().zip(snapshot) {
            for ((_, var), flag) in scope.iter_mut().zip(flags) {
                var.assigned = *flag;
            }
        }
    }

    /// Keep a variable assigned only if the other branch (whose flags
    /// are `other`) assigned it too.
    fn intersect_assigned(&mut self, other: &[Vec<bool>]) {
        for (scope, flags) in self.scopes.iter_mut().zip(other) {
            for ((_, var), flag) in scope.iter_mut().zip(flags) {
                var.assigned = var.assigned && *flag;
            }
        }
    }

    fn local_decl(
        &mut self,
        ty: &TypeRef,
        is_final: bool,
        declarators: &[LocalDeclarator],
        span: SourceSpan,
    ) {
        let Some(var_ty) = type_from_ref(ty) else {
            let what = match ty {
                TypeRef::Named(name) => format!("unknown type '{name}'"),
                TypeRef::Array(_) => String::from("arrays are not yet supported by jvmjs"),
                _ => String::from("this type cannot be used for a variable"),
            };
            self.error(span, what);
            return;
        };

        for declarator in declarators {
            if self.lookup(&declarator.name).is_some() {
                self.error(
                    declarator.span,
                    format!(
                        "variable '{}' is already defined in this method",
                        declarator.name
                    ),
                );
                continue;
            }
            let slot = self.next_slot;
            self.next_slot += var_ty.width();

            let assigned = if let Some(init) = &declarator.init {
                // `int[] a = {1, 2};` — the literal takes its type
                // from the declaration.
                if let Expr::ArrayLiteral { elements, span } = init {
                    if matches!(var_ty, JType::Array { .. }) {
                        self.emit_array_literal(elements, var_ty, *span);
                    } else {
                        self.error(
                            *span,
                            format!("illegal initializer for {}", var_ty.describe()),
                        );
                    }
                } else {
                    let init_ty = self.expr(init);
                    self.convert_for_assignment(init_ty, var_ty, init.span());
                }
                self.emit_store(slot, var_ty);
                true
            } else {
                false
            };
            self.scopes
                .last_mut()
                .expect("scope stack is never empty")
                .push((
                    declarator.name.clone(),
                    LocalVar {
                        slot,
                        ty: var_ty,
                        is_final,
                        assigned,
                    },
                ));
        }
    }

    fn assign(&mut self, name: &str, op: Option<BinaryOp>, value: &Expr, span: SourceSpan) {
        let Some(var) = self.lookup(name) else {
            self.error(
                span,
                format!("cannot find variable '{name}' — declare it first"),
            );
            // Still emit the value expression for its own diagnostics.
            self.expr(value);
            self.code.discard();
            return;
        };
        let (slot, var_ty, is_final, assigned) = (var.slot, var.ty, var.is_final, var.assigned);

        match op {
            None => {
                if is_final && assigned {
                    self.error(span, format!("cannot assign to final variable '{name}'"));
                } else if is_final && !self.loop_stack.is_empty() {
                    self.error(
                        span,
                        format!("final variable '{name}' might be assigned in a loop"),
                    );
                }
                let value_ty = self.expr(value);
                self.convert_for_assignment(value_ty, var_ty, value.span());
                self.emit_store(slot, var_ty);
                if let Some(var) = self.lookup(name) {
                    var.assigned = true;
                }
            }
            Some(op) => {
                if is_final {
                    self.error(span, format!("cannot assign to final variable '{name}'"));
                }
                if !assigned {
                    self.error(
                        span,
                        format!("variable '{name}' might not have been initialized"),
                    );
                }
                // `s += x` is string concatenation when s is a String.
                if var_ty == JType::Str {
                    if op != BinaryOp::Add {
                        self.error(span, "only '+=' can be applied to a String");
                        return;
                    }
                    self.emit_load(slot, var_ty);
                    self.begin_concat_with_value_on_stack(JType::Str);
                    let part_ty = self.expr(value);
                    self.append_part(part_ty, value.span());
                    self.finish_concat();
                    self.emit_store(slot, var_ty);
                    return;
                }

                let value_ty = self.type_of(value);
                if var_ty == JType::Boolean || !value_ty.is_numeric() {
                    if value_ty != JType::Error && var_ty != JType::Boolean {
                        self.error(
                            value.span(),
                            format!(
                                "operator '{}' cannot be applied to {} and {}",
                                compound_symbol(op),
                                var_ty.describe(),
                                value_ty.describe()
                            ),
                        );
                    } else if var_ty == JType::Boolean {
                        self.error(span, "compound assignment cannot be applied to a boolean");
                    }
                    return;
                }

                // Compound assignment: promote, operate, then cast back
                // to the variable's type implicitly (JLS §15.26.2).
                let promoted = promote(var_ty, value_ty);
                self.emit_load(slot, var_ty);
                self.numeric_conversion(var_ty, promoted);
                let actual = self.expr(value);
                self.numeric_conversion(actual, promoted);
                self.arithmetic_op(op, promoted);
                self.narrow_back(promoted, var_ty);
                self.emit_store(slot, var_ty);
            }
        }
    }

    // ----- arrays -----

    /// Emit array reference + index for an element access, returning
    /// the element type.
    fn array_and_index(&mut self, array: &Expr, index: &Expr) -> Option<JType> {
        let array_ty = self.expr(array);
        let Some(element) = array_ty.element_type() else {
            if array_ty != JType::Error {
                self.error(
                    array.span(),
                    format!("array required, but {} found", array_ty.describe()),
                );
            }
            return None;
        };
        let index_ty = self.expr(index);
        if !matches!(index_ty, JType::Int | JType::Char | JType::Error) {
            self.error(
                index.span(),
                format!(
                    "incompatible types: {} cannot be converted to int (array index)",
                    index_ty.describe()
                ),
            );
        }
        Some(element)
    }

    /// The array-load opcode for an element type.
    fn xaload(&mut self, element: JType) {
        let opcode = match element {
            JType::Double => op::DALOAD,
            JType::Boolean => op::BALOAD,
            JType::Char => op::CALOAD,
            JType::Int => op::IALOAD,
            _ => op::AALOAD,
        };
        // Pops arrayref + index, pushes the element.
        self.code.push_op(opcode, element.width());
        self.code.drop_stack(2 + element.width());
        self.code.grow_stack(element.width());
    }

    /// The array-store opcode for an element type.
    fn xastore(&mut self, element: JType) {
        let opcode = match element {
            JType::Double => op::DASTORE,
            JType::Boolean => op::BASTORE,
            JType::Char => op::CASTORE,
            JType::Int => op::IASTORE,
            _ => op::AASTORE,
        };
        self.code.push_op(opcode, 0);
        self.code.drop_stack(2 + element.width());
    }

    /// `a[i] = v`, `a[i] += v`, `a[i]++` (as `+= 1`).
    fn assign_element(
        &mut self,
        array: &Expr,
        index: &Expr,
        op_kind: Option<BinaryOp>,
        value: &Expr,
        span: SourceSpan,
    ) {
        let Some(element) = self.array_and_index(array, index) else {
            self.expr(value);
            self.code.discard();
            return;
        };

        match op_kind {
            None => {
                let value_ty = self.expr(value);
                self.convert_for_assignment(value_ty, element, value.span());
                self.xastore(element);
            }
            Some(op_kind) => {
                if element == JType::Str {
                    if op_kind != BinaryOp::Add {
                        self.error(span, "only '+=' can be applied to a String element");
                        return;
                    }
                    // arrayref, index on stack: duplicate for the
                    // read-modify-write.
                    self.code.push_op(op::DUP2, 2);
                    self.xaload(element);
                    self.begin_concat_with_value_on_stack(JType::Str);
                    let part_ty = self.expr(value);
                    self.append_part(part_ty, value.span());
                    self.finish_concat();
                    self.xastore(element);
                    return;
                }
                let value_ty = self.type_of(value);
                if element == JType::Boolean || !element.is_numeric() || !value_ty.is_numeric() {
                    if value_ty != JType::Error {
                        self.error(
                            span,
                            format!(
                                "operator '{}' cannot be applied to {} and {}",
                                compound_symbol(op_kind),
                                element.describe(),
                                value_ty.describe()
                            ),
                        );
                    }
                    return;
                }
                let promoted = promote(element, value_ty);
                self.code.push_op(op::DUP2, 2);
                self.xaload(element);
                self.numeric_conversion(element, promoted);
                let actual = self.expr(value);
                self.numeric_conversion(actual, promoted);
                self.arithmetic_op(op_kind, promoted);
                self.narrow_back(promoted, element);
                self.xastore(element);
            }
        }
    }

    /// `for (Type name : array) body`, desugared to an indexed loop
    /// over synthetic (unnamed) locals.
    fn for_each(
        &mut self,
        ty: &TypeRef,
        name: &str,
        iterable: &Expr,
        body: &Stmt,
        span: SourceSpan,
    ) {
        let iterable_ty = self.expr(iterable);
        let Some(element) = iterable_ty.element_type() else {
            if iterable_ty != JType::Error {
                self.error(
                    iterable.span(),
                    format!(
                        "for-each needs an array, but {} found (ArrayList arrives later)",
                        iterable_ty.describe()
                    ),
                );
            }
            return;
        };
        let Some(var_ty) = type_from_ref(ty) else {
            self.error(span, "unknown type for the for-each variable");
            self.code.discard();
            return;
        };

        // Synthetic slots for the array and the index.
        let array_slot = self.next_slot;
        self.next_slot += 1;
        let index_slot = self.next_slot;
        self.next_slot += 1;
        self.emit_store(array_slot, iterable_ty);
        self.code.push_op(op::ICONST_0, 1);
        self.emit_store(index_slot, JType::Int);

        // The loop variable lives in its own scope.
        self.scopes.push(Vec::new());
        if self.lookup(name).is_some() {
            self.error(
                span,
                format!("variable '{name}' is already defined in this method"),
            );
        }
        let var_slot = self.next_slot;
        self.next_slot += var_ty.width();
        self.scopes.last_mut().expect("scope pushed").push((
            name.to_owned(),
            LocalVar {
                slot: var_slot,
                ty: var_ty,
                is_final: false,
                assigned: true,
            },
        ));

        let cond_label = self.code.new_label();
        let continue_label = self.code.new_label();
        let end = self.code.new_label();

        self.code.bind(cond_label);
        self.emit_load(index_slot, JType::Int);
        self.emit_load(array_slot, iterable_ty);
        self.code.push_op(op::ARRAYLENGTH, 1);
        self.code.drop_stack(1);
        self.code.branch(op::IF_ICMPGE, end, 2);

        self.emit_load(array_slot, iterable_ty);
        self.emit_load(index_slot, JType::Int);
        self.xaload(element);
        self.convert_for_assignment(element, var_ty, span);
        self.emit_store(var_slot, var_ty);

        let before = self.assigned_flags();
        self.loop_stack.push(LoopLabels {
            break_label: end,
            continue_label,
        });
        self.statement(body);
        self.loop_stack.pop();
        self.restore_assigned(&before);

        self.code.bind(continue_label);
        self.emit_load(index_slot, JType::Int);
        self.code.push_op(op::ICONST_1, 1);
        self.code.push_op(op::IADD, 0);
        self.code.drop_stack(1);
        self.emit_store(index_slot, JType::Int);
        self.code.branch(op::GOTO, cond_label, 0);
        self.code.bind(end);
        self.scopes.pop();
    }

    fn expression_statement(&mut self, expr: &Expr) {
        let Expr::Call {
            receiver,
            method,
            args,
            span,
        } = expr
        else {
            self.error(expr.span(), "this expression is not a statement in Java");
            return;
        };

        match self.call_target(receiver.as_deref(), *span) {
            None => {}
            Some(CallTarget::Stream(stream)) => self.print_call(stream, method, args, *span),
            Some(CallTarget::Static(class)) => {
                // The result of a call statement (if any) is discarded.
                if let Some(Some(ty)) = self.static_call(&class, method, args, *span) {
                    let opcode = if ty.width() == 2 { op::POP2 } else { op::POP };
                    self.code.push_op(opcode, 0);
                    self.code.drop_stack(ty.width());
                }
            }
        }
    }

    /// Classify what a call's receiver refers to, reporting an error
    /// for unsupported shapes.
    fn call_target(&mut self, receiver: Option<&Expr>, _span: SourceSpan) -> Option<CallTarget> {
        match receiver {
            None => Some(CallTarget::Static(self.current_class.to_owned())),
            Some(Expr::Name {
                path,
                span: receiver_span,
            }) => match path.iter().map(String::as_str).collect::<Vec<_>>()[..] {
                ["System", "out"] => Some(CallTarget::Stream("out")),
                ["System", "err"] => Some(CallTarget::Stream("err")),
                [single] => {
                    if self.lookup(single).is_some() {
                        self.error(
                            *receiver_span,
                            "method calls on values are not yet supported by jvmjs \
                             (objects arrive in a later stage)",
                        );
                        None
                    } else if self.table.has_class(single) {
                        Some(CallTarget::Static(single.to_owned()))
                    } else {
                        self.error(*receiver_span, format!("cannot find symbol: '{single}'"));
                        None
                    }
                }
                _ => {
                    self.error(
                        *receiver_span,
                        "only System.out/System.err and ClassName.method(...) calls are \
                         supported by jvmjs so far",
                    );
                    None
                }
            },
            Some(other) => {
                self.error(
                    other.span(),
                    "method calls on values are not yet supported by jvmjs \
                     (objects arrive in a later stage)",
                );
                None
            }
        }
    }

    /// Resolve and emit a static method call. `None` means a
    /// diagnostic was reported; `Some(ret)` is the method's return
    /// type (`None` for void), with the value left on the stack.
    #[allow(clippy::option_option)] // error / void / value are three distinct outcomes
    fn static_call(
        &mut self,
        class: &str,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            // Emit the arguments so their own diagnostics surface.
            for arg in args {
                self.expr(arg);
            }
            return None;
        }

        let table = self.table;
        let sig = match table.resolve(class, method, &arg_types) {
            Resolution::Found(sig) => sig.clone(),
            Resolution::UnknownName => {
                self.error(
                    span,
                    format!(
                        "cannot find symbol: method {method}({}) in class {class}",
                        describe_types(&arg_types)
                    ),
                );
                return None;
            }
            Resolution::NoneApplicable => {
                self.error(
                    span,
                    format!(
                        "no suitable method found for {method}({}) in class {class}",
                        describe_types(&arg_types)
                    ),
                );
                return None;
            }
            Resolution::Ambiguous(candidates) => {
                self.error(
                    span,
                    format!(
                        "reference to {method} is ambiguous: both method {} match",
                        candidates.join(" and method ")
                    ),
                );
                return None;
            }
        };

        if !sig.is_static {
            self.error(
                span,
                format!(
                    "non-static method {method}() cannot be referenced from a static context \
                     (instance methods arrive with objects)"
                ),
            );
            return None;
        }
        if sig.params.contains(&JType::Unsupported) || sig.ret == Some(JType::Unsupported) {
            self.error(
                span,
                format!(
                    "method {} uses types not yet supported by jvmjs",
                    sig.describe()
                ),
            );
            return None;
        }

        for (arg, param) in args.iter().zip(&sig.params) {
            let actual = self.expr(arg);
            self.numeric_conversion(actual, *param);
        }
        let method_ref = intern_method_ref(self.pool, class, method, &sig.descriptor());
        let ret_width = sig.ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKESTATIC, method_ref, ret_width);
        let args_width: u16 = sig.params.iter().map(|p| p.width()).sum();
        self.code.drop_stack(args_width);
        Some(sig.ret)
    }

    /// `System.out.println(...)` and friends.
    fn print_call(&mut self, stream: &'static str, method: &str, args: &[Expr], span: SourceSpan) {
        if method != "println" && method != "print" {
            self.error(
                span,
                format!("PrintStream.{method} is not supported by jvmjs (try print or println)"),
            );
            return;
        }

        let field = intern_field_ref(
            self.pool,
            "java/lang/System",
            stream,
            "Ljava/io/PrintStream;",
        );
        self.code.push_op_u16(op::GETSTATIC, field, 1);

        let arg_descriptor = match args {
            [] => {
                if method == "print" {
                    self.error(span, "print() requires an argument (println() does not)");
                    return;
                }
                String::from("()V")
            }
            [arg] => {
                let arg_ty = self.expr(arg);
                match self.print_descriptor(arg_ty, arg.span()) {
                    Some(descriptor) => descriptor,
                    None => return,
                }
            }
            _ => {
                self.error(span, "print/println take at most one argument");
                return;
            }
        };

        let method_ref =
            intern_method_ref(self.pool, "java/io/PrintStream", method, &arg_descriptor);
        self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 0);
        self.code
            .drop_stack(1 + descriptor_arg_width(&arg_descriptor));
    }

    fn print_descriptor(&mut self, ty: JType, span: SourceSpan) -> Option<String> {
        match ty {
            JType::Int => Some(String::from("(I)V")),
            JType::Double => Some(String::from("(D)V")),
            JType::Boolean => Some(String::from("(Z)V")),
            JType::Char => Some(String::from("(C)V")),
            JType::Str => Some(String::from("(Ljava/lang/String;)V")),
            JType::Null => {
                self.error(
                    span,
                    "println(null) is ambiguous in Java; pass a String or \"null\"",
                );
                None
            }
            JType::Array { .. } => {
                self.error(
                    span,
                    "printing an array directly is not supported by jvmjs \
                     (print its elements in a loop)",
                );
                None
            }
            JType::Unsupported => {
                self.error(span, "this value's type is not yet supported by jvmjs");
                None
            }
            JType::Error => None,
        }
    }

    // ----- expressions -----

    /// The static type of an expression, without emitting code. Must
    /// agree with what [`Self::expr`] leaves on the stack.
    fn type_of(&mut self, expr: &Expr) -> JType {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Int(_) => JType::Int,
                Literal::Double(_) => JType::Double,
                Literal::Str(_) => JType::Str,
                Literal::Char(_) => JType::Char,
                Literal::Bool(_) => JType::Boolean,
                Literal::Null => JType::Null,
            },
            Expr::Name { path, .. } if path.len() == 1 => {
                self.lookup(&path[0]).map_or(JType::Error, |v| v.ty)
            }
            Expr::Name { path, .. }
                if path.len() == 2
                    && path[1] == "length"
                    && self
                        .lookup(&path[0])
                        .is_some_and(|v| matches!(v.ty, JType::Array { .. })) =>
            {
                JType::Int
            }
            Expr::Name { .. } | Expr::ArrayLiteral { .. } => JType::Error,
            Expr::Index { array, .. } => self.type_of(array).element_type().unwrap_or(JType::Error),
            Expr::Field { object, name, .. } => {
                if name == "length" && matches!(self.type_of(object), JType::Array { .. }) {
                    JType::Int
                } else {
                    JType::Error
                }
            }
            Expr::NewArray { elem, dims, .. } => {
                match (
                    type_from_ref(elem).and_then(elem_type_of),
                    u8::try_from(dims.len()),
                ) {
                    (Some(element), Ok(count)) => JType::Array {
                        elem: element,
                        dims: count,
                    },
                    _ => JType::Error,
                }
            }
            Expr::Call {
                receiver,
                method,
                args,
                ..
            } => {
                // Mirror emission-path resolution, silently.
                let class = match receiver.as_deref() {
                    None => self.current_class.to_owned(),
                    Some(Expr::Name { path, .. })
                        if path.len() == 1
                            && self.lookup(&path[0]).is_none()
                            && self.table.has_class(&path[0]) =>
                    {
                        path[0].clone()
                    }
                    _ => return JType::Error,
                };
                let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
                let table = self.table;
                match table.resolve(&class, method, &arg_types) {
                    Resolution::Found(sig) => sig.ret.unwrap_or(JType::Error),
                    _ => JType::Error,
                }
            }
            Expr::Unary {
                op: UnaryOp::Not, ..
            } => JType::Boolean,
            Expr::Unary {
                op: UnaryOp::Neg,
                operand,
                ..
            } => match self.type_of(operand) {
                JType::Double => JType::Double,
                t if t.is_numeric() => JType::Int,
                _ => JType::Error,
            },
            Expr::Cast { ty, .. } => type_from_ref(ty).unwrap_or(JType::Error),
            Expr::Binary { op, lhs, rhs, .. } => match op {
                BinaryOp::Add => {
                    let (lt, rt) = (self.type_of(lhs), self.type_of(rhs));
                    if lt == JType::Str || rt == JType::Str {
                        JType::Str
                    } else {
                        promote(lt, rt)
                    }
                }
                BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
                    promote(self.type_of(lhs), self.type_of(rhs))
                }
                _ => JType::Boolean,
            },
        }
    }

    /// Emit code leaving the expression's value on the stack; returns
    /// its type ([`JType::Error`] if a diagnostic was reported).
    fn expr(&mut self, expr: &Expr) -> JType {
        match expr {
            Expr::Literal { value, span } => self.literal(value, *span),
            Expr::Name { path, span } => self.name(path, *span),
            Expr::Call {
                receiver,
                method,
                args,
                span,
            } => match self.call_target(receiver.as_deref(), *span) {
                None => JType::Error,
                Some(CallTarget::Stream(_)) => {
                    self.error(*span, "print/println do not return a value");
                    JType::Error
                }
                Some(CallTarget::Static(class)) => {
                    match self.static_call(&class, method, args, *span) {
                        None => JType::Error,
                        Some(Some(ty)) => ty,
                        Some(None) => {
                            self.error(
                                *span,
                                format!("'{method}' returns void, so it cannot be used as a value"),
                            );
                            JType::Error
                        }
                    }
                }
            },
            Expr::Unary { op, operand, span } => self.unary(*op, operand, *span),
            Expr::Cast { ty, operand, span } => self.cast(ty, operand, *span),
            Expr::Binary { op, lhs, rhs, span } => self.binary(*op, lhs, rhs, *span),
            Expr::Index { array, index, .. } => match self.array_and_index(array, index) {
                Some(element) => {
                    self.xaload(element);
                    element
                }
                None => JType::Error,
            },
            Expr::Field { object, name, span } => self.field(object, name, *span),
            Expr::NewArray {
                elem,
                dims,
                init,
                span,
            } => self.new_array(elem, dims, init.as_deref(), *span),
            Expr::ArrayLiteral { span, .. } => {
                self.error(
                    *span,
                    "an array initializer { ... } can only be used in a declaration \
                     (or write new int[] { ... })",
                );
                JType::Error
            }
        }
    }

    /// Field access on a value: only `.length` on arrays exists so far.
    fn field(&mut self, object: &Expr, name: &str, span: SourceSpan) -> JType {
        let object_ty = self.expr(object);
        if object_ty == JType::Error {
            return JType::Error;
        }
        if name == "length" && matches!(object_ty, JType::Array { .. }) {
            self.code.push_op(op::ARRAYLENGTH, 1);
            self.code.drop_stack(1);
            return JType::Int;
        }
        if object_ty == JType::Str && name == "length" {
            self.error(
                span,
                "String.length() is a method — call it with parentheses \
                 (String methods arrive with the class library)",
            );
            return JType::Error;
        }
        self.error(
            span,
            format!("cannot find field '{name}' on {}", object_ty.describe()),
        );
        JType::Error
    }

    /// `new T[n]`, `new T[n][m]`, `new T[n][]`, `new T[] {...}`.
    fn new_array(
        &mut self,
        elem: &TypeRef,
        dims: &[Option<Expr>],
        init: Option<&[Expr]>,
        span: SourceSpan,
    ) -> JType {
        let Ok(dim_count) = u8::try_from(dims.len()) else {
            self.error(span, "too many array dimensions");
            return JType::Error;
        };
        let Some(element) = type_from_ref(elem).and_then(elem_type_of) else {
            self.error(span, "unknown array element type");
            return JType::Error;
        };
        let array_ty = JType::Array {
            elem: element,
            dims: dim_count,
        };

        if let Some(init) = init {
            self.emit_array_literal(init, array_ty, span);
            return array_ty;
        }

        // Emit the sized dimension counts.
        let sized: Vec<&Expr> = dims.iter().map_while(Option::as_ref).collect();
        for size in &sized {
            let size_ty = self.expr(size);
            if !matches!(size_ty, JType::Int | JType::Char | JType::Error) {
                self.error(
                    size.span(),
                    format!(
                        "incompatible types: {} cannot be converted to int (array size)",
                        size_ty.describe()
                    ),
                );
            }
        }

        if sized.len() == 1 && dim_count == 1 {
            self.emit_new_1d(element);
        } else {
            // Multi-dimensional (or partially-sized) creation.
            let class_index = {
                let descriptor = array_ty.descriptor();
                intern_class(self.pool, &descriptor)
            };
            if sized.len() == 1 {
                // e.g. `new int[3][]` — one dimension allocated, rows null.
                // anewarray's element class is one dimension down.
                let element_descriptor = array_ty
                    .element_type()
                    .expect("array has an element type")
                    .descriptor();
                let element_class = intern_class(self.pool, &element_descriptor);
                self.code.push_op_u16(op::ANEWARRAY, element_class, 1);
                self.code.drop_stack(1);
            } else {
                self.code.bytes.push(op::MULTIANEWARRAY);
                self.code
                    .bytes
                    .extend_from_slice(&class_index.to_be_bytes());
                self.code
                    .bytes
                    .push(u8::try_from(sized.len()).expect("dims fit u8"));
                self.code
                    .drop_stack(u16::try_from(sized.len()).expect("dims fit u16"));
                self.code.grow_stack(1);
            }
        }
        array_ty
    }

    /// Allocate a one-dimensional array of `element` with the length
    /// already on the stack.
    fn emit_new_1d(&mut self, element: ElemType) {
        match element {
            ElemType::Str => {
                let class = intern_class(self.pool, "java/lang/String");
                self.code.push_op_u16(op::ANEWARRAY, class, 1);
                self.code.drop_stack(1);
            }
            prim => {
                let atype = match prim {
                    ElemType::Int => op::T_INT,
                    ElemType::Double => op::T_DOUBLE,
                    ElemType::Boolean => op::T_BOOLEAN,
                    ElemType::Char => op::T_CHAR,
                    ElemType::Str => unreachable!(),
                };
                self.code.push_op(op::NEWARRAY, 1);
                self.code.bytes.push(atype);
                self.code.drop_stack(1);
            }
        }
    }

    /// Emit an array from a `{...}` literal, leaving the reference on
    /// the stack. Nested literals build the inner dimensions.
    fn emit_array_literal(&mut self, elements: &[Expr], array_ty: JType, span: SourceSpan) {
        let Some(element) = array_ty.element_type() else {
            self.error(
                span,
                format!(
                    "array initializer cannot be assigned to {}",
                    array_ty.describe()
                ),
            );
            return;
        };
        let Ok(count) = i32::try_from(elements.len()) else {
            self.error(span, "array initializer is too large");
            return;
        };
        self.push_int(count);
        if let JType::Array { elem, dims: 1 } = array_ty {
            self.emit_new_1d(elem);
        } else {
            let element_descriptor = element.descriptor();
            let element_class = intern_class(self.pool, &element_descriptor);
            self.code.push_op_u16(op::ANEWARRAY, element_class, 1);
            self.code.drop_stack(1);
        }

        for (position, value) in elements.iter().enumerate() {
            self.code.push_op(op::DUP, 1);
            self.push_int(i32::try_from(position).expect("checked above"));
            if let Expr::ArrayLiteral {
                elements: nested, ..
            } = value
            {
                self.emit_array_literal(nested, element, value.span());
            } else {
                let value_ty = self.expr(value);
                self.convert_for_assignment(value_ty, element, value.span());
            }
            self.xastore(element);
        }
    }

    fn literal(&mut self, literal: &Literal, span: SourceSpan) -> JType {
        match literal {
            Literal::Int(value) => {
                let Ok(value) = i32::try_from(*value) else {
                    self.error(
                        span,
                        format!("integer literal {value} is out of range for int"),
                    );
                    return JType::Error;
                };
                self.push_int(value);
                JType::Int
            }
            Literal::Char(value) => {
                let Ok(code_unit) = u16::try_from(*value as u32) else {
                    self.error(span, "character literal does not fit in a Java char");
                    return JType::Error;
                };
                self.push_int(i32::from(code_unit));
                JType::Char
            }
            Literal::Bool(value) => {
                self.code
                    .push_op(if *value { op::ICONST_1 } else { op::ICONST_0 }, 1);
                JType::Boolean
            }
            Literal::Double(value) => {
                let index = self.pool.intern(Constant::Double(*value));
                self.code.push_op_u16(op::LDC2_W, index, 2);
                JType::Double
            }
            Literal::Str(value) => {
                let utf8 = self.pool.intern_utf8(value);
                let index = self.pool.intern(Constant::String { string_index: utf8 });
                self.code.push_ldc(index);
                JType::Str
            }
            Literal::Null => {
                self.code.push_op(op::ACONST_NULL, 1);
                JType::Null
            }
        }
    }

    fn name(&mut self, path: &[String], span: SourceSpan) -> JType {
        // `a.length` parses as a dotted name; resolve it as array
        // length when `a` is an array-typed local.
        if path.len() == 2
            && path[1] == "length"
            && let Some(var) = self.lookup(&path[0])
            && matches!(var.ty, JType::Array { .. })
        {
            let (slot, ty) = (var.slot, var.ty);
            self.emit_load(slot, ty);
            self.code.push_op(op::ARRAYLENGTH, 1);
            self.code.drop_stack(1);
            return JType::Int;
        }
        if path.len() != 1 {
            self.error(
                span,
                format!(
                    "'{}' is not a known variable (field access is not yet supported by jvmjs)",
                    path.join(".")
                ),
            );
            return JType::Error;
        }
        let name = &path[0];
        let Some(var) = self.lookup(name) else {
            self.error(span, format!("cannot find variable '{name}'"));
            return JType::Error;
        };
        let (slot, ty, assigned) = (var.slot, var.ty, var.assigned);
        if ty == JType::Unsupported {
            self.error(
                span,
                format!("the type of '{name}' is not yet supported by jvmjs"),
            );
            return JType::Error;
        }
        if !assigned {
            self.error(
                span,
                format!("variable '{name}' might not have been initialized"),
            );
            return JType::Error;
        }
        self.emit_load(slot, ty);
        ty
    }

    fn unary(&mut self, op: UnaryOp, operand: &Expr, span: SourceSpan) -> JType {
        match op {
            UnaryOp::Neg => {
                let ty = self.expr(operand);
                match ty {
                    JType::Double => {
                        self.code.push_op(op::DNEG, 0);
                        JType::Double
                    }
                    JType::Int | JType::Char => {
                        self.code.push_op(op::INEG, 0);
                        JType::Int
                    }
                    JType::Error => JType::Error,
                    other => {
                        self.error(
                            span,
                            format!("operator '-' cannot be applied to {}", other.describe()),
                        );
                        JType::Error
                    }
                }
            }
            UnaryOp::Not => {
                let ty = self.expr(operand);
                if ty == JType::Error {
                    return JType::Error;
                }
                if ty != JType::Boolean {
                    self.error(
                        span,
                        format!("operator '!' cannot be applied to {}", ty.describe()),
                    );
                    return JType::Error;
                }
                // !x: 0 -> 1, 1 -> 0.
                let flip = self.code.new_label();
                let end = self.code.new_label();
                self.code.branch(op::IFNE, flip, 1);
                self.code.push_op(op::ICONST_1, 1);
                self.code.branch(op::GOTO, end, 0);
                self.code.bind(flip);
                self.code.push_op(op::ICONST_0, 1);
                self.code.bind(end);
                JType::Boolean
            }
        }
    }

    fn cast(&mut self, ty: &TypeRef, operand: &Expr, span: SourceSpan) -> JType {
        let Some(target) = type_from_ref(ty) else {
            self.error(span, "this cast target is not supported by jvmjs");
            self.expr(operand);
            return JType::Error;
        };
        let source = self.expr(operand);
        if source == JType::Error {
            return JType::Error;
        }
        match (source, target) {
            (s, t) if s == t => t,
            (JType::Double, JType::Int) => {
                self.code.push_op(op::D2I, 0);
                self.code.drop_stack(1);
                JType::Int
            }
            (JType::Int | JType::Char, JType::Double) => {
                self.code.push_op(op::I2D, 1);
                JType::Double
            }
            (JType::Int, JType::Char) => {
                self.code.push_op(op::I2C, 0);
                JType::Char
            }
            (JType::Double, JType::Char) => {
                self.code.push_op(op::D2I, 0);
                self.code.drop_stack(1);
                self.code.push_op(op::I2C, 0);
                JType::Char
            }
            (JType::Char, JType::Int) => JType::Int,
            (source, target) => {
                self.error(
                    span,
                    format!("cannot cast {} to {}", source.describe(), target.describe()),
                );
                JType::Error
            }
        }
    }

    fn binary(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: SourceSpan) -> JType {
        match op {
            BinaryOp::And | BinaryOp::Or => self.logical(op, lhs, rhs, span),
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => self.comparison(op, lhs, rhs, span),
            BinaryOp::Add if self.type_of(lhs) == JType::Str || self.type_of(rhs) == JType::Str => {
                self.concat(lhs, rhs)
            }
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
                let (lt, rt) = (self.type_of(lhs), self.type_of(rhs));
                if lt == JType::Error || rt == JType::Error {
                    // Emit for nested diagnostics, then bail.
                    self.expr(lhs);
                    self.expr(rhs);
                    return JType::Error;
                }
                if !lt.is_numeric() || !rt.is_numeric() {
                    self.expr(lhs);
                    self.expr(rhs);
                    self.error(
                        span,
                        format!(
                            "operator '{}' cannot be applied to {} and {}",
                            arithmetic_symbol(op),
                            lt.describe(),
                            rt.describe()
                        ),
                    );
                    return JType::Error;
                }
                let target = promote(lt, rt);
                let actual_l = self.expr(lhs);
                self.numeric_conversion(actual_l, target);
                let actual_r = self.expr(rhs);
                self.numeric_conversion(actual_r, target);
                self.arithmetic_op(op, target);
                target
            }
        }
    }

    fn logical(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: SourceSpan) -> JType {
        let lt = self.expr(lhs);
        if lt != JType::Boolean && lt != JType::Error {
            self.error(
                span,
                format!(
                    "operator '{}' needs boolean operands, got {}",
                    if op == BinaryOp::And { "&&" } else { "||" },
                    lt.describe()
                ),
            );
        }
        let short = self.code.new_label();
        let end = self.code.new_label();
        let (jump, shortcut_value) = if op == BinaryOp::And {
            (op::IFEQ, op::ICONST_0)
        } else {
            (op::IFNE, op::ICONST_1)
        };
        self.code.branch(jump, short, 1);
        let rt = self.expr(rhs);
        if rt != JType::Boolean && rt != JType::Error {
            self.error(
                span,
                format!(
                    "operator '{}' needs boolean operands, got {}",
                    if op == BinaryOp::And { "&&" } else { "||" },
                    rt.describe()
                ),
            );
        }
        self.code.branch(op::GOTO, end, 0);
        self.code.bind(short);
        self.code.push_op(shortcut_value, 1);
        self.code.bind(end);
        JType::Boolean
    }

    fn comparison(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: SourceSpan) -> JType {
        let (lt, rt) = (self.type_of(lhs), self.type_of(rhs));
        if lt == JType::Error || rt == JType::Error {
            self.expr(lhs);
            self.expr(rhs);
            return JType::Error;
        }

        let is_equality = matches!(op, BinaryOp::Eq | BinaryOp::Ne);
        let both_boolean = lt == JType::Boolean && rt == JType::Boolean;
        let both_refs = lt.is_reference() && rt.is_reference();
        let both_numeric = lt.is_numeric() && rt.is_numeric();

        if !(both_numeric || (is_equality && (both_boolean || both_refs))) {
            self.expr(lhs);
            self.expr(rhs);
            self.error(
                span,
                format!(
                    "operator '{}' cannot be applied to {} and {}",
                    comparison_symbol(op),
                    lt.describe(),
                    rt.describe()
                ),
            );
            return JType::Error;
        }

        if both_numeric && promote(lt, rt) == JType::Double {
            let actual_l = self.expr(lhs);
            self.numeric_conversion(actual_l, JType::Double);
            let actual_r = self.expr(rhs);
            self.numeric_conversion(actual_r, JType::Double);
            // NaN handling (JVMS §6.5 dcmp<op>): pick the comparison
            // whose NaN result makes the branch fall through to false.
            let (cmp, jump) = match op {
                BinaryOp::Lt => (op::DCMPG, op::IFLT),
                BinaryOp::Le => (op::DCMPG, op::IFLE),
                BinaryOp::Gt => (op::DCMPL, op::IFGT),
                BinaryOp::Ge => (op::DCMPL, op::IFGE),
                BinaryOp::Eq => (op::DCMPL, op::IFEQ),
                BinaryOp::Ne => (op::DCMPL, op::IFNE),
                _ => unreachable!(),
            };
            self.code.push_op(cmp, 0);
            self.code.drop_stack(3); // two doubles -> one int
            self.boolean_from_branch(jump);
        } else if both_refs {
            self.expr(lhs);
            self.expr(rhs);
            let jump = if op == BinaryOp::Eq {
                op::IF_ACMPEQ
            } else {
                op::IF_ACMPNE
            };
            self.boolean_from_branch_binary(jump);
        } else {
            // int/char/boolean comparison as 32-bit ints.
            let target = if both_numeric {
                JType::Int
            } else {
                JType::Boolean
            };
            let actual_l = self.expr(lhs);
            if both_numeric {
                self.numeric_conversion(actual_l, target);
            }
            let actual_r = self.expr(rhs);
            if both_numeric {
                self.numeric_conversion(actual_r, target);
            }
            let jump = match op {
                BinaryOp::Eq => op::IF_ICMPEQ,
                BinaryOp::Ne => op::IF_ICMPNE,
                BinaryOp::Lt => op::IF_ICMPLT,
                BinaryOp::Le => op::IF_ICMPLE,
                BinaryOp::Gt => op::IF_ICMPGT,
                BinaryOp::Ge => op::IF_ICMPGE,
                _ => unreachable!(),
            };
            self.boolean_from_branch_binary(jump);
        }
        JType::Boolean
    }

    /// `<binary-jump>` variant: pops two operands.
    fn boolean_from_branch_binary(&mut self, jump: u8) {
        let if_true = self.code.new_label();
        let end = self.code.new_label();
        self.code.branch(jump, if_true, 2);
        self.code.push_op(op::ICONST_0, 1);
        self.code.branch(op::GOTO, end, 0);
        self.code.bind(if_true);
        self.code.push_op(op::ICONST_1, 1);
        self.code.bind(end);
    }

    /// `<unary-jump>` variant: pops the int left by a `dcmp` opcode.
    fn boolean_from_branch(&mut self, jump: u8) {
        let if_true = self.code.new_label();
        let end = self.code.new_label();
        self.code.branch(jump, if_true, 1);
        self.code.push_op(op::ICONST_0, 1);
        self.code.branch(op::GOTO, end, 0);
        self.code.bind(if_true);
        self.code.push_op(op::ICONST_1, 1);
        self.code.bind(end);
    }

    // ----- string concatenation (javac-8-style StringBuilder chain) -----

    /// Emit `new StringBuilder(); dup; <init>` — the start of a concat
    /// chain. If `first` is already on the stack (e.g. `s += x`), use
    /// [`Self::begin_concat_with_value_on_stack`] instead.
    fn begin_concat(&mut self) {
        let builder_class = intern_class(self.pool, "java/lang/StringBuilder");
        self.code.push_op_u16(op::NEW, builder_class, 1);
        self.code.push_op(op::DUP, 1);
        let init = intern_method_ref(self.pool, "java/lang/StringBuilder", "<init>", "()V");
        self.code.push_op_u16(op::INVOKESPECIAL, init, 0);
        self.code.drop_stack(1);
    }

    /// Start a concat chain when the first part's value is already on
    /// the stack: builder is created underneath via swap-free reorder
    /// (value, new builder) → append.
    fn begin_concat_with_value_on_stack(&mut self, value_ty: JType) {
        // Stack: [value]. Emit builder, then append the value by
        // swapping via the append call itself is not possible — so
        // build (builder, value) order with a DUP_X1-free approach:
        // allocate builder, swap. SWAP only works on category-1 values,
        // which all our reference/int values are; the only category-2
        // value is double, which this path never sees (Str only).
        debug_assert_ne!(value_ty, JType::Double);
        self.begin_concat();
        self.code.push_op(op::SWAP, 0);
        self.append_part(
            value_ty,
            SourceSpan {
                start: crate::diagnostics::SourcePosition { line: 0, column: 0 },
                end: crate::diagnostics::SourcePosition { line: 0, column: 0 },
            },
        );
    }

    /// Append the value on top of the stack (above the builder).
    fn append_part(&mut self, ty: JType, span: SourceSpan) {
        let descriptor = match ty {
            JType::Int => "(I)Ljava/lang/StringBuilder;",
            JType::Double => "(D)Ljava/lang/StringBuilder;",
            JType::Boolean => "(Z)Ljava/lang/StringBuilder;",
            JType::Char => "(C)Ljava/lang/StringBuilder;",
            JType::Str | JType::Null => "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
            JType::Array { .. } => {
                self.error(
                    span,
                    "concatenating an array directly is not supported by jvmjs \
                     (append its elements in a loop)",
                );
                return;
            }
            JType::Unsupported => {
                self.error(span, "this value's type is not yet supported by jvmjs");
                return;
            }
            JType::Error => return,
        };
        let append = intern_method_ref(self.pool, "java/lang/StringBuilder", "append", descriptor);
        self.code.push_op_u16(op::INVOKEVIRTUAL, append, 0);
        self.code.drop_stack(ty.width());
    }

    fn finish_concat(&mut self) {
        let to_string = intern_method_ref(
            self.pool,
            "java/lang/StringBuilder",
            "toString",
            "()Ljava/lang/String;",
        );
        self.code.push_op_u16(op::INVOKEVIRTUAL, to_string, 0);
    }

    fn concat(&mut self, lhs: &Expr, rhs: &Expr) -> JType {
        let mut parts = Vec::new();
        self.flatten_concat(lhs, &mut parts);
        self.flatten_concat(rhs, &mut parts);

        self.begin_concat();
        for part in parts {
            let ty = self.expr(part);
            self.append_part(ty, part.span());
        }
        self.finish_concat();
        JType::Str
    }

    /// Flatten nested string-typed `+` nodes into one builder chain.
    /// `1 + 2 + "a"` keeps `1 + 2` intact (its type is int); only
    /// string-typed additions flatten.
    fn flatten_concat<'e>(&mut self, expr: &'e Expr, out: &mut Vec<&'e Expr>) {
        if let Expr::Binary {
            op: BinaryOp::Add,
            lhs,
            rhs,
            ..
        } = expr
            && self.type_of(expr) == JType::Str
        {
            self.flatten_concat(lhs, out);
            self.flatten_concat(rhs, out);
        } else {
            out.push(expr);
        }
    }

    // ----- emission helpers -----

    fn push_int(&mut self, value: i32) {
        match value {
            -1..=5 => {
                let opcode = match value {
                    -1 => op::ICONST_M1,
                    0 => op::ICONST_0,
                    1 => op::ICONST_1,
                    2 => op::ICONST_2,
                    3 => op::ICONST_3,
                    4 => op::ICONST_4,
                    _ => op::ICONST_5,
                };
                self.code.push_op(opcode, 1);
            }
            -128..=127 => {
                self.code.push_op(op::BIPUSH, 1);
                self.code.bytes.push((value as i8).cast_unsigned());
            }
            -32768..=32767 => {
                self.code.push_op(op::SIPUSH, 1);
                self.code
                    .bytes
                    .extend_from_slice(&(value as i16).to_be_bytes());
            }
            _ => {
                let index = self.pool.intern(Constant::Integer(value));
                self.code.push_ldc(index);
            }
        }
    }

    /// Widening numeric conversion toward `target` (no-op when types
    /// already agree or aren't numeric).
    fn numeric_conversion(&mut self, from: JType, target: JType) {
        if target == JType::Double && matches!(from, JType::Int | JType::Char) {
            self.code.push_op(op::I2D, 1);
        }
    }

    /// Implicit narrowing after a compound assignment (JLS §15.26.2).
    fn narrow_back(&mut self, from: JType, to: JType) {
        if from == JType::Double && matches!(to, JType::Int | JType::Char) {
            self.code.push_op(op::D2I, 0);
            self.code.drop_stack(1);
        }
        if to == JType::Char {
            self.code.push_op(op::I2C, 0);
        }
    }

    fn arithmetic_op(&mut self, operator: BinaryOp, ty: JType) {
        let opcode = match (operator, ty) {
            (BinaryOp::Add, JType::Double) => op::DADD,
            (BinaryOp::Sub, JType::Double) => op::DSUB,
            (BinaryOp::Mul, JType::Double) => op::DMUL,
            (BinaryOp::Div, JType::Double) => op::DDIV,
            (BinaryOp::Rem, JType::Double) => op::DREM,
            (BinaryOp::Add, _) => op::IADD,
            (BinaryOp::Sub, _) => op::ISUB,
            (BinaryOp::Mul, _) => op::IMUL,
            (BinaryOp::Div, _) => op::IDIV,
            (BinaryOp::Rem, _) => op::IREM,
            _ => unreachable!("not an arithmetic operator"),
        };
        self.code.push_op(opcode, 0);
        self.code.drop_stack(ty.width());
    }

    fn emit_load(&mut self, slot: u16, ty: JType) {
        let (base, short_base) = match ty {
            JType::Double => (op::DLOAD, op::DLOAD_0),
            JType::Str | JType::Null | JType::Array { .. } => (op::ALOAD, op::ALOAD_0),
            _ => (op::ILOAD, op::ILOAD_0),
        };
        self.local_op(base, short_base, slot);
        self.code.grow_stack(ty.width());
    }

    fn emit_store(&mut self, slot: u16, ty: JType) {
        let (base, short_base) = match ty {
            JType::Double => (op::DSTORE, op::DSTORE_0),
            JType::Str | JType::Null | JType::Array { .. } => (op::ASTORE, op::ASTORE_0),
            _ => (op::ISTORE, op::ISTORE_0),
        };
        self.local_op(base, short_base, slot);
        self.code.drop_stack(ty.width());
    }

    fn local_op(&mut self, base: u8, short_base: u8, slot: u16) {
        if slot <= 3 {
            self.code
                .bytes
                .push(short_base + u8::try_from(slot).expect("slot <= 3"));
        } else if let Ok(narrow) = u8::try_from(slot) {
            self.code.bytes.push(base);
            self.code.bytes.push(narrow);
        } else {
            // 256+ locals needs the `wide` prefix; nobody's CSA program
            // gets there, so report instead of emitting bad code.
            self.diagnostics.push(Diagnostic {
                severity: crate::diagnostics::Severity::Error,
                message: String::from("too many local variables in one method"),
                path: self.path.to_owned(),
                span: None,
            });
        }
    }

    /// Convert (or reject) a value being assigned into a variable of
    /// type `to` (JLS §5.2 assignment contexts, minus boxing).
    fn convert_for_assignment(&mut self, from: JType, to: JType, span: SourceSpan) {
        match (from, to) {
            // Identity, char-to-int widening, and null-to-String all
            // need no code.
            (JType::Error, _) | (JType::Char, JType::Int) | (JType::Null, JType::Str) => {}
            (f, t) if f == t => {}
            (JType::Int | JType::Char, JType::Double) => self.code.push_op(op::I2D, 1),
            (JType::Double, JType::Int | JType::Char) => {
                self.error(
                    span,
                    format!(
                        "possible lossy conversion from double to {}; add an explicit cast",
                        to.describe()
                    ),
                );
            }
            (JType::Int, JType::Char) => {
                self.error(
                    span,
                    "possible lossy conversion from int to char; add a cast",
                );
            }
            (from, to) => {
                self.error(
                    span,
                    format!(
                        "incompatible types: {} cannot be converted to {}",
                        from.describe(),
                        to.describe()
                    ),
                );
            }
        }
    }
}

fn compound_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+=",
        BinaryOp::Sub => "-=",
        BinaryOp::Mul => "*=",
        BinaryOp::Div => "/=",
        BinaryOp::Rem => "%=",
        _ => "?=",
    }
}

fn arithmetic_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
        _ => "?",
    }
}

fn comparison_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        _ => "?",
    }
}

/// Operand-stack width of the single argument in a `(X)V` descriptor.
fn descriptor_arg_width(descriptor: &str) -> u16 {
    match descriptor {
        "()V" => 0,
        "(D)V" => 2,
        _ => 1,
    }
}

fn intern_class(pool: &mut ConstantPool, binary_name: &str) -> CpIndex {
    let name_index = pool.intern_utf8(binary_name);
    pool.intern(Constant::Class { name_index })
}

fn intern_name_and_type(pool: &mut ConstantPool, name: &str, descriptor: &str) -> CpIndex {
    let name_index = pool.intern_utf8(name);
    let descriptor_index = pool.intern_utf8(descriptor);
    pool.intern(Constant::NameAndType {
        name_index,
        descriptor_index,
    })
}

fn intern_field_ref(pool: &mut ConstantPool, class: &str, name: &str, descriptor: &str) -> CpIndex {
    let class_index = intern_class(pool, class);
    let name_and_type_index = intern_name_and_type(pool, name, descriptor);
    pool.intern(Constant::FieldRef {
        class_index,
        name_and_type_index,
    })
}

fn intern_method_ref(
    pool: &mut ConstantPool,
    class: &str,
    name: &str,
    descriptor: &str,
) -> CpIndex {
    let class_index = intern_class(pool, class);
    let name_and_type_index = intern_name_and_type(pool, name, descriptor);
    pool.intern(Constant::MethodRef {
        class_index,
        name_and_type_index,
    })
}

/// A code-attribute label, resolved when the builder finishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Label(usize);

/// Accumulates bytecode with label patching and running stack-depth
/// tracking.
///
/// Depth tracking is linear (it doesn't model control flow), so
/// `max_stack` can overestimate around branch diamonds. That is safe:
/// our VM sizes its stack dynamically and never reads `max_stack`, and
/// no external verifier consumes our output (`specs/RUNTIME.md`).
struct CodeBuilder {
    bytes: Vec<u8>,
    depth: u16,
    max_stack: u16,
    labels: Vec<Option<usize>>,
    patches: Vec<(usize, usize, Label)>,
}

impl CodeBuilder {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            depth: 0,
            max_stack: 0,
            labels: Vec::new(),
            patches: Vec::new(),
        }
    }

    fn new_label(&mut self) -> Label {
        self.labels.push(None);
        Label(self.labels.len() - 1)
    }

    fn bind(&mut self, label: Label) {
        self.labels[label.0] = Some(self.bytes.len());
    }

    /// Emit a branch instruction with a to-be-patched 16-bit offset.
    fn branch(&mut self, opcode: u8, label: Label, pops: u16) {
        let opcode_at = self.bytes.len();
        self.bytes.push(opcode);
        let patch_at = self.bytes.len();
        self.bytes.extend_from_slice(&[0, 0]);
        self.patches.push((patch_at, opcode_at, label));
        self.drop_stack(pops);
    }

    fn push_op(&mut self, opcode: u8, pushes: u16) {
        self.bytes.push(opcode);
        self.grow_stack(pushes);
    }

    fn push_op_u16(&mut self, opcode: u8, operand: u16, pushes: u16) {
        self.bytes.push(opcode);
        self.bytes.extend_from_slice(&operand.to_be_bytes());
        self.grow_stack(pushes);
    }

    /// `ldc` with automatic widening to `ldc_w` for large pool indices.
    fn push_ldc(&mut self, index: CpIndex) {
        if let Ok(narrow) = u8::try_from(index) {
            self.bytes.push(op::LDC);
            self.bytes.push(narrow);
        } else {
            self.bytes.push(op::LDC_W);
            self.bytes.extend_from_slice(&index.to_be_bytes());
        }
        self.grow_stack(1);
    }

    fn grow_stack(&mut self, pushes: u16) {
        self.depth += pushes;
        self.max_stack = self.max_stack.max(self.depth);
    }

    fn drop_stack(&mut self, pops: u16) {
        self.depth = self.depth.saturating_sub(pops);
    }

    /// Note a value left on the stack that won't be consumed (error
    /// recovery paths).
    fn discard(&mut self) {
        self.drop_stack(1);
    }

    /// Resolve all label patches and return the final bytecode.
    fn finish(mut self) -> (Vec<u8>, u16) {
        for (patch_at, opcode_at, label) in &self.patches {
            let target = self.labels[label.0].expect("branch to unbound label");
            let offset = i32::try_from(target).expect("code too large")
                - i32::try_from(*opcode_at).expect("code too large");
            let offset = i16::try_from(offset).expect("branch offset exceeds 16 bits");
            self.bytes[*patch_at..*patch_at + 2].copy_from_slice(&offset.to_be_bytes());
        }
        (self.bytes, self.max_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;
    use jvmjs_classfile::read_code_attribute;

    fn generate_ok(source: &str) -> Vec<CompiledClass> {
        let (tokens, lex_errors) = lex("Test.java", source);
        assert!(lex_errors.is_empty(), "{lex_errors:?}");
        let (unit, parse_errors) = parse("Test.java", tokens);
        assert!(parse_errors.is_empty(), "{parse_errors:?}");
        let (classes, errors) = generate(&[(String::from("Test.java"), unit)]);
        assert!(errors.is_empty(), "{errors:?}");
        classes
    }

    fn generate_errors(source: &str) -> Vec<Diagnostic> {
        let (tokens, _) = lex("Test.java", source);
        let (unit, parse_errors) = parse("Test.java", tokens);
        assert!(parse_errors.is_empty(), "{parse_errors:?}");
        let (_, errors) = generate(&[(String::from("Test.java"), unit)]);
        errors
    }

    #[test]
    fn hello_world_produces_expected_bytecode() {
        let classes = generate_ok(
            r#"
            public class Main {
                public static void main(String[] args) {
                    System.out.println("Hello, World!");
                }
            }
            "#,
        );
        let class = &classes[0].class_file;
        let pool = &class.constant_pool;
        let main = &class.methods[0];
        assert_eq!(pool.get_utf8(main.name_index), Some("main"));
        assert_eq!(
            pool.get_utf8(main.descriptor_index),
            Some("([Ljava/lang/String;)V")
        );

        let code = read_code_attribute(&main.attributes[0].info).expect("valid Code attribute");
        assert_eq!(code.max_locals, 1);
        assert_eq!(code.code[0], op::GETSTATIC);
        assert_eq!(code.code[3], op::LDC);
        assert_eq!(code.code[5], op::INVOKEVIRTUAL);
        assert_eq!(code.code[8], op::RETURN);
    }

    #[test]
    fn locals_get_slots_and_doubles_are_wide() {
        let classes = generate_ok(
            r"
            class L {
                static void f() {
                    int a = 1;
                    double d = 2.5;
                    int b = a;
                    System.out.println(d);
                    System.out.println(b);
                }
            }
            ",
        );
        let method = &classes[0].class_file.methods[0];
        let code = read_code_attribute(&method.attributes[0].info).unwrap();
        // a=slot0, d=slots1-2, b=slot3.
        assert_eq!(code.max_locals, 4);
        assert_eq!(code.code[0], op::ICONST_1);
        assert_eq!(code.code[1], op::ISTORE_0);
    }

    #[test]
    fn branches_resolve_to_valid_offsets() {
        let classes = generate_ok(
            r"
            class B {
                static void f() {
                    boolean x = 1 < 2 && 3 != 4;
                    System.out.println(!x);
                }
            }
            ",
        );
        let method = &classes[0].class_file.methods[0];
        let code = read_code_attribute(&method.attributes[0].info).unwrap();
        // Every branch target must land inside the method.
        let mut pc = 0usize;
        while pc < code.code.len() {
            let opcode = code.code[pc];
            match opcode {
                _ if (op::IFEQ..=op::GOTO).contains(&opcode) => {
                    let offset = i16::from_be_bytes([code.code[pc + 1], code.code[pc + 2]]);
                    let target = i64::try_from(pc).unwrap() + i64::from(offset);
                    let in_range = usize::try_from(target).is_ok_and(|t| t < code.code.len());
                    assert!(in_range, "branch at {pc} to out-of-range {target}");
                    pc += 3;
                }
                _ if opcode == op::BIPUSH
                    || opcode == op::LDC
                    || (op::ILOAD..=op::ALOAD).contains(&opcode)
                    || (op::ISTORE..=op::ASTORE).contains(&opcode) =>
                {
                    pc += 2;
                }
                _ if opcode == op::SIPUSH
                    || opcode == op::LDC_W
                    || opcode == op::LDC2_W
                    || opcode == op::GETSTATIC
                    || opcode == op::INVOKEVIRTUAL
                    || opcode == op::INVOKESPECIAL
                    || opcode == op::NEW =>
                {
                    pc += 3;
                }
                _ => pc += 1,
            }
        }
    }

    #[test]
    fn string_concat_uses_stringbuilder() {
        let classes = generate_ok(
            r#"
            class C {
                static void f() {
                    int x = 7;
                    System.out.println("x = " + x + "!");
                }
            }
            "#,
        );
        let class = &classes[0].class_file;
        let utf8: Vec<&str> = class
            .constant_pool
            .iter_slots()
            .filter_map(|c| match c {
                Constant::Utf8(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(utf8.contains(&"java/lang/StringBuilder"));
        assert!(utf8.contains(&"(I)Ljava/lang/StringBuilder;"));
        assert!(utf8.contains(&"toString"));
    }

    #[test]
    fn type_errors_are_reported() {
        let cases: &[(&str, &str)] = &[
            ("int x = 2.5;", "possible lossy conversion"),
            ("int x = true;", "incompatible types"),
            ("boolean b = 1;", "incompatible types"),
            (r#"int y = 1; String s = "a" - y;"#, "cannot be applied"),
            ("boolean b = true; int c = b + 1;", "cannot be applied"),
            ("int z = q;", "cannot find variable"),
            (
                "int a = 1; boolean c = a && true;",
                "needs boolean operands",
            ),
        ];
        for (body, expected) in cases {
            let errors = generate_errors(&format!("class T {{ static void f() {{ {body} }} }}"));
            assert!(
                errors.iter().any(|e| e.message.contains(expected)),
                "case '{body}': expected '{expected}' in {errors:?}"
            );
        }
    }

    #[test]
    fn definite_assignment_is_checked_linearly() {
        let errors =
            generate_errors("class T { static void f() { int x; System.out.println(x); } }");
        assert!(
            errors[0]
                .message
                .contains("might not have been initialized")
        );

        // Declaring then assigning before use is fine.
        generate_ok("class T { static void f() { int x; x = 3; System.out.println(x); } }");
    }

    #[test]
    fn final_variables_reject_reassignment() {
        let errors = generate_errors("class T { static void f() { final int x = 1; x = 2; } }");
        assert!(
            errors[0]
                .message
                .contains("cannot assign to final variable")
        );
    }

    #[test]
    fn duplicate_locals_are_rejected_across_scopes() {
        let errors = generate_errors("class T { static void f() { int x = 1; { int x = 2; } } }");
        assert!(errors[0].message.contains("already defined"));
    }
}
