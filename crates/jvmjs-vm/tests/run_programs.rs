//! End-to-end VM tests.
//!
//! Two fixture strategies (see `specs/SCOPE.md` consequences):
//! - hand-assembled class files built with `jvmjs-classfile`'s writer,
//!   which pin the VM's behavior independently of our compiler, and
//! - compile-then-run tests through `jvmjs-compiler`, which pin the
//!   whole engine end to end.

use jvmjs_classfile::opcodes as op;
use jvmjs_classfile::{
    AttributeInfo, CODE_ATTRIBUTE, ClassFile, CodeAttribute, Constant, MethodAccessFlags,
    MethodInfo, write_code_attribute,
};
use jvmjs_vm::{BufferedConsole, ExitStatus, VirtualFileSystem, Vm, VmError, VmOptions};

/// Assemble a class equivalent to:
/// ```java
/// public class Fixture {
///     public static void main(String[] args) {
///         System.out.println("hand-assembled");
///     }
/// }
/// ```
fn hand_assembled_hello() -> ClassFile {
    let mut class = ClassFile::new_java11();
    let pool = &mut class.constant_pool;

    let fixture_name = pool.intern_utf8("Fixture");
    class.this_class = pool.intern(Constant::Class {
        name_index: fixture_name,
    });
    let object_name = pool.intern_utf8("java/lang/Object");
    class.super_class = pool.intern(Constant::Class {
        name_index: object_name,
    });

    let system_name = pool.intern_utf8("java/lang/System");
    let system_class = pool.intern(Constant::Class {
        name_index: system_name,
    });
    let out_name = pool.intern_utf8("out");
    let out_descriptor = pool.intern_utf8("Ljava/io/PrintStream;");
    let out_nat = pool.intern(Constant::NameAndType {
        name_index: out_name,
        descriptor_index: out_descriptor,
    });
    let out_field = pool.intern(Constant::FieldRef {
        class_index: system_class,
        name_and_type_index: out_nat,
    });

    let message_utf8 = pool.intern_utf8("hand-assembled");
    let message = pool.intern(Constant::String {
        string_index: message_utf8,
    });

    let stream_name = pool.intern_utf8("java/io/PrintStream");
    let stream_class = pool.intern(Constant::Class {
        name_index: stream_name,
    });
    let println_name = pool.intern_utf8("println");
    let println_descriptor = pool.intern_utf8("(Ljava/lang/String;)V");
    let println_nat = pool.intern(Constant::NameAndType {
        name_index: println_name,
        descriptor_index: println_descriptor,
    });
    let println_method = pool.intern(Constant::MethodRef {
        class_index: stream_class,
        name_and_type_index: println_nat,
    });

    let mut code = Vec::new();
    code.push(op::GETSTATIC);
    code.extend_from_slice(&out_field.to_be_bytes());
    code.push(op::LDC);
    code.push(u8::try_from(message).expect("small pool"));
    code.push(op::INVOKEVIRTUAL);
    code.extend_from_slice(&println_method.to_be_bytes());
    code.push(op::RETURN);

    let code_attribute = CodeAttribute {
        max_stack: 2,
        max_locals: 1,
        code,
        exception_table: Vec::new(),
        attributes: Vec::new(),
    };

    let main_name = pool.intern_utf8("main");
    let main_descriptor = pool.intern_utf8("([Ljava/lang/String;)V");
    let code_name = pool.intern_utf8(CODE_ATTRIBUTE);
    class.methods.push(MethodInfo {
        access_flags: MethodAccessFlags(MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
        name_index: main_name,
        descriptor_index: main_descriptor,
        attributes: vec![AttributeInfo {
            name_index: code_name,
            info: write_code_attribute(&code_attribute),
        }],
    });
    class
}

fn run_class(class: ClassFile, main: &str) -> (Result<ExitStatus, VmError>, BufferedConsole) {
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    vm.load_class(class).expect("class should load");
    let result = vm.run_main(main, &[]);
    (result, console)
}

fn compile_and_run(source: &str, main: &str) -> (Result<ExitStatus, VmError>, BufferedConsole) {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: format!("{main}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file)
            .expect("compiled class should load");
    }
    let result = vm.run_main(main, &[]);
    (result, console)
}

#[test]
fn runs_hand_assembled_hello_world() {
    let (result, console) = run_class(hand_assembled_hello(), "Fixture");
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "hand-assembled\n");
    assert_eq!(console.stderr_text(), "");
}

#[test]
fn hand_assembled_class_survives_binary_round_trip() {
    // The same fixture, but written to bytes and read back before
    // running — pinning writer + reader + interpreter together.
    let bytes = jvmjs_classfile::write_class_file(&hand_assembled_hello());
    let class = jvmjs_classfile::read_class_file(&bytes).expect("fixture should parse");
    let (result, console) = run_class(class, "Fixture");
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "hand-assembled\n");
}

