/**
 * Step 3 + Step 4 together: a RemoteJvmSession (editor) driving the
 * sandbox host bridge over a real MessageChannel, with a fake engine
 * standing in for JvmWorkerSession. Verifies method forwarding, the
 * stdout stream and stdin round-trip crossing both hops, and the
 * handshake origin gate. No DOM, no worker, no WASM.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { CompileResult, RunResult } from '../src/index.js';
import { RemoteJvmSession } from '../src/remote-session.js';
import {
  isSandboxInit,
  SANDBOX_CHANNEL,
  SANDBOX_PROTOCOL_VERSION,
  type SandboxInitMessage,
} from '../src/sandbox-rpc.js';
import { startSandboxHost, wireSessionToPort, type HandshakeTarget } from '../src/sandbox-host.js';
import type { JvmSessionApi, WorkerRunOptions } from '../src/session-api.js';

const OK_RUN: RunResult = { status: 'completed', exitCode: 0 };
const EDITOR_ORIGIN = 'https://editor.test';

/** A recording stand-in for the real engine, with hooks to drive callbacks. */
class FakeSession implements JvmSessionApi {
  terminated = false;
  lastRunOptions: WorkerRunOptions | undefined;
  onRun: ((options: WorkerRunOptions) => Promise<RunResult>) | undefined;

  version(): Promise<string> {
    return Promise.resolve('fake-1.0');
  }
  compile(): Promise<CompileResult> {
    return Promise.resolve({ success: true, classNames: ['Main'], diagnostics: [] });
  }
  async run(_mainClass: string, options: WorkerRunOptions = {}): Promise<RunResult> {
    this.lastRunOptions = options;
    return this.onRun ? this.onRun(options) : OK_RUN;
  }
  runDebug(): Promise<RunResult> {
    return Promise.resolve(OK_RUN);
  }
  requestPause(): void {
    /* no-op */
  }
  writeFile(): Promise<void> {
    return Promise.resolve();
  }
  readFile(): Promise<Uint8Array> {
    return Promise.resolve(new TextEncoder().encode('bytes'));
  }
  readTextFile(): Promise<string> {
    return Promise.resolve('bytes');
  }
  listDir(): Promise<string[]> {
    return Promise.resolve(['/a']);
  }
  exists(): Promise<boolean> {
    return Promise.resolve(true);
  }
  mkdir(): Promise<void> {
    return Promise.resolve();
  }
  remove(): Promise<void> {
    return Promise.resolve();
  }
  terminate(): void {
    this.terminated = true;
  }
}

let cleanups: (() => void)[] = [];

/** Editor and host wired end to end across a channel, sharing one fake engine. */
function loop(fake: FakeSession = new FakeSession()): {
  editor: RemoteJvmSession;
  fake: FakeSession;
} {
  const channel = new MessageChannel();
  const host = wireSessionToPort(channel.port2, () => Promise.resolve(fake));
  const editor = RemoteJvmSession.overPort(channel.port1);
  cleanups.push(() => {
    editor.terminate();
    host.close();
  });
  return { editor, fake };
}

afterEach(() => {
  for (const cleanup of cleanups) {
    cleanup();
  }
  cleanups = [];
  vi.restoreAllMocks();
});

