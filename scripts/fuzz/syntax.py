#!/usr/bin/env python3
"""Break a program in one place and see whether both engines say WHERE.

A syntax mistake is the first thing a student meets, and the hand-written
sweep behind `specs/LANGUAGE.md`'s "Where a parse error points" was eighteen
programs. This is the same question asked of hundreds: take a program that
compiles, mutate ONE token — delete it, duplicate it, swap it with its
neighbour, or replace it with a different symbol — and compare what the two
engines report.

What is compared is the LINE of the first error and the NUMBER of errors, not
the wording: caturra's parser is deliberately more explicit than javac's
("expected ';' to end the declaration" where javac says "';' expected"), which
`specs/LANGUAGE.md` writes down. The line is what a student is told to look at
and what an editor marks in the gutter.

The COLUMN is counted and reported, but it does not gate: javac has a
convention per diagnostic for which token its caret sits under — the operator
of a binary expression, the name of a method that clashes, the token after a
gap — and caturra matches many but not all. A column difference on the right
line is a different (and much smaller) thing than blaming the wrong line.

A mutation that still compiles is skipped (both engines agree it is fine, and
there is nothing to compare); one that javac accepts and caturra refuses is
reported as a REFUSAL, which is the dangerous direction.

    scripts/fuzz/syntax.py [cases-dir] [--count N] [--seed N] [--verbose]
                           [--dump DIR]

`--dump` keeps the programs whose first error the two engines put in different
places, so a run's counterexamples can be read rather than guessed at.

With no directory it generates programs with `programs.py` first. Needs
`javac` on PATH and a built `target/release/examples/diagnostics`.
"""
import os, random, re, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ENGINE = os.path.join(REPO, "target/release/examples/diagnostics")

# The single-token edits a student really makes: a token dropped, one typed
# twice, two transposed, and a symbol mistyped as its neighbour on the keyboard
# or in the grammar.
SWAPS = {
    ";": [",", ":", ""],
    ",": [";", ""],
    "(": ["[", "{"],
    ")": ["]", "}"],
    "{": ["("],
    "}": [")"],
    "=": ["==", ":"],
    "==": ["="],
    ".": [","],
    "[": ["("],
    "]": [")"],
}

TOKEN = re.compile(r'"[^"\n]*"|\'[^\'\n]*\'|\w+|==|<=|>=|!=|\+\+|--|&&|\|\||[^\s\w]')

# A COMMENT is not code, and breaking one is not a syntax mistake a student
# makes — deleting one `/` of a `//` turns the rest of the line into nonsense,
# and where either engine starts complaining about nonsense is arbitrary.
COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)


def comment_spans(source):
    return [(m.start(), m.end()) for m in COMMENT.finditer(source)]


def mutate(source, rng):
    """One edit, and a note of what it was."""
    tokens = [(m.group(), m.start(), m.end()) for m in TOKEN.finditer(source)]
    # Never touch the class declaration line: a mutated class NAME is a
    # different mistake (the file/class rule), not a syntax one.
    hidden = comment_spans(source)
    body = [
        t
        for t in tokens
        if t[1] > source.index("{")
        and not any(start <= t[1] < end for start, end in hidden)
    ]
    if not body:
        return None
    text, start, end = rng.choice(body)
    kind = rng.choice(["delete", "double", "swap", "replace"])
    if kind == "replace":
        options = SWAPS.get(text)
        if not options:
            kind = "delete"
        else:
            return source[:start] + rng.choice(options) + source[end:], f"replace {text!r}"
    if kind == "delete":
        return source[:start] + source[end:], f"delete {text!r}"
    if kind == "double":
        return source[:end] + " " + text + source[end:], f"double {text!r}"
    # swap with the following token
    following = [t for t in body if t[1] >= end]
    if not following:
        return None
    other, ostart, oend = following[0]
    return (
        source[:start] + other + source[end:ostart] + text + source[oend:],
        f"swap {text!r} {other!r}",
    )


def javac_errors(path):
    """(line, column) of each error javac reports, in order."""
    with tempfile.TemporaryDirectory() as out:
        done = subprocess.run(
            ["javac", "-d", out, path], capture_output=True, text=True, timeout=120
        )
    lines = done.stderr.splitlines()
    found = []
    for index, line in enumerate(lines):
        if ": error: " not in line:
            continue
        where = line.split(": error: ", 1)[0]
        number = where.rsplit(":", 1)[-1]
        caret = lines[index + 2] if index + 2 < len(lines) else ""
        column = len(caret) - len(caret.lstrip()) + 1 if caret.strip() == "^" else None
        found.append((int(number) if number.isdigit() else 0, column))
    return found


def caturra_errors(path):
    done = subprocess.run(
        [ENGINE, os.path.abspath(path)], capture_output=True, text=True, cwd=REPO, timeout=120
    )
    if done.returncode != 0:
        raise RuntimeError(f"the engine CRASHED on {path}: {done.stderr.strip()[:300]}")
    found = []
    for line in done.stdout.splitlines():
        if not line.startswith("Error@"):
            continue
        where = line.split(": ", 1)[0]
        line_text, column_text = where.split("@", 1)[1].split(",")
        found.append((int(line_text), int(column_text)))
    return found


