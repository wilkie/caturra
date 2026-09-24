#!/usr/bin/env python3
"""A random stream PIPELINE, end to end, against a real JDK.

    scripts/fuzz/pipelines.py [--seed N] [--programs N] [--runs N] [--out DIR]
    scripts/fuzz/run.py <DIR>

A pipeline is a source, a run of intermediate ops and a terminal, and almost
every defect this engine has had in stream land was at a JOIN between two of
them rather than in any one: an element type that survived `map` but not
`sorted`, a `findFirst` that wrapped a null, a collector whose result had no
type inline. `collections.py` puts one collection under a random sequence of
calls; this does the same for the pipeline, where the sequence is the shape of
the thing.

Each line prints the terminal's answer, and a line that THROWS prints the
exception instead — so an ordering or laziness difference lands on the
pipeline that caused it.
"""

import argparse
import os
import random
import sys

# A source, the element it yields, and whether it is a primitive pipeline.
SOURCES = [
    ('Stream.of("pear", "fig", "date", "fig")', "obj", "Stream<String>"),
    ('Stream.of("b", "a", "c")', "obj", "Stream<String>"),
    ("Stream.<String>empty()", "obj", "Stream<String>"),
    ('List.of("pear", "fig", "date").stream()', "obj", "Stream<String>"),
    ('new ArrayList<>(List.of("x", "yy", "zzz")).stream()', "obj", "Stream<String>"),
    ('new TreeSet<>(List.of("q", "a", "m")).stream()', "obj", "Stream<String>"),
    ('Arrays.stream(new String[] {"one", "two", "three"})', "obj", "Stream<String>"),
    ('Stream.iterate("a", s -> s + "a").limit(4)', "obj", "Stream<String>"),
    ('Stream.generate(() -> "g").limit(3)', "obj", "Stream<String>"),
    ('Arrays.stream("the quick brown fox".split(" "))', "obj", "Stream<String>"),
    ("IntStream.range(0, 5)", "int", "IntStream"),
    ("IntStream.rangeClosed(1, 4)", "int", "IntStream"),
    ("IntStream.of(3, 1, 4, 1, 5)", "int", "IntStream"),
    ("Arrays.stream(new int[] {9, -2, 7})", "int", "IntStream"),
    ('"hello".chars()', "int", "IntStream"),
    ("IntStream.iterate(1, v -> v * 2).limit(5)", "int", "IntStream"),
]

# (code, the element it yields afterwards) — `None` keeps the current one.
OBJECT_OPS = [
    ("filter(s -> s.length() > 2)", None),
    ('filter(s -> !s.startsWith("f"))', None),
    ('map(s -> s + "!")', None),
    ("map(String::toUpperCase)", None),
    ("sorted()", None),
    ("sorted(Comparator.reverseOrder())", None),
    ("sorted(Comparator.comparingInt(String::length))", None),
    ("distinct()", None),
    ("limit(2)", None),
    ("skip(1)", None),
    ("peek(s -> sink.append(s))", None),
    ('flatMap(s -> Stream.of(s, s + "2"))', None),
    ("takeWhile(s -> s.length() < 4)", None),
    ("dropWhile(s -> s.length() < 4)", None),
    ("mapToInt(String::length)", "int"),
]

INT_OPS = [
    ("filter(v -> v % 2 == 0)", None),
    ("map(v -> v + 1)", None),
    ("sorted()", None),
    ("distinct()", None),
    ("limit(3)", None),
    ("skip(1)", None),
    ("peek(v -> sink.append(v))", None),
    ("flatMap(v -> IntStream.of(v, -v))", None),
    ("takeWhile(v -> v < 4)", None),
    ("dropWhile(v -> v < 4)", None),
    ('mapToObj(v -> "n" + v)', "obj"),
    ("asLongStream().mapToInt(v -> (int) v)", None),
    ("boxed().mapToInt(v -> v)", None),
]

