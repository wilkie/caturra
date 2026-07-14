//! Differential tests for the GRADING path: run a validator through **real
//! `JUnit` 5** and through caturra, and require the same verdict for every test.
//!
//! This is the surface that decides whether a student's work is marked correct.
//! caturra cannot *run* `JUnit` — its VM cannot even parse a class file containing
//! a lambda (`CONSTANT_InvokeDynamic` is not in its constant pool), and it has no
//! JDK to load `JUnit`'s own dependencies from — so it models `JUnit` instead, in
//! `stdlib/junit.java` plus a synthesized `__ValidationRunner`. A model can be
//! wrong; this is what says so. Same architecture as `differential_media.rs`:
//! the real library runs on a real JVM, caturra runs its model, and the outputs
//! are compared.
//!
//! The reference mirrors Code.org's own `BaseTestRunner` (vendored in
//! `javabuilder/lib`): select each test class, build a `LauncherDiscoveryRequest`,
//! execute it, and record what the listener reports. Nothing else in
//! `BaseTestRunner` bears on a verdict — the rest is message plumbing for the
//! browser — and compiling it directly would mean shimming half of a module that
//! declares the AWS SDK. The `JUnit` version is not ours to pick: `javabuilder`
//! runs student validators on jupiter 5.6.0 under platform-launcher 1.8.1, so
//! `scripts/fetch-test-jars.py` pins exactly that.
//!
//! Needs `pnpm testjars:fetch` (a gitignored `vendor/junit/`) and a JDK; skips
//! with a note otherwise, so a checkout without them still builds. Set
//! `CATURRA_REQUIRE_JUNIT=1` to make a missing one a failure instead.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

/// Prints one `__VTEST\t<PASS|FAIL>\t<name>\t<message>` line per test — the very
/// format caturra's synthesized `__ValidationRunner` prints, so the two engines'
/// output can be compared line for line.
const REF_RUNNER: &str = r#"
import java.io.PrintStream;
import java.util.ArrayList;
import java.util.List;
import org.junit.platform.engine.TestExecutionResult;
import org.junit.platform.engine.discovery.ClassSelector;
import org.junit.platform.engine.discovery.DiscoverySelectors;
import org.junit.platform.launcher.Launcher;
import org.junit.platform.launcher.LauncherDiscoveryRequest;
import org.junit.platform.launcher.TestExecutionListener;
import org.junit.platform.launcher.TestIdentifier;
import org.junit.platform.launcher.TestPlan;
import org.junit.platform.launcher.core.LauncherDiscoveryRequestBuilder;
import org.junit.platform.launcher.core.LauncherFactory;

public class RefRunner {
  public static void main(String[] args) throws Exception {
    // JUnit chatters on stdout; keep it out of the comparison.
    PrintStream out = System.out;
    System.setOut(System.err);

    List<ClassSelector> selectors = new ArrayList<>();
    for (String name : args) {
      selectors.add(DiscoverySelectors.selectClass(Class.forName(name)));
    }
    LauncherDiscoveryRequest request =
        LauncherDiscoveryRequestBuilder.request().selectors(selectors).build();
    Launcher launcher = LauncherFactory.create();
    launcher.registerTestExecutionListeners(
        new TestExecutionListener() {
          @Override
          public void executionFinished(TestIdentifier id, TestExecutionResult result) {
            if (!id.isTest()) {
              return;
            }
            boolean passed = result.getStatus() == TestExecutionResult.Status.SUCCESSFUL;
            String message = result.getThrowable().map(Throwable::getMessage).orElse("");
            if (message == null) {
              message = "";
            }
            out.println(
                "__VTEST\t"
                    + (passed ? "PASS" : "FAIL")
                    + "\t"
                    + id.getDisplayName()
                    + "\t"
                    + message.replace('\t', ' ').replace('\n', ' '));
          }
        });
    TestPlan plan = launcher.discover(request);
    if (plan.containsTests()) {
      launcher.execute(plan);
    }
    out.flush();
  }
}
"#;

