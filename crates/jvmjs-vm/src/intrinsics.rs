//! Intrinsic ("native") classes: core library members whose semantics
//! live in Rust rather than in interpreted bytecode.
//!
//! Resolution order per `specs/SCOPE.md`: intrinsics → baked-in classlib
//! → user classes. v0 covers `java.lang.System.out`/`err` and
//! `java.io.PrintStream.print`/`println`; the surface grows with
//! `specs/LANGUAGE.md` staging.

use crate::io::ConsoleIo;
use crate::value::{Heap, HeapObject, HeapRef, JValue, StdStream};
use crate::vfs::VirtualFileSystem;
use crate::vm::VmError;

/// Lazily allocated intrinsic singletons (`System.out`, `System.err`).
#[derive(Debug, Default)]
pub struct IntrinsicStatics {
    stdout: Option<HeapRef>,
    stderr: Option<HeapRef>,
    stdin: Option<HeapRef>,
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
            ("java/lang/System", "in") => {
                let reference = *self
                    .stdin
                    .get_or_insert_with(|| heap.alloc(HeapObject::InputStream));
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
        "java/util/Scanner" => Some(HeapObject::Scanner {
            buffer: String::new(),
            pos: 0,
            eof: false,
        }),
        "java/util/ArrayList" => Some(HeapObject::ArrayList(Vec::new())),
        "java/io/File" => Some(HeapObject::File(String::new())),
        "java/io/PrintWriter" => Some(HeapObject::Writer {
            path: String::new(),
        }),
        _ => {
            if jvmjs_classfile::exceptions::is_exception_class(class) {
                return Some(HeapObject::Exception {
                    class_name: jvmjs_classfile::exceptions::dotted(class),
                    message: None,
                });
            }
            None
        }
    }
}

/// Invoke an intrinsic constructor (`invokespecial <init>`).
pub fn invoke_special(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    receiver: HeapRef,
    class: &str,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<(), VmError> {
    // Object's constructor does nothing — every user constructor calls
    // it as the implicit super().
    if class == "java/lang/Object" && method == "<init>" && descriptor == "()V" {
        return Ok(());
    }

    let string_arg = |heap: &Heap, value: &JValue| -> Result<String, VmError> {
        match value {
            JValue::Ref(Some(reference)) => heap
                .string_text(*reference)
                .ok_or_else(|| throw("java.lang.ClassCastException: not a String")),
            JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
            _ => Err(throw("java.lang.VerifyError: expected a String argument")),
        }
    };
    let file_arg = |heap: &Heap, value: &JValue| -> Result<String, VmError> {
        match value {
            JValue::Ref(Some(reference)) => match heap.get(*reference) {
                Some(HeapObject::File(path)) => Ok(path.clone()),
                _ => Err(throw("java.lang.ClassCastException: not a File")),
            },
            JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
            _ => Err(throw("java.lang.VerifyError: expected a File argument")),
        }
    };

    match (method, descriptor) {
        // Fully initialized at `new` (the Scanner over System.in
        // ignores the stream object — stdin is the only stream). The
        // no-arg case also covers `super()` into a library throwable
        // from a user exception class.
        ("<init>", "()V" | "(Ljava/io/InputStream;)V") => Ok(()),
        ("<init>", "(Ljava/lang/String;)V") => {
            let text = string_arg(heap, &args[0])?;
            match heap.get_mut(receiver) {
                Some(HeapObject::File(path)) => {
                    *path = text;
                    Ok(())
                }
                Some(HeapObject::Exception { message, .. }) => {
                    *message = Some(text);
                    Ok(())
                }
                // A user exception class chaining `super("message")`
                // into its library throwable parent: stash the message
                // in a reserved field.
                Some(HeapObject::Instance { .. })
                    if jvmjs_classfile::exceptions::is_exception_class(class) =>
                {
                    let reference = heap.alloc_string(&text);
                    if let Some(HeapObject::Instance { fields, .. }) = heap.get_mut(receiver) {
                        fields.insert(String::from("__message"), JValue::Ref(Some(reference)));
                    }
                    Ok(())
                }
                Some(HeapObject::Writer { path }) => {
                    // PrintWriter(String) truncates on open (like Java)
                    // and writes through as the program prints.
                    path.clone_from(&text);
                    vfs.write_file(&text, Vec::new())
                        .map_err(|e| throw(format!("java.io.FileNotFoundException: {e}")))?;
                    Ok(())
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                ))),
            }
        }
        ("<init>", "(Ljava/io/File;)V") => {
            let target = file_arg(heap, &args[0])?;
            match heap.get(receiver) {
                Some(HeapObject::Writer { .. }) => {
                    vfs.write_file(&target, Vec::new())
                        .map_err(|e| throw(format!("java.io.FileNotFoundException: {e}")))?;
                    if let Some(HeapObject::Writer { path }) = heap.get_mut(receiver) {
                        *path = target;
                    }
                    Ok(())
                }
                Some(HeapObject::Scanner { .. }) => {
                    // Scanner(File): slurp the whole file up front.
                    let content = vfs
                        .read_file(&target)
                        .map_err(|_| {
                            throw(format!(
                                "java.io.FileNotFoundException: {target} \
                                 (No such file or directory)"
                            ))
                        })?
                        .to_vec();
                    let text = String::from_utf8_lossy(&content).into_owned();
                    if let Some(HeapObject::Scanner { buffer, eof, pos }) = heap.get_mut(receiver) {
                        *buffer = text;
                        *pos = 0;
                        *eof = true;
                    }
                    Ok(())
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                ))),
            }
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class}.{method}{descriptor}"
        ))),
    }
}

