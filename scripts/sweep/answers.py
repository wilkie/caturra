#!/usr/bin/env python3
"""Every library method's answer, read through a LAMBDA's parameter.

    scripts/sweep/answers.py [--verbose] [Class ...]

`behaviour.py` calls every method and compares what it answers; it writes the
RECEIVER out in full, so the call's type comes from the expression. This asks
the other half: does the value carry its type through a lambda's parameter?

    java.nio.file.Path p = Stream.of(PATH).map(x -> x.getFileName())
                                          .findFirst().get();

A receiver whose element type the lambda pass cannot name makes `x` an
`Object`, and then the call on it is "cannot find symbol" — about a method the
class plainly has. Printing the answer would not show it (`println` takes an
`Object`), so each call is ASSIGNED to the type the JDK declares for it: the
assignment is what demands the type, and a refusal is the finding.

Exit status is 1 if anything diverged, so it can gate a script.
"""

import json
import os
import re
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "scripts/coverage"))
import behaviour  # noqa: E402  (the argument bank and the signature reader)

ENGINE = os.path.join(REPO, "target/release/examples/compatrun")

# A receiver whose own value differs between runs makes every answer on it
# noise; `behaviour.py` skips the same ones for the same reason.
NOISY = re.compile(r"random|now\(\)|currentTimeMillis|nanoTime|hashCode|identity")

# Classes whose receiver is not a VALUE a stream can hold, or whose methods
# are all static — nothing to ask here.
SKIP = {"STATIC", "SKIP", None}


def declared(name):
    """A JDK type name as Java source: `[I` is `int[]`, a class stays as it is
    (raw where it is generic, which compiles and is enough to demand a type)."""
    primitive = {
        "[I": "int[]", "[J": "long[]", "[D": "double[]", "[C": "char[]",
        "[B": "byte[]", "[S": "short[]", "[F": "float[]", "[Z": "boolean[]",
    }
    if name in primitive:
        return primitive[name]
    if name.startswith("[L") and name.endswith(";"):
        return name[2:-1] + "[]"
    if name.startswith("[["):
        inner = declared(name[1:])
        return inner + "[]" if inner else None
    return name


def calls_for(class_name, receiver, overloads, answered):
    """One assignment per answered, buildable, value-returning overload."""
    calls = []
    for name, is_static, returns, params in overloads:
        if is_static or name not in answered or name in behaviour.NON_DETERMINISTIC:
            continue
        if returns == "void" or NOISY.search(name):
            continue
        signature = f"{name}({','.join(params)})"
        if behaviour.WRITTEN.get(f"{class_name}.{signature}") or behaviour.WRITTEN.get(
            f"*.{signature}"
        ):
            continue  # a call the bank cannot type; `behaviour.py` owns those
        banks = [behaviour.BANK.get(p) for p in params]
        if any(bank is None for bank in banks):
            continue
        arguments = ", ".join(bank[0] for bank in banks)
        held = declared(returns)
        if held is None or "$" in held:
            continue
        call = f"x.{name}({arguments})"
        # The receiver is a DECLARED local of the method, not the expression
        # itself: the question here is whether a value carries its type
        # through a lambda's PARAMETER, and streaming the expression would ask
        # instead whether the pass can type that expression — a different
        # question, which `behaviour.py` already covers from the other side.
        body = (
            f"{{ {held} v = java.util.stream.Stream.of(RECEIVER)"
            f".map(x -> {call}).findFirst().get(); return v; }}"
        )
        calls.append((f"{signature}", body))
    return calls


def probe_source(class_name, receiver, calls):
    lines = [
        "public class Probe {",
        "    interface Body { Object get() throws Throwable; }",
        "    static void s(String l, Body b) {",
        '        try { System.out.println(l + " = " + b.get()); }',
        "        catch (Throwable e) {",
        '            System.out.println(l + " ! " + e.getClass().getName());',
        "        }",
        "    }",
        # The field `behaviour.RECEIVER_OVERRIDES` reflects on (`Field` is
        # `Probe.class.getDeclaredField("n")`). Missing here, that receiver
        # threw NoSuchFieldException on BOTH engines and the whole class was
        # filed as something caturra could not type.
        "    int n = 1;",
        "    public static void main(String[] args) throws Throwable {",
        # Declared HERE rather than inside each lambda: a local written inside
        # a lambda body is a scope of its own, and asking about that would be
        # a different question again.
        f"        {class_name} recv = {receiver};",
    ]
    for label, call in calls:
        body = call.replace("RECEIVER", "recv")
        lines.append(f"        s({json.dumps(label)}, () -> {body});")
    lines += ["    }", "}"]
    return "\n".join(lines) + "\n"


IDENTITY = re.compile(r"@[0-9a-f]+")

# Divergences that are DECLARED, with the reason, rather than found — the same
# arrangement `behaviour.py` has, for the answers only this sweep reaches.
DECLARED = [
    (
        r"java\.util\.stream\.\w*Pipeline\$|java\.util\.stream\.SliceOps\$",
        "caturra names every stream `ReferencePipeline$Head`; a JDK names it "
        "by family (`IntPipeline`) and by the OP that produced it "
        "(`IntPipeline$9`, `SliceOps$2`), which is its own internal numbering",
    ),
    (
        r"^java\.util\.BitSet\.size\(\)",
        "a BitSet is stored as the words that HOLD bits, trimmed, so the "
        "ALLOCATED word count `size()` reports is not derivable from it — the "
        "same kind of modelling choice as the HashMap treeify gap",
    ),
]

