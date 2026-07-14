#!/usr/bin/env python3
"""Ask a real JDK 11 and caturra what each snippet actually does, and write the
answers into `apps/playground/src/compat/features.json` — the manifest the
compatibility page publishes.

**Nothing here is a claim.** A feature is "supported" because caturra produced the
same bytes a real JDK produced, "unsupported" because javac ACCEPTED the program
and caturra rejected it (with the reason it gives), and "beyond-11" because javac
11 rejected it too. The page shows the recorded JDK output next to a live in-browser
run of the same program, and `tests/compat_manifest.rs` re-checks the whole thing
against the JDK on every CI run — so a claim here cannot quietly go stale.

usage: record.py [--check]     (--check reports disagreements without writing)
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from snippets import BEYOND, FEATURES, GAPS  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(REPO, "apps/playground/src/compat/features.json")
COMPATRUN = os.path.join(REPO, "target/release/examples/compatrun")


def jdk(source, main):
    """(compiles, ran_ok, output_or_error) from a real javac + java.

    Compiling and running are different questions, and the manifest turns on the
    first one: a gap is a program javac ACCEPTS and caturra rejects. Conflating
    them filed BufferedReader under "not Java 11" because the file it opens did
    not exist yet."""
    work = tempfile.mkdtemp(prefix="compat-")
    try:
        path = os.path.join(work, f"{main}.java")
        open(path, "w").write(source)
        compiled = subprocess.run(
            ["javac", "-nowarn", "-d", work, path],
            capture_output=True, text=True, timeout=120,
        )
        if compiled.returncode != 0:
            first = (compiled.stderr.strip().splitlines() or [""])[0]
            return False, False, first
        ran = subprocess.run(
            ["java", "-cp", work, main],
            capture_output=True, text=True, timeout=120, cwd=work,
        )
        if ran.returncode != 0:
            first = (ran.stderr.strip().splitlines() or [""])[0]
            return True, False, first
        return True, True, ran.stdout
    finally:
        shutil.rmtree(work, ignore_errors=True)


def caturra(source, main):
    """(ok, output_or_error) from the engine itself."""
    work = tempfile.mkdtemp(prefix="compat-")
    try:
        path = os.path.join(work, f"{main}.java")
        open(path, "w").write(source)
        ran = subprocess.run(
            [COMPATRUN, path, main],
            capture_output=True, text=True, timeout=180, cwd=work,
        )
        if ran.returncode != 0:
            return False, (ran.stderr.strip().splitlines() or ["engine crashed"])[0]
        result = json.loads(ran.stdout)
        if result["ok"]:
            return True, result["stdout"]
        return False, result["error"]
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="report, do not write")
    args = ap.parse_args()

    if not os.path.isfile(COMPATRUN):
        sys.exit("build it first: cargo build --release -p caturra-vm --example compatrun")

    entries = []
    problems = []
    for feature in FEATURES + GAPS + BEYOND:
        source = feature["source"].strip() + "\n"
        jdk_compiles, jdk_ok, jdk_out = jdk(source, feature["main"])
        cat_ok, cat_out = caturra(source, feature["main"])
        expected_kind = (
            "supported" if feature in FEATURES else "unsupported" if feature in GAPS else "beyond-11"
        )

        entry = dict(
            id=feature["id"],
            category=feature["category"],
            title=feature["title"],
            summary=feature["summary"],
            main=feature["main"],
            source=source,
        )

        if jdk_ok and cat_ok and jdk_out == cat_out:
            entry["status"] = "supported"
            entry["expected"] = jdk_out
        elif jdk_compiles and not cat_ok:
            # Real Java 11 (javac took it) that caturra turns away, saying why.
            entry["status"] = "unsupported"
            entry["reason"] = cat_out
        elif not jdk_compiles and not cat_ok:
            # javac 11 will not have it either — so neither should we.
            entry["status"] = "beyond-11"
            entry["reason"] = cat_out
            entry["javac"] = jdk_out
        else:
            entry["status"] = "DIVERGENT"
            entry["expected"] = jdk_out
            entry["actual"] = cat_out

        if entry["status"] != expected_kind:
            problems.append(
                f"  {feature['id']}: listed as {expected_kind}, engines say {entry['status']}\n"
                f"      jdk({jdk_ok}): {jdk_out.strip()[:150]!r}\n"
                f"      cat({cat_ok}): {cat_out.strip()[:150]!r}"
            )
        entries.append(entry)

    counts = {}
    for entry in entries:
        counts[entry["status"]] = counts.get(entry["status"], 0) + 1
    print("recorded:", counts)
    if problems:
        print("\nDISAGREEMENTS (the snippet list says one thing, the engines another):")
        print("\n".join(problems))

    if args.check:
        return 1 if problems else 0

    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w") as handle:
        json.dump(entries, handle, indent=2)
        handle.write("\n")
    subprocess.run(
        ["pnpm", "exec", "prettier", "--write", "--log-level", "warn", OUT],
        cwd=REPO, check=True,
    )
    print(f"wrote {OUT} ({len(entries)} features)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
