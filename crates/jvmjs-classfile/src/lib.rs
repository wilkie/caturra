//! JVM class file model, reader, and writer.
//!
//! This crate defines an in-memory representation of `.class` files as
//! specified by the Java Virtual Machine Specification (JVMS §4), plus
//! binary (de)serialization. The compiler crate produces these structures
//! and the VM crate consumes them, so the class file is the contract
//! between the two halves of the system.
//!
//! We target Java 11 class files: major version 55, minor version 0.

mod constant_pool;
pub mod debug;
mod model;
pub mod opcodes;
mod reader;
mod writer;

pub use constant_pool::{Constant, ConstantPool, CpIndex};
pub use model::{
    AttributeInfo, ClassAccessFlags, ClassFile, CodeAttribute, ExceptionTableEntry,
    FieldAccessFlags, FieldInfo, MethodAccessFlags, MethodInfo,
};
pub use reader::{ReadError, read_class_file, read_code_attribute};
pub use writer::{write_class_file, write_code_attribute};

/// Name of the `Code` attribute (JVMS §4.7.3).
pub const CODE_ATTRIBUTE: &str = "Code";

/// The `0xCAFEBABE` magic number that begins every class file.
pub const MAGIC: u32 = 0xCAFE_BABE;

/// Class file major version for Java 11.
pub const JAVA_11_MAJOR: u16 = 55;

/// Class file minor version for Java 11.
pub const JAVA_11_MINOR: u16 = 0;
