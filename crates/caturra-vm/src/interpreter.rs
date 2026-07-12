//! The bytecode dispatch loop (JVMS §2.5–2.6, §6.5).
//!
//! A classic frame interpreter: locals + operand stack per method,
//! decoding one opcode at a time. The opcode surface grows in lockstep
//! with the compiler (`specs/LANGUAGE.md`); an opcode the VM doesn't
//! know yet is a `VmError`, never undefined behavior.

use std::collections::HashMap;
use std::rc::Rc;

use caturra_classfile::{
    CODE_ATTRIBUTE, ClassFile, CodeAttribute, Constant, MethodInfo, opcodes as op,
    read_code_attribute,
};

use crate::debug::{
    Breakpoint, DebugCommand, DebugFrameSnapshot, DebugHost, DebugSnapshot, PauseReason,
    WatchEvaluator,
};
use crate::intrinsics::{self, IntrinsicStatics};
use crate::io::ConsoleIo;
use crate::value::{Heap, HeapRef, JValue, MapViewKind};
use crate::vfs::VirtualFileSystem;
use crate::vm::VmError;

/// State for one `run`: the loaded classes, heap, interned strings,
/// intrinsic singletons, and the instruction/call-depth budgets.
pub(crate) struct Interpreter<'run> {
    pub classes: &'run HashMap<String, ClassFile>,
    pub console: &'run mut dyn ConsoleIo,
    pub vfs: &'run mut VirtualFileSystem,
    pub heap: Heap,
    pub intrinsic_statics: IntrinsicStatics,
    string_pool: HashMap<String, HeapRef>,
    rng: intrinsics::JavaRng,
    /// Static field values per user class, created on first use.
    statics: HashMap<String, HashMap<String, JValue>>,
    /// Resolved instance-field keys, memoized per call site — the class holding
    /// the code plus the constant-pool index of its field ref. Field access is
    /// the hottest operation in object-heavy programs (a 400x400 image filter
    /// does millions), and resolving a key from scratch allocates several
    /// Strings and walks the superclass chain every time. `classes` is immutable
    /// for the run, so a `ClassFile`'s address is a stable id.
    field_keys: HashMap<(usize, u16), Rc<str>>,
    /// Defaulted instance-field map per class, cloned on each `new`.
    field_templates: HashMap<String, HashMap<Rc<str>, JValue>>,
    /// Classes whose initialization has started (JVMS §5.5: recursive
    /// initialization by the same thread proceeds).
    init_started: std::collections::HashSet<String>,
    remaining_instructions: u64,
    max_call_depth: u32,
    /// Frames suspended beneath a nested run (see
    /// [`Interpreter::run_nested`]), innermost run last: the caller's
    /// location, then the frames below it. Kept here rather than on the host
    /// stack so that call depth and stack traces still see them.
    suspended_runs: Vec<(Option<CurrentLocation>, Vec<Frame<'run>>)>,
    /// The number of frames in `suspended_runs`, plus one native caller per
    /// nested run. Call depth counts them: a `toString()` reached from a
    /// container's must still see the recursion it is already inside.
    nested_frame_base: usize,
    /// Suspended caller frames; the active frame lives as a local in
    /// [`Self::run_loop`] and is swapped in and out on call/return.
    /// Keeping frames as plain data (rather than host-stack recursion)
    /// decouples Java call depth from the host stack and makes frames
    /// inspectable — the foundation for step debugging.
    frames: Vec<Frame<'run>>,
    /// The active frame's (class, method, file, line) for stack traces;
    /// class/method/file update on frame switches, line at each
    /// line-table boundary.
    current_location: Option<CurrentLocation>,
    /// Parsed `Code` attributes with their debug tables, keyed by
    /// `MethodInfo` address (stable for the run — `classes` is borrowed
    /// for `'run`). Parsing once per method instead of once per call.
    code_cache: HashMap<usize, Rc<MethodCode>>,
    /// Debugger state, present only for debug runs.
    debug: Option<DebugState<'run>>,
    /// Arena keeping compiled watch classes alive for the run (their
    /// frames borrow them at `'run`), set for debug runs.
    watch_arena: Option<&'run typed_arena::Arena<ClassFile>>,
    /// The most recently `athrow`n object; the unwinder binds this to
    /// the catch variable so a user exception keeps its identity and
    /// fields (library throws allocate fresh objects instead).
    last_thrown: Option<HeapRef>,
}

/// Where the active frame is, for stack traces and snapshots.
struct CurrentLocation {
    class_name: String,
    method_name: String,
    source_file: String,
    line: Option<u16>,
}

/// Live debugger state for one run.
struct DebugState<'run> {
    host: &'run mut dyn DebugHost,
    breakpoints: std::collections::HashSet<(String, u32)>,
    /// Active step goal from the last pause.
    step: Option<StepGoal>,
    /// The source line we just resumed from; suppress re-pausing anywhere
    /// on it until execution leaves it. A single line can compile to
    /// several boundary PCs, so skipping only the paused address would
    /// re-fire a breakpoint (or step) at the line's next boundary.
    resume_line: Option<(String, u16)>,
    /// Countdown to the next interrupt poll.
    poll_in: u32,
}

/// What "one step" means, decided when the step command was issued.
struct StepGoal {
    mode: DebugCommand,
    /// Suspended-frame count when the step began.
    depth: usize,
}

const INTERRUPT_POLL_INTERVAL: u32 = 4096;

impl<'run> Interpreter<'run> {
    pub fn new(
        classes: &'run HashMap<String, ClassFile>,
        console: &'run mut dyn ConsoleIo,
        vfs: &'run mut VirtualFileSystem,
        max_instructions: u64,
        max_call_depth: u32,
        random_seed: Option<u64>,
    ) -> Self {
        Self {
            classes,
            console,
            vfs,
            heap: Heap::new(),
            intrinsic_statics: IntrinsicStatics::default(),
            string_pool: HashMap::new(),
            rng: intrinsics::JavaRng::new(random_seed),
            statics: HashMap::new(),
            field_keys: HashMap::new(),
            field_templates: HashMap::new(),
            init_started: std::collections::HashSet::new(),
            remaining_instructions: max_instructions,
            max_call_depth,
            suspended_runs: Vec::new(),
            nested_frame_base: 0,
            frames: Vec::new(),
            current_location: None,
            code_cache: HashMap::new(),
            debug: None,
            watch_arena: None,
            last_thrown: None,
        }
    }

    /// Attach a debugger for this run.
    pub fn attach_debugger(
        &mut self,
        host: &'run mut dyn DebugHost,
        breakpoints: &[Breakpoint],
        watch_arena: &'run typed_arena::Arena<ClassFile>,
    ) {
        self.watch_arena = Some(watch_arena);
        self.debug = Some(DebugState {
            host,
            breakpoints: breakpoints
                .iter()
                .map(|b| (b.file.clone(), b.line))
                .collect(),
            step: None,
            resume_line: None,
            poll_in: INTERRUPT_POLL_INTERVAL,
        });
    }

    /// Allocate (or reuse) the interned string for `text`, per JLS
    /// string-literal semantics.
    pub fn intern_string(&mut self, text: &str) -> HeapRef {
        if let Some(existing) = self.string_pool.get(text) {
            return *existing;
        }
        let reference = self.heap.alloc_string(text);
        self.string_pool.insert(text.to_owned(), reference);
        reference
    }

    /// Execute one method to completion, returning its result value
    /// (`None` for void). The Java call stack is an explicit
    /// [`Vec<Frame>`]; the host stack stays O(1) regardless of Java
    /// call depth. `max_call_depth` frames → `StackOverflowError`.
    pub fn execute(
        &mut self,
        class: &'run ClassFile,
        method: &'run MethodInfo,
        locals: Vec<JValue>,
    ) -> Result<Option<JValue>, VmError> {
        debug_assert!(self.frames.is_empty(), "execute is not re-entrant");
        let frame = self.make_frame(class, method, locals)?;
        let result = self.run_loop(frame);
        if let Err(VmError::UncaughtException(message)) = result {
            let message = self.attach_stack_trace(message);
            self.frames.clear();
            return Err(VmError::UncaughtException(message));
        }
        self.frames.clear();
        result
    }

    /// Append a Java-style stack trace (`\tat Class.method(Class.java)`
    /// lines, innermost first) to an uncaught exception's message.
    fn attach_stack_trace(&self, message: String) -> String {
        use std::fmt::Write as _;
        if message.contains("\n\tat ") {
            return message;
        }
        let mut full = message;
        let location = |file: &str, line: Option<u16>| match line {
            Some(line) => format!("{file}:{line}"),
            None => file.to_owned(),
        };
        if let Some(current) = &self.current_location {
            let _ = write!(
                full,
                "\n\tat {}.{}({})",
                current.class_name,
                current.method_name,
                location(&current.source_file, current.line)
            );
        }
        let trace_frames = |frames: &[Frame<'run>], full: &mut String| {
            for suspended in frames.iter().rev() {
                let class_name = suspended.class.class_name().unwrap_or("<unknown>");
                let _ = write!(
                    full,
                    "\n\tat {class_name}.{}({})",
                    suspended.method_name,
                    location(&suspended.code.source_file, suspended.current_line)
                );
            }
        };
        trace_frames(&self.frames, &mut full);
        // Then the callers below each nested run (a `toString()` invoked while
        // rendering a container), innermost run first.
        for (caller, frames) in self.suspended_runs.iter().rev() {
            if let Some(caller) = caller {
                let _ = write!(
                    full,
                    "\n\tat {}.{}({})",
                    caller.class_name,
                    caller.method_name,
                    location(&caller.source_file, caller.line)
                );
            }
            trace_frames(frames, &mut full);
        }
        full
    }

    /// The per-instruction debugger hook: decides whether to pause at
    /// `addr` (line boundary hit, step goal met, or host interrupt) and
    /// blocks in `on_pause` if so. Returns the resume command.
    fn debug_checkpoint(
        &mut self,
        frame: &mut Frame<'run>,
        boundary_line: u16,
    ) -> Option<DebugCommand> {
        let depth = self.call_depth();
        let debug = self.debug.as_mut().expect("caller checked debug mode");
        let at_line_boundary = boundary_line != 0;

        // Just resumed: suppress re-pausing anywhere on the line we paused
        // at until execution moves off it. One source line can compile to
        // several boundary PCs, so skipping only the paused address would
        // re-fire a breakpoint (or step) at the line's next boundary.
        if at_line_boundary {
            match &debug.resume_line {
                Some((file, line)) if file == &frame.code.source_file && *line == boundary_line => {
                    return None;
                }
                _ => debug.resume_line = None,
            }
        }

        // Host interrupt (pause / stop button), polled sparsely.
        debug.poll_in = debug.poll_in.saturating_sub(1);
        let interrupted = if debug.poll_in == 0 {
            debug.poll_in = INTERRUPT_POLL_INTERVAL;
            debug.host.interrupt_requested()
        } else {
            false
        };

        let reason = if interrupted {
            Some(PauseReason::Interrupt)
        } else if at_line_boundary
            && debug
                .breakpoints
                .contains(&(frame.code.source_file.clone(), u32::from(boundary_line)))
        {
            Some(PauseReason::Breakpoint)
        } else if at_line_boundary
            && debug.step.as_ref().is_some_and(|goal| match goal.mode {
                DebugCommand::StepInto => true,
                DebugCommand::StepOver => depth <= goal.depth,
                DebugCommand::StepOut => depth < goal.depth,
                _ => false,
            })
        {
            Some(PauseReason::Step)
        } else {
            None
        };
        let reason = reason?;

        let snapshot = self.build_snapshot(frame, reason);
        // Take the debugger out so the host can borrow the interpreter
        // (as the watch evaluator) while we call it — which also makes
        // watch execution run without debug hooks, exactly right
        // (watches don't hit breakpoints).
        let debug = self.debug.take().expect("still debugging");
        let control = {
            let mut watch = FrameWatchEvaluator {
                interp: self,
                active: frame,
            };
            debug.host.on_pause(&snapshot, &mut watch)
        };
        self.debug = Some(debug);
        let debug = self.debug.as_mut().expect("just restored");
        if let Some(new_breakpoints) = control.breakpoints {
            debug.breakpoints = new_breakpoints
                .into_iter()
                .map(|b| (b.file, b.line))
                .collect();
        }
        debug.step = match control.command {
            DebugCommand::StepInto | DebugCommand::StepOver | DebugCommand::StepOut => {
                Some(StepGoal {
                    mode: control.command,
                    depth,
                })
            }
            _ => None,
        };
        debug.resume_line = Some((frame.code.source_file.clone(), boundary_line));
        Some(control.command)
    }

    /// Snapshot the paused call stack, innermost frame first.
    fn build_snapshot(&self, active: &Frame<'run>, reason: PauseReason) -> DebugSnapshot {
        let mut frames = Vec::with_capacity(1 + self.frames.len());
        frames.push(self.snapshot_frame(active));
        for suspended in self.frames.iter().rev() {
            frames.push(self.snapshot_frame(suspended));
        }
        DebugSnapshot { reason, frames }
    }

    fn snapshot_frame(&self, frame: &Frame<'run>) -> DebugFrameSnapshot {
        let pc = u16::try_from(frame.pc).unwrap_or(u16::MAX);
        let locals = frame
            .code
            .local_vars
            .iter()
            .filter(|(_, _, _, start_pc)| *start_pc <= pc)
            .map(|(name, source_type, slot, _)| {
                let value = frame
                    .locals
                    .get(usize::from(*slot))
                    .copied()
                    .unwrap_or(JValue::NULL);
                crate::debug::LocalSnapshot {
                    name: name.clone(),
                    type_name: source_type.clone(),
                    value: self.render_value(value),
                }
            })
            .collect();
        DebugFrameSnapshot {
            class_name: frame.class.class_name().unwrap_or("<unknown>").to_owned(),
            method_name: frame.method_name.to_owned(),
            source_file: frame.code.source_file.clone(),
            line: frame.current_line.map(u32::from),
            locals,
        }
    }

    /// Render a value for the paused-locals view. Instances expand one
    /// level (`Dog@1{age=3, name="Rex"}`); nested references inside
    /// stay shallow to keep the display bounded.
    fn render_value(&self, value: JValue) -> String {
        self.render_value_depth(value, 0)
    }

    fn render_shallow(&self, value: JValue) -> String {
        self.render_value_depth(value, 1)
    }

    #[allow(clippy::too_many_lines)] // one rendering ladder
    fn render_value_depth(&self, value: JValue, depth: u8) -> String {
        match value {
            JValue::Int(v) => v.to_string(),
            JValue::Long(v) => v.to_string(),
            JValue::Float(v) => v.to_string(),
            JValue::Double(v) => intrinsics::java_double_to_string(v),
            JValue::Ref(None) => String::from("null"),
            JValue::Ref(Some(reference)) => match self.heap.get(reference) {
                Some(crate::value::HeapObject::JavaString(units)) => {
                    format!("\"{}\"", String::from_utf16_lossy(units))
                }
                Some(crate::value::HeapObject::IntArray(kind, values)) => {
                    render_array(values.iter().map(|v| int_element_text(*kind, *v)))
                }
                Some(crate::value::HeapObject::DoubleArray(values)) => {
                    render_array(values.iter().map(|v| intrinsics::java_double_to_string(*v)))
                }
                Some(crate::value::HeapObject::RefArray(_, values)) => {
                    render_array(values.iter().map(|v| self.render_shallow(*v)))
                }
                Some(
                    crate::value::HeapObject::ArrayList(values)
                    | crate::value::HeapObject::LinkedList(values)
                    | crate::value::HeapObject::ArrayDeque(values)
                    | crate::value::HeapObject::Stack(values),
                ) => render_array(values.iter().map(|v| self.render_shallow(*v))),
                Some(crate::value::HeapObject::Instance { class_name, fields }) => {
                    if depth > 0 || fields.is_empty() {
                        return format!("{class_name}@{reference:x}");
                    }
                    // Sorted for stable display (fields live in a map);
                    // reserved names (the throwable message) hidden.
                    let mut keys: Vec<&str> = fields
                        .keys()
                        .map(|k| &**k)
                        .filter(|k| !split_field_key(k).1.starts_with("__"))
                        .collect();
                    keys.sort_unstable();
                    // A hidden field shares its simple name with the one it
                    // hides, so name both by their declaring class; otherwise
                    // the simple name reads as the student wrote it.
                    let mut seen: HashMap<&str, usize> = HashMap::new();
                    for key in &keys {
                        *seen.entry(split_field_key(key).1).or_default() += 1;
                    }
                    let rendered: Vec<String> = keys
                        .into_iter()
                        .map(|key| {
                            let (owner, simple) = split_field_key(key);
                            let label = if seen[simple] > 1 {
                                format!("{owner}.{simple}")
                            } else {
                                simple.to_owned()
                            };
                            format!("{label}={}", self.render_shallow(fields[key]))
                        })
                        .collect();
                    format!("{class_name}@{reference:x}{{{}}}", rendered.join(", "))
                }
                Some(crate::value::HeapObject::Exception {
                    class_name,
                    message,
                }) => match message {
                    Some(message) => format!("{class_name}: {message}"),
                    None => class_name.clone(),
                },
                Some(crate::value::HeapObject::Field {
                    declaring,
                    name,
                    descriptor,
                    access,
                    ..
                }) => intrinsics::field_to_string(declaring, name, descriptor, *access),
                Some(crate::value::HeapObject::Constructor {
                    declaring,
                    descriptor,
                    access,
                }) => intrinsics::constructor_to_string(declaring, descriptor, *access),
                Some(crate::value::HeapObject::Class { name }) => format!("class {name}"),
                Some(crate::value::HeapObject::Scanner { .. }) => String::from("Scanner"),
                Some(crate::value::HeapObject::File(path)) => format!("File({path})"),
                Some(crate::value::HeapObject::Writer { path }) => {
                    format!("PrintWriter({path})")
                }
                _ => String::from("<object>"),
            },
        }
    }

