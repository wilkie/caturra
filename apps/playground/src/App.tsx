import { useEffect, useRef, useState } from 'react';
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import TextField from '@mui/material/TextField';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Paper from '@mui/material/Paper';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import BugReportIcon from '@mui/icons-material/BugReport';
import ScienceIcon from '@mui/icons-material/Science';
import StopIcon from '@mui/icons-material/Stop';
import AddIcon from '@mui/icons-material/Add';
import CloseIcon from '@mui/icons-material/Close';
import {
  JvmWorkerSession,
  type DebugCommandName,
  type DebugControlResponse,
  type DebugPauseSnapshot,
  type Diagnostic,
} from '@jvmjs/core';
import type { EditorState } from '@codemirror/state';
import type { EditorView } from '@codemirror/view';
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

interface TestResult {
  passed: boolean;
  name: string;
  message: string;
}

interface ConsoleLine {
  text: string;
  kind: 'normal' | 'error';
}

type Phase = 'idle' | 'running' | 'debugging' | 'testing';

/** Parse the runner's `__VTEST\t<PASS|FAIL>\t<name>\t<message>` lines. */
function parseTestResults(output: string): TestResult[] {
  const results: TestResult[] = [];
  for (const line of output.split('\n')) {
    if (!line.startsWith('__VTEST\t')) {
      continue;
    }
    const [, status, name, message] = line.split('\t');
    results.push({ passed: status === 'PASS', name: name ?? '', message: message ?? '' });
  }
  return results;
}

function formatDiagnostic(diagnostic: Diagnostic): string {
  const location = diagnostic.start
    ? `${diagnostic.path}:${String(diagnostic.start.line)}:${String(diagnostic.start.column)}`
    : diagnostic.path;
  return `${location}: ${diagnostic.severity}: ${diagnostic.message}\n`;
}

