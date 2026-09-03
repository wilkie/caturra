//! Binary class file serialization (JVMS §4).

use crate::MAGIC;
use crate::constant_pool::Constant;
use crate::model::{AttributeInfo, ClassFile, CodeAttribute};

/// Serialize a class file to its binary representation.
#[must_use]
pub fn write_class_file(class: &ClassFile) -> Vec<u8> {
    let mut out = Vec::with_capacity(1024);

    out.extend_from_slice(&MAGIC.to_be_bytes());
    out.extend_from_slice(&class.minor_version.to_be_bytes());
    out.extend_from_slice(&class.major_version.to_be_bytes());

    out.extend_from_slice(&class.constant_pool.count().to_be_bytes());
    for constant in class.constant_pool.iter_slots() {
        write_constant(&mut out, constant);
    }

    out.extend_from_slice(&class.access_flags.0.to_be_bytes());
    out.extend_from_slice(&class.this_class.to_be_bytes());
    out.extend_from_slice(&class.super_class.to_be_bytes());

    out.extend_from_slice(&u16_len(class.interfaces.len()).to_be_bytes());
    for interface in &class.interfaces {
        out.extend_from_slice(&interface.to_be_bytes());
    }

    out.extend_from_slice(&u16_len(class.fields.len()).to_be_bytes());
    for field in &class.fields {
        out.extend_from_slice(&field.access_flags.0.to_be_bytes());
        out.extend_from_slice(&field.name_index.to_be_bytes());
        out.extend_from_slice(&field.descriptor_index.to_be_bytes());
        write_attributes(&mut out, &field.attributes);
    }

    out.extend_from_slice(&u16_len(class.methods.len()).to_be_bytes());
    for method in &class.methods {
        out.extend_from_slice(&method.access_flags.0.to_be_bytes());
        out.extend_from_slice(&method.name_index.to_be_bytes());
        out.extend_from_slice(&method.descriptor_index.to_be_bytes());
        write_attributes(&mut out, &method.attributes);
    }

    write_attributes(&mut out, &class.attributes);
    out
}

/// Encode a `Code` attribute's payload bytes (JVMS §4.7.3), suitable as
/// the `info` of an [`AttributeInfo`] whose name is `Code`.
#[must_use]
pub fn write_code_attribute(code: &CodeAttribute) -> Vec<u8> {
    let mut out = Vec::with_capacity(code.code.len() + 32);
    out.extend_from_slice(&code.max_stack.to_be_bytes());
    out.extend_from_slice(&code.max_locals.to_be_bytes());
    out.extend_from_slice(
        &u32::try_from(code.code.len())
            .expect("bytecode exceeds u32 length")
            .to_be_bytes(),
    );
    out.extend_from_slice(&code.code);
    out.extend_from_slice(&u16_len(code.exception_table.len()).to_be_bytes());
    for entry in &code.exception_table {
        out.extend_from_slice(&entry.start_pc.to_be_bytes());
        out.extend_from_slice(&entry.end_pc.to_be_bytes());
        out.extend_from_slice(&entry.handler_pc.to_be_bytes());
        out.extend_from_slice(&entry.catch_type.to_be_bytes());
    }
    write_attributes(&mut out, &code.attributes);
    out
}

fn u16_len(len: usize) -> u16 {
    u16::try_from(len).expect("class file section exceeds u16 length")
}

