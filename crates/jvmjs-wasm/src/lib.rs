//! The WebAssembly boundary.
//!
//! Everything JavaScript can see lives here: a [`JvmSession`] that owns a
//! virtual filesystem and a set of compiled classes, with methods that
//! mirror the CLI tools they replace — [`JvmSession::compile`] is
//! `javac`, [`JvmSession::run`] is `java`.
//!
//! Data crosses the boundary as serde-serialized plain objects (via
//! `serde-wasm-bindgen`), and console IO crosses as JavaScript callbacks
//! so the host page can stream output as it happens.

use jvmjs_compiler::{Severity, SourceFile};
use jvmjs_vm::{ConsoleIo, VirtualFileSystem, Vm, VmOptions};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Mirrors [`jvmjs_compiler::SourceFile`] across the JS boundary.
#[derive(Debug, Deserialize)]
struct JsSourceFile {
    path: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct JsPosition {
    line: u32,
    column: u32,
}

#[derive(Debug, Serialize)]
struct JsDiagnostic {
    severity: &'static str,
    message: String,
    path: String,
    start: Option<JsPosition>,
    end: Option<JsPosition>,
}

#[derive(Debug, Serialize)]
struct JsCompileResult {
    success: bool,
    /// Binary names of the classes produced, e.g. `["Main"]`.
    #[serde(rename = "classNames")]
    class_names: Vec<String>,
    diagnostics: Vec<JsDiagnostic>,
    /// Entry class for a validation ("Test") run, if `JUnit` tests were found.
    #[serde(rename = "validationEntry")]
    validation_entry: Option<String>,
}

/// A debugger host backed by JS callbacks; `on_pause` blocks the
/// worker until the main thread supplies a command.
struct JsDebugHost<'js> {
    on_pause: &'js js_sys::Function,
    poll_interrupt: Option<&'js js_sys::Function>,
    /// Active watch expressions, replaceable from pause responses.
    watches: Vec<String>,
    /// The program's source files, needed to compile watch classes
    /// against the user's own classes.
    sources: &'js [SourceFile],
}

impl JsDebugHost<'_> {
    /// Compile and run every active watch against the paused frame.
    fn evaluate_watches(
        &self,
        snapshot: &jvmjs_vm::DebugSnapshot,
        watch: &mut dyn jvmjs_vm::WatchEvaluator,
    ) -> Vec<(String, Result<String, String>)> {
        self.watches
            .iter()
            .map(|expression| {
                (
                    expression.clone(),
                    evaluate_watch(expression, snapshot, self.sources, watch),
                )
            })
            .collect()
    }
}

/// Synthesize, compile, and evaluate one watch expression in the
/// innermost paused frame.
fn evaluate_watch(
    expression: &str,
    snapshot: &jvmjs_vm::DebugSnapshot,
    sources: &[SourceFile],
    watch: &mut dyn jvmjs_vm::WatchEvaluator,
) -> Result<String, String> {
    let frame = snapshot
        .frames
        .first()
        .ok_or_else(|| String::from("no paused frame"))?;

    // Parameters mirror the frame's locals by name; `this` is a
    // keyword and locals without a source form can't be referenced.
    let usable: Vec<&jvmjs_vm::LocalSnapshot> = frame
        .locals
        .iter()
        .filter(|l| l.name != "this" && !l.type_name.is_empty())
        .collect();
    let params = usable
        .iter()
        .map(|l| format!("{} {}", l.type_name, l.name))
        .collect::<Vec<_>>()
        .join(", ");
    let param_names: Vec<String> = usable.iter().map(|l| l.name.clone()).collect();

    // `"" + (expr)` sidesteps not knowing the expression's type: the
    // compiler's concat machinery renders every printable type.
    let watch_source = format!(
        "class __JvmjsWatch {{\n    static String __eval({params}) \
         {{\n        return \"\" + ({expression});\n    }}\n}}\n"
    );
    let mut all = sources.to_vec();
    all.push(SourceFile {
        path: String::from("__JvmjsWatch.java"),
        text: watch_source,
    });

    let compilation = jvmjs_compiler::compile(&all);
    if !compilation.success() {
        // Report the first diagnostic in the watch itself (javac
        // wording — same messages the editor shows).
        let message = compilation
            .diagnostics
            .iter()
            .find(|d| d.path == "__JvmjsWatch.java")
            .map_or_else(
                || String::from("the watch expression does not compile"),
                |d| d.message.clone(),
            );
        return Err(message);
    }
    let class = compilation
        .classes
        .iter()
        .find(|c| c.binary_name == "__JvmjsWatch")
        .ok_or_else(|| String::from("watch class missing from compilation"))?;

    watch.evaluate(&class.class_file, "__eval", &param_names)
}