/// Invoke an intrinsic instance method. Returns `Ok(None)` for `void`,
/// `Ok(Some(value))` for a result, or `Err` if the member is not an
/// intrinsic the VM knows.
#[allow(clippy::too_many_arguments)] // one boundary call from the dispatch loop
pub fn invoke_virtual(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    vfs: &mut VirtualFileSystem,
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
        (HeapObject::JavaString(_), _) => string_method(heap, receiver, method, args),
        (HeapObject::Scanner { .. }, _) => scanner_method(heap, console, receiver, method),
        (HeapObject::ArrayList(_), _) => list_method(heap, receiver, method, descriptor, args),
        (HeapObject::File(_), _) => file_method(heap, vfs, receiver, method),
        (
            HeapObject::Exception {
                class_name,
                message,
            },
            "getMessage" | "toString",
        ) => {
            let rendered = if method == "getMessage" {
                match message {
                    Some(message) => message.clone(),
                    None => return Ok(Some(JValue::NULL)),
                }
            } else {
                match message {
                    Some(message) => format!("{class_name}: {message}"),
                    None => class_name.clone(),
                }
            };
            let reference = heap.alloc_string(&rendered);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        (HeapObject::Writer { .. }, _) => {
            writer_method(heap, vfs, receiver, method, descriptor, args)
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class}.{method}{descriptor}"
        ))),
    }
}

fn throw(message: impl Into<String>) -> VmError {
    VmError::UncaughtException(message.into())
}

