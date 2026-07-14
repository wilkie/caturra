//! The caturra half of the corpus GRADING sweep — the companion to
//! `tests/differential_validation.rs`, scaled from a handful of hand-written
//! validators to every level in the corpus. It compiles each staged level's
//! sources + validator, runs its tests, and prints one row per level:
//!
//!   `name <TAB> OK <TAB> <__VTEST line>\x01<__VTEST line>...`
//!   `name <TAB> CE <TAB> <first error>` | `NOTESTS` | `NOENTRY` | `LOADFAIL`
//!
//! The other half runs the same levels through a real JVM (real `JUnit` 5, and
//! for `org.code` levels the real media/neighborhood/validation libraries), and
//! the two sets of rows are compared verdict by verdict. That comparison is the
//! only thing that says whether caturra grades a student the way Code.org does.
//!
//! usage: `valsweep <cases-dir> <packed-assets-dir> > caturra.tsv`
//!
//! `<cases-dir>/<n>/` holds one level: its `*.java` (student sources with
//! `start/` merged in and the main class renamed to `Main`, exactly as
//! `generate-csa-levels.py` stages it — get that wrong and you manufacture
//! hundreds of phantom failures), a `name` file, and any `*.txt`/`*.csv` the
//! level reads (`grid.txt` included). `<packed-assets-dir>/<level name>/` holds
//! `__packed_<asset>` files: the level's images already decoded by the REAL
//! `org.code.media` library, so that an image decode is never what the sweep is
//! comparing. Both are gitignored corpora, so this reads them by path.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

/// The verdicts a run printed, with a planned-but-unreported test counted as a
/// failure: a VM error is not a Java throwable, so a dead run must not be able
/// to pass for a short one.
fn verdicts(stdout: &str) -> Vec<String> {
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
    lines
}

fn one(case: &Path, packed: &Path) -> Option<(String, String)> {
    let name = std::fs::read_to_string(case.join("name")).ok()?;
    let name = name.trim().to_owned();

    let mut sources = Vec::new();
    let mut extras: Vec<(String, Vec<u8>)> = Vec::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(case)
        .ok()?
        .filter_map(|e| Some(e.ok()?.path()))
        .collect();
    entries.sort();
    for path in entries {
        let file = path.file_name()?.to_str()?.to_owned();
        match path.extension().and_then(|e| e.to_str()) {
            Some("java") => sources.push(caturra_compiler::SourceFile {
                path: file,
                text: String::from_utf8_lossy(&std::fs::read(&path).ok()?).into_owned(),
            }),
            Some("txt" | "csv") => extras.push((file, std::fs::read(&path).ok()?)),
            _ => {}
        }
    }
    if !sources.iter().any(|s| s.text.contains("org.junit")) {
        return Some((name, "NOTESTS".to_owned()));
    }

    // This level's assets, decoded by the REAL library and packed as the host
    // preloads them, so an image decode is not what the sweep compares.
    let mut assets: Vec<(String, Vec<u8>)> = Vec::new();
    for entry in std::fs::read_dir(packed.join(&name))
        .into_iter()
        .flatten()
        .flatten()
    {
        let file = entry.file_name().to_string_lossy().into_owned();
        if let Some(asset) = file.strip_prefix("__packed_")
            && let Ok(bytes) = std::fs::read(entry.path())
        {
            assets.push((format!("__caturra_image_{asset}"), bytes));
        }
    }

    let compilation = caturra_compiler::compile(&sources);
    if !compilation.success() {
        let first = compilation
            .diagnostics
            .iter()
            .find(|d| format!("{:?}", d.severity) == "Error")
            .map_or_else(String::new, |d| d.message.replace(['\t', '\n'], " "));
        return Some((name, format!("CE\t{first}")));
    }
    let Some(entry) = compilation.validation_entry.clone() else {
        return Some((name, "NOENTRY".to_owned()));
    };

    let mut vfs = VirtualFileSystem::new();
    for (file, bytes) in extras.into_iter().chain(assets) {
        let _ = vfs.write_file(&file, bytes);
    }
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        if vm.load_class(class.class_file).is_err() {
            drop(vm);
            return Some((name, "LOADFAIL".to_owned()));
        }
    }
    let _ = vm.run_main(&entry, &[]);
    drop(vm);
    let lines = verdicts(&console.stdout_text());
    Some((name, format!("OK\t{}", lines.join("\u{1}"))))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let cases = PathBuf::from(args.next().expect("cases dir"));
    let packed = PathBuf::from(args.next().expect("packed assets dir"));

    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&cases)
        .expect("cases")
        .filter_map(|e| {
            let path = e.ok()?.path();
            path.is_dir().then_some(path)
        })
        .collect();
    dirs.sort();

    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(dir) = dirs.get(index) else { return };
                    if let Some(row) = one(dir, &packed) {
                        out.lock().expect("lock").push(row);
                    }
                }
            });
        }
    });

    let mut rows = out.into_inner().expect("rows");
    rows.sort();
    for (name, rest) in rows {
        println!("{name}\t{rest}");
    }
}
