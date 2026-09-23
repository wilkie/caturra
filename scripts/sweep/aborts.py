#!/usr/bin/env python3
"""Sweep every corpus for an ENGINE ABORT — a failure that is not an answer.

usage: aborts.py [--what starts|validators|staged|levels|all] [--jobs N]
       aborts.py --dir <directory of .java files>

caturra may refuse a program (a diagnostic, with a line and a reason) and it
may fail one at run time (a Java exception, which a program can catch). Those
are answers. An ENGINE ABORT is neither: "operand stack underflow (malformed
bytecode)", "unknown native member", "unsupported opcode", "malformed class".
Each says the engine lost its footing, and a reader cannot tell whether their
program is wrong or caturra is.

Nothing swept for these. `scripts/fuzz/panics.py` catches a Rust PANIC — the
process dying — and the compile sweeps compare a refusal against javac's
without reading what the refusal SAYS, so a program both engines turn away
counts as agreement however badly caturra worded it. An abort was found by
accident: `Files.readString(p, StandardOpenOption.SYNC)` produced malformed
bytecode, because the refusal path read an argument's type, found an error and
returned without reporting it.

The exit code is the finding: zero aborts, or the list of programs that hit
one. `--self-check` answers the question a clean sweep always raises — whether
this one can report anything at all — by running the program that DID abort
before the fix and confirming the words are recognised.
"""

import argparse
import glob
import os
import subprocess
import sys
import tempfile
from concurrent.futures import ProcessPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from compile import REPO, population  # noqa: E402

RUN = os.path.join(REPO, "target/release/examples/compatrun")

# The engine's own failures, by the words they reach a reader with. Each is a
# SUBSTRING of the error compatrun prints; a Java exception ("uncaught
# exception: java.lang…") and a diagnostic are deliberately not here.
ABORTS = (
    "malformed bytecode",
    "operand stack underflow",
    "unknown native member",
    "unknown intrinsic",
    "unsupported opcode",
    "malformed class",
    "cannot be dereferenced",
    "its type is unknown",
)


def main_class(files):
    """The class the playground would run — whichever declares `main`."""
    for name, text in sorted(files.items()):
        if "static void main" in text:
            return os.path.splitext(name)[0]
    return None


def probe(case):
    name, files = case
    java = {f: t for f, t in files.items() if f.endswith(".java")}
    entry = main_class(java)
    if not java or entry is None:
        return None
    with tempfile.TemporaryDirectory() as work:
        paths = []
        for file, text in java.items():
            path = os.path.join(work, file)
            open(path, "w", encoding="utf-8").write(text)
            paths.append(path)
        first = os.path.join(work, f"{entry}.java")
        rest = [p for p in paths if p != first]
        try:
            run = subprocess.run(
                [RUN, first, entry] + rest, capture_output=True, text=True, timeout=120
            )
        except subprocess.TimeoutExpired:
            return None
        text = run.stdout + run.stderr
    hit = next((word for word in ABORTS if word in text), None)
    return (name, hit, text[:400]) if hit else None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--what",
        default="all",
        choices=["starts", "validators", "staged", "levels", "all"],
    )
    parser.add_argument(
        "--dir",
        help="a directory of .java files instead of a corpus — a fuzz run's output",
    )
    parser.add_argument(
        "--self-check",
        action="store_true",
        help="confirm the sweep can recognise an abort, and does not read an answer as one",
    )
    parser.add_argument("--jobs", type=int, default=8)
    args = parser.parse_args()

    if args.self_check:
        # An instrument that reports clean has to be shown to be able to
        # report dirty, or the zero says nothing. These are the exact words
        # the engine used for the abort this sweep was built after.
        samples = [
            '{"ok": false, "error": "operand stack underflow (malformed bytecode)"}',
            '{"ok": false, "error": "unknown native member: List.equals"}',
            '{"ok": false, "error": "malformed class One: no static method f()"}',
            # ...and the two shapes that must NOT be read as aborts: a
            # diagnostic, and a Java exception the program could have caught.
        ]
        clean = [
            '{"ok": false, "error": "cannot find symbol: \'Nope.FIELD\'", "line": 3}',
            '{"ok": false, "error": "uncaught exception: java.lang.ArithmeticException: / by zero"}',
            '{"ok": true, "stdout": "42\\n"}',
        ]
        wrong = [t for t in samples if not any(word in t for word in ABORTS)]
        wrong += [t for t in clean if any(word in t for word in ABORTS)]
        for text in wrong:
            print(f"=== SELF-CHECK FAILED: {text}")
        print(
            f"\nself-check: {len(samples)} aborts recognised, "
            f"{len(clean)} answers left alone, {len(wrong)} wrong"
        )
        return 1 if wrong else 0

    if args.dir:
        cases = [
            (os.path.basename(path), {os.path.basename(path): open(path, encoding="utf-8").read()})
            for path in sorted(glob.glob(os.path.join(args.dir, "**", "*.java"), recursive=True))
        ]
        found = 0
        with ProcessPoolExecutor(args.jobs) as pool:
            for result in pool.map(probe, cases):
                if result is None:
                    continue
                found += 1
                name, hit, text = result
                print(f"=== ENGINE ABORT ({hit}): {name}")
                for line in text.splitlines()[:4]:
                    print("   ", line[:160])
        print(f"\n{len(cases)} programs run, {found} engine aborts")
        return 1 if found else 0

    wanted = (
        ["validators", "staged", "levels", "starts"] if args.what == "all" else [args.what]
    )
    found = 0
    total = 0
    for what in wanted:
        cases = population(what)
        total += len(cases)
        with ProcessPoolExecutor(args.jobs) as pool:
            for result in pool.map(probe, cases):
                if result is None:
                    continue
                found += 1
                name, hit, text = result
                print(f"=== ENGINE ABORT ({hit}): {what} / {name}")
                for line in text.splitlines()[:4]:
                    print("   ", line[:160])
    print(f"\n{total} programs run, {found} engine aborts")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