#[test]
fn compiles_and_runs_hello_world() {
    let (result, console) = compile_and_run(
        r#"
        public class Main {
            public static void main(String[] args) {
                System.out.println("Hello, World!");
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "Hello, World!\n");
}

#[test]
fn compiles_and_runs_every_println_overload() {
    let (result, console) = compile_and_run(
        r#"
        public class Kitchen {
            public static void main(String[] args) {
                System.out.print("value: ");
                System.out.println(42);
                System.out.println(3.5);
                System.out.println(2.0);
                System.out.println(true);
                System.out.println('x');
                System.out.println();
                System.err.println("to stderr");
            }
        }
        "#,
        "Kitchen",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "value: 42\n3.5\n2.0\ntrue\nx\n\n");
    assert_eq!(console.stderr_text(), "to stderr\n");
}

#[test]
fn string_literals_are_interned_within_a_run() {
    // Same literal printed twice should hit the same heap object; this
    // is observable indirectly (no crash, correct output) and pins the
    // intern path.
    let (result, console) = compile_and_run(
        r#"
        class Twice {
            public static void main(String[] args) {
                System.out.println("same");
                System.out.println("same");
            }
        }
        "#,
        "Twice",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "same\nsame\n");
}

// ----- stage 1: locals, operators, string concatenation -----

/// Compile, run, and return stdout, asserting a clean completion.
fn run_stdout(source: &str, main: &str) -> String {
    let (result, console) = compile_and_run(source, main);
    assert!(
        matches!(result, Ok(ExitStatus::Completed)),
        "{result:?}; stderr: {}",
        console.stderr_text()
    );
    console.stdout_text()
}

#[test]
fn arithmetic_follows_java_semantics() {
    let out = run_stdout(
        r"
        public class Arith {
            public static void main(String[] args) {
                int a = 7, b = 2;
                System.out.println(a + b);
                System.out.println(a - b);
                System.out.println(a * b);
                System.out.println(a / b);
                System.out.println(a % b);
                System.out.println(-a);
                System.out.println(a / 2.0);
                System.out.println(1 + 2 * 3);
                System.out.println((1 + 2) * 3);
                System.out.println(2147483647 + 1);
            }
        }
        ",
        "Arith",
    );
    assert_eq!(out, "9\n5\n14\n3\n1\n-7\n3.5\n7\n9\n-2147483648\n");
}

#[test]
fn division_by_zero_throws_like_java() {
    let (result, console) = compile_and_run(
        r"
        public class Div {
            public static void main(String[] args) {
                int zero = 0;
                System.out.println(1 / zero);
            }
        }
        ",
        "Div",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.ArithmeticException: / by zero"),
        "{}",
        console.stderr_text()
    );
    // Double division by zero is Infinity, not an exception.
    let out = run_stdout(
        r"
        public class DDiv {
            public static void main(String[] args) {
                double zero = 0.0;
                System.out.println(1.0 / zero);
            }
        }
        ",
        "DDiv",
    );
    assert_eq!(out, "Infinity\n");
}

#[test]
fn comparisons_and_logic_short_circuit() {
    let out = run_stdout(
        r"
        public class Logic {
            public static void main(String[] args) {
                int x = 5;
                System.out.println(x > 3);
                System.out.println(x <= 4);
                System.out.println(x == 5 && x != 5);
                System.out.println(x == 5 || x != 5);
                System.out.println(!(x > 3));
                int zero = 0;
                System.out.println(zero != 0 && 10 / zero > 1);
                System.out.println(zero == 0 || 10 / zero > 1);
                System.out.println(1.5 > 1.4);
                System.out.println(0.0 / zero == 0.0 / zero);
            }
        }
        ",
        "Logic",
    );
    // The two short-circuit lines must not throw ArithmeticException,
    // and NaN == NaN is false.
    assert_eq!(
        out,
        "true\nfalse\nfalse\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\n"
    );
}

#[test]
fn assignment_compound_and_increment() {
    let out = run_stdout(
        r"
        public class Assign {
            public static void main(String[] args) {
                int x = 10;
                x += 5;
                x -= 3;
                x *= 2;
                x /= 4;
                x %= 4;
                System.out.println(x);
                x++;
                ++x;
                x--;
                System.out.println(x);
                double d = 1.5;
                d *= 2;
                System.out.println(d);
                int narrowed = 7;
                narrowed += 0.5;
                System.out.println(narrowed);
                char c = 'a';
                c++;
                System.out.println(c);
            }
        }
        ",
        "Assign",
    );
    // x: 10+5-3=12*2=24/4=6%4=2; then ++,++,-- -> 3.
    // narrowed: (int)(7 + 0.5) = 7. c: 'a'+1 = 'b'.
    assert_eq!(out, "2\n3\n3.0\n7\nb\n");
}

#[test]
fn string_concatenation_matches_java() {
    let out = run_stdout(
        r#"
        public class Concat {
            public static void main(String[] args) {
                int x = 42;
                double d = 2.0;
                boolean flag = true;
                char c = '!';
                String s = "x = " + x;
                System.out.println(s);
                System.out.println("d = " + d);
                System.out.println(flag + " story" + c);
                System.out.println(1 + 2 + "a");
                System.out.println("a" + 1 + 2);
                String nothing = null;
                System.out.println("value: " + nothing);
                s += "!";
                System.out.println(s);
            }
        }
        "#,
        "Concat",
    );
    assert_eq!(
        out,
        "x = 42\nd = 2.0\ntrue story!\n3a\na12\nvalue: null\nx = 42!\n"
    );
}

#[test]
fn casts_and_promotions() {
    let out = run_stdout(
        r"
        public class Casts {
            public static void main(String[] args) {
                double d = 9.99;
                System.out.println((int) d);
                System.out.println((int) -9.99);
                int i = 65;
                System.out.println((char) i);
                System.out.println((double) i);
                char c = 'A';
                int fromChar = c + 1;
                System.out.println(fromChar);
                System.out.println((char) (c + 1));
            }
        }
        ",
        "Casts",
    );
    assert_eq!(out, "9\n-9\nA\n65.0\n66\nB\n");
}

#[test]
fn string_reference_equality_of_literals() {
    // Literals are interned, so == is true for identical literals; a
    // runtime-built string is a different object.
    let out = run_stdout(
        r#"
        public class Eq {
            public static void main(String[] args) {
                String a = "hi";
                String b = "hi";
                System.out.println(a == b);
                String c = a + "!";
                String d = "hi!";
                System.out.println(c == d);
            }
        }
        "#,
        "Eq",
    );
    assert_eq!(out, "true\nfalse\n");
}

// ----- stage 2: control flow -----

#[test]
fn if_else_chains_and_dangling_else() {
    let out = run_stdout(
        r#"
        public class Grades {
            public static void main(String[] args) {
                int score = 85;
                if (score >= 90) System.out.println("A");
                else if (score >= 80) System.out.println("B");
                else if (score >= 70) System.out.println("C");
                else System.out.println("F");

                // Dangling else binds to the inner if.
                int x = 5;
                if (x > 0)
                    if (x > 10) System.out.println("big");
                    else System.out.println("small");
            }
        }
        "#,
        "Grades",
    );
    assert_eq!(out, "B\nsmall\n");
}

#[test]
fn while_and_do_while_semantics() {
    let out = run_stdout(
        r"
        public class Loops {
            public static void main(String[] args) {
                int n = 3;
                while (n > 0) {
                    System.out.println(n);
                    n--;
                }
                // The while body never runs; the do-while body runs once.
                while (false == true) System.out.println(-1);
                do System.out.println(99); while (false);
            }
        }
        ",
        "Loops",
    );
    assert_eq!(out, "3\n2\n1\n99\n");
}

#[test]
fn for_loops_sum_and_nest() {
    let out = run_stdout(
        r#"
        public class Sums {
            public static void main(String[] args) {
                int total = 0;
                for (int i = 1; i <= 10; i++) {
                    total += i;
                }
                System.out.println(total);

                String grid = "";
                for (int r = 0; r < 3; r++) {
                    for (int c = 0; c < 3; c++) {
                        grid += r * 3 + c;
                    }
                    grid += "|";
                }
                System.out.println(grid);

                // Two loops can reuse the same index name (loop scoping),
                // and multi-init/multi-update headers work.
                for (int i = 0, j = 3; i < j; i++, j--) {
                    System.out.println(i + "," + j);
                }
            }
        }
        "#,
        "Sums",
    );
    assert_eq!(out, "55\n012|345|678|\n0,3\n1,2\n");
}

#[test]
fn break_and_continue() {
    let out = run_stdout(
        r"
        public class Jumps {
            public static void main(String[] args) {
                for (int i = 0; i < 10; i++) {
                    if (i % 2 == 0) continue;
                    if (i > 6) break;
                    System.out.println(i);
                }
                // break/continue bind to the INNER loop.
                int found = 0;
                for (int i = 0; i < 3; i++) {
                    for (int j = 0; j < 3; j++) {
                        if (j > i) break;
                        found++;
                    }
                }
                System.out.println(found);
                int n = 0;
                while (true) {
                    n++;
                    if (n == 5) break;
                }
                System.out.println(n);
            }
        }
        ",
        "Jumps",
    );
    assert_eq!(out, "1\n3\n5\n6\n5\n");
}

#[test]
fn fizzbuzz_works() {
    let out = run_stdout(
        r#"
        public class FizzBuzz {
            public static void main(String[] args) {
                for (int i = 1; i <= 15; i++) {
                    if (i % 15 == 0) System.out.println("FizzBuzz");
                    else if (i % 3 == 0) System.out.println("Fizz");
                    else if (i % 5 == 0) System.out.println("Buzz");
                    else System.out.println(i);
                }
            }
        }
        "#,
        "FizzBuzz",
    );
    assert_eq!(
        out,
        "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n"
    );
}

#[test]
fn definite_assignment_across_branches() {
    // Both branches assign: OK.
    let ok = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "T.java".into(),
        text: r"
            class T {
                static void f() {
                    int x;
                    if (1 < 2) { x = 1; } else { x = 2; }
                    System.out.println(x);
                }
            }
        "
        .into(),
    }]);
    assert!(ok.success(), "{:?}", ok.diagnostics);

    // Only one branch assigns: javac-style error.
    let bad = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "T.java".into(),
        text: r"
            class T {
                static void f() {
                    int x;
                    if (1 < 2) { x = 1; }
                    System.out.println(x);
                }
            }
        "
        .into(),
    }]);
    assert!(!bad.success());
    assert!(
        bad.diagnostics[0]
            .message
            .contains("might not have been initialized"),
        "{:?}",
        bad.diagnostics
    );

    // Assignment only inside a loop body doesn't count either.
    let loopy = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "T.java".into(),
        text: r"
            class T {
                static void f() {
                    int x;
                    while (1 < 2) { x = 1; }
                    System.out.println(x);
                }
            }
        "
        .into(),
    }]);
    assert!(!loopy.success());
}

