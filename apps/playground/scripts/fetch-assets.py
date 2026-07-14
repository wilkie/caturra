#!/usr/bin/env python3
"""Download the theater starter assets the CSA levels reference.

The corpus levels play named sounds and draw named images:

    scene.playSound("birds.wav");        SoundLoader.read("beat.wav");
    scene.drawImage("house.png", ...);   new Image("motivation.jpg");

Code.org hosts them per level, keyed by a uuid in each level's `starter_assets`
map:

    https://studio.code.org/level_starter_assets/<level>/uuid/<uuid>

Those responses carry no CORP/CORS headers, so a cross-origin isolated editor
(COEP: require-corp) cannot fetch them at runtime — they have to be served
same-origin. They are also far too large to vendor (~177 MB of WAV, ~117 MB of
images), so we download them as an install step (like artifacts/) and transcode
for transport:

  * sounds -> mono 44.1 kHz MP3 (decodeAudioData handles MP3 everywhere, unlike
    Ogg in Safari, and resamples to the AudioContext rate, so SoundLoader.read
    still hands the program 44.1 kHz samples)
  * images -> downscaled to fit the 400x400 stage, keeping PNG (alpha) as PNG

Output (git-ignored):
    apps/playground/public/level-assets/<file>
    apps/playground/public/level-assets/manifest.json   {"birds.wav": "level-assets/birds.mp3", ...}

The editor merges this manifest over its built-in defaults (beatbox.wav, which is
committed) and loads only the assets a program actually names. Without running
this script the app still works — unresolved names stay silent / draw a
placeholder.

The name -> (level, uuid) map is `level-assets.json`, generated from every
level's `starter_assets` (the authoritative list: a level often provides an asset
the student is expected to name themselves, so it cannot be derived by scanning
starter code). When the git-ignored artifacts/ tree is present it is read
directly, so the list cannot go stale; `--regen` rewrites the JSON from it.

Usage:  python3 apps/playground/scripts/fetch-assets.py [--force] [--regen]
Requires: ffmpeg on PATH, and Pillow for the handful of palette PNGs ffmpeg's
decoder rejects (see transcode_image).
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
import urllib.parse
import urllib.request

BASE = "https://studio.code.org/level_starter_assets"
HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
ART = os.environ.get("ARTIFACTS_DIR", os.path.join(REPO, "artifacts"))
MAP_FILE = os.path.join(HERE, "level-assets.json")
OUT_DIR = os.path.join(HERE, "..", "public", "level-assets")
SERVED_PREFIX = "level-assets"  # relative to the app's base URL

SOUND_EXT = {".wav"}
IMAGE_EXT = {".jpg", ".jpeg", ".png"}
STAGE = 400  # the theater canvas is 400x400; nothing needs to be larger


def from_artifacts() -> dict:
    """Every level's `starter_assets`, read from artifacts/ when present."""
    found: dict = {}
    for path in sorted(glob.glob(os.path.join(ART, "*", "properties.json"))):
        try:
            props = json.load(open(path))
        except Exception:  # noqa: BLE001 - a malformed level shouldn't stop the fetch
            continue
        level = os.path.basename(os.path.dirname(path))
        for name, uuid in (props.get("starter_assets") or {}).items():
            ext = os.path.splitext(str(name))[1].lower()
            if ext in SOUND_EXT | IMAGE_EXT and name not in found:
                found[name] = [level, uuid]
    return found


def load_map(regen: bool) -> dict:
    """artifacts/ is authoritative when available; otherwise the committed JSON.
    A name can appear in several levels with different content — one wins."""
    discovered = from_artifacts()
    if regen:
        if not discovered:
            print(f"error: --regen needs artifacts/ (looked in {ART})", file=sys.stderr)
            sys.exit(1)
        with open(MAP_FILE, "w") as handle:
            json.dump(dict(sorted(discovered.items())), handle, indent=2, sort_keys=True)
            handle.write("\n")
        prettify([MAP_FILE])
        print(f"regenerated {MAP_FILE} ({len(discovered)} assets)")
    if discovered:
        return discovered
    with open(MAP_FILE) as handle:
        return json.load(handle)


def safe_name(name: str) -> str:
    """A URL/file-safe basename (asset names contain spaces, e.g. 'Screen Shot ....png')."""
    stem, ext = os.path.splitext(name)
    return re.sub(r"[^A-Za-z0-9_.-]", "_", stem) + ext.lower()


def transcode_sound(src: str, dest: str) -> None:
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", src,
         "-ac", "1", "-ar", "44100", "-c:a", "libmp3lame", "-b:a", "96k", dest],
        check=True,
    )


