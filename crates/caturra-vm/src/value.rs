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
    /// `Map.Entry.comparingByKey()` / `comparingByValue()` — order two map
    /// entries by the natural ordering of their keys (or values). A key
    /// extractor cannot express these: `ByKey` runs a `Function` from the
    /// heap, and there is no user lambda here to run.
    ///
    /// `inner` is the overload that takes an ordering for the key (or value)
    /// instead of its natural one — `comparingByValue(reverseOrder())`.
    Entry {
        by_value: bool,
        inner: Option<HeapRef>,
    },
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
    /// `takeWhile(pred)` — pass elements until one fails the predicate, then
    /// STOP the source (unlike `filter`, which keeps looking).
    TakeWhile(HeapRef),
    /// `dropWhile(pred)` — drop elements while the predicate holds, then pass
    /// every one that follows, including later elements that would match.
    DropWhile(HeapRef),
    /// `sorted()` / `sorted(cmp)` — a stateful BARRIER: it buffers the whole
    /// upstream and emits nothing until the source is exhausted, then emits in
    /// order. Lazy like every other op, which is what makes the side effects
    /// upstream of it wait for a terminal — and vanish entirely when the
    /// terminal is a `count()` that never traverses.
    Sorted(Option<HeapRef>),
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
///
/// Not `Eq`: `Collectors.reducing` carries an identity VALUE, which may be a
/// double, and a float has no total equality.
#[derive(Debug, Clone, PartialEq)]
pub enum CollectorKind {
    /// `Collectors.toList()` — an `ArrayList` of the elements, in order.
    ToList,
    /// `Collectors.toSet()` — a `HashSet`, in the JDK's iteration order.
    ToSet,
    /// `Collectors.toUnmodifiableList/Set/Map` — the collector inside it, with
    /// its result handed to `List.of`/`Set.of`/`Map.ofEntries`. Composing
    /// rather than duplicating is what keeps the gathering in one place: the
    /// unmodifiable forms differ from the plain ones only in the finish, where
    /// the result becomes immutable AND a null element becomes an NPE.
    Unmodifiable(Box<CollectorKind>),
    /// `Collectors.joining(...)` — the elements' text, with a delimiter,
    /// prefix, and suffix.
    Joining {
        delimiter: String,
        prefix: String,
        suffix: String,
    },
    /// `Collectors.toCollection(supplier)` — the collection the supplier
    /// builds, filled in encounter order. It is how a stream is gathered into
    /// something other than the default `ArrayList`/`HashSet`: a `TreeSet`, a
    /// `LinkedList`, an `ArrayDeque`.
    ToCollection(HeapRef),
    /// `Collectors.counting()` — a `Long` of how many elements arrived.
    Counting,
    /// `Collectors.groupingBy(classifier)` — a `HashMap` from each element's
    /// key to the `List` of elements that produced it, in encounter order.
    /// `downstream` gathers each group when one was given (`groupingBy(f, g)`).
    GroupingBy {
        classifier: HeapRef,
        downstream: Option<HeapRef>,
        /// The MAP to gather into, when the three-argument form named one —
        /// `groupingBy(f, TreeMap::new, downstream)`, which is how a program
        /// asks for its groups in key order.
        factory: Option<HeapRef>,
    },
    /// `Collectors.partitioningBy(predicate)` — a two-entry map, `false` then
    /// `true`, each holding the `List` of elements on that side. `downstream`
    /// gathers each side when one was given, exactly as `groupingBy`'s does.
    PartitioningBy {
        predicate: HeapRef,
        downstream: Option<HeapRef>,
    },
    /// `Collectors.toMap(keyFn, valueFn)` — a `HashMap` built from the two
    /// functions. A duplicate key with no merge function is an
    /// `IllegalStateException`, as the JDK's is.
    ToMap {
        key: HeapRef,
        value: HeapRef,
        merge: Option<HeapRef>,
        /// The MAP to gather into, when the four-argument form named one —
        /// `toMap(k, v, merge, TreeMap::new)`. Without it the result is the
        /// `HashMap` the two- and three-argument forms promise.
        factory: Option<HeapRef>,
    },
    /// `Collectors.summingInt(f)` / `summingLong` / `summingDouble` — the sum
    /// of the mapped values, and `averagingInt`/… their mean (always a
    /// `Double`, and 0.0 over no elements).
    Summing { mapper: HeapRef, kind: SumKind },
    /// `Collectors.mapping(f, downstream)` — each element through `f`, then
    /// gathered by the collector below. Almost always a groupingBy downstream,
    /// which is the shape it exists for.
    Mapping {
        mapper: HeapRef,
        downstream: HeapRef,
    },
    /// `Collectors.filtering(p, downstream)` — the elements that pass, then
    /// the collector below. Not the same as filtering the STREAM when it is a
    /// `groupingBy` downstream: a group whose every member fails still exists,
    /// holding nothing, where a filtered stream would not have made the key.
    Filtering {
        predicate: HeapRef,
        downstream: HeapRef,
    },
    /// `Collectors.flatMapping(f, downstream)` — each element's stream
    /// flattened, then the collector below.
    FlatMapping {
        mapper: HeapRef,
        downstream: HeapRef,
    },
    /// `Collectors.collectingAndThen(downstream, finisher)` — gather, then
    /// hand the result to one more function. The way a group is counted, or
    /// frozen, without a second pass.
    CollectingAndThen {
        downstream: HeapRef,
        finisher: HeapRef,
    },
    /// `Collectors.maxBy(cmp)` / `minBy(cmp)` — an `Optional` of the extreme
    /// element, empty over no elements.
    Extreme { comparator: HeapRef, max: bool },
    /// `Collectors.reducing(...)` in its three forms: with an identity (the
    /// result is a plain value), without one (an `Optional`), and with a
    /// mapper applied before each combine.
    Reducing {
        identity: Option<JValue>,
        mapper: Option<HeapRef>,
        operator: HeapRef,
    },
    /// `Collectors.summarizingInt/Long/Double(f)` — count, sum, min, max and
    /// average of the mapped values, in one object.
    Summarizing { mapper: HeapRef, kind: SummaryKind },
}

/// Which summary object a stream or a [`CollectorKind::Summarizing`] answers.
/// The three differ in what they PRINT (`%f` for a double's sum/min/max) and
/// in the identity values an empty summary keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryKind {
    Int,
    Long,
    Double,
}