impl jvmjs_vm::DebugHost for JsDebugHost<'_> {
    fn on_pause(
        &mut self,
        snapshot: &jvmjs_vm::DebugSnapshot,
        watch: &mut dyn jvmjs_vm::WatchEvaluator,
    ) -> jvmjs_vm::DebugControl {
        // Evaluate the active watch list, then keep the host informed
        // through as many refresh rounds as it asks for (each round
        // may replace the watch list — e.g. the user added one while
        // paused — and wants fresh values back).
        loop {
            let results = self.evaluate_watches(snapshot, watch);
            let payload = snapshot_to_json(snapshot, &results);
            let response = self
                .on_pause
                .call1(&JsValue::NULL, &JsValue::from_str(&payload))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let (control, refresh) = parse_debug_control(&response);
            if let Some(watches) = control.watches {
                self.watches = watches;
            }
            if !refresh {
                return control.control;
            }
        }
    }

    fn interrupt_requested(&mut self) -> bool {
        self.poll_interrupt
            .and_then(|f| f.call0(&JsValue::NULL).ok())
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

#[derive(Serialize)]
struct JsLocal<'s> {
    name: &'s str,
    #[serde(rename = "type")]
    type_name: &'s str,
    value: &'s str,
}

#[derive(Serialize)]
struct JsSnapshotFrame<'s> {
    #[serde(rename = "className")]
    class_name: &'s str,
    #[serde(rename = "methodName")]
    method_name: &'s str,
    #[serde(rename = "sourceFile")]
    source_file: &'s str,
    line: Option<u32>,
    locals: Vec<JsLocal<'s>>,
}

#[derive(Serialize)]
struct JsWatchResult<'s> {
    expression: &'s str,
    value: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct JsSnapshot<'s> {
    reason: &'static str,
    frames: Vec<JsSnapshotFrame<'s>>,
    #[serde(rename = "watchResults")]
    watch_results: Vec<JsWatchResult<'s>>,
}

fn snapshot_to_json(
    snapshot: &jvmjs_vm::DebugSnapshot,
    watch_results: &[(String, Result<String, String>)],
) -> String {
    let js = JsSnapshot {
        reason: match snapshot.reason {
            jvmjs_vm::PauseReason::Breakpoint => "breakpoint",
            jvmjs_vm::PauseReason::Step => "step",
            jvmjs_vm::PauseReason::Interrupt => "interrupt",
        },
        frames: snapshot
            .frames
            .iter()
            .map(|f| JsSnapshotFrame {
                class_name: &f.class_name,
                method_name: &f.method_name,
                source_file: &f.source_file,
                line: f.line,
                locals: f
                    .locals
                    .iter()
                    .map(|l| JsLocal {
                        name: &l.name,
                        type_name: &l.type_name,
                        value: &l.value,
                    })
                    .collect(),
            })
            .collect(),
        watch_results: watch_results
            .iter()
            .map(|(expression, outcome)| JsWatchResult {
                expression,
                value: outcome.as_ref().ok().cloned(),
                error: outcome.as_ref().err().cloned(),
            })
            .collect(),
    };
    serde_json::to_string(&js).unwrap_or_default()
}

#[derive(Deserialize)]
struct JsBreakpoint {
    file: String,
    line: u32,
}

#[derive(Deserialize)]
struct JsDebugControl {
    command: String,
    breakpoints: Option<Vec<JsBreakpoint>>,
    watches: Option<Vec<String>>,
}

/// A parsed pause response: the VM-facing control plus the watch-list
/// replacement, and whether this was a refresh round (re-evaluate and
/// re-present without resuming).
struct ParsedControl {
    control: jvmjs_vm::DebugControl,
    watches: Option<Vec<String>>,
}

fn parse_breakpoints(json: &str) -> Vec<jvmjs_vm::Breakpoint> {
    serde_json::from_str::<Vec<JsBreakpoint>>(json)
        .unwrap_or_default()
        .into_iter()
        .map(|b| jvmjs_vm::Breakpoint {
            file: b.file,
            line: b.line,
        })
        .collect()
}

fn parse_debug_control(json: &str) -> (ParsedControl, bool) {
    let parsed: Option<JsDebugControl> = serde_json::from_str(json).ok();
    let (command, breakpoints, watches) = parsed
        .map_or((String::from("terminate"), None, None), |c| {
            (c.command, c.breakpoints, c.watches)
        });
    let refresh = command == "refresh";
    let control = jvmjs_vm::DebugControl {
        command: match command.as_str() {
            // A refresh round never reaches the VM; Continue is a
            // placeholder for the loop.
            "continue" | "refresh" => jvmjs_vm::DebugCommand::Continue,
            "stepOver" => jvmjs_vm::DebugCommand::StepOver,
            "stepInto" => jvmjs_vm::DebugCommand::StepInto,
            "stepOut" => jvmjs_vm::DebugCommand::StepOut,
            _ => jvmjs_vm::DebugCommand::Terminate,
        },
        breakpoints: breakpoints.map(|list| {
            list.into_iter()
                .map(|b| jvmjs_vm::Breakpoint {
                    file: b.file,
                    line: b.line,
                })
                .collect()
        }),
    };
    (ParsedControl { control, watches }, refresh)
}

