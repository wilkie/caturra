#!/usr/bin/env python3
"""The REFERENCE half of the grading sweep: run every staged level's validator
on a real JVM, and record the verdict it gives each test. `valsweep` (the
caturra half) records the same thing, and `compare.py` diffs them. That
comparison is the only thing that says whether caturra grades a student the way
Code.org does.

Two modes, because the corpus splits in two:

  `--mode junit`   — levels that touch no `org.code.*` library. Real JUnit 5 and
                     nothing else, through `RefRunner`. This is what checks
                     caturra's JUnit model and its reflection: which tests run,
                     in what order, whether they pass, and what a failure says.

  `--mode orgcode` — levels that DO use `org.code.*`. Real JUnit 5 plus the real
                     media / neighborhood / validation libraries (built by
                     `build-reference.py`), through `OrgCodeRefRunner`, which
                     stands up the `ValidationProtocol` session the way
                     javabuilder's `CodeExecutionManager` does. Without that
                     session the real `NeighborhoodTestRunner.run()` returns
                     nothing at all.

The org.code mode also KEEPS each level's images as the real library decoded
them (`__packed_*`, see `PackAssets.java`), so that when caturra runs the same
level it starts from byte-identical pixels and an image decode is never what the
sweep is comparing.

usage: reference.py --mode junit|orgcode [--cases DIR] [--out FILE] [--packed DIR]
"""
import argparse
import glob
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from concurrent.futures import ProcessPoolExecutor, as_completed

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CLASSES = os.path.join(REPO, "vendor", "sweep-classes")
ASSETS = os.path.join(REPO, "apps/playground/public/level-assets")

ASSET_RE = re.compile(r'"([\w\-. ]+\.(?:jpg|jpeg|png|gif))"')
SOUND_RE = re.compile(r'"([\w\-. ]+\.(?:wav|mp3))"')


def manifest():
    """The playground's map from the name a level DECLARES (`retrobeat.wav`) to
    the file we actually store (`level-assets/retrobeat.mp3` — the fetcher
    transcodes for the browser). The declared name is the only one the source
    code ever says."""
    try:
        return json.load(open(os.path.join(ASSETS, "manifest.json")))
    except (OSError, ValueError):
        return {}


def stage_sound(declared, stored, work):
    """Put `declared` in `work` as something the REAL library can decode.

    We keep sounds as mp3 (a browser wants them small), and `javax.sound` cannot
    read mp3 — so transcode back to PCM under the name the level asks for. The
    samples are lossily one round-trip from Code.org's, but BOTH engines get
    exactly these samples, and that is what the sweep compares."""
    source = os.path.join(ASSETS, os.path.basename(stored))
    if not os.path.isfile(source):
        return False
    done = subprocess.run(
        ["ffmpeg", "-y", "-loglevel", "error", "-i", source, "-ac", "1", "-ar", "44100",
         "-c:a", "pcm_s16le", os.path.join(work, declared)],
        capture_output=True,
        text=True,
        timeout=120,
    )
    return done.returncode == 0


def jars():
    return ":".join(sorted(glob.glob(os.path.join(REPO, "vendor", "junit", "*.jar"))))


def grid_json(path):
    """The level's grid in the real GridFactory's JSON. caturra's `grid.txt` is
    `tileType,paintCount` rows; the world is the same, the encoding is the
    host's."""
    rows = [line.split() for line in open(path).read().strip().split("\n") if line.strip()]
    return json.dumps(
        [
            [{"tileType": int(c.split(",")[0]), "value": int(c.split(",")[1])} for c in row]
            for row in rows
        ]
    )


