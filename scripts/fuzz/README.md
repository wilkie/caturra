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

**`collections.py` — a random sequence of operations.** A collection's ORDER
and its return VALUES are where a reimplementation drifts: which element
`remove` answers, what `put` gives back, where an insertion lands, what
`toString` shows. Each program builds one collection and applies random calls,
printing the answer of EVERY call and the whole collection after it — so a
divergence lands on the operation that caused it — with each call wrapped, so
the operations that THROW are compared too. It also walks hash collections
large enough to resize several times, which is the most detailed claim caturra
makes about the library. Nine thousand lines over ten seeds, no divergence.

**`regex.py` — the regex engine against a JDK's.**
`crates/caturra-vm/src/regex.rs` is a hand-written backtracking engine, and its
input space — quantifiers, classes, groups, alternation, anchors, boundaries,
backreferences — is far larger than any battery. Patterns come from a grammar,
inputs are drawn from the pattern's own characters so a fair share of them
match, and every observable answer is compared: `matches`, each `find`'s
span and groups, `split`, `replaceAll`, and the exception a bad pattern throws.

Its patterns now include atomic groups (`(?>X)`) and the two possessive
quantifiers that were missing (`?+`, `{m,n}+`). Everything it has ever found
has been one question: how much CAPTURE STATE survives a failed attempt.
Pulling on that replaced this engine's whole-snapshot save-and-restore with the
one restore a JDK actually does — a group's own tail, when its continuation
fails — and closed six shapes at once: an empty iteration ending a repetition,
a fixed-width body re-writing its group on the way out, an optional
zero-length repetition leaving its group unset, `?` not being a counted
closure, a failed branch's capture staying readable, and a possessive
repetition's last, failed attempt writing through. The spec section "The
atomic group, and what a group is left holding" writes the rules down. Forty
seeds and roughly 26000 probes now agree.

**`time.py` — `java.time` at scale.** The calendar is arithmetic caturra WROTE
— the proleptic Gregorian rules, the month-end clamping, the epoch-day
conversion both ways, the ISO text, the pattern engine — so it is exactly the
kind of code that is right on the cases someone thought of and wrong two
centuries out. Three generated programs (dates, times, formats), every answer
printed, compared byte for byte. It found five divergences the hand-written
batteries missed on its first run: `ChronoUnit.DAYS.between` overflowing a long
two millennia out, `HOURS.between` on two DATES (which `java.time` refuses
rather than treating as midnight), the text of a negative `Duration`
(`PT-11H-59M-59.999999999S`, which signs every part), the year of the ERA (`y`
is never negative — year -1 is the year 2), and the narrow text forms.

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

## Speed

`run.py` builds the engine ONCE and calls the release binary. It used to invoke
`cargo run --example compatrun` per case, which pays cargo's freshness check and
runs a DEBUG build: about two seconds a case, which is most of a long run and
the reason the big ones were never run. 200 generated programs now take about
four minutes, and the 299-case position sweep about six.

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

## `panics.py` — the engine must not crash

The other tools here compare what caturra SAYS with what a JDK says. A crash
says nothing, so none of them was watching for one: `Stream.generate(null)`
ended in `unreachable!("guarded by caller")`, and a panic prints no
diagnostics — which reads exactly like a clean compile, and made the coverage
measurement score the whole probe as supported.

`panics.py` asks only that the engine exit cleanly. Over the API surface
`scripts/coverage/` walks, it calls every modelled method with each of ten
argument SHAPES — a null, a one-parameter lambda, a two-parameter one, a
supplier, a method reference, a constructor reference, a number, a string, an
object, an array — filling every parameter of every overload with the same one.
Whether the call is legal is beside the point: an illegal one must be refused,
not crash. A failing batch is bisected line by line so the report names the one
call that did it.

    scripts/fuzz/panics.py [--verbose]

The first run found 15 crashes at one site: a lambda with FEWER parameters than
its interface takes (`stream.max(x -> x)`, where a comparator takes two)
indexed past the end of the lambda's own parameter list.

## `syntax.py` — break a program in one place

The hand-written syntax sweep behind "Where a parse error points" was eighteen
programs. This asks the same question of hundreds: take a program that
compiles, mutate ONE token — delete it, type it twice, transpose it with its
neighbour, or replace it with a symbol next to it in the grammar — and compare
what the two engines report.

What is compared is the POSITION of the first error and the NUMBER of errors,
not the wording: caturra's parser is deliberately more explicit than javac's,
which `specs/LANGUAGE.md` writes down. Position is what an editor underlines;
count is what a student reads as "how much did I break".

    scripts/fuzz/syntax.py [cases-dir] [--count N] [--seed N] [--verbose]

A mutation that still compiles is skipped. One that javac ACCEPTS and caturra
refuses is reported as a refusal, and one that javac REFUSES and caturra
compiles as an acceptance — the dangerous direction, and the exit code is
non-zero for either.

At 200 mutations: 172 of 193 put the first error in the same place, 0 in either
dangerous direction. It found one accepts-invalid on its first run — a `;`
after a `return` is an unreachable STATEMENT, and caturra dropped empty
statements at parse time because they do nothing at run time.

## `format.py` — a specifier, wrong in several ways at once

`String.format` has a grammar of its own — flags, width, precision, and a
conversion — and most combinations are illegal. A program that gets one wrong
does not see "your format string is bad": it sees a specific exception, and
WHICH one is whatever `java.util.Formatter` reaches first.

This builds random templates out of random specifiers, hands each a random
argument (`int`, `double`, `String`, `null`, a `Character`, a `Date`, a list),
and prints either the formatted text or the exception's class and message.
Every probe is wrapped in a `try`/`catch`, so one failure does not truncate
the run.

    scripts/fuzz/format.py [--count N] [--seed N] [--programs N] [--out DIR]

The first run diverged on about 120 of 1900 probes, nearly all of them
ordering: caturra validated a specifier in the order the fields are WRITTEN,
and a JDK validates in an order that differs per conversion family — an
integer's precision is illegal before its flags are, but a general
conversion's `0` is a flag mismatch rather than a missing width. It also
found that two checks are not parse-time at all: `%#s` is legal until an
argument arrives (a `Formattable` accepts it), so a MISSING argument is
reported first; and `%(o` against a `double` is a conversion mismatch, because
the argument's type is judged before the flags that only make sense while
printing an integer.

Arguments whose text carries an identity hash (an array, an anonymous
`Object`) are deliberately not in the pool: `%h` of one differs between two
runs of the same JDK, so it compares nothing.
