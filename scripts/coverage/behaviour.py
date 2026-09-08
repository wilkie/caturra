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
# The AWKWARD end of each primitive is where a library's own arithmetic and its
# range complaints live, and the bank had none of it: three ints, one char, no
# NaN. Widening it is the same lesson as widening the TYPES was — "a tool's own
# report of what it skipped is a measurement" — one level down, at the values.
BANK = {
    "int": ["1", "0", "-2", "Integer.MAX_VALUE", "Integer.MIN_VALUE"],
    "long": ["1L", "-2L", "Long.MAX_VALUE", "Long.MIN_VALUE"],
    "double": [
        "1.5",
        "-0.5",
        "Double.NaN",
        "Double.POSITIVE_INFINITY",
        "-0.0",
        "Double.MIN_VALUE",
    ],
    "float": ["1.5f", "Float.NaN", "Float.NEGATIVE_INFINITY", "-0.0f"],
    "short": ["(short) 1", "Short.MIN_VALUE", "Short.MAX_VALUE"],
    "byte": ["(byte) 1", "Byte.MIN_VALUE", "Byte.MAX_VALUE"],
    # A NUL, a LONE surrogate (not a character at all) and the last unit.
    "char": ["'q'", "'\\u0000'", "'\\uD83D'", "Character.MAX_VALUE"],
    "boolean": ["true", "false"],
    # ...and a string carrying a surrogate PAIR, one carrying a lone surrogate,
    # and one carrying a NUL — the three shapes that tell a code-unit walk from
    # a code-point one.
    "java.lang.String": [
        '"ab"',
        '""',
        '"a b"',
        '"\\uD83D\\uDE00"',
        '"\\uD83D"',
        '"\\u0000"',
    ],
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
    # The rest of what the sweep's own report asked for. It named 639
    # overloads it could not build a call for, and 106 distinct types; these
    # are its head, in the order it counted them. Every one is a VALUE that is
    # the same twice — a lambda written out rather than a method reference to
    # something with state, and a fixed path rather than a temporary file.
    "java.nio.file.Path": ['java.nio.file.Path.of("f.txt")'],
    "[Ljava.lang.StackTraceElement;": [
        'new StackTraceElement[] {new StackTraceElement("C", "m", "C.java", 1)}'
    ],
    # RAW, like the casts beside them: a `Function<Object, Object>` fits only a
    # raw receiver, where the bare interface is an unchecked conversion javac
    # takes against any element type. That is what let these run at all.
    "java.util.function.Function": ["(java.util.function.Function) (v -> v)"],
    "java.util.function.BiFunction": [
        "(java.util.function.BiFunction) ((x, y) -> x)"
    ],
    "java.util.function.Predicate": ["(java.util.function.Predicate) (v -> true)"],
    "java.lang.Enum": ["java.time.Month.MARCH"],
    "java.util.function.Consumer": ["(java.util.function.Consumer) (v -> {})"],
    # NOT `java.util.Locale`: caturra models it as a namespace with no VALUE
    # of the type, and says so — an honest refusal, but one that kills the
    # probe it appears in, and six classes with it. The overloads that take one
    # are reported as unbuildable, which is what they are.
    # ...but this one keeps its type argument: a RAW `IntFunction` handed to
    # `flatMap` builds a lambda whose parameter is a reference, and a primitive
    # pipeline then feeds it an int.
    "java.util.function.IntFunction": [
        '(java.util.function.IntFunction<Object>) (n -> "" + n)'
    ],
    "[S": ["new short[] {3, 1}"],
    "[F": ["new float[] {1.5f, 0.5f}"],
    "[Z": ["new boolean[] {true, false}"],
    "java.util.function.Supplier": ['(java.util.function.Supplier) (() -> "s")'],
    "java.util.function.BiConsumer": [
        "(java.util.function.BiConsumer) ((x, y) -> {})"
    ],
    "java.time.temporal.TemporalAccessor": ["java.time.LocalDate.of(2024, 3, 14)"],
    "java.time.temporal.Temporal": ["java.time.LocalDate.of(2024, 3, 14)"],
    "java.time.temporal.TemporalAmount": ["java.time.Period.ofDays(3)"],
    "java.time.temporal.TemporalAdjuster": [
        "java.time.temporal.TemporalAdjusters.firstDayOfMonth()"
    ],
    "java.time.chrono.ChronoLocalDate": ["java.time.LocalDate.of(2024, 3, 14)"],
    "java.time.format.FormatStyle": ["java.time.format.FormatStyle.SHORT"],
    "java.util.function.BinaryOperator": [
        "(java.util.function.BinaryOperator) ((x, y) -> x)"
    ],
    "java.util.function.UnaryOperator": [
        "(java.util.function.UnaryOperator) (v -> v)"
    ],
    "java.util.function.DoublePredicate": [
        "(java.util.function.DoublePredicate) (n -> n > 1)"
    ],
    "java.util.function.IntPredicate": ["(java.util.function.IntPredicate) (n -> n > 1)"],
    "java.util.function.LongPredicate": ["(java.util.function.LongPredicate) (n -> n > 1)"],
    "java.util.function.IntUnaryOperator": [
        "(java.util.function.IntUnaryOperator) (n -> n * 2)"
    ],
    "java.util.function.LongUnaryOperator": [
        "(java.util.function.LongUnaryOperator) (n -> n * 2)"
    ],
    "java.util.function.DoubleUnaryOperator": [
        "(java.util.function.DoubleUnaryOperator) (n -> n * 2)"
    ],
    "java.util.function.IntConsumer": ["(java.util.function.IntConsumer) (n -> {})"],
    "java.util.function.LongConsumer": ["(java.util.function.LongConsumer) (n -> {})"],
    "java.util.function.DoubleConsumer": ["(java.util.function.DoubleConsumer) (n -> {})"],
    "java.util.function.IntSupplier": ["(java.util.function.IntSupplier) (() -> 4)"],
    "java.util.function.LongSupplier": ["(java.util.function.LongSupplier) (() -> 4L)"],
    "java.util.function.DoubleSupplier": [
        "(java.util.function.DoubleSupplier) (() -> 4.0)"
    ],
    "java.util.function.IntBinaryOperator": [
        "(java.util.function.IntBinaryOperator) ((x, y) -> x + y)"
    ],
    "java.util.function.ToIntFunction": [
        "(java.util.function.ToIntFunction) (v -> 1)"
    ],
    "java.util.function.ToLongFunction": [
        "(java.util.function.ToLongFunction) (v -> 1L)"
    ],
    "java.util.function.ToDoubleFunction": [
        "(java.util.function.ToDoubleFunction) (v -> 1.5)"
    ],
    "java.util.function.IntToLongFunction": [
        "(java.util.function.IntToLongFunction) (n -> n)"
    ],
    "java.util.function.IntToDoubleFunction": [
        "(java.util.function.IntToDoubleFunction) (n -> n)"
    ],
    "java.lang.Runnable": ["(Runnable) (() -> {})"],
    "java.util.stream.Collector": ["java.util.stream.Collectors.toList()"],
    "java.io.Writer": ["new java.io.StringWriter()"],
    "[Ljava.lang.Class;": ["new Class<?>[] {String.class}"],
    "[Ljava.lang.Comparable;": ['new Comparable[] {"a"}'],
    "java.lang.StringBuilder": ['new StringBuilder("ab")'],
    "java.io.File": ['new java.io.File("f.txt")'],
    "java.util.Optional": ['java.util.Optional.of("a")'],
    "java.util.Iterator": ['java.util.List.of("a").iterator()'],
    "java.util.stream.Stream": ['java.util.stream.Stream.of("a", "b")'],
    "java.util.stream.IntStream": ["java.util.stream.IntStream.of(1, 2)"],
    "java.time.LocalDate": ["java.time.LocalDate.of(2024, 3, 14)"],
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
    # A class the PROBE declares. caturra has no class file for a library
    # class, so its member questions are refused rather than answered with an
    # empty list — and a refusal aborts the run, which costs this sweep every
    # call after it. The probe's own class is the representative receiver
    # anyway: reflection here answers about the classes a program declares.
    "java.lang.Class": "Probe.class",
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
        r"^java\.io\.File\.(getAbsolute|getCanonical)|^java\.nio\.file\.Path\.toAbsolutePath",
        "caturra's filesystem is rooted at / and has no working directory",
    ),
    (
        r"^java\.util\.Comparator\.(naturalOrder|reverseOrder|nullsFirst|nullsLast)",
        "a comparator caturra synthesized is not one of a JDK's named classes",
    ),
    (
        r"^java\.nio\.file\.Path\w*\.\w+\(java\.lang\.String[,)][^#]*#[45]\b",
        "a JDK validates the TEXT of a path — a NUL is refused outright and a "
        "lone surrogate cannot be encoded in the platform charset — where "
        "caturra's filesystem is in memory and takes any string as a name",
    ),
    (
        r"^java\.lang\.String\.format|^java\.time\.format\.DateTimeFormatter\.ofPattern",
        "a format TEMPLATE is read as Rust text, so an unpaired surrogate in "
        "one becomes U+FFFD where a JDK carries it through and shows `?`. The "
        "arguments and the result keep their units; only the template itself "
        "is converted, and threading units through the whole formatter for a "
        "template nobody writes is not worth what it would cost",
    ),
    (
        r"^java\.util\.Collections\.nCopies",
        "a JDK's copies list is LAZY — it stores the value once and answers "
        "`size()` from a number — where caturra's really holds that many "
        "references, so a count near `Integer.MAX_VALUE` is an "
        "OutOfMemoryError here and a list there",
    ),
    (
        r"^java\.util\.stream\.\w*Stream\.flatMap",
        "caturra's `flatMap` runs its function when the pipeline is BUILT and "
        "a JDK's when a terminal pulls, so a function that answers something "
        "other than a stream is refused at a different moment — the answer is "
        "the same once anything asks for it",
    ),
    (
        r"^java\.util\.(Set|Map)\.(of|copyOf|ofEntries)",
        "a JDK's immutable Set and Map iterate in an order randomized per JVM "
        "run (the SALT), so the same call answers `[a, b]` on one run and "
        "`[b, a]` on the next; caturra's is stable, which is the only thing a "
        "second engine can be",
    ),
    (
        r"^java\.io\.InputStreamReader\.(read|skip|transferTo)",
        "the JDK side runs with stdin CLOSED, so a reader over System.in "
        "throws where caturra's console — which has an input box behind it — "
        "answers end of input; a fact about the harness, not about either "
        "engine",
    ),
    (
        r"^java\.util\.Scanner\.(toString|reset|useDelimiter|useRadix|useLocale|skip)",
        "a JDK's Scanner prints its internal bookkeeping — `need input`, "
        "`skipped`, and a `source closed` that turns true when the source runs "
        "dry rather than when `close()` is called — none of which caturra "
        "models; it answers Object's default rather than guess at six fields. "
        "Every method that ANSWERS the scanner reads back through that same "
        "toString, so the configuration setters are the same one fact",
    ),
    (
        r"^java\.util\.stream\.(Int|Long|Double)Stream\.iterator",
        "the three primitive streams share one method table, so which adapter family "
        "a cursor came from is not recorded (an OBJECT stream's is exact)",
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
    """The calls to exercise, and the overloads no argument bank could build.

    A receiver that is not itself DETERMINISTIC would make every value on the
    class noise — `UUID.randomUUID()` was one, and its bits read as a finding
    on each run. `measure.py` does not care (it never runs anything), so this
    is checked here.
    """
    if re.search(r"random|now\(\)|currentTimeMillis|nanoTime", receiver):
        return [], [f"{class_name}: {receiver} is not the same twice"]
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


def simple_type(name):
    """A parameter type as caturra's diagnostics spell it: `[C` is `char[]`,
    `java.lang.String` is `String`."""
    primitive = {
        "[I": "int[]", "[J": "long[]", "[D": "double[]", "[C": "char[]",
        "[B": "byte[]", "[S": "short[]", "[F": "float[]", "[Z": "boolean[]",
    }
    if name in primitive:
        return primitive[name]
    if name.startswith("[L") and name.endswith(";"):
        return name[2:-1].rsplit(".", 1)[-1] + "[]"
    return name.rsplit(".", 1)[-1]


def call_signature(label):
    """`write([C,int,int)#0` -> `("write", ("char[]", "int", "int"))`."""
    name, rest = label.split("(", 1)
    params = rest.rsplit(")", 1)[0]
    return (name, tuple(simple_type(p) for p in params.split(",") if p))


def refused_signatures(message):
    """The calls caturra's diagnostic actually named, by signature.

    Its "no suitable method found for write(char[])" says which OVERLOAD was
    not offered; the name alone says only which family it was in.
    """
    found = set()
    for name, args in re.findall(r"for (\w+)\(([^)]*)\)", message):
        if args.strip() in ("", "no arguments"):
            found.add((name, ()))
        else:
            found.add((name, tuple(a.strip() for a in args.split(","))))
    return found


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
        # A TIMEOUT on either side is an answer, not an accident: a run that
        # never ends is as bad as a crash, and letting the exception out killed
        # the whole sweep at whichever class reached it first. Reported like a
        # crash, so the bisect drops the call that caused it.
        try:
            jdk = subprocess.run(
                ["java", "-cp", directory, "Probe"],
                capture_output=True, text=True, timeout=120, cwd=directory,
            ).stdout
        except subprocess.TimeoutExpired:
            return None, "a JDK did not finish in 120s"
        try:
            result = subprocess.run(
                [ENGINE, path, "Probe"], capture_output=True, text=True, cwd=REPO, timeout=120
            )
        except subprocess.TimeoutExpired:
            return None, "caturra did not finish in 120s"
    try:
        answer = json.loads(result.stdout)
    except ValueError:
        # No JSON at all means the engine did not finish — a PANIC, which is
        # the worst answer there is and reads like a clean run to anything
        # comparing output. `scripts/fuzz/panics.py` asks this question of the
        # COMPILER; a crash while running is only visible here, and
        # `Scanner.nextInt(1)` was one.
        crash = next(
            (l for l in result.stderr.splitlines() if "panicked at" in l),
            (result.stderr.strip().splitlines() or [""])[-1],
        )
        return None, f"caturra produced no JSON — {crash}"
    if answer.get("error"):
        # The LINE, when the diagnostic carried one: it is what lets the bisect
        # drop the offending call rather than guess from the wording. Without
        # it a message that names no method — "incompatible types: String
        # cannot be converted to Integer", from an argument the bank chose
        # badly — cost the whole class its sweep.
        line = answer.get("line") or 0
        # EVERY line caturra complained about, so one round drops them all —
        # the same reading the javac branch already gets from its own list.
        every = ",".join(str(n) for n in answer.get("lines") or [line])
        return None, f"caturra:{every}: " + answer["error"].splitlines()[0]
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


def drop_by_bisection(class_name, calls):
    """The call list without the first call that makes the probe fail.

    The last resort, for a message that names nothing the bisect can act on.
    One `run_both` per halving, so it costs a handful of runs rather than one
    per call.
    """
    if len(calls) <= 1:
        return []
    low, high = 0, len(calls)
    while high - low > 1:
        middle = (low + high) // 2
        jdk, _ = run_both(probe_source(class_name, calls[:middle]))
        if jdk is None:
            high = middle
        else:
            low = middle
    return calls[:low] + calls[low + 1 :]


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
        #
        # Sixty rounds, not thirty: a class with many missing overloads spends
        # a round on each, and `java.util.Arrays` — 300 calls, a dozen shapes
        # javac itself would not take from the bank — ran the budget out and
        # reported the whole class as "did not run".
        jdk, cat = None, ""
        for _ in range(60):
            jdk, cat = run_both(probe_source(class_name, calls))
            if jdk is not None:
                break
            caturra_line = re.match(r"caturra:([\d,]+): ", cat)
            if cat.startswith("javac: "):
                # javac names the LINE, and the preamble is a fixed height.
                bad = {n - HEADER_LINES - 1 for n in javac_rejected_lines(cat)}
                keep = [c for at, c in enumerate(calls) if at not in bad]
                why = "javac would not take the probe"
            elif caturra_line and caturra_line.group(1) != "0":
                # ...and so does caturra, now that the harness passes them on —
                # ALL of them, so one round drops every bad call rather than
                # one per round.
                bad = {
                    int(n) - HEADER_LINES - 1
                    for n in caturra_line.group(1).split(",")
                    if n != "0"
                }
                keep = [c for n, c in enumerate(calls) if n not in bad]
                why = "caturra would not take the probe"
                # A LINE and a SIGNATURE both: caturra's "no suitable method
                # found for compare(int[],int,int,int[],int,int)" names the
                # line it is on AND the overload, and dropping only the line
                # spent one round per CALL. Eight primitive kinds x three
                # arguments exhausted the retry budget on one missing overload
                # and cost `java.util.Arrays` its whole sweep.
                by_signature = refused_signatures(cat)
                if by_signature:
                    wider = [
                        c for c in calls if call_signature(c[0]) not in by_signature
                    ]
                    if wider and len(wider) < len(keep):
                        keep = wider
                        why = "caturra offers no such overload"
            else:
                # caturra names the METHOD, not the line: an overload it does
                # not offer. (The measurement is name-level, so this is an
                # expected answer and not a divergence.)
                #
                # By the SIGNATURE, not the name. Dropping every call with the
                # name cost `write(String)` its run because `write(char[])` was
                # missing, and then reported both as missing: a list of 144
                # "overloads caturra does not offer" in which `Integer.parseInt`
                # and `BitSet.set` were flatly wrong.
                refused_sigs = refused_signatures(cat)
                keep = [c for c in calls if call_signature(c[0]) not in refused_sigs]
                why = "caturra offers no such overload"
                if not refused_sigs or len(keep) == len(calls):
                    # A message that names no signature at all: fall back to the
                    # name, and SAY that the drop was by name — the overloads
                    # beside it may work perfectly.
                    # ...and the REFUSALS, which name the method rather than
                    # the call: "Collection.spliterator exists in Java, but
                    # …" and "unknown native member: X.y". A refusal used to
                    # match neither pattern, so the loop gave up and the whole
                    # CLASS was lost — seven classes to `spliterator` alone,
                    # for a fact already declared.
                    named = set(
                        re.findall(
                            r"for (\w+)\(|method (\w+)\b"
                            r"|[\w.$]*\.(\w+) exists in Java"
                            r"|unknown native member: [\w/$]*\.?(\w+)",
                            cat,
                        )
                    )
                    names = {n for pair in named for n in pair if n}
                    keep = [c for c in calls if c[0].split("(")[0] not in names]
                    why = "dropped with a sibling overload caturra does not offer"
            if not keep or len(keep) == len(calls):
                # NOTHING could be dropped: a message that names neither a
                # line nor a signature nor a method. Rather than lose the
                # class, find the offending call by BISECTION — the smallest
                # prefix that still fails ends on it — and drop that one.
                # Without this a single refusal with an unrecognised wording
                # (`Collections.addAll` on a poly expression, an argument the
                # bank typed badly) cost the whole class its sweep.
                keep = drop_by_bisection(class_name, calls)
                why = "bisected: the message named nothing to drop"
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
