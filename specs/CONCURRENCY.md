# CONCURRENCY — Java threads on a single-threaded engine

- **Status:** accepted — phases 0 and 1 implemented 2026-09-25, phases 2 and 3 2026-09-26
- **Date:** 2026-09-25
- **Refines:** [EXECUTION.md](EXECUTION.md), [RUNTIME.md](RUNTIME.md)
- **Amends:** the "Threads" non-goal in [SCOPE.md](SCOPE.md)

## Context

caturra refuses `java.lang.Thread` outright, and `synchronized` is accepted
because on one thread a monitor is never contended. That was the right call
while nothing could be concurrent. It is now the one place the Code.org corpus
and javac disagree (a student project that paints a neighborhood from seven
threads and `join`s them), it is the last `java.lang` class a course program
plausibly names, and `Thread.sleep` — which students reach for the moment they
want an animation to pace itself — cannot be honoured without it.

The question this document answers is how to run Java threads **accurately**
inside the engine as it stands: a synchronous Rust interpreter compiled to
WebAssembly, running on the single thread of one Web Worker, with every
blocking operation (stdin, Swing events, the debugger's pause) implemented as
worker-side JavaScript parking on `Atomics.wait` (see EXECUTION.md).

Four facts about the engine make this far more tractable than it first
sounds, and one fact makes it bounded:

1. **Frames are data, not host recursion.** The Java call stack is
   `Vec<Frame>` on the heap (RUNTIME.md, "Method calls and the call stack").
   A thread's entire execution state is a vector of plain structs; swapping
   one vector for another is a thread switch.
2. **There is already a safepoint.** The dispatch loop has exactly one place
   where the world is consistent — between two instructions at the base
   nesting level, where the collector runs. A scheduler switches there and
   nowhere else, for the same reason.
3. **There is already a rewind-and-retry idiom.** A `<clinit>` triggered by an
   instruction runs as frames pushed beneath it, and the instruction
   re-executes afterwards (`Frame::pc_reexecutes`). A blocking instruction
   parks the same way: rewind, run something else, re-execute when woken.
4. **The host already knows how to block.** The worker parks on
   `Atomics.wait`; a timed wait is the same call with a timeout.

The bounded part:

5. **Native code re-enters the interpreter on the host stack.** Twenty-two
   sites call `run_nested` — a comparator inside a sort, the lambda a
   `forEach` or a stream op runs, a `toString` while a container renders, a
   map's `compute` function, a Swing event handler. While a thread is inside
   one of these, its continuation lives on the Rust/WASM stack, and a WASM
   stack cannot be set aside and resumed. **A thread inside native re-entry
   cannot be switched away from.** Everything below is designed around that
   one constraint rather than pretending it away.

## Decision

**Green threads (M:1), scheduled by the interpreter at its safepoint, with
time-slice preemption by instruction count. A blocking operation parks the
thread; the host really waits only when every thread is parked. `synchronized`
becomes a real monitor. `Thread.sleep` really sleeps.**

- One WASM thread, one worker, no parallelism. Java threads are interleaved
  by the interpreter, never run at the same time. This is exactly what a JVM
  looks like on a one-core machine, which is the mental model the course
  teaches.
- Every switch happens between two bytecode instructions of a thread that is
  at its base nesting level. Nothing about the collector's safety argument
  changes: parked threads' frames are data the collector can walk.
- The engine's contract with the host stays synchronous (EXECUTION.md's
  decision stands). One new host hook: a bounded wait.

## Non-goals

- **Parallel execution.** Running Java threads on several workers would need
  the heap, the interpreter and every intrinsic to be shared-memory safe.
  Nothing a course program does needs the speed, and the semantics students
  need are interleaving, not parallelism.
- **Memory-model effects.** With one execution thread every write is visible
  to every thread the instant it happens: the program runs sequentially
  consistent, which is a _legal_ JVM execution. `volatile` is accepted and
  changes nothing. A program cannot observe a stale read here; no correct
  program depends on one.
- **Priorities.** `setPriority` is accepted and ignored; the JDK documents that
  priorities are a hint an implementation may ignore.
- **`Thread.stop/suspend/resume`, `ThreadGroup`, `ThreadLocal`** — the first
  three are deprecated-for-removal, the last is a value store a course program
  never needs. Refused by name, as now.
