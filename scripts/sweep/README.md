# The grading sweep

`tests/differential_validation.rs` checks caturra's grading against real JUnit 5
on a handful of hand-written validators. This is the same check at corpus scale:
**every level Code.org ships, graded by both engines, verdict by verdict.**

It is the only thing that says whether caturra passes and fails a student the way
Code.org does — and it has caught bugs the hand-written tests did not (a painter's
ending direction after a turn; reflective exceptions that were _uncatchable_, so a
validator silently died after its first test and the student was told everything
passed). It has also _missed_ bugs the hand-written tests caught, because it only
compares verdicts and not message text. **Write both.**

## Running it

Needs a JDK, the pinned JUnit jars (`pnpm testjars:fetch`), the level corpus
(`artifacts/`), and — for the `org.code` half — a checkout of Code.org's
`javabuilder`. All four are gitignored: none of them are ours to vendor.

```sh
scripts/sweep/stage.py                        # corpus  -> vendor/sweep-cases/
scripts/sweep/build-reference.py              # the REAL org.code libraries
scripts/sweep/reference.py --mode junit       # verdicts from a real JVM
scripts/sweep/reference.py --mode orgcode     #   (and the real libraries)

cargo run --release -p caturra-vm --example valsweep -- \
    vendor/sweep-cases vendor/sweep-packed > vendor/sweep-caturra.tsv

scripts/sweep/compare.py vendor/sweep-reference-junit.json   vendor/sweep-caturra.tsv
scripts/sweep/compare.py vendor/sweep-reference-orgcode.json vendor/sweep-caturra.tsv
```

## Where the corpus splits

A level either touches `org.code.*` or it does not, and the two halves need
different references:

- **`--mode junit`** — real JUnit 5 and nothing else. Checks the JUnit model and
  caturra's reflection: which tests run, in what order, whether they pass, what a
  failure _says_.
- **`--mode orgcode`** — real JUnit 5 _plus_ Code.org's real media / neighborhood
  / validation libraries. These need a live session to say anything at all:
  `OrgCodeRefRunner` stands up the `ValidationProtocol` the way javabuilder's
  `CodeExecutionManager` does, with `System.out` piped through the real
  `OutputPrintStream` so a `println` becomes the `SYSTEM_OUT` message the trackers
  wait for. Without that wiring `NeighborhoodTestRunner.run()` returns nothing.

`build-reference.py` compiles those libraries from the javabuilder checkout. The
real `org.code.protocol` declares the AWS SDK, so `reference/shim/` supplies the
few protocol classes the libraries actually touch and goes first on the sourcepath;
`reference/stub/` does the same for `org.code.theater`, which wants a renderer we
do not stand up. Everything whose _behaviour_ is being compared — media,
neighborhood, validation — is the real thing, unmodified.

## Baselines

Plain JUnit **1736 / 1743**. org.code **869 / 871**. Every remaining divergence is
a known level defect, each verified rather than assumed: validators seeded with
`Math.random()` (the reference itself flips them between runs), one level giving
two tests the same `@Order` (JUnit breaks the tie by `getDeclaredMethods()` order,
which the JVM does not specify), and one whose solution reads `> 15` where its own
test means `>= 15`.

**If a number moves, suspect the harness before the engine.** See the warnings at
the top of `compare.py` and `stage.py` — bad staging once manufactured 723 and then
469 phantom compile failures, and a change in which assets got staged once looked
exactly like a compiler fix repairing 126 levels.
