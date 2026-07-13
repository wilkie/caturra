//! Differential tests for `org.code.media`: run the same program through the
//! **real Code.org library** and through caturra, and require identical stdout.
//!
//! `differential.rs` does this for `java.*` against the installed JDK. The
//! `org.code` model had no equivalent — it is hand-written from the docs, and
//! the corpus parity sweep excludes `org.code` programs entirely — so it was
//! the one part of caturra never compared against a reference. Three
//! wrong-picture bugs were sitting in it (an unclamped channel smeared colour
//! across the packed pixel, a blank image came out black instead of white, and
//! `getPixel` handed back a new `Pixel` each call instead of the cached one).
//! Each was found by eye, on a rendered image. This is the tool that finds the
//! next one.
//!
//! The reference is the vendored `javabuilder/` checkout of Code.org's actual
//! source. Its `media` package needs only four types from `org.code.protocol`
//! (a context, a protocol, a content manager, an exception base), so those are
//! shimmed below and the real `Color`, `Pixel` and `Image` compile against
//! them unmodified. Skips (with a note) when the checkout or a JDK is absent;
//! set `CATURRA_REQUIRE_JAVABUILDER=1` to make that a failure instead.
//!
//! **What this cannot pin.** `Image(String)` loads a named asset, which in the
//! real library means an HTTP fetch through the content manager and a JPEG
//! decode; in caturra the host preloads decoded pixels into the VFS. The
//! decode is the host's, not the model's, so the file-loading path is out of
//! scope here — every test below builds its image in code. And the real
//! `Color`/`Pixel`/`Image` override neither `toString` nor `equals`, so
//! printing one gives `org.code.media.Color@1b6d3586`: an identity hash, and a
//! package-qualified name caturra's flat namespace has no way to produce.
//! caturra prints `10 20 30` instead. That divergence is deliberate and
//! unmatchable, so nothing here prints a bare `Color`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use caturra_vm::{BufferedConsole, VirtualFileSystem, Vm, VmOptions};

/// The four `org.code.protocol` types the media package touches. The real ones
/// drag in the whole session protocol (output adapters, message handlers, a
/// singleton registry); these satisfy the same call sites and nothing else. A
/// content manager that resolves an asset name to a local file URL is enough
/// for `Image(String)` to compile — no test calls it (see the module note).
const PROTOCOL_SHIM: &[(&str, &str)] = &[
    (
        "JavabuilderRuntimeException.java",
        r"package org.code.protocol;

public abstract class JavabuilderRuntimeException extends RuntimeException {
  protected JavabuilderRuntimeException(Enum key) { super(key.toString()); }
  protected JavabuilderRuntimeException(Enum key, Throwable cause) { super(key.toString(), cause); }
  protected JavabuilderRuntimeException(Enum key, String fallbackMessage) { super(key.toString()); }
}
",
    ),
    (
        "ContentManager.java",
        r"package org.code.protocol;

public interface ContentManager {
  String getAssetUrl(String filename);
}
",
    ),
    (
        "GlobalProtocol.java",
        r#"package org.code.protocol;

import java.io.File;

public class GlobalProtocol {
  private final ContentManager contentManager =
      filename -> new File(System.getProperty("caturra.assets", "."), filename).toURI().toString();

  public ContentManager getContentManager() { return this.contentManager; }
}
"#,
    ),
    (
        "JavabuilderContext.java",
        r"package org.code.protocol;

public class JavabuilderContext {
  private static final JavabuilderContext INSTANCE = new JavabuilderContext();
  private final GlobalProtocol protocol = new GlobalProtocol();

  public static JavabuilderContext getInstance() { return INSTANCE; }

  public GlobalProtocol getGlobalProtocol() { return this.protocol; }
}
",
    ),
];

/// The real library's sources, relative to the `media` package root.
const MEDIA_SOURCES: &[&str] = &[
    "Color.java",
    "Pixel.java",
    "Image.java",
    "support/MediaRuntimeException.java",
    "support/MediaRuntimeExceptionKeys.java",
];

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
}

