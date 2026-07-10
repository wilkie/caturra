//! Differential tests: run the same program through the installed JDK
//! (`javac` + `java`) and through the caturra engine, and require
//! identical stdout. This pins our semantics to the reference
//! implementation rather than to our own expectations.
//!
//! Skips (with a note) when no JDK is installed, so a checkout without Java
//! still builds; `apt install openjdk-11-jdk` enables it. A skipped test
//! reports `ok`, which is indistinguishable from a passing one — so CI sets
//! `CATURRA_REQUIRE_JDK=1`, and then a missing JDK is a failure rather than
//! ninety silent no-ops.

use std::io::Write;
use std::process::Command;

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

fn jdk_available() -> bool {
    let present = Command::new("javac")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
        && Command::new("java")
            .arg("--version")
            .output()
            .is_ok_and(|out| out.status.success());
    assert!(
        present || std::env::var_os("CATURRA_REQUIRE_JDK").is_none(),
        "CATURRA_REQUIRE_JDK is set but no JDK is on PATH — the differential \
         suite would have reported `ok` without comparing anything"
    );
    present
}

/// Whether javac refuses to compile `source`.
fn javac_rejects(class_name: &str, source: &str) -> bool {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(source, &mut hasher);
    let fingerprint = std::hash::Hasher::finish(&hasher);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("reject-{class_name}-{fingerprint:x}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let java_file = dir.join(format!("{class_name}.java"));
    std::fs::write(&java_file, source).expect("write source");
    let compile = Command::new("javac")
        .arg(java_file.file_name().expect("file name"))
        .current_dir(&dir)
        .output()
        .expect("javac runs");
    !compile.status.success()
}

/// javac's headline for the first error: what follows `error:` on its first
/// line. `None` when javac accepts the program.
fn javac_first_error(class_name: &str, source: &str) -> Option<String> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(source, &mut hasher);
    let fingerprint = std::hash::Hasher::finish(&hasher);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("wording-{class_name}-{fingerprint:x}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let java_file = dir.join(format!("{class_name}.java"));
    std::fs::write(&java_file, source).expect("write source");
    let compile = Command::new("javac")
        .arg(java_file.file_name().expect("file name"))
        .current_dir(&dir)
        .output()
        .expect("javac runs");
    if compile.status.success() {
        return None;
    }
    String::from_utf8_lossy(&compile.stderr)
        .lines()
        .find_map(|line| line.split_once("error: ").map(|(_, rest)| rest.to_owned()))
}

/// caturra's first diagnostic. `None` when caturra accepts the program.
fn caturra_first_error(class_name: &str, source: &str) -> Option<String> {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{class_name}.java"),
        text: source.to_owned(),
    }]);
    compilation
        .diagnostics
        .first()
        .map(|diagnostic| diagnostic.message.clone())
}

/// Both javac and caturra must refuse `source`.
///
/// A `differential_test!` can only run a program javac accepts, so the suite
/// cannot catch caturra growing *more permissive* than javac — which is the
/// direction that hurts, because such a program compiles in the playground and
/// fails on a real JDK. This checks that direction directly.
fn assert_both_reject(class_name: &str, source: &str) {
    assert!(
        javac_rejects(class_name, source),
        "javac ACCEPTS {class_name}, so caturra rejecting it is a deliberate \
         strictness rather than a shared rule — do not assert it here"
    );
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{class_name}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        !compilation.success(),
        "caturra ACCEPTS {class_name}, which javac rejects: it would compile \
         in the playground and fail on a real JDK"
    );
}

/// Run `source` through javac+java, returning stdout.
fn run_with_jdk(class_name: &str, source: &str) -> String {
    run_with_jdk_stdin(class_name, source, "")
}

/// Run through the JDK with piped standard input.
fn run_with_jdk_stdin(class_name: &str, source: &str, stdin: &str) -> String {
    // Two tests may legitimately declare the same class name; give each its
    // own directory, or javac's output races between them.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(source, &mut hasher);
    let fingerprint = std::hash::Hasher::finish(&hasher);
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("differential-{class_name}-{fingerprint:x}"));
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

/// Run `source` through the caturra engine, returning stdout.
fn run_with_caturra(class_name: &str, source: &str) -> String {
    run_with_caturra_stdin(class_name, source, "")
}

/// Run through caturra with scripted standard input.
fn run_with_caturra_stdin(class_name: &str, source: &str, stdin: &str) -> String {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{class_name}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "caturra rejected {class_name}: {:?}",
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
        "caturra run failed for {class_name}: {result:?}; stderr: {}",
        console.stderr_text()
    );
    console.stdout_text()
}

fn assert_same_output(class_name: &str, source: &str) {
    let expected = run_with_jdk(class_name, source);
    let actual = run_with_caturra(class_name, source);
    assert_eq!(
        actual, expected,
        "output diverges from the reference JDK for {class_name}"
    );
}

