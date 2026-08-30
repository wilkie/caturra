#!/usr/bin/env python3
"""Diff what a corpus level PRINTS: caturra against a real JDK, level by level.

`compare.py` diffs the two engines' grading verdicts. A verdict is a coarse
signal — a level whose solution prints the wrong number still passes every test
that does not look at that number — so this is the other half: run each staged
level's `main` on both engines with the same stdin and the same data files, and
compare the console byte for byte.

It compares the FAILURE as well as the output. A level whose `main` throws is
not skipped: the exception a student sees is console text too, and comparing
only the levels that complete would leave the largest category of Scanner-driven
levels (`No line found`, because a sweep has no keyboard) unchecked.

**The one divergence to expect** is a listing of `getDeclaredConstructors()` /
`getDeclaredMethods()`. The JVM returns those "in no particular order" (it is
HotSpot's internal method array, ordered by symbol address); caturra returns
declaration order. The two agree on the SET and differ on the sequence, so this
script says so rather than counting it. **The number to watch is the
"unexplained" count it prints at the end.**

usage: stdout.py [--cases vendor/sweep-cases] [--jobs 8] [--only 0500 ...]
"""
import argparse
import collections
import concurrent.futures
import glob
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

# A level that draws randomness prints something different every run, on either
# engine. The same rule `compare.py` applies to a validator, applied to a whole
# level: here the SOLUTION's randomness matters too, not just the test's.
NONDETERMINISM = re.compile(
    r"Math\s*\.\s*random\s*\(|new\s+Random\s*\(\s*\)|currentTimeMillis|nanoTime"
    r"|identityHashCode|System\s*\.\s*getenv|getProperty"
)
# A program that prints a reflective listing prints it in an order the JVM does
# not specify. Recognised so the report can separate it from a real difference.
REFLECTIVE = re.compile(r"getDeclaredConstructors|getConstructors|getDeclaredMethods|getMethods")
IDENTITY_HASH = re.compile(rb"@[0-9a-f]+")
THROWN = re.compile(r'Exception in thread "main" (.*)')


def sources(case):
    """The level's program: every source but its tests."""
    return [path for path in sorted(glob.glob(os.path.join(case, "*.java")))
            if "Test.java" not in path]


def main_class(paths):
    for path in paths:
        text = pathlib.Path(path).read_text(errors="replace")
        if "static void main" in text:
            return os.path.basename(path)[:-len(".java")]
    return None


def run_level(job):
    case, name, binary, work = job
    paths = sources(case)
    entry = main_class(paths)
    if not entry:
        return None
    text = "\n".join(pathlib.Path(path).read_text(errors="replace") for path in paths)
    if "org.code" in text:
        return None

    staged = os.path.join(work, os.path.basename(case))
    os.makedirs(staged, exist_ok=True)
    for path in paths + glob.glob(os.path.join(case, "*.txt")) + glob.glob(os.path.join(case, "*.csv")):
        shutil.copy(path, staged)
    compiled = [os.path.join(staged, os.path.basename(path)) for path in paths]
    javac = subprocess.run(["javac", "-nowarn", "-d", staged] + compiled,
                           capture_output=True, text=True)
    if javac.returncode != 0:
        # javac rejects it: the COMPILE sweep's business, not this one's.
        return None
    try:
        jdk = subprocess.run(["java", "-cp", staged, entry], capture_output=True,
                             timeout=30, input=b"", cwd=staged)
    except subprocess.TimeoutExpired:
        return None
    extras = [path for path in compiled if os.path.basename(path) != entry + ".java"]
    try:
        cat = subprocess.run([binary, os.path.join(staged, entry + ".java"), entry] + extras,
                             capture_output=True, timeout=120, cwd=staged)
    except subprocess.TimeoutExpired:
        return (name, "TIMEOUT", "", "caturra did not finish")
    try:
        payload = json.loads(cat.stdout.decode("utf8", "replace"))
    except ValueError:
        return (name, "NOJSON", "", cat.stdout.decode("utf8", "replace")[:200])

    thrown = THROWN.search(jdk.stderr.decode("utf8", "replace"))
    want = (IDENTITY_HASH.sub(b"@X", jdk.stdout).decode("utf8", "replace"),
            thrown.group(1).strip() if thrown else "")
    # caturra prefixes what escaped `main`; a JDK writes the class and message
    # alone on the `Exception in thread "main"` line. Same failure, two shapes.
    error = payload.get("error", "").split("\n")[0].strip()
    error = error[len("uncaught exception: "):] if error.startswith("uncaught exception: ") else error
    got = (IDENTITY_HASH.sub(b"@X", payload.get("stdout", "").encode()).decode("utf8", "replace"),
           "" if payload.get("ok") else error)
    if want == got:
        return None

    if NONDETERMINISM.search(text):
        verdict = "random"
    elif sorted(want[0].split("\n")) == sorted(got[0].split("\n")) and REFLECTIVE.search(text):
        verdict = "reflective order"
    else:
        verdict = "UNEXPLAINED"
    return (name, verdict) + window(want, got)


def window(want, got):
    """The first place the two consoles part company, with what led up to it —
    a 400-character prefix of two long identical listings says nothing."""
    if want[0] != got[0]:
        left, right = want[0].split("\n"), got[0].split("\n")
        at = next((i for i in range(max(len(left), len(right)))
                   if left[i:i + 1] != right[i:i + 1]), 0)
        head = f"line {at + 1}: "
        return (head + repr("\n".join(left[at:at + 3]))[:200],
                head + repr("\n".join(right[at:at + 3]))[:200])
    return (repr(want[1])[:200], repr(got[1])[:200])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cases", default="vendor/sweep-cases")
    parser.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    parser.add_argument("--only", nargs="*", default=None,
                        help="staged case directories to run, e.g. 0500")
    options = parser.parse_args()

    build = subprocess.run(["cargo", "build", "--release", "-p", "caturra-vm",
                            "--example", "compatrun"])
    if build.returncode != 0:
        return 1
    binary = os.path.abspath("target/release/examples/compatrun")

    cases = []
    for case in sorted(pathlib.Path(options.cases).iterdir()):
        if not (case / "name").is_file():
            continue
        if options.only and case.name not in options.only:
            continue
        cases.append((str(case), (case / "name").read_text().strip()))
    print(f"{len(cases)} staged levels", file=sys.stderr)

    rows = []
    with tempfile.TemporaryDirectory(prefix="sweep-stdout-") as work:
        jobs = [(case, name, binary, work) for case, name in cases]
        with concurrent.futures.ThreadPoolExecutor(options.jobs) as pool:
            for done, row in enumerate(pool.map(run_level, jobs), 1):
                if done % 200 == 0:
                    print(f"  {done}/{len(jobs)}", file=sys.stderr)
                if row:
                    rows.append(row)

    counts = collections.Counter(row[1] for row in rows)
    for name, verdict, want, got in sorted(rows):
        if verdict != "reflective order":
            print(f"{verdict:16} {name}\n    jdk: {want}\n    cat: {got}")
    print()
    for verdict, count in sorted(counts.items()):
        print(f"{count:5} {verdict}")
    print(f"\nunexplained: {counts['UNEXPLAINED'] + counts['TIMEOUT'] + counts['NOJSON']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
