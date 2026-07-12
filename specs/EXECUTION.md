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
- `@caturra/core` ships a synchronous in-process wrapper plus two async drivers
  behind a shared `JvmSessionApi`, selected at runtime by `openJvmSession`:
  - `JvmSession` — the direct, synchronous wrapper around the WASM module. Used
    in Node (Vitest), in the worker itself, and by advanced embedders.
  - `JvmWorkerSession` — the **default browser API**: async, spawns a same-origin
    worker, streams stdout/stderr as events, forwards stdin requests to a
    host-supplied source (array of lines or an async function).
  - `RemoteJvmSession` — runs the engine in a cross-origin sandbox iframe so the
    editor origin's session data stays isolated; see Cross-origin sandbox mode
    below. Same `JvmSessionApi`, so callers are identical either way.
- Serving requirement: `SharedArrayBuffer` needs cross-origin isolation, so any
  page embedding the worker session must send
  `Cross-Origin-Opener-Policy: same-origin` and
  `Cross-Origin-Embedder-Policy: require-corp`. The playground's Vite config
  does this for dev and preview. If a page is not isolated, the worker session
  still works but stdin reads return EOF (with a console warning). Running the
  engine on a _separate_ origin to isolate session data is a further option —
  see Cross-origin sandbox mode below.
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

- Deployers of caturra-based pages must set COOP/COEP headers to get interactive
  console input; this goes in the README.
- The WASM module is loaded inside the worker only; the main thread never
  instantiates the engine (version queries etc. go through the message
  protocol).
- stdout/stderr arrive on the main thread as message events, so output ordering
  relative to program completion is preserved by the protocol (result message
  is sent after all output messages).

## Cross-origin sandbox mode (2026-07-12)

To run the editor on one domain while keeping the engine — which executes
arbitrary student Java and owns the WASM heap — from ever touching the editor
origin's cookies or storage, the engine can run inside a **hidden iframe on a
separate origin**. Browsers partition storage and cookies per origin, so a bug
(or hostile program) in the sandbox cannot read the editor's session data.

- **Why an iframe, not a cross-origin worker.** A `Worker` always inherits the
  spawning document's origin — there is no such thing as a cross-origin worker.
  Only a document can hold a different origin, so a real `JvmWorkerSession` runs
  _inside_ the iframe (`sandbox-host.ts`), same-origin with its own worker; all
  existing SAB machinery works there unchanged.
- **Why the SharedArrayBuffer never leaves the sandbox.** A `SharedArrayBuffer`
  cannot cross a cross-site boundary — different registrable domains are separate
  agent clusters, and `postMessage` throws `DataCloneError`. So nothing
  SAB-shaped crosses. The editor holds a thin RPC proxy (`remote-session.ts`) and
  exchanges only plain, structured-cloneable values with the sandbox over a
  private `MessagePort` (`sandbox-rpc.ts`). The engine's stdin/Swing/pause
  callbacks are already async, so the inner worker just parks on its SAB until
  the editor's answer returns over the port — the worker protocol is unchanged,
  and blocking stdin/Swing/debug keep working across the boundary.
- **Handshake & trust.** The editor embeds the iframe with
  `allow="cross-origin-isolated"` and _no_ `sandbox` attribute (an opaque origin
  would lose cross-origin isolation and storage partitioning — the whole point),
  then posts an init message targeted at the exact sandbox origin, transferring
  one end of a `MessageChannel`. The sandbox (`startSandboxHost`) validates
  `event.origin` against an allowlist before adopting the port; the port itself
  is then a private capability needing no further origin checks.

### Deployment runbook

Two supported topologies:

**A — Single origin (default).** The engine runs in a same-origin worker; the
page sends `Cross-Origin-Opener-Policy: same-origin` +
`Cross-Origin-Embedder-Policy: require-corp` for SAB (see the serving requirement
above). This is the current playground / GitHub Pages setup — static hosts use
`coi-serviceworker.js` to fake the headers same-origin. Leave
`VITE_SANDBOX_ORIGIN` unset.

