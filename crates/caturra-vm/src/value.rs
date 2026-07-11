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

/// Which `int`-width primitive an [`HeapObject::IntArray`] holds. They share
/// one representation, but a `boolean[]` prints `true` where an `int[]`
/// prints `1`, and hashes 1231/1237 where an `int[]` hashes its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntKind {
    Int,
    Boolean,
    Char,
}

/// How a factory-built `Comparator` orders two values — evaluated natively by
/// the interpreter (which can run the key extractor and `compareTo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparatorSpec {
    /// `Comparator.naturalOrder()` — the elements' own `compareTo`.
    Natural,
    /// `Comparator.comparing(keyExtractor)` / `comparingInt(...)` — compare the
    /// keys the extractor (a `Function`) returns, by their natural ordering.
    ByKey(HeapRef),
    /// `comparator.reversed()` / `Comparator.reverseOrder()` — the inverse.
    Reversed(HeapRef),
    /// `first.thenComparing(second)` — `first`, then `second` on a tie.
    Then(HeapRef, HeapRef),
}

/// What a `Stream.collect(Collectors.…())` gathers its elements into.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectorKind {
    /// `Collectors.toList()` — an `ArrayList` of the elements, in order.
    ToList,
    /// `Collectors.toSet()` — a `HashSet`, in the JDK's iteration order.
    ToSet,
    /// `Collectors.joining(...)` — the elements' text, with a delimiter,
    /// prefix, and suffix.
    Joining {
        delimiter: String,
        prefix: String,
        suffix: String,
    },
}

/// Which flavour of `Optional` a [`HeapObject::Optional`] is — only its
/// `toString` prefix differs (`Optional[x]` / `OptionalInt[x]` / `OptionalDouble[x]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalKind {
    Ref,
    Int,
    Double,
}

impl OptionalKind {
    /// The `toString` prefix Java uses for this flavour.
    #[must_use]
    pub fn prefix(self) -> &'static str {
        match self {
            OptionalKind::Ref => "Optional",
            OptionalKind::Int => "OptionalInt",
            OptionalKind::Double => "OptionalDouble",
        }
    }
}

/// Which of a map's three views a [`HeapObject::MapView`] presents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapViewKind {
    Keys,
    Values,
    Entries,
}

/// Which standard stream an intrinsic `PrintStream` writes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdStream {
    Out,
    Err,
}

