# The fuzz toolkit

Generated programs, run on a real JDK and on caturra, compared byte for byte.
Everything here writes `.java` files into a directory; `run.py` compiles and
runs each one on both engines and prints what diverged.

```sh
scripts/fuzz/programs.py 300 20260827 --out /tmp/fz/cases   # typed random programs
scripts/fuzz/run.py /tmp/fz/cases

scripts/fuzz/positions.py --out /tmp/pos/cases              # one expression, twelve positions
scripts/fuzz/run.py /tmp/pos/cases
```

Needs a JDK 11 on `PATH` (`javac`/`java`) and this repo, which `run.py` finds
from its own location.

## What each one is for

**`programs.py` — typed random programs.** It tracks the type of every variable
it declares and only builds an expression where its type fits, so javac's
rejection rate is zero and every case is a real comparison rather than a syntax
check. The statement mix is deliberately the shapes a student program is made
of, plus the ones that have historically broken: a generic class, a map walked
by its entries, method references, nested lambdas, a `subList` handed to an
algorithm. A SEED is the whole state, so a divergence reproduces exactly.

**`positions.py` — the mirror sweep.** One expression written in twelve
syntactic positions, each printing the same thing. A position that differs is a
position the compiler types differently from the others, which is the recurring
defect in this codebase: one fact read by two paths, updated in one. It is how
the anonymous-class scope bug was found — a stream pipeline inside an anonymous
class body was refused while the identical pipeline one line outside compiled.

## Two things the runner does that a hand-rolled loop forgets

**Compile each case ON ITS OWN.** `javac a.java b.java` writes no class files at
all when one of them fails, so a single bad program makes every other program
look like it diverged. The first version of this reported 62 divergences that
were one syntax error.

**Compare the FAILURE, not just stdout.** A program that throws is compared by
its exception class and message — the JDK writes `Exception in thread "main" X`
to stderr, caturra answers JSON, and both are reduced to one line. Without that
every deliberate `NoSuchElementException` probe reads as a divergence.

Identity hashes are normalized (`@1b6d3586` → `@X`): two JVMs never agree on
one, and a program that prints one is comparing nothing.

## Where the other sweeps live

The generators for the sweeps that are narrower than these — a receiver ×
operation matrix over the collections, "one behaviour written five ways", a
Scanner input matrix, one ordinary MISTAKE per program against javac's wording
— are written fresh each time from the recipe in `specs/LANGUAGE.md`, which
records what each found. They are a few dozen lines each; the two kept here are
the two worth re-running unchanged.
