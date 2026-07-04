import {
  JvmWorkerSession,
  type DebugCommandName,
  type DebugControlResponse,
  type DebugPauseSnapshot,
  type Diagnostic,
} from '@jvmjs/core';
import type { EditorState } from '@codemirror/state';
import {
  breakpointLines,
  breakpointLinesInState,
  createEditor,
  createFileState,
  getSource,
  setPausedLine,
  setSource,
  showDiagnostics,
  toggleBreakpointAtLine,
  type SourceSquiggle,
} from './editor.js';
import { NeighborhoodViz, type NeighborhoodState } from './neighborhood.js';

/** A default 10×10 grid (a wall and a paint bucket) for neighborhood runs. */
function defaultNeighborhoodGrid(): string {
  const size = 10;
  const rows: string[] = [];
  for (let y = 0; y < size; y++) {
    const cells: string[] = [];
    for (let x = 0; x < size; x++) {
      let cell = '1,0';
      if (x === 4 && (y === 4 || y === 5)) {
        cell = '0,0'; // wall
      } else if (x === 7 && y === 2) {
        cell = '1,3'; // paint bucket
      }
      cells.push(cell);
    }
    rows.push(cells.join(' '));
  }
  return `${rows.join('\n')}\n`;
}

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

const fileTabsEl = mustGet('#file-tabs', HTMLDivElement);
const addFileEl = mustGet('#add-file', HTMLButtonElement);
const vizEl = mustGet('#viz', HTMLDivElement);
const neighborhoodViz = new NeighborhoodViz(mustGet('#neighborhood-canvas', HTMLCanvasElement));

const editor = createEditor(sourceEl, DEFAULT_PROGRAM);

// ----- File tabs: one EditorView, one detached state per inactive
// ----- file. Breakpoints and squiggles live inside each state, so
// ----- they travel with their tab. Main.java is fixed.
let activeFile = 'Main.java';
const inactiveFiles = new Map<string, EditorState>();
/** Squiggles per file, applied when a tab becomes active. */
const fileSquiggles = new Map<string, SourceSquiggle[]>();

function allFileNames(): string[] {
  const names = new Set<string>([activeFile, ...inactiveFiles.keys()]);
  return ['Main.java', ...[...names].filter((n) => n !== 'Main.java').sort()];
}

function switchToFile(name: string): void {
  if (name === activeFile || !inactiveFiles.has(name)) {
    return;
  }
  inactiveFiles.set(activeFile, editor.state);
  const next = inactiveFiles.get(name);
  if (!next) {
    return;
  }
  inactiveFiles.delete(name);
  editor.setState(next);
  activeFile = name;
  showDiagnostics(editor, fileSquiggles.get(name) ?? []);
  renderTabs();
}

function addFile(name: string, text = ''): void {
  const fileName = name.endsWith('.java') ? name : `${name}.java`;
  if (fileName === activeFile || inactiveFiles.has(fileName)) {
    switchToFile(fileName);
    return;
  }
  inactiveFiles.set(fileName, createFileState(text));
  renderTabs();
  switchToFile(fileName);
}

function removeFile(name: string): void {
  if (name === 'Main.java') {
    return;
  }
  if (name === activeFile) {
    switchToFile('Main.java');
  }
  inactiveFiles.delete(name);
  fileSquiggles.delete(name);
  renderTabs();
}

function renderTabs(): void {
  for (const tab of [...fileTabsEl.querySelectorAll('.file-tab')]) {
    tab.remove();
  }
  for (const name of allFileNames()) {
    const tab = document.createElement('button');
    tab.className = 'file-tab';
    tab.dataset.file = name;
    if (name === activeFile) {
      tab.classList.add('active');
    }
    tab.textContent = name;
    tab.addEventListener('click', () => {
      switchToFile(name);
    });
    if (name !== 'Main.java') {
      const close = document.createElement('span');
      close.className = 'file-close';
      close.textContent = ' ×';
      close.addEventListener('click', (event) => {
        event.stopPropagation();
        removeFile(name);
      });
      tab.appendChild(close);
    }
    fileTabsEl.insertBefore(tab, addFileEl);
  }
}

