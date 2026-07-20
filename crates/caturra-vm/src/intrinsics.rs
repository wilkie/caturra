//! Intrinsic ("native") classes: core library members whose semantics
//! live in Rust rather than in interpreted bytecode.
//!
//! Resolution order per `specs/SCOPE.md`: intrinsics → baked-in classlib
//! → user classes. v0 covers `java.lang.System.out`/`err` and
//! `java.io.PrintStream.print`/`println`; the surface grows with
//! `specs/LANGUAGE.md` staging.

use crate::io::ConsoleIo;
use crate::map::JavaHashMap;
use crate::value::{Heap, HeapObject, HeapRef, IntKind, JValue, MapViewKind, StdStream};
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
        // A bare `new Object()` — an identity-only object (used e.g. to test
        // that `equals` distinguishes an unrelated instance).
        "java/lang/Object" => Some(HeapObject::Instance {
            class_name: std::rc::Rc::from("java/lang/Object"),
            layout: std::rc::Rc::new(crate::value::ClassLayout::default()),
            fields: Vec::new(),
        }),
        "java/lang/String" => Some(HeapObject::JavaString(Vec::new())),
        "java/lang/StringBuilder" => Some(HeapObject::StringBuilder(Vec::new())),
        "java/util/Scanner" => Some(HeapObject::Scanner {
            buffer: String::new(),
            pos: 0,
            eof: false,
            // Until a `File` claims it, a Scanner reads standard input.
            stdin: true,
            closed: false,
        }),
        "java/util/ArrayList" => Some(HeapObject::ArrayList(Vec::new())),
        "java/util/LinkedList" => Some(HeapObject::LinkedList(Vec::new())),
        "java/util/ArrayDeque" => Some(HeapObject::ArrayDeque(Vec::new())),
        "java/util/Stack" => Some(HeapObject::Stack(Vec::new())),
        "java/util/HashMap" => Some(HeapObject::HashMap(JavaHashMap::new())),
        "java/util/HashSet" => Some(HeapObject::HashSet(JavaHashMap::new())),
        "java/util/TreeSet" => Some(HeapObject::TreeSet {
            values: Vec::new(),
            comparator: None,
        }),
        "java/util/PriorityQueue" => Some(HeapObject::PriorityQueue {
            heap: Vec::new(),
            comparator: None,
        }),
        "java/util/TreeMap" => Some(HeapObject::TreeMap {
            entries: Vec::new(),
            comparator: None,
        }),
        "java/io/File" => Some(HeapObject::File(String::new())),
        "java/io/PrintWriter" => Some(HeapObject::Writer {
            path: String::new(),
        }),
        "java/io/BufferedReader" | "java/io/FileReader" | "java/io/InputStreamReader" => {
            Some(HeapObject::Reader {
                buffer: String::new(),
                pos: 0,
                stdin: false,
                closed: false,
            })
        }
        _ => {
            if caturra_classfile::exceptions::is_exception_class(class) {
                return Some(HeapObject::Exception {
                    class_name: caturra_classfile::exceptions::dotted(class),
                    message: None,
                    cause: None,
                    suppressed: Vec::new(),
                });
            }
            None
        }
    }
}

/// Invoke an intrinsic constructor (`invokespecial <init>`).
#[allow(clippy::too_many_lines)]
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
        ("<init>", "(Ljava/io/InputStream;)V")
            if matches!(heap.get(receiver), Some(HeapObject::Reader { .. })) =>
        {
            // `new InputStreamReader(System.in)` — read from standard input.
            if let Some(HeapObject::Reader { stdin, .. }) = heap.get_mut(receiver) {
                *stdin = true;
            }
            Ok(())
        }
        ("<init>", "()V" | "(Ljava/io/InputStream;)V") => Ok(()),
        // `new BufferedReader(reader)` — wrap the reader, inheriting its state.
        ("<init>", "(Ljava/io/Reader;)V") => {
            let JValue::Ref(Some(inner)) = args[0] else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let wrapped = match heap.get(inner) {
                Some(HeapObject::Reader {
                    buffer,
                    pos,
                    stdin,
                    closed,
                }) => (buffer.clone(), *pos, *stdin, *closed),
                _ => return Err(throw("java.lang.ClassCastException: not a Reader")),
            };
            if let Some(HeapObject::Reader {
                buffer,
                pos,
                stdin,
                closed,
            }) = heap.get_mut(receiver)
            {
                *buffer = wrapped.0;
                *pos = wrapped.1;
                *stdin = wrapped.2;
                *closed = wrapped.3;
            }
            Ok(())
        }
        // `new StringBuilder(capacity)`: a sizing hint with no observable
        // effect — caturra models contents, not the backing array — EXCEPT
        // that a negative capacity throws, as it allocates a `char[capacity]`.
        ("<init>", "(I)V") if matches!(heap.get(receiver), Some(HeapObject::StringBuilder(_))) => {
            if let JValue::Int(capacity) = args[0]
                && capacity < 0
            {
                return Err(throw(format!(
                    "java.lang.NegativeArraySizeException: {capacity}"
                )));
            }
            Ok(())
        }
        // `new HashMap<>(initialCapacity)`: unlike a builder's, a map's
        // capacity IS observable — it sets the table length, and so the
        // iteration order.
        ("<init>", "(I)V") => {
            let JValue::Int(capacity) = args[0] else {
                return Err(throw("java.lang.VerifyError: expected an int argument"));
            };
            if capacity < 0 {
                return Err(throw(format!(
                    "java.lang.IllegalArgumentException: Illegal initial capacity: {capacity}"
                )));
            }
            match heap.get_mut(receiver) {
                // `new HashSet<>(initialCapacity)` builds `new HashMap<>(cap)`,
                // so the hint reaches the backing map identically.
                Some(HeapObject::HashMap(map) | HeapObject::HashSet(map)) => {
                    *map = JavaHashMap::with_capacity_hint(capacity);
                    Ok(())
                }
                _ => Ok(()),
            }
        }
        // `new HashMap<>(otherMap)`: copies the entries; the copy's own
        // table is sized to fit, so its order may differ from the source's.
        ("<init>", "(Ljava/util/Map;)V") => {
            let JValue::Ref(Some(source)) = args[0] else {
                return Err(throw("java.lang.NullPointerException"));
            };
            let entries = match heap.get(source) {
                Some(HeapObject::HashMap(map)) => map.hashed_entries_in_order(),
                _ => return Err(throw("java.lang.ClassCastException: not a Map")),
            };
            let mut copy = JavaHashMap::sized_for(entries.len());
            for (hash, key, value) in entries {
                copy.insert_new(hash, key, value);
            }
            match heap.get_mut(receiver) {
                Some(HeapObject::HashMap(map)) => *map = copy,
                _ => return Err(throw("java.lang.ClassCastException: not a Map")),
            }
            Ok(())
        }
        ("<init>", "(Ljava/lang/String;)V") => {
            // `new StringBuilder(otherBuilder)` reaches here too (a
            // StringBuilder IS a CharSequence), so read the argument's chars
            // whether it is a String or a builder.
            let text = match &args[0] {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                        String::from_utf16_lossy(units)
                    }
                    _ => return Err(throw("java.lang.ClassCastException: not a String")),
                },
                JValue::Ref(None) => return Err(throw("java.lang.NullPointerException")),
                _ => return Err(throw("java.lang.VerifyError: expected a String argument")),
            };
            match heap.get_mut(receiver) {
                // `new String(String)` / `new StringBuilder(String)`: seed
                // with a fresh copy of the chars (both store UTF-16 units).
                Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                    *units = text.encode_utf16().collect();
                    Ok(())
                }
                Some(HeapObject::File(path)) => {
                    *path = text;
                    Ok(())
                }
                Some(HeapObject::Scanner { .. }) => {
                    // `new Scanner(String)`: the whole literal is the source,
                    // fully present (eof), and closing it must not touch stdin.
                    if let Some(HeapObject::Scanner {
                        buffer,
                        pos,
                        eof,
                        stdin,
                        ..
                    }) = heap.get_mut(receiver)
                    {
                        *buffer = text;
                        *pos = 0;
                        *eof = true;
                        *stdin = false;
                    }
                    Ok(())
                }
                Some(HeapObject::Exception { message, .. }) => {
                    *message = Some(text);
                    Ok(())
                }
                // A user exception class chaining `super("message")`
                // into its library throwable parent: stash the message
                // in the slot its layout reserved for it.
                Some(HeapObject::Instance { .. })
                    if caturra_classfile::exceptions::is_exception_class(class) =>
                {
                    let reference = heap.alloc_string(&text);
                    if let Some(field) = heap
                        .get_mut(receiver)
                        .and_then(|object| object.field_mut("__message"))
                    {
                        *field = JValue::Ref(Some(reference));
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
                Some(HeapObject::Reader {
                    buffer,
                    pos,
                    stdin,
                    closed,
                }) => {
                    // FileReader(String): slurp the whole file up front.
                    let content = vfs.read_file(&text).map_err(|_| {
                        throw(format!(
                            "java.io.FileNotFoundException: {text} (No such file or directory)"
                        ))
                    })?;
                    *buffer = String::from_utf8_lossy(content).into_owned();
                    *pos = 0;
                    *stdin = false;
                    *closed = false;
                    Ok(())
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                ))),
            }
        }
        // Exception chaining. `new X(message, cause)` stores both; `new
        // X(cause)` derives the message from the cause's toString, as Java's
        // `Throwable(Throwable)` does. `super(...)` from a user exception class
        // into its library parent lands here too (a user Instance receiver).
        ("<init>", "(Ljava/lang/String;Ljava/lang/Throwable;)V")
            if caturra_classfile::exceptions::is_exception_class(class) =>
        {
            let text = string_arg(heap, &args[0])?;
            let cause = ref_arg(&args[1]);
            set_exception_cause(heap, receiver, class, Some(text), cause);
            Ok(())
        }
        ("<init>", "(Ljava/lang/Throwable;)V")
            if caturra_classfile::exceptions::is_exception_class(class) =>
        {
            let cause = ref_arg(&args[0]);
            let message = cause.map(|c| throwable_to_string(heap, c));
            set_exception_cause(heap, receiver, class, message, cause);
            Ok(())
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
                    if let Some(HeapObject::Scanner {
                        buffer,
                        eof,
                        pos,
                        stdin,
                        ..
                    }) = heap.get_mut(receiver)
                    {
                        *buffer = text;
                        *pos = 0;
                        *eof = true;
                        // A file scanner: closing it must not close stdin.
                        *stdin = false;
                    }
                    Ok(())
                }
                Some(HeapObject::Reader { .. }) => {
                    // FileReader(File): slurp the whole file up front.
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
                    if let Some(HeapObject::Reader {
                        buffer,
                        pos,
                        stdin,
                        closed,
                    }) = heap.get_mut(receiver)
                    {
                        *buffer = text;
                        *pos = 0;
                        *stdin = false;
                        *closed = false;
                    }
                    Ok(())
                }
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                ))),
            }
        }
        // `new String(char[])` — char arrays are stored as int arrays.
        ("<init>", "([C)V") => {
            let units: Vec<u16> = match &args[0] {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::IntArray(_, values)) => values
                        .iter()
                        .map(|v| u16::try_from(v & 0xFFFF).unwrap_or(0))
                        .collect(),
                    _ => return Err(throw("java.lang.ClassCastException: not a char[]")),
                },
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            if let Some(HeapObject::JavaString(target)) = heap.get_mut(receiver) {
                *target = units;
                Ok(())
            } else {
                Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                )))
            }
        }
        // `new ArrayList<>(collection)` — copy the source collection's items.
        ("<init>", "(Ljava/util/Collection;)V") => {
            let items = match args.first() {
                // `list_values` reads through an `Arrays.asList` view too, so
                // `new ArrayList<>(Arrays.asList(...))` — the standard way to
                // get a mutable copy — works. (The richer collection sources
                // are handled by the interpreter's own copy path; this arm
                // only sees the list-shaped ones.)
                Some(JValue::Ref(Some(reference))) => match heap.list_values(*reference) {
                    Some(items) => items.clone(),
                    None => return Err(throw("java.lang.ClassCastException: not a Collection")),
                },
                Some(JValue::Ref(None)) => return Err(throw("java.lang.NullPointerException")),
                _ => Vec::new(),
            };
            if let Some(HeapObject::ArrayList(target)) = heap.get_mut(receiver) {
                *target = items;
                Ok(())
            } else {
                Err(VmError::UnknownIntrinsic(format!(
                    "{class}.{method}{descriptor}"
                )))
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
#[allow(clippy::too_many_lines)] // one intrinsic-dispatch matrix
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

    // Boxed wrapper methods: unboxing accessors and Object methods.
    if let HeapObject::Boxed { class_name, value } = receiver_object {
        let class_name = class_name.clone();
        let value = *value;
        return boxed_virtual(heap, &class_name, value, method, args);
    }
    match (receiver_object, method) {
        (HeapObject::PrintStream(stream), "printf") => {
            let stream = *stream;
            let template = match args.first() {
                Some(JValue::Ref(Some(reference))) => {
                    heap.string_text(*reference).unwrap_or_default()
                }
                Some(JValue::Ref(None)) => {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException: format is null",
                    )));
                }
                _ => String::new(),
            };
            let format_args = crate::format::args_from_descriptor(descriptor, &args[1..])?;
            let text = crate::format::java_format(heap, &template, &format_args)?;
            match stream {
                StdStream::Out => console.stdout(text.as_bytes()),
                StdStream::Err => console.stderr(text.as_bytes()),
            }
            Ok(None)
        }
        (HeapObject::PrintStream(stream), "print" | "println") => {
            let stream = *stream;
            let text = print_argument_text(heap, descriptor, args)?;
            // While `SystemOutTestRunner` is capturing, each print/println call
            // on `System.out` is one message (the argument, without the
            // println newline) — matching javabuilder's per-call messages.
            if stream == StdStream::Out && console.capturing() {
                console.capture_message(&text);
            } else {
                let mut out = text;
                if method == "println" {
                    out.push('\n');
                }
                match stream {
                    StdStream::Out => console.stdout(out.as_bytes()),
                    StdStream::Err => console.stderr(out.as_bytes()),
                }
            }
            Ok(None)
        }
        (HeapObject::Iterator { .. }, _) => iterator_method(heap, receiver, method),
        (HeapObject::StringBuilder(_), _) => {
            builder_method(heap, receiver, method, descriptor, args)
        }
        (HeapObject::JavaString(_), _) => string_method(heap, receiver, method, args),
        (HeapObject::Scanner { .. }, _) => scanner_method(heap, console, receiver, method),
        (HeapObject::Reader { .. }, _) => reader_method(heap, console, receiver, method),
        (
            HeapObject::ArrayList(_) | HeapObject::LinkedList(_) | HeapObject::ArrayBackedList(_),
            _,
        ) => list_method(heap, receiver, method, descriptor, args),
        // An ArrayDeque shares the LinkedList `Deque` semantics (head-based
        // push/pop through `list_method`) but forbids null elements, so guard
        // every insertion before delegating.
        (HeapObject::ArrayDeque(_), _) => {
            array_deque_reject_null(heap, method, args)?;
            list_method(heap, receiver, method, descriptor, args)
        }
        // A `Stack` is a `List`, so it shares `list_method` for everything but
        // the five LIFO operations, whose `top`-end semantics and
        // `EmptyStackException` differ from the deque/list methods of the same
        // spelling.
        (HeapObject::Stack(_), _) => match stack_method(heap, receiver, method, args)? {
            Some(value) => Ok(Some(value)),
            None => list_method(heap, receiver, method, descriptor, args),
        },
        (HeapObject::File(_), _) => file_method(heap, vfs, receiver, method),
        (HeapObject::Path(_), _) => path_method(heap, receiver, method),
        (
            HeapObject::Exception {
                class_name,
                message,
                ..
            },
            "printStackTrace",
        ) => {
            // Real Java writes the trace to System.err. caturra keeps no frame
            // line info, so emit the exception's header line (the part
            // students actually read).
            let header = match message {
                Some(message) => format!("{class_name}: {message}\n"),
                None => format!("{class_name}\n"),
            };
            console.stderr(header.as_bytes());
            Ok(None)
        }
        (
            HeapObject::Exception {
                class_name,
                message,
                ..
            },
            "getMessage" | "toString" | "getLocalizedMessage",
        ) => {
            let rendered = if method == "getMessage" || method == "getLocalizedMessage" {
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
        // `getCause()` — the chained cause, or null.
        (HeapObject::Exception { cause, .. }, "getCause") => Ok(Some(JValue::Ref(*cause))),
        // `initCause(Throwable)` sets the cause and returns `this`.
        (HeapObject::Exception { .. }, "initCause") => {
            let cause = args.first().and_then(ref_arg);
            if let Some(HeapObject::Exception { cause: slot, .. }) = heap.get_mut(receiver) {
                *slot = cause;
            }
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        // `getSuppressed()` — the exceptions a try-with-resources attached
        // because `close()` threw while this one was already propagating.
        (HeapObject::Exception { suppressed, .. }, "getSuppressed") => {
            let elements: Vec<JValue> = suppressed
                .iter()
                .map(|reference| JValue::Ref(Some(*reference)))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(
                String::from("[Ljava/lang/Throwable;"),
                elements,
            ));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `addSuppressed(t)` — what the try-with-resources desugaring calls
        // when a resource's `close()` throws and the body's exception wins.
        (HeapObject::Exception { .. }, "addSuppressed") => {
            if let Some(JValue::Ref(Some(extra))) = args.first().copied()
                && let Some(HeapObject::Exception { suppressed, .. }) = heap.get_mut(receiver)
            {
                suppressed.push(extra);
            }
            Ok(None)
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

/// The heap reference an argument holds (`None` for `null` or a non-reference).
fn ref_arg(value: &JValue) -> Option<HeapRef> {
    match value {
        JValue::Ref(r) => *r,
        _ => None,
    }
}

/// A throwable's default `toString`: its binary class name, plus `": message"`
/// when it has one. Used to derive `new X(cause)`'s message, as
/// `Throwable(Throwable)` does. (A user override of `toString` is not consulted
/// — this is only the constructor's message derivation.)
fn throwable_to_string(heap: &Heap, reference: HeapRef) -> String {
    match heap.get(reference) {
        Some(HeapObject::Exception {
            class_name,
            message,
            ..
        }) => match message {
            Some(message) => format!("{class_name}: {message}"),
            None => class_name.clone(),
        },
        Some(HeapObject::Instance { class_name, .. }) => {
            let message = heap
                .get(reference)
                .and_then(|o| o.field("__message"))
                .and_then(|v| match v {
                    JValue::Ref(Some(r)) => heap.string_text(r),
                    _ => None,
                });
            let dotted = class_name.replace('/', ".");
            match message {
                Some(message) => format!("{dotted}: {message}"),
                None => dotted,
            }
        }
        _ => String::from("java.lang.Throwable"),
    }
}

/// Store an exception's message and chained cause: a library `Exception`
/// object sets its own fields; a user exception `Instance` (reached through
/// `super(message, cause)`) stashes them in the `__message`/`__cause` slots its
/// layout reserves.
fn set_exception_cause(
    heap: &mut Heap,
    receiver: HeapRef,
    class: &str,
    message: Option<String>,
    cause: Option<HeapRef>,
) {
    match heap.get(receiver) {
        Some(HeapObject::Exception { .. }) => {
            if let Some(HeapObject::Exception {
                message: slot,
                cause: cause_slot,
                ..
            }) = heap.get_mut(receiver)
            {
                *slot = message;
                *cause_slot = cause;
            }
        }
        Some(HeapObject::Instance { .. })
            if caturra_classfile::exceptions::is_exception_class(class) =>
        {
            if let Some(text) = message {
                let reference = heap.alloc_string(&text);
                if let Some(field) = heap
                    .get_mut(receiver)
                    .and_then(|object| object.field_mut("__message"))
                {
                    *field = JValue::Ref(Some(reference));
                }
            }
            if let Some(field) = heap
                .get_mut(receiver)
                .and_then(|object| object.field_mut("__cause"))
            {
                *field = JValue::Ref(cause);
            }
        }
        _ => {}
    }
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
                // A StringBuilder is a CharSequence too: `str.contentEquals(sb)`.
                Some(HeapObject::JavaString(other) | HeapObject::StringBuilder(other)) => {
                    Ok(other.clone())
                }
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
        ("split", [pattern]) => {
            let pattern = arg_units(pattern)?;
            let parts = split_regex(&units, &pattern, 0)?;
            let refs: Vec<JValue> = parts
                .into_iter()
                .map(|part| JValue::Ref(Some(heap.alloc(HeapObject::JavaString(part)))))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(
                String::from("[Ljava/lang/String;"),
                refs,
            ));
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
        ("matches", [pattern]) => {
            let pattern = arg_units(pattern)?;
            Ok(Some(JValue::Int(i32::from(matches_regex(
                &units, &pattern,
            )?))))
        }
        ("replaceAll", [pattern, replacement]) => {
            let pattern = arg_units(pattern)?;
            let replacement = arg_units(replacement)?;
            let replaced = replace_regex(&units, &pattern, &replacement, false)?;
            let reference = heap.alloc(HeapObject::JavaString(replaced));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("replaceFirst", [pattern, replacement]) => {
            let pattern = arg_units(pattern)?;
            let replacement = arg_units(replacement)?;
            let replaced = replace_regex(&units, &pattern, &replacement, true)?;
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
        ("split", [pattern, JValue::Int(limit)]) => {
            let pattern = arg_units(pattern)?;
            let parts = split_regex(&units, &pattern, *limit)?;
            let refs: Vec<JValue> = parts
                .into_iter()
                .map(|part| JValue::Ref(Some(heap.alloc(HeapObject::JavaString(part)))))
                .collect();
            let reference = heap.alloc(HeapObject::RefArray(
                String::from("[Ljava/lang/String;"),
                refs,
            ));
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
            let reference = heap.alloc(HeapObject::IntArray(IntKind::Char, values));
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
        ) => get_chars(heap, &units, *begin, *end, *target, *at),
        ("toString", []) => Ok(Some(JValue::Ref(Some(receiver)))),
        ("intern", []) => {
            let canonical = heap.find_string(&units).unwrap_or(receiver);
            Ok(Some(JValue::Ref(Some(canonical))))
        }
        ("codePointAt", [JValue::Int(index)]) => {
            code_point_at(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointBefore", [JValue::Int(index)]) => {
            code_point_before(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointCount", [JValue::Int(begin), JValue::Int(end)]) => {
            code_point_count(&units, *begin, *end).map(|n| Some(JValue::Int(n)))
        }
        ("offsetByCodePoints", [JValue::Int(index), JValue::Int(offset)]) => {
            offset_by_code_points(&units, *index, *offset).map(|at| Some(JValue::Int(at)))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("String.{method}"))),
    }
}

/// `getChars(begin, end, dest, at)` — copy UTF-16 units into a `char[]`.
/// Shared by `String` and `StringBuilder`.
fn get_chars(
    heap: &mut Heap,
    units: &[u16],
    begin: i32,
    end: i32,
    target: HeapRef,
    at: i32,
) -> Result<Option<JValue>, VmError> {
    let source = usize::try_from(begin)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(b, e)| b <= e && *e <= units.len())
        .map(|(b, e)| units[b..e].to_vec())
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: begin {begin}, end {end}, \
                 length {}",
                units.len()
            ))
        })?;
    let at = usize::try_from(at).map_err(|_| {
        throw(format!(
            "java.lang.ArrayIndexOutOfBoundsException: Index {at} out of bounds"
        ))
    })?;
    let Some(HeapObject::IntArray(_, values)) = heap.get_mut(target) else {
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

/// `codePointBefore(index)`: the code point ending just before `index`.
fn code_point_before(units: &[u16], index: i32) -> Result<i32, VmError> {
    let before = index - 1;
    if before < 0 {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: index {index}"
        )));
    }
    // A low surrogate preceded by a high one forms a pair.
    let at = usize::try_from(before).unwrap_or(usize::MAX);
    if at > 0
        && units.get(at).is_some_and(|u| is_low_surrogate(*u))
        && units.get(at - 1).is_some_and(|u| is_high_surrogate(*u))
    {
        return code_point_at(units, before - 1);
    }
    code_point_at(units, before)
}