#[derive(Debug, Serialize)]
struct JsRunResult {
    /// `"completed"`, `"exited"`, `"stopped"`, or `"error"`.
    status: &'static str,
    #[serde(rename = "exitCode")]
    exit_code: i32,
    /// Present when `status` is `"error"`.
    error: Option<String>,
}

/// Console IO that forwards to JavaScript callbacks.
struct JsConsole<'a> {
    stdout: &'a js_sys::Function,
    stderr: &'a js_sys::Function,
    stdin: Option<&'a js_sys::Function>,
}

impl ConsoleIo for JsConsole<'_> {
    fn now_millis(&mut self) -> i64 {
        #[allow(clippy::cast_possible_truncation)]
        let millis = js_sys::Date::now() as i64;
        millis
    }

    fn stdout(&mut self, bytes: &[u8]) {
        let text = JsValue::from_str(&String::from_utf8_lossy(bytes));
        let _ = self.stdout.call1(&JsValue::NULL, &text);
    }

    fn stderr(&mut self, bytes: &[u8]) {
        let text = JsValue::from_str(&String::from_utf8_lossy(bytes));
        let _ = self.stderr.call1(&JsValue::NULL, &text);
    }

    fn read_line(&mut self) -> Option<String> {
        let result = self.stdin?.call0(&JsValue::NULL).ok()?;
        result.as_string()
    }
}

/// One JVM sandbox: a virtual filesystem plus the classes compiled into
/// it. A page can create several sessions and they share nothing.
#[wasm_bindgen]
#[derive(Debug)]
pub struct JvmSession {
    /// Sources of the last successful compile — watch expressions are
    /// compiled together with them so they can reference user classes.
    sources: Vec<SourceFile>,
    vfs: VirtualFileSystem,
    classes: Vec<jvmjs_compiler::CompiledClass>,
}

