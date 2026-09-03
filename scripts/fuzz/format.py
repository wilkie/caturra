#!/usr/bin/env python3
"""`String.format` against a real JDK's `java.util.Formatter`.

`crates/caturra-vm/src/format.rs` is a hand-written implementation of Java's
formatter: conversions, flags, width, precision, argument indexes, and — the
part most easily got wrong — the exact exception for every combination that is
not allowed. The space is a product of five independent choices, which is too
large to walk by hand and exactly the shape a generator is for.

    scripts/fuzz/format.py [--seed N] [--out DIR]
    scripts/fuzz/run.py <DIR>

Every probe prints the formatted text, or the exception's class and message —
so a wrong ANSWER and a wrong REFUSAL are both caught.
"""

import argparse
import re
import os
import random
import sys

CONVERSIONS = list("bBsScCdoxXeEfgGhH%n") + [
    # The DATE-TIME conversions are two characters, and which of the two a
    # failure names is its own rule.
    "t" + suffix
    for suffix in "HIklMSLNpzZsQBbhAaCYyjmdeRTrDFc"
] + ["T" + suffix for suffix in "HMSpBbAaYymdRTrDFc"]
FLAGS = ["", "", "", "-", "+", "0", ",", "(", " ", "#", "-,", "+0", ",(", "0,"]
WIDTHS = ["", "", "", "1", "3", "8", "12", "20"]
PRECISIONS = ["", "", "", ".0", ".1", ".3", ".8"]

# One argument of each type a program passes, written as Java source.
ARGUMENTS = [
    "42", "-7", "0", "Integer.MAX_VALUE", "Integer.MIN_VALUE",
    "3.5", "-0.125", "0.0", "-0.0", "1e10", "1e-10", "Double.NaN",
    "Double.POSITIVE_INFINITY", "Double.MAX_VALUE", "Double.MIN_VALUE",
    "123456789L", "-1L", "Long.MIN_VALUE",
    '"text"', '""', '"a longer string"', "null",
    "'x'", "'\\n'", "true", "false",
    "(byte) 5", "(short) -300", "3.5f",
    # NOT an array: its `toString` carries an identity hash, which differs
    # between runs of the same JDK — a value the reference cannot reproduce
    # compares nothing.
    "java.util.List.of(1, 2)",
    # The `java.time` values a date-time conversion takes — and the two
    # AMOUNTS, which it does not.
    "java.time.LocalDate.of(2024, 2, 29)",
    "java.time.LocalTime.of(13, 45, 30, 123456789)",
    "java.time.LocalDateTime.of(2024, 2, 29, 0, 5, 9)",
    "java.time.LocalDate.of(-44, 3, 15)",
    "java.time.Duration.ofHours(2)",
    "java.time.Period.ofDays(3)",
]

# What a date-time conversion may be handed: the `java.time` values, and the
# other types it must REFUSE — but never a number, which would be a question
# about the host's time zone.
TEMPORAL = [
    "java.time.LocalDate.of(2024, 2, 29)",
    "java.time.LocalTime.of(13, 45, 30, 123456789)",
    "java.time.LocalDateTime.of(2024, 2, 29, 0, 5, 9)",
    "java.time.LocalDate.of(-44, 3, 15)",
    "java.time.Duration.ofHours(2)",
    "java.time.Period.ofDays(3)",
    '"text"',
    "null",
    "true",
]
# NOT `Month.MAY` or `DayOfWeek.FRIDAY`: an ENUM's `hashCode` is its identity
# hash, so a `%h` of one differs between two runs of the same JDK — the same
# noise class as an array's `toString`. The construct table pin covers what
# they answer for every date-time conversion.


def specifier(rng):
    conversion = rng.choice(CONVERSIONS)
    if conversion in "%n":
        # These take no argument, and only `%` takes flags at all.
        return "%" + (rng.choice(["", "5"]) if conversion == "%" else "") + conversion
    return (
        "%"
        + rng.choice(FLAGS)
        + rng.choice(WIDTHS)
        + rng.choice(PRECISIONS)
        + conversion
    )


def template(rng):
    """A format string: literal text around one to three specifiers."""
    parts = []
    for _ in range(rng.randint(1, 3)):
        parts.append(rng.choice(["", "x", " ", "[", "-"]))
        parts.append(specifier(rng))
    parts.append(rng.choice(["", "]", "!"]))
    return "".join(parts)


def java_literal(text):
    return text.replace("\\", "\\\\").replace('"', '\\"')


def program(rng, index, count):
    probes = []
    for _ in range(count):
        shape = template(rng)
        # As many arguments as there are specifiers, give or take one — a
        # missing argument and a spare one are both worth comparing.
        wanted = shape.count("%") - shape.count("%%") - shape.count("%n")
        wanted = max(0, wanted + rng.choice([0, 0, 0, -1, 1]))
        # A DATE-TIME conversion is only ever handed a `java.time` value.
        # Over a `long` it means milliseconds read in the DEFAULT TIME ZONE,
        # and the CLI host answers UTC where a JDK answers its own — the same
        # reason `LocalDate.now()` is not compared either.
        # (The flags, width and precision sit BETWEEN the `%` and the `t`,
        # so looking for the two characters side by side misses most of them.)
        dated = re.search(r"%[-+ 0,(#<0-9.$]*[tT]", shape) is not None
        pool = TEMPORAL if dated else ARGUMENTS
        args = ", ".join(rng.choice(pool) for _ in range(wanted))
        probes.append((shape, args))
    lines = "\n".join(
        f'        probe("{java_literal(shape)}"{", " + args if args else ""});'
        for shape, args in probes
    )
    return f"""public class FormatFuzz{index} {{
    static void probe(String shape, Object... args) {{
        try {{
            System.out.println(shape + " => [" + String.format(shape, args) + "]");
        }} catch (Throwable e) {{
            System.out.println(shape + " ! " + e.getClass().getName() + ": " + e.getMessage());
        }}
    }}

    public static void main(String[] args) {{
{lines}
    }}
}}
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=20260902)
    parser.add_argument("--programs", type=int, default=4)
    parser.add_argument("--count", type=int, default=150, help="probes per program")
    parser.add_argument("--out", default="/tmp/formatfz/cases")
    args = parser.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(args.out, exist_ok=True)
    for index in range(args.programs):
        source = program(rng, index, args.count)
        open(os.path.join(args.out, f"FormatFuzz{index}.java"), "w", encoding="utf-8").write(source)
    print(f"{args.programs} programs in {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