/// `java.lang.String` instance methods over UTF-16 code units.
#[allow(clippy::too_many_lines)]
fn string_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(HeapObject::JavaString(units)) = heap.get(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    let units = units.clone();
    let len = i32::try_from(units.len()).unwrap_or(i32::MAX);

    // Resolve a string argument's code units (null → NPE like Java).
    let arg_units = |value: &JValue| -> Result<Vec<u16>, VmError> {
        match value {
            JValue::Ref(Some(reference)) => match heap.get(*reference) {
                Some(HeapObject::JavaString(other)) => Ok(other.clone()),
                _ => Err(throw("java.lang.ClassCastException: not a String")),
            },
            JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
            _ => Err(throw("java.lang.VerifyError: expected a String argument")),
        }
    };

    match (method, args) {
        ("length", []) => Ok(Some(JValue::Int(len))),
        ("isEmpty", []) => Ok(Some(JValue::Int(i32::from(units.is_empty())))),
        ("charAt", [JValue::Int(index)]) => {
            let unit = usize::try_from(*index)
                .ok()
                .and_then(|i| units.get(i))
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.StringIndexOutOfBoundsException: String index out of \
                         range: {index}"
                    ))
                })?;
            Ok(Some(JValue::Int(i32::from(*unit))))
        }
        ("substring", [JValue::Int(begin)]) => substring(heap, &units, *begin, len),
        // subSequence is substring by another name (CharSequence view).
        ("substring" | "subSequence", [JValue::Int(begin), JValue::Int(end)]) => {
            substring_range(heap, &units, *begin, *end)
        }
        ("indexOf", [needle @ JValue::Ref(_)]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(index_of(&units, &needle))))
        }
        ("contains", [needle]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(i32::from(index_of(&units, &needle) >= 0))))
        }
        ("startsWith", [prefix]) => {
            let prefix = arg_units(prefix)?;
            Ok(Some(JValue::Int(i32::from(units.starts_with(&prefix)))))
        }
        ("endsWith", [suffix]) => {
            let suffix = arg_units(suffix)?;
            Ok(Some(JValue::Int(i32::from(units.ends_with(&suffix)))))
        }
        ("equals", [other]) => {
            // equals(Object): a null or non-string argument is false in
            // Java; ours can only receive strings or null.
            let result = match other {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(other_units)) => *other_units == units,
                    _ => false,
                },
                _ => false,
            };
            Ok(Some(JValue::Int(i32::from(result))))
        }
        ("equalsIgnoreCase", [other]) => {
            let other = arg_units(other)?;
            let a = String::from_utf16_lossy(&units).to_lowercase();
            let b = String::from_utf16_lossy(&other).to_lowercase();
            Ok(Some(JValue::Int(i32::from(a == b))))
        }
        ("compareTo", [other]) => {
            let other = arg_units(other)?;
            Ok(Some(JValue::Int(compare_utf16(&units, &other))))
        }
        ("split", [delimiter]) => {
            let delimiter = arg_units(delimiter)?;
            let parts = split_units(&units, &delimiter);
            let refs: Vec<JValue> = parts
                .into_iter()
                .map(|part| JValue::Ref(Some(heap.alloc(HeapObject::JavaString(part)))))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(refs));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("replace", [JValue::Int(from), JValue::Int(to)]) => {
            let from = u16::try_from(*from).unwrap_or(u16::MAX);
            let to = u16::try_from(*to).unwrap_or(u16::MAX);
            let replaced: Vec<u16> = units
                .iter()
                .map(|unit| if *unit == from { to } else { *unit })
                .collect();
            let reference = heap.alloc(HeapObject::JavaString(replaced));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("replace", [from, to]) => {
            let from = arg_units(from)?;
            let to = arg_units(to)?;
            let replaced = replace_units(&units, &from, &to);
            let reference = heap.alloc(HeapObject::JavaString(replaced));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toUpperCase", []) => {
            let text = String::from_utf16_lossy(&units).to_uppercase();
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toLowerCase", []) => {
            let text = String::from_utf16_lossy(&units).to_lowercase();
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("trim", []) => {
            let text = String::from_utf16_lossy(&units);
            let reference = heap.alloc_string(text.trim_matches(|c| c <= ' '));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("strip" | "stripLeading" | "stripTrailing", []) => {
            let text = String::from_utf16_lossy(&units);
            let stripped = match method {
                "strip" => text.trim_matches(char::is_whitespace),
                "stripLeading" => text.trim_start_matches(char::is_whitespace),
                _ => text.trim_end_matches(char::is_whitespace),
            };
            let reference = heap.alloc_string(stripped);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("isBlank", []) => {
            let blank = String::from_utf16_lossy(&units)
                .chars()
                .all(char::is_whitespace);
            Ok(Some(JValue::Int(i32::from(blank))))
        }
        ("repeat", [JValue::Int(count)]) => {
            if *count < 0 {
                return Err(throw(format!(
                    "java.lang.IllegalArgumentException: count is negative: {count}"
                )));
            }
            let mut repeated =
                Vec::with_capacity(units.len() * usize::try_from(*count).unwrap_or(0));
            for _ in 0..*count {
                repeated.extend_from_slice(&units);
            }
            let reference = heap.alloc(HeapObject::JavaString(repeated));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("concat", [other]) => {
            let mut joined = units.clone();
            joined.extend(arg_units(other)?);
            let reference = heap.alloc(HeapObject::JavaString(joined));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("compareToIgnoreCase", [other]) => {
            let other = arg_units(other)?;
            let fold = |unit: u16| -> u16 {
                char::from_u32(u32::from(unit)).map_or(unit, |c| {
                    let upper = c.to_uppercase().next().unwrap_or(c);
                    let lower = upper.to_lowercase().next().unwrap_or(upper);
                    u16::try_from(u32::from(lower) & 0xFFFF).unwrap_or(unit)
                })
            };
            let folded_self: Vec<u16> = units.iter().map(|u| fold(*u)).collect();
            let folded_other: Vec<u16> = other.iter().map(|u| fold(*u)).collect();
            Ok(Some(JValue::Int(compare_utf16(
                &folded_self,
                &folded_other,
            ))))
        }
        ("contentEquals", [other]) => {
            let other = arg_units(other)?;
            Ok(Some(JValue::Int(i32::from(units == other))))
        }
        ("hashCode", []) => {
            let mut hash: i32 = 0;
            for unit in &units {
                hash = hash.wrapping_mul(31).wrapping_add(i32::from(*unit));
            }
            Ok(Some(JValue::Int(hash)))
        }
        ("indexOf", [JValue::Int(ch)]) => Ok(Some(JValue::Int(index_of_char(&units, *ch, 0)))),
        ("indexOf", [JValue::Int(ch), JValue::Int(from)]) => {
            Ok(Some(JValue::Int(index_of_char(&units, *ch, *from))))
        }
        ("indexOf", [needle, JValue::Int(from)]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(index_of_from(&units, &needle, *from))))
        }
        ("lastIndexOf", [JValue::Int(ch)]) => {
            Ok(Some(JValue::Int(last_index_of_char(&units, *ch, i32::MAX))))
        }
        ("lastIndexOf", [JValue::Int(ch), JValue::Int(from)]) => {
            Ok(Some(JValue::Int(last_index_of_char(&units, *ch, *from))))
        }
        ("lastIndexOf", [needle]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(last_index_of_from(
                &units,
                &needle,
                i32::MAX,
            ))))
        }
        ("lastIndexOf", [needle, JValue::Int(from)]) => {
            let needle = arg_units(needle)?;
            Ok(Some(JValue::Int(last_index_of_from(
                &units, &needle, *from,
            ))))
        }
        ("split", [delimiter, JValue::Int(limit)]) => {
            let delimiter = arg_units(delimiter)?;
            let parts = split_units_limit(&units, &delimiter, *limit);
            let refs: Vec<JValue> = parts
                .into_iter()
                .map(|part| JValue::Ref(Some(heap.alloc(HeapObject::JavaString(part)))))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(refs));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("startsWith", [prefix, JValue::Int(offset)]) => {
            let prefix = arg_units(prefix)?;
            let starts = usize::try_from(*offset)
                .ok()
                .and_then(|at| units.get(at..))
                .is_some_and(|rest| rest.starts_with(&prefix));
            Ok(Some(JValue::Int(i32::from(starts))))
        }
        ("toCharArray", []) => {
            let values: Vec<i32> = units.iter().map(|u| i32::from(*u)).collect();
            let reference = heap.alloc(HeapObject::IntArray(values));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        (
            "getChars",
            [
                JValue::Int(begin),
                JValue::Int(end),
                JValue::Ref(Some(target)),
                JValue::Int(at),
            ],
        ) => {
            let source = usize::try_from(*begin)
                .ok()
                .zip(usize::try_from(*end).ok())
                .filter(|(b, e)| b <= e && *e <= units.len())
                .map(|(b, e)| units[b..e].to_vec())
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.StringIndexOutOfBoundsException: begin {begin}, end {end}, \
                         length {}",
                        units.len()
                    ))
                })?;
            let at = usize::try_from(*at).map_err(|_| {
                throw(format!(
                    "java.lang.ArrayIndexOutOfBoundsException: Index {at} out of bounds"
                ))
            })?;
            let Some(HeapObject::IntArray(values)) = heap.get_mut(*target) else {
                return Err(throw("java.lang.NullPointerException"));
            };
            if at + source.len() > values.len() {
                let bad = at + source.len() - 1;
                let len = values.len();
                return Err(throw(format!(
                    "java.lang.ArrayIndexOutOfBoundsException: Index {bad} out of bounds \
                     for length {len}"
                )));
            }
            for (index, unit) in source.iter().enumerate() {
                values[at + index] = i32::from(*unit);
            }
            Ok(None)
        }
        ("toString", []) => Ok(Some(JValue::Ref(Some(receiver)))),
        ("intern", []) => {
            let canonical = heap.find_string(&units).unwrap_or(receiver);
            Ok(Some(JValue::Ref(Some(canonical))))
        }
        ("codePointAt", [JValue::Int(index)]) => {
            code_point_at(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointBefore", [JValue::Int(index)]) => {
            let before = index - 1;
            if before < 0 {
                return Err(throw(format!(
                    "java.lang.StringIndexOutOfBoundsException: index {index}"
                )));
            }
            // A low surrogate preceded by a high one forms a pair.
            let at = usize::try_from(before).unwrap_or(usize::MAX);
            if at > 0
                && units.get(at).is_some_and(|u| (0xDC00..0xE000).contains(u))
                && units
                    .get(at - 1)
                    .is_some_and(|u| (0xD800..0xDC00).contains(u))
            {
                return code_point_at(&units, before - 1).map(|cp| Some(JValue::Int(cp)));
            }
            code_point_at(&units, before).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointCount", [JValue::Int(begin), JValue::Int(end)]) => {
            let range = usize::try_from(*begin)
                .ok()
                .zip(usize::try_from(*end).ok())
                .filter(|(b, e)| b <= e && *e <= units.len())
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.IndexOutOfBoundsException: begin {begin}, end {end}, \
                         length {}",
                        units.len()
                    ))
                })?;
            let count = char::decode_utf16(units[range.0..range.1].iter().copied()).count();
            Ok(Some(JValue::Int(i32::try_from(count).unwrap_or(i32::MAX))))
        }
        ("offsetByCodePoints", [JValue::Int(index), JValue::Int(offset)]) => {
            let mut at = usize::try_from(*index)
                .map_err(|_| throw(format!("java.lang.IndexOutOfBoundsException: {index}")))?;
            if at > units.len() {
                return Err(throw(format!(
                    "java.lang.IndexOutOfBoundsException: {index}"
                )));
            }
            let mut remaining = *offset;
            while remaining > 0 {
                if at >= units.len() {
                    return Err(throw(format!(
                        "java.lang.IndexOutOfBoundsException: {offset}"
                    )));
                }
                at += if units.get(at).is_some_and(|u| (0xD800..0xDC00).contains(u))
                    && units
                        .get(at + 1)
                        .is_some_and(|u| (0xDC00..0xE000).contains(u))
                {
                    2
                } else {
                    1
                };
                remaining -= 1;
            }
            while remaining < 0 {
                if at == 0 {
                    return Err(throw(format!(
                        "java.lang.IndexOutOfBoundsException: {offset}"
                    )));
                }
                at -= if at >= 2
                    && units
                        .get(at - 1)
                        .is_some_and(|u| (0xDC00..0xE000).contains(u))
                    && units
                        .get(at - 2)
                        .is_some_and(|u| (0xD800..0xDC00).contains(u))
                {
                    2
                } else {
                    1
                };
                remaining += 1;
            }
            Ok(Some(JValue::Int(i32::try_from(at).unwrap_or(i32::MAX))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("String.{method}"))),
    }
}