def one(job):
    case, mode, packed_dir = job
    name = open(os.path.join(case, "name")).read().strip()
    sources = sorted(glob.glob(os.path.join(case, "*.java")))
    texts = {
        os.path.basename(f): open(f, encoding="utf-8", errors="replace").read() for f in sources
    }

    uses_orgcode = any("org.code" in t for t in texts.values())
    if mode == "junit" and uses_orgcode:
        return name, {"status": "ORGCODE"}
    if mode == "orgcode" and not uses_orgcode:
        return name, {"status": "NOTORGCODE"}

    tests = [n[:-5] for n, t in texts.items() if "org.junit" in t]
    if not tests:
        return name, {"status": "NOTESTS"}
    classpath = f"{jars()}:{CLASSES}"

    work = tempfile.mkdtemp(prefix="sweepref-")
    try:
        for filename, text in texts.items():
            open(os.path.join(work, filename), "w", encoding="utf-8").write(text)
        for extra in glob.glob(os.path.join(case, "*.txt")) + glob.glob(
            os.path.join(case, "*.csv")
        ):
            if mode == "orgcode" and os.path.basename(extra) == "grid.txt":
                continue
            shutil.copy(extra, work)

        java = ["java", "-Djava.awt.headless=true", f"-Dcaturra.assets={work}"]
        if mode == "orgcode":
            grid = os.path.join(case, "grid.txt")
            if os.path.isfile(grid):
                open(os.path.join(work, "grid.txt"), "w").write(grid_json(grid))
            # The level's images, decoded by the REAL library and kept.
            have = []
            for asset in sorted({a for t in texts.values() for a in ASSET_RE.findall(t)}):
                src = os.path.join(ASSETS, asset)
                if os.path.isfile(src):
                    shutil.copy(src, os.path.join(work, asset))
                    have.append(asset)
            if have:
                subprocess.run(
                    java + ["-cp", classpath, "PackAssets"] + have,
                    cwd=work,
                    capture_output=True,
                    text=True,
                    timeout=180,
                )
            # And its sounds. A level that reads samples it never got would be
            # graded on silence by both engines and agree about nothing.
            names = manifest()
            heard = []
            for sound in sorted({s for t in texts.values() for s in SOUND_RE.findall(t)}):
                stored = names.get(sound, sound)
                if stage_sound(sound, stored, work):
                    heard.append(sound)
            if heard:
                subprocess.run(
                    java + ["-cp", classpath, "PackSounds"] + heard,
                    cwd=work,
                    capture_output=True,
                    text=True,
                    timeout=180,
                )

        compiled = subprocess.run(
            ["javac", "-nowarn", "-cp", classpath, "-d", "."]
            + [os.path.basename(f) for f in sources],
            cwd=work,
            capture_output=True,
            text=True,
            timeout=240,
        )
        if compiled.returncode != 0:
            first = (compiled.stderr.splitlines() or [""])[0]
            return name, {"status": "CE", "detail": first[:160]}

        if mode == "junit":
            argv = ["RefRunner"] + tests
        else:
            user = [n[:-5] for n, t in texts.items() if "org.junit" not in t]
            argv = [
                "OrgCodeRefRunner",
                "Main" if "Main.java" in texts else "-",
                ",".join(user),
            ] + tests
        ran = subprocess.run(
            java + ["-cp", f"{classpath}:."] + argv,
            cwd=work,
            capture_output=True,
            text=True,
            timeout=240,
        )
        if mode == "orgcode":
            found = glob.glob(os.path.join(work, "__packed_*"))
            if found:
                out = os.path.join(packed_dir, name)
                os.makedirs(out, exist_ok=True)
                for asset in found:
                    shutil.copy(asset, out)

        lines = [l for l in ran.stdout.splitlines() if l.startswith("__VTEST\t")]
        # THE REFERENCE MAY NOT UNDER-REPORT EITHER. A JVM that died mid-run has
        # already printed the verdicts of the tests it got through, and taking
        # those at face value compares caturra against a roster the reference
        # never finished — a level whose own validator OOMs looked exactly like a
        # level where caturra had invented an extra test. A non-zero exit is not
        # a result, whatever it managed to print first.
        if ran.returncode != 0 or not lines:
            blame = re.search(r"(?:Exception|Error)[^\n]*", ran.stderr)
            return name, {
                "status": "RUN",
                "tests": [],
                "detail": blame.group(0) if blame else ran.stderr[:150],
            }
        return name, {"status": "OK", "tests": lines}
    except subprocess.TimeoutExpired:
        return name, {"status": "TIMEOUT"}
    except OSError as err:
        return name, {"status": "ERR", "detail": str(err)}
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["junit", "orgcode"], required=True)
    ap.add_argument("--cases", default=os.path.join(REPO, "vendor", "sweep-cases"))
    ap.add_argument("--out", default=None)
    ap.add_argument("--packed", default=os.path.join(REPO, "vendor", "sweep-packed"))
    args = ap.parse_args()

    if not jars():
        sys.exit("no JUnit jars: run `pnpm testjars:fetch` first")
    if not os.path.isfile(os.path.join(CLASSES, "RefRunner.class")):
        sys.exit("no reference runners: run `scripts/sweep/build-reference.py` first")
    cases = sorted(glob.glob(os.path.join(args.cases, "*")))
    if not cases:
        sys.exit(f"no staged levels in {args.cases}: run `scripts/sweep/stage.py` first")

    out = args.out or os.path.join(REPO, "vendor", f"sweep-reference-{args.mode}.json")

    print(f"{len(cases)} staged levels", flush=True)
    results = {}
    with ProcessPoolExecutor(max_workers=8) as pool:
        futures = {pool.submit(one, (case, args.mode, args.packed)): case for case in cases}
        for done, future in enumerate(as_completed(futures), start=1):
            name, meta = future.result()
            results[name] = meta
            if done % 200 == 0:
                print(f"  {done}/{len(cases)}", flush=True)
    json.dump(results, open(out, "w"))

    kinds = {}
    for meta in results.values():
        kinds[meta["status"]] = kinds.get(meta["status"], 0) + 1
    print("reference outcomes:", kinds)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
