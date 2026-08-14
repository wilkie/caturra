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
    /// `Comparator.comparing(keyExtractor, keyComparator)` — extract the key,
    /// then order the KEYS by a comparator of their own rather than naturally.
    ByKeyWith(HeapRef, HeapRef),
    /// `String.CASE_INSENSITIVE_ORDER` — compares two strings by their
    /// case-folded code units.
    CaseInsensitive,
    /// `Comparator.nullsFirst(inner)` / `nullsLast(inner)` — `null` sorts
    /// before (or after) everything, two nulls are equal, and anything else is
    /// left to `inner`. `first` selects which end the nulls go to.
    ///
    /// The inner comparator is optional because `nullsFirst(null)` is legal
    /// and means "all non-null elements compare equal" — not natural ordering,
    /// which is a distinction the JDK draws and it would be easy to lose.
    Nulls { first: bool, inner: Option<HeapRef> },
}

/// A pending intermediate stream operation. The terminal pulls each source
/// element through these in order; `sorted` is not here because it is a
/// barrier that materializes the pipeline instead of deferring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamOp {
    /// `filter(pred)` — drop an element the predicate rejects.
    Filter(HeapRef),
    /// `map`/`mapToInt`/`mapToObj`/… — replace the element with the function's
    /// result (all one thing here, since primitives are stored unboxed).
    Map(HeapRef),
    /// `peek(consumer)` — run the consumer for its side effect, pass through.
    Peek(HeapRef),
    /// `limit(n)` — pass the first `n` elements, then stop the source.
    Limit(usize),
    /// `skip(n)` — drop the first `n` elements.
    Skip(usize),
    /// `distinct()` — pass an element only the first time it is seen.
    Distinct,
    /// `boxed()` — the primitive pipeline becomes an OBJECT one, so each
    /// element becomes its wrapper. Not a retyping: a collection stores boxed
    /// references at rest, so a raw `int` reaching one is a `VerifyError` at the
    /// next reference use.
    Box,
    /// `asLongStream()` / `asDoubleStream()` — every element WIDENS to that
    /// primitive. Not a pure retyping: a downstream lambda declared over the
    /// new width unboxes what it is handed, and a stray `Integer` there is a
    /// `ClassCastException`.
    WidenToLong,
    WidenToDouble,
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
    /// `Collectors.counting()` — a `Long` of how many elements arrived.
    Counting,
    /// `Collectors.groupingBy(classifier)` — a `HashMap` from each element's
    /// key to the `List` of elements that produced it, in encounter order.
    /// `downstream` gathers each group when one was given (`groupingBy(f, g)`).
    GroupingBy {
        classifier: HeapRef,
        downstream: Option<HeapRef>,
    },
    /// `Collectors.partitioningBy(predicate)` — a two-entry map, `false` then
    /// `true`, each holding the `List` of elements on that side.
    PartitioningBy(HeapRef),
    /// `Collectors.toMap(keyFn, valueFn)` — a `HashMap` built from the two
    /// functions. A duplicate key with no merge function is an
    /// `IllegalStateException`, as the JDK's is.
    ToMap {
        key: HeapRef,
        value: HeapRef,
        merge: Option<HeapRef>,
    },
    /// `Collectors.summingInt(f)` / `summingLong` / `summingDouble` — the sum
    /// of the mapped values, and `averagingInt`/… their mean (always a
    /// `Double`, and 0.0 over no elements).
    Summing { mapper: HeapRef, kind: SumKind },
}

/// Which numeric summary a [`CollectorKind::Summing`] produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SumKind {
    Int,
    Long,
    Double,
    Averaging,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapViewKind {
    Keys,
    Values,
    Entries,
}

