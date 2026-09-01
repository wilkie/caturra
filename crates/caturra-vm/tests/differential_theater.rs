//! Differential tests for `org.code.theater`: the same drawing program through
//! the **real Code.org library** and through caturra, compared action by
//! action.
//!
//! This is the last part of the course library nothing compared. The reason it
//! was left is real: the JDK library's `Theater.playScenes` renders a scene
//! into a `BufferedImage` and encodes a GIF, and caturra deliberately draws in
//! the browser instead — there is no common pixel surface, which is why
//! `scripts/sweep/reference` STUBS `Scene` entirely. `scripts/coverage/course
//! .py --semantic` reported those 29 names as unreachable.
//!
//! But a Scene RECORDS before it renders: the real one builds a
//! `List<SceneAction>`, and caturra's writes the same drawing in its own
//! vocabulary to `theater.log`. Those two ARE comparable, and they are what a
//! student's program actually produces — the pixels after them are the same
//! drawing, made by two renderers nobody is asking to agree.
//!
//! The two record the STATE differently, and the difference is the interesting
//! part: the real library bakes the stroke, fill and width into each shape as
//! it is drawn, while caturra emits the state change as its own command. So
//! caturra's log is folded here — state applied to each shape as it goes —
//! which is exactly the reading a renderer has to take, and a divergence in it
//! is a shape drawn in the wrong colour.
//!
//! Two of `Scene`'s names are recorded by both and compared by neither:
//! `drawImage` and `playSound` carry a REFERENCE to pixels or samples, and the
//! two libraries reference them differently — caturra's log names a buffer it
//! handed the host, where the real action holds the `Image` object. The
//! geometry around them could be compared by teaching this file to expand
//! caturra's `size` into the width and height the real action derives, which
//! would be the drawing rule written down twice; it is left alone instead, and
//! `scripts/coverage/course.py --semantic` counts them as not run.
//!
//! Needs the `javabuilder/` checkout and a JDK; skips with a note otherwise,
//! and `CATURRA_REQUIRE_JAVABUILDER=1` makes that a failure.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

/// Prints a Scene's recorded actions in caturra's `theater.log` vocabulary.
/// Lives in `org.code.theater` because `Scene.getActions()` is package-private
/// — the real library exposes the list to its own renderer and to nothing else.
const PROBE: &str = r#"
package org.code.theater;

import java.util.List;
import org.code.media.Color;
import org.code.theater.support.*;

public class TheaterProbe {
    static String color(Color c) {
        return c == null ? "none" : c.getRed() + " " + c.getGreen() + " " + c.getBlue();
    }

    static String paint(PaintableAction a) {
        return " stroke=" + color(a.getStrokeColor()) + " fill=" + color(a.getFillColor())
                + " width=" + a.getStrokeWidth();
    }

    public static void dump(Scene scene) {
        List<SceneAction> actions = scene.getActions();
        for (SceneAction action : actions) {
            switch (action.getType()) {
                case CLEAR_SCENE: {
                    ClearSceneAction a = (ClearSceneAction) action;
                    System.out.println("clear " + color(a.getColor()));
                    break;
                }
                case PAUSE: {
                    PauseAction a = (PauseAction) action;
                    System.out.println("pause " + a.getSeconds());
                    break;
                }
                case PLAY_NOTE: {
                    PlayNoteAction a = (PlayNoteAction) action;
                    System.out.println("note " + a.getInstrument() + " " + a.getNote() + " "
                            + a.getSeconds());
                    break;
                }
                case DRAW_RECTANGLE: {
                    DrawRectangleAction a = (DrawRectangleAction) action;
                    System.out.println("rectangle " + a.getX() + " " + a.getY() + " "
                            + a.getWidth() + " " + a.getHeight() + paint(a));
                    break;
                }
                case DRAW_ELLIPSE: {
                    DrawEllipseAction a = (DrawEllipseAction) action;
                    System.out.println("ellipse " + a.getX() + " " + a.getY() + " "
                            + a.getWidth() + " " + a.getHeight() + paint(a));
                    break;
                }
                case DRAW_LINE: {
                    DrawLineAction a = (DrawLineAction) action;
                    System.out.println("line " + a.getStartX() + " " + a.getStartY() + " "
                            + a.getEndX() + " " + a.getEndY() + " stroke=" + color(a.getColor())
                            + " width=" + a.getStrokeWidth());
                    break;
                }
                case DRAW_POLYGON: {
                    DrawPolygonAction a = (DrawPolygonAction) action;
                    System.out.println("polygon " + a.getX() + " " + a.getY() + " "
                            + a.getSides() + " " + a.getRadius() + paint(a));
                    break;
                }
                case DRAW_SHAPE: {
                    DrawShapeAction a = (DrawShapeAction) action;
                    StringBuilder out = new StringBuilder("shape");
                    for (int p : a.getPoints()) out.append(" ").append(p);
                    System.out.println(out + " " + a.isClosed() + paint(a));
                    break;
                }
                case DRAW_TEXT: {
                    DrawTextAction a = (DrawTextAction) action;
                    System.out.println("text \"" + a.getText() + "\" " + a.getX() + " " + a.getY()
                            + " " + a.getRotation() + " height=" + a.getHeight()
                            + " font=" + a.getFont() + " style=" + a.getFontStyle()
                            + " color=" + color(a.getColor()));
                    break;
                }
                default:
                    System.out.println("other " + action.getType());
            }
        }
    }
}
"#;

