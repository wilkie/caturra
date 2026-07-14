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
            other => out.push(other),
        }
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: compatrun <file.java> <MainClass>");
    let main_class = args.next().expect("main class");
    let text = std::fs::read_to_string(&path).expect("read source");

    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: path.clone(),
        text,
    }]);
    if !compilation.success() {
        let first = compilation
            .diagnostics
            .iter()
            .find(|d| format!("{:?}", d.severity) == "Error")
            .map_or_else(String::new, |d| d.message.clone());
        println!("{{\"ok\": false, \"error\": \"{}\"}}", escape(&first));
        return;
    }

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
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
            "{{\"ok\": true, \"stdout\": \"{}\"}}",
            escape(&console.stdout_text())
        ),
        Err(err) => println!(
            "{{\"ok\": false, \"error\": \"{}\", \"stdout\": \"{}\"}}",
            escape(&err.to_string()),
            escape(&console.stdout_text())
        ),
    }
}
