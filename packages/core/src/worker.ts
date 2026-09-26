/**
 * The engine worker: owns the WASM module and one synchronous
 * {@link JvmSession}, and services requests from
 * {@link JvmWorkerSession} on the main thread.
 *
 * Runs as a module worker; spawned by `worker-session.ts` via
 * `new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' })`.
 */
import { initJvm, JvmSession, jvmVersion } from './index.js';
import type { DebugControlResponse } from './index.js';
import type { WorkerRequest, WorkerResponse } from './protocol.js';
import { consumeInterrupt, pollLineBlocking, readLineBlocking } from './stdin-channel.js';

/**
 * Minimal typing for the dedicated-worker global scope. (The full
 * `webworker` TS lib conflicts with `dom`, which the rest of the
 * package needs; these two members are all the worker uses.)
 */
interface WorkerScope {
  postMessage(message: WorkerResponse): void;
  addEventListener(type: 'message', listener: (event: MessageEvent<WorkerRequest>) => void): void;
}

const scope = globalThis as unknown as WorkerScope;

let sessionPromise: Promise<JvmSession> | undefined;

async function session(): Promise<JvmSession> {
  sessionPromise ??= (async () => {
    await initJvm();
    return JvmSession.create();
  })();
  return sessionPromise;
}

/**
 * The Swing event-dispatch thread's wait (specs/CONCURRENCY.md, phase 3): the
 * engine asks for the next event with a timeout, often, while other Java
 * threads run. The tree is posted for the main thread to render only when it
 * CHANGED since the render the main thread is still answering — an unchanged
 * look costs nothing but the wait — and after an event is taken, the next
 * wait always posts, since that answer is spent.
 */
function swingEventPump(
  swingBuffer: SharedArrayBuffer,
  id: number,
): (tree: string, timeoutMs: number) => string | null | undefined {
  let posted: string | null = null;
  return (tree, timeoutMs) => {
    const request =
      tree === posted
        ? null
        : () => {
            posted = tree;
            scope.postMessage({ id, type: 'swing-render', tree });
          };
    const event = pollLineBlocking(swingBuffer, request, timeoutMs);
    if (event !== undefined) {
      posted = null;
    }
    return event;
  };
}

async function handle(request: WorkerRequest): Promise<unknown> {
  switch (request.type) {
    case 'version':
      return jvmVersion();
    case 'compile':
      return (await session()).compile(request.sources);
    case 'run': {
      const { id, stdinBuffer, swingBuffer } = request;
      const readStdin = stdinBuffer
        ? () =>
            readLineBlocking(stdinBuffer, () => {
              scope.postMessage({ id, type: 'stdin-request' });
            })
        : () => null;
      // Swing event pump: post the current tree for the main thread to
      // render, then park on the shared channel until it supplies the next
      // event or the time runs out (same blocking pattern as stdin/debug).
      const awaitUiEvent = swingBuffer ? swingEventPump(swingBuffer, id) : undefined;
      // Blocking JOptionPane dialog: same channel (dialogs and events never
      // overlap — the loop isn't parked while a listener shows a dialog).
      const showDialog = swingBuffer
        ? (kind: string, message: string) =>
            readLineBlocking(swingBuffer, () => {
              scope.postMessage({ id, type: 'swing-dialog', kind, message });
            })
        : undefined;
      return (await session()).run(request.mainClass, {
        args: request.args,
        onStdout: (text) => {
          scope.postMessage({ id, type: 'stdout', text });
        },
        onStderr: (text) => {
          scope.postMessage({ id, type: 'stderr', text });
        },
        readStdin,
        ...(awaitUiEvent ? { awaitUiEvent } : {}),
        ...(showDialog ? { showDialog } : {}),
      });
    }
    case 'runDebug': {
      const { id, stdinBuffer, debugBuffer, interruptFlag, swingBuffer } = request;
      const readStdin = stdinBuffer
        ? () =>
            readLineBlocking(stdinBuffer, () => {
              scope.postMessage({ id, type: 'stdin-request' });
            })
        : () => null;
      // Interactive Swing under the debugger: same event pump as `run`, so
      // a listener runs (and can hit a breakpoint via onPause below).
      const awaitUiEvent = swingBuffer ? swingEventPump(swingBuffer, id) : undefined;
      const showDialog = swingBuffer
        ? (kind: string, message: string) =>
            readLineBlocking(swingBuffer, () => {
              scope.postMessage({ id, type: 'swing-dialog', kind, message });
            })
        : undefined;
      return (await session()).runDebug(request.mainClass, {
        args: request.args,
        breakpoints: request.breakpoints,
        watches: request.watches,
        onStdout: (text) => {
          scope.postMessage({ id, type: 'stdout', text });
        },
        onStderr: (text) => {
          scope.postMessage({ id, type: 'stderr', text });
        },
        readStdin,
        onPause: (snapshot) => {
          // Park on the shared channel until the main thread answers
          // with a command (same blocking pattern as stdin).
          const response = readLineBlocking(debugBuffer, () => {
            scope.postMessage({ id, type: 'debug-paused', snapshot });
          });
          return response === null
            ? { command: 'terminate' }
            : (JSON.parse(response) as DebugControlResponse);
        },
        pollInterrupt: () => consumeInterrupt(interruptFlag),
        ...(awaitUiEvent ? { awaitUiEvent } : {}),
        ...(showDialog ? { showDialog } : {}),
      });
    }
    case 'writeFile':
      (await session()).writeFile(request.path, request.contents);
      return undefined;
    case 'readFile':
      return (await session()).readFile(request.path);
    case 'listDir':
      return (await session()).listDir(request.path);
    case 'exists':
      return (await session()).exists(request.path);
    case 'mkdir':
      (await session()).mkdir(request.path);
      return undefined;
    case 'remove':
      (await session()).remove(request.path);
      return undefined;
  }
}

scope.addEventListener('message', (event) => {
  const request = event.data;
  void handle(request).then(
    (value) => {
      scope.postMessage({ id: request.id, type: 'result', value });
    },
    (error: unknown) => {
      scope.postMessage({ id: request.id, type: 'error', message: String(error) });
    },
  );
});