/// `String.codePointAt`: the code point at a UTF-16 index (pairs
/// combine; unpaired surrogates return themselves, like Java).
fn code_point_at(units: &[u16], index: i32) -> Result<i32, VmError> {
    let at = usize::try_from(index)
        .ok()
        .filter(|i| *i < units.len())
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: index {index}"
            ))
        })?;
    let unit = units[at];
    if (0xD800..0xDC00).contains(&unit)
        && let Some(low) = units.get(at + 1)
        && (0xDC00..0xE000).contains(low)
    {
        let combined = 0x10000 + ((u32::from(unit) - 0xD800) << 10) + (u32::from(*low) - 0xDC00);
        return Ok(i32::try_from(combined).unwrap_or(i32::MAX));
    }
    Ok(i32::from(unit))
}

/// `indexOf(int ch, int from)` — the char as its UTF-16 encoding.
fn index_of_char(haystack: &[u16], ch: i32, from: i32) -> i32 {
    let Some(encoded) = encode_char(ch) else {
        return -1;
    };
    index_of_from(haystack, &encoded, from)
}

fn last_index_of_char(haystack: &[u16], ch: i32, from: i32) -> i32 {
    let Some(encoded) = encode_char(ch) else {
        return -1;
    };
    last_index_of_from(haystack, &encoded, from)
}

fn encode_char(ch: i32) -> Option<Vec<u16>> {
    let ch = u32::try_from(ch).ok()?;
    let c = char::from_u32(ch)?;
    let mut buffer = [0u16; 2];
    Some(c.encode_utf16(&mut buffer).to_vec())
}

/// `indexOf(needle, fromIndex)` with Java's clamping.
fn index_of_from(haystack: &[u16], needle: &[u16], from: i32) -> i32 {
    let start = usize::try_from(from.max(0)).unwrap_or(0);
    if needle.is_empty() {
        return i32::try_from(start.min(haystack.len())).unwrap_or(i32::MAX);
    }
    if start >= haystack.len() || needle.len() > haystack.len() - start {
        return -1;
    }
    for at in start..=(haystack.len() - needle.len()) {
        if &haystack[at..at + needle.len()] == needle {
            return i32::try_from(at).unwrap_or(i32::MAX);
        }
    }
    -1
}

/// `lastIndexOf(needle, fromIndex)`: rightmost match at or before
/// `from`, with Java's clamping.
fn last_index_of_from(haystack: &[u16], needle: &[u16], from: i32) -> i32 {
    if from < 0 {
        return -1;
    }
    let limit = usize::try_from(from)
        .unwrap_or(usize::MAX)
        .min(haystack.len().saturating_sub(needle.len()));
    if needle.is_empty() {
        return i32::try_from(
            usize::try_from(from)
                .unwrap_or(usize::MAX)
                .min(haystack.len()),
        )
        .unwrap_or(i32::MAX);
    }
    if needle.len() > haystack.len() {
        return -1;
    }
    for at in (0..=limit).rev() {
        if haystack[at..].starts_with(needle) {
            return i32::try_from(at).unwrap_or(i32::MAX);
        }
    }
    -1
}

