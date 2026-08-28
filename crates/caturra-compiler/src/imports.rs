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
    // `Locale` is usable only as the leading argument of a format call, which
    // is where a program actually reaches for it: caturra formats in the
    // US/root locale, so `String.format(Locale.US, …)` is exact and the
    // argument is dropped.
    "Locale",
];

const JAVA_IO: &[&str] = &[
    "File",
    "PrintWriter",
    "BufferedReader",
    "FileReader",
    "InputStreamReader",
    "FileNotFoundException",
    "IOException",
    // `Closeable` is modeled as an interface (it is `AutoCloseable` plus a
    // narrower `throws`), so a resource class may implement the one every
    // I/O tutorial names.
    "Closeable",
];
/// `java.nio.file` — the modeled slice: build a `Path` and read/write it through
/// `Files`. The rest of `java.nio` stays unsupported.
const JAVA_NIO_FILE: &[&str] = &["Files", "Path", "Paths"];
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
            "Vector",
            "Hashtable",
            "AbstractList",
            "AbstractCollection",
            "AbstractSet",
            "AbstractSequentialList",
        ],
    ),
    // `Enumeration` is the pre-collections cursor: `Collections.enumeration`
    // and `Collections.list` are its only real uses today, and neither is
    // modelled.
    ("java.util", &["Enumeration"]),
    // `Runtime` reports free/total/max memory and runs external processes.
    // caturra collects on its own schedule inside one WASM instance, so every
    // number it could answer would be fiction about a heap the program cannot
    // influence — and `exec` has nothing to exec.
    ("java.lang", &["Runtime", "Process", "ProcessBuilder"]),
    (
        "java.io",
        &[
            "Serializable",
            "BufferedWriter",
            "FileWriter",
            "PrintStream",
            "InputStream",
            "OutputStream",
            "Reader",
            "Writer",
        ],
    ),
    // `Thread` and friends are real Java that caturra will not be growing: a
    // program here runs on one thread, in one WASM instance. Saying "unknown type
    // 'Thread'" about a class every Java programmer knows reads as our bug.
    (
        "java.lang",
        &[
            // The synchronized twin of `StringBuilder`. Aliasing the two would
            // make `getClass()` lie about which one a program built, and
            // synchronization is the only other difference — so it is refused
            // by name, which at least says what it is.
            "StringBuffer",
            "StrictMath",
            "Thread",
            "ThreadLocal",
            "Process",
            "ProcessBuilder",
        ],
    ),
];

/// Real JDK packages we don't model at all (for wildcard/unknown-class
/// imports of them, an honest message beats "does not exist").
const KNOWN_UNSUPPORTED_PACKAGES: &[&str] = &[
    "java.net",
    "java.nio",
    "java.time",
    "java.text",
    "java.math",
    "java.sql",
    "java.util.concurrent",
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
        .map(|(package, _)| not_supported(&format!("{package}.{simple}")))
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
    ("java.nio.charset", JAVA_NIO_CHARSET),
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
            if REQUIRES_IMPORT.contains(name) && (import.wildcard || import.path.last() == Some(&(*name).to_owned()))
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
    error: F,
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
        error,
    };
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
            // javac: "cannot find symbol — symbol: class Scanner,
            // location: class Main".
            (self.error)(format!("cannot find symbol: class {name}"), span);
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
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
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
