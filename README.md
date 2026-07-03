# jvmjs

A JVM for the browser: a Java compiler (`javac`) and runtime (`java`) implemented in
Rust, compiled to WebAssembly, and wrapped in TypeScript — no backend server. The
target language surface is the AP Computer Science A (CSA) subset of **Java 11**:
classes, interfaces, inheritance, arrays, `String`, `ArrayList`, wrapper types,
`Math`, console IO (`System.out.println`, `Scanner`), and `java.io.File` backed by
an in-memory virtual filesystem.

## Architecture

```
crates/                          Rust workspace (the engine)
  jvmjs-classfile/               .class file model + binary reader/writer (JVMS §4)
  jvmjs-compiler/                javac: lexer → parser → semantics → codegen
  jvmjs-vm/                      java: interpreter, virtual filesystem, console IO
  jvmjs-wasm/                    wasm-bindgen boundary (JvmSession: compile/run/vfs)
packages/
  core/                          @jvmjs/core — typed TS wrapper around the WASM
apps/
  playground/                    Vite demo app: edit, compile, and run Java in-page
e2e/                             Playwright tests driving the playground
specs/                           Project decisions and specifications (start with SCOPE.md)
```

Data flow: `@jvmjs/core` loads the WASM module and exposes `JvmSession`.
`session.compile([{ path, text }])` is `javac` — it returns structured diagnostics
(with 1-based line/column spans for editor squiggles) and retains compiled classes.
`session.run('Main', { onStdout, onStderr, readStdin })` is `java` — console IO
streams through callbacks, and `java.io.File` operations hit the session's virtual
filesystem, which the host page can seed and inspect (`writeFile`, `readTextFile`,
`listDir`, ...). Sessions are fully isolated from one another.

## Status

**Stages 1–3 of the language are live** (see `specs/LANGUAGE.md`): programs
with local variables, operators, casts, string concatenation, full control
flow, and **user-defined static methods** (parameters, returns, recursion,
overloading, cross-class calls) compile with our compiler and run on our VM
in the browser. Semantics are Java-exact and now **verified against OpenJDK
11**: a differential test suite runs identical programs through
`javac`+`java` and jvmjs and requires byte-identical stdout (int wrapping,
NaN, `Double.toString` incl. `-0.0` and `1.0E7`, overload resolution,
`StackOverflowError`, ...). Compiler errors use javac's wording ("missing
return statement", "reference to f is ambiguous", "cannot find symbol").
Implemented and tested:

- class file model, constant pool, `Code` attribute, binary read/write round-trip
- compiler: lexer (complete token surface) → recursive-descent parser →
  codegen, for the v0 subset in `specs/LANGUAGE.md`; everything else gets a
  friendly, located "not yet supported by jvmjs" diagnostic with recovery
- VM: heap (UTF-16 strings, interning), bytecode dispatch loop for the slice
  opcodes, intrinsics (`System.out`/`err`, `PrintStream.print`/`println` with
  Java-style formatting), instruction budget against infinite loops
- worker execution model (`specs/EXECUTION.md`): the engine runs in a dedicated
  Web Worker via `JvmWorkerSession` (async API, streamed output), with blocking
  stdin over SharedArrayBuffer + Atomics ready for when `Scanner` lands
- virtual filesystem + console IO plumbing across the whole stack

Next per `specs/LANGUAGE.md` staging: arrays (1D, then 2D) and `for`-each.
The class library strategy is in `specs/SCOPE.md`.

Dev note: with a JDK installed (`javac`/`java` on PATH), `cargo test`
includes the differential suite; without one those tests skip.

Deployment note: pages embedding jvmjs need `Cross-Origin-Opener-Policy:
same-origin` and `Cross-Origin-Embedder-Policy: require-corp` headers for
interactive console input (the playground's Vite config shows how).

## Prerequisites

- Rust via `rustup` (the pinned toolchain in `rust-toolchain.toml` auto-installs,
  including the `wasm32-unknown-unknown` target)
- Node 22 (`nvm use 22`) with corepack enabled (`corepack enable`); pnpm is
  version-pinned by the `packageManager` field

## Commands

```sh
pnpm install            # JS dependencies (also installs the wasm-pack binary)
pnpm build:wasm         # Rust → WASM → packages/core/src/wasm/generated/
pnpm build              # build:wasm + typecheck + bundle the playground
pnpm dev                # playground dev server at http://localhost:5173
pnpm test               # Vitest (runs the real WASM engine in Node)
pnpm test:rust          # cargo test --workspace
pnpm test:e2e           # Playwright against the built playground (needs: pnpm exec playwright install chromium)
pnpm lint               # eslint + prettier --check
pnpm lint:rust          # cargo clippy (pedantic) + cargo fmt --check
```

`pnpm build:wasm` must run before the JS tests/builds — the generated bindings in
`packages/core/src/wasm/generated/` are gitignored build output.
