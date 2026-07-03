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
    BinaryOp, ClassDecl, CompilationUnit, Expr, Literal, LocalDeclarator, MethodDecl, Stmt,
    TypeRef, UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourceSpan};

/// Generate class files for every class in the unit. Diagnostics and
/// classes are both returned; callers treat any error diagnostic as
/// failing the compilation.
#[must_use]
pub fn generate(path: &str, unit: &CompilationUnit) -> (Vec<CompiledClass>, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let classes = unit
        .classes
        .iter()
        .map(|class| CompiledClass {
            binary_name: class.name.clone(),
            class_file: emit_class(path, &mut diagnostics, class),
        })
        .collect();
    (classes, diagnostics)
}

fn emit_class(path: &str, diagnostics: &mut Vec<Diagnostic>, decl: &ClassDecl) -> ClassFile {
    let mut class = ClassFile::new_java11();
    class.this_class = intern_class(&mut class.constant_pool, &decl.name);
    class.super_class = intern_class(&mut class.constant_pool, "java/lang/Object");

    for method in &decl.methods {
        let compiled = emit_method(path, diagnostics, &mut class.constant_pool, method);
        class.methods.push(compiled);
    }
    class
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
    /// A type jvmjs doesn't handle yet (e.g. `String[] args`).
    Unsupported,
    /// A diagnostic was already reported for this subtree.
    Error,
}

impl JType {
    fn describe(self) -> &'static str {
        match self {
            JType::Int => "int",
            JType::Double => "double",
            JType::Boolean => "boolean",
            JType::Char => "char",
            JType::Str => "String",
            JType::Null => "null",
            JType::Unsupported => "an unsupported type",
            JType::Error => "an unknown type",
        }
    }

    fn is_numeric(self) -> bool {
        matches!(self, JType::Int | JType::Double | JType::Char)
    }

    /// Width in operand-stack slots (JVMS §2.6.2).
    fn width(self) -> u16 {
        if self == JType::Double { 2 } else { 1 }
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
    pool: &mut ConstantPool,
    decl: &MethodDecl,
) -> MethodInfo {
    let mut body = BodyGen {
        path,
        diagnostics,
        pool,
        code: CodeBuilder::new(),
        scopes: vec![Vec::new()],
        next_slot: u16::from(!decl.is_static),
    };

    if decl.return_type != TypeRef::Void {
        body.error(
            decl.span,
            "methods that return a value are not yet supported by jvmjs",
        );
    }

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

/// Per-method-body emission state.
struct BodyGen<'a> {
    path: &'a str,
    diagnostics: &'a mut Vec<Diagnostic>,
    pool: &'a mut ConstantPool,
    code: CodeBuilder,
    /// Lexical scopes (innermost last); Java forbids shadowing locals,
    /// so declaration checks search all of them.
    scopes: Vec<Vec<(String, LocalVar)>>,
    next_slot: u16,
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
                name,
                op,
                value,
                span,
            } => {
                self.assign(name, *op, value, *span);
            }
            Stmt::Expr(expr) => self.expression_statement(expr),
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
                let init_ty = self.expr(init);
                self.convert_for_assignment(init_ty, var_ty, init.span());
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

        let stream_field = receiver.as_deref().and_then(|r| match r {
            Expr::Name { path, .. } => {
                match path.iter().map(String::as_str).collect::<Vec<_>>()[..] {
                    ["System", "out"] => Some("out"),
                    ["System", "err"] => Some("err"),
                    _ => None,
                }
            }
            _ => None,
        });
        let Some(stream_field) = stream_field else {
            self.error(
                *span,
                "only calls on System.out and System.err are supported by jvmjs so far",
            );
            return;
        };
        if method != "println" && method != "print" {
            self.error(
                *span,
                format!("PrintStream.{method} is not supported by jvmjs (try print or println)"),
            );
            return;
        }

        let field = intern_field_ref(
            self.pool,
            "java/lang/System",
            stream_field,
            "Ljava/io/PrintStream;",
        );
        self.code.push_op_u16(op::GETSTATIC, field, 1);

        let arg_descriptor = match &args[..] {
            [] => {
                if method == "print" {
                    self.error(*span, "print() requires an argument (println() does not)");
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
                self.error(*span, "print/println take at most one argument");
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
            Expr::Name { .. } | Expr::Call { .. } => JType::Error,
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
            Expr::Call { span, .. } => {
                self.error(
                    *span,
                    "method calls that return a value are not yet supported by jvmjs",
                );
                JType::Error
            }
            Expr::Unary { op, operand, span } => self.unary(*op, operand, *span),
            Expr::Cast { ty, operand, span } => self.cast(ty, operand, *span),
            Expr::Binary { op, lhs, rhs, span } => self.binary(*op, lhs, rhs, *span),
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
        let both_refs =
            matches!(lt, JType::Str | JType::Null) && matches!(rt, JType::Str | JType::Null);
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
            JType::Str | JType::Null => (op::ALOAD, op::ALOAD_0),
            _ => (op::ILOAD, op::ILOAD_0),
        };
        self.local_op(base, short_base, slot);
        self.code.grow_stack(ty.width());
    }

    fn emit_store(&mut self, slot: u16, ty: JType) {
        let (base, short_base) = match ty {
            JType::Double => (op::DSTORE, op::DSTORE_0),
            JType::Str | JType::Null => (op::ASTORE, op::ASTORE_0),
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
        let (classes, errors) = generate("Test.java", &unit);
        assert!(errors.is_empty(), "{errors:?}");
        classes
    }

    fn generate_errors(source: &str) -> Vec<Diagnostic> {
        let (tokens, _) = lex("Test.java", source);
        let (unit, parse_errors) = parse("Test.java", tokens);
        assert!(parse_errors.is_empty(), "{parse_errors:?}");
        let (_, errors) = generate("Test.java", &unit);
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
