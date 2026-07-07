/**
 * Message protocol between {@link JvmWorkerSession} (main thread) and
 * the engine worker. See `specs/EXECUTION.md`.
 *
 * Every request carries an `id`; the worker answers with exactly one
 * `result` or `error` carrying the same id, preceded by any number of
 * `stdout` / `stderr` / `stdin-request` events for `run` requests.
 * Worker message delivery is ordered, so output events always arrive
 * before the run's result.
 */
import type {
  CompileResult,
  DebugBreakpoint,
  DebugPauseSnapshot,
  JavaSourceFile,
  RunResult,
} from './index.js';

export type WorkerRequest =
  | { id: number; type: 'version' }
  | { id: number; type: 'compile'; sources: JavaSourceFile[] }
  | {
      id: number;
      type: 'run';
      mainClass: string;
      args: string[];
      /** Present when the page is cross-origin isolated; carries stdin. */
      stdinBuffer?: SharedArrayBuffer;
      /** Blocking Swing event channel: the worker parks here between UI events. */
      swingBuffer?: SharedArrayBuffer;
    }
  | {
      id: number;
      type: 'runDebug';
      mainClass: string;
      args: string[];
      breakpoints: DebugBreakpoint[];
      watches: string[];
      /** Blocking command channel: the worker parks here while paused. */
      debugBuffer: SharedArrayBuffer;
      /** Pause-button flag, polled by the engine between instructions. */
      interruptFlag: SharedArrayBuffer;
      stdinBuffer?: SharedArrayBuffer;
    }
  | { id: number; type: 'writeFile'; path: string; contents: string | Uint8Array }
  | { id: number; type: 'readFile'; path: string }
  | { id: number; type: 'listDir'; path: string }
  | { id: number; type: 'exists'; path: string }
  | { id: number; type: 'mkdir'; path: string }
  | { id: number; type: 'remove'; path: string };

export interface ResultValueByType {
  version: string;
  compile: CompileResult;
  run: RunResult;
  runDebug: RunResult;
  writeFile: undefined;
  readFile: Uint8Array;
  listDir: string[];
  exists: boolean;
  mkdir: undefined;
  remove: undefined;
}

export type WorkerResponse =
  | { id: number; type: 'result'; value: unknown }
  | { id: number; type: 'error'; message: string }
  | { id: number; type: 'stdout'; text: string }
  | { id: number; type: 'stderr'; text: string }
  | { id: number; type: 'stdin-request' }
  | { id: number; type: 'debug-paused'; snapshot: DebugPauseSnapshot }
  | { id: number; type: 'swing-render'; tree: string };