addFileEl.addEventListener('click', () => {
  const name = prompt('New file name (e.g. Helper.java)');
  if (name !== null && name.trim() !== '') {
    addFile(name.trim());
  }
});
renderTabs();

/** All files as compiler inputs (the active tab reads the live view). */
function collectSources(): { path: string; text: string }[] {
  const sources = [{ path: activeFile, text: getSource(editor) }];
  for (const [name, state] of inactiveFiles) {
    sources.push({ path: name, text: state.doc.toString() });
  }
  return sources;
}

// Stable automation hooks (Playwright drives the editor through these
// rather than through CodeMirror's contenteditable internals).
declare global {
  interface Window {
    playground: {
      setSource: (text: string) => void;
      getSource: () => string;
      toggleBreakpoint: (line: number) => void;
      breakpointLines: () => number[];
      setFile: (name: string, text: string) => void;
      selectFile: (name: string) => void;
      activeFile: () => string;
      neighborhoodState: () => NeighborhoodState;
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
  setFile: (name, text) => {
    addFile(name, text);
    setSource(editor, text);
  },
  selectFile: (name) => {
    switchToFile(name);
  },
  activeFile: () => activeFile,
  neighborhoodState: () => neighborhoodViz.state(),
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
  fileSquiggles.clear();
  for (const diagnostic of diagnostics) {
    if (!diagnostic.start) {
      continue;
    }
    const squiggle: SourceSquiggle = {
      severity: diagnostic.severity,
      message: diagnostic.message,
      startLine: diagnostic.start.line,
      startColumn: diagnostic.start.column,
      endLine: diagnostic.end?.line,
      endColumn: diagnostic.end?.column,
    };
    const existing = fileSquiggles.get(diagnostic.path) ?? [];
    existing.push(squiggle);
    fileSquiggles.set(diagnostic.path, existing);
  }
  showDiagnostics(editor, fileSquiggles.get(activeFile) ?? []);
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
  const breakpoints = breakpointLines(editor).map((line) => ({ file: activeFile, line }));
  for (const [name, state] of inactiveFiles) {
    for (const line of breakpointLinesInState(state)) {
      breakpoints.push({ file: name, line });
    }
  }
  return breakpoints;
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
    append(`$ javac ${allFileNames().join(' ')}\n`);
    const compiled = await session.compile(collectSources());
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
          const pausedFile = snapshot.frames[0]?.sourceFile;
          if (pausedFile !== undefined && pausedFile !== activeFile) {
            switchToFile(pausedFile);
          }
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

/** Whether the program uses the neighborhood library (drives the canvas). */
function isNeighborhoodProgram(): boolean {
  return collectSources().some((source) => source.text.includes('org.code.neighborhood'));
}

/** After a neighborhood run, read the emitted stream and animate it. */
async function renderNeighborhood(): Promise<void> {
  const session = await sessionReady;
  let messagesText = '';
  try {
    messagesText = await session.readTextFile('neighborhood.jsonl');
  } catch {
    // No painter was created — nothing to animate.
  }
  if (messagesText.trim() === '') {
    vizEl.hidden = true;
    return;
  }
  const gridText = await session.readTextFile('grid.txt');
  vizEl.hidden = false;
  const messages = neighborhoodViz.load(gridText, messagesText);
  neighborhoodViz.play(messages);
}

async function runProgram(): Promise<void> {
  runEl.disabled = true;
  consoleEl.textContent = '';
  neighborhoodViz.stop();
  try {
    const session = await sessionReady;
    const neighborhood = isNeighborhoodProgram();
    if (neighborhood) {
      // Seed the grid and clear any prior run's animation stream.
      await session.writeFile('grid.txt', defaultNeighborhoodGrid());
      await session.remove('neighborhood.jsonl').catch(() => undefined);
    } else {
      vizEl.hidden = true;
    }
    append(`$ javac ${allFileNames().join(' ')}\n`);
    const compiled = await session.compile(collectSources());
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
      if (neighborhood && result.status !== 'error') {
        await renderNeighborhood();
      }
    }
  } catch (error) {
    append(`${String(error)}\n`, 'error');
  } finally {
    runEl.disabled = false;
  }
}

void main();
