//! Differential tests for `org.code.neighborhood`: run the same Painter program
//! through the **real Code.org library** and through caturra, and require
//! identical stdout.
//!
//! The companion of `differential_media.rs`, against the same vendored
//! `javabuilder/` checkout. `Painter` is the most queryable surface caturra
//! models — `getX`, `getDirection`, `canMove`, `isOnPaint`, `getMyPaint`,
//! `getColor` — so almost all of it can be pinned by printing.
//!
//! The real package needs more scaffolding than `media` did: the session
//! protocol (`Painter` fires a UI message on every action), and `org.json`
//! (its `grid.txt` is JSON). Both are shimmed below — the output adapter to a
//! no-op, since nothing here reads the UI stream, and `org.json` to a small
//! recursive-descent parser over the array/object/int subset `GridFactory`
//! uses. `ClientMessageDetailKeys` is dependency-free, so the real one is
//! compiled rather than faked.
//!
//! The `org.json` shim is the one hand-written link in the chain, so it was
//! checked against the genuine article: the real package was built twice, once
//! against this shim and once against `org.json:json:20180813`, and both
//! produced identical grids — for the fixtures below and for a ragged grid, a
//! non-JSON file and an empty array, the malformed cases where two parsers are
//! likeliest to part company. (Real jars are not a dependency of this repo, so
//! the shim is what ships.)
//!
//! **The grid is expressed twice, on purpose.** `grid.txt` is written by the
//! host, never by a student: Code.org's is JSON, caturra's is rows of
//! `tileType,paintCount`. That is an internal contract on each side, not a
//! semantic, so a fixture below is one *world* rendered into each engine's
//! own encoding. What is compared is what the Painter does in it.
//!
//! Skips (with a note) when the checkout or a JDK is absent; set
//! `CATURRA_REQUIRE_JAVABUILDER=1` to make that a failure instead.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

/// The session protocol, reduced to what `Painter`, `World` and the grid types
/// actually touch. `Painter` sends a message per action for the UI to animate;
/// nothing here reads that stream, so the adapter swallows them.
const PROTOCOL_SHIM: &[(&str, &str)] = &[
    (
        "JavabuilderRuntimeException.java",
        r"package org.code.protocol;

public abstract class JavabuilderRuntimeException extends RuntimeException {
  protected JavabuilderRuntimeException(Enum key) { super(key.toString()); }
  protected JavabuilderRuntimeException(Enum key, Throwable cause) { super(key.toString(), cause); }
}
",
    ),
    (
        "InternalExceptionKey.java",
        r"package org.code.protocol;

public enum InternalExceptionKey { INTERNAL_EXCEPTION }
",
    ),
    (
        "InternalServerRuntimeException.java",
        r"package org.code.protocol;

public class InternalServerRuntimeException extends JavabuilderRuntimeException {
  public InternalServerRuntimeException(InternalExceptionKey key) { super(key); }
  public InternalServerRuntimeException(InternalExceptionKey key, Throwable cause) { super(key, cause); }
}
",
    ),
    (
        "ClientMessageType.java",
        r"package org.code.protocol;

public enum ClientMessageType { NEIGHBORHOOD, THEATER, SYSTEM }
",
    ),
    (
        "ClientMessage.java",
        r"package org.code.protocol;

import java.util.HashMap;

public abstract class ClientMessage {
  private final ClientMessageType type;
  private final String value;

  protected ClientMessage(ClientMessageType type, String value, HashMap<String, String> detail) {
    this.type = type;
    this.value = value;
  }

  protected ClientMessage(ClientMessageType type, String value) { this(type, value, null); }

  public ClientMessageType getType() { return this.type; }

  public String getValue() { return this.value; }

  public boolean shouldAlwaysSend() { return true; }
}
",
    ),
    (
        "OutputAdapter.java",
        r"package org.code.protocol;

public interface OutputAdapter {
  void sendMessage(ClientMessage message);
}
",
    ),
    (
        "JavabuilderSharedObject.java",
        r"package org.code.protocol;

public abstract class JavabuilderSharedObject {}
",
    ),
    (
        "GlobalProtocol.java",
        r"package org.code.protocol;

public class GlobalProtocol {
  private final OutputAdapter outputAdapter = message -> {};

  public OutputAdapter getOutputAdapter() { return this.outputAdapter; }
}
",
    ),
    (
        "JavabuilderContext.java",
        r"package org.code.protocol;

import java.util.HashMap;

public class JavabuilderContext {
  private static final JavabuilderContext INSTANCE = new JavabuilderContext();
  private final GlobalProtocol protocol = new GlobalProtocol();
  private final HashMap<Class, JavabuilderSharedObject> objects = new HashMap<>();

  public static JavabuilderContext getInstance() { return INSTANCE; }

  public GlobalProtocol getGlobalProtocol() { return this.protocol; }

  public JavabuilderSharedObject get(Class key) { return this.objects.get(key); }

  public void register(Class key, JavabuilderSharedObject value) { this.objects.put(key, value); }
}
",
    ),
];