#[test]
fn infinite_loop_hits_the_instruction_budget() {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "Spin.java".into(),
        text: r"
            public class Spin {
                public static void main(String[] args) {
                    while (true) { }
                }
            }
        "
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(
        VmOptions {
            max_instructions: 10_000,
            ..VmOptions::default()
        },
        &mut vfs,
        &mut console,
    );
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Spin", &[]);
    assert!(
        matches!(result, Err(VmError::InstructionBudgetExceeded)),
        "{result:?}"
    );
}

// ----- stage 3: user-defined static methods -----

#[test]
fn calls_helpers_with_params_and_returns() {
    let out = run_stdout(
        r#"
        public class Temps {
            public static double toFahrenheit(double celsius) {
                return celsius * 9 / 5 + 32;
            }

            public static String describe(double f) {
                if (f >= 80) return "hot";
                if (f >= 60) return "mild";
                return "cold";
            }

            static void report(double celsius) {
                double f = toFahrenheit(celsius);
                System.out.println(celsius + "C = " + f + "F (" + describe(f) + ")");
            }

            public static void main(String[] args) {
                report(30.0);
                report(15);
                report(-5.0);
            }
        }
        "#,
        "Temps",
    );
    assert_eq!(
        out,
        "30.0C = 86.0F (hot)\n15.0C = 59.0F (cold)\n-5.0C = 23.0F (cold)\n"
    );
}

#[test]
fn recursion_and_mutual_recursion() {
    let out = run_stdout(
        r"
        public class Rec {
            static int factorial(int n) {
                if (n <= 1) return 1;
                return n * factorial(n - 1);
            }

            static int fib(int n) {
                if (n < 2) return n;
                return fib(n - 1) + fib(n - 2);
            }

            static boolean isEven(int n) {
                if (n == 0) return true;
                return isOdd(n - 1);
            }

            static boolean isOdd(int n) {
                if (n == 0) return false;
                return isEven(n - 1);
            }

            public static void main(String[] args) {
                System.out.println(factorial(10));
                System.out.println(fib(15));
                System.out.println(isEven(20));
                System.out.println(isOdd(20));
            }
        }
        ",
        "Rec",
    );
    assert_eq!(out, "3628800\n610\ntrue\nfalse\n");
}

