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

## Runaway protection

`VmOptions::max_instructions` bounds interpreted instructions per run
(default 500M) so `while (true) {}` ends with a friendly error instead of a
frozen tab; the worker host can also hard-terminate (see
[EXECUTION.md](EXECUTION.md)).
