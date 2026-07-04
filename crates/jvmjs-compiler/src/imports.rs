//! Import validation and enforcement.
//!
//! Two directions, both matching javac where the library overlaps:
//! - import declarations must name something real (unknown classes and
//!   packages get javac's wording; real Java classes outside the CSA
//!   library get an honest "not supported by jvmjs" instead), and
//! - using a non-`java.lang` library class (`Scanner`, `ArrayList`,
//!   `File`, `PrintWriter`) requires the matching import — javac's
//!   "cannot find symbol: class Scanner". User-defined classes shadow
//!   library names and never need imports.

use std::collections::HashSet;

use crate::ast::{ClassDecl, CompilationUnit, Expr, ImportDecl, Stmt, TypeRef};
use crate::diagnostics::{Diagnostic, SourceSpan};

/// Importable library classes, per package.
const JAVA_UTIL: &[&str] = &[
    "Scanner",
    "ArrayList",
    "InputMismatchException",
    "NoSuchElementException",
    "IllegalFormatException",
    "UnknownFormatConversionException",
    "MissingFormatArgumentException",
    "IllegalFormatConversionException",
];
const JAVA_IO: &[&str] = &[
    "File",
    "PrintWriter",
    "FileNotFoundException",
    "IOException",
];
/// `java.lang` is implicitly imported; explicit imports of it are
/// legal (and redundant) in Java, so accept the names we model.
const JAVA_LANG: &[&str] = &[
    "String",
    "Object",
    "System",
    "Math",
    "Integer",
    "Double",
    "Long",
    "Float",
    "Short",
    "Byte",
    "Boolean",
    "Character",
    "StringBuilder",
    "Exception",
    "RuntimeException",
    "ArithmeticException",
    "NullPointerException",
    "ArrayIndexOutOfBoundsException",
    "IndexOutOfBoundsException",
    "StringIndexOutOfBoundsException",
    "NegativeArraySizeException",
    "NumberFormatException",
    "ClassCastException",
    "StackOverflowError",
    "Throwable",
    "Error",
    "IllegalArgumentException",
    "IllegalStateException",
];

/// Real Java classes students may reach for that jvmjs doesn't
/// implement — named so the message is honest instead of a misleading
/// "cannot find symbol".
const KNOWN_UNSUPPORTED: &[(&str, &[&str])] = &[
    (
        "java.util",
        &[
            "HashMap",
            "HashSet",
            "TreeMap",
            "TreeSet",
            "LinkedList",
            "ArrayDeque",
            "Stack",
            "List",
            "Map",
            "Set",
            "Queue",
            "Deque",
            "Iterator",
            "Collections",
            "Arrays",
            "Random",
            "Optional",
            "Comparator",
        ],
    ),
    (
        "java.io",
        &[
            "BufferedReader",
            "BufferedWriter",
            "FileReader",
            "FileWriter",
            "InputStreamReader",
            "PrintStream",
            "InputStream",
            "OutputStream",
            "Reader",
            "Writer",
        ],
    ),
];

/// Real JDK packages we don't model at all (for wildcard/unknown-class
/// imports of them, an honest message beats "does not exist").
const KNOWN_UNSUPPORTED_PACKAGES: &[&str] = &[
    "java.awt",
    "java.net",
    "java.nio",
    "java.time",
    "java.text",
    "java.math",
    "java.sql",
    "javax.swing",
    "java.util.function",
    "java.util.stream",
    "java.util.regex",
    "java.util.concurrent",
];

/// Library type names whose use requires an import.
const REQUIRES_IMPORT: &[&str] = &[
    "Scanner",
    "ArrayList",
    "File",
    "PrintWriter",
    "InputMismatchException",
    "NoSuchElementException",
    "IOException",
    "FileNotFoundException",
];

/// Resolve a fully qualified library name (`java.util.Scanner`) to the
/// simple name the compiler models. Fully qualified uses never need an
/// import — that is their purpose in Java.
pub(crate) fn canonical_library_class(dotted: &str) -> Option<&'static str> {
    let (package, class) = dotted.rsplit_once('.')?;
    let known: &[&str] = match package {
        "java.util" => &["Scanner", "ArrayList"],
        "java.io" => &["File", "PrintWriter"],
        "java.lang" => &[
            "String",
            "Math",
            "Integer",
            "Double",
            "Long",
            "Float",
            "Short",
            "Byte",
            "Boolean",
            "Character",
            "System",
        ],
        _ => return None,
    };
    known.iter().find(|name| **name == class).copied()
}