export function App(): React.JSX.Element {
  // ----- Reactive UI state -----
  const [version, setVersion] = useState('');
  const [ready, setReady] = useState(false);
  const [phase, setPhase] = useState<Phase>('idle');
  const [consoleLines, setConsoleLines] = useState<ConsoleLine[]>([]);
  const [fileNames, setFileNames] = useState<string[]>(['Main.java']);
  const [activeFile, setActiveFile] = useState('Main.java');
  const [unitIndex, setUnitIndex] = useState('');
  const [levelValue, setLevelValue] = useState('');
  const [theaterValue, setTheaterValue] = useState('');
  const [levels, setLevels] = useState<CsaLevel[]>([]);
  const [view, setView] = useState<'none' | 'neighborhood' | 'theater'>('none');
  const [testResults, setTestResults] = useState<TestResult[]>([]);
  const [debugBar, setDebugBar] = useState(false);
  const [paused, setPaused] = useState<DebugPauseSnapshot | null>(null);
  const [stdin, setStdin] = useState('');
  const [watchInput, setWatchInput] = useState('');

  // ----- Imperative state (read inside async handlers via refs) -----
  const sourceElRef = useRef<HTMLDivElement>(null);
  const neighborhoodCanvasRef = useRef<HTMLCanvasElement>(null);
  const theaterCanvasRef = useRef<HTMLCanvasElement>(null);
  const editorRef = useRef<EditorView | null>(null);
  const neighborhoodVizRef = useRef<NeighborhoodViz | null>(null);
  const theaterVizRef = useRef<TheaterViz | null>(null);
  const sessionRef = useRef<Promise<JvmWorkerSession>>(JvmWorkerSession.create());
  const inactiveFilesRef = useRef(new Map<string, EditorState>());
  const fileSquigglesRef = useRef(new Map<string, SourceSquiggle[]>());
  const activeFileRef = useRef('Main.java');
  const currentGridRef = useRef('');
  const validationFilesRef = useRef<CsaLevelFile[]>([]);
  const watchesRef = useRef<string[]>([]);
  const stopRequestedRef = useRef(false);
  const wasStopped = (): boolean => stopRequestedRef.current;
  const debugResolverRef = useRef<((command: DebugCommandName | 'refresh') => void) | null>(null);

  activeFileRef.current = activeFile;

  // ----- Console -----
  const append = (text: string, kind: 'normal' | 'error' = 'normal'): void => {
    setConsoleLines((prev) => [...prev, { text, kind }]);
  };
  const clearConsole = (): void => {
    setConsoleLines([]);
  };

  // ----- File tabs (Main.java fixed; one EditorState per inactive file) -----
  const editor = (): EditorView => {
    if (editorRef.current === null) {
      throw new Error('editor not mounted');
    }
    return editorRef.current;
  };

  const allFileNames = (): string[] => {
    const names = new Set<string>([activeFileRef.current, ...inactiveFilesRef.current.keys()]);
    return ['Main.java', ...[...names].filter((n) => n !== 'Main.java').sort()];
  };
  const refreshTabs = (): void => {
    setFileNames(allFileNames());
  };

  const switchToFile = (name: string): void => {
    if (name === activeFileRef.current || !inactiveFilesRef.current.has(name)) {
      return;
    }
    inactiveFilesRef.current.set(activeFileRef.current, editor().state);
    const next = inactiveFilesRef.current.get(name);
    if (!next) {
      return;
    }
    inactiveFilesRef.current.delete(name);
    editor().setState(next);
    activeFileRef.current = name;
    setActiveFile(name);
    showDiagnostics(editor(), fileSquigglesRef.current.get(name) ?? []);
    refreshTabs();
  };

  const addFile = (name: string, text = ''): void => {
    const fileName = name.endsWith('.java') ? name : `${name}.java`;
    if (fileName === activeFileRef.current || inactiveFilesRef.current.has(fileName)) {
      switchToFile(fileName);
      return;
    }
    inactiveFilesRef.current.set(fileName, createFileState(text));
    refreshTabs();
    switchToFile(fileName);
  };

  const removeFile = (name: string): void => {
    if (name === 'Main.java') {
      return;
    }
    if (name === activeFileRef.current) {
      switchToFile('Main.java');
    }
    inactiveFilesRef.current.delete(name);
    fileSquigglesRef.current.delete(name);
    refreshTabs();
  };

  const loadLevelFiles = (files: CsaLevelFile[]): void => {
    for (const name of [...inactiveFilesRef.current.keys()]) {
      removeFile(name);
    }
    switchToFile('Main.java');
    const mainFile = files.find((file) => file.path === 'Main.java');
    setSource(editor(), mainFile?.text ?? '');
    for (const file of files) {
      if (file.path !== 'Main.java') {
        addFile(file.path, file.text);
      }
    }
    const firstFile = files[0]?.path;
    switchToFile(mainFile || firstFile === undefined ? 'Main.java' : firstFile);
  };

  const collectSources = (): { path: string; text: string }[] => {
    const sources = [{ path: activeFileRef.current, text: getSource(editor()) }];
    for (const [name, state] of inactiveFilesRef.current) {
      sources.push({ path: name, text: state.doc.toString() });
    }
    return sources;
  };

  const reportDiagnostics = (diagnostics: Diagnostic[]): void => {
    for (const diagnostic of diagnostics) {
      append(formatDiagnostic(diagnostic), diagnostic.severity === 'error' ? 'error' : 'normal');
    }
    fileSquigglesRef.current.clear();
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
      const existing = fileSquigglesRef.current.get(diagnostic.path) ?? [];
      existing.push(squiggle);
      fileSquigglesRef.current.set(diagnostic.path, existing);
    }
    showDiagnostics(editor(), fileSquigglesRef.current.get(activeFileRef.current) ?? []);
  };

  // ----- Pickers -----
  const onUnitChange = (value: string): void => {
    setUnitIndex(value);
    const unit = CSA_UNITS[Number(value)];
    setLevels(unit ? unit.levels : []);
    setLevelValue('');
  };

  const onLevelChange = (value: string): void => {
    setLevelValue(value);
    const level = levels[Number(value)];
    if (!level) {
      return;
    }
    loadLevelFiles(level.files);
    validationFilesRef.current = level.validationFiles;
    setTestResults([]);
    if (level.view === 'neighborhood') {
      currentGridRef.current = level.grid;
      setView('neighborhood');
      neighborhoodVizRef.current?.load(level.grid, '');
    } else {
      setView('none');
    }
  };

  const onTheaterChange = (value: string): void => {
    setTheaterValue(value);
    const level = THEATER_LEVELS[Number(value)];
    if (!level) {
      return;
    }
    setSource(editor(), level.starter);
    validationFilesRef.current = [];
    setTestResults([]);
    setView('theater');
    theaterVizRef.current?.reset();
  };

  // ----- Neighborhood / theater rendering -----
  const isNeighborhoodProgram = (): boolean =>
    collectSources().some((source) => source.text.includes('org.code.neighborhood'));
  const isTheaterProgram = (): boolean =>
    collectSources().some(
      (source) =>
        source.text.includes('org.code.theater') || source.text.includes('org.code.media'),
    );

  const renderNeighborhood = async (): Promise<void> => {
    const session = await sessionRef.current;
    let messagesText = '';
    try {
      messagesText = await session.readTextFile('neighborhood.jsonl');
    } catch {
      // No painter was created — nothing to animate.
    }
    if (messagesText.trim() === '') {
      setView('none');
      return;
    }
    const gridText = await session.readTextFile('grid.txt');
    setView('neighborhood');
    const viz = neighborhoodVizRef.current;
    if (viz) {
      const messages = viz.load(gridText, messagesText);
      viz.play(messages);
    }
  };

  const renderTheater = async (): Promise<void> => {
    const session = await sessionRef.current;
    let log = '';
    try {
      log = await session.readTextFile('theater.log');
    } catch {
      // No scene was played — nothing to draw.
    }
    if (log.trim() === '') {
      setView('none');
      return;
    }
    setView('theater');
    void theaterVizRef.current?.play(log);
  };

  // ----- Run / Test / Stop -----
  const runProgram = async (): Promise<void> => {
    stopRequestedRef.current = false;
    setPhase('running');
    clearConsole();
    neighborhoodVizRef.current?.stop();
    try {
      const session = await sessionRef.current;
      const neighborhood = isNeighborhoodProgram();
      const theater = !neighborhood && isTheaterProgram();
      if (neighborhood) {
        await session.writeFile('grid.txt', currentGridRef.current);
        await session.remove('neighborhood.jsonl').catch(() => undefined);
      } else if (theater) {
        await session.remove('theater.log').catch(() => undefined);
        setView('none');
      } else {
        setView('none');
      }
      append(`$ javac ${allFileNames().join(' ')}\n`);
      const compiled = await session.compile(collectSources());
      reportDiagnostics(compiled.diagnostics);
      if (compiled.success) {
        append('$ java Main\n');
        const stdinLines = stdin === '' ? [] : stdin.split('\n');
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
      if (!wasStopped()) {
        append(`${String(error)}\n`, 'error');
      }
    } finally {
      if (!wasStopped()) {
        setPhase('idle');
      }
    }
  };

  const stopProgram = async (): Promise<void> => {
    stopRequestedRef.current = true;
    const session = await sessionRef.current;
    session.terminate();
    sessionRef.current = JvmWorkerSession.create();
    await sessionRef.current;
    append('\n^C program stopped\n');
    setPhase('idle');
  };

  const testProgram = async (): Promise<void> => {
    setPhase('testing');
    clearConsole();
    setTestResults([]);
    try {
      const session = await sessionRef.current;
      const sources = [...collectSources(), ...validationFilesRef.current];
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
      setTestResults(results);
      const passed = results.filter((r) => r.passed).length;
      append(`${String(passed)} / ${String(results.length)} tests passed\n`);
    } catch (error) {
      append(`${String(error)}\n`, 'error');
    } finally {
      setPhase('idle');
    }
  };

  // ----- Debugger -----
  const currentBreakpoints = (): { file: string; line: number }[] => {
    const breakpoints = breakpointLines(editor()).map((line) => ({
      file: activeFileRef.current,
      line,
    }));
    for (const [name, state] of inactiveFilesRef.current) {
      for (const line of breakpointLinesInState(state)) {
        breakpoints.push({ file: name, line });
      }
    }
    return breakpoints;
  };

  const nextDebugCommand = (): Promise<DebugCommandName | 'refresh'> =>
    new Promise((resolve) => {
      debugResolverRef.current = (command) => {
        debugResolverRef.current = null;
        resolve(command);
      };
    });

  const sendDebugCommand = (command: DebugCommandName): void => {
    debugResolverRef.current?.(command);
  };

  const addWatch = (): void => {
    const expression = watchInput.trim();
    if (expression !== '') {
      watchesRef.current = [...watchesRef.current, expression];
      setWatchInput('');
      debugResolverRef.current?.('refresh');
    }
  };

  const removeWatch = (index: number): void => {
    watchesRef.current = watchesRef.current.filter((_, i) => i !== index);
    debugResolverRef.current?.('refresh');
  };

  const debugProgram = async (): Promise<void> => {
    setPhase('debugging');
    clearConsole();
    try {
      const session = await sessionRef.current;
      append(`$ javac ${allFileNames().join(' ')}\n`);
      const compiled = await session.compile(collectSources());
      reportDiagnostics(compiled.diagnostics);
      if (compiled.success) {
        append('$ java Main (debugger attached)\n');
        const stdinLines = stdin === '' ? [] : stdin.split('\n');
        const result = await session.runDebug('Main', {
          stdin: stdinLines,
          breakpoints: currentBreakpoints(),
          watches: [...watchesRef.current],
          onStdout: (text) => {
            append(text);
          },
          onStderr: (text) => {
            append(text, 'error');
          },
          onPause: async (snapshot): Promise<DebugControlResponse> => {
            setPaused(snapshot);
            const pausedFile = snapshot.frames[0]?.sourceFile;
            if (pausedFile !== undefined && pausedFile !== activeFileRef.current) {
              switchToFile(pausedFile);
            }
            setPausedLine(editor(), snapshot.frames[0]?.line ?? null);
            setDebugBar(true);
            const command = await nextDebugCommand();
            if (command === 'refresh') {
              return { command, watches: [...watchesRef.current] };
            }
            setPaused(null);
            setPausedLine(editor(), null);
            return {
              command,
              breakpoints: currentBreakpoints(),
              watches: [...watchesRef.current],
            };
          },
        });
        setDebugBar(false);
        setPaused(null);
        setPausedLine(editor(), null);
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
      setPhase('idle');
    }
  };

  // ----- One-time setup: mount editor, wire hooks, fetch version -----
  useEffect(() => {
    if (sourceElRef.current === null) {
      return;
    }
    const view = createEditor(sourceElRef.current, DEFAULT_PROGRAM);
    editorRef.current = view;
    const nbViz = neighborhoodCanvasRef.current
      ? new NeighborhoodViz(neighborhoodCanvasRef.current)
      : null;
    const thViz = theaterCanvasRef.current ? new TheaterViz(theaterCanvasRef.current) : null;
    neighborhoodVizRef.current = nbViz;
    theaterVizRef.current = thViz;

    window.playground = {
      setSource: (text) => {
        setSource(view, text);
      },
      getSource: () => getSource(view),
      toggleBreakpoint: (line) => {
        toggleBreakpointAtLine(view, line);
      },
      breakpointLines: () => breakpointLines(view),
      setFile: (name, text) => {
        addFile(name, text);
        setSource(view, text);
      },
      selectFile: (name) => {
        switchToFile(name);
      },
      activeFile: () => activeFileRef.current,
      neighborhoodState: (): NeighborhoodState => nbViz?.state() ?? { colors: [], painters: [] },
    };

    void (async () => {
      const session = await sessionRef.current;
      setVersion(`engine v${await session.version()}`);
      setReady(true);
    })();

    return () => {
      view.destroy();
    };
  }, []);

  const runDisabled = !ready || phase === 'running' || phase === 'debugging';
  const debugDisabled = !ready || phase === 'running' || phase === 'debugging';
  const testDisabled = !ready || phase === 'running' || phase === 'testing';

  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100vh' }}>
      <AppBar position="static" color="default" elevation={1}>
        <Toolbar variant="dense" sx={{ gap: 2 }}>
          <Typography variant="h6" component="h1" sx={{ fontWeight: 700 }}>
            jvmjs playground
          </Typography>
          <Typography
            variant="body2"
            color="text.secondary"
            data-testid="engine-version"
            id="engine-version"
          >
            {version}
          </Typography>
        </Toolbar>
      </AppBar>

      <Box
        component="main"
        sx={{ display: 'flex', flex: 1, minHeight: 0, gap: 1, p: 1, alignItems: 'stretch' }}
      >
        {/* Editor pane */}
        <Paper
          className="editor-pane"
          variant="outlined"
          sx={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, p: 1, gap: 1 }}
        >
          <Stack
            direction="row"
            spacing={0.5}
            data-testid="file-tabs"
            id="file-tabs"
            sx={{ flexWrap: 'wrap', alignItems: 'center' }}
          >
            {fileNames.map((name) => (
              <Button
                key={name}
                data-file={name}
                size="small"
                variant={name === activeFile ? 'contained' : 'text'}
                onClick={() => {
                  switchToFile(name);
                }}
                sx={{ textTransform: 'none', minWidth: 0, py: 0.25 }}
                endIcon={
                  name !== 'Main.java' ? (
                    <CloseIcon
                      fontSize="inherit"
                      onClick={(event) => {
                        event.stopPropagation();
                        removeFile(name);
                      }}
                    />
                  ) : undefined
                }
              >
                {name}
              </Button>
            ))}
            <IconButton
              id="add-file"
              data-testid="add-file"
              size="small"
              title="Add a file"
              onClick={() => {
                const name = prompt('New file name (e.g. Helper.java)');
                if (name !== null && name.trim() !== '') {
                  addFile(name.trim());
                }
              }}
            >
              <AddIcon fontSize="small" />
            </IconButton>
          </Stack>

          <Box id="source" data-testid="source" ref={sourceElRef} sx={{ flex: 1, minHeight: 0 }} />
          <Typography variant="caption" color="text.secondary" className="hint">
            Click a line number to set a breakpoint.
          </Typography>

          <TextField
            label="Program input (one line per Scanner read)"
            value={stdin}
            onChange={(event) => {
              setStdin(event.target.value);
            }}
            multiline
            minRows={3}
            maxRows={3}
            size="small"
            slotProps={{ htmlInput: { spellCheck: false, id: 'stdin', 'data-testid': 'stdin' } }}
          />

          <Box className="run-row" sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
            <Button
              id="run"
              data-testid="run"
              variant="contained"
              startIcon={<PlayArrowIcon />}
              disabled={runDisabled}
              onClick={() => void runProgram()}
            >
              Compile &amp; Run
            </Button>
            <Button
              id="debug"
              data-testid="debug"
              variant="outlined"
              startIcon={<BugReportIcon />}
              disabled={debugDisabled}
              onClick={() => void debugProgram()}
            >
              Debug
            </Button>
            <Button
              id="test"
              data-testid="test"
              variant="outlined"
              startIcon={<ScienceIcon />}
              disabled={testDisabled}
              title="Run the level's tests"
              onClick={() => void testProgram()}
            >
              Test
            </Button>
            {phase === 'running' && (
              <Button
                id="stop-run"
                data-testid="stop-run"
                variant="outlined"
                color="error"
                startIcon={<StopIcon />}
                title="Stop the running program"
                onClick={() => void stopProgram()}
              >
                Stop
              </Button>
            )}
            <PickerSelect
              id="unit-select"
              label="Choose a CSA unit"
              placeholder="Unit…"
              value={unitIndex}
              onChange={onUnitChange}
            >
              {CSA_UNITS.map((unit, index) => (
                <option key={unit.name} value={index}>
                  {unit.name}
                </option>
              ))}
            </PickerSelect>
            <PickerSelect
              id="level-select"
              label="Load a level"
              placeholder="Level…"
              value={levelValue}
              onChange={onLevelChange}
            >
              {groupLevels(levels)}
            </PickerSelect>
            <PickerSelect
              id="theater-level"
              label="Load a Theater example"
              placeholder="Theater…"
              value={theaterValue}
              onChange={onTheaterChange}
            >
              {THEATER_LEVELS.map((level, index) => (
                <option key={level.name} value={index}>
                  {level.name}
                </option>
              ))}
            </PickerSelect>
          </Box>
        </Paper>

        {/* Console pane */}
        <Paper
          className="console-pane"
          variant="outlined"
          sx={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0, p: 1, gap: 1 }}
        >
          <Box id="debug-bar" data-testid="debug-bar" hidden={!debugBar}>
            <Button
              id="resume"
              data-testid="resume"
              size="small"
              onClick={() => {
                sendDebugCommand('continue');
              }}
            >
              Continue
            </Button>
            <Button
              id="step-over"
              data-testid="step-over"
              size="small"
              onClick={() => {
                sendDebugCommand('stepOver');
              }}
            >
              Step Over
            </Button>
            <Button
              id="step-into"
              data-testid="step-into"
              size="small"
              onClick={() => {
                sendDebugCommand('stepInto');
              }}
            >
              Step Into
            </Button>
            <Button
              id="step-out"
              data-testid="step-out"
              size="small"
              onClick={() => {
                sendDebugCommand('stepOut');
              }}
            >
              Step Out
            </Button>
            <Button
              id="stop"
              data-testid="stop"
              size="small"
              color="error"
              onClick={() => {
                sendDebugCommand('terminate');
              }}
            >
              Stop
            </Button>
          </Box>

          <Box id="paused-view" data-testid="paused-view" hidden={paused === null}>
            <Typography variant="subtitle2">Paused</Typography>
            <Box component="pre" id="frames" data-testid="frames" sx={{ m: 0 }}>
              {paused ? renderFrames(paused) : ''}
            </Box>
            <Typography variant="subtitle2">Watches</Typography>
            <Box className="watch-row" sx={{ display: 'flex', gap: 1, my: 0.5 }}>
              <TextField
                size="small"
                placeholder="e.g. total * 2"
                value={watchInput}
                onChange={(event) => {
                  setWatchInput(event.target.value);
                }}
                slotProps={{ htmlInput: { id: 'watch-input', 'data-testid': 'watch-input' } }}
              />
              <Button id="watch-add" data-testid="watch-add" size="small" onClick={addWatch}>
                Add
              </Button>
            </Box>
            <Box
              component="ul"
              id="watches"
              data-testid="watches"
              sx={{ listStyle: 'none', pl: 0, m: 0 }}
            >
              {(paused?.watchResults ?? []).map((result, index) => (
                <li key={`${result.expression}-${String(index)}`}>
                  <span className={result.error != null ? 'watch-error' : undefined}>
                    {result.error != null
                      ? `${result.expression} — ${result.error}`
                      : `${result.expression} = ${result.value ?? ''}`}
                  </span>
                  <button
                    title="Remove watch"
                    onClick={() => {
                      removeWatch(index);
                    }}
                  >
                    ×
                  </button>
                </li>
              ))}
            </Box>
          </Box>

          <Box id="viz" data-testid="viz" hidden={view !== 'neighborhood'}>
            <Typography variant="subtitle2">Neighborhood</Typography>
            <canvas
              id="neighborhood-canvas"
              data-testid="neighborhood-canvas"
              ref={neighborhoodCanvasRef}
            />
          </Box>
          <Box id="theater-viz" data-testid="theater-viz" hidden={view !== 'theater'}>
            <Typography variant="subtitle2">Theater</Typography>
            <canvas id="theater-canvas" data-testid="theater-canvas" ref={theaterCanvasRef} />
          </Box>

          <Box
            component="ul"
            id="test-results"
            data-testid="test-results"
            hidden={testResults.length === 0}
            sx={{ listStyle: 'none', pl: 0, m: 0, fontFamily: 'monospace' }}
          >
            {testResults.map((result, index) => (
              <li
                key={`${result.name}-${String(index)}`}
                className={result.passed ? 'test-pass' : 'test-fail'}
              >
                {`${result.passed ? '✓' : '✗'} ${result.name}${
                  !result.passed && result.message ? ` — ${result.message}` : ''
                }`}
              </li>
            ))}
          </Box>

          <Typography variant="subtitle2">Console</Typography>
          <Box
            component="pre"
            id="console"
            data-testid="console"
            sx={{ flex: 1, minHeight: 0, overflow: 'auto', m: 0 }}
          >
            {consoleLines.map((line, index) => (
              <span key={index} className={line.kind === 'error' ? 'console-error' : undefined}>
                {line.text}
              </span>
            ))}
          </Box>
        </Paper>
      </Box>
    </Box>
  );
}