def main():
    # …minus the VALUES of the flags, which are positional-looking.
    argv = sys.argv[1:]
    skip = set()
    for flag in ("--count", "--seed", "--dump"):
        if flag in argv:
            skip.add(argv.index(flag) + 1)
    args = [a for i, a in enumerate(argv) if not a.startswith("--") and i not in skip]
    verbose = "--verbose" in sys.argv
    count = 200
    if "--count" in argv:
        count = int(argv[argv.index("--count") + 1])
    seed = 20260901
    if "--seed" in argv:
        seed = int(argv[argv.index("--seed") + 1])
    rng = random.Random(seed)
    dump = None
    if "--dump" in argv:
        dump = argv[argv.index("--dump") + 1]
        os.makedirs(dump, exist_ok=True)

    directory = args[0] if args else None
    if directory is None:
        directory = tempfile.mkdtemp(prefix="syntax-sources-")
        subprocess.run(
            [sys.executable, os.path.join(REPO, "scripts/fuzz/programs.py"),
             "40", str(seed), "--out", directory],
            check=True, capture_output=True,
        )
    sources = sorted(name for name in os.listdir(directory) if name.endswith(".java"))
    if not sources:
        sys.exit(f"no .java in {directory}")

    checked = skipped = agreed = columns = 0
    disagreed, refusals, fewer, accepted = [], [], [], []
    with tempfile.TemporaryDirectory() as work:
        for index in range(count):
            name = sources[index % len(sources)]
            original = open(os.path.join(directory, name)).read()
            edit = mutate(original, rng)
            if edit is None:
                continue
            broken, what = edit
            path = os.path.join(work, name)
            with open(path, "w") as handle:
                handle.write(broken)
            want = javac_errors(path)
            got = caturra_errors(path)
            if not want:
                # javac accepts it. So must caturra: refusing a program a JDK
                # compiles is the dangerous direction, mutation or not.
                if got:
                    refusals.append((name, what, got[0]))
                skipped += 1
                continue
            checked += 1
            if not got:
                # javac refuses it and caturra does not: a program that
                # compiles HERE and fails on a JDK, which is the direction
                # that matters most.
                accepted.append((name, what, want[0]))
                if dump:
                    stem = f"{name[:-5]}_accepted_{len(accepted)}"
                    with open(os.path.join(dump, f"{stem}.java"), "w") as handle:
                        handle.write(broken.replace(name[:-5], stem))
                    with open(os.path.join(dump, f"{stem}.txt"), "w") as handle:
                        handle.write(f"{what}\njdk {want}\ncat []\n")
                continue
            if want[0][0] == got[0][0]:
                agreed += 1
                columns += int(want[0][1] == got[0][1])
                if len(want) != len(got):
                    # The first mistake is in the same PLACE; the engines
                    # differ in how much they say after it. javac recovers and
                    # keeps parsing, caturra usually stops — fewer is the safe
                    # direction, and worth counting separately from being
                    # wrong about where.
                    fewer.append((name, what, len(want), len(got)))
            else:
                disagreed.append((name, what, want, got))
                if dump:
                    stem = f"{name[:-5]}_{len(disagreed)}"
                    with open(os.path.join(dump, f"{stem}.java"), "w") as handle:
                        handle.write(broken.replace(name[:-5], stem))
                    with open(os.path.join(dump, f"{stem}.txt"), "w") as handle:
                        handle.write(f"{what}\njdk {want}\ncat {got}\n")
                if verbose:
                    print(f"--- {name}: {what}\n    jdk: {want}\n    cat: {got}")

    for name, what, first in refusals:
        print(f"REFUSED {name}: {what} — javac accepts, caturra says {first}")
    for name, what, first in accepted:
        print(f"ACCEPTED {name}: {what} — javac refuses at {first}, caturra compiles it")
    for name, what, want, got in disagreed[: 10 if not verbose else 0]:
        print(f"--- {name}: {what}\n    jdk: {want}\n    cat: {got}")
    trailing = sum(1 for _, _, want, got in fewer if got < want)
    print(
        f"\n{checked} broken programs compared ({skipped} still compiled)\n"
        f"  {agreed} put the FIRST error on the same LINE "
        f"({columns} on the same column too)\n"
        f"  of those, {agreed - len(fewer)} say the same number of things, "
        f"{trailing} say less after it and {len(fewer) - trailing} say more\n"
        f"  {len(disagreed)} put the first error on a different LINE\n"
        f"  {len(accepted)} compiled a program javac refuses\n"
        f"  {len(refusals)} refused a program javac accepts"
    )
    return 1 if refusals or accepted else 0


if __name__ == "__main__":
    sys.exit(main())