/// The vendored Code.org sources, if this checkout has them. `javabuilder/` is
/// a local clone, not part of the repository, so its absence is a skip.
fn media_package() -> Option<PathBuf> {
    let package = workspace_root()
        .join("javabuilder/org-code-javabuilder/media/src/main/java/org/code/media");
    package.join("Image.java").is_file().then_some(package)
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

/// Whether the reference can run at all. Reports why when it cannot, so a
/// skipped test does not read as a passing one.
fn reference_available() -> bool {
    let available = jdk_available() && media_package().is_some();
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

/// Compile the shim and the real `Color`/`Pixel`/`Image` once per test binary,
/// and hand back the classpath directory holding them.
fn reference_classes() -> &'static Path {
    static CLASSES: OnceLock<PathBuf> = OnceLock::new();
    CLASSES.get_or_init(|| {
        let package = media_package().expect("caller checked the checkout is present");
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("org-code-media");
        let (shim, classes) = (root.join("shim/org/code/protocol"), root.join("classes"));
        std::fs::create_dir_all(&shim).expect("create shim dir");
        std::fs::create_dir_all(&classes).expect("create classes dir");

        let mut sources: Vec<PathBuf> = Vec::new();
        for (name, text) in PROTOCOL_SHIM {
            let path = shim.join(name);
            std::fs::write(&path, text).expect("write shim source");
            sources.push(path);
        }
        sources.extend(MEDIA_SOURCES.iter().map(|name| package.join(name)));

        let compile = Command::new("javac")
            .arg("-d")
            .arg(&classes)
            .args(&sources)
            .output()
            .expect("javac runs");
        assert!(
            compile.status.success(),
            "the real org.code.media did not compile against the shim — its \
             dependencies have moved: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        classes
    })
}

/// Run `source` against the real library, returning stdout.
fn run_with_reference(class_name: &str, source: &str) -> String {
    let classes = reference_classes();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(source, &mut hasher);
    let fingerprint = std::hash::Hasher::finish(&hasher);
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("media-{class_name}-{fingerprint:x}"));
    std::fs::create_dir_all(&dir).expect("create temp dir");
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

    // Headless: the media package reaches into java.awt, and CI has no display.
    let run = Command::new("java")
        .arg("-Djava.awt.headless=true")
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

/// Run `source` through caturra, returning stdout.
fn run_with_caturra(class_name: &str, source: &str) -> String {
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
    let mut console = BufferedConsole::with_input(Vec::<String>::new());
    let mut vm = Vm::new(
        VmOptions {
            // The image walks are millions of instructions; the default budget
            // is sized for a lesson, not for a 400x400 filter.
            max_instructions: 2_000_000_000,
            ..VmOptions::default()
        },
        &mut vfs,
        &mut console,
    );
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

fn assert_same_output(class_name: &str, source: &str) {
    let expected = run_with_reference(class_name, source);
    let actual = run_with_caturra(class_name, source);
    assert_eq!(
        actual, expected,
        "caturra's org.code.media diverges from the real Code.org library for \
         {class_name}"
    );
}

macro_rules! media_differential_test {
    ($name:ident, $class:literal, $source:literal) => {
        #[test]
        fn $name() {
            if !reference_available() {
                return;
            }
            assert_same_output($class, $source);
        }
    };
}

// A blank image is WHITE (Image(int,int) fills with DEFAULT_BACKGROUND_COLOR),
// not the black that Java's zero-fill would leave — caturra drew black until
// this suite existed. getPixel hands back the SAME Pixel for a coordinate (the
// real Image caches a Pixel[][]), so `==` holds and an out-of-range coordinate
// is an AIOOBE off that array rather than a Pixel silently aliasing another row.
media_differential_test!(
    media_blank_image_and_pixel_identity,
    "MediaBlank",
    r#"
import org.code.media.*;

public class MediaBlank {
    public static void main(String[] args) {
        Image img = new Image(4, 3);
        System.out.println(img.getWidth() + "x" + img.getHeight());

        Pixel p = img.getPixel(0, 0);
        System.out.println(p.getRed() + "," + p.getGreen() + "," + p.getBlue());
        System.out.println(p.getX() + " " + p.getY());

        // The same Pixel object every time, so a write through one handle is a
        // write through the other, and identity holds.
        System.out.println(img.getPixel(2, 1) == img.getPixel(2, 1));
        Pixel a = img.getPixel(2, 1);
        Pixel b = img.getPixel(2, 1);
        a.setRed(7);
        System.out.println(b.getRed());

        // Out of range is an exception, not a wrapped-around neighbour.
        try {
            img.getPixel(4, 0);
            System.out.println("no throw");
        } catch (ArrayIndexOutOfBoundsException e) {
            System.out.println("aioobe");
        }
    }
}
"#
);

// Every Color constructor runs each channel through sanitizeValue, which clamps
// to 0..255 — and Pixel's channel setters inherit that by going through
// Color.copyWithRed(...). caturra packed the raw value into the pixel instead,
// so a negative channel smeared its sign bits across the other two: the red and
// yellow streaks over a sharpened image that started this whole exercise.
media_differential_test!(
    media_channels_clamp,
    "MediaClamp",
    r#"
import org.code.media.*;

public class MediaClamp {
    static void show(Color c) {
        System.out.println(c.getRed() + "," + c.getGreen() + "," + c.getBlue());
    }

    public static void main(String[] args) {
        show(new Color(300, -5, 128));
        show(new Color(-1, 256, 255));
        show(Color.copyWithRed(new Color(1, 2, 3), 999));
        show(Color.copyWithGreen(new Color(1, 2, 3), -999));
        show(Color.copyWithBlue(new Color(1, 2, 3), 128));
        show(new Color(new Color(-7, 300, 12)));

        // The named constants go through the same clamp: LIME is written
        // `new Color(0, 256, 0)` upstream, and 256 is not a green.
        show(new Color("lime"));
        show(new Color("orange"));

        Image img = new Image(2, 1);
        Pixel p = img.getPixel(0, 0);
        p.setRed(-40);
        p.setGreen(300);
        p.setBlue(128);
        System.out.println(p.getRed() + "," + p.getGreen() + "," + p.getBlue());
        p.setColor(new Color(-1, -1, 999));
        System.out.println(p.getRed() + "," + p.getGreen() + "," + p.getBlue());

        // The neighbouring pixel is untouched by any of it (a wrapped channel
        // used to corrupt whatever shared the packed word).
        Pixel q = img.getPixel(1, 0);
        System.out.println(q.getRed() + "," + q.getGreen() + "," + q.getBlue());
    }
}
"#
);

// setPixel / clear / getColor / the copy constructor, and the aliasing rules
// between an Image and the Pixels handed out of it.
media_differential_test!(
    media_image_writes_and_copies,
    "MediaWrites",
    r#"
import org.code.media.*;

public class MediaWrites {
    static void show(Pixel p) {
        System.out.println(p.getRed() + "," + p.getGreen() + "," + p.getBlue());
    }

    public static void main(String[] args) {
        Image img = new Image(3, 2);
        img.clear(new Color(10, 20, 30));
        show(img.getPixel(0, 0));
        show(img.getPixel(2, 1));

        img.setPixel(1, 1, new Color(200, 100, 50));
        show(img.getPixel(1, 1));

        // A Pixel taken before the write still sees it: it is the image's.
        Pixel held = img.getPixel(0, 1);
        img.setPixel(0, 1, new Color(1, 2, 3));
        show(held);

        // getColor is a snapshot; mutating the pixel does not change it.
        Color snapshot = held.getColor();
        held.setRed(255);
        System.out.println(snapshot.getRed() + " " + held.getRed());

        // A copy is independent of its source, in both directions.
        Image copy = new Image(img);
        System.out.println(copy.getWidth() + "x" + copy.getHeight());
        copy.setPixel(1, 1, new Color(0, 0, 0));
        show(copy.getPixel(1, 1));
        show(img.getPixel(1, 1));
        img.setPixel(2, 0, new Color(9, 9, 9));
        show(copy.getPixel(2, 0));
    }
}
"#
);

// The U5L8 exemplar's algorithm, end to end on a 120x120 image: build a
// deterministic picture, sharpen it against the top-left neighbour, and hash
// every channel of the result. The hash is computed by the program itself, so
// both engines run the same arithmetic and any divergence in the library — a
// clamp, a default background, an aliased pixel — moves it. This is the test
// that would have caught the streaks without anyone looking at a picture.
media_differential_test!(
    media_sharpen_filter_matches_pixel_for_pixel,
    "MediaSharpen",
    r#"
import org.code.media.*;

public class MediaSharpen {
    public static void main(String[] args) {
        int size = 120;
        Image img = new Image(size, size);

        // A deterministic picture with plenty of edges, so the filter's
        // differences swing hard in both directions.
        for (int x = 0; x < size; x++) {
            for (int y = 0; y < size; y++) {
                int r = (x * 7 + y * 3) % 256;
                int g = (x * x + y) % 256;
                int b = ((x / 8) % 2 == 0) ? 250 : 5;
                img.setPixel(x, y, new Color(r, g, b));
            }
        }

        Pixel[][] pixels = new Pixel[img.getHeight()][img.getWidth()];
        for (int row = 0; row < pixels.length; row++) {
            for (int col = 0; col < pixels[0].length; col++) {
                pixels[row][col] = img.getPixel(col, row);
            }
        }

        // Verbatim from the CSA U5L8 solution: it caps at 255 and never at 0,
        // so it hands the setters negatives on every dark edge.
        for (int row = 1; row < pixels.length - 1; row++) {
            for (int col = 1; col < pixels[0].length - 1; col++) {
                Pixel currentPixel = pixels[row][col];

                int redDiff = currentPixel.getRed() - pixels[row - 1][col - 1].getRed();
                int greenDiff = currentPixel.getGreen() - pixels[row - 1][col - 1].getGreen();
                int blueDiff = currentPixel.getBlue() - pixels[row - 1][col - 1].getBlue();
                int averageDiff = (redDiff + greenDiff + blueDiff) / 3;

                int newRed = currentPixel.getRed() + averageDiff;
                int newGreen = currentPixel.getGreen() + averageDiff;
                int newBlue = currentPixel.getBlue() + averageDiff;

                if (newRed > 255) { newRed = 255; }
                if (newGreen > 255) { newGreen = 255; }
                if (newBlue > 255) { newBlue = 255; }

                currentPixel.setRed(newRed);
                currentPixel.setGreen(newGreen);
                currentPixel.setBlue(newBlue);
            }
        }

        // FNV-1a over every channel: one number that only matches if all
        // 43200 of them do.
        int hash = -2128831035;
        for (int y = 0; y < size; y++) {
            for (int x = 0; x < size; x++) {
                Pixel p = img.getPixel(x, y);
                hash = (hash ^ p.getRed()) * 16777619;
                hash = (hash ^ p.getGreen()) * 16777619;
                hash = (hash ^ p.getBlue()) * 16777619;
            }
        }
        System.out.println("sharpened hash " + hash);

        // And a few pixels in the clear, so a mismatch says something.
        for (int i = 0; i < 4; i++) {
            Pixel p = img.getPixel(i * 30 + 1, i * 30 + 1);
            System.out.println(p.getRed() + "," + p.getGreen() + "," + p.getBlue());
        }
    }
}
"#
);
