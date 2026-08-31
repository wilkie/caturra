#!/usr/bin/env python3
"""Compare the WHOLE diagnostic list of each case with javac's.

`run.py` compares what a program prints, and `compatrun` reports only the
first error — so how many errors caturra reports, and in what order, went
unmeasured. javac has phases, and they are visible: it runs flow analysis only
when attribution produced no errors, and attributes nothing that failed to
parse. A program with a type error AND a missing return is reported by javac
with the type error alone, which is the FIRST thing a student sees.

    scripts/fuzz/diaglist.py <cases-dir>

Each `.java` file in the directory is compiled by both, and the ordered list of
error messages is compared. Exit status is 1 if anything differs.
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
    with tempfile.TemporaryDirectory() as out:
        result = subprocess.run(
            ["javac", "-d", out, path], capture_output=True, text=True, timeout=120
        )
    return [
        line.split("error: ", 1)[1].strip()
        for line in result.stderr.splitlines()
        if ": error: " in line
    ]


def caturra_errors(path):
    result = subprocess.run(
        [ENGINE, os.path.abspath(path)], capture_output=True, text=True, cwd=REPO, timeout=120
    )
    return [
        line.split("Error: ", 1)[1].strip()
        for line in result.stdout.splitlines()
        if line.startswith("Error: ")
    ]


def agrees(want, got):
    """Whether caturra's list says what javac's does.

    Most of caturra's messages are javac's headline plus the detail javac
    prints on its indented `symbol:`/`required:` continuation lines, which this
    only ever sees the first line of. A caturra line that STARTS with javac's
    is that convention, not a divergence — the wording that names the mistake
    is the same, and the rest is a continuation line moved inline.
    """
    return len(want) == len(got) and all(c.startswith(j) for j, c in zip(want, got))


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    directory = sys.argv[1]
    build_engine()
    cases = sorted(f for f in os.listdir(directory) if f.endswith(".java"))
    differing = 0
    for case in cases:
        path = os.path.join(directory, case)
        want, got = javac_errors(path), caturra_errors(path)
        if agrees(want, got):
            continue
        differing += 1
        print(f"--- {case[:-5]}")
        print(f"    jdk: {want}")
        print(f"    cat: {got}")
    print(f"{len(cases)} cases, {differing} differing")
    return 1 if differing else 0


if __name__ == "__main__":
    sys.exit(main())