    /// Build a frame for a call: depth check, cached `Code` lookup,
    /// locals sized to `max_locals`.
    fn make_frame(
        &mut self,
        class: &'run ClassFile,
        method: &'run MethodInfo,
        mut locals: Vec<JValue>,
    ) -> Result<Frame<'run>, VmError> {
        if self.call_depth() >= usize::try_from(self.max_call_depth).unwrap_or(usize::MAX) {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.StackOverflowError",
            )));
        }
        let key = std::ptr::from_ref(method) as usize;
        let code = if let Some(cached) = self.code_cache.get(&key) {
            Rc::clone(cached)
        } else {
            let parsed = Rc::new(parse_method_code(class, method)?);
            self.code_cache.insert(key, Rc::clone(&parsed));
            parsed
        };
        if locals.len() < usize::from(code.attr.max_locals) {
            locals.resize(usize::from(code.attr.max_locals), JValue::Int(0));
        }
        let method_name = class
            .constant_pool
            .get_utf8(method.name_index)
            .unwrap_or("<unknown>");
        Ok(Frame {
            class,
            method_name,
            code,
            pc: 0,
            current_line: None,
            locals,
            stack: Vec::new(),
            box_return_as: None,
        })
    }

    /// The dispatch loop is one long match by design; splitting it per
    /// opcode family would only scatter the instruction set. The outer
    /// `'frames` loop re-establishes per-frame context after every
    /// call/return; the inner loop decodes instructions.
    #[allow(clippy::too_many_lines)]
    fn run_loop(&mut self, mut frame: Frame<'run>) -> Result<Option<JValue>, VmError> {
        'frames: loop {
            let class = frame.class;
            let class_name = class.class_name().unwrap_or("<unknown>").to_owned();
            self.current_location = Some(CurrentLocation {
                class_name: class_name.clone(),
                method_name: frame.method_name.to_owned(),
                source_file: frame.code.source_file.clone(),
                line: frame.current_line,
            });
            let malformed = |reason: String| VmError::MalformedClass {
                name: class_name.clone(),
                reason,
            };
            let code = Rc::clone(&frame.code);
            let bytes = &code.attr.code;
            let mut pc = frame.pc;

            loop {
                if self.remaining_instructions == 0 {
                    return Err(VmError::InstructionBudgetExceeded);
                }
                self.remaining_instructions -= 1;

                let addr = pc;

                // Source-line boundary: update the current line (for
                // stack traces) and give the debugger a chance to
                // pause before this line's first instruction runs.
                let boundary_line = code.boundary_lines.get(addr).copied().unwrap_or(0);
                if boundary_line != 0 {
                    frame.current_line = Some(boundary_line);
                    if let Some(location) = &mut self.current_location {
                        location.line = Some(boundary_line);
                    }
                }
                if self.debug.is_some() {
                    frame.pc = addr;
                    if self.debug_checkpoint(&mut frame, boundary_line)
                        == Some(DebugCommand::Terminate)
                    {
                        return Err(VmError::Stopped);
                    }
                }

                let opcode = *bytes.get(addr).ok_or_else(|| {
                    malformed(format!(
                        "execution ran off the end of bytecode at pc {addr}"
                    ))
                })?;
                pc = addr + 1;

                // One instruction, in a closure so a thrown Java
                // exception surfaces here (where the frame stack is
                // intact for handler search) instead of exiting the VM.
                let flow = (|| -> Result<Flow<'run>, VmError> {
                    match opcode {
                        op::NOP => {}
                        op::ACONST_NULL => frame.stack.push(JValue::NULL),
                        op::LCONST_0 => frame.stack.push(JValue::Long(0)),
                        op::LCONST_1 => frame.stack.push(JValue::Long(1)),
                        op::FCONST_0 => frame.stack.push(JValue::Float(0.0)),
                        op::FCONST_1 => frame.stack.push(JValue::Float(1.0)),
                        op::FCONST_2 => frame.stack.push(JValue::Float(2.0)),
                        op::ICONST_M1..=op::ICONST_5 => {
                            let value = i32::from(opcode) - i32::from(op::ICONST_0);
                            frame.stack.push(JValue::Int(value));
                        }
                        op::BIPUSH => {
                            let byte = read_u8(bytes, &mut pc, &malformed)?;
                            frame.stack.push(JValue::Int(i32::from(byte.cast_signed())));
                        }
                        op::SIPUSH => {
                            let short = read_u16(bytes, &mut pc, &malformed)?;
                            frame
                                .stack
                                .push(JValue::Int(i32::from(short.cast_signed())));
                        }
                        op::LDC => {
                            let index = u16::from(read_u8(bytes, &mut pc, &malformed)?);
                            let value = self.load_constant(class, index, &malformed)?;
                            frame.stack.push(value);
                        }
                        op::LDC_W | op::LDC2_W => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            let value = self.load_constant(class, index, &malformed)?;
                            frame.stack.push(value);
                        }

                        // ----- locals -----
                        op::ILOAD | op::DLOAD | op::ALOAD | op::LLOAD | op::FLOAD => {
                            let slot = usize::from(read_u8(bytes, &mut pc, &malformed)?);
                            frame.load(slot, &malformed)?;
                        }
                        op::ILOAD_0..=op::ILOAD_3 => {
                            frame.load(usize::from(opcode - op::ILOAD_0), &malformed)?;
                        }
                        op::DLOAD_0..=op::DLOAD_3 => {
                            frame.load(usize::from(opcode - op::DLOAD_0), &malformed)?;
                        }
                        op::ALOAD_0..=op::ALOAD_3 => {
                            frame.load(usize::from(opcode - op::ALOAD_0), &malformed)?;
                        }
                        op::LLOAD_0..=op::LLOAD_3 => {
                            frame.load(usize::from(opcode - op::LLOAD_0), &malformed)?;
                        }
                        op::FLOAD_0..=op::FLOAD_3 => {
                            frame.load(usize::from(opcode - op::FLOAD_0), &malformed)?;
                        }
                        op::ISTORE | op::DSTORE | op::ASTORE | op::LSTORE | op::FSTORE => {
                            let slot = usize::from(read_u8(bytes, &mut pc, &malformed)?);
                            frame.store(slot, &malformed)?;
                        }
                        op::ISTORE_0..=op::ISTORE_3 => {
                            frame.store(usize::from(opcode - op::ISTORE_0), &malformed)?;
                        }
                        op::DSTORE_0..=op::DSTORE_3 => {
                            frame.store(usize::from(opcode - op::DSTORE_0), &malformed)?;
                        }
                        op::ASTORE_0..=op::ASTORE_3 => {
                            frame.store(usize::from(opcode - op::ASTORE_0), &malformed)?;
                        }
                        op::LSTORE_0..=op::LSTORE_3 => {
                            frame.store(usize::from(opcode - op::LSTORE_0), &malformed)?;
                        }
                        op::FSTORE_0..=op::FSTORE_3 => {
                            frame.store(usize::from(opcode - op::FSTORE_0), &malformed)?;
                        }

                        // ----- stack manipulation -----
                        op::POP => {
                            frame.pop()?;
                        }
                        op::POP2 => {
                            // One category-2 value or two category-1 values.
                            let top = frame.pop()?;
                            if !matches!(top, JValue::Double(_) | JValue::Long(_)) {
                                frame.pop()?;
                            }
                        }
                        op::DUP => {
                            let top = *frame.stack.last().ok_or(VmError::StackUnderflow)?;
                            frame.stack.push(top);
                        }
                        op::DUP2 => {
                            // One category-2 value or the top two category-1s.
                            let top = *frame.stack.last().ok_or(VmError::StackUnderflow)?;
                            if matches!(top, JValue::Double(_) | JValue::Long(_)) {
                                frame.stack.push(top);
                            } else {
                                let len = frame.stack.len();
                                if len < 2 {
                                    return Err(VmError::StackUnderflow);
                                }
                                let under = frame.stack[len - 2];
                                frame.stack.push(under);
                                frame.stack.push(top);
                            }
                        }
                        op::DUP_X1 => {
                            let len = frame.stack.len();
                            if len < 2 {
                                return Err(VmError::StackUnderflow);
                            }
                            let top = frame.stack[len - 1];
                            frame.stack.insert(len - 2, top);
                        }
                        op::DUP_X2 => {
                            // Form 2 when the value underneath is category
                            // 2 (double/long occupy one JValue here).
                            let len = frame.stack.len();
                            if len < 2 {
                                return Err(VmError::StackUnderflow);
                            }
                            let top = frame.stack[len - 1];
                            let under_cat2 =
                                matches!(frame.stack[len - 2], JValue::Double(_) | JValue::Long(_));
                            let depth = if under_cat2 { 2 } else { 3 };
                            if len < depth {
                                return Err(VmError::StackUnderflow);
                            }
                            frame.stack.insert(len - depth, top);
                        }
                        op::DUP2_X1 => {
                            // Form 2: a category-2 top over one category-1.
                            let len = frame.stack.len();
                            if len < 2 {
                                return Err(VmError::StackUnderflow);
                            }
                            let top = frame.stack[len - 1];
                            if matches!(top, JValue::Double(_) | JValue::Long(_)) {
                                frame.stack.insert(len - 2, top);
                            } else {
                                if len < 3 {
                                    return Err(VmError::StackUnderflow);
                                }
                                let pair = [frame.stack[len - 2], frame.stack[len - 1]];
                                frame.stack.insert(len - 3, pair[1]);
                                frame.stack.insert(len - 3, pair[0]);
                            }
                        }
                        op::DUP2_X2 => {
                            let len = frame.stack.len();
                            if len < 2 {
                                return Err(VmError::StackUnderflow);
                            }
                            let top_cat2 =
                                matches!(frame.stack[len - 1], JValue::Double(_) | JValue::Long(_));
                            let top_width = if top_cat2 { 1 } else { 2 };
                            if len < top_width + 1 {
                                return Err(VmError::StackUnderflow);
                            }
                            let under_index = len - top_width - 1;
                            let under_cat2 = matches!(
                                frame.stack[under_index],
                                JValue::Double(_) | JValue::Long(_)
                            );
                            let under_width = if under_cat2 { 1 } else { 2 };
                            let insert_at = len - top_width - under_width;
                            let copied: Vec<JValue> = frame.stack[len - top_width..].to_vec();
                            for (offset, value) in copied.into_iter().enumerate() {
                                frame.stack.insert(insert_at + offset, value);
                            }
                        }
                        op::SWAP => {
                            let len = frame.stack.len();
                            if len < 2 {
                                return Err(VmError::StackUnderflow);
                            }
                            frame.stack.swap(len - 1, len - 2);
                        }

                        // ----- arithmetic (Java wrapping semantics) -----
                        op::IADD => frame.int_binop(i32::wrapping_add)?,
                        op::ISUB => frame.int_binop(i32::wrapping_sub)?,
                        op::IMUL => frame.int_binop(i32::wrapping_mul)?,
                        op::IDIV => frame.int_division(i32::wrapping_div)?,
                        op::IREM => frame.int_division(i32::wrapping_rem)?,
                        op::INEG => {
                            let value = frame.pop_int()?;
                            frame.stack.push(JValue::Int(value.wrapping_neg()));
                        }
                        // Shift counts mask to the low five bits (JVMS).
                        op::ISHL => {
                            frame.int_binop(|a, b| a.wrapping_shl(b.cast_unsigned() & 0x1F))?;
                        }
                        op::ISHR => {
                            frame.int_binop(|a, b| a.wrapping_shr(b.cast_unsigned() & 0x1F))?;
                        }
                        op::IUSHR => frame.int_binop(|a, b| {
                            (a.cast_unsigned().wrapping_shr(b.cast_unsigned() & 0x1F)).cast_signed()
                        })?,
                        op::FADD => frame.float_binop(|a, b| a + b)?,
                        op::FSUB => frame.float_binop(|a, b| a - b)?,
                        op::FMUL => frame.float_binop(|a, b| a * b)?,
                        op::FDIV => frame.float_binop(|a, b| a / b)?,
                        op::FREM => frame.float_binop(|a, b| a % b)?,
                        op::FNEG => {
                            let value = frame.pop_float()?;
                            frame.stack.push(JValue::Float(-value));
                        }
                        op::FCMPL | op::FCMPG => {
                            let b = frame.pop_float()?;
                            let a = frame.pop_float()?;
                            let result = if a.is_nan() || b.is_nan() {
                                if opcode == op::FCMPG { 1 } else { -1 }
                            } else if a > b {
                                1
                            } else if a < b {
                                -1
                            } else {
                                0
                            };
                            frame.stack.push(JValue::Int(result));
                        }
                        op::LADD => frame.long_binop(i64::wrapping_add)?,
                        op::LSUB => frame.long_binop(i64::wrapping_sub)?,
                        op::LMUL => frame.long_binop(i64::wrapping_mul)?,
                        op::LDIV => frame.long_division(i64::wrapping_div)?,
                        op::LREM => frame.long_division(i64::wrapping_rem)?,
                        op::LNEG => {
                            let value = frame.pop_long()?;
                            frame.stack.push(JValue::Long(value.wrapping_neg()));
                        }
                        op::LAND => frame.long_binop(|a, b| a & b)?,
                        op::LOR => frame.long_binop(|a, b| a | b)?,
                        op::LXOR => frame.long_binop(|a, b| a ^ b)?,
                        // Long shift counts come from an int and mask
                        // to six bits (JVMS).
                        op::LSHL => {
                            let count = frame.pop_int()?.cast_unsigned() & 0x3F;
                            let value = frame.pop_long()?;
                            frame.stack.push(JValue::Long(value.wrapping_shl(count)));
                        }
                        op::LSHR => {
                            let count = frame.pop_int()?.cast_unsigned() & 0x3F;
                            let value = frame.pop_long()?;
                            frame.stack.push(JValue::Long(value.wrapping_shr(count)));
                        }
                        op::LUSHR => {
                            let count = frame.pop_int()?.cast_unsigned() & 0x3F;
                            let value = frame.pop_long()?;
                            frame.stack.push(JValue::Long(
                                value.cast_unsigned().wrapping_shr(count).cast_signed(),
                            ));
                        }
                        op::LCMP => {
                            let b = frame.pop_long()?;
                            let a = frame.pop_long()?;
                            frame.stack.push(JValue::Int(match a.cmp(&b) {
                                std::cmp::Ordering::Less => -1,
                                std::cmp::Ordering::Equal => 0,
                                std::cmp::Ordering::Greater => 1,
                            }));
                        }
                        op::IAND => frame.int_binop(|a, b| a & b)?,
                        op::IOR => frame.int_binop(|a, b| a | b)?,
                        op::IXOR => frame.int_binop(|a, b| a ^ b)?,
                        op::DADD => frame.double_binop(|a, b| a + b)?,
                        op::DSUB => frame.double_binop(|a, b| a - b)?,
                        op::DMUL => frame.double_binop(|a, b| a * b)?,
                        op::DDIV => frame.double_binop(|a, b| a / b)?,
                        op::DREM => frame.double_binop(|a, b| a % b)?,
                        op::DNEG => {
                            let value = frame.pop_double()?;
                            frame.stack.push(JValue::Double(-value));
                        }

                        // ----- conversions -----
                        op::I2D => {
                            let value = frame.pop_int()?;
                            frame.stack.push(JValue::Double(f64::from(value)));
                        }
                        op::D2I => {
                            let value = frame.pop_double()?;
                            // `as` matches JVM d2i: NaN -> 0, saturating bounds.
                            frame.stack.push(JValue::Int(value as i32));
                        }
                        op::I2F => {
                            let value = frame.pop_int()?;
                            #[allow(clippy::cast_precision_loss)]
                            frame.stack.push(JValue::Float(value as f32));
                        }
                        op::L2F => {
                            let value = frame.pop_long()?;
                            #[allow(clippy::cast_precision_loss)]
                            frame.stack.push(JValue::Float(value as f32));
                        }
                        op::F2I => {
                            let value = frame.pop_float()?;
                            // `as` matches JVM f2i: NaN -> 0, saturating.
                            #[allow(clippy::cast_possible_truncation)]
                            frame.stack.push(JValue::Int(value as i32));
                        }
                        op::F2L => {
                            let value = frame.pop_float()?;
                            #[allow(clippy::cast_possible_truncation)]
                            frame.stack.push(JValue::Long(value as i64));
                        }
                        op::F2D => {
                            let value = frame.pop_float()?;
                            frame.stack.push(JValue::Double(f64::from(value)));
                        }
                        op::D2F => {
                            let value = frame.pop_double()?;
                            #[allow(clippy::cast_possible_truncation)]
                            frame.stack.push(JValue::Float(value as f32));
                        }
                        op::I2L => {
                            let value = frame.pop_int()?;
                            frame.stack.push(JValue::Long(i64::from(value)));
                        }
                        op::L2I => {
                            let value = frame.pop_long()?;
                            #[allow(clippy::cast_possible_truncation)]
                            frame.stack.push(JValue::Int(value as i32));
                        }
                        op::L2D => {
                            let value = frame.pop_long()?;
                            #[allow(clippy::cast_precision_loss)]
                            frame.stack.push(JValue::Double(value as f64));
                        }
                        op::D2L => {
                            let value = frame.pop_double()?;
                            // `as` matches JVM d2l: NaN -> 0, saturating.
                            #[allow(clippy::cast_possible_truncation)]
                            frame.stack.push(JValue::Long(value as i64));
                        }
                        op::I2B => {
                            let value = frame.pop_int()?;
                            #[allow(clippy::cast_possible_truncation)]
                            frame.stack.push(JValue::Int(i32::from(value as i8)));
                        }
                        op::I2S => {
                            let value = frame.pop_int()?;
                            #[allow(clippy::cast_possible_truncation)]
                            frame.stack.push(JValue::Int(i32::from(value as i16)));
                        }
                        op::I2C => {
                            let value = frame.pop_int()?;
                            // i2c truncates to 16 bits and zero-extends.
                            let truncated = (value.cast_unsigned() & 0xFFFF).cast_signed();
                            frame.stack.push(JValue::Int(truncated));
                        }

                        // ----- comparisons and branches -----
                        op::DCMPL | op::DCMPG => {
                            let b = frame.pop_double()?;
                            let a = frame.pop_double()?;
                            let result = if a.is_nan() || b.is_nan() {
                                if opcode == op::DCMPG { 1 } else { -1 }
                            } else if a < b {
                                -1
                            } else {
                                i32::from(a > b)
                            };
                            frame.stack.push(JValue::Int(result));
                        }
                        op::IFEQ..=op::IFLE => {
                            let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                            let value = frame.pop_int()?;
                            let jump = match opcode {
                                op::IFEQ => value == 0,
                                op::IFNE => value != 0,
                                op::IFLT => value < 0,
                                op::IFGE => value >= 0,
                                op::IFGT => value > 0,
                                _ => value <= 0,
                            };
                            if jump {
                                pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                            }
                        }
                        op::IF_ICMPEQ..=op::IF_ICMPLE => {
                            let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                            let b = frame.pop_int()?;
                            let a = frame.pop_int()?;
                            let jump = match opcode {
                                op::IF_ICMPEQ => a == b,
                                op::IF_ICMPNE => a != b,
                                op::IF_ICMPLT => a < b,
                                op::IF_ICMPGE => a >= b,
                                op::IF_ICMPGT => a > b,
                                _ => a <= b,
                            };
                            if jump {
                                pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                            }
                        }
                        op::IF_ACMPEQ | op::IF_ACMPNE => {
                            let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                            let b = frame.pop_ref()?;
                            let a = frame.pop_ref()?;
                            let jump = (a == b) == (opcode == op::IF_ACMPEQ);
                            if jump {
                                pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                            }
                        }
                        op::IFNULL | op::IFNONNULL => {
                            let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                            let reference = frame.pop_ref()?;
                            if reference.is_none() == (opcode == op::IFNULL) {
                                pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                            }
                        }
                        op::GOTO => {
                            let offset = read_u16(bytes, &mut pc, &malformed)?.cast_signed();
                            pc = branch_target(addr, offset, bytes.len(), &malformed)?;
                        }
                        op::CHECKCAST | op::INSTANCEOF => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            let target = class
                                .constant_pool
                                .get_class_name(index)
                                .ok_or_else(|| malformed(format!("bad class ref at pool {index}")))?
                                .to_owned();
                            let reference = frame.pop_ref()?;
                            let matches_type = match reference {
                                None => false,
                                // Everything is an Object.
                                Some(_) if target == "java/lang/Object" => true,
                                // An array target (`[I`, `[Ljava/lang/String;`,
                                // `[[I`): the reference must be an array whose
                                // class is assignable to it.
                                Some(reference) if target.starts_with('[') => {
                                    intrinsics::array_class_name(&self.heap, reference)
                                        .is_some_and(|actual| array_cast_ok(&actual, &target))
                                }
                                Some(reference) => match self.heap.get(reference) {
                                    Some(crate::value::HeapObject::Instance {
                                        class_name, ..
                                    }) => self.is_runtime_subtype(class_name, &target),
                                    Some(crate::value::HeapObject::JavaString(_)) => {
                                        target == "java/lang/String"
                                    }
                                    Some(crate::value::HeapObject::ArrayList(_)) => {
                                        target == "java/util/ArrayList"
                                            || target == "java/util/List"
                                    }
                                    Some(crate::value::HeapObject::Stack(_)) => matches!(
                                        target.as_str(),
                                        "java/util/Stack"
                                            | "java/util/Vector"
                                            | "java/util/List"
                                            | "java/util/Collection"
                                    ),
                                    Some(crate::value::HeapObject::LinkedList(_)) => matches!(
                                        target.as_str(),
                                        "java/util/LinkedList"
                                            | "java/util/List"
                                            | "java/util/Queue"
                                            | "java/util/Deque"
                                            | "java/util/Collection"
                                    ),
                                    Some(crate::value::HeapObject::ArrayDeque(_)) => matches!(
                                        target.as_str(),
                                        "java/util/ArrayDeque"
                                            | "java/util/Queue"
                                            | "java/util/Deque"
                                            | "java/util/Collection"
                                    ),
                                    // Boxed wrapper: `x instanceof Integer`.
                                    Some(crate::value::HeapObject::Boxed {
                                        class_name, ..
                                    }) => *class_name == target,
                                    // A reflect Type: it is a ParameterizedType
                                    // only when it carries type arguments.
                                    Some(crate::value::HeapObject::ReflectType {
                                        args, ..
                                    }) => {
                                        target == "java/lang/reflect/Type"
                                            || (target == "java/lang/reflect/ParameterizedType"
                                                && !args.is_empty())
                                    }
                                    _ => false,
                                },
                            };
                            if opcode == op::INSTANCEOF {
                                frame.stack.push(JValue::Int(i32::from(matches_type)));
                            } else {
                                // checkcast: null always passes; a mismatch throws.
                                if reference.is_some() && !matches_type {
                                    let actual = reference
                                        .and_then(|r| {
                                            intrinsics::array_class_name(&self.heap, r).or_else(
                                                || match self.heap.get(r) {
                                                    Some(crate::value::HeapObject::Instance {
                                                        class_name,
                                                        ..
                                                    }) => Some(class_name.clone()),
                                                    Some(crate::value::HeapObject::JavaString(
                                                        _,
                                                    )) => Some(String::from("java/lang/String")),
                                                    _ => None,
                                                },
                                            )
                                        })
                                        .unwrap_or_else(|| String::from("<object>"));
                                    return Err(VmError::UncaughtException(format!(
                                        "java.lang.ClassCastException: class {actual} cannot be cast \
                                 to class {target}"
                                    )));
                                }
                                frame.stack.push(JValue::Ref(reference));
                            }
                        }

                        op::ATHROW => {
                            let reference = frame.pop_ref()?.ok_or_else(|| {
                                VmError::UncaughtException(String::from(
                                    "java.lang.NullPointerException: cannot throw null",
                                ))
                            })?;
                            self.last_thrown = Some(reference);
                            match self.heap.get(reference) {
                                Some(crate::value::HeapObject::Exception {
                                    class_name,
                                    message,
                                }) => {
                                    return Err(VmError::UncaughtException(match message {
                                        Some(message) => format!("{class_name}: {message}"),
                                        None => class_name.clone(),
                                    }));
                                }
                                Some(crate::value::HeapObject::Instance { class_name, fields })
                                    if self.instance_is_throwable(class_name) =>
                                {
                                    let message =
                                        fields.get("__message").and_then(|value| match value {
                                            JValue::Ref(Some(text)) => self.heap.string_text(*text),
                                            _ => None,
                                        });
                                    return Err(VmError::UncaughtException(match message {
                                        Some(message) => format!("{class_name}: {message}"),
                                        None => class_name.clone(),
                                    }));
                                }
                                _ => {
                                    return Err(malformed(String::from(
                                        "athrow on a non-throwable",
                                    )));
                                }
                            }
                        }

                        // ----- objects -----
                        op::NEW => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            let target = class
                                .constant_pool
                                .get_class_name(index)
                                .ok_or_else(|| malformed(format!("bad class ref at pool {index}")))?
                                .to_owned();
                            let object = if self.classes.contains_key(&target) {
                                if let Some(chain) = self.begin_initialization(&target)?
                                    && !chain.is_empty()
                                {
                                    // Run <clinit> chain first, then re-execute
                                    // this instruction (JVMS §5.5).
                                    frame.pc = addr;
                                    return Ok(Flow::InitChain(chain));
                                }
                                self.new_instance(&target)
                            } else {
                                intrinsics::instantiate(&target).ok_or_else(|| {
                                    VmError::UnknownIntrinsic(format!(
                                        "cannot instantiate {target}"
                                    ))
                                })?
                            };
                            let reference = self.heap.alloc(object);
                            frame.stack.push(JValue::Ref(Some(reference)));
                        }
                        op::INVOKESPECIAL => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            if let Some(callee) =
                                self.invoke_special_op(class, &mut frame, index, &malformed)?
                            {
                                frame.pc = pc;
                                return Ok(Flow::Call(callee));
                            }
                        }
                        op::GETSTATIC | op::PUTSTATIC => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            let field_class = class
                                .constant_pool
                                .get_member_ref(index)
                                .map(|(c, _, _)| c.to_owned())
                                .ok_or_else(|| {
                                    malformed(format!("bad field ref at pool {index}"))
                                })?;
                            if let Some(chain) = self.begin_initialization(&field_class)?
                                && !chain.is_empty()
                            {
                                frame.pc = addr;
                                return Ok(Flow::InitChain(chain));
                            }
                            if opcode == op::GETSTATIC {
                                self.getstatic_op(class, &mut frame, index, &malformed)?;
                            } else {
                                self.putstatic_op(class, &mut frame, index, &malformed)?;
                            }
                        }
                        op::GETFIELD => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            self.getfield_op(class, &mut frame, index, &malformed)?;
                        }
                        op::PUTFIELD => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            self.putfield_op(class, &mut frame, index, &malformed)?;
                        }
                        op::INVOKEVIRTUAL => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            if let Some(callee) =
                                self.invoke_virtual_op(class, &mut frame, index, &malformed)?
                            {
                                frame.pc = pc;
                                return Ok(Flow::Call(callee));
                            }
                        }
                        op::INVOKESTATIC => {
                            let index = read_u16(bytes, &mut pc, &malformed)?;
                            let (target_class, method_name, descriptor) = class
                                .constant_pool
                                .get_member_ref(index)
                                .map(|(c, m, d)| (c.to_owned(), m.to_owned(), d.to_owned()))
                                .ok_or_else(|| {
                                    malformed(format!("bad method ref at pool {index}"))
                                })?;
                            // Initialization retry happens before arguments are
                            // popped so the re-executed instruction sees them.
                            if let Some(chain) = self.begin_initialization(&target_class)?
                                && !chain.is_empty()
                            {
                                frame.pc = addr;
                                return Ok(Flow::InitChain(chain));
                            }
                            let widths = descriptor_arg_widths(&descriptor)
                                .ok_or_else(|| malformed(format!("bad descriptor {descriptor}")))?;
                            let mut args = Vec::with_capacity(widths.len());
                            for _ in &widths {
                                args.push(frame.pop()?);
                            }
                            args.reverse();
                            if let Some(callee) = self.invoke_static(
                                &target_class,
                                &method_name,
                                &descriptor,
                                &widths,
                                &args,
                                &mut frame,
                            )? {
                                frame.pc = pc;
                                return Ok(Flow::Call(callee));
                            }
                        }
                        // ----- arrays -----
                        op::NEWARRAY => {
                            let atype = read_u8(bytes, &mut pc, &malformed)?;
                            let length = check_array_size(frame.pop_int()?)?;
                            let object = match atype {
                                op::T_INT => crate::value::HeapObject::IntArray(
                                    crate::value::IntKind::Int,
                                    vec![0; length],
                                ),
                                op::T_BOOLEAN => crate::value::HeapObject::IntArray(
                                    crate::value::IntKind::Boolean,
                                    vec![0; length],
                                ),
                                op::T_CHAR => crate::value::HeapObject::IntArray(
                                    crate::value::IntKind::Char,
                                    vec![0; length],
                                ),
                                op::T_DOUBLE => {
                                    crate::value::HeapObject::DoubleArray(vec![0.0; length])
                                }
                                op::T_LONG => crate::value::HeapObject::LongArray(vec![0; length]),
                                op::T_FLOAT => {
                                    crate::value::HeapObject::FloatArray(vec![0.0; length])
                                }
                                op::T_SHORT => {
                                    crate::value::HeapObject::ShortArray(vec![0; length])
                                }
                                op::T_BYTE => crate::value::HeapObject::ByteArray(vec![0; length]),
                                other => {
                                    return Err(malformed(format!(
                                        "unsupported newarray type {other}"
                                    )));
                                }
                            };
                            let reference = self.heap.alloc(object);
                            frame.stack.push(JValue::Ref(Some(reference)));
                        }
                        op::ANEWARRAY => {
                            let class_index = read_u16(bytes, &mut pc, &malformed)?;
                            // The pool names the ELEMENT class (`java/lang/String`)
                            // or, for a row of a higher-dimension array, its own
                            // descriptor (`[I`). The array's class is one `[` deeper.
                            let element =
                                class.constant_pool.get_class_name(class_index).ok_or_else(
                                    || malformed(format!("bad class ref at pool {class_index}")),
                                )?;
                            let array_class = array_class_of(element);
                            let length = check_array_size(frame.pop_int()?)?;
                            let reference = self.heap.alloc(crate::value::HeapObject::RefArray(
                                array_class,
                                vec![JValue::NULL; length],
                            ));
                            frame.stack.push(JValue::Ref(Some(reference)));
                        }
                        op::MULTIANEWARRAY => {
                            let class_index = read_u16(bytes, &mut pc, &malformed)?;
                            let dims = usize::from(read_u8(bytes, &mut pc, &malformed)?);
                            let descriptor = class
                                .constant_pool
                                .get_class_name(class_index)
                                .ok_or_else(|| {
                                    malformed(format!("bad class ref at pool {class_index}"))
                                })?
                                .to_owned();
                            let mut counts = Vec::with_capacity(dims);
                            for _ in 0..dims {
                                counts.push(frame.pop_int()?);
                            }
                            counts.reverse();
                            let reference =
                                self.alloc_multi_array(&descriptor, &counts, &malformed)?;
                            frame.stack.push(JValue::Ref(Some(reference)));
                        }
                        op::ARRAYLENGTH => {
                            let reference = frame.pop_ref()?.ok_or_else(null_array)?;
                            let length = match self.heap.get(reference) {
                                Some(crate::value::HeapObject::IntArray(_, v)) => v.len(),
                                Some(crate::value::HeapObject::DoubleArray(v)) => v.len(),
                                Some(crate::value::HeapObject::RefArray(_, v)) => v.len(),
                                Some(crate::value::HeapObject::LongArray(v)) => v.len(),
                                Some(crate::value::HeapObject::FloatArray(v)) => v.len(),
                                Some(crate::value::HeapObject::ShortArray(v)) => v.len(),
                                Some(crate::value::HeapObject::ByteArray(v)) => v.len(),
                                _ => {
                                    return Err(malformed(String::from(
                                        "arraylength on a non-array",
                                    )));
                                }
                            };
                            frame
                                .stack
                                .push(JValue::Int(i32::try_from(length).unwrap_or(i32::MAX)));
                        }
                        op::IALOAD | op::BALOAD | op::CALOAD | op::SALOAD => {
                            let (reference, index) = frame.pop_array_access()?;
                            let value = match self.heap.get(reference) {
                                Some(crate::value::HeapObject::IntArray(_, values)) => {
                                    *array_get(values, index)?
                                }
                                Some(crate::value::HeapObject::ByteArray(values))
                                    if opcode == op::BALOAD =>
                                {
                                    i32::from(*array_get(values, index)?)
                                }
                                Some(crate::value::HeapObject::ShortArray(values))
                                    if opcode == op::SALOAD =>
                                {
                                    i32::from(*array_get(values, index)?)
                                }
                                _ => {
                                    return Err(malformed(String::from(
                                        "int-array load on a non-int-array",
                                    )));
                                }
                            };
                            frame.stack.push(JValue::Int(value));
                        }
                        op::FALOAD => {
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::FloatArray(values)) =
                                self.heap.get(reference)
                            else {
                                return Err(malformed(String::from("faload on a non-float-array")));
                            };
                            let value = *array_get(values, index)?;
                            frame.stack.push(JValue::Float(value));
                        }
                        op::LALOAD => {
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::LongArray(values)) =
                                self.heap.get(reference)
                            else {
                                return Err(malformed(String::from("laload on a non-long-array")));
                            };
                            let value = *array_get(values, index)?;
                            frame.stack.push(JValue::Long(value));
                        }
                        op::DALOAD => {
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::DoubleArray(values)) =
                                self.heap.get(reference)
                            else {
                                return Err(malformed(String::from(
                                    "daload on a non-double-array",
                                )));
                            };
                            let value = *array_get(values, index)?;
                            frame.stack.push(JValue::Double(value));
                        }
                        op::AALOAD => {
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::RefArray(_, values)) =
                                self.heap.get(reference)
                            else {
                                return Err(malformed(String::from(
                                    "aaload on a non-reference-array",
                                )));
                            };
                            let value = *array_get(values, index)?;
                            frame.stack.push(value);
                        }
                        op::IASTORE | op::BASTORE | op::CASTORE | op::SASTORE => {
                            let value = frame.pop_int()?;
                            let (reference, index) = frame.pop_array_access()?;
                            match self.heap.get_mut(reference) {
                                Some(crate::value::HeapObject::IntArray(_, values)) => {
                                    let slot = array_get_mut(values, index)?;
                                    // castore masks to the element width
                                    // (JVMS); boolean arrays hold 0/1.
                                    *slot = if opcode == op::CASTORE {
                                        i32::from(value.cast_unsigned() as u16)
                                    } else {
                                        value
                                    };
                                }
                                Some(crate::value::HeapObject::ByteArray(values))
                                    if opcode == op::BASTORE =>
                                {
                                    #[allow(clippy::cast_possible_truncation)]
                                    {
                                        *array_get_mut(values, index)? = value as i8;
                                    }
                                }
                                Some(crate::value::HeapObject::ShortArray(values))
                                    if opcode == op::SASTORE =>
                                {
                                    #[allow(clippy::cast_possible_truncation)]
                                    {
                                        *array_get_mut(values, index)? = value as i16;
                                    }
                                }
                                _ => {
                                    return Err(malformed(String::from(
                                        "int-array store on a non-int-array",
                                    )));
                                }
                            }
                        }
                        op::FASTORE => {
                            let value = frame.pop_float()?;
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::FloatArray(values)) =
                                self.heap.get_mut(reference)
                            else {
                                return Err(malformed(String::from(
                                    "fastore on a non-float-array",
                                )));
                            };
                            *array_get_mut(values, index)? = value;
                        }
                        op::LASTORE => {
                            let value = frame.pop_long()?;
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::LongArray(values)) =
                                self.heap.get_mut(reference)
                            else {
                                return Err(malformed(String::from("lastore on a non-long-array")));
                            };
                            *array_get_mut(values, index)? = value;
                        }
                        op::DASTORE => {
                            let value = frame.pop_double()?;
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::DoubleArray(values)) =
                                self.heap.get_mut(reference)
                            else {
                                return Err(malformed(String::from(
                                    "dastore on a non-double-array",
                                )));
                            };
                            *array_get_mut(values, index)? = value;
                        }
                        op::AASTORE => {
                            let value = frame.pop()?;
                            let (reference, index) = frame.pop_array_access()?;
                            let Some(crate::value::HeapObject::RefArray(_, values)) =
                                self.heap.get_mut(reference)
                            else {
                                return Err(malformed(String::from(
                                    "aastore on a non-reference-array",
                                )));
                            };
                            *array_get_mut(values, index)? = value;
                        }

                        op::RETURN => return Ok(Flow::Return(None)),
                        op::IRETURN | op::DRETURN | op::ARETURN | op::LRETURN | op::FRETURN => {
                            let value = frame.pop()?;
                            return Ok(Flow::Return(Some(value)));
                        }
                        other => return Err(VmError::UnsupportedOpcode(other)),
                    }
                    Ok(Flow::Next)
                })();

                match flow {
                    Ok(Flow::Next) => {}
                    Ok(Flow::Call(callee)) => {
                        frame.pc = pc;
                        self.frames.push(std::mem::replace(&mut frame, callee));
                        continue 'frames;
                    }
                    Ok(Flow::InitChain(mut chain)) => {
                        // `frame.pc` was rewound by the arm; the chain
                        // runs ancestor-first as pseudo-callers.
                        let first = chain.remove(0);
                        self.frames.push(std::mem::replace(&mut frame, first));
                        for pending in chain.into_iter().rev() {
                            self.frames.push(pending);
                        }
                        continue 'frames;
                    }
                    Ok(Flow::Return(value)) => {
                        let box_return_as = frame.box_return_as.take();
                        match self.frames.pop() {
                            None => return Ok(value),
                            Some(caller) => {
                                frame = caller;
                                if let Some(mut value) = value {
                                    if let Some(wrapper) = box_return_as {
                                        value = self.box_if_primitive(value, &wrapper);
                                    }
                                    frame.stack.push(value);
                                } else if box_return_as.is_some() {
                                    // A reflective `invoke` of a void method
                                    // still hands back one value (`null`).
                                    frame.stack.push(JValue::NULL);
                                }
                                continue 'frames;
                            }
                        }
                    }
                    Err(error) => {
                        // Capture the trace at the throw point — the
                        // handler search below unwinds (consumes)
                        // frames, so this must come first. Handlers
                        // only read the first line, so the trace rides
                        // along harmlessly when caught.
                        let error = match error {
                            VmError::UncaughtException(message) => {
                                VmError::UncaughtException(self.attach_stack_trace(message))
                            }
                            other => other,
                        };
                        if self.unwind_to_handler(&mut frame, addr, &error)? {
                            continue 'frames;
                        }
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Search the active frame, then each suspended caller, for an
    /// exception handler covering the faulting pc whose catch type
    /// matches the thrown class. On a match: unwind to that frame,
    /// clear its operand stack, push the exception object, and point
    /// `frame.pc` at the handler. Returns whether a handler was found
    /// (`false` leaves non-Java errors untouched).
    fn unwind_to_handler(
        &mut self,
        frame: &mut Frame<'run>,
        addr: usize,
        error: &VmError,
    ) -> Result<bool, VmError> {
        // Only Java exceptions unwind; host errors (budget, stop,
        // malformed classes) are not catchable.
        let VmError::UncaughtException(text) = error else {
            return Ok(false);
        };
        let first_line = text.lines().next().unwrap_or_default();
        let (dotted, message) = match first_line.split_once(": ") {
            Some((class, message)) => (class, Some(message.to_owned())),
            None => (first_line, None),
        };
        let internal = dotted.replace('.', "/");
        let is_library = caturra_classfile::exceptions::is_exception_class(&internal);
        if !is_library && !self.instance_is_throwable(dotted) {
            return Ok(false);
        }

        // A user exception keeps its thrown object (identity, fields);
        // library throws materialize one at the catch.
        let thrown_object = if is_library {
            None
        } else {
            self.last_thrown.take()
        };

        // The active frame first, at the faulting instruction.
        let mut search_pc = addr;
        loop {
            if let Some(handler_pc) = self.handler_for(frame, search_pc, dotted) {
                let exception = thrown_object.unwrap_or_else(|| {
                    self.heap.alloc(crate::value::HeapObject::Exception {
                        class_name: dotted.to_owned(),
                        message: message.clone(),
                    })
                });
                frame.stack.clear();
                frame.stack.push(JValue::Ref(Some(exception)));
                frame.pc = handler_pc;
                return Ok(true);
            }
            match self.frames.pop() {
                Some(caller) => {
                    // The caller's saved pc points just past its invoke
                    // instruction; step back inside it so try ranges
                    // covering the call match.
                    search_pc = caller.pc.saturating_sub(1);
                    *frame = caller;
                }
                None => return Ok(false),
            }
        }
    }

    /// `getstatic` (kept out of the dispatch loop so its temporaries
    /// don't inflate every interpreter frame in debug builds).
    #[inline(never)]
    fn getstatic_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (field_class, field_name, _descriptor) = class
            .constant_pool
            .get_member_ref(index)
            .map(|(c, f, d)| (c.to_owned(), f.to_owned(), d.to_owned()))
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        // The dispatch loop ran the initialization check already.
        let value = if self.classes.contains_key(&field_class) {
            *self
                .statics
                .get(&field_class)
                .and_then(|fields| fields.get(&field_name))
                .ok_or_else(|| {
                    malformed(format!("unknown static field {field_class}.{field_name}"))
                })?
        } else {
            self.intrinsic_statics
                .static_field(&mut self.heap, &field_class, &field_name)
                .ok_or_else(|| VmError::UnknownIntrinsic(format!("{field_class}.{field_name}")))?
        };
        frame.stack.push(value);
        Ok(())
    }

    #[inline(never)]
    fn putstatic_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (field_class, field_name, _descriptor) = class
            .constant_pool
            .get_member_ref(index)
            .map(|(c, f, d)| (c.to_owned(), f.to_owned(), d.to_owned()))
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        if !self.classes.contains_key(&field_class) {
            return Err(VmError::UnknownIntrinsic(format!(
                "{field_class}.{field_name}"
            )));
        }
        // The dispatch loop ran the initialization check already.
        let value = frame.pop()?;
        let slot = self
            .statics
            .get_mut(&field_class)
            .and_then(|fields| fields.get_mut(&field_name))
            .ok_or_else(|| malformed(format!("unknown static field {field_class}.{field_name}")))?;
        *slot = value;
        Ok(())
    }

    #[inline(never)]
    /// The heap key for `owner.name`, as JVMS field resolution finds it:
    /// the owner itself, then its superclasses. The compiler always names the
    /// declaring class, so the first probe normally hits. A bare name is the
    /// fallback for the VM's own instances (a throwable's `__message`).
    fn resolve_field_key(&self, owner: &str, name: &str, reference: HeapRef) -> Option<String> {
        let Some(crate::value::HeapObject::Instance { fields, .. }) = self.heap.get(reference)
        else {
            return None;
        };
        let mut current = Some(owner.to_owned());
        let mut steps = 0usize;
        while let Some(class_name) = current {
            steps += 1;
            if steps > self.classes.len() + 1 {
                break; // cycle guard
            }
            let key = instance_field_key(&class_name, name);
            if fields.contains_key(key.as_str()) {
                return Some(key);
            }
            current = self
                .classes
                .get(&class_name)
                .and_then(|class| class.constant_pool.get_class_name(class.super_class))
                .map(str::to_owned);
        }
        fields.contains_key(name).then(|| name.to_owned())
    }

    fn getfield_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let site = (std::ptr::from_ref(class) as usize, index);
        let reference = frame.pop_ref()?.ok_or_else(|| {
            let name = class
                .constant_pool
                .get_member_ref(index)
                .map_or("?", |(_, name, _)| name);
            VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot read field \"{name}\" \
                 because the object is null"
            ))
        })?;
        // Memoized key for this call site. A different receiver shape falls
        // through to the full resolution below, so semantics are unchanged.
        if let Some(key) = self.field_keys.get(&site)
            && let Some(crate::value::HeapObject::Instance { fields, .. }) =
                self.heap.get(reference)
            && let Some(value) = fields.get(&**key)
        {
            frame.stack.push(*value);
            return Ok(());
        }
        let (owner, field_name, _) = class
            .constant_pool
            .get_member_ref(index)
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        let (owner, field_name) = (owner.to_owned(), field_name.to_owned());
        let key = self
            .resolve_field_key(&owner, &field_name, reference)
            .ok_or_else(|| malformed(format!("unknown field {field_name}")))?;
        let Some(crate::value::HeapObject::Instance { fields, .. }) = self.heap.get(reference)
        else {
            return Err(malformed(String::from("getfield on a non-object")));
        };
        let value = *fields
            .get(key.as_str())
            .ok_or_else(|| malformed(format!("unknown field {field_name}")))?;
        self.field_keys.insert(site, Rc::from(key.as_str()));
        frame.stack.push(value);
        Ok(())
    }

    #[inline(never)]
    fn putfield_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let site = (std::ptr::from_ref(class) as usize, index);
        let value = frame.pop()?;
        let reference = frame.pop_ref()?.ok_or_else(|| {
            let name = class
                .constant_pool
                .get_member_ref(index)
                .map_or("?", |(_, name, _)| name);
            VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot assign field \"{name}\" \
                 because the object is null"
            ))
        })?;
        // Memoized key for this call site; assigning in place also avoids
        // re-allocating the key on every write.
        if let Some(key) = self.field_keys.get(&site)
            && let Some(crate::value::HeapObject::Instance { fields, .. }) =
                self.heap.get_mut(reference)
            && let Some(slot) = fields.get_mut(&**key)
        {
            *slot = value;
            return Ok(());
        }
        let (owner, field_name, _) = class
            .constant_pool
            .get_member_ref(index)
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        let (owner, field_name) = (owner.to_owned(), field_name.to_owned());
        let key = self
            .resolve_field_key(&owner, &field_name, reference)
            .unwrap_or_else(|| instance_field_key(&owner, &field_name));
        let Some(crate::value::HeapObject::Instance { fields, .. }) = self.heap.get_mut(reference)
        else {
            return Err(malformed(String::from("putfield on a non-object")));
        };
        let key: Rc<str> = Rc::from(key.as_str());
        fields.insert(Rc::clone(&key), value);
        self.field_keys.insert(site, key);
        Ok(())
    }

    /// `invokespecial`: user constructors and `super.method(...)`
    /// (returned as a frame to push) or intrinsic `<init>`s (handled
    /// inline, `None`).
    #[inline(never)]
    #[allow(clippy::too_many_lines)] // user ctors + several intrinsic <init> forms
    fn invoke_special_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame<'run>,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<Option<Frame<'run>>, VmError> {
        let (target_class, method_name, descriptor) = class
            .constant_pool
            .get_member_ref(index)
            .map(|(c, m, d)| (c.to_owned(), m.to_owned(), d.to_owned()))
            .ok_or_else(|| malformed(format!("bad method ref at pool {index}")))?;
        let widths = descriptor_arg_widths(&descriptor)
            .ok_or_else(|| malformed(format!("bad descriptor {descriptor}")))?;
        let mut args = Vec::with_capacity(widths.len());
        for _ in &widths {
            args.push(frame.pop()?);
        }
        args.reverse();
        let Some(receiver) = frame.pop_ref()? else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        // `new HashSet<>(collection)` copies the elements, deduplicating — which
        // may run a user `equals`/`hashCode`, so it belongs here rather than in
        // the heap-only intrinsic layer. Java pre-sizes the backing map to
        // `max((int)(size/.75f)+1, 16)`, so a large source lands on a bigger
        // table and iterates accordingly.
        if target_class == "java/util/HashSet" && descriptor == "(Ljava/util/Collection;)V" {
            use crate::value::HeapObject;
            let JValue::Ref(Some(source)) = args[0] else {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.NullPointerException",
                )));
            };
            let elements = self.collection_elements(source);
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let hint = std::cmp::max((elements.len() as f32 / 0.75) as i32 + 1, 16);
            if let Some(HeapObject::HashSet(map)) = self.heap.get_mut(receiver) {
                *map = crate::map::JavaHashMap::with_capacity_hint(hint);
            }
            for element in elements {
                self.set_add(receiver, element)?;
            }
            return Ok(None);
        }
        // `new LinkedList<>(collection)` copies its elements in order — from any
        // list, set, or map view, so it belongs here rather than in the
        // list-only intrinsic layer.
        if target_class == "java/util/LinkedList" && descriptor == "(Ljava/util/Collection;)V" {
            use crate::value::HeapObject;
            let JValue::Ref(Some(source)) = args[0] else {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.NullPointerException",
                )));
            };
            let elements = self.collection_elements(source);
            if let Some(HeapObject::LinkedList(values)) = self.heap.get_mut(receiver) {
                *values = elements;
            }
            return Ok(None);
        }
        // `new ArrayDeque<>(collection)` copies its elements in order, rejecting
        // null (as every ArrayDeque insertion does); `new ArrayDeque<>(int)` is
        // a capacity hint, leaving the empty deque `new_collection` built.
        if target_class == "java/util/ArrayDeque" {
            use crate::value::HeapObject;
            if descriptor == "(I)V" {
                return Ok(None);
            }
            if descriptor == "(Ljava/util/Collection;)V" {
                let JValue::Ref(Some(source)) = args[0] else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                let elements = self.collection_elements(source);
                if elements.iter().any(|v| matches!(v, JValue::Ref(None))) {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                }
                if let Some(HeapObject::ArrayDeque(values)) = self.heap.get_mut(receiver) {
                    *values = elements;
                }
                return Ok(None);
            }
        }
        // `new TreeSet<>(comparator)` stores the comparator; `new TreeSet<>(c)`
        // copies and sorts a collection. Both compare with user code, so they
        // belong here.
        if target_class == "java/util/TreeSet" {
            use crate::value::HeapObject;
            if descriptor == "(Ljava/util/Comparator;)V" {
                let comparator = match args[0] {
                    JValue::Ref(r) => r,
                    _ => None,
                };
                if let Some(HeapObject::TreeSet {
                    comparator: slot, ..
                }) = self.heap.get_mut(receiver)
                {
                    *slot = comparator;
                }
                return Ok(None);
            }
            if descriptor == "(Ljava/util/Collection;)V" || descriptor == "(Ljava/util/SortedSet;)V"
            {
                let JValue::Ref(Some(source)) = args[0] else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                // A source TreeSet passes its comparator on; otherwise natural.
                let comparator = self.tree_set_comparator(source);
                if let Some(HeapObject::TreeSet {
                    comparator: slot, ..
                }) = self.heap.get_mut(receiver)
                {
                    *slot = comparator;
                }
                for element in self.collection_elements(source) {
                    self.tree_set_add(receiver, element)?;
                }
                return Ok(None);
            }
        }
        // `new TreeMap<>(comparator)` stores the comparator; `new TreeMap<>(m)`
        // copies and sorts a map. Both compare keys with user code.
        if target_class == "java/util/TreeMap" {
            use crate::value::HeapObject;
            if descriptor == "(Ljava/util/Comparator;)V" {
                let comparator = match args[0] {
                    JValue::Ref(r) => r,
                    _ => None,
                };
                if let Some(HeapObject::TreeMap {
                    comparator: slot, ..
                }) = self.heap.get_mut(receiver)
                {
                    *slot = comparator;
                }
                return Ok(None);
            }
            if descriptor == "(Ljava/util/Map;)V" || descriptor == "(Ljava/util/SortedMap;)V" {
                let JValue::Ref(Some(source)) = args[0] else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                let comparator = self.tree_map_comparator(source);
                if let Some(HeapObject::TreeMap {
                    comparator: slot, ..
                }) = self.heap.get_mut(receiver)
                {
                    *slot = comparator;
                }
                for (key, value) in self.map_entries(source) {
                    self.map_put(receiver, key, value)?;
                }
                return Ok(None);
            }
        }
        // `new PriorityQueue<>(...)`: a comparator (with or without a capacity),
        // or a collection to heapify. All compare with user code.
        if target_class == "java/util/PriorityQueue" {
            use crate::value::HeapObject;
            let store_comparator = |vm: &mut Self, value: JValue| {
                let comparator = match value {
                    JValue::Ref(r) => r,
                    _ => None,
                };
                if let Some(HeapObject::PriorityQueue {
                    comparator: slot, ..
                }) = vm.heap.get_mut(receiver)
                {
                    *slot = comparator;
                }
            };
            if descriptor == "(Ljava/util/Comparator;)V" {
                store_comparator(self, args[0]);
                return Ok(None);
            }
            if descriptor == "(ILjava/util/Comparator;)V" {
                // args[0] is the capacity hint (no observable effect here).
                store_comparator(self, args[1]);
                return Ok(None);
            }
            if descriptor == "(Ljava/util/Collection;)V" {
                let JValue::Ref(Some(source)) = args[0] else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                // A PriorityQueue or SortedSet source passes its comparator on,
                // and its elements are already in valid heap order (a sorted
                // sequence is a min-heap), so no re-heapify is needed.
                let source_kind = self.heap.get(source);
                let already_heaped = matches!(
                    source_kind,
                    Some(HeapObject::PriorityQueue { .. } | HeapObject::TreeSet { .. })
                );
                let comparator = self
                    .pq_comparator(source)
                    .or_else(|| self.tree_set_comparator(source));
                if let Some(HeapObject::PriorityQueue {
                    comparator: slot, ..
                }) = self.heap.get_mut(receiver)
                {
                    *slot = comparator;
                }
                let elements = self.collection_elements(source);
                self.set_pq_heap(receiver, elements);
                if !already_heaped {
                    self.pq_heapify(receiver)?;
                }
                return Ok(None);
            }
        }
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        if classes.contains_key(&target_class) {
            // A user constructor or a `super.method(...)` call; the
            // method may be inherited, so walk the chain from the
            // named class.
            let mut found: Option<(&'run ClassFile, &'run MethodInfo)> = None;
            let mut current = classes.get(&target_class);
            let mut steps = 0usize;
            while let Some(candidate) = current {
                steps += 1;
                if steps > classes.len() + 1 {
                    break;
                }
                if let Some(method) = candidate.methods.iter().find(|m| {
                    !m.access_flags
                        .contains(caturra_classfile::MethodAccessFlags::ABSTRACT)
                        && candidate.constant_pool.get_utf8(m.name_index)
                            == Some(method_name.as_str())
                        && candidate.constant_pool.get_utf8(m.descriptor_index)
                            == Some(descriptor.as_str())
                }) {
                    found = Some((candidate, method));
                    break;
                }
                current = candidate
                    .constant_pool
                    .get_class_name(candidate.super_class)
                    .and_then(|super_name| classes.get(super_name));
            }
            let (target, ctor) = found.ok_or_else(|| {
                malformed(format!(
                    "no method {method_name}{descriptor} in {target_class} or its superclasses"
                ))
            })?;
            let mut locals = Vec::with_capacity(1 + args.len());
            locals.push(JValue::Ref(Some(receiver)));
            for (value, width) in args.iter().zip(&widths) {
                locals.push(*value);
                if *width == 2 {
                    locals.push(JValue::Int(0));
                }
            }
            // Constructors and `super.method(...)` calls both come
            // through here; any return value flows back through the
            // return opcodes when the pushed frame pops.
            return Ok(Some(self.make_frame(target, ctor, locals)?));
        }
        intrinsics::invoke_special(
            &mut self.heap,
            self.vfs,
            receiver,
            &target_class,
            &method_name,
            &descriptor,
            &args,
        )?;
        Ok(None)
    }

    /// Begin a user class's static initialization on first use
    /// (JVMS §5.5). Marks the class (and any uninitialized
    /// superclasses) started, installs default static field values, and
    /// returns the `<clinit>` frames to run in order — superclass
    /// first — as pseudo-caller frames. Returns `None` when the class
    /// is already initialized (or isn't a user class); `Some(vec![])`
    /// when initialization completed with no `<clinit>` bodies to run.
    fn begin_initialization(
        &mut self,
        class_name: &str,
    ) -> Result<Option<Vec<Frame<'run>>>, VmError> {
        if self.init_started.contains(class_name) || !self.classes.contains_key(class_name) {
            return Ok(None);
        }

        // Collect the uninitialized chain, subclass → ancestor.
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let mut chain: Vec<&'run ClassFile> = Vec::new();
        let mut current = classes.get(class_name);
        while let Some(class) = current {
            let name = class.class_name().unwrap_or_default();
            if self.init_started.contains(name) {
                break;
            }
            self.init_started.insert(name.to_owned());
            chain.push(class);

            let mut fields = HashMap::new();
            for field in &class.fields {
                if !field
                    .access_flags
                    .contains(caturra_classfile::FieldAccessFlags::STATIC)
                {
                    continue;
                }
                let field_name = class
                    .constant_pool
                    .get_utf8(field.name_index)
                    .unwrap_or_default()
                    .to_owned();
                let descriptor = class
                    .constant_pool
                    .get_utf8(field.descriptor_index)
                    .unwrap_or_default();
                fields.insert(field_name, default_for_descriptor(descriptor));
            }
            self.statics.insert(name.to_owned(), fields);

            current = class
                .constant_pool
                .get_class_name(class.super_class)
                .filter(|s| *s != "java/lang/Object")
                .and_then(|super_name| classes.get(super_name));
        }

        // <clinit> frames in execution order: ancestor-most first.
        let mut frames = Vec::new();
        for class in chain.into_iter().rev() {
            if let Some(clinit) = find_static_method(class, "<clinit>", "()V") {
                frames.push(self.make_frame(class, clinit, Vec::new())?);
            }
        }
        Ok(Some(frames))
    }

    /// Allocate an instance of a user class with defaulted fields,
    /// including fields inherited from superclasses.
    ///
    /// The defaulted field map is built once per class and cached: rebuilding it
    /// meant walking the superclass chain and formatting a `Declaring.name` key
    /// for every field on *every* `new`, which made allocation the most expensive
    /// operation in the VM. Cloning the template instead copies `Rc` keys, so a
    /// pixel-filter program allocating hundreds of thousands of objects pays a
    /// refcount bump per field rather than a string allocation.
    fn new_instance(&mut self, class_name: &str) -> crate::value::HeapObject {
        if let Some(template) = self.field_templates.get(class_name) {
            return crate::value::HeapObject::Instance {
                class_name: class_name.to_owned(),
                fields: template.clone(),
            };
        }
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let mut fields: HashMap<Rc<str>, JValue> = HashMap::new();
        let mut current = classes.get(class_name);
        let mut steps = 0usize;
        while let Some(class) = current {
            steps += 1;
            if steps > classes.len() + 1 {
                break; // cycle guard (compiler rejects these anyway)
            }
            for field in &class.fields {
                if field
                    .access_flags
                    .contains(caturra_classfile::FieldAccessFlags::STATIC)
                {
                    continue;
                }
                let name = class
                    .constant_pool
                    .get_utf8(field.name_index)
                    .unwrap_or_default();
                let descriptor = class
                    .constant_pool
                    .get_utf8(field.descriptor_index)
                    .unwrap_or_default();
                // Keyed by the DECLARING class: a subclass may hide a
                // superclass field of the same name, and the two are
                // distinct slots. `or_insert` would have collapsed them.
                let key = instance_field_key(class_name_of(class), name);
                fields.insert(Rc::from(key.as_str()), default_for_descriptor(descriptor));
            }
            current = class
                .constant_pool
                .get_class_name(class.super_class)
                .and_then(|super_name| classes.get(super_name));
        }
        self.field_templates
            .insert(class_name.to_owned(), fields.clone());
        crate::value::HeapObject::Instance {
            class_name: class_name.to_owned(),
            fields,
        }
    }

    /// Find a handler in `frame`'s exception table covering `pc` for
    /// the thrown class (dotted or user name). Entry order is
    /// significant (first match wins, like the JVMS).
    fn handler_for(&self, frame: &Frame<'_>, pc: usize, thrown_dotted: &str) -> Option<usize> {
        let pool = &frame.class.constant_pool;
        for entry in &frame.code.attr.exception_table {
            let start = usize::from(entry.start_pc);
            let end = usize::from(entry.end_pc);
            if pc < start || pc >= end {
                continue;
            }
            // catch_type 0 catches everything (finally-style).
            let matches = entry.catch_type == 0
                || pool
                    .get_class_name(entry.catch_type)
                    .is_some_and(|catch| self.thrown_matches(thrown_dotted, catch));
            if matches {
                return Some(usize::from(entry.handler_pc));
            }
        }
        None
    }

    /// Whether a user class descends from a library throwable (its
    /// class-file superclass chain reaches the exceptions table).
    fn instance_is_throwable(&self, class_name: &str) -> bool {
        let mut current = self.classes.get(class_name);
        let mut steps = 0usize;
        while let Some(class) = current {
            steps += 1;
            if steps > self.classes.len() + 1 {
                return false;
            }
            let Some(super_name) = class.constant_pool.get_class_name(class.super_class) else {
                return false;
            };
            if super_name.contains('/') {
                return caturra_classfile::exceptions::is_exception_class(super_name);
            }
            current = self.classes.get(super_name);
        }
        false
    }

    /// Exception-catch matching across user and library classes:
    /// whether the thrown class (dotted library or simple user name)
    /// is the catch type or a subclass of it.
    fn thrown_matches(&self, thrown_dotted: &str, catch_name: &str) -> bool {
        let thrown_internal = thrown_dotted.replace('.', "/");
        // Library-thrown: pure table walk (a library class is never
        // under a user class).
        if caturra_classfile::exceptions::is_exception_class(&thrown_internal) {
            return caturra_classfile::exceptions::is_exception_subclass(
                &thrown_internal,
                catch_name,
            );
        }
        // User-thrown: walk the user chain, crossing into the library
        // table at the first java/... superclass.
        let mut current_name = thrown_dotted.to_owned();
        let mut steps = 0usize;
        loop {
            steps += 1;
            if steps > self.classes.len() + 2 {
                return false;
            }
            if current_name == catch_name {
                return true;
            }
            let Some(class) = self.classes.get(&current_name) else {
                return caturra_classfile::exceptions::is_exception_subclass(
                    &current_name,
                    catch_name,
                );
            };
            match class.constant_pool.get_class_name(class.super_class) {
                Some(super_name) => current_name = super_name.to_owned(),
                None => return false,
            }
        }
    }

    /// Whether `sub` (a user class name) is `sup` or inherits from it,
    /// walking `extends` and `implements` edges.
    fn is_runtime_subtype(&self, sub: &str, sup: &str) -> bool {
        if sub == sup {
            return true;
        }
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let mut stack = vec![sub.to_owned()];
        let mut steps = 0usize;
        while let Some(name) = stack.pop() {
            steps += 1;
            if steps > classes.len() * 4 + 4 {
                return false;
            }
            if name == sup {
                return true;
            }
            if let Some(class) = classes.get(&name) {
                if let Some(parent) = class.constant_pool.get_class_name(class.super_class)
                    && parent != "java/lang/Object"
                {
                    stack.push(parent.to_owned());
                }
                for interface in &class.interfaces {
                    if let Some(iface) = class.constant_pool.get_class_name(*interface) {
                        stack.push(iface.to_owned());
                    }
                }
            }
        }
        false
    }

    /// Run `frame` to completion, isolated from the frames already suspended.
    ///
    /// Native code sometimes has to make a real Java call: rendering a value
    /// as text calls its `toString()`. The interpreter keeps frames as data
    /// rather than host-stack recursion, so a nested call means running a
    /// second, private frame stack to completion and putting the old one
    /// back. An exception that escapes the nested call surfaces as an `Err`,
    /// which the outer frames' handlers then see as usual.
    fn run_nested(&mut self, frame: Frame<'run>) -> Result<Option<JValue>, VmError> {
        let location = self.current_location.take();
        let suspended = std::mem::take(&mut self.frames);
        // The suspended frames plus the caller that is running natively right
        // now: a nested call is that much deeper, even though `frames` starts
        // empty so that an exception cannot unwind past the native call.
        let base = self.nested_frame_base + suspended.len() + 1;
        let outer_base = std::mem::replace(&mut self.nested_frame_base, base);
        self.suspended_runs.push((location, suspended));

        let result = self.run_loop(frame);

        let (location, suspended) = self.suspended_runs.pop().expect("pushed above");
        self.nested_frame_base = outer_base;
        self.frames = suspended;
        self.current_location = location;
        result
    }

    /// How deep the Java call stack is, counting frames suspended beneath any
    /// nested run.
    fn call_depth(&self) -> usize {
        self.nested_frame_base + self.frames.len()
    }

    /// `obj.toString()`, dispatched virtually. Falls back to Java's default
    /// `Class@hash` when neither the class nor its ancestors declare one.
    fn instance_to_string(
        &mut self,
        receiver: HeapRef,
        class_name: &str,
    ) -> Result<String, VmError> {
        let returned = match self.user_virtual_dispatch(
            receiver,
            class_name,
            "toString",
            "()Ljava/lang/String;",
            &[],
        )? {
            UserDispatch::Call(frame) => self.run_nested(frame)?,
            UserDispatch::Value(value) => value,
        };
        Ok(match returned {
            Some(JValue::Ref(Some(text))) => self.heap.string_text(text).unwrap_or_default(),
            // A `toString()` returning null renders as "null" rather than
            // throwing, the way `String.valueOf(Object)` does.
            _ => String::from("null"),
        })
    }

    /// `String.valueOf(value)`: the text Java would produce. Unlike the
    /// intrinsic renderers, which see only the heap, this calls a user
    /// `toString()` — and so renders a list or a map by rendering its
    /// elements, exactly as `AbstractCollection` and `AbstractMap` do.
    #[allow(clippy::too_many_lines)] // one arm per renderable container kind
    fn string_value_of(&mut self, value: JValue, depth: u32) -> Result<String, VmError> {
        use crate::value::HeapObject;
        // An indirect cycle (a list inside a list that holds it) overflows a
        // real JVM's stack. It must not overflow the host's.
        if depth > MAX_RENDER_DEPTH {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.StackOverflowError",
            )));
        }
        let reference = match value {
            JValue::Int(v) => return Ok(v.to_string()),
            JValue::Long(v) => return Ok(v.to_string()),
            JValue::Float(v) => return Ok(intrinsics::java_float_to_string(v)),
            JValue::Double(v) => return Ok(intrinsics::java_double_to_string(v)),
            JValue::Ref(None) => return Ok(String::from("null")),
            JValue::Ref(Some(reference)) => reference,
        };
        // An unmodifiable set/map view renders exactly as its backing, so
        // unwrap to it first.
        let reference = match self.heap.get(reference) {
            Some(HeapObject::UnmodifiableSet(inner) | HeapObject::UnmodifiableMap(inner)) => *inner,
            _ => reference,
        };
        // Copy out what we need before calling back into Java, which may
        // allocate and so cannot hold a borrow of the heap.
        let renderable = match self.heap.get(reference) {
            Some(HeapObject::Instance { class_name, .. }) => {
                Renderable::Instance(class_name.clone())
            }
            Some(
                HeapObject::ArrayList(items)
                | HeapObject::LinkedList(items)
                | HeapObject::ArrayDeque(items)
                | HeapObject::Stack(items),
            ) => Renderable::List(items.clone()),
            Some(HeapObject::UnmodifiableList(_)) => Renderable::List(self.list_items(reference)),
            Some(HeapObject::HashMap(map)) => Renderable::Map(map.entries_in_order()),
            // A TreeMap prints `{k=v, ...}` with its entries already key-sorted.
            Some(HeapObject::TreeMap { entries, .. }) => Renderable::Map(entries.clone()),
            // A set prints as `[a, b]` — its elements are the backing map's
            // keys, so the keys-view renderer produces exactly that.
            Some(HeapObject::HashSet(map)) => {
                Renderable::View(map.entries_in_order(), MapViewKind::Keys)
            }
            // A TreeSet prints its (already sorted) elements as `[a, b]`.
            Some(HeapObject::TreeSet { values, .. }) => Renderable::List(values.clone()),
            // A PriorityQueue prints its heap-array order (as Java's does).
            Some(HeapObject::PriorityQueue { heap, .. }) => Renderable::List(heap.clone()),
            Some(HeapObject::Optional { value, kind }) => Renderable::Optional(*value, *kind),
            Some(HeapObject::MapView { map, kind }) => {
                let (map, kind) = (*map, *kind);
                // A view over any map (Hash or Tree) renders its entries in that
                // map's iteration order.
                if matches!(
                    self.heap.get(map),
                    Some(HeapObject::HashMap(_) | HeapObject::TreeMap { .. })
                ) {
                    Renderable::View(self.map_entries(map), kind)
                } else {
                    Renderable::Opaque
                }
            }
            Some(HeapObject::MapEntry { map, key }) => Renderable::Entry(*map, *key),
            _ => Renderable::Opaque,
        };

        match renderable {
            Renderable::Instance(class_name) => self.instance_to_string(reference, &class_name),
            Renderable::List(items) => {
                let mut parts = Vec::with_capacity(items.len());
                for item in items {
                    parts.push(self.render_element(item, reference, depth, THIS_COLLECTION)?);
                }
                Ok(format!("[{}]", parts.join(", ")))
            }
            Renderable::Map(entries) => {
                let mut parts = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    parts.push(format!(
                        "{}={}",
                        self.render_element(key, reference, depth, THIS_MAP)?,
                        self.render_element(value, reference, depth, THIS_MAP)?
                    ));
                }
                Ok(format!("{{{}}}", parts.join(", ")))
            }
            Renderable::View(entries, kind) => {
                let mut parts = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    parts.push(match kind {
                        MapViewKind::Keys => {
                            self.render_element(key, reference, depth, THIS_COLLECTION)?
                        }
                        MapViewKind::Values => {
                            self.render_element(value, reference, depth, THIS_COLLECTION)?
                        }
                        MapViewKind::Entries => format!(
                            "{}={}",
                            self.render_element(key, reference, depth, THIS_COLLECTION)?,
                            self.render_element(value, reference, depth, THIS_COLLECTION)?
                        ),
                    });
                }
                Ok(format!("[{}]", parts.join(", ")))
            }
            Renderable::Entry(map, key) => {
                let value = self.map_entry_value(map, key)?;
                Ok(format!(
                    "{}={}",
                    self.render_element(key, reference, depth, THIS_MAP)?,
                    self.render_element(value, reference, depth, THIS_MAP)?
                ))
            }
            Renderable::Optional(inner, kind) => Ok(match inner {
                Some(present) => {
                    format!(
                        "{}[{}]",
                        kind.prefix(),
                        self.string_value_of(present, depth + 1)?
                    )
                }
                None => format!("{}.empty", kind.prefix()),
            }),
            Renderable::Opaque => Ok(intrinsics::object_display(&self.heap, value)),
        }
    }

    /// One element of a container. A container holding itself renders as
    /// `(this Collection)` instead of recursing, exactly as Java's does.
    fn render_element(
        &mut self,
        item: JValue,
        owner: HeapRef,
        depth: u32,
        self_text: &str,
    ) -> Result<String, VmError> {
        if item == JValue::Ref(Some(owner)) {
            return Ok(self_text.to_owned());
        }
        self.string_value_of(item, depth + 1)
    }

    /// The `java.util.Collections` helpers the VM answers rather than the
    /// bundled Java: a list stores unboxed primitives, so bundled Java could
    /// not compare them (`get` yields a bare `int`, not a `Comparable`), and
    /// `max`/`min` must hand back the element's own type. Returns whether it
    /// answered.
    #[allow(clippy::too_many_lines)] // one method per arm
    fn collections_static_intrinsic(
        &mut self,
        frame: &mut Frame<'run>,
        class_name: &str,
        method_name: &str,
        args: &[JValue],
    ) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        if class_name != "Collections" {
            return Ok(false);
        }
        // These two build a list rather than reading one.
        match (method_name, args) {
            ("nCopies", [JValue::Int(count), value]) => {
                let Ok(count) = usize::try_from(*count) else {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.IllegalArgumentException: List length = {count}"
                    )));
                };
                let list = self.heap.alloc(HeapObject::ArrayList(vec![*value; count]));
                frame.stack.push(JValue::Ref(Some(list)));
                return Ok(true);
            }
            ("emptyList", []) => {
                let empty = self.heap.alloc(HeapObject::ArrayList(Vec::new()));
                let view = self.heap.alloc(HeapObject::UnmodifiableList(empty));
                frame.stack.push(JValue::Ref(Some(view)));
                return Ok(true);
            }
            // `singletonList(e)` — an immutable one-element list.
            ("singletonList", [value]) => {
                let inner = self.heap.alloc(HeapObject::ArrayList(vec![*value]));
                let view = self.heap.alloc(HeapObject::UnmodifiableList(inner));
                frame.stack.push(JValue::Ref(Some(view)));
                return Ok(true);
            }
            // `reverseOrder()` reverses natural ordering; `reverseOrder(cmp)`
            // reverses a given comparator — both a `ComparatorSpec::Reversed`,
            // exactly as `Comparator.reverseOrder()`/`reversed()` build.
            ("reverseOrder", []) => {
                use crate::value::ComparatorSpec;
                let natural = self
                    .heap
                    .alloc(HeapObject::Comparator(ComparatorSpec::Natural));
                let reversed = self
                    .heap
                    .alloc(HeapObject::Comparator(ComparatorSpec::Reversed(natural)));
                frame.stack.push(JValue::Ref(Some(reversed)));
                return Ok(true);
            }
            ("reverseOrder", [JValue::Ref(Some(comparator))]) => {
                use crate::value::ComparatorSpec;
                let reversed = self
                    .heap
                    .alloc(HeapObject::Comparator(ComparatorSpec::Reversed(
                        *comparator,
                    )));
                frame.stack.push(JValue::Ref(Some(reversed)));
                return Ok(true);
            }
            // Immutable empty / single-element set and map views.
            ("emptySet", []) => {
                let inner = self
                    .heap
                    .alloc(HeapObject::HashSet(crate::map::JavaHashMap::new()));
                let view = self.heap.alloc(HeapObject::UnmodifiableSet(inner));
                frame.stack.push(JValue::Ref(Some(view)));
                return Ok(true);
            }
            ("emptyMap", []) => {
                let inner = self
                    .heap
                    .alloc(HeapObject::HashMap(crate::map::JavaHashMap::new()));
                let view = self.heap.alloc(HeapObject::UnmodifiableMap(inner));
                frame.stack.push(JValue::Ref(Some(view)));
                return Ok(true);
            }
            ("singleton", [element]) => {
                let inner = self
                    .heap
                    .alloc(HeapObject::HashSet(crate::map::JavaHashMap::new()));
                self.set_add(inner, *element)?;
                let view = self.heap.alloc(HeapObject::UnmodifiableSet(inner));
                frame.stack.push(JValue::Ref(Some(view)));
                return Ok(true);
            }
            ("singletonMap", [key, value]) => {
                let inner = self
                    .heap
                    .alloc(HeapObject::HashMap(crate::map::JavaHashMap::new()));
                self.map_put(inner, *key, *value)?;
                let view = self.heap.alloc(HeapObject::UnmodifiableMap(inner));
                frame.stack.push(JValue::Ref(Some(view)));
                return Ok(true);
            }
            _ => {}
        }

        let Some(JValue::Ref(Some(list))) = args.first().copied() else {
            return Ok(false);
        };
        if method_name == "unmodifiableList" {
            let view = self.heap.alloc(HeapObject::UnmodifiableList(list));
            frame.stack.push(JValue::Ref(Some(view)));
            return Ok(true);
        }
        if method_name == "unmodifiableSet" || method_name == "unmodifiableSortedSet" {
            let view = self.heap.alloc(HeapObject::UnmodifiableSet(list));
            frame.stack.push(JValue::Ref(Some(view)));
            return Ok(true);
        }
        if method_name == "unmodifiableMap" || method_name == "unmodifiableSortedMap" {
            let view = self.heap.alloc(HeapObject::UnmodifiableMap(list));
            frame.stack.push(JValue::Ref(Some(view)));
            return Ok(true);
        }
        // Everything below reads, or writes, the list itself.
        let unmodifiable = self.is_unmodifiable_list(list);
        let reference = self.backing_list(list);
        let items = self
            .heap
            .list_values(reference)
            .map(|items| (reference, items.clone()));
        match (method_name, args) {
            ("sort", [_] | [_, JValue::Ref(Some(_))]) => {
                let Some((reference, items)) = items else {
                    return Ok(true);
                };
                if unmodifiable {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.UnsupportedOperationException",
                    )));
                }
                // A second argument is the comparator; one argument sorts by
                // natural ordering.
                let comparator = match args {
                    [_, JValue::Ref(Some(comparator))] => Some(*comparator),
                    _ => None,
                };
                let sorted = self.merge_sort_by(items, comparator)?;
                if let Some(slot) = self.heap.list_values_mut(reference) {
                    *slot = sorted;
                }
            }
            // `Collections.max`/`min` keep the first of equal elements, because
            // they only replace the candidate on a strict improvement.
            ("max" | "min", [_]) => {
                let Some((_, items)) = items else {
                    return Ok(true);
                };
                let (first, rest) = items.split_first().ok_or_else(|| {
                    VmError::UncaughtException(String::from("java.util.NoSuchElementException"))
                })?;
                let mut candidate = *first;
                for item in rest {
                    let order = self.compare_for_sort(*item, candidate)?;
                    let better = if method_name == "max" {
                        order > 0
                    } else {
                        order < 0
                    };
                    if better {
                        candidate = *item;
                    }
                }
                frame.stack.push(candidate);
            }
            // `Collections.frequency` asks the probe, as `Objects.equals` does.
            ("frequency", [_, probe]) => {
                let Some((_, items)) = items else {
                    frame.stack.push(JValue::Int(0));
                    return Ok(true);
                };
                let mut count = 0i32;
                for item in items {
                    if self.java_equals(*probe, item)? {
                        count += 1;
                    }
                }
                frame.stack.push(JValue::Int(count));
            }
            // `Collections.binarySearch` over a sorted list, by the elements'
            // own `compareTo`.
            ("binarySearch", [_, key]) => {
                let Some((_, items)) = items else {
                    return Ok(true);
                };
                let (mut low, mut high) = (0usize, items.len());
                let mut found = None;
                while low < high {
                    let mid = low + (high - low) / 2;
                    match self.compare_for_sort(items[mid], *key)?.cmp(&0) {
                        std::cmp::Ordering::Less => low = mid + 1,
                        std::cmp::Ordering::Greater => high = mid,
                        std::cmp::Ordering::Equal => {
                            found = Some(mid);
                            break;
                        }
                    }
                }
                let index = match found {
                    Some(at) => i32::try_from(at).unwrap_or(i32::MAX),
                    None => -(i32::try_from(low).unwrap_or(i32::MAX) + 1),
                };
                frame.stack.push(JValue::Int(index));
            }
            // `Collections.addAll(list, elements...)`: the compiler packed the
            // varargs into an array of the list's element type.
            ("addAll", [_, JValue::Ref(Some(elements))]) => {
                if unmodifiable {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.UnsupportedOperationException",
                    )));
                }
                let elements = self.array_elements(*elements).ok_or_else(|| {
                    VmError::UnknownIntrinsic(String::from("addAll needs an array"))
                })?;
                let changed = !elements.is_empty();
                if let Some(slot) = self.heap.list_values_mut(reference) {
                    slot.extend(elements);
                }
                frame.stack.push(JValue::Int(i32::from(changed)));
            }
            // reverse/swap/shuffle are bundled Java, and mutate through
            // `set`, which an unmodifiable view already refuses.
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Every element of any array, as the values a list would store.
    fn array_elements(&self, reference: HeapRef) -> Option<Vec<JValue>> {
        use crate::value::HeapObject as Object;
        Some(match self.heap.get(reference)? {
            Object::IntArray(_, values) => values.iter().copied().map(JValue::Int).collect(),
            Object::DoubleArray(values) => values.iter().copied().map(JValue::Double).collect(),
            Object::LongArray(values) => values.iter().copied().map(JValue::Long).collect(),
            Object::FloatArray(values) => values.iter().copied().map(JValue::Float).collect(),
            Object::ShortArray(values) => {
                values.iter().map(|v| JValue::Int(i32::from(*v))).collect()
            }
            Object::ByteArray(values) => {
                values.iter().map(|v| JValue::Int(i32::from(*v))).collect()
            }
            Object::RefArray(_, values) => values.clone(),
            _ => return None,
        })
    }

    /// The `java.util.Arrays` methods the VM answers rather than the bundled
    /// Java: they need an array's element kind at run time, which only the
    /// heap knows once the static type is gone. Returns whether it answered.
    /// `java.util.stream.Collectors` factories, each producing a `Collector`
    /// recipe for `Stream.collect`. Returns whether it answered.
    /// `java.util.Comparator` static factories — each builds a native
    /// [`crate::value::ComparatorSpec`]. Returns whether it answered.
    #[allow(clippy::unnecessary_wraps)] // uniform with the other *_static_intrinsic
    fn comparator_static_intrinsic(
        &mut self,
        frame: &mut Frame<'run>,
        class_name: &str,
        method_name: &str,
        args: &[JValue],
    ) -> Result<bool, VmError> {
        use crate::value::{ComparatorSpec, HeapObject};
        if class_name != "Comparator" && class_name != "java/util/Comparator" {
            return Ok(false);
        }
        let spec = match (method_name, args) {
            ("naturalOrder", []) => ComparatorSpec::Natural,
            ("reverseOrder", []) => {
                let natural = self
                    .heap
                    .alloc(HeapObject::Comparator(ComparatorSpec::Natural));
                ComparatorSpec::Reversed(natural)
            }
            // `comparing`/`comparingInt`/`comparingDouble`/`comparingLong` — the
            // key is compared by natural ordering, so all four are the same here.
            (
                "comparing" | "comparingInt" | "comparingDouble" | "comparingLong",
                [JValue::Ref(Some(extractor))],
            ) => ComparatorSpec::ByKey(*extractor),
            _ => return Ok(false),
        };
        let comparator = self.heap.alloc(HeapObject::Comparator(spec));
        frame.stack.push(JValue::Ref(Some(comparator)));
        Ok(true)
    }

    #[allow(clippy::unnecessary_wraps)] // uniform with the other *_static_intrinsic
    fn collectors_static_intrinsic(
        &mut self,
        frame: &mut Frame<'run>,
        class_name: &str,
        method_name: &str,
        args: &[JValue],
    ) -> Result<bool, VmError> {
        use crate::value::{CollectorKind, HeapObject};
        if class_name != "Collectors" && class_name != "java/util/stream/Collectors" {
            return Ok(false);
        }
        let text = |vm: &Self, value: &JValue| match value {
            JValue::Ref(Some(r)) => vm.heap.string_text(*r).unwrap_or_default(),
            _ => String::new(),
        };
        let kind = match (method_name, args) {
            ("toList" | "toUnmodifiableList", []) => CollectorKind::ToList,
            ("toSet" | "toUnmodifiableSet", []) => CollectorKind::ToSet,
            ("joining", []) => CollectorKind::Joining {
                delimiter: String::new(),
                prefix: String::new(),
                suffix: String::new(),
            },
            ("joining", [delimiter]) => CollectorKind::Joining {
                delimiter: text(self, delimiter),
                prefix: String::new(),
                suffix: String::new(),
            },
            ("joining", [delimiter, prefix, suffix]) => CollectorKind::Joining {
                delimiter: text(self, delimiter),
                prefix: text(self, prefix),
                suffix: text(self, suffix),
            },
            _ => return Ok(false),
        };
        let collector = self.heap.alloc(HeapObject::Collector(kind));
        frame.stack.push(JValue::Ref(Some(collector)));
        Ok(true)
    }

    fn arrays_static_intrinsic(
        &mut self,
        frame: &mut Frame<'run>,
        class_name: &str,
        method_name: &str,
        args: &[JValue],
    ) -> Result<bool, VmError> {
        if class_name != "Arrays" {
            return Ok(false);
        }
        // `asList` builds a list from the varargs array the compiler packed,
        // keeping the elements as a list stores them: unboxed.
        if let ("asList", [JValue::Ref(Some(elements))]) = (method_name, args) {
            let items = self.array_elements(*elements).ok_or_else(|| {
                VmError::UnknownIntrinsic(String::from("Arrays.asList needs an array"))
            })?;
            let list = self.heap.alloc(crate::value::HeapObject::ArrayList(items));
            frame.stack.push(JValue::Ref(Some(list)));
            return Ok(true);
        }
        // `setAll(array, generator)` stores `generator.apply(i)` at each index i;
        // the array's own kind decides how each result is stored. Void.
        if let ("setAll", [JValue::Ref(Some(array)), JValue::Ref(Some(generator))]) =
            (method_name, args)
        {
            use crate::value::HeapObject;
            let (array, generator) = (*array, *generator);
            let len = match self.heap.get(array) {
                Some(HeapObject::IntArray(_, v)) => v.len(),
                Some(HeapObject::DoubleArray(v)) => v.len(),
                Some(HeapObject::RefArray(_, v)) => v.len(),
                _ => {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                }
            };
            for i in 0..len {
                let index = JValue::Int(i32::try_from(i).unwrap_or(i32::MAX));
                let value = self.call_apply(generator, index)?;
                match self.heap.get_mut(array) {
                    Some(HeapObject::IntArray(_, v)) => {
                        if let JValue::Int(n) = value {
                            v[i] = n;
                        }
                    }
                    Some(HeapObject::DoubleArray(v)) => {
                        v[i] = match value {
                            JValue::Double(d) => d,
                            JValue::Int(n) => f64::from(n),
                            JValue::Float(f) => f64::from(f),
                            _ => 0.0,
                        };
                    }
                    Some(HeapObject::RefArray(_, v)) => v[i] = value,
                    _ => {}
                }
            }
            return Ok(true);
        }
        // These three recurse into element arrays, so an element array's kind
        // decides how it renders and hashes, and the elements' own
        // toString/equals/hashCode may be the user's.
        match (method_name, args) {
            ("deepToString", [value]) => {
                let text = self.deep_to_string(*value, &mut Vec::new())?;
                let reference = self.heap.alloc_string(&text);
                frame.stack.push(JValue::Ref(Some(reference)));
                return Ok(true);
            }
            ("deepEquals", [ours, theirs]) => {
                let equal = self.deep_equals(*ours, *theirs, 0)?;
                frame.stack.push(JValue::Int(i32::from(equal)));
                return Ok(true);
            }
            ("deepHashCode", [value]) => {
                let hash = self.deep_hash_code(*value, 0)?;
                frame.stack.push(JValue::Int(hash));
                return Ok(true);
            }
            _ => {}
        }
        // `copyOf`/`copyOfRange`/`fill`/`binarySearch` are one method each in
        // the VM and nine in Java, because the heap already knows the array's
        // element kind — which `copyOf` must reproduce, and against which a
        // reference `binarySearch` compares with the element's `compareTo`.
        if let Some(result) = self.arrays_array_intrinsic(method_name, args)? {
            if let Some(value) = result {
                frame.stack.push(value);
            }
            return Ok(true);
        }
        Ok(false)
    }

    /// `Arrays.copyOf`/`copyOfRange`/`fill`/`binarySearch`. Returns `None`
    /// when `method` is none of those, `Some(None)` for the void `fill`.
    #[allow(clippy::option_option)] // "not mine" vs "no value"
    fn arrays_array_intrinsic(
        &mut self,
        method: &str,
        args: &[JValue],
    ) -> Result<Option<Option<JValue>>, VmError> {
        let source = |args: &[JValue]| -> Result<HeapRef, VmError> {
            match args.first() {
                Some(JValue::Ref(Some(reference))) => Ok(*reference),
                _ => Err(VmError::UncaughtException(String::from(
                    "java.lang.NullPointerException",
                ))),
            }
        };
        let result = match (method, args) {
            ("copyOf", [_, JValue::Int(new_length)]) => {
                let source = source(args)?;
                let Ok(new_length) = usize::try_from(*new_length) else {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.NegativeArraySizeException: {new_length}"
                    )));
                };
                Some(self.array_copy_of_range(source, 0, new_length)?)
            }
            ("copyOfRange", [_, JValue::Int(from), JValue::Int(to)]) => {
                let source = source(args)?;
                let length = self.array_length(source).unwrap_or(0);
                if from > to {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.IllegalArgumentException: {from} > {to}"
                    )));
                }
                let Ok(start) = usize::try_from(*from) else {
                    return Err(array_index_error(*from));
                };
                if start > length {
                    return Err(array_index_error(*from));
                }
                let new_length = usize::try_from(to - from).unwrap_or(0);
                Some(self.array_copy_of_range(source, start, new_length)?)
            }
            ("fill", [_, value]) => {
                let target = source(args)?;
                let length = self.array_length(target).unwrap_or(0);
                self.array_fill(target, 0, length, *value)?;
                None
            }
            ("fill", [_, JValue::Int(from), JValue::Int(to), value]) => {
                let target = source(args)?;
                let (from, to) = self.array_range(target, *from, *to)?;
                self.array_fill(target, from, to, *value)?;
                None
            }
            ("binarySearch", [_, key]) => {
                let target = source(args)?;
                let length = self.array_length(target).unwrap_or(0);
                Some(JValue::Int(
                    self.array_binary_search(target, 0, length, *key)?,
                ))
            }
            ("binarySearch", [_, JValue::Int(from), JValue::Int(to), key]) => {
                let target = source(args)?;
                let (from, to) = self.array_range(target, *from, *to)?;
                Some(JValue::Int(
                    self.array_binary_search(target, from, to, *key)?,
                ))
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    /// Java's `rangeCheck`, shared by the ranged `fill` and `binarySearch`.
    fn array_range(&self, target: HeapRef, from: i32, to: i32) -> Result<(usize, usize), VmError> {
        if from > to {
            return Err(VmError::UncaughtException(format!(
                "java.lang.IllegalArgumentException: fromIndex({from}) > toIndex({to})"
            )));
        }
        let length = self.array_length(target).unwrap_or(0);
        let start = usize::try_from(from).map_err(|_| array_index_error(from))?;
        let end = usize::try_from(to).map_err(|_| array_index_error(to))?;
        if end > length {
            return Err(array_index_error(to));
        }
        Ok((start, end))
    }

    /// `Arrays.copyOfRange(source, from, from + length)`: a fresh array of the
    /// same kind, padded with the element type's default wherever the range
    /// runs past the end. `Arrays.copyOf` is this with `from = 0`.
    fn array_copy_of_range(
        &mut self,
        source: HeapRef,
        from: usize,
        length: usize,
    ) -> Result<JValue, VmError> {
        use crate::value::HeapObject as Object;
        fn taken<T: Copy>(values: &[T], from: usize, length: usize, default: T) -> Vec<T> {
            let mut copy = vec![default; length];
            let available = values.len().saturating_sub(from).min(length);
            copy[..available].copy_from_slice(&values[from..from + available]);
            copy
        }
        let object = match self.heap.get(source) {
            Some(Object::IntArray(kind, values)) => {
                Object::IntArray(*kind, taken(values, from, length, 0))
            }
            Some(Object::DoubleArray(values)) => {
                Object::DoubleArray(taken(values, from, length, 0.0))
            }
            Some(Object::LongArray(values)) => Object::LongArray(taken(values, from, length, 0)),
            Some(Object::FloatArray(values)) => {
                Object::FloatArray(taken(values, from, length, 0.0))
            }
            Some(Object::ShortArray(values)) => Object::ShortArray(taken(values, from, length, 0)),
            Some(Object::ByteArray(values)) => Object::ByteArray(taken(values, from, length, 0)),
            Some(Object::RefArray(class, values)) => {
                // `copyOf` returns an array of the SOURCE's own type.
                Object::RefArray(class.clone(), taken(values, from, length, JValue::NULL))
            }
            _ => {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.ArrayStoreException: not an array",
                )));
            }
        };
        Ok(JValue::Ref(Some(self.heap.alloc(object))))
    }

    /// `Arrays.fill(target, from, to, value)`.
    fn array_fill(
        &mut self,
        target: HeapRef,
        from: usize,
        to: usize,
        value: JValue,
    ) -> Result<(), VmError> {
        use crate::value::HeapObject as Object;
        match (self.heap.get_mut(target), value) {
            (Some(Object::IntArray(_, values)), JValue::Int(value)) => {
                values[from..to].fill(value);
            }
            (Some(Object::DoubleArray(values)), JValue::Double(value)) => {
                values[from..to].fill(value);
            }
            (Some(Object::LongArray(values)), JValue::Long(value)) => {
                values[from..to].fill(value);
            }
            (Some(Object::FloatArray(values)), JValue::Float(value)) => {
                values[from..to].fill(value);
            }
            (Some(Object::ShortArray(values)), JValue::Int(value)) => {
                values[from..to].fill(value as i16);
            }
            (Some(Object::ByteArray(values)), JValue::Int(value)) => {
                values[from..to].fill(value as i8);
            }
            (Some(Object::RefArray(_, values)), value) => values[from..to].fill(value),
            _ => {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.ArrayStoreException: wrong element type",
                )));
            }
        }
        Ok(())
    }

    /// `Arrays.binarySearch(target, from, to, key)` over a sorted range: the
    /// index of `key`, or `-(insertion point) - 1` when it is absent.
    fn array_binary_search(
        &mut self,
        target: HeapRef,
        from: usize,
        to: usize,
        key: JValue,
    ) -> Result<i32, VmError> {
        let (mut low, mut high) = (from, to);
        while low < high {
            let mid = low + (high - low) / 2;
            match self.array_compare_at(target, mid, key)?.cmp(&0) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return Ok(i32::try_from(mid).unwrap_or(i32::MAX)),
            }
        }
        Ok(-(i32::try_from(low).unwrap_or(i32::MAX) + 1))
    }

    /// Compare `target[at]` with `key`, as the element type's natural order
    /// does. A reference element compares by its own `compareTo`.
    fn array_compare_at(
        &mut self,
        target: HeapRef,
        at: usize,
        key: JValue,
    ) -> Result<i32, VmError> {
        use crate::value::HeapObject as Object;
        let order = match (self.heap.get(target), key) {
            (Some(Object::IntArray(_, values)), JValue::Int(key)) => values[at].cmp(&key),
            (Some(Object::ShortArray(values)), JValue::Int(key)) => i32::from(values[at]).cmp(&key),
            (Some(Object::ByteArray(values)), JValue::Int(key)) => i32::from(values[at]).cmp(&key),
            (Some(Object::LongArray(values)), JValue::Long(key)) => values[at].cmp(&key),
            // `Double.compare`/`Float.compare` order -0.0 below 0.0 and NaN
            // above everything, which is what binarySearch expects.
            (Some(Object::DoubleArray(values)), JValue::Double(key)) => {
                return Ok(java_double_compare(values[at], key));
            }
            (Some(Object::FloatArray(values)), JValue::Float(key)) => {
                return Ok(java_float_compare(values[at], key));
            }
            (Some(Object::RefArray(_, values)), key) => {
                let element = values[at];
                return self.compare_for_sort(element, key);
            }
            _ => {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.ArrayStoreException: wrong key type",
                )));
            }
        };
        Ok(match order {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }

    /// The length of any array on the heap.
    fn array_length(&self, reference: HeapRef) -> Option<usize> {
        use crate::value::HeapObject as Object;
        Some(match self.heap.get(reference)? {
            Object::IntArray(_, values) => values.len(),
            Object::DoubleArray(values) => values.len(),
            Object::LongArray(values) => values.len(),
            Object::FloatArray(values) => values.len(),
            Object::ShortArray(values) => values.len(),
            Object::ByteArray(values) => values.len(),
            Object::RefArray(_, values) => values.len(),
            _ => return None,
        })
    }

    /// `Arrays.toString(primitiveArray)`, for an element array whose static
    /// type is gone. Returns `None` when the object is not a primitive array.
    fn primitive_array_text(&self, reference: HeapRef) -> Option<String> {
        use crate::value::HeapObject;
        let joined = |parts: Vec<String>| parts.join(", ");
        let rendered = match self.heap.get(reference)? {
            HeapObject::IntArray(kind, values) => joined(
                values
                    .iter()
                    .map(|value| int_element_text(*kind, *value))
                    .collect(),
            ),
            HeapObject::DoubleArray(values) => joined(
                values
                    .iter()
                    .map(|value| intrinsics::java_double_to_string(*value))
                    .collect(),
            ),
            HeapObject::FloatArray(values) => joined(
                values
                    .iter()
                    .map(|value| intrinsics::java_float_to_string(*value))
                    .collect(),
            ),
            HeapObject::LongArray(values) => joined(values.iter().map(i64::to_string).collect()),
            HeapObject::ShortArray(values) => joined(values.iter().map(i16::to_string).collect()),
            HeapObject::ByteArray(values) => joined(values.iter().map(i8::to_string).collect()),
            _ => return None,
        };
        Some(format!("[{rendered}]"))
    }

    /// `Arrays.hashCode(primitiveArray)`. `None` when not a primitive array.
    fn primitive_array_hash(&self, reference: HeapRef) -> Option<i32> {
        use crate::value::HeapObject;
        let fold = |hashes: Vec<i32>| {
            hashes.into_iter().fold(1i32, |result, hash| {
                result.wrapping_mul(31).wrapping_add(hash)
            })
        };
        let hash = match self.heap.get(reference)? {
            HeapObject::IntArray(kind, values) => fold(
                values
                    .iter()
                    .map(|value| int_element_hash(*kind, *value))
                    .collect(),
            ),
            HeapObject::DoubleArray(values) => fold(
                values
                    .iter()
                    .map(|value| intrinsics::java_double_hash_public(*value))
                    .collect(),
            ),
            HeapObject::FloatArray(values) => {
                fold(values.iter().map(|value| float_hash(*value)).collect())
            }
            HeapObject::LongArray(values) => fold(
                values
                    .iter()
                    .map(|value| intrinsics::fold_to_int(*value))
                    .collect(),
            ),
            HeapObject::ShortArray(values) => fold(values.iter().map(|v| i32::from(*v)).collect()),
            HeapObject::ByteArray(values) => fold(values.iter().map(|v| i32::from(*v)).collect()),
            _ => return None,
        };
        Some(hash)
    }

    /// `Arrays.equals` of two primitive arrays. `None` when they are not both
    /// primitive arrays of the same kind — Java then falls back to `equals`,
    /// which for arrays is identity.
    fn primitive_arrays_equal(&self, a: HeapRef, b: HeapRef) -> Option<bool> {
        use crate::value::HeapObject;
        Some(match (self.heap.get(a)?, self.heap.get(b)?) {
            (HeapObject::IntArray(ka, va), HeapObject::IntArray(kb, vb)) if ka == kb => va == vb,
            // `Double.equals` compares raw bits: NaN equals itself, -0.0 does
            // not equal 0.0.
            (HeapObject::DoubleArray(va), HeapObject::DoubleArray(vb)) => {
                va.len() == vb.len()
                    && va
                        .iter()
                        .zip(vb)
                        .all(|(x, y)| x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()))
            }
            (HeapObject::FloatArray(va), HeapObject::FloatArray(vb)) => {
                va.len() == vb.len()
                    && va
                        .iter()
                        .zip(vb)
                        .all(|(x, y)| x.to_bits() == y.to_bits() || (x.is_nan() && y.is_nan()))
            }
            (HeapObject::LongArray(va), HeapObject::LongArray(vb)) => va == vb,
            (HeapObject::ShortArray(va), HeapObject::ShortArray(vb)) => va == vb,
            (HeapObject::ByteArray(va), HeapObject::ByteArray(vb)) => va == vb,
            _ => return None,
        })
    }

    /// `Arrays.deepToString`. `path` holds the arrays we are inside, so an
    /// array that contains itself renders as `[...]` rather than recursing —
    /// which is what Java's `dejaVu` set is for.
    fn deep_to_string(
        &mut self,
        value: JValue,
        path: &mut Vec<HeapRef>,
    ) -> Result<String, VmError> {
        use crate::value::HeapObject;
        if path.len() > MAX_RENDER_DEPTH as usize {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.StackOverflowError",
            )));
        }
        let JValue::Ref(Some(reference)) = value else {
            return Ok(String::from("null"));
        };
        let Some(HeapObject::RefArray(_, items)) = self.heap.get(reference) else {
            return self.string_value_of(value, 0);
        };
        let items = items.clone();
        if items.is_empty() {
            return Ok(String::from("[]"));
        }
        path.push(reference);
        let mut parts = Vec::with_capacity(items.len());
        for item in items {
            parts.push(self.deep_element_text(item, path)?);
        }
        path.pop();
        Ok(format!("[{}]", parts.join(", ")))
    }

    fn deep_element_text(
        &mut self,
        item: JValue,
        path: &mut Vec<HeapRef>,
    ) -> Result<String, VmError> {
        use crate::value::HeapObject;
        if let JValue::Ref(Some(reference)) = item {
            if let Some(text) = self.primitive_array_text(reference) {
                return Ok(text);
            }
            if matches!(self.heap.get(reference), Some(HeapObject::RefArray(_, _))) {
                if path.contains(&reference) {
                    return Ok(String::from("[...]"));
                }
                return self.deep_to_string(item, path);
            }
        }
        // Not an array: its own toString, `null` included.
        self.string_value_of(item, 0)
    }

    /// `Arrays.deepEquals`. A cycle overflows the stack here as it does on a
    /// real JVM — unlike `deepToString`, Java's has no `dejaVu` guard.
    fn deep_equals(&mut self, a: JValue, b: JValue, depth: u32) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        if depth > MAX_RENDER_DEPTH {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.StackOverflowError",
            )));
        }
        if a == b {
            return Ok(true);
        }
        let (JValue::Ref(Some(left)), JValue::Ref(Some(right))) = (a, b) else {
            return Ok(false);
        };
        let (Some(HeapObject::RefArray(_, ours)), Some(HeapObject::RefArray(_, theirs))) =
            (self.heap.get(left), self.heap.get(right))
        else {
            return Ok(false);
        };
        let (ours, theirs) = (ours.clone(), theirs.clone());
        if ours.len() != theirs.len() {
            return Ok(false);
        }
        for (ours, theirs) in ours.into_iter().zip(theirs) {
            if !self.deep_element_equals(ours, theirs, depth)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn deep_element_equals(
        &mut self,
        ours: JValue,
        theirs: JValue,
        depth: u32,
    ) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        if ours == theirs {
            return Ok(true);
        }
        if ours == JValue::NULL || theirs == JValue::NULL {
            return Ok(false);
        }
        if let (JValue::Ref(Some(left)), JValue::Ref(Some(right))) = (ours, theirs) {
            if matches!(
                (self.heap.get(left), self.heap.get(right)),
                (
                    Some(HeapObject::RefArray(_, _)),
                    Some(HeapObject::RefArray(_, _))
                )
            ) {
                return self.deep_equals(ours, theirs, depth + 1);
            }
            if let Some(equal) = self.primitive_arrays_equal(left, right) {
                return Ok(equal);
            }
        }
        self.java_equals(ours, theirs)
    }

    /// `Arrays.deepHashCode`.
    fn deep_hash_code(&mut self, value: JValue, depth: u32) -> Result<i32, VmError> {
        use crate::value::HeapObject;
        if depth > MAX_RENDER_DEPTH {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.StackOverflowError",
            )));
        }
        let JValue::Ref(Some(reference)) = value else {
            return Ok(0);
        };
        let Some(HeapObject::RefArray(_, items)) = self.heap.get(reference) else {
            return Ok(0);
        };
        let items = items.clone();
        let mut result = 1i32;
        for item in items {
            let element = self.deep_element_hash(item, depth)?;
            result = result.wrapping_mul(31).wrapping_add(element);
        }
        Ok(result)
    }

    fn deep_element_hash(&mut self, item: JValue, depth: u32) -> Result<i32, VmError> {
        use crate::value::HeapObject;
        if item == JValue::NULL {
            return Ok(0);
        }
        if let JValue::Ref(Some(reference)) = item {
            if matches!(self.heap.get(reference), Some(HeapObject::RefArray(_, _))) {
                return self.deep_hash_code(item, depth + 1);
            }
            if let Some(hash) = self.primitive_array_hash(reference) {
                return Ok(hash);
            }
        }
        self.java_hash_code(item)
    }

    /// `list.toString()` / `map.toString()` and the two map views'. Answered
    /// here rather than in the intrinsic layer because rendering an element
    /// may call a user `toString()`, which needs the interpreter.
    fn container_to_string(
        &mut self,
        frame: &mut Frame<'run>,
        receiver: HeapRef,
        method_name: &str,
        descriptor: &str,
    ) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        let renders_its_elements = matches!(
            self.heap.get(receiver),
            Some(
                HeapObject::ArrayList(_)
                    | HeapObject::LinkedList(_)
                    | HeapObject::ArrayDeque(_)
                    | HeapObject::Stack(_)
                    | HeapObject::UnmodifiableList(_)
                    | HeapObject::UnmodifiableSet(_)
                    | HeapObject::UnmodifiableMap(_)
                    | HeapObject::HashMap(_)
                    | HeapObject::TreeMap { .. }
                    | HeapObject::HashSet(_)
                    | HeapObject::TreeSet { .. }
                    | HeapObject::PriorityQueue { .. }
                    | HeapObject::Optional { .. }
                    | HeapObject::MapView { .. }
                    | HeapObject::MapEntry { .. }
            )
        );
        if !renders_its_elements
            || method_name != "toString"
            || descriptor != "()Ljava/lang/String;"
        {
            return Ok(false);
        }
        let text = self.string_value_of(JValue::Ref(Some(receiver)), 0)?;
        let reference = self.heap.alloc_string(&text);
        frame.stack.push(JValue::Ref(Some(reference)));
        Ok(true)
    }

    /// `println(Object)` and `StringBuilder.append(Object)` render their
    /// argument, which may call a user `toString()`. Rather than duplicate
    /// the console-capturing and appending logic, render the argument here
    /// and hand the intrinsic layer the equivalent `String` call.
    fn render_object_argument(
        &mut self,
        receiver: HeapRef,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Option<(&'static str, JValue)>, VmError> {
        use crate::value::HeapObject;
        let string_form = match (self.heap.get(receiver), method_name, descriptor) {
            (Some(HeapObject::PrintStream(_)), "print" | "println", "(Ljava/lang/Object;)V") => {
                "(Ljava/lang/String;)V"
            }
            (
                Some(HeapObject::StringBuilder(_)),
                "append",
                "(Ljava/lang/Object;)Ljava/lang/StringBuilder;",
            ) => "(Ljava/lang/String;)Ljava/lang/StringBuilder;",
            _ => return Ok(None),
        };
        let text = self.string_value_of(args[0], 0)?;
        let reference = self.heap.alloc_string(&text);
        Ok(Some((string_form, JValue::Ref(Some(reference)))))
    }

    /// `a.equals(b)`, dispatching a user `equals(Object)` override. Values the
    /// VM can compare on its own (primitives, wrappers, strings, null) never
    /// run Java.
    fn java_equals(&mut self, a: JValue, b: JValue) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        if let JValue::Ref(Some(reference)) = a
            && let Some(HeapObject::Instance { class_name, .. }) = self.heap.get(reference)
        {
            let class_name = class_name.clone();
            let returned = match self.user_virtual_dispatch(
                reference,
                &class_name,
                "equals",
                "(Ljava/lang/Object;)Z",
                &[b],
            )? {
                UserDispatch::Call(frame) => self.run_nested(frame)?,
                // `Object.equals`: reference identity, supplied inline.
                UserDispatch::Value(value) => value,
            };
            return Ok(matches!(returned, Some(JValue::Int(result)) if result != 0));
        }
        Ok(intrinsics::native_equals(&self.heap, a, b))
    }

    /// `value.hashCode()`, dispatching a user `hashCode()` override.
    fn java_hash_code(&mut self, value: JValue) -> Result<i32, VmError> {
        use crate::value::HeapObject;
        if let JValue::Ref(Some(reference)) = value
            && let Some(HeapObject::Instance { class_name, .. }) = self.heap.get(reference)
        {
            let class_name = class_name.clone();
            let returned =
                match self.user_virtual_dispatch(reference, &class_name, "hashCode", "()I", &[])? {
                    UserDispatch::Call(frame) => self.run_nested(frame)?,
                    // `Object.hashCode`: the identity hash, supplied inline.
                    UserDispatch::Value(value) => value,
                };
            return match returned {
                Some(JValue::Int(hash)) => Ok(hash),
                _ => Err(VmError::UnknownIntrinsic(format!(
                    "{class_name}.hashCode did not return an int"
                ))),
            };
        }
        Ok(intrinsics::native_hash(&self.heap, value))
    }

    /// The list's elements, detached from the heap borrow. An unmodifiable
    /// view reads through to what it wraps.
    fn list_items(&self, receiver: HeapRef) -> Vec<JValue> {
        self.heap
            .list_values(self.backing_list(receiver))
            .cloned()
            .unwrap_or_default()
    }

    fn is_unmodifiable_list(&self, reference: HeapRef) -> bool {
        matches!(
            self.heap.get(reference),
            Some(crate::value::HeapObject::UnmodifiableList(_))
        )
    }

    /// The list a reference ultimately names, unwrapping unmodifiable views.
    fn backing_list(&self, reference: HeapRef) -> HeapRef {
        let mut current = reference;
        // Views of views are possible; the chain is short and acyclic.
        for _ in 0..MAX_RENDER_DEPTH {
            match self.heap.get(current) {
                Some(crate::value::HeapObject::UnmodifiableList(inner)) => current = *inner,
                _ => break,
            }
        }
        current
    }

    /// `list.contains(probe)`: Java asks the *probe*, not the element.
    fn list_contains(&mut self, list: HeapRef, probe: JValue) -> Result<bool, VmError> {
        Ok(self.list_index_of(list, probe, false)? >= 0)
    }

    /// The `ArrayList` argument of `addAll`/`containsAll`/`removeAll`/
    /// `retainAll`, seen through any unmodifiable view.
    fn list_argument(&self, argument: JValue) -> Result<HeapRef, VmError> {
        let JValue::Ref(Some(other)) = argument else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let other = self.backing_list(other);
        if self.heap.list_values(other).is_none() {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.ClassCastException: not a Collection",
            )));
        }
        Ok(other)
    }

    /// `list.indexOf(probe)`: Java asks the *probe* whether it equals each
    /// element, so an asymmetric `equals` behaves as it does on a real JVM.
    fn list_index_of(
        &mut self,
        receiver: HeapRef,
        probe: JValue,
        from_end: bool,
    ) -> Result<i32, VmError> {
        let items = self.list_items(receiver);
        let positions: Vec<usize> = if from_end {
            (0..items.len()).rev().collect()
        } else {
            (0..items.len()).collect()
        };
        for at in positions {
            if self.java_equals(probe, items[at])? {
                return Ok(i32::try_from(at).unwrap_or(-1));
            }
        }
        Ok(-1)
    }

    /// The `ArrayList` methods that compare elements. They live here rather
    /// than in the intrinsic layer because a user class may override `equals`
    /// and `hashCode`, and calling those needs the interpreter.
    #[allow(clippy::too_many_lines)] // one arm per element-comparing method
    fn list_equality_intrinsic(
        &mut self,
        receiver: HeapRef,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        if self.heap.list_values(receiver).is_none() {
            return Ok(Answered::No);
        }
        let result = match (method_name, descriptor, args) {
            ("forEach", _, [JValue::Ref(Some(consumer))]) => {
                self.list_for_each(receiver, *consumer)?;
                return Ok(Answered::Void);
            }
            ("removeIf", _, [JValue::Ref(Some(predicate))]) => {
                let removed = self.list_remove_if(receiver, *predicate)?;
                return Ok(Answered::Value(JValue::Int(i32::from(removed))));
            }
            ("replaceAll", _, [JValue::Ref(Some(op))]) => {
                self.list_replace_all(receiver, *op)?;
                return Ok(Answered::Void);
            }
            ("sort", _, [JValue::Ref(Some(comparator))]) => {
                let items = self.list_items(receiver);
                let sorted = self.merge_sort_by(items, Some(*comparator))?;
                if let Some(slot) = self.heap.list_values_mut(receiver) {
                    *slot = sorted;
                }
                return Ok(Answered::Void);
            }
            ("contains", _, [probe]) => {
                JValue::Int(i32::from(self.list_index_of(receiver, *probe, false)? >= 0))
            }
            ("indexOf", _, [probe]) => JValue::Int(self.list_index_of(receiver, *probe, false)?),
            ("lastIndexOf", _, [probe]) => JValue::Int(self.list_index_of(receiver, *probe, true)?),
            // `Stack.search(o)`: the 1-based distance from the top (the end) of
            // the topmost matching element, or -1 if absent. Java's uses
            // `lastIndexOf`, so a duplicate reports its position nearest the top.
            ("search", _, [probe]) => {
                let at = self.list_index_of(receiver, *probe, true)?;
                JValue::Int(if at >= 0 {
                    i32::try_from(self.list_items(receiver).len()).unwrap_or(i32::MAX) - at
                } else {
                    -1
                })
            }
            ("remove", "(Ljava/lang/Object;)Z", [probe]) => {
                let at = self.list_index_of(receiver, *probe, false)?;
                let found = at >= 0;
                if let Ok(at) = usize::try_from(at)
                    && let Some(items) = self.heap.list_values_mut(receiver)
                {
                    items.remove(at);
                }
                JValue::Int(i32::from(found))
            }
            // `AbstractList.equals`: same size, and each element equal in
            // order — asking *this* list's element, not the other's.
            ("equals", _, [JValue::Ref(other)]) => {
                let ours = self.list_items(receiver);
                // An unmodifiable view equals the list it wraps.
                let theirs = match other.map(|other| self.backing_list(other)) {
                    Some(other) => match self.heap.list_values(other) {
                        Some(items) => items.clone(),
                        _ => return Ok(Answered::Value(JValue::Int(0))),
                    },
                    None => return Ok(Answered::Value(JValue::Int(0))),
                };
                let mut equal = ours.len() == theirs.len();
                for (ours, theirs) in ours.iter().zip(&theirs) {
                    if !equal {
                        break;
                    }
                    equal = self.java_equals(*ours, *theirs)?;
                }
                JValue::Int(i32::from(equal))
            }
            // `AbstractCollection.containsAll` asks *this* list whether it
            // contains each of the other's elements, so the probe is theirs.
            ("containsAll", _, [other]) => {
                let other = self.list_argument(*other)?;
                let mut all = true;
                for theirs in self.list_items(other) {
                    if !self.list_contains(receiver, theirs)? {
                        all = false;
                        break;
                    }
                }
                JValue::Int(i32::from(all))
            }
            // `removeAll`/`retainAll` ask the *other* collection whether it
            // contains each of ours, so here the probe is ours. Both report
            // whether the list changed.
            ("removeAll" | "retainAll", _, [other]) => {
                let other = self.list_argument(*other)?;
                let keep_when_present = method_name == "retainAll";
                let ours = self.list_items(receiver);
                let mut kept = Vec::with_capacity(ours.len());
                for item in ours {
                    if self.list_contains(other, item)? == keep_when_present {
                        kept.push(item);
                    }
                }
                let changed = kept.len() != self.list_items(receiver).len();
                if let Some(items) = self.heap.list_values_mut(receiver) {
                    *items = kept;
                }
                JValue::Int(i32::from(changed))
            }
            // `AbstractList.hashCode`: the 31-fold of the elements' own.
            ("hashCode", _, []) => {
                let items = self.list_items(receiver);
                let mut hash = 1i32;
                for item in items {
                    let element = self.java_hash_code(item)?;
                    hash = hash.wrapping_mul(31).wrapping_add(element);
                }
                JValue::Int(hash)
            }
            _ => return Ok(Answered::No),
        };
        Ok(Answered::Value(result))
    }

    /// The entry position holding a key equal to `key`, if any. Java compares
    /// hashes first and only then `equals`, so a key whose `hashCode`
    /// disagrees with its `equals` goes missing — here exactly as there.
    fn map_find(&mut self, map: HeapRef, key: JValue) -> Result<Option<usize>, VmError> {
        // A TreeMap locates a key by comparison, not hashing.
        if matches!(
            self.heap.get(map),
            Some(crate::value::HeapObject::TreeMap { .. })
        ) {
            return self.tree_map_index_of(map, key);
        }
        let hash = self.java_hash_code(key)?;
        self.map_find_hashed(map, hash, key)
    }

    /// The comparator a `TreeMap` orders keys with (or `None` for natural
    /// ordering), detached from the heap borrow.
    fn tree_map_comparator(&self, map: HeapRef) -> Option<HeapRef> {
        match self.heap.get(map) {
            Some(crate::value::HeapObject::TreeMap { comparator, .. }) => *comparator,
            _ => None,
        }
    }

    /// The position of the entry whose key compares equal to `key`, if present.
    fn tree_map_index_of(&mut self, map: HeapRef, key: JValue) -> Result<Option<usize>, VmError> {
        let comparator = self.tree_map_comparator(map);
        for (index, (stored, _)) in self.map_entries(map).into_iter().enumerate() {
            if self.compare_with(key, stored, comparator)? == 0 {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    /// The same, when the key's hash is already in hand — Java hashes a key
    /// once per operation, even when the operation both looks up and inserts.
    fn map_find_hashed(
        &mut self,
        map: HeapRef,
        hash: i32,
        key: JValue,
    ) -> Result<Option<usize>, VmError> {
        use crate::value::HeapObject;
        let Some(HeapObject::HashMap(entries) | HeapObject::HashSet(entries)) = self.heap.get(map)
        else {
            return Ok(None);
        };
        for at in entries.candidates(hash) {
            let Some(HeapObject::HashMap(entries) | HeapObject::HashSet(entries)) =
                self.heap.get(map)
            else {
                return Ok(None);
            };
            let stored = entries.key_at(at);
            // `getNode` takes identity before `equals`, so an object always
            // finds itself even if its `equals` says otherwise.
            if stored == key || self.java_equals(key, stored)? {
                return Ok(Some(at));
            }
        }
        Ok(None)
    }

    /// The value a `Map.Entry` currently maps to (it is a live view).
    fn map_entry_value(&mut self, map: HeapRef, key: JValue) -> Result<JValue, VmError> {
        let Some(at) = self.map_find(map, key)? else {
            return Ok(JValue::NULL);
        };
        Ok(self.map_value_at(map, at))
    }

    /// Whether any value in the map equals `probe`. `AbstractMap` asks the
    /// probe, not the held value.
    fn map_contains_value(&mut self, map: HeapRef, probe: JValue) -> Result<bool, VmError> {
        for (_, held) in self.map_entries(map) {
            if self.java_equals(probe, held)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// A map's entries in iteration order, detached from the heap borrow.
    fn map_entries(&self, map: HeapRef) -> Vec<(JValue, JValue)> {
        match self.heap.get(map) {
            Some(
                crate::value::HeapObject::HashMap(entries)
                | crate::value::HeapObject::HashSet(entries),
            ) => entries.entries_in_order(),
            // A TreeMap's entries are already stored in key order.
            Some(crate::value::HeapObject::TreeMap { entries, .. }) => entries.clone(),
            _ => Vec::new(),
        }
    }

    fn map_len(&self, map: HeapRef) -> usize {
        match self.heap.get(map) {
            Some(
                crate::value::HeapObject::HashMap(entries)
                | crate::value::HeapObject::HashSet(entries),
            ) => entries.len(),
            Some(crate::value::HeapObject::TreeMap { entries, .. }) => entries.len(),
            _ => 0,
        }
    }

    /// `map.put(key, value)`, returning the previous value (or `null`).
    fn map_put(&mut self, map: HeapRef, key: JValue, value: JValue) -> Result<JValue, VmError> {
        use crate::value::HeapObject;
        if matches!(self.heap.get(map), Some(HeapObject::TreeMap { .. })) {
            return self.tree_map_put(map, key, value);
        }
        let hash = self.java_hash_code(key)?;
        if let Some(at) = self.map_find_hashed(map, hash, key)?
            && let Some(HeapObject::HashMap(entries) | HeapObject::HashSet(entries)) =
                self.heap.get_mut(map)
        {
            return Ok(entries.set_value_at(at, value));
        }
        if let Some(HeapObject::HashMap(entries) | HeapObject::HashSet(entries)) =
            self.heap.get_mut(map)
        {
            entries.insert_new(hash, key, value);
        }
        Ok(JValue::NULL)
    }

    /// `treeMap.put(key, value)`: replace the value if the key is present
    /// (`compare == 0`), otherwise insert the entry in sorted key position.
    /// Returns the previous value, or `null`.
    fn tree_map_put(
        &mut self,
        map: HeapRef,
        key: JValue,
        value: JValue,
    ) -> Result<JValue, VmError> {
        use crate::value::HeapObject;
        let comparator = self.tree_map_comparator(map);
        let entries = self.map_entries(map);
        let mut at = entries.len();
        for (index, (stored, _)) in entries.iter().enumerate() {
            let ordering = self.compare_with(key, *stored, comparator)?;
            if ordering == 0 {
                let Some(HeapObject::TreeMap { entries, .. }) = self.heap.get_mut(map) else {
                    return Ok(JValue::NULL);
                };
                return Ok(std::mem::replace(&mut entries[index].1, value));
            }
            if ordering < 0 {
                at = index;
                break;
            }
        }
        if let Some(HeapObject::TreeMap { entries, .. }) = self.heap.get_mut(map) {
            entries.insert(at, (key, value));
        }
        Ok(JValue::NULL)
    }

    /// `java.util.HashMap` and its views. Every method that compares a key or
    /// a value lives here rather than in the intrinsic layer, because a user
    /// class may override `equals` and `hashCode`.
    #[allow(clippy::too_many_lines)] // one method table
    fn map_intrinsic(
        &mut self,
        receiver: HeapRef,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        use crate::value::HeapObject;
        // A TreeMap flows through the same arms: its methods reach the map
        // helpers, which route a TreeMap to comparison-based lookup and its
        // sorted vector. The sorted-only navigation methods are handled below.
        match self.heap.get(receiver) {
            Some(HeapObject::HashMap(_) | HeapObject::TreeMap { .. }) => {}
            Some(HeapObject::HashSet(_)) => {
                return self.set_intrinsic(receiver, method_name, descriptor, args);
            }
            Some(HeapObject::TreeSet { .. }) => {
                return self.tree_set_intrinsic(receiver, method_name, descriptor, args);
            }
            // Unmodifiable set/map views: a mutator throws; every read delegates
            // to the backing collection (recursing with its own reference).
            Some(HeapObject::UnmodifiableSet(inner)) => {
                let inner = *inner;
                if is_set_mutator(method_name) {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.UnsupportedOperationException",
                    )));
                }
                return self.map_intrinsic(inner, method_name, descriptor, args);
            }
            Some(HeapObject::UnmodifiableMap(inner)) => {
                let inner = *inner;
                if is_map_mutator(method_name) {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.UnsupportedOperationException",
                    )));
                }
                // `keySet`/`values`/`entrySet` of an unmodifiable map are
                // themselves unmodifiable (else a `keySet().remove(k)` would
                // write through), so wrap the delegated view.
                let answered = self.map_intrinsic(inner, method_name, descriptor, args)?;
                if matches!(method_name, "keySet" | "values" | "entrySet")
                    && let Answered::Value(JValue::Ref(Some(view))) = answered
                {
                    let wrapped = self.heap.alloc(HeapObject::UnmodifiableSet(view));
                    return Ok(Answered::Value(JValue::Ref(Some(wrapped))));
                }
                return Ok(answered);
            }
            Some(HeapObject::PriorityQueue { .. }) => {
                return self.priority_queue_intrinsic(receiver, method_name, descriptor, args);
            }
            Some(HeapObject::MapView { map, kind }) => {
                let (map, kind) = (*map, *kind);
                return self.map_view_intrinsic(map, kind, method_name, descriptor, args);
            }
            Some(HeapObject::MapEntry { map, key }) => {
                let (map, key) = (*map, *key);
                return self.map_entry_intrinsic(map, key, method_name, descriptor, args);
            }
            _ => return Ok(Answered::No),
        }

        let result = match (method_name, args) {
            ("size", []) => JValue::Int(i32::try_from(self.map_len(receiver)).unwrap_or(i32::MAX)),
            ("isEmpty", []) => JValue::Int(i32::from(self.map_len(receiver) == 0)),
            ("clear", []) => {
                match self.heap.get_mut(receiver) {
                    Some(HeapObject::HashMap(entries)) => entries.clear(),
                    Some(HeapObject::TreeMap { entries, .. }) => entries.clear(),
                    _ => {}
                }
                return Ok(Answered::Void);
            }
            ("forEach", [JValue::Ref(Some(consumer))]) => {
                self.map_for_each(receiver, *consumer)?;
                return Ok(Answered::Void);
            }
            // TreeMap key navigation (the receiver is only ever a TreeMap here,
            // since HashMap does not declare these). `firstKey`/`lastKey` throw
            // on an empty map; the others return `null` when absent.
            ("firstKey" | "lastKey", []) => {
                let entries = self.map_entries(receiver);
                let entry = if method_name == "firstKey" {
                    entries.first()
                } else {
                    entries.last()
                };
                let Some((key, _)) = entry else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.util.NoSuchElementException",
                    )));
                };
                *key
            }
            ("floorKey" | "ceilingKey" | "lowerKey" | "higherKey", [probe]) => {
                self.tree_map_navigate_key(receiver, method_name, *probe)?
            }
            ("containsKey", [key]) => {
                JValue::Int(i32::from(self.map_find(receiver, *key)?.is_some()))
            }
            ("containsValue", [value]) => {
                JValue::Int(i32::from(self.map_contains_value(receiver, *value)?))
            }
            ("get", [key]) => self.map_entry_value(receiver, *key)?,
            ("getOrDefault", [key, fallback]) => match self.map_find(receiver, *key)? {
                Some(at) => self.map_value_at(receiver, at),
                None => *fallback,
            },
            ("put", [key, value]) => self.map_put(receiver, *key, *value)?,
            ("putIfAbsent", [key, value]) => {
                // Java treats a key mapped to null as absent here.
                match self.map_find(receiver, *key)? {
                    Some(at) if self.map_value_at(receiver, at) != JValue::NULL => {
                        self.map_value_at(receiver, at)
                    }
                    _ => {
                        self.map_put(receiver, *key, *value)?;
                        JValue::NULL
                    }
                }
            }
            ("remove", [key]) => match self.map_find(receiver, *key)? {
                Some(at) => self.map_remove_at(receiver, at),
                None => JValue::NULL,
            },
            ("remove", [key, value]) => {
                let removed = match self.map_find(receiver, *key)? {
                    Some(at) => {
                        let held = self.map_value_at(receiver, at);
                        if self.java_equals(*value, held)? {
                            self.map_remove_at(receiver, at);
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                };
                JValue::Int(i32::from(removed))
            }
            ("replace", [key, value]) => match self.map_find(receiver, *key)? {
                Some(at) => self.map_set_value_at(receiver, at, *value),
                None => JValue::NULL,
            },
            ("replace", [key, expected, value]) => {
                let replaced = match self.map_find(receiver, *key)? {
                    Some(at) => {
                        let held = self.map_value_at(receiver, at);
                        if self.java_equals(*expected, held)? {
                            self.map_set_value_at(receiver, at, *value);
                            true
                        } else {
                            false
                        }
                    }
                    None => false,
                };
                JValue::Int(i32::from(replaced))
            }
            ("putAll", [JValue::Ref(Some(source))]) => {
                for (key, value) in self.map_entries(*source) {
                    self.map_put(receiver, key, value)?;
                }
                return Ok(Answered::Void);
            }
            // `AbstractMap.equals`: the same entries, in whatever order, with
            // values compared by `equals`.
            ("equals", [JValue::Ref(other)]) => {
                let Some(other) = *other else {
                    return Ok(Answered::Value(JValue::Int(0)));
                };
                if !matches!(
                    self.heap.get(other),
                    Some(HeapObject::HashMap(_) | HeapObject::TreeMap { .. })
                ) || self.map_len(receiver) != self.map_len(other)
                {
                    return Ok(Answered::Value(JValue::Int(0)));
                }
                let mut equal = true;
                for (key, value) in self.map_entries(receiver) {
                    let Some(at) = self.map_find(other, key)? else {
                        equal = false;
                        break;
                    };
                    let theirs = self.map_value_at(other, at);
                    if !self.java_equals(value, theirs)? {
                        equal = false;
                        break;
                    }
                }
                JValue::Int(i32::from(equal))
            }
            // `AbstractMap.hashCode`: the sum of the entries', and an entry's
            // is its key's hash XOR its value's.
            ("hashCode", []) => {
                let mut sum = 0i32;
                for (key, value) in self.map_entries(receiver) {
                    let entry = self.java_hash_code(key)? ^ self.java_hash_code(value)?;
                    sum = sum.wrapping_add(entry);
                }
                JValue::Int(sum)
            }
            // Java's three views are live: a later `put` shows through them.
            ("keySet", []) => self.alloc_map_view(receiver, MapViewKind::Keys),
            ("values", []) => self.alloc_map_view(receiver, MapViewKind::Values),
            ("entrySet", []) => self.alloc_map_view(receiver, MapViewKind::Entries),
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "HashMap.{method_name}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// `java.util.HashSet`. Backed by a [`crate::map::JavaHashMap`] keyed on the
    /// elements, so its iteration order is a real `HashSet`'s exactly. Element
    /// comparison may run a user `equals`/`hashCode`, so — as with the map —
    /// every element-touching operation lives here rather than in the intrinsic
    /// layer.
    #[allow(clippy::too_many_lines)] // one method table
    fn set_intrinsic(
        &mut self,
        receiver: HeapRef,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        use crate::value::HeapObject;
        let result = match (method_name, args) {
            ("size", []) => JValue::Int(i32::try_from(self.map_len(receiver)).unwrap_or(i32::MAX)),
            ("isEmpty", []) => JValue::Int(i32::from(self.map_len(receiver) == 0)),
            ("clear", []) => {
                if let Some(HeapObject::HashSet(entries)) = self.heap.get_mut(receiver) {
                    entries.clear();
                }
                return Ok(Answered::Void);
            }
            ("contains", [probe]) => {
                JValue::Int(i32::from(self.map_find(receiver, *probe)?.is_some()))
            }
            ("add", [element]) => JValue::Int(i32::from(self.set_add(receiver, *element)?)),
            ("remove", [element]) => {
                let removed = match self.map_find(receiver, *element)? {
                    Some(at) => {
                        self.map_remove_at(receiver, at);
                        true
                    }
                    None => false,
                };
                JValue::Int(i32::from(removed))
            }
            ("addAll", [JValue::Ref(Some(source))]) => {
                let mut changed = false;
                for element in self.collection_elements(*source) {
                    changed |= self.set_add(receiver, element)?;
                }
                JValue::Int(i32::from(changed))
            }
            ("containsAll", [JValue::Ref(Some(source))]) => {
                let mut all = true;
                for element in self.collection_elements(*source) {
                    if self.map_find(receiver, element)?.is_none() {
                        all = false;
                        break;
                    }
                }
                JValue::Int(i32::from(all))
            }
            ("removeAll", [JValue::Ref(Some(source))]) => {
                let mut changed = false;
                for element in self.collection_elements(*source) {
                    if let Some(at) = self.map_find(receiver, element)? {
                        self.map_remove_at(receiver, at);
                        changed = true;
                    }
                }
                JValue::Int(i32::from(changed))
            }
            ("retainAll", [JValue::Ref(Some(source))]) => {
                let keep = self.collection_elements(*source);
                let mut changed = false;
                for element in self.collection_elements(receiver) {
                    let mut found = false;
                    for candidate in &keep {
                        if element == *candidate || self.java_equals(element, *candidate)? {
                            found = true;
                            break;
                        }
                    }
                    if !found && let Some(at) = self.map_find(receiver, element)? {
                        self.map_remove_at(receiver, at);
                        changed = true;
                    }
                }
                JValue::Int(i32::from(changed))
            }
            ("forEach", [JValue::Ref(Some(consumer))]) => {
                self.set_for_each(receiver, *consumer)?;
                return Ok(Answered::Void);
            }
            ("removeIf", [JValue::Ref(Some(predicate))]) => {
                let removed = self.set_remove_if(receiver, *predicate)?;
                return Ok(Answered::Value(JValue::Int(i32::from(removed))));
            }
            // caturra's own indexed accessor, standing in for the iterator the
            // enhanced-for loop would otherwise need.
            ("__get", [JValue::Int(position)]) => {
                let position = usize::try_from(*position).unwrap_or(usize::MAX);
                let entry = match self.heap.get(receiver) {
                    Some(HeapObject::HashSet(entries)) => entries.entry_at(position),
                    _ => None,
                };
                let Some((element, _)) = entry else {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.IndexOutOfBoundsException: Index {position} out of bounds"
                    )));
                };
                element
            }
            // `AbstractSet.equals`: another Set of the same size holding them all.
            ("equals", [JValue::Ref(other)]) => {
                let Some(other) = *other else {
                    return Ok(Answered::Value(JValue::Int(0)));
                };
                if !matches!(self.heap.get(other), Some(HeapObject::HashSet(_)))
                    || self.map_len(receiver) != self.map_len(other)
                {
                    return Ok(Answered::Value(JValue::Int(0)));
                }
                let mut equal = true;
                for element in self.collection_elements(receiver) {
                    if self.map_find(other, element)?.is_none() {
                        equal = false;
                        break;
                    }
                }
                JValue::Int(i32::from(equal))
            }
            // `AbstractSet.hashCode`: the sum of the elements' hash codes.
            ("hashCode", []) => {
                let mut sum = 0i32;
                for element in self.collection_elements(receiver) {
                    sum = sum.wrapping_add(self.java_hash_code(element)?);
                }
                JValue::Int(sum)
            }
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "HashSet.{method_name}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// `set.add(element)`: store it as a key mapped to a placeholder if absent,
    /// reporting whether the set changed. Java's `HashSet.add` is exactly
    /// `map.put(e, PRESENT) == null`.
    fn set_add(&mut self, set: HeapRef, element: JValue) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        let hash = self.java_hash_code(element)?;
        if self.map_find_hashed(set, hash, element)?.is_some() {
            return Ok(false);
        }
        if let Some(HeapObject::HashSet(entries)) = self.heap.get_mut(set) {
            entries.insert_new(hash, element, JValue::NULL);
        }
        Ok(true)
    }

    /// `set.forEach(consumer)`: call `accept(element)` on each element in
    /// iteration order, running the synthesized lambda through the nested-call
    /// machinery (a user `accept` may itself touch the set).
    /// `set.removeIf(predicate)` / `treeSet.removeIf(predicate)`: drop the
    /// elements the predicate accepts, reporting whether any went. (The list
    /// backings go through `list_remove_if`; a set rebuilds its own storage —
    /// a `TreeSet`'s vector directly, a `HashSet` by re-removing each match.)
    fn set_remove_if(&mut self, receiver: HeapRef, predicate: HeapRef) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        if !matches!(self.heap.get(predicate), Some(HeapObject::Instance { .. })) {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        }
        let mut kept = Vec::new();
        let mut removed = Vec::new();
        for element in self.collection_elements(receiver) {
            if self.call_test(predicate, element)? {
                removed.push(element);
            } else {
                kept.push(element);
            }
        }
        if removed.is_empty() {
            return Ok(false);
        }
        match self.heap.get(receiver) {
            // The kept elements are already in sorted order, so write them back.
            Some(HeapObject::TreeSet { .. }) => {
                if let Some(HeapObject::TreeSet { values, .. }) = self.heap.get_mut(receiver) {
                    *values = kept;
                }
            }
            // Re-find and remove each match (the map re-searches by key each
            // time, so shifting positions are handled).
            Some(HeapObject::HashSet(_)) => {
                for element in removed {
                    if let Some(at) = self.map_find(receiver, element)? {
                        self.map_remove_at(receiver, at);
                    }
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn set_for_each(&mut self, receiver: HeapRef, consumer: HeapRef) -> Result<(), VmError> {
        let elements = self.collection_elements(receiver);
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(consumer)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        for element in elements {
            let dispatched = self.user_virtual_dispatch(
                consumer,
                &class_name,
                "accept",
                "(Ljava/lang/Object;)V",
                &[element],
            )?;
            if let UserDispatch::Call(frame) = dispatched {
                self.run_nested(frame)?;
            }
        }
        Ok(())
    }

    /// The comparator a `TreeSet` was built with (or `None` for natural
    /// ordering), detached from the heap borrow.
    fn tree_set_comparator(&self, set: HeapRef) -> Option<HeapRef> {
        match self.heap.get(set) {
            Some(crate::value::HeapObject::TreeSet { comparator, .. }) => *comparator,
            _ => None,
        }
    }

    /// The elements of a `TreeSet`, in sorted order (the vector is kept sorted).
    fn tree_set_values(&self, set: HeapRef) -> Vec<JValue> {
        match self.heap.get(set) {
            Some(crate::value::HeapObject::TreeSet { values, .. }) => values.clone(),
            _ => Vec::new(),
        }
    }

    /// `set.add(element)`: insert it in sorted position unless an equal element
    /// (`compare == 0`) is already present. Reports whether the set changed.
    fn tree_set_add(&mut self, set: HeapRef, element: JValue) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        let comparator = self.tree_set_comparator(set);
        let values = self.tree_set_values(set);
        let mut at = values.len();
        for (index, existing) in values.iter().enumerate() {
            let ordering = self.compare_with(element, *existing, comparator)?;
            if ordering == 0 {
                return Ok(false); // already present
            }
            if ordering < 0 {
                at = index;
                break;
            }
        }
        if let Some(HeapObject::TreeSet { values, .. }) = self.heap.get_mut(set) {
            values.insert(at, element);
        }
        Ok(true)
    }

    /// The index of the element equal to `probe` (`compare == 0`), if present.
    fn tree_set_index_of(&mut self, set: HeapRef, probe: JValue) -> Result<Option<usize>, VmError> {
        let comparator = self.tree_set_comparator(set);
        for (index, existing) in self.tree_set_values(set).into_iter().enumerate() {
            if self.compare_with(probe, existing, comparator)? == 0 {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    /// `java.util.TreeSet`. The elements are kept sorted, so `first`/`last` and
    /// the `floor`/`ceiling`/`lower`/`higher` navigation read straight off the
    /// vector; membership and ordering compare with the set's `Comparator` or
    /// the elements' natural ordering, both of which may run user code.
    #[allow(clippy::too_many_lines)] // one method table
    fn tree_set_intrinsic(
        &mut self,
        receiver: HeapRef,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        use crate::value::HeapObject;
        let len = self.tree_set_values(receiver).len();
        let result = match (method_name, args) {
            ("size", []) => JValue::Int(i32::try_from(len).unwrap_or(i32::MAX)),
            ("isEmpty", []) => JValue::Int(i32::from(len == 0)),
            ("clear", []) => {
                if let Some(HeapObject::TreeSet { values, .. }) = self.heap.get_mut(receiver) {
                    values.clear();
                }
                return Ok(Answered::Void);
            }
            ("add", [element]) => JValue::Int(i32::from(self.tree_set_add(receiver, *element)?)),
            ("contains", [probe]) => JValue::Int(i32::from(
                self.tree_set_index_of(receiver, *probe)?.is_some(),
            )),
            ("remove", [probe]) => {
                let removed = match self.tree_set_index_of(receiver, *probe)? {
                    Some(at) => {
                        if let Some(HeapObject::TreeSet { values, .. }) =
                            self.heap.get_mut(receiver)
                        {
                            values.remove(at);
                        }
                        true
                    }
                    None => false,
                };
                JValue::Int(i32::from(removed))
            }
            ("addAll", [JValue::Ref(Some(source))]) => {
                let mut changed = false;
                for element in self.collection_elements(*source) {
                    changed |= self.tree_set_add(receiver, element)?;
                }
                JValue::Int(i32::from(changed))
            }
            ("containsAll", [JValue::Ref(Some(source))]) => {
                let mut all = true;
                for element in self.collection_elements(*source) {
                    if self.tree_set_index_of(receiver, element)?.is_none() {
                        all = false;
                        break;
                    }
                }
                JValue::Int(i32::from(all))
            }
            // The two ends. Empty throws `NoSuchElementException`.
            ("first" | "last", []) => {
                let values = self.tree_set_values(receiver);
                let value = if method_name == "first" {
                    values.first()
                } else {
                    values.last()
                };
                let Some(value) = value else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.util.NoSuchElementException",
                    )));
                };
                *value
            }
            // Navigation: floor ≤ e, ceiling ≥ e, lower < e, higher > e; absent
            // → null.
            ("floor" | "ceiling" | "lower" | "higher", [probe]) => {
                self.tree_set_navigate(receiver, method_name, *probe)?
            }
            ("pollFirst" | "pollLast", []) => {
                let values = self.tree_set_values(receiver);
                if values.is_empty() {
                    JValue::NULL
                } else {
                    let at = if method_name == "pollFirst" {
                        0
                    } else {
                        values.len() - 1
                    };
                    let value = values[at];
                    if let Some(HeapObject::TreeSet { values, .. }) = self.heap.get_mut(receiver) {
                        values.remove(at);
                    }
                    value
                }
            }
            ("forEach", [JValue::Ref(Some(consumer))]) => {
                self.tree_set_for_each(receiver, *consumer)?;
                return Ok(Answered::Void);
            }
            ("removeIf", [JValue::Ref(Some(predicate))]) => {
                let removed = self.set_remove_if(receiver, *predicate)?;
                return Ok(Answered::Value(JValue::Int(i32::from(removed))));
            }
            // The enhanced-for accessor: the element at a sorted position.
            ("__get", [JValue::Int(position)]) => {
                let position = usize::try_from(*position).unwrap_or(usize::MAX);
                let Some(value) = self.tree_set_values(receiver).get(position).copied() else {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.IndexOutOfBoundsException: Index {position} out of bounds"
                    )));
                };
                value
            }
            ("equals", [JValue::Ref(other)]) => {
                let Some(other) = *other else {
                    return Ok(Answered::Value(JValue::Int(0)));
                };
                let ours = self.tree_set_values(receiver);
                let mut equal = self.set_len(other) == ours.len();
                if equal {
                    for element in ours {
                        if !self.set_contains_any(other, element)? {
                            equal = false;
                            break;
                        }
                    }
                }
                JValue::Int(i32::from(equal))
            }
            ("hashCode", []) => {
                let mut sum = 0i32;
                for element in self.tree_set_values(receiver) {
                    sum = sum.wrapping_add(self.java_hash_code(element)?);
                }
                JValue::Int(sum)
            }
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "TreeSet.{method_name}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// `floor`/`ceiling`/`lower`/`higher` over the sorted vector.
    fn tree_set_navigate(
        &mut self,
        set: HeapRef,
        which: &str,
        probe: JValue,
    ) -> Result<JValue, VmError> {
        let comparator = self.tree_set_comparator(set);
        let mut best = JValue::NULL;
        for candidate in self.tree_set_values(set) {
            let ordering = self.compare_with(candidate, probe, comparator)?;
            let matches = match which {
                "floor" => ordering <= 0,
                "lower" => ordering < 0,
                "ceiling" => ordering >= 0,
                "higher" => ordering > 0,
                _ => false,
            };
            if matches {
                // floor/lower want the greatest match (keep climbing); ceiling/
                // higher want the least, so the first hit in sorted order wins.
                if which == "ceiling" || which == "higher" {
                    return Ok(candidate);
                }
                best = candidate;
            }
        }
        Ok(best)
    }

    /// Whether a set (Tree or Hash) contains an element equal to `probe`.
    fn set_contains_any(&mut self, set: HeapRef, probe: JValue) -> Result<bool, VmError> {
        use crate::value::HeapObject;
        match self.heap.get(set) {
            Some(HeapObject::TreeSet { .. }) => Ok(self.tree_set_index_of(set, probe)?.is_some()),
            Some(HeapObject::HashSet(_)) => Ok(self.map_find(set, probe)?.is_some()),
            _ => Ok(false),
        }
    }

    /// The element count of a set (Tree or Hash).
    fn set_len(&self, set: HeapRef) -> usize {
        match self.heap.get(set) {
            Some(crate::value::HeapObject::TreeSet { values, .. }) => values.len(),
            _ => self.map_len(set),
        }
    }

    /// `treeSet.forEach(consumer)`: call `accept` on each element in order.
    fn tree_set_for_each(&mut self, receiver: HeapRef, consumer: HeapRef) -> Result<(), VmError> {
        let elements = self.tree_set_values(receiver);
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(consumer)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        for element in elements {
            let dispatched = self.user_virtual_dispatch(
                consumer,
                &class_name,
                "accept",
                "(Ljava/lang/Object;)V",
                &[element],
            )?;
            if let UserDispatch::Call(frame) = dispatched {
                self.run_nested(frame)?;
            }
        }
        Ok(())
    }

    /// The comparator a `PriorityQueue` orders by (or `None` for natural
    /// ordering), detached from the heap borrow.
    fn pq_comparator(&self, pq: HeapRef) -> Option<HeapRef> {
        match self.heap.get(pq) {
            Some(crate::value::HeapObject::PriorityQueue { comparator, .. }) => *comparator,
            _ => None,
        }
    }

    /// The heap-array of a `PriorityQueue` (its iteration order), detached from
    /// the borrow.
    fn pq_heap(&self, pq: HeapRef) -> Vec<JValue> {
        match self.heap.get(pq) {
            Some(crate::value::HeapObject::PriorityQueue { heap, .. }) => heap.clone(),
            _ => Vec::new(),
        }
    }

    fn set_pq_heap(&mut self, pq: HeapRef, values: Vec<JValue>) {
        if let Some(crate::value::HeapObject::PriorityQueue { heap, .. }) = self.heap.get_mut(pq) {
            *heap = values;
        }
    }

    /// `queue.offer(e)`: append and sift up into heap position. Mirrors Java's
    /// `siftUp` (swap-based, which lands each element where the shift-based JDK
    /// code does), so the resulting heap array is identical.
    fn pq_offer(&mut self, pq: HeapRef, element: JValue) -> Result<(), VmError> {
        let comparator = self.pq_comparator(pq);
        let mut heap = self.pq_heap(pq);
        heap.push(element);
        let mut k = heap.len() - 1;
        while k > 0 {
            let parent = (k - 1) >> 1;
            if self.compare_with(heap[k], heap[parent], comparator)? >= 0 {
                break;
            }
            heap.swap(k, parent);
            k = parent;
        }
        self.set_pq_heap(pq, heap);
        Ok(())
    }

    /// Sift the element at `start` down to its heap position, over `heap`.
    fn pq_sift_down(
        &mut self,
        heap: &mut [JValue],
        comparator: Option<HeapRef>,
        start: usize,
    ) -> Result<(), VmError> {
        let size = heap.len();
        let mut k = start;
        let half = size >> 1;
        while k < half {
            let mut child = 2 * k + 1;
            let right = child + 1;
            if right < size && self.compare_with(heap[child], heap[right], comparator)? > 0 {
                child = right;
            }
            if self.compare_with(heap[k], heap[child], comparator)? <= 0 {
                break;
            }
            heap.swap(k, child);
            k = child;
        }
        Ok(())
    }

    /// `queue.poll()`: remove and return the least element (the root), moving
    /// the last element to the root and sifting it down — the JDK's algorithm,
    /// so the remaining heap array matches. `null` when empty.
    fn pq_poll(&mut self, pq: HeapRef) -> Result<JValue, VmError> {
        let comparator = self.pq_comparator(pq);
        let mut heap = self.pq_heap(pq);
        if heap.is_empty() {
            return Ok(JValue::NULL);
        }
        let result = heap[0];
        let last = heap.pop().expect("non-empty");
        if !heap.is_empty() {
            heap[0] = last;
            self.pq_sift_down(&mut heap, comparator, 0)?;
        }
        self.set_pq_heap(pq, heap);
        Ok(result)
    }

    /// `queue.remove(o)`: drop the first element that `equals` `o`, then repair
    /// the heap the way Java's `removeAt` does (move the last element into the
    /// hole, sift it down, and — if it did not move — sift it up).
    fn pq_remove_object(&mut self, pq: HeapRef, probe: JValue) -> Result<bool, VmError> {
        let comparator = self.pq_comparator(pq);
        let mut heap = self.pq_heap(pq);
        let mut at = None;
        for (index, element) in heap.iter().enumerate() {
            if self.java_equals(probe, *element)? {
                at = Some(index);
                break;
            }
        }
        let Some(i) = at else {
            return Ok(false);
        };
        let last = heap.len() - 1;
        if i == last {
            heap.pop();
        } else {
            let moved = heap.pop().expect("non-empty");
            heap[i] = moved;
            self.pq_sift_down(&mut heap, comparator, i)?;
            if heap[i] == moved {
                // It did not sift down; try sifting it up.
                let mut k = i;
                while k > 0 {
                    let parent = (k - 1) >> 1;
                    if self.compare_with(heap[k], heap[parent], comparator)? >= 0 {
                        break;
                    }
                    heap.swap(k, parent);
                    k = parent;
                }
            }
        }
        self.set_pq_heap(pq, heap);
        Ok(true)
    }

    /// Build a valid heap from an arbitrary vector (Java's `heapify`: sift down
    /// from the last internal node to the root). Used by the collection copy
    /// constructor.
    fn pq_heapify(&mut self, pq: HeapRef) -> Result<(), VmError> {
        let comparator = self.pq_comparator(pq);
        let mut heap = self.pq_heap(pq);
        let size = heap.len();
        for start in (0..size / 2).rev() {
            self.pq_sift_down(&mut heap, comparator, start)?;
        }
        self.set_pq_heap(pq, heap);
        Ok(())
    }

    /// `java.util.PriorityQueue`. A real binary min-heap so `peek`/`poll` return
    /// the least element and the heap-array iteration order matches a JVM's.
    fn priority_queue_intrinsic(
        &mut self,
        receiver: HeapRef,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        let len = self.pq_heap(receiver).len();
        let result = match (method_name, args) {
            ("size", []) => JValue::Int(i32::try_from(len).unwrap_or(i32::MAX)),
            ("isEmpty", []) => JValue::Int(i32::from(len == 0)),
            ("clear", []) => {
                self.set_pq_heap(receiver, Vec::new());
                return Ok(Answered::Void);
            }
            ("add" | "offer", [element]) => {
                self.pq_offer(receiver, *element)?;
                // add/offer both report success (true); the descriptor is `Z`.
                JValue::Int(1)
            }
            ("peek", []) => self
                .pq_heap(receiver)
                .first()
                .copied()
                .unwrap_or(JValue::NULL),
            ("poll", []) => self.pq_poll(receiver)?,
            // `element`/`remove()` are peek/poll that throw when empty.
            ("element", []) => {
                let Some(top) = self.pq_heap(receiver).first().copied() else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.util.NoSuchElementException",
                    )));
                };
                top
            }
            ("remove", []) => {
                if len == 0 {
                    return Err(VmError::UncaughtException(String::from(
                        "java.util.NoSuchElementException",
                    )));
                }
                self.pq_poll(receiver)?
            }
            ("remove", [probe]) => JValue::Int(i32::from(self.pq_remove_object(receiver, *probe)?)),
            ("contains", [probe]) => {
                let mut found = false;
                for element in self.pq_heap(receiver) {
                    if self.java_equals(*probe, element)? {
                        found = true;
                        break;
                    }
                }
                JValue::Int(i32::from(found))
            }
            ("addAll", [JValue::Ref(Some(source))]) => {
                let mut changed = false;
                for element in self.collection_elements(*source) {
                    self.pq_offer(receiver, element)?;
                    changed = true;
                }
                JValue::Int(i32::from(changed))
            }
            ("forEach", [JValue::Ref(Some(consumer))]) => {
                // Heap-array order, as Java's iterator yields.
                let elements = self.pq_heap(receiver);
                let Some(crate::value::HeapObject::Instance { class_name, .. }) =
                    self.heap.get(*consumer)
                else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                let class_name = class_name.clone();
                for element in elements {
                    let dispatched = self.user_virtual_dispatch(
                        *consumer,
                        &class_name,
                        "accept",
                        "(Ljava/lang/Object;)V",
                        &[element],
                    )?;
                    if let UserDispatch::Call(frame) = dispatched {
                        self.run_nested(frame)?;
                    }
                }
                return Ok(Answered::Void);
            }
            // The enhanced-for accessor: the element at a heap-array position.
            // (`get` is what the `Queue`-typed for-each loop emits; a user
            // cannot call it — `Queue` declares no `get`.)
            ("__get" | "get", [JValue::Int(position)]) => {
                let position = usize::try_from(*position).unwrap_or(usize::MAX);
                let Some(element) = self.pq_heap(receiver).get(position).copied() else {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.IndexOutOfBoundsException: Index {position} out of bounds"
                    )));
                };
                element
            }
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "PriorityQueue.{method_name}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// Whether a heap object is a collection `stream()` can be taken of.
    fn is_streamable(&self, reference: HeapRef) -> bool {
        use crate::value::HeapObject;
        matches!(
            self.heap.get(reference),
            Some(
                HeapObject::ArrayList(_)
                    | HeapObject::LinkedList(_)
                    | HeapObject::ArrayDeque(_)
                    | HeapObject::Stack(_)
                    | HeapObject::UnmodifiableList(_)
                    | HeapObject::UnmodifiableSet(_)
                    | HeapObject::HashSet(_)
                    | HeapObject::TreeSet { .. }
                    | HeapObject::PriorityQueue { .. }
                    | HeapObject::MapView { .. }
            )
        )
    }

    /// `collection.stream()` and every method on the resulting `Stream`. A
    /// stream is modelled eagerly (its `Vec` holds the current pipeline), so an
    /// intermediate op makes a new one and a terminal op consumes it.
    fn stream_dispatch(
        &mut self,
        receiver: HeapRef,
        method: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        use crate::value::HeapObject;
        if method == "stream" && args.is_empty() && self.is_streamable(receiver) {
            let elements = self.collection_elements(receiver);
            let stream = self.heap.alloc(HeapObject::Stream(elements));
            return Ok(Answered::Value(JValue::Ref(Some(stream))));
        }
        if matches!(self.heap.get(receiver), Some(HeapObject::Stream(_))) {
            return self.stream_intrinsic(receiver, method, descriptor, args);
        }
        if matches!(self.heap.get(receiver), Some(HeapObject::Optional { .. })) {
            return self.optional_intrinsic(receiver, method, descriptor, args);
        }
        // Combinators on a factory-built comparator.
        if matches!(self.heap.get(receiver), Some(HeapObject::Comparator(_))) {
            use crate::value::ComparatorSpec;
            match (method, args) {
                ("reversed", []) => {
                    let reversed = self
                        .heap
                        .alloc(HeapObject::Comparator(ComparatorSpec::Reversed(receiver)));
                    return Ok(Answered::Value(JValue::Ref(Some(reversed))));
                }
                ("thenComparing", [JValue::Ref(Some(next))]) => {
                    // `thenComparing(keyExtractor)` wraps a Function; the
                    // `Comparator` overload passes one straight through.
                    let second = if descriptor.contains("function/Function") {
                        self.heap
                            .alloc(HeapObject::Comparator(ComparatorSpec::ByKey(*next)))
                    } else {
                        *next
                    };
                    let then = self.heap.alloc(HeapObject::Comparator(ComparatorSpec::Then(
                        receiver, second,
                    )));
                    return Ok(Answered::Value(JValue::Ref(Some(then))));
                }
                ("compare", [a, b]) => {
                    return Ok(Answered::Value(JValue::Int(
                        self.compare_by_spec(*a, *b, receiver)?,
                    )));
                }
                _ => {}
            }
        }
        Ok(Answered::No)
    }

    /// The current elements of a `Stream`, detached from the heap borrow.
    fn stream_elements(&self, stream: HeapRef) -> Vec<JValue> {
        match self.heap.get(stream) {
            Some(crate::value::HeapObject::Stream(elements)) => elements.clone(),
            _ => Vec::new(),
        }
    }

    fn alloc_stream(&mut self, elements: Vec<JValue>) -> JValue {
        JValue::Ref(Some(
            self.heap.alloc(crate::value::HeapObject::Stream(elements)),
        ))
    }

    fn alloc_optional(
        &mut self,
        value: Option<JValue>,
        kind: crate::value::OptionalKind,
    ) -> JValue {
        JValue::Ref(Some(
            self.heap
                .alloc(crate::value::HeapObject::Optional { value, kind }),
        ))
    }

    /// `java.util.Optional` / `OptionalInt` / `OptionalDouble` methods.
    fn optional_intrinsic(
        &mut self,
        receiver: HeapRef,
        method: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        let value = match self.heap.get(receiver) {
            Some(crate::value::HeapObject::Optional { value, .. }) => *value,
            _ => return Ok(Answered::No),
        };
        let result = match (method, args) {
            ("isPresent", []) => JValue::Int(i32::from(value.is_some())),
            ("isEmpty", []) => JValue::Int(i32::from(value.is_none())),
            // The accessor and `orElseThrow()` yield the value or throw.
            ("get" | "getAsInt" | "getAsDouble" | "getAsLong" | "orElseThrow", []) => value
                .ok_or_else(|| {
                    VmError::UncaughtException(String::from(
                        "java.util.NoSuchElementException: No value present",
                    ))
                })?,
            ("orElse", [fallback]) => value.unwrap_or(*fallback),
            ("ifPresent", [JValue::Ref(Some(consumer))]) => {
                if let Some(present) = value {
                    self.call_functional(*consumer, "accept", "(Ljava/lang/Object;)V", present)?;
                }
                return Ok(Answered::Void);
            }
            // `filter(predicate)`: keep a present value only if it matches, else
            // an empty Optional. (Only a reference Optional has `filter`.)
            ("filter", [JValue::Ref(Some(predicate))]) => {
                let kept = match value {
                    Some(present) if self.call_test(*predicate, present)? => Some(present),
                    _ => None,
                };
                return Ok(Answered::Value(
                    self.alloc_optional(kept, crate::value::OptionalKind::Ref),
                ));
            }
            // `map(function)`: transform a present value, else stay empty. A
            // function yielding null makes an empty Optional, as Java's does.
            // The result's element is erased to `Object`, so a primitive is
            // boxed — `get()` is bytecode-typed as a reference.
            ("map", [JValue::Ref(Some(function))]) => {
                let mapped = match value {
                    Some(present) => {
                        let result = self
                            .call_functional(
                                *function,
                                "apply",
                                "(Ljava/lang/Object;)Ljava/lang/Object;",
                                present,
                            )?
                            .unwrap_or(JValue::NULL);
                        match result {
                            JValue::Ref(None) => None,
                            JValue::Ref(Some(_)) => Some(result),
                            primitive => {
                                Some(JValue::Ref(Some(self.box_primitive_value(primitive))))
                            }
                        }
                    }
                    None => None,
                };
                return Ok(Answered::Value(
                    self.alloc_optional(mapped, crate::value::OptionalKind::Ref),
                ));
            }
            // `orElseGet(supplier)`: the value, or the supplier's result.
            ("orElseGet", [JValue::Ref(Some(supplier))]) => match value {
                Some(present) => present,
                None => self.call_apply_supplier(*supplier)?,
            },
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "Optional.{method}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// Call a one-argument functional lambda (`test`/`apply`/`accept`) on an
    /// element, running the synthesized class through the nested-call path.
    fn call_functional(
        &mut self,
        target: HeapRef,
        method: &str,
        descriptor: &str,
        element: JValue,
    ) -> Result<Option<JValue>, VmError> {
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(target)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        let dispatched =
            self.user_virtual_dispatch(target, &class_name, method, descriptor, &[element])?;
        Ok(match dispatched {
            UserDispatch::Call(frame) => self.run_nested(frame)?,
            UserDispatch::Value(value) => value,
        })
    }

    /// `predicate.test(element)`.
    fn call_test(&mut self, predicate: HeapRef, element: JValue) -> Result<bool, VmError> {
        let result = self.call_functional(predicate, "test", "(Ljava/lang/Object;)Z", element)?;
        Ok(result.is_some() && result != Some(JValue::Int(0)))
    }

    /// `function.apply(element)`, unboxing a primitive result so it stores like
    /// caturra's other collection elements (as `list.replaceAll` does).
    fn call_apply(&mut self, function: HeapRef, element: JValue) -> Result<JValue, VmError> {
        let result = self.call_functional(
            function,
            "apply",
            "(Ljava/lang/Object;)Ljava/lang/Object;",
            element,
        )?;
        Ok(self.unbox_functional_result(result))
    }

    /// `supplier.get()` (a zero-argument lambda), unboxing a primitive result —
    /// backs `Optional.orElseGet`.
    fn call_apply_supplier(&mut self, supplier: HeapRef) -> Result<JValue, VmError> {
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(supplier)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        let dispatched =
            self.user_virtual_dispatch(supplier, &class_name, "get", "()Ljava/lang/Object;", &[])?;
        let result = match dispatched {
            UserDispatch::Call(frame) => self.run_nested(frame)?,
            UserDispatch::Value(value) => value,
        };
        Ok(self.unbox_functional_result(result))
    }

    /// Unbox a boxed-primitive lambda result so it stores like caturra's other
    /// (unboxed) values; a reference or absent result passes through.
    fn unbox_functional_result(&self, result: Option<JValue>) -> JValue {
        match result {
            Some(JValue::Ref(Some(r))) => match self.heap.get(r) {
                Some(crate::value::HeapObject::Boxed { value, .. }) => *value,
                _ => JValue::Ref(Some(r)),
            },
            Some(value) => value,
            None => JValue::NULL,
        }
    }

    /// `java.util.stream.Stream` operations. Element comparison/transformation
    /// may run user lambdas, so this lives in the interpreter.
    #[allow(clippy::too_many_lines)] // one method table
    fn stream_intrinsic(
        &mut self,
        receiver: HeapRef,
        method: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        use crate::value::OptionalKind;
        let elements = self.stream_elements(receiver);
        let result = match (method, args) {
            ("filter", [JValue::Ref(Some(pred))]) => {
                let mut kept = Vec::with_capacity(elements.len());
                for element in elements {
                    if self.call_test(*pred, element)? {
                        kept.push(element);
                    }
                }
                self.alloc_stream(kept)
            }
            // `map`/`mapToInt`/`mapToObj` all apply a one-argument function to
            // each element; the difference (object vs int result) is compile-time
            // only, since caturra stores primitives unboxed.
            (
                "map" | "mapToInt" | "mapToObj" | "mapToLong" | "mapToDouble",
                [JValue::Ref(Some(function))],
            ) => {
                let mut mapped = Vec::with_capacity(elements.len());
                for element in elements {
                    mapped.push(self.call_apply(*function, element)?);
                }
                self.alloc_stream(mapped)
            }
            // `IntStream.boxed()` / `asLongStream()` — a no-op here (elements are
            // stored unboxed either way), just a retyping.
            ("boxed" | "asLongStream" | "asDoubleStream", []) => self.alloc_stream(elements),
            // `IntStream.sum()` — add the (unboxed int) elements, wrapping.
            ("sum", []) => {
                let mut total = 0i32;
                for element in elements {
                    if let JValue::Int(n) = element {
                        total = total.wrapping_add(n);
                    }
                }
                JValue::Int(total)
            }
            // `IntStream.toArray()` — the ints as an `int[]`.
            ("toArray", []) => {
                let ints: Vec<i32> = elements
                    .into_iter()
                    .map(|e| if let JValue::Int(n) = e { n } else { 0 })
                    .collect();
                JValue::Ref(Some(self.heap.alloc(crate::value::HeapObject::IntArray(
                    crate::value::IntKind::Int,
                    ints,
                ))))
            }
            ("sorted", []) => {
                let sorted = self.merge_sort_by(elements, None)?;
                self.alloc_stream(sorted)
            }
            ("sorted", [JValue::Ref(Some(comparator))]) => {
                let sorted = self.merge_sort_by(elements, Some(*comparator))?;
                self.alloc_stream(sorted)
            }
            ("distinct", []) => {
                let mut unique: Vec<JValue> = Vec::new();
                for element in elements {
                    let mut seen = false;
                    for kept in &unique {
                        if self.java_equals(element, *kept)? {
                            seen = true;
                            break;
                        }
                    }
                    if !seen {
                        unique.push(element);
                    }
                }
                self.alloc_stream(unique)
            }
            ("limit", [count]) => {
                let count = stream_count_arg(*count);
                self.alloc_stream(elements.into_iter().take(count).collect())
            }
            ("skip", [count]) => {
                let count = stream_count_arg(*count);
                self.alloc_stream(elements.into_iter().skip(count).collect())
            }
            ("peek", [JValue::Ref(Some(consumer))]) => {
                for element in &elements {
                    self.call_functional(*consumer, "accept", "(Ljava/lang/Object;)V", *element)?;
                }
                self.alloc_stream(elements)
            }
            ("forEach" | "forEachOrdered", [JValue::Ref(Some(consumer))]) => {
                for element in elements {
                    self.call_functional(*consumer, "accept", "(Ljava/lang/Object;)V", element)?;
                }
                return Ok(Answered::Void);
            }
            ("count", []) => JValue::Long(i64::try_from(elements.len()).unwrap_or(i64::MAX)),
            ("findFirst" | "findAny", []) => {
                self.alloc_optional(elements.first().copied(), OptionalKind::Ref)
            }
            // `Stream.max`/`min(Comparator)` — the extreme by the comparator,
            // keeping the accumulator on ties (Java's `maxBy`/`minBy`).
            ("max" | "min", [JValue::Ref(Some(comparator))]) => {
                let want_max = method == "max";
                let mut best: Option<JValue> = None;
                for element in elements {
                    best = Some(match best {
                        None => element,
                        Some(current) => {
                            let ordering =
                                self.compare_with(element, current, Some(*comparator))?;
                            let take = if want_max { ordering > 0 } else { ordering < 0 };
                            if take { element } else { current }
                        }
                    });
                }
                self.alloc_optional(best, OptionalKind::Ref)
            }
            // `IntStream.max`/`min()` — natural int ordering.
            ("max" | "min", []) => {
                let want_max = method == "max";
                let mut best: Option<i32> = None;
                for element in &elements {
                    if let JValue::Int(n) = element {
                        best = Some(match best {
                            None => *n,
                            Some(current) if want_max => current.max(*n),
                            Some(current) => current.min(*n),
                        });
                    }
                }
                self.alloc_optional(best.map(JValue::Int), OptionalKind::Int)
            }
            // `IntStream.average()` — the mean as an OptionalDouble.
            ("average", []) => {
                let value = if elements.is_empty() {
                    None
                } else {
                    let sum: i64 = elements
                        .iter()
                        .map(|e| {
                            if let JValue::Int(n) = e {
                                i64::from(*n)
                            } else {
                                0
                            }
                        })
                        .sum();
                    #[allow(clippy::cast_precision_loss)]
                    let mean = sum as f64 / elements.len() as f64;
                    Some(JValue::Double(mean))
                };
                self.alloc_optional(value, OptionalKind::Double)
            }
            ("anyMatch" | "allMatch" | "noneMatch", [JValue::Ref(Some(pred))]) => {
                let mut any = false;
                let mut all = true;
                for element in elements {
                    if self.call_test(*pred, element)? {
                        any = true;
                    } else {
                        all = false;
                    }
                }
                let matched = match method {
                    "anyMatch" => any,
                    "allMatch" => all,
                    _ => !any,
                };
                JValue::Int(i32::from(matched))
            }
            ("collect", [JValue::Ref(Some(collector))]) => {
                self.stream_collect(elements, *collector)?
            }
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "Stream.{method}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// `stream.collect(collector)`: gather the elements per the collector recipe.
    fn stream_collect(
        &mut self,
        elements: Vec<JValue>,
        collector: HeapRef,
    ) -> Result<JValue, VmError> {
        use crate::value::{CollectorKind, HeapObject};
        let Some(HeapObject::Collector(kind)) = self.heap.get(collector) else {
            return Err(VmError::UnknownIntrinsic(String::from(
                "collect: not a Collector",
            )));
        };
        match kind.clone() {
            CollectorKind::ToList => Ok(JValue::Ref(Some(
                self.heap.alloc(HeapObject::ArrayList(elements)),
            ))),
            CollectorKind::ToSet => {
                let set = self
                    .heap
                    .alloc(HeapObject::HashSet(crate::map::JavaHashMap::new()));
                for element in elements {
                    self.set_add(set, element)?;
                }
                Ok(JValue::Ref(Some(set)))
            }
            CollectorKind::Joining {
                delimiter,
                prefix,
                suffix,
            } => {
                let mut parts = Vec::with_capacity(elements.len());
                for element in elements {
                    parts.push(self.string_value_of(element, 0)?);
                }
                let text = format!("{prefix}{}{suffix}", parts.join(&delimiter));
                Ok(JValue::Ref(Some(self.heap.alloc_string(&text))))
            }
        }
    }

    /// The elements of any collection caturra can walk, in iteration order: an
    /// `ArrayList` (or its unmodifiable view), a `HashSet`, a `TreeSet`, or a
    /// map's `keySet()`/`values()` view. Backs `addAll`/`removeAll`/`retainAll`
    /// and the collection-copy constructors.
    fn collection_elements(&self, reference: HeapRef) -> Vec<JValue> {
        use crate::value::HeapObject;
        match self.heap.get(reference) {
            Some(
                HeapObject::ArrayList(_)
                | HeapObject::LinkedList(_)
                | HeapObject::ArrayDeque(_)
                | HeapObject::Stack(_)
                | HeapObject::UnmodifiableList(_),
            ) => self.list_items(reference),
            Some(HeapObject::TreeSet { values, .. }) => values.clone(),
            Some(HeapObject::PriorityQueue { heap, .. }) => heap.clone(),
            Some(HeapObject::HashSet(entries)) => entries
                .entries_in_order()
                .into_iter()
                .map(|(element, _)| element)
                .collect(),
            Some(HeapObject::MapView { map, kind }) => {
                let (map, kind) = (*map, *kind);
                self.map_entries(map)
                    .into_iter()
                    .map(move |(key, value)| match kind {
                        MapViewKind::Values => value,
                        _ => key,
                    })
                    .collect()
            }
            // An unmodifiable set view walks its backing set.
            Some(HeapObject::UnmodifiableSet(inner)) => self.collection_elements(*inner),
            _ => Vec::new(),
        }
    }

    fn map_value_at(&self, map: HeapRef, at: usize) -> JValue {
        match self.heap.get(map) {
            Some(
                crate::value::HeapObject::HashMap(entries)
                | crate::value::HeapObject::HashSet(entries),
            ) => entries.value_at(at),
            Some(crate::value::HeapObject::TreeMap { entries, .. }) => {
                entries.get(at).map_or(JValue::NULL, |(_, value)| *value)
            }
            _ => JValue::NULL,
        }
    }

    /// Overwrite the value at a position, returning the previous one.
    fn map_set_value_at(&mut self, map: HeapRef, at: usize, value: JValue) -> JValue {
        match self.heap.get_mut(map) {
            Some(crate::value::HeapObject::HashMap(entries)) => entries.set_value_at(at, value),
            Some(crate::value::HeapObject::TreeMap { entries, .. }) if at < entries.len() => {
                std::mem::replace(&mut entries[at].1, value)
            }
            _ => JValue::NULL,
        }
    }

    /// `floorKey`/`ceilingKey`/`lowerKey`/`higherKey` over the sorted keys.
    fn tree_map_navigate_key(
        &mut self,
        map: HeapRef,
        which: &str,
        probe: JValue,
    ) -> Result<JValue, VmError> {
        let comparator = self.tree_map_comparator(map);
        let mut best = JValue::NULL;
        for (key, _) in self.map_entries(map) {
            let ordering = self.compare_with(key, probe, comparator)?;
            let matches = match which {
                "floorKey" => ordering <= 0,
                "lowerKey" => ordering < 0,
                "ceilingKey" => ordering >= 0,
                "higherKey" => ordering > 0,
                _ => false,
            };
            if matches {
                // floor/lower want the greatest match; ceiling/higher the least.
                if which == "ceilingKey" || which == "higherKey" {
                    return Ok(key);
                }
                best = key;
            }
        }
        Ok(best)
    }

    /// The key/value pair at a position in a map's iteration order.
    fn map_entry_at(&self, map: HeapRef, at: usize) -> Option<(JValue, JValue)> {
        match self.heap.get(map) {
            Some(
                crate::value::HeapObject::HashMap(entries)
                | crate::value::HeapObject::HashSet(entries),
            ) => entries.entry_at(at),
            Some(crate::value::HeapObject::TreeMap { entries, .. }) => entries.get(at).copied(),
            _ => None,
        }
    }

    fn map_remove_at(&mut self, map: HeapRef, at: usize) -> JValue {
        match self.heap.get_mut(map) {
            Some(
                crate::value::HeapObject::HashMap(entries)
                | crate::value::HeapObject::HashSet(entries),
            ) => entries.remove_at(at),
            Some(crate::value::HeapObject::TreeMap { entries, .. }) => {
                if at < entries.len() {
                    entries.remove(at).1
                } else {
                    JValue::NULL
                }
            }
            _ => JValue::NULL,
        }
    }

    fn alloc_map_view(&mut self, map: HeapRef, kind: MapViewKind) -> JValue {
        JValue::Ref(Some(
            self.heap
                .alloc(crate::value::HeapObject::MapView { map, kind }),
        ))
    }

    /// `keySet()` / `values()` / `entrySet()`. These are views, not copies, so
    /// every method reads through to the map as it stands now.
    ///
    /// `__get(int)` is caturra's own: the enhanced-for loop compiles to an
    /// index loop over it, because caturra has no iterators. Mutating the map
    /// inside such a loop silently sees the new contents, where a real JDK
    /// would throw `ConcurrentModificationException`.
    #[allow(clippy::too_many_lines)] // one method table
    fn map_view_intrinsic(
        &mut self,
        map: HeapRef,
        kind: MapViewKind,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        use crate::value::HeapObject;
        let result = match (method_name, args) {
            ("size", []) => JValue::Int(i32::try_from(self.map_len(map)).unwrap_or(i32::MAX)),
            ("isEmpty", []) => JValue::Int(i32::from(self.map_len(map) == 0)),
            ("contains", [probe]) => {
                let found = match kind {
                    MapViewKind::Keys => self.map_find(map, *probe)?.is_some(),
                    MapViewKind::Values => self.map_contains_value(map, *probe)?,
                    // `entrySet().contains(entry)` would need entry equality;
                    // the compiler does not offer it.
                    MapViewKind::Entries => false,
                };
                JValue::Int(i32::from(found))
            }
            ("forEach", [JValue::Ref(Some(consumer))]) => {
                self.view_for_each(map, kind, *consumer)?;
                return Ok(Answered::Void);
            }
            // The element at a position in the map's iteration order.
            ("__get", [JValue::Int(position)]) => {
                let position = usize::try_from(*position).unwrap_or(usize::MAX);
                let Some((key, value)) = self.map_entry_at(map, position) else {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.IndexOutOfBoundsException: Index {position} out of bounds"
                    )));
                };
                match kind {
                    MapViewKind::Keys => key,
                    MapViewKind::Values => value,
                    MapViewKind::Entries => {
                        JValue::Ref(Some(self.heap.alloc(HeapObject::MapEntry { map, key })))
                    }
                }
            }
            // A map's views do not support `add` — Java throws here, so must we
            // (accepting it silently would be the dangerous direction).
            ("add" | "addAll", [_]) => {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.UnsupportedOperationException",
                )));
            }
            // `remove`/`clear` on a view DO write through to the map, as Java's.
            ("clear", []) => {
                if let Some(HeapObject::HashMap(entries)) = self.heap.get_mut(map) {
                    entries.clear();
                }
                return Ok(Answered::Void);
            }
            ("remove", [element]) => {
                let removed = match kind {
                    MapViewKind::Keys => match self.map_find(map, *element)? {
                        Some(at) => {
                            self.map_remove_at(map, at);
                            true
                        }
                        None => false,
                    },
                    // `values().remove(o)` drops the first entry mapping to `o`.
                    MapViewKind::Values => {
                        let mut removed = false;
                        for (key, value) in self.map_entries(map) {
                            if self.java_equals(*element, value)? {
                                if let Some(at) = self.map_find(map, key)? {
                                    self.map_remove_at(map, at);
                                }
                                removed = true;
                                break;
                            }
                        }
                        removed
                    }
                    MapViewKind::Entries => {
                        return Err(VmError::UnknownIntrinsic(format!(
                            "Map view.{method_name}{descriptor}"
                        )));
                    }
                };
                JValue::Int(i32::from(removed))
            }
            ("removeAll", [JValue::Ref(Some(source))]) if kind == MapViewKind::Keys => {
                let mut changed = false;
                for element in self.collection_elements(*source) {
                    if let Some(at) = self.map_find(map, element)? {
                        self.map_remove_at(map, at);
                        changed = true;
                    }
                }
                JValue::Int(i32::from(changed))
            }
            ("retainAll", [JValue::Ref(Some(source))]) if kind == MapViewKind::Keys => {
                let keep = self.collection_elements(*source);
                let mut changed = false;
                for (key, _) in self.map_entries(map) {
                    let mut found = false;
                    for candidate in &keep {
                        if key == *candidate || self.java_equals(key, *candidate)? {
                            found = true;
                            break;
                        }
                    }
                    if !found && let Some(at) = self.map_find(map, key)? {
                        self.map_remove_at(map, at);
                        changed = true;
                    }
                }
                JValue::Int(i32::from(changed))
            }
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "Map view.{method_name}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// One `Map.Entry` from an `entrySet()`. Java's entry is a view onto its
    /// map, so `getValue` sees a later `put` and `setValue` writes through.
    fn map_entry_intrinsic(
        &mut self,
        map: HeapRef,
        key: JValue,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<Answered, VmError> {
        let result = match (method_name, args) {
            ("getKey", []) => key,
            ("getValue", []) => self.map_entry_value(map, key)?,
            ("setValue", [value]) => self.map_put(map, key, *value)?,
            // `Map.Entry.hashCode` is the key's hash XOR the value's.
            ("hashCode", []) => {
                let value = self.map_entry_value(map, key)?;
                JValue::Int(self.java_hash_code(key)? ^ self.java_hash_code(value)?)
            }
            _ => {
                return Err(VmError::UnknownIntrinsic(format!(
                    "Map.Entry.{method_name}{descriptor}"
                )));
            }
        };
        Ok(Answered::Value(result))
    }

    /// Dispatch an instance method on a user-defined object, with the
    /// default `toString` when the class doesn't define one. Returns
    /// either a frame to push or an inline result value.
    #[allow(clippy::too_many_lines)] // one dispatch resolution path
    fn user_virtual_dispatch(
        &mut self,
        receiver: HeapRef,
        instance_class: &str,
        method_name: &str,
        descriptor: &str,
        args: &[JValue],
    ) -> Result<UserDispatch<'run>, VmError> {
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        if !classes.contains_key(instance_class) {
            return Err(VmError::MalformedClass {
                name: instance_class.to_owned(),
                reason: String::from("instance of an unloaded class"),
            });
        }
        // Virtual dispatch: search the instance's class, then up the
        // superclass chain (skipping abstract declarations).
        let mut found: Option<(&'run ClassFile, &'run MethodInfo)> = None;
        let mut current = classes.get(instance_class);
        let mut steps = 0usize;
        while let Some(candidate) = current {
            steps += 1;
            if steps > classes.len() + 1 {
                break;
            }
            if let Some(method) = candidate.methods.iter().find(|m| {
                !m.access_flags
                    .contains(caturra_classfile::MethodAccessFlags::STATIC)
                    && !m
                        .access_flags
                        .contains(caturra_classfile::MethodAccessFlags::ABSTRACT)
                    && candidate.constant_pool.get_utf8(m.name_index) == Some(method_name)
                    && candidate.constant_pool.get_utf8(m.descriptor_index) == Some(descriptor)
            }) {
                found = Some((candidate, method));
                break;
            }
            current = candidate
                .constant_pool
                .get_class_name(candidate.super_class)
                .and_then(|super_name| classes.get(super_name));
        }
        // No override on the superclass chain: fall back to an
        // inherited interface default method (JLS §9.4). Search the
        // implemented interfaces breadth-first, including
        // super-interfaces.
        if found.is_none() {
            let mut queue: Vec<&'run ClassFile> = collect_interfaces(classes, instance_class);
            let mut seen = 0usize;
            while let Some(iface) = queue.pop() {
                seen += 1;
                if seen > classes.len() * 2 + 2 {
                    break;
                }
                if let Some(method) = iface.methods.iter().find(|m| {
                    !m.access_flags
                        .contains(caturra_classfile::MethodAccessFlags::STATIC)
                        && !m
                            .access_flags
                            .contains(caturra_classfile::MethodAccessFlags::ABSTRACT)
                        && iface.constant_pool.get_utf8(m.name_index) == Some(method_name)
                        && iface.constant_pool.get_utf8(m.descriptor_index) == Some(descriptor)
                }) {
                    found = Some((iface, method));
                    break;
                }
                // Enqueue super-interfaces of this interface.
                for index in &iface.interfaces {
                    if let Some(name) = iface.constant_pool.get_class_name(*index)
                        && let Some(parent) = classes.get(name)
                    {
                        queue.push(parent);
                    }
                }
            }
        }
        // Erasure bridge: a call through a generic interface uses the
        // erased descriptor (`compareTo(Object)`), but the class holds
        // the specific one (`compareTo(Card)`). Match by name and
        // argument count when the exact descriptor is not present.
        if found.is_none() {
            let want_args = descriptor_arg_count(descriptor);
            let mut current = classes.get(instance_class);
            let mut steps = 0usize;
            while let Some(candidate) = current {
                steps += 1;
                if steps > classes.len() + 1 {
                    break;
                }
                if let Some(method) = candidate.methods.iter().find(|m| {
                    !m.access_flags
                        .contains(caturra_classfile::MethodAccessFlags::STATIC)
                        && !m
                            .access_flags
                            .contains(caturra_classfile::MethodAccessFlags::ABSTRACT)
                        && candidate.constant_pool.get_utf8(m.name_index) == Some(method_name)
                        && candidate
                            .constant_pool
                            .get_utf8(m.descriptor_index)
                            .and_then(descriptor_arg_count)
                            == want_args
                }) {
                    found = Some((candidate, method));
                    break;
                }
                current = candidate
                    .constant_pool
                    .get_class_name(candidate.super_class)
                    .and_then(|super_name| classes.get(super_name));
            }
        }
        let Some((class, method)) = found else {
            // Throwable-descended classes inherit getMessage/toString.
            if self.instance_is_throwable(instance_class)
                && (method_name == "getMessage" || method_name == "toString")
                && descriptor == "()Ljava/lang/String;"
            {
                let message = match self.heap.get(receiver) {
                    Some(crate::value::HeapObject::Instance { fields, .. }) => {
                        fields.get("__message").and_then(|value| match value {
                            JValue::Ref(Some(text)) => self.heap.string_text(*text),
                            _ => None,
                        })
                    }
                    _ => None,
                };
                if method_name == "getMessage" {
                    return Ok(UserDispatch::Value(Some(match message {
                        Some(text) => JValue::Ref(Some(self.heap.alloc_string(&text))),
                        None => JValue::NULL,
                    })));
                }
                let text = match message {
                    Some(message) => format!("{instance_class}: {message}"),
                    None => instance_class.to_owned(),
                };
                let reference = self.heap.alloc_string(&text);
                return Ok(UserDispatch::Value(Some(JValue::Ref(Some(reference)))));
            }
            // Object.toString() default: "ClassName@<hex>".
            if method_name == "toString" && descriptor == "()Ljava/lang/String;" {
                let text = format!("{instance_class}@{receiver:x}");
                let reference = self.heap.alloc_string(&text);
                return Ok(UserDispatch::Value(Some(JValue::Ref(Some(reference)))));
            }
            // Object.getClass(): a reflection `Class` handle.
            if method_name == "getClass" && descriptor == "()Ljava/lang/Class;" {
                let reference = self.heap.alloc(crate::value::HeapObject::Class {
                    name: instance_class.to_owned(),
                });
                return Ok(UserDispatch::Value(Some(JValue::Ref(Some(reference)))));
            }
            // Object.equals(Object): reference identity when not overridden.
            if method_name == "equals" && descriptor == "(Ljava/lang/Object;)Z" {
                let equal = matches!(args.first(), Some(JValue::Ref(Some(r))) if *r == receiver);
                return Ok(UserDispatch::Value(Some(JValue::Int(i32::from(equal)))));
            }
            // Object.hashCode(): identity hash.
            if method_name == "hashCode" && descriptor == "()I" {
                return Ok(UserDispatch::Value(Some(JValue::Int(i32::from_ne_bytes(
                    receiver.to_ne_bytes(),
                )))));
            }
            return Err(VmError::UnknownIntrinsic(format!(
                "{instance_class}.{method_name}{descriptor}"
            )));
        };

        // The instance's class chain was initialized when the object
        // was constructed (`new` runs <clinit> first).
        let widths = descriptor_arg_widths(descriptor).ok_or_else(|| VmError::MalformedClass {
            name: instance_class.to_owned(),
            reason: format!("bad descriptor {descriptor}"),
        })?;
        let mut locals = Vec::with_capacity(1 + args.len());
        locals.push(JValue::Ref(Some(receiver)));
        for (value, width) in args.iter().zip(&widths) {
            locals.push(*value);
            if *width == 2 {
                locals.push(JValue::Int(0));
            }
        }
        Ok(UserDispatch::Call(self.make_frame(class, method, locals)?))
    }

    /// Recursively allocate a (possibly partial) multi-dimensional
    /// array described by `descriptor` (e.g. `[[I`) with the given
    /// leading dimension counts.
    fn alloc_multi_array(
        &mut self,
        descriptor: &str,
        counts: &[i32],
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<HeapRef, VmError> {
        let (first, rest_counts) = counts
            .split_first()
            .ok_or_else(|| malformed(String::from("multianewarray with zero dimensions")))?;
        let length = check_array_size(*first)?;
        let element_descriptor = descriptor
            .strip_prefix('[')
            .ok_or_else(|| malformed(format!("bad array descriptor {descriptor}")))?;

        if rest_counts.is_empty() {
            use crate::value::{HeapObject, IntKind};
            let object = match element_descriptor {
                "I" => HeapObject::IntArray(IntKind::Int, vec![0; length]),
                "Z" => HeapObject::IntArray(IntKind::Boolean, vec![0; length]),
                "C" => HeapObject::IntArray(IntKind::Char, vec![0; length]),
                "D" => HeapObject::DoubleArray(vec![0.0; length]),
                "J" => HeapObject::LongArray(vec![0; length]),
                "F" => HeapObject::FloatArray(vec![0.0; length]),
                "S" => HeapObject::ShortArray(vec![0; length]),
                "B" => HeapObject::ByteArray(vec![0; length]),
                _ => HeapObject::RefArray(
                    format!("[{element_descriptor}"),
                    vec![JValue::NULL; length],
                ),
            };
            return Ok(self.heap.alloc(object));
        }

        let mut rows = Vec::with_capacity(length);
        for _ in 0..length {
            let row = self.alloc_multi_array(element_descriptor, rest_counts, malformed)?;
            rows.push(JValue::Ref(Some(row)));
        }
        Ok(self.heap.alloc(crate::value::HeapObject::RefArray(
            descriptor.to_owned(),
            rows,
        )))
    }

    /// `invokestatic`: a user-defined static method becomes a frame to
    /// push (locals built from the popped arguments — doubles occupy
    /// two slots, JVMS §2.6.1); intrinsic statics run inline and push
    /// their result onto the caller's stack.
    fn invoke_static(
        &mut self,
        class_name: &str,
        method_name: &str,
        descriptor: &str,
        widths: &[u16],
        args: &[JValue],
        frame: &mut Frame<'run>,
    ) -> Result<Option<Frame<'run>>, VmError> {
        // `Type.class` literal (compiler-lowered): a `Class` handle for any
        // type name, including primitives — never throws.
        if class_name == "java/lang/Class" && method_name == "__forType" {
            let name = match args.first() {
                Some(JValue::Ref(Some(reference))) => {
                    self.heap.string_text(*reference).unwrap_or_default()
                }
                _ => String::new(),
            };
            let reference = self.heap.alloc(crate::value::HeapObject::Class { name });
            frame.stack.push(JValue::Ref(Some(reference)));
            return Ok(None);
        }
        // `Collections.sort(list)` — natural-ordering sort. Done natively
        // because caturra stores unboxed primitives in a list, so a bundled Java
        // version could not compare them (`get` yields a bare `int`, not a
        // `Comparable`). Strings compare by UTF-16 code units (Java semantics),
        // and a user class compares with its own `compareTo`.
        if self.collections_static_intrinsic(frame, class_name, method_name, args)? {
            return Ok(None);
        }
        if self.arrays_static_intrinsic(frame, class_name, method_name, args)? {
            return Ok(None);
        }
        if self.collectors_static_intrinsic(frame, class_name, method_name, args)? {
            return Ok(None);
        }
        if self.comparator_static_intrinsic(frame, class_name, method_name, args)? {
            return Ok(None);
        }
        // `IntStream.range(a, b)` / `rangeClosed(a, b)` — a stream of consecutive
        // ints (the VM models an IntStream as a Stream of unboxed ints).
        if (class_name == "IntStream" || class_name == "java/util/stream/IntStream")
            && let ("range" | "rangeClosed", [JValue::Int(from), JValue::Int(to)]) =
                (method_name, args)
        {
            let end = if method_name == "rangeClosed" {
                to.saturating_add(1)
            } else {
                *to
            };
            let ints: Vec<JValue> = (*from..end).map(JValue::Int).collect();
            let stream = self.heap.alloc(crate::value::HeapObject::Stream(ints));
            frame.stack.push(JValue::Ref(Some(stream)));
            return Ok(None);
        }
        // `Optional.of(x)` / `empty()` / `ofNullable(x)`.
        if class_name == "Optional" || class_name == "java/util/Optional" {
            let value = match (method_name, args) {
                ("empty", []) => Some(None),
                ("of", [v]) => Some(Some(*v)),
                ("ofNullable", [v]) => Some((*v != JValue::NULL).then_some(*v)),
                _ => None,
            };
            if let Some(value) = value {
                let optional = self.alloc_optional(value, crate::value::OptionalKind::Ref);
                frame.stack.push(optional);
                return Ok(None);
            }
        }
        // `Class.forName(name)` — a reflection handle for a loaded class.
        // Needs the class table, which the intrinsic layer lacks.
        if class_name == "java/lang/Class" && method_name == "forName" {
            let name = match args.first() {
                Some(JValue::Ref(Some(reference))) => self.heap.string_text(*reference),
                _ => None,
            };
            let value = match name {
                Some(name) if self.classes.contains_key(&name) => {
                    let reference = self.heap.alloc(crate::value::HeapObject::Class { name });
                    JValue::Ref(Some(reference))
                }
                other => {
                    return Err(VmError::UncaughtException(format!(
                        "java.lang.ClassNotFoundException: {}",
                        other.unwrap_or_default()
                    )));
                }
            };
            frame.stack.push(value);
            return Ok(None);
        }
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let Some(class) = classes.get(class_name) else {
            // Intrinsic statics: Math.*, Integer.parseInt, ...
            let result = intrinsics::invoke_static(
                &mut self.heap,
                &mut self.rng,
                self.console,
                self.vfs,
                class_name,
                method_name,
                descriptor,
                args,
            )?;
            if let Some(value) = result {
                frame.stack.push(value);
            }
            return Ok(None);
        };
        // The caller ran the initialization check before popping args.
        // JVMS 5.4.3.3: resolution searches the named class, then its
        // superclasses. `Derived.who()` and `derived.who()` both name
        // `Derived` while `who` is declared in `Base`.
        let (class, method) = resolve_static_method(classes, class, method_name, descriptor)
            .ok_or_else(|| VmError::MalformedClass {
                name: class_name.to_owned(),
                reason: format!("no static method {method_name}{descriptor}"),
            })?;

        let mut locals = Vec::with_capacity(args.len());
        for (value, width) in args.iter().zip(widths) {
            locals.push(*value);
            if *width == 2 {
                locals.push(JValue::Int(0)); // padding for the wide slot
            }
        }
        Ok(Some(self.make_frame(class, method, locals)?))
    }

    /// `invokevirtual`: resolve the method ref, pop arguments and
    /// receiver, and dispatch — user methods become a frame to push,
    /// intrinsics run inline.
    #[allow(clippy::too_many_lines)] // one dispatch path with several fast-paths
    fn invoke_virtual_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame<'run>,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<Option<Frame<'run>>, VmError> {
        let (target_class, method_name, descriptor) = class
            .constant_pool
            .get_member_ref(index)
            .map(|(c, m, d)| (c.to_owned(), m.to_owned(), d.to_owned()))
            .ok_or_else(|| malformed(format!("bad method ref at pool {index}")))?;

        let arg_count = descriptor_arg_count(&descriptor)
            .ok_or_else(|| malformed(format!("bad descriptor {descriptor}")))?;
        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(frame.pop()?);
        }
        args.reverse();

        let receiver = match frame.pop()? {
            JValue::Ref(receiver) => receiver,
            // An unboxed primitive used as an Object receiver — e.g. `get()`
            // from an `ArrayList<Object>` (caturra stores primitives unboxed)
            // followed by `.equals(...)`/`.toString()`. Box it and dispatch on
            // the wrapper.
            primitive => Some(self.box_primitive_value(primitive)),
        };
        let Some(receiver) = receiver else {
            return Err(VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot invoke \
                 \"{target_class}.{method_name}\" because the receiver is null"
            )));
        };

        // An unmodifiable view refuses every mutator and forwards the rest to
        // the list it wraps, so `size`, `get` and `toString` read through.
        let receiver = if self.is_unmodifiable_list(receiver) {
            if LIST_MUTATORS.contains(&method_name.as_str()) {
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.UnsupportedOperationException",
                )));
            }
            self.backing_list(receiver)
        } else {
            receiver
        };

        // User-defined objects dispatch on the instance's actual class;
        // everything else is an intrinsic (PrintStream, StringBuilder).
        let instance_class = match self.heap.get(receiver) {
            Some(crate::value::HeapObject::Instance { class_name, .. }) => Some(class_name.clone()),
            _ => None,
        };
        let is_reflect = matches!(
            self.heap.get(receiver),
            Some(
                crate::value::HeapObject::Class { .. }
                    | crate::value::HeapObject::Field { .. }
                    | crate::value::HeapObject::Constructor { .. }
                    | crate::value::HeapObject::Method { .. }
                    | crate::value::HeapObject::ReflectType { .. }
            )
        );
        // `Constructor.newInstance` runs a constructor (a frame), so it can't
        // go through the value-returning reflect path.
        if method_name == "newInstance"
            && matches!(
                self.heap.get(receiver),
                Some(crate::value::HeapObject::Constructor { .. })
            )
        {
            return self.reflect_new_instance(receiver, &args, frame);
        }
        // `Method.invoke` runs a frame too, so it can't use the value path.
        if method_name == "invoke"
            && matches!(
                self.heap.get(receiver),
                Some(crate::value::HeapObject::Method { .. })
            )
        {
            return self.reflect_invoke(receiver, &args);
        }
        // `getClass()` on any object (library intrinsics included, e.g. a
        // String or a boxed Integer from a reflective `Object[]`).
        if method_name == "getClass" && descriptor == "()Ljava/lang/Class;" {
            let name = self.object_class_name(receiver);
            let reference = self.heap.alloc(crate::value::HeapObject::Class { name });
            frame.stack.push(JValue::Ref(Some(reference)));
            return Ok(None);
        }
        // `Object` methods on an array (arrays don't override them): identity
        // equals/hashCode and a default toString.
        if is_array_object(self.heap.get(receiver)) {
            match method_name.as_str() {
                "equals" => {
                    let equal =
                        matches!(args.first(), Some(JValue::Ref(Some(r))) if *r == receiver);
                    frame.stack.push(JValue::Int(i32::from(equal)));
                    return Ok(None);
                }
                "hashCode" => {
                    frame
                        .stack
                        .push(JValue::Int(intrinsics::identity_hash(receiver)));
                    return Ok(None);
                }
                "toString" => {
                    let text =
                        intrinsics::array_to_string(&self.heap, receiver).unwrap_or_default();
                    let reference = self.heap.alloc_string(&text);
                    frame.stack.push(JValue::Ref(Some(reference)));
                    return Ok(None);
                }
                _ => {}
            }
        }
        // Turning a value into text can call a user `toString()`, which needs
        // the interpreter; the intrinsic layer holds only the heap.
        if self.container_to_string(frame, receiver, &method_name, &descriptor)? {
            return Ok(None);
        }
        // Likewise for comparing elements or keys, which may call a user
        // `equals` and `hashCode`.
        let compared = match self.stream_dispatch(receiver, &method_name, &descriptor, &args)? {
            Answered::No => {
                match self.list_equality_intrinsic(receiver, &method_name, &descriptor, &args)? {
                    Answered::No => {
                        self.map_intrinsic(receiver, &method_name, &descriptor, &args)?
                    }
                    answered => answered,
                }
            }
            answered => answered,
        };
        match compared {
            Answered::No => {}
            Answered::Void => return Ok(None),
            Answered::Value(value) => {
                frame.stack.push(value);
                return Ok(None);
            }
        }
        let rendered = self.render_object_argument(receiver, &method_name, &descriptor, &args)?;
        let (descriptor, args) = match rendered {
            Some((string_form, text)) => (String::from(string_form), vec![text]),
            None => (descriptor, args),
        };

        let result = if let Some(instance_class) = instance_class {
            match self.user_virtual_dispatch(
                receiver,
                &instance_class,
                &method_name,
                &descriptor,
                &args,
            )? {
                UserDispatch::Call(callee) => return Ok(Some(callee)),
                UserDispatch::Value(value) => value,
            }
        } else if is_reflect {
            // Class/Field methods need the class table (superclass,
            // declared fields), which the intrinsic layer lacks.
            self.reflect_virtual(receiver, &method_name, &args)?
        } else {
            intrinsics::invoke_virtual(
                &mut self.heap,
                self.console,
                self.vfs,
                receiver,
                &target_class,
                &method_name,
                &descriptor,
                &args,
            )?
        };
        if let Some(value) = result {
            frame.stack.push(value);
        }
        Ok(None)
    }

    /// `java.lang.Class` / `java.lang.reflect.Field` methods — the
    /// structural, read-only reflection the curriculum uses. Reads the
    /// loaded [`ClassFile`] metadata; performs no invocation.
    /// The binary class name behind any heap object, for `getClass()`.
    fn object_class_name(&self, receiver: HeapRef) -> String {
        use crate::value::HeapObject;
        match self.heap.get(receiver) {
            Some(
                HeapObject::Instance { class_name, .. } | HeapObject::Boxed { class_name, .. },
            ) => class_name.clone(),
            Some(HeapObject::JavaString(_)) => String::from("java/lang/String"),
            Some(HeapObject::StringBuilder(_)) => String::from("java/lang/StringBuilder"),
            Some(HeapObject::ArrayList(_)) => String::from("java/util/ArrayList"),
            Some(HeapObject::LinkedList(_)) => String::from("java/util/LinkedList"),
            Some(HeapObject::ArrayDeque(_)) => String::from("java/util/ArrayDeque"),
            Some(HeapObject::Stack(_)) => String::from("java/util/Stack"),
            Some(HeapObject::HashSet(_)) => String::from("java/util/HashSet"),
            Some(HeapObject::TreeSet { .. }) => String::from("java/util/TreeSet"),
            Some(HeapObject::TreeMap { .. }) => String::from("java/util/TreeMap"),
            Some(HeapObject::PriorityQueue { .. }) => String::from("java/util/PriorityQueue"),
            Some(HeapObject::Optional { kind, .. }) => {
                format!("java/util/{}", kind.prefix())
            }
            _ if is_array_object(self.heap.get(receiver)) => {
                intrinsics::array_class_name(&self.heap, receiver).unwrap_or_default()
            }
            _ => String::from("java/lang/Object"),
        }
    }

    /// The `Class` names inside a `Class[]` argument (for `getConstructor`).
    fn class_array_names(&self, arg: Option<&JValue>) -> Vec<String> {
        let Some(JValue::Ref(Some(reference))) = arg else {
            return Vec::new();
        };
        let Some(crate::value::HeapObject::RefArray(_, elems)) = self.heap.get(*reference) else {
            return Vec::new();
        };
        elems
            .iter()
            .filter_map(|v| {
                let JValue::Ref(Some(cr)) = v else {
                    return None;
                };
                match self.heap.get(*cr) {
                    Some(crate::value::HeapObject::Class { name }) => Some(name.clone()),
                    _ => None,
                }
            })
            .collect()
    }

    /// `Constructor.newInstance(Object[])`: allocate an instance of the
    /// declaring class and run its `<init>` with the (unboxed) arguments.
    /// The new object is left on the caller's stack so it survives the void
    /// `<init>` return.
    fn reflect_new_instance(
        &mut self,
        receiver: HeapRef,
        args: &[JValue],
        frame: &mut Frame<'run>,
    ) -> Result<Option<Frame<'run>>, VmError> {
        use crate::value::HeapObject;
        let (declaring, descriptor) = match self.heap.get(receiver) {
            Some(HeapObject::Constructor {
                declaring,
                descriptor,
                ..
            }) => (declaring.clone(), descriptor.clone()),
            _ => return Ok(None),
        };
        let initargs: Vec<JValue> = match args.first() {
            Some(JValue::Ref(Some(r))) => match self.heap.get(*r) {
                Some(HeapObject::RefArray(_, elems)) => elems.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        let param_descs = parse_descriptor_params(&descriptor);
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let target = classes.get(&declaring).ok_or_else(|| {
            VmError::UncaughtException(format!("java.lang.InstantiationException: {declaring}"))
        })?;
        let ctor = target
            .methods
            .iter()
            .find(|m| {
                target.constant_pool.get_utf8(m.name_index) == Some("<init>")
                    && target.constant_pool.get_utf8(m.descriptor_index)
                        == Some(descriptor.as_str())
            })
            .ok_or_else(|| {
                VmError::UncaughtException(format!(
                    "java.lang.NoSuchMethodException: {declaring}.<init>"
                ))
            })?;
        let object = self.new_instance(&declaring);
        let instance = self.heap.alloc(object);
        let mut locals = vec![JValue::Ref(Some(instance))];
        for (arg, desc) in initargs.iter().zip(&param_descs) {
            locals.push(self.unbox_for(*arg, desc));
            if desc == "J" || desc == "D" {
                locals.push(JValue::Int(0)); // wide slot padding
            }
        }
        // Leave the new object on the caller's stack; the void `<init>` frame
        // returns nothing, so this becomes newInstance's result.
        frame.stack.push(JValue::Ref(Some(instance)));
        Ok(Some(self.make_frame(target, ctor, locals)?))
    }

    /// `Method.invoke(Object obj, Object... args)`: run the reflected method
    /// as a frame. For a static method `obj` is ignored; the (unboxed)
    /// arguments arrive as the `Object[]`. A primitive return is boxed on the
    /// way out so `invoke` hands back a wrapper (real Java semantics).
    fn reflect_invoke(
        &mut self,
        receiver: HeapRef,
        args: &[JValue],
    ) -> Result<Option<Frame<'run>>, VmError> {
        use crate::value::HeapObject;
        let (declaring, name, descriptor, access) = match self.heap.get(receiver) {
            Some(HeapObject::Method {
                declaring,
                name,
                descriptor,
                access,
            }) => (declaring.clone(), name.clone(), descriptor.clone(), *access),
            _ => return Ok(None),
        };
        let is_static = access & 0x0008 != 0;
        let call_args: Vec<JValue> = match args.get(1) {
            Some(JValue::Ref(Some(r))) => match self.heap.get(*r) {
                Some(HeapObject::RefArray(_, elems)) => elems.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        };
        let param_descs = parse_descriptor_params(&descriptor);
        let ret_desc = descriptor.rsplit(')').next().unwrap_or("V").to_owned();
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let target = classes.get(&declaring).ok_or_else(|| {
            VmError::UncaughtException(format!("java.lang.NoSuchMethodException: {declaring}"))
        })?;
        let target_method = target
            .methods
            .iter()
            .find(|m| {
                target.constant_pool.get_utf8(m.name_index) == Some(name.as_str())
                    && target.constant_pool.get_utf8(m.descriptor_index)
                        == Some(descriptor.as_str())
            })
            .ok_or_else(|| {
                VmError::UncaughtException(format!(
                    "java.lang.NoSuchMethodException: {declaring}.{name}"
                ))
            })?;
        let mut locals = Vec::new();
        if !is_static {
            locals.push(args.first().copied().unwrap_or(JValue::NULL));
        }
        for (arg, desc) in call_args.iter().zip(&param_descs) {
            locals.push(self.unbox_for(*arg, desc));
            if desc == "J" || desc == "D" {
                locals.push(JValue::Int(0)); // wide slot padding
            }
        }
        let mut new_frame = self.make_frame(target, target_method, locals)?;
        // Mark this as an `invoke` frame: its result is always an Object, so a
        // primitive return is boxed (Integer/Double/…) and a void return still
        // yields `null` (the caller's bytecode always expects one value back).
        new_frame.box_return_as = Some(ret_desc);
        Ok(Some(new_frame))
    }

    /// Box an unboxed primitive value into its wrapper on the heap, so it can
    /// be an `Object` receiver. Int-family values box as `Integer` (their exact
    /// wrapper is not recoverable from the erased value, but the wrapped value
    /// compares and prints correctly for the reflective/collection paths).
    fn box_primitive_value(&mut self, value: JValue) -> HeapRef {
        let class_name = match value {
            JValue::Long(_) => "java/lang/Long",
            JValue::Double(_) => "java/lang/Double",
            JValue::Float(_) => "java/lang/Float",
            _ => "java/lang/Integer",
        };
        self.heap.alloc(crate::value::HeapObject::Boxed {
            class_name: String::from(class_name),
            value,
        })
    }

    /// A stable merge sort, as `Collections.sort` is. It cannot use
    /// `slice::sort_by`, because comparing two user objects runs their
    /// `compareTo` — which may throw, and so must be able to fail.
    /// A stable merge sort by a comparator when `comparator` is `Some` (its
    /// `compare(a, b)` runs user Java), otherwise by natural ordering.
    fn merge_sort_by(
        &mut self,
        mut items: Vec<JValue>,
        comparator: Option<HeapRef>,
    ) -> Result<Vec<JValue>, VmError> {
        if items.len() <= 1 {
            return Ok(items);
        }
        let right = items.split_off(items.len() / 2);
        let left = self.merge_sort_by(items, comparator)?;
        let right = self.merge_sort_by(right, comparator)?;

        let mut merged = Vec::with_capacity(left.len() + right.len());
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            // `<= 0` takes from the left on a tie, which is what keeps the
            // sort stable.
            if self.compare_with(left[i], right[j], comparator)? <= 0 {
                merged.push(left[i]);
                i += 1;
            } else {
                merged.push(right[j]);
                j += 1;
            }
        }
        merged.extend_from_slice(&left[i..]);
        merged.extend_from_slice(&right[j..]);
        Ok(merged)
    }

    /// Compare two elements by a comparator's `compare` (user Java) or, with
    /// no comparator, by natural ordering.
    fn compare_with(
        &mut self,
        a: JValue,
        b: JValue,
        comparator: Option<HeapRef>,
    ) -> Result<i32, VmError> {
        match comparator {
            None => self.compare_for_sort(a, b),
            // A factory-built comparator (`Comparator.comparing`/`reversed`/…):
            // evaluate its recipe natively.
            Some(comparator)
                if matches!(
                    self.heap.get(comparator),
                    Some(crate::value::HeapObject::Comparator(_))
                ) =>
            {
                self.compare_by_spec(a, b, comparator)
            }
            Some(comparator) => {
                let Some(crate::value::HeapObject::Instance { class_name, .. }) =
                    self.heap.get(comparator)
                else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                let class_name = class_name.clone();
                let dispatched = self.user_virtual_dispatch(
                    comparator,
                    &class_name,
                    "compare",
                    "(Ljava/lang/Object;Ljava/lang/Object;)I",
                    &[a, b],
                )?;
                let result = match dispatched {
                    UserDispatch::Call(frame) => self.run_nested(frame)?,
                    UserDispatch::Value(value) => value,
                };
                Ok(match result {
                    Some(JValue::Int(n)) => n,
                    _ => 0,
                })
            }
        }
    }

    /// Compare two list elements the way `Collections.sort` does: a user
    /// object by its own `compareTo`, everything else natively.
    fn compare_for_sort(&mut self, a: JValue, b: JValue) -> Result<i32, VmError> {
        // A natural-ordering comparison calls `a.compareTo(b)`, so either one
        // being null throws — as sorting a list holding a null does in Java.
        if a == JValue::NULL || b == JValue::NULL {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        }
        if let JValue::Ref(Some(reference)) = a
            && let Some(crate::value::HeapObject::Instance { class_name, .. }) =
                self.heap.get(reference)
        {
            let class_name = class_name.clone();
            return self.call_compare_to(reference, &class_name, b);
        }
        Ok(match self.compare_values(&a, &b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        })
    }

    /// Evaluate a factory-built [`crate::value::ComparatorSpec`] on two values.
    fn compare_by_spec(
        &mut self,
        a: JValue,
        b: JValue,
        comparator: HeapRef,
    ) -> Result<i32, VmError> {
        use crate::value::{ComparatorSpec, HeapObject};
        let spec = match self.heap.get(comparator) {
            Some(HeapObject::Comparator(spec)) => spec.clone(),
            _ => return Ok(0),
        };
        match spec {
            ComparatorSpec::Natural => self.compare_for_sort(a, b),
            ComparatorSpec::ByKey(extractor) => {
                let key_a = self.call_apply(extractor, a)?;
                let key_b = self.call_apply(extractor, b)?;
                self.compare_for_sort(key_a, key_b)
            }
            ComparatorSpec::Reversed(inner) => Ok(-self.compare_with(a, b, Some(inner))?),
            ComparatorSpec::Then(first, second) => {
                let primary = self.compare_with(a, b, Some(first))?;
                if primary != 0 {
                    Ok(primary)
                } else {
                    self.compare_with(a, b, Some(second))
                }
            }
        }
    }

    /// `map.forEach(consumer)`: call `accept(key, value)` on every entry, in
    /// the map's iteration order. The consumer is a synthesized lambda class,
    /// so this runs user Java from native code — the machinery `toString` and
    /// `compareTo` already use.
    ///
    /// The entries are snapshotted first: a lambda that mutates the map would
    /// otherwise invalidate the positions mid-walk. Java throws
    /// `ConcurrentModificationException` there; caturra walks the snapshot,
    /// the same deviation its for-each over a map already has.
    fn map_for_each(&mut self, receiver: HeapRef, consumer: HeapRef) -> Result<(), VmError> {
        let entries = self.map_entries(receiver);
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(consumer)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        for (key, value) in entries {
            let dispatched = self.user_virtual_dispatch(
                consumer,
                &class_name,
                "accept",
                "(Ljava/lang/Object;Ljava/lang/Object;)V",
                &[key, value],
            )?;
            if let UserDispatch::Call(frame) = dispatched {
                self.run_nested(frame)?;
            }
        }
        Ok(())
    }

    /// `list.forEach(consumer)`: call `accept(element)` on each element in
    /// order, running the synthesized lambda class through the nested-call
    /// machinery. The elements are snapshotted first (a `forEach` that mutates
    /// the list is a `ConcurrentModificationException` in Java; caturra walks
    /// the snapshot, its existing deviation).
    fn list_for_each(&mut self, receiver: HeapRef, consumer: HeapRef) -> Result<(), VmError> {
        let items = self.list_items(receiver);
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(consumer)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        for element in items {
            let dispatched = self.user_virtual_dispatch(
                consumer,
                &class_name,
                "accept",
                "(Ljava/lang/Object;)V",
                &[element],
            )?;
            if let UserDispatch::Call(frame) = dispatched {
                self.run_nested(frame)?;
            }
        }
        Ok(())
    }

    /// `list.removeIf(predicate)`: keep only the elements the predicate
    /// rejects, in order, and report whether any were removed.
    fn list_remove_if(&mut self, receiver: HeapRef, predicate: HeapRef) -> Result<bool, VmError> {
        let items = self.list_items(receiver);
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(predicate)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        let mut kept: Vec<JValue> = Vec::with_capacity(items.len());
        for element in items {
            let dispatched = self.user_virtual_dispatch(
                predicate,
                &class_name,
                "test",
                "(Ljava/lang/Object;)Z",
                &[element],
            )?;
            let result = match dispatched {
                UserDispatch::Call(frame) => self.run_nested(frame)?,
                UserDispatch::Value(value) => value,
            };
            let drop = result != Some(JValue::Int(0)) && result.is_some();
            if !drop {
                kept.push(element);
            }
        }
        let removed = kept.len() != self.list_items(receiver).len();
        if let Some(list) = self.heap.list_values_mut(receiver) {
            *list = kept;
        }
        Ok(removed)
    }

    /// `list.replaceAll(operator)`: replace each element with `apply(element)`,
    /// in order. The operator's erased SAM returns `Object`, so the result is
    /// already a `JValue` the list stores directly (boxing is a no-op here).
    fn list_replace_all(&mut self, receiver: HeapRef, operator: HeapRef) -> Result<(), VmError> {
        let items = self.list_items(receiver);
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(operator)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        let mut replaced: Vec<JValue> = Vec::with_capacity(items.len());
        for element in items {
            let dispatched = self.user_virtual_dispatch(
                operator,
                &class_name,
                "apply",
                "(Ljava/lang/Object;)Ljava/lang/Object;",
                &[element],
            )?;
            let result = match dispatched {
                UserDispatch::Call(frame) => self.run_nested(frame)?,
                UserDispatch::Value(value) => value,
            };
            // The erased `apply` returns `Object`, so a primitive result comes
            // back boxed. A caturra list stores wrappers unboxed, so unbox it;
            // a String or user object stays a reference.
            let stored = match result {
                Some(JValue::Ref(Some(r))) => match self.heap.get(r) {
                    Some(crate::value::HeapObject::Boxed { value, .. }) => *value,
                    _ => JValue::Ref(Some(r)),
                },
                Some(value) => value,
                None => JValue::NULL,
            };
            replaced.push(stored);
        }
        if let Some(list) = self.heap.list_values_mut(receiver) {
            *list = replaced;
        }
        Ok(())
    }

    /// `keySet()/values()/entrySet().forEach(consumer)`: call `accept` on each
    /// view element in the map's iteration order — a key, a value, or a live
    /// `Map.Entry`. Runs user Java through the nested-call path.
    fn view_for_each(
        &mut self,
        map: HeapRef,
        kind: MapViewKind,
        consumer: HeapRef,
    ) -> Result<(), VmError> {
        let entries = self.map_entries(map);
        let Some(crate::value::HeapObject::Instance { class_name, .. }) = self.heap.get(consumer)
        else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let class_name = class_name.clone();
        for (key, value) in entries {
            let element = match kind {
                MapViewKind::Keys => key,
                MapViewKind::Values => value,
                MapViewKind::Entries => JValue::Ref(Some(
                    self.heap
                        .alloc(crate::value::HeapObject::MapEntry { map, key }),
                )),
            };
            let dispatched = self.user_virtual_dispatch(
                consumer,
                &class_name,
                "accept",
                "(Ljava/lang/Object;)V",
                &[element],
            )?;
            if let UserDispatch::Call(frame) = dispatched {
                self.run_nested(frame)?;
            }
        }
        Ok(())
    }

    /// `a.compareTo(b)` on a user object. The erased descriptor finds a
    /// `compareTo(Card)` through the dispatch bridge; a class that declares
    /// none is not `Comparable`, which is what Java complains about.
    fn call_compare_to(
        &mut self,
        receiver: HeapRef,
        class_name: &str,
        other: JValue,
    ) -> Result<i32, VmError> {
        let dispatched = self.user_virtual_dispatch(
            receiver,
            class_name,
            "compareTo",
            "(Ljava/lang/Object;)I",
            &[other],
        );
        let returned = match dispatched {
            Ok(UserDispatch::Call(frame)) => self.run_nested(frame)?,
            Ok(UserDispatch::Value(value)) => value,
            Err(VmError::UnknownIntrinsic(_)) => {
                return Err(VmError::UncaughtException(format!(
                    "java.lang.ClassCastException: class {class_name} cannot be cast to \
                     class java.lang.Comparable"
                )));
            }
            Err(other) => return Err(other),
        };
        match returned {
            Some(JValue::Int(order)) => Ok(order),
            _ => Err(VmError::UnknownIntrinsic(format!(
                "{class_name}.compareTo did not return an int"
            ))),
        }
    }

    /// Natural-ordering comparison of two list elements for `Collections.sort`:
    /// numbers by value, strings by UTF-16 code units, wrappers by their
    /// primitive. Incomparable pairs are treated as equal (stable no-op).
    fn compare_values(&self, a: &JValue, b: &JValue) -> std::cmp::Ordering {
        use crate::value::HeapObject;
        use std::cmp::Ordering;
        match (a, b) {
            (JValue::Int(x), JValue::Int(y)) => x.cmp(y),
            (JValue::Long(x), JValue::Long(y)) => x.cmp(y),
            (JValue::Double(x), JValue::Double(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
            (JValue::Float(x), JValue::Float(y)) => x.partial_cmp(y).unwrap_or(Ordering::Equal),
            (JValue::Ref(Some(ra)), JValue::Ref(Some(rb))) => {
                match (self.heap.get(*ra), self.heap.get(*rb)) {
                    (Some(HeapObject::JavaString(sa)), Some(HeapObject::JavaString(sb))) => {
                        sa.cmp(sb)
                    }
                    (
                        Some(HeapObject::Boxed { value: va, .. }),
                        Some(HeapObject::Boxed { value: vb, .. }),
                    ) => self.compare_values(&va.clone(), &vb.clone()),
                    _ => Ordering::Equal,
                }
            }
            _ => Ordering::Equal,
        }
    }

    /// Unbox `value` when the target parameter descriptor is a primitive.
    fn unbox_for(&self, value: JValue, param_desc: &str) -> JValue {
        if matches!(param_desc, "I" | "S" | "B" | "C" | "Z" | "J" | "D" | "F")
            && let JValue::Ref(Some(reference)) = value
            && let Some(crate::value::HeapObject::Boxed { value: inner, .. }) =
                self.heap.get(reference)
        {
            return *inner;
        }
        value
    }

    /// Whether `sub` is `sup` or a (transitive) subclass of it, walking the
    /// loaded class-file superclass chain (stops at the first library class).
    fn class_is_subtype(&self, sub: &str, sup: &str) -> bool {
        let mut current = sub.to_owned();
        loop {
            if current == sup {
                return true;
            }
            let Some(class) = self.classes.get(&current) else {
                return false;
            };
            let Some(super_name) = class.constant_pool.get_class_name(class.super_class) else {
                return false;
            };
            if super_name.contains('/') {
                return super_name == sup; // reached java/lang/Object etc.
            }
            current = super_name.to_owned();
        }
    }

    #[allow(clippy::too_many_lines)]
    fn reflect_virtual(
        &mut self,
        receiver: HeapRef,
        method: &str,
        args: &[JValue],
    ) -> Result<Option<JValue>, VmError> {
        use crate::value::HeapObject;
        match self.heap.get(receiver) {
            Some(HeapObject::Class { name }) => {
                let name = name.clone();
                match method {
                    // `getName` is the binary name (qualified for library types,
                    // e.g. `java.util.ArrayList`; a user class in the default
                    // package is already just its simple name). `getSimpleName`
                    // always drops the package.
                    // An array's binary name spells its element class with
                    // dots (`[Ljava.lang.String;`), though the descriptor the
                    // heap and the constant pool carry uses slashes.
                    "getName" => {
                        let binary = if name.starts_with('[') {
                            name.replace('/', ".")
                        } else {
                            name.clone()
                        };
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&binary)))))
                    }
                    "getSimpleName" => {
                        let simple = simple_class_name(&name);
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(simple)))))
                    }
                    // `Class.toString()` is "class <name>" (used when a Class is
                    // concatenated, e.g. a reflect type argument).
                    "toString" => {
                        let text = format!("class {}", simple_class_name(&name));
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&text)))))
                    }
                    // `this.isAssignableFrom(other)`: `other` is `this` or a
                    // subclass of `this` (walk `other`'s superclass chain).
                    "isAssignableFrom" => {
                        let other = match args.first() {
                            Some(JValue::Ref(Some(reference))) => match self.heap.get(*reference) {
                                Some(HeapObject::Class { name }) => Some(name.clone()),
                                _ => None,
                            },
                            _ => None,
                        };
                        let assignable =
                            other.is_some_and(|other| self.class_is_subtype(&other, &name));
                        Ok(Some(JValue::Int(i32::from(assignable))))
                    }
                    "getSuperclass" => {
                        let super_name = self.classes.get(&name).and_then(|cf| {
                            cf.constant_pool
                                .get_class_name(cf.super_class)
                                .map(str::to_owned)
                        });
                        match super_name {
                            Some(super_name) => {
                                let simple = simple_class_name(&super_name).to_owned();
                                let reference = self.heap.alloc(HeapObject::Class { name: simple });
                                Ok(Some(JValue::Ref(Some(reference))))
                            }
                            // `Object` (or an unloaded root) has no super.
                            None => Ok(Some(JValue::NULL)),
                        }
                    }
                    "getDeclaredFields" => {
                        let fields: Vec<(String, String, u16, Option<String>)> = self
                            .classes
                            .get(&name)
                            .map(|cf| {
                                cf.fields
                                    .iter()
                                    .map(|fi| {
                                        (
                                            cf.constant_pool
                                                .get_utf8(fi.name_index)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            cf.constant_pool
                                                .get_utf8(fi.descriptor_index)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            fi.access_flags.0,
                                            field_signature(cf, fi),
                                        )
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let simple = simple_class_name(&name).to_owned();
                        let refs: Vec<JValue> = fields
                            .into_iter()
                            .map(|(field_name, descriptor, access, signature)| {
                                JValue::Ref(Some(self.heap.alloc(HeapObject::Field {
                                    declaring: simple.clone(),
                                    name: field_name,
                                    descriptor,
                                    access,
                                    signature,
                                })))
                            })
                            .collect();
                        let array = self.heap.alloc(HeapObject::RefArray(
                            String::from("[Ljava/lang/reflect/Field;"),
                            refs,
                        ));
                        Ok(Some(JValue::Ref(Some(array))))
                    }
                    "getDeclaredMethods" | "getMethods" => {
                        let public_only = method == "getMethods";
                        let methods: Vec<(String, String, u16)> = self
                            .classes
                            .get(&name)
                            .map(|cf| {
                                cf.methods
                                    .iter()
                                    .filter(|m| {
                                        cf.constant_pool.get_utf8(m.name_index) != Some("<init>")
                                            && cf.constant_pool.get_utf8(m.name_index)
                                                != Some("<clinit>")
                                    })
                                    .filter(|m| {
                                        !public_only
                                            || m.access_flags.contains(
                                                caturra_classfile::MethodAccessFlags::PUBLIC,
                                            )
                                    })
                                    .map(|m| {
                                        (
                                            cf.constant_pool
                                                .get_utf8(m.name_index)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            cf.constant_pool
                                                .get_utf8(m.descriptor_index)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            m.access_flags.0,
                                        )
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let simple = simple_class_name(&name).to_owned();
                        let refs: Vec<JValue> = methods
                            .into_iter()
                            .map(|(mname, descriptor, access)| {
                                JValue::Ref(Some(self.heap.alloc(HeapObject::Method {
                                    declaring: simple.clone(),
                                    name: mname,
                                    descriptor,
                                    access,
                                })))
                            })
                            .collect();
                        let array = self.heap.alloc(HeapObject::RefArray(
                            String::from("[Ljava/lang/reflect/Method;"),
                            refs,
                        ));
                        Ok(Some(JValue::Ref(Some(array))))
                    }
                    "getDeclaredConstructors" | "getConstructors" => {
                        let public_only = method == "getConstructors";
                        let ctors: Vec<(String, u16)> = self
                            .classes
                            .get(&name)
                            .map(|cf| {
                                cf.methods
                                    .iter()
                                    .filter(|m| {
                                        cf.constant_pool.get_utf8(m.name_index) == Some("<init>")
                                    })
                                    .filter(|m| {
                                        !public_only
                                            || m.access_flags.contains(
                                                caturra_classfile::MethodAccessFlags::PUBLIC,
                                            )
                                    })
                                    .map(|m| {
                                        (
                                            cf.constant_pool
                                                .get_utf8(m.descriptor_index)
                                                .unwrap_or_default()
                                                .to_owned(),
                                            m.access_flags.0,
                                        )
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let simple = simple_class_name(&name).to_owned();
                        let refs: Vec<JValue> = ctors
                            .into_iter()
                            .map(|(descriptor, access)| {
                                JValue::Ref(Some(self.heap.alloc(HeapObject::Constructor {
                                    declaring: simple.clone(),
                                    descriptor,
                                    access,
                                })))
                            })
                            .collect();
                        let array = self.heap.alloc(HeapObject::RefArray(
                            String::from("[Ljava/lang/reflect/Constructor;"),
                            refs,
                        ));
                        Ok(Some(JValue::Ref(Some(array))))
                    }
                    "getConstructor" | "getDeclaredConstructor" => {
                        // args[0] is a Class[] of parameter types.
                        let param_class_names = self.class_array_names(args.first());
                        let found = self.classes.get(&name).and_then(|cf| {
                            cf.methods
                                .iter()
                                .filter(|m| {
                                    cf.constant_pool.get_utf8(m.name_index) == Some("<init>")
                                })
                                .find_map(|m| {
                                    let desc = cf
                                        .constant_pool
                                        .get_utf8(m.descriptor_index)
                                        .unwrap_or_default();
                                    constructor_params_match(desc, &param_class_names)
                                        .then(|| (desc.to_owned(), m.access_flags.0))
                                })
                        });
                        match found {
                            Some((descriptor, access)) => {
                                let declaring = simple_class_name(&name).to_owned();
                                Ok(Some(JValue::Ref(Some(self.heap.alloc(
                                    HeapObject::Constructor {
                                        declaring,
                                        descriptor,
                                        access,
                                    },
                                )))))
                            }
                            None => Err(VmError::UncaughtException(format!(
                                "java.lang.NoSuchMethodException: {}.<init>()",
                                simple_class_name(&name)
                            ))),
                        }
                    }
                    "getDeclaredField" | "getField" => {
                        let field_name = match args.first() {
                            Some(JValue::Ref(Some(r))) => {
                                self.heap.string_text(*r).unwrap_or_default()
                            }
                            _ => String::new(),
                        };
                        let found = self.classes.get(&name).and_then(|cf| {
                            cf.fields.iter().find_map(|fi| {
                                let fname =
                                    cf.constant_pool.get_utf8(fi.name_index).unwrap_or_default();
                                (fname == field_name).then(|| {
                                    (
                                        fname.to_owned(),
                                        cf.constant_pool
                                            .get_utf8(fi.descriptor_index)
                                            .unwrap_or_default()
                                            .to_owned(),
                                        fi.access_flags.0,
                                        field_signature(cf, fi),
                                    )
                                })
                            })
                        });
                        match found {
                            Some((fname, descriptor, access, signature)) => {
                                let declaring = simple_class_name(&name).to_owned();
                                Ok(Some(JValue::Ref(Some(self.heap.alloc(
                                    HeapObject::Field {
                                        declaring,
                                        name: fname,
                                        descriptor,
                                        access,
                                        signature,
                                    },
                                )))))
                            }
                            None => Err(VmError::UncaughtException(format!(
                                "java.lang.NoSuchFieldException: {field_name}"
                            ))),
                        }
                    }
                    "getMethod" | "getDeclaredMethod" => {
                        let method_name = match args.first() {
                            Some(JValue::Ref(Some(r))) => {
                                self.heap.string_text(*r).unwrap_or_default()
                            }
                            _ => String::new(),
                        };
                        // Optional Class[] of parameter types (getMethod is
                        // varargs; a bare `getMethod(name)` means no params).
                        let param_class_names = self.class_array_names(args.get(1));
                        let found = self.classes.get(&name).and_then(|cf| {
                            cf.methods.iter().find_map(|m| {
                                let mname =
                                    cf.constant_pool.get_utf8(m.name_index).unwrap_or_default();
                                if mname != method_name {
                                    return None;
                                }
                                let desc = cf
                                    .constant_pool
                                    .get_utf8(m.descriptor_index)
                                    .unwrap_or_default();
                                constructor_params_match(desc, &param_class_names)
                                    .then(|| (desc.to_owned(), m.access_flags.0))
                            })
                        });
                        match found {
                            Some((descriptor, access)) => {
                                let declaring = simple_class_name(&name).to_owned();
                                Ok(Some(JValue::Ref(Some(self.heap.alloc(
                                    HeapObject::Method {
                                        declaring,
                                        name: method_name,
                                        descriptor,
                                        access,
                                    },
                                )))))
                            }
                            None => Err(VmError::UncaughtException(format!(
                                "java.lang.NoSuchMethodException: {}.{method_name}",
                                simple_class_name(&name)
                            ))),
                        }
                    }
                    other => Err(VmError::UnknownIntrinsic(format!("Class.{other}"))),
                }
            }
            Some(HeapObject::Field {
                declaring,
                name,
                descriptor,
                access,
                signature,
            }) => {
                let (declaring, name, descriptor, access, signature) = (
                    declaring.clone(),
                    name.clone(),
                    descriptor.clone(),
                    *access,
                    signature.clone(),
                );
                match method {
                    "getName" => Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&name))))),
                    "getModifiers" => Ok(Some(JValue::Int(i32::from(access)))),
                    "getType" => {
                        let type_name = intrinsics::type_name_of_descriptor(&descriptor);
                        let reference = self.heap.alloc(HeapObject::Class { name: type_name });
                        Ok(Some(JValue::Ref(Some(reference))))
                    }
                    // `Field.getGenericType()`: a ParameterizedType when the
                    // field carries a Signature attribute, else the raw type.
                    "getGenericType" => {
                        let (raw, args) = signature
                            .as_deref()
                            .and_then(parse_parameterized)
                            .unwrap_or_else(|| {
                                (intrinsics::type_name_of_descriptor(&descriptor), Vec::new())
                            });
                        let reference = self.heap.alloc(HeapObject::ReflectType { raw, args });
                        Ok(Some(JValue::Ref(Some(reference))))
                    }
                    "toString" => {
                        let text =
                            intrinsics::field_to_string(&declaring, &name, &descriptor, access);
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&text)))))
                    }
                    // `setAccessible` is a no-op (caturra enforces no access control).
                    "setAccessible" => Ok(None),
                    "getInt" | "getLong" | "getDouble" | "getBoolean" => {
                        // Primitive accessors widen the field's value, or refuse.
                        let raw =
                            self.read_reflected_field(&declaring, &name, access, args.first())?;
                        Self::reflect_get_as(method, &descriptor, raw).map(Some)
                    }
                    "get" => {
                        // `Field.get` returns Object — a primitive field boxes
                        // (real Java returns an Integer/Double/… wrapper).
                        let raw =
                            self.read_reflected_field(&declaring, &name, access, args.first())?;
                        Ok(Some(self.box_if_primitive(raw, &descriptor)))
                    }
                    "set" | "setInt" | "setLong" | "setDouble" | "setBoolean" => {
                        // `set` stores whatever it is given; the typed setters
                        // widen the value into the field, or refuse.
                        let value = match args.get(1).copied() {
                            Some(value) if method != "set" => {
                                Some(Self::reflect_set_as(method, &descriptor, value)?)
                            }
                            other => other,
                        };
                        self.write_reflected_field(&declaring, &name, access, args.first(), value)?;
                        Ok(None)
                    }
                    other => Err(VmError::UnknownIntrinsic(format!("Field.{other}"))),
                }
            }
            Some(HeapObject::Constructor {
                declaring,
                descriptor,
                access,
            }) => {
                let (declaring, descriptor, access) =
                    (declaring.clone(), descriptor.clone(), *access);
                match method {
                    "getName" => Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&declaring))))),
                    "getModifiers" => Ok(Some(JValue::Int(i32::from(access)))),
                    "toString" => {
                        let text =
                            intrinsics::constructor_to_string(&declaring, &descriptor, access);
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&text)))))
                    }
                    other => Err(VmError::UnknownIntrinsic(format!("Constructor.{other}"))),
                }
            }
            Some(HeapObject::Method {
                declaring,
                name,
                descriptor,
                access,
            }) => {
                let (declaring, name, descriptor, access) =
                    (declaring.clone(), name.clone(), descriptor.clone(), *access);
                match method {
                    "getName" => Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&name))))),
                    "getModifiers" => Ok(Some(JValue::Int(i32::from(access)))),
                    "getReturnType" => {
                        let ret = descriptor.rsplit(')').next().unwrap_or("V");
                        let type_name = intrinsics::type_name_of_descriptor(ret);
                        let reference = self.heap.alloc(HeapObject::Class { name: type_name });
                        Ok(Some(JValue::Ref(Some(reference))))
                    }
                    "getParameterTypes" => {
                        let params = parse_descriptor_params(&descriptor);
                        let refs: Vec<JValue> = params
                            .iter()
                            .map(|p| {
                                let type_name = intrinsics::type_name_of_descriptor(p);
                                JValue::Ref(Some(
                                    self.heap.alloc(HeapObject::Class { name: type_name }),
                                ))
                            })
                            .collect();
                        let array = self.heap.alloc(HeapObject::RefArray(
                            String::from("[Ljava/lang/Class;"),
                            refs,
                        ));
                        Ok(Some(JValue::Ref(Some(array))))
                    }
                    "getParameterCount" => {
                        let count = parse_descriptor_params(&descriptor).len();
                        Ok(Some(JValue::Int(i32::try_from(count).unwrap_or(0))))
                    }
                    "toString" => {
                        let text = format!("{declaring}.{name}{descriptor}");
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&text)))))
                    }
                    "setAccessible" => Ok(None),
                    // `invoke` runs a frame and is handled before this value path.
                    other => Err(VmError::UnknownIntrinsic(format!("Method.{other}"))),
                }
            }
            Some(HeapObject::ReflectType { raw, args }) => {
                let (raw, args) = (raw.clone(), args.clone());
                match method {
                    // Each type argument becomes a `Class` (its `toString` is
                    // "class <name>", which the helpers concatenate).
                    "getActualTypeArguments" => {
                        let refs: Vec<JValue> = args
                            .iter()
                            .map(|name| {
                                JValue::Ref(Some(self.heap.alloc(HeapObject::Class {
                                    name: simple_class_name(name).to_owned(),
                                })))
                            })
                            .collect();
                        let array = self.heap.alloc(HeapObject::RefArray(
                            String::from("[Ljava/lang/reflect/Type;"),
                            refs,
                        ));
                        Ok(Some(JValue::Ref(Some(array))))
                    }
                    "getRawType" => {
                        let reference = self.heap.alloc(HeapObject::Class {
                            name: simple_class_name(&raw).to_owned(),
                        });
                        Ok(Some(JValue::Ref(Some(reference))))
                    }
                    "getTypeName" | "toString" => {
                        let dotted = raw.replace('/', ".");
                        let text = if args.is_empty() {
                            dotted
                        } else {
                            let inner: Vec<String> =
                                args.iter().map(|a| a.replace('/', ".")).collect();
                            format!("{dotted}<{}>", inner.join(", "))
                        };
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&text)))))
                    }
                    other => Err(VmError::UnknownIntrinsic(format!("Type.{other}"))),
                }
            }
            _ => Err(VmError::UnknownIntrinsic(format!("reflect.{method}"))),
        }
    }

    /// Convert a field's raw value for `Field.getInt/getLong/getDouble/
    /// getBoolean`. Java lets a typed accessor WIDEN the field's type but never
    /// narrow it — `getLong` on an `int` field is `7L`, while `getInt` on a
    /// `long` field is an `IllegalArgumentException`. Returning the raw value,
    /// as this used to, put a `Long` where the bytecode expected an `Int`.
    fn reflect_get_as(method: &str, descriptor: &str, raw: JValue) -> Result<JValue, VmError> {
        let bad = || {
            Err(VmError::UncaughtException(format!(
                "java.lang.IllegalArgumentException: cannot read a field of type \
                 '{descriptor}' with {method}"
            )))
        };
        // `byte`, `short` and `char` are already Ints at runtime.
        let integral = matches!(descriptor, "B" | "S" | "C" | "I");
        match method {
            "getBoolean" if descriptor == "Z" => Ok(raw),
            "getInt" if integral => Ok(raw),
            "getLong" => match (descriptor, raw) {
                (_, JValue::Int(v)) if integral => Ok(JValue::Long(i64::from(v))),
                ("J", value @ JValue::Long(_)) => Ok(value),
                _ => bad(),
            },
            "getDouble" => match (descriptor, raw) {
                (_, JValue::Int(v)) if integral => Ok(JValue::Double(f64::from(v))),
                #[allow(clippy::cast_precision_loss)] // Java widens long->double the same way
                ("J", JValue::Long(v)) => Ok(JValue::Double(v as f64)),
                ("F", JValue::Float(v)) => Ok(JValue::Double(f64::from(v))),
                ("D", value @ JValue::Double(_)) => Ok(value),
                _ => bad(),
            },
            _ => bad(),
        }
    }

    /// The mirror image for `Field.setInt/setLong/setDouble/setBoolean`: the
    /// value may widen into the field, never narrow. `setInt` into a `long`
    /// field stores `5L`; `setLong` into an `int` field is an error.
    fn reflect_set_as(method: &str, descriptor: &str, value: JValue) -> Result<JValue, VmError> {
        let bad = || {
            Err(VmError::UncaughtException(format!(
                "java.lang.IllegalArgumentException: cannot write a field of type \
                 '{descriptor}' with {method}"
            )))
        };
        #[allow(clippy::cast_precision_loss)] // Java widens long->double/float likewise
        match (method, descriptor, value) {
            // Exact match: store as given.
            ("setBoolean", "Z", v)
            | ("setInt", "I", v)
            | ("setLong", "J", v)
            | ("setDouble", "D", v) => Ok(v),
            // Widening conversions.
            ("setInt", "J", JValue::Int(v)) => Ok(JValue::Long(i64::from(v))),
            ("setInt", "F", JValue::Int(v)) => Ok(JValue::Float(v as f32)),
            ("setInt", "D", JValue::Int(v)) => Ok(JValue::Double(f64::from(v))),
            ("setLong", "F", JValue::Long(v)) => Ok(JValue::Float(v as f32)),
            ("setLong", "D", JValue::Long(v)) => Ok(JValue::Double(v as f64)),
            _ => bad(),
        }
    }

    /// Read a field reflectively (`Field.get`): from the class statics for a
    /// static field, otherwise from the instance passed as the argument.
    fn read_reflected_field(
        &mut self,
        declaring: &str,
        name: &str,
        access: u16,
        obj: Option<&JValue>,
    ) -> Result<JValue, VmError> {
        if access & 0x0008 != 0 {
            return self
                .statics
                .get(declaring)
                .and_then(|fields| fields.get(name))
                .copied()
                .ok_or_else(|| {
                    VmError::UncaughtException(format!("java.lang.NoSuchFieldException: {name}"))
                });
        }
        let Some(JValue::Ref(Some(reference))) = obj else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let key = instance_field_key(declaring, name);
        match self.heap.get(*reference) {
            Some(crate::value::HeapObject::Instance { fields, .. }) => {
                fields.get(key.as_str()).copied().ok_or_else(|| {
                    VmError::UncaughtException(format!("java.lang.NoSuchFieldException: {name}"))
                })
            }
            _ => Err(VmError::UncaughtException(format!(
                "cannot read field {name}"
            ))),
        }
    }

    /// Box a primitive value into its wrapper (for `Field.get`, which is
    /// typed `Object`); references pass through unchanged.
    fn box_if_primitive(&mut self, value: JValue, descriptor: &str) -> JValue {
        let wrapper = match descriptor {
            "I" => "java/lang/Integer",
            "J" => "java/lang/Long",
            "D" => "java/lang/Double",
            "F" => "java/lang/Float",
            "Z" => "java/lang/Boolean",
            "S" => "java/lang/Short",
            "B" => "java/lang/Byte",
            "C" => "java/lang/Character",
            _ => return value,
        };
        let reference = self.heap.alloc(crate::value::HeapObject::Boxed {
            class_name: wrapper.to_owned(),
            value,
        });
        JValue::Ref(Some(reference))
    }

    /// Write a field reflectively (`Field.set`).
    fn write_reflected_field(
        &mut self,
        declaring: &str,
        name: &str,
        access: u16,
        obj: Option<&JValue>,
        value: Option<JValue>,
    ) -> Result<(), VmError> {
        let value = value.unwrap_or(JValue::NULL);
        if access & 0x0008 != 0 {
            if let Some(fields) = self.statics.get_mut(declaring) {
                fields.insert(name.to_owned(), value);
            }
            return Ok(());
        }
        let Some(JValue::Ref(Some(reference))) = obj else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let key = instance_field_key(declaring, name);
        match self.heap.get_mut(*reference) {
            Some(crate::value::HeapObject::Instance { fields, .. }) => {
                fields.insert(Rc::from(key.as_str()), value);
                Ok(())
            }
            _ => Err(VmError::UncaughtException(format!(
                "cannot set field {name}"
            ))),
        }
    }

    /// Materialize a loadable constant-pool entry as a value
    /// (JVMS §5.1: `ldc` family).
    fn load_constant(
        &mut self,
        class: &ClassFile,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<JValue, VmError> {
        let constant = class
            .constant_pool
            .get(index)
            .ok_or_else(|| malformed(format!("bad constant pool index {index}")))?;
        match constant {
            Constant::Integer(v) => Ok(JValue::Int(*v)),
            Constant::Float(v) => Ok(JValue::Float(*v)),
            Constant::Long(v) => Ok(JValue::Long(*v)),
            Constant::Double(v) => Ok(JValue::Double(*v)),
            Constant::String { string_index } => {
                let text = class
                    .constant_pool
                    .get_utf8(*string_index)
                    .ok_or_else(|| malformed(format!("bad string constant at {index}")))?
                    .to_owned();
                Ok(JValue::Ref(Some(self.intern_string(&text))))
            }
            other => Err(malformed(format!(
                "cannot load constant {other:?} with ldc"
            ))),
        }
    }
}

