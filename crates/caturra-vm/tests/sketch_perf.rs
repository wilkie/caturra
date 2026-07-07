//! Ad-hoc performance probe for the Swing "sketch pad" pattern: a drag
//! listener that appends a point and repaints, so paintComponent redraws a
//! growing polyline every tick. Run with:
//!   cargo test -p caturra-vm --test `sketch_perf` -- --nocapture --ignored
//!
//! It compares three paintComponent bodies at increasing point counts to
//! attribute the per-tick cost:
//!   B  empty paint          -> dispatch + append + base serialize
//!   C  iterate, draw nothing -> + O(N) list iteration per tick
//!   A  draw a line per point -> + command building + __joined/__esc serialize
//! The total-time scaling ratio (t(2N)/t(N)) reveals the exponent: ~2 linear,
//! ~4 quadratic, ~8 cubic.

use std::time::Instant;

use caturra_vm::{ExitStatus, VirtualFileSystem, Vm, VmOptions};

/// A console that feeds `n` drag events at the panel (c1) with distinct
/// coordinates, then ends the session. It also records the serialized tree
/// length of the final tick, so we can see the payload growth.
struct DragConsole {
    remaining: u32,
    step: u32,
    last_tree_len: usize,
}

impl caturra_vm::ConsoleIo for DragConsole {
    fn stdout(&mut self, _bytes: &[u8]) {}
    fn stderr(&mut self, _bytes: &[u8]) {}
    fn read_line(&mut self) -> Option<String> {
        None
    }
    fn ui_await_event(&mut self, tree: &str) -> Option<String> {
        self.last_tree_len = tree.len();
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        self.step += 1;
        // Wander around the canvas so successive points differ.
        let x = 20 + (self.step * 7) % 360;
        let y = 20 + (self.step * 13) % 260;
        Some(format!("c1\n__drag={x},{y}"))
    }
}

fn sketch_source(paint_body: &str) -> String {
    format!(
        r#"
        import javax.swing.*;
        import java.awt.*;
        import java.awt.event.*;
        public class Main {{
            static java.util.ArrayList<Integer> xs = new java.util.ArrayList<Integer>();
            static java.util.ArrayList<Integer> ys = new java.util.ArrayList<Integer>();
            static int sink = 0;
            static SketchPanel panel;
            public static void main(String[] args) {{
                JFrame frame = new JFrame("Sketch");
                panel = new SketchPanel();
                panel.setPreferredSize(new Dimension(400, 300));
                panel.addMouseMotionListener(new MouseAdapter() {{
                    public void mouseDragged(MouseEvent e) {{
                        Main.xs.add(e.getX());
                        Main.ys.add(e.getY());
                        Main.panel.repaint();
                    }}
                }});
                frame.add(panel);
                frame.setVisible(true);
            }}
        }}
        class SketchPanel extends JPanel {{
            public void paintComponent(Graphics g) {{
                {paint_body}
            }}
        }}
        "#
    )
}

const PAINT_EMPTY: &str = "";
const PAINT_ITERATE: &str = r"
    int s = 0;
    for (int i = 1; i < Main.xs.size(); i++) {
        s = s + Main.xs.get(i) + Main.ys.get(i);
    }
    Main.sink = s;
";
const PAINT_FULL: &str = r"
    for (int i = 1; i < Main.xs.size(); i++) {
        g.drawLine(Main.xs.get(i - 1), Main.ys.get(i - 1), Main.xs.get(i), Main.ys.get(i));
    }
";

/// Draw `points` points and return (`elapsed_ms`, final tree length).
fn run_sketch(paint_body: &str, points: u32) -> (f64, usize) {
    let source = sketch_source(paint_body);
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: String::from("Main.java"),
        text: source,
    }]);
    assert!(
        compilation.success(),
        "compile failed: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    let mut console = DragConsole {
        remaining: points,
        step: 0,
        last_tree_len: 0,
    };
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("load");
    }
    let start = Instant::now();
    let result = vm.run_main("Main", &[]);
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    assert!(matches!(result, Ok(ExitStatus::Completed)), "{result:?}");
    (elapsed, console.last_tree_len)
}

#[test]
#[ignore = "performance probe; run explicitly with --nocapture --ignored"]
fn sketch_pad_scaling() {
    // DELIBERATELY TINY. The full-draw variant allocates ~O(N^3) u16s into a
    // heap that never frees, so even N=400 tries for ~9 GB and the OOM killer
    // takes down the whole session. N<=80 stays under ~80 MB. Do not raise
    // these without capping heap growth first (that is itself the finding).
    let sizes = [20u32, 40, 80];
    let variants: [(&str, &str); 3] = [
        ("B empty-paint  ", PAINT_EMPTY),
        ("C iterate-only ", PAINT_ITERATE),
        ("A full-draw    ", PAINT_FULL),
    ];
    // Warm up (caches, first-run costs).
    run_sketch(PAINT_FULL, 50);

    println!("\nsketch pad: total time to draw N points (drag ticks)\n");
    for (label, body) in variants {
        let mut prev: Option<(u32, f64)> = None;
        for n in sizes {
            let (ms, tree_len) = run_sketch(body, n);
            let ratio = prev.map_or(String::from("   -"), |(pn, pms)| {
                format!("{:.1}x", (ms / pms) / (f64::from(n) / f64::from(pn)) * 2.0)
            });
            // `ratio` is t(2N)/t(N) normalized so linear≈2.0, quad≈4.0, cubic≈8.0.
            let raw_ratio = prev.map_or(0.0, |(_, pms)| ms / pms);
            println!(
                "  {label}  N={n:>5}  {ms:>9.2} ms  tree={tree_len:>8}B  \
                 t(2N)/t(N)={raw_ratio:>4.1}  [{ratio}]"
            );
            prev = Some((n, ms));
        }
        println!();
    }
}
