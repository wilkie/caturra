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

use caturra_classfile::opcodes as op;
use caturra_classfile::{
    AttributeInfo, CODE_ATTRIBUTE, ClassFile, CodeAttribute, Constant, ConstantPool, CpIndex,
    MethodAccessFlags, MethodInfo, write_code_attribute,
};

use crate::CompiledClass;
use crate::ast::{
    AssignTarget, BinaryOp, CatchClause, ClassDecl, CompilationUnit, Expr, FieldDecl, InitBlock,
    Literal, LocalDeclarator, MethodDecl, Stmt, SwitchArm, TypeRef, UnaryOp,
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

#[allow(clippy::too_many_lines)] // one class-assembly sequence
fn emit_class(
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    table: &MethodTable,
    decl: &ClassDecl,
) -> ClassFile {
    let mut class = ClassFile::new_java11();
    class.this_class = intern_class(&mut class.constant_pool, &decl.name);
    // For an anonymous class the extends/implements split was resolved
    // by the table; use its class name (Object) and interface list.
    let anon_interfaces: Vec<String> = if decl.is_anonymous {
        table.anon_supertypes(&decl.name)
    } else {
        Vec::new()
    };
    let anon_super = if decl.is_anonymous {
        Some(table.anon_super_name(&decl.name))
    } else {
        None
    };
    let super_name = if let Some(anon) = &anon_super {
        anon.as_str()
    } else {
        decl.superclass
            .as_deref()
            .map_or("java/lang/Object", |name| {
                // `extends Exception` etc: the parent is a library throwable.
                caturra_classfile::exceptions::internal_name_of(name).unwrap_or(name)
            })
    };
    class.super_class = intern_class(&mut class.constant_pool, super_name);

    // SourceFile attribute: which compilation unit this class came
    // from — the debugger keys breakpoints by (file, line).
    let file_name = path.rsplit('/').next().unwrap_or(path);
    let file_index = class.constant_pool.intern_utf8(file_name);
    let source_file_name = class
        .constant_pool
        .intern_utf8(caturra_classfile::debug::SOURCE_FILE_ATTRIBUTE);
    class.attributes.push(AttributeInfo {
        name_index: source_file_name,
        info: caturra_classfile::debug::encode_source_file(file_index),
    });
    for interface in decl.interfaces.iter().chain(anon_interfaces.iter()) {
        let index = intern_class(&mut class.constant_pool, interface);
        class.interfaces.push(index);
    }
    {
        let mut flags = class.access_flags.0;
        if decl.is_abstract {
            flags |= caturra_classfile::ClassAccessFlags::ABSTRACT;
        }
        if decl.is_interface {
            flags |= caturra_classfile::ClassAccessFlags::INTERFACE;
            flags &= !caturra_classfile::ClassAccessFlags::SUPER;
        }
        class.access_flags = caturra_classfile::ClassAccessFlags(flags);
    }

    for field in &decl.fields {
        let ty = table.resolve_type(&field.ty).unwrap_or(JType::Unsupported);
        if ty == JType::Unsupported {
            diagnostics.push(Diagnostic::error(
                path,
                unresolved_type_message(&field.ty, table),
                field.span,
            ));
        }
        let mut flags = if field.is_private {
            caturra_classfile::FieldAccessFlags::PRIVATE
        } else {
            caturra_classfile::FieldAccessFlags::PUBLIC
        };
        if field.is_static {
            flags |= caturra_classfile::FieldAccessFlags::STATIC;
        }
        if field.is_final {
            flags |= caturra_classfile::FieldAccessFlags::FINAL;
        }
        let name_index = class.constant_pool.intern_utf8(&field.name);
        let descriptor_index = class.constant_pool.intern_utf8(&ty.descriptor(table));
        // A generic field (`ArrayList<Friend>`) carries a Signature attribute
        // so reflection (`Field.getGenericType()`) can recover its type args.
        let mut attributes = Vec::new();
        if let Some(signature) = generic_field_signature(&field.ty, table) {
            let name = class.constant_pool.intern_utf8("Signature");
            let sig_index = class.constant_pool.intern_utf8(&signature);
            attributes.push(AttributeInfo {
                name_index: name,
                info: sig_index.to_be_bytes().to_vec(),
            });
        }
        class.fields.push(caturra_classfile::FieldInfo {
            access_flags: caturra_classfile::FieldAccessFlags(flags),
            name_index,
            descriptor_index,
            attributes,
        });
    }

    for method in &decl.methods {
        let compiled = emit_method(
            path,
            diagnostics,
            table,
            decl,
            &mut class.constant_pool,
            method,
        );
        class.methods.push(compiled);
    }

    // Default constructor when none is declared (JLS §8.8.9).
    if !decl.is_interface && !decl.methods.iter().any(|m| m.is_constructor) {
        let default_ctor = MethodDecl {
            name: decl.name.clone(),
            is_static: false,
            is_public: true,
            is_private: false,
            is_constructor: true,
            is_abstract: false,
            type_params: Vec::new(),
            return_type: TypeRef::Void,
            params: Vec::new(),
            body: Vec::new(),
            annotations: Vec::new(),
            span: decl.span,
        };
        let compiled = emit_method(
            path,
            diagnostics,
            table,
            decl,
            &mut class.constant_pool,
            &default_ctor,
        );
        class.methods.push(compiled);
    }

    // Static field initializers and `static {}` blocks run in a
    // synthesized <clinit>, interleaved in source order.
    let has_static_init = decl.fields.iter().any(|f| f.is_static && f.init.is_some())
        || decl.init_blocks.iter().any(|b| b.is_static);
    if has_static_init {
        let compiled = emit_clinit(path, diagnostics, table, decl, &mut class.constant_pool);
        class.methods.push(compiled);
    }
    class
}

/// Synthesize `static {}`-style initialization for static fields.
fn emit_clinit(
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    table: &MethodTable,
    decl: &ClassDecl,
    pool: &mut ConstantPool,
) -> MethodInfo {
    let class_id = table.class_id(&decl.name).expect("class registered");
    let mut body = BodyGen {
        path,
        diagnostics,
        pool,
        table,
        current_class: &decl.name,
        current_class_id: class_id,
        in_static: true,
        in_constructor: false,
        return_type: None,
        code: CodeBuilder::new(),
        scopes: vec![Vec::new()],
        next_slot: 0,
        loop_stack: Vec::new(),
        pending_label: None,
        finally_stack: Vec::new(),
        local_var_debug: Vec::new(),
    };
    body.emit_ordered_initializers(decl, true);
    body.code.push_op(op::RETURN, 0);
    let max_locals = body.next_slot;
    let local_var_debug = std::mem::take(&mut body.local_var_debug);
    let (bytecode, max_stack, line_numbers, exception_table) = body.code.finish();
    finish_method_info(
        pool,
        "<clinit>",
        "()V",
        MethodAccessFlags::STATIC,
        max_stack,
        max_locals,
        bytecode,
        &line_numbers,
        &local_var_debug,
        exception_table,
    )
}

/// Assemble the final [`MethodInfo`] from emitted parts, including the
/// `LineNumberTable` and `LocalVariableTable` debug attributes nested
/// in the `Code` attribute.
#[allow(clippy::too_many_arguments)] // one assembly point
fn finish_method_info(
    pool: &mut ConstantPool,
    name: &str,
    descriptor: &str,
    flags: u16,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    line_numbers: &[(u16, u16)],
    local_var_debug: &[(String, String, Option<String>, u16, u16)],
    exception_table: Vec<caturra_classfile::ExceptionTableEntry>,
) -> MethodInfo {
    let code_len = u16::try_from(code.len()).unwrap_or(u16::MAX);
    let mut code_attributes = Vec::new();
    if !line_numbers.is_empty() {
        let name_index = pool.intern_utf8(caturra_classfile::debug::LINE_NUMBER_TABLE_ATTRIBUTE);
        code_attributes.push(AttributeInfo {
            name_index,
            info: caturra_classfile::debug::encode_line_number_table(line_numbers),
        });
    }
    if !local_var_debug.is_empty() {
        let entries: Vec<caturra_classfile::debug::LocalVariableEntry> = local_var_debug
            .iter()
            .map(|(var_name, var_descriptor, _, slot, start_pc)| {
                caturra_classfile::debug::LocalVariableEntry {
                    start_pc: *start_pc,
                    length: code_len.saturating_sub(*start_pc),
                    name_index: pool.intern_utf8(var_name),
                    descriptor_index: pool.intern_utf8(var_descriptor),
                    index: *slot,
                }
            })
            .collect();
        let name_index = pool.intern_utf8(caturra_classfile::debug::LOCAL_VARIABLE_TABLE_ATTRIBUTE);
        code_attributes.push(AttributeInfo {
            name_index,
            info: caturra_classfile::debug::encode_local_variable_table(&entries),
        });

        // Generic locals additionally carry their full signature
        // (JVMS §4.7.14) so tooling can recover `ArrayList<Integer>`.
        let typed: Vec<caturra_classfile::debug::LocalVariableEntry> = local_var_debug
            .iter()
            .filter_map(|(var_name, _, signature, slot, start_pc)| {
                signature
                    .as_ref()
                    .map(|signature| caturra_classfile::debug::LocalVariableEntry {
                        start_pc: *start_pc,
                        length: code_len.saturating_sub(*start_pc),
                        name_index: pool.intern_utf8(var_name),
                        descriptor_index: pool.intern_utf8(signature),
                        index: *slot,
                    })
            })
            .collect();
        if !typed.is_empty() {
            let name_index =
                pool.intern_utf8(caturra_classfile::debug::LOCAL_VARIABLE_TYPE_TABLE_ATTRIBUTE);
            code_attributes.push(AttributeInfo {
                name_index,
                info: caturra_classfile::debug::encode_local_variable_table(&typed),
            });
        }
    }
    let code_attribute = CodeAttribute {
        max_stack,
        max_locals,
        code,
        exception_table,
        attributes: code_attributes,
    };
    let name_index = pool.intern_utf8(name);
    let descriptor_index = pool.intern_utf8(descriptor);
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

/// A compact handle for a user-defined class; index into
/// [`MethodTable::class_names`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClassId(u16);

/// One method or constructor signature, as seen by call resolution.
/// Constructors use the JVM name `<init>` and a void return.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // mirrors Java method modifiers
struct MethodSig {
    name: String,
    params: Vec<JType>,
    /// `None` is `void`.
    ret: Option<JType>,
    is_static: bool,
    is_private: bool,
    is_abstract: bool,
    /// The last parameter is `Type... name` — extra trailing arguments
    /// pack into the array.
    is_varargs: bool,
}

impl MethodSig {
    fn describe(&self, table: &MethodTable) -> String {
        let params: Vec<String> = self.params.iter().map(|p| p.describe(table)).collect();
        format!("{}({})", self.name, params.join(","))
    }

    fn descriptor(&self, table: &MethodTable) -> String {
        let mut out = String::from("(");
        for param in &self.params {
            out.push_str(&param.descriptor(table));
        }
        out.push(')');
        match self.ret {
            None => out.push('V'),
            Some(ty) => out.push_str(&ty.descriptor(table)),
        }
        out
    }
}

/// One field of a user-defined class.
#[derive(Debug, Clone)]
struct FieldSig {
    name: String,
    ty: JType,
    is_static: bool,
    is_private: bool,
    is_final: bool,
}

/// Everything call/field resolution knows about one class.
struct ClassInfo {
    id: ClassId,
    superclass: Option<ClassId>,
    /// `extends Exception` etc: a library throwable parent (internal
    /// name). Mutually exclusive with `superclass`.
    library_superclass: Option<&'static str>,
    interfaces: Vec<ClassId>,
    is_abstract: bool,
    is_interface: bool,
    is_enum: bool,
    /// A synthesized anonymous/lambda class's enclosing class. Hoisting the
    /// body to the top level loses sight of that class's STATIC fields, so a
    /// bare name that resolves to none of the usual places tries them.
    enclosing: Option<String>,
    /// Number of generic type parameters (`class Box<T>` → 1). Only
    /// single-parameter classes get type-argument tracking.
    type_param_count: usize,
    methods: Vec<MethodSig>,
    fields: Vec<FieldSig>,
}

/// All classes in the compilation.
struct MethodTable {
    /// Indexed by [`ClassId`].
    class_names: Vec<String>,
    classes: std::collections::HashMap<String, ClassInfo>,
    /// The synthetic `java.lang.Object` top type (every reference type
    /// is a subtype). Type parameters erase to it.
    object_id: ClassId,
    /// Class names brought in by `import static X.*` (e.g. `JUnit`
    /// `Assertions`) — unqualified calls fall back to their statics.
    static_imports: Vec<String>,
    /// Validation ("Test") mode: private-access checks are relaxed so a
    /// `JUnit` validator can reach the student's internals. Set when a
    /// source imports `org.junit`.
    relax_access: bool,
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
    /// Two passes: ids first, then member signatures.
    #[allow(clippy::too_many_lines)]
    fn build(units: &[(String, CompilationUnit)], diagnostics: &mut Vec<Diagnostic>) -> Self {
        // Pass 1: class names get ids so member types can refer to any
        // class regardless of declaration order.
        // Static imports (`import static X.*`) and validation mode.
        let mut static_imports = Vec::new();
        let mut relax_access = false;
        for (_, unit) in units {
            for import in &unit.imports {
                if import.path.first().map(String::as_str) == Some("org")
                    && import.path.get(1).map(String::as_str) == Some("junit")
                {
                    relax_access = true;
                }
                if import.is_static
                    && import.wildcard
                    && let Some(class) = import.path.last()
                {
                    static_imports.push(class.clone());
                }
            }
        }
        let mut table = Self {
            class_names: Vec::new(),
            classes: std::collections::HashMap::new(),
            object_id: ClassId(0),
            static_imports,
            relax_access,
        };
        // The synthetic top type: `Object`. It carries the universal
        // methods; user classes are registered as its subtypes.
        let object_id = ClassId(0);
        table.class_names.push(String::from("java/lang/Object"));
        let string_ret = Some(JType::Str);
        table.classes.insert(
            String::from("java/lang/Object"),
            ClassInfo {
                id: object_id,
                superclass: None,
                library_superclass: None,
                interfaces: Vec::new(),
                enclosing: None,
                is_abstract: false,
                is_interface: false,
                is_enum: false,
                type_param_count: 0,
                methods: vec![
                    MethodSig {
                        name: String::from("toString"),
                        params: Vec::new(),
                        ret: string_ret,
                        is_static: false,
                        is_private: false,
                        is_abstract: false,
                        is_varargs: false,
                    },
                    MethodSig {
                        name: String::from("getClass"),
                        params: Vec::new(),
                        ret: Some(JType::Class),
                        is_static: false,
                        is_private: false,
                        is_abstract: false,
                        is_varargs: false,
                    },
                    MethodSig {
                        name: String::from("hashCode"),
                        params: Vec::new(),
                        ret: Some(JType::Int),
                        is_static: false,
                        is_private: false,
                        is_abstract: false,
                        is_varargs: false,
                    },
                    MethodSig {
                        name: String::from("equals"),
                        params: vec![JType::Object(object_id)],
                        ret: Some(JType::Boolean),
                        is_static: false,
                        is_private: false,
                        is_abstract: false,
                        is_varargs: false,
                    },
                ],
                fields: Vec::new(),
            },
        );
        table.object_id = object_id;
        // `java.lang.Comparable<T>` — a functional interface every
        // user class may implement (`compareTo` compares to a peer).
        let comparable_id = ClassId(1);
        table.class_names.push(String::from("Comparable"));
        table.classes.insert(
            String::from("Comparable"),
            ClassInfo {
                id: comparable_id,
                superclass: None,
                library_superclass: None,
                interfaces: Vec::new(),
                enclosing: None,
                is_abstract: true,
                is_interface: true,
                is_enum: false,
                type_param_count: 1,
                methods: vec![MethodSig {
                    name: String::from("compareTo"),
                    params: vec![JType::TypeVar],
                    ret: Some(JType::Int),
                    is_static: false,
                    is_private: false,
                    is_abstract: true,
                    is_varargs: false,
                }],
                fields: Vec::new(),
            },
        );
        for (_, unit) in units {
            for class in &unit.classes {
                if table.classes.contains_key(&class.name) {
                    continue; // duplicate classes are reported by compile()
                }
                let id = ClassId(u16::try_from(table.class_names.len()).unwrap_or(u16::MAX));
                table.class_names.push(class.name.clone());
                table.classes.insert(
                    class.name.clone(),
                    ClassInfo {
                        id,
                        superclass: None,
                        library_superclass: None,
                        interfaces: Vec::new(),
                        enclosing: class.enclosing.clone(),
                        is_abstract: false,
                        // Known in pass 1 so the anonymous-class extends-vs-
                        // implements check (pass 2) is order-independent: an
                        // anonymous class implementing an interface declared in
                        // a later unit (e.g. the injected ActionListener) must
                        // still resolve it as an interface.
                        is_interface: class.is_interface,
                        is_enum: false,
                        type_param_count: 0,
                        methods: Vec::new(),
                        fields: Vec::new(),
                    },
                );
            }
        }

        // Pass 2: members, with types resolved against the full class
        // list.
        for (path, unit) in units {
            for class in &unit.classes {
                let mut methods = Vec::new();
                let mut fields = Vec::new();

                for field in &class.fields {
                    let sig = FieldSig {
                        name: field.name.clone(),
                        ty: table.resolve_type(&field.ty).unwrap_or(JType::Unsupported),
                        is_static: field.is_static,
                        is_private: field.is_private,
                        is_final: field.is_final,
                    };
                    if fields.iter().any(|f: &FieldSig| f.name == sig.name) {
                        diagnostics.push(Diagnostic::error(
                            path,
                            format!(
                                "variable {} is already defined in class {}",
                                sig.name, class.name
                            ),
                            field.span,
                        ));
                    } else {
                        fields.push(sig);
                    }
                }

                for method in &class.methods {
                    let sig = MethodSig {
                        name: if method.is_constructor {
                            String::from("<init>")
                        } else {
                            method.name.clone()
                        },
                        params: method
                            .params
                            .iter()
                            .map(|p| table.resolve_type(&p.ty).unwrap_or(JType::Unsupported))
                            .collect(),
                        ret: match &method.return_type {
                            TypeRef::Void => None,
                            other => Some(table.resolve_type(other).unwrap_or(JType::Unsupported)),
                        },
                        is_static: method.is_static,
                        is_private: method.is_private,
                        is_abstract: method.is_abstract,
                        is_varargs: method.params.last().is_some_and(|p| p.is_varargs),
                    };
                    if methods
                        .iter()
                        .any(|m: &MethodSig| m.name == sig.name && m.params == sig.params)
                    {
                        let what = if method.is_constructor {
                            format!("constructor {}", class.name)
                        } else {
                            format!("method {}", sig.describe(&table))
                        };
                        diagnostics.push(Diagnostic::error(
                            path,
                            format!("{what} is already defined in class {}", class.name),
                            method.span,
                        ));
                    } else {
                        methods.push(sig);
                    }
                }

                // A class with no declared constructor gets the default
                // one (JLS §8.8.9).
                if !class.is_interface && !methods.iter().any(|m| m.name == "<init>") {
                    methods.push(MethodSig {
                        name: String::from("<init>"),
                        params: Vec::new(),
                        ret: None,
                        is_static: false,
                        is_private: false,
                        is_abstract: false,
                        is_varargs: false,
                    });
                }

                let mut library_superclass = None;
                let superclass = class.superclass.as_ref().and_then(|name| {
                    let id = table.class_id(name);
                    if id.is_none() {
                        // `extends Exception` and friends: a library
                        // throwable parent.
                        if let Some(internal) =
                            caturra_classfile::exceptions::internal_name_of(name)
                        {
                            library_superclass = Some(internal);
                        } else {
                            diagnostics.push(Diagnostic::error(
                                path,
                                crate::imports::unsupported_class_reason(name)
                                    .unwrap_or_else(|| format!("cannot find symbol: class {name}")),
                                class.span,
                            ));
                        }
                    }
                    id
                });
                let interface_ids: Vec<ClassId> = class
                    .interfaces
                    .iter()
                    .filter_map(|name| {
                        // `implements Comparator<T>` implements the bundled
                        // erased `__Comparator` (a student-facing alias).
                        let name = comparator_alias(name);
                        let id = table.class_id(name);
                        if id.is_none() {
                            diagnostics.push(Diagnostic::error(
                                path,
                                crate::imports::unsupported_class_reason(name)
                                    .unwrap_or_else(|| format!("cannot find symbol: class {name}")),
                                class.span,
                            ));
                        }
                        id
                    })
                    .collect();

                // An anonymous class's single supertype is an
                // `implements` if it names an interface, otherwise an
                // `extends`.
                let (superclass, interface_ids) = if class.is_anonymous
                    && let Some(super_id) = superclass
                    && table.info_by_id(super_id).is_some_and(|i| i.is_interface)
                {
                    (None, vec![super_id])
                } else {
                    (superclass, interface_ids)
                };
                let info = table
                    .classes
                    .get_mut(&class.name)
                    .expect("registered in pass 1");
                info.methods = methods;
                info.fields = fields;
                // A class with no declared parent is a subtype of the
                // synthetic Object top type (so `widens(C, Object)`
                // holds), except interfaces and Object itself.
                info.superclass = if superclass.is_some() || library_superclass.is_some() {
                    superclass
                } else if !class.is_interface {
                    Some(object_id)
                } else {
                    None
                };
                info.library_superclass = library_superclass;
                info.interfaces = interface_ids;
                info.is_abstract = class.is_abstract;
                info.is_interface = class.is_interface;
                info.is_enum = class.is_enum;
                info.type_param_count = class.type_params.len();
            }
        }
        table.check_hierarchy(units, diagnostics);
        table
    }

    /// Post-pass hierarchy validation: cycles, extends-shape, override
    /// compatibility, abstract completeness, field hiding.
    #[allow(clippy::too_many_lines)]
    fn check_hierarchy(
        &mut self,
        units: &[(String, CompilationUnit)],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Cycles: walk each chain with a step bound.
        let mut cyclic = Vec::new();
        for info in self.classes.values() {
            let mut current = info.superclass;
            let mut steps = 0usize;
            while let Some(id) = current {
                if id == info.id || steps > self.class_names.len() {
                    cyclic.push(info.id);
                    break;
                }
                steps += 1;
                current = self.info_by_id(id).and_then(|i| i.superclass);
            }
        }
        for id in cyclic {
            let name = self.class_name(id).to_owned();
            diagnostics.push(Diagnostic {
                severity: crate::diagnostics::Severity::Error,
                message: format!("cyclic inheritance involving {name}"),
                path: String::new(),
                span: None,
            });
            // Break the cycle so later chain walks terminate.
            if let Some(info) = self.classes.get_mut(&name) {
                info.superclass = None;
            }
        }

        for (path, unit) in units {
            for class in &unit.classes {
                let Some(info) = self.classes.get(&class.name) else {
                    continue;
                };
                // Shape checks: classes extend classes, implement interfaces.
                if let Some(sup) = info.superclass
                    && self.info_by_id(sup).is_some_and(|s| s.is_interface)
                {
                    diagnostics.push(Diagnostic::error(
                        path,
                        format!(
                            "class {} cannot extend interface {} (use implements)",
                            class.name,
                            self.class_name(sup)
                        ),
                        class.span,
                    ));
                }
                for iface in &info.interfaces {
                    if self.info_by_id(*iface).is_some_and(|i| !i.is_interface)
                        && !class.is_interface
                    {
                        diagnostics.push(Diagnostic::error(
                            path,
                            format!(
                                "{} is not an interface (use extends for classes)",
                                self.class_name(*iface)
                            ),
                            class.span,
                        ));
                    }
                }

                // Override compatibility against ancestors.
                for method in &class.methods {
                    if method.is_constructor {
                        continue;
                    }
                    let params: Vec<JType> = method
                        .params
                        .iter()
                        .map(|p| self.resolve_type(&p.ty).unwrap_or(JType::Unsupported))
                        .collect();
                    let ret = match &method.return_type {
                        TypeRef::Void => None,
                        other => Some(self.resolve_type(other).unwrap_or(JType::Unsupported)),
                    };
                    let mut ancestor = info.superclass;
                    while let Some(id) = ancestor {
                        let Some(parent) = self.info_by_id(id) else {
                            break;
                        };
                        if let Some(sup_sig) = parent
                            .methods
                            .iter()
                            .find(|m| m.name == method.name && m.params == params && !m.is_private)
                        {
                            let compatible_return = sup_sig.ret == ret
                                || matches!(
                                    (ret, sup_sig.ret),
                                    (Some(JType::Object(sub)), Some(JType::Object(sup)))
                                        if self.is_subtype(sub, sup)
                                );
                            if !compatible_return || sup_sig.is_static != method.is_static {
                                diagnostics.push(Diagnostic::error(
                                    path,
                                    format!(
                                        "{}() in {} cannot override {}() in {}",
                                        method.name,
                                        class.name,
                                        method.name,
                                        self.class_name(id)
                                    ),
                                    method.span,
                                ));
                            }
                            break;
                        }
                        ancestor = parent.superclass;
                    }
                }

                // Concrete classes must implement all abstract methods.
                if !class.is_abstract && !class.is_interface {
                    let unimplemented = self.missing_abstract_method(info.id);
                    if let Some((method_name, owner)) = unimplemented {
                        diagnostics.push(Diagnostic::error(
                            path,
                            format!(
                                "{} is not abstract and does not override abstract method \
                                 {method_name}() in {owner}",
                                class.name
                            ),
                            class.span,
                        ));
                    }
                }
            }
        }
    }

    fn info_by_id(&self, id: ClassId) -> Option<&ClassInfo> {
        let name = self.class_names.get(usize::from(id.0))?;
        self.classes.get(name)
    }

    /// The resolved superclass internal name of an anonymous class:
    /// `java/lang/Object` when it implements an interface, otherwise
    /// the class it extends.
    fn anon_super_name(&self, name: &str) -> String {
        self.classes
            .get(name)
            .and_then(|info| info.superclass)
            .map_or_else(
                || String::from("java/lang/Object"),
                |id| self.class_name(id).to_owned(),
            )
    }

    /// The resolved interface names an anonymous class implements
    /// (its declared supertype, when that is an interface).
    fn anon_supertypes(&self, name: &str) -> Vec<String> {
        self.classes
            .get(name)
            .map(|info| {
                info.interfaces
                    .iter()
                    .map(|id| self.class_name(*id).to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether the class with this id is an `enum`.
    fn is_enum(&self, id: ClassId) -> bool {
        self.info_by_id(id).is_some_and(|info| info.is_enum)
    }

    /// Whether `sub` is `sup` or reachable via extends/implements.
    fn is_subtype(&self, sub: ClassId, sup: ClassId) -> bool {
        if sub == sup {
            return true;
        }
        let mut steps = 0usize;
        let mut stack = vec![sub];
        while let Some(id) = stack.pop() {
            steps += 1;
            if steps > self.class_names.len() * 4 {
                return false; // cycle guard
            }
            if id == sup {
                return true;
            }
            if let Some(info) = self.info_by_id(id) {
                if let Some(parent) = info.superclass {
                    stack.push(parent);
                }
                stack.extend(info.interfaces.iter().copied());
            }
        }
        false
    }

    /// Find one abstract method (from the superclass chain or any
    /// interface) with no concrete implementation in the chain.
    fn missing_abstract_method(&self, class: ClassId) -> Option<(String, String)> {
        // Collect every abstract signature visible to this class.
        let mut required: Vec<(&MethodSig, ClassId)> = Vec::new();
        let mut stack = vec![class];
        let mut steps = 0usize;
        while let Some(id) = stack.pop() {
            steps += 1;
            if steps > self.class_names.len() * 4 {
                break;
            }
            if let Some(info) = self.info_by_id(id) {
                for m in &info.methods {
                    if m.is_abstract {
                        required.push((m, id));
                    }
                }
                if let Some(parent) = info.superclass {
                    stack.push(parent);
                }
                stack.extend(info.interfaces.iter().copied());
            }
        }

        'outer: for (sig, owner) in required {
            // Look for a concrete implementation along the class chain.
            let mut current = Some(class);
            let mut steps = 0usize;
            while let Some(id) = current {
                steps += 1;
                if steps > self.class_names.len() + 1 {
                    break;
                }
                if let Some(info) = self.info_by_id(id) {
                    if info.methods.iter().any(|m| {
                        m.name == sig.name
                            && !m.is_abstract
                            && self.params_override(&m.params, &sig.params)
                    }) {
                        continue 'outer;
                    }
                    current = info.superclass;
                } else {
                    break;
                }
            }
            return Some((sig.name.clone(), self.class_name(owner).to_owned()));
        }
        None
    }

    /// Whether a concrete method's parameters override an abstract
    /// method's, allowing for erasure: an abstract type-variable or
    /// `Object` parameter is satisfied by any reference argument (the
    /// generic-interface bridge, e.g. `compareTo(Foo)` implements
    /// `Comparable<Foo>.compareTo(T)`).
    fn params_override(&self, concrete: &[JType], abstract_: &[JType]) -> bool {
        concrete.len() == abstract_.len()
            && concrete.iter().zip(abstract_).all(|(c, a)| {
                c == a
                    || *a == JType::TypeVar
                    || (*a == JType::Object(self.object_id) && c.is_reference())
            })
    }

    fn has_class(&self, name: &str) -> bool {
        self.classes.contains_key(name)
    }

    fn class_id(&self, name: &str) -> Option<ClassId> {
        self.classes.get(name).map(|c| c.id)
    }

    fn class_name(&self, id: ClassId) -> &str {
        self.class_names
            .get(usize::from(id.0))
            .map_or("<unknown>", String::as_str)
    }

    /// Resolve a source-level type, mapping class names (including
    /// array elements) to ids.
    #[allow(clippy::too_many_lines)]
    fn resolve_type(&self, ty: &TypeRef) -> Option<JType> {
        match ty {
            TypeRef::Named(name) => {
                // `java.util.Scanner` and friends resolve without an
                // import; qualified names never match user classes.
                let simple = crate::imports::canonical_library_class(name).unwrap_or(name.as_str());
                if simple == "String" {
                    return Some(JType::Str);
                }
                // The synthetic Object top type.
                if name == "Object" || name == "java.lang.Object" {
                    return Some(JType::Object(self.object_id));
                }
                if (name == "Comparable" || name == "java.lang.Comparable")
                    && let Some(id) = self.class_id("Comparable")
                {
                    return Some(JType::Object(id));
                }
                // `Comparator` aliases the bundled erased `__Comparator`.
                if (name == "Comparator" || name == "java.util.Comparator")
                    && !self.has_class("Comparator")
                    && let Some(id) = self.class_id("__Comparator")
                {
                    return Some(JType::Object(id));
                }
                // The erased single type variable of the enclosing
                // generic class.
                if name == crate::parser::TYPEVAR_SENTINEL {
                    return Some(JType::TypeVar);
                }
                // Wrapper types (`Integer`, `Double`, ...) as a boxed
                // value — unless the name is a user class of that name.
                if !self.has_class(simple)
                    && let Some(elem) = wrapper_elem(simple)
                {
                    return Some(JType::Boxed(elem));
                }
                // User classes shadow the intrinsic simple names. A
                // qualified library name resolves here too, because the
                // bundled classes (`Arrays`, `Random`, ...) live in the
                // table under their simple name.
                if (!name.contains('.') || crate::imports::canonical_library_class(name).is_some())
                    && let Some(id) = self.class_id(simple)
                {
                    return Some(JType::Object(id));
                }
                // `Outer.Inner`: a qualified reference to a nested user
                // class (flattened to a top-level simple name).
                if name.contains('.')
                    && crate::imports::canonical_library_class(name).is_none()
                    && let Some(last) = name.rsplit('.').next()
                    && let Some(id) = self.class_id(last)
                {
                    return Some(JType::Object(id));
                }
                match simple {
                    "Scanner" => Some(JType::Scanner),
                    "StringBuilder" => Some(JType::StringBuilder),
                    "File" => Some(JType::File),
                    "PrintWriter" => Some(JType::Writer),
                    "OptionalInt" if !self.has_class(simple) => Some(JType::OptionalInt),
                    "OptionalDouble" if !self.has_class(simple) => Some(JType::OptionalDouble),
                    "Class" => Some(JType::Class),
                    "Field" => Some(JType::Field),
                    "Method" => Some(JType::Method),
                    "Type" | "ParameterizedType" => Some(JType::Type),
                    "Constructor" => Some(JType::Constructor),
                    other => {
                        let internal = if name.contains('.') {
                            name.replace('.', "/")
                        } else {
                            caturra_classfile::exceptions::internal_name_of(other)?.to_owned()
                        };
                        exception_id(&internal).map(JType::Exception)
                    }
                }
            }
            TypeRef::Generic { base, args } => {
                let mut simple =
                    crate::imports::canonical_library_class(base).unwrap_or(base.as_str());
                // `List<E>` is the interface form of the ArrayList caturra models.
                if simple == "List" && !self.has_class("List") {
                    simple = "ArrayList";
                }
                // `Map<K, V>` is the interface form of the HashMap caturra models.
                if simple == "Map" && !self.has_class("Map") {
                    simple = "HashMap";
                }
                // `HashSet<E>` is the concrete form of the Set caturra models.
                if simple == "HashSet" && !self.has_class("HashSet") {
                    simple = "Set";
                }
                if simple == "ArrayList" && args.len() == 1 && !self.has_class(simple) {
                    elem_from_type_arg(&args[0], self).map(JType::List)
                } else if simple == "Stack" && args.len() == 1 && !self.has_class(simple) {
                    elem_from_type_arg(&args[0], self).map(JType::Stack)
                } else if simple == "HashMap" && args.len() == 2 && !self.has_class(simple) {
                    let key = elem_from_type_arg(&args[0], self)?;
                    let value = elem_from_type_arg(&args[1], self)?;
                    Some(JType::Map { key, value })
                } else if matches!(simple, "TreeMap" | "SortedMap" | "NavigableMap")
                    && args.len() == 2
                    && !self.has_class(simple)
                {
                    let key = elem_from_type_arg(&args[0], self)?;
                    let value = elem_from_type_arg(&args[1], self)?;
                    Some(JType::TreeMap { key, value })
                } else if simple == "Set" && args.len() == 1 && !self.has_class(simple) {
                    elem_from_type_arg(&args[0], self).map(JType::Set)
                } else if matches!(simple, "TreeSet" | "SortedSet" | "NavigableSet")
                    && args.len() == 1
                    && !self.has_class(simple)
                {
                    elem_from_type_arg(&args[0], self).map(JType::TreeSet)
                } else if simple == "Optional" && args.len() == 1 && !self.has_class(simple) {
                    elem_from_type_arg(&args[0], self).map(JType::Optional)
                } else if matches!(
                    simple,
                    "LinkedList" | "ArrayDeque" | "Queue" | "Deque" | "PriorityQueue"
                ) && args.len() == 1
                    && !self.has_class(simple)
                {
                    let role = match simple {
                        // A PriorityQueue is a Queue (a distinct heap object at
                        // runtime, but the same restricted method surface).
                        "Queue" | "PriorityQueue" => SeqRole::Queue,
                        "Deque" => SeqRole::Deque,
                        "ArrayDeque" => SeqRole::ArrayDeque,
                        _ => SeqRole::Full,
                    };
                    elem_from_type_arg(&args[0], self).map(|elem| JType::LinkedList { elem, role })
                } else if simple == "Collection" && args.len() == 1 && !self.has_class(simple) {
                    elem_from_type_arg(&args[0], self).map(JType::Collection)
                } else if matches!(simple, "Map.Entry" | "Entry")
                    && args.len() == 2
                    && !self.has_class(simple)
                {
                    let key = elem_from_type_arg(&args[0], self)?;
                    let value = elem_from_type_arg(&args[1], self)?;
                    Some(JType::MapEntry { key, value })
                } else if matches!(simple, "Comparator")
                    && !self.has_class("Comparator")
                    && let Some(id) = self.class_id("__Comparator")
                {
                    // `Comparator<T>` erases to the bundled `__Comparator`.
                    Some(JType::Object(id))
                } else if !self.has_class(base)
                    && matches!(simple, "Class" | "Constructor" | "Field")
                {
                    // `Class<?>` / `Constructor<?>` erase to the raw reflection type.
                    Some(match simple {
                        "Constructor" => JType::Constructor,
                        "Field" => JType::Field,
                        "Method" => JType::Method,
                        _ => JType::Class,
                    })
                } else if let Some(id) = self.class_id(base) {
                    // A single-type-parameter user class tracks its
                    // argument (`Box<String>`); otherwise it is raw.
                    let single = self.info_by_id(id).is_some_and(|i| i.type_param_count == 1);
                    let arg = if single && args.len() == 1 {
                        elem_from_type_arg(&args[0], self)
                    } else {
                        None
                    };
                    match arg {
                        // Track only reference type arguments (primitive
                        // args would need boxing, which caturra lacks).
                        Some(elem @ (ElemType::Str | ElemType::Object(_))) => {
                            Some(JType::Generic {
                                class: id,
                                arg: elem,
                            })
                        }
                        _ => Some(JType::Object(id)),
                    }
                } else {
                    None
                }
            }
            TypeRef::Array(_) => {
                // Peel dimensions, then resolve the base.
                let mut dims: u8 = 0;
                let mut current = ty;
                while let TypeRef::Array(next) = current {
                    dims = dims.checked_add(1)?;
                    current = next;
                }
                let elem = match self.resolve_type(current)? {
                    JType::Int => ElemType::Int,
                    JType::Double => ElemType::Double,
                    JType::Long => ElemType::Long,
                    JType::Float => ElemType::Float,
                    JType::Short => ElemType::Short,
                    JType::Byte => ElemType::Byte,
                    JType::Boolean => ElemType::Boolean,
                    JType::Char => ElemType::Char,
                    JType::Str => ElemType::Str,
                    JType::Object(id) => ElemType::Object(id),
                    JType::Field => ElemType::Field,
                    JType::Method => ElemType::Method,
                    JType::Constructor => ElemType::Constructor,
                    JType::Class => ElemType::Class,
                    // A wrapper array (`Integer[]`) stores its primitives
                    // directly, like the corresponding primitive array.
                    JType::Boxed(elem) => elem,
                    _ => return None,
                };
                Some(JType::Array { elem, dims })
            }
            other => type_from_ref(other),
        }
    }

    /// The library throwable this class descends from, if any: walks
    /// the user `extends` chain to its `library_superclass`.
    fn library_throwable_ancestor(&self, class: ClassId) -> Option<&'static str> {
        let mut current = Some(class);
        let mut steps = 0usize;
        while let Some(id) = current {
            steps += 1;
            if steps > self.class_names.len() + 1 {
                return None;
            }
            let info = self.info_by_id(id)?;
            if let Some(library) = info.library_superclass {
                return Some(library);
            }
            current = info.superclass;
        }
        None
    }

    /// Whether instances of this user class can be thrown/caught.
    fn is_throwable(&self, class: ClassId) -> bool {
        self.library_throwable_ancestor(class).is_some()
    }

    /// Look up a field in a class or its ancestors, returning the
    /// owning class alongside.
    fn field(&self, class: &str, name: &str) -> Option<(ClassId, &FieldSig)> {
        let mut current = self.classes.get(class);
        let mut steps = 0usize;
        while let Some(info) = current {
            steps += 1;
            if steps > self.class_names.len() + 1 {
                return None;
            }
            if let Some(field) = info.fields.iter().find(|f| f.name == name) {
                return Some((info.id, field));
            }
            current = info.superclass.and_then(|id| self.info_by_id(id));
        }
        None
    }

    /// Resolve `class.name(args)`: applicable-by-widening, exact match
    /// first, then the unique most-specific method.
    fn resolve(&self, class: &str, name: &str, args: &[JType]) -> Resolution<'_> {
        if !self.classes.contains_key(class) {
            return Resolution::UnknownName;
        }
        // Walk the chain (and interfaces, for interface receivers),
        // nearest declaration first; an override shadows its ancestor.
        let mut named: Vec<&MethodSig> = Vec::new();
        let start = self.classes.get(class).map(|i| i.id);
        let mut stack = vec![start];
        let mut steps = 0usize;
        while let Some(Some(id)) = stack.pop() {
            steps += 1;
            if steps > self.class_names.len() * 4 {
                break;
            }
            if let Some(info) = self.info_by_id(id) {
                // JLS §8.4.8: an interface's static method is NOT inherited.
                // `C implements I` cannot call `C.hi()` or `c.hi()` for a
                // static `I.hi()` — only `I.hi()`. javac says "cannot find
                // symbol"; caturra used to compile it and die at run time.
                let skip_static = info.is_interface && Some(id) != start;
                for m in &info.methods {
                    if skip_static && m.is_static {
                        continue;
                    }
                    if m.name == name
                        && !named
                            .iter()
                            .any(|seen| seen.params == m.params && seen.name == m.name)
                    {
                        named.push(m);
                    }
                }
                if let Some(parent) = info.superclass {
                    stack.push(Some(parent));
                }
                for iface in &info.interfaces {
                    stack.push(Some(*iface));
                }
            }
        }
        if named.is_empty() {
            return Resolution::UnknownName;
        }
        let applicable: Vec<&MethodSig> = named
            .iter()
            .copied()
            .filter(|m| {
                m.params.len() == args.len()
                    && m.params.iter().zip(args).all(|(p, a)| widens(*a, *p, self))
            })
            .collect();
        // JLS §15.12.2: fixed-arity applicability is tried first; only
        // if nothing matches do varargs methods enter the running.
        if applicable.is_empty() {
            let varargs_applicable: Vec<&MethodSig> = named
                .iter()
                .copied()
                .filter(|m| m.is_varargs && self.varargs_applicable(m, args))
                .collect();
            return match varargs_applicable.len() {
                0 => Resolution::NoneApplicable,
                1 => Resolution::Found(varargs_applicable[0]),
                _ => Resolution::Ambiguous(
                    varargs_applicable
                        .iter()
                        .map(|m| m.describe(self))
                        .collect(),
                ),
            };
        }
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
                                .all(|(a, b)| widens(*a, *b, self))
                        })
                    })
                    .collect();
                if most_specific.len() == 1 {
                    Resolution::Found(most_specific[0])
                } else {
                    Resolution::Ambiguous(applicable.iter().map(|m| m.describe(self)).collect())
                }
            }
        }
    }

    /// Whether a varargs method accepts `args` in spread or array form:
    /// the fixed leading parameters must widen, and either the trailing
    /// arguments each widen to the element type (spread), or a single
    /// trailing argument is the array itself (array form).
    fn varargs_applicable(&self, m: &MethodSig, args: &[JType]) -> bool {
        let fixed = m.params.len() - 1;
        if args.len() < fixed {
            return false;
        }
        if !m.params[..fixed]
            .iter()
            .zip(args)
            .all(|(p, a)| widens(*a, *p, self))
        {
            return false;
        }
        let array_ty = m.params[fixed];
        let Some(elem) = array_ty.element_type() else {
            return false;
        };
        // Array form: exactly one trailing arg, assignable to the array.
        if args.len() == m.params.len() && widens(args[fixed], array_ty, self) {
            return true;
        }
        // Spread form: every trailing arg widens to the element type.
        args[fixed..].iter().all(|a| widens(*a, elem, self))
    }
}

/// Exception ids: 0 is `java/lang/Throwable`, `i + 1` indexes
/// [`caturra_classfile::exceptions::EXCEPTIONS`].
fn exception_id(internal: &str) -> Option<u8> {
    if internal == "java/lang/Throwable" {
        return Some(0);
    }
    caturra_classfile::exceptions::EXCEPTIONS
        .iter()
        .position(|(class, _)| *class == internal)
        .and_then(|index| u8::try_from(index + 1).ok())
}

/// A catch clause's resolved type: a library throwable or a
/// user-defined exception class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CatchKind {
    Library(u8),
    User(ClassId),
}

impl CatchKind {
    fn jtype(self) -> JType {
        match self {
            CatchKind::Library(id) => JType::Exception(id),
            CatchKind::User(id) => JType::Object(id),
        }
    }

    /// The class-file name for the exception-table entry.
    fn table_name(self, table: &MethodTable) -> String {
        match self {
            CatchKind::Library(id) => exception_internal(id).to_owned(),
            CatchKind::User(id) => table.class_name(id).to_owned(),
        }
    }

    /// The simple display name (javac's already-caught wording).
    fn simple_name(self, table: &MethodTable) -> String {
        match self {
            CatchKind::Library(id) => exception_internal(id)
                .rsplit('/')
                .next()
                .unwrap_or_default()
                .to_owned(),
            CatchKind::User(id) => table.class_name(id).to_owned(),
        }
    }

    /// Whether `self` is `other` or a subclass of it (so an earlier
    /// `other` clause masks a later `self`).
    fn is_masked_by(self, other: CatchKind, table: &MethodTable) -> bool {
        match (self, other) {
            (CatchKind::Library(a), CatchKind::Library(b)) => {
                caturra_classfile::exceptions::is_exception_subclass(
                    exception_internal(a),
                    exception_internal(b),
                )
            }
            (CatchKind::User(a), CatchKind::User(b)) => table.is_subtype(a, b),
            (CatchKind::User(a), CatchKind::Library(b)) => {
                table.library_throwable_ancestor(a).is_some_and(|ancestor| {
                    caturra_classfile::exceptions::is_exception_subclass(
                        ancestor,
                        exception_internal(b),
                    )
                })
            }
            (CatchKind::Library(_), CatchKind::User(_)) => false,
        }
    }
}

fn exception_internal(id: u8) -> &'static str {
    if id == 0 {
        return "java/lang/Throwable";
    }
    caturra_classfile::exceptions::EXCEPTIONS
        .get(usize::from(id - 1))
        .map_or("java/lang/Throwable", |(class, _)| class)
}

/// A boxed wrapper viewed as its primitive for numeric operators
/// (auto-unboxing); non-wrapper types pass through.
fn numeric_view(ty: JType) -> JType {
    match ty {
        JType::Boxed(elem) => elem.base_type(),
        other => other,
    }
}

/// The `ElemType` a primitive `JType` boxes into (`int` -> Integer).
fn boxable_primitive(ty: JType) -> Option<ElemType> {
    Some(match ty {
        JType::Int => ElemType::Int,
        JType::Double => ElemType::Double,
        JType::Long => ElemType::Long,
        JType::Float => ElemType::Float,
        JType::Short => ElemType::Short,
        JType::Byte => ElemType::Byte,
        JType::Char => ElemType::Char,
        JType::Boolean => ElemType::Boolean,
        _ => return None,
    })
}

/// The boxed primitive kind for a wrapper class simple name, if any.
fn wrapper_elem(name: &str) -> Option<ElemType> {
    Some(match name {
        "Integer" => ElemType::Int,
        "Double" => ElemType::Double,
        "Long" => ElemType::Long,
        "Float" => ElemType::Float,
        "Short" => ElemType::Short,
        "Byte" => ElemType::Byte,
        "Character" => ElemType::Char,
        "Boolean" => ElemType::Boolean,
        _ => return None,
    })
}

/// The internal (slash) name of the wrapper class for a boxed
/// primitive: `Integer`, `Double`, ...
fn wrapper_internal(elem: ElemType) -> &'static str {
    match elem {
        ElemType::Int => "java/lang/Integer",
        ElemType::Double => "java/lang/Double",
        ElemType::Long => "java/lang/Long",
        ElemType::Float => "java/lang/Float",
        ElemType::Short => "java/lang/Short",
        ElemType::Byte => "java/lang/Byte",
        ElemType::Char => "java/lang/Character",
        ElemType::Boolean => "java/lang/Boolean",
        ElemType::Str
        | ElemType::Object(_)
        | ElemType::Field
        | ElemType::Method
        | ElemType::Constructor
        | ElemType::Class => "java/lang/Object",
    }
}

/// The display name of a list element as its wrapper type.
/// The name inside `ty` that explains why it does not resolve: a real Java
/// class caturra does not model, or a qualified name that names nothing.
/// Searched base first, then the type arguments, so
/// `ArrayList<LinkedList<Integer>>` blames the `LinkedList`.
fn unsupported_name_in(ty: &TypeRef) -> Option<String> {
    let (name, args): (&str, &[TypeRef]) = match ty {
        TypeRef::Named(name) => (name.as_str(), &[]),
        TypeRef::Generic { base, args } => (base.as_str(), args.as_slice()),
        TypeRef::Array(inner) => return unsupported_name_in(inner),
        _ => return None,
    };
    if name.contains('.') {
        if crate::imports::canonical_library_class(name).is_none() {
            return Some(crate::imports::unknown_qualified_message(name));
        }
        // The class is modeled. With type arguments, one of them is at
        // fault; without, the class itself is unusable here — a raw
        // `java.util.HashMap` — and `unknown_qualified_message` says so.
        return args.iter().find_map(unsupported_name_in).or_else(|| {
            args.is_empty()
                .then(|| crate::imports::unknown_qualified_message(name))
        });
    }
    if let Some(reason) = crate::imports::unsupported_class_reason(name) {
        return Some(reason);
    }
    args.iter().find_map(unsupported_name_in)
}

/// The first name in `ty` the table cannot resolve at all — a typo, most
/// likely. Type arguments are searched before the base, so
/// `ArrayList<Frobnicator>` blames the `Frobnicator`.
fn unknown_name_in(ty: &TypeRef, table: &MethodTable) -> Option<String> {
    match ty {
        TypeRef::Named(name) => table.resolve_type(ty).is_none().then(|| name.clone()),
        TypeRef::Generic { base, args } => args
            .iter()
            .find_map(|arg| unknown_name_in(arg, table))
            .or_else(|| {
                table
                    .resolve_type(&TypeRef::Named(base.clone()))
                    .is_none()
                    .then(|| base.clone())
            }),
        TypeRef::Array(inner) => unknown_name_in(inner, table),
        _ => None,
    }
}

/// Why a declared type does not resolve. A real Java 11 class caturra does
/// not model says so by name — `LinkedList<Integer> l;` reads to a student
/// exactly as its import and its `new` do, rather than as a typo or as the
/// unhelpful "this type cannot be used for a variable".
fn unresolved_type_message(ty: &TypeRef, table: &MethodTable) -> String {
    if let Some(reason) = unsupported_name_in(ty) {
        return reason;
    }
    if let Some(name) = unknown_name_in(ty, table) {
        return format!("unknown type '{name}'");
    }
    match ty {
        TypeRef::Named(name) | TypeRef::Generic { base: name, .. } => {
            format!("unknown type '{name}'")
        }
        TypeRef::Array(_) => String::from("arrays are not yet supported by caturra"),
        _ => String::from("this type cannot be used for a variable"),
    }
}

fn wrapper_name(elem: ElemType, table: &MethodTable) -> String {
    match elem {
        ElemType::Int => String::from("Integer"),
        ElemType::Double => String::from("Double"),
        ElemType::Long => String::from("Long"),
        ElemType::Float => String::from("Float"),
        ElemType::Short => String::from("Short"),
        ElemType::Byte => String::from("Byte"),
        ElemType::Boolean => String::from("Boolean"),
        ElemType::Char => String::from("Character"),
        ElemType::Str => String::from("String"),
        // The top type is stored under its internal name, which must not
        // reach a diagnostic: javac writes `ArrayList<Object>`.
        ElemType::Object(id) if id == table.object_id => String::from("Object"),
        ElemType::Object(id) => table.class_name(id).to_owned(),
        ElemType::Field => String::from("Field"),
        ElemType::Method => String::from("Method"),
        ElemType::Constructor => String::from("Constructor"),
        ElemType::Class => String::from("Class"),
    }
}

/// The JVM generic signature of a parameterized field type
/// (`ArrayList<Friend>` → `Ljava/util/ArrayList<LFriend;>;`), for the field
/// `Signature` attribute. `None` for non-generic or non-reference types.
fn generic_field_signature(ty: &TypeRef, table: &MethodTable) -> Option<String> {
    let TypeRef::Generic { args, .. } = ty else {
        return None;
    };
    if args.is_empty() {
        return None;
    }
    let erased = table.resolve_type(ty)?.descriptor(table);
    let base = erased.strip_suffix(';')?;
    if !base.starts_with('L') {
        return None;
    }
    let mut arg_sig = String::new();
    for arg in args {
        arg_sig.push_str(&table.resolve_type(arg)?.descriptor(table));
    }
    Some(format!("{base}<{arg_sig}>;"))
}

/// The [`ElemType`] a generic type argument denotes, per CSA usage
/// (wrapper classes, String, or a user class).
fn elem_from_type_arg(arg: &TypeRef, table: &MethodTable) -> Option<ElemType> {
    match arg {
        TypeRef::Named(name) => {
            // A user class shadows the wrapper/library simple names
            // (a level may define its own `Character`).
            if !name.contains('.')
                && let Some(id) = table.class_id(name)
            {
                return Some(ElemType::Object(id));
            }
            match crate::imports::canonical_library_class(name).unwrap_or(name.as_str()) {
                "Integer" => Some(ElemType::Int),
                "Double" => Some(ElemType::Double),
                "Boolean" => Some(ElemType::Boolean),
                "Character" => Some(ElemType::Char),
                "String" => Some(ElemType::Str),
                "Object" => Some(ElemType::Object(table.object_id)),
                other => table.class_id(other).map(ElemType::Object),
            }
        }
        _ => None,
    }
}

/// The [`ElemType`] for a base (non-array) type, if it can be an array
/// element.
fn elem_type_of(ty: JType) -> Option<ElemType> {
    match ty {
        JType::Int => Some(ElemType::Int),
        JType::Double => Some(ElemType::Double),
        JType::Long => Some(ElemType::Long),
        JType::Float => Some(ElemType::Float),
        JType::Short => Some(ElemType::Short),
        JType::Byte => Some(ElemType::Byte),
        JType::Boolean => Some(ElemType::Boolean),
        JType::Char => Some(ElemType::Char),
        JType::Str => Some(ElemType::Str),
        JType::Object(id) => Some(ElemType::Object(id)),
        JType::Class => Some(ElemType::Class),
        // A wrapper array element stores its primitive (`Integer[]` -> `int[]`).
        JType::Boxed(elem) => Some(elem),
        _ => None,
    }
}

/// The element type of any single-element collection type (a `List`, `Set`,
/// `TreeSet`, `Collection`, or `LinkedList`/`Queue`/`Deque`), for a constructor
/// that copies one.
fn collection_element_type(ty: JType) -> Option<ElemType> {
    match ty {
        JType::List(elem)
        | JType::Set(elem)
        | JType::TreeSet(elem)
        | JType::Collection(elem)
        | JType::LinkedList { elem, .. } => Some(elem),
        _ => None,
    }
}

/// The bundled interface a source `Comparator` name aliases: student code
/// writes `Comparator`, which caturra models with the erased `__Comparator`
/// (its `compare(Object, Object)` reaches a user `compare(T, T)` through the
/// VM's erasure bridge, exactly as `Comparable.compareTo` does).
fn comparator_alias(name: &str) -> &str {
    match name {
        "Comparator" | "java.util.Comparator" => "__Comparator",
        other => other,
    }
}

/// Method-invocation / assignment widening (JLS §5.3 without boxing),
/// including reference widening up the class hierarchy.
#[allow(clippy::too_many_lines)] // one big disjunction of widening rules
fn widens(from: JType, to: JType, table: &MethodTable) -> bool {
    from == to
        || matches!(
            (from, to),
            (JType::Char, JType::Int)
                | (
                    JType::Int | JType::Char | JType::Long | JType::Float,
                    JType::Double
                )
                | (JType::Int | JType::Char, JType::Long)
                | (JType::Int | JType::Char | JType::Long, JType::Float)
                | (JType::Byte, JType::Short)
                | (
                    JType::Byte | JType::Short,
                    JType::Int | JType::Long | JType::Float | JType::Double
                )
                | (
                    JType::Null,
                    JType::Str | JType::Array { .. } | JType::Object(_)
                )
        )
        || matches!(
            (from, to),
            (JType::Object(sub), JType::Object(sup)) if table.is_subtype(sub, sup)
        )
        || matches!(
            (from, to),
            (JType::Exception(sub), JType::Exception(sup))
                if caturra_classfile::exceptions::is_exception_subclass(
                    exception_internal(sub),
                    exception_internal(sup),
                )
        )
        || matches!(
            (from, to),
            (JType::Object(sub), JType::Exception(sup))
                if table.library_throwable_ancestor(sub).is_some_and(|ancestor| {
                    caturra_classfile::exceptions::is_exception_subclass(
                        ancestor,
                        exception_internal(sup),
                    )
                })
        )
        || (from == JType::Null && to.is_reference())
        || (to == JType::Object(table.object_id) && from.is_reference())
        // A LinkedList (or a narrower Queue/Deque face) assigns to a wider face
        // of the same element type: `Queue<E> q = new LinkedList<>()`.
        || matches!(
            (from, to),
            (
                JType::LinkedList { elem: a, role: r },
                JType::LinkedList { elem: b, role: to_role },
            ) if a == b && r.widens_to(to_role)
        )
        // The concrete LinkedList is a List; any of them is a Collection.
        || matches!(
            (from, to),
            (JType::LinkedList { elem: a, role: SeqRole::Full }, JType::List(b)) if a == b
        )
        || matches!(
            (from, to),
            (JType::LinkedList { elem: a, .. }, JType::Collection(b)) if a == b
        )
        // A Stack is a List (it extends Vector), and so a Collection, of its
        // element type: `List<E> l = new Stack<>()`.
        || matches!(
            (from, to),
            (JType::Stack(a), JType::List(b) | JType::Collection(b)) if a == b
        )
        // A TreeSet is a Set (and a Collection) of its element type.
        || matches!(
            (from, to),
            (JType::TreeSet(a), JType::Set(b) | JType::Collection(b)) if a == b
        )
        // A TreeMap is a Map of its key/value types.
        || matches!(
            (from, to),
            (
                JType::TreeMap { key: k1, value: v1 },
                JType::Map { key: k2, value: v2 },
            ) if k1 == k2 && v1 == v2
        )
        // A parameterized type and its raw class erase alike, so they
        // are mutually assignable (`Box<String> b = new Box<>()`).
        || matches!((from.erased_class(), to.erased_class()), (Some(a), Some(b)) if a == b)
        // Any reference (including another type var) stores into a T.
        || (to == JType::TypeVar && from.is_reference())
        || (from == JType::TypeVar && to == JType::Object(table.object_id))
        // Autoboxing / unboxing in assignment and method invocation.
        || matches!((from, to), (JType::Boxed(e), t) if e.base_type() == t)
        || matches!((from, to), (f, JType::Boxed(e)) if e.base_type() == f)
        // Array covariance: `Card[]` widens to `Comparable[]`.
        || matches!(
            (from, to),
            (
                JType::Array { elem: ElemType::Object(sub), dims: d1 },
                JType::Array { elem: ElemType::Object(sup), dims: d2 },
            ) if d1 == d2 && table.is_subtype(sub, sup)
        )
        // Any reference array widens to `Object[]` (e.g. `String[]`,
        // `Field[]` -> `Object[]` for `Arrays.toString`).
        || matches!(
            (from, to),
            (
                JType::Array { elem: ElemType::Str | ElemType::Field | ElemType::Constructor, dims: d1 },
                JType::Array { elem: ElemType::Object(sup), dims: d2 },
            ) if d1 == d2 && sup == table.object_id
        )
        // A multi-dimensional array is an `Object[]` of its rows, whatever
        // the leaf element type: `int[][]` is an `Object[]`, `int[]` is not.
        || matches!(
            (from, to),
            (
                JType::Array { dims: d1, .. },
                JType::Array { elem: ElemType::Object(sup), dims: 1 },
            ) if d1 >= 2 && sup == table.object_id
        )
        // Autoboxing a primitive to the top `Object` type (`Object o = 5;`,
        // or a primitive in an `Object...` varargs).
        || matches!(
            (from, to),
            (
                JType::Int | JType::Long | JType::Double | JType::Float
                    | JType::Short | JType::Byte | JType::Char | JType::Boolean,
                JType::Object(id),
            ) if id == table.object_id
        )
}

/// The element base type of an array (arrays of arrays are expressed
/// via [`JType::Array`]'s `dims`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ElemType {
    Int,
    Double,
    Long,
    Float,
    Short,
    Byte,
    Boolean,
    Char,
    Str,
    /// A user-defined class.
    Object(ClassId),
    /// `java.lang.reflect.Field` (element of `getDeclaredFields()`).
    Field,
    /// `java.lang.reflect.Method` (element of `getDeclaredMethods()`).
    Method,
    /// `java.lang.reflect.Constructor` (element of `getDeclaredConstructors()`).
    Constructor,
    /// `java.lang.Class` (element of a `Class[]`, e.g. `getConstructor` args).
    Class,
}

impl ElemType {
    fn descriptor(self, table: &MethodTable) -> String {
        match self {
            ElemType::Int => String::from("I"),
            ElemType::Double => String::from("D"),
            ElemType::Long => String::from("J"),
            ElemType::Float => String::from("F"),
            ElemType::Short => String::from("S"),
            ElemType::Byte => String::from("B"),
            ElemType::Boolean => String::from("Z"),
            ElemType::Char => String::from("C"),
            ElemType::Str => String::from("Ljava/lang/String;"),
            ElemType::Object(id) => format!("L{};", table.class_name(id)),
            ElemType::Field => String::from("Ljava/lang/reflect/Field;"),
            ElemType::Method => String::from("Ljava/lang/reflect/Method;"),
            ElemType::Constructor => String::from("Ljava/lang/reflect/Constructor;"),
            ElemType::Class => String::from("Ljava/lang/Class;"),
        }
    }

    fn base_type(self) -> JType {
        match self {
            ElemType::Int => JType::Int,
            ElemType::Double => JType::Double,
            ElemType::Long => JType::Long,
            ElemType::Float => JType::Float,
            ElemType::Short => JType::Short,
            ElemType::Byte => JType::Byte,
            ElemType::Boolean => JType::Boolean,
            ElemType::Char => JType::Char,
            ElemType::Str => JType::Str,
            ElemType::Object(id) => JType::Object(id),
            ElemType::Field => JType::Field,
            ElemType::Method => JType::Method,
            ElemType::Constructor => JType::Constructor,
            ElemType::Class => JType::Class,
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
    /// An instance of a user-defined class.
    Object(ClassId),
    /// `long` (64-bit; two JVMS local slots).
    Long,
    /// `float` (32-bit, one slot).
    Float,
    /// `short` (stored as int; matters only for conversions).
    Short,
    /// `byte` (stored as int).
    Byte,
    /// `java.util.Scanner` (intrinsic).
    Scanner,
    /// `java.lang.StringBuilder` (intrinsic).
    StringBuilder,
    /// `java.lang.Class` (reflection intrinsic; from `obj.getClass()`).
    Class,
    /// `java.lang.reflect.Field` (reflection intrinsic).
    Field,
    /// `java.lang.reflect.Method` (reflection intrinsic).
    Method,
    /// `java.lang.reflect.Type` / `ParameterizedType` (from
    /// `Field.getGenericType()`).
    Type,
    /// `java.lang.reflect.Constructor` (reflection intrinsic).
    Constructor,
    /// A library throwable; the id indexes the shared exception table
    /// (`caturra_classfile::exceptions`), 0 = `Throwable` itself.
    Exception(u8),
    /// `java.io.File` (intrinsic, backed by the virtual filesystem).
    File,
    /// `java.io.PrintWriter` (intrinsic, writes into the virtual
    /// filesystem).
    Writer,
    /// `java.util.ArrayList<E>` (intrinsic; E tracked at compile time,
    /// erased at runtime).
    List(ElemType),
    /// `java.util.Stack<E>` — a `Vector`-backed LIFO. It *is* a `List` (extends
    /// `Vector`), so it shares the list method surface and widens to
    /// `List`/`Collection`; distinct from `List` because it adds
    /// `push`/`pop`/`peek`/`empty`/`search` (which act on the top / the end).
    Stack(ElemType),
    /// `java.util.stream.Stream<E>` — an eager pipeline of elements (see the
    /// VM). `map` erases the element to `Object`; `collect` adopts the result
    /// element from the assignment context.
    Stream(ElemType),
    /// A `java.util.stream.Collectors` recipe, the argument to `Stream.collect`.
    Collector,
    /// `java.util.stream.IntStream` — a stream of primitive `int`s (the VM
    /// models it as a `Stream` of unboxed ints). Adds numeric terminals
    /// (`sum`/`toArray`) the object `Stream` lacks.
    IntStream,
    /// `java.util.Optional<E>` — a present-or-absent value; `get` returns `E`.
    Optional(ElemType),
    /// `java.util.OptionalInt` — `getAsInt` returns `int`.
    OptionalInt,
    /// `java.util.OptionalDouble` — `getAsDouble` returns `double`.
    OptionalDouble,
    /// `java.util.LinkedList<E>` and its `Queue`/`Deque` interface views. The
    /// storage is a list; the `role` restricts which methods the receiver
    /// exposes, so `Queue<E>.get(i)` is rejected exactly as javac rejects it.
    LinkedList {
        elem: ElemType,
        role: SeqRole,
    },
    /// `java.util.HashMap<K, V>` / `java.util.Map<K, V>` (intrinsic; K and
    /// V tracked at compile time, erased at runtime).
    Map {
        key: ElemType,
        value: ElemType,
    },
    /// `java.util.Set<E>` — a map's `keySet()` view.
    Set(ElemType),
    /// `java.util.TreeSet<E>` (also its `SortedSet`/`NavigableSet` faces): a
    /// sorted set, backed by an ordered vector. Distinct from `Set` because it
    /// adds the sorted navigation (`first`/`last`/`floor`/`ceiling`/…).
    TreeSet(ElemType),
    /// `java.util.TreeMap<K, V>` (also its `SortedMap`/`NavigableMap` faces): a
    /// sorted map. Distinct from `Map` because it adds the key navigation
    /// (`firstKey`/`lastKey`/`floorKey`/`ceilingKey`/…).
    TreeMap {
        key: ElemType,
        value: ElemType,
    },
    /// `java.util.Collection<E>` — a map's `values()` view.
    Collection(ElemType),
    /// `Set<Map.Entry<K, V>>` — a map's `entrySet()` view.
    EntrySet {
        key: ElemType,
        value: ElemType,
    },
    /// One `java.util.Map.Entry<K, V>`.
    MapEntry {
        key: ElemType,
        value: ElemType,
    },
    /// A parameterized user class with a single tracked type argument
    /// (`Box<String>`). Erases to `Object(class)` at the bytecode
    /// level; the argument enables cast-free reads of type-variable
    /// members.
    Generic {
        class: ClassId,
        arg: ElemType,
    },
    /// The (single) type parameter of a generic class, seen while
    /// compiling that class's own body. Erases to `Object`; at external
    /// use it substitutes to the receiver's tracked type argument.
    TypeVar,
    /// A boxed primitive wrapper (`Integer`, `Double`, ...). The
    /// `ElemType` is the primitive kind (never `Str`/`Object`).
    Boxed(ElemType),
    /// A type caturra doesn't handle yet.
    Unsupported,
    /// A diagnostic was already reported for this subtree.
    Error,
}

/// Which sequence class/face a `JType::LinkedList` receiver presents — the
/// `Queue`/`Deque` interfaces are shared by several concrete classes, so they
/// all flow through this one `JType`. `Full` is the concrete `LinkedList` (every
/// `List`+`Deque` method); `ArrayDeque` is the concrete `java.util.ArrayDeque`
/// (the full `Deque` surface, but NOT a `List`); `Queue`/`Deque` are the
/// narrower interface views a `Queue<E>`/`Deque<E>` variable exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeqRole {
    Full,
    ArrayDeque,
    Queue,
    Deque,
}

impl SeqRole {
    /// The JVM internal name of the type this role presents.
    fn internal(self) -> &'static str {
        match self {
            SeqRole::Full => "java/util/LinkedList",
            SeqRole::ArrayDeque => "java/util/ArrayDeque",
            SeqRole::Queue => "java/util/Queue",
            SeqRole::Deque => "java/util/Deque",
        }
    }

    /// Whether a receiver of this role is assignable to `other`'s role. The
    /// concrete `LinkedList` and `ArrayDeque` are each a `Queue` and a `Deque`
    /// (their interfaces), and a `Deque` is also a `Queue` (`Deque extends
    /// Queue`) — but the two concrete classes are unrelated (neither widens to
    /// the other), and a `Queue` is neither a `Deque` nor a concrete class.
    fn widens_to(self, other: SeqRole) -> bool {
        if self == other {
            return true;
        }
        matches!(
            (self, other),
            (
                SeqRole::Full | SeqRole::ArrayDeque,
                SeqRole::Queue | SeqRole::Deque
            ) | (SeqRole::Deque, SeqRole::Queue)
        )
    }
}

impl JType {
    fn describe(self, table: &MethodTable) -> String {
        match self {
            JType::Object(id) if id == table.object_id => String::from("Object"),
            JType::Boxed(elem) => wrapper_name(elem, table),
            JType::Generic { class, arg } => {
                format!(
                    "{}<{}>",
                    table.class_name(class),
                    arg.base_type().describe(table)
                )
            }
            JType::Map { key, value } => format!(
                "HashMap<{}, {}>",
                key.base_type().describe(table),
                value.base_type().describe(table)
            ),
            JType::TreeMap { key, value } => format!(
                "TreeMap<{}, {}>",
                key.base_type().describe(table),
                value.base_type().describe(table)
            ),
            JType::Set(elem) => format!("Set<{}>", elem.base_type().describe(table)),
            JType::TreeSet(elem) => format!("TreeSet<{}>", elem.base_type().describe(table)),
            JType::Stream(elem) => format!("Stream<{}>", elem.base_type().describe(table)),
            JType::Collector => String::from("Collector"),
            JType::IntStream => String::from("IntStream"),
            JType::Optional(elem) => format!("Optional<{}>", elem.base_type().describe(table)),
            JType::OptionalInt => String::from("OptionalInt"),
            JType::OptionalDouble => String::from("OptionalDouble"),
            JType::LinkedList { elem, role } => {
                let name = match role {
                    SeqRole::Full => "LinkedList",
                    SeqRole::ArrayDeque => "ArrayDeque",
                    SeqRole::Queue => "Queue",
                    SeqRole::Deque => "Deque",
                };
                format!("{name}<{}>", elem.base_type().describe(table))
            }
            JType::Collection(elem) => {
                format!("Collection<{}>", elem.base_type().describe(table))
            }
            JType::EntrySet { key, value } => format!(
                "Set<Map.Entry<{}, {}>>",
                key.base_type().describe(table),
                value.base_type().describe(table)
            ),
            JType::MapEntry { key, value } => format!(
                "Map.Entry<{}, {}>",
                key.base_type().describe(table),
                value.base_type().describe(table)
            ),
            JType::TypeVar => String::from("Object"),
            JType::Int => String::from("int"),
            JType::Double => String::from("double"),
            JType::Boolean => String::from("boolean"),
            JType::Char => String::from("char"),
            JType::Str => String::from("String"),
            JType::Null => String::from("null"),
            JType::Array { elem, dims } => {
                let mut out = elem.base_type().describe(table);
                for _ in 0..dims {
                    out.push_str("[]");
                }
                out
            }
            JType::Object(id) => table.class_name(id).to_owned(),
            JType::Long => String::from("long"),
            JType::Float => String::from("float"),
            JType::Short => String::from("short"),
            JType::Byte => String::from("byte"),
            JType::Scanner => String::from("Scanner"),
            JType::StringBuilder => String::from("StringBuilder"),
            JType::Class => String::from("Class"),
            JType::Field => String::from("Field"),
            JType::Method => String::from("Method"),
            JType::Type => String::from("Type"),
            JType::Constructor => String::from("Constructor"),
            JType::Exception(id) => exception_internal(id)
                .rsplit('/')
                .next()
                .unwrap_or("Throwable")
                .to_owned(),
            JType::File => String::from("File"),
            JType::Writer => String::from("PrintWriter"),
            JType::List(elem) => {
                format!("ArrayList<{}>", wrapper_name(elem, table))
            }
            JType::Stack(elem) => {
                format!("Stack<{}>", wrapper_name(elem, table))
            }
            JType::Unsupported => String::from("an unsupported type"),
            JType::Error => String::from("an unknown type"),
        }
    }

    fn is_numeric(self) -> bool {
        matches!(
            self,
            JType::Int
                | JType::Double
                | JType::Char
                | JType::Long
                | JType::Float
                | JType::Short
                | JType::Byte
        )
    }

    fn is_reference(self) -> bool {
        matches!(
            self,
            JType::Str
                | JType::Null
                | JType::Array { .. }
                | JType::Object(_)
                | JType::Scanner
                | JType::File
                | JType::Writer
                | JType::List(_)
                | JType::Stack(_)
                | JType::LinkedList { .. }
                | JType::Map { .. }
                | JType::TreeMap { .. }
                | JType::Set(_)
                | JType::TreeSet(_)
                | JType::Stream(_)
                | JType::Collector
                | JType::IntStream
                | JType::Optional(_)
                | JType::OptionalInt
                | JType::OptionalDouble
                | JType::Collection(_)
                | JType::EntrySet { .. }
                | JType::MapEntry { .. }
                | JType::Exception(_)
                | JType::Generic { .. }
                | JType::TypeVar
                | JType::Boxed(_)
                | JType::Class
                | JType::Field
                | JType::Method
                | JType::Type
                | JType::StringBuilder
                | JType::Constructor
        )
    }

    /// The erased class id of a reference to a user class or a
    /// parameterized type (`Box<String>` and raw `Box` share one).
    fn erased_class(self) -> Option<ClassId> {
        match self {
            JType::Object(id) | JType::Generic { class: id, .. } => Some(id),
            _ => None,
        }
    }

    /// Width in operand-stack slots (JVMS §2.6.2).
    fn width(self) -> u16 {
        if matches!(self, JType::Double | JType::Long) {
            2
        } else {
            1
        }
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
    fn descriptor(self, table: &MethodTable) -> String {
        match self {
            // Erasure: a parameterized type is its raw class; a type
            // variable is Object.
            JType::Boxed(elem) => format!("L{};", wrapper_internal(elem)),
            JType::Generic { class, .. } => format!("L{};", table.class_name(class)),
            JType::Map { .. } => String::from("Ljava/util/HashMap;"),
            JType::TreeMap { .. } => String::from("Ljava/util/TreeMap;"),
            JType::Set(_) | JType::EntrySet { .. } => String::from("Ljava/util/Set;"),
            JType::TreeSet(_) => String::from("Ljava/util/TreeSet;"),
            JType::Stream(_) => String::from("Ljava/util/stream/Stream;"),
            JType::Collector => String::from("Ljava/util/stream/Collector;"),
            JType::IntStream => String::from("Ljava/util/stream/IntStream;"),
            JType::Optional(_) => String::from("Ljava/util/Optional;"),
            JType::OptionalInt => String::from("Ljava/util/OptionalInt;"),
            JType::OptionalDouble => String::from("Ljava/util/OptionalDouble;"),
            JType::LinkedList { role, .. } => format!("L{};", role.internal()),
            JType::Collection(_) => String::from("Ljava/util/Collection;"),
            JType::MapEntry { .. } => String::from("Ljava/util/Map$Entry;"),
            JType::Int => String::from("I"),
            JType::Double => String::from("D"),
            JType::Boolean => String::from("Z"),
            JType::Char => String::from("C"),
            JType::Str | JType::Null => String::from("Ljava/lang/String;"),
            JType::Array { elem, dims } => {
                let mut out = "[".repeat(usize::from(dims));
                out.push_str(&elem.descriptor(table));
                out
            }
            JType::Object(id) => format!("L{};", table.class_name(id)),
            JType::Long => String::from("J"),
            JType::Float => String::from("F"),
            JType::Short => String::from("S"),
            JType::Byte => String::from("B"),
            JType::Scanner => String::from("Ljava/util/Scanner;"),
            JType::StringBuilder => String::from("Ljava/lang/StringBuilder;"),
            JType::Class => String::from("Ljava/lang/Class;"),
            JType::Field => String::from("Ljava/lang/reflect/Field;"),
            JType::Method => String::from("Ljava/lang/reflect/Method;"),
            JType::Type => String::from("Ljava/lang/reflect/Type;"),
            JType::Constructor => String::from("Ljava/lang/reflect/Constructor;"),
            JType::Exception(id) => format!("L{};", exception_internal(id)),
            JType::File => String::from("Ljava/io/File;"),
            JType::Writer => String::from("Ljava/io/PrintWriter;"),
            JType::List(_) => String::from("Ljava/util/ArrayList;"),
            JType::Stack(_) => String::from("Ljava/util/Stack;"),
            // Only reachable for methods that already produced a
            // diagnostic; the descriptor keeps the class file coherent.
            JType::TypeVar | JType::Unsupported | JType::Error => {
                String::from("Ljava/lang/Object;")
            }
        }
    }
}

/// Binary numeric promotion (JLS §5.6.2), within the CSA type set.
/// The arity, descriptor and return of `Arrays.deepToString`/`deepEquals`/
/// `deepHashCode`, which the VM answers rather than the bundled Java.
fn arrays_deep_signature(method: &str) -> Option<(usize, &'static str, JType)> {
    match method {
        "deepToString" => Some((1, "([Ljava/lang/Object;)Ljava/lang/String;", JType::Str)),
        "deepHashCode" => Some((1, "([Ljava/lang/Object;)I", JType::Int)),
        "deepEquals" => Some((
            2,
            "([Ljava/lang/Object;[Ljava/lang/Object;)Z",
            JType::Boolean,
        )),
        _ => None,
    }
}

/// The `java.util.Collections` helpers caturra models, whether in the bundled
/// Java or in the VM.
fn is_collections_method(method: &str) -> bool {
    matches!(
        method,
        "reverse"
            | "swap"
            | "sort"
            | "shuffle"
            | "max"
            | "min"
            | "frequency"
            | "nCopies"
            | "binarySearch"
            | "addAll"
            | "unmodifiableList"
            | "emptyList"
            | "singletonList"
            | "reverseOrder"
            | "emptySet"
            | "emptyMap"
            | "singleton"
            | "singletonMap"
            | "unmodifiableSet"
            | "unmodifiableMap"
    )
}

/// The four `Arrays` methods the VM answers over any array kind.
fn is_arrays_array_method(method: &str) -> bool {
    matches!(method, "copyOf" | "copyOfRange" | "fill" | "binarySearch")
}

/// Whether a type is an `Object[]`: a multi-dimensional array (whose elements
/// are arrays, and so references) or a one-dimensional array of references.
fn is_reference_array(ty: JType) -> bool {
    match ty {
        JType::Array { dims, .. } if dims >= 2 => true,
        JType::Array { elem, .. } => elem.base_type().is_reference(),
        // `Arrays.deepToString(null)` is legal.
        JType::Null => true,
        _ => false,
    }
}

fn promote(a: JType, b: JType) -> JType {
    if a == JType::Double || b == JType::Double {
        JType::Double
    } else if a == JType::Float || b == JType::Float {
        JType::Float
    } else if a == JType::Long || b == JType::Long {
        JType::Long
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
        TypeRef::Long => Some(JType::Long),
        TypeRef::Float => Some(JType::Float),
        TypeRef::Short => Some(JType::Short),
        TypeRef::Byte => Some(JType::Byte),
        TypeRef::Named(name) if name == "String" => Some(JType::Str),
        TypeRef::Named(name) if name == "Class" => Some(JType::Class),
        TypeRef::Named(name) if name == "Field" => Some(JType::Field),
        TypeRef::Named(name) if name == "Method" => Some(JType::Method),
        TypeRef::Named(name) if name == "Type" || name == "ParameterizedType" => Some(JType::Type),
        TypeRef::Named(name) if name == "Constructor" => Some(JType::Constructor),
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
                TypeRef::Named(name) if name == "Field" => ElemType::Field,
                TypeRef::Named(name) if name == "Constructor" => ElemType::Constructor,
                _ => return None,
            };
            Some(JType::Array { elem, dims })
        }
        _ => None,
    }
}

/// A compile-time constant int/char case label (literals, optionally
/// negated).
fn constant_int_value(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Literal {
            value: Literal::Int(v),
            ..
        } => Some(*v),
        Expr::Literal {
            value: Literal::Char(c),
            ..
        } => Some(i64::from(u32::from(*c))),
        Expr::Unary {
            op: UnaryOp::Neg,
            operand,
            ..
        } => constant_int_value(operand).map(|v| -v),
        _ => None,
    }
}

/// The span a statement's line marker should use (`None` for blocks —
/// their inner statements mark themselves).
fn statement_span(stmt: &Stmt) -> Option<SourceSpan> {
    match stmt {
        Stmt::Block(_) => None,
        Stmt::LocalDecl { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::ForEach { span, .. }
        | Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::DoWhile { span, .. }
        | Stmt::For { span, .. }
        | Stmt::Break { span, .. }
        | Stmt::Continue { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::SuperCall { span, .. }
        | Stmt::ThisCall { span, .. }
        | Stmt::Try { span, .. }
        | Stmt::Throw { span, .. }
        | Stmt::Switch { span, .. }
        | Stmt::Labeled { span, .. } => Some(*span),
        Stmt::Expr(expr) => Some(expr.span()),
    }
}

struct LocalVar {
    slot: u16,
    ty: JType,
    is_final: bool,
    assigned: bool,
}

#[allow(clippy::too_many_lines)] // ctor chaining preamble is cohesive
fn emit_method(
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    table: &MethodTable,
    class_decl: &ClassDecl,
    pool: &mut ConstantPool,
    decl: &MethodDecl,
) -> MethodInfo {
    let class_id = table.class_id(&class_decl.name).expect("class registered");
    let return_type = match &decl.return_type {
        TypeRef::Void => None,
        other => Some(table.resolve_type(other).unwrap_or(JType::Unsupported)),
    };
    let mut body = BodyGen {
        path,
        diagnostics,
        pool,
        table,
        current_class: &class_decl.name,
        current_class_id: class_id,
        in_static: decl.is_static,
        in_constructor: decl.is_constructor,
        return_type,
        code: CodeBuilder::new(),
        scopes: vec![Vec::new()],
        next_slot: u16::from(!decl.is_static),
        loop_stack: Vec::new(),
        pending_label: None,
        finally_stack: Vec::new(),
        local_var_debug: Vec::new(),
    };

    if !decl.is_static {
        body.record_local_debug("this", JType::Object(class_id), 0);
    }
    for param in &decl.params {
        let ty = table.resolve_type(&param.ty).unwrap_or(JType::Unsupported);
        let slot = body.next_slot;
        body.next_slot += ty.width();
        body.record_local_debug(&param.name, ty, slot);
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

    // Abstract / interface methods have no body and no Code attribute.
    if decl.is_abstract {
        let descriptor = method_descriptor(path, diagnostics, table, decl);
        let mut flags = MethodAccessFlags::ABSTRACT;
        if decl.is_public || class_decl.is_interface {
            flags |= MethodAccessFlags::PUBLIC;
        }
        let name_index = pool.intern_utf8(&decl.name);
        let descriptor_index = pool.intern_utf8(&descriptor);
        return MethodInfo {
            access_flags: MethodAccessFlags(flags),
            name_index,
            descriptor_index,
            attributes: Vec::new(),
        };
    }

    let mut statements: &[Stmt] = &decl.body;
    if decl.is_constructor {
        // Chaining (JLS §12.5): an explicit super(...)/this(...) must be
        // first; otherwise the implicit super() runs. Field
        // initializers run only on the super path.
        let explicit = statements.first().and_then(|stmt| match stmt {
            Stmt::SuperCall { args, span } => Some((true, args, *span)),
            Stmt::ThisCall { args, span } => Some((false, args, *span)),
            _ => None,
        });
        if let Some((is_super, args, span)) = explicit {
            statements = &statements[1..];
            if is_super {
                body.emit_constructor_call_on_this(
                    class_decl
                        .superclass
                        .as_deref()
                        .unwrap_or("java/lang/Object"),
                    args,
                    span,
                );
                body.emit_instance_field_initializers(class_decl);
            } else {
                let current = class_decl.name.clone();
                body.emit_constructor_call_on_this(&current, args, span);
                // Delegated constructor already ran the field inits.
            }
        } else {
            // An anonymous class implementing an interface extends
            // Object; one extending a class extends that class.
            let anon_super = class_decl
                .is_anonymous
                .then(|| table.anon_super_name(&class_decl.name));
            let super_name = anon_super.as_deref().unwrap_or_else(|| {
                class_decl
                    .superclass
                    .as_deref()
                    .unwrap_or("java/lang/Object")
            });
            body.emit_constructor_call_on_this(super_name, &[], decl.span);
            body.emit_instance_field_initializers(class_decl);
        }
    }

    for stmt in statements {
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
    let local_var_debug = std::mem::take(&mut body.local_var_debug);
    let (bytecode, max_stack, line_numbers, exception_table) = body.code.finish();

    let descriptor = method_descriptor(path, diagnostics, table, decl);
    let mut flags = 0;
    if decl.is_public || (class_decl.is_interface && !decl.is_private) {
        flags |= MethodAccessFlags::PUBLIC;
    }
    if decl.is_private {
        flags |= MethodAccessFlags::PRIVATE;
    }
    if decl.is_static {
        flags |= MethodAccessFlags::STATIC;
    }
    let jvm_name = if decl.is_constructor {
        "<init>"
    } else {
        &decl.name
    };
    finish_method_info(
        pool,
        jvm_name,
        &descriptor,
        flags,
        max_stack,
        max_locals,
        bytecode,
        &line_numbers,
        &local_var_debug,
        exception_table,
    )
}

#[allow(clippy::too_many_lines)] // one type-descriptor matcher
/// The return type of a wrapper instance method (`intValue`, `compareTo`, …),
/// mirroring `boxed_instance_call` for `type_of`.
fn boxed_method_return(method: &str) -> Option<JType> {
    Some(match method {
        "intValue" | "hashCode" | "compareTo" => JType::Int,
        "shortValue" => JType::Short,
        "byteValue" => JType::Byte,
        "longValue" => JType::Long,
        "doubleValue" => JType::Double,
        "floatValue" => JType::Float,
        "charValue" => JType::Char,
        "booleanValue" | "equals" => JType::Boolean,
        "toString" => JType::Str,
        _ => return None,
    })
}

/// The primitive descriptor char for a wrapper class name (`Integer` → `I`),
/// for wrapper arrays which store their primitives directly.
fn wrapper_primitive_char(name: &str) -> Option<char> {
    match crate::imports::canonical_library_class(name).unwrap_or(name) {
        "Integer" => Some('I'),
        "Double" => Some('D'),
        "Long" => Some('J'),
        "Float" => Some('F'),
        "Short" => Some('S'),
        "Byte" => Some('B'),
        "Character" => Some('C'),
        "Boolean" => Some('Z'),
        _ => None,
    }
}

#[allow(clippy::too_many_lines)] // one descriptor builder with a type-mapping matrix
fn method_descriptor(
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
    table: &MethodTable,
    decl: &MethodDecl,
) -> String {
    fn push_type(
        path: &str,
        diagnostics: &mut Vec<Diagnostic>,
        table: &MethodTable,
        out: &mut String,
        ty: &TypeRef,
        span: SourceSpan,
    ) {
        match ty {
            TypeRef::Void => out.push('V'),
            TypeRef::Int => out.push('I'),
            TypeRef::Double => out.push('D'),
            TypeRef::Boolean => out.push('Z'),
            TypeRef::Long => out.push('J'),
            TypeRef::Float => out.push('F'),
            TypeRef::Short => out.push('S'),
            TypeRef::Byte => out.push('B'),
            TypeRef::Char => out.push('C'),
            TypeRef::Array(inner) => {
                out.push('[');
                // A wrapper array (`Integer[]`) stores its primitives directly,
                // like the corresponding primitive array (matches resolve_type).
                if let TypeRef::Named(name) = inner.as_ref()
                    && !table.has_class(name)
                    && let Some(prim) = wrapper_primitive_char(name)
                {
                    out.push(prim);
                } else {
                    push_type(path, diagnostics, table, out, inner, span);
                }
            }
            TypeRef::Generic { base, .. } => {
                let mut simple =
                    crate::imports::canonical_library_class(base).unwrap_or(base.as_str());
                if simple == "List" && !table.has_class("List") {
                    simple = "ArrayList";
                }
                if simple == "Map" && !table.has_class("Map") {
                    simple = "HashMap";
                }
                if simple == "HashSet" && !table.has_class("HashSet") {
                    simple = "Set";
                }
                if matches!(
                    simple,
                    "LinkedList"
                        | "Queue"
                        | "Deque"
                        | "PriorityQueue"
                        | "ArrayDeque"
                        | "Stack"
                        | "Optional"
                ) && !table.has_class(simple)
                {
                    out.push_str("Ljava/util/");
                    out.push_str(simple);
                    out.push(';');
                } else if simple == "ArrayList" {
                    out.push_str("Ljava/util/ArrayList;");
                } else if simple == "HashMap" {
                    out.push_str("Ljava/util/HashMap;");
                } else if matches!(simple, "TreeMap" | "SortedMap" | "NavigableMap")
                    && !table.has_class(simple)
                {
                    out.push_str("Ljava/util/TreeMap;");
                } else if simple == "Set" && !table.has_class(simple) {
                    out.push_str("Ljava/util/Set;");
                } else if matches!(simple, "TreeSet" | "SortedSet" | "NavigableSet")
                    && !table.has_class(simple)
                {
                    out.push_str("Ljava/util/TreeSet;");
                } else if simple == "Collection" && !table.has_class(simple) {
                    out.push_str("Ljava/util/Collection;");
                } else if matches!(simple, "Map.Entry" | "Entry") && !table.has_class(simple) {
                    out.push_str("Ljava/util/Map$Entry;");
                } else if !table.has_class(base) && simple == "Class" {
                    out.push_str("Ljava/lang/Class;");
                } else if !table.has_class(base) && simple == "Constructor" {
                    out.push_str("Ljava/lang/reflect/Constructor;");
                } else if !table.has_class(base) && simple == "Field" {
                    out.push_str("Ljava/lang/reflect/Field;");
                } else {
                    let message = crate::imports::unsupported_class_reason(base)
                        .filter(|_| !table.has_class(base))
                        .unwrap_or_else(|| format!("unknown generic type '{base}'"));
                    diagnostics.push(Diagnostic::error(path, message, span));
                    out.push_str("Ljava/lang/Object;");
                }
            }
            TypeRef::Named(name) => {
                let simple = crate::imports::canonical_library_class(name).unwrap_or(name.as_str());
                if simple == "String" {
                    out.push_str("Ljava/lang/String;");
                } else if simple == "StringBuilder" && !table.has_class(simple) {
                    out.push_str("Ljava/lang/StringBuilder;");
                } else if simple == "Scanner" && !table.has_class(simple) {
                    out.push_str("Ljava/util/Scanner;");
                } else if simple == "File" && !table.has_class(simple) {
                    out.push_str("Ljava/io/File;");
                } else if simple == "PrintWriter" && !table.has_class(simple) {
                    out.push_str("Ljava/io/PrintWriter;");
                } else if simple == "Class" && !table.has_class(simple) {
                    out.push_str("Ljava/lang/Class;");
                } else if simple == "Field" && !table.has_class(simple) {
                    out.push_str("Ljava/lang/reflect/Field;");
                } else if simple == "Method" && !table.has_class(simple) {
                    out.push_str("Ljava/lang/reflect/Method;");
                } else if simple == "Constructor" && !table.has_class(simple) {
                    out.push_str("Ljava/lang/reflect/Constructor;");
                } else if (simple == "Type" || simple == "ParameterizedType")
                    && !table.has_class(simple)
                {
                    out.push_str("Ljava/lang/reflect/Type;");
                } else if name == crate::parser::TYPEVAR_SENTINEL
                    || name == "Object"
                    || name == "java.lang.Object"
                {
                    // Erasure: a type variable / Object both descriptor
                    // to java/lang/Object.
                    out.push_str("Ljava/lang/Object;");
                } else if !table.has_class(simple)
                    && let Some(elem) = wrapper_elem(simple)
                {
                    out.push('L');
                    out.push_str(wrapper_internal(elem));
                    out.push(';');
                } else if (!name.contains('.')
                    || crate::imports::canonical_library_class(name).is_some())
                    && table.has_class(simple)
                {
                    out.push('L');
                    out.push_str(simple);
                    out.push(';');
                } else {
                    let message = if name.contains('.') {
                        crate::imports::unknown_qualified_message(name)
                    } else {
                        format!("unknown type '{name}'")
                    };
                    diagnostics.push(Diagnostic::error(path, message, span));
                    out.push_str("Ljava/lang/Object;");
                }
            }
        }
    }

    let mut descriptor = String::from("(");
    for param in &decl.params {
        push_type(
            path,
            diagnostics,
            table,
            &mut descriptor,
            &param.ty,
            decl.span,
        );
    }
    descriptor.push(')');
    push_type(
        path,
        diagnostics,
        table,
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
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Throw { .. } => {
            false
        }
        // JLS: a try statement completes normally iff (the try block or
        // at least one catch block can complete normally) and the
        // finally block, if any, can complete normally.
        // A switch completes normally unless a default exists and every
        // path ends abruptly; approximate: no default, any break, or
        // the final arm falling out all mean normal completion.
        Stmt::Switch { arms, .. } => {
            let has_default = arms
                .iter()
                .any(|arm| arm.labels.iter().any(Option::is_none));
            !has_default
                || arms
                    .last()
                    .is_some_and(|arm| block_completes_normally(&arm.body))
                || arms.iter().any(|arm| arm.body.iter().any(has_direct_break))
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            (block_completes_normally(body)
                || catches
                    .iter()
                    .any(|clause| block_completes_normally(&clause.body)))
                && finally_body
                    .as_ref()
                    .is_none_or(|stmts| block_completes_normally(stmts))
        }
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

/// A parameter of an intrinsic method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BParam {
    Int,
    Double,
    Long,
    Float,
    Short,
    Byte,
    Boolean,
    Char,
    Str,
    /// `char[]`.
    CharArray,
    /// The receiver's own list type (`addAll(otherList)`).
    SelfList,
    /// Any collection whose element type is assignable to the receiver's
    /// (`set.addAll(aList)`, `set.retainAll(anotherSet)`).
    SelfCollection,
    /// The list's element type (autoboxed at the boundary).
    Elem,
    /// `java.lang.Class` (`Class.isAssignableFrom(Class)`).
    Class,
    /// Any reference array (`getConstructor(Class[])`, `newInstance(Object[])`).
    RefArray,
    /// `java.lang.Object` (`Field.get(Object)`, `Method.invoke(Object, ...)`).
    Object,
    /// The erased target type of a `Map.forEach` lambda: a class that
    /// implements the bundled `__BiConsumer`. Only the lambda desugaring
    /// synthesizes one, so `map.forEach(anythingElse)` is refused.
    BiConsumer,
    /// `list.forEach`'s erased `__Consumer`, `list.removeIf`'s
    /// `__Predicate`, and `list.replaceAll`'s `__UnaryOperator` — likewise
    /// only produced by lambda desugaring.
    Consumer,
    Predicate,
    UnaryOperator,
    /// `Optional.orElseGet`'s erased `__Supplier` (a zero-argument lambda).
    Supplier,
    /// `list.sort`/`Collections.sort`'s erased `__Comparator`.
    Comparator,
    /// `java.lang.StringBuilder` (`StringBuilder.compareTo(StringBuilder)`).
    Builder,
    /// A map's key type, boxed when primitive (`map.get(k)`).
    Key,
    /// A map's value type, boxed when primitive (`map.put(k, v)`).
    Val,
    /// The receiver's own map type (`putAll(otherMap)`).
    SelfMap,
    /// A `Collector` (`Stream.collect(Collectors.toList())`).
    Collector,
}

/// The return of an intrinsic method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BRet {
    Void,
    /// The `java.io.PrintWriter` itself — `append`/`format` return the writer
    /// for chaining.
    Writer,
    Int,
    Double,
    Long,
    Float,
    Short,
    Byte,
    Boolean,
    Char,
    /// `Stream<E>` of the receiver's element type (an element-preserving op).
    Stream,
    /// `Stream<Object>` — an op (`map`) whose element type is erased.
    StreamErased,
    /// `IntStream` (`mapToInt`, and the `IntStream` intermediate ops).
    IntStream,
    /// `Stream<Integer>` — `IntStream.boxed()`.
    StreamInteger,
    /// `int[]` — `IntStream.toArray()`.
    IntArray,
    /// `Optional<E>` of the receiver's element (`findFirst`, `max`, `min`).
    Optional,
    /// `Optional.map` — an `Optional` whose element is erased to `Object`.
    OptionalErased,
    /// `OptionalInt` (`IntStream.max`/`min`).
    OptionalInt,
    /// `OptionalDouble` (`IntStream.average`).
    OptionalDouble,
    /// `Collector` (a `Collectors.toX()` factory result).
    Collector,
    /// A `Comparator` (the `Comparator` static factories / combinators) — the
    /// bundled `__Comparator` interface type.
    Comparator,
    /// `null`-typed, adopting the assignment context — for `collect`, whose
    /// element type comes from the left-hand side (like a diamond `new`).
    Nullish,
    Str,
    /// `String[]` (e.g. `String.split`).
    StrArray,
    /// `char[]` (e.g. `String.toCharArray`).
    CharArray,
    /// The list's element type.
    Elem,
    /// The element type, boxed when primitive — for a `Queue`/`Deque` return
    /// that is `null` on an empty collection (`poll`/`peek`), so the absence
    /// is representable exactly as Java's is.
    BoxedElem,
    /// `java.lang.Class` (`getClass`, `getSuperclass`).
    Class,
    /// `Field[]` (`Class.getDeclaredFields`).
    FieldArray,
    /// `Class[]` (`Method.getParameterTypes`).
    ClassArray,
    /// `Method[]` (`Class.getDeclaredMethods`).
    MethodArray,
    /// `Constructor[]` (`Class.getDeclaredConstructors`).
    ConstructorArray,
    /// A single `java.lang.reflect.Constructor` (`Class.getConstructor`).
    Constructor,
    /// A single `java.lang.reflect.Field` (`Class.getDeclaredField`).
    Field,
    /// A single `java.lang.reflect.Method` (`Class.getMethod`).
    Method,
    /// A `java.lang.reflect.Type` (`Field.getGenericType`).
    Type,
    /// `Object[]` (`ParameterizedType.getActualTypeArguments`, typed loosely).
    ObjectArray,
    /// `java.lang.StringBuilder` (`StringBuilder.append`, for chaining).
    Builder,
    /// `java.lang.Object` (`Constructor.newInstance`).
    Object,
    /// A map's value type, boxed when primitive (`map.get(k)` returns
    /// `null` for a missing key, so it cannot be a bare primitive).
    Val,
    /// A map's key type, boxed when primitive (`entry.getKey()`).
    Key,
    /// `Set<K>` (`map.keySet()`).
    Keys,
    /// `Collection<V>` (`map.values()`).
    Values,
    /// `Set<Map.Entry<K, V>>` (`map.entrySet()`).
    Entries,
}

/// One intrinsic method signature the compiler knows about.
struct BuiltinMethod {
    name: &'static str,
    params: &'static [BParam],
    ret: BRet,
    descriptor: &'static str,
}

/// Compact [`BuiltinMethod`] constructor for the big tables.
const fn bm(
    name: &'static str,
    params: &'static [BParam],
    ret: BRet,
    descriptor: &'static str,
) -> BuiltinMethod {
    BuiltinMethod {
        name,
        params,
        ret,
        descriptor,
    }
}

const D: BParam = BParam::Double;
const L: BParam = BParam::Long;
const F: BParam = BParam::Float;
const I: BParam = BParam::Int;
const C: BParam = BParam::Char;
const Z: BParam = BParam::Boolean;
const S: BParam = BParam::Str;

const STRING_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "length",
        params: &[],
        ret: BRet::Int,
        descriptor: "()I",
    },
    BuiltinMethod {
        name: "isEmpty",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "charAt",
        params: &[BParam::Int],
        ret: BRet::Char,
        descriptor: "(I)C",
    },
    BuiltinMethod {
        name: "substring",
        params: &[BParam::Int],
        ret: BRet::Str,
        descriptor: "(I)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "substring",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Str,
        descriptor: "(II)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "indexOf",
        params: &[BParam::Str],
        ret: BRet::Int,
        descriptor: "(Ljava/lang/String;)I",
    },
    BuiltinMethod {
        name: "equals",
        params: &[BParam::Str],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/Object;)Z",
    },
    BuiltinMethod {
        name: "equalsIgnoreCase",
        params: &[BParam::Str],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/String;)Z",
    },
    BuiltinMethod {
        name: "compareTo",
        params: &[BParam::Str],
        ret: BRet::Int,
        descriptor: "(Ljava/lang/String;)I",
    },
    BuiltinMethod {
        name: "contains",
        params: &[BParam::Str],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/CharSequence;)Z",
    },
    BuiltinMethod {
        name: "startsWith",
        params: &[BParam::Str],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/String;)Z",
    },
    BuiltinMethod {
        name: "endsWith",
        params: &[BParam::Str],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/String;)Z",
    },
    BuiltinMethod {
        name: "split",
        params: &[BParam::Str],
        ret: BRet::StrArray,
        descriptor: "(Ljava/lang/String;)[Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "replace",
        params: &[BParam::Char, BParam::Char],
        ret: BRet::Str,
        descriptor: "(CC)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "replace",
        params: &[BParam::Str, BParam::Str],
        ret: BRet::Str,
        descriptor: "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "toUpperCase",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "toLowerCase",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "trim",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "strip",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "stripLeading",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "stripTrailing",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "isBlank",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "repeat",
        params: &[BParam::Int],
        ret: BRet::Str,
        descriptor: "(I)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "concat",
        params: &[BParam::Str],
        ret: BRet::Str,
        descriptor: "(Ljava/lang/String;)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "compareToIgnoreCase",
        params: &[BParam::Str],
        ret: BRet::Int,
        descriptor: "(Ljava/lang/String;)I",
    },
    BuiltinMethod {
        name: "contentEquals",
        params: &[BParam::Str],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/CharSequence;)Z",
    },
    BuiltinMethod {
        name: "hashCode",
        params: &[],
        ret: BRet::Int,
        descriptor: "()I",
    },
    BuiltinMethod {
        name: "indexOf",
        params: &[BParam::Int],
        ret: BRet::Int,
        descriptor: "(I)I",
    },
    BuiltinMethod {
        name: "indexOf",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Int,
        descriptor: "(II)I",
    },
    BuiltinMethod {
        name: "indexOf",
        params: &[BParam::Str, BParam::Int],
        ret: BRet::Int,
        descriptor: "(Ljava/lang/String;I)I",
    },
    BuiltinMethod {
        name: "lastIndexOf",
        params: &[BParam::Int],
        ret: BRet::Int,
        descriptor: "(I)I",
    },
    BuiltinMethod {
        name: "lastIndexOf",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Int,
        descriptor: "(II)I",
    },
    BuiltinMethod {
        name: "lastIndexOf",
        params: &[BParam::Str],
        ret: BRet::Int,
        descriptor: "(Ljava/lang/String;)I",
    },
    BuiltinMethod {
        name: "lastIndexOf",
        params: &[BParam::Str, BParam::Int],
        ret: BRet::Int,
        descriptor: "(Ljava/lang/String;I)I",
    },
    BuiltinMethod {
        name: "split",
        params: &[BParam::Str, BParam::Int],
        ret: BRet::StrArray,
        descriptor: "(Ljava/lang/String;I)[Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "startsWith",
        params: &[BParam::Str, BParam::Int],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/String;I)Z",
    },
    BuiltinMethod {
        name: "subSequence",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Str,
        descriptor: "(II)Ljava/lang/CharSequence;",
    },
    BuiltinMethod {
        name: "toCharArray",
        params: &[],
        ret: BRet::CharArray,
        descriptor: "()[C",
    },
    BuiltinMethod {
        name: "getChars",
        params: &[BParam::Int, BParam::Int, BParam::CharArray, BParam::Int],
        ret: BRet::Void,
        descriptor: "(II[CI)V",
    },
    BuiltinMethod {
        name: "toString",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "intern",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "codePointAt",
        params: &[BParam::Int],
        ret: BRet::Int,
        descriptor: "(I)I",
    },
    BuiltinMethod {
        name: "codePointBefore",
        params: &[BParam::Int],
        ret: BRet::Int,
        descriptor: "(I)I",
    },
    BuiltinMethod {
        name: "codePointCount",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Int,
        descriptor: "(II)I",
    },
    BuiltinMethod {
        name: "offsetByCodePoints",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Int,
        descriptor: "(II)I",
    },
];

/// `String` static methods (`String.valueOf(...)`).
const STRING_STATIC_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "valueOf",
        params: &[BParam::Int],
        ret: BRet::Str,
        descriptor: "(I)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "valueOf",
        params: &[BParam::Long],
        ret: BRet::Str,
        descriptor: "(J)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "valueOf",
        params: &[BParam::Float],
        ret: BRet::Str,
        descriptor: "(F)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "valueOf",
        params: &[BParam::Double],
        ret: BRet::Str,
        descriptor: "(D)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "valueOf",
        params: &[BParam::Char],
        ret: BRet::Str,
        descriptor: "(C)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "valueOf",
        params: &[BParam::Boolean],
        ret: BRet::Str,
        descriptor: "(Z)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "valueOf",
        params: &[BParam::CharArray],
        ret: BRet::Str,
        descriptor: "([C)Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "copyValueOf",
        params: &[BParam::CharArray],
        ret: BRet::Str,
        descriptor: "([C)Ljava/lang/String;",
    },
];

/// Real Java 11 members caturra cannot model, per class, with the honest
/// reason — students find these in documentation, and "cannot find
/// symbol" would read as a bug.
#[rustfmt::skip]
const UNSUPPORTED_MEMBERS: &[(&str, &str, &str)] = &[
    ("String", "matches", "regular expressions are not supported by caturra"),
    ("String", "replaceAll", "regular expressions are not supported by caturra"),
    ("String", "replaceFirst", "regular expressions are not supported by caturra"),
    ("String", "getBytes", "byte arrays are not supported by caturra"),
    ("String", "chars", "streams are not supported by caturra"),
    ("String", "codePoints", "streams are not supported by caturra"),
    ("String", "lines", "streams are not supported by caturra"),
    ("String", "join", "varargs are not supported by caturra"),
    ("StringBuilder", "capacity", "caturra does not model a builder's capacity, only its contents"),
    ("StringBuilder", "chars", "streams are not supported by caturra"),
    ("StringBuilder", "codePoints", "streams are not supported by caturra"),
    ("Integer", "decode", "system properties are not supported by caturra"),
    ("Integer", "getInteger", "system properties are not supported by caturra"),
    ("ArrayList", "iterator", "iterators are not supported by caturra (use for-each or an index loop)"),
    ("ArrayList", "listIterator", "iterators are not supported by caturra (use for-each or an index loop)"),
    ("ArrayList", "parallelStream", "streams are not supported by caturra"),
    ("ArrayList", "toArray", "Object arrays are not supported by caturra"),
    ("ArrayList", "subList", "list views are not supported by caturra"),
    ("ArrayList", "clone", "clone is not supported by caturra"),
    ("Scanner", "useDelimiter", "regular expressions are not supported by caturra"),
    ("Scanner", "findInLine", "regular expressions are not supported by caturra"),
    ("Scanner", "findWithinHorizon", "regular expressions are not supported by caturra"),
    ("Scanner", "skip", "regular expressions are not supported by caturra"),
    ("Scanner", "tokens", "streams are not supported by caturra"),
    ("Scanner", "findAll", "streams are not supported by caturra"),
    ("Scanner", "nextBigInteger", "BigInteger is not supported by caturra"),
    ("Scanner", "nextBigDecimal", "BigDecimal is not supported by caturra"),
    ("HashMap", "replaceAll", "lambdas are not supported by caturra"),
    ("HashMap", "compute", "lambdas are not supported by caturra"),
    ("HashMap", "computeIfAbsent", "lambdas are not supported by caturra"),
    ("HashMap", "computeIfPresent", "lambdas are not supported by caturra"),
    ("HashMap", "merge", "lambdas are not supported by caturra"),
    ("HashMap", "clone", "clone is not supported by caturra"),
    ("HashMap", "of", "varargs are not supported by caturra"),
    ("HashMap", "ofEntries", "varargs are not supported by caturra"),
    ("Set", "iterator", "iterators are not supported by caturra (use for-each)"),
    ("Set", "removeIf", "Set.removeIf is not supported by caturra"),
    ("TreeMap", "clone", "clone is not supported by caturra"),
    ("TreeMap", "headMap", "TreeMap range views are not supported by caturra"),
    ("TreeMap", "tailMap", "TreeMap range views are not supported by caturra"),
    ("TreeMap", "subMap", "TreeMap range views are not supported by caturra"),
    ("TreeMap", "descendingMap", "TreeMap.descendingMap is not supported by caturra"),
    ("TreeMap", "firstEntry", "TreeMap entry views are not supported by caturra (use firstKey)"),
    ("TreeMap", "lastEntry", "TreeMap entry views are not supported by caturra (use lastKey)"),
    ("TreeMap", "pollFirstEntry", "TreeMap entry views are not supported by caturra"),
    ("TreeMap", "pollLastEntry", "TreeMap entry views are not supported by caturra"),
    ("TreeSet", "iterator", "iterators are not supported by caturra (use for-each)"),
    ("TreeSet", "descendingIterator", "iterators are not supported by caturra"),
    ("TreeSet", "descendingSet", "TreeSet.descendingSet is not supported by caturra"),
    ("TreeSet", "removeIf", "TreeSet.removeIf is not supported by caturra"),
    ("TreeSet", "headSet", "TreeSet range views are not supported by caturra"),
    ("TreeSet", "tailSet", "TreeSet range views are not supported by caturra"),
    ("TreeSet", "subSet", "TreeSet range views are not supported by caturra"),
    ("LinkedList", "iterator", "iterators are not supported by caturra (use for-each or an index loop)"),
    ("LinkedList", "listIterator", "iterators are not supported by caturra (use for-each or an index loop)"),
    ("LinkedList", "descendingIterator", "iterators are not supported by caturra"),
    ("LinkedList", "removeIf", "LinkedList.removeIf is not supported by caturra"),
    ("LinkedList", "toArray", "Object arrays are not supported by caturra"),
    ("Collection", "iterator", "iterators are not supported by caturra (use for-each)"),
    ("Collection", "removeIf", "lambdas are not supported by caturra"),
    ("Collection", "add", "a map's values() does not support add — Java throws UnsupportedOperationException"),
    ("Collection", "remove", "removing through a map's view is not supported by caturra (remove from the map itself)"),
    ("Collection", "clear", "clearing through a map's view is not supported by caturra (clear the map itself)"),
];

/// The source-level class name of a receiver that [`UNSUPPORTED_MEMBERS`]
/// has entries for. Empty for every other receiver, so that (say) a `File`
/// method never matches a `String` entry of the same name.
fn receiver_class_name(receiver: JType) -> &'static str {
    match receiver {
        JType::Str => "String",
        JType::Scanner => "Scanner",
        JType::List(_) => "ArrayList",
        JType::Map { .. } => "HashMap",
        JType::TreeMap { .. } => "TreeMap",
        JType::Set(_) | JType::EntrySet { .. } => "Set",
        JType::TreeSet(_) => "TreeSet",
        JType::LinkedList { .. } => "LinkedList",
        JType::Collection(_) => "Collection",
        JType::MapEntry { .. } => "Map.Entry",
        JType::StringBuilder => "StringBuilder",
        _ => "",
    }
}

/// The honest not-supported reason for a class member, if known.
fn unsupported_member(class: &str, method: &str) -> Option<&'static str> {
    UNSUPPORTED_MEMBERS
        .iter()
        .find(|(c, m, _)| *c == class && *m == method)
        .map(|(_, _, reason)| *reason)
}

const SCANNER_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "nextInt",
        params: &[],
        ret: BRet::Int,
        descriptor: "()I",
    },
    BuiltinMethod {
        name: "nextDouble",
        params: &[],
        ret: BRet::Double,
        descriptor: "()D",
    },
    BuiltinMethod {
        name: "next",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "nextLine",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "hasNext",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "hasNextInt",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "hasNextDouble",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "hasNextLine",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    bm("nextLong", &[], BRet::Long, "()J"),
    bm("nextFloat", &[], BRet::Float, "()F"),
    bm("nextShort", &[], BRet::Short, "()S"),
    bm("hasNextShort", &[], BRet::Boolean, "()Z"),
    bm("nextByte", &[], BRet::Byte, "()B"),
    bm("hasNextByte", &[], BRet::Boolean, "()Z"),
    bm("hasNextFloat", &[], BRet::Boolean, "()Z"),
    bm("hasNextLong", &[], BRet::Boolean, "()Z"),
    bm("nextBoolean", &[], BRet::Boolean, "()Z"),
    bm("hasNextBoolean", &[], BRet::Boolean, "()Z"),
    bm("close", &[], BRet::Void, "()V"),
];

const LIST_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "size",
        params: &[],
        ret: BRet::Int,
        descriptor: "()I",
    },
    // `forEach(Consumer)` walks the list; `removeIf(Predicate)` walks it and
    // drops elements the predicate accepts, returning whether any went.
    BuiltinMethod {
        name: "forEach",
        params: &[BParam::Consumer],
        ret: BRet::Void,
        descriptor: "(Ljava/lang/Object;)V",
    },
    BuiltinMethod {
        name: "removeIf",
        params: &[BParam::Predicate],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/Object;)Z",
    },
    // `replaceAll(UnaryOperator)` applies the operator to each element in
    // place.
    BuiltinMethod {
        name: "replaceAll",
        params: &[BParam::UnaryOperator],
        ret: BRet::Void,
        descriptor: "(Ljava/lang/Object;)V",
    },
    // `sort(Comparator)` — a stable sort by the comparator.
    BuiltinMethod {
        name: "sort",
        params: &[BParam::Comparator],
        ret: BRet::Void,
        descriptor: "(Ljava/lang/Object;)V",
    },
    BuiltinMethod {
        name: "isEmpty",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "add",
        params: &[BParam::Elem],
        ret: BRet::Boolean,
        descriptor: "(Ljava/lang/Object;)Z",
    },
    BuiltinMethod {
        name: "add",
        params: &[BParam::Int, BParam::Elem],
        ret: BRet::Void,
        descriptor: "(ILjava/lang/Object;)V",
    },
    BuiltinMethod {
        name: "get",
        params: &[BParam::Int],
        ret: BRet::Elem,
        descriptor: "(I)Ljava/lang/Object;",
    },
    BuiltinMethod {
        name: "set",
        params: &[BParam::Int, BParam::Elem],
        ret: BRet::Elem,
        descriptor: "(ILjava/lang/Object;)Ljava/lang/Object;",
    },
    BuiltinMethod {
        name: "remove",
        params: &[BParam::Int],
        ret: BRet::Elem,
        descriptor: "(I)Ljava/lang/Object;",
    },
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "contains",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "indexOf",
        &[BParam::Elem],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    bm(
        "lastIndexOf",
        &[BParam::Elem],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    bm(
        "remove",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "containsAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "removeAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "retainAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "addAll",
        &[I, BParam::SelfList],
        BRet::Boolean,
        "(ILjava/util/Collection;)Z",
    ),
    bm(
        "equals",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    // Capacity hints: real methods, observable-free in this VM.
    bm("ensureCapacity", &[I], BRet::Void, "(I)V"),
    bm("trimToSize", &[], BRet::Void, "()V"),
];

/// `java.util.Stack<E>` — everything a `List` has (it extends `Vector`), plus
/// the five LIFO operations. `push`/`pop`/`peek` act on the top (the end);
/// `empty` mirrors `isEmpty`; `search` is a 1-based distance from the top.
const STACK_METHODS: &[BuiltinMethod] = &[
    bm(
        "push",
        &[BParam::Elem],
        BRet::Elem,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm("pop", &[], BRet::Elem, "()Ljava/lang/Object;"),
    bm("peek", &[], BRet::Elem, "()Ljava/lang/Object;"),
    bm("empty", &[], BRet::Boolean, "()Z"),
    bm(
        "search",
        &[BParam::Elem],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    // The `List`/`Vector` surface.
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "removeIf",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "replaceAll",
        &[BParam::UnaryOperator],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "sort",
        &[BParam::Comparator],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "add",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "add",
        &[BParam::Int, BParam::Elem],
        BRet::Void,
        "(ILjava/lang/Object;)V",
    ),
    bm("get", &[BParam::Int], BRet::Elem, "(I)Ljava/lang/Object;"),
    bm(
        "set",
        &[BParam::Int, BParam::Elem],
        BRet::Elem,
        "(ILjava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "remove",
        &[BParam::Int],
        BRet::Elem,
        "(I)Ljava/lang/Object;",
    ),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "contains",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "indexOf",
        &[BParam::Elem],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    bm(
        "lastIndexOf",
        &[BParam::Elem],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    bm(
        "remove",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "containsAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "removeAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "retainAll",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "equals",
        &[BParam::SelfList],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// The `java.util.Queue<E>` methods a `Queue` variable exposes. The nullable
/// `poll`/`peek` return the boxed element so their empty-collection `null` is
/// representable; `remove()`/`element()` throw on empty instead.
const QUEUE_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "add",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "offer",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("remove", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("poll", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("peek", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("element", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm(
        "contains",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "remove",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `java.util.Deque<E>` — everything a `Queue` has, plus the two-ended and
/// stack (`push`/`pop`) operations.
const DEQUE_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "add",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "offer",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("remove", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("poll", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("peek", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("element", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("push", &[BParam::Elem], BRet::Void, "(Ljava/lang/Object;)V"),
    bm("pop", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm(
        "removeIf",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addFirst",
        &[BParam::Elem],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "addLast",
        &[BParam::Elem],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "offerFirst",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "offerLast",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("removeFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("removeLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("pollFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("pollLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("getFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("getLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("peekFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("peekLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm(
        "contains",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "remove",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// The concrete `java.util.LinkedList<E>` — every `List` method (it is a
/// `List`), plus the `Deque`/`Queue` operations. `get`/`set`/`remove(int)` and
/// the index methods come from being a list; the rest are the deque face.
const LINKEDLIST_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "add",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "add",
        &[BParam::Int, BParam::Elem],
        BRet::Void,
        "(ILjava/lang/Object;)V",
    ),
    bm("get", &[BParam::Int], BRet::Elem, "(I)Ljava/lang/Object;"),
    bm(
        "set",
        &[BParam::Int, BParam::Elem],
        BRet::Elem,
        "(ILjava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "remove",
        &[BParam::Int],
        BRet::Elem,
        "(I)Ljava/lang/Object;",
    ),
    bm(
        "remove",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "contains",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "indexOf",
        &[BParam::Elem],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    bm(
        "lastIndexOf",
        &[BParam::Elem],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    bm(
        "addAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "containsAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "equals",
        &[BParam::Object],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    // The Queue/Deque face.
    bm(
        "offer",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("remove", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("poll", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("peek", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("element", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("push", &[BParam::Elem], BRet::Void, "(Ljava/lang/Object;)V"),
    bm("pop", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm(
        "removeIf",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addFirst",
        &[BParam::Elem],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "addLast",
        &[BParam::Elem],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "offerFirst",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "offerLast",
        &[BParam::Elem],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("removeFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("removeLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("pollFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("pollLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("getFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("getLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("peekFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("peekLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
];

/// `java.util.stream.Stream<E>`. Intermediate ops return a stream; `map` erases
/// the element to `Object` (its output type is not tracked); `collect` returns
/// a `null`-typed result that adopts the assignment context, like a diamond.
const STREAM_METHODS: &[BuiltinMethod] = &[
    bm(
        "filter",
        &[BParam::Predicate],
        BRet::Stream,
        "(Ljava/util/function/Predicate;)Ljava/util/stream/Stream;",
    ),
    bm(
        "map",
        &[BParam::UnaryOperator],
        BRet::StreamErased,
        "(Ljava/util/function/Function;)Ljava/util/stream/Stream;",
    ),
    bm(
        "mapToInt",
        &[BParam::UnaryOperator],
        BRet::IntStream,
        "(Ljava/util/function/ToIntFunction;)Ljava/util/stream/IntStream;",
    ),
    bm("sorted", &[], BRet::Stream, "()Ljava/util/stream/Stream;"),
    bm(
        "sorted",
        &[BParam::Comparator],
        BRet::Stream,
        "(Ljava/util/Comparator;)Ljava/util/stream/Stream;",
    ),
    bm("distinct", &[], BRet::Stream, "()Ljava/util/stream/Stream;"),
    bm(
        "limit",
        &[BParam::Long],
        BRet::Stream,
        "(J)Ljava/util/stream/Stream;",
    ),
    bm(
        "skip",
        &[BParam::Long],
        BRet::Stream,
        "(J)Ljava/util/stream/Stream;",
    ),
    bm(
        "peek",
        &[BParam::Consumer],
        BRet::Stream,
        "(Ljava/util/function/Consumer;)Ljava/util/stream/Stream;",
    ),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/util/function/Consumer;)V",
    ),
    bm(
        "forEachOrdered",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/util/function/Consumer;)V",
    ),
    bm("count", &[], BRet::Long, "()J"),
    bm(
        "anyMatch",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/util/function/Predicate;)Z",
    ),
    bm(
        "allMatch",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/util/function/Predicate;)Z",
    ),
    bm(
        "noneMatch",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/util/function/Predicate;)Z",
    ),
    bm(
        "collect",
        &[BParam::Collector],
        BRet::Nullish,
        "(Ljava/util/stream/Collector;)Ljava/lang/Object;",
    ),
    bm("findFirst", &[], BRet::Optional, "()Ljava/util/Optional;"),
    bm("findAny", &[], BRet::Optional, "()Ljava/util/Optional;"),
    bm(
        "max",
        &[BParam::Comparator],
        BRet::Optional,
        "(Ljava/util/Comparator;)Ljava/util/Optional;",
    ),
    bm(
        "min",
        &[BParam::Comparator],
        BRet::Optional,
        "(Ljava/util/Comparator;)Ljava/util/Optional;",
    ),
];

/// `java.util.stream.IntStream` — a primitive int stream. Its lambdas take a
/// single `int` (the VM stores the elements unboxed, so the same erased
/// interfaces serve). `sum`/`toArray` are the numeric terminals `Stream` lacks;
/// `average`/`min`/`max` return `Optional…`, which caturra does not model.
const INTSTREAM_METHODS: &[BuiltinMethod] = &[
    bm(
        "filter",
        &[BParam::Predicate],
        BRet::IntStream,
        "(Ljava/util/function/IntPredicate;)Ljava/util/stream/IntStream;",
    ),
    bm(
        "map",
        &[BParam::UnaryOperator],
        BRet::IntStream,
        "(Ljava/util/function/IntUnaryOperator;)Ljava/util/stream/IntStream;",
    ),
    bm(
        "mapToObj",
        &[BParam::UnaryOperator],
        BRet::StreamErased,
        "(Ljava/util/function/IntFunction;)Ljava/util/stream/Stream;",
    ),
    bm(
        "boxed",
        &[],
        BRet::StreamInteger,
        "()Ljava/util/stream/Stream;",
    ),
    bm(
        "sorted",
        &[],
        BRet::IntStream,
        "()Ljava/util/stream/IntStream;",
    ),
    bm(
        "distinct",
        &[],
        BRet::IntStream,
        "()Ljava/util/stream/IntStream;",
    ),
    bm(
        "limit",
        &[BParam::Long],
        BRet::IntStream,
        "(J)Ljava/util/stream/IntStream;",
    ),
    bm(
        "skip",
        &[BParam::Long],
        BRet::IntStream,
        "(J)Ljava/util/stream/IntStream;",
    ),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/util/function/IntConsumer;)V",
    ),
    bm("sum", &[], BRet::Int, "()I"),
    bm("count", &[], BRet::Long, "()J"),
    bm("toArray", &[], BRet::IntArray, "()[I"),
    bm("max", &[], BRet::OptionalInt, "()Ljava/util/OptionalInt;"),
    bm("min", &[], BRet::OptionalInt, "()Ljava/util/OptionalInt;"),
    bm(
        "average",
        &[],
        BRet::OptionalDouble,
        "()Ljava/util/OptionalDouble;",
    ),
    bm(
        "anyMatch",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/util/function/IntPredicate;)Z",
    ),
    bm(
        "allMatch",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/util/function/IntPredicate;)Z",
    ),
    bm(
        "noneMatch",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/util/function/IntPredicate;)Z",
    ),
];

/// `java.util.stream.IntStream` static factories.
const INTSTREAM_STATIC_METHODS: &[BuiltinMethod] = &[
    bm(
        "range",
        &[BParam::Int, BParam::Int],
        BRet::IntStream,
        "(II)Ljava/util/stream/IntStream;",
    ),
    bm(
        "rangeClosed",
        &[BParam::Int, BParam::Int],
        BRet::IntStream,
        "(II)Ljava/util/stream/IntStream;",
    ),
];

/// `java.util.Optional<E>` — `get`/`orElse`/`orElseThrow` yield the element.
const OPTIONAL_METHODS: &[BuiltinMethod] = &[
    bm("isPresent", &[], BRet::Boolean, "()Z"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("get", &[], BRet::Elem, "()Ljava/lang/Object;"),
    bm("orElseThrow", &[], BRet::Elem, "()Ljava/lang/Object;"),
    bm(
        "orElse",
        &[BParam::Key],
        BRet::Elem,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "ifPresent",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/util/function/Consumer;)V",
    ),
    // `filter(predicate)` keeps a present value only if it matches, so the
    // result is an `Optional` of the same element type.
    bm(
        "filter",
        &[BParam::Predicate],
        BRet::Optional,
        "(Ljava/util/function/Predicate;)Ljava/util/Optional;",
    ),
    // `map(function)` transforms a present value; its result element is erased
    // (like a stream's `map`), so the `Optional`'s element becomes `Object`.
    bm(
        "map",
        &[BParam::UnaryOperator],
        BRet::OptionalErased,
        "(Ljava/util/function/Function;)Ljava/util/Optional;",
    ),
    // `orElseGet(supplier)` yields the value or the supplier's result — both the
    // element type.
    bm(
        "orElseGet",
        &[BParam::Supplier],
        BRet::Elem,
        "(Ljava/util/function/Supplier;)Ljava/lang/Object;",
    ),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `java.util.OptionalInt` — `getAsInt`/`orElse` yield an `int`.
const OPTIONALINT_METHODS: &[BuiltinMethod] = &[
    bm("isPresent", &[], BRet::Boolean, "()Z"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("getAsInt", &[], BRet::Int, "()I"),
    bm("orElseThrow", &[], BRet::Int, "()I"),
    bm("orElse", &[BParam::Int], BRet::Int, "(I)I"),
    bm(
        "ifPresent",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/util/function/IntConsumer;)V",
    ),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `java.util.OptionalDouble` — `getAsDouble`/`orElse` yield a `double`.
const OPTIONALDOUBLE_METHODS: &[BuiltinMethod] = &[
    bm("isPresent", &[], BRet::Boolean, "()Z"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("getAsDouble", &[], BRet::Double, "()D"),
    bm("orElseThrow", &[], BRet::Double, "()D"),
    bm("orElse", &[BParam::Double], BRet::Double, "(D)D"),
    bm(
        "ifPresent",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/util/function/DoubleConsumer;)V",
    ),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

const FILE_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "exists",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "isFile",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "isDirectory",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "delete",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "mkdir",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    BuiltinMethod {
        name: "createNewFile",
        params: &[],
        ret: BRet::Boolean,
        descriptor: "()Z",
    },
    // Java returns long; caturra has no long surface, so int (documented
    // deviation — virtual files are small).
    BuiltinMethod {
        name: "length",
        params: &[],
        ret: BRet::Int,
        descriptor: "()J",
    },
    BuiltinMethod {
        name: "getName",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "getPath",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
];

const WRITER_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "println",
        params: &[],
        ret: BRet::Void,
        descriptor: "()V",
    },
    BuiltinMethod {
        name: "println",
        params: &[BParam::Str],
        ret: BRet::Void,
        descriptor: "(Ljava/lang/String;)V",
    },
    BuiltinMethod {
        name: "println",
        params: &[BParam::Int],
        ret: BRet::Void,
        descriptor: "(I)V",
    },
    BuiltinMethod {
        name: "println",
        params: &[BParam::Double],
        ret: BRet::Void,
        descriptor: "(D)V",
    },
    BuiltinMethod {
        name: "println",
        params: &[BParam::Boolean],
        ret: BRet::Void,
        descriptor: "(Z)V",
    },
    BuiltinMethod {
        name: "println",
        params: &[BParam::Char],
        ret: BRet::Void,
        descriptor: "(C)V",
    },
    BuiltinMethod {
        name: "print",
        params: &[BParam::Str],
        ret: BRet::Void,
        descriptor: "(Ljava/lang/String;)V",
    },
    BuiltinMethod {
        name: "print",
        params: &[BParam::Int],
        ret: BRet::Void,
        descriptor: "(I)V",
    },
    BuiltinMethod {
        name: "print",
        params: &[BParam::Double],
        ret: BRet::Void,
        descriptor: "(D)V",
    },
    BuiltinMethod {
        name: "print",
        params: &[BParam::Boolean],
        ret: BRet::Void,
        descriptor: "(Z)V",
    },
    BuiltinMethod {
        name: "print",
        params: &[BParam::Char],
        ret: BRet::Void,
        descriptor: "(C)V",
    },
    // `write(String)` writes the whole string; `write(int)` writes a single
    // character (its low 16 bits), NOT the decimal — the VM keys on the
    // descriptor.
    BuiltinMethod {
        name: "write",
        params: &[BParam::Str],
        ret: BRet::Void,
        descriptor: "(Ljava/lang/String;)V",
    },
    BuiltinMethod {
        name: "write",
        params: &[BParam::Int],
        ret: BRet::Void,
        descriptor: "(I)V",
    },
    // `append` returns the writer, for chaining. `append(char)` writes the
    // character; `append(CharSequence)` writes the text.
    BuiltinMethod {
        name: "append",
        params: &[BParam::Char],
        ret: BRet::Writer,
        descriptor: "(C)Ljava/io/PrintWriter;",
    },
    BuiltinMethod {
        name: "append",
        params: &[BParam::Str],
        ret: BRet::Writer,
        descriptor: "(Ljava/lang/String;)Ljava/io/PrintWriter;",
    },
    BuiltinMethod {
        name: "close",
        params: &[],
        ret: BRet::Void,
        descriptor: "()V",
    },
    BuiltinMethod {
        name: "flush",
        params: &[],
        ret: BRet::Void,
        descriptor: "()V",
    },
];

const EXCEPTION_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "getMessage",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "toString",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
    BuiltinMethod {
        name: "printStackTrace",
        params: &[],
        ret: BRet::Void,
        descriptor: "()V",
    },
    BuiltinMethod {
        name: "getLocalizedMessage",
        params: &[],
        ret: BRet::Str,
        descriptor: "()Ljava/lang/String;",
    },
];

const MATH_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "abs",
        params: &[BParam::Int],
        ret: BRet::Int,
        descriptor: "(I)I",
    },
    BuiltinMethod {
        name: "abs",
        params: &[BParam::Double],
        ret: BRet::Double,
        descriptor: "(D)D",
    },
    BuiltinMethod {
        name: "pow",
        params: &[BParam::Double, BParam::Double],
        ret: BRet::Double,
        descriptor: "(DD)D",
    },
    BuiltinMethod {
        name: "sqrt",
        params: &[BParam::Double],
        ret: BRet::Double,
        descriptor: "(D)D",
    },
    BuiltinMethod {
        name: "random",
        params: &[],
        ret: BRet::Double,
        descriptor: "()D",
    },
    BuiltinMethod {
        name: "floor",
        params: &[BParam::Double],
        ret: BRet::Double,
        descriptor: "(D)D",
    },
    BuiltinMethod {
        name: "ceil",
        params: &[BParam::Double],
        ret: BRet::Double,
        descriptor: "(D)D",
    },
    // Java returns long; caturra surfaces int (documented deviation —
    // the classroom idiom is `(int) Math.round(x)` anyway).
    BuiltinMethod {
        name: "round",
        params: &[BParam::Double],
        ret: BRet::Int,
        descriptor: "(D)I",
    },
    BuiltinMethod {
        name: "max",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Int,
        descriptor: "(II)I",
    },
    BuiltinMethod {
        name: "max",
        params: &[BParam::Double, BParam::Double],
        ret: BRet::Double,
        descriptor: "(DD)D",
    },
    BuiltinMethod {
        name: "min",
        params: &[BParam::Int, BParam::Int],
        ret: BRet::Int,
        descriptor: "(II)I",
    },
    BuiltinMethod {
        name: "min",
        params: &[BParam::Double, BParam::Double],
        ret: BRet::Double,
        descriptor: "(DD)D",
    },
    bm("sin", &[D], BRet::Double, "(D)D"),
    bm("cos", &[D], BRet::Double, "(D)D"),
    bm("tan", &[D], BRet::Double, "(D)D"),
    bm("asin", &[D], BRet::Double, "(D)D"),
    bm("acos", &[D], BRet::Double, "(D)D"),
    bm("atan", &[D], BRet::Double, "(D)D"),
    bm("atan2", &[D, D], BRet::Double, "(DD)D"),
    bm("sinh", &[D], BRet::Double, "(D)D"),
    bm("cosh", &[D], BRet::Double, "(D)D"),
    bm("tanh", &[D], BRet::Double, "(D)D"),
    bm("exp", &[D], BRet::Double, "(D)D"),
    bm("expm1", &[D], BRet::Double, "(D)D"),
    bm("log", &[D], BRet::Double, "(D)D"),
    bm("log10", &[D], BRet::Double, "(D)D"),
    bm("log1p", &[D], BRet::Double, "(D)D"),
    bm("cbrt", &[D], BRet::Double, "(D)D"),
    bm("hypot", &[D, D], BRet::Double, "(DD)D"),
    bm("rint", &[D], BRet::Double, "(D)D"),
    bm("signum", &[D], BRet::Double, "(D)D"),
    bm("toDegrees", &[D], BRet::Double, "(D)D"),
    bm("toRadians", &[D], BRet::Double, "(D)D"),
    bm("copySign", &[D, D], BRet::Double, "(DD)D"),
    bm("ulp", &[D], BRet::Double, "(D)D"),
    bm("nextUp", &[D], BRet::Double, "(D)D"),
    bm("nextDown", &[D], BRet::Double, "(D)D"),
    bm("nextAfter", &[D, D], BRet::Double, "(DD)D"),
    bm("fma", &[D, D, D], BRet::Double, "(DDD)D"),
    bm("IEEEremainder", &[D, D], BRet::Double, "(DD)D"),
    bm("getExponent", &[D], BRet::Int, "(D)I"),
    bm("floorDiv", &[I, I], BRet::Int, "(II)I"),
    bm("floorMod", &[I, I], BRet::Int, "(II)I"),
    bm("addExact", &[I, I], BRet::Int, "(II)I"),
    bm("subtractExact", &[I, I], BRet::Int, "(II)I"),
    bm("multiplyExact", &[I, I], BRet::Int, "(II)I"),
    bm("negateExact", &[I], BRet::Int, "(I)I"),
    bm("incrementExact", &[I], BRet::Int, "(I)I"),
    bm("decrementExact", &[I], BRet::Int, "(I)I"),
    bm("abs", &[L], BRet::Long, "(J)J"),
    bm("abs", &[F], BRet::Float, "(F)F"),
    bm("max", &[F, F], BRet::Float, "(FF)F"),
    bm("min", &[F, F], BRet::Float, "(FF)F"),
    bm("signum", &[F], BRet::Float, "(F)F"),
    bm("max", &[L, L], BRet::Long, "(JJ)J"),
    bm("min", &[L, L], BRet::Long, "(JJ)J"),
    bm("toIntExact", &[L], BRet::Int, "(J)I"),
    bm("multiplyHigh", &[L, L], BRet::Long, "(JJ)J"),
    bm("multiplyFull", &[I, I], BRet::Long, "(II)J"),
    // The float overload comes first: for an `int` argument both apply, and
    // javac picks the more specific `float`.
    bm("scalb", &[F, I], BRet::Float, "(FI)F"),
    bm("scalb", &[D, I], BRet::Double, "(DI)D"),
];

const INTEGER_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "parseInt",
        params: &[BParam::Str],
        ret: BRet::Int,
        descriptor: "(Ljava/lang/String;)I",
    },
    BuiltinMethod {
        name: "toString",
        params: &[BParam::Int],
        ret: BRet::Str,
        descriptor: "(I)Ljava/lang/String;",
    },
    bm("toString", &[I, I], BRet::Str, "(II)Ljava/lang/String;"),
    bm("parseInt", &[S, I], BRet::Int, "(Ljava/lang/String;I)I"),
    bm("toBinaryString", &[I], BRet::Str, "(I)Ljava/lang/String;"),
    bm("toOctalString", &[I], BRet::Str, "(I)Ljava/lang/String;"),
    bm("toHexString", &[I], BRet::Str, "(I)Ljava/lang/String;"),
    // valueOf returns the primitive value (no boxed identity/caching
    // semantics in this VM — a documented deviation).
    bm("valueOf", &[I], BRet::Int, "(I)I"),
    bm(
        "valueOf",
        &[S],
        BRet::Int,
        "(Ljava/lang/String;)Ljava/lang/Integer;",
    ),
    bm("compare", &[I, I], BRet::Int, "(II)I"),
    bm("compareUnsigned", &[I, I], BRet::Int, "(II)I"),
    bm("max", &[I, I], BRet::Int, "(II)I"),
    bm("min", &[I, I], BRet::Int, "(II)I"),
    bm("sum", &[I, I], BRet::Int, "(II)I"),
    bm("hashCode", &[I], BRet::Int, "(I)I"),
    bm("signum", &[I], BRet::Int, "(I)I"),
    bm("bitCount", &[I], BRet::Int, "(I)I"),
    bm("highestOneBit", &[I], BRet::Int, "(I)I"),
    bm("lowestOneBit", &[I], BRet::Int, "(I)I"),
    bm("numberOfLeadingZeros", &[I], BRet::Int, "(I)I"),
    bm("numberOfTrailingZeros", &[I], BRet::Int, "(I)I"),
    bm("reverse", &[I], BRet::Int, "(I)I"),
    bm("reverseBytes", &[I], BRet::Int, "(I)I"),
    bm("rotateLeft", &[I, I], BRet::Int, "(II)I"),
    bm("rotateRight", &[I, I], BRet::Int, "(II)I"),
    bm("parseUnsignedInt", &[S], BRet::Int, "(Ljava/lang/String;)I"),
    bm("toUnsignedString", &[I], BRet::Str, "(I)Ljava/lang/String;"),
    bm(
        "toUnsignedString",
        &[I, I],
        BRet::Str,
        "(II)Ljava/lang/String;",
    ),
    bm("divideUnsigned", &[I, I], BRet::Int, "(II)I"),
    bm("remainderUnsigned", &[I, I], BRet::Int, "(II)I"),
];

const DOUBLE_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "parseDouble",
        params: &[BParam::Str],
        ret: BRet::Double,
        descriptor: "(Ljava/lang/String;)D",
    },
    BuiltinMethod {
        name: "toString",
        params: &[BParam::Double],
        ret: BRet::Str,
        descriptor: "(D)Ljava/lang/String;",
    },
    bm("valueOf", &[D], BRet::Double, "(D)D"),
    bm(
        "valueOf",
        &[S],
        BRet::Double,
        "(Ljava/lang/String;)Ljava/lang/Double;",
    ),
    bm("isNaN", &[D], BRet::Boolean, "(D)Z"),
    bm("isInfinite", &[D], BRet::Boolean, "(D)Z"),
    bm("isFinite", &[D], BRet::Boolean, "(D)Z"),
    bm("compare", &[D, D], BRet::Int, "(DD)I"),
    bm("max", &[D, D], BRet::Double, "(DD)D"),
    bm("min", &[D, D], BRet::Double, "(DD)D"),
    bm("sum", &[D, D], BRet::Double, "(DD)D"),
    bm("hashCode", &[D], BRet::Int, "(D)I"),
    bm("toHexString", &[D], BRet::Str, "(D)Ljava/lang/String;"),
    bm("doubleToLongBits", &[D], BRet::Long, "(D)J"),
    bm("doubleToRawLongBits", &[D], BRet::Long, "(D)J"),
    bm("longBitsToDouble", &[L], BRet::Double, "(J)D"),
];

const CHARACTER_METHODS: &[BuiltinMethod] = &[
    BuiltinMethod {
        name: "isDigit",
        params: &[BParam::Char],
        ret: BRet::Boolean,
        descriptor: "(C)Z",
    },
    BuiltinMethod {
        name: "isLetter",
        params: &[BParam::Char],
        ret: BRet::Boolean,
        descriptor: "(C)Z",
    },
    BuiltinMethod {
        name: "isLetterOrDigit",
        params: &[BParam::Char],
        ret: BRet::Boolean,
        descriptor: "(C)Z",
    },
    BuiltinMethod {
        name: "isUpperCase",
        params: &[BParam::Char],
        ret: BRet::Boolean,
        descriptor: "(C)Z",
    },
    BuiltinMethod {
        name: "isLowerCase",
        params: &[BParam::Char],
        ret: BRet::Boolean,
        descriptor: "(C)Z",
    },
    BuiltinMethod {
        name: "toUpperCase",
        params: &[BParam::Char],
        ret: BRet::Char,
        descriptor: "(C)C",
    },
    BuiltinMethod {
        name: "toLowerCase",
        params: &[BParam::Char],
        ret: BRet::Char,
        descriptor: "(C)C",
    },
    bm("isAlphabetic", &[C], BRet::Boolean, "(I)Z"),
    bm("isWhitespace", &[C], BRet::Boolean, "(C)Z"),
    bm("isSpaceChar", &[C], BRet::Boolean, "(C)Z"),
    bm("isJavaIdentifierStart", &[C], BRet::Boolean, "(C)Z"),
    bm("isJavaIdentifierPart", &[C], BRet::Boolean, "(C)Z"),
    bm("isDefined", &[C], BRet::Boolean, "(C)Z"),
    bm("isISOControl", &[C], BRet::Boolean, "(C)Z"),
    bm("isTitleCase", &[C], BRet::Boolean, "(C)Z"),
    bm("toTitleCase", &[C], BRet::Char, "(C)C"),
    bm("getNumericValue", &[C], BRet::Int, "(C)I"),
    bm("digit", &[C, I], BRet::Int, "(CI)I"),
    bm("forDigit", &[I, I], BRet::Char, "(II)C"),
    bm("compare", &[C, C], BRet::Int, "(CC)I"),
    bm("hashCode", &[C], BRet::Int, "(C)I"),
    bm("toString", &[C], BRet::Str, "(C)Ljava/lang/String;"),
    bm("valueOf", &[C], BRet::Char, "(C)C"),
    bm("isHighSurrogate", &[C], BRet::Boolean, "(C)Z"),
    bm("isLowSurrogate", &[C], BRet::Boolean, "(C)Z"),
    bm("isSurrogate", &[C], BRet::Boolean, "(C)Z"),
    bm("charCount", &[I], BRet::Int, "(I)I"),
];

const SHORT_METHODS: &[BuiltinMethod] = &[
    bm("parseShort", &[S], BRet::Short, "(Ljava/lang/String;)S"),
    bm(
        "toString",
        &[BParam::Short],
        BRet::Str,
        "(S)Ljava/lang/String;",
    ),
    bm(
        "valueOf",
        &[BParam::Short],
        BRet::Short,
        "(S)Ljava/lang/Short;",
    ),
    bm(
        "valueOf",
        &[S],
        BRet::Short,
        "(Ljava/lang/String;)Ljava/lang/Short;",
    ),
    bm(
        "compare",
        &[BParam::Short, BParam::Short],
        BRet::Int,
        "(SS)I",
    ),
    bm("hashCode", &[BParam::Short], BRet::Int, "(S)I"),
    bm("reverseBytes", &[BParam::Short], BRet::Short, "(S)S"),
];

const BYTE_METHODS: &[BuiltinMethod] = &[
    bm("parseByte", &[S], BRet::Byte, "(Ljava/lang/String;)B"),
    bm(
        "toString",
        &[BParam::Byte],
        BRet::Str,
        "(B)Ljava/lang/String;",
    ),
    bm(
        "valueOf",
        &[BParam::Byte],
        BRet::Byte,
        "(B)Ljava/lang/Byte;",
    ),
    bm(
        "valueOf",
        &[S],
        BRet::Byte,
        "(Ljava/lang/String;)Ljava/lang/Byte;",
    ),
    bm("compare", &[BParam::Byte, BParam::Byte], BRet::Int, "(BB)I"),
    bm("hashCode", &[BParam::Byte], BRet::Int, "(B)I"),
];

const FLOAT_METHODS: &[BuiltinMethod] = &[
    bm("parseFloat", &[S], BRet::Float, "(Ljava/lang/String;)F"),
    bm("toString", &[F], BRet::Str, "(F)Ljava/lang/String;"),
    bm("valueOf", &[F], BRet::Float, "(F)F"),
    bm(
        "valueOf",
        &[S],
        BRet::Float,
        "(Ljava/lang/String;)Ljava/lang/Float;",
    ),
    bm("isNaN", &[F], BRet::Boolean, "(F)Z"),
    bm("isInfinite", &[F], BRet::Boolean, "(F)Z"),
    bm("isFinite", &[F], BRet::Boolean, "(F)Z"),
    bm("compare", &[F, F], BRet::Int, "(FF)I"),
    bm("max", &[F, F], BRet::Float, "(FF)F"),
    bm("min", &[F, F], BRet::Float, "(FF)F"),
    bm("sum", &[F, F], BRet::Float, "(FF)F"),
    bm("hashCode", &[F], BRet::Int, "(F)I"),
    bm("floatToIntBits", &[F], BRet::Int, "(F)I"),
    bm("floatToRawIntBits", &[F], BRet::Int, "(F)I"),
    bm("intBitsToFloat", &[I], BRet::Float, "(I)F"),
];

const LONG_METHODS: &[BuiltinMethod] = &[
    bm("parseLong", &[S], BRet::Long, "(Ljava/lang/String;)J"),
    bm("toString", &[L], BRet::Str, "(J)Ljava/lang/String;"),
    bm("toBinaryString", &[L], BRet::Str, "(J)Ljava/lang/String;"),
    bm("toOctalString", &[L], BRet::Str, "(J)Ljava/lang/String;"),
    bm("toHexString", &[L], BRet::Str, "(J)Ljava/lang/String;"),
    bm("valueOf", &[L], BRet::Long, "(J)J"),
    bm(
        "valueOf",
        &[S],
        BRet::Long,
        "(Ljava/lang/String;)Ljava/lang/Long;",
    ),
    bm("compare", &[L, L], BRet::Int, "(JJ)I"),
    bm("max", &[L, L], BRet::Long, "(JJ)J"),
    bm("min", &[L, L], BRet::Long, "(JJ)J"),
    bm("sum", &[L, L], BRet::Long, "(JJ)J"),
    bm("signum", &[L], BRet::Int, "(J)I"),
    bm("hashCode", &[L], BRet::Int, "(J)I"),
    bm("bitCount", &[L], BRet::Int, "(J)I"),
    bm("numberOfLeadingZeros", &[L], BRet::Int, "(J)I"),
    bm("numberOfTrailingZeros", &[L], BRet::Int, "(J)I"),
    bm("reverse", &[L], BRet::Long, "(J)J"),
    bm("reverseBytes", &[L], BRet::Long, "(J)J"),
];

const SYSTEM_METHODS: &[BuiltinMethod] = &[
    bm("currentTimeMillis", &[], BRet::Long, "()J"),
    bm("nanoTime", &[], BRet::Long, "()J"),
    // `arraycopy(Object src, int srcPos, Object dest, int destPos, int len)`.
    // Typed to arrays, where javac takes `Object` and throws
    // `ArrayStoreException` for anything else — stricter, the safe direction.
    bm(
        "arraycopy",
        &[BParam::RefArray, I, BParam::RefArray, I, I],
        BRet::Void,
        "(Ljava/lang/Object;ILjava/lang/Object;II)V",
    ),
    bm("lineSeparator", &[], BRet::Str, "()Ljava/lang/String;"),
    // Terminate the program with the given status code (Java's System.exit).
    bm("exit", &[I], BRet::Void, "(I)V"),
    // Internal standard-out capture, used only by the bundled
    // SystemOutTestRunner (org.code.validation): one String per print/println.
    bm("__captureStart", &[], BRet::Void, "()V"),
    bm("__captureEnd", &[], BRet::StrArray, "()[Ljava/lang/String;"),
    // Internal Swing event pump, used only by the bundled __SwingRuntime:
    // render the current component tree (JSON), block until the next UI
    // event, and return its payload (or null when the window is closed).
    bm(
        "__uiAwait",
        &[S],
        BRet::Str,
        "(Ljava/lang/String;)Ljava/lang/String;",
    ),
    // Internal blocking dialog (bundled JOptionPane): show a modal of the
    // given kind with the message, block until the user answers, and return
    // the response (a code / entered text, or null if dismissed).
    bm(
        "__uiDialog",
        &[S, S],
        BRet::Str,
        "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
    ),
    // Internal image-pixel bridge, used only by the bundled org.code.media.Image:
    // the host preloads a named asset's pixels into the VFS as bytes (u32 width,
    // u32 height, then RGB triples) and these hand the whole buffer across in one
    // native call — a 400x400 image is 160k pixels, far too many to ferry through
    // the interpreter a token at a time. `__writeImage` sends an edited image back
    // out for the host to draw.
    bm(
        "__imageDims",
        &[S],
        BRet::IntArray,
        "(Ljava/lang/String;)[I",
    ),
    bm(
        "__imagePixels",
        &[S],
        BRet::IntArray,
        "(Ljava/lang/String;)[I",
    ),
    bm(
        "__writeImage",
        &[S, I, I, BParam::RefArray],
        BRet::Void,
        "(Ljava/lang/String;II[I)V",
    ),
];

const BOOLEAN_METHODS: &[BuiltinMethod] = &[
    bm("parseBoolean", &[S], BRet::Boolean, "(Ljava/lang/String;)Z"),
    bm("toString", &[Z], BRet::Str, "(Z)Ljava/lang/String;"),
    bm("valueOf", &[Z], BRet::Boolean, "(Z)Z"),
    bm("compare", &[Z, Z], BRet::Int, "(ZZ)I"),
    bm("hashCode", &[Z], BRet::Int, "(Z)I"),
    bm("logicalAnd", &[Z, Z], BRet::Boolean, "(ZZ)Z"),
    bm("logicalOr", &[Z, Z], BRet::Boolean, "(ZZ)Z"),
    bm("logicalXor", &[Z, Z], BRet::Boolean, "(ZZ)Z"),
];

/// `java.lang.Class` reflection methods (structural, read-only).
const CLASS_METHODS: &[BuiltinMethod] = &[
    bm("getSimpleName", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("getName", &[], BRet::Str, "()Ljava/lang/String;"),
    bm(
        "isAssignableFrom",
        &[BParam::Class],
        BRet::Boolean,
        "(Ljava/lang/Class;)Z",
    ),
    bm("getSuperclass", &[], BRet::Class, "()Ljava/lang/Class;"),
    bm(
        "getDeclaredFields",
        &[],
        BRet::FieldArray,
        "()[Ljava/lang/reflect/Field;",
    ),
    bm(
        "getDeclaredMethods",
        &[],
        BRet::MethodArray,
        "()[Ljava/lang/reflect/Method;",
    ),
    bm(
        "getMethods",
        &[],
        BRet::MethodArray,
        "()[Ljava/lang/reflect/Method;",
    ),
    bm(
        "getDeclaredConstructors",
        &[],
        BRet::ConstructorArray,
        "()[Ljava/lang/reflect/Constructor;",
    ),
    bm(
        "getConstructors",
        &[],
        BRet::ConstructorArray,
        "()[Ljava/lang/reflect/Constructor;",
    ),
    bm(
        "getConstructor",
        &[BParam::RefArray],
        BRet::Constructor,
        "([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;",
    ),
    bm(
        "getDeclaredConstructor",
        &[BParam::RefArray],
        BRet::Constructor,
        "([Ljava/lang/Class;)Ljava/lang/reflect/Constructor;",
    ),
    bm(
        "getDeclaredField",
        &[BParam::Str],
        BRet::Field,
        "(Ljava/lang/String;)Ljava/lang/reflect/Field;",
    ),
    bm(
        "getField",
        &[BParam::Str],
        BRet::Field,
        "(Ljava/lang/String;)Ljava/lang/reflect/Field;",
    ),
    bm(
        "getMethod",
        &[BParam::Str, BParam::RefArray],
        BRet::Method,
        "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;",
    ),
    bm(
        "getMethod",
        &[BParam::Str],
        BRet::Method,
        "(Ljava/lang/String;)Ljava/lang/reflect/Method;",
    ),
    bm(
        "getDeclaredMethod",
        &[BParam::Str, BParam::RefArray],
        BRet::Method,
        "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;",
    ),
    bm(
        "getDeclaredMethod",
        &[BParam::Str],
        BRet::Method,
        "(Ljava/lang/String;)Ljava/lang/reflect/Method;",
    ),
];

/// `java.lang.reflect.Constructor` methods.
const CONSTRUCTOR_METHODS: &[BuiltinMethod] = &[
    bm("getName", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("getModifiers", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    bm(
        "newInstance",
        &[BParam::RefArray],
        BRet::Object,
        "([Ljava/lang/Object;)Ljava/lang/Object;",
    ),
];

/// `java.lang.reflect.Field` methods.
const FIELD_METHODS: &[BuiltinMethod] = &[
    bm("getName", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("getType", &[], BRet::Class, "()Ljava/lang/Class;"),
    bm("getModifiers", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("setAccessible", &[Z], BRet::Void, "(Z)V"),
    bm(
        "get",
        &[BParam::Object],
        BRet::Object,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "getInt",
        &[BParam::Object],
        BRet::Int,
        "(Ljava/lang/Object;)I",
    ),
    bm(
        "getLong",
        &[BParam::Object],
        BRet::Long,
        "(Ljava/lang/Object;)J",
    ),
    bm(
        "getDouble",
        &[BParam::Object],
        BRet::Double,
        "(Ljava/lang/Object;)D",
    ),
    bm(
        "getBoolean",
        &[BParam::Object],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "set",
        &[BParam::Object, BParam::Object],
        BRet::Void,
        "(Ljava/lang/Object;Ljava/lang/Object;)V",
    ),
    bm(
        "setInt",
        &[BParam::Object, I],
        BRet::Void,
        "(Ljava/lang/Object;I)V",
    ),
    bm(
        "setLong",
        &[BParam::Object, L],
        BRet::Void,
        "(Ljava/lang/Object;J)V",
    ),
    bm(
        "setDouble",
        &[BParam::Object, D],
        BRet::Void,
        "(Ljava/lang/Object;D)V",
    ),
    bm(
        "setBoolean",
        &[BParam::Object, Z],
        BRet::Void,
        "(Ljava/lang/Object;Z)V",
    ),
    bm(
        "getGenericType",
        &[],
        BRet::Type,
        "()Ljava/lang/reflect/Type;",
    ),
];

/// `java.lang.reflect.Type` / `ParameterizedType` methods.
const TYPE_METHODS: &[BuiltinMethod] = &[
    bm(
        "getActualTypeArguments",
        &[],
        BRet::ObjectArray,
        "()[Ljava/lang/reflect/Type;",
    ),
    bm("getRawType", &[], BRet::Type, "()Ljava/lang/reflect/Type;"),
    bm("getTypeName", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `java.lang.reflect.Method` methods.
const METHOD_METHODS: &[BuiltinMethod] = &[
    bm("getName", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("getModifiers", &[], BRet::Int, "()I"),
    bm("getReturnType", &[], BRet::Class, "()Ljava/lang/Class;"),
    bm(
        "getParameterTypes",
        &[],
        BRet::ClassArray,
        "()[Ljava/lang/Class;",
    ),
    bm("getParameterCount", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("setAccessible", &[Z], BRet::Void, "(Z)V"),
    // invoke(Object receiver, Object... args); the varargs are packed into an
    // Object[] at the call site.
    bm(
        "invoke",
        &[BParam::Object, BParam::RefArray],
        BRet::Object,
        "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
    ),
];

/// `java.lang.StringBuilder` methods (`append` returns the builder for
/// chaining; the VM stores UTF-16 units).
const STRINGBUILDER_METHODS: &[BuiltinMethod] = &[
    bm(
        "append",
        &[S],
        BRet::Builder,
        "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[I],
        BRet::Builder,
        "(I)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[L],
        BRet::Builder,
        "(J)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[D],
        BRet::Builder,
        "(D)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[F],
        BRet::Builder,
        "(F)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[Z],
        BRet::Builder,
        "(Z)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[C],
        BRet::Builder,
        "(C)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[BParam::CharArray],
        BRet::Builder,
        "([C)Ljava/lang/StringBuilder;",
    ),
    bm(
        "append",
        &[BParam::Object],
        BRet::Builder,
        "(Ljava/lang/Object;)Ljava/lang/StringBuilder;",
    ),
    bm(
        "appendCodePoint",
        &[I],
        BRet::Builder,
        "(I)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, S],
        BRet::Builder,
        "(ILjava/lang/String;)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, C],
        BRet::Builder,
        "(IC)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, Z],
        BRet::Builder,
        "(IZ)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, I],
        BRet::Builder,
        "(II)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, L],
        BRet::Builder,
        "(IJ)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, F],
        BRet::Builder,
        "(IF)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, D],
        BRet::Builder,
        "(ID)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, BParam::CharArray],
        BRet::Builder,
        "(I[C)Ljava/lang/StringBuilder;",
    ),
    bm(
        "insert",
        &[I, BParam::Object],
        BRet::Builder,
        "(ILjava/lang/Object;)Ljava/lang/StringBuilder;",
    ),
    bm(
        "delete",
        &[I, I],
        BRet::Builder,
        "(II)Ljava/lang/StringBuilder;",
    ),
    bm(
        "deleteCharAt",
        &[I],
        BRet::Builder,
        "(I)Ljava/lang/StringBuilder;",
    ),
    bm(
        "replace",
        &[I, I, S],
        BRet::Builder,
        "(IILjava/lang/String;)Ljava/lang/StringBuilder;",
    ),
    bm("reverse", &[], BRet::Builder, "()Ljava/lang/StringBuilder;"),
    bm("setCharAt", &[I, C], BRet::Void, "(IC)V"),
    bm("setLength", &[I], BRet::Void, "(I)V"),
    bm("ensureCapacity", &[I], BRet::Void, "(I)V"),
    bm("trimToSize", &[], BRet::Void, "()V"),
    bm("indexOf", &[S], BRet::Int, "(Ljava/lang/String;)I"),
    bm("indexOf", &[S, I], BRet::Int, "(Ljava/lang/String;I)I"),
    bm("lastIndexOf", &[S], BRet::Int, "(Ljava/lang/String;)I"),
    bm("lastIndexOf", &[S, I], BRet::Int, "(Ljava/lang/String;I)I"),
    bm("substring", &[I], BRet::Str, "(I)Ljava/lang/String;"),
    bm("substring", &[I, I], BRet::Str, "(II)Ljava/lang/String;"),
    bm(
        "subSequence",
        &[I, I],
        BRet::Str,
        "(II)Ljava/lang/CharSequence;",
    ),
    bm(
        "compareTo",
        &[BParam::Builder],
        BRet::Int,
        "(Ljava/lang/StringBuilder;)I",
    ),
    bm(
        "getChars",
        &[I, I, BParam::CharArray, I],
        BRet::Void,
        "(II[CI)V",
    ),
    bm("codePointAt", &[I], BRet::Int, "(I)I"),
    bm("codePointBefore", &[I], BRet::Int, "(I)I"),
    bm("codePointCount", &[I, I], BRet::Int, "(II)I"),
    bm("offsetByCodePoints", &[I, I], BRet::Int, "(II)I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("length", &[], BRet::Int, "()I"),
    bm("charAt", &[I], BRet::Char, "(I)C"),
];

/// `java.util.Arrays` static methods.
/// `java.util.HashMap<K, V>` methods. Keys and values are erased to `Object`
/// in the descriptors, as javac erases them; the compiler autoboxes at the
/// boundary so that a missing key can hand back a real `null`.
const MAP_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    // `forEach(BiConsumer)`: the VM walks the entries in iteration order and
    // calls the lambda class's `accept` on each.
    bm(
        "forEach",
        &[BParam::BiConsumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm(
        "containsKey",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "containsValue",
        &[BParam::Val],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "get",
        &[BParam::Key],
        BRet::Val,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "getOrDefault",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "put",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "putIfAbsent",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "remove",
        &[BParam::Key],
        BRet::Val,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "remove",
        &[BParam::Key, BParam::Val],
        BRet::Boolean,
        "(Ljava/lang/Object;Ljava/lang/Object;)Z",
    ),
    bm(
        "replace",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "replace",
        &[BParam::Key, BParam::Val, BParam::Val],
        BRet::Boolean,
        "(Ljava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;)Z",
    ),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "putAll",
        &[BParam::SelfMap],
        BRet::Void,
        "(Ljava/util/Map;)V",
    ),
    bm(
        "equals",
        &[BParam::SelfMap],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("keySet", &[], BRet::Keys, "()Ljava/util/Set;"),
    bm("values", &[], BRet::Values, "()Ljava/util/Collection;"),
    bm("entrySet", &[], BRet::Entries, "()Ljava/util/Set;"),
];

/// `java.util.TreeMap<K, V>` (also its `SortedMap`/`NavigableMap` faces): the
/// `Map` surface plus the sorted-key navigation. `firstKey`/`lastKey` throw on
/// an empty map; the `floorKey`/`ceilingKey`/`lowerKey`/`higherKey` return the
/// boxed key so an absent result is `null`. Keys iterate in sorted order.
const TREEMAP_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm(
        "forEach",
        &[BParam::BiConsumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm(
        "containsKey",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "containsValue",
        &[BParam::Val],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "get",
        &[BParam::Key],
        BRet::Val,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "getOrDefault",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "put",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "putIfAbsent",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "remove",
        &[BParam::Key],
        BRet::Val,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "remove",
        &[BParam::Key, BParam::Val],
        BRet::Boolean,
        "(Ljava/lang/Object;Ljava/lang/Object;)Z",
    ),
    bm(
        "replace",
        &[BParam::Key, BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "putAll",
        &[BParam::SelfMap],
        BRet::Void,
        "(Ljava/util/Map;)V",
    ),
    bm(
        "equals",
        &[BParam::SelfMap],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
    bm("keySet", &[], BRet::Keys, "()Ljava/util/Set;"),
    bm("values", &[], BRet::Values, "()Ljava/util/Collection;"),
    bm("entrySet", &[], BRet::Entries, "()Ljava/util/Set;"),
    // Sorted navigation. The keys are boxed so an absent result is `null`.
    bm("firstKey", &[], BRet::Key, "()Ljava/lang/Object;"),
    bm("lastKey", &[], BRet::Key, "()Ljava/lang/Object;"),
    bm(
        "floorKey",
        &[BParam::Key],
        BRet::Key,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "ceilingKey",
        &[BParam::Key],
        BRet::Key,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "lowerKey",
        &[BParam::Key],
        BRet::Key,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "higherKey",
        &[BParam::Key],
        BRet::Key,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
];

/// `java.util.Set<E>` / `java.util.Collection<E>` — a map's `keySet()` and
/// `values()` views. `__get` is caturra's own indexed accessor, standing in
/// for the iterator the enhanced-for loop would otherwise need.
const VIEW_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    // `forEach(Consumer)` over the view's elements (keys or values).
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "contains",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `java.util.Set<E>` — a standalone `HashSet`, or a map's `keySet()`. Both
/// present the mutable `Set` interface; at runtime a `keySet()` view throws
/// `UnsupportedOperationException` on `add` and writes through to its map on
/// `remove`/`clear`, exactly as Java's does. `__get` (the enhanced-for
/// accessor) is synthesized by `for_each`, not listed here.
const SET_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "add",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "remove",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "contains",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "removeAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "retainAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "containsAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm(
        "removeIf",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "equals",
        &[BParam::Object],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `java.util.TreeSet<E>` (also its `SortedSet`/`NavigableSet` faces): the
/// mutable `Set` surface plus sorted navigation. `first`/`last` throw on an
/// empty set; the `floor`/`ceiling`/`lower`/`higher` and `pollFirst`/`pollLast`
/// return the boxed element so an absent/empty result is `null`.
const TREESET_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("clear", &[], BRet::Void, "()V"),
    bm(
        "add",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "remove",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "contains",
        &[BParam::Key],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "addAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "containsAll",
        &[BParam::SelfCollection],
        BRet::Boolean,
        "(Ljava/util/Collection;)Z",
    ),
    bm(
        "forEach",
        &[BParam::Consumer],
        BRet::Void,
        "(Ljava/lang/Object;)V",
    ),
    bm("first", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("last", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm(
        "floor",
        &[BParam::Key],
        BRet::BoxedElem,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "ceiling",
        &[BParam::Key],
        BRet::BoxedElem,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "lower",
        &[BParam::Key],
        BRet::BoxedElem,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm(
        "higher",
        &[BParam::Key],
        BRet::BoxedElem,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm("pollFirst", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm("pollLast", &[], BRet::BoxedElem, "()Ljava/lang/Object;"),
    bm(
        "removeIf",
        &[BParam::Predicate],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm(
        "equals",
        &[BParam::Object],
        BRet::Boolean,
        "(Ljava/lang/Object;)Z",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `Set<Map.Entry<K, V>>` — a map's `entrySet()` view.
const ENTRY_SET_METHODS: &[BuiltinMethod] = &[
    bm("size", &[], BRet::Int, "()I"),
    bm("isEmpty", &[], BRet::Boolean, "()Z"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// `java.util.Map.Entry<K, V>` — a live view onto one mapping.
const ENTRY_METHODS: &[BuiltinMethod] = &[
    bm("getKey", &[], BRet::Key, "()Ljava/lang/Object;"),
    bm("getValue", &[], BRet::Val, "()Ljava/lang/Object;"),
    bm(
        "setValue",
        &[BParam::Val],
        BRet::Val,
        "(Ljava/lang/Object;)Ljava/lang/Object;",
    ),
    bm("hashCode", &[], BRet::Int, "()I"),
    bm("toString", &[], BRet::Str, "()Ljava/lang/String;"),
];

/// The intrinsic method table and JVM class for a receiver type.
fn builtin_instance_table(ty: JType) -> Option<(&'static str, &'static [BuiltinMethod])> {
    match ty {
        JType::Str => Some(("java/lang/String", STRING_METHODS)),
        JType::StringBuilder => Some(("java/lang/StringBuilder", STRINGBUILDER_METHODS)),
        JType::Class => Some(("java/lang/Class", CLASS_METHODS)),
        JType::Field => Some(("java/lang/reflect/Field", FIELD_METHODS)),
        JType::Method => Some(("java/lang/reflect/Method", METHOD_METHODS)),
        JType::Type => Some(("java/lang/reflect/Type", TYPE_METHODS)),
        JType::Constructor => Some(("java/lang/reflect/Constructor", CONSTRUCTOR_METHODS)),
        JType::Scanner => Some(("java/util/Scanner", SCANNER_METHODS)),
        JType::File => Some(("java/io/File", FILE_METHODS)),
        JType::Exception(id) => Some((exception_internal(id), EXCEPTION_METHODS)),
        JType::Writer => Some(("java/io/PrintWriter", WRITER_METHODS)),
        JType::List(_) => Some(("java/util/ArrayList", LIST_METHODS)),
        JType::Stack(_) => Some(("java/util/Stack", STACK_METHODS)),
        JType::Stream(_) => Some(("java/util/stream/Stream", STREAM_METHODS)),
        JType::IntStream => Some(("java/util/stream/IntStream", INTSTREAM_METHODS)),
        JType::Optional(_) => Some(("java/util/Optional", OPTIONAL_METHODS)),
        JType::OptionalInt => Some(("java/util/OptionalInt", OPTIONALINT_METHODS)),
        JType::OptionalDouble => Some(("java/util/OptionalDouble", OPTIONALDOUBLE_METHODS)),
        JType::LinkedList { role, .. } => Some(match role {
            SeqRole::Full => ("java/util/LinkedList", LINKEDLIST_METHODS),
            SeqRole::ArrayDeque => ("java/util/ArrayDeque", DEQUE_METHODS),
            SeqRole::Queue => ("java/util/Queue", QUEUE_METHODS),
            SeqRole::Deque => ("java/util/Deque", DEQUE_METHODS),
        }),
        JType::Map { .. } => Some(("java/util/HashMap", MAP_METHODS)),
        JType::TreeMap { .. } => Some(("java/util/TreeMap", TREEMAP_METHODS)),
        JType::Set(_) => Some(("java/util/Set", SET_METHODS)),
        JType::TreeSet(_) => Some(("java/util/TreeSet", TREESET_METHODS)),
        JType::Collection(_) => Some(("java/util/Collection", VIEW_METHODS)),
        JType::EntrySet { .. } => Some(("java/util/Set", ENTRY_SET_METHODS)),
        JType::MapEntry { .. } => Some(("java/util/Map$Entry", ENTRY_METHODS)),
        _ => None,
    }
}

/// The intrinsic static-method table for a class name.
const CLASS_STATIC_METHODS: &[BuiltinMethod] = &[bm(
    "forName",
    &[BParam::Str],
    BRet::Class,
    "(Ljava/lang/String;)Ljava/lang/Class;",
)];

/// `java.util.stream.Collectors` factories — each returns a `Collector` the VM
/// recognizes in `Stream.collect`.
const COLLECTORS_METHODS: &[BuiltinMethod] = &[
    bm(
        "toList",
        &[],
        BRet::Collector,
        "()Ljava/util/stream/Collector;",
    ),
    bm(
        "toUnmodifiableList",
        &[],
        BRet::Collector,
        "()Ljava/util/stream/Collector;",
    ),
    bm(
        "toSet",
        &[],
        BRet::Collector,
        "()Ljava/util/stream/Collector;",
    ),
    bm(
        "toUnmodifiableSet",
        &[],
        BRet::Collector,
        "()Ljava/util/stream/Collector;",
    ),
    bm(
        "joining",
        &[],
        BRet::Collector,
        "()Ljava/util/stream/Collector;",
    ),
    bm(
        "joining",
        &[BParam::Str],
        BRet::Collector,
        "(Ljava/lang/CharSequence;)Ljava/util/stream/Collector;",
    ),
    bm(
        "joining",
        &[BParam::Str, BParam::Str, BParam::Str],
        BRet::Collector,
        "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/util/stream/Collector;",
    ),
];

/// `java.util.Comparator` static factories. `comparing*` takes a key-extractor
/// `Function`; the four `comparing`/`comparingInt`/`comparingDouble`/
/// `comparingLong` compare the key naturally, so they share one implementation.
const COMPARATOR_STATIC_METHODS: &[BuiltinMethod] = &[
    bm(
        "naturalOrder",
        &[],
        BRet::Comparator,
        "()Ljava/util/Comparator;",
    ),
    bm(
        "reverseOrder",
        &[],
        BRet::Comparator,
        "()Ljava/util/Comparator;",
    ),
    bm(
        "comparing",
        &[BParam::UnaryOperator],
        BRet::Comparator,
        "(Ljava/util/function/Function;)Ljava/util/Comparator;",
    ),
    bm(
        "comparingInt",
        &[BParam::UnaryOperator],
        BRet::Comparator,
        "(Ljava/util/function/ToIntFunction;)Ljava/util/Comparator;",
    ),
    bm(
        "comparingDouble",
        &[BParam::UnaryOperator],
        BRet::Comparator,
        "(Ljava/util/function/ToDoubleFunction;)Ljava/util/Comparator;",
    ),
    bm(
        "comparingLong",
        &[BParam::UnaryOperator],
        BRet::Comparator,
        "(Ljava/util/function/ToLongFunction;)Ljava/util/Comparator;",
    ),
];

fn builtin_static_table(class: &str) -> Option<(&'static str, &'static [BuiltinMethod])> {
    match class {
        "Math" => Some(("java/lang/Math", MATH_METHODS)),
        "Collectors" => Some(("java/util/stream/Collectors", COLLECTORS_METHODS)),
        "Comparator" => Some(("java/util/Comparator", COMPARATOR_STATIC_METHODS)),
        "IntStream" => Some(("java/util/stream/IntStream", INTSTREAM_STATIC_METHODS)),
        "Class" => Some(("java/lang/Class", CLASS_STATIC_METHODS)),
        "Integer" => Some(("java/lang/Integer", INTEGER_METHODS)),
        "Double" => Some(("java/lang/Double", DOUBLE_METHODS)),
        "Character" => Some(("java/lang/Character", CHARACTER_METHODS)),
        "String" => Some(("java/lang/String", STRING_STATIC_METHODS)),
        "Boolean" => Some(("java/lang/Boolean", BOOLEAN_METHODS)),
        "Long" => Some(("java/lang/Long", LONG_METHODS)),
        "Float" => Some(("java/lang/Float", FLOAT_METHODS)),
        "Short" => Some(("java/lang/Short", SHORT_METHODS)),
        "Byte" => Some(("java/lang/Byte", BYTE_METHODS)),
        "System" => Some(("java/lang/System", SYSTEM_METHODS)),
        _ => None,
    }
}

/// A compile-time intrinsic constant's value.
#[derive(Debug, Clone, Copy)]
enum BuiltinConstant {
    Int(i32),
    Double(f64),
    Char(u16),
    Bool(bool),
    Long(i64),
    Float(f32),
}

/// Intrinsic static constants (`Integer.MAX_VALUE`, `Math.PI`, ...).
#[allow(clippy::match_same_arms)] // one arm per documented constant reads clearer
fn builtin_static_constant(class: &str, field: &str) -> Option<BuiltinConstant> {
    use BuiltinConstant::{Bool, Char, Double, Int};
    match (class, field) {
        ("Integer", "MAX_VALUE") => Some(Int(i32::MAX)),
        ("Integer", "MIN_VALUE") => Some(Int(i32::MIN)),
        ("Short", "MAX_VALUE") => Some(Int(i32::from(i16::MAX))),
        ("Short", "MIN_VALUE") => Some(Int(i32::from(i16::MIN))),
        ("Short", "SIZE") => Some(Int(16)),
        ("Short", "BYTES") => Some(Int(2)),
        ("Byte", "MAX_VALUE") => Some(Int(i32::from(i8::MAX))),
        ("Byte", "MIN_VALUE") => Some(Int(i32::from(i8::MIN))),
        ("Byte", "SIZE") => Some(Int(8)),
        ("Byte", "BYTES") => Some(Int(1)),
        ("Integer" | "Float", "SIZE") => Some(Int(32)),
        ("Integer" | "Float", "BYTES") => Some(Int(4)),
        ("Math", "PI") => Some(Double(std::f64::consts::PI)),
        ("Math", "E") => Some(Double(std::f64::consts::E)),
        ("Double", "MAX_VALUE") => Some(Double(f64::MAX)),
        ("Double", "MIN_VALUE") => Some(Double(f64::from_bits(1))),
        ("Double", "MIN_NORMAL") => Some(Double(f64::MIN_POSITIVE)),
        ("Double", "POSITIVE_INFINITY") => Some(Double(f64::INFINITY)),
        ("Double", "NEGATIVE_INFINITY") => Some(Double(f64::NEG_INFINITY)),
        ("Double", "NaN") => Some(Double(f64::NAN)),
        ("Double" | "Long", "SIZE") => Some(Int(64)),
        ("Double" | "Long", "BYTES") => Some(Int(8)),
        ("Character", "MIN_VALUE") => Some(Char(0)),
        ("Character", "MAX_VALUE") => Some(Char(u16::MAX)),
        ("Character", "MIN_RADIX") => Some(Int(2)),
        ("Character", "MAX_RADIX") => Some(Int(36)),
        ("Boolean", "TRUE") => Some(Bool(true)),
        ("Boolean", "FALSE") => Some(Bool(false)),
        ("Long", "MAX_VALUE") => Some(BuiltinConstant::Long(i64::MAX)),
        ("Long", "MIN_VALUE") => Some(BuiltinConstant::Long(i64::MIN)),
        ("Float", "MAX_VALUE") => Some(BuiltinConstant::Float(f32::MAX)),
        ("Float", "MIN_VALUE") => Some(BuiltinConstant::Float(f32::from_bits(1))),
        ("Float", "MIN_NORMAL") => Some(BuiltinConstant::Float(f32::MIN_POSITIVE)),
        ("Float", "POSITIVE_INFINITY") => Some(BuiltinConstant::Float(f32::INFINITY)),
        ("Float", "NEGATIVE_INFINITY") => Some(BuiltinConstant::Float(f32::NEG_INFINITY)),
        ("Float", "NaN") => Some(BuiltinConstant::Float(f32::NAN)),
        _ => None,
    }
}

/// The receiver's type arguments, for the builtin parameter and return
/// kinds that depend on them. `ArrayList<E>` supplies one; `Map<K,V>` two.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct TypeArgs {
    /// A list's element type, or a map's key type.
    first: Option<ElemType>,
    /// A map's value type.
    second: Option<ElemType>,
}

impl TypeArgs {
    /// The type arguments a receiver carries, if it is a generic intrinsic.
    fn of(receiver: JType) -> Self {
        match receiver {
            // A list's element, and a view's own element, are the first
            // type argument; a map's key and value are the two.
            JType::List(elem)
            | JType::Stack(elem)
            | JType::Set(elem)
            | JType::TreeSet(elem)
            | JType::Stream(elem)
            | JType::Optional(elem)
            | JType::Collection(elem)
            | JType::LinkedList { elem, .. } => Self {
                first: Some(elem),
                second: None,
            },
            // An IntStream's element is a primitive `int`.
            JType::IntStream => Self {
                first: Some(ElemType::Int),
                second: None,
            },
            JType::Map { key, value }
            | JType::TreeMap { key, value }
            | JType::EntrySet { key, value }
            | JType::MapEntry { key, value } => Self {
                first: Some(key),
                second: Some(value),
            },
            _ => Self::default(),
        }
    }
}

/// A type argument as it appears in a method signature: a primitive one is
/// really its wrapper, since `Map<String, Integer>.get` returns `Integer`
/// and can return `null`. This is what lets `map.put(k, v);` discard a
/// reference while `int old = map.put(k, v);` throws on a new key, exactly
/// as Java's unboxing does.
fn boxed_if_primitive(elem: Option<ElemType>) -> JType {
    match elem {
        Some(
            primitive @ (ElemType::Int
            | ElemType::Double
            | ElemType::Long
            | ElemType::Float
            | ElemType::Short
            | ElemType::Byte
            | ElemType::Boolean
            | ElemType::Char),
        ) => JType::Boxed(primitive),
        Some(reference) => reference.base_type(),
        None => JType::Error,
    }
}

fn bparam_type(param: BParam, args: TypeArgs) -> JType {
    match param {
        BParam::Int => JType::Int,
        BParam::Double => JType::Double,
        BParam::Long => JType::Long,
        BParam::Float => JType::Float,
        BParam::Short => JType::Short,
        BParam::Byte => JType::Byte,
        BParam::Boolean => JType::Boolean,
        BParam::Char => JType::Char,
        BParam::Str => JType::Str,
        BParam::CharArray => JType::Array {
            elem: ElemType::Char,
            dims: 1,
        },
        BParam::SelfList => args.first.map_or(JType::Error, JType::List),
        BParam::SelfCollection => args.first.map_or(JType::Error, JType::Collection),
        BParam::Elem => args.first.map_or(JType::Error, ElemType::base_type),
        BParam::Class => JType::Class,
        BParam::RefArray => JType::Error,
        // `BiConsumer` never reaches here: `bparam_matches` answers it
        // directly, because only the method table knows the target class.
        BParam::Object
        | BParam::BiConsumer
        | BParam::Consumer
        | BParam::Predicate
        | BParam::UnaryOperator
        | BParam::Supplier
        | BParam::Comparator => JType::Object(ClassId(0)),
        BParam::Builder => JType::StringBuilder,
        BParam::Key => boxed_if_primitive(args.first),
        BParam::Val => boxed_if_primitive(args.second),
        BParam::SelfMap => match (args.first, args.second) {
            (Some(key), Some(value)) => JType::Map { key, value },
            _ => JType::Error,
        },
        BParam::Collector => JType::Collector,
    }
}

/// Whether an argument type satisfies a builtin parameter (widening).
fn bparam_matches(param: BParam, arg: JType, args: TypeArgs, table: &MethodTable) -> bool {
    match param {
        BParam::RefArray => matches!(arg, JType::Array { .. }),
        // Any reference (or boxable value) satisfies an `Object` parameter.
        BParam::Object => widens(arg, JType::Object(table.object_id), table),
        BParam::BiConsumer => matches!(
            (arg, table.class_id("__BiConsumer")),
            (JType::Object(id), Some(target)) if table.is_subtype(id, target)
        ),
        BParam::Consumer => matches!(
            (arg, table.class_id("__Consumer")),
            (JType::Object(id), Some(target)) if table.is_subtype(id, target)
        ),
        BParam::Predicate => matches!(
            (arg, table.class_id("__Predicate")),
            (JType::Object(id), Some(target)) if table.is_subtype(id, target)
        ),
        BParam::UnaryOperator => matches!(
            (arg, table.class_id("__UnaryOperator")),
            (JType::Object(id), Some(target)) if table.is_subtype(id, target)
        ),
        BParam::Supplier => matches!(
            (arg, table.class_id("__Supplier")),
            (JType::Object(id), Some(target)) if table.is_subtype(id, target)
        ),
        BParam::Comparator => matches!(
            (arg, table.class_id("__Comparator")),
            (JType::Object(id), Some(target)) if table.is_subtype(id, target)
        ),
        BParam::Collector => matches!(arg, JType::Collector | JType::Null),
        // A collection whose elements are assignable to the receiver's — a
        // `List`, `Set` or `Collection` of a widening element type. `null`
        // is a Collection too.
        BParam::SelfCollection => match (arg, args.first) {
            (JType::Null, _) => true,
            (
                JType::List(elem)
                | JType::Set(elem)
                | JType::TreeSet(elem)
                | JType::Collection(elem)
                | JType::LinkedList { elem, .. },
                Some(want),
            ) => widens(elem.base_type(), want.base_type(), table),
            _ => false,
        },
        other => widens(arg, bparam_type(other, args), table),
    }
}

/// Overload selection for intrinsic methods: applicable-by-widening
/// with exact match preferred (mirrors user-method resolution).
fn pick_builtin<'m>(
    methods: &'m [BuiltinMethod],
    name: &str,
    args: &[JType],
    type_args: TypeArgs,
    table: &MethodTable,
) -> Option<&'m BuiltinMethod> {
    let applicable: Vec<&BuiltinMethod> = methods
        .iter()
        .filter(|m| {
            m.name == name
                && m.params.len() == args.len()
                && m.params
                    .iter()
                    .zip(args)
                    .all(|(p, a)| bparam_matches(*p, *a, type_args, table))
        })
        .collect();
    if let Some(exact) = applicable.iter().find(|m| {
        m.params
            .iter()
            .zip(args)
            .all(|(p, a)| bparam_type(*p, type_args) == *a)
    }) {
        return Some(exact);
    }
    applicable.first().copied()
}

fn bret_type(ret: BRet, args: TypeArgs, table: &MethodTable) -> Option<JType> {
    match ret {
        BRet::Void => None,
        BRet::Writer => Some(JType::Writer),
        BRet::Int => Some(JType::Int),
        BRet::Double => Some(JType::Double),
        BRet::Long => Some(JType::Long),
        BRet::Float => Some(JType::Float),
        BRet::Short => Some(JType::Short),
        BRet::Byte => Some(JType::Byte),
        BRet::Boolean => Some(JType::Boolean),
        BRet::Char => Some(JType::Char),
        BRet::Str => Some(JType::Str),
        BRet::StrArray => Some(JType::Array {
            elem: ElemType::Str,
            dims: 1,
        }),
        BRet::CharArray => Some(JType::Array {
            elem: ElemType::Char,
            dims: 1,
        }),
        BRet::Elem => Some(args.first.map_or(JType::Error, ElemType::base_type)),
        BRet::Stream => Some(args.first.map_or(JType::Error, JType::Stream)),
        BRet::StreamErased => Some(JType::Stream(ElemType::Object(table.object_id))),
        BRet::IntStream => Some(JType::IntStream),
        BRet::StreamInteger => Some(JType::Stream(ElemType::Int)),
        BRet::IntArray => Some(JType::Array {
            elem: ElemType::Int,
            dims: 1,
        }),
        BRet::Optional => Some(args.first.map_or(JType::Error, JType::Optional)),
        // `Optional.map` erases its result element to `Object`, like a stream's.
        BRet::OptionalErased => Some(JType::Optional(ElemType::Object(table.object_id))),
        BRet::OptionalInt => Some(JType::OptionalInt),
        BRet::OptionalDouble => Some(JType::OptionalDouble),
        BRet::Collector => Some(JType::Collector),
        BRet::Comparator => Some(
            table
                .class_id("__Comparator")
                .map_or(JType::Error, JType::Object),
        ),
        BRet::Nullish => Some(JType::Null),
        BRet::Val => Some(boxed_if_primitive(args.second)),
        // A map key and a queue/deque nullable element both box the first arg.
        BRet::Key | BRet::BoxedElem => Some(boxed_if_primitive(args.first)),
        BRet::Keys => Some(args.first.map_or(JType::Error, JType::Set)),
        BRet::Values => Some(args.second.map_or(JType::Error, JType::Collection)),
        BRet::Entries => Some(match (args.first, args.second) {
            (Some(key), Some(value)) => JType::EntrySet { key, value },
            _ => JType::Error,
        }),
        BRet::Class => Some(JType::Class),
        BRet::FieldArray => Some(JType::Array {
            elem: ElemType::Field,
            dims: 1,
        }),
        BRet::ClassArray => Some(JType::Array {
            elem: ElemType::Class,
            dims: 1,
        }),
        BRet::MethodArray => Some(JType::Array {
            elem: ElemType::Method,
            dims: 1,
        }),
        BRet::ConstructorArray => Some(JType::Array {
            elem: ElemType::Constructor,
            dims: 1,
        }),
        BRet::Constructor => Some(JType::Constructor),
        BRet::Field => Some(JType::Field),
        BRet::Method => Some(JType::Method),
        BRet::Type => Some(JType::Type),
        BRet::ObjectArray => Some(JType::Array {
            elem: ElemType::Object(table.object_id),
            dims: 1,
        }),
        BRet::Builder => Some(JType::StringBuilder),
        BRet::Object => Some(JType::Object(table.object_id)),
    }
}

/// How a field assignment reaches its object.
enum FieldReceiver<'e> {
    /// A static field — no receiver.
    Static,
    /// The implicit `this` (slot 0).
    This,
    /// An explicit receiver expression.
    Object(&'e Expr),
}

/// What a method call's receiver refers to.
enum CallTarget<'e> {
    /// `System.out` / `System.err`.
    Stream(&'static str),
    /// Static methods of the named class.
    Static(String),
    /// A bare call: a method of the current class (static or, via the
    /// implicit `this`, instance).
    Own,
    /// A call on a receiver expression (`obj.method(...)`).
    Instance(&'e Expr),
}

fn describe_types(types: &[JType], table: &MethodTable) -> String {
    let names: Vec<String> = types.iter().map(|t| t.describe(table)).collect();
    names.join(",")
}

/// Break/continue targets for the innermost enclosing loop.
#[derive(Clone)]
struct LoopLabels {
    break_label: Label,
    continue_label: Label,
    /// `false` for switch entries: `break` binds to them but
    /// `continue` skips past to the nearest real loop.
    is_loop: bool,
    /// The source label (`outer:`) attached to this loop/switch, if
    /// any — the target of `break outer;` / `continue outer;`.
    label: Option<String>,
}

/// Per-method-body emission state.
struct BodyGen<'a> {
    path: &'a str,
    diagnostics: &'a mut Vec<Diagnostic>,
    pool: &'a mut ConstantPool,
    table: &'a MethodTable,
    current_class: &'a str,
    current_class_id: ClassId,
    /// Whether the enclosing method is static (constructors are not).
    in_static: bool,
    in_constructor: bool,
    /// Declared return type; `None` is `void`.
    return_type: Option<JType>,
    code: CodeBuilder,
    /// Lexical scopes (innermost last); Java forbids shadowing locals,
    /// so declaration checks search all of them.
    scopes: Vec<Vec<(String, LocalVar)>>,
    next_slot: u16,
    /// Innermost-last enclosing loops, for `break`/`continue`.
    loop_stack: Vec<LoopLabels>,
    /// A source label seen just before a loop/switch, consumed by the
    /// next loop/switch entry so `break label;` can target it.
    pending_label: Option<String>,
    /// Enclosing `finally` blocks (innermost last): the statements to
    /// duplicate on abrupt exits, with the loop depth at try entry so
    /// `break`/`continue` only run guards inside the exited loop.
    finally_stack: Vec<(Vec<Stmt>, usize)>,
    /// `(name, descriptor, generic signature, slot, live-from offset)`
    /// per declared local, for the `LocalVariableTable` (and, when the
    /// signature is present, `LocalVariableTypeTable`) debug
    /// attributes. Slots are never reused, so live ranges safely
    /// extend to the end of the method.
    local_var_debug: Vec<(String, String, Option<String>, u16, u16)>,
}

impl BodyGen<'_> {
    fn error(&mut self, span: SourceSpan, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(self.path, message, span));
    }

    /// Record a declared local for the `LocalVariableTable` (and, for
    /// generic types, the `LocalVariableTypeTable` signature).
    fn record_local_debug(&mut self, name: &str, ty: JType, slot: u16) {
        let descriptor = ty.descriptor(self.table);
        let signature = match ty {
            JType::List(elem) => Some(format!(
                "Ljava/util/ArrayList<{}>;",
                ElemType::base_type(elem).descriptor(self.table)
            )),
            _ => None,
        };
        let start = self.code.offset();
        self.local_var_debug
            .push((name.to_owned(), descriptor, signature, slot, start));
    }

    /// `["java","lang","Math","abs"]` → `["Math","abs"]`: collapse a
    /// fully qualified library prefix in a dotted name, unless a local
    /// variable named `java` shadows the package (Java's obscuring
    /// rules: variables win).
    fn strip_package_prefix(&mut self, path: &[String]) -> Option<Vec<String>> {
        if path.len() < 3 || path[0] != "java" || self.lookup("java").is_some() {
            return None;
        }
        let dotted = format!("java.{}.{}", path[1], path[2]);
        let simple = crate::imports::canonical_library_class(&dotted)?;
        let mut short = Vec::with_capacity(path.len() - 2);
        short.push(simple.to_owned());
        short.extend_from_slice(&path[3..]);
        Some(short)
    }

    /// A STATIC field of the class that enclosed this synthesized anonymous
    /// or lambda class. Hoisting the body to the top level lost the name, but
    /// Java resolves it through the enclosing class, so we do too. Instance
    /// fields would need the enclosing `this` captured, which caturra does
    /// not do — they still report "cannot find variable".
    fn enclosing_static_field(&self, name: &str) -> Option<(ClassId, FieldSig)> {
        let enclosing = self
            .table
            .info_by_id(self.current_class_id)
            .and_then(|info| info.enclosing.clone())?;
        let (owner, field) = self.table.field(&enclosing, name)?;
        field.is_static.then(|| (owner, field.clone()))
    }

    /// The `__caturraOuter` field, if this is a lambda that captured its
    /// enclosing instance, with the enclosing class id.
    fn captured_outer(&self) -> Option<(FieldSig, ClassId)> {
        let cls = self.table.class_name(self.current_class_id).to_owned();
        let (_, outer) = self.table.field(&cls, crate::capture::OUTER_FIELD)?;
        if let JType::Object(enclosing) = outer.ty {
            Some((outer.clone(), enclosing))
        } else {
            None
        }
    }

    /// An INSTANCE field of the enclosing class, reached through the captured
    /// `__caturraOuter`. Reads/writes are live on the real enclosing object,
    /// which is what a lambda accessing `this.f` does.
    fn enclosing_instance_field(&self, name: &str) -> Option<(FieldSig, ClassId, FieldSig)> {
        let (outer, enclosing) = self.captured_outer()?;
        let enc_name = self.table.class_name(enclosing).to_owned();
        let (field_owner_id, field) = self.table.field(&enc_name, name)?;
        (!field.is_static).then(|| (outer, field_owner_id, field.clone()))
    }

    /// Push the captured enclosing instance (`this.__caturraOuter`) onto the
    /// stack. The caller then reads or writes a field of it.
    fn push_captured_outer(&mut self, outer: &FieldSig) {
        self.code.push_op(op::ALOAD_0, 1);
        self.emit_getfield(self.current_class_id, outer);
    }

    /// The class `super` denotes: this class's superclass, or the implicit
    /// `Object` when it declares none. A field then resolves from there
    /// upward, which is what `super.n` means — the field the superclass
    /// sees, not this class's field of the same name.
    fn superclass_id(&self) -> ClassId {
        self.table
            .info_by_id(self.current_class_id)
            .and_then(|info| info.superclass)
            .unwrap_or(self.table.object_id)
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

    #[allow(clippy::too_many_lines)] // one arm per statement kind
    fn statement(&mut self, stmt: &Stmt) {
        if let Some(span) = statement_span(stmt) {
            self.code.mark_line(span.start.line);
        }
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
                AssignTarget::Field { object, name } => {
                    self.assign_field_target(object, name, *op, value, *span);
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
            Stmt::Break { label, span } => {
                let target_index = match label {
                    Some(name) => self
                        .loop_stack
                        .iter()
                        .rposition(|entry| entry.label.as_deref() == Some(name.as_str())),
                    None => self.loop_stack.last().map(|_| self.loop_stack.len() - 1),
                };
                match target_index {
                    Some(index) => {
                        let target = self.loop_stack[index].break_label;
                        self.emit_pending_finallys(Some(index));
                        self.code.branch(op::GOTO, target, 0);
                    }
                    None => self.error(
                        *span,
                        match label {
                            Some(name) => format!("undefined label: {name}"),
                            None => {
                                String::from("'break' can only be used inside a loop or switch")
                            }
                        },
                    ),
                }
            }
            Stmt::Continue { label, span } => {
                let target_index = match label {
                    Some(name) => {
                        let found = self
                            .loop_stack
                            .iter()
                            .rposition(|entry| entry.label.as_deref() == Some(name.as_str()));
                        // A labeled continue must name a LOOP.
                        match found {
                            Some(index) if self.loop_stack[index].is_loop => Some(index),
                            Some(_) => {
                                self.error(*span, format!("not a loop label: {name}"));
                                None
                            }
                            None => {
                                self.error(*span, format!("undefined label: {name}"));
                                None
                            }
                        }
                    }
                    None => self.loop_stack.iter().rposition(|entry| entry.is_loop),
                };
                match (target_index, label) {
                    (Some(index), _) => {
                        let target = self.loop_stack[index].continue_label;
                        self.emit_pending_finallys(Some(index));
                        self.code.branch(op::GOTO, target, 0);
                    }
                    (None, None) => {
                        self.error(*span, "'continue' can only be used inside a loop");
                    }
                    (None, Some(_)) => {} // already reported above
                }
            }
            Stmt::Labeled { label, body, .. } => self.labeled_statement(label, body),
            Stmt::Return { value, span } => self.return_statement(value.as_ref(), *span),
            Stmt::SuperCall { span, .. } | Stmt::ThisCall { span, .. } => {
                self.error(
                    *span,
                    "call to super/this must be the first statement in a constructor",
                );
            }
            Stmt::Try {
                body,
                catches,
                finally_body,
                span,
            } => self.try_statement(body, catches, finally_body.as_deref(), *span),
            Stmt::Throw { value, span } => self.throw_statement(value, *span),
            Stmt::Switch {
                selector,
                arms,
                span,
            } => self.switch_statement(selector, arms, *span),
        }
    }

    /// `switch` lowered to an evaluate-once selector plus a
    /// compare-and-jump chain, with fall-through bodies and `break`
    /// binding to the switch end. (Real javac emits tableswitch /
    /// lookupswitch; the chain is semantically identical and the VM
    /// has no performance stake.)
    #[allow(clippy::too_many_lines)] // one lowering plan
    /// `label: statement`. A label on a loop/switch is consumed by that
    /// construct (so `break label`/`continue label` reach it); a label
    /// on any other statement gets a synthetic break target so
    /// `break label` can jump past it (`continue label` there is a
    /// compile error, reported at the continue site).
    fn labeled_statement(&mut self, label: &str, body: &Stmt) {
        let attaches_directly = matches!(
            body,
            Stmt::While { .. }
                | Stmt::DoWhile { .. }
                | Stmt::For { .. }
                | Stmt::ForEach { .. }
                | Stmt::Switch { .. }
        );
        if attaches_directly {
            // The loop/switch entry will take this label.
            self.pending_label = Some(label.to_owned());
            self.statement(body);
            // Defensive: if the body somehow didn't consume it, clear.
            self.pending_label = None;
            return;
        }
        let end = self.code.new_label();
        self.loop_stack.push(LoopLabels {
            break_label: end,
            continue_label: end, // unused: continue can't target a non-loop
            is_loop: false,
            label: Some(label.to_owned()),
        });
        self.statement(body);
        self.loop_stack.pop();
        self.code.bind(end);
    }

    #[allow(clippy::too_many_lines)] // one lowering plan
    fn switch_statement(&mut self, selector: &Expr, arms: &[SwitchArm], span: SourceSpan) {
        let selector_ty = self.expr(selector);
        let is_string = selector_ty == JType::Str;
        let is_int = matches!(
            selector_ty,
            JType::Int | JType::Char | JType::Short | JType::Byte
        );
        let enum_class = match selector_ty {
            JType::Object(id) if self.table.is_enum(id) => Some(id),
            _ => None,
        };
        if selector_ty != JType::Error && !is_string && !is_int && enum_class.is_none() {
            // javac's wording for a long selector; other types get the
            // general incompatibility.
            self.error(
                selector.span(),
                if selector_ty == JType::Long {
                    String::from("incompatible types: possible lossy conversion from long to int")
                } else {
                    format!(
                        "incompatible types: {} cannot be converted to int (or String) \
                         for switch",
                        selector_ty.describe(self.table)
                    )
                },
            );
        }
        let selector_slot = self.next_slot;
        self.next_slot += selector_ty.width().max(1);
        self.emit_store(selector_slot, selector_ty);

        let end = self.code.new_label();
        let arm_labels: Vec<Label> = arms.iter().map(|_| self.code.new_label()).collect();
        let mut default_arm: Option<usize> = None;
        let mut seen_ints: Vec<i64> = Vec::new();
        let mut seen_strings: Vec<String> = Vec::new();

        // The dispatch chain: one comparison per case label.
        for (index, arm) in arms.iter().enumerate() {
            for label in &arm.labels {
                let Some(value) = label else {
                    if default_arm.is_some() {
                        self.error(arm.span, "duplicate default label");
                    }
                    default_arm = Some(index);
                    continue;
                };
                if let Some(enum_id) = enum_class {
                    // Enum switch: the case label is an unqualified
                    // constant name, compared by reference identity
                    // (constants are singletons).
                    let Expr::Name { path, .. } = value else {
                        self.error(
                            value.span(),
                            "an enum switch case label must be the constant's simple name",
                        );
                        continue;
                    };
                    let const_name = path.join(".");
                    if seen_strings.iter().any(|s| s == &const_name) {
                        self.error(value.span(), "duplicate case label");
                    }
                    seen_strings.push(const_name.clone());
                    self.emit_load(selector_slot, selector_ty);
                    let enum_name = self.table.class_name(enum_id).to_owned();
                    let descriptor = format!("L{enum_name};");
                    let field_ref =
                        intern_field_ref(self.pool, &enum_name, &const_name, &descriptor);
                    self.code.push_op_u16(op::GETSTATIC, field_ref, 1);
                    self.code.branch(op::IF_ACMPEQ, arm_labels[index], 2);
                } else if is_string {
                    let Expr::Literal {
                        value: Literal::Str(text),
                        ..
                    } = value
                    else {
                        self.error(value.span(), "case labels must be constants");
                        continue;
                    };
                    if seen_strings.iter().any(|s| s == text) {
                        self.error(value.span(), "duplicate case label");
                    }
                    seen_strings.push(text.clone());
                    // selector.equals("text")
                    self.emit_load(selector_slot, selector_ty);
                    let utf8 = self.pool.intern_utf8(text);
                    let string_index = self.pool.intern(Constant::String { string_index: utf8 });
                    self.code.push_ldc(string_index);
                    let equals = intern_method_ref(
                        self.pool,
                        "java/lang/String",
                        "equals",
                        "(Ljava/lang/Object;)Z",
                    );
                    self.code.push_op_u16(op::INVOKEVIRTUAL, equals, 1);
                    self.code.drop_stack(2);
                    self.code.branch(op::IFNE, arm_labels[index], 1);
                } else {
                    let Some(constant) = constant_int_value(value) else {
                        self.error(value.span(), "case labels must be constants");
                        continue;
                    };
                    if seen_ints.contains(&constant) {
                        self.error(value.span(), "duplicate case label");
                    }
                    seen_ints.push(constant);
                    self.emit_load(selector_slot, selector_ty);
                    self.push_int(i32::try_from(constant).unwrap_or_default());
                    self.code.branch(op::IF_ICMPEQ, arm_labels[index], 2);
                }
            }
        }
        // No label matched: default arm or straight past the switch.
        match default_arm {
            Some(index) => self.code.branch(op::GOTO, arm_labels[index], 0),
            None => self.code.branch(op::GOTO, end, 0),
        }

        // Bodies in order; fall-through is the natural next arm.
        let before_flags = self.assigned_flags();
        self.loop_stack.push(LoopLabels {
            break_label: end,
            continue_label: end, // never targeted: continue skips switches
            is_loop: false,
            label: self.pending_label.take(),
        });
        for (index, arm) in arms.iter().enumerate() {
            self.code.bind(arm_labels[index]);
            self.code.mark_line(arm.span.start.line);
            self.scopes.push(Vec::new());
            for stmt in &arm.body {
                self.statement(stmt);
            }
            self.scopes.pop();
        }
        self.loop_stack.pop();
        self.code.bind(end);
        // Conservative definite assignment: paths through the switch
        // vary, so restore the pre-switch state.
        self.restore_assigned(&before_flags);
        let _ = span;
    }

    /// `try { ... } catch (...) { ... } finally { ... }` — a JVMS
    /// exception table entry per catch clause plus a catch-all for the
    /// finally, handlers after the protected range. The finally body is
    /// duplicated on every exit path (javac's strategy).
    #[allow(clippy::too_many_lines)] // one emission plan, clearest linear
    fn try_statement(
        &mut self,
        body: &[Stmt],
        catches: &[CatchClause],
        finally_body: Option<&[Stmt]>,
        span: SourceSpan,
    ) {
        // Resolve catch types first: exception classes only, and no
        // clause may be masked by an earlier broader one (javac:
        // "exception X has already been caught").
        let mut resolved: Vec<Option<CatchKind>> = Vec::with_capacity(catches.len());
        for (index, clause) in catches.iter().enumerate() {
            let kind = self.resolve_catch_type(clause);
            if let Some(kind) = kind {
                for earlier in resolved.iter().take(index).flatten() {
                    if kind.is_masked_by(*earlier, self.table) {
                        self.error(
                            clause.span,
                            format!(
                                "exception {} has already been caught",
                                kind.simple_name(self.table)
                            ),
                        );
                    }
                }
            }
            resolved.push(kind);
        }

        // Abrupt exits (return/break/continue) inside the protected
        // region must run the finally first; the guard stack tells
        // them to.
        if let Some(finally_stmts) = finally_body {
            self.finally_stack
                .push((finally_stmts.to_vec(), self.loop_stack.len()));
        }

        let before_flags = self.assigned_flags();
        let start = self.code.offset();
        self.scopes.push(Vec::new());
        for stmt in body {
            self.statement(stmt);
        }
        self.scopes.pop();
        let end = self.code.offset();
        if start == end {
            // An empty protected range is illegal in the table; with
            // nothing to throw, the catches are dead anyway. The
            // finally still runs.
            self.restore_assigned(&before_flags);
            if let Some(finally_stmts) = finally_body {
                self.finally_stack.pop();
                self.emit_block(finally_stmts);
            }
            let _ = span;
            return;
        }
        let after = self.code.new_label();
        // Normal completion path: the finally copy runs un-guarded.
        if let Some(finally_stmts) = finally_body {
            let guard = self.finally_stack.pop();
            self.emit_block(finally_stmts);
            if let Some(guard) = guard {
                self.finally_stack.push(guard);
            }
        }
        self.code.branch(op::GOTO, after, 0);
        let try_flags = self.assigned_flags();
        let mut branch_flags = vec![try_flags];
        // Catch-body ranges for the finally catch-all (an exception
        // thrown inside a catch must still run the finally; one thrown
        // inside a finally copy must not re-run it).
        let mut catch_ranges: Vec<(u16, u16)> = Vec::new();

        for (clause, id) in catches.iter().zip(&resolved) {
            self.restore_assigned(&before_flags);
            let handler = self.code.offset();
            self.code.mark_line(clause.span.start.line);
            // The VM pushes the thrown object before jumping here.
            self.code.assume_stack(1);

            self.scopes.push(Vec::new());
            let Some(kind) = id else {
                // Unresolvable type: an error was reported; skip body
                // emission to avoid cascades.
                self.scopes.pop();
                continue;
            };
            let ty = kind.jtype();
            let slot = self.next_slot;
            self.next_slot += 1;
            self.emit_store(slot, ty);
            self.record_local_debug(&clause.name, ty, slot);
            if self.lookup(&clause.name).is_some() {
                self.error(
                    clause.span,
                    format!(
                        "variable '{}' is already defined in this method",
                        clause.name
                    ),
                );
            }
            self.scopes.last_mut().expect("scope pushed").push((
                clause.name.clone(),
                LocalVar {
                    slot,
                    ty,
                    is_final: false,
                    assigned: true,
                },
            ));
            for stmt in &clause.body {
                self.statement(stmt);
            }
            self.scopes.pop();
            let catch_body_end = self.code.offset();
            catch_ranges.push((handler, catch_body_end));
            if let Some(finally_stmts) = finally_body {
                let guard = self.finally_stack.pop();
                self.emit_block(finally_stmts);
                if let Some(guard) = guard {
                    self.finally_stack.push(guard);
                }
            }
            self.code.branch(op::GOTO, after, 0);
            branch_flags.push(self.assigned_flags());

            let catch_class = intern_class(self.pool, &kind.table_name(self.table));
            self.code
                .add_exception_entry(start, end, handler, catch_class);
        }

        // The finally's catch-all: run the finally, then rethrow.
        if let Some(finally_stmts) = finally_body {
            self.finally_stack.pop();
            let handler = self.code.offset();
            self.code.assume_stack(1);
            let scratch = self.next_slot;
            self.next_slot += 1;
            self.emit_store(scratch, JType::Exception(0));
            self.emit_block(finally_stmts);
            self.emit_load(scratch, JType::Exception(0));
            self.code.push_op(op::ATHROW, 0);
            self.code.drop_stack(1);
            // catch_type 0 = any; covers the try body and each catch
            // body, but never the finally copies themselves.
            self.code.add_exception_entry(start, end, handler, 0);
            for (range_start, range_end) in catch_ranges {
                if range_start != range_end {
                    self.code
                        .add_exception_entry(range_start, range_end, handler, 0);
                }
            }
        }
        self.code.bind(after);

        // JLS definite assignment: assigned after try-catch iff
        // assigned after the try block AND after every catch block
        // (each path above already includes the finally's effects).
        if let Some(first) = branch_flags.first().cloned() {
            self.restore_assigned(&first);
            for flags in &branch_flags[1..] {
                self.intersect_assigned(flags);
            }
        }
    }

    /// Emit statements in a fresh scope (a finally copy).
    fn emit_block(&mut self, statements: &[Stmt]) {
        self.scopes.push(Vec::new());
        for stmt in statements {
            self.statement(stmt);
        }
        self.scopes.pop();
    }

    /// Duplicate the bodies of enclosing `finally` blocks before an
    /// abrupt exit, innermost first. `down_to_loop` limits the walk for
    /// `break`/`continue` (only guards entered inside the exited loop
    /// run); `None` (a `return`) runs them all. Guards are disabled
    /// while emitting so a finally body's own exits cannot recurse.
    fn emit_pending_finallys(&mut self, down_to_loop: Option<usize>) {
        if self.finally_stack.is_empty() {
            return;
        }
        let saved = std::mem::take(&mut self.finally_stack);
        for (statements, loop_len) in saved.iter().rev() {
            if down_to_loop.is_none_or(|target| *loop_len > target) {
                self.emit_block(statements);
            }
        }
        self.finally_stack = saved;
    }

    /// The resolved type of a catch clause, with javac-flavored errors
    /// for non-throwables.
    fn resolve_catch_type(&mut self, clause: &CatchClause) -> Option<CatchKind> {
        let TypeRef::Named(name) = &clause.ty else {
            self.error(clause.span, "expected an exception class in 'catch'");
            return None;
        };
        if name.contains('.') {
            let internal = name.replace('.', "/");
            if caturra_classfile::exceptions::is_exception_class(&internal) {
                return exception_id(&internal).map(CatchKind::Library);
            }
            self.error(
                clause.span,
                crate::imports::unknown_qualified_message(name.as_str()),
            );
            return None;
        }
        // User exception classes shadow library names, like elsewhere.
        if let Some(id) = self.table.class_id(name) {
            if self.table.is_throwable(id) {
                return Some(CatchKind::User(id));
            }
            self.error(
                clause.span,
                format!("incompatible types: {name} cannot be converted to Throwable"),
            );
            return None;
        }
        if let Some(internal) = caturra_classfile::exceptions::internal_name_of(name.as_str()) {
            return exception_id(internal).map(CatchKind::Library);
        }
        self.error(clause.span, format!("cannot find symbol: class {name}"));
        None
    }

    /// `throw expr;` — the expression must be a throwable.
    fn throw_statement(&mut self, value: &Expr, span: SourceSpan) {
        let ty = self.expr(value);
        match ty {
            JType::Exception(_) | JType::Null | JType::Error => {}
            JType::Object(id) if self.table.is_throwable(id) => {}
            other => {
                self.error(
                    span,
                    format!(
                        "incompatible types: {} cannot be converted to Throwable",
                        other.describe(self.table)
                    ),
                );
            }
        }
        self.code.push_op(op::ATHROW, 0);
        self.code.drop_stack(1);
    }

    fn return_statement(&mut self, value: Option<&Expr>, span: SourceSpan) {
        match (self.return_type, value) {
            (None, None) => {
                // Enclosing finally blocks run before the method exits.
                self.emit_pending_finallys(None);
                self.code.push_op(op::RETURN, 0);
            }
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
                self.convert_for_assignment_const(
                    actual,
                    expected,
                    value.span(),
                    constant_int_value(value),
                );
                // Enclosing finally blocks run with the return value
                // parked on the stack (they balance to empty).
                self.emit_pending_finallys(None);
                let opcode = match expected {
                    JType::Double => op::DRETURN,
                    JType::Long => op::LRETURN,
                    JType::Float => op::FRETURN,
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
                    ty.describe(self.table)
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
            is_loop: true,
            label: self.pending_label.take(),
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
            is_loop: true,
            label: self.pending_label.take(),
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
            is_loop: true,
            label: self.pending_label.take(),
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
        let Some(base_ty) = self.table.resolve_type(ty) else {
            self.error(span, unresolved_type_message(ty, self.table));
            return;
        };

        for declarator in declarators {
            // C-style `int a[], b;` gives the extra dimension to `a` alone, so
            // each declarator resolves its own type.
            let var_ty = if declarator.extra_dims == 0 {
                base_ty
            } else {
                let decl_ty = crate::ast::array_of(ty.clone(), declarator.extra_dims);
                let Some(resolved) = self.table.resolve_type(&decl_ty) else {
                    self.error(declarator.span, String::from("unsupported array type"));
                    continue;
                };
                resolved
            };
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
                            format!("illegal initializer for {}", var_ty.describe(self.table)),
                        );
                    }
                } else {
                    let init_ty = self.expr(init);
                    self.convert_for_assignment_const(
                        init_ty,
                        var_ty,
                        init.span(),
                        constant_int_value(init),
                    );
                }
                self.emit_store(slot, var_ty);
                true
            } else {
                false
            };
            // Recorded after the initializer's store (javac does the
            // same): a paused debugger doesn't show the variable until
            // it actually holds its value.
            self.record_local_debug(&declarator.name, var_ty, slot);
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

    #[allow(clippy::too_many_lines)] // one arm per target kind
    fn assign(&mut self, name: &str, op: Option<BinaryOp>, value: &Expr, span: SourceSpan) {
        if self.lookup(name).is_none() {
            // Implicit field of the current class.
            if let Some((owner, field)) = self.table.field(self.current_class, name) {
                let field = field.clone();
                if !field.is_static && self.in_static {
                    self.error(
                        span,
                        format!(
                            "non-static variable {name} cannot be referenced from a \
                             static context"
                        ),
                    );
                    return;
                }
                let receiver = if field.is_static {
                    FieldReceiver::Static
                } else {
                    FieldReceiver::This
                };
                self.assign_field(owner, &receiver, &field, op, value, span);
                return;
            }
            // A static field of the enclosing class of this lambda/anonymous
            // class: shared mutable state, so it is written through, not
            // captured by value.
            if let Some((owner, field)) = self.enclosing_static_field(name) {
                self.assign_field(owner, &FieldReceiver::Static, &field, op, value, span);
                return;
            }
            // An instance field of the enclosing class, written live through
            // the captured `__caturraOuter` (which `this` now denotes).
            if let Some((_, field_owner, field)) = self.enclosing_instance_field(name) {
                let outer = Expr::This { span };
                self.assign_field(
                    field_owner,
                    &FieldReceiver::Object(&outer),
                    &field,
                    op,
                    value,
                    span,
                );
                return;
            }
        }
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
                self.convert_for_assignment_const(
                    value_ty,
                    var_ty,
                    value.span(),
                    constant_int_value(value),
                );
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
                // A wrapper on either side unboxes first (JLS §15.26.2), so
                // `total += map.get(k)` and `count += 1` on an `Integer` both
                // work. The result boxes back when the target is a wrapper.
                let target = numeric_view(var_ty);
                let operand = numeric_view(value_ty);
                if target == JType::Boolean || !target.is_numeric() || !operand.is_numeric() {
                    if target == JType::Boolean {
                        self.error(span, "compound assignment cannot be applied to a boolean");
                    } else if value_ty != JType::Error {
                        self.error(
                            value.span(),
                            format!(
                                "operator '{}' cannot be applied to {} and {}",
                                compound_symbol(op),
                                var_ty.describe(self.table),
                                value_ty.describe(self.table)
                            ),
                        );
                    }
                    return;
                }

                // Compound assignment: promote, operate, then cast back
                // to the variable's type implicitly (JLS §15.26.2).
                let promoted = promote(target, operand);
                self.emit_load(slot, var_ty);
                self.numeric_conversion(var_ty, promoted);
                let actual = self.expr(value);
                self.numeric_conversion(actual, promoted);
                self.arithmetic_op(op, promoted);
                self.narrow_back(promoted, target);
                if let JType::Boxed(elem) = var_ty {
                    self.emit_box(elem);
                }
                self.emit_store(slot, var_ty);
            }
        }
    }

    // ----- field assignment -----

    /// `p.x = v`, `this.x = v`, `ClassName.staticField = v` (and the
    /// compound forms).
    fn assign_field_target(
        &mut self,
        object: &Expr,
        name: &str,
        op_kind: Option<BinaryOp>,
        value: &Expr,
        span: SourceSpan,
    ) {
        // `ClassName.field = v` — a static target.
        if let Expr::Name { path, .. } = object
            && path.len() == 1
            && self.lookup(&path[0]).is_none()
            && let Some(class_id) = self.table.class_id(&path[0])
        {
            let Some((_owner, field)) = self.resolve_field(class_id, name, span) else {
                self.expr(value);
                self.code.discard();
                return;
            };
            if !field.is_static {
                self.error(
                    span,
                    format!(
                        "non-static variable {name} cannot be referenced from a static context"
                    ),
                );
                return;
            }
            self.assign_field(
                class_id,
                &FieldReceiver::Static,
                &field,
                op_kind,
                value,
                span,
            );
            return;
        }

        let object_ty = self.type_of(object);
        let JType::Object(class_id) = object_ty else {
            if matches!(object_ty, JType::Array { .. }) && name == "length" {
                self.error(span, "cannot assign a value to final variable length");
            } else if object_ty != JType::Error {
                self.error(
                    span,
                    format!(
                        "cannot find symbol: field '{name}' on {}",
                        object_ty.describe(self.table)
                    ),
                );
            }
            return;
        };
        let Some((owner, field)) = self.resolve_field(class_id, name, span) else {
            return;
        };
        if field.is_static {
            // `instance.staticField = v`: evaluate the instance for
            // side effects, discard it, then assign the static field.
            self.expr(object);
            self.code.push_op(op::POP, 0);
            self.code.drop_stack(1);
            self.assign_field(owner, &FieldReceiver::Static, &field, op_kind, value, span);
            return;
        }
        self.assign_field(
            owner,
            &FieldReceiver::Object(object),
            &field,
            op_kind,
            value,
            span,
        );
    }

    /// Shared emission for plain/compound field assignment once the
    /// field and receiver kind are known.
    #[allow(clippy::too_many_lines)]
    fn assign_field(
        &mut self,
        class_id: ClassId,
        receiver: &FieldReceiver<'_>,
        field: &FieldSig,
        op_kind: Option<BinaryOp>,
        value: &Expr,
        span: SourceSpan,
    ) {
        if field.is_final && !(self.in_constructor && class_id == self.current_class_id) {
            self.error(
                span,
                format!("cannot assign a value to final variable {}", field.name),
            );
            return;
        }
        let class_name = self.table.class_name(class_id).to_owned();
        let field_ref = intern_field_ref(
            self.pool,
            &class_name,
            &field.name,
            &field.ty.descriptor(self.table),
        );
        let is_static = field.is_static;

        // Push the receiver (if any).
        match receiver {
            FieldReceiver::Static => {}
            FieldReceiver::This => self.code.push_op(op::ALOAD_0, 1),
            FieldReceiver::Object(object) => {
                let ty = self.expr(object);
                if ty == JType::Error {
                    return;
                }
            }
        }

        match op_kind {
            None => {
                let value_ty = self.expr(value);
                self.convert_for_assignment_const(
                    value_ty,
                    field.ty,
                    value.span(),
                    constant_int_value(value),
                );
                if is_static {
                    self.code.push_op_u16(op::PUTSTATIC, field_ref, 0);
                    self.code.drop_stack(field.ty.width());
                } else {
                    self.code.push_op_u16(op::PUTFIELD, field_ref, 0);
                    self.code.drop_stack(1 + field.ty.width());
                }
            }
            Some(op_kind) => {
                // String += concatenation, or numeric compound.
                if field.ty == JType::Str {
                    if op_kind != BinaryOp::Add {
                        self.error(span, "only '+=' can be applied to a String");
                        return;
                    }
                    if !is_static {
                        self.code.push_op(op::DUP, 1);
                    }
                    self.code.push_op_u16(
                        if is_static {
                            op::GETSTATIC
                        } else {
                            op::GETFIELD
                        },
                        field_ref,
                        1,
                    );
                    if !is_static {
                        self.code.drop_stack(1);
                    }
                    self.begin_concat_with_value_on_stack(JType::Str);
                    let part_ty = self.expr(value);
                    self.append_part(part_ty, value.span());
                    self.finish_concat();
                    if is_static {
                        self.code.push_op_u16(op::PUTSTATIC, field_ref, 0);
                        self.code.drop_stack(1);
                    } else {
                        self.code.push_op_u16(op::PUTFIELD, field_ref, 0);
                        self.code.drop_stack(2);
                    }
                    return;
                }
                let value_ty = self.type_of(value);
                // A wrapper field or value unboxes first, and boxes back.
                let target = numeric_view(field.ty);
                let operand = numeric_view(value_ty);
                if target == JType::Boolean || !target.is_numeric() || !operand.is_numeric() {
                    if value_ty != JType::Error {
                        self.error(
                            span,
                            format!(
                                "operator '{}' cannot be applied to {} and {}",
                                compound_symbol(op_kind),
                                field.ty.describe(self.table),
                                value_ty.describe(self.table)
                            ),
                        );
                    }
                    return;
                }
                let promoted = promote(target, operand);
                if !is_static {
                    self.code.push_op(op::DUP, 1);
                }
                self.code.push_op_u16(
                    if is_static {
                        op::GETSTATIC
                    } else {
                        op::GETFIELD
                    },
                    field_ref,
                    field.ty.width(),
                );
                if !is_static {
                    self.code.drop_stack(1);
                }
                self.numeric_conversion(field.ty, promoted);
                let actual = self.expr(value);
                self.numeric_conversion(actual, promoted);
                self.arithmetic_op(op_kind, promoted);
                self.narrow_back(promoted, target);
                if let JType::Boxed(elem) = field.ty {
                    self.emit_box(elem);
                }
                if is_static {
                    self.code.push_op_u16(op::PUTSTATIC, field_ref, 0);
                    self.code.drop_stack(field.ty.width());
                } else {
                    self.code.push_op_u16(op::PUTFIELD, field_ref, 0);
                    self.code.drop_stack(1 + field.ty.width());
                }
            }
        }
    }

    // ----- objects -----

    /// Emit `this.field = <init>` / `Class.field = <init>` for a field
    /// initializer (used by constructors and `<clinit>`).
    fn emit_field_initializer(&mut self, field: &FieldDecl, init: &Expr) {
        let ty = self
            .table
            .resolve_type(&field.ty)
            .unwrap_or(JType::Unsupported);
        if !field.is_static {
            self.code.push_op(op::ALOAD_0, 1);
        }
        if let Expr::ArrayLiteral { elements, span } = init {
            if matches!(ty, JType::Array { .. }) {
                self.emit_array_literal(elements, ty, *span);
            } else {
                self.error(
                    *span,
                    format!("illegal initializer for {}", ty.describe(self.table)),
                );
            }
        } else {
            let init_ty = self.expr(init);
            self.convert_for_assignment_const(init_ty, ty, init.span(), constant_int_value(init));
        }
        let field_ref = intern_field_ref(
            self.pool,
            self.current_class,
            &field.name,
            &ty.descriptor(self.table),
        );
        if field.is_static {
            self.code.push_op_u16(op::PUTSTATIC, field_ref, 0);
            self.code.drop_stack(ty.width());
        } else {
            self.code.push_op_u16(op::PUTFIELD, field_ref, 0);
            self.code.drop_stack(1 + ty.width());
        }
    }

    /// Emit `super(...)`/`this(...)`-style constructor invocation with
    /// `this` (slot 0) as the receiver.
    fn emit_constructor_call_on_this(&mut self, class_name: &str, args: &[Expr], span: SourceSpan) {
        self.code.push_op(op::ALOAD_0, 1);
        if class_name == "java/lang/Object" {
            let object_init = intern_method_ref(self.pool, "java/lang/Object", "<init>", "()V");
            self.code.push_op_u16(op::INVOKESPECIAL, object_init, 0);
            self.code.drop_stack(1);
            return;
        }
        // `super(...)` into a library throwable parent: the no-arg or
        // String-message constructor.
        if let Some(internal) =
            caturra_classfile::exceptions::internal_name_of(class_name).or_else(|| {
                caturra_classfile::exceptions::is_exception_class(class_name)
                    .then(|| {
                        caturra_classfile::exceptions::internal_name_of(
                            class_name.rsplit('/').next().unwrap_or(class_name),
                        )
                    })
                    .flatten()
            })
            && !self.table.has_class(class_name)
        {
            match args {
                [] => {
                    let init = intern_method_ref(self.pool, internal, "<init>", "()V");
                    self.code.push_op_u16(op::INVOKESPECIAL, init, 0);
                    self.code.drop_stack(1);
                }
                [message] => {
                    let message_ty = self.expr(message);
                    if message_ty != JType::Str && message_ty != JType::Error {
                        self.error(
                            message.span(),
                            format!(
                                "incompatible types: {} cannot be converted to String",
                                message_ty.describe(self.table)
                            ),
                        );
                    }
                    let init =
                        intern_method_ref(self.pool, internal, "<init>", "(Ljava/lang/String;)V");
                    self.code.push_op_u16(op::INVOKESPECIAL, init, 0);
                    self.code.drop_stack(2);
                }
                _ => {
                    self.error(
                        span,
                        "exception constructors take at most one String message",
                    );
                }
            }
            return;
        }
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            for arg in args {
                self.expr(arg);
            }
            return;
        }
        let table = self.table;
        let sig = if let Resolution::Found(sig) = table.resolve(class_name, "<init>", &arg_types) {
            sig.clone()
        } else {
            self.error(
                span,
                format!(
                    "constructor {class_name} in class {class_name} cannot be applied to \
                     given types ({})",
                    describe_types(&arg_types, self.table)
                ),
            );
            return;
        };
        let args_width = self.emit_call_args(args, &sig, span);
        let descriptor = sig.descriptor(self.table);
        let init_ref = intern_method_ref(self.pool, class_name, "<init>", &descriptor);
        self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
        self.code.drop_stack(1 + args_width);
    }

    /// The instance-field initializers, in declaration order.
    fn emit_instance_field_initializers(&mut self, class_decl: &ClassDecl) {
        self.emit_ordered_initializers(class_decl, false);
    }

    /// Emit the field initializers and initializer blocks of the given
    /// staticness, interleaved in source order (JLS §12.4.2 / §12.5).
    fn emit_ordered_initializers(&mut self, class_decl: &ClassDecl, is_static: bool) {
        enum Action<'a> {
            Field(&'a FieldDecl),
            Block(&'a InitBlock),
        }
        let mut actions: Vec<(usize, Action)> = Vec::new();
        for field in &class_decl.fields {
            if field.is_static == is_static && field.init.is_some() {
                actions.push((field.order, Action::Field(field)));
            }
        }
        for block in &class_decl.init_blocks {
            if block.is_static == is_static {
                actions.push((block.order, Action::Block(block)));
            }
        }
        actions.sort_by_key(|(order, _)| *order);
        for (_, action) in actions {
            match action {
                Action::Field(field) => {
                    if let Some(init) = &field.init {
                        self.emit_field_initializer(field, init);
                    }
                }
                Action::Block(block) => {
                    // Each block is its own local scope.
                    self.scopes.push(Vec::new());
                    for stmt in &block.body {
                        self.statement(stmt);
                    }
                    self.scopes.pop();
                }
            }
        }
    }

    /// Look up a field of `class_id`, with a javac-style private-access
    /// check. Returns a copied signature.
    fn resolve_field(
        &mut self,
        class_id: ClassId,
        name: &str,
        span: SourceSpan,
    ) -> Option<(ClassId, FieldSig)> {
        let class_name = self.table.class_name(class_id).to_owned();
        let Some((owner, field)) = self.table.field(&class_name, name) else {
            // `describe`, not the raw name: the top type is stored under its
            // internal `java/lang/Object`, which must not reach a diagnostic.
            let described = JType::Object(class_id).describe(self.table);
            self.error(
                span,
                format!("cannot find symbol: field '{name}' in class {described}"),
            );
            return None;
        };
        let field = field.clone();
        if field.is_private && owner != self.current_class_id && !self.table.relax_access {
            self.error(
                span,
                format!(
                    "{name} has private access in {}",
                    self.table.class_name(owner)
                ),
            );
            return None;
        }
        Some((owner, field))
    }

    /// Emit a read of a field of the current class through the implicit
    /// or explicit receiver already handled by the caller.
    fn emit_getfield(&mut self, class_id: ClassId, field: &FieldSig) -> JType {
        let class_name = self.table.class_name(class_id).to_owned();
        let field_ref = intern_field_ref(
            self.pool,
            &class_name,
            &field.name,
            &field.ty.descriptor(self.table),
        );
        if field.is_static {
            self.code
                .push_op_u16(op::GETSTATIC, field_ref, field.ty.width());
        } else {
            self.code
                .push_op_u16(op::GETFIELD, field_ref, field.ty.width());
            self.code.drop_stack(1);
        }
        field.ty
    }

    /// The static type of a `new` expression (pure).
    fn type_of_new_object(&mut self, class: &str, type_args: &[TypeRef]) -> JType {
        let simple = crate::imports::canonical_library_class(class).unwrap_or(class);
        let class = if class.contains('.') { simple } else { class };
        if let Some(id) = self.table.class_id(class) {
            return JType::Object(id);
        }
        if let Some(internal) = caturra_classfile::exceptions::internal_name_of(class)
            && let Some(id) = exception_id(internal)
        {
            return JType::Exception(id);
        }
        match class {
            "Object" => JType::Object(self.table.object_id),
            "StringBuilder" => JType::StringBuilder,
            "Scanner" => JType::Scanner,
            "File" => JType::File,
            "PrintWriter" => JType::Writer,
            "ArrayList" => match type_args {
                // A diamond `new ArrayList<>(...)` gets its element from the
                // context — `Null` (assignable to any List), matching what
                // `new_array_list` returns, so a nested `new C(new ArrayList<>(x))`
                // does not read as an `Error` argument.
                [arg] => elem_from_type_arg(arg, self.table).map_or(JType::Null, JType::List),
                _ => JType::Null,
            },
            "Stack" => match type_args {
                [arg] => elem_from_type_arg(arg, self.table).map_or(JType::Null, JType::Stack),
                _ => JType::Null,
            },
            "HashMap" | "Map" => match type_args {
                [key, value] => match (
                    elem_from_type_arg(key, self.table),
                    elem_from_type_arg(value, self.table),
                ) {
                    (Some(key), Some(value)) => JType::Map { key, value },
                    _ => JType::Null,
                },
                _ => JType::Null,
            },
            "HashSet" | "Set" => match type_args {
                // A diamond `new HashSet<>(...)` gets its element from context —
                // `Null` (assignable to any Set), matching `new_hash_set`.
                [arg] => elem_from_type_arg(arg, self.table).map_or(JType::Null, JType::Set),
                _ => JType::Null,
            },
            "LinkedList" => match type_args {
                [arg] => elem_from_type_arg(arg, self.table).map_or(JType::Null, |elem| {
                    JType::LinkedList {
                        elem,
                        role: SeqRole::Full,
                    }
                }),
                _ => JType::Null,
            },
            "PriorityQueue" => match type_args {
                [arg] => elem_from_type_arg(arg, self.table).map_or(JType::Null, |elem| {
                    JType::LinkedList {
                        elem,
                        role: SeqRole::Queue,
                    }
                }),
                _ => JType::Null,
            },
            "ArrayDeque" => match type_args {
                [arg] => elem_from_type_arg(arg, self.table).map_or(JType::Null, |elem| {
                    JType::LinkedList {
                        elem,
                        role: SeqRole::ArrayDeque,
                    }
                }),
                _ => JType::Null,
            },
            "TreeSet" => match type_args {
                [arg] => elem_from_type_arg(arg, self.table).map_or(JType::Null, JType::TreeSet),
                _ => JType::Null,
            },
            "TreeMap" => match type_args {
                [key, value] => match (
                    elem_from_type_arg(key, self.table),
                    elem_from_type_arg(value, self.table),
                ) {
                    (Some(key), Some(value)) => JType::TreeMap { key, value },
                    _ => JType::Null,
                },
                _ => JType::Null,
            },
            _ => JType::Error,
        }
    }

    /// A wrapper constructor (`new Integer(5)`, `new Double(3.5)`) —
    /// deprecated in Java but taught in some curricula. Boxes the
    /// argument (a primitive, or a String to parse).
    fn new_wrapper(&mut self, elem: ElemType, args: &[Expr], span: SourceSpan) -> JType {
        let [arg] = args else {
            self.error(span, "a wrapper constructor takes one argument");
            return JType::Error;
        };
        let prim = elem.base_type();
        let actual = self.expr(arg);
        if actual == JType::Str {
            // `new Integer("5")`: parse, then box.
            let internal = wrapper_internal(elem);
            let (parse, descriptor): (&str, String) = match elem {
                ElemType::Double => ("parseDouble", String::from("(Ljava/lang/String;)D")),
                ElemType::Long => ("parseLong", String::from("(Ljava/lang/String;)J")),
                ElemType::Float => ("parseFloat", String::from("(Ljava/lang/String;)F")),
                ElemType::Boolean => ("parseBoolean", String::from("(Ljava/lang/String;)Z")),
                _ => ("parseInt", String::from("(Ljava/lang/String;)I")),
            };
            let method_ref = intern_method_ref(self.pool, internal, parse, &descriptor);
            self.code
                .push_op_u16(op::INVOKESTATIC, method_ref, prim.width());
            self.code.drop_stack(1);
        } else {
            self.convert_for_assignment(actual, prim, arg.span());
        }
        self.emit_box(elem);
        JType::Boxed(elem)
    }

    /// `new ClassName(args)` (or an intrinsic: Scanner, `ArrayList`).
    #[allow(clippy::too_many_lines)] // qualified-name + intrinsic dispatch
    fn new_object(
        &mut self,
        class_name: &str,
        type_args: &[TypeRef],
        args: &[Expr],
        span: SourceSpan,
    ) -> JType {
        // `new java.util.Scanner(...)`: resolve the qualified name (and
        // reject unknown ones with javac's wording).
        let class_name = if class_name.contains('.') {
            if let Some(simple) = crate::imports::canonical_library_class(class_name) {
                simple
            } else if let Some(last) = class_name.rsplit('.').next()
                && self.table.class_id(last).is_some()
            {
                // `new Outer.Inner(...)`: a qualified reference to a
                // flattened nested user class.
                last
            } else {
                self.error(span, crate::imports::unknown_qualified_message(class_name));
                return JType::Error;
            }
        } else {
            class_name
        };
        if self.table.class_id(class_name).is_none() {
            match class_name {
                "Object" if args.is_empty() => return self.new_bare_object(),
                "StringBuilder" => return self.new_string_builder(args, span),
                "String" => return self.new_string(args, span),
                "Scanner" => return self.new_scanner(args, span),
                "ArrayList" => return self.new_array_list(type_args, args, span),
                "Stack" => return self.new_stack(type_args, args, span),
                "HashMap" => return self.new_hash_map(type_args, args, span),
                "HashSet" => return self.new_hash_set(type_args, args, span),
                "TreeSet" => return self.new_tree_set(type_args, args, span),
                "TreeMap" => return self.new_tree_map(type_args, args, span),
                "LinkedList" => return self.new_linked_list(type_args, args, span),
                "ArrayDeque" => return self.new_array_deque(type_args, args, span),
                "PriorityQueue" => return self.new_priority_queue(type_args, args, span),
                "File" => return self.new_file(args, span),
                "PrintWriter" => return self.new_writer(args, span),
                "Integer" | "Double" | "Long" | "Float" | "Short" | "Byte" | "Character"
                | "Boolean"
                    if wrapper_elem(class_name).is_some() =>
                {
                    return self.new_wrapper(
                        wrapper_elem(class_name).expect("checked"),
                        args,
                        span,
                    );
                }
                other => {
                    if let Some(internal) = caturra_classfile::exceptions::internal_name_of(other)
                        && let Some(id) = exception_id(internal)
                    {
                        return self.new_exception(id, args, span);
                    }
                }
            }
        }
        // A real Java class caturra does not model: say so, rather than
        // "is not a generic type" or "cannot find symbol". A user class of
        // the same name shadows it and has a `class_id`.
        if self.table.class_id(class_name).is_none()
            && let Some(reason) = crate::imports::unsupported_class_reason(class_name)
        {
            self.error(span, reason);
            return JType::Error;
        }
        if !type_args.is_empty() {
            self.error(span, format!("{class_name} is not a generic type"));
            return JType::Error;
        }
        let Some(class_id) = self.table.class_id(class_name) else {
            let classlib = ["String", "Object", "Integer", "Double", "StringBuilder"];
            if classlib.contains(&class_name) {
                self.error(
                    span,
                    format!(
                        "'new {class_name}(...)' is not yet supported by caturra — \
                         {class_name} arrives with the class library"
                    ),
                );
            } else {
                self.error(span, format!("cannot find symbol: class {class_name}"));
            }
            return JType::Error;
        };
        if self
            .table
            .info_by_id(class_id)
            .is_some_and(|i| i.is_abstract || i.is_interface)
        {
            self.error(
                span,
                format!("{class_name} is abstract; cannot be instantiated"),
            );
            return JType::Error;
        }
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            for arg in args {
                self.expr(arg);
            }
            return JType::Error;
        }

        let table = self.table;
        let sig = match table.resolve(class_name, "<init>", &arg_types) {
            Resolution::Found(sig) => sig.clone(),
            Resolution::UnknownName | Resolution::NoneApplicable => {
                self.error(
                    span,
                    format!(
                        "constructor {class_name} in class {class_name} cannot be applied to \
                         given types ({})",
                        describe_types(&arg_types, self.table)
                    ),
                );
                return JType::Error;
            }
            Resolution::Ambiguous(candidates) => {
                self.error(
                    span,
                    format!(
                        "reference to {class_name} is ambiguous: both {} match",
                        candidates.join(" and ")
                    ),
                );
                return JType::Error;
            }
        };
        if sig.is_private && class_id != self.current_class_id && !self.table.relax_access {
            self.error(
                span,
                format!("{class_name}() has private access in {class_name}"),
            );
            return JType::Error;
        }

        let class_index = intern_class(self.pool, class_name);
        self.code.push_op_u16(op::NEW, class_index, 1);
        self.code.push_op(op::DUP, 1);
        let args_width = self.emit_call_args(args, &sig, span);
        let descriptor = sig.descriptor(self.table);
        let init_ref = intern_method_ref(self.pool, class_name, "<init>", &descriptor);
        self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
        self.code.drop_stack(1 + args_width);
        JType::Object(class_id)
    }

    /// `new SomeException()` / `new SomeException("message")`.
    fn new_exception(&mut self, id: u8, args: &[Expr], span: SourceSpan) -> JType {
        let internal = exception_internal(id);
        let class_index = intern_class(self.pool, internal);
        self.code.push_op_u16(op::NEW, class_index, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref = intern_method_ref(self.pool, internal, "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [message] => {
                let message_ty = self.expr(message);
                if message_ty != JType::Str && message_ty != JType::Error {
                    self.error(
                        message.span(),
                        format!(
                            "incompatible types: {} cannot be converted to String",
                            message_ty.describe(self.table)
                        ),
                    );
                }
                let init_ref =
                    intern_method_ref(self.pool, internal, "<init>", "(Ljava/lang/String;)V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
            }
            _ => {
                self.error(
                    span,
                    "exception constructors take at most one String message",
                );
                return JType::Error;
            }
        }
        JType::Exception(id)
    }

    /// `new Scanner(System.in)` or `new Scanner(fileExpr)`.
    /// `new String()` / `new String(String)` / `new String(char[])` —
    /// creates a fresh (distinct-reference) `String`. The class library
    /// String is intrinsic, so this emits `new; dup; args; <init>`.
    fn new_string(&mut self, args: &[Expr], span: SourceSpan) -> JType {
        let string_class = intern_class(self.pool, "java/lang/String");
        self.code.push_op_u16(op::NEW, string_class, 1);
        self.code.push_op(op::DUP, 1);
        let descriptor = match args {
            [] => "()V",
            [arg] => {
                let ty = self.expr(arg);
                match ty {
                    JType::Str | JType::Null => "(Ljava/lang/String;)V",
                    JType::Array {
                        elem: ElemType::Char,
                        dims: 1,
                    } => "([C)V",
                    JType::Error => return JType::Error,
                    other => {
                        self.error(
                            span,
                            format!("no String constructor takes {}", other.describe(self.table)),
                        );
                        return JType::Error;
                    }
                }
            }
            _ => {
                self.error(
                    span,
                    "caturra supports new String(), new String(String), and new String(char[])",
                );
                return JType::Error;
            }
        };
        let init_ref = intern_method_ref(self.pool, "java/lang/String", "<init>", descriptor);
        self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
        // Pop the consumed args plus the duplicated receiver slot.
        let arg_width = u16::from(descriptor != "()V");
        self.code.drop_stack(1 + arg_width);
        JType::Str
    }

    /// `new StringBuilder()`, `new StringBuilder(String)` or
    /// `new StringBuilder(int)` (an initial-capacity hint caturra ignores —
    /// it does not model capacity, and nothing observes it).
    fn new_string_builder(&mut self, args: &[Expr], span: SourceSpan) -> JType {
        let class_index = intern_class(self.pool, "java/lang/StringBuilder");
        self.code.push_op_u16(op::NEW, class_index, 1);
        self.code.push_op(op::DUP, 1);
        let descriptor = match args {
            [] => "()V",
            [arg] => match self.expr(arg) {
                JType::Str | JType::Error => "(Ljava/lang/String;)V",
                ty if widens(ty, JType::Int, self.table) => "(I)V",
                _ => {
                    self.error(span, "new StringBuilder(...) takes a String or an int");
                    "(Ljava/lang/String;)V"
                }
            },
            _ => {
                self.error(span, "new StringBuilder(...) takes at most one argument");
                "()V"
            }
        };
        let init = intern_method_ref(self.pool, "java/lang/StringBuilder", "<init>", descriptor);
        self.code.push_op_u16(op::INVOKESPECIAL, init, 0);
        self.code.drop_stack(u16::from(!args.is_empty()));
        JType::StringBuilder
    }

    /// `new Object()` — an identity-only object (NEW + the no-op `<init>`).
    fn new_bare_object(&mut self) -> JType {
        let class_index = intern_class(self.pool, "java/lang/Object");
        self.code.push_op_u16(op::NEW, class_index, 1);
        self.code.push_op(op::DUP, 1);
        let init_ref = intern_method_ref(self.pool, "java/lang/Object", "<init>", "()V");
        self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
        self.code.drop_stack(1);
        JType::Object(self.table.object_id)
    }

    fn new_scanner(&mut self, args: &[Expr], span: SourceSpan) -> JType {
        let reads_stdin = matches!(
            args,
            [Expr::Name { path, .. }]
                if matches!(
                    path.iter().map(String::as_str).collect::<Vec<_>>()[..],
                    ["System", "in"] | ["java", "lang", "System", "in"]
                )
        );
        let scanner_class = intern_class(self.pool, "java/util/Scanner");
        self.code.push_op_u16(op::NEW, scanner_class, 1);
        self.code.push_op(op::DUP, 1);

        if reads_stdin {
            let stdin_field =
                intern_field_ref(self.pool, "java/lang/System", "in", "Ljava/io/InputStream;");
            self.code.push_op_u16(op::GETSTATIC, stdin_field, 1);
            let init_ref = intern_method_ref(
                self.pool,
                "java/util/Scanner",
                "<init>",
                "(Ljava/io/InputStream;)V",
            );
            self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
            self.code.drop_stack(2);
            return JType::Scanner;
        }

        // Scanner over a File.
        if let [file] = args {
            let file_ty = self.expr(file);
            if file_ty == JType::Error {
                return JType::Error;
            }
            if file_ty == JType::File {
                let init_ref = intern_method_ref(
                    self.pool,
                    "java/util/Scanner",
                    "<init>",
                    "(Ljava/io/File;)V",
                );
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
                return JType::Scanner;
            }
        }
        self.error(
            span,
            "Scanner reads System.in or a File: new Scanner(System.in) / \
             new Scanner(new File(\"data.txt\"))",
        );
        JType::Error
    }

    /// `new File(pathString)`.
    fn new_file(&mut self, args: &[Expr], span: SourceSpan) -> JType {
        let file_class = intern_class(self.pool, "java/io/File");
        self.code.push_op_u16(op::NEW, file_class, 1);
        self.code.push_op(op::DUP, 1);
        if let [path] = args {
            let path_ty = self.expr(path);
            if path_ty == JType::Error {
                return JType::Error;
            }
            if path_ty == JType::Str {
                let init_ref =
                    intern_method_ref(self.pool, "java/io/File", "<init>", "(Ljava/lang/String;)V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
                return JType::File;
            }
        }
        self.error(span, "File takes one String path: new File(\"data.txt\")");
        JType::Error
    }

    /// `new PrintWriter(pathString)` or `new PrintWriter(fileExpr)`.
    fn new_writer(&mut self, args: &[Expr], span: SourceSpan) -> JType {
        let writer_class = intern_class(self.pool, "java/io/PrintWriter");
        self.code.push_op_u16(op::NEW, writer_class, 1);
        self.code.push_op(op::DUP, 1);
        if let [target] = args {
            let target_ty = self.expr(target);
            if target_ty == JType::Error {
                return JType::Error;
            }
            let descriptor = match target_ty {
                JType::Str => Some("(Ljava/lang/String;)V"),
                JType::File => Some("(Ljava/io/File;)V"),
                _ => None,
            };
            if let Some(descriptor) = descriptor {
                let init_ref =
                    intern_method_ref(self.pool, "java/io/PrintWriter", "<init>", descriptor);
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
                return JType::Writer;
            }
        }
        self.error(
            span,
            "PrintWriter takes a path or a File: new PrintWriter(\"out.txt\")",
        );
        JType::Error
    }

    /// `new ArrayList<E>()` (diamond allowed when the declaration names
    /// the element type — students write both).
    fn new_array_list(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if args.len() > 1 {
            self.error(span, "ArrayList takes at most one constructor argument");
            return JType::Error;
        }
        let mut elem = match type_args {
            [] => {
                // Diamond `new ArrayList<>()`: the element type comes
                // from the assignment context (or the copy source below);
                // leave it generic here.
                None
            }
            [arg] => {
                let Some(elem) = elem_from_type_arg(arg, self.table) else {
                    self.error(
                        span,
                        "ArrayList element type must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some(elem)
            }
            _ => {
                self.error(span, "ArrayList takes one type argument");
                return JType::Error;
            }
        };
        let list_class = intern_class(self.pool, "java/util/ArrayList");
        self.code.push_op_u16(op::NEW, list_class, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref = intern_method_ref(self.pool, "java/util/ArrayList", "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [source] => {
                // Copy constructor `new ArrayList<>(collection)`: seed the
                // new list with the source collection's elements.
                let source_ty = self.expr(source);
                if elem.is_none()
                    && let JType::List(source_elem) = source_ty
                {
                    elem = Some(source_elem);
                }
                let init_ref = intern_method_ref(
                    self.pool,
                    "java/util/ArrayList",
                    "<init>",
                    "(Ljava/util/Collection;)V",
                );
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2); // the dup'd receiver + the source
            }
            _ => unreachable!("arg count checked above"),
        }
        match elem {
            Some(elem) => JType::List(elem),
            // Diamond: callers in declaration position convert with
            // the declared type; `Null` behaves as assignable-to-any
            // reference, which matches the diamond's intent.
            None => JType::Null,
        }
    }

    /// `new Stack<E>()` — the only constructor `java.util.Stack` declares. (It
    /// inherits `Vector`'s copy constructor but does not expose one of its own,
    /// so `new Stack<>(collection)` does not compile, matching javac.)
    fn new_stack(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if !args.is_empty() {
            self.error(span, "Stack takes no constructor arguments");
            return JType::Error;
        }
        let elem = match type_args {
            [] => None,
            [arg] => {
                let Some(elem) = elem_from_type_arg(arg, self.table) else {
                    self.error(
                        span,
                        "Stack element type must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some(elem)
            }
            _ => {
                self.error(span, "Stack takes one type argument");
                return JType::Error;
            }
        };
        let stack_class = intern_class(self.pool, "java/util/Stack");
        self.code.push_op_u16(op::NEW, stack_class, 1);
        self.code.push_op(op::DUP, 1);
        let init_ref = intern_method_ref(self.pool, "java/util/Stack", "<init>", "()V");
        self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
        self.code.drop_stack(1);
        match elem {
            Some(elem) => JType::Stack(elem),
            None => JType::Null,
        }
    }

    /// `new HashMap<K, V>()`, `new HashMap<>(initialCapacity)`, or the copy
    /// constructor `new HashMap<>(otherMap)`.
    fn new_hash_map(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if args.len() > 1 {
            self.error(span, "HashMap takes at most one constructor argument");
            return JType::Error;
        }
        let mut entry = match type_args {
            // Diamond `new HashMap<>()`: the key and value types come from
            // the assignment context (or the copy source below).
            [] => None,
            [key, value] => {
                let key = elem_from_type_arg(key, self.table);
                let value = elem_from_type_arg(value, self.table);
                let (Some(key), Some(value)) = (key, value) else {
                    self.error(
                        span,
                        "HashMap key and value types must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some((key, value))
            }
            _ => {
                self.error(span, "HashMap takes two type arguments");
                return JType::Error;
            }
        };
        let map_class = intern_class(self.pool, "java/util/HashMap");
        self.code.push_op_u16(op::NEW, map_class, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref = intern_method_ref(self.pool, "java/util/HashMap", "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [source] => {
                let source_ty = self.expr(source);
                // `new HashMap<>(map)` copies; `new HashMap<>(16)` sizes.
                let descriptor = if let JType::Map { key, value } = source_ty {
                    if entry.is_none() {
                        entry = Some((key, value));
                    }
                    "(Ljava/util/Map;)V"
                } else {
                    if !widens(source_ty, JType::Int, self.table) {
                        self.error(span, "new HashMap(...) takes a Map or an int capacity");
                    }
                    "(I)V"
                };
                let init_ref =
                    intern_method_ref(self.pool, "java/util/HashMap", "<init>", descriptor);
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2); // the dup'd receiver + the argument
            }
            _ => unreachable!("arg count checked above"),
        }
        match entry {
            Some((key, value)) => JType::Map { key, value },
            None => JType::Null,
        }
    }

    /// `new LinkedList<>()` / `new LinkedList<>(collection)`. Emits
    /// `new java.util.LinkedList`; the concrete type exposes the full
    /// `List`+`Deque` surface (`SeqRole::Full`).
    fn new_linked_list(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if args.len() > 1 {
            self.error(span, "LinkedList takes at most one constructor argument");
            return JType::Error;
        }
        let mut elem = match type_args {
            [] => None,
            [arg] => {
                let Some(elem) = elem_from_type_arg(arg, self.table) else {
                    self.error(
                        span,
                        "LinkedList element type must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some(elem)
            }
            _ => {
                self.error(span, "LinkedList takes one type argument");
                return JType::Error;
            }
        };
        let list_class = intern_class(self.pool, "java/util/LinkedList");
        self.code.push_op_u16(op::NEW, list_class, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref =
                    intern_method_ref(self.pool, "java/util/LinkedList", "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [source] => {
                let source_ty = self.expr(source);
                match source_ty {
                    JType::List(source_elem)
                    | JType::Set(source_elem)
                    | JType::Collection(source_elem)
                    | JType::LinkedList {
                        elem: source_elem, ..
                    } => {
                        if elem.is_none() {
                            elem = Some(source_elem);
                        }
                    }
                    JType::Null => {}
                    _ => self.error(span, "new LinkedList(...) takes a Collection"),
                }
                let init_ref = intern_method_ref(
                    self.pool,
                    "java/util/LinkedList",
                    "<init>",
                    "(Ljava/util/Collection;)V",
                );
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
            }
            _ => unreachable!("arg count checked above"),
        }
        match elem {
            Some(elem) => JType::LinkedList {
                elem,
                role: SeqRole::Full,
            },
            None => JType::Null,
        }
    }

    /// `new ArrayDeque<>()`, `new ArrayDeque<>(numElements)` (a capacity hint),
    /// or `new ArrayDeque<>(collection)`. Emits `new java.util.ArrayDeque`; the
    /// concrete type exposes the full `Deque` surface (`SeqRole::ArrayDeque`)
    /// but, unlike a `LinkedList`, is not a `List`.
    fn new_array_deque(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if args.len() > 1 {
            self.error(span, "ArrayDeque takes at most one constructor argument");
            return JType::Error;
        }
        let mut elem = match type_args {
            [] => None,
            [arg] => {
                let Some(elem) = elem_from_type_arg(arg, self.table) else {
                    self.error(
                        span,
                        "ArrayDeque element type must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some(elem)
            }
            _ => {
                self.error(span, "ArrayDeque takes one type argument");
                return JType::Error;
            }
        };
        let deque_class = intern_class(self.pool, "java/util/ArrayDeque");
        self.code.push_op_u16(op::NEW, deque_class, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref =
                    intern_method_ref(self.pool, "java/util/ArrayDeque", "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [source] => {
                let source_ty = self.expr(source);
                // `ArrayDeque(int numElements)` is a capacity hint (no elements);
                // any collection form is the copy constructor.
                let descriptor = if source_ty == JType::Int {
                    "(I)V"
                } else {
                    match source_ty {
                        JType::List(source_elem)
                        | JType::Set(source_elem)
                        | JType::Collection(source_elem)
                        | JType::LinkedList {
                            elem: source_elem, ..
                        } => {
                            if elem.is_none() {
                                elem = Some(source_elem);
                            }
                        }
                        JType::Null => {}
                        _ => self.error(
                            span,
                            "new ArrayDeque(...) takes an int capacity or a Collection",
                        ),
                    }
                    "(Ljava/util/Collection;)V"
                };
                let init_ref =
                    intern_method_ref(self.pool, "java/util/ArrayDeque", "<init>", descriptor);
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
            }
            _ => unreachable!("arg count checked above"),
        }
        match elem {
            Some(elem) => JType::LinkedList {
                elem,
                role: SeqRole::ArrayDeque,
            },
            None => JType::Null,
        }
    }

    /// The result type of `stream.collect(collector)` from the collector's
    /// syntactic form. `joining()` is a `String`; `toSet()`/`toList()` give a
    /// Set/List of the stream's element — but once `map` has erased the element
    /// to `Object`, a `null` that adopts the assignment context (like a diamond
    /// `new`), so `List<R> r = ....map(...).collect(toList())` still type-checks.
    fn collector_result_type(&mut self, collector: &Expr, stream_elem: ElemType) -> JType {
        let erased = matches!(stream_elem, ElemType::Object(id) if id == self.table.object_id);
        let factory = match collector {
            Expr::Call {
                receiver: Some(receiver),
                method,
                ..
            } if matches!(receiver.as_ref(), Expr::Name { path, .. } if path.len() == 1 && path[0] == "Collectors") => {
                method.as_str()
            }
            _ => return JType::Null,
        };
        match factory {
            "joining" => JType::Str,
            "toSet" | "toUnmodifiableSet" if !erased => JType::Set(stream_elem),
            "toList" | "toUnmodifiableList" if !erased => JType::List(stream_elem),
            _ => JType::Null,
        }
    }

    /// `comparator.reversed()` / `.thenComparing(...)` — build a derived
    /// comparator. The receiver comparator is already on the stack.
    #[allow(clippy::option_option)]
    fn emit_comparator_combinator(
        &mut self,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let comparator_ty = self
            .table
            .class_id("__Comparator")
            .map_or(JType::Error, JType::Object);
        if method == "reversed" {
            if !args.is_empty() {
                self.error(span, "reversed() takes no arguments");
            }
            let method_ref = intern_method_ref(
                self.pool,
                "java/util/Comparator",
                "reversed",
                "()Ljava/util/Comparator;",
            );
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            return Some(Some(comparator_ty));
        }
        // thenComparing: a key-extractor Function, or another Comparator.
        let [next] = args else {
            self.error(span, "thenComparing takes one argument");
            return None;
        };
        let next_ty = self.type_of(next);
        let descriptor = if self.is_comparator_type(next_ty) {
            "(Ljava/util/Comparator;)Ljava/util/Comparator;"
        } else {
            "(Ljava/util/function/Function;)Ljava/util/Comparator;"
        };
        self.expr(next);
        let method_ref = intern_method_ref(
            self.pool,
            "java/util/Comparator",
            "thenComparing",
            descriptor,
        );
        self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
        self.code.drop_stack(2);
        Some(Some(comparator_ty))
    }

    /// Whether a type is a `Comparator` — a `__Comparator` implementor (a user
    /// class, a desugared lambda, or a `Comparator`-typed value).
    fn is_comparator_type(&self, ty: JType) -> bool {
        matches!(
            (ty, self.table.class_id("__Comparator")),
            (JType::Object(id), Some(target)) if self.table.is_subtype(id, target)
        )
    }

    /// `new PriorityQueue<>()` / `(int)` / `(Comparator)` / `(int, Comparator)` /
    /// `(Collection)`. Emits `new java.util.PriorityQueue`; the VM keeps a real
    /// binary heap. Reuses the `Queue` type — a `PriorityQueue` *is* a `Queue`.
    fn new_priority_queue(
        &mut self,
        type_args: &[TypeRef],
        args: &[Expr],
        span: SourceSpan,
    ) -> JType {
        if args.len() > 2 {
            self.error(
                span,
                "PriorityQueue takes at most two constructor arguments",
            );
            return JType::Error;
        }
        let mut elem = match type_args {
            [] => None,
            [arg] => {
                let Some(elem) = elem_from_type_arg(arg, self.table) else {
                    self.error(
                        span,
                        "PriorityQueue element type must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some(elem)
            }
            _ => {
                self.error(span, "PriorityQueue takes one type argument");
                return JType::Error;
            }
        };
        let pq_class = intern_class(self.pool, "java/util/PriorityQueue");
        self.code.push_op_u16(op::NEW, pq_class, 1);
        self.code.push_op(op::DUP, 1);
        let descriptor = match args {
            [] => "()V",
            [only] => {
                let ty = self.expr(only);
                if self.is_comparator_type(ty) {
                    "(Ljava/util/Comparator;)V"
                } else if let Some(source_elem) = collection_element_type(ty) {
                    if elem.is_none() {
                        elem = Some(source_elem);
                    }
                    "(Ljava/util/Collection;)V"
                } else {
                    if !widens(ty, JType::Int, self.table) {
                        self.error(
                            span,
                            "new PriorityQueue(...) takes an int capacity, a Comparator, \
                             or a Collection",
                        );
                    }
                    "(I)V"
                }
            }
            [capacity, comparator] => {
                let capacity_ty = self.expr(capacity);
                if !widens(capacity_ty, JType::Int, self.table) {
                    self.error(
                        span,
                        "new PriorityQueue(int, Comparator): the first argument is the capacity",
                    );
                }
                let comparator_ty = self.expr(comparator);
                if !self.is_comparator_type(comparator_ty) {
                    self.error(
                        span,
                        "new PriorityQueue(int, Comparator): the second argument is a Comparator",
                    );
                }
                "(ILjava/util/Comparator;)V"
            }
            _ => unreachable!("arg count checked above"),
        };
        let init_ref =
            intern_method_ref(self.pool, "java/util/PriorityQueue", "<init>", descriptor);
        self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
        self.code
            .drop_stack(1 + u16::try_from(args.len()).unwrap_or(0));
        match elem {
            Some(elem) => JType::LinkedList {
                elem,
                role: SeqRole::Queue,
            },
            None => JType::Null,
        }
    }

    /// `new HashSet<>()` / `new HashSet<>(int)` / `new HashSet<>(collection)`.
    /// Emits `new java.util.HashSet` so the VM builds a set backed by a map of
    /// the elements; the copy constructor deduplicates at runtime.
    fn new_hash_set(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if args.len() > 1 {
            self.error(span, "HashSet takes at most one constructor argument");
            return JType::Error;
        }
        let mut elem = match type_args {
            // Diamond `new HashSet<>()`: the element type comes from the
            // assignment context (or the copy source below).
            [] => None,
            [arg] => {
                let Some(elem) = elem_from_type_arg(arg, self.table) else {
                    self.error(
                        span,
                        "HashSet element type must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some(elem)
            }
            _ => {
                self.error(span, "HashSet takes one type argument");
                return JType::Error;
            }
        };
        let set_class = intern_class(self.pool, "java/util/HashSet");
        self.code.push_op_u16(op::NEW, set_class, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref = intern_method_ref(self.pool, "java/util/HashSet", "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [source] => {
                let source_ty = self.expr(source);
                // `new HashSet<>(c)` copies a collection; `new HashSet<>(16)` sizes.
                let descriptor = match source_ty {
                    JType::List(source_elem)
                    | JType::Set(source_elem)
                    | JType::Collection(source_elem) => {
                        if elem.is_none() {
                            elem = Some(source_elem);
                        }
                        "(Ljava/util/Collection;)V"
                    }
                    _ => {
                        if !widens(source_ty, JType::Int, self.table) {
                            self.error(
                                span,
                                "new HashSet(...) takes a Collection or an int capacity",
                            );
                        }
                        "(I)V"
                    }
                };
                let init_ref =
                    intern_method_ref(self.pool, "java/util/HashSet", "<init>", descriptor);
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2); // the dup'd receiver + the argument
            }
            _ => unreachable!("arg count checked above"),
        }
        match elem {
            Some(elem) => JType::Set(elem),
            None => JType::Null,
        }
    }

    /// `new TreeSet<>()` (natural ordering), `new TreeSet<>(comparator)`, or
    /// `new TreeSet<>(collection)` (copy + sort). Emits `new java.util.TreeSet`.
    fn new_tree_set(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if args.len() > 1 {
            self.error(span, "TreeSet takes at most one constructor argument");
            return JType::Error;
        }
        let mut elem = match type_args {
            [] => None,
            [arg] => {
                let Some(elem) = elem_from_type_arg(arg, self.table) else {
                    self.error(
                        span,
                        "TreeSet element type must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some(elem)
            }
            _ => {
                self.error(span, "TreeSet takes one type argument");
                return JType::Error;
            }
        };
        let set_class = intern_class(self.pool, "java/util/TreeSet");
        self.code.push_op_u16(op::NEW, set_class, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref = intern_method_ref(self.pool, "java/util/TreeSet", "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [source] => {
                let source_ty = self.expr(source);
                let descriptor = match source_ty {
                    JType::List(source_elem)
                    | JType::Set(source_elem)
                    | JType::TreeSet(source_elem)
                    | JType::Collection(source_elem) => {
                        if elem.is_none() {
                            elem = Some(source_elem);
                        }
                        "(Ljava/util/Collection;)V"
                    }
                    // A comparator: a `__Comparator` instance (a desugared
                    // lambda, a method reference, or a Comparator variable).
                    JType::Object(id)
                        if self
                            .table
                            .class_id("__Comparator")
                            .is_some_and(|target| self.table.is_subtype(id, target)) =>
                    {
                        "(Ljava/util/Comparator;)V"
                    }
                    _ => {
                        self.error(span, "new TreeSet(...) takes a Collection or a Comparator");
                        "(Ljava/util/Collection;)V"
                    }
                };
                let init_ref =
                    intern_method_ref(self.pool, "java/util/TreeSet", "<init>", descriptor);
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
            }
            _ => unreachable!("arg count checked above"),
        }
        match elem {
            Some(elem) => JType::TreeSet(elem),
            None => JType::Null,
        }
    }

    /// `new TreeMap<>()` (natural ordering), `new TreeMap<>(comparator)`, or
    /// `new TreeMap<>(map)` (copy + sort by key). Emits `new java.util.TreeMap`.
    fn new_tree_map(&mut self, type_args: &[TypeRef], args: &[Expr], span: SourceSpan) -> JType {
        if args.len() > 1 {
            self.error(span, "TreeMap takes at most one constructor argument");
            return JType::Error;
        }
        let mut entry = match type_args {
            [] => None,
            [key, value] => {
                let key = elem_from_type_arg(key, self.table);
                let value = elem_from_type_arg(value, self.table);
                let (Some(key), Some(value)) = (key, value) else {
                    self.error(
                        span,
                        "TreeMap key and value types must be Integer, Double, Boolean, \
                         Character, String, or a class",
                    );
                    return JType::Error;
                };
                Some((key, value))
            }
            _ => {
                self.error(span, "TreeMap takes two type arguments");
                return JType::Error;
            }
        };
        let map_class = intern_class(self.pool, "java/util/TreeMap");
        self.code.push_op_u16(op::NEW, map_class, 1);
        self.code.push_op(op::DUP, 1);
        match args {
            [] => {
                let init_ref = intern_method_ref(self.pool, "java/util/TreeMap", "<init>", "()V");
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(1);
            }
            [source] => {
                let source_ty = self.expr(source);
                let descriptor = match source_ty {
                    JType::Map { key, value } | JType::TreeMap { key, value } => {
                        if entry.is_none() {
                            entry = Some((key, value));
                        }
                        "(Ljava/util/Map;)V"
                    }
                    JType::Object(id)
                        if self
                            .table
                            .class_id("__Comparator")
                            .is_some_and(|target| self.table.is_subtype(id, target)) =>
                    {
                        "(Ljava/util/Comparator;)V"
                    }
                    _ => {
                        self.error(span, "new TreeMap(...) takes a Map or a Comparator");
                        "(Ljava/util/Map;)V"
                    }
                };
                let init_ref =
                    intern_method_ref(self.pool, "java/util/TreeMap", "<init>", descriptor);
                self.code.push_op_u16(op::INVOKESPECIAL, init_ref, 0);
                self.code.drop_stack(2);
            }
            _ => unreachable!("arg count checked above"),
        }
        match entry {
            Some((key, value)) => JType::TreeMap { key, value },
            None => JType::Null,
        }
    }

    /// Substitute a tracked type argument for a type variable on a
    /// read: emits a `checkcast` to the argument's class and returns
    /// its concrete type. Non-type-variable types pass through.
    fn substitute_type_var(&mut self, ty: JType, arg: ElemType) -> JType {
        if ty != JType::TypeVar {
            return ty;
        }
        let concrete = arg.base_type();
        let internal = match concrete {
            JType::Str => Some(String::from("java/lang/String")),
            JType::Object(id) => Some(self.table.class_name(id).to_owned()),
            _ => None,
        };
        if let Some(internal) = internal {
            let class_index = intern_class(self.pool, &internal);
            self.code.push_op_u16(op::CHECKCAST, class_index, 0);
        }
        concrete
    }

    /// A method call on a boxed wrapper receiver (already on the
    /// stack): unboxing accessors and Object methods.
    #[allow(clippy::option_option)]
    fn boxed_instance_call(
        &mut self,
        elem: ElemType,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let internal = wrapper_internal(elem);
        // (return type, descriptor, argument coercion)
        let plan: Option<(JType, String)> = match method {
            "intValue" | "hashCode" => Some((JType::Int, String::from("()I"))),
            "shortValue" => Some((JType::Short, String::from("()S"))),
            "byteValue" => Some((JType::Byte, String::from("()B"))),
            "longValue" => Some((JType::Long, String::from("()J"))),
            "doubleValue" => Some((JType::Double, String::from("()D"))),
            "floatValue" => Some((JType::Float, String::from("()F"))),
            "charValue" => Some((JType::Char, String::from("()C"))),
            "booleanValue" => Some((JType::Boolean, String::from("()Z"))),
            "toString" => Some((JType::Str, String::from("()Ljava/lang/String;"))),
            _ => None,
        };
        if let Some((ret, descriptor)) = plan {
            let method_ref = intern_method_ref(self.pool, internal, method, &descriptor);
            let ret_width = ret.width();
            self.code
                .push_op_u16(op::INVOKEVIRTUAL, method_ref, ret_width);
            self.code.drop_stack(1);
            return Some(Some(ret));
        }
        // equals(Object) / compareTo(Wrapper): one argument, boxed.
        match method {
            "equals" | "compareTo" => {
                let [arg] = args else {
                    self.error(span, format!("{method} takes one argument"));
                    return None;
                };
                let arg_ty = self.expr(arg);
                let (target, descriptor) = if method == "equals" {
                    (
                        JType::Object(self.table.object_id),
                        String::from("(Ljava/lang/Object;)Z"),
                    )
                } else {
                    (JType::Boxed(elem), format!("(L{internal};)I"))
                };
                self.convert_for_assignment(arg_ty, target, arg.span());
                let method_ref = intern_method_ref(self.pool, internal, method, &descriptor);
                self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
                self.code.drop_stack(2);
                let ret = if method == "equals" {
                    JType::Boolean
                } else {
                    JType::Int
                };
                Some(Some(ret))
            }
            other => {
                self.error(
                    span,
                    format!("cannot find symbol: method {other} in class {internal}"),
                );
                None
            }
        }
    }

    /// Resolve and emit an instance method call with the receiver
    /// expression. `None` means a diagnostic was reported.
    #[allow(clippy::option_option)] // error / void / value are distinct outcomes
    fn instance_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let receiver_ty = self.expr(receiver);
        // `Comparator` combinators (`reversed`/`thenComparing`) on a comparator
        // value — the bundled `__Comparator` interface has no such methods, so
        // caturra builds a derived comparator itself.
        if matches!(method, "reversed" | "thenComparing") && self.is_comparator_type(receiver_ty) {
            return self.emit_comparator_combinator(method, args, span);
        }
        let class_id = match receiver_ty {
            JType::Object(id) => id,
            // A parameterized receiver: dispatch on the erased class,
            // then substitute the type variable in the return with the
            // tracked argument (inserting a checkcast on reads).
            JType::Generic { class, arg } => {
                let result =
                    self.emit_virtual_call_on_stacked_receiver(class, method, args, span)?;
                return Some(result.map(|ret| self.substitute_type_var(ret, arg)));
            }
            // Wrapper instance methods (intValue, compareTo, ...).
            JType::Boxed(elem) => {
                return self.boxed_instance_call(elem, method, args, span);
            }
            // A method on a type variable: only Object's methods.
            JType::TypeVar => self.table.object_id,
            JType::Error => return None,
            JType::Str
            | JType::StringBuilder
            | JType::Scanner
            | JType::File
            | JType::Writer
            | JType::List(_)
            | JType::Stack(_)
            | JType::LinkedList { .. }
            | JType::Map { .. }
            | JType::TreeMap { .. }
            | JType::Set(_)
            | JType::TreeSet(_)
            | JType::Stream(_)
            | JType::Collector
            | JType::IntStream
            | JType::Optional(_)
            | JType::OptionalInt
            | JType::OptionalDouble
            | JType::Collection(_)
            | JType::EntrySet { .. }
            | JType::MapEntry { .. }
            | JType::Class
            | JType::Field
            | JType::Method
            | JType::Type
            | JType::Constructor
            | JType::Exception(_) => {
                return self.builtin_instance_call(receiver_ty, method, args, span);
            }
            // Arrays are Objects: `equals`/`hashCode`/`toString`/`getClass`.
            JType::Array { .. } => {
                return self.array_object_call(method, args, span);
            }
            // A wrapper method on a primitive receiver (`someInt.intValue()`,
            // `x.compareTo(y)`) — autobox and dispatch on the wrapper.
            JType::Int
            | JType::Double
            | JType::Long
            | JType::Float
            | JType::Short
            | JType::Byte
            | JType::Char
            | JType::Boolean => {
                if let Some(elem) = elem_type_of(receiver_ty) {
                    self.emit_box(elem);
                    return self.boxed_instance_call(elem, method, args, span);
                }
                self.error(
                    span,
                    format!(
                        "cannot call methods on {}",
                        receiver_ty.describe(self.table)
                    ),
                );
                return None;
            }
            other => {
                self.error(
                    span,
                    format!("cannot call methods on {}", other.describe(self.table)),
                );
                return None;
            }
        };
        self.emit_virtual_call_on_stacked_receiver(class_id, method, args, span)
    }

    /// `Object` methods on an array receiver (already on the stack): identity
    /// `equals`/`hashCode`, and `toString`/`getClass`.
    #[allow(clippy::option_option)] // matches the call-dispatch return shape
    fn array_object_call(
        &mut self,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let (ret, descriptor): (JType, &str) = match (method, args.len()) {
            ("equals", 1) => (JType::Boolean, "(Ljava/lang/Object;)Z"),
            ("hashCode", 0) => (JType::Int, "()I"),
            ("toString", 0) => (JType::Str, "()Ljava/lang/String;"),
            ("getClass", 0) => (JType::Class, "()Ljava/lang/Class;"),
            _ => {
                self.error(span, format!("cannot call {method}(...) on an array"));
                return None;
            }
        };
        for arg in args {
            self.expr(arg);
        }
        let method_ref = intern_method_ref(self.pool, "java/lang/Object", method, descriptor);
        self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
        self.code
            .drop_stack(u16::try_from(1 + args.len()).unwrap_or(1)); // receiver + args
        Some(Some(ret))
    }

    /// Resolve and emit an intrinsic instance call (String / Scanner /
    /// `ArrayList`); the receiver value is already on the stack.
    #[allow(clippy::option_option, clippy::too_many_lines)]
    fn builtin_instance_call(
        &mut self,
        receiver_ty: JType,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        if receiver_ty == JType::Writer && matches!(method, "printf" | "format") {
            let (tags, width) = self.emit_format_varargs(args, span)?;
            // `printf` returns void; `format` returns the writer (for chaining).
            let (ret_desc, ret_width, ret_ty) = if method == "format" {
                ("Ljava/io/PrintWriter;", 1, Some(JType::Writer))
            } else {
                ("V", 0, None)
            };
            let descriptor = format!("(Ljava/lang/String;{tags}){ret_desc}");
            let method_ref =
                intern_method_ref(self.pool, "java/io/PrintWriter", method, &descriptor);
            self.code
                .push_op_u16(op::INVOKEVIRTUAL, method_ref, ret_width);
            self.code.drop_stack(1 + width);
            return Some(ret_ty);
        }
        // Method.invoke(Object receiver, Object... args): pack the trailing
        // varargs into an Object[] (autoboxing primitives).
        if receiver_ty == JType::Method && method == "invoke" {
            return self.emit_method_invoke(args, span);
        }
        // `collection.stream()` → a Stream of the receiver's element type. Works
        // on any collection (list, set, tree, queue, or a map view); the VM
        // dispatches by the receiver object, so the invokevirtual class is only
        // nominal.
        if method == "stream"
            && args.is_empty()
            && matches!(
                receiver_ty,
                JType::List(_)
                    | JType::Stack(_)
                    | JType::LinkedList { .. }
                    | JType::Set(_)
                    | JType::TreeSet(_)
                    | JType::Collection(_)
            )
            && let Some(elem) = TypeArgs::of(receiver_ty).first
        {
            let (class, _) = builtin_instance_table(receiver_ty).expect("collection has a table");
            let method_ref =
                intern_method_ref(self.pool, class, "stream", "()Ljava/util/stream/Stream;");
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            return Some(Some(JType::Stream(elem)));
        }
        // `stream.collect(collector)`: the result type comes from the collector
        // — `joining()` is a `String`, `toList()`/`toSet()` a List/Set of the
        // stream's element (or, once `map` has erased it, `null` adopting the
        // assignment context, like a diamond).
        if let JType::Stream(elem) = receiver_ty
            && method == "collect"
            && args.len() == 1
        {
            let result_ty = self.collector_result_type(&args[0], elem);
            self.expr(&args[0]);
            let method_ref = intern_method_ref(
                self.pool,
                "java/util/stream/Stream",
                "collect",
                "(Ljava/util/stream/Collector;)Ljava/lang/Object;",
            );
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(2);
            return Some(Some(result_ty));
        }
        let (class, methods) =
            builtin_instance_table(receiver_ty).expect("caller checked receiver kind");
        // A real Java member caturra cannot model: say so before type-checking
        // the arguments, whose own errors would mask it. `map.forEach(...)`
        // should blame the missing lambda support, not the lambda.
        let receiver_class = receiver_class_name(receiver_ty);
        if !methods.iter().any(|m| m.name == method)
            && let Some(reason) = unsupported_member(receiver_class, method)
        {
            self.error(
                span,
                format!("{receiver_class}.{method} exists in Java, but {reason}"),
            );
            return None;
        }
        let elem = TypeArgs::of(receiver_ty);
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            for arg in args {
                self.expr(arg);
            }
            return None;
        }

        let Some(chosen) = pick_builtin(methods, method, &arg_types, elem, self.table) else {
            if methods.iter().any(|m| m.name == method) {
                self.error(
                    span,
                    format!(
                        "no suitable method found for {method}({}) in class {}",
                        describe_types(&arg_types, self.table),
                        receiver_ty.describe(self.table)
                    ),
                );
            } else {
                self.error(
                    span,
                    format!(
                        "cannot find symbol: method {method}({}) in class {}",
                        describe_types(&arg_types, self.table),
                        receiver_ty.describe(self.table)
                    ),
                );
            }
            return None;
        };

        let mut args_width: u16 = 0;
        for (arg, param) in args.iter().zip(chosen.params) {
            let param_ty = bparam_type(*param, elem);
            let actual = self.expr(arg);
            if matches!(param_ty, JType::Boxed(_)) {
                // `numeric_conversion` unboxes a wrapper argument but never
                // boxes a primitive one, which `Map.put(K, V)` needs.
                self.convert_for_assignment(actual, param_ty, span);
            } else {
                self.numeric_conversion(actual, param_ty);
            }
            args_width += param_ty.width();
        }
        let method_ref = intern_method_ref(self.pool, class, chosen.name, chosen.descriptor);
        let ret = bret_type(chosen.ret, elem, self.table);
        let ret_width = ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKEVIRTUAL, method_ref, ret_width);
        self.code.drop_stack(1 + args_width);
        Some(ret)
    }

    /// Emit the format string + variadic arguments of a
    /// `format`/`printf` call, returning the argument portion of the
    /// synthesized descriptor (which carries each argument's type tag
    /// to the VM) and the pushed stack width.
    fn emit_format_varargs(&mut self, args: &[Expr], span: SourceSpan) -> Option<(String, u16)> {
        let [template, rest @ ..] = args else {
            self.error(span, "format needs a format string");
            return None;
        };
        let template_ty = self.expr(template);
        if template_ty != JType::Str && template_ty != JType::Error {
            self.error(
                template.span(),
                format!(
                    "incompatible types: {} cannot be converted to String",
                    template_ty.describe(self.table)
                ),
            );
        }
        let mut tags = String::new();
        let mut width: u16 = 1;
        for arg in rest {
            let ty = self.expr(arg);
            // Objects format via toString, like Java's %s.
            let ty = match ty {
                JType::Object(_)
                | JType::List(_)
                | JType::Stack(_)
                | JType::Map { .. }
                | JType::Set(_)
                | JType::Collection(_)
                | JType::EntrySet { .. }
                | JType::MapEntry { .. }
                | JType::File
                | JType::Exception(_) => self.coerce_to_string_for_output(ty),
                other => other,
            };
            match ty {
                JType::Int => {
                    tags.push('I');
                    width += 1;
                }
                JType::Double => {
                    tags.push('D');
                    width += 2;
                }
                JType::Long => {
                    tags.push('J');
                    width += 2;
                }
                JType::Float => {
                    tags.push('F');
                    width += 1;
                }
                JType::Short => {
                    tags.push('S');
                    width += 1;
                }
                JType::Byte => {
                    tags.push('B');
                    width += 1;
                }
                JType::Char => {
                    tags.push('C');
                    width += 1;
                }
                JType::Boolean => {
                    tags.push('Z');
                    width += 1;
                }
                JType::Str | JType::Null => {
                    tags.push_str("Ljava/lang/String;");
                    width += 1;
                }
                JType::Error => return None,
                other => {
                    self.error(
                        arg.span(),
                        format!("cannot format {}", other.describe(self.table)),
                    );
                    return None;
                }
            }
        }
        Some((tags, width))
    }

    /// Emit an intrinsic static call (`Math.abs(...)`, ...).
    #[allow(clippy::option_option)]
    fn builtin_static_call(
        &mut self,
        class: &str,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        // `String.format` is variadic — the one call shape the fixed
        // signature tables cannot express.
        if class == "String" && method == "format" {
            let (tags, width) = self.emit_format_varargs(args, span)?;
            let descriptor = format!("(Ljava/lang/String;{tags})Ljava/lang/String;");
            let method_ref =
                intern_method_ref(self.pool, "java/lang/String", "format", &descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            self.code.drop_stack(width);
            return Some(Some(JType::Str));
        }
        let (jvm_class, methods) = builtin_static_table(class).expect("caller checked");
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            for arg in args {
                self.expr(arg);
            }
            return None;
        }
        let Some(chosen) =
            pick_builtin(methods, method, &arg_types, TypeArgs::default(), self.table)
        else {
            if let Some(reason) = unsupported_member(class, method) {
                self.error(
                    span,
                    format!("{class}.{method} exists in Java, but {reason}"),
                );
            } else if methods.iter().any(|m| m.name == method) {
                // The name exists, so the symbol is found; no overload takes
                // these arguments. javac says the same, and the instance-call
                // path already did.
                self.error(
                    span,
                    format!(
                        "no suitable method found for {method}({}) in class {class}",
                        describe_types(&arg_types, self.table)
                    ),
                );
            } else {
                self.error(
                    span,
                    format!(
                        "cannot find symbol: method {method}({}) in class {class}",
                        describe_types(&arg_types, self.table)
                    ),
                );
            }
            return None;
        };
        let mut args_width: u16 = 0;
        for (arg, param) in args.iter().zip(chosen.params) {
            let param_ty = bparam_type(*param, TypeArgs::default());
            let actual = self.expr(arg);
            self.numeric_conversion(actual, param_ty);
            args_width += param_ty.width();
        }
        let method_ref = intern_method_ref(self.pool, jvm_class, chosen.name, chosen.descriptor);
        let ret = bret_type(chosen.ret, TypeArgs::default(), self.table);
        let ret_width = ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKESTATIC, method_ref, ret_width);
        self.code.drop_stack(args_width);
        Some(ret)
    }

    /// The receiver object is already on the stack; resolve the
    /// overload, emit arguments, and invoke.
    #[allow(clippy::option_option)]
    fn emit_virtual_call_on_stacked_receiver(
        &mut self,
        class_id: ClassId,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let class_name = self.table.class_name(class_id).to_owned();
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            for arg in args {
                self.expr(arg);
            }
            return None;
        }

        let table = self.table;
        let sig = match table.resolve(&class_name, method, &arg_types) {
            Resolution::Found(sig) => sig.clone(),
            Resolution::UnknownName => {
                // Throwable-descended classes inherit getMessage /
                // toString from their library parent.
                if self.table.is_throwable(class_id)
                    && (method == "getMessage" || method == "toString")
                    && args.is_empty()
                {
                    let method_ref =
                        intern_method_ref(self.pool, &class_name, method, "()Ljava/lang/String;");
                    self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
                    self.code.drop_stack(1);
                    return Some(Some(JType::Str));
                }
                self.error(
                    span,
                    format!(
                        "cannot find symbol: method {method}({}) in class {class_name}",
                        describe_types(&arg_types, self.table)
                    ),
                );
                return None;
            }
            Resolution::NoneApplicable => {
                self.error(
                    span,
                    format!(
                        "no suitable method found for {method}({}) in class {class_name}",
                        describe_types(&arg_types, self.table)
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
        if sig.is_private && class_id != self.current_class_id && !self.table.relax_access {
            self.error(
                span,
                format!("{method}() has private access in {class_name}"),
            );
            return None;
        }
        // `obj.staticMethod(...)`: legal if discouraged (javac warns under
        // `-Xlint:static`). The receiver expression is evaluated for its side
        // effects and then discarded, so a `null` receiver does NOT throw —
        // nothing is dereferenced. `obj.staticField` already behaved this way.
        if sig.is_static {
            self.code.push_op(op::POP, 0);
            self.code.drop_stack(1);
            let args_width = self.emit_call_args(args, &sig, span);
            let descriptor = sig.descriptor(self.table);
            let method_ref = intern_method_ref(self.pool, &class_name, method, &descriptor);
            let ret_width = sig.ret.map_or(0, JType::width);
            self.code
                .push_op_u16(op::INVOKESTATIC, method_ref, ret_width);
            self.code.drop_stack(args_width);
            return Some(sig.ret);
        }

        let args_width = self.emit_call_args(args, &sig, span);
        let descriptor = sig.descriptor(self.table);
        let method_ref = intern_method_ref(self.pool, &class_name, method, &descriptor);
        let ret_width = sig.ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKEVIRTUAL, method_ref, ret_width);
        self.code.drop_stack(1 + args_width);
        Some(sig.ret)
    }

    /// Coerce a value on the stack into a `String` for printing or
    /// concatenation: objects go through their `toString()` (the VM
    /// supplies `ClassName@hex` when a class doesn't define one).
    fn coerce_to_string_for_output(&mut self, ty: JType) -> JType {
        if let JType::Exception(id) = ty {
            let method_ref = intern_method_ref(
                self.pool,
                exception_internal(id),
                "toString",
                "()Ljava/lang/String;",
            );
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            return JType::Str;
        }
        if ty == JType::File {
            let method_ref = intern_method_ref(
                self.pool,
                "java/io/File",
                "toString",
                "()Ljava/lang/String;",
            );
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            return JType::Str;
        }
        if ty == JType::StringBuilder {
            let method_ref = intern_method_ref(
                self.pool,
                "java/lang/StringBuilder",
                "toString",
                "()Ljava/lang/String;",
            );
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            return JType::Str;
        }
        if let Some(class) = match ty {
            JType::List(_) => Some("java/util/ArrayList"),
            JType::Stack(_) => Some("java/util/Stack"),
            _ => None,
        } {
            let method_ref =
                intern_method_ref(self.pool, class, "toString", "()Ljava/lang/String;");
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            return JType::Str;
        }
        if let Some((class, _)) = builtin_instance_table(ty)
            && matches!(
                ty,
                JType::Map { .. }
                    | JType::Set(_)
                    | JType::Collection(_)
                    | JType::EntrySet { .. }
                    | JType::MapEntry { .. }
            )
        {
            let method_ref =
                intern_method_ref(self.pool, class, "toString", "()Ljava/lang/String;");
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            return JType::Str;
        }
        // A boxed wrapper is left alone: it may be null (`map.get(absent)`),
        // and printing/appending it as an Object renders "null" as Java does
        // rather than throwing on `Integer.toString()`.
        if let JType::Object(class_id) = ty {
            // String.valueOf semantics: null prints as "null" instead
            // of throwing, so guard the toString call.
            let class_name = self.table.class_name(class_id).to_owned();
            let null_case = self.code.new_label();
            let done = self.code.new_label();
            self.code.push_op(op::DUP, 1);
            self.code.branch(op::IFNULL, null_case, 1);
            let method_ref =
                intern_method_ref(self.pool, &class_name, "toString", "()Ljava/lang/String;");
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
            self.code.drop_stack(1);
            self.code.branch(op::GOTO, done, 0);
            self.code.bind(null_case);
            self.code.push_op(op::POP, 0);
            self.code.drop_stack(1);
            let utf8 = self.pool.intern_utf8("null");
            let index = self.pool.intern(Constant::String { string_index: utf8 });
            self.code.push_ldc(index);
            self.code.bind(done);
            JType::Str
        } else {
            ty
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
                    format!(
                        "array required, but {} found",
                        array_ty.describe(self.table)
                    ),
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
                    index_ty.describe(self.table)
                ),
            );
        }
        Some(element)
    }

    /// The array-load opcode for an element type.
    fn xaload(&mut self, element: JType) {
        let opcode = match element {
            JType::Double => op::DALOAD,
            JType::Long => op::LALOAD,
            JType::Float => op::FALOAD,
            JType::Short => op::SALOAD,
            JType::Byte | JType::Boolean => op::BALOAD,
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
            JType::Long => op::LASTORE,
            JType::Float => op::FASTORE,
            JType::Short => op::SASTORE,
            JType::Byte | JType::Boolean => op::BASTORE,
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
                self.convert_for_assignment_const(
                    value_ty,
                    element,
                    value.span(),
                    constant_int_value(value),
                );
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
                // A wrapper value unboxes first; array elements are already
                // primitive here.
                let operand = numeric_view(value_ty);
                if element == JType::Boolean || !element.is_numeric() || !operand.is_numeric() {
                    if value_ty != JType::Error {
                        self.error(
                            span,
                            format!(
                                "operator '{}' cannot be applied to {} and {}",
                                compound_symbol(op_kind),
                                element.describe(self.table),
                                value_ty.describe(self.table)
                            ),
                        );
                    }
                    return;
                }
                let promoted = promote(element, operand);
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
        // Every intrinsic collection compiles to the same index loop: caturra
        // has no iterators, so each exposes a positional accessor instead.
        let indexed = match iterable_ty {
            JType::List(elem) | JType::Stack(elem) | JType::LinkedList { elem, .. } => {
                Some(("get", elem.base_type()))
            }
            JType::Set(elem) | JType::TreeSet(elem) | JType::Collection(elem) => {
                Some(("__get", boxed_if_primitive(Some(elem))))
            }
            JType::EntrySet { key, value } => Some(("__get", JType::MapEntry { key, value })),
            _ => None,
        };
        if let Some((accessor, element)) = indexed {
            self.for_each_indexed(ty, name, iterable_ty, accessor, element, body, span);
            return;
        }
        let Some(element) = iterable_ty.element_type() else {
            if iterable_ty != JType::Error {
                self.error(
                    iterable.span(),
                    format!(
                        "for-each needs an array, an ArrayList, or a map view, but {} found",
                        iterable_ty.describe(self.table)
                    ),
                );
            }
            return;
        };
        let Some(var_ty) = self.table.resolve_type(ty) else {
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
        self.record_local_debug(name, var_ty, var_slot);
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
            is_loop: true,
            label: self.pending_label.take(),
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

    /// `for (T x : list)` desugared to an indexed loop over
    /// `size()`/`get(int)` (the list reference is already on the stack).
    /// The enhanced-for loop over an intrinsic collection, compiled to an
    /// index loop over `size()` and a positional accessor. `accessor` is
    /// `get` for an `ArrayList` and caturra's own `__get` for a map view,
    /// which has no positional method in Java because it has an iterator.
    #[allow(clippy::too_many_arguments)] // the loop's five moving parts
    fn for_each_indexed(
        &mut self,
        ty: &TypeRef,
        name: &str,
        iterable_ty: JType,
        accessor: &str,
        element: JType,
        body: &Stmt,
        span: SourceSpan,
    ) {
        let (class, _) = builtin_instance_table(iterable_ty).expect("an intrinsic collection");
        let Some(var_ty) = self.table.resolve_type(ty) else {
            self.error(span, "unknown type for the for-each variable");
            self.code.discard();
            return;
        };

        let list_slot = self.next_slot;
        self.next_slot += 1;
        let index_slot = self.next_slot;
        self.next_slot += 1;
        self.emit_store(list_slot, iterable_ty);
        self.code.push_op(op::ICONST_0, 1);
        self.emit_store(index_slot, JType::Int);

        self.scopes.push(Vec::new());
        if self.lookup(name).is_some() {
            self.error(
                span,
                format!("variable '{name}' is already defined in this method"),
            );
        }
        let var_slot = self.next_slot;
        self.next_slot += var_ty.width();
        self.record_local_debug(name, var_ty, var_slot);
        self.scopes.last_mut().expect("scope pushed").push((
            name.to_owned(),
            LocalVar {
                slot: var_slot,
                ty: var_ty,
                is_final: false,
                assigned: true,
            },
        ));

        let size_ref = intern_method_ref(self.pool, class, "size", "()I");
        let get_ref = intern_method_ref(self.pool, class, accessor, "(I)Ljava/lang/Object;");

        let cond_label = self.code.new_label();
        let continue_label = self.code.new_label();
        let end = self.code.new_label();

        self.code.bind(cond_label);
        self.emit_load(index_slot, JType::Int);
        self.emit_load(list_slot, iterable_ty);
        self.code.push_op_u16(op::INVOKEVIRTUAL, size_ref, 1);
        self.code.drop_stack(1);
        self.code.branch(op::IF_ICMPGE, end, 2);

        self.emit_load(list_slot, iterable_ty);
        self.emit_load(index_slot, JType::Int);
        self.code
            .push_op_u16(op::INVOKEVIRTUAL, get_ref, element.width());
        self.code.drop_stack(2);
        self.convert_for_assignment(element, var_ty, span);
        self.emit_store(var_slot, var_ty);

        let before = self.assigned_flags();
        self.loop_stack.push(LoopLabels {
            break_label: end,
            continue_label,
            is_loop: true,
            label: self.pending_label.take(),
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
        let outcome = match expr {
            Expr::Call {
                receiver,
                method,
                args,
                span,
            } => match self.call_target(receiver.as_deref(), *span) {
                None => None,
                Some(CallTarget::Stream(stream)) => {
                    self.print_call(stream, method, args, *span);
                    None
                }
                Some(CallTarget::Static(class)) => self.static_call(&class, method, args, *span),
                Some(CallTarget::Own) => self.own_call(method, args, *span),
                Some(CallTarget::Instance(object)) => {
                    self.instance_call(object, method, args, *span)
                }
            },
            // `super.method(...);` as a statement.
            Expr::SuperMethodCall { method, args, span } => {
                self.super_method_call(method, args, *span)
            }
            // `new Foo();` — a class instance creation is a statement
            // expression (JLS §14.8). Run the constructor for its effects and
            // drop the reference, which the discard below does.
            Expr::NewObject { .. } => Some(Some(self.expr(expr))),
            _ => {
                self.error(expr.span(), "this expression is not a statement in Java");
                return;
            }
        };
        // The result of a call statement (if any) is discarded.
        if let Some(Some(ty)) = outcome {
            let opcode = if ty.width() == 2 { op::POP2 } else { op::POP };
            self.code.push_op(opcode, 0);
            self.code.drop_stack(ty.width());
        }
    }

    /// Classify what a call's receiver refers to, reporting an error
    /// for unsupported shapes.
    fn call_target<'e>(
        &mut self,
        receiver: Option<&'e Expr>,
        _span: SourceSpan,
    ) -> Option<CallTarget<'e>> {
        match receiver {
            None => Some(CallTarget::Own),
            Some(
                expr @ Expr::Name {
                    path,
                    span: receiver_span,
                },
            ) => match self
                .strip_package_prefix(path)
                .as_deref()
                .unwrap_or(path)
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()[..]
            {
                ["System", "out"] => Some(CallTarget::Stream("out")),
                ["System", "err"] => Some(CallTarget::Stream("err")),
                [single] => {
                    if self.lookup(single).is_some() {
                        Some(CallTarget::Instance(expr))
                    } else if self.table.has_class(single)
                        || builtin_static_table(single).is_some()
                        // `Optional` holds static factories (`of`/`empty`/
                        // `ofNullable`) but is not a bundled class nor in a fixed
                        // static table — its return type is argument-dependent.
                        || single == "Optional"
                    {
                        Some(CallTarget::Static(single.to_owned()))
                    } else if self.table.field(self.current_class, single).is_some()
                        || self.enclosing_static_field(single).is_some()
                    {
                        // A field of the current class used as receiver — or a
                        // static field of the class this lambda came from.
                        Some(CallTarget::Instance(expr))
                    } else {
                        self.error(*receiver_span, format!("cannot find symbol: '{single}'"));
                        None
                    }
                }
                // `java.util.Nope.f()`: a qualified library name that
                // does not resolve. Say so as javac does, rather than
                // letting the field-access chain below report a missing
                // variable named `java`. Reached only when the prefix
                // did not strip, so a real library name never lands here.
                ["java", package, class, ..] if self.lookup("java").is_none() => {
                    let dotted = format!("java.{package}.{class}");
                    self.error(
                        *receiver_span,
                        crate::imports::unknown_qualified_message(&dotted),
                    );
                    None
                }
                // Dotted receivers (p.pos.move()) are general
                // expressions; name() knows how to read them.
                _ => Some(CallTarget::Instance(expr)),
            },
            Some(other) => Some(CallTarget::Instance(other)),
        }
    }

    /// A bare call `method(args)`: static → invokestatic; instance →
    /// through the implicit `this`.
    #[allow(clippy::option_option)]
    fn own_call(&mut self, method: &str, args: &[Expr], span: SourceSpan) -> Option<Option<JType>> {
        // Peek resolution to decide static vs instance dispatch.
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        let table = self.table;
        let own = table.resolve(self.current_class, method, &arg_types);
        let is_instance = matches!(&own, Resolution::Found(sig) if !sig.is_static);
        // Unqualified call to a `import static X.*` member (JUnit
        // `assertTrue(...)` → `Assertions.assertTrue(...)`).
        if !matches!(own, Resolution::Found(_)) {
            for class in self.table.static_imports.clone() {
                if matches!(
                    self.table.resolve(&class, method, &arg_types),
                    Resolution::Found(sig) if sig.is_static
                ) {
                    return self.static_call(&class, method, args, span);
                }
            }
        }
        if is_instance {
            if self.in_static {
                self.error(
                    span,
                    format!(
                        "non-static method {method}() cannot be referenced from a static \
                         context (instance methods need an object)"
                    ),
                );
                return None;
            }
            self.code.push_op(op::ALOAD_0, 1);
            return self.emit_virtual_call_on_stacked_receiver(
                self.current_class_id,
                method,
                args,
                span,
            );
        }
        // A bare call to an enclosing instance method, from inside a lambda:
        // `() -> helper()` is `__caturraOuter.helper()`.
        if !matches!(own, Resolution::Found(_))
            && !self.in_constructor
            && let Some((_, enclosing)) = self.captured_outer()
        {
            let enc_name = self.table.class_name(enclosing).to_owned();
            let found_instance = matches!(
                self.table.resolve(&enc_name, method, &arg_types),
                Resolution::Found(sig) if !sig.is_static
            );
            if found_instance {
                let outer = Expr::This { span };
                self.expr(&outer); // pushes the enclosing instance
                return self.emit_virtual_call_on_stacked_receiver(enclosing, method, args, span);
            }
        }
        self.static_call(self.current_class, method, args, span)
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
        if !self.table.has_class(class) && builtin_static_table(class).is_some() {
            return self.builtin_static_call(class, method, args, span);
        }
        // `Arrays.asList` is variadic (`<T> List<T> asList(T...)`): a lone
        // array argument is the varargs array; anything else packs into one.
        if class == "Arrays" && method == "asList" {
            return self.emit_arrays_as_list(args, span);
        }
        // `Arrays.deepToString/deepEquals/deepHashCode` recurse into element
        // arrays, which needs each element array's kind at run time. The VM
        // answers them; the bundled Java cannot.
        if class == "Arrays" && arrays_deep_signature(method).is_some() {
            return self.emit_arrays_deep_call(method, args, span);
        }
        // `Arrays.copyOf/copyOfRange/fill/binarySearch` are one VM method each
        // rather than nine bundled overloads: the heap knows the element kind,
        // and `copyOf` must return an array of the source's own type, which a
        // bundled `Object[] copyOf(Object[], int)` could not.
        if class == "Arrays" && is_arrays_array_method(method) {
            return self.emit_arrays_array_call(method, args, span);
        }
        // `Arrays.setAll(array, i -> ...)` fills the array from a generator of
        // the index; the generator is already the erased `__UnaryOperator`.
        if class == "Arrays" && method == "setAll" {
            return self.emit_arrays_set_all(args, span);
        }
        // `Collections.reverse/swap` are generic over the list element type;
        // emit the (uniform-at-runtime) list argument(s) and call the bundle,
        // bypassing invariant List<elem> parameter matching.
        if class == "Collections" && is_collections_method(method) {
            return self.emit_collections_call(method, args, span);
        }
        // `Optional.of/ofNullable(x)` build an Optional of the argument's type;
        // `Optional.empty()` adopts its context (types like `null`). The return
        // type is argument-dependent, so no fixed static table fits.
        if class == "Optional" && matches!(method, "of" | "ofNullable" | "empty") {
            return self.emit_optional_static(method, args, span);
        }
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
                        describe_types(&arg_types, self.table)
                    ),
                );
                return None;
            }
            Resolution::NoneApplicable => {
                self.error(
                    span,
                    format!(
                        "no suitable method found for {method}({}) in class {class}",
                        describe_types(&arg_types, self.table)
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
                    "method {} uses types not yet supported by caturra",
                    sig.describe(self.table)
                ),
            );
            return None;
        }

        let args_width = self.emit_call_args(args, &sig, span);
        let method_ref = intern_method_ref(self.pool, class, method, &sig.descriptor(self.table));
        let ret_width = sig.ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKESTATIC, method_ref, ret_width);
        self.code.drop_stack(args_width);
        // `Arrays.asList` is generic (`<T> List<T>`): the bundle returns a raw
        // `ArrayList`, but type the result as a `List` of the argument array's
        // element type so `List<String> x = Arrays.asList(strs)` type-checks.
        Some(sig.ret)
    }

    /// `Optional.of(x)` / `Optional.ofNullable(x)` / `Optional.empty()`. The VM
    /// stores the value directly (unboxed, as caturra stores every element), so
    /// the call passes the primitive with a descriptor built from its type,
    /// exactly as `Collections.nCopies` does — no boxing. `empty()` returns a
    /// `null`-typed Optional that adopts its assignment context.
    #[allow(clippy::option_option)] // call-dispatch return shape
    fn emit_optional_static(
        &mut self,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        if method == "empty" {
            if !args.is_empty() {
                self.no_suitable_library_method("Optional", method, args, span);
                return None;
            }
            let method_ref =
                intern_method_ref(self.pool, "Optional", "empty", "()Ljava/util/Optional;");
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            return Some(Some(JType::Null));
        }
        let [value] = args else {
            self.no_suitable_library_method("Optional", method, args, span);
            return None;
        };
        let value_ty = self.expr(value);
        let Some(elem) = elem_type_of(value_ty) else {
            self.error(
                value.span(),
                format!(
                    "Optional.{method} cannot hold {}",
                    value_ty.describe(self.table)
                ),
            );
            self.code.discard();
            return None;
        };
        let descriptor = format!(
            "({})Ljava/util/Optional;",
            elem.base_type().descriptor(self.table)
        );
        let method_ref = intern_method_ref(self.pool, "Optional", method, &descriptor);
        self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
        self.code.drop_stack(value_ty.width());
        Some(Some(JType::Optional(elem)))
    }

    /// `Collections.reverse(list)` / `Collections.swap(list, i, j)`: emit the
    /// arguments and call the bundle directly (the list element type does not
    /// affect the runtime representation).
    #[allow(clippy::option_option, clippy::unnecessary_wraps)] // call-dispatch return shape
    #[allow(clippy::too_many_lines)] // one method per arm
    fn emit_collections_call(
        &mut self,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        // `emptyList()` has no arguments at all. Its element type comes from
        // whatever it is assigned to, so it types as `null` does: assignable
        // to any list.
        if method == "emptyList" {
            if !args.is_empty() {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            }
            let method_ref = intern_method_ref(
                self.pool,
                "Collections",
                "emptyList",
                "()Ljava/util/ArrayList;",
            );
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            return Some(Some(JType::Null));
        }
        // `reverseOrder()` / `reverseOrder(cmp)` build a `Comparator` (a reversed
        // one), the same value `Comparator.reverseOrder()`/`reversed()` produce.
        if method == "reverseOrder" {
            match args {
                [] => {}
                [comparator] => {
                    let ty = self.expr(comparator);
                    if !self.is_comparator_type(ty) {
                        self.error(
                            comparator.span(),
                            "Collections.reverseOrder(cmp) takes a Comparator",
                        );
                        self.code.discard();
                        return None;
                    }
                }
                _ => {
                    self.no_suitable_library_method("Collections", method, args, span);
                    return None;
                }
            }
            let descriptor = if args.is_empty() {
                "()Ljava/util/Comparator;"
            } else {
                "(Ljava/util/Comparator;)Ljava/util/Comparator;"
            };
            let method_ref =
                intern_method_ref(self.pool, "Collections", "reverseOrder", descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            self.code.drop_stack(args.len() as u16);
            return Some(Some(
                self.table
                    .class_id("__Comparator")
                    .map_or(JType::Error, JType::Object),
            ));
        }
        // `singletonList(e)` builds an immutable one-element `List` of the
        // argument's type — the element passes through unboxed, like `nCopies`.
        if method == "singletonList" {
            let [value] = args else {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            };
            let value_ty = self.expr(value);
            let Some(elem) = elem_type_of(value_ty) else {
                self.error(
                    value.span(),
                    format!(
                        "Collections.singletonList cannot make a list of {}",
                        value_ty.describe(self.table)
                    ),
                );
                self.code.discard();
                return None;
            };
            let descriptor = format!(
                "({})Ljava/util/ArrayList;",
                elem.base_type().descriptor(self.table)
            );
            let method_ref =
                intern_method_ref(self.pool, "Collections", "singletonList", &descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            self.code.drop_stack(value_ty.width());
            return Some(Some(JType::List(elem)));
        }
        // `emptySet()`/`emptyMap()` — immutable empty collections whose element
        // type comes from the assignment context, so they type as `null`.
        if method == "emptySet" || method == "emptyMap" {
            if !args.is_empty() {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            }
            let descriptor = if method == "emptySet" {
                "()Ljava/util/Set;"
            } else {
                "()Ljava/util/Map;"
            };
            let method_ref = intern_method_ref(self.pool, "Collections", method, descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            return Some(Some(JType::Null));
        }
        // `singleton(e)` — an immutable one-element `Set` of the argument's type.
        if method == "singleton" {
            let [value] = args else {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            };
            let value_ty = self.expr(value);
            let Some(elem) = elem_type_of(value_ty) else {
                self.error(
                    value.span(),
                    format!(
                        "Collections.singleton cannot make a set of {}",
                        value_ty.describe(self.table)
                    ),
                );
                self.code.discard();
                return None;
            };
            let descriptor = format!(
                "({})Ljava/util/Set;",
                elem.base_type().descriptor(self.table)
            );
            let method_ref = intern_method_ref(self.pool, "Collections", "singleton", &descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            self.code.drop_stack(value_ty.width());
            return Some(Some(JType::Set(elem)));
        }
        // `singletonMap(k, v)` — an immutable one-entry `Map`.
        if method == "singletonMap" {
            let [key_arg, value_arg] = args else {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            };
            let key_ty = self.expr(key_arg);
            let Some(key) = elem_type_of(key_ty) else {
                self.error(
                    key_arg.span(),
                    "Collections.singletonMap key has no element type",
                );
                self.code.discard();
                return None;
            };
            let value_ty = self.expr(value_arg);
            let Some(value) = elem_type_of(value_ty) else {
                self.error(
                    value_arg.span(),
                    "Collections.singletonMap value has no element type",
                );
                self.code.discard();
                return None;
            };
            let descriptor = format!(
                "({}{})Ljava/util/Map;",
                key.base_type().descriptor(self.table),
                value.base_type().descriptor(self.table)
            );
            let method_ref =
                intern_method_ref(self.pool, "Collections", "singletonMap", &descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            self.code.drop_stack(key_ty.width() + value_ty.width());
            return Some(Some(JType::Map { key, value }));
        }
        // `unmodifiableSet(set)` / `unmodifiableMap(map)` — an immutable view of
        // the argument, keeping its own set/map type.
        if method == "unmodifiableSet" || method == "unmodifiableMap" {
            let [collection] = args else {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            };
            let collection_ty = self.expr(collection);
            let wants_set = method == "unmodifiableSet";
            let ok = if wants_set {
                matches!(collection_ty, JType::Set(_) | JType::TreeSet(_))
            } else {
                matches!(collection_ty, JType::Map { .. } | JType::TreeMap { .. })
            };
            if !ok {
                self.error(
                    collection.span(),
                    format!(
                        "Collections.{method} takes a {}",
                        if wants_set { "Set" } else { "Map" }
                    ),
                );
                self.code.discard();
                return None;
            }
            let descriptor = if wants_set {
                "(Ljava/util/Set;)Ljava/util/Set;"
            } else {
                "(Ljava/util/Map;)Ljava/util/Map;"
            };
            let method_ref = intern_method_ref(self.pool, "Collections", method, descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            self.code.drop_stack(1);
            return Some(Some(collection_ty));
        }
        // `nCopies(n, value)` is the only other one whose first argument is
        // not a list; its element type comes from the value.
        if method == "nCopies" {
            let [count, value] = args else {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            };
            let count_ty = self.expr(count);
            self.numeric_conversion(count_ty, JType::Int);
            let value_ty = self.expr(value);
            let Some(elem) = elem_type_of(value_ty) else {
                self.error(
                    value.span(),
                    format!(
                        "Collections.nCopies cannot make a list of {}",
                        value_ty.describe(self.table)
                    ),
                );
                self.code.discard();
                return None;
            };
            let descriptor = format!(
                "(I{})Ljava/util/ArrayList;",
                elem.base_type().descriptor(self.table)
            );
            let method_ref = intern_method_ref(self.pool, "Collections", "nCopies", &descriptor);
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
            self.code.drop_stack(1 + value_ty.width());
            return Some(Some(JType::List(elem)));
        }

        // `Collections.sort(list, comparator)` — the comparator is a
        // desugared `__Comparator`, and the element need not be Comparable.
        if method == "sort" && args.len() == 2 {
            let JType::List(elem) = self.type_of(&args[0]) else {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            };
            let is_comparator = matches!(
                (self.type_of(&args[1]), self.table.class_id("__Comparator")),
                (JType::Object(id), Some(target)) if self.table.is_subtype(id, target)
            );
            if !is_comparator {
                self.no_suitable_library_method("Collections", method, args, span);
                return None;
            }
            self.expr(&args[0]);
            self.expr(&args[1]);
            let method_ref = intern_method_ref(
                self.pool,
                "Collections",
                "sort",
                "(Ljava/util/ArrayList;Ljava/lang/Object;)V",
            );
            self.code.push_op_u16(op::INVOKESTATIC, method_ref, 0);
            self.code.drop_stack(2);
            let _ = elem;
            return Some(None);
        }
        let want = match method {
            "reverse" | "sort" | "max" | "min" | "unmodifiableList" => 1usize,
            "frequency" | "binarySearch" => 2,
            "swap" => 3,
            // `addAll(list, elements...)` is variadic.
            "addAll" => args.len().max(1),
            // `shuffle(list)` or `shuffle(list, random)`.
            "shuffle" => {
                if args.len() == 2 {
                    2
                } else {
                    1
                }
            }
            _ => unreachable!("caller checked method"),
        };
        if args.len() != want {
            self.no_suitable_library_method("Collections", method, args, span);
            return None;
        }
        let list_ty = self.type_of(&args[0]);
        let JType::List(elem) = list_ty else {
            // javac reports this as overload resolution failing, not as one
            // argument's type: `no suitable method found for max(int)`.
            self.no_suitable_library_method("Collections", method, args, span);
            return None;
        };
        let element_ty = elem.base_type();
        let element_descriptor = element_ty.descriptor(self.table);

        // `sort`/`max`/`min`/`binarySearch` are declared over
        // `T extends Comparable<? super T>`, so a list of a class that does
        // not implement `Comparable` is a compile error, not a runtime
        // `ClassCastException`. `Arrays.sort` already refuses one, because
        // its bundled parameter is `Comparable[]`; `Collections` is native
        // and so has to ask. javac reports the unsatisfied bound as overload
        // resolution failing, which is what `no_suitable_library_method`
        // says. Only a class whose supertypes we can see is refused: the
        // wrappers and `String` are all `Comparable`, and an erased type
        // variable may well be one at the use site.
        if matches!(method, "sort" | "max" | "min" | "binarySearch")
            && let ElemType::Object(id) = elem
            && let Some(comparable) = self.table.class_id("Comparable")
            && !self.table.is_subtype(id, comparable)
        {
            self.no_suitable_library_method("Collections", method, args, span);
            return None;
        }

        self.expr(&args[0]);
        let mut width: u16 = 1;
        let (descriptor, ret) = match method {
            "reverse" | "sort" => (String::from("(Ljava/util/ArrayList;)V"), None),
            "swap" => {
                for index in &args[1..] {
                    let ty = self.expr(index);
                    self.numeric_conversion(ty, JType::Int);
                    width += 1;
                }
                (String::from("(Ljava/util/ArrayList;II)V"), None)
            }
            "shuffle" => {
                if let Some(random) = args.get(1) {
                    let ty = self.expr(random);
                    // The two-argument form takes a `java.util.Random`, and
                    // nothing else: a seeded one replays the JDK's permutation.
                    let is_random =
                        matches!(ty, JType::Object(id) if self.table.class_name(id) == "Random");
                    if !is_random && ty != JType::Error {
                        self.error(
                            random.span(),
                            format!(
                                "incompatible types: {} cannot be converted to Random",
                                ty.describe(self.table)
                            ),
                        );
                    }
                    width += 1;
                    (String::from("(Ljava/util/ArrayList;LRandom;)V"), None)
                } else {
                    (String::from("(Ljava/util/ArrayList;)V"), None)
                }
            }
            // The VM answers these: a list stores unboxed primitives, so a
            // bundled Java version could not compare them, and `max`/`min`
            // must hand back the element's own type.
            "max" | "min" => (
                format!("(Ljava/util/ArrayList;){element_descriptor}"),
                Some(element_ty),
            ),
            "frequency" | "binarySearch" => {
                let actual = self.expr(&args[1]);
                self.convert_for_assignment(actual, element_ty, args[1].span());
                width += element_ty.width();
                (
                    format!("(Ljava/util/ArrayList;{element_descriptor})I"),
                    Some(JType::Int),
                )
            }
            "unmodifiableList" => (
                String::from("(Ljava/util/ArrayList;)Ljava/util/ArrayList;"),
                Some(list_ty),
            ),
            // `addAll(list, e1, e2, ...)`: pack the varargs into an array of
            // the list's element type, as javac packs them into a `T[]`. A
            // lone array argument already *is* that array.
            "addAll" => {
                let elements = JType::Array { elem, dims: 1 };
                // A lone `T[]` already *is* the varargs array — but only for a
                // reference `T`. javac has no `Integer[]`/`int[]` conflation,
                // and rejects `addAll(List<Integer>, int[])`.
                if let [single] = &args[1..]
                    && elem.base_type().is_reference()
                    && self.type_of(single) == elements
                {
                    self.expr(single);
                } else {
                    self.emit_array_literal(&args[1..], elements, span);
                }
                width += 1;
                (
                    format!(
                        "(Ljava/util/ArrayList;{})Z",
                        elements.descriptor(self.table)
                    ),
                    Some(JType::Boolean),
                )
            }
            _ => unreachable!("arity checked above"),
        };
        let method_ref = intern_method_ref(self.pool, "Collections", method, &descriptor);
        let ret_width = ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKESTATIC, method_ref, ret_width);
        self.code.drop_stack(width);
        Some(ret)
    }

    /// `Arrays.asList(T...)`: pack the arguments into an `Object[]`, call the
    /// bundled `Arrays.asList(Object[])`, and type the result `List<elem>`. A
    /// single array argument is passed through as the varargs array.
    #[allow(clippy::option_option, clippy::unnecessary_wraps)] // call-dispatch return shape
    fn emit_arrays_as_list(&mut self, args: &[Expr], span: SourceSpan) -> Option<Option<JType>> {
        let object_elem = ElemType::Object(self.table.object_id);
        // Pack into an array of the ELEMENT type, not `Object[]`: a list holds
        // its primitives unboxed, so `Arrays.asList(1, 2)` must not box them.
        let elem = if let [single] = args {
            if let JType::Array { elem, dims: 1 } = self.type_of(single) {
                // The lone array argument *is* the varargs array.
                self.expr(single);
                elem
            } else {
                let scalar = elem_type_of(self.type_of(single)).unwrap_or(object_elem);
                self.emit_array_literal(
                    args,
                    JType::Array {
                        elem: scalar,
                        dims: 1,
                    },
                    span,
                );
                scalar
            }
        } else {
            let scalar = args
                .first()
                .and_then(|a| elem_type_of(self.type_of(a)))
                .unwrap_or(object_elem);
            self.emit_array_literal(
                args,
                JType::Array {
                    elem: scalar,
                    dims: 1,
                },
                span,
            );
            scalar
        };
        // The VM builds the list, so the array's element kind carries through
        // rather than erasing to `Object`.
        let descriptor = format!(
            "({})Ljava/util/ArrayList;",
            JType::Array { elem, dims: 1 }.descriptor(self.table)
        );
        let method_ref = intern_method_ref(self.pool, "Arrays", "asList", &descriptor);
        self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
        self.code.drop_stack(1); // the array argument
        Some(Some(JType::List(elem)))
    }

    /// `Arrays.deepToString(a)` / `deepEquals(a, b)` / `deepHashCode(a)`.
    /// Each takes a reference array — `int[][]` is one, `int[]` is not, which
    /// is exactly what javac accepts for an `Object[]` parameter.
    #[allow(clippy::option_option)] // call-dispatch return shape
    fn emit_arrays_deep_call(
        &mut self,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let (arity, descriptor, ret) = arrays_deep_signature(method).expect("caller checked");
        if args.len() != arity {
            let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
            let described = describe_types(&arg_types, self.table);
            self.error(
                span,
                format!("no suitable method found for {method}({described}) in class Arrays"),
            );
            return None;
        }
        for arg in args {
            let ty = self.expr(arg);
            if ty != JType::Error && !is_reference_array(ty) {
                self.error(
                    arg.span(),
                    format!(
                        "incompatible types: {} cannot be converted to Object[]",
                        ty.describe(self.table)
                    ),
                );
            }
        }
        let method_ref = intern_method_ref(self.pool, "Arrays", method, descriptor);
        self.code
            .push_op_u16(op::INVOKESTATIC, method_ref, ret.width());
        self.code
            .drop_stack(u16::try_from(arity).unwrap_or(u16::MAX));
        Some(Some(ret))
    }

    /// javac's wording when no overload of a bundled `java.util` helper takes
    /// these arguments.
    fn no_suitable_library_method(
        &mut self,
        class: &str,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) {
        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            return;
        }
        let described = describe_types(&arg_types, self.table);
        self.error(
            span,
            format!("no suitable method found for {method}({described}) in class {class}"),
        );
    }

    /// javac's wording when no `Arrays` overload matches the arguments.
    fn no_suitable_arrays_method(&mut self, method: &str, args: &[Expr], span: SourceSpan) {
        self.no_suitable_library_method("Arrays", method, args, span);
    }

    /// `Arrays.setAll(array, generator)` — the generator (already the erased
    /// `__UnaryOperator`) is applied to each index and stored. Returns void; the
    /// VM keys on the array's own kind for how to store each result.
    #[allow(clippy::option_option)] // call-dispatch return shape
    fn emit_arrays_set_all(&mut self, args: &[Expr], span: SourceSpan) -> Option<Option<JType>> {
        let [array_arg, generator_arg] = args else {
            self.no_suitable_library_method("Arrays", "setAll", args, span);
            return None;
        };
        let array_ty = self.expr(array_arg);
        if !matches!(array_ty, JType::Array { .. }) {
            self.error(array_arg.span(), "Arrays.setAll takes an array");
            self.code.discard();
            return None;
        }
        self.expr(generator_arg);
        let descriptor = format!(
            "({}Ljava/util/function/IntUnaryOperator;)V",
            array_ty.descriptor(self.table)
        );
        let method_ref = intern_method_ref(self.pool, "Arrays", "setAll", &descriptor);
        self.code.push_op_u16(op::INVOKESTATIC, method_ref, 0);
        self.code.drop_stack(2);
        Some(None)
    }

    /// `Arrays.copyOf(a, n)`, `copyOfRange(a, from, to)`, `fill(a[, from, to], v)`
    /// and `binarySearch(a[, from, to], key)`. Arity alone tells the ranged
    /// overloads apart, as it does in Java.
    #[allow(clippy::option_option)] // call-dispatch return shape
    fn emit_arrays_array_call(
        &mut self,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        let ranged = args.len() == 4;
        let arity_ok = match method {
            "copyOf" => args.len() == 2,
            "copyOfRange" => args.len() == 3,
            "fill" | "binarySearch" => args.len() == 2 || ranged,
            _ => false,
        };
        let source_ty = args.first().map_or(JType::Error, |a| self.type_of(a));
        let JType::Array { elem, dims } = source_ty else {
            self.no_suitable_arrays_method(method, args, span);
            return None;
        };
        if !arity_ok {
            self.no_suitable_arrays_method(method, args, span);
            return None;
        }
        // A row of a multi-dimensional array is itself an array.
        let element_ty = if dims > 1 {
            JType::Array {
                elem,
                dims: dims - 1,
            }
        } else {
            elem.base_type()
        };
        // Java has no `binarySearch(boolean[], boolean)`: booleans have no order.
        if method == "binarySearch" && element_ty == JType::Boolean {
            self.no_suitable_arrays_method(method, args, span);
            return None;
        }

        let source_descriptor = source_ty.descriptor(self.table);
        let element_descriptor = element_ty.descriptor(self.table);
        self.expr(&args[0]);
        let mut args_width = 1u16;

        let emit_index = |out: &mut Self, index: &Expr| {
            let actual = out.type_of(index);
            if !widens(actual, JType::Int, out.table) && actual != JType::Error {
                out.error(
                    index.span(),
                    format!(
                        "incompatible types: {} cannot be converted to int",
                        actual.describe(out.table)
                    ),
                );
            }
            let actual = out.expr(index);
            out.numeric_conversion(actual, JType::Int);
        };
        // `convert_for_assignment` reports an incompatible value itself, in
        // javac's wording.
        let emit_value = |out: &mut Self, value: &Expr| {
            let actual = out.expr(value);
            out.convert_for_assignment(actual, element_ty, value.span());
        };

        let (descriptor, ret) = match method {
            "copyOf" => {
                emit_index(self, &args[1]);
                args_width += 1;
                (
                    format!("({source_descriptor}I){source_descriptor}"),
                    Some(source_ty),
                )
            }
            "copyOfRange" => {
                emit_index(self, &args[1]);
                emit_index(self, &args[2]);
                args_width += 2;
                (
                    format!("({source_descriptor}II){source_descriptor}"),
                    Some(source_ty),
                )
            }
            "fill" | "binarySearch" => {
                let indices = if ranged {
                    emit_index(self, &args[1]);
                    emit_index(self, &args[2]);
                    args_width += 2;
                    "II"
                } else {
                    ""
                };
                emit_value(self, args.last().expect("arity checked"));
                args_width += element_ty.width();
                let ret = if method == "fill" {
                    None
                } else {
                    Some(JType::Int)
                };
                let returns = if method == "fill" { "V" } else { "I" };
                (
                    format!("({source_descriptor}{indices}{element_descriptor}){returns}"),
                    ret,
                )
            }
            _ => unreachable!("caller checked the method name"),
        };

        let method_ref = intern_method_ref(self.pool, "Arrays", method, &descriptor);
        let ret_width = ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKESTATIC, method_ref, ret_width);
        self.code.drop_stack(args_width);
        Some(ret)
    }

    /// `System.out.println(...)` and friends.
    fn print_call(&mut self, stream: &'static str, method: &str, args: &[Expr], span: SourceSpan) {
        if method == "printf" {
            let field = intern_field_ref(
                self.pool,
                "java/lang/System",
                stream,
                "Ljava/io/PrintStream;",
            );
            self.code.push_op_u16(op::GETSTATIC, field, 1);
            let Some((tags, width)) = self.emit_format_varargs(args, span) else {
                self.code.discard();
                return;
            };
            let descriptor = format!("(Ljava/lang/String;{tags})V");
            let method_ref =
                intern_method_ref(self.pool, "java/io/PrintStream", "printf", &descriptor);
            self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 0);
            self.code.drop_stack(1 + width);
            return;
        }
        if method != "println" && method != "print" {
            // javac: "cannot find symbol — symbol: method prinn(String),
            // location: variable out of type PrintStream", flattened to
            // our single-line diagnostic form.
            let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
            self.error(
                span,
                format!(
                    "cannot find symbol: method {method}({}) in class PrintStream",
                    describe_types(&arg_types, self.table)
                ),
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
                let arg_ty = self.coerce_to_string_for_output(arg_ty);
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
            // Reached only if not already coerced; treat as Object.
            // `println(char[])` is a real overload and prints the CHARACTERS.
            // It must precede the `Object` arm below, which every other array
            // takes to reach `Object.toString()` (`[I@1b6d3586`).
            JType::Array {
                elem: ElemType::Char,
                dims: 1,
            } => Some(String::from("([C)V")),
            JType::Generic { .. }
            | JType::StringBuilder
            | JType::TypeVar
            | JType::Boxed(_)
            | JType::Map { .. }
            | JType::TreeMap { .. }
            | JType::Set(_)
            | JType::TreeSet(_)
            | JType::Stream(_)
            | JType::Collector
            | JType::IntStream
            | JType::Optional(_)
            | JType::OptionalInt
            | JType::OptionalDouble
            | JType::LinkedList { .. }
            | JType::Collection(_)
            | JType::EntrySet { .. }
            | JType::MapEntry { .. }
            | JType::Class
            | JType::Field
            | JType::Method
            | JType::Type
            | JType::Constructor
            | JType::Array { .. } => Some(String::from("(Ljava/lang/Object;)V")),
            JType::Int | JType::Short | JType::Byte => Some(String::from("(I)V")),
            JType::Double => Some(String::from("(D)V")),
            JType::Long => Some(String::from("(J)V")),
            JType::Float => Some(String::from("(F)V")),
            JType::Boolean => Some(String::from("(Z)V")),
            JType::Char => Some(String::from("(C)V")),
            // Objects and lists are coerced to String before this is
            // consulted; File is coerced to its path string upstream.
            JType::Str
            | JType::Object(_)
            | JType::List(_)
            | JType::Stack(_)
            | JType::Exception(_)
            | JType::File => Some(String::from("(Ljava/lang/String;)V")),
            JType::Scanner | JType::Writer => {
                self.error(
                    span,
                    format!("printing a {} is not supported", ty.describe(self.table)),
                );
                None
            }
            JType::Null => {
                self.error(
                    span,
                    "println(null) is ambiguous in Java; pass a String or \"null\"",
                );
                None
            }
            JType::Unsupported => {
                self.error(span, "this value's type is not yet supported by caturra");
                None
            }
            JType::Error => None,
        }
    }

    // ----- expressions -----

    /// The static type of an expression, without emitting code. Must
    /// agree with what [`Self::expr`] leaves on the stack. One arm per
    /// expression kind keeps it in lockstep with `expr`.
    #[allow(clippy::too_many_lines)]
    fn type_of(&mut self, expr: &Expr) -> JType {
        match expr {
            Expr::Literal { value, .. } => match value {
                Literal::Int(_) => JType::Int,
                Literal::Long(_) => JType::Long,
                Literal::Float(_) => JType::Float,
                Literal::Double(_) => JType::Double,
                Literal::Str(_) => JType::Str,
                Literal::Char(_) => JType::Char,
                Literal::Bool(_) => JType::Boolean,
                Literal::Null => JType::Null,
            },
            Expr::Name { path, span } if self.strip_package_prefix(path).is_some() => {
                let short = self.strip_package_prefix(path).expect("checked in guard");
                self.type_of(&Expr::Name {
                    path: short,
                    span: *span,
                })
            }
            Expr::Name { path, span }
                if path.len() >= 2
                    && (self.lookup(&path[0]).is_some()
                        || self.table.field(self.current_class, &path[0]).is_some()) =>
            {
                let (prefix, last) = path.split_at(path.len() - 1);
                let object = Expr::Name {
                    path: prefix.to_vec(),
                    span: *span,
                };
                self.type_of(&Expr::Field {
                    object: Box::new(object),
                    name: last[0].clone(),
                    span: *span,
                })
            }
            Expr::Name { path, .. } if path.len() == 1 => {
                if let Some(var) = self.lookup(&path[0]) {
                    return var.ty;
                }
                // Implicit this-field or static field of the current class,
                // then a static field of the enclosing class of a synthesized
                // lambda. `name()` emits all three, and `type_of` must agree —
                // an expression typed `Error` is silently dropped.
                self.table
                    .field(self.current_class, &path[0])
                    .map(|(_, f)| f.ty)
                    .or_else(|| self.enclosing_static_field(&path[0]).map(|(_, f)| f.ty))
                    .or_else(|| {
                        self.enclosing_instance_field(&path[0])
                            .map(|(_, _, f)| f.ty)
                    })
                    .unwrap_or(JType::Error)
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
            Expr::Name { path, .. }
                if path.len() == 2
                    && !self.table.has_class(&path[0])
                    && builtin_static_constant(&path[0], &path[1]).is_some() =>
            {
                match builtin_static_constant(&path[0], &path[1]) {
                    Some(BuiltinConstant::Double(_)) => JType::Double,
                    Some(BuiltinConstant::Char(_)) => JType::Char,
                    Some(BuiltinConstant::Bool(_)) => JType::Boolean,
                    Some(BuiltinConstant::Long(_)) => JType::Long,
                    Some(BuiltinConstant::Float(_)) => JType::Float,
                    _ => JType::Int,
                }
            }
            Expr::Name { path, .. } if path.len() == 2 => {
                // `x.field` on a local object, or `Class.staticField`.
                if let Some(var) = self.lookup(&path[0]) {
                    if let JType::Object(id) = var.ty {
                        let owner = self.table.class_name(id).to_owned();
                        return self
                            .table
                            .field(&owner, &path[1])
                            .map_or(JType::Error, |(_, f)| f.ty);
                    }
                    return JType::Error;
                }
                self.table
                    .field(&path[0], &path[1])
                    .map_or(JType::Error, |(_, f)| f.ty)
            }
            Expr::Name { path, span } if path.len() > 2 => {
                let (prefix, last) = path.split_at(path.len() - 1);
                let object = Expr::Name {
                    path: prefix.to_vec(),
                    span: *span,
                };
                self.type_of(&Expr::Field {
                    object: Box::new(object),
                    name: last[0].clone(),
                    span: *span,
                })
            }
            Expr::Name { .. }
            | Expr::ArrayLiteral { .. }
            | Expr::Lambda { .. }
            | Expr::MethodRef { .. } => JType::Error,
            Expr::Index { array, .. } => self.type_of(array).element_type().unwrap_or(JType::Error),
            Expr::Field { object, name, .. }
                if name == "class" && matches!(object.as_ref(), Expr::Name { .. }) =>
            {
                JType::Class
            }
            Expr::Field { object, name, .. } => match self.type_of(object) {
                JType::Array { .. } if name == "length" => JType::Int,
                JType::Object(id) => {
                    let owner = self.table.class_name(id).to_owned();
                    self.table
                        .field(&owner, name)
                        .map_or(JType::Error, |(_, f)| f.ty)
                }
                _ => JType::Error,
            },
            Expr::NewArray { elem, dims, .. } => {
                match (
                    self.table.resolve_type(elem).and_then(elem_type_of),
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
                // `Comparator` combinators build another comparator (mirrors the
                // emission-path intercept in `instance_call`).
                if matches!(method.as_str(), "reversed" | "thenComparing")
                    && let Some(recv) = receiver.as_deref()
                    && let recv_ty = self.type_of(recv)
                    && self.is_comparator_type(recv_ty)
                {
                    return self
                        .table
                        .class_id("__Comparator")
                        .map_or(JType::Error, JType::Object);
                }
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
                    Some(Expr::Name { path, .. })
                        if {
                            let short = self.strip_package_prefix(path);
                            let effective = short.as_deref().unwrap_or(path);
                            effective.len() == 1
                                && self.lookup(&effective[0]).is_none()
                                && builtin_static_table(&effective[0]).is_some()
                        } =>
                    {
                        let short = self.strip_package_prefix(path);
                        let path = short.as_deref().unwrap_or(path);
                        // `String.format` is variadic and special-cased in the
                        // emission path (not in the static table); it returns
                        // String. Mirror that here so it can be an argument.
                        if path[0] == "String" && method == "format" {
                            return JType::Str;
                        }
                        // Intrinsic static (Math.abs, ...).
                        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
                        let (_, methods) = builtin_static_table(&path[0]).expect("checked above");
                        return pick_builtin(
                            methods,
                            method,
                            &arg_types,
                            TypeArgs::default(),
                            self.table,
                        )
                        .and_then(|m| bret_type(m.ret, TypeArgs::default(), self.table))
                        .unwrap_or(JType::Error);
                    }
                    Some(other) => match self.type_of(other) {
                        JType::Object(id) => self.table.class_name(id).to_owned(),
                        // A wrapper method on a primitive/boxed receiver
                        // (`someInt.intValue()`) — autoboxed at emission.
                        JType::Int
                        | JType::Double
                        | JType::Long
                        | JType::Float
                        | JType::Short
                        | JType::Byte
                        | JType::Char
                        | JType::Boolean
                        | JType::Boxed(_) => {
                            return boxed_method_return(method).unwrap_or(JType::Error);
                        }
                        receiver_ty @ (JType::Str
                        | JType::StringBuilder
                        | JType::Scanner
                        | JType::File
                        | JType::Writer
                        | JType::List(_)
                        | JType::Stack(_)
                        | JType::LinkedList { .. }
                        | JType::Map { .. }
                        | JType::TreeMap { .. }
                        | JType::Set(_)
                        | JType::TreeSet(_)
                        | JType::Stream(_)
                        | JType::Collector
                        | JType::IntStream
                        | JType::Optional(_)
                        | JType::OptionalInt
                        | JType::OptionalDouble
                        | JType::Collection(_)
                        | JType::EntrySet { .. }
                        | JType::MapEntry { .. }
                        | JType::Exception(_)
                        | JType::Class
                        | JType::Field
                        | JType::Method
                        | JType::Type
                        | JType::Constructor) => {
                            let elem = TypeArgs::of(receiver_ty);
                            let arg_types: Vec<JType> =
                                args.iter().map(|a| self.type_of(a)).collect();
                            let (_, methods) = builtin_instance_table(receiver_ty)
                                .expect("matched builtin receivers");
                            return pick_builtin(methods, method, &arg_types, elem, self.table)
                                .and_then(|m| bret_type(m.ret, elem, self.table))
                                .unwrap_or(JType::Error);
                        }
                        // `Object` methods on an array receiver. Mirror
                        // `array_object_call`, or `type_of` and the emitter
                        // disagree and the bytecode fails to verify.
                        JType::Array { .. } => {
                            return match (method.as_str(), args.len()) {
                                ("equals", 1) => JType::Boolean,
                                ("hashCode", 0) => JType::Int,
                                ("toString", 0) => JType::Str,
                                ("getClass", 0) => JType::Class,
                                _ => JType::Error,
                            };
                        }
                        _ => return JType::Error,
                    },
                };
                let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
                // `Collections.max/min/frequency/nCopies` are in no method
                // table either: the VM answers them.
                if class == "Collections" {
                    match method.as_str() {
                        "max" | "min" => {
                            let list = args.first().map_or(JType::Error, |a| self.type_of(a));
                            return match list {
                                JType::List(elem) => elem.base_type(),
                                _ => JType::Error,
                            };
                        }
                        "frequency" | "binarySearch" => return JType::Int,
                        "addAll" => return JType::Boolean,
                        "unmodifiableList" => {
                            return args.first().map_or(JType::Error, |a| self.type_of(a));
                        }
                        // Empty immutable collections adopt their context.
                        "emptyList" | "emptySet" | "emptyMap" => return JType::Null,
                        "nCopies" => {
                            let value = args.get(1).map_or(JType::Error, |a| self.type_of(a));
                            return elem_type_of(value).map_or(JType::Error, JType::List);
                        }
                        "singletonList" => {
                            let value = args.first().map_or(JType::Error, |a| self.type_of(a));
                            return elem_type_of(value).map_or(JType::Error, JType::List);
                        }
                        "reverseOrder" => {
                            return self
                                .table
                                .class_id("__Comparator")
                                .map_or(JType::Error, JType::Object);
                        }
                        "singleton" => {
                            let value = args.first().map_or(JType::Error, |a| self.type_of(a));
                            return elem_type_of(value).map_or(JType::Error, JType::Set);
                        }
                        "singletonMap" => {
                            let key = args.first().map_or(JType::Error, |a| self.type_of(a));
                            let value = args.get(1).map_or(JType::Error, |a| self.type_of(a));
                            return match (elem_type_of(key), elem_type_of(value)) {
                                (Some(key), Some(value)) => JType::Map { key, value },
                                _ => JType::Error,
                            };
                        }
                        "unmodifiableSet" | "unmodifiableMap" => {
                            return args.first().map_or(JType::Error, |a| self.type_of(a));
                        }
                        _ => {}
                    }
                }
                if class == "Optional" {
                    match method.as_str() {
                        "of" | "ofNullable" => {
                            let arg = args.first().map_or(JType::Error, |a| self.type_of(a));
                            return elem_type_of(arg).map_or(JType::Error, JType::Optional);
                        }
                        // An empty Optional adopts its context, typing like `null`.
                        "empty" => return JType::Null,
                        _ => {}
                    }
                }
                // The VM answers these, so they are in no method table.
                if class == "Arrays" {
                    if let Some((_, _, ret)) = arrays_deep_signature(method) {
                        return ret;
                    }
                    match method.as_str() {
                        // `copyOf`/`copyOfRange` return the source's own type.
                        "copyOf" | "copyOfRange" => {
                            return args.first().map_or(JType::Error, |a| self.type_of(a));
                        }
                        "binarySearch" => return JType::Int,
                        // `fill` is void, which `type_of` spells `Error`.
                        "fill" => return JType::Error,
                        _ => {}
                    }
                }
                let table = self.table;
                match table.resolve(&class, method, &arg_types) {
                    Resolution::Found(sig) => sig.ret.unwrap_or(JType::Error),
                    _ => {
                        // Inherited Throwable members (mirrors emission).
                        if (method == "getMessage" || method == "toString")
                            && args.is_empty()
                            && table
                                .class_id(&class)
                                .is_some_and(|id| table.is_throwable(id))
                        {
                            JType::Str
                        } else {
                            JType::Error
                        }
                    }
                }
            }
            Expr::Unary {
                op: UnaryOp::Not, ..
            }
            | Expr::InstanceOf { .. } => JType::Boolean,
            Expr::Unary {
                op: UnaryOp::BitNot,
                operand,
                ..
            } => match self.type_of(operand) {
                JType::Long => JType::Long,
                _ => JType::Int,
            },
            Expr::Unary {
                op: UnaryOp::Neg,
                operand,
                ..
            } => match self.type_of(operand) {
                JType::Double => JType::Double,
                JType::Long => JType::Long,
                JType::Float => JType::Float,
                t if t.is_numeric() => JType::Int,
                _ => JType::Error,
            },
            Expr::Cast { ty, .. } => self.table.resolve_type(ty).unwrap_or(JType::Error),
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
                // These must agree with what `bitwise` / the shift arm actually
                // emit: `long & long` yields a long, and a shift takes the type
                // of its LEFT operand alone (JLS §15.19) — the count doesn't
                // widen it. Reporting int here made `(x << 4) + y` emit IADD
                // over a long and fail verification.
                BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                    let (lt, rt) = (
                        numeric_view(self.type_of(lhs)),
                        numeric_view(self.type_of(rhs)),
                    );
                    if lt == JType::Boolean && rt == JType::Boolean {
                        JType::Boolean
                    } else {
                        promote(lt, rt)
                    }
                }
                BinaryOp::Shl | BinaryOp::Shr | BinaryOp::Ushr => {
                    if numeric_view(self.type_of(lhs)) == JType::Long {
                        JType::Long
                    } else {
                        JType::Int
                    }
                }
                _ => JType::Boolean,
            },
            Expr::Super { .. } => {
                if self.in_static {
                    JType::Error
                } else {
                    JType::Object(self.superclass_id())
                }
            }
            Expr::This { .. } => {
                if self.in_static {
                    JType::Error
                } else if let Some((_, enclosing)) = (!self.in_constructor)
                    .then(|| self.captured_outer())
                    .flatten()
                {
                    JType::Object(enclosing)
                } else {
                    JType::Object(self.current_class_id)
                }
            }
            Expr::NewObject {
                class, type_args, ..
            } => self.type_of_new_object(class, type_args),
            Expr::Ternary { then, els, .. } => {
                let then_ty = self.type_of(then);
                let els_ty = self.type_of(els);
                if then_ty == els_ty {
                    then_ty
                } else if then_ty.is_numeric() && els_ty.is_numeric() {
                    promote(then_ty, els_ty)
                } else if then_ty == JType::Null {
                    els_ty
                } else if els_ty == JType::Null {
                    then_ty
                } else if widens(then_ty, els_ty, self.table) {
                    els_ty
                } else if widens(els_ty, then_ty, self.table) {
                    then_ty
                } else {
                    JType::Error
                }
            }
            Expr::IncDec { target, .. } => match self.type_of(target) {
                ty if ty.is_numeric() => ty,
                _ => JType::Error,
            },
            Expr::SuperMethodCall { method, args, .. } => {
                let current = self.table.class_name(self.current_class_id).to_owned();
                let Some(superclass) = self.table.classes.get(&current).and_then(|c| c.superclass)
                else {
                    return JType::Error;
                };
                let super_name = self.table.class_name(superclass).to_owned();
                let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
                let table = self.table;
                match table.resolve(&super_name, method, &arg_types) {
                    Resolution::Found(sig) => sig.ret.unwrap_or(JType::Error),
                    _ => JType::Error,
                }
            }
        }
    }

    /// Emit code leaving the expression's value on the stack; returns
    /// its type ([`JType::Error`] if a diagnostic was reported).
    #[allow(clippy::too_many_lines)] // one dispatch arm per expression kind
    fn expr(&mut self, expr: &Expr) -> JType {
        match expr {
            Expr::Literal { value, span } => self.literal(value, *span),
            Expr::Name { path, span } => self.name(path, *span),
            Expr::Lambda { span, .. } | Expr::MethodRef { span, .. } => {
                self.error(
                    *span,
                    "a lambda or method reference is only allowed where a \
                     functional-interface type is expected",
                );
                JType::Error
            }
            Expr::Call {
                receiver,
                method,
                args,
                span,
            } => {
                let outcome = match self.call_target(receiver.as_deref(), *span) {
                    None => None,
                    Some(CallTarget::Stream(_)) => {
                        self.error(*span, "print/println do not return a value");
                        None
                    }
                    Some(CallTarget::Static(class)) => {
                        self.static_call(&class, method, args, *span)
                    }
                    Some(CallTarget::Own) => self.own_call(method, args, *span),
                    Some(CallTarget::Instance(object)) => {
                        self.instance_call(object, method, args, *span)
                    }
                };
                match outcome {
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
            Expr::This { span } => {
                if self.in_static {
                    self.error(
                        *span,
                        "non-static variable this cannot be referenced from a static context",
                    );
                    return JType::Error;
                }
                // Inside a lambda, `this` is the ENCLOSING instance (JLS
                // §15.27.2), captured as `__caturraOuter`. Not in the
                // synthesized constructor, whose `this` is the lambda itself.
                if !self.in_constructor
                    && let Some((outer, enclosing)) = self.captured_outer()
                {
                    self.push_captured_outer(&outer);
                    return JType::Object(enclosing);
                }
                self.code.push_op(op::ALOAD_0, 1);
                JType::Object(self.current_class_id)
            }
            // `super` as a field receiver: the object is `this`, but the
            // lookup starts at the superclass. Fields are not virtual, so
            // this is all `super.n` means. Never reached for
            // `super.method(...)` — that is `Expr::SuperMethodCall`.
            Expr::Super { span } => {
                if self.in_static {
                    self.error(
                        *span,
                        "non-static variable super cannot be referenced from a static context",
                    );
                    return JType::Error;
                }
                self.code.push_op(op::ALOAD_0, 1);
                JType::Object(self.superclass_id())
            }
            Expr::NewObject {
                class,
                type_args,
                args,
                span,
            } => self.new_object(class, type_args, args, *span),
            Expr::Ternary {
                cond,
                then,
                els,
                span,
            } => self.ternary(cond, then, els, *span),
            Expr::IncDec {
                target,
                increment,
                prefix,
                span,
            } => self.inc_dec(target, *increment, *prefix, *span),
            Expr::InstanceOf { value, ty, span } => self.instance_of(value, ty, *span),
            Expr::SuperMethodCall { method, args, span } => {
                match self.super_method_call(method, args, *span) {
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
        }
    }

    /// `value instanceof Type` (reference types only).
    fn instance_of(&mut self, value: &Expr, ty: &TypeRef, span: SourceSpan) -> JType {
        let value_ty = self.expr(value);
        let Some(target) = self.table.resolve_type(ty) else {
            self.error(span, "unknown type in instanceof");
            return JType::Error;
        };
        let class_name = match target {
            JType::Object(id) => self.table.class_name(id).to_owned(),
            // `x instanceof Integer` / `String` — wrapper and String tests.
            JType::Boxed(elem) => wrapper_internal(elem).to_owned(),
            JType::Str => String::from("java/lang/String"),
            // `x instanceof ArrayList<…>` — a runtime list check.
            JType::List(_) => String::from("java/util/ArrayList"),
            JType::Stack(_) => String::from("java/util/Stack"),
            JType::Map { .. } => String::from("java/util/HashMap"),
            // `type instanceof ParameterizedType` — a runtime check on the
            // reflect Type's kind (the VM inspects the value).
            JType::Type => match ty {
                TypeRef::Named(name) => format!("java/lang/reflect/{name}"),
                _ => String::from("java/lang/reflect/Type"),
            },
            other => {
                self.error(
                    span,
                    format!(
                        "instanceof needs a class type, got {}",
                        other.describe(self.table)
                    ),
                );
                return JType::Error;
            }
        };
        if !value_ty.is_reference() && value_ty != JType::Error {
            self.error(
                span,
                format!(
                    "unexpected type: {} cannot be tested with instanceof",
                    value_ty.describe(self.table)
                ),
            );
            return JType::Error;
        }
        let class_index = intern_class(self.pool, &class_name);
        self.code.push_op_u16(op::INSTANCEOF, class_index, 1);
        self.code.drop_stack(1);
        JType::Boolean
    }

    /// `super.method(args)` — non-virtual dispatch to the superclass.
    #[allow(clippy::option_option)]
    fn super_method_call(
        &mut self,
        method: &str,
        args: &[Expr],
        span: SourceSpan,
    ) -> Option<Option<JType>> {
        if self.in_static {
            self.error(
                span,
                "non-static variable super cannot be referenced from a static context",
            );
            return None;
        }
        let current = self.table.class_name(self.current_class_id).to_owned();
        let Some(superclass) = self.table.classes.get(&current).and_then(|c| c.superclass) else {
            self.error(span, format!("{current} has no superclass"));
            return None;
        };
        let super_name = self.table.class_name(superclass).to_owned();

        let arg_types: Vec<JType> = args.iter().map(|a| self.type_of(a)).collect();
        if arg_types.contains(&JType::Error) {
            for arg in args {
                self.expr(arg);
            }
            return None;
        }
        let table = self.table;
        let sig = if let Resolution::Found(sig) = table.resolve(&super_name, method, &arg_types) {
            sig.clone()
        } else {
            self.error(
                span,
                format!(
                    "cannot find symbol: method {method}({}) in class {super_name}",
                    describe_types(&arg_types, self.table)
                ),
            );
            return None;
        };
        if sig.is_static || sig.is_abstract {
            self.error(
                span,
                format!("cannot call {method}() via super (it has no body there)"),
            );
            return None;
        }

        self.code.push_op(op::ALOAD_0, 1);
        let args_width = self.emit_call_args(args, &sig, span);
        let descriptor = sig.descriptor(self.table);
        let method_ref = intern_method_ref(self.pool, &super_name, method, &descriptor);
        let ret_width = sig.ret.map_or(0, JType::width);
        self.code
            .push_op_u16(op::INVOKESPECIAL, method_ref, ret_width);
        self.code.drop_stack(1 + args_width);
        Some(sig.ret)
    }

    /// Field access on a value: only `.length` on arrays exists so far.
    /// Emit a `Type.class` literal: push the type's canonical name and turn it
    /// into a `Class` handle via the `Class.__forType` intrinsic.
    fn class_literal(&mut self, type_name: &str) -> JType {
        // Array class literals (`int[].class`, `String[][].class`) carry a
        // trailing `[]` per dimension; their `Class` name is the JVM array
        // descriptor (`[I`, `[[Ljava/lang/String;`).
        let mut base = type_name;
        let mut dims = 0usize;
        while let Some(stripped) = base.strip_suffix("[]") {
            base = stripped;
            dims += 1;
        }
        let base_name = match base {
            "int" | "double" | "boolean" | "char" | "long" | "float" | "short" | "byte" => {
                base.to_owned()
            }
            "String" => String::from("java/lang/String"),
            "Object" => String::from("java/lang/Object"),
            "Integer" | "Double" | "Boolean" | "Character" | "Long" | "Float" | "Short"
            | "Byte" => {
                format!("java/lang/{base}")
            }
            other => self.table.class_id(other).map_or_else(
                || other.to_owned(),
                |id| self.table.class_name(id).to_owned(),
            ),
        };
        let canonical = if dims == 0 {
            base_name
        } else {
            let element = match base {
                "int" => String::from("I"),
                "long" => String::from("J"),
                "double" => String::from("D"),
                "float" => String::from("F"),
                "boolean" => String::from("Z"),
                "char" => String::from("C"),
                "short" => String::from("S"),
                "byte" => String::from("B"),
                _ => format!("L{base_name};"),
            };
            format!("{}{element}", "[".repeat(dims))
        };
        let utf8 = self.pool.intern_utf8(&canonical);
        let index = self.pool.intern(Constant::String { string_index: utf8 });
        self.code.push_ldc(index);
        let method_ref = intern_method_ref(
            self.pool,
            "java/lang/Class",
            "__forType",
            "(Ljava/lang/String;)Ljava/lang/Class;",
        );
        self.code.push_op_u16(op::INVOKESTATIC, method_ref, 1);
        self.code.drop_stack(1);
        JType::Class
    }

    fn field(&mut self, object: &Expr, name: &str, span: SourceSpan) -> JType {
        // `Type.class` class literal — the object is a type name, not a value,
        // so intercept before evaluating it.
        if name == "class"
            && let Expr::Name { path, .. } = object
        {
            return self.class_literal(path.last().map_or("", String::as_str));
        }
        let object_ty = self.expr(object);
        if object_ty == JType::Error {
            return JType::Error;
        }
        if name == "length" && matches!(object_ty, JType::Array { .. }) {
            self.code.push_op(op::ARRAYLENGTH, 1);
            self.code.drop_stack(1);
            return JType::Int;
        }
        if let JType::Object(class_id)
        | JType::Generic {
            class: class_id, ..
        } = object_ty
        {
            let Some((owner, field)) = self.resolve_field(class_id, name, span) else {
                return JType::Error;
            };
            if field.is_static {
                // `instance.staticField` (legal, if discouraged): the
                // instance is evaluated then discarded.
                self.code.push_op(op::POP, 0);
                self.code.drop_stack(1);
                return self.emit_getfield(owner, &field);
            }
            let field_ty = self.emit_getfield(owner, &field);
            // Substitute the tracked type argument for a type-variable
            // field (`cell.value` on a `Cell<String>` reads a String).
            if let JType::Generic { arg, .. } = object_ty {
                return self.substitute_type_var(field_ty, arg);
            }
            return field_ty;
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
            format!(
                "cannot find field '{name}' on {}",
                object_ty.describe(self.table)
            ),
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
        let Some(element) = self.table.resolve_type(elem).and_then(elem_type_of) else {
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
                        size_ty.describe(self.table)
                    ),
                );
            }
        }

        if sized.len() == 1 && dim_count == 1 {
            self.emit_new_1d(element);
        } else {
            // Multi-dimensional (or partially-sized) creation.
            let class_index = {
                let descriptor = array_ty.descriptor(self.table);
                intern_class(self.pool, &descriptor)
            };
            if sized.len() == 1 {
                // e.g. `new int[3][]` — one dimension allocated, rows null.
                // anewarray's element class is one dimension down.
                let element_descriptor = array_ty
                    .element_type()
                    .expect("array has an element type")
                    .descriptor(self.table);
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
            ElemType::Object(id) => {
                let class_name = self.table.class_name(id).to_owned();
                let class = intern_class(self.pool, &class_name);
                self.code.push_op_u16(op::ANEWARRAY, class, 1);
                self.code.drop_stack(1);
            }
            ElemType::Field => {
                let class = intern_class(self.pool, "java/lang/reflect/Field");
                self.code.push_op_u16(op::ANEWARRAY, class, 1);
                self.code.drop_stack(1);
            }
            ElemType::Method => {
                let class = intern_class(self.pool, "java/lang/reflect/Method");
                self.code.push_op_u16(op::ANEWARRAY, class, 1);
                self.code.drop_stack(1);
            }
            ElemType::Constructor => {
                let class = intern_class(self.pool, "java/lang/reflect/Constructor");
                self.code.push_op_u16(op::ANEWARRAY, class, 1);
                self.code.drop_stack(1);
            }
            ElemType::Class => {
                let class = intern_class(self.pool, "java/lang/Class");
                self.code.push_op_u16(op::ANEWARRAY, class, 1);
                self.code.drop_stack(1);
            }
            prim => {
                let atype = match prim {
                    ElemType::Int => op::T_INT,
                    ElemType::Double => op::T_DOUBLE,
                    ElemType::Long => op::T_LONG,
                    ElemType::Float => op::T_FLOAT,
                    ElemType::Short => op::T_SHORT,
                    ElemType::Byte => op::T_BYTE,
                    ElemType::Boolean => op::T_BOOLEAN,
                    ElemType::Char => op::T_CHAR,
                    ElemType::Str
                    | ElemType::Object(_)
                    | ElemType::Field
                    | ElemType::Method
                    | ElemType::Constructor
                    | ElemType::Class => unreachable!(),
                };
                self.code.push_op(op::NEWARRAY, 1);
                self.code.bytes.push(atype);
                self.code.drop_stack(1);
            }
        }
    }

    /// Emit an array from a `{...}` literal, leaving the reference on
    /// the stack. Nested literals build the inner dimensions.
    /// Emit a resolved method/constructor's arguments, packing the
    /// trailing arguments of a varargs call into a fresh array (unless
    /// the call already passes an assignable array). Returns the total
    /// stack width of the pushed arguments.
    fn emit_call_args(&mut self, args: &[Expr], sig: &MethodSig, span: SourceSpan) -> u16 {
        if !sig.is_varargs {
            for (arg, param) in args.iter().zip(&sig.params) {
                let actual = self.expr(arg);
                self.convert_for_assignment(actual, *param, arg.span());
            }
            return sig.params.iter().map(|p| p.width()).sum();
        }
        let fixed = sig.params.len() - 1;
        let array_ty = sig.params[fixed];
        for (arg, param) in args.iter().zip(&sig.params).take(fixed) {
            let actual = self.expr(arg);
            self.convert_for_assignment(actual, *param, arg.span());
        }
        // Array form: a single trailing argument assignable to the
        // varargs array is passed straight through.
        let array_form = args.len() == sig.params.len()
            && widens(self.type_of(&args[fixed]), array_ty, self.table);
        if array_form {
            let actual = self.expr(&args[fixed]);
            self.numeric_conversion(actual, array_ty);
        } else {
            let literal_span = args.get(fixed).map_or(span, Expr::span);
            self.emit_array_literal(&args[fixed..], array_ty, literal_span);
        }
        let fixed_width: u16 = sig.params[..fixed].iter().map(|p| p.width()).sum();
        fixed_width + 1
    }

    /// `Method.invoke(Object receiver, Object... args)`: the Method is already
    /// on the stack; push the receiver object, pack the remaining arguments
    /// into an `Object[]`, and call `invoke`.
    #[allow(clippy::option_option, clippy::unnecessary_wraps)] // call-dispatch return shape
    fn emit_method_invoke(&mut self, args: &[Expr], span: SourceSpan) -> Option<Option<JType>> {
        let object_ty = JType::Object(self.table.object_id);
        match args.first() {
            Some(obj) => {
                let obj_ty = self.expr(obj);
                self.convert_for_assignment(obj_ty, object_ty, obj.span());
            }
            None => self.code.push_op(op::ACONST_NULL, 1),
        }
        let rest = if args.len() > 1 { &args[1..] } else { &[] };
        let array_ty = JType::Array {
            elem: ElemType::Object(self.table.object_id),
            dims: 1,
        };
        self.emit_array_literal(rest, array_ty, span);
        let method_ref = intern_method_ref(
            self.pool,
            "java/lang/reflect/Method",
            "invoke",
            "(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
        );
        self.code.push_op_u16(op::INVOKEVIRTUAL, method_ref, 1);
        self.code.drop_stack(3); // Method receiver + object + array
        Some(Some(object_ty))
    }

    fn emit_array_literal(&mut self, elements: &[Expr], array_ty: JType, span: SourceSpan) {
        let Some(element) = array_ty.element_type() else {
            self.error(
                span,
                format!(
                    "array initializer cannot be assigned to {}",
                    array_ty.describe(self.table)
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
            let element_descriptor = element.descriptor(self.table);
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
                self.convert_for_assignment_const(
                    value_ty,
                    element,
                    value.span(),
                    constant_int_value(value),
                );
            }
            self.xastore(element);
        }
    }

    fn literal(&mut self, literal: &Literal, span: SourceSpan) -> JType {
        match literal {
            Literal::Long(value) => {
                match *value {
                    0 => self.code.push_op(op::LCONST_0, 2),
                    1 => self.code.push_op(op::LCONST_1, 2),
                    other => {
                        let index = self.pool.intern(Constant::Long(other));
                        self.code.push_op_u16(op::LDC2_W, index, 2);
                    }
                }
                JType::Long
            }
            Literal::Float(value) => {
                if value.to_bits() == 0.0f32.to_bits() {
                    self.code.push_op(op::FCONST_0, 1);
                } else if value.to_bits() == 1.0f32.to_bits() {
                    self.code.push_op(op::FCONST_1, 1);
                } else if value.to_bits() == 2.0f32.to_bits() {
                    self.code.push_op(op::FCONST_2, 1);
                } else {
                    let index = self.pool.intern(Constant::Float(*value));
                    self.code.push_ldc(index);
                }
                JType::Float
            }
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

    #[allow(clippy::too_many_lines)] // one resolution ladder, clearest linear
    fn name(&mut self, path: &[String], span: SourceSpan) -> JType {
        // Fully qualified statics: java.lang.Integer.MAX_VALUE, ...
        if let Some(short) = self.strip_package_prefix(path) {
            return self.name(&short, span);
        }
        // A dotted path whose head is a value — a local, or an implicit
        // field of `this` — is a field-access chain (`node.next.value`,
        // `top.value`, `arr.length`). Resolve all but the last segment
        // to an object, then read the final field.
        if path.len() >= 2
            && (self.lookup(&path[0]).is_some()
                || self.table.field(self.current_class, &path[0]).is_some())
        {
            let (prefix, last) = path.split_at(path.len() - 1);
            let object = Expr::Name {
                path: prefix.to_vec(),
                span,
            };
            return self.field(&object, &last[0], span);
        }
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
        // Intrinsic constants: Integer.MAX_VALUE / MIN_VALUE.
        if path.len() == 2
            && !self.table.has_class(&path[0])
            && self.lookup(&path[0]).is_none()
            && let Some(value) = builtin_static_constant(&path[0], &path[1])
        {
            match value {
                BuiltinConstant::Int(value) => {
                    self.push_int(value);
                    return JType::Int;
                }
                BuiltinConstant::Double(value) => {
                    let index = self.pool.intern(Constant::Double(value));
                    self.code.push_op_u16(op::LDC2_W, index, 2);
                    return JType::Double;
                }
                BuiltinConstant::Char(value) => {
                    self.push_int(i32::from(value));
                    return JType::Char;
                }
                BuiltinConstant::Bool(value) => {
                    self.push_int(i32::from(value));
                    return JType::Boolean;
                }
                BuiltinConstant::Long(value) => {
                    let index = self.pool.intern(Constant::Long(value));
                    self.code.push_op_u16(op::LDC2_W, index, 2);
                    return JType::Long;
                }
                BuiltinConstant::Float(value) => {
                    let index = self.pool.intern(Constant::Float(value));
                    self.code.push_ldc(index);
                    return JType::Float;
                }
            }
        }
        // `x.field` on a local object, or `Class.staticField`.
        if path.len() == 2 {
            if let Some(var) = self.lookup(&path[0]) {
                if let JType::Object(id) = var.ty {
                    let (slot, ty) = (var.slot, var.ty);
                    self.emit_load(slot, ty);
                    let Some((owner, field)) = self.resolve_field(id, &path[1], span) else {
                        return JType::Error;
                    };
                    if field.is_static {
                        // `local.staticField`: discard the loaded
                        // instance, then read the static field.
                        self.code.push_op(op::POP, 0);
                        self.code.drop_stack(1);
                        return self.emit_getfield(owner, &field);
                    }
                    return self.emit_getfield(owner, &field);
                }
            } else if let Some(class_id) = self.table.class_id(&path[0]) {
                let Some((owner, field)) = self.resolve_field(class_id, &path[1], span) else {
                    return JType::Error;
                };
                if !field.is_static {
                    self.error(
                        span,
                        format!(
                            "non-static variable {} cannot be referenced from a static context",
                            field.name
                        ),
                    );
                    return JType::Error;
                }
                return self.emit_getfield(owner, &field);
            }
        }
        // Longer dotted paths (`a.next.value`, `Class.field.x`) are a
        // field-access chain: resolve all but the last segment to an
        // object, then read the final field.
        if path.len() > 2 {
            let (prefix, last) = path.split_at(path.len() - 1);
            let object = Expr::Name {
                path: prefix.to_vec(),
                span,
            };
            return self.field(&object, &last[0], span);
        }
        if path.len() != 1 {
            self.error(span, format!("cannot find symbol: '{}'", path.join(".")));
            return JType::Error;
        }
        let name = &path[0];
        if self.lookup(name).is_none() {
            // Implicit field of the current class.
            if let Some((owner, field)) = self.table.field(self.current_class, name) {
                let field = field.clone();
                if field.is_static {
                    return self.emit_getfield(owner, &field);
                }
                if self.in_static {
                    self.error(
                        span,
                        format!(
                            "non-static variable {name} cannot be referenced from a \
                             static context"
                        ),
                    );
                    return JType::Error;
                }
                self.code.push_op(op::ALOAD_0, 1);
                return self.emit_getfield(owner, &field);
            }
            // A static field of the class this lambda/anonymous class came
            // from: hoisting lost the bare name, Java still resolves it.
            if let Some((owner, field)) = self.enclosing_static_field(name) {
                return self.emit_getfield(owner, &field);
            }
            // An instance field of the enclosing class, through the captured
            // `__caturraOuter`. Live on the real enclosing object.
            if let Some((outer, field_owner, field)) = self.enclosing_instance_field(name) {
                self.push_captured_outer(&outer);
                return self.emit_getfield(field_owner, &field);
            }
        }
        let Some(var) = self.lookup(name) else {
            self.error(span, format!("cannot find variable '{name}'"));
            return JType::Error;
        };
        let (slot, ty, assigned) = (var.slot, var.ty, var.assigned);
        if ty == JType::Unsupported {
            self.error(
                span,
                format!("the type of '{name}' is not yet supported by caturra"),
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
                    JType::Long => {
                        self.code.push_op(op::LNEG, 0);
                        JType::Long
                    }
                    JType::Float => {
                        self.code.push_op(op::FNEG, 0);
                        JType::Float
                    }
                    JType::Int | JType::Char | JType::Short | JType::Byte => {
                        self.code.push_op(op::INEG, 0);
                        JType::Int
                    }
                    JType::Error => JType::Error,
                    other => {
                        self.error(
                            span,
                            format!(
                                "operator '-' cannot be applied to {}",
                                other.describe(self.table)
                            ),
                        );
                        JType::Error
                    }
                }
            }
            UnaryOp::BitNot => {
                let ty = self.expr(operand);
                match ty {
                    JType::Int | JType::Char | JType::Short | JType::Byte => {
                        // ~x is x ^ -1.
                        self.code.push_op(op::ICONST_M1, 1);
                        self.code.push_op(op::IXOR, 0);
                        self.code.drop_stack(1);
                        JType::Int
                    }
                    JType::Long => {
                        let index = self.pool.intern(Constant::Long(-1));
                        self.code.push_op_u16(op::LDC2_W, index, 2);
                        self.code.push_op(op::LXOR, 0);
                        self.code.drop_stack(2);
                        JType::Long
                    }
                    JType::Error => JType::Error,
                    other => {
                        self.error(
                            span,
                            format!(
                                "operator '~' cannot be applied to {}",
                                other.describe(self.table)
                            ),
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
                        format!(
                            "operator '!' cannot be applied to {}",
                            ty.describe(self.table)
                        ),
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

    #[allow(clippy::too_many_lines)] // one conversion matrix
    fn cast(&mut self, ty: &TypeRef, operand: &Expr, span: SourceSpan) -> JType {
        let Some(target) = self.table.resolve_type(ty) else {
            self.error(span, "this cast target is not supported by caturra");
            self.expr(operand);
            return JType::Error;
        };
        let source = self.expr(operand);
        if source == JType::Error {
            return JType::Error;
        }
        // Casting a reference (commonly an erased Object) to String: a
        // runtime checkcast to java/lang/String.
        if target == JType::Str && source.is_reference() {
            if source != JType::Str {
                let class_index = intern_class(self.pool, "java/lang/String");
                self.code.push_op_u16(op::CHECKCAST, class_index, 0);
            }
            return JType::Str;
        }
        // Casting a reference (commonly an erased Object) to a List: a runtime
        // checkcast to java/util/ArrayList.
        if let JType::List(_) = target
            && source.is_reference()
        {
            if !matches!(source, JType::List(_)) {
                let class_index = intern_class(self.pool, "java/util/ArrayList");
                self.code.push_op_u16(op::CHECKCAST, class_index, 0);
            }
            return target;
        }
        // Reference casts between class/interface types.
        if let JType::Object(target_id) = target {
            match source {
                JType::Null => return target,
                JType::Object(source_id) => {
                    let upcast = self.table.is_subtype(source_id, target_id);
                    let downcast = self.table.is_subtype(target_id, source_id);
                    let source_is_interface = self
                        .table
                        .info_by_id(source_id)
                        .is_some_and(|i| i.is_interface);
                    let target_is_interface = self
                        .table
                        .info_by_id(target_id)
                        .is_some_and(|i| i.is_interface);
                    if upcast {
                        return target; // always safe, no check needed
                    }
                    if downcast || source_is_interface || target_is_interface {
                        let class_name = self.table.class_name(target_id).to_owned();
                        let class_index = intern_class(self.pool, &class_name);
                        self.code.push_op_u16(op::CHECKCAST, class_index, 0);
                        return target;
                    }
                    self.error(
                        span,
                        format!(
                            "incompatible types: {} cannot be converted to {}",
                            source.describe(self.table),
                            target.describe(self.table)
                        ),
                    );
                    return JType::Error;
                }
                // An array widens to `Object` — it is a reference, and every
                // array is-a Object. A safe upcast, no runtime check, the
                // value unchanged (the JVM leaves the reference on the stack).
                JType::Array { .. } if target_id == self.table.object_id => {
                    return target;
                }
                _ => {
                    self.error(
                        span,
                        format!(
                            "incompatible types: {} cannot be converted to {}",
                            source.describe(self.table),
                            target.describe(self.table)
                        ),
                    );
                    return JType::Error;
                }
            }
        }
        // Unbox a reference to a primitive (JLS §5.5): an erased `Object`
        // (e.g. from `Field.get`) or a wrapper, cast to `int`/`double`/…
        // `checkcast Wrapper` + `Wrapper.xxxValue()` — caturra's boxed values
        // answer the unboxing accessor directly.
        let erased_object = matches!(source, JType::Object(id) if id == self.table.object_id);
        if erased_object || matches!(source, JType::Boxed(_)) {
            // `(Double) obj` — cast to a wrapper: the runtime value is already
            // the boxed wrapper, so retag without emitting a conversion.
            if matches!(target, JType::Boxed(_)) {
                return target;
            }
            if let Some(elem) = elem_type_of(target) {
                self.emit_unbox(elem);
                return target;
            }
        }
        match (source, target) {
            (s, t) if s == t => t,
            (JType::Int | JType::Char | JType::Byte | JType::Short, JType::Long) => {
                self.code.push_op(op::I2L, 1);
                JType::Long
            }
            (JType::Long, JType::Int) => {
                self.code.push_op(op::L2I, 0);
                self.code.drop_stack(1);
                JType::Int
            }
            (JType::Long, JType::Char) => {
                self.code.push_op(op::L2I, 0);
                self.code.drop_stack(1);
                self.code.push_op(op::I2C, 0);
                JType::Char
            }
            (JType::Long, JType::Double) => {
                self.code.push_op(op::L2D, 0);
                JType::Double
            }
            (JType::Double, JType::Long) => {
                self.code.push_op(op::D2L, 0);
                JType::Long
            }
            (JType::Int | JType::Char | JType::Byte | JType::Short, JType::Float) => {
                self.code.push_op(op::I2F, 0);
                JType::Float
            }
            (JType::Long, JType::Float) => {
                self.code.push_op(op::L2F, 0);
                self.code.drop_stack(1);
                JType::Float
            }
            (JType::Double, JType::Float) => {
                self.code.push_op(op::D2F, 0);
                self.code.drop_stack(1);
                JType::Float
            }
            (JType::Float, JType::Int) => {
                self.code.push_op(op::F2I, 0);
                JType::Int
            }
            (JType::Float, JType::Char) => {
                self.code.push_op(op::F2I, 0);
                self.code.push_op(op::I2C, 0);
                JType::Char
            }
            (JType::Float, JType::Long) => {
                self.code.push_op(op::F2L, 1);
                JType::Long
            }
            (JType::Float, JType::Double) => {
                self.code.push_op(op::F2D, 1);
                JType::Double
            }
            (JType::Int | JType::Char | JType::Short, JType::Byte) => {
                self.code.push_op(op::I2B, 0);
                JType::Byte
            }
            (JType::Int | JType::Char | JType::Byte, JType::Short) => {
                self.code.push_op(op::I2S, 0);
                JType::Short
            }
            (JType::Long, JType::Byte) => {
                self.code.push_op(op::L2I, 0);
                self.code.drop_stack(1);
                self.code.push_op(op::I2B, 0);
                JType::Byte
            }
            (JType::Long, JType::Short) => {
                self.code.push_op(op::L2I, 0);
                self.code.drop_stack(1);
                self.code.push_op(op::I2S, 0);
                JType::Short
            }
            (JType::Float, JType::Byte) => {
                self.code.push_op(op::F2I, 0);
                self.code.push_op(op::I2B, 0);
                JType::Byte
            }
            (JType::Float, JType::Short) => {
                self.code.push_op(op::F2I, 0);
                self.code.push_op(op::I2S, 0);
                JType::Short
            }
            (JType::Double, JType::Byte) => {
                self.code.push_op(op::D2I, 0);
                self.code.drop_stack(1);
                self.code.push_op(op::I2B, 0);
                JType::Byte
            }
            (JType::Double, JType::Short) => {
                self.code.push_op(op::D2I, 0);
                self.code.drop_stack(1);
                self.code.push_op(op::I2S, 0);
                JType::Short
            }
            (JType::Double, JType::Int) => {
                self.code.push_op(op::D2I, 0);
                self.code.drop_stack(1);
                JType::Int
            }
            (JType::Int | JType::Char | JType::Byte | JType::Short, JType::Double) => {
                self.code.push_op(op::I2D, 1);
                JType::Double
            }
            (JType::Int | JType::Byte | JType::Short, JType::Char) => {
                self.code.push_op(op::I2C, 0);
                JType::Char
            }
            (JType::Double, JType::Char) => {
                self.code.push_op(op::D2I, 0);
                self.code.drop_stack(1);
                self.code.push_op(op::I2C, 0);
                JType::Char
            }
            (JType::Char | JType::Byte | JType::Short, JType::Int) => JType::Int,
            // Cast a reference down to an array type: `(int[]) obj`,
            // `(String[]) obj`, `(int[][]) obj`. A runtime `checkcast` to the
            // array's class descriptor (`[I`, `[Ljava/lang/String;`, `[[I`),
            // which is the array's own name in the constant pool.
            (JType::Null, JType::Array { .. }) => target,
            (src, JType::Array { .. }) if src.is_reference() => {
                let descriptor = target.descriptor(self.table);
                let class_index = intern_class(self.pool, &descriptor);
                self.code.push_op_u16(op::CHECKCAST, class_index, 0);
                target
            }
            (source, target) => {
                self.error(
                    span,
                    format!(
                        "cannot cast {} to {}",
                        source.describe(self.table),
                        target.describe(self.table)
                    ),
                );
                JType::Error
            }
        }
    }

    /// `cond ? then : else` — branches around the two value paths.
    #[allow(clippy::too_many_lines)] // one join-typing ladder
    fn ternary(&mut self, cond: &Expr, then: &Expr, els: &Expr, span: SourceSpan) -> JType {
        let then_ty = self.type_of(then);
        let els_ty = self.type_of(els);
        let table = self.table;
        let target = if then_ty == JType::Error || els_ty == JType::Error {
            JType::Error
        } else if then_ty == els_ty {
            then_ty
        } else if then_ty.is_numeric() && els_ty.is_numeric() {
            promote(then_ty, els_ty)
        } else if then_ty == JType::Null && els_ty.is_reference() {
            els_ty
        } else if els_ty == JType::Null && then_ty.is_reference() {
            then_ty
        } else if widens(then_ty, els_ty, table) {
            els_ty
        } else if widens(els_ty, then_ty, table) {
            then_ty
        } else {
            self.error(
                span,
                format!(
                    "incompatible types in conditional: {} and {}",
                    then_ty.describe(self.table),
                    els_ty.describe(self.table)
                ),
            );
            JType::Error
        };

        let cond_ty = self.expr(cond);
        if cond_ty != JType::Boolean && cond_ty != JType::Error {
            self.error(
                cond.span(),
                format!(
                    "incompatible types: {} cannot be converted to boolean",
                    cond_ty.describe(self.table)
                ),
            );
        }
        let else_label = self.code.new_label();
        let end = self.code.new_label();
        self.code.branch(op::IFEQ, else_label, 1);
        let actual = self.expr(then);
        if target.is_numeric() {
            self.numeric_conversion(actual, target);
        }
        self.code.branch(op::GOTO, end, 0);
        // The stack model tracks a single path; rewind for the else.
        self.code.drop_stack(target.width());
        self.code.bind(else_label);
        let actual = self.expr(els);
        if target.is_numeric() {
            self.numeric_conversion(actual, target);
        }
        self.code.bind(end);
        target
    }

    /// `x++` / `--a[i]` in expression position: postfix leaves the old
    /// value on the stack, prefix the new one.
    #[allow(clippy::too_many_lines)] // three target shapes × two orders
    fn inc_dec(&mut self, target: &Expr, increment: bool, prefix: bool, span: SourceSpan) -> JType {
        let one_op = |emitter: &mut Self, ty: JType| {
            if ty == JType::Double {
                let index = emitter.pool.intern(Constant::Double(1.0));
                emitter.code.push_op_u16(op::LDC2_W, index, 2);
                emitter
                    .code
                    .push_op(if increment { op::DADD } else { op::DSUB }, 0);
                emitter.code.drop_stack(2);
            } else if ty == JType::Long {
                emitter.code.push_op(op::LCONST_1, 2);
                emitter
                    .code
                    .push_op(if increment { op::LADD } else { op::LSUB }, 0);
                emitter.code.drop_stack(2);
            } else if ty == JType::Float {
                emitter.code.push_op(op::FCONST_1, 1);
                emitter
                    .code
                    .push_op(if increment { op::FADD } else { op::FSUB }, 0);
                emitter.code.drop_stack(1);
            } else {
                emitter.code.push_op(op::ICONST_1, 1);
                emitter
                    .code
                    .push_op(if increment { op::IADD } else { op::ISUB }, 0);
                emitter.code.drop_stack(1);
                if ty == JType::Char {
                    emitter.code.push_op(op::I2C, 0);
                }
                if ty == JType::Byte {
                    emitter.code.push_op(op::I2B, 0);
                }
                if ty == JType::Short {
                    emitter.code.push_op(op::I2S, 0);
                }
            }
        };

        match target {
            Expr::Name { path, .. } if path.len() == 1 => {
                let Some(var) = self.lookup(&path[0]) else {
                    self.error(span, format!("cannot find variable '{}'", path[0]));
                    return JType::Error;
                };
                let (slot, ty) = (var.slot, var.ty);
                if !ty.is_numeric() {
                    self.error(
                        span,
                        format!(
                            "++/-- needs a numeric variable, got {}",
                            ty.describe(self.table)
                        ),
                    );
                    return JType::Error;
                }
                self.emit_load(slot, ty);
                if !prefix {
                    // Keep the old value under the update.
                    if ty.width() == 2 {
                        self.code.push_op(op::DUP2, 2);
                    } else {
                        self.code.push_op(op::DUP, 1);
                    }
                }
                one_op(self, ty);
                if prefix {
                    if ty.width() == 2 {
                        self.code.push_op(op::DUP2, 2);
                    } else {
                        self.code.push_op(op::DUP, 1);
                    }
                }
                self.emit_store(slot, ty);
                ty
            }
            Expr::Index { array, index, .. } => {
                let array_ty = self.expr(array);
                let Some(elem_ty) = array_ty.element_type() else {
                    self.error(span, "++/-- on a non-array element");
                    return JType::Error;
                };
                if !elem_ty.is_numeric() {
                    self.error(span, "++/-- needs a numeric element");
                    return JType::Error;
                }
                let index_ty = self.expr(index);
                self.numeric_conversion(index_ty, JType::Int);
                // [arr, i] → dup2 → [arr, i, arr, i] → load elem.
                self.code.push_op(op::DUP2, 2);
                let (load, store) = match elem_ty {
                    JType::Double => (op::DALOAD, op::DASTORE),
                    JType::Long => (op::LALOAD, op::LASTORE),
                    JType::Float => (op::FALOAD, op::FASTORE),
                    JType::Short => (op::SALOAD, op::SASTORE),
                    JType::Byte => (op::BALOAD, op::BASTORE),
                    JType::Char => (op::CALOAD, op::CASTORE),
                    _ => (op::IALOAD, op::IASTORE),
                };
                self.code.push_op(load, elem_ty.width());
                self.code.drop_stack(2);
                if !prefix {
                    // Old value tucked under [arr, i, old].
                    if elem_ty.width() == 2 {
                        self.code.push_op(op::DUP2_X2, 2);
                    } else {
                        self.code.push_op(op::DUP_X2, 1);
                    }
                }
                one_op(self, elem_ty);
                if prefix {
                    if elem_ty.width() == 2 {
                        self.code.push_op(op::DUP2_X2, 2);
                    } else {
                        self.code.push_op(op::DUP_X2, 1);
                    }
                }
                self.code.push_op(store, 0);
                self.code.drop_stack(2 + elem_ty.width());
                elem_ty
            }
            _ => {
                // Field targets (this.count++, obj.n--): reuse the
                // statement lowering path via a synthetic compound
                // assignment... which cannot yield a value; report the
                // one unsupported shape honestly.
                self.error(
                    span,
                    "++/-- as an expression works on variables and array elements \
                     (use a statement form for fields)",
                );
                self.expr(target);
                JType::Error
            }
        }
    }

    fn binary(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: SourceSpan) -> JType {
        match op {
            BinaryOp::And | BinaryOp::Or => self.logical(op, lhs, rhs, span),
            BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => {
                self.bitwise(op, lhs, rhs, span)
            }
            BinaryOp::Shl | BinaryOp::Shr | BinaryOp::Ushr => {
                let (lt, rt) = (
                    numeric_view(self.type_of(lhs)),
                    numeric_view(self.type_of(rhs)),
                );
                let integral = |t: JType| {
                    matches!(
                        t,
                        JType::Int | JType::Char | JType::Long | JType::Short | JType::Byte
                    )
                };
                if lt != JType::Error && rt != JType::Error && (!integral(lt) || !integral(rt)) {
                    self.error(
                        span,
                        format!(
                            "operator '{}' cannot be applied to {} and {}",
                            arithmetic_symbol(op),
                            lt.describe(self.table),
                            rt.describe(self.table)
                        ),
                    );
                }
                // The left operand's type decides the result; the
                // count is always int (JLS §15.19).
                let long_shift = lt == JType::Long;
                let actual = self.expr(lhs);
                if !long_shift {
                    self.numeric_conversion(actual, JType::Int);
                }
                let actual = self.expr(rhs);
                if actual == JType::Long {
                    self.code.push_op(op::L2I, 0);
                    self.code.drop_stack(1);
                } else {
                    self.numeric_conversion(actual, JType::Int);
                }
                let opcode = match (op, long_shift) {
                    (BinaryOp::Shl, false) => op::ISHL,
                    (BinaryOp::Shr, false) => op::ISHR,
                    (BinaryOp::Ushr, false) => op::IUSHR,
                    (BinaryOp::Shl, true) => op::LSHL,
                    (BinaryOp::Shr, true) => op::LSHR,
                    (BinaryOp::Ushr, true) => op::LUSHR,
                    _ => unreachable!(),
                };
                self.code.push_op(opcode, 0);
                self.code.drop_stack(1);
                if long_shift { JType::Long } else { JType::Int }
            }
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
                let (lt, rt) = (
                    numeric_view(self.type_of(lhs)),
                    numeric_view(self.type_of(rhs)),
                );
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
                            lt.describe(self.table),
                            rt.describe(self.table)
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

    /// `& | ^`: bitwise on ints, non-short-circuit logical on
    /// booleans (both operands always evaluate — the JLS semantics).
    fn bitwise(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: SourceSpan) -> JType {
        let (lt, rt) = (
            numeric_view(self.type_of(lhs)),
            numeric_view(self.type_of(rhs)),
        );
        let boolean = lt == JType::Boolean && rt == JType::Boolean;
        let is_integral = |t: JType| {
            matches!(
                t,
                JType::Int | JType::Char | JType::Long | JType::Short | JType::Byte
            )
        };
        let integral = is_integral(lt) && is_integral(rt);
        if lt != JType::Error && rt != JType::Error && !boolean && !integral {
            self.error(
                span,
                format!(
                    "operator '{}' cannot be applied to {} and {}",
                    arithmetic_symbol(op),
                    lt.describe(self.table),
                    rt.describe(self.table)
                ),
            );
        }
        let target = if boolean {
            JType::Boolean
        } else if integral {
            promote(lt, rt)
        } else {
            JType::Int
        };
        let actual = self.expr(lhs);
        if integral {
            self.numeric_conversion(actual, target);
        }
        let actual = self.expr(rhs);
        if integral {
            self.numeric_conversion(actual, target);
        }
        let opcode = match (op, target) {
            (BinaryOp::BitAnd, JType::Long) => op::LAND,
            (BinaryOp::BitOr, JType::Long) => op::LOR,
            (BinaryOp::BitXor, JType::Long) => op::LXOR,
            (BinaryOp::BitAnd, _) => op::IAND,
            (BinaryOp::BitOr, _) => op::IOR,
            (_, _) => op::IXOR,
        };
        self.code.push_op(opcode, 0);
        self.code.drop_stack(target.width());
        target
    }

    fn logical(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: SourceSpan) -> JType {
        let lt = self.expr(lhs);
        if lt != JType::Boolean && lt != JType::Error {
            self.error(
                span,
                format!(
                    "operator '{}' needs boolean operands, got {}",
                    if op == BinaryOp::And { "&&" } else { "||" },
                    lt.describe(self.table)
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
                    rt.describe(self.table)
                ),
            );
        }
        self.code.branch(op::GOTO, end, 0);
        self.code.bind(short);
        self.code.push_op(shortcut_value, 1);
        self.code.bind(end);
        JType::Boolean
    }

    #[allow(clippy::too_many_lines)] // one arm per operand-type family
    fn comparison(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: SourceSpan) -> JType {
        let (raw_l, raw_r) = (self.type_of(lhs), self.type_of(rhs));
        // Comparing against `null` is a reference comparison (JLS §15.21.3),
        // so a wrapper stays boxed: `map.get(k) == null` asks whether the
        // mapping is absent, and must not try to unbox it first.
        let against_null = matches!(op, BinaryOp::Eq | BinaryOp::Ne)
            && (raw_l == JType::Null || raw_r == JType::Null);
        let (lt, rt) = if against_null {
            (raw_l, raw_r)
        } else {
            (numeric_view(raw_l), numeric_view(raw_r))
        };
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
                    lt.describe(self.table),
                    rt.describe(self.table)
                ),
            );
            return JType::Error;
        }

        if both_numeric && promote(lt, rt) == JType::Long {
            let actual_l = self.expr(lhs);
            self.numeric_conversion(actual_l, JType::Long);
            let actual_r = self.expr(rhs);
            self.numeric_conversion(actual_r, JType::Long);
            let jump = match op {
                BinaryOp::Lt => op::IFLT,
                BinaryOp::Le => op::IFLE,
                BinaryOp::Gt => op::IFGT,
                BinaryOp::Ge => op::IFGE,
                BinaryOp::Eq => op::IFEQ,
                BinaryOp::Ne => op::IFNE,
                _ => unreachable!(),
            };
            self.code.push_op(op::LCMP, 0);
            self.code.drop_stack(3); // two longs -> one int
            self.boolean_from_branch(jump);
            return JType::Boolean;
        }
        if both_numeric && promote(lt, rt) == JType::Float {
            let actual_l = self.expr(lhs);
            self.numeric_conversion(actual_l, JType::Float);
            let actual_r = self.expr(rhs);
            self.numeric_conversion(actual_r, JType::Float);
            let (cmp, jump) = match op {
                BinaryOp::Lt => (op::FCMPG, op::IFLT),
                BinaryOp::Le => (op::FCMPG, op::IFLE),
                BinaryOp::Gt => (op::FCMPL, op::IFGT),
                BinaryOp::Ge => (op::FCMPL, op::IFGE),
                BinaryOp::Eq => (op::FCMPL, op::IFEQ),
                BinaryOp::Ne => (op::FCMPL, op::IFNE),
                _ => unreachable!(),
            };
            self.code.push_op(cmp, 0);
            self.code.drop_stack(1);
            self.boolean_from_branch(jump);
            return JType::Boolean;
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
        let ty = self.coerce_to_string_for_output(ty);
        let descriptor = match ty {
            JType::Generic { .. }
            | JType::StringBuilder
            | JType::TypeVar
            | JType::Boxed(_)
            | JType::Map { .. }
            | JType::TreeMap { .. }
            | JType::Set(_)
            | JType::TreeSet(_)
            | JType::Stream(_)
            | JType::Collector
            | JType::IntStream
            | JType::Optional(_)
            | JType::OptionalInt
            | JType::OptionalDouble
            | JType::LinkedList { .. }
            | JType::Collection(_)
            | JType::EntrySet { .. }
            | JType::MapEntry { .. }
            | JType::Class
            | JType::Field
            | JType::Method
            | JType::Type
            | JType::Constructor
            // Concatenation is `String.valueOf(Object)` (JLS 5.1.11), so even
            // a `char[]` renders as `[C@hash` here, unlike `println(char[])`.
            | JType::Array { .. } => "(Ljava/lang/Object;)Ljava/lang/StringBuilder;",
            JType::Int | JType::Short | JType::Byte => "(I)Ljava/lang/StringBuilder;",
            JType::Long => "(J)Ljava/lang/StringBuilder;",
            JType::Float => "(F)Ljava/lang/StringBuilder;",
            JType::Double => "(D)Ljava/lang/StringBuilder;",
            JType::Boolean => "(Z)Ljava/lang/StringBuilder;",
            JType::Char => "(C)Ljava/lang/StringBuilder;",
            JType::Str
            | JType::Null
            | JType::Object(_)
            | JType::List(_)
            | JType::Stack(_)
            | JType::File
            | JType::Exception(_) => "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
            JType::Scanner | JType::Writer => {
                self.error(
                    span,
                    format!(
                        "concatenating a {} is not supported",
                        ty.describe(self.table)
                    ),
                );
                return;
            }
            JType::Unsupported => {
                self.error(span, "this value's type is not yet supported by caturra");
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
    /// Box the primitive on the stack into its wrapper via
    /// `Wrapper.valueOf(prim)`.
    fn emit_box(&mut self, elem: ElemType) {
        let internal = wrapper_internal(elem);
        let prim = elem.base_type();
        let descriptor = format!("({})L{internal};", prim.descriptor(self.table));
        let method_ref = intern_method_ref(self.pool, internal, "valueOf", &descriptor);
        self.code.push_op_u16(op::INVOKESTATIC, method_ref, 0);
        // prim (width w) -> one reference; net stack change 1 - w.
        self.code.drop_stack(prim.width());
        self.code.grow_stack(1);
    }

    /// Unbox the wrapper on the stack into its primitive via
    /// `wrapper.xValue()`.
    fn emit_unbox(&mut self, elem: ElemType) {
        let internal = wrapper_internal(elem);
        let prim = elem.base_type();
        let method = match elem {
            ElemType::Int
            | ElemType::Str
            | ElemType::Object(_)
            | ElemType::Field
            | ElemType::Method
            | ElemType::Constructor
            | ElemType::Class => "intValue",
            ElemType::Double => "doubleValue",
            ElemType::Long => "longValue",
            ElemType::Float => "floatValue",
            ElemType::Short => "shortValue",
            ElemType::Byte => "byteValue",
            ElemType::Char => "charValue",
            ElemType::Boolean => "booleanValue",
        };
        let descriptor = format!("(){}", prim.descriptor(self.table));
        let method_ref = intern_method_ref(self.pool, internal, method, &descriptor);
        self.code
            .push_op_u16(op::INVOKEVIRTUAL, method_ref, prim.width());
        self.code.drop_stack(1);
    }

    fn numeric_conversion(&mut self, from: JType, target: JType) {
        // Auto-unbox a wrapper operand to its primitive, then convert.
        if let JType::Boxed(elem) = from {
            self.emit_unbox(elem);
            self.numeric_conversion(elem.base_type(), target);
            return;
        }
        match (from, target) {
            (JType::Int | JType::Char | JType::Short | JType::Byte, JType::Double) => {
                self.code.push_op(op::I2D, 1);
            }
            (JType::Int | JType::Char | JType::Short | JType::Byte, JType::Long) => {
                self.code.push_op(op::I2L, 1);
            }
            (JType::Int | JType::Char | JType::Short | JType::Byte, JType::Float) => {
                self.code.push_op(op::I2F, 0);
            }
            (JType::Long, JType::Double) => self.code.push_op(op::L2D, 0),
            (JType::Long, JType::Float) => {
                self.code.push_op(op::L2F, 0);
                self.code.drop_stack(1);
            }
            (JType::Float, JType::Double) => self.code.push_op(op::F2D, 1),
            _ => {}
        }
    }

    /// Implicit narrowing after a compound assignment (JLS §15.26.2).
    fn narrow_back(&mut self, from: JType, to: JType) {
        match (from, to) {
            (JType::Double, JType::Float) => {
                self.code.push_op(op::D2F, 0);
                self.code.drop_stack(1);
            }
            (JType::Float, JType::Int | JType::Char | JType::Short | JType::Byte) => {
                self.code.push_op(op::F2I, 0);
            }
            (JType::Float, JType::Long) => {
                self.code.push_op(op::F2L, 1);
            }
            (JType::Float, JType::Double) => self.code.push_op(op::F2D, 1),
            (JType::Long, JType::Float) => {
                self.code.push_op(op::L2F, 0);
                self.code.drop_stack(1);
            }
            (JType::Int | JType::Char, JType::Float) => self.code.push_op(op::I2F, 0),
            (JType::Double, JType::Int | JType::Char | JType::Short | JType::Byte) => {
                self.code.push_op(op::D2I, 0);
                self.code.drop_stack(1);
            }
            (JType::Double, JType::Long) => {
                self.code.push_op(op::D2L, 0);
            }
            (JType::Long, JType::Int | JType::Char | JType::Short | JType::Byte) => {
                self.code.push_op(op::L2I, 0);
                self.code.drop_stack(1);
            }
            (JType::Long, JType::Double) => {
                self.code.push_op(op::L2D, 0);
            }
            _ => {}
        }
        if to == JType::Char {
            self.code.push_op(op::I2C, 0);
        }
        if to == JType::Byte {
            self.code.push_op(op::I2B, 0);
        }
        if to == JType::Short {
            self.code.push_op(op::I2S, 0);
        }
    }

    fn arithmetic_op(&mut self, operator: BinaryOp, ty: JType) {
        let opcode = match (operator, ty) {
            (BinaryOp::Add, JType::Float) => op::FADD,
            (BinaryOp::Sub, JType::Float) => op::FSUB,
            (BinaryOp::Mul, JType::Float) => op::FMUL,
            (BinaryOp::Div, JType::Float) => op::FDIV,
            (BinaryOp::Rem, JType::Float) => op::FREM,
            (BinaryOp::Add, JType::Long) => op::LADD,
            (BinaryOp::Sub, JType::Long) => op::LSUB,
            (BinaryOp::Mul, JType::Long) => op::LMUL,
            (BinaryOp::Div, JType::Long) => op::LDIV,
            (BinaryOp::Rem, JType::Long) => op::LREM,
            (BinaryOp::BitAnd, JType::Long) => op::LAND,
            (BinaryOp::BitOr, JType::Long) => op::LOR,
            (BinaryOp::BitXor, JType::Long) => op::LXOR,
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
            (BinaryOp::BitAnd, _) => op::IAND,
            (BinaryOp::BitOr, _) => op::IOR,
            (BinaryOp::BitXor, _) => op::IXOR,
            (BinaryOp::Shl, _) => op::ISHL,
            (BinaryOp::Shr, _) => op::ISHR,
            (BinaryOp::Ushr, _) => op::IUSHR,
            _ => unreachable!("not an arithmetic operator"),
        };
        self.code.push_op(opcode, 0);
        self.code.drop_stack(ty.width());
    }

    fn emit_load(&mut self, slot: u16, ty: JType) {
        let (base, short_base) = match ty {
            JType::Double => (op::DLOAD, op::DLOAD_0),
            JType::Long => (op::LLOAD, op::LLOAD_0),
            JType::Float => (op::FLOAD, op::FLOAD_0),
            JType::Str
            | JType::Null
            | JType::Array { .. }
            | JType::Scanner
            | JType::File
            | JType::Writer
            | JType::List(_)
            | JType::Stack(_)
            | JType::Map { .. }
            | JType::Set(_)
            | JType::Collection(_)
            | JType::EntrySet { .. }
            | JType::MapEntry { .. }
            | JType::Exception(_) => (op::ALOAD, op::ALOAD_0),
            _ => (op::ILOAD, op::ILOAD_0),
        };
        self.local_op(base, short_base, slot);
        self.code.grow_stack(ty.width());
    }

    fn emit_store(&mut self, slot: u16, ty: JType) {
        let (base, short_base) = match ty {
            JType::Double => (op::DSTORE, op::DSTORE_0),
            JType::Long => (op::LSTORE, op::LSTORE_0),
            JType::Float => (op::FSTORE, op::FSTORE_0),
            JType::Str
            | JType::Null
            | JType::Array { .. }
            | JType::Scanner
            | JType::File
            | JType::Writer
            | JType::List(_)
            | JType::Stack(_)
            | JType::Map { .. }
            | JType::Set(_)
            | JType::Collection(_)
            | JType::EntrySet { .. }
            | JType::MapEntry { .. }
            | JType::Exception(_) => (op::ASTORE, op::ASTORE_0),
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
        self.convert_for_assignment_const(from, to, span, None);
    }

    /// Like [`Self::convert_for_assignment`], but aware of the JLS
    /// constant-narrowing rule: an int CONSTANT in range assigns to
    /// byte/short/char without a cast (`byte b = 5;`).
    #[allow(clippy::too_many_lines)] // one conversion matrix
    fn convert_for_assignment_const(
        &mut self,
        from: JType,
        to: JType,
        span: SourceSpan,
        constant: Option<i64>,
    ) {
        // Constant narrowing (JLS §5.2): the value is already an int
        // on the stack; in-range constants need no code at all.
        if from == JType::Int
            && let Some(value) = constant
        {
            let in_range = match to {
                JType::Byte => i8::try_from(value).is_ok(),
                JType::Short => i16::try_from(value).is_ok(),
                JType::Char => (0..=i64::from(u16::MAX)).contains(&value),
                _ => false,
            };
            if in_range {
                return;
            }
        }
        // Autoboxing / auto-unboxing (JLS §5.1.7 / §5.1.8).
        if from != to {
            // Box a primitive into its wrapper, or into Object.
            if let JType::Boxed(elem) = to
                && from == elem.base_type()
            {
                self.emit_box(elem);
                return;
            }
            if to == JType::Object(self.table.object_id)
                && let Some(elem) = boxable_primitive(from)
            {
                self.emit_box(elem);
                return;
            }
            // Unbox a wrapper to its primitive (then widen if needed).
            if let JType::Boxed(elem) = from {
                let primitive = elem.base_type();
                self.emit_unbox(elem);
                self.convert_for_assignment(primitive, to, span);
                return;
            }
        }
        match (from, to) {
            // Identity, small-int widening, and null-to-String all
            // need no code (byte/short/char are ints at runtime).
            (JType::Error, _)
            | (JType::Char | JType::Short | JType::Byte, JType::Int)
            | (JType::Byte, JType::Short) => {}
            (JType::Null, to) if to.is_reference() => {}
            (JType::Object(sub), JType::Object(sup)) if self.table.is_subtype(sub, sup) => {}
            // Any reference type widens to the Object top type.
            (from, JType::Object(id)) if id == self.table.object_id && from.is_reference() => {}
            // A parameterized type and its raw class erase alike.
            (a, b) if a.erased_class().is_some() && a.erased_class() == b.erased_class() => {}
            // A LinkedList (its Queue/Deque face) or a TreeSet erases to a
            // list/set, so assigning it to a wider face, a `List`/`Set`, or a
            // `Collection` of the same element type needs no code — as `widens`
            // already allows.
            (JType::LinkedList { .. } | JType::TreeSet(_) | JType::TreeMap { .. }, _)
                if widens(from, to, self.table) => {}
            // Array covariance: `Card[]` assigns to `Comparable[]`.
            (
                JType::Array {
                    elem: ElemType::Object(sub),
                    dims: d1,
                },
                JType::Array {
                    elem: ElemType::Object(sup),
                    dims: d2,
                },
            ) if d1 == d2 && self.table.is_subtype(sub, sup) => {}
            // Any reference array widens to `Object[]` (`String[]`,
            // `Field[]` -> `Object[]` for `Arrays.toString`).
            (
                JType::Array {
                    elem: ElemType::Str | ElemType::Field | ElemType::Constructor,
                    dims: d1,
                },
                JType::Array {
                    elem: ElemType::Object(sup),
                    dims: d2,
                },
            ) if d1 == d2 && sup == self.table.object_id => {}
            // A multi-dimensional array is an `Object[]` of its rows.
            (
                JType::Array { dims: d1, .. },
                JType::Array {
                    elem: ElemType::Object(sup),
                    dims: 1,
                },
            ) if d1 >= 2 && sup == self.table.object_id => {}
            // Any reference stores into a type variable; a type variable
            // reads out as Object.
            (from, JType::TypeVar) if from.is_reference() => {}
            (JType::TypeVar, JType::Object(id)) if id == self.table.object_id => {}
            (f, t) if f == t => {}
            (JType::Int | JType::Char | JType::Short | JType::Byte, JType::Double) => {
                self.code.push_op(op::I2D, 1);
            }
            (JType::Int | JType::Char | JType::Short | JType::Byte, JType::Long) => {
                self.code.push_op(op::I2L, 1);
            }
            (JType::Long, JType::Double) => self.code.push_op(op::L2D, 0),
            (JType::Int | JType::Char | JType::Short | JType::Byte, JType::Float) => {
                self.code.push_op(op::I2F, 0);
            }
            (JType::Long, JType::Float) => {
                self.code.push_op(op::L2F, 0);
                self.code.drop_stack(1);
            }
            (JType::Float, JType::Double) => self.code.push_op(op::F2D, 1),
            (JType::Int | JType::Char | JType::Short, JType::Byte)
            | (JType::Int | JType::Char, JType::Short)
            | (JType::Short | JType::Byte, JType::Char) => {
                self.error(
                    span,
                    format!(
                        "incompatible types: possible lossy conversion from {} to {}",
                        from.describe(self.table),
                        to.describe(self.table)
                    ),
                );
            }
            (JType::Float, JType::Int | JType::Char | JType::Long) => {
                self.error(
                    span,
                    format!(
                        "incompatible types: possible lossy conversion from float to {}",
                        to.describe(self.table)
                    ),
                );
            }
            (JType::Double, JType::Float) => {
                self.error(
                    span,
                    "incompatible types: possible lossy conversion from double to float",
                );
            }
            (JType::Long, JType::Int | JType::Char) => {
                self.error(
                    span,
                    format!(
                        "incompatible types: possible lossy conversion from long to {}",
                        to.describe(self.table)
                    ),
                );
            }
            (JType::Double, JType::Long) => {
                self.error(
                    span,
                    "incompatible types: possible lossy conversion from double to long",
                );
            }
            (JType::Double, JType::Int | JType::Char) => {
                self.error(
                    span,
                    format!(
                        "possible lossy conversion from double to {}; add an explicit cast",
                        to.describe(self.table)
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
                        from.describe(self.table),
                        to.describe(self.table)
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
        BinaryOp::BitAnd => "&=",
        BinaryOp::BitOr => "|=",
        BinaryOp::BitXor => "^=",
        BinaryOp::Shl => "<<=",
        BinaryOp::Shr => ">>=",
        BinaryOp::Ushr => ">>>=",
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
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
        BinaryOp::BitXor => "^",
        BinaryOp::Shl => "<<",
        BinaryOp::Shr => ">>",
        BinaryOp::Ushr => ">>>",
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
    /// `(bytecode offset, source line)` pairs for the
    /// `LineNumberTable` debug attribute.
    line_numbers: Vec<(u16, u16)>,
    /// JVMS exception table entries (offsets are final at emission).
    exception_entries: Vec<caturra_classfile::ExceptionTableEntry>,
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
            line_numbers: Vec::new(),
            exception_entries: Vec::new(),
            labels: Vec::new(),
            patches: Vec::new(),
        }
    }

    /// The VM materializes state at an exception handler (one thrown
    /// object on an otherwise empty stack); account for it.
    fn assume_stack(&mut self, depth: u16) {
        self.depth = self.depth.max(depth);
        self.max_stack = self.max_stack.max(self.depth);
    }

    /// Register a `try` range with its handler and catch type.
    fn add_exception_entry(&mut self, start_pc: u16, end_pc: u16, handler_pc: u16, catch: CpIndex) {
        self.exception_entries
            .push(caturra_classfile::ExceptionTableEntry {
                start_pc,
                end_pc,
                handler_pc,
                catch_type: catch,
            });
    }

    /// Current bytecode offset (for debug live ranges).
    fn offset(&self) -> u16 {
        u16::try_from(self.bytes.len()).unwrap_or(u16::MAX)
    }

    /// Record that the code emitted from here on corresponds to
    /// `line` (1-based). Consecutive duplicate lines and same-offset
    /// re-marks collapse.
    fn mark_line(&mut self, line: u32) {
        let line = u16::try_from(line).unwrap_or(u16::MAX);
        let offset = u16::try_from(self.bytes.len()).unwrap_or(u16::MAX);
        if let Some((last_offset, last_line)) = self.line_numbers.last_mut() {
            if *last_line == line {
                return;
            }
            if *last_offset == offset {
                *last_line = line;
                return;
            }
        }
        self.line_numbers.push((offset, line));
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
    #[allow(clippy::type_complexity)] // one assembly hand-off
    fn finish(
        mut self,
    ) -> (
        Vec<u8>,
        u16,
        Vec<(u16, u16)>,
        Vec<caturra_classfile::ExceptionTableEntry>,
    ) {
        for (patch_at, opcode_at, label) in &self.patches {
            let target = self.labels[label.0].expect("branch to unbound label");
            let offset = i32::try_from(target).expect("code too large")
                - i32::try_from(*opcode_at).expect("code too large");
            let offset = i16::try_from(offset).expect("branch offset exceeds 16 bits");
            self.bytes[*patch_at..*patch_at + 2].copy_from_slice(&offset.to_be_bytes());
        }
        (
            self.bytes,
            self.max_stack,
            self.line_numbers,
            self.exception_entries,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;
    use caturra_classfile::read_code_attribute;

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