/// `String.split(delimiter, limit)` with Java's limit semantics over a
/// literal delimiter: positive caps the part count (the last keeps the
/// rest), zero drops trailing empties, negative keeps them.
fn split_units_limit(haystack: &[u16], delimiter: &[u16], limit: i32) -> Vec<Vec<u16>> {
    if limit == 0 {
        return split_units(haystack, delimiter);
    }
    let mut parts: Vec<Vec<u16>> = Vec::new();
    if delimiter.is_empty() {
        for unit in haystack {
            if limit > 0 && i32::try_from(parts.len()).unwrap_or(i32::MAX) == limit - 1 {
                break;
            }
            parts.push(vec![*unit]);
        }
        let consumed: usize = parts.len();
        if consumed < haystack.len() {
            parts.push(haystack[consumed..].to_vec());
        } else if limit < 0 || parts.is_empty() {
            parts.push(Vec::new());
        }
        return parts;
    }
    let mut start = 0;
    let mut at = 0;
    while at + delimiter.len() <= haystack.len() {
        if limit > 0 && i32::try_from(parts.len()).unwrap_or(i32::MAX) == limit - 1 {
            break;
        }
        if &haystack[at..at + delimiter.len()] == delimiter {
            parts.push(haystack[start..at].to_vec());
            at += delimiter.len();
            start = at;
        } else {
            at += 1;
        }
    }
    parts.push(haystack[start..].to_vec());
    parts
}

fn substring(
    heap: &mut Heap,
    units: &[u16],
    begin: i32,
    len: i32,
) -> Result<Option<JValue>, VmError> {
    let begin_usize = usize::try_from(begin).ok().filter(|b| *b <= units.len());
    let Some(begin_usize) = begin_usize else {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: begin {begin}, end {len}, length {len}"
        )));
    };
    let reference = heap.alloc(HeapObject::JavaString(units[begin_usize..].to_vec()));
    Ok(Some(JValue::Ref(Some(reference))))
}

fn substring_range(
    heap: &mut Heap,
    units: &[u16],
    begin: i32,
    end: i32,
) -> Result<Option<JValue>, VmError> {
    let length = units.len();
    let valid = usize::try_from(begin)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(b, e)| b <= e && *e <= length);
    let Some((begin_usize, end_usize)) = valid else {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: begin {begin}, end {end}, \
             length {length}"
        )));
    };
    let reference = heap.alloc(HeapObject::JavaString(
        units[begin_usize..end_usize].to_vec(),
    ));
    Ok(Some(JValue::Ref(Some(reference))))
}

/// `String.indexOf(String)` over UTF-16 units (-1 when absent).
fn index_of(haystack: &[u16], needle: &[u16]) -> i32 {
    if needle.is_empty() {
        return 0;
    }
    if needle.len() > haystack.len() {
        return -1;
    }
    for start in 0..=(haystack.len() - needle.len()) {
        if &haystack[start..start + needle.len()] == needle {
            return i32::try_from(start).unwrap_or(i32::MAX);
        }
    }
    -1
}

/// `String.compareTo` semantics: difference of first differing code
/// unit, else length difference.
fn compare_utf16(a: &[u16], b: &[u16]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return i32::from(*x) - i32::from(*y);
        }
    }
    i32::try_from(a.len()).unwrap_or(i32::MAX) - i32::try_from(b.len()).unwrap_or(i32::MAX)
}

/// `String.split` with a literal delimiter. Matches Java's default
/// behavior of dropping trailing empty strings; an empty delimiter
/// splits into single code units (as Java's empty regex does). The
/// delimiter is NOT a regex — a documented deviation that only shows
/// for metacharacter delimiters like `"."`.
fn split_units(haystack: &[u16], delimiter: &[u16]) -> Vec<Vec<u16>> {
    let mut parts: Vec<Vec<u16>> = Vec::new();
    if delimiter.is_empty() {
        parts.extend(haystack.iter().map(|unit| vec![*unit]));
    } else {
        let mut start = 0;
        let mut at = 0;
        while at + delimiter.len() <= haystack.len() {
            if &haystack[at..at + delimiter.len()] == delimiter {
                parts.push(haystack[start..at].to_vec());
                at += delimiter.len();
                start = at;
            } else {
                at += 1;
            }
        }
        parts.push(haystack[start..].to_vec());
    }
    // Java (limit 0) removes trailing empty strings.
    while parts.last().is_some_and(Vec::is_empty) {
        parts.pop();
    }
    parts
}

/// `String.replace(CharSequence, CharSequence)` — literal in Java too.
fn replace_units(haystack: &[u16], from: &[u16], to: &[u16]) -> Vec<u16> {
    if from.is_empty() {
        // Java inserts `to` between every character.
        let mut out = to.to_vec();
        for unit in haystack {
            out.push(*unit);
            out.extend_from_slice(to);
        }
        return out;
    }
    let mut out = Vec::with_capacity(haystack.len());
    let mut at = 0;
    while at < haystack.len() {
        if at + from.len() <= haystack.len() && &haystack[at..at + from.len()] == from {
            out.extend_from_slice(to);
            at += from.len();
        } else {
            out.push(haystack[at]);
            at += 1;
        }
    }
    out
}

/// `java.util.Scanner` methods, pulling lines from the console on
/// demand. Tokens are whitespace-delimited (Java's default).
fn scanner_method(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    method: &str,
) -> Result<Option<JValue>, VmError> {
    match method {
        "nextLine" => {
            let line = scanner_next_line(heap, console, receiver)?
                .ok_or_else(|| throw("java.util.NoSuchElementException: No line found"))?;
            let reference = heap.alloc_string(&line);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        "hasNextLine" => {
            let has = scanner_peek_line(heap, console, receiver)?;
            Ok(Some(JValue::Int(i32::from(has))))
        }
        "next" => {
            let token = scanner_next_token(heap, console, receiver)?
                .ok_or_else(|| throw("java.util.NoSuchElementException"))?;
            let reference = heap.alloc_string(&token);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        "hasNext" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            Ok(Some(JValue::Int(i32::from(token.is_some()))))
        }
        "nextInt" => {
            let token = scanner_next_token(heap, console, receiver)?
                .ok_or_else(|| throw("java.util.NoSuchElementException"))?;
            let value: i32 = token
                .parse()
                .map_err(|_| throw("java.util.InputMismatchException"))?;
            Ok(Some(JValue::Int(value)))
        }
        "hasNextInt" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| t.parse::<i32>().is_ok());
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextDouble" => {
            let token = scanner_next_token(heap, console, receiver)?
                .ok_or_else(|| throw("java.util.NoSuchElementException"))?;
            let value: f64 = token
                .parse()
                .map_err(|_| throw("java.util.InputMismatchException"))?;
            Ok(Some(JValue::Double(value)))
        }
        "hasNextDouble" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| t.parse::<f64>().is_ok());
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Scanner.{method}"))),
    }
}

