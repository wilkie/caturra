# SCOPE — what caturra implements, and where the class library comes from

- **Status:** accepted
- **Date:** 2026-07-02
- **Folder convention:** `specs/` holds project decisions and specifications as
  markdown. New decisions get their own file (or extend an existing one) and are
  linked from here when they refine this scope.

## Context

caturra is a browser-only JVM (Rust → WASM engine, TypeScript wrapper) aimed at
running the Java taught in AP Computer Science A (CSA). A JVM is only useful with
a class library (`java.lang.String`, `System.out`, `ArrayList`, ...), and we had
three options: boot the real OpenJDK classes, port an Apache-licensed
reimplementation (TeaVM classlib, GWT/J2CL JRE emulation, Apache Harmony), or
write our own.

OpenJDK's library is impractical here: its bottom layer drags in `Unsafe`,
VarHandles, `invokedynamic` string concatenation, module and charset machinery —
most of a production VM's complexity for no educational benefit, plus GPLv2+CPE
license friction against this MIT repo. The Apache-licensed reimplementations
assume their own runtimes at the bottom, so they would be ports, not drop-ins.
Meanwhile the CSA surface is genuinely small (~30–40 classes, most thin).

## Decision

**We write our own class library, scoped to the CSA standards, and expose our
implementations through the VM's bootstrap class loader.** When a program refers
to `java.lang.String` or `java.util.ArrayList`, the class loader resolves it to
caturra's implementation — user code never supplies or overrides these core classes.

Implementation is two-layered:

1. **Rust intrinsics** for classes whose semantics are entangled with the VM:
   `Object`, `String`, `StringBuilder`, `Math`, `System`, primitive wrappers,
   core exceptions, and the `PrintStream` / `InputStream` / `File` natives that
   wire into the existing `ConsoleIo` trait and `VirtualFileSystem`.
2. **Java source in this repo** (a `classlib/` tree) for classes expressible on
   top of that floor: `ArrayList`, `Scanner`, and similar. These are compiled by
   our own `javac` at engine build time and baked into the WASM — which doubles
   as a permanent integration test of the compiler.

Whether a given class starts as an intrinsic and later migrates to Java source
(or vice versa) is an implementation detail; the class loader boundary is the
contract.

## The CSA surface (initial commitment)

The AP Java Quick Reference is the floor: everything on it must work exactly as
the exam expects. Non-exhaustive summary of that floor:

| Area        | Classes / features                                                                                                                                                                                                                                                                                                                      |
| ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `java.lang` | `Object` (`equals`, `toString`), `String` (`length`, `substring`, `indexOf`, `equals`, `compareTo`, concatenation), `Math` (`abs`, `pow`, `sqrt`, `random`), `Integer` / `Double` (constants, parsing), core exceptions (`NullPointerException`, `ArithmeticException`, `IndexOutOfBoundsException` family, `IllegalArgumentException`) |
| `java.util` | `ArrayList` / `List` (`size`, `add`, `get`, `set`, `remove`)                                                                                                                                                                                                                                                                            |
| Console     | `System.out.print` / `println`                                                                                                                                                                                                                                                                                                          |
| Language    | primitives (`int`, `double`, `boolean`), operators, control flow, 1D/2D arrays, classes, constructors, static/instance members, inheritance and polymorphism, `super`, method overloading/overriding                                                                                                                                    |

On top of that floor, we commit to the pieces this project already promises:

- `java.util.Scanner` and `System.in` (console input via the host page)
- `java.io.File` plus basic readers/writers, backed by the virtual filesystem
- `char` / `StringBuilder` and other commonly-taught companions of the above
- `interface` and `abstract class` support — classroom material even where the
  current exam de-emphasizes them

## Beyond CSA

The library may grow past the exam surface where it makes content more
accessible (e.g. `HashMap`, `String.format`, more of `java.util`). Rules for
additions:

- CSA-floor behavior is never compromised to accommodate an extension.
- Each addition should serve educational content, not JDK completeness — "a
  student's textbook uses it" is the bar.
- Notable additions get recorded in `specs/` (this file or a linked one).

### What has been added, and why

