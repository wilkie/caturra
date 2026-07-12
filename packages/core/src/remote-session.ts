/**
 * The editor-side driver for a JVM running inside a cross-origin
 * sandbox iframe (see `specs/EXECUTION.md`). It implements the same
 * {@link JvmSessionApi} as the local {@link JvmWorkerSession}, so the
 * editor can be pointed at either without code changes.
 *
 * All engine state — and every SharedArrayBuffer blocking channel —
 * lives inside the sandbox. This class only exchanges plain,
 * structured-cloneable messages with it over a private {@link
 * MessagePort}: it forwards method calls, re-emits the sandbox's
 * `stdout`/`stderr` streams to the caller's callbacks, and answers the
 * sandbox's `stdin`/Swing/pause callbacks by invoking the caller's
 * (possibly async) handlers. The round-trip is exactly the async
 * `stdin` source the engine already tolerates, so the worker protocol
 * inside the sandbox is unchanged.
 */
import type {
  CompileResult,
  DebugControlResponse,
  DebugPauseSnapshot,
  JavaSourceFile,
  RunResult,
} from './index.js';
import {
  RpcEndpoint,
  SANDBOX_CHANNEL,
  SANDBOX_PROTOCOL_VERSION,
  type DebugRunCallParams,
  type RunCallParams,
  type RunId,
  type SandboxCallbacks,
  type SandboxEvents,
  type SandboxInitMessage,
} from './sandbox-rpc.js';
import type { JvmSessionApi, WorkerDebugRunOptions, WorkerRunOptions } from './session-api.js';
import { normalizeStdin } from './session-api.js';

/** Options for {@link RemoteJvmSession.create}. */
export interface RemoteJvmSessionOptions {
  /**
   * Origin of the sandbox host, e.g. `https://sandbox.example.com`. Must
   * be a different registrable domain from the editor for storage/cookie
   * isolation to apply.
   */
  sandboxOrigin: string;
  /** Path of the runner document on the sandbox origin. Defaults to `/`. */
  runnerPath?: string;
  /** Where to attach the hidden iframe. Defaults to `document.body`. */
  container?: HTMLElement;
  /** How long to wait for the handshake before failing. Defaults to 30s. */
  timeoutMs?: number;
}

/** The caller's callbacks for one in-flight run, keyed by {@link RunId}. */
interface ActiveRun {
  onStdout?: ((text: string) => void) | undefined;
  onStderr?: ((text: string) => void) | undefined;
  nextLine?: (() => Promise<string | null>) | undefined;
  onSwingEvent?: ((tree: string) => Promise<string | null>) | undefined;
  onSwingDialog?: ((kind: string, message: string) => Promise<string | null>) | undefined;
  onPause?:
    | ((snapshot: DebugPauseSnapshot) => DebugControlResponse | Promise<DebugControlResponse>)
    | undefined;
}

const DEFAULT_TIMEOUT_MS = 30_000;

export class RemoteJvmSession implements JvmSessionApi {
  readonly #endpoint: RpcEndpoint;
  readonly #teardown: () => void;
  readonly #runs = new Map<RunId, ActiveRun>();
  #nextRunId = 1;
  #terminated = false;

  private constructor(endpoint: RpcEndpoint, teardown: () => void) {
    this.#endpoint = endpoint;
    this.#teardown = teardown;

    // Sandbox → editor output streams.
    endpoint.on('stdout', (p: SandboxEvents['stdout']) => {
      this.#runs.get(p.runId)?.onStdout?.(p.text);
    });
    endpoint.on('stderr', (p: SandboxEvents['stderr']) => {
      this.#runs.get(p.runId)?.onStderr?.(p.text);
    });

    // Sandbox → editor blocking callbacks. Like JvmWorkerSession's answers,
    // a throwing/absent handler resolves to the safe default (EOF / close /
    // terminate) rather than propagating an error into the parked engine.
    endpoint.handle('stdin', (p: SandboxCallbacks['stdin']['params']) =>
      this.#answerStdin(p.runId),
    );
    endpoint.handle('swingEvent', (p: SandboxCallbacks['swingEvent']['params']) =>
      this.#answerSwing(p.runId, p.tree),
    );
    endpoint.handle('swingDialog', (p: SandboxCallbacks['swingDialog']['params']) =>
      this.#answerDialog(p.runId, p.kind, p.message),
    );
    endpoint.handle('pause', (p: SandboxCallbacks['pause']['params']) =>
      this.#answerPause(p.runId, p.snapshot),
    );
  }

