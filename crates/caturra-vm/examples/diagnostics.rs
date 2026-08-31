//! Every diagnostic a program produces, one per line, as `severity: message`.
//!
//! `compatrun` prints the FIRST error, which is what a differential pin
//! compares — and for years that was the only thing compared, so how many
//! errors caturra reports, and in what ORDER, was never measured against a
//! JDK. It differed: javac runs its flow phase only after attribution
//! succeeded, and attributes nothing that failed to parse, so a program with
//! both kinds of mistake is reported with one of them alone.
//!
//!     cargo run --release -p caturra-vm --example diagnostics -- Foo.java [More.java ...]
//!
//! `scripts/fuzz/diaglist.py` runs this beside javac over a directory of
//! cases and compares the whole list.

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    assert!(
        !paths.is_empty(),
        "usage: diagnostics <file.java> [more.java ...]"
    );
    let sources: Vec<caturra_compiler::SourceFile> = paths
        .iter()
        .map(|path| {
            let text = std::fs::read_to_string(path).expect("read source");
            let name = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Main.java")
                .to_owned();
            caturra_compiler::SourceFile { path: name, text }
        })
        .collect();
    let compilation = caturra_compiler::compile(&sources);
    for diagnostic in &compilation.diagnostics {
        // The FIRST line only: javac's continuation lines (`symbol:`,
        // `required:`) are indented, and this list is compared with the one
        // `javac` prints on its `error:` lines.
        // The POSITION too: javac prints a line on every error and a caret
        // under a column, and nothing had ever compared either — a message can
        // be right and point at the wrong place, which is what an editor
        // underlines.
        let (line, column) = diagnostic
            .span
            .map_or((0, 0), |span| (span.start.line, span.start.column));
        println!(
            "{:?}@{line},{column}: {}",
            diagnostic.severity,
            diagnostic.message.lines().next().unwrap_or("")
        );
    }
}
