# External corpora

Programs other people wrote, run through a JDK and through caturra.

The Code.org corpus in `artifacts/` is what caturra was built against, and
every sweep over it now comes back clean, so it no longer finds anything. These
corpora are other people's code in other styles, and they do.

The code is not ours to redistribute, so it is fetched into a cache OUTSIDE the
repository (`~/.cache/caturra-corpus/`). Only the fetchers and the sweep are
committed.

| script       | what it does                                                                                                                                                                                                                                                                                                                                                                                                     |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `rosetta.py` | Fetches every Rosetta Code task with a Java solution (MediaWiki API, cached, 50 pages a request) and writes each code block that has a `main` as a program: `<task>__<n>/<Class>.java` + `meta.json`.                                                                                                                                                                                                            |
| `sweep.py`   | Compiles each program with `javac --release 11`. One that needs later Java or a non-JDK library is **out of scope**. The rest run on the JDK twice; a program whose output differs between runs is **nondeterministic** and set aside. Each remaining program is run on caturra and lands in `same` / `refused` / `runtime-refused` / `differs` / `slow` / `crashed`, and refusals are grouped by message shape. |

```sh
python3 scripts/corpus/rosetta.py            # ~1650 programs, ~230 MB of cached pages
cargo build --release --examples
python3 scripts/corpus/sweep.py --jobs 8     # ~20 minutes
python3 scripts/corpus/sweep.py --only Y_combinator --list refused
```

The first sweep (2026-09-28) found 761 of 1149 in-scope programs agreeing.
Refusals of what caturra honestly does not model (`java.awt`, `java.net`, …)
are counted too, and should be read apart from the ones that are bugs.