fn write_constant(out: &mut Vec<u8>, constant: &Constant) {
    // Unusable is the phantom second slot of a wide constant; it has no
    // bytes of its own in the binary format.
    if matches!(constant, Constant::Unusable) {
        return;
    }
    out.push(constant.tag());

    match constant {
        Constant::Utf8(s) => write_modified_utf8(out, &s.encode_utf16().collect::<Vec<_>>()),
        Constant::Utf8Units(units) => write_modified_utf8(out, units),
        Constant::Integer(v) => out.extend_from_slice(&v.to_be_bytes()),
        Constant::Float(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Constant::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        Constant::Double(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Constant::Class { name_index } => out.extend_from_slice(&name_index.to_be_bytes()),
        Constant::String { string_index } => out.extend_from_slice(&string_index.to_be_bytes()),
        Constant::FieldRef {
            class_index,
            name_and_type_index,
        }
        | Constant::MethodRef {
            class_index,
            name_and_type_index,
        }
        | Constant::InterfaceMethodRef {
            class_index,
            name_and_type_index,
        } => {
            out.extend_from_slice(&class_index.to_be_bytes());
            out.extend_from_slice(&name_and_type_index.to_be_bytes());
        }
        Constant::NameAndType {
            name_index,
            descriptor_index,
        } => {
            out.extend_from_slice(&name_index.to_be_bytes());
            out.extend_from_slice(&descriptor_index.to_be_bytes());
        }
        Constant::Unusable => unreachable!(),
    }
}

fn write_attributes(out: &mut Vec<u8>, attributes: &[AttributeInfo]) {
    out.extend_from_slice(&u16_len(attributes.len()).to_be_bytes());
    for attribute in attributes {
        out.extend_from_slice(&attribute.name_index.to_be_bytes());
        out.extend_from_slice(
            &u32::try_from(attribute.info.len())
                .expect("attribute exceeds u32 length")
                .to_be_bytes(),
        );
        out.extend_from_slice(&attribute.info);
    }
}

/// "Modified UTF-8" (JVMS §4.4.7): like UTF-8, except that U+0000 is written
/// as two bytes and every code point above the BMP is written as its
/// SURROGATE PAIR, three bytes each. Encoding the units one at a time gives
/// both for free — and gives an UNPAIRED surrogate a spelling too, which is
/// the whole reason a string constant is carried as units.
fn write_modified_utf8(out: &mut Vec<u8>, units: &[u16]) {
    let start = out.len();
    out.extend_from_slice(&[0, 0]);
    for unit in units {
        let value = u32::from(*unit);
        match value {
            0x0001..=0x007F => out.push(u8::try_from(value).unwrap_or(b'?')),
            0 | 0x0080..=0x07FF => {
                out.push(u8::try_from(0xC0 | (value >> 6)).unwrap_or(b'?'));
                out.push(u8::try_from(0x80 | (value & 0x3F)).unwrap_or(b'?'));
            }
            _ => {
                out.push(u8::try_from(0xE0 | (value >> 12)).unwrap_or(b'?'));
                out.push(u8::try_from(0x80 | ((value >> 6) & 0x3F)).unwrap_or(b'?'));
                out.push(u8::try_from(0x80 | (value & 0x3F)).unwrap_or(b'?'));
            }
        }
    }
    let length = u16_len(out.len() - start - 2);
    out[start..start + 2].copy_from_slice(&length.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read_class_file;

    #[test]
    fn round_trips_a_code_attribute() {
        let code = CodeAttribute {
            max_stack: 2,
            max_locals: 1,
            code: vec![crate::opcodes::RETURN],
            exception_table: vec![crate::ExceptionTableEntry {
                start_pc: 0,
                end_pc: 1,
                handler_pc: 1,
                catch_type: 0,
            }],
            attributes: Vec::new(),
        };
        let bytes = write_code_attribute(&code);
        let parsed = crate::reader::read_code_attribute(&bytes).expect("should parse");
        assert_eq!(parsed, code);
    }

    /// Build a small but structurally complete class and verify that
    /// write → read reproduces it exactly.
    #[test]
    fn round_trips_a_minimal_class() {
        let mut class = ClassFile::new_java11();
        let object_name = class.constant_pool.intern_utf8("java/lang/Object");
        let object = class.constant_pool.push(Constant::Class {
            name_index: object_name,
        });
        let main_name = class.constant_pool.intern_utf8("Main");
        let main = class.constant_pool.push(Constant::Class {
            name_index: main_name,
        });
        class
            .constant_pool
            .push(Constant::Long(0x1122_3344_5566_7788));
        class.constant_pool.push(Constant::Double(3.5));
        class.this_class = main;
        class.super_class = object;

        let bytes = write_class_file(&class);
        let parsed = read_class_file(&bytes).expect("round trip should parse");
        assert_eq!(parsed, class);
        assert_eq!(parsed.class_name(), Some("Main"));
    }
}