/// javac-style message for a fully qualified name the library doesn't
/// model (honest "not supported" for real Java classes and packages).
pub(crate) fn unknown_qualified_message(dotted: &str) -> String {
    let Some((package, class)) = dotted.rsplit_once('.') else {
        return format!("unknown type '{dotted}'");
    };
    if package_classes(package).is_some() {
        if KNOWN_UNSUPPORTED
            .iter()
            .any(|(pkg, names)| *pkg == package && names.contains(&class))
            || package_classes(package).is_some_and(|names| names.contains(&class))
        {
            // Real (or modeled-but-not-usable-here) Java class.
            return not_supported(dotted);
        }
        return format!("cannot find symbol: class {class} in package {package}");
    }
    if KNOWN_UNSUPPORTED_PACKAGES.contains(&package) {
        return not_supported(&format!("package {package}"));
    }
    format!("package {package} does not exist")
}

fn package_classes(package: &str) -> Option<&'static [&'static str]> {
    match package {
        "java.util" => Some(JAVA_UTIL),
        "java.io" => Some(JAVA_IO),
        "java.lang" => Some(JAVA_LANG),
        _ => None,
    }
}

fn not_supported(what: &str) -> String {
    format!("{what} is not supported by jvmjs (the class library covers the AP CS A subset)")
}

/// Validate a unit's imports and check that library classes are only
/// used under a matching import. `user_classes` holds every class name
/// defined across the whole compilation (one shared namespace).
pub fn check_unit(
    path: &str,
    unit: &CompilationUnit,
    user_classes: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut error = |message: String, span: SourceSpan| {
        diagnostics.push(Diagnostic::error(path, message, span));
    };

    // Which import-requiring names this unit has enabled.
    let mut enabled: HashSet<&'static str> = HashSet::new();

    for import in &unit.imports {
        validate_import(import, &mut enabled, &mut error);
    }

    for class in &unit.classes {
        check_class(class, user_classes, &enabled, &mut error);
    }
}

fn validate_import(
    import: &ImportDecl,
    enabled: &mut HashSet<&'static str>,
    error: &mut impl FnMut(String, SourceSpan),
) {
    let package = if import.wildcard {
        import.path.join(".")
    } else {
        import.path[..import.path.len() - 1].join(".")
    };

    if let Some(classes) = package_classes(&package) {
        if import.wildcard {
            for name in classes {
                if REQUIRES_IMPORT.contains(name) {
                    enabled.insert(name);
                }
            }
            return;
        }
        let class = import.path.last().expect("non-empty import path");
        if let Some(name) = classes.iter().find(|n| *n == class) {
            if REQUIRES_IMPORT.contains(name) {
                enabled.insert(name);
            }
            return;
        }
        if KNOWN_UNSUPPORTED
            .iter()
            .any(|(pkg, names)| *pkg == package && names.contains(&class.as_str()))
        {
            error(not_supported(&format!("{package}.{class}")), import.span);
            return;
        }
        // javac: "cannot find symbol — symbol: class X, location:
        // package java.util".
        error(
            format!("cannot find symbol: class {class} in package {package}"),
            import.span,
        );
        return;
    }

    if KNOWN_UNSUPPORTED_PACKAGES.contains(&package.as_str()) {
        error(not_supported(&format!("package {package}")), import.span);
    } else {
        // javac: "package foo.bar does not exist".
        error(format!("package {package} does not exist"), import.span);
    }
}

// ----- Use-site enforcement -----

struct UseCheck<'a, F: FnMut(String, SourceSpan)> {
    user_classes: &'a HashSet<String>,
    enabled: &'a HashSet<&'static str>,
    error: F,
}

fn check_class(
    class: &ClassDecl,
    user_classes: &HashSet<String>,
    enabled: &HashSet<&'static str>,
    error: &mut impl FnMut(String, SourceSpan),
) {
    let mut check = UseCheck {
        user_classes,
        enabled,
        error,
    };
    for field in &class.fields {
        check.type_ref(&field.ty, field.span);
        if let Some(init) = &field.init {
            check.expr(init);
        }
    }
    for method in &class.methods {
        check.type_ref(&method.return_type, method.span);
        for param in &method.params {
            check.type_ref(&param.ty, method.span);
        }
        for stmt in &method.body {
            check.stmt(stmt);
        }
    }
    for block in &class.init_blocks {
        for stmt in &block.body {
            check.stmt(stmt);
        }
    }
}