/// Pull one more line of input into the scanner's buffer. Returns
/// whether a line was added.
fn scanner_fill(heap: &mut Heap, console: &mut dyn ConsoleIo, receiver: HeapRef) {
    let line = console.read_line();
    let Some(HeapObject::Scanner { buffer, eof, .. }) = heap.get_mut(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    if let Some(line) = line {
        buffer.push_str(&line);
        buffer.push('\n');
    } else {
        *eof = true;
    }
}

fn scanner_state(heap: &Heap, receiver: HeapRef) -> (String, usize, bool) {
    match heap.get(receiver) {
        Some(HeapObject::Scanner { buffer, pos, eof }) => (buffer.clone(), *pos, *eof),
        _ => unreachable!("receiver kind checked by caller"),
    }
}

fn scanner_set_pos(heap: &mut Heap, receiver: HeapRef, new_pos: usize) {
    if let Some(HeapObject::Scanner { pos, .. }) = heap.get_mut(receiver) {
        *pos = new_pos;
    }
}

/// Read up to the next newline (consuming it); `None` at EOF.
fn scanner_next_line(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<String>, VmError> {
    loop {
        let (buffer, pos, eof) = scanner_state(heap, receiver);
        if let Some(offset) = buffer[pos..].find('\n') {
            let line = buffer[pos..pos + offset].to_owned();
            scanner_set_pos(heap, receiver, pos + offset + 1);
            return Ok(Some(line));
        }
        if eof {
            // Trailing text without a newline still counts as a line.
            if pos < buffer.len() {
                let line = buffer[pos..].to_owned();
                scanner_set_pos(heap, receiver, buffer.len());
                return Ok(Some(line));
            }
            return Ok(None);
        }
        scanner_fill(heap, console, receiver);
    }
}

fn scanner_peek_line(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<bool, VmError> {
    loop {
        let (buffer, pos, eof) = scanner_state(heap, receiver);
        if buffer[pos..].contains('\n') || (eof && pos < buffer.len()) {
            return Ok(true);
        }
        if eof {
            return Ok(false);
        }
        scanner_fill(heap, console, receiver);
    }
}

/// Advance past whitespace and read one token; `None` at EOF.
fn scanner_next_token(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<String>, VmError> {
    let token = scanner_peek_token(heap, console, receiver)?;
    if let Some(token) = &token {
        let (buffer, pos, _) = scanner_state(heap, receiver);
        let skip = buffer[pos..]
            .find(token.as_str())
            .expect("peeked token is present");
        scanner_set_pos(heap, receiver, pos + skip + token.len());
    }
    Ok(token)
}

fn scanner_peek_token(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<String>, VmError> {
    loop {
        let (buffer, pos, eof) = scanner_state(heap, receiver);
        let rest = &buffer[pos..];
        let trimmed = rest.trim_start();
        if !trimmed.is_empty() {
            // A complete token needs trailing whitespace or EOF.
            let token: String = trimmed.chars().take_while(|c| !c.is_whitespace()).collect();
            if trimmed.len() > token.len() || eof {
                return Ok(Some(token));
            }
        }
        if eof {
            return Ok(None);
        }
        scanner_fill(heap, console, receiver);
    }
}

/// `java.util.ArrayList` methods (values stored unboxed).
fn list_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let list_len = match heap.get(receiver) {
        Some(HeapObject::ArrayList(values)) => values.len(),
        _ => unreachable!("receiver kind checked by caller"),
    };
    let check = |index: i32, limit: usize| -> Result<usize, VmError> {
        usize::try_from(index)
            .ok()
            .filter(|i| *i < limit)
            .ok_or_else(|| {
                throw(format!(
                    "java.lang.IndexOutOfBoundsException: Index {index} out of bounds for \
                     length {list_len}"
                ))
            })
    };
    match (method, descriptor, args) {
        ("size", _, []) => Ok(Some(JValue::Int(
            i32::try_from(list_len).unwrap_or(i32::MAX),
        ))),
        ("isEmpty", _, []) => Ok(Some(JValue::Int(i32::from(list_len == 0)))),
        ("add", "(Ljava/lang/Object;)Z", [value]) => {
            let value = *value;
            if let Some(HeapObject::ArrayList(values)) = heap.get_mut(receiver) {
                values.push(value);
            }
            Ok(Some(JValue::Int(1)))
        }
        ("add", "(ILjava/lang/Object;)V", [JValue::Int(index), value]) => {
            // Insertion allows index == size.
            let at = usize::try_from(*index)
                .ok()
                .filter(|i| *i <= list_len)
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.IndexOutOfBoundsException: Index {index} out of bounds \
                         for length {list_len}"
                    ))
                })?;
            let value = *value;
            if let Some(HeapObject::ArrayList(values)) = heap.get_mut(receiver) {
                values.insert(at, value);
            }
            Ok(None)
        }
        ("get", _, [JValue::Int(index)]) => {
            let at = check(*index, list_len)?;
            match heap.get(receiver) {
                Some(HeapObject::ArrayList(values)) => Ok(Some(values[at])),
                _ => unreachable!(),
            }
        }
        ("set", _, [JValue::Int(index), value]) => {
            let at = check(*index, list_len)?;
            let value = *value;
            let Some(HeapObject::ArrayList(values)) = heap.get_mut(receiver) else {
                unreachable!()
            };
            let previous = values[at];
            values[at] = value;
            Ok(Some(previous))
        }
        ("remove", _, [JValue::Int(index)]) => {
            let at = check(*index, list_len)?;
            let Some(HeapObject::ArrayList(values)) = heap.get_mut(receiver) else {
                unreachable!()
            };
            Ok(Some(values.remove(at)))
        }
        ("toString", _, []) => {
            let values = match heap.get(receiver) {
                Some(HeapObject::ArrayList(values)) => values.clone(),
                _ => unreachable!(),
            };
            let mut parts = Vec::with_capacity(values.len());
            for value in values {
                parts.push(display_jvalue(heap, value));
            }
            let text = format!("[{}]", parts.join(", "));
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "ArrayList.{method}{descriptor}"
        ))),
    }
}

