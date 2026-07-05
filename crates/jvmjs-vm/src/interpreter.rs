//! The bytecode dispatch loop (JVMS §2.5–2.6, §6.5).
//!
//! A classic frame interpreter: locals + operand stack per method,
//! decoding one opcode at a time. The opcode surface grows in lockstep
//! with the compiler (`specs/LANGUAGE.md`); an opcode the VM doesn't
//! know yet is a `VmError`, never undefined behavior.

use std::collections::HashMap;
use std::rc::Rc;

use jvmjs_classfile::{
    CODE_ATTRIBUTE, ClassFile, CodeAttribute, Constant, MethodInfo, opcodes as op,
    read_code_attribute,
};

use crate::debug::{
    Breakpoint, DebugCommand, DebugFrameSnapshot, DebugHost, DebugSnapshot, PauseReason,
    WatchEvaluator,
};
use crate::intrinsics::{self, IntrinsicStatics};
use crate::io::ConsoleIo;
use crate::value::{Heap, HeapRef, JValue};
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
    /// Classes whose initialization has started (JVMS §5.5: recursive
    /// initialization by the same thread proceeds).
    init_started: std::collections::HashSet<String>,
    remaining_instructions: u64,
    max_call_depth: u32,
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
    /// Skip the pause check once at this address (just resumed there).
    resume_at: Option<usize>,
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
            init_started: std::collections::HashSet::new(),
            remaining_instructions: max_instructions,
            max_call_depth,
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
            resume_at: None,
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
        for suspended in self.frames.iter().rev() {
            let class_name = suspended.class.class_name().unwrap_or("<unknown>");
            let _ = write!(
                full,
                "\n\tat {class_name}.{}({})",
                suspended.method_name,
                location(&suspended.code.source_file, suspended.current_line)
            );
        }
        full
    }

    /// The per-instruction debugger hook: decides whether to pause at
    /// `addr` (line boundary hit, step goal met, or host interrupt) and
    /// blocks in `on_pause` if so. Returns the resume command.
    fn debug_checkpoint(
        &mut self,
        frame: &mut Frame<'run>,
        addr: usize,
        boundary_line: u16,
    ) -> Option<DebugCommand> {
        let depth = self.frames.len();
        let debug = self.debug.as_mut().expect("caller checked debug mode");

        // Just resumed: the first instruction after a pause is the one
        // we paused on — run it without re-pausing. The marker clears
        // immediately so looping back to the same address pauses again.
        if let Some(resume_addr) = debug.resume_at.take()
            && resume_addr == addr
        {
            return None;
        }

        // Host interrupt (pause / stop button), polled sparsely.
        debug.poll_in = debug.poll_in.saturating_sub(1);
        let interrupted = if debug.poll_in == 0 {
            debug.poll_in = INTERRUPT_POLL_INTERVAL;
            debug.host.interrupt_requested()
        } else {
            false
        };

        let at_line_boundary = boundary_line != 0;
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
        debug.resume_at = Some(addr);
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
            method_name: frame.method_name.clone(),
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
                Some(crate::value::HeapObject::IntArray(values)) => {
                    render_array(values.iter().map(i32::to_string))
                }
                Some(crate::value::HeapObject::DoubleArray(values)) => {
                    render_array(values.iter().map(|v| intrinsics::java_double_to_string(*v)))
                }
                Some(crate::value::HeapObject::RefArray(values)) => {
                    render_array(values.iter().map(|v| self.render_shallow(*v)))
                }
                Some(crate::value::HeapObject::ArrayList(values)) => {
                    render_array(values.iter().map(|v| self.render_shallow(*v)))
                }
                Some(crate::value::HeapObject::Instance { class_name, fields }) => {
                    if depth > 0 || fields.is_empty() {
                        return format!("{class_name}@{reference:x}");
                    }
                    // Sorted for stable display (fields live in a map);
                    // reserved names (the throwable message) hidden.
                    let mut names: Vec<&String> =
                        fields.keys().filter(|k| !k.starts_with("__")).collect();
                    names.sort();
                    let rendered: Vec<String> = names
                        .into_iter()
                        .map(|name| format!("{name}={}", self.render_shallow(fields[name])))
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
        if self.frames.len() >= usize::try_from(self.max_call_depth).unwrap_or(usize::MAX) {
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
            .unwrap_or("<unknown>")
            .to_owned();
        Ok(Frame {
            class,
            method_name,
            code,
            pc: 0,
            current_line: None,
            locals,
            stack: Vec::new(),
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
                method_name: frame.method_name.clone(),
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
                    if self.debug_checkpoint(&mut frame, addr, boundary_line)
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
                                    _ => false,
                                },
                            };
                            if opcode == op::INSTANCEOF {
                                frame.stack.push(JValue::Int(i32::from(matches_type)));
                            } else {
                                // checkcast: null always passes; a mismatch throws.
                                if reference.is_some() && !matches_type {
                                    let actual = match reference.and_then(|r| self.heap.get(r)) {
                                        Some(crate::value::HeapObject::Instance {
                                            class_name,
                                            ..
                                        }) => class_name.clone(),
                                        Some(crate::value::HeapObject::JavaString(_)) => {
                                            String::from("java/lang/String")
                                        }
                                        _ => String::from("<object>"),
                                    };
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
                                op::T_INT | op::T_BOOLEAN | op::T_CHAR => {
                                    crate::value::HeapObject::IntArray(vec![0; length])
                                }
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
                            let _class_index = read_u16(bytes, &mut pc, &malformed)?;
                            let length = check_array_size(frame.pop_int()?)?;
                            let reference =
                                self.heap.alloc(crate::value::HeapObject::RefArray(vec![
                            JValue::NULL;
                            length
                        ]));
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
                                Some(crate::value::HeapObject::IntArray(v)) => v.len(),
                                Some(crate::value::HeapObject::DoubleArray(v)) => v.len(),
                                Some(crate::value::HeapObject::RefArray(v)) => v.len(),
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
                                Some(crate::value::HeapObject::IntArray(values)) => {
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
                            let Some(crate::value::HeapObject::RefArray(values)) =
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
                                Some(crate::value::HeapObject::IntArray(values)) => {
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
                            let Some(crate::value::HeapObject::RefArray(values)) =
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
                    Ok(Flow::Return(value)) => match self.frames.pop() {
                        None => return Ok(value),
                        Some(caller) => {
                            frame = caller;
                            if let Some(value) = value {
                                frame.stack.push(value);
                            }
                            continue 'frames;
                        }
                    },
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
        let is_library = jvmjs_classfile::exceptions::is_exception_class(&internal);
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
    fn getfield_op(
        &mut self,
        class: &ClassFile,
        frame: &mut Frame,
        index: u16,
        malformed: &impl Fn(String) -> VmError,
    ) -> Result<(), VmError> {
        let (_, field_name, _) = class
            .constant_pool
            .get_member_ref(index)
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        let field_name = field_name.to_owned();
        let reference = frame.pop_ref()?.ok_or_else(|| {
            VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot read field \"{field_name}\" \
                 because the object is null"
            ))
        })?;
        let Some(crate::value::HeapObject::Instance { fields, .. }) = self.heap.get(reference)
        else {
            return Err(malformed(String::from("getfield on a non-object")));
        };
        let value = *fields
            .get(&field_name)
            .ok_or_else(|| malformed(format!("unknown field {field_name}")))?;
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
        let (_, field_name, _) = class
            .constant_pool
            .get_member_ref(index)
            .ok_or_else(|| malformed(format!("bad field ref at pool {index}")))?;
        let field_name = field_name.to_owned();
        let value = frame.pop()?;
        let reference = frame.pop_ref()?.ok_or_else(|| {
            VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot assign field \"{field_name}\" \
                 because the object is null"
            ))
        })?;
        let Some(crate::value::HeapObject::Instance { fields, .. }) = self.heap.get_mut(reference)
        else {
            return Err(malformed(String::from("putfield on a non-object")));
        };
        fields.insert(field_name, value);
        Ok(())
    }

    /// `invokespecial`: user constructors and `super.method(...)`
    /// (returned as a frame to push) or intrinsic `<init>`s (handled
    /// inline, `None`).
    #[inline(never)]
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
                        .contains(jvmjs_classfile::MethodAccessFlags::ABSTRACT)
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
                    .contains(jvmjs_classfile::FieldAccessFlags::STATIC)
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
    fn new_instance(&mut self, class_name: &str) -> crate::value::HeapObject {
        let classes: &'run HashMap<String, ClassFile> = self.classes;
        let mut fields = HashMap::new();
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
                    .contains(jvmjs_classfile::FieldAccessFlags::STATIC)
                {
                    continue;
                }
                let name = class
                    .constant_pool
                    .get_utf8(field.name_index)
                    .unwrap_or_default()
                    .to_owned();
                let descriptor = class
                    .constant_pool
                    .get_utf8(field.descriptor_index)
                    .unwrap_or_default();
                fields
                    .entry(name)
                    .or_insert_with(|| default_for_descriptor(descriptor));
            }
            current = class
                .constant_pool
                .get_class_name(class.super_class)
                .and_then(|super_name| classes.get(super_name));
        }
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
                return jvmjs_classfile::exceptions::is_exception_class(super_name);
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
        if jvmjs_classfile::exceptions::is_exception_class(&thrown_internal) {
            return jvmjs_classfile::exceptions::is_exception_subclass(
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
                return jvmjs_classfile::exceptions::is_exception_subclass(
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
                    .contains(jvmjs_classfile::MethodAccessFlags::STATIC)
                    && !m
                        .access_flags
                        .contains(jvmjs_classfile::MethodAccessFlags::ABSTRACT)
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
                        .contains(jvmjs_classfile::MethodAccessFlags::STATIC)
                        && !m
                            .access_flags
                            .contains(jvmjs_classfile::MethodAccessFlags::ABSTRACT)
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
                        .contains(jvmjs_classfile::MethodAccessFlags::STATIC)
                        && !m
                            .access_flags
                            .contains(jvmjs_classfile::MethodAccessFlags::ABSTRACT)
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
            let object = match element_descriptor {
                "I" | "Z" | "C" => crate::value::HeapObject::IntArray(vec![0; length]),
                "D" => crate::value::HeapObject::DoubleArray(vec![0.0; length]),
                _ => crate::value::HeapObject::RefArray(vec![JValue::NULL; length]),
            };
            return Ok(self.heap.alloc(object));
        }

        let mut rows = Vec::with_capacity(length);
        for _ in 0..length {
            let row = self.alloc_multi_array(element_descriptor, rest_counts, malformed)?;
            rows.push(JValue::Ref(Some(row)));
        }
        Ok(self.heap.alloc(crate::value::HeapObject::RefArray(rows)))
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
        let method = find_static_method(class, method_name, descriptor).ok_or_else(|| {
            VmError::MalformedClass {
                name: class_name.to_owned(),
                reason: format!("no static method {method_name}{descriptor}"),
            }
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

        let JValue::Ref(receiver) = frame.pop()? else {
            return Err(malformed(String::from(
                "invokevirtual receiver is not a reference",
            )));
        };
        let Some(receiver) = receiver else {
            return Err(VmError::UncaughtException(format!(
                "java.lang.NullPointerException: cannot invoke \
                 \"{target_class}.{method_name}\" because the receiver is null"
            )));
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
            )
        );
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
                    "getSimpleName" | "getName" => {
                        let simple = simple_class_name(&name);
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(simple)))))
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
                        let fields: Vec<(String, String, u16)> = self
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
                                        )
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        let simple = simple_class_name(&name).to_owned();
                        let refs: Vec<JValue> = fields
                            .into_iter()
                            .map(|(field_name, descriptor, access)| {
                                JValue::Ref(Some(self.heap.alloc(HeapObject::Field {
                                    declaring: simple.clone(),
                                    name: field_name,
                                    descriptor,
                                    access,
                                })))
                            })
                            .collect();
                        let array = self.heap.alloc(HeapObject::RefArray(refs));
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
                                                jvmjs_classfile::MethodAccessFlags::PUBLIC,
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
                        let array = self.heap.alloc(HeapObject::RefArray(refs));
                        Ok(Some(JValue::Ref(Some(array))))
                    }
                    other => Err(VmError::UnknownIntrinsic(format!("Class.{other}"))),
                }
            }
            Some(HeapObject::Field {
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
                    "getType" => {
                        let type_name = intrinsics::type_name_of_descriptor(&descriptor);
                        let reference = self.heap.alloc(HeapObject::Class { name: type_name });
                        Ok(Some(JValue::Ref(Some(reference))))
                    }
                    "toString" => {
                        let text =
                            intrinsics::field_to_string(&declaring, &name, &descriptor, access);
                        Ok(Some(JValue::Ref(Some(self.heap.alloc_string(&text)))))
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
            _ => Err(VmError::UnknownIntrinsic(format!("reflect.{method}"))),
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
        let saved_budget = interp.remaining_instructions;
        interp.remaining_instructions = saved_budget.min(WATCH_INSTRUCTION_BUDGET);

        let result = interp.execute(class, method, locals);

        let spent = saved_budget.min(WATCH_INSTRUCTION_BUDGET) - interp.remaining_instructions;
        interp.remaining_instructions = saved_budget.saturating_sub(spent);
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
    method_name: String,
    code: Rc<MethodCode>,
    pc: usize,
    /// The last source line crossed (1-based), from `LineNumberTable`.
    current_line: Option<u16>,
    locals: Vec<JValue>,
    stack: Vec<JValue>,
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
            Some(jvmjs_classfile::debug::LINE_NUMBER_TABLE_ATTRIBUTE) => {
                for (start_pc, line) in
                    jvmjs_classfile::debug::decode_line_number_table(&nested.info)
                        .unwrap_or_default()
                {
                    if let Some(slot) = boundary_lines.get_mut(usize::from(start_pc)) {
                        *slot = line;
                    }
                }
            }
            Some(jvmjs_classfile::debug::LOCAL_VARIABLE_TABLE_ATTRIBUTE) => {
                for entry in jvmjs_classfile::debug::decode_local_variable_table(&nested.info)
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
            Some(jvmjs_classfile::debug::LOCAL_VARIABLE_TYPE_TABLE_ATTRIBUTE) => {
                for entry in jvmjs_classfile::debug::decode_local_variable_table(&nested.info)
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
                == Some(jvmjs_classfile::debug::SOURCE_FILE_ATTRIBUTE)
        })
        .and_then(|a| jvmjs_classfile::debug::decode_source_file(&a.info))
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
/// jvmjs is a flat namespace, so this is usually a no-op.
fn simple_class_name(name: &str) -> &str {
    name.rsplit(['/', '.']).next().unwrap_or(name)
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
fn find_static_method<'c>(
    class: &'c ClassFile,
    name: &str,
    descriptor: &str,
) -> Option<&'c MethodInfo> {
    class.methods.iter().find(|m| {
        m.access_flags
            .contains(jvmjs_classfile::MethodAccessFlags::STATIC)
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
