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

## Non-goals (for now)

Threads and `synchronized` semantics, reflection, class loading of user-supplied
`.class`/`.jar` binaries, JNI, security manager, modules, generics _erasure
corner cases_ beyond what `ArrayList<E>` needs, floating-point `strictfp`
distinctions, and full `java.time` / `java.net` / charset support. If one of
these becomes needed, it gets its own spec first.

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
