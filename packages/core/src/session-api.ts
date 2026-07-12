/**
 * The async JVM session contract shared by every driver of the engine:
 * the local {@link JvmWorkerSession} (a same-origin Web Worker) and the
 * cross-origin {@link RemoteJvmSession} (a worker inside a sandbox
 * iframe). The editor consumes this interface, so it can be pointed at
 * either driver without code changes (see `specs/EXECUTION.md`).
 */
import type {
  CompileResult,
  DebugBreakpoint,
  DebugControlResponse,
  DebugPauseSnapshot,
  JavaSourceFile,
  RunResult,
} from './index.js';

/** Where a run's standard input comes from. */
export type StdinSource = string[] | (() => string | null | Promise<string | null>);

/**
 * Reduce a {@link StdinSource} to a single per-read function returning
 * the next line (or `null` at end of input). An array is served in
 * order; a function is awaited each call.
 */
export function normalizeStdin(source: StdinSource | undefined): () => Promise<string | null> {
  if (source === undefined) {
    return () => Promise.resolve(null);
  }
  if (Array.isArray(source)) {
    let next = 0;
    return () => Promise.resolve(next < source.length ? (source[next++] ?? null) : null);
  }
  return async () => source();
}

/** Options for {@link JvmSessionApi.run}. */
export interface WorkerRunOptions {
  /** Command-line arguments passed to `main(String[] args)`. */
  args?: string[];
  /** Receives chunks of standard output as the program writes them. */
  onStdout?: (text: string) => void;
  /** Receives chunks of standard error as the program writes them. */
  onStderr?: (text: string) => void;
  /**
   * Standard input: either lines to serve in order (EOF after the
   * last), or a function called per read returning a line or `null`
   * for EOF. Needs a cross-origin isolated page.
   */
  stdin?: StdinSource;
  /**
   * Swing event pump for an interactive `JFrame`. Called with the current
   * component tree (JSON) each time the program needs the next UI event:
   * render the tree, then resolve with the next event's payload (the
   * activated component's id, then newline-separated `id=value` field
   * states) or `null` to close the window. May be async — the engine
   * stays parked until it resolves. Needs a cross-origin isolated page.
   */
  onSwingEvent?: (tree: string) => Promise<string | null>;
  /**
   * Blocking JOptionPane dialog. Called with `(kind, message)`; show a modal
   * and resolve with the response (option code / typed text) or `null` when
   * dismissed. May be async. Needs a cross-origin isolated page.
   */
  onSwingDialog?: (kind: string, message: string) => Promise<string | null>;
}

/** Options for {@link JvmSessionApi.runDebug}. */
export interface WorkerDebugRunOptions extends WorkerRunOptions {
  breakpoints?: DebugBreakpoint[];
  /** Watch expressions evaluated at every pause. */
  watches?: string[];
  /**
   * Called at every pause. May be async: the engine stays parked until
   * the returned promise resolves with a command (e.g. after the user
   * clicks a step button).
   */
  onPause: (snapshot: DebugPauseSnapshot) => DebugControlResponse | Promise<DebugControlResponse>;
}

/**
 * An isolated JVM sandbox running off the main thread: compile Java,
 * run (or debug) a `main`, and reach the virtual filesystem. Every
 * method is async because the engine lives in another context (a Web
 * Worker, or a worker inside a cross-origin iframe).
 *
 * ```ts
 * const session = await JvmWorkerSession.create();
 * const compiled = await session.compile([{ path: 'Main.java', text }]);
 * if (compiled.success) {
 *   await session.run('Main', { onStdout: append, stdin: ['first line'] });
 * }
 * session.terminate();
 * ```
 */
export interface JvmSessionApi {
  /** Version of the underlying engine, e.g. `0.1.0`. */
  version(): Promise<string>;

  /** `javac`: compile Java source files in the session. */
  compile(sources: JavaSourceFile[]): Promise<CompileResult>;

  /** `java`: run a compiled class's `main`. */
  run(mainClass: string, options?: WorkerRunOptions): Promise<RunResult>;

  /**
   * `java` under a debugger: pauses at `breakpoints`, calling `onPause`
   * for each stop; {@link requestPause} interrupts a running program.
   */
  runDebug(mainClass: string, options: WorkerDebugRunOptions): Promise<RunResult>;

  /**
   * Ask the running debug program to pause at the next opportunity
   * (the pause button). No-op when no debug run is active.
   */
  requestPause(): void;

  /** Write a file into the session's virtual filesystem. */
  writeFile(path: string, contents: string | Uint8Array): Promise<void>;

  /** Read a file as bytes. Rejects if it does not exist. */
  readFile(path: string): Promise<Uint8Array>;

  /** Read a file as UTF-8 text. Rejects if it does not exist. */
  readTextFile(path: string): Promise<string>;

  /** List the immediate children of a directory, as absolute paths. */
  listDir(path: string): Promise<string[]>;

  /** Whether a file or directory exists. */
  exists(path: string): Promise<boolean>;

  /** Create a directory (and any missing parents). */
  mkdir(path: string): Promise<void>;

  /** Delete a file or empty directory. */
  remove(path: string): Promise<void>;

  /**
   * Hard-stop the session (and any running program) immediately.
   * The session is unusable afterwards; create a new one to continue.
   */
  terminate(): void;
}
