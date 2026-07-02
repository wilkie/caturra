//! Bytecode generation: AST → [`ClassFile`].
//!
//! Scope matches `specs/LANGUAGE.md` v0: methods whose bodies are
//! `System.out` / `System.err` `print`/`println` calls with literal
//! arguments. Anything else produces a friendly diagnostic and the
//! class is not emitted.

use jvmjs_classfile::opcodes as op;
use jvmjs_classfile::{
    AttributeInfo, CODE_ATTRIBUTE, ClassFile, CodeAttribute, Constant, ConstantPool, CpIndex,
    MethodAccessFlags, MethodInfo, write_code_attribute,
};

use crate::CompiledClass;
use crate::ast::{ClassDecl, CompilationUnit, Expr, Literal, MethodDecl, Stmt, TypeRef};
use crate::diagnostics::{Diagnostic, SourceSpan};

/// Generate class files for every class in the unit. Diagnostics and
/// classes are both returned; callers treat any error diagnostic as
/// failing the compilation.
#[must_use]
pub fn generate(path: &str, unit: &CompilationUnit) -> (Vec<CompiledClass>, Vec<Diagnostic>) {
    let mut generator = Generator {
        path,
        diagnostics: Vec::new(),
    };
    let classes = unit
        .classes
        .iter()
        .map(|class| CompiledClass {
            binary_name: class.name.clone(),
            class_file: generator.class(class),
        })
        .collect();
    (classes, generator.diagnostics)
}

