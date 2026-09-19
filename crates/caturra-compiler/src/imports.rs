//! Import validation and enforcement.
//!
//! Two directions, both matching javac where the library overlaps:
//! - import declarations must name something real (unknown classes and
//!   packages get javac's wording; real Java classes outside the CSA
//!   library get an honest "not supported by caturra" instead), and
//! - using a non-`java.lang` library class (`Scanner`, `ArrayList`,
//!   `File`, `PrintWriter`) requires the matching import — javac's
//!   "cannot find symbol: class Scanner". User-defined classes shadow
//!   library names and never need imports.

use std::collections::HashSet;

use crate::ast::{ClassDecl, CompilationUnit, Expr, ImportDecl, Stmt, TypeRef};
use crate::diagnostics::{Diagnostic, SourceSpan};

/// Importable library classes, per package.
const JAVA_UTIL: &[&str] = &[
    "Scanner",
    // The pre-`split` word walker, and the identifier every "give this a
    // unique name" exercise reaches for.
    "StringTokenizer",
    "UUID",
    "Base64",
    "BitSet",
    "AbstractMap",
    "ArrayList",
    "List",
    "HashMap",
    "LinkedHashMap",
    "Map",
    "TreeMap",
    "SortedMap",
    "NavigableMap",
    "Set",
    "HashSet",
    "LinkedHashSet",
    "TreeSet",
    "SortedSet",
    "NavigableSet",
    "LinkedList",
    "Queue",
    "Deque",
    "PriorityQueue",
    "ArrayDeque",
    "Stack",
    // The pre-collections trio: the synchronized list and map every textbook
    // written before 1998 uses, and the cursor they hand back.
    "Vector",
    "Hashtable",
    "Enumeration",
    // A `Vector` and an `ArrayList` are `RandomAccess`, and an algorithm that
    // asks is asking about a real interface.
    "RandomAccess",
    "Collection",
    "Comparator",
    "Iterator",
    "ListIterator",
    "Optional",
    "OptionalInt",
    "OptionalDouble",
    "OptionalLong",
    // What `IntStream.summaryStatistics()` answers. The TYPE was modelled and
    // the name was not importable, so a program could chain through one and
    // never name it: "cannot find symbol: class IntSummaryStatistics".
    "IntSummaryStatistics",
    "LongSummaryStatistics",
    "DoubleSummaryStatistics",
    "Arrays",
    "Objects",
    "Random",
    // The enum-keyed collections. Their ITERATION ORDER is the constants'
    // declaration order, which is an enum's natural ordering — so they are the
    // sorted collections underneath and a plain Map/Set on the surface.
    "EnumMap",
    "EnumSet",
    "Collections",
    "StringJoiner",
    "InputMismatchException",
    "NoSuchElementException",
    "EmptyStackException",
    "ConcurrentModificationException",
    "IllegalFormatException",
    "UnknownFormatConversionException",
    "MissingFormatArgumentException",
    "IllegalFormatConversionException",
    "IllegalFormatCodePointException",
    // The rest of the family a bad format string throws. Every one of these is
    // raised BY NAME from `String.format` already; they were simply not
    // importable, so a program that caught one under its own single import was
    // told the class does not exist in java.util.
    "DuplicateFormatFlagsException",
    "FormatFlagsConversionMismatchException",
    "IllegalFormatFlagsException",
    "IllegalFormatPrecisionException",
    "IllegalFormatWidthException",
    "MissingFormatWidthException",
    // `Locale` is usable only as the leading argument of a format call, which
    // is where a program actually reaches for it: caturra formats in the
    // US/root locale, so `String.format(Locale.US, …)` is exact and the
    // argument is dropped.
    "Locale",
];

const JAVA_IO: &[&str] = &[
    "File",
    "PrintWriter",
    // The two that make text look like a file: a reader over a String, and a
    // writer a program reads back.
    "StringReader",
    "StringWriter",
    // `System.out` IS a `PrintStream`, and a test captures printing by
    // handing `System.setOut` one over a `ByteArrayOutputStream` — the only
    // `OutputStream` there is here, which is why the abstract name is a face
    // of it rather than a type of its own.
    "PrintStream",
    "ByteArrayOutputStream",
    "OutputStream",
    "BufferedReader",
    "FileReader",
    "InputStreamReader",
    "FileNotFoundException",
    "IOException",
    // `Closeable` is modeled as an interface (it is `AutoCloseable` plus a
    // narrower `throws`), so a resource class may implement the one every
    // I/O tutorial names.
    "Closeable",
    // Modelled, and absent from this list — so each worked written under a
    // wildcard import and was "cannot find symbol ... in package java.io"
    // under its own. `Reader` is the face the three readers share, and
    // `FileWriter` writes one.
    "Reader",
    // The abstract FACE both writers wear — a `PrintWriter` and a
    // `StringWriter` are each one, and a variable holding either is declared
    // as this.
    "Writer",
    "BufferedWriter",
    "FileWriter",
    "UncheckedIOException",
    "UnsupportedEncodingException",
];
/// `java.nio.file` — the modeled slice: build a `Path` and read/write it through
/// `Files`. The rest of `java.nio` stays unsupported.
const JAVA_NIO_FILE: &[&str] = &["Files", "Path", "Paths"];

/// `java.time` — the slice that is pure calendar arithmetic. A date, and the
/// two enums it answers with. What needs a ZONE (a `ZonedDateTime`, or what
/// "today" is) is not here: the browser has the IANA database and vendoring a
/// second copy would only add a version to disagree with.
const JAVA_TIME: &[&str] = &[
    "LocalDate",
    // The three PARTIAL dates: a year, a month of a year, and a day of a year
    // that has no year.
    "Year",
    "YearMonth",
    "MonthDay",
    "LocalTime",
    "LocalDateTime",
    "Duration",
    "Period",
    "DayOfWeek",
    "Month",
    "DateTimeException",
];

/// `java.time.chrono` — the era a date belongs to.
const JAVA_TIME_CHRONO: &[&str] = &["IsoEra", "IsoChronology"];

/// `java.time.temporal` — the unit a program names to ask `between`.
const JAVA_TIME_TEMPORAL: &[&str] = &[
    "ChronoUnit",
    "ChronoField",
    "ValueRange",
    "TemporalAdjusters",
    "TemporalQueries",
    "TemporalQuery",
    "TemporalAdjuster",
    "UnsupportedTemporalTypeException",
];

/// `java.time.format` — only the exception, so far: `LocalDate.parse` throws
/// it and a program may catch it by name.
/// `TextStyle` and `FormatStyle` name a style, and caturra reads the constant
/// where it is written — `Month.getDisplayName(TextStyle.FULL, Locale.US)`,
/// `DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM)`. Like `Locale`
/// beside them they are usable as a QUALIFIER and not as a variable's type:
/// there is no value of either to hold, and naming one says so.
const JAVA_TIME_FORMAT: &[&str] = &[
    "DateTimeFormatter",
    "DateTimeParseException",
    "TextStyle",
    "FormatStyle",
];
/// `java.math` — arbitrary-precision arithmetic: the integers a `long`
/// overflows on.
const JAVA_MATH: &[&str] = &["BigInteger", "BigDecimal", "RoundingMode", "MathContext"];

/// `java.text` — a number rendered for a person to read, by pattern.
const JAVA_TEXT: &[&str] = &[
    "DecimalFormat",
    "NumberFormat",
    "ParseException",
    "ParsePosition",
    "FieldPosition",
];

