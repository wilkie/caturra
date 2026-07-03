//! Runtime values and the heap (see `specs/RUNTIME.md`).
//!
//! References are indices into a per-run heap `Vec`; there is no
//! garbage collector in v1 — the heap is dropped when the run ends.

/// Index of an object on the heap.
pub type HeapRef = u32;

/// A value on the operand stack or in a local variable slot.
///
/// `long`/`double` occupy two slots in real frames (JVMS §2.6); this VM
/// stores them as one `JValue` and accounts for slot widths only where
/// the spec's numbering is observable (locals indices, `max_stack`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    /// An object reference; `None` is Java's `null`.
    Ref(Option<HeapRef>),
}

impl JValue {
    pub const NULL: JValue = JValue::Ref(None);
}

/// Which standard stream an intrinsic `PrintStream` writes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdStream {
    Out,
    Err,
}

/// An object on the heap.
#[derive(Debug, Clone, PartialEq)]
pub enum HeapObject {
    /// A `java.lang.String`: UTF-16 code units for exact Java semantics.
    JavaString(Vec<u16>),
    /// A `java.lang.StringBuilder` (used by compiled string concatenation).
    StringBuilder(Vec<u16>),
    /// The intrinsic object behind `System.out` / `System.err`.
    PrintStream(StdStream),
    /// An `int[]`, `boolean[]`, or `char[]` — all stored as i32 per
    /// JVMS array-load semantics (stores mask to the element width).
    IntArray(Vec<i32>),
    /// A `double[]`.
    DoubleArray(Vec<f64>),
    /// A reference array (e.g. `String[] args`, or the rows of a 2D
    /// array).
    RefArray(Vec<JValue>),
    /// An instance of a user-defined class.
    Instance {
        class_name: String,
        fields: std::collections::HashMap<String, JValue>,
    },
}

/// The per-run object heap.
#[derive(Debug, Default)]
pub struct Heap {
    objects: Vec<HeapObject>,
}

impl Heap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate an object, returning its reference.
    pub fn alloc(&mut self, object: HeapObject) -> HeapRef {
        let index = u32::try_from(self.objects.len()).expect("heap exhausted");
        self.objects.push(object);
        index
    }

    /// Allocate a Java string from Rust UTF-8 text.
    pub fn alloc_string(&mut self, text: &str) -> HeapRef {
        self.alloc(HeapObject::JavaString(text.encode_utf16().collect()))
    }

    #[must_use]
    pub fn get(&self, reference: HeapRef) -> Option<&HeapObject> {
        self.objects.get(reference as usize)
    }

    #[must_use]
    pub fn get_mut(&mut self, reference: HeapRef) -> Option<&mut HeapObject> {
        self.objects.get_mut(reference as usize)
    }

    /// Read a Java string back as Rust text (lossy on unpaired
    /// surrogates, which is what console output wants).
    #[must_use]
    pub fn string_text(&self, reference: HeapRef) -> Option<String> {
        match self.get(reference)? {
            HeapObject::JavaString(units) => Some(String::from_utf16_lossy(units)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_round_trip_through_utf16() {
        let mut heap = Heap::new();
        let reference = heap.alloc_string("héllo 🌍");
        assert_eq!(heap.string_text(reference).as_deref(), Some("héllo 🌍"));
    }

    #[test]
    fn string_length_is_utf16_units() {
        let mut heap = Heap::new();
        // '🌍' is two UTF-16 code units — Java's length() would say 2.
        let reference = heap.alloc_string("🌍");
        let Some(HeapObject::JavaString(units)) = heap.get(reference) else {
            panic!("expected a string");
        };
        assert_eq!(units.len(), 2);
    }
}
