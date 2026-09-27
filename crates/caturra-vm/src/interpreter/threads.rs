//! Java threads on a single-threaded engine (specs/CONCURRENCY.md, phase 1).
//!
//! Every thread is DATA: a saved frame stack and a park state. One runs at a
//! time; the interpreter swaps its `frames` for another thread's at the
//! safepoint, when the running thread's quantum is spent or it parks. A
//! blocking call never blocks the host: it rewinds `pc` to the invoking
//! instruction and parks ([`Flow::Park`]), and the scheduler re-executes the
//! same instruction once the thread may run again — which then consults
//! [`JavaThread::wake`] to learn how the wait ended.
//!
//! The host waits only when EVERY live thread is parked and one of them is
//! waiting on time; if none is, nothing can ever wake them, and the run ends
//! with a thread dump rather than hanging a browser tab.

use std::collections::{HashMap, VecDeque};

use super::{Flow, Frame, Interpreter, resolve_virtual};
use crate::value::{HeapRef, JValue};
use crate::vm::VmError;

/// Instructions a thread runs before the next runnable one gets its turn.
/// Small enough that a slice boundary lands inside an unsynchronized `count++`
/// often enough for the textbook race to lose updates (a two-thread
/// 100,000-increment race loses hundreds here); large enough that a short
/// thread — a handful of `println`s — finishes inside its first slice, as it
/// does on a JDK, where starting a thread costs far more than running one.
const QUANTUM: u64 = 10_000;

/// How long one host wait may last before the run looks around again — the
/// same slice `Thread.sleep` waits in, so a debugger's interrupt is polled.
const WAIT_SLICE: i64 = 100;

/// What a parked thread is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Park {
    Runnable,
    Sleeping {
        until: i64,
    },
    Joining {
        target: usize,
        until: Option<i64>,
    },
    /// Entering a monitor another thread holds.
    Blocked {
        monitor: HeapRef,
    },
    /// Inside `Object.wait`: the monitor was released, to be re-taken at
    /// `count` once the wait ends.
    Waiting {
        monitor: HeapRef,
        until: Option<i64>,
        notified: bool,
        count: u32,
    },
    Terminated,
}

/// How a park ended — read by the re-executed call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Wake {
    /// The time ran out, the target finished, or a notify arrived.
    Done,
    Interrupted,
}

pub(super) struct JavaThread<'run> {
    /// The `java.lang.Thread`; `main`'s is made the first time a program
    /// asks for it.
    pub(super) object: Option<HeapRef>,
    /// The whole Java stack while the thread is not running, top frame last.
    pub(super) frames: Vec<Frame<'run>>,
    pub(super) park: Park,
    pub(super) wake: Option<Wake>,
    pub(super) interrupted: bool,
    pub(super) daemon: bool,
}

impl JavaThread<'_> {
    fn new(object: Option<HeapRef>, daemon: bool) -> Self {
        JavaThread {
            object,
            frames: Vec::new(),
            park: Park::Runnable,
            wake: None,
            interrupted: false,
            daemon,
        }
    }

    fn is_alive(&self) -> bool {
        self.park != Park::Terminated
    }
}

/// A monitor that is held or waited on. An entry exists only while one of
/// the two is true.
#[derive(Debug, Default)]
pub(super) struct Monitor {
    owner: Option<usize>,
    count: u32,
    /// Threads inside `wait` that no notify has chosen yet, oldest first.
    waiters: VecDeque<usize>,
}

pub(super) struct Threads<'run> {
    /// Index 0 is `main`.
    pub(super) list: Vec<JavaThread<'run>>,
    pub(super) current: usize,
    pub(super) by_object: HashMap<HeapRef, usize>,
    pub(super) monitors: HashMap<HeapRef, Monitor>,
    /// Set while `main`'s own run is the one executing: only there may a
    /// thread end, or the running thread change.
    pub(super) scheduling: bool,
    /// `main` ended with an uncaught exception while other threads were
    /// still running; its banner was printed when it happened, and the run
    /// reports it once they have finished.
    pub(super) main_failure: Option<String>,
    /// That banner has been printed already.
    pub(super) uncaught_reported: bool,
}

impl<'run> Threads<'run> {
    pub(super) fn new() -> Self {
        Threads {
            list: vec![JavaThread::new(None, false)],
            current: 0,
            by_object: HashMap::new(),
            monitors: HashMap::new(),
            scheduling: false,
            main_failure: None,
            uncaught_reported: false,
        }
    }

