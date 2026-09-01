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

    scripts/coverage/course.py [--verbose]

Needs `javac`/`java` on PATH, `vendor/sweep-classes` built
(`scripts/sweep/build-reference.py`), and a built
`target/release/examples/diagnostics`.
"""
import os, subprocess, sys, tempfile
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ENGINE = os.path.join(REPO, "target/release/examples/diagnostics")
CLASSES = os.path.join(REPO, "vendor/sweep-classes")

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
    "org.code.validation.PainterLog": ("new org.code.validation.PainterLog()", ""),
    "org.code.validation.PainterEvent": ("new org.code.validation.PainterEvent()", ""),
    "org.code.validation.NeighborhoodLog": ("new org.code.validation.NeighborhoodLog()", ""),
    "org.code.validation.Position": ("new org.code.validation.Position()", ""),
    "org.code.validation.NeighborhoodActionType": (
        "org.code.validation.NeighborhoodActionType.MOVE",
        "",
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


def measure(class_name, receiver, imports, methods):
    """(known, missing) names for one class, read from caturra's diagnostics."""
    simple = class_name.rsplit(".", 1)[1]
    lines, order = [], []
    for name, is_static, arity in methods:
        target = simple if is_static or receiver == "STATIC" else receiver
        lines.append(f"        {target}.{name}({', '.join(['null'] * arity)});")
        order.append(name)
    source = (
        f"{imports}\npublic class Probe {{\n    public static void main(String[] args) {{\n"
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


def main():
    verbose = "--verbose" in sys.argv
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
