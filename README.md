# caturra

A JVM for the browser: a Java compiler (`javac`) and runtime (`java`) implemented in
Rust, compiled to WebAssembly, and wrapped in TypeScript — no backend server. The
target language surface is the AP Computer Science A (CSA) subset of **Java 11**:
classes, interfaces, inheritance, arrays, `String`, `ArrayList`, wrapper types,
`Math`, console IO (`System.out.println`, `Scanner`), and `java.io.File` backed by
an in-memory virtual filesystem.

## Architecture

```
crates/                          Rust workspace (the engine)
  caturra-classfile/               .class file model + binary reader/writer (JVMS §4)
  caturra-compiler/                javac: lexer → parser → semantics → codegen
  caturra-vm/                      java: interpreter, virtual filesystem, console IO
  caturra-wasm/                    wasm-bindgen boundary (JvmSession: compile/run/vfs)
packages/
  core/                          @caturra/core — typed TS wrapper around the WASM
apps/
  playground/                    Vite demo app: edit, compile, and run Java in-page
e2e/                             Playwright tests driving the playground
specs/                           Project decisions and specifications (start with SCOPE.md)
```

Data flow: `@caturra/core` loads the WASM module and exposes `JvmSession`.
`session.compile([{ path, text }])` is `javac` — it returns structured diagnostics
(with 1-based line/column spans for editor squiggles) and retains compiled classes.
`session.run('Main', { onStdout, onStderr, readStdin })` is `java` — console IO
streams through callbacks, and `java.io.File` operations hit the session's virtual
filesystem, which the host page can seed and inspect (`writeFile`, `readTextFile`,
`listDir`, ...). Sessions are fully isolated from one another.

## Status

**All eight stages are live — the full CSA surface from `specs/SCOPE.md` is implemented** (see `specs/LANGUAGE.md`): programs
with local variables, operators, casts, string concatenation, full control
flow, **user-defined static methods**, **arrays** (1D/2D, initializers,
for-each), **classes with objects**, **inheritance** (`extends`, `super`, overriding,
`interface`/`abstract`, `instanceof`, casts), and now the **class
library** — `String` methods, `Math`, `Integer`/`Double`,
`ArrayList<E>` with autoboxing and for-each, `Scanner` reading real
console input (in the browser: the SharedArrayBuffer blocking-stdin
path), and `java.io.File`/`PrintWriter` over the virtual filesystem
(seedable and inspectable from JavaScript) — compile with our compiler
and run on our VM. Semantics are
Java-exact and **verified against OpenJDK 11**: a differential test suite
runs identical programs through `javac`+`java` and caturra and requires
byte-identical stdout. Compiler errors use javac's wording, and runtime
exceptions use Java 11's messages (`ArrayIndexOutOfBoundsException: Index 5
out of bounds for length 3`). Implemented and tested:

- class file model, constant pool, `Code` attribute, binary read/write round-trip
- compiler: lexer (complete token surface) → recursive-descent parser →
  codegen, for the v0 subset in `specs/LANGUAGE.md`; everything else gets a
  friendly, located "not yet supported by caturra" diagnostic with recovery
- VM: heap (UTF-16 strings, interning), bytecode dispatch loop for the slice
  opcodes, intrinsics (`System.out`/`err`, `PrintStream.print`/`println` with
  Java-style formatting), instruction budget against infinite loops
- worker execution model (`specs/EXECUTION.md`): the engine runs in a dedicated
  Web Worker via `JvmWorkerSession` (async API, streamed output), with blocking
  stdin over SharedArrayBuffer + Atomics ready for when `Scanner` lands
- virtual filesystem + console IO plumbing across the whole stack

With the language and library surface complete, the open frontiers are
polish: an explicit-frame interpreter for deep recursion
(`specs/RUNTIME.md`), richer diagnostics, and playground UX.

Dev note: with a JDK installed (`javac`/`java` on PATH), `cargo test`
includes the differential suite; without one those tests skip.

Deployment note: pages embedding caturra need `Cross-Origin-Opener-Policy:
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
pnpm sounds:fetch       # optional: download the CSA levels' theater sounds (needs ffmpeg)
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

`pnpm sounds:fetch` is optional. The theater levels play Code.org-hosted sounds
(`birds.wav`, `retrobeat.wav`, …) that are too large to vendor (~104 MB) and that
an isolated page can't fetch cross-origin, so they are downloaded and transcoded
to MP3 (~6 MB) into the gitignored `apps/playground/public/sounds/level/`. Without
it, everything still runs — `playNote`, generated `playSound(double[])`, and the
bundled `beatbox.wav` work, and unknown sound names are silent. See
[specs/EXECUTION.md](specs/EXECUTION.md).