/// `org.json`, reduced to the array/object/number subset `GridFactory` reads.
/// The grid fixtures below are written by this file, so what this parser has to
/// cope with is exactly what this file emits.
const JSON_SHIM: &[(&str, &str)] = &[
    (
        "JSONException.java",
        r"package org.json;

public class JSONException extends RuntimeException {
  public JSONException(String message) { super(message); }
}
",
    ),
    (
        "JSONTokener.java",
        r#"package org.json;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/** Recursive descent over arrays, objects, strings, ints and null. */
final class JSONTokener {
  private final String text;
  private int pos;

  JSONTokener(String text) { this.text = text; this.pos = 0; }

  Object parse() {
    skipWhitespace();
    Object value = parseValue();
    skipWhitespace();
    if (this.pos != this.text.length()) { throw new JSONException("trailing content"); }
    return value;
  }

  private void skipWhitespace() {
    while (this.pos < this.text.length() && Character.isWhitespace(this.text.charAt(this.pos))) {
      this.pos++;
    }
  }

  private char peek() {
    if (this.pos >= this.text.length()) { throw new JSONException("unexpected end of input"); }
    return this.text.charAt(this.pos);
  }

  private Object parseValue() {
    char c = peek();
    if (c == '[') { return parseArray(); }
    if (c == '{') { return parseObject(); }
    if (c == '"') { return parseString(); }
    return parseLiteral();
  }

  private List<Object> parseArray() {
    List<Object> values = new ArrayList<>();
    this.pos++;
    skipWhitespace();
    if (peek() == ']') { this.pos++; return values; }
    while (true) {
      skipWhitespace();
      values.add(parseValue());
      skipWhitespace();
      char c = peek();
      this.pos++;
      if (c == ']') { return values; }
      if (c != ',') { throw new JSONException("expected , or ]"); }
    }
  }

  private Map<String, Object> parseObject() {
    Map<String, Object> entries = new HashMap<>();
    this.pos++;
    skipWhitespace();
    if (peek() == '}') { this.pos++; return entries; }
    while (true) {
      skipWhitespace();
      String key = parseString();
      skipWhitespace();
      if (peek() != ':') { throw new JSONException("expected :"); }
      this.pos++;
      skipWhitespace();
      entries.put(key, parseValue());
      skipWhitespace();
      char c = peek();
      this.pos++;
      if (c == '}') { return entries; }
      if (c != ',') { throw new JSONException("expected , or }"); }
    }
  }

  private String parseString() {
    if (peek() != '"') { throw new JSONException("expected a string"); }
    this.pos++;
    StringBuilder text = new StringBuilder();
    while (true) {
      char c = peek();
      this.pos++;
      if (c == '"') { return text.toString(); }
      text.append(c);
    }
  }

  private Object parseLiteral() {
    int start = this.pos;
    while (this.pos < this.text.length() && ",]} \t\r\n".indexOf(this.text.charAt(this.pos)) < 0) {
      this.pos++;
    }
    String token = this.text.substring(start, this.pos);
    if (token.equals("null")) { return JSONObject.NULL; }
    if (token.equals("true")) { return Boolean.TRUE; }
    if (token.equals("false")) { return Boolean.FALSE; }
    try {
      return Integer.valueOf(token);
    } catch (NumberFormatException notANumber) {
      throw new JSONException("bad literal '" + token + "'");
    }
  }
}
"#,
    ),
    (
        "JSONArray.java",
        r#"package org.json;

import java.util.List;
import java.util.Map;

public class JSONArray {
  private final List<Object> values;

  @SuppressWarnings("unchecked")
  public JSONArray(String source) {
    Object parsed = new JSONTokener(source).parse();
    if (!(parsed instanceof List)) { throw new JSONException("not a JSON array"); }
    this.values = (List<Object>) parsed;
  }

  JSONArray(List<Object> values) { this.values = values; }

  public int length() { return this.values.size(); }

  @SuppressWarnings("unchecked")
  public Object get(int index) {
    if (index < 0 || index >= this.values.size()) { throw new JSONException("index out of range"); }
    Object value = this.values.get(index);
    if (value instanceof List) { return new JSONArray((List<Object>) value); }
    if (value instanceof Map) { return new JSONObject((Map<String, Object>) value); }
    return value;
  }
}
"#,
    ),
    (
        "JSONObject.java",
        r#"package org.json;

import java.util.Map;

public class JSONObject {
  /** The sentinel org.json uses for a JSON null. */
  public static final Object NULL = new Object() {
    @Override public String toString() { return "null"; }
    @Override public boolean equals(Object other) { return other == this || other == null; }
    @Override public int hashCode() { return 0; }
  };

  private final Map<String, Object> entries;

  JSONObject(Map<String, Object> entries) { this.entries = entries; }

  public boolean has(String key) { return this.entries.containsKey(key); }

  public boolean isNull(String key) {
    Object value = this.entries.get(key);
    return value == null || NULL.equals(value);
  }

  public Object get(String key) {
    if (!this.entries.containsKey(key)) { throw new JSONException("no such key: " + key); }
    return this.entries.get(key);
  }
}
"#,
    ),
];

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