/// What a [`HeapObject::Iterator`] may write back through to the collection it
/// walks. A JDK iterator inherits its mutators from that collection — the JDK's
/// `AbstractList.Itr.remove` calls the list's own `remove`, so a view that
/// refuses one refuses the other. Modelling the cursor as always-mutable let
/// `Collections.unmodifiableList(data).iterator().remove()` destroy the very
/// list the view exists to protect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorWrites {
    /// Everything: `remove`, and a `ListIterator`'s `set` and `add`. What a
    /// plain mutable collection's iterator allows.
    All,
    /// `set` only — `AbstractList`'s cursor over a FIXED-SIZE list, which is
    /// what `Arrays.asList(a).listIterator()` returns. An element write goes
    /// straight through to the array; `add`/`remove` would change the length,
    /// and the cursor checks its own state (`IllegalStateException`) before
    /// the list gets to refuse.
    FixedSize,
    /// Nothing, and the refusal reads `UnsupportedOperationException: remove`
    /// — `Arrays.asList(a).iterator()` is JDK 9's `ArrayItr`, which simply
    /// does not implement `remove`, so `Iterator`'s default throws that
    /// whatever the cursor's state.
    ArrayCursor,
    /// Nothing, but only after the cursor's own state check — a GENERIC cursor
    /// (`AbstractList`'s, or the `EmptyIterator` behind `emptyList()`) over a
    /// collection that happens to refuse. `Collections.nCopies` and the
    /// `empty*` factories hand these out.
    NoneChecked,
    /// Nothing — an unmodifiable or immutable view's own cursor, which refuses
    /// outright rather than looking at its state or asking the collection.
    None,
}

/// Which standard stream an intrinsic `PrintStream` writes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdStream {
    Out,
    Err,
}

/// Where each of a class's instance fields lives in an object.
///
/// Fields are keyed `Declaring.name`: a subclass may hide a superclass field of
/// the same name (JLS 8.3), and the two are distinct storage. Inherited fields
/// come first, so a class's layout is a prefix of every subclass's — which is
/// what lets a `getfield` site cache the slot it resolved once and index
/// straight into any later receiver, whatever its exact runtime class.
#[derive(Debug, Default)]
pub struct ClassLayout {
    /// Slot → key. Enumerating an object's fields (`toString`, the debugger)
    /// reads names from here rather than from the object itself.
    names: Vec<std::rc::Rc<str>>,
    /// Key → slot.
    index: std::collections::HashMap<std::rc::Rc<str>, usize>,
}

impl ClassLayout {
    /// Append a field, or return the slot it already has (a class re-declaring
    /// an inherited name shadows it, and that is a *different* key).
    pub fn push(&mut self, key: std::rc::Rc<str>) -> usize {
        if let Some(&slot) = self.index.get(&key) {
            return slot;
        }
        let slot = self.names.len();
        self.index.insert(std::rc::Rc::clone(&key), slot);
        self.names.push(key);
        slot
    }

    /// The slot holding `key`, if this class has such a field.
    #[must_use]
    pub fn slot(&self, key: &str) -> Option<usize> {
        self.index.get(key).copied()
    }

