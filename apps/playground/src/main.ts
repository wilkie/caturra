import {
  JvmWorkerSession,
  type DebugCommandName,
  type DebugControlResponse,
  type DebugPauseSnapshot,
  type Diagnostic,
} from '@jvmjs/core';
import {
  breakpointLines,
  createEditor,
  getSource,
  setPausedLine,
  setSource,
  showDiagnostics,
  toggleBreakpointAtLine,
  type SourceSquiggle,
} from './editor.js';

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

const sourceEl = mustGet('#source', HTMLDivElement);
const stdinEl = mustGet('#stdin', HTMLTextAreaElement);
const consoleEl = mustGet('#console', HTMLPreElement);
const runEl = mustGet('#run', HTMLButtonElement);
const versionEl = mustGet('#engine-version', HTMLSpanElement);
const debugEl = mustGet('#debug', HTMLButtonElement);
const debugBarEl = mustGet('#debug-bar', HTMLDivElement);
const pausedViewEl = mustGet('#paused-view', HTMLDivElement);
const framesEl = mustGet('#frames', HTMLPreElement);
const watchInputEl = mustGet('#watch-input', HTMLInputElement);
const watchAddEl = mustGet('#watch-add', HTMLButtonElement);
const watchesEl = mustGet('#watches', HTMLUListElement);

/** Watch expressions, kept across runs so re-debugging keeps them. */
const watchExpressions: string[] = [];

const editor = createEditor(sourceEl, DEFAULT_PROGRAM);

// Stable automation hooks (Playwright drives the editor through these
// rather than through CodeMirror's contenteditable internals).
declare global {
  interface Window {
    playground: {
      setSource: (text: string) => void;
      getSource: () => string;
      toggleBreakpoint: (line: number) => void;
      breakpointLines: () => number[];
    };
  }
}
window.playground = {
  setSource: (text) => {
    setSource(editor, text);
  },
  getSource: () => getSource(editor),
  toggleBreakpoint: (line) => {
    toggleBreakpointAtLine(editor, line);
  },
  breakpointLines: () => breakpointLines(editor),
};

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

