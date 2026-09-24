#!/usr/bin/env python3
"""One MINIMAL program per position that takes a lambda, against a real JDK.

usage: lambdas.py [name ...] [--jobs N] [--self-check]

A lambda needs two things caturra decides separately: the position has to be a
functional-interface one (the lambda pass must know what the call wants), and
the bundled interface it desugars to has to be INJECTED into the program. The
second is decided by scanning the source text, and a pin program says far more
than the feature it pins — `Collections.sort` in a helper is enough to inject
the interface a directory filter erases to. So the filtered listing passed its
own two pins and failed in every program that mentioned no comparator and no
stream: "incompatible types: lambda expression cannot be converted to
BiFunction".

Each case here is therefore written to name NOTHING the position does not
need: the import it requires, the call, and the lambda. What it measures is
the position itself rather than the company it keeps.
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile
from concurrent.futures import ProcessPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from compile import REPO  # noqa: E402

RUN = os.path.join(REPO, "target/release/examples/compatrun")

CASES = {}


def case(name, imports, body, members=""):
    """One program: the imports the position needs, and the call that takes a lambda."""
    CASES[name] = (
        "".join(f"import {i};\n" for i in imports)
        + "public class Main {\n"
        + "".join(f"    {line.strip()}\n" for line in members.strip().splitlines() if line.strip())
        + "    public static void main(String[] args) throws Exception {\n"
        + "".join(f"        {line.strip()}\n" for line in body.strip().splitlines())
        + "    }\n}\n"
    )


LIST = ["java.util.ArrayList", "java.util.List"]
case(
    "list-forEach",
    LIST,
    """
    List<String> x = new ArrayList<>(List.of("a", "b"));
    x.forEach(s -> System.out.println(s));
    """,
)
case(
    "list-removeIf",
    LIST,
    """
    List<String> x = new ArrayList<>(List.of("a", "bb"));
    x.removeIf(s -> s.length() > 1);
    System.out.println(x);
    """,
)
case(
    "list-replaceAll",
    LIST,
    """
    List<String> x = new ArrayList<>(List.of("a", "b"));
    x.replaceAll(s -> s + "!");
    System.out.println(x);
    """,
)
case(
    "list-sort",
    LIST,
    """
    List<String> x = new ArrayList<>(List.of("ccc", "a"));
    x.sort((p, q) -> p.length() - q.length());
    System.out.println(x);
    """,
)
case(
    "list-stream-filter",
    LIST + ["java.util.stream.Collectors"],
    """
    List<String> x = new ArrayList<>(List.of("a", "bb"));
    System.out.println(x.stream().filter(s -> s.length() == 1).collect(Collectors.toList()));
    """,
)
case(
    "iterator-forEachRemaining",
    LIST,
    """
    List<String> x = new ArrayList<>(List.of("a", "b"));
    x.iterator().forEachRemaining(s -> System.out.println(s));
    """,
)

MAP = ["java.util.HashMap", "java.util.Map"]
case(
    "map-forEach",
    MAP,
    """
    Map<String, Integer> m = new HashMap<>();
    m.put("a", 1);
    m.forEach((k, v) -> System.out.println(k + v));
    """,
)
case(
    "map-compute",
    MAP,
    """
    Map<String, Integer> m = new HashMap<>();
    m.put("a", 1);
    m.compute("a", (k, v) -> v + 1);
    System.out.println(m);
    """,
)
case(
    "map-computeIfAbsent",
    MAP,
    """
    Map<String, Integer> m = new HashMap<>();
    m.computeIfAbsent("a", k -> k.length());
    System.out.println(m);
    """,
)
case(
    "map-merge",
    MAP,
    """
    Map<String, Integer> m = new HashMap<>();
    m.put("a", 1);
    m.merge("a", 5, (p, q) -> p + q);
    System.out.println(m);
    """,
)
case(
    "map-replaceAll",
    MAP,
    """
    Map<String, Integer> m = new HashMap<>();
    m.put("a", 1);
    m.replaceAll((k, v) -> v * 2);
    System.out.println(m);
    """,
)

OPT = ["java.util.Optional"]
case(
    "optional-map",
    OPT,
    """
    System.out.println(Optional.of("a").map(s -> s + "!").get());
    """,
)
case(
    "optional-flatMap",
    OPT,
    """
    System.out.println(Optional.of("a").flatMap(s -> Optional.of(s + "!")).get());
    """,
)
case(
    "optional-filter",
    OPT,
    """
    System.out.println(Optional.of("a").filter(s -> s.isEmpty()).isPresent());
    """,
)
case(
    "optional-ifPresent",
    OPT,
    """
    Optional.of("a").ifPresent(s -> System.out.println(s));
    """,
)
case(
    "optional-ifPresentOrElse",
    OPT,
    """
    Optional.empty().ifPresentOrElse(s -> System.out.println(s), () -> System.out.println("none"));
    """,
)
case(
    "optional-or",
    OPT,
    """
    System.out.println(Optional.empty().or(() -> Optional.of("b")).get());
    """,
)
case(
    "optional-orElseGet",
    OPT,
    """
    System.out.println(Optional.empty().orElseGet(() -> "b"));
    """,
)
case(
    "optional-orElseThrow",
    OPT,
    """
    System.out.println(Optional.of("a").orElseThrow(() -> new IllegalStateException("x")));
    """,
)

ARR = ["java.util.Arrays"]
case(
    "arrays-sort",
    ARR,
    """
    String[] a = {"ccc", "a"};
    Arrays.sort(a, (x, y) -> x.length() - y.length());
    System.out.println(Arrays.toString(a));
    """,
)
case(
    "arrays-sort-range",
    ARR,
    """
    String[] a = {"dddd", "ccc", "a", "bb"};
    Arrays.sort(a, 1, 4, (x, y) -> x.length() - y.length());
    System.out.println(Arrays.toString(a));
    """,
)
case(
    "arrays-parallelSort",
    ARR,
    """
    String[] a = {"ccc", "a"};
    Arrays.parallelSort(a, (x, y) -> x.length() - y.length());
    System.out.println(Arrays.toString(a));
    """,
)
case(
    "arrays-setAll",
    ARR,
    """
    int[] a = new int[3];
    Arrays.setAll(a, i -> i * i);
    System.out.println(Arrays.toString(a));
    """,
)
case(
    "arrays-parallelSetAll",
    ARR,
    """
    int[] a = new int[3];
    Arrays.parallelSetAll(a, i -> i * i);
    System.out.println(Arrays.toString(a));
    """,
)
case(
    "arrays-parallelPrefix",
    ARR,
    """
    int[] a = {1, 2, 3};
    Arrays.parallelPrefix(a, (x, y) -> x + y);
    System.out.println(Arrays.toString(a));
    """,
)
case(
    "arrays-stream-map",
    ARR,
    """
    String[] a = {"a", "bb"};
    System.out.println(Arrays.stream(a).map(s -> s.length()).count());
    """,
)

COLL = ["java.util.ArrayList", "java.util.Collections", "java.util.List"]
case(
    "collections-sort",
    COLL,
    """
    List<String> x = new ArrayList<>(List.of("ccc", "a"));
    Collections.sort(x, (p, q) -> p.length() - q.length());
    System.out.println(x);
    """,
)
case(
    "collections-max",
    COLL,
    """
    List<String> x = new ArrayList<>(List.of("ccc", "a"));
    System.out.println(Collections.max(x, (p, q) -> p.length() - q.length()));
    """,
)
case(
    "collections-binarySearch",
    COLL,
    """
    List<String> x = new ArrayList<>(List.of("a", "ccc"));
    System.out.println(Collections.binarySearch(x, "ccc", (p, q) -> p.length() - q.length()));
    """,
)
case(
    "treeset-ctor",
    ["java.util.TreeSet"],
    """
    TreeSet<String> t = new TreeSet<>((p, q) -> p.length() - q.length());
    t.add("a");
    t.add("bb");
    System.out.println(t);
    """,
)
case(
    "priorityqueue-ctor",
    ["java.util.PriorityQueue"],
    """
    PriorityQueue<String> q = new PriorityQueue<>((p, r) -> r.length() - p.length());
    q.add("a");
    q.add("bb");
    System.out.println(q.poll());
    """,
)
case(
    "comparator-comparing",
    ["java.util.ArrayList", "java.util.Comparator", "java.util.List"],
    """
    List<String> x = new ArrayList<>(List.of("ccc", "a"));
    x.sort(Comparator.comparing(s -> s.length()));
    System.out.println(x);
    """,
)

case(
    "file-list-filter",
    ["java.io.File"],
    """
    new File("fx").mkdirs();
    new File("fx/a.txt").createNewFile();
    System.out.println(dir().list((d, n) -> n.endsWith(".txt")).length);
    """,
    members="""
    static File dir() { return new File("fx"); }
    """,
)
case(
    "file-listFiles-filter",
    ["java.io.File"],
    """
    new File("fy").mkdirs();
    new File("fy/a.txt").createNewFile();
    System.out.println(new File("fy").listFiles(f -> f.isFile()).length);
    """,
)
case(
    "files-walk",
    ["java.nio.file.Files", "java.nio.file.Path"],
    """
    new java.io.File("fz").mkdirs();
    new java.io.File("fz/a.txt").createNewFile();
    System.out.println(Files.walk(Path.of("fz")).filter(p -> p.toString().endsWith(".txt")).count());
    """,
)

case(
    "objects-requireNonNull",
    ["java.util.Objects"],
    """
    System.out.println(Objects.requireNonNull("x", () -> "was null"));
    """,
)
case(
    "objects-requireNonNullElseGet",
    ["java.util.Objects"],
    """
    System.out.println(Objects.requireNonNullElseGet(null, () -> "b"));
    """,
)
case(
    "matcher-replaceAll-fn",
    ["java.util.regex.Pattern"],
    """
    System.out.println(Pattern.compile("a").matcher("aba").replaceAll(m -> m.group().toUpperCase()));
    """,
)
case(
    "pattern-splitAsStream",
    ["java.util.regex.Pattern"],
    """
    System.out.println(Pattern.compile(",").splitAsStream("a,,b").filter(s -> !s.isEmpty()).count());
    """,
)
case(
    "string-chars",
    [],
    """
    System.out.println("abc".chars().filter(c -> c > 97).count());
    """,
)
case(
    "interface-value",
    ["java.util.function.Function"],
    """
    Function<String, Integer> f = s -> s.length();
    System.out.println(f.apply("abcd"));
    """,
)
case(
    "runnable-value",
    [],
    """
    Runnable r = () -> System.out.println("ran");
    r.run();
    """,
)


# The functional interfaces' OWN combinators, which take a lambda like any
# other position — and whose receiver may be a cast, a call or a ternary rather
# than a variable. Found because the behaviour sweep's bank writes its
# receivers as casts, so `((Predicate<String>) (s -> true)).and(s -> …)` is the
# spelling that was refused.
case(
    "predicate-and",
    ["java.util.function.Predicate"],
    """
    Predicate<String> p = s -> s.isEmpty();
    System.out.println(p.and(s -> s.length() == 1).test("a"));
    """,
)
case(
    "predicate-and-cast-receiver",
    ["java.util.function.Predicate"],
    """
    System.out.println(((Predicate<String>) (s -> true)).and(s -> s.isEmpty()).test("a"));
    """,
)
case(
    "predicate-not",
    ["java.util.function.Predicate"],
    """
    System.out.println(Predicate.not(String::isEmpty).test("a"));
    """,
)
case(
    "function-andThen",
    ["java.util.function.Function"],
    """
    Function<String, String> f = s -> s + "1";
    System.out.println(f.andThen(s -> s + "2").apply("x"));
    """,
)
case(
    "function-compose",
    ["java.util.function.Function"],
    """
    Function<String, String> f = s -> s + "1";
    System.out.println(f.compose((String s) -> s + "0").apply("x"));
    """,
)
case(
    "consumer-andThen",
    ["java.util.function.Consumer"],
    """
    StringBuilder sb = new StringBuilder();
    Consumer<String> c = s -> sb.append(s);
    c.andThen(s -> sb.append(s.length())).accept("x");
    System.out.println(sb);
    """,
)
case(
    "comparator-thenComparing-cast-receiver",
    ["java.util.Comparator"],
    """
    System.out.println(((Comparator<String>) ((a, b) -> 0)).thenComparing(s -> s.length()).compare("ab", "b"));
    """,
)


def jdk(name, source, work):
    """What a real JDK does: the first line of a refusal, or the program's output."""
    here = os.path.join(work, name + ".jdk")
    os.makedirs(here, exist_ok=True)
    open(os.path.join(here, "Main.java"), "w", encoding="utf-8").write(source)
    built = subprocess.run(["javac", "Main.java"], cwd=here, capture_output=True, text=True)
    if built.returncode != 0:
        return "REFUSED: " + built.stderr.strip().splitlines()[0]
    ran = subprocess.run(["java", "Main"], cwd=here, capture_output=True, text=True, timeout=120)
    return (ran.stdout + ran.stderr).strip()