/// Best-effort display of a stored value for `ArrayList.toString`
/// (ints print as ints; the VM cannot distinguish boxed Character /
/// Boolean, a documented deviation).
fn display_jvalue(heap: &Heap, value: JValue) -> String {
    match value {
        JValue::Int(v) => v.to_string(),
        JValue::Long(v) => v.to_string(),
        JValue::Float(v) => v.to_string(),
        JValue::Double(v) => java_double_to_string(v),
        JValue::Ref(None) => String::from("null"),
        JValue::Ref(Some(reference)) => match heap.get(reference) {
            Some(HeapObject::JavaString(units)) => String::from_utf16_lossy(units),
            Some(HeapObject::Instance { class_name, .. }) => {
                // User toString would need re-entering the interpreter;
                // fall back to the default form here.
                format!("{class_name}@{reference:x}")
            }
            _ => String::from("<object>"),
        },
    }
}

/// `java.io.File` methods over the virtual filesystem.
fn file_method(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    receiver: HeapRef,
    method: &str,
) -> Result<Option<JValue>, VmError> {
    let path = match heap.get(receiver) {
        Some(HeapObject::File(path)) => path.clone(),
        _ => unreachable!("receiver kind checked by caller"),
    };
    let boolean = |b: bool| Ok(Some(JValue::Int(i32::from(b))));
    match method {
        "exists" => boolean(vfs.exists(&path)),
        "isFile" => boolean(vfs.is_file(&path)),
        "isDirectory" => boolean(vfs.is_directory(&path)),
        "delete" => boolean(vfs.remove(&path).is_ok()),
        "mkdir" => boolean(!vfs.exists(&path) && vfs.mkdir(&path).is_ok()),
        "createNewFile" => {
            if vfs.exists(&path) {
                boolean(false)
            } else {
                boolean(vfs.write_file(&path, Vec::new()).is_ok())
            }
        }
        // Java returns long; jvmjs surfaces int (virtual files are small).
        "length" => Ok(Some(JValue::Int(
            i32::try_from(vfs.len(&path)).unwrap_or(i32::MAX),
        ))),
        "getName" => {
            let normalized = VirtualFileSystem::normalize(&path);
            let name = normalized.rsplit('/').next().unwrap_or_default();
            let reference = heap.alloc_string(name);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // getPath / toString return the path as written.
        "getPath" | "toString" => {
            let reference = heap.alloc_string(&path);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("File.{method}"))),
    }
}