/// The VM side of watch evaluation: invokes a compiled watch method
/// against the paused frame's locals, sharing the live heap/statics.
struct FrameWatchEvaluator<'a, 'run> {
    interp: &'a mut Interpreter<'run>,
    active: &'a Frame<'run>,
}

/// Instruction allowance per watch evaluation — generous for
/// expressions, tight enough that an accidental `while(true)` in a
/// watch reports quickly instead of hanging the pause.
const WATCH_INSTRUCTION_BUDGET: u64 = 10_000_000;

impl WatchEvaluator for FrameWatchEvaluator<'_, '_> {
    fn evaluate(
        &mut self,
        class: &ClassFile,
        method_name: &str,
        param_names: &[String],
    ) -> Result<String, String> {
        let interp = &mut *self.interp;
        let arena = interp
            .watch_arena
            .ok_or_else(|| String::from("watch evaluation is unavailable"))?;
        // The frame will borrow the class at 'run; park it in the
        // run-scoped arena to satisfy that.
        let class: &ClassFile = arena.alloc(class.clone());

        let method = class
            .methods
            .iter()
            .find(|m| class.constant_pool.get_utf8(m.name_index) == Some(method_name))
            .ok_or_else(|| format!("watch method {method_name} missing"))?;
        let descriptor = class
            .constant_pool
            .get_utf8(method.descriptor_index)
            .unwrap_or("()Ljava/lang/String;");
        let widths = descriptor_arg_widths(descriptor)
            .ok_or_else(|| String::from("bad watch descriptor"))?;

        // Arguments: the paused frame's current values, by name.
        let mut locals = Vec::with_capacity(param_names.len());
        for (name, width) in param_names.iter().zip(&widths) {
            let slot = self
                .active
                .code
                .local_vars
                .iter()
                .find(|(n, _, _, _)| n == name)
                .map(|(_, _, slot, _)| *slot)
                .ok_or_else(|| format!("no local named {name}"))?;
            let value = self
                .active
                .locals
                .get(usize::from(slot))
                .copied()
                .unwrap_or(JValue::NULL);
            locals.push(value);
            if *width == 2 {
                locals.push(JValue::Int(0));
            }
        }

        // Run isolated from the paused program's control state: fresh
        // frame stack, capped budget; heap/statics/console are shared
        // (that is the point — watches see live state).
        let saved_frames = std::mem::take(&mut interp.frames);
        let saved_location = interp.current_location.take();
        // A watch is its own stack, even when the pause happened inside a
        // nested run (a breakpoint in a `toString()` reached from a
        // container's).
        let saved_runs = std::mem::take(&mut interp.suspended_runs);
        let saved_base = std::mem::replace(&mut interp.nested_frame_base, 0);
        let saved_budget = interp.remaining_instructions;
        interp.remaining_instructions = saved_budget.min(WATCH_INSTRUCTION_BUDGET);

        let result = interp.execute(class, method, locals);

        let spent = saved_budget.min(WATCH_INSTRUCTION_BUDGET) - interp.remaining_instructions;
        interp.remaining_instructions = saved_budget.saturating_sub(spent);
        interp.nested_frame_base = saved_base;
        interp.suspended_runs = saved_runs;
        interp.frames = saved_frames;
        interp.current_location = saved_location;

        match result {
            Ok(Some(JValue::Ref(Some(reference)))) => interp
                .heap
                .string_text(reference)
                .ok_or_else(|| String::from("watch produced a non-string result")),
            Ok(_) => Err(String::from("watch produced no value")),
            Err(VmError::UncaughtException(message)) => {
                // First line only: the trace points into synthesized
                // code, which would confuse more than help.
                Err(message.lines().next().unwrap_or("exception").to_owned())
            }
            Err(VmError::InstructionBudgetExceeded) => {
                Err(String::from("the watch expression ran too long"))
            }
            Err(other) => Err(other.to_string()),
        }
    }
}