- **Uncaught-exception handlers beyond the default** in phase 1.

## Design

### The thread as data

```rust
struct JavaThread<'run> {
    id: u32,                       // Thread.getId(); main is 1
    name: String,                  // "main", "Thread-0", or setName's
    object: Option<HeapRef>,       // the java.lang.Thread instance (None for main until asked)
    frames: Vec<Frame<'run>>,      // the whole Java stack; empty until start()
    location: Option<CurrentLocation<'run>>,
    state: ThreadState,
    interrupted: bool,
    daemon: bool,
    quantum_left: u32,
}

enum ThreadState {
    New,
    Runnable,
    Sleeping { until: i64 },
    Joining { target: u32, until: Option<i64> },
    Blocked { monitor: HeapRef },                        // monitorenter
    Waiting { monitor: HeapRef, until: Option<i64>, notified: bool, reacquire: u32 },
    Terminated,
}
```

The interpreter keeps `threads: Vec<JavaThread>` and `current: usize`. The
fields it has today for one run — `frames`, `current_location` — become the
current thread's, swapped in and out at a switch. `suspended_runs` and
`nested_frame_base` (the native re-entry stack) stay on the interpreter,
because they describe the _host_ stack, which only the current thread can be
on; a switch is only permitted when `suspended_runs` is empty, so a parked
thread never has any.

The instruction budget stays global (it bounds the run, not a thread). The
call-depth limit is per thread. `MAX_NESTED_RUNS` needs no change: only the
running thread can nest.

### Where a switch happens, and when

The safepoint in `run_loop` gains one check, after the collector's:

```
if self.suspended_runs.is_empty() && self.should_switch() {
    frame.pc = pc; self.park_current(frame, reason);
    frame = self.resume_next();     // the top frame of whichever thread runs next
    continue 'frames;               // re-establishes class/code/pc for it
}
```

`should_switch` is true when the current thread's quantum is spent and
another thread is runnable, when the current thread just parked, or when it
called `Thread.yield()`. The outer `'frames` loop already re-establishes
per-frame context after every call and return, so a switch costs nothing it
does not already do.

Scheduling is **round-robin with a fixed instruction quantum**, deterministic
for a given program and input. Two consequences, both intended:

- **Reproducible.** The same program prints the same interleaving every run,
  so a pin can compare it, and a student can reason about what they saw.
- **Races are still visible.** The textbook demonstration — two threads each
  incrementing a shared `int` a million times, ending short of two million —
  works: `count++` is four instructions (`getstatic`, `iconst_1`, `iadd`,
  `putstatic`), and with a quantum of a few thousand instructions a slice
  boundary lands inside it hundreds of times per run. The quantum is chosen
  so that it does; it is a tuning constant, recorded with its measurement.

A JDK's interleaving is not reproducible, and no pin should pretend it is:
the pins below compare only what a JDK's output _determines_ (see Testing). A
seeded jitter on the quantum is a possible later option for a "show me a
different interleaving" button; it is not part of this design.

`start()` does not switch — a JDK keeps running the caller, and so does this.
The new thread runs when the caller's quantum ends or it parks. That is why
`main` usually prints before the thread it just started, here as there.

### Parking: the rewind-and-retry idiom

A blocking intrinsic never blocks the host itself. It returns a new control
signal, `Park(ThreadState)`; the dispatch loop sets the thread's state,
rewinds `pc` to the invoking instruction, marks the frame `pc_reexecutes`, and
switches. When the thread is resumed, the same instruction runs again and the
intrinsic is asked again — this time with the thread's state to consult:

- **`Thread.sleep(ms)`**: first call records `until = now + ms` and parks.
  On re-execution, `now >= until` returns normally (and clears the state);
  otherwise it parks again. An interrupt wakes it early: the re-executed call
  sees `interrupted`, clears it, and throws `InterruptedException` — the JDK's
  order.
- **`join()` / `join(ms)`**: parks on `Joining { target }` until the target is
  `Terminated` (or the deadline passes). Joining a thread that was never
  started returns at once, as a JDK's does.
- **`monitorenter`** (a `synchronized` block or method): if the monitor is
  free or owned by this thread, take it and continue; otherwise park on
  `Blocked { monitor }`. Re-execution retries the acquisition.
