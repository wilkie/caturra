# VALIDATION — reflection, a class loader, and the "Test" run mode

- **Status:** proposed (future phase; not yet implemented)
- **Date:** 2026-07-04
- **Refines:** [RUNTIME.md](RUNTIME.md), [EXECUTION.md](EXECUTION.md)

## Motivation

Code.org's CSA levels ship a **validation program** separate from the student's
code. It is not run as the student's `main`; it is a distinct **Test /
Validation execution** that inspects the student's classes with reflection,
invokes the student's `main`, captures its output (console text and/or the
neighborhood action stream), and asserts correctness — the automated "Did I get
it right?" check behind each level.

jvmjs cannot run these today. In the curriculum picker we currently **strip the
reflection-based validator file** (any source importing `java.lang.reflect`
whose class nothing else references) so the student program compiles; the check
itself is dropped. This spec describes what it would take to actually run
validation. It is a self-contained phase, orthogonal to the language/VM work.

## How the real system does it (from `javabuilder`)

- **Execution modes** (`ExecutionType`): `RUN`, `TEST`, `COMPILE_ONLY`.
  Validation is a flavour of test run.
- **Two file sets.** `JavaRunner` compiles the student's `javaFiles` *and* the
  level's `validationFiles` together, but they are distinct inputs — the
  validator is teacher-authored and not part of the submission.
- **`ValidationRunner`** loads the compiled classes through a `URLClassLoader`,
  builds a `ValidationProtocol` context holding: the student `main` (found via
  reflection), a `NeighborhoodTracker`, a `SystemOutTracker`, and the list of
  class names; then runs JUnit tests from the validation classes via a
  `JavabuilderTestExecutionListener`.
- **`ValidationProtocol`** is what the validation tests call: `invokeMainMethod()`
  runs the student's program under capture; `getSystemOutMessages()` returns what
  it printed; the neighborhood tracker records the painter action stream.
- **Reflection surface actually used** is modest and structural — inspecting
  shape, not deep metaprogramming. Observed in the Unit 2 validators:
  `Class.getDeclaredFields() / getSuperclass() / getName()`,
  `Field.get(obj) / set(obj,v) / getName() / getType() / getModifiers()`,
  `obj.getClass()`, and `Modifier.*`. Method/constructor reflection
  (`getDeclaredMethods`, `getMethod(...).invoke`, `getConstructor(...).newInstance`)
  appears in later units.

## What jvmjs must add

In rough dependency order. Each item is independently useful.

1. **A reflective class registry.** jvmjs already holds every loaded class
   (`MethodTable` in the compiler, the VM's class map) with fields, methods, and
   supertypes. Reflection is a *read view* over that data plus instance field
   access — not new execution machinery. Expose: class by name, declared
   fields (name/type/modifiers), declared methods/constructors (signatures +
   invocation), superclass, and get/set of an instance field by `Field`.

2. **`java.lang.reflect` intrinsics.** Model `Class`, `Field`, `Method`,
   `Constructor`, `Modifier` as intrinsic-backed heap objects (like `Scanner`,
   `PrintWriter` today). `Field.get/set` read/write the heap object's slots;
   `Method.invoke` / `Constructor.newInstance` call through the existing
   interpreter dispatch with a reflected argument array. `setAccessible` is a
   no-op (jvmjs does not enforce private access at the VM level — access control
   is a compile-time check). Start with the structural subset (item's surface
   above); add method/constructor invocation next.

3. **A class loader seam.** Today classes are loaded up front and referenced by
   `ClassId`. Reflection needs name→class lookup and the ability to hold a
   `Class` handle. This is mostly surfacing the existing registry; a real
   `URLClassLoader` is unnecessary — a single flat namespace (which jvmjs
   already has) suffices.

4. **A minimal test runner ("JUnit-lite").** Validation classes use JUnit
   (`@Test`, assertions). Rather than port JUnit, recognize `@Test`-annotated
   methods (or a convention), run each, treat a thrown `AssertionError` /
   exception as failure, and report pass/fail per test. Provide the handful of
   `org.junit.jupiter.api.Assertions` methods the validators use
   (`assertEquals`, `assertTrue`, `assertNotNull`, …). Annotations require the
   compiler to at least *parse and ignore* `@Test`/`@DisplayName` (it currently
   rejects annotations) and retain them as metadata for the runner.

5. **Output capture + `ValidationProtocol`.** `invokeMainMethod()` runs the
   student `main` with `System.out` captured to a buffer (`SystemOutTracker`)
   and the neighborhood action stream captured to a list — we already emit that
   stream to `neighborhood.jsonl` (see RUNTIME.md), so the tracker is a second
   sink over the same events. Expose `getSystemOutMessages()` and the action
   list to the validation code.

6. **The `VALIDATION` run mode.** A new entry alongside `run`/`runDebug`:
   compile `[validationFiles..., studentFiles...]`, then run the validation
   entry (its `@Test` methods) rather than the student `main`. Returns a
   structured pass/fail result (per-test), not stdout.

## Scope boundaries

- **Structural validation first.** The common case (Attributes, Constructors,
  Accessors/Mutators lessons) only needs field/superclass reflection + `main`
  invocation + `System.out` capture. Method/constructor invocation and richer
  JUnit come later.
- **No access-control enforcement.** `setAccessible(true)` is implicit; jvmjs
  never blocks private access at runtime (it is a `javac`-style compile check
  only), which actually simplifies `Field.get/set`.
- **Not real JUnit.** A convention-based `@Test` runner is enough; full JUnit
  Platform is out of scope.
- **Encrypted validators.** Some levels store the validator encrypted
  (`encrypted_examples`); those need the decrypted source, which we may not
  have. Out of scope until a source is available.

## Integration with the playground

- A **"Test" / "Check"** action beside Run: compiles the student's editor files
  plus the level's validation files (the ones the picker generator currently
  strips — they would instead be carried as a level's `validationFiles`), runs
  the `VALIDATION` mode, and renders per-test pass/fail.
- The picker's start-code-only data would gain an optional `validationFiles`
  field so a level can be *checked*, not just run.

## Open questions

- Do we have (unencrypted) validator sources for enough levels to be worth it,
  or only the plaintext `*Helper.java` cases we see in Unit 2?
- How much of the corpus's `java.lang.reflect` usage is this validation pattern
  (recoverable) versus student-facing reflection (rare, per the console survey)?
- Annotation support in the compiler is a prerequisite (parse-and-retain `@Test`)
  and is the one piece that touches the language front end.
