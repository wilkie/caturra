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
import { TheaterViz } from './theater.js';
import { CSA_UNITS, type CsaLevel, type CsaLevelFile } from './csa-levels.js';

interface TheaterLevel {
  name: string;
  starter: string;
}

// Theater examples: each starter draws with org.code.theater onto the
// 400x400 stage (shapes/text are fully rendered; images are placeholders).
const THEATER_LEVELS: TheaterLevel[] = [
  {
    name: 'Shapes',
    starter: `import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {
    Scene scene = new Scene();
    scene.clear("aqua");

    scene.setFillColor(new Color(120, 190, 255));
    scene.setStrokeColor(new Color(20, 40, 80));
    scene.setStrokeWidth(3.0);
    scene.drawRectangle(40, 60, 320, 180);

    scene.setFillColor(Color.YELLOW);
    scene.drawEllipse(150, 40, 100, 100);

    scene.setStrokeColor(Color.RED);
    scene.setStrokeWidth(4.0);
    scene.drawLine(0, 380, 400, 300);

    scene.setFillColor(Color.GREEN);
    scene.setStrokeColor(new Color(0, 60, 0));
    scene.drawRegularPolygon(300, 320, 6, 50);

    scene.setTextColor("black");
    scene.setTextHeight(32);
    scene.setTextStyle(Font.SANS, FontStyle.BOLD);
    scene.drawText("Hello Theater", 55, 300);

    Theater.playScenes(scene);
  }
}
`,
  },
  {
    name: 'Animation',
    starter: `import org.code.theater.*;
import org.code.media.*;

public class Main {
  public static void main(String[] args) {
    Scene scene = new Scene();
    String[] colors = {"red", "orange", "yellow", "green", "blue", "purple"};

    // Each pause() is one frame: an expanding ring of rainbow color.
    for (int i = 0; i < 18; i++) {
      scene.clear("white");
      scene.setFillColor(new Color(colors[i % colors.length]));
      int r = 20 + i * 10;
      scene.drawEllipse(200 - r, 200 - r, 2 * r, 2 * r);

      scene.setTextColor("black");
      scene.setTextHeight(22);
      scene.setTextStyle(Font.MONO, FontStyle.NORMAL);
      scene.drawText("frame " + i, 150, 30);
      scene.pause(0.15);
    }
    Theater.playScenes(scene);
  }
}
`,
  },
];

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
const testEl = mustGet('#test', HTMLButtonElement);
const stopRunEl = mustGet('#stop-run', HTMLButtonElement);
const testResultsEl = mustGet('#test-results', HTMLUListElement);
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
const unitSelectEl = mustGet('#unit-select', HTMLSelectElement);
const levelSelectEl = mustGet('#level-select', HTMLSelectElement);
const neighborhoodViz = new NeighborhoodViz(mustGet('#neighborhood-canvas', HTMLCanvasElement));
const theaterVizEl = mustGet('#theater-viz', HTMLDivElement);
const theaterLevelEl = mustGet('#theater-level', HTMLSelectElement);
const theaterViz = new TheaterViz(mustGet('#theater-canvas', HTMLCanvasElement));

/** Grid seeded on the next neighborhood run (a picked level's maze). */
let currentNeighborhoodGrid = '';

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

/** Load a level's files into the editor: Main.java plus a tab per class. */
function loadLevelFiles(files: CsaLevelFile[]): void {
  for (const name of [...inactiveFiles.keys()]) {
    removeFile(name);
  }
  switchToFile('Main.java');
  const mainFile = files.find((file) => file.path === 'Main.java');
  // A level with no runnable Main (e.g. an FRQ: classes to implement,
  // checked by the Test button) leaves Main.java empty and opens the
  // first student file instead.
  setSource(editor, mainFile?.text ?? '');
  for (const file of files) {
    if (file.path !== 'Main.java') {
      addFile(file.path, file.text);
    }
  }
  const firstFile = files[0]?.path;
  switchToFile(mainFile || firstFile === undefined ? 'Main.java' : firstFile);
}

// Unit picker → level picker. Choosing a unit populates the level
// dropdown (grouped by lesson); choosing a level loads its files and, for
// neighborhood levels, shows its grid.
let currentUnitLevels: CsaLevel[] = [];
// Teacher-authored validator files for the loaded level; the Test button
// compiles them alongside the student's editor tabs.
let currentValidationFiles: CsaLevelFile[] = [];