  /**
   * Spawn the sandbox iframe, complete the port handshake, and wait for
   * the engine to answer. Rejects (and tears the iframe down) if the
   * sandbox does not load or respond within `timeoutMs`.
   */
  static async create(options: RemoteJvmSessionOptions): Promise<RemoteJvmSession> {
    const url = new URL(options.runnerPath ?? '/', options.sandboxOrigin);
    const targetOrigin = url.origin;
    const container = options.container ?? document.body;
    const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;

    const iframe = document.createElement('iframe');
    // Delegate cross-origin isolation so the sandbox's own worker keeps
    // SharedArrayBuffer. Deliberately no `sandbox` attribute: the frame
    // must retain its real origin (opaque origins lose COI and storage
    // partitioning, which is the entire point of the separate domain).
    iframe.setAttribute('allow', 'cross-origin-isolated');
    iframe.setAttribute('title', 'caturra sandbox');
    iframe.setAttribute('aria-hidden', 'true');
    iframe.style.cssText = 'position:absolute;width:0;height:0;border:0;visibility:hidden;';

    const loaded = new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        reject(
          new Error(
            `caturra sandbox iframe did not load within ${String(timeoutMs)}ms (${url.href})`,
          ),
        );
      }, timeoutMs);
      iframe.addEventListener(
        'load',
        () => {
          clearTimeout(timer);
          resolve();
        },
        { once: true },
      );
    });
    iframe.src = url.href;
    container.appendChild(iframe);
    const remove = (): void => {
      iframe.remove();
    };

    try {
      await loaded;
    } catch (error) {
      remove();
      throw error;
    }

    const frameWindow = iframe.contentWindow;
    if (!frameWindow) {
      remove();
      throw new Error('caturra sandbox iframe has no content window');
    }

    // Hand the sandbox one end of a private channel; it validates our
    // origin before adopting the port. The port itself is unguessable, so
    // no origin checks are needed on the messages that follow.
    const channel = new MessageChannel();
    const init: SandboxInitMessage = {
      channel: SANDBOX_CHANNEL,
      protocol: SANDBOX_PROTOCOL_VERSION,
    };
    frameWindow.postMessage(init, targetOrigin, [channel.port2]);

    const session = new RemoteJvmSession(new RpcEndpoint(channel.port1), remove);
    try {
      await withTimeout(
        session.version(),
        timeoutMs,
        `caturra sandbox did not respond within ${String(timeoutMs)}ms (${targetOrigin})`,
      );
    } catch (error) {
      session.terminate();
      throw error;
    }
    return session;
  }

  /**
   * Wrap an already-open {@link MessagePort} whose far end speaks the
   * sandbox protocol, skipping the iframe/handshake. For custom
   * transports and tests; prefer {@link create} in the browser.
   */
  static overPort(port: MessagePort, teardown: () => void = noop): RemoteJvmSession {
    return new RemoteJvmSession(new RpcEndpoint(port), teardown);
  }

  async version(): Promise<string> {
    return this.#endpoint.request<string>('version', undefined);
  }

  async compile(sources: JavaSourceFile[]): Promise<CompileResult> {
    return this.#endpoint.request<CompileResult>('compile', { sources });
  }

  async run(mainClass: string, options: WorkerRunOptions = {}): Promise<RunResult> {
    const runId = this.#register(options);
    const params: RunCallParams = {
      runId,
      mainClass,
      args: options.args ?? [],
      wants: {
        stdout: options.onStdout !== undefined,
        stderr: options.onStderr !== undefined,
        stdin: options.stdin !== undefined,
        swingEvent: options.onSwingEvent !== undefined,
        swingDialog: options.onSwingDialog !== undefined,
      },
    };
    try {
      return await this.#endpoint.request<RunResult>('run', params);
    } finally {
      this.#runs.delete(runId);
    }
  }

  async runDebug(mainClass: string, options: WorkerDebugRunOptions): Promise<RunResult> {
    const runId = this.#register(options, options.onPause);
    const params: DebugRunCallParams = {
      runId,
      mainClass,
      args: options.args ?? [],
      breakpoints: options.breakpoints ?? [],
      watches: options.watches ?? [],
      wants: {
        stdout: options.onStdout !== undefined,
        stderr: options.onStderr !== undefined,
        stdin: options.stdin !== undefined,
        swingEvent: options.onSwingEvent !== undefined,
        swingDialog: options.onSwingDialog !== undefined,
      },
    };
    try {
      return await this.#endpoint.request<RunResult>('runDebug', params);
    } finally {
      this.#runs.delete(runId);
    }
  }

  requestPause(): void {
    this.#endpoint.notify('requestPause', undefined);
  }

  async writeFile(path: string, contents: string | Uint8Array): Promise<void> {
    await this.#endpoint.request('writeFile', { path, contents });
  }

  async readFile(path: string): Promise<Uint8Array> {
    return this.#endpoint.request<Uint8Array>('readFile', { path });
  }

  async readTextFile(path: string): Promise<string> {
    return new TextDecoder().decode(await this.readFile(path));
  }

  async listDir(path: string): Promise<string[]> {
    return this.#endpoint.request<string[]>('listDir', { path });
  }

  async exists(path: string): Promise<boolean> {
    return this.#endpoint.request<boolean>('exists', { path });
  }

  async mkdir(path: string): Promise<void> {
    await this.#endpoint.request('mkdir', { path });
  }

  async remove(path: string): Promise<void> {
    await this.#endpoint.request('remove', { path });
  }

  terminate(): void {
    if (this.#terminated) {
      return;
    }
    this.#terminated = true;
    // Best-effort graceful stop; tearing down the iframe below is the
    // hard guarantee that the worker and its state are gone.
    this.#endpoint.notify('terminate', undefined);
    this.#endpoint.close();
    this.#runs.clear();
    this.#teardown();
  }

  #register(options: WorkerRunOptions, onPause?: ActiveRun['onPause']): RunId {
    const runId = this.#nextRunId++;
    this.#runs.set(runId, {
      onStdout: options.onStdout,
      onStderr: options.onStderr,
      nextLine: options.stdin !== undefined ? normalizeStdin(options.stdin) : undefined,
      onSwingEvent: options.onSwingEvent,
      onSwingDialog: options.onSwingDialog,
      onPause,
    });
    return runId;
  }

  async #answerStdin(runId: RunId): Promise<string | null> {
    const run = this.#runs.get(runId);
    if (!run?.nextLine) {
      return null;
    }
    try {
      return await run.nextLine();
    } catch {
      return null;
    }
  }

  async #answerSwing(runId: RunId, tree: string): Promise<string | null> {
    const run = this.#runs.get(runId);
    if (!run?.onSwingEvent) {
      return null;
    }
    try {
      return await run.onSwingEvent(tree);
    } catch {
      return null;
    }
  }

  async #answerDialog(runId: RunId, kind: string, message: string): Promise<string | null> {
    const run = this.#runs.get(runId);
    if (!run?.onSwingDialog) {
      return null;
    }
    try {
      return await run.onSwingDialog(kind, message);
    } catch {
      return null;
    }
  }

  async #answerPause(runId: RunId, snapshot: DebugPauseSnapshot): Promise<DebugControlResponse> {
    const run = this.#runs.get(runId);
    if (!run?.onPause) {
      return { command: 'terminate' };
    }
    try {
      return await run.onPause(snapshot);
    } catch {
      return { command: 'terminate' };
    }
  }
}

function noop(): void {
  /* no teardown */
}

function withTimeout<T>(promise: Promise<T>, ms: number, message: string): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(message));
    }, ms);
    promise.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      (error: unknown) => {
        clearTimeout(timer);
        reject(error instanceof Error ? error : new Error(String(error)));
      },
    );
  });
}