function renderFrames(snapshot: DebugPauseSnapshot): string {
  const lines: string[] = [];
  for (const [index, frame] of snapshot.frames.entries()) {
    const at = frame.line === null ? frame.sourceFile : `${frame.sourceFile}:${String(frame.line)}`;
    lines.push(`${index === 0 ? '→' : ' '} ${frame.className}.${frame.methodName} (${at})`);
    for (const local of frame.locals) {
      lines.push(`      ${local.name} = ${local.value}`);
    }
  }
  return lines.join('\n');
}

function groupLevels(levels: CsaLevel[]): React.JSX.Element[] {
  const groups: { lesson: string; items: { index: number; name: string }[] }[] = [];
  for (const [index, level] of levels.entries()) {
    let group = groups.at(-1);
    if (group?.lesson !== level.lesson) {
      group = { lesson: level.lesson, items: [] };
      groups.push(group);
    }
    group.items.push({ index, name: level.name });
  }
  return groups.map((group) => (
    <optgroup key={group.lesson} label={group.lesson}>
      {group.items.map((item) => (
        <option key={item.index} value={item.index}>
          {item.name}
        </option>
      ))}
    </optgroup>
  ));
}

interface PickerSelectProps {
  id: string;
  label: string;
  placeholder: string;
  value: string;
  onChange: (value: string) => void;
  children: React.ReactNode;
}

/** A native `<select>` (so Playwright's selectOption works) with MUI styling. */
function PickerSelect({
  id,
  label,
  placeholder,
  value,
  onChange,
  children,
}: PickerSelectProps): React.JSX.Element {
  return (
    <TextField
      select
      size="small"
      label={placeholder}
      value={value}
      onChange={(event) => {
        onChange(event.target.value);
      }}
      slotProps={{
        select: { native: true, inputProps: { id, 'data-testid': id, 'aria-label': label } },
        inputLabel: { shrink: true },
      }}
      sx={{ minWidth: 130 }}
    >
      <option value="" disabled>
        {placeholder}
      </option>
      {children}
    </TextField>
  );
}