/// What one dispatched instruction asks the frame loop to do next.
enum Flow<'run> {
    /// Fall through to the next instruction.
    Next,
    /// Push this callee frame and continue inside it.
    Call(Frame<'run>),
    /// Pop the current frame, delivering a value to the caller (or out
    /// of `execute` for the last frame).
    Return(Option<JValue>),
    /// Static initializers must run first (execution order), then the
    /// current instruction re-executes (`frame.pc` already rewound).
    InitChain(Vec<Frame<'run>>),
}

/// The outcome of dispatching a virtual call on a user object.
/// One element of an `int[]`, `boolean[]` or `char[]`, printed as Java prints
/// it. The three share a representation, so only the kind tells them apart.
fn int_element_text(kind: crate::value::IntKind, value: i32) -> String {
    match kind {
        crate::value::IntKind::Int => value.to_string(),
        crate::value::IntKind::Boolean => (value != 0).to_string(),
        crate::value::IntKind::Char => char::from_u32(value.cast_unsigned())
            .unwrap_or('\u{FFFD}')
            .to_string(),
    }
}

/// An `ArrayIndexOutOfBoundsException` naming the offending index.
/// Whether a `Set` method mutates it — so an unmodifiable set view must throw
/// `UnsupportedOperationException` rather than delegate.
fn is_set_mutator(method: &str) -> bool {
    matches!(
        method,
        "add"
            | "remove"
            | "clear"
            | "addAll"
            | "removeAll"
            | "retainAll"
            | "removeIf"
            | "pollFirst"
            | "pollLast"
    )
}

/// Whether a `Map` method mutates it (an unmodifiable map view throws on these).
fn is_map_mutator(method: &str) -> bool {
    matches!(
        method,
        "put"
            | "remove"
            | "clear"
            | "putAll"
            | "putIfAbsent"
            | "replace"
            | "replaceAll"
            | "merge"
            | "compute"
            | "computeIfAbsent"
            | "computeIfPresent"
    )
}

fn array_index_error(index: i32) -> VmError {
    VmError::UncaughtException(format!("java.lang.ArrayIndexOutOfBoundsException: {index}"))
}

/// `Double.compare`: a total order where -0.0 sits below 0.0 and NaN above
/// everything. `<` and `>` agree with neither.
fn java_double_compare(a: f64, b: f64) -> i32 {
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    let bits = |value: f64| {
        if value.is_nan() { f64::NAN } else { value }
            .to_bits()
            .cast_signed()
    };
    bits(a).cmp(&bits(b)) as i32
}

/// `Float.compare`, likewise.
fn java_float_compare(a: f32, b: f32) -> i32 {
    if a < b {
        return -1;
    }
    if a > b {
        return 1;
    }
    let bits = |value: f32| {
        if value.is_nan() { f32::NAN } else { value }
            .to_bits()
            .cast_signed()
    };
    bits(a).cmp(&bits(b)) as i32
}

/// `Float.hashCode`, which collapses every NaN payload to one.
fn float_hash(value: f32) -> i32 {
    if value.is_nan() {
        f32::NAN.to_bits().cast_signed()
    } else {
        value.to_bits().cast_signed()
    }
}

/// One element of an `int[]`, `boolean[]` or `char[]`, hashed as Java hashes
/// it: `Boolean.hashCode` is 1231/1237, not the value.
fn int_element_hash(kind: crate::value::IntKind, value: i32) -> i32 {
    match kind {
        crate::value::IntKind::Int | crate::value::IntKind::Char => value,
        crate::value::IntKind::Boolean => {
            if value != 0 {
                1231
            } else {
                1237
            }
        }
    }
}

/// The `List` methods an unmodifiable view refuses. Java's throws
/// `UnsupportedOperationException` from each of them.
const LIST_MUTATORS: &[&str] = &[
    "add",
    "remove",
    "set",
    "clear",
    "addAll",
    "removeAll",
    "retainAll",
];

/// Java's marker for a container that holds itself, instead of recursing.
const THIS_COLLECTION: &str = "(this Collection)";
const THIS_MAP: &str = "(this Map)";

/// How deeply [`Interpreter::string_value_of`] nests containers before it calls
/// the shape a cycle. A real JVM throws `StackOverflowError`; so do we,
/// rather than exhausting the host stack.
const MAX_RENDER_DEPTH: u32 = 64;

/// What [`Interpreter::string_value_of`] found on the heap, lifted out of it so
/// that rendering can call back into Java.
enum Renderable {
    /// An instance of a loaded class: ask its `toString()`.
    Instance(String),
    List(Vec<JValue>),
    Map(Vec<(JValue, JValue)>),
    View(Vec<(JValue, JValue)>, MapViewKind),
    /// A `Map.Entry`: its map and its key, so the value is read live.
    Entry(HeapRef, JValue),
    /// An `Optional`/`OptionalInt`/`OptionalDouble`: its value (if present) and
    /// its `toString` prefix.
    Optional(Option<JValue>, crate::value::OptionalKind),
    /// Anything the intrinsic layer can render without calling Java.
    Opaque,
}

/// Whether one of the interpreter-side collection methods answered a call,
/// and with what. These live outside the intrinsic layer because comparing
/// elements may run a user `equals`, `hashCode` or `toString`.
enum Answered {
    /// Not one of ours; the intrinsic layer should try.
    No,
    /// Answered, with no value (a `void` method).
    Void,
    Value(JValue),
}

enum UserDispatch<'run> {
    /// Push this frame and continue executing inside it.
    Call(Frame<'run>),
    /// The call was satisfied inline (default `toString`).
    Value(Option<JValue>),
}

/// One Java method activation, as plain data: everything a debugger
/// would need to show a paused call (owner class, method name, program
/// counter, locals, operand stack).
struct Frame<'run> {
    class: &'run ClassFile,
    /// Borrowed from the class's constant pool: allocating this per call showed
    /// up as pure overhead in call-heavy programs (an image filter makes millions).
    method_name: &'run str,
    code: Rc<MethodCode>,
    pc: usize,
    /// The last source line crossed (1-based), from `LineNumberTable`.
    current_line: Option<u16>,
    locals: Vec<JValue>,
    stack: Vec<JValue>,
    /// When set, this frame's return value is boxed into the named wrapper
    /// before delivery (a reflective `Method.invoke` of a primitive-returning
    /// method must hand back an `Integer`/`Double`/… like real Java).
    box_return_as: Option<String>,
}

