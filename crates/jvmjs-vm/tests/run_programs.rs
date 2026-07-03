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