/// `java.nio.charset` — the charsets a program names when it turns text into
/// bytes and back. `Charset` is the type; `StandardCharsets` holds the
/// constants; the exceptions are what an unknown name throws.
const JAVA_NIO_CHARSET: &[&str] = &[
    "Charset",
    "StandardCharsets",
    "UnsupportedCharsetException",
    "IllegalCharsetNameException",
];
/// `java.util.function` — the standard functional interfaces. They alias the
/// bundled erased `__`-interfaces (see `functional_erased` in codegen).
/// Every `java.util.function` interface caturra names. The SAME set has to be
/// known in two other places — its erased interface (`functional_erased`) and
/// its SAM shape (the lambda pass) — and a name present here but missing there
/// is a type that a program may WRITE and then cannot use. A test walks this
/// list against both.
pub(crate) const JAVA_UTIL_FUNCTION: &[&str] = &[
    "Function",
    "BiFunction",
    "UnaryOperator",
    "BinaryOperator",
    "Predicate",
    "BiPredicate",
    "Consumer",
    "BiConsumer",
    "Supplier",
    // The PRIMITIVE specializations. Each erases onto one of the bundled SAMs
    // above (same arity, same shape) and differs only in the name of its
    // method, so they cost a table entry each rather than an interface each.
    // Half of `java.util.function` was nameable and half was not, which is not
    // a distinction a program can be expected to keep track of.
    "IntFunction",
    "IntPredicate",
    "IntSupplier",
    "IntConsumer",
    "IntUnaryOperator",
    "IntBinaryOperator",
    "DoublePredicate",
    "DoubleSupplier",
    "DoubleConsumer",
    "DoubleUnaryOperator",
    "DoubleBinaryOperator",
    "LongPredicate",
    "LongSupplier",
    "LongConsumer",
    "LongUnaryOperator",
    "LongBinaryOperator",
    "BooleanSupplier",
    "ToIntFunction",
    "ToDoubleFunction",
    "ToLongFunction",
    // The rest of the package. `java.util.function` has 43 interfaces in Java
    // 11 and 29 of them were nameable, which is not a line a program can be
    // expected to know is there.
    "DoubleFunction",
    "LongFunction",
    "IntToLongFunction",
    "IntToDoubleFunction",
    "LongToIntFunction",
    "LongToDoubleFunction",
    "DoubleToIntFunction",
    "DoubleToLongFunction",
    "ObjIntConsumer",
    "ObjLongConsumer",
    "ObjDoubleConsumer",
    "ToIntBiFunction",
    "ToLongBiFunction",
    "ToDoubleBiFunction",
];
/// `java.util.regex` — the compiled pattern, the matcher that walks an input,
/// the frozen match either hands out, and the exception a bad pattern throws.
const JAVA_UTIL_REGEX: &[&str] = &[
    "Pattern",
    "Matcher",
    "MatchResult",
    "PatternSyntaxException",
];

const JAVA_UTIL_STREAM: &[&str] = &[
    "Stream",
    "IntStream",
    "LongStream",
    "DoubleStream",
    "Collectors",
    "Collector",
];
/// `java.lang` is implicitly imported; explicit imports of it are
/// legal (and redundant) in Java, so accept the names we model.
const JAVA_LANG: &[&str] = &[
    "String",
    "Object",
    "System",
    "Math",
    "Integer",
    "Double",
    "Long",
    "Float",
    "Short",
    "Byte",
    "Boolean",
    "Character",
    "Number",
    "StringBuilder",
    // Its synchronized twin. On one thread there is no lock to take, so the
    // two share a storage and a table here; the class each NAMES is recorded
    // on the object, which is what kept them apart.
    "StringBuffer",
    // Modelled as types and reachable unqualified, but absent from this list,
    // so their class literals reported a bare `CharSequence`/`Iterable`.
    "CharSequence",
    "Iterable",
    "Comparable",
    "Exception",
    "RuntimeException",
    "ArithmeticException",
    "NullPointerException",
    "ArrayIndexOutOfBoundsException",
    "IndexOutOfBoundsException",
    "StringIndexOutOfBoundsException",
    "NegativeArraySizeException",
    "NumberFormatException",
    "ClassCastException",
    "StackOverflowError",
    "Throwable",
    "Error",
    "IllegalArgumentException",
    "IllegalStateException",
    // `Runnable` is a functional interface, not a threading one: `r.run()`
    // runs on the spot. `Thread` stays unsupported (a program here runs on one
    // thread, in one WASM instance) and refusing `Runnable` beside it also
    // refused the lambda target every callback example uses.
    "Runnable",
    // Modelled, and reachable unqualified because `java.lang` is imported on
    // its own — but ABSENT here, so the redundant single import a program is
    // entitled to write (`import java.lang.InterruptedException;`) was
    // "cannot find symbol: class InterruptedException in package java.lang",
    // about a class that works one line later. The list is what the package
    // OFFERS, not what a program has to import to use.
    "Class",
    "Enum",
    "Cloneable",
    "AutoCloseable",
    "StackTraceElement",
    "AssertionError",
    "ArrayStoreException",
    "UnsupportedOperationException",
    "ClassNotFoundException",
    "CloneNotSupportedException",
    "InterruptedException",
    "IllegalAccessException",
    "InstantiationException",
    "NoSuchFieldException",
    "NoSuchMethodException",
    "ReflectiveOperationException",
    "ExceptionInInitializerError",
    "LinkageError",
    "NoClassDefFoundError",
    "OutOfMemoryError",
    "VerifyError",
    "VirtualMachineError",
];

