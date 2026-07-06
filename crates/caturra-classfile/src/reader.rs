//! Binary class file parsing (JVMS §4).

use crate::MAGIC;
use crate::constant_pool::{Constant, ConstantPool};
use crate::model::{
    AttributeInfo, ClassAccessFlags, ClassFile, CodeAttribute, ExceptionTableEntry,
    FieldAccessFlags, FieldInfo, MethodAccessFlags, MethodInfo,
};

/// Errors produced while parsing a class file.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReadError {
    #[error("not a class file: bad magic number {0:#010x}")]
    BadMagic(u32),
    #[error("unsupported class file version {major}.{minor} (newest supported is 55.0 / Java 11)")]
    UnsupportedVersion { major: u16, minor: u16 },
    #[error("unknown constant pool tag {0}")]
    UnknownConstantTag(u8),
    #[error("truncated class file at offset {0}")]
    Truncated(usize),
    #[error("invalid UTF-8 in constant pool entry")]
    InvalidUtf8,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ReadError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or(ReadError::Truncated(self.pos))?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(ReadError::Truncated(self.pos))?;
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, ReadError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ReadError> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, ReadError> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }
}

/// Parse a class file from its binary representation.
pub fn read_class_file(bytes: &[u8]) -> Result<ClassFile, ReadError> {
    let mut c = Cursor::new(bytes);

    let magic = c.u32()?;
    if magic != MAGIC {
        return Err(ReadError::BadMagic(magic));
    }
    let minor_version = c.u16()?;
    let major_version = c.u16()?;
    if major_version > crate::JAVA_11_MAJOR {
        return Err(ReadError::UnsupportedVersion {
            major: major_version,
            minor: minor_version,
        });
    }

    let constant_pool = read_constant_pool(&mut c)?;

    let access_flags = ClassAccessFlags(c.u16()?);
    let this_class = c.u16()?;
    let super_class = c.u16()?;

    let interface_count = c.u16()?;
    let mut interfaces = Vec::with_capacity(usize::from(interface_count));
    for _ in 0..interface_count {
        interfaces.push(c.u16()?);
    }

    let field_count = c.u16()?;
    let mut fields = Vec::with_capacity(usize::from(field_count));
    for _ in 0..field_count {
        fields.push(FieldInfo {
            access_flags: FieldAccessFlags(c.u16()?),
            name_index: c.u16()?,
            descriptor_index: c.u16()?,
            attributes: read_attributes(&mut c)?,
        });
    }

    let method_count = c.u16()?;
    let mut methods = Vec::with_capacity(usize::from(method_count));
    for _ in 0..method_count {
        methods.push(MethodInfo {
            access_flags: MethodAccessFlags(c.u16()?),
            name_index: c.u16()?,
            descriptor_index: c.u16()?,
            attributes: read_attributes(&mut c)?,
        });
    }

    let attributes = read_attributes(&mut c)?;

    Ok(ClassFile {
        minor_version,
        major_version,
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    })
}

fn read_constant_pool(c: &mut Cursor<'_>) -> Result<ConstantPool, ReadError> {
    let count = c.u16()?;
    let mut pool = ConstantPool::new();
    let mut index = 1u16;
    while index < count {
        let tag = c.u8()?;
        let constant = match tag {
            1 => {
                let len = usize::from(c.u16()?);
                let bytes = c.take(len)?;
                // TODO: decode "modified UTF-8" (JVMS §4.4.7) instead of
                // assuming standard UTF-8; they differ for NUL and
                // supplementary characters.
                let s = std::str::from_utf8(bytes).map_err(|_| ReadError::InvalidUtf8)?;
                Constant::Utf8(s.to_owned())
            }
            3 => Constant::Integer(c.u32()?.cast_signed()),
            4 => Constant::Float(f32::from_bits(c.u32()?)),
            5 => Constant::Long((u64::from(c.u32()?) << 32 | u64::from(c.u32()?)).cast_signed()),
            6 => Constant::Double(f64::from_bits(
                u64::from(c.u32()?) << 32 | u64::from(c.u32()?),
            )),
            7 => Constant::Class {
                name_index: c.u16()?,
            },
            8 => Constant::String {
                string_index: c.u16()?,
            },
            9 => Constant::FieldRef {
                class_index: c.u16()?,
                name_and_type_index: c.u16()?,
            },
            10 => Constant::MethodRef {
                class_index: c.u16()?,
                name_and_type_index: c.u16()?,
            },
            11 => Constant::InterfaceMethodRef {
                class_index: c.u16()?,
                name_and_type_index: c.u16()?,
            },
            12 => Constant::NameAndType {
                name_index: c.u16()?,
                descriptor_index: c.u16()?,
            },
            other => return Err(ReadError::UnknownConstantTag(other)),
        };
        index += if constant.is_wide() { 2 } else { 1 };
        pool.push(constant);
    }
    Ok(pool)
}

/// Decode a `Code` attribute's payload bytes (JVMS §4.7.3), i.e. the
/// `info` of an [`AttributeInfo`] whose name is `Code`.
pub fn read_code_attribute(bytes: &[u8]) -> Result<CodeAttribute, ReadError> {
    let mut c = Cursor::new(bytes);
    let max_stack = c.u16()?;
    let max_locals = c.u16()?;
    let code_length = c.u32()? as usize;
    let code = c.take(code_length)?.to_vec();

    let exception_count = c.u16()?;
    let mut exception_table = Vec::with_capacity(usize::from(exception_count));
    for _ in 0..exception_count {
        exception_table.push(ExceptionTableEntry {
            start_pc: c.u16()?,
            end_pc: c.u16()?,
            handler_pc: c.u16()?,
            catch_type: c.u16()?,
        });
    }

    let attributes = read_attributes(&mut c)?;
    Ok(CodeAttribute {
        max_stack,
        max_locals,
        code,
        exception_table,
        attributes,
    })
}

fn read_attributes(c: &mut Cursor<'_>) -> Result<Vec<AttributeInfo>, ReadError> {
    let count = c.u16()?;
    let mut attributes = Vec::with_capacity(usize::from(count));
    for _ in 0..count {
        let name_index = c.u16()?;
        let len = c.u32()? as usize;
        let info = c.take(len)?.to_vec();
        attributes.push(AttributeInfo { name_index, info });
    }
    Ok(attributes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_magic() {
        let err = read_class_file(&[0x00, 0x01, 0x02, 0x03, 0, 0, 0, 55]).unwrap_err();
        assert_eq!(err, ReadError::BadMagic(0x0001_0203));
    }

    #[test]
    fn rejects_future_versions() {
        // Magic + minor 0 + major 61 (Java 17).
        let bytes = [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x3D];
        let err = read_class_file(&bytes).unwrap_err();
        assert_eq!(
            err,
            ReadError::UnsupportedVersion {
                major: 61,
                minor: 0
            }
        );
    }

    #[test]
    fn rejects_truncated_input() {
        let err = read_class_file(&[0xCA, 0xFE]).unwrap_err();
        assert_eq!(err, ReadError::Truncated(0));
    }
}
