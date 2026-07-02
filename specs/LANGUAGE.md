# LANGUAGE — compiler surface and staging

- **Status:** accepted
- **Date:** 2026-07-02
- **Refines:** [SCOPE.md](SCOPE.md)

## Strategy

Hand-written recursive-descent parser with error recovery — chosen for
student-quality diagnostics and zero dependencies. The parser recognizes more
than the compiler can compile: constructs we plan to support but haven't built
yet produce a **friendly, specific diagnostic** ("`if` statements are not yet
supported by jvmjs") with a source span, never a cryptic parse error. Constructs
outside SCOPE.md entirely (lambdas, streams, modules) get a "not supported by
jvmjs" diagnostic phrased as a scope statement, not a bug.

The full target surface is defined by [SCOPE.md](SCOPE.md); this file tracks
what each stage of the compiler actually accepts, starting from the vertical
slice.

## Vertical slice (v0) — accepted today

- Compilation unit: one or more top-level class declarations (no `package`,
  no `import` — parsed and reported as not-yet-supported, then skipped).
- Class declaration: modifiers, name, `{ ... }` body of method declarations.
- Method declaration: modifiers (`public` / `static` etc.), `void` or named /
  primitive / array return type, parameter list, block body.
- Statements: blocks and expression statements.
- Expressions: literals (`int`, `double`, `String`, `char`, `boolean`, `null`),
  dotted names (`System.out`), and method calls on dotted names.
- Callable surface: `System.out` / `System.err` `print` / `println` with zero
  arguments or one literal argument.

Everything else parses into a not-yet-supported diagnostic with recovery, so a
file full of future-Java still reports one clear message per construct.

## Codegen choices

- Class file format: major version 55 (Java 11), as fixed in
  `jvmjs-classfile`.
- String concatenation compiles to `StringBuilder` chains (javac-8 style), not
  `invokedynamic`/`StringConcatFactory` — keeps the VM free of `indy` support.
- No `StackMapTable` emission (see [RUNTIME.md](RUNTIME.md) — our VM is the
  only verifier).
- No default `<init>` constructor is emitted until object instantiation lands
  (v0 classes are never instantiated). This is a deviation from javac output
  and is revisited with `new`.

## Staging after v0 (each stage flips its diagnostics to real support)

1. Local variables, assignment, arithmetic/comparison/logical operators,
   string concatenation.
2. Control flow: `if`/`else`, `while`, `for`, `break`/`continue`.
3. Static methods with parameters and returns (user-defined), recursion.
4. Arrays (1D, then 2D), `for-each`.
5. Objects: fields, constructors, `new`, instance methods, `this`.
6. Inheritance: `extends`, `super`, overriding, polymorphic dispatch;
   `interface` / `abstract`.
7. Generics as far as `ArrayList<E>` requires (erasure, autoboxing).

The staging order optimizes for what CSA course units need earliest.