/// `codePointCount(begin, end)`: code points in the half-open range.
fn code_point_count(units: &[u16], begin: i32, end: i32) -> Result<i32, VmError> {
    let (begin, end) = usize::try_from(begin)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(b, e)| b <= e && *e <= units.len())
        .ok_or_else(|| {
            throw(format!(
                "java.lang.IndexOutOfBoundsException: begin {begin}, end {end}, length {}",
                units.len()
            ))
        })?;
    let count = char::decode_utf16(units[begin..end].iter().copied()).count();
    Ok(i32::try_from(count).unwrap_or(i32::MAX))
}

/// `offsetByCodePoints(index, offset)`: the unit index `offset` code
/// points away from `index`.
fn offset_by_code_points(units: &[u16], index: i32, offset: i32) -> Result<i32, VmError> {
    let mut at = usize::try_from(index)
        .ok()
        .filter(|at| *at <= units.len())
        .ok_or_else(|| throw(format!("java.lang.IndexOutOfBoundsException: {index}")))?;
    let out_of_range = || throw(format!("java.lang.IndexOutOfBoundsException: {offset}"));
    let mut remaining = offset;
    while remaining > 0 {
        if at >= units.len() {
            return Err(out_of_range());
        }
        at += if units.get(at).is_some_and(|u| is_high_surrogate(*u))
            && units.get(at + 1).is_some_and(|u| is_low_surrogate(*u))
        {
            2
        } else {
            1
        };
        remaining -= 1;
    }
    while remaining < 0 {
        if at == 0 {
            return Err(out_of_range());
        }
        at -= if at >= 2
            && units.get(at - 1).is_some_and(|u| is_low_surrogate(*u))
            && units.get(at - 2).is_some_and(|u| is_high_surrogate(*u))
        {
            2
        } else {
            1
        };
        remaining += 1;
    }
    Ok(i32::try_from(at).unwrap_or(i32::MAX))
}

fn is_high_surrogate(unit: u16) -> bool {
    (0xD800..0xDC00).contains(&unit)
}

fn is_low_surrogate(unit: u16) -> bool {
    (0xDC00..0xE000).contains(&unit)
}

/// `java.lang.StringBuilder` instance methods. The builder stores UTF-16
/// code units, exactly as `String` does, so indices mean what Java says
/// they mean even across supplementary characters.
///
/// `capacity` is deliberately absent — caturra models a builder's contents
/// but not its backing array, so there is no honest number to return.
/// `ensureCapacity`/`trimToSize` are therefore the no-ops they observably
/// are without it.
#[allow(clippy::too_many_lines)] // one method table
fn builder_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(HeapObject::StringBuilder(units)) = heap.get(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    let units = units.clone();
    let count = units.len();
    let len = i32::try_from(count).unwrap_or(i32::MAX);
    let params = descriptor_params(descriptor);

    // Resolve a String argument's code units (null → NPE like Java).
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
        ("charAt", [JValue::Int(index)]) => {
            let at = check_index(*index, count)?;
            Ok(Some(JValue::Int(i32::from(units[at]))))
        }
        ("toString", []) => {
            let text = heap.alloc(HeapObject::JavaString(units));
            Ok(Some(JValue::Ref(Some(text))))
        }
        // `equals` is Object identity: StringBuilder does not override it, so
        // two builders are equal only when they are the SAME object.
        ("equals", [other]) => {
            let same = matches!(other, JValue::Ref(Some(r)) if *r == receiver);
            Ok(Some(JValue::Int(i32::from(same))))
        }
        // `append(char[], offset, len)` — a sub-range of the array.
        ("append", [array, JValue::Int(offset), JValue::Int(length)]) => {
            let chars = char_array_units(heap, array)?;
            let offset = usize::try_from(*offset).unwrap_or(usize::MAX);
            let length = usize::try_from(*length).unwrap_or(usize::MAX);
            let end = offset.checked_add(length);
            let slice = end
                .filter(|e| *e <= chars.len())
                .map(|e| chars[offset..e].to_vec())
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.ArrayIndexOutOfBoundsException: {offset}"
                    ))
                })?;
            let mut appended = units;
            appended.extend(slice);
            builder_store(heap, receiver, appended);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("append", [value]) => {
            let mut appended = units;
            appended.extend(builder_value_text(heap, params, value)?);
            builder_store(heap, receiver, appended);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("appendCodePoint", [JValue::Int(code_point)]) => {
            let mut appended = units;
            appended.extend(code_point_units(*code_point)?);
            builder_store(heap, receiver, appended);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("insert", [JValue::Int(offset), value]) => {
            // The offset is the first parameter; the value's own descriptor
            // is whatever follows it.
            let value_units = builder_value_text(heap, &params["I".len()..], value)?;
            let at = check_offset(*offset, count)?;
            let mut inserted = units;
            inserted.splice(at..at, value_units);
            builder_store(heap, receiver, inserted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("delete", [JValue::Int(start), JValue::Int(end)]) => {
            let (start, end) = check_range(*start, *end, count)?;
            let mut deleted = units;
            deleted.drain(start..end);
            builder_store(heap, receiver, deleted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("deleteCharAt", [JValue::Int(index)]) => {
            let at = check_index(*index, count)?;
            let mut deleted = units;
            deleted.remove(at);
            builder_store(heap, receiver, deleted);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("replace", [JValue::Int(start), JValue::Int(end), value]) => {
            let value_units = arg_units(value)?;
            let (start, end) = check_range(*start, *end, count)?;
            let mut replaced = units;
            replaced.splice(start..end, value_units);
            builder_store(heap, receiver, replaced);
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("reverse", []) => {
            builder_store(heap, receiver, reverse_units(&units));
            Ok(Some(JValue::Ref(Some(receiver))))
        }
        ("setCharAt", [JValue::Int(index), JValue::Int(ch)]) => {
            let at = check_index(*index, count)?;
            let mut updated = units;
            updated[at] = u16::try_from(*ch).unwrap_or(u16::MAX);
            builder_store(heap, receiver, updated);
            Ok(None)
        }
        ("setLength", [JValue::Int(new_length)]) => {
            let new_length = usize::try_from(*new_length).map_err(|_| {
                throw(format!(
                    "java.lang.StringIndexOutOfBoundsException: String index out of range: {new_length}"
                ))
            })?;
            let mut resized = units;
            // Growing pads with the null character, as Java does.
            resized.resize(new_length, 0);
            builder_store(heap, receiver, resized);
            Ok(None)
        }
        // caturra does not model capacity, so there is nothing to reserve
        // or release. See the doc comment above.
        ("ensureCapacity", [JValue::Int(_)]) | ("trimToSize", []) => Ok(None),
        ("indexOf", [needle]) => Ok(Some(JValue::Int(index_of(&units, &arg_units(needle)?)))),
        ("indexOf", [needle, JValue::Int(from)]) => Ok(Some(JValue::Int(index_of_from(
            &units,
            &arg_units(needle)?,
            *from,
        )))),
        ("lastIndexOf", [needle]) => Ok(Some(JValue::Int(last_index_of_from(
            &units,
            &arg_units(needle)?,
            i32::MAX,
        )))),
        ("lastIndexOf", [needle, JValue::Int(from)]) => Ok(Some(JValue::Int(last_index_of_from(
            &units,
            &arg_units(needle)?,
            *from,
        )))),
        ("substring", [JValue::Int(begin)]) => substring(heap, &units, *begin, len),
        // subSequence is substring by another name (CharSequence view). Unlike
        // String's substring, StringBuilder's out-of-range message says
        // "start"/"end" (not "begin"), and does NOT clamp end.
        ("substring" | "subSequence", [JValue::Int(begin), JValue::Int(end)]) => {
            builder_substring(heap, &units, *begin, *end)
        }
        ("compareTo", [other]) => {
            let other = match other {
                JValue::Ref(Some(reference)) => match heap.get(*reference) {
                    Some(HeapObject::StringBuilder(other)) => other.clone(),
                    _ => return Err(throw("java.lang.ClassCastException: not a StringBuilder")),
                },
                _ => return Err(throw("java.lang.NullPointerException")),
            };
            Ok(Some(JValue::Int(compare_utf16(&units, &other))))
        }
        (
            "getChars",
            [
                JValue::Int(begin),
                JValue::Int(end),
                JValue::Ref(Some(target)),
                JValue::Int(at),
            ],
        ) => get_chars(heap, &units, *begin, *end, *target, *at),
        ("codePointAt", [JValue::Int(index)]) => {
            code_point_at(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointBefore", [JValue::Int(index)]) => {
            code_point_before(&units, *index).map(|cp| Some(JValue::Int(cp)))
        }
        ("codePointCount", [JValue::Int(begin), JValue::Int(end)]) => {
            code_point_count(&units, *begin, *end).map(|n| Some(JValue::Int(n)))
        }
        ("offsetByCodePoints", [JValue::Int(index), JValue::Int(offset)]) => {
            offset_by_code_points(&units, *index, *offset).map(|at| Some(JValue::Int(at)))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("StringBuilder.{method}"))),
    }
}

/// Overwrite a builder's contents.
fn builder_store(heap: &mut Heap, receiver: HeapRef, units: Vec<u16>) {
    let Some(HeapObject::StringBuilder(slot)) = heap.get_mut(receiver) else {
        unreachable!("receiver kind checked by caller");
    };
    *slot = units;
}

/// An index into existing contents (`0 <= index < count`).
fn check_index(index: i32, count: usize) -> Result<usize, VmError> {
    usize::try_from(index)
        .ok()
        .filter(|at| *at < count)
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: index {index},length {count}"
            ))
        })
}

/// A position between characters (`0 <= offset <= count`), as `insert` takes.
fn check_offset(offset: i32, count: usize) -> Result<usize, VmError> {
    usize::try_from(offset)
        .ok()
        .filter(|at| *at <= count)
        .ok_or_else(|| {
            throw(format!(
                "java.lang.StringIndexOutOfBoundsException: offset {offset},length {count}"
            ))
        })
}

/// A `delete`/`replace` range: `start` must be a valid offset no greater
/// than `end`, and `end` is clamped to the length (Java tolerates a large
/// `end` here, unlike `substring`).
fn check_range(start: i32, end: i32, count: usize) -> Result<(usize, usize), VmError> {
    // JDK 11 clamps `end` to the length FIRST, so the message reports the
    // clamped value: `replace(9, 10, ...)` on a length-3 builder is
    // "start 9, end 3, length 3", not "end 10".
    let clamped_end = end.min(i32::try_from(count).unwrap_or(i32::MAX));
    let bad = || {
        throw(format!(
            "java.lang.StringIndexOutOfBoundsException: start {start}, \
             end {clamped_end}, length {count}"
        ))
    };
    if start > clamped_end {
        return Err(bad());
    }
    let start = usize::try_from(start).ok().filter(|s| *s <= count);
    let start = start.ok_or_else(bad)?;
    let end = usize::try_from(clamped_end).unwrap_or(0);
    Ok((start, end))
}

/// The UTF-16 units of a code point, for `appendCodePoint`.
fn code_point_units(code_point: i32) -> Result<Vec<u16>, VmError> {
    // The message shows the value as UPPERCASE hex of its unsigned bits —
    // `-1` is `0xFFFFFFFF`, `0x110000` is itself (JDK 11's
    // `Character.toString(int)` / `appendCodePoint`).
    let invalid = || {
        throw(format!(
            "java.lang.IllegalArgumentException: Not a valid Unicode code point: 0x{:X}",
            code_point.cast_unsigned()
        ))
    };
    let valid = u32::try_from(code_point)
        .ok()
        .filter(|cp| *cp <= 0x0010_FFFF)
        .ok_or_else(invalid)?;
    // Surrogate code points have no scalar value but are still one `char`.
    if (0xD800..0xE000).contains(&valid) {
        return Ok(vec![u16::try_from(valid).unwrap_or(u16::MAX)]);
    }
    let Some(ch) = char::from_u32(valid) else {
        return Err(invalid());
    };
    let mut buffer = [0u16; 2];
    Ok(ch.encode_utf16(&mut buffer).to_vec())
}

/// `StringBuilder.reverse()`: reverse the code units, but keep the two
/// halves of a valid surrogate pair in order — Java reverses by code point.
fn reverse_units(units: &[u16]) -> Vec<u16> {
    let mut reversed = Vec::with_capacity(units.len());
    let mut at = units.len();
    while at > 0 {
        if at >= 2 && is_low_surrogate(units[at - 1]) && is_high_surrogate(units[at - 2]) {
            reversed.extend_from_slice(&units[at - 2..at]);
            at -= 2;
        } else {
            reversed.push(units[at - 1]);
            at -= 1;
        }
    }
    reversed
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

fn substring(
    heap: &mut Heap,
    units: &[u16],
    begin: i32,
    len: i32,
) -> Result<Option<JValue>, VmError> {
    // JDK 11's single-arg `substring(beginIndex)` reports "String index out of
    // range: N": a negative beginIndex reports beginIndex itself, and an
    // over-long one reports the negative subLen (`length - beginIndex`). This
    // differs from the two-arg form's "begin/end/length" wording.
    if begin < 0 {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: String index out of range: {begin}"
        )));
    }
    let sub_len = len - begin;
    if sub_len < 0 {
        return Err(throw(format!(
            "java.lang.StringIndexOutOfBoundsException: String index out of range: {sub_len}"
        )));
    }
    let begin_usize = usize::try_from(begin).unwrap_or(0).min(units.len());
    let reference = heap.alloc(HeapObject::JavaString(units[begin_usize..].to_vec()));
    Ok(Some(JValue::Ref(Some(reference))))
}

