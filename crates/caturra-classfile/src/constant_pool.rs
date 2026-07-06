//! The constant pool (JVMS §4.4).

/// A 1-based index into the constant pool, as used by bytecode and
/// class file structures.
pub type CpIndex = u16;

/// A single constant pool entry.
///
/// `Long` and `Double` occupy two slots in the pool per the spec; the
/// [`ConstantPool`] container handles that bookkeeping so callers can
/// treat entries uniformly.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Utf8(String),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name_index: CpIndex,
    },
    String {
        string_index: CpIndex,
    },
    FieldRef {
        class_index: CpIndex,
        name_and_type_index: CpIndex,
    },
    MethodRef {
        class_index: CpIndex,
        name_and_type_index: CpIndex,
    },
    InterfaceMethodRef {
        class_index: CpIndex,
        name_and_type_index: CpIndex,
    },
    NameAndType {
        name_index: CpIndex,
        descriptor_index: CpIndex,
    },
    /// Placeholder for the second slot of a `Long`/`Double` entry.
    Unusable,
}

impl Constant {
    /// Whether this constant occupies two pool slots (JVMS §4.4.5).
    #[must_use]
    pub fn is_wide(&self) -> bool {
        matches!(self, Constant::Long(_) | Constant::Double(_))
    }

    /// The tag byte used in the binary format (JVMS Table 4.4-A).
    #[must_use]
    pub fn tag(&self) -> u8 {
        match self {
            Constant::Utf8(_) => 1,
            Constant::Integer(_) => 3,
            Constant::Float(_) => 4,
            Constant::Long(_) => 5,
            Constant::Double(_) => 6,
            Constant::Class { .. } => 7,
            Constant::String { .. } => 8,
            Constant::FieldRef { .. } => 9,
            Constant::MethodRef { .. } => 10,
            Constant::InterfaceMethodRef { .. } => 11,
            Constant::NameAndType { .. } => 12,
            Constant::Unusable => panic!("Unusable constant has no tag"),
        }
    }
}

/// The constant pool of a class file.
///
/// Entries are 1-indexed; index 0 is invalid per the spec. Wide entries
/// (`Long`/`Double`) are followed by an [`Constant::Unusable`] filler slot.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConstantPool {
    entries: Vec<Constant>,
}

impl ConstantPool {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The `constant_pool_count` value for the binary format: number of
    /// slots plus one.
    #[must_use]
    pub fn count(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).expect("constant pool overflow")
    }

    /// Append a constant, returning its 1-based index.
    pub fn push(&mut self, constant: Constant) -> CpIndex {
        let index = u16::try_from(self.entries.len() + 1).expect("constant pool overflow");
        let wide = constant.is_wide();
        self.entries.push(constant);
        if wide {
            self.entries.push(Constant::Unusable);
        }
        index
    }

    /// Look up a constant by 1-based index.
    #[must_use]
    pub fn get(&self, index: CpIndex) -> Option<&Constant> {
        if index == 0 {
            return None;
        }
        self.entries.get(usize::from(index) - 1)
    }

    /// Look up a UTF-8 constant, returning its string value.
    #[must_use]
    pub fn get_utf8(&self, index: CpIndex) -> Option<&str> {
        match self.get(index)? {
            Constant::Utf8(s) => Some(s),
            _ => None,
        }
    }

    /// Resolve a `Class` constant to its binary class name
    /// (e.g. `java/lang/String`).
    #[must_use]
    pub fn get_class_name(&self, index: CpIndex) -> Option<&str> {
        match self.get(index)? {
            Constant::Class { name_index } => self.get_utf8(*name_index),
            _ => None,
        }
    }

    /// Resolve a `NameAndType` constant to `(name, descriptor)`.
    #[must_use]
    pub fn get_name_and_type(&self, index: CpIndex) -> Option<(&str, &str)> {
        match self.get(index)? {
            Constant::NameAndType {
                name_index,
                descriptor_index,
            } => Some((
                self.get_utf8(*name_index)?,
                self.get_utf8(*descriptor_index)?,
            )),
            _ => None,
        }
    }

    /// Resolve a `FieldRef`, `MethodRef`, or `InterfaceMethodRef` to
    /// `(class name, member name, descriptor)`.
    #[must_use]
    pub fn get_member_ref(&self, index: CpIndex) -> Option<(&str, &str, &str)> {
        match self.get(index)? {
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
                let class = self.get_class_name(*class_index)?;
                let (name, descriptor) = self.get_name_and_type(*name_and_type_index)?;
                Some((class, name, descriptor))
            }
            _ => None,
        }
    }

    /// Iterate over all slots, including `Unusable` fillers.
    pub fn iter_slots(&self) -> impl Iterator<Item = &Constant> {
        self.entries.iter()
    }

    /// Find an existing identical entry or insert a new one, returning
    /// its index. `Double`/`Float` compare by bit pattern, not IEEE
    /// equality — `-0.0` and `0.0` are distinct constants, and `NaN`
    /// deduplicates against itself.
    pub fn intern(&mut self, constant: Constant) -> CpIndex {
        fn identical(a: &Constant, b: &Constant) -> bool {
            match (a, b) {
                (Constant::Double(x), Constant::Double(y)) => x.to_bits() == y.to_bits(),
                (Constant::Float(x), Constant::Float(y)) => x.to_bits() == y.to_bits(),
                _ => a == b,
            }
        }
        for (i, entry) in self.entries.iter().enumerate() {
            if identical(entry, &constant) {
                return u16::try_from(i + 1).expect("constant pool overflow");
            }
        }
        self.push(constant)
    }

    /// Find an existing UTF-8 entry or insert a new one, returning its index.
    pub fn intern_utf8(&mut self, value: &str) -> CpIndex {
        self.intern(Constant::Utf8(value.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_get_are_one_indexed() {
        let mut pool = ConstantPool::new();
        let a = pool.push(Constant::Integer(42));
        assert_eq!(a, 1);
        assert_eq!(pool.get(0), None);
        assert_eq!(pool.get(1), Some(&Constant::Integer(42)));
    }

    #[test]
    fn wide_constants_take_two_slots() {
        let mut pool = ConstantPool::new();
        let long_index = pool.push(Constant::Long(1));
        let next = pool.push(Constant::Integer(2));
        assert_eq!(long_index, 1);
        assert_eq!(next, 3);
        assert_eq!(pool.get(2), Some(&Constant::Unusable));
        assert_eq!(pool.count(), 4);
    }

    #[test]
    fn intern_utf8_deduplicates() {
        let mut pool = ConstantPool::new();
        let a = pool.intern_utf8("main");
        let b = pool.intern_utf8("main");
        assert_eq!(a, b);
    }

    #[test]
    fn class_name_resolves_through_utf8() {
        let mut pool = ConstantPool::new();
        let name = pool.intern_utf8("java/lang/Object");
        let class = pool.push(Constant::Class { name_index: name });
        assert_eq!(pool.get_class_name(class), Some("java/lang/Object"));
    }
}
