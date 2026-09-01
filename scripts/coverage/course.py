#!/usr/bin/env python3
"""How much of the COURSE library can a program call?

`measure.py` asks that of `java.*`. This asks it of `org.code` — the library
the Code.org curriculum is written against, which matters more to a student
here than the tail of `java.util`. The denominator is Code.org's OWN source,
by reflection over the classes `scripts/sweep/build-reference.py` compiles into
`vendor/sweep-classes` from the vendored `javabuilder/` checkout; the numerator
is what caturra answers for a call to each name.

Three names are excluded and named below rather than counted as gaps: they
hand back `java.awt` types (a `BufferedImage`, an AWT `Color`), which have no
meaning in a browser and no student-facing use.

    scripts/coverage/course.py [--verbose] [--semantic]

`--semantic` asks the OTHER question: of the names caturra answers, which ones
has a differential test ever run against the real library? Compiling is not
behaving — `Pixel.getSourceImage` would have counted as covered by the first
question the moment it existed, and only the second one proves it hands back
the same image. This is a text mention of `name(` in the suites, which is a
heuristic and is labelled as one: it can say a name is UNCOVERED with
confidence, and "covered" only means some program calls it.

Needs `javac`/`java` on PATH, `vendor/sweep-classes` built
(`scripts/sweep/build-reference.py`), and a built
`target/release/examples/diagnostics`.
"""
import os, subprocess, sys, tempfile
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ENGINE = os.path.join(REPO, "target/release/examples/diagnostics")
CLASSES = os.path.join(REPO, "vendor/sweep-classes")

# `org.code.validation` replays the student's program, so a probe that names it
# needs one to replay: the bundle's test runner calls `Main.main`.
MAIN = "class Main { public static void main(String[] args) { } }\n"

# `NeighborhoodTestRunner` reads the neighborhood's action log, so the runner
# is injected only when that package is reachable too — a probe that imports
# the validation package alone does not see it.
VALIDATION_IMPORTS = "import org.code.validation.*;\nimport org.code.neighborhood.*;"

# One expression of each type, and the imports a probe needs to write it.
# `STATIC` means the class's own name is the receiver.
RECEIVERS = {
    "org.code.neighborhood.Painter": ("new Painter()", "import org.code.neighborhood.Painter;"),
    "org.code.media.Image": ('new Image("a.png")', "import org.code.media.Image;"),
    "org.code.media.Color": ("Color.RED", "import org.code.media.Color;"),
    "org.code.media.Pixel": (
        'new Image("a.png").getPixel(0, 0)',
        "import org.code.media.Image;\nimport org.code.media.Pixel;",
    ),
    "org.code.media.Font": ("Font.SANS", "import org.code.media.Font;"),
    "org.code.media.FontStyle": ("FontStyle.BOLD", "import org.code.media.FontStyle;"),
    "org.code.theater.Theater": ("STATIC", "import org.code.theater.Theater;"),
    "org.code.theater.Scene": ("new Scene()", "import org.code.theater.Scene;"),
    "org.code.theater.Instrument": ("Instrument.PIANO", "import org.code.theater.Instrument;"),
    "org.code.validation.ValidationHelper": (
        "STATIC",
        "import org.code.validation.ValidationHelper;",
    ),
    "org.code.validation.NeighborhoodLog": (
        "NeighborhoodTestRunner.run()",
        VALIDATION_IMPORTS,
    ),
    "org.code.validation.PainterLog": (
        "NeighborhoodTestRunner.run().getPainterLogs()[0]",
        VALIDATION_IMPORTS,
    ),
    "org.code.validation.PainterEvent": (
        "NeighborhoodTestRunner.run().getPainterLogs()[0].getEvents().get(0)",
        VALIDATION_IMPORTS,
    ),
    "org.code.validation.Position": (
        "NeighborhoodTestRunner.run().getPainterLogs()[0].getStartingPosition()",
        VALIDATION_IMPORTS,
    ),
    "org.code.validation.NeighborhoodActionType": (
        "NeighborhoodActionType.MOVE",
        VALIDATION_IMPORTS,
    ),
}

# Real names of the real library that caturra will not answer, with the reason.
# Counting them as gaps would say the model is incomplete where it is bounded.
EXCLUDED = {
    ("org.code.media.Image", "getBufferedImage"): "hands back a java.awt BufferedImage",
    ("org.code.media.Image", "getImageAssetFromFile"): "loads through the AWT decoder",
    ("org.code.media.Color", "convertToAWTColor"): "hands back a java.awt Color",
}


def inventory():
    """Every public method of the course classes, from Code.org's own source."""
    with tempfile.TemporaryDirectory() as directory:
        subprocess.run(
            ["javac", "-d", directory, os.path.join(REPO, "scripts/coverage/ApiList.java")],
            check=True, capture_output=True,
        )
        listing = subprocess.run(
            ["java", "-cp", f"{directory}:{CLASSES}", "ApiList", *RECEIVERS],
            check=True, capture_output=True, text=True, timeout=300,
        ).stdout
    api = defaultdict(list)
    for line in listing.splitlines():
        class_name, name, is_static, arity = line.split("\t")
        api[class_name].append((name, is_static == "true", int(arity)))
    return api