impl SummaryKind {
    /// The class this summary is an instance of.
    #[must_use]
    pub fn class(self) -> &'static str {
        match self {
            SummaryKind::Int => "java/util/IntSummaryStatistics",
            SummaryKind::Long => "java/util/LongSummaryStatistics",
            SummaryKind::Double => "java/util/DoubleSummaryStatistics",
        }
    }
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
/// `toString` prefix differs (`Optional[x]` / `OptionalInt[x]` /
/// `OptionalLong[x]` / `OptionalDouble[x]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalKind {
    Ref,
    Int,
    Long,
    Double,
}

impl OptionalKind {
    /// The `toString` prefix Java uses for this flavour.
    #[must_use]
    pub fn prefix(self) -> &'static str {
        match self {
            OptionalKind::Ref => "Optional",
            OptionalKind::Int => "OptionalInt",
            OptionalKind::Long => "OptionalLong",
            OptionalKind::Double => "OptionalDouble",
        }
    }
}

/// One end of a [`HeapObject::SortedView`]'s range: the value the program
/// named, and whether the range includes it. `headSet(x)` excludes, `tailSet(x)`
/// includes, and the four-argument `subSet`/`subMap` say which for each end.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SortedBound {
    pub value: JValue,
    pub inclusive: bool,
}

/// Which face a [`HeapObject::SortedView`] presents: the set itself, the map
/// itself, or a map's KEYS as a set (`navigableKeySet`/`descendingKeySet`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortedFace {
    /// A view of a `TreeSet` — its elements.
    Set,
    /// A view of a `TreeMap` — its entries.
    Map,
    /// A view of a `TreeMap`'s KEYS, presented as a `NavigableSet`.
    Keys,
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
    /// Nothing, and NOT fail-fast: a `Vector`'s `elements()` and a
    /// `Hashtable`'s `keys()`/`elements()` predate `modCount`, so a change
    /// made mid-walk is simply seen rather than thrown at. Its past-the-end
    /// complaint carries the JDK's own text, which names the collection:
    /// "Vector Enumeration", "Hashtable Enumerator".
    Enumerator,
}

/// Which standard stream an intrinsic `PrintStream` writes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdStream {
    Out,
    Err,
}

/// What a `DateTimeFormatter` was built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateFormatKind {
    Pattern(String),
    /// `ofLocalizedDate`/`ofLocalizedTime`/`ofLocalizedDateTime` — a date
    /// style, a time style, or both, by `FormatStyle`'s own ordinal (0 FULL,
    /// 1 LONG, 2 MEDIUM, 3 SHORT). It prints through an ordinary PATTERN, the
    /// one en-US gives that style; what it cannot be is a pattern in the first
    /// place, because a JDK's `toString` says `Localized(FULL,)` and because a
    /// localized TIME reports a missing zone with a chronology beside it.
    Localized(Option<u8>, Option<u8>),
    /// One of `ISO_LOCAL_DATE` (0), `ISO_LOCAL_TIME` (1) and
    /// `ISO_LOCAL_DATE_TIME` (2). `ISO_DATE`/`ISO_TIME`/`ISO_DATE_TIME` are
    /// the same three for a value with no zone in it.
    Iso(u8),
}

/// One `java.time` value. Small and `Copy`, because these are value types:
/// every operation on one answers a NEW one, and nothing mutates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Temporal {
    Date(crate::time::Date),
    Time(crate::time::Time),
    DateTime(crate::time::DateTime),
    Duration(crate::time::Duration),
    Period(crate::time::Period),
    /// A `java.time.temporal.ChronoUnit` constant, by its own ordinal — an
    /// enum, so interned like the other two.
    Unit(u8),
    /// `java.time.DayOfWeek`, 1..=7 with Monday at 1.
    DayOfWeek(u8),
    /// `java.time.Month`, 1..=12.
    Month(u8),
    /// A `java.time.temporal.ChronoField` constant, by its own ordinal.
    Field(u8),
    /// `java.time.temporal.ValueRange` — what a field can hold.
    Range(crate::time::ValueRange),
    /// A `java.time.temporal.TemporalAdjusters` rule.
    Adjuster(crate::time::Adjuster),
    /// `java.time.chrono.IsoEra` — 0 is BCE and 1 is CE.
    Era(u8),
    /// `java.time.format.TextStyle`, by its own ordinal. It is an ENUM like
    /// the ones above it, and was modelled only as a constant the compiler
    /// read where it was written — so `TextStyle.values()`, a `TextStyle`
    /// variable and a switch over one were all refused.
    TextStyle(u8),
    /// `java.time.format.FormatStyle`, likewise.
    FormatStyle(u8),
    /// `java.time.Year` — a year on its own, which is a value a program keeps
    /// when the month and day would be a lie.
    Year(i32),
    /// `java.time.YearMonth` — a month of a year, the unit a statement covers.
    YearMonth(i32, u8),
    /// `java.time.MonthDay` — a day of a year that has no year: a birthday.
    MonthDay(u8, u8),
}

/// `java.time.format.TextStyle`'s constants, in the enum's own order — which
/// is `ordinal()`, and the order `values()` answers in.
pub const TEXT_STYLE_NAMES: [&str; 6] = [
    "FULL",
    "FULL_STANDALONE",
    "SHORT",
    "SHORT_STANDALONE",
    "NARROW",
    "NARROW_STANDALONE",
];

/// ...and `java.time.format.FormatStyle`'s.
pub const FORMAT_STYLE_NAMES: [&str; 4] = ["FULL", "LONG", "MEDIUM", "SHORT"];

