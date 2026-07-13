//! Differential tests for the GRADING path of levels that use `org.code.*`.
//!
//! The companion of `differential_validation.rs`, which covers the plain-JUnit
//! validators. These ones reach into Code.org's own libraries — a
//! `NeighborhoodTestRunner.run()` that replays a student's Painter, a
//! `SystemOutTestRunner.run()` that reads back what they printed, an `EasyMock`
//! partial mock that checks which methods a student's method called — and
//! caturra models every one of them (`stdlib/neighborhood.java`,
//! `stdlib/validation.java`, `stdlib/easymock.java`). Those models decide
//! whether a student's work is marked correct, and nothing verified them.
//!
//! So: the whole reference stack, on a real JVM. The REAL `org.code.media`,
//! `org.code.neighborhood` and `org.code.validation` sources out of the vendored
//! `javabuilder/` checkout, real `JUnit` 5, real `EasyMock` — and the session those
//! libraries expect, stood up exactly as javabuilder's `CodeExecutionManager`
//! does it: a `ValidationProtocol` holding a `NeighborhoodTracker` and a
//! `SystemOutTracker`, fed by an `OutputAdapter`, with `System.out` redirected
//! through the real `OutputPrintStream` so a student's `println` becomes the
//! `SYSTEM_OUT` message the tracker is waiting for. Without that wiring
//! `NeighborhoodTestRunner.run()` has nothing to return.
//!
//! Only three things are stubbed, and none of them is under test: the session
//! protocol's singletons (message plumbing for a browser), `org.json` (checked
//! against the real jar — see `differential_neighborhood.rs`), and `Scene`,
//! whose real semantics are pixels drawn by `Java2D` into a GIF that caturra
//! deliberately never produces (see `differential_media.rs`).
//!
//! Needs `pnpm testjars:fetch`, the `javabuilder/` checkout and a JDK; skips
//! with a note otherwise. `CATURRA_REQUIRE_JUNIT=1` makes a missing one a
//! failure.
//!
//! Run across the corpus (871 comparable levels), this agreed with the real
//! grader on 869. The two it did not are both defects in the levels themselves:
//! one gives two tests the same `@Order` and then depends on which runs first
//! (`JUnit` breaks the tie by `getDeclaredMethods()` order, which the JVM spec
//! leaves undefined), and the other ships a solution with an off-by-one
//! (`> 15` where its own test means `>= 15`) that the real grader fails about
//! one run in thirty-six.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

