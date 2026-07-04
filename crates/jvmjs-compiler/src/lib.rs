//! The `javac`-equivalent half of jvmjs: compiles Java 11 source (the
//! AP Computer Science A subset) to JVM class files.
//!
//! Pipeline: [`lexer`] → [`parser`] (AST) → [`codegen`]
//! ([`jvmjs_classfile::ClassFile`]). The supported language surface and
//! its staging plan live in `specs/LANGUAGE.md`; constructs beyond the
//! current stage produce friendly "not yet supported" diagnostics.

pub mod ast;
mod capture;
pub mod codegen;
pub mod diagnostics;
mod imports;
mod lambda;
pub mod lexer;
pub mod parser;

use std::collections::HashMap;

use jvmjs_classfile::ClassFile;

pub use diagnostics::{Diagnostic, Severity, SourcePosition, SourceSpan};

/// One Java source file presented to the compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Path of the file, e.g. `Main.java` or `com/example/Main.java`.
    pub path: String,
    /// Full source text.
    pub text: String,
}

/// A compiled class, ready to be loaded by the VM or written to the
/// virtual filesystem as a `.class` file.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledClass {
    /// Binary class name, e.g. `com/example/Main`.
    pub binary_name: String,
    pub class_file: ClassFile,
}

/// The result of compiling a set of source files. Compilation is
/// all-or-nothing: if any error-severity diagnostic is present,
/// `classes` is empty.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Compilation {
    pub classes: Vec<CompiledClass>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Compilation {
    /// Whether compilation succeeded (no error-severity diagnostics).
    #[must_use]
    pub fn success(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|d| d.severity != Severity::Error)
    }
}

/// Bundled clean-room implementation of `org.code.neighborhood`,
/// injected when a source imports that package.
const NEIGHBORHOOD_LIB: &str = include_str!("stdlib/neighborhood.java");

/// Bundled clean-room implementation of `org.code.theater` and
/// `org.code.media`, injected when either package is imported.
const THEATER_LIB: &str = include_str!("stdlib/theater.java");

/// Whether any unit imports a class from the given package path
/// (matches both `import pkg.*` and `import pkg.Class`).
fn imports_package(units: &[(String, ast::CompilationUnit)], package: &[&str]) -> bool {
    units.iter().any(|(_, unit)| {
        unit.imports.iter().any(|import| {
            let prefix = if import.wildcard {
                import.path.as_slice()
            } else {
                &import.path[..import.path.len().saturating_sub(1)]
            };
            prefix == package
        })
    })
}

/// Compile a set of Java source files. All files are parsed first so
/// classes can call each other's static methods regardless of file
/// order.
#[must_use]
pub fn compile(sources: &[SourceFile]) -> Compilation {
    let mut compilation = Compilation::default();
    let mut seen: HashMap<String, String> = HashMap::new();
    let mut units = Vec::new();

    for source in sources {
        let (tokens, mut lex_errors) = lexer::lex(&source.path, &source.text);
        compilation.diagnostics.append(&mut lex_errors);

        let (unit, mut parse_errors) = parser::parse(&source.path, tokens);
        compilation.diagnostics.append(&mut parse_errors);

        for class in &unit.classes {
            if let Some(other_path) = seen.get(&class.name) {
                compilation.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: format!("class '{}' is already defined in {other_path}", class.name),
                    path: source.path.clone(),
                    span: Some(class.span),
                });
            } else {
                seen.insert(class.name.clone(), source.path.clone());
            }
        }
        units.push((source.path.clone(), unit));
    }

    // Auto-inject bundled library sources when their package is
    // imported (e.g. the Code.org neighborhood Painter). Parsed like
    // any other unit so its classes participate in resolution.
    if imports_package(&units, &["org", "code", "neighborhood"]) {
        let (tokens, _) = lexer::lex("<neighborhood>", NEIGHBORHOOD_LIB);
        let (unit, mut errs) = parser::parse("<neighborhood>", tokens);
        compilation.diagnostics.append(&mut errs);
        units.push((String::from("<neighborhood>"), unit));
    }
    if imports_package(&units, &["org", "code", "theater"])
        || imports_package(&units, &["org", "code", "media"])
    {
        let (tokens, _) = lexer::lex("<theater>", THEATER_LIB);
        let (unit, mut errs) = parser::parse("<theater>", tokens);
        compilation.diagnostics.append(&mut errs);
        units.push((String::from("<theater>"), unit));
    }

    // Import validation and enforcement (after all units parse, since
    // user classes anywhere in the compilation shadow library names).
    let user_classes: std::collections::HashSet<String> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter().map(|c| c.name.clone()))
        .collect();
    for (path, unit) in &units {
        imports::check_unit(path, unit, &user_classes, &mut compilation.diagnostics);
    }

    lambda::desugar_lambdas(&mut units);
    capture::resolve_captures(&mut units);
    let (classes, mut codegen_errors) = codegen::generate(&units);
    compilation.diagnostics.append(&mut codegen_errors);

    if compilation.success() {
        compilation.classes = classes;
    }
    compilation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexical_errors_are_reported_with_positions() {
        let result = compile(&[SourceFile {
            path: String::from("Main.java"),
            text: String::from("class Main { char c = 'x; }"),
        }]);
        assert!(!result.success());
        assert!(result.classes.is_empty());
        let error = &result.diagnostics[0];
        assert_eq!(error.path, "Main.java");
        assert!(
            error.message.contains("character literal"),
            "{}",
            error.message
        );
    }

    #[test]
    fn hello_world_compiles_to_a_class() {
        let result = compile(&[SourceFile {
            path: String::from("Main.java"),
            text: String::from(
                r#"
                public class Main {
                    public static void main(String[] args) {
                        System.out.println("Hello, World!");
                    }
                }
                "#,
            ),
        }]);
        assert!(result.success(), "{:?}", result.diagnostics);
        assert_eq!(result.classes.len(), 1);
        assert_eq!(result.classes[0].binary_name, "Main");
        // The class file is structurally valid: it round-trips.
        let bytes = jvmjs_classfile::write_class_file(&result.classes[0].class_file);
        let parsed = jvmjs_classfile::read_class_file(&bytes).expect("valid class file");
        assert_eq!(parsed.class_name(), Some("Main"));
    }

    #[test]
    fn duplicate_classes_across_files_are_reported() {
        let result = compile(&[
            SourceFile {
                path: "A.java".into(),
                text: "class Main { }".into(),
            },
            SourceFile {
                path: "B.java".into(),
                text: "class Main { }".into(),
            },
        ]);
        assert!(!result.success());
        assert!(
            result.diagnostics[0]
                .message
                .contains("already defined in A.java")
        );
    }

    #[test]
    fn unsupported_features_fail_with_friendly_messages() {
        let result = compile(&[SourceFile {
            path: String::from("Main.java"),
            text: String::from("class Main { static void run() { synchronized (x) { } } }"),
        }]);
        assert!(!result.success());
        assert!(result.classes.is_empty());
        assert!(result.diagnostics[0].message.contains("not yet supported"));
    }
}
