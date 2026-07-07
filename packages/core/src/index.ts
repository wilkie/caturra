/**
 * @caturra/core — the browser-facing API of the caturra engine.
 *
 * The engine itself (compiler + VM) is Rust compiled to WebAssembly;
 * this package loads that module and wraps its raw boundary in typed,
 * documented methods. Nothing here talks to a server: compilation and
 * execution happen entirely in the page.
 */
import initWasm, {
  JvmSession as WasmSession,
  version as wasmVersion,
} from './wasm/generated/caturra.js';

/** One Java source file, e.g. `{ path: 'Main.java', text: '...' }`. */
export interface JavaSourceFile {
  path: string;
  text: string;
}

/** A 1-based line/column position, as editors display them. */
export interface Position {
  line: number;
  column: number;
}

/** A compiler error or warning, with a source location when known. */
export interface Diagnostic {
  severity: 'error' | 'warning';
  message: string;
  path: string;
  start?: Position | null;
  end?: Position | null;
}

/** The result of a `javac` call. */
export interface CompileResult {
  success: boolean;
  /** Binary names of the produced classes, e.g. `['Main']`. */
  classNames: string[];
  diagnostics: Diagnostic[];
  /** Entry class for a validation ("Test") run, if JUnit @Test methods were found. */
  validationEntry?: string;
}

/** The result of a `java` call. */
export interface RunResult {
  /**
   * How the program ended. `error` covers uncaught exceptions and VM
   * errors; `stopped` means the debugger terminated it.
   */
  status: 'completed' | 'exited' | 'stopped' | 'error';
  exitCode: number;
  error?: string | null;
}

// ----- Debugging -----

/** A source breakpoint: file as compiled (`Main.java`) + 1-based line. */
export interface DebugBreakpoint {
  file: string;
  line: number;
}

/** How to proceed after a pause. */
export type DebugCommandName = 'continue' | 'stepOver' | 'stepInto' | 'stepOut' | 'terminate';

/**
 * The host's answer to a pause. The `refresh` pseudo-command replaces
 * the watch list and re-evaluates without resuming — `onPause` is
 * called again with fresh results.
 */
export interface DebugControlResponse {
  command: DebugCommandName | 'refresh';
  /** Full replacement of the breakpoint set (edits made while paused). */
  breakpoints?: DebugBreakpoint[];
  /** Full replacement of the watch expressions. */
  watches?: string[];
}

/** One named local at a pause point. */
export interface DebugLocal {
  name: string;
  /** Java source type (empty when not representable). */
  type: string;
  /** Rendered value for display. */
  value: string;
}

/** One paused frame, innermost first in {@link DebugPauseSnapshot}. */
export interface DebugFrame {
  className: string;
  methodName: string;
  sourceFile: string;
  /** Current 1-based source line, when line info covers the pc. */
  line: number | null;
  /** Named locals live at the pause point. */
  locals: DebugLocal[];
}

/** One watch expression's outcome at a pause. */
export interface WatchResult {
  expression: string;
  /** Rendered value, when evaluation succeeded. */
  value?: string | null;
  /** Compile or runtime error message otherwise (javac wording). */
  error?: string | null;
}

/** Everything needed to render a paused program. */
export interface DebugPauseSnapshot {
  reason: 'breakpoint' | 'step' | 'interrupt';
  frames: DebugFrame[];
  /** Active watch expressions evaluated against this pause. */
  watchResults: WatchResult[];
}

/** Options for {@link JvmSession.runDebug}. */
export interface DebugRunOptions extends RunOptions {
  breakpoints?: DebugBreakpoint[];
  /** Watch expressions evaluated at every pause. */
  watches?: string[];
  /**
   * Called at every pause; must return the command synchronously (in
   * the worker this blocks on a SharedArrayBuffer until the user
   * chooses — see JvmWorkerSession.runDebug for the async form).
   */
  onPause: (snapshot: DebugPauseSnapshot) => DebugControlResponse;
  /** Polled periodically; return `true` to pause (pause button). */
  pollInterrupt?: () => boolean;
}

/** Console hooks for a program run. */
export interface RunOptions {
  /** Command-line arguments passed to `main(String[] args)`. */
  args?: string[];
  /** Receives chunks of standard output as the program writes them. */
  onStdout?: (text: string) => void;
  /** Receives chunks of standard error as the program writes them. */
  onStderr?: (text: string) => void;
  /**
   * Called when the program reads a line from standard input
   * (`Scanner`, `System.in`). Return `null` to signal end of input.
   */
  readStdin?: () => string | null;
  /**
   * Swing event pump. Called with the current component tree (JSON) each
   * time an interactive `JFrame` needs the next event: render the tree and
   * return the next event's payload — the activated component's id, then
   * newline-separated `id=value` field states — or `null` to close the
   * window. In the worker this blocks on a SharedArrayBuffer until the
   * user interacts (see `JvmWorkerSession.run`).
   */
  awaitUiEvent?: (tree: string) => string | null;
}