/// `StringBuilder.substring(start, end)` — like `String.substring` but with
/// the builder's `start`/`end` message wording (String says `begin`), and it
/// does NOT clamp `end` (an over-long `end` is an error, unlike `delete`).
fn builder_substring(
    heap: &mut Heap,
    units: &[u16],
    start: i32,
    end: i32,
) -> Result<Option<JValue>, VmError> {
    let length = units.len();
    let bad = || {
        throw(format!(
            "java.lang.StringIndexOutOfBoundsException: start {start}, end {end}, length {length}"
        ))
    };
    let valid = usize::try_from(start)
        .ok()
        .zip(usize::try_from(end).ok())
        .filter(|(s, e)| s <= e && *e <= length);
    let Some((start, end)) = valid else {
        return Err(bad());
    };
    let reference = heap.alloc(HeapObject::JavaString(units[start..end].to_vec()));
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

/// Compile a pattern, reporting a bad one as Java's `PatternSyntaxException`.
fn compile_regex(pattern: &[u16]) -> Result<crate::regex::Regex, VmError> {
    crate::regex::Regex::new(pattern).map_err(|error| {
        throw(format!(
            "java.util.regex.PatternSyntaxException: {}",
            error.message()
        ))
    })
}

/// Successive matches of `regex` in `input`, as Java's `Matcher.find` walks
/// them: each search resumes at the previous match's end, and a zero-width
/// match advances one unit so the loop cannot stall.
fn find_matches(regex: &crate::regex::Regex, input: &[u16]) -> Vec<crate::regex::Match> {
    let mut found = Vec::new();
    let mut at = 0;
    while at <= input.len() {
        let Some(one) = regex.find_at(input, at) else {
            break;
        };
        at = if one.end == one.start {
            one.end + 1
        } else {
            one.end
        };
        found.push(one);
    }
    found
}

/// `String.split(regex, limit)` — JDK 11's algorithm, step for step.
///
/// The delimiter is a REGULAR EXPRESSION, not a literal. Several rules here
/// look arbitrary but are load-bearing, and each one is a case caturra used to
/// get wrong:
///
/// * a zero-width match at position 0 contributes no leading `""`;
/// * if nothing matched at all, the result is the whole input as ONE element,
///   which is why `"".split(",")` is `[""]` and not `[]`;
/// * `limit > 0` caps the parts, with the last one holding the entire rest;
/// * `limit == 0` drops trailing empty strings, `limit < 0` keeps them.
fn split_regex(input: &[u16], pattern: &[u16], limit: i32) -> Result<Vec<Vec<u16>>, VmError> {
    let regex = compile_regex(pattern)?;
    let match_limited = limit > 0;
    let cap = usize::try_from(limit).unwrap_or(0);
    let mut parts: Vec<Vec<u16>> = Vec::new();
    let mut index = 0;

    for one in find_matches(&regex, input) {
        if !match_limited || parts.len() < cap - 1 {
            // No empty leading substring for a zero-width match at the very
            // beginning of the input.
            if index == 0 && one.start == 0 && one.end == 0 {
                continue;
            }
            parts.push(input[index..one.start].to_vec());
            index = one.end;
        } else if parts.len() == cap - 1 {
            // The last permitted part takes everything that is left.
            parts.push(input[index..].to_vec());
            index = one.end;
        }
    }

    // No match anywhere: the input comes back whole and untouched.
    if index == 0 {
        return Ok(vec![input.to_vec()]);
    }
    if !match_limited || parts.len() < cap {
        parts.push(input[index..].to_vec());
    }
    if limit == 0 {
        while parts.last().is_some_and(Vec::is_empty) {
            parts.pop();
        }
    }
    Ok(parts)
}

/// `String.matches` — the pattern must match the entire string.
fn matches_regex(input: &[u16], pattern: &[u16]) -> Result<bool, VmError> {
    Ok(compile_regex(pattern)?.matches_whole(input))
}

/// `String.replaceAll` / `replaceFirst`.
///
/// The replacement is not literal: `$1` inserts a group and `\` escapes the
/// next character, matching `Matcher.appendReplacement`. A `$` naming a group
/// that does not exist is an error in Java, not a literal dollar.
fn replace_regex(
    input: &[u16],
    pattern: &[u16],
    replacement: &[u16],
    first_only: bool,
) -> Result<Vec<u16>, VmError> {
    let regex = compile_regex(pattern)?;
    let mut out: Vec<u16> = Vec::new();
    let mut index = 0;
    for one in find_matches(&regex, input) {
        if one.start < index {
            // A zero-width match immediately after a consumed one.
            continue;
        }
        out.extend_from_slice(&input[index..one.start]);
        out.extend(expand_replacement(input, &one, replacement, &regex)?);
        index = one.end;
        if first_only {
            break;
        }
    }
    out.extend_from_slice(&input[index..]);
    Ok(out)
}

/// Expand `$n` group references and `\` escapes in a replacement string.
fn expand_replacement(
    input: &[u16],
    one: &crate::regex::Match,
    replacement: &[u16],
    regex: &crate::regex::Regex,
) -> Result<Vec<u16>, VmError> {
    let mut out: Vec<u16> = Vec::new();
    let mut at = 0;
    while at < replacement.len() {
        let unit = replacement[at];
        if unit == u16::from(b'\\') {
            at += 1;
            if at < replacement.len() {
                out.push(replacement[at]);
                at += 1;
            }
            continue;
        }
        if unit != u16::from(b'$') {
            out.push(unit);
            at += 1;
            continue;
        }
        at += 1;
        // Java takes the longest run of digits that still names a group.
        let mut group = None;
        while at < replacement.len() {
            let digit = replacement[at];
            if !(0x30..=0x39).contains(&digit) {
                break;
            }
            let extended = group.unwrap_or(0) * 10 + usize::from(digit - 0x30);
            if extended > regex.group_count() {
                break;
            }
            group = Some(extended);
            at += 1;
        }
        let Some(group) = group else {
            return Err(throw(String::from(
                "java.lang.IllegalArgumentException: Illegal group reference",
            )));
        };
        if let Some(Some((from, to))) = one.groups.get(group).copied() {
            out.extend_from_slice(&input[from..to]);
        }
    }
    Ok(out)
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
#[allow(clippy::too_many_lines)] // one arm per Scanner method
fn scanner_method(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    method: &str,
) -> Result<Option<JValue>, VmError> {
    // Every method but `close` refuses a closed Scanner, as the JDK's does.
    // Closing twice is a no-op there, so `close` is exempt.
    let (stdin, closed) = match heap.get(receiver) {
        Some(HeapObject::Scanner { stdin, closed, .. }) => (*stdin, *closed),
        _ => unreachable!("receiver kind checked by caller"),
    };
    if closed && method != "close" {
        return Err(throw("java.lang.IllegalStateException: Scanner closed"));
    }
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
        "nextLong" => {
            let value = scanner_take(heap, console, receiver, |t| t.parse::<i64>().ok())?;
            Ok(Some(JValue::Long(value)))
        }
        "hasNextLong" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| t.parse::<i64>().is_ok());
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextFloat" => {
            let value = scanner_take(heap, console, receiver, |t| {
                is_java_float_token(t)
                    .then(|| t.parse::<f32>().ok())
                    .flatten()
            })?;
            Ok(Some(JValue::Float(value)))
        }
        "hasNextFloat" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| is_java_float_token(&t));
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextShort" | "nextByte" => {
            let (lo, hi) = if method == "nextShort" {
                (i32::from(i16::MIN), i32::from(i16::MAX))
            } else {
                (i32::from(i8::MIN), i32::from(i8::MAX))
            };
            let value = scanner_take(heap, console, receiver, |t| {
                t.parse::<i32>().ok().filter(|v| *v >= lo && *v <= hi)
            })?;
            Ok(Some(JValue::Int(value)))
        }
        "hasNextShort" | "hasNextByte" => {
            let (lo, hi) = if method == "hasNextShort" {
                (i32::from(i16::MIN), i32::from(i16::MAX))
            } else {
                (i32::from(i8::MIN), i32::from(i8::MAX))
            };
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token
                .and_then(|t| t.parse::<i32>().ok())
                .is_some_and(|v| v >= lo && v <= hi);
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextBoolean" => {
            let value = scanner_take(heap, console, receiver, |t| {
                if t.eq_ignore_ascii_case("true") {
                    Some(1)
                } else if t.eq_ignore_ascii_case("false") {
                    Some(0)
                } else {
                    None
                }
            })?;
            Ok(Some(JValue::Int(value)))
        }
        "hasNextBoolean" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token
                .is_some_and(|t| t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("false"));
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        // `close()` on a `System.in` scanner closes the stream itself, so a
        // later `new Scanner(System.in)` reads nothing — the JDK does this,
        // and a program that closes stdin and reads again dies on a real JVM.
        // A file scanner closes only itself.
        "close" => {
            if let Some(HeapObject::Scanner { closed, .. }) = heap.get_mut(receiver) {
                *closed = true;
            }
            if stdin {
                console.close_stdin();
            }
            Ok(None)
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
            let value = scanner_take(heap, console, receiver, |t| t.parse::<i32>().ok())?;
            Ok(Some(JValue::Int(value)))
        }
        "hasNextInt" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| t.parse::<i32>().is_ok());
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        "nextDouble" => {
            let value = scanner_take(heap, console, receiver, |t| {
                is_java_float_token(t)
                    .then(|| t.parse::<f64>().ok())
                    .flatten()
            })?;
            Ok(Some(JValue::Double(value)))
        }
        "hasNextDouble" => {
            let token = scanner_peek_token(heap, console, receiver)?;
            let ok = token.is_some_and(|t| is_java_float_token(&t));
            Ok(Some(JValue::Int(i32::from(ok))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Scanner.{method}"))),
    }
}

/// `java.io.BufferedReader`/`FileReader` methods. A file reader hands out lines
/// from its slurped buffer; a `System.in` reader pulls each line from the
/// console. `readLine` returns null at end of stream (not an exception).
fn reader_method(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    method: &str,
) -> Result<Option<JValue>, VmError> {
    let (stdin, closed) = match heap.get(receiver) {
        Some(HeapObject::Reader { stdin, closed, .. }) => (*stdin, *closed),
        _ => unreachable!("receiver kind checked by caller"),
    };
    match method {
        "readLine" => {
            let line = if closed {
                None
            } else if stdin {
                console.read_line()
            } else {
                reader_next_line(heap, receiver)
            };
            Ok(Some(match line {
                Some(text) => JValue::Ref(Some(heap.alloc_string(&text))),
                None => JValue::Ref(None),
            }))
        }
        "read" => {
            let ch = if closed || stdin {
                -1
            } else {
                reader_next_char(heap, receiver)
            };
            Ok(Some(JValue::Int(ch)))
        }
        "ready" => {
            let ready = !closed
                && !stdin
                && matches!(heap.get(receiver), Some(HeapObject::Reader { buffer, pos, .. }) if *pos < buffer.len());
            Ok(Some(JValue::Int(i32::from(ready))))
        }
        "close" => {
            if let Some(HeapObject::Reader { closed, .. }) = heap.get_mut(receiver) {
                *closed = true;
            }
            Ok(None)
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "BufferedReader.{method}"
        ))),
    }
}

/// The next line from a file reader's buffer, without its terminator, or `None`
/// at end. A line ends at `\n`, `\r`, or `\r\n` (both a real `BufferedReader`
/// and this treat them alike). `\n`/`\r` are ASCII, so scanning bytes never splits a
/// multi-byte UTF-8 character.
fn reader_next_line(heap: &mut Heap, receiver: HeapRef) -> Option<String> {
    let Some(HeapObject::Reader { buffer, pos, .. }) = heap.get_mut(receiver) else {
        return None;
    };
    if *pos >= buffer.len() {
        return None;
    }
    let bytes = buffer.as_bytes();
    let start = *pos;
    let mut end = start;
    while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
        end += 1;
    }
    let line = buffer[start..end].to_string();
    let mut next = end;
    if next < bytes.len() {
        if bytes[next] == b'\r' {
            next += 1;
            if next < bytes.len() && bytes[next] == b'\n' {
                next += 1;
            }
        } else {
            next += 1; // '\n'
        }
    }
    *pos = next;
    Some(line)
}

/// The next character from a file reader's buffer as its code point, or -1 at
/// end. (A supplementary character would arrive as two UTF-16 units on a JVM;
/// this hands back the code point, which agrees for the Basic Multilingual
/// Plane a `read()` almost always sees.)
fn reader_next_char(heap: &mut Heap, receiver: HeapRef) -> i32 {
    let Some(HeapObject::Reader { buffer, pos, .. }) = heap.get_mut(receiver) else {
        return -1;
    };
    let Some(ch) = buffer[*pos..].chars().next() else {
        return -1;
    };
    *pos += ch.len_utf8();
    i32::try_from(u32::from(ch)).unwrap_or(-1)
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
        Some(HeapObject::Scanner {
            buffer, pos, eof, ..
        }) => (buffer.clone(), *pos, *eof),
        _ => unreachable!("receiver kind checked by caller"),
    }
}

fn scanner_set_pos(heap: &mut Heap, receiver: HeapRef, new_pos: usize) {
    if let Some(HeapObject::Scanner { pos, .. }) = heap.get_mut(receiver) {
        *pos = new_pos;
    }
}

/// Read up to the next newline (consuming it); `None` at EOF.
/// Drop a single trailing carriage return, so a `\r\n`-terminated line reads
/// the way the JDK's `nextLine` returns it (terminator excluded).
fn strip_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