for (const [index, unit] of CSA_UNITS.entries()) {
  const option = document.createElement('option');
  option.value = String(index);
  option.textContent = unit.name;
  unitSelectEl.appendChild(option);
}

function populateLevels(levels: CsaLevel[]): void {
  levelSelectEl.textContent = '';
  const placeholder = document.createElement('option');
  placeholder.value = '';
  placeholder.disabled = true;
  placeholder.selected = true;
  placeholder.textContent = 'Level…';
  levelSelectEl.appendChild(placeholder);
  let lastLesson = '';
  let group: HTMLOptGroupElement | null = null;
  for (const [index, level] of levels.entries()) {
    if (level.lesson !== lastLesson) {
      group = document.createElement('optgroup');
      group.label = level.lesson;
      levelSelectEl.appendChild(group);
      lastLesson = level.lesson;
    }
    const option = document.createElement('option');
    option.value = String(index);
    option.textContent = level.name;
    (group ?? levelSelectEl).appendChild(option);
  }
}

unitSelectEl.addEventListener('change', () => {
  const unit = CSA_UNITS[Number(unitSelectEl.value)];
  if (!unit) {
    return;
  }
  currentUnitLevels = unit.levels;
  populateLevels(unit.levels);
});

levelSelectEl.addEventListener('change', () => {
  const level = currentUnitLevels[Number(levelSelectEl.value)];
  if (!level) {
    return;
  }
  loadLevelFiles(level.files);
  currentValidationFiles = level.validationFiles;
  renderTestResults([]);
  theaterVizEl.hidden = true;
  if (level.view === 'neighborhood') {
    currentNeighborhoodGrid = level.grid;
    vizEl.hidden = false;
    neighborhoodViz.load(level.grid, '');
  } else {
    vizEl.hidden = true;
  }
});

// Theater picker: choosing an example loads its program and shows a
// blank stage, ready for Run to draw on it.
for (const [index, level] of THEATER_LEVELS.entries()) {
  const option = document.createElement('option');
  option.value = String(index);
  option.textContent = level.name;
  theaterLevelEl.appendChild(option);
}
theaterLevelEl.addEventListener('change', () => {
  const level = THEATER_LEVELS[Number(theaterLevelEl.value)];
  if (!level) {
    return;
  }
  setSource(editor, level.starter);
  currentValidationFiles = [];
  renderTestResults([]);
  vizEl.hidden = true;
  theaterVizEl.hidden = false;
  theaterViz.reset();
});

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
// replaces the compiled classes, like re-invoking javac + java. `let`,
// not `const`: Stop terminates the worker (the only way to interrupt a
// tight loop) and we respawn a fresh one for the next run.
let sessionReady = JvmWorkerSession.create();

// True between clicking Stop and the terminated run settling, so the
// resulting rejection is reported as "stopped" rather than an error.
let stopRequested = false;

/** Enter the running state: only Stop is actionable. */
function startRun(): void {
  stopRequested = false;
  runEl.disabled = true;
  debugEl.disabled = true;
  testEl.disabled = true;
  stopRunEl.hidden = false;
}

/** Leave the running state, restoring the normal controls. */
function finishRun(): void {
  runEl.disabled = false;
  debugEl.disabled = false;
  testEl.disabled = false;
  stopRunEl.hidden = true;
}

/**
 * Stop a running program. A tight loop won't process a postMessage, so
 * the only reliable interrupt is terminating the worker; we then respawn
 * a fresh session (the next Run recompiles into it).
 */
async function stopProgram(): Promise<void> {
  stopRequested = true;
  stopRunEl.disabled = true;
  const session = await sessionReady;
  session.terminate();
  sessionReady = JvmWorkerSession.create();
  await sessionReady;
  append('\n^C program stopped\n');
  stopRunEl.disabled = false;
  finishRun();
}

async function main(): Promise<void> {
  const session = await sessionReady;
  versionEl.textContent = `engine v${await session.version()}`;
  runEl.disabled = false;

  debugEl.disabled = false;
  testEl.disabled = false;
  runEl.addEventListener('click', () => {
    void runProgram();
  });
  debugEl.addEventListener('click', () => {
    void debugProgram();
  });
  testEl.addEventListener('click', () => {
    void testProgram();
  });
  stopRunEl.addEventListener('click', () => {
    void stopProgram();
  });
}