struct Generator<'a> {
    path: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl Generator<'_> {
    fn error(&mut self, span: SourceSpan, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(self.path, message, span));
    }

    fn class(&mut self, decl: &ClassDecl) -> ClassFile {
        let mut class = ClassFile::new_java11();
        class.this_class = intern_class(&mut class.constant_pool, &decl.name);
        class.super_class = intern_class(&mut class.constant_pool, "java/lang/Object");

        for method in &decl.methods {
            let compiled = self.method(&mut class.constant_pool, method);
            class.methods.push(compiled);
        }
        class
    }

    fn method(&mut self, pool: &mut ConstantPool, decl: &MethodDecl) -> MethodInfo {
        if decl.return_type != TypeRef::Void {
            self.error(
                decl.span,
                "methods that return a value are not yet supported by jvmjs",
            );
        }

        let descriptor = self.descriptor(decl);
        let mut flags = 0;
        if decl.is_public {
            flags |= MethodAccessFlags::PUBLIC;
        }
        if decl.is_static {
            flags |= MethodAccessFlags::STATIC;
        }

        let mut code = CodeBuilder::new();
        for stmt in &decl.body {
            self.statement(pool, &mut code, stmt);
        }
        code.push_op(op::RETURN, 0);

        // Static: params only. Instance: `this` occupies local 0.
        let param_slots: u16 =
            decl.params.iter().map(|p| slot_width(&p.ty)).sum::<u16>() + u16::from(!decl.is_static);

        let code_attribute = CodeAttribute {
            max_stack: code.max_stack,
            max_locals: param_slots,
            code: code.bytes,
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

    fn descriptor(&mut self, decl: &MethodDecl) -> String {
        let mut descriptor = String::from("(");
        for param in &decl.params {
            self.push_type_descriptor(&mut descriptor, &param.ty, decl.span);
        }
        descriptor.push(')');
        self.push_type_descriptor(&mut descriptor, &decl.return_type, decl.span);
        descriptor
    }

    fn push_type_descriptor(&mut self, out: &mut String, ty: &TypeRef, span: SourceSpan) {
        match ty {
            TypeRef::Void => out.push('V'),
            TypeRef::Int => out.push('I'),
            TypeRef::Double => out.push('D'),
            TypeRef::Boolean => out.push('Z'),
            TypeRef::Char => out.push('C'),
            TypeRef::Array(inner) => {
                out.push('[');
                self.push_type_descriptor(out, inner, span);
            }
            TypeRef::Named(name) => {
                if name == "String" {
                    out.push_str("Ljava/lang/String;");
                } else {
                    // TODO(classlib): resolve simple names against the
                    // compilation unit and the class library.
                    self.error(span, format!("unknown type '{name}'"));
                    out.push_str("Ljava/lang/Object;");
                }
            }
        }
    }

    fn statement(&mut self, pool: &mut ConstantPool, code: &mut CodeBuilder, stmt: &Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                for inner in statements {
                    self.statement(pool, code, inner);
                }
            }
            Stmt::Expr(expr) => self.expression_statement(pool, code, expr),
        }
    }

    fn expression_statement(
        &mut self,
        pool: &mut ConstantPool,
        code: &mut CodeBuilder,
        expr: &Expr,
    ) {
        let Expr::Call {
            receiver,
            method,
            args,
            span,
        } = expr
        else {
            self.error(
                expr.span(),
                "only method-call statements (like System.out.println(...)) are supported \
                 by jvmjs so far",
            );
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

        // Push the PrintStream, then the argument, then invoke.
        let field = intern_field_ref(
            pool,
            "java/lang/System",
            stream_field,
            "Ljava/io/PrintStream;",
        );
        code.push_op_u16(op::GETSTATIC, field, 1);

        let arg_descriptor = match &args[..] {
            [] => {
                if method == "print" {
                    self.error(*span, "print() requires an argument (println() does not)");
                    return;
                }
                String::from("()V")
            }
            [Expr::Literal { value, span }] => {
                let Some(descriptor) = self.literal_argument(pool, code, value, *span) else {
                    return;
                };
                descriptor
            }
            [other] => {
                self.error(
                    other.span(),
                    "only literal arguments to print/println are supported by jvmjs so far",
                );
                return;
            }
            _ => {
                self.error(*span, "print/println take at most one argument");
                return;
            }
        };

        let method_ref = intern_method_ref(pool, "java/io/PrintStream", method, &arg_descriptor);
        // invokevirtual pops receiver + args; net stack effect handled by
        // the builder's running depth.
        code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 0);
        code.drop_stack(1 + descriptor_arg_width(&arg_descriptor));
    }

    /// Emit code pushing one literal; returns the matching print/println
    /// descriptor, or `None` if a diagnostic was recorded.
    fn literal_argument(
        &mut self,
        pool: &mut ConstantPool,
        code: &mut CodeBuilder,
        literal: &Literal,
        span: SourceSpan,
    ) -> Option<String> {
        match literal {
            Literal::Int(value) => {
                let Ok(value) = i32::try_from(*value) else {
                    self.error(
                        span,
                        format!("integer literal {value} is out of range for int"),
                    );
                    return None;
                };
                push_int(pool, code, value);
                Some(String::from("(I)V"))
            }
            Literal::Char(value) => {
                let code_point = *value as u32;
                let Ok(value) = i32::try_from(code_point) else {
                    return None;
                };
                if code_point > 0xFFFF {
                    self.error(span, "character literal does not fit in a Java char");
                    return None;
                }
                push_int(pool, code, value);
                Some(String::from("(C)V"))
            }
            Literal::Bool(value) => {
                code.push_op(if *value { op::ICONST_1 } else { op::ICONST_0 }, 1);
                Some(String::from("(Z)V"))
            }
            Literal::Double(value) => {
                let index = pool.intern(Constant::Double(*value));
                code.push_op_u16(op::LDC2_W, index, 2);
                Some(String::from("(D)V"))
            }
            Literal::Str(value) => {
                let utf8 = pool.intern_utf8(value);
                let index = pool.intern(Constant::String { string_index: utf8 });
                code.push_ldc(index);
                Some(String::from("(Ljava/lang/String;)V"))
            }
            Literal::Null => {
                self.error(
                    span,
                    "println(null) is ambiguous in Java; pass a String or use \"null\"",
                );
                None
            }
        }
    }
}

