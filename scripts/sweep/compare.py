#!/usr/bin/env python3
"""Diff the two halves of the grading sweep: the verdicts a real JVM gave each
test (`reference.py`) against the verdicts caturra gave (`valsweep`). A level
where the two disagree is a level where caturra would grade a student
differently from Code.org.

**Two traps, both of which made me report bugs that were not there:**

  - **Some validators do not pin their own verdict down**, so a flip against
    them says nothing about the engine. Two ways, and they differ:
    unseeded `Math.random()` makes the REFERENCE ITSELF flip between runs
    (re-run it before believing anything), while two tests sharing an `@Order`
    value run in an order JUnit does not specify — each engine picks its own,
    and if those tests share state the same submission grades either way.
    This script READS the staged sources and says which divergence is which,
    rather than leaving the reader to remember the rule. **The number to watch
    is the "unexplained" count it prints at the end.**
  - **A divergence can be the HARNESS, not the engine.** 126 image levels once
    appeared to flip when all that had changed was which assets the caturra half
    staged. Hold the harness constant — run both halves from the same tree, and
    if you have changed the harness, re-run the unchanged engine through it too.

usage: compare.py <reference.json> <caturra.tsv>
"""
import argparse
import collections
import json
import pathlib
import re


# A validator that draws randomness with no seed grades the SAME submission
# differently from one run to the next, so a flip against it says nothing about
# the engine. The trap is written down in this file's docstring; reading the
# sources says WHICH levels it applies to, which is what a reader of the output
# actually needs.
UNSEEDED = re.compile(r"Math\s*\.\s*random\s*\(|new\s+Random\s*\(\s*\)")

# Two tests that share an `@Order` value run in an order JUnit does not
# specify. That is only a level defect when they also share state — a static
# counter one resets and the other does not — but then the SAME submission
# passes or fails depending on which ran first, so a flip against the reference
# says nothing about the engine either.
ORDER = re.compile(r"@Order\s*\(\s*(\d+)\s*\)")


def unseeded_levels(cases):
    """{level name: what it draws} for every staged level whose TEST sources
    use unseeded randomness."""
    found = {}
    if not cases.is_dir():
        return found
    for case in sorted(cases.iterdir()):
        name_file = case / "name"
        if not name_file.is_file():
            continue
        reasons = set()
        for source in case.glob("*Test*.java"):
            text = source.read_text(errors="replace")
            for hit in UNSEEDED.findall(text):
                reasons.add("unseeded " + " ".join(hit.split()) + ") — re-run the reference")
            orders = ORDER.findall(text)
            for value in sorted({v for v in orders if orders.count(v) > 1}):
                reasons.add(
                    f"two tests at @Order({value}) — the order between them is "
                    "unspecified, and each engine picks its own"
                )
        if reasons:
            found[name_file.read_text().strip()] = ", ".join(sorted(reasons))
    return found


def verdicts(lines):
    """`__VTEST\tPASS|FAIL\tname\tmessage` -> {name: PASS|FAIL}."""
    out = {}
    for line in lines:
        fields = line.split("\t")
        if len(fields) > 2:
            out[fields[2]] = fields[1]
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("reference")
    ap.add_argument("caturra")
    ap.add_argument(
        "--cases",
        default="vendor/sweep-cases",
        help="the staged level sources, read to say WHY a level diverges",
    )
    args = ap.parse_args()
    coin_flips = unseeded_levels(pathlib.Path(args.cases))

    reference = json.load(open(args.reference))
    caturra = {}
    for line in open(args.caturra, encoding="utf-8"):
        parts = line.rstrip("\n").split("\t", 2)
        if len(parts) < 2:
            continue
        caturra[parts[0]] = {
            "status": parts[1],
            "tests": parts[2].split("\x01") if len(parts) > 2 and parts[2] else [],
        }

    stats = collections.Counter()
    same = 0
    divergent = []
    for name, ref in reference.items():
        # Levels belonging to the other half of the corpus.
        if ref["status"] in ("ORGCODE", "NOTORGCODE"):
            continue
        if ref["status"] != "OK":
            stats["ref " + ref["status"]] += 1
            continue
        cat = caturra.get(name)
        if not cat or cat["status"] not in ("OK", "RUN"):
            stats[f"caturra {cat['status'] if cat else 'MISSING'} (ref OK)"] += 1
            continue
        stats["both ran"] += 1
        ref_v, cat_v = verdicts(ref["tests"]), verdicts(cat["tests"])
        if ref_v == cat_v:
            same += 1
        else:
            divergent.append(
                (
                    name,
                    {k: (ref_v[k], cat_v[k]) for k in set(ref_v) & set(cat_v) if ref_v[k] != cat_v[k]},
                    set(ref_v) - set(cat_v),
                    set(cat_v) - set(ref_v),
                )
            )

    print("=== levels ===")
    for kind, count in sorted(stats.items(), key=lambda kv: -kv[1]):
        print(f"  {count:5}  {kind}")
    print()
    print("=== per-test verdicts, on levels both engines ran ===")
    print(f"  IDENTICAL:  {same}")
    print(f"  DIVERGENT:  {len(divergent)}")
    print()
    kinds = collections.Counter()
    for _, flipped, only_ref, only_cat in divergent:
        if flipped:
            kinds["verdict flipped (PASS<->FAIL)"] += 1
        if only_ref:
            kinds["test missing in caturra"] += 1
        if only_cat:
            kinds["extra test in caturra"] += 1
    for kind, count in kinds.most_common():
        print(f"  {count:5}  {kind}")
    if divergent:
        print()
    for name, flipped, only_ref, only_cat in divergent:
        bits = []
        if flipped:
            bits.append(f"flipped={list(flipped.items())[:2]}")
        if only_ref:
            bits.append(f"only-in-ref={list(only_ref)[:1]}")
        if only_cat:
            bits.append(f"only-in-caturra={list(only_cat)[:1]}")
        print(f"  {name}: {'; '.join(bits)[:150]}")
        if name in coin_flips:
            print(f"      ^ the validator does not pin this down: {coin_flips[name]}")
    unexplained = [name for name, _, _, _ in divergent if name not in coin_flips]
    if divergent:
        print()
        print(
            f"  {len(divergent) - len(unexplained)} of {len(divergent)} sit on a "
            f"validator that does not pin its own verdict down; "
            f"{len(unexplained)} UNEXPLAINED"
        )


if __name__ == "__main__":
    main()
