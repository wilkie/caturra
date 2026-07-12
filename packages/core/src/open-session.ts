/**
 * Pick a session driver at runtime (see `specs/EXECUTION.md`). With a
 * `sandboxOrigin`, the engine runs in a cross-origin iframe on that
 * origin ({@link RemoteJvmSession}), isolating its state from the editor
 * origin's cookies and storage. Without one, it runs in a same-origin
 * Web Worker ({@link JvmWorkerSession}) — the right default for local
 * dev, tests, and single-origin deployments.
 *
 * Both drivers implement {@link JvmSessionApi}, so callers are identical
 * either way.
 */
import { RemoteJvmSession } from './remote-session.js';
import type { JvmSessionApi } from './session-api.js';
import { JvmWorkerSession } from './worker-session.js';

/** Options for {@link openJvmSession}. */
export interface OpenJvmSessionOptions {
  /**
   * Origin of the sandbox host. When set (and non-empty), the engine runs
   * in a hidden iframe on this origin; otherwise it runs in a same-origin
   * worker. Should be a different registrable domain from the editor for
   * storage/cookie isolation to apply.
   */
  sandboxOrigin?: string | undefined;
  /** Path of the runner document on the sandbox origin. Defaults to `/`. */
  runnerPath?: string | undefined;
  /** Where to attach the hidden iframe. Defaults to `document.body`. */
  container?: HTMLElement | undefined;
  /** How long to wait for the sandbox handshake before failing. Defaults to 30s. */
  timeoutMs?: number | undefined;
}

/** Open a JVM session, choosing the iframe or worker driver from {@link OpenJvmSessionOptions}. */
export function openJvmSession(options: OpenJvmSessionOptions = {}): Promise<JvmSessionApi> {
  if (options.sandboxOrigin) {
    return RemoteJvmSession.create({
      sandboxOrigin: options.sandboxOrigin,
      ...(options.runnerPath !== undefined ? { runnerPath: options.runnerPath } : {}),
      ...(options.container !== undefined ? { container: options.container } : {}),
      ...(options.timeoutMs !== undefined ? { timeoutMs: options.timeoutMs } : {}),
    });
  }
  return JvmWorkerSession.create();
}
