#!/usr/bin/env python3
"""Compare the WHOLE diagnostic list of each case with javac's.

`run.py` compares what a program prints, and `compatrun` reports only the
first error — so how many errors caturra reports, and in what order, went
unmeasured. javac has phases, and they are visible: it runs flow analysis only
when attribution produced no errors, and attributes nothing that failed to
parse. A program with a type error AND a missing return is reported by javac
with the type error alone, which is the FIRST thing a student sees.

    scripts/fuzz/diaglist.py <cases-dir> [--columns]

Each `.java` file in the directory is compiled by both, and the ordered list of
error messages is compared, with the LINE each is reported on. Exit status is 1
if anything differs.

`--columns` reports how often the COLUMN agrees as well. It is not part of the
comparison: javac has a convention per diagnostic for which token its caret
sits under — the argument whose type is wrong, the dot of a missing member,
the `<` of a wrong type-argument list — and caturra matches about half of them.
The count is there to be moved, not to gate.
"""
import os
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ENGINE = os.path.join(REPO, "target/release/examples/diagnostics")


def build_engine():
    built = subprocess.run(
        ["cargo", "build", "--release", "-p", "caturra-vm", "--example", "diagnostics"],
        cwd=REPO,
    )
    if built.returncode != 0:
        sys.exit("the engine did not build")


def javac_errors(path):
    """javac's errors as `(line, column, message)`. The POSITION is compared
    too: a message can be right and point at the wrong place, and an editor
    underlines the place. The column comes from the caret line javac prints
    two lines below each error; it is `None` when there is no caret."""
    with tempfile.TemporaryDirectory() as out:
        result = subprocess.run(
            ["javac", "-d", out, path], capture_output=True, text=True, timeout=120
        )
    lines = result.stderr.splitlines()
    found = []
    for index, line in enumerate(lines):
        if ": error: " not in line:
            continue
        where, message = line.split(": error: ", 1)
        number = where.rsplit(":", 1)[-1]
        caret = lines[index + 2] if index + 2 < len(lines) else ""
        column = len(caret) - len(caret.lstrip()) + 1 if caret.strip() == "^" else None
        found.append((int(number) if number.isdigit() else 0, column, message.strip()))
    return found


def caturra_errors(path):
    result = subprocess.run(
        [ENGINE, os.path.abspath(path)], capture_output=True, text=True, cwd=REPO, timeout=120
    )
    # A CRASH prints no diagnostics, and a program javac accepts has none
    # either — so a panic on valid code read as perfect agreement here. The
    # engine's exit code is the only thing that tells them apart.
    if result.returncode != 0:
        raise RuntimeError(
            f"the engine CRASHED on {path} (exit {result.returncode}): "
            f"{result.stderr.strip()[:400]}"
        )
    found = []
    for line in result.stdout.splitlines():
        if not line.startswith("Error@"):
            continue
        where, message = line.split(": ", 1)
        line_text, column_text = where.split("@", 1)[1].split(",")
        found.append((int(line_text), int(column_text), message.strip()))
    return found


def agrees(want, got):
    """Whether caturra's list says what javac's does.

    Most of caturra's messages are javac's headline plus the detail javac
    prints on its indented `symbol:`/`required:` continuation lines, which this
    only ever sees the first line of. A caturra line that STARTS with javac's
    is that convention, not a divergence — the wording that names the mistake
    is the same, and the rest is a continuation line moved inline.
    """
    return len(want) == len(got) and all(
        want_line == got_line and got_message.startswith(want_message)
        for (want_line, _, want_message), (got_line, _, got_message) in zip(want, got)
    )


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    directory = sys.argv[1]
    columns = "--columns" in sys.argv
    build_engine()
    cases = sorted(f for f in os.listdir(directory) if f.endswith(".java"))
    differing = 0
    column_same = column_differ = 0
    for case in cases:
        path = os.path.join(directory, case)
        want, got = javac_errors(path), caturra_errors(path)
        if columns and len(want) == len(got):
            for (wl, wc, wm), (gl, gc, gm) in zip(want, got):
                if wc is None or wl != gl or not gm.startswith(wm):
                    continue
                if wc == gc:
                    column_same += 1
                else:
                    column_differ += 1
        if agrees(want, got):
            continue
        differing += 1
        print(f"--- {case[:-5]}")
        print(f"    jdk: {want}")
        print(f"    cat: {got}")
    print(f"{len(cases)} cases, {differing} differing")
    if columns:
        print(f"columns: {column_same} agree, {column_differ} differ")
    return 1 if differing else 0


if __name__ == "__main__":
    sys.exit(main())
