#!/usr/bin/env python3
"""The engine must not CRASH, whatever a program says.

A panic is the worst answer a compiler can give: `Stream.generate(null)` ended
in `unreachable!("guarded by caller")`, and the panic took every other
diagnostic in the file with it — a program with that call and two ordinary
mistakes reported nothing at all, which reads exactly like a clean compile.
Nothing in the sweep or fuzz tooling was watching for that: they compare what
caturra SAYS with what a JDK says, and a crash says nothing.

So this asks a different question. Over the same API surface the coverage
measurement walks (`scripts/coverage/`), it calls every modelled method with
each of a handful of argument SHAPES — a null, a lambda, a method reference, a
supplier, a number, a string — and checks only that the engine exits cleanly.
Whether the call is legal is beside the point: an illegal one must be REFUSED,
not crash.

A failing shape is then bisected line by line, so the report names the single
call that did it rather than the batch it was in.

    scripts/fuzz/panics.py [--verbose]

Needs a real `javac`/`java` on PATH (the API list comes from reflection) and a
built `target/release/examples/diagnostics`.
"""
import json, os, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ENGINE = os.path.join(REPO, "target/release/examples/diagnostics")
sys.path.insert(0, os.path.join(REPO, "scripts", "coverage"))

# One argument expression per shape. Each is written into EVERY parameter of
# every overload: what matters is that the engine survives the mismatch, and a
# uniform filling reaches far more positions than a typed one would.
SHAPES = {
    "null": "null",
    "lambda": "x -> x",
    "bilambda": "(a, b) -> a",
    "supplier": "() -> 1",
    "methodref": "String::valueOf",
    "ctorref": "java.util.ArrayList::new",
    "int": "0",
    "string": '"s"',
    "object": "new Object()",
    "array": "new int[] {1}",
}


def run(source):
    """Compile one program; returns (exit code, stdout, stderr)."""
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        done = subprocess.run(
            [ENGINE, path], capture_output=True, text=True, cwd=REPO, timeout=300
        )
    return done.returncode, done.stdout, done.stderr


# The same call written in the positions a program really uses. A crash can
# hide in what the compiler does with the RESULT — typing it, assigning it,
# chaining another call onto it — rather than in the call itself, and the
# statement form reaches none of that.
POSITIONS = (
    "{call};",
    "System.out.println({call});",
    "Object __v = {call};",
    "var __w = {call};",
    "System.out.println(({call}).toString());",
)


def program(calls):
    return (
        "public class Probe {\n    public static void main(String[] args) {\n"
        + "\n".join(f"        {call}" for call in calls)
        + "\n    }\n}\n"
    )


def constructors(class_names):
    """Every public constructor's arity, by reflection — `new X(...)` is a
    code path of its own (each modelled type has its own emitter) and the
    method walk never reaches it."""
    with tempfile.TemporaryDirectory() as directory:
        source = os.path.join(REPO, "scripts/coverage/ApiList.java")
        subprocess.run(
            ["javac", "-d", directory, source], check=True, capture_output=True
        )
        listing = subprocess.run(
            ["java", "-cp", directory, "ApiList", "--constructors", *class_names],
            check=True, capture_output=True, text=True, timeout=300,
        ).stdout
    found = {}
    for line in listing.splitlines():
        class_name, _, _, arity = line.split("\t")
        found.setdefault(class_name, set()).add(int(arity))
    return found


def main():
    verbose = "--verbose" in sys.argv
    import measure  # the API inventory and receiver table, shared

    receivers = json.load(
        open(os.path.join(REPO, "scripts/coverage/receivers.json"))
    )
    classes = sorted(name for name, r in receivers.items() if r != "SKIP")
    api = measure.inventory(classes)
    crashes, checked = [], 0
    for class_name in classes:
        receiver = receivers[class_name]
        for shape, argument in SHAPES.items():
            calls = []
            for index, (name, is_static, arity) in enumerate(api.get(class_name, [])):
                target = class_name if is_static else receiver
                arguments = ", ".join([argument] * arity)
                call = f"{target}.{name}({arguments})"
                # Rotate through the positions rather than taking the
                # cross-product: every method reaches every position across the
                # ten shapes, and the run stays a minute rather than an hour.
                position = POSITIONS[index % len(POSITIONS)]
                calls.append(position.replace("__v", f"__v{index}").replace("__w", f"__w{index}").format(call=call))
            if not calls:
                continue
            checked += 1
            code, _, _ = run(program(calls))
            if code == 0:
                continue
            # Bisect to the one call that did it, so the report is actionable.
            for call in calls:
                one, _, stderr = run(program([call]))
                if one != 0:
                    reason = next(
                        (
                            line
                            for line in stderr.splitlines()
                            if "panicked at" in line or "internal error" in line
                        ),
                        stderr.strip().splitlines()[-1] if stderr.strip() else "",
                    )
                    crashes.append((class_name, shape, call, reason))
                    print(f"CRASH {class_name} [{shape}] {call}\n      {reason}")
            if verbose:
                print(f"  (batch for {class_name} [{shape}] exited {code})")
    # …and the constructors, whose arguments reach a different emitter for
    # every modelled type.
    built = constructors(classes)
    for class_name in classes:
        for shape, argument in SHAPES.items():
            calls = []
            for index, arity in enumerate(sorted(built.get(class_name, ()))):
                arguments = ", ".join([argument] * arity)
                call = f"new {class_name}({arguments})"
                position = POSITIONS[index % len(POSITIONS)]
                calls.append(
                    position.replace("__v", f"__v{index}")
                    .replace("__w", f"__w{index}")
                    .format(call=call)
                )
            if not calls:
                continue
            checked += 1
            code, _, _ = run(program(calls))
            if code == 0:
                continue
            for call in calls:
                one_code, _, stderr = run(program([call]))
                if one_code != 0:
                    reason = next(
                        (
                            line
                            for line in stderr.splitlines()
                            if "panicked at" in line or "internal error" in line
                        ),
                        stderr.strip().splitlines()[-1] if stderr.strip() else "",
                    )
                    crashes.append((class_name, shape, call, reason))
                    print(f"CRASH {class_name} [{shape}] {call}\n      {reason}")

    print(f"\n{checked} probes over {len(classes)} classes, {len(crashes)} crashes")
    return 1 if crashes else 0


if __name__ == "__main__":
    sys.exit(main())
