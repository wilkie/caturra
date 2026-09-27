//! Compile Java sources with caturra and write the class files it produces to
//! a directory, so `javap -c -p` can show exactly what was emitted.
//!
//! `cargo run --release --example classdump -- <out-dir> <file.java> [more.java ...]`

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args
        .next()
        .expect("usage: classdump <out-dir> <file.java> [more.java ...]");
    let sources: Vec<caturra_compiler::SourceFile> = args
        .map(|path| {
            let text = std::fs::read_to_string(&path).expect("read source");
            let name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Main.java")
                .to_owned();
            caturra_compiler::SourceFile { path: name, text }
        })
        .collect();
    let compilation = caturra_compiler::compile(&sources);
    for diagnostic in &compilation.diagnostics {
        eprintln!("{diagnostic:?}");
    }
    for class in &compilation.classes {
        let path = std::path::Path::new(&out).join(format!("{}.class", class.binary_name));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create output directory");
        }
        std::fs::write(
            &path,
            caturra_classfile::write_class_file(&class.class_file),
        )
        .expect("write class file");
    }
    println!("{} classes", compilation.classes.len());
}
