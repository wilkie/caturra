# RUNTIME — VM representation choices

- **Status:** accepted
- **Date:** 2026-07-02
- **Refines:** [SCOPE.md](SCOPE.md)

## Heap and garbage collection

Index-based heap: objects live in a `Vec`, references are `u32` indices
(`JValue::Ref`), `null` is represented explicitly. **No garbage collector in
v1** — sessions are short-lived and exam-scale programs allocate modestly; the
heap is freed when the run ends. The design keeps an upgrade path to a simple
mark-sweep collector (all references flow through `JValue`, so roots are
enumerable: frames' locals + operand stacks + statics + interned strings).
Revisit when real programs hit memory limits, not before.

## Strings

Java string semantics are UTF-16: `length()`, `charAt()`, `substring()` count
UTF-16 code units. VM strings store `Vec<u16>` so those methods are exact, and
convert at the edges (console output, VFS, the JS boundary). String literals are
interned per session, as the JLS requires for `==` behavior on literals.

## Intrinsics (native classes)

A registry keyed by `(class binary name, method name, descriptor)` maps to Rust
functions; intrinsic static fields (e.g. `java/lang/System.out`) resolve the
same way. Class resolution order, per SCOPE.md: **intrinsics → baked-in
classlib → user classes**. Intrinsic objects (e.g. the `PrintStream` behind
`System.out`) are ordinary heap objects with intrinsic-backed methods, so user
code can pass them around like any reference.

## Bundled library sources (Code.org `org.code.neighborhood`)

Some curriculum libraries are provided as **clean-room Java source compiled
alongside student code**, rather than as native intrinsics. When a source
imports `org.code.neighborhood`, `compile()` auto-injects
`compiler/src/stdlib/neighborhood.java` as an extra unit; its classes then
resolve like any user class. This keeps the library behaviourally faithful to
the real Code.org implementation and diff-testable, and reuses the whole
language surface (enums, 2-D arrays, `ArrayList`, exceptions) the compiler
already supports. Support types are prefixed `__Nbhd`; only `Painter` is public.

The **grid** (`serialized_maze` from a level's `properties.json`) is injected by
the harness as a `grid.txt` file in the VFS (rows of space-separated
`tileType,paintCount` cells); the bundled `World` reads it with `Scanner`, so no
JSON parsing lives in the VM. Missing `grid.txt` falls back to an empty 10×10.

Phase 1 is a **headless simulation**: painter/grid state and console output are
byte-identical to the real library (601/614 real neighborhood solutions run to
completion; the rest need interactive stdin or exceed the recursion limit).

**Phase 2 (animation stream)** is wired for neighborhood: each Painter action
also emits the exact javabuilder `ClientMessage` — `{"type":"NEIGHBORHOOD",
"value":"MOVE","detail":{"id","direction",…}}` — that the real
`apps/src/miniApps/neighborhood/Neighborhood.ts` renderer consumes. Messages are
appended write-through to the VFS file `neighborhood.jsonl` via a single
kept-open `PrintWriter` (append is O(1), so a 1500-action level costs ~0.3s, not
the O(n²) of rewriting). The frontend replays that stream over the grid. `apps/playground` closes this
loop: on a neighborhood run it seeds `grid.txt`, runs, reads back
`neighborhood.jsonl` via the worker's `readFile`, and animates the painter on a
canvas (`apps/playground/src/neighborhood.ts`) — a self-contained renderer that
consumes the same protocol as the real `Neighborhood.ts`. Theater differs — the
real system renders it to a GIF server-side (`GifWriter`), so its Phase 2 is a
browser Canvas port, not a message stream: `apps/playground/src/theater.ts`
replays the `theater.log` draw commands onto a 400×400 canvas, matching
`GraphicsHelper`'s coordinate/order semantics (top-left rects/ellipses,
center-based polygons, baseline text, fill-then-stroke) and holding a frame at
each `pause`. Pixel-exact parity with the server GIF is out of reach (AWT vs
Canvas font metrics / image decoding), but shapes, colours, text, and animation
render faithfully; Phase-1 blank images draw as placeholder boxes.

The same pattern covers **`org.code.theater` + `org.code.media`**
(`compiler/src/stdlib/theater.java`, injected on either import). `Scene` records
each draw call (rectangle/ellipse/line/text/image/shape, colours, notes) into a
command log; `Theater.playScenes` writes the log to the VFS file `theater.log`.
The canvas is 400×400 and `Color` values (27 named + hex) match the real
library; `Image`/`Pixel` back real pixel arrays so image-processing exercises
compute correctly. 448/617 real theater solutions run headless to completion
(94% of those that terminate without needing an unseeded data-file asset or
looping forever on animation). Actual pixel/GIF rendering and audio decoding are
Phase 2; `SoundLoader.read` returns silence and file-loaded `Image`s are blank.

## Exceptions

Java exceptions are heap objects; the interpreter carries them as the `Err`
variant of Rust `Result` while unwinding, consulting each frame's exception
table on the way. An uncaught exception prints a `java`-style stack trace to
stderr and ends the run with a nonzero status.

## Class initialization

`<clinit>` runs lazily on first active use of a class (JVMS §5.5), tracked with
an initialization state per class to handle recursive initialization.

## Bytecode verification

Our VM is the only consumer of our compiler's output, so there is no separate
verification pass in v1; the interpreter validates structurally as it executes
(operand stack underflow, bad constant-pool indices, and unknown opcodes are
`VmError`s, not panics). `StackMapTable` attributes are neither emitted nor
read. Revisit if we ever want real JVMs to load our class files.

## Method calls and the call stack

The Java call stack is an explicit heap-allocated `Vec<Frame>` (adopted
2026-07-02, replacing the original host-stack recursion). Each `Frame` is
plain data — owner class, method name, pc, locals, operand stack — and
the dispatch loop is one flat loop: call opcodes push a frame, return
opcodes pop one. `<clinit>` bodies run as pseudo-caller frames pushed
beneath the triggering instruction, which rewinds and re-executes after
initialization (JVMS §5.5 ordering, superclass first).

Consequences and intent:

- `VmOptions::max_call_depth` (default 4096, roughly Java-like) is a pure
  semantics knob raising `StackOverflowError`; the host (Rust/WASM) stack
  stays O(1) at any Java depth, so debug-build frame sizes and the ~1MB
  WASM stack no longer constrain the limit.
- Parsed `Code` attributes are cached per method (`Rc`), so hot recursive
  calls no longer re-parse bytecode on every invocation.
- Uncaught exceptions print `java`-style `\tat Class.method(Class.java)`
  trace lines (no line numbers yet — we don't emit LineNumberTable).
- Frames being inspectable data is the intended foundation for a step
  debugger in the playground: pause the loop, walk `Vec<Frame>`, show
  locals per frame.

## Runaway protection

`VmOptions::max_instructions` bounds interpreted instructions per run
(default 500M) so `while (true) {}` ends with a friendly error instead of a
frozen tab; the worker host can also hard-terminate (see
[EXECUTION.md](EXECUTION.md)).