/// Calls the program's `main` and then dumps the Scene it left behind.
const RUNNER: &str = r"
public class TheaterRunner {
    public static void main(String[] args) throws Exception {
        Drawing.main(new String[0]);
        org.code.theater.TheaterProbe.dump(Drawing.scene);
    }
}
";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

fn javabuilder() -> Option<PathBuf> {
    let root = workspace_root().join("javabuilder/org-code-javabuilder");
    root.join("theater/src/main/java/org/code/theater/Scene.java")
        .is_file()
        .then_some(root)
}

fn jdk_available() -> bool {
    Command::new("javac")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
        && Command::new("java")
            .arg("--version")
            .output()
            .is_ok_and(|out| out.status.success())
}

fn reference_available() -> bool {
    let available = jdk_available() && javabuilder().is_some();
    assert!(
        available || std::env::var_os("CATURRA_REQUIRE_JAVABUILDER").is_none(),
        "CATURRA_REQUIRE_JAVABUILDER is set, but {}: this suite would have \
         reported `ok` without comparing anything",
        if jdk_available() {
            "the javabuilder/ checkout is missing"
        } else {
            "no JDK is on PATH"
        }
    );
    if !available {
        eprintln!(
            "skipping: {}",
            if jdk_available() {
                "no javabuilder/ checkout to compare against"
            } else {
                "no JDK on PATH"
            }
        );
    }
    available
}

/// The source path that lets javac pull exactly the Code.org classes it needs
/// — the theater package, the media package it draws with, and the protocol
/// shim `scripts/sweep/reference` already keeps for the same purpose.
fn source_path(root: &Path) -> String {
    [
        workspace_root().join("scripts/sweep/reference/shim"),
        root.join("theater/src/main/java"),
        root.join("media/src/main/java"),
        root.join("protocol/src/main/java"),
        root.join("lang/src/main/java"),
    ]
    .iter()
    .map(|path| path.display().to_string())
    .collect::<Vec<_>>()
    .join(":")
}

fn probe_classes() -> &'static Path {
    static CLASSES: OnceLock<PathBuf> = OnceLock::new();
    CLASSES.get_or_init(|| {
        let root = javabuilder().expect("caller checked the checkout is present");
        let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("org-code-theater");
        let sources = out.join("src/org/code/theater");
        std::fs::create_dir_all(&sources).expect("create probe dir");
        let probe = sources.join("TheaterProbe.java");
        std::fs::write(&probe, PROBE).expect("write the probe");
        let compile = Command::new("javac")
            .args(["-nowarn", "-d"])
            .arg(&out)
            .arg("-sourcepath")
            .arg(source_path(&root))
            .arg(&probe)
            .output()
            .expect("javac runs");
        assert!(
            compile.status.success(),
            "the real org.code.theater did not compile — its dependencies have \
             moved: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        out
    })
}

