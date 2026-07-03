//! Intrinsic ("native") classes: core library members whose semantics
//! live in Rust rather than in interpreted bytecode.
//!
//! Resolution order per `specs/SCOPE.md`: intrinsics → baked-in classlib
//! → user classes. v0 covers `java.lang.System.out`/`err` and
//! `java.io.PrintStream.print`/`println`; the surface grows with
//! `specs/LANGUAGE.md` staging.

use crate::io::ConsoleIo;
use crate::value::{Heap, HeapObject, HeapRef, JValue, StdStream};
use crate::vm::VmError;

/// Lazily allocated intrinsic singletons (`System.out`, `System.err`).
#[derive(Debug, Default)]
pub struct IntrinsicStatics {
    stdout: Option<HeapRef>,
    stderr: Option<HeapRef>,
}

impl IntrinsicStatics {
    /// Resolve an intrinsic static field like `java/lang/System.out`,
    /// allocating its singleton on first use.
    pub fn static_field(&mut self, heap: &mut Heap, class: &str, field: &str) -> Option<JValue> {
        match (class, field) {
            ("java/lang/System", "out") => {
                let reference = *self
                    .stdout
                    .get_or_insert_with(|| heap.alloc(HeapObject::PrintStream(StdStream::Out)));
                Some(JValue::Ref(Some(reference)))
            }
            ("java/lang/System", "err") => {
                let reference = *self
                    .stderr
                    .get_or_insert_with(|| heap.alloc(HeapObject::PrintStream(StdStream::Err)));
                Some(JValue::Ref(Some(reference)))
            }
            _ => None,
        }
    }
}

/// Instantiate an intrinsic class (the `new` opcode). Returns `None`
/// for classes the VM doesn't know how to construct.
#[must_use]
pub fn instantiate(class: &str) -> Option<HeapObject> {
    match class {
        "java/lang/StringBuilder" => Some(HeapObject::StringBuilder(Vec::new())),
        _ => None,
    }
}

/// Invoke an intrinsic constructor (`invokespecial <init>`).
pub fn invoke_special(
    heap: &Heap,
    receiver: HeapRef,
    class: &str,
    method: &str,
    descriptor: &str,
) -> Result<(), VmError> {
    // Object's constructor does nothing — every user constructor calls
    // it as the implicit super().
    if class == "java/lang/Object" && method == "<init>" && descriptor == "()V" {
        return Ok(());
    }
    match (heap.get(receiver), method, descriptor) {
        // StringBuilder is fully initialized at `new`; its no-arg
        // constructor has nothing left to do.
        (Some(HeapObject::StringBuilder(_)), "<init>", "()V") => Ok(()),
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class}.{method}{descriptor}"
        ))),
    }
}

/// Invoke an intrinsic instance method. Returns `Ok(None)` for `void`,
/// `Ok(Some(value))` for a result, or `Err` if the member is not an
/// intrinsic the VM knows.
pub fn invoke_virtual(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    class: &str,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let receiver_object = heap.get(receiver).ok_or_else(|| VmError::MalformedClass {
        name: class.to_owned(),
        reason: format!("dangling heap reference {receiver}"),
    })?;

    match (receiver_object, method) {
        (HeapObject::PrintStream(stream), "print" | "println") => {
            let stream = *stream;
            let mut text = print_argument_text(heap, descriptor, args)?;
            if method == "println" {
                text.push('\n');
            }
            let bytes = text.as_bytes();
            match stream {
                StdStream::Out => console.stdout(bytes),
                StdStream::Err => console.stderr(bytes),
            }
            Ok(None)
        }
        (HeapObject::StringBuilder(_), "append") => {
            let appended = append_argument_text(heap, descriptor, args)?;
            let Some(HeapObject::StringBuilder(units)) = heap.get_mut(receiver) else {
                unreachable!("receiver kind checked above");
            };
            units.extend(appended.encode_utf16());
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        (HeapObject::StringBuilder(units), "toString") => {
            let units = units.clone();
            let text = heap.alloc(HeapObject::JavaString(units));
            Ok(Some(JValue::Ref(Some(text))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class}.{method}{descriptor}"
        ))),
    }
}