impl Temporal {
    /// The class a `getClass()` reports, and the name an exception mentions.
    #[must_use]
    pub fn class_name(self) -> &'static str {
        match self {
            Temporal::Date(_) => "java/time/LocalDate",
            Temporal::Time(_) => "java/time/LocalTime",
            Temporal::Duration(_) => "java/time/Duration",
            Temporal::Period(_) => "java/time/Period",
            Temporal::Unit(_) => "java/time/temporal/ChronoUnit",
            Temporal::DateTime(_) => "java/time/LocalDateTime",
            Temporal::DayOfWeek(_) => "java/time/DayOfWeek",
            Temporal::Month(_) => "java/time/Month",
            Temporal::Field(_) => "java/time/temporal/ChronoField",
            Temporal::Range(_) => "java/time/temporal/ValueRange",
            // Every adjuster is a lambda inside the JDK, so its class is a
            // synthetic one; a program never prints it usefully.
            Temporal::Adjuster(_) => "java/time/temporal/TemporalAdjusters",
            Temporal::Era(_) => "java/time/chrono/IsoEra",
            Temporal::TextStyle(_) => "java/time/format/TextStyle",
            Temporal::FormatStyle(_) => "java/time/format/FormatStyle",
            Temporal::Year(_) => "java/time/Year",
            Temporal::YearMonth(_, _) => "java/time/YearMonth",
            Temporal::MonthDay(_, _) => "java/time/MonthDay",
        }
    }

    /// Whether this value is `Comparable`. An AMOUNT is not — "1 year and 2
    /// days" and "1 month and 40 days" have no order between them — nor is a
    /// field's range or an adjuster, so a `TreeSet` of any of the three is the
    /// cast error a JDK raises rather than a set.
    #[must_use]
    pub fn is_ordered(self) -> bool {
        !matches!(
            self,
            Temporal::Period(_) | Temporal::Range(_) | Temporal::Adjuster(_)
        )
    }

    /// What `toString()` gives.
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Temporal::Date(date) => date.to_string(),
            Temporal::Time(time) => time.to_string(),
            Temporal::Duration(amount) => amount.to_string(),
            Temporal::Period(period) => period.to_string(),
            Temporal::Unit(unit) => crate::time::unit_name(unit).to_owned(),
            Temporal::DateTime(when) => when.to_string(),
            Temporal::DayOfWeek(day) => crate::time::day_name(day).to_owned(),
            Temporal::Month(month) => crate::time::month_name(month).to_owned(),
            Temporal::Field(field) => crate::time::field_info(field).text.to_owned(),
            Temporal::Range(range) => range.text(),
            Temporal::Adjuster(adjuster) => format!("{adjuster:?}"),
            Temporal::Era(era) => String::from(if era == 0 { "BCE" } else { "CE" }),
            // An enum's default `toString` IS its constant, and neither of
            // these overrides it.
            Temporal::TextStyle(style) => String::from(TEXT_STYLE_NAMES[usize::from(style)]),
            Temporal::FormatStyle(style) => String::from(FORMAT_STYLE_NAMES[usize::from(style)]),
            // A year is written as at least four digits, and a month-day with
            // the two leading dashes that say it has no year.
            Temporal::Year(year) => format!("{year:04}"),
            Temporal::YearMonth(year, month) => format!("{year:04}-{month:02}"),
            Temporal::MonthDay(month, day) => format!("--{month:02}-{day:02}"),
        }
    }
}

/// Where a `PrintStream` puts what it is given: one of the two standard
/// streams, or a `ByteArrayOutputStream` the program owns — which is how a
/// `JUnit` test captures what a program prints (`System.setOut(new
/// PrintStream(captor))`), and two of Code.org's own validators do exactly
/// that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintSink {
    Std(StdStream),
    Bytes(HeapRef),
    /// A `java.io.StringWriter` — a `PrintWriter` over one collects CHARACTERS
    /// rather than bytes, which is what makes `sw.toString()` legible.
    Text(HeapRef),
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

/// Where a stream's elements come from.
///
/// Almost always a vector the source already holds. The two `Stream.iterate`
/// and `Stream.generate` shapes are INFINITE, though — the elements exist only
/// as a rule for making the next one, and only a short-circuiting operation
/// downstream ever ends the traversal (as in a JDK, where an unbounded
/// terminal simply never returns).
#[derive(Debug, Clone, PartialEq)]
pub enum StreamSource {
    Fixed(Vec<JValue>),
    /// `Stream.iterate(seed, next)`: the seed, then `next` of the one before.
    Iterate {
        seed: JValue,
        next: HeapRef,
    },
    /// `Stream.generate(supplier)`: a fresh call per element.
    Generate {
        supplier: HeapRef,
    },
    /// `matcher.results()`: the matches still ahead of a Matcher, pulled ONE
    /// at a time from the matcher itself. A JDK's is lazy over the same
    /// matcher — `results().limit(1)` consumes one match and leaves the rest
    /// to `find()` — so a fixed vector would have consumed them all.
    Matches {
        matcher: HeapRef,
    },
}

