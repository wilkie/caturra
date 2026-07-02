import { JvmWorkerSession, type Diagnostic } from '@jvmjs/core';

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

  runEl.addEventListener('click', () => {
    void runProgram();
  });
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
