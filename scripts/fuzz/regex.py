#!/usr/bin/env python3
"""caturra's own regular-expression engine against a real JDK's.

`crates/caturra-vm/src/regex.rs` is a hand-written backtracking engine over
UTF-16 code units. It is used by `String.matches`, `split`, `replaceAll` and the
whole of `Pattern`/`Matcher`, and its input space — quantifiers, classes,
groups, alternation, anchors, boundaries, backreferences — is far larger than
any battery of hand-written cases can walk. So: generate patterns from a
grammar, run them against inputs drawn from the SAME grammar (so a fair share
of them match), and compare every observable answer.

    scripts/fuzz/regex.py [--seed N] [--count N] [--out DIR]
    scripts/fuzz/run.py <DIR>

What is compared, per (pattern, input): whether it matches, every `find`'s
start/end/group, the group count and each group's text, `split`, `replaceAll`,
and the exception a bad pattern throws.
"""

import argparse
import os
import random
import sys

# Pieces that appear in student regexes, and the ones that stress a
# backtracking engine: nested quantifiers, alternation inside groups,
# reluctant and possessive forms, boundaries, backreferences.
# Weighted toward what actually appears in a program: literals and small
# classes, with the awkward ones present but rarer. A pattern nobody could
# match tests only the failure path.
ATOMS = (
    ["a", "b", "c", "0", "1", " "] * 4
    + ["[abc]", "[a-c]", "[01]", "[^a]", ".", "\\\\d", "\\\\w", "\\\\s"] * 2
    + ["\\\\D", "\\\\W", "\\\\S", "[.]", "\\\\.", "x"]
    # The named properties and the two Perl whitespace classes. The ASCII
    # POSIX names and the Unicode ones of the same name mean DIFFERENT sets,
    # which is most of what there is to get wrong here.
    + [
        "\\\\p{Alpha}", "\\\\p{Digit}", "\\\\p{Punct}", "\\\\p{Lower}",
        "\\\\p{Alnum}", "\\\\p{Space}", "\\\\p{XDigit}", "\\\\p{Graph}",
        "\\\\p{L}", "\\\\p{Lu}", "\\\\p{Nd}", "\\\\p{P}", "\\\\P{L}",
        "\\\\p{IsAlphabetic}", "\\\\p{IsLatin}", "\\\\p{InBasicLatin}",
        "\\\\p{javaLowerCase}", "\\\\p{javaJavaIdentifierPart}",
        "[\\\\p{L}&&[^\\\\p{Lu}]]", "[\\\\p{Alpha}\\\\d]",
        "\\\\h", "\\\\v", "\\\\H", "\\\\V", "\\\\X", "\\\\R",
    ]
)
QUANTIFIERS = [
    "", "", "", "?", "*", "+", "{2}", "{1,3}", "{2,}", "??", "*?", "+?", "*+", "++", "?+", "{1,2}+",
]
ANCHORS = ["^", "$", "\\\\b", "\\\\B", "\\\\b{g}", "\\\\G"]

# The inline flags, which change what the classes above MEAN.
FLAGS = ["", "", "", "", "(?i)", "(?m)", "(?s)", "(?d)", "(?U)", "(?U)(?i)", "(?dm)"]


def atom(rng, depth):
    """One piece of a pattern, sometimes a group of pieces."""
    if depth > 0 and rng.random() < 0.22:
        inner = "".join(atom(rng, depth - 1) for _ in range(rng.randint(1, 3)))
        if rng.random() < 0.3:
            inner += "|" + "".join(atom(rng, depth - 1) for _ in range(rng.randint(1, 2)))
        kind = rng.choice(["(", "(", "(", "(?:", "(?=", "(?!", "(?>"])
        # A lookahead with a quantifier after it is not what a program writes.
        if kind in ("(?=", "(?!"):
            return kind + inner + ")"
        return kind + inner + ")" + rng.choice(QUANTIFIERS)
    return rng.choice(ATOMS) + rng.choice(QUANTIFIERS)


def pattern(rng):
    body = "".join(atom(rng, 2) for _ in range(rng.randint(1, 3)))
    body = rng.choice(FLAGS) + body
    # An anchor belongs at the END it anchors: `$` in front matches nothing at
    # all, and a pattern nothing can match tests only the failure path.
    if rng.random() < 0.12:
        body = rng.choice(["^", "\\\\b", "\\\\B"]) + body
    if rng.random() < 0.12:
        body += rng.choice(["$", "\\\\b"])
    if rng.random() < 0.08:
        # A backreference, which only a backtracking engine can answer.
        body = "(" + rng.choice(ATOMS) + ")" + body + "\\\\1"
    return body


def plausible(rng, source):
    """A string built from the pattern's own LITERAL characters — so a fair
    share of the inputs reach the interesting paths instead of failing at the
    first atom."""
    letters = [c for c in source if c.isalnum() or c == " "]
    if not letters:
        return subject(rng)
    return "".join(rng.choice(letters) for _ in range(rng.randint(1, 6)))


def subject(rng):
    # The same alphabet the atoms are drawn from, so a fair share of the
    # inputs actually match something.
    alphabet = "aabbc01  .x"
    return "".join(rng.choice(alphabet) for _ in range(rng.randint(0, 10)))


def java_literal(text):
    """`text` as it must be written inside a Java string literal."""
    return text.replace("\\", "\\\\").replace('"', '\\"')


def program(rng, index, count):
    cases = []
    for _ in range(count):
        pat = pattern(rng)
        for _ in range(3):
            cases.append((pat, subject(rng)))
        # And two built from the pattern's own characters.
        cases.append((pat, plausible(rng, pat)))
        cases.append((pat, plausible(rng, pat)))
    lines = "\n".join(
        f'        probe("{java_literal(pat)}", "{java_literal(text)}");' for pat, text in cases
    )
    return f"""import java.util.Arrays;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class RegexFuzz{index} {{
    static void probe(String source, String text) {{
        try {{
            Pattern p = Pattern.compile(source);
            Matcher m = p.matcher(text);
            StringBuilder out = new StringBuilder();
            out.append(source).append(" ~ ").append(text).append(" | ");
            out.append(m.matches()).append(" ");
            m.reset();
            while (m.find()) {{
                out.append("[").append(m.start()).append(",").append(m.end()).append(")");
                out.append(m.group());
                for (int g = 1; g <= m.groupCount(); g++) {{
                    out.append("<").append(m.group(g)).append(">");
                }}
                // A zero-width match must still advance, or this never ends.
                if (m.end() == m.start() && m.end() >= text.length()) {{
                    break;
                }}
            }}
            out.append(" | ").append(Arrays.toString(text.split(source)));
            out.append(" | ").append(text.replaceAll(source, "<$0>"));
            out.append(" | ").append(text.matches(source));
            System.out.println(out);
        }} catch (Throwable e) {{
            System.out.println(source + " ~ " + text + " ! " + e.getClass().getName()
                + ": " + e.getMessage());
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
    parser.add_argument("--count", type=int, default=60, help="patterns per program")
    parser.add_argument("--programs", type=int, default=4)
    parser.add_argument("--out", default="/tmp/regexfz/cases")
    args = parser.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(args.out, exist_ok=True)
    for index in range(args.programs):
        source = program(rng, index, args.count)
        path = os.path.join(args.out, f"RegexFuzz{index}.java")
        open(path, "w", encoding="utf-8").write(source)
    print(f"{args.programs} programs in {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
