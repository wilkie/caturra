#!/usr/bin/env python3
"""Every builtin method table's DESCRIPTOR, against the JDK's own signature.

`measure.py` asks whether a method NAME exists and `behaviour.py` whether the
answer matches; neither reads the descriptor beside the entry. It is not
decoration: the descriptor is what tells the argument check which functional
interface a call really wants, so a wrong one refuses a variable of the right
type — `IntStream.takeWhile` was spelled with the OBJECT stream's `Predicate`
and turned away an `IntPredicate` while `filter` next to it took one.

The class ↔ table pairing is read out of `builtin_instance_table` and
`builtin_static_table` rather than kept in a second list here, so a table added
to the compiler is checked without anyone remembering to add it.

Reports, per table, the entries whose descriptor names a REFERENCE type no
overload of that name and arity in a real JDK 11 names. Primitives are not
compared: caturra widens `char` to `int` in places on purpose, and the argument
check reads the parameter kind for those.

    scripts/coverage/signatures.py [--verbose] [Class ...]

Exit status is 1 if anything is unexplained, so it can gate a script.
"""

import argparse
import os
import tempfile
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CODEGEN = os.path.join(REPO, "crates/caturra-compiler/src/codegen.rs")

# Descriptors caturra spells its own way, with the reason. Each is a fact about
# the model, not a table that drifted.
# A parameter caturra models as an opaque `Object`, with the reason. The
# parameter KIND beside the descriptor carries the check for these; the
# descriptor cannot name the interface because caturra has no value of it.
ERASED = ("java.util.function.", "java.time.temporal.Temporal", "java.util.Comparator")

KNOWN = [
    (
        r"^java\.util\.stream\.IntStream\.",
        "one table serves all three primitive streams, so its descriptors are "
        "spelled in the `Int` flavour and the receiver's element type says "
        "which family a call is in",
    ),
    (
        r"\.__",
        "a caturra-internal helper, which no JDK class declares",
    ),
]


def known_reason(label):
    for pattern, reason in KNOWN:
        if re.search(pattern, label):
            return reason
    return None


def tables():
    """Every (JDK class, table constant) the compiler pairs up."""
    source = open(CODEGEN).read()
    found = {}
    for internal, const in re.findall(
        r'=>\s*Some\(\(\s*"([\w/$]+)"\s*,\s*(\w+_METHODS)\s*\)\)', source
    ):
        found.setdefault(const, internal.replace("/", "."))
    return found


def entries(const):
    """The (name, descriptor) pairs a table declares."""
    source = open(CODEGEN).read()
    start = source.find(f"const {const}")
    if start < 0:
        return []
    # To the next top-level item, not to the next `\n];`: a one-entry table is
    # written `= &[bm(...)];` and ends `)];`, so looking for the line-start
    # form ran on into the NEXT table and blamed it for those entries.
    end = source.find("\nconst ", start + 1)
    block = source[start : end if end > 0 else len(source)]
    return re.findall(
        r'bm\(\s*"([A-Za-z0-9_]+)",\s*&\[[^\]]*\],\s*BRet::\w+,\s*"([^"]+)"', block
    )


def descriptor_params(descriptor):
    """`(Ljava/lang/String;II)V` -> ['java.lang.String', 'I', 'I']."""
    inner = descriptor[descriptor.index("(") + 1 : descriptor.index(")")]
    out, at = [], 0
    while at < len(inner):
        if inner[at] == "L":
            end = inner.index(";", at)
            out.append(inner[at + 1 : end].replace("/", "."))
            at = end + 1
        elif inner[at] == "[":
            end = at + 1
            while inner[end] == "[":
                end += 1
            if inner[end] == "L":
                end = inner.index(";", end)
            out.append(inner[at : end + 1])
            at = end + 1
        else:
            out.append(inner[at])
            at += 1
    return out


def narrows(pairs, work):
    """Which of these (caturra, jdk) parameter pairs is a NARROWING.

    caturra names one concrete type where a JDK names the interface it
    implements — `LocalDate.isBefore(LocalDate)` for a JDK's
    `(ChronoLocalDate)` — because that concrete type is the only one modelled.
    Such a descriptor accepts strictly less than a JDK's, which is a modelling
    choice rather than drift, and asking a JDK is exact where a hand-kept list
    of families would go stale.
    """
    if not pairs:
        return set()
    subprocess.run(
        ["javac", "-d", work, os.path.join(REPO, "scripts/coverage/Assignable.java")],
        check=True,
    )
    run = subprocess.run(
        ["java", "-cp", work, "Assignable"],
        input="".join(f"{a}\t{b}\n" for a, b in sorted(pairs)),
        capture_output=True,
        text=True,
    )
    out = set()
    for line in run.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) == 3 and parts[2] == "true":
            out.add((parts[0], parts[1]))
    return out


