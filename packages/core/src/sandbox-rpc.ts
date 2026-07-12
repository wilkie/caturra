/**
 * The *outer* message protocol, between the editor page and a JVM
 * running inside a cross-origin sandbox iframe. This is distinct from
 * the inner worker protocol (`protocol.ts`), which stays entirely
 * within the sandbox origin and is the only channel that carries a
 * SharedArrayBuffer.
 *
 * A `SharedArrayBuffer` cannot cross a cross-site boundary (different
 * registrable domains are separate agent clusters), so nothing here
 * carries one: only plain, structured-cloneable values travel between
 * the editor and the sandbox. The blocking stdin/Swing/debug channels
 * live wholly inside the sandbox, where the worker and the iframe's
 * main thread share an origin. See `specs/EXECUTION.md`.
 *
 * The transport is a private {@link MessagePort} handed to the iframe
 * during the origin-checked {@link SandboxInitMessage} handshake. On top
 * of it, {@link RpcEndpoint} provides symmetric request/response plus
 * one-way notifications, used in both directions:
 *
 *   editor → host   session methods ({@link SandboxMethods}) and
 *                   control notifications ({@link SandboxNotifications})
 *   host → editor   callback requests ({@link SandboxCallbacks}, e.g. the
 *                   next stdin line) and output streams ({@link SandboxEvents})
 */
import type {
  CompileResult,
  DebugBreakpoint,
  DebugControlResponse,
  DebugPauseSnapshot,
  JavaSourceFile,
  RunResult,
} from './index.js';

/** Bumped when the wire format changes incompatibly. */
export const SANDBOX_PROTOCOL_VERSION = 1;

/** Discriminates a caturra handshake from other cross-frame chatter. */
export const SANDBOX_CHANNEL = 'caturra-sandbox';

/**
 * Sent by the editor to the sandbox iframe via `window.postMessage`
 * (targeted at the exact sandbox origin), transferring `port2` of a
 * fresh {@link MessageChannel} as the sole entry in the message's
 * transfer list. The iframe validates `event.origin` before adopting
 * the port; everything afterwards flows over that port.
 */
export interface SandboxInitMessage {
  channel: typeof SANDBOX_CHANNEL;
  protocol: number;
}

/** Whether an incoming message is a caturra handshake. */
export function isSandboxInit(data: unknown): data is SandboxInitMessage {
  return (
    typeof data === 'object' &&
    data !== null &&
    (data as { channel?: unknown }).channel === SANDBOX_CHANNEL
  );
}

// ---------------------------------------------------------------------------
// Typed protocol surface (the source of truth for both endpoints)
// ---------------------------------------------------------------------------

/** A monotonic id, minted by the editor, scoping one run's streams and callbacks. */
export type RunId = number;

/** Which optional per-run callbacks the editor wants the host to wire. */
export interface RunCallbackFlags {
  stdout: boolean;
  stderr: boolean;
  stdin: boolean;
  swingEvent: boolean;
  swingDialog: boolean;
}

/** Serializable half of a `run` call — the run options minus their (unsendable) callbacks. */
export interface RunCallParams {
  runId: RunId;
  mainClass: string;
  args: string[];
  wants: RunCallbackFlags;
}

/** Serializable half of a `runDebug` call. `onPause` is always wired for a debug run. */
export interface DebugRunCallParams extends RunCallParams {
  breakpoints: DebugBreakpoint[];
  watches: string[];
}

/** editor → host request/response methods (mirror {@link JvmSessionApi}). */
export interface SandboxMethods {
  version: { params: undefined; result: string };
  compile: { params: { sources: JavaSourceFile[] }; result: CompileResult };
  run: { params: RunCallParams; result: RunResult };
  runDebug: { params: DebugRunCallParams; result: RunResult };
  writeFile: { params: { path: string; contents: string | Uint8Array }; result: undefined };
  readFile: { params: { path: string }; result: Uint8Array };
  listDir: { params: { path: string }; result: string[] };
  exists: { params: { path: string }; result: boolean };
  mkdir: { params: { path: string }; result: undefined };
  remove: { params: { path: string }; result: undefined };
}

/** editor → host one-way control notifications (no payload). */
export interface SandboxNotifications {
  /** Ask the active debug run to pause at the next opportunity. */
  requestPause: undefined;
  /** Hard-stop the session and tear down the channel. */
  terminate: undefined;
}

/**
 * host → editor request/response callbacks, raised while a run is in
 * flight. The editor answers by invoking the caller's callback; the
 * host parks its SharedArrayBuffer channel until the answer returns.
 */
export interface SandboxCallbacks {
  stdin: { params: { runId: RunId }; result: string | null };
  swingEvent: { params: { runId: RunId; tree: string }; result: string | null };
  swingDialog: { params: { runId: RunId; kind: string; message: string }; result: string | null };
  pause: { params: { runId: RunId; snapshot: DebugPauseSnapshot }; result: DebugControlResponse };
}