/// A parsed `Code` attribute plus its debug tables, cached per method.
struct MethodCode {
    attr: CodeAttribute,
    /// `boundary_lines[pc]` is the source line starting at `pc`, or 0 —
    /// O(1) per-instruction lookup in the dispatch loop.
    boundary_lines: Vec<u16>,
    /// Named locals from `LocalVariableTable` (+`...TypeTable`):
    /// `(name, java source type, slot, live-from pc)`. The source type
    /// is fully qualified outside `java.lang` so synthesized watch
    /// code compiles without imports; empty when unrepresentable.
    local_vars: Vec<(String, String, u16, u16)>,
    /// The class's `SourceFile` (falls back to `ClassName.java`).
    source_file: String,
}

/// A JVM descriptor or generic signature rendered as Java source, fully
/// qualified outside `java.lang` (so synthesized code needs no
/// imports). Empty string when the type has no source form here.
fn source_type_of(descriptor: &str) -> String {
    match descriptor {
        "I" => return String::from("int"),
        "J" => return String::from("long"),
        "F" => return String::from("float"),
        "S" => return String::from("short"),
        "B" => return String::from("byte"),
        "D" => return String::from("double"),
        "Z" => return String::from("boolean"),
        "C" => return String::from("char"),
        "Ljava/lang/String;" => return String::from("String"),
        "Ljava/util/Scanner;" => return String::from("java.util.Scanner"),
        "Ljava/io/File;" => return String::from("java.io.File"),
        "Ljava/io/PrintWriter;" => return String::from("java.io.PrintWriter"),
        _ => {}
    }
    if let Some(rest) = descriptor.strip_prefix('[') {
        let elem = source_type_of(rest);
        if elem.is_empty() {
            return String::new();
        }
        return format!("{elem}[]");
    }
    if let Some(elem) = descriptor
        .strip_prefix("Ljava/util/ArrayList<")
        .and_then(|s| s.strip_suffix(">;"))
    {
        let elem = match elem {
            "I" => "Integer",
            "D" => "Double",
            "Z" => "Boolean",
            "C" => "Character",
            _ => {
                return match source_type_of(elem).as_str() {
                    "" => String::new(),
                    other => format!("java.util.ArrayList<{other}>"),
                };
            }
        };
        return format!("java.util.ArrayList<{elem}>");
    }
    // A user class: `LFoo;` → `Foo` (single shared namespace, no
    // package prefix). Library classes were matched above.
    if let Some(name) = descriptor
        .strip_prefix('L')
        .and_then(|s| s.strip_suffix(';'))
        && !name.contains('/')
    {
        return name.to_owned();
    }
    String::new()
}