impl StreamSource {
    /// The elements already in hand — empty for a generated source, whose
    /// elements do not exist until they are pulled.
    #[must_use]
    pub fn fixed(&self) -> &[JValue] {
        match self {
            StreamSource::Fixed(values) => values,
            _ => &[],
        }
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
    /// The intrinsic object behind `System.out` / `System.err` — or one a
    /// program built over a byte buffer of its own.
    PrintStream(PrintSink),
    /// A `java.time` value: a `LocalDate`, or one of the two enums it
    /// answers with. Immutable, and compared BY VALUE — a `LocalDate` is not
    /// interned (`LocalDate.of(y,m,d) == LocalDate.of(y,m,d)` is false on a
    /// JDK, as it is here), while `DayOfWeek`/`Month` are real enum constants
    /// and must be the same object every time.
    Temporal(Temporal),
    /// A `java.time.format.DateTimeFormatter`: a pattern, or one of the ISO
    /// constants — which are NOT the value's `toString` (an ISO time always
    /// writes its seconds, where `toString` leaves them out).
    DateFormat(DateFormatKind),
    /// A `java.io.ByteArrayOutputStream`: the bytes written into it so far.
    /// The only `OutputStream` caturra models, and the one a test captures
    /// output with.
    ByteStream(Vec<u8>),
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
        /// The token separator, as a REGEX source, when `useDelimiter` set
        /// one. `None` is the JDK's default — any run of whitespace — which
        /// is not the same as the pattern `\s+`: the default also trims the
        /// input's leading and trailing whitespace away rather than yielding
        /// an empty token for it.
        delimiter: Option<String>,
        /// The radix `nextInt`/`nextLong`/`hasNextInt` read in, as
        /// `useRadix` set it. Ten until a program says otherwise, and back to
        /// ten after `reset()`. It does NOT reach `nextDouble`, which is
        /// always decimal — a JDK's `useRadix(16)` leaves `1.5` alone.
        radix: u32,
        /// What the LAST successful read matched: the token's text and where
        /// it was, which is all `match()` answers. `None` until a read
        /// succeeds — `hasNext()` alone does not set one, and asking before
        /// then is an `IllegalStateException`.
        matched: Option<(String, usize, usize)>,
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
        /// Where `reset()` returns to, set by `mark(readAheadLimit)`. A
        /// `StringReader` and a `BufferedReader` both support marks (a JDK's
        /// `markSupported` says so), and `reset()` before any `mark` returns
        /// to the START for a `StringReader` — which is what its own mark
        /// field is initialized to.
        mark: usize,
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
        source: StreamSource,
        ops: Vec<StreamOp>,
    },
    /// `list.subList(from, to)` — a live RANGE of another list. Reads and
    /// writes go through to the backing list, and a structural change made
    /// AROUND the view (rather than through it) invalidates it, exactly as a
    /// JDK's does. `seen` is the backing length the view last agreed with,
    /// which is how that change is noticed (caturra models modCount as the
    /// length; see the fail-fast cursors).
    SubList {
        backing: HeapRef,
        from: usize,
        len: usize,
        seen: usize,
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
        min: i64,
        max: i64,
        /// `Int` or `Long` — the two differ only in the identity values an
        /// EMPTY summary keeps and in the name it prints.
        kind: SummaryKind,
    },
    /// A `java.util.DoubleSummaryStatistics`. Its own variant rather than a
    /// third `kind`, because every number in it is a double: an empty one
    /// reports `Infinity` and `-Infinity`, and its `toString` prints the sum
    /// and bounds with `%f` where the integral ones print them whole.
    DoubleSummaryStats {
        count: i64,
        sum: f64,
        min: f64,
        max: f64,
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
    /// A live view of a sorted set or map: a RANGE of one, or the whole of it
    /// REVERSED, or both. Every `NavigableSet`/`NavigableMap` view is this one
    /// object — `headSet`/`tailSet`/`subSet`, `headMap`/`tailMap`/`subMap`,
    /// `descendingSet`, `descendingMap`, `descendingKeySet` and
    /// `navigableKeySet` differ only in their bounds, their direction, and
    /// which face they present.
    ///
    /// The bounds are the VALUES the program named, never positions, which is
    /// what keeps the view live: a key put into the backing map inside the
    /// range shows through, and one outside it does not. `range` is those
    /// bounds already resolved to a `from..to` slice of the backing's sorted
    /// vector, refreshed by every call made THROUGH the view (where the
    /// interpreter is at hand and the comparator may run user code). The plain
    /// readers — printing, `for` each, hashing — re-resolve it themselves
    /// whenever the ordering is NATIVE, which is every tree keyed by a number,
    /// a character or a String; only a tree ordered by user code falls back to
    /// the cached range.
    SortedView {
        backing: HeapRef,
        lo: Option<SortedBound>,
        hi: Option<SortedBound>,
        descending: bool,
        face: SortedFace,
        range: (usize, usize),
        /// The BACKING's length when the range was last resolved. A JDK's view
        /// cursor carries the TREE's `modCount`, not the view's own — so
        /// adding to the tree outside the range still ends an iteration of the
        /// view with a `ConcurrentModificationException`, which the view's own
        /// unchanged length could never notice. (caturra models `modCount` as
        /// a length everywhere; see the fail-fast cursors.)
        seen: usize,
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
        /// Built by `descendingIterator()`: walks from the END toward the
        /// front, so `next()` does what `previous()` does on a list cursor.
        /// The index is the position AFTER the element `next()` will return,
        /// which is what makes `remove()` land on the right one.
        descending: bool,
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
    /// A slot the collector reclaimed. Nothing points at it — if anything did,
    /// the mark phase would have kept the object — so it answers no method and
    /// simply waits to be handed out again.
    Free,
    /// The marker object behind `System.in`.
    InputStream,
    /// A `java.io.File`: a path into the virtual filesystem.
    File(String),
    /// A `java.nio.file.Path`: a filesystem path (from `Path.of`/`Paths.get`),
    /// read and written through `Files`.
    Path(String),
    /// A `java.math.BigInteger`: an integer of any size, immutable and
    /// compared BY VALUE. It is not interned — the small ones are cached by a
    /// JDK, but `==` on them is not a promise any program may lean on, so
    /// caturra allocates each one.
    BigInteger(crate::bigint::BigInt),
    /// A `java.math.BigDecimal`: an unscaled integer and a SCALE, which is
    /// observable — `2.0` and `2.00` compare equal, are not `equals`, and hash
    /// differently.
    BigDecimal(crate::decimal::BigDec),
    /// A `java.math.RoundingMode` constant, by its own ordinal. A real enum, so
    /// interned: `==` on one is what a program writes.
    RoundingMode(u8),
    /// A `java.math.MathContext`: how many significant digits to keep, and how
    /// to round what falls off.
    MathContext { precision: i32, mode: u8 },
    /// A `java.util.StringTokenizer` — the text, the delimiters, where the
    /// cursor sits, and whether the delimiters are handed out as tokens too.
    StringTokenizer {
        text: Vec<u16>,
        delimiters: Vec<u16>,
        pos: usize,
        return_delimiters: bool,
    },
    /// A `java.util.UUID` — two longs, and nothing else.
    Uuid(i64, i64),
    /// A `java.util.Base64.Encoder` or `.Decoder` — which alphabet, whether it
    /// pads, and whether it wraps at 76 characters.
    Base64 {
        url: bool,
        mime: bool,
        padding: bool,
        decoding: bool,
    },
    /// A `java.util.BitSet` — the bits, as 64 to a word.
    BitSet(Vec<u64>),
    /// A `java.io.StringWriter` — the characters written into it so far.
    StringWriter(Vec<u16>),
    /// A `Collections.synchronized*` wrapper. On one thread a monitor is never
    /// contended, so it is a live view that writes THROUGH and refuses
    /// nothing — the only reason it is an object of its own is that
    /// `getClass()` names the wrapper, and wrapping must not change what the
    /// original answers.
    SynchronizedView(HeapRef),
    /// A `java.io.BufferedWriter` over another writer. It really BUFFERS —
    /// what a program writes is not in the target until a `flush` or a
    /// `close`, and a JDK lets it see that: `sw.toString()` is empty in
    /// between. Modelling it as a pass-through would answer the wrong thing
    /// for the one program that checks.
    BufferedWriter {
        target: HeapRef,
        buffer: Vec<u16>,
        closed: bool,
    },
    /// A `java.text.DecimalFormat` — the parsed pattern, and the limits a
    /// program may then change on it. `NumberFormat` is the same object under
    /// a narrower name, which is what a JDK's factories answer with too.
    NumberFormat(Box<crate::numfmt::NumberPattern>),
    /// A `java.nio.charset.Charset` — `StandardCharsets.UTF_8` and the names
    /// beside it. It carries its canonical NAME and nothing else, which is all
    /// `getBytes`, `new String(bytes, …)` and its own `toString` need.
    Charset(String),
    /// A `java.util.regex.Pattern`: the pattern SOURCE as the program wrote it
    /// — which is what `pattern()` answers — beside the flags it was compiled
    /// with. The two are folded into what the engine reads at each use, and
    /// the compiled form is rebuilt there, exactly as `String.matches` does.
    Pattern { source: Vec<u16>, flags: i32 },
    /// A `java.util.regex.Matcher`: a pattern, the text it walks, where the
    /// next search starts, and the last match's spans (group 0 first) — plus
    /// the REGION it is confined to, how that region's edges behave, what the
    /// last attempt learned, and how far `appendReplacement` has copied.
    Matcher {
        pattern: HeapRef,
        input: Vec<u16>,
        at: usize,
        last: Option<Vec<Option<(usize, usize)>>>,
        region: (usize, usize),
        anchoring: bool,
        transparent: bool,
        hit_end: bool,
        require_end: bool,
        appended: usize,
    },
    /// A `java.util.regex.MatchResult` — one match, frozen: the spans it
    /// captured and the text they index. `toMatchResult` and the `results()`
    /// stream hand these out, and unlike a Matcher they never move.
    MatchResult {
        input: Vec<u16>,
        groups: Vec<Option<(usize, usize)>>,
    },
    /// The `Predicate` a `Pattern` answers — `asPredicate` (find) and
    /// `asMatchPredicate` (matches). It behaves as a lambda would, so
    /// `filter(pattern.asPredicate())` works, but it runs no user code.
    RegexPredicate { pattern: HeapRef, whole: bool },
    /// A `java.io.PrintWriter` into the virtual filesystem
    /// (write-through: output is durable without `close()`).
    Writer {
        path: String,
        /// A `PrintWriter` built over a `StringWriter` writes THERE instead of
        /// into the filesystem, and `sw.toString()` is how the program reads
        /// it back.
        text: Option<HeapRef>,
    },
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
        /// The one piece a few throwables carry that their MESSAGE does not.
        ///
        /// Most of them need nothing here: a JDK builds
        /// `UnknownFormatConversionException`'s message out of the conversion
        /// (`Conversion = 'q'`), so `getConversion()` reads it back out of the
        /// message and a CONSTRUCTED one and a THROWN one answer alike with
        /// one implementation. Only where the message does not carry the value
        /// — `ParseException`'s error offset, `DateTimeParseException`'s
        /// parsed text and index — is it stored, `\u{1f}`-separated.
        detail: Option<String>,
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

/// Every heap reference a value holds — nothing for a primitive, one for a
/// non-null reference.
pub fn visit_value(value: JValue, visit: &mut impl FnMut(HeapRef)) {
    if let JValue::Ref(Some(reference)) = value {
        visit(reference);
    }
}

impl StreamOp {
    /// The lambda a pipeline stage holds.
    fn visit_refs(&self, visit: &mut impl FnMut(HeapRef)) {
        match self {
            StreamOp::Filter(f)
            | StreamOp::Map(f)
            | StreamOp::Peek(f)
            | StreamOp::TakeWhile(f)
            | StreamOp::DropWhile(f) => visit(*f),
            StreamOp::Sorted(comparator) => {
                if let Some(comparator) = comparator {
                    visit(*comparator);
                }
            }
            StreamOp::Limit(_)
            | StreamOp::Skip(_)
            | StreamOp::Distinct
            | StreamOp::Box
            | StreamOp::WidenToLong
            | StreamOp::WidenToDouble => {}
        }
    }
}

impl CollectorKind {
    fn visit_refs(&self, visit: &mut impl FnMut(HeapRef)) {
        match self {
            CollectorKind::ToList
            | CollectorKind::ToSet
            | CollectorKind::Joining { .. }
            | CollectorKind::Counting => {}
            CollectorKind::Unmodifiable(inner) => inner.visit_refs(visit),
            CollectorKind::GroupingBy {
                classifier,
                downstream,
                factory,
            } => {
                visit(*classifier);
                if let Some(downstream) = downstream {
                    visit(*downstream);
                }
                if let Some(factory) = factory {
                    visit(*factory);
                }
            }
            CollectorKind::Summing { mapper: f, .. }
            | CollectorKind::Summarizing { mapper: f, .. }
            | CollectorKind::Extreme { comparator: f, .. }
            // The supplier is a live reference: it is called when the
            // collector finishes, so the GC must keep it until then.
            | CollectorKind::ToCollection(f) => {
                visit(*f);
            }
            // The two-part collectors: the callback AND the collector under
            // it are both live until the gathering finishes.
            CollectorKind::Filtering {
                predicate: first,
                downstream: second,
            }
            | CollectorKind::FlatMapping {
                mapper: first,
                downstream: second,
            }
            | CollectorKind::CollectingAndThen {
                downstream: first,
                finisher: second,
            } => {
                visit(*first);
                visit(*second);
            }
            CollectorKind::Reducing {
                identity,
                mapper,
                operator,
            } => {
                if let Some(JValue::Ref(Some(seed))) = identity {
                    visit(*seed);
                }
                if let Some(mapper) = mapper {
                    visit(*mapper);
                }
                visit(*operator);
            }
            CollectorKind::PartitioningBy {
                predicate,
                downstream,
            } => {
                visit(*predicate);
                if let Some(downstream) = downstream {
                    visit(*downstream);
                }
            }
            CollectorKind::Mapping { mapper, downstream } => {
                visit(*mapper);
                visit(*downstream);
            }
            CollectorKind::ToMap {
                key,
                value,
                merge,
                factory,
            } => {
                visit(*key);
                if let Some(factory) = factory {
                    visit(*factory);
                }
                visit(*value);
                if let Some(merge) = merge {
                    visit(*merge);
                }
            }
        }
    }
}

impl ComparatorSpec {
    fn visit_refs(&self, visit: &mut impl FnMut(HeapRef)) {
        match self {
            ComparatorSpec::Natural | ComparatorSpec::CaseInsensitive => {}
            ComparatorSpec::ByKey(f) | ComparatorSpec::Reversed(f) => visit(*f),
            ComparatorSpec::Then(a, b) | ComparatorSpec::ByKeyWith(a, b) => {
                visit(*a);
                visit(*b);
            }
            // Both carry an OPTIONAL inner comparator — an entry factory given
            // an ordering for its key, and a nulls-first/last wrapper.
            ComparatorSpec::Entry { inner, .. } | ComparatorSpec::Nulls { inner, .. } => {
                if let Some(inner) = inner {
                    visit(*inner);
                }
            }
        }
    }
}

impl StreamSource {
    fn visit_refs(&self, visit: &mut impl FnMut(HeapRef)) {
        match self {
            StreamSource::Fixed(values) => {
                for value in values {
                    visit_value(*value, visit);
                }
            }
            StreamSource::Iterate { seed, next } => {
                visit_value(*seed, visit);
                visit(*next);
            }
            StreamSource::Generate { supplier } => visit(*supplier),
            StreamSource::Matches { matcher } => visit(*matcher),
        }
    }
}

impl HeapObject {
    /// Every heap reference this object holds. The collector's mark phase
    /// walks it, so a reference MISSED here is an object freed while something
    /// still points at it — the one bug a collector must not have. The match
    /// has no wildcard arm for exactly that reason: a new variant that holds a
    /// reference cannot compile until it is listed.
    #[allow(clippy::too_many_lines)] // one arm per heap kind, and no wildcard
    pub fn visit_refs(&self, visit: &mut impl FnMut(HeapRef)) {
        let values = |values: &Vec<JValue>, visit: &mut dyn FnMut(HeapRef)| {
            for value in values {
                if let JValue::Ref(Some(reference)) = value {
                    visit(*reference);
                }
            }
        };
        match self {
            // A `PrintStream` over a program's own buffer KEEPS that buffer
            // alive: `System.setOut(new PrintStream(captor))` and then reading
            // `captor` after collection.
            HeapObject::PrintStream(PrintSink::Bytes(reference) | PrintSink::Text(reference)) => {
                visit(*reference);
            }
            // ...and a buffered writer holds whatever it wraps.
            HeapObject::BufferedWriter { target, .. } => visit(*target),
            // A writer over a `StringWriter` holds it.
            HeapObject::Writer { text, .. } => {
                if let Some(reference) = text {
                    visit(*reference);
                }
            }
            // No references at all: the text, the primitive arrays, the
            // handles that hold only names, the stream/file endpoints.
            HeapObject::JavaString(_)
            | HeapObject::StringBuilder(_)
            | HeapObject::ByteStream(_)
            | HeapObject::Temporal(_)
            | HeapObject::DateFormat(_)
            | HeapObject::PrintStream(PrintSink::Std(_))
            | HeapObject::IntArray(_, _)
            | HeapObject::DoubleArray(_)
            | HeapObject::LongArray(_)
            | HeapObject::FloatArray(_)
            | HeapObject::ShortArray(_)
            | HeapObject::ByteArray(_)
            | HeapObject::Scanner { .. }
            | HeapObject::Reader { .. }
            | HeapObject::SummaryStats { .. }
            | HeapObject::DoubleSummaryStats { .. }
            | HeapObject::InputStream
            | HeapObject::File(_)
            | HeapObject::Path(_)
            | HeapObject::Charset(_)
            | HeapObject::BigInteger(_)
            | HeapObject::BigDecimal(_)
            | HeapObject::RoundingMode(_)
            | HeapObject::MathContext { .. }
            | HeapObject::NumberFormat(_)
            | HeapObject::StringTokenizer { .. }
            | HeapObject::Uuid(_, _)
            | HeapObject::Base64 { .. }
            | HeapObject::BitSet(_)
            | HeapObject::StringWriter(_)
            | HeapObject::Pattern { .. }
            | HeapObject::MatchResult { .. }
            | HeapObject::Class { .. }
            | HeapObject::Field { .. }
            | HeapObject::ReflectType { .. }
            | HeapObject::StackFrame { .. }
            | HeapObject::Constructor { .. }
            | HeapObject::Free
            | HeapObject::Method { .. } => {}
            // A matcher — and the predicate a pattern answers — hold the
            // pattern they walk.
            HeapObject::Matcher { pattern, .. } | HeapObject::RegexPredicate { pattern, .. } => {
                visit(*pattern);
            }
            HeapObject::Boxed { value, .. } => visit_value(*value, visit),
            HeapObject::RefArray(_, items)
            | HeapObject::ArrayList(items)
            | HeapObject::LinkedList(items)
            | HeapObject::ArrayDeque(items)
            | HeapObject::Stack(items) => values(items, visit),
            HeapObject::Instance { fields, .. } => values(fields, visit),
            HeapObject::UnmodifiableList(inner)
            | HeapObject::ArrayBackedList(inner)
            // A synchronized wrapper holds what it wraps, exactly as an
            // unmodifiable one does.
            | HeapObject::SynchronizedView(inner)
            | HeapObject::UnmodifiableSet(inner)
            | HeapObject::UnmodifiableMap(inner)
            | HeapObject::SubList { backing: inner, .. }
            | HeapObject::Iterator { source: inner, .. }
            | HeapObject::MapView { map: inner, .. } => visit(*inner),
            // A view's BOUNDS are values, and a value is a reference when the
            // tree is keyed by objects: `set.headSet(new Point(4))` holds the
            // only pointer to that Point. Visiting the backing alone freed it.
            HeapObject::SortedView {
                backing, lo, hi, ..
            } => {
                visit(*backing);
                for bound in [lo, hi].into_iter().flatten() {
                    visit_value(bound.value, visit);
                }
            }
            HeapObject::HashMap(map) | HeapObject::HashSet(map) => map.visit_refs(visit),
            HeapObject::TreeMap {
                entries,
                comparator,
            } => {
                for (key, value) in entries {
                    visit_value(*key, visit);
                    visit_value(*value, visit);
                }
                if let Some(comparator) = comparator {
                    visit(*comparator);
                }
            }
            HeapObject::Stream { source, ops } => {
                source.visit_refs(visit);
                for op in ops {
                    op.visit_refs(visit);
                }
            }
            HeapObject::Collector(kind) => kind.visit_refs(visit),
            HeapObject::Comparator(spec) => spec.visit_refs(visit),
            HeapObject::Optional { value, .. } => {
                if let Some(value) = value {
                    visit_value(*value, visit);
                }
            }
            HeapObject::PriorityQueue { heap, comparator }
            | HeapObject::TreeSet {
                values: heap,
                comparator,
            } => {
                values(heap, visit);
                if let Some(comparator) = comparator {
                    visit(*comparator);
                }
            }
            HeapObject::MapEntry { map, key, .. } => {
                visit(*map);
                visit_value(*key, visit);
            }
            HeapObject::Exception {
                cause, suppressed, ..
            } => {
                if let Some(cause) = cause {
                    visit(*cause);
                }
                for one in suppressed {
                    visit(*one);
                }
            }
        }
    }
}

impl HeapObject {
    /// Roughly how much memory this object costs the HOST — which is what the
    /// heap budget is about. The production VM is a browser WASM instance, and
    /// a program that outgrows it does not get a slow answer, it gets a dead
    /// tab; a budget it can be told about turns that into a catchable
    /// `OutOfMemoryError`. An estimate is enough for that: the exact byte
    /// count of a Rust `Vec` is not what a Java program reasons about anyway.
    #[must_use]
    pub fn approximate_bytes(&self) -> usize {
        const BASE: usize = size_of::<HeapObject>();
        let value = size_of::<JValue>();
        BASE + match self {
            HeapObject::JavaString(units) | HeapObject::StringBuilder(units) => units.len() * 2,
            HeapObject::IntArray(_, items) => items.len() * 4,
            HeapObject::DoubleArray(items) => items.len() * 8,
            HeapObject::LongArray(items) => items.len() * 8,
            HeapObject::FloatArray(items) => items.len() * 4,
            HeapObject::ShortArray(items) => items.len() * 2,
            HeapObject::ByteArray(items) => items.len(),
            HeapObject::RefArray(_, items)
            | HeapObject::ArrayList(items)
            | HeapObject::LinkedList(items)
            | HeapObject::ArrayDeque(items)
            | HeapObject::Stack(items)
            | HeapObject::PriorityQueue { heap: items, .. }
            | HeapObject::TreeSet { values: items, .. } => items.len() * value,
            HeapObject::Instance { fields, .. } => fields.len() * value,
            HeapObject::TreeMap { entries, .. } => entries.len() * value * 2,
            HeapObject::HashMap(map) | HeapObject::HashSet(map) => map.len() * value * 3,
            HeapObject::Stream { source, .. } => size_of_val(source.fixed()),
            HeapObject::Scanner { buffer, .. } | HeapObject::Reader { buffer, .. } => buffer.len(),
            _ => 0,
        }
    }
}

/// A heap smaller than this never collects: the walk costs more than the slots
/// are worth, and a short program should not pay for one at all.
const HEAP_FLOOR: usize = 1 << 16;

/// Allocating this many bytes since the last collection is reason enough for
/// another one, however few OBJECTS they were. A single `new int[10_000_000]`
/// is one slot and forty megabytes.
const BYTES_PER_COLLECTION: usize = 64 << 20;

/// The per-run object heap.
/// `new StringBuilder()` starts here, and a `String` seed adds its length.
pub const DEFAULT_BUILDER_CAPACITY: usize = 16;

#[derive(Debug)]
pub struct Heap {
    objects: Vec<HeapObject>,
    /// Slots the collector reclaimed, ready to be handed out again. A
    /// reference is an INDEX, so a swept slot can be reused as it stands —
    /// nothing moves, and every reference the program holds keeps pointing at
    /// the object it named.
    free: Vec<HeapRef>,
    /// Bytes allocated since the last collection, so a program that allocates
    /// FEW but HUGE objects still reaches a safepoint's collection — the slot
    /// count alone would never notice one array of ten million.
    allocated_bytes: usize,
    /// What the last sweep found live, for the budget check.
    live_bytes: usize,
    /// How large the live set may grow before the next safepoint collects.
    /// Set after each collection to twice what survived (never below the
    /// floor), so a program that really does hold a large live set stops
    /// paying for collections that free nothing.
    threshold: usize,
    /// The autoboxing cache (JLS §5.1.7): `Wrapper.valueOf` returns a SHARED
    /// reference for a value in the cached range, so `Integer a = 100, b = 100;
    /// a == b` is true while `200 == 200` (out of range) is false. Keyed by
    /// (wrapper tag, value) — see [`Heap::box_wrapper`].
    wrapper_cache: std::collections::HashMap<(u8, i64), HeapRef>,
    /// The `java.time` enum constants, keyed by (kind, value).
    enum_pool: std::collections::HashMap<(u8, u8), HeapRef>,
    /// What class a VIEW object reports: `Collections.emptyList()` and
    /// `List.of(a, b)` are both an unmodifiable list here, and a JDK names
    /// them `Collections$EmptyList` and `ImmutableCollections$List12`. It
    /// lives on the HEAP rather than the interpreter because every path that
    /// names a class needs it — including the formatter, whose exception
    /// message was answering the generic name while `getClass()` answered the
    /// real one.
    view_class: std::collections::HashMap<HeapRef, &'static str>,
    /// What `new Vector<>(n)` asked for. A JDK's `capacity()` is that figure
    /// doubled as often as the contents needed, and nothing else can observe
    /// it — so the initial number is all there is to remember.
    vector_capacity: std::collections::HashMap<HeapRef, (usize, usize)>,
    /// What an object RENDERS TO in a format call. Only the interpreter can
    /// run a user `toString`, so `String.format`'s arguments are rendered
    /// ahead of the formatter — but the formatter still has to name the
    /// argument's CLASS when a conversion rejects it (`%d` of a list), so the
    /// text is recorded BESIDE the object rather than in place of it. Pruned
    /// by the collector like every other side map.
    format_text: std::collections::HashMap<HeapRef, (Option<std::rc::Rc<str>>, Option<i32>)>,
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

impl Default for Heap {
    /// A fresh heap collects only once it passes the floor — deriving this
    /// would have started the threshold at ZERO, so the first allocation of
    /// every program would have triggered a collection.
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            free: Vec::new(),
            allocated_bytes: 0,
            live_bytes: 0,
            threshold: HEAP_FLOOR,
            wrapper_cache: std::collections::HashMap::new(),
            enum_pool: std::collections::HashMap::new(),
            view_class: std::collections::HashMap::new(),
            vector_capacity: std::collections::HashMap::new(),
            format_text: std::collections::HashMap::new(),
            builder_capacity: std::collections::HashMap::new(),
        }
    }
}

impl Heap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate an object, returning its reference.
    pub fn alloc(&mut self, object: HeapObject) -> HeapRef {
        self.allocated_bytes += object.approximate_bytes();
        if let Some(slot) = self.free.pop() {
            self.objects[slot as usize] = object;
            return slot;
        }
        let index = u32::try_from(self.objects.len()).expect("heap exhausted");
        self.objects.push(object);
        index
    }

    /// How many slots exist, live and free alike.
    #[must_use]
    pub fn slots(&self) -> usize {
        self.objects.len()
    }

    /// Whether the live set has grown past the point where collecting is worth
    /// the walk.
    #[must_use]
    pub fn wants_collection(&self) -> bool {
        self.objects.len().saturating_sub(self.free.len()) >= self.threshold
            || self.allocated_bytes >= BYTES_PER_COLLECTION
    }

    /// What the last sweep found live. Zero before the first collection, so a
    /// budget check has to run one first.
    #[must_use]
    pub fn live_bytes(&self) -> usize {
        self.live_bytes
    }

    /// The wrapper cache's boxes: a ROOT, because `Integer.valueOf(1)` must
    /// answer the same reference for the life of the program.
    pub fn cached_boxes(&self) -> impl Iterator<Item = HeapRef> + '_ {
        self.wrapper_cache.values().copied()
    }

