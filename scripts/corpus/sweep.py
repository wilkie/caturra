#!/usr/bin/env python3
"""Run a fetched corpus (see rosetta.py) through a JDK and through caturra.

Each program is first compiled with `javac --release 11`: one that needs a
later Java, or a library outside the JDK, is OUT OF SCOPE and counted apart
— it says nothing about caturra. The rest run on the JDK twice (a program
whose output differs between two runs — time, randomness, hash order — is
set aside as nondeterministic), then on caturra, and each lands in one bucket:

  same            stdout and the outcome agree
  refused         caturra would not compile it (grouped by its first message)
  runtime-refused caturra compiled it but stopped with a reason of its own
  differs         both ran; stdout or the outcome differs
  slow            the JDK finished; caturra did not within its limit
  crashed         caturra's harness failed outright

    python3 scripts/corpus/sweep.py [--corpus DIR] [--jobs N] [--only SUBSTR] [--list BUCKET]
"""
import argparse
import collections
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from concurrent.futures import ProcessPoolExecutor

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
COMPATRUN = os.path.join(REPO, "target", "release", "examples", "compatrun")
DEFAULT = os.path.expanduser("~/.cache/caturra-corpus/rosetta")
JDK_LIMIT, CATURRA_LIMIT = 10, 60


def run_jdk(work, main):
    try:
        r = subprocess.run(
            ["java", "-Djava.awt.headless=true", "-Xss8m", "-cp", "out", main],
            cwd=work, capture_output=True, text=True, timeout=JDK_LIMIT, stdin=subprocess.DEVNULL,
        )
    except subprocess.TimeoutExpired:
        return None
    except UnicodeDecodeError:
        return None
    if "java.awt.HeadlessException" in r.stderr:
        return "headless"
    return r.stdout, r.returncode == 0


def first_line(text):
    text = (text or "").strip()
    return text.splitlines()[0] if text else ""


def examine(folder):
    meta = json.load(open(os.path.join(folder, "meta.json")))
    name = os.path.basename(folder)
    source = os.path.join(folder, meta["file"])
    with tempfile.TemporaryDirectory() as work:
        shutil.copy(source, work)
        path = os.path.join(work, meta["file"])
        compiled = subprocess.run(
            ["javac", "--release", "11", "-nowarn", "-d", "out", meta["file"]],
            cwd=work, capture_output=True, text=True, timeout=120,
        )
        if compiled.returncode:
            return name, "out-of-scope", first_line(re.sub(r"^\S+\.java:\d+: ", "", compiled.stderr))
        # A `package` line puts the class in that package, and a JDK runs it
        # by its QUALIFIED name.
        package = re.search(r"^\s*package\s+([\w.]+)\s*;", open(path, encoding="utf-8").read(), re.M)
        main_class = f"{package.group(1)}.{meta['main']}" if package else meta["main"]
        first = run_jdk(work, main_class)
        if first is None:
            return name, "out-of-scope", "the JDK run does not finish in time"
        # A GUI program on a headless JDK dies before it does anything: it
        # tests nothing here, whatever caturra makes of it.
        if first == "headless":
            return name, "out-of-scope", "needs a display (HeadlessException on the JDK)"
        second = run_jdk(work, main_class)
        if second != first:
            return name, "nondeterministic", ""
        jdk_out, jdk_ok = first
        try:
            r = subprocess.run(
                [COMPATRUN, path, meta["main"]], capture_output=True, text=True,
                timeout=CATURRA_LIMIT, stdin=subprocess.DEVNULL,
            )
        except subprocess.TimeoutExpired:
            return name, "slow", ""
        try:
            got = json.loads(r.stdout)
        except ValueError:
            return name, "crashed", first_line(r.stderr) or first_line(r.stdout)
        out, ok, error = got.get("stdout", ""), got.get("ok", False), got.get("error", "")
        if "line" in got and not ok and not out:
            return name, "refused", first_line(error)
        if out == jdk_out and ok == jdk_ok:
            return name, "same", ""
        uncaught = error.startswith("uncaught exception")
        if not ok and not uncaught and jdk_ok:
            return name, "runtime-refused", first_line(error)
        return name, "differs", first_line(error) if not ok else "stdout differs"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", default=DEFAULT)
    parser.add_argument("--jobs", type=int, default=6)
    parser.add_argument("--only", default="")
    parser.add_argument("--list", default="", help="print every program in this bucket")
    parser.add_argument("--json", default="", help="write every result here")
    args = parser.parse_args()
    folders = sorted(
        os.path.join(args.corpus, d) for d in os.listdir(args.corpus)
        if not d.startswith("_") and args.only in d
        and os.path.exists(os.path.join(args.corpus, d, "meta.json"))
    )
    buckets = collections.Counter()
    reasons = collections.defaultdict(collections.Counter)
    results = []
    with ProcessPoolExecutor(args.jobs) as pool:
        for name, bucket, detail in pool.map(examine, folders):
            buckets[bucket] += 1
            results.append((name, bucket, detail))
            if bucket in ("refused", "runtime-refused", "differs", "crashed", "out-of-scope"):
                # Group messages by shape, not by the names in them.
                shape = re.sub(r"'[^']*'|\b[A-Z]\w*\b|\d+", "…", detail)[:110]
                reasons[bucket][shape] += 1
    in_scope = sum(v for k, v in buckets.items() if k not in ("out-of-scope", "nondeterministic"))
    print(f"{len(folders)} programs; {in_scope} in scope (Java 11, deterministic, finish on a JDK)")
    for bucket in ("same", "refused", "runtime-refused", "differs", "slow", "crashed", "nondeterministic", "out-of-scope"):
        if buckets[bucket]:
            print(f"  {bucket:16} {buckets[bucket]}")
    for bucket in ("refused", "runtime-refused", "differs", "crashed"):
        if reasons[bucket]:
            print(f"\n{bucket}, by message shape:")
            for shape, count in reasons[bucket].most_common(25):
                print(f"  {count:4}  {shape}")
    if args.list:
        print(f"\n{args.list}:")
        for name, bucket, detail in results:
            if bucket == args.list:
                print(f"  {name}: {detail}")
    if args.json:
        json.dump(results, open(args.json, "w"), indent=1)


if __name__ == "__main__":
    main()
