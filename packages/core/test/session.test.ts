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
  const wasmPath = new URL('../src/wasm/generated/caturra_bg.wasm', import.meta.url);
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
        text: 'class Main { static boolean f() { return "hi".matches("h."); } }',
      },
    ]);
    expect(result.success).toBe(false);
    expect(result.diagnostics[0]?.message).toMatch(/not supported by caturra/);
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

  it('runs a stage-7 program: classlib with Scanner input', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Stage7.java',
        text: `
import java.util.ArrayList;
import java.util.Scanner;

public class Stage7 {
    public static void main(String[] args) {
        Scanner in = new Scanner(System.in);
        ArrayList<String> shouts = new ArrayList<>();
        int n = in.nextInt();
        for (int i = 0; i < n; i++) {
            shouts.add(in.next().toUpperCase() + "!");
        }
        System.out.println(shouts);
        System.out.println(Math.max(Integer.parseInt("40"), shouts.size()));
        String first = shouts.get(0);
        System.out.println(first.substring(0, first.length() - 1));
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const inputs = ['3 hey ho', "let's-go"];
    let next = 0;
    const result = session.run('Stage7', {
      onStdout: (text) => stdout.push(text),
      readStdin: () => (next < inputs.length ? (inputs[next++] ?? null) : null),
    });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe("[HEY!, HO!, LET'S-GO!]\n40\nHEY\n");
  });

  it('runs a stage-8 program: file IO full circle through the VFS', async () => {
    const session = await createJvmSession();
    // The host seeds an input file...
    session.writeFile('/grades.txt', '90 85 77 100');
    const compiled = session.compile([
      {
        path: 'Curve.java',
        text: `
import java.io.File;
import java.io.PrintWriter;
import java.util.Scanner;

public class Curve {
    public static void main(String[] args) throws Exception {
        Scanner in = new Scanner(new File("/grades.txt"));
        PrintWriter out = new PrintWriter("/curved.txt");
        int count = 0;
        while (in.hasNextInt()) {
            out.println(Math.min(100, in.nextInt() + 5));
            count++;
        }
        out.close();
        System.out.println("curved " + count + " grades");
    }
}
`,
      },
    ]);
    expect(compiled.diagnostics).toEqual([]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Curve', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('curved 4 grades\n');
    // ...and reads Java's output back.
    expect(session.readTextFile('/curved.txt')).toBe('95\n90\n82\n100\n');
  });

  it('handles Java-like recursion depth on the WASM stack', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Deep.java',
        text: `
public class Deep {
    static int sum(int n) {
        if (n == 0) return 0;
        return n + sum(n - 1);
    }

    public static void main(String[] args) {
        System.out.println(sum(3000));
    }
}
`,
      },
    ]);
    expect(compiled.success).toBe(true);

    const stdout: string[] = [];
    const result = session.run('Deep', { onStdout: (text) => stdout.push(text) });
    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('4501500\n');
  });

  it('debugs a program: breakpoints, stepping, and named locals', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Debug.java',
        text: `public class Debug {
    static int twice(int n) {
        int doubled = n * 2;
        return doubled;
    }

    public static void main(String[] args) {
        int sum = 0;
        for (int i = 1; i <= 2; i++) {
            sum += twice(i);
        }
        System.out.println(sum);
    }
}
`,
      },
    ]);
    expect(compiled.success).toBe(true);

    const pauses: import('../src/index.js').DebugPauseSnapshot[] = [];
    const script: import('../src/index.js').DebugCommandName[] = [
      'stepInto', // from line 10 into twice()
      'stepOut', // back to main
      'continue', // to the second breakpoint arrival
      'continue',
    ];
    let step = 0;
    const stdout: string[] = [];
    const result = session.runDebug('Debug', {
      breakpoints: [{ file: 'Debug.java', line: 10 }],
      onStdout: (text) => stdout.push(text),
      onPause: (snapshot) => {
        pauses.push(snapshot);
        return { command: script[step++] ?? 'continue' };
      },
    });

    expect(result.status).toBe('completed');
    expect(stdout.join('')).toBe('6\n');
    expect(pauses.length).toBe(4);

    // First pause: the breakpoint, with loop locals visible.
    expect(pauses[0]?.reason).toBe('breakpoint');
    expect(pauses[0]?.frames[0]?.methodName).toBe('main');
    expect(pauses[0]?.frames[0]?.line).toBe(10);
    const locals0 = new Map(pauses[0]?.frames[0]?.locals.map((l) => [l.name, l.value]));
    expect(locals0.get('i')).toBe('1');
    expect(locals0.get('sum')).toBe('0');

    // Second pause: stepped into twice(), call stack is two deep.
    expect(pauses[1]?.reason).toBe('step');
    expect(pauses[1]?.frames[0]?.methodName).toBe('twice');
    expect(pauses[1]?.frames.length).toBe(2);
    const locals1 = new Map(pauses[1]?.frames[0]?.locals.map((l) => [l.name, l.value]));
    expect(locals1.get('n')).toBe('1');

    // Third pause: stepped out, back in main.
    expect(pauses[2]?.frames[0]?.methodName).toBe('main');
  });

  it('debug terminate stops the program with status stopped', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'Halt.java',
        text: `public class Halt {
    public static void main(String[] args) {
        int x = 1;
        System.out.println(x);
    }
}
`,
      },
    ]);
    expect(compiled.success).toBe(true);
    const stdout: string[] = [];
    const result = session.runDebug('Halt', {
      breakpoints: [{ file: 'Halt.java', line: 3 }],
      onStdout: (text) => stdout.push(text),
      onPause: () => ({ command: 'terminate' }),
    });
    expect(result.status).toBe('stopped');
    expect(stdout.join('')).toBe('');
  });

  it('evaluates watch expressions at pauses, including refresh rounds', async () => {
    const session = await createJvmSession();
    const compiled = session.compile([
      {
        path: 'W.java',
        text: `import java.util.ArrayList;

public class W {
    public static void main(String[] args) {
        ArrayList<String> names = new ArrayList<>();
        int count = 0;
        for (int i = 0; i < 2; i++) {
            names.add("n" + i);
            count++;
        }
        System.out.println(count);
    }
}
`,
      },
    ]);
    expect(compiled.success).toBe(true);

    const rounds: import('../src/index.js').WatchResult[][] = [];
    let addedWatch = false;
    const result = session.runDebug('W', {
      breakpoints: [{ file: 'W.java', line: 9 }], // count++;
      watches: ['names.size()', 'count * 100'],
      onPause: (snapshot) => {
        rounds.push(snapshot.watchResults);
        if (!addedWatch) {
          // Add a watch while paused: refresh re-evaluates in place.
          addedWatch = true;
          return {
            command: 'refresh',
            watches: ['names.size()', 'count * 100', 'names.get(0).toUpperCase()'],
          };
        }
        return { command: 'continue' };
      },
    });
    expect(result.status).toBe('completed');

    // Round 1: initial watches at the first pause.
    expect(rounds[0]?.map((w) => w.value)).toEqual(['1', '0']);
    // Round 2: same pause after refresh, with the added watch live.
    expect(rounds[1]?.map((w) => w.value)).toEqual(['1', '0', 'N0']);
    // Round 3: second loop iteration, values advanced.
    expect(rounds[2]?.map((w) => w.value)).toEqual(['2', '100', 'N0']);

    // Typed locals ride along in the snapshot.
    expect(result.status).toBe('completed');
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
