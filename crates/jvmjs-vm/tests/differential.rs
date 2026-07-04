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
    run_with_jdk_stdin(class_name, source, "")
}

/// Run through the JDK with piped standard input.
fn run_with_jdk_stdin(class_name: &str, source: &str, stdin: &str) -> String {
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

    let mut child = Command::new("java")
        .arg(class_name)
        .current_dir(&dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("java starts");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(stdin.as_bytes())
        .expect("write stdin");
    let run = child.wait_with_output().expect("java runs");
    assert!(
        run.status.success(),
        "java failed for {class_name}: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).into_owned()
}

/// Run `source` through the jvmjs engine, returning stdout.
fn run_with_jvmjs(class_name: &str, source: &str) -> String {
    run_with_jvmjs_stdin(class_name, source, "")
}

/// Run through jvmjs with scripted standard input.
fn run_with_jvmjs_stdin(class_name: &str, source: &str, stdin: &str) -> String {
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
    let mut console = BufferedConsole::with_input(stdin.lines().map(str::to_owned));
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

fn assert_same_output_with_stdin(class_name: &str, source: &str, stdin: &str) {
    let expected = run_with_jdk_stdin(class_name, source, stdin);
    let actual = run_with_jvmjs_stdin(class_name, source, stdin);
    assert_eq!(
        actual, expected,
        "output diverges from the reference JDK for {class_name} (with stdin)"
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
    diff_inheritance,
    "DiffInherit",
    r#"
abstract class Vehicle {
    private String kind;
    protected int wheels;
    static String log = "";

    Vehicle(String kind, int wheels) {
        this.kind = kind;
        this.wheels = wheels;
        log += wheels;
    }

    abstract double speed();

    public String toString() {
        return kind + "/" + wheels + "/" + speed();
    }
}

class Bike extends Vehicle {
    Bike() {
        super("bike", 2);
    }

    double speed() { return 15.5; }
}

class Car extends Vehicle {
    private double top;

    Car(double top) {
        super("car", 4);
        this.top = top;
    }

    Car() {
        this(120.0);
    }

    double speed() { return top; }
}

public class DiffInherit {
    public static void main(String[] args) {
        Vehicle[] fleet = { new Bike(), new Car(200.0), new Car() };
        for (Vehicle v : fleet) {
            System.out.println(v);
        }
        System.out.println(Vehicle.log);
        for (Vehicle v : fleet) {
            System.out.println(v instanceof Car);
        }
        Vehicle first = fleet[1];
        if (first instanceof Car) {
            Car c = (Car) first;
            System.out.println("cast ok: " + c.speed());
        }
    }
}
"#
);

differential_test!(
    diff_interfaces_and_super,
    "DiffIface",
    r#"
interface Greeter {
    String greet(String name);
}

class Plain implements Greeter {
    public String greet(String name) { return "hi " + name; }
}

class Loud extends Plain {
    public String greet(String name) {
        return super.greet(name) + "!!!";
    }
}

public class DiffIface {
    static String greetAll(Greeter g) {
        return g.greet("ada") + " / " + g.greet("alan");
    }

    public static void main(String[] args) {
        Greeter p = new Plain();
        Greeter l = new Loud();
        System.out.println(greetAll(p));
        System.out.println(greetAll(l));
        System.out.println(l instanceof Plain);
        System.out.println(p instanceof Loud);
    }
}
"#
);

differential_test!(
    diff_string_methods,
    "DiffStrings",
    r#"
public class DiffStrings {
    public static void main(String[] args) {
        String s = "Concatenate ALL the strings";
        System.out.println(s.length());
        System.out.println(s.charAt(0));
        System.out.println(s.substring(12));
        System.out.println(s.substring(0, 11));
        System.out.println(s.indexOf("ALL"));
        System.out.println(s.indexOf("all"));
        System.out.println(s.toUpperCase());
        System.out.println(s.toLowerCase());
        System.out.println("a".compareTo("b"));
        System.out.println("b".compareTo("a"));
        System.out.println("same".compareTo("same"));
        System.out.println("ab".compareTo("abc"));
        System.out.println(s.equals("nope"));
        System.out.println(s.contains("ALL"));
        System.out.println("  trimmed 	".trim());
        String built = "";
        for (int i = 0; i < 3; i++) {
            built += s.charAt(i);
        }
        System.out.println(built);
    }
}
"#
);

differential_test!(
    diff_math_and_wrappers,
    "DiffMath",
    r#"
public class DiffMath {
    public static void main(String[] args) {
        System.out.println(Math.abs(-9));
        System.out.println(Math.abs(-9.75));
        System.out.println(Math.pow(3.0, 4.0));
        System.out.println(Math.sqrt(2.0));
        System.out.println(Math.max(-1, 1) + " " + Math.min(-1, 1));
        System.out.println(Math.max(0.5, 0.25));
        System.out.println(Integer.MAX_VALUE);
        System.out.println(Integer.MIN_VALUE);
        System.out.println(Integer.parseInt("123") + Integer.parseInt("-23"));
        System.out.println(Double.parseDouble("1.5e2"));
    }
}
"#
);

differential_test!(
    diff_array_list,
    "DiffList",
    r#"
import java.util.ArrayList;

public class DiffList {
    public static void main(String[] args) {
        ArrayList<Integer> nums = new ArrayList<Integer>();
        for (int i = 1; i <= 5; i++) {
            nums.add(i * i);
        }
        nums.add(2, 99);
        System.out.println(nums);
        System.out.println(nums.size());
        System.out.println(nums.get(2));
        System.out.println(nums.set(0, -1));
        System.out.println(nums.remove(3));
        System.out.println(nums);
        int total = 0;
        for (int n : nums) total += n;
        System.out.println(total);

        ArrayList<String> words = new ArrayList<>();
        words.add("delta");
        words.add(0, "alpha");
        System.out.println(words);
        String all = "";
        for (String w : words) all += w.substring(0, 1);
        System.out.println(all);
    }
}
"#
);

#[test]
fn diff_scanner_stdin() {
    if !jdk_available() {
        eprintln!("skipping: no JDK on PATH");
        return;
    }
    assert_same_output_with_stdin(
        "DiffScan",
        r#"
import java.util.Scanner;

public class DiffScan {
    public static void main(String[] args) {
        Scanner in = new Scanner(System.in);
        int count = in.nextInt();
        double scale = in.nextDouble();
        double total = 0;
        for (int i = 0; i < count; i++) {
            total += in.nextInt() * scale;
        }
        in.nextLine();
        String label = in.nextLine();
        System.out.println(label.trim() + ": " + total);
        System.out.println(in.hasNextInt());
        String tail = in.next();
        System.out.println(tail.toUpperCase());
    }
}
"#,
        "3 0.5\n10 20 30\n  weighted sum  \nfinal-token\n",
    );
}

differential_test!(
    diff_file_io,
    "DiffFiles",
    r#"
import java.io.File;
import java.io.PrintWriter;
import java.util.Scanner;

public class DiffFiles {
    public static void main(String[] args) throws Exception {
        PrintWriter out = new PrintWriter("diff-io.txt");
        for (int i = 1; i <= 3; i++) {
            out.println("line " + i + " = " + (i * 1.5));
        }
        out.print("end");
        out.close();

        File f = new File("diff-io.txt");
        System.out.println(f.exists());
        System.out.println(f.getName());

        Scanner in = new Scanner(f);
        while (in.hasNextLine()) {
            System.out.println("[" + in.nextLine() + "]");
        }

        Scanner tokens = new Scanner(new File("diff-io.txt"));
        System.out.println(tokens.next());
        System.out.println(tokens.next());
        System.out.println(tokens.next());
        System.out.println(tokens.next());

        System.out.println(f.delete());
        System.out.println(f.exists());
    }
}
"#
);

differential_test!(
    diff_deep_recursion,
    "DiffDeep",
    r"
public class DiffDeep {
    static int sum(int n) {
        if (n == 0) {
            return 0;
        }
        return n + sum(n - 1);
    }

    static int fib(int n) {
        if (n < 2) {
            return n;
        }
        return fib(n - 1) + fib(n - 2);
    }

    public static void main(String[] args) {
        System.out.println(sum(3000));
        System.out.println(fib(20));
    }
}
"
);

/// `javap -l` is the reference reader for debug attributes: if it
/// prints our `LineNumberTable` / `LocalVariableTable` / `SourceFile`, the
/// encoding is real.
#[test]
fn javap_accepts_debug_attributes() {
    if !jdk_available() {
        eprintln!("skipping: no JDK on PATH");
        return;
    }
    let compilation = jvmjs_compiler::compile(&[jvmjs_compiler::SourceFile {
        path: "Probe.java".into(),
        text: r#"
public class Probe {
    static int triple(int x) {
        int result = x * 3;
        return result;
    }

    public static void main(String[] args) {
        int a = 2;
        double b = 1.5;
        String label = "hi";
        System.out.println(triple(a) + b + label);
    }
}
"#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let dir = std::env::temp_dir().join(format!("jvmjs-javap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    for class in compilation.classes {
        let bytes = jvmjs_classfile::write_class_file(&class.class_file);
        std::fs::write(dir.join(format!("{}.class", class.binary_name)), bytes)
            .expect("write class");
    }
    let output = Command::new("javap")
        .args(["-l", "Probe.class"])
        .current_dir(&dir)
        .output()
        .expect("javap runs");
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(output.status.success(), "javap rejected the class: {text}");
    assert!(text.contains("LineNumberTable"), "{text}");
    assert!(text.contains("LocalVariableTable"), "{text}");
    // Locals are named, with the declared slots.
    assert!(text.contains("result"), "{text}");
    assert!(text.contains("label"), "{text}");
    assert!(text.contains("args"), "{text}");
}

differential_test!(
    diff_wildcard_imports,
    "DiffWild",
    r#"
import java.util.*;
import java.io.*;

public class DiffWild {
    public static void main(String[] args) throws IOException {
        ArrayList<Integer> nums = new ArrayList<>();
        for (int i = 0; i < 4; i++) nums.add(i * 3);
        PrintWriter out = new PrintWriter("wild.txt");
        for (int n : nums) out.println(n);
        out.close();
        Scanner in = new Scanner(new File("wild.txt"));
        int total = 0;
        while (in.hasNextInt()) total += in.nextInt();
        System.out.println(total);
    }
}
"#
);

differential_test!(
    diff_fully_qualified_names,
    "DiffFq",
    r#"
public class DiffFq {
    public static void main(String[] args) throws java.io.IOException {
        java.util.ArrayList<java.lang.Integer> nums = new java.util.ArrayList<Integer>();
        nums.add(java.lang.Math.max(3, 8));
        nums.add(java.lang.Integer.parseInt("34"));
        java.io.PrintWriter out = new java.io.PrintWriter("fq.txt");
        for (int n : nums) out.println(n * 1.5);
        out.close();
        java.util.Scanner in = new java.util.Scanner(new java.io.File("fq.txt"));
        double total = 0;
        while (in.hasNextDouble()) total += in.nextDouble();
        java.lang.System.out.println(total);
        java.lang.String s = "qualified";
        System.out.println(s.substring(0, 4).toUpperCase());
    }
}
"#
);

differential_test!(
    diff_classlib_extras,
    "DiffExtras",
    r#"
public class DiffExtras {
    public static void main(String[] args) {
        String csv = "ada,alan,,grace,";
        String[] names = csv.split(",");
        System.out.println(names.length);
        for (String name : names) {
            System.out.println("[" + name + "]");
        }
        System.out.println("banana".replace('a', 'o'));
        System.out.println("mississippi".replace("ss", "SS"));
        System.out.println(Math.floor(-2.5) + " " + Math.ceil(-2.5));
        System.out.println(Math.round(2.5) + " " + Math.round(-2.5) + " " + Math.round(2.4));
        System.out.println(Math.PI);
        System.out.println(Math.E);
        System.out.println(Integer.toString(-42) + Double.toString(1.5));
        System.out.println(Character.isDigit('7'));
        System.out.println(Character.isLetter('7'));
        System.out.println(Character.isUpperCase('a') + " " + Character.isLowerCase('a'));
        System.out.println(Character.toUpperCase('q'));
        System.out.println(Character.toLowerCase('Q'));
        String word = "hello world";
        String[] parts = word.split(" ");
        System.out.println(parts[0].toUpperCase() + "!" + parts[1]);
    }
}
"#
);

differential_test!(
    diff_try_catch,
    "DiffCatch",
    r#"
import java.util.Scanner;

public class DiffCatch {
    static int parse(String text) {
        try {
            return Integer.parseInt(text);
        } catch (NumberFormatException e) {
            System.out.println("bad: " + e.getMessage());
            return -1;
        }
    }

    static double divide(int a, int b) {
        try {
            return a / b;
        } catch (ArithmeticException e) {
            System.out.println(e);
            throw new IllegalArgumentException("b must be nonzero");
        }
    }

    public static void main(String[] args) {
        System.out.println(parse("42"));
        System.out.println(parse("x42"));
        try {
            int[] grades = new int[3];
            grades[3] = 100;
        } catch (IndexOutOfBoundsException e) {
            System.out.println("bounds: " + e.getMessage());
        }
        try {
            System.out.println(divide(10, 2));
            System.out.println(divide(10, 0));
        } catch (IllegalArgumentException e) {
            System.out.println("rejected: " + e.getMessage());
        }
        try {
            String s = null;
            System.out.println(s.length());
        } catch (RuntimeException e) {
            System.out.println("runtime caught");
        }
        System.out.println("done");
    }
}
"#
);

differential_test!(
    diff_finally,
    "DiffFinally",
    r#"
public class DiffFinally {
    static String log = "";

    static int viaReturn(int n) {
        try {
            return 100 / n;
        } catch (ArithmeticException e) {
            log += "[caught " + e.getMessage() + "]";
            return -1;
        } finally {
            log += "[fin]";
        }
    }

    public static void main(String[] args) {
        int a = viaReturn(4);
        int b = viaReturn(0);
        System.out.println(a + " " + b);
        System.out.println(log);

        for (int i = 0; i < 4; i++) {
            try {
                if (i % 2 == 0) {
                    continue;
                }
                if (i == 3) {
                    break;
                }
                System.out.println("body " + i);
            } finally {
                System.out.println("cleanup " + i);
            }
        }

        try {
            try {
                throw new IllegalStateException("boom");
            } finally {
                System.out.println("inner cleanup");
            }
        } catch (IllegalStateException e) {
            System.out.println("outer: " + e.getMessage());
        }
        System.out.println("end");
    }
}
"#
);

differential_test!(
    diff_user_exceptions,
    "DiffUserExc",
    r#"
class InsufficientFundsException extends Exception {
    private double shortfall;

    InsufficientFundsException(String message, double shortfall) {
        super(message);
        this.shortfall = shortfall;
    }

    double getShortfall() { return shortfall; }
}

class Account {
    private double balance;

    Account(double balance) { this.balance = balance; }

    void withdraw(double amount) throws InsufficientFundsException {
        if (amount > balance) {
            throw new InsufficientFundsException(
                "cannot withdraw " + amount, amount - balance);
        }
        balance -= amount;
        System.out.println("balance now " + balance);
    }
}

public class DiffUserExc {
    public static void main(String[] args) {
        Account account = new Account(100.0);
        try {
            account.withdraw(30.0);
            account.withdraw(90.0);
            account.withdraw(10.0);
        } catch (InsufficientFundsException e) {
            System.out.println("declined: " + e.getMessage());
            System.out.println("short by " + e.getShortfall());
        } finally {
            System.out.println("audited");
        }

        try {
            try {
                throw new InsufficientFundsException("nested", 1.0);
            } catch (Exception e) {
                System.out.println("rethrowing " + e.getMessage());
                throw e;
            }
        } catch (Exception e) {
            System.out.println("outer " + e.getMessage());
        }
        System.out.println("end");
    }
}
"#
);

differential_test!(
    diff_string_full_api,
    "DiffStrFull",
    r#"
public class DiffStrFull {
    public static void main(String[] args) {
        String s = "The Quick Brown Fox";

        System.out.println(s.strip() + "|" + "  pad  ".strip() + "|");
        System.out.println("  pad  ".stripLeading() + "|");
        System.out.println("  pad  ".stripTrailing() + "|");
        System.out.println("".isBlank() + " " + "  ".isBlank() + " " + "x".isBlank());
        System.out.println("ab".repeat(3) + " " + "x".repeat(0) + "|");
        System.out.println("foo".concat("bar"));
        System.out.println("HELLO".compareToIgnoreCase("hello"));
        System.out.println("apple".compareToIgnoreCase("Banana"));
        System.out.println(s.contentEquals("The Quick Brown Fox"));
        System.out.println("hello".hashCode() + " " + "".hashCode() + " " + s.hashCode());

        System.out.println(s.indexOf('o') + " " + s.indexOf('o', 13) + " " + s.indexOf('z'));
        System.out.println(s.indexOf("o", 5) + " " + s.indexOf("Quick", 99));
        System.out.println(s.lastIndexOf('o') + " " + s.lastIndexOf('o', 12));
        System.out.println(s.lastIndexOf("o") + " " + s.lastIndexOf("The") + " " + s.lastIndexOf("zzz"));
        System.out.println("".indexOf("") + " " + "abc".indexOf("", 2) + " " + "abc".lastIndexOf(""));

        String csv = "a,b,,c,,";
        String[] p1 = csv.split(",", 3);
        System.out.println(p1.length + " [" + p1[2] + "]");
        String[] p2 = csv.split(",", -1);
        System.out.println(p2.length);
        String[] p3 = csv.split(",", 1);
        System.out.println(p3.length + " [" + p3[0] + "]");

        System.out.println(s.startsWith("Quick", 4) + " " + s.startsWith("Quick", 5));
        System.out.println(s.subSequence(4, 9));

        char[] chars = "wave".toCharArray();
        chars[0] = 'c';
        System.out.println(String.valueOf(chars) + " " + String.copyValueOf(chars));
        char[] target = new char[10];
        "abcdef".getChars(1, 4, target, 2);
        System.out.println(String.valueOf(target[2]) + target[3] + target[4]);

        System.out.println(s.toString() == s);
        String i1 = ("Qu" + "ick").intern();
        System.out.println(i1 == "Quick");

        System.out.println(String.valueOf(42) + String.valueOf(2.5) + String.valueOf('x') + String.valueOf(true));

        System.out.println("cafe".codePointAt(1) + " " + "cafe".codePointBefore(2));
        System.out.println("cafe".codePointCount(0, 4) + " " + "cafe".offsetByCodePoints(0, 3));
    }
}
"#
);

differential_test!(
    diff_math_full_api,
    "DiffMathFull",
    r#"
public class DiffMathFull {
    public static void main(String[] args) {
        // Exact-friendly inputs: transcendental implementations may
        // differ by 1 ulp between libms on irrational results.
        System.out.println(Math.sin(0.0) + " " + Math.cos(0.0) + " " + Math.tan(0.0));
        System.out.println(Math.asin(0.0) + " " + Math.acos(1.0) + " " + Math.atan(0.0));
        System.out.println(Math.atan2(0.0, 1.0) + " " + Math.atan2(1.0, 0.0));
        System.out.println(Math.sinh(0.0) + " " + Math.cosh(0.0) + " " + Math.tanh(0.0));
        System.out.println(Math.exp(0.0) + " " + Math.expm1(0.0));
        System.out.println(Math.log(1.0) + " " + Math.log10(100.0) + " " + Math.log1p(0.0));
        System.out.println(Math.cbrt(27.0) + " " + Math.cbrt(-8.0));
        System.out.println(Math.hypot(3.0, 4.0));
        System.out.println(Math.rint(2.5) + " " + Math.rint(3.5) + " " + Math.rint(-2.5));
        System.out.println(Math.signum(-7.5) + " " + Math.signum(0.0) + " " + Math.signum(3.0));
        System.out.println(Math.toDegrees(Math.PI) + " " + Math.toRadians(180.0) / Math.PI);
        System.out.println(Math.copySign(3.0, -1.0) + " " + Math.ulp(1.0));
        System.out.println(Math.nextUp(1.0) + " " + Math.nextDown(1.0) + " " + Math.nextAfter(1.0, 2.0));
        System.out.println(Math.fma(2.0, 3.0, 4.0));
        System.out.println(Math.IEEEremainder(5.0, 3.0) + " " + Math.IEEEremainder(-5.0, 3.0));
        System.out.println(Math.getExponent(8.0) + " " + Math.getExponent(1.0) + " " + Math.getExponent(0.5));
        System.out.println(Math.floorDiv(7, 2) + " " + Math.floorDiv(-7, 2) + " " + Math.floorDiv(7, -2));
        System.out.println(Math.floorMod(7, 2) + " " + Math.floorMod(-7, 2) + " " + Math.floorMod(7, -2));
        System.out.println(Math.addExact(1, 2) + " " + Math.multiplyExact(6, 7));
        try {
            Math.addExact(Integer.MAX_VALUE, 1);
        } catch (ArithmeticException e) {
            System.out.println("overflow: " + e.getMessage());
        }
        try {
            Math.negateExact(Integer.MIN_VALUE);
        } catch (ArithmeticException e) {
            System.out.println("negate: " + e.getMessage());
        }
    }
}
"#
);

differential_test!(
    diff_wrappers_full_api,
    "DiffWrapFull",
    r#"
public class DiffWrapFull {
    public static void main(String[] args) {
        System.out.println(Integer.parseInt("ff", 16) + " " + Integer.parseInt("-101", 2));
        System.out.println(Integer.toString(255, 16) + " " + Integer.toString(-42, 2));
        System.out.println(Integer.toBinaryString(10) + " " + Integer.toOctalString(64) + " " + Integer.toHexString(-1));
        System.out.println(Integer.valueOf(7) + Integer.valueOf("35"));
        System.out.println(Integer.compare(3, 9) + " " + Integer.compare(9, 3) + " " + Integer.compare(4, 4));
        System.out.println(Integer.max(3, 9) + " " + Integer.min(3, 9) + " " + Integer.sum(3, 9));
        System.out.println(Integer.hashCode(42) + " " + Integer.signum(-5));
        System.out.println(Integer.bitCount(255) + " " + Integer.highestOneBit(100) + " " + Integer.lowestOneBit(12));
        System.out.println(Integer.numberOfLeadingZeros(1) + " " + Integer.numberOfTrailingZeros(8));
        System.out.println(Integer.reverse(1) + " " + Integer.reverseBytes(1));
        System.out.println(Integer.rotateLeft(1, 4) + " " + Integer.rotateRight(16, 4));
        System.out.println(Integer.parseUnsignedInt("4294967295") + " " + Integer.toUnsignedString(-1));
        System.out.println(Integer.toUnsignedString(-1, 16));
        System.out.println(Integer.divideUnsigned(-2, 3) + " " + Integer.remainderUnsigned(-2, 3));
        System.out.println(Integer.compareUnsigned(-1, 1));
        System.out.println(Integer.SIZE + " " + Integer.BYTES);

        System.out.println(Double.MAX_VALUE + " " + Double.MIN_VALUE + " " + Double.MIN_NORMAL);
        System.out.println(Double.POSITIVE_INFINITY + " " + Double.NEGATIVE_INFINITY + " " + Double.NaN);
        System.out.println(Double.isNaN(0.0 / 0.0) + " " + Double.isInfinite(1.0 / 0.0) + " " + Double.isFinite(1.0));
        System.out.println(Double.compare(1.0, 2.0) + " " + Double.compare(0.0, -0.0) + " " + Double.compare(Double.NaN, 1.0));
        System.out.println(Double.max(-0.0, 0.0) + " " + Double.min(1.5, 2.5) + " " + Double.sum(1.25, 2.5));
        System.out.println(Double.hashCode(1.5) + " " + Double.hashCode(0.0));
        System.out.println(Double.toHexString(1.0) + " " + Double.toHexString(-2.5) + " " + Double.toHexString(0.0));
        System.out.println(Double.valueOf("2.5") + Double.valueOf(0.25));

        System.out.println(Character.isAlphabetic('x') + " " + Character.isWhitespace('\t') + " " + Character.isSpaceChar('\t'));
        System.out.println(Character.isJavaIdentifierStart('_') + " " + Character.isJavaIdentifierPart('7'));
        System.out.println(Character.isISOControl('\n') + " " + Character.isDefined('a'));
        System.out.println(Character.getNumericValue('7') + " " + Character.getNumericValue('f') + " " + Character.getNumericValue('!'));
        System.out.println(Character.digit('f', 16) + " " + Character.digit('9', 8) + " " + Character.forDigit(11, 16));
        System.out.println(Character.compare('a', 'b') + " " + Character.hashCode('A'));
        System.out.println(Character.toString('q') + Character.valueOf('!'));
        System.out.println(Character.isHighSurrogate('a') + " " + Character.charCount(65) + " " + Character.charCount(128512));
        System.out.println(Character.MIN_RADIX + " " + Character.MAX_RADIX);

        System.out.println(Boolean.parseBoolean("TRUE") + " " + Boolean.parseBoolean("nope"));
        System.out.println(Boolean.toString(true) + " " + Boolean.valueOf(false));
        System.out.println(Boolean.compare(true, false) + " " + Boolean.compare(false, false));
        System.out.println(Boolean.hashCode(true) + " " + Boolean.hashCode(false));
        System.out.println(Boolean.logicalAnd(true, false) + " " + Boolean.logicalOr(true, false) + " " + Boolean.logicalXor(true, true));
        System.out.println(Boolean.TRUE + " " + Boolean.FALSE);
    }
}
"#
);

differential_test!(
    diff_list_full_api,
    "DiffListFull",
    r#"
import java.util.ArrayList;

public class DiffListFull {
    public static void main(String[] args) {
        ArrayList<String> names = new ArrayList<>();
        names.add("ada");
        names.add("alan");
        names.add("grace");
        names.add("alan");

        System.out.println(names.contains("alan") + " " + names.contains("linus"));
        System.out.println(names.indexOf("alan") + " " + names.lastIndexOf("alan") + " " + names.indexOf("x"));
        System.out.println(names.remove("alan") + " " + names + " " + names.remove("nobody"));

        ArrayList<String> more = new ArrayList<>();
        more.add("edsger");
        more.add("barbara");
        System.out.println(names.addAll(more) + " " + names);
        ArrayList<String> insert = new ArrayList<>();
        insert.add("X");
        names.addAll(1, insert);
        System.out.println(names);

        ArrayList<String> copy = new ArrayList<>();
        copy.addAll(names);
        System.out.println(names.equals(copy) + " " + names.hashCode() + " " + copy.hashCode());
        copy.set(0, "zzz");
        System.out.println(names.equals(copy));

        System.out.println(names.toString());
        names.ensureCapacity(100);
        names.trimToSize();
        names.clear();
        System.out.println(names.isEmpty() + " " + names.size() + " " + names);

        ArrayList<Integer> nums = new ArrayList<>();
        nums.add(5);
        nums.add(7);
        nums.add(5);
        System.out.println(nums.contains(7) + " " + nums.indexOf(5) + " " + nums.lastIndexOf(5));
        System.out.println(nums.hashCode());
    }
}
"#
);

differential_test!(
    diff_string_format,
    "DiffFormat",
    r#"
import java.util.*;

public class DiffFormat {
    public static void main(String[] args) {
        // Conversions.
        System.out.println(String.format("%d|%s|%f|%c|%b", 42, "hi", 2.5, 'x', true));
        System.out.println(String.format("%S|%B|%C", "shout", false, 'q'));
        System.out.println(String.format("%x|%X|%o", 255, 255, 8));
        System.out.println(String.format("%e|%E", 12345.6789, 0.000123));
        System.out.println(String.format("%g|%G", 12345.6789, 0.0000123));
        System.out.println(String.format("100%% done|%n").length());

        // Flags, width, precision.
        System.out.println(String.format("[%5d][%-5d][%05d]", 42, 42, 42));
        System.out.println(String.format("[%+d][% d][%+d]", 42, 42, -42));
        System.out.println(String.format("[%,d][%,d]", 1234567, -9876543));
        System.out.println(String.format("[%(d][%(d]", -42, 42));
        System.out.println(String.format("[%#x][%#o]", 255, 8));
        System.out.println(String.format("[%10.3f][%-10.2f][%010.1f]", 3.14159, 2.5, -7.25));
        System.out.println(String.format("[%.0f][%.4f]", 2.5, 1.0 / 3.0));
        System.out.println(String.format("[%8.3s][%-6.2s]", "truncate", "me"));
        System.out.println(String.format("[%,.2f]", 1234567.891));

        // HALF_UP rounding ties (would be wrong with half-even).
        System.out.println(String.format("%.2f %.2f %.1f %.0f", 2.675, 0.125, 0.25, 2.5));
        System.out.println(String.format("%.2e %.1e", 1.115e3, 2.5e-3));

        // Argument indexes and reuse.
        System.out.println(String.format("%2$s-%1$s-%2$s", "a", "b"));

        // Special values and null.
        String nothing = null;
        System.out.println(String.format("%s|%b|%h", nothing, nothing, nothing));
        System.out.println(String.format("%f|%e|%g", 0.0, 0.0, 0.0));
        System.out.println(String.format("%f", 1.0 / 0.0) + " " + String.format("%f", 0.0 / 0.0));

        // %h hash rendering.
        System.out.println(String.format("%h|%H", "hello", "hello"));

        // printf on streams.
        System.out.printf("printf: %d %s %.3f%n", 7, "seven", 7.0 / 3.0);
        System.out.printf("no args%n");

        // Errors are the real Java exception types.
        try {
            String.format("%d", "not a number");
        } catch (IllegalFormatConversionException e) {
            System.out.println("conversion: " + e.getMessage());
        }
        try {
            String.format("%d %d", 1);
        } catch (MissingFormatArgumentException e) {
            System.out.println("missing: " + e.getMessage());
        }
        try {
            String.format("%q", 1);
        } catch (UnknownFormatConversionException e) {
            System.out.println("unknown: " + e.getMessage());
        }
    }
}
"#
);

differential_test!(
    diff_expressions_milestone,
    "DiffExprs",
    r#"
public class DiffExprs {
    public static void main(String[] args) {
        // Ternary, nesting, mixed numeric branches.
        int a = 7;
        System.out.println(a > 5 ? "big" : "small");
        System.out.println(a % 2 == 0 ? a / 2 : a * 3 + 1);
        double mixed = a > 0 ? 1 : 2.5;
        System.out.println(mixed);
        String s = a < 0 ? null : "present";
        System.out.println(s);
        System.out.println(a > 3 ? a > 6 ? "six+" : "four-six" : "small");

        // Bitwise and shifts, precedence, compound forms.
        int bits = 0b1010_1100;
        System.out.println(bits + " " + 0x1F + " " + 0755 + " " + 1_000_000);
        System.out.println((bits & 0x0F) + " " + (bits | 0x03) + " " + (bits ^ 0xFF));
        System.out.println(~bits + " " + (bits << 2) + " " + (bits >> 2) + " " + (-8 >> 1) + " " + (-8 >>> 28));
        System.out.println(1 << 33);
        int acc = 0xF0;
        acc &= 0x3C; System.out.print(acc + " ");
        acc |= 0x03; System.out.print(acc + " ");
        acc ^= 0xFF; System.out.print(acc + " ");
        acc <<= 2;   System.out.print(acc + " ");
        acc >>= 1;   System.out.print(acc + " ");
        acc >>>= 1;  System.out.println(acc);
        boolean flag = true & false | true ^ false;
        System.out.println(flag + " " + (2 + 3 << 1) + " " + (1 | 2 & 3));

        // Expression-position increment/decrement.
        int x = 5;
        int y = x++ + ++x;
        System.out.println(x + " " + y);
        int[] arr = {10, 20, 30};
        int i = 0;
        int grabbed = arr[i++] + arr[i++];
        System.out.println(grabbed + " " + i);
        arr[0] = --x * 2;
        System.out.println(arr[0] + " " + x);
        int old = arr[1]--;
        System.out.println(old + " " + arr[1]);
        double d = 1.5;
        System.out.println(d++ + " " + d + " " + --d);
        char c = 'a';
        System.out.println(c++ + "" + c + (char) (c + 1));
    }
}
"#
);

differential_test!(
    diff_switch,
    "DiffSwitch",
    r#"
public class DiffSwitch {
    static String dayKind(int day) {
        switch (day) {
            case 1:
            case 7:
                return "weekend";
            case 2:
            case 3:
            case 4:
            case 5:
            case 6:
                return "weekday";
            default:
                return "invalid";
        }
    }

    public static void main(String[] args) {
        for (int day = 0; day <= 8; day++) {
            System.out.print(dayKind(day).charAt(0));
        }
        System.out.println();

        // Fall-through accumulates.
        for (int level = 1; level <= 4; level++) {
            String message = "";
            switch (level) {
                case 3:
                    message += "high ";
                case 2:
                    message += "medium ";
                case 1:
                    message += "low";
                    break;
                default:
                    message = "off the chart";
            }
            System.out.println(level + ": " + message);
        }

        // String switch with break and default position first.
        String[] commands = { "start", "stop", "pause", "reset" };
        for (String command : commands) {
            switch (command) {
                default:
                    System.out.println("unknown: " + command);
                    break;
                case "start":
                    System.out.println("go");
                    break;
                case "stop":
                case "pause":
                    System.out.println("halt-ish: " + command);
                    break;
            }
        }

        // char selector; switch inside a loop with continue past it.
        int vowels = 0;
        String text = "hello world";
        for (int i = 0; i < text.length(); i++) {
            switch (text.charAt(i)) {
                case 'a': case 'e': case 'i': case 'o': case 'u':
                    vowels++;
                    break;
                case ' ':
                    continue;
            }
        }
        System.out.println(vowels);

        // No default: falls straight through when unmatched.
        switch (99) {
            case 1:
                System.out.println("one");
        }
        System.out.println("done");
    }
}
"#
);

differential_test!(
    diff_long_type,
    "DiffLong",
    r#"
public class DiffLong {
    static long factorial(int n) {
        long result = 1L;
        for (int i = 2; i <= n; i++) {
            result *= i;
        }
        return result;
    }

    public static void main(String[] args) {
        // Literals, arithmetic, overflow wrapping.
        long big = 9_000_000_000L;
        long hexed = 0xFFFF_FFFF_FFFFL;
        System.out.println(big + " " + hexed + " " + 1L + " " + 0L);
        System.out.println(big * 3 + " " + big / 7 + " " + big % 7 + " " + -big);
        System.out.println(Long.MAX_VALUE + " " + Long.MIN_VALUE);
        System.out.println(Long.MAX_VALUE + 1);
        System.out.println(factorial(20));

        // Mixed promotion int/long/double, casts both ways.
        int small = 41;
        long widened = small + 1L;
        double d = widened / 2 + 0.5;
        System.out.println(widened + " " + d + " " + (int) big + " " + (long) 2.75 + " " + (char) 66L);

        // Comparisons and LCMP paths.
        System.out.println((big > hexed) + " " + (big == 9000000000L) + " " + (0L >= -1L));

        // Bitwise / shifts (six-bit count masking).
        System.out.println((big & 0xFFL) + " " + (big | 1) + " " + (big ^ big) + " " + ~0L);
        System.out.println((1L << 40) + " " + (-16L >> 2) + " " + (-16L >>> 60) + " " + (1L << 65));

        // long[] arrays, increments, compound narrowing.
        long[] values = { 1L, 1L << 34, 3L };
        values[0]++;
        values[1] += 5;
        long total = 0;
        for (long value : values) {
            total += value;
        }
        System.out.println(values[0] + " " + values[1] + " " + total + " " + values.length);
        int shrunk = 5;
        shrunk += 10L;
        System.out.println(shrunk);
        long counter = 1;
        System.out.println(counter++ + " " + counter + " " + --counter);

        // Long statics and Math long overloads.
        System.out.println(Long.parseLong("-9000000000") + " " + Long.toString(255L));
        System.out.println(Long.toBinaryString(10L) + " " + Long.toHexString(-1L));
        System.out.println(Long.compare(2L, 9L) + " " + Long.max(2L, 9L) + " " + Long.sum(2L, 9L));
        System.out.println(Long.hashCode(9000000000L) + " " + Long.bitCount(255L));
        System.out.println(Long.numberOfLeadingZeros(1L) + " " + Long.reverseBytes(1L));
        System.out.println(Math.abs(-9000000000L) + " " + Math.max(1L, 2L) + " " + Math.toIntExact(42L));
        System.out.println(Math.multiplyHigh(Long.MAX_VALUE, 2L));
        System.out.println(Double.doubleToLongBits(2.0) + " " + Double.longBitsToDouble(4611686018427387904L));

        // Formatting with %d and long.
        System.out.println(String.format("%d|%,d|%x", 9000000000L, 9000000000L, 255L));
        System.out.println(String.valueOf(9000000000L) + " " + ("" + 123L));

        // Ternary joining long with int.
        long chosen = small > 0 ? 1 : big;
        System.out.println(chosen);
    }
}
"#
);

differential_test!(
    diff_float_type,
    "DiffFloat",
    r#"
public class DiffFloat {
    static float half(float value) {
        return value / 2.0f;
    }

    public static void main(String[] args) {
        // Literals and rendering (Float.toString shortest digits).
        float a = 1.5f;
        float tenth = 0.1f;
        System.out.println(a + " " + tenth + " " + 1f + " " + 0.0f + " " + -0.0f);
        System.out.println(1e10f + " " + 1.23e-5f + " " + 3.4028235e38f);

        // Arithmetic in f32 precision (NOT double).
        System.out.println(tenth + 0.2f);
        System.out.println(a * tenth + " " + a / tenth + " " + a % 0.4f + " " + -a);
        System.out.println(half(7f));

        // Promotion: int op float -> float, long op float -> float,
        // float op double -> double.
        int i = 3;
        long big = 1L << 40;
        System.out.println(i + a);
        System.out.println(big + a);
        System.out.println(a + 0.25);

        // Casts every direction.
        System.out.println((int) 2.99f + " " + (long) -2.99f + " " + (char) 66.9f);
        System.out.println((float) 1.0 / 3 + " " + (float) (1.0 / 3.0));
        System.out.println((float) 16777217 + " " + (float) 16777217L);

        // Comparisons and NaN.
        float nan = 0.0f / 0.0f;
        System.out.println((a > tenth) + " " + (nan == nan) + " " + (nan < a) + " " + (nan >= a));
        System.out.println(1.0f / 0.0f + " " + -1.0f / 0.0f);

        // float[] arrays, ++/--, compound assignment.
        float[] values = { 0.5f, 1.5f, 2.5f };
        values[0]++;
        values[1] -= 0.25f;
        values[2] *= 2;
        float total = 0f;
        for (float value : values) {
            total += value;
        }
        System.out.println(values[0] + " " + values[1] + " " + values[2] + " " + total);
        float ticker = 1.0f;
        System.out.println(ticker++ + " " + ticker + " " + --ticker);
        double widened = a;
        System.out.println(widened);

        // Float statics and Math overloads.
        System.out.println(Float.parseFloat("2.5") + " " + Float.toString(0.1f));
        System.out.println(Float.isNaN(nan) + " " + Float.isInfinite(1f / 0f) + " " + Float.isFinite(a));
        System.out.println(Float.compare(-0.0f, 0.0f) + " " + Float.max(1f, 2f) + " " + Float.sum(1f, 2f));
        System.out.println(Float.MAX_VALUE + " " + Float.MIN_VALUE + " " + Float.MIN_NORMAL);
        System.out.println(Float.floatToIntBits(2.0f) + " " + Float.intBitsToFloat(1073741824));
        System.out.println(Float.hashCode(2.0f) + " " + Float.NaN + " " + Float.POSITIVE_INFINITY);
        System.out.println(Math.abs(-2.5f) + " " + Math.max(1.5f, 2.5f) + " " + Math.min(-0.0f, 0.0f) + " " + Math.signum(-7.5f));

        // Formatting: %f widens through doubleValue(), %s uses Float.toString.
        System.out.println(String.format("%.3f|%s|%e", 0.1f, 0.1f, 1234.5f));
        System.out.println(String.valueOf(0.1f) + " " + ("" + 2.5f));

        // Ternary joining and lossy-direction rejection is compile-time;
        // here the valid joins.
        float chosen = i > 0 ? 1 : a;
        double mixed = i > 0 ? a : 2.0;
        System.out.println(chosen + " " + mixed);
    }
}
"#
);

differential_test!(
    diff_short_byte,
    "DiffSmall",
    r#"
public class DiffSmall {
    static short doubled(short value) {
        return (short) (value * 2);
    }

    public static void main(String[] args) {
        // Constant narrowing in declarations and assignments.
        byte b = 100;
        short s = 30000;
        byte negative = -128;
        s = 12; b = -5;
        System.out.println(b + " " + s + " " + negative);

        // Arithmetic promotes to int; casts truncate with wraparound.
        byte a1 = 100, a2 = 27;
        int sum = a1 + a2;
        byte wrapped = (byte) (a1 + a2);
        short bigger = (short) 70000;
        System.out.println(sum + " " + wrapped + " " + bigger + " " + (byte) 130 + " " + (short) -70000);
        System.out.println((byte) 3.99 + " " + (short) 3.99 + " " + (byte) 300L + " " + (char) (short) 66);
        System.out.println(doubled((short) 12345) + " " + doubled((short) 30000));

        // Widening into everything.
        int wi = b; long wl = b; float wf = s; double wd = s;
        System.out.println(wi + " " + wl + " " + wf + " " + wd);

        // Compound assignment narrows back implicitly.
        byte acc = 10;
        acc += 5; acc *= 3; acc -= 100; acc /= 2;
        System.out.println(acc);
        short shifty = 1000;
        shifty <<= 4; shifty >>= 2; shifty ^= 0xFF;
        System.out.println(shifty);
        byte counter = 126;
        counter++; counter++; // wraps to -128
        System.out.println(counter + " " + counter--);

        // Arrays with distinct element semantics.
        byte[] bytes = { 1, -2, 127 };
        short[] shorts = { 100, -200, 32767 };
        bytes[0] += 200;      // wraps within the array store
        shorts[2]++;          // wraps to -32768
        int total = 0;
        for (byte x : bytes) { total += x; }
        for (short x : shorts) { total += x; }
        System.out.println(bytes[0] + " " + shorts[2] + " " + total + " " + bytes.length);

        // Bitwise on bytes/shorts (int results).
        System.out.println((b & 0xFF) + " " + (s | 1) + " " + (~b) + " " + (b << 2));

        // switch on byte and short selectors.
        switch (b) {
            case -5:
                System.out.println("minus five");
                break;
            default:
                System.out.println("other");
        }

        // Wrapper classes.
        System.out.println(Byte.MAX_VALUE + " " + Byte.MIN_VALUE + " " + Short.MAX_VALUE + " " + Short.MIN_VALUE);
        System.out.println(Byte.parseByte("-77") + " " + Short.parseShort("22222"));
        System.out.println(Byte.compare((byte) 1, (byte) 2) + " " + Short.compare((short) 5, (short) 5));
        System.out.println(Byte.toString((byte) 88) + " " + Short.toString((short) -99));
        System.out.println(Short.reverseBytes((short) 0x0102) + " " + Byte.hashCode((byte) 42));
        try {
            Byte.parseByte("200");
        } catch (NumberFormatException e) {
            System.out.println("range: " + (e.getMessage() != null));
        }

        // Formatting and concat go through int.
        System.out.println(String.format("%d %d %x", b, s, (byte) -1));
        System.out.println("" + b + s);

        // Ternary joins to int.
        int chosen = b > 0 ? b : s;
        System.out.println(chosen);
    }
}
"#
);

differential_test!(
    diff_labeled_break_continue,
    "DiffLabels",
    r#"
public class DiffLabels {
    public static void main(String[] args) {
        // Labeled break out of a nested loop.
        String found = "";
        search:
        for (int i = 0; i < 5; i++) {
            for (int j = 0; j < 5; j++) {
                if (i * j == 6) {
                    found = i + "," + j;
                    break search;
                }
            }
        }
        System.out.println(found);

        // Labeled continue skips to the outer loop's update.
        int pairs = 0;
        outer:
        for (int i = 1; i <= 4; i++) {
            for (int j = 1; j <= 4; j++) {
                if (i == j) continue outer;
                pairs++;
            }
        }
        System.out.println(pairs);

        // Three-deep with both labeled and unlabeled forms.
        String trace = "";
        top:
        for (int a = 0; a < 3; a++) {
            for (int b = 0; b < 3; b++) {
                for (int c = 0; c < 3; c++) {
                    if (c == 2) break;            // inner only
                    if (b == 2) continue top;     // all the way out
                    if (a == 2) break top;        // done entirely
                    trace += "" + a + b + c + " ";
                }
            }
        }
        System.out.println(trace.trim());

        // Labeled while and do-while.
        int n = 0;
        counter:
        while (true) {
            n++;
            if (n % 2 == 0) continue counter;
            if (n > 7) break counter;
        }
        System.out.println(n);

        // Labeled block: break jumps past it.
        int result = 0;
        block: {
            result = 1;
            if (args.length == 0) break block;
            result = 2;
        }
        System.out.println(result);

        // Labeled loop over a for-each.
        int[] grid = { 3, 1, 4, 1, 5, 9, 2, 6 };
        int target = -1;
        scan:
        for (int row = 0; row < 2; row++) {
            for (int value : grid) {
                if (value == 9) {
                    target = row * 100 + value;
                    break scan;
                }
            }
        }
        System.out.println(target);

        // Label reuse in sibling scopes (each is independent).
        int hits = 0;
        loop:
        for (int i = 0; i < 3; i++) {
            if (i == 1) continue loop;
            hits += 10;
        }
        loop:
        for (int i = 0; i < 3; i++) {
            if (i == 2) break loop;
            hits += 1;
        }
        System.out.println(hits);
    }
}
"#
);

differential_test!(
    diff_initializer_blocks,
    "DiffInit",
    r#"
public class DiffInit {
    static int a = 1;
    static int b;
    static int counter;

    // Static block interleaves with the static field inits above/below.
    static {
        b = a + 10;
        counter = 100;
    }

    static int c = b * 2;

    static {
        counter += c;
    }

    int x = 5;
    int y;
    String tag;

    // Instance block runs after super(), in source order with inits.
    {
        y = x * 2;
        tag = "made:" + x;
        counter++;
    }

    int z = y + 1;

    DiffInit() {
        // Constructor body runs after instance inits.
        z += 100;
    }

    DiffInit(int seed) {
        this();      // delegation: instance inits run once, via this()
        z += seed;
    }

    public static void main(String[] args) {
        System.out.println(a + " " + b + " " + c + " " + counter);

        DiffInit first = new DiffInit();
        System.out.println(first.x + " " + first.y + " " + first.z + " " + first.tag);
        System.out.println(counter);

        DiffInit second = new DiffInit(1000);
        System.out.println(second.z + " " + second.tag);
        System.out.println(counter);

        // A third, to confirm static blocks ran exactly once.
        DiffInit third = new DiffInit();
        System.out.println(a + " " + b + " " + c + " " + third.y);
    }
}
"#
);

differential_test!(
    diff_enum_simple,
    "DiffEnumSimple",
    r#"
enum Direction { NORTH, EAST, SOUTH, WEST }

public class DiffEnumSimple {
    static Direction turnRight(Direction d) {
        switch (d) {
            case NORTH: return Direction.EAST;
            case EAST: return Direction.SOUTH;
            case SOUTH: return Direction.WEST;
            default: return Direction.NORTH;
        }
    }

    public static void main(String[] args) {
        System.out.println(Direction.NORTH + " " + Direction.WEST.ordinal());
        for (Direction d : Direction.values()) {
            System.out.print(d.name() + "->" + turnRight(d).name() + " ");
        }
        System.out.println();
        System.out.println(Direction.valueOf("SOUTH") == Direction.SOUTH);
        System.out.println(Direction.values().length);

        // Reference equality and use as a field/local.
        Direction here = Direction.EAST;
        boolean east = here == Direction.EAST;
        System.out.println(east + " " + (here == Direction.WEST));

        // valueOf failure.
        try {
            Direction.valueOf("UP");
        } catch (IllegalArgumentException e) {
            System.out.println("bad: " + (e.getMessage() != null));
        }

        // Enum in a String concat chain and equality in a loop.
        int northish = 0;
        for (Direction d : Direction.values()) {
            if (d == Direction.NORTH || d == Direction.SOUTH) northish++;
        }
        System.out.println(northish);
    }
}
"#
);

differential_test!(
    diff_enum_with_fields,
    "DiffEnumFields",
    r#"
enum Planet {
    MERCURY(3.303e+23, 2.4397e6),
    EARTH(5.976e+24, 6.37814e6),
    JUPITER(1.9e+27, 7.1492e7);

    private final double mass;
    private final double radius;

    Planet(double mass, double radius) {
        this.mass = mass;
        this.radius = radius;
    }

    static final double G = 6.67300E-11;

    double surfaceGravity() {
        return G * mass / (radius * radius);
    }

    double surfaceWeight(double otherMass) {
        return otherMass * surfaceGravity();
    }

    @Override
    public String toString() {
        return name() + "[ordinal=" + ordinal() + "]";
    }
}

public class DiffEnumFields {
    public static void main(String[] args) {
        double earthWeight = 175;
        double mass = earthWeight / Planet.EARTH.surfaceGravity();
        for (Planet p : Planet.values()) {
            System.out.println(p + " weight: " + Math.round(p.surfaceWeight(mass)));
        }
        System.out.println(Planet.JUPITER.surfaceGravity() > Planet.EARTH.surfaceGravity());
        System.out.println(Planet.valueOf("MERCURY").ordinal());

        // Custom toString flows through switch and concat.
        Planet home = Planet.EARTH;
        switch (home) {
            case EARTH: System.out.println("home is " + home); break;
            default: System.out.println("away");
        }
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
