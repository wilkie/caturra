//! Debug-metadata attributes (JVMS §4.7.10, §4.7.12, §4.7.13):
//! `SourceFile`, `LineNumberTable`, and `LocalVariableTable`.
//!
//! These are the traditional carriers of source-level debug info — the
//! compiler emits them and the VM's debugger reads them to map bytecode
//! back to source lines and slot indices back to variable names.

pub const SOURCE_FILE_ATTRIBUTE: &str = "SourceFile";
pub const LINE_NUMBER_TABLE_ATTRIBUTE: &str = "LineNumberTable";
pub const LOCAL_VARIABLE_TABLE_ATTRIBUTE: &str = "LocalVariableTable";
/// Generic signatures for locals whose erased descriptor loses type
/// arguments (`ArrayList<Integer>`); same entry layout as
/// `LocalVariableTable` with a signature index in place of the
/// descriptor (JVMS §4.7.14).
pub const LOCAL_VARIABLE_TYPE_TABLE_ATTRIBUTE: &str = "LocalVariableTypeTable";

/// One `LocalVariableTable` entry: a named variable live in `[start_pc,
/// start_pc + length)` at local slot `index`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalVariableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub index: u16,
}

/// Encode a `SourceFile` attribute body.
#[must_use]
pub fn encode_source_file(utf8_index: u16) -> Vec<u8> {
    utf8_index.to_be_bytes().to_vec()
}

/// Decode a `SourceFile` attribute body to its Utf8 index.
#[must_use]
pub fn decode_source_file(info: &[u8]) -> Option<u16> {
    let bytes: [u8; 2] = info.try_into().ok()?;
    Some(u16::from_be_bytes(bytes))
}

/// Encode a `LineNumberTable` body from `(start_pc, line)` pairs.
#[must_use]
pub fn encode_line_number_table(entries: &[(u16, u16)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + entries.len() * 4);
    out.extend_from_slice(
        &u16::try_from(entries.len())
            .unwrap_or(u16::MAX)
            .to_be_bytes(),
    );
    for (start_pc, line) in entries {
        out.extend_from_slice(&start_pc.to_be_bytes());
        out.extend_from_slice(&line.to_be_bytes());
    }
    out
}

/// Decode a `LineNumberTable` body to `(start_pc, line)` pairs.
#[must_use]
pub fn decode_line_number_table(info: &[u8]) -> Option<Vec<(u16, u16)>> {
    let count = usize::from(u16::from_be_bytes(info.get(0..2)?.try_into().ok()?));
    let mut entries = Vec::with_capacity(count);
    let mut at = 2;
    for _ in 0..count {
        let start_pc = u16::from_be_bytes(info.get(at..at + 2)?.try_into().ok()?);
        let line = u16::from_be_bytes(info.get(at + 2..at + 4)?.try_into().ok()?);
        entries.push((start_pc, line));
        at += 4;
    }
    Some(entries)
}

/// Encode a `LocalVariableTable` body.
#[must_use]
pub fn encode_local_variable_table(entries: &[LocalVariableEntry]) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + entries.len() * 10);
    out.extend_from_slice(
        &u16::try_from(entries.len())
            .unwrap_or(u16::MAX)
            .to_be_bytes(),
    );
    for entry in entries {
        out.extend_from_slice(&entry.start_pc.to_be_bytes());
        out.extend_from_slice(&entry.length.to_be_bytes());
        out.extend_from_slice(&entry.name_index.to_be_bytes());
        out.extend_from_slice(&entry.descriptor_index.to_be_bytes());
        out.extend_from_slice(&entry.index.to_be_bytes());
    }
    out
}

/// Decode a `LocalVariableTable` body.
#[must_use]
pub fn decode_local_variable_table(info: &[u8]) -> Option<Vec<LocalVariableEntry>> {
    let count = usize::from(u16::from_be_bytes(info.get(0..2)?.try_into().ok()?));
    let mut entries = Vec::with_capacity(count);
    let mut at = 2;
    for _ in 0..count {
        let field = |offset: usize| -> Option<u16> {
            Some(u16::from_be_bytes(
                info.get(at + offset..at + offset + 2)?.try_into().ok()?,
            ))
        };
        entries.push(LocalVariableEntry {
            start_pc: field(0)?,
            length: field(2)?,
            name_index: field(4)?,
            descriptor_index: field(6)?,
            index: field(8)?,
        });
        at += 10;
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_number_table_round_trips() {
        let entries = vec![(0, 3), (7, 4), (19, 6)];
        let encoded = encode_line_number_table(&entries);
        assert_eq!(decode_line_number_table(&encoded), Some(entries));
    }

    #[test]
    fn local_variable_table_round_trips() {
        let entries = vec![
            LocalVariableEntry {
                start_pc: 0,
                length: 40,
                name_index: 7,
                descriptor_index: 8,
                index: 0,
            },
            LocalVariableEntry {
                start_pc: 12,
                length: 28,
                name_index: 9,
                descriptor_index: 10,
                index: 2,
            },
        ];
        let encoded = encode_local_variable_table(&entries);
        assert_eq!(decode_local_variable_table(&encoded), Some(entries));
    }

    #[test]
    fn source_file_round_trips() {
        assert_eq!(decode_source_file(&encode_source_file(42)), Some(42));
    }
}