/// Render a `StringBuilder.append` argument the way Java would.
fn append_argument_text(heap: &Heap, descriptor: &str, args: &[JValue]) -> Result<String, VmError> {
    let text = match (descriptor, args) {
        ("(I)Ljava/lang/StringBuilder;", [JValue::Int(v)]) => v.to_string(),
        ("(Z)Ljava/lang/StringBuilder;", [JValue::Int(v)]) => {
            if *v != 0 { "true" } else { "false" }.to_owned()
        }
        ("(C)Ljava/lang/StringBuilder;", [JValue::Int(v)]) => {
            let unit = u32::try_from(*v).unwrap_or(u32::from(u16::MAX));
            char::from_u32(unit).map_or_else(|| String::from('\u{FFFD}'), String::from)
        }
        ("(D)Ljava/lang/StringBuilder;", [JValue::Double(v)]) => java_double_to_string(*v),
        ("(Ljava/lang/String;)Ljava/lang/StringBuilder;", [JValue::Ref(reference)]) => {
            match reference {
                None => String::from("null"),
                Some(reference) => heap.string_text(*reference).ok_or_else(|| {
                    VmError::UnknownIntrinsic(String::from(
                        "append argument is not a string object",
                    ))
                })?,
            }
        }
        _ => {
            return Err(VmError::UnknownIntrinsic(format!(
                "StringBuilder.append overload {descriptor}"
            )));
        }
    };
    Ok(text)
}

/// Render a print/println argument the way Java would.
fn print_argument_text(heap: &Heap, descriptor: &str, args: &[JValue]) -> Result<String, VmError> {
    let text = match (descriptor, args) {
        ("()V", []) => String::new(),
        ("(I)V", [JValue::Int(v)]) => v.to_string(),
        ("(Z)V", [JValue::Int(v)]) => if *v != 0 { "true" } else { "false" }.to_owned(),
        ("(C)V", [JValue::Int(v)]) => {
            let unit = u32::try_from(*v).unwrap_or(u32::from(u16::MAX));
            char::from_u32(unit).map_or_else(|| String::from('\u{FFFD}'), String::from)
        }
        ("(D)V", [JValue::Double(v)]) => java_double_to_string(*v),
        ("(Ljava/lang/String;)V", [JValue::Ref(reference)]) => match reference {
            None => String::from("null"),
            Some(reference) => heap.string_text(*reference).ok_or_else(|| {
                VmError::UnknownIntrinsic(String::from("println argument is not a string object"))
            })?,
        },
        _ => {
            return Err(VmError::UnknownIntrinsic(format!(
                "PrintStream overload {descriptor} with {} args",
                args.len()
            )));
        }
    };
    Ok(text)
}

