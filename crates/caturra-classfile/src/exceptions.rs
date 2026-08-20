//! The library exception hierarchy, shared by the compiler (catch
//! typing, unreachable-catch checks) and the VM (handler matching).
//!
//! Internal (slash) names. User-defined exception classes live outside
//! this table: their superclass chains are walked class-file by class-file
//! and cross into it at the first `java/...` superclass (the VM's
//! `thrown_matches`/`instance_is_throwable`), so the table is the whole
//! catchable world only for LIBRARY classes.

/// `(class, superclass)` pairs; `java/lang/Throwable` is the root.
///
/// **A library class missing from this table is not merely unnamed in a
/// `catch` — it is UNCATCHABLE, and kills the run.** The VM's unwinder consults this to decide
/// whether a thrown class is a throwable at all, so an exception it does not
/// know escapes even `catch (Throwable)`. The reflective exceptions below were
/// missing, and the Unit 2 constructor/attribute validators throw them: the
/// first one aborted the whole validation run, so the tests after it never ran
/// and the student was told every test passed.
pub const EXCEPTIONS: &[(&str, &str)] = &[
    ("java/lang/Exception", "java/lang/Throwable"),
    ("java/lang/Error", "java/lang/Throwable"),
    ("java/lang/RuntimeException", "java/lang/Exception"),
    // Reflection (JLS: all checked, all under ReflectiveOperationException).
    (
        "java/lang/ReflectiveOperationException",
        "java/lang/Exception",
    ),
    (
        "java/lang/ClassNotFoundException",
        "java/lang/ReflectiveOperationException",
    ),
    (
        "java/lang/InstantiationException",
        "java/lang/ReflectiveOperationException",
    ),
    (
        "java/lang/NoSuchFieldException",
        "java/lang/ReflectiveOperationException",
    ),
    (
        "java/lang/NoSuchMethodException",
        "java/lang/ReflectiveOperationException",
    ),
    (
        "java/lang/IllegalAccessException",
        "java/lang/ReflectiveOperationException",
    ),
    (
        "java/lang/reflect/InvocationTargetException",
        "java/lang/ReflectiveOperationException",
    ),
    // `Object.clone()` throws this when the class does not implement the
    // `Cloneable` marker — a CHECKED exception, which is why the copy idiom
    // has to declare or catch it.
    (
        "java/lang/CloneNotSupportedException",
        "java/lang/Exception",
    ),
    // Linkage errors are Errors, not Exceptions: `catch (Exception)` must not
    // take a VerifyError, but `catch (Throwable)` must.
    ("java/lang/LinkageError", "java/lang/Error"),
    // An exception escaping a static initializer is wrapped in this (an Error,
    // so `catch (Exception)` does not take it — its CAUSE is the original).
    (
        "java/lang/ExceptionInInitializerError",
        "java/lang/LinkageError",
    ),
    // Thrown on every active use of a class whose initialization already failed
    // abruptly (JLS §12.4.2 — the class is permanently Erroneous).
    ("java/lang/NoClassDefFoundError", "java/lang/LinkageError"),
    ("java/lang/VerifyError", "java/lang/LinkageError"),
    (
        "java/lang/IllegalArgumentException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/IllegalStateException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/ArithmeticException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/NullPointerException",
        "java/lang/RuntimeException",
    ),
    ("java/lang/ClassCastException", "java/lang/RuntimeException"),
    (
        "java/lang/UnsupportedOperationException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/ArrayStoreException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/NegativeArraySizeException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/IndexOutOfBoundsException",
        "java/lang/RuntimeException",
    ),
    (
        "java/lang/ArrayIndexOutOfBoundsException",
        "java/lang/IndexOutOfBoundsException",
    ),
    (
        "java/lang/StringIndexOutOfBoundsException",
        "java/lang/IndexOutOfBoundsException",
    ),
    (
        "java/lang/NumberFormatException",
        "java/lang/IllegalArgumentException",
    ),
    // The VM-condition errors sit under VirtualMachineError, as the JDK's do —
    // `catch (VirtualMachineError e)` catches both on a real JVM.
    ("java/lang/VirtualMachineError", "java/lang/Error"),
    (
        "java/lang/StackOverflowError",
        "java/lang/VirtualMachineError",
    ),
    (
        "java/lang/OutOfMemoryError",
        "java/lang/VirtualMachineError",
    ),
    (
        "java/util/NoSuchElementException",
        "java/lang/RuntimeException",
    ),
    (
        "java/util/InputMismatchException",
        "java/util/NoSuchElementException",
    ),
    (
        "java/util/EmptyStackException",
        "java/lang/RuntimeException",
    ),
    (
        "java/util/ConcurrentModificationException",
        "java/lang/RuntimeException",
    ),
    (
        "java/util/regex/PatternSyntaxException",
        "java/lang/IllegalArgumentException",
    ),
    (
        "java/util/IllegalFormatException",
        "java/lang/IllegalArgumentException",
    ),
    (
        "java/util/UnknownFormatConversionException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/MissingFormatArgumentException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/IllegalFormatConversionException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/IllegalFormatCodePointException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/FormatFlagsConversionMismatchException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/IllegalFormatPrecisionException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/IllegalFormatFlagsException",
        "java/util/IllegalFormatException",
    ),
    // Its own IllegalFormatException, NOT a kind of IllegalFormatFlagsException
    // — the two are siblings, however alike their names read.
    (
        "java/util/DuplicateFormatFlagsException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/MissingFormatWidthException",
        "java/util/IllegalFormatException",
    ),
    (
        "java/util/IllegalFormatWidthException",
        "java/util/IllegalFormatException",
    ),
    ("java/io/IOException", "java/lang/Exception"),
    ("java/io/FileNotFoundException", "java/io/IOException"),
    ("java/io/UncheckedIOException", "java/lang/RuntimeException"),
    // The `java.nio.file` failures a program can catch. They are IOExceptions,
    // and were not in this table at all: `Files.readAllLines` on a missing file
    // threw a `NoSuchFileException` that `catch (IOException e)` did not catch,
    // so a program that handles a missing file died instead.
    ("java/nio/file/FileSystemException", "java/io/IOException"),
    (
        "java/nio/file/NoSuchFileException",
        "java/nio/file/FileSystemException",
    ),
    (
        "java/nio/file/FileAlreadyExistsException",
        "java/nio/file/FileSystemException",
    ),
    (
        "java/nio/file/DirectoryNotEmptyException",
        "java/nio/file/FileSystemException",
    ),
    (
        "java/nio/file/AccessDeniedException",
        "java/nio/file/FileSystemException",
    ),
    (
        "java/nio/file/InvalidPathException",
        "java/lang/IllegalArgumentException",
    ),
];