    /// The field keys, in slot order.
    #[must_use]
    pub fn names(&self) -> &[std::rc::Rc<str>] {
        &self.names
    }
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
    Boxed {
        class_name: std::rc::Rc<str>,
        value: JValue,
    },
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
    /// An instance of a user-defined class: its fields laid out flat, one per
    /// slot, in the order its [`ClassLayout`] fixes. The layout is shared by
    /// every instance of the class, so allocating an object copies a vector of
    /// defaults — no hashing, no per-field allocation — and reading a field is
    /// an index rather than a map probe.
    Instance {
        class_name: std::rc::Rc<str>,
        layout: std::rc::Rc<ClassLayout>,
        fields: Vec<JValue>,
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
    /// A `java.io.BufferedReader`/`FileReader`/`InputStreamReader` — one reader
    /// kind. A file reader slurps the whole file into `buffer` up front; a
    /// `stdin` reader pulls each line from the console lazily. `readLine`
    /// returns the next line (no terminator) or null; `read` the next char or
    /// -1.
    Reader {
        buffer: String,
        /// Cursor into `buffer` (UTF-8 byte index) for a file reader.
        pos: usize,
        /// Reads standard input rather than a file.
        stdin: bool,
        /// `close()` was called; further reads return end-of-stream.
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
    /// A `java.util.ArrayDeque` — the same ordered-sequence storage as a
    /// `LinkedList` used as a `Deque` (head-based `push`/`pop`), kept a distinct
    /// kind for two honest reasons: it forbids null elements (every insertion
    /// throws `NullPointerException`, unlike the null-tolerant `LinkedList`), and
    /// `getClass()` reports `java.util.ArrayDeque`. It is not a `List`.
    ArrayDeque(Vec<JValue>),
    /// A `java.util.Stack` — a `Vector`-backed LIFO. The same ordered-sequence
    /// storage as an `ArrayList` (it *is* a `List`, so every list method reads
    /// it), kept a distinct kind so `push`/`pop`/`peek` act on the top (the
    /// *end*, unlike a `Deque`'s head), an empty `pop`/`peek` throws
    /// `EmptyStackException`, and `getClass()` stays honest.
    Stack(Vec<JValue>),
    /// An unmodifiable *view* of a list (`Collections.unmodifiableList`,
    /// `Collections.emptyList`). Java's is a view too: a later `add` to the
    /// backing list shows through, and every mutator throws.
    UnmodifiableList(HeapRef),
    /// `Arrays.asList(array)` — a FIXED-SIZE list backed by the array itself.
    /// `set` writes through to the array and vice versa; `add`/`remove` throw
    /// `UnsupportedOperationException`. Only a REFERENCE array can back one:
    /// `Arrays.asList(int[])` is a one-element `List<int[]>` in Java, not a
    /// list of the ints.
    ArrayBackedList(HeapRef),
    /// An unmodifiable *view* of a set (`Collections.unmodifiableSet`,
    /// `emptySet`, `singleton`). Reads delegate to the backing `HashSet`/
    /// `TreeSet`; every mutator throws `UnsupportedOperationException`.
    UnmodifiableSet(HeapRef),
    /// An unmodifiable *view* of a map (`Collections.unmodifiableMap`,
    /// `emptyMap`, `singletonMap`). Reads delegate to the backing `HashMap`/
    /// `TreeMap`; every mutator throws `UnsupportedOperationException`.
    UnmodifiableMap(HeapRef),
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
    /// A `java.util.stream.Stream` (element type erased), modelled LAZILY: the
    /// source elements plus the pending intermediate operations. Nothing runs
    /// until a terminal operation PULLS elements one at a time through the op
    /// chain, so side effects interleave and a short-circuit terminal
    /// (`findFirst`/`anyMatch`/`limit`) stops the source early — exactly as a
    /// JDK does, and unlike the previous eager `Vec` that ran every stage in
    /// full. `sorted` is a barrier: it materializes and re-sources.
    Stream {
        source: Vec<JValue>,
        ops: Vec<StreamOp>,
    },
    /// The recipe a `Stream.collect` gathers into, from a `Collectors` factory.
    Collector(CollectorKind),
    /// A `Comparator` built by the `Comparator` static factories / combinators
    /// (`comparing`/`naturalOrder`/`reversed`/`thenComparing`) rather than a
    /// user class. The interpreter evaluates it natively (see [`ComparatorSpec`]).
    Comparator(ComparatorSpec),
    /// A `java.util.IntSummaryStatistics` — what one pass of an `IntStream`
    /// gathered. Held as the five numbers so the accessors and `toString`
    /// answer without re-walking anything.
    SummaryStats {
        count: i64,
        sum: i64,
        min: i32,
        max: i32,
    },
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
    /// A live `java.util.Iterator` over a list or set: the collection it walks,
    /// the position it will return next, and the position it last returned (for
    /// `remove()`, `None` before the first `next()` or right after a `remove()`).
    ///
    /// `expected_len` makes the iterator FAIL-FAST: the collection's length when
    /// this iterator last agreed with it. `next()` compares and throws
    /// `ConcurrentModificationException` if someone else changed the collection
    /// meanwhile; the iterator's own `remove()` re-syncs it.
    ///
    /// Length stands in for the JDK's `modCount` because every structural
    /// modification of the collections caturra models changes the size —
    /// `put` of an existing key and `list.set(i, v)` are not structural and do
    /// not bump `modCount` either. The gap is a modification that nets out to
    /// the same size between two `next()` calls (an add AND a remove), which
    /// the JDK catches and this does not; see the note in `iterator_method`.
    Iterator {
        source: HeapRef,
        index: usize,
        last: Option<usize>,
        expected_len: usize,
        /// What this cursor may write back through — see [`IteratorWrites`].
        writes: IteratorWrites,
        /// Built by `listIterator()` rather than `iterator()`. Only `getClass`
        /// can tell the difference, and it can: a JDK's are separate classes
        /// (`ArrayList$Itr` vs `ArrayList$ListItr`).
        list: bool,
    },
    /// A live view onto a map: `keySet()`, `values()` or `entrySet()`.
    /// Java's are views too, so a later `put` shows through.
    ///
    /// `read_only` marks a view taken from a `Collections.unmodifiableMap`:
    /// every mutator throws, its iterator cannot remove, and the entries it
    /// hands out refuse `setValue`.
    MapView {
        map: HeapRef,
        kind: MapViewKind,
        read_only: bool,
    },
    /// One `Map.Entry` from an `entrySet()`, resolved against its map so
    /// that `getValue`/`setValue` see the current value. `read_only` is set
    /// on an entry from an unmodifiable map's view, whose `setValue` throws.
    MapEntry {
        map: HeapRef,
        key: JValue,
        read_only: bool,
    },
    /// The marker object behind `System.in`.
    InputStream,
    /// A `java.io.File`: a path into the virtual filesystem.
    File(String),
    /// A `java.nio.file.Path`: a filesystem path (from `Path.of`/`Paths.get`),
    /// read and written through `Files`.
    Path(String),
    /// A `java.io.PrintWriter` into the virtual filesystem
    /// (write-through: output is durable without `close()`).
    Writer { path: String },
    /// A throwable: a library exception class (dotted name) with its
    /// optional message. Bound by `catch` handlers, created by
    /// `new SomeException(...)`.
    Exception {
        class_name: String,
        message: Option<String>,
        /// The chained cause (`new X(msg, cause)` / `initCause`), for
        /// `getCause()`. `None` when the exception has no cause.
        cause: Option<HeapRef>,
        /// Exceptions suppressed in favour of this one — what a
        /// try-with-resources attaches when `close()` throws while the body
        /// is already unwinding (JLS §14.20.3.1).
        suppressed: Vec<HeapRef>,
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
    /// One `java.lang.StackTraceElement` from `Throwable.getStackTrace()`.
    /// The trace is recorded as rendered lines (`Cls.m(File.java:12)`), which
    /// is what `printStackTrace` prints; these are those lines taken apart so
    /// a program can read the pieces.
    StackFrame {
        declaring: String,
        method: String,
        file: Option<String>,
        line: i32,
    },
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

impl HeapObject {
    /// Read an instance field by its `Declaring.name` key. `None` if this is
    /// not an instance, or its class has no such field.
    ///
    /// The interpreter's hot paths index a memoized slot instead; this is for
    /// the cold ones (reflection, the throwable message) that only have a name.
    #[must_use]
    pub fn field(&self, key: &str) -> Option<JValue> {
        match self {
            HeapObject::Instance { layout, fields, .. } => fields.get(layout.slot(key)?).copied(),
            _ => None,
        }
    }

    /// A handle on an instance field, to assign through.
    pub fn field_mut(&mut self, key: &str) -> Option<&mut JValue> {
        match self {
            HeapObject::Instance { layout, fields, .. } => {
                let slot = layout.slot(key)?;
                fields.get_mut(slot)
            }
            _ => None,
        }
    }
}

/// The per-run object heap.
/// `new StringBuilder()` starts here, and a `String` seed adds its length.
pub const DEFAULT_BUILDER_CAPACITY: usize = 16;

#[derive(Debug, Default)]
pub struct Heap {
    objects: Vec<HeapObject>,
    /// The autoboxing cache (JLS §5.1.7): `Wrapper.valueOf` returns a SHARED
    /// reference for a value in the cached range, so `Integer a = 100, b = 100;
    /// a == b` is true while `200 == 200` (out of range) is false. Keyed by
    /// (wrapper tag, value) — see [`Heap::box_wrapper`].
    wrapper_cache: std::collections::HashMap<(u8, i64), HeapRef>,
    /// A builder's `capacity()`, which is HISTORY-dependent and so cannot be
    /// derived from its contents: appending ten characters one at a time
    /// leaves the initial 16, while appending forty at once jumps straight to
    /// 40. A side table, so the `StringBuilder` variant stays a plain
    /// `Vec<u16>` and keeps sharing its or-patterns with `JavaString`.
    builder_capacity: std::collections::HashMap<HeapRef, usize>,
}

/// The autoboxing-cache key for a wrapper class and value, or `None` when the
/// value is outside the cached range (so each boxing is a distinct object).
/// The `u8` tag keeps different wrappers of the same numeric value apart.
fn wrapper_cache_key(class: &str, value: JValue) -> Option<(u8, i64)> {
    let (tag, v) = match (class, value) {
        ("java/lang/Integer", JValue::Int(v)) => (0u8, i64::from(v)),
        ("java/lang/Short", JValue::Int(v)) => (1, i64::from(v)),
        ("java/lang/Byte", JValue::Int(v)) => (2, i64::from(v)),
        ("java/lang/Long", JValue::Long(v)) => (3, v),
        // Character caches 0..=127; Boolean caches both values.
        ("java/lang/Character", JValue::Int(v)) => {
            return (0..=127).contains(&v).then_some((4, i64::from(v)));
        }
        ("java/lang/Boolean", JValue::Int(v)) => return Some((5, i64::from(v))),
        _ => return None,
    };
    (-128..=127).contains(&v).then_some((tag, v))
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

    /// Box a primitive into its wrapper via the autoboxing cache (JLS §5.1.7):
    /// a value in the cached range shares one reference so `==` on two such
    /// boxings is true, while out-of-range values each get a fresh reference.
    /// Integer/Short/Byte/Long cache -128..=127, Character 0..=127, Boolean
    /// both values; Double/Float are never cached.
    /// A builder's capacity: 16 by default, as `new StringBuilder()` has.
    #[must_use]
    pub fn builder_capacity(&self, reference: HeapRef) -> usize {
        self.builder_capacity
            .get(&reference)
            .copied()
            .unwrap_or(DEFAULT_BUILDER_CAPACITY)
    }

    pub fn set_builder_capacity(&mut self, reference: HeapRef, capacity: usize) {
        self.builder_capacity.insert(reference, capacity);
    }

    /// Grow to hold `needed`, the way `AbstractStringBuilder` does: double and
    /// add two, or jump straight to what is needed when that is still short.
    pub fn grow_builder_capacity(&mut self, reference: HeapRef, needed: usize) {
        let current = self.builder_capacity(reference);
        if needed <= current {
            return;
        }
        let doubled = current.saturating_mul(2).saturating_add(2);
        self.set_builder_capacity(reference, doubled.max(needed));
    }

    pub fn box_wrapper(&mut self, class: &str, value: JValue) -> HeapRef {
        let cache_key = wrapper_cache_key(class, value);
        if let Some(key) = cache_key
            && let Some(&reference) = self.wrapper_cache.get(&key)
        {
            return reference;
        }
        let reference = self.alloc(HeapObject::Boxed {
            class_name: std::rc::Rc::from(class),
            value,
        });
        if let Some(key) = cache_key {
            self.wrapper_cache.insert(key, reference);
        }
        reference
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
            Some(
                HeapObject::ArrayList(values)
                | HeapObject::LinkedList(values)
                | HeapObject::ArrayDeque(values)
                | HeapObject::Stack(values),
            ) => Some(values),
            // A view reads straight out of the array it is backed by, so a
            // write to either side is visible from the other.
            Some(HeapObject::ArrayBackedList(array)) => match self.get(*array) {
                Some(HeapObject::RefArray(_, values)) => Some(values),
                _ => None,
            },
            _ => None,
        }
    }

    /// The mutable backing vector of an `ArrayList` or a `LinkedList`.
    #[must_use]
    pub fn list_values_mut(&mut self, reference: HeapRef) -> Option<&mut Vec<JValue>> {
        // A view writes THROUGH to its array, so `list.set(0, x)` changes
        // `array[0]`. Resolved first because the borrow cannot span the match.
        if let Some(HeapObject::ArrayBackedList(array)) = self.get(reference) {
            let array = *array;
            return match self.get_mut(array) {
                Some(HeapObject::RefArray(_, values)) => Some(values),
                _ => None,
            };
        }
        match self.get_mut(reference) {
            Some(
                HeapObject::ArrayList(values)
                | HeapObject::LinkedList(values)
                | HeapObject::ArrayDeque(values)
                | HeapObject::Stack(values),
            ) => Some(values),
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