**B — Two origins (isolated).** The engine runs in an iframe on a _different
registrable domain_ from the editor. GitHub Pages cannot send the headers below,
so the sandbox needs a header-capable host (Cloudflare, Netlify, S3+CloudFront,
…). Configure both origins:

- **Editor origin** (`apps/playground`) — response headers:
  - `Cross-Origin-Opener-Policy: same-origin`
  - `Cross-Origin-Embedder-Policy: require-corp`

  so the page is cross-origin isolated and can delegate isolation into the frame.
  Build with `VITE_SANDBOX_ORIGIN=https://sandbox.example`. The
  `allow="cross-origin-isolated"` attribute on the iframe is set automatically by
  `RemoteJvmSession`.

- **Sandbox origin** (`apps/sandbox`) — response headers on the document **and
  its worker/wasm/assets**:
  - `Cross-Origin-Opener-Policy: same-origin`
  - `Cross-Origin-Embedder-Policy: require-corp` — so the frame is isolated → SAB
  - `Cross-Origin-Resource-Policy: cross-origin` — so the editor's `require-corp`
    may embed it
  - `Content-Security-Policy: frame-ancestors https://editor.example` — so only
    the editor may frame it

  Build with `VITE_EDITOR_ORIGINS=https://editor.example` (comma-separated for
  more than one editor). `apps/sandbox/vite.config.ts` already emits the first
  three headers for `dev`/`preview`; production hosting must replicate all four.