/// Build the drawing program, run it on a real JVM, and dump the actions.
fn run_with_reference(body: &str) -> String {
    let root = javabuilder().expect("checked");
    let classes = probe_classes();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(body, &mut hasher);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("theater-{:x}", std::hash::Hasher::finish(&hasher)));
    std::fs::create_dir_all(&dir).expect("create case dir");
    std::fs::write(dir.join("Drawing.java"), program(body, false)).expect("write program");
    std::fs::write(dir.join("TheaterRunner.java"), RUNNER).expect("write runner");

    let compile = Command::new("javac")
        .args(["-nowarn", "-cp"])
        .arg(classes)
        .arg("-d")
        .arg(&dir)
        .arg("-sourcepath")
        .arg(format!("{}:{}", source_path(&root), dir.display()))
        .arg(dir.join("Drawing.java"))
        .arg(dir.join("TheaterRunner.java"))
        .output()
        .expect("javac runs");
    assert!(
        compile.status.success(),
        "javac rejected the drawing against the real library: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .arg("-cp")
        .arg(format!("{}:{}", dir.display(), classes.display()))
        .arg("TheaterRunner")
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "the real library failed to run the drawing: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).into_owned()
}

/// The same drawing on caturra: run it, read the `theater.log` it wrote, and
/// FOLD the state commands into the shapes they apply to, which is what the
/// real library's actions carry.
fn run_with_caturra(body: &str) -> String {
    let source = program(body, true);
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("Drawing.java"),
        text: source,
    }]);
    assert!(
        compilation.success(),
        "caturra rejected the drawing: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("class loads");
    }
    let result = vm.run_main("Drawing", &[]);
    drop(vm);
    assert!(
        result.is_ok(),
        "caturra failed to run the drawing: {result:?}; stderr: {}",
        console.stderr_text()
    );
    let log = vfs
        .read_file("theater.log")
        .expect("Theater.playScenes writes theater.log");
    let mut out = String::new();
    out.push_str(&console.stdout_text());
    out.push_str(&fold(&String::from_utf8_lossy(log)));
    out
}

/// The Scene's defaults, from the real `Scene`'s own constants: black stroke
/// and fill, a stroke one wide, 20-high sans text in black.
fn fold(log: &str) -> String {
    let (mut stroke, mut fill, mut width) = (
        String::from("0 0 0"),
        String::from("0 0 0"),
        String::from("1.0"),
    );
    let (mut height, mut font, mut style, mut text_color) = (
        String::from("20"),
        String::from("SANS"),
        String::from("NORMAL"),
        String::from("0 0 0"),
    );
    let mut folded = String::new();
    for line in log.lines() {
        let (head, rest) = line.split_once(' ').unwrap_or((line, ""));
        match head {
            "strokeColor" => rest.clone_into(&mut stroke),
            "fillColor" => rest.clone_into(&mut fill),
            "strokeWidth" => rest.clone_into(&mut width),
            "textHeight" => rest.clone_into(&mut height),
            "textColor" => rest.clone_into(&mut text_color),
            "textStyle" => {
                let (one, two) = rest.split_once(' ').unwrap_or((rest, ""));
                one.clone_into(&mut font);
                two.clone_into(&mut style);
            }
            "rectangle" | "ellipse" | "polygon" | "shape" => {
                let _ = writeln!(folded, "{line} stroke={stroke} fill={fill} width={width}");
            }
            // A line has no inside to fill.
            "line" => {
                let _ = writeln!(folded, "{line} stroke={stroke} width={width}");
            }
            "text" => {
                let _ = writeln!(
                    folded,
                    "{line} height={height} font={font} style={style} color={text_color}"
                );
            }
            _ => {
                let _ = writeln!(folded, "{line}");
            }
        }
    }
    folded
}