/// Real Java classes students may reach for that caturra doesn't
/// implement — named so the message is honest instead of a misleading
/// "cannot find symbol".
const KNOWN_UNSUPPORTED: &[(&str, &[&str])] = &[
    // The `Abstract*` skeletons: extending one means inheriting a dozen
    // concrete methods written in terms of the two the subclass supplies
    // (`get`/`size`), which caturra's builtin collections do not model.
    // Refusing them BY NAME at least says so, instead of claiming a real
    // java.util class does not exist.
    (
        "java.util",
        &[
            "AbstractList",
            "AbstractCollection",
            "AbstractSet",
            "AbstractSequentialList",
        ],
    ),
    // The old date-and-time classes. `java.time` replaced all of them, and
    // caturra models the arithmetic slice of it — but these five are shaped
    // around the DEFAULT TIME ZONE, and `new Date().toString()` prints a
    // zone abbreviation with daylight saving applied. Answering that needs a
    // real timezone database, which caturra deliberately does not vendor (see
    // specs/SCOPE.md), so they are refused by name rather than approximated.
    (
        "java.util",
        &[
            "Date",
            "Calendar",
            "GregorianCalendar",
            "TimeZone",
            "SimpleTimeZone",
        ],
    ),
    (
        "java.text",
        &["SimpleDateFormat", "DateFormat", "DateFormatSymbols"],
    ),
    // `PrimitiveIterator` (and its `OfInt`/`OfLong`/`OfDouble` members) is what
    // a PRIMITIVE stream's `iterator()` answers. The object streams' cursor is
    // modelled; this one is not, and a program that names the type deserves to
    // be told that rather than "package PrimitiveIterator does not exist".
    (
        "java.util",
        &["PrimitiveIterator", "Spliterator", "Spliterators"],
    ),
    // `Runtime` reports free/total/max memory and runs external processes.
    // caturra collects on its own schedule inside one WASM instance, so every
    // number it could answer would be fiction about a heap the program cannot
    // influence — and `exec` has nothing to exec.
    ("java.lang", &["Runtime", "Process", "ProcessBuilder"]),
    (
        "java.io",
        // `Reader`, `FileWriter` and `Writer` used to sit here: a class
        // listed in both tables resolves fine written under a wildcard import
        // and is refused by its own single import, which is the same fact
        // answered two ways.
        &["Serializable", "InputStream"],
    ),
    // The rest of `java.lang`: the JVM's own errors (a program can CATCH one,
    // and naming it should say what it is), the class-loading and module
    // machinery, the security manager, and the thread-group half of the
    // threading model — plus the five standard ANNOTATION types, which are
    // interfaces a program may name as a type even though it never does. They
    // are listed for the TYPE position only: `@Override` is parsed as an
    // annotation and never reaches this table.
    (
        "java.lang",
        &[
            "Override",
            "Deprecated",
            "SafeVarargs",
            "FunctionalInterface",
            "SuppressWarnings",
            "AbstractMethodError",
            "Appendable",
            "BootstrapMethodError",
            "ClassCircularityError",
            "ClassFormatError",
            "ClassLoader",
            "ClassValue",
            "Compiler",
            "EnumConstantNotPresentException",
            "IllegalAccessError",
            "IllegalCallerException",
            "IllegalMonitorStateException",
            "IllegalThreadStateException",
            "IncompatibleClassChangeError",
            "InheritableThreadLocal",
            "InstantiationError",
            "InternalError",
            "LayerInstantiationException",
            "Module",
            "ModuleLayer",
            "NoSuchFieldError",
            "NoSuchMethodError",
            "Package",
            "ProcessHandle",
            "Readable",
            "RuntimePermission",
            "SecurityException",
            "SecurityManager",
            "StackWalker",
            "ThreadDeath",
            "ThreadGroup",
            "TypeNotPresentException",
            "UnknownError",
            "UnsatisfiedLinkError",
            "UnsupportedClassVersionError",
            "Void",
        ],
    ),
    // The rest of `java.lang.reflect`. caturra models the READ-ONLY surface a
    // program uses to look at itself — `getClass`, `getDeclaredFields`,
    // `Method`, `Constructor`, `Modifier` — and none of the machinery that
    // CHANGES anything (`Proxy`, `AccessibleObject`, `Array`) or describes a
    // generic signature.
    (
        "java.lang.reflect",
        &[
            "AccessibleObject",
            "AnnotatedArrayType",
            "AnnotatedElement",
            "AnnotatedParameterizedType",
            "AnnotatedType",
            "AnnotatedTypeVariable",
            "AnnotatedWildcardType",
            "Array",
            "Executable",
            "GenericArrayType",
            "GenericDeclaration",
            "GenericSignatureFormatError",
            "InaccessibleObjectException",
            "InvocationHandler",
            "MalformedParameterizedTypeException",
            "MalformedParametersException",
            "Member",
            "Parameter",
            "Proxy",
            "ReflectPermission",
            "TypeVariable",
            "UndeclaredThrowableException",
            "WildcardType",
        ],
    ),
    // The rest of `java.io`. caturra models the READER/WRITER side a program
    // reads text with, and the streams `System.out`/`System.in` already are —
    // not the byte-stream hierarchy, object serialization, pipes, or the
    // random-access file. Each is real Java, so each says so by name.
    (
        "java.io",
        &[
            "BufferedInputStream",
            "BufferedOutputStream",
            "ByteArrayInputStream",
            "CharArrayReader",
            "CharArrayWriter",
            "CharConversionException",
            "Console",
            "DataInput",
            "DataInputStream",
            "DataOutput",
            "DataOutputStream",
            "EOFException",
            "Externalizable",
            "FileDescriptor",
            "FileFilter",
            "FileInputStream",
            "FileOutputStream",
            "FilePermission",
            "FilenameFilter",
            "FilterInputStream",
            "FilterOutputStream",
            "FilterReader",
            "FilterWriter",
            "Flushable",
            "IOError",
            "InterruptedIOException",
            "InvalidClassException",
            "InvalidObjectException",
            "LineNumberInputStream",
            "LineNumberReader",
            "NotActiveException",
            "NotSerializableException",
            "ObjectInput",
            "ObjectInputFilter",
            "ObjectInputStream",
            "ObjectInputValidation",
            "ObjectOutput",
            "ObjectOutputStream",
            "ObjectStreamClass",
            "ObjectStreamConstants",
            "ObjectStreamException",
            "ObjectStreamField",
            "OptionalDataException",
            "OutputStreamWriter",
            "PipedInputStream",
            "PipedOutputStream",
            "PipedReader",
            "PipedWriter",
            "PushbackInputStream",
            "PushbackReader",
            "RandomAccessFile",
            "SequenceInputStream",
            "SerializablePermission",
            "StreamCorruptedException",
            "StreamTokenizer",
            "StringBufferInputStream",
            "SyncFailedException",
            "UTFDataFormatException",
            "WriteAbortedException",
        ],
    ),
    // The rest of `java.util`. `Properties`, `ResourceBundle` and `Currency`
    // are locale and configuration machinery caturra has no host for; the
    // format-exception family belongs to a `Formatter` object caturra does not
    // expose (the exceptions themselves are THROWN, by their real names, from
    // `String.format`); `Timer`, `Observable` and `ServiceLoader` need a
    // runtime this one is not.
    (
        "java.util",
        &[
            "AbstractQueue",
            "Currency",
            "Dictionary",
            "EventListener",
            "EventListenerProxy",
            "EventObject",
            "Formattable",
            "FormattableFlags",
            "Formatter",
            "FormatterClosedException",
            "IdentityHashMap",
            "IllformedLocaleException",
            "InvalidPropertiesFormatException",
            "ListResourceBundle",
            "MissingResourceException",
            "Observable",
            "Observer",
            "Properties",
            "PropertyPermission",
            "PropertyResourceBundle",
            "ResourceBundle",
            "ServiceConfigurationError",
            "ServiceLoader",
            "SplittableRandom",
            "Timer",
            "TimerTask",
            "TooManyListenersException",
            "UnknownFormatFlagsException",
            "WeakHashMap",
        ],
    ),
    // `BaseStream` is the interface `Stream`/`IntStream` share, and
    // `StreamSupport` builds one from a `Spliterator` — which caturra does not
    // model either.
    ("java.util.stream", &["BaseStream", "StreamSupport"]),
    // `java.nio.file` past the three caturra models (`Files`, `Path`,
    // `Paths`): the whole rest of the package, recorded from a JDK 11 module
    // image. Every other partly-modelled package was on this list already, so
    // these forty-one alone were told "cannot find symbol" — javac's wording
    // for a class that does not exist, about forty-one that do.
    (
        "java.nio.file",
        &[
            "AccessDeniedException",
            "AccessMode",
            "AtomicMoveNotSupportedException",
            "ClosedDirectoryStreamException",
            "ClosedFileSystemException",
            "ClosedWatchServiceException",
            "CopyOption",
            "DirectoryIteratorException",
            "DirectoryNotEmptyException",
            "DirectoryStream",
            "FileAlreadyExistsException",
            "FileStore",
            "FileSystem",
            "FileSystemAlreadyExistsException",
            "FileSystemException",
            "FileSystemLoopException",
            "FileSystemNotFoundException",
            "FileSystems",
            "FileVisitOption",
            "FileVisitResult",
            "FileVisitor",
            "InvalidPathException",
            "LinkOption",
            "LinkPermission",
            "NoSuchFileException",
            "NotLinkException",
            "OpenOption",
            "PathMatcher",
            "ProviderMismatchException",
            "ProviderNotFoundException",
            "ReadOnlyFileSystemException",
            "SecureDirectoryStream",
            "SimpleFileVisitor",
            "StandardCopyOption",
            "StandardOpenOption",
            "StandardWatchEventKinds",
            "WatchEvent",
            "WatchKey",
            "WatchService",
            "Watchable",
        ],
    ),
    // The rest of `java.text`. `DecimalFormat`/`NumberFormat` are modelled;
    // collation, bidi, break iteration and message formatting each need the
    // locale data caturra deliberately does not vendor.
    (
        "java.text",
        &[
            "Annotation",
            "AttributedCharacterIterator",
            "AttributedString",
            "Bidi",
            "BreakIterator",
            "CharacterIterator",
            "ChoiceFormat",
            "CollationElementIterator",
            "CollationKey",
            "Collator",
            "DecimalFormatSymbols",
            "Format",
            "MessageFormat",
            "Normalizer",
            "RuleBasedCollator",
            "StringCharacterIterator",
        ],
    ),
    // The rest of `java.time`: every one of these carries an INSTANT or a ZONE.
    // caturra models the arithmetic slice — the local dates and times, which
    // are pure arithmetic — and answering a zone honestly needs a timezone
    // database it does not vendor (see specs/SCOPE.md).
    (
        "java.time",
        &[
            "Clock",
            "Instant",
            "OffsetDateTime",
            "OffsetTime",
            "ZoneId",
            "ZoneOffset",
            "ZonedDateTime",
        ],
    ),
    // `DateTimeFormatter` is modelled from a PATTERN; the builder, the resolver
    // style and the sign/decimal styles that shape one are not.
    (
        "java.time.format",
        &[
            "DateTimeFormatterBuilder",
            "DecimalStyle",
            "ResolverStyle",
            "SignStyle",
        ],
    ),
    // `ChronoField`, `ChronoUnit` and `TemporalAdjusters` are modelled — the
    // INTERFACES behind them are not, so a program cannot write its own
    // `TemporalField` or query.
    (
        "java.time.temporal",
        &[
            "IsoFields",
            "JulianFields",
            "Temporal",
            "TemporalAccessor",
            "TemporalAmount",
            "TemporalField",
            "TemporalUnit",
            "WeekFields",
        ],
    ),
    // The six standard charsets encode and decode exactly as a JDK's; the
    // incremental coder objects behind them, and the exceptions only they
    // throw, are not modelled.
    (
        "java.nio.charset",
        &[
            "CharacterCodingException",
            "CharsetDecoder",
            "CharsetEncoder",
            "CoderMalfunctionError",
            "CoderResult",
            "CodingErrorAction",
            "MalformedInputException",
            "UnmappableCharacterException",
        ],
    ),
    // `Thread` and friends are real Java that caturra will not be growing: a
    // program here runs on one thread, in one WASM instance. Saying "unknown type
    // 'Thread'" about a class every Java programmer knows reads as our bug.
    (
        "java.lang",
        &[
            "StrictMath",
            "Thread",
            "ThreadLocal",
            "Process",
            "ProcessBuilder",
        ],
    ),
];