fn scanner_next_line(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
) -> Result<Option<String>, VmError> {
    loop {
        let (buffer, pos, eof) = scanner_state(heap, receiver);
        if let Some(offset) = buffer[pos..].find('\n') {
            // The JDK's line separator matches `\r\n` as one terminator, so a
            // CRLF source yields `a`, not `a\r` — strip the carriage return.
            let line = strip_cr(&buffer[pos..pos + offset]).to_owned();
            scanner_set_pos(heap, receiver, pos + offset + 1);
            return Ok(Some(line));
        }
        if eof {
            // Trailing text without a newline still counts as a line.
            if pos < buffer.len() {
                let line = strip_cr(&buffer[pos..]).to_owned();
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

/// Read one token and translate it, consuming it only if the translation works.
///
/// `java.util.Scanner` "will not pass the token that caused the exception", so
/// after `nextInt()` throws `InputMismatchException` the offending token is
/// still there — which is what makes the usual recovery loop
/// (`catch (InputMismatchException e) { in.next(); }`) skip the bad word rather
/// than the one after it. `None` from `parse` means a mismatch.
fn scanner_take<T>(
    heap: &mut Heap,
    console: &mut dyn ConsoleIo,
    receiver: HeapRef,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Result<T, VmError> {
    let token = scanner_peek_token(heap, console, receiver)?
        .ok_or_else(|| throw("java.util.NoSuchElementException"))?;
    let value = parse(&token).ok_or_else(|| throw("java.util.InputMismatchException"))?;
    scanner_next_token(heap, console, receiver)?;
    Ok(value)
}

/// Whether a token is a float/double literal *as `java.util.Scanner` reads it*:
/// an optional sign, then either exactly `NaN` / `Infinity` or a decimal number.
///
/// Rust's `from_str` is more generous — it also takes `nan`, `inf`, `infinity`
/// in any case — so parsing directly would make `hasNextDouble("inf")` true
/// where Java says false. Anything with a letter other than an exponent `e`/`E`
/// is rejected, which also covers `1.5f`, `1.5d` and `0x10` exactly as Java does.
fn is_java_float_token(token: &str) -> bool {
    let digits = token.strip_prefix(['-', '+']).unwrap_or(token);
    if digits == "NaN" || digits == "Infinity" {
        return true;
    }
    if digits.is_empty()
        || digits
            .bytes()
            .any(|b| b.is_ascii_alphabetic() && b != b'e' && b != b'E')
    {
        return false;
    }
    token.parse::<f64>().is_ok()
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

/// Element equality for list membership: value equality for numbers
/// and strings (Java's `equals`), reference equality for user objects
/// (correct here, since `equals` overriding is not supported).
/// The elements `System.arraycopy` lifts out of the source before writing
/// them, so that a copy within one array behaves as Java's does: as if it
/// went through a temporary.
enum ArrayChunk {
    Int(Vec<i32>),
    Double(Vec<f64>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Short(Vec<i16>),
    Byte(Vec<i8>),
    Ref(Vec<JValue>),
}

/// `System.arraycopy(src, srcPos, dest, destPos, length)`.
/// The `count` elements of `source` starting at `from`, as a chunk that
/// borrows nothing — `arraycopy` writes back into the same heap, and a
/// self-copy must read the old values.
fn take_chunk(heap: &Heap, source: HeapRef, from: usize, count: usize) -> Option<ArrayChunk> {
    Some(match heap.get(source) {
        Some(HeapObject::IntArray(_, values)) => {
            ArrayChunk::Int(values[from..from + count].to_vec())
        }
        Some(HeapObject::DoubleArray(values)) => {
            ArrayChunk::Double(values[from..from + count].to_vec())
        }
        Some(HeapObject::LongArray(values)) => {
            ArrayChunk::Long(values[from..from + count].to_vec())
        }
        Some(HeapObject::FloatArray(values)) => {
            ArrayChunk::Float(values[from..from + count].to_vec())
        }
        Some(HeapObject::ShortArray(values)) => {
            ArrayChunk::Short(values[from..from + count].to_vec())
        }
        Some(HeapObject::ByteArray(values)) => {
            ArrayChunk::Byte(values[from..from + count].to_vec())
        }
        Some(HeapObject::RefArray(_, values)) => {
            ArrayChunk::Ref(values[from..from + count].to_vec())
        }
        _ => return None,
    })
}

fn system_arraycopy(heap: &mut Heap, args: &[JValue]) -> Result<Option<JValue>, VmError> {
    let [
        JValue::Ref(source),
        JValue::Int(source_pos),
        JValue::Ref(destination),
        JValue::Int(destination_pos),
        JValue::Int(length),
    ] = args
    else {
        return Err(VmError::UnknownIntrinsic(String::from("System.arraycopy")));
    };
    let (Some(source), Some(destination)) = (*source, *destination) else {
        return Err(throw("java.lang.NullPointerException"));
    };

    // The component types must match exactly: a `boolean[]` never copies into
    // an `int[]`, even though both hold their elements as 32-bit words.
    let store_error = || throw("java.lang.ArrayStoreException: incompatible array types");
    let (Some(source_object), Some(destination_object)) = (heap.get(source), heap.get(destination))
    else {
        return Err(store_error());
    };
    let (source_len, destination_len) = match (source_object, destination_object) {
        (HeapObject::IntArray(from, a), HeapObject::IntArray(to, b)) if from == to => {
            (a.len(), b.len())
        }
        (HeapObject::DoubleArray(a), HeapObject::DoubleArray(b)) => (a.len(), b.len()),
        (HeapObject::LongArray(a), HeapObject::LongArray(b)) => (a.len(), b.len()),
        (HeapObject::FloatArray(a), HeapObject::FloatArray(b)) => (a.len(), b.len()),
        (HeapObject::ShortArray(a), HeapObject::ShortArray(b)) => (a.len(), b.len()),
        (HeapObject::ByteArray(a), HeapObject::ByteArray(b)) => (a.len(), b.len()),
        (HeapObject::RefArray(_, a), HeapObject::RefArray(_, b)) => (a.len(), b.len()),
        _ => return Err(store_error()),
    };

    let out_of_range = |index: i32| {
        throw(format!(
            "java.lang.ArrayIndexOutOfBoundsException: arraycopy: {index}"
        ))
    };
    // A negative position or length is out of range before any bound is even
    // consulted.
    for index in [*source_pos, *destination_pos, *length] {
        if index < 0 {
            return Err(out_of_range(index));
        }
    }
    let (Ok(from), Ok(to), Ok(count)) = (
        usize::try_from(*source_pos),
        usize::try_from(*destination_pos),
        usize::try_from(*length),
    ) else {
        return Err(out_of_range(*length));
    };
    if from + count > source_len {
        return Err(out_of_range(*source_pos));
    }
    if to + count > destination_len {
        return Err(out_of_range(*destination_pos));
    }

    let taken = take_chunk(heap, source, from, count).ok_or_else(store_error)?;
    match (heap.get_mut(destination), taken) {
        (Some(HeapObject::IntArray(_, values)), ArrayChunk::Int(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::DoubleArray(values)), ArrayChunk::Double(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::LongArray(values)), ArrayChunk::Long(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::FloatArray(values)), ArrayChunk::Float(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::ShortArray(values)), ArrayChunk::Short(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::ByteArray(values)), ArrayChunk::Byte(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        (Some(HeapObject::RefArray(_, values)), ArrayChunk::Ref(taken)) => {
            values[to..to + count].copy_from_slice(&taken);
        }
        _ => return Err(store_error()),
    }
    Ok(None)
}

/// `2^n`, exactly, for `-1022 <= n <= 1023` — every such power is a normal
/// `double`, so it is just an exponent field.
fn power_of_two(exponent: i32) -> f64 {
    f64::from_bits(u64::try_from(exponent + 1023).unwrap_or(0) << 52)
}

/// `Math.scalb(d, n)`: `d * 2^n`, rounded once. The JDK gets the single
/// rounding by scaling in one small step and then in 512-sized ones, rather
/// than by multiplying by `2^n` directly — which would round twice whenever
/// the result underflows into the subnormals.
fn java_scalb_double(value: f64, scale: i32) -> f64 {
    // `Double.MAX_EXPONENT + -Double.MIN_EXPONENT + SIGNIFICAND_WIDTH + 1`:
    // past this, the result is certainly zero or infinity.
    const MAX_SCALE: i32 = 1023 + 1022 + 53 + 1;
    let (mut scale, increment, delta) = if scale < 0 {
        (scale.max(-MAX_SCALE), -512, power_of_two(-512))
    } else {
        (scale.min(MAX_SCALE), 512, power_of_two(512))
    };
    // The remainder of `scale` toward zero, in `[-511, 511]`.
    let toward_zero = ((scale >> 8).cast_unsigned() >> 23).cast_signed();
    let adjust = ((scale + toward_zero) & 511) - toward_zero;

    let mut value = value * power_of_two(adjust);
    scale -= adjust;
    while scale != 0 {
        value *= delta;
        scale -= increment;
    }
    value
}

/// `Math.scalb(f, n)`. A `float` needs no staging: every scale that matters
/// fits in one exact `double` multiply.
fn java_scalb_float(value: f32, scale: i32) -> f32 {
    // `Float.MAX_EXPONENT + -Float.MIN_EXPONENT + SIGNIFICAND_WIDTH + 1`.
    const MAX_SCALE: i32 = 127 + 126 + 24 + 1;
    let scale = scale.clamp(-MAX_SCALE, MAX_SCALE);
    #[allow(clippy::cast_possible_truncation)]
    let scaled = (f64::from(value) * power_of_two(scale)) as f32;
    scaled
}

/// The wrapper class and the primitive behind a value, when it is boxed.
fn unboxed(heap: &Heap, value: JValue) -> (Option<&str>, JValue) {
    if let JValue::Ref(Some(reference)) = value
        && let Some(HeapObject::Boxed { class_name, value }) = heap.get(reference)
    {
        return (Some(class_name), *value);
    }
    (None, value)
}

/// `a.equals(b)` for every value the VM can compare without running Java:
/// primitives, wrappers, strings, and `null`. A user object compares by
/// identity — [`Interpreter::java_equals`] overrides that with its `equals`.
///
/// `Double.equals` and `Float.equals` compare raw bits, not numbers, so
/// `-0.0` differs from `0.0` and `NaN` equals itself. `==` says the opposite
/// of both.
pub(crate) fn native_equals(heap: &Heap, a: JValue, b: JValue) -> bool {
    let (class_a, a) = unboxed(heap, a);
    let (class_b, b) = unboxed(heap, b);
    // `Integer.valueOf(1).equals(Long.valueOf(1))` is false.
    if let (Some(class_a), Some(class_b)) = (class_a, class_b)
        && class_a != class_b
    {
        return false;
    }
    match (a, b) {
        (JValue::Int(x), JValue::Int(y)) => x == y,
        (JValue::Long(x), JValue::Long(y)) => x == y,
        (JValue::Double(x), JValue::Double(y)) => {
            x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan())
        }
        (JValue::Float(x), JValue::Float(y)) => {
            x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan())
        }
        (JValue::Ref(None), JValue::Ref(None)) => true,
        (JValue::Ref(Some(x)), JValue::Ref(Some(y))) => {
            x == y
                || matches!(
                    (heap.get(x), heap.get(y)),
                    (
                        Some(HeapObject::JavaString(sx)),
                        Some(HeapObject::JavaString(sy)),
                    ) if sx == sy
                )
        }
        _ => false,
    }
}

/// A stored value's Java hash code, for `ArrayList.hashCode`.
#[allow(clippy::cast_possible_wrap)]
/// `value.hashCode()` for every value the VM can hash without running Java.
/// A user object hashes by identity — [`Interpreter::java_hash_code`]
/// overrides that with its `hashCode`.
pub(crate) fn native_hash(heap: &Heap, value: JValue) -> i32 {
    let (class, value) = unboxed(heap, value);
    // `Boolean` is the one wrapper whose hash is not its value.
    if class == Some("java/lang/Boolean") {
        return if value == JValue::Int(0) { 1237 } else { 1231 };
    }
    match value {
        // Integer, Short, Byte and Character all hash to their value.
        JValue::Int(v) => v,
        JValue::Long(v) => fold_to_int(v),
        JValue::Double(v) => java_double_hash(v),
        JValue::Float(v) => float_to_int_bits(v),
        JValue::Ref(None) => 0,
        JValue::Ref(Some(reference)) => match heap.get(reference) {
            Some(HeapObject::JavaString(units)) => units.iter().fold(0i32, |hash, unit| {
                hash.wrapping_mul(31).wrapping_add(i32::from(*unit))
            }),
            // Identity hash (arbitrary in Java too).
            _ => reference.cast_signed(),
        },
    }
}

/// `(int) (v ^ (v >>> 32))`, how `Long` and `Double` hash a 64-bit value.
pub(crate) fn fold_to_int(value: i64) -> i32 {
    (value ^ (value.cast_unsigned() >> 32).cast_signed()) as i32
}

/// `Float.floatToIntBits`, which collapses every NaN payload to one.
fn float_to_int_bits(value: f32) -> i32 {
    if value.is_nan() {
        f32::NAN.to_bits().cast_signed()
    } else {
        value.to_bits().cast_signed()
    }
}

/// `Some(true)` for a method whose descriptor returns `boolean`, else `None`
/// (a void method). Lets one arm serve both an appending `boolean` form
/// (`offer`) and a void one (`addLast`) without unbalancing the caller's stack.
fn boolean_return_if(descriptor: &str) -> Option<JValue> {
    descriptor.ends_with(")Z").then_some(JValue::Int(1))
}

/// `java.util.ArrayDeque` forbids null elements: every insertion throws
/// `NullPointerException` on a null, unlike the null-tolerant `LinkedList`
/// backing it shares. Guards the single-element inserts and — best effort for
/// the common list-backed source — `addAll`, before the shared `list_method`
/// performs the insertion.
fn array_deque_reject_null(heap: &Heap, method: &str, args: &[JValue]) -> Result<(), VmError> {
    let inserts_single = matches!(
        method,
        "add" | "offer" | "addFirst" | "addLast" | "offerFirst" | "offerLast" | "push"
    );
    if inserts_single && matches!(args.first(), Some(JValue::Ref(None))) {
        return Err(throw("java.lang.NullPointerException"));
    }
    if method == "addAll"
        && let Some(JValue::Ref(Some(source))) = args.first()
        && heap
            .list_values(*source)
            .is_some_and(|values| values.iter().any(|v| matches!(v, JValue::Ref(None))))
    {
        return Err(throw("java.lang.NullPointerException"));
    }
    Ok(())
}

/// The `java.util.Stack` LIFO operations, which differ from the like-named
/// deque/list methods: `push`/`pop`/`peek` act on the TOP (the vector's end,
/// not a deque's head), and an empty `pop`/`peek` throws `EmptyStackException`.
/// Every one of them returns a value, so `Ok(None)` unambiguously means "not a
/// stack method" and the caller falls through to the shared `list_method` (a
/// Stack is a `List`). `search` compares elements, so it lives in the
/// interpreter with the other element-comparing list methods.
fn stack_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let Some(values) = heap.list_values_mut(receiver) else {
        unreachable!("stack receiver checked by caller")
    };
    let answer = match (method, args) {
        // `push` returns the pushed element (not a boolean like `add`).
        ("push", [value]) => {
            let value = *value;
            values.push(value);
            value
        }
        ("pop", []) => values
            .pop()
            .ok_or_else(|| throw("java.util.EmptyStackException"))?,
        ("peek", []) => values
            .last()
            .copied()
            .ok_or_else(|| throw("java.util.EmptyStackException"))?,
        ("empty", []) => JValue::Int(i32::from(values.is_empty())),
        _ => return Ok(None),
    };
    Ok(Some(answer))
}

/// `java.util.ArrayList` methods, and the positional `java.util.LinkedList`
/// (`Queue`/`Deque`) methods — both store their elements the same way, so the
/// index-and-end operations that need no element comparison share this layer.
/// (`contains`/`indexOf`/`remove(Object)`/`equals` compare elements, so they
/// live in the interpreter, which can call a user `equals`.)
#[allow(clippy::too_many_lines)] // one arm per documented method
/// The map a `keySet()`/`values()` view iterates, if `source` is one.
fn view_map(heap: &Heap, source: HeapRef) -> Option<HeapRef> {
    match heap.get(source) {
        Some(HeapObject::MapView { map, .. }) => Some(*map),
        _ => None,
    }
}

/// The element count of a collection an iterator walks (list, set, or map view).
/// Throw `ConcurrentModificationException` if the collection changed size
/// since the iterator (or for-each loop) last agreed with it.
///
/// This is the fail-fast check the JDK spells `checkForComodification`. It
/// exists so a program that mutates a collection while walking it fails HERE,
/// loudly, instead of silently skipping an element — before this, removing
/// during a for-each quietly skipped the next one, and adding looped forever.
pub(crate) fn check_comodification(
    heap: &Heap,
    source: HeapRef,
    expected_len: usize,
) -> Result<(), VmError> {
    if iterated_len(heap, source) == expected_len {
        return Ok(());
    }
    Err(throw("java.util.ConcurrentModificationException"))
}

/// `iterated_len` for callers outside this module.
pub(crate) fn iterated_len_of(heap: &Heap, source: HeapRef) -> usize {
    iterated_len(heap, source)
}

fn iterated_len(heap: &Heap, source: HeapRef) -> usize {
    if let Some(values) = heap.list_values(source) {
        return values.len();
    }
    if let Some(map) = view_map(heap, source) {
        return iterated_len(heap, map);
    }
    // An unmodifiable wrapper iterates the collection it wraps. Missing these
    // made the length read as 0, which the comodification check saw as a
    // change and turned into a spurious CME on a perfectly ordinary for-each.
    if let Some(
        HeapObject::UnmodifiableList(inner)
        | HeapObject::UnmodifiableSet(inner)
        | HeapObject::UnmodifiableMap(inner),
    ) = heap.get(source)
    {
        return iterated_len(heap, *inner);
    }
    match heap.get(source) {
        Some(HeapObject::HashSet(entries) | HeapObject::HashMap(entries)) => entries.len(),
        Some(HeapObject::TreeSet { values, .. }) => values.len(),
        Some(HeapObject::TreeMap { entries, .. }) => entries.len(),
        _ => 0,
    }
}

/// The element at `index` in that collection's iteration order. For a `keySet()`
/// view it is the key; for a `values()` view, the value.
fn iterated_get(heap: &Heap, source: HeapRef, index: usize) -> JValue {
    if let Some(values) = heap.list_values(source) {
        return values.get(index).copied().unwrap_or(JValue::NULL);
    }
    if let Some(HeapObject::MapView { map, kind }) = heap.get(source) {
        return match kind {
            MapViewKind::Values => map_value_at(heap, *map, index),
            _ => map_key_at(heap, *map, index),
        };
    }
    match heap.get(source) {
        // A HashSet stores its elements as the KEYS of its backing map.
        Some(HeapObject::HashSet(entries)) => entries.key_at(index),
        Some(HeapObject::TreeSet { values, .. }) => {
            values.get(index).copied().unwrap_or(JValue::NULL)
        }
        _ => JValue::NULL,
    }
}

fn map_key_at(heap: &Heap, map: HeapRef, index: usize) -> JValue {
    match heap.get(map) {
        Some(HeapObject::HashMap(entries)) => entries.key_at(index),
        Some(HeapObject::TreeMap { entries, .. }) => {
            entries.get(index).map_or(JValue::NULL, |(key, _)| *key)
        }
        _ => JValue::NULL,
    }
}

fn map_value_at(heap: &Heap, map: HeapRef, index: usize) -> JValue {
    match heap.get(map) {
        Some(HeapObject::HashMap(entries)) => entries.value_at(index),
        Some(HeapObject::TreeMap { entries, .. }) => {
            entries.get(index).map_or(JValue::NULL, |(_, value)| *value)
        }
        _ => JValue::NULL,
    }
}

/// Remove the element at `index` from the collection an iterator is walking —
/// `Iterator.remove()`, which takes the last element `next()` returned out of the
/// underlying collection. Through a `keySet()`/`values()` view this writes back to
/// the map, as Java's do.
fn iterated_remove(heap: &mut Heap, source: HeapRef, index: usize) {
    if let Some(values) = heap.list_values_mut(source)
        && index < values.len()
    {
        values.remove(index);
        return;
    }
    let target = view_map(heap, source).unwrap_or(source);
    match heap.get_mut(target) {
        Some(HeapObject::HashSet(entries) | HeapObject::HashMap(entries)) => {
            entries.remove_at(index);
        }
        Some(HeapObject::TreeSet { values, .. }) if index < values.len() => {
            values.remove(index);
        }
        Some(HeapObject::TreeMap { entries, .. }) if index < entries.len() => {
            entries.remove(index);
        }
        _ => {}
    }
}

/// `java.util.Iterator`: `hasNext`/`next`/`remove` over the position the iterator
/// holds. The collection is read live, so a `remove()` (or any other mutation)
/// shows through on the next call — caturra does not model
/// `ConcurrentModificationException`.
fn iterator_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
) -> Result<Option<JValue>, VmError> {
    let Some(HeapObject::Iterator {
        source,
        index,
        last,
        expected_len,
    }) = heap.get(receiver)
    else {
        unreachable!("receiver kind checked by caller");
    };
    let (source, index, last, expected_len) = (*source, *index, *last, *expected_len);
    match method {
        // `hasNext` does NOT check for comodification — the JDK's is a bare
        // `cursor != size`. That is not an oversight to fix: it is what makes
        // removing the SECOND-TO-LAST element end a for-each silently instead
        // of throwing, because the shortened size makes `hasNext` false before
        // `next` ever gets to complain. Checking here would "improve" caturra
        // into disagreeing with every real JVM.
        "hasNext" => Ok(Some(JValue::Int(i32::from(
            index != iterated_len(heap, source),
        )))),
        "next" => {
            check_comodification(heap, source, expected_len)?;
            if index >= iterated_len(heap, source) {
                return Err(throw("java.util.NoSuchElementException"));
            }
            // An `entrySet()` iterator returns a `Map.Entry` — a reference to the
            // map and the key at this position — resolved live like the entries a
            // for-each yields.
            if let Some(HeapObject::MapView {
                map,
                kind: MapViewKind::Entries,
            }) = heap.get(source)
            {
                let (map, key) = (*map, map_key_at(heap, *map, index));
                let entry = heap.alloc(HeapObject::MapEntry { map, key });
                if let Some(HeapObject::Iterator { index, last, .. }) = heap.get_mut(receiver) {
                    *last = Some(*index);
                    *index += 1;
                }
                return Ok(Some(JValue::Ref(Some(entry))));
            }
            // A list stores primitives unboxed; box one so `next()` returns the
            // wrapper for every collection alike (a set's element is already a
            // reference). `next()` is typed `BoxedElem`, which expects this.
            let element = match iterated_get(heap, source, index) {
                primitive @ (JValue::Int(_)
                | JValue::Long(_)
                | JValue::Double(_)
                | JValue::Float(_)) => {
                    let class_name = match primitive {
                        JValue::Long(_) => "java/lang/Long",
                        JValue::Double(_) => "java/lang/Double",
                        JValue::Float(_) => "java/lang/Float",
                        _ => "java/lang/Integer",
                    };
                    let boxed = heap.alloc(HeapObject::Boxed {
                        class_name: std::rc::Rc::from(class_name),
                        value: primitive,
                    });
                    JValue::Ref(Some(boxed))
                }
                reference => reference,
            };
            if let Some(HeapObject::Iterator { index, last, .. }) = heap.get_mut(receiver) {
                *last = Some(*index);
                *index += 1;
            }
            Ok(Some(element))
        }
        "remove" => {
            let Some(position) = last else {
                return Err(throw("java.lang.IllegalStateException"));
            };
            // A stale iterator cannot remove either (the JDK checks here too).
            check_comodification(heap, source, expected_len)?;
            iterated_remove(heap, source, position);
            // The cursor steps back onto the hole so the next element is not
            // skipped, and `remove()` cannot be called twice in a row. This is
            // the ONE legal modification during iteration, so the iterator
            // re-syncs its expectation rather than tripping over itself.
            let len = iterated_len(heap, source);
            if let Some(HeapObject::Iterator {
                index,
                last,
                expected_len,
                ..
            }) = heap.get_mut(receiver)
            {
                *index = position;
                *last = None;
                *expected_len = len;
            }
            Ok(None)
        }
        other => Err(VmError::UnknownIntrinsic(format!("Iterator.{other}"))),
    }
}

#[allow(clippy::too_many_lines)] // one arm per list/queue/deque method
fn list_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let list_len = heap
        .list_values(receiver)
        .map_or_else(|| unreachable!("receiver kind checked by caller"), Vec::len);
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
    // A `Deque`/`Queue` end operation that must not run on an empty list:
    // `getFirst`/`removeFirst`/`element` throw `NoSuchElementException`, while
    // `peek`/`poll` return `null` — the caller picks by passing the throw flag.
    let end_value = |heap: &mut Heap, from_front: bool, remove: bool, throw_empty: bool| {
        let Some(values) = heap.list_values_mut(receiver) else {
            unreachable!()
        };
        if values.is_empty() {
            return if throw_empty {
                Err(throw("java.util.NoSuchElementException"))
            } else {
                Ok(Some(JValue::NULL))
            };
        }
        let at = if from_front { 0 } else { values.len() - 1 };
        let value = if remove {
            values.remove(at)
        } else {
            values[at]
        };
        Ok(Some(value))
    };
    match (method, descriptor, args) {
        ("iterator", _, []) => {
            let expected_len = iterated_len(heap, receiver);
            let iterator = heap.alloc(HeapObject::Iterator {
                source: receiver,
                index: 0,
                last: None,
                expected_len,
            });
            Ok(Some(JValue::Ref(Some(iterator))))
        }
        ("size", _, []) => Ok(Some(JValue::Int(
            i32::try_from(list_len).unwrap_or(i32::MAX),
        ))),
        ("isEmpty", _, []) => Ok(Some(JValue::Int(i32::from(list_len == 0)))),
        // `add`/`offer`/`addLast`/`offerLast` all append; `addFirst`/`offerFirst`
        // and `push` prepend. The `offer*` forms and `Queue.add` return a
        // boolean; `addFirst`/`addLast`/`push` are void — the descriptor says
        // which, and returning a value for a void method would unbalance the
        // stack.
        (
            "add" | "offer" | "addLast" | "offerLast",
            "(Ljava/lang/Object;)Z" | "(Ljava/lang/Object;)V",
            [value],
        ) => {
            let value = *value;
            if let Some(values) = heap.list_values_mut(receiver) {
                values.push(value);
            }
            Ok(boolean_return_if(descriptor))
        }
        ("addFirst" | "offerFirst" | "push", _, [value]) => {
            let value = *value;
            if let Some(values) = heap.list_values_mut(receiver) {
                values.insert(0, value);
            }
            Ok(boolean_return_if(descriptor))
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
            if let Some(values) = heap.list_values_mut(receiver) {
                values.insert(at, value);
            }
            Ok(None)
        }
        ("get", _, [JValue::Int(index)]) => {
            let at = check(*index, list_len)?;
            Ok(Some(
                heap.list_values(receiver).map_or(JValue::NULL, |v| v[at]),
            ))
        }
        ("set", _, [JValue::Int(index), value]) => {
            let at = check(*index, list_len)?;
            let value = *value;
            let Some(values) = heap.list_values_mut(receiver) else {
                unreachable!()
            };
            let previous = values[at];
            values[at] = value;
            Ok(Some(previous))
        }
        ("remove", _, [JValue::Int(index)]) => {
            let at = check(*index, list_len)?;
            let Some(values) = heap.list_values_mut(receiver) else {
                unreachable!()
            };
            Ok(Some(values.remove(at)))
        }
        // Positional ends. `peek*`/`poll*` are null-on-empty; `getFirst`/
        // `getLast`/`removeFirst`/`removeLast`/`element`/`pop`/`remove()` throw.
        ("peek" | "peekFirst", _, []) => end_value(heap, true, false, false),
        ("peekLast", _, []) => end_value(heap, false, false, false),
        ("poll" | "pollFirst", _, []) => end_value(heap, true, true, false),
        ("pollLast", _, []) => end_value(heap, false, true, false),
        ("getFirst" | "element", _, []) => end_value(heap, true, false, true),
        ("getLast", _, []) => end_value(heap, false, false, true),
        ("removeFirst" | "pop" | "remove", _, []) => end_value(heap, true, true, true),
        ("removeLast", _, []) => end_value(heap, false, true, true),
        ("clear", _, []) => {
            if let Some(values) = heap.list_values_mut(receiver) {
                values.clear();
            }
            Ok(None)
        }
        ("addAll", "(Ljava/util/Collection;)Z", [JValue::Ref(other)]) => {
            let other = other.ok_or_else(|| throw("java.lang.NullPointerException"))?;
            let incoming = match heap.list_values(other) {
                Some(values) => values.clone(),
                _ => return Err(throw("java.lang.ClassCastException: not a list")),
            };
            let changed = !incoming.is_empty();
            if let Some(values) = heap.list_values_mut(receiver) {
                values.extend(incoming);
            }
            Ok(Some(JValue::Int(i32::from(changed))))
        }
        ("addAll", "(ILjava/util/Collection;)Z", [JValue::Int(index), JValue::Ref(other)]) => {
            let other = other.ok_or_else(|| throw("java.lang.NullPointerException"))?;
            let at = usize::try_from(*index)
                .ok()
                .filter(|i| *i <= list_len)
                .ok_or_else(|| {
                    throw(format!(
                        "java.lang.IndexOutOfBoundsException: Index {index} out of bounds \
                         for length {list_len}"
                    ))
                })?;
            let incoming = match heap.list_values(other) {
                Some(values) => values.clone(),
                _ => return Err(throw("java.lang.ClassCastException: not a list")),
            };
            let changed = !incoming.is_empty();
            if let Some(values) = heap.list_values_mut(receiver) {
                for (offset, value) in incoming.into_iter().enumerate() {
                    values.insert(at + offset, value);
                }
            }
            Ok(Some(JValue::Int(i32::from(changed))))
        }
        // Capacity hints: real methods, observable-free here.
        ("ensureCapacity", _, [JValue::Int(_)]) | ("trimToSize", _, []) => Ok(None),
        _ => Err(VmError::UnknownIntrinsic(format!(
            "List.{method}{descriptor}"
        ))),
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
        // Java returns long; caturra surfaces int (virtual files are small).
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

/// `java.nio.file.Path` methods. `getFileName`/`getParent` return a `Path`;
/// `getParent` is null when the path has no directory part.
fn path_method(
    heap: &mut Heap,
    receiver: HeapRef,
    method: &str,
) -> Result<Option<JValue>, VmError> {
    let path = match heap.get(receiver) {
        Some(HeapObject::Path(path)) => path.clone(),
        _ => unreachable!("receiver kind checked by caller"),
    };
    match method {
        "toString" => Ok(Some(JValue::Ref(Some(heap.alloc_string(&path))))),
        "getFileName" => {
            let name = path.rsplit('/').next().unwrap_or(&path).to_owned();
            Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Path(name))))))
        }
        "getParent" => match path.rsplit_once('/') {
            Some((parent, _)) if !parent.is_empty() => Ok(Some(JValue::Ref(Some(
                heap.alloc(HeapObject::Path(parent.to_owned())),
            )))),
            _ => Ok(Some(JValue::Ref(None))),
        },
        _ => Err(VmError::UnknownIntrinsic(format!("Path.{method}"))),
    }
}

/// `java.io.PrintWriter` methods: formatting matches `PrintStream`, but
/// output appends to the writer's file in the virtual filesystem.
/// The single character a `write(int)`/`append(char)` code denotes — its low 16
/// bits, one UTF-16 code unit.
fn char_from_code(code: i32) -> String {
    let unit = u32::try_from(code & 0xFFFF).unwrap_or(0);
    char::from_u32(unit).map(String::from).unwrap_or_default()
}

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
        "printf" | "format" => {
            let template = match args.first() {
                Some(JValue::Ref(Some(reference))) => {
                    heap.string_text(*reference).unwrap_or_default()
                }
                Some(JValue::Ref(None)) => {
                    return Err(throw("java.lang.NullPointerException: format is null"));
                }
                _ => String::new(),
            };
            let format_args = crate::format::args_from_descriptor(descriptor, &args[1..])?;
            let text = crate::format::java_format(heap, &template, &format_args)?;
            vfs.append_file(&path, text.as_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            // `format` returns the writer for chaining; `printf` is void.
            Ok((method == "format").then_some(JValue::Ref(Some(receiver))))
        }
        // `write(String)` writes the whole string; `write(int)` writes a single
        // character (its low 16 bits).
        "write" => {
            let text = if descriptor.starts_with("(I)") {
                match args.first() {
                    Some(JValue::Int(code)) => char_from_code(*code),
                    _ => String::new(),
                }
            } else {
                match args.first() {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    _ => String::new(),
                }
            };
            vfs.append_file(&path, text.as_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(None)
        }
        // `append(char)` writes the character; `append(CharSequence)` the text
        // (a null appends the four characters "null"). Returns the writer.
        "append" => {
            let text = if descriptor.starts_with("(C)") {
                match args.first() {
                    Some(JValue::Int(code)) => char_from_code(*code),
                    _ => String::new(),
                }
            } else {
                match args.first() {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    Some(JValue::Ref(None)) => String::from("null"),
                    _ => String::new(),
                }
            };
            vfs.append_file(&path, text.as_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(JValue::Ref(Some(receiver))))
        }
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

/// Intrinsic static methods, routed per class. Every method a student
/// can find in the Java 11 documentation either works or reports an
/// honest "not supported" reason at compile time.
fn is_wrapper_class(class: &str) -> bool {
    matches!(
        class,
        "java/lang/Integer"
            | "java/lang/Double"
            | "java/lang/Long"
            | "java/lang/Float"
            | "java/lang/Short"
            | "java/lang/Byte"
            | "java/lang/Character"
            | "java/lang/Boolean"
    )
}

/// Render a boxed primitive the way its wrapper's `toString` does.
pub(crate) fn boxed_to_string(class_name: &str, value: JValue) -> String {
    match (class_name, value) {
        ("java/lang/Double", JValue::Double(v)) => java_double_to_string(v),
        ("java/lang/Float", JValue::Float(v)) => java_float_to_string(v),
        ("java/lang/Long", JValue::Long(v)) => v.to_string(),
        ("java/lang/Boolean", JValue::Int(v)) => (v != 0).to_string(),
        ("java/lang/Character", JValue::Int(v)) => char::from_u32(u32::try_from(v).unwrap_or(0))
            .unwrap_or('\u{FFFD}')
            .to_string(),
        (_, JValue::Int(v)) => v.to_string(),
        (_, other) => format!("{other:?}"),
    }
}

/// Instance methods on a boxed wrapper: the unboxing accessors
/// (`intValue`, ...) and the Object methods (`toString`, `equals`,
/// `hashCode`, `compareTo`).
fn boxed_virtual(
    heap: &mut Heap,
    class_name: &str,
    value: JValue,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    // Unboxing accessors convert the stored primitive to the requested
    // numeric type.
    let as_double = |v: JValue| match v {
        JValue::Int(n) => f64::from(n),
        JValue::Long(n) => {
            #[allow(clippy::cast_precision_loss)]
            {
                n as f64
            }
        }
        JValue::Float(n) => f64::from(n),
        JValue::Double(n) => n,
        JValue::Ref(_) => 0.0,
    };
    let as_long = |v: JValue| match v {
        JValue::Int(n) => i64::from(n),
        JValue::Long(n) => n,
        #[allow(clippy::cast_possible_truncation)]
        JValue::Float(n) => n as i64,
        #[allow(clippy::cast_possible_truncation)]
        JValue::Double(n) => n as i64,
        JValue::Ref(_) => 0,
    };
    #[allow(clippy::cast_possible_truncation)]
    let as_int = |v: JValue| match v {
        JValue::Int(n) => n,
        JValue::Long(n) => n as i32,
        JValue::Float(n) => n as i32,
        JValue::Double(n) => n as i32,
        JValue::Ref(_) => 0,
    };
    match method {
        "intValue" | "shortValue" | "byteValue" | "charValue" => {
            Ok(Some(JValue::Int(as_int(value))))
        }
        "longValue" => Ok(Some(JValue::Long(as_long(value)))),
        "doubleValue" => Ok(Some(JValue::Double(as_double(value)))),
        #[allow(clippy::cast_possible_truncation)]
        "floatValue" => Ok(Some(JValue::Float(as_double(value) as f32))),
        "booleanValue" => Ok(Some(value)),
        "toString" => {
            let text = boxed_to_string(class_name, value);
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        // Each wrapper's own `hashCode`, keyed by class: `Boolean` is
        // 1231/1237 (NOT its 0/1 value), `Long`/`Double`/`Float` fold their
        // bits, and `Integer`/`Short`/`Byte`/`Character` hash to their value.
        "hashCode" => {
            let hash = if class_name == "java/lang/Boolean" {
                if value == JValue::Int(0) { 1237 } else { 1231 }
            } else {
                match value {
                    JValue::Int(n) => n,
                    JValue::Long(n) => fold_to_int(n),
                    JValue::Double(n) => java_double_hash(n),
                    JValue::Float(n) => float_to_int_bits(n),
                    JValue::Ref(_) => 0,
                }
            };
            Ok(Some(JValue::Int(hash)))
        }
        "equals" => {
            // Equal iff the other operand is a wrapper of the same class and
            // value, or a raw unboxed primitive of equal value (caturra stores
            // primitives unboxed, so an `Object` arg may arrive unboxed — Java
            // would autobox it, `Integer.equals(5)` is true).
            let equal = match args.first() {
                Some(JValue::Ref(Some(other))) => match heap.get(*other) {
                    Some(HeapObject::Boxed {
                        class_name: other_class,
                        value: other_value,
                    }) => &**other_class == class_name && values_bit_equal(value, *other_value),
                    _ => false,
                },
                Some(
                    other @ (JValue::Int(_)
                    | JValue::Long(_)
                    | JValue::Double(_)
                    | JValue::Float(_)),
                ) => values_bit_equal(value, *other),
                _ => false,
            };
            Ok(Some(JValue::Int(i32::from(equal))))
        }
        "compareTo" => {
            let other = match args.first() {
                Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::Boxed { value, .. }) => *value,
                    _ => JValue::Int(0),
                },
                _ => JValue::Int(0),
            };
            let ordering = match (value, other) {
                (JValue::Double(a), JValue::Double(b)) => a.total_cmp(&b),
                (JValue::Float(a), JValue::Float(b)) => a.total_cmp(&b),
                (JValue::Long(a), JValue::Long(b)) => a.cmp(&b),
                _ => as_long(value).cmp(&as_long(other)),
            };
            Ok(Some(JValue::Int(match ordering {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })))
        }
        _ => Err(VmError::UnknownIntrinsic(format!(
            "{class_name}.{method} on a boxed value"
        ))),
    }
}

fn values_bit_equal(a: JValue, b: JValue) -> bool {
    match (a, b) {
        (JValue::Int(x), JValue::Int(y)) => x == y,
        (JValue::Long(x), JValue::Long(y)) => x == y,
        (JValue::Double(x), JValue::Double(y)) => x.to_bits() == y.to_bits(),
        (JValue::Float(x), JValue::Float(y)) => x.to_bits() == y.to_bits(),
        _ => false,
    }
}

/// Read an image pixel file written by the host: width and height as
/// little-endian u32, then `width * height` RGB triples. `None` when the file is
/// absent or truncated — the caller then reports "no image".
fn image_bytes<'a>(
    heap: &Heap,
    vfs: &'a VirtualFileSystem,
    args: &[JValue],
) -> Option<(u32, u32, &'a [u8])> {
    let Some(JValue::Ref(Some(reference))) = args.first() else {
        return None;
    };
    let path = heap.string_text(*reference)?;
    let bytes = vfs.read_file(&path).ok()?;
    if bytes.len() < 8 {
        return None;
    }
    let width = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let height = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    let count = (width as usize)
        .checked_mul(height as usize)?
        .checked_mul(3)?;
    let rgb = bytes.get(8..8 + count)?;
    Some((width, height, rgb))
}

/// Sound samples, natively — the audio half of `system_image`, and for the same
/// reason only more so.
///
/// Samples cross the VFS as raw little-endian signed 16-bit PCM: exactly the
/// bytes the audio decoded from, `2 * len` of them. `beat.wav` is 4.9 MILLION
/// samples, so the text this used to be (`count s0 s1 …`) was 24 MB that the
/// *modelled* `Scanner` parsed one interpreted `nextInt()` at a time, and that
/// `playSound` built one interpreted `StringBuilder.append` at a time. Nobody
/// noticed, because a missing asset used to hand back silence and so no level
/// ever read a real sound.
///
/// An absent sound reads back empty, which is what lets `SoundLoader.read` throw
/// `FILE_NOT_FOUND` instead of inventing silence.
fn system_sound(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    method: &str,
    args: &[JValue],
) -> Option<JValue> {
    let Some(JValue::Ref(Some(reference))) = args.first() else {
        return None;
    };
    let path = heap.string_text(*reference)?;
    if method == "__soundSamples" {
        let samples = vfs.read_file(&path).map_or_else(
            |_| Vec::new(),
            |bytes| {
                bytes
                    .chunks_exact(2)
                    .map(|pair| f64::from(i16::from_le_bytes([pair[0], pair[1]])) / 32768.0)
                    .collect()
            },
        );
        return Some(JValue::Ref(Some(
            heap.alloc(HeapObject::DoubleArray(samples)),
        )));
    }
    // __writeSound(path, samples[])
    let Some(JValue::Ref(Some(array))) = args.get(1) else {
        return None;
    };
    let Some(HeapObject::DoubleArray(samples)) = heap.get(*array) else {
        return None;
    };
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "clamped to [-1, 1] and scaled, so it is in i16 range"
        )]
        let quantized = (clamped * 32767.0).round() as i16;
        bytes.extend_from_slice(&quantized.to_le_bytes());
    }
    let _ = vfs.write_file(&path, bytes);
    None
}