/** Print diagnostics to the console pane and squiggle them in the editor. */
function reportDiagnostics(diagnostics: Diagnostic[]): void {
  for (const diagnostic of diagnostics) {
    append(formatDiagnostic(diagnostic), diagnostic.severity === 'error' ? 'error' : 'normal');
  }
  const squiggles: SourceSquiggle[] = diagnostics
    .filter((diagnostic) => diagnostic.path === 'Main.java' && diagnostic.start)
    .map((diagnostic) => ({
      severity: diagnostic.severity,
      message: diagnostic.message,
      startLine: diagnostic.start?.line ?? 1,
      startColumn: diagnostic.start?.column ?? 1,
      endLine: diagnostic.end?.line,
      endColumn: diagnostic.end?.column,
    }));
  showDiagnostics(editor, squiggles);
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

function currentBreakpoints(): { file: string; line: number }[] {
  return breakpointLines(editor).map((line) => ({ file: 'Main.java', line }));
}

function renderPause(snapshot: DebugPauseSnapshot): void {
  const lines: string[] = [];
  for (const [index, frame] of snapshot.frames.entries()) {
    const at = frame.line === null ? frame.sourceFile : `${frame.sourceFile}:${String(frame.line)}`;
    lines.push(`${index === 0 ? '→' : ' '} ${frame.className}.${frame.methodName} (${at})`);
    for (const local of frame.locals) {
      lines.push(`      ${local.name} = ${local.value}`);
    }
  }
  framesEl.textContent = lines.join('\n');
  renderWatches(snapshot.watchResults);
  pausedViewEl.hidden = false;
}

function renderWatches(results: DebugPauseSnapshot['watchResults']): void {
  watchesEl.textContent = '';
  for (const [index, result] of results.entries()) {
    const item = document.createElement('li');
    const text = document.createElement('span');
    text.textContent =
      result.error != null
        ? `${result.expression} — ${result.error}`
        : `${result.expression} = ${result.value ?? ''}`;
    if (result.error != null) {
      text.className = 'watch-error';
    }
    const remove = document.createElement('button');
    remove.textContent = '×';
    remove.title = 'Remove watch';
    remove.addEventListener('click', () => {
      watchExpressions.splice(index, 1);
      // Wake the pending command wait as a refresh round.
      pendingRefresh?.();
    });
    item.append(text, remove);
    watchesEl.appendChild(item);
  }
}

/** Set while paused: resolves the command wait with a refresh round. */
let pendingRefresh: (() => void) | undefined;

/**
 * Wait for one debug-bar click (a command) or a watch edit (a refresh
 * round that re-evaluates in place).
 */
function nextDebugCommand(): Promise<DebugCommandName | 'refresh'> {
  return new Promise((resolve) => {
    const buttons: [string, DebugCommandName][] = [
      ['#resume', 'continue'],
      ['#step-over', 'stepOver'],
      ['#step-into', 'stepInto'],
      ['#step-out', 'stepOut'],
      ['#stop', 'terminate'],
    ];
    const cleanups: (() => void)[] = [];
    const finish = (command: DebugCommandName | 'refresh'): void => {
      for (const cleanup of cleanups) {
        cleanup();
      }
      pendingRefresh = undefined;
      resolve(command);
    };
    for (const [selector, command] of buttons) {
      const button = mustGet(selector, HTMLButtonElement);
      const onClick = (): void => {
        finish(command);
      };
      button.addEventListener('click', onClick);
      cleanups.push(() => {
        button.removeEventListener('click', onClick);
      });
    }
    const onAdd = (): void => {
      const expression = watchInputEl.value.trim();
      if (expression !== '') {
        watchExpressions.push(expression);
        watchInputEl.value = '';
        finish('refresh');
      }
    };
    watchAddEl.addEventListener('click', onAdd);
    cleanups.push(() => {
      watchAddEl.removeEventListener('click', onAdd);
    });
    pendingRefresh = () => {
      finish('refresh');
    };
  });
}

async function debugProgram(): Promise<void> {
  runEl.disabled = true;
  debugEl.disabled = true;
  consoleEl.textContent = '';
  try {
    const session = await sessionReady;
    append('$ javac Main.java\n');
    const compiled = await session.compile([{ path: 'Main.java', text: getSource(editor) }]);
    reportDiagnostics(compiled.diagnostics);
    if (compiled.success) {
      append('$ java Main (debugger attached)\n');
      const stdinLines = stdinEl.value === '' ? [] : stdinEl.value.split('\n');
      const result = await session.runDebug('Main', {
        stdin: stdinLines,
        breakpoints: currentBreakpoints(),
        watches: [...watchExpressions],
        onStdout: (text) => {
          append(text);
        },
        onStderr: (text) => {
          append(text, 'error');
        },
        onPause: async (snapshot): Promise<DebugControlResponse> => {
          renderPause(snapshot);
          setPausedLine(editor, snapshot.frames[0]?.line ?? null);
          debugBarEl.hidden = false;
          const command = await nextDebugCommand();
          if (command === 'refresh') {
            // Stay paused: replace the watch list and re-evaluate.
            return { command, watches: [...watchExpressions] };
          }
          pausedViewEl.hidden = true;
          setPausedLine(editor, null);
          // Send the gutter's and watch panel's current state along.
          return {
            command,
            breakpoints: currentBreakpoints(),
            watches: [...watchExpressions],
          };
        },
      });
      debugBarEl.hidden = true;
      pausedViewEl.hidden = true;
      setPausedLine(editor, null);
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
    const compiled = await session.compile([{ path: 'Main.java', text: getSource(editor) }]);
    reportDiagnostics(compiled.diagnostics);
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