/// Every real JDK package caturra does not model, recorded from a Java 11
/// module image. A wildcard import of one, or a qualified name inside it,
/// gets "package java.security is not supported by caturra" — where "package
/// java.security does not exist" is a false statement about the JDK, and reads
/// as a typo for a package the student can see in the documentation.
///
/// The list is exhaustive rather than the four names it used to hold, because
/// the four were whichever ones somebody had happened to hit. A package NOT
/// here and not modelled really does not exist, and still says so.
const KNOWN_UNSUPPORTED_PACKAGES: &[&str] = &[
    "java.applet",
    "java.awt.color",
    "java.awt.datatransfer",
    "java.awt.desktop",
    "java.awt.dnd",
    "java.awt.dnd.peer",
    "java.awt.font",
    "java.awt.geom",
    "java.awt.im",
    "java.awt.im.spi",
    "java.awt.image",
    "java.awt.image.renderable",
    "java.awt.peer",
    "java.awt.print",
    "java.beans",
    "java.beans.beancontext",
    "java.lang.annotation",
    "java.lang.instrument",
    "java.lang.invoke",
    "java.lang.management",
    "java.lang.module",
    "java.lang.ref",
    "java.net",
    "java.net.http",
    "java.net.spi",
    "java.nio",
    "java.nio.channels",
    "java.nio.channels.spi",
    "java.nio.charset.spi",
    "java.nio.file.attribute",
    "java.nio.file.spi",
    "java.rmi",
    "java.rmi.activation",
    "java.rmi.dgc",
    "java.rmi.registry",
    "java.rmi.server",
    "java.security",
    "java.security.acl",
    "java.security.cert",
    "java.security.interfaces",
    "java.security.spec",
    "java.sql",
    "java.text.spi",
    "java.time.zone",
    "java.util.concurrent",
    "java.util.concurrent.atomic",
    "java.util.concurrent.locks",
    "java.util.jar",
    "java.util.logging",
    "java.util.prefs",
    "java.util.spi",
    "java.util.zip",
    "javax.annotation.processing",
    "javax.crypto",
    "javax.crypto.interfaces",
    "javax.crypto.spec",
    "javax.imageio",
    "javax.imageio.event",
    "javax.imageio.metadata",
    "javax.imageio.plugins.bmp",
    "javax.imageio.plugins.jpeg",
    "javax.imageio.plugins.tiff",
    "javax.imageio.spi",
    "javax.imageio.stream",
    "javax.lang.model",
    "javax.lang.model.element",
    "javax.lang.model.type",
    "javax.lang.model.util",
    "javax.management",
    "javax.management.loading",
    "javax.management.modelmbean",
    "javax.management.monitor",
    "javax.management.openmbean",
    "javax.management.relation",
    "javax.management.remote",
    "javax.management.remote.rmi",
    "javax.management.timer",
    "javax.naming",
    "javax.naming.directory",
    "javax.naming.event",
    "javax.naming.ldap",
    "javax.naming.spi",
    "javax.net",
    "javax.net.ssl",
    "javax.print",
    "javax.print.attribute",
    "javax.print.attribute.standard",
    "javax.print.event",
    "javax.rmi.ssl",
    "javax.script",
    "javax.security.auth",
    "javax.security.auth.callback",
    "javax.security.auth.kerberos",
    "javax.security.auth.login",
    "javax.security.auth.spi",
    "javax.security.auth.x500",
    "javax.security.cert",
    "javax.security.sasl",
    "javax.smartcardio",
    "javax.sound.midi",
    "javax.sound.midi.spi",
    "javax.sound.sampled",
    "javax.sound.sampled.spi",
    "javax.sql",
    "javax.sql.rowset",
    "javax.sql.rowset.serial",
    "javax.sql.rowset.spi",
    "javax.swing.colorchooser",
    "javax.swing.filechooser",
    "javax.swing.plaf",
    "javax.swing.plaf.basic",
    "javax.swing.plaf.metal",
    "javax.swing.plaf.multi",
    "javax.swing.plaf.nimbus",
    "javax.swing.plaf.synth",
    "javax.swing.text.html",
    "javax.swing.text.html.parser",
    "javax.swing.text.rtf",
    "javax.swing.undo",
    "javax.tools",
    "javax.transaction.xa",
    "javax.xml",
    "javax.xml.catalog",
    "javax.xml.crypto",
    "javax.xml.crypto.dom",
    "javax.xml.crypto.dsig",
    "javax.xml.crypto.dsig.dom",
    "javax.xml.crypto.dsig.keyinfo",
    "javax.xml.crypto.dsig.spec",
    "javax.xml.datatype",
    "javax.xml.namespace",
    "javax.xml.parsers",
    "javax.xml.stream",
    "javax.xml.stream.events",
    "javax.xml.stream.util",
    "javax.xml.transform",
    "javax.xml.transform.dom",
    "javax.xml.transform.sax",
    "javax.xml.transform.stax",
    "javax.xml.transform.stream",
    "javax.xml.validation",
    "javax.xml.xpath",
];

/// Library type names whose use requires an import.
const REQUIRES_IMPORT: &[&str] = &[
    "PatternSyntaxException",
    "Scanner",
    "ArrayList",
    "List",
    "HashMap",
    "Map",
    "TreeMap",
    "SortedMap",
    "NavigableMap",
    "Set",
    "HashSet",
    "TreeSet",
    "SortedSet",
    "NavigableSet",
    "EnumMap",
    "EnumSet",
    // A JTable's model and a JTree's nodes live in `javax.swing.table` and
    // `javax.swing.tree`, NOT in `javax.swing`: naming one needs its own
    // import, and `import javax.swing.*` alone does not provide it.
    "TableModel",
    "AbstractTableModel",
    "DefaultTableModel",
    "TableCellRenderer",
    "DefaultTableCellRenderer",
    "TableColumn",
    "TableColumnModel",
    "TreeModel",
    "DefaultTreeModel",
    "TreeCellRenderer",
    "DefaultTreeCellRenderer",
    "DefaultMutableTreeNode",
    "TreePath",
    "LinkedList",
    "Queue",
    "Deque",
    "PriorityQueue",
    "ArrayDeque",
    "Stack",
    "Vector",
    "Hashtable",
    "Enumeration",
    "RandomAccess",
    "Collection",
    "Comparator",
    "Iterator",
    "ListIterator",
    "Function",
    "BiFunction",
    "UnaryOperator",
    "BinaryOperator",
    "Predicate",
    "Consumer",
    "BiConsumer",
    "Supplier",
    "Collectors",
    "Stream",
    "File",
    "PrintWriter",
    "PrintStream",
    "ByteArrayOutputStream",
    "OutputStream",
    "BufferedReader",
    "FileReader",
    "InputStreamReader",
    "Files",
    "Path",
    "Paths",
    "InputMismatchException",
    "NoSuchElementException",
    "IOException",
    "FileNotFoundException",
    // Every library name that is not in `java.lang` needs an import — the
    // rule javac enforces, and the one caturra let through: `Arrays.sort(x)`
    // with no `import java.util.Arrays;` compiled here and fails on a JDK,
    // which is the direction that must never be wrong. The bundled libraries
    // (swing, org.code) are gated by INJECTION instead: without the import
    // their classes do not exist at all.
    "AbstractMap",
    "LinkedHashMap",
    "LinkedHashSet",
    "Optional",
    "OptionalInt",
    "OptionalDouble",
    "OptionalLong",
    "Arrays",
    "Objects",
    "Random",
    "Collections",
    "StringJoiner",
    "EmptyStackException",
    "ConcurrentModificationException",
    "IllegalFormatException",
    "UnknownFormatConversionException",
    "MissingFormatArgumentException",
    "IllegalFormatConversionException",
    "IllegalFormatCodePointException",
    "Locale",
    "IntStream",
    "LongStream",
    "DoubleStream",
    "Collector",
    "Pattern",
    "Matcher",
    "MatchResult",
    "Closeable",
    "Charset",
    "StandardCharsets",
    "UnsupportedCharsetException",
    "IllegalCharsetNameException",
    "Method",
    "Field",
    "Constructor",
    "Modifier",
    "InvocationTargetException",
    // The primitive-specialized functional interfaces, for the same reason.
    "BiPredicate",
    "IntFunction",
    "IntPredicate",
    "IntSupplier",
    "IntConsumer",
    "IntUnaryOperator",
    "IntBinaryOperator",
    "DoublePredicate",
    "DoubleSupplier",
    "DoubleConsumer",
    "DoubleUnaryOperator",
    "DoubleBinaryOperator",
    "LongPredicate",
    "LongSupplier",
    "LongConsumer",
    "LongUnaryOperator",
    "LongBinaryOperator",
    "BooleanSupplier",
    "ToIntFunction",
    "ToDoubleFunction",
    "ToLongFunction",
    "DoubleFunction",
    "LongFunction",
    "IntToLongFunction",
    "IntToDoubleFunction",
    "LongToIntFunction",
    "LongToDoubleFunction",
    "DoubleToIntFunction",
    "DoubleToLongFunction",
    "ObjIntConsumer",
    "ObjLongConsumer",
    "ObjDoubleConsumer",
    "ToIntBiFunction",
    "ToLongBiFunction",
    "ToDoubleBiFunction",
    // The course libraries need an import exactly as `java.util` does. They
    // were exempt, and the exemption only shows with several FILES: the
    // bundle an `import org.code.neighborhood.Painter` injects is global (one
    // class table), so a SECOND file could write `extends Painter` with no
    // import of its own and compile. javac scopes an import to its own
    // compilation unit, and one of the corpus's own levels is written that
    // way.
    "LocalDate",
    "LocalTime",
    "LocalDateTime",
    "Duration",
    "Period",
    "ChronoUnit",
    "ChronoField",
    "ValueRange",
    "TemporalAdjusters",
    "TemporalQueries",
    "TemporalQuery",
    "IsoEra",
    "IsoChronology",
    "DateTimeFormatter",
    "DayOfWeek",
    "Month",
    "Painter",
    "Scene",
    "Theater",
    "Instrument",
    "Image",
    "Pixel",
    "Font",
    "FontStyle",
    "SoundLoader",
    "NeighborhoodTestRunner",
    "SystemOutTestRunner",
    "NeighborhoodLog",
    "PainterLog",
    "PainterEvent",
    "Position",
    "NeighborhoodActionType",
    "ValidationHelper",
];