/// The same drawing for both engines, with one difference: only caturra's
/// PLAYS it. The real `Theater` renders through an AWS content manager, so
/// naming it at all drags the whole session protocol into the compile — and
/// the reference does not need it, since its runner reads the Scene's actions
/// directly.
fn program(body: &str, plays: bool) -> String {
    let (import, play) = if plays {
        (
            "import org.code.theater.Theater;\n",
            "        Theater.playScenes(scene);\n",
        )
    } else {
        ("", "")
    };
    format!(
        "import org.code.media.Color;\n\
         import org.code.media.Font;\n\
         import org.code.media.FontStyle;\n\
         import org.code.theater.Instrument;\n\
         import org.code.theater.Scene;\n\
         {import}\n\
         public class Drawing {{\n\
         \x20   public static Scene scene = new Scene();\n\n\
         \x20   public static void main(String[] args) throws Exception {{\n\
         {body}\n\
         {play}\
         \x20   }}\n\
         }}\n"
    )
}

fn assert_same_drawing(body: &str) {
    // The reference never calls `playScenes` — its runner dumps the actions
    // instead — so the caturra side's extra `playScenes` writes the log and
    // prints nothing.
    let expected = run_with_reference(body);
    let actual = run_with_caturra(body);
    assert_eq!(
        expected, actual,
        "caturra's drawing diverges from the real Code.org library"
    );
}

macro_rules! theater_differential_test {
    ($name:ident, $body:literal) => {
        #[test]
        fn $name() {
            if !reference_available() {
                return;
            }
            assert_same_drawing($body);
        }
    };
}

// Every shape, with the colours and the stroke width that were in force when
// it was drawn — which the real library bakes into the action and caturra
// records as a state change, so a divergence here is a shape drawn wrong.
theater_differential_test!(
    theater_shapes_carry_the_state_they_were_drawn_with,
    r#"        scene.clear("white");
        scene.setFillColor("red");
        scene.setStrokeColor(new Color(0, 0, 255));
        scene.setStrokeWidth(3.0);
        scene.drawRectangle(10, 20, 30, 40);
        scene.drawEllipse(5, 6, 7, 8);
        scene.drawLine(1, 2, 3, 4);
        scene.drawRegularPolygon(50, 60, 5, 25);
        scene.drawShape(new int[] {0, 0, 10, 0, 10, 10}, true);
        scene.removeFillColor();
        scene.drawRectangle(1, 1, 2, 2);
        scene.removeStrokeColor();
        scene.drawEllipse(2, 2, 3, 3);"#
);

// Text carries four pieces of state of its own, and the two enums it names are
// the ones that were missing constants.
theater_differential_test!(
    theater_text_carries_its_font_and_height,
    r#"        scene.drawText("default", 1, 2);
        scene.setTextHeight(30);
        scene.setTextStyle(Font.SERIF, FontStyle.BOLD_ITALIC);
        scene.setTextColor("green");
        scene.drawText("styled", 7, 8);
        scene.drawText("tilted", 9, 10, 45.0);
        scene.setTextStyle(Font.MONO, FontStyle.ITALIC);
        scene.setTextColor(new Color(1, 2, 3));
        scene.drawText("again", 0, 0, -12.5);"#
);

// The two actions that are not drawing at all: a pause (clamped to a tenth of
// a second, in both) and a note.
theater_differential_test!(
    theater_pauses_and_notes,
    r"        scene.pause(0.05);
        scene.pause(2.0);
        scene.playNote(60, 0.5);
        scene.playNote(Instrument.BASS, 40, 1.5);
        scene.playNoteAndPause(Instrument.PIANO, 55, 0.25);"
);

// The named colours, which are a table on both sides and are where a typo
// would sit forever: a shape drawn in the wrong colour renders happily.
theater_differential_test!(
    theater_named_colours_agree,
    r#"        String[] names = {
            "white", "silver", "gray", "black", "red", "maroon", "yellow", "olive",
            "lime", "green", "aqua", "teal", "blue", "navy", "fuchsia", "purple",
            "pink", "orange", "gold", "brown", "chocolate", "tan", "turquoise",
            "indigo", "violet", "beige", "ivory",
        };
        for (String name : names) {
            scene.setFillColor(name);
            scene.drawRectangle(0, 0, 1, 1);
        }
        // …and a name the table does not have is a failure, not a default.
        // "grey" is the one a program reaches for and does not get.
        try {
            scene.setFillColor("grey");
            System.out.println("no throw");
        } catch (IllegalArgumentException e) {
            System.out.println("threw: " + e.getMessage());
        }"#
);