- **`Object.wait()` / `wait(ms)`**: requires ownership (else
  `IllegalMonitorStateException` — with a null message on Java 11; the
  "current thread is not owner" wording is a later JDK's). Releases the
  monitor fully, remembering the recursion count, and parks on `Waiting`.
  `notify` marks one waiter `notified` (FIFO — the JDK leaves the choice
  unspecified; this is the one HotSpot makes in practice, and it is
  recorded); `notifyAll` marks all. A notified (or timed-out, or interrupted)
  waiter is not runnable until it has re-acquired the monitor, at the
  remembered count — the second phase of the same state.

Each of these is one small state machine attached to the thread, and the
dispatch loop knows nothing about any of them beyond "park" and "the thread is
runnable again". The scheduler decides runnability by inspecting states:
`Sleeping` past its deadline, `Joining` a terminated target, `Blocked` on a
free monitor, `Waiting` that is notified/timed out/interrupted and whose
monitor is free.

### When nothing can run

If every live thread is parked, the scheduler asks: is any of them waiting on
**time**? If so, the host is asked to wait until the earliest deadline — this
is the only moment the host sleeps, and it is how `Thread.sleep` in a
one-thread program becomes a real pause. If no thread is waiting on time, no
thread can ever wake: every one is joining, blocked or waiting on something
only another parked thread could release. A JDK hangs here. caturra will not
hang a browser tab: it ends the run with

```
caturra: every thread is blocked and none is waiting on time
  "main"     joining "Thread-0"
  "Thread-0" waiting on Object@1a2b (holds none)
```

— a thread dump, which is what a person debugging the hang on a JDK would
have to produce by hand. This is a divergence from a JDK of the _stricter_
kind (a program that hangs there is refused here), and it is recorded in
LANGUAGE.md's divergence list with its pin.

### The host contract: one bounded wait

`ConsoleIo` gains

```rust
/// Wait up to `millis` for anything at all to happen, or for the time to
/// pass. Returns how many milliseconds actually elapsed.
fn wait_millis(&mut self, millis: u32) -> u32;
```

- **Worker (browser):** `Atomics.wait` with a timeout on a dedicated
  `SharedArrayBuffer` when the page is cross-origin isolated; otherwise a
  `performance.now()` spin in the worker (which burns a core but blocks no
  page — the worker has nothing else to do). Waits are made in slices (100 ms)
  so the debugger's interrupt flag is polled during a long sleep; Stop is
  `worker.terminate()` and needs nothing.
- **Tests and the CLI:** a **virtual clock**. `now_millis` is a counter that a
  `wait_millis` advances instantly. A sleep-heavy program runs in no time and
  every timing pin is deterministic; `nanoTime` advances with it. This is the
  same arrangement the tests already have for `Math.random` (a fixed seed).
- **Swing** kept blocking the host in phase 1: a background thread could not
  run while the UI waited for a click. Phase 3 gave the wait a timeout
  (`ui_poll_event`), so the event pump never holds the host while another
  thread could run. `ui_dialog` (a `JOptionPane`) still blocks: nothing else
  runs while a dialog is up.

`System.currentTimeMillis` and `nanoTime` keep reading the host clock, so a
program that times its own sleep sees the pause it asked for.

### The nested-run boundary

The one constraint, stated as behaviour a program can observe. A thread is
"inside native re-entry" while it runs code that native code invoked: a
comparator, a `forEach`/stream/`removeIf`/`compute*` callback, a `toString`
called during rendering, a Swing event handler.

- **`sleep` there really sleeps** — the host waits — but no other Java thread
  runs during it. Time passes correctly; interleaving does not.
- **`yield` there is a no-op.**
- **`join`, `wait`, and a contended `monitorenter` there** cannot be
  honoured: the thread that would satisfy them cannot run. Rather than hang,
  the run ends with

  ```
  caturra: "Thread-0" cannot wait for "main" inside a callback that library
  code invoked (a comparator, a forEach body, an event handler); move the
  wait outside the callback
  ```

  This is honest in the way every caturra refusal is: it names the exact
  limit, and the program can be rewritten around it. It is expected to be
  rare — a `join` inside a `forEach` lambda is unusual code — and a Swing
  handler that _sleeps_ (the common case, to animate) is unaffected.

The boundary is not permanent. Two routes would remove it, both out of scope
here: making the twenty-two re-entry sites resumable (each becomes a state
machine driven from the dispatch loop, the way `<clinit>` already is), or
WASM stack switching (JSPI), which would let a nested run be suspended as a
unit. Either is its own spec; this one makes the limit explicit so that spec
has a measured starting point.

### Monitors

A `synchronized` block compiles to real `monitorenter`/`monitorexit` with the
exception-table exit javac emits, and a `synchronized` method's
`ACC_SYNCHRONIZED` flag is honoured at invoke and return (including the
exceptional return). The interpreter keeps a monitor table keyed by object:

```rust
struct Monitor { owner: u32, count: u32, waiters: Vec<u32> }
```

An entry exists only while the object is locked or has waiters; a parked
thread's monitor is a collector root, and the table is retained with the
heap's other side tables. `Collections.synchronized*` wrappers, which today
alias the collection they wrap, lock their own monitor on every call — they
are already separate heap objects, so this is a `monitorenter` around the
delegation and nothing more.

`IllegalMonitorStateException` is thrown for `wait`/`notify` without
ownership — with a null message, which is what Java 11 gives (measured;
later JDKs added the words).

### Program lifetime

- `main` returning does not end the run: the scheduler keeps going until no
  **non-daemon** thread is live. `System.exit` ends everything at once, as
  now.
- An exception escaping a thread's `run` prints, to standard error,
  `Exception in thread "Thread-0" java.lang.IllegalStateException: …` and the
  trace, the thread ends, and the others continue — the default uncaught
  handler. A JDK's trace ends in `at java.base/java.lang.Thread.run(Thread.
java:829)` beneath the program's own frames; the thread's bottom frame here
  is that call, so the line is a real frame's, not a decoration. The process
  still exits 0 (measured), and a daemon thread still sleeping does not keep
  it alive. `main`'s uncaught exception keeps its current shape; whether the
  run's status is "uncaught" or "completed" when other threads outlive `main`
  is decided by measurement against a JDK, not assumed.
- Thread names are `Thread-N`, numbered from zero in creation order, as a
  JDK's; `toString` is `Thread[Thread-0,5,main]`; `getId` numbers from a
  measured base (23 on the JDK used here, after the JVM's own). `start()`
  twice is `IllegalThreadStateException` with a null message. `sleep(-1)` is
  `IllegalArgumentException: timeout value is negative`.
  Every message is captured from a JDK first, the way every other message
  here was.

### Garbage collection and budgets

Roots are every thread's frames plus the current thread's `temp_roots` and
suspended runs, exactly as today with more frame vectors to walk. The
safepoint condition ("base nesting level of the running thread") is unchanged
and still sufficient: a parked thread has no native re-entry by construction.
The heap budget, the instruction budget and the debugger's interrupt flag are
all global and all checked where they are now.

### The debugger

The snapshot gains the list of threads with their states, and marks the
current one; frames shown are the current thread's. A step advances the
current thread only, unless it parks — then the others run until it resumes,
which is what "step over a `join`" has to mean. A breakpoint is hit by
whichever thread reaches it, and the snapshot says which.

_Built (2026-09-26):_ `DebugSnapshot::threads` — every live thread, the
paused one first, each with its name, `getState()` name, daemon flag and
stack (the others' as the scheduler saved them). The playground's paused view
lists them under the call stack when a program has more than one, leaving out
the bundled library's own frames. Pinned by
`debugger_lists_every_thread_at_a_pause` and the browser test "a pause lists
every thread with its state and where it stands".

### Library surface, by phase

**Phase 0 — sleep and identity on one thread.** `Thread.sleep(ms)`,
`sleep(ms, nanos)`, `currentThread()`, `getName`/`setName`, `getId`,
`isInterrupted`/`interrupted`/`interrupt` (on `main`), `yield`, `Thread` as a
type. `sleep` waits for real through `wait_millis`. No scheduler yet; the
refusal for `start()` names the phase. _Small: a day._

_Done (2026-09-25)._ As built: `Thread` is bundled Java
(`stdlib/thread.java`) with `Thread.State` kept at the top level as
`__ThreadState` (a nested class would be hoisted as `State` and collide with
programs' own); `sleep` reaches the host through the reserved
`__System.__sleep`, in slices of at most 100 ms; the WASM console parks on
`Atomics.wait` on a private shared cell, spinning on the clock where that is
unavailable; `BufferedConsole` keeps the virtual clock. Constructing a thread,
`run()`, priorities and daemon flags are in too, since none needs a second
thread. Not yet: the debugger's pause is not polled during a long sleep (the
slices are there for it).

**Phase 1 — threads.** `new Thread(Runnable)`, `(Runnable, String)`,
`(String)`, a subclass overriding `run`; `start`, `run`, `join`, `join(ms)`,
`isAlive`, `getState`, `setDaemon`/`isDaemon`, `setPriority`/`getPriority`
(ignored), `interrupt` waking `sleep`/`join`/`wait`; `Object.wait/notify/
notifyAll`; real monitors for `synchronized`; the uncaught handler's output;
main-exits-last; the blocked-everywhere report; the nested-run refusal; the
debugger's thread list; the virtual clock in tests. _The unit of work: the
scheduler and parking are perhaps 800 lines, the monitors 300, the Thread
class (bundled Java plus intrinsics) 400, the host hook 100, and the pins as
many again. The risk is not size but the number of places that must agree —
the same shape every unit in LANGUAGE.md has had._

_Done (2026-09-25)._ As built, and where it differs from the design above:

- The scheduler is `crates/caturra-vm/src/interpreter/threads.rs`. The
  quantum is 10,000 instructions: a 100,000-increment two-thread race loses
  updates, and a thread of a few `println`s finishes inside its first slice.
  The slice is one compare at the safepoint (`remaining_instructions <
