#!/usr/bin/env python3
"""How much of a JDK 11 class does caturra know? Measured, not estimated.

For every public method a real JDK offers on the classes caturra models, this
writes a call with that overload's own ARITY and `null` for every argument,
then reads which complaint comes back:

* "cannot find symbol" — the name is unknown to caturra;
* "X.y exists in Java, but ..." — the name is REFUSED, on purpose and with a
  reason. That is not coverage: the call does not run. It counts as unknown
  here, and `--why` lists every one of them with the reason it gives;
* anything else ("no suitable method found", "incompatible types", or no
  error at all) — the name is known, and only these arguments are wrong.

A name counts as known if ANY of its overloads' arities is. The arity matters:
a first attempt called everything with seven nulls, and caturra reports a
reported a wrong-arity call on `Objects` as "cannot find symbol" — so
`Objects.requireNonNull` read as unknown, and the measurement was a lower
bound on itself. That was caturra's bug, not the probe's, and it is fixed;
the arity is still emitted, because a wrong one measures the wrong thing.

That makes this NAME-level: it says whether `substring` exists, not whether
both of its overloads do. It is the coarsest honest question, and the answer
is checkable rather than asserted. Object's own methods are excluded (they are
answered generically, for every receiver), as are the classes whose receiver
cannot be written as one expression.

Needs a real `javac`/`java` on PATH (the denominator comes from reflection,
`ApiList.java`) and a built `target/release/examples/diagnostics`.

    scripts/coverage/measure.py [--verbose] [--why] [--write-answered]

`--write-answered` records which names came back ANSWERED, as
`scripts/coverage/answered.json`. `behaviour.py` reads it: whether a name
exists and whether it answers CORRECTLY are two questions, and the second one
should be asked of exactly the names the first one passed.
"""
import json, os, re, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ENGINE = os.path.join(REPO, "target/release/examples/diagnostics")
def measure(class_name, receiver, methods):
    """Returns (known, unknown) method-name lists for one class."""
    lines, order = [], []
    for name, is_static, arity in methods:
        target = class_name if is_static else receiver
        arguments = ", ".join(["null"] * arity)
        lines.append(f"        {target}.{name}({arguments});")
        order.append(name)
    source = (
        "public class Probe {\n    public static void main(String[] args) {\n"
        + "\n".join(lines)
        + "\n    }\n}\n"
    )
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        result = subprocess.run(
            [ENGINE, path], capture_output=True, text=True, cwd=REPO, timeout=300
        )
    # A CRASH reads exactly like a clean compile — no diagnostics — and every
    # method in the probe would then count as known. `Stream.generate(null)`
    # panicked the compiler, and this scored `Stream` at 44/44 while the same
    # calls one at a time were "cannot find symbol". Anything but a clean exit
    # is a measurement failure, not a result.
    if result.returncode != 0:
        raise RuntimeError(
            f"the engine failed on the {class_name} probe "
            f"(exit {result.returncode}): {result.stderr.strip()[:400]}"
        )
    unknown_at = set()
    for line in result.stdout.splitlines():
        if not line.startswith("Error@"):
            continue
        where, message = line.split(": ", 1)
        row = int(where.split("@", 1)[1].split(",")[0])
        # Line 3 is the first call.
        index = row - 3
        if 0 <= index < len(order) and (
            "cannot find symbol" in message
            or "not supported by caturra" in message
            or "exists in Java, but" in message
        ):
            unknown_at.add(index)
    known, missing = set(), set()
    for index, name in enumerate(order):
        (missing if index in unknown_at else known).add(name)
    # One overload answering is the whole name: `substring(1)` proves the name
    # even where `substring(1, 2)` is the one that failed.
    missing -= known
    return sorted(known), sorted(missing)


def complaint(class_name, receiver, name, methods):
    """The single diagnostic caturra gives for one missing method name.

    A missing name should EXPLAIN itself — "Collections.checkedList exists in
    Java, but ..." — rather than read as a typo. `--why` prints these so the
    refusals can be read as a list, and fails when one of them is still a bare
    "cannot find symbol".
    """
    arity = min(a for n, _, a in methods if n == name)
    is_static = next(s for n, s, a in methods if n == name and a == arity)
    # The same target expression `measure` writes — a fully qualified static
    # receiver resolves down a different path from a simple one, and the two
    # spellings gave different complaints for the same missing name.
    target = class_name if is_static else receiver
    call = f"{target}.{name}({', '.join(['null'] * arity)});"
    source = (
        "public class Probe {\n    public static void main(String[] args) {\n        "
        + call
        + "\n    }\n}\n"
    )
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        out = subprocess.run(
            [ENGINE, path], capture_output=True, text=True, cwd=REPO, timeout=300
        ).stdout
    for line in out.splitlines():
        if line.startswith("Error@"):
            return line.split(": ", 1)[1]
    return "(no error)"


