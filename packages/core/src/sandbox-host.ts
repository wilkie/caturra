/**
 * The code that runs *inside* the sandbox iframe (see
 * `specs/EXECUTION.md`). It owns the real engine — a same-origin
 * {@link JvmWorkerSession} and its SharedArrayBuffer channels — and
 * exposes it to the editor across the cross-origin boundary via the
 * outer RPC protocol.
 *
 * The bridge is deliberately thin: incoming method calls run straight
 * against the local session, its `stdout`/`stderr` become notifications,
 * and its blocking `stdin`/Swing/pause callbacks become RPC requests
 * back to the editor. Because those session callbacks may be async, the
 * inner worker simply parks on its SharedArrayBuffer until the editor's
 * answer returns — no change to the worker protocol, and no
 * SharedArrayBuffer ever crosses the cross-origin boundary.
 */
import type { DebugControlResponse } from './index.js';
import {
  isSandboxInit,
  RpcEndpoint,
  SANDBOX_PROTOCOL_VERSION,
  type DebugRunCallParams,
  type RunCallParams,
  type SandboxMethods,
} from './sandbox-rpc.js';
import type { JvmSessionApi, WorkerDebugRunOptions, WorkerRunOptions } from './session-api.js';
import { JvmWorkerSession } from './worker-session.js';

/** The subset of `window` the handshake listener needs (injectable for tests). */
export interface HandshakeTarget {
  addEventListener(type: 'message', listener: (event: MessageEvent) => void): void;
  removeEventListener(type: 'message', listener: (event: MessageEvent) => void): void;
}

/** Options for {@link startSandboxHost}. */
export interface SandboxHostOptions {
  /**
   * Editor origins permitted to drive this sandbox, matched exactly
   * against `event.origin`. Handshakes from anywhere else are ignored.
   * (A `Content-Security-Policy: frame-ancestors` header on the sandbox
   * document should enforce the same list at the framing layer.)
   */
  allowedOrigins: string[];
  /** Engine factory. Defaults to {@link JvmWorkerSession.create}; injectable for tests. */
  createSession?: () => Promise<JvmSessionApi>;
  /** Where to listen for the handshake. Defaults to `window`. */
  target?: HandshakeTarget;
}

/**
 * Begin listening for the editor's handshake. On the first message from
 * an allowed origin carrying a {@link MessagePort}, adopt the port, wire
 * the engine to it, and stop listening (one editor per sandbox).
 */
export function startSandboxHost(options: SandboxHostOptions): void {
  const allowed = new Set(options.allowedOrigins);
  const target: HandshakeTarget = options.target ?? window;
  const createSession = options.createSession ?? (() => JvmWorkerSession.create());

  const onMessage = (event: MessageEvent): void => {
    if (!isSandboxInit(event.data)) {
      return;
    }
    if (!allowed.has(event.origin)) {
      console.warn(`caturra sandbox: rejected handshake from disallowed origin ${event.origin}`);
      return;
    }
    if (event.data.protocol !== SANDBOX_PROTOCOL_VERSION) {
      console.warn(
        `caturra sandbox: unsupported protocol ${String(event.data.protocol)} ` +
          `(this host speaks ${String(SANDBOX_PROTOCOL_VERSION)})`,
      );
      return;
    }
    const port = event.ports[0];
    if (!port) {
      console.warn('caturra sandbox: handshake carried no MessagePort');
      return;
    }
    target.removeEventListener('message', onMessage);
    wireSessionToPort(port, createSession);
  };

  target.addEventListener('message', onMessage);
}

/**
 * Wire an engine to a ready {@link MessagePort} speaking the sandbox
 * protocol. The session is created lazily on the first request. Exposed
 * for tests and custom transports; {@link startSandboxHost} is the
 * normal entry point.
 */
export function wireSessionToPort(
  port: MessagePort,
  createSession: () => Promise<JvmSessionApi>,
): RpcEndpoint {
  const endpoint = new RpcEndpoint(port);
  let sessionPromise: Promise<JvmSessionApi> | undefined;
  const session = (): Promise<JvmSessionApi> => (sessionPromise ??= createSession());

  endpoint.handle('version', async () => (await session()).version());
  endpoint.handle('compile', async (p: SandboxMethods['compile']['params']) =>
    (await session()).compile(p.sources),
  );
  endpoint.handle('run', async (p: RunCallParams) =>
    (await session()).run(p.mainClass, runOptions(endpoint, p)),
  );
  endpoint.handle('runDebug', async (p: DebugRunCallParams) =>
    (await session()).runDebug(p.mainClass, debugRunOptions(endpoint, p)),
  );
  endpoint.handle('writeFile', async (p: SandboxMethods['writeFile']['params']) => {
    await (await session()).writeFile(p.path, p.contents);
  });
  endpoint.handle('readFile', async (p: SandboxMethods['readFile']['params']) =>
    (await session()).readFile(p.path),
  );
  endpoint.handle('listDir', async (p: SandboxMethods['listDir']['params']) =>
    (await session()).listDir(p.path),
  );
  endpoint.handle('exists', async (p: SandboxMethods['exists']['params']) =>
    (await session()).exists(p.path),
  );
  endpoint.handle('mkdir', async (p: SandboxMethods['mkdir']['params']) => {
    await (await session()).mkdir(p.path);
  });
  endpoint.handle('remove', async (p: SandboxMethods['remove']['params']) => {
    await (await session()).remove(p.path);
  });

  endpoint.on('requestPause', () => {
    // Only meaningful while a debug run is live, so the session exists by now.
    if (sessionPromise) {
      void sessionPromise.then((s) => {
        s.requestPause();
      });
    }
  });
  endpoint.on('terminate', () => {
    if (sessionPromise) {
      void sessionPromise.then((s) => {
        s.terminate();
      });
    }
    endpoint.close();
  });

  return endpoint;
}

/** Bridge the caller-requested run callbacks to RPC. Absent callbacks stay unset. */
function runOptions(endpoint: RpcEndpoint, p: RunCallParams): WorkerRunOptions {
  const runId = p.runId;
  const options: WorkerRunOptions = { args: p.args };
  if (p.wants.stdout) {
    options.onStdout = (text) => {
      endpoint.notify('stdout', { runId, text });
    };
  }
  if (p.wants.stderr) {
    options.onStderr = (text) => {
      endpoint.notify('stderr', { runId, text });
    };
  }
  if (p.wants.stdin) {
    options.stdin = () => endpoint.request<string | null>('stdin', { runId });
  }
  if (p.wants.swingEvent) {
    options.onSwingEvent = (tree) => endpoint.request<string | null>('swingEvent', { runId, tree });
  }
  if (p.wants.swingDialog) {
    options.onSwingDialog = (kind, message) =>
      endpoint.request<string | null>('swingDialog', { runId, kind, message });
  }
  return options;
}

/** Like {@link runOptions}, plus the debugger's breakpoints, watches, and pause callback. */
function debugRunOptions(endpoint: RpcEndpoint, p: DebugRunCallParams): WorkerDebugRunOptions {
  const runId = p.runId;
  return {
    ...runOptions(endpoint, p),
    breakpoints: p.breakpoints,
    watches: p.watches,
    onPause: (snapshot) => endpoint.request<DebugControlResponse>('pause', { runId, snapshot }),
  };
}