/// `PROTOCOL_SHIM`
const PROTOCOL_SHIM: &[(&str, &str)] = &[
    (
        "ContentManager.java",
        r"package org.code.protocol;
public interface ContentManager { String getAssetUrl(String filename); }
",
    ),
    (
        "GlobalProtocol.java",
        r#"package org.code.protocol;
import java.io.File;
/** The session singletons the libraries reach for. The output adapter is set by
 *  the harness so a Painter's actions reach the validation trackers. */
public class GlobalProtocol {
  private OutputAdapter outputAdapter = message -> {};
  private final ContentManager contentManager =
      filename -> new File(System.getProperty("caturra.assets", "."), filename).toURI().toString();
  public void setOutputAdapter(OutputAdapter adapter) { this.outputAdapter = adapter; }
  public OutputAdapter getOutputAdapter() { return this.outputAdapter; }
  public ContentManager getContentManager() { return this.contentManager; }
}
"#,
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
    (
        "JavabuilderRuntimeException.java",
        r"package org.code.protocol;

public abstract class JavabuilderRuntimeException extends RuntimeException {
  private String fallbackMessage;

  protected JavabuilderRuntimeException(Enum key) { super(key.toString()); }
  protected JavabuilderRuntimeException(Enum key, Throwable cause) { super(key.toString(), cause); }
  protected JavabuilderRuntimeException(Enum key, String fallbackMessage) {
    super(key.toString());
    this.fallbackMessage = fallbackMessage;
  }

  public void setFallbackMessage(String fallbackMessage) { this.fallbackMessage = fallbackMessage; }
  public String getFallbackMessage() { return this.fallbackMessage; }
}
",
    ),
];

/// `JSON_SHIM`
const JSON_SHIM: &[(&str, &str)] = &[
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
        "JSONException.java",
        r"package org.json;

public class JSONException extends RuntimeException {
  public JSONException(String message) { super(message); }
}
",
    ),
    (
        "JSONObject.java",
        r#"package org.json;

import java.util.HashMap;
import java.util.Map;
import java.util.Set;

/** The subset org.code uses: a string/int map, built from a Map or parsed. */
public class JSONObject {
  public static final Object NULL = new Object() {
    @Override public String toString() { return "null"; }
    @Override public boolean equals(Object other) { return other == this || other == null; }
    @Override public int hashCode() { return 0; }
  };

  private final Map<String, Object> entries;

  JSONObject(Map<String, Object> entries) { this.entries = entries; }

  public JSONObject() { this.entries = new HashMap<>(); }

  /** ClientMessage builds its detail from a HashMap<String, String>. */
  public JSONObject(HashMap<String, String> source) {
    this.entries = new HashMap<>();
    if (source != null) {
      for (Map.Entry<String, String> e : source.entrySet()) {
        this.entries.put(e.getKey(), e.getValue());
      }
    }
  }

  public boolean has(String key) { return this.entries.containsKey(key); }

  public int length() { return this.entries.size(); }

  /** The real org.json returns Set<String>; NeighborhoodTracker streams over it. */
  public Set<String> keySet() { return this.entries.keySet(); }

  public boolean isNull(String key) {
    Object value = this.entries.get(key);
    return value == null || NULL.equals(value);
  }

  public Object get(String key) {
    if (!this.entries.containsKey(key)) { throw new JSONException("no such key: " + key); }
    return this.entries.get(key);
  }

  public String getString(String key) { return String.valueOf(get(key)); }

  public int getInt(String key) {
    Object value = get(key);
    if (value instanceof Integer) { return ((Integer) value).intValue(); }
    return Integer.parseInt(String.valueOf(value));
  }

  public boolean getBoolean(String key) {
    Object value = get(key);
    if (value instanceof Boolean) { return ((Boolean) value).booleanValue(); }
    return Boolean.parseBoolean(String.valueOf(value));
  }

  public JSONObject put(String key, Object value) {
    this.entries.put(key, value);
    return this;
  }

  @Override
  public String toString() { return this.entries.toString(); }
}
"#,
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
];

/// `THEATER_STUB`
const THEATER_STUB: &[(&str, &str)] = &[
    (
        "Instrument.java",
        r"package org.code.theater;
public enum Instrument { PIANO, BASS }
",
    ),
    (
        "Scene.java",
        r#"package org.code.theater;

import java.io.FileOutputStream;
import java.io.IOException;
import org.code.media.Color;
import org.code.media.Font;
import org.code.media.FontStyle;
import org.code.media.Image;
import org.code.media.Pixel;

/**
 * NOT the real Scene: a stand-in with the real Scene's exact public signatures,
 * so a corpus level compiles and runs unchanged, and so `drawImage` can dump
 * the pixels the program actually produced. The real Scene draws into a
 * BufferedImage and encodes a GIF; caturra deliberately renders in the browser
 * instead, so there is no common pixel surface to compare there. What IS being
 * compared through this stub is the student's filter code running against the
 * REAL org.code.media Image/Pixel/Color — the pixel maths, which is the whole
 * of what caturra models.
 */
public class Scene {
  private static int drawn = 0;

  public Scene() {}

  public final int getWidth() { return 400; }

  public final int getHeight() { return 400; }

  public final void clear(String color) { new Color(color); }

  public final void clear(Color color) {}

  public final void playSound(double[] sound) {}

  public final void playSound(String filename) {}

  public final void playNote(int note, double seconds) {}

  public final void playNoteAndPause(int note, double seconds) {}

  public final void playNote(Instrument instrument, int note, double seconds) {}

  public final void playNoteAndPause(Instrument instrument, int note, double seconds) {}

  public final void pause(double seconds) {}

  /** Every drawn image is dumped: width, height, then one RGB triple per pixel. */
  private void dump(Image image) {
    try {
      int w = image.getWidth();
      int h = image.getHeight();
      byte[] out = new byte[8 + w * h * 3];
      out[0] = (byte) (w & 0xff); out[1] = (byte) ((w >> 8) & 0xff);
      out[2] = (byte) ((w >> 16) & 0xff); out[3] = (byte) ((w >> 24) & 0xff);
      out[4] = (byte) (h & 0xff); out[5] = (byte) ((h >> 8) & 0xff);
      out[6] = (byte) ((h >> 16) & 0xff); out[7] = (byte) ((h >> 24) & 0xff);
      int at = 8;
      for (int y = 0; y < h; y++) {
        for (int x = 0; x < w; x++) {
          Pixel p = image.getPixel(x, y);
          out[at++] = (byte) p.getRed();
          out[at++] = (byte) p.getGreen();
          out[at++] = (byte) p.getBlue();
        }
      }
      FileOutputStream f = new FileOutputStream("__ref_img_" + drawn);
      f.write(out);
      f.close();
      drawn++;
    } catch (IOException e) {
      throw new RuntimeException(e);
    }
  }

  public final void drawImage(Image image, int x, int y, int size) { dump(image); }

  public final void drawImage(Image image, int x, int y, int size, double rotation) { dump(image); }

  public final void drawImage(Image image, int x, int y, int width, int height, double rotation) { dump(image); }

  public final void drawImage(String filename, int x, int y, int size) { new Image(filename); }

  public final void drawImage(String filename, int x, int y, int size, double rotation) { new Image(filename); }

  public final void drawImage(String filename, int x, int y, int width, int height, double rotation) { new Image(filename); }

  public final void setTextStyle(Font font, FontStyle style) {}

  public final void setTextHeight(int height) {}

  public final void setTextColor(String color) { new Color(color); }

  public final void setTextColor(Color color) {}

  public final void drawText(String text, int x, int y) {}

  public final void drawText(String text, int x, int y, double rotation) {}

  public final void drawLine(int startX, int startY, int endX, int endY) {}

  public final void drawRegularPolygon(int x, int y, int sides, int radius) {}

  public final void drawShape(int[] points, boolean close) {}

  public final void drawEllipse(int x, int y, int width, int height) {}

  public final void drawRectangle(int x, int y, int width, int height) {}

  public final void setStrokeWidth(double width) {}

  public final void setFillColor(Color color) {}

  public final void setStrokeColor(Color color) {}

  public final void setFillColor(String color) { new Color(color); }

  public final void setStrokeColor(String color) { new Color(color); }

  public final void removeStrokeColor() {}

  public final void removeFillColor() {}
}
"#,
    ),
    (
        "Theater.java",
        r"package org.code.theater;
public class Theater {
  public static void playScenes(Scene... scenes) {}
}
",
    ),
];

/// The reference grading run for an org.code level.
const REF_RUNNER: &str = r#"import java.io.PrintStream;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.code.javabuilder.OutputPrintStream;
import org.code.protocol.ClientMessage;
import org.code.protocol.JavabuilderContext;
import org.code.protocol.OutputAdapter;
import org.code.validation.support.NeighborhoodTracker;
import org.code.validation.support.SystemOutTracker;
import org.code.validation.support.ValidationProtocol;
import org.junit.platform.engine.TestExecutionResult;
import org.junit.platform.engine.discovery.ClassSelector;
import org.junit.platform.engine.discovery.DiscoverySelectors;
import org.junit.platform.launcher.Launcher;
import org.junit.platform.launcher.LauncherDiscoveryRequest;
import org.junit.platform.launcher.TestExecutionListener;
import org.junit.platform.launcher.TestIdentifier;
import org.junit.platform.launcher.TestPlan;
import org.junit.platform.launcher.core.LauncherDiscoveryRequestBuilder;
import org.junit.platform.launcher.core.LauncherFactory;

/**
 * The reference grading run for levels that use org.code.*: stand up the session
 * the real runtime would, then discover and execute the validator with real
 * JUnit 5, printing one `__VTEST\t<PASS|FAIL>\t<name>\t<message>` line per test.
 *
 * `NeighborhoodTestRunner.run()` and `SystemOutTestRunner.run()` reach through
 * JavabuilderContext for a ValidationProtocol, invoke the student's main, and
 * hand back what the trackers observed — so the harness must build that protocol
 * exactly as javabuilder's CodeExecutionManager does: a NeighborhoodTracker and
 * a SystemOutTracker fed by an OutputAdapter, and System.out redirected through
 * the real OutputPrintStream so a student's println becomes a SYSTEM_OUT message
 * the tracker can see.
 *
 * Usage: OrgCodeRefRunner <MainClass|-> <userClass,userClass,...|-> <TestClass...>
 */
public class OrgCodeRefRunner {
  public static void main(String[] args) throws Exception {
    PrintStream real = System.out;

    String mainClassName = args[0];
    List<String> tests = Arrays.asList(args).subList(2, args.length);

    Method mainMethod = null;
    if (!mainClassName.equals("-")) {
      Class<?> mainClass = Class.forName(mainClassName);
      mainMethod = mainClass.getMethod("main", String[].class);
    }
    // ValidationHelper.getClassNames() hands the validator the student's
    // classes, so it must be all of them, not just the one declaring main.
    List<String> userClassNames = new ArrayList<>();
    if (!args[1].equals("-")) {
      for (String name : args[1].split(",")) {
        if (!name.isEmpty()) {
          userClassNames.add(name);
        }
      }
    }

    NeighborhoodTracker neighborhood = new NeighborhoodTracker();
    SystemOutTracker systemOut = new SystemOutTracker();
    ValidationProtocol protocol =
        new ValidationProtocol(mainMethod, neighborhood, systemOut, userClassNames);
    JavabuilderContext.getInstance().register(ValidationProtocol.class, protocol);

    // Every message a Painter fires, and every line the student prints, reaches
    // the trackers — which is the whole of what the validators then assert on.
    OutputAdapter adapter =
        new OutputAdapter() {
          @Override
          public void sendMessage(ClientMessage message) {
            protocol.trackEvent(message);
          }
        };
    JavabuilderContext.getInstance().getGlobalProtocol().setOutputAdapter(adapter);
    System.setOut(new OutputPrintStream(adapter));

    List<ClassSelector> selectors = new ArrayList<>();
    for (String name : tests) {
      selectors.add(DiscoverySelectors.selectClass(Class.forName(name)));
    }
    LauncherDiscoveryRequest request =
        LauncherDiscoveryRequestBuilder.request().selectors(selectors).build();
    Launcher launcher = LauncherFactory.create();
    launcher.registerTestExecutionListeners(
        new TestExecutionListener() {
          @Override
          public void executionFinished(TestIdentifier id, TestExecutionResult result) {
            if (!id.isTest()) {
              return;
            }
            boolean passed = result.getStatus() == TestExecutionResult.Status.SUCCESSFUL;
            String message = result.getThrowable().map(Throwable::getMessage).orElse("");
            if (message == null) {
              message = "";
            }
            real.println(
                "__VTEST\t"
                    + (passed ? "PASS" : "FAIL")
                    + "\t"
                    + id.getDisplayName()
                    + "\t"
                    + message.replace('\t', ' ').replace('\r', ' ').replace('\n', ' '));
          }
        });
    TestPlan plan = launcher.discover(request);
    if (plan.containsTests()) {
      launcher.execute(plan);
    }
    real.flush();
  }
}
"#;

/// The real library sources compiled into the reference, relative to
/// `javabuilder/org-code-javabuilder/`. `ClientMessage` and its friends are
/// dependency-light enough to use as-is; `OutputPrintStream` and the redirection
/// stream are what turn a student's `println` into the message the
/// `SystemOutTracker` reads.
const REAL_SOURCES: &[&str] = &[
    "protocol/src/main/java/org/code/protocol/ClientMessage.java",
    "protocol/src/main/java/org/code/protocol/ClientMessageType.java",
    "protocol/src/main/java/org/code/protocol/ClientMessageDetailKeys.java",
    "protocol/src/main/java/org/code/protocol/JavabuilderSharedObject.java",
    "protocol/src/main/java/org/code/protocol/OutputAdapter.java",
    "media/src/main/java/org/code/media/Color.java",
    "media/src/main/java/org/code/media/Pixel.java",
    "media/src/main/java/org/code/media/Image.java",
    "media/src/main/java/org/code/media/Font.java",
    "media/src/main/java/org/code/media/FontStyle.java",
    "media/src/main/java/org/code/media/support/MediaRuntimeException.java",
    "media/src/main/java/org/code/media/support/MediaRuntimeExceptionKeys.java",
    "lib/src/main/java/org/code/javabuilder/OutputPrintStream.java",
    "lib/src/main/java/org/code/javabuilder/OutputRedirectionStream.java",
    "lib/src/main/java/org/code/javabuilder/SystemOutMessage.java",
];

/// Whole packages taken as they are.
const REAL_PACKAGES: &[&str] = &[
    "neighborhood/src/main/java/org/code/neighborhood",
    "neighborhood/src/main/java/org/code/neighborhood/support",
    "validation/src/main/java/org/code/validation",
    "validation/src/main/java/org/code/validation/support",
];

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

fn javabuilder() -> Option<PathBuf> {
    let root = workspace_root().join("javabuilder/org-code-javabuilder");
    root.join("validation").is_dir().then_some(root)
}

/// The jars from `pnpm testjars:fetch` — `JUnit` 5, and `EasyMock` for the Unit 1
/// validators that mock a student's Painter.
fn jars() -> Option<String> {
    let dir = workspace_root().join("vendor/junit");
    let mut jars: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension()? == "jar").then_some(path)
        })
        .collect();
    if jars.is_empty() {
        return None;
    }
    jars.sort();
    Some(
        jars.iter()
            .map(|jar| jar.display().to_string())
            .collect::<Vec<_>>()
            .join(":"),
    )
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
    let available = jdk_available() && jars().is_some() && javabuilder().is_some();
    assert!(
        available || std::env::var_os("CATURRA_REQUIRE_JUNIT").is_none(),
        "CATURRA_REQUIRE_JUNIT is set but the reference cannot be built (needs a \
         JDK, `pnpm testjars:fetch`, and the javabuilder/ checkout): this suite \
         would have reported `ok` without comparing anything"
    );
    if !available {
        eprintln!("skipping: needs a JDK, `pnpm testjars:fetch`, and the javabuilder/ checkout");
    }
    available
}