interface TestResult {
  passed: boolean;
  name: string;
  message: string;
}

/** Parse the runner's `__VTEST\t<PASS|FAIL>\t<name>\t<message>` lines. */
function parseTestResults(output: string): TestResult[] {
  const results: TestResult[] = [];
  for (const line of output.split('\n')) {
    if (!line.startsWith('__VTEST\t')) {
      continue;
    }
    const [, status, name, message] = line.split('\t');
    results.push({
      passed: status === 'PASS',
      name: name ?? '',
      message: message ?? '',
    });
  }
  return results;
}

function renderTestResults(results: TestResult[]): void {
  testResultsEl.textContent = '';
  testResultsEl.hidden = results.length === 0;
  for (const result of results) {
    const item = document.createElement('li');
    item.className = result.passed ? 'test-pass' : 'test-fail';
    const mark = result.passed ? '✓' : '✗';
    const detail = !result.passed && result.message ? ` — ${result.message}` : '';
    item.textContent = `${mark} ${result.name}${detail}`;
    testResultsEl.appendChild(item);
  }
}

/** Compile and run the level's JUnit validator, showing per-test results. */
async function testProgram(): Promise<void> {
  testEl.disabled = true;
  consoleEl.textContent = '';
  renderTestResults([]);
  try {
    const session = await sessionReady;
    // Student files plus the level's teacher-authored validator files.
    const sources = [...collectSources(), ...currentValidationFiles];
    append(`$ javac ${sources.map((source) => source.path).join(' ')}\n`);
    const compiled = await session.compile(sources);
    reportDiagnostics(compiled.diagnostics);
    if (!compiled.success) {
      return;
    }
    if (compiled.validationEntry === undefined) {
      append('(this level has no tests)\n');
      return;
    }
    let output = '';
    const result = await session.run(compiled.validationEntry, {
      onStdout: (text) => {
        output += text;
      },
      onStderr: (text) => {
        append(text, 'error');
      },
    });
    if (result.status === 'error') {
      append(`${result.error ?? 'unknown VM error'}\n`, 'error');
      return;
    }
    const results = parseTestResults(output);
    renderTestResults(results);
    const passed = results.filter((r) => r.passed).length;
    append(`${String(passed)} / ${String(results.length)} tests passed\n`);
  } catch (error) {
    append(`${String(error)}\n`, 'error');
  } finally {
    testEl.disabled = false;
  }
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

/** Whether the program uses the theater library (drives the stage). */
function isTheaterProgram(): boolean {
  return collectSources().some(
    (source) => source.text.includes('org.code.theater') || source.text.includes('org.code.media'),
  );
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

/** After a theater run, read the draw-command log and replay it. */
async function renderTheater(): Promise<void> {
  const session = await sessionReady;
  let log = '';
  try {
    log = await session.readTextFile('theater.log');
  } catch {
    // No scene was played — nothing to draw.
  }
  if (log.trim() === '') {
    theaterVizEl.hidden = true;
    return;
  }
  theaterVizEl.hidden = false;
  void theaterViz.play(log);
}

async function runProgram(): Promise<void> {
  startRun();
  consoleEl.textContent = '';
  neighborhoodViz.stop();
  try {
    const session = await sessionReady;
    const neighborhood = isNeighborhoodProgram();
    const theater = !neighborhood && isTheaterProgram();
    if (neighborhood) {
      // Seed the grid and clear any prior run's animation stream.
      await session.writeFile('grid.txt', currentNeighborhoodGrid);
      await session.remove('neighborhood.jsonl').catch(() => undefined);
      theaterVizEl.hidden = true;
    } else if (theater) {
      await session.remove('theater.log').catch(() => undefined);
      vizEl.hidden = true;
    } else {
      vizEl.hidden = true;
      theaterVizEl.hidden = true;
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
      if (result.status !== 'error') {
        if (neighborhood) {
          await renderNeighborhood();
        } else if (theater) {
          await renderTheater();
        }
      }
    }
  } catch (error) {
    // A terminated worker (Stop) rejects the in-flight run; that path
    // already reset the UI and printed the notice.
    if (!stopRequested) {
      append(`${String(error)}\n`, 'error');
    }
  } finally {
    if (!stopRequested) {
      finishRun();
    }
  }
}

void main();