Each of these went past the exam surface for the reason beside it. The list is
kept current as things land, so "is X in scope?" has an answer that is not a
grep of the source.

| Addition                                                                                                                                                                                                                                                                                                                                      | Why                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The rest of `java.util.Scanner` — `delimiter`, `radix`/`useRadix`, `reset`, `match`, the `Iterator<String>` face                                                                                                                                                                                                                              | The most-used class in the curriculum, and the lowest-scoring one a student writes (29/39). Its configuration methods work on a closed scanner where the reading ones refuse — measured, not assumed. `locale`/`useLocale` stay refusals, because `Locale` is a written constant here and the default is host state.                                                                                                                                                                           |
| `java.io.File`'s metadata — the permission bits, the times, `listRoots`, `createTempFile`                                                                                                                                                                                                                                                     | The lowest-scoring class a student actually uses (23/40). The permissions are real state AND enforced, which a JDK capture established after the first version of the code assumed otherwise; the space queries and `toURI` stay refusals, because an in-memory filesystem has no device and `java.net` is not modelled.                                                                                                                                                                       |
| The `Collections` wrappers, the empty factories, `Arrays`' parallel family, `Objects`' range checks, `Scanner.nextBigInteger`/`nextBigDecimal`                                                                                                                                                                                                | Chosen by `scripts/coverage/measure.py` rather than by what a probe stumbled into. On one thread a synchronized wrapper IS the collection and a parallel sort IS a sort, so most of this is delegation; the `Scanner` pair was a refusal that had outlived its reason.                                                                                                                                                                                                                         |
| `java.io.BufferedWriter`, and the four readers told apart                                                                                                                                                                                                                                                                                     | A `BufferedWriter` is what every file-writing exercise wraps a `FileWriter` in, and it really buffers — "my file is empty" is the lesson a forgotten `close()` teaches. The four readers shared one type, so `getClass()` named the wrong one and `readLine` was offered on all of them.                                                                                                                                                                                                       |
| `Charset`'s own five questions, `IsoEra`'s field surface, `ValueRange.of`, a UUID's time fields and `nameUUIDFromBytes`, `BigDecimal.sqrt`, `BigInteger.sqrtAndRemainder`                                                                                                                                                                     | The last of the measured list, and the point at which `measure.py --why` exits zero: every method name a JDK offers on a class caturra models now runs or says why. `aliases()` came with a repair — the alias list and the names `forName` accepts were two hand-written lists, and the second was a subset, so `Charset.forName("646")` failed for an alias a JDK answers.                                                                                                                   |
| `java.text.DecimalFormat` / `NumberFormat` — the four affixes, the multiplier, the always-shown separator, the two parsing flags, `parseObject`, `clone`, and the five inherited factories                                                                                                                                                    | The two classes a money exercise is written in. Most of it was already in the pattern the engine parses, so the getters were a field read each — but what a JDK does with them is not obvious, and the capture settled three things a guess gets wrong. `toPattern()` after a setter is the surprising one: a programmatically-set affix is stored as a LITERAL, so `setNegativePrefix("-")` comes back as `'-'`.                                                                              |
| `java.lang.reflect` — `getDeclaringClass` on all three, the eight typed `Field` accessors, `Constructor`'s parameter shape, the flag questions (`isVarArgs`, `isEnumConstant`, `isDefault`, `isBridge`, `isSynthetic`), the accessibility trio, `Modifier`'s six masks, and a member's `throws` clause                                                                    | The worst block the measurement found, and the one the GRADING path walks — a validator finds a student's method by reflection before it can call it. Three access-flag bits were never emitted, so three questions had WRONG answers rather than missing ones; a package-private field reported itself `public`. The eight typed accessors are now one widening rule read both ways instead of two hand-rolled matches that had come to disagree. The class files carry an `Exceptions` attribute now, so `getExceptionTypes` answers and a member's `toString` ends in its `throws` clause. Reflecting on a LIBRARY class's members still finds nothing: those classes have no class file here.                                             |
| The partial dates finished (`plus`/`minus`/`with`/`until`/`range`/`now`, `YearMonth.withYear`/`withMonth`, `Year.isValidMonthDay`), the primitive `Optional`s' `ifPresentOrElse`/`orElseGet`/`stream`, `Byte`/`Short.decode` and `compareUnsigned`, `Float.toHexString`, `asIterator`, the three null streams, `BitSet`/`BigInteger` as bytes | The ordinary end of what the widened measurement found — what a student writes rather than what a framework calls. Every question about moving a partial date is that question about the date it fills out to, so the calendar arithmetic exists once and what each kind supports falls out rather than being listed. Four JDK facts here are only visible in a capture, and two plain bugs turned up beside them: `Year.of(5)` printed as "0005", and a CLOSED `PrintWriter` went on writing. |
| What a throwable carries — `setStackTrace` on all 46 of them, `new StackTraceElement(...)`, `isNativeMethod`, the eleven constructors that take more than a message, and the summary statistics a program fills itself                                                                                                                        | The first thing the widened measurement found. Eleven of these classes could not be scored at all, because their constructors take something other than a message and the probe's own receiver did not compile. Where the JDK's message is built from the extra value, the getter reads it back out of the message — so an exception a program constructs and one caturra's formatter threw answer alike, from one implementation.                                                             |
| The long tail of the classes already modelled — `ArrayDeque.clone`, `getDeclaringClass` on an enum constant, `Arrays.compareUnsigned`, `Random.longs`, `ByteArrayOutputStream.writeBytes`/`writeTo`, and eleven more of `java.lang.Class`                                                                                                     | Named by `scripts/coverage/measure.py`, not chosen: they were what was left. Each was captured from a JDK first, and each does something the obvious guess does not — `compareUnsigned` answers a sign for `int[]` and a difference for `byte[]`; a bounded `Random.longs` redraws until unbiased; `ArrayDeque.clone()` answers a deque where `ArrayList.clone()` answers `Object`.                                                                                                            |
| `java.io.Writer` (the face), `java.time.format.TextStyle` / `FormatStyle` (as values), `Collector<T, A, R>`, the map-gathering `Collectors` overloads                                                                                                                                                                                         | Not additions so much as REPAIRS: each was ordinary Java that caturra turned away. A `Writer` variable holds either writer; the two styles are enums like the rest of `java.time`; a collector can be factored into a variable and can name the map it gathers into.                                                                                                                                                                                                                           |
| `java.util.Vector` / `Hashtable` / `Enumeration`                                                                                                                                                                                                                                                                                              | The collections that predate the collections framework. Every AP reading list still names them, and half the Java written before 1998 is built on them — so a student meeting one in real code should be able to run it here rather than be told it does not exist. Their observable differences from `ArrayList`/`HashMap` (capacity, bucket order, no nulls, a cursor that is not fail-fast) are exactly what makes them worth having.                                                       |
| `java.time.Year` / `YearMonth` / `MonthDay`                                                                                                                                                                                                                                                                                                   | The three partial dates: a year on its own, the month a statement covers, and a birthday. They complete the arithmetic slice of `java.time`.                                                                                                                                                                                                                                                                                                                                                   |
| `java.util.Base64`, `java.util.BitSet`, `java.util.RandomAccess`                                                                                                                                                                                                                                                                              | Base64 is how a program moves bytes through text; a BitSet is the sieve every course writes; RandomAccess is the marker an algorithm asks about.                                                                                                                                                                                                                                                                                                                                               |
| `java.util.StringTokenizer`, `java.util.UUID`, `java.io.StringReader` / `StringWriter`                                                                                                                                                                                                                                                        | Four a program reaches for without thinking. The tokenizer is still taught before `split`; a `StringWriter` behind a `PrintWriter` is how output gets tested.                                                                                                                                                                                                                                                                                                                                  |
| `java.text.DecimalFormat` / `NumberFormat`                                                                                                                                                                                                                                                                                                    | The moment a number has to appear on a screen. `String.format` covers much of it, but a pattern is what a program keeps in a field and applies over and over.                                                                                                                                                                                                                                                                                                                                  |
| `java.math.BigDecimal` (+ `RoundingMode`, `MathContext`)                                                                                                                                                                                                                                                                                      | The answer to `0.1 + 0.2`, and the type every money exercise should be written in. The scale is observable, so it teaches something a `double` cannot.                                                                                                                                                                                                                                                                                                                                         |
| `java.math.BigInteger`                                                                                                                                                                                                                                                                                                                        | Exact arithmetic past `long` — factorials, RSA-shaped exercises, and the "why did my number go negative?" lesson that overflow teaches. Its own bignum core (`crates/caturra-vm/src/bigint.rs`), for the same reason the regex engine and the float formatter are our own.                                                                                                                                                                                                                     |

