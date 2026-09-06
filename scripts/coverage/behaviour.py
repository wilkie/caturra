#!/usr/bin/env python3
"""Does every method caturra ANSWERS answer the same thing a JDK does?

`measure.py` asks whether a name exists — it calls with `null` arguments and
reads the diagnostic, never running anything. That is a real question and it
is not this one. A name it counts as answered can still be wrong:
`IsoEra.getDisplayName` compiled, ran, and gave "Anno Domini" for all six
styles, because the emitter named the wrong class and the VM arm that knew
better was never reached.

So: for every method the measurement counts as answered, write a call with
REAL arguments, run it on a JDK and on caturra, and diff. One program per
class, each call wrapped so a thrown exception is compared too — the class and
message, which is where half the interesting differences live.

    scripts/coverage/behaviour.py [--verbose] [class ...]

A call this cannot build (a parameter type with nothing in the argument bank)
is REPORTED, not skipped in silence — the same rule the receiver check follows.
Needs a real `javac`/`java` on PATH and a built
`target/release/examples/compatrun`.
"""
import json, os, re, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ENGINE = os.path.join(REPO, "target/release/examples/compatrun")

# One expression per parameter type. Values are deliberately awkward where an
# awkward one is cheap: a negative int, a string with a space, an empty range.
BANK = {
    "int": ["1", "0", "-2"],
    "long": ["1L", "-2L"],
    "double": ["1.5", "-0.5"],
    "float": ["1.5f"],
    "short": ["(short) 1"],
    "byte": ["(byte) 1"],
    "char": ["'q'"],
    "boolean": ["true", "false"],
    "java.lang.String": ['"ab"', '""', '"a b"'],
    "java.lang.CharSequence": ['"ab"'],
    "java.lang.Object": ['"ab"'],
    "java.lang.Integer": ["Integer.valueOf(1)"],
    "java.lang.Long": ["Long.valueOf(1L)"],
    "java.lang.Double": ["Double.valueOf(1.5)"],
    "java.lang.Character": ["Character.valueOf('q')"],
    "java.lang.Boolean": ["Boolean.valueOf(true)"],
    "java.lang.Class": ["String.class"],
    "java.lang.Throwable": ['new RuntimeException("m")'],
    "java.lang.Iterable": ['java.util.List.of("a", "b")'],
    "java.util.Collection": ['java.util.List.of("a", "b")'],
    "java.util.List": ['java.util.List.of("a", "b")'],
    "java.util.Set": ['java.util.Set.of("a")'],
    "java.util.Map": ['java.util.Map.of("k", "v")'],
    "java.util.Comparator": ["java.util.Comparator.<String>naturalOrder()"],
    "java.util.Random": ["new java.util.Random(1)"],
    "java.math.BigInteger": ["java.math.BigInteger.valueOf(7)"],
    "java.math.BigDecimal": ['new java.math.BigDecimal("2.5")'],
    "java.math.MathContext": ["java.math.MathContext.DECIMAL64"],
    "java.math.RoundingMode": ["java.math.RoundingMode.HALF_UP"],
    "java.nio.charset.Charset": ["java.nio.charset.StandardCharsets.UTF_8"],
    "java.time.LocalDate": ["java.time.LocalDate.of(2024, 3, 14)"],
    "java.time.LocalTime": ["java.time.LocalTime.of(1, 2, 3)"],
    "java.time.LocalDateTime": ["java.time.LocalDateTime.of(2024, 3, 14, 1, 2)"],
    "java.time.Duration": ["java.time.Duration.ofHours(2)"],
    "java.time.Period": ["java.time.Period.ofDays(3)"],
    "java.time.Month": ["java.time.Month.MARCH"],
    "java.time.DayOfWeek": ["java.time.DayOfWeek.MONDAY"],
    "java.time.Year": ["java.time.Year.of(2024)"],
    "java.time.YearMonth": ["java.time.YearMonth.of(2024, 3)"],
    "java.time.MonthDay": ["java.time.MonthDay.of(3, 14)"],
    "java.time.format.DateTimeFormatter": [
        'java.time.format.DateTimeFormatter.ofPattern("yyyy")'
    ],
    "java.time.format.TextStyle": ["java.time.format.TextStyle.FULL"],
    "java.time.temporal.TemporalUnit": ["java.time.temporal.ChronoUnit.DAYS"],
    "java.time.temporal.TemporalField": ["java.time.temporal.ChronoField.YEAR"],
    "java.time.temporal.ChronoUnit": ["java.time.temporal.ChronoUnit.DAYS"],
    "java.time.temporal.ChronoField": ["java.time.temporal.ChronoField.YEAR"],
    "java.util.regex.Pattern": ['java.util.regex.Pattern.compile("a+")'],
    "java.util.BitSet": ["new java.util.BitSet()"],
    "java.util.UUID": ['java.util.UUID.fromString("a8098c1a-f86e-11da-bd1a-00112444be1e")'],
    "[I": ["new int[] {3, 1, 2}"],
    "[J": ["new long[] {3L, 1L}"],
    "[D": ["new double[] {1.5, 0.5}"],
    "[C": ["new char[] {'a', 'b'}"],
    "[B": ["new byte[] {1, 2}"],
    "[Ljava.lang.String;": ['new String[] {"a", "b"}'],
    "[Ljava.lang.Object;": ['new Object[] {"a"}'],
}