    /// A `java.time` ENUM constant — the same object every time, because
    /// `date.getDayOfWeek() == DayOfWeek.MONDAY` is how the comparison is
    /// written and an enum constant is a singleton. A `LocalDate` is NOT
    /// interned (a JDK's is not either), so it allocates.
    pub fn intern_temporal(&mut self, value: Temporal) -> HeapRef {
        let key = match value {
            // Only the two ENUMS are interned; a date, a time and a date-time
            // are ordinary values, and a JDK does not intern those either.
            Temporal::Date(_)
            | Temporal::Time(_)
            | Temporal::DateTime(_)
            | Temporal::Duration(_)
            | Temporal::Period(_)
            | Temporal::Range(_)
            // A year, a year-month and a month-day are VALUES, not enums: a
            // JDK does not intern them and `==` on two is false.
            | Temporal::Year(_)
            | Temporal::YearMonth(_, _)
            | Temporal::MonthDay(_, _)
            | Temporal::Adjuster(_) => {
                return self.alloc(HeapObject::Temporal(value));
            }
            Temporal::Unit(unit) => (2u8, unit),
            Temporal::DayOfWeek(day) => (0u8, day),
            Temporal::Month(month) => (1u8, month),
            Temporal::Field(field) => (3u8, field),
            Temporal::Era(era) => (4u8, era),
            Temporal::TextStyle(style) => (6u8, style),
            Temporal::FormatStyle(style) => (7u8, style),
        };
        if let Some(existing) = self.enum_pool.get(&key) {
            return *existing;
        }
        let reference = self.alloc(HeapObject::Temporal(value));
        self.enum_pool.insert(key, reference);
        reference
    }

