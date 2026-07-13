#!/usr/bin/env python3
"""Fetch the JUnit 5 jars the validation differential needs, into vendor/junit/.

`crates/caturra-vm/tests/differential_validation.rs` compiles each corpus
validator against REAL JUnit and runs it through Code.org's own
`ValidationRunner` on a real JVM, then compares which tests passed against what
caturra's `stdlib/junit.java` model says. Those jars are the reference; they are
never loaded by caturra (its VM cannot even parse them — a lambda's
`CONSTANT_InvokeDynamic` is not in its constant pool), and they never enter
anything caturra ships. `javac` and `java` do all the work.

The versions are NOT ours to choose. They are the ones the vendored
`javabuilder/` checkout declares — `lib/build.gradle` runs student validators
through `junit-platform-launcher:1.8.1` with `junit-jupiter:5.6.0` — because a
reference that runs a different JUnit than Code.org does is not the reference.
Gradle resolves the platform to the highest requested version, so the launcher
pulls platform 1.8.1 up under jupiter 5.6.0; this mirrors that.

Every jar is pinned by exact coordinate and verified against a SHA-256 recorded
below, so a later fetch cannot silently drift to different bytes. Maven Central
publishes a `.sha1` next to each artifact; that is checked too, which catches a
corrupted download but — being served by the same host — is not by itself a
guarantee of provenance. The pinned SHA-256 is what makes the fetch
reproducible.

Usage:  python3 scripts/fetch-test-jars.py [--force]
        (or `pnpm testjars:fetch`)
"""
import argparse
import hashlib
import os
import sys
import urllib.request

BASE = "https://repo1.maven.org/maven2"
OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "vendor", "junit")

# (group, artifact, version, sha256). The graph javabuilder's lib/ resolves:
# jupiter 5.6.0 for the API and engine the validators are written against, and
# the JUnit Platform at 1.8.1, which its launcher requires.
JARS = [
    ("org.junit.jupiter", "junit-jupiter-api", "5.6.0",
     "128a9828798f978fadfcda255ba365f908e58f6c37275c9e5f671cbd660a9a33"),
    ("org.junit.jupiter", "junit-jupiter-engine", "5.6.0",
     "670c95d2e60099ce747e2ffefd716dbed5afedd0b995949a3592d7c88e796d2d"),
    ("org.junit.platform", "junit-platform-commons", "1.8.1",
     "fa4fa68c8bd54dd0cb49c3fcbe9b2e42f4da6bedbe7e7ccf2a05f1a1e609b593"),
    ("org.junit.platform", "junit-platform-engine", "1.8.1",
     "702868ed7e86b9b4672ede0f1e185e905baca9afab57746a7c650be3c7bca047"),
    ("org.junit.platform", "junit-platform-launcher", "1.8.1",
     "83a9ed68adcb76e60316a4d682fc48507865df2f0ab35f82695cc9995410e05e"),
    ("org.opentest4j", "opentest4j", "1.2.0",
     "58812de60898d976fb81ef3b62da05c6604c18fd4a249f5044282479fc286af2"),
    ("org.apiguardian", "apiguardian-api", "1.1.0",
     "a9aae9ff8ae3e17a2a18f79175e82b16267c246fbbd3ca9dfbbb290b08dcfdd4"),
]


def coordinates(group, artifact, version):
    path = group.replace(".", "/")
    name = f"{artifact}-{version}.jar"
    return f"{BASE}/{path}/{artifact}/{version}/{name}", name


def fetch(url, timeout=120):
    with urllib.request.urlopen(url, timeout=timeout) as response:
        return response.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--force", action="store_true", help="re-download even if present")
    parser.add_argument(
        "--print-hashes",
        action="store_true",
        help="print the SHA-256 of each fetched jar, to pin in this file",
    )
    args = parser.parse_args()

    os.makedirs(OUT_DIR, exist_ok=True)
    digests = {}
    failed = []

    for group, artifact, version, pinned in JARS:
        url, name = coordinates(group, artifact, version)
        dest = os.path.join(OUT_DIR, name)
        if os.path.exists(dest) and not args.force:
            digests[name] = hashlib.sha256(open(dest, "rb").read()).hexdigest()
            continue
        try:
            jar = fetch(url)
            # Maven publishes a .sha1 beside each artifact: catches a corrupt
            # or truncated download.
            expected_sha1 = fetch(url + ".sha1").decode().split()[0].strip()
            actual_sha1 = hashlib.sha1(jar).hexdigest()
            if actual_sha1 != expected_sha1:
                failed.append(f"{name}: sha1 {actual_sha1} != published {expected_sha1}")
                continue
            digest = hashlib.sha256(jar).hexdigest()
            if pinned and not args.print_hashes and digest != pinned:
                failed.append(
                    f"{name}: sha256 {digest} does not match the pinned "
                    f"{pinned} — the artifact changed; do not use it"
                )
                continue
            with open(dest, "wb") as out:
                out.write(jar)
            digests[name] = digest
        except Exception as error:  # noqa: BLE001 - report every failure, not the first
            failed.append(f"{name}: {error}")

    if args.print_hashes:
        for name, digest in sorted(digests.items()):
            print(f'    "{digest}",  # {name}')

    total = sum(os.path.getsize(os.path.join(OUT_DIR, f)) for f in os.listdir(OUT_DIR))
    print(
        f"{len(digests)}/{len(JARS)} jars in {os.path.normpath(OUT_DIR)} "
        f"({total / 1e6:.1f} MB)"
    )
    if failed:
        print(f"{len(failed)} failed:", file=sys.stderr)
        for item in failed:
            print(f"  {item}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