# Receivers this sweep needs that differ from the measurement's.
#
# `measure.py` only needs a receiver whose TYPE resolves — it never runs
# anything — so `String.class.getDeclaredFields()[0]` is fine there. Running it
# is another matter: a LIBRARY class has no class file here, so that array is
# empty and every call on it is an ArrayIndexOutOfBounds. The probe reflects on
# ITSELF instead, which is what a validator does to a student's class, and
# `probe_source` gives it a field and a method to find.
RECEIVER_OVERRIDES = {
    "java.lang.reflect.Field": 'Probe.class.getDeclaredField("n")',
    # By NAME, not `[0]`: the order `getDeclaredMethods` answers in is
    # unspecified, so an index picks a different method on each engine.
    "java.lang.reflect.Method": 'Probe.class.getDeclaredMethod("main", String[].class)',
    "java.lang.reflect.Constructor": "Probe.class.getDeclaredConstructors()[0]",
}

# Divergences that are DECLARED, with the reason, rather than found. Each is a
# fact about what caturra models, already written down in specs/LANGUAGE.md —
# not a defect waiting to be fixed. Matched against the JDK's answer.
KNOWN = [
    (
        r"\$\$Lambda",
        "a lambda's class is its address in a JDK, different on every run",
    ),
    (
        r"^java\.io\.File\.(getAbsolute|getCanonical)",
        "caturra's filesystem is rooted at / and has no working directory",
    ),
    (
        r"^java\.util\.Comparator\.(naturalOrder|reverseOrder|nullsFirst|nullsLast)",
        "a comparator caturra synthesized is not one of a JDK's named classes",
    ),
]


# Answers that cannot agree between two runs, let alone two engines: a clock, a
# random draw, an identity hash, a live host fact. Their PRESENCE is what
# `measure.py` checks; their value is nobody's to pin.
NON_DETERMINISTIC = {
    "now", "randomUUID", "random", "currentTimeMillis", "nanoTime", "identityHashCode",
    "hashCode", "freeMemory", "totalMemory", "maxMemory", "availableProcessors",
    "lineSeparator", "getProperty", "getenv", "createTempFile", "listRoots",
    "lastModified", "setLastModified", "getStackTrace", "fillInStackTrace",
    "printStackTrace", "toString",
}


def signatures(class_names):
    """Every public overload of each class, with its parameter TYPES."""
    with tempfile.TemporaryDirectory() as directory:
        source = os.path.join(REPO, "scripts/coverage/ApiList.java")
        subprocess.run(["javac", "-d", directory, source], check=True, capture_output=True)
        listing = subprocess.run(
            ["java", "-cp", directory, "ApiList", "--signatures", *class_names],
            check=True, capture_output=True, text=True, timeout=300,
        ).stdout
    api = {}
    for line in listing.splitlines():
        class_name, name, is_static, returns, params = line.split("\t")
        parts = [p for p in params.split(",") if p]
        api.setdefault(class_name, []).append((name, is_static == "true", returns, parts))
    return api


