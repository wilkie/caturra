#!/usr/bin/env python3
"""Compile the REFERENCE libraries: Code.org's real `org.code.media`,
`org.code.neighborhood` and `org.code.validation`, exactly as a student's
validator sees them on Code.org's own servers.

These are what caturra's models are checked against. They come from a local
checkout of Code.org's `javabuilder` (gitignored — it is not ours to vendor), so
this script compiles from wherever that checkout is — together with the two
reference runners — into a gitignored `vendor/sweep-classes/`.

**Why the shims.** The real `org.code.protocol` module declares the AWS SDK and
expects a live browser session; compiling it whole would drag half of javabuilder
(and its cloud dependencies) into a test harness. So `reference/shim/` provides
the handful of protocol classes the libraries actually touch — the session
singletons, the JSON the neighborhood serializes through — and goes FIRST on the
sourcepath, so it wins over the real module while everything else the libraries
reference is still compiled from Code.org's own source. `reference/stub/`
does the same for `org.code.theater`, which needs a renderer we do not stand up
(see [[orgcode-media-parity]]: theater gets no harness).

usage: build-reference.py [--javabuilder PATH]
"""
import argparse
import glob
import os
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
HERE = os.path.dirname(os.path.abspath(__file__))
REF = os.path.join(HERE, "reference")
OUT = os.path.join(REPO, "vendor", "sweep-classes")

# The modules whose behaviour caturra models, and which therefore have to be the
# real thing. `theater` is stubbed, `protocol` is shimmed.
REAL_MODULES = ["media", "neighborhood", "validation"]


def jars():
    found = sorted(glob.glob(os.path.join(REPO, "vendor", "junit", "*.jar")))
    if not found:
        sys.exit("no JUnit jars: run `pnpm testjars:fetch` first")
    return ":".join(found)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--javabuilder",
        default=os.path.join(REPO, "javabuilder", "org-code-javabuilder"),
        help="a checkout of Code.org's javabuilder (gitignored)",
    )
    args = ap.parse_args()
    jb = args.javabuilder
    if not os.path.isdir(jb):
        sys.exit(f"no javabuilder checkout at {jb} (--javabuilder to point elsewhere)")

    sources = []
    for module in REAL_MODULES:
        root = os.path.join(jb, module, "src", "main", "java")
        if not os.path.isdir(root):
            sys.exit(f"javabuilder is missing its {module} module ({root})")
        sources += glob.glob(os.path.join(root, "**", "*.java"), recursive=True)
    sources += glob.glob(os.path.join(REF, "stub", "**", "*.java"), recursive=True)
    # The runners themselves. `OrgCodeRefRunner` reaches into the AWS-heavy `lib`
    # module for the real `OutputPrintStream` (the pipe that turns a `println`
    # into the `SYSTEM_OUT` message the trackers wait for), which is why `lib` is
    # on the sourcepath and not in the compile set: javac takes the three classes
    # it actually needs and leaves the cloud behind.
    sources += sorted(glob.glob(os.path.join(REF, "*.java")))

    # Shim first: it SHADOWS the real protocol/theater classes it names, and the
    # rest are compiled from Code.org's source on demand.
    sourcepath = ":".join(
        [
            os.path.join(REF, "shim"),
            os.path.join(REF, "stub"),
            os.path.join(jb, "protocol", "src", "main", "java"),
            os.path.join(jb, "lang", "src", "main", "java"),
            os.path.join(jb, "lib", "src", "main", "java"),
        ]
    )

    os.makedirs(OUT, exist_ok=True)
    result = subprocess.run(
        ["javac", "-nowarn", "-cp", jars(), "-sourcepath", sourcepath, "-d", OUT] + sources,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        sys.exit(result.stderr or "javac failed")
    count = sum(len(files) for _, _, files in os.walk(OUT))
    for runner in ("RefRunner.class", "OrgCodeRefRunner.class", "PackAssets.class"):
        if not os.path.isfile(os.path.join(OUT, runner)):
            sys.exit(f"javac reported success but did not produce {runner}")
    print(f"built {count} reference classes into {os.path.relpath(OUT, REPO)}")


if __name__ == "__main__":
    main()