# `probe_source`'s preamble, so a line number names a call.
HEADER_LINES = 12


def javac_refused(source):
    """The 1-based probe lines javac itself will not take — this sweep's own
    mistakes (an argument the bank chose badly), not findings."""
    import tempfile

    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        built = subprocess.run(
            ["javac", "-d", directory, path], capture_output=True, text=True, timeout=300
        )
    return {int(m) for m in re.findall(r"Probe\.java:(\d+): error", built.stderr)}


def main():
    verbose = "--verbose" in sys.argv
    wanted = [a for a in sys.argv[1:] if not a.startswith("--")]
    receivers = json.load(open(os.path.join(REPO, "scripts/coverage/receivers.json")))
    answered = json.load(open(os.path.join(REPO, "scripts/coverage/answered.json")))
    classes = [c for c in sorted(receivers) if not wanted or c in wanted]
    api = behaviour.signatures(classes)

    asked, known, refused, diverged, unbuilt = 0, 0, [], [], []
    for class_name in classes:
        receiver = behaviour.RECEIVER_OVERRIDES.get(class_name, receivers.get(class_name))
        if receiver in SKIP or NOISY.search(receiver):
            continue
        calls = calls_for(
            class_name, receiver, api.get(class_name, []), set(answered.get(class_name, []))
        )
        if not calls:
            continue
        # Until javac takes the probe, not once: javac reports FLOW errors
        # (an unreported checked exception) only after attribution succeeds,
        # so dropping the first round's lines can reveal more — and those were
        # then filed as caturra refusing a program javac refuses too.
        for _ in range(6):
            bad = javac_refused(probe_source(class_name, receiver, calls))
            if not bad:
                break
            calls = [c for at, c in enumerate(calls) if at + HEADER_LINES not in bad]
        # caturra REFUSING a call is the finding this sweep exists for, and a
        # refusal stops the whole program — so the refused lines are recorded
        # by NAME, dropped, and the rest asked again. Repeated, because one
        # round only names the calls the compiler reached.
        for _ in range(12):
            if not calls:
                break
            jdk, cat = behaviour.run_both(probe_source(class_name, receiver, calls))
            if jdk is not None:
                break
            reason = str(cat)
            # A RUN-TIME failure with no line is the receiver itself failing to
            # build — `new FileReader("f.txt")` with no such file — before any
            # call is asked. Neither engine can answer a probe like that, so it
            # is counted as what it is rather than as a type caturra lacked.
            if reason.startswith("caturra:0: uncaught exception"):
                unbuilt.append((class_name, reason.split(": ", 2)[-1]))
                calls = []
                break
            if not reason.startswith("caturra:"):
                refused.append((class_name, reason))
                calls = []
                break
            lines = {int(n) for n in reason.split(":", 2)[1].split(",") if n.strip()}
            named = [c for at, c in enumerate(calls) if at + HEADER_LINES in lines]
            for label, _ in named:
                refused.append((class_name, f"{label}  {reason.split(': ', 1)[-1]}"))
            keep = [c for at, c in enumerate(calls) if at + HEADER_LINES not in lines]
            if len(keep) == len(calls):
                refused.append((class_name, reason))
                calls = []
                break
            calls = keep
        else:
            calls = []
        if not calls or jdk is None:
            continue
        asked += len(calls)
        for want, got in zip(jdk.splitlines(), cat.splitlines()):
            if IDENTITY.sub("@X", want) == IDENTITY.sub("@X", got):
                continue
            # The DECLARATIONS `behaviour.py` already carries — a working
            # directory that differs between engines, a lambda's address —
            # are facts about the model, and this sweep reads the same
            # answers, so it reads the same list rather than a second copy.
            full = f"{class_name}.{want}"
            if any(re.search(pattern, full) for pattern, _ in behaviour.KNOWN) or any(
                re.search(pattern, full) for pattern, _ in DECLARED
            ):
                known += 1
                if "--declared" in sys.argv:
                    print(f"DECLARED {full}")
                continue
            diverged.append((class_name, want, got))

    # A refusal whose reason is that the JDK's DECLARED answer is a class
    # caturra does not model at all — `BaseStream`, `Spliterator`,
    # `TemporalUnit` — is not a gap in what the lambda pass can type: the call
    # itself works, held as its compile-time type, and the refusal names the
    # class. Counted apart, so the backlog is the questions the pass could
    # still answer.
    unmodelled = [
        (c, m) for c, m in refused
        if re.search(r"is not supported by caturra|cannot find symbol: class \w+ in package", m)
    ]
    refused = [r for r in refused if r not in unmodelled]
    if verbose:
        for class_name, message in refused:
            print(f"{class_name}.{message}")
    for class_name, want, got in diverged:
        print(f"{class_name}\n    jdk: {want}\n    cat: {got}")
    print(f"\n{asked} answers read through a lambda, {len(diverged)} diverging, "
          f"{known} declared")
    # A REFUSAL is a measurement, not a failure: it says the lambda pass has no
    # type for that answer, which is a backlog to work through rather than a
    # regression. The gate is the DIVERGENCES — an answer both engines give,
    # differently.
    print(f"{len(refused)} answers caturra has no type for (--verbose lists them)")
    print(f"{len(unmodelled)} answers whose declared class caturra does not model, refused by name")
    for class_name, why in unbuilt:
        print(f"receiver not built: {class_name} — {why}")
    return 1 if diverged else 0


if __name__ == "__main__":
    sys.exit(main())