/// The nested library types the compiler models, as (enclosing simple name,
/// nested name, the two-part name the compiler uses). `Map.Entry` is the only
/// one — but a qualified `java.util.Map.Entry` has to reach it.
const NESTED_LIBRARY_CLASSES: &[(&str, &str, &str)] = &[
    ("Map", "Entry", "Map.Entry"),
    // caturra models an `AbstractMap.SimpleEntry` AS a `Map.Entry` (an entry
    // over a hidden one-mapping map), so the name resolves to that type. It
    // could be CONSTRUCTED and not NAMED: `new AbstractMap.SimpleEntry<>(k, v)`
    // compiled while `AbstractMap.SimpleEntry<K, V> e = …` was "package
    // AbstractMap does not exist", about a package that is a class.
    ("AbstractMap", "SimpleEntry", "Map.Entry"),
    // `Base64` is only a namespace for its two coders, and a program that keeps
    // one in a field has to be able to NAME it.
    ("Base64", "Encoder", "Base64.Encoder"),
    ("Base64", "Decoder", "Base64.Decoder"),
];

/// Resolve a fully qualified library name (`java.util.Scanner`) to the
/// simple name the compiler models. Fully qualified uses never need an
/// import — that is their purpose in Java.
///
/// The modeled classes are exactly `package_classes`'. This kept a second,
/// hand-maintained list of the packages that hold them, and it drifted
/// twice: first `java.util.Arrays.fill(...)` did not resolve though
/// `java.lang.Math.abs(...)` did, then `java.util.stream.Stream<String> s`
/// was refused as unsupported though `import java.util.stream.*` worked.
/// Asking `package_classes` directly is what stops it drifting a third time.
/// A NESTED library type by the way source spells it — `Map.Entry`, and the
/// qualified `java.util.Map.Entry`. [`canonical_library_class`] only resolves
/// the qualified form: it splits at the last dot and asks the PACKAGE table,
/// so a bare `Map.Entry` asks about a package called `Map` and gets nothing.
pub(crate) fn nested_library_class(name: &str) -> Option<&'static str> {
    if let Some(canonical) = canonical_library_class(name) {
        return Some(canonical);
    }
    let (outer, nested) = name.rsplit_once('.')?;
    let outer = canonical_library_class(outer).unwrap_or(outer);
    NESTED_LIBRARY_CLASSES
        .iter()
        .find(|(enclosing, inner, _)| *enclosing == outer && *inner == nested)
        .map(|(_, _, canonical)| *canonical)
}

pub(crate) fn canonical_library_class(dotted: &str) -> Option<&'static str> {
    let (package, class) = dotted.rsplit_once('.')?;
    if let Some(known) = package_classes(package) {
        return known.iter().find(|name| **name == class).copied();
    }
    // `java.util.Map.Entry` — a qualified use of a nested library type. What
    // precedes the last dot is the enclosing class rather than a package.
    let outer = canonical_library_class(package)?;
    NESTED_LIBRARY_CLASSES
        .iter()
        .find(|(enclosing, nested, _)| *enclosing == outer && *nested == class)
        .map(|(_, _, canonical)| *canonical)
}

/// The package-qualified INTERNAL name of a library class, by its simple name
/// (`ArrayList` -> `java/util/ArrayList`).
///
/// `Class.getName()` is fully qualified, and a class literal that carried only
/// the simple name reported `Math` where a JDK reports `java.lang.Math`.
/// `java.lang` is searched first, so a simple name that exists in both
/// packages resolves the way an unqualified source reference would.
/// Every library class caturra models, by its simple name. One list, so a
/// class added to a package list is a class every pass knows about.
pub(crate) fn library_class_names() -> impl Iterator<Item = &'static str> {
    PACKAGES
        .iter()
        .flat_map(|(_, classes)| classes.iter().copied())
}

pub(crate) fn qualified_library_class(simple: &str) -> Option<String> {
    let mut packages: Vec<&(&str, &[&str])> = PACKAGES.iter().collect();
    packages.sort_by_key(|(package, _)| *package != "java.lang");
    packages.iter().find_map(|(package, classes)| {
        classes
            .contains(&simple)
            .then(|| format!("{package}.{simple}").replace('.', "/"))
    })
}

/// The honest reason a class caturra models only as a namespace for static
/// members cannot name a variable: `Math m;`. javac accepts that declaration
/// — `Math` is an ordinary class type — so caturra is stricter here, and has
/// to say why rather than call a class everyone has used an unknown type.
///
/// Only the packages whose classes caturra models natively: a bundled
/// `Painter` is compiled as an ordinary class and names a variable fine.
///
/// Only sound once the name has failed to resolve as a type, which is the one
/// place it is asked — a user class named `Math` shadows the library one, and
/// would have resolved.
pub(crate) fn unusable_library_type_reason(simple: &str) -> Option<String> {
    PACKAGES
        .iter()
        .filter(|(package, _)| package.starts_with("java."))
        .find(|(_, classes)| classes.contains(&simple))
        .map(|(package, _)| {
            // NOT `not_supported`: these classes ARE supported — `Math.abs`,
            // `Collectors.toList`, `TextStyle.SHORT` all work — and saying
            // "java.lang.Math is not supported by caturra" about the class
            // every program has used is a false statement, in the one place a
            // student would read it as authoritative. What is missing is a
            // VALUE of the type, which is all a variable could hold.
            format!(
                "{package}.{simple} cannot name a variable in caturra: its members work \
                 (written out, as {simple}.…), but no value of the type is modelled"
            )
        })
}

/// Nested types of classes caturra DOES model, and does not model itself.
///
/// A program that names one has to be told about the NESTED name: the
/// enclosing class works, so the "cannot name a variable" message about it
/// would be about the wrong thing — and "cannot find symbol" would claim a
/// real class does not exist.
const UNSUPPORTED_NESTED: &[(&str, &str, &str)] = &[
    // The attribute a `FieldPosition` can carry instead of a number, and what
    // `formatToCharacterIterator` reports with. caturra models the two
    // numbered fields (`INTEGER_FIELD`, `FRACTION_FIELD`) and no attribute.
    ("NumberFormat", "Field", "java.text.NumberFormat.Field"),
    ("DecimalFormat", "Field", "java.text.NumberFormat.Field"),
];

/// The honest reason a nested type of a modelled class cannot be named.
pub(crate) fn unsupported_nested_reason(outer: &str, nested: &str) -> Option<String> {
    UNSUPPORTED_NESTED
        .iter()
        .find(|(enclosing, name, _)| *enclosing == outer && *name == nested)
        .map(|(_, _, dotted)| not_supported(dotted))
}

