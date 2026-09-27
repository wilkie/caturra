#!/bin/bash
# Re-measure the Swing event-dispatch traces (see generate.py). Needs Xvfb and
# the installed headless openjdk-11, and fetches the matching GUI package
# (openjdk-11-jre: libawt_xawt.so) with `apt download` — no root needed. The
# JDK is copied into a work directory with that library beside the others, so
# its build, and so every line number, is the installed JDK's own.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
work="${1:-$(mktemp -d)}"
installed="$(dirname "$(dirname "$(readlink -f "$(command -v javac)")")")"
mkdir -p "$work/deb"
(cd "$work/deb" && apt download openjdk-11-jre >/dev/null && dpkg-deb -x openjdk-11-jre_*.deb x)
cp -rL "$installed" "$work/jdk" 2>/dev/null || true
cp "$work"/deb/x/usr/lib/jvm/*/lib/*.so "$work/jdk/lib/" 2>/dev/null || true
for rig in Rig1 Rig2 Rig3; do
  "$work/jdk/bin/javac" -d "$work" "$here/$rig.java"
  lower="$(echo "$rig" | tr 'R' 'r')"
  (cd "$work" && timeout 180 xvfb-run -a "$work/jdk/bin/java" "$rig" > "$here/$lower.txt" 2>&1 || true)
done
python3 "$here/generate.py"