/// The verdicts out of a run's stdout, one per test, in the order the engine ran
/// them. Both engines print the same `__VTEST` format, so this is the whole
/// comparison.
///
/// caturra also announces its roster up front (`__VPLAN`), because a VM error is
/// not a Java throwable and so cannot be caught by the generated runner: an
/// unsupported intrinsic or an exhausted budget kills the run where it stands.
/// A planned test with no verdict therefore did not run, and is reported here as
/// a failure. Silently dropping it is the one thing a grader must never do —
/// that is how a run that died after two of four tests came to say "2/2 passed".
fn verdicts(stdout: &str) -> String {
    let mut planned: Vec<&str> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut reported: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for line in stdout.lines() {
        if let Some(name) = line.strip_prefix("__VPLAN\t") {
            planned.push(name);
        } else if line.starts_with("__VTEST\t") {
            if let Some(name) = line.split('\t').nth(2) {
                reported.insert(name);
            }
            lines.push(line.to_owned());
        }
    }
    for name in planned {
        if !reported.contains(name) {
            lines.push(format!(
                "__VTEST\tFAIL\t{name}\tthe engine stopped before this test ran"
            ));
        }
    }
    let mut out = String::new();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

/// The `JUnit` 5 jars, if `pnpm testjars:fetch` has been run.
fn junit_classpath() -> Option<String> {
    let dir = workspace_root().join("vendor/junit");
    let mut jars: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()? == "jar").then_some(path)
        })
        .collect();
    if jars.is_empty() {
        return None;
    }
    jars.sort();
    Some(
        jars.iter()
            .map(|jar| jar.display().to_string())
            .collect::<Vec<_>>()
            .join(":"),
    )
}

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

fn reference_available() -> bool {
    let available = jdk_available() && junit_classpath().is_some();
    assert!(
        available || std::env::var_os("CATURRA_REQUIRE_JUNIT").is_none(),
        "CATURRA_REQUIRE_JUNIT is set, but {}: this suite would have reported \
         `ok` without comparing anything",
        if jdk_available() {
            "vendor/junit is empty — run `pnpm testjars:fetch`"
        } else {
            "no JDK is on PATH"
        }
    );
    if !available {
        eprintln!(
            "skipping: {}",
            if jdk_available() {
                "no JUnit 5 jars — run `pnpm testjars:fetch`"
            } else {
                "no JDK on PATH"
            }
        );
    }
    available
}

/// Compile `RefRunner` against the real `JUnit` once per test binary.
fn ref_runner_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let classpath = junit_classpath().expect("caller checked the jars are present");
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("junit-ref-runner");
        std::fs::create_dir_all(&dir).expect("create dir");
        let source = dir.join("RefRunner.java");
        std::fs::write(&source, REF_RUNNER).expect("write RefRunner");
        let compile = Command::new("javac")
            .args(["-nowarn", "-cp", &classpath, "-d", "."])
            .arg("RefRunner.java")
            .current_dir(&dir)
            .output()
            .expect("javac runs");
        assert!(
            compile.status.success(),
            "RefRunner did not compile against the JUnit jars: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        dir
    })
}

/// Run `source` (one validator class) through the real `JUnit` 5.
fn run_with_reference(class_name: &str, source: &str) -> String {
    let classpath = junit_classpath().expect("checked");
    let runner = ref_runner_dir();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(source, &mut hasher);
    let fingerprint = std::hash::Hasher::finish(&hasher);
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("junit-{class_name}-{fingerprint:x}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join(format!("{class_name}.java")), source).expect("write source");

    let compile = Command::new("javac")
        .args(["-nowarn", "-cp", &classpath, "-d", "."])
        .arg(format!("{class_name}.java"))
        .current_dir(&dir)
        .output()
        .expect("javac runs");
    assert!(
        compile.status.success(),
        "javac rejected {class_name}: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new("java")
        .arg("-cp")
        .arg(format!("{classpath}:{}:.", runner.display()))
        .arg("RefRunner")
        .arg(class_name)
        .current_dir(&dir)
        .output()
        .expect("java runs");
    verdicts(&String::from_utf8_lossy(&run.stdout))
}

/// Run the same validator through caturra's `JUnit` model.
fn run_with_caturra(class_name: &str, source: &str) -> String {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{class_name}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "caturra rejected {class_name}: {:?}",
        compilation.diagnostics
    );
    let entry = compilation
        .validation_entry
        .clone()
        .expect("caturra found no tests to run");
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("class loads");
    }
    // A validator that kills the VM (an uncatchable error) is exactly the bug
    // this suite exists to catch, so the run's outcome is deliberately not
    // asserted: what is compared is the verdicts it managed to print.
    let _ = vm.run_main(&entry, &[]);
    drop(vm);
    verdicts(&console.stdout_text())
}