/// The honest reason a real Java 11 class caturra does not model cannot
/// be used, from its simple name: `LinkedList` → "java.util.LinkedList is
/// not supported by caturra ...". `None` when the name is not one of them.
///
/// Only sound once the name has failed to resolve — a user class called
/// `Stack` shadows the library one, and would have resolved.
pub(crate) fn unsupported_class_reason(simple: &str) -> Option<String> {
    KNOWN_UNSUPPORTED
        .iter()
        .find(|(_, names)| names.contains(&simple))
        .map(|(package, _)| not_supported(&format!("{package}.{simple}")))
}

/// javac-style message for a fully qualified name the library doesn't
/// model (honest "not supported" for real Java classes and packages).
pub(crate) fn unknown_qualified_message(dotted: &str) -> String {
    let Some((package, class)) = dotted.rsplit_once('.') else {
        return format!("unknown type '{dotted}'");
    };
    // A NESTED type of a class caturra does not model — `PrimitiveIterator.OfInt`,
    // which is what a primitive stream's `iterator()` answers. The qualifier
    // reads as a package here, and "package PrimitiveIterator does not exist"
    // is a wrong answer about a type `java.util` really has.
    if let Some(reason) = unsupported_class_reason(package) {
        return reason;
    }
    if package_classes(package).is_some() {
        if KNOWN_UNSUPPORTED
            .iter()
            .any(|(pkg, names)| *pkg == package && names.contains(&class))
            || package_classes(package).is_some_and(|names| names.contains(&class))
        {
            // Real (or modeled-but-not-usable-here) Java class.
            return not_supported(dotted);
        }
        return format!("cannot find symbol: class {class} in package {package}");
    }
    if KNOWN_UNSUPPORTED_PACKAGES.contains(&package) {
        return not_supported(&format!("package {package}"));
    }
    format!("package {package} does not exist")
}

/// Every package the compiler models, and the classes in it. One table, read
/// by `package_classes` and by every reverse lookup: a second, hand-maintained
/// list beside this one has drifted twice already.
///
/// The `org.code.*`, `javax.swing.*` and `java.awt.*` entries are the bundled
/// clean-room library (auto-injected in `compile`); those classes resolve like
/// user classes, and the import just validates.
static PACKAGES: &[(&str, &[&str])] = &[
    ("java.util", JAVA_UTIL),
    ("java.util.stream", JAVA_UTIL_STREAM),
    ("java.util.regex", JAVA_UTIL_REGEX),
    ("java.util.function", JAVA_UTIL_FUNCTION),
    ("java.io", JAVA_IO),
    ("java.nio.file", JAVA_NIO_FILE),
    ("java.time", JAVA_TIME),
    ("java.time.format", JAVA_TIME_FORMAT),
    ("java.time.temporal", JAVA_TIME_TEMPORAL),
    ("java.time.chrono", JAVA_TIME_CHRONO),
    ("java.nio.charset", JAVA_NIO_CHARSET),
    ("java.math", JAVA_MATH),
    ("java.text", JAVA_TEXT),
    ("java.lang", JAVA_LANG),
    ("java.lang.reflect", JAVA_LANG_REFLECT),
    ("org.code.neighborhood", ORG_CODE_NEIGHBORHOOD),
    ("org.code.validation", ORG_CODE_VALIDATION),
    ("org.code.theater", ORG_CODE_THEATER),
    ("org.code.media", ORG_CODE_MEDIA),
    ("javax.swing", JAVAX_SWING),
    ("javax.swing.event", JAVAX_SWING_EVENT),
    ("javax.swing.border", JAVAX_SWING_BORDER),
    ("javax.swing.text", JAVAX_SWING_TEXT),
    ("javax.swing.table", JAVAX_SWING_TABLE),
    ("javax.swing.tree", JAVAX_SWING_TREE),
    ("javax.accessibility", JAVAX_ACCESSIBILITY),
    ("java.awt", JAVA_AWT),
    ("java.awt.event", JAVA_AWT_EVENT),
];

fn package_classes(package: &str) -> Option<&'static [&'static str]> {
    PACKAGES
        .iter()
        .find(|(name, _)| *name == package)
        .map(|(_, classes)| *classes)
}

/// The public class of the bundled neighborhood library.
/// The structural reflection caturra models — what a grading harness inspects
/// a student's class with. Listing the package here (rather than only waving
/// its IMPORT through) is what lets a QUALIFIED `java.lang.reflect.Method`
/// resolve, and gives these classes their fully qualified `getName()`.
static JAVA_LANG_REFLECT: &[&str] = &[
    "Method",
    "Field",
    "Constructor",
    "Modifier",
    "InvocationTargetException",
];

static ORG_CODE_NEIGHBORHOOD: &[&str] = &["Painter"];

/// Public classes of the bundled validation library (neighborhood harness).
static ORG_CODE_VALIDATION: &[&str] = &[
    "NeighborhoodTestRunner",
    "SystemOutTestRunner",
    "NeighborhoodLog",
    "PainterLog",
    "PainterEvent",
    "Position",
    "NeighborhoodActionType",
    "ValidationHelper",
];

/// Public classes of the bundled accessible-DOM Swing library. The widgets
/// resolve like user classes (auto-injected in `compile`); the import just
/// validates and is what gates the injection.
static JAVAX_SWING: &[&str] = &[
    "JFrame",
    "JPanel",
    "JScrollPane",
    "JTabbedPane",
    "JSplitPane",
    "JToolBar",
    "JLabel",
    "JButton",
    "JToggleButton",
    "BoxLayout",
    "Box",
    "JTextField",
    "JPasswordField",
    "JTextArea",
    "JCheckBox",
    "JRadioButton",
    "JComboBox",
    "ComboBoxModel",
    "MutableComboBoxModel",
    "DefaultComboBoxModel",
    "JList",
    "ListSelectionModel",
    "ListModel",
    "AbstractListModel",
    "DefaultListModel",
    "ListCellRenderer",
    "DefaultListCellRenderer",
    "JTree",
    "JTable",
    "JProgressBar",
    "JSpinner",
    "SpinnerNumberModel",
    "JSlider",
    "ButtonGroup",
    "Timer",
    "JOptionPane",
    "JMenuBar",
    "JMenu",
    "JMenuItem",
    "JCheckBoxMenuItem",
    "JRadioButtonMenuItem",
    "BorderFactory",
    "Border",
    "SwingUtilities",
    "SwingConstants",
    "KeyStroke",
    "Action",
    "AbstractAction",
];
/// `javax.accessibility`: the `AccessibleContext` handle (getAccessibleContext).
static JAVAX_ACCESSIBILITY: &[&str] = &["AccessibleContext"];

/// `javax.swing.border`: the `Border` handle returned by `BorderFactory` plus
/// the concrete subtypes (constructible directly, e.g. `new LineBorder(...)`).
static JAVAX_SWING_BORDER: &[&str] = &[
    "Border",
    "LineBorder",
    "EmptyBorder",
    "MatteBorder",
    "EtchedBorder",
    "BevelBorder",
    "TitledBorder",
    "CompoundBorder",
];
static JAVA_AWT: &[&str] = &[
    "Color",
    "Dimension",
    "Font",
    "Graphics",
    "Graphics2D",
    "BasicStroke",
    "LayoutManager",
    "FlowLayout",
    "GridLayout",
    "BorderLayout",
    "GridBagLayout",
    "GridBagConstraints",
    "Insets",
    "Container",
    "Component",
];
/// java.awt.event (listeners): the functional interfaces students attach
/// (addActionListener / addItemListener), the mouse listener/adapter, plus
/// their event objects.
static JAVA_AWT_EVENT: &[&str] = &[
    "ActionListener",
    "ActionEvent",
    "ItemListener",
    "ItemEvent",
    "MouseListener",
    "MouseMotionListener",
    "MouseAdapter",
    "MouseEvent",
    "KeyListener",
    "KeyAdapter",
    "KeyEvent",
    "InputEvent",
];
/// `javax.swing.event`: the `ChangeListener` (`JSlider`) and
/// `ListSelectionListener` (`JList`) plus their events.
static JAVAX_SWING_EVENT: &[&str] = &[
    "ChangeListener",
    "ChangeEvent",
    "ListSelectionListener",
    "ListSelectionEvent",
    "ListDataListener",
    "ListDataEvent",
    "TableModelListener",
    "TableModelEvent",
    "TreeSelectionListener",
    "TreeSelectionEvent",
    "TreeModelListener",
    "TreeModelEvent",
    "DocumentListener",
    "DocumentEvent",
];

