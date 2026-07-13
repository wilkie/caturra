//! Bench harness for the `CSA U5L8-L5d_pilot_oo2025` solution: the pixel-walk
//! program the interpreter's object and field paths are tuned against.
//!
//! Reports wall time and the exact allocation count (a counting global
//! allocator wraps the system one), so a change that trades allocations for
//! time — or claims to remove them and doesn't — shows up as a number.
//!
//! `cargo run --release --example u5l8`

// A counting global allocator is the only way to see the VM's allocation count
// from outside it, and `GlobalAlloc` is an unsafe trait.
#![allow(
    unsafe_code,
    clippy::undocumented_unsafe_blocks,
    clippy::multiple_unsafe_ops_per_block
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// The level's sources live in the (git-ignored) Code.org corpus, like
/// `nbhcheck`'s do. Pass another directory of `.java` files to measure that
/// instead; the entry class is the one declaring `main`.
const LEVEL: &str = "artifacts/CSA U5L8-L5d_pilot_oo2025/solution";

fn main() {
    let root = match std::env::args().nth(1) {
        Some(path) => std::path::PathBuf::from(path),
        None => std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("workspace root")
            .join(LEVEL),
    };
    let Ok(entries) = std::fs::read_dir(&root) else {
        eprintln!(
            "no sources at {}\n\
             The corpus is not in the repository; fetch it, or pass a directory \
             of .java files to measure.",
            root.display()
        );
        std::process::exit(1);
    };
    let mut paths: Vec<std::path::PathBuf> = entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()? == "java").then_some(path)
        })
        .collect();
    paths.sort();
    let sources: Vec<caturra_compiler::SourceFile> = paths
        .iter()
        .map(|path| caturra_compiler::SourceFile {
            path: path.file_name().unwrap().to_string_lossy().into_owned(),
            text: std::fs::read_to_string(path).expect("read source"),
        })
        .collect();

    let entry = sources
        .iter()
        .find(|source| source.text.contains("static void main"))
        .map(|source| source.path.trim_end_matches(".java").to_owned())
        .expect("a class declaring main");

    let started = std::time::Instant::now();
    let compiled = caturra_compiler::compile(&sources);
    let compile_time = started.elapsed();
    assert!(compiled.success(), "{:?}", compiled.diagnostics);

    // The asset the playground would have preloaded: LE u32 width, u32 height,
    // then one RGB triple per pixel. Without it the program finds no image and
    // walks nothing, so there is no pixel walk left to measure. The content is
    // irrelevant to the shape — only the 400x400 size is.
    let (width, height) = (400u32, 400u32);
    let mut image = Vec::with_capacity(8 + (width * height * 3) as usize);
    image.extend_from_slice(&width.to_le_bytes());
    image.extend_from_slice(&height.to_le_bytes());
    for i in 0..(width * height) {
        let shade = u8::try_from(i % 256).unwrap_or(0);
        image.extend_from_slice(&[shade, shade.wrapping_add(85), shade.wrapping_add(170)]);
    }
    let mut vfs = caturra_vm::VirtualFileSystem::new();
    vfs.write_file("__caturra_image_cellphone.jpg", image)
        .expect("write image");
    let mut console = caturra_vm::BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = caturra_vm::Vm::new(
        caturra_vm::VmOptions {
            max_instructions: 5_000_000_000,
            ..caturra_vm::VmOptions::default()
        },
        &mut vfs,
        &mut console,
    );
    for class in compiled.classes {
        vm.load_class(class.class_file).expect("load");
    }

    let before = ALLOCS.load(Ordering::Relaxed);
    let started = std::time::Instant::now();
    let outcome = vm.run_main(&entry, &[]);
    let run_time = started.elapsed();
    let allocations = ALLOCS.load(Ordering::Relaxed) - before;

    println!(
        "compile {compile_time:?}  run {run_time:?}  allocations {allocations}  status {}",
        match &outcome {
            Ok(status) => format!("{status:?}"),
            Err(error) => format!("{error:?}"),
        }
    );
}