impl<F: FnMut(String, SourceSpan)> UseCheck<'_, F> {
    fn name(&mut self, name: &str, span: SourceSpan) {
        if name.contains('.') {
            // Fully qualified: no import needed; codegen validates it.
            return;
        }
        if REQUIRES_IMPORT.contains(&name)
            && !self.user_classes.contains(name)
            && !self.enabled.contains(name)
        {
            // javac: "cannot find symbol — symbol: class Scanner,
            // location: class Main".
            (self.error)(format!("cannot find symbol: class {name}"), span);
        }
    }

    fn type_ref(&mut self, ty: &TypeRef, span: SourceSpan) {
        match ty {
            TypeRef::Named(name) => self.name(name, span),
            TypeRef::Generic { base, args } => {
                self.name(base, span);
                for arg in args {
                    self.type_ref(arg, span);
                }
            }
            TypeRef::Array(inner) => self.type_ref(inner, span),
            _ => {}
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per statement kind
    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                for inner in statements {
                    self.stmt(inner);
                }
            }
            Stmt::Expr(expr) => self.expr(expr),
            Stmt::LocalDecl {
                ty,
                declarators,
                span,
                ..
            } => {
                self.type_ref(ty, *span);
                for declarator in declarators {
                    if let Some(init) = &declarator.init {
                        self.expr(init);
                    }
                }
            }
            Stmt::Assign { target, value, .. } => {
                match target {
                    crate::ast::AssignTarget::Var(_) => {}
                    crate::ast::AssignTarget::Index { array, index } => {
                        self.expr(array);
                        self.expr(index);
                    }
                    crate::ast::AssignTarget::Field { object, .. } => self.expr(object),
                }
                self.expr(value);
            }
            Stmt::ForEach {
                ty,
                iterable,
                body,
                span,
                ..
            } => {
                self.type_ref(ty, *span);
                self.expr(iterable);
                self.stmt(body);
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                self.expr(cond);
                self.stmt(then);
                if let Some(els) = els {
                    self.stmt(els);
                }
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.stmt(body);
            }
            Stmt::DoWhile { body, cond, .. } => {
                self.stmt(body);
                self.expr(cond);
            }
            Stmt::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                if let Some(init) = init {
                    self.stmt(init);
                }
                if let Some(cond) = cond {
                    self.expr(cond);
                }
                for stmt in update {
                    self.stmt(stmt);
                }
                self.stmt(body);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.expr(value);
                }
            }
            Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
                for arg in args {
                    self.expr(arg);
                }
            }
            Stmt::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                for stmt in body {
                    self.stmt(stmt);
                }
                for clause in catches {
                    self.type_ref(&clause.ty, clause.span);
                    for stmt in &clause.body {
                        self.stmt(stmt);
                    }
                }
                for stmt in finally_body.iter().flatten() {
                    self.stmt(stmt);
                }
            }
            Stmt::Throw { value, .. } => self.expr(value),
            Stmt::Switch { selector, arms, .. } => {
                self.expr(selector);
                for arm in arms {
                    for label in arm.labels.iter().flatten() {
                        self.expr(label);
                    }
                    for stmt in &arm.body {
                        self.stmt(stmt);
                    }
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::Labeled { body, .. } => self.stmt(body),
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } => {}
            Expr::Call { receiver, args, .. } => {
                if let Some(receiver) = receiver {
                    self.expr(receiver);
                }
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Unary { operand, .. } => self.expr(operand),
            Expr::Cast { ty, operand, span } => {
                self.type_ref(ty, *span);
                self.expr(operand);
            }
            Expr::Index { array, index, .. } => {
                self.expr(array);
                self.expr(index);
            }
            Expr::Field { object, .. } => self.expr(object),
            Expr::NewArray {
                elem, dims, init, ..
            } => {
                self.type_ref(elem, expr.span());
                for dim in dims.iter().flatten() {
                    self.expr(dim);
                }
                for element in init.iter().flatten() {
                    self.expr(element);
                }
            }
            Expr::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.expr(element);
                }
            }
            Expr::NewObject {
                class,
                type_args,
                args,
                span,
            } => {
                self.name(class, *span);
                for arg in type_args {
                    self.type_ref(arg, *span);
                }
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::InstanceOf { value, ty, span } => {
                self.expr(value);
                self.type_ref(ty, *span);
            }
            Expr::SuperMethodCall { args, .. } => {
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::Ternary {
                cond, then, els, ..
            } => {
                self.expr(cond);
                self.expr(then);
                self.expr(els);
            }
            Expr::IncDec { target, .. } => self.expr(target),
        }
    }
}
