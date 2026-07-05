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
use std::fmt::Write as _;

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
    /// Entry class for a validation ("Test") run — set when `JUnit` `@Test`
    /// methods were found and a synthetic runner was injected. Run this
    /// class's `main` instead of the student's to execute the tests.
    pub validation_entry: Option<String>,
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

/// Bundled `org.junit.jupiter.api.Assertions` subset, injected when a
/// source imports `org.junit` (the validation "Test" mode).
const JUNIT_LIB: &str = include_str!("stdlib/junit.java");

/// Bundled `java.util.Arrays` subset (`toString`), injected when a source
/// imports `java.util` — element concatenation coerces each via `toString`.
const ARRAYS_LIB: &str = include_str!("stdlib/arrays.java");

/// Bundled `org.code.validation` (the neighborhood test harness), injected
/// when a source imports it. Reads the action log recorded by `__NbhdWorld`.
const VALIDATION_LIB: &str = include_str!("stdlib/validation.java");

/// A discovered `@Test` method for the synthetic validation runner.
struct TestCase {
    method: String,
    display: String,
    order: i32,
}

/// A test class: its `@BeforeAll`/`@BeforeEach` setup methods and ordered
/// `@Test`s.
struct TestClass {
    name: String,
    before_all: Vec<String>,
    before_each: Vec<String>,
    tests: Vec<TestCase>,
}

/// Collect `JUnit` `@Test` methods (with `@Order`/`@DisplayName`/`@BeforeEach`)
/// from the user's classes.
fn collect_tests(units: &[(String, ast::CompilationUnit)]) -> Vec<TestClass> {
    let mut classes = Vec::new();
    for (_, unit) in units {
        for class in &unit.classes {
            let has = |m: &ast::MethodDecl, n: &str| m.annotations.iter().any(|a| a.name == n);
            let before_all: Vec<String> = class
                .methods
                .iter()
                .filter(|m| has(m, "BeforeAll"))
                .map(|m| m.name.clone())
                .collect();
            let before_each: Vec<String> = class
                .methods
                .iter()
                .filter(|m| has(m, "BeforeEach"))
                .map(|m| m.name.clone())
                .collect();
            let mut tests: Vec<TestCase> = class
                .methods
                .iter()
                .filter(|m| has(m, "Test"))
                .map(|m| {
                    let order = m
                        .annotations
                        .iter()
                        .find(|a| a.name == "Order")
                        .and_then(|a| a.int_arg)
                        .unwrap_or(i32::MAX);
                    let display = m
                        .annotations
                        .iter()
                        .find(|a| a.name == "DisplayName")
                        .and_then(|a| a.str_arg.clone())
                        .unwrap_or_else(|| m.name.clone());
                    TestCase {
                        method: m.name.clone(),
                        display,
                        order,
                    }
                })
                .collect();
            tests.sort_by_key(|t| t.order);
            if !tests.is_empty() {
                classes.push(TestClass {
                    name: class.name.clone(),
                    before_all,
                    before_each,
                    tests,
                });
            }
        }
    }
    classes
}

/// Generate a runner `main` that instantiates each test class, runs its
/// `@BeforeEach` setup and each `@Test`, and prints one
/// `__VTEST\t<PASS|FAIL>\t<name>\t<message>` line per test.
fn validation_runner_source(classes: &[TestClass]) -> String {
    let mut src = String::from(
        "public class __ValidationRunner {\n  public static void main(String[] args) {\n",
    );
    for class in classes {
        // `@BeforeAll` runs once (static); tolerate failure so per-test
        // errors are still reported individually.
        for setup in &class.before_all {
            src.push_str("    try {\n");
            let _ = writeln!(src, "      {}.{setup}();", class.name);
            src.push_str("    } catch (Throwable __e) {}\n");
        }
        for test in &class.tests {
            let name = test
                .display
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace(['\t', '\n', '\r'], " ");
            src.push_str("    try {\n");
            let _ = writeln!(src, "      {0} __t = new {0}();", class.name);
            for setup in &class.before_each {
                let _ = writeln!(src, "      __t.{setup}();");
            }
            let _ = writeln!(src, "      __t.{}();", test.method);
            let _ = writeln!(
                src,
                "      System.out.println(\"__VTEST\\tPASS\\t{name}\\t\");"
            );
            src.push_str("    } catch (Throwable __e) {\n");
            let _ = writeln!(
                src,
                "      System.out.println(\"__VTEST\\tFAIL\\t{name}\\t\" + __e.getMessage());"
            );
            src.push_str("    }\n");
        }
    }
    src.push_str("  }\n}\n");
    src
}

