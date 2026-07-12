/**
 * Editor-side tests for RemoteJvmSession, driven over a real
 * MessageChannel against a stub "sandbox host" (a plain RpcEndpoint on
 * the far port). No DOM, no iframe, no WASM — this exercises the mapping
 * from JvmSessionApi calls onto the RPC surface, the stdout/stderr
 * streaming, and the stdin round-trip.
 */
import { afterEach, describe, expect, it } from 'vitest';

import { RemoteJvmSession } from '../src/remote-session.js';
import type { RunResult } from '../src/index.js';
import { RpcEndpoint, type RunCallParams } from '../src/sandbox-rpc.js';

const OK_RUN: RunResult = { status: 'completed', exitCode: 0 };

let cleanups: (() => void)[] = [];

/** A RemoteJvmSession wired to a stub host over an in-memory channel. */
function wired(): { session: RemoteJvmSession; host: RpcEndpoint } {
  const channel = new MessageChannel();
  const host = new RpcEndpoint(channel.port2);
  const session = RemoteJvmSession.overPort(channel.port1);
  cleanups.push(() => {
    session.terminate();
    host.close();
  });
  return { session, host };
}

afterEach(() => {
  for (const cleanup of cleanups) {
    cleanup();
  }
  cleanups = [];
});

describe('RemoteJvmSession', () => {
  it('forwards version and compile to the host', async () => {
    const { session, host } = wired();
    host.handle('version', () => '9.9.9');
    host.handle('compile', () => ({ success: true, classNames: ['Main'], diagnostics: [] }));

    await expect(session.version()).resolves.toBe('9.9.9');
    await expect(session.compile([{ path: 'Main.java', text: 'class Main {}' }])).resolves.toEqual({
      success: true,
      classNames: ['Main'],
      diagnostics: [],
    });
  });

  it('streams stdout back to the run callback and resolves with the run result', async () => {
    const { session, host } = wired();
    host.handle('run', (p: RunCallParams) => {
      host.notify('stdout', { runId: p.runId, text: 'Hello, ' });
      host.notify('stdout', { runId: p.runId, text: 'World!\n' });
      return OK_RUN;
    });

    const chunks: string[] = [];
    const result = await session.run('Main', {
      onStdout: (text) => {
        chunks.push(text);
      },
    });
    expect(chunks.join('')).toBe('Hello, World!\n');
    expect(result).toEqual(OK_RUN);
  });

  it('round-trips stdin: the host asks, the editor answers from the source', async () => {
    const { session, host } = wired();
    const consumed: (string | null)[] = [];
    host.handle('run', async (p: RunCallParams) => {
      // The host wires stdin only when asked to.
      expect(p.wants.stdin).toBe(true);
      consumed.push(await host.request<string | null>('stdin', { runId: p.runId }));
      consumed.push(await host.request<string | null>('stdin', { runId: p.runId }));
      consumed.push(await host.request<string | null>('stdin', { runId: p.runId }));
      return OK_RUN;
    });

    await session.run('Main', { stdin: ['first', 'second'] });
    // Two lines, then EOF (null) once the array is exhausted.
    expect(consumed).toEqual(['first', 'second', null]);
  });

  it('does not flag stdin when the caller supplied no source', async () => {
    const { session, host } = wired();
    let wantsStdin = true;
    host.handle('run', (p: RunCallParams) => {
      wantsStdin = p.wants.stdin;
      return OK_RUN;
    });
    await session.run('Main', {});
    expect(wantsStdin).toBe(false);
  });

  it('round-trips readFile bytes and decodes readTextFile', async () => {
    const { session, host } = wired();
    const bytes = new TextEncoder().encode('line one\n');
    host.handle('readFile', () => bytes);

    await expect(session.readFile('/x.txt')).resolves.toEqual(bytes);
    await expect(session.readTextFile('/x.txt')).resolves.toBe('line one\n');
  });

  it('rejects an in-flight run when terminated', async () => {
    const { session, host } = wired();
    host.handle('run', () => new Promise<RunResult>(() => undefined));
    const running = session.run('Main');
    session.terminate();
    await expect(running).rejects.toThrow(/channel is closed/);
  });
});