    /// Every reference the scheduler holds: live threads' objects and
    /// stacks, and every monitor in use.
    pub(super) fn roots(&self) -> impl Iterator<Item = JValue> + '_ {
        let objects = self
            .list
            .iter()
            .filter(|thread| thread.is_alive())
            .filter_map(|thread| thread.object)
            .chain(self.monitors.keys().copied())
            .map(|reference| JValue::Ref(Some(reference)));
        let frames = self
            .list
            .iter()
            .flat_map(|thread| thread.frames.iter())
            .flat_map(|frame| frame.locals.iter().chain(frame.stack.iter()).copied());
        objects.chain(frames)
    }

    /// Forget the `Thread` objects of finished threads the program no longer
    /// holds (a live thread's is a root, so it is never swept).
    pub(super) fn prune(&mut self, alive: impl Fn(&HeapRef) -> bool) {
        self.by_object.retain(|object, _| alive(object));
        for thread in &mut self.list {
            if thread.object.is_some_and(|object| !alive(&object)) {
                thread.object = None;
            }
        }
    }

    fn live_non_daemon(&self) -> bool {
        self.list
            .iter()
            .any(|thread| thread.is_alive() && !thread.daemon)
    }

    fn free(&self, monitor: HeapRef) -> bool {
        self.monitors
            .get(&monitor)
            .is_none_or(|entry| entry.owner.is_none())
    }

    /// Whether thread `index` may run now, making it so — and settling how
    /// its park ended — if it may.
    fn try_wake(&mut self, index: usize, now: i64) -> bool {
        let thread = &self.list[index];
        let interrupted = thread.interrupted;
        let due = |until: Option<i64>| until.is_some_and(|until| now >= until);
        let wake = match thread.park {
            Park::Runnable => return true,
            Park::Terminated => return false,
            Park::Sleeping { until } => {
                if interrupted {
                    Wake::Interrupted
                } else if now >= until {
                    Wake::Done
                } else {
                    return false;
                }
            }
            Park::Joining { target, until } => {
                if interrupted {
                    Wake::Interrupted
                } else if !self.list[target].is_alive() || due(until) {
                    Wake::Done
                } else {
                    return false;
                }
            }
            Park::Blocked { monitor } => {
                if !self.free(monitor) {
                    return false;
                }
                // The re-executed `monitorenter` takes it.
                self.list[index].park = Park::Runnable;
                return true;
            }
            Park::Waiting {
                monitor,
                until,
                notified,
                count,
            } => {
                if !(notified || interrupted || due(until)) || !self.free(monitor) {
                    return false;
                }
                let entry = self.monitors.entry(monitor).or_default();
                entry.waiters.retain(|waiter| *waiter != index);
                entry.owner = Some(index);
                entry.count = count;
                if notified {
                    Wake::Done
                } else if interrupted {
                    Wake::Interrupted
                } else {
                    Wake::Done
                }
            }
        };
        let thread = &mut self.list[index];
        thread.park = Park::Runnable;
        thread.wake = Some(wake);
        true
    }

    /// Whether thread `index` could run now — `try_wake`'s question, asked
    /// without settling anything.
    fn could_run(&self, index: usize, now: i64) -> bool {
        let thread = &self.list[index];
        let due = |until: Option<i64>| until.is_some_and(|until| now >= until);
        match thread.park {
            Park::Runnable => true,
            Park::Terminated => false,
            Park::Sleeping { until } => thread.interrupted || now >= until,
            Park::Joining { target, until } => {
                thread.interrupted || !self.list[target].is_alive() || due(until)
            }
            Park::Blocked { monitor } => self.free(monitor),
            Park::Waiting {
                monitor,
                until,
                notified,
                ..
            } => (notified || thread.interrupted || due(until)) && self.free(monitor),
        }
    }

    /// The earliest moment a parked thread's wait runs out, if any is timed.
    fn next_deadline(&self) -> Option<i64> {
        self.list
            .iter()
            .filter_map(|thread| match thread.park {
                Park::Sleeping { until } => Some(until),
                Park::Joining { until, .. } | Park::Waiting { until, .. } => until,
                _ => None,
            })
            .min()
    }

    /// `Thread.getState()`'s ordinal (NEW, RUNNABLE, BLOCKED, WAITING,
    /// TIMED_WAITING, TERMINATED).
    fn status(&self, index: usize) -> i32 {
        match self.list[index].park {
            Park::Runnable => 1,
            Park::Blocked { .. } | Park::Waiting { notified: true, .. } => 2,
            Park::Joining { until: None, .. } | Park::Waiting { until: None, .. } => 3,
            Park::Sleeping { .. } | Park::Joining { .. } | Park::Waiting { .. } => 4,
            Park::Terminated => 5,
        }
    }
}