fn assert_same_verdicts(class_name: &str, source: &str) {
    let expected = run_with_reference(class_name, source);
    let actual = run_with_caturra(class_name, source);
    assert_eq!(
        actual, expected,
        "caturra grades {class_name} differently from real JUnit 5"
    );
}

macro_rules! validation_differential_test {
    ($name:ident, $class:literal, $source:literal) => {
        #[test]
        fn $name() {
            if !reference_available() {
                return;
            }
            assert_same_verdicts($class, $source);
        }
    };
}

// What a student reads in the results panel. JUnit is fussier about the label
// than it looks: `@DisplayName` is TRIMMED, and without one the default is
// `name()` — the parentheses are part of it. caturra printed the raw annotation
// and a bare method name, which mislabelled the tests of 1721 corpus levels.
// Also: `@Order` decides the sequence, and `@BeforeEach` runs before every test
// on a FRESH instance, so state never leaks from one test to the next.
validation_differential_test!(
    diff_junit_names_order_and_lifecycle,
    "NamesTest",
    r#"
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class NamesTest {
    int counter;

    @BeforeEach
    public void setUp() {
        counter = 10;
    }

    @Test
    @Order(1)
    @DisplayName("  padded, and JUnit trims it  ")
    public void alpha() {
        counter++;
        assertEquals(11, counter);
    }

    @Test
    @Order(2)
    @DisplayName("state does not leak between tests => ")
    public void bravo() {
        assertEquals(10, counter, "BeforeEach should have reset the counter");
    }

    @Test
    @Order(3)
    public void noDisplayNameSoJunitAddsParens() {
        assertTrue(true);
    }
}
"#
);

// What a failure says, and what it is. An assertion carries its message plus
// JUnit's own "expected: <x> but was: <y>"; a thrown exception carries its own.
// A test that throws does not stop the ones after it.
validation_differential_test!(
    diff_junit_failure_messages,
    "FailuresTest",
    r#"
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class FailuresTest {
    @Test
    @Order(1)
    public void assertionFails() {
        assertEquals(3, 4, "three is not four");
    }

    @Test
    @Order(2)
    public void exceptionEscapes() {
        throw new IllegalStateException("boom");
    }

    @Test
    @Order(3)
    public void deltaIsHonoured() {
        assertEquals(0.02, 0.02000001, 0.0001);
        assertEquals(0.02, 0.03, 0.0001, "outside the delta");
    }

    @Test
    @Order(4)
    public void stillRuns() {
        assertTrue(true);
    }
}
"#
);

// A reflective exception must be CATCHABLE. `ClassNotFoundException`,
// `NoSuchMethodException` and their siblings were missing from caturra's
// exception hierarchy — which does not mean "cannot be named in a catch", it
// means UNCATCHABLE: the throw escaped even `catch (Throwable)` and killed the
// run. The Unit 2 constructor validators throw exactly this through
// ConstructorsHelper, so every test after the first silently vanished and the
// student was told all of them passed.
validation_differential_test!(
    diff_junit_reflective_exceptions_are_catchable,
    "ReflectTest",
    r#"
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class ReflectTest {
    @Test
    @Order(1)
    @DisplayName("a reflective exception is caught, not fatal => ")
    public void reflectiveExceptionIsCatchable() {
        boolean caught = false;
        try {
            Class.forName("NoSuchClassAnywhere");
            fail("there is no such class");
        } catch (Throwable e) {
            caught = true;
        }
        assertTrue(caught, "ClassNotFoundException should be catchable");
    }

    @Test
    @Order(2)
    @DisplayName("and the tests after it still run => ")
    public void laterTestsStillRun() {
        assertTrue(true);
    }
}
"#
);