/// `javax.swing.table`: a `JTable`'s MODEL and its renderers. They are not in
/// `javax.swing` — `import javax.swing.*` does not bring them, which is what
/// javac enforces and caturra did not: `DefaultTableModel` resolved off the
/// wildcard alone (a program that compiles here and fails on a JDK), while the
/// import that really provides it was refused as a package that does not exist.
static JAVAX_SWING_TABLE: &[&str] = &[
    "TableModel",
    "AbstractTableModel",
    "DefaultTableModel",
    "TableCellRenderer",
    "DefaultTableCellRenderer",
    "TableColumn",
    "TableColumnModel",
];

/// `javax.swing.tree`: a `JTree`'s model and nodes, for the same reason.
static JAVAX_SWING_TREE: &[&str] = &[
    "TreeModel",
    "DefaultTreeModel",
    "TreeCellRenderer",
    "DefaultTreeCellRenderer",
    "DefaultMutableTreeNode",
    "TreePath",
];

/// `javax.swing.text`: the `Document` handle (getDocument).
static JAVAX_SWING_TEXT: &[&str] = &["Document", "BadLocationException", "JTextComponent"];

/// Public classes of the bundled theater/media library.
static ORG_CODE_THEATER: &[&str] = &["Scene", "Theater", "Instrument"];
static ORG_CODE_MEDIA: &[&str] = &[
    "Image",
    "Pixel",
    "Color",
    "Font",
    "FontStyle",
    "SoundLoader",
];

fn not_supported(what: &str) -> String {
    format!("{what} is not supported by caturra (the class library covers the AP CS A subset)")
}

/// Validate a unit's imports and check that library classes are only
/// used under a matching import. `user_classes` holds every class name
/// defined across the whole compilation (one shared namespace).
pub fn check_unit(
    path: &str,
    unit: &CompilationUnit,
    user_classes: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut error = |message: String, span: SourceSpan| {
        diagnostics.push(Diagnostic::error(path, message, span));
    };

    // Which import-requiring names this unit has enabled.
    let mut enabled: HashSet<&'static str> = HashSet::new();

    for import in &unit.imports {
        validate_import(import, &mut enabled, &mut error);
    }

    for class in &unit.classes {
        check_class(class, user_classes, &enabled, &mut error);
    }
}

fn validate_import(
    import: &ImportDecl,
    enabled: &mut HashSet<&'static str>,
    error: &mut impl FnMut(String, SourceSpan),
) {
    // JUnit (`org.junit.*`) is accepted wholesale for the validation
    // "Test" mode: annotations are retained but never resolved as types,
    // and `Assertions` is provided by the bundled test library. Static
    // imports (`import static ...Assertions.*`) come through here too.
    if import.path.first().map(String::as_str) == Some("org")
        && import.path.get(1).map(String::as_str) == Some("junit")
    {
        return;
    }
    // EasyMock (`import static org.easymock.EasyMock.*`) — partial-mock builder
    // chains are rewritten away; `expect`/`replay`/`verify` resolve to the
    // bundled `EasyMock` like a static import.
    if import.path.first().map(String::as_str) == Some("org")
        && import.path.get(1).map(String::as_str) == Some("easymock")
    {
        return;
    }
    // `java.lang.reflect.*` — the structural reflection subset used by student
    // helpers like `AttributesHelper`. The IMPORT is waved through for the
    // whole package, not just the classes modelled below: a harness that
    // imports `Parameter` and never uses it compiled before this check
    // existed, and an unused import is not where to tell it otherwise. Using
    // an unmodelled one still fails at the USE site, which is where the
    // program actually depends on it.
    if import.path.first().map(String::as_str) == Some("java")
        && import.path.get(1).map(String::as_str) == Some("lang")
        && import.path.get(2).map(String::as_str) == Some("reflect")
    {
        // ...but the names it DOES model are still enabled by it: waving the
        // import through without them left `Field[] fs = …` reported as a
        // missing class under the very import that provides it.
        for name in JAVA_LANG_REFLECT {
            if REQUIRES_IMPORT.contains(name)
                && (import.wildcard || import.path.last() == Some(&(*name).to_owned()))
            {
                enabled.insert(name);
            }
        }
        return;
    }

    // A STATIC import names a MEMBER after the class: `import static
    // java.lang.Math.max` is the class `java.lang.Math` and the member `max`, so
    // the package ends one element earlier than for an ordinary import. Reading it
    // the usual way made the package `java.lang.Math`, and the message was "package
    // java.lang.Math does not exist" — about a class everyone has used.
    // `import java.util.List`      -> package java.util          (drop the class)
    // `import java.util.*`          -> package java.util          (drop nothing)
    // `import static java.lang.Math.max` -> package java.lang     (drop member + class)
    // `import static java.lang.Math.*`   -> package java.lang     (drop the class)
    let drop = usize::from(!import.wildcard) + usize::from(import.is_static);
    let package = import.path[..import.path.len().saturating_sub(drop)].join(".");
    // For a static import the class is what the member hangs off, not the last name.
    let class_index = import
        .path
        .len()
        .saturating_sub(1 + usize::from(import.is_static && !import.wildcard));

    if let Some(classes) = package_classes(&package) {
        if import.wildcard {
            for name in classes {
                if REQUIRES_IMPORT.contains(name) {
                    enabled.insert(name);
                }
            }
            return;
        }
        let class = &import.path[class_index];
        if let Some(name) = classes.iter().find(|n| *n == class) {
            if REQUIRES_IMPORT.contains(name) {
                enabled.insert(name);
            }
            return;
        }
        if KNOWN_UNSUPPORTED
            .iter()
            .any(|(pkg, names)| *pkg == package && names.contains(&class.as_str()))
        {
            error(not_supported(&format!("{package}.{class}")), import.span);
            return;
        }
        // javac: "cannot find symbol — symbol: class X, location:
        // package java.util".
        error(
            format!("cannot find symbol: class {class} in package {package}"),
            import.span,
        );
        return;
    }

    // `import static Helper.twice;` — a static import out of the unnamed package,
    // which javac rejects too ("cannot find symbol"). caturra has only the unnamed
    // package, so this is where a user class lands; refusing it is right, but
    // "package  does not exist" is not the way to say so.
    if package.is_empty() {
        let class = &import.path[class_index];
        error(format!("cannot find symbol: class {class}"), import.span);
        return;
    }
    if KNOWN_UNSUPPORTED_PACKAGES.contains(&package.as_str()) {
        error(not_supported(&format!("package {package}")), import.span);
    } else {
        // javac: "package foo.bar does not exist".
        error(format!("package {package} does not exist"), import.span);
    }
}

// ----- Use-site enforcement -----

struct UseCheck<'a, F: FnMut(String, SourceSpan)> {
    user_classes: &'a HashSet<String>,
    enabled: &'a HashSet<&'static str>,
    location: String,
    error: F,
}

/// A supertype as written, without its type ARGUMENTS: `Comparable<Pet>` is a
/// use of `Comparable`. (The arguments are checked where they are written, as
/// part of the fields and methods that mention them.)
fn supertype_simple_name(written: &str) -> &str {
    written.split('<').next().unwrap_or(written).trim()
}

fn check_class(
    class: &ClassDecl,
    user_classes: &HashSet<String>,
    enabled: &HashSet<&'static str>,
    error: &mut impl FnMut(String, SourceSpan),
) {
    let mut check = UseCheck {
        user_classes,
        enabled,
        // The class being walked — javac's "location:" line names it.
        location: class.name.clone(),
        error,
    };
    // The SUPERTYPES a class names. `class PeopleModel extends
    // AbstractTableModel` needs `javax.swing.table.*` exactly as a field of
    // that type does, and the walk covered fields, parameters, returns and
    // bodies — everything but the one position a class is written in. javac
    // refuses it; a playground demo shipped with the wrong import for months
    // because caturra did not.
    for name in class.superclass.iter().chain(class.interfaces.iter()) {
        check.name(supertype_simple_name(name), class.span);
    }
    for field in &class.fields {
        check.type_ref(&field.ty, field.span);
        if let Some(init) = &field.init {
            check.expr(init);
        }
    }
    for method in &class.methods {
        check.type_ref(&method.return_type, method.span);
        for param in &method.params {
            check.type_ref(&param.ty, method.span);
        }
        for stmt in &method.body {
            check.stmt(stmt);
        }
    }
    for block in &class.init_blocks {
        for stmt in &block.body {
            check.stmt(stmt);
        }
    }
}