#[test]
fn cross_class_static_calls_and_overloads() {
    let out = run_stdout(
        r#"
        class MathUtil {
            static int max(int a, int b) {
                if (a > b) return a;
                return b;
            }

            static double max(double a, double b) {
                if (a > b) return a;
                return b;
            }

            static String label(int v) { return "int:" + v; }
            static String label(double v) { return "double:" + v; }
        }

        public class Uses {
            public static void main(String[] args) {
                System.out.println(MathUtil.max(3, 9));
                System.out.println(MathUtil.max(2.5, 1.5));
                System.out.println(MathUtil.label(7));
                System.out.println(MathUtil.label(7.0));
                System.out.println(MathUtil.max(1, 2.5));
                char c = 'a';
                System.out.println(MathUtil.label(c));
            }
        }
        "#,
        "Uses",
    );
    // Exact overload wins for ints; widening picks the double overload
    // for max(1, 2.5); char widens to int so label('a') prints 97.
    assert_eq!(out, "9\n2.5\nint:7\ndouble:7.0\n2.5\nint:97\n");
}

#[test]
fn void_call_statements_and_value_discard() {
    let out = run_stdout(
        r#"
        public class Effects {
            static int counter() {
                System.out.println("counted");
                return 42;
            }

            static double wide() { return 1.5; }

            public static void main(String[] args) {
                // Results discarded (pop / pop2), side effects kept.
                counter();
                wide();
                System.out.println("done");
            }
        }
        "#,
        "Effects",
    );
    assert_eq!(out, "counted\ndone\n");
}

#[test]
fn runaway_recursion_overflows_like_java() {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "Deep.java".into(),
        text: r"
        public class Deep {
            static int down(int n) {
                return down(n + 1);
            }

            public static void main(String[] args) {
                System.out.println(down(0));
            }
        }
        "
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    // A small explicit limit keeps the test independent of the host
    // (debug-build) stack size; the default limit works the same way.
    let mut vm = Vm::new(
        VmOptions {
            max_call_depth: 64,
            ..VmOptions::default()
        },
        &mut vfs,
        &mut console,
    );
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Deep", &[]);
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.StackOverflowError"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn stage3_compile_errors_match_javac_wording() {
    let cases: &[(&str, &str)] = &[
        ("static int f() { int x = 1; }", "missing return statement"),
        ("static void f() { return 5; }", "unexpected return value"),
        ("static int f() { return; }", "missing return value"),
        (
            "static void f() { h(1); }",
            "cannot find symbol: method h(int)",
        ),
        (
            "static void f(int a, double b) { } static void f(double a, int b) { } \
             static void g() { f(1, 2); }",
            "reference to f is ambiguous",
        ),
        (
            "void inst() { } static void f() { inst(); }",
            "non-static method inst() cannot be referenced from a static context",
        ),
        (
            "static int f() { return 1; } static int f() { return 2; }",
            "method f() is already defined",
        ),
        (
            "static void g() { } static void f() { int x = g(); }",
            "returns void",
        ),
    ];
    for (body, expected) in cases {
        let result = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
            path: "T.java".into(),
            text: format!("class T {{ {body} }}"),
        }]);
        assert!(!result.success(), "case '{body}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{body}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn while_true_with_return_needs_no_trailing_return() {
    let out = run_stdout(
        r"
        public class Search {
            static int firstSquareAbove(int limit) {
                int n = 1;
                while (true) {
                    if (n * n > limit) return n * n;
                    n++;
                }
            }

            public static void main(String[] args) {
                System.out.println(firstSquareAbove(50));
            }
        }
        ",
        "Search",
    );
    assert_eq!(out, "64\n");
}

// ----- stage 4: arrays -----

#[test]
fn arrays_create_fill_and_read() {
    let out = run_stdout(
        r#"
        public class Arrays1 {
            public static void main(String[] args) {
                int[] squares = new int[5];
                for (int i = 0; i < squares.length; i++) {
                    squares[i] = i * i;
                }
                for (int i = 0; i < squares.length; i++) {
                    System.out.print(squares[i]);
                    System.out.print(" ");
                }
                System.out.println();

                double[] temps = {98.6, 99.1, 97.5};
                System.out.println(temps[1]);
                boolean[] flags = new boolean[2];
                System.out.println(flags[0]);
                char[] letters = {'j', 'v', 'm'};
                letters[0] = 'J';
                System.out.println("" + letters[0] + letters[1] + letters[2]);
                String[] names = {"Ada", "Grace"};
                System.out.println(names[0] + " & " + names[1]);
            }
        }
        "#,
        "Arrays1",
    );
    assert_eq!(out, "0 1 4 9 16 \n99.1\nfalse\nJvm\nAda & Grace\n");
}

#[test]
fn array_algorithms_and_for_each() {
    let out = run_stdout(
        r#"
        public class Algo {
            static int sum(int[] values) {
                int total = 0;
                for (int v : values) total += v;
                return total;
            }

            static int max(int[] values) {
                int best = values[0];
                for (int v : values) {
                    if (v > best) best = v;
                }
                return best;
            }

            static void reverse(int[] values) {
                for (int i = 0; i < values.length / 2; i++) {
                    int tmp = values[i];
                    values[i] = values[values.length - 1 - i];
                    values[values.length - 1 - i] = tmp;
                }
            }

            public static void main(String[] args) {
                int[] data = {3, 1, 4, 1, 5, 9, 2, 6};
                System.out.println(sum(data));
                System.out.println(max(data));
                reverse(data);
                String s = "";
                for (int v : data) s += v;
                System.out.println(s);
                data[0] += 10;
                data[1]++;
                System.out.println(data[0] + "," + data[1]);
            }
        }
        "#,
        "Algo",
    );
    assert_eq!(out, "31\n9\n62951413\n16,3\n");
}