/// Compile the shims, the real libraries and the runner once per test binary.
fn reference() -> &'static (PathBuf, String) {
    static REF: OnceLock<(PathBuf, String)> = OnceLock::new();
    REF.get_or_init(|| {
        let jb = javabuilder().expect("checked");
        let classpath = jars().expect("checked");
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("orgcode-validation");
        let classes = root.join("classes");
        std::fs::create_dir_all(&classes).expect("create classes");

        let mut sources: Vec<PathBuf> = Vec::new();
        for (dir, shim) in [
            ("shim/org/code/protocol", PROTOCOL_SHIM),
            ("shim/org/json", JSON_SHIM),
            ("stub/org/code/theater", THEATER_STUB),
        ] {
            let dir = root.join(dir);
            std::fs::create_dir_all(&dir).expect("create shim dir");
            for (name, text) in shim {
                let path = dir.join(name);
                std::fs::write(&path, text).expect("write shim");
                sources.push(path);
            }
        }
        sources.extend(REAL_SOURCES.iter().map(|rel| jb.join(rel)));
        for package in REAL_PACKAGES {
            let entries = std::fs::read_dir(jb.join(package)).expect("read a real package");
            sources.extend(entries.filter_map(|entry| {
                let path = entry.ok()?.path();
                (path.extension()? == "java").then_some(path)
            }));
        }

        let compile = Command::new("javac")
            .args(["-nowarn", "-cp", &classpath, "-d"])
            .arg(&classes)
            .args(&sources)
            .output()
            .expect("javac runs");
        assert!(
            compile.status.success(),
            "the real org.code libraries did not compile against the shims — their \
             dependencies have moved: {}",
            String::from_utf8_lossy(&compile.stderr)
        );

        let full = format!("{classpath}:{}", classes.display());
        std::fs::write(root.join("OrgCodeRefRunner.java"), REF_RUNNER).expect("write runner");
        let compile = Command::new("javac")
            .args(["-nowarn", "-cp", &full, "-d", "."])
            .arg("OrgCodeRefRunner.java")
            .current_dir(&root)
            .output()
            .expect("javac runs");
        assert!(
            compile.status.success(),
            "OrgCodeRefRunner did not compile: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        (root, full)
    })
}