/// Whether `name` (internal form) is a known throwable class.
#[must_use]
pub fn is_exception_class(name: &str) -> bool {
    name == "java/lang/Throwable" || EXCEPTIONS.iter().any(|(class, _)| *class == name)
}

/// Whether `sub` is `sup` or inherits from it (internal names).
#[must_use]
pub fn is_exception_subclass(sub: &str, sup: &str) -> bool {
    let mut current = sub;
    loop {
        if current == sup {
            return true;
        }
        match EXCEPTIONS.iter().find(|(class, _)| *class == current) {
            Some((_, parent)) => current = parent,
            None => return false,
        }
    }
}

/// The superclass of a throwable (internal names), or `None` at `Throwable`.
///
/// A multi-catch needs it: the caught variable's type is a supertype of every
/// alternative, so the compiler climbs from one until it covers the rest.
#[must_use]
pub fn parent_of(name: &str) -> Option<&'static str> {
    EXCEPTIONS
        .iter()
        .find(|(class, _)| *class == name)
        .map(|(_, parent)| *parent)
}

/// Resolve a simple name (`ArithmeticException`) to its internal name.
#[must_use]
pub fn internal_name_of(simple: &str) -> Option<&'static str> {
    if simple == "Throwable" {
        return Some("java/lang/Throwable");
    }
    EXCEPTIONS
        .iter()
        .map(|(class, _)| *class)
        .find(|class| class.rsplit('/').next() == Some(simple))
}

/// The dotted display form (`java.lang.ArithmeticException`).
#[must_use]
pub fn dotted(internal: &str) -> String {
    internal.replace('/', ".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtype_chains_walk_to_throwable() {
        assert!(is_exception_subclass(
            "java/lang/ArrayIndexOutOfBoundsException",
            "java/lang/IndexOutOfBoundsException"
        ));
        assert!(is_exception_subclass(
            "java/lang/ArrayIndexOutOfBoundsException",
            "java/lang/RuntimeException"
        ));
        assert!(is_exception_subclass(
            "java/lang/ArithmeticException",
            "java/lang/Throwable"
        ));
        assert!(!is_exception_subclass(
            "java/lang/ArithmeticException",
            "java/lang/Error"
        ));
        assert!(!is_exception_subclass(
            "java/lang/StackOverflowError",
            "java/lang/Exception"
        ));
        assert!(is_exception_subclass(
            "java/io/FileNotFoundException",
            "java/lang/Exception"
        ));
    }

    #[test]
    fn simple_names_resolve() {
        assert_eq!(
            internal_name_of("NumberFormatException"),
            Some("java/lang/NumberFormatException")
        );
        assert_eq!(
            internal_name_of("InputMismatchException"),
            Some("java/util/InputMismatchException")
        );
        assert_eq!(internal_name_of("NotAThing"), None);
    }
}