/// Image pixels, natively (`org.code.media.Image`). A 400x400 image is 160k
/// pixels; ferrying that through `PrintWriter`/`Scanner` text costs about a
/// minute in the interpreter, so the whole buffer crosses the VFS as bytes and is
/// packed/unpacked here in one call. `__imageDims`/`__imagePixels` read what the
/// host preloaded; `__writeImage` sends an edited image back out to be drawn.
fn system_image(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let ints = |heap: &mut Heap, values: Vec<i32>| {
        Ok(Some(JValue::Ref(Some(
            heap.alloc(HeapObject::IntArray(IntKind::Int, values)),
        ))))
    };
    match method {
        "__imageDims" => {
            let dims = match image_bytes(heap, vfs, args) {
                Some((width, height, _)) => vec![
                    i32::try_from(width).unwrap_or(0),
                    i32::try_from(height).unwrap_or(0),
                ],
                None => Vec::new(),
            };
            ints(heap, dims)
        }
        "__imagePixels" => {
            let pixels = match image_bytes(heap, vfs, args) {
                Some((width, height, rgb)) => (0..(width as usize).saturating_mul(height as usize))
                    .map(|i| {
                        let (r, g, b) = (rgb[i * 3], rgb[i * 3 + 1], rgb[i * 3 + 2]);
                        (i32::from(r) << 16) | (i32::from(g) << 8) | i32::from(b)
                    })
                    .collect(),
                None => Vec::new(),
            };
            ints(heap, pixels)
        }
        // __writeImage(path, width, height, packed[])
        _ => {
            let Some(JValue::Ref(Some(reference))) = args.first() else {
                return Ok(None);
            };
            let Some(path) = heap.string_text(*reference) else {
                return Ok(None);
            };
            let (Some(JValue::Int(width)), Some(JValue::Int(height))) = (args.get(1), args.get(2))
            else {
                return Ok(None);
            };
            let Some(JValue::Ref(Some(array))) = args.get(3) else {
                return Ok(None);
            };
            let Some(HeapObject::IntArray(_, packed)) = heap.get(*array) else {
                return Ok(None);
            };
            let (width, height) = (
                u32::try_from(*width).unwrap_or(0),
                u32::try_from(*height).unwrap_or(0),
            );
            let count = (width as usize).saturating_mul(height as usize);
            let mut bytes = Vec::with_capacity(8 + count * 3);
            bytes.extend_from_slice(&width.to_le_bytes());
            bytes.extend_from_slice(&height.to_le_bytes());
            for i in 0..count {
                let value = packed.get(i).copied().unwrap_or(0);
                bytes.push(u8::try_from((value >> 16) & 0xff).unwrap_or(0));
                bytes.push(u8::try_from((value >> 8) & 0xff).unwrap_or(0));
                bytes.push(u8::try_from(value & 0xff).unwrap_or(0));
            }
            let _ = vfs.write_file(&path, bytes);
            Ok(None)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn invoke_static(
    heap: &mut Heap,
    rng: &mut JavaRng,
    console: &mut dyn ConsoleIo,
    vfs: &mut VirtualFileSystem,
    class: &str,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    // Autoboxing: the compiler emits `Wrapper.valueOf(prim)LWrapper;`
    // (a wrapper return) to box. User `Integer.valueOf(7)` is compiled
    // with an `int` return and falls through unboxed.
    if method == "valueOf"
        && args.len() == 1
        && is_wrapper_class(class)
        && descriptor.ends_with(&format!("L{class};"))
        && !matches!(args[0], JValue::Ref(_))
    {
        let reference = heap.box_wrapper(class, args[0]);
        return Ok(Some(JValue::Ref(Some(reference))));
    }
    // `new Integer(5)` must mint a DISTINCT object every time (JLS §15.9.4:
    // a class instance creation expression always creates a new object), so
    // `new Integer(5) == new Integer(5)` is FALSE while
    // `Integer.valueOf(5) == Integer.valueOf(5)` is true. The compiler emits
    // this caturra-internal name for the deprecated constructors precisely so
    // the autoboxing cache above is bypassed.
    if method == "__newWrapper" && args.len() == 1 && is_wrapper_class(class) {
        let reference = heap.alloc(HeapObject::Boxed {
            class_name: std::rc::Rc::from(class),
            value: args[0],
        });
        return Ok(Some(JValue::Ref(Some(reference))));
    }
    match class {
        "java/lang/Math" => math_static(rng, method, args),
        "java/lang/Integer" => integer_static(heap, method, args),
        "java/lang/Double" => double_static(heap, method, args),
        "java/lang/Character" => character_static(heap, method, args),
        "java/lang/Boolean" => boolean_static(heap, method, args),
        "java/lang/Long" => long_static(heap, method, args),
        "java/lang/Float" => float_static(heap, method, args),
        "java/lang/Short" => small_int_static(heap, "Short", method, args),
        "java/lang/Byte" => small_int_static(heap, "Byte", method, args),
        "java/lang/System" => match method {
            "arraycopy" => system_arraycopy(heap, args),
            // The JVM's is system-dependent; caturra always runs where a line
            // ends with a newline.
            "lineSeparator" => Ok(Some(JValue::Ref(Some(heap.alloc_string("\n"))))),
            "currentTimeMillis" => Ok(Some(JValue::Long(console.now_millis()))),
            "nanoTime" => Ok(Some(JValue::Long(
                console.now_millis().wrapping_mul(1_000_000),
            ))),
            // Terminate the program: unwind the whole stack uncatchably (no
            // catch/finally runs, matching the real JVM) with the status code.
            "exit" => {
                let code = match args.first() {
                    Some(JValue::Int(code)) => *code,
                    _ => 0,
                };
                Err(VmError::SystemExit(code))
            }
            // Standard-out capture for org.code.validation's SystemOutTestRunner.
            "__captureStart" => {
                console.begin_capture();
                Ok(None)
            }
            "__captureEnd" => {
                let messages = console.take_capture();
                let refs: Vec<JValue> = messages
                    .iter()
                    .map(|m| JValue::Ref(Some(heap.alloc_string(m))))
                    .collect();
                Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::RefArray(
                    String::from("[Ljava/lang/String;"),
                    refs,
                ))))))
            }
            // Swing event pump: render the tree (arg 0), block for the next
            // event, return its payload String — or null to end the loop.
            "__uiAwait" => {
                let tree = match args.first() {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    _ => String::new(),
                };
                match console.ui_await_event(&tree) {
                    Some(payload) => Ok(Some(JValue::Ref(Some(heap.alloc_string(&payload))))),
                    None => Ok(Some(JValue::Ref(None))),
                }
            }
            // Blocking JOptionPane dialog: kind (arg 0) + message (arg 1);
            // returns the response String, or null when dismissed.
            "__uiDialog" => {
                let arg_str = |i: usize| match args.get(i) {
                    Some(JValue::Ref(Some(reference))) => {
                        heap.string_text(*reference).unwrap_or_default()
                    }
                    _ => String::new(),
                };
                let (kind, message) = (arg_str(0), arg_str(1));
                match console.ui_dialog(&kind, &message) {
                    Some(response) => Ok(Some(JValue::Ref(Some(heap.alloc_string(&response))))),
                    None => Ok(Some(JValue::Ref(None))),
                }
            }
            "__imageDims" | "__imagePixels" | "__writeImage" => {
                system_image(heap, vfs, method, args)
            }
            "__soundSamples" | "__writeSound" => Ok(system_sound(heap, vfs, method, args)),
            _ => Err(VmError::UnknownIntrinsic(format!("System.{method}"))),
        },
        "java/lang/String" => string_static(heap, method, descriptor, args),
        // `Path.of` / `Paths.get` — wrap a path string as a Path.
        "java/nio/file/Path" | "java/nio/file/Paths" => {
            let text = arg_string(heap, &args[0])?;
            Ok(Some(JValue::Ref(Some(heap.alloc(HeapObject::Path(text))))))
        }
        "java/nio/file/Files" => files_static(heap, vfs, method, args),
        _ => Err(VmError::UnknownIntrinsic(format!("{class}.{method}"))),
    }
}

/// The text of a `String` argument (null throws NPE).
fn arg_string(heap: &Heap, value: &JValue) -> Result<String, VmError> {
    match value {
        JValue::Ref(Some(reference)) => Ok(heap.string_text(*reference).unwrap_or_default()),
        JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
        _ => Err(throw("java.lang.VerifyError: expected a String argument")),
    }
}

/// The path string behind a `java.nio.file.Path` argument.
fn path_arg(heap: &Heap, value: &JValue) -> Result<String, VmError> {
    match value {
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            Some(HeapObject::Path(path)) => Ok(path.clone()),
            _ => Err(throw("java.lang.ClassCastException: not a Path")),
        },
        JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
        _ => Err(throw("java.lang.VerifyError: expected a Path argument")),
    }
}

/// `java.nio.file.Files` static methods, over the virtual filesystem.
fn files_static(
    heap: &mut Heap,
    vfs: &mut VirtualFileSystem,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let path = path_arg(heap, &args[0])?;
    let not_found = || throw(format!("java.nio.file.NoSuchFileException: {path}"));
    match method {
        "readString" => {
            let content = vfs.read_file(&path).map_err(|_| not_found())?.to_vec();
            let text = String::from_utf8_lossy(&content).into_owned();
            Ok(Some(JValue::Ref(Some(heap.alloc_string(&text)))))
        }
        "readAllLines" => {
            let content = vfs.read_file(&path).map_err(|_| not_found())?.to_vec();
            let text = String::from_utf8_lossy(&content).into_owned();
            let lines: Vec<JValue> = text
                .lines()
                .map(|line| JValue::Ref(Some(heap.alloc_string(line))))
                .collect();
            Ok(Some(JValue::Ref(Some(
                heap.alloc(HeapObject::ArrayList(lines)),
            ))))
        }
        "writeString" => {
            let text = arg_string(heap, &args[1])?;
            vfs.write_file(&path, text.into_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(args[0]))
        }
        "write" => {
            let lines = match args.get(1) {
                Some(JValue::Ref(Some(reference))) => heap.list_values(*reference).cloned(),
                _ => None,
            }
            .ok_or_else(|| throw("java.lang.NullPointerException"))?;
            let mut text = String::new();
            for line in &lines {
                if let JValue::Ref(Some(reference)) = line {
                    text.push_str(&heap.string_text(*reference).unwrap_or_default());
                }
                text.push('\n');
            }
            vfs.write_file(&path, text.into_bytes())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(args[0]))
        }
        "exists" | "isRegularFile" => Ok(Some(JValue::Int(i32::from(vfs.exists(&path))))),
        "notExists" => Ok(Some(JValue::Int(i32::from(!vfs.exists(&path))))),
        "isDirectory" => Ok(Some(JValue::Int(i32::from(vfs.is_directory(&path))))),
        "delete" => {
            vfs.remove(&path).map_err(|_| not_found())?;
            Ok(None)
        }
        "createFile" => {
            vfs.write_file(&path, Vec::new())
                .map_err(|e| throw(format!("java.io.IOException: {e}")))?;
            Ok(Some(args[0]))
        }
        "createDirectory" => {
            let _ = vfs.mkdir(&path);
            Ok(Some(args[0]))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Files.{method}"))),
    }
}