/** host → editor one-way output streams. */
export interface SandboxEvents {
  stdout: { runId: RunId; text: string };
  stderr: { runId: RunId; text: string };
}

// ---------------------------------------------------------------------------
// Transport: a symmetric request/response + notify endpoint over a port
// ---------------------------------------------------------------------------

/** Wire frames exchanged over the {@link MessagePort}. */
export type RpcMessage =
  | { kind: 'request'; id: number; method: string; params: unknown }
  | { kind: 'response'; id: number; ok: true; value: unknown }
  | { kind: 'response'; id: number; ok: false; error: string }
  | { kind: 'notify'; method: string; params: unknown };

type RequestHandler = (params: never) => unknown;
type NotifyListener = (params: never) => void;

interface PendingCall {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
}

const CLOSED_MESSAGE = 'caturra sandbox channel is closed';

/**
 * A JSON-RPC-shaped endpoint over a {@link MessagePort}. Symmetric:
 * either side may {@link request} the peer, {@link notify} it, register
 * a request {@link handle}r, or listen for a notification with {@link on}.
 *
 * Correlation ids are endpoint-local: a `response` always answers one of
 * *our* outstanding requests, while a `request` is always the peer asking
 * *us*, so the two directions never collide. Port delivery is ordered, so
 * a run's streamed output always precedes its final result.
 */
export class RpcEndpoint {
  readonly #port: MessagePort;
  #nextId = 1;
  #closed = false;
  readonly #pending = new Map<number, PendingCall>();
  readonly #handlers = new Map<string, RequestHandler>();
  readonly #listeners = new Map<string, NotifyListener>();

  constructor(port: MessagePort) {
    this.#port = port;
    port.onmessage = (event: MessageEvent<RpcMessage>) => {
      void this.#dispatch(event.data);
    };
  }

  /** Call `method` on the peer and resolve with its result (or reject on error). */
  request<T = unknown>(method: string, params?: unknown): Promise<T> {
    if (this.#closed) {
      return Promise.reject(new Error(CLOSED_MESSAGE));
    }
    const id = this.#nextId++;
    const promise = new Promise<unknown>((resolve, reject) => {
      this.#pending.set(id, { resolve, reject });
    });
    this.#port.postMessage({ kind: 'request', id, method, params } satisfies RpcMessage);
    return promise as Promise<T>;
  }

  /** Fire a one-way notification at the peer. No reply is expected. */
  notify(method: string, params?: unknown): void {
    if (this.#closed) {
      return;
    }
    this.#port.postMessage({ kind: 'notify', method, params } satisfies RpcMessage);
  }

  /**
   * Register the responder for incoming `method` requests (last
   * registration wins). Annotate the handler's `params` to type the
   * payload at the call site — e.g. `handle('add', (p: AddParams) => …)`.
   */
  handle(method: string, handler: RequestHandler): void {
    this.#handlers.set(method, handler);
  }

  /** Register a listener for incoming `method` notifications (last registration wins). */
  on(method: string, listener: NotifyListener): void {
    this.#listeners.set(method, listener);
  }

  /** Reject every outstanding request and close the port. Idempotent. */
  close(): void {
    if (this.#closed) {
      return;
    }
    this.#closed = true;
    const failure = new Error(CLOSED_MESSAGE);
    for (const pending of this.#pending.values()) {
      pending.reject(failure);
    }
    this.#pending.clear();
    this.#port.onmessage = null;
    this.#port.close();
  }

  async #dispatch(message: RpcMessage): Promise<void> {
    switch (message.kind) {
      case 'response': {
        const pending = this.#pending.get(message.id);
        if (!pending) {
          return;
        }
        this.#pending.delete(message.id);
        if (message.ok) {
          pending.resolve(message.value);
        } else {
          pending.reject(new Error(message.error));
        }
        return;
      }
      case 'request': {
        const handler = this.#handlers.get(message.method);
        if (!handler) {
          this.#reply({
            kind: 'response',
            id: message.id,
            ok: false,
            error: `caturra sandbox: no handler for "${message.method}"`,
          });
          return;
        }
        try {
          const value = await handler(message.params as never);
          this.#reply({ kind: 'response', id: message.id, ok: true, value });
        } catch (error) {
          this.#reply({ kind: 'response', id: message.id, ok: false, error: errorMessage(error) });
        }
        return;
      }
      case 'notify': {
        this.#listeners.get(message.method)?.(message.params as never);
        return;
      }
    }
  }

  #reply(message: RpcMessage): void {
    if (this.#closed) {
      return;
    }
    this.#port.postMessage(message);
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
