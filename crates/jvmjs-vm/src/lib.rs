//! The `java`-equivalent half of jvmjs: a JVM bytecode interpreter with a
//! virtual filesystem and pluggable console IO, designed to run inside a
//! browser via WebAssembly (no OS, no threads, no JIT).
//!
//! Scope: the AP Computer Science A (CSA) subset of Java 11 — classes,
//! interfaces, inheritance, arrays, `String`, `ArrayList`, wrapper types,
//! `Math`, console IO via `System.out` / `Scanner`, and `java.io.File`
//! backed by the in-memory [`vfs::VirtualFileSystem`].

mod interpreter;
pub mod intrinsics;
pub mod io;
pub mod value;
pub mod vfs;
pub mod vm;

pub use io::{BufferedConsole, ConsoleIo};
pub use value::{Heap, HeapObject, HeapRef, JValue};
pub use vfs::{VfsError, VirtualFileSystem};
pub use vm::{ExitStatus, Vm, VmError, VmOptions};
