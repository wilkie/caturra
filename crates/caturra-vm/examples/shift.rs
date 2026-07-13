//! Bench: the `CSA U5L10-L4d` "shift" filter — the corpus level the sweep found
//! blowing the instruction budget. `makeShift(100)` is 400 rows x 100 shifts x
//! 400 pixels = 16M `getColor`/`setColor` round trips, each one allocating a
//! `Color`.
//!
//! `cargo run --release --example shift`

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

const LEVEL: &str = "artifacts/CSA U5L10-L4d_2022/solution";

fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");

    let Ok(entries) = std::fs::read_dir(root.join(LEVEL)) else {
        eprintln!(
            "no sources at {}\n\
             The corpus is not in the repository; fetch it to run this bench.",
            root.join(LEVEL).display()
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
    let compiled = caturra_compiler::compile(&sources);
    assert!(compiled.success(), "{:?}", compiled.diagnostics);

    // The asset, in the packing the host preloads: LE u32 width, u32 height,
    // then one RGB triple per pixel. Decoding a JPEG is not this crate's job,
    // so a synthetic image of the same 400x400 shape stands in — the filter
    // walks every pixel regardless of what is in them.
    let (width, height) = (400u32, 400u32);
    let mut image = Vec::with_capacity(8 + (width * height * 3) as usize);
    image.extend_from_slice(&width.to_le_bytes());
    image.extend_from_slice(&height.to_le_bytes());
    for i in 0..(width * height) {
        let shade = u8::try_from(i % 256).unwrap_or(0);
        image.extend_from_slice(&[shade, shade.wrapping_add(85), shade.wrapping_add(170)]);
    }
    let mut vfs = caturra_vm::VirtualFileSystem::new();
    vfs.write_file("__caturra_image_electricguitars.jpg", image)
        .expect("write image");
    let mut console = caturra_vm::BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = caturra_vm::Vm::new(caturra_vm::VmOptions::default(), &mut vfs, &mut console);
    for class in compiled.classes {
        vm.load_class(class.class_file).expect("load");
    }

    let before = ALLOCS.load(Ordering::Relaxed);
    let started = std::time::Instant::now();
    let outcome = vm.run_main("MyTheater", &[]);
    let elapsed = started.elapsed();
    let allocations = ALLOCS.load(Ordering::Relaxed) - before;
    println!(
        "run {elapsed:?}  allocations {allocations}  status {}",
        match &outcome {
            Ok(status) => format!("{status:?}"),
            Err(error) => format!("{error:?}"),
        }
    );
}
