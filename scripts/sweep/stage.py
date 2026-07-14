#!/usr/bin/env python3
"""Stage every corpus level that has BOTH a validator and a solution, into the
form both engines will compile: the solution's sources plus the level's
`validation/` files — exactly what the playground's Test button compiles
together, since a solution is by definition the answer its validator accepts.

**Staging is most of the work, and getting it wrong invents failures that are
not there.** Two rules, both lifted from `apps/playground/scripts/
generate-csa-levels.py` because they must not drift from it:

  - `start/` is merged into `solution/`. `Main.java` and the teacher-provided
    `*Helper.java` live only in `start/`, so a solution compiled alone is
    missing them. Forgetting this once cost 723 phantom compile failures.
  - The class declaring `main()` is renamed to `Main`, which is what the
    playground does and therefore what every validator references. Forgetting
    THIS cost another 469.

usage: stage.py [--artifacts DIR] [--out DIR]
"""
import argparse
import glob
import json
import os
import re
import shutil

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def grid_caturra(maze):
    """The level's grid in caturra's `tileType,paintCount` encoding. The
    reference re-encodes the same world as the real GridFactory's JSON."""
    return (
        "\n".join(
            " ".join(f"{c.get('tileType', 1)},{c.get('value', 0)}" for c in row) for row in maze
        )
        + "\n"
    )


def method_count(text):
    return len(re.findall(r"\b(?:public|private|protected)\b[^;=\n{]*\(", text))


def merged_sources(path):
    """The files the playground's editor would hold: the level's solution, plus
    the `start/` files it does not carry."""

    def read(kind):
        return {
            os.path.basename(f): open(f, encoding="utf-8", errors="replace").read()
            for f in sorted(glob.glob(os.path.join(path, kind, "*.java")))
        }

    start, solution = read("start"), read("solution")
    if not solution:
        return {}
    merged = dict(solution)
    for name, text in start.items():
        if (
            name not in solution
            or name.endswith("Helper.java")
            or method_count(text) > method_count(solution[name])
        ):
            merged[name] = text
    return merged


def rename_main(files):
    """The playground renames whichever class declares `main()` to `Main`, which
    is why the validators reference `Main`."""
    out = {}
    for name, text in files.items():
        if "static void main" in text:
            match = re.search(r"class\s+(\w+)", text[: text.index("static void main")])
            if match:
                text = re.sub(
                    r"(\bclass\s+)" + re.escape(match.group(1)) + r"\b", r"\1Main", text, count=1
                )
            out["Main.java"] = text
        else:
            out[name] = text
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--artifacts", default=os.path.join(REPO, "artifacts"))
    ap.add_argument("--out", default=os.path.join(REPO, "vendor", "sweep-cases"))
    args = ap.parse_args()

    shutil.rmtree(args.out, ignore_errors=True)
    os.makedirs(args.out)
    staged = 0
    for path in sorted(glob.glob(os.path.join(args.artifacts, "*"))):
        validators = sorted(glob.glob(os.path.join(path, "validation", "*.java")))
        sources = rename_main(merged_sources(path))
        if not validators or not sources:
            continue
        case = os.path.join(args.out, f"{staged:04d}")
        os.makedirs(case)
        for name, text in sources.items():
            open(os.path.join(case, name), "w", encoding="utf-8").write(text)
        for src in validators:
            shutil.copy(src, os.path.join(case, os.path.basename(src)))
        # The level's data files, as the playground writes them.
        for data in glob.glob(os.path.join(path, "start", "*.txt")) + glob.glob(
            os.path.join(path, "start", "*.csv")
        ):
            shutil.copy(data, os.path.join(case, os.path.basename(data)))
        try:
            maze = json.load(open(os.path.join(path, "properties.json"))).get("serialized_maze")
            if maze:
                open(os.path.join(case, "grid.txt"), "w").write(grid_caturra(maze))
        except (OSError, ValueError):
            pass
        open(os.path.join(case, "name"), "w").write(os.path.basename(path))
        staged += 1
    print(f"staged {staged} levels with a validator and a solution into {args.out}")


if __name__ == "__main__":
    main()
