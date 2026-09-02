#!/usr/bin/env python3
"""The collections, one random operation at a time, against a real JDK.

A collection's ORDER and its return VALUES are where a reimplementation
drifts: which element `remove` answers, what `put` gives back, where an
insertion lands, what the iterator sees after a resize, what `toString` shows.
caturra models the JDK's exact iteration order (including a `HashMap`'s bucket
order), and that is a claim worth putting under load rather than under a
handful of cases.

    scripts/fuzz/collections.py [--seed N] [--out DIR]
    scripts/fuzz/run.py <DIR>

Each program builds one collection and applies a random sequence of
operations, printing the answer of every call AND the whole collection after
it — so a divergence is caught at the operation that caused it, not ten steps
later.
"""

import argparse
import os
import random
import sys

# (declaration, how to make one) — the shapes a program actually declares.
KINDS = [
    ("List<String>", "new ArrayList<>()", "list"),
    ("List<String>", "new LinkedList<>()", "list"),
    ("Set<String>", "new HashSet<>()", "set"),
    ("Set<String>", "new LinkedHashSet<>()", "set"),
    ("Set<String>", "new TreeSet<>()", "sorted-set"),
    ("Map<String, Integer>", "new HashMap<>()", "map"),
    ("Map<String, Integer>", "new LinkedHashMap<>()", "map"),
    ("Map<String, Integer>", "new TreeMap<>()", "sorted-map"),
    ("Deque<String>", "new ArrayDeque<>()", "deque"),
    ("Queue<String>", "new PriorityQueue<>()", "queue"),
]

KEYS = ["a", "b", "c", "d", "e", "f", "g", "h", "zz", "A", "1", "10", "2"]

# The calls that answer nothing, which cannot be printed as an expression.
VOID_CALLS = ("add(0,", "add(Math", "addFirst", "addLast")


def operations(rng, kind, count):
    """A random sequence of calls on a collection of this kind."""
    out = []
    key = lambda: rng.choice(KEYS)
    number = lambda: rng.randint(-3, 9)
    for _ in range(count):
        if kind == "list":
            out.append(
                rng.choice(
                    [
                        f'add("{key()}")',
                        f'add(0, "{key()}")',
                        f'add(Math.max(0, Math.min({number()}, c.size())), "{key()}")',
                        f'remove("{key()}")',
                        f'contains("{key()}")',
                        f'indexOf("{key()}")',
                        f'lastIndexOf("{key()}")',
                        "size()",
                        "isEmpty()",
                        f'set(c.isEmpty() ? 0 : Math.floorMod({number()}, c.size()), "{key()}")',
                        "c.isEmpty() ? null : c.get(0)",
                        "c.isEmpty() ? null : c.remove(0)",
                    ]
                )
            )
        elif kind in ("set", "sorted-set"):
            out.append(
                rng.choice(
                    [
                        f'add("{key()}")',
                        f'remove("{key()}")',
                        f'contains("{key()}")',
                        "size()",
                        "isEmpty()",
                    ]
                )
            )
        elif kind in ("map", "sorted-map"):
            out.append(
                rng.choice(
                    [
                        f'put("{key()}", {number()})',
                        f'remove("{key()}")',
                        f'get("{key()}")',
                        f'containsKey("{key()}")',
                        f'containsValue({number()})',
                        f'getOrDefault("{key()}", {number()})',
                        f'putIfAbsent("{key()}", {number()})',
                        "size()",
                        "keySet()",
                        "values()",
                    ]
                )
            )
        elif kind == "deque":
            out.append(
                rng.choice(
                    [
                        f'add("{key()}")',
                        f'addFirst("{key()}")',
                        f'addLast("{key()}")',
                        f'offer("{key()}")',
                        "poll()",
                        "pollLast()",
                        "peek()",
                        "peekLast()",
                        "size()",
                    ]
                )
            )
        else:
            out.append(
                rng.choice(
                    [
                        f'add("{key()}")',
                        f'offer("{key()}")',
                        "poll()",
                        "peek()",
                        "size()",
                        f'contains("{key()}")',
                    ]
                )
            )
    return out