def caturra(name, source, work):
    """The same question of caturra, in the same shape."""
    here = os.path.join(work, name + ".cat")
    os.makedirs(here, exist_ok=True)
    open(os.path.join(here, "Main.java"), "w", encoding="utf-8").write(source)
    ran = subprocess.run(
        [RUN, "Main.java", "Main"], cwd=here, capture_output=True, text=True, timeout=120
    )
    try:
        answer = json.loads(ran.stdout)
    except ValueError:
        return "ABORT: " + (ran.stdout + ran.stderr)[:200]
    if not answer.get("ok"):
        return "REFUSED: " + answer.get("error", "").splitlines()[0]
    return (answer.get("stdout", "") + answer.get("stderr", "")).strip()


def probe(case_and_work):
    name, source, work = case_and_work
    return name, jdk(name, source, work), caturra(name, source, work)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("names", nargs="*", help="only these cases (default: all)")
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument(
        "--self-check",
        action="store_true",
        help="confirm every case really writes a lambda, and that a difference is reported",
    )
    args = parser.parse_args()

    if args.self_check:
        # A case with no arrow and no method reference in it measures nothing,
        # however green it reports — the blind spot this sweep exists to close.
        silent = [n for n, s in CASES.items() if "->" not in s and "::" not in s]
        for name in silent:
            print(f"=== SELF-CHECK FAILED: {name} writes no lambda")
        # ...and the comparison itself: two answers that differ must be
        # reported, and two that agree must not.
        pairs = [("a", "a", False), ("a", "b", True), ("1", "REFUSED: x", True)]
        wrong = [p for p in pairs if (p[0] != p[1]) is not p[2]]
        for pair in wrong:
            print(f"=== SELF-CHECK FAILED: {pair}")
        print(
            f"\nself-check: {len(CASES)} cases write a lambda, "
            f"{len(pairs)} comparisons judged, {len(silent) + len(wrong)} wrong"
        )
        return 1 if silent or wrong else 0

    chosen = {n: s for n, s in CASES.items() if not args.names or n in args.names}
    missing = [n for n in args.names if n not in CASES]
    for name in missing:
        print(f"no such case: {name}")
    bad = 0
    with tempfile.TemporaryDirectory() as work:
        cases = [(n, s, work) for n, s in chosen.items()]
        with ProcessPoolExecutor(args.jobs) as pool:
            for name, expected, actual in pool.map(probe, cases):
                if expected != actual:
                    bad += 1
                    print(f"-- {name}\n   jdk:     {expected!r}\n   caturra: {actual!r}")
    print(f"\n{len(chosen)} functional positions, {bad} disagreeing")
    return 1 if bad or missing else 0


if __name__ == "__main__":
    sys.exit(main())