/// The Java type name for a field descriptor: `I` -> `int`,
/// `Ljava/lang/String;` -> `java.lang.String`, `[I` -> `int[]`.
#[must_use]
pub fn type_name_of_descriptor(descriptor: &str) -> String {
    let base = descriptor.trim_start_matches('[');
    let dims = descriptor.len() - base.len();
    let mut name = match base {
        "I" => String::from("int"),
        "J" => String::from("long"),
        "D" => String::from("double"),
        "F" => String::from("float"),
        "Z" => String::from("boolean"),
        "C" => String::from("char"),
        "S" => String::from("short"),
        "B" => String::from("byte"),
        "V" => String::from("void"),
        other if other.starts_with('L') && other.ends_with(';') => {
            other[1..other.len() - 1].replace('/', ".")
        }
        other => other.to_owned(),
    };
    for _ in 0..dims {
        name.push_str("[]");
    }
    name
}

/// Java's canonical `Field.toString()`:
/// `<modifiers> <type> <DeclaringClass>.<name>`.
#[must_use]
pub fn field_to_string(declaring: &str, name: &str, descriptor: &str, access: u16) -> String {
    use caturra_classfile::FieldAccessFlags as F;
    let mut out = String::new();
    for (flag, word) in [
        (F::PUBLIC, "public"),
        (F::PRIVATE, "private"),
        (F::PROTECTED, "protected"),
        (F::STATIC, "static"),
        (F::FINAL, "final"),
        (F::TRANSIENT, "transient"),
        (F::VOLATILE, "volatile"),
    ] {
        if access & flag != 0 {
            out.push_str(word);
            out.push(' ');
        }
    }
    out.push_str(&type_name_of_descriptor(descriptor));
    out.push(' ');
    out.push_str(declaring);
    out.push('.');
    out.push_str(name);
    out
}

/// The Java type names of a method descriptor's parameters:
/// `(Ljava/lang/String;I)V` -> `["java.lang.String", "int"]`.
#[must_use]
pub fn param_type_names(descriptor: &str) -> Vec<String> {
    let mut names = Vec::new();
    let inner = match (descriptor.find('('), descriptor.find(')')) {
        (Some(open), Some(close)) if open < close => &descriptor[open + 1..close],
        _ => return names,
    };
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        while i < bytes.len() && bytes[i] == b'[' {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'L' {
            while i < bytes.len() && bytes[i] != b';' {
                i += 1;
            }
            i += 1; // consume ';'
        } else {
            i += 1; // a single primitive char
        }
        names.push(type_name_of_descriptor(&inner[start..i]));
    }
    names
}

/// Java's canonical `Constructor.toString()`:
/// `<modifiers> <DeclaringClass>(<param types>)`.
#[must_use]
pub fn constructor_to_string(declaring: &str, descriptor: &str, access: u16) -> String {
    use caturra_classfile::MethodAccessFlags as M;
    let mut out = String::new();
    for (flag, word) in [
        (M::PUBLIC, "public"),
        (M::PRIVATE, "private"),
        (M::PROTECTED, "protected"),
    ] {
        if access & flag != 0 {
            out.push_str(word);
            out.push(' ');
        }
    }
    out.push_str(declaring);
    out.push('(');
    out.push_str(&param_type_names(descriptor).join(","));
    out.push(')');
    out
}

/// Java's `Math.max`/`min` double semantics: NaN wins, `+0.0 > -0.0`.
#[allow(clippy::float_cmp)]
fn java_double_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        // +0.0 beats -0.0.
        return if a.is_sign_positive() { a } else { b };
    }
    if a > b { a } else { b }
}

#[allow(clippy::float_cmp)]
fn java_double_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        return f64::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() { a } else { b };
    }
    if a < b { a } else { b }
}

/// `Math.floorDiv` with Java's toward-negative-infinity semantics.
fn java_floor_div(a: i32, b: i32) -> Result<i32, VmError> {
    if b == 0 {
        return Err(throw("java.lang.ArithmeticException: / by zero"));
    }
    let quotient = a.wrapping_div(b);
    if (a ^ b) < 0 && quotient.wrapping_mul(b) != a {
        Ok(quotient - 1)
    } else {
        Ok(quotient)
    }
}

/// The `long` form of [`java_floor_div`].
fn java_floor_div_long(a: i64, b: i64) -> Result<i64, VmError> {
    if b == 0 {
        return Err(throw("java.lang.ArithmeticException: / by zero"));
    }
    let quotient = a.wrapping_div(b);
    if (a ^ b) < 0 && quotient.wrapping_mul(b) != a {
        Ok(quotient - 1)
    } else {
        Ok(quotient)
    }
}

fn overflow() -> VmError {
    throw("java.lang.ArithmeticException: integer overflow")
}

/// `Math.round(double)` — half-up toward positive infinity, computed on the
/// bit pattern exactly as JDK 11 does. The naive `(a + 0.5).floor()` rounds
/// `0.49999999999999994` UP, because adding `0.5` to it in `double` overflows
/// to `1.0`; this does not. The out-of-`[0,63]` branch is the `(long) a` cast,
/// which Rust's saturating float→int matches (NaN→0, ±∞→`i64::MIN`/`i64::MAX`).
fn java_round_double(a: f64) -> i64 {
    const SIGNIFICAND_WIDTH: i64 = 53;
    const EXP_BIAS: i64 = 1023;
    const EXP_BIT_MASK: i64 = 0x7FF0_0000_0000_0000;
    const SIGNIF_BIT_MASK: i64 = 0x000F_FFFF_FFFF_FFFF;
    let bits = a.to_bits().cast_signed();
    let biased_exp = (bits & EXP_BIT_MASK) >> (SIGNIFICAND_WIDTH - 1);
    let shift = (SIGNIFICAND_WIDTH - 2 + EXP_BIAS) - biased_exp;
    if (shift & -64) == 0 {
        let mut r = (bits & SIGNIF_BIT_MASK) | (SIGNIF_BIT_MASK + 1);
        if bits < 0 {
            r = -r;
        }
        ((r >> shift) + 1) >> 1
    } else {
        a as i64
    }
}

/// `Math.round(float)` — the 32-bit twin of [`java_round_double`], returning
/// an `int`.
fn java_round_float(a: f32) -> i32 {
    const SIGNIFICAND_WIDTH: i32 = 24;
    const EXP_BIAS: i32 = 127;
    const EXP_BIT_MASK: i32 = 0x7F80_0000;
    const SIGNIF_BIT_MASK: i32 = 0x007F_FFFF;
    let bits = a.to_bits().cast_signed();
    let biased_exp = (bits & EXP_BIT_MASK) >> (SIGNIFICAND_WIDTH - 1);
    let shift = (SIGNIFICAND_WIDTH - 2 + EXP_BIAS) - biased_exp;
    if (shift & -32) == 0 {
        let mut r = (bits & SIGNIF_BIT_MASK) | (SIGNIF_BIT_MASK + 1);
        if bits < 0 {
            r = -r;
        }
        ((r >> shift) + 1) >> 1
    } else {
        a as i32
    }
}

/// Java's `Math.pow`. Rust's `f64::powf` (glibc) is correctly rounded except on
/// exact ties, where it rounds half-away from zero while the JDK rounds half-to-
/// even — e.g. `17^13`, an odd integer sitting exactly on a double midpoint. The
/// JDK's `Math.pow` is correctly rounded for *every* integer exponent (verified
/// against a real JVM), so an integer base raised to a non-negative integer
/// power is computed exactly: the exact `u128` power cast to `f64` rounds half-
/// to-even, matching the JDK. Everything else defers to glibc, which already
/// matches the JDK bit-for-bit for non-integer / non-representable results.
#[allow(clippy::float_cmp)]
fn java_pow(a: f64, b: f64) -> f64 {
    // Java's one deviation from IEEE 754: |x| == 1 with an infinite exponent.
    if a.abs() == 1.0 && b.is_infinite() {
        return f64::NAN;
    }
    exact_integer_pow(a, b).unwrap_or_else(|| a.powf(b))
}

/// The correctly-rounded value of integer `a` raised to a non-negative integer
/// power `b`, when the exact power fits in `u128`; otherwise `None` (defer to
/// glibc). Negative exponents and |base| < 2 already match the JDK bit-for-bit,
/// so they are left to `powf`.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp
)]
fn exact_integer_pow(a: f64, b: f64) -> Option<f64> {
    if a.fract() != 0.0 || b.fract() != 0.0 || !(1.0..=127.0).contains(&b) {
        return None;
    }
    let mag = a.abs();
    if mag < 2.0 {
        return None;
    }
    let exp = b as u32;
    let base = mag as u128;
    let mut acc: u128 = 1;
    for _ in 0..exp {
        acc = acc.checked_mul(base)?;
    }
    // `u128 as f64` rounds to nearest, ties to even — the correctly-rounded
    // result the JDK produces.
    let val = acc as f64;
    Some(if a < 0.0 && exp % 2 == 1 { -val } else { val })
}

#[allow(clippy::too_many_lines)] // one arm per documented method
#[allow(clippy::float_cmp, clippy::many_single_char_names)]
fn math_static(
    rng: &mut JavaRng,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let d = |v: f64| Ok(Some(JValue::Double(v)));
    let i = |v: i32| Ok(Some(JValue::Int(v)));
    match (method, args) {
        ("abs", [JValue::Int(v)]) => i(v.wrapping_abs()),
        ("abs", [JValue::Long(v)]) => Ok(Some(JValue::Long(v.wrapping_abs()))),
        ("abs", [JValue::Float(v)]) => Ok(Some(JValue::Float(v.abs()))),
        ("max", [JValue::Float(a), JValue::Float(b)]) => {
            Ok(Some(JValue::Float(java_float_max(*a, *b))))
        }
        ("min", [JValue::Float(a), JValue::Float(b)]) => {
            Ok(Some(JValue::Float(java_float_min(*a, *b))))
        }
        ("signum", [JValue::Float(v)]) => Ok(Some(JValue::Float(if *v == 0.0 || v.is_nan() {
            *v
        } else {
            v.signum()
        }))),
        ("max", [JValue::Long(a), JValue::Long(b)]) => Ok(Some(JValue::Long((*a).max(*b)))),
        ("min", [JValue::Long(a), JValue::Long(b)]) => Ok(Some(JValue::Long((*a).min(*b)))),
        ("toIntExact", [JValue::Long(v)]) => {
            i32::try_from(*v).map_or_else(|_| Err(overflow()), |v| Ok(Some(JValue::Int(v))))
        }
        ("multiplyHigh", [JValue::Long(a), JValue::Long(b)]) => {
            let product = i128::from(*a) * i128::from(*b);
            #[allow(clippy::cast_possible_truncation)]
            Ok(Some(JValue::Long((product >> 64) as i64)))
        }
        // The exact 64-bit product of two ints, which `int * int` would wrap.
        ("multiplyFull", [JValue::Int(a), JValue::Int(b)]) => {
            Ok(Some(JValue::Long(i64::from(*a) * i64::from(*b))))
        }
        ("scalb", [JValue::Double(value), JValue::Int(scale)]) => {
            d(java_scalb_double(*value, *scale))
        }
        ("scalb", [JValue::Float(value), JValue::Int(scale)]) => {
            Ok(Some(JValue::Float(java_scalb_float(*value, *scale))))
        }
        ("abs", [JValue::Double(v)]) => d(v.abs()),
        ("sqrt", [JValue::Double(v)]) => d(v.sqrt()),
        ("cbrt", [JValue::Double(v)]) => d(v.cbrt()),
        ("pow", [JValue::Double(a), JValue::Double(b)]) => d(java_pow(*a, *b)),
        ("hypot", [JValue::Double(a), JValue::Double(b)]) => d(a.hypot(*b)),
        ("max", [JValue::Int(a), JValue::Int(b)]) => i((*a).max(*b)),
        ("max", [JValue::Double(a), JValue::Double(b)]) => d(java_double_max(*a, *b)),
        ("min", [JValue::Int(a), JValue::Int(b)]) => i((*a).min(*b)),
        ("min", [JValue::Double(a), JValue::Double(b)]) => d(java_double_min(*a, *b)),
        ("random", []) => d(rng.next_double()),
        ("floor", [JValue::Double(v)]) => d(v.floor()),
        ("ceil", [JValue::Double(v)]) => d(v.ceil()),
        ("rint", [JValue::Double(v)]) => d(v.round_ties_even()),
        ("round", [JValue::Double(v)]) => Ok(Some(JValue::Long(java_round_double(*v)))),
        ("round", [JValue::Float(v)]) => i(java_round_float(*v)),
        ("sin", [JValue::Double(v)]) => d(v.sin()),
        ("cos", [JValue::Double(v)]) => d(v.cos()),
        ("tan", [JValue::Double(v)]) => d(v.tan()),
        ("asin", [JValue::Double(v)]) => d(v.asin()),
        ("acos", [JValue::Double(v)]) => d(v.acos()),
        ("atan", [JValue::Double(v)]) => d(v.atan()),
        ("atan2", [JValue::Double(a), JValue::Double(b)]) => d(a.atan2(*b)),
        ("sinh", [JValue::Double(v)]) => d(v.sinh()),
        ("cosh", [JValue::Double(v)]) => d(v.cosh()),
        ("tanh", [JValue::Double(v)]) => d(v.tanh()),
        ("exp", [JValue::Double(v)]) => d(v.exp()),
        ("expm1", [JValue::Double(v)]) => d(v.exp_m1()),
        ("log", [JValue::Double(v)]) => d(v.ln()),
        ("log10", [JValue::Double(v)]) => d(v.log10()),
        ("log1p", [JValue::Double(v)]) => d(v.ln_1p()),
        ("signum", [JValue::Double(v)]) => {
            // Rust's signum maps ±0 to ±1; Java keeps ±0 and NaN.
            d(if *v == 0.0 || v.is_nan() {
                *v
            } else {
                v.signum()
            })
        }
        ("toDegrees", [JValue::Double(v)]) => d(v.to_degrees()),
        ("toRadians", [JValue::Double(v)]) => d(v.to_radians()),
        ("copySign", [JValue::Double(a), JValue::Double(b)]) => d(a.copysign(*b)),
        ("ulp", [JValue::Double(v)]) => {
            let v = v.abs();
            d(if v.is_nan() {
                f64::NAN
            } else if v.is_infinite() {
                f64::INFINITY
            } else if v == f64::MAX {
                f64::MAX - f64::from_bits(f64::MAX.to_bits() - 1)
            } else {
                v.next_up() - v
            })
        }
        ("nextUp", [JValue::Double(v)]) => d(v.next_up()),
        ("nextDown", [JValue::Double(v)]) => d(v.next_down()),
        ("nextUp", [JValue::Float(v)]) => Ok(Some(JValue::Float(v.next_up()))),
        ("nextDown", [JValue::Float(v)]) => Ok(Some(JValue::Float(v.next_down()))),
        ("nextAfter", [JValue::Double(start), JValue::Double(direction)]) => {
            d(if start.is_nan() || direction.is_nan() {
                f64::NAN
            } else if start == direction {
                *direction
            } else if direction > start {
                start.next_up()
            } else {
                start.next_down()
            })
        }
        ("fma", [JValue::Double(a), JValue::Double(b), JValue::Double(c)]) => d(a.mul_add(*b, *c)),
        ("IEEEremainder", [JValue::Double(a), JValue::Double(b)]) => {
            let quotient = (a / b).round_ties_even();
            d(if quotient.is_infinite() || quotient.is_nan() {
                f64::NAN
            } else {
                a - quotient * b
            })
        }
        ("getExponent", [JValue::Double(v)]) => {
            let bits = (v.to_bits() >> 52) & 0x7FF;
            i(if v.is_nan() || v.is_infinite() {
                1024
            } else if bits == 0 {
                -1023 // zero and subnormals
            } else {
                i32::try_from(bits).unwrap_or(0) - 1023
            })
        }
        ("floorDiv", [JValue::Int(a), JValue::Int(b)]) => java_floor_div(*a, *b).and_then(i),
        ("floorMod", [JValue::Int(a), JValue::Int(b)]) => {
            let quotient = java_floor_div(*a, *b)?;
            i(a.wrapping_sub(quotient.wrapping_mul(*b)))
        }
        ("floorDiv", [JValue::Long(a), JValue::Long(b)]) => {
            java_floor_div_long(*a, *b).map(|q| Some(JValue::Long(q)))
        }
        ("floorMod", [JValue::Long(a), JValue::Long(b)]) => {
            let quotient = java_floor_div_long(*a, *b)?;
            Ok(Some(JValue::Long(
                a.wrapping_sub(quotient.wrapping_mul(*b)),
            )))
        }
        ("addExact", [JValue::Int(a), JValue::Int(b)]) => {
            a.checked_add(*b).map_or_else(|| Err(overflow()), i)
        }
        ("subtractExact", [JValue::Int(a), JValue::Int(b)]) => {
            a.checked_sub(*b).map_or_else(|| Err(overflow()), i)
        }
        ("multiplyExact", [JValue::Int(a), JValue::Int(b)]) => {
            a.checked_mul(*b).map_or_else(|| Err(overflow()), i)
        }
        ("negateExact", [JValue::Int(v)]) => v.checked_neg().map_or_else(|| Err(overflow()), i),
        ("incrementExact", [JValue::Int(v)]) => v.checked_add(1).map_or_else(|| Err(overflow()), i),
        ("decrementExact", [JValue::Int(v)]) => v.checked_sub(1).map_or_else(|| Err(overflow()), i),
        _ => Err(VmError::UnknownIntrinsic(format!("Math.{method}"))),
    }
}

