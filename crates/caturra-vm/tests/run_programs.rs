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
        json.contains(r#""type":"textfield","text":"","columns":12,"id":"c2""#),
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
    assert!(json.contains("fillArc 120 40 60 60 30 220"), "no arc: {json}");
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
    assert!(json.contains(r#""type":"toolbarsep""#), "no separator: {json}");
    assert!(json.contains(r#""type":"button""#), "no toolbar buttons: {json}");
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
    assert!(json.contains(r#""left":{"type":"label""#), "no left component: {json}");
    assert!(json.contains(r#""right":{"type":"button""#), "no right component: {json}");
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
    assert!(json.contains(r#""type":"tabbedpane","placement":1,"selectedIndex":1"#), "no tabbedpane: {json}");
    assert!(json.contains(r#""title":"Alpha","component":{"type":"label""#), "no alpha tab: {json}");
    assert!(json.contains(r#""title":"Beta","component":{"type":"button""#), "no beta tab: {json}");
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
        json.contains(r#""border":{"type":"matte","thickness":1,"color":"220,140,40","insets":"4,0,4,0"}"#),
        "no matte border: {json}"
    );
    assert!(
        json.contains(r#""border":{"type":"bevel","thickness":2,"insets":"0,0,0,0","raised":false}"#),
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
    assert!(json.contains(r#""font":"1 22 Serif""#), "no widget font: {json}");
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
    // and list equality dispatching a user equals override.
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
    assert_eq!(out, "[1, 2, 3]\n[fig, kiwi, pear]\nfig\nfalse true\n");
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
        public class Main {
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
        public class Main {
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
        public class Main {
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
        public class Main {
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
        public class Main {
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
            "class A { int x; } class B extends A { int x; }",
            "hiding the inherited field 'x' from A is not supported",
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
            "import java.util.HashMap; class M { }",
            "java.util.HashMap is not supported by caturra (the class library covers the AP CS A subset)",
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
            path: "Program.java".into(),
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
            "import java.util.ArrayList; class M { static void f() { ArrayList<Integer> a = new ArrayList<>(); a.sort(null); } }",
            "ArrayList.sort exists in Java, but comparators are not supported by caturra",
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
