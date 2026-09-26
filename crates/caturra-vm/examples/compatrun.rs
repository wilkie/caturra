//! Run one Java source file through caturra and report what happened, as JSON:
//! `{"ok": true, "stdout": "…"}` or `{"ok": false, "error": "…"}`.
//!
//! The companion to `scripts/compat-record.py`, which asks the same question of a
//! real JDK. Between them they establish what caturra actually supports, which is
//! what the compatibility page publishes — and `tests/compat_manifest.rs` keeps
//! the two honest.
//!
//! usage: compatrun <file.java> <MainClass>

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

fn escape(text: &str) -> String {
    let mut out = String::new();
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Every other CONTROL character has to be escaped too, or the
            // JSON is unparseable: a program that prints one — `setLength`
            // pads with NULs, and a `(char) 1` is an ordinary value — made
            // this output unreadable, which every reader of it (the compat
            // recorder, the fuzz runner) reported as "printed no JSON"
            // instead of as a difference.
            other if (other as u32) < 0x20 || other == '\u{7f}' => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", other as u32);
            }
            other => out.push(other),
        }
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: compatrun <file.java> <MainClass> [more.java ...]");
    let main_class = args.next().expect("main class");
    let text = std::fs::read_to_string(&path).expect("read source");

    // A program may be spread over several files — a corpus level is a student
    // class beside the `Main` that drives it — so any extra paths are compiled
    // with the first. One file remains the ordinary case.
    let mut sources = vec![caturra_compiler::SourceFile {
        path: path.clone(),
        text,
    }];
    let rest: Vec<String> = args.collect();
    let mut wants_stdin = false;
    for extra in &rest {
        if extra == "--stdin" {
            wants_stdin = true;
            continue;
        }
        let text = std::fs::read_to_string(extra).expect("read source");
        let name = std::path::Path::new(extra)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Extra.java")
            .to_owned();
        sources.push(caturra_compiler::SourceFile { path: name, text });
    }
    let compilation = caturra_compiler::compile(&sources);
    if !compilation.success() {
        let first = compilation
            .diagnostics
            .iter()
            .find(|d| format!("{:?}", d.severity) == "Error");
        let message = first.map_or_else(String::new, |d| d.message.clone());
        // ...and the LINE it is on. javac names one and caturra's diagnostics
        // carry one; only this harness dropped it, so a reader with several
        // candidate lines — the coverage sweep, deciding which call to leave
        // out — had to guess from the wording and gave up when it could not.
        let line = first.and_then(|d| d.span).map_or(0, |span| span.start.line);
        // ...and EVERY line, not only the first's. javac reports the whole
        // list and its reader drops them all at once; caturra reports the
        // whole list too, and passing only one of them made a probe with a
        // hundred bad calls take a hundred rounds — which is how
        // `java.util.Arrays` ran the sweep's retry budget out.
        let mut lines: Vec<String> = compilation
            .diagnostics
            .iter()
            .filter(|d| format!("{:?}", d.severity) == "Error")
            .filter_map(|d| d.span.map(|span| span.start.line.to_string()))
            .collect();
        lines.dedup();
        println!(
            "{{\"ok\": false, \"error\": \"{}\", \"line\": {line}, \"lines\": [{}]}}",
            escape(&message),
            lines.join(",")
        );
        return;
    }

    let mut vfs = VirtualFileSystem::new();
    // Any file beside the program is staged into the virtual filesystem under
    // its bare name, so a probe that reads `data.txt` finds the same bytes the
    // JDK reads from the directory it ran in.
    let dir = match std::path::Path::new(&path).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => std::path::PathBuf::from("."),
    };
    {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let is_source = entry.path().extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("java") || ext.eq_ignore_ascii_case("class")
                });
                if is_source {
                    continue;
                }
                if let Ok(bytes) = std::fs::read(entry.path()) {
                    let _ = vfs.seed_file(&name, bytes);
                }
            }
        }
    }
    // Standard input, when the program reads any: passed through as lines, the
    // shape `BufferedConsole` scripts. A probe that compares a Scanner-driven
    // program needs the same input on both engines.
    let mut stdin_lines: Vec<String> = Vec::new();
    if wants_stdin {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut text).expect("read stdin");
        stdin_lines = text.lines().map(ToOwned::to_owned).collect();
    }
    let mut console = BufferedConsole::with_input(stdin_lines);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        if let Err(err) = vm.load_class(class.class_file) {
            drop(vm);
            println!(
                "{{\"ok\": false, \"error\": \"load: {}\"}}",
                escape(&err.to_string())
            );
            return;
        }
    }
    let outcome = vm.run_main(&main_class, &[]);
    drop(vm);
    match outcome {
        Ok(_) => println!(
            "{{\"ok\": true, \"stdout\": \"{}\", \"stderr\": \"{}\"}}",
            escape(&console.stdout_text()),
            escape(&console.stderr_text())
        ),
        Err(err) => println!(
            "{{\"ok\": false, \"error\": \"{}\", \"stdout\": \"{}\", \"stderr\": \"{}\"}}",
            escape(&err.to_string()),
            escape(&console.stdout_text()),
            escape(&console.stderr_text())
        ),
    }
}
