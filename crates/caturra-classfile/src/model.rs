//! In-memory class file structures (JVMS §4.1).

use crate::constant_pool::{ConstantPool, CpIndex};
use crate::{JAVA_11_MAJOR, JAVA_11_MINOR};

/// Class access and property flags (JVMS Table 4.1-B).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClassAccessFlags(pub u16);

impl ClassAccessFlags {
    pub const PUBLIC: u16 = 0x0001;
    pub const FINAL: u16 = 0x0010;
    pub const SUPER: u16 = 0x0020;
    pub const INTERFACE: u16 = 0x0200;
    pub const ABSTRACT: u16 = 0x0400;
    pub const SYNTHETIC: u16 = 0x1000;
    pub const ANNOTATION: u16 = 0x2000;
    pub const ENUM: u16 = 0x4000;

    #[must_use]
    pub fn contains(self, flag: u16) -> bool {
        self.0 & flag != 0
    }
}

/// Field access flags (JVMS Table 4.5-A).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FieldAccessFlags(pub u16);

impl FieldAccessFlags {
    pub const PUBLIC: u16 = 0x0001;
    pub const PRIVATE: u16 = 0x0002;
    pub const PROTECTED: u16 = 0x0004;
    pub const STATIC: u16 = 0x0008;
    pub const FINAL: u16 = 0x0010;
    pub const VOLATILE: u16 = 0x0040;
    pub const TRANSIENT: u16 = 0x0080;
    pub const SYNTHETIC: u16 = 0x1000;
    pub const ENUM: u16 = 0x4000;

    #[must_use]
    pub fn contains(self, flag: u16) -> bool {
        self.0 & flag != 0
    }
}

/// Method access flags (JVMS Table 4.6-A).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MethodAccessFlags(pub u16);

impl MethodAccessFlags {
    pub const PUBLIC: u16 = 0x0001;
    pub const PRIVATE: u16 = 0x0002;
    pub const PROTECTED: u16 = 0x0004;
    pub const STATIC: u16 = 0x0008;
    pub const FINAL: u16 = 0x0010;
    pub const SYNCHRONIZED: u16 = 0x0020;
    pub const VARARGS: u16 = 0x0080;
    pub const NATIVE: u16 = 0x0100;
    pub const ABSTRACT: u16 = 0x0400;
    pub const SYNTHETIC: u16 = 0x1000;

    #[must_use]
    pub fn contains(self, flag: u16) -> bool {
        self.0 & flag != 0
    }
}

/// A raw attribute: name plus uninterpreted payload bytes.
///
/// Well-known attributes (`Code`, `LineNumberTable`, ...) are decoded
/// on demand by the consumer; keeping the raw bytes preserves unknown
/// attributes across a read/write round trip.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeInfo {
    pub name_index: CpIndex,
    pub info: Vec<u8>,
}

/// A decoded `Code` attribute (JVMS §4.7.3).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntry>,
    pub attributes: Vec<AttributeInfo>,
}

/// One entry in a `Code` attribute's exception table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: CpIndex,
}

/// A field declaration (JVMS §4.5).
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
    pub name_index: CpIndex,
    pub descriptor_index: CpIndex,
    pub attributes: Vec<AttributeInfo>,
}

/// A method declaration (JVMS §4.6).
#[derive(Debug, Clone, PartialEq)]
pub struct MethodInfo {
    pub access_flags: MethodAccessFlags,
    pub name_index: CpIndex,
    pub descriptor_index: CpIndex,
    pub attributes: Vec<AttributeInfo>,
}

/// A complete class file (JVMS §4.1).
#[derive(Debug, Clone, PartialEq)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: CpIndex,
    pub super_class: CpIndex,
    pub interfaces: Vec<CpIndex>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Vec<AttributeInfo>,
}

impl ClassFile {
    /// A new, empty Java 11 class file.
    #[must_use]
    pub fn new_java11() -> Self {
        Self {
            minor_version: JAVA_11_MINOR,
            major_version: JAVA_11_MAJOR,
            constant_pool: ConstantPool::new(),
            access_flags: ClassAccessFlags(ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER),
            this_class: 0,
            super_class: 0,
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
        }
    }

    /// The binary name of this class (e.g. `com/example/Main`), if the
    /// constant pool entries are well-formed.
    #[must_use]
    pub fn class_name(&self) -> Option<&str> {
        self.constant_pool.get_class_name(self.this_class)
    }
}
