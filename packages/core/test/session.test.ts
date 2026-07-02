/**
 * These tests exercise the real WASM engine from Node: the module is
 * instantiated from the wasm-pack output on disk (run `pnpm build:wasm`
 * first), so they cover the full Rust ↔ TypeScript boundary.
 */
import { readFile } from 'node:fs/promises';
import { beforeAll, describe, expect, it } from 'vitest';

import { createJvmSession, initJvm, jvmVersion, type JvmSession } from '../src/index.js';

const HELLO_WORLD = `
public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
`;

beforeAll(async () => {
  const wasmPath = new URL('../src/wasm/generated/jvmjs_bg.wasm', import.meta.url);
  await initJvm(await readFile(wasmPath));
});

describe('engine', () => {
  it('reports its version', async () => {
    await expect(jvmVersion()).resolves.toMatch(/^\d+\.\d+\.\d+$/);
  });
});

describe('virtual filesystem', () => {
  let session: JvmSession;

  beforeAll(async () => {
    session = await createJvmSession();
  });

  it('round-trips text files', () => {
    session.writeFile('/data/input.txt', 'line one\nline two\n');
    expect(session.readTextFile('/data/input.txt')).toBe('line one\nline two\n');
    expect(session.exists('/data/input.txt')).toBe(true);
    expect(session.exists('/data/missing.txt')).toBe(false);
  });

  it('lists directories', () => {
    session.writeFile('/dir/a.txt', 'a');
    session.writeFile('/dir/b.txt', 'b');
    expect(session.listDir('/dir')).toEqual(['/dir/a.txt', '/dir/b.txt']);
  });

  it('creates and removes entries', () => {
    session.mkdir('/scratch');
    expect(session.exists('/scratch')).toBe(true);
    session.remove('/scratch');
    expect(session.exists('/scratch')).toBe(false);
  });

  it('throws a descriptive error for missing files', () => {
    expect(() => session.readFile('/nope.txt')).toThrow(/no such file/);
  });

  it('is isolated between sessions', async () => {
    const other = await createJvmSession();
    expect(other.exists('/data/input.txt')).toBe(false);
  });
});

describe('javac (compile)', () => {
  it('reports lexical errors with positions', async () => {
    const session = await createJvmSession();
    const result = session.compile([
      { path: 'Main.java', text: 'class Main { String s = "unterminated; }' },
    ]);
    expect(result.success).toBe(false);
    const [diagnostic] = result.diagnostics;
    expect(diagnostic?.severity).toBe('error');
    expect(diagnostic?.path).toBe('Main.java');
    expect(diagnostic?.message).toMatch(/unterminated string/);
    expect(diagnostic?.start).toMatchObject({ line: 1, column: 25 });
  });

  it('compiles Hello World', async () => {
    const session = await createJvmSession();
    const result = session.compile([{ path: 'Main.java', text: HELLO_WORLD }]);
    expect(result.diagnostics).toEqual([]);
    expect(result.success).toBe(true);
    expect(result.classNames).toEqual(['Main']);
  });

  it('gives friendly diagnostics for not-yet-supported Java', async () => {
    const session = await createJvmSession();
    const result = session.compile([
      { path: 'Main.java', text: 'class Main { static void f() { int x = 1; } }' },
    ]);
    expect(result.success).toBe(false);
    expect(result.diagnostics[0]?.message).toMatch(
      /local variable declarations are not yet supported/,
    );
  });
});

describe('java (run)', () => {
  it('runs Hello World and streams stdout', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([{ path: 'Main.java', text: HELLO_WORLD }]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Main', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(result.exitCode).toBe(0);
    expect(stdout.join('')).toBe('Hello, World!\n');
  });

  it('routes System.err to the stderr callback', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Errs.java',
        text: 'class Errs { public static void main(String[] args) { System.err.println("oops"); } }',
      },
    ]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const stderr: string[] = [];
    const result = session.run('Errs', {
      onStdout: (text) => stdout.push(text),
      onStderr: (text) => stderr.push(text),
    });
    expect(result.status).toBe('completed');
    expect(stderr.join('')).toBe('oops\n');
    expect(stdout).toEqual([]);
  });

  it('reports an error for a class that was never compiled', async () => {
    const session = await createJvmSession();
    const result = session.run('Main', {});
    expect(result.status).toBe('error');
    expect(result.exitCode).toBe(1);
    expect(result.error).toMatch(/class not found: Main/);
  });
});
