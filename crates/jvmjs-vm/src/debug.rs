//! The debugger host interface: breakpoints, stepping, and paused-state
//! snapshots.
//!
//! The engine stays synchronous: when execution pauses, the VM calls
//! [`DebugHost::on_pause`] and *blocks* until it returns a command —
//! the same blocking-host-callback pattern the console uses for stdin
//! (in the browser: `SharedArrayBuffer` + Atomics in a Web Worker).
//!
//! Pauses happen at source-line boundaries, located via the
//! `LineNumberTable` the compiler emits; locals are named via its
//! `LocalVariableTable`. Frames are plain data (`specs/RUNTIME.md`),
//! so snapshots are cheap walks, not stack unwinds.

/// A source breakpoint, keyed the traditional way: file + 1-based line.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Breakpoint {
    /// Source file name as compiled (e.g. `Main.java`).
    pub file: String,
    pub line: u32,
}

/// Why execution paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseReason {
    Breakpoint,
    Step,
    /// The host requested a pause (pause button).
    Interrupt,
}

/// How to proceed after a pause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCommand {
    /// Run until the next breakpoint (or interrupt).
    Continue,
    /// Pause at the next line in this frame or a caller.
    StepOver,
    /// Pause at the next line anywhere, including called methods.
    StepInto,
    /// Pause at the next line after the current method returns.
    StepOut,
    /// Stop the program.
    Terminate,
}

/// The host's response to a pause: a command, plus an optional full
/// replacement of the breakpoint set (edits made while paused).
#[derive(Debug, Clone)]
pub struct DebugControl {
    pub command: DebugCommand,
    pub breakpoints: Option<Vec<Breakpoint>>,
}

/// One named local at a pause point.
#[derive(Debug, Clone)]
pub struct LocalSnapshot {
    pub name: String,
    /// Java source type, fully qualified outside `java.lang` (empty
    /// when unrepresentable) — enough to synthesize code against.
    pub type_name: String,
    /// Rendered value for display.
    pub value: String,
}

/// One paused frame, innermost first in [`DebugSnapshot::frames`].
#[derive(Debug, Clone)]
pub struct DebugFrameSnapshot {
    pub class_name: String,
    pub method_name: String,
    pub source_file: String,
    /// Current 1-based source line, when line info covers the pc.
    pub line: Option<u32>,
    /// Named locals live at the pc.
    pub locals: Vec<LocalSnapshot>,
}

/// Everything the host needs to render a paused program.
#[derive(Debug, Clone)]
pub struct DebugSnapshot {
    pub reason: PauseReason,
    pub frames: Vec<DebugFrameSnapshot>,
}

/// Evaluates watch expressions against the paused program, provided by
/// the VM to [`DebugHost::on_pause`].
///
/// The host compiles a watch expression into a class holding one
/// static method whose parameters mirror (by name) the innermost
/// frame's locals; the VM invokes it with the live values — heap
/// references and statics shared, so `list.size()` sees the real
/// list. Runs under a small instruction budget; errors (compile
/// problems, exceptions, budget) come back as `Err(message)`.
pub trait WatchEvaluator {
    /// Invoke `class.{method_name}` with the named locals of the
    /// innermost paused frame as arguments, returning the rendered
    /// result.
    fn evaluate(
        &mut self,
        class: &jvmjs_classfile::ClassFile,
        method_name: &str,
        param_names: &[String],
    ) -> Result<String, String>;
}

/// The host side of a debug session. Supplied to
/// [`crate::Vm::run_main_debug`].
pub trait DebugHost {
    /// Execution is paused; block until the user picks a command.
    /// `watch` evaluates expressions against the paused state.
    fn on_pause(
        &mut self,
        snapshot: &DebugSnapshot,
        watch: &mut dyn WatchEvaluator,
    ) -> DebugControl;

    /// Polled periodically while running: return `true` to request a
    /// pause (a pause button; also how a stop button interrupts an
    /// infinite loop — respond to the pause with `Terminate`).
    fn interrupt_requested(&mut self) -> bool {
        false
    }
}