    /// `RoundingMode.HALF_UP` and the seven beside it — interned in the same
    /// pool the `java.time` enums use, because they are enums for the same
    /// reason: a program compares them with `==`.
    pub fn intern_rounding_mode(&mut self, ordinal: u8) -> HeapRef {
        let key = (5u8, ordinal);
        if let Some(existing) = self.enum_pool.get(&key) {
            return *existing;
        }
        let reference = self.alloc(HeapObject::RoundingMode(ordinal));
        self.enum_pool.insert(key, reference);
        reference
    }

    /// Record what class a view object answers to.
    pub fn set_view_class(&mut self, reference: HeapRef, class: &'static str) {
        self.view_class.insert(reference, class);
    }

    /// Record what a `Vector`'s constructor asked for: the capacity, and the
    /// growth STEP beside it — a positive increment adds that many slots where
    /// the default doubles, and `capacity()` is where the difference shows.
    pub fn set_vector_capacity(&mut self, reference: HeapRef, capacity: usize, increment: usize) {
        self.vector_capacity
            .insert(reference, (capacity, increment));
    }

    /// ...and read it back.
    #[must_use]
    pub fn vector_capacity_of(&self, reference: HeapRef) -> Option<(usize, usize)> {
        self.vector_capacity.get(&reference).copied()
    }