def check_receiver(class_name, receiver, imports):
    """The receiver EXPRESSION must compile on its own.

    A receiver that does not is a broken probe, not a missing method: every
    call written on it fails, and if the failure does not say "cannot find
    symbol" the whole class reads as fully supported. That is how
    `PainterEvent` scored 2/2 while `getDetails` did not exist — the receiver
    named a constructor caturra does not have.
    """
    if receiver == "STATIC":
        return
    source = (
        f"{imports}\n{MAIN}public class Probe {{\n    public static void main(String[] args) {{\n"
        f"        Object __receiver = {receiver};\n    }}\n}}\n"
    )
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        done = subprocess.run([ENGINE, path], capture_output=True, text=True, cwd=REPO)
    if done.returncode != 0 or "Error@" in done.stdout:
        raise RuntimeError(
            f"the {class_name} probe's receiver does not compile — fix the table, "
            f"not the engine:\n  {receiver}\n  {done.stdout.strip()[:200]}"
        )


def measure(class_name, receiver, imports, methods):
    """(known, missing) names for one class, read from caturra's diagnostics."""
    check_receiver(class_name, receiver, imports)
    simple = class_name.rsplit(".", 1)[1]
    lines, order = [], []
    for name, is_static, arity in methods:
        target = simple if is_static or receiver == "STATIC" else receiver
        lines.append(f"        {target}.{name}({', '.join(['null'] * arity)});")
        order.append(name)
    source = (
        f"{imports}\n{MAIN}public class Probe {{\n    public static void main(String[] args) {{\n"
        + "\n".join(lines)
        + "\n    }\n}\n"
    )
    first = source.count("\n", 0, source.index("public class Probe")) + 3
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        done = subprocess.run([ENGINE, path], capture_output=True, text=True, cwd=REPO)
    if done.returncode != 0:
        raise RuntimeError(f"the engine failed on the {class_name} probe: {done.stderr[:300]}")
    unknown = set()
    for line in done.stdout.splitlines():
        if not line.startswith("Error@"):
            continue
        where, message = line.split(": ", 1)
        index = int(where.split("@", 1)[1].split(",")[0]) - first
        if 0 <= index < len(order) and (
            "cannot find symbol" in message or "not supported by caturra" in message
        ):
            unknown.add(index)
    known = {name for index, name in enumerate(order) if index not in unknown}
    missing = {name for index, name in enumerate(order) if index in unknown} - known
    return sorted(known), sorted(missing)


# The suites that run a program through the REAL library and through caturra.
SUITES = (
    "crates/caturra-vm/tests/differential_media.rs",
    "crates/caturra-vm/tests/differential_neighborhood.rs",
    "crates/caturra-vm/tests/differential_validation_orgcode.rs",
)

# What no stdout comparison can reach. `org.code.theater` draws: the real
# library's output is a GIF written through an AWS content manager, and
# caturra's is a canvas in the browser. The e2e tests drive that; a differential
# test cannot.
UNCOMPARABLE = ("org.code.theater.",)


def semantic():
    """Which answered names a differential suite has ever run."""
    text = ""
    for suite in SUITES:
        path = os.path.join(REPO, suite)
        if os.path.isfile(path):
            text += open(path).read()
    api = inventory()
    covered = uncovered = skipped = 0
    for class_name in RECEIVERS:
        names = sorted({name for name, _, _ in api.get(class_name, [])})
        names = [n for n in names if (class_name, n) not in EXCLUDED]
        if class_name.startswith(UNCOMPARABLE):
            skipped += len(names)
            print(f"{class_name:44} {len(names):3} names — not comparable by stdout")
            continue
        run = [n for n in names if f"{n}(" in text]
        missing = [n for n in names if f"{n}(" not in text]
        covered += len(run)
        uncovered += len(missing)
        print(f"{class_name:44} {len(run):3}/{len(names):3} run against the real library")
        if missing:
            print(f"    never run: {' '.join(missing)}")
    total = covered + uncovered
    share = 100.0 * covered / total if total else 0.0
    print(
        f"\n{covered}/{total} names ({share:.1f}%) have been run against the real "
        f"library; {skipped} draw, and no stdout comparison reaches them"
    )
    return 0 if uncovered == 0 else 1


def main():
    verbose = "--verbose" in sys.argv
    if "--semantic" in sys.argv:
        if not os.path.isdir(CLASSES):
            sys.exit("no vendor/sweep-classes — run scripts/sweep/build-reference.py first")
        return semantic()
    if not os.path.isdir(CLASSES):
        sys.exit("no vendor/sweep-classes — run scripts/sweep/build-reference.py first")
    api = inventory()
    total_known = total_all = 0
    excluded_seen = []
    for class_name, (receiver, imports) in RECEIVERS.items():
        methods = api.get(class_name, [])
        known, missing = measure(class_name, receiver, imports, methods)
        bounded = [name for name in missing if (class_name, name) in EXCLUDED]
        missing = [name for name in missing if (class_name, name) not in EXCLUDED]
        excluded_seen += [(class_name, name) for name in bounded]
        counted = len(known) + len(missing)
        total_known += len(known)
        total_all += counted
        share = 100.0 * len(known) / counted if counted else 0.0
        print(f"{class_name:44} {len(known):3}/{counted:3}  {share:5.1f}%")
        if missing:
            print(f"    missing: {' '.join(missing)}")
        if verbose and bounded:
            for name in bounded:
                print(f"    bounded: {name} — {EXCLUDED[(class_name, name)]}")
    share = 100.0 * total_known / total_all if total_all else 0.0
    print(
        f"\n{len(RECEIVERS)} course classes, {total_known}/{total_all} method names "
        f"({share:.1f}%), {len(excluded_seen)} bounded by design"
    )
    return 0 if total_known == total_all else 1


if __name__ == "__main__":
    sys.exit(main())
