#!/usr/bin/env python3
"""A random CHAIN of collection views, and writes through both ends of it.

    scripts/fuzz/views.py [--seed N] [--programs N] [--chains N] [--steps N] [--out DIR]
    scripts/fuzz/run.py <DIR>

A view is a window on a collection, and the two hardest things about one are
what a write through it reaches and when a change underneath it is noticed.
Composing views is where both go wrong: a sub-range of a sub-range, the
`keySet()` of a `subMap()`, a cursor over a `subList` whose backing grows. Each
of those was a real defect, and each was invisible to a probe that took one
view at a time.

So: build a collection, wrap it in one to three views, then apply random
operations through the VIEW and through the BACKING, printing the answer of
every call and both containers after it. A call that throws prints its
exception, because half the answers here are `UnsupportedOperationException`,
`IllegalArgumentException` and `ConcurrentModificationException`.
"""

import argparse
import os
import random
import sys

# Each shape is (declared type, how the ops below are written). A view's shape
# decides what may be asked of it and what the NEXT view in the chain may be.
LIST, NAVSET, NAVMAP, VALUES, ENTRIES = "list", "navset", "navmap", "values", "entries"

BACKINGS = [
    ('List<String> back = new ArrayList<>(List.of("a", "b", "c", "d", "e"));', LIST),
    ('List<String> back = new LinkedList<>(List.of("a", "b", "c", "d"));', LIST),
    ('NavigableSet<String> back = new TreeSet<>(List.of("a", "b", "c", "d", "e"));', NAVSET),
    (
        "NavigableMap<String, Integer> back = new TreeMap<>("
        'Map.of("a", 1, "b", 2, "c", 3, "d", 4));',
        NAVMAP,
    ),
]

KEYS = ["a", "b", "c", "d", "e", "f"]


def views_for(rng, shape, name, into):
    """One view of `name`, its declaration appended to `into`; the new shape."""
    key = lambda: rng.choice(KEYS)
    if shape == LIST:
        # A sub-range of a list, which may itself be narrowed again.
        low = rng.randint(0, 2)
        high = rng.randint(low, 3)
        choice = rng.choice(["sub", "unmodifiable", "synchronized"])
        if choice == "sub":
            into.append(f"List<String> {name} = prev.subList({low}, Math.min({high}, prev.size()));")
        elif choice == "unmodifiable":
            into.append(f"List<String> {name} = Collections.unmodifiableList(prev);")
        else:
            into.append(f"List<String> {name} = Collections.synchronizedList(prev);")
        return LIST
    if shape == NAVSET:
        choice = rng.choice(["head", "tail", "sub", "descending", "unmodifiable"])
        if choice == "head":
            into.append(f'NavigableSet<String> {name} = prev.headSet("{key()}", {rng.choice(["true", "false"])});')
        elif choice == "tail":
            into.append(f'NavigableSet<String> {name} = prev.tailSet("{key()}", {rng.choice(["true", "false"])});')
        elif choice == "sub":
            lo, hi = sorted([key(), key()])
            into.append(
                f'NavigableSet<String> {name} = prev.subSet("{lo}", true, "{hi}", '
                f'{rng.choice(["true", "false"])});'
            )
        elif choice == "descending":
            into.append(f"NavigableSet<String> {name} = prev.descendingSet();")
        else:
            into.append(f"NavigableSet<String> {name} = new TreeSet<>(Collections.unmodifiableSortedSet(prev));")
        return NAVSET
    if shape == NAVMAP:
        choice = rng.choice(["head", "tail", "sub", "descending", "keys", "values", "entries"])
        if choice == "head":
            into.append(f'NavigableMap<String, Integer> {name} = prev.headMap("{key()}", {rng.choice(["true", "false"])});')
            return NAVMAP
        if choice == "tail":
            into.append(f'NavigableMap<String, Integer> {name} = prev.tailMap("{key()}", {rng.choice(["true", "false"])});')
            return NAVMAP
        if choice == "sub":
            lo, hi = sorted([key(), key()])
            into.append(
                f'NavigableMap<String, Integer> {name} = prev.subMap("{lo}", true, "{hi}", '
                f'{rng.choice(["true", "false"])});'
            )
            return NAVMAP
        if choice == "descending":
            into.append(f"NavigableMap<String, Integer> {name} = prev.descendingMap();")
            return NAVMAP
        if choice == "keys":
            into.append(f"NavigableSet<String> {name} = prev.navigableKeySet();")
            return NAVSET
        if choice == "values":
            into.append(f"Collection<Integer> {name} = prev.values();")
            return VALUES
        into.append(f"Set<Map.Entry<String, Integer>> {name} = prev.entrySet();")
        return ENTRIES
    return shape


