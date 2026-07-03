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