describe('sandbox host bridge', () => {
  it('forwards method calls to the engine', async () => {
    const { editor } = loop();
    await expect(editor.version()).resolves.toBe('fake-1.0');
    await expect(editor.compile([])).resolves.toEqual({
      success: true,
      classNames: ['Main'],
      diagnostics: [],
    });
    await expect(editor.exists('/a')).resolves.toBe(true);
    await expect(editor.readTextFile('/a')).resolves.toBe('bytes');
  });

  it('streams stdout from the engine to the editor callback', async () => {
    const { editor, fake } = loop();
    fake.onRun = (options) => {
      options.onStdout?.('out!');
      return Promise.resolve(OK_RUN);
    };
    const chunks: string[] = [];
    await editor.run('Main', {
      onStdout: (text) => {
        chunks.push(text);
      },
    });
    expect(chunks).toEqual(['out!']);
  });

  it('round-trips stdin: engine asks, editor answers from its source', async () => {
    const { editor, fake } = loop();
    const seen: (string | null)[] = [];
    fake.onRun = async (options) => {
      // The bridge wires a stdin source only when the editor has one.
      expect(options.stdin).toBeTypeOf('function');
      const next = options.stdin as () => Promise<string | null>;
      seen.push(await next());
      seen.push(await next());
      return OK_RUN;
    };
    await editor.run('Main', { stdin: ['only-line'] });
    expect(seen).toEqual(['only-line', null]);
  });

  it('leaves stdin unwired when the editor supplies no source', async () => {
    const { editor, fake } = loop();
    await editor.run('Main', {});
    expect(fake.lastRunOptions?.stdin).toBeUndefined();
    expect(fake.lastRunOptions?.onStdout).toBeUndefined();
  });

  it('forwards terminate to the engine', async () => {
    const fake = new FakeSession();
    const { editor } = loop(fake);
    // Force lazy creation so terminate has a session to reach.
    await editor.version();
    editor.terminate();
    await flush();
    expect(fake.terminated).toBe(true);
  });
});

describe('startSandboxHost handshake gate', () => {
  function fakeTarget(): HandshakeTarget & { fire: (event: MessageEvent) => void } {
    const listeners = new Set<(event: MessageEvent) => void>();
    return {
      addEventListener: (_type, listener) => listeners.add(listener),
      removeEventListener: (_type, listener) => listeners.delete(listener),
      fire: (event) => {
        for (const listener of [...listeners]) {
          listener(event);
        }
      },
    };
  }

  function initEvent(origin: string, port: MessagePort): MessageEvent {
    const data: SandboxInitMessage = {
      channel: SANDBOX_CHANNEL,
      protocol: SANDBOX_PROTOCOL_VERSION,
    };
    return { data, origin, ports: [port] } as unknown as MessageEvent;
  }

  it('adopts the port and serves requests from an allowed origin', async () => {
    const target = fakeTarget();
    const created = vi.fn(() => Promise.resolve<JvmSessionApi>(new FakeSession()));
    startSandboxHost({ allowedOrigins: [EDITOR_ORIGIN], target, createSession: created });

    const channel = new MessageChannel();
    target.fire(initEvent(EDITOR_ORIGIN, channel.port2));
    const editor = RemoteJvmSession.overPort(channel.port1);
    cleanups.push(() => {
      editor.terminate();
    });

    await expect(editor.version()).resolves.toBe('fake-1.0');
    expect(created).toHaveBeenCalledTimes(1);
  });

  it('ignores a handshake from a disallowed origin', async () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => undefined);
    const target = fakeTarget();
    const created = vi.fn(() => Promise.resolve<JvmSessionApi>(new FakeSession()));
    startSandboxHost({ allowedOrigins: [EDITOR_ORIGIN], target, createSession: created });

    const channel = new MessageChannel();
    target.fire(initEvent('https://evil.test', channel.port2));
    const editor = RemoteJvmSession.overPort(channel.port1);
    cleanups.push(() => {
      editor.terminate();
      channel.port2.close();
    });

    // No handler was wired, so the request never gets a response.
    const outcome = await Promise.race([
      editor.version().then(() => 'answered'),
      flush().then(() => 'silent'),
    ]);
    expect(outcome).toBe('silent');
    expect(created).not.toHaveBeenCalled();
    expect(warn).toHaveBeenCalledWith(expect.stringContaining('disallowed origin'));
  });

  it('recognizes only its own handshake shape', () => {
    expect(isSandboxInit({ channel: SANDBOX_CHANNEL, protocol: 1 })).toBe(true);
    expect(isSandboxInit({ channel: 'something-else' })).toBe(false);
    expect(isSandboxInit(null)).toBe(false);
  });
});

/** Let queued MessagePort deliveries and microtasks drain. */
function flush(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 10));
}