def jdk_signatures(classes, work):
    """Every public overload a JDK declares, as {class: {name: [params]}}."""
    subprocess.run(
        ["javac", "-d", work, os.path.join(REPO, "scripts/coverage/ApiList.java")],
        check=True,
    )
    run = subprocess.run(
        ["java", "-cp", work, "ApiList", "--signatures", *classes],
        capture_output=True,
        text=True,
    )
    out = {}
    for line in run.stdout.splitlines():
        parts = line.split("\t")
        if len(parts) < 5:
            continue
        out.setdefault(parts[0], {}).setdefault(parts[1], []).append(
            [p for p in parts[4].split(",") if p]
        )
    return out


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--verbose", action="store_true")
    parser.add_argument("only", nargs="*", default=None)
    options = parser.parse_args()

    paired = tables()
    classes = sorted({c for c in paired.values() if c.startswith("java")})
    if options.only:
        classes = [c for c in classes if c in options.only]
    with tempfile.TemporaryDirectory(prefix="signatures-") as work:
        jdk = jdk_signatures(classes, work)

    checked, wrong, declared = 0, [], []
    for const, cls in sorted(paired.items()):
        if cls not in jdk:
            continue
        for name, descriptor in entries(const):
            if name not in jdk[cls]:
                continue
            params = descriptor_params(descriptor)
            checked += 1
            # Only the REFERENCE parameters: caturra widens the primitives on
            # purpose, and the argument check reads the parameter kind there.
            same_arity = [w for w in jdk[cls][name] if len(w) == len(params)]
            # A VARARGS method's zero-varargs form is a real call, and caturra
            # declares it as its own arity: `getMethod(name)` is `getMethod(name)`
            # on a JDK too, with an empty `Class[]`.
            if any(
                len(w) == len(params) + 1 and w[-1].startswith("[")
                and all(p == q for p, q in zip(params, w))
                for w in jdk[cls][name]
            ):
                continue
            if any(
                all("." not in p or p == q for p, q in zip(params, w))
                for w in same_arity
            ):
                continue
            label = f"{cls}.{name}{descriptor}"
            reason = known_reason(label)
            if reason:
                declared.append((label, jdk[cls][name], reason))
            else:
                wrong.append((label, jdk[cls][name], params, same_arity))

    # ...and of what is left, the ones that merely NARROW a JDK's parameter.
    def erased(caturra, wanted):
        return caturra == "java.lang.Object" and wanted.startswith(ERASED)

    asked = {
        (p, q)
        for _, _, params, same_arity in wrong
        for w in same_arity
        for p, q in zip(params, w)
        if p != q and "." in p and "." in q and not erased(p, q)
    }
    with tempfile.TemporaryDirectory(prefix="assignable-") as work:
        narrowing = narrows(asked, work)
    remaining = []
    for label, wanted, params, same_arity in wrong:
        why = None
        for w in same_arity:
            if all(p == q or (p, q) in narrowing for p, q in zip(params, w)):
                why = "narrows a JDK's parameter to the one concrete type caturra models"
            elif all(p == q or (p, q) in narrowing or erased(p, q) for p, q in zip(params, w)):
                why = "an erased parameter: caturra models it as an opaque Object, and the parameter KIND beside the descriptor carries the check"
        if why:
            declared.append((label, wanted, why))
        else:
            remaining.append((label, wanted, None))
    wrong = remaining

    print(f"{checked} descriptors checked over {len(classes)} classes, "
          f"{len(wrong)} disagreeing ({len(declared)} declared)")
    for label, wanted, _ in wrong:
        print(f"  {label}")
        for w in wanted:
            print(f"      jdk: ({','.join(w)})")
    if options.verbose and declared:
        print(f"\n{len(declared)} declared:")
        for label, _, reason in sorted(declared):
            print(f"  {label}\n      {reason}")
    return 1 if wrong else 0


if __name__ == "__main__":
    sys.exit(main())