impl<F: FnMut(String, SourceSpan)> UseCheck<'_, F> {
    fn name(&mut self, name: &str, span: SourceSpan) {
        if name.contains('.') {
            // Fully qualified: no import needed; codegen validates it.
            return;
        }
        if REQUIRES_IMPORT.contains(&name)
            && !self.user_classes.contains(name)
            && !self.enabled.contains(name)
        {
            // javac's three-line block, which every other "cannot find
            // symbol" here already uses: the headline, what was looked for,
            // and where it was looked for from.
            (self.error)(
                format!(
                    "cannot find symbol\n  symbol:   class {name}\n  location: class {}",
                    self.location
                ),
                span,
            );
        }
    }

    fn type_ref(&mut self, ty: &TypeRef, span: SourceSpan) {
        match ty {
            TypeRef::Named(name) => self.name(name, span),
            TypeRef::Generic { base, args } => {
                self.name(base, span);
                for arg in args {
                    self.type_ref(arg, span);
                }
            }
            TypeRef::Array(inner) => self.type_ref(inner, span),
            _ => {}
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per statement kind
    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                for inner in statements {
                    self.stmt(inner);
                }
            }
            Stmt::Expr(expr) => self.expr(expr),
            Stmt::LocalDecl {
                ty,
                declarators,
                span,
                ..
            } => {
                self.type_ref(ty, *span);
                for declarator in declarators {
                    if let Some(init) = &declarator.init {
                        self.expr(init);
                    }
                }
            }
            Stmt::Assign { target, value, .. } => {
                match target {
                    crate::ast::AssignTarget::Var(_) => {}
                    crate::ast::AssignTarget::Index { array, index } => {
                        self.expr(array);
                        self.expr(index);
                    }
                    crate::ast::AssignTarget::Field { object, .. } => self.expr(object),
                }
                self.expr(value);
            }
            Stmt::ForEach {
                ty,
                iterable,
                body,
                span,
                ..
            } => {
                self.type_ref(ty, *span);
                self.expr(iterable);
                self.stmt(body);
            }
            Stmt::If {
                cond, then, els, ..
            } => {
                self.expr(cond);
                self.stmt(then);
                if let Some(els) = els {
                    self.stmt(els);
                }
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.stmt(body);
            }
            Stmt::DoWhile { body, cond, .. } => {
                self.stmt(body);
                self.expr(cond);
            }
            Stmt::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                if let Some(init) = init {
                    self.stmt(init);
                }
                if let Some(cond) = cond {
                    self.expr(cond);
                }
                for stmt in update {
                    self.stmt(stmt);
                }
                self.stmt(body);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.expr(value);
                }
            }
            Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
                for arg in args {
                    self.expr(arg);
                }
            }
            Stmt::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                for stmt in body {
                    self.stmt(stmt);
                }
                for clause in catches {
                    for ty in &clause.types {
                        self.type_ref(ty, clause.span);
                    }
                    for stmt in &clause.body {
                        self.stmt(stmt);
                    }
                }
                for stmt in finally_body.iter().flatten() {
                    self.stmt(stmt);
                }
            }
            Stmt::Throw { value, .. } => self.expr(value),
            Stmt::Switch { selector, arms, .. } => {
                self.expr(selector);
                for arm in arms {
                    for label in arm.labels.iter().flatten() {
                        self.expr(label);
                    }
                    for stmt in &arm.body {
                        self.stmt(stmt);
                    }
                }
            }
            // An empty statement holds nothing, like a break or a continue.
            Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Empty(_) => {}
            Stmt::Labeled { body, .. } => self.stmt(body),
        }
    }

    /// A name used as a STATIC RECEIVER — `Arrays.sort(x)`, `Locale.US` — is a
    /// use of that class, and it needs the same import a type position does.
    /// Only this shape: a bare name standing alone is a variable read, and a
    /// local may legitimately be called anything.
    fn static_receiver(&mut self, receiver: &Expr) {
        if let Expr::Name { path, span } = receiver
            && let Some(first) = path.first()
        {
            self.name(first, *span);
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per expression kind
    fn expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal { .. } | Expr::This { .. } | Expr::Super { .. } => {}
            // A DOTTED name is a static access — `Locale.US`,
            // `StandardCharsets.UTF_8` — and needs its class's import. A
            // single-segment name is a variable read, which needs nothing.
            Expr::Name { path, span } => {
                if path.len() > 1
                    && let Some(first) = path.first()
                {
                    self.name(first, *span);
                }
            }
            Expr::Call { receiver, args, .. } => {
                if let Some(receiver) = receiver {
                    self.static_receiver(receiver);
                    self.expr(receiver);
                }
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Unary { operand, .. } => self.expr(operand),
            Expr::Cast { ty, operand, span } => {
                self.type_ref(ty, *span);
                self.expr(operand);
            }
            Expr::Index { array, index, .. } => {
                self.expr(array);
                self.expr(index);
            }
            Expr::Field { object, .. } => {
                self.static_receiver(object);
                self.expr(object);
            }
            Expr::NewArray {
                elem, dims, init, ..
            } => {
                self.type_ref(elem, expr.span());
                for dim in dims.iter().flatten() {
                    self.expr(dim);
                }
                for element in init.iter().flatten() {
                    self.expr(element);
                }
            }
            Expr::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.expr(element);
                }
            }
            Expr::NewObject {
                class,
                type_args,
                args,
                span,
                ..
            } => {
                self.name(class, *span);
                for arg in type_args {
                    self.type_ref(arg, *span);
                }
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::InstanceOf { value, ty, span } => {
                self.expr(value);
                self.type_ref(ty, *span);
            }
            Expr::SuperMethodCall { args, .. } => {
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::Ternary {
                cond, then, els, ..
            } => {
                self.expr(cond);
                self.expr(then);
                self.expr(els);
            }
            Expr::IncDec { target, .. } => self.expr(target),
            Expr::MethodRef { qualifier, .. } => self.expr(qualifier),
            Expr::Lambda { body, .. } => match body {
                crate::ast::LambdaBody::Expr(e) => self.expr(e),
                crate::ast::LambdaBody::Block(stmts) => {
                    for s in stmts {
                        self.stmt(s);
                    }
                }
            },
            Expr::Assign { target, value, .. } => {
                match target {
                    crate::ast::AssignTarget::Index { array, index } => {
                        self.expr(array);
                        self.expr(index);
                    }
                    crate::ast::AssignTarget::Field { object, .. } => self.expr(object),
                    crate::ast::AssignTarget::Var(_) => {}
                }
                self.expr(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{KNOWN_UNSUPPORTED, KNOWN_UNSUPPORTED_PACKAGES, PACKAGES};

    /// A class listed in BOTH tables is one fact answered two ways: it
    /// resolves fine written under a wildcard import and is refused by its own
    /// single import. `java.io.Reader` and `java.io.FileWriter` sat like that
    /// for months — modelled all along, and told they were not supported.
    #[test]
    fn no_class_is_both_offered_and_refused() {
        for (package, refused) in KNOWN_UNSUPPORTED {
            let Some((_, offered)) = PACKAGES.iter().find(|(name, _)| name == package) else {
                continue;
            };
            for name in *refused {
                assert!(
                    !offered.contains(name),
                    "{package}.{name} is both offered and refused"
                );
            }
        }
    }

    /// ...and a package listed in both is the same mistake one level up: the
    /// whole package would be refused, and the per-class list under it would
    /// never be read.
    #[test]
    fn no_package_is_both_modelled_and_refused() {
        for (package, _) in PACKAGES {
            assert!(
                !KNOWN_UNSUPPORTED_PACKAGES.contains(package),
                "package {package} is both modelled and refused"
            );
        }
    }

    /// A per-class refusal only reaches a program if the PACKAGE is modelled —
    /// otherwise the package's own refusal answers first and the class list is
    /// dead weight that will drift unnoticed.
    #[test]
    fn every_refused_class_sits_in_a_modelled_package() {
        for (package, names) in KNOWN_UNSUPPORTED {
            assert!(
                PACKAGES.iter().any(|(name, _)| name == package),
                "{package} lists {names:?} as refused, but the package is not modelled"
            );
        }
    }

    /// The package refusal list is recorded from a real Java 11 module image,
    /// so every entry is a package that EXISTS. A name with no dot could not
    /// be one, and a `javax.` or `java.` prefix is what makes the honest
    /// message honest.
    #[test]
    fn every_refused_package_is_a_real_jdk_one() {
        for package in KNOWN_UNSUPPORTED_PACKAGES {
            assert!(
                package.starts_with("java.") || package.starts_with("javax."),
                "{package} is not a JDK package"
            );
            assert!(package.contains('.'), "{package} is not a package name");
        }
    }
}