impl Default for JvmSession {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl JvmSession {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            vfs: VirtualFileSystem::new(),
            sources: Vec::new(),
            classes: Vec::new(),
        }
    }

    /// `javac`: compile Java source files.
    ///
    /// `sources` is an array of `{ path, text }`. Returns
    /// `{ success, classNames, diagnostics }`. On success the compiled
    /// classes are retained in the session for [`JvmSession::run`].
    pub fn compile(&mut self, sources: JsValue) -> Result<JsValue, JsValue> {
        let sources: Vec<JsSourceFile> = serde_wasm_bindgen::from_value(sources)
            .map_err(|e| JsValue::from_str(&format!("invalid sources argument: {e}")))?;
        let sources: Vec<SourceFile> = sources
            .into_iter()
            .map(|s| SourceFile {
                path: s.path,
                text: s.text,
            })
            .collect();

        let compilation = jvmjs_compiler::compile(&sources);

        let result = JsCompileResult {
            success: compilation.success(),
            class_names: compilation
                .classes
                .iter()
                .map(|c| c.binary_name.clone())
                .collect(),
            diagnostics: compilation
                .diagnostics
                .iter()
                .map(|d| JsDiagnostic {
                    severity: match d.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                    },
                    message: d.message.clone(),
                    path: d.path.clone(),
                    start: d.span.map(|s| JsPosition {
                        line: s.start.line,
                        column: s.start.column,
                    }),
                    end: d.span.map(|s| JsPosition {
                        line: s.end.line,
                        column: s.end.column,
                    }),
                })
                .collect(),
            validation_entry: compilation.validation_entry.clone(),
        };

        if result.success {
            self.classes = compilation.classes;
            self.sources = sources;
        }

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// `java`: run `public static void main(String[] args)` of a
    /// previously compiled class.
    ///
    /// `stdout` and `stderr` receive string chunks as the program
    /// writes them. `stdin`, if provided, is called when the program
    /// reads a line and should return a string or `null` for EOF.
    #[allow(clippy::needless_pass_by_value)]
    pub fn run(
        &mut self,
        main_class: &str,
        args: Vec<String>,
        stdout: &js_sys::Function,
        stderr: &js_sys::Function,
        stdin: Option<js_sys::Function>,
    ) -> Result<JsValue, JsValue> {
        let mut console = JsConsole {
            stdout,
            stderr,
            stdin: stdin.as_ref(),
        };
        // Real entropy for Math.random(); tests off-browser use the
        // deterministic default seed instead.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let seed = (js_sys::Math::random() * 9_007_199_254_740_992.0) as u64;
        let options = VmOptions {
            random_seed: Some(seed),
            ..VmOptions::default()
        };
        let mut vm = Vm::new(options, &mut self.vfs, &mut console);
        for compiled in &self.classes {
            vm.load_class(compiled.class_file.clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }

        let result = match vm.run_main(main_class, &args) {
            Ok(jvmjs_vm::vm::ExitStatus::Completed) => JsRunResult {
                status: "completed",
                exit_code: 0,
                error: None,
            },
            Ok(jvmjs_vm::vm::ExitStatus::Exited(code)) => JsRunResult {
                status: "exited",
                exit_code: code,
                error: None,
            },
            Err(e) => JsRunResult {
                status: "error",
                exit_code: 1,
                error: Some(e.to_string()),
            },
        };
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// `java` under a debugger. `breakpoints` is JSON
    /// `[{"file","line"}]`. `on_pause` receives a snapshot as a JSON
    /// string and must return a command JSON string
    /// `{"command": "continue|stepOver|stepInto|stepOut|terminate",
    ///   "breakpoints"?: [{"file","line"}]}` — it may block (the worker
    /// waits on a `SharedArrayBuffer`). `poll_interrupt`, if given, is
    /// called periodically and returns whether to pause.
    #[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
    #[wasm_bindgen(js_name = runDebug)]
    pub fn run_debug(
        &mut self,
        main_class: &str,
        args: Vec<String>,
        breakpoints_json: &str,
        watches_json: &str,
        stdout: &js_sys::Function,
        stderr: &js_sys::Function,
        stdin: Option<js_sys::Function>,
        on_pause: &js_sys::Function,
        poll_interrupt: Option<js_sys::Function>,
    ) -> Result<JsValue, JsValue> {
        let mut console = JsConsole {
            stdout,
            stderr,
            stdin: stdin.as_ref(),
        };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let seed = (js_sys::Math::random() * 9_007_199_254_740_992.0) as u64;
        let options = VmOptions {
            random_seed: Some(seed),
            ..VmOptions::default()
        };
        let breakpoints = parse_breakpoints(breakpoints_json);
        let mut vm = Vm::new(options, &mut self.vfs, &mut console);
        for compiled in &self.classes {
            vm.load_class(compiled.class_file.clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        let mut host = JsDebugHost {
            on_pause,
            poll_interrupt: poll_interrupt.as_ref(),
            watches: serde_json::from_str(watches_json).unwrap_or_default(),
            sources: &self.sources,
        };
        let result = match vm.run_main_debug(main_class, &args, &breakpoints, &mut host) {
            Ok(jvmjs_vm::vm::ExitStatus::Completed) => JsRunResult {
                status: "completed",
                exit_code: 0,
                error: None,
            },
            Ok(jvmjs_vm::vm::ExitStatus::Exited(code)) => JsRunResult {
                status: "exited",
                exit_code: code,
                error: None,
            },
            Err(jvmjs_vm::VmError::Stopped) => JsRunResult {
                status: "stopped",
                exit_code: 0,
                error: None,
            },
            Err(e) => JsRunResult {
                status: "error",
                exit_code: 1,
                error: Some(e.to_string()),
            },
        };
        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    // ----- Virtual filesystem, exposed so the host page can seed
    // ----- inputs and inspect outputs of java.io.File operations.

    /// Write a file into the session's virtual filesystem.
    #[wasm_bindgen(js_name = writeFile)]
    pub fn write_file(&mut self, path: &str, contents: &[u8]) -> Result<(), JsValue> {
        self.vfs
            .write_file(path, contents.to_vec())
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Read a file from the session's virtual filesystem.
    #[wasm_bindgen(js_name = readFile)]
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, JsValue> {
        self.vfs
            .read_file(path)
            .map(<[u8]>::to_vec)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// List the immediate children of a directory (absolute paths).
    #[wasm_bindgen(js_name = listDir)]
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, JsValue> {
        self.vfs
            .list_dir(path)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Whether a file or directory exists.
    #[must_use]
    pub fn exists(&self, path: &str) -> bool {
        self.vfs.exists(path)
    }

    /// Create a directory (and missing parents).
    pub fn mkdir(&mut self, path: &str) -> Result<(), JsValue> {
        self.vfs
            .mkdir(path)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Delete a file or empty directory.
    pub fn remove(&mut self, path: &str) -> Result<(), JsValue> {
        self.vfs
            .remove(path)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Version of the jvmjs engine, for display in the host page.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}