/// Slot width of a type in the locals array (JVMS §2.6.1).
fn slot_width(ty: &TypeRef) -> u16 {
    match ty {
        TypeRef::Double => 2,
        _ => 1,
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

fn push_int(pool: &mut ConstantPool, code: &mut CodeBuilder, value: i32) {
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
            code.push_op(opcode, 1);
        }
        -128..=127 => {
            code.push_op(op::BIPUSH, 1);
            code.bytes.push((value as i8).cast_unsigned());
        }
        -32768..=32767 => {
            code.push_op(op::SIPUSH, 1);
            code.bytes.extend_from_slice(&(value as i16).to_be_bytes());
        }
        _ => {
            let index = pool.intern(Constant::Integer(value));
            code.push_ldc(index);
        }
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

/// Accumulates bytecode while tracking operand stack depth, so
/// `max_stack` is exact rather than guessed.
struct CodeBuilder {
    bytes: Vec<u8>,
    depth: u16,
    max_stack: u16,
}

impl CodeBuilder {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            depth: 0,
            max_stack: 0,
        }
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
        assert_eq!(classes.len(), 1);
        let class = &classes[0].class_file;
        assert_eq!(class.class_name(), Some("Main"));

        let pool = &class.constant_pool;
        let main = &class.methods[0];
        assert_eq!(pool.get_utf8(main.name_index), Some("main"));
        assert_eq!(
            pool.get_utf8(main.descriptor_index),
            Some("([Ljava/lang/String;)V")
        );
        assert!(main.access_flags.contains(MethodAccessFlags::PUBLIC));
        assert!(main.access_flags.contains(MethodAccessFlags::STATIC));

        let code = read_code_attribute(&main.attributes[0].info).expect("valid Code attribute");
        assert_eq!(code.max_stack, 2);
        assert_eq!(code.max_locals, 1);
        // getstatic #x, ldc #y, invokevirtual #z, return
        assert_eq!(code.code[0], op::GETSTATIC);
        assert_eq!(code.code[3], op::LDC);
        assert_eq!(code.code[5], op::INVOKEVIRTUAL);
        assert_eq!(code.code[8], op::RETURN);
    }

    #[test]
    fn println_overloads_pick_descriptors() {
        let classes = generate_ok(
            r#"
            class Demo {
                static void show() {
                    System.out.println(42);
                    System.out.println(3.5);
                    System.out.println(true);
                    System.out.println('x');
                    System.out.println();
                    System.out.print("no newline");
                }
            }
            "#,
        );
        let class = &classes[0].class_file;
        let pool = &class.constant_pool;
        let all_utf8: Vec<&str> = pool
            .iter_slots()
            .filter_map(|c| match c {
                Constant::Utf8(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        for descriptor in [
            "(I)V",
            "(D)V",
            "(Z)V",
            "(C)V",
            "()V",
            "(Ljava/lang/String;)V",
        ] {
            assert!(
                all_utf8.contains(&descriptor),
                "missing {descriptor} in {all_utf8:?}"
            );
        }
    }

    #[test]
    fn double_println_needs_two_stack_slots() {
        let classes = generate_ok(r"class D { static void f() { System.out.println(2.5); } }");
        let method = &classes[0].class_file.methods[0];
        let code = read_code_attribute(&method.attributes[0].info).unwrap();
        assert_eq!(code.max_stack, 3); // PrintStream ref + wide double
    }

    #[test]
    fn unsupported_calls_are_diagnosed() {
        let (tokens, _) = lex("Test.java", r"class A { static void f() { foo(); } }");
        let (unit, parse_errors) = parse("Test.java", tokens);
        assert!(parse_errors.is_empty());
        let (_, errors) = generate("Test.java", &unit);
        assert_eq!(errors.len(), 1);
        assert!(
            errors[0].message.contains("System.out"),
            "{}",
            errors[0].message
        );
    }
}
