//! End-to-end VM tests.
//!
//! Two fixture strategies (see `specs/SCOPE.md` consequences):
//! - hand-assembled class files built with `caturra-classfile`'s writer,
//!   which pin the VM's behavior independently of our compiler, and
//! - compile-then-run tests through `caturra-compiler`, which pin the
//!   whole engine end to end.

use caturra_classfile::opcodes as op;
use caturra_classfile::{
    AttributeInfo, CODE_ATTRIBUTE, ClassFile, CodeAttribute, Constant, MethodAccessFlags,
    MethodInfo, write_code_attribute,
};
use caturra_vm::{
    Breakpoint, DebugCommand, DebugControl, DebugHost, DebugSnapshot, PauseReason, WatchEvaluator,
};
use caturra_vm::{BufferedConsole, ExitStatus, VirtualFileSystem, Vm, VmError, VmOptions};

/// Assemble a class equivalent to:
/// ```java
/// public class Fixture {
///     public static void main(String[] args) {
///         System.out.println("hand-assembled");
///     }
/// }
/// ```
fn hand_assembled_hello() -> ClassFile {
    let mut class = ClassFile::new_java11();
    let pool = &mut class.constant_pool;

    let fixture_name = pool.intern_utf8("Fixture");
    class.this_class = pool.intern(Constant::Class {
        name_index: fixture_name,
    });
    let object_name = pool.intern_utf8("java/lang/Object");
    class.super_class = pool.intern(Constant::Class {
        name_index: object_name,
    });

    let system_name = pool.intern_utf8("java/lang/System");
    let system_class = pool.intern(Constant::Class {
        name_index: system_name,
    });
    let out_name = pool.intern_utf8("out");
    let out_descriptor = pool.intern_utf8("Ljava/io/PrintStream;");
    let out_nat = pool.intern(Constant::NameAndType {
        name_index: out_name,
        descriptor_index: out_descriptor,
    });
    let out_field = pool.intern(Constant::FieldRef {
        class_index: system_class,
        name_and_type_index: out_nat,
    });

    let message_utf8 = pool.intern_utf8("hand-assembled");
    let message = pool.intern(Constant::String {
        string_index: message_utf8,
    });

    let stream_name = pool.intern_utf8("java/io/PrintStream");
    let stream_class = pool.intern(Constant::Class {
        name_index: stream_name,
    });
    let println_name = pool.intern_utf8("println");
    let println_descriptor = pool.intern_utf8("(Ljava/lang/String;)V");
    let println_nat = pool.intern(Constant::NameAndType {
        name_index: println_name,
        descriptor_index: println_descriptor,
    });
    let println_method = pool.intern(Constant::MethodRef {
        class_index: stream_class,
        name_and_type_index: println_nat,
    });

    let mut code = Vec::new();
    code.push(op::GETSTATIC);
    code.extend_from_slice(&out_field.to_be_bytes());
    code.push(op::LDC);
    code.push(u8::try_from(message).expect("small pool"));
    code.push(op::INVOKEVIRTUAL);
    code.extend_from_slice(&println_method.to_be_bytes());
    code.push(op::RETURN);

    let code_attribute = CodeAttribute {
        max_stack: 2,
        max_locals: 1,
        code,
        exception_table: Vec::new(),
        attributes: Vec::new(),
    };

    let main_name = pool.intern_utf8("main");
    let main_descriptor = pool.intern_utf8("([Ljava/lang/String;)V");
    let code_name = pool.intern_utf8(CODE_ATTRIBUTE);
    class.methods.push(MethodInfo {
        access_flags: MethodAccessFlags(MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
        name_index: main_name,
        descriptor_index: main_descriptor,
        attributes: vec![AttributeInfo {
            name_index: code_name,
            info: write_code_attribute(&code_attribute),
        }],
    });
    class
}

fn run_class(class: ClassFile, main: &str) -> (Result<ExitStatus, VmError>, BufferedConsole) {
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    vm.load_class(class).expect("class should load");
    let result = vm.run_main(main, &[]);
    (result, console)
}

fn compile_and_run(source: &str, main: &str) -> (Result<ExitStatus, VmError>, BufferedConsole) {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{main}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file)
            .expect("compiled class should load");
    }
    let result = vm.run_main(main, &[]);
    (result, console)
}

#[test]
fn casts_a_super_method_call() {
    // `(Type) super.method()` — a cast whose operand starts with `super`.
    let (result, console) = compile_and_run(
        r#"
        class Base {
            Object describe() { return "base"; }
        }
        public class Main extends Base {
            String shout() { return ((String) super.describe()).toUpperCase(); }
            public static void main(String[] args) {
                System.out.println(new Main().shout());
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "BASE\n");
}

#[test]
fn system_exit_terminates_with_a_status_code() {
    // System.exit(code) ends the program immediately — the code returns the
    // given status and nothing after the call runs.
    let (result, console) = compile_and_run(
        r#"
        public class Main {
            public static void main(String[] args) {
                System.out.println("before");
                System.exit(3);
                System.out.println("after");
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Exited(3))), "{result:?}");
    assert_eq!(console.stdout_text(), "before\n");
}

#[test]
fn system_exit_is_uncatchable_and_skips_finally() {
    // Like the real JVM, System.exit unwinds past catch and finally — neither
    // the handler nor the finally block runs.
    let (result, console) = compile_and_run(
        r#"
        public class Main {
            public static void main(String[] args) {
                try {
                    System.exit(0);
                } catch (Throwable t) {
                    System.out.println("caught");
                } finally {
                    System.out.println("finally");
                }
                System.out.println("returned");
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Exited(0))), "{result:?}");
    assert_eq!(console.stdout_text(), "");
}

#[test]
fn runs_hand_assembled_hello_world() {
    let (result, console) = run_class(hand_assembled_hello(), "Fixture");
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "hand-assembled\n");
    assert_eq!(console.stderr_text(), "");
}

#[test]
fn hand_assembled_class_survives_binary_round_trip() {
    // The same fixture, but written to bytes and read back before
    // running — pinning writer + reader + interpreter together.
    let bytes = caturra_classfile::write_class_file(&hand_assembled_hello());
    let class = caturra_classfile::read_class_file(&bytes).expect("fixture should parse");
    let (result, console) = run_class(class, "Fixture");
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "hand-assembled\n");
}

#[test]
fn compiles_and_runs_hello_world() {
    let (result, console) = compile_and_run(
        r#"
        public class Main {
            public static void main(String[] args) {
                System.out.println("Hello, World!");
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "Hello, World!\n");
}

#[test]
fn compiles_and_runs_every_println_overload() {
    let (result, console) = compile_and_run(
        r#"
        public class Kitchen {
            public static void main(String[] args) {
                System.out.print("value: ");
                System.out.println(42);
                System.out.println(3.5);
                System.out.println(2.0);
                System.out.println(true);
                System.out.println('x');
                System.out.println();
                System.err.println("to stderr");
            }
        }
        "#,
        "Kitchen",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "value: 42\n3.5\n2.0\ntrue\nx\n\n");
    assert_eq!(console.stderr_text(), "to stderr\n");
}

#[test]
fn string_literals_are_interned_within_a_run() {
    // Same literal printed twice should hit the same heap object; this
    // is observable indirectly (no crash, correct output) and pins the
    // intern path.
    let (result, console) = compile_and_run(
        r#"
        class Twice {
            public static void main(String[] args) {
                System.out.println("same");
                System.out.println("same");
            }
        }
        "#,
        "Twice",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "same\nsame\n");
}

// ----- stage 1: locals, operators, string concatenation -----

/// Compile, run, and return stdout, asserting a clean completion.
fn run_stdout(source: &str, main: &str) -> String {
    let (result, console) = compile_and_run(source, main);
    assert!(
        matches!(result, Ok(ExitStatus::Completed)),
        "{result:?}; stderr: {}",
        console.stderr_text()
    );
    console.stdout_text()
}

/// Compile a neighborhood program (the bundled `org.code.neighborhood`
/// library is auto-injected), seed the grid file, and return stdout.
fn run_neighborhood(source: &str, main: &str, grid: &str) -> String {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{main}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    vfs.write_file("grid.txt", grid.as_bytes().to_vec())
        .expect("seed grid");
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    let result = vm.run_main(main, &[]);
    assert!(
        matches!(result, Ok(ExitStatus::Completed)),
        "{result:?}; stderr: {}",
        console.stderr_text()
    );
    console.stdout_text()
}

/// Compile a theater program (the bundled `org.code.theater` /
/// `org.code.media` library is auto-injected), run it, and return
/// `(stdout, theater.log)` — the log is the recorded draw commands.
fn run_theater(source: &str, main: &str) -> (String, String) {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{main}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    let result = vm.run_main(main, &[]);
    assert!(
        matches!(result, Ok(ExitStatus::Completed)),
        "{result:?}; stderr: {}",
        console.stderr_text()
    );
    let log = vfs
        .read_file("theater.log")
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_default();
    (console.stdout_text(), log)
}

#[test]
fn theater_scene_draw_commands() {
    let (stdout, log) = run_theater(
        r#"
        import org.code.theater.*;
        import org.code.media.*;
        public class ArtScene extends Scene {
            public static void main(String[] args) {
                ArtScene s = new ArtScene();
                s.clear("white");
                s.setFillColor("blue");
                s.setStrokeColor(new Color(0, 0, 0));
                s.setStrokeWidth(2.0);
                s.drawRectangle(10, 20, 100, 50);
                s.setFillColor(Color.RED);
                s.drawEllipse(150, 150, 80, 80);
                s.setTextStyle(Font.SANS, FontStyle.BOLD);
                s.drawText("Hello", 200, 300);
                Image img = new Image(64, 48);
                s.drawImage(img, 5, 5, 64);
                s.playNote(60, 1.0);
                System.out.println("canvas " + s.getWidth() + "x" + s.getHeight());
                Theater.playScenes(s);
            }
        }
        "#,
        "ArtScene",
    );
    assert_eq!(stdout, "canvas 400x400\n");
    assert_eq!(
        log,
        "clear 255 255 255\n\
         fillColor 0 0 255\n\
         strokeColor 0 0 0\n\
         strokeWidth 2.0\n\
         rectangle 10 20 100 50\n\
         fillColor 255 0 0\n\
         ellipse 150 150 80 80\n\
         textStyle SANS BOLD\n\
         text \"Hello\" 200 300 0.0\n\
         image 64x48 5 5 64\n\
         note PIANO 60 1.0\n"
    );
}

#[test]
fn theater_image_pixel_manipulation() {
    let (stdout, _log) = run_theater(
        r##"
        import org.code.theater.*;
        import org.code.media.*;
        public class ImgScene extends Scene {
            public static void main(String[] args) {
                Image img = new Image(4, 3);
                img.clear(new Color(10, 20, 30));
                for (int x = 0; x < img.getWidth(); x++) {
                    for (int y = 0; y < img.getHeight(); y++) {
                        Pixel p = img.getPixel(x, y);
                        p.setRed(255 - p.getRed());
                        p.setGreen(255 - p.getGreen());
                        p.setBlue(255 - p.getBlue());
                    }
                }
                Pixel c = img.getPixel(0, 0);
                System.out.println("corner=" + c.getRed() + "," + c.getGreen() + "," + c.getBlue());
                Color orange = new Color("orange");
                System.out.println("orange=" + orange.getRed() + "," + orange.getGreen() + "," + orange.getBlue());
                Color hex = new Color("#40E0D0");
                System.out.println("hex=" + hex.getRed() + "," + hex.getGreen() + "," + hex.getBlue());
            }
        }
        "##,
        "ImgScene",
    );
    assert_eq!(
        stdout,
        "corner=245,235,225\norange=255,165,0\nhex=64,224,208\n"
    );
}

/// `SoundLoader.read` returns a non-empty **silent** buffer (10 s at 44.1 kHz)
/// rather than `double[0]`, so the Code.org sound lessons' time-indexed clip
/// extraction (`sound[start * 44100]`, e.g. `createClip(sound, 2, 5)`) runs
/// instead of throwing `ArrayIndexOutOfBoundsException`. Headless audio can't be
/// decoded, so the samples are silence (zeros) — the honest stub. Regression
/// pin for the theater sound-clip levels.
#[test]
fn sound_loader_returns_indexable_silent_buffer() {
    let (stdout, log) = run_theater(
        r#"
        import org.code.theater.*;
        import org.code.media.*;
        public class SoundArt extends Scene {
            static double[] createClip(double[] sound, int start, int end) {
                int startIndex = start * 44100;
                int endIndex = end * 44100;
                double[] clip = new double[endIndex - startIndex];
                for (int i = startIndex; i < endIndex; i++) {
                    clip[i - startIndex] = sound[i];
                }
                return clip;
            }
            public static void main(String[] args) {
                double[] sound = SoundLoader.read("beat.wav");
                System.out.println("len=" + sound.length);
                double[] clip = createClip(sound, 2, 5);
                System.out.println("clip=" + clip.length + " first=" + clip[0]);
                SoundArt s = new SoundArt();
                s.playSound(clip);
                Theater.playScenes(s);
            }
        }
        "#,
        "SoundArt",
    );
    assert_eq!(stdout, "len=441000\nclip=132300 first=0.0\n");
    assert_eq!(log, "sound 132300 samples\n");
}

/// Compile a Swing program (the bundled `javax.swing` / `java.awt` library
/// is auto-injected), run it, and return `swing.json` — the component tree
/// `frame.setVisible(true)` serializes for the accessible-DOM renderer.
fn run_swing(source: &str, main: &str) -> String {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{main}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    let result = vm.run_main(main, &[]);
    assert!(
        matches!(result, Ok(ExitStatus::Completed)),
        "{result:?}; stderr: {}",
        console.stderr_text()
    );
    vfs.read_file("swing.json")
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_default()
}

#[test]
fn swing_serializes_an_accessible_component_tree() {
    // A labelled form: the label associates with the field (setLabelFor),
    // and a grid layout holds the widgets. Construction order fixes the ids
    // (frame c0, then each widget), so `for` points at the field's id.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Sign Up");
                frame.setLayout(new GridLayout(2, 2));
                JLabel nameLabel = new JLabel("Name:");
                JTextField nameField = new JTextField(12);
                nameLabel.setLabelFor(nameField);
                JCheckBox subscribe = new JCheckBox("Email me", true);
                JButton submit = new JButton("Submit");
                submit.setEnabled(false);
                frame.add(nameLabel);
                frame.add(nameField);
                frame.add(subscribe);
                frame.add(submit);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    // Root frame carries its title and grid layout.
    assert!(json.contains(r#""type":"frame""#), "no frame: {json}");
    assert!(json.contains(r#""title":"Sign Up""#), "no title: {json}");
    assert!(json.contains(r#""layout":"grid 2 2""#), "no layout: {json}");
    // The label associates with the text field's id (c2: frame=c0, label=c1,
    // field=c2), which is what the DOM renders as `<label for>`.
    assert!(
        json.contains(r#""type":"textfield","text":"","columns":12,"editable":true,"id":"c2""#),
        "no field: {json}"
    );
    assert!(
        json.contains(r#""type":"label","text":"Name:","for":"c2""#),
        "no label association: {json}"
    );
    // Checkbox selection and disabled button survive serialization.
    assert!(
        json.contains(r#""type":"checkbox","text":"Email me","selected":true"#),
        "no checkbox: {json}"
    );
    assert!(
        json.contains(r#""type":"button","text":"Submit","id":"c4","enabled":false"#),
        "no disabled button: {json}"
    );
}

#[test]
fn swing_border_layout_records_child_regions() {
    // add(component, BorderLayout.X) records the region on the child so the
    // renderer can place it; add() with no constraint carries none (the
    // renderer defaults it to CENTER).
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Layout");
                frame.setLayout(new BorderLayout());
                frame.add(new JLabel("Top"), BorderLayout.NORTH);
                frame.add(new JLabel("Bottom"), BorderLayout.SOUTH);
                frame.add(new JLabel("Middle"));
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""layout":"border""#),
        "no border layout: {json}"
    );
    assert!(
        json.contains(r#""text":"Top","id":"c1","enabled":true,"region":"North""#),
        "no north region: {json}"
    );
    assert!(
        json.contains(r#""text":"Bottom","id":"c2","enabled":true,"region":"South""#),
        "no south region: {json}"
    );
    // The unconstrained child carries no region field.
    assert!(
        json.contains(r#""text":"Middle","id":"c3","enabled":true}"#),
        "middle should have no region: {json}"
    );
}

#[test]
fn swing_table_serializes_headers_and_rows() {
    // A JTable serializes its column names and cell grid (each cell via
    // toString), for the accessible <table> renderer.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Scores");
                String[] columns = {"Name", "Score"};
                Object[][] data = {
                    {"Ada", "95"},
                    {"Al", "88"}
                };
                JTable table = new JTable(data, columns);
                frame.add(table);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(json.contains(r#""type":"table""#), "no table: {json}");
    assert!(
        json.contains(r#""headers":["Name","Score"]"#),
        "no headers: {json}"
    );
    assert!(
        json.contains(r#""cells":[["Ada","95"],["Al","88"]]"#),
        "no cells: {json}"
    );
    assert!(json.contains(r#""selectedRow":-1"#), "no selection: {json}");
}

#[test]
fn swing_table_row_selection_fires_listener() {
    // Selecting a row syncs the selected index and fires the selection model's
    // ListSelectionListener; getValueAt reads a cell. Ids: frame c0, table c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        public class Main {
            static JTable table;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Scores");
                String[] columns = {"Name", "Score"};
                Object[][] data = {{"Ada", "95"}, {"Al", "88"}, {"Bo", "72"}};
                table = new JTable(data, columns);
                table.getSelectionModel().addListSelectionListener(e -> {
                    int r = Main.table.getSelectedRow();
                    System.out.println("row " + r + " = " + Main.table.getValueAt(r, 0));
                });
                frame.add(table);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\nc1=2")),
            Some(String::from("c1\nc1=0")),
        ],
    );
    assert_eq!(out, "row 2 = Bo\nrow 0 = Ada\n");
}

#[test]
fn swing_default_table_model_updates_rows_on_render() {
    // A JTable backed by a DefaultTableModel re-reads the model each render, so
    // buttons that addRow / removeRow / setValueAt change the serialized grid.
    // Ids: frame c0, table c1, add c2, bump c3, remove c4.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static DefaultTableModel model;
            static int n = 0;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Grid");
                String[] cols = {"Item", "Qty"};
                Object[][] data = {{"Apple", "3"}};
                model = new DefaultTableModel(data, cols);
                JTable table = new JTable(model);
                JButton add = new JButton("Add");
                add.addActionListener(e -> {
                    Main.n = Main.n + 1;
                    Object[] row = {"Item " + Main.n, "1"};
                    Main.model.addRow(row);
                    System.out.println("rows " + Main.model.getRowCount());
                });
                JButton bump = new JButton("Bump");
                bump.addActionListener(e -> {
                    Main.model.setValueAt("9", 0, 1);
                    System.out.println("apple qty " + Main.model.getValueAt(0, 1));
                });
                JButton remove = new JButton("Remove");
                remove.addActionListener(e -> {
                    Main.model.removeRow(0);
                    System.out.println("rows " + Main.model.getRowCount());
                });
                frame.add(table);
                frame.add(add);
                frame.add(bump);
                frame.add(remove);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        // Add a row (2), bump apple's qty, then remove the first row (1 left).
        vec![
            Some(String::from("c2")),
            Some(String::from("c3")),
            Some(String::from("c4")),
        ],
    );
    assert_eq!(out, "rows 2\napple qty 9\nrows 1\n");
}

#[test]
fn swing_table_cell_edit_updates_the_model() {
    // A committed cell edit arrives as "edit:<row>,<col>,<value>" on the table's
    // field line, under the "__edit" sentinel so no selection listener fires;
    // __setFromHost routes it to setValueAt. Ids: frame c0, table c1, button c2.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static JTable table;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Grid");
                String[] cols = {"Item", "Qty"};
                Object[][] data = {{"Apples", "3"}};
                table = new JTable(new DefaultTableModel(data, cols));
                JButton show = new JButton("Show");
                show.addActionListener(e -> System.out.println("qty " + Main.table.getValueAt(0, 1)));
                frame.add(table);
                frame.add(show);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        // Edit cell (0,1) to "10" (no listener), then click Show to read it back.
        vec![
            Some(String::from("__edit\nc1=edit:0,1,10")),
            Some(String::from("c2")),
        ],
    );
    assert_eq!(out, "qty 10\n");
}

#[test]
fn swing_box_layout_stacks_with_struts() {
    // A Y_AXIS BoxLayout serializes as "box 1"; Box struts/glue become spacers.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Stack");
                JPanel panel = new JPanel();
                panel.setLayout(new BoxLayout(panel, BoxLayout.Y_AXIS));
                panel.add(new JButton("Top"));
                panel.add(Box.createVerticalStrut(12));
                panel.add(new JButton("Bottom"));
                panel.add(Box.createVerticalGlue());
                frame.add(panel);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""layout":"box 1""#),
        "no box layout: {json}"
    );
    assert!(
        json.contains(r#""type":"strut","w":0,"h":12,"glue":false"#),
        "no strut: {json}"
    );
    assert!(
        json.contains(r#""type":"strut","w":0,"h":0,"glue":true"#),
        "no glue: {json}"
    );
}

#[test]
fn swing_progress_bar_serializes_value_and_string() {
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Loading");
                JProgressBar bar = new JProgressBar(0, 200);
                bar.setValue(50);
                bar.setStringPainted(true);
                frame.add(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""type":"progressbar","min":0,"max":200,"value":50,"indeterminate":false"#),
        "no progressbar: {json}"
    );
    // 50 of 0..200 is 25%.
    assert!(
        json.contains(r#""stringPainted":true,"string":"25%""#),
        "no string: {json}"
    );
}

#[test]
fn swing_spinner_change_fires_and_reads_value() {
    // A JSpinner is a numeric field: changing it syncs the value and fires the
    // ChangeListener; getValue() returns a boxed Integer. Ids: frame c0,
    // spinner c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        public class Main {
            static JSpinner spinner;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Pick");
                spinner = new JSpinner(new SpinnerNumberModel(3, 0, 10, 1));
                spinner.addChangeListener(e -> {
                    int v = (Integer) Main.spinner.getValue();
                    System.out.println("value " + v);
                });
                frame.add(spinner);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\nc1=7")),
            Some(String::from("c1\nc1=2")),
        ],
    );
    assert_eq!(out, "value 7\nvalue 2\n");
}

#[test]
fn swing_custom_painting_records_graphics_commands() {
    // A JPanel subclass overriding paintComponent draws into a Graphics
    // recorder during serialization; the commands (and the panel size) land
    // in swing.json for the canvas renderer to replay.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Art");
                Art art = new Art();
                art.setPreferredSize(new Dimension(100, 80));
                frame.add(art);
                frame.setVisible(true);
            }
        }
        class Art extends JPanel {
            public void paintComponent(Graphics g) {
                super.paintComponent(g);
                g.setColor(new Color(255, 0, 0));
                g.fillRect(10, 10, 50, 30);
                g.setColor(Color.BLACK);
                g.drawString("hi", 12, 25);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""pw":100,"ph":80"#),
        "no panel size: {json}"
    );
    assert!(json.contains("setColor 255 0 0"), "no fill color: {json}");
    assert!(json.contains("fillRect 10 10 50 30"), "no rect: {json}");
    assert!(
        json.contains(r#"drawString \"hi\" 12 25"#),
        "no text: {json}"
    );
}

#[test]
fn swing_graphics_records_font_polygon_and_arc() {
    // The richer Graphics ops serialize into swing.json for the canvas
    // renderer: a set font (style/size/family), a filled polygon (flat x,y
    // pairs), and a pie-slice arc (bounds + angles).
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Draw");
                Art art = new Art();
                art.setPreferredSize(new Dimension(200, 160));
                frame.add(art);
                frame.setVisible(true);
            }
        }
        class Art extends JPanel {
            public void paintComponent(Graphics g) {
                super.paintComponent(g);
                int[] xs = {60, 20, 100};
                int[] ys = {40, 110, 110};
                g.setColor(new Color(80, 200, 255));
                g.fillPolygon(xs, ys, 3);
                g.setColor(new Color(255, 180, 60));
                g.fillArc(120, 40, 60, 60, 30, 220);
                g.setFont(new Font("SansSerif", Font.BOLD, 26));
                g.drawString("Hi", 40, 150);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains("fillPolygon 60 40 20 110 100 110"),
        "no polygon: {json}"
    );
    assert!(
        json.contains("fillArc 120 40 60 60 30 220"),
        "no arc: {json}"
    );
    assert!(json.contains("setFont 1 26 SansSerif"), "no font: {json}");
}

#[test]
fn swing_tabbed_pane_switches_tabs_and_fires_change_listener() {
    // A JTabbedPane serializes its tabs (title + component) and selected index.
    // Selecting a tab reports its index; the ChangeListener reads the new
    // selection via getSelectedIndex(). Ids: frame c0, tabbedpane c1, then the
    // tab components (labels c2, c3, c4).
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        public class Main {
            static JTabbedPane tabs;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Tabs");
                tabs = new JTabbedPane();
                tabs.addTab("One", new JLabel("first"));
                tabs.addTab("Two", new JLabel("second"));
                tabs.addTab("Three", new JLabel("third"));
                tabs.addChangeListener(e -> {
                    System.out.println("tab " + Main.tabs.getSelectedIndex()
                        + " = " + Main.tabs.getTitleAt(Main.tabs.getSelectedIndex()));
                });
                frame.add(tabs);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\nc1=2")),
            Some(String::from("c1\nc1=0")),
        ],
    );
    assert_eq!(out, "tab 2 = Three\ntab 0 = One\n");
}

#[test]
fn swing_request_focus_serializes_a_focus_request() {
    // requestFocus() records a one-shot focus request ("<cid>:<seq>") on the
    // tree; the renderer moves keyboard focus there. Ids: frame c0, name c1,
    // email c2 — focusing email yields "c2:1".
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Form");
                JTextField name = new JTextField(12);
                JTextField email = new JTextField(12);
                frame.add(name);
                frame.add(email);
                email.requestFocus();
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""focus":"c2:1""#),
        "no focus request: {json}"
    );
}

#[test]
fn swing_action_set_enabled_propagates_to_its_components() {
    // action.setEnabled(false) disables every component built from it — even
    // after they exist (their enabled tracks the action live).
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                Action save = new AbstractAction("Save") {
                    public void actionPerformed(ActionEvent e) {}
                };
                JButton button = new JButton(save);
                JMenuItem item = new JMenuItem(save);
                System.out.println("before " + button.isEnabled() + " " + item.isEnabled());
                save.setEnabled(false);
                System.out.println("after " + button.isEnabled() + " " + item.isEnabled());
                frame.add(button);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![],
    );
    assert_eq!(out, "before true true\nafter false false\n");
}

#[test]
fn swing_abstract_action_shared_by_button_and_menu_item() {
    // One AbstractAction drives both a JButton and a JMenuItem: each takes its
    // text/tooltip from the action, and activating either fires the same
    // actionPerformed. Ids: frame c0, button c1, menu c2, item c3.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                Action save = new AbstractAction("Save") {
                    public void actionPerformed(ActionEvent e) {
                        System.out.println("saved via " + e.getActionCommand());
                    }
                };
                save.putValue(Action.SHORT_DESCRIPTION, "Save the document");
                JButton button = new JButton(save);
                System.out.println("button text=" + button.getText());
                JMenuBar bar = new JMenuBar();
                JMenu file = new JMenu("File");
                file.add(new JMenuItem(save));
                bar.add(file);
                frame.setJMenuBar(bar);
                frame.add(button);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1")), // click the button
            Some(String::from("c3")), // click the menu item
        ],
    );
    assert_eq!(out, "button text=Save\nsaved via Save\nsaved via Save\n");
}

#[test]
fn swing_default_tree_model_fires_tree_model_listener_events() {
    // Mutating a DefaultTreeModel notifies its TreeModelListener with the parent
    // path, the child index, and the child itself.
    let (result, console) = compile_and_run(
        r#"
        import javax.swing.*;
        import javax.swing.event.*;
        public class Main {
            public static void main(String[] args) {
                DefaultMutableTreeNode root = new DefaultMutableTreeNode("Colors");
                DefaultMutableTreeNode warm = new DefaultMutableTreeNode("Warm");
                root.add(warm);
                DefaultTreeModel model = new DefaultTreeModel(root);
                model.addTreeModelListener(new TreeModelListener() {
                    public void treeNodesChanged(TreeModelEvent e) {
                        System.out.println("changed " + e.getTreePath()
                            + " at " + e.getChildIndices()[0]);
                    }
                    public void treeNodesInserted(TreeModelEvent e) {
                        System.out.println("inserted " + e.getChildren()[0]
                            + " at " + e.getChildIndices()[0] + " under " + e.getTreePath());
                    }
                    public void treeNodesRemoved(TreeModelEvent e) {
                        System.out.println("removed " + e.getChildren()[0]);
                    }
                    public void treeStructureChanged(TreeModelEvent e) {
                        System.out.println("structure " + e.getTreePath());
                    }
                });

                DefaultMutableTreeNode red = new DefaultMutableTreeNode("Red");
                model.insertNodeInto(red, warm, 0);
                System.out.println("count=" + model.getChildCount(warm)
                    + " leaf=" + model.isLeaf(red));
                model.valueForPathChanged(new TreePath(red.getPath()), "Crimson");
                System.out.println("now " + model.getChild(warm, 0));
                model.removeNodeFromParent(red);
                System.out.println("count=" + model.getChildCount(warm));
                model.reload();

                // A JTree accepts any TreeModel, not just a bare root node.
                JTree tree = new JTree(model);
                System.out.println("root=" + tree.getModel().getRoot());
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "inserted Red at 0 under [Colors, Warm]\n\
         count=1 leaf=true\n\
         changed [Colors, Warm] at 0\n\
         now Crimson\n\
         removed Crimson\n\
         count=0\n\
         structure [Colors]\n\
         root=Colors\n"
    );
}

#[test]
fn swing_tree_renders_a_custom_tree_model() {
    // A custom TreeModel over plain Strings — nodes need not be
    // DefaultMutableTreeNodes, and getChild may build them on the fly.
    let tree = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        class NumbersModel implements TreeModel {
            public Object getRoot() { return "Numbers"; }
            public Object getChild(Object parent, int index) { return "" + (index + 1); }
            public int getChildCount(Object parent) { return getRoot().equals(parent) ? 3 : 0; }
            public boolean isLeaf(Object node) { return !getRoot().equals(node); }
            public int getIndexOfChild(Object parent, Object child) { return -1; }
            public void valueForPathChanged(TreePath path, Object newValue) {}
            public void addTreeModelListener(TreeModelListener l) {}
            public void removeTreeModelListener(TreeModelListener l) {}
        }
        public class Main {
            public static void main(String[] args) {
                JTree tree = new JTree(new NumbersModel());
                JFrame frame = new JFrame("Numbers");
                frame.add(tree);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        tree.contains(r#""text":"Numbers","leaf":false,"expanded":true"#),
        "{tree}"
    );
    assert!(tree.contains(r#""text":"1","leaf":true"#), "{tree}");
    assert!(tree.contains(r#""text":"3","leaf":true"#), "{tree}");
    // getChild builds a fresh String each call, but equals-matching keeps one
    // stable id per node rather than minting a new one every render.
    assert!(tree.contains(r#""id":"n1""#), "{tree}");
    assert!(tree.contains(r#""id":"n4""#), "{tree}");
    assert!(!tree.contains(r#""id":"n5""#), "{tree}");
}

#[test]
fn swing_tree_cell_renderer_styles_nodes() {
    // A TreeCellRenderer decides each node's text and colour, and is told the
    // node's VISIBLE row (collapsed children are neither drawn nor numbered).
    let tree = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                DefaultMutableTreeNode root = new DefaultMutableTreeNode("Colors");
                DefaultMutableTreeNode warm = new DefaultMutableTreeNode("Warm");
                warm.add(new DefaultMutableTreeNode("Red"));
                DefaultMutableTreeNode cool = new DefaultMutableTreeNode("Cool");
                cool.add(new DefaultMutableTreeNode("Blue"));
                root.add(warm);
                root.add(cool);

                JTree tree = new JTree(root);
                tree.expandPath(new TreePath(warm.getPath()));
                tree.setCellRenderer(new DefaultTreeCellRenderer() {
                    public Component getTreeCellRendererComponent(JTree tree, Object value,
                            boolean selected, boolean expanded, boolean leaf, int row,
                            boolean hasFocus) {
                        Component base = super.getTreeCellRendererComponent(
                            tree, value, selected, expanded, leaf, row, hasFocus);
                        JLabel label = (JLabel) base;
                        label.setText(row + ": " + value);
                        if (leaf) label.setForeground(new Color(0, 120, 0));
                        return label;
                    }
                });

                JFrame frame = new JFrame("Colors");
                frame.add(tree);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    // Visible rows: 0 Colors, 1 Warm, 2 Red, 3 Cool. Cool is collapsed, so Blue
    // is neither serialized nor numbered.
    assert!(tree.contains(r#""text":"0: Colors""#), "{tree}");
    assert!(tree.contains(r#""text":"1: Warm""#), "{tree}");
    assert!(tree.contains(r#""text":"2: Red""#), "{tree}");
    assert!(tree.contains(r#""text":"3: Cool""#), "{tree}");
    assert!(!tree.contains("Blue"), "{tree}");
    // Only the leaf is green — the shared instance resets colour between nodes,
    // so it doesn't leak onto Cool (drawn after Red).
    assert!(
        tree.contains(
            r#""text":"2: Red","leaf":true,"expanded":false,"selected":false,"fg":"0,120,0""#
        ),
        "{tree}"
    );
    assert!(
        tree.contains(r#""text":"3: Cool","leaf":false,"expanded":false,"selected":false}"#),
        "{tree}"
    );
}

#[test]
fn swing_tree_selects_nodes_and_tracks_expansion() {
    // A JTree over DefaultMutableTreeNodes. Node ids are assigned in creation
    // order (root n1, warm n2, red n3). Selecting fires the listener with a
    // TreePath; the "__tree" sentinel expands without firing it. Ids: frame c0,
    // tree c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        public class Main {
            static JTree tree;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Colors");
                DefaultMutableTreeNode root = new DefaultMutableTreeNode("Colors");
                DefaultMutableTreeNode warm = new DefaultMutableTreeNode("Warm");
                DefaultMutableTreeNode red = new DefaultMutableTreeNode("Red");
                warm.add(red);
                root.add(warm);
                tree = new JTree(root);
                tree.addTreeSelectionListener(e -> {
                    DefaultMutableTreeNode node =
                        (DefaultMutableTreeNode) Main.tree.getLastSelectedPathComponent();
                    System.out.println("picked " + node.getUserObject()
                        + " leaf=" + node.isLeaf()
                        + " level=" + node.getLevel()
                        + " path=" + Main.tree.getSelectionPath());
                });
                System.out.println("warm expanded=" + tree.isExpanded(new TreePath(warm.getPath())));
                frame.add(tree);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("__tree\nc1=exp:n2")), // expand Warm, no listener
            Some(String::from("c1\nc1=sel:n3")),     // select Red
            Some(String::from("c1\nc1=sel:n1")),     // select the root
        ],
    );
    assert_eq!(
        out,
        "warm expanded=false\n\
         picked Red leaf=true level=2 path=[Colors, Warm, Red]\n\
         picked Colors leaf=false level=0 path=[Colors]\n"
    );
}

#[test]
fn swing_table_sorts_rows_and_converts_view_indices() {
    // Clicking a header cycles ascending -> descending -> unsorted. Row indices
    // are VIEW indices; convertRowIndexToModel maps back. Ids: frame c0, table
    // c1, button c2. Scores sort numerically ("10" after "9"), not as text.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static JTable table;
            static DefaultTableModel model;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Scores");
                String[] cols = {"Player", "Score"};
                Object[][] data = {{"Ada", "9"}, {"Bo", "10"}, {"Cy", "2"}};
                model = new DefaultTableModel(data, cols);
                table = new JTable(model);
                table.setAutoCreateRowSorter(true);
                JButton show = new JButton("Show");
                show.addActionListener(e -> {
                    StringBuilder s = new StringBuilder();
                    for (int r = 0; r < Main.table.getRowCount(); r++) {
                        if (r > 0) s.append(" ");
                        s.append(Main.table.getValueAt(r, 0));
                    }
                    System.out.println(s + " | row0 model=" + Main.table.convertRowIndexToModel(0));
                });
                frame.add(table);
                frame.add(show);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c2")),                // unsorted: model order
            Some(String::from("__sort\nc1=sort:1")), // click Score -> ascending
            Some(String::from("c2")),
            Some(String::from("__sort\nc1=sort:1")), // again -> descending
            Some(String::from("c2")),
            Some(String::from("__sort\nc1=sort:1")), // third -> unsorted
            Some(String::from("c2")),
        ],
    );
    assert_eq!(
        out,
        "Ada Bo Cy | row0 model=0\n\
         Cy Ada Bo | row0 model=2\n\
         Bo Ada Cy | row0 model=1\n\
         Ada Bo Cy | row0 model=0\n"
    );
}

#[test]
fn swing_table_cell_renderer_styles_a_column() {
    // Per-column renderers: one right-aligns the Score column (alignment set
    // once, and it sticks) and colours negative scores red.
    let tree = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                String[] cols = {"Player", "Score"};
                Object[][] data = {{"Ada", "10"}, {"Bo", "-3"}};
                DefaultTableModel model = new DefaultTableModel(data, cols);
                JTable table = new JTable(model);

                DefaultTableCellRenderer scores = new DefaultTableCellRenderer() {
                    public Component getTableCellRendererComponent(JTable table, Object value,
                            boolean isSelected, boolean hasFocus, int row, int column) {
                        Component base = super.getTableCellRendererComponent(
                            table, value, isSelected, hasFocus, row, column);
                        JLabel label = (JLabel) base;
                        if (("" + value).startsWith("-")) label.setForeground(new Color(200, 0, 0));
                        return label;
                    }
                };
                scores.setHorizontalAlignment(SwingConstants.RIGHT);
                table.getColumnModel().getColumn(1).setCellRenderer(scores);
                table.getColumnModel().getColumn(1).setHeaderValue("Points");

                JFrame frame = new JFrame("Scores");
                frame.add(table);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    // setHeaderValue overrides the model's column name.
    assert!(tree.contains(r#""headers":["Player","Points"]"#), "{tree}");
    // Column 0 has no renderer ({}); column 1 is right-aligned (RIGHT == 4),
    // and only the negative score is red — the shared instance resets colour
    // between cells but keeps the alignment.
    assert!(
        tree.contains(r#""cellStyles":[[{},{"halign":4}],[{},{"fg":"200,0,0","halign":4}]]"#),
        "{tree}"
    );
}

#[test]
fn swing_custom_abstract_table_model_subclass() {
    // A student subclasses AbstractTableModel, implementing only the three
    // abstract queries; the base supplies default column names, non-editable
    // cells, listener management, and the fireTableXxx notifications.
    let (result, console) = compile_and_run(
        r#"
        import javax.swing.*;
        import javax.swing.event.*;
        class TimesModel extends AbstractTableModel {
            public int getRowCount() { return 3; }
            public int getColumnCount() { return 3; }
            public Object getValueAt(int r, int c) { return (r + 1) * (c + 1); }
        }
        public class Main {
            public static void main(String[] args) {
                TimesModel model = new TimesModel();
                System.out.println("rows=" + model.getRowCount() + " cols=" + model.getColumnCount());
                System.out.println("v(2,2)=" + model.getValueAt(2, 2));
                System.out.println("colName1=" + model.getColumnName(1));
                System.out.println("editable=" + model.isCellEditable(0, 0));
                model.addTableModelListener(new TableModelListener() {
                    public void tableChanged(TableModelEvent e) {
                        System.out.println("fired type=" + e.getType()
                            + " rows=" + e.getFirstRow() + ".." + e.getLastRow());
                    }
                });
                model.fireTableDataChanged();
                model.fireTableRowsInserted(1, 1);
                // A JTable accepts any TableModel, not just DefaultTableModel.
                JTable table = new JTable(model);
                System.out.println("table cell=" + table.getValueAt(1, 2));
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "rows=3 cols=3\nv(2,2)=9\ncolName1=B\neditable=false\nfired type=0 rows=0..2147483647\nfired type=1 rows=1..1\ntable cell=6\n"
    );
}

#[test]
fn swing_table_model_fires_table_model_listener_events() {
    // A DefaultTableModel notifies its TableModelListener on every mutation with
    // the right type (INSERT=1, UPDATE=0, DELETE=-1), row range, and column.
    let (result, console) = compile_and_run(
        r#"
        import javax.swing.*;
        import javax.swing.event.*;
        public class Main {
            public static void main(String[] args) {
                String[] cols = {"Name", "Score"};
                Object[][] data = {{"Ada", "10"}};
                DefaultTableModel model = new DefaultTableModel(data, cols);
                model.addTableModelListener(new TableModelListener() {
                    public void tableChanged(TableModelEvent e) {
                        System.out.println("type=" + e.getType()
                            + " rows=" + e.getFirstRow() + ".." + e.getLastRow()
                            + " col=" + e.getColumn());
                    }
                });
                model.addRow(new Object[]{"Bo", "7"});
                model.setValueAt("99", 0, 1);
                model.removeRow(0);
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "type=1 rows=1..1 col=-1\ntype=0 rows=0..0 col=1\ntype=-1 rows=0..0 col=-1\n"
    );
}

/// `new Foo();` is a statement expression (JLS 14.8): the constructor runs and
/// the reference is discarded. Array creation is not one, and stays an error.
/// Cross-checked against a real JDK by `diff_bare_new_statement`.
#[test]
fn bare_new_runs_the_constructor_and_discards_the_reference() {
    let out = run_stdout(
        r#"
        class Counter {
            static int made = 0;
            Counter() { made++; System.out.println("ctor " + made); }
        }
        public class Main {
            public static void main(String[] args) {
                new Counter();
                new Counter();
                for (int i = 0; i < 2; i++) new Counter();
                // A bundled library type works the same way.
                new StringBuilder("dropped");
                System.out.println("made=" + Counter.made);
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "ctor 1\nctor 2\nctor 3\nctor 4\nmade=4\n");
}

/// C-style declarators (`String args[]`), which javac accepts. The brackets
/// bind to the NAME, so in `int a[], b;` only `a` is an array — the asymmetry
/// this test exists to pin. Cross-checked against a real JDK by
/// `diff_c_style_array_declarators`.
#[test]
fn c_style_array_declarators_bind_to_the_name() {
    let out = run_stdout(
        r#"
        public class Main {
            private int f[] = {1, 2};
            static int g[], h;
            static int sum(int a[], int b) { return a[0] + b; }
            public static void main(String args[]) {
                int x[] = {5, 6};
                int y[], z;
                y = new int[] {7};
                z = 9;
                g = new int[] {4};
                h = 1;
                int grid[][] = new int[2][3];
                grid[1][2] = 8;
                System.out.println(args.length + " " + x[1] + " " + y[0] + " " + z);
                System.out.println(g[0] + " " + h + " " + sum(x, 3) + " " + new Main().f[1]);
                System.out.println(grid[1][2] + " " + grid.length + " " + grid[0].length);
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "0 6 7 9\n4 1 8 2\n8 2 3\n");
}

/// Regression: `type_of` reported `int` for shifts and for `long`-typed
/// `& | ^`, while the emitter produced a long — so `(x << 4) + y` emitted IADD
/// over a long value and failed verification. Every expected value below comes
/// from running the same program under a real JDK 11.
#[test]
fn long_bit_ops_and_shifts_promote_like_javac() {
    let out = run_stdout(
        r#"
        public class Main {
            static int f() { return 3; }
            static long g() { return 5L; }
            public static void main(String[] args) {
                long x = 5L; int b = 2;
                System.out.println(x + b);
                System.out.println(((long) f() << 4) + f());   // shift keeps the left type
                System.out.println((g() & 0xFFL) + f());       // long & long stays long
                System.out.println((g() | 0L) + 1);
                System.out.println((g() ^ 1L) * 2);
                System.out.println((f() << 2) + f());          // an int shift stays int
                System.out.println((f() & 1) + 1);
                System.out.println((int) ((g() >>> 1) + 1));
                System.out.println(((long) f() << 32) >> 32);
                boolean p = true, q = false;
                System.out.println((p & q) + " " + (p | q) + " " + (p ^ q));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "7\n51\n8\n6\n8\n15\n2\n3\n3\nfalse true true\n");
}

/// Every expected value below was produced by running the same program under a
/// real JDK 11 (`java.util.Random`), not by re-deriving them from our own
/// implementation — a seeded Random must replay the JVM's exact sequence.
#[test]
fn random_seeded_matches_the_jdk_sequence() {
    let out = run_stdout(
        r#"
        import java.util.Random;
        public class Main {
            public static void main(String[] args) {
                Random r = new Random(42);
                System.out.println(r.nextInt() + " " + r.nextInt());
                r = new Random(42);
                System.out.println(r.nextInt(100) + " " + r.nextInt(100) + " " + r.nextInt(100));
                // A power-of-two bound takes a different branch than the
                // rejection loop, so pin both.
                r = new Random(42);
                System.out.println(r.nextInt(64) + " " + r.nextInt(64));
                r = new Random(0);
                System.out.println(r.nextInt());
                r = new Random(123456789L);
                System.out.println(r.nextInt(10) + " " + r.nextInt(10)
                    + " " + r.nextInt(10) + " " + r.nextInt(10));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(
        out,
        "-1170105035 234785527\n\
         30 63 48\n\
         46 3\n\
         -1155484576\n\
         5 0 3 4\n"
    );
}

#[test]
fn random_seeded_matches_the_jdk_for_every_draw_kind() {
    let out = run_stdout(
        r#"
        import java.util.Random;
        public class Main {
            public static void main(String[] args) {
                Random r = new Random(42);
                System.out.println(r.nextDouble() + " " + r.nextDouble());
                r = new Random(42);
                System.out.println(r.nextBoolean() + " " + r.nextBoolean() + " " + r.nextBoolean());
                r = new Random(42);
                System.out.println(r.nextLong());
                r = new Random(42);
                System.out.println(r.nextFloat());
                // nextGaussian goes through Math.log, whose last ulp can differ
                // from the JDK's StrictMath — a deviation LANGUAGE.md records
                // for every transcendental — so compare to 9 places. Real Java
                // prints 1.1419053154730547 and 0.9194079489827879. The second
                // draw comes from the polar method's cache, so both are pinned.
                r = new Random(42);
                System.out.println(String.format("%.9f %.9f", r.nextGaussian(), r.nextGaussian()));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(
        out,
        "0.7275636800328681 0.6832234717598454\n\
         true false true\n\
         -5025562857975149833\n\
         0.7275637\n\
         1.141905315 0.919407949\n"
    );
}

#[test]
fn random_same_seed_replays_and_set_seed_resets() {
    // The bug this fixes: `new Random(seed)` used to ignore the seed entirely,
    // so two Randoms with the same seed diverged and nothing was reproducible.
    let out = run_stdout(
        r#"
        import java.util.Random;
        public class Main {
            public static void main(String[] args) {
                Random a = new Random(7);
                Random b = new Random(7);
                boolean same = true;
                for (int i = 0; i < 50; i++) {
                    if (a.nextInt(1000) != b.nextInt(1000)) same = false;
                }
                System.out.println("same=" + same);

                Random r = new Random(42);
                int first = r.nextInt();
                r.nextInt();
                r.setSeed(42);
                System.out.println("reset=" + (r.nextInt() == first));

                // Bounds stay in range across the rejection loop.
                Random g = new Random(1);
                boolean inRange = true;
                for (int i = 0; i < 2000; i++) {
                    int v = g.nextInt(7);
                    if (v < 0 || v >= 7) inRange = false;
                }
                System.out.println("inRange=" + inRange);

                try {
                    g.nextInt(0);
                    System.out.println("no throw");
                } catch (IllegalArgumentException e) {
                    System.out.println("caught: " + e.getMessage());
                }
            }
        }
        "#,
        "Main",
    );
    assert_eq!(
        out,
        "same=true\nreset=true\ninRange=true\ncaught: bound must be positive\n"
    );
}

#[test]
fn swing_list_cell_renderer_styles_each_row() {
    // A ListCellRenderer decides each row's text and colours. The renderer is
    // asked once per row at paint time, with the raw value and its index.
    let tree = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                String[] tasks = {"buy milk", "!urgent call", "walk dog"};
                JList list = new JList(tasks);
                list.setCellRenderer(new DefaultListCellRenderer() {
                    public Component getListCellRendererComponent(JList list, Object value,
                            int index, boolean isSelected, boolean cellHasFocus) {
                        JLabel label = (JLabel) super.getListCellRendererComponent(
                            list, value, index, isSelected, cellHasFocus);
                        String text = "" + value;
                        if (text.startsWith("!")) {
                            label.setText((index + 1) + ". " + text.substring(1).toUpperCase());
                            label.setForeground(new Color(200, 0, 0));
                        } else {
                            label.setText((index + 1) + ". " + text);
                        }
                        return label;
                    }
                });
                JFrame frame = new JFrame("Tasks");
                frame.add(list);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    // Renderer-produced text, numbered per index.
    assert!(tree.contains(r#""1. buy milk""#), "{tree}");
    assert!(tree.contains(r#""2. URGENT CALL""#), "{tree}");
    assert!(tree.contains(r#""3. walk dog""#), "{tree}");
    // Only the urgent row carries a colour — the shared renderer instance is
    // reset between rows, so red does not leak onto row 3.
    assert!(
        tree.contains(r#""itemStyles":[{},{"fg":"200,0,0"},{}]"#),
        "{tree}"
    );
}

#[test]
fn swing_custom_abstract_list_model_subclass() {
    // A student subclasses AbstractListModel, implementing only getSize and
    // getElementAt; the base supplies listener management and the protected
    // fireXxx notifications. A JList accepts any ListModel.
    let (result, console) = compile_and_run(
        r#"
        import javax.swing.*;
        import javax.swing.event.*;
        import java.util.ArrayList;
        class SquaresModel extends AbstractListModel {
            ArrayList<Integer> values = new ArrayList<Integer>();
            public int getSize() { return values.size(); }
            public Object getElementAt(int index) { return values.get(index); }
            public void addSquare(int n) {
                values.add(n * n);
                fireIntervalAdded(this, values.size() - 1, values.size() - 1);
            }
        }
        public class Main {
            public static void main(String[] args) {
                SquaresModel model = new SquaresModel();
                model.addListDataListener(new ListDataListener() {
                    public void intervalAdded(ListDataEvent e) {
                        System.out.println("added [" + e.getIndex0() + "] size=" + e.getSource());
                    }
                    public void intervalRemoved(ListDataEvent e) {}
                    public void contentsChanged(ListDataEvent e) {}
                });
                model.addSquare(3);
                model.addSquare(4);
                System.out.println("size=" + model.getSize() + " at1=" + model.getElementAt(1));
                // A JList reads any ListModel through getSize/getElementAt.
                JList list = new JList(model);
                list.setSelectedIndex(1);
                System.out.println("selected=" + list.getSelectedValue());
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    let out = console.stdout_text();
    assert!(out.contains("added [0]"), "{out}");
    assert!(out.contains("added [1]"), "{out}");
    assert!(out.contains("size=2 at1=16"), "{out}");
    assert!(out.contains("selected=16"), "{out}");
}

#[test]
fn swing_list_model_fires_data_listener_events() {
    // A DefaultListModel notifies its ListDataListener on every mutation, with
    // the right event type and interval — synchronously, like AbstractListModel.
    let (result, console) = compile_and_run(
        r#"
        import javax.swing.*;
        import javax.swing.event.*;
        public class Main {
            public static void main(String[] args) {
                DefaultListModel model = new DefaultListModel();
                model.addListDataListener(new ListDataListener() {
                    public void intervalAdded(ListDataEvent e) {
                        System.out.println("added [" + e.getIndex0() + "," + e.getIndex1()
                            + "] size=" + ((DefaultListModel) e.getSource()).getSize());
                    }
                    public void intervalRemoved(ListDataEvent e) {
                        System.out.println("removed [" + e.getIndex0() + "," + e.getIndex1() + "]");
                    }
                    public void contentsChanged(ListDataEvent e) {
                        System.out.println("changed [" + e.getIndex0() + "]");
                    }
                });
                model.addElement("a");
                model.addElement("b");
                model.add(1, "x");
                model.set(0, "A");
                model.remove(2);
                model.removeAllElements();
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "added [0,0] size=1\nadded [1,1] size=2\nadded [1,1] size=3\nchanged [0]\nremoved [2,2]\nremoved [0,1]\n"
    );
}

#[test]
fn swing_text_field_reads_the_users_live_caret_and_selection() {
    // The host reports the real cursor with each event ("__caret:<cid>=s,e"),
    // so the listener sees what the user actually has selected. Ids: frame c0,
    // field c1, button c2.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static JTextField field;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Caret");
                field = new JTextField("hello world", 20);
                JButton show = new JButton("Show");
                show.addActionListener(e -> {
                    System.out.println("caret=" + Main.field.getCaretPosition()
                        + " sel=[" + Main.field.getSelectionStart()
                        + "," + Main.field.getSelectionEnd() + "]"
                        + " text=" + Main.field.getSelectedText());
                });
                frame.add(field);
                frame.add(show);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            // The user selected "world" (offsets 6..11) and clicked Show.
            Some(String::from("c2\nc1=hello world\n__caret:c1=6,11")),
            // Now just a caret at 5, nothing selected.
            Some(String::from("c2\nc1=hello world\n__caret:c1=5,5")),
        ],
    );
    assert_eq!(
        out,
        "caret=11 sel=[6,11] text=world\ncaret=5 sel=[5,5] text=null\n"
    );
}

#[test]
fn swing_text_component_select_all_requests_a_caret_move() {
    // Programmatic select/selectAll asks the host to move the real caret; the
    // request is serialized once (and then cleared, so it can't fight the user).
    let tree = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Caret");
                JTextField field = new JTextField("hello", 10);
                field.selectAll();
                JTextArea area = new JTextArea("abc", 3, 10);
                area.setCaretPosition(2);
                frame.add(field);
                frame.add(area);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(tree.contains(r#""caretReq":"0,5""#), "{tree}");
    assert!(tree.contains(r#""caretReq":"2,2""#), "{tree}");
}

#[test]
fn swing_text_area_text_operations() {
    // JTextArea's text ops: line queries, offsets, insert, replaceRange, and a
    // bounds error raising the checked BadLocationException (like the JDK).
    let (result, console) = compile_and_run(
        r#"
        import javax.swing.*;
        import javax.swing.text.BadLocationException;
        public class Main {
            public static void main(String[] args) throws BadLocationException {
                JTextArea area = new JTextArea("hello\nworld");
                System.out.println("lines=" + area.getLineCount());
                System.out.println("start1=" + area.getLineStartOffset(1));
                System.out.println("end0=" + area.getLineEndOffset(0));
                System.out.println("lineOf8=" + area.getLineOfOffset(8));
                System.out.println("get=" + area.getText(0, 5));
                area.insert("!", 5);
                System.out.println("afterInsert=" + area.getText(0, 6));
                area.replaceRange("Earth", 7, 12);
                System.out.println("full=" + area.getText().replace("\n", "|"));
                try {
                    area.getLineStartOffset(99);
                } catch (BadLocationException e) {
                    System.out.println("bad=" + e.getMessage());
                }
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "lines=2\nstart1=6\nend0=6\nlineOf8=1\nget=hello\nafterInsert=hello!\nfull=hello!|Earth\nbad=No such line\n"
    );
}

#[test]
fn swing_combo_box_is_backed_by_a_default_combo_box_model() {
    // JComboBox is model-backed: a DefaultComboBoxModel drives its items and
    // selection, fires ListDataEvents, and moves the selection when the
    // selected element is removed (like real Swing).
    let (result, console) = compile_and_run(
        r#"
        import javax.swing.*;
        import javax.swing.event.*;
        public class Main {
            public static void main(String[] args) {
                String[] fruits = {"Apple", "Banana"};
                DefaultComboBoxModel model = new DefaultComboBoxModel(fruits);
                JComboBox combo = new JComboBox(model);
                model.addListDataListener(new ListDataListener() {
                    public void intervalAdded(ListDataEvent e) {
                        System.out.println("added [" + e.getIndex0() + "]");
                    }
                    public void intervalRemoved(ListDataEvent e) {
                        System.out.println("removed [" + e.getIndex0() + "]");
                    }
                    public void contentsChanged(ListDataEvent e) {
                        System.out.println("selection -> " + Main.name(e.getSource()));
                    }
                });
                System.out.println("selected=" + combo.getSelectedItem()
                    + " idx=" + combo.getSelectedIndex());
                // addItem goes through the mutable model.
                combo.addItem("Cherry");
                System.out.println("count=" + combo.getItemCount() + " at2=" + combo.getItemAt(2));
                // Removing the selected element (Apple) reselects a neighbour.
                combo.removeItem("Apple");
                System.out.println("after remove: " + combo.getSelectedItem()
                    + " idx=" + combo.getSelectedIndex());
                // The combo and the model agree on the selection.
                combo.setSelectedIndex(1);
                System.out.println("model says " + model.getSelectedItem());
            }
            static String name(Object source) {
                return ((DefaultComboBoxModel) source).getSelectedItem() + "";
            }
        }
        "#,
        "Main",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "selected=Apple idx=0\nadded [2]\ncount=3 at2=Cherry\nremoved [0]\nafter remove: Banana idx=0\nselection -> Cherry\nmodel says Cherry\n"
    );
}

#[test]
fn swing_editable_combo_box_custom_values_and_item_management() {
    // An editable JComboBox: runtime addItem/removeItem, and getSelectedItem
    // returns the current text — a listed value (with its index) or a custom
    // typed one (index -1). Ids: frame c0, combo c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static JComboBox combo;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Combo");
                String[] fruits = {"Apple", "Banana"};
                combo = new JComboBox(fruits);
                combo.setEditable(true);
                combo.addItem("Cherry");
                combo.removeItem("Apple");
                combo.addActionListener(e ->
                    System.out.println("chose " + Main.combo.getSelectedItem()
                        + " idx=" + Main.combo.getSelectedIndex()));
                System.out.println("count=" + combo.getItemCount());
                frame.add(combo);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\nc1=Banana")),      // picked a listed value
            Some(String::from("c1\nc1=Dragonfruit")), // typed a custom value
        ],
    );
    assert_eq!(
        out,
        "count=2\nchose Banana idx=0\nchose Dragonfruit idx=-1\n"
    );
}

#[test]
fn swing_document_listener_fires_insert_and_remove() {
    // getDocument().addDocumentListener fires per edit: insertUpdate when the
    // text grows, removeUpdate when it shrinks. The host marks each with a
    // "__doc=" line. Ids: frame c0, field c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        public class Main {
            static JTextField field;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Live");
                field = new JTextField(16);
                field.getDocument().addDocumentListener(new DocumentListener() {
                    public void insertUpdate(DocumentEvent e) {
                        System.out.println("insert len=" + Main.field.getText().length());
                    }
                    public void removeUpdate(DocumentEvent e) {
                        System.out.println("remove len=" + Main.field.getText().length());
                    }
                    public void changedUpdate(DocumentEvent e) {}
                });
                frame.add(field);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\nc1=ab\n__doc=c1")), // typed "ab" (insert)
            Some(String::from("c1\nc1=abc\n__doc=c1")), // typed "c" (insert)
            Some(String::from("c1\nc1=ab\n__doc=c1")), // deleted (remove)
        ],
    );
    assert_eq!(out, "insert len=2\ninsert len=3\nremove len=2\n");
}

#[test]
fn swing_text_field_action_listener_fires_on_enter() {
    // A JTextField with an ActionListener fires on Enter; the listener reads the
    // typed value (synced from the host before dispatch). Ids: frame c0, field
    // c1. The host reports the field's value with the activation ("c1\nc1=...").
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            static JTextField search;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Search");
                search = new JTextField(16);
                search.addActionListener(e -> System.out.println("search: " + Main.search.getText()));
                frame.add(search);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("c1\nc1=hello world"))],
    );
    assert_eq!(out, "search: hello world\n");
}

#[test]
fn swing_label_displayed_mnemonic_serializes() {
    // JLabel.setDisplayedMnemonic + setLabelFor: the label carries its mnemonic
    // and the target field's id, so the renderer can underline the letter and
    // wire Alt+letter to focus the field. Ids: frame c0, field c1, label c2.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Form");
                JTextField name = new JTextField(12);
                JLabel label = new JLabel("Name:");
                label.setLabelFor(name);
                label.setDisplayedMnemonic('N');
                frame.add(label);
                frame.add(name);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""type":"label","text":"Name:","for":"c1","mnemonic":"N""#),
        "no label mnemonic: {json}"
    );
}

#[test]
fn swing_accessible_context_names_and_describes_a_component() {
    // getAccessibleContext().setAccessibleName / setAccessibleDescription flow
    // to the tree as accName / accDesc (the renderer maps them to aria-*).
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.accessibility.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("A11y");
                JSlider volume = new JSlider(0, 11);
                AccessibleContext ctx = volume.getAccessibleContext();
                ctx.setAccessibleName("Volume");
                ctx.setAccessibleDescription("Playback level, 0 to 11");
                frame.add(volume);
                // The name round-trips through the same handle.
                System.out.println(volume.getAccessibleContext().getAccessibleName());
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""accName":"Volume""#),
        "no accessible name: {json}"
    );
    assert!(
        json.contains(r#""accDesc":"Playback level, 0 to 11""#),
        "no accessible description: {json}"
    );
}

#[test]
fn swing_menu_item_accelerator_serializes_shortcut_text() {
    // setAccelerator(KeyStroke) serializes a shortcut string used both as the
    // menu hint and the key the renderer matches. Both the modern *_DOWN_MASK
    // and the legacy ActionEvent.*_MASK forms render the same.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                JMenuBar bar = new JMenuBar();
                JMenu file = new JMenu("File");
                JMenuItem save = new JMenuItem("Save");
                save.setAccelerator(KeyStroke.getKeyStroke(KeyEvent.VK_S, InputEvent.CTRL_DOWN_MASK));
                file.add(save);
                JMenuItem saveAs = new JMenuItem("Save As");
                saveAs.setAccelerator(KeyStroke.getKeyStroke(KeyEvent.VK_S,
                    ActionEvent.CTRL_MASK | ActionEvent.SHIFT_MASK));
                file.add(saveAs);
                bar.add(file);
                frame.setJMenuBar(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""text":"Save","accel":"Ctrl+S""#),
        "no Ctrl+S: {json}"
    );
    assert!(
        json.contains(r#""text":"Save As","accel":"Ctrl+Shift+S""#),
        "no Ctrl+Shift+S: {json}"
    );
}

#[test]
fn swing_check_and_radio_menu_items_toggle_and_group() {
    // A JCheckBoxMenuItem toggles on each activation; JRadioButtonMenuItems in a
    // ButtonGroup are mutually exclusive (selecting one deselects the others).
    // Ids: frame c0, menu c1, wrap c2, light c3, dark c4.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            static JCheckBoxMenuItem wrap;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                JMenuBar bar = new JMenuBar();
                JMenu view = new JMenu("View");
                wrap = new JCheckBoxMenuItem("Word Wrap");
                wrap.addActionListener(e -> System.out.println("wrap " + Main.wrap.isSelected()));
                view.add(wrap);

                JRadioButtonMenuItem light = new JRadioButtonMenuItem("Light", true);
                JRadioButtonMenuItem dark = new JRadioButtonMenuItem("Dark");
                ButtonGroup group = new ButtonGroup();
                group.add(light);
                group.add(dark);
                light.addActionListener(e ->
                    System.out.println("light=" + light.isSelected() + " dark=" + dark.isSelected()));
                dark.addActionListener(e ->
                    System.out.println("light=" + light.isSelected() + " dark=" + dark.isSelected()));
                view.add(light);
                view.add(dark);

                bar.add(view);
                frame.setJMenuBar(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c2")), // toggle Word Wrap on
            Some(String::from("c2")), // toggle it off
            Some(String::from("c4")), // pick Dark
            Some(String::from("c3")), // pick Light
        ],
    );
    assert_eq!(
        out,
        "wrap true\nwrap false\nlight=false dark=true\nlight=true dark=false\n"
    );
}

#[test]
fn swing_check_menu_item_serializes_selected_state() {
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                JMenuBar bar = new JMenuBar();
                JMenu view = new JMenu("View");
                view.add(new JCheckBoxMenuItem("Word Wrap", true));
                view.add(new JRadioButtonMenuItem("Light", true));
                bar.add(view);
                frame.setJMenuBar(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""type":"checkmenuitem","text":"Word Wrap","selected":true"#),
        "no check item: {json}"
    );
    assert!(
        json.contains(r#""type":"radiomenuitem","text":"Light","selected":true"#),
        "no radio item: {json}"
    );
}

#[test]
fn swing_grid_bag_layout_serializes_per_child_constraints() {
    // A GridBagLayout serializes each child's GridBagConstraints. A reused
    // constraints object is COPIED on add, so each child keeps its own cell
    // (mutating gbc after the first add must not change the first child).
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Grid");
                frame.setLayout(new GridBagLayout());
                GridBagConstraints gbc = new GridBagConstraints();
                gbc.gridx = 0;
                gbc.gridy = 0;
                gbc.insets = new Insets(4, 4, 4, 4);
                frame.add(new JLabel("Name:"), gbc);
                gbc.gridx = 1;
                gbc.weightx = 1.0;
                gbc.fill = GridBagConstraints.HORIZONTAL;
                frame.add(new JTextField(10), gbc);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""layout":"gridbag""#),
        "not gridbag: {json}"
    );
    // First child: (0,0), no weight, no fill — its own copy (proving the add
    // took a copy, not a reference to the later-mutated gbc).
    assert!(
        json.contains(r#""gbc":{"gridx":0,"gridy":0,"gridwidth":1,"gridheight":1,"weightx":0.0,"weighty":0.0,"anchor":10,"fill":0,"insets":"4,4,4,4"}"#),
        "no label gbc (copy failed?): {json}"
    );
    // Second child: (1,0), weightx 1, HORIZONTAL fill.
    assert!(
        json.contains(r#""gridx":1,"gridy":0,"gridwidth":1,"gridheight":1,"weightx":1.0,"weighty":0.0,"anchor":10,"fill":2"#),
        "no field gbc: {json}"
    );
}

#[test]
fn swing_button_action_command_and_do_click() {
    // setActionCommand flows to ActionEvent.getActionCommand(); doClick() fires
    // the listener programmatically (before the event loop even starts).
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Cmd");
                JButton b = new JButton("Save");
                b.setActionCommand("SAVE_DOC");
                b.addActionListener(e -> System.out.println("cmd " + e.getActionCommand()));
                b.doClick();
                frame.add(b);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("c1"))],
    );
    assert_eq!(out, "cmd SAVE_DOC\ncmd SAVE_DOC\n");
}

#[test]
fn swing_toggle_button_flips_and_fires() {
    // A JToggleButton toggles its selected state on click and reports it; the
    // ItemListener sees the new state. Ids: frame c0, toggle c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            static JToggleButton bold;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Toggle");
                bold = new JToggleButton("Bold");
                bold.addItemListener(e -> System.out.println("bold " + Main.bold.isSelected()));
                frame.add(bold);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\nc1=true")),
            Some(String::from("c1\nc1=false")),
        ],
    );
    assert_eq!(out, "bold true\nbold false\n");
}

#[test]
fn swing_button_mnemonic_serializes() {
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Mnem");
                JButton b = new JButton("Save");
                b.setMnemonic(KeyEvent.VK_S);
                frame.add(b);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(json.contains(r#""mnemonic":"S""#), "no mnemonic: {json}");
}

#[test]
fn swing_password_field_and_editable() {
    // JPasswordField serializes as a masked field and getPassword() returns the
    // text as a char[]; JTextField.setEditable(false) marks it read-only.
    let json = run_swing(
        r#"
        import javax.swing.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Login");
                JPasswordField pw = new JPasswordField(12);
                pw.setText("secret");
                JTextField readonly = new JTextField("frozen");
                readonly.setEditable(false);
                frame.add(pw);
                frame.add(readonly);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""password":true"#),
        "no password flag: {json}"
    );
    assert!(
        json.contains(r#""text":"frozen","columns":0,"editable":false"#),
        "not read-only: {json}"
    );
}

#[test]
fn swing_password_field_get_password_returns_chars() {
    // getPassword() -> char[]; String(char[]) reconstructs it.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        public class Main {
            public static void main(String[] args) {
                JPasswordField pw = new JPasswordField();
                pw.setText("hunter2");
                char[] chars = pw.getPassword();
                System.out.println("len " + chars.length + " = " + new String(chars));
            }
        }
        "#,
        "Main",
        vec![],
    );
    assert_eq!(out, "len 7 = hunter2\n");
}

#[test]
fn swing_color_helpers_sizing_and_alignment() {
    // Color(int), getRed/Green/Blue, darker(); setPreferredSize on a plain
    // widget; JLabel horizontal alignment (JLabel.CENTER via SwingConstants).
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                Color c = new Color(0x3366CC);
                System.out.println(c.getRed() + "," + c.getGreen() + "," + c.getBlue());
                Color d = new Color(200, 100, 50).darker();
                System.out.println(d.getRed() + "," + d.getGreen() + "," + d.getBlue());
                System.out.println("center " + JLabel.CENTER + " right " + SwingConstants.RIGHT);
            }
        }
        "#,
        "Main",
        vec![],
    );
    assert_eq!(out, "51,102,204\n140,70,35\ncenter 0 right 4\n");
}

#[test]
fn swing_preferred_size_and_alignment_serialize() {
    // A widget's preferred size and a label's alignment reach the tree.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Sized");
                JButton big = new JButton("Big");
                big.setPreferredSize(new Dimension(160, 50));
                JLabel title = new JLabel("Centered", SwingConstants.CENTER);
                frame.add(big);
                frame.add(title);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""psize":"160,50""#),
        "no preferred size: {json}"
    );
    assert!(json.contains(r#""halign":0"#), "no alignment: {json}");
}

#[test]
fn swing_invoke_later_content_pane_and_visibility() {
    // The common tutorial bootstrap: SwingUtilities.invokeLater builds the UI,
    // getContentPane().add adds to the frame, and a hidden component serializes
    // its hidden flag. invokeLater runs synchronously (no separate EDT).
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                SwingUtilities.invokeLater(new Runnable() {
                    public void run() {
                        JFrame f = new JFrame("Boot");
                        f.getContentPane().add(new JButton("Go"));
                        JLabel secret = new JLabel("hidden");
                        secret.setVisible(false);
                        f.getContentPane().add(secret);
                        f.setVisible(true);
                    }
                });
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""text":"Go""#),
        "content pane add failed: {json}"
    );
    assert!(
        json.contains(r#""text":"hidden","hidden":true"#) || json.contains(r#""hidden":true"#),
        "no hidden flag: {json}"
    );
}

#[test]
fn swing_container_remove_and_component_count() {
    // Container.remove / removeAll / getComponentCount manage children.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static JPanel panel;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Dyn");
                panel = new JPanel();
                JButton a = new JButton("A");
                panel.add(a);
                panel.add(new JButton("B"));
                JButton report = new JButton("Report");
                report.addActionListener(e -> {
                    System.out.println("count " + Main.panel.getComponentCount());
                    Main.panel.remove(0);
                    System.out.println("after remove " + Main.panel.getComponentCount());
                    Main.panel.removeAll();
                    System.out.println("after clear " + Main.panel.getComponentCount());
                });
                frame.add(panel);
                frame.add(report);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("c4"))],
    );
    assert_eq!(out, "count 2\nafter remove 1\nafter clear 0\n");
}

#[test]
fn swing_invoke_later_with_lambda_runs_the_ui() {
    // SwingUtilities.invokeLater(() -> ...) — the lambda targets Runnable.
    let json = run_swing(
        r#"
        import javax.swing.*;
        public class Main {
            public static void main(String[] args) {
                SwingUtilities.invokeLater(() -> {
                    JFrame f = new JFrame("Lambda");
                    f.add(new JLabel("ready"));
                    f.setVisible(true);
                });
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""text":"ready""#),
        "lambda invokeLater failed: {json}"
    );
}

#[test]
fn swing_set_bounds_and_null_layout_serialize_for_absolute_positioning() {
    // setLayout(null) requests absolute positioning ("none"); a child's
    // setBounds serializes its x,y,w,h so the renderer can place it. Mirrors
    // the classic null-layout tutorial. Ids: frame c0, button c1.
    let json = run_swing(
        r#"
        import javax.swing.*;
        public class Main {
            public static void main(String[] args) {
                JFrame f = new JFrame();
                JButton b1 = new JButton("Hello, World!");
                b1.setBounds(90, 100, 180, 40);
                f.add(b1);
                f.setSize(400, 400);
                f.setLayout(null);
                f.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(json.contains(r#""layout":"none""#), "not absolute: {json}");
    assert!(
        json.contains(r#""width":400,"height":400"#),
        "no frame size: {json}"
    );
    assert!(
        json.contains(r#""bounds":"90,100,180,40""#),
        "no bounds: {json}"
    );
}

#[test]
fn swing_tool_bar_serializes_buttons_separator_and_dispatches() {
    // A JToolBar serializes its name, orientation, and child controls (with a
    // separator between groups); a toolbar button's ActionListener fires on
    // click. Ids: frame c0, toolbar c1, cut c2, copy c3, separator c4, about c5.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                JToolBar bar = new JToolBar("Actions");
                bar.add(new JButton("Cut"));
                bar.add(new JButton("Copy"));
                bar.addSeparator();
                bar.add(new JButton("About"));
                frame.add(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""type":"toolbar","orientation":0,"name":"Actions""#),
        "no toolbar: {json}"
    );
    assert!(
        json.contains(r#""type":"toolbarsep""#),
        "no separator: {json}"
    );
    assert!(
        json.contains(r#""type":"button""#),
        "no toolbar buttons: {json}"
    );
}

#[test]
fn swing_tool_bar_button_fires_listener() {
    // A button inside a toolbar dispatches its ActionListener like any other.
    // Ids: frame c0, toolbar c1, save c2.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                JToolBar bar = new JToolBar();
                JButton save = new JButton("Save");
                save.addActionListener(e -> System.out.println("saved"));
                bar.add(save);
                frame.add(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("c2"))],
    );
    assert_eq!(out, "saved\n");
}

#[test]
fn swing_split_pane_serializes_both_sides_and_divider() {
    // A JSplitPane serializes its orientation, divider location, and the two
    // nested components. Constructing with the (orientation, left, right) ctor
    // and setting the divider both reflect in the tree.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Split");
                JSplitPane split = new JSplitPane(JSplitPane.HORIZONTAL_SPLIT,
                    new JLabel("nav"), new JButton("content"));
                split.setDividerLocation(120);
                frame.add(split);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""type":"splitpane","orientation":1,"divider":120"#),
        "no splitpane: {json}"
    );
    assert!(
        json.contains(r#""left":{"type":"label""#),
        "no left component: {json}"
    );
    assert!(
        json.contains(r#""right":{"type":"button""#),
        "no right component: {json}"
    );
}

#[test]
fn swing_tabbed_pane_serializes_tabs_and_selection() {
    // The serialized tree carries each tab's title and nested component plus the
    // current selection and placement, for the tablist/tabpanel renderer.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Tabs");
                JTabbedPane tabs = new JTabbedPane();
                tabs.addTab("Alpha", new JLabel("a"));
                tabs.addTab("Beta", new JButton("b"));
                tabs.setSelectedIndex(1);
                frame.add(tabs);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""type":"tabbedpane","placement":1,"selectedIndex":1"#),
        "no tabbedpane: {json}"
    );
    assert!(
        json.contains(r#""title":"Alpha","component":{"type":"label""#),
        "no alpha tab: {json}"
    );
    assert!(
        json.contains(r#""title":"Beta","component":{"type":"button""#),
        "no beta tab: {json}"
    );
}

#[test]
fn swing_border_factory_serializes_border_descriptors() {
    // BorderFactory borders serialize as a descriptor on the component: a line
    // border (colour + thickness), an empty padding border (insets), and a
    // titled group box (caption). Ids: frame c0, then the three panels.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.border.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Borders");
                JPanel a = new JPanel();
                a.setBorder(BorderFactory.createLineBorder(new Color(200, 40, 40), 3));
                JPanel b = new JPanel();
                b.setBorder(BorderFactory.createEmptyBorder(5, 10, 15, 20));
                JPanel c = new JPanel();
                Border titled = BorderFactory.createTitledBorder("Options");
                c.setBorder(titled);
                frame.add(a);
                frame.add(b);
                frame.add(c);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""border":{"type":"line","thickness":3,"color":"200,40,40""#),
        "no line border: {json}"
    );
    assert!(
        json.contains(r#""border":{"type":"empty","thickness":1,"insets":"5,10,15,20"}"#),
        "no empty border: {json}"
    );
    assert!(
        json.contains(r#""border":{"type":"titled","thickness":1,"title":"Options""#),
        "no titled border: {json}"
    );
}

#[test]
fn swing_border_factory_compound_matte_and_bevel() {
    // The richer border kinds serialize too: a compound border nests outer +
    // inner descriptors, a matte border carries per-side insets + colour, and a
    // bevel is raised or lowered. Also exercises constructing a subtype directly
    // (new EmptyBorder) rather than via the factory.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.border.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Borders");
                JPanel a = new JPanel();
                a.setBorder(BorderFactory.createCompoundBorder(
                    BorderFactory.createLineBorder(new Color(30, 90, 200), 2),
                    new EmptyBorder(6, 8, 6, 8)));
                JPanel b = new JPanel();
                b.setBorder(BorderFactory.createMatteBorder(4, 0, 4, 0, new Color(220, 140, 40)));
                JPanel c = new JPanel();
                c.setBorder(BorderFactory.createLoweredBevelBorder());
                frame.add(a);
                frame.add(b);
                frame.add(c);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(
            r#""border":{"type":"compound","thickness":1,"insets":"0,0,0,0","outer":{"type":"line","thickness":2,"color":"30,90,200","insets":"0,0,0,0"},"inner":{"type":"empty","thickness":1,"insets":"6,8,6,8"}}"#
        ),
        "no compound border: {json}"
    );
    assert!(
        json.contains(
            r#""border":{"type":"matte","thickness":1,"color":"220,140,40","insets":"4,0,4,0"}"#
        ),
        "no matte border: {json}"
    );
    assert!(
        json.contains(
            r#""border":{"type":"bevel","thickness":2,"insets":"0,0,0,0","raised":false}"#
        ),
        "no lowered bevel: {json}"
    );
}

#[test]
fn swing_component_font_serializes() {
    // Component.setFont serializes "<style> <size> <family>" on the widget so
    // the renderer can style its text (JLabel here; applies to any component).
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Fonts");
                JLabel label = new JLabel("Title");
                label.setFont(new Font("Serif", Font.BOLD, 22));
                frame.add(label);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""font":"1 22 Serif""#),
        "no widget font: {json}"
    );
}

#[test]
fn swing_graphics2d_set_stroke_records_line_width() {
    // A custom panel casts the Graphics to Graphics2D and sets a wider pen; the
    // stroke width/cap/join serialize for the canvas renderer. The cast must
    // succeed (the panel builds a Graphics2D), and BasicStroke(3) widens int→float.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Pen");
                Sketch s = new Sketch();
                s.setPreferredSize(new Dimension(120, 80));
                frame.add(s);
                frame.setVisible(true);
            }
        }
        class Sketch extends JPanel {
            public void paintComponent(Graphics g) {
                super.paintComponent(g);
                Graphics2D g2 = (Graphics2D) g;
                g2.setStroke(new BasicStroke(3));
                g2.setColor(new Color(255, 0, 0));
                g2.drawLine(10, 10, 100, 60);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains("setStroke 3 2 0"),
        "no stroke (width 3, cap square, join miter): {json}"
    );
    assert!(json.contains("drawLine 10 10 100 60"), "no line: {json}");
}

#[test]
fn swing_key_listener_fires_pressed_typed_released() {
    // A KeyListener on a focusable panel: keydown delivers keyPressed (and, for
    // a printable char, keyTyped); keyup delivers keyReleased. The host reports
    // "__key=<type>,<code>,<char>" (type 0 = down, 1 = up). Ids: frame c0,
    // panel c1. An arrow key has no character (CHAR_UNDEFINED), so it fires no
    // keyTyped; the letter 'a' (code 65, char 97) fires both pressed and typed.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Move");
                KeyPanel panel = new KeyPanel();
                frame.add(panel);
                frame.setVisible(true);
            }
        }
        class KeyPanel extends JPanel {
            public KeyPanel() {
                addKeyListener(new KeyAdapter() {
                    public void keyPressed(KeyEvent e) {
                        System.out.println("pressed " + e.getKeyCode());
                    }
                    public void keyReleased(KeyEvent e) {
                        System.out.println("released " + e.getKeyCode());
                    }
                    public void keyTyped(KeyEvent e) {
                        System.out.println("typed " + e.getKeyChar());
                    }
                });
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\n__key=0,39,65535")), // keydown ArrowRight (VK_RIGHT)
            Some(String::from("c1\n__key=1,39,65535")), // keyup ArrowRight
            Some(String::from("c1\n__key=0,65,97")),    // keydown 'a'
        ],
    );
    assert_eq!(out, "pressed 39\nreleased 39\npressed 65\ntyped a\n");
}

#[test]
fn swing_key_listener_serializes_focusable_flag() {
    // A panel with a KeyListener advertises "key":true so the renderer makes it
    // focusable and wires keydown/keyup; a plain panel does not.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Move");
                JPanel panel = new JPanel();
                panel.addKeyListener(new KeyAdapter() {
                    public void keyPressed(KeyEvent e) {}
                });
                frame.add(panel);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(json.contains(r#""key":true"#), "no key flag: {json}");
}

#[test]
fn swing_interactive_program_compiles_and_renders_initial_frame() {
    // A listener-driven program. With no interactive host (BufferedConsole's
    // ui_await_event returns None), the event loop ends at once — but the
    // initial frame still renders, and crucially the lambda, addActionListener,
    // and the __SwingRuntime event loop all compile and run to completion.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static int count = 0;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Counter");
                JLabel label = new JLabel("Clicks: 0");
                JButton button = new JButton("Click me");
                button.addActionListener(e -> {
                    Main.count++;
                    label.setText("Clicks: " + Main.count);
                });
                frame.add(label);
                frame.add(button);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""type":"label","text":"Clicks: 0""#),
        "no initial label: {json}"
    );
    assert!(
        json.contains(r#""type":"button","text":"Click me""#),
        "no button: {json}"
    );
}

/// A console that scripts Swing UI events (and captures stdout), so the
/// event loop and window-close handling can be exercised headlessly. Each
/// `__uiAwait` returns the next scripted event; `Some(None)` and running out
/// both yield `None` (the loop's end-of-session signal).
struct ScriptedUiConsole {
    events: std::collections::VecDeque<Option<String>>,
    dialogs: std::collections::VecDeque<Option<String>>,
    stdout: Vec<u8>,
}

impl ScriptedUiConsole {
    fn new(events: Vec<Option<String>>) -> Self {
        Self {
            events: events.into_iter().collect(),
            dialogs: std::collections::VecDeque::new(),
            stdout: Vec::new(),
        }
    }
    /// Script the responses to `JOptionPane` dialogs, in order.
    fn with_dialogs(mut self, dialogs: Vec<Option<String>>) -> Self {
        self.dialogs = dialogs.into_iter().collect();
        self
    }
    fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

impl caturra_vm::ConsoleIo for ScriptedUiConsole {
    fn stdout(&mut self, bytes: &[u8]) {
        self.stdout.extend_from_slice(bytes);
    }
    fn stderr(&mut self, _bytes: &[u8]) {}
    fn read_line(&mut self) -> Option<String> {
        None
    }
    fn ui_await_event(&mut self, _tree: &str) -> Option<String> {
        self.events.pop_front().flatten()
    }
    fn ui_dialog(&mut self, _kind: &str, _message: &str) -> Option<String> {
        self.dialogs.pop_front().flatten()
    }
}

fn run_swing_scripted(source: &str, main: &str, events: Vec<Option<String>>) -> String {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{main}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    let mut console = ScriptedUiConsole::new(events);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    let result = vm.run_main(main, &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    console.stdout_text()
}

/// Construction order fixes ids: frame c0, button c1. A shared program that
/// prints on a click and again after the event loop returns.
const SWING_CLOSE_PROGRAM: &str = r#"
    import javax.swing.*;
    import java.awt.*;
    public class Main {
        public static void main(String[] args) {
            JFrame frame = new JFrame("W");
            CLOSE_OP
            JButton b = new JButton("Go");
            b.addActionListener(e -> System.out.println("clicked"));
            frame.add(b);
            frame.setVisible(true);
            System.out.println("after close");
        }
    }
"#;

#[test]
fn swing_close_with_exit_on_close_ends_the_program() {
    // The close button (EXIT_ON_CLOSE) ends the event loop: setVisible
    // returns and the rest of main runs — without dispatching the click.
    let source = SWING_CLOSE_PROGRAM.replace(
        "CLOSE_OP",
        "frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);",
    );
    let out = run_swing_scripted(&source, "Main", vec![Some(String::from("__close"))]);
    assert_eq!(out, "after close\n", "close should end the loop cleanly");
}

#[test]
fn swing_close_with_default_hide_keeps_running() {
    // The default (HIDE_ON_CLOSE) ignores the close, so the loop keeps
    // going: the next event (a click on button c1) fires its listener, and
    // only the end-of-session signal (None) finally returns.
    let out = run_swing_scripted(
        &SWING_CLOSE_PROGRAM.replace("CLOSE_OP", ""),
        "Main",
        vec![Some(String::from("__close")), Some(String::from("c1"))],
    );
    assert_eq!(
        out, "clicked\nafter close\n",
        "close was ignored, click ran"
    );
}

#[test]
fn swing_mouse_listener_fires_with_coordinates() {
    // A panel with a MouseListener (an anonymous MouseAdapter subclass) gets
    // its mousePressed called with the click coordinates the host reports in
    // a "__mouse=x,y" payload line. Ids: frame c0, panel c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Paint");
                JPanel panel = new JPanel();
                panel.addMouseListener(new MouseAdapter() {
                    public void mousePressed(MouseEvent e) {
                        System.out.println("press " + e.getX() + "," + e.getY());
                    }
                });
                frame.add(panel);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("c1\n__mouse=50,60"))],
    );
    assert_eq!(out, "press 50,60\n");
}

#[test]
fn swing_timer_ticks_fire_the_action_listener() {
    // A started Timer fires its ActionListener on each host-scheduled tick.
    // The listener here is an anonymous class implementing the *injected*
    // ActionListener interface — which also exercises the pass-1 is_interface
    // fix (an anon class implementing an interface from a later unit). Ids:
    // frame c0, timer t1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            static int ticks = 0;
            public static void main(String[] args) {
                JFrame frame = new JFrame("T");
                Timer timer = new Timer(50, new ActionListener() {
                    public void actionPerformed(ActionEvent e) {
                        Main.ticks++;
                        System.out.println("tick " + Main.ticks);
                    }
                });
                timer.start();
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("__timer:t1")),
            Some(String::from("__timer:t1")),
        ],
    );
    assert_eq!(out, "tick 1\ntick 2\n");
}

#[test]
fn swing_lambda_as_constructor_argument_is_target_typed() {
    // A lambda passed straight to a constructor (`new Timer(40, e -> …)`) is
    // target-typed by the constructor's parameter, so it desugars like a
    // method-argument lambda. Ids: frame c0, timer t1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static int ticks = 0;
            public static void main(String[] args) {
                JFrame frame = new JFrame("T");
                Timer timer = new Timer(50, e -> {
                    Main.ticks++;
                    System.out.println("tick " + Main.ticks);
                });
                timer.start();
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("__timer:t1"))],
    );
    assert_eq!(out, "tick 1\n");
}

#[test]
fn swing_joptionpane_dialogs_block_and_return_responses() {
    // JOptionPane dialogs run in plain main (no event loop). Each show*
    // blocks on the host and returns its scripted response: the input dialog
    // yields "Ada", the confirm yields "0" (YES_OPTION); the message dialog's
    // response is ignored.
    let source = r#"
        import javax.swing.*;
        public class Main {
            public static void main(String[] args) {
                String name = JOptionPane.showInputDialog("name?");
                System.out.println("hi " + name);
                int ok = JOptionPane.showConfirmDialog(null, "ok?");
                if (ok == JOptionPane.YES_OPTION) {
                    System.out.println("confirmed");
                    JOptionPane.showMessageDialog(null, "done");
                }
            }
        }
    "#;
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("Main.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    let mut console = ScriptedUiConsole::new(vec![])
        .with_dialogs(vec![Some(String::from("Ada")), Some(String::from("0"))]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    let result = vm.run_main("Main", &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "hi Ada\nconfirmed\n");
}

#[test]
fn swing_text_area_reads_a_multi_line_value() {
    // A JTextArea holds multi-line text. The host percent-escapes the value's
    // newlines (%0A) so it survives the newline-delimited payload; the loop
    // decodes it, and getText() returns the real multi-line string. Ids:
    // frame c0, area c1, button c2.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Notes");
                JTextArea area = new JTextArea(3, 20);
                JButton save = new JButton("Save");
                save.addActionListener(e -> System.out.println("[" + area.getText() + "]"));
                frame.add(area);
                frame.add(save);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        // Click Save (c2); the area (c1) holds "line one\nline two" (escaped).
        vec![Some(String::from("c2\nc1=line one%0Aline two"))],
    );
    assert_eq!(out, "[line one\nline two]\n");
}

#[test]
fn swing_scroll_pane_registers_and_dispatches_its_view() {
    // A component wrapped directly in a JScrollPane is never added to a
    // Container, so the scroll pane must register it — otherwise the click
    // couldn't find the button. Ids: frame c0, button c1, scroll pane c2.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("S");
                JButton go = new JButton("Go");
                go.addActionListener(e -> System.out.println("go!"));
                JScrollPane scroll = new JScrollPane(go);
                scroll.setPreferredSize(new Dimension(120, 80));
                frame.add(scroll);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("c1"))],
    );
    assert_eq!(out, "go!\n");
}

#[test]
fn swing_list_selection_fires_listener_with_value() {
    // Selecting a JList item fires its (lambda) ListSelectionListener and
    // getSelectedValue reads the picked item. Ids: frame c0, list c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        public class Main {
            static JList list;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Pick");
                list = new JList(new String[]{"Apple", "Banana", "Cherry"});
                list.addListSelectionListener(e -> System.out.println("picked " + Main.list.getSelectedValue()));
                frame.add(list);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        // Select index 2 (Cherry), then index 0 (Apple).
        vec![
            Some(String::from("c1\nc1=2")),
            Some(String::from("c1\nc1=0")),
        ],
    );
    assert_eq!(out, "picked Cherry\npicked Apple\n");
}

#[test]
fn swing_default_list_model_updates_the_list_on_render() {
    // A JList backed by a DefaultListModel re-reads the model every render, so
    // a button that mutates the model changes what the list serializes. Ids:
    // frame c0, list c1, add-button c2, clear-button c3.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            static DefaultListModel model;
            static int n = 0;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Tasks");
                model = new DefaultListModel();
                model.addElement("first");
                JList list = new JList(model);
                JButton add = new JButton("Add");
                add.addActionListener(e -> {
                    Main.n = Main.n + 1;
                    Main.model.addElement("task " + Main.n);
                    System.out.println("size " + Main.model.getSize());
                });
                JButton clear = new JButton("Clear");
                clear.addActionListener(e -> {
                    Main.model.clear();
                    System.out.println("size " + Main.model.getSize());
                });
                frame.add(list);
                frame.add(add);
                frame.add(clear);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        // Add twice (grows to 3), then clear (drops to 0).
        vec![
            Some(String::from("c2")),
            Some(String::from("c2")),
            Some(String::from("c3")),
        ],
    );
    assert_eq!(out, "size 2\nsize 3\nsize 0\n");
}

#[test]
fn swing_multi_select_list_reports_all_indices() {
    // A multi-select JList: the host reports the chosen indices comma-separated,
    // and getSelectedIndices / getSelectedValues expose all of them (with
    // getSelectedValue still the first). Ids: frame c0, list c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import javax.swing.event.*;
        public class Main {
            static JList list;
            public static void main(String[] args) {
                JFrame frame = new JFrame("Pick");
                list = new JList(new String[]{"Red", "Green", "Blue"});
                list.setSelectionMode(ListSelectionModel.MULTIPLE_INTERVAL_SELECTION);
                list.addListSelectionListener(e -> {
                    int[] idx = Main.list.getSelectedIndices();
                    System.out.println("count " + idx.length + " first " + Main.list.getSelectedValue());
                });
                frame.add(list);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        // Select indices 0 and 2 (Red, Blue), then clear to none.
        vec![
            Some(String::from("c1\nc1=0,2")),
            Some(String::from("c1\nc1=")),
        ],
    );
    assert_eq!(out, "count 2 first Red\ncount 0 first null\n");
}

#[test]
fn swing_nested_submenu_item_dispatches() {
    // A JMenu can be added to another JMenu as a submenu; items inside a nested
    // submenu still register and dispatch. JMenu is a Component too, so ids are
    // frame c0, File c1, Export c2, "As image" c3, PNG c4, JPEG c5.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                JMenuBar bar = new JMenuBar();
                JMenu file = new JMenu("File");
                JMenuItem export = new JMenuItem("Export");
                export.addActionListener(e -> System.out.println("export"));
                JMenu asImage = new JMenu("As image");
                JMenuItem png = new JMenuItem("PNG");
                png.addActionListener(e -> System.out.println("png"));
                JMenuItem jpeg = new JMenuItem("JPEG");
                jpeg.addActionListener(e -> System.out.println("jpeg"));
                asImage.add(png);
                asImage.add(jpeg);
                file.add(export);
                file.add(asImage);
                bar.add(file);
                frame.setJMenuBar(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        // Click the deep submenu item (JPEG=c5), then the top item (Export=c2).
        vec![Some(String::from("c5")), Some(String::from("c2"))],
    );
    assert_eq!(out, "jpeg\nexport\n");
}

#[test]
fn swing_menu_item_click_fires_its_listener() {
    // A menu item fires its ActionListener when activated, like a button.
    // JMenuBar is a plain holder (no id); JMenu is a Component, so ids are
    // frame c0, file c1, Open c2, Quit c3 (the separator is c4).
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Editor");
                JMenuBar bar = new JMenuBar();
                JMenu file = new JMenu("File");
                JMenuItem open = new JMenuItem("Open");
                open.addActionListener(e -> System.out.println("open!"));
                JMenuItem quit = new JMenuItem("Quit");
                quit.addActionListener(e -> System.out.println("quit!"));
                file.add(open);
                file.addSeparator();
                file.add(quit);
                bar.add(file);
                frame.setJMenuBar(bar);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![Some(String::from("c3")), Some(String::from("c2"))],
    );
    assert_eq!(out, "quit!\nopen!\n");
}

#[test]
fn swing_mouse_motion_listener_fires_on_drag() {
    // A panel with a MouseMotionListener (an anonymous MouseAdapter, which
    // implements both mouse interfaces) gets mouseDragged with the drag
    // coordinates from a "__drag=x,y" payload line. Ids: frame c0, panel c1.
    let out = run_swing_scripted(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Pad");
                JPanel panel = new JPanel();
                panel.addMouseMotionListener(new MouseAdapter() {
                    public void mouseDragged(MouseEvent e) {
                        System.out.println("drag " + e.getX() + "," + e.getY());
                    }
                });
                frame.add(panel);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
        vec![
            Some(String::from("c1\n__drag=10,20")),
            Some(String::from("c1\n__drag=15,28")),
        ],
    );
    assert_eq!(out, "drag 10,20\ndrag 15,28\n");
}

#[test]
fn swing_rich_controls_dispatch_their_listeners() {
    // Non-button controls now dispatch: a checkbox toggle fires its
    // ItemListener, a combo box selection and a slider drag fire their
    // listeners, and a radio button selection fires (with its state synced).
    // Ids follow construction order (frame c0, checkbox c1, combo c2, slider
    // c3, radios c4/c5); each scripted payload is "<cid>\n<id>=<value>...".
    let source = r#"
        import javax.swing.*;
        import javax.swing.event.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("Controls");
                frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
                JCheckBox agree = new JCheckBox("Agree");
                agree.addItemListener(e -> System.out.println("agree=" + agree.isSelected()));
                JComboBox size = new JComboBox(new String[]{"S", "M", "L"});
                size.addActionListener(e -> System.out.println("size=" + size.getSelectedItem()));
                JSlider level = new JSlider(0, 10, 5);
                level.addChangeListener(e -> System.out.println("level=" + level.getValue()));
                JRadioButton red = new JRadioButton("Red", true);
                JRadioButton blue = new JRadioButton("Blue");
                ButtonGroup colors = new ButtonGroup();
                colors.add(red);
                colors.add(blue);
                blue.addItemListener(e -> System.out.println("blue=" + blue.isSelected()));
                frame.add(agree);
                frame.add(size);
                frame.add(level);
                frame.add(red);
                frame.add(blue);
                frame.setVisible(true);
            }
        }
    "#;
    let out = run_swing_scripted(
        source,
        "Main",
        vec![
            Some(String::from("c1\nc1=true")),           // tick the checkbox
            Some(String::from("c2\nc2=2")),              // combo box -> "L"
            Some(String::from("c3\nc3=8")),              // slider -> 8
            Some(String::from("c5\nc5=true\nc4=false")), // pick the Blue radio
        ],
    );
    assert_eq!(out, "agree=true\nsize=L\nlevel=8\nblue=true\n");
}

#[test]
fn neighborhood_painter_simulation() {
    // A 4x4 grid: a wall at (2,1) and a one-unit paint bucket at (1,2).
    let grid = "1,0 1,0 1,0 1,0\n1,0 1,0 0,0 1,0\n1,0 1,1 1,0 1,0\n1,0 1,0 1,0 1,0\n";
    let out = run_neighborhood(
        r#"
        import org.code.neighborhood.*;
        public class NbhdMain {
            public static void main(String[] args) {
                Painter p = new Painter(0, 0, "east", 5);
                System.out.println(p.getDirection() + " " + p.getX() + " " + p.getY() + " " + p.hasPaint());
                p.paint("red");
                p.move();
                p.paint("blue");
                System.out.println("onPaint=" + p.isOnPaint() + " color=" + p.getColor() + " left=" + p.getMyPaint());
                p.turnLeft();
                System.out.println("nowFacing=" + p.getDirection() + " canMove=" + p.canMove());
                Painter q = new Painter(1, 2, "north", 0);
                System.out.println("q onBucket=" + q.isOnBucket());
                q.takePaint();
                q.paint("green");
                System.out.println("q left=" + q.getMyPaint() + " qcolor=" + q.getColor());
                p.scrapePaint();
                System.out.println("after scrape p color=" + p.getColor());
            }
        }
        "#,
        "NbhdMain",
        grid,
    );
    assert_eq!(
        out,
        "east 0 0 true\n\
         onPaint=true color=blue left=3\n\
         nowFacing=north canMove=false\n\
         q onBucket=true\n\
         q left=0 qcolor=green\n\
         after scrape p color=null\n"
    );
}

#[test]
fn wrapper_array_and_return_assignment_and_nested_copy() {
    // Integer[] with .intValue(); `return x += y`; a copy-constructed list
    // nested as a constructor argument.
    let out = run_stdout(
        r#"
        import java.util.*;
        public class Main {
            static int sum(Integer[] xs) {
                int s = 0;
                for (int i = 0; i < xs.length; i++) s += xs[i].intValue();
                return s += 0;
            }
            public static void main(String[] a) {
                Integer[] xs = {new Integer(2), new Integer(3), new Integer(5)};
                System.out.println(sum(xs));
                ArrayList<String> src = new ArrayList<String>();
                src.add("a"); src.add("b");
                Box box = new Box(new ArrayList<>(src));
                System.out.println(box.get());
            }
        }
        class Box {
            private ArrayList<String> items;
            public Box(ArrayList<String> items) { this.items = items; }
            public ArrayList<String> get() { return items; }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "10\n[a, b]\n");
}

/// `Arrays.setAll(array, generator)` fills each slot from a generator of its
/// index, for `int[]`, object, and `double[]` arrays. Pinned JDK-free; the
/// byte-for-byte JDK match is in `diff_arrays_set_all`.
#[test]
fn arrays_set_all_fills_from_index() {
    let out = run_stdout(
        r#"
        import java.util.Arrays;
        public class A {
            public static void main(String[] args) {
                int[] a = new int[5];
                Arrays.setAll(a, i -> i * i);
                System.out.println(Arrays.toString(a));

                String[] s = new String[4];
                Arrays.setAll(s, i -> "item" + i);
                System.out.println(Arrays.toString(s));

                double[] d = new double[3];
                Arrays.setAll(d, i -> i / 2.0);
                System.out.println(Arrays.toString(d));

                int[] b = new int[4];
                Arrays.setAll(b, i -> b.length - i);
                System.out.println(Arrays.toString(b));
            }
        }
        "#,
        "A",
    );
    assert_eq!(
        out,
        "[0, 1, 4, 9, 16]\n\
         [item0, item1, item2, item3]\n\
         [0.0, 0.5, 1.0]\n\
         [4, 3, 2, 1]\n"
    );
}

#[test]
fn arraylist_copy_and_aslist_varargs() {
    // ArrayList copy constructor (independent list), Arrays.asList varargs,
    // Collections.reverse over any element type.
    let out = run_stdout(
        r#"
        import java.util.*;
        public class Main {
            public static void main(String[] a) {
                List<String> v = Arrays.asList("a", "b", "c");
                ArrayList<String> copy = new ArrayList<String>(v);
                copy.add("d");
                System.out.println(v.size() + " " + copy.size());
                Collections.reverse(copy);
                System.out.println(copy);
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "3 4\n[d, c, b, a]\n");
}

#[test]
fn string_builder_and_format_argument() {
    // First-class StringBuilder (append chaining, toString, length) plus the
    // fix for String.format(...) used directly as a method argument.
    let out = run_stdout(
        r#"
        public class Main {
            public static void main(String[] a) {
                StringBuilder sb = new StringBuilder("start");
                sb.append(":").append(1).append(true);
                System.out.println(sb.toString());
                System.out.println(sb.length());
                StringBuilder r = new StringBuilder();
                for (int i = 0; i < 3; i++) {
                    r.append(String.format("v%d,", i));
                }
                System.out.println(r);
                System.out.println("x" + String.format("%.2f", 3.14159) + "y");
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "start:1true\n11\nv0,v1,v2,\nx3.14y\n");
}

#[test]
#[allow(clippy::needless_raw_string_hashes)]
fn user_object_default_equals_is_identity() {
    // A class with no equals override uses Object identity equality.
    let out = run_stdout(
        r#"
        public class Main {
            public static void main(String[] a) {
                Main x = new Main();
                Main y = new Main();
                System.out.println(x.equals(x));
                System.out.println(x.equals(y));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "true\nfalse\n");
}

#[test]
fn reflection_array_class_literal() {
    // int[].class carries the JVM array descriptor, so getMethod matches a
    // method declared with an int[] parameter.
    let out = run_stdout(
        r#"
        import java.lang.reflect.Method;
        public class Main {
            public int total(int[] xs) { int s = 0; for (int x : xs) s += x; return s; }
            public static void main(String[] a) throws Exception {
                System.out.println(int[].class.getName());
                Method m = Main.class.getMethod("total", new Class[]{int[].class});
                System.out.println(m.getName());
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "[I\ntotal\n");
}

#[test]
fn reflection_method_get_and_invoke() {
    // getMethod(name, Class[]) -> Method.getName/getModifiers; invoke with
    // varargs, returning a boxed primitive cast back to its wrapper.
    let out = run_stdout(
        r#"
        import java.lang.reflect.Method;
        public class Main {
            public int add(int a, int b) { return a + b; }
            public static String hi() { return "hi"; }
            public static void main(String[] a) throws Exception {
                Method add = Main.class.getMethod("add", new Class[]{int.class, int.class});
                System.out.println(add.getName());
                Main m = new Main();
                int r = (int) add.invoke(m, 3, 4);
                System.out.println(r);
                Method hi = Main.class.getMethod("hi");
                System.out.println((String) hi.invoke(null));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "add\n7\nhi\n");
}

#[test]
fn reflection_call_result_in_string_concat() {
    // A reflection method call as a nested argument inside a string concat used
    // to leave a malformed stack (type_of did not type reflection receivers).
    let out = run_stdout(
        r#"
        import java.lang.reflect.*;
        public class Main {
            private static int total = 5;
            public static void main(String[] args) throws Exception {
                Field f = Main.class.getDeclaredField("total");
                System.out.println("static=" + Modifier.isStatic(f.getModifiers()));
                System.out.println("name=" + f.getName() + " type=" + f.getType().getName());
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "static=true\nname=total type=int\n");
}

#[test]
fn reflection_method_array_and_void_invoke_statement() {
    // getDeclaredMethods() -> Method[], getReturnType/getName qualified,
    // getParameterTypes, and invoke of a void method as a bare statement.
    let out = run_stdout(
        r#"
        import java.lang.reflect.*;
        public class Main {
            private int hours = 0;
            public void addHours(Main other, int n) { hours += n; }
            public java.util.ArrayList<String> makeList() { return new java.util.ArrayList<>(); }
            public int getHours() { return hours; }
            public static void main(String[] args) throws Exception {
                Main obj = new Main();
                Main other = new Main();
                for (Method m : Main.class.getDeclaredMethods()) {
                    if (m.getName().equals("addHours")) {
                        System.out.println(m.getParameterCount());
                        m.invoke(obj, other, 5);   // void invoke as a statement
                    }
                    if (m.getName().equals("makeList")) {
                        System.out.println(m.getReturnType().getName());
                    }
                }
                System.out.println(obj.getHours());
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "2\njava.util.ArrayList\n5\n");
}

#[test]
fn reflection_generic_field_type() {
    // Field.getGenericType() -> ParameterizedType -> getActualTypeArguments():
    // the compiler emits a Signature attribute the VM reflects on.
    let out = run_stdout(
        r#"
        import java.lang.reflect.*;
        import java.util.*;
        public class Main {
            private ArrayList<Friend> friends;
            private int age;
            public static void main(String[] a) {
                for (Field f : Main.class.getDeclaredFields()) {
                    String text = f.getName();
                    Type t = f.getGenericType();
                    if (t instanceof ParameterizedType) {
                        ParameterizedType p = (ParameterizedType) t;
                        text += "<" + p.getActualTypeArguments()[0] + ">";
                    }
                    System.out.println(text);
                }
            }
        }
        class Friend {}
        "#,
        "Main",
    );
    assert_eq!(out, "friends<class Friend>\nage\n");
}

#[test]
fn reflection_field_access_and_modifiers() {
    // getDeclaredField -> Field.get (boxing) -> (int) unbox cast; getModifiers
    // + Modifier.isStatic/isFinal on a static final field.
    let out = run_stdout(
        r#"
        import java.lang.reflect.Field;
        import java.lang.reflect.Modifier;
        public class Main {
            static final int LIMIT = 42;
            int count = 7;
            public static void main(String[] a) throws Exception {
                Main m = new Main();
                Field cf = Main.class.getDeclaredField("count");
                int c = (int) cf.get(m);
                System.out.println(c);
                Field lf = Main.class.getDeclaredField("LIMIT");
                System.out.println((int) lf.get(null));
                int mods = lf.getModifiers();
                System.out.println(Modifier.isStatic(mods));
                System.out.println(Modifier.isFinal(mods));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "7\n42\ntrue\ntrue\n");
}

#[test]
fn array_object_methods_reference_semantics() {
    // Arrays are Objects: equals is reference identity (not element compare).
    let out = run_stdout(
        r#"
        public class Main {
            public static void main(String[] a) {
                String[] x = {"a", "b"};
                String[] alias = x;
                String[] same = {"a", "b"};
                System.out.println(x.equals(alias));
                System.out.println(x.equals(same));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "true\nfalse\n");
}

#[test]
fn sort_arrays_and_collections_and_list_equals() {
    // Arrays.sort(int[]/String[]), Collections.sort(list), a generic downcast,
    // and list equality dispatching a user equals override — `a.equals(b)`
    // agrees with the hand-written element loop beside it.
    let out = run_stdout(
        r#"
        import java.util.*;
        public class Main {
            public static void main(String[] args) {
                int[] xs = {3, 1, 2};
                Arrays.sort(xs);
                System.out.println(Arrays.toString(xs));
                ArrayList<String> ws = new ArrayList<String>();
                ws.add("pear"); ws.add("fig"); ws.add("kiwi");
                Collections.sort(ws);
                System.out.println(ws);
                Object o = ws;
                ArrayList<String> back = (ArrayList<String>) o;
                System.out.println(back.get(0));
                ArrayList<P> a = new ArrayList<P>();
                a.add(new P("x")); a.add(new P("y"));
                ArrayList<P> b = new ArrayList<P>();
                b.add(new P("x")); b.add(new P("y"));
                System.out.println(a.equals(b) + " " + __eq(a, b));
            }
            static boolean __eq(Object x, Object y) {
                if (x instanceof ArrayList<Object> && y instanceof ArrayList<Object>) {
                    ArrayList<Object> p = (ArrayList<Object>) x;
                    ArrayList<Object> q = (ArrayList<Object>) y;
                    if (p.size() != q.size()) return false;
                    for (int i = 0; i < p.size(); i++)
                        if (!p.get(i).equals(q.get(i))) return false;
                    return true;
                }
                return x.equals(y);
            }
        }
        class P {
            private String name;
            public P(String n) { name = n; }
            public boolean equals(Object o) {
                if (!(o instanceof P)) return false;
                return name.equals(((P) o).name);
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "[1, 2, 3]\n[fig, kiwi, pear]\nfig\ntrue true\n");
}

#[test]
fn easymock_full_mock_records_and_replays() {
    // createMock(T.class): a full mock of a class with no no-arg constructor,
    // recording a sequence of return values then replaying them.
    let out = run_stdout(
        r"
        import static org.easymock.EasyMock.*;
        public class Main {
            public static void main(String[] args) {
                Soda mock = createMock(Soda.class);
                expect(mock.getQuantity()).andReturn(2);
                expect(mock.getQuantity()).andReturn(3);
                replay(mock);
                System.out.println(mock.getQuantity() + mock.getQuantity());
                verify(mock);
            }
        }
        class Soda {
            private int quantity;
            public Soda(int q) { quantity = q; }
            public int getQuantity() { return quantity; }
        }
        ",
        "Main",
    );
    assert_eq!(out, "5\n");
}

#[test]
fn easymock_with_constructor_and_mocked_methods() {
    // withConstructor(args) forwards to the super constructor;
    // addMockedMethods("a","b") mocks several methods at once.
    let out = run_neighborhood(
        r#"
        import org.code.neighborhood.*;
        import static org.easymock.EasyMock.*;
        class Main {
            public static void main(String[] args) {}
        }
        class Robot extends Painter {
            public Robot(int x, int y, String d, int p) { super(x, y, d, p); }
            public void go() { move(); paint("red"); move(); }
        }
        class Checker {
            public static void main(String[] args) {
                Robot r = partialMockBuilder(Robot.class)
                    .withConstructor(0, 0, "east", 5)
                    .addMockedMethods("move", "paint")
                    .createMock();
                r.move(); r.paint("red"); r.move();
                replay(r);
                r.go();
                verify(r);
                System.out.println("ok");
            }
        }
        "#,
        "Checker",
        "1,0 1,0
1,0 1,0
",
    );
    assert_eq!(out, "ok\n");
}

#[test]
fn easymock_partial_mock_record_replay_verify() {
    // EasyMock partial mocks: the mocked methods are stubbed + counted while
    // the real method under test runs; verify() checks the interaction counts.
    let out = run_neighborhood(
        r#"
        import org.code.neighborhood.*;
        import static org.easymock.EasyMock.*;
        class Main {
            public static void main(String[] args) {}
        }
        class Robot extends Painter {
            public void moveThrice() { for (int i = 0; i < 3; i++) move(); }
            public void paintWhile() { while (hasPaint()) { paint("red"); move(); } }
        }
        class Checker {
            public static void main(String[] args) {
                // void call-counting
                Robot r = partialMockBuilder(Robot.class).addMockedMethod("move").createMock();
                r.move(); r.move(); r.move();
                replay(r);
                r.moveThrice();
                verify(r);
                System.out.println("counted");

                // expect(...).andReturn(...).times(...) with a real loop
                Robot r2 = partialMockBuilder(Robot.class)
                    .addMockedMethod("hasPaint")
                    .addMockedMethod("paint")
                    .addMockedMethod("move")
                    .createMock();
                expect(r2.hasPaint()).andReturn(true).times(2);
                r2.paint("red"); r2.move();
                r2.paint("red"); r2.move();
                expect(r2.hasPaint()).andReturn(false).times(1);
                replay(r2);
                r2.paintWhile();
                verify(r2);
                System.out.println("returned");

                // an unexpected extra call is caught
                Robot r3 = partialMockBuilder(Robot.class).addMockedMethod("move").createMock();
                r3.move(); r3.move();
                replay(r3);
                try {
                    r3.moveThrice();
                    System.out.println("no-catch");
                } catch (Throwable e) {
                    System.out.println("caught");
                }
            }
        }
        "#,
        "Checker",
        "1,0 1,0\n1,0 1,0\n",
    );
    assert_eq!(out, "counted\nreturned\ncaught\n");
}

#[test]
fn system_out_test_runner_captures_printed_lines() {
    // org.code.validation SystemOutTestRunner: runs the student main and
    // returns the lines it printed. Capture redirects (System.setOut
    // semantics), so the student's own output does not reach the console.
    let out = run_stdout(
        r#"
        import org.code.validation.*;
        import java.util.*;
        class Main {
            public static void main(String[] a) {
                System.out.println("first");
                System.out.println("second");
            }
        }
        class Checker {
            public static void main(String[] a) {
                List<String> lines = SystemOutTestRunner.run();
                System.out.println("count=" + lines.size());
                System.out.println("has=" + lines.contains("second"));
            }
        }
        "#,
        "Checker",
    );
    assert_eq!(out, "count=2\nhas=true\n");
}

#[test]
fn class_reflection_get_constructor_and_new_instance() {
    // Reflective instantiation: class literals (incl. `int.class`), a
    // `Class<?>[]` built from arg types, getConstructor (with boxing), and
    // newInstance — the surface the Unit 2 ConstructorsHelper uses.
    let out = run_stdout(
        r#"
        import java.util.*;
        import java.lang.reflect.*;
        public class Main {
            public static void main(String[] a) throws Exception {
                Object[] initargs = { "Seahawks", 8 };
                Class<?>[] ptypes = new Class<?>[initargs.length];
                for (int i = 0; i < initargs.length; i++) {
                    if (initargs[i] instanceof Integer) {
                        ptypes[i] = int.class;
                    } else {
                        ptypes[i] = initargs[i].getClass();
                    }
                }
                Constructor<?> ctor = Team.class.getConstructor(ptypes);
                Team t = (Team) ctor.newInstance(initargs);
                System.out.println(t.getName() + " " + t.getWins());
            }
        }
        class Team {
            private String name; private int wins;
            public Team() {}
            public Team(String name, int wins) { this.name = name; this.wins = wins; }
            public String getName() { return name; }
            public int getWins() { return wins; }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "Seahawks 8\n");
}

#[test]
fn class_reflection_forname_and_assignable() {
    // Class.forName + isAssignableFrom (the reflection the neighborhood
    // subclass validators use).
    let out = run_stdout(
        r#"
        public class Main {
            public static void main(String[] a) throws Exception {
                Class animal = Class.forName("Animal");
                Class dog = new Dog().getClass();
                System.out.println(animal.isAssignableFrom(dog));
                System.out.println(dog.isAssignableFrom(animal));
                System.out.println(Class.forName("Dog").getSimpleName());
            }
        }
        class Animal {}
        class Dog extends Animal {}
        "#,
        "Main",
    );
    assert_eq!(out, "true\nfalse\nDog\n");
}

#[test]
fn validation_helper_lists_student_classes() {
    // ValidationHelper.getClassNames() reports the student's classes (not the
    // injected library classes like Painter).
    let out = run_neighborhood(
        r#"
        import org.code.neighborhood.*;
        import org.code.validation.*;
        import java.util.*;
        class Main {
            public static void main(String[] a) {}
        }
        class Rabbit {}
        class Checker {
            public static void main(String[] a) {
                List<String> names = ValidationHelper.getClassNames();
                System.out.println(names.contains("Rabbit") + " " + names.contains("Painter"));
            }
        }
        "#,
        "Checker",
        "1,0\n",
    );
    assert_eq!(out, "true false\n");
}

#[test]
fn neighborhood_validation_counts_painter_actions() {
    // org.code.validation: NeighborhoodTestRunner.run() invokes the student's
    // Main, then reports the recorded action log — including query methods
    // (isOnBucket) that the visual stream omits.
    let grid = "1,3 1,0 1,0\n1,0 1,0 1,0\n1,0 1,0 1,0\n";
    let out = run_neighborhood(
        r#"
        import org.code.neighborhood.*;
        import org.code.validation.*;
        class Main {
            public static void main(String[] args) {
                Painter p = new Painter(0, 0, "east", 0);
                while (p.isOnBucket()) {
                    p.takePaint();
                }
                p.move();
            }
        }
        class Checker {
            public static void main(String[] args) {
                NeighborhoodLog log = NeighborhoodTestRunner.run();
                PainterLog[] logs = log.getPainterLogs();
                PainterLog pl = logs[0];
                System.out.println("painters=" + logs.length);
                System.out.println("bucketChecks=" + pl.actionCount(NeighborhoodActionType.IS_ON_BUCKET));
                System.out.println("takes=" + pl.actionCount(NeighborhoodActionType.TAKE_PAINT));
                System.out.println("moveOnce=" + pl.didActionExactly(NeighborhoodActionType.MOVE, 1));
                System.out.println("takesAtLeast2=" + pl.didActionAtLeast(NeighborhoodActionType.TAKE_PAINT, 2));
            }
        }
        "#,
        "Checker",
        grid,
    );
    assert_eq!(
        out,
        "painters=1\n\
         bucketChecks=4\n\
         takes=3\n\
         moveOnce=true\n\
         takesAtLeast2=true\n"
    );
}

#[test]
fn neighborhood_emits_javabuilder_message_stream() {
    // The bundled Painter emits the same NEIGHBORHOOD ClientMessages the
    // real javabuilder marshals to the frontend renderer.
    let grid = "1,0 1,0 1,0\n1,0 1,0 1,0\n1,0 1,0 1,0\n";
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "Runner.java".to_owned(),
        text: r#"
            import org.code.neighborhood.*;
            public class Runner {
                public static void main(String[] args) {
                    Painter p = new Painter(0, 0, "east", 3);
                    p.paint("red");
                    p.move();
                    p.turnLeft();
                    p.scrapePaint();
                }
            }
        "#
        .to_owned(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    vfs.write_file("grid.txt", grid.as_bytes().to_vec())
        .unwrap();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    assert!(matches!(
        vm.run_main("Runner", &[]),
        Ok(ExitStatus::Completed)
    ));
    let messages =
        String::from_utf8_lossy(vfs.read_file("neighborhood.jsonl").unwrap()).into_owned();
    assert_eq!(
        messages,
        "{\"type\":\"NEIGHBORHOOD\",\"value\":\"INITIALIZE_PAINTER\",\"detail\":{\"id\":\"painter-0\",\"direction\":\"east\",\"x\":\"0\",\"y\":\"0\",\"paint\":\"3\"}}\n\
         {\"type\":\"NEIGHBORHOOD\",\"value\":\"PAINT\",\"detail\":{\"id\":\"painter-0\",\"color\":\"red\"}}\n\
         {\"type\":\"NEIGHBORHOOD\",\"value\":\"MOVE\",\"detail\":{\"id\":\"painter-0\",\"direction\":\"east\"}}\n\
         {\"type\":\"NEIGHBORHOOD\",\"value\":\"TURN_LEFT\",\"detail\":{\"id\":\"painter-0\",\"direction\":\"north\"}}\n\
         {\"type\":\"NEIGHBORHOOD\",\"value\":\"REMOVE_PAINT\",\"detail\":{\"id\":\"painter-0\"}}\n"
    );
}

#[test]
fn neighborhood_painter_subclass_and_walls() {
    // Painter subclass (the canonical PainterPlus level), a wall stops
    // the run, and the default painter has infinite paint on a 20x20.
    let mut grid = String::new();
    for _ in 0..20 {
        let row: Vec<&str> = std::iter::repeat_n("1,0", 20).collect();
        grid.push_str(&row.join(" "));
        grid.push('\n');
    }
    let out = run_neighborhood(
        r#"
        import org.code.neighborhood.*;
        class PainterPlus extends Painter {
            public void turnRight() { turnLeft(); turnLeft(); turnLeft(); }
            public void moveFast() { while (canMove()) { move(); paint("blue"); } }
        }
        public class Runner {
            public static void main(String[] args) {
                PainterPlus m = new PainterPlus();
                System.out.println("infinitePaint=" + m.hasPaint());
                m.paint("red");
                m.moveFast();
                m.turnRight();
                m.moveFast();
                System.out.println("x=" + m.getX() + " y=" + m.getY() + " facing=" + m.getDirection());
            }
        }
        "#,
        "Runner",
        &grid,
    );
    assert_eq!(out, "infinitePaint=true\nx=19 y=19 facing=south\n");
}

#[test]
fn arrays_tostring_calls_element_tostring() {
    // Arrays.toString on a user-object array must call each element's
    // toString (not print addresses), so equal-content arrays compare equal.
    let out = run_stdout(
        r#"
        import java.util.*;
        public class Main {
            public static void main(String[] args) {
                Point[] a = { new Point(1, 2), new Point(3, 4) };
                Point[] b = { new Point(1, 2), new Point(3, 4) };
                System.out.println(Arrays.toString(a));
                System.out.println(Arrays.toString(a).equals(Arrays.toString(b)));
            }
        }
        class Point {
            private int x, y;
            public Point(int x, int y) { this.x = x; this.y = y; }
            public String toString() { return "(" + x + ", " + y + ")"; }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "[(1, 2), (3, 4)]\ntrue\n");
}

#[test]
fn structural_reflection_prints_field_signatures() {
    // The AttributesHelper surface end to end: getClass, getSuperclass,
    // getDeclaredFields, canonical Field.toString via Arrays.toString.
    let out = run_stdout(
        r"
        import java.lang.reflect.*;
        import java.util.*;
        public class Main {
            public static void main(String[] args) {
                Object o = new Dog();
                Class c = o.getClass();
                System.out.println(c.getSimpleName());
                System.out.println(c.getSuperclass().getSimpleName());
                Field[] fs = c.getDeclaredFields();
                System.out.println(Arrays.toString(fs));
            }
        }
        class Animal { private int legs; }
        class Dog extends Animal { private String name; public int age; }
        ",
        "Main",
    );
    assert_eq!(
        out,
        "Dog\nAnimal\n[private java.lang.String Dog.name, public int Dog.age]\n"
    );
}

#[test]
fn new_string_constructor() {
    // `new String(...)`: empty, copy-of-String (distinct reference), char[].
    let out = run_stdout(
        r#"
        public class Main {
            public static void main(String[] args) {
                char[] cs = {'h', 'i'};
                System.out.println(new String("Welcome"));
                System.out.println("[" + new String() + "]");
                System.out.println(new String(cs));
                String x = new String("cat");
                System.out.println(x.equals("cat") + " " + (x == "cat"));
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "Welcome\n[]\nhi\ntrue false\n");
}

#[test]
fn structural_reflection_prints_constructor_signatures() {
    // Class.getDeclaredConstructors + canonical Constructor.toString,
    // the ConstructorsHelper surface.
    let out = run_stdout(
        r#"
        import java.lang.reflect.*;
        import java.util.*;
        public class Main {
            public static void main(String[] args) {
                Constructor[] cs = new Dessert().getClass().getDeclaredConstructors();
                System.out.println(Arrays.toString(cs));
            }
        }
        class Dessert {
            private String flavor;
            public Dessert() { flavor = "vanilla"; }
            public Dessert(String f, int n) { flavor = f; }
        }
        "#,
        "Main",
    );
    assert_eq!(
        out,
        "[public Dessert(), public Dessert(java.lang.String,int)]\n"
    );
}

#[test]
fn junit_validation_runner_reports_pass_and_fail() {
    // A JUnit validator with a passing and a failing test; the injected
    // __ValidationRunner runs each and reports per-test outcomes.
    let compilation = caturra_compiler::compile(&[
        caturra_compiler::SourceFile {
            path: "Counter.java".into(),
            text: "public class Counter { private int count = 0;                    public void increment() { count += 2; }                    public int getCount() { return count; } }"
                .into(),
        },
        caturra_compiler::SourceFile {
            path: "CounterTest.java".into(),
            text: r#"
                import static org.junit.jupiter.api.Assertions.*;
                import org.junit.jupiter.api.Test;
                import org.junit.jupiter.api.Order;
                import org.junit.jupiter.api.DisplayName;
                public class CounterTest {
                    @Test @Order(1) @DisplayName("starts at zero")
                    public void a() { assertEquals(0, new Counter().getCount()); }
                    @Test @Order(2) @DisplayName("increments by one")
                    public void b() { Counter c = new Counter(); c.increment(); assertEquals(1, c.getCount()); }
                }
            "#
            .into(),
        },
    ]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let entry = compilation
        .validation_entry
        .clone()
        .expect("validation entry");
    assert_eq!(entry, "__ValidationRunner");
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    vm.run_main(&entry, &[]).expect("run");
    assert_eq!(
        console.stdout_text(),
        "__VTEST\tPASS\tstarts at zero\t\n\
         __VTEST\tFAIL\tincrements by one\texpected 1 but was 2\n"
    );
}

#[test]
fn junit_validator_static_import_and_relaxed_access() {
    // The validation "Test" mode: `import static Assertions.*` makes
    // assertX unqualified, org.junit relaxes private access so a validator
    // can read internals, and assertions throw on failure.
    let out = run_stdout(
        r#"
        import static org.junit.jupiter.api.Assertions.*;
        class Box { private int n = 3; }
        public class Main {
            public static void main(String[] args) {
                Box b = new Box();
                assertEquals(3, b.n);
                assertTrue(b.n > 0);
                System.out.println("assertions passed");
                try {
                    assertEquals(9, b.n);
                    System.out.println("no throw");
                } catch (RuntimeException e) {
                    System.out.println("caught: " + e.getMessage());
                }
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "assertions passed\ncaught: expected 9 but was 3\n");
}

#[test]
fn codeorg_console_patterns() {
    let out = run_stdout(
        r#"
        class Character {
            String name;
            Character(String name) { this.name = name; }
            public String toString() { return name; }
        }
        class Account {
            protected double balance;
            Account(double b) { balance = b; }
            void withdraw(double a) { balance -= a; }
        }
        class BasicAccount extends Account {
            BasicAccount(double b) { super(b); }
            void withdraw(double a) { super.withdraw(a); }
        }
        class Store { public static String tax = "10%"; }
        public class Co {
            public static void main(String[] args) {
                double d = .25;
                Integer i = new Integer(40);
                Integer p = new Integer("2");
                System.out.println(d + " " + (i + p));
                java.util.ArrayList<Character> c = new java.util.ArrayList<Character>();
                c.add(new Character("Mario"));
                System.out.println(c.get(0) + " " + c.size());
                Store s = new Store();
                System.out.println(s.tax);
                s.tax = "0%";
                System.out.println(Store.tax);
                BasicAccount a = new BasicAccount(100.0);
                a.withdraw(30.0);
                System.out.println(a.balance);
            }
        }
        "#,
        "Co",
    );
    assert_eq!(out, "0.25 42\nMario 1\n10%\n0%\n70.0\n");
}

#[test]
fn comparable_interface_and_sorting() {
    let out = run_stdout(
        r#"
        class Card implements Comparable<Card> {
            private int rank;
            public Card(int rank) { this.rank = rank; }
            public int compareTo(Card other) { return rank - other.rank; }
            public String toString() { return "" + rank; }
        }
        public class Cmp {
            static void sort(Comparable[] arr) {
                for (int i = 0; i < arr.length; i++) {
                    int min = i;
                    for (int j = i + 1; j < arr.length; j++)
                        if (arr[j].compareTo(arr[min]) < 0) min = j;
                    Comparable t = arr[min]; arr[min] = arr[i]; arr[i] = t;
                }
            }
            public static void main(String[] args) {
                Card a = new Card(8), b = new Card(3);
                System.out.println((a.compareTo(b) > 0) + " " + a.compareTo(a));
                Card[] cards = { new Card(5), new Card(1), new Card(9), new Card(3) };
                sort(cards);
                for (Card c : cards) System.out.print(c + " ");
                System.out.println();
            }
        }
        "#,
        "Cmp",
    );
    assert_eq!(out, "true 0\n1 3 5 9 \n");
}

#[test]
fn method_references_all_kinds() {
    let out = run_stdout(
        r#"
        interface StrToInt { int apply(String s); }
        interface IntBiOp { int apply(int a, int b); }
        interface Factory { Cell make(int v); }
        class Cell {
            int v;
            Cell(int v) { this.v = v; }
            public String toString() { return "Cell:" + v; }
        }
        public class Mr {
            public static void main(String[] args) {
                StrToInt len = String::length;
                StrToInt parse = Integer::parseInt;
                IntBiOp mx = Math::max;
                Factory f = Cell::new;
                System.out.println(len.apply("hello") + " " + parse.apply("77"));
                System.out.println(mx.apply(4, 9) + " " + f.make(3));
            }
        }
        "#,
        "Mr",
    );
    assert_eq!(out, "5 77\n9 Cell:3\n");
}

#[test]
fn lambdas_target_typed_and_capturing() {
    let out = run_stdout(
        r#"
        interface IntFn { int apply(int x); }
        interface BiFn { int apply(int a, int b); }
        interface Act { void run(); }
        public class La {
            static int applyTo(IntFn f, int v) { return f.apply(v); }
            public static void main(String[] args) {
                IntFn inc = x -> x + 1;
                BiFn add = (a, b) -> a + b;
                System.out.println(inc.apply(9) + " " + add.apply(3, 4));
                System.out.println(applyTo(x -> x * x, 5));
                int base = 100;
                IntFn shift = x -> x + base;
                System.out.println(shift.apply(7));
                Act a = () -> System.out.print("hi ");
                a.run();
                System.out.println("done");
            }
        }
        "#,
        "La",
    );
    assert_eq!(out, "10 7\n25\n107\nhi done\n");
}

#[test]
fn anonymous_class_captures_locals() {
    let out = run_stdout(
        r#"
        interface Fn { int apply(int x); }
        public class Cap {
            public static void main(String[] args) {
                int offset = 100;
                int scale = 2;
                Fn f = new Fn() {
                    public int apply(int x) { return x * scale + offset; }
                };
                System.out.println(f.apply(5) + " " + f.apply(10));
                String tag = "n=";
                Fn g = new Fn() {
                    private int calls = 0;
                    public int apply(int x) {
                        calls++;
                        System.out.print(tag + calls + " ");
                        return x + offset;
                    }
                };
                System.out.println(g.apply(1) + " " + g.apply(2));
            }
        }
        "#,
        "Cap",
    );
    assert_eq!(out, "110 120\nn=1 n=2 101 102\n");
}

#[test]
fn anonymous_class_implements_interface() {
    let out = run_stdout(
        r#"
        interface Op { int apply(int a, int b); }
        public class Anon {
            static int run(Op op, int x, int y) { return op.apply(x, y); }
            public static void main(String[] args) {
                Op add = new Op() {
                    public int apply(int a, int b) { return a + b; }
                };
                Op mul = new Op() {
                    public int apply(int a, int b) { return a * b; }
                };
                System.out.println(add.apply(3, 4) + " " + mul.apply(3, 4));
                System.out.println(run(add, 10, 20) + " " + run(mul, 6, 7));
            }
        }
        "#,
        "Anon",
    );
    assert_eq!(out, "7 12\n30 42\n");
}

#[test]
fn autoboxing_wrappers_and_arithmetic() {
    let out = run_stdout(
        r#"
        public class Box {
            static int take(Integer n) { return n; }
            public static void main(String[] args) {
                Integer i = 5;
                int x = i;
                Double d = 2.5;
                double y = d;
                Object o = 42;
                System.out.println(i + " " + x + " " + d + " " + y + " " + o);
                Integer a = 10, b = 4;
                System.out.println((a + b) + " " + (a * b) + " " + (a > b) + " " + a.compareTo(b));
                System.out.println(take(99) + " " + i.doubleValue());
            }
        }
        "#,
        "Box",
    );
    assert_eq!(out, "5 5 2.5 2.5 42\n14 40 true 1\n99 5.0\n");
}

#[test]
fn generic_class_and_object_top_type() {
    let out = run_stdout(
        r#"
        class Holder<T> {
            private T item;
            void put(T item) { this.item = item; }
            T take() { return item; }
        }
        public class Gen {
            static <T> T identity(T x) { return x; }
            public static void main(String[] args) {
                Holder<String> h = new Holder<>();
                h.put("cached");
                String s = h.take();
                System.out.println(s + " " + s.length());
                System.out.println((String) identity("id"));
                Object o = "top";
                System.out.println(o + " " + o.equals("top"));
            }
        }
        "#,
        "Gen",
    );
    assert_eq!(out, "cached 6\nid\ntop true\n");
}

#[test]
fn nested_classes_and_field_chains() {
    let out = run_stdout(
        r#"
        public class Chain {
            static class Cell {
                int v;
                Cell link;
                Cell(int v) { this.v = v; }
            }
            static class Holder {
                Cell head;
                int firstTwo() { return head.v + head.link.v; }
            }
            public static void main(String[] args) {
                Cell a = new Cell(3);
                a.link = new Cell(4);
                a.link.link = new Cell(5);
                // Deep field-access chain and mutation through it.
                System.out.println(a.v + a.link.v + a.link.link.v);
                a.link.link.v = 50;
                System.out.println(a.link.link.v);
                Holder h = new Holder();
                h.head = a;
                System.out.println(h.firstTwo() + " " + h.head.link.v);
            }
        }
        "#,
        "Chain",
    );
    assert_eq!(out, "12\n50\n7 4\n");
}

#[test]
fn enums_singletons_values_and_switch() {
    let out = run_stdout(
        r#"
        enum Coin {
            PENNY(1), NICKEL(5), DIME(10), QUARTER(25);
            private final int cents;
            Coin(int cents) { this.cents = cents; }
            int cents() { return cents; }
        }
        public class Money {
            public static void main(String[] args) {
                int total = 0;
                for (Coin c : Coin.values()) total += c.cents();
                System.out.println(total + " " + Coin.values().length);
                Coin c = Coin.valueOf("DIME");
                System.out.println(c + " " + c.ordinal() + " " + c.cents());
                String kind = "";
                switch (c) {
                    case PENNY: kind = "copper"; break;
                    case DIME: case QUARTER: kind = "silver"; break;
                    default: kind = "other";
                }
                System.out.println(kind + " " + (c == Coin.DIME));
            }
        }
        "#,
        "Money",
    );
    assert_eq!(out, "41 4\nDIME 2 10\nsilver true\n");
}

#[test]
fn arithmetic_follows_java_semantics() {
    let out = run_stdout(
        r"
        public class Arith {
            public static void main(String[] args) {
                int a = 7, b = 2;
                System.out.println(a + b);
                System.out.println(a - b);
                System.out.println(a * b);
                System.out.println(a / b);
                System.out.println(a % b);
                System.out.println(-a);
                System.out.println(a / 2.0);
                System.out.println(1 + 2 * 3);
                System.out.println((1 + 2) * 3);
                System.out.println(2147483647 + 1);
            }
        }
        ",
        "Arith",
    );
    assert_eq!(out, "9\n5\n14\n3\n1\n-7\n3.5\n7\n9\n-2147483648\n");
}

#[test]
fn division_by_zero_throws_like_java() {
    let (result, console) = compile_and_run(
        r"
        public class Div {
            public static void main(String[] args) {
                int zero = 0;
                System.out.println(1 / zero);
            }
        }
        ",
        "Div",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.ArithmeticException: / by zero"),
        "{}",
        console.stderr_text()
    );
    // Double division by zero is Infinity, not an exception.
    let out = run_stdout(
        r"
        public class DDiv {
            public static void main(String[] args) {
                double zero = 0.0;
                System.out.println(1.0 / zero);
            }
        }
        ",
        "DDiv",
    );
    assert_eq!(out, "Infinity\n");
}

#[test]
fn comparisons_and_logic_short_circuit() {
    let out = run_stdout(
        r"
        public class Logic {
            public static void main(String[] args) {
                int x = 5;
                System.out.println(x > 3);
                System.out.println(x <= 4);
                System.out.println(x == 5 && x != 5);
                System.out.println(x == 5 || x != 5);
                System.out.println(!(x > 3));
                int zero = 0;
                System.out.println(zero != 0 && 10 / zero > 1);
                System.out.println(zero == 0 || 10 / zero > 1);
                System.out.println(1.5 > 1.4);
                System.out.println(0.0 / zero == 0.0 / zero);
            }
        }
        ",
        "Logic",
    );
    // The two short-circuit lines must not throw ArithmeticException,
    // and NaN == NaN is false.
    assert_eq!(
        out,
        "true\nfalse\nfalse\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\n"
    );
}

#[test]
fn assignment_compound_and_increment() {
    let out = run_stdout(
        r"
        public class Assign {
            public static void main(String[] args) {
                int x = 10;
                x += 5;
                x -= 3;
                x *= 2;
                x /= 4;
                x %= 4;
                System.out.println(x);
                x++;
                ++x;
                x--;
                System.out.println(x);
                double d = 1.5;
                d *= 2;
                System.out.println(d);
                int narrowed = 7;
                narrowed += 0.5;
                System.out.println(narrowed);
                char c = 'a';
                c++;
                System.out.println(c);
            }
        }
        ",
        "Assign",
    );
    // x: 10+5-3=12*2=24/4=6%4=2; then ++,++,-- -> 3.
    // narrowed: (int)(7 + 0.5) = 7. c: 'a'+1 = 'b'.
    assert_eq!(out, "2\n3\n3.0\n7\nb\n");
}

#[test]
fn string_concatenation_matches_java() {
    let out = run_stdout(
        r#"
        public class Concat {
            public static void main(String[] args) {
                int x = 42;
                double d = 2.0;
                boolean flag = true;
                char c = '!';
                String s = "x = " + x;
                System.out.println(s);
                System.out.println("d = " + d);
                System.out.println(flag + " story" + c);
                System.out.println(1 + 2 + "a");
                System.out.println("a" + 1 + 2);
                String nothing = null;
                System.out.println("value: " + nothing);
                s += "!";
                System.out.println(s);
            }
        }
        "#,
        "Concat",
    );
    assert_eq!(
        out,
        "x = 42\nd = 2.0\ntrue story!\n3a\na12\nvalue: null\nx = 42!\n"
    );
}

#[test]
fn casts_and_promotions() {
    let out = run_stdout(
        r"
        public class Casts {
            public static void main(String[] args) {
                double d = 9.99;
                System.out.println((int) d);
                System.out.println((int) -9.99);
                int i = 65;
                System.out.println((char) i);
                System.out.println((double) i);
                char c = 'A';
                int fromChar = c + 1;
                System.out.println(fromChar);
                System.out.println((char) (c + 1));
            }
        }
        ",
        "Casts",
    );
    assert_eq!(out, "9\n-9\nA\n65.0\n66\nB\n");
}

#[test]
fn string_reference_equality_of_literals() {
    // Literals are interned, so == is true for identical literals; a
    // runtime-built string is a different object.
    let out = run_stdout(
        r#"
        public class Eq {
            public static void main(String[] args) {
                String a = "hi";
                String b = "hi";
                System.out.println(a == b);
                String c = a + "!";
                String d = "hi!";
                System.out.println(c == d);
            }
        }
        "#,
        "Eq",
    );
    assert_eq!(out, "true\nfalse\n");
}

// ----- stage 2: control flow -----

#[test]
fn if_else_chains_and_dangling_else() {
    let out = run_stdout(
        r#"
        public class Grades {
            public static void main(String[] args) {
                int score = 85;
                if (score >= 90) System.out.println("A");
                else if (score >= 80) System.out.println("B");
                else if (score >= 70) System.out.println("C");
                else System.out.println("F");

                // Dangling else binds to the inner if.
                int x = 5;
                if (x > 0)
                    if (x > 10) System.out.println("big");
                    else System.out.println("small");
            }
        }
        "#,
        "Grades",
    );
    assert_eq!(out, "B\nsmall\n");
}

#[test]
fn while_and_do_while_semantics() {
    let out = run_stdout(
        r"
        public class Loops {
            public static void main(String[] args) {
                int n = 3;
                while (n > 0) {
                    System.out.println(n);
                    n--;
                }
                // The while body never runs; the do-while body runs once.
                while (false == true) System.out.println(-1);
                do System.out.println(99); while (false);
            }
        }
        ",
        "Loops",
    );
    assert_eq!(out, "3\n2\n1\n99\n");
}

#[test]
fn for_loops_sum_and_nest() {
    let out = run_stdout(
        r#"
        public class Sums {
            public static void main(String[] args) {
                int total = 0;
                for (int i = 1; i <= 10; i++) {
                    total += i;
                }
                System.out.println(total);

                String grid = "";
                for (int r = 0; r < 3; r++) {
                    for (int c = 0; c < 3; c++) {
                        grid += r * 3 + c;
                    }
                    grid += "|";
                }
                System.out.println(grid);

                // Two loops can reuse the same index name (loop scoping),
                // and multi-init/multi-update headers work.
                for (int i = 0, j = 3; i < j; i++, j--) {
                    System.out.println(i + "," + j);
                }
            }
        }
        "#,
        "Sums",
    );
    assert_eq!(out, "55\n012|345|678|\n0,3\n1,2\n");
}

#[test]
fn break_and_continue() {
    let out = run_stdout(
        r"
        public class Jumps {
            public static void main(String[] args) {
                for (int i = 0; i < 10; i++) {
                    if (i % 2 == 0) continue;
                    if (i > 6) break;
                    System.out.println(i);
                }
                // break/continue bind to the INNER loop.
                int found = 0;
                for (int i = 0; i < 3; i++) {
                    for (int j = 0; j < 3; j++) {
                        if (j > i) break;
                        found++;
                    }
                }
                System.out.println(found);
                int n = 0;
                while (true) {
                    n++;
                    if (n == 5) break;
                }
                System.out.println(n);
            }
        }
        ",
        "Jumps",
    );
    assert_eq!(out, "1\n3\n5\n6\n5\n");
}

#[test]
fn fizzbuzz_works() {
    let out = run_stdout(
        r#"
        public class FizzBuzz {
            public static void main(String[] args) {
                for (int i = 1; i <= 15; i++) {
                    if (i % 15 == 0) System.out.println("FizzBuzz");
                    else if (i % 3 == 0) System.out.println("Fizz");
                    else if (i % 5 == 0) System.out.println("Buzz");
                    else System.out.println(i);
                }
            }
        }
        "#,
        "FizzBuzz",
    );
    assert_eq!(
        out,
        "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n"
    );
}

#[test]
fn definite_assignment_across_branches() {
    // Both branches assign: OK.
    let ok = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "T.java".into(),
        text: r"
            class T {
                static void f() {
                    int x;
                    if (1 < 2) { x = 1; } else { x = 2; }
                    System.out.println(x);
                }
            }
        "
        .into(),
    }]);
    assert!(ok.success(), "{:?}", ok.diagnostics);

    // Only one branch assigns: javac-style error.
    let bad = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "T.java".into(),
        text: r"
            class T {
                static void f() {
                    int x;
                    if (1 < 2) { x = 1; }
                    System.out.println(x);
                }
            }
        "
        .into(),
    }]);
    assert!(!bad.success());
    assert!(
        bad.diagnostics[0]
            .message
            .contains("might not have been initialized"),
        "{:?}",
        bad.diagnostics
    );

    // Assignment only inside a loop body doesn't count either.
    let loopy = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "T.java".into(),
        text: r"
            class T {
                static void f() {
                    int x;
                    while (1 < 2) { x = 1; }
                    System.out.println(x);
                }
            }
        "
        .into(),
    }]);
    assert!(!loopy.success());
}

#[test]
fn infinite_loop_hits_the_instruction_budget() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "Spin.java".into(),
        text: r"
            public class Spin {
                public static void main(String[] args) {
                    while (true) { }
                }
            }
        "
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(
        VmOptions {
            max_instructions: 10_000,
            ..VmOptions::default()
        },
        &mut vfs,
        &mut console,
    );
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Spin", &[]);
    assert!(
        matches!(result, Err(VmError::InstructionBudgetExceeded)),
        "{result:?}"
    );
}

// ----- stage 3: user-defined static methods -----

#[test]
fn calls_helpers_with_params_and_returns() {
    let out = run_stdout(
        r#"
        public class Temps {
            public static double toFahrenheit(double celsius) {
                return celsius * 9 / 5 + 32;
            }

            public static String describe(double f) {
                if (f >= 80) return "hot";
                if (f >= 60) return "mild";
                return "cold";
            }

            static void report(double celsius) {
                double f = toFahrenheit(celsius);
                System.out.println(celsius + "C = " + f + "F (" + describe(f) + ")");
            }

            public static void main(String[] args) {
                report(30.0);
                report(15);
                report(-5.0);
            }
        }
        "#,
        "Temps",
    );
    assert_eq!(
        out,
        "30.0C = 86.0F (hot)\n15.0C = 59.0F (cold)\n-5.0C = 23.0F (cold)\n"
    );
}

#[test]
fn recursion_and_mutual_recursion() {
    let out = run_stdout(
        r"
        public class Rec {
            static int factorial(int n) {
                if (n <= 1) return 1;
                return n * factorial(n - 1);
            }

            static int fib(int n) {
                if (n < 2) return n;
                return fib(n - 1) + fib(n - 2);
            }

            static boolean isEven(int n) {
                if (n == 0) return true;
                return isOdd(n - 1);
            }

            static boolean isOdd(int n) {
                if (n == 0) return false;
                return isEven(n - 1);
            }

            public static void main(String[] args) {
                System.out.println(factorial(10));
                System.out.println(fib(15));
                System.out.println(isEven(20));
                System.out.println(isOdd(20));
            }
        }
        ",
        "Rec",
    );
    assert_eq!(out, "3628800\n610\ntrue\nfalse\n");
}

#[test]
fn cross_class_static_calls_and_overloads() {
    let out = run_stdout(
        r#"
        class MathUtil {
            static int max(int a, int b) {
                if (a > b) return a;
                return b;
            }

            static double max(double a, double b) {
                if (a > b) return a;
                return b;
            }

            static String label(int v) { return "int:" + v; }
            static String label(double v) { return "double:" + v; }
        }

        public class Uses {
            public static void main(String[] args) {
                System.out.println(MathUtil.max(3, 9));
                System.out.println(MathUtil.max(2.5, 1.5));
                System.out.println(MathUtil.label(7));
                System.out.println(MathUtil.label(7.0));
                System.out.println(MathUtil.max(1, 2.5));
                char c = 'a';
                System.out.println(MathUtil.label(c));
            }
        }
        "#,
        "Uses",
    );
    // Exact overload wins for ints; widening picks the double overload
    // for max(1, 2.5); char widens to int so label('a') prints 97.
    assert_eq!(out, "9\n2.5\nint:7\ndouble:7.0\n2.5\nint:97\n");
}

#[test]
fn void_call_statements_and_value_discard() {
    let out = run_stdout(
        r#"
        public class Effects {
            static int counter() {
                System.out.println("counted");
                return 42;
            }

            static double wide() { return 1.5; }

            public static void main(String[] args) {
                // Results discarded (pop / pop2), side effects kept.
                counter();
                wide();
                System.out.println("done");
            }
        }
        "#,
        "Effects",
    );
    assert_eq!(out, "counted\ndone\n");
}

#[test]
fn runaway_recursion_overflows_like_java() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "Deep.java".into(),
        text: r"
        public class Deep {
            static int down(int n) {
                return down(n + 1);
            }

            public static void main(String[] args) {
                System.out.println(down(0));
            }
        }
        "
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    // The default limit (4096) is safe to exhaust: frames live on the
    // heap, not the host stack.
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Deep", &[]);
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.StackOverflowError"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn stage3_compile_errors_match_javac_wording() {
    let cases: &[(&str, &str)] = &[
        ("static int f() { int x = 1; }", "missing return statement"),
        ("static void f() { return 5; }", "unexpected return value"),
        ("static int f() { return; }", "missing return value"),
        (
            "static void f() { h(1); }",
            "cannot find symbol: method h(int)",
        ),
        (
            "static void f(int a, double b) { } static void f(double a, int b) { } \
             static void g() { f(1, 2); }",
            "reference to f is ambiguous",
        ),
        (
            "void inst() { } static void f() { inst(); }",
            "non-static method inst() cannot be referenced from a static context",
        ),
        (
            "static int f() { return 1; } static int f() { return 2; }",
            "method f() is already defined",
        ),
        (
            "static void g() { } static void f() { int x = g(); }",
            "returns void",
        ),
    ];
    for (body, expected) in cases {
        let result = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: "T.java".into(),
            text: format!("class T {{ {body} }}"),
        }]);
        assert!(!result.success(), "case '{body}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{body}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn while_true_with_return_needs_no_trailing_return() {
    let out = run_stdout(
        r"
        public class Search {
            static int firstSquareAbove(int limit) {
                int n = 1;
                while (true) {
                    if (n * n > limit) return n * n;
                    n++;
                }
            }

            public static void main(String[] args) {
                System.out.println(firstSquareAbove(50));
            }
        }
        ",
        "Search",
    );
    assert_eq!(out, "64\n");
}

// ----- stage 4: arrays -----

#[test]
fn arrays_create_fill_and_read() {
    let out = run_stdout(
        r#"
        public class Arrays1 {
            public static void main(String[] args) {
                int[] squares = new int[5];
                for (int i = 0; i < squares.length; i++) {
                    squares[i] = i * i;
                }
                for (int i = 0; i < squares.length; i++) {
                    System.out.print(squares[i]);
                    System.out.print(" ");
                }
                System.out.println();

                double[] temps = {98.6, 99.1, 97.5};
                System.out.println(temps[1]);
                boolean[] flags = new boolean[2];
                System.out.println(flags[0]);
                char[] letters = {'j', 'v', 'm'};
                letters[0] = 'J';
                System.out.println("" + letters[0] + letters[1] + letters[2]);
                String[] names = {"Ada", "Grace"};
                System.out.println(names[0] + " & " + names[1]);
            }
        }
        "#,
        "Arrays1",
    );
    assert_eq!(out, "0 1 4 9 16 \n99.1\nfalse\nJvm\nAda & Grace\n");
}

#[test]
fn array_algorithms_and_for_each() {
    let out = run_stdout(
        r#"
        public class Algo {
            static int sum(int[] values) {
                int total = 0;
                for (int v : values) total += v;
                return total;
            }

            static int max(int[] values) {
                int best = values[0];
                for (int v : values) {
                    if (v > best) best = v;
                }
                return best;
            }

            static void reverse(int[] values) {
                for (int i = 0; i < values.length / 2; i++) {
                    int tmp = values[i];
                    values[i] = values[values.length - 1 - i];
                    values[values.length - 1 - i] = tmp;
                }
            }

            public static void main(String[] args) {
                int[] data = {3, 1, 4, 1, 5, 9, 2, 6};
                System.out.println(sum(data));
                System.out.println(max(data));
                reverse(data);
                String s = "";
                for (int v : data) s += v;
                System.out.println(s);
                data[0] += 10;
                data[1]++;
                System.out.println(data[0] + "," + data[1]);
            }
        }
        "#,
        "Algo",
    );
    assert_eq!(out, "31\n9\n62951413\n16,3\n");
}

#[test]
fn two_dimensional_arrays() {
    let out = run_stdout(
        r#"
        public class Grid {
            public static void main(String[] args) {
                int[][] m = new int[3][4];
                for (int r = 0; r < m.length; r++) {
                    for (int c = 0; c < m[r].length; c++) {
                        m[r][c] = r * 10 + c;
                    }
                }
                System.out.println(m.length + "x" + m[0].length);
                System.out.println(m[2][3]);

                int[][] jagged = {{1}, {2, 3}, {4, 5, 6}};
                int total = 0;
                for (int[] row : jagged) {
                    total += row.length;
                }
                System.out.println(total);
                System.out.println(jagged[2][1]);

                // Row objects are references: aliasing is visible.
                int[] alias = m[1];
                alias[0] = 99;
                System.out.println(m[1][0]);
            }
        }
        "#,
        "Grid",
    );
    assert_eq!(out, "3x4\n23\n6\n5\n99\n");
}

#[test]
fn array_exceptions_match_java_wording() {
    let (result, console) = compile_and_run(
        r"
        public class Oob {
            public static void main(String[] args) {
                int[] a = new int[3];
                int i = 5;
                System.out.println(a[i]);
            }
        }
        ",
        "Oob",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console.stderr_text().contains(
            "java.lang.ArrayIndexOutOfBoundsException: Index 5 out of bounds for length 3"
        ),
        "{}",
        console.stderr_text()
    );

    let (result, console) = compile_and_run(
        r"
        public class Neg {
            public static void main(String[] args) {
                int n = -2;
                int[] a = new int[n];
                System.out.println(a.length);
            }
        }
        ",
        "Neg",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.NegativeArraySizeException: -2"),
        "{}",
        console.stderr_text()
    );

    let (result, console) = compile_and_run(
        r"
        public class NullRow {
            public static void main(String[] args) {
                int[][] m = new int[2][];
                System.out.println(m[0][0]);
            }
        }
        ",
        "NullRow",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.NullPointerException"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn main_args_are_finally_usable() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "Args.java".into(),
        text: r#"
        public class Args {
            public static void main(String[] args) {
                System.out.println(args.length);
                for (String arg : args) {
                    System.out.println("arg: " + arg);
                }
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Args", &["hello".into(), "world".into()]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "2\narg: hello\narg: world\n");
}

// ----- stage 5: objects -----

#[test]
fn classic_bank_account_class() {
    let out = run_stdout(
        r#"
        class Account {
            private String owner;
            private double balance;
            private static int accountCount = 0;

            public Account(String owner, double start) {
                this.owner = owner;
                balance = start;
                accountCount++;
            }

            public Account(String owner) {
                this.owner = owner;
                this.balance = 0.0;
                accountCount++;
            }

            public void deposit(double amount) {
                if (amount > 0) balance += amount;
            }

            public boolean withdraw(double amount) {
                if (amount <= balance) {
                    balance -= amount;
                    return true;
                }
                return false;
            }

            public double getBalance() { return balance; }
            public static int getCount() { return accountCount; }

            public String toString() {
                return owner + ": $" + balance;
            }
        }

        public class Bank {
            public static void main(String[] args) {
                Account a = new Account("Ada", 100.0);
                Account b = new Account("Grace");
                a.deposit(50.0);
                b.deposit(25.0);
                System.out.println(a.withdraw(200.0));
                System.out.println(a.withdraw(75.0));
                System.out.println(a);
                System.out.println(b);
                System.out.println("accounts: " + Account.getCount());
            }
        }
        "#,
        "Bank",
    );
    assert_eq!(out, "false\ntrue\nAda: $75.0\nGrace: $25.0\naccounts: 2\n");
}

#[test]
fn field_initializers_defaults_and_statics() {
    let out = run_stdout(
        r#"
        class Config {
            static int created = 0;
            static final double VERSION = 2.5;
            int id = next();
            String label;
            boolean enabled;

            static int next() {
                created++;
                return created;
            }
        }

        public class Defaults {
            public static void main(String[] args) {
                Config first = new Config();
                Config second = new Config();
                System.out.println(first.id + " " + second.id);
                System.out.println(second.label + " " + second.enabled);
                System.out.println(Config.VERSION);
                System.out.println(Config.created);
            }
        }
        "#,
        "Defaults",
    );
    assert_eq!(out, "1 2\nnull false\n2.5\n2\n");
}

#[test]
fn objects_are_references() {
    let out = run_stdout(
        r#"
        class Point {
            int x;
            int y;

            Point(int x, int y) {
                this.x = x;
                this.y = y;
            }

            void translate(int dx, int dy) {
                x += dx;
                y += dy;
            }

            public String toString() { return "(" + x + ", " + y + ")"; }
        }

        public class Refs {
            static void moveIt(Point p) {
                p.translate(10, 10);
            }

            public static void main(String[] args) {
                Point a = new Point(1, 2);
                Point alias = a;
                moveIt(a);
                System.out.println(alias);
                System.out.println(a == alias);
                System.out.println(a == new Point(11, 12));

                Point[] path = new Point[2];
                System.out.println(path[0]);
                path[0] = a;
                path[0].x = 99;
                System.out.println(a.x);
            }
        }
        "#,
        "Refs",
    );
    assert_eq!(out, "(11, 12)\ntrue\nfalse\nnull\n99\n");
}

#[test]
fn default_tostring_prints_class_at_hex() {
    let out = run_stdout(
        r#"
        class Ghost { }

        public class Spooky {
            public static void main(String[] args) {
                Ghost g = new Ghost();
                System.out.println(g);
                System.out.println("boo: " + g);
            }
        }
        "#,
        "Spooky",
    );
    let mut lines = out.lines();
    let first = lines.next().expect("one line");
    assert!(first.starts_with("Ghost@"), "{first}");
    let second = lines.next().expect("two lines");
    assert!(second.starts_with("boo: Ghost@"), "{second}");
}

#[test]
fn npe_on_null_receiver_and_field() {
    let (result, console) = compile_and_run(
        r"
        class Widget {
            int size;
            void grow() { size++; }
        }

        public class Broken {
            public static void main(String[] args) {
                Widget w = null;
                w.grow();
            }
        }
        ",
        "Broken",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.NullPointerException"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn stage5_compile_errors_match_javac_wording() {
    let cases: &[(&str, &str)] = &[
        (
            "class A { private int x; } class B { static void f() { A a = new A(); \
             System.out.println(a.x); } }",
            "x has private access in A",
        ),
        (
            "class A { private void m() { } } class B { static void f() { A a = new A(); \
             a.m(); } }",
            "m() has private access in A",
        ),
        (
            "class A { int x; static void f() { System.out.println(x); } }",
            "non-static variable x cannot be referenced from a static context",
        ),
        (
            "class A { int x; static void f() { System.out.println(this.x); } }",
            "non-static variable this cannot be referenced from a static context",
        ),
        (
            "class Pt { Pt(int x) { } } class B { static void f() { Pt p = new Pt(); } }",
            "constructor Pt in class Pt cannot be applied to given types",
        ),
        (
            "class A { int x; int x; }",
            "variable x is already defined in class A",
        ),
        (
            "class A { A(int v) { } A(int w) { } }",
            "constructor A is already defined",
        ),
        (
            "class A { final int x = 1; void f() { x = 2; } }",
            "cannot assign a value to final variable x",
        ),
    ];
    for (source, expected) in cases {
        let result = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: "T.java".into(),
            text: (*source).into(),
        }]);
        assert!(!result.success(), "case '{source}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{source}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

// ----- stage 6: inheritance -----

#[test]
fn classic_shape_hierarchy_with_polymorphism() {
    let out = run_stdout(
        r#"
        abstract class Shape {
            private String name;

            Shape(String name) {
                this.name = name;
            }

            abstract double area();

            String getName() { return name; }

            public String toString() {
                return name + " with area " + area();
            }
        }

        class Circle extends Shape {
            private double radius;

            Circle(double radius) {
                super("circle");
                this.radius = radius;
            }

            double area() { return 3.14159 * radius * radius; }
        }

        class Rect extends Shape {
            private double w;
            private double h;

            Rect(double w, double h) {
                super("rect");
                this.w = w;
                this.h = h;
            }

            Rect(double side) {
                this(side, side);
            }

            double area() { return w * h; }
        }

        public class Shapes {
            public static void main(String[] args) {
                Shape[] shapes = new Shape[3];
                shapes[0] = new Circle(1.0);
                shapes[1] = new Rect(2.0, 3.0);
                shapes[2] = new Rect(2.0);
                double total = 0.0;
                for (Shape s : shapes) {
                    System.out.println(s);
                    total += s.area();
                }
                System.out.println(total);
                System.out.println(shapes[1].getName());
            }
        }
        "#,
        "Shapes",
    );
    assert_eq!(
        out,
        "circle with area 3.14159\nrect with area 6.0\nrect with area 4.0\n13.14159\nrect\n"
    );
}

#[test]
fn super_method_calls_and_overriding() {
    let out = run_stdout(
        r#"
        class Animal {
            String speak() { return "..."; }
            String describe() { return "an animal says " + speak(); }
        }

        class Dog extends Animal {
            String speak() { return "woof"; }
            String describe() {
                return super.describe() + " (a dog: " + super.speak() + "/" + speak() + ")";
            }
        }

        public class Zoo {
            public static void main(String[] args) {
                Animal a = new Dog();
                // Dynamic dispatch inside inherited methods too.
                System.out.println(a.describe());
                Animal plain = new Animal();
                System.out.println(plain.describe());
            }
        }
        "#,
        "Zoo",
    );
    assert_eq!(
        out,
        "an animal says woof (a dog: .../woof)\nan animal says ...\n"
    );
}

#[test]
fn interfaces_and_instanceof_and_casts() {
    let out = run_stdout(
        r#"
        interface Playable {
            String play();
        }

        class Guitar implements Playable {
            public String play() { return "strum"; }
        }

        class Drum implements Playable {
            public String play() { return "boom"; }
            int hits() { return 2; }
        }

        public class Band {
            public static void main(String[] args) {
                Playable[] band = { new Guitar(), new Drum() };
                for (Playable p : band) {
                    System.out.print(p.play() + " ");
                }
                System.out.println();
                for (Playable p : band) {
                    System.out.println(p instanceof Drum);
                    if (p instanceof Drum) {
                        Drum d = (Drum) p;
                        System.out.println("hits: " + d.hits());
                    }
                }
                Playable nothing = null;
                System.out.println(nothing instanceof Guitar);
            }
        }
        "#,
        "Band",
    );
    assert_eq!(out, "strum boom \nfalse\ntrue\nhits: 2\nfalse\n");
}

#[test]
fn class_cast_exception_matches_java() {
    let (result, console) = compile_and_run(
        r"
        class A { }
        class B extends A { }

        public class Bad {
            public static void main(String[] args) {
                A a = new A();
                B b = (B) a;
                System.out.println(b);
            }
        }
        ",
        "Bad",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.ClassCastException: class A cannot be cast to class B"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn inherited_fields_and_super_chaining() {
    let out = run_stdout(
        r#"
        class Base {
            int base = 10;
            static String log = "";

            Base() {
                log += "B";
            }

            Base(int extra) {
                this();
                log += "b" + extra;
            }
        }

        class Derived extends Base {
            int derived = 20;

            Derived() {
                super(5);
                log += "D";
            }

            int total() { return base + derived; }
        }

        public class Chain {
            public static void main(String[] args) {
                Derived d = new Derived();
                System.out.println(Base.log);
                System.out.println(d.total());
                System.out.println(d.base + " " + d.derived);
            }
        }
        "#,
        "Chain",
    );
    assert_eq!(out, "Bb5D\n30\n10 20\n");
}

#[test]
fn stage6_compile_errors_match_javac_wording() {
    let cases: &[(&str, &str)] = &[
        (
            "class A { String f() { return \"a\"; } } class B extends A { int f() { return 1; } }",
            "f() in B cannot override f() in A",
        ),
        (
            "abstract class A { abstract void go(); } class M { static void f() { A a = new A(); } }",
            "A is abstract; cannot be instantiated",
        ),
        (
            "abstract class A { abstract void go(); } class B extends A { }",
            "B is not abstract and does not override abstract method go() in A",
        ),
        (
            "interface S { double area(); } class C implements S { }",
            "C is not abstract and does not override abstract method area() in S",
        ),
        (
            "class A { A(int x) { } } class B extends A { B() { } }",
            "constructor A in class A cannot be applied to given types",
        ),
        (
            "class A { void f() { int x = 1; super(); } }",
            "call to super/this must be the first statement in a constructor",
        ),
        (
            r#"class M { static void f() { System.out.prinn("hello"); } }"#,
            "cannot find symbol: method prinn(String) in class PrintStream",
        ),
        (
            "class M { static void f() { Scanner in = new Scanner(System.in); } }",
            "cannot find symbol: class Scanner",
        ),
        (
            "import java.util.Scanner; class M { static void f() { ArrayList<Integer> a = new ArrayList<>(); } }",
            "cannot find symbol: class ArrayList",
        ),
        (
            "import java.util.NotReal; class M { }",
            "cannot find symbol: class NotReal in package java.util",
        ),
        (
            "import foo.bar.Baz; class M { }",
            "package foo.bar does not exist",
        ),
        (
            "import java.util.Vector; class M { }",
            "java.util.Vector is not supported by caturra (the class library covers the AP CS A subset)",
        ),
        (
            // java.awt / javax.swing (and java.awt.event listeners) are
            // modeled now; an unmodeled package still reports honestly.
            "import java.nio.*; class M { }",
            "package java.nio is not supported by caturra",
        ),
        (
            "class M { static void f() { java.util.NotReal x = null; } }",
            "cannot find symbol: class NotReal in package java.util",
        ),
        (
            "class M { static void f() { foo.bar.Baz x = null; } }",
            "package foo.bar does not exist",
        ),
        (
            "class M { static void f() { java.util.HashMap x = null; } }",
            "java.util.HashMap is not supported by caturra",
        ),
    ];
    for (source, expected) in cases {
        let result = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: "T.java".into(),
            text: (*source).into(),
        }]);
        assert!(!result.success(), "case '{source}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{source}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

// ----- stage 7: the class library -----

#[test]
fn string_methods_match_java() {
    let out = run_stdout(
        r#"
        public class Strings {
            public static void main(String[] args) {
                String s = "Hello, World";
                System.out.println(s.length());
                System.out.println(s.charAt(4));
                System.out.println(s.substring(7));
                System.out.println(s.substring(0, 5));
                System.out.println(s.indexOf("World"));
                System.out.println(s.indexOf("xyz"));
                System.out.println(s.equals("Hello, World"));
                System.out.println(s.equals("hello, world"));
                System.out.println(s.equalsIgnoreCase("HELLO, WORLD"));
                System.out.println("Ab".compareTo("b"));
                System.out.println("apple".compareTo("apple"));
                System.out.println(s.toUpperCase());
                System.out.println(s.toLowerCase());
                System.out.println("  pad  ".trim());
                System.out.println(s.contains("lo, W"));
                System.out.println(s.startsWith("Hell") + " " + s.endsWith("!"));
                System.out.println("".isEmpty());
            }
        }
        "#,
        "Strings",
    );
    assert_eq!(
        out,
        "12\no\nWorld\nHello\n7\n-1\ntrue\nfalse\ntrue\n-33\n0\nHELLO, WORLD\nhello, world\npad\ntrue\ntrue false\ntrue\n"
    );
}

#[test]
fn math_and_wrappers() {
    let out = run_stdout(
        r#"
        public class Maths {
            public static void main(String[] args) {
                System.out.println(Math.abs(-7));
                System.out.println(Math.abs(-2.5));
                System.out.println(Math.pow(2.0, 10.0));
                System.out.println(Math.sqrt(144.0));
                System.out.println(Math.max(3, 9) + " " + Math.min(3, 9));
                System.out.println(Math.max(1.5, 2.5));
                double r = Math.random();
                System.out.println(r >= 0.0 && r < 1.0);
                System.out.println(Integer.MAX_VALUE);
                System.out.println(Integer.MIN_VALUE);
                System.out.println(Integer.parseInt("-42") + 1);
                System.out.println(Double.parseDouble("2.5") * 2);
            }
        }
        "#,
        "Maths",
    );
    assert_eq!(
        out,
        "7\n2.5\n1024.0\n12.0\n9 3\n2.5\ntrue\n2147483647\n-2147483648\n-41\n5.0\n"
    );
}

#[test]
fn scanner_reads_tokens_and_lines() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "In.java".into(),
        text: r#"
        import java.util.Scanner;
        public class In {
            public static void main(String[] args) {
                Scanner in = new Scanner(System.in);
                int n = in.nextInt();
                double d = in.nextDouble();
                String word = in.next();
                in.nextLine();
                String line = in.nextLine();
                System.out.println(n * 2);
                System.out.println(d + 0.5);
                System.out.println(word.toUpperCase());
                System.out.println("[" + line + "]");
                System.out.println(in.hasNextInt());
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(["21 3.5 hey trailing", "a whole line"]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("In", &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "42\n4.0\nHEY\n[a whole line]\nfalse\n"
    );
}

#[test]
fn array_list_basics_and_for_each() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        public class Lists {
            public static void main(String[] args) {
                ArrayList<Integer> nums = new ArrayList<Integer>();
                nums.add(10);
                nums.add(20);
                nums.add(1, 15);
                System.out.println(nums.size());
                System.out.println(nums.get(1));
                nums.set(0, 5);
                System.out.println(nums);
                int removed = nums.remove(2);
                System.out.println(removed + " -> " + nums);
                int total = 0;
                for (int n : nums) total += n;
                System.out.println(total);

                ArrayList<String> names = new ArrayList<>();
                names.add("Ada");
                names.add("Grace");
                System.out.println(names);
                for (String name : names) System.out.println(name.toUpperCase());

                ArrayList<Double> temps = new ArrayList<Double>();
                temps.add(98.6);
                System.out.println(temps.get(0) + 1.0);
                System.out.println(temps.isEmpty());
            }
        }
        "#,
        "Lists",
    );
    assert_eq!(
        out,
        "3\n15\n[5, 15, 20]\n20 -> [5, 15]\n20\n[Ada, Grace]\nADA\nGRACE\n99.6\nfalse\n"
    );
}

#[test]
fn array_list_of_objects() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        class Song {
            private String title;
            Song(String title) { this.title = title; }
            String getTitle() { return title; }
        }

        public class Playlist {
            public static void main(String[] args) {
                ArrayList<Song> queue = new ArrayList<Song>();
                queue.add(new Song("Daisy"));
                queue.add(new Song("Bicycle"));
                for (Song s : queue) {
                    System.out.println(s.getTitle());
                }
                Song first = queue.get(0);
                System.out.println(first.getTitle().length());
            }
        }
        "#,
        "Playlist",
    );
    assert_eq!(out, "Daisy\nBicycle\n5\n");
}

#[test]
fn classlib_exceptions_match_java_wording() {
    let cases: &[(&str, &str)] = &[
        (
            r#"String s = "abc"; System.out.println(s.charAt(5));"#,
            "java.lang.StringIndexOutOfBoundsException: String index out of range: 5",
        ),
        (
            r#"String s = "abc"; System.out.println(s.substring(1, 9));"#,
            "java.lang.StringIndexOutOfBoundsException: begin 1, end 9, length 3",
        ),
        (
            r#"System.out.println(Integer.parseInt("x1"));"#,
            "java.lang.NumberFormatException: For input string: \"x1\"",
        ),
    ];
    for (body, expected) in cases {
        let (result, console) = compile_and_run(
            &format!("public class T {{ public static void main(String[] args) {{ {body} }} }}"),
            "T",
        );
        assert!(
            matches!(result, Err(VmError::UncaughtException(_))),
            "case '{body}': {result:?}"
        );
        assert!(
            console.stderr_text().contains(expected),
            "case '{body}': expected '{expected}' in {}",
            console.stderr_text()
        );
    }

    // ArrayList index errors.
    let (result, console) = compile_and_run(
        r"
        import java.util.ArrayList;
        public class L {
            public static void main(String[] args) {
                ArrayList<Integer> list = new ArrayList<Integer>();
                list.add(7);
                System.out.println(list.get(3));
            }
        }
        ",
        "L",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.lang.IndexOutOfBoundsException: Index 3 out of bounds for length 1"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn math_random_is_deterministic_per_seed() {
    let source = r"
        public class R {
            public static void main(String[] args) {
                System.out.println(Math.random());
                System.out.println((int) (Math.random() * 6) + 1);
            }
        }
    ";
    let run = |seed: Option<u64>| {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: "R.java".into(),
            text: source.into(),
        }]);
        assert!(compilation.success());
        let mut vfs = VirtualFileSystem::new();
        let mut console = BufferedConsole::new();
        let mut vm = Vm::new(
            VmOptions {
                random_seed: seed,
                ..VmOptions::default()
            },
            &mut vfs,
            &mut console,
        );
        for class in compilation.classes {
            vm.load_class(class.class_file).unwrap();
        }
        vm.run_main("R", &[]).unwrap();
        console.stdout_text()
    };
    // Same seed → same output; different seed → different draw.
    assert_eq!(run(Some(7)), run(Some(7)));
    assert_ne!(run(Some(7)), run(Some(8)));
    // All draws in range.
    for line in run(Some(99)).lines() {
        let value: f64 = line.parse().unwrap();
        assert!((0.0..7.0).contains(&value), "{value}");
    }
}

// ----- stage 8: file IO over the virtual filesystem -----

#[test]
fn print_writer_writes_and_scanner_reads_back() {
    let out = run_stdout(
        r#"
        import java.io.File;
        import java.io.PrintWriter;
        import java.util.Scanner;

        public class RoundTrip {
            public static void main(String[] args) throws Exception {
                PrintWriter out = new PrintWriter("scores.txt");
                out.println("Ada 95");
                out.println("Alan 88");
                out.print(2);
                out.close();

                File f = new File("scores.txt");
                System.out.println(f.exists() + " " + f.getName());

                Scanner in = new Scanner(f);
                while (in.hasNextLine()) {
                    System.out.println("> " + in.nextLine());
                }
            }
        }
        "#,
        "RoundTrip",
    );
    assert_eq!(out, "true scores.txt\n> Ada 95\n> Alan 88\n> 2\n");
}

/// `PrintWriter.write`/`append`/`format`: `write(String)` writes the text and
/// `write(int)` a single character; `append` writes a char or text and returns
/// the writer for chaining; `format` is `printf` that returns the writer.
/// Pinned JDK-free; a live-JDK match is in `diff_print_writer_write_append`.
#[test]
fn print_writer_write_append_format() {
    let out = run_stdout(
        r#"
        import java.io.File;
        import java.io.PrintWriter;
        import java.util.Scanner;
        public class W {
            public static void main(String[] args) throws Exception {
                PrintWriter out = new PrintWriter("out.txt");
                out.write("hello");
                out.write(32);
                out.write("world");
                out.println();
                out.append('A').append("BC").append('!');
                out.println();
                out.format("%d-%s", 42, "x").println();
                out.printf("%.1f%n", 3.14);
                out.close();

                Scanner in = new Scanner(new File("out.txt"));
                while (in.hasNextLine()) { System.out.println("> " + in.nextLine()); }
            }
        }
        "#,
        "W",
    );
    assert_eq!(out, "> hello world\n> ABC!\n> 42-x\n> 3.1\n");
}

#[test]
fn scanner_file_tokenizing_and_file_ops() {
    let out = run_stdout(
        r#"
        import java.io.File;
        import java.io.PrintWriter;
        import java.util.Scanner;

        public class Grades {
            public static void main(String[] args) throws Exception {
                PrintWriter w = new PrintWriter(new File("nums.txt"));
                w.println("10 20 30");
                w.println("2.5");
                w.close();

                Scanner in = new Scanner(new File("nums.txt"));
                int total = 0;
                for (int i = 0; i < 3; i++) total += in.nextInt();
                double scale = in.nextDouble();
                System.out.println(total * scale);

                File f = new File("nums.txt");
                System.out.println(f.length() > 0);
                System.out.println(f.isFile() + " " + f.isDirectory());
                System.out.println(f.delete());
                System.out.println(f.exists());
                System.out.println(f.delete());

                File dir = new File("logs");
                System.out.println(dir.mkdir());
                System.out.println(dir.isDirectory());
                File fresh = new File("logs/new.txt");
                System.out.println(fresh.createNewFile() + " " + fresh.length());
            }
        }
        "#,
        "Grades",
    );
    assert_eq!(
        out,
        "150.0\ntrue\ntrue false\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue 0\n"
    );
}

#[test]
fn missing_file_throws_like_java() {
    let (result, console) = compile_and_run(
        r#"
        import java.io.File;
        import java.util.Scanner;

        public class Missing {
            public static void main(String[] args) throws Exception {
                Scanner in = new Scanner(new File("ghost.txt"));
                System.out.println(in.nextLine());
            }
        }
        "#,
        "Missing",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    assert!(
        console
            .stderr_text()
            .contains("java.io.FileNotFoundException: ghost.txt (No such file or directory)"),
        "{}",
        console.stderr_text()
    );
}

#[test]
fn vfs_seeded_files_are_visible_to_java() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "Reader.java".into(),
        text: r#"
        import java.io.File;
        import java.util.Scanner;

        public class Reader {
            public static void main(String[] args) throws Exception {
                Scanner in = new Scanner(new File("/data/input.txt"));
                int total = 0;
                while (in.hasNextInt()) total += in.nextInt();
                System.out.println(total);
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);

    let mut vfs = VirtualFileSystem::new();
    vfs.write_file("/data/input.txt", "5 10 15 20".as_bytes().to_vec())
        .unwrap();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("Reader", &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "50\n");
    // And output written by Java is visible to the host afterwards.
    assert!(vfs.exists("/data/input.txt"));
}

// ----- explicit-frame interpreter -----

#[test]
fn deep_recursion_works_like_java() {
    // Depth ~3000 — far past what host-stack recursion survived
    // (the old engine was capped at 256 frames); real Java handles it.
    let out = run_stdout(
        r"
        public class DeepSum {
            static int sum(int n) {
                if (n == 0) {
                    return 0;
                }
                return n + sum(n - 1);
            }

            public static void main(String[] args) {
                System.out.println(sum(3000));
            }
        }
        ",
        "DeepSum",
    );
    assert_eq!(out, "4501500\n");
}

#[test]
fn mutual_recursion_at_depth() {
    let out = run_stdout(
        r"
        public class PingPong {
            static int ping(int n) {
                if (n == 0) {
                    return 0;
                }
                return 1 + pong(n - 1);
            }

            static int pong(int n) {
                if (n == 0) {
                    return 0;
                }
                return 1 + ping(n - 1);
            }

            public static void main(String[] args) {
                System.out.println(ping(2500));
            }
        }
        ",
        "PingPong",
    );
    assert_eq!(out, "2500\n");
}

#[test]
fn uncaught_exceptions_carry_a_stack_trace() {
    let (result, console) = compile_and_run(
        r"
        public class Trace {
            static int explode() {
                return 1 / 0;
            }

            static int middle() {
                return explode();
            }

            public static void main(String[] args) {
                System.out.println(middle());
            }
        }
        ",
        "Trace",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    let stderr = console.stderr_text();
    assert!(
        stderr.contains("java.lang.ArithmeticException: / by zero"),
        "{stderr}"
    );
    // Innermost first with source line numbers, like real java.
    let explode_at = stderr.find("\tat Trace.explode(Trace.java:4)").unwrap();
    let middle_at = stderr.find("\tat Trace.middle(Trace.java:8)").unwrap();
    let main_at = stderr.find("\tat Trace.main(Trace.java:12)").unwrap();
    assert!(explode_at < middle_at && middle_at < main_at, "{stderr}");
}

#[test]
fn clinit_chain_runs_ancestors_first_via_frames() {
    // Touching C's statics initializes A, then B, then C — with each
    // <clinit> running as a pseudo-caller frame, deep static-time
    // recursion included.
    let out = run_stdout(
        r#"
        class A {
            static String log = start();
            static String start() { return "A"; }
        }

        class B extends A {
            static String tag = A.log + "B";
        }

        class C extends B {
            static String tag2 = B.tag + "C" + depth(200);
            static int depth(int n) {
                if (n == 0) {
                    return 0;
                }
                return depth(n - 1);
            }
        }

        public class InitChain {
            public static void main(String[] args) {
                System.out.println(C.tag2);
                System.out.println(A.log);
            }
        }
        "#,
        "InitChain",
    );
    assert_eq!(out, "ABC0\nA\n");
}

// ----- the debugger -----

/// A scripted debug host: follows a fixed list of commands and records
/// every pause's snapshot for assertions.
struct ScriptedHost {
    script: Vec<DebugCommand>,
    at: usize,
    pauses: Vec<DebugSnapshot>,
}

impl ScriptedHost {
    fn new(script: Vec<DebugCommand>) -> Self {
        Self {
            script,
            at: 0,
            pauses: Vec::new(),
        }
    }
}

impl DebugHost for ScriptedHost {
    fn on_pause(&mut self, snapshot: &DebugSnapshot, _: &mut dyn WatchEvaluator) -> DebugControl {
        self.pauses.push(snapshot.clone());
        let command = self
            .script
            .get(self.at)
            .copied()
            .unwrap_or(DebugCommand::Continue);
        self.at += 1;
        DebugControl {
            command,
            breakpoints: None,
        }
    }
}

fn debug_run(
    source: &str,
    class: &str,
    breakpoints: &[(&str, u32)],
    script: Vec<DebugCommand>,
) -> (ScriptedHost, String, Result<ExitStatus, VmError>) {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{class}.java"),
        text: source.into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for compiled in compilation.classes {
        vm.load_class(compiled.class_file).unwrap();
    }
    let breakpoints: Vec<Breakpoint> = breakpoints
        .iter()
        .map(|(file, line)| Breakpoint {
            file: (*file).to_owned(),
            line: *line,
        })
        .collect();
    let mut host = ScriptedHost::new(script);
    let result = vm.run_main_debug(class, &[], &breakpoints, &mut host);
    (host, console.stdout_text(), result)
}

const DEBUG_PROGRAM: &str = r"public class Dbg {
    static int square(int x) {
        int result = x * x;
        return result;
    }

    public static void main(String[] args) {
        int total = 0;
        for (int i = 1; i <= 3; i++) {
            total += square(i);
        }
        System.out.println(total);
    }
}
";

#[test]
fn breakpoint_pauses_each_arrival_with_named_locals() {
    // Line 10 is `total += square(i);` — inside the loop, 3 arrivals.
    let (host, stdout, result) = debug_run(
        DEBUG_PROGRAM,
        "Dbg",
        &[("Dbg.java", 10)],
        vec![
            DebugCommand::Continue,
            DebugCommand::Continue,
            DebugCommand::Continue,
        ],
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(stdout, "14\n");
    assert_eq!(host.pauses.len(), 3);
    for (index, pause) in host.pauses.iter().enumerate() {
        assert_eq!(pause.reason, PauseReason::Breakpoint);
        let top = &pause.frames[0];
        assert_eq!(top.class_name, "Dbg");
        assert_eq!(top.method_name, "main");
        assert_eq!(top.source_file, "Dbg.java");
        assert_eq!(top.line, Some(10));
        // Named locals with values at the pause point.
        let get = |name: &str| {
            top.locals.iter().find(|l| l.name == name).map_or_else(
                || panic!("missing local {name}: {:?}", top.locals),
                |l| l.value.clone(),
            )
        };
        let i = i32::try_from(index).expect("small index") + 1;
        assert_eq!(get("i"), i.to_string());
        let expected_total: i32 = (1..i).map(|n| n * n).sum();
        assert_eq!(get("total"), expected_total.to_string());
        assert!(get("args").starts_with('['), "{:?}", top.locals);
    }
}

#[test]
fn step_into_descends_and_step_out_returns() {
    let (host, stdout, result) = debug_run(
        DEBUG_PROGRAM,
        "Dbg",
        &[("Dbg.java", 10)],
        vec![
            DebugCommand::StepInto, // from line 10 into square's line 3
            DebugCommand::StepOut,  // back to main
            DebugCommand::Continue, // skip the 2nd breakpoint arrival
            DebugCommand::Continue, // skip the 3rd
        ],
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(stdout, "14\n");

    // Pause 2: inside square(), with the call stack showing main below.
    let inside = &host.pauses[1];
    assert_eq!(inside.reason, PauseReason::Step);
    assert_eq!(inside.frames[0].method_name, "square");
    assert_eq!(inside.frames[0].line, Some(3));
    assert_eq!(inside.frames.len(), 2);
    assert_eq!(inside.frames[1].method_name, "main");
    let x = inside.frames[0]
        .locals
        .iter()
        .find(|l| l.name == "x")
        .map(|l| l.value.clone());
    assert_eq!(x.as_deref(), Some("1"));

    // Pause 3: stepped out, back in main.
    let out = &host.pauses[2];
    assert_eq!(out.frames[0].method_name, "main");
    assert_eq!(out.frames.len(), 1);
}

#[test]
fn step_over_stays_in_the_frame() {
    let (host, _, result) = debug_run(
        DEBUG_PROGRAM,
        "Dbg",
        &[("Dbg.java", 8)], // int total = 0;
        vec![
            DebugCommand::StepOver, // to line 9 (for)
            DebugCommand::StepOver, // to line 10 — over the call, not into it
            DebugCommand::StepOver, // loop update / next line, still in main
            DebugCommand::Continue,
        ],
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert!(host.pauses.len() >= 4, "{}", host.pauses.len());
    for pause in &host.pauses[..4] {
        assert_eq!(pause.frames[0].method_name, "main", "{pause:?}");
        assert_eq!(pause.frames.len(), 1);
    }
    assert_eq!(host.pauses[1].frames[0].line, Some(9));
    assert_eq!(host.pauses[2].frames[0].line, Some(10));
}

#[test]
fn terminate_stops_the_program() {
    let (host, stdout, result) = debug_run(
        DEBUG_PROGRAM,
        "Dbg",
        &[("Dbg.java", 10)],
        vec![DebugCommand::Terminate],
    );
    assert!(matches!(result, Err(VmError::Stopped)), "{result:?}");
    assert_eq!(stdout, "");
    assert_eq!(host.pauses.len(), 1);
}

#[test]
fn interrupt_pauses_a_running_loop() {
    struct InterruptingHost {
        asked: bool,
        paused: bool,
    }
    impl DebugHost for InterruptingHost {
        fn on_pause(
            &mut self,
            snapshot: &DebugSnapshot,
            _: &mut dyn WatchEvaluator,
        ) -> DebugControl {
            assert_eq!(snapshot.reason, PauseReason::Interrupt);
            self.paused = true;
            DebugControl {
                command: DebugCommand::Terminate,
                breakpoints: None,
            }
        }
        fn interrupt_requested(&mut self) -> bool {
            self.asked = true;
            true
        }
    }

    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "Spin.java".into(),
        text: r"
        public class Spin {
            public static void main(String[] args) {
                while (true) {
                    int x = 1;
                }
            }
        }
        "
        .into(),
    }]);
    assert!(compilation.success());
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for compiled in compilation.classes {
        vm.load_class(compiled.class_file).unwrap();
    }
    let mut host = InterruptingHost {
        asked: false,
        paused: false,
    };
    let result = vm.run_main_debug("Spin", &[], &[], &mut host);
    assert!(matches!(result, Err(VmError::Stopped)), "{result:?}");
    assert!(host.asked && host.paused);
}

#[test]
fn imports_enable_library_classes() {
    // Wildcards work, java.lang imports are legal-and-redundant, and
    // exception imports (used only in throws clauses) are accepted.
    let out = run_stdout(
        r#"
        import java.lang.String;
        import java.util.*;
        import java.io.*;

        public class Wild {
            public static void main(String[] args) throws IOException {
                ArrayList<String> list = new ArrayList<>();
                list.add("wildcard");
                PrintWriter out = new PrintWriter("w.txt");
                out.println(list.get(0));
                out.close();
                Scanner in = new Scanner(new File("w.txt"));
                System.out.println(in.nextLine());
            }
        }
        "#,
        "Wild",
    );
    assert_eq!(out, "wildcard\n");
}

#[test]
fn user_class_named_like_library_needs_no_import() {
    let out = run_stdout(
        r#"
        class Scanner {
            String kind() { return "homemade"; }
        }

        public class Shadow {
            public static void main(String[] args) {
                Scanner s = new Scanner();
                System.out.println(s.kind());
            }
        }
        "#,
        "Shadow",
    );
    assert_eq!(out, "homemade\n");
}

#[test]
fn fully_qualified_names_need_no_import() {
    let out = run_stdout(
        r#"
        public class Qualified {
            public static void main(String[] args) throws java.io.IOException {
                java.util.ArrayList<java.lang.Integer> nums =
                    new java.util.ArrayList<Integer>();
                nums.add(java.lang.Integer.parseInt("20"));
                nums.add(java.lang.Math.max(1, 2));
                java.io.PrintWriter out = new java.io.PrintWriter("q.txt");
                for (int n : nums) out.println(n);
                out.close();
                java.util.Scanner in = new java.util.Scanner(new java.io.File("q.txt"));
                int total = 0;
                while (in.hasNextInt()) total += in.nextInt();
                java.lang.System.out.println(total + java.lang.Integer.MAX_VALUE % 10);
                java.lang.String label = "ok";
                System.out.println(label.toUpperCase());
            }
        }
        "#,
        "Qualified",
    );
    assert_eq!(out, "29\nOK\n");
}

#[test]
fn qualified_scanner_reads_stdin() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "QIn.java".into(),
        text: r"
        public class QIn {
            public static void main(String[] args) {
                java.util.Scanner in = new java.util.Scanner(java.lang.System.in);
                System.out.println(in.nextInt() * 2);
            }
        }
        "
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(["21"]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    vm.run_main("QIn", &[]).unwrap();
    assert_eq!(console.stdout_text(), "42\n");
}

#[test]
fn local_variable_shadows_the_java_package() {
    // Java's obscuring rules: a variable named `java` wins over the
    // package, so `java.x` is a field access, not a qualified name.
    let out = run_stdout(
        r"
        class Holder {
            int x = 7;
        }

        public class Obscured {
            public static void main(String[] args) {
                Holder java = new Holder();
                System.out.println(java.x);
            }
        }
        ",
        "Obscured",
    );
    assert_eq!(out, "7\n");
}

// ----- watch expressions -----

/// Mimics the wasm boundary's watch orchestration: synthesize a class
/// whose static method's parameters mirror the paused frame's locals,
/// compile it with the program sources, and evaluate in the VM.
fn eval_watch(
    expression: &str,
    program: &str,
    snapshot: &DebugSnapshot,
    watch: &mut dyn WatchEvaluator,
) -> Result<String, String> {
    let frame = snapshot.frames.first().ok_or("no frame")?;
    let usable: Vec<_> = frame
        .locals
        .iter()
        .filter(|l| l.name != "this" && !l.type_name.is_empty())
        .collect();
    let params = usable
        .iter()
        .map(|l| format!("{} {}", l.type_name, l.name))
        .collect::<Vec<_>>()
        .join(", ");
    let names: Vec<String> = usable.iter().map(|l| l.name.clone()).collect();
    let source = format!(
        "class __CaturraWatch {{ static String __eval({params}) {{ return \"\" + ({expression}); }} }}"
    );
    let compilation = caturra_compiler::compile(&[
        caturra_compiler::SourceFile {
            // The wasm boundary passes the program's real source files, so a
            // public class sits in a file of its own name (JLS §7.6).
            path: format!("{}.java", frame.class_name),
            text: program.into(),
        },
        caturra_compiler::SourceFile {
            path: "__CaturraWatch.java".into(),
            text: source,
        },
    ]);
    if !compilation.success() {
        return Err(compilation
            .diagnostics
            .iter()
            .find(|d| d.path == "__CaturraWatch.java")
            .map_or_else(
                || String::from("watch does not compile"),
                |d| d.message.clone(),
            ));
    }
    let class = compilation
        .classes
        .iter()
        .find(|c| c.binary_name == "__CaturraWatch")
        .ok_or("watch class missing")?;
    watch.evaluate(&class.class_file, "__eval", &names)
}

#[test]
fn watch_expressions_see_live_state() {
    const PROGRAM: &str = r"
import java.util.ArrayList;

public class WatchMe {
    static int scale(int n) { return n * 10; }

    public static void main(String[] args) {
        ArrayList<Integer> nums = new ArrayList<>();
        int total = 0;
        for (int i = 1; i <= 3; i++) {
            total += i;
            nums.add(i);
        }
        System.out.println(total);
    }
}
";

    struct WatchingHost {
        observations: Vec<Vec<Result<String, String>>>,
    }
    impl DebugHost for WatchingHost {
        fn on_pause(
            &mut self,
            snapshot: &DebugSnapshot,
            watch: &mut dyn WatchEvaluator,
        ) -> DebugControl {
            let round = [
                "total * 2 + i",
                "nums",
                "nums.size()",
                "WatchMe.scale(i)",
                "totall + 1",
            ]
            .iter()
            .map(|expr| eval_watch(expr, PROGRAM, snapshot, watch))
            .collect();
            self.observations.push(round);
            DebugControl {
                command: DebugCommand::Continue,
                breakpoints: None,
            }
        }
    }

    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "WatchMe.java".into(),
        text: PROGRAM.into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let mut host = WatchingHost {
        observations: Vec::new(),
    };
    // Line 11: `total += i;` inside the loop.
    let breakpoints = [Breakpoint {
        file: "WatchMe.java".into(),
        line: 11,
    }];
    let result = vm.run_main_debug("WatchMe", &[], &breakpoints, &mut host);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "6\n");
    assert_eq!(host.observations.len(), 3);

    // Iteration 2 (i=2, total=1 before +=, nums=[1]).
    let round = &host.observations[1];
    assert_eq!(round[0].as_deref(), Ok("4")); // 1*2 + 2
    assert_eq!(round[1].as_deref(), Ok("[1]"));
    assert_eq!(round[2].as_deref(), Ok("1"));
    assert_eq!(round[3].as_deref(), Ok("20")); // scale(2)
    // The bad watch reports the compiler's javac-style message.
    let error = round[4].as_ref().unwrap_err();
    assert!(error.contains("cannot find variable 'totall'"), "{error}");

    // Values advance between pauses: iteration 3 sees updated state.
    assert_eq!(host.observations[2][1].as_deref(), Ok("[1, 2]"));

    // The program's own output was not polluted by watch evaluation.
    assert_eq!(console.stdout_text(), "6\n");
}

#[test]
fn paused_view_expands_instance_fields() {
    struct FieldHost {
        seen: Vec<String>,
    }
    impl DebugHost for FieldHost {
        fn on_pause(
            &mut self,
            snapshot: &DebugSnapshot,
            _: &mut dyn WatchEvaluator,
        ) -> DebugControl {
            for local in &snapshot.frames[0].locals {
                self.seen.push(format!("{} = {}", local.name, local.value));
            }
            DebugControl {
                command: DebugCommand::Continue,
                breakpoints: None,
            }
        }
    }

    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "Pets.java".into(),
        text: r#"
        class Dog {
            String name;
            int age;
            Dog buddy;
            Dog(String name, int age) { this.name = name; this.age = age; }
        }

        public class Pets {
            public static void main(String[] args) {
                Dog rex = new Dog("Rex", 3);
                Dog pal = new Dog("Pal", 5);
                rex.buddy = pal;
                System.out.println(rex.age + pal.age);
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let mut host = FieldHost { seen: Vec::new() };
    let breakpoints = [Breakpoint {
        file: "Pets.java".into(),
        line: 14, // System.out.println(...)
    }];
    vm.run_main_debug("Pets", &[], &breakpoints, &mut host)
        .unwrap();

    // One level deep, fields sorted, nested instance shallow.
    let rex = host.seen.iter().find(|s| s.starts_with("rex = ")).unwrap();
    assert!(
        rex.contains("Dog@")
            && rex.contains("age=3")
            && rex.contains("buddy=Dog@")
            && rex.contains("name=\"Rex\""),
        "{rex}"
    );
    // The nested buddy is NOT expanded (no inner braces after buddy=Dog@N).
    let after_buddy = rex.split("buddy=").nth(1).unwrap();
    assert!(
        !after_buddy.split(',').next().unwrap().contains('{'),
        "{rex}"
    );
}

// ----- try / catch / throw -----

#[test]
fn try_catch_basics() {
    let out = run_stdout(
        r#"
        public class Catcher {
            public static void main(String[] args) {
                try {
                    int[] a = new int[2];
                    a[5] = 1;
                    System.out.println("unreached");
                } catch (ArithmeticException e) {
                    System.out.println("wrong handler");
                } catch (ArrayIndexOutOfBoundsException e) {
                    System.out.println("caught: " + e.getMessage());
                }
                try {
                    int x = 1 / 0;
                } catch (RuntimeException e) {
                    // Subtype catch: ArithmeticException < RuntimeException.
                    System.out.println(e);
                }
                System.out.println("after");
            }
        }
        "#,
        "Catcher",
    );
    assert_eq!(
        out,
        "caught: Index 5 out of bounds for length 2\n\
         java.lang.ArithmeticException: / by zero\nafter\n"
    );
}

#[test]
fn exceptions_unwind_across_frames() {
    let out = run_stdout(
        r#"
        public class Unwind {
            static int depth3() { return 10 / 0; }
            static int depth2() { return depth3(); }
            static int depth1() { return depth2(); }

            public static void main(String[] args) {
                try {
                    System.out.println(depth1());
                } catch (ArithmeticException e) {
                    System.out.println("unwound: " + e.getMessage());
                }
                System.out.println("still running");
            }
        }
        "#,
        "Unwind",
    );
    assert_eq!(out, "unwound: / by zero\nstill running\n");
}

#[test]
fn throw_and_rethrow() {
    let out = run_stdout(
        r#"
        public class Thrower {
            static void validate(int age) {
                if (age < 0) {
                    throw new IllegalArgumentException("age " + age + " is negative");
                }
            }

            public static void main(String[] args) {
                try {
                    validate(-3);
                } catch (IllegalArgumentException e) {
                    System.out.println("rejected: " + e.getMessage());
                }
                try {
                    try {
                        throw new RuntimeException("inner");
                    } catch (RuntimeException e) {
                        // Rethrow to the outer handler.
                        throw e;
                    }
                } catch (RuntimeException e) {
                    System.out.println("outer got " + e.getMessage());
                }
                try {
                    throw new IllegalStateException();
                } catch (IllegalStateException e) {
                    System.out.println("message: " + e.getMessage());
                    System.out.println(e);
                }
            }
        }
        "#,
        "Thrower",
    );
    assert_eq!(
        out,
        "rejected: age -3 is negative\nouter got inner\nmessage: null\n\
         java.lang.IllegalStateException\n"
    );
}

#[test]
fn uncaught_exceptions_still_terminate() {
    let (result, console) = compile_and_run(
        r#"
        public class Escapes {
            public static void main(String[] args) {
                try {
                    int x = 1 / 0;
                } catch (NullPointerException e) {
                    System.out.println("wrong type");
                }
            }
        }
        "#,
        "Escapes",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    let stderr = console.stderr_text();
    assert!(
        stderr.contains("java.lang.ArithmeticException: / by zero"),
        "{stderr}"
    );
    assert!(
        stderr.contains("\tat Escapes.main(Escapes.java:5)"),
        "{stderr}"
    );
}

#[test]
fn catch_stack_overflow_error() {
    let out = run_stdout(
        r#"
        public class DeepCatch {
            static int down(int n) { return down(n + 1); }

            public static void main(String[] args) {
                try {
                    System.out.println(down(0));
                } catch (Error e) {
                    System.out.println("caught " + e);
                }
                System.out.println("recovered");
            }
        }
        "#,
        "DeepCatch",
    );
    assert_eq!(out, "caught java.lang.StackOverflowError\nrecovered\n");
}

#[test]
fn try_catch_compile_errors_match_javac() {
    let cases: &[(&str, &str)] = &[
        (
            "class M { static void f() { try { int x = 1/0; } catch (Exception e) { } catch (ArithmeticException e) { } } }",
            "exception ArithmeticException has already been caught",
        ),
        (
            "class M { static void f() { try { int x = 1/0; } catch (NotReal e) { } } }",
            "cannot find symbol: class NotReal",
        ),
        (
            "class M { static void f() { throw new M(); } }",
            "cannot be converted to Throwable",
        ),
        (
            r#"class M { static void f() { throw "text"; } }"#,
            "incompatible types: String cannot be converted to Throwable",
        ),
        (
            "class M { static void f() { try { int x = 1; } } }",
            "'try' needs at least one 'catch' clause or a 'finally' block",
        ),
        (
            r#"class M { static void f() { boolean b = "ab".matches("a."); } }"#,
            "String.matches exists in Java, but regular expressions are not supported by caturra",
        ),
        (
            r#"class M { static void f() { String s = "a".replaceAll("a", "b"); } }"#,
            "String.replaceAll exists in Java, but regular expressions are not supported",
        ),
        (
            r#"class M { static void f() { String s = String.join(",", "a", "b"); } }"#,
            "String.join exists in Java, but varargs are not supported by caturra",
        ),
        (
            r#"class M { static void f() { "abc".lines(); } }"#,
            "String.lines exists in Java, but streams are not supported by caturra",
        ),
        (
            "import java.util.ArrayList; class M { static void f() { ArrayList<Integer> a = new ArrayList<>(); a.iterator(); } }",
            "ArrayList.iterator exists in Java, but iterators are not supported by caturra (use for-each or an index loop)",
        ),
        (
            "import java.util.Scanner; class M { static void f() { Scanner s = new Scanner(System.in); s.nextBigInteger(); } }",
            "Scanner.nextBigInteger exists in Java, but BigInteger is not supported by caturra",
        ),
    ];
    for (source, expected) in cases {
        let result = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: "T.java".into(),
            text: (*source).into(),
        }]);
        assert!(!result.success(), "case '{source}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{source}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn labeled_break_continue_errors_match_javac() {
    let cases: &[(&str, &str)] = &[
        (
            "class M { static void f() { while (true) { break nope; } } }",
            "undefined label: nope",
        ),
        (
            "class M { static void f() { lbl: { continue lbl; } } }",
            "not a loop label: lbl",
        ),
        (
            "class M { static void f() { continue; } }",
            "'continue' can only be used inside a loop",
        ),
    ];
    for (source, expected) in cases {
        let result = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: "T.java".into(),
            text: (*source).into(),
        }]);
        assert!(!result.success(), "case '{source}' compiled unexpectedly");
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains(expected)),
            "case '{source}': expected '{expected}' in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn missing_return_accounts_for_try_catch() {
    // Both paths return: no missing-return error.
    let out = run_stdout(
        r#"
        public class Paths {
            static String attempt(int n) {
                try {
                    return "ok " + (10 / n);
                } catch (ArithmeticException e) {
                    return "fail";
                }
            }

            public static void main(String[] args) {
                System.out.println(attempt(5));
                System.out.println(attempt(0));
            }
        }
        "#,
        "Paths",
    );
    assert_eq!(out, "ok 2\nfail\n");
}

#[test]
fn finally_runs_on_every_path() {
    let out = run_stdout(
        r#"
        public class Fin {
            static String log = "";

            static int normal() {
                try {
                    log += "t";
                    return 1;
                } finally {
                    log += "F";
                }
            }

            static int caught() {
                try {
                    return 10 / 0;
                } catch (ArithmeticException e) {
                    log += "c";
                    return 2;
                } finally {
                    log += "F";
                }
            }

            static int uncaughtThrough() {
                try {
                    return 10 / 0;
                } finally {
                    log += "F";
                }
            }

            public static void main(String[] args) {
                // Separate statements: `log += normal()` would read log
                // BEFORE the call and wipe its side effects (Java
                // compound-assignment semantics).
                int a = normal();
                log += a;
                int b = caught();
                log += b;
                try {
                    int c = uncaughtThrough();
                    log += c;
                } catch (ArithmeticException e) {
                    log += "outer";
                }
                // break and continue through finally.
                for (int i = 0; i < 3; i++) {
                    try {
                        if (i == 1) {
                            continue;
                        }
                        if (i == 2) {
                            break;
                        }
                        log += i;
                    } finally {
                        log += "f";
                    }
                }
                System.out.println(log);
            }
        }
        "#,
        "Fin",
    );
    assert_eq!(out, "tF1cF2Fouter0fff\n");
}

#[test]
fn try_finally_without_catch() {
    let out = run_stdout(
        r#"
        public class NoCatch {
            public static void main(String[] args) {
                try {
                    System.out.println("work");
                } finally {
                    System.out.println("cleanup");
                }
                System.out.println("done");
            }
        }
        "#,
        "NoCatch",
    );
    assert_eq!(out, "work\ncleanup\ndone\n");
}

#[test]
fn user_defined_exceptions() {
    let out = run_stdout(
        r#"
        class TooSmallException extends Exception {
            private int actual;

            TooSmallException(String message, int actual) {
                super(message);
                this.actual = actual;
            }

            int getActual() { return actual; }
        }

        class EmptyException extends RuntimeException {
        }

        public class Users {
            static void check(int n) throws TooSmallException {
                if (n < 10) {
                    throw new TooSmallException("need at least 10, got " + n, n);
                }
                System.out.println(n + " ok");
            }

            public static void main(String[] args) {
                try {
                    check(25);
                    check(3);
                } catch (TooSmallException e) {
                    // Inherited getMessage + own field + own method.
                    System.out.println("caught: " + e.getMessage());
                    System.out.println("actual was " + e.getActual());
                    System.out.println(e);
                }

                // Subtype catch: a user exception under its library parent.
                try {
                    throw new TooSmallException("subtype", 1);
                } catch (Exception e) {
                    System.out.println("as Exception: " + e.getMessage());
                }

                // No-message user runtime exception via catch-all parent.
                try {
                    throw new EmptyException();
                } catch (RuntimeException e) {
                    System.out.println("message: " + e.getMessage());
                    System.out.println(e);
                }
            }
        }
        "#,
        "Users",
    );
    assert_eq!(
        out,
        "25 ok\ncaught: need at least 10, got 3\nactual was 3\n\
         TooSmallException: need at least 10, got 3\n\
         as Exception: subtype\nmessage: null\nEmptyException\n"
    );
}

#[test]
fn user_exception_uncaught_and_hierarchy_errors() {
    // Uncaught user exception: java-style stderr.
    let (result, console) = compile_and_run(
        r#"
        class BoomException extends RuntimeException {
            BoomException(String m) { super(m); }
        }

        public class Boom {
            public static void main(String[] args) {
                throw new BoomException("kaboom");
            }
        }
        "#,
        "Boom",
    );
    assert!(
        matches!(result, Err(VmError::UncaughtException(_))),
        "{result:?}"
    );
    let stderr = console.stderr_text();
    assert!(stderr.contains("BoomException: kaboom"), "{stderr}");
    assert!(stderr.contains("\tat Boom.main(Boom.java"), "{stderr}");

    // catch of a non-throwable user class: javac wording.
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "T.java".into(),
        text:
            "class Pet { } class M { static void f() { try { int x = 1/0; } catch (Pet p) { } } }"
                .into(),
    }]);
    assert!(!compilation.success());
    assert!(
        compilation.diagnostics.iter().any(|d| d
            .message
            .contains("incompatible types: Pet cannot be converted to Throwable")),
        "{:?}",
        compilation.diagnostics
    );

    // Masking a user exception with its library parent first.
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "T.java".into(),
        text: "class E extends Exception { } class M { static void f() { try { int x = 1/0; } catch (Exception a) { } catch (E b) { } } }".into(),
    }]);
    assert!(!compilation.success());
    assert!(
        compilation
            .diagnostics
            .iter()
            .any(|d| d.message.contains("exception E has already been caught")),
        "{:?}",
        compilation.diagnostics
    );
}

#[test]
fn transcendental_functions_behave_sanely() {
    // Property checks (irrational results can differ by 1 ulp from
    // Java between libm implementations, so the differential suite
    // sticks to exact inputs and these guard the rest).
    let out = run_stdout(
        r"
        public class Trig {
            public static void main(String[] args) {
                double x = 0.7;
                System.out.println(Math.abs(Math.sin(x) * Math.sin(x)
                    + Math.cos(x) * Math.cos(x) - 1.0) < 1e-15);
                System.out.println(Math.abs(Math.log(Math.exp(2.0)) - 2.0) < 1e-15);
                System.out.println(Math.abs(Math.atan(Math.tan(0.5)) - 0.5) < 1e-15);
                System.out.println(Math.abs(Math.hypot(5.0, 12.0) - 13.0) < 1e-15);
                System.out.println(Math.ulp(1.0) > 0.0 && Math.ulp(1.0) < 1e-15);
                boolean inRange = true;
                for (int i = 0; i < 100; i++) {
                    double r = Math.random();
                    if (r < 0.0 || r >= 1.0) {
                        inRange = false;
                    }
                }
                System.out.println(inRange);
            }
        }
        ",
        "Trig",
    );
    assert_eq!(out, "true\ntrue\ntrue\ntrue\ntrue\ntrue\n");
}

#[test]
fn scanner_boolean_and_close() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "B.java".into(),
        text: r#"
        import java.util.Scanner;
        public class B {
            public static void main(String[] args) {
                Scanner in = new Scanner(System.in);
                System.out.println(in.hasNextBoolean());
                boolean first = in.nextBoolean();
                boolean second = in.nextBoolean();
                System.out.println(first + " " + second);
                System.out.println(in.hasNextBoolean());
                in.close();
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(["TRUE false 42"]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    vm.run_main("B", &[]).unwrap();
    assert_eq!(console.stdout_text(), "true\ntrue false\nfalse\n");
}

/// `java.util.Scanner` "will not pass the token that caused the exception", so a
/// failed `nextX` leaves the token in place. Without that, the usual recovery
/// loop skips the token AFTER the bad one. Also pins Java's float grammar, which
/// is narrower than Rust's `from_str` (`nan`/`inf` are not floats to Java).
/// Cross-checked against a real JDK by `diff_scanner_values_and_mismatch_recovery`.
#[test]
fn scanner_mismatch_leaves_the_token_and_honours_javas_float_grammar() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("S.java"),
        text: r#"
        import java.util.Scanner;
        import java.util.InputMismatchException;
        public class S {
            public static void main(String[] args) {
                Scanner in = new Scanner(System.in);
                // A bad token stays put, so the recovery loop skips the RIGHT word.
                try { in.nextInt(); } catch (InputMismatchException e) { System.out.println("mismatch"); }
                System.out.println("same: " + in.next());
                // In range for int, out of range for byte: still not consumed.
                try { in.nextByte(); } catch (InputMismatchException e) { System.out.println("range"); }
                System.out.println("as int: " + in.nextInt());
                // Rust parses these as floats; java.util.Scanner does not.
                System.out.println(in.hasNextDouble() + " " + in.hasNextFloat() + " " + in.next());
                System.out.println(in.hasNextDouble() + " " + in.next());
                // But the exact spellings are floats, sign and all.
                System.out.println(in.nextDouble() + " " + in.nextDouble());
                System.out.println(in.nextLong() + " " + in.nextShort() + " " + in.nextByte());
            }
        }
        "#
        .into(),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input([
        "abc 999 inf nan NaN -Infinity 9223372036854775807 -32768 -128",
    ]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    vm.run_main("S", &[]).unwrap();
    assert_eq!(
        console.stdout_text(),
        "mismatch\n\
         same: abc\n\
         range\n\
         as int: 999\n\
         false false inf\n\
         false nan\n\
         NaN -Infinity\n\
         9223372036854775807 -32768 -128\n"
    );
}

#[test]
fn current_time_millis_is_wired() {
    let out = run_stdout(
        r#"
        public class Clock {
            public static void main(String[] args) {
                long start = System.currentTimeMillis();
                long total = 0;
                for (int i = 0; i < 1000; i++) {
                    total += i;
                }
                long elapsed = System.currentTimeMillis() - start;
                System.out.println(total + " " + (elapsed >= 0));
            }
        }
        "#,
        "Clock",
    );
    assert_eq!(out, "499500 true\n");
}

#[test]
fn missing_main_is_reported() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: "NoMain.java".into(),
        text: "class NoMain { static void other() { } }".into(),
    }]);
    assert!(compilation.success());
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("NoMain", &[]);
    assert!(
        matches!(result, Err(VmError::NoMainMethod(_))),
        "{result:?}"
    );
}

#[test]
fn instruction_budget_stops_runaway_bytecode() {
    // A method whose bytecode never reaches `return`: goto 0 would be
    // ideal but isn't implemented yet, so use a tiny budget instead.
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::new();
    let mut vm = Vm::new(
        VmOptions {
            max_instructions: 2,
            ..VmOptions::default()
        },
        &mut vfs,
        &mut console,
    );
    vm.load_class(hand_assembled_hello()).unwrap();
    let result = vm.run_main("Fixture", &[]);
    assert!(
        matches!(result, Err(VmError::InstructionBudgetExceeded)),
        "{result:?}"
    );
}

#[test]
fn string_builder_appends_and_concats_as_content() {
    let out = run_stdout(
        r#"
        public class Main {
            public static void main(String[] args) {
                StringBuilder sb = new StringBuilder();
                sb.append("world");
                String s = "hello " + sb;      // StringBuilder in a + concat
                StringBuilder sb2 = new StringBuilder();
                sb2.append("[").append(sb);    // append(StringBuilder)
                System.out.println(s);
                System.out.println(sb2.toString());
            }
        }
        "#,
        "Main",
    );
    assert_eq!(out, "hello world\n[world\n");
}

#[test]
fn swing_table_read_only_via_model_override() {
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        class ReadOnlyModel extends DefaultTableModel {
            public ReadOnlyModel(Object[][] d, Object[] c) { super(d, c); }
            public boolean isCellEditable(int row, int col) { return false; }
        }
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("RO");
                String[] cols = {"Name", "Score"};
                Object[][] data = {{"Ada", "95"}};
                JTable table = new JTable(new ReadOnlyModel(data, cols));
                frame.add(table);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""editable":false"#),
        "override not honored: {json}"
    );
    assert!(
        json.contains(r#""cells":[["Ada","95"]]"#),
        "data lost: {json}"
    );
}

#[test]
fn swing_table_read_only_via_anonymous_subclass() {
    // The anonymous form with constructor args now works: super(data, cols) is
    // forwarded and the isCellEditable override virtual-dispatches.
    let json = run_swing(
        r#"
        import javax.swing.*;
        import java.awt.*;
        public class Main {
            public static void main(String[] args) {
                JFrame frame = new JFrame("RO");
                String[] cols = {"Name", "Score"};
                Object[][] data = {{"Ada", "95"}};
                JTable table = new JTable(data, cols) {
                    public boolean isCellEditable(int row, int col) { return false; }
                };
                frame.add(table);
                frame.setVisible(true);
            }
        }
        "#,
        "Main",
    );
    assert!(
        json.contains(r#""editable":false"#),
        "override not honored: {json}"
    );
    assert!(
        json.contains(r#""cells":[["Ada","95"]]"#),
        "data lost: {json}"
    );
}

#[test]
fn anonymous_class_forwards_super_args_and_captures_a_local() {
    // The synthesized constructor takes the super-arg (5) first, then the
    // captured local (extra=100): super(5) sets `base`, and the override reads
    // base + extra.
    let out = run_stdout(
        r"
        class Base {
            int base;
            Base(int b) { this.base = b; }
            int compute() { return base; }
        }
        public class Main {
            public static void main(String[] args) {
                int extra = 100;
                Base b = new Base(5) {
                    int compute() { return base + extra; }
                };
                System.out.println(b.compute());
            }
        }
        ",
        "Main",
    );
    assert_eq!(out, "105\n");
}

/// The Java 11 `StringBuilder` surface beyond `append`/`toString`. Every line
/// here is cross-checked against a real JDK by the `diff_string_builder_*`
/// differential tests; this one keeps the coverage on a JDK-less CI box.
#[test]
fn string_builder_exposes_the_java_11_api() {
    let out = run_stdout(
        r#"
        public class B {
            public static void main(String[] args) {
                StringBuilder b = new StringBuilder("abcdef");
                System.out.println(b.reverse());
                System.out.println(b.insert(0, "<").insert(b.length(), 42));
                System.out.println(b.delete(1, 3).deleteCharAt(0).replace(0, 2, "XY"));
                b.setCharAt(0, 'z');
                b.setLength(3);
                System.out.println(b + " " + b.length() + " " + b.charAt(2));

                StringBuilder w = new StringBuilder("hello world, hello");
                System.out.println(w.indexOf("hello") + " " + w.indexOf("hello", 1)
                        + " " + w.lastIndexOf("hello") + " " + w.indexOf("zzz"));
                System.out.println(w.substring(6) + "|" + w.substring(0, 5) + "|" + w.subSequence(2, 4));
                System.out.println(new StringBuilder("abc").compareTo(new StringBuilder("abd")));

                // An initial-capacity hint is accepted and ignored; capacity is
                // not modelled, so ensureCapacity/trimToSize are no-ops.
                StringBuilder c = new StringBuilder(64);
                c.ensureCapacity(200);
                c.append(new char[] {'x', 'y'});
                c.trimToSize();
                c.appendCodePoint(0x1F600);
                System.out.println(c + " " + c.length() + " " + c.codePointCount(0, c.length()));

                // reverse() keeps a surrogate pair together, so the pair lands
                // at the front intact rather than as two swapped halves.
                System.out.println(new StringBuilder(c.toString()).reverse().codePointAt(0));

                char[] dest = new char[3];
                w.getChars(0, 2, dest, 1);
                System.out.println(dest[0] + " " + dest[1] + " " + dest[2]);

                StringBuilder e = new StringBuilder();
                try { e.deleteCharAt(0); } catch (StringIndexOutOfBoundsException x) { System.out.println("deleteCharAt"); }
                try { e.insert(1, "x"); } catch (StringIndexOutOfBoundsException x) { System.out.println("insert"); }
                try { e.appendCodePoint(-1); } catch (IllegalArgumentException x) { System.out.println("appendCodePoint"); }
                // delete tolerates an end past the length.
                System.out.println(new StringBuilder("hello").delete(2, 99));
            }
        }
        "#,
        "B",
    );
    assert_eq!(
        out,
        "fedcba\n\
         <fedcba42\n\
         XYba42\n\
         zYb 3 b\n\
         0 13 13 -1\n\
         world, hello|hello|ll\n\
         -1\n\
         xy\u{1F600} 4 3\n\
         128512\n\
         \u{0} h e\n\
         deleteCharAt\n\
         insert\n\
         appendCodePoint\n\
         he\n"
    );
}

/// `java.util.HashMap` with the JDK's own iteration order, and the boxing
/// that lets a missing key be `null`. Cross-checked against a real JDK by
/// the `diff_hash_map_*` differential tests; this keeps the coverage on a
/// JDK-less CI box.
#[test]
fn hash_map_iterates_and_boxes_like_the_jdk() {
    let out = run_stdout(
        r#"
        import java.util.HashMap;
        import java.util.Map;
        public class H {
            public static void main(String[] args) {
                Map<String, Integer> counts = new HashMap<>();
                counts.put("pear", 4);
                counts.put("apple", 1);
                counts.put("fig", 9);
                // Bucket order, not insertion order.
                System.out.println(counts);
                System.out.println(counts.keySet() + " " + counts.values());

                // A missing key is null, and unboxing it throws.
                System.out.println(counts.get("zz"));
                System.out.println(counts.get("zz") == null);
                try {
                    int bad = counts.get("zz");
                    System.out.println(bad);
                } catch (NullPointerException e) {
                    System.out.println("npe");
                }
                // But `put` as a statement discards the null silently.
                counts.put("kiwi", 2);
                System.out.println(counts.size());

                int total = 0;
                for (int value : counts.values()) {
                    total += value;
                }
                System.out.println(total);
                for (Map.Entry<String, Integer> entry : counts.entrySet()) {
                    entry.setValue(entry.getValue() + 1);
                }
                System.out.println(counts);

                // Keys compare with equals, so -0.0 and 0.0 are distinct.
                Map<Double, String> byDouble = new HashMap<>();
                byDouble.put(-0.0, "neg");
                byDouble.put(0.0, "pos");
                System.out.println(byDouble.size() + " " + byDouble.get(-0.0));
            }
        }
        "#,
        "H",
    );
    assert_eq!(
        out,
        "{apple=1, pear=4, fig=9}\n\
         [apple, pear, fig] [1, 4, 9]\n\
         null\n\
         true\n\
         npe\n\
         4\n\
         16\n\
         {apple=2, pear=5, kiwi=3, fig=10}\n\
         2 neg\n"
    );
}

/// `java.util.HashSet` iterates in a real `HashSet`'s bucket order (it reuses
/// the map machinery), deduplicates, and supports the bulk operations. Pinned
/// JDK-free for CI; the byte-for-byte JDK match is in `differential.rs`.
#[test]
fn hash_set_iterates_and_dedups_like_the_jdk() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.HashSet;
        import java.util.List;
        import java.util.Set;
        public class S {
            public static void main(String[] args) {
                Set<String> s = new HashSet<>();
                System.out.println(s.add("apple") + " " + s.add("banana") + " " + s.add("apple"));
                System.out.println(s.size() + " " + s.contains("banana") + " " + s.contains("cherry"));
                System.out.println(s);                 // bucket order: [banana, apple]

                HashSet<Integer> nums = new HashSet<>();
                for (int i = 10; i >= 1; i--) {
                    nums.add(i);
                }
                System.out.println(nums);              // [1, 2, ..., 10]
                System.out.println(nums.remove(5) + " " + nums.remove(99));
                System.out.println(nums);

                List<Integer> dups = new ArrayList<>();
                dups.add(3); dups.add(1); dups.add(3); dups.add(2); dups.add(1);
                Set<Integer> deduped = new HashSet<>(dups);
                System.out.println(deduped + " " + deduped.size());

                int total = 0;
                for (int x : nums) {
                    total += x;
                }
                System.out.println(total);
            }
        }
        "#,
        "S",
    );
    assert_eq!(
        out,
        "true true false\n\
         2 true false\n\
         [banana, apple]\n\
         [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]\n\
         true false\n\
         [1, 2, 3, 4, 6, 7, 8, 9, 10]\n\
         [1, 2, 3] 3\n\
         50\n"
    );
}

/// A map's `keySet()` presents the mutable `Set` interface: `add` throws
/// `UnsupportedOperationException` (as Java's does — accepting it would be the
/// dangerous direction), while `remove` writes through to the backing map.
#[test]
fn keyset_view_add_throws_but_remove_writes_through() {
    let out = run_stdout(
        r#"
        import java.util.HashMap;
        import java.util.Map;
        import java.util.Set;
        public class K {
            public static void main(String[] args) {
                Map<String, Integer> m = new HashMap<>();
                m.put("a", 1);
                m.put("b", 2);
                Set<String> keys = m.keySet();
                try {
                    keys.add("c");
                    System.out.println("added");
                } catch (UnsupportedOperationException e) {
                    System.out.println("uoe");
                }
                keys.remove("a");
                System.out.println(m);
            }
        }
        "#,
        "K",
    );
    assert_eq!(out, "uoe\n{b=2}\n");
}

/// `java.util.Optional` from the stream terminals: `findFirst`/`max`/`min`/
/// `average`, with `isPresent`/`get`/`orElse` and the `Optional[x]`/`.empty`
/// toString. Pinned JDK-free for CI; the JDK match is in `differential.rs`.
#[test]
fn optionals_from_stream_terminals() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Arrays;
        import java.util.List;
        import java.util.Optional;
        public class O {
            public static void main(String[] args) {
                List<Integer> nums = new ArrayList<>(Arrays.asList(3, 1, 4, 1, 5));
                System.out.println(nums.stream().mapToInt(x -> x).max().getAsInt());   // 5
                System.out.println(nums.stream().mapToInt(x -> x).average().getAsDouble());
                List<String> ws = new ArrayList<>(Arrays.asList("fig", "banana"));
                Optional<String> longest = ws.stream().max((a, b) -> a.length() - b.length());
                System.out.println(longest.get());                                    // banana
                Optional<String> z = ws.stream().filter(w -> w.startsWith("z")).findFirst();
                System.out.println(z.isPresent() + " " + z.orElse("none"));            // false none
                System.out.println(z);                                                // Optional.empty
            }
        }
        "#,
        "O",
    );
    assert_eq!(out, "5\n2.8\nbanana\nfalse none\nOptional.empty\n");
}

/// `Optional.map(function)` transforms a present value (its element erased,
/// like a stream's `map`), and `orElseGet(supplier)` computes a fallback only
/// when absent. Pinned JDK-free; the JDK match is in `diff_optional_map`.
#[test]
fn optional_map_and_or_else_get() {
    let out = run_stdout(
        r#"
        import java.util.Optional;
        public class O {
            public static void main(String[] args) {
                Optional<String> a = Optional.of("hello");
                System.out.println(a.map(s -> s.length()).get());
                System.out.println(a.map(s -> s.toUpperCase()).get());
                Optional<String> e = Optional.empty();
                System.out.println(e.map(s -> s.length()).isPresent());
                System.out.println(a.map(s -> s + "!").orElse("none"));
                System.out.println(e.map(s -> s + "!").orElse("none"));

                System.out.println(a.orElseGet(() -> "default"));
                System.out.println(e.orElseGet(() -> "computed"));
                Optional<Integer> n = Optional.empty();
                System.out.println(n.orElseGet(() -> 6 * 7));
                Optional<Integer> m = Optional.of(10);
                System.out.println(m.orElseGet(() -> 99));
            }
        }
        "#,
        "O",
    );
    assert_eq!(
        out,
        "5\nHELLO\nfalse\nhello!\nnone\nhello\ncomputed\n42\n10\n"
    );
}

/// `Optional.ifPresent(consumer)` runs its lambda only when present, and
/// `filter(predicate)` keeps a present value only if it matches (else empty).
/// Both take a lambda parameterized by the Optional's element. Pinned JDK-free;
/// the byte-for-byte JDK match is in `diff_optional_lambdas`.
#[test]
fn optional_if_present_and_filter() {
    let out = run_stdout(
        r#"
        import java.util.Optional;
        public class O {
            public static void main(String[] args) {
                Optional<String> a = Optional.of("hello");
                a.ifPresent(s -> System.out.println("got " + s));
                Optional<String> b = Optional.empty();
                b.ifPresent(s -> System.out.println("never " + s));

                Optional<Integer> n = Optional.of(7);
                System.out.println(n.filter(x -> x > 5).isPresent());
                System.out.println(n.filter(x -> x > 10).isPresent());
                System.out.println(n.filter(x -> x > 5).get());
                System.out.println(b.filter(s -> s.length() > 0).isPresent());

                Optional<String> c = Optional.of("world");
                System.out.println(c.filter(s -> s.startsWith("w")).orElse("none"));
                System.out.println(c.filter(s -> s.startsWith("z")).orElse("none"));
                System.out.println(c.filter(s -> s.startsWith("z")));
            }
        }
        "#,
        "O",
    );
    assert_eq!(
        out,
        "got hello\n\
         true\n\
         false\n\
         7\n\
         false\n\
         world\n\
         none\n\
         Optional.empty\n"
    );
}

/// The `Optional` factories: `of(x)` / `ofNullable(x)` wrap a present value,
/// `empty()` an absent one, so a method can build and return an `Optional`.
/// Pinned JDK-free; the byte-for-byte JDK match is in `diff_optional_factories`.
#[test]
fn optional_factories_build_present_and_absent() {
    let out = run_stdout(
        r#"
        import java.util.Optional;
        public class O {
            static Optional<String> find(int x) {
                if (x > 0) { return Optional.of("pos"); }
                return Optional.empty();
            }
            public static void main(String[] args) {
                Optional<Integer> a = Optional.of(5);
                System.out.println(a.isPresent() + " " + a.get() + " " + a);
                Optional<String> b = Optional.empty();
                System.out.println(b.isPresent() + " " + b.orElse("none") + " " + b);
                Optional<String> c = Optional.ofNullable("hi");
                System.out.println(c.get());
                System.out.println(find(3).get() + " " + find(-1).isPresent());
            }
        }
        "#,
        "O",
    );
    assert_eq!(
        out,
        "true 5 Optional[5]\n\
         false none Optional.empty\n\
         hi\n\
         pos false\n"
    );
}

/// `java.util.stream.IntStream`: `mapToInt`/`range` and the numeric terminals,
/// with primitive-int lambdas. Pinned JDK-free for CI; the byte-for-byte JDK
/// match is in `differential.rs`.
#[test]
fn int_stream_sums_and_ranges() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Arrays;
        import java.util.List;
        import java.util.stream.IntStream;
        public class I {
            public static void main(String[] args) {
                List<String> words = new ArrayList<>(Arrays.asList("apple", "fig", "banana"));
                System.out.println(words.stream().mapToInt(w -> w.length()).sum());  // 14
                System.out.println(IntStream.range(0, 5).sum());                     // 10
                System.out.println(IntStream.rangeClosed(1, 5).sum());               // 15
                System.out.println(IntStream.range(1, 4).map(i -> i * i).sum());     // 14
                System.out.println(IntStream.rangeClosed(1, 10).filter(i -> i % 2 == 0).count());
                int[] a = IntStream.range(0, 4).toArray();
                System.out.println(Arrays.toString(a));                              // [0, 1, 2, 3]
            }
        }
        "#,
        "I",
    );
    assert_eq!(out, "14\n10\n15\n14\n5\n[0, 1, 2, 3]\n");
}

/// `java.util.stream.Stream`: the filter/map/sorted/collect pipeline, with
/// lambda parameter types flowing from the source collection. Pinned JDK-free
/// for CI; the byte-for-byte JDK match is in `differential.rs`.
#[test]
fn stream_pipeline_filters_maps_and_collects() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.List;
        import java.util.stream.Collectors;
        public class S {
            public static void main(String[] args) {
                List<String> words = new ArrayList<>();
                words.add("apple"); words.add("fig"); words.add("banana"); words.add("kiwi");

                List<String> longUpper = words.stream()
                    .filter(w -> w.length() > 3)
                    .map(w -> w.toUpperCase())
                    .sorted()
                    .collect(Collectors.toList());
                System.out.println(longUpper);              // [APPLE, BANANA, KIWI]

                System.out.println(words.stream().filter(w -> w.length() > 3).count());
                System.out.println(words.stream().anyMatch(w -> w.startsWith("b")));
                System.out.println(words.stream().allMatch(w -> w.length() >= 3));

                String joined = words.stream()
                    .sorted()
                    .collect(Collectors.joining(", ", "[", "]"));
                System.out.println(joined);

                int total = 0;
                for (String w : words) { total += w.length(); }
                System.out.println(total);
            }
        }
        "#,
        "S",
    );
    assert_eq!(
        out,
        "[APPLE, BANANA, KIWI]\n\
         3\n\
         true\n\
         true\n\
         [apple, banana, fig, kiwi]\n\
         18\n"
    );
}

/// `java.util.PriorityQueue` is a real binary min-heap: `peek`/`poll` return the
/// least element, while iteration and `toString` show the heap-array order
/// (Java's exact `siftUp`/`siftDown`). Pinned JDK-free for CI; the byte-for-byte
/// JDK match is in `differential.rs`.
#[test]
fn priority_queue_is_a_binary_min_heap() {
    let out = run_stdout(
        r#"
        import java.util.PriorityQueue;
        public class P {
            public static void main(String[] args) {
                PriorityQueue<Integer> pq = new PriorityQueue<>();
                pq.offer(5); pq.offer(1); pq.offer(3); pq.offer(2); pq.offer(4);
                System.out.println(pq);              // heap-array order, not sorted
                System.out.println(pq.peek());       // 1
                StringBuilder sb = new StringBuilder();
                while (!pq.isEmpty()) { sb.append(pq.poll()).append(" "); }
                System.out.println(sb.toString().trim());  // 1 2 3 4 5

                PriorityQueue<Integer> max = new PriorityQueue<>((a, b) -> b - a);
                max.add(1); max.add(3); max.add(2);
                System.out.println(max.peek() + " " + max.poll() + " " + max.poll());
            }
        }
        "#,
        "P",
    );
    assert_eq!(out, "[1, 2, 3, 5, 4]\n1\n1 2 3 4 5\n3 3 2\n");
}

/// A user `java.util.Comparator` (class or lambda) orders `sort`, `TreeSet`,
/// and `TreeMap` — it aliases the bundled erased `__Comparator`, and its
/// `compare(T, T)` reaches through the VM's erasure bridge. Pinned JDK-free for
/// CI; the byte-for-byte JDK match is in `differential.rs`.
#[test]
fn user_comparator_orders_sorts_and_trees() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Comparator;
        import java.util.List;
        import java.util.TreeSet;
        public class C {
            static class ByLen implements Comparator<String> {
                public int compare(String a, String b) { return a.length() - b.length(); }
            }
            public static void main(String[] args) {
                List<String> ws = new ArrayList<>();
                ws.add("ccc"); ws.add("a"); ws.add("bb");
                ws.sort(new ByLen());
                System.out.println(ws);                  // [a, bb, ccc]

                Comparator<Integer> desc = (a, b) -> b - a;
                List<Integer> ns = new ArrayList<>();
                ns.add(1); ns.add(3); ns.add(2);
                ns.sort(desc);
                System.out.println(ns);                  // [3, 2, 1]

                TreeSet<Integer> t = new TreeSet<>((a, b) -> b - a);
                t.add(1); t.add(3); t.add(2);
                System.out.println(t + " " + t.first()); // [3, 2, 1] 3
            }
        }
        "#,
        "C",
    );
    assert_eq!(out, "[a, bb, ccc]\n[3, 2, 1]\n[3, 2, 1] 3\n");
}

/// The `Comparator` static factories and combinators build comparators from
/// method-reference key extractors and chain them. Pinned JDK-free for CI; the
/// byte-for-byte JDK match is in `diff_comparator_factories`.
#[test]
fn comparator_factories_build_and_chain_comparators() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Comparator;
        import java.util.List;
        public class C {
            static class P {
                String name; int age;
                P(String n, int a) { name = n; age = a; }
                String getName() { return name; }
                int getAge() { return age; }
                public String toString() { return name + "(" + age + ")"; }
            }
            public static void main(String[] args) {
                List<P> ps = new ArrayList<>();
                ps.add(new P("Carol", 30)); ps.add(new P("Alice", 30)); ps.add(new P("Bob", 25));

                ps.sort(Comparator.comparingInt(P::getAge));
                System.out.println(ps);                     // [Bob(25), Carol(30), Alice(30)]

                ps.sort(Comparator.comparingInt(P::getAge).reversed());
                System.out.println(ps);                     // [Carol(30), Alice(30), Bob(25)]

                ps.sort(Comparator.comparingInt(P::getAge).thenComparing(P::getName));
                System.out.println(ps);                     // [Bob(25), Alice(30), Carol(30)]

                ps.sort(Comparator.comparing(P::getName).reversed());
                System.out.println(ps);                     // [Carol(30), Bob(25), Alice(30)]

                List<String> ws = new ArrayList<>();
                ws.add("banana"); ws.add("apple"); ws.add("cherry");
                ws.sort(Comparator.naturalOrder());
                System.out.println(ws);                     // [apple, banana, cherry]
                ws.sort(Comparator.reverseOrder());
                System.out.println(ws);                     // [cherry, banana, apple]
            }
        }
        "#,
        "C",
    );
    assert_eq!(
        out,
        "[Bob(25), Carol(30), Alice(30)]\n\
         [Carol(30), Alice(30), Bob(25)]\n\
         [Bob(25), Alice(30), Carol(30)]\n\
         [Carol(30), Bob(25), Alice(30)]\n\
         [apple, banana, cherry]\n\
         [cherry, banana, apple]\n"
    );
}

/// The immutable Set/Map factories: `emptySet`/`emptyMap`, `singleton(e)`/
/// `singletonMap(k, v)`, and `unmodifiableSet`/`unmodifiableMap` (whose views
/// are unmodifiable too). Every mutator throws. Pinned JDK-free; the
/// byte-for-byte JDK match is in `diff_immutable_set_and_map`.
#[test]
fn collections_immutable_set_and_map() {
    let out = run_stdout(
        r#"
        import java.util.*;
        public class C {
            public static void main(String[] args) {
                Set<String> es = Collections.emptySet();
                System.out.println(es + " " + es.size() + " " + es.isEmpty());

                Map<String, Integer> em = Collections.emptyMap();
                System.out.println(em + " " + em.size());

                Set<Integer> one = Collections.singleton(7);
                System.out.println(one + " " + one.contains(7) + " " + one.contains(8));

                Map<String, Integer> sm = Collections.singletonMap("a", 1);
                System.out.println(sm + " " + sm.get("a") + " " + sm.containsKey("a"));

                Set<Integer> backing = new HashSet<>(Arrays.asList(1, 2, 3));
                Set<Integer> us = Collections.unmodifiableSet(backing);
                System.out.println(us.size() + " " + us.contains(2));

                Map<String, Integer> bm = new HashMap<>();
                bm.put("x", 10); bm.put("y", 20);
                Map<String, Integer> um = Collections.unmodifiableMap(bm);
                int sum = 0;
                for (int v : um.values()) { sum += v; }
                System.out.println(um.get("x") + " " + um.size() + " " + sum);

                try { one.add(9); } catch (UnsupportedOperationException e) { System.out.println("set"); }
                try { sm.put("b", 2); } catch (UnsupportedOperationException e) { System.out.println("map"); }
                try { us.remove(1); } catch (UnsupportedOperationException e) { System.out.println("view"); }
                try { um.keySet().remove("x"); } catch (UnsupportedOperationException e) { System.out.println("keyset"); }
            }
        }
        "#,
        "C",
    );
    assert_eq!(
        out,
        "[] 0 true\n\
         {} 0\n\
         [7] true false\n\
         {a=1} 1 true\n\
         3 true\n\
         10 2 30\n\
         set\nmap\nview\nkeyset\n"
    );
}

/// `Collections.singletonList(e)` is an immutable one-element list;
/// `Collections.reverseOrder()` / `reverseOrder(cmp)` are reversed comparators
/// that order a sort or a `TreeSet`. Pinned JDK-free; the byte-for-byte JDK
/// match is in `diff_collections_helpers`.
#[test]
fn collections_singleton_list_and_reverse_order() {
    let out = run_stdout(
        r#"
        import java.util.*;
        public class C {
            public static void main(String[] args) {
                List<Integer> one = Collections.singletonList(42);
                System.out.println(one + " " + one.size() + " " + one.get(0));

                List<Integer> xs = new ArrayList<>(Arrays.asList(3, 1, 4, 1, 5));
                xs.sort(Collections.reverseOrder());
                System.out.println(xs);

                List<String> ws = new ArrayList<>(Arrays.asList("fig", "apple", "cherry"));
                Comparator<String> byLen = (a, b) -> a.length() - b.length();
                ws.sort(Collections.reverseOrder(byLen));
                System.out.println(ws);

                TreeSet<Integer> t = new TreeSet<>(Collections.reverseOrder());
                t.add(3); t.add(1); t.add(2);
                System.out.println(t);

                try {
                    one.add(9);
                } catch (UnsupportedOperationException e) {
                    System.out.println("immutable");
                }
            }
        }
        "#,
        "C",
    );
    assert_eq!(
        out,
        "[42] 1 42\n\
         [5, 4, 3, 1, 1]\n\
         [cherry, apple, fig]\n\
         [3, 2, 1]\n\
         immutable\n"
    );
}

/// `Collection.removeIf(predicate)` drops the matching elements in place across
/// every collection: `HashSet`/`TreeSet` (rebuilding their own storage),
/// `LinkedList`/`ArrayDeque`/`Stack` (the shared vector), returning whether any
/// went. Pinned JDK-free; the byte-for-byte JDK match is in `diff_remove_if`.
#[test]
fn remove_if_drops_matching_across_collections() {
    let out = run_stdout(
        r#"
        import java.util.*;
        public class R {
            public static void main(String[] args) {
                Set<Integer> hs = new HashSet<>(Arrays.asList(1, 2, 3, 4, 5, 6));
                hs.removeIf(x -> x % 2 == 0);
                System.out.println(hs);

                TreeSet<Integer> ts = new TreeSet<>(Arrays.asList(5, 3, 8, 1, 4, 9, 2));
                ts.removeIf(x -> x > 4);
                System.out.println(ts);

                LinkedList<String> ll = new LinkedList<>(Arrays.asList("apple", "fig", "banana", "kiwi"));
                ll.removeIf(s -> s.length() > 4);
                System.out.println(ll);

                Deque<Integer> dq = new ArrayDeque<>(Arrays.asList(1, 2, 3, 4, 5));
                dq.removeIf(x -> x == 3);
                System.out.println(dq);

                Stack<Integer> st = new Stack<>();
                for (int i = 1; i <= 6; i++) { st.push(i); }
                st.removeIf(x -> x % 3 == 0);
                System.out.println(st);

                Set<Integer> s2 = new TreeSet<>(Arrays.asList(1, 2, 3));
                System.out.println(s2.removeIf(x -> x > 10));
                System.out.println(s2.removeIf(x -> x < 2) + " " + s2);
            }
        }
        "#,
        "R",
    );
    assert_eq!(
        out,
        "[1, 3, 5]\n\
         [1, 2, 3, 4]\n\
         [fig, kiwi]\n\
         [1, 2, 4, 5]\n\
         [1, 2, 4, 5]\n\
         false\n\
         true [2, 3]\n"
    );
}

/// `java.util.ArrayDeque` is a `Deque` and `Queue` (not a `List`): head-based
/// `push`/`pop`/`peekFirst`, tail-based `offer`/`peekLast`, usable as a stack or
/// a FIFO queue, with a copy constructor and for-each. Pinned JDK-free for CI;
/// the byte-for-byte JDK match is in `diff_array_deque`.
#[test]
fn array_deque_is_a_deque_and_a_queue() {
    let out = run_stdout(
        r#"
        import java.util.ArrayDeque;
        import java.util.Arrays;
        import java.util.Deque;
        import java.util.Queue;
        public class D {
            public static void main(String[] args) {
                Deque<Integer> d = new ArrayDeque<>();
                d.addFirst(1); d.addLast(2); d.push(0);
                System.out.println(d + " " + d.peekFirst() + " " + d.peekLast());
                System.out.println(d.pop() + " " + d.pollLast() + " " + d);

                Queue<String> q = new ArrayDeque<>();
                q.offer("a"); q.offer("b"); q.offer("c");
                System.out.println(q.poll() + " " + q);

                ArrayDeque<Integer> st = new ArrayDeque<>();
                for (int i = 1; i <= 3; i++) { st.push(i); }
                StringBuilder sb = new StringBuilder();
                while (!st.isEmpty()) { sb.append(st.pop()); }
                System.out.println(st + " " + sb);

                ArrayDeque<Integer> c = new ArrayDeque<>(Arrays.asList(5, 6, 7));
                int sum = 0;
                for (int x : c) { sum += x; }
                System.out.println(c + " " + sum + " " + c.contains(6) + " " + c.size());
            }
        }
        "#,
        "D",
    );
    assert_eq!(
        out,
        "[0, 1, 2] 0 2\n\
         0 2 [1]\n\
         a [b, c]\n\
         [] 321\n\
         [5, 6, 7] 18 true 3\n"
    );
}

/// `java.util.ArrayDeque` forbids null elements: every insertion throws
/// `NullPointerException`, unlike the null-tolerant `LinkedList`. Pinned
/// JDK-free.
#[test]
fn array_deque_rejects_null_elements() {
    let (result, console) = compile_and_run(
        r#"
        import java.util.ArrayDeque;
        public class D {
            public static void main(String[] args) {
                ArrayDeque<String> d = new ArrayDeque<>();
                d.add("ok");
                try {
                    d.push(null);
                } catch (NullPointerException e) {
                    System.out.println("rejected");
                }
                System.out.println(d);
            }
        }
        "#,
        "D",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "rejected\n[ok]\n");
}

/// `java.util.Stack` is a `Vector`-backed LIFO: `push`/`pop`/`peek` act on the
/// top (the end), `empty`/`search` round out the stack face, and — because it
/// is a `List` — every list method and for-each work too. Pinned JDK-free for
/// CI; the byte-for-byte JDK match is in `diff_stack_lifo_and_list`.
#[test]
fn stack_is_a_lifo_and_a_list() {
    let out = run_stdout(
        r#"
        import java.util.List;
        import java.util.Stack;
        public class S {
            public static void main(String[] args) {
                Stack<Integer> s = new Stack<>();
                s.push(1); s.push(2); s.push(3);
                System.out.println(s);                    // [1, 2, 3]
                System.out.println(s.peek() + " " + s.pop() + " " + s);
                System.out.println(s.empty() + " " + s.size());
                s.push(5);
                System.out.println(s.search(1) + " " + s.search(5) + " " + s.search(99));
                System.out.println(s.get(0) + " " + s.contains(5) + " " + s.indexOf(2));

                // A Stack is a List.
                List<String> ls = new Stack<>();
                ls.add("a"); ls.add("b");
                System.out.println(ls);                   // [a, b]
                for (String x : ls) { System.out.print(x); }
                System.out.println();

                // Drain it LIFO.
                Stack<String> st = new Stack<>();
                st.push("x"); st.push("y");
                while (!st.empty()) { System.out.print(st.pop()); }
                System.out.println(" tail=" + s);
            }
        }
        "#,
        "S",
    );
    assert_eq!(
        out,
        "[1, 2, 3]\n\
         3 3 [1, 2]\n\
         false 2\n\
         3 1 -1\n\
         1 true 1\n\
         [a, b]\n\
         ab\n\
         yx tail=[1, 2, 5]\n"
    );
}

/// An empty `Stack.pop()`/`peek()` throws `EmptyStackException`, not the
/// `NoSuchElementException` a `Deque` throws. Pinned JDK-free.
#[test]
fn empty_stack_pop_throws_empty_stack_exception() {
    let (result, console) = compile_and_run(
        r#"
        import java.util.Stack;
        public class S {
            public static void main(String[] args) {
                Stack<Integer> s = new Stack<>();
                try {
                    s.pop();
                } catch (java.util.EmptyStackException e) {
                    System.out.println("caught empty");
                }
                System.out.println("done");
            }
        }
        "#,
        "S",
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(console.stdout_text(), "caught empty\ndone\n");
}

/// `java.util.TreeMap` keeps its entries sorted by key, so iteration, the
/// views, and the key navigation all read off the sorted vector. Pinned
/// JDK-free for CI; the byte-for-byte JDK match is in `differential.rs`.
#[test]
fn tree_map_keeps_entries_sorted_by_key() {
    let out = run_stdout(
        r#"
        import java.util.Map;
        import java.util.TreeMap;
        public class T {
            public static void main(String[] args) {
                TreeMap<String, Integer> t = new TreeMap<>();
                t.put("banana", 3); t.put("apple", 5); t.put("cherry", 1);
                System.out.println(t);                          // {apple=5, banana=3, cherry=1}
                System.out.println(t.firstKey() + " " + t.lastKey());
                System.out.println(t.floorKey("b") + " " + t.ceilingKey("b"));
                System.out.println(t.keySet() + " " + t.values());
                System.out.println(t.get("apple") + " " + t.getOrDefault("z", -1));
                t.put("apple", 9);
                System.out.println(t.remove("cherry") + " " + t);

                TreeMap<Integer, Integer> n = new TreeMap<>();
                for (int i = 4; i >= 1; i--) { n.put(i, i * i); }
                int sum = 0;
                for (Map.Entry<Integer, Integer> e : n.entrySet()) { sum += e.getValue(); }
                System.out.println(n + " " + sum);
            }
        }
        "#,
        "T",
    );
    assert_eq!(
        out,
        "{apple=5, banana=3, cherry=1}\n\
         apple cherry\n\
         apple banana\n\
         [apple, banana, cherry] [5, 3, 1]\n\
         5 -1\n\
         1 {apple=9, banana=3}\n\
         {1=1, 2=4, 3=9, 4=16} 30\n"
    );
}

/// `java.util.TreeSet` keeps its elements sorted (natural / `Comparable`
/// ordering), deduplicates by `compareTo == 0`, and answers the navigation
/// methods off the sorted vector. Pinned JDK-free for CI; the byte-for-byte
/// JDK match is in `differential.rs`.
#[test]
fn tree_set_keeps_elements_sorted() {
    let out = run_stdout(
        r#"
        import java.util.TreeSet;
        public class T {
            public static void main(String[] args) {
                TreeSet<Integer> t = new TreeSet<>();
                t.add(5); t.add(1); t.add(3); t.add(1); t.add(9);
                System.out.println(t);                              // [1, 3, 5, 9]
                System.out.println(t.first() + " " + t.last());     // 1 9
                System.out.println(t.floor(4) + " " + t.ceiling(4));// 3 5
                System.out.println(t.lower(5) + " " + t.higher(5)); // 3 9
                System.out.println(t.floor(0) + " " + t.ceiling(99));// null null
                System.out.println(t.contains(3) + " " + t.remove(3));
                System.out.println(t.pollFirst() + " " + t);        // 1 [5, 9]
                int sum = 0;
                for (int x : t) { sum += x; }
                System.out.println(sum);                            // 14
            }
        }
        "#,
        "T",
    );
    assert_eq!(
        out,
        "[1, 3, 5, 9]\n\
         1 9\n\
         3 5\n\
         3 9\n\
         null null\n\
         true true\n\
         1 [5, 9]\n\
         14\n"
    );
}

/// `java.util.LinkedList` as a `Queue`/`Deque`/`List`: order matches a list's,
/// the empty ends return `null` (poll/peek) or throw (remove), and the
/// interface faces expose the right methods. Pinned JDK-free for CI; the
/// byte-for-byte JDK match is in `differential.rs`.
#[test]
fn linked_list_serves_as_queue_deque_and_list() {
    let out = run_stdout(
        r#"
        import java.util.Deque;
        import java.util.LinkedList;
        import java.util.Queue;
        public class L {
            public static void main(String[] args) {
                Queue<Integer> q = new LinkedList<>();
                q.add(1); q.offer(2); q.offer(3);
                System.out.println(q.poll() + " " + q.peek() + " " + q.size());
                System.out.println(q);                // [2, 3]

                Deque<Integer> st = new LinkedList<>();
                st.push(10); st.push(20);
                System.out.println(st.pop() + " " + st.peek());
                System.out.println(st);               // [10]

                LinkedList<String> ll = new LinkedList<>();
                ll.addFirst("b"); ll.addFirst("a"); ll.addLast("c");
                System.out.println(ll + " " + ll.get(1) + " " + ll.getLast());

                Queue<Integer> empty = new LinkedList<>();
                System.out.println(empty.poll() + " " + empty.peek());
                try { empty.remove(); } catch (java.util.NoSuchElementException e) {
                    System.out.println("nse");
                }
            }
        }
        "#,
        "L",
    );
    assert_eq!(
        out,
        "1 2 2\n\
         [2, 3]\n\
         20 10\n\
         [10]\n\
         [a, b, c] b c\n\
         null null\n\
         nse\n"
    );
}

/// Java's `Map` members that caturra cannot model name a reason rather than
/// pretending they do not exist. The reason has to win over the arguments'
/// own errors: `map.forEach(lambda)` must blame the missing lambda support,
/// not the lambda, which is in a perfectly good position.
#[test]
fn unsupported_map_members_explain_themselves() {
    for (source, want) in [
        (
            "import java.util.ArrayList; class M { static void r() { new ArrayList<String>().toArray(); } }",
            "ArrayList.toArray exists in Java, but Object arrays are not supported by caturra",
        ),
        (
            "import java.util.HashMap; class M { static void r() { new HashMap<String, Integer>().merge(null, null, null); } }",
            "HashMap.merge exists in Java, but lambdas are not supported by caturra",
        ),
        (
            "import java.util.HashMap; class M { static void r() { new HashMap<String, Integer>().keySet().iterator(); } }",
            "Set.iterator exists in Java, but iterators are not supported by caturra (use for-each)",
        ),
        (
            "import java.util.HashMap; class M { static void r() { new HashMap<String, Integer>().values().removeIf(x -> true); } }",
            "Collection.removeIf exists in Java, but lambdas are not supported by caturra",
        ),
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// A value coerced to text uses its own `toString()`, even when the coercion
/// happens inside an intrinsic that only the VM can see into. Cross-checked
/// against a real JDK by `diff_to_string_inside_collections` and
/// `diff_to_string_edge_cases`; this keeps the coverage on a JDK-less CI box.
#[test]
fn collections_render_elements_with_their_own_to_string() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.HashMap;
        import java.util.Map;

        class Student {
            String name;
            Student(String name) { this.name = name; }
            public String toString() { return "S(" + name + ")"; }
        }

        class Boom {
            public String toString() { throw new IllegalStateException("boom"); }
        }

        public class R {
            public static void main(String[] args) {
                Student ann = new Student("Ann");
                ArrayList<Student> list = new ArrayList<Student>();
                list.add(ann);
                System.out.println(list);
                System.out.println("concat: " + list);

                Map<String, Student> byName = new HashMap<String, Student>();
                byName.put("a", ann);
                System.out.println(byName + " " + byName.values() + " " + byName.entrySet());

                StringBuilder sb = new StringBuilder();
                sb.append(ann);
                System.out.println(sb);

                // A collection holding itself renders as Java's marker.
                ArrayList<Object> self = new ArrayList<Object>();
                self.add(self);
                System.out.println(self);

                // An exception from toString reaches the caller's catch.
                ArrayList<Boom> boom = new ArrayList<Boom>();
                boom.add(new Boom());
                try {
                    System.out.println(boom);
                } catch (IllegalStateException e) {
                    System.out.println("caught " + e.getMessage());
                }

                // Two collections holding each other overflow, they do not hang.
                ArrayList<Object> a = new ArrayList<Object>();
                ArrayList<Object> b = new ArrayList<Object>();
                a.add(b);
                b.add(a);
                try {
                    System.out.println(a);
                } catch (StackOverflowError e) {
                    System.out.println("cycle overflows");
                }
            }
        }
        "#,
        "R",
    );
    assert_eq!(
        out,
        "[S(Ann)]\n\
         concat: [S(Ann)]\n\
         {a=S(Ann)} [S(Ann)] [a=S(Ann)]\n\
         S(Ann)\n\
         [(this Collection)]\n\
         caught boom\n\
         cycle overflows\n"
    );
}

/// `Collections.sort` orders user objects by their own `compareTo`, stably.
/// Before the interpreter could call back into Java it compared every pair of
/// instances as equal, and so silently left the list alone. Cross-checked
/// against a real JDK by `diff_collections_sort_uses_compare_to`.
#[test]
fn collections_sort_calls_compare_to() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Collections;

        class Card implements Comparable<Card> {
            int rank;
            String tag;
            Card(int rank, String tag) { this.rank = rank; this.tag = tag; }
            public int compareTo(Card other) { return rank - other.rank; }
            public String toString() { return "C" + rank + tag; }
        }

        public class S {
            public static void main(String[] args) {
                ArrayList<Card> cards = new ArrayList<Card>();
                cards.add(new Card(3, "a"));
                cards.add(new Card(1, "b"));
                cards.add(new Card(2, "c"));
                cards.add(new Card(1, "d"));
                Collections.sort(cards);
                // Sorted, and the two rank-1 cards keep their original order.
                System.out.println(cards);
            }
        }
        "#,
        "S",
    );
    assert_eq!(out, "[C1b, C1d, C2c, C3a]\n");
}

/// `Collections.sort`/`max`/`min`/`binarySearch` are declared over
/// `T extends Comparable<? super T>`, so a list of a class that does not
/// implement `Comparable` is a compile error — not, as it was until
/// 2026-07-09, a `ClassCastException` at run time. That was the last place
/// caturra accepted a program javac rejects. `Arrays.sort` already refused
/// one, because its bundled parameter is `Comparable[]`. Pinned against
/// javac by `reject_sorting_a_list_of_non_comparables` and friends.
#[test]
fn collections_comparable_bound_rejects_like_javac() {
    for (source, want) in [
        (
            "Collections.sort(new ArrayList<Plain>());",
            "no suitable method found for sort(ArrayList<Plain>) in class Collections",
        ),
        (
            "Plain p = Collections.max(new ArrayList<Plain>());",
            "no suitable method found for max(ArrayList<Plain>) in class Collections",
        ),
        (
            "Plain p = Collections.min(new ArrayList<Plain>());",
            "no suitable method found for min(ArrayList<Plain>) in class Collections",
        ),
        (
            "int i = Collections.binarySearch(new ArrayList<Plain>(), new Plain());",
            "no suitable method found for binarySearch(ArrayList<Plain>,Plain) in class Collections",
        ),
        (
            "Collections.sort(new ArrayList<Object>());",
            "no suitable method found for sort(ArrayList<Object>) in class Collections",
        ),
    ] {
        let text = format!(
            "import java.util.*; class Plain {{ int x; }} \
             class M {{ static void r() {{ {source} }} }}"
        );
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// The bound applies to exactly the four methods that declare it. javac puts
/// none on `reverse`/`shuffle`/`swap`/`frequency`/`nCopies`, so neither do we
/// — a new strictness that over-reaches is as wrong as a missing one.
#[test]
fn the_comparable_bound_does_not_reach_the_unbounded_collections_methods() {
    let out = run_stdout(
        r"
        import java.util.ArrayList;
        import java.util.Collections;

        class Plain { int x; }

        public class M {
            public static void main(String[] args) {
                ArrayList<Plain> items = new ArrayList<Plain>();
                Plain one = new Plain();
                items.add(one);
                items.add(new Plain());
                Collections.reverse(items);
                Collections.shuffle(items);
                Collections.swap(items, 0, 1);
                System.out.println(items.size());
                System.out.println(Collections.frequency(items, one));
            }
        }
        ",
        "M",
    );
    assert_eq!(out, "2\n1\n");
}

/// A `Comparable` element still sorts, including one that inherits the
/// interface from a superclass rather than declaring it.
#[test]
fn the_comparable_bound_accepts_an_inherited_comparable() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Collections;

        class Base implements Comparable<Base> {
            int rank;
            Base(int rank) { this.rank = rank; }
            public int compareTo(Base other) { return this.rank - other.rank; }
            public String toString() { return "" + this.rank; }
        }
        class Sub extends Base {
            Sub(int rank) { super(rank); }
        }

        public class M {
            public static void main(String[] args) {
                ArrayList<Sub> items = new ArrayList<Sub>();
                items.add(new Sub(3));
                items.add(new Sub(1));
                Collections.sort(items);
                System.out.println(items);
                System.out.println(Collections.max(items));
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "[1, 3]\n3\n");
}

/// A collection compares its elements with their own `equals`, and hashes its
/// keys with their own `hashCode`. Cross-checked against a real JDK by
/// `diff_user_equals_in_collections` and `diff_equals_edge_cases`.
#[test]
fn collections_use_the_user_equals_and_hash_code() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.HashMap;
        import java.util.Map;

        class Point {
            int x;
            int y;
            Point(int x, int y) { this.x = x; this.y = y; }
            public boolean equals(Object other) {
                if (!(other instanceof Point)) return false;
                Point that = (Point) other;
                return x == that.x && y == that.y;
            }
            public int hashCode() { return 31 * x + y; }
            public String toString() { return "(" + x + "," + y + ")"; }
        }

        // equals without hashCode: the map cannot find the key again, exactly
        // as on a real JVM.
        class Broken {
            int id;
            Broken(int id) { this.id = id; }
            public boolean equals(Object other) {
                return other instanceof Broken && ((Broken) other).id == id;
            }
        }

        public class E {
            public static void main(String[] args) {
                ArrayList<Point> list = new ArrayList<Point>();
                list.add(new Point(1, 2));
                list.add(new Point(3, 4));
                System.out.println(list.contains(new Point(1, 2)) + " " + list.indexOf(new Point(3, 4)));
                System.out.println(list.remove(new Point(1, 2)) + " " + list);

                ArrayList<Point> one = new ArrayList<Point>();
                one.add(new Point(1, 2));
                ArrayList<Point> two = new ArrayList<Point>();
                two.add(new Point(1, 2));
                System.out.println(one.equals(two) + " " + (one.hashCode() == two.hashCode()));

                Map<Point, String> map = new HashMap<Point, String>();
                map.put(new Point(1, 2), "first");
                map.put(new Point(1, 2), "second");
                System.out.println(map.size() + " " + map.get(new Point(1, 2)) + " " + map);

                Map<Broken, String> broken = new HashMap<Broken, String>();
                broken.put(new Broken(1), "x");
                System.out.println(broken.get(new Broken(1)) + " " + broken.containsKey(new Broken(1)));

                // Double.equals compares bits: -0.0 is a distinct key, NaN is
                // its own. `==` disagrees with both.
                Map<Double, String> byDouble = new HashMap<Double, String>();
                byDouble.put(0.0, "pos");
                byDouble.put(-0.0, "neg");
                byDouble.put(Double.NaN, "nan");
                System.out.println(byDouble.size() + " " + byDouble.get(Double.NaN));

                // Boolean hashes to 1231/1237, not 0/1.
                Map<Boolean, String> flags = new HashMap<Boolean, String>();
                flags.put(true, "t");
                flags.put(false, "f");
                System.out.println(flags);
            }
        }
        "#,
        "E",
    );
    assert_eq!(
        out,
        "true 1\n\
         true [(3,4)]\n\
         true true\n\
         1 second {(1,2)=second}\n\
         null false\n\
         3 nan\n\
         {false=f, true=t}\n"
    );
}

/// A compound assignment unboxes a wrapper on either side and boxes the
/// result back (JLS §15.26.2). `total += map.get(key)` is the reason it
/// matters; `Integer x = 5; x += 1;` used to leave a raw int in a reference
/// slot, which only failed later. Cross-checked by
/// `diff_compound_assignment_unboxes`.
#[test]
fn compound_assignment_unboxes_and_reboxes() {
    let out = run_stdout(
        r#"
        import java.util.HashMap;
        import java.util.Map;
        public class C {
            static Integer field = 10;
            public static void main(String[] args) {
                Map<String, Double> weights = new HashMap<String, Double>();
                weights.put("a", 1.5);
                double total = 0;
                total += weights.get("a");
                System.out.println(total);

                Integer counter = 5;
                counter += 1;
                counter++;
                System.out.println(counter + " " + (counter instanceof Integer));

                Long big = 5L;
                big += 1;
                System.out.println(big);

                field += 5;
                System.out.println(field);

                int[] cells = {1};
                Integer step = 3;
                cells[0] += step;
                System.out.println(cells[0]);

                char letter = 'a';
                Integer one = 1;
                letter += one;
                System.out.println(letter);
            }
        }
        "#,
        "C",
    );
    assert_eq!(out, "1.5\n7 true\n6\n15\n4\nb\n");
}

/// `Arrays.equals`/`hashCode` ask each element's own `equals`/`hashCode`, and
/// `Arrays.sort` of a reference array asks their `compareTo`. Cross-checked
/// against a real JDK by `diff_arrays_equals_and_hash_code` and
/// `diff_arrays_sort_uses_compare_to`.
#[test]
fn arrays_compare_elements_with_their_own_methods() {
    let out = run_stdout(
        r#"
        import java.util.Arrays;

        class Point {
            int x;
            Point(int x) { this.x = x; }
            public boolean equals(Object other) {
                return other instanceof Point && ((Point) other).x == x;
            }
            public int hashCode() { return x; }
            public String toString() { return "P" + x; }
        }

        class Card implements Comparable<Card> {
            int rank;
            String tag;
            Card(int rank, String tag) { this.rank = rank; this.tag = tag; }
            public int compareTo(Card other) { return rank - other.rank; }
            public String toString() { return "C" + rank + tag; }
        }

        public class A {
            public static void main(String[] args) {
                Point[] points = {new Point(1), new Point(2)};
                Point[] same = {new Point(1), new Point(2)};
                System.out.println(Arrays.equals(points, same) + " "
                        + (Arrays.hashCode(points) == Arrays.hashCode(same)));

                // A null element compares null-safely.
                String[] words = {"a", null};
                String[] sameWords = {"a", null};
                System.out.println(Arrays.equals(words, sameWords));

                // Doubles compare raw bits: NaN equals itself, -0.0 does not
                // equal 0.0. `==` says the opposite of both.
                System.out.println(Arrays.equals(new double[] {Double.NaN}, new double[] {Double.NaN})
                        + " " + Arrays.equals(new double[] {0.0}, new double[] {-0.0}));

                // A reference array sorts by compareTo, stably.
                Card[] cards = {new Card(2, "a"), new Card(1, "b"), new Card(1, "c")};
                Arrays.sort(cards);
                System.out.println(Arrays.toString(cards));

                // A null array, and an element that is itself an array.
                int[] none = null;
                System.out.println(Arrays.equals(none, none) + " " + Arrays.hashCode(none));
                int[] inner = {1};
                System.out.println(Arrays.equals(new Object[] {inner}, new Object[] {inner})
                        + " " + Arrays.equals(new Object[] {inner}, new Object[] {new int[] {1}}));
            }
        }
        "#,
        "A",
    );
    assert_eq!(
        out,
        "true true\n\
         true\n\
         true false\n\
         [C1b, C1c, C2a]\n\
         true 0\n\
         true false\n"
    );
}

/// A trailing comma is legal in an array initializer (JLS §10.6), and an
/// omitted element is not. Cross-checked by `diff_array_initializer_trailing_comma`.
#[test]
fn array_initializers_allow_a_trailing_comma() {
    let out = run_stdout(
        r#"
        public class T {
            public static void main(String[] args) {
                int[] numbers = {1, 2,};
                int[][] grid = {
                    {1, 2,},
                    {3, 4,},
                };
                System.out.println(numbers.length + " " + grid[1][1] + " " + new int[] {5,}[0]);
            }
        }
        "#,
        "T",
    );
    assert_eq!(out, "2 4 5\n");

    let rejected = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("T.java"),
        text: String::from("class T { static void r() { int[] x = {1,,2}; } }"),
    }]);
    assert!(
        !rejected.success(),
        "an omitted element is not a trailing comma"
    );
}

/// `Arrays.deepToString`/`deepEquals`/`deepHashCode` recurse into element
/// arrays, keeping each row's element type: a `boolean[]` prints `true` where
/// an `int[]` prints `1`, and hashes 1231/1237 rather than its value.
/// Cross-checked against a real JDK by `diff_arrays_deep_operations`.
#[test]
fn arrays_deep_operations_recurse_into_rows() {
    let out = run_stdout(
        r#"
        import java.util.Arrays;

        class Point {
            int x;
            Point(int x) { this.x = x; }
            public boolean equals(Object other) {
                return other instanceof Point && ((Point) other).x == x;
            }
            public int hashCode() { return x; }
            public String toString() { return "P" + x; }
        }

        public class D {
            public static void main(String[] args) {
                int[][] ints = {{1, 2}, {3}};
                System.out.println(Arrays.deepToString(ints));
                System.out.println(Arrays.deepToString(new boolean[][] {{true, false}}));
                System.out.println(Arrays.deepToString(new char[][] {{'a', 'b'}}));
                System.out.println(Arrays.deepToString(new Point[][] {{new Point(1)}}));

                // deepEquals compares rows element-wise; equals compares them
                // by identity, so it says false for equal-but-distinct rows.
                int[][] same = {{1, 2}, {3}};
                System.out.println(Arrays.deepEquals(ints, same) + " " + Arrays.equals(ints, same));
                System.out.println(Arrays.deepHashCode(ints) == Arrays.deepHashCode(same));
                System.out.println(Arrays.deepHashCode(ints) + " "
                        + Arrays.deepHashCode(new boolean[][] {{true, false}}));

                // A boolean[] never equals an int[] of the same shape.
                System.out.println(Arrays.deepEquals(
                        new Object[] {new boolean[] {true}}, new Object[] {new int[] {1}}));

                // Nulls, an empty array, and a self-reference.
                System.out.println(Arrays.deepToString(null) + " " + Arrays.deepHashCode(null)
                        + " " + Arrays.deepEquals(null, null));
                System.out.println(Arrays.deepToString(new Object[0]));
                Object[] self = new Object[1];
                self[0] = self;
                System.out.println(Arrays.deepToString(self));

                // deepHashCode has no cycle guard, so it overflows as Java's does.
                try {
                    System.out.println(Arrays.deepHashCode(self));
                } catch (StackOverflowError e) {
                    System.out.println("overflows");
                }
            }
        }
        "#,
        "D",
    );
    assert_eq!(
        out,
        "[[1, 2], [3]]\n\
         [[true, false]]\n\
         [[a, b]]\n\
         [[P1]]\n\
         true false\n\
         true\n\
         31809 40390\n\
         false\n\
         null 0 true\n\
         []\n\
         [[...]]\n\
         overflows\n"
    );
}

/// Every primitive leaf type survives `multianewarray`. `long[][]`,
/// `float[][]`, `short[][]` and `byte[][]` used to allocate their rows as
/// reference arrays, so the first store into one failed.
#[test]
fn two_dimensional_arrays_of_every_primitive() {
    let out = run_stdout(
        r#"
        public class G {
            public static void main(String[] args) {
                long[][] longs = new long[2][2];
                float[][] floats = new float[2][2];
                short[][] shorts = new short[2][2];
                byte[][] bytes = new byte[2][2];
                boolean[][] flags = new boolean[2][2];
                char[][] letters = new char[2][2];
                longs[0][0] = 5L;
                floats[0][0] = 1.5f;
                shorts[0][0] = (short) 3;
                bytes[0][0] = (byte) 4;
                flags[0][0] = true;
                letters[0][0] = 'x';
                System.out.println(longs[0][0] + " " + floats[0][0] + " " + shorts[0][0]);
                System.out.println(bytes[0][0] + " " + flags[0][0] + " " + letters[0][0]);
                System.out.println(longs[1][1] + " " + floats[1][1] + " " + flags[1][1]);
                // A multi-dimensional array is an Object[] of its rows.
                Object[] rows = longs;
                System.out.println(rows.length);
            }
        }
        "#,
        "G",
    );
    assert_eq!(out, "5 1.5 3\n4 true x\n0 0.0 false\n2\n");
}

/// `Arrays.copyOf`/`copyOfRange`/`fill`/`binarySearch` over every array kind.
/// The VM answers them, so `copyOf` returns an array of the source's own kind
/// and a reference `binarySearch` compares with the element's `compareTo`.
/// Cross-checked against a real JDK by `diff_arrays_copy_fill_and_binary_search`.
#[test]
fn arrays_copy_fill_and_binary_search() {
    let out = run_stdout(
        r#"
        import java.util.Arrays;

        class Card implements Comparable<Card> {
            int rank;
            Card(int rank) { this.rank = rank; }
            public int compareTo(Card other) { return rank - other.rank; }
            public String toString() { return "C" + rank; }
        }

        public class B {
            public static void main(String[] args) {
                int[] sorted = {1, 3, 5, 7};
                System.out.println(Arrays.binarySearch(sorted, 5) + " " + Arrays.binarySearch(sorted, 4));
                System.out.println(Arrays.binarySearch(sorted, 1, 3, 5));
                System.out.println(Arrays.binarySearch(new Card[] {new Card(1), new Card(3)}, new Card(3)));
                // -0.0 sorts below 0.0 and NaN above everything.
                double[] doubles = {-0.0, 0.0, 1.5, Double.NaN};
                System.out.println(Arrays.binarySearch(doubles, -0.0) + " "
                        + Arrays.binarySearch(doubles, Double.NaN));

                // copyOf keeps the source's own type and pads with the default.
                System.out.println(Arrays.toString(Arrays.copyOf(sorted, 6)));
                String[] words = Arrays.copyOf(new String[] {"a"}, 3);
                System.out.println(Arrays.toString(words));
                System.out.println(Arrays.toString(Arrays.copyOf(new boolean[] {true}, 2)));
                System.out.println(Arrays.deepToString(Arrays.copyOf(new int[][] {{1}}, 2)));
                System.out.println(Arrays.toString(Arrays.copyOfRange(sorted, 2, 6)));

                int[] filled = new int[4];
                Arrays.fill(filled, 7);
                Arrays.fill(filled, 1, 3, 9);
                System.out.println(Arrays.toString(filled));
                char[] letters = new char[2];
                Arrays.fill(letters, 'q');
                System.out.println(new String(letters));

                // Java's own bounds checks.
                try { Arrays.copyOf(sorted, -1); }
                catch (NegativeArraySizeException e) { System.out.println("negative"); }
                try { Arrays.fill(filled, 3, 1, 0); }
                catch (IllegalArgumentException e) { System.out.println("from > to"); }
                try { Arrays.binarySearch(new String[] {"a"}, null); }
                catch (NullPointerException e) { System.out.println("null key"); }
            }
        }
        "#,
        "B",
    );
    assert_eq!(
        out,
        "2 -3\n\
         2\n\
         1\n\
         0 3\n\
         [1, 3, 5, 7, 0, 0]\n\
         [a, null, null]\n\
         [true, false]\n\
         [[1], null]\n\
         [5, 7, 0, 0]\n\
         [7, 9, 9, 7]\n\
         qq\n\
         negative\n\
         from > to\n\
         null key\n"
    );
}

/// A natural-ordering sort calls `compareTo`, so a null element throws — but a
/// lone element is never compared. `Collections.sort` used to treat every null
/// as equal and quietly sort around it.
#[test]
fn sorting_a_null_element_throws() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Collections;
        public class S {
            public static void main(String[] args) {
                ArrayList<String> words = new ArrayList<String>();
                words.add("b");
                words.add(null);
                try {
                    Collections.sort(words);
                    System.out.println("sorted");
                } catch (NullPointerException e) {
                    System.out.println("throws");
                }
                ArrayList<String> lone = new ArrayList<String>();
                lone.add(null);
                Collections.sort(lone);
                System.out.println(lone);
            }
        }
        "#,
        "S",
    );
    assert_eq!(out, "throws\n[null]\n");
}

/// The four VM-answered `Arrays` methods accept and reject exactly what javac
/// does, with javac's wording. `boolean[]` has no `binarySearch` — booleans
/// have no order — and `copyOf` of a `String[]` is a `String[]`.
#[test]
fn arrays_copy_fill_reject_like_javac() {
    for (source, want) in [
        (
            "boolean[] b = {true}; int i = Arrays.binarySearch(b, true);",
            "no suitable method found for binarySearch(boolean[],boolean) in class Arrays",
        ),
        (
            "int[] x = {1}; int i = Arrays.binarySearch(x);",
            "no suitable method found for binarySearch(int[]) in class Arrays",
        ),
        (
            "int[] x = {1}; Arrays.fill(x, 0, 1);",
            "no suitable method found for fill(int[],int,int) in class Arrays",
        ),
        (
            "Arrays.fill(5, 1);",
            "no suitable method found for fill(int,int) in class Arrays",
        ),
        (
            "int[] x = {1}; Arrays.fill(x, \"s\");",
            "incompatible types: String cannot be converted to int",
        ),
        (
            "String[] s = {\"a\"}; int[] t = Arrays.copyOf(s, 2);",
            "incompatible types: String[] cannot be converted to int[]",
        ),
    ] {
        let text = format!("import java.util.Arrays; class M {{ static void r() {{ {source} }} }}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// `Collections.max`/`min`/`frequency`/`nCopies`/`shuffle`. The VM answers the
/// first four, because a list stores unboxed primitives and `max`/`min` must
/// hand back the element's own type; `shuffle` is bundled Java over `Random`,
/// so a seeded one replays the JDK's permutation. Cross-checked against a real
/// JDK by `diff_collections_helpers`.
#[test]
fn collections_helpers_use_compare_to_and_equals() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Collections;
        import java.util.List;
        import java.util.Random;

        class Card implements Comparable<Card> {
            int rank;
            String tag;
            Card(int rank, String tag) { this.rank = rank; this.tag = tag; }
            public int compareTo(Card other) { return rank - other.rank; }
            public String toString() { return "C" + rank + tag; }
        }

        public class C {
            public static void main(String[] args) {
                ArrayList<Integer> numbers = new ArrayList<Integer>();
                numbers.add(3);
                numbers.add(1);
                numbers.add(5);
                numbers.add(1);
                int biggest = Collections.max(numbers);
                System.out.println(biggest + " " + Collections.min(numbers));
                System.out.println(Collections.frequency(numbers, 1) + " "
                        + Collections.frequency(numbers, 9));

                // A user class compares by its own compareTo; ties keep the first.
                ArrayList<Card> cards = new ArrayList<Card>();
                cards.add(new Card(5, "b"));
                cards.add(new Card(5, "c"));
                cards.add(new Card(1, "d"));
                System.out.println(Collections.max(cards) + " " + Collections.min(cards));

                List<String> copies = Collections.nCopies(3, "z");
                System.out.println(copies + " " + Collections.nCopies(0, "q"));

                // A seeded Random replays the JDK's permutation exactly.
                ArrayList<Integer> deck = new ArrayList<Integer>();
                for (int i = 0; i < 8; i++) {
                    deck.add(i);
                }
                Collections.shuffle(deck, new Random(42));
                System.out.println(deck);

                // max/min call compareTo, so a null element throws, and an
                // empty collection has no maximum.
                ArrayList<Integer> empty = new ArrayList<Integer>();
                try { Collections.max(empty); }
                catch (java.util.NoSuchElementException e) { System.out.println("empty"); }
                try { Collections.nCopies(-1, "a"); }
                catch (IllegalArgumentException e) { System.out.println("negative"); }
                ArrayList<String> holes = new ArrayList<String>();
                holes.add("a");
                holes.add(null);
                try { Collections.max(holes); }
                catch (NullPointerException e) { System.out.println("null element"); }
                System.out.println(Collections.frequency(holes, null));
            }
        }
        "#,
        "C",
    );
    assert_eq!(
        out,
        "5 1\n\
         2 0\n\
         C5b C1d\n\
         [z, z, z] []\n\
         [2, 6, 3, 1, 4, 0, 7, 5]\n\
         empty\n\
         negative\n\
         null element\n\
         1\n"
    );
}

/// The `Collections` helpers reject what javac rejects. `shuffle`'s second
/// argument is a `Random` and nothing else — accepting an `int` there would
/// compile here and fail on a JDK.
#[test]
fn collections_helpers_reject_like_javac() {
    for (source, want) in [
        (
            "Collections.max(5);",
            "no suitable method found for max(int) in class Collections",
        ),
        (
            "ArrayList<Integer> l = new ArrayList<Integer>(); Collections.shuffle(l, 5);",
            "incompatible types: int cannot be converted to Random",
        ),
        (
            "ArrayList<Integer> l = new ArrayList<Integer>(); Collections.max(l, l);",
            "no suitable method found for max(ArrayList<Integer>,ArrayList<Integer>) in class Collections",
        ),
    ] {
        let text = format!("import java.util.*; class M {{ static void r() {{ {source} }} }}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// `Collections.binarySearch`/`addAll`/`unmodifiableList`/`emptyList`.
/// The unmodifiable list is a *view*: a later change to the backing list shows
/// through, and every mutator throws. Cross-checked against a real JDK by
/// `diff_collections_binary_search_and_add_all` and `_unmodifiable_and_empty`.
#[test]
fn collections_binary_search_add_all_and_unmodifiable() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Arrays;
        import java.util.Collections;
        import java.util.List;

        class Card implements Comparable<Card> {
            int rank;
            Card(int rank) { this.rank = rank; }
            public int compareTo(Card other) { return rank - other.rank; }
            public String toString() { return "C" + rank; }
        }

        public class C {
            public static void main(String[] args) {
                ArrayList<Integer> numbers = new ArrayList<Integer>();
                System.out.println(Collections.addAll(numbers, 1, 3, 5, 7) + " " + numbers);
                System.out.println(Collections.addAll(numbers) + " " + numbers.size());
                System.out.println(Collections.binarySearch(numbers, 5) + " "
                        + Collections.binarySearch(numbers, 4));

                ArrayList<String> words = new ArrayList<String>();
                String[] more = {"g", "i"};
                Collections.addAll(words, "a");
                Collections.addAll(words, more);
                System.out.println(words);

                // binarySearch compares with the element's own compareTo.
                ArrayList<Card> cards = new ArrayList<Card>();
                Collections.addAll(cards, new Card(1), new Card(3));
                System.out.println(Collections.binarySearch(cards, new Card(3)));

                // Arrays.asList keeps its elements unboxed, so they read back.
                List<Integer> boxed = Arrays.asList(1, 2, 3);
                int first = boxed.get(0);
                System.out.println(boxed + " " + first + " " + (boxed.get(1) + 1));

                // An unmodifiable list is a view: it refuses every mutator, and
                // a later change to the backing list shows through.
                List<Integer> readOnly = Collections.unmodifiableList(numbers);
                try { readOnly.add(9); }
                catch (UnsupportedOperationException e) { System.out.println("add"); }
                try { Collections.sort(readOnly); }
                catch (UnsupportedOperationException e) { System.out.println("sort"); }
                try { Collections.reverse(readOnly); }
                catch (UnsupportedOperationException e) { System.out.println("reverse"); }
                numbers.add(11);
                int total = 0;
                for (int value : readOnly) {
                    total += value;
                }
                System.out.println(readOnly + " " + total + " " + readOnly.equals(numbers));

                List<String> empty = Collections.emptyList();
                System.out.println(empty + " " + empty.isEmpty());
                try { empty.add("x"); }
                catch (UnsupportedOperationException e) { System.out.println("empty add"); }
            }
        }
        "#,
        "C",
    );
    assert_eq!(
        out,
        "true [1, 3, 5, 7]\n\
         false 4\n\
         2 -3\n\
         [a, g, i]\n\
         1\n\
         [1, 2, 3] 1 3\n\
         add\n\
         sort\n\
         reverse\n\
         [1, 3, 5, 7, 11] 27 true\n\
         [] true\n\
         empty add\n"
    );
}

/// These four reject what javac rejects. `Collections.addAll(List<Integer>,
/// int[])` is the one to watch: javac has no `Integer[]`/`int[]` conflation,
/// so accepting it would compile here and fail on a JDK.
#[test]
fn collections_search_and_view_reject_like_javac() {
    for (source, want) in [
        (
            "ArrayList<Integer> l = new ArrayList<Integer>(); Collections.addAll(l, \"x\");",
            "incompatible types: String cannot be converted to int",
        ),
        (
            "ArrayList<Integer> l = new ArrayList<Integer>(); int[] a = {1}; Collections.addAll(l, a);",
            "incompatible types: int[] cannot be converted to int",
        ),
        (
            "ArrayList<Integer> l = new ArrayList<Integer>(); int i = Collections.binarySearch(l, \"x\");",
            "incompatible types: String cannot be converted to int",
        ),
        (
            "Collections.unmodifiableList(5);",
            "no suitable method found for unmodifiableList(int) in class Collections",
        ),
    ] {
        let text = format!("import java.util.*; class M {{ static void r() {{ {source} }} }}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// `containsAll`/`removeAll`/`retainAll` compare with the elements' own
/// `equals`, and ask the side Java asks: `containsAll` asks the *other*
/// collection's element, `removeAll` and `retainAll` ask this list's.
/// Cross-checked against a real JDK by `diff_list_contains_remove_retain_all`.
#[test]
fn list_bulk_operations_use_equals() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Collections;
        import java.util.List;

        class Point {
            int x;
            Point(int x) { this.x = x; }
            public boolean equals(Object other) {
                return other instanceof Point && ((Point) other).x == x;
            }
            public int hashCode() { return x; }
            public String toString() { return "P" + x; }
        }

        public class B {
            public static void main(String[] args) {
                ArrayList<Integer> numbers = new ArrayList<Integer>();
                Collections.addAll(numbers, 1, 2, 3, 2);
                ArrayList<Integer> some = new ArrayList<Integer>();
                Collections.addAll(some, 2, 4);
                System.out.println(numbers.containsAll(some) + " " + numbers.containsAll(numbers)
                        + " " + numbers.containsAll(new ArrayList<Integer>()));

                // Every occurrence goes, and the result says whether it changed.
                ArrayList<Integer> removed = new ArrayList<Integer>(numbers);
                System.out.println(removed.removeAll(some) + " " + removed);
                ArrayList<Integer> unchanged = new ArrayList<Integer>(numbers);
                System.out.println(unchanged.removeAll(new ArrayList<Integer>()) + " " + unchanged);
                ArrayList<Integer> retained = new ArrayList<Integer>(numbers);
                System.out.println(retained.retainAll(some) + " " + retained);

                // The elements' own equals decides.
                ArrayList<Point> points = new ArrayList<Point>();
                Collections.addAll(points, new Point(1), new Point(2));
                ArrayList<Point> twos = new ArrayList<Point>();
                Collections.addAll(twos, new Point(2));
                System.out.println(points.containsAll(twos));
                System.out.println(points.removeAll(twos) + " " + points);

                // Nulls compare, and an unmodifiable view refuses to change.
                ArrayList<String> words = new ArrayList<String>();
                Collections.addAll(words, "a", null);
                ArrayList<String> holes = new ArrayList<String>();
                Collections.addAll(holes, (String) null);
                System.out.println(words.containsAll(holes) + " " + words.removeAll(holes) + " " + words);

                List<Integer> readOnly = Collections.unmodifiableList(numbers);
                System.out.println(readOnly.containsAll(some));
                try { readOnly.removeAll(some); }
                catch (UnsupportedOperationException e) { System.out.println("refused"); }
                try { numbers.containsAll(null); }
                catch (NullPointerException e) { System.out.println("null argument"); }
            }
        }
        "#,
        "B",
    );
    assert_eq!(
        out,
        "false true true\n\
         true [1, 3]\n\
         false [1, 2, 3, 2]\n\
         true [2, 2]\n\
         true\n\
         true [P1]\n\
         true true [a]\n\
         false\n\
         refused\n\
         null argument\n"
    );
}

/// A cast of a `null` literal names an overload. It used to fail to parse,
/// while `(x) - 1` must stay a parenthesized expression.
#[test]
fn a_null_literal_can_be_cast() {
    let out = run_stdout(
        r#"
        public class N {
            public static void main(String[] args) {
                String text = (String) null;
                Object thing = (Object) null;
                int x = 5;
                System.out.println(text + " " + (thing == null) + " " + ((x) - 1) + " " + ((x) + 1));
            }
        }
        "#,
        "N",
    );
    assert_eq!(out, "null true 4 6\n");
}

/// `System.arraycopy` over every element kind, including a copy within one
/// array (which behaves as if it went through a temporary) and the component
/// type check that keeps a `boolean[]` out of an `int[]`, even though both
/// hold their elements as 32-bit words. `System.lineSeparator()` is always a
/// newline. Cross-checked against a real JDK by
/// `diff_system_arraycopy_and_line_separator`.
#[test]
fn system_arraycopy_and_line_separator() {
    let out = run_stdout(
        r#"
        import java.util.Arrays;
        public class S {
            public static void main(String[] args) {
                int[] source = {1, 2, 3, 4, 5};
                int[] destination = new int[5];
                System.arraycopy(source, 1, destination, 0, 3);
                System.out.println(Arrays.toString(destination));

                // Overlapping, both directions.
                int[] forward = {1, 2, 3, 4, 5};
                System.arraycopy(forward, 0, forward, 1, 4);
                int[] backward = {1, 2, 3, 4, 5};
                System.arraycopy(backward, 1, backward, 0, 4);
                System.out.println(Arrays.toString(forward) + Arrays.toString(backward));

                String[] words = {"a", "b"};
                String[] copied = new String[3];
                System.arraycopy(words, 0, copied, 1, 2);
                System.out.println(Arrays.toString(copied));

                // The rows of a 2D array are references, so they alias.
                int[][] grid = {{1}, {2}};
                int[][] rows = new int[2][];
                System.arraycopy(grid, 0, rows, 0, 2);
                System.out.println(Arrays.deepToString(rows) + " " + (grid[0] == rows[0]));

                // A zero-length copy at the very end is legal.
                System.arraycopy(source, 5, destination, 5, 0);

                try { System.arraycopy(source, 0, destination, 0, 9); }
                catch (ArrayIndexOutOfBoundsException e) { System.out.println("too long"); }
                try { System.arraycopy(source, 0, destination, 0, -1); }
                catch (ArrayIndexOutOfBoundsException e) { System.out.println("negative"); }
                int[] none = null;
                try { System.arraycopy(none, 0, destination, 0, 1); }
                catch (NullPointerException e) { System.out.println("null"); }
                try { System.arraycopy(new boolean[5], 0, destination, 0, 1); }
                catch (ArrayStoreException e) { System.out.println("boolean into int"); }

                System.out.println(System.lineSeparator().equals("\n")
                        + " " + System.lineSeparator().length());
            }
        }
        "#,
        "S",
    );
    assert_eq!(
        out,
        "[2, 3, 4, 0, 0]\n\
         [1, 1, 2, 3, 4][2, 3, 4, 5, 5]\n\
         [null, a, b]\n\
         [[1], [2]] true\n\
         too long\n\
         negative\n\
         null\n\
         boolean into int\n\
         true 1\n"
    );
}

/// `System.arraycopy` is typed to arrays, where javac takes `Object` and
/// throws `ArrayStoreException` — stricter, so anything compiling here still
/// compiles on a JDK.
#[test]
fn system_arraycopy_rejects_a_non_array() {
    for (source, want) in [
        (
            "int[] d = new int[2]; System.arraycopy(\"a\", 0, d, 0, 1);",
            "no suitable method found for arraycopy(String,int,int[],int,int) in class System",
        ),
        (
            "int[] d = new int[2]; System.arraycopy(d, 0, d, 0);",
            "no suitable method found for arraycopy(int[],int,int[],int) in class System",
        ),
    ] {
        let text = format!("class M {{ static void r() {{ {source} }} }}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// `Math.multiplyFull`, `Math.scalb` (both overloads) and `Random.nextBytes`.
/// `scalb` scales in stages, as the JDK does, so a result that underflows into
/// the subnormals is rounded once rather than twice. Cross-checked against a
/// real JDK by `diff_math_multiply_full_and_scalb` and `diff_random_next_bytes`,
/// which sweep every scale and every seed.
#[test]
fn math_multiply_full_scalb_and_random_next_bytes() {
    let out = run_stdout(
        r#"
        import java.util.Arrays;
        import java.util.Random;
        public class M {
            public static void main(String[] args) {
                System.out.println(Math.multiplyFull(3, 4) + " "
                        + Math.multiplyFull(Integer.MAX_VALUE, Integer.MAX_VALUE) + " "
                        + (Integer.MAX_VALUE * Integer.MAX_VALUE));

                System.out.println(Math.scalb(1.5, 3) + " " + Math.scalb(1.5, -3));
                // One rounding, not two, on the way into the subnormals.
                System.out.println(Math.scalb(1.5, -1074) + " " + Math.scalb(1.0, -1075)
                        + " " + Math.scalb(4.9E-324, 1));
                System.out.println(Math.scalb(1.0, 2000) + " " + Math.scalb(1.0, -2000)
                        + " " + Math.scalb(-0.0, 5) + " " + Math.scalb(Double.NaN, 5));
                System.out.println(Math.scalb(1.5f, 3) + " " + Math.scalb(1.5f, -150)
                        + " " + Math.scalb(1.0f, 300));
                // An int argument picks the more specific float overload.
                System.out.println(Math.scalb(1, 3) == 8.0f);

                byte[] five = new byte[5];
                new Random(42).nextBytes(five);
                System.out.println(Arrays.toString(five));
                // Four bytes per draw, so the generator lands where Java's does.
                Random shared = new Random(99);
                byte[] three = new byte[3];
                shared.nextBytes(three);
                System.out.println(Arrays.toString(three) + " " + shared.nextInt(100));
                byte[] none = new byte[0];
                new Random(1).nextBytes(none);
                System.out.println(Arrays.toString(none));
            }
        }
        "#,
        "M",
    );
    assert_eq!(
        out,
        "12 4611686014132420609 1\n\
         12.0 0.1875\n\
         1.0E-323 0.0 1.0E-323\n\
         Infinity 0.0 -0.0 NaN\n\
         12.0 1.4E-45 Infinity\n\
         true\n\
         [53, -99, 65, -70, -9]\n\
         [118, -6, -14] 58\n\
         []\n"
    );
}

/// These three reject what javac rejects: `multiplyFull` is int-only, `scalb`
/// takes an `int` scale, and `nextBytes` takes a `byte[]`.
#[test]
fn math_and_random_additions_reject_like_javac() {
    for (source, want) in [
        (
            "long x = Math.multiplyFull(1L, 2L);",
            "no suitable method found for multiplyFull(long,long) in class Math",
        ),
        (
            "double x = Math.scalb(1.5, 3.0);",
            "no suitable method found for scalb(double,double) in class Math",
        ),
        (
            "int[] b = new int[2]; new java.util.Random(1).nextBytes(b);",
            "no suitable method found for nextBytes(int[]) in class Random",
        ),
    ] {
        let text = format!("class M {{ static void r() {{ {source} }} }}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// Fully qualified library names in *expression* position, without any
/// import. `java.lang.Math.abs(...)` always worked, but `java.util` did
/// not: `canonical_library_class` kept a second, hand-maintained class
/// list that had drifted from the real one, so `java.util.Arrays.fill`
/// resolved to nothing and the field-access chain blamed a missing
/// variable named `java`. Cross-checked against a real JDK by
/// `diff_fully_qualified_names_in_expression_position`.
#[test]
fn fully_qualified_library_names_resolve_in_expression_position() {
    let out = run_stdout(
        r#"
        public class M {
            public static void main(String[] args) {
                int[] filled = new int[3];
                java.util.Arrays.fill(filled, 7);
                System.out.println(java.util.Arrays.toString(filled));

                java.util.List<Integer> list = new java.util.ArrayList<Integer>();
                list.add(5);
                list.add(2);
                java.util.Collections.sort(list);
                System.out.println(list);
                System.out.println(java.util.Collections.max(list));

                java.util.Random random = new java.util.Random(42);
                System.out.println(random.nextInt(100));

                java.lang.StringBuilder builder = new java.lang.StringBuilder();
                builder.append("ab").append(1);
                System.out.println(builder.reverse());

                System.out.println(java.lang.Math.abs(-5));
                java.lang.System.out.println("qualified");
            }
        }
        "#,
        "M",
    );
    assert_eq!(
        out, "[7, 7, 7]\n[2, 5]\n5\n30\n1ba\n5\nqualified\n",
        "qualified names should behave exactly as the imported ones do"
    );
}

/// A variable named `java` obscures the package (JLS §6.4.2), so the
/// qualified form must lose to it — otherwise the fix above would have
/// silently changed what `java` means.
#[test]
fn a_variable_named_java_obscures_the_package() {
    let out = run_stdout(
        "public class M { public static void main(String[] a) { int java = 3; \
         System.out.println(java); } }",
        "M",
    );
    assert_eq!(out, "3\n");
}

/// A qualified name that does not resolve reports the class and package,
/// as javac does — not the `java` prefix as a missing variable. Pinned
/// against javac by `reject_unknown_class_in_a_known_package` and
/// `reject_unknown_package_in_expression_position`.
#[test]
fn unresolvable_qualified_names_reject_like_javac() {
    for (source, want) in [
        (
            "java.util.Nope.f();",
            "cannot find symbol: class Nope in package java.util",
        ),
        ("java.zzz.Nope.f();", "package java.zzz does not exist"),
    ] {
        let text = format!("class M {{ static void r() {{ {source} }} }}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// A real Java 11 class caturra does not model says so by name, wherever it
/// is written. Until 2026-07-09 only `import` and `new` gave the honest
/// reason: a declaration said "this type cannot be used for a variable", a
/// field "unknown type for field 'items'", and `extends` "cannot find
/// symbol" — all of which read as a typo for a class the student can see in
/// the documentation. javac accepts every program below, so caturra is
/// deliberately strict here (`strict_linked_list_is_refused_by_name`).
#[test]
fn unmodeled_library_classes_explain_themselves_in_every_position() {
    let reason = "is not supported by caturra";
    for (label, source) in [
        (
            "local",
            "class M { static void r() { Vector<Integer> l; } }",
        ),
        ("raw local", "class M { static void r() { Vector l; } }"),
        ("field", "class M { Vector<Integer> items; }"),
        (
            "parameter",
            "class M { static void f(Vector<Integer> l) {} }",
        ),
        ("array", "class M { static void r() { Vector[] l; } }"),
        (
            "new",
            "class M { static void r() { Object o = new Vector<Integer>(); } }",
        ),
        ("extends", "class D extends Vector {} class M {}"),
        ("implements", "class D implements Iterator {} class M {}"),
        (
            "type argument",
            "class M { static void r() { ArrayList<Vector> l; } }",
        ),
        (
            "qualified",
            "class M { static void r() { java.util.Hashtable<Integer, Integer> m; } }",
        ),
    ] {
        let text = format!("import java.util.*;\n{source}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {label}");
        let found = compilation
            .diagnostics
            .iter()
            .any(|d| d.message.contains(reason));
        assert!(
            found,
            "{label}: expected an honest reason, got: {:?}",
            compilation
                .diagnostics
                .iter()
                .map(|d| &d.message)
                .collect::<Vec<_>>()
        );
    }
}

/// The honest reason must not swallow a genuine typo, and a user class of the
/// same name still shadows the library one.
#[test]
fn the_honest_reason_does_not_hide_a_typo_or_shadow_a_user_class() {
    for (source, want) in [
        (
            "class M { static void r() { Frobnicator f; } }",
            "unknown type 'Frobnicator'",
        ),
        (
            "class M { static void r() { Frobnicator<Integer> f; } }",
            "unknown type 'Frobnicator'",
        ),
        // The base is modeled; the argument is the typo.
        (
            "class M { static void r() { ArrayList<Frobnicator> l; } }",
            "unknown type 'Frobnicator'",
        ),
        (
            "class D extends Nope {} class M {}",
            "cannot find symbol: class Nope",
        ),
    ] {
        let text = format!("import java.util.*;\n{source}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// A user class named `Stack` shadows the library one caturra does not model,
/// so it compiles and runs — the honest reason is only sound once the name
/// has failed to resolve.
#[test]
fn a_user_class_shadows_an_unmodeled_library_name() {
    let out = run_stdout(
        "import java.util.*;\n\
         class Stack { int depth; }\n\
         public class M { public static void main(String[] a) { \
         Stack s = new Stack(); s.depth = 4; System.out.println(s.depth); } }",
        "M",
    );
    assert_eq!(out, "4\n");
}

/// Printing or concatenating an array gives `Object.toString()`, exactly as a
/// real JVM does: the class descriptor, `@`, and the identity hash in hex.
/// Useless to read, and precisely what a student sees on a JDK — until
/// 2026-07-09 caturra refused to compile it at all. The hash is the heap
/// reference, so only its shape is asserted here; byte-for-byte agreement
/// with a real JDK is pinned by `diff_array_default_to_string`.
#[test]
fn printing_an_array_gives_javas_default_to_string() {
    let out = run_stdout(
        r#"
        public class M {
            public static void main(String[] args) {
                int[] ints = {1, 2, 3};
                String[] strings = {"x"};
                int[][] grid = {{1}};
                System.out.println(head("" + ints));
                System.out.println(head("" + strings));
                System.out.println(head("" + grid));
                System.out.println(head("" + grid[0]));
                System.out.println(head("" + new boolean[1]));
                System.out.println(head(ints.toString()));
                System.out.println(ints.getClass().getName());
                System.out.println(strings.getClass().getName());
            }
            static String head(String s) {
                int at = s.indexOf("@");
                if (at < 0) return "NO-AT:" + s;
                String hex = s.substring(at + 1);
                boolean ok = hex.length() > 0;
                for (int i = 0; i < hex.length(); i++) {
                    char c = hex.charAt(i);
                    if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) ok = false;
                }
                return s.substring(0, at) + "@" + (ok ? "<hex>" : "BAD");
            }
        }
        "#,
        "M",
    );
    assert_eq!(
        out,
        "[I@<hex>\n\
         [Ljava.lang.String;@<hex>\n\
         [[I@<hex>\n\
         [I@<hex>\n\
         [Z@<hex>\n\
         [I@<hex>\n\
         [I\n\
         [Ljava.lang.String;\n"
    );
}

/// `println(char[])` is a real overload that prints the CHARACTERS, while
/// concatenation is `String.valueOf(Object)` and prints `[C@hash`. A classic
/// Java trap, reproduced rather than smoothed over.
#[test]
fn println_of_a_char_array_prints_its_characters_but_concatenation_does_not() {
    let out = run_stdout(
        r#"
        public class M {
            public static void main(String[] args) {
                char[] chars = {'h', 'i'};
                System.out.println(chars);
                System.out.print(chars);
                System.out.println();
                String concatenated = "" + chars;
                System.out.println(concatenated.startsWith("[C@"));
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "hi\nhi\ntrue\n");
}

/// `type_of` used to answer `Error` for an `Object` method on an array, while
/// the emitter happily emitted the call. A statement whose type is `Error` is
/// dropped, so `len(ints.toString())` compiled to nothing and printed nothing
/// — silently. The two must agree.
#[test]
fn an_object_method_on_an_array_types_as_the_emitter_emits_it() {
    let out = run_stdout(
        r"
        public class M {
            static int len(String s) { return s.length(); }
            static boolean same(Object a) { return true; }
            public static void main(String[] args) {
                int[] ints = {1, 2};
                System.out.println(len(ints.toString()) > 0);
                System.out.println(len(ints.getClass().getName()));
                System.out.println(ints.hashCode() == ints.hashCode());
                System.out.println(ints.equals(ints));
            }
        }
        ",
        "M",
    );
    assert_eq!(out, "true\n2\ntrue\ntrue\n");
}

/// A subclass may hide a superclass field of the same name (JLS §8.3). The two
/// are DISTINCT slots, and which one an access means is decided by the static
/// type at the access site, not by the object's runtime class — hiding is not
/// overriding. caturra rejected this outright until 2026-07-09, and had to,
/// because the heap keyed instance fields by name alone and the two slots
/// silently merged. Cross-checked against a real JDK by `diff_field_hiding`.
#[test]
fn a_subclass_field_hides_rather_than_overrides() {
    let out = run_stdout(
        r#"
        class Sup {
            private String tag = "sup";
            protected int n = 1;
            public String read() { return tag + "/" + n; }
            public int getN() { return n; }
            public void bump() { n = n + 10; }
        }
        class Sub extends Sup {
            private int tag = 42;
            protected int n = 2;
            public String read2() { return tag + "/" + n; }
            public void bumpSub() { n = n + 100; }
        }
        public class M {
            public static void main(String[] args) {
                Sub sub = new Sub();
                Sup up = sub;
                System.out.println(sub.read());
                System.out.println(sub.read2());
                System.out.println(sub.getN());
                System.out.println(up.n + " " + sub.n + " " + ((Sup) sub).n);
                sub.bump();
                sub.bumpSub();
                System.out.println(sub.read() + " " + sub.read2());
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "sup/1\n42/2\n1\n1 2 1\nsup/11 42/102\n");
}

/// The corpus shape: a private field hidden by one of a DIFFERENT type. A
/// private field is not inherited at all, so this is the easy half of hiding
/// — but it still needs two slots. Reflection reads the slot of the class
/// that declared the `Field`. Pinned by `diff_field_hiding_with_reflection`.
#[test]
fn a_hidden_private_field_of_another_type_keeps_its_own_slot() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        class Base {
            private ArrayList<String> data;
            Base() { data = new ArrayList<String>(); data.add("base"); }
            public String show() { return data.toString(); }
        }
        class Derived extends Base {
            private String data;
            Derived() { super(); data = "derived"; }
            public String show2() { return data; }
        }
        public class M {
            public static void main(String[] args) {
                Derived d = new Derived();
                System.out.println(d.show());
                System.out.println(d.show2());
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "[base]\nderived\n");
}

/// The debugger names a field by its declaring class only when it has to:
/// a hidden name would otherwise appear twice with nothing to tell the two
/// apart. Unhidden fields keep the plain name the student wrote.
#[test]
fn the_debugger_qualifies_a_hidden_field_and_only_a_hidden_field() {
    let source = "class Sup {\n\
        \x20   protected int n = 1;\n\
        \x20   protected int only = 7;\n\
        }\n\
        class Sub extends Sup {\n\
        \x20   protected int n = 2;\n\
        }\n\
        public class Dbg {\n\
        \x20   public static void main(String[] args) {\n\
        \x20       Sub sub = new Sub();\n\
        \x20       System.out.println(sub.n);\n\
        \x20   }\n\
        }\n";
    let (host, _stdout, result) = debug_run(
        source,
        "Dbg",
        &[("Dbg.java", 11)],
        vec![DebugCommand::Continue],
    );
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    let top = &host.pauses[0].frames[0];
    let sub = top
        .locals
        .iter()
        .find(|l| l.name == "sub")
        .map(|l| l.value.clone())
        .expect("local sub");
    assert!(
        sub.contains("Sub.n=2") && sub.contains("Sup.n=1"),
        "a hidden field is qualified by its declaring class: {sub}"
    );
    assert!(
        sub.contains("only=7") && !sub.contains("Sup.only"),
        "an unhidden field keeps its plain name: {sub}"
    );
}

/// `super.field` reads and writes the field the SUPERCLASS sees. Fields are
/// not virtual, so that is all it means: the receiver is `this`, and only the
/// lookup moves up. Supported since 2026-07-09 — the parser used to refuse it
/// outright, and could not have done otherwise while the heap merged a hidden
/// field with the one it hid. Pinned against a real JDK by
/// `diff_super_field_access`.
#[test]
fn super_field_reads_and_writes_the_superclasss_slot() {
    let out = run_stdout(
        r#"
        class A { protected int n = 1; protected String s = "a"; }
        class B extends A {
            int n = 2;
            int viaSuper() { return super.n; }
            void setSuper(int v) { super.n = v; }
            void bumpSuper() { super.n += 5; super.n++; }
            String cat() { return super.s + super.n; }
            int all() { return super.n + this.n + n; }
            int lenOfSuperField() { return super.s.length(); }
        }
        class C extends B { int n = 3; int cSuper() { return super.n; } }
        public class M {
            public static void main(String[] args) {
                B b = new B();
                System.out.println(b.viaSuper() + " " + b.n);
                b.setSuper(9);
                System.out.println(b.viaSuper() + " " + b.n);
                b.bumpSuper();
                System.out.println(b.viaSuper());
                System.out.println(b.cat());
                System.out.println(b.all());
                System.out.println(b.lenOfSuperField());
                C c = new C();
                System.out.println(c.cSuper() + " " + c.n + " " + ((A) c).n);
            }
        }
        "#,
        "M",
    );
    // `super.n` in C is B's `n` (2), not A's (1): the lookup starts at the
    // superclass and walks up only if it must.
    assert_eq!(out, "1 2\n9 2\n15\na15\n19\n1\n2 3 1\n");
}

/// The three ways `super.field` is refused, in javac's wording. Pinned against
/// a real javac by `reject_super_field_*` and `reject_wording_tracks_javac`.
#[test]
fn super_field_rejects_like_javac() {
    for (source, want) in [
        (
            "class A { private int n = 1; } class B extends A { int f() { return super.n; } }",
            "n has private access in A",
        ),
        (
            "class A { protected int n = 1; } \
             class B extends A { static int f() { return super.n; } }",
            "non-static variable super cannot be referenced from a static context",
        ),
        (
            "class B { int n = 1; int f() { return super.n; } }",
            "cannot find symbol: field 'n' in class Object",
        ),
        (
            "class A { int q = 1; } class B extends A { int f() { return super.zz; } }",
            "cannot find symbol: field 'zz' in class A",
        ),
    ] {
        let text = format!("{source} class M {{ static void r() {{}} }}");
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text,
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// An empty statement (`;`, JLS §14.6) parses to nothing and must not disturb
/// the statements around it. Until 2026-07-09 `block_body` treated it exactly
/// like a parse error and ran `recover_to_statement_boundary`, which skips to
/// the next `;` — so a stray semicolon **silently deleted the statement after
/// it**, with no diagnostic. `int x = 1;; total += x;` compiled, ran, and
/// quietly dropped the `total += x`. Found by diffing 2254 corpus programs
/// against a real JDK; it was the sole cause of every divergence. Pinned by
/// `diff_empty_statement`.
#[test]
fn a_stray_semicolon_does_not_swallow_the_next_statement() {
    let out = run_stdout(
        r#"
        public class M {
            public static void main(String[] args) {
                int total = 0;
                for (int i = 0; i < 3; i++) {
                    int x = i + 1;;
                    total += x;
                }
                System.out.println(total);
                int y = 5;;
                System.out.println(y);
                ;
                System.out.println("lone");
                ;;;
                System.out.println("three");
                if (total > 0) {
                    ;
                    System.out.println("in if");
                }
                int count = 0;
                do { count++;; } while (count < 2);
                System.out.println(count);
                for (int i = 0; i < 0; i++);
                System.out.println("end");
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "6\n5\nlone\nthree\nin if\n2\nend\n");
}

/// The recovery an empty statement must not trigger is still reached by a real
/// parse error, and still resynchronises at the next statement boundary — so
/// one bad statement yields one diagnostic, not a cascade.
#[test]
fn a_parse_error_still_recovers_at_the_statement_boundary() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("M.java"),
        text: String::from(
            "public class M { public static void main(String[] a) { \
             int x = ; int y = 2; System.out.println(y); } }",
        ),
    }]);
    assert!(
        !compilation.success(),
        "a missing expression should not compile"
    );
    let errors: Vec<&String> = compilation
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, caturra_compiler::diagnostics::Severity::Error))
        .map(|d| &d.message)
        .collect();
    assert_eq!(errors.len(), 1, "one error, not a cascade: {errors:?}");
    assert!(
        errors[0].contains("expected an expression"),
        "got: {}",
        errors[0]
    );
}

/// `Scanner.close()` on a `System.in` scanner closes the stream, exactly as
/// the JDK's does: the closed Scanner refuses every method but `close`, and a
/// *new* `Scanner(System.in)` then sees end of input. Until 2026-07-09
/// caturra's `close()` was a no-op, so a program that closed stdin and read
/// again worked in the playground and died with `NoSuchElementException` on a
/// real JVM — the one place caturra was more permissive at run time. Found by
/// diffing 2254 corpus solutions against a real JDK. Pinned by
/// `diff_scanner_close_closes_standard_in`.
#[test]
fn closing_a_standard_in_scanner_closes_standard_in() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("In.java"),
        text: String::from(
            r#"
            import java.util.NoSuchElementException;
            import java.util.Scanner;
            public class In {
                public static void main(String[] args) {
                    Scanner first = new Scanner(System.in);
                    System.out.println(first.next());
                    first.close();
                    try { first.next(); System.out.println("no throw"); }
                    catch (IllegalStateException e) { System.out.println("ISE " + e.getMessage()); }
                    first.close();
                    Scanner second = new Scanner(System.in);
                    System.out.println("hasNext=" + second.hasNext());
                    try { second.next(); System.out.println("no throw"); }
                    catch (NoSuchElementException e) { System.out.println("NSE"); }
                }
            }
            "#,
        ),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    let mut console = BufferedConsole::with_input(["alpha beta", "gamma"]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("In", &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "alpha\nISE Scanner closed\nhasNext=false\nNSE\n"
    );
}

/// Closing a *file* Scanner closes only that Scanner. Standard input is a
/// different stream and must survive — the bundled libraries read files with
/// a Scanner and close it, and would otherwise take stdin down with them.
#[test]
fn closing_a_file_scanner_leaves_standard_in_open() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("In.java"),
        text: String::from(
            r#"
            import java.io.File;
            import java.io.FileNotFoundException;
            import java.util.Scanner;
            public class In {
                public static void main(String[] args) throws FileNotFoundException {
                    Scanner file = new Scanner(new File("d.txt"));
                    System.out.println(file.next());
                    file.close();
                    try { file.next(); System.out.println("no throw"); }
                    catch (IllegalStateException e) { System.out.println("ISE " + e.getMessage()); }
                    Scanner again = new Scanner(new File("d.txt"));
                    System.out.println(again.next());
                    again.close();
                    Scanner in = new Scanner(System.in);
                    System.out.println(in.next());
                }
            }
            "#,
        ),
    }]);
    assert!(compilation.success(), "{:?}", compilation.diagnostics);
    let mut vfs = VirtualFileSystem::new();
    vfs.write_file("d.txt", b"inFile more".to_vec()).unwrap();
    let mut console = BufferedConsole::with_input(["fromStdin"]);
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).unwrap();
    }
    let result = vm.run_main("In", &[]);
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    assert_eq!(
        console.stdout_text(),
        "inFile\nISE Scanner closed\ninFile\nfromStdin\n"
    );
}

/// `obj.staticMethod(...)` — legal if discouraged, and javac only warns under
/// `-Xlint:static`. caturra rejected it outright until 2026-07-09, though it
/// already allowed `obj.staticField`. The receiver expression is evaluated for
/// its side effects and then discarded, so a `null` receiver does not throw:
/// nothing is dereferenced. Pinned against a real JDK by
/// `diff_static_method_through_an_instance`.
#[test]
fn a_static_method_can_be_called_through_an_instance() {
    let out = run_stdout(
        r#"
        class H {
            static int calls = 3;
            static int twice(int x) { return 2 * x; }
            static void hello() { System.out.println("hello"); }
        }
        public class M {
            static H make() { System.out.println("make() ran"); return new H(); }
            public static void main(String[] args) {
                H h = new H();
                System.out.println(h.twice(4));
                h.hello();
                System.out.println(h.calls);
                System.out.println(make().twice(3));
                H none = null;
                System.out.println(none.twice(7));
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "8\nhello\n3\nmake() ran\n6\n14\n");
}

/// A static method inherited from a superclass resolves through it, whether
/// named by the subclass or reached through a subclass instance (JVMS §5.4.3.3
/// searches the named class, then its superclasses). `Derived.who()` used to
/// compile and then die with `MalformedClass: no static method who()` —
/// `invokestatic` demanded the method be declared in the class the ref named.
#[test]
fn an_inherited_static_method_resolves_through_the_superclass() {
    let out = run_stdout(
        r#"
        class Base { static String who() { return "Base"; } }
        class Derived extends Base {}
        public class M {
            public static void main(String[] args) {
                System.out.println(Derived.who());
                Derived d = new Derived();
                System.out.println(d.who());
                Base b = d;
                System.out.println(b.who());
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "Base\nBase\nBase\n");
}

/// JLS §8.4.8: an interface's static method is **not** inherited by the
/// classes that implement it. Only the interface names it. caturra used to
/// compile `C.hi()` and crash at run time. Its default methods are inherited,
/// so the rule must not reach them. Pinned by
/// `reject_static_interface_method_through_a_class` and
/// `diff_static_interface_method_through_the_interface`.
#[test]
fn an_interface_static_method_is_not_inherited_but_a_default_method_is() {
    let out = run_stdout(
        r#"
        interface Greeter {
            static String hi() { return "hi"; }
            default String greet() { return hi() + " " + name(); }
            String name();
        }
        class Person implements Greeter { public String name() { return "ada"; } }
        public class M {
            public static void main(String[] args) {
                System.out.println(Greeter.hi());
                System.out.println(new Person().greet());
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "hi\nhi ada\n");

    for source in [
        "interface I { static String hi() { return \"h\"; } void go(); } \
         class C implements I { public void go() {} } \
         class M { static void r() { C.hi(); } }",
        "interface I { static String hi() { return \"h\"; } void go(); } \
         class C implements I { public void go() {} } \
         class M { static void r() { new C().hi(); } }",
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        assert!(
            compilation.diagnostics[0]
                .message
                .contains("cannot find symbol"),
            "got: {}",
            compilation.diagnostics[0].message
        );
    }
}

/// JLS §7.6: a public top-level type must be declared in a file named after
/// it, and javac enforces it. caturra ignored the rule until 2026-07-09, so a
/// program that broke it compiled in the playground and failed on a real JDK.
/// Only 29 of 6682 corpus levels violate it, and every one of those already
/// fails on a JDK. Pinned against a real javac by
/// `reject_public_class_in_a_mismatched_file` and the wording table.
#[test]
fn a_public_top_level_type_must_match_its_file_name() {
    for (path, source, want) in [
        (
            "Foo.java",
            "public class Bar {}",
            "class Bar is public, should be declared in a file named Bar.java",
        ),
        (
            "Foo.java",
            "public interface Baz { void go(); }",
            "interface Baz is public, should be declared in a file named Baz.java",
        ),
        (
            "Foo.java",
            "public enum Qux { A }",
            "enum Qux is public, should be declared in a file named Qux.java",
        ),
        (
            "Foo.java",
            "public abstract class Zap {}",
            "class Zap is public, should be declared in a file named Zap.java",
        ),
        // Only one public top-level type per file, and it must be the one
        // the file is named for.
        (
            "Foo.java",
            "public class Foo {} public class Other {}",
            "class Other is public, should be declared in a file named Other.java",
        ),
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from(path),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// The rule binds only *public* *top-level* types. Package-private classes,
/// interfaces and enums may share any file — most caturra tests rely on it —
/// and a `public` nested type is not top-level. Pinned by
/// `diff_package_private_classes_share_a_file`.
#[test]
fn the_file_name_rule_spares_package_private_and_nested_types() {
    let out = run_stdout(
        r#"
        class Helper { static int twice(int x) { return 2 * x; } }
        interface Named { String name(); }
        enum Colour { RED, GREEN }
        public class M {
            public static class Inner { static String hi() { return "inner"; } }
            public static void main(String[] args) {
                System.out.println(Helper.twice(21));
                System.out.println(Colour.GREEN);
                System.out.println(Inner.hi());
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "42\nGREEN\ninner\n");
}

/// `map.forEach((k, v) -> ...)` (2026-07-09), the last construct the Code.org
/// corpus needed. The lambda's parameter types come from the RECEIVER's
/// declared type arguments — no other target type in caturra is instantiated
/// that way — and the synthesized class implements the bundled erased
/// `__BiConsumer`, opening with the two casts javac would put in a bridge
/// method. The VM walks the entries in the map's iteration order and calls
/// `accept` through the same nested-call machinery `compareTo` uses. Pinned
/// against a real JDK by `diff_map_for_each_lambda`.
#[test]
fn map_for_each_runs_a_lambda_over_every_entry() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.HashMap;
        import java.util.Map;
        public class M {
            private HashMap<Double, String> vocab = new HashMap<Double, String>();
            ArrayList<String> between(double low, double high) {
                ArrayList<String> found = new ArrayList<String>();
                vocab.forEach((key, value) -> {
                    if (key > low && key < high) { found.add(value); }
                });
                return found;
            }
            public static void main(String[] args) {
                M it = new M();
                it.vocab.put(1.1, "alpha");
                it.vocab.put(2.2, "beta");
                it.vocab.put(3.3, "gamma");
                System.out.println(it.between(1.0, 3.0));

                Map<String, Integer> counts = new HashMap<String, Integer>();
                counts.put("a", 1);
                ArrayList<String> out = new ArrayList<String>();
                counts.forEach((k, n) -> out.add(k + "=" + (n + 1)));
                System.out.println(out);

                new HashMap<String, String>().forEach((k, v) -> System.out.println("never"));
                System.out.println("done");
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "[alpha, beta]\n[a=2]\ndone\n");
}

/// A lambda is hoisted to a top-level class, which loses sight of the
/// enclosing class's STATIC fields. Java resolves them through the enclosing
/// class, so caturra does too — and they are shared state, not captured by
/// value: a later write is visible inside, and the lambda's writes are visible
/// outside. Until 2026-07-09 this was `cannot find variable`. Pinned by
/// `diff_lambda_reads_enclosing_static_field`.
#[test]
fn a_lambda_shares_the_enclosing_classs_static_fields() {
    let out = run_stdout(
        r#"
        interface Action { void go(); }
        public class M {
            static int hits = 0;
            public static void main(String[] args) {
                Action read = () -> System.out.println("saw " + hits);
                read.go();
                hits = 7;
                read.go();
                Action write = () -> { hits = hits + 10; };
                write.go();
                System.out.println(hits);
                write.go();
                System.out.println(hits);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "saw 0\nsaw 7\n17\n27\n");
}

/// The enclosing-static fallback had to be added to `type_of` as well as to
/// the emitter. With only the emitter taught, `hits = hits + 10` typed its
/// right-hand side as `Error`, which silently dropped the read: the lambda
/// stored `10` every time instead of accumulating. `type_of` and the emitter
/// must agree — the fourth time that has bitten this codebase.
#[test]
fn type_of_agrees_with_the_emitter_on_an_enclosing_static_field() {
    let out = run_stdout(
        r#"
        interface Action { void go(); }
        public class M {
            static int total = 5;
            static String tag = "t";
            public static void main(String[] args) {
                Action a = () -> { total = total * 2; tag = tag + "!"; };
                a.go();
                a.go();
                System.out.println(total + tag);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "20t!!\n");
}

/// `forEach` is refused where javac refuses it, and where caturra cannot read
/// the receiver's type arguments it says so rather than guessing.
#[test]
fn map_for_each_rejects_what_it_cannot_target_type() {
    for (source, want) in [
        // javac: `int cannot be converted to BiConsumer`.
        (
            "import java.util.*; class M { static void r() { \
             Map<String, Integer> m = new HashMap<String, Integer>(); m.forEach(5); } }",
            "no suitable method found for forEach",
        ),
        // javac: `incompatible parameter types in lambda expression`.
        (
            "import java.util.*; class M { static void r() { \
             Map<String, Integer> m = new HashMap<String, Integer>(); \
             m.forEach(k -> System.out.println(k)); } }",
            "only allowed where a functional-interface type is expected",
        ),
        // A receiver with no declaration to read: javac accepts, caturra does
        // not — the safe direction, and it says which construct is missing.
        (
            "import java.util.*; class M { static Map<String, Integer> get() { return null; } \
             static void r() { get().forEach((k, v) -> System.out.println(k)); } }",
            "only allowed where a functional-interface type is expected",
        ),
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// JLS §4.12.4: a local variable referenced from a lambda or inner class must
/// be final or effectively final. caturra copies a capture into a synthetic
/// field, which would silently hide a later write — so it accepted a lambda
/// javac rejects, the last place lambdas were more permissive than a JDK.
/// Now refused, in javac's wording. Pinned against a real javac by
/// `reject_lambda_captures_a_reassigned_local` and friends.
#[test]
fn a_lambda_may_not_capture_a_non_effectively_final_local() {
    for (source, want) in [
        (
            "interface Fn { int go(); } class M { \
             static Fn r() { int c = 0; Fn f = () -> c; c = 5; return f; } }",
            "referenced from a lambda expression must be final or effectively final",
        ),
        (
            "interface Fn { int go(); } class M { \
             static Fn r(int p) { p = 1; return () -> p; } }",
            "referenced from a lambda expression must be final or effectively final",
        ),
        (
            "interface Fn { int go(); } class M { \
             static void r() { for (int i = 0; i < 2; i++) { Fn f = () -> i; } } }",
            "referenced from a lambda expression must be final or effectively final",
        ),
        (
            "abstract class A { abstract int go(); } class M { \
             static A r() { int c = 0; A a = new A() { int go() { return c; } }; c = 5; return a; } }",
            "referenced from an inner class must be final or effectively final",
        ),
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        let message = &compilation.diagnostics[0].message;
        assert!(message.contains(want), "expected {want:?}, got: {message}");
    }
}

/// An effectively-final capture still compiles and runs: an initializer never
/// reassigned, a blank local assigned once, and an untouched parameter are all
/// captured by value. Pinned by `diff_effectively_final_capture_is_accepted`.
#[test]
fn an_effectively_final_local_is_captured_normally() {
    let out = run_stdout(
        r"
        interface IntFn { int go(); }
        public class M {
            static IntFn withParam(int p) { return () -> p * 10; }
            public static void main(String[] args) {
                int base = 10;
                IntFn a = () -> base + 1;
                System.out.println(a.go());
                int once;
                once = 7;
                IntFn b = () -> once * 2;
                System.out.println(b.go());
                System.out.println(withParam(5).go());
            }
        }
        ",
        "M",
    );
    assert_eq!(out, "11\n14\n50\n");
}

/// An array is a reference and widens to `Object` — `(Object) intArray` and
/// passing an array to an `Object` parameter. A safe upcast, no runtime
/// check. caturra rejected the explicit cast until 2026-07-09 ("int[] cannot
/// be converted to Object"), the safe direction but valid Java. Pinned by
/// `diff_array_widens_to_object`.
#[test]
fn an_array_widens_to_object() {
    let out = run_stdout(
        r#"
        public class M {
            static boolean isNull(Object o) { return o == null; }
            public static void main(String[] args) {
                int[] ints = {1, 2, 3};
                Object o = (Object) ints;
                System.out.println(o == ints);
                System.out.println(isNull(ints));
                String[] strings = {"a"};
                Object o2 = (Object) strings;
                System.out.println(o2 == strings);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "true\nfalse\ntrue\n");
}

/// A lambda in an instance method may read, write, or call the enclosing
/// instance's members — `() -> field`, `() -> this.m()`, `() -> field = x`,
/// `() -> helper()`. Java captures the enclosing `this` (its `this$0`); caturra
/// captures it as a synthetic `__caturraOuter` field and resolves those
/// references through it, live on the real object. Until 2026-07-09 any such
/// reference was "cannot find variable" — the safe direction, but the most
/// common lambda shape in real OO Java. Pinned against a real JDK by
/// `diff_lambda_captures_enclosing_instance`.
#[test]
fn a_lambda_reaches_the_enclosing_instance() {
    let out = run_stdout(
        r"
        interface IntFn { int go(); }
        interface Run { void go(); }
        public class M {
            int field = 100;
            int twice() { return field * 2; }
            IntFn read() { return () -> field; }
            IntFn callMethod() { return () -> twice(); }
            Run write() { return () -> { field = field + 10; }; }
            int withLocal(int param) {
                int local = 3;
                IntFn a = () -> field + local + param;
                return a.go();
            }
            public static void main(String[] args) {
                M m = new M();
                System.out.println(m.read().go());
                System.out.println(m.callMethod().go());
                m.write().go();
                System.out.println(m.field);
                System.out.println(m.withLocal(10));
            }
        }
        ",
        "M",
    );
    assert_eq!(out, "100\n200\n110\n123\n");
}

/// The capture is scoped to a lambda directly in an instance method. A nested
/// lambda reaching an instance field two levels up needs transitive capture,
/// which is unsupported — a compile error (javac accepts it, so the safe
/// direction) rather than a wrong answer.
#[test]
fn a_nested_lambda_reaching_an_instance_field_is_a_clean_error() {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("M.java"),
        text: String::from(
            "interface Fn { int go(); } class M { int field = 1; \
             int r() { Fn outer = () -> { Fn inner = () -> field; return inner.go(); }; \
             return outer.go(); } }",
        ),
    }]);
    assert!(!compilation.success(), "nested capture is not supported");
    // A diagnostic, not a panic or bad bytecode.
    assert!(
        compilation
            .diagnostics
            .iter()
            .any(|d| matches!(d.severity, caturra_compiler::diagnostics::Severity::Error)),
        "expected a compile error"
    );
}

/// A lambda (or method reference) passed to `list.add(...)` / `list.set(...)`
/// is target-typed against the receiver's element type, not a user method
/// signature. `add`/`set` take the element last. Until 2026-07-09 caturra only
/// target-typed a lambda against a single-candidate user method, so a lambda
/// element was "only allowed where a functional-interface type is expected".
/// Pinned against a real JDK by `diff_lambda_as_a_list_element`.
#[test]
fn a_lambda_is_target_typed_as_a_list_element() {
    let out = run_stdout(
        r"
        import java.util.ArrayList;
        import java.util.List;
        interface IntFn { int go(); }
        public class M {
            public static void main(String[] args) {
                ArrayList<IntFn> l = new ArrayList<IntFn>();
                l.add(() -> 7);
                l.add(() -> 8);
                l.add(1, () -> 9);
                l.set(0, () -> 5);
                int sum = 0;
                for (int i = 0; i < l.size(); i++) sum += l.get(i).go();
                System.out.println(sum);
                int base = 100;
                List<IntFn> m = new ArrayList<IntFn>();
                m.add(() -> base + 1);
                System.out.println(m.get(0).go());
            }
        }
        ",
        "M",
    );
    assert_eq!(out, "22\n101\n");
}

/// The element target-typing fires only where javac would accept it. A
/// non-functional element type, `contains` (which takes `Object`), and a
/// parameter-arity mismatch all stay rejected — the safe direction.
#[test]
fn a_lambda_list_element_is_refused_where_javac_refuses_it() {
    for source in [
        // Element type is not a functional interface.
        "import java.util.*; class M { static void r() { \
         ArrayList<String> l = new ArrayList<String>(); l.add(() -> \"x\"); } }",
        // `contains` takes Object, not the element type — no target typing.
        "import java.util.*; interface Fn { int go(); } class M { static void r() { \
         ArrayList<Fn> l = new ArrayList<Fn>(); l.contains(() -> 1); } }",
        // The lambda's shape does not match the SAM.
        "import java.util.*; interface Two { int go(int x); } class M { static void r() { \
         ArrayList<Two> l = new ArrayList<Two>(); l.add(() -> 1); } }",
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
    }
}

/// `(int[]) obj` and `(String[]) obj` — an array downcast. The primitive-array
/// form (`(int[])`, `(int[][])`) did not even parse until 2026-07-09
/// ("expected '.class' after a primitive type"), and reference-array casts
/// parsed but were rejected in codegen. Both now emit a runtime `checkcast`.
/// Pinned against a real JDK by `diff_array_downcast`.
#[test]
fn an_object_downcasts_to_an_array_type() {
    let out = run_stdout(
        r#"
        public class M {
            public static void main(String[] args) {
                Object oi = new int[] {1, 2, 3};
                System.out.println(((int[]) oi)[2]);
                Object os = new String[] {"x", "y"};
                System.out.println(((String[]) os)[1]);
                Object og = new int[2][3];
                int[][] g = (int[][]) og;
                System.out.println(g.length + " " + g[0].length);
                Object on = null;
                System.out.println(((int[]) on) == null);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "3\ny\n2 3\ntrue\n");
}

/// The array checkcast throws `ClassCastException` exactly where the JVM does:
/// a wrong element type, a non-array object, a primitive-array mismatch, and a
/// primitive array cast to `Object[]`. A reference array (and a nested array)
/// casts to `Object[]` and succeeds. Pinned against a real JDK by
/// `diff_array_downcast`.
#[test]
fn a_bad_array_downcast_throws_class_cast_exception() {
    let out = run_stdout(
        r#"
        public class M {
            public static void main(String[] args) {
                Object anInt = new int[] {1};
                Object aStr = new String[] {"a"};
                Object nested = new int[2][2];
                Object plain = "hello";
                report("wrongElem", anInt, 0);
                report("nonArray", plain, 0);
                report("primInvariant", anInt, 1);
                report("refToObjectArr", aStr, 2);
                report("primToObjectArr", anInt, 2);
                report("nestedToObjectArr", nested, 2);
            }
            static void report(String label, Object o, int kind) {
                try {
                    if (kind == 0) { String[] x = (String[]) o; }
                    else if (kind == 1) { long[] x = (long[]) o; }
                    else { Object[] x = (Object[]) o; }
                    System.out.println(label + ": ok");
                } catch (ClassCastException e) {
                    System.out.println(label + ": CCE");
                }
            }
        }
        "#,
        "M",
    );
    assert_eq!(
        out,
        "wrongElem: CCE\nnonArray: CCE\nprimInvariant: CCE\n\
         refToObjectArr: ok\nprimToObjectArr: CCE\nnestedToObjectArr: ok\n"
    );
}

/// `list.forEach(x -> ...)` (a `Consumer<E>`) and `list.removeIf(x -> ...)` (a
/// `Predicate<E>`) work (2026-07-09): the lambda's parameter is the receiver's
/// element type, and the erased SAM is the bundled `__Consumer`/`__Predicate`.
/// Until then both reported "lambdas are not supported". Pinned against a real
/// JDK by `diff_list_for_each_and_remove_if`.
#[test]
fn a_list_for_each_and_remove_if_run_lambdas() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.List;
        public class M {
            public static void main(String[] args) {
                List<Integer> nums = new ArrayList<Integer>();
                for (int i = 1; i <= 5; i++) nums.add(i);
                int[] total = {0};
                nums.forEach(n -> { total[0] += n; });
                System.out.println(total[0]);
                boolean removed = nums.removeIf(n -> n % 2 == 0);
                System.out.println(nums + " " + removed);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "15\n[1, 3, 5] true\n");
}

/// The list-lambda target-typing fires only where javac accepts it: a
/// two-parameter lambda to `forEach`, a non-boolean `removeIf`, and a
/// non-lambda argument all stay rejected, and `replaceAll` is still an honest
/// "not supported".
#[test]
fn a_list_lambda_is_refused_where_javac_refuses_it() {
    for source in [
        "import java.util.*; class M { static void r() { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.forEach((x, y) -> x); } }",
        "import java.util.*; class M { static void r() { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.removeIf(n -> n + 1); } }",
        "import java.util.*; class M { static void r() { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.forEach(5); } }",
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
    }
}

/// `list.replaceAll(x -> ...)` (a `UnaryOperator<E>`) applies the operator to
/// each element in place (2026-07-09). The erased SAM returns `Object`, so a
/// primitive result comes back boxed and is unboxed to match the list's
/// storage; the body's result is coerced to the element type, so a wrong
/// return is rejected. Pinned against a real JDK by `diff_list_replace_all`.
#[test]
fn a_list_replace_all_transforms_each_element() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        public class M {
            public static void main(String[] args) {
                ArrayList<Integer> nums = new ArrayList<Integer>();
                nums.add(1); nums.add(2); nums.add(3);
                nums.replaceAll(n -> n * 10);
                System.out.println(nums);
                int sum = 0;
                for (int i = 0; i < nums.size(); i++) sum += nums.get(i);
                System.out.println(sum);
                ArrayList<String> w = new ArrayList<String>();
                w.add("a"); w.add("bb");
                w.replaceAll(s -> s.toUpperCase() + s.length());
                System.out.println(w);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "[10, 20, 30]\n60\n[A1, BB2]\n");
}

/// `replaceAll` keeps the element-type check javac makes, even though the SAM
/// erases to `Object` — a result of the wrong type is rejected, in an
/// expression body and a block body alike. The safe direction.
#[test]
fn a_replace_all_result_must_convert_to_the_element_type() {
    for source in [
        "import java.util.*; class M { static void r() { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.replaceAll(n -> \"x\"); } }",
        "import java.util.*; class M { static void r() { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.replaceAll(n -> { return \"x\"; }); } }",
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
        assert!(
            compilation.diagnostics[0]
                .message
                .contains("cannot be converted to Integer"),
            "got: {}",
            compilation.diagnostics[0].message
        );
    }
}

/// `list.sort((a, b) -> ...)` and `Collections.sort(list, (a, b) -> ...)` sort
/// by a comparator (2026-07-09). The lambda's two parameters are the element
/// type and it returns int; the erased SAM is the bundled `__Comparator`, and
/// the VM runs a stable merge sort calling `compare`. `Collections.sort(l,
/// cmp)` previously compiled and **silently ignored** the comparator, the
/// worst kind of divergence. Pinned against a real JDK by
/// `diff_sort_with_comparator`.
#[test]
fn a_list_sorts_by_a_comparator_lambda() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.Collections;
        import java.util.List;
        public class M {
            public static void main(String[] args) {
                List<Integer> nums = new ArrayList<Integer>();
                nums.add(3); nums.add(1); nums.add(2);
                nums.sort((x, y) -> x - y);
                System.out.println(nums);
                nums.sort((x, y) -> y - x);
                System.out.println(nums);
                List<String> w = new ArrayList<String>();
                w.add("ccc"); w.add("a"); w.add("bb");
                Collections.sort(w, (x, y) -> x.length() - y.length());
                System.out.println(w);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "[1, 2, 3]\n[3, 2, 1]\n[a, bb, ccc]\n");
}

/// The comparator target-typing fires only where javac accepts it: a
/// one-parameter lambda and a non-int result are both rejected, and
/// `Collections.sort(l)` and `list` natural-ordering keep working.
#[test]
fn a_sort_comparator_is_refused_where_javac_refuses_it() {
    for source in [
        "import java.util.*; class M { static void r() { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.sort(n -> n); } }",
        "import java.util.*; class M { static void r() { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.sort((x, y) -> \"z\"); } }",
    ] {
        let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
            path: String::from("M.java"),
            text: String::from(source),
        }]);
        assert!(!compilation.success(), "should not compile: {source}");
    }
    // Natural ordering (no comparator) still sorts.
    let out = run_stdout(
        "import java.util.*; public class M { public static void main(String[] a) { \
         ArrayList<Integer> l = new ArrayList<Integer>(); l.add(3); l.add(1); l.add(2); \
         Collections.sort(l); System.out.println(l); } }",
        "M",
    );
    assert_eq!(out, "[1, 2, 3]\n");
}

/// A lambda expression body may be a statement expression — an assignment or
/// `++`/`--` — not only a value expression. `n -> sum[0] += n`, `() ->
/// count++`, `() -> x = 5` all work (2026-07-09); they were parse errors
/// before. A value body (`x -> x + 1`) is unaffected. Pinned against a real
/// JDK by `diff_lambda_statement_expression_body`.
#[test]
fn a_lambda_body_may_be_a_statement_expression() {
    let out = run_stdout(
        r"
        import java.util.ArrayList;
        import java.util.List;
        interface Run { void go(); }
        public class M {
            static int c = 0;
            public static void main(String[] args) {
                int[] sum = {0};
                List<Integer> nums = new ArrayList<Integer>();
                nums.add(1); nums.add(2); nums.add(3);
                nums.forEach(n -> sum[0] += n);
                System.out.println(sum[0]);
                Run inc = () -> c++;
                inc.go(); inc.go();
                System.out.println(c);
                Run set = () -> c = 50;
                set.go();
                System.out.println(c);
            }
        }
        ",
        "M",
    );
    assert_eq!(out, "6\n2\n50\n");
}

/// `map.keySet().forEach(...)` and `map.values().forEach(...)` — and a `Set`/
/// `Collection` variable holding such a view — run a `Consumer` lambda over
/// the view's elements in the map's iteration order (2026-07-09). The lambda's
/// parameter is typed from the map's key or value type. `entrySet().forEach`
/// (a `Map.Entry` parameter) remains a compile error, the safe direction.
/// Pinned against a real JDK by `diff_map_view_for_each`.
#[test]
fn a_map_view_runs_for_each() {
    let out = run_stdout(
        r#"
        import java.util.Collection;
        import java.util.HashMap;
        import java.util.Map;
        import java.util.Set;
        public class M {
            public static void main(String[] args) {
                Map<Integer, String> m = new HashMap<Integer, String>();
                m.put(1, "a"); m.put(2, "b"); m.put(3, "c");
                int[] keySum = {0};
                m.keySet().forEach(k -> keySum[0] += k);
                System.out.println(keySum[0]);
                int[] valLen = {0};
                Collection<String> vals = m.values();
                vals.forEach(v -> valLen[0] += v.length());
                System.out.println(valLen[0]);
                Set<Integer> keys = m.keySet();
                int[] count = {0};
                keys.forEach(k -> count[0]++);
                System.out.println(count[0]);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "6\n3\n3\n");
}

/// `StringBuilder` works as a field, parameter, return type, and captured
/// local — `push_type` (the JVM descriptor builder) handled `String`/`Scanner`
/// but not `StringBuilder`, so any of those gave "unknown type '`StringBuilder`'"
/// (2026-07-09). This unblocks the common `list.forEach(x -> sb.append(x))`
/// pattern. Pinned against a real JDK by `diff_string_builder_as_a_type`.
#[test]
fn string_builder_works_as_a_type_everywhere() {
    let out = run_stdout(
        r#"
        import java.util.ArrayList;
        import java.util.List;
        interface Run { void go(); }
        public class M {
            StringBuilder field = new StringBuilder();
            static StringBuilder twice(StringBuilder s) { return s.append(s); }
            public static void main(String[] args) {
                StringBuilder sb = new StringBuilder();
                Run r = () -> sb.append("x");
                r.go(); r.go();
                System.out.println(sb);
                StringBuilder acc = new StringBuilder();
                List<String> w = new ArrayList<String>();
                w.add("a"); w.add("b");
                w.forEach(s -> acc.append(s));
                System.out.println(acc);
                M obj = new M();
                obj.field.append("f");
                System.out.println(obj.field);
            }
        }
        "#,
        "M",
    );
    assert_eq!(out, "xx\nab\nf\n");
}