/// One level: its sources (the student's, then the validator), and the grid the
/// host would have written. The grid is given once and rendered into each
/// engine's own `grid.txt` encoding — Code.org's JSON, caturra's
/// `tileType,paintCount` rows — because that file is written by the host, never
/// by a student.
struct Level<'a> {
    sources: &'a [(&'a str, &'a str)],
    grid: Option<&'a str>,
}

fn grid_as_json(grid: &str) -> String {
    let rows: Vec<String> = grid
        .trim()
        .lines()
        .map(|line| {
            let cells: Vec<String> = line
                .split_whitespace()
                .map(|cell| {
                    let (tile, paint) = cell.split_once(',').unwrap_or((cell, "0"));
                    format!(r#"{{"tileType":{tile},"value":{paint}}}"#)
                })
                .collect();
            format!("[{}]", cells.join(","))
        })
        .collect();
    format!("[{}]", rows.join(","))
}

/// The `__VTEST` verdicts, reconciled against the roster the runner announced.
/// A planned test with no verdict did not run — a VM error is not a Java
/// throwable, so nothing in the generated runner could have caught it — and it
/// is a failure, never an absence.
fn verdicts(stdout: &str) -> String {
    let mut planned: Vec<&str> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let mut reported: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for line in stdout.lines() {
        if let Some(name) = line.strip_prefix("__VPLAN\t") {
            planned.push(name);
        } else if line.starts_with("__VTEST\t") {
            if let Some(name) = line.split('\t').nth(2) {
                reported.insert(name);
            }
            lines.push(line.to_owned());
        }
    }
    for name in planned {
        if !reported.contains(name) {
            lines.push(format!(
                "__VTEST\tFAIL\t{name}\tthe engine stopped before this test ran"
            ));
        }
    }
    let mut out = String::new();
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn run_with_reference(name: &str, level: &Level) -> String {
    let (root, classpath) = reference();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&level.sources.len(), &mut hasher);
    for (file, text) in level.sources {
        std::hash::Hash::hash(file, &mut hasher);
        std::hash::Hash::hash(text, &mut hasher);
    }
    let fingerprint = std::hash::Hasher::finish(&hasher);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("ocv-{name}-{fingerprint:x}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    for (file, text) in level.sources {
        std::fs::write(dir.join(file), text).expect("write source");
    }
    if let Some(grid) = level.grid {
        std::fs::write(dir.join("grid.txt"), grid_as_json(grid)).expect("write grid");
    }

    let compile = Command::new("javac")
        .args(["-nowarn", "-cp", classpath, "-d", "."])
        .args(level.sources.iter().map(|(file, _)| *file))
        .current_dir(&dir)
        .output()
        .expect("javac runs");
    assert!(
        compile.status.success(),
        "javac rejected {name} against the real libraries: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let tests: Vec<&str> = level
        .sources
        .iter()
        .filter(|(_, text)| text.contains("org.junit"))
        .map(|(file, _)| file.trim_end_matches(".java"))
        .collect();
    let users: Vec<&str> = level
        .sources
        .iter()
        .filter(|(_, text)| !text.contains("org.junit"))
        .map(|(file, _)| file.trim_end_matches(".java"))
        .collect();
    let main = if users.contains(&"Main") { "Main" } else { "-" };

    let run = Command::new("java")
        .arg("-Djava.awt.headless=true")
        .arg("-cp")
        .arg(format!("{classpath}:{}:.", root.display()))
        .arg("OrgCodeRefRunner")
        .arg(main)
        .arg(users.join(","))
        .args(&tests)
        .current_dir(&dir)
        .output()
        .expect("java runs");
    verdicts(&String::from_utf8_lossy(&run.stdout))
}

fn run_with_caturra(name: &str, level: &Level) -> String {
    let sources: Vec<caturra_compiler::SourceFile> = level
        .sources
        .iter()
        .map(|(file, text)| caturra_compiler::SourceFile {
            path: (*file).to_owned(),
            text: (*text).to_owned(),
        })
        .collect();
    let compilation = caturra_compiler::compile(&sources);
    assert!(
        compilation.success(),
        "caturra rejected {name}: {:?}",
        compilation.diagnostics
    );
    let entry = compilation
        .validation_entry
        .clone()
        .expect("caturra found no tests");
    let mut vfs = VirtualFileSystem::new();
    if let Some(grid) = level.grid {
        vfs.write_file("grid.txt", grid.trim().to_owned())
            .expect("write grid");
    }
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = Vm::new(VmOptions::default(), &mut vfs, &mut console);
    for class in compilation.classes {
        vm.load_class(class.class_file).expect("class loads");
    }
    // A run that dies is exactly what this suite exists to catch, so the
    // outcome is not asserted: what is compared is the verdicts, reconciled.
    let _ = vm.run_main(&entry, &[]);
    drop(vm);
    verdicts(&console.stdout_text())
}

fn assert_same_verdicts(name: &str, level: &Level) {
    let expected = run_with_reference(name, level);
    let actual = run_with_caturra(name, level);
    assert_eq!(
        actual, expected,
        "caturra grades {name} differently from the real Code.org grader"
    );
}

/// A 4x4 world, all open, with a bucket of 3 paint at (1,1).
const GRID: &str = "\
1,0 1,0 1,0 1,0
1,0 1,3 1,0 1,0
1,0 1,0 1,0 1,0
1,0 1,0 1,0 1,0";

// `NeighborhoodTestRunner.run()` replays the student's `main` and hands the
// validator a log of everything the Painter did — where it went, what it
// painted, what it picked up. It only has that log because the session wiring
// routes every Painter message into a tracker, so this exercises the whole
// chain: the student's Painter, the message stream, the tracker, the log's
// query methods, and the verdict JUnit reaches from them.
#[test]
fn orgcode_neighborhood_test_runner() {
    if !reference_available() {
        return;
    }
    assert_same_verdicts(
        "Nbh",
        &Level {
            grid: Some(GRID),
            sources: &[
                (
                    "Main.java",
                    r#"
import org.code.neighborhood.*;

public class Main {
    public static void main(String[] args) {
        Painter painter = new Painter(0, 0, "east", 2);
        painter.move();
        painter.paint("red");
        painter.turnLeft();
        System.out.println("done at " + painter.getX() + "," + painter.getY());
    }
}
"#,
                ),
                (
                    "MainTest.java",
                    r#"
import org.code.validation.*;
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class MainTest {
    NeighborhoodLog log;

    @BeforeEach
    public void setup() {
        log = NeighborhoodTestRunner.run();
    }

    @Test
    @Order(1)
    @DisplayName("the painter ends where the program left it => ")
    public void endsAtTheRightPlace() {
        PainterLog[] painters = log.getPainterLogs();
        assertEquals(1, painters.length, "one painter should have been created");
        Position end = painters[0].getEndingPosition();
        assertEquals(1, end.getX(), "final x");
        assertEquals(0, end.getY(), "final y");
        assertEquals("north", end.getDirection(), "turned left from east");
    }

    @Test
    @Order(2)
    @DisplayName("the painter painted exactly once, and moved once => ")
    public void didTheRightActions() {
        PainterLog painter = log.getPainterLogs()[0];
        assertTrue(painter.didActionExactly(NeighborhoodActionType.PAINT, 1), "painted once");
        assertTrue(painter.didActionExactly(NeighborhoodActionType.MOVE, 1), "moved once");
        assertEquals(1, painter.getEndingPaintCount(), "spent one of its two paints");
    }

    @Test
    @Order(3)
    @DisplayName("a claim that does not hold fails => ")
    public void thisOneFails() {
        PainterLog painter = log.getPainterLogs()[0];
        assertEquals(3, painter.getEndingPosition().getX(), "the painter did not go this far");
    }
}
"#,
                ),
            ],
        },
    );
}

// `SystemOutTestRunner.run()` replays the student's `main` and hands back what
// they printed, line by line. caturra models that too, and the lines have to
// arrive the same way: one message per `println`, and no stray blank.
#[test]
fn orgcode_system_out_test_runner() {
    if !reference_available() {
        return;
    }
    assert_same_verdicts(
        "SysOut",
        &Level {
            grid: None,
            sources: &[
                (
                    "Main.java",
                    r#"
public class Main {
    public static void main(String[] args) {
        Dessert cake = new Dessert("cake", 3);
        System.out.println(cake.describe());
        System.out.println("servings: " + cake.getServings());
    }
}
"#,
                ),
                (
                    "Dessert.java",
                    r#"
public class Dessert {
    private String name;
    private int servings;

    public Dessert(String name, int servings) {
        this.name = name;
        this.servings = servings;
    }

    public String describe() {
        return "A " + name;
    }

    public int getServings() {
        return servings;
    }
}
"#,
                ),
                (
                    "DessertTest.java",
                    r#"
import java.util.List;
import org.code.validation.*;
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class DessertTest {
    @Test
    @Order(1)
    @DisplayName("the program prints what it should => ")
    public void printsTheRightThing() {
        List<String> output = SystemOutTestRunner.run();
        assertEquals(2, output.size(), "two lines should have been printed");
        assertTrue(output.get(0).contains("A cake"), "first line describes the dessert");
        assertTrue(output.get(1).contains("servings: 3"), "second line has the servings");
    }

    @Test
    @Order(2)
    @DisplayName("a claim about the output that does not hold fails => ")
    public void thisOneFails() {
        List<String> output = SystemOutTestRunner.run();
        assertTrue(output.get(0).contains("pie"), "there is no pie here");
    }
}
"#,
                ),
            ],
        },
    );
}