    /// The class a view answers to, if it is one.
    #[must_use]
    pub fn view_class_of(&self, reference: HeapRef) -> Option<&'static str> {
        self.view_class.get(&reference).copied()
    }

    /// Drop the views whose objects the collector swept.
    pub fn retain_views(&mut self, alive: impl Fn(HeapRef) -> bool) {
        self.view_class.retain(|reference, _| alive(*reference));
        self.vector_capacity
            .retain(|reference, _| alive(*reference));
    }

    /// The interned library ENUM constants — `java.time`'s and
    /// `RoundingMode`'s: roots, for the same reason the boxes are.
    pub fn interned_enums(&self) -> impl Iterator<Item = HeapRef> + '_ {
        self.enum_pool.values().copied()
    }

    /// The object at a slot, for the collector's walk — which must see even a
    /// slot the program can no longer reach.
    #[must_use]
    pub fn object_at(&self, index: usize) -> Option<&HeapObject> {
        self.objects.get(index)
    }

    /// Free every slot the mark phase did not reach, and set the threshold for
    /// the next collection from what survived. Nothing moves: a swept slot is
    /// simply available again.
    pub fn sweep(&mut self, marked: &[bool]) -> usize {
        let mut freed = 0;
        let mut live_bytes = 0;
        for index in 0..self.objects.len() {
            if marked.get(index).copied().unwrap_or(true) {
                live_bytes += self.objects[index].approximate_bytes();
                continue;
            }
            if matches!(self.objects[index], HeapObject::Free) {
                continue;
            }
            self.objects[index] = HeapObject::Free;
            let reference = u32::try_from(index).expect("index came from this vector");
            self.builder_capacity.remove(&reference);
            self.free.push(reference);
            freed += 1;
        }
        let live = self.objects.len() - self.free.len();
        self.threshold = std::cmp::max(HEAP_FLOOR, live.saturating_mul(2));
        self.live_bytes = live_bytes;
        self.allocated_bytes = 0;
        freed
    }

    /// Allocate a Java string from Rust UTF-8 text.
    pub fn alloc_string(&mut self, text: &str) -> HeapRef {
        self.alloc(HeapObject::JavaString(text.encode_utf16().collect()))
    }

    /// A string from UTF-16 units directly. Going through a Rust `String`
    /// loses a LONE surrogate — `String::from_utf16_lossy` writes U+FFFD —
    /// and a `char` in a Java string is allowed to be one.
    pub fn alloc_string_units(&mut self, units: &[u16]) -> HeapRef {
        self.alloc(HeapObject::JavaString(units.to_vec()))
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
    /// Record what `reference` renders to for a format call.
    pub fn set_format_text(&mut self, reference: HeapRef, text: &str) {
        self.format_text.entry(reference).or_default().0 = Some(std::rc::Rc::from(text));
    }

    /// Record what `reference`'s `hashCode()` answered, for `%h`.
    pub fn set_format_hash(&mut self, reference: HeapRef, hash: i32) {
        self.format_text.entry(reference).or_default().1 = Some(hash);
    }

    /// What `reference` renders to, if a format call asked for it.
    #[must_use]
    pub fn rendered_format_text(&self, reference: HeapRef) -> Option<&str> {
        self.format_text
            .get(&reference)
            .and_then(|(text, _)| text.as_deref())
    }

    /// The hash a format call took of `reference`.
    #[must_use]
    pub fn rendered_format_hash(&self, reference: HeapRef) -> Option<i32> {
        self.format_text.get(&reference).and_then(|(_, hash)| *hash)
    }

    /// Drop the rendered text of objects the collector reclaimed.
    pub fn retain_format_text(&mut self, alive: impl Fn(&HeapRef) -> bool) {
        self.format_text.retain(|reference, _| alive(reference));
    }

    #[must_use]
    pub fn string_text(&self, reference: HeapRef) -> Option<String> {
        match self.get(reference)? {
            HeapObject::JavaString(units) => Some(String::from_utf16_lossy(units)),
            _ => None,
        }
    }

    /// The string's own UTF-16 units, which `string_text` cannot hand back
    /// faithfully: an unpaired surrogate is a `char` no Rust `String` holds.
    #[must_use]
    pub fn string_units(&self, reference: HeapRef) -> Option<&[u16]> {
        match self.get(reference)? {
            HeapObject::JavaString(units) => Some(units),
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