OBJECT_TERMINALS = [
    "count()",
    "collect(Collectors.toList())",
    "collect(Collectors.toCollection(TreeSet::new))",
    'collect(Collectors.joining(","))',
    'collect(Collectors.joining(",", "[", "]"))',
    "collect(Collectors.groupingBy(String::length))",
    "collect(Collectors.groupingBy(s -> s.charAt(0), Collectors.counting()))",
    "collect(Collectors.partitioningBy(s -> s.length() > 2))",
    "collect(Collectors.toMap(s -> s, String::length, (a, b) -> a))",
    "collect(Collectors.summingInt(String::length))",
    "collect(Collectors.averagingInt(String::length))",
    "collect(Collectors.mapping(String::toUpperCase, Collectors.toList()))",
    "findFirst()",
    "anyMatch(s -> s.isEmpty())",
    "allMatch(s -> s.length() > 0)",
    "noneMatch(s -> s.isEmpty())",
    "min(Comparator.naturalOrder())",
    "max(Comparator.comparingInt(String::length))",
    'reduce("", (a, b) -> a + b)',
    "reduce((a, b) -> a + b)",
    "toArray().length",
    # `iterator()` is deliberately NOT here: caturra materializes the pipeline
    # when a cursor is asked for, where a JDK pulls, so every pipeline with a
    # `peek` before one differs in what the sink holds — a divergence written
    # down in specs/LANGUAGE.md ("A random pipeline, end to end") rather than
    # fixed. Generating it would make this sweep report the same known thing on
    # most seeds, which is how a gate stops being read.
]

INT_TERMINALS = [
    "count()",
    "sum()",
    "average()",
    "min()",
    "max()",
    "summaryStatistics()",
    "boxed().collect(Collectors.toList())",
    "anyMatch(v -> v < 0)",
    "allMatch(v -> v < 100)",
    "reduce(0, (a, b) -> a + b)",
    "reduce((a, b) -> a * b)",
    "findFirst()",
    "toArray().length",
    "mapToObj(Integer::toString).collect(Collectors.joining(\"|\"))",
]


def pipeline(rng, ops):
    """One pipeline: a source, `ops` intermediate steps, and a terminal."""
    source, element, _ = rng.choice(SOURCES)
    steps = [source]
    for _ in range(ops):
        table = OBJECT_OPS if element == "obj" else INT_OPS
        code, becomes = rng.choice(table)
        steps.append(code)
        element = becomes or element
    terminal = rng.choice(OBJECT_TERMINALS if element == "obj" else INT_TERMINALS)
    steps.append(terminal)
    return ".".join(steps)


def program(rng, index, runs, ops):
    lines = []
    for at in range(runs):
        text = pipeline(rng, rng.randint(0, ops))
        label = f"{at:02d}"
        # Each pipeline is its own statement, and the `sink` a `peek` writes to
        # is printed with the answer: a peek that runs at a different MOMENT is
        # the laziness difference this is looking for.
        lines.append(f'        step("{label}", () -> {text});')
    return TEMPLATE.format(index=index, calls="\n".join(lines))


TEMPLATE = """\
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.List;
import java.util.TreeSet;
import java.util.stream.Collectors;
import java.util.stream.IntStream;
import java.util.stream.Stream;

public class PipeFuzz{index} {{
    interface Body {{ Object get(); }}
    static StringBuilder sink = new StringBuilder();

    static void step(String label, Body body) {{
        sink.setLength(0);
        try {{
            Object answer = body.get();
            System.out.println(label + " -> " + answer + " | " + sink);
        }} catch (Throwable e) {{
            System.out.println(label + " ! " + e.getClass().getName() + ": " + e.getMessage()
                + " | " + sink);
        }}
    }}

    public static void main(String[] args) {{
{calls}
    }}
}}
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=20260923)
    parser.add_argument("--programs", type=int, default=4)
    parser.add_argument("--runs", type=int, default=40, help="pipelines per program")
    parser.add_argument("--ops", type=int, default=4, help="most intermediate ops")
    parser.add_argument("--out", default="/tmp/pipefz/cases")
    args = parser.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(args.out, exist_ok=True)
    for index in range(args.programs):
        source = program(rng, index, args.runs, args.ops)
        open(os.path.join(args.out, f"PipeFuzz{index}.java"), "w", encoding="utf-8").write(source)
    print(f"{args.programs} programs in {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
