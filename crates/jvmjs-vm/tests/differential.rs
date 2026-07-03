//! Differential tests: run the same program through the installed JDK
//! (`javac` + `java`) and through the jvmjs engine, and require
//! identical stdout. This pins our semantics to the reference
//! implementation rather than to our own expectations.
//!
//! Skips (with a note) when no JDK is installed, so CI without Java
//! still passes; locally, `apt install openjdk-11-jdk` enables it.

use std::io::Write;
use std::process::Command;

use jvmjs_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

fn jdk_available() -> bool {
    Command::new("javac")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
        && Command::new("java")
            .arg("--version")
            .output()
            .is_ok_and(|out| out.status.success())
}

/// Run `source` through javac+java, returning stdout.
fn run_with_jdk(class_name: &str, source: &str) -> String {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("differential-{class_name}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let java_file = dir.join(format!("{class_name}.java"));
    let mut file = std::fs::File::create(&java_file).expect("create source file");
    file.write_all(source.as_bytes()).expect("write source");
    drop(file);

    let compile = Command::new("javac")
        .arg(java_file.file_name().expect("file name"))
        .current_dir(&dir)
        .output()
        .expect("javac runs");
    assert!(
        compile.status.success(),
        "javac rejected {class_name}: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new("java")
        .arg(class_name)
        .current_dir(&dir)
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "java failed for {class_name}: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).into_owned()
}

/// Run `source` through the jvmjs engine, returning stdout.
fn run_with_jvmjs(class_name: &str, source: &str) -> String {
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: format!("{class_name}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "jvmjs rejected {class_name}: {:?}",
        compilation.diagnostics
    );

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("class loads");
    }
    let result = vm.run_main(class_name, &[]);
    assert!(
        result.is_ok(),
        "jvmjs run failed for {class_name}: {result:?}; stderr: {}",
        console.stderr_text()
    );
    console.stdout_text()
}

fn assert_same_output(class_name: &str, source: &str) {
    let expected = run_with_jdk(class_name, source);
    let actual = run_with_jvmjs(class_name, source);
    assert_eq!(
        actual, expected,
        "output diverges from the reference JDK for {class_name}"
    );
}

macro_rules! differential_test {
    ($name:ident, $class:literal, $source:literal) => {
        #[test]
        fn $name() {
            if !jdk_available() {
                eprintln!("skipping: no JDK on PATH");
                return;
            }
            assert_same_output($class, $source);
        }
    };
}

differential_test!(
    diff_arithmetic_and_formatting,
    "DiffArith",
    r"
public class DiffArith {
    public static void main(String[] args) {
        System.out.println(7 / 2);
        System.out.println(7 % 2);
        System.out.println(-7 / 2);
        System.out.println(-7 % 2);
        System.out.println(7 / 2.0);
        System.out.println(2147483647 + 1);
        System.out.println(-2147483648);
        System.out.println(2.0);
        System.out.println(0.1 + 0.2);
        System.out.println(10000000.0);
        System.out.println(1.0 / 0.0);
        System.out.println(-1.0 / 0.0);
        System.out.println(0.0 / 0.0);
        System.out.println(1.5 % 0.5);
        System.out.println(-0.0);
    }
}
"
);

differential_test!(
    diff_comparisons_and_logic,
    "DiffLogic",
    r"
public class DiffLogic {
    public static void main(String[] args) {
        double nan = 0.0 / 0.0;
        System.out.println(nan == nan);
        System.out.println(nan != nan);
        System.out.println(nan < 1.0);
        System.out.println(nan >= 1.0);
        int zero = 0;
        System.out.println(zero != 0 && 1 / zero > 0);
        System.out.println(zero == 0 || 1 / zero > 0);
        System.out.println(!(1 < 2));
        System.out.println('a' < 'b');
    }
}
"
);

differential_test!(
    diff_string_concat,
    "DiffConcat",
    r#"
public class DiffConcat {
    public static void main(String[] args) {
        int x = 42;
        System.out.println("x = " + x);
        System.out.println(1 + 2 + "a");
        System.out.println("a" + 1 + 2);
        System.out.println("d: " + 2.0 + ", c: " + 'q' + ", b: " + true);
        String s = null;
        System.out.println("null? " + s);
        String t = "ab";
        t += 'c';
        t += 5;
        t += 1.5;
        System.out.println(t);
        System.out.println("" + 'a' + 'b');
    }
}
"#
);

differential_test!(
    diff_casts_and_char_arithmetic,
    "DiffCasts",
    r"
public class DiffCasts {
    public static void main(String[] args) {
        System.out.println((int) 9.99);
        System.out.println((int) -9.99);
        System.out.println((char) 66);
        System.out.println((double) 5);
        System.out.println('a' + 1);
        System.out.println((char) ('a' + 1));
        char c = 'z';
        c++;
        System.out.println(c);
        int big = 70000;
        System.out.println((int) (char) big);
        double d = 1e10;
        System.out.println((int) d);
    }
}
"
);

differential_test!(
    diff_control_flow,
    "DiffFlow",
    r#"
public class DiffFlow {
    public static void main(String[] args) {
        for (int i = 1; i <= 15; i++) {
            if (i % 15 == 0) System.out.println("FizzBuzz");
            else if (i % 3 == 0) System.out.println("Fizz");
            else if (i % 5 == 0) System.out.println("Buzz");
            else System.out.println(i);
        }
        int n = 3;
        do { System.out.println(n); n--; } while (n > 0);
        for (int i = 0; i < 10; i++) {
            if (i % 2 == 0) continue;
            if (i > 6) break;
            System.out.println(i);
        }
    }
}
"#
);

differential_test!(
    diff_methods_and_recursion,
    "DiffMethods",
    r#"
public class DiffMethods {
    static int factorial(int n) {
        if (n <= 1) return 1;
        return n * factorial(n - 1);
    }

    static double average(int a, int b) {
        return (a + b) / 2.0;
    }

    static String shout(String s) { return s + "!"; }

    static void tick() { System.out.println("tick"); }

    public static void main(String[] args) {
        System.out.println(factorial(12));
        System.out.println(average(3, 4));
        System.out.println(shout(shout("hey")));
        tick();
    }
}
"#
);

differential_test!(
    diff_overload_resolution,
    "DiffOverload",
    r#"
public class DiffOverload {
    static String f(int v) { return "int:" + v; }
    static String f(double v) { return "double:" + v; }

    public static void main(String[] args) {
        System.out.println(f(1));
        System.out.println(f(1.0));
        System.out.println(f('a'));
        char c = 'b';
        System.out.println(f(c));
        System.out.println(f(1 + 1));
        System.out.println(f(1 / 2));
        System.out.println(f(1 / 2.0));
    }
}
"#
);

differential_test!(
    diff_arrays,
    "DiffArrays",
    r#"
public class DiffArrays {
    static int sum(int[] values) {
        int total = 0;
        for (int v : values) total += v;
        return total;
    }

    public static void main(String[] args) {
        int[] data = {5, 3, 8, 1};
        System.out.println(data.length);
        System.out.println(sum(data));
        data[2] += 10;
        data[0]++;
        System.out.println(data[2] + " " + data[0]);

        double[] d = new double[3];
        d[1] = 2.5;
        System.out.println(d[0] + " " + d[1]);

        boolean[] b = new boolean[2];
        System.out.println(b[1]);

        char[] c = {'h', 'i'};
        System.out.println("" + c[0] + c[1]);

        String[] s = new String[2];
        System.out.println(s[0]);
        s[0] = "set";
        System.out.println(s[0]);
    }
}
"#
);

differential_test!(
    diff_2d_arrays,
    "Diff2D",
    r#"
public class Diff2D {
    public static void main(String[] args) {
        int[][] m = new int[3][4];
        for (int r = 0; r < m.length; r++) {
            for (int c = 0; c < m[r].length; c++) {
                m[r][c] = r * m[r].length + c;
            }
        }
        int total = 0;
        for (int[] row : m) {
            for (int v : row) total += v;
        }
        System.out.println(total);
        System.out.println(m.length + " " + m[0].length);

        int[][] jag = {{1}, {2, 3}, {4, 5, 6}};
        System.out.println(jag[2].length);
        System.out.println(jag[1][1]);

        int[] alias = m[2];
        alias[0] = -1;
        System.out.println(m[2][0]);
    }
}
"#
);

differential_test!(
    diff_objects,
    "DiffObjects",
    r#"
class Counter {
    private int value;
    private final int step;
    static int instances = 0;

    Counter(int step) {
        this.step = step;
        instances++;
    }

    void tick() { value += step; }
    int get() { return value; }

    public String toString() { return "Counter(" + value + " by " + step + ")"; }
}

public class DiffObjects {
    public static void main(String[] args) {
        Counter a = new Counter(3);
        Counter b = new Counter(5);
        for (int i = 0; i < 4; i++) {
            a.tick();
            b.tick();
        }
        System.out.println(a.get() + " " + b.get());
        System.out.println(a);
        System.out.println(Counter.instances);
        Counter alias = a;
        alias.tick();
        System.out.println(a.get());
        System.out.println(a == alias);
        System.out.println(a == b);
        Counter none = null;
        System.out.println("gone: " + none);
    }
}
"#
);

differential_test!(
    diff_object_arrays_and_fields,
    "DiffObjArr",
    r#"
class Pt {
    int x;
    int y;
    Pt(int x, int y) { this.x = x; this.y = y; }
    public String toString() { return "(" + x + "," + y + ")"; }
}

public class DiffObjArr {
    static int taxicab(Pt a, Pt b) {
        int dx = a.x - b.x;
        int dy = a.y - b.y;
        if (dx < 0) dx = -dx;
        if (dy < 0) dy = -dy;
        return dx + dy;
    }

    public static void main(String[] args) {
        Pt[] path = new Pt[3];
        path[0] = new Pt(0, 0);
        path[1] = new Pt(3, 4);
        path[2] = new Pt(-2, 1);
        int total = 0;
        for (int i = 1; i < path.length; i++) {
            total += taxicab(path[i - 1], path[i]);
        }
        System.out.println(total);
        System.out.println(path[2]);
        path[1].x += 10;
        System.out.println(path[1]);
        System.out.println(path[0] == path[1]);
    }
}
"#
);

differential_test!(
    diff_compound_assignment_narrowing,
    "DiffCompound",
    r"
public class DiffCompound {
    public static void main(String[] args) {
        int x = 7;
        x += 0.9;
        System.out.println(x);
        x *= 2.5;
        System.out.println(x);
        char c = 'a';
        c += 2;
        System.out.println(c);
        double d = 10;
        d /= 4;
        System.out.println(d);
        int wrap = 2000000000;
        wrap += wrap;
        System.out.println(wrap);
    }
}
"
);