// `assertDoesNotThrow(() -> student.method(), message)` — a lambda bound to
// JUnit's `Executable`. Three separate things had to be true for this to
// compile, and none of them was:
//   * the JUnit model had no `Executable` and no `assertDoesNotThrow`;
//   * a lambda passed as a CALL ARGUMENT only got a target type when the method
//     was declared exactly once, and `assertDoesNotThrow` is declared twice;
//   * a captured enclosing instance field used as a METHOD RECEIVER inside the
//     lambda (`() -> testObject.check(...)`) did not resolve — reading such a
//     field worked, calling through it did not.
// The failure wording is JUnit's own.
validation_differential_test!(
    diff_junit_assert_does_not_throw,
    "DoesNotThrowTest",
    r#"
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

class Blog {
    private String title;

    Blog(String title) {
        this.title = title;
    }

    boolean checkForKeyword(String target) {
        // Walks off the end for a target longer than the title, unless the
        // loop header is right.
        for (int i = 0; i + target.length() <= title.length(); i++) {
            if (title.substring(i, i + target.length()).equals(target)) {
                return true;
            }
        }
        return false;
    }

    void explode() {
        throw new IllegalStateException("kaboom");
    }
}

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class DoesNotThrowTest {
    Blog testObject;

    @BeforeEach
    public void setup() {
        testObject = new Blog("artificialintelligence");
    }

    @Test
    @Order(1)
    @DisplayName("the method does not throw => ")
    public void doesNotThrow() {
        assertDoesNotThrow(() -> testObject.checkForKeyword("intel"), "should not have thrown");
    }

    @Test
    @Order(2)
    @DisplayName("and it still finds the keyword => ")
    public void findsIt() {
        assertTrue(testObject.checkForKeyword("intel"), "the keyword is in there");
    }

    @Test
    @Order(3)
    @DisplayName("a body that throws fails, with JUnit's wording => ")
    public void bodyThrows() {
        assertDoesNotThrow(() -> testObject.explode(), "this one blows up");
    }

    @Test
    @Order(4)
    @DisplayName("and without a message of its own => ")
    public void bodyThrowsUnnamed() {
        assertDoesNotThrow(() -> testObject.explode());
    }
}
"#
);

// `assertThrows(IllegalStateException.class, () -> student.method())`. A
// SUBCLASS counts — an IllegalStateException satisfies
// `assertThrows(RuntimeException.class, …)` — so this is a runtime type test,
// not a name comparison, and getting that wrong would fail correct work. It
// needed three things caturra did not have: `instanceof`/`isInstance` that climb
// the LIBRARY EXCEPTION hierarchy (nothing consulted that table, so every such
// test was false), `Throwable.getClass()`, and a `Class.getName()` that returns
// the binary name — it used to report the internal `java/lang/String`, which is
// not a name Java has ever produced.
validation_differential_test!(
    diff_junit_assert_throws,
    "ThrowsTest",
    r#"
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

class BadStudent extends RuntimeException {
    public BadStudent(String message) {
        super(message);
    }
}

class Fragile {
    void explode() {
        throw new IllegalStateException("kaboom");
    }

    void divide(int by) {
        int x = 10 / by;
    }

    void reject() {
        throw new BadStudent("no");
    }

    void fine() {
    }
}

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class ThrowsTest {
    Fragile testObject;

    @BeforeEach
    public void setup() {
        testObject = new Fragile();
    }

    @Test
    @Order(1)
    @DisplayName("the exact type is accepted => ")
    public void exactType() {
        assertThrows(IllegalStateException.class, () -> testObject.explode());
    }

    @Test
    @Order(2)
    @DisplayName("a SUPERTYPE is accepted too => ")
    public void superType() {
        assertThrows(RuntimeException.class, () -> testObject.explode());
        assertThrows(Throwable.class, () -> testObject.explode());
    }

    @Test
    @Order(3)
    @DisplayName("a real exception from real code => ")
    public void realException() {
        assertThrows(ArithmeticException.class, () -> testObject.divide(0));
    }

    @Test
    @Order(4)
    @DisplayName("nothing thrown is a failure => ")
    public void nothingThrown() {
        assertThrows(IllegalStateException.class, () -> testObject.fine());
    }

    @Test
    @Order(5)
    @DisplayName("the wrong type is a failure => ")
    public void wrongType() {
        assertThrows(NullPointerException.class, () -> testObject.explode());
    }

    @Test
    @Order(6)
    @DisplayName("and both failures carry a message when given one => ")
    public void withMessage() {
        assertThrows(NullPointerException.class, () -> testObject.explode(), "expected an NPE");
    }

    @Test
    @Order(7)
    @DisplayName("the thrown exception is RETURNED for inspection => ")
    public void returnsTheThrowable() {
        Throwable thrown = assertThrows(IllegalStateException.class, () -> testObject.explode());
        assertEquals("kaboom", thrown.getMessage(), "the message survives the throw");
    }

    @Test
    @Order(8)
    @DisplayName("a user-defined exception, by its own type and by its super => ")
    public void userDefined() {
        assertThrows(BadStudent.class, () -> testObject.reject());
        assertThrows(RuntimeException.class, () -> testObject.reject());
    }
}
"#
);
