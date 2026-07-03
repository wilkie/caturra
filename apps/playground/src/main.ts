import {
  JvmWorkerSession,
  type DebugCommandName,
  type DebugControlResponse,
  type DebugPauseSnapshot,
  type Diagnostic,
} from '@jvmjs/core';

const DEFAULT_PROGRAM = `public class Main {
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}
`;

function mustGet<T extends Element>(selector: string, type: new () => T): T {
  const element = document.querySelector(selector);
  if (!(element instanceof type)) {
    throw new Error(`playground markup is missing ${selector}`);
  }
  return element;
}

const sourceEl = mustGet('#source', HTMLTextAreaElement);
const stdinEl = mustGet('#stdin', HTMLTextAreaElement);
const consoleEl = mustGet('#console', HTMLPreElement);
const runEl = mustGet('#run', HTMLButtonElement);
const versionEl = mustGet('#engine-version', HTMLSpanElement);
const breakpointsEl = mustGet('#breakpoints', HTMLInputElement);
const debugEl = mustGet('#debug', HTMLButtonElement);
const debugBarEl = mustGet('#debug-bar', HTMLDivElement);
const pausedViewEl = mustGet('#paused-view', HTMLDivElement);
const framesEl = mustGet('#frames', HTMLPreElement);

sourceEl.value = DEFAULT_PROGRAM;

function append(text: string, kind: 'normal' | 'error' = 'normal'): void {
  const span = document.createElement('span');
  span.textContent = text;
  if (kind === 'error') {
    span.className = 'console-error';
  }
  consoleEl.appendChild(span);
}

function formatDiagnostic(diagnostic: Diagnostic): string {
  const location = diagnostic.start
    ? `${diagnostic.path}:${String(diagnostic.start.line)}:${String(diagnostic.start.column)}`
    : diagnostic.path;
  return `${location}: ${diagnostic.severity}: ${diagnostic.message}\n`;
}

// One long-lived engine worker for the page; each Compile & Run
// replaces the compiled classes, like re-invoking javac + java.
const sessionReady = JvmWorkerSession.create();

async function main(): Promise<void> {
  const session = await sessionReady;
  versionEl.textContent = `engine v${await session.version()}`;
  runEl.disabled = false;

  debugEl.disabled = false;
  runEl.addEventListener('click', () => {
    void runProgram();
  });
  debugEl.addEventListener('click', () => {
    void debugProgram();
  });
}

function parseBreakpointLines(): number[] {
  return breakpointsEl.value
    .split(',')
    .map((part) => Number.parseInt(part.trim(), 10))
    .filter((line) => Number.isFinite(line) && line > 0);
}

function renderPause(snapshot: DebugPauseSnapshot): void {
  const lines: string[] = [];
  for (const [index, frame] of snapshot.frames.entries()) {
    const at = frame.line === null ? frame.sourceFile : `${frame.sourceFile}:${String(frame.line)}`;
    lines.push(`${index === 0 ? '→' : ' '} ${frame.className}.${frame.methodName} (${at})`);
    for (const [name, value] of frame.locals) {
      lines.push(`      ${name} = ${value}`);
    }
  }
  framesEl.textContent = lines.join('\n');
  pausedViewEl.hidden = false;
}

/** Wait for one debug-bar click and translate it to a command. */
function nextDebugCommand(): Promise<DebugCommandName> {
  return new Promise((resolve) => {
    const buttons: [string, DebugCommandName][] = [
      ['#resume', 'continue'],
      ['#step-over', 'stepOver'],
      ['#step-into', 'stepInto'],
      ['#step-out', 'stepOut'],
      ['#stop', 'terminate'],
    ];
    const cleanups: (() => void)[] = [];
    for (const [selector, command] of buttons) {
      const button = mustGet(selector, HTMLButtonElement);
      const onClick = (): void => {
        for (const cleanup of cleanups) {
          cleanup();
        }
        resolve(command);
      };
      button.addEventListener('click', onClick);
      cleanups.push(() => {
        button.removeEventListener('click', onClick);
      });
    }
  });
}

async function debugProgram(): Promise<void> {
  runEl.disabled = true;
  debugEl.disabled = true;
  consoleEl.textContent = '';
  try {
    const session = await sessionReady;
    append('$ javac Main.java\n');
    const compiled = await session.compile([{ path: 'Main.java', text: sourceEl.value }]);
    for (const diagnostic of compiled.diagnostics) {
      append(formatDiagnostic(diagnostic), diagnostic.severity === 'error' ? 'error' : 'normal');
    }
    if (compiled.success) {
      append('$ java Main (debugger attached)\n');
      const stdinLines = stdinEl.value === '' ? [] : stdinEl.value.split('\n');
      const result = await session.runDebug('Main', {
        stdin: stdinLines,
        breakpoints: parseBreakpointLines().map((line) => ({ file: 'Main.java', line })),
        onStdout: (text) => {
          append(text);
        },
        onStderr: (text) => {
          append(text, 'error');
        },
        onPause: async (snapshot): Promise<DebugControlResponse> => {
          renderPause(snapshot);
          debugBarEl.hidden = false;
          const command = await nextDebugCommand();
          pausedViewEl.hidden = true;
          return { command };
        },
      });
      debugBarEl.hidden = true;
      pausedViewEl.hidden = true;
      if (result.status === 'error') {
        append(`${result.error ?? 'unknown VM error'}\n`, 'error');
      } else if (result.status === 'stopped') {
        append('(stopped by the debugger)\n');
      } else if (result.status === 'exited') {
        append(`(exit code ${String(result.exitCode)})\n`);
      }
    }
  } catch (error) {
    append(`${String(error)}\n`, 'error');
  } finally {
    runEl.disabled = false;
    debugEl.disabled = false;
  }
}

async function runProgram(): Promise<void> {
  runEl.disabled = true;
  consoleEl.textContent = '';
  try {
    const session = await sessionReady;
    append('$ javac Main.java\n');
    const compiled = await session.compile([{ path: 'Main.java', text: sourceEl.value }]);
    for (const diagnostic of compiled.diagnostics) {
      append(formatDiagnostic(diagnostic), diagnostic.severity === 'error' ? 'error' : 'normal');
    }
    if (compiled.success) {
      append('$ java Main\n');
      const stdinLines = stdinEl.value === '' ? [] : stdinEl.value.split('\n');
      const result = await session.run('Main', {
        stdin: stdinLines,
        onStdout: (text) => {
          append(text);
        },
        onStderr: (text) => {
          append(text, 'error');
        },
      });
      if (result.status === 'error') {
        append(`${result.error ?? 'unknown VM error'}\n`, 'error');
      } else if (result.status === 'exited') {
        append(`(exit code ${String(result.exitCode)})\n`);
      }
    }
  } catch (error) {
    append(`${String(error)}\n`, 'error');
  } finally {
    runEl.disabled = false;
  }
}

void main();