/// Java's `Integer.toString(int, radix)`: lowercase digits, radix
/// clamped to 10 when out of range.
fn int_to_string_radix(value: i32, radix: i32) -> String {
    let radix = if (2..=36).contains(&radix) { radix } else { 10 };
    let radix = u32::try_from(radix).expect("radix in range");
    let mut magnitude = i64::from(value).unsigned_abs();
    let mut digits = Vec::new();
    loop {
        let digit = u32::try_from(magnitude % u64::from(radix)).expect("digit < radix");
        digits.push(char::from_digit(digit, radix).expect("valid digit"));
        magnitude /= u64::from(radix);
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

fn parse_int_text(heap: &Heap, value: &JValue) -> Result<String, VmError> {
    match value {
        JValue::Ref(Some(reference)) => heap
            .string_text(*reference)
            .ok_or_else(|| throw("java.lang.ClassCastException: not a String")),
        // JDK 11's message for a null string is literally "null".
        JValue::Ref(None) => Err(throw("java.lang.NumberFormatException: null")),
        _ => Err(throw("java.lang.VerifyError: expected a String argument")),
    }
}

fn number_format(text: &str) -> VmError {
    throw(format!(
        "java.lang.NumberFormatException: For input string: \"{text}\""
    ))
}

#[allow(clippy::too_many_lines)] // one arm per documented method
#[allow(clippy::many_single_char_names)]
fn integer_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let i = |v: i32| Ok(Some(JValue::Int(v)));
    let s = |heap: &mut Heap, text: String| {
        let reference = heap.alloc_string(&text);
        Ok(Some(JValue::Ref(Some(reference))))
    };
    match (method, args) {
        ("parseInt" | "valueOf", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            text.parse().map_or_else(|_| Err(number_format(&text)), i)
        }
        ("parseInt", [text @ JValue::Ref(_), JValue::Int(radix)]) => {
            let text = parse_int_text(heap, text)?;
            let radix = u32::try_from(*radix).ok().filter(|r| (2..=36).contains(r));
            radix
                .and_then(|r| i32::from_str_radix(&text, r).ok())
                .map_or_else(|| Err(number_format(&text)), i)
        }
        ("parseUnsignedInt", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            // JDK 11 rejects a leading minus with its own message, before the
            // general "For input string" one.
            if text.starts_with('-') {
                return Err(throw(format!(
                    "java.lang.NumberFormatException: Illegal leading minus sign \
                     on unsigned string {text}."
                )));
            }
            text.parse::<u32>()
                .map_or_else(|_| Err(number_format(&text)), |v| i(v.cast_signed()))
        }
        ("toString", [JValue::Int(v)]) => s(heap, v.to_string()),
        ("toString", [JValue::Int(v), JValue::Int(radix)]) => {
            s(heap, int_to_string_radix(*v, *radix))
        }
        ("toBinaryString", [JValue::Int(v)]) => s(heap, format!("{:b}", v.cast_unsigned())),
        ("toOctalString", [JValue::Int(v)]) => s(heap, format!("{:o}", v.cast_unsigned())),
        ("toHexString", [JValue::Int(v)]) => s(heap, format!("{:x}", v.cast_unsigned())),
        ("toUnsignedString", [JValue::Int(v)]) => s(heap, v.cast_unsigned().to_string()),
        ("toUnsignedString", [JValue::Int(v), JValue::Int(radix)]) => {
            let radix = if (2..=36).contains(radix) { *radix } else { 10 };
            let radix = u32::try_from(radix).expect("radix in range");
            let mut magnitude = v.cast_unsigned();
            let mut digits = Vec::new();
            loop {
                digits.push(char::from_digit(magnitude % radix, radix).expect("valid digit"));
                magnitude /= radix;
                if magnitude == 0 {
                    break;
                }
            }
            s(heap, digits.iter().rev().collect())
        }

        ("compare", [JValue::Int(a), JValue::Int(b)]) => i(match a.cmp(b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }),
        ("compareUnsigned", [JValue::Int(a), JValue::Int(b)]) => {
            i(match a.cast_unsigned().cmp(&b.cast_unsigned()) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })
        }
        ("max", [JValue::Int(a), JValue::Int(b)]) => i((*a).max(*b)),
        ("min", [JValue::Int(a), JValue::Int(b)]) => i((*a).min(*b)),
        ("sum", [JValue::Int(a), JValue::Int(b)]) => i(a.wrapping_add(*b)),
        // `Integer.valueOf(int)` answers a REFERENCE, through the autoboxing
        // cache, so `valueOf(100) == valueOf(100)` is true and `valueOf(200)`
        // is not. `hashCode` stays a plain int — they were one arm, which is
        // why valueOf handed back a bare primitive.
        ("valueOf", [JValue::Int(v)]) => {
            let reference = heap.box_wrapper("java/lang/Integer", JValue::Int(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("hashCode", [JValue::Int(v)]) => i(*v),
        ("signum", [JValue::Int(v)]) => i(v.signum()),
        ("bitCount", [JValue::Int(v)]) => i(i32::try_from(v.count_ones()).unwrap_or(0)),
        ("highestOneBit", [JValue::Int(v)]) => i(if *v == 0 {
            0
        } else {
            // Logical shift: the sign bit must not smear downward.
            ((1u32 << 31) >> v.leading_zeros()).cast_signed()
        }),
        ("lowestOneBit", [JValue::Int(v)]) => i(v.wrapping_neg() & v),
        ("numberOfLeadingZeros", [JValue::Int(v)]) => {
            i(i32::try_from(v.leading_zeros()).unwrap_or(32))
        }
        ("numberOfTrailingZeros", [JValue::Int(v)]) => {
            i(i32::try_from(v.trailing_zeros()).unwrap_or(32))
        }
        ("reverse", [JValue::Int(v)]) => i(v.reverse_bits()),
        ("reverseBytes", [JValue::Int(v)]) => i(v.swap_bytes()),
        ("rotateLeft", [JValue::Int(v), JValue::Int(n)]) => {
            i(v.rotate_left(n.cast_unsigned() % 32))
        }
        ("rotateRight", [JValue::Int(v), JValue::Int(n)]) => {
            i(v.rotate_right(n.cast_unsigned() % 32))
        }
        ("divideUnsigned", [JValue::Int(a), JValue::Int(b)]) => {
            if *b == 0 {
                return Err(throw("java.lang.ArithmeticException: / by zero"));
            }
            i((a.cast_unsigned() / b.cast_unsigned()).cast_signed())
        }
        ("remainderUnsigned", [JValue::Int(a), JValue::Int(b)]) => {
            if *b == 0 {
                return Err(throw("java.lang.ArithmeticException: / by zero"));
            }
            i((a.cast_unsigned() % b.cast_unsigned()).cast_signed())
        }
        // The int's 32 bits as an unsigned value in a long (no sign extension).
        ("toUnsignedLong", [JValue::Int(v)]) => {
            Ok(Some(JValue::Long(i64::from(v.cast_unsigned()))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Integer.{method}"))),
    }
}

/// Java's `Double.hashCode`: fold the canonical bit pattern.
pub(crate) fn java_double_hash_public(v: f64) -> i32 {
    java_double_hash(v)
}

fn java_double_hash(v: f64) -> i32 {
    let bits = if v.is_nan() {
        0x7FF8_0000_0000_0000_u64
    } else {
        v.to_bits()
    };
    (((bits ^ (bits >> 32)) & 0xFFFF_FFFF) as u32).cast_signed()
}

/// Java's `Double.toHexString`.
pub(crate) fn java_double_to_hex(v: f64) -> String {
    if v.is_nan() {
        return String::from("NaN");
    }
    if v.is_infinite() {
        return String::from(if v > 0.0 { "Infinity" } else { "-Infinity" });
    }
    let sign = if v.is_sign_negative() { "-" } else { "" };
    let bits = v.to_bits();
    let exponent = (bits >> 52) & 0x7FF;
    let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;
    if exponent == 0 {
        if mantissa == 0 {
            return format!("{sign}0x0.0p0");
        }
        // Subnormal.
        let mut hex = format!("{mantissa:013x}");
        while hex.len() > 1 && hex.ends_with('0') {
            hex.pop();
        }
        return format!("{sign}0x0.{hex}p-1022");
    }
    let mut hex = format!("{mantissa:013x}");
    while hex.len() > 1 && hex.ends_with('0') {
        hex.pop();
    }
    let unbiased = i64::try_from(exponent).unwrap_or(0) - 1023;
    format!("{sign}0x1.{hex}p{unbiased}")
}

/// Strip an optional trailing Java float/double type suffix (`f`/`F`/`d`/`D`),
/// legal on any numeric floating string — `"1.0f"`, `"3.14d"`, `"0x1p4d"`.
fn strip_float_suffix(s: &str) -> &str {
    match s.chars().last() {
        Some('f' | 'F' | 'd' | 'D') if s.len() > 1 => &s[..s.len() - 1],
        _ => s,
    }
}

/// Parse a hexadecimal floating-point string (`0x1.8p1` = 3.0), the form Rust's
/// float parser rejects but Java's `parseDouble`/`parseFloat` grammar accepts.
/// The `p`/`P` binary exponent is mandatory (as in Java).
fn parse_hex_float(s: &str) -> Option<f64> {
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let rest = rest
        .strip_prefix("0x")
        .or_else(|| rest.strip_prefix("0X"))?;
    let (mantissa, exponent) = rest.split_once(['p', 'P'])?;
    let exponent: i32 = exponent.parse().ok()?;
    let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let mut value = 0f64;
    for c in int_part.chars() {
        value = value * 16.0 + f64::from(c.to_digit(16)?);
    }
    let mut scale = 1.0 / 16.0;
    for c in frac_part.chars() {
        value += f64::from(c.to_digit(16)?) * scale;
        scale /= 16.0;
    }
    value *= 2f64.powi(exponent);
    Some(if neg { -value } else { value })
}

/// Parse per Java's `Double.parseDouble` grammar — a trailing type suffix and
/// the hexadecimal form, both of which Rust's `str::parse` rejects.
fn parse_java_double(trimmed: &str) -> Option<f64> {
    let core = strip_float_suffix(trimmed);
    let unsigned = core.trim_start_matches(['+', '-']);
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        return parse_hex_float(core);
    }
    core.parse().ok()
}

/// `Float.parseFloat`'s grammar — the same suffix and hex forms, rounded to a
/// float.
#[allow(clippy::cast_possible_truncation)]
fn parse_java_float(trimmed: &str) -> Option<f32> {
    let core = strip_float_suffix(trimmed);
    let unsigned = core.trim_start_matches(['+', '-']);
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        return parse_hex_float(core).map(|value| value as f32);
    }
    core.parse().ok()
}

#[allow(clippy::float_cmp)]
fn double_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let d = |v: f64| Ok(Some(JValue::Double(v)));
    let z = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    match (method, args) {
        ("parseDouble" | "valueOf", [text @ JValue::Ref(_)]) => {
            // JDK 11 `parseDouble(null)` is a NullPointerException (it reads
            // the null string's length), not a NumberFormatException.
            if matches!(text, JValue::Ref(None)) {
                return Err(throw("java.lang.NullPointerException"));
            }
            let text = parse_int_text(heap, text)?;
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Err(throw("java.lang.NumberFormatException: empty String"));
            }
            parse_java_double(trimmed).map_or_else(|| Err(number_format(&text)), d)
        }
        ("toString", [JValue::Double(v)]) => {
            let reference = heap.alloc_string(&java_double_to_string(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("toHexString", [JValue::Double(v)]) => {
            let reference = heap.alloc_string(&java_double_to_hex(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("valueOf", [JValue::Double(v)]) => d(*v),
        ("isNaN", [JValue::Double(v)]) => z(v.is_nan()),
        ("isInfinite", [JValue::Double(v)]) => z(v.is_infinite()),
        ("isFinite", [JValue::Double(v)]) => z(v.is_finite()),
        ("compare", [JValue::Double(a), JValue::Double(b)]) => {
            // Java: -0.0 < 0.0 and NaN is the largest.
            let ordering = if *a < *b {
                -1
            } else if *a > *b {
                1
            } else {
                let a_bits = if a.is_nan() {
                    0x7FF8_0000_0000_0000_u64
                } else {
                    a.to_bits()
                };
                let b_bits = if b.is_nan() {
                    0x7FF8_0000_0000_0000_u64
                } else {
                    b.to_bits()
                };
                match a_bits.cast_signed().cmp(&b_bits.cast_signed()) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            };
            Ok(Some(JValue::Int(ordering)))
        }
        ("max", [JValue::Double(a), JValue::Double(b)]) => d(java_double_max(*a, *b)),
        ("min", [JValue::Double(a), JValue::Double(b)]) => d(java_double_min(*a, *b)),
        ("sum", [JValue::Double(a), JValue::Double(b)]) => d(a + b),
        ("hashCode", [JValue::Double(v)]) => Ok(Some(JValue::Int(java_double_hash(*v)))),
        ("doubleToLongBits", [JValue::Double(v)]) => {
            let bits = if v.is_nan() {
                0x7FF8_0000_0000_0000_u64
            } else {
                v.to_bits()
            };
            Ok(Some(JValue::Long(bits.cast_signed())))
        }
        ("doubleToRawLongBits", [JValue::Double(v)]) => {
            Ok(Some(JValue::Long(v.to_bits().cast_signed())))
        }
        ("longBitsToDouble", [JValue::Long(v)]) => {
            Ok(Some(JValue::Double(f64::from_bits(v.cast_unsigned()))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Double.{method}"))),
    }
}

/// Java's `Character.isWhitespace` (NOT Unicode `White_Space`: excludes
/// the no-break spaces, includes the ASCII separators).
fn java_is_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0B' | '\x0C' | '\r' | '\x1C'..='\x1F')
        || (java_is_space_char(c) && !matches!(c, '\u{00A0}' | '\u{2007}' | '\u{202F}'))
}

/// Java's `Character.isSpaceChar` (Unicode space separators).
fn java_is_space_char(c: char) -> bool {
    matches!(
        c,
        ' ' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{2028}' | '\u{2029}' | '\u{202F}' | '\u{205F}' | '\u{3000}'
    )
}

/// Start code point of every Unicode decimal-digit (category `Nd`) run in the
/// BMP, one per script. Each run is exactly ten contiguous code points valued
/// 0..=9 (Unicode 10.0, matching JDK 11). Extracted directly from a real JVM.
const ND_STARTS: [u32; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

/// Java's Unicode decimal-digit value (category `Nd`): 0..=9, or `None`.
/// Rust's `char::to_digit` is ASCII-only, so Arabic-Indic '٠', fullwidth '０',
/// Devanagari '५' etc. all need this table.
fn nd_digit_value(c: char) -> Option<u32> {
    let cp = u32::from(c);
    // `then`, not `then_some`: the latter evaluates its argument eagerly, so
    // `cp - s` underflowed for every start above `cp` — a debug-build panic on
    // something as ordinary as `Character.isDigit('A')`. Release builds wrapped
    // instead and the range test still rejected the value, which is how a
    // panic on a common call went unnoticed.
    ND_STARTS
        .iter()
        .find_map(|&s| (cp >= s && cp <= s + 9).then(|| cp - s))
}

/// Java's `Character.toUpperCase(char)`: the *simple* (single-char) uppercase
/// mapping. Rust's `char::to_uppercase` is the *full* mapping, multi-char for
/// ~100 BMP chars. Where the full mapping is one char it equals the simple one;
/// where it is multi-char the simple mapping is either the char itself (ß,
/// ligatures) or, for 27 polytonic-Greek letters with ypogegrammeni, a specific
/// char in the 1F88.. titlecase block.
fn java_simple_upper(c: char) -> char {
    let mut it = c.to_uppercase();
    let first = it.next().unwrap_or(c);
    if it.next().is_none() {
        return first;
    }
    match c {
        '\u{1F80}'..='\u{1F87}' | '\u{1F90}'..='\u{1F97}' | '\u{1FA0}'..='\u{1FA7}' => {
            char::from_u32(u32::from(c) + 8).unwrap_or(c)
        }
        '\u{1FB3}' => '\u{1FBC}',
        '\u{1FC3}' => '\u{1FCC}',
        '\u{1FF3}' => '\u{1FFC}',
        _ => c,
    }
}

#[allow(clippy::too_many_lines)] // one arm per documented method
#[allow(clippy::many_single_char_names)]
fn character_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let z = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    let c_of = |unit: &i32| char::from_u32(u32::try_from(*unit).unwrap_or(0)).unwrap_or('\u{FFFD}');
    let ch_ret = |c: char| {
        Ok(Some(JValue::Int(
            i32::try_from(u32::from(c) & 0xFFFF).unwrap_or(0),
        )))
    };
    match (method, args) {
        ("isDigit", [JValue::Int(v)]) => z(nd_digit_value(c_of(v)).is_some()),

        ("isLetterOrDigit", [JValue::Int(v)]) => {
            let c = c_of(v);
            z(c.is_alphabetic() || nd_digit_value(c).is_some())
        }
        // isAlphabetic is a superset of isLetter in real Java (letter
        // numbers); identical under this approximation.
        ("isLetter" | "isAlphabetic", [JValue::Int(v)]) => z(c_of(v).is_alphabetic()),
        ("isUpperCase", [JValue::Int(v)]) => z(c_of(v).is_uppercase()),
        ("isLowerCase", [JValue::Int(v)]) => z(c_of(v).is_lowercase()),
        ("isWhitespace", [JValue::Int(v)]) => z(java_is_whitespace(c_of(v))),
        ("isSpaceChar", [JValue::Int(v)]) => z(java_is_space_char(c_of(v))),
        ("isJavaIdentifierStart", [JValue::Int(v)]) => {
            let c = c_of(v);
            z(c.is_alphabetic() || c == '_' || c == '$')
        }
        ("isJavaIdentifierPart", [JValue::Int(v)]) => {
            let c = c_of(v);
            z(c.is_alphanumeric() || c == '_' || c == '$')
        }
        ("isDefined", [JValue::Int(v)]) => {
            z(char::from_u32(u32::try_from(*v).unwrap_or(0)).is_some())
        }
        ("isISOControl", [JValue::Int(v)]) => z(matches!(*v, 0..=0x1F | 0x7F..=0x9F)),
        ("isTitleCase", [JValue::Int(v)]) => {
            let c = c_of(v);
            let upper = c.to_uppercase().next().unwrap_or(c);
            let lower = c.to_lowercase().next().unwrap_or(c);
            z(upper != c && lower != c)
        }
        // toTitleCase reuses the simple-uppercase mapping; the ~30 chars whose
        // titlecase differs from their uppercase remain approximated.
        ("toTitleCase" | "toUpperCase", [JValue::Int(v)]) => ch_ret(java_simple_upper(c_of(v))),
        ("toLowerCase", [JValue::Int(v)]) => {
            // The first char of Rust's full lowercase equals Java's simple
            // lowercase across the whole BMP (verified against a real JVM).
            let c = c_of(v);
            ch_ret(c.to_lowercase().next().unwrap_or(c))
        }
        ("getNumericValue", [JValue::Int(v)]) => {
            let c = c_of(v);
            // Nd decimal value (0..=9), else a Latin letter's 10..=35 value.
            // The letter-number/other-number specials (Roman numerals -> value,
            // fractions -> -2, superscripts) remain approximated as -1.
            let value = nd_digit_value(c)
                .or_else(|| c.to_digit(36))
                .and_then(|d| i32::try_from(d).ok())
                .unwrap_or(-1);
            Ok(Some(JValue::Int(value)))
        }
        ("digit", [JValue::Int(v), JValue::Int(radix)]) => {
            let c = c_of(v);
            let value = u32::try_from(*radix)
                .ok()
                .filter(|r| (2..=36).contains(r))
                .and_then(|r| {
                    nd_digit_value(c)
                        .filter(|&d| d < r)
                        .or_else(|| c.to_digit(r))
                })
                .and_then(|d| i32::try_from(d).ok())
                .unwrap_or(-1);
            Ok(Some(JValue::Int(value)))
        }
        ("forDigit", [JValue::Int(digit), JValue::Int(radix)]) => {
            let c = u32::try_from(*radix)
                .ok()
                .filter(|r| (2..=36).contains(r))
                .zip(u32::try_from(*digit).ok())
                .filter(|(r, d)| d < r)
                .and_then(|(r, d)| char::from_digit(d, r))
                .unwrap_or('\0');
            ch_ret(c)
        }
        ("compare", [JValue::Int(a), JValue::Int(b)]) => Ok(Some(JValue::Int(a - b))),

        ("toString", [JValue::Int(v)]) => {
            let reference = heap.alloc_string(&c_of(v).to_string());
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("valueOf" | "hashCode", [JValue::Int(v)]) => Ok(Some(JValue::Int(*v))),
        ("isHighSurrogate", [JValue::Int(v)]) => z((0xD800..0xDC00).contains(v)),
        ("isLowSurrogate", [JValue::Int(v)]) => z((0xDC00..0xE000).contains(v)),
        ("isSurrogate", [JValue::Int(v)]) => z((0xD800..0xE000).contains(v)),
        ("charCount", [JValue::Int(v)]) => Ok(Some(JValue::Int(if *v >= 0x10000 { 2 } else { 1 }))),
        _ => Err(VmError::UnknownIntrinsic(format!("Character.{method}"))),
    }
}

fn boolean_static(
    heap: &mut Heap,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let z = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    match (method, args) {
        ("parseBoolean", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            z(text.eq_ignore_ascii_case("true"))
        }
        ("toString", [JValue::Int(v)]) => {
            let reference = heap.alloc_string(if *v != 0 { "true" } else { "false" });
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("valueOf", [JValue::Int(v)]) => z(*v != 0),
        ("compare", [JValue::Int(a), JValue::Int(b)]) => {
            Ok(Some(JValue::Int(if (*a != 0) == (*b != 0) {
                0
            } else if *a != 0 {
                1
            } else {
                -1
            })))
        }
        // Java's fixed Boolean hash codes.
        ("hashCode", [JValue::Int(v)]) => Ok(Some(JValue::Int(if *v != 0 { 1231 } else { 1237 }))),
        ("logicalAnd", [JValue::Int(a), JValue::Int(b)]) => z(*a != 0 && *b != 0),
        ("logicalOr", [JValue::Int(a), JValue::Int(b)]) => z(*a != 0 || *b != 0),
        ("logicalXor", [JValue::Int(a), JValue::Int(b)]) => z((*a != 0) != (*b != 0)),
        _ => Err(VmError::UnknownIntrinsic(format!("Boolean.{method}"))),
    }
}

#[allow(clippy::too_many_lines, clippy::many_single_char_names)] // one arm per documented method
fn long_static(heap: &mut Heap, method: &str, args: &[JValue]) -> Result<Option<JValue>, VmError> {
    let l = |v: i64| Ok(Some(JValue::Long(v)));
    let i = |v: i32| Ok(Some(JValue::Int(v)));
    let s = |heap: &mut Heap, text: String| {
        let reference = heap.alloc_string(&text);
        Ok(Some(JValue::Ref(Some(reference))))
    };
    match (method, args) {
        ("parseLong" | "valueOf", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            text.parse().map_or_else(|_| Err(number_format(&text)), l)
        }
        ("valueOf", [JValue::Long(v)]) => l(*v),
        ("toString", [JValue::Long(v)]) => s(heap, v.to_string()),
        ("toBinaryString", [JValue::Long(v)]) => s(heap, format!("{:b}", v.cast_unsigned())),
        ("toOctalString", [JValue::Long(v)]) => s(heap, format!("{:o}", v.cast_unsigned())),
        ("toHexString", [JValue::Long(v)]) => s(heap, format!("{:x}", v.cast_unsigned())),
        ("compare", [JValue::Long(a), JValue::Long(b)]) => i(match a.cmp(b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }),
        ("max", [JValue::Long(a), JValue::Long(b)]) => l((*a).max(*b)),
        ("min", [JValue::Long(a), JValue::Long(b)]) => l((*a).min(*b)),
        ("sum", [JValue::Long(a), JValue::Long(b)]) => l(a.wrapping_add(*b)),
        ("signum", [JValue::Long(v)]) => i(i32::try_from(v.signum()).unwrap_or(0)),
        ("hashCode", [JValue::Long(v)]) => i((((v.cast_unsigned() ^ (v.cast_unsigned() >> 32))
            & 0xFFFF_FFFF) as u32)
            .cast_signed()),
        ("bitCount", [JValue::Long(v)]) => i(i32::try_from(v.count_ones()).unwrap_or(0)),
        ("numberOfLeadingZeros", [JValue::Long(v)]) => {
            i(i32::try_from(v.leading_zeros()).unwrap_or(64))
        }
        ("numberOfTrailingZeros", [JValue::Long(v)]) => {
            i(i32::try_from(v.trailing_zeros()).unwrap_or(64))
        }
        ("reverse", [JValue::Long(v)]) => l(v.reverse_bits()),
        ("reverseBytes", [JValue::Long(v)]) => l(v.swap_bytes()),
        ("highestOneBit", [JValue::Long(v)]) => l(if *v == 0 {
            0
        } else {
            1i64.wrapping_shl(v.cast_unsigned().ilog2())
        }),
        ("lowestOneBit", [JValue::Long(v)]) => l(v & v.wrapping_neg()),
        // Rotation distance is used modulo 64 (JLS / the JDK).
        ("rotateLeft", [JValue::Long(v), JValue::Int(d)]) => {
            l(v.rotate_left((d & 63).cast_unsigned()))
        }
        ("rotateRight", [JValue::Long(v), JValue::Int(d)]) => {
            l(v.rotate_right((d & 63).cast_unsigned()))
        }
        ("divideUnsigned", [JValue::Long(a), JValue::Long(b)]) => {
            if *b == 0 {
                return Err(throw("java.lang.ArithmeticException: / by zero"));
            }
            l((a.cast_unsigned() / b.cast_unsigned()).cast_signed())
        }
        ("remainderUnsigned", [JValue::Long(a), JValue::Long(b)]) => {
            if *b == 0 {
                return Err(throw("java.lang.ArithmeticException: / by zero"));
            }
            l((a.cast_unsigned() % b.cast_unsigned()).cast_signed())
        }
        ("compareUnsigned", [JValue::Long(a), JValue::Long(b)]) => {
            i(match a.cast_unsigned().cmp(&b.cast_unsigned()) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })
        }
        ("toUnsignedString", [JValue::Long(v)]) => s(heap, v.cast_unsigned().to_string()),
        ("toUnsignedString", [JValue::Long(v), JValue::Int(radix)]) => {
            let radix = if (2..=36).contains(radix) { *radix } else { 10 };
            let radix = u64::try_from(radix).expect("radix in range");
            let mut magnitude = v.cast_unsigned();
            let mut digits = Vec::new();
            loop {
                let digit = u32::try_from(magnitude % radix).expect("digit < radix");
                let radix32 = u32::try_from(radix).expect("radix in range");
                digits.push(char::from_digit(digit, radix32).expect("valid digit"));
                magnitude /= radix;
                if magnitude == 0 {
                    break;
                }
            }
            s(heap, digits.iter().rev().collect())
        }
        _ => Err(VmError::UnknownIntrinsic(format!("Long.{method}"))),
    }
}

/// `Short` and `Byte` wrapper statics — identical shapes, differing
/// only in range.
fn small_int_static(
    heap: &mut Heap,
    class: &str,
    method: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let (lo, hi) = if class == "Short" {
        (i32::from(i16::MIN), i32::from(i16::MAX))
    } else {
        (i32::from(i8::MIN), i32::from(i8::MAX))
    };
    match (method, args) {
        ("parseShort" | "parseByte" | "valueOf", [text @ JValue::Ref(_)]) => {
            let text = parse_int_text(heap, text)?;
            let value: i32 = text.parse().map_err(|_| number_format(&text))?;
            if value < lo || value > hi {
                return Err(number_format_range(&text));
            }
            Ok(Some(JValue::Int(value)))
        }
        ("valueOf" | "hashCode", [JValue::Int(v)]) => Ok(Some(JValue::Int(*v))),
        ("toString", [JValue::Int(v)]) => {
            let reference = heap.alloc_string(&v.to_string());
            Ok(Some(JValue::Ref(Some(reference))))
        }
        // `Short.compare`/`Byte.compare` return `x - y` (the JDK source), NOT
        // the -1/0/1 sign that `Integer.compare` gives — the values are small
        // enough that the subtraction cannot overflow.
        ("compare", [JValue::Int(a), JValue::Int(b)]) => Ok(Some(JValue::Int(a - b))),
        ("reverseBytes", [JValue::Int(v)]) => {
            #[allow(clippy::cast_possible_truncation)]
            let value = *v as i16;
            Ok(Some(JValue::Int(i32::from(value.swap_bytes()))))
        }
        _ => Err(VmError::UnknownIntrinsic(format!("{class}.{method}"))),
    }
}

/// javac range message for `Byte.parseByte("200")`.
fn number_format_range(text: &str) -> VmError {
    // `Short.parseShort`/`Byte.parseByte` range-check the result of
    // `Integer.parseInt(s, radix)` and throw exactly this — no class suffix.
    VmError::UncaughtException(format!(
        "java.lang.NumberFormatException: Value out of range. Value:\"{text}\" Radix:10"
    ))
}

fn float_static(heap: &mut Heap, method: &str, args: &[JValue]) -> Result<Option<JValue>, VmError> {
    let f = |v: f32| Ok(Some(JValue::Float(v)));
    let b = |v: bool| Ok(Some(JValue::Int(i32::from(v))));
    match (method, args) {
        ("parseFloat" | "valueOf", [text @ JValue::Ref(_)]) => {
            // Like Double: null NPEs, an empty/blank string is "empty String".
            if matches!(text, JValue::Ref(None)) {
                return Err(throw("java.lang.NullPointerException"));
            }
            let text = parse_int_text(heap, text)?;
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Err(throw("java.lang.NumberFormatException: empty String"));
            }
            parse_java_float(trimmed).map_or_else(|| Err(number_format(&text)), f)
        }
        ("valueOf", [JValue::Float(v)]) => f(*v),
        ("toString", [JValue::Float(v)]) => {
            let reference = heap.alloc_string(&java_float_to_string(*v));
            Ok(Some(JValue::Ref(Some(reference))))
        }
        ("isNaN", [JValue::Float(v)]) => b(v.is_nan()),
        ("isInfinite", [JValue::Float(v)]) => b(v.is_infinite()),
        ("isFinite", [JValue::Float(v)]) => b(v.is_finite()),
        ("compare", [JValue::Float(a), JValue::Float(b)]) => {
            // Java total order: -0.0 < 0.0, NaN greatest.
            Ok(Some(JValue::Int(match a.total_cmp(b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            })))
        }
        ("max", [JValue::Float(a), JValue::Float(b)]) => f(java_float_max(*a, *b)),
        ("min", [JValue::Float(a), JValue::Float(b)]) => f(java_float_min(*a, *b)),
        ("sum", [JValue::Float(a), JValue::Float(b)]) => f(a + b),
        ("hashCode" | "floatToIntBits", [JValue::Float(v)]) => {
            let bits = if v.is_nan() {
                0x7FC0_0000_u32
            } else {
                v.to_bits()
            };
            Ok(Some(JValue::Int(bits.cast_signed())))
        }
        ("floatToRawIntBits", [JValue::Float(v)]) => {
            Ok(Some(JValue::Int(v.to_bits().cast_signed())))
        }
        ("intBitsToFloat", [JValue::Int(v)]) => f(f32::from_bits(v.cast_unsigned())),
        _ => Err(VmError::UnknownIntrinsic(format!("Float.{method}"))),
    }
}

/// Java `Math.max(float)`: NaN wins; +0.0 beats -0.0.
fn java_float_max(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return f32::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_positive() || b.is_sign_positive() {
            0.0
        } else {
            -0.0
        };
    }
    if a > b { a } else { b }
}

fn java_float_min(a: f32, b: f32) -> f32 {
    if a.is_nan() || b.is_nan() {
        return f32::NAN;
    }
    if a == 0.0 && b == 0.0 {
        return if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        };
    }
    if a < b { a } else { b }
}

fn string_static(
    heap: &mut Heap,
    method: &str,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    if method == "format" {
        let template = match args.first() {
            Some(JValue::Ref(Some(reference))) => heap.string_text(*reference).unwrap_or_default(),
            Some(JValue::Ref(None)) => {
                return Err(throw("java.lang.NullPointerException: format is null"));
            }
            _ => String::new(),
        };
        let format_args = crate::format::args_from_descriptor(descriptor, &args[1..])?;
        let text = crate::format::java_format(heap, &template, &format_args)?;
        let reference = heap.alloc_string(&text);
        return Ok(Some(JValue::Ref(Some(reference))));
    }
    if method == "join" {
        return string_join(heap, descriptor, args);
    }
    match (method, args) {
        ("valueOf" | "copyValueOf", [value]) => {
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
                (_, JValue::Long(v)) => v.to_string(),
                ("(F)Ljava/lang/String;", JValue::Float(v)) => java_float_to_string(*v),
                (_, JValue::Double(v)) => java_double_to_string(*v),
                (_, JValue::Ref(Some(reference))) => match heap.get(*reference) {
                    Some(HeapObject::IntArray(_, values)) => values
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
        _ => Err(VmError::UnknownIntrinsic(format!("String.{method}"))),
    }
}

/// `String.join(delimiter, ...)`. The codegen descriptor says how argument 1 is
/// shaped: a `String[]`, a `List`, or the first of individual `CharSequence`
/// arguments. A null delimiter or a null array/iterable throws; a null element
/// joins as the text `null`, as `StringJoiner` produces.
fn string_join(
    heap: &mut Heap,
    descriptor: &str,
    args: &[JValue],
) -> Result<Option<JValue>, VmError> {
    let delimiter = match args.first() {
        Some(JValue::Ref(Some(reference))) => heap.string_text(*reference).unwrap_or_default(),
        Some(JValue::Ref(None)) => {
            return Err(throw("java.lang.NullPointerException: delimiter is null"));
        }
        _ => String::new(),
    };
    let shape = descriptor
        .strip_prefix("(Ljava/lang/String;")
        .and_then(|rest| rest.strip_suffix(")Ljava/lang/String;"))
        .unwrap_or("");
    let elements: Vec<JValue> = if shape.starts_with('[') {
        match args.get(1) {
            Some(JValue::Ref(Some(reference))) => match heap.get(*reference) {
                Some(HeapObject::RefArray(_, values)) => values.clone(),
                _ => return Err(throw("java.lang.NullPointerException")),
            },
            _ => return Err(throw("java.lang.NullPointerException: elements are null")),
        }
    } else if shape.starts_with("Ljava/util/List;") {
        match args.get(1) {
            Some(JValue::Ref(Some(reference))) => heap
                .list_values(*reference)
                .cloned()
                .ok_or_else(|| throw("java.lang.NullPointerException"))?,
            _ => return Err(throw("java.lang.NullPointerException: elements are null")),
        }
    } else {
        args.get(1..).unwrap_or(&[]).to_vec()
    };
    let parts: Vec<String> = elements
        .iter()
        .map(|value| match value {
            JValue::Ref(Some(reference)) => heap
                .string_text(*reference)
                .unwrap_or_else(|| String::from("null")),
            _ => String::from("null"),
        })
        .collect();
    let reference = heap.alloc_string(&parts.join(&delimiter));
    Ok(Some(JValue::Ref(Some(reference))))
}

/// Render a `StringBuilder.append` argument the way Java would.
/// Render an `Object`-typed value inline (`String.valueOf` semantics) for the
/// reflection handles that flow through `append(Object)`/`print(Object)`.
/// An object's identity hash: its heap reference, as a real JVM uses its
/// address. `Object.toString()` prints this same value in hex, so the two
/// agree exactly as they do on a JVM.
pub(crate) fn identity_hash(reference: HeapRef) -> i32 {
    i32::from_ne_bytes(reference.to_ne_bytes())
}

/// The JVM class descriptor of an array on the heap: `[I`, `[[I`,
/// `[Ljava/lang/String;`. A primitive array carries it in its variant; a
/// reference array remembers its own, because once the static type is gone
/// the heap is the only thing that still knows the element type.
pub(crate) fn array_class_name(heap: &Heap, reference: HeapRef) -> Option<String> {
    Some(match heap.get(reference)? {
        HeapObject::IntArray(IntKind::Int, _) => String::from("[I"),
        HeapObject::IntArray(IntKind::Boolean, _) => String::from("[Z"),
        HeapObject::IntArray(IntKind::Char, _) => String::from("[C"),
        HeapObject::DoubleArray(_) => String::from("[D"),
        HeapObject::LongArray(_) => String::from("[J"),
        HeapObject::FloatArray(_) => String::from("[F"),
        HeapObject::ShortArray(_) => String::from("[S"),
        HeapObject::ByteArray(_) => String::from("[B"),
        HeapObject::RefArray(class, _) => class.clone(),
        _ => return None,
    })
}

/// `Object.toString()` for an array, exactly as the JDK writes it:
/// `getClass().getName() + "@" + Integer.toHexString(hashCode())`, where
/// `getName()` spells a reference element with dots. An `int[]` is
/// `[I@1b6d3586`; a `String[]` is `[Ljava.lang.String;@4554617c`. Useless
/// to read, and precisely what a student sees on a real JVM.
pub(crate) fn array_to_string(heap: &Heap, reference: HeapRef) -> Option<String> {
    let name = array_class_name(heap, reference)?.replace('/', ".");
    Some(format!("{name}@{:x}", identity_hash(reference)))
}

pub(crate) fn object_display(heap: &Heap, value: JValue) -> String {
    match value {
        JValue::Ref(None) => String::from("null"),
        JValue::Ref(Some(reference)) => match heap.get(reference) {
            // A StringBuilder renders as its content (String.valueOf/append(Object)
            // call toString), same as a String.
            Some(HeapObject::JavaString(units) | HeapObject::StringBuilder(units)) => {
                String::from_utf16_lossy(units)
            }
            Some(HeapObject::Class { name }) => format!("class {name}"),
            // An array does not override `toString`, so it gets Object's:
            // `[I@1b6d3586`. Not `Arrays.toString`'s `[1, 2, 3]`.
            Some(_) if array_class_name(heap, reference).is_some() => {
                array_to_string(heap, reference).unwrap_or_default()
            }
            Some(HeapObject::Boxed { class_name, value }) => boxed_to_string(class_name, *value),
            Some(HeapObject::Field {
                declaring,
                name,
                descriptor,
                access,
                ..
            }) => field_to_string(declaring, name, descriptor, *access),
            Some(HeapObject::Constructor {
                declaring,
                descriptor,
                access,
            }) => constructor_to_string(declaring, descriptor, *access),
            Some(HeapObject::Method {
                declaring,
                name,
                descriptor,
                ..
            }) => format!("{declaring}.{name}{descriptor}"),
            Some(HeapObject::ReflectType { raw, args }) => {
                let dotted = raw.replace('/', ".");
                if args.is_empty() {
                    dotted
                } else {
                    format!("{dotted}<{}>", args.join(", "))
                }
            }
            Some(HeapObject::Instance { class_name, .. }) => format!("{class_name}@{reference:x}"),
            _ => format!("object@{reference:x}"),
        },
        other => format!("{other:?}"),
    }
}

fn builder_value_text(heap: &Heap, param: &str, value: &JValue) -> Result<Vec<u16>, VmError> {
    let text = match (param, value) {
        ("I", JValue::Int(v)) => v.to_string(),
        ("J", JValue::Long(v)) => v.to_string(),
        ("F", JValue::Float(v)) => java_float_to_string(*v),
        ("Z", JValue::Int(v)) => if *v != 0 { "true" } else { "false" }.to_owned(),
        ("D", JValue::Double(v)) => java_double_to_string(*v),
        // A `char` is one UTF-16 unit, even an unpaired surrogate — which
        // `char::from_u32` would reject. Keep the raw unit.
        ("C", JValue::Int(v)) => return Ok(vec![u16::try_from(*v).unwrap_or(u16::MAX)]),
        ("[C", value) => return char_array_units(heap, value),
        ("Ljava/lang/String;", JValue::Ref(reference)) => match reference {
            None => String::from("null"),
            Some(reference) => heap.string_text(*reference).ok_or_else(|| {
                VmError::UnknownIntrinsic(String::from("argument is not a string object"))
            })?,
        },
        // `insert(int, Object)` — `String.valueOf(Object)` semantics.
        // `append(Object)` never reaches here: the interpreter renders its
        // argument first, since doing so may call a user `toString()`.
        ("Ljava/lang/Object;", value) => object_display(heap, *value),
        _ => {
            return Err(VmError::UnknownIntrinsic(format!(
                "StringBuilder overload with parameter {param}"
            )));
        }
    };
    Ok(text.encode_utf16().collect())
}

/// The UTF-16 units of a `char[]` argument (`char` values live in an
/// `IntArray`, so an unpaired surrogate survives the round trip).
fn char_array_units(heap: &Heap, value: &JValue) -> Result<Vec<u16>, VmError> {
    match value {
        JValue::Ref(Some(reference)) => match heap.get(*reference) {
            Some(HeapObject::IntArray(_, values)) => Ok(values
                .iter()
                .map(|v| u16::try_from(*v).unwrap_or(u16::MAX))
                .collect()),
            _ => Err(throw("java.lang.ClassCastException: not a char[]")),
        },
        JValue::Ref(None) => Err(throw("java.lang.NullPointerException")),
        _ => Err(throw("java.lang.VerifyError: expected a char[] argument")),
    }
}

/// The parameter descriptors of a method descriptor, i.e. what sits
/// between the parentheses (`"(ILjava/lang/String;)V"` → `"ILjava/lang/String;"`).
fn descriptor_params(descriptor: &str) -> &str {
    descriptor
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map_or("", |(params, _)| params)
}

/// Render a print/println argument the way Java would.
fn print_argument_text(heap: &Heap, descriptor: &str, args: &[JValue]) -> Result<String, VmError> {
    let text = match (descriptor, args) {
        ("()V", []) => String::new(),
        ("(I)V", [JValue::Int(v)]) => v.to_string(),
        ("(J)V", [JValue::Long(v)]) => v.to_string(),
        ("(F)V", [JValue::Float(v)]) => java_float_to_string(*v),
        ("(Z)V", [JValue::Int(v)]) => if *v != 0 { "true" } else { "false" }.to_owned(),
        ("(C)V", [JValue::Int(v)]) => {
            let unit = u32::try_from(*v).unwrap_or(u32::from(u16::MAX));
            char::from_u32(unit).map_or_else(|| String::from('\u{FFFD}'), String::from)
        }
        ("(D)V", [JValue::Double(v)]) => java_double_to_string(*v),
        // `println(char[])` is a real overload: it prints the CHARACTERS.
        // (`"" + chars` does not — that is `append(Object)` and prints
        // `[C@hash`. A classic Java trap, faithfully reproduced.)
        ("([C)V", [value]) => String::from_utf16_lossy(&char_array_units(heap, value)?),
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
    // Java's FloatingDecimal renders the smallest subnormal as
    // 4.9E-324; shortest-round-trip formatting would say 5.0E-324.
    if value.to_bits() == 1 {
        return String::from("4.9E-324");
    }
    if value.to_bits() == 0x8000_0000_0000_0001 {
        return String::from("-4.9E-324");
    }
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

/// `Float.toString`, matching `OpenJDK` 11's `FloatingDecimal`.
pub(crate) fn java_float_to_string(value: f32) -> String {
    crate::floatdec::java_float_to_string(value)
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