### What this engine does not have, and how it says so

A real Java class caturra does not model is refused BY NAME —
"java.util.Properties is not supported by caturra (the class library covers the
AP CS A subset)" — wherever it is written, and so is a real JDK package. The
whole surface was swept against a Java 11 module image on 2026-09-04 (see
"Every class this engine does not have" in `specs/LANGUAGE.md`), so the answer
is exhaustive rather than a list of whichever names somebody had happened to
hit. A package that really does not exist keeps javac's own "does not exist",
which is what makes the honest message honest.

The same is now true one level down, of METHODS. Every method name a real JDK
11 offers on a class caturra models either runs or explains itself — "`X.y`
exists in Java, but …" — and never reads as "cannot find symbol", which is what
a typo looks like. That is checked rather than claimed:
`scripts/coverage/measure.py --why` prints the complaint for every missing name
and exits non-zero if one of them is a bare "cannot find symbol". That asks
whether a name EXISTS; `scripts/coverage/behaviour.py` asks whether it answers
the same thing a JDK does, by calling every answered method with real arguments
and diffing — 2532 calls over 233 classes. The same run
reports how much is answered, with a refusal counting as MISSING rather than as
known.

That measurement used to be taken over 56 hand-picked classes — less than a
quarter of the 232 core `java.*` classes caturra names, and the 176 it skipped
were skipped for no better reason than that nobody had written down a receiver
expression for them. All 232 are measured now: **3138 of 3385 method names
across 225 classes**, and `--why` exits zero: every one of the 3385 either runs
or explains itself. A receiver that does not itself compile is reported as a
measurement failure rather than counted as a class scoring zero — eleven were,
on the first run, and none is now.