def calls_for(class_name, receiver, overloads, answered):
    """The calls to exercise, and the overloads no argument bank could build."""
    calls, unbuildable = [], []
    for name, is_static, returns, params in overloads:
        if name not in answered or name in NON_DETERMINISTIC:
            continue
        banks = [BANK.get(p) for p in params]
        if any(bank is None for bank in banks):
            unbuildable.append(f"{class_name}.{name}({','.join(params)})")
            continue
        # One call per bank entry of the FIRST parameter, so an awkward value
        # gets exercised without the cross-product exploding.
        width = max((len(bank) for bank in banks), default=1)
        for at in range(width if params else 1):
            arguments = ", ".join(bank[min(at, len(bank) - 1)] for bank in banks)
            # A CLASS name cannot be parenthesised; a receiver EXPRESSION has
            # to be, or `"ab".length()` and `1 + 2 . foo()` parse differently.
            target = class_name if is_static else f"({receiver})"
            label = f"{name}({','.join(params)})#{at}"
            call = f"{target}.{name}({arguments})"
            # A `void` method has nothing to print, so it is run for its
            # EFFECT and its exception — the lambda answers a fixed word.
            body = call if returns != "void" else f'{{ {call}; return "ok"; }}'
            calls.append((label, body))
    return calls, unbuildable


def probe_source(cls, calls):
    """One program: every call wrapped so a THROW is compared too."""
    lines = [
        "public class Probe {",
        "    interface Body { Object get() throws Throwable; }",
        "    static void s(String l, Body b) {",
        "        try { System.out.println(l + \" = \" + b.get()); }",
        "        catch (Throwable e) {",
        "            System.out.println(l + \" ! \" + e.getClass().getName() + \": \" + e.getMessage());",
        "        }",
        "    }",
        # A field, so a probe that reflects on itself finds one — and the
        # implicit constructor gives `getDeclaredConstructors()` something too.
        "    int n = 1;",
        "    public static void main(String[] args) throws Throwable {",
    ]
    for label, call in calls:
        text = json.dumps(f"{cls}.{label}")
        lines.append(f"        s({text}, () -> {call});")
    lines += ["    }", "}"]
    return "\n".join(lines) + "\n"


# `probe_source`'s preamble, so a javac line number names a call.
HEADER_LINES = 10


def javac_rejected_lines(stderr):
    """Which 1-based lines of the probe javac would not take."""
    return {int(m) for m in re.findall(r"Probe\.java:(\d+): error", stderr)}


def run_both(source):
    """(jdk stdout, caturra stdout) for one probe, or (None, reason)."""
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "Probe.java")
        with open(path, "w") as handle:
            handle.write(source)
        built = subprocess.run(
            ["javac", "-d", directory, path], capture_output=True, text=True, timeout=300
        )
        if built.returncode != 0:
            return None, "javac: " + built.stderr.strip().splitlines()[0]
        # In the TEMP directory, not the repo: a probe that writes a file must
        # not leave one behind. (caturra's own filesystem is in memory, so only
        # the JDK side can.)
        jdk = subprocess.run(
            ["java", "-cp", directory, "Probe"],
            capture_output=True, text=True, timeout=300, cwd=directory,
        ).stdout
        result = subprocess.run(
            [ENGINE, path, "Probe"], capture_output=True, text=True, cwd=REPO, timeout=300
        )
    try:
        answer = json.loads(result.stdout)
    except ValueError:
        return None, "caturra produced no JSON"
    if answer.get("error"):
        return None, "caturra: " + answer["error"].splitlines()[0]
    return jdk, answer.get("stdout") or ""