fn assert_same_output_with_stdin(class_name: &str, source: &str, stdin: &str) {
    let expected = run_with_jdk_stdin(class_name, source, stdin);
    let actual = run_with_caturra_stdin(class_name, source, stdin);
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

/// A program neither javac nor caturra may accept.
macro_rules! differential_reject {
    ($name:ident, $class:literal, $source:literal) => {
        #[test]
        fn $name() {
            if !jdk_available() {
                eprintln!("skipping: no JDK on PATH");
                return;
            }
            assert_both_reject($class, $source);
        }
    };
}

/// A program javac accepts and caturra refuses. caturra is deliberately
/// stricter in a handful of places — always where javac's parameter is
/// `Object` or an erased `Collection`, and the mismatch would only surface as
/// a runtime `ClassCastException` or `ArrayStoreException`. Being stricter is
/// safe (nothing that compiles here fails on a JDK), but it is a divergence,
/// and this pins it so it cannot be widened or lost by accident.
macro_rules! stricter_than_javac {
    ($name:ident, $class:literal, $source:literal) => {
        #[test]
        fn $name() {
            if !jdk_available() {
                eprintln!("skipping: no JDK on PATH");
                return;
            }
            assert!(
                javac_first_error($class, $source).is_none(),
                "javac rejects {}, so this is a shared rule, not a strictness",
                $class
            );
            assert!(
                caturra_first_error($class, $source).is_some(),
                "caturra now accepts {} — the documented strictness is gone",
                $class
            );
        }
    };
}

/// A program javac refuses and caturra accepts. This is the direction that
/// hurts: it compiles in the playground and fails on a real JDK.
///
/// **There are currently no such programs**, which is why this macro has no
/// callers. The last three — `Collections.sort`/`max`/`binarySearch` over a
/// non-`Comparable` element — were closed on 2026-07-09 and became
/// `differential_reject!` cases. Keep the macro: it is how the next one gets
/// recorded, and a case asserted here is a case that cannot be forgotten.
#[allow(unused_macros)]
macro_rules! looser_than_javac {
    ($name:ident, $class:literal, $source:literal) => {
        #[test]
        fn $name() {
            if !jdk_available() {
                eprintln!("skipping: no JDK on PATH");
                return;
            }
            assert!(
                javac_first_error($class, $source).is_some(),
                "javac accepts {}, so caturra accepting it is no divergence",
                $class
            );
            assert!(
                caturra_first_error($class, $source).is_none(),
                "caturra now rejects {} — good; delete this test and the note \
                 in LANGUAGE.md",
                $class
            );
        }
    };
}

/// The same, for programs that read stdin.
macro_rules! differential_test_stdin {
    ($name:ident, $class:literal, $source:literal, $stdin:literal) => {
        #[test]
        fn $name() {
            if !jdk_available() {
                eprintln!("skipping: no JDK on PATH");
                return;
            }
            assert_same_output_with_stdin($class, $source, $stdin);
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
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
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

    let dir = std::env::temp_dir().join(format!("caturra-javap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    for class in compilation.classes {
        let bytes = caturra_classfile::write_class_file(&class.class_file);
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
    diff_varargs,
    "DiffVarargs",
    r#"
public class DiffVarargs {
    static int sum(int... numbers) {
        int total = 0;
        for (int n : numbers) total += n;
        return total;
    }

    static String join(String sep, String... parts) {
        String result = "";
        for (int i = 0; i < parts.length; i++) {
            if (i > 0) result += sep;
            result += parts[i];
        }
        return result;
    }

    static double avg(double first, double... rest) {
        double total = first;
        for (double d : rest) total += d;
        return total / (rest.length + 1);
    }

    // Overloading: a fixed-arity method wins over varargs.
    static String pick(int a, int b) { return "fixed:" + (a + b); }
    static String pick(int... xs) { return "varargs:" + xs.length; }

    public static void main(String[] args) {
        // Spread form with varying counts, including zero.
        System.out.println(sum() + " " + sum(5) + " " + sum(1, 2, 3) + " " + sum(1, 2, 3, 4, 5));

        // Passing an actual array (array form).
        int[] arr = { 10, 20, 30 };
        System.out.println(sum(arr));

        // A fixed leading parameter plus varargs.
        System.out.println(join("-", "a", "b", "c"));
        System.out.println(join(", ", "solo"));
        System.out.println(join("+") + "|");   // zero trailing args

        // Widening of trailing args to the element type.
        System.out.println(avg(2.0, 4, 6));    // ints widen to double
        System.out.println(avg(5.0));

        // Overload resolution: fixed-arity beats varargs at arity 2.
        System.out.println(pick(3, 4));
        System.out.println(pick(1));
        System.out.println(pick(1, 2, 3));

        // Varargs argument built from expressions.
        int x = 7;
        System.out.println(sum(x, x * 2, x - 1));
    }
}
"#
);

differential_test!(
    diff_nested_classes,
    "DiffNested",
    r#"
public class DiffNested {
    static class Node {
        int value;
        Node next;
        Node(int value) { this.value = value; }
        int reach() {
            int count = 1;
            Node n = next;
            while (n != null) { count++; n = n.next; }
            return count;
        }
    }

    static class Stack {
        private Node top;
        private int size;
        void push(int v) {
            Node node = new Node(v);
            node.next = top;
            top = node;
            size++;
        }
        int pop() {
            int v = top.value;
            top = top.next;
            size--;
            return v;
        }
        boolean isEmpty() { return top == null; }
        int size() { return size; }
    }

    enum Op { ADD, SUB, MUL }

    static int apply(Op op, int a, int b) {
        switch (op) {
            case ADD: return a + b;
            case SUB: return a - b;
            default: return a * b;
        }
    }

    public static void main(String[] args) {
        // Linked list via nested Node, chained field access.
        Node head = new Node(1);
        head.next = new Node(2);
        head.next.next = new Node(3);
        head.next.next.next = new Node(4);
        System.out.println(head.value + " " + head.next.next.value + " " + head.reach());

        // Field mutation through a chain.
        head.next.value = 20;
        System.out.println(head.next.value + " " + head.next.next.value);

        // A second nested class using the first.
        Stack stack = new Stack();
        for (int i = 1; i <= 5; i++) stack.push(i * i);
        System.out.println(stack.size() + " " + stack.isEmpty());
        int sum = 0;
        while (!stack.isEmpty()) sum += stack.pop();
        System.out.println(sum + " " + stack.isEmpty());

        // Nested enum with switch.
        for (Op op : Op.values()) {
            System.out.print(op + "=" + apply(op, 6, 4) + " ");
        }
        System.out.println();

        // Qualified nested-type reference from the outer class.
        DiffNested.Node solo = new DiffNested.Node(99);
        System.out.println(solo.value + " " + solo.reach());
    }
}
"#
);

differential_test!(
    diff_interface_default_methods,
    "DiffDefault",
    r#"
interface Greeter {
    String name();
    // A default method calls an abstract method of the same interface.
    default String greet() {
        return "Hello, " + name() + "!";
    }
    default String shout() {
        return greet().toUpperCase();
    }
}

interface Describable {
    default String describe() { return "a thing"; }
}

class Person implements Greeter {
    private String who;
    Person(String who) { this.who = who; }
    public String name() { return who; }
}

class Robot implements Greeter, Describable {
    public String name() { return "R2"; }
    // Override one default, inherit the other.
    public String greet() { return "BEEP " + name(); }
}

public class DiffDefault {
    static String greetAny(Greeter g) {
        return g.greet();
    }

    public static void main(String[] args) {
        Person p = new Person("Ada");
        System.out.println(p.greet());
        System.out.println(p.shout());

        Robot r = new Robot();
        System.out.println(r.greet());     // overridden
        System.out.println(r.shout());     // inherited default, calls overridden greet
        System.out.println(r.describe());  // inherited from a second interface

        // Dispatch through an interface-typed reference.
        System.out.println(greetAny(p));
        System.out.println(greetAny(r));

        Greeter g = p;
        System.out.println(g.greet() + " / " + g.name());
    }
}
"#
);

differential_test!(
    diff_user_generics,
    "DiffGenerics",
    r#"
class Box<T> {
    private T value;
    Box(T value) { this.value = value; }
    T get() { return value; }
    void set(T value) { this.value = value; }
    boolean has() { return value != null; }
}

class Pair<A, B> {
    private final A first;
    private final B second;
    Pair(A first, B second) { this.first = first; this.second = second; }
    A first() { return first; }
    B second() { return second; }
    String show() { return "(" + first + ", " + second + ")"; }
}

class GStack<E> {
    private Node<E> top;
    private int size;
    void push(E item) {
        Node<E> node = new Node<>(item);
        node.next = top;
        top = node;
        size++;
    }
    E pop() {
        E item = top.data;
        top = top.next;
        size--;
        return item;
    }
    boolean isEmpty() { return size == 0; }
    int size() { return size; }
}

class Node<E> {
    E data;
    Node<E> next;
    Node(E data) { this.data = data; }
}

public class DiffGenerics {
    static <T> String describe(T value) {
        return value.toString();
    }

    public static void main(String[] args) {
        // Cast-free reads: box.get() yields String directly.
        Box<String> sbox = new Box<>("hi");
        String s = sbox.get();
        System.out.println(s + " " + s.length() + " " + sbox.has());
        sbox.set("bye");
        System.out.println(sbox.get().toUpperCase());

        // Multi-parameter generics are raw (reads need casts).
        Pair<String, String> pair = new Pair<>("x", "y");
        System.out.println(pair.show() + " " + (String) pair.first());

        // Generic stack: pop() yields String, no cast.
        GStack<String> stack = new GStack<>();
        stack.push("a");
        stack.push("b");
        stack.push("c");
        System.out.println(stack.size());
        String out = "";
        while (!stack.isEmpty()) out += stack.pop();
        System.out.println(out);

        // Generic method (still needs a cast on the T return).
        System.out.println(describe("text"));

        // Object top type.
        Object obj = "erased";
        System.out.println(obj + " " + obj.equals("erased"));
    }
}
"#
);

differential_test!(
    diff_autoboxing,
    "DiffBox",
    r#"
import java.util.ArrayList;

public class DiffBox {
    // Boxing across a method boundary.
    static int unwrap(Integer boxed) { return boxed; }
    static Integer wrap(int primitive) { return primitive; }

    public static void main(String[] args) {
        // Box on assignment, unbox on assignment.
        Integer i = 5;
        int x = i;
        System.out.println(i + " " + x);

        Double d = 3.5;
        double dv = d;
        System.out.println(d + " " + dv);

        Boolean flag = true;
        boolean bv = flag;
        System.out.println(flag + " " + bv);

        Character c = 'Q';
        char cv = c;
        System.out.println(c + " " + cv);

        Long bignum = 9000000000L;
        long lv = bignum;
        System.out.println(bignum + " " + lv);

        // Box to Object.
        Object o1 = 42;
        Object o2 = 2.5;
        Object o3 = true;
        System.out.println(o1 + " " + o2 + " " + o3);

        // Arithmetic auto-unboxes.
        Integer a = 10, b = 3;
        System.out.println((a + b) + " " + (a * b) + " " + (a - b) + " " + (a / b) + " " + (a % b));
        System.out.println((a > b) + " " + (a <= b) + " " + (a == 10));
        Double p = 1.5, q = 2.5;
        System.out.println((p + q) + " " + (p < q));

        // Mixed boxed/primitive arithmetic.
        Integer m = 7;
        int result = m + 3;
        double mixed = m + 0.5;
        System.out.println(result + " " + mixed);

        // Wrapper instance methods.
        Integer n = 15;
        System.out.println(n.intValue() + " " + n.doubleValue() + " " + n.compareTo(20));
        System.out.println(n.equals(15) + " " + n.equals(16) + " " + n.toString());
        System.out.println(n.hashCode());

        // Boxing through method calls.
        System.out.println(unwrap(99) + " " + wrap(7));

        // Autoboxing into a collection (already supported) plus reads.
        ArrayList<Integer> list = new ArrayList<>();
        list.add(1);
        list.add(2);
        list.add(3);
        int sum = 0;
        for (int v : list) sum += v;
        System.out.println(sum + " " + list.get(1));

        // Integer caching semantics don't matter for value use.
        Integer big1 = 1000;
        int backToPrim = big1 + 1;
        System.out.println(backToPrim);
    }
}
"#
);

differential_test!(
    diff_anonymous_classes,
    "DiffAnon",
    r#"
interface Transformer {
    int transform(int value);
}

interface Describable {
    String describe();
    default String shout() { return describe().toUpperCase(); }
}

abstract class Animal {
    abstract String sound();
    String describe() { return "an animal that says " + sound(); }
}

public class DiffAnon {
    static int applyAll(Transformer t, int[] values) {
        int total = 0;
        for (int v : values) total += t.transform(v);
        return total;
    }

    public static void main(String[] args) {
        // Anonymous class implementing an interface.
        Transformer doubler = new Transformer() {
            public int transform(int value) { return value * 2; }
        };
        Transformer square = new Transformer() {
            public int transform(int value) {
                return value * value;
            }
        };
        System.out.println(doubler.transform(5) + " " + square.transform(5));

        int[] nums = { 1, 2, 3, 4 };
        System.out.println(applyAll(doubler, nums) + " " + applyAll(square, nums));

        // Anonymous class with its own field and helper method.
        Transformer counter = new Transformer() {
            private int calls = 0;
            public int transform(int value) {
                calls++;
                return value + calls;
            }
        };
        System.out.println(counter.transform(10) + " " + counter.transform(10) + " " + counter.transform(10));

        // Inherited default method through an anonymous class.
        Describable thing = new Describable() {
            public String describe() { return "a widget"; }
        };
        System.out.println(thing.describe() + " / " + thing.shout());

        // Anonymous class extending an abstract class, overriding one
        // method and inheriting another.
        Animal cat = new Animal() {
            String sound() { return "meow"; }
        };
        System.out.println(cat.sound() + " -> " + cat.describe());

        // Anonymous class in an expression used immediately.
        System.out.println(new Transformer() {
            public int transform(int v) { return -v; }
        }.transform(42));
    }
}
"#
);

differential_test!(
    diff_anonymous_capture,
    "DiffCapture",
    r#"
interface IntFn { int apply(int x); }
interface Sink { void accept(int x); }
interface Producer { String make(); }

public class DiffCapture {
    static int applyTo(IntFn f, int v) { return f.apply(v); }

    public static void main(String[] args) {
        // Capture a single effectively-final local.
        int offset = 100;
        IntFn addOffset = new IntFn() {
            public int apply(int x) { return x + offset; }
        };
        System.out.println(addOffset.apply(5) + " " + applyTo(addOffset, 20));

        // Capture several locals of mixed types.
        int base = 10;
        int scale = 3;
        String label = "result=";
        Producer p = new Producer() {
            public String make() { return label + (base * scale); }
        };
        System.out.println(p.make());

        // Capture in a loop: each anonymous instance sees its own value.
        IntFn[] fns = new IntFn[3];
        for (int i = 0; i < 3; i++) {
            final int captured = i * 10;
            fns[i] = new IntFn() {
                public int apply(int x) { return x + captured; }
            };
        }
        System.out.println(fns[0].apply(1) + " " + fns[1].apply(1) + " " + fns[2].apply(1));

        // Capture strings and an int used in a loop.
        String greeting = "hi";
        int times = 3;
        Producer repeater = new Producer() {
            public String make() {
                String out = "";
                for (int k = 0; k < times; k++) out += greeting;
                return out;
            }
        };
        System.out.println(repeater.make());

        // A captured value combined with the method's own parameter and
        // a field the anonymous class declares.
        int seed = 7;
        IntFn accumulate = new IntFn() {
            private int total = 0;
            public int apply(int x) {
                total += x + seed;
                return total;
            }
        };
        System.out.println(accumulate.apply(1) + " " + accumulate.apply(2) + " " + accumulate.apply(3));

        // Capturing an array and indexing into it.
        int[] data = { 5, 10, 15 };
        int index = 2;
        IntFn lookup = new IntFn() {
            public int apply(int x) { return data[index] + x; }
        };
        System.out.println(lookup.apply(100));
    }
}
"#
);

differential_test!(
    diff_lambdas,
    "DiffLambda",
    r#"
interface IntFn { int apply(int x); }
interface IntBiFn { int apply(int a, int b); }
interface Pred { boolean test(int x); }
interface Runner { void run(); }
interface Producer { String make(); }

public class DiffLambda {
    static int applyTo(IntFn f, int v) { return f.apply(v); }
    static int count(int[] xs, Pred p) {
        int n = 0;
        for (int x : xs) if (p.test(x)) n++;
        return n;
    }
    static IntFn compose(IntFn f, IntFn g) {
        return x -> f.apply(g.apply(x));
    }

    public static void main(String[] args) {
        // Expression and parenthesized forms.
        IntFn inc = x -> x + 1;
        IntFn sq = (x) -> x * x;
        System.out.println(inc.apply(10) + " " + sq.apply(6));

        // Multiple parameters, expression and block bodies.
        IntBiFn add = (a, b) -> a + b;
        IntBiFn max = (a, b) -> {
            if (a > b) return a;
            return b;
        };
        System.out.println(add.apply(3, 4) + " " + max.apply(9, 2));

        // Lambda as a method argument.
        System.out.println(applyTo(x -> x * 10, 5));
        int[] nums = { 1, 2, 3, 4, 5, 6 };
        System.out.println(count(nums, x -> x % 2 == 0));

        // Capturing lambdas.
        int base = 100;
        int factor = 3;
        IntFn affine = x -> x * factor + base;
        System.out.println(affine.apply(5));

        String prefix = "value: ";
        Producer p = () -> prefix + affine.apply(1);
        System.out.println(p.make());

        // Void lambda with a side effect.
        Runner r = () -> System.out.println("ran");
        r.run();

        // Lambda returned from a method (capturing its parameters).
        IntFn plusOneThenDouble = compose(x -> x * 2, x -> x + 1);
        System.out.println(plusOneThenDouble.apply(4));

        // Lambda stored in an array and called.
        IntFn[] table = new IntFn[3];
        for (int i = 0; i < 3; i++) {
            final int mult = i + 1;
            table[i] = x -> x * mult;
        }
        System.out.println(table[0].apply(10) + " " + table[1].apply(10) + " " + table[2].apply(10));

        // A lambda whose body captures and uses several things.
        int threshold = 4;
        Pred big = x -> x > threshold;
        System.out.println(big.test(5) + " " + big.test(2) + " " + count(nums, big));
    }
}
"#
);

differential_test!(
    diff_method_references,
    "DiffMethodRef",
    r#"
interface StrToInt { int apply(String s); }
interface IntBiOp { int apply(int a, int b); }
interface Factory { Box make(int v); }
interface Consumer { void accept(String s); }
interface Mapper { String apply(String s); }

class Box {
    int value;
    Box(int value) { this.value = value; }
    public String toString() { return "Box(" + value + ")"; }
    int doubled() { return value * 2; }
    static int triple(int n) { return n * 3; }
}

public class DiffMethodRef {
    static int useStrFn(StrToInt f, String s) { return f.apply(s); }

    public static void main(String[] args) {
        // Unbound instance method references.
        StrToInt len = String::length;
        System.out.println(len.apply("method") + " " + len.apply("reference"));

        Mapper upper = String::toUpperCase;
        Mapper trim = String::trim;
        System.out.println(upper.apply("hi") + "|" + trim.apply("  pad  ") + "|");

        // Static method references (library and user).
        StrToInt parse = Integer::parseInt;
        System.out.println(parse.apply("42") + " " + parse.apply("-8"));

        IntBiOp mx = Math::max;
        IntBiOp mn = Math::min;
        System.out.println(mx.apply(3, 9) + " " + mn.apply(3, 9));

        // Constructor reference.
        Factory f = Box::new;
        Box b = f.make(21);
        System.out.println(b + " " + b.doubled());

        // Bound instance method reference (on a value).
        Consumer out = System.out::println;
        out.accept("bound instance ref");

        // A method reference passed as an argument.
        System.out.println(useStrFn(String::length, "argument"));

        // Method reference used in a loop, and stored in an array.
        StrToInt[] fns = new StrToInt[2];
        fns[0] = String::length;
        fns[1] = Integer::parseInt;
        System.out.println(fns[0].apply("abcd") + " " + fns[1].apply("1000"));
    }
}
"#
);

differential_test!(
    diff_comparable,
    "DiffComparable",
    r#"
class Card implements Comparable<Card> {
    private int rank;
    public Card(int rank) { this.rank = rank; }
    public int getRank() { return rank; }
    public int compareTo(Card other) { return rank - other.rank; }
    public String toString() { return "C" + rank; }
}

interface Ranked<T> {
    int rankAgainst(T other);
}

class Player implements Ranked<Player>, Comparable<Player> {
    int score;
    Player(int score) { this.score = score; }
    public int rankAgainst(Player other) { return score - other.score; }
    public int compareTo(Player other) { return score - other.score; }
}

public class DiffComparable {
    // Selection sort over Comparable — array covariance + interface dispatch.
    static void sort(Comparable[] arr) {
        for (int i = 0; i < arr.length; i++) {
            int min = i;
            for (int j = i + 1; j < arr.length; j++) {
                if (arr[j].compareTo(arr[min]) < 0) min = j;
            }
            Comparable t = arr[min];
            arr[min] = arr[i];
            arr[i] = t;
        }
    }

    static Comparable maxOf(Comparable a, Comparable b) {
        return a.compareTo(b) >= 0 ? a : b;
    }

    public static void main(String[] args) {
        // Direct compareTo on a concrete type.
        Card a = new Card(7), b = new Card(3);
        System.out.println((a.compareTo(b) > 0) + " " + (b.compareTo(a) < 0) + " " + a.compareTo(a));

        // Sort through the Comparable interface (covariant array).
        Card[] cards = { new Card(5), new Card(2), new Card(8), new Card(1), new Card(4) };
        sort(cards);
        for (Card c : cards) System.out.print(c + " ");
        System.out.println();

        // A user generic interface implemented alongside Comparable.
        Player p = new Player(50), q = new Player(30);
        System.out.println(p.rankAgainst(q) + " " + p.compareTo(q));

        // A method taking Comparable, returning the larger.
        Card biggerCard = (Card) maxOf(a, b);
        System.out.println(biggerCard.getRank());
        Player winner = (Player) maxOf(p, q);
        System.out.println(winner.score);

        // Sort players too.
        Player[] players = { new Player(40), new Player(10), new Player(70), new Player(20) };
        sort(players);
        for (Player pl : players) System.out.print(pl.score + " ");
        System.out.println();
    }
}
"#
);

differential_test!(
    diff_codeorg_console_patterns,
    "DiffCodeorg",
    r#"
// Patterns drawn from the Code.org CSA console curriculum.

// A user class named Character shadows java.lang.Character.
class Character {
    String name;
    boolean hero;
    Character(String name, boolean hero) { this.name = name; this.hero = hero; }
    public String toString() { return name + (hero ? "(hero)" : "(villain)"); }
}

class FoodTruck {
    public static String businessName = "Project Mercury Pastries";
    private String owner;
    FoodTruck(String owner) { this.owner = owner; }
    public String getOwner() { return owner; }
}

class Account {
    protected double balance;
    Account(double b) { balance = b; }
    void withdraw(double amount) { balance -= amount; }
}

class BasicAccount extends Account {
    BasicAccount(double b) { super(b); }
    void withdraw(double amount) {
        // super.method() as a statement.
        super.withdraw(amount);
        System.out.println("Withdrew " + amount);
    }
}

public class DiffCodeorg {
    public static void main(String[] args) {
        // Leading-dot double literals.
        double tax = .1;
        double discount = .25;
        double total = 100 * (1 + tax) - .5;
        System.out.println(tax + " " + discount + " " + total);

        // Wrapper constructors (deprecated but taught).
        Integer i = new Integer(42);
        Double d = new Double(3.14);
        Integer parsed = new Integer("100");
        System.out.println(i + " " + d + " " + parsed + " " + (i + parsed));

        // ArrayList of a user-defined Character type.
        java.util.ArrayList<Character> cast = new java.util.ArrayList<Character>();
        cast.add(new Character("Mario", true));
        cast.add(new Character("Bowser", false));
        for (Character c : cast) System.out.print(c + " ");
        System.out.println();

        // Static field access and assignment through an instance.
        FoodTruck rosies = new FoodTruck("Rosie");
        FoodTruck erins = new FoodTruck("Erin");
        System.out.println(rosies.getOwner() + ": " + rosies.businessName);
        rosies.businessName = "The Neighborhood Food Truck";
        System.out.println(erins.getOwner() + ": " + erins.businessName);
        System.out.println(FoodTruck.businessName);

        // super.method() as a statement.
        BasicAccount acct = new BasicAccount(100.0);
        acct.withdraw(30.0);
        System.out.println(acct.balance);
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

differential_test!(
    diff_long_bit_ops_and_shifts,
    "DiffLongBits",
    r#"
public class DiffLongBits {
    static int f() { return 3; }
    static long g() { return 5L; }
    public static void main(String[] args) {
        // A shift takes the type of its LEFT operand; long & | ^ stay long.
        // caturra once typed all of these int, emitting IADD over a long value.
        System.out.println(((long) f() << 4) + f());
        System.out.println((g() & 0xFFL) + f());
        System.out.println((g() | 0L) + 1);
        System.out.println((g() ^ 1L) * 2);
        System.out.println((f() << 2) + f());
        System.out.println((f() & 1) + 1);
        System.out.println((g() >>> 1) + 1);
        System.out.println(((long) f() << 32) >> 32);
        System.out.println(((long) f() << 32) >>> 40);
        long mask = (1L << 48) - 1;
        System.out.println(mask);
        System.out.println((0x5DEECE66DL * 25214903917L + 0xBL) & mask);
        boolean p = true, q = false;
        System.out.println((p & q) + " " + (p | q) + " " + (p ^ q));
    }
}
"#
);

differential_test!(
    diff_random_seeded_sequences,
    "DiffRandom",
    r#"
import java.util.Random;
public class DiffRandom {
    public static void main(String[] args) {
        Random r = new Random(42);
        System.out.println(r.nextInt() + " " + r.nextInt());
        r = new Random(42);
        System.out.println(r.nextInt(100) + " " + r.nextInt(100) + " " + r.nextInt(100));
        r = new Random(42);            // a power-of-two bound takes the other branch
        System.out.println(r.nextInt(64) + " " + r.nextInt(64));
        r = new Random(0);
        System.out.println(r.nextInt());
        r = new Random(42);
        System.out.println(r.nextDouble() + " " + r.nextDouble());
        r = new Random(42);
        System.out.println(r.nextBoolean() + " " + r.nextBoolean() + " " + r.nextBoolean());
        r = new Random(42);
        System.out.println(r.nextLong());
        r = new Random(42);
        System.out.println(r.nextFloat());
        r = new Random(123456789L);
        for (int i = 0; i < 8; i++) System.out.print(r.nextInt(10) + " ");
        System.out.println();
        r = new Random(42);
        r.nextInt();
        r.setSeed(42);
        System.out.println(r.nextInt());
        // nextGaussian goes through Math.log, which can differ from the JDK's
        // StrictMath in the last ulp, so compare to 9 places.
        r = new Random(42);
        System.out.println(String.format("%.9f %.9f", r.nextGaussian(), r.nextGaussian()));
    }
}
"#
);

differential_test!(
    diff_c_style_array_declarators,
    "DiffCStyle",
    r#"
public class DiffCStyle {
    private int f[] = {1, 2};
    static int g[], h;                       // g is int[]; h is a plain int
    static int sum(int a[], int b) { return a[0] + b; }
    static String first(String s[]) { return s[0]; }
    public static void main(String args[]) {
        int x[] = {5, 6};
        int y[], z;                          // again: only y is an array
        y = new int[] {7};
        z = 9;
        g = new int[] {4};
        h = 1;
        int grid[][] = new int[2][3];
        grid[1][2] = 8;
        for (int e : x) System.out.print(e + " ");
        System.out.println();
        System.out.println(args.length + " " + x[1] + " " + y[0] + " " + z + " " + g[0] + " " + h);
        System.out.println(sum(x, 3) + " " + new DiffCStyle().f[1] + " " + grid[1][2]
            + " " + first(new String[] {"hi"}));
        System.out.println(x.length + " " + grid.length + " " + grid[0].length);
    }
}
"#
);

differential_test!(
    diff_bare_new_statement,
    "DiffBareNew",
    r#"
class Counter {
    static int made = 0;
    int id;
    Counter() { made++; id = made; System.out.println("ctor " + id); }
    Counter(int n) { made += n; System.out.println("ctor+" + n); }
    void go() { System.out.println("go " + id); }
}
public class DiffBareNew {
    public static void main(String[] args) {
        // A class instance creation is a statement expression (JLS 14.8): the
        // constructor runs for its effects and the reference is discarded.
        new Counter();
        new Counter();
        new Counter(5);
        new Counter().go();
        System.out.println("made=" + Counter.made);
        new String("ignored");
        StringBuilder sb = new StringBuilder("keep");
        new StringBuilder("dropped");
        System.out.println(sb.toString() + " " + sb.length());
        for (int i = 0; i < 2; i++) new Counter();
        if (Counter.made > 0) new Counter();
        System.out.println("made=" + Counter.made);
    }
}
"#
);

// Every `hasNextX` classification, over the tokens that separate Java's Scanner
// grammar from Rust's `from_str`: `nan`/`inf`/`infinity` are floats to Rust but
// not to Java, and `1.5f`/`0x10`/`1_000` are floats to neither.
differential_test_stdin!(
    diff_scanner_token_classification,
    "DiffScanTok",
    r#"
import java.util.Scanner;
public class DiffScanTok {
    public static void main(String[] args) {
        Scanner in = new Scanner(System.in);
        while (in.hasNext()) {
            String f = "";
            f += in.hasNextInt()     ? "I" : ".";
            f += in.hasNextLong()    ? "L" : ".";
            f += in.hasNextShort()   ? "S" : ".";
            f += in.hasNextByte()    ? "B" : ".";
            f += in.hasNextFloat()   ? "F" : ".";
            f += in.hasNextDouble()  ? "D" : ".";
            f += in.hasNextBoolean() ? "Z" : ".";
            System.out.println(f + "  " + in.next());
        }
    }
}
"#,
    "5 -5 +5 0 127 128 -128 -129 32767 32768 2147483647 2147483648 \
9223372036854775807 9223372036854775808 1.5 -1.5 1e5 1E5 .5 5. NaN Infinity -Infinity +NaN \
nan inf infinity INFINITY Nan 0x10 010 1_000 true TRUE False fALSe abc 1.5f 1.5d -0.0 1e400"
);

// Values, and the rule that makes the usual recovery loop work: a failed
// `nextX` does NOT consume the offending token.
differential_test_stdin!(
    diff_scanner_values_and_mismatch_recovery,
    "DiffScanVal",
    r#"
import java.util.Scanner;
import java.util.InputMismatchException;
import java.util.NoSuchElementException;
public class DiffScanVal {
    public static void main(String[] args) {
        Scanner in = new Scanner(System.in);
        System.out.println(in.nextLong() + " " + in.nextLong());
        System.out.println(in.nextFloat() + " " + in.nextFloat() + " " + in.nextFloat());
        System.out.println(in.nextDouble() + " " + in.nextDouble());
        System.out.println(in.nextShort() + " " + in.nextByte());
        System.out.println(in.nextBoolean() + " " + in.nextBoolean());

        // hasNextX peeks; it must not consume.
        System.out.println(in.hasNextInt() + " " + in.hasNextLong() + " " + in.hasNextFloat()
            + " " + in.hasNextShort() + " " + in.hasNextByte() + " " + in.hasNextBoolean());
        System.out.println("still: " + in.next());

        // A mismatch leaves the token in place, so `next()` sees it again.
        try { in.nextInt(); } catch (InputMismatchException e) { System.out.println("int mismatch"); }
        System.out.println("same token: " + in.next());
        // Out of range for byte, but a fine int.
        try { in.nextByte(); } catch (InputMismatchException e) { System.out.println("byte range"); }
        System.out.println("as int: " + in.nextInt());
        // Too big for int, fine for long.
        try { in.nextInt(); } catch (InputMismatchException e) { System.out.println("int range"); }
        System.out.println("as long: " + in.nextLong());

        System.out.println("line[" + in.nextLine() + "]");
        System.out.println("hasNext=" + in.hasNext() + " hasNextLine=" + in.hasNextLine());
        try { in.next(); } catch (NoSuchElementException e) { System.out.println("no element"); }
        try { in.nextLine(); } catch (NoSuchElementException e) { System.out.println("no line"); }
    }
}
"#,
    "9223372036854775807 -9223372036854775808\n\
1.5 -0.25 3\n\
1e5 -0.0\n\
-32768 -128\n\
TRUE false\n\
42 abc 999 2147483648 trailing\n"
);

// The overflow-throwing arithmetic. Every one of these must throw exactly at the
// boundary and nowhere else, and `Math.abs(MIN_VALUE)` must NOT throw.
// (`Math.absExact` is Java 15+, so it cannot appear here — see run_programs.)
differential_test!(
    diff_math_exact_overflow,
    "DiffExact",
    r#"
public class DiffExact {
    static void show(String label, int value) { System.out.println(label + " = " + value); }
    public static void main(String[] args) {
        int min = Integer.MIN_VALUE;
        int max = Integer.MAX_VALUE;
        try { show("addExact(max,1)", Math.addExact(max, 1)); }
        catch (ArithmeticException e) { System.out.println("addExact(max,1) ! " + e.getMessage()); }
        try { show("addExact(min,-1)", Math.addExact(min, -1)); }
        catch (ArithmeticException e) { System.out.println("addExact(min,-1) ! " + e.getMessage()); }
        show("addExact(max,0)", Math.addExact(max, 0));

        try { show("subExact(min,1)", Math.subtractExact(min, 1)); }
        catch (ArithmeticException e) { System.out.println("subExact(min,1) ! " + e.getMessage()); }
        try { show("subExact(max,-1)", Math.subtractExact(max, -1)); }
        catch (ArithmeticException e) { System.out.println("subExact(max,-1) ! " + e.getMessage()); }
        show("subExact(min,0)", Math.subtractExact(min, 0));

        try { show("mulExact(min,-1)", Math.multiplyExact(min, -1)); }
        catch (ArithmeticException e) { System.out.println("mulExact(min,-1) ! " + e.getMessage()); }
        try { show("mulExact(65536,65536)", Math.multiplyExact(65536, 65536)); }
        catch (ArithmeticException e) { System.out.println("mulExact(65536,65536) ! " + e.getMessage()); }
        show("mulExact(max,1)", Math.multiplyExact(max, 1));

        try { show("negExact(min)", Math.negateExact(min)); }
        catch (ArithmeticException e) { System.out.println("negExact(min) ! " + e.getMessage()); }
        show("negExact(max)", Math.negateExact(max));

        try { show("incExact(max)", Math.incrementExact(max)); }
        catch (ArithmeticException e) { System.out.println("incExact(max) ! " + e.getMessage()); }
        show("incExact(max-1)", Math.incrementExact(max - 1));

        try { show("decExact(min)", Math.decrementExact(min)); }
        catch (ArithmeticException e) { System.out.println("decExact(min) ! " + e.getMessage()); }
        show("decExact(min+1)", Math.decrementExact(min + 1));

        try { show("toIntExact(2^31)", Math.toIntExact(2147483648L)); }
        catch (ArithmeticException e) { System.out.println("toIntExact(2^31) ! " + e.getMessage()); }
        try { show("toIntExact(-2^31-1)", Math.toIntExact(-2147483649L)); }
        catch (ArithmeticException e) { System.out.println("toIntExact(-2^31-1) ! " + e.getMessage()); }
        show("toIntExact(max)", Math.toIntExact(2147483647L));
        show("toIntExact(min)", Math.toIntExact(-2147483648L));

        // Plain abs wraps at MIN_VALUE rather than throwing.
        show("abs(min)", Math.abs(min));
    }
}
"#
);

// `Field.getX`/`setX` widen the field's type but never narrow it, and refuse
// anything else with IllegalArgumentException. caturra used to return the raw
// value for every typed getter, so `getLong` on an int field produced an Int
// where the bytecode expected a Long.
differential_test!(
    diff_reflect_field_typed_accessors,
    "DiffFieldAcc",
    r#"
import java.lang.reflect.Field;
class Box {
    private int i = 7;
    private long l = 9000000000L;
    private double d = 2.5;
    private boolean b = true;
    private char c = 'A';
    private byte y = 3;
}
public class DiffFieldAcc {
    static Field f(String name) throws Exception {
        Field field = Box.class.getDeclaredField(name);
        field.setAccessible(true);
        return field;
    }
    public static void main(String[] args) throws Exception {
        Box box = new Box();
        // Widening reads.
        System.out.println(f("i").getInt(box) + " " + f("i").getLong(box) + " " + f("i").getDouble(box));
        System.out.println(f("l").getLong(box) + " " + f("l").getDouble(box));
        System.out.println(f("d").getDouble(box) + " " + f("b").getBoolean(box));
        System.out.println(f("c").getInt(box) + " " + f("y").getInt(box));
        // Narrowing or mismatched reads are refused.
        try { f("l").getInt(box); } catch (IllegalArgumentException e) { System.out.println("getInt(long) IAE"); }
        try { f("d").getLong(box); } catch (IllegalArgumentException e) { System.out.println("getLong(double) IAE"); }
        try { f("b").getInt(box); } catch (IllegalArgumentException e) { System.out.println("getInt(boolean) IAE"); }
        try { f("i").getBoolean(box); } catch (IllegalArgumentException e) { System.out.println("getBoolean(int) IAE"); }
        // Widening writes.
        f("i").setInt(box, 42);
        f("l").setInt(box, 5);
        f("d").setLong(box, 6L);
        f("b").setBoolean(box, false);
        System.out.println(f("i").getInt(box) + " " + f("l").getLong(box)
            + " " + f("d").getDouble(box) + " " + f("b").getBoolean(box));
        f("d").setInt(box, 5);
        System.out.println(f("d").getDouble(box));
        // Narrowing writes are refused.
        try { f("i").setLong(box, 5L); } catch (IllegalArgumentException e) { System.out.println("setLong(int) IAE"); }
        try { f("i").setDouble(box, 1.5); } catch (IllegalArgumentException e) { System.out.println("setDouble(int) IAE"); }
        try { f("i").setBoolean(box, true); } catch (IllegalArgumentException e) { System.out.println("setBoolean(int) IAE"); }
        // `get` still boxes, as in Java.
        System.out.println(f("i").get(box));
    }
}
"#
);

differential_test!(
    diff_string_builder_mutators,
    "DiffBuilderMutate",
    r#"
public class DiffBuilderMutate {
    public static void main(String[] args) {
        StringBuilder b = new StringBuilder("Hello");
        b.append(", ").append("world").append('!');
        b.append(1).append(2L).append(3.5).append(0.5f).append(true);
        b.append(new char[] {'[', ']'});
        System.out.println(b);

        StringBuilder c = new StringBuilder("abcdef");
        System.out.println(c.reverse());
        System.out.println(c.insert(0, "<").insert(c.length(), ">"));
        System.out.println(c.insert(1, 42).insert(1, 'x').insert(1, true));
        System.out.println(c.insert(0, 1.5).insert(0, 2L).insert(0, new char[] {'-'}));
        System.out.println(c.delete(1, 3));
        System.out.println(c.deleteCharAt(0));
        System.out.println(c.replace(0, 2, "XY"));
        System.out.println(c.replace(2, 1000, "!"));

        c.setCharAt(0, 'z');
        System.out.println(c);
        c.setLength(2);
        System.out.println(c + " " + c.length());
        c.setLength(4);
        System.out.println(c.length() + " " + (int) c.charAt(3));

        StringBuilder d = new StringBuilder(64);
        d.ensureCapacity(200);
        d.append("hi");
        d.trimToSize();
        System.out.println(d + " " + d.length());
    }
}
"#
);

differential_test!(
    diff_string_builder_search_and_extract,
    "DiffBuilderRead",
    r#"
public class DiffBuilderRead {
    public static void main(String[] args) {
        StringBuilder b = new StringBuilder("hello world, hello");
        System.out.println(b.length() + " " + b.charAt(1));
        System.out.println(b.indexOf("hello") + " " + b.indexOf("hello", 1));
        System.out.println(b.lastIndexOf("hello") + " " + b.lastIndexOf("hello", 5));
        System.out.println(b.indexOf("zzz") + " " + b.lastIndexOf("zzz"));
        System.out.println(b.substring(6) + "|" + b.substring(0, 5) + "|" + b.subSequence(2, 4));

        char[] dest = new char[7];
        for (int i = 0; i < dest.length; i++) {
            dest[i] = '.';
        }
        b.getChars(0, 5, dest, 1);
        for (int i = 0; i < dest.length; i++) {
            System.out.print(dest[i]);
        }
        System.out.println();

        System.out.println(new StringBuilder("abc").compareTo(new StringBuilder("abd")));
        System.out.println(new StringBuilder("abd").compareTo(new StringBuilder("abc")));
        System.out.println(new StringBuilder("abc").compareTo(new StringBuilder("abc")));
        System.out.println(new StringBuilder("abc").compareTo(new StringBuilder("ab")));
    }
}
"#
);

differential_test!(
    diff_string_builder_code_points_and_errors,
    "DiffBuilderEdges",
    r#"
public class DiffBuilderEdges {
    public static void main(String[] args) {
        // A supplementary character is two code units but one code point,
        // and reverse() must not split the surrogate pair.
        StringBuilder b = new StringBuilder("a");
        b.appendCodePoint(0x1F600);
        b.append('z');
        System.out.println(b.length() + " " + b.codePointCount(0, b.length()));
        System.out.println(b.codePointAt(1) + " " + b.codePointBefore(3));
        System.out.println(b.offsetByCodePoints(0, 2) + " " + b.offsetByCodePoints(4, -2));
        StringBuilder reversed = new StringBuilder(b.toString()).reverse();
        System.out.println(reversed.codePointAt(1) + " " + reversed.charAt(0));

        StringBuilder e = new StringBuilder();
        try { e.charAt(0); } catch (StringIndexOutOfBoundsException x) { System.out.println("charAt"); }
        try { e.deleteCharAt(0); } catch (StringIndexOutOfBoundsException x) { System.out.println("deleteCharAt"); }
        try { e.insert(1, "x"); } catch (StringIndexOutOfBoundsException x) { System.out.println("insert"); }
        try { e.setLength(-1); } catch (StringIndexOutOfBoundsException x) { System.out.println("setLength"); }
        try { e.setCharAt(0, 'a'); } catch (StringIndexOutOfBoundsException x) { System.out.println("setCharAt"); }
        try { e.delete(1, 2); } catch (StringIndexOutOfBoundsException x) { System.out.println("delete"); }
        try { e.replace(2, 1, "x"); } catch (StringIndexOutOfBoundsException x) { System.out.println("replace"); }
        try { e.substring(1); } catch (StringIndexOutOfBoundsException x) { System.out.println("substring"); }
        try { e.appendCodePoint(-1); } catch (IllegalArgumentException x) { System.out.println("appendCodePoint"); }

        // delete/replace tolerate an end past the length; substring does not.
        System.out.println(new StringBuilder("hello").delete(2, 99));
        System.out.println(new StringBuilder("hello").replace(1, 99, "-"));
    }
}
"#
);

differential_test!(
    diff_hash_map_iteration_order,
    "DiffMapOrder",
    r#"
import java.util.HashMap;

public class DiffMapOrder {
    public static void main(String[] args) {
        // Enough keys to force two resizes, so the final table length —
        // and therefore the bucket order — depends on the growth history.
        String[] words = ("the quick brown fox jumps over a lazy dog and then some more "
                + "words alpha beta gamma delta epsilon zeta eta theta iota kappa lambda "
                + "mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega").split(" ");
        HashMap<String, Integer> counts = new HashMap<String, Integer>();
        for (int i = 0; i < words.length; i++) {
            counts.put(words[i], i);
        }
        System.out.println(counts);
        System.out.println(counts.size());

        // Removal never shrinks the table, so what remains keeps its order.
        counts.remove("fox");
        counts.remove("alpha");
        counts.remove("omega");
        System.out.println(counts);

        // Re-putting an existing key keeps its position.
        counts.put("the", 99);
        System.out.println(counts);

        HashMap<Integer, String> byInt = new HashMap<Integer, String>();
        for (int i = 40; i >= 0; i -= 3) {
            byInt.put(i, "v" + i);
        }
        System.out.println(byInt);

        // -0.0 and 0.0 are distinct keys: Double.equals compares raw bits.
        HashMap<Double, String> byDouble = new HashMap<Double, String>();
        byDouble.put(1.5, "a");
        byDouble.put(-0.0, "b");
        byDouble.put(0.0, "c");
        byDouble.put(2.25, "d");
        byDouble.put(100.0, "e");
        System.out.println(byDouble);
        System.out.println(byDouble.get(-0.0) + " " + byDouble.get(0.0));

        // An initial-capacity hint changes the table length, hence the order.
        HashMap<String, Integer> sized = new HashMap<String, Integer>(4);
        for (int i = 0; i < 8; i++) {
            sized.put("k" + i, i);
        }
        System.out.println(sized);

        // A copy pre-sizes its own table, so its order may differ.
        HashMap<String, Integer> copy = new HashMap<String, Integer>(counts);
        System.out.println(copy);
        System.out.println(copy.equals(counts));
    }
}
"#
);

differential_test!(
    diff_hash_map_core_methods,
    "DiffMapCore",
    r#"
import java.util.HashMap;
import java.util.Map;

public class DiffMapCore {
    public static void main(String[] args) {
        Map<String, Integer> m = new HashMap<>();
        System.out.println(m.isEmpty() + " " + m.size() + " " + m);
        System.out.println(m.put("a", 1) + " " + m.put("a", 2));
        m.put("b", 3);
        System.out.println(m.get("a") + " " + m.get("zz"));
        System.out.println(m.containsKey("a") + " " + m.containsKey("zz"));
        System.out.println(m.containsValue(3) + " " + m.containsValue(99));
        System.out.println(m.getOrDefault("a", -1) + " " + m.getOrDefault("zz", -1));
        System.out.println(m.putIfAbsent("a", 50) + " " + m.putIfAbsent("c", 7) + " " + m);
        System.out.println(m.replace("a", 20) + " " + m.replace("zz", 1));
        System.out.println(m.replace("b", 3, 30) + " " + m.replace("b", 999, 40) + " " + m);
        System.out.println(m.remove("c") + " " + m.remove("zz"));
        System.out.println(m.remove("b", 999) + " " + m.remove("b", 30) + " " + m);

        Map<String, Integer> other = new HashMap<>();
        other.put("a", 20);
        System.out.println(m.equals(other) + " " + (m.hashCode() == other.hashCode()));
        other.put("q", 1);
        System.out.println(m.equals(other));

        Map<String, Integer> all = new HashMap<>();
        all.putAll(other);
        System.out.println(all + " " + all.size());
        all.clear();
        System.out.println(all + " " + all.isEmpty());
    }
}
"#
);

differential_test!(
    diff_hash_map_null_and_unboxing,
    "DiffMapNull",
    r#"
import java.util.HashMap;

public class DiffMapNull {
    public static void main(String[] args) {
        HashMap<String, Double> sentiment = new HashMap<String, Double>();
        sentiment.put("good", 1.5);

        // A missing key is null, and printing it prints "null".
        System.out.println(sentiment.get("nope"));

        // Unboxing that null throws, which is what the Review Lab relies on.
        try {
            double bad = sentiment.get("nope");
            System.out.println(bad);
        } catch (NullPointerException e) {
            System.out.println("npe on missing");
        }

        // But `put` as a statement discards a null previous value silently,
        // while reading that value unboxes it.
        sentiment.put("fresh", 2.0);
        try {
            double previous = sentiment.put("brand-new", 3.0);
            System.out.println(previous);
        } catch (NullPointerException e) {
            System.out.println("npe on new key");
        }
        System.out.println(sentiment.size());

        // A null value is a legal mapping, and distinct from an absent key.
        HashMap<String, String> names = new HashMap<String, String>();
        names.put("here", null);
        System.out.println(names.get("here") + " " + names.get("gone"));
        System.out.println(names.containsKey("here") + " " + names.containsKey("gone"));
        System.out.println(names.putIfAbsent("here", "now") + " " + names);

        // A null key is legal too, and always lands in bucket 0.
        HashMap<String, Integer> withNull = new HashMap<String, Integer>();
        withNull.put("z", 1);
        withNull.put(null, 0);
        System.out.println(withNull + " " + withNull.get(null));
    }
}
"#
);

differential_test!(
    diff_hash_map_views,
    "DiffMapViews",
    r#"
import java.util.Collection;
import java.util.HashMap;
import java.util.Map;
import java.util.Set;

public class DiffMapViews {
    public static void main(String[] args) {
        Map<String, Integer> counts = new HashMap<>();
        counts.put("pear", 4);
        counts.put("apple", 1);
        counts.put("fig", 9);
        counts.put("quince", 16);

        System.out.println(counts.keySet());
        System.out.println(counts.values());
        System.out.println(counts.entrySet());

        // The three views iterate in the map's order.
        for (String key : counts.keySet()) {
            System.out.print(key + " ");
        }
        System.out.println();
        int total = 0;
        for (int value : counts.values()) {
            total += value;
        }
        System.out.println(total);
        for (Map.Entry<String, Integer> entry : counts.entrySet()) {
            System.out.print(entry.getKey() + "=" + entry.getValue() + ";");
        }
        System.out.println();

        Set<String> keys = counts.keySet();
        Collection<Integer> values = counts.values();
        System.out.println(keys.size() + " " + keys.contains("fig") + " " + keys.contains("zzz"));
        System.out.println(values.size() + " " + values.contains(9) + " " + values.isEmpty());

        // A view is live, not a copy.
        counts.put("kiwi", 2);
        System.out.println(keys + " " + keys.size());

        // An entry is a view too: setValue writes through to the map.
        for (Map.Entry<String, Integer> entry : counts.entrySet()) {
            entry.setValue(entry.getValue() * 10);
        }
        System.out.println(counts);

        // Iterating a map with a primitive key unboxes the loop variable.
        Map<Integer, String> byInt = new HashMap<>();
        byInt.put(7, "seven");
        byInt.put(3, "three");
        int keySum = 0;
        for (int key : byInt.keySet()) {
            keySum += key;
        }
        System.out.println(keySum);
        for (String value : byInt.values()) {
            System.out.print(value + ".");
        }
        System.out.println();

        Map<String, Integer> empty = new HashMap<>();
        System.out.println(empty.keySet() + " " + empty.values() + " " + empty.entrySet());
        for (String key : empty.keySet()) {
            System.out.println("unreachable " + key);
        }
        System.out.println(empty.keySet().isEmpty());
    }
}
"#
);

differential_test!(
    diff_to_string_inside_collections,
    "DiffToString",
    r#"
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;

class Student {
    String name;
    Student(String name) { this.name = name; }
    public String toString() { return "S(" + name + ")"; }
}

class Plain {
    int x = 7;
}

public class DiffToString {
    public static void main(String[] args) {
        Student ann = new Student("Ann");
        ArrayList<Student> list = new ArrayList<Student>();
        list.add(ann);
        list.add(new Student("Bo"));

        // A container renders its elements with their own toString.
        System.out.println(list);
        System.out.println("concat: " + list);
        System.out.println(String.format("%s", list));

        Map<String, Student> byName = new HashMap<String, Student>();
        byName.put("a", ann);
        System.out.println(byName);
        System.out.println(byName.values());
        System.out.println(byName.entrySet());

        Map<Student, String> byStudent = new HashMap<Student, String>();
        byStudent.put(ann, "x");
        System.out.println(byStudent.keySet());

        // StringBuilder.append(Object) too.
        StringBuilder sb = new StringBuilder();
        sb.append(ann).append('|').append(list);
        System.out.println(sb);

        // A class without toString keeps Java's default form (the identity
        // hash is arbitrary, so only its shape is comparable).
        ArrayList<Plain> plains = new ArrayList<Plain>();
        plains.add(new Plain());
        String rendered = plains.toString();
        System.out.println(rendered.startsWith("[Plain@") && rendered.endsWith("]"));

        // Nulls and empties.
        ArrayList<Student> holes = new ArrayList<Student>();
        holes.add(null);
        System.out.println(holes + " " + new ArrayList<Student>());

        // Inherited toString.
        ArrayList<Object> mixed = new ArrayList<Object>();
        mixed.add(ann);
        mixed.add("plain string");
        System.out.println(mixed);
    }
}
"#
);

differential_test!(
    diff_to_string_edge_cases,
    "DiffToStringEdges",
    r#"
import java.util.ArrayList;

class Boom {
    public String toString() { throw new IllegalStateException("boom"); }
}

class Nully {
    public String toString() { return null; }
}

class Node {
    int n;
    Node next;
    Node(int n) { this.n = n; }
    public String toString() { return next == null ? "" + n : n + "->" + next; }
}

class Loop {
    public String toString() { return "" + this; }
}

public class DiffToStringEdges {
    public static void main(String[] args) {
        // A collection holding itself does not recurse.
        ArrayList<Object> self = new ArrayList<Object>();
        self.add(self);
        System.out.println(self);
        ArrayList<Object> outer = new ArrayList<Object>();
        outer.add("x");
        outer.add(self);
        System.out.println(outer);

        // An exception thrown by toString propagates to the caller.
        ArrayList<Boom> boom = new ArrayList<Boom>();
        boom.add(new Boom());
        try {
            System.out.println(boom);
        } catch (IllegalStateException e) {
            System.out.println("caught " + e.getMessage());
        }

        // A toString returning null renders as "null", it does not throw.
        ArrayList<Nully> nully = new ArrayList<Nully>();
        nully.add(new Nully());
        System.out.println(nully);

        // A toString that calls another object's toString.
        Node head = new Node(1);
        head.next = new Node(2);
        ArrayList<Node> nodes = new ArrayList<Node>();
        nodes.add(head);
        System.out.println(nodes);

        // Two collections holding each other overflow the stack, as in Java.
        ArrayList<Object> a = new ArrayList<Object>();
        ArrayList<Object> b = new ArrayList<Object>();
        a.add(b);
        b.add(a);
        try {
            System.out.println(a);
        } catch (StackOverflowError e) {
            System.out.println("indirect cycle overflows");
        }

        // So does a toString that renders itself.
        ArrayList<Loop> loop = new ArrayList<Loop>();
        loop.add(new Loop());
        try {
            System.out.println(loop);
        } catch (StackOverflowError e) {
            System.out.println("self-rendering toString overflows");
        }
        System.out.println("still running");
    }
}
"#
);

differential_test!(
    diff_collections_sort_uses_compare_to,
    "DiffSort",
    r#"
import java.util.ArrayList;
import java.util.Collections;

class Card implements Comparable<Card> {
    int rank;
    String tag;
    Card(int rank, String tag) { this.rank = rank; this.tag = tag; }
    public int compareTo(Card other) { return rank - other.rank; }
    public String toString() { return "C" + rank + tag; }
}

class Reversed implements Comparable<Reversed> {
    int n;
    Reversed(int n) { this.n = n; }
    public int compareTo(Reversed other) { return other.n - n; }
    public String toString() { return "R" + n; }
}

public class DiffSort {
    public static void main(String[] args) {
        // A user class sorts by its own compareTo, and ties keep their
        // original order: Collections.sort is stable.
        ArrayList<Card> cards = new ArrayList<Card>();
        cards.add(new Card(3, "a"));
        cards.add(new Card(1, "b"));
        cards.add(new Card(2, "c"));
        cards.add(new Card(1, "d"));
        cards.add(new Card(1, "e"));
        Collections.sort(cards);
        System.out.println(cards);

        ArrayList<Reversed> reversed = new ArrayList<Reversed>();
        reversed.add(new Reversed(1));
        reversed.add(new Reversed(3));
        reversed.add(new Reversed(2));
        Collections.sort(reversed);
        System.out.println(reversed);

        ArrayList<String> words = new ArrayList<String>();
        words.add("pear");
        words.add("apple");
        words.add("fig");
        Collections.sort(words);
        System.out.println(words);

        ArrayList<Integer> numbers = new ArrayList<Integer>();
        numbers.add(3);
        numbers.add(-1);
        numbers.add(2);
        Collections.sort(numbers);
        System.out.println(numbers);

        // Degenerate sizes.
        ArrayList<Card> one = new ArrayList<Card>();
        one.add(new Card(5, "z"));
        Collections.sort(one);
        System.out.println(one + " " + new ArrayList<Card>());
    }
}
"#
);

differential_test!(
    diff_user_equals_in_collections,
    "DiffEquals",
    r#"
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;

class Point {
    int x;
    int y;
    Point(int x, int y) { this.x = x; this.y = y; }
    public boolean equals(Object other) {
        if (!(other instanceof Point)) return false;
        Point that = (Point) other;
        return x == that.x && y == that.y;
    }
    public int hashCode() { return 31 * x + y; }
    public String toString() { return "(" + x + "," + y + ")"; }
}

// equals without hashCode: two equal keys land in different buckets, so the
// map keeps both — a real JVM does the same.
class Broken {
    int id;
    Broken(int id) { this.id = id; }
    public boolean equals(Object other) {
        return other instanceof Broken && ((Broken) other).id == id;
    }
    public String toString() { return "B" + id; }
}

public class DiffEquals {
    public static void main(String[] args) {
        Point a = new Point(1, 2);
        Point b = new Point(1, 2);
        Point c = new Point(3, 4);

        ArrayList<Point> list = new ArrayList<Point>();
        list.add(a);
        list.add(c);
        list.add(new Point(1, 2));
        System.out.println(list.contains(b) + " " + list.contains(new Point(9, 9)));
        System.out.println(list.indexOf(b) + " " + list.lastIndexOf(b));
        System.out.println(list.remove(b) + " " + list);

        ArrayList<Point> one = new ArrayList<Point>();
        one.add(new Point(1, 2));
        ArrayList<Point> two = new ArrayList<Point>();
        two.add(new Point(1, 2));
        System.out.println(one.equals(two) + " " + (one.hashCode() == two.hashCode()));
        two.add(c);
        System.out.println(one.equals(two));

        // A map's iteration order follows the user's hashCode, across a resize.
        Map<Point, Integer> byPoint = new HashMap<Point, Integer>();
        for (int i = 0; i < 12; i++) {
            byPoint.put(new Point(i, i), i);
        }
        System.out.println(byPoint);
        System.out.println(byPoint.keySet());
        System.out.println(byPoint.get(new Point(5, 5)) + " " + byPoint.containsKey(new Point(99, 99)));
        System.out.println(byPoint.remove(new Point(3, 3)) + " " + byPoint.size());

        // Re-putting an equal key replaces the value and keeps one entry.
        Map<Point, String> single = new HashMap<Point, String>();
        single.put(a, "first");
        single.put(b, "second");
        System.out.println(single.size() + " " + single + " " + single.get(new Point(1, 2)));
        System.out.println(single.containsValue("second") + " " + single.containsValue("first"));

        // Map equality and hashCode go through the elements' own.
        Map<Point, Integer> left = new HashMap<Point, Integer>();
        left.put(new Point(1, 1), 10);
        Map<Point, Integer> right = new HashMap<Point, Integer>();
        right.put(new Point(1, 1), 10);
        System.out.println(left.equals(right) + " " + (left.hashCode() == right.hashCode()));

        // equals without hashCode: the key cannot be found again.
        Map<Broken, String> broken = new HashMap<Broken, String>();
        broken.put(new Broken(1), "x");
        System.out.println(broken.get(new Broken(1)) + " " + broken.containsKey(new Broken(1)));
        broken.put(new Broken(1), "y");
        System.out.println(broken.size());

        // But a list, which only uses equals, finds it.
        ArrayList<Broken> brokenList = new ArrayList<Broken>();
        brokenList.add(new Broken(1));
        System.out.println(brokenList.contains(new Broken(1)));
    }
}
"#
);

differential_test!(
    diff_equals_edge_cases,
    "DiffEqualsEdges",
    r#"
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;

class Boom {
    public boolean equals(Object other) { throw new IllegalStateException("eq"); }
    public int hashCode() { return 1; }
}

public class DiffEqualsEdges {
    public static void main(String[] args) {
        // Double.equals compares raw bits: -0.0 differs from 0.0, NaN equals
        // itself. `==` says the opposite of both.
        Map<Double, String> byDouble = new HashMap<Double, String>();
        byDouble.put(0.0, "pos");
        byDouble.put(-0.0, "neg");
        byDouble.put(Double.NaN, "nan");
        System.out.println(byDouble.size() + " " + byDouble.get(Double.NaN) + " " + byDouble.get(-0.0));

        ArrayList<Double> doubles = new ArrayList<Double>();
        doubles.add(Double.NaN);
        doubles.add(0.0);
        System.out.println(doubles.contains(Double.NaN) + " " + doubles.contains(-0.0));

        // Boolean hashes to 1231/1237, not to 0/1.
        Map<Boolean, String> flags = new HashMap<Boolean, String>();
        flags.put(true, "t");
        flags.put(false, "f");
        System.out.println(flags);

        // Character and Integer keys.
        Map<Character, Integer> letters = new HashMap<Character, Integer>();
        letters.put('a', 1);
        letters.put('z', 2);
        System.out.println(letters + " " + letters.get('z'));

        // A null key still maps, and a null element still compares.
        Map<String, Integer> withNull = new HashMap<String, Integer>();
        withNull.put(null, 0);
        withNull.put("a", 1);
        System.out.println(withNull.get(null) + " " + withNull.containsKey(null));
        ArrayList<String> holes = new ArrayList<String>();
        holes.add(null);
        holes.add("x");
        System.out.println(holes.contains(null) + " " + holes.indexOf(null) + " " + holes.remove(null));

        // An exception from equals reaches the caller's catch.
        ArrayList<Boom> boom = new ArrayList<Boom>();
        boom.add(new Boom());
        try {
            boom.contains(new Boom());
            System.out.println("no throw");
        } catch (IllegalStateException e) {
            System.out.println("caught " + e.getMessage());
        }

        // Strings and boxed wrappers still compare by value.
        ArrayList<String> words = new ArrayList<String>();
        words.add("pear");
        System.out.println(words.contains("pe" + "ar") + " " + words.indexOf("fig"));
        ArrayList<Integer> numbers = new ArrayList<Integer>();
        numbers.add(7);
        System.out.println(numbers.contains(7) + " " + numbers.contains(8));
    }
}
"#
);

differential_test!(
    diff_hash_map_colliding_bins_resize,
    "DiffMapCollide",
    r#"
import java.util.HashMap;
import java.util.Map;

// Every key lands in the same bucket of a 16- or 32-long table, and the keys
// only spread out once the table reaches 64. Staying under 8 per bin there
// keeps us clear of the treeified bins caturra does not model.
class Collides {
    int id;
    Collides(int id) { this.id = id; }
    public boolean equals(Object other) {
        return other instanceof Collides && ((Collides) other).id == id;
    }
    public int hashCode() { return id * 32; }
    public String toString() { return "c" + id; }
}

public class DiffMapCollide {
    public static void main(String[] args) {
        // A bin of 8 makes Java resize rather than treeify while the table is
        // under 64 long, which reshuffles every bucket.
        for (int count = 6; count <= 14; count++) {
            Map<Collides, Integer> map = new HashMap<Collides, Integer>();
            for (int i = 0; i < count; i++) {
                map.put(new Collides(i), i);
            }
            System.out.println(count + ": " + map.keySet());
        }

        // The same, reached through a copy constructor's pre-sized table.
        Map<Collides, Integer> source = new HashMap<Collides, Integer>();
        for (int i = 0; i < 10; i++) {
            source.put(new Collides(i), i);
        }
        System.out.println(new HashMap<Collides, Integer>(source).keySet());

        // And through an explicit initial capacity.
        Map<Collides, Integer> sized = new HashMap<Collides, Integer>(4);
        for (int i = 0; i < 10; i++) {
            sized.put(new Collides(i), i);
        }
        System.out.println(sized.keySet());
    }
}
"#
);

differential_test!(
    diff_compound_assignment_unboxes,
    "DiffCompoundBoxing",
    r#"
import java.util.ArrayList;
import java.util.HashMap;
import java.util.Map;

public class DiffCompoundBoxing {
    static Integer field = 10;
    static Double doubleField = 1.0;

    public static void main(String[] args) {
        // `total += map.get(k)` unboxes the wrapper the map hands back.
        Map<String, Double> weights = new HashMap<String, Double>();
        weights.put("a", 1.5);
        weights.put("b", 2.5);
        double total = 0;
        for (String key : weights.keySet()) {
            total += weights.get(key);
        }
        System.out.println(total);

        ArrayList<Integer> numbers = new ArrayList<Integer>();
        numbers.add(3);
        int sum = 0;
        sum += numbers.get(0);
        Integer boxed = 4;
        sum += boxed;
        System.out.println(sum);

        // A wrapper target unboxes, operates, and boxes back.
        Integer counter = 5;
        counter += 1;
        counter++;
        ++counter;
        counter--;
        System.out.println(counter + " " + (counter instanceof Integer));

        Long big = 5L;
        big += 1;
        System.out.println(big);

        Double half = 1.5;
        half += 1;
        System.out.println(half);

        // Fields and array elements too.
        field += 5;
        doubleField += 0.5;
        System.out.println(field + " " + doubleField);

        int[] cells = {1, 2};
        Integer step = 3;
        cells[0] += step;
        System.out.println(cells[0]);

        // The implicit narrowing cast still applies.
        char letter = 'a';
        Integer one = 1;
        letter += one;
        System.out.println(letter);
    }
}
"#
);

differential_test!(
    diff_arrays_equals_and_hash_code,
    "DiffArraysEquals",
    r#"
import java.util.Arrays;

class Point {
    int x;
    Point(int x) { this.x = x; }
    public boolean equals(Object other) {
        return other instanceof Point && ((Point) other).x == x;
    }
    public int hashCode() { return x; }
    public String toString() { return "P" + x; }
}

// equals without hashCode: Arrays.equals still finds them equal, but
// Arrays.hashCode does not agree, exactly as on a real JVM.
class Loose {
    int x;
    Loose(int x) { this.x = x; }
    public boolean equals(Object other) {
        return other instanceof Loose && ((Loose) other).x == x;
    }
}

public class DiffArraysEquals {
    public static void main(String[] args) {
        int[] ints = {1, 2};
        int[] sameInts = {1, 2};
        int[] otherInts = {1, 3};
        System.out.println(Arrays.equals(ints, sameInts) + " " + Arrays.equals(ints, otherInts));
        System.out.println(Arrays.equals(ints, new int[] {1}) + " " + Arrays.equals(new int[0], new int[0]));

        // A reference array asks each element's own equals, null-safely.
        String[] words = {"a", null};
        String[] sameWords = {"a", null};
        System.out.println(Arrays.equals(words, sameWords));
        Point[] points = {new Point(1), new Point(2)};
        Point[] samePoints = {new Point(1), new Point(2)};
        System.out.println(Arrays.equals(points, samePoints));
        System.out.println(Arrays.hashCode(points) == Arrays.hashCode(samePoints));

        Loose[] loose = {new Loose(1)};
        Loose[] sameLoose = {new Loose(1)};
        System.out.println(Arrays.equals(loose, sameLoose) + " "
                + (Arrays.hashCode(loose) == Arrays.hashCode(sameLoose)));

        // Doubles and floats compare raw bits: NaN equals itself, -0.0 does
        // not equal 0.0. `==` says the opposite of both.
        double[] nan = {Double.NaN};
        double[] alsoNan = {Double.NaN};
        System.out.println(Arrays.equals(nan, alsoNan) + " " + (Double.NaN == Double.NaN));
        double[] positiveZero = {0.0};
        double[] negativeZero = {-0.0};
        System.out.println(Arrays.equals(positiveZero, negativeZero) + " " + (0.0 == -0.0));
        float[] nanFloat = {Float.NaN};
        float[] alsoNanFloat = {Float.NaN};
        System.out.println(Arrays.equals(nanFloat, alsoNanFloat));

        // Every hash code, exactly.
        System.out.println(Arrays.hashCode(ints) + " " + Arrays.hashCode(new int[0]));
        System.out.println(Arrays.hashCode(nan) + " " + Arrays.hashCode(positiveZero)
                + " " + Arrays.hashCode(negativeZero));
        System.out.println(Arrays.hashCode(new boolean[] {true, false}));
        System.out.println(Arrays.hashCode(new long[] {5L}) + " " + Arrays.hashCode(new char[] {'a'}));
        System.out.println(Arrays.hashCode(words));

        // A null array.
        int[] none = null;
        System.out.println(Arrays.equals(none, none) + " " + Arrays.equals(none, ints)
                + " " + Arrays.hashCode(none));

        // An element that is itself an array compares by identity.
        int[] inner = {1};
        Object[] holding = {inner};
        Object[] holdingSame = {inner};
        Object[] holdingCopy = {new int[] {1}};
        System.out.println(Arrays.equals(holding, holdingSame) + " " + Arrays.equals(holding, holdingCopy));
    }
}
"#
);

differential_test!(
    diff_arrays_sort_uses_compare_to,
    "DiffArraysSort",
    r#"
import java.util.Arrays;

class Card implements Comparable<Card> {
    int rank;
    String tag;
    Card(int rank, String tag) { this.rank = rank; this.tag = tag; }
    public int compareTo(Card other) { return rank - other.rank; }
    public String toString() { return "C" + rank + tag; }
}

public class DiffArraysSort {
    public static void main(String[] args) {
        // A reference array sorts by its elements' compareTo, and ties keep
        // their original order: Arrays.sort of a reference array is stable.
        Card[] cards = {
            new Card(3, "a"), new Card(1, "b"), new Card(2, "c"),
            new Card(1, "d"), new Card(1, "e"),
        };
        Arrays.sort(cards);
        System.out.println(Arrays.toString(cards));

        Card[] one = {new Card(5, "z")};
        Arrays.sort(one);
        System.out.println(Arrays.toString(one) + " " + Arrays.toString(new Card[0]));

        // The primitive and String overloads still work.
        int[] numbers = {3, 1, 2};
        Arrays.sort(numbers);
        System.out.println(Arrays.toString(numbers));
        String[] words = {"pear", "fig", "kiwi"};
        Arrays.sort(words);
        System.out.println(Arrays.toString(words));
    }
}
"#
);

differential_test!(
    diff_array_initializer_trailing_comma,
    "DiffTrailingComma",
    r#"
public class DiffTrailingComma {
    public static void main(String[] args) {
        // A trailing comma is legal in an array initializer (JLS §10.6).
        int[] numbers = {1, 2,};
        int[] empty = {};
        String[] words = {"a",};
        int[][] grid = {
            {1, 2,},
            {3, 4,},
        };
        System.out.println(numbers.length + " " + empty.length + " " + words[0]);
        System.out.println(grid.length + " " + grid[1][1]);
        int[] fromNew = new int[] {5, 6,};
        System.out.println(fromNew[1]);
    }
}
"#
);

differential_test!(
    diff_arrays_deep_operations,
    "DiffArraysDeep",
    r#"
import java.util.Arrays;

class Point {
    int x;
    Point(int x) { this.x = x; }
    public boolean equals(Object other) {
        return other instanceof Point && ((Point) other).x == x;
    }
    public int hashCode() { return x; }
    public String toString() { return "P" + x; }
}

public class DiffArraysDeep {
    public static void main(String[] args) {
        // A row's element type survives erasure: a boolean[] prints `true`,
        // a char[] prints its characters, an int[] prints numbers.
        int[][] ints = {{1, 2}, {3}};
        boolean[][] flags = {{true, false}};
        char[][] letters = {{'a', 'b'}};
        double[][] doubles = {{1.5, Double.NaN}};
        float[][] floats = {{0.5f}};
        long[][] longs = {{5L}};
        short[][] shorts = {{(short) 3}};
        byte[][] bytes = {{(byte) 4}};
        String[][] words = {{"x", null}};
        Point[][] points = {{new Point(1)}};

        System.out.println(Arrays.deepToString(ints));
        System.out.println(Arrays.deepToString(flags));
        System.out.println(Arrays.deepToString(letters));
        System.out.println(Arrays.deepToString(doubles));
        System.out.println(Arrays.deepToString(floats));
        System.out.println(Arrays.deepToString(longs));
        System.out.println(Arrays.deepToString(shorts) + " " + Arrays.deepToString(bytes));
        System.out.println(Arrays.deepToString(words));
        System.out.println(Arrays.deepToString(points));

        // deepEquals compares rows element-wise; equals compares them by
        // identity, so it says false for equal-but-distinct rows.
        int[][] sameInts = {{1, 2}, {3}};
        System.out.println(Arrays.deepEquals(ints, sameInts) + " " + Arrays.equals(ints, sameInts));
        System.out.println(Arrays.deepEquals(ints, new int[][] {{1, 2}, {4}}));
        System.out.println(Arrays.deepEquals(points, new Point[][] {{new Point(1)}}));
        System.out.println(Arrays.deepEquals(flags, new boolean[][] {{true, false}}));

        // A boolean[] never equals an int[] of the same shape.
        Object[] asBooleans = {new boolean[] {true}};
        Object[] asInts = {new int[] {1}};
        System.out.println(Arrays.deepEquals(asBooleans, asInts));

        // deepHashCode agrees with deepEquals, exactly.
        System.out.println(Arrays.deepHashCode(ints) + " " + Arrays.deepHashCode(sameInts));
        System.out.println(Arrays.deepHashCode(flags) + " " + Arrays.deepHashCode(letters));
        System.out.println(Arrays.deepHashCode(doubles) + " " + Arrays.deepHashCode(longs));
        System.out.println(Arrays.deepHashCode(words) + " " + Arrays.deepHashCode(points));

        // Ragged nesting, nulls, and a null array.
        Object[] mixed = {new int[] {1}, "s", null, new Object[] {new int[] {2}}};
        System.out.println(Arrays.deepToString(mixed) + " " + Arrays.deepHashCode(mixed));
        System.out.println(Arrays.deepToString(null) + " " + Arrays.deepHashCode(null)
                + " " + Arrays.deepEquals(null, null) + " " + Arrays.deepEquals(null, ints));
        System.out.println(Arrays.deepToString(new Object[0]) + " " + Arrays.deepHashCode(new Object[0]));

        // An array that holds itself renders as [...] rather than recursing,
        // but two arrays holding the same inner array are not a cycle.
        Object[] self = new Object[1];
        self[0] = self;
        System.out.println(Arrays.deepToString(self));
        Object[] inner = {1};
        System.out.println(Arrays.deepToString(new Object[] {inner, inner}));

        // deepEquals and deepHashCode have no such guard: they overflow.
        Object[] otherSelf = new Object[1];
        otherSelf[0] = otherSelf;
        try {
            System.out.println(Arrays.deepEquals(self, otherSelf));
        } catch (StackOverflowError e) {
            System.out.println("deepEquals overflows");
        }
        try {
            System.out.println(Arrays.deepHashCode(self));
        } catch (StackOverflowError e) {
            System.out.println("deepHashCode overflows");
        }
        System.out.println("still running");
    }
}
"#
);

differential_test!(
    diff_two_dimensional_primitive_arrays,
    "DiffTwoDimPrimitives",
    r#"
import java.util.Arrays;

public class DiffTwoDimPrimitives {
    public static void main(String[] args) {
        // Every primitive leaf type, allocated through multianewarray.
        int[][] ints = new int[2][2];
        boolean[][] flags = new boolean[2][2];
        char[][] letters = new char[2][2];
        double[][] doubles = new double[2][2];
        long[][] longs = new long[2][2];
        float[][] floats = new float[2][2];
        short[][] shorts = new short[2][2];
        byte[][] bytes = new byte[2][2];

        ints[0][0] = 1;
        flags[0][0] = true;
        letters[0][0] = 'x';
        doubles[0][0] = 1.5;
        longs[0][0] = 5L;
        floats[0][0] = 1.5f;
        shorts[0][0] = (short) 3;
        bytes[0][0] = (byte) 4;

        System.out.println(ints[0][0] + " " + flags[0][0] + " " + letters[0][0] + " " + doubles[0][0]);
        System.out.println(longs[0][0] + " " + floats[0][0] + " " + shorts[0][0] + " " + bytes[0][0]);
        System.out.println(longs[1][1] + " " + floats[1][1] + " " + shorts[1][1] + " " + bytes[1][1]);
        System.out.println(Arrays.toString(longs[0]) + " " + Arrays.toString(bytes[0]));

        // A ragged allocation leaves the rows null.
        long[][] ragged = new long[2][];
        ragged[0] = new long[] {7L};
        System.out.println(ragged[0][0] + " " + (ragged[1] == null));

        // A multi-dimensional array is an Object[] of its rows.
        Object[] rows = ints;
        System.out.println(rows.length + " " + Arrays.deepToString(rows));
    }
}
"#
);

differential_test!(
    diff_arrays_copy_fill_and_binary_search,
    "DiffArraysCopyFill",
    r#"
import java.util.Arrays;

class Card implements Comparable<Card> {
    int rank;
    Card(int rank) { this.rank = rank; }
    public int compareTo(Card other) { return rank - other.rank; }
    public String toString() { return "C" + rank; }
}

public class DiffArraysCopyFill {
    public static void main(String[] args) {
        int[] sorted = {1, 3, 5, 7};
        System.out.println(Arrays.binarySearch(sorted, 5) + " " + Arrays.binarySearch(sorted, 4)
                + " " + Arrays.binarySearch(sorted, 0) + " " + Arrays.binarySearch(sorted, 9));
        System.out.println(Arrays.binarySearch(sorted, 1, 3, 5) + " " + Arrays.binarySearch(sorted, 1, 3, 7));
        System.out.println(Arrays.binarySearch(new String[] {"a", "c", "e"}, "c") + " "
                + Arrays.binarySearch(new String[] {"a", "c", "e"}, "b"));
        System.out.println(Arrays.binarySearch(new Card[] {new Card(1), new Card(3)}, new Card(3))
                + " " + Arrays.binarySearch(new Card[] {new Card(1), new Card(3)}, new Card(2)));
        // -0.0 sorts below 0.0 and NaN above everything, so binarySearch finds
        // each of them where Double.compare puts it.
        double[] doubles = {-0.0, 0.0, 1.5, Double.NaN};
        System.out.println(Arrays.binarySearch(doubles, 0.0) + " " + Arrays.binarySearch(doubles, -0.0)
                + " " + Arrays.binarySearch(doubles, Double.NaN));
        System.out.println(Arrays.binarySearch(new char[] {'a', 'c'}, 'c') + " "
                + Arrays.binarySearch(new char[] {'a', 'c'}, 'b'));
        System.out.println(Arrays.binarySearch(new long[] {1L, 5L}, 5L) + " "
                + Arrays.binarySearch(new int[0], 5));

        // copyOf keeps the source's own type and pads with the element default.
        System.out.println(Arrays.toString(Arrays.copyOf(sorted, 2)) + " "
                + Arrays.toString(Arrays.copyOf(sorted, 6)));
        String[] words = Arrays.copyOf(new String[] {"a"}, 3);
        System.out.println(Arrays.toString(words) + " " + words.length);
        System.out.println(Arrays.toString(Arrays.copyOf(new boolean[] {true}, 3)) + " "
                + Arrays.toString(Arrays.copyOf(new char[] {'x'}, 2)).length());
        int[][] grid = {{1}, {2}};
        System.out.println(Arrays.deepToString(Arrays.copyOf(grid, 3)));

        System.out.println(Arrays.toString(Arrays.copyOfRange(sorted, 1, 3)) + " "
                + Arrays.toString(Arrays.copyOfRange(sorted, 2, 6)) + " "
                + Arrays.toString(Arrays.copyOfRange(sorted, 4, 4)));

        int[] filled = new int[4];
        Arrays.fill(filled, 7);
        System.out.println(Arrays.toString(filled));
        Arrays.fill(filled, 1, 3, 9);
        System.out.println(Arrays.toString(filled));
        String[] strings = new String[3];
        Arrays.fill(strings, "z");
        System.out.println(Arrays.toString(strings));
        double[] halves = new double[2];
        Arrays.fill(halves, 1.5);
        long[] longs = new long[2];
        Arrays.fill(longs, 8L);
        byte[] bytes = new byte[2];
        Arrays.fill(bytes, (byte) 3);
        boolean[] flags = new boolean[2];
        Arrays.fill(flags, true);
        char[] letters = new char[2];
        Arrays.fill(letters, 'q');
        System.out.println(Arrays.toString(halves) + " " + Arrays.toString(longs) + " "
                + Arrays.toString(bytes) + " " + Arrays.toString(flags) + " " + Arrays.toString(letters));
        // An int widens into an int[] element; a char does too.
        Arrays.fill(filled, 'a');
        System.out.println(filled[0]);
    }
}
"#
);

differential_test!(
    diff_arrays_copy_fill_errors,
    "DiffArraysCopyFillErrors",
    r#"
import java.util.Arrays;

class Plain {
    int x;
}

public class DiffArraysCopyFillErrors {
    public static void main(String[] args) {
        int[] numbers = {1, 2, 3};
        try { Arrays.copyOf(numbers, -1); } catch (NegativeArraySizeException e) { System.out.println("negative size"); }
        try { Arrays.copyOfRange(numbers, 3, 1); } catch (IllegalArgumentException e) { System.out.println("copy from > to"); }
        try { Arrays.copyOfRange(numbers, 9, 10); } catch (ArrayIndexOutOfBoundsException e) { System.out.println("copy past end"); }
        try { Arrays.fill(numbers, 3, 1, 0); } catch (IllegalArgumentException e) { System.out.println("fill from > to"); }
        try { Arrays.fill(numbers, -1, 2, 0); } catch (ArrayIndexOutOfBoundsException e) { System.out.println("fill negative"); }
        try { Arrays.fill(numbers, 0, 9, 0); } catch (ArrayIndexOutOfBoundsException e) { System.out.println("fill past end"); }
        try { Arrays.binarySearch(numbers, 3, 1, 0); } catch (IllegalArgumentException e) { System.out.println("search from > to"); }
        try { Arrays.binarySearch(numbers, 0, 9, 0); } catch (ArrayIndexOutOfBoundsException e) { System.out.println("search past end"); }

        // A reference binarySearch calls the element's compareTo, so a class
        // that has none is not Comparable, and a null key throws.
        Plain[] plains = {new Plain(), new Plain()};
        try { System.out.println(Arrays.binarySearch(plains, new Plain())); }
        catch (ClassCastException e) { System.out.println("not comparable"); }
        try { System.out.println(Arrays.binarySearch(new String[] {"a", "c"}, null)); }
        catch (NullPointerException e) { System.out.println("null key"); }

        int[] none = null;
        try { Arrays.copyOf(none, 2); } catch (NullPointerException e) { System.out.println("null array"); }

        // Degenerate but legal.
        System.out.println(Arrays.toString(Arrays.copyOf(new int[0], 2))
                + " " + Arrays.toString(Arrays.copyOfRange(numbers, 3, 3))
                + " " + Arrays.binarySearch(new int[0], 5));
        System.out.println("done");
    }
}
"#
);

differential_test!(
    diff_sorting_a_null_element_throws,
    "DiffSortNull",
    r#"
import java.util.ArrayList;
import java.util.Collections;

public class DiffSortNull {
    public static void main(String[] args) {
        // Natural-ordering sort calls compareTo, so a null element throws —
        // but a single element is never compared.
        ArrayList<String> words = new ArrayList<String>();
        words.add("b");
        words.add(null);
        words.add("a");
        try {
            Collections.sort(words);
            System.out.println("sorted " + words);
        } catch (NullPointerException e) {
            System.out.println("sort throws");
        }
        ArrayList<String> lone = new ArrayList<String>();
        lone.add(null);
        Collections.sort(lone);
        System.out.println(lone);
    }
}
"#
);

differential_test!(
    diff_collections_helpers,
    "DiffCollectionsHelpers",
    r#"
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Random;

class Card implements Comparable<Card> {
    int rank;
    String tag;
    Card(int rank, String tag) { this.rank = rank; this.tag = tag; }
    public int compareTo(Card other) { return rank - other.rank; }
    public String toString() { return "C" + rank + tag; }
}

public class DiffCollectionsHelpers {
    public static void main(String[] args) {
        ArrayList<Integer> numbers = new ArrayList<Integer>();
        numbers.add(3);
        numbers.add(1);
        numbers.add(5);
        numbers.add(1);
        System.out.println(Collections.max(numbers) + " " + Collections.min(numbers));
        int biggest = Collections.max(numbers);
        System.out.println(biggest);

        ArrayList<String> words = new ArrayList<String>();
        words.add("pear");
        words.add("fig");
        words.add("apple");
        System.out.println(Collections.max(words) + " " + Collections.min(words));

        // A user class compares by its own compareTo, and ties keep the first.
        ArrayList<Card> cards = new ArrayList<Card>();
        cards.add(new Card(2, "a"));
        cards.add(new Card(5, "b"));
        cards.add(new Card(5, "c"));
        cards.add(new Card(1, "d"));
        System.out.println(Collections.max(cards) + " " + Collections.min(cards));

        // frequency asks the probe's own equals; Card has none, so identity.
        System.out.println(Collections.frequency(numbers, 1) + " " + Collections.frequency(numbers, 9));
        System.out.println(Collections.frequency(words, "fig"));
        ArrayList<Card> pair = new ArrayList<Card>();
        pair.add(new Card(1, "x"));
        pair.add(new Card(1, "y"));
        System.out.println(Collections.frequency(pair, new Card(1, "z")));

        List<String> copies = Collections.nCopies(3, "z");
        System.out.println(copies + " " + copies.size());
        System.out.println(Collections.nCopies(2, 0) + " " + Collections.nCopies(0, "q"));
        int total = 0;
        for (int value : Collections.nCopies(3, 7)) {
            total += value;
        }
        System.out.println(total);

        // A seeded Random makes shuffle replay the JDK's own permutation.
        ArrayList<Integer> deck = new ArrayList<Integer>();
        for (int i = 0; i < 8; i++) {
            deck.add(i);
        }
        Collections.shuffle(deck, new Random(42));
        System.out.println(deck);
        Collections.shuffle(deck, new Random(7));
        System.out.println(deck);
        ArrayList<String> letters = new ArrayList<String>();
        letters.add("a");
        letters.add("b");
        letters.add("c");
        Collections.shuffle(letters, new Random(1));
        System.out.println(letters);

        // Degenerate sizes.
        ArrayList<Integer> empty = new ArrayList<Integer>();
        Collections.shuffle(empty, new Random(1));
        System.out.println(empty + " " + Collections.frequency(empty, 1));
    }
}
"#
);

differential_test!(
    diff_collections_helper_errors,
    "DiffCollectionsErrors",
    r#"
import java.util.ArrayList;
import java.util.Collections;

public class DiffCollectionsErrors {
    public static void main(String[] args) {
        ArrayList<Integer> empty = new ArrayList<Integer>();
        try {
            Collections.max(empty);
        } catch (java.util.NoSuchElementException e) {
            System.out.println("max of empty");
        }
        try {
            Collections.min(empty);
        } catch (java.util.NoSuchElementException e) {
            System.out.println("min of empty");
        }
        try {
            Collections.nCopies(-1, "a");
        } catch (IllegalArgumentException e) {
            System.out.println("negative count");
        }

        // max/min call compareTo, so a null element throws.
        ArrayList<String> holes = new ArrayList<String>();
        holes.add("a");
        holes.add(null);
        try {
            Collections.max(holes);
        } catch (NullPointerException e) {
            System.out.println("null element");
        }
        // But a lone element is never compared, and frequency counts nulls.
        ArrayList<String> lone = new ArrayList<String>();
        lone.add(null);
        System.out.println(Collections.max(lone) + " " + Collections.frequency(holes, null));
        System.out.println("done");
    }
}
"#
);

differential_test!(
    diff_collections_binary_search_and_add_all,
    "DiffCollectionsSearch",
    r#"
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

class Card implements Comparable<Card> {
    int rank;
    String tag;
    Card(int rank, String tag) { this.rank = rank; this.tag = tag; }
    public int compareTo(Card other) { return rank - other.rank; }
    public String toString() { return "C" + rank + tag; }
}

public class DiffCollectionsSearch {
    public static void main(String[] args) {
        ArrayList<Integer> numbers = new ArrayList<Integer>();
        System.out.println(Collections.addAll(numbers, 1, 3, 5, 7) + " " + numbers);
        System.out.println(Collections.addAll(numbers) + " " + numbers.size());
        System.out.println(Collections.binarySearch(numbers, 5) + " " + Collections.binarySearch(numbers, 4)
                + " " + Collections.binarySearch(numbers, 0) + " " + Collections.binarySearch(numbers, 9));

        ArrayList<String> words = new ArrayList<String>();
        Collections.addAll(words, "a", "c", "e");
        System.out.println(Collections.binarySearch(words, "c") + " " + Collections.binarySearch(words, "b"));
        String[] more = {"g", "i"};
        Collections.addAll(words, more);
        System.out.println(words);

        // binarySearch compares with the element's own compareTo.
        ArrayList<Card> cards = new ArrayList<Card>();
        Collections.addAll(cards, new Card(1, "a"), new Card(3, "b"));
        System.out.println(Collections.binarySearch(cards, new Card(3, "z")) + " " + cards);
        System.out.println(Collections.binarySearch(new ArrayList<Integer>(), 5));

        // Arrays.asList keeps its elements unboxed, so a primitive list reads back.
        List<Integer> boxed = Arrays.asList(1, 2, 3);
        int first = boxed.get(0);
        System.out.println(boxed + " " + first + " " + (boxed.get(1) + 1) + " " + boxed.indexOf(3));
    }
}
"#
);

differential_test!(
    diff_collections_unmodifiable_and_empty,
    "DiffCollectionsUnmodifiable",
    r#"
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

public class DiffCollectionsUnmodifiable {
    public static void main(String[] args) {
        ArrayList<Integer> numbers = new ArrayList<Integer>();
        Collections.addAll(numbers, 1, 3, 5, 7);

        List<Integer> readOnly = Collections.unmodifiableList(numbers);
        System.out.println(readOnly + " " + readOnly.size() + " " + readOnly.get(1)
                + " " + readOnly.contains(5) + " " + readOnly.indexOf(7));

        // Every mutator refuses, through the list and through Collections.
        try { readOnly.add(9); } catch (UnsupportedOperationException e) { System.out.println("add"); }
        try { readOnly.set(0, 9); } catch (UnsupportedOperationException e) { System.out.println("set"); }
        try { readOnly.remove(0); } catch (UnsupportedOperationException e) { System.out.println("remove"); }
        try { readOnly.clear(); } catch (UnsupportedOperationException e) { System.out.println("clear"); }
        try { Collections.sort(readOnly); } catch (UnsupportedOperationException e) { System.out.println("sort"); }
        try { Collections.reverse(readOnly); } catch (UnsupportedOperationException e) { System.out.println("reverse"); }
        try { Collections.addAll(readOnly, 9); } catch (UnsupportedOperationException e) { System.out.println("addAll"); }

        // It is a view: a later change to the backing list shows through.
        numbers.add(11);
        System.out.println(readOnly + " " + readOnly.size());
        int total = 0;
        for (int value : readOnly) {
            total += value;
        }
        System.out.println(total);
        System.out.println(readOnly.equals(numbers) + " " + numbers.equals(readOnly));
        System.out.println(Collections.max(readOnly) + " " + Collections.frequency(readOnly, 3));

        List<String> empty = Collections.emptyList();
        System.out.println(empty + " " + empty.size() + " " + empty.isEmpty());
        try { empty.add("x"); } catch (UnsupportedOperationException e) { System.out.println("empty add"); }
    }
}
"#
);

differential_test!(
    diff_list_contains_remove_retain_all,
    "DiffBulkOps",
    r#"
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

class Point {
    int x;
    Point(int x) { this.x = x; }
    public boolean equals(Object other) {
        return other instanceof Point && ((Point) other).x == x;
    }
    public int hashCode() { return x; }
    public String toString() { return "P" + x; }
}

public class DiffBulkOps {
    public static void main(String[] args) {
        ArrayList<Integer> numbers = new ArrayList<Integer>();
        Collections.addAll(numbers, 1, 2, 3, 2);
        ArrayList<Integer> some = new ArrayList<Integer>();
        Collections.addAll(some, 2, 4);
        ArrayList<Integer> subset = new ArrayList<Integer>();
        Collections.addAll(subset, 1, 3);
        System.out.println(numbers.containsAll(some) + " " + numbers.containsAll(subset)
                + " " + numbers.containsAll(numbers) + " " + numbers.containsAll(new ArrayList<Integer>()));

        // removeAll and retainAll report whether the list changed, and remove
        // every occurrence.
        ArrayList<Integer> removed = new ArrayList<Integer>(numbers);
        System.out.println(removed.removeAll(some) + " " + removed);
        ArrayList<Integer> unchanged = new ArrayList<Integer>(numbers);
        System.out.println(unchanged.removeAll(new ArrayList<Integer>()) + " " + unchanged);
        ArrayList<Integer> retained = new ArrayList<Integer>(numbers);
        System.out.println(retained.retainAll(some) + " " + retained);
        ArrayList<Integer> emptied = new ArrayList<Integer>(numbers);
        System.out.println(emptied.retainAll(new ArrayList<Integer>()) + " " + emptied);
        ArrayList<Integer> all = new ArrayList<Integer>(numbers);
        System.out.println(all.retainAll(numbers) + " " + all);

        // They compare with the elements' own equals.
        ArrayList<Point> points = new ArrayList<Point>();
        Collections.addAll(points, new Point(1), new Point(2));
        ArrayList<Point> twos = new ArrayList<Point>();
        Collections.addAll(twos, new Point(2));
        System.out.println(points.containsAll(twos));
        ArrayList<Point> shrunk = new ArrayList<Point>(points);
        System.out.println(shrunk.removeAll(twos) + " " + shrunk);

        // Nulls compare too.
        ArrayList<String> words = new ArrayList<String>();
        Collections.addAll(words, "a", null, "b");
        ArrayList<String> holes = new ArrayList<String>();
        Collections.addAll(holes, (String) null);
        System.out.println(words.containsAll(holes) + " " + words.removeAll(holes) + " " + words);

        // An unmodifiable view reads, but refuses to change.
        ArrayList<Integer> backing = new ArrayList<Integer>();
        Collections.addAll(backing, 1, 2, 3, 2);
        List<Integer> readOnly = Collections.unmodifiableList(backing);
        System.out.println(readOnly.containsAll(some) + " " + backing.containsAll(readOnly));
        try { readOnly.removeAll(some); } catch (UnsupportedOperationException e) { System.out.println("removeAll"); }
        try { readOnly.retainAll(some); } catch (UnsupportedOperationException e) { System.out.println("retainAll"); }

        try { numbers.containsAll(null); } catch (NullPointerException e) { System.out.println("null argument"); }
        System.out.println("done");
    }
}
"#
);

differential_test!(
    diff_cast_of_a_null_literal,
    "DiffCastNull",
    r#"
public class DiffCastNull {
    public static void main(String[] args) {
        // A cast of `null` names an overload; it is not a parenthesized
        // expression, and `(x) - 1` still is one.
        String text = (String) null;
        Object thing = (Object) null;
        System.out.println(text + " " + thing + " " + (text == null));
        int x = 5;
        System.out.println((x) - 1);
        System.out.println((x) + 1);
    }
}
"#
);

differential_test!(
    diff_system_arraycopy_and_line_separator,
    "DiffArrayCopy",
    r#"
import java.util.Arrays;

public class DiffArrayCopy {
    public static void main(String[] args) {
        int[] source = {1, 2, 3, 4, 5};
        int[] destination = new int[5];
        System.arraycopy(source, 1, destination, 0, 3);
        System.out.println(Arrays.toString(destination));

        // A copy within one array behaves as if it went through a temporary,
        // in both directions.
        int[] forward = {1, 2, 3, 4, 5};
        System.arraycopy(forward, 0, forward, 1, 4);
        System.out.println(Arrays.toString(forward));
        int[] backward = {1, 2, 3, 4, 5};
        System.arraycopy(backward, 1, backward, 0, 4);
        System.out.println(Arrays.toString(backward));

        // Every element kind.
        String[] words = {"a", "b", "c"};
        String[] copied = new String[3];
        System.arraycopy(words, 0, copied, 1, 2);
        System.out.println(Arrays.toString(copied));
        double[] doubles = new double[2];
        System.arraycopy(new double[] {1.5, 2.5}, 0, doubles, 0, 2);
        long[] longs = new long[2];
        System.arraycopy(new long[] {7L, 8L}, 0, longs, 0, 2);
        char[] letters = new char[2];
        System.arraycopy(new char[] {'x', 'y'}, 0, letters, 0, 2);
        boolean[] flags = new boolean[2];
        System.arraycopy(new boolean[] {true, false}, 0, flags, 0, 2);
        byte[] bytes = new byte[2];
        System.arraycopy(new byte[] {1, 2}, 0, bytes, 0, 2);
        short[] shorts = new short[2];
        System.arraycopy(new short[] {3, 4}, 0, shorts, 0, 2);
        float[] floats = new float[2];
        System.arraycopy(new float[] {1.5f, 2.5f}, 0, floats, 0, 2);
        System.out.println(Arrays.toString(doubles) + Arrays.toString(longs) + Arrays.toString(letters));
        System.out.println(Arrays.toString(flags) + Arrays.toString(bytes)
                + Arrays.toString(shorts) + Arrays.toString(floats));

        // The rows of a 2D array are references, so they alias.
        int[][] grid = {{1}, {2}};
        int[][] rows = new int[2][];
        System.arraycopy(grid, 0, rows, 0, 2);
        System.out.println(Arrays.deepToString(rows) + " " + (grid[0] == rows[0]));

        // A zero-length copy at the very end is legal.
        System.arraycopy(source, 5, destination, 5, 0);
        System.out.println("empty copy");

        try { System.arraycopy(source, 0, destination, 0, 9); }
        catch (ArrayIndexOutOfBoundsException e) { System.out.println("too long"); }
        try { System.arraycopy(source, -1, destination, 0, 1); }
        catch (ArrayIndexOutOfBoundsException e) { System.out.println("negative position"); }
        try { System.arraycopy(source, 0, destination, 0, -1); }
        catch (ArrayIndexOutOfBoundsException e) { System.out.println("negative length"); }
        int[] none = null;
        try { System.arraycopy(none, 0, destination, 0, 1); }
        catch (NullPointerException e) { System.out.println("null"); }
        // The component types must match exactly.
        try { System.arraycopy(source, 0, new long[5], 0, 1); }
        catch (ArrayStoreException e) { System.out.println("int into long"); }
        try { System.arraycopy(new boolean[5], 0, destination, 0, 1); }
        catch (ArrayStoreException e) { System.out.println("boolean into int"); }

        // Runs on a system where a line ends with a newline.
        System.out.println(System.lineSeparator().equals("\n")
                + " " + System.lineSeparator().length());
    }
}
"#
);

differential_test!(
    diff_math_multiply_full_and_scalb,
    "DiffScalb",
    r#"
public class DiffScalb {
    public static void main(String[] args) {
        // multiplyFull keeps the whole 64-bit product that `int * int` wraps.
        System.out.println(Math.multiplyFull(3, 4) + " " + Math.multiplyFull(-3, 4));
        System.out.println(Math.multiplyFull(Integer.MAX_VALUE, Integer.MAX_VALUE)
                + " " + (Integer.MAX_VALUE * Integer.MAX_VALUE));
        System.out.println(Math.multiplyFull(Integer.MIN_VALUE, Integer.MIN_VALUE));

        System.out.println(Math.scalb(1.5, 3) + " " + Math.scalb(1.5, -3) + " " + Math.scalb(1.5, 0));
        // Underflow into the subnormals must round exactly once.
        System.out.println(Math.scalb(1.5, -1074) + " " + Math.scalb(1.0, -1074)
                + " " + Math.scalb(1.0, -1075) + " " + Math.scalb(4.9E-324, 1));
        System.out.println(Math.scalb(1.0, 2000) + " " + Math.scalb(1.0, -2000)
                + " " + Math.scalb(-1.0, 2000));
        System.out.println(Math.scalb(1.0, 1023) + " " + Math.scalb(1.0, 1024)
                + " " + Math.scalb(Double.MAX_VALUE, 1));
        System.out.println(Math.scalb(0.0, 5) + " " + Math.scalb(-0.0, 5)
                + " " + Math.scalb(Double.NaN, 5) + " " + Math.scalb(Double.POSITIVE_INFINITY, -5));
        System.out.println(Math.scalb(1.5f, 3) + " " + Math.scalb(1.5f, -150)
                + " " + Math.scalb(1.0f, 300) + " " + Math.scalb(1.4E-45f, 1)
                + " " + Math.scalb(1.0f, -300));
        // An `int` argument: both overloads apply, and the more specific
        // `float` one wins, in caturra as in javac.
        System.out.println(Math.scalb(1, 3) + " " + (Math.scalb(1, 3) == 8.0f));

        // Every scale, over values spanning normals, subnormals and overflow.
        long hash = 1469598103934665603L;
        double[] values = {1.0, 1.5, 0.1, 3.7e-300, 1.7e300, 4.9E-324, 2.2250738585072014E-308, -7.25, 1e-310};
        for (int v = 0; v < values.length; v++) {
            for (int n = -1100; n <= 1100; n++) {
                hash = (hash ^ Double.doubleToLongBits(Math.scalb(values[v], n))) * 1099511628211L;
            }
        }
        float[] floats = {1.0f, 1.5f, 0.1f, 3.7e-38f, 1.7e38f, 1.4E-45f, -7.25f};
        for (int v = 0; v < floats.length; v++) {
            for (int n = -300; n <= 300; n++) {
                hash = (hash ^ Float.floatToIntBits(Math.scalb(floats[v], n))) * 1099511628211L;
            }
        }
        System.out.println(hash);
    }
}
"#
);

differential_test!(
    diff_random_next_bytes,
    "DiffNextBytes",
    r#"
import java.util.Arrays;
import java.util.Random;

public class DiffNextBytes {
    public static void main(String[] args) {
        byte[] five = new byte[5];
        new Random(42).nextBytes(five);
        System.out.println(Arrays.toString(five));
        byte[] eight = new byte[8];
        new Random(7).nextBytes(eight);
        System.out.println(Arrays.toString(eight));
        byte[] none = new byte[0];
        new Random(1).nextBytes(none);
        byte[] one = new byte[1];
        new Random(1).nextBytes(one);
        System.out.println(Arrays.toString(none) + " " + Arrays.toString(one));

        // The draws advance the same generator, four bytes per next(32).
        Random shared = new Random(99);
        byte[] three = new byte[3];
        shared.nextBytes(three);
        System.out.println(Arrays.toString(three) + " " + shared.nextInt(100));

        // Every seed and length, bytes and the generator's resulting state.
        long hash = 1469598103934665603L;
        for (int seed = 0; seed < 40; seed++) {
            for (int length = 0; length <= 17; length++) {
                byte[] bytes = new byte[length];
                Random random = new Random(seed);
                random.nextBytes(bytes);
                for (int i = 0; i < length; i++) {
                    hash = (hash ^ bytes[i]) * 1099511628211L;
                }
                hash = (hash ^ random.nextInt()) * 1099511628211L;
            }
        }
        System.out.println(hash);
    }
}
"#
);

// ---------------------------------------------------------------------------
// Programs javac rejects. caturra must reject them too, or they would compile
// in the playground and fail on a real JDK. A `differential_test!` cannot
// catch this: it only ever runs programs javac accepts.
// ---------------------------------------------------------------------------

differential_reject!(
    reject_post_java_11_math_abs_exact,
    "RejectAbsExact",
    r"
public class RejectAbsExact {
    public static void main(String[] args) {
        System.out.println(Math.absExact(-5));
    }
}
"
);

differential_reject!(
    reject_post_java_11_random_next_int_range,
    "RejectNextIntRange",
    r"
import java.util.Random;

public class RejectNextIntRange {
    public static void main(String[] args) {
        System.out.println(new Random(1).nextInt(2, 5));
    }
}
"
);

differential_reject!(
    reject_shuffle_without_a_random,
    "RejectShuffle",
    r"
import java.util.ArrayList;
import java.util.Collections;

public class RejectShuffle {
    public static void main(String[] args) {
        Collections.shuffle(new ArrayList<Integer>(), 5);
    }
}
"
);

differential_reject!(
    reject_add_all_of_a_primitive_array,
    "RejectAddAll",
    r"
import java.util.ArrayList;
import java.util.Collections;

public class RejectAddAll {
    public static void main(String[] args) {
        ArrayList<Integer> numbers = new ArrayList<Integer>();
        int[] values = {1, 2};
        Collections.addAll(numbers, values);
    }
}
"
);

differential_reject!(
    reject_binary_search_of_a_boolean_array,
    "RejectBooleanSearch",
    r"
import java.util.Arrays;

public class RejectBooleanSearch {
    public static void main(String[] args) {
        System.out.println(Arrays.binarySearch(new boolean[] {true}, true));
    }
}
"
);

differential_reject!(
    reject_deep_to_string_of_a_one_dimensional_primitive_array,
    "RejectDeepToString",
    r"
import java.util.Arrays;

public class RejectDeepToString {
    public static void main(String[] args) {
        System.out.println(Arrays.deepToString(new int[] {1}));
    }
}
"
);

differential_reject!(
    reject_array_copy_with_too_few_arguments,
    "RejectArrayCopy",
    r"
public class RejectArrayCopy {
    public static void main(String[] args) {
        int[] cells = new int[2];
        System.arraycopy(cells, 0, cells, 0);
    }
}
"
);

differential_reject!(
    reject_multiply_full_of_longs,
    "RejectMultiplyFull",
    r"
public class RejectMultiplyFull {
    public static void main(String[] args) {
        System.out.println(Math.multiplyFull(1L, 2L));
    }
}
"
);

differential_reject!(
    reject_scalb_with_a_double_scale,
    "RejectScalb",
    r"
public class RejectScalb {
    public static void main(String[] args) {
        System.out.println(Math.scalb(1.5, 3.0));
    }
}
"
);

differential_reject!(
    reject_next_bytes_of_an_int_array,
    "RejectNextBytes",
    r"
import java.util.Random;

public class RejectNextBytes {
    public static void main(String[] args) {
        new Random(1).nextBytes(new int[2]);
    }
}
"
);

differential_reject!(
    reject_collections_max_of_a_non_collection,
    "RejectMax",
    r"
import java.util.Collections;

public class RejectMax {
    public static void main(String[] args) {
        System.out.println(Collections.max(5));
    }
}
"
);

differential_reject!(
    reject_an_omitted_array_initializer_element,
    "RejectOmittedElement",
    r"
public class RejectOmittedElement {
    public static void main(String[] args) {
        int[] cells = {1,,2};
        System.out.println(cells.length);
    }
}
"
);

differential_reject!(
    reject_copy_of_into_the_wrong_array_type,
    "RejectCopyOf",
    r#"
import java.util.Arrays;

public class RejectCopyOf {
    public static void main(String[] args) {
        String[] words = {"a"};
        int[] wrong = Arrays.copyOf(words, 2);
        System.out.println(wrong.length);
    }
}
"#
);

differential_reject!(
    reject_fill_with_an_incompatible_value,
    "RejectFill",
    r#"
import java.util.Arrays;

public class RejectFill {
    public static void main(String[] args) {
        int[] cells = new int[2];
        Arrays.fill(cells, "s");
    }
}
"#
);

differential_test!(
    diff_fully_qualified_names_in_expression_position,
    "DiffFqn",
    r#"
public class DiffFqn {
    public static void main(String[] args) {
        int[] filled = new int[3];
        java.util.Arrays.fill(filled, 7);
        System.out.println(java.util.Arrays.toString(filled));

        int[] unsorted = {3, 1, 2};
        java.util.Arrays.sort(unsorted);
        System.out.println(java.util.Arrays.toString(unsorted));
        System.out.println(java.util.Arrays.binarySearch(unsorted, 2));

        java.util.List<Integer> list = new java.util.ArrayList<Integer>();
        list.add(5);
        list.add(2);
        java.util.Collections.sort(list);
        System.out.println(list);
        System.out.println(java.util.Collections.max(list));
        System.out.println(java.util.Collections.frequency(list, 5));

        java.util.Random random = new java.util.Random(42);
        System.out.println(random.nextInt(100));

        java.lang.StringBuilder builder = new java.lang.StringBuilder();
        builder.append("ab").append(1);
        System.out.println(builder.reverse());

        System.out.println(java.lang.Math.abs(-5));
        System.out.println(java.lang.Integer.MAX_VALUE);
        java.lang.System.out.println("qualified");

        java.util.Map<String, Integer> map = new java.util.HashMap<String, Integer>();
        map.put("k", 1);
        System.out.println(map.get("k"));

        // JLS 6.4.2: a variable named `java` obscures the package.
        int java = 3;
        System.out.println(java);
    }
}
"#
);

differential_reject!(
    reject_unknown_class_in_a_known_package,
    "RejectQualifiedClass",
    "public class RejectQualifiedClass { static void r() { java.util.Nope.f(); } }"
);

differential_reject!(
    reject_unknown_package_in_expression_position,
    "RejectQualifiedPackage",
    "public class RejectQualifiedPackage { static void r() { java.zzz.Nope.f(); } }"
);

stricter_than_javac!(
    strict_linked_list_is_refused_by_name,
    "StrictLinkedList",
    "import java.util.*;\npublic class StrictLinkedList { static void r() { LinkedList<Integer> l; } }"
);

stricter_than_javac!(
    strict_hash_set_is_refused_by_name,
    "StrictHashSet",
    "import java.util.*;\npublic class StrictHashSet { static void r() { HashSet<Integer> s; } }"
);

differential_test!(
    diff_array_default_to_string,
    "DiffArrayToString",
    r#"
public class DiffArrayToString {
    // The identity hash legitimately differs between JVMs, so keep the class
    // name and assert only that what follows `@` is lowercase hex.
    static String head(String s) {
        int at = s.indexOf("@");
        if (at < 0) return "NO-AT:" + s;
        String hex = s.substring(at + 1);
        boolean ok = hex.length() > 0;
        for (int i = 0; i < hex.length(); i++) {
            char c = hex.charAt(i);
            if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) ok = false;
        }
        return s.substring(0, at) + "@" + (ok ? "<hex>" : "BAD:" + hex);
    }

    public static void main(String[] args) {
        int[] ints = {1, 2, 3};
        char[] chars = {'a', 'b'};
        String[] strings = {"x"};
        boolean[] booleans = {true};
        double[] doubles = {1.5};
        long[] longs = {1L};
        byte[] bytes = {1};
        short[] shorts = {1};
        float[] floats = {1.0f};
        int[][] grid = {{1}};
        String[][] words = {{"a"}};
        Object[] objects = {"e"};

        System.out.println(head("" + ints));
        System.out.println(head("" + strings));
        System.out.println(head("" + booleans));
        System.out.println(head("" + doubles));
        System.out.println(head("" + longs));
        System.out.println(head("" + bytes));
        System.out.println(head("" + shorts));
        System.out.println(head("" + floats));
        System.out.println(head("" + grid));
        System.out.println(head("" + grid[0]));
        System.out.println(head("" + words));
        System.out.println(head("" + words[0]));
        System.out.println(head("" + objects));
        System.out.println(head("" + new int[0]));
        System.out.println(head("" + new String[2]));
        System.out.println(head("" + new int[2][]));

        // `println(char[])` is a real overload and prints the characters;
        // concatenation is String.valueOf(Object) and does not.
        System.out.println(chars);
        System.out.print(chars);
        System.out.println();
        System.out.println(head("" + chars));

        // toString() agrees with concatenation, and hashCode is the identity
        // hash that toString prints.
        System.out.println(head(ints.toString()).equals(head("" + ints)));
        System.out.println(ints.hashCode() == ints.hashCode());
        System.out.println(ints.hashCode() == strings.hashCode());
        System.out.println(ints.equals(ints));
        System.out.println(ints.equals(new int[] {1, 2, 3}));

        // getClass().getName() is the same name toString() uses.
        System.out.println(ints.getClass().getName());
        System.out.println(strings.getClass().getName());
        System.out.println(grid.getClass().getName());
        System.out.println(words.getClass().getName());
    }
}
"#
);

differential_test!(
    diff_field_hiding,
    "DiffHiding",
    r#"
class HidingSuper {
    private String tag = "super-private";
    protected int n = 1;
    public String read() { return tag + "/" + n; }
    public int getN() { return n; }
    public void bump() { n = n + 10; }
}

class HidingSub extends HidingSuper {
    // Hides a private field (never inherited) with a DIFFERENT type, and a
    // protected one with the same type. Both are distinct slots.
    private int tag = 42;
    protected int n = 2;
    public String read2() { return tag + "/" + n; }
    public int subN() { return n; }
    public void bumpSub() { n = n + 100; }
}

public class DiffHiding {
    public static void main(String[] args) {
        HidingSub sub = new HidingSub();
        HidingSuper up = sub;

        // Each class's own methods see their own field.
        System.out.println(sub.read());
        System.out.println(sub.read2());
        System.out.println(sub.subN());
        System.out.println(sub.getN());

        // The STATIC type of the reference decides which field an access
        // means — hiding is not overriding.
        System.out.println(up.n);
        System.out.println(sub.n);
        System.out.println(((HidingSuper) sub).n);

        // The two slots mutate independently.
        sub.bump();
        sub.bumpSub();
        System.out.println(sub.read());
        System.out.println(sub.read2());
        System.out.println(up.n + " " + sub.n);
    }
}
"#
);

differential_test!(
    diff_field_hiding_with_reflection,
    "DiffHidingReflect",
    r#"
import java.util.ArrayList;
import java.lang.reflect.Field;

class HidingBase {
    private ArrayList<String> data;
    static int count = 1;
    HidingBase() { data = new ArrayList<String>(); data.add("base"); }
    public String show() { return data.toString(); }
}

class HidingDerived extends HidingBase {
    private String data;
    static int count = 2;
    HidingDerived() { super(); data = "derived"; }
    public String show2() { return data; }
}

public class DiffHidingReflect {
    public static void main(String[] args) throws Exception {
        HidingDerived derived = new HidingDerived();
        System.out.println(derived.show());
        System.out.println(derived.show2());
        System.out.println(HidingBase.count + " " + HidingDerived.count);

        // getDeclaredFields lists a class's OWN fields, not what it hides.
        Field[] fields = HidingDerived.class.getDeclaredFields();
        System.out.println(fields.length);
        for (int i = 0; i < fields.length; i++) System.out.println(fields[i].getName());

        // Each Field reads the slot of the class that declared it.
        Field base = HidingBase.class.getDeclaredField("data");
        base.setAccessible(true);
        System.out.println(base.get(derived));
        Field sub = HidingDerived.class.getDeclaredField("data");
        sub.setAccessible(true);
        System.out.println(sub.get(derived));
    }
}
"#
);

differential_test!(
    diff_super_field_access,
    "DiffSuperField",
    r#"
class SupA {
    protected int n = 1;
    protected String s = "a";
}

class SupB extends SupA {
    // Hides `n`. `super.n` is SupA's field; `n` and `this.n` are SupB's.
    int n = 2;
    int viaSuper() { return super.n; }
    void setSuper(int v) { super.n = v; }
    void bumpSuper() { super.n += 5; super.n++; }
    String cat() { return super.s + super.n; }
    int all() { return super.n + this.n + n; }
    int lenOfSuperField() { return super.s.length(); }
}

// `super.n` resolves from the SUPERCLASS upward: SupC's super is SupB,
// which declares its own `n`, so it is SupB's — not SupA's.
class SupC extends SupB {
    int n = 3;
    int cSuper() { return super.n; }
}

public class DiffSuperField {
    public static void main(String[] args) {
        SupB b = new SupB();
        System.out.println(b.viaSuper() + " " + b.n);
        b.setSuper(9);
        System.out.println(b.viaSuper() + " " + b.n);
        b.bumpSuper();
        System.out.println(b.viaSuper());
        System.out.println(b.cat());
        System.out.println(b.all());
        System.out.println(b.lenOfSuperField());

        SupC c = new SupC();
        System.out.println(c.cSuper() + " " + c.n + " " + ((SupB) c).n + " " + ((SupA) c).n);
    }
}
"#
);

differential_reject!(
    reject_super_field_that_is_private,
    "RejectSuperPrivate",
    "class RSPa { private int n = 1; }\nclass RSPb extends RSPa { int f() { return super.n; } }\npublic class RejectSuperPrivate { static void r() {} }"
);

differential_reject!(
    reject_super_field_in_a_static_context,
    "RejectSuperStatic",
    "class RSSa { protected int n = 1; }\nclass RSSb extends RSSa { static int f() { return super.n; } }\npublic class RejectSuperStatic { static void r() {} }"
);

differential_reject!(
    reject_super_field_without_a_superclass,
    "RejectSuperNone",
    "class RSNb { int n = 1; int f() { return super.n; } }\npublic class RejectSuperNone { static void r() {} }"
);

// ---------------------------------------------------------------------------
// Reject wording, checked against javac rather than against our own memory of
// it. Both sides are pinned: if javac's phrasing changes with the JDK, or if
// caturra's drifts, the table says so.
// ---------------------------------------------------------------------------

/// How caturra's diagnostic relates to javac's headline for the same program.
enum Wording {
    /// Word for word.
    Same,
    /// javac's headline, plus the detail javac prints on its `symbol:` and
    /// `location:` continuation lines.
    Prefix,
    /// Deliberately different, and why. Promoting one of these to `Prefix`
    /// when caturra starts agreeing is the point of asserting it.
    Differs(&'static str),
}

/// `(class, source, javac's headline, caturra's message, relation)`.
const REJECT_WORDING: &[(&str, &str, &str, &str, Wording)] = &[
    (
        "WordAbsExact",
        "public class WordAbsExact { static int r() { return Math.absExact(-5); } }",
        "cannot find symbol",
        "cannot find symbol: method absExact(int) in class Math",
        Wording::Prefix,
    ),
    (
        "WordNextIntRange",
        "import java.util.Random;\npublic class WordNextIntRange { static int r() { return new Random(1).nextInt(2, 5); } }",
        "no suitable method found for nextInt(int,int)",
        "no suitable method found for nextInt(int,int) in class Random",
        Wording::Prefix,
    ),
    (
        "WordScalb",
        "public class WordScalb { static double r() { return Math.scalb(1.5, 3.0); } }",
        "no suitable method found for scalb(double,double)",
        "no suitable method found for scalb(double,double) in class Math",
        Wording::Prefix,
    ),
    (
        "WordBooleanSearch",
        "import java.util.Arrays;\npublic class WordBooleanSearch { static int r() { return Arrays.binarySearch(new boolean[] {true}, true); } }",
        "no suitable method found for binarySearch(boolean[],boolean)",
        "no suitable method found for binarySearch(boolean[],boolean) in class Arrays",
        Wording::Prefix,
    ),
    (
        "WordSearchArity",
        "import java.util.Arrays;\npublic class WordSearchArity { static int r() { return Arrays.binarySearch(new int[2]); } }",
        "no suitable method found for binarySearch(int[])",
        "no suitable method found for binarySearch(int[]) in class Arrays",
        Wording::Prefix,
    ),
    (
        "WordFillArity",
        "import java.util.Arrays;\npublic class WordFillArity { static void r() { Arrays.fill(new int[2], 0, 1); } }",
        "no suitable method found for fill(int[],int,int)",
        "no suitable method found for fill(int[],int,int) in class Arrays",
        Wording::Prefix,
    ),
    (
        "WordMax",
        "import java.util.Collections;\npublic class WordMax { static void r() { Collections.max(5); } }",
        "no suitable method found for max(int)",
        "no suitable method found for max(int) in class Collections",
        Wording::Prefix,
    ),
    (
        "WordMaxArity",
        "import java.util.*;\npublic class WordMaxArity { static void r() { ArrayList<Integer> l = new ArrayList<Integer>(); Collections.max(l, l); } }",
        "no suitable method found for max(ArrayList<Integer>,ArrayList<Integer>)",
        "no suitable method found for max(ArrayList<Integer>,ArrayList<Integer>) in class Collections",
        Wording::Prefix,
    ),
    (
        "WordDeepToString",
        "import java.util.Arrays;\npublic class WordDeepToString { static String r() { return Arrays.deepToString(new int[] {1}); } }",
        "incompatible types: int[] cannot be converted to Object[]",
        "incompatible types: int[] cannot be converted to Object[]",
        Wording::Same,
    ),
    (
        "WordShuffle",
        "import java.util.*;\npublic class WordShuffle { static void r() { Collections.shuffle(new ArrayList<Integer>(), 5); } }",
        "incompatible types: int cannot be converted to Random",
        "incompatible types: int cannot be converted to Random",
        Wording::Same,
    ),
    (
        "WordMultiplyFull",
        "public class WordMultiplyFull { static long r() { return Math.multiplyFull(1L, 2L); } }",
        "incompatible types: possible lossy conversion from long to int",
        "no suitable method found for multiplyFull(long,long) in class Math",
        Wording::Differs(
            "javac has a single multiplyFull(int,int), so it reports converting \
             the argument rather than resolving the overload",
        ),
    ),
    (
        "WordArrayCopyArity",
        "public class WordArrayCopyArity { static void r() { int[] c = new int[2]; System.arraycopy(c, 0, c, 0); } }",
        "method arraycopy in class System cannot be applied to given types;",
        "no suitable method found for arraycopy(int[],int,int[],int) in class System",
        Wording::Differs(
            "javac has a single arraycopy, and words a lone mismatched candidate \
             as `cannot be applied to given types`",
        ),
    ),
    (
        "WordUnmodifiable",
        "import java.util.Collections;\npublic class WordUnmodifiable { static void r() { Collections.unmodifiableList(5); } }",
        "method unmodifiableList in class Collections cannot be applied to given types;",
        "no suitable method found for unmodifiableList(int) in class Collections",
        Wording::Differs("as above: javac has a single candidate"),
    ),
    (
        "WordFill",
        "import java.util.Arrays;\npublic class WordFill { static void r() { Arrays.fill(new int[2], \"s\"); } }",
        "no suitable method found for fill(int[],String)",
        "incompatible types: String cannot be converted to int",
        Wording::Differs(
            "caturra names the offending argument, which is what the student \
             needs; javac reports overload resolution failing across eighteen \
             overloads",
        ),
    ),
    (
        "WordAddAll",
        "import java.util.*;\npublic class WordAddAll { static void r() { ArrayList<Integer> l = new ArrayList<Integer>(); int[] v = {1}; Collections.addAll(l, v); } }",
        "method addAll in class Collections cannot be applied to given types;",
        "incompatible types: int[] cannot be converted to int",
        Wording::Differs("as above: caturra names the argument"),
    ),
    (
        "WordListSearch",
        "import java.util.*;\npublic class WordListSearch { static int r() { ArrayList<Integer> l = new ArrayList<Integer>(); return Collections.binarySearch(l, \"x\"); } }",
        "no suitable method found for binarySearch(ArrayList<Integer>,String)",
        "incompatible types: String cannot be converted to int",
        Wording::Differs("as above: caturra names the argument"),
    ),
    (
        "WordCopyOf",
        "import java.util.Arrays;\npublic class WordCopyOf { static void r() { String[] w = {\"a\"}; int[] x = Arrays.copyOf(w, 2); } }",
        "incompatible types: inference variable T has incompatible bounds",
        "incompatible types: String[] cannot be converted to int[]",
        Wording::Differs(
            "javac explains its generic inference; caturra names the two array \
             types, which is the same fact without the machinery",
        ),
    ),
    (
        "WordNextBytes",
        "import java.util.Random;\npublic class WordNextBytes { static void r() { new Random(1).nextBytes(new int[2]); } }",
        "incompatible types: int[] cannot be converted to byte[]",
        "no suitable method found for nextBytes(int[]) in class Random",
        Wording::Differs(
            "the mirror of `WordFill`: here javac names the argument and caturra \
             reports the overload, because nextBytes is a bundled Java method \
             and goes through user-method resolution",
        ),
    ),
    (
        "WordSortPlainList",
        "import java.util.*;\nclass WordPlainA {}\npublic class WordSortPlainList { static void r() { Collections.sort(new ArrayList<WordPlainA>()); } }",
        "no suitable method found for sort(ArrayList<WordPlainA>)",
        "no suitable method found for sort(ArrayList<WordPlainA>) in class Collections",
        Wording::Prefix,
    ),
    (
        "WordSortObjectList",
        "import java.util.*;\npublic class WordSortObjectList { static void r() { Collections.sort(new ArrayList<Object>()); } }",
        "no suitable method found for sort(ArrayList<Object>)",
        "no suitable method found for sort(ArrayList<Object>) in class Collections",
        Wording::Prefix,
    ),
    (
        "WordBsPlainList",
        "import java.util.*;\nclass WordPlainB {}\npublic class WordBsPlainList { static int r() { return Collections.binarySearch(new ArrayList<WordPlainB>(), new WordPlainB()); } }",
        "no suitable method found for binarySearch(ArrayList<WordPlainB>,WordPlainB)",
        "no suitable method found for binarySearch(ArrayList<WordPlainB>,WordPlainB) in class Collections",
        Wording::Prefix,
    ),
    (
        "WordSuperStatic",
        "class WSSa { protected int n = 1; }\nclass WSSb extends WSSa { static int f() { return super.n; } }\npublic class WordSuperStatic { static void r() {} }",
        "non-static variable super cannot be referenced from a static context",
        "non-static variable super cannot be referenced from a static context",
        Wording::Same,
    ),
    (
        "WordSuperPrivate",
        "class WSPa { private int n = 1; }\nclass WSPb extends WSPa { int f() { return super.n; } }\npublic class WordSuperPrivate { static void r() {} }",
        "n has private access in WSPa",
        "n has private access in WSPa",
        Wording::Same,
    ),
    (
        "WordSuperNoSuperclass",
        "class WSNb { int n = 1; int f() { return super.n; } }\npublic class WordSuperNoSuperclass { static void r() {} }",
        "cannot find symbol",
        "cannot find symbol: field 'n' in class Object",
        Wording::Prefix,
    ),
    (
        "WordQualifiedClass",
        "public class WordQualifiedClass { static void r() { java.util.Nope.f(); } }",
        "cannot find symbol",
        "cannot find symbol: class Nope in package java.util",
        Wording::Prefix,
    ),
    (
        "WordQualifiedPackage",
        "public class WordQualifiedPackage { static void r() { java.zzz.Nope.f(); } }",
        "package java.zzz does not exist",
        "package java.zzz does not exist",
        Wording::Same,
    ),
    (
        "WordOmittedElement",
        "public class WordOmittedElement { static void r() { int[] c = {1,,2}; } }",
        "illegal start of expression",
        "expected an expression",
        Wording::Differs(
            "a generic parser message, not a library one; changing it would touch \
             every expression position",
        ),
    ),
];

/// caturra's reject wording, checked against javac's rather than against our
/// own memory of it — the tests that assert these strings in isolation cannot
/// tell you whether javac still says the same thing.
#[test]
fn reject_wording_tracks_javac() {
    if !jdk_available() {
        eprintln!("skipping: no JDK on PATH");
        return;
    }
    for (class_name, source, javac_headline, caturra_message, wording) in REJECT_WORDING {
        let javac = javac_first_error(class_name, source);
        assert_eq!(
            javac.as_deref(),
            Some(*javac_headline),
            "javac's wording for {class_name} changed"
        );
        let caturra = caturra_first_error(class_name, source);
        assert_eq!(
            caturra.as_deref(),
            Some(*caturra_message),
            "caturra's wording for {class_name} changed"
        );
        match wording {
            Wording::Same => assert_eq!(
                caturra_message, javac_headline,
                "{class_name} is recorded as Same"
            ),
            Wording::Prefix => assert!(
                caturra_message.starts_with(javac_headline) && caturra_message != javac_headline,
                "{class_name} is recorded as Prefix, but caturra's message does not \
                 extend javac's headline"
            ),
            Wording::Differs(reason) => assert!(
                !caturra_message.starts_with(javac_headline),
                "{class_name} is recorded as Differs, but caturra now follows javac — \
                 promote it to Prefix or Same. The recorded reason to differ was: {reason}"
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Where caturra and javac deliberately disagree. Both directions are asserted
// against a live javac, so neither list can drift: a `stricter` case that
// javac starts rejecting is no longer a strictness, and a `looser` case that
// caturra starts rejecting should be deleted from the list, not left passing.
// ---------------------------------------------------------------------------

stricter_than_javac!(
    strict_arraycopy_rejects_a_non_array_source,
    "StrictArrayCopy",
    "public class StrictArrayCopy { static void r() { int[] c = new int[2]; System.arraycopy(\"a\", 0, c, 0, 1); } }"
);

stricter_than_javac!(
    strict_string_builder_has_no_capacity,
    "StrictSbCapacity",
    "public class StrictSbCapacity { static int r() { return new StringBuilder().capacity(); } }"
);

stricter_than_javac!(
    strict_fill_checks_the_element_type_of_a_reference_array,
    "StrictFillObjArr",
    "import java.util.Arrays;\npublic class StrictFillObjArr { static void r() { Arrays.fill(new String[1], 5); } }"
);

stricter_than_javac!(
    strict_array_sort_demands_a_comparable_element,
    "StrictSortPlainArr",
    "import java.util.Arrays;\nclass StrictPlainA {}\npublic class StrictSortPlainArr { static void r() { Arrays.sort(new StrictPlainA[2]); } }"
);

stricter_than_javac!(
    strict_frequency_demands_the_lists_element_type,
    "StrictFreqWrongType",
    "import java.util.*;\npublic class StrictFreqWrongType { static int r() { return Collections.frequency(new ArrayList<Integer>(), \"x\"); } }"
);

stricter_than_javac!(
    strict_contains_all_demands_the_lists_element_type,
    "StrictContainsAllOther",
    "import java.util.*;\npublic class StrictContainsAllOther { static boolean r() { return new ArrayList<Integer>().containsAll(new ArrayList<String>()); } }"
);

stricter_than_javac!(
    strict_collections_add_all_takes_no_array,
    "StrictAddAllBoxedArr",
    "import java.util.*;\npublic class StrictAddAllBoxedArr { static void r() { Collections.addAll(new ArrayList<Integer>(), new Integer[] {1}); } }"
);

// `Collections.sort`/`max`/`min`/`binarySearch` over a non-`Comparable`
// element used to be the only place caturra accepted code javac rejects.
// They were `looser_than_javac!` cases until 2026-07-09; now both compilers
// refuse them, so they are asserted as the shared rule they became.
differential_reject!(
    reject_sorting_a_list_of_non_comparables,
    "RejectSortPlainList",
    "import java.util.*;\nclass RejPlainA {}\npublic class RejectSortPlainList { static void r() { Collections.sort(new ArrayList<RejPlainA>()); } }"
);

differential_reject!(
    reject_max_of_a_list_of_non_comparables,
    "RejectMaxPlainList",
    "import java.util.*;\nclass RejPlainB {}\npublic class RejectMaxPlainList { static void r() { Collections.max(new ArrayList<RejPlainB>()); } }"
);

differential_reject!(
    reject_min_of_a_list_of_non_comparables,
    "RejectMinPlainList",
    "import java.util.*;\nclass RejPlainD {}\npublic class RejectMinPlainList { static void r() { Collections.min(new ArrayList<RejPlainD>()); } }"
);

differential_reject!(
    reject_binary_search_of_a_list_of_non_comparables,
    "RejectBsPlainList",
    "import java.util.*;\nclass RejPlainC {}\npublic class RejectBsPlainList { static void r() { Collections.binarySearch(new ArrayList<RejPlainC>(), new RejPlainC()); } }"
);

differential_reject!(
    reject_sorting_a_list_of_object,
    "RejectSortObjectList",
    "import java.util.*;\npublic class RejectSortObjectList { static void r() { Collections.sort(new ArrayList<Object>()); } }"
);
