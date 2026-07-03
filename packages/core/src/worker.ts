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
import { consumeInterrupt, readLineBlocking } from './stdin-channel.js';

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

async function handle(request: WorkerRequest): Promise<unknown> {
  switch (request.type) {
    case 'version':
      return jvmVersion();
    case 'compile':
      return (await session()).compile(request.sources);
    case 'run': {
      const { id, stdinBuffer } = request;
      const readStdin = stdinBuffer
        ? () =>
            readLineBlocking(stdinBuffer, () => {
              scope.postMessage({ id, type: 'stdin-request' });
            })
        : () => null;
      return (await session()).run(request.mainClass, {
        args: request.args,
        onStdout: (text) => {
          scope.postMessage({ id, type: 'stdout', text });
        },
        onStderr: (text) => {
          scope.postMessage({ id, type: 'stderr', text });
        },
        readStdin,
      });
    }
    case 'runDebug': {
      const { id, stdinBuffer, debugBuffer, interruptFlag } = request;
      const readStdin = stdinBuffer
        ? () =>
            readLineBlocking(stdinBuffer, () => {
              scope.postMessage({ id, type: 'stdin-request' });
            })
        : () => null;
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