def normalized(line):
    """One answer, with what no two runs can agree on taken out.

    A default `toString` is `Class@hash`, and the hash is an address — two
    JVMs never match, and neither do two runs of one. A stream's CLASS is the
    same kind of thing: `IntPipeline$Head` names a JDK's internal pipeline
    stage, and caturra's pipeline is not built out of the same classes.
    """
    line = re.sub(r"@[0-9a-f]+", "@x", line)
    # A stream's CLASS is a documented divergence, not a finding: a JDK names a
    # pipeline after its last operation AND its element family
    # (`IntPipeline$Head`, `ReferencePipeline$3`), and caturra's one `Stream`
    # object records neither — so `getClass()` answers `Object`, and its
    # default `toString` says the same thing.
    # A stream's pipeline CLASS: a JDK renames it after every operation and by
    # element family, and caturra names the source stage always — exact for a
    # fresh stream, stale after an intermediate op.
    return re.sub(r"java\.util\.stream\.\w+\$\w+@x", "<a stream>@x", line)


def main():
    verbose = "--verbose" in sys.argv
    wanted = [a for a in sys.argv[1:] if not a.startswith("--")]
    receivers = json.load(open(os.path.join(REPO, "scripts/coverage/receivers.json")))
    answered = json.load(open(os.path.join(REPO, "scripts/coverage/answered.json")))
    classes = [c for c in sorted(receivers) if not wanted or c in wanted]
    api = signatures(classes)

    diverged, refused, unbuildable, known, exercised = [], [], [], [], 0
    for class_name in classes:
        receiver = RECEIVER_OVERRIDES.get(class_name, receivers.get(class_name))
        if receiver in (None, "STATIC", "SKIP") and receiver != "STATIC":
            continue
        calls, cannot = calls_for(
            class_name, receiver, api.get(class_name, []), set(answered.get(class_name, []))
        )
        unbuildable += cannot
        if not calls:
            continue
        # A call one of the two ENGINES will not compile is dropped and the
        # probe re-run: one bad line — an inherited generic static, an overload
        # caturra does not offer — should not cost the whole class its sweep.
        # Everything dropped is reported rather than quietly lost.
        jdk, cat = None, ""
        for _ in range(8):
            jdk, cat = run_both(probe_source(class_name, calls))
            if jdk is not None:
                break
            if cat.startswith("javac: "):
                # javac names the LINE, and the preamble is a fixed height.
                bad = {n - HEADER_LINES - 1 for n in javac_rejected_lines(cat)}
                keep = [c for at, c in enumerate(calls) if at not in bad]
                why = "javac would not take the probe"
            else:
                # caturra names the METHOD, not the line: an overload it does
                # not offer. (The measurement is name-level, so this is an
                # expected answer and not a divergence.)
                named = set(re.findall(r"for (\w+)\(|method (\w+)\b", cat))
                names = {n for pair in named for n in pair if n}
                keep = [c for c in calls if c[0].split("(")[0] not in names]
                why = "caturra offers no such overload"
            if not keep or len(keep) == len(calls):
                break
            dropped = [c for c in calls if c not in keep]
            unbuildable += [f"{class_name}.{c[0]} ({why})" for c in dropped]
            calls = keep
        if jdk is None:
            refused.append(f"{class_name}: {cat}")
            continue
        exercised += len(calls)
        # Keyed by LABEL, not by position: a call that ends one engine's run
        # early would otherwise shift every line after it and report the whole
        # rest of the class as diverging.
        theirs = {l.split(" ", 1)[0]: normalized(l) for l in cat.splitlines()}
        for line in jdk.splitlines():
            want = normalized(line)
            label = line.split(" ", 1)[0]
            got = theirs.get(label)
            if got is None:
                diverged.append(f"  JDK: {want}\n  cat: (no answer — the run ended before this)")
                continue
            if want == got:
                continue
            declared = next((why for pattern, why in KNOWN if re.search(pattern, want)), None)
            if declared:
                known.append(f"{want.split(' ')[0]}: {declared}")
                continue
            diverged.append(f"  JDK: {want}\n  cat: {got}")

    print(
        f"{exercised} calls exercised over {len(classes)} classes, "
        f"{len(diverged)} diverging ({len(known)} declared)"
    )
    for line in diverged:
        print(line)
    if refused:
        print(f"\n{len(refused)} probes did not run:")
        for line in refused:
            print(f"  {line}")
    if verbose and unbuildable:
        print(f"\n{len(unbuildable)} overloads have no argument in the bank:")
        for line in sorted(set(unbuildable)):
            print(f"  {line}")
    sys.exit(1 if diverged else 0)


if __name__ == "__main__":
    main()