## Non-goals (for now)

Threads (`synchronized` parses and runs — on one thread a monitor is never
contended — but nothing is concurrent), class loading of user-supplied
`.class`/`.jar` binaries, JNI, security manager, modules, floating-point
`strictfp` distinctions, and `java.net`. If one of these becomes needed, it gets
its own spec first.

`java.util.Date`, `Calendar`, `GregorianCalendar`, `SimpleDateFormat` and
`TimeZone` are a non-goal for a MEASURED reason rather than a guess:
`new Date().toString()` prints a timezone abbreviation with daylight saving
applied, and `Calendar`/`SimpleDateFormat` are shaped around the same default
zone. Answering that honestly needs a real timezone database, which caturra
deliberately does not vendor (see `java.time` below). They stay refusals that
name themselves, rather than approximations that would be wrong by an hour
twice a year.

Two entries that used to sit here have since been built and are no longer
non-goals: `java.time` (the arithmetic slice — see `specs/LANGUAGE.md`; what
needs a time ZONE is still out) and `java.nio.charset` (the six standard
charsets, encoding and decoding exactly as a JDK's do). Reflection has likewise
grown past "non-goal" into the read-only surface `getClass`, `getDeclaredFields`
and friends give.

## Licensing rule

The repo stays MIT. We may study Apache-2.0 projects (TeaVM, GWT/J2CL, Harmony)
and the Java SE specifications/javadoc for _behavior_, but we do not copy GPL
(OpenJDK) source into this repo. Behavioral edge cases should be captured as
tests, written from the spec, not lifted code.

## Consequences

- The VM interpreter must define an intrinsic ("native method") mechanism and a
  bootstrap class-loader resolution order: intrinsics → baked-in classlib →
  user-compiled classes.
- The engine build gains a step: compile `classlib/*.java` with our compiler and
  embed the results. Until our codegen works, VM development can proceed against
  `.class` fixtures generated by a real `javac -target 11` at dev time (fixtures
  only — never shipped).
- Exact-method compatibility with the AP Quick Reference becomes a test suite of
  its own (assert signatures and observable behavior per class).
