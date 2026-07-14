//! The compatibility page publishes a claim about every feature it lists. This is
//! what stops that claim from quietly becoming false.
//!
//! For each entry in `apps/playground/src/compat/features.json`:
//!
//! - **supported** — caturra runs the program and prints exactly the `expected`
//!   output the manifest records, AND a real `javac`/`java` prints that same
//!   output. So "supported" means *the JDK and caturra agree*, checked against the
//!   JDK on every run, not "someone ticked a box".
//! - **unsupported** — **javac ACCEPTS the program** and caturra rejects it, saying
//!   why. Requiring javac to accept it is what makes the gap honest: it proves we
//!   are declining real Java 11, not inventing a limitation for something that was
//!   never Java in the first place.
//! - **beyond-11** — javac 11 rejects it too, so rejecting it is the *correct*
//!   behaviour. A feature caturra accepted here would be a program that runs in the
//!   playground and fails on the JDK the course targets.
//!
//! The page then runs the very same programs live in the browser and shows the
//! result against `expected`, so a regression shows up on the page itself.
//!
//! Regenerate the manifest with `scripts/compat/record.py` — it asks both engines
//! and writes down what they say.

use std::process::Command;

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

fn manifest() -> Vec<serde_json::Value> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../apps/playground/src/compat/features.json"
    );
    let text = std::fs::read_to_string(path).expect("features.json");
    serde_json::from_str(&text).expect("features.json is valid JSON")
}

fn jdk_available() -> bool {
    let ok = Command::new("javac")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success());
    assert!(
        ok || std::env::var_os("CATURRA_REQUIRE_JDK").is_none(),
        "CATURRA_REQUIRE_JDK is set but no JDK is on PATH: this suite would have \
         reported `ok` without checking a single claim against real Java"
    );
    ok
}

/// `(compiles, stdout)` from a real javac + java.
fn real_java(source: &str, main: &str) -> (bool, String) {
    let dir = std::env::temp_dir().join(format!("caturra-compat-{main}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    let file = dir.join(format!("{main}.java"));
    std::fs::write(&file, source).expect("write source");

    let compiled = Command::new("javac")
        .args(["-nowarn", "-d"])
        .arg(&dir)
        .arg(&file)
        .output()
        .expect("javac runs");
    if !compiled.status.success() {
        let _ = std::fs::remove_dir_all(&dir);
        return (false, String::new());
    }
    let ran = Command::new("java")
        .arg("-cp")
        .arg(&dir)
        .arg(main)
        .current_dir(&dir)
        .output()
        .expect("java runs");
    let _ = std::fs::remove_dir_all(&dir);
    (true, String::from_utf8_lossy(&ran.stdout).into_owned())
}

/// `Ok(stdout)` if caturra compiles and runs it, `Err(first error)` if not.
fn caturra(source: &str, main: &str) -> Result<String, String> {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{main}.java"),
        text: source.to_owned(),
    }]);
    if !compilation.success() {
        let first = compilation
            .diagnostics
            .iter()
            .find(|d| format!("{:?}", d.severity) == "Error")
            .map_or_else(String::new, |d| d.message.clone());
        return Err(first);
    }
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("class loads");
    }
    let outcome = vm.run_main(main, &[]);
    drop(vm);
    match outcome {
        Ok(_) => Ok(console.stdout_text()),
        Err(err) => Err(err.to_string()),
    }
}

fn field<'v>(entry: &'v serde_json::Value, name: &str) -> &'v str {
    entry[name]
        .as_str()
        .unwrap_or_else(|| panic!("{name} missing from {}", entry["id"]))
}

/// Every "supported" feature really is: caturra's output IS the JDK's output.
#[test]
fn supported_features_match_a_real_jdk() {
    let jdk = jdk_available();
    if !jdk {
        eprintln!("skipping the JDK half: no javac on PATH");
    }
    for entry in manifest() {
        if field(&entry, "status") != "supported" {
            continue;
        }
        let (id, main) = (field(&entry, "id"), field(&entry, "main"));
        let (source, expected) = (field(&entry, "source"), field(&entry, "expected"));

        let actual = caturra(source, main)
            .unwrap_or_else(|err| panic!("{id} is listed as supported, but caturra says: {err}"));
        assert_eq!(
            actual, expected,
            "{id}: caturra's output is not what the manifest promises the page"
        );

        if jdk {
            let (compiles, from_jdk) = real_java(source, main);
            assert!(
                compiles,
                "{id}: javac rejects the program we ship as working"
            );
            assert_eq!(
                from_jdk, expected,
                "{id}: the manifest's expected output is no longer what a real JDK prints"
            );
        }
    }
}

/// Every "unsupported" feature is REAL Java that we decline, with a reason.
///
/// javac has to accept it. Otherwise "not supported" could quietly cover a
/// program that was never valid Java at all, and the page would be taking credit
/// for a limitation it does not have.
#[test]
fn unsupported_features_are_real_java_that_caturra_declines() {
    let jdk = jdk_available();
    for entry in manifest() {
        if field(&entry, "status") != "unsupported" {
            continue;
        }
        let (id, main) = (field(&entry, "id"), field(&entry, "main"));
        let (source, reason) = (field(&entry, "source"), field(&entry, "reason"));

        let outcome = caturra(source, main);
        assert!(
            outcome.is_err(),
            "{id} is listed as unsupported, but caturra runs it: {outcome:?}"
        );
        assert_eq!(
            outcome.unwrap_err(),
            reason,
            "{id}: caturra no longer gives the reason the page prints"
        );

        if jdk {
            let (compiles, _) = real_java(source, main);
            assert!(
                compiles,
                "{id} is listed as a gap in Java 11 support, but javac rejects it too — \
                 so it is not a gap, and the page must not claim one"
            );
        }
    }
}

/// Every "beyond-11" feature is newer than Java 11 — javac 11 turns it down as
/// well, which is what makes caturra's refusal correct rather than a shortfall.
#[test]
fn beyond_java_11_features_are_rejected_by_javac_too() {
    let jdk = jdk_available();
    for entry in manifest() {
        if field(&entry, "status") != "beyond-11" {
            continue;
        }
        let (id, main) = (field(&entry, "id"), field(&entry, "main"));
        let source = field(&entry, "source");

        assert!(
            caturra(source, main).is_err(),
            "{id}: caturra accepts post-Java-11 syntax — a program that runs here and \
             fails on the JDK the course targets"
        );
        if jdk {
            let (compiles, _) = real_java(source, main);
            assert!(
                !compiles,
                "{id} is filed as beyond Java 11, but javac 11 compiles it — so it IS \
                 Java 11, and refusing it is a gap we should own"
            );
        }
    }
}

/// The page groups by category and keys by id; neither may be empty or repeat.
#[test]
fn the_manifest_is_well_formed() {
    let entries = manifest();
    assert!(!entries.is_empty(), "no features");
    let mut ids = std::collections::HashSet::new();
    for entry in &entries {
        let id = field(entry, "id");
        assert!(ids.insert(id.to_owned()), "duplicate feature id: {id}");
        for required in ["category", "title", "summary", "main", "source", "status"] {
            assert!(!field(entry, required).is_empty(), "{id}: empty {required}");
        }
        let status = field(entry, "status");
        assert!(
            matches!(status, "supported" | "unsupported" | "beyond-11"),
            "{id}: unknown status {status}"
        );
    }
}
