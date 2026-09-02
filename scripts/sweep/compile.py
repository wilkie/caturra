#!/usr/bin/env python3
"""Compile a POPULATION of Java files with caturra and with javac, and compare.

usage: compile.py [--what starts|validators|staged|levels] [--jobs N]

The grading sweep compiles one set: a level's `solution` merged with its
`start`, staged only when a solution exists. Everything outside that set was
unmeasured, and every population below hid a divergence when it was first
asked:

  `--what validators`  the 85 levels that ship a validator and NO solution.
                       Nothing had ever compiled them, and they are exactly
                       what a student's submission is graded against.
  `--what starts`      every level's `start` files ALONE — what a student
                       opens before typing anything. The staged sweep only
                       ever sees them merged with the solution.
  `--what staged`      the staged cases themselves (`vendor/sweep-cases`),
                       which the grading sweep runs but never compares
                       COMPILE-for-compile with javac.
  `--what levels`      the levels the playground SHIPS
                       (`apps/playground/src/csa-units/unit-*.ts`) — the only
                       population that is checked in rather than vendored,
                       and the one where a refusal is a level broken today.

A divergence in either direction is the finding. "caturra only" is a level a
student cannot compile here and can compile on a JDK; "javac only" is the
dangerous direction — a program caturra accepts that a JDK refuses.
"""

import argparse
import glob
import os
import re
import subprocess
import sys
import tempfile
from concurrent.futures import ProcessPoolExecutor

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DIAG = os.path.join(REPO, "target/release/examples/diagnostics")
CLASSPATH = ":".join(
    sorted(glob.glob(os.path.join(REPO, "vendor/junit/*.jar")))
    + [os.path.join(REPO, "vendor/sweep-classes")]
)


def rename_main(files):
    """The playground renames whichever class declares `main()` to `Main`,
    which is why the validators reference `Main`."""
    out = {}
    for name, text in files.items():
        if "static void main" in text:
            match = re.search(r"class\s+(\w+)", text[: text.index("static void main")])
            if match:
                text = re.sub(
                    r"(\bclass\s+)" + re.escape(match.group(1)) + r"\b", r"\1Main", text, count=1
                )
            out["Main.java"] = text
        else:
            out[name] = text
    return out


def read(paths):
    return {os.path.basename(p): open(p, encoding="utf-8").read() for p in sorted(paths)}


def js_string(text, at):
    """Read the single-quoted JS string starting at `text[at]`."""
    assert text[at] == "'"
    out, i = [], at + 1
    escapes = {"n": "\n", "t": "\t", "r": "\r", "\\": "\\", "'": "'", '"': '"'}
    while text[i] != "'":
        if text[i] == "\\":
            following = text[i + 1]
            if following == "u":
                out.append(chr(int(text[i + 2 : i + 6], 16)))
                i += 6
                continue
            out.append(escapes.get(following, following))
            i += 2
            continue
        out.append(text[i])
        i += 1
    # A JS string spells an astral character as a SURROGATE PAIR of `\u`
    # escapes; decoded one at a time they are lone surrogates, which no UTF-8
    # encoder will take.
    joined = "".join(out).encode("utf-16", "surrogatepass").decode("utf-16")
    return joined, i + 1


def shipped_levels():
    """(name, {file: text}) for every level the playground ships."""
    levels = []
    for path in sorted(glob.glob(os.path.join(REPO, "apps/playground/src/csa-units/unit-*.ts"))):
        text = open(path, encoding="utf-8").read()
        starts = [m for m in re.finditer(r"\{\s*\n\s*name: '", text)]
        for index, match in enumerate(starts):
            name, _ = js_string(text, match.end() - 1)
            end = starts[index + 1].start() if index + 1 < len(starts) else len(text)
            block = text[match.end() : end]
            files = {}
            for found in re.finditer(r"path: '([^']*)',\s*\n\s*text: '", block):
                body, _ = js_string(block, found.end() - 1)
                files[os.path.basename(found.group(1))] = body
            levels.append((name, files))
    return levels


def population(what):
    """(name, {file: text}) for each case in the chosen population."""
    if what == "levels":
        return shipped_levels()
    if what == "staged":
        cases = []
        for case in sorted(glob.glob(os.path.join(REPO, "vendor/sweep-cases/*"))):
            name_file = os.path.join(case, "name")
            name = open(name_file).read().strip() if os.path.exists(name_file) else case
            cases.append((name, read(glob.glob(os.path.join(case, "*.java")))))
        return cases
    cases = []
    for level in sorted(glob.glob(os.path.join(REPO, "artifacts", "*"))):
        validators = glob.glob(os.path.join(level, "validation", "*.java"))
        starts = glob.glob(os.path.join(level, "start", "*.java"))
        if not starts:
            continue
        if what == "validators":
            # The levels the grading sweep cannot stage: a validator, and no
            # solution for it to grade.
            if not validators or glob.glob(os.path.join(level, "solution", "*.java")):
                continue
            files = rename_main(read(starts))
            files.update(read(validators))
        else:
            files = read(starts)
        cases.append((os.path.basename(level), files))
    return cases


def compare(case):
    name, files = case
    java = {f: t for f, t in files.items() if f.endswith(".java")}
    if not java:
        return None
    with tempfile.TemporaryDirectory() as work:
        paths = []
        for file, text in java.items():
            path = os.path.join(work, file)
            open(path, "w", encoding="utf-8").write(text)
            paths.append(path)
        try:
            ours = subprocess.run([DIAG] + paths, capture_output=True, text=True, timeout=120)
            caturra = [l for l in ours.stdout.strip().splitlines() if l.startswith("Error@")]
        except subprocess.TimeoutExpired:
            caturra = ["the compiler did not finish"]
        out = os.path.join(work, "out")
        os.makedirs(out)
        jdk = subprocess.run(
            ["javac", "-nowarn", "-cp", CLASSPATH, "-d", out] + paths,
            capture_output=True,
            text=True,
        )
        javac = [l for l in jdk.stderr.splitlines() if " error: " in l]
    return name, caturra, javac


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--what", default="validators", choices=["starts", "validators", "staged", "levels"]
    )
    parser.add_argument("--jobs", type=int, default=8)
    args = parser.parse_args()

    cases = population(args.what)
    print(f"{len(cases)} cases in `{args.what}`", flush=True)
    both_ok = both_refuse = ours_only = jdk_only = 0
    with ProcessPoolExecutor(args.jobs) as pool:
        for result in pool.map(compare, cases):
            if result is None:
                continue
            name, caturra, javac = result
            if not caturra and not javac:
                both_ok += 1
            elif caturra and javac:
                both_refuse += 1
            elif caturra:
                ours_only += 1
                print(f"=== CATURRA ONLY: {name}")
                for line in caturra[:3]:
                    print("   ", line)
            else:
                jdk_only += 1
                print(f"=== JAVAC ONLY: {name}")
                for line in javac[:3]:
                    print("   ", line)
    print(
        f"\nboth compile {both_ok}, both refuse {both_refuse}, "
        f"caturra only {ours_only}, javac only {jdk_only}"
    )
    # A divergence in EITHER direction is a finding; "javac only" is the
    # dangerous one.
    return 1 if ours_only or jdk_only else 0


if __name__ == "__main__":
    sys.exit(main())
