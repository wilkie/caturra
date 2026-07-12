#!/usr/bin/env python3
"""Download the theater sound assets the CSA levels reference.

The corpus levels play named sounds (`playSound("birds.wav")`,
`SoundLoader.read("beat.wav")`). Code.org hosts them per level, keyed by a uuid
in each level's `starter_assets` map:

    https://studio.code.org/level_starter_assets/<level>/uuid/<uuid>.wav

Those responses carry no CORP/CORS headers, so a cross-origin isolated editor
(COEP: require-corp) cannot fetch them at runtime — they have to be served
same-origin. They are also large (~104 MB raw for the 18 names the playground's
levels use), so rather than vendoring them we download them as an install step
(like artifacts/) and transcode to mono MP3 for transport — a tenth the size,
and decodeAudioData resamples back to the AudioContext's rate, so
SoundLoader.read still hands the program 44.1 kHz samples.

Output (git-ignored):
    apps/playground/public/sounds/level/<name>.mp3
    apps/playground/public/sounds/level/manifest.json   {"birds.wav": "sounds/level/birds.mp3", ...}

The editor merges this manifest over its built-in defaults (beatbox.wav, which
is committed), and preloads only the sounds a program actually names. Without
running this script the app still works — unresolved names fall back to silence.

Usage:  python3 apps/playground/scripts/fetch-sounds.py [--force]
Requires: ffmpeg on PATH.
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request

# name -> (level, uuid). Baked in so this runs on a fresh clone without the
# git-ignored artifacts/ tree. Regenerate from artifacts/*/properties.json
# ("starter_assets") if the level set changes. A name can appear in several
# levels with different audio; we take one representative per name.
ASSETS = {
    "barking.wav": ("CSA U7L1-L1d_-f", "912a1425-6ba7-45bf-8a2a-fa539c893d04.wav"),
    "beat.wav": ("CSA U4L2-L7c_2023", "9c0c0870-b648-4e4c-ba9a-0da898661640.wav"),
    "birds.wav": ("CSA U4L1-L1_2022", "27d6bf54-fb36-411e-bb2d-3a384ca73862.wav"),
    "bubblesSound.wav": ("CSA U7L1-L1b_-f", "e992028e-4d2d-4dd5-8a01-0724f8c1a8c7.wav"),
    "coinSound.wav": ("CSA U7L1-L1a_-f", "c2afda70-3dad-48e8-880a-937719fcb035.wav"),
    "forestsounds.wav": ("CSA U4L1-L5b_2022", "7775d3a3-80a1-44bc-b674-78acf62d6166.wav"),
    "gamemusic.wav": ("CSA U4L1-L5a_2022", "8290df6a-ea7b-4ccd-a957-24b4a3e66056.wav"),
    "garySound.wav": ("CSA U7L1-L1b_-f", "bc77681b-bdeb-4f81-9d5f-1609b56be59d.wav"),
    "guitar.wav": ("CSA U4L1-L2_2022", "0cc8659e-d9f1-4b71-aea6-52da8ae44b29.wav"),
    "happymusic.wav": ("CSA U4L1-L5a_2022", "9981c99a-4fcb-4c5f-98a5-632bd5f5121c.wav"),
    "mrKrabsSound.wav": ("CSA U7L1-L1b_-f", "21256433-4cf9-402f-885a-97305f8a6c4e.wav"),
    "patrickSound.wav": ("CSA U7L1-L1b_-f", "2b642fb8-8d16-401f-9e3d-ec985770ae5c.wav"),
    "purpleSnailSound.wav": ("CSA U7L1-L1b_-f", "df19cfd8-82e0-450f-909f-e8cfa2b5ff50.wav"),
    "retrobeat.wav": ("CSA U4L2-L7d_2023", "45d76103-a700-4660-9187-5c3c3076c2e4.wav"),
    "sandySound.wav": ("CSA U7L1-L1b_-f", "ab1bc558-f50e-4d60-af29-f04d9e514fdb.wav"),
    "spongebobSound.wav": ("CSA U7L1-L1b_-f", "e05fbe71-19f7-44d3-b88c-5c9696494168.wav"),
    "squidwardSound.wav": ("CSA U7L1-L1b_-f", "a3749aa9-651b-4fdb-a175-5f542510832a.wav"),
    "welcome.wav": ("CSA U4L2-L7c_2023", "2f061cb7-60c8-47f6-9909-46c665ba8989.wav"),
}

BASE = "https://studio.code.org/level_starter_assets"
HERE = os.path.dirname(os.path.abspath(__file__))
OUT_DIR = os.path.join(HERE, "..", "public", "sounds", "level")
# Served path, relative to the app's base URL.
SERVED_PREFIX = "sounds/level"


def asset_url(level: str, uuid: str) -> str:
    return f"{BASE}/{urllib.parse.quote(level)}/uuid/{uuid}"


def transcode(src: str, dest: str) -> None:
    """Mono 44.1 kHz MP3: universally decodable by decodeAudioData (unlike Ogg
    in Safari), and ~10x smaller than the source WAV."""
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
         "-i", src, "-ac", "1", "-ar", "44100", "-c:a", "libmp3lame", "-b:a", "96k", dest],
        check=True,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--force", action="store_true", help="re-download even if present")
    args = parser.parse_args()

    if not shutil.which("ffmpeg"):
        print("error: ffmpeg not found on PATH", file=sys.stderr)
        return 1

    os.makedirs(OUT_DIR, exist_ok=True)
    manifest = {}
    downloaded = 0
    total_bytes = 0

    for name, (level, uuid) in sorted(ASSETS.items()):
        stem = os.path.splitext(name)[0]
        dest = os.path.join(OUT_DIR, f"{stem}.mp3")
        manifest[name] = f"{SERVED_PREFIX}/{stem}.mp3"

        if os.path.exists(dest) and not args.force:
            total_bytes += os.path.getsize(dest)
            continue

        url = asset_url(level, uuid)
        print(f"  {name} ...", end="", flush=True)
        try:
            with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
                raw = tmp.name
            with urllib.request.urlopen(url, timeout=120) as response:
                with open(raw, "wb") as out:
                    shutil.copyfileobj(response, out)
            transcode(raw, dest)
            os.unlink(raw)
        except Exception as error:  # noqa: BLE001 - report and keep going
            print(f" FAILED ({error})")
            manifest.pop(name, None)
            continue
        size = os.path.getsize(dest)
        total_bytes += size
        downloaded += 1
        print(f" {size / 1e6:.1f} MB")

    with open(os.path.join(OUT_DIR, "manifest.json"), "w") as handle:
        json.dump(manifest, handle, indent=2, sort_keys=True)

    print(
        f"\n{len(manifest)} sounds available "
        f"({downloaded} newly downloaded, {total_bytes / 1e6:.1f} MB on disk) in {OUT_DIR}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