#[test]
fn two_dimensional_arrays() {
    let out = run_stdout(
        r#"
        public class Grid {
            public static void main(String[] args) {
                int[][] m = new int[3][4];
                for (int r = 0; r < m.length; r++) {
                    for (int c = 0; c < m[r].length; c++) {
                        m[r][c] = r * 10 + c;
                    }
                }
                System.out.println(m.length + "x" + m[0].length);
                System.out.println(m[2][3]);

                int[][] jagged = {{1}, {2, 3}, {4, 5, 6}};
                int total = 0;
                for (int[] row : jagged) {
                    total += row.length;
                }
                System.out.println(total);
                System.out.println(jagged[2][1]);

                // Row objects are references: aliasing is visible.
                int[] alias = m[1];
                alias[0] = 99;
                System.out.println(m[1][0]);
            }
        }
        "#,
        "Grid",
    );
    assert_eq!(out, "3x4\n23\n6\n5\n99\n");
}

#[test]
fn array_exceptions_match_java_wording() {
    let (result, console) = compile_and_run(
        r"
        public class Oob {
            public static void main(String[] args) {
                int[] a = new int[3];
                int i = 5;
                System.out.println(a[i]);
            }
        }
        ",
        "Oob",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console.stderr_text().contains(
            "java.lang.ArrayIndexOutOfBoundsException: Index 5 out of bounds for length 3"
        ),
        "{}",
        console.stderr_text()
    );

    let (result, console) = compile_and_run(
        r"
        public class Neg {
            public static void main(String[] args) {
                int n = -2;
                int[] a = new int[n];
                System.out.println(a.length);
            }
        }
        ",
        "Neg",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.NegativeArraySizeException: -2"),
        "{}",
        console.stderr_text()
    );

    let (result, console) = compile_and_run(
        r"
        public class NullRow {
            public static void main(String[] args) {
                int[][] m = new int[2][];
                System.out.println(m[0][0]);
            }
        }
        ",
        "NullRow",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.NullPointerException"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn main_args_are_finally_usable() {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "Args.java".into(),
        text: r#"
        public class Args {
            public static void main(String[] args) {
                System.out.println(args.length);
                for (String arg : args) {
                    System.out.println("arg: " + arg);
                }
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Args", &["hello".into(), "world".into()]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "2\narg: hello\narg: world\n");
}

// ----- stage 5: objects -----

#[test]
fn classic_bank_account_class() {
    let out = run_stdout(
        r#"
        class Account {
            private String owner;
            private double balance;
            private static int accountCount = 0;

            public Account(String owner, double start) {
                this.owner = owner;
                balance = start;
                accountCount++;
            }

            public Account(String owner) {
                this.owner = owner;
                this.balance = 0.0;
                accountCount++;
            }

            public void deposit(double amount) {
                if (amount > 0) balance += amount;
            }

            public boolean withdraw(double amount) {
                if (amount <= balance) {
                    balance -= amount;
                    return true;
                }
                return false;
            }

            public double getBalance() { return balance; }
            public static int getCount() { return accountCount; }

            public String toString() {
                return owner + ": $" + balance;
            }
        }

        public class Bank {
            public static void main(String[] args) {
                Account a = new Account("Ada", 100.0);
                Account b = new Account("Grace");
                a.deposit(50.0);
                b.deposit(25.0);
                System.out.println(a.withdraw(200.0));
                System.out.println(a.withdraw(75.0));
                System.out.println(a);
                System.out.println(b);
                System.out.println("accounts: " + Account.getCount());
            }
        }
        "#,
        "Bank",
    );
    assert_eq!(out, "false\ntrue\nAda: $75.0\nGrace: $25.0\naccounts: 2\n");
}

#[test]
fn field_initializers_defaults_and_statics() {
    let out = run_stdout(
        r#"
        class Config {
            static int created = 0;
            static final double VERSION = 2.5;
            int id = next();
            String label;
            boolean enabled;

            static int next() {
                created++;
                return created;
            }
        }

        public class Defaults {
            public static void main(String[] args) {
                Config first = new Config();
                Config second = new Config();
                System.out.println(first.id + " " + second.id);
                System.out.println(second.label + " " + second.enabled);
                System.out.println(Config.VERSION);
                System.out.println(Config.created);
            }
        }
        "#,
        "Defaults",
    );
    assert_eq!(out, "1 2\nnull false\n2.5\n2\n");
}

#[test]
fn objects_are_references() {
    let out = run_stdout(
        r#"
        class Point {
            int x;
            int y;

            Point(int x, int y) {
                this.x = x;
                this.y = y;
            }

            void translate(int dx, int dy) {
                x += dx;
                y += dy;
            }

            public String toString() { return "(" + x + ", " + y + ")"; }
        }

        public class Refs {
            static void moveIt(Point p) {
                p.translate(10, 10);
            }

            public static void main(String[] args) {
                Point a = new Point(1, 2);
                Point alias = a;
                moveIt(a);
                System.out.println(alias);
                System.out.println(a == alias);
                System.out.println(a == new Point(11, 12));

                Point[] path = new Point[2];
                System.out.println(path[0]);
                path[0] = a;
                path[0].x = 99;
                System.out.println(a.x);
            }
        }
        "#,
        "Refs",
    );
    assert_eq!(out, "(11, 12)\ntrue\nfalse\nnull\n99\n");
}

#[test]
fn default_tostring_prints_class_at_hex() {
    let out = run_stdout(
        r#"
        class Ghost { }

        public class Spooky {
            public static void main(String[] args) {
                Ghost g = new Ghost();
                System.out.println(g);
                System.out.println("boo: " + g);
            }
        }
        "#,
        "Spooky",
    );
    let mut lines = out.lines();
    let first = lines.next().expect("one line");
    assert!(first.starts_with("Ghost@"), "{first}");
    let second = lines.next().expect("two lines");
    assert!(second.starts_with("boo: Ghost@"), "{second}");
}

#[test]
fn npe_on_null_receiver_and_field() {
    let (result, console) = compile_and_run(
        r"
        class Widget {
            int size;
            void grow() { size++; }
        }

        public class Broken {
            public static void main(String[] args) {
                Widget w = null;
                w.grow();
            }
        }
        ",
        "Broken",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.NullPointerException"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn stage5_compile_errors_match_javac_wording() {
    let cases: &[(&str, &str)] = &[
        (
            "class A { private int x; } class B { static void f() { A a = new A(); \
             System.out.println(a.x); } }",
            "x has private access in A",
        ),
        (
            "class A { private void m() { } } class B { static void f() { A a = new A(); \
             a.m(); } }",
            "m() has private access in A",
        ),
        (
            "class A { int x; static void f() { System.out.println(x); } }",
            "non-static variable x cannot be referenced from a static context",
        ),
        (
            "class A { int x; static void f() { System.out.println(this.x); } }",
            "non-static variable this cannot be referenced from a static context",
        ),
        (
            "class Pt { Pt(int x) { } } class B { static void f() { Pt p = new Pt(); } }",
            "constructor Pt in class Pt cannot be applied to given types",
        ),
        (
            "class A { int x; int x; }",
            "variable x is already defined in class A",
        ),
        (
            "class A { A(int v) { } A(int w) { } }",
            "constructor A is already defined",
        ),
        (
            "class A { final int x = 1; void f() { x = 2; } }",
            "cannot assign a value to final variable x",
        ),
    ];
    for (source, expected) in cases {
        let result = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
            path: "T.java".into(),
            text: (*source).into(),
        }]);
        assert!(!result.success(), "case '{source}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{source}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

// ----- stage 6: inheritance -----

#[test]
fn classic_shape_hierarchy_with_polymorphism() {
    let out = run_stdout(
        r#"
        abstract class Shape {
            private String name;

            Shape(String name) {
                this.name = name;
            }

            abstract double area();

            String getName() { return name; }

            public String toString() {
                return name + " with area " + area();
            }
        }

        class Circle extends Shape {
            private double radius;

            Circle(double radius) {
                super("circle");
                this.radius = radius;
            }

            double area() { return 3.14159 * radius * radius; }
        }

        class Rect extends Shape {
            private double w;
            private double h;

            Rect(double w, double h) {
                super("rect");
                this.w = w;
                this.h = h;
            }

            Rect(double side) {
                this(side, side);
            }

            double area() { return w * h; }
        }

        public class Shapes {
            public static void main(String[] args) {
                Shape[] shapes = new Shape[3];
                shapes[0] = new Circle(1.0);
                shapes[1] = new Rect(2.0, 3.0);
                shapes[2] = new Rect(2.0);
                double total = 0.0;
                for (Shape s : shapes) {
                    System.out.println(s);
                    total += s.area();
                }
                System.out.println(total);
                System.out.println(shapes[1].getName());
            }
        }
        "#,
        "Shapes",
    );
    assert_eq!(
        out,
        "circle with area 3.14159\nrect with area 6.0\nrect with area 4.0\n13.14159\nrect\n"
    );
}

#[test]
fn super_method_calls_and_overriding() {
    let out = run_stdout(
        r#"
        class Animal {
            String speak() { return "..."; }
            String describe() { return "an animal says " + speak(); }
        }

        class Dog extends Animal {
            String speak() { return "woof"; }
            String describe() {
                return super.describe() + " (a dog: " + super.speak() + "/" + speak() + ")";
            }
        }

        public class Zoo {
            public static void main(String[] args) {
                Animal a = new Dog();
                // Dynamic dispatch inside inherited methods too.
                System.out.println(a.describe());
                Animal plain = new Animal();
                System.out.println(plain.describe());
            }
        }
        "#,
        "Zoo",
    );
    assert_eq!(
        out,
        "an animal says woof (a dog: .../woof)\nan animal says ...\n"
    );
}

#[test]
fn interfaces_and_instanceof_and_casts() {
    let out = run_stdout(
        r#"
        interface Playable {
            String play();
        }

        class Guitar implements Playable {
            public String play() { return "strum"; }
        }

        class Drum implements Playable {
            public String play() { return "boom"; }
            int hits() { return 2; }
        }

        public class Band {
            public static void main(String[] args) {
                Playable[] band = { new Guitar(), new Drum() };
                for (Playable p : band) {
                    System.out.print(p.play() + " ");
                }
                System.out.println();
                for (Playable p : band) {
                    System.out.println(p instanceof Drum);
                    if (p instanceof Drum) {
                        Drum d = (Drum) p;
                        System.out.println("hits: " + d.hits());
                    }
                }
                Playable nothing = null;
                System.out.println(nothing instanceof Guitar);
            }
        }
        "#,
        "Band",
    );
    assert_eq!(out, "strum boom \nfalse\ntrue\nhits: 2\nfalse\n");
}

#[test]
fn class_cast_exception_matches_java() {
    let (result, console) = compile_and_run(
        r"
        class A { }
        class B extends A { }

        public class Bad {
            public static void main(String[] args) {
                A a = new A();
                B b = (B) a;
                System.out.println(b);
            }
        }
        ",
        "Bad",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.ClassCastException: class A cannot be cast to class B"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn inherited_fields_and_super_chaining() {
    let out = run_stdout(
        r#"
        class Base {
            int base = 10;
            static String log = "";

            Base() {
                log += "B";
            }

            Base(int extra) {
                this();
                log += "b" + extra;
            }
        }

        class Derived extends Base {
            int derived = 20;

            Derived() {
                super(5);
                log += "D";
            }

            int total() { return base + derived; }
        }

        public class Chain {
            public static void main(String[] args) {
                Derived d = new Derived();
                System.out.println(Base.log);
                System.out.println(d.total());
                System.out.println(d.base + " " + d.derived);
            }
        }
        "#,
        "Chain",
    );
    assert_eq!(out, "Bb5D\n30\n10 20\n");
}

#[test]
fn stage6_compile_errors_match_javac_wording() {
    let cases: &[(&str, &str)] = &[
        (
            "class A { String f() { return \"a\"; } } class B extends A { int f() { return 1; } }",
            "f() in B cannot override f() in A",
        ),
        (
            "abstract class A { abstract void go(); } class M { static void f() { A a = new A(); } }",
            "A is abstract; cannot be instantiated",
        ),
        (
            "abstract class A { abstract void go(); } class B extends A { }",
            "B is not abstract and does not override abstract method go() in A",
        ),
        (
            "interface S { double area(); } class C implements S { }",
            "C is not abstract and does not override abstract method area() in S",
        ),
        (
            "class A { A(int x) { } } class B extends A { B() { } }",
            "constructor A in class A cannot be applied to given types",
        ),
        (
            "class A { int x; } class B extends A { int x; }",
            "hiding the inherited field 'x' from A is not supported",
        ),
        (
            "class A { void f() { int x = 1; super(); } }",
            "call to super/this must be the first statement in a constructor",
        ),
    ];
    for (source, expected) in cases {
        let result = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
            path: "T.java".into(),
            text: (*source).into(),
        }]);
        assert!(!result.success(), "case '{source}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{source}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

// ----- stage 7: the class library -----

#[test]
fn string_methods_match_java() {
    let out = run_stdout(
        r#"
        public class Strings {
            public static void main(String[] args) {
                String s = "Hello, World";
                System.out.println(s.length());
                System.out.println(s.charAt(4));
                System.out.println(s.substring(7));
                System.out.println(s.substring(0, 5));
                System.out.println(s.indexOf("World"));
                System.out.println(s.indexOf("xyz"));
                System.out.println(s.equals("Hello, World"));
                System.out.println(s.equals("hello, world"));
                System.out.println(s.equalsIgnoreCase("HELLO, WORLD"));
                System.out.println("Ab".compareTo("b"));
                System.out.println("apple".compareTo("apple"));
                System.out.println(s.toUpperCase());
                System.out.println(s.toLowerCase());
                System.out.println("  pad  ".trim());
                System.out.println(s.contains("lo, W"));
                System.out.println(s.startsWith("Hell") + " " + s.endsWith("!"));
                System.out.println("".isEmpty());
            }
        }
        "#,
        "Strings",
    );
    assert_eq!(
        out,
        "12\no\nWorld\nHello\n7\n-1\ntrue\nfalse\ntrue\n-33\n0\nHELLO, WORLD\nhello, world\npad\ntrue\ntrue false\ntrue\n"
    );
}

#[test]
fn math_and_wrappers() {
    let out = run_stdout(
        r#"
        public class Maths {
            public static void main(String[] args) {
                System.out.println(Math.abs(-7));
                System.out.println(Math.abs(-2.5));
                System.out.println(Math.pow(2.0, 10.0));
                System.out.println(Math.sqrt(144.0));
                System.out.println(Math.max(3, 9) + " " + Math.min(3, 9));
                System.out.println(Math.max(1.5, 2.5));
                double r = Math.random();
                System.out.println(r >= 0.0 && r < 1.0);
                System.out.println(Integer.MAX_VALUE);
                System.out.println(Integer.MIN_VALUE);
                System.out.println(Integer.parseInt("-42") + 1);
                System.out.println(Double.parseDouble("2.5") * 2);
            }
        }
        "#,
        "Maths",
    );
    assert_eq!(
        out,
        "7\n2.5\n1024.0\n12.0\n9 3\n2.5\ntrue\n2147483647\n-2147483648\n-41\n5.0\n"
    );
}

#[test]
fn scanner_reads_tokens_and_lines() {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "In.java".into(),
        text: r#"
        import java.util.Scanner;
        public class In {
            public static void main(String[] args) {
                Scanner in = new Scanner(System.in);
                int n = in.nextInt();
                double d = in.nextDouble();
                String word = in.next();
                in.nextLine();
                String line = in.nextLine();
                System.out.println(n * 2);
                System.out.println(d + 0.5);
                System.out.println(word.toUpperCase());
                System.out.println("[" + line + "]");
                System.out.println(in.hasNextInt());
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(["21 3.5 hey trailing", "a whole line"]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("In", &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "42\n4.0\nHEY\n[a whole line]\nfalse\n"
    );
}

#[test]
fn array_list_basics_and_for_each() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        public class Lists {
            public static void main(String[] args) {
                ArrayList<Integer> nums = new ArrayList<Integer>();
                nums.add(10);
                nums.add(20);
                nums.add(1, 15);
                System.out.println(nums.size());
                System.out.println(nums.get(1));
                nums.set(0, 5);
                System.out.println(nums);
                int removed = nums.remove(2);
                System.out.println(removed + " -> " + nums);
                int total = 0;
                for (int n : nums) total += n;
                System.out.println(total);

                ArrayList<String> names = new ArrayList<>();
                names.add("Ada");
                names.add("Grace");
                System.out.println(names);
                for (String name : names) System.out.println(name.toUpperCase());

                ArrayList<Double> temps = new ArrayList<Double>();
                temps.add(98.6);
                System.out.println(temps.get(0) + 1.0);
                System.out.println(temps.isEmpty());
            }
        }
        "#,
        "Lists",
    );
    assert_eq!(
        out,
        "3\n15\n[5, 15, 20]\n20 -> [5, 15]\n20\n[Ada, Grace]\nADA\nGRACE\n99.6\nfalse\n"
    );
}

#[test]
fn array_list_of_objects() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        class Song {
            private String title;
            Song(String title) { this.title = title; }
            String getTitle() { return title; }
        }

        public class Playlist {
            public static void main(String[] args) {
                ArrayList<Song> queue = new ArrayList<Song>();
                queue.add(new Song("Daisy"));
                queue.add(new Song("Bicycle"));
                for (Song s : queue) {
                    System.out.println(s.getTitle());
                }
                Song first = queue.get(0);
                System.out.println(first.getTitle().length());
            }
        }
        "#,
        "Playlist",
    );
    assert_eq!(out, "Daisy\nBicycle\n5\n");
}

#[test]
fn classlib_exceptions_match_java_wording() {
    let cases: &[(&str, &str)] = &[
        (
            r#"String s = "abc"; System.out.println(s.charAt(5));"#,
            "java.lang.StringIndexOutOfBoundsException: String index out of range: 5",
        ),
        (
            r#"String s = "abc"; System.out.println(s.substring(1, 9));"#,
            "java.lang.StringIndexOutOfBoundsException: begin 1, end 9, length 3",
        ),
        (
            r#"System.out.println(Integer.parseInt("x1"));"#,
            "java.lang.NumberFormatException: For input string: \"x1\"",
        ),
    ];
    for (body, expected) in cases {
        let (result, console) = compile_and_run(
            &format!("public class T {{ public static void main(String[] args) {{ {body} }} }}"),
            "T",
        );
        assert!(
            matches!(result, Err(VmError::UncaughtException(_))),
            "case '{body}': {result:?}"
        );
        assert!(
            console.stderr_text().contains(expected),
            "case '{body}': expected '{expected}' in {}",
            console.stderr_text()
        );
    }

    // ArrayList index errors.
    let (result, console) = compile_and_run(
        r"
        import java.util.ArrayList;
        public class L {
            public static void main(String[] args) {
                ArrayList<Integer> list = new ArrayList<Integer>();
                list.add(7);
                System.out.println(list.get(3));
            }
        }
        ",
        "L",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.IndexOutOfBoundsException: Index 3 out of bounds for length 1"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn math_random_is_deterministic_per_seed() {
    let source = r"
        public class R {
            public static void main(String[] args) {
                System.out.println(Math.random());
                System.out.println((int) (Math.random() * 6) + 1);
            }
        }
    ";
    let run = |seed: Option<u64>| {
        let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
            path: "R.java".into(),
            text: source.into(),
        }]);
        assert!(compilation.success());
        let mut vfs = VirtualFileSystem::new();
        let mut console = BufferedConsole::new();
        let mut vm = Vm::new(
            VmOptions {
                random_seed: seed,
                ..VmOptions::default()
            },
            &mut vfs,
            &mut console,
        );
        for class in compilation.classes {
            vm.load_class(class.class_file).unwrap();
        }
        vm.run_main("R", &[]).unwrap();
        console.stdout_text()
    };
    // Same seed → same output; different seed → different draw.
    assert_eq!(run(Some(7)), run(Some(7)));
    assert_ne!(run(Some(7)), run(Some(8)));
    // All draws in range.
    for line in run(Some(99)).lines() {
        let value: f64 = line.parse().unwrap();
        assert!((0.0..7.0).contains(&value), "{value}");
    }
}

// ----- stage 8: file IO over the virtual filesystem -----

#[test]
fn print_writer_writes_and_scanner_reads_back() {
    let out = run_stdout(
        r#"
        import java.io.File;
        import java.io.PrintWriter;
        import java.util.Scanner;

        public class RoundTrip {
            public static void main(String[] args) throws Exception {
                PrintWriter out = new PrintWriter("scores.txt");
                out.println("Ada 95");
                out.println("Alan 88");
                out.print(2);
                out.close();

                File f = new File("scores.txt");
                System.out.println(f.exists() + " " + f.getName());

                Scanner in = new Scanner(f);
                while (in.hasNextLine()) {
                    System.out.println("> " + in.nextLine());
                }
            }
        }
        "#,
        "RoundTrip",
    );
    assert_eq!(out, "true scores.txt\n> Ada 95\n> Alan 88\n> 2\n");
}

#[test]
fn scanner_file_tokenizing_and_file_ops() {
    let out = run_stdout(
        r#"
        import java.io.File;
        import java.io.PrintWriter;
        import java.util.Scanner;

        public class Grades {
            public static void main(String[] args) throws Exception {
                PrintWriter w = new PrintWriter(new File("nums.txt"));
                w.println("10 20 30");
                w.println("2.5");
                w.close();

                Scanner in = new Scanner(new File("nums.txt"));
                int total = 0;
                for (int i = 0; i < 3; i++) total += in.nextInt();
                double scale = in.nextDouble();
                System.out.println(total * scale);

                File f = new File("nums.txt");
                System.out.println(f.length() > 0);
                System.out.println(f.isFile() + " " + f.isDirectory());
                System.out.println(f.delete());
                System.out.println(f.exists());
                System.out.println(f.delete());

                File dir = new File("logs");
                System.out.println(dir.mkdir());
                System.out.println(dir.isDirectory());
                File fresh = new File("logs/new.txt");
                System.out.println(fresh.createNewFile() + " " + fresh.length());
            }
        }
        "#,
        "Grades",
    );
    assert_eq!(
        out,
        "150.0\ntrue\ntrue false\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue 0\n"
    );
}

#[test]
fn missing_file_throws_like_java() {
    let (result, console) = compile_and_run(
        r#"
        import java.io.File;
        import java.util.Scanner;

        public class Missing {
            public static void main(String[] args) throws Exception {
                Scanner in = new Scanner(new File("ghost.txt"));
                System.out.println(in.nextLine());
            }
        }
        "#,
        "Missing",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.io.FileNotFoundException: ghost.txt (No such file or directory)"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn vfs_seeded_files_are_visible_to_java() {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "Reader.java".into(),
        text: r#"
        import java.io.File;
        import java.util.Scanner;

        public class Reader {
            public static void main(String[] args) throws Exception {
                Scanner in = new Scanner(new File("/data/input.txt"));
                int total = 0;
                while (in.hasNextInt()) total += in.nextInt();
                System.out.println(total);
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    vfs.write_file("/data/input.txt", "5 10 15 20".as_bytes().to_vec())
        .unwrap();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Reader", &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "50\n");
    // And output written by Java is visible to the host afterwards.
    assert!(vfs.exists("/data/input.txt"));
}

#[test]
fn missing_main_is_reported() {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "NoMain.java".into(),
        text: "class NoMain { static void other() { } }".into(),
    }]);
    assert!(compilation.success());
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("NoMain", &[]);
    assert!(
        matches!(result, Err(VmError::NoMainMethod(_))),
        "{result:?}"
    );
}

#[test]
fn instruction_budget_stops_runaway_bytecode() {
    // A method whose bytecode never reaches `return`: goto 0 would be
    // ideal but isn't implemented yet, so use a tiny budget instead.
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(
        VmOptions {
            max_instructions: 2,
            ..VmOptions::default()
        },
        &mut vfs,
        &mut console,
    );
    vm.load_class(hand_assembled_hello()).unwrap();
    let result = vm.run_main("Fixture", &[]);
    assert!(
        matches!(result, Err(VmError::InstructionBudgetExceeded)),
        "{result:?}"
    );
}