/// `java.io.PrintWriter` methods: formatting matches `PrintStream`, but
/// output appends to the writer's file in the virtual filesystem.
fn writer_method(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    receiver: HeapRef,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let path = match heap.get(receiver) {
        Some(HeapObject::Writer { path }) => path.clone(),
        _ => unreachable!("receiver kind checked by caller"),
    };
    match method {
        "print" | "println" => {
            let mut text = print_argument_text(heap, descriptor, args)?;
            if method == "println" {
                text.push('\n');
            }
            vfs.append_file(&path, text.as_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(None)
        }
        // Write-through means close/flush have nothing left to do.
        "close" | "flush" => Ok(None),
        _ => Err(VmError::UnknownIntrinsic(format!("PrintWriter.{method}"))),
    }
}

/// Java's `java.util.Random` LCG, used for `Math.random()`.
#[derive(Debug)]
pub struct JavaRng {
    seed: u64,
}

impl JavaRng {
    const MULTIPLIER: u64 = 0x5_DEEC_E66D;
    const MASK: u64 = (1 << 48) - 1;

    #[must_use]
    pub fn new(seed: Option<u64>) -> Self {
        let seed = seed.unwrap_or(0x5EED_1234_5678);
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: u32) -> u64 {
        self.seed = self.seed.wrapping_mul(Self::MULTIPLIER).wrapping_add(0xB) & Self::MASK;
        self.seed >> (48 - bits)
    }

    /// `Random.nextDouble()`: uniform in `[0, 1)`.
    #[allow(clippy::cast_precision_loss)] // both values are < 2^53
    pub fn next_double(&mut self) -> f64 {
        let high = self.next(26) << 27;
        let low = self.next(27);
        ((high + low) as f64) / ((1u64 << 53) as f64)
    }
}

/// Intrinsic static methods (`Math.*`, `Integer.parseInt`,
/// `Double.parseDouble`).
#[allow(clippy::too_many_lines)] // one arm per static intrinsic
pub fn invoke_static(
    heap: &mut Heap,
    rng: &mut JavaRng,
    class: &str,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let string_arg = |value: &JValue| -> Result<String, VmError> {
        match value {
            JValue::Ref(Some(reference)) => heap
                .string_text(*reference)
                .ok_or_else(|| throw("java.lang.ClassCastException: not a String")),
            JValue::Ref(None) => Err(throw(
                "java.lang.NumberFormatException: Cannot parse null string",
            )),
            _ => Err(throw("java.lang.VerifyError: expected a String argument")),
        }
    };

    match (class, method, args) {
        ("java/lang/Math", "abs", [JValue::Int(v)]) => Ok(Some(JValue::Int(v.wrapping_abs()))),
        ("java/lang/Math", "abs", [JValue::Double(v)]) => Ok(Some(JValue::Double(v.abs()))),
        ("java/lang/Math", "sqrt", [JValue::Double(v)]) => Ok(Some(JValue::Double(v.sqrt()))),
        ("java/lang/Math", "pow", [JValue::Double(a), JValue::Double(b)]) => {
            Ok(Some(JValue::Double(a.powf(*b))))
        }
        ("java/lang/Math", "max", [JValue::Int(a), JValue::Int(b)]) => {
            Ok(Some(JValue::Int((*a).max(*b))))
        }
        ("java/lang/Math", "max", [JValue::Double(a), JValue::Double(b)]) => {
            Ok(Some(JValue::Double(a.max(*b))))
        }
        ("java/lang/Math", "min", [JValue::Int(a), JValue::Int(b)]) => {
            Ok(Some(JValue::Int((*a).min(*b))))
        }
        ("java/lang/Math", "min", [JValue::Double(a), JValue::Double(b)]) => {
            Ok(Some(JValue::Double(a.min(*b))))
        }
        ("java/lang/Math", "random", []) => Ok(Some(JValue::Double(rng.next_double()))),
        ("java/lang/Math", "floor", [JValue::Double(v)]) => Ok(Some(JValue::Double(v.floor()))),
        ("java/lang/Math", "ceil", [JValue::Double(v)]) => Ok(Some(JValue::Double(v.ceil()))),
        ("java/lang/Math", "round", [JValue::Double(v)]) => {
            // Java rounds half-up toward positive infinity.
            #[allow(clippy::cast_possible_truncation)]
            Ok(Some(JValue::Int((v + 0.5).floor() as i32)))
        }
        ("java/lang/Integer", "toString", [JValue::Int(v)]) => {
            let reference = heap.alloc_string(&v.to_string());
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("java/lang/Double", "toString", [JValue::Double(v)]) => {
            let reference = heap.alloc_string(&java_double_to_string(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("java/lang/Character", method, [JValue::Int(unit)]) => {
            let ch = char::from_u32(u32::try_from(*unit).unwrap_or(0)).unwrap_or('\u{FFFD}');
            match method {
                "isDigit" => Ok(Some(JValue::Int(i32::from(ch.is_ascii_digit())))),
                "isLetter" => Ok(Some(JValue::Int(i32::from(ch.is_alphabetic())))),
                "isLetterOrDigit" => Ok(Some(JValue::Int(i32::from(
                    ch.is_alphabetic() || ch.is_ascii_digit(),
                )))),
                "isUpperCase" => Ok(Some(JValue::Int(i32::from(ch.is_uppercase())))),
                "isLowerCase" => Ok(Some(JValue::Int(i32::from(ch.is_lowercase())))),
                "toUpperCase" | "toLowerCase" => {
                    let converted = if method == "toUpperCase" {
                        ch.to_uppercase().next().unwrap_or(ch)
                    } else {
                        ch.to_lowercase().next().unwrap_or(ch)
                    };
                    let unit = u32::from(converted);
                    Ok(Some(JValue::Int(i32::try_from(unit & 0xFFFF).unwrap_or(0))))
                }
                _ => Err(VmError::UnknownIntrinsic(format!("Character.{method}"))),
            }
        }
        ("java/lang/String", "valueOf" | "copyValueOf", [value]) => {
            // The descriptor disambiguates int/char/boolean, which all
            // arrive as JValue::Int.
            let text = match (descriptor, value) {
                ("(C)Ljava/lang/String;", JValue::Int(v)) => {
                    char::from_u32(u32::try_from(*v).unwrap_or(0))
                        .unwrap_or('\u{FFFD}')
                        .to_string()
                }
                ("(Z)Ljava/lang/String;", JValue::Int(v)) => {
                    String::from(if *v != 0 { "true" } else { "false" })
                }
                (_, JValue::Int(v)) => v.to_string(),
                (_, JValue::Double(v)) => java_double_to_string(*v),
                (_, JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::IntArray(values)) => values
                        .iter()
                        .map(|v| {
                            char::from_u32(u32::try_from(*v).unwrap_or(0)).unwrap_or('\u{FFFD}')
                        })
                        .collect(),
                    _ => heap.string_text(*reference).unwrap_or_default(),
                },
                (_, JValue::Ref(None)) => String::from("null"),
                _ => String::new(),
            };
            let reference = heap.alloc_string(&text);
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("java/lang/Integer", "parseInt", [value]) => {
            let text = string_arg(value)?;
            let parsed: i32 = text.parse().map_err(|_| {
                throw(format!(
                    "java.lang.NumberFormatException: For input string: \"{text}\""
                ))
            })?;
            Ok(Some(JValue::Int(parsed)))
        }
        ("java/lang/Double", "parseDouble", [value]) => {
            let text = string_arg(value)?;
            let parsed: f64 = text.trim().parse().map_err(|_| {
                throw(format!(
                    "java.lang.NumberFormatException: For input string: \"{text}\""
                ))
            })?;
            Ok(Some(JValue::Double(parsed)))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("{class}.{method}"))),
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
pub(crate) fn java_double_to_string(value: f64) -> String {
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
        let mut vfs = VirtualFileSystem::new();

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
            &mut vfs,
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
        let mut vfs = VirtualFileSystem::new();
        let JValue::Ref(Some(err)) = statics
            .static_field(&mut heap, "java/lang/System", "err")
            .unwrap()
        else {
            panic!("expected a reference");
        };
        invoke_virtual(
            &mut heap,
            &mut console,
            &mut vfs,
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
        let mut vfs = VirtualFileSystem::new();
        let builder =
            heap.alloc(instantiate("java/lang/StringBuilder").expect("known intrinsic class"));
        invoke_special(
            &mut heap,
            &mut vfs,
            builder,
            "java/lang/StringBuilder",
            "<init>",
            "()V",
            &[],
        )
        .unwrap();

        let hello = heap.alloc_string("x = ");
        invoke_virtual(
            &mut heap,
            &mut console,
            &mut vfs,
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
            &mut vfs,
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
            &mut vfs,
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