/// Parse a method's `Code` attribute and debug tables.
fn parse_method_code(class: &ClassFile, method: &MethodInfo) -> Result<MethodCode, VmError> {
    let class_name = class.class_name().unwrap_or("<unknown>").to_owned();
    let malformed = |reason: String| VmError::MalformedClass {
        name: class_name.clone(),
        reason,
    };
    let code_info = method
        .attributes
        .iter()
        .find(|a| class.constant_pool.get_utf8(a.name_index) == Some(CODE_ATTRIBUTE))
        .ok_or_else(|| malformed(String::from("method has no Code attribute")))?;
    let attr = read_code_attribute(&code_info.info)
        .map_err(|e| malformed(format!("bad Code attribute: {e}")))?;

    let mut boundary_lines = vec![0u16; attr.code.len()];
    let mut local_vars: Vec<(String, String, u16, u16)> = Vec::new();
    let mut signatures: Vec<(String, String)> = Vec::new();
    for nested in &attr.attributes {
        match class.constant_pool.get_utf8(nested.name_index) {
            Some(caturra_classfile::debug::LINE_NUMBER_TABLE_ATTRIBUTE) => {
                for (start_pc, line) in
                    caturra_classfile::debug::decode_line_number_table(&nested.info)
                        .unwrap_or_default()
                {
                    if let Some(slot) = boundary_lines.get_mut(usize::from(start_pc)) {
                        *slot = line;
                    }
                }
            }
            Some(caturra_classfile::debug::LOCAL_VARIABLE_TABLE_ATTRIBUTE) => {
                for entry in caturra_classfile::debug::decode_local_variable_table(&nested.info)
                    .unwrap_or_default()
                {
                    let name = class
                        .constant_pool
                        .get_utf8(entry.name_index)
                        .unwrap_or("?")
                        .to_owned();
                    let descriptor = class
                        .constant_pool
                        .get_utf8(entry.descriptor_index)
                        .unwrap_or("")
                        .to_owned();
                    local_vars.push((name, descriptor, entry.index, entry.start_pc));
                }
            }
            Some(caturra_classfile::debug::LOCAL_VARIABLE_TYPE_TABLE_ATTRIBUTE) => {
                for entry in caturra_classfile::debug::decode_local_variable_table(&nested.info)
                    .unwrap_or_default()
                {
                    let name = class
                        .constant_pool
                        .get_utf8(entry.name_index)
                        .unwrap_or("?")
                        .to_owned();
                    let signature = class
                        .constant_pool
                        .get_utf8(entry.descriptor_index)
                        .unwrap_or("")
                        .to_owned();
                    signatures.push((name, signature));
                }
            }
            _ => {}
        }
    }
    // Second pass: descriptors (or generic signatures) → Java source
    // types, so debugger hosts can synthesize code referencing them.
    for (name, descriptor, _, _) in &mut local_vars {
        let signature = signatures
            .iter()
            .find(|(n, _)| n == name)
            .map_or(descriptor.as_str(), |(_, s)| s.as_str());
        *descriptor = source_type_of(signature);
    }

    let source_file = class
        .attributes
        .iter()
        .find(|a| {
            class.constant_pool.get_utf8(a.name_index)
                == Some(caturra_classfile::debug::SOURCE_FILE_ATTRIBUTE)
        })
        .and_then(|a| caturra_classfile::debug::decode_source_file(&a.info))
        .and_then(|index| class.constant_pool.get_utf8(index))
        .map_or_else(|| format!("{class_name}.java"), str::to_owned);

    Ok(MethodCode {
        attr,
        boundary_lines,
        local_vars,
        source_file,
    })
}