slice_end`), zero while only one thread is alive.
- Parking is the rewind idiom for EVERY blocking call, not only
  `monitorenter`: the call leaves its arguments on the stack, parks, and on
  re-execution reads `wake` (how the park ended). The scheduler settles
  runnability — and re-takes a waiter's monitor at its count — when it
  chooses the thread, so a woken thread never runs an instruction it may not.
- `synchronized` is desugared by the parser (`__monitorEnter`, a `try`, and
  `__monitorExit` in its `finally`), not emitted as `monitorenter`/
  `monitorexit`, and a `synchronized` method is its body wrapped the same way:
  codegen's `finally` machinery already handles every abrupt exit. No
  `ACC_SYNCHRONIZED` flag is written.
- The bundled `Thread` keeps no run state; the scheduler does, keyed by the
  `Thread` object. A thread's first frame is the bundled `Thread.__entry`,
  which calls `run` and prints the default handler's banner for what escapes —
  so the handler is Java, as a JDK's is. The trace shows
  `java.base/java.lang.Thread.run(Thread.java:829)`, the one library frame a
  trace shows here.
- The uncaught handlers themselves (`setUncaughtExceptionHandler` and the
  default) stay refused by name; the default behaviour is what runs.
- The debugger's thread list came with phase 3 (see "The debugger").

**Phase 2 — `java.util.concurrent`, the course-sized part.** `Executors.
newFixedThreadPool/newSingleThreadExecutor/newCachedThreadPool`,
`ExecutorService.submit/execute/shutdown/awaitTermination`, `Future.get`,
`Callable`, `CountDownLatch`, `AtomicInteger/AtomicLong/AtomicBoolean`,
`ConcurrentHashMap` (a `HashMap` under a monitor), `TimeUnit`,
`ReentrantLock`. Written as **bundled Java** on top of `Thread`,
`synchronized` and `wait/notify`, which is how a JDK writes them too — so
they are mostly source, and each is measured against a JDK as it lands.

_Done (2026-09-26)._ As built: `stdlib/concurrent.java`, injected when a
source names the package. The executors, futures, `TimeUnit`, `CountDownLatch`,
`Semaphore`, `ReentrantLock`/`Condition` and the four single-value atomics.
`ConcurrentHashMap` came after, as a flag on the native map with its own
table model (`crates/caturra-vm/src/chm.rs`) and weakly consistent cursors —
caturra's collections are native, not Java a bundled class could extend. The
other concurrent collections stay refused by name, as do `CyclicBarrier`,
`CompletableFuture`, the
scheduled and fork/join executors. A pool the program never shuts down ends in
the everything-parked report, with a line naming `shutdown()`.

**Phase 3 — Swing under threads.** The event pump as a parkable wait, so a
worker thread runs while the window waits; `SwingUtilities.invokeLater`
and `javax.swing.Timer` as threads that post to the pump. This is what a
student's animated Swing program actually is.

_Done (2026-09-26)._ As built, all of it bundled Java over phase 1's
primitives except the one wait:

- **The event-dispatch thread** is a bundled `Thread` named
  `AWT-EventQueue-0` (`__EventQueue` in `stdlib/swing.java`, which now brings
  `Thread` with it). It starts with the first thing that needs it — a shown
  interactive window, `invokeLater`/`invokeAndWait`, a timer's first tick — and
  runs posted tasks in order, then the window's next event. `setVisible(true)`
  RETURNS, as a JDK's does: before this, it entered the event loop on the
  calling thread, so code after it ran only once the window closed.
  `EXIT_ON_CLOSE` is `System.exit(0)`, ending every thread; `DISPOSE_ON_CLOSE`
  takes the window away. With no window and nothing posted for a second the
  thread ends, as AWT's auto-shutdown does — which is how a Swing program ends.
- **The wait for the window** is `__System.__uiWait(tree, timeout)`, a
  scheduler call over the host's `ui_poll_event(tree, timeout)`. When another
  thread could run it only LOOKS (a zero timeout) and answers `__busy`; the
  dispatch thread then waits 20 ms on its own queue — a posted task wakes it
  at once — and looks again, re-rendering whatever the others changed. When
  every other thread is parked the host waits until the window says
  something or the first of their timed waits runs out. Nothing here parks
  natively: the dispatch thread's own `Object.wait` does.
- **`invokeLater`/`invokeAndWait`** (on `SwingUtilities` and
  `java.awt.EventQueue`) post to that thread. `invokeAndWait` declares the
  JDK's checked exceptions, wraps what the task threw in an
  `InvocationTargetException`, and is an `Error` from the dispatch thread
  itself. `isEventDispatchThread()` is true only there. What escapes a task or
  a listener prints the uncaught banner and the thread carries on (its trace:
  see "The dispatch thread's traces" below).
- **`javax.swing.Timer`** has a daemon `TimerQueue` thread, as a JDK's does,
  that posts one tick at a time to the dispatch thread (coalescing), so the
  listeners run there, last-added first. A daemon cannot keep a program alive:
  a timer whose first tick comes after `main` returns never fires, on a JDK as
  here. Timers are no longer the host's to schedule (the tree's `timers` list
  is gone).
- **The host.** The worker's `awaitUiEvent(tree, timeoutMs)` answers the
  event, `null` for an ended session, or `undefined` when the time ran out;
  it posts the tree only when it changed, and an event that arrives while the
  engine is busy waits in the channel for the next look
  (`pollLineBlocking`). The page settles a superseded render with
  `undefined`.
- **Dialogs** (a follow-up the same day). A `JOptionPane` waits like the
  window — `System.__uiDialog` is a scheduler call over the host's
  `ui_poll_dialog(kind, message, timeout)` — so other threads run while it is
  up. The dialog is shown once and asked again after each timed-out wait; it
  has its own channel (`dialogBuffer`), since a dialog on one thread and the
  window's wait can both be outstanding; and `JOptionPane` shows one dialog
  at a time (a second thread's waits on a lock), so an answer always belongs
  to the dialog that is up.
- **The dispatch thread's traces** are a JDK's to the line — for a posted
  task, a timer's tick, and a listener a window event runs. The bundled runtime
  has a JDK's shape (`__EventDispatchThread extends Thread`, posted work
  wrapped in an `__InvocationEvent`, every listener called from its own
  `__fire…` method, and the event's entry point — a click's `__onEvent`, or
  `doClick` — above that), and the interpreter's `library_frame_lines` writes
  each bundled frame as the JDK frames it stands for. A window event's frames
  need a display to measure, so they were: `scripts/compat/awt-traces/`
  captures them with a GUI build of the same JDK (Ubuntu's `openjdk-11-jre`,
  fetched with `apt download`, beside the installed headless one) under Xvfb,
  `java.awt.Robot` making the clicks and keystrokes, and `generate.py` turns
  the 31 captured scenarios into `interpreter/awt_frames.rs`, splitting each
  chain into the listener's call and how the event arrived. Measuring them
  showed two things that were WRONG, not just unshown: a click is three
  events (pressed, released, clicked) and a key two (pressed, typed), each
  dispatched on its own, so one listener's exception must not skip the next;
  and a text component compared an edit against `""` rather than the text it
  had, so shortening a field built with text reported an insert.

- **`javax.swing.SwingWorker`** (a follow-up, 2026-09-26) is bundled Java too
  (`stdlib/swingworker.java`), over the concurrency bundle as a JDK's is over
  `java.util.concurrent`: a `FutureTask` over `doInBackground`, run by a pool
  of up to ten daemon threads named `SwingWorker-pool-N-thread-M`; published
  chunks reach `process` on the dispatch thread, coalesced, and so does
  `done`; the "state" and "progress" properties go to their listeners on the
  dispatch thread through `java.beans.PropertyChangeSupport` (now modelled:
  `stdlib/beans.java`). `java.lang.Void` came with it, for
  `SwingWorker<String, Void>`. Pinned by `a_swing_worker` and the browser test
  "a SwingWorker reports progress and its result on the event thread".

Pinned by `the_event_dispatch_thread`,
`a_timer_alone_does_not_keep_a_program_alive`,
`the_dispatch_thread_keeps_a_timer_going`, `an_exception_on_the_dispatch_thread`
(whole traces; against a headless JDK — and `a_listener_run_by_do_click` — the harness now passes `-Djava.awt.headless=true`),
the scripted-window tests `swing_a_worker_thread_runs_while_the_window_waits`
and `swing_main_animates_a_label_while_the_window_is_up`, and the browser test
"a worker thread updates the window while it stays responsive" (the
"Background worker" demo).

## Alternatives rejected

- **Run `start()` synchronously to completion.** Correct for start-then-join
  and nothing else: a producer/consumer deadlocks, a `sleep` in the thread
  stalls `main`, and every interleaving lesson is silently wrong. Cheap, but
  it teaches the wrong thing, and this project has refused every cheaper
  wrong answer so far.
- **Real parallelism: one worker per Java thread over shared WASM memory.**
  The heap is a Rust `Vec` in linear memory owned by one interpreter; the
  intrinsics, the collector, the side tables and the string pool are all
  single-owner. Making them shared is a rewrite of the engine for a speedup no
  course program needs.
- **Preemption by wall clock.** Non-deterministic: every pin becomes flaky and
  every student sees a different run. Instruction quanta give the same races
  reproducibly.
- **Asyncify / JSPI to suspend nested runs now.** The right long-term answer
  to the one hard limit, but it doubles the WASM (asyncify) or depends on a
  browser feature whose availability must be checked at the time; deferred to
  its own spec with the limit measured first.

## Consequences

- SCOPE.md's "Threads" non-goal becomes a goal with a spec; the
  compatibility page's `Threads` entry moves from unsupported to supported as
  each phase lands.
- Two new documented divergences, both stricter than a JDK: the
  everything-blocked report where a JDK hangs, and the nested-run refusal.
  Each gets a `stricter_than_javac!` pin and a LANGUAGE.md bullet.
- `synchronized` stops being free. A synchronized method's cost is one table
  lookup at entry and exit; single-threaded programs never contend, so nothing
  parks.
- Deployment is unchanged: a non-isolated page still runs threads, with
  `sleep` as a worker-side spin instead of an `Atomics.wait`.

## Testing

Differential pins compare what a JDK's output _determines_, never the
interleaving it happened to produce:

- **Structure:** `start`/`join` ordering, results computed by joined threads,
  a synchronized counter's final value, `wait/notify` hand-offs whose sequence
  is forced (a bounded buffer of size one), a `CountDownLatch` release order.
- **Time:** elapsed bounds — `sleep(100)` takes at least 100 ms by
  `nanoTime`, on both engines (real time on a JDK, virtual here).
- **Races:** property pins — the unsynchronized counter ends _at or below_
  the total on both, and caturra's own result is stable run to run (its
  determinism is a promise worth pinning).
- **Refusals:** the two stricter divergences, with their wording.
- **The corpus:** the student neighborhood project compiles and paints, and
  the `starts` sweep's last disagreement with javac closes.
- **Every message** (`IllegalMonitorStateException`, `sleep(-1)`, a second
  `start()`, the uncaught-handler line) is captured from a JDK before it is
  written, as every message in this engine has been.

The existing gates all apply unchanged; a single-threaded program must
produce byte-identical output before and after every phase, which the corpus
sweeps check on every commit.
