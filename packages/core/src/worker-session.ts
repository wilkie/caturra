/**
 * The primary browser API (see `specs/EXECUTION.md`): an async session
 * whose engine runs in a dedicated Web Worker, keeping the page
 * responsive and enabling truly blocking `Scanner`/`System.in` reads
 * via SharedArrayBuffer.
 *
 * Requires a cross-origin isolated page (COOP/COEP headers) for
 * interactive stdin; without isolation everything else works but stdin
 * reads return EOF.
 */
import type {
  CompileResult,
  DebugBreakpoint,
  DebugControlResponse,
  DebugPauseSnapshot,
  JavaSourceFile,
  RunResult,
} from './index.js';
import type { ResultValueByType, WorkerRequest, WorkerResponse } from './protocol.js';
import {
  createInterruptFlag,
  createStdinBuffer,
  requestInterrupt,
  supplyLine,
} from './stdin-channel.js';

/** Where a run's standard input comes from. */
export type StdinSource = string[] | (() => string | null | Promise<string | null>);

/** Options for {@link JvmWorkerSession.run}. */
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
}

/** Options for {@link JvmWorkerSession.runDebug}. */
export interface WorkerDebugRunOptions extends WorkerRunOptions {
  breakpoints?: DebugBreakpoint[];
  /**
   * Called at every pause. May be async: the engine stays parked until
   * the returned promise resolves with a command (e.g. after the user
   * clicks a step button).
   */
  onPause: (snapshot: DebugPauseSnapshot) => DebugControlResponse | Promise<DebugControlResponse>;
}

interface Pending {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
  onStdout?: ((text: string) => void) | undefined;
  onStderr?: ((text: string) => void) | undefined;
  nextLine?: (() => Promise<string | null>) | undefined;
  stdinBuffer?: SharedArrayBuffer | undefined;
  onPause?:
    | ((snapshot: DebugPauseSnapshot) => DebugControlResponse | Promise<DebugControlResponse>)
    | undefined;
  debugBuffer?: SharedArrayBuffer | undefined;
}

function normalizeStdin(source: StdinSource | undefined): () => Promise<string | null> {
  if (source === undefined) {
    return () => Promise.resolve(null);
  }
  if (Array.isArray(source)) {
    let next = 0;
    return () => Promise.resolve(next < source.length ? (source[next++] ?? null) : null);
  }
  return async () => source();
}