/// Whether any unit imports something under the given prefix path
/// (`["org","junit"]` matches `import static org.junit.jupiter....`).
fn imports_prefix(units: &[(String, ast::CompilationUnit)], prefix: &[&str]) -> bool {
    units.iter().any(|(_, unit)| {
        unit.imports.iter().any(|import| {
            import.path.len() >= prefix.len() && import.path[..prefix.len()] == *prefix
        })
    })
}

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
    // org.code.validation (neighborhood test harness): runs the student's main
    // and reports the recorded action log. Depends on __NbhdWorld above.
    if imports_package(&units, &["org", "code", "validation"]) {
        let (tokens, _) = lexer::lex("<validation-lib>", VALIDATION_LIB);
        let (unit, mut errs) = parser::parse("<validation-lib>", tokens);
        compilation.diagnostics.append(&mut errs);
        units.push((String::from("<validation-lib>"), unit));
    }
    if imports_package(&units, &["org", "code", "theater"])
        || imports_package(&units, &["org", "code", "media"])
    {
        let (tokens, _) = lexer::lex("<theater>", THEATER_LIB);
        let (unit, mut errs) = parser::parse("<theater>", tokens);
        compilation.diagnostics.append(&mut errs);
        units.push((String::from("<theater>"), unit));
    }
    let mut validation_entry = None;
    if sources.iter().any(|s| s.text.contains("Arrays."))
        && !units
            .iter()
            .any(|(_, unit)| unit.classes.iter().any(|c| c.name == "Arrays"))
    {
        let (tokens, _) = lexer::lex("<arrays>", ARRAYS_LIB);
        let (unit, mut errs) = parser::parse("<arrays>", tokens);
        compilation.diagnostics.append(&mut errs);
        units.push((String::from("<arrays>"), unit));
    }
    if imports_prefix(&units, &["org", "junit"]) {
        let (tokens, _) = lexer::lex("<junit>", JUNIT_LIB);
        let (unit, mut errs) = parser::parse("<junit>", tokens);
        compilation.diagnostics.append(&mut errs);
        units.push((String::from("<junit>"), unit));
        // Synthesize a runner that executes the discovered @Test methods.
        let tests = collect_tests(&units);
        if !tests.is_empty() {
            let source = validation_runner_source(&tests);
            let (tokens, _) = lexer::lex("<validation>", &source);
            let (unit, mut errs) = parser::parse("<validation>", tokens);
            compilation.diagnostics.append(&mut errs);
            units.push((String::from("<validation>"), unit));
            validation_entry = Some(String::from("__ValidationRunner"));
        }
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
        compilation.validation_entry = validation_entry;
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
    fn structural_reflection_compiles() {
        // The `AttributesHelper` surface: getClass / getSimpleName /
        // getSuperclass / getDeclaredFields + Arrays.toString(Field[]).
        let result = compile(&[
            SourceFile {
                path: String::from("Dog.java"),
                text: String::from("public class Dog { private String name; private int age; }"),
            },
            SourceFile {
                path: String::from("Main.java"),
                text: String::from(
                    r#"
                    import java.lang.reflect.*;
                    import java.util.*;
                    public class Main {
                        public static void main(String[] args) {
                            Object o = new Dog();
                            Class c = o.getClass();
                            String n = c.getSimpleName();
                            Class sup = c.getSuperclass();
                            Field[] fields = c.getDeclaredFields();
                            System.out.println(n + " extends " + sup.getSimpleName());
                            System.out.println(Arrays.toString(fields));
                        }
                    }
                    "#,
                ),
            },
        ]);
        assert!(result.success(), "{:?}", result.diagnostics);
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
