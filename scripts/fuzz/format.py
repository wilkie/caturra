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
import os
import random
import sys

CONVERSIONS = list("bBsScCdoxXeEfgGhH%n")
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
]


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
        args = ", ".join(rng.choice(ARGUMENTS) for _ in range(wanted))
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
