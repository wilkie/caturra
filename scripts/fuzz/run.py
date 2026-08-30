#!/usr/bin/env python3
"""Run a directory of generated Java programs through BOTH engines and diff.

Each case is one `.java` file whose class name matches its file name, with an
optional `.stdin` beside it — or a DIRECTORY of `.java` files named for the one
holding `main` (`Pair/Pair.java` + `Pair/Helper.java`), which is how a real
program is written and the only way to probe what one file means in another. Every case is compiled ON ITS OWN (a batch compile
that fails writes no class files for the programs that were fine, which reads
as "every program diverged"), run on a real JDK, then run through caturra's
`compatrun`, and the two outputs are compared byte for byte.

What is compared: stdout, and — when the program ends by throwing — the
exception CLASS and message. The JDK writes that to stderr as
`Exception in thread "main" X: msg`; caturra reports it as JSON. Both are
reduced to the same one line, so a program that fails identically on both
engines is not a divergence.

Identity hashes (`@1b6d3586`) are normalized: two JVMs never agree on one, and
a program that prints one is comparing nothing.

    scripts/fuzz/run.py <cases-dir> [--keep-going]

Exit status is 1 if anything diverged, so it can gate a script.
"""
import json
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
IDENTITY = re.compile(r"@[0-9a-f]+")


def normalize(text):
    return IDENTITY.sub("@X", text)


def jdk_failure(stderr):
    line = next((l for l in stderr.splitlines() if l.startswith("Exception in thread")), "")
    return line.split('"main" ', 1)[-1].strip()


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    cases = sys.argv[1]
    out = os.path.join(cases, "..", "out")
    os.makedirs(out, exist_ok=True)
    # A case is either one `.java` file or a DIRECTORY of them named for the
    # class holding `main` — a real program is several files, and the
    # interesting bugs (a name that means one class here and another there,
    # access across a file boundary) need at least two.
    names = sorted(f[:-5] for f in os.listdir(cases) if f.endswith(".java"))
    names += sorted(d for d in os.listdir(cases)
                    if os.path.isdir(os.path.join(cases, d)) and d != "out"
                    and os.path.exists(os.path.join(cases, d, d + ".java")))
    rejected, diffs = [], []
    for name in sorted(names):
        directory = os.path.join(cases, name)
        if os.path.isdir(directory):
            sources = sorted(os.path.join(directory, f) for f in os.listdir(directory)
                             if f.endswith(".java"))
            source = os.path.join(directory, name + ".java")
            stdin_path = os.path.join(directory, name + ".stdin")
        else:
            source = os.path.join(cases, name + ".java")
            sources = [source]
            stdin_path = os.path.join(cases, name + ".stdin")
        stdin = open(stdin_path).read() if os.path.exists(stdin_path) else ""
        javac = subprocess.run(["javac", "-d", out] + sources, capture_output=True, text=True)
        if javac.returncode:
            first = javac.stderr.splitlines()[0] if javac.stderr else "?"
            rejected.append((name, first))
            continue
        # BYTES, not text: Python's universal-newline translation turns a `\r`
        # the program printed into a `\n`, so a program whose output really
        # does contain one (`"a\r\nb".split("\n")`) reads as a divergence that
        # is entirely the harness.
        jdk = subprocess.run(["java", "-cp", out, name], input=stdin.encode(),
                             capture_output=True, timeout=120)
        expected = normalize(jdk.stdout.decode("utf-8", "replace"))
        if jdk.returncode:
            expected += "!! " + jdk_failure(jdk.stderr.decode("utf-8", "replace")) + "\n"
        command = ["cargo", "run", "-q", "--example", "compatrun", "--", os.path.abspath(source), name]
        command += [os.path.abspath(other) for other in sources if other != source]
        if stdin:
            command.append("--stdin")
        got = subprocess.run(command, input=stdin, capture_output=True, text=True,
                             cwd=REPO, timeout=300)
        try:
            answer = json.loads(got.stdout)
        except json.JSONDecodeError:
            diffs.append((name, "caturra printed no JSON", got.stdout[:400] + got.stderr[-400:]))
            continue
        actual = normalize(answer.get("stdout", ""))
        if not answer.get("ok"):
            first = answer.get("error", "").split("\n")[0].replace("uncaught exception: ", "")
            actual += "!! " + first + "\n"
        if actual != expected:
            diffs.append((name, "diverges", f"jdk={expected!r}\ncat={actual!r}"))
    print(f"{len(names)} cases, {len(rejected)} javac-rejected, {len(diffs)} diverging")
    for name, why in rejected:
        print(f"~~~ {name}: {why}")
    for name, kind, detail in diffs:
        print(f"--- {name} [{kind}]\n{detail}")
    sys.exit(1 if diffs else 0)


if __name__ == "__main__":
    main()