impl Frame<'_> {
    fn pop(&mut self) -> Result<JValue, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn pop_int(&mut self) -> Result<i32, VmError> {
        match self.pop()? {
            JValue::Int(value) => Ok(value),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected an int on the stack, found {other:?}"
            ))),
        }
    }

    fn pop_float(&mut self) -> Result<f32, VmError> {
        match self.pop()? {
            JValue::Float(value) => Ok(value),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected a float on the stack, found {other:?}"
            ))),
        }
    }

    fn float_binop(&mut self, apply: impl Fn(f32, f32) -> f32) -> Result<(), VmError> {
        let b = self.pop_float()?;
        let a = self.pop_float()?;
        self.stack.push(JValue::Float(apply(a, b)));
        Ok(())
    }

    fn pop_long(&mut self) -> Result<i64, VmError> {
        match self.pop()? {
            JValue::Long(value) => Ok(value),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected a long on the stack, found {other:?}"
            ))),
        }
    }

    fn long_binop(&mut self, apply: impl Fn(i64, i64) -> i64) -> Result<(), VmError> {
        let b = self.pop_long()?;
        let a = self.pop_long()?;
        self.stack.push(JValue::Long(apply(a, b)));
        Ok(())
    }

    fn long_division(&mut self, apply: impl Fn(i64, i64) -> i64) -> Result<(), VmError> {
        let b = self.pop_long()?;
        let a = self.pop_long()?;
        if b == 0 {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.ArithmeticException: / by zero",
            )));
        }
        self.stack.push(JValue::Long(apply(a, b)));
        Ok(())
    }

    fn pop_double(&mut self) -> Result<f64, VmError> {
        match self.pop()? {
            JValue::Double(value) => Ok(value),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected a double on the stack, found {other:?}"
            ))),
        }
    }

    fn pop_ref(&mut self) -> Result<Option<HeapRef>, VmError> {
        match self.pop()? {
            JValue::Ref(reference) => Ok(reference),
            other => Err(VmError::UncaughtException(format!(
                "java.lang.VerifyError: expected a reference on the stack, found {other:?}"
            ))),
        }
    }

    /// Pop `index` then `arrayref` for an array access, raising NPE for
    /// a null array.
    fn pop_array_access(&mut self) -> Result<(HeapRef, i32), VmError> {
        let index = self.pop_int()?;
        let reference = self.pop_ref()?.ok_or_else(null_array)?;
        Ok((reference, index))
    }

    fn load(&mut self, slot: usize, malformed: &impl Fn(String) -> VmError) -> Result<(), VmError> {
        let value = *self
            .locals
            .get(slot)
            .ok_or_else(|| malformed(format!("load from out-of-range local slot {slot}")))?;
        self.stack.push(value);
        Ok(())
    }

    fn store(
        &mut self,
        slot: usize,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let value = self.pop()?;
        let target = self
            .locals
            .get_mut(slot)
            .ok_or_else(|| malformed(format!("store to out-of-range local slot {slot}")))?;
        *target = value;
        Ok(())
    }

    fn int_binop(&mut self, apply: impl Fn(i32, i32) -> i32) -> Result<(), VmError> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        self.stack.push(JValue::Int(apply(a, b)));
        Ok(())
    }

    /// `idiv`/`irem`: like [`Self::int_binop`] plus Java's division-by-zero
    /// exception.
    fn int_division(&mut self, apply: impl Fn(i32, i32) -> i32) -> Result<(), VmError> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        if b == 0 {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.ArithmeticException: / by zero",
            )));
        }
        self.stack.push(JValue::Int(apply(a, b)));
        Ok(())
    }

    fn double_binop(&mut self, apply: impl Fn(f64, f64) -> f64) -> Result<(), VmError> {
        let b = self.pop_double()?;
        let a = self.pop_double()?;
        self.stack.push(JValue::Double(apply(a, b)));
        Ok(())
    }
}