/// Format a `double` the way `Double.toString` does for common cases.
///
/// TODO(classlib): full `Double.toString` semantics (shortest digits
/// that round-trip, exact scientific-notation thresholds). Current
/// coverage: NaN/infinities, integral values gaining `.0`, and the 1e7
/// switch to scientific notation.
fn java_double_to_string(value: f64) -> String {
    if value.is_nan() {
        return String::from("NaN");
    }
    if value.is_infinite() {
        return String::from(if value > 0.0 { "Infinity" } else { "-Infinity" });
    }
    let magnitude = value.abs();
    if magnitude != 0.0 && !(1e-3..1e7).contains(&magnitude) {
        // Java: "1.0E7". Rust {:E}: "1E7" — restore the ".0".
        let formatted = format!("{value:E}");
        if let Some((mantissa, exponent)) = formatted.split_once('E')
            && !mantissa.contains('.')
        {
            return format!("{mantissa}.0E{exponent}");
        }
        return formatted;
    }
    if value.fract() == 0.0 {
        return format!("{value:.1}");
    }
    format!("{value}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::BufferedConsole;

    #[test]
    fn println_string_writes_line_to_stdout() {
        let mut heap = Heap::new();
        let mut statics = IntrinsicStatics::default();
        let mut console = BufferedConsole::new();

        let JValue::Ref(Some(out)) = statics
            .static_field(&mut heap, "java/lang/System", "out")
            .unwrap()
        else {
            panic!("expected a reference");
        };
        let text = heap.alloc_string("Hello, World!");
        invoke_virtual(
            &mut heap,
            &mut console,
            out,
            "java/io/PrintStream",
            "println",
            "(Ljava/lang/String;)V",
            &[JValue::Ref(Some(text))],
        )
        .unwrap();
        assert_eq!(console.stdout_text(), "Hello, World!\n");
    }

    #[test]
    fn system_out_is_a_singleton() {
        let mut heap = Heap::new();
        let mut statics = IntrinsicStatics::default();
        let a = statics.static_field(&mut heap, "java/lang/System", "out");
        let b = statics.static_field(&mut heap, "java/lang/System", "out");
        assert_eq!(a, b);
    }

    #[test]
    fn err_goes_to_stderr() {
        let mut heap = Heap::new();
        let mut statics = IntrinsicStatics::default();
        let mut console = BufferedConsole::new();
        let JValue::Ref(Some(err)) = statics
            .static_field(&mut heap, "java/lang/System", "err")
            .unwrap()
        else {
            panic!("expected a reference");
        };
        invoke_virtual(
            &mut heap,
            &mut console,
            err,
            "java/io/PrintStream",
            "println",
            "(I)V",
            &[JValue::Int(7)],
        )
        .unwrap();
        assert_eq!(console.stderr_text(), "7\n");
        assert_eq!(console.stdout_text(), "");
    }

    #[test]
    fn stringbuilder_appends_and_converts() {
        let mut heap = Heap::new();
        let mut console = BufferedConsole::new();
        let builder =
            heap.alloc(instantiate("java/lang/StringBuilder").expect("known intrinsic class"));
        invoke_special(&heap, builder, "java/lang/StringBuilder", "<init>", "()V").unwrap();

        let hello = heap.alloc_string("x = ");
        invoke_virtual(
            &mut heap,
            &mut console,
            builder,
            "java/lang/StringBuilder",
            "append",
            "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
            &[JValue::Ref(Some(hello))],
        )
        .unwrap();
        invoke_virtual(
            &mut heap,
            &mut console,
            builder,
            "java/lang/StringBuilder",
            "append",
            "(I)Ljava/lang/StringBuilder;",
            &[JValue::Int(42)],
        )
        .unwrap();
        let result = invoke_virtual(
            &mut heap,
            &mut console,
            builder,
            "java/lang/StringBuilder",
            "toString",
            "()Ljava/lang/String;",
            &[],
        )
        .unwrap();
        let Some(JValue::Ref(Some(text))) = result else {
            panic!("expected a string reference, got {result:?}");
        };
        assert_eq!(heap.string_text(text).as_deref(), Some("x = 42"));
    }

    #[test]
    fn doubles_format_like_java() {
        assert_eq!(java_double_to_string(3.5), "3.5");
        assert_eq!(java_double_to_string(2.0), "2.0");
        assert_eq!(java_double_to_string(0.0), "0.0");
        assert_eq!(java_double_to_string(-4.0), "-4.0");
        assert_eq!(java_double_to_string(10_000_000.0), "1.0E7");
        assert_eq!(java_double_to_string(f64::NAN), "NaN");
        assert_eq!(java_double_to_string(f64::INFINITY), "Infinity");
    }

    #[test]
    fn null_string_prints_null() {
        let heap = Heap::new();
        let text = print_argument_text(&heap, "(Ljava/lang/String;)V", &[JValue::NULL]).unwrap();
        assert_eq!(text, "null");
    }
}