/** Accepted by {@link initJvm} to override where the .wasm comes from. */
export type WasmSource = Parameters<typeof initWasm>[0];

let initPromise: Promise<void> | undefined;

/**
 * Load and instantiate the WASM engine. Called implicitly by
 * {@link createJvmSession}; call it directly to preload, or to supply
 * the module bytes yourself (e.g. in Node-based tests).
 */
export async function initJvm(wasm?: WasmSource): Promise<void> {
  initPromise ??= initWasm(wasm).then(() => undefined);
  await initPromise;
}

/** Version of the underlying engine, e.g. `0.1.0`. */
export async function jvmVersion(): Promise<string> {
  await initJvm();
  return wasmVersion();
}

/**
 * An isolated JVM sandbox: a virtual filesystem plus whatever classes
 * have been compiled into it. Sessions share nothing, so a page can run
 * several programs side by side.
 */
export class JvmSession {
  readonly #inner: WasmSession;

  private constructor(inner: WasmSession) {
    this.#inner = inner;
  }

  /** Create a session, loading the WASM engine on first use. */
  static async create(wasm?: WasmSource): Promise<JvmSession> {
    await initJvm(wasm);
    return new JvmSession(new WasmSession());
  }

  /**
   * `javac`: compile Java source files. On success the classes are
   * retained by the session and can be started with {@link run}.
   */
  compile(sources: JavaSourceFile[]): CompileResult {
    return this.#inner.compile(sources) as CompileResult;
  }

  /** `java`: run `public static void main` of a compiled class. */
  run(mainClass: string, options: RunOptions = {}): RunResult {
    return this.#inner.run(
      mainClass,
      options.args ?? [],
      (text: string) => options.onStdout?.(text),
      (text: string) => options.onStderr?.(text),
      options.readStdin ?? null,
      options.awaitUiEvent ?? null,
    ) as RunResult;
  }

  /**
   * `java` under a debugger: pauses at breakpoints (and interrupts),
   * calling `onPause` for each stop. Synchronous — see
   * `JvmWorkerSession.runDebug` for the browser-friendly async form.
   */
  runDebug(mainClass: string, options: DebugRunOptions): RunResult {
    return this.#inner.runDebug(
      mainClass,
      options.args ?? [],
      JSON.stringify(options.breakpoints ?? []),
      JSON.stringify(options.watches ?? []),
      (text: string) => options.onStdout?.(text),
      (text: string) => options.onStderr?.(text),
      options.readStdin ?? null,
      (snapshotJson: string) => {
        const snapshot = JSON.parse(snapshotJson) as DebugPauseSnapshot;
        return JSON.stringify(options.onPause(snapshot));
      },
      options.pollInterrupt ?? null,
    ) as RunResult;
  }

  // ----- Virtual filesystem (backs java.io.File inside the VM) -----

  /** Write a file, creating parent directories as needed. */
  writeFile(path: string, contents: string | Uint8Array): void {
    const bytes = typeof contents === 'string' ? new TextEncoder().encode(contents) : contents;
    this.#inner.writeFile(path, bytes);
  }

  /** Read a file as bytes. Throws if it does not exist. */
  readFile(path: string): Uint8Array {
    return this.#inner.readFile(path);
  }

  /** Read a file as UTF-8 text. Throws if it does not exist. */
  readTextFile(path: string): string {
    return new TextDecoder().decode(this.readFile(path));
  }

  /** List the immediate children of a directory, as absolute paths. */
  listDir(path: string): string[] {
    return this.#inner.listDir(path);
  }

  /** Whether a file or directory exists. */
  exists(path: string): boolean {
    return this.#inner.exists(path);
  }

  /** Create a directory (and any missing parents). */
  mkdir(path: string): void {
    this.#inner.mkdir(path);
  }

  /** Delete a file or empty directory. */
  remove(path: string): void {
    this.#inner.remove(path);
  }
}

/** Shorthand for {@link JvmSession.create}. */
export async function createJvmSession(wasm?: WasmSource): Promise<JvmSession> {
  return JvmSession.create(wasm);
}

// The primary browser API: the engine in a Web Worker with blocking
// stdin support (see specs/EXECUTION.md). The synchronous JvmSession
// above remains for Node, tests, and the worker's own internals.
export { JvmWorkerSession } from './worker-session.js';
export type { StdinSource, WorkerDebugRunOptions, WorkerRunOptions } from './worker-session.js';
