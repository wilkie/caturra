/**
 * Transport-level tests for the editor↔sandbox RPC endpoint, exercised
 * over a real MessageChannel (Node provides one globally). No browser,
 * no WASM — just the correlation, notify, and teardown logic.
 */
import { afterEach, describe, expect, it } from 'vitest';

import { RpcEndpoint } from '../src/sandbox-rpc.js';

let open: RpcEndpoint[] = [];

function pair(): [RpcEndpoint, RpcEndpoint] {
  const channel = new MessageChannel();
  const a = new RpcEndpoint(channel.port1);
  const b = new RpcEndpoint(channel.port2);
  open.push(a, b);
  return [a, b];
}

afterEach(() => {
  for (const endpoint of open) {
    endpoint.close();
  }
  open = [];
});

describe('RpcEndpoint', () => {
  it('round-trips a request to the peer handler', async () => {
    const [editor, host] = pair();
    host.handle('add', (p: { a: number; b: number }) => p.a + p.b);
    await expect(editor.request<number>('add', { a: 2, b: 3 })).resolves.toBe(5);
  });

  it('awaits async handlers', async () => {
    const [editor, host] = pair();
    host.handle('slow', async () => {
      await Promise.resolve();
      return 'done';
    });
    await expect(editor.request('slow')).resolves.toBe('done');
  });

  it('propagates a handler throw as a rejection with its message', async () => {
    const [editor, host] = pair();
    host.handle('boom', () => {
      throw new Error('kaboom');
    });
    await expect(editor.request('boom')).rejects.toThrow('kaboom');
  });

  it('rejects a request with no registered handler', async () => {
    const [editor] = pair();
    await expect(editor.request('missing')).rejects.toThrow(/no handler for "missing"/);
  });

  it('is symmetric: the host can call back into the editor', async () => {
    const [editor, host] = pair();
    editor.handle('stdin', (p: { runId: number }) => `line for ${String(p.runId)}`);
    await expect(host.request<string>('stdin', { runId: 7 })).resolves.toBe('line for 7');
  });

  it('correlates concurrent in-flight requests independently', async () => {
    const [editor, host] = pair();
    const gates = new Map<number, () => void>();
    host.handle('gated', (p: { n: number }) => {
      return new Promise<number>((resolve) => {
        gates.set(p.n, () => {
          resolve(p.n * 10);
        });
      });
    });
    const first = editor.request<number>('gated', { n: 1 });
    const second = editor.request<number>('gated', { n: 2 });
    await flush();
    // Resolve out of order: second, then first.
    resolveGate(gates, 2);
    resolveGate(gates, 1);
    await expect(Promise.all([first, second])).resolves.toEqual([10, 20]);
  });

  it('delivers one-way notifications without a reply', async () => {
    const [editor, host] = pair();
    const seen: unknown[] = [];
    host.on('log', (params: unknown) => {
      seen.push(params);
    });
    editor.notify('log', { text: 'hello' });
    await flush();
    expect(seen).toEqual([{ text: 'hello' }]);
  });

  it('rejects outstanding requests when closed', async () => {
    const [editor, host] = pair();
    host.handle('never', () => new Promise(() => undefined));
    const pending = editor.request('never');
    editor.close();
    await expect(pending).rejects.toThrow(/channel is closed/);
  });

  it('rejects new requests after close', async () => {
    const [editor] = pair();
    editor.close();
    await expect(editor.request('anything')).rejects.toThrow(/channel is closed/);
  });
});

function resolveGate(gates: Map<number, () => void>, n: number): void {
  const gate = gates.get(n);
  if (!gate) {
    throw new Error(`no gate registered for ${String(n)}`);
  }
  gate();
}

/** Let queued MessagePort deliveries and microtasks drain. */
function flush(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}