def order_run(rng, run):
    """A hash collection large enough to RESIZE, printed in iteration order.

    A `HashMap`'s order is its bucket order, which changes as the table grows
    — every insert past 12, 24, 48 … entries rehashes the lot. It is the most
    detailed claim caturra makes about the collections, so it is worth walking
    with a hundred keys rather than a handful.
    """
    count = rng.choice([13, 25, 40, 64, 100])
    prefix = rng.choice(["k", "key", "", "x"])
    style = rng.choice(["number", "letter", "mixed"])
    return f"""    static void order{run}() {{
        System.out.println("--- order {style} {count} {prefix}");
        Map<String, Integer> m = new HashMap<>();
        Set<String> s = new HashSet<>();
        for (int i = 0; i < {count}; i++) {{
            String k = "{prefix}" + ({"i" if style == "number" else
                                      "(char) ('a' + i % 26)" if style == "letter" else
                                      "i + \"\" + (char) ('a' + i % 7)"});
            m.put(k, i);
            s.add(k);
            if (i % 7 == 3) {{
                m.remove(k);
                s.remove(k);
            }}
        }}
        System.out.println(m);
        System.out.println(s);
        System.out.println(m.keySet() + " " + m.values());
        System.out.println(m.size() + " " + s.size() + " " + m.hashCode() + " " + s.hashCode());
        StringBuilder walked = new StringBuilder();
        for (Map.Entry<String, Integer> e : m.entrySet()) {{
            walked.append(e.getKey()).append("=").append(e.getValue()).append(" ");
        }}
        System.out.println(walked);
    }}"""


def program(rng, index, runs, steps):
    blocks = []
    for run in range(runs):
        declared, made, kind = rng.choice(KINDS)
        calls = operations(rng, kind, steps)
        lines = [f"        {declared} c = {made};", f'        System.out.println("--- {made}");']
        for call in calls:
            if call.startswith("c."):
                expression = call
            else:
                expression = f"c.{call}"
            # Every call prints its ANSWER and the collection after it, so a
            # divergence lands on the operation that caused it. A call that
            # answers NOTHING is run first and reported as void, since a void
            # expression cannot be an argument.
            label = call.replace(chr(34), chr(39))
            # Wrapped, because an operation that THROWS is worth comparing
            # too — and because an uncaught one would take the rest of the
            # run with it.
            if any(call.startswith(prefix) for prefix in VOID_CALLS):
                lines.append(
                    f'        try {{ {expression}; step("{label}", "void", c); }}'
                    f' catch (Throwable e) {{ failed("{label}", e, c); }}'
                )
            else:
                lines.append(
                    f'        try {{ step("{label}", {expression}, c); }}'
                    f' catch (Throwable e) {{ failed("{label}", e, c); }}'
                )
        # `hashCode` is only printed where the interface DEFINES it: a
        # `Deque` and a `Queue` inherit `Object`'s, which is an identity hash
        # and differs between runs of the same JDK.
        tail = "c + \" \" + c.size()"
        if kind not in ("deque", "queue"):
            tail += " + \" \" + c.hashCode()"
        lines.append(f"        System.out.println({tail});")
        blocks.append("    static void run%d() {\n%s\n    }" % (run, "\n".join(lines)))
    orders = [order_run(rng, run) for run in range(3)]
    calls = "\n".join(
        [f"        run{run}();" for run in range(runs)]
        + [f"        order{run}();" for run in range(len(orders))]
    )
    bodies = "\n\n".join(blocks + orders)
    return f"""import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.PriorityQueue;
import java.util.Queue;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeSet;

public class CollFuzz{index} {{
    static void step(String what, Object answer, Object after) {{
        System.out.println(what + " -> " + answer + " | " + after);
    }}

    static void failed(String what, Throwable e, Object after) {{
        System.out.println(what + " ! " + e.getClass().getName() + ": " + e.getMessage()
            + " | " + after);
    }}

{bodies}

    public static void main(String[] args) {{
{calls}
    }}
}}
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=20260902)
    parser.add_argument("--programs", type=int, default=4)
    parser.add_argument("--runs", type=int, default=12, help="collections per program")
    parser.add_argument("--steps", type=int, default=25, help="operations per collection")
    parser.add_argument("--out", default="/tmp/collfz/cases")
    args = parser.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(args.out, exist_ok=True)
    for index in range(args.programs):
        source = program(rng, index, args.runs, args.steps)
        open(os.path.join(args.out, f"CollFuzz{index}.java"), "w", encoding="utf-8").write(source)
    print(f"{args.programs} programs in {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
