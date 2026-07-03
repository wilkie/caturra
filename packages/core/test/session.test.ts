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
      {
        path: 'Main.java',
        text: 'class Main { static void f() { String s = new String("hi"); } }',
      },
    ]);
    expect(result.success).toBe(false);
    expect(result.diagnostics[0]?.message).toMatch(/arrives with the class library/);
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

  it('runs a stage-1 program: locals, operators, and concatenation', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Stage1.java',
        text: `
public class Stage1 {
    public static void main(String[] args) {
        int celsius = 25;
        double fahrenheit = celsius * 9 / 5.0 + 32;
        String report = "temp: " + fahrenheit + " F";
        boolean warm = fahrenheit > 70.0 && celsius < 100;
        System.out.println(report);
        System.out.println(warm);
        celsius += 5;
        celsius++;
        System.out.println((char) (64 + celsius / 31));
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Stage1', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('temp: 77.0 F\ntrue\nA\n');
  });

  it('runs a stage-2 program: loops, branches, break/continue', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Stage2.java',
        text: `
public class Stage2 {
    public static void main(String[] args) {
        String beats = "";
        for (int i = 1; i <= 20; i++) {
            if (i % 3 == 0 && i % 5 == 0) { beats += "!"; continue; }
            if (i > 15) break;
            if (i % 2 == 0) beats += "x";
            else beats += ".";
        }
        System.out.println(beats);
        int n = 1, steps = 0;
        while (n != 1 || steps == 0) {
            if (steps == 0) n = 27;
            else if (n % 2 == 0) n /= 2;
            else n = 3 * n + 1;
            steps++;
        }
        System.out.println("collatz(27) steps: " + steps);
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Stage2', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('.x.x.x.x.x.x.x!\ncollatz(27) steps: 112\n');
  });

  it('runs a stage-3 program: methods, recursion, overloads', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Stage3.java',
        text: `
public class Stage3 {
    static int gcd(int a, int b) {
        if (b == 0) return a;
        return gcd(b, a % b);
    }

    static String pretty(int v) { return "int " + v; }
    static String pretty(double v) { return "double " + v; }

    public static void main(String[] args) {
        System.out.println(gcd(1071, 462));
        System.out.println(pretty(21));
        System.out.println(pretty(21.0));
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Stage3', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('21\nint 21\ndouble 21.0\n');
  });

  it('runs a stage-4 program: arrays, 2D, for-each', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Stage4.java',
        text: `
public class Stage4 {
    public static void main(String[] args) {
        int[] data = {4, 8, 15, 16, 23, 42};
        int total = 0;
        for (int v : data) total += v;
        System.out.println("sum: " + total + " of " + data.length);

        int[][] grid = new int[2][3];
        grid[1][2] = 7;
        System.out.println(grid[1][2] + " " + grid[0][0]);

        String[] words = {"lost", "found"};
        words[0] += "!";
        System.out.println(words[0]);
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Stage4', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('sum: 108 of 6\n7 0\nlost!\n');
  });

  it('runs a stage-5 program: classes, objects, toString', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Stage5.java',
        text: `
class Pet {
    private String name;
    private int treats;
    static int adopted = 0;

    Pet(String name) {
        this.name = name;
        adopted++;
    }

    void reward() { treats++; }

    public String toString() { return name + " (" + treats + " treats)"; }
}

public class Stage5 {
    public static void main(String[] args) {
        Pet rex = new Pet("Rex");
        Pet mochi = new Pet("Mochi");
        rex.reward();
        rex.reward();
        mochi.reward();
        System.out.println(rex);
        System.out.println(mochi);
        System.out.println("adopted: " + Pet.adopted);
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Stage5', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('Rex (2 treats)\nMochi (1 treats)\nadopted: 2\n');
  });

  it('runs a stage-6 program: inheritance and polymorphism', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Stage6.java',
        text: `
abstract class Employee {
    private String name;
    Employee(String name) { this.name = name; }
    abstract double pay();
    public String toString() { return name + " earns " + pay(); }
}

class Salaried extends Employee {
    private double annual;
    Salaried(String name, double annual) {
        super(name);
        this.annual = annual;
    }
    double pay() { return annual / 12; }
}

class Hourly extends Employee {
    private double rate;
    private int hours;
    Hourly(String name, double rate, int hours) {
        super(name);
        this.rate = rate;
        this.hours = hours;
    }
    double pay() { return rate * hours; }
}

public class Stage6 {
    public static void main(String[] args) {
        Employee[] staff = { new Salaried("Ada", 120000.0), new Hourly("Alan", 50.0, 80) };
        for (Employee e : staff) {
            System.out.println(e);
            System.out.println(e instanceof Hourly);
        }
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Stage6', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('Ada earns 10000.0\nfalse\nAlan earns 4000.0\ntrue\n');
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