/// An object on the heap.
///
/// Deliberately not `PartialEq`: Java equality is heap-aware (two `Integer`
/// objects holding 20 are equal; two references are not), so comparing
/// heap objects structurally would quietly give the wrong answer.
#[derive(Debug, Clone)]
pub enum HeapObject {
    /// A `java.lang.String`: UTF-16 code units for exact Java semantics.
    JavaString(Vec<u16>),
    /// A `java.lang.StringBuilder` (used by compiled string concatenation).
    StringBuilder(Vec<u16>),
    /// The intrinsic object behind `System.out` / `System.err`.
    PrintStream(StdStream),
    /// An `int[]`, `boolean[]`, or `char[]` — all stored as i32 per
    /// JVMS array-load semantics (stores mask to the element width). The
    /// kind is carried because the three render and hash differently once
    /// their static type is gone (`Arrays.deepToString(boolean[][])`).
    IntArray(IntKind, Vec<i32>),
    /// A `double[]`.
    DoubleArray(Vec<f64>),
    /// A `long[]`.
    LongArray(Vec<i64>),
    /// A `float[]`.
    FloatArray(Vec<f32>),
    /// A boxed primitive wrapper (`Integer`, `Double`, ...). Holds the
    /// wrapper's internal class name and the primitive value.
    Boxed { class_name: String, value: JValue },
    /// A `short[]`.
    ShortArray(Vec<i16>),
    /// A `byte[]` (distinct from boolean arrays, which reuse
    /// [`HeapObject::IntArray`]: `bastore` truncates for bytes).
    ByteArray(Vec<i8>),
    /// A reference array (e.g. `String[] args`, or the rows of a 2D
    /// array), with its JVM class descriptor: `[Ljava/lang/String;`,
    /// `[[I`. Java's `toString()` and `getClass().getName()` name the
    /// element type, and once the static type is gone the heap is the
    /// only thing that still knows it. Primitive arrays derive theirs
    /// from the variant instead.
    RefArray(String, Vec<JValue>),
    /// An instance of a user-defined class.
    Instance {
        class_name: String,
        fields: std::collections::HashMap<String, JValue>,
    },
    /// A `java.util.Scanner` over standard input: buffered text pulled
    /// from the console line by line.
    Scanner {
        buffer: String,
        /// Cursor into `buffer` (UTF-8 byte index).
        pos: usize,
        eof: bool,
        /// Reads standard input rather than a file. `close()` on such a
        /// scanner closes the underlying stream, as the JDK's does — every
        /// later `Scanner(System.in)` then sees end of input.
        stdin: bool,
        /// `close()` was called. Every method but `close` then throws
        /// `IllegalStateException: Scanner closed`.
        closed: bool,
    },
    /// A `java.util.ArrayList` (element types erased; values are
    /// stored directly — boxing is a no-op in this VM).
    ArrayList(Vec<JValue>),
    /// A `java.util.LinkedList` — the same ordered-sequence storage as an
    /// `ArrayList` (this VM does not model node links or their cost), kept a
    /// distinct kind only so `getClass()` stays honest and so the compiler can
    /// offer its `Queue`/`Deque` methods. Reachable through a `List`, `Queue`,
    /// or `Deque` variable, each of which exposes a different method subset.
    LinkedList(Vec<JValue>),
    /// An unmodifiable *view* of a list (`Collections.unmodifiableList`,
    /// `Collections.emptyList`). Java's is a view too: a later `add` to the
    /// backing list shows through, and every mutator throws.
    UnmodifiableList(HeapRef),
    /// A `java.util.HashMap` (key/value types erased), carrying the JDK's
    /// iteration order. See [`crate::map`].
    HashMap(crate::map::JavaHashMap),
    /// A `java.util.TreeMap` (key/value types erased): its entries kept sorted
    /// by key, so iteration, `firstKey`/`lastKey` and the key navigation read
    /// straight off the vector. `comparator` orders the keys (or `None` for
    /// natural `Comparable` ordering); both may run user code, so all ordering
    /// lives in the interpreter.
    TreeMap {
        entries: Vec<(JValue, JValue)>,
        comparator: Option<HeapRef>,
    },
    /// A `java.util.HashSet` (element type erased). The JDK backs a `HashSet`
    /// with a `HashMap` whose keys are the elements, so a set's iteration
    /// order is exactly that map's key order — modelled by reusing
    /// [`crate::map::JavaHashMap`] with each element stored as a key mapped to
    /// a placeholder value.
    HashSet(crate::map::JavaHashMap),
    /// A `java.util.stream.Stream` (element type erased). Modelled eagerly: the
    /// vector holds the pipeline's current elements, each intermediate operation
    /// (`filter`/`map`/`sorted`/…) produces a new one, and a terminal operation
    /// (`collect`/`forEach`/`count`/…) consumes it. Finite streams give
    /// identical results to a lazy JDK stream.
    Stream(Vec<JValue>),
    /// The recipe a `Stream.collect` gathers into, from a `Collectors` factory.
    Collector(CollectorKind),
    /// A `Comparator` built by the `Comparator` static factories / combinators
    /// (`comparing`/`naturalOrder`/`reversed`/`thenComparing`) rather than a
    /// user class. The interpreter evaluates it natively (see [`ComparatorSpec`]).
    Comparator(ComparatorSpec),
    /// A `java.util.Optional` / `OptionalInt` / `OptionalDouble`: a value that
    /// is present or absent. `kind` is only for `toString` (`Optional[x]` vs
    /// `OptionalInt[x]`); the accessors (`get`/`getAsInt`/`getAsDouble`) are the
    /// same at runtime, the compiler having picked the right one.
    Optional {
        value: Option<JValue>,
        kind: OptionalKind,
    },
    /// A `java.util.PriorityQueue` (element type erased): a binary min-heap in
    /// an array, so `peek`/`poll` return the least element while iteration and
    /// `toString` show the heap-array order — replicated exactly (Java's
    /// `siftUp`/`siftDown`) so both match a real JVM. `comparator` orders the
    /// elements, or `None` for natural (`Comparable`) ordering.
    PriorityQueue {
        heap: Vec<JValue>,
        comparator: Option<HeapRef>,
    },
    /// A `java.util.TreeSet` (element type erased): its elements kept in sorted
    /// order, so iteration, `first`/`last` and the navigation methods read
    /// straight off the vector. `comparator` is the `Comparator` instance the
    /// set was built with, or `None` for natural (`Comparable`) ordering; both
    /// may run user code, so all ordering lives in the interpreter.
    TreeSet {
        values: Vec<JValue>,
        comparator: Option<HeapRef>,
    },
    /// A live view onto a map: `keySet()`, `values()` or `entrySet()`.
    /// Java's are views too, so a later `put` shows through.
    MapView { map: HeapRef, kind: MapViewKind },
    /// One `Map.Entry` from an `entrySet()`, resolved against its map so
    /// that `getValue`/`setValue` see the current value.
    MapEntry { map: HeapRef, key: JValue },
    /// The marker object behind `System.in`.
    InputStream,
    /// A `java.io.File`: a path into the virtual filesystem.
    File(String),
    /// A `java.io.PrintWriter` into the virtual filesystem
    /// (write-through: output is durable without `close()`).
    Writer { path: String },
    /// A throwable: a library exception class (dotted name) with its
    /// optional message. Bound by `catch` handlers, created by
    /// `new SomeException(...)`.
    Exception {
        class_name: String,
        message: Option<String>,
    },
    /// A `java.lang.Class` handle from `obj.getClass()` — the (flat,
    /// simple) class name is enough for the structural reflection the
    /// curriculum uses.
    Class { name: String },
    /// A `java.lang.reflect.Field` from `Class.getDeclaredFields()`.
    /// Self-contained so `toString()` needs no class lookup.
    Field {
        declaring: String,
        name: String,
        /// JVM field descriptor, e.g. `I` or `Ljava/lang/String;`.
        descriptor: String,
        /// Raw `FieldAccessFlags` bits.
        access: u16,
        /// The generic signature from the field's `Signature` attribute, e.g.
        /// `Ljava/util/ArrayList<LFriend;>;` (for `Field.getGenericType()`).
        signature: Option<String>,
    },
    /// A `java.lang.reflect.Type` / `ParameterizedType` from
    /// `Field.getGenericType()`: the raw type and its type arguments (empty
    /// for a non-parameterized type).
    ReflectType { raw: String, args: Vec<String> },
    /// A `java.lang.reflect.Constructor` from `getDeclaredConstructors()`.
    Constructor {
        declaring: String,
        /// The `<init>` method descriptor, e.g. `(Ljava/lang/String;I)V`.
        descriptor: String,
        /// Raw `MethodAccessFlags` bits.
        access: u16,
    },
    /// A `java.lang.reflect.Method` from `Class.getMethod(name, Class[])`.
    Method {
        declaring: String,
        name: String,
        /// The method descriptor, e.g. `(DI)D`.
        descriptor: String,
        /// Raw `MethodAccessFlags` bits.
        access: u16,
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

    /// The backing element vector of an `ArrayList` or a `LinkedList` — both
    /// store their elements the same way, so list operations read either.
    #[must_use]
    pub fn list_values(&self, reference: HeapRef) -> Option<&Vec<JValue>> {
        match self.get(reference) {
            Some(HeapObject::ArrayList(values) | HeapObject::LinkedList(values)) => Some(values),
            _ => None,
        }
    }

    /// The mutable backing vector of an `ArrayList` or a `LinkedList`.
    #[must_use]
    pub fn list_values_mut(&mut self, reference: HeapRef) -> Option<&mut Vec<JValue>> {
        match self.get_mut(reference) {
            Some(HeapObject::ArrayList(values) | HeapObject::LinkedList(values)) => Some(values),
            _ => None,
        }
    }

    /// The first string object with these exact code units, if any
    /// (linear scan — `String.intern()` at educational heap sizes).
    #[must_use]
    pub fn find_string(&self, units: &[u16]) -> Option<HeapRef> {
        self.objects
            .iter()
            .position(
                |object| matches!(object, HeapObject::JavaString(existing) if existing == units),
            )
            .and_then(|index| u32::try_from(index).ok())
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
