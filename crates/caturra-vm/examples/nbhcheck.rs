//! Corpus-level verifier used by the playground's level generator
//! (`apps/playground/scripts/generate-csa-levels.py`). Reads a `.rec` file of
//! `F:name\x1ftext` (Java source), `D:name\x1ftext` (data files written to the
//! VFS), and `G:grid` records, compiles + runs the level under a low
//! instruction budget, and prints a one-line status:
//!   `OK <neighborhood-actions> VT<vtest-lines>` | `CE: <msg>` |
//!   `RUN: <err>` | `BUDGET` (infinite loop) | `LOAD: <err>`.
use std::io::Read;

fn find_main(sources: &[caturra_compiler::SourceFile]) -> Option<String> {
    for f in sources {
        if let Some(i) = f.text.find("static void main")
            && let Some(c) = f.text[..i].rfind("class ")
        {
            let name: String = f.text[c + 6..]
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn main() {
    let mut buf = String::new();
    std::fs::File::open(std::env::args().nth(1).unwrap())
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();
    let mut files = Vec::new();
    let mut grid = String::new();
    let mut data: Vec<(String, String)> = Vec::new();
    for part in buf.split('\u{1e}') {
        if let Some(r) = part.strip_prefix("F:") {
            if let Some((n, t)) = r.split_once('\u{1f}') {
                files.push(caturra_compiler::SourceFile {
                    path: n.into(),
                    text: t.into(),
                });
            }
        } else if let Some(r) = part.strip_prefix("D:") {
            if let Some((n, t)) = r.split_once('\u{1f}') {
                data.push((n.into(), t.into()));
            }
        } else if let Some(g) = part.strip_prefix("G:") {
            grid = g.into();
        }
    }
    let compiled = caturra_compiler::compile(&files);
    if !compiled.success() {
        let err = compiled
            .diagnostics
            .iter()
            .find(|d| matches!(d.severity, caturra_compiler::diagnostics::Severity::Error));
        println!("CE: {}", err.map(|d| d.message.clone()).unwrap_or_default());
        return;
    }
    let entry = compiled
        .validation_entry
        .clone()
        .unwrap_or_else(|| find_main(&files).unwrap_or_default());
    let mut vfs = caturra_vm::VirtualFileSystem::new();
    if !grid.is_empty() {
        vfs.write_file("grid.txt", grid.into_bytes()).unwrap();
    }
    for (name, text) in &data {
        vfs.write_file(name, text.clone().into_bytes()).unwrap();
    }
    // Dummy stdin so Scanner-input levels run; a low budget so an infinite
    // loop trips quickly (the playground's real 500M budget stops it just as
    // safely at runtime).
    let input: Vec<String> = (0..200).map(|_| String::from("7")).collect();
    let mut console = caturra_vm::BufferedConsole::with_input(input);
    let options = caturra_vm::VmOptions {
        max_instructions: 20_000_000,
        ..caturra_vm::VmOptions::default()
    };
    let mut vm = caturra_vm::Vm::new(options, &mut vfs, &mut console);
    for class in compiled.classes {
        if let Err(e) = vm.load_class(class.class_file) {
            println!("LOAD: {e:?}");
            return;
        }
    }
    match vm.run_main(&entry, &[]) {
        Ok(_) => {
            let vt = console
                .stdout_text()
                .lines()
                .filter(|l| l.starts_with("__VTEST"))
                .count();
            let actions = vfs
                .read_file("neighborhood.jsonl")
                .map_or(0, |b| String::from_utf8_lossy(b).lines().count());
            println!("OK {actions} VT{vt}");
        }
        Err(caturra_vm::VmError::InstructionBudgetExceeded) => println!("BUDGET"),
        Err(e) => println!("RUN: {e:?}"),
    }
}