/**
 * An isolated JVM sandbox running in a Web Worker.
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
export class JvmWorkerSession {
  readonly #worker: Worker;
  readonly #pending = new Map<number, Pending>();
  #nextId = 1;
  #activeInterruptFlag: SharedArrayBuffer | undefined;

  private constructor(worker: Worker) {
    this.#worker = worker;
    worker.addEventListener('message', (event: MessageEvent<WorkerResponse>) => {
      this.#onMessage(event.data);
    });
    worker.addEventListener('error', (event) => {
      const failure = new Error(`jvmjs worker failed: ${event.message}`);
      for (const pending of this.#pending.values()) {
        pending.reject(failure);
      }
      this.#pending.clear();
    });
  }

  /** Spawn the worker and wait for the engine to load. */
  static async create(): Promise<JvmWorkerSession> {
    const worker = new Worker(new URL('./worker.ts', import.meta.url), {
      type: 'module',
      name: 'jvmjs-engine',
    });
    const session = new JvmWorkerSession(worker);
    await session.version();
    return session;
  }

  /** Version of the underlying engine, e.g. `0.1.0`. */
  async version(): Promise<string> {
    return this.#request('version', { type: 'version' });
  }

  /** `javac`: compile Java source files in the worker. */
  async compile(sources: JavaSourceFile[]): Promise<CompileResult> {
    return this.#request('compile', { type: 'compile', sources });
  }

  /** `java`: run a compiled class's `main` in the worker. */
  async run(mainClass: string, options: WorkerRunOptions = {}): Promise<RunResult> {
    const isolated =
      typeof SharedArrayBuffer !== 'undefined' &&
      (globalThis as { crossOriginIsolated?: boolean }).crossOriginIsolated === true;
    if (options.stdin !== undefined && !isolated) {
      console.warn(
        'jvmjs: stdin needs a cross-origin isolated page (COOP/COEP headers); ' +
          'the program will see end-of-input. See specs/EXECUTION.md.',
      );
    }
    const stdinBuffer = isolated ? createStdinBuffer() : undefined;

    const id = this.#nextId++;
    const promise = new Promise<unknown>((resolve, reject) => {
      this.#pending.set(id, {
        resolve,
        reject,
        onStdout: options.onStdout,
        onStderr: options.onStderr,
        nextLine: normalizeStdin(options.stdin),
        stdinBuffer,
      });
    });
    const request: WorkerRequest = {
      id,
      type: 'run',
      mainClass,
      args: options.args ?? [],
    };
    if (stdinBuffer) {
      request.stdinBuffer = stdinBuffer;
    }
    this.#worker.postMessage(request);
    return promise as Promise<RunResult>;
  }

  /**
   * `java` under a debugger: pauses at `breakpoints`, calling `onPause`
   * for each stop; {@link requestPause} interrupts a running program.
   * Requires a cross-origin isolated page (the pause channel blocks on
   * a SharedArrayBuffer, like stdin).
   */
  async runDebug(mainClass: string, options: WorkerDebugRunOptions): Promise<RunResult> {
    const isolated =
      typeof SharedArrayBuffer !== 'undefined' &&
      (globalThis as { crossOriginIsolated?: boolean }).crossOriginIsolated === true;
    if (!isolated) {
      throw new Error(
        'jvmjs: debugging needs a cross-origin isolated page (COOP/COEP headers); ' +
          'see specs/EXECUTION.md.',
      );
    }
    const stdinBuffer = createStdinBuffer();
    const debugBuffer = createStdinBuffer();
    const interruptFlag = createInterruptFlag();
    this.#activeInterruptFlag = interruptFlag;

    const id = this.#nextId++;
    const promise = new Promise<unknown>((resolve, reject) => {
      this.#pending.set(id, {
        resolve,
        reject,
        onStdout: options.onStdout,
        onStderr: options.onStderr,
        nextLine: normalizeStdin(options.stdin),
        stdinBuffer,
        onPause: options.onPause,
        debugBuffer,
      });
    });
    this.#worker.postMessage({
      id,
      type: 'runDebug',
      mainClass,
      args: options.args ?? [],
      breakpoints: options.breakpoints ?? [],
      debugBuffer,
      interruptFlag,
      stdinBuffer,
    } satisfies WorkerRequest);
    try {
      return (await promise) as RunResult;
    } finally {
      this.#activeInterruptFlag = undefined;
    }
  }

  /**
   * Ask the running debug program to pause at the next opportunity
   * (the pause button). No-op when no debug run is active.
   */
  requestPause(): void {
    if (this.#activeInterruptFlag) {
      requestInterrupt(this.#activeInterruptFlag);
    }
  }

  // ----- Virtual filesystem passthrough -----

  /** Write a file into the session's virtual filesystem. */
  async writeFile(path: string, contents: string | Uint8Array): Promise<void> {
    await this.#request('writeFile', { type: 'writeFile', path, contents });
  }

  /** Read a file as bytes. Rejects if it does not exist. */
  async readFile(path: string): Promise<Uint8Array> {
    return this.#request('readFile', { type: 'readFile', path });
  }

  /** Read a file as UTF-8 text. Rejects if it does not exist. */
  async readTextFile(path: string): Promise<string> {
    return new TextDecoder().decode(await this.readFile(path));
  }

  /** List the immediate children of a directory, as absolute paths. */
  async listDir(path: string): Promise<string[]> {
    return this.#request('listDir', { type: 'listDir', path });
  }

  /** Whether a file or directory exists. */
  async exists(path: string): Promise<boolean> {
    return this.#request('exists', { type: 'exists', path });
  }

  /** Create a directory (and any missing parents). */
  async mkdir(path: string): Promise<void> {
    await this.#request('mkdir', { type: 'mkdir', path });
  }

  /** Delete a file or empty directory. */
  async remove(path: string): Promise<void> {
    await this.#request('remove', { type: 'remove', path });
  }

  /**
   * Hard-stop the worker (and any running program) immediately.
   * The session is unusable afterwards; create a new one to continue.
   */
  terminate(): void {
    this.#worker.terminate();
    const terminated = new Error('jvmjs worker session was terminated');
    for (const pending of this.#pending.values()) {
      pending.reject(terminated);
    }
    this.#pending.clear();
  }

  async #request<K extends WorkerRequest['type']>(
    _kind: K,
    body: Omit<Extract<WorkerRequest, { type: K }>, 'id'>,
  ): Promise<ResultValueByType[K]> {
    const id = this.#nextId++;
    const promise = new Promise<unknown>((resolve, reject) => {
      this.#pending.set(id, { resolve, reject });
    });
    this.#worker.postMessage({ id, ...body });
    return promise as Promise<ResultValueByType[K]>;
  }

  #onMessage(message: WorkerResponse): void {
    const pending = this.#pending.get(message.id);
    if (!pending) {
      return;
    }
    switch (message.type) {
      case 'result':
        this.#pending.delete(message.id);
        pending.resolve(message.value);
        break;
      case 'error':
        this.#pending.delete(message.id);
        pending.reject(new Error(message.message));
        break;
      case 'stdout':
        pending.onStdout?.(message.text);
        break;
      case 'stderr':
        pending.onStderr?.(message.text);
        break;
      case 'stdin-request':
        void this.#answerStdin(pending);
        break;
      case 'debug-paused':
        void this.#answerPause(pending, message.snapshot);
        break;
    }
  }

  async #answerPause(pending: Pending, snapshot: DebugPauseSnapshot): Promise<void> {
    if (!pending.debugBuffer || !pending.onPause) {
      return;
    }
    let control: DebugControlResponse;
    try {
      control = await pending.onPause(snapshot);
    } catch {
      control = { command: 'terminate' };
    }
    supplyLine(pending.debugBuffer, JSON.stringify(control));
  }

  async #answerStdin(pending: Pending): Promise<void> {
    if (!pending.stdinBuffer || !pending.nextLine) {
      return;
    }
    let line: string | null;
    try {
      line = await pending.nextLine();
    } catch {
      line = null;
    }
    supplyLine(pending.stdinBuffer, line);
  }
}