fn neighborhood_package() -> Option<PathBuf> {
    let package = workspace_root()
        .join("javabuilder/org-code-javabuilder/neighborhood/src/main/java/org/code/neighborhood");
    package.join("Painter.java").is_file().then_some(package)
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
    let available = jdk_available() && neighborhood_package().is_some();
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

/// Compile the shims and the real neighborhood package once per test binary.
fn reference_classes() -> &'static Path {
    static CLASSES: OnceLock<PathBuf> = OnceLock::new();
    CLASSES.get_or_init(|| {
        let package = neighborhood_package().expect("caller checked the checkout is present");
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("org-code-neighborhood");
        let classes = root.join("classes");
        std::fs::create_dir_all(&classes).expect("create classes dir");

        let mut sources: Vec<PathBuf> = Vec::new();
        for (package_dir, shim) in [("org/code/protocol", PROTOCOL_SHIM), ("org/json", JSON_SHIM)] {
            let dir = root.join("shim").join(package_dir);
            std::fs::create_dir_all(&dir).expect("create shim dir");
            for (name, text) in shim {
                let path = dir.join(name);
                std::fs::write(&path, text).expect("write shim source");
                sources.push(path);
            }
        }
        // Dependency-free, so the real one compiles as-is rather than being faked.
        sources.push(workspace_root().join(
            "javabuilder/org-code-javabuilder/protocol/src/main/java/org/code/protocol/ClientMessageDetailKeys.java",
        ));
        for directory in [package.clone(), package.join("support")] {
            let entries = std::fs::read_dir(&directory).expect("read the real sources");
            sources.extend(entries.filter_map(|entry| {
                let path = entry.ok()?.path();
                (path.extension()? == "java").then_some(path)
            }));
        }

        let compile = Command::new("javac")
            .arg("-d")
            .arg(&classes)
            .args(&sources)
            .output()
            .expect("javac runs");
        assert!(
            compile.status.success(),
            "the real org.code.neighborhood did not compile against the shims — \
             its dependencies have moved: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        classes
    })
}

/// One world, in caturra's `grid.txt` encoding: rows of `tileType,paintCount`.
/// The reference's JSON is derived from it, so a fixture cannot describe two
/// different worlds by accident.
fn grid_as_json(grid: &str) -> String {
    let rows: Vec<String> = grid
        .trim()
        .lines()
        .map(|line| {
            let squares: Vec<String> = line
                .split_whitespace()
                .map(|cell| {
                    let (tile, paint) = cell.split_once(',').unwrap_or((cell, "0"));
                    format!(r#"{{"tileType":{tile},"value":{paint}}}"#)
                })
                .collect();
            format!("[{}]", squares.join(","))
        })
        .collect();
    format!("[{}]", rows.join(","))
}

/// Run `source` against the real library, in a directory holding `grid.txt`.
fn run_with_reference(class_name: &str, source: &str, grid: &str) -> String {
    let classes = reference_classes();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&(source, grid), &mut hasher);
    let fingerprint = std::hash::Hasher::finish(&hasher);
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("nbh-{class_name}-{fingerprint:x}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("grid.txt"), grid_as_json(grid)).expect("write grid");
    let java_file = dir.join(format!("{class_name}.java"));
    let mut file = std::fs::File::create(&java_file).expect("create source file");
    file.write_all(source.as_bytes()).expect("write source");
    drop(file);

    let compile = Command::new("javac")
        .arg("-cp")
        .arg(classes)
        .arg(java_file.file_name().expect("file name"))
        .current_dir(&dir)
        .output()
        .expect("javac runs");
    assert!(
        compile.status.success(),
        "javac rejected {class_name} against the real library: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new("java")
        .arg("-cp")
        .arg(format!(".:{}", classes.display()))
        .arg(class_name)
        .current_dir(&dir)
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "the real library failed for {class_name}: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).into_owned()
}

fn run_with_caturra(class_name: &str, source: &str, grid: &str) -> String {
    let compilation = caturra_compiler::compile(&[caturra_compiler::SourceFile {
        path: format!("{class_name}.java"),
        text: source.to_owned(),
    }]);
    assert!(
        compilation.success(),
        "caturra rejected {class_name}: {:?}",
        compilation.diagnostics
    );
    let mut vfs = VirtualFileSystem::new();
    vfs.write_file("grid.txt", grid.trim().to_owned())
        .expect("write grid");
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("class loads");
    }
    let result = vm.run_main(class_name, &[]);
    drop(vm);
    assert!(
        result.is_ok(),
        "caturra run failed for {class_name}: {result:?}; stderr: {}",
        console.stderr_text()
    );
    console.stdout_text()
}

fn assert_same_output(class_name: &str, source: &str, grid: &str) {
    let expected = run_with_reference(class_name, source, grid);
    let actual = run_with_caturra(class_name, source, grid);
    assert_eq!(
        actual, expected,
        "caturra's org.code.neighborhood diverges from the real Code.org \
         library for {class_name}"
    );
}

macro_rules! neighborhood_differential_test {
    ($name:ident, $class:literal, $grid:expr, $source:literal) => {
        #[test]
        fn $name() {
            if !reference_available() {
                return;
            }
            assert_same_output($class, $source, $grid);
        }
    };
}

/// 4x4, all open, with a bucket of 3 paint at (1,1) and a wall at (2,0).
/// (`tileType` 0 is a wall, 1 open, 2 start, 3 finish, 4 obstacle, 5 both.)
const SMALL: &str = "\
1,0 1,0 0,0 1,0
1,0 1,3 1,0 1,0
1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0";

/// The same 4x4, with an obstacle (also impassable) and a finish square.
const MIXED: &str = "\
1,0 4,0 1,0 3,0
1,0 1,2 1,0 1,0
5,0 1,0 0,0 1,0
1,0 1,0 1,0 2,0";

// Movement, turning and the compass: where the painter is, which way it faces,
// and what it may walk into. canMove sees walls, obstacles and the grid's edge.
neighborhood_differential_test!(
    nbh_moves_turns_and_looks,
    "NbhMove",
    SMALL,
    r#"
import org.code.neighborhood.*;

public class NbhMove {
    static void where(Painter p) {
        System.out.println(p.getX() + "," + p.getY() + " facing " + p.getDirection()
            + " north=" + p.isFacingNorth() + " east=" + p.isFacingEast()
            + " south=" + p.isFacingSouth() + " west=" + p.isFacingWest());
    }

    public static void main(String[] args) {
        Painter p = new Painter(0, 0, "East", 5);
        where(p);

        // The wall at (2,0) and the grid's edge are both impassable.
        System.out.println(p.canMove("east") + " " + p.canMove("north") + " "
            + p.canMove("west") + " " + p.canMove("south"));

        p.move();
        where(p);
        System.out.println("can go east into the wall? " + p.canMove());

        p.turnLeft();
        where(p);
        p.turnLeft();
        p.turnLeft();
        where(p);
        p.move();
        where(p);

        // Walking off the edge throws, and leaves the painter where it was.
        Painter edge = new Painter(0, 0, "North", 0);
        try {
            edge.move();
            System.out.println("no throw");
        } catch (RuntimeException e) {
            System.out.println("blocked");
        }
        where(edge);
    }
}
"#
);

// Paint: what the painter carries, what it puts down, and what it picks up.
// A bucket square holds paint; a painted square reports its colour.
neighborhood_differential_test!(
    nbh_paint_and_buckets,
    "NbhPaint",
    SMALL,
    r#"
import org.code.neighborhood.*;

public class NbhPaint {
    public static void main(String[] args) {
        Painter p = new Painter(0, 0, "South", 2);
        System.out.println("paint=" + p.getMyPaint() + " has=" + p.hasPaint());

        p.paint("red");
        System.out.println("onPaint=" + p.isOnPaint() + " color=" + p.getColor()
            + " left=" + p.getMyPaint());

        // Scraping takes it back off the square.
        p.scrapePaint();
        System.out.println("after scrape onPaint=" + p.isOnPaint());

        // The bucket at (1,1): stand on it and take paint from it. Facing
        // South, one left turn faces East.
        p.move();
        p.turnLeft();
        p.move();
        System.out.println("at " + p.getX() + "," + p.getY() + " bucket=" + p.isOnBucket());
        p.takePaint();
        p.takePaint();
        System.out.println("carrying " + p.getMyPaint() + " still a bucket? " + p.isOnBucket());
        p.takePaint();
        System.out.println("carrying " + p.getMyPaint() + " bucket now? " + p.isOnBucket());

        // Taking from an empty bucket, and painting with an empty painter,
        // both print rather than throw.
        p.takePaint();
        Painter dry = new Painter(3, 3, "North", 0);
        dry.paint("blue");
        System.out.println("dry painted? " + dry.isOnPaint() + " has=" + dry.hasPaint());
    }
}
"#
);

// The tile types that are not plain open squares, and the paint counts a level
// seeds them with.
neighborhood_differential_test!(
    nbh_tile_types,
    "NbhTiles",
    MIXED,
    r#"
import org.code.neighborhood.*;

public class NbhTiles {
    public static void main(String[] args) {
        // (1,0) is an obstacle and (2,2) a wall: neither is passable. A start
        // (2,0 here is open), finish and start-and-finish square all are.
        Painter p = new Painter(0, 0, "East", 0);
        System.out.println("into the obstacle? " + p.canMove("east"));
        System.out.println("south onto open? " + p.canMove("south"));

        Painter finish = new Painter(3, 0, "South", 0);
        System.out.println("finish square is where the painter stands: "
            + finish.getX() + "," + finish.getY());

        Painter both = new Painter(0, 2, "East", 0);
        System.out.println("start-and-finish is passable: " + both.canMove("north"));

        // The bucket the fixture seeds at (1,1).
        Painter onBucket = new Painter(1, 1, "North", 0);
        System.out.println("bucket=" + onBucket.isOnBucket() + " paint=" + onBucket.getMyPaint());
        onBucket.takePaint();
        onBucket.takePaint();
        System.out.println("emptied: bucket=" + onBucket.isOnBucket()
            + " carrying=" + onBucket.getMyPaint());
    }
}
"#
);

// A default `new Painter()` starts at (0,0) facing East with no paint — and gets
// INFINITE paint, but only on a grid of 20 or more a side. On a small grid it is
// simply a painter with nothing to paint with.
neighborhood_differential_test!(
    nbh_default_painter_has_no_paint_on_a_small_grid,
    "NbhDefaultSmall",
    SMALL,
    r#"
import org.code.neighborhood.*;

public class NbhDefaultSmall {
    public static void main(String[] args) {
        Painter p = new Painter();
        System.out.println(p.getX() + "," + p.getY() + " " + p.getDirection()
            + " paint=" + p.getMyPaint() + " has=" + p.hasPaint());
        p.paint("red");
        System.out.println("painted? " + p.isOnPaint());
    }
}
"#
);

// The failure surface a student actually hits: a bad colour, a wall, a painter
// placed off the grid. What is thrown, and what it says.
neighborhood_differential_test!(
    nbh_errors,
    "NbhErrors",
    SMALL,
    r##"
import org.code.neighborhood.*;

public class NbhErrors {
    public static void main(String[] args) {
        // A colour the library does not know.
        Painter p = new Painter(0, 0, "East", 5);
        try {
            p.paint("banana");
            System.out.println("no throw");
        } catch (RuntimeException e) {
            System.out.println("threw " + e.getMessage());
        }
        System.out.println("after a bad colour: onPaint=" + p.isOnPaint()
            + " paint=" + p.getMyPaint());

        // A web colour and a hex colour, both of which it does know.
        p.paint("red");
        System.out.println("web: " + p.getColor());
        Painter hex = new Painter(3, 3, "North", 2);
        hex.paint("#FF00FF");
        System.out.println("hex: " + hex.getColor());

        // Painting the square a bucket is standing on.
        Painter onBucket = new Painter(1, 1, "North", 5);
        try {
            onBucket.paint("blue");
            System.out.println("no throw; color=" + onBucket.getColor()
                + " paint=" + onBucket.getMyPaint());
        } catch (RuntimeException e) {
            System.out.println("threw " + e.getMessage());
        }

        // Walking into a wall.
        Painter walled = new Painter(1, 0, "East", 0);
        try {
            walled.move();
            System.out.println("no throw");
        } catch (RuntimeException e) {
            System.out.println("threw " + e.getMessage());
        }

        // A painter placed outside the grid.
        try {
            new Painter(9, 9, "North", 0);
            System.out.println("no throw");
        } catch (RuntimeException e) {
            System.out.println("threw " + e.getMessage());
        }
    }
}
"##
);