impl<'run> Interpreter<'run> {
    /// The thread behind a `Thread` object, if it has been started (or is
    /// `main`).
    fn thread_of(&self, value: Option<&JValue>) -> Option<usize> {
        match value {
            Some(JValue::Ref(Some(object))) => self.threads.by_object.get(object).copied(),
            _ => None,
        }
    }

    /// One thread as the debugger lists it.
    pub(super) fn snapshot_thread(
        &self,
        index: usize,
        frames: Vec<crate::debug::DebugFrameSnapshot>,
    ) -> crate::debug::DebugThreadSnapshot {
        const STATES: [&str; 6] = [
            "NEW",
            "RUNNABLE",
            "BLOCKED",
            "WAITING",
            "TIMED_WAITING",
            "TERMINATED",
        ];
        let current = index == self.threads.current;
        let state = if current {
            "RUNNABLE"
        } else {
            let ordinal = usize::try_from(self.threads.status(index)).unwrap_or(1);
            STATES.get(ordinal).copied().unwrap_or("RUNNABLE")
        };
        crate::debug::DebugThreadSnapshot {
            name: self.thread_name(index),
            state,
            daemon: self.threads.list[index].daemon,
            current,
            frames,
        }
    }

    /// A thread's name as the program last set it, for the uncaught banner.
    pub(super) fn thread_name(&self, index: usize) -> String {
        self.threads.list[index]
            .object
            .and_then(|object| self.heap.get(object)?.field("java/lang/Thread.__name"))
            .and_then(|name| match name {
                JValue::Ref(Some(text)) => self.heap.string_text(text),
                _ => None,
            })
            .unwrap_or_else(|| String::from(if index == 0 { "main" } else { "?" }))
    }

    /// Start the next slice: another thread gets a turn after `QUANTUM`
    /// instructions, if there is another live thread at all.
    fn start_slice(&mut self) {
        let others = self
            .threads
            .list
            .iter()
            .enumerate()
            .any(|(index, thread)| index != self.threads.current && thread.is_alive());
        self.slice_end = if others {
            self.remaining_instructions.saturating_sub(QUANTUM)
        } else {
            0
        };
    }

    /// Whether the running thread may be switched away from here: in `main`'s
    /// run, at the base nesting level. A thread inside a callback that native
    /// code invoked cannot be — its caller is on the host stack.
    pub(super) fn may_switch(&self) -> bool {
        self.threads.scheduling && self.suspended_runs.is_empty()
    }

    /// Park the running thread (whose top frame is `frame`) and resume
    /// whichever runs next — possibly the same one.
    pub(super) fn switch_from(&mut self, frame: Frame<'run>) -> Result<Frame<'run>, VmError> {
        self.frames.push(frame);
        let current = self.threads.current;
        self.threads.list[current].frames = std::mem::take(&mut self.frames);
        self.resume_next()
    }

    /// Choose the next thread to run, waiting on the host clock if every
    /// live one is parked on time, and install its stack.
    pub(super) fn resume_next(&mut self) -> Result<Frame<'run>, VmError> {
        let count = self.threads.list.len();
        loop {
            let now = self.console.now_millis();
            let current = self.threads.current;
            let chosen = (1..=count)
                .map(|offset| (current + offset) % count)
                .find(|&index| {
                    !self.threads.list[index].frames.is_empty() && self.threads.try_wake(index, now)
                });
            if let Some(next) = chosen {
                self.threads.current = next;
                self.frames = std::mem::take(&mut self.threads.list[next].frames);
                let frame = self.frames.pop().ok_or(VmError::StackUnderflow)?;
                self.start_slice();
                return Ok(frame);
            }
            let Some(deadline) = self.threads.next_deadline() else {
                return Err(VmError::Unsupported(self.deadlock_report()));
            };
            let wait = (deadline - now).clamp(1, WAIT_SLICE);
            let wait = u32::try_from(wait).unwrap_or(1);
            self.console.wait_millis(wait);
        }
    }