def receiver_compiles(receiver):
    """Whether the RECEIVER expression itself compiles.

    A receiver that does not is a measurement failure and not a result: every
    method on it comes back "cannot find symbol" about the receiver, and the
    class reads as 0/N when the truth is that the probe was wrong. Asked with
    `hashCode()`, which every receiver in Java has and caturra answers
    generically.
    """
    source = (
        "public class Probe {\n    public static void main(String[] args) throws Exception {\n"
        f"        System.out.println(({receiver}).hashCode());\n    }}\n}}\n"
    )
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        result = subprocess.run(
            [ENGINE, path], capture_output=True, text=True, cwd=REPO, timeout=300
        )
    problems = [l for l in result.stdout.splitlines() if l.startswith("Error@")]
    return (result.returncode == 0 and not problems), (problems[0] if problems else "")


def inventory(class_names):
    """What a REAL JDK offers on each class, by reflection — the denominator.

    Asking the running JDK rather than reading a checked-in list keeps the
    measurement honest about the version it is measured against: run this on
    something other than a JDK 11 and the number moves, which is the point.
    """
    with tempfile.TemporaryDirectory() as directory:
        source = os.path.join(REPO, "scripts/coverage/ApiList.java")
        subprocess.run(
            ["javac", "-d", directory, source], check=True, capture_output=True
        )
        listing = subprocess.run(
            ["java", "-cp", directory, "ApiList", *class_names],
            check=True, capture_output=True, text=True, timeout=300,
        ).stdout
    api = {}
    for line in listing.splitlines():
        class_name, name, is_static, arity = line.split("\t")
        api.setdefault(class_name, []).append((name, is_static == "true", int(arity)))
    return api


def modelled_classes():
    """Which classes caturra names at all, read from the compiler's own table.

    `imports.rs` is the single list an import is checked against, so counting
    it is counting what a program can NAME. This is the coarser of the two
    numbers here and the one that moves in whole packages.
    """
    text = open(os.path.join(REPO, "crates/caturra-compiler/src/imports.rs")).read()
    table = text.split("static PACKAGES:", 1)[1].split("];", 1)[0]
    packages = {}
    for package, constant in re.findall(r'\("([\w.]+)",\s*(\w+)\)', table):
        body = re.split(rf"(?:const|static) {constant}:[^=]*= &\[", text, 1)[1]
        body = body.split("];", 1)[0]
        packages[package] = sorted(set(re.findall(r'"([\w$]+)"', body)))
    return packages


def main():
    verbose = "--verbose" in sys.argv
    why = "--why" in sys.argv
    write_answered = "--write-answered" in sys.argv
    answered = {}
    receivers = json.load(
        open(os.path.join(REPO, "scripts/coverage/receivers.json"))
    )
    api = inventory(sorted(receivers))
    total_known = total_all = 0
    rows, unexplained, broken = [], [], []
    for class_name, methods in sorted(api.items()):
        receiver = receivers.get(class_name)
        if receiver in (None, "SKIP"):
            continue
        if receiver != "STATIC":
            compiles, complained = receiver_compiles(receiver)
            if not compiles:
                broken.append(f"{class_name}: {receiver} — {complained}")
                continue
        known, missing = measure(class_name, receiver, methods)
        if why:
            for name in missing:
                said = complaint(class_name, receiver, name, methods)
                if "cannot find symbol" in said:
                    unexplained.append(f"{class_name}.{name}")
                print(f"{class_name}.{name}: {said}")
        answered[class_name] = known
        total_known += len(known)
        total_all += len(known) + len(missing)
        rows.append((class_name, len(known), len(known) + len(missing), missing))
    for class_name, known, total, missing in rows:
        share = 100.0 * known / total if total else 0.0
        print(f"{class_name:34} {known:3}/{total:3}  {share:5.1f}%")
        if verbose and missing:
            print(f"    missing: {' '.join(missing)}")
    share = 100.0 * total_known / total_all if total_all else 0.0
    print(f"\n{len(rows)} classes, {total_known}/{total_all} method names known ({share:.1f}%)")

    # The other half of the estimate: how many classes there are to have
    # methods on. A share is meaningless here — nobody models all of the JDK —
    # so this is an inventory, not a percentage.
    if broken:
        print(f"\n{len(broken)} receivers do not compile — these classes were NOT measured:")
        for line in broken:
            print(f"  {line}")

    if why and unexplained:
        print(f"\n{len(unexplained)} missing names say only \"cannot find symbol\":")
        for name in unexplained:
            print(f"  {name}")
        sys.exit(1)

    if write_answered:
        path = os.path.join(REPO, "scripts/coverage/answered.json")
        with open(path, "w") as handle:
            json.dump(answered, handle, indent=1, sort_keys=True)
            handle.write("\n")
        print(f"wrote {path}")

    packages = modelled_classes()
    print("\nclasses caturra names, by package:")
    core = 0
    for package, classes in sorted(packages.items(), key=lambda kv: -len(kv[1])):
        if package.startswith(("java.", "javax.")) and not package.startswith(
            ("javax.swing", "javax.accessibility", "java.awt")
        ):
            core += len(classes)
        print(f"  {package:26} {len(classes):3}")
    total = sum(len(c) for c in packages.values())
    print(f"  {'total':26} {total:3}   ({core} in core java.*, the rest Swing/AWT/org.code)")


if __name__ == "__main__":
    main()