def scale_image(src: str, dest: str) -> None:
    # Downscale to fit the stage, never upscale; keep PNG so alpha survives.
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", src,
         "-vf", f"scale='min({STAGE},iw)':-1", dest],
        check=True,
    )


def transcode_image(src: str, dest: str) -> None:
    try:
        scale_image(src, dest)
        return
    except subprocess.CalledProcessError:
        pass
    # ffmpeg's PNG decoder rejects a few of Code.org's palette PNGs (youtube,
    # egypt, russia, earth) that every other decoder reads happily. They used
    # to be dropped from the manifest, which the engine then papered over with
    # a blank image — so three lessons quietly drew an empty box. Re-encode to
    # RGBA with Pillow and hand ffmpeg something it will read.
    try:
        from PIL import Image  # noqa: PLC0415 - only needed on this rare path
    except ImportError as error:
        raise RuntimeError(
            f"ffmpeg cannot decode {os.path.basename(src)}; install Pillow "
            "(pip install pillow) so it can be normalized first"
        ) from error
    normalized = src + ".rgba.png"
    try:
        with Image.open(src) as image:
            image.convert("RGBA").save(normalized)
        scale_image(normalized, dest)
    finally:
        if os.path.exists(normalized):
            os.unlink(normalized)


def prettify(paths):
    """Format the emitted JSON with the repo's own prettier.

    `level-assets.json` is committed, so it is ordinary lint-checked source and
    `pnpm lint` has an opinion about it. Running prettier here rather than
    hand-matching its style (which wraps an entry only when the line would run
    past 100 columns) keeps `--regen` a no-op instead of a phantom diff — the same
    bargain generate-csa-levels.py makes, and for the same reason.
    """
    subprocess.run(
        ["pnpm", "exec", "prettier", "--write", "--log-level", "warn", *paths],
        cwd=REPO,
        check=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--force", action="store_true", help="re-download even if present")
    parser.add_argument("--regen", action="store_true", help="rewrite level-assets.json from artifacts/")
    args = parser.parse_args()

    if not shutil.which("ffmpeg"):
        print("error: ffmpeg not found on PATH", file=sys.stderr)
        return 1

    assets = load_map(args.regen)
    os.makedirs(OUT_DIR, exist_ok=True)
    manifest, failed = {}, []
    downloaded = total = 0

    for name, (level, uuid) in sorted(assets.items()):
        ext = os.path.splitext(name)[1].lower()
        sound = ext in SOUND_EXT
        out_name = safe_name(os.path.splitext(name)[0] + (".mp3" if sound else ext))
        dest = os.path.join(OUT_DIR, out_name)
        manifest[name] = f"{SERVED_PREFIX}/{out_name}"

        if os.path.exists(dest) and not args.force:
            total += os.path.getsize(dest)
            continue

        raw = None
        try:
            with tempfile.NamedTemporaryFile(suffix=ext, delete=False) as tmp:
                raw = tmp.name
            with urllib.request.urlopen(asset_url(level, uuid), timeout=180) as response:
                with open(raw, "wb") as out:
                    shutil.copyfileobj(response, out)
            (transcode_sound if sound else transcode_image)(raw, dest)
        except Exception as error:  # noqa: BLE001 - report and keep going
            failed.append(f"{name} ({error})")
            manifest.pop(name, None)
            continue
        finally:
            if raw and os.path.exists(raw):
                os.unlink(raw)

        total += os.path.getsize(dest)
        downloaded += 1
        if downloaded % 25 == 0:
            print(f"  ... {downloaded} downloaded")

    with open(os.path.join(OUT_DIR, "manifest.json"), "w") as handle:
        json.dump(manifest, handle, indent=1, sort_keys=True)

    sounds = sum(1 for n in manifest if os.path.splitext(n)[1].lower() in SOUND_EXT)
    print(
        f"\n{len(manifest)} assets available ({sounds} sounds, {len(manifest) - sounds} images; "
        f"{downloaded} newly downloaded, {total / 1e6:.1f} MB on disk) in {OUT_DIR}"
    )
    if failed:
        print(f"{len(failed)} failed:", file=sys.stderr)
        for item in failed[:10]:
            print(f"  {item}", file=sys.stderr)
    return 0


def asset_url(level: str, uuid: str) -> str:
    return f"{BASE}/{urllib.parse.quote(level)}/uuid/{uuid}"


if __name__ == "__main__":
    sys.exit(main())