def operations(rng, shape, name, count):
    """Random calls on a view of this shape, each wrapped so a throw is compared."""
    key = lambda: rng.choice(KEYS)
    out = []
    for _ in range(count):
        if shape == LIST:
            call = rng.choice([
                f"{name}.size()",
                f"{name}.isEmpty()",
                f'{name}.contains("{key()}")',
                f'{name}.indexOf("{key()}")',
                f"{name}.toString()",
                f'{name}.add("{key()}")',
                f'{name}.remove("{key()}")',
                f'{name}.set(0, "{key()}")',
                f"{name}.get(0)",
                f"walk({name})",
                f"{name}.stream().collect(java.util.stream.Collectors.joining(\",\"))",
                f"sortOf({name})",
            ])
        elif shape == NAVSET:
            call = rng.choice([
                f"{name}.size()",
                f'{name}.contains("{key()}")',
                f"{name}.toString()",
                f'{name}.add("{key()}")',
                f'{name}.remove("{key()}")',
                f"{name}.first()",
                f"{name}.last()",
                f'{name}.higher("{key()}")',
                f'{name}.floor("{key()}")',
                f"{name}.pollFirst()",
                f"{name}.pollLast()",
                f"walk({name})",
            ])
        elif shape == NAVMAP:
            call = rng.choice([
                f"{name}.size()",
                f'{name}.get("{key()}")',
                f'{name}.containsKey("{key()}")',
                f"{name}.toString()",
                f'{name}.put("{key()}", {rng.randint(0, 9)})',
                f'{name}.remove("{key()}")',
                f"{name}.firstEntry()",
                f"{name}.lastEntry()",
                f'{name}.headMap("{key()}").size()',
                f"{name}.keySet().toString()",
            ])
        elif shape == VALUES:
            call = rng.choice([
                f"{name}.size()",
                f"{name}.toString()",
                f"{name}.contains({rng.randint(0, 5)})",
                f"{name}.remove({rng.randint(0, 5)})",
                f"walkAny({name})",
            ])
        else:
            call = rng.choice([
                f"{name}.size()",
                f"{name}.toString()",
                f"walkAny({name})",
                f"{name}.removeIf(e -> e.getValue() == {rng.randint(0, 5)})",
                f"setFirstValue({name})",
            ])
        out.append(call)
    return out


def backing_change(rng, shape):
    """A write to the BACKING, which a view must see — or refuse to be walked after."""
    key = rng.choice(KEYS)
    if shape == LIST:
        # `clear()` answers nothing, and the wrapper below takes a value — a
        # block lambda gives it one.
        return rng.choice(
            [
                f'back.add("{key}")',
                f'back.remove("{key}")',
                '{ back.clear(); return "cleared"; }',
            ]
        )
    if shape == NAVSET:
        return rng.choice([f'back.add("{key}")', f'back.remove("{key}")'])
    return rng.choice([f'back.put("{key}", 9)', f'back.remove("{key}")'])


def program(rng, index, chains, steps):
    lines = []
    for at in range(chains):
        decl, shape = rng.choice(BACKINGS)
        # ONE block per chain, and one `try` around the whole of it: a view
        # declared inside a narrower scope would not be visible to the next
        # one, and a setup that throws (a sub-range outside its view) ends that
        # chain, which is itself an answer worth printing.
        lines.append("        {")
        lines.append(f"            {decl}")
        lines.append("            try {")
        view = "back"
        current = shape
        for level in range(rng.randint(1, 3)):
            built: list[str] = []
            nxt = views_for(rng, current, f"v{level}", built)
            for line in built:
                lines.append("                " + line.replace("prev", view))
            for call in operations(rng, nxt, f"v{level}", steps):
                label = f"{at}.{level}"
                lines.append(f'                step("{label}", () -> {call}, back);')
                if rng.random() < 0.3:
                    lines.append(
                        f'                step("{label}*", () -> {backing_change(rng, shape)}, back);'
                    )
            view = f"v{level}"
            current = nxt
            # `values()` and `entrySet()` are the end of a chain: neither
            # narrows further.
            if current in (VALUES, ENTRIES):
                break
        lines.append("            } catch (RuntimeException e) {")
        lines.append(
            f'                System.out.println("{at} setup ! " + e.getClass().getName()'
            ' + ": " + e.getMessage());'
        )
        lines.append("            }")
        lines.append("        }")
    return TEMPLATE.format(index=index, body="\n".join(lines))


TEMPLATE = """\
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.Iterator;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.NavigableMap;
import java.util.NavigableSet;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeSet;

public class ViewFuzz{index} {{
    interface Body {{ Object get(); }}

    static void step(String label, Body body, Object backing) {{
        Object answer;
        try {{
            answer = body.get();
        }} catch (Throwable e) {{
            answer = e.getClass().getName() + ": " + e.getMessage();
        }}
        String after;
        try {{
            after = String.valueOf(backing);
        }} catch (Throwable e) {{
            after = e.getClass().getName();
        }}
        System.out.println(label + " -> " + answer + " | " + after);
    }}

    static String walk(List<String> items) {{
        StringBuilder out = new StringBuilder();
        for (Iterator<String> it = items.iterator(); it.hasNext(); ) {{ out.append(it.next()); }}
        return out.toString();
    }}

    static String walk(NavigableSet<String> items) {{
        StringBuilder out = new StringBuilder();
        for (String v : items) {{ out.append(v); }}
        return out.toString();
    }}

    static String walkAny(Collection<?> items) {{
        StringBuilder out = new StringBuilder();
        for (Object v : items) {{ out.append(v).append(','); }}
        return out.toString();
    }}

    static String sortOf(List<String> items) {{
        items.sort(null);
        return items.toString();
    }}

    static String setFirstValue(Set<Map.Entry<String, Integer>> entries) {{
        for (Map.Entry<String, Integer> e : entries) {{ e.setValue(7); return e.toString(); }}
        return "empty";
    }}

    public static void main(String[] args) {{
{body}
    }}
}}
"""


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--seed", type=int, default=20260924)
    parser.add_argument("--programs", type=int, default=4)
    parser.add_argument("--chains", type=int, default=8, help="view chains per program")
    parser.add_argument("--steps", type=int, default=4, help="calls per view")
    parser.add_argument("--out", default="/tmp/viewfz/cases")
    args = parser.parse_args()

    rng = random.Random(args.seed)
    os.makedirs(args.out, exist_ok=True)
    for index in range(args.programs):
        source = program(rng, index, args.chains, args.steps)
        open(os.path.join(args.out, f"ViewFuzz{index}.java"), "w", encoding="utf-8").write(source)
    print(f"{args.programs} programs in {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
