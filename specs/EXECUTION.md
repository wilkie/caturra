# EXECUTION — how programs run in the browser

- **Status:** accepted
- **Date:** 2026-07-02
- **Refines:** [SCOPE.md](SCOPE.md)

## Context

WASM on the browser's main thread cannot block, but Java console input
(`Scanner.nextLine()`, `System.in.read()`) is blocking by nature, and student
programs routinely run long loops. We need real blocking input and a responsive
page without making the engine itself asynchronous.

## Decision

**The Rust engine stays synchronous; the browser host runs it in a dedicated Web
Worker, and blocking stdin is implemented with `SharedArrayBuffer` + `Atomics`.**

- The engine-side contract is unchanged: `ConsoleIo::read_line()` blocks from
  Rust's point of view. In the worker, the JavaScript stdin callback performs
  `Atomics.wait` on a `SharedArrayBuffer` until the main thread supplies a line
  (or EOF), so the "block" happens in worker JS, never on the main thread.
- `@jvmjs/core` ships two API layers:
  - `JvmSession` — the direct, synchronous wrapper around the WASM module. Used
    in Node (Vitest), in the worker itself, and by advanced embedders.
  - `JvmWorkerSession` — the **primary browser API**: async, spawns the worker,
    streams stdout/stderr as events, forwards stdin requests to a host-supplied
    source (array of lines or an async function).
- Serving requirement: `SharedArrayBuffer` needs cross-origin isolation, so any
  page embedding the worker session must send
  `Cross-Origin-Opener-Policy: same-origin` and
  `Cross-Origin-Embedder-Policy: require-corp`. The playground's Vite config
  does this for dev and preview. If a page is not isolated, the worker session
  still works but stdin reads return EOF (with a console warning).
- Runaway-program protection stays in the engine: the interpreter enforces
  `VmOptions::max_instructions` per run. The worker can additionally be
  terminated from the main thread (`session.terminate()`) as a hard stop.

## Alternatives rejected

- **Main-thread v1, worker later** — cheaper now, but the async API break and
  the stdin story would land eventually anyway; decided to pay once, up front.
- **Resumable/asyncified interpreter** — avoids workers and SAB but infects the
  entire interpreter with suspension points (or doubles the WASM via asyncify);
  the heaviest option with the least educational payoff.

## Consequences

- Deployers of jvmjs-based pages must set COOP/COEP headers to get interactive
  console input; this goes in the README.
- The WASM module is loaded inside the worker only; the main thread never
  instantiates the engine (version queries etc. go through the message
  protocol).
- stdout/stderr arrive on the main thread as message events, so output ordering
  relative to program completion is preserved by the protocol (result message
  is sent after all output messages).

## Debugging

Traditional source-level debugging (added 2026-07-02), carried by the
standard class-file metadata so the pipeline stays honest:

- The compiler emits `SourceFile`, `LineNumberTable` (statement-level pc
  → line marks), and `LocalVariableTable` (named locals with live
  ranges; slots are never reused, so ranges extend to method end).
  `javap -l` accepts and prints all three — verified in the
  differential suite. Stack traces read them too:
  `at Main.main(Main.java:12)`.
- The VM pauses _before_ the first instruction of a marked line when a
  breakpoint (`file` + 1-based `line`) matches, a step goal is met
  (into / over / out, tracked by suspended-frame depth), or the host's
  interrupt flag is set (polled every 4096 instructions). Pausing calls
  the host's `on_pause` with a snapshot — frames innermost-first, each
  with class, method, file, line, and named locals — and blocks until
  it returns continue / step / terminate (plus optional breakpoint
  replacement).
- In the browser, the pause blocks the engine worker on a
  SharedArrayBuffer command channel (the stdin pattern); the main
  thread's `JvmWorkerSession.runDebug` exposes an async `onPause` so UI
  clicks resolve the command. `requestPause()` sets a shared interrupt
  flag — also the stop button for runaway loops. Requires cross-origin
  isolation, like stdin.
- Watch expressions (2026-07-03): at each pause the wasm boundary
  synthesizes `class __JvmjsWatch {{ static String __eval(<the paused
frame's locals as typed parameters>) {{ return "" + (<expr>); }} }}`,
  compiles it together with the program sources (so watches can call
  user methods and read statics — and bad watches get javac wording),
  and the VM invokes it against the live frame values with heap and
  statics shared. Isolation: fresh frame stack, ~10M instruction cap,
  debug hooks off; results ride in the pause payload. Parameter types
  come from the debug tables (`LocalVariableTypeTable` recovers
  `ArrayList<E>` from erasure), fully qualified so no imports are
  needed. `this` is not synthesizable (keyword) — instance-frame
  watches use locals and statics. A `refresh` pseudo-command
  re-evaluates a replaced watch list without resuming.
- The playground editor is CodeMirror 6 (bundled locally — no CDN)
  with Java syntax and the traditional debug UX: clicking a line
  number toggles a red breakpoint dot in a dedicated gutter, the
  paused line is highlighted and scrolled into view, and gutter edits
  made while paused ride back on the resume command's breakpoint
  replacement. Playwright drives the editor through
  `window.playground` hooks (setSource/toggleBreakpoint) rather than
  CodeMirror's contenteditable internals; one test exercises the real
  line-number click path.