    /// The running thread's stack has emptied: it is over. Answers the frame
    /// to continue with, or `None` when no thread that keeps the program
    /// alive is left.
    pub(super) fn thread_finished(&mut self) -> Result<Option<Frame<'run>>, VmError> {
        let current = self.threads.current;
        self.threads.list[current].park = Park::Terminated;
        self.threads.list[current].interrupted = false;
        if !self.threads.live_non_daemon() {
            return Ok(None);
        }
        self.resume_next().map(Some)
    }

    /// Whether a thread other than the running one keeps the program alive.
    pub(super) fn others_keep_running(&self) -> bool {
        let current = self.threads.current;
        current != 0
            || self
                .threads
                .list
                .iter()
                .enumerate()
                .any(|(index, thread)| index != current && thread.is_alive() && !thread.daemon)
    }

    /// The last thread that keeps the program alive has ended: the run's
    /// outcome is `main`'s.
    pub(super) fn finish_run(&mut self, value: Option<JValue>) -> Result<Option<JValue>, VmError> {
        match self.threads.main_failure.take() {
            Some(message) => Err(VmError::UncaughtException(message)),
            None if self.threads.current == 0 => Ok(value),
            None => Ok(None),
        }
    }

    /// The run ends with every thread parked and none waiting on time.
    fn deadlock_report(&self) -> String {
        use std::fmt::Write as _;
        let mut report = String::from(
            "caturra: every thread is blocked and none is waiting on time (on a JDK, the program would hang here)",
        );
        for (index, thread) in self.threads.list.iter().enumerate() {
            let what = match thread.park {
                Park::Joining { target, .. } => {
                    format!("joining \"{}\"", self.thread_name(target))
                }
                Park::Blocked { monitor } => {
                    let owner = self
                        .threads
                        .monitors
                        .get(&monitor)
                        .and_then(|entry| entry.owner)
                        .map_or_else(String::new, |owner| {
                            format!(" held by \"{}\"", self.thread_name(owner))
                        });
                    format!("blocked on {}{owner}", self.monitor_label(monitor))
                }
                Park::Waiting { monitor, .. } => {
                    format!("waiting on {}", self.monitor_label(monitor))
                }
                _ => continue,
            };
            let _ = write!(report, "\n  \"{}\" {what}", self.thread_name(index));
        }
        // The commonest way to get here: a pool's threads wait for work until
        // the pool is shut down, and a program that never calls `shutdown()`
        // leaves them waiting — a JDK does not exit either.
        let idle_pool = self.threads.list.iter().any(|thread| {
            matches!(thread.park, Park::Waiting { monitor, .. }
                if self.object_class_name(monitor).ends_with("ThreadPoolExecutor"))
        });
        if idle_pool {
            report.push_str(
                "\n(an ExecutorService's threads wait for more work until it is shut down: \
                 call shutdown() once every task is submitted)",
            );
        }
        report
    }

    fn monitor_label(&self, monitor: HeapRef) -> String {
        let class = self.object_class_name(monitor);
        let simple = class.rsplit(['.', '/']).next().unwrap_or(&class).to_owned();
        let article = if simple.starts_with(['A', 'E', 'I', 'O', 'U']) {
            "an"
        } else {
            "a"
        };
        format!("{article} {simple}")
    }

    /// A blocking call made inside a callback: the thread that would end
    /// the wait cannot run while native code is on the host stack.
    fn nested_wait_refusal(&self, what: &str) -> VmError {
        VmError::Unsupported(format!(
            "caturra: \"{}\" cannot {what} inside a callback that library code invoked \
             (a comparator, a forEach body, an event handler); move the wait outside the callback",
            self.thread_name(self.threads.current)
        ))
    }

    /// Rewind to the invoking instruction and park the running thread.
    fn park(&mut self, frame: &mut Frame<'run>, addr: usize, park: Park) -> Flow<'run> {
        let current = self.threads.current;
        self.threads.list[current].park = park;
        frame.pc = addr;
        Flow::Park
    }

    /// Pop `count` arguments the rewind left on the stack.
    fn drop_args(frame: &mut Frame<'run>, count: usize) -> Result<Vec<JValue>, VmError> {
        let mut args = Vec::with_capacity(count);
        for _ in 0..count {
            args.push(frame.pop()?);
        }
        args.reverse();
        Ok(args)
    }

    /// `__System.__…`, the scheduler's half of the bundled `Thread` and of
    /// `synchronized`. The arguments are still on the stack: a call that
    /// parks leaves them there for its re-execution. `None` for a name that
    /// is not the scheduler's.
    #[allow(clippy::too_many_lines)] // one arm per call
    pub(super) fn scheduler_call(
        &mut self,
        frame: &mut Frame<'run>,
        addr: usize,
        method: &str,
    ) -> Result<Option<Flow<'run>>, VmError> {
        let current = self.threads.current;
        let peek = |frame: &Frame<'run>, depth: usize| -> Option<JValue> {
            frame
                .stack
                .len()
                .checked_sub(depth + 1)
                .map(|at| frame.stack[at])
        };
        let flow = match method {
            "__sleep" => {
                let wake = self.threads.list[current].wake.take();
                let Some(JValue::Long(millis)) = peek(frame, 0) else {
                    return Ok(None);
                };
                let interrupted = match wake {
                    Some(wake) => wake == Wake::Interrupted,
                    None if self.threads.list[current].interrupted => true,
                    None if millis <= 0 => false,
                    None if !self.may_switch() => {
                        // Inside a callback the host really waits, and nothing
                        // else runs meanwhile.
                        let mut left = u64::try_from(millis).unwrap_or(0);
                        while left > 0 {
                            let slice = u32::try_from(left.min(100)).unwrap_or(100);
                            let waited = self.console.wait_millis(slice).clamp(1, slice);
                            left -= u64::from(waited);
                        }
                        false
                    }
                    None => {
                        let until = self.console.now_millis().saturating_add(millis);
                        return Ok(Some(self.park(frame, addr, Park::Sleeping { until })));
                    }
                };
                Self::drop_args(frame, 1)?;
                if interrupted {
                    self.threads.list[current].interrupted = false;
                }
                frame.stack.push(JValue::Int(i32::from(interrupted)));
                Flow::Next
            }
            "__join" => {
                let wake = self.threads.list[current].wake.take();
                let Some(JValue::Long(millis)) = peek(frame, 0) else {
                    return Ok(None);
                };
                let target = self.thread_of(peek(frame, 1).as_ref());
                let interrupted = match (wake, target) {
                    (Some(wake), _) => wake == Wake::Interrupted,
                    _ if self.threads.list[current].interrupted => true,
                    (None, None) => false,
                    (None, Some(target)) if !self.threads.list[target].is_alive() => false,
                    (None, Some(target)) => {
                        if !self.may_switch() {
                            let name = self.thread_name(target);
                            return Err(self.nested_wait_refusal(&format!("wait for \"{name}\"")));
                        }
                        let until =
                            (millis > 0).then(|| self.console.now_millis().saturating_add(millis));
                        return Ok(Some(self.park(
                            frame,
                            addr,
                            Park::Joining { target, until },
                        )));
                    }
                };
                Self::drop_args(frame, 2)?;
                if interrupted {
                    self.threads.list[current].interrupted = false;
                }
                frame.stack.push(JValue::Int(i32::from(interrupted)));
                Flow::Next
            }
            "__monitorEnter" => {
                let Some(JValue::Ref(Some(monitor))) = peek(frame, 0) else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                let entry = self.threads.monitors.entry(monitor).or_default();
                match entry.owner {
                    None => {
                        entry.owner = Some(current);
                        entry.count = 1;
                    }
                    Some(owner) if owner == current => entry.count += 1,
                    Some(_) => {
                        if !self.may_switch() {
                            let label = self.monitor_label(monitor);
                            return Err(self
                                .nested_wait_refusal(&format!("wait for the monitor of {label}")));
                        }
                        return Ok(Some(self.park(frame, addr, Park::Blocked { monitor })));
                    }
                }
                Self::drop_args(frame, 1)?;
                Flow::Next
            }
            "__monitorExit" => {
                let Some(JValue::Ref(Some(monitor))) = peek(frame, 0) else {
                    return Err(VmError::UncaughtException(String::from(
                        "java.lang.NullPointerException",
                    )));
                };
                Self::drop_args(frame, 1)?;
                self.release(monitor, current)?;
                Flow::Next
            }
            "__holdsLock" => {
                let args = Self::drop_args(frame, 1)?;
                let held = match args.first() {
                    Some(JValue::Ref(Some(monitor))) => self
                        .threads
                        .monitors
                        .get(monitor)
                        .is_some_and(|entry| entry.owner == Some(current)),
                    _ => false,
                };
                frame.stack.push(JValue::Int(i32::from(held)));
                Flow::Next
            }
            "__start" => {
                let args = Self::drop_args(frame, 2)?;
                let (Some(JValue::Ref(Some(object))), Some(JValue::Int(daemon))) =
                    (args.first(), args.get(1))
                else {
                    return Ok(Some(Flow::Next));
                };
                self.start_thread(*object, *daemon != 0)?;
                Flow::Next
            }
            "__setMain" => {
                let args = Self::drop_args(frame, 1)?;
                if let Some(JValue::Ref(Some(object))) = args.first() {
                    self.threads.list[0].object = Some(*object);
                    self.threads.by_object.insert(*object, 0);
                }
                Flow::Next
            }
            "__currentThread" => {
                let object = self.threads.list[current].object;
                frame.stack.push(JValue::Ref(object));
                Flow::Next
            }
            "__threadStatus" => {
                let args = Self::drop_args(frame, 1)?;
                let status = self
                    .thread_of(args.first())
                    .map_or(0, |index| self.threads.status(index));
                frame.stack.push(JValue::Int(status));
                Flow::Next
            }
            "__interrupt" => {
                let args = Self::drop_args(frame, 1)?;
                if let Some(index) = self.thread_of(args.first())
                    && self.threads.list[index].is_alive()
                {
                    self.threads.list[index].interrupted = true;
                }
                Flow::Next
            }
            "__isInterrupted" => {
                let args = Self::drop_args(frame, 2)?;
                let clear = matches!(args.get(1), Some(JValue::Int(1)));
                let answer = match self.thread_of(args.first()) {
                    Some(index) => {
                        let thread = &mut self.threads.list[index];
                        let was = thread.interrupted;
                        if clear {
                            thread.interrupted = false;
                        }
                        was
                    }
                    None => false,
                };
                frame.stack.push(JValue::Int(i32::from(answer)));
                Flow::Next
            }
            "__uiWait" => return self.ui_wait(frame).map(Some),
            "__uiDialog" => return self.ui_dialog_wait(frame, addr).map(Some),
            "__yield" => {
                if self.may_switch() {
                    self.slice_end = u64::MAX;
                }
                Flow::Next
            }
            _ => return Ok(None),
        };
        Ok(Some(flow))
    }

    /// How long a wait for the window (or a dialog) may hold the host, and
    /// whether another thread could run right now. Another runnable thread:
    /// only LOOK (zero). Every other thread parked: until the first of their
    /// timed waits runs out, or `own`, whichever is first. No other thread
    /// (or inside a callback, where none can run): `own`.
    fn host_wait_budget(&mut self, own: Option<u32>) -> (Option<u32>, bool) {
        let current = self.threads.current;
        let now = self.console.now_millis();
        let others: Vec<usize> = (0..self.threads.list.len())
            .filter(|&index| {
                index != current
                    && self.threads.list[index].is_alive()
                    && !self.threads.list[index].frames.is_empty()
            })
            .collect();
        let can_switch = self.may_switch() && !others.is_empty();
        let others_ready = can_switch
            && others
                .iter()
                .any(|&index| self.threads.could_run(index, now));
        let timeout = if others_ready {
            Some(0)
        } else if can_switch {
            let theirs = others
                .iter()
                .filter_map(|&index| match self.threads.list[index].park {
                    Park::Sleeping { until } => Some(until),
                    Park::Joining { until, .. } | Park::Waiting { until, .. } => until,
                    _ => None,
                })
                .min()
                .map(|until| u32::try_from((until - now).max(0)).unwrap_or(u32::MAX));
            match (own, theirs) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            }
        } else {
            own
        };
        (timeout, others_ready)
    }

    /// `JOptionPane`'s dialog (`System.__uiDialog(kind, message)`): shown once,
    /// and waited for on the same terms as the window, so other threads run
    /// while it is up. A wait that runs out parks the thread — for a moment
    /// if others have work, or just long enough for the one whose wait ran
    /// out to go first — and the re-executed call asks again; the host does
    /// not show the same dialog twice.
    fn ui_dialog_wait(
        &mut self,
        frame: &mut Frame<'run>,
        addr: usize,
    ) -> Result<Flow<'run>, VmError> {
        let current = self.threads.current;
        self.threads.list[current].wake = None;
        let text_at = |this: &Self, frame: &Frame<'run>, depth: usize| {
            frame
                .stack
                .len()
                .checked_sub(depth + 1)
                .and_then(|at| match frame.stack[at] {
                    JValue::Ref(Some(text)) => this.heap.string_text(text),
                    _ => None,
                })
                .unwrap_or_default()
        };
        let (kind, message) = (text_at(self, frame, 1), text_at(self, frame, 0));
        let (timeout, others_ready) = self.host_wait_budget(None);
        let answer = match self.console.ui_poll_dialog(&kind, &message, timeout) {
            crate::io::UiPoll::Event(answer) => Some(answer),
            crate::io::UiPoll::Closed => None,
            crate::io::UiPoll::TimedOut => {
                let pause = if others_ready { 20 } else { 0 };
                let until = self.console.now_millis().saturating_add(pause);
                return Ok(self.park(frame, addr, Park::Sleeping { until }));
            }
        };
        Self::drop_args(frame, 2)?;
        let value = answer.map(|answer| self.heap.alloc_string(&answer));
        frame.stack.push(JValue::Ref(value));
        Ok(Flow::Next)
    }

    /// `__System.__uiWait(tree, timeout)`: the event-dispatch thread waits for
    /// its window. It never holds the host while another thread could run:
    /// then it only LOOKS (a zero timeout), and with nothing there answers
    /// `"__busy"` — the dispatch thread waits a moment on its own queue, where
    /// a posted task wakes it, and looks again, re-rendering what the others
    /// changed. When every other thread is parked the host waits — until the
    /// window says something, the given timeout passes, or the first other
    /// thread's wait runs out — and a wait that ran out answers `"__idle"`.
    fn ui_wait(&mut self, frame: &mut Frame<'run>) -> Result<Flow<'run>, VmError> {
        let answer = |this: &mut Self, frame: &mut Frame<'run>, text: Option<&str>| {
            Self::drop_args(frame, 2)?;
            let value = text.map(|text| this.heap.alloc_string(text));
            frame.stack.push(JValue::Ref(value));
            Ok(Flow::Next)
        };
        let stack_len = frame.stack.len();
        let (Some(JValue::Ref(tree)), Some(JValue::Long(own))) = (
            stack_len.checked_sub(2).map(|at| frame.stack[at]),
            stack_len.checked_sub(1).map(|at| frame.stack[at]),
        ) else {
            return Err(VmError::UnknownIntrinsic(String::from("__uiWait")));
        };
        let tree = tree
            .and_then(|tree| self.heap.string_text(tree))
            .unwrap_or_default();
        let own = u32::try_from(own).ok();
        let (timeout, others_ready) = self.host_wait_budget(own);
        match self.console.ui_poll_event(&tree, timeout) {
            crate::io::UiPoll::Event(payload) => answer(self, frame, Some(&payload)),
            crate::io::UiPoll::Closed => answer(self, frame, None),
            crate::io::UiPoll::TimedOut if others_ready => answer(self, frame, Some("__busy")),
            crate::io::UiPoll::TimedOut => answer(self, frame, Some("__idle")),
        }
    }

    /// Give up one hold on `monitor`.
    fn release(&mut self, monitor: HeapRef, current: usize) -> Result<(), VmError> {
        let Some(entry) = self.threads.monitors.get_mut(&monitor) else {
            return Err(illegal_monitor_state());
        };
        if entry.owner != Some(current) {
            return Err(illegal_monitor_state());
        }
        entry.count -= 1;
        if entry.count == 0 {
            entry.owner = None;
            if entry.waiters.is_empty() {
                self.threads.monitors.remove(&monitor);
            }
        }
        Ok(())
    }

    /// `thread.start()`: a new stack whose first frame is the bundled
    /// `Thread.__entry`, which calls `run` and reports what escapes it.
    fn start_thread(&mut self, object: HeapRef, daemon: bool) -> Result<(), VmError> {
        let class_name = match self.heap.get(object) {
            Some(crate::value::HeapObject::Instance { class_name, .. }) => class_name.to_string(),
            _ => String::new(),
        };
        let classes = self.classes;
        let Some((class, method)) = resolve_virtual(classes, &class_name, "__entry", "()V") else {
            return Err(VmError::UnknownIntrinsic(String::from("Thread.__entry")));
        };
        let frame = self.make_frame(class, method, vec![JValue::Ref(Some(object))])?;
        let index = self.threads.list.len();
        let mut thread = JavaThread::new(Some(object), daemon);
        thread.frames.push(frame);
        self.threads.list.push(thread);
        self.threads.by_object.insert(object, index);
        if self.slice_end == 0 {
            self.start_slice();
        }
        Ok(())
    }

    /// `Object.wait`/`notify`/`notifyAll` — final on every object, so the
    /// call is always this one. The receiver and arguments are still on the
    /// stack.
    pub(super) fn object_monitor_call(
        &mut self,
        frame: &mut Frame<'run>,
        addr: usize,
        method: &str,
        descriptor: &str,
    ) -> Result<Flow<'run>, VmError> {
        let current = self.threads.current;
        let argc = match descriptor {
            "()V" => 0,
            "(J)V" => 1,
            _ => 2,
        };
        let receiver_at = frame
            .stack
            .len()
            .checked_sub(argc + 1)
            .ok_or(VmError::StackUnderflow)?;
        let JValue::Ref(Some(monitor)) = frame.stack[receiver_at] else {
            return Err(VmError::UncaughtException(String::from(
                "java.lang.NullPointerException",
            )));
        };
        let owns = self
            .threads
            .monitors
            .get(&monitor)
            .is_some_and(|entry| entry.owner == Some(current));
        if method != "wait" {
            Self::drop_args(frame, argc + 1)?;
            if !owns {
                return Err(illegal_monitor_state());
            }
            let entry = self
                .threads
                .monitors
                .get_mut(&monitor)
                .ok_or_else(illegal_monitor_state)?;
            let chosen: Vec<usize> = if method == "notify" {
                entry.waiters.pop_front().into_iter().collect()
            } else {
                entry.waiters.drain(..).collect()
            };
            for waiter in chosen {
                if let Park::Waiting { notified, .. } = &mut self.threads.list[waiter].park {
                    *notified = true;
                }
            }
            return Ok(Flow::Next);
        }
        if let Some(wake) = self.threads.list[current].wake.take() {
            Self::drop_args(frame, argc + 1)?;
            if wake == Wake::Interrupted {
                self.threads.list[current].interrupted = false;
                return Err(VmError::UncaughtException(String::from(
                    "java.lang.InterruptedException",
                )));
            }
            return Ok(Flow::Next);
        }
        let millis = match frame.stack.get(receiver_at + 1) {
            Some(JValue::Long(millis)) => *millis,
            _ => 0,
        };
        let nanos = match frame.stack.get(receiver_at + 2) {
            Some(JValue::Int(nanos)) => Some(*nanos),
            _ => None,
        };
        let fail = |frame: &mut Frame<'run>, error: VmError| -> Result<Flow<'run>, VmError> {
            Self::drop_args(frame, argc + 1)?;
            Err(error)
        };
        if millis < 0 {
            return fail(
                frame,
                VmError::UncaughtException(String::from(
                    "java.lang.IllegalArgumentException: timeout value is negative",
                )),
            );
        }
        if let Some(nanos) = nanos
            && !(0..=999_999).contains(&nanos)
        {
            return fail(
                frame,
                VmError::UncaughtException(String::from(
                    "java.lang.IllegalArgumentException: nanosecond timeout value out of range",
                )),
            );
        }
        // JDK 11's `wait(long, int)` adds a millisecond for ANY nanos.
        let millis = millis + i64::from(nanos.is_some_and(|nanos| nanos > 0));
        if !owns {
            return fail(frame, illegal_monitor_state());
        }
        if self.threads.list[current].interrupted {
            self.threads.list[current].interrupted = false;
            return fail(
                frame,
                VmError::UncaughtException(String::from("java.lang.InterruptedException")),
            );
        }
        if !self.may_switch() {
            let label = self.monitor_label(monitor);
            return fail(frame, self.nested_wait_refusal(&format!("wait on {label}")));
        }
        let entry = self
            .threads
            .monitors
            .get_mut(&monitor)
            .ok_or_else(illegal_monitor_state)?;
        let count = entry.count;
        entry.owner = None;
        entry.count = 0;
        entry.waiters.push_back(current);
        let until = (millis > 0).then(|| self.console.now_millis().saturating_add(millis));
        Ok(self.park(
            frame,
            addr,
            Park::Waiting {
                monitor,
                until,
                notified: false,
                count,
            },
        ))
    }
}

/// Java 11's carries no message (later JDKs added "current thread is not
/// owner").
fn illegal_monitor_state() -> VmError {
    VmError::UncaughtException(String::from("java.lang.IllegalMonitorStateException"))
}