Verification checklist: editor page reports `crossOriginIsolated === true`; the
sandbox document is reachable and reports `crossOriginIsolated === true` inside
the frame; the sandbox origin differs from the editor origin (a different
registrable domain, not merely a different port, so cookies partition too). The
two-origin path is exercised by `e2e/sandbox.spec.ts` (editor on `localhost`,
sandbox on `127.0.0.1`) — compile/run, blocking stdin across the boundary, and
storage isolation. For local or manual testing, a **loopback-only**
`?sandbox=<origin>` query parameter flips a single editor build into iframe mode;
it is ignored for non-loopback origins, so a deployed build cannot be redirected
at an arbitrary host.

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
  synthesizes `class __CaturraWatch {{ static String __eval(<the paused
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
- Paused locals expand instance fields one level, sorted
  (`Dog@1{age=3, buddy=Dog@2, name="Rex"}`); nested references stay
  shallow. Locals appear only after their initializer runs (LVT
  `start_pc` is post-store, like javac).
- The playground supports multiple files as tabs (2026-07-03):
  `Main.java` is fixed; each tab is a detached CodeMirror `EditorState`
  so breakpoints and squiggles travel with their file; compiles send
  every tab; pausing switches to the paused frame's tab automatically.
- The playground editor is CodeMirror 6 (bundled locally — no CDN)
  with Java syntax and the traditional debug UX: clicking a line
  number toggles a red breakpoint dot in a dedicated gutter, the
  paused line is highlighted and scrolled into view, and gutter edits
  made while paused ride back on the resume command's breakpoint
  replacement. Playwright drives the editor through
  `window.playground` hooks (setSource/toggleBreakpoint) rather than
  CodeMirror's contenteditable internals; one test exercises the real
  line-number click path.

## Theater sound (2026-07-12)

`org.code.theater` sound plays on the **editor**, never the engine: the hidden
worker/sandbox can't drive audio (autoplay policy), and the editor is where the
Run gesture unlocks Web Audio. The engine only records intent — into the theater
command log and companion VFS files — and `TheaterViz` on the editor synthesizes
and plays.

- `playNote(...)` — a Web Audio oscillator per note (MIDI → Hz; PIANO triangle,
  BASS sawtooth, percussive envelope). Nothing leaves the engine beyond the
  `note` log line.
- `playSound(double[])` — the engine serializes the samples to a VFS file
  `__caturra_pcm_<id>` and logs `sound pcm <id> <len>`; the editor reads the file
  and plays it as an `AudioBuffer`. This is the from-scratch synthesis /
  array-manipulation path.
- `playSound(String)` / `SoundLoader.read(name)` — named assets, served
  **same-origin** (COEP-safe to fetch). Before a run the editor decodes the asset
  and preloads its samples to `__caturra_sound_<name>`, so `read` returns real
  audio to manipulate and `playSound(name)` plays the decoded buffer. Names with
  no asset fall back to indexable silence (`double[441000]`).
- Sample wire format (engine ↔ editor, both directions): the sample count, then
  that many space-separated signed 16-bit ints. The VM writes it with
  `PrintWriter` (it has no binary streams) and reads it with `Scanner`.

Because sample data rides ordinary VFS files (`writeFile`/`readTextFile`), sound
works identically in worker and sandbox modes — the data crosses the RPC, the
audio stays on the editor.

### Named starter assets (sounds and images)

The CSA levels play Code.org-hosted sounds (`birds.wav`, `retrobeat.wav`, …) and
draw hosted images (`house.png`, `motivation.jpg`, …), keyed by uuid in each
level's `starter_assets` map at
`studio.code.org/level_starter_assets/<level>/uuid/<uuid>`. Those responses carry
no CORP/CORS headers, so a cross-origin **isolated** editor (`COEP: require-corp`)
cannot fetch them at runtime — they must be served same-origin. They are also far
too large to vendor (~177 MB of audio, ~117 MB of images).

So they are **downloaded as an install step, not vendored** (like `artifacts/`):
`pnpm assets:fetch` pulls them and transcodes for transport into the git-ignored
`apps/playground/public/level-assets/`, writing a `manifest.json` of name → path.
Sounds become mono MP3 — `decodeAudioData` handles MP3 everywhere (Ogg does not in
Safari) and resamples to the AudioContext's rate, so the program still sees
44.1 kHz samples. Images are downscaled to fit the 400×400 stage, keeping PNG so
alpha survives.

The name → (level, uuid) map is `apps/playground/scripts/level-assets.json`,
generated from every level's `starter_assets`. That is the authoritative list and
**must not** be derived by scanning starter code: a level often provides an asset
the student is expected to name themselves (e.g. `cheerful.wav` in
`CSA U4L1-L6d_pilot_oo2025` appears only in the solution). When the git-ignored
`artifacts/` tree is present the script reads it directly, so the list cannot go
stale; `--regen` rewrites the JSON from it.

Resolution is by **name**, not by level: the editor scans a program's source for
the asset literals it names and loads only those (there are ~674 assets — far too
many to preload wholesale). It merges the fetched manifest over its built-in
defaults (`beatbox.wav`, which ships with the app, so sound works with no
download). A name that resolves to nothing stays silent, or draws the placeholder
box for an image. Note a name can map to different content in different levels;
the fetch keeps one representative per name.

### Image pixels

`org.code.media.Image` carries real pixels, the mirror of the sound path — but it
**cannot** use the same text-over-`Scanner` transport. A 400×400 image is 160k
pixels; a probe pushing that many ints through `PrintWriter`/`Scanner` took **65
seconds** in the interpreter (the audio path only just gets away with ~100k
samples). So pixels cross natively:

- The VFS carries a **binary** pixel buffer: width and height as little-endian
  `u32`, then RGB triples, row-major.
- Three intrinsics (`System.__imageDims`, `__imagePixels`, `__writeImage`) pack
  and unpack it in Rust, so an image loads or stores in **one** native call
  instead of 160k interpreted ones. `invoke_static` takes the VFS for this.
- `Image` holds its pixels **packed** (`int[] px`, `0xRRGGBB`, row-major) rather
  than three `int[][]`, so construction is a direct assignment from the intrinsic
  with no per-pixel decode loop. `getPixel`/`setPixel` unpack on demand, which is
  the student's own loop cost and unavoidable.

Flow: before a run the editor decodes each named image the program mentions and
writes its pixels to `__caturra_image_<name>`, so `new Image(name)` returns the
real thing (an unresolved name still yields the blank 100×100). `drawImage(Image,
…)` writes the (possibly edited) buffer to `__caturra_img_<id>` and logs `image
obj <id> …`; the editor reads it back and draws it. So pixel-editing lessons —
filters, greyscale, channel swaps — run and show their result.