/// `[a, b, c]` capped at 20 elements for the locals view.
/// The simple name of a (possibly `/`- or `.`-qualified) class name.
/// caturra is a flat namespace, so this is usually a no-op.
fn simple_class_name(name: &str) -> &str {
    name.rsplit(['/', '.']).next().unwrap_or(name)
}

/// Read a field's generic `Signature` attribute (e.g.
/// `Ljava/util/ArrayList<LFriend;>;`), if present.
fn field_signature(cf: &ClassFile, fi: &caturra_classfile::FieldInfo) -> Option<String> {
    fi.attributes.iter().find_map(|a| {
        if cf.constant_pool.get_utf8(a.name_index) == Some("Signature") && a.info.len() == 2 {
            let index = u16::from_be_bytes([a.info[0], a.info[1]]);
            cf.constant_pool.get_utf8(index).map(str::to_owned)
        } else {
            None
        }
    })
}

/// Parse a parameterized generic signature (`Ljava/util/ArrayList<LFriend;>;`)
/// into `(raw class, type-argument class names)`. `None` when not
/// parameterized.
fn parse_parameterized(signature: &str) -> Option<(String, Vec<String>)> {
    let open = signature.find('<')?;
    let close = signature.rfind('>')?;
    let raw = signature.get(1..open)?.to_owned(); // strip leading 'L'
    let mut args = Vec::new();
    for part in signature.get(open + 1..close)?.split(';') {
        let part = part.trim();
        if let Some(name) = part.strip_prefix('L') {
            args.push(name.to_owned());
        }
    }
    Some((raw, args))
}

/// An instance field's key on the heap: its DECLARING class and its name.
///
/// A subclass may hide a superclass field of the same name (JLS 8.3). The two
/// are distinct slots, and which one an access means is fixed by the static
/// type at the access site — which the bytecode records as the field ref's
/// owner. Keying by name alone silently merged them.
fn instance_field_key(owner: &str, name: &str) -> String {
    format!("{owner}.{name}")
}

/// The declaring class and simple name of a heap field key.
fn split_field_key(key: &str) -> (&str, &str) {
    key.rsplit_once('.').unwrap_or(("", key))
}

/// A class file's own name.
fn class_name_of(class: &ClassFile) -> &str {
    class
        .constant_pool
        .get_class_name(class.this_class)
        .unwrap_or_default()
}

/// Whether a value whose array class is `actual` casts to the array target
/// `target` (JVMS checkcast on arrays). Exact match always; and any reference
/// array — a reference-element `[L…;` or a nested `[[…` — is-a `Object[]`, but
/// a primitive-element array (`[I`) is not.
fn array_cast_ok(actual: &str, target: &str) -> bool {
    if actual == target {
        return true;
    }
    target == "[Ljava/lang/Object;" && (actual.starts_with("[L") || actual.starts_with("[["))
}

/// The JVM class descriptor of an array whose ELEMENT class is `element`,
/// as `anewarray`'s constant pool spells it: a plain class name becomes
/// `[Lname;`, and an already-array element (`[I`, a row of `int[][]`)
/// just gains a dimension.
fn array_class_of(element: &str) -> String {
    if element.starts_with('[') {
        format!("[{element}")
    } else {
        format!("[L{element};")
    }
}

/// Whether a heap object is an array (all kinds), for `Object`-method dispatch.
fn is_array_object(object: Option<&crate::value::HeapObject>) -> bool {
    use crate::value::HeapObject;
    matches!(
        object,
        Some(
            HeapObject::RefArray(_, _)
                | HeapObject::IntArray(..)
                | HeapObject::DoubleArray(_)
                | HeapObject::LongArray(_)
                | HeapObject::FloatArray(_)
                | HeapObject::ShortArray(_)
                | HeapObject::ByteArray(_)
        )
    )
}

/// Parse the parameter descriptors from a method descriptor, e.g.
/// `(Ljava/lang/String;I)V` -> `["Ljava/lang/String;", "I"]`.
fn parse_descriptor_params(descriptor: &str) -> Vec<String> {
    let mut params = Vec::new();
    let inner = descriptor
        .strip_prefix('(')
        .and_then(|s| s.split(')').next())
        .unwrap_or("");
    let mut chars = inner.chars().peekable();
    while let Some(&c) = chars.peek() {
        let mut param = String::new();
        while chars.peek() == Some(&'[') {
            param.push(chars.next().unwrap());
        }
        match chars.next() {
            Some('L') => {
                param.push('L');
                for ch in chars.by_ref() {
                    param.push(ch);
                    if ch == ';' {
                        break;
                    }
                }
            }
            Some(prim) => param.push(prim),
            None => break,
        }
        let _ = c;
        params.push(param);
    }
    params
}

/// Whether a `<init>` descriptor's parameters match the given `Class[]` names
/// (with wrapper/primitive boxing equivalence, as `getConstructor` allows).
fn constructor_params_match(descriptor: &str, class_names: &[String]) -> bool {
    let params = parse_descriptor_params(descriptor);
    params.len() == class_names.len()
        && params
            .iter()
            .zip(class_names)
            .all(|(desc, class_name)| class_matches_descriptor(class_name, desc))
}

/// Whether a `Class` (by name) matches a parameter descriptor. Strict, like
/// real reflection: a primitive `Class` matches only its primitive descriptor
/// (no wrapper boxing) — `int.class` matches `(int)`, not `(Integer)`.
fn class_matches_descriptor(class_name: &str, desc: &str) -> bool {
    match class_name {
        "int" => desc == "I",
        "long" => desc == "J",
        "double" => desc == "D",
        "float" => desc == "F",
        "boolean" => desc == "Z",
        "char" => desc == "C",
        "short" => desc == "S",
        "byte" => desc == "B",
        // Array class literals carry the JVM descriptor as their name (`[I`).
        name if name.starts_with('[') => desc == name,
        _ => desc == format!("L{class_name};"),
    }
}

fn render_array(items: impl Iterator<Item = String>) -> String {
    let mut parts: Vec<String> = items.take(21).collect();
    if parts.len() > 20 {
        parts.truncate(20);
        parts.push(String::from("…"));
    }
    format!("[{}]", parts.join(", "))
}

/// The default value for a field of the given descriptor (JLS §4.12.5).
fn default_for_descriptor(descriptor: &str) -> JValue {
    match descriptor.chars().next() {
        Some('D') => JValue::Double(0.0),
        Some('I' | 'Z' | 'C' | 'B' | 'S') => JValue::Int(0),
        Some('J') => JValue::Long(0),
        Some('F') => JValue::Float(0.0),
        _ => JValue::NULL,
    }
}

fn null_array() -> VmError {
    VmError::UncaughtException(String::from(
        "java.lang.NullPointerException: the array is null",
    ))
}

/// Validate an array creation size (JVMS: negative →
/// `NegativeArraySizeException`).
fn check_array_size(size: i32) -> Result<usize, VmError> {
    usize::try_from(size).map_err(|_| {
        VmError::UncaughtException(format!("java.lang.NegativeArraySizeException: {size}"))
    })
}

/// Bounds-checked array element access with Java 11's exception
/// message format.
/// The interfaces implemented by a class and its superclasses, as
/// loaded `ClassFile`s (direct interfaces only; super-interfaces are
/// enqueued during the default-method search).
fn collect_interfaces<'run>(
    classes: &'run HashMap<String, ClassFile>,
    instance_class: &str,
) -> Vec<&'run ClassFile> {
    let mut result = Vec::new();
    let mut current = classes.get(instance_class);
    let mut steps = 0usize;
    while let Some(class) = current {
        steps += 1;
        if steps > classes.len() + 1 {
            break;
        }
        for index in &class.interfaces {
            if let Some(name) = class.constant_pool.get_class_name(*index)
                && let Some(iface) = classes.get(name)
            {
                result.push(iface);
            }
        }
        current = class
            .constant_pool
            .get_class_name(class.super_class)
            .and_then(|super_name| classes.get(super_name));
    }
    result
}

fn array_get<T>(values: &[T], index: i32) -> Result<&T, VmError> {
    usize::try_from(index)
        .ok()
        .and_then(|i| values.get(i))
        .ok_or_else(|| out_of_bounds(index, values.len()))
}

fn array_get_mut<T>(values: &mut [T], index: i32) -> Result<&mut T, VmError> {
    let length = values.len();
    usize::try_from(index)
        .ok()
        .and_then(|i| values.get_mut(i))
        .ok_or_else(|| out_of_bounds(index, length))
}

fn out_of_bounds(index: i32, length: usize) -> VmError {
    VmError::UncaughtException(format!(
        "java.lang.ArrayIndexOutOfBoundsException: Index {index} out of bounds for length {length}"
    ))
}

/// Resolve a branch offset relative to the branch opcode's address.
fn branch_target(
    addr: usize,
    offset: i16,
    code_len: usize,
    malformed: &impl Fn(String) -> VmError,
) -> Result<usize, VmError> {
    let target = i64::try_from(addr).expect("code fits i64") + i64::from(offset);
    usize::try_from(target)
        .ok()
        .filter(|t| *t < code_len)
        .ok_or_else(|| malformed(format!("branch at pc {addr} to out-of-range {target}")))
}

fn read_u8(
    bytes: &[u8],
    pc: &mut usize,
    malformed: &impl Fn(String) -> VmError,
) -> Result<u8, VmError> {
    let byte = *bytes
        .get(*pc)
        .ok_or_else(|| malformed(format!("truncated instruction at pc {pc}")))?;
    *pc += 1;
    Ok(byte)
}

fn read_u16(
    bytes: &[u8],
    pc: &mut usize,
    malformed: &impl Fn(String) -> VmError,
) -> Result<u16, VmError> {
    let high = read_u8(bytes, pc, malformed)?;
    let low = read_u8(bytes, pc, malformed)?;
    Ok(u16::from_be_bytes([high, low]))
}

/// Find a class's static method by name and exact descriptor.
/// JVMS 5.4.3.3 method resolution for `invokestatic`: the named class, then
/// each superclass. Returns the DECLARING class file too, because the frame
/// runs against that class's constant pool.
fn resolve_static_method<'c>(
    classes: &'c HashMap<String, ClassFile>,
    class: &'c ClassFile,
    name: &str,
    descriptor: &str,
) -> Option<(&'c ClassFile, &'c MethodInfo)> {
    let mut current = Some(class);
    let mut steps = 0usize;
    while let Some(class) = current {
        steps += 1;
        if steps > classes.len() + 1 {
            return None; // cycle guard (the compiler rejects these anyway)
        }
        if let Some(method) = find_static_method(class, name, descriptor) {
            return Some((class, method));
        }
        current = class
            .constant_pool
            .get_class_name(class.super_class)
            .and_then(|super_name| classes.get(super_name));
    }
    None
}

fn find_static_method<'c>(
    class: &'c ClassFile,
    name: &str,
    descriptor: &str,
) -> Option<&'c MethodInfo> {
    class.methods.iter().find(|m| {
        m.access_flags
            .contains(caturra_classfile::MethodAccessFlags::STATIC)
            && class.constant_pool.get_utf8(m.name_index) == Some(name)
            && class.constant_pool.get_utf8(m.descriptor_index) == Some(descriptor)
    })
}

/// Local-variable slot widths of each argument in a method descriptor
/// (JVMS §2.6.1: double/long take two slots).
fn descriptor_arg_widths(descriptor: &str) -> Option<Vec<u16>> {
    let inner = descriptor.strip_prefix('(')?;
    let (args, _return_type) = inner.split_once(')')?;
    let mut widths = Vec::new();
    let mut chars = args.chars();
    while let Some(c) = chars.next() {
        match c {
            'B' | 'C' | 'F' | 'I' | 'S' | 'Z' => widths.push(1),
            'D' | 'J' => widths.push(2),
            'L' => {
                chars.by_ref().find(|&c| c == ';')?;
                widths.push(1);
            }
            '[' => {
                // Arrays are references (width 1) regardless of the
                // element type; consume the element descriptor.
                let mut element = chars.next()?;
                while element == '[' {
                    element = chars.next()?;
                }
                if element == 'L' {
                    chars.by_ref().find(|&c| c == ';')?;
                }
                widths.push(1);
            }
            _ => return None,
        }
    }
    Some(widths)
}

/// Number of arguments in a method descriptor (each argument is one
/// operand-stack entry in this VM regardless of JVMS slot width).
fn descriptor_arg_count(descriptor: &str) -> Option<usize> {
    descriptor_arg_widths(descriptor).map(|w| w.len())
}

/// A `Stream.limit`/`skip` count (a `long`, but occasionally passed as an int),
/// clamped to a usable index.
fn stream_count_arg(value: JValue) -> usize {
    let raw = match value {
        JValue::Long(n) => n,
        JValue::Int(n) => i64::from(n),
        _ => 0,
    };
    usize::try_from(raw).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_arg_counts() {
        assert_eq!(descriptor_arg_count("()V"), Some(0));
        assert_eq!(descriptor_arg_count("(I)V"), Some(1));
        assert_eq!(descriptor_arg_count("(IDZ)V"), Some(3));
        assert_eq!(descriptor_arg_count("(Ljava/lang/String;I)V"), Some(2));
        assert_eq!(descriptor_arg_count("([Ljava/lang/String;)V"), Some(1));
        assert_eq!(descriptor_arg_count("([[I[D)V"), Some(2));
        assert_eq!(descriptor_arg_count("bogus"), None);
    }
}
