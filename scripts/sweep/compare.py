#!/usr/bin/env python3
"""Diff the two halves of the grading sweep: the verdicts a real JVM gave each
test (`reference.py`) against the verdicts caturra gave (`valsweep`). A level
where the two disagree is a level where caturra would grade a student
differently from Code.org.

**Two traps, both of which made me report bugs that were not there:**

  - **Some validators are seeded with `Math.random()`, so the REFERENCE ITSELF
    flips PASS/FAIL between runs.** Re-run the reference before believing a
    flip. Every divergence left in the corpus today is one of these.
  - **A divergence can be the HARNESS, not the engine.** 126 image levels once
    appeared to flip when all that had changed was which assets the caturra half
    staged. Hold the harness constant — run both halves from the same tree, and
    if you have changed the harness, re-run the unchanged engine through it too.

usage: compare.py <reference.json> <caturra.tsv>
"""
import argparse
import collections
import json


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
    args = ap.parse_args()

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


if __name__ == "__main__":
    main()
