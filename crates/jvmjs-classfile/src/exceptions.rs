//! The library exception hierarchy, shared by the compiler (catch
//! typing, unreachable-catch checks) and the VM (handler matching).
//!
//! Internal (slash) names. User-defined exception classes are not yet
//! supported, so this closed table is the whole catchable world.

/// `(class, superclass)` pairs; `java/lang/Throwable` is the root.
pub const EXCEPTIONS: &[(&str, &str)] = &[
    ("java/lang/Exception", "java/lang/Throwable"),
    ("java/lang/Error", "java/lang/Throwable"),
    ("java/lang/RuntimeException", "java/lang/Exception"),
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
    ("java/lang/StackOverflowError", "java/lang/Error"),
    (
        "java/util/NoSuchElementException",
        "java/lang/RuntimeException",
    ),
    (
        "java/util/InputMismatchException",
        "java/util/NoSuchElementException",
    ),
    ("java/io/IOException", "java/lang/Exception"),
    ("java/io/FileNotFoundException", "java/io/IOException"),
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
