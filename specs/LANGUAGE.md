# LANGUAGE — compiler surface and staging

- **Status:** accepted
- **Date:** 2026-07-02
- **Refines:** [SCOPE.md](SCOPE.md)

## Strategy

Hand-written recursive-descent parser with error recovery — chosen for
student-quality diagnostics and zero dependencies. The parser recognizes more
than the compiler can compile: constructs we plan to support but haven't built
yet produce a **friendly, specific diagnostic** ("`if` statements are not yet
supported by caturra") with a source span, never a cryptic parse error. Constructs
outside SCOPE.md entirely (lambdas, streams, modules) get a "not supported by
caturra" diagnostic phrased as a scope statement, not a bug.

The full target surface is defined by [SCOPE.md](SCOPE.md); this file tracks
what each stage of the compiler actually accepts, starting from the vertical
slice.

## Accepted today (v0 slice + stages 1–8: the full SCOPE.md surface)

- Compilation unit: one or more top-level class declarations (no `package`,
  no `import` — parsed and reported as not-yet-supported, then skipped).
  A **public top-level type must be declared in a file named after it**
  (JLS §7.6), so at most one per file — `class Bar is public, should be
declared in a file named Bar.java`, javac's wording exactly, for classes,
  interfaces and enums (2026-07-09). Package-private types may share any
  file, as may a `public` **nested** type, which is not top-level. caturra
  ignored the rule until then, so a program that broke it compiled in the
  playground and failed on a real JDK; 22 of 6682 corpus levels break it,
  and javac rejects every one. Pinned by
  `reject_public_class_in_a_mismatched_file` and the wording table.
- Class declaration: modifiers, name, `{ ... }` body of method declarations.
- Method declaration: modifiers (`public` / `static` etc.), `void` or named /
  primitive / array return type, parameter list, block body.
- Statements: blocks, expression statements — method calls and class
  instance creation (`new Foo();` runs the constructor and discards the
  reference), the JLS §14.8 set; array creation (`new int[3];`) is not a
  statement, as in javac —
  local variable declarations (`int a = 1, b;`, `final double d = 2.5;`,
  `String s = "hi";`), assignment (`=`, `+=`, `-=`, `*=`, `/=`, `%=`), and
  statement-position `++`/`--` (prefix or postfix), which lower to
  `+= 1` / `-= 1`.
- Control flow: `if`/`else` (dangling `else` binds inner, like Java),
  `while`, `do`/`while`, `for` (including multi-declarator init,
  comma-separated updates, and `for (;;)`), and unlabeled
  `break`/`continue` binding to the innermost loop. Conditions must be
  `boolean`, and `if (x = 1)` gets a "did you mean '=='?" hint.
  Definite assignment is branch-aware: both `if`/`else` arms assigning
  counts, a lone branch or loop body doesn't, a `do` body does
  (JLS §16-style, conservative on constant conditions).
- Expressions, with Java precedence:
  - literals (`int`, `double`, `String`, `char`, `boolean`, `null`), with
    `-2147483648` folding so `Integer.MIN_VALUE` is writable;
  - local variable reads;
  - arithmetic `+ - * / %` and unary `-` with binary numeric promotion
    (JLS §5.6.2), 32-bit wrapping int semantics, `ArithmeticException` on
    int division by zero, IEEE double semantics (`Infinity`, `NaN`);
  - comparisons `< <= > >= == !=` (numeric, boolean equality, and reference
    equality on strings — literals are interned, so the classic `==` vs
    `equals` lesson works) with Java NaN behavior;
  - short-circuit `&&` / `||` and `!`;
  - primitive casts `(int) (double) (char) (boolean)` including Java's d2i
    saturation and i2c truncation;
  - string concatenation with `+`/`+=` (compiled to `StringBuilder` chains),
    formatting `int`/`double`/`char`/`boolean`/`null` exactly as Java does.
- **Fail-fast iteration** (2026-07-18): every collection's iterator and every
  for-each detect concurrent modification and throw
  `ConcurrentModificationException`, across ArrayList, LinkedList, HashSet,
  TreeSet, HashMap/TreeMap views, Stack, ArrayDeque and PriorityQueue. Before
  this, modifying inside a for-each silently skipped an element and *adding*
  looped forever — caturra hung rather than failing. The JDK's exact quirks are
  reproduced: `hasNext()` does NOT check, so removing the second-to-last
  element ends the loop with no exception; replacing a value (`put` of an
  existing key, `list.set`) is not a structural modification and never throws;
  `Iterator.remove()` is the one legal modification; and `removeIf` with a
  mutating predicate runs the predicate over the whole range before throwing,
  so its side effects land and the removals are abandoned (JDK 11 shape).
  Detection uses the collection's LENGTH as the JDK's `modCount` — every
  structural change to the collections caturra models changes the size. The
  documented gap: a modification that nets out to the same size between two
  `next()` calls (an add AND a remove) is not caught, where a real JVM would.
- **Compound assignment to a wrapper** (2026-07-19): the implicit cast of
  `E1 op= E2` (JLS §15.26.2) is a NARROWING PRIMITIVE conversion, so it exists
  only when the target is a primitive. `byte b = 10; b += 1000;` still wraps to
  -14, but `Byte b = 10; b += 1000;` and `Integer i = 1; i += 2.7;` are now
  refused as javac refuses them — assignment to a wrapper is a boxing
  conversion, and boxing does not narrow. A wrapper target whose result already
  fits (`Integer i += 2`, `Double d += 2`, `i++`) is unaffected.
- **`Arrays.asList` is a fixed-size, write-through view** (2026-07-19): backed
  by the array (`new HeapObject::ArrayBackedList`), so `list.set(0, x)` writes
  to `array[0]` and vice versa, while `add`/`remove` and the other
  length-changing operations throw `UnsupportedOperationException` — it used to
  be an independent mutable copy. `Collections.nCopies` is now immutable too
  (wrapped in the existing `UnmodifiableList`). `new ArrayList<>(view)` still
  makes an independent mutable copy. **Documented gap:** `Arrays.asList` of a
  PRIMITIVE array (`asList(int[])`) still returns a list of the elements rather
  than the one-element `List<int[]>` Java produces — a deliberate Java gotcha
  javac itself warns about, near-zero in practice, and it needs the nested
  array-element machinery to model.
- **`java.util.Objects`** (2026-07-20): the null-safe static helpers —
  `equals(a, b)`, `hashCode(o)`, `hash(o...)`, `toString(o)` /
  `toString(o, default)`, `isNull` / `nonNull`, and `requireNonNull(o)` /
  `requireNonNull(o, message)`. `equals`/`hashCode` dispatch a user
  `equals`/`hashCode` override (a VM intrinsic, reusing the same
  `java_equals`/`java_hash_code` that back collection membership), `hash` folds
  its arguments exactly like `Arrays.hashCode(Object[])`, `toString` renders
  like `String.valueOf` (so `null` is `"null"`), and `requireNonNull` throws
  the JDK's `NullPointerException` — with the given message or `getMessage() ==
  null`. `requireNonNull` returns its argument's inferred type `T` (a checkcast
  narrows the erased `Object` back), so `String s = Objects.requireNonNull(x)`
  needs no cast. A primitive argument autoboxes into the `Object` parameters.
- **try-with-resources** (2026-07-19) follows JLS §14.20.3's translation
  rather than the bare `finally { r.close(); }` it used to desugar to. The
  body's exception WINS and `close()`'s is attached as suppressed (reachable
  via `getSuppressed()`, and `addSuppressed` is available directly); the
  resource declarations sit INSIDE the guarded try, so an exception from a
  resource initializer is caught by the statement's own catch and any earlier
  resource still closes; a null resource is skipped rather than dereferenced;
  and the Java 9 form `try (existingVariable)` parses. Multi-catch now
  enforces JLS §14.20: alternatives may not be related by subclassing, and the
  parameter is implicitly final. **Not checked:** that a resource's type
  implements `AutoCloseable`. The desugaring happens in the parser, before any
  type is known, and the obvious cast-based check would reject `PrintWriter`
  and `Scanner` — which caturra models as intrinsic types rather than as
  classes implementing the interface.
- **Dispatch** (2026-07-19): overload resolution runs JLS §15.12.2's PHASES —
  everything applicable without boxing is considered first, and only if nothing
  matches does boxing enter — so `m(Integer)` beats `m(int)` for an `Integer`
  argument and `f(long)` beats `f(Integer)` for an `int` (which used to be
  reported ambiguous). A call through a superclass-typed reference reaches the
  SUPERCLASS overload: the bridge pass had been synthesizing a bridge for any
  superclass `Object` parameter, which hijacked ordinary overloads, and now
  only a real erased type variable qualifies. A PRIVATE method binds
  statically (`invokespecial`), so a superclass's call to its own private
  method no longer lands in a subclass's unrelated private method of the same
  name. Overriding a `final` method (JLS §8.4.3.3) and weakening an override's
  access (§8.4.8.3) are rejected, as javac rejects them.
- **Set.equals and the tree collections' first insert** (2026-07-19):
  `AbstractSet.equals` is specified across implementations and is symmetric, so
  a `HashSet` equals a `TreeSet` holding the same elements — it used to demand
  a `HashSet` on the other side, which made it one-way. A `keySet()` view
  compares as a Set too (a `values()` view does NOT: `AbstractCollection` does
  not override `equals`, so it stays identity, as on a JDK). Separately,
  `TreeSet.add`/`TreeMap.put` now compare the first element WITH ITSELF, the
  JDK's "type (and possibly null) check": before, the first insert compared
  against nothing, so `null` and a non-`Comparable` element were accepted and
  only the SECOND insert complained.
- **Interface inheritance semantics** (2026-07-20): a `private` interface method
  is not inherited (JLS §9.4), so calling it on an implementing class is now the
  compile-time "cannot find symbol" javac gives, not a method that resolved and
  died at run time — the resolver skips an inherited interface's `private` (and,
  as before, `static`) methods. And a type that inherits the same DEFAULT method
  from two UNRELATED interfaces without overriding is an ambiguous inheritance
  (JLS §8.4.8): `class C implements A, B` (both `default x()`) — and equally
  `interface AB extends A, B` — is now the "types A and B are incompatible" error
  rather than a silent arbitrary pick. A related override (a sub-interface's more
  specific default, or the type's own) resolves it, as on javac.
- **Varargs overload resolution picks the most specific** (2026-07-21, JLS
  §15.12.2.5): when several varargs methods apply, `f(Integer...)`/`f(String...)`
  beats `f(Object...)` (its element is more specific, and `Object...` is the
  least specific) instead of caturra reporting the call ambiguous. The
  fixed-arity path already did this; the varargs path only counted candidates.
  **Documented gap:** two overloads that differ ONLY as `f(int...)` vs
  `f(Integer...)` are still rejected as duplicate definitions — caturra stores
  wrapper arrays unboxed, so both erase to `int[]` (the same representation limit
  behind passing an `Integer[]` to `Object...`).
- **Interface member validation** (2026-07-20, JLS §9.4): an interface method
  may not be `protected` or `final`, `static` and `default` are mutually
  exclusive, a `static`/`default`/`private` method must have a body while a plain
  abstract one must not, and a `default` method may not override
  `toString`/`hashCode`/`equals` from `java.lang.Object` — all silently accepted
  before (the `protected`/`default` modifiers were parsed and discarded). An
  interface field, implicitly `public static final`, must have an initializer
  (`interface F { int X; }` is now the "= expected" error). Valid interfaces —
  abstract + default + static + private methods and constants — are unaffected.
- **Primitive-to-`Object` cast** (2026-07-20): `(Object) i` performs a boxing
  conversion (JLS §5.5.1), the cast counterpart of `Object o = i;` — so
  `((Object)(long) 5).getClass()` is `Long`. It was rejected as an incompatible
  type even though the assignment form already boxed.
- **Constant-expression narrowing** (2026-07-20, JLS §5.2): a constant of type
  byte/short/char/int whose value fits assigns to a narrower byte/short/char
  without a cast — not only an int literal (`byte b = 5`) but a char literal
  (`byte b = 'A'`), a constant `final` variable (`final int c = 65; char ch =
  c`), and folded arithmetic (`byte b = x + y` for final `x`, `y`). A new
  `const_int` resolves constant variables (reusing `const_eval`) and folds the
  arithmetic and integer-bitwise operators in i64 — a wrapping overflow only
  makes a value LESS likely to fit, so any imprecision stays stricter than javac,
  never looser. The same rule now range-checks a switch case label against a
  byte/short/char selector (JLS §14.11), so `switch (aByte) { case 200: }` is the
  compile error javac gives instead of a silently-accepted always-false label.
- **Float parsing and hex-float formatting** (2026-07-20): `Double.parseDouble`
  and `Float.parseFloat` accept the Java grammar Rust's parser rejects — a
  trailing type suffix (`"1.0f"`, `"3.14d"`) and the hexadecimal form
  (`"0x1.8p1"` = 3.0) — instead of throwing NumberFormatException and aborting;
  invalid strings still throw. The `%a`/`%A` formatter conversion (hexadecimal
  floating-point) renders the text `Double.toHexString` produces (`0x1.0p0`),
  where it used to throw UnknownFormatConversionException.
- **Exception semantics** (2026-07-20): a USER exception subclass now widens to
  its bundled superclass (`Exception e = new MyException()` — the assignment
  matrix gained the `Object -> Exception` arm `widens` already had); its inherited
  Throwable/Object methods resolve (`getClass`, `getLocalizedMessage`, `getCause`,
  `initCause`, `add`/`getSuppressed`, `printStackTrace` — the compiler falls back
  to the exception method table for any throwable receiver, and the VM aliases
  `getLocalizedMessage` to `getMessage`). `initCause` is now modelled; a
  `Throwable[]` type resolves (as `Object[]`, so reading an element needs a cast
  to `Throwable`, which is now a legal down-cast). A failed cast reports JDK 11's
  module/loader parenthetical (`... are in module java.base of loader
  'bootstrap'`). **Gap:** an unwrapped `getSuppressed()[i].getMessage()` still
  needs an explicit `(Throwable)` cast — `ElemType` has no exception variant.
- **Static-initialization order** (2026-07-20, JLS §12.4): (1) a class whose
  `<clinit>` throws is permanently Erroneous — the first active use throws
  ExceptionInInitializerError, and every LATER use now throws
  `NoClassDefFoundError: Could not initialize class X` (a new `init_failed` set;
  the class no longer re-initializes or reads its fields as defaults). (2) An
  inherited static METHOD called through a subclass name (`Sub.m()`) emits the
  DECLARING class in the method ref, so only that class initializes — the method
  counterpart of the inherited static-field fix. (3) An enum's explicit `static`
  block runs AFTER all constants are constructed (the constant initializers, not
  just the user fields, now precede it in `<clinit>`); it used to run BETWEEN two
  constant constructions.
- **Four round-5 crashes fixed** (2026-07-21): (1) a method call on a bare `new
  Object()` (equals/hashCode/toString/getClass) aborted with "malformed class
  java.lang.Object" — the unloaded-class guard now lets `java/lang/Object` reach
  the Object-method fallback, and the default `toString` dots the name. (2) a
  for-each over a wildcard `List<?>` holding boxed ints VerifyError'd — the
  wildcard element resolves to `Object` but the list stores primitives unboxed,
  so the fetch now BOXES (a new list `__get` at the interpreter level). (3)
  `Map`/`List`/`Set.forEach(null)` throws NullPointerException instead of an
  internal "unknown native member". (4) an inner-class FIELD initializer reading
  an enclosing instance field NPE'd — the `__caturraOuter` link is now stored
  before the field initializers, not after.
- **Enums, round 2** (2026-07-31, round 7) — 8 of 10 findings, six of them
  the dangerous direction:
  - **The declarations JLS §8.9 forbids** now are: a `final` or `abstract`
    modifier on the enum (it is implicitly both, as its constants
    require), an access modifier on its constructor (implicitly private),
    and a redeclaration of `values()` or `valueOf(String)` — the last was
    not merely accepted but REPLACED the synthesized member, so a user
    `values()` returning null made `E.values().length` throw.
  - **A static declaration inside a constant's body** is refused ("Illegal
    static declaration in inner class"), since that body is an anonymous
    class; a `static final` constant variable stays legal, as in Java.
  - **`new E()` says "enum types may not be instantiated"** and `class C
    extends E` says "cannot inherit from final E" — both used to complain
    that the constructor could not be applied to `()`, which is not why
    either is wrong. Both checks exempt what the desugaring itself
    generates: the constants' own construction, and a constant body (an
    anonymous subclass of the enum, the one thing that may extend it).
  - **A constructor may delegate with `this(...)`**: the enum rewrite used
    to insert the hidden name/ordinal stores AHEAD of the delegation,
    which both displaced the mandatory-first call and made the blank final
    the delegate assigns look unassigned. The synthetic arguments are
    threaded THROUGH the delegation instead, so only the constructor that
    ends the chain stores them.
  - **Two left open**: `java.lang.Enum` is not a nameable type (`Enum<E> e
    = E.A`), and a PRIVATE instance field of the enum read by simple name
    from a constant body is accepted where javac refuses it — a real javac
    quirk (a non-private field, or a getter, compiles on both).
- **Generics, round 3** (2026-07-31, round 7) — 8 of 12 findings:
  - **A type variable in ARGUMENT position keeps its bound**: `<T extends
    Number> T firstOf(List<T> l)` could not `return l.get(0)`, because the
    erased `List<T>` element read out as `Object`. The erasure marker now
    carries the bound, so the element reads as `Number` while applicability
    stays as permissive as a type variable requires.
  - **`new ArrayList<T>()`** — the type ARGUMENTS of a `new` expression are
    erased like any others; they were left alone, so the constructor saw a
    bare `T` and refused an element type it could not name.
  - **A generic constructor declaration** (`<T> H(T t)`) parses: the type
    parameters are read before either the constructor or the method shape
    is recognized.
  - **`new T[n]` is "generic array creation"** (JLS §15.10.1): the type
    variable stays marked through erasure so the array creation can refuse
    it, instead of silently allocating an `Object[]`.
  - **`list.add(null)` on a `List<?>`** is the one legal write to a
    wildcard collection (`null` is assignable to every reference type); the
    blanket refusal of element writes now lets it through.
  - **An override may name the SUBSTITUTED return type**: `class SBox
    extends Box<String>` inherits `String get()`, so declaring `String
    get()` is an ordinary override, not a covariant one. Accepting it
    required fixing DISPATCH first — the JVM looks up by descriptor, and
    the parent's erased `T get()` matched the call site while the child's
    `String get()` did not, so the inherited body ran. Virtual resolution
    now prefers a same-name, same-PARAMETERS method in the more derived
    class before climbing, which is exactly a covariant override (an
    ordinary overload differs in its parameters and is never captured).
  - **An intersection cast** (`(Comparable<String> & Serializable) s`)
    parses and erases to its first type (JLS §4.9), instead of being a bare
    "expected an expression". `java.io.Serializable` itself is now named as
    unsupported rather than "cannot find symbol".
  - **Four left open**, all about the explicit type WITNESS
    (`Collections.<String>emptyList()`): caturra parses it and drops it, so
    its arity is unchecked and it neither narrows a result nor types a
    varargs call. Honoring it means carrying the witness on the call node
    through resolution, which is its own change.
- **Method references, in every form and context** (2026-07-31, round 7):
  - **Target-typed wherever a lambda is**: a CAST (`(Op) P::m` — the usual
    way to give a reference a type where nothing else would), an ARRAY
    INITIALIZER (`Op[] ops = { P::m }`), a FIELD assignment through another
    object (`q.stored = q::add`), and both branches of a CONDITIONAL. The
    last needed a type rule as well: two synthesized lambda classes are
    unrelated, so the conditional joined them at `Object`; it now joins at
    the interface they share (the erased least upper bound), when there is
    exactly one.
  - **The parse shapes**: `super::m`, `String[]::new` and `int[]::new`
    (modelled as the allocating lambda they denote), `Box<String>::new`,
    and an explicit witness `Type::<T>m` (erased). `super::m` cannot be a
    plain lambda — a synthesized class may not make a non-virtual call on
    another object's superclass — so the enclosing class gets a bridge
    method that does, exactly as javac emits one.
  - **A bound reference's variable need not be effectively final** (JLS
    §15.13.3): its receiver is read when the reference is created, so a
    later assignment cannot be observed. Synthesized reference classes are
    named apart from lambda classes so the capture rules can differ.
  - **The checks that decide which of the four forms applies** (JLS
    §15.13.1), each of which caturra used to accept or misdiagnose: a
    STATIC method named through an instance, a name fitting BOTH the static
    and unbound-instance forms (ambiguous), an arity fitting neither (it
    blamed a wrapper class it had inferred), a checked exception the
    interface does not declare, and `SomeEnum::new` (which said the
    constructor took the wrong arguments rather than that enums may not be
    instantiated).
  - **Two gaps left open**, both about WHEN a bound receiver is evaluated:
    `make()::read` re-evaluates the receiver expression on every call
    (Java evaluates it once, at reference creation), and a null receiver
    NPEs at the first call rather than at creation. Both need the receiver
    hoisted out of the synthesized class's body, which is a statement-level
    rewrite this pass does not do yet.
- **Unicode escapes are translated BEFORE lexing** (2026-07-31, round 7,
  JLS §3.3) — the unicode-text cluster:
  - **A `\uXXXX` escape is a property of the SOURCE TEXT, not of string
    literals.** caturra decoded them inside literals only, so six programs
    behaved differently from javac: `"\u0022"` compiled (the quote should
    close the literal early), `"a\u000Ab"` and `'\u000A'` compiled (the
    escape is a real line terminator, so both are unclosed), a `\u000A`
    inside a `//` comment did NOT end the comment — silently hiding live
    code — and the `\uuuu0041` and identifier forms were rejected outright.
    A new translation pass runs before the lexer, honoring the eligibility
    rule (a backslash preceded by an EVEN number of backslashes, so
    `"\\u0041"` stays six characters) and combining a SURROGATE PAIR of
    escapes into the supplementary character it spells. Known gap: an
    UNPAIRED surrogate escape still renders as U+FFFD, because caturra's
    tokens hold Rust `char`s.
  - **Octal escapes** (JLS §3.10.6): one to three digits, at most `\377`,
    with the three-digit form needing a leading 0-3 — so `\400` is `\40`
    then a literal `0`, and `\1234` is `S4`.
  - **`Character.toChars`/`toCodePoint`** and **`String.codePoints()`** (a
    real `IntStream`, where a surrogate pair counts once) — the last had
    been refused with the FALSE claim that streams are unsupported.
  - **The Character data tables**, generated from JDK 11 itself so they
    match the Unicode version caturra targets: `getNumericValue` knows the
    letter-numbers, superscripts and fractions (Roman `Ⅷ` is 8, `½` is -2);
    the integer parsers accept every Unicode decimal digit
    (`Integer.parseInt("٣٤")` is 34, as `Character.digit` always allowed);
    `toTitleCase` returns the THIRD form of the twelve Latin digraphs (`ǆ`
    titlecases to `ǅ`, not to `Ǆ`); `equalsIgnoreCase` compares code unit
    by code unit with the SIMPLE mappings (`"İ".equalsIgnoreCase("i")` is
    true — lowercasing whole strings with Rust's FULL mapping expanded
    `\u0130` to two characters); and `isLetter` excludes the
    letter-NUMBERS that `isAlphabetic` includes.
- **Floating point: the total order, the parse grammar, the literal shapes**
  (2026-07-30, round 7):
  - **`Double`/`Float` impose a TOTAL order** (JLS §4.2.3): NaN is greater
    than everything and equal to itself, and -0.0 is strictly less than
    0.0. Every sorted structure compared PRIMITIVELY, so
    `partial_cmp(...).unwrap_or(Equal)` made each NaN comparison "equal" —
    a NaN swallowed a `TreeSet` (four elements collapsed to two, the map to
    one), `Collections.max` never saw it, and a `PriorityQueue` polled in
    the wrong order. One shared comparison now canonicalizes NaN and uses
    the IEEE total order, which is exactly `Double.compare`. The identity
    relations beside it are unchanged: `Double.valueOf(NaN).equals(NaN)` is
    true, `0.0.equals(-0.0)` is false, `NaN == NaN` is false.
  - **`Double.parseDouble`/`Float.parseFloat` take Java's word forms
    only** — `NaN` and `Infinity`, with an optional sign and no type
    suffix. Rust's parser also accepts `inf`, `infinity` and `nan` in any
    case, so `parseDouble("infinity")` had been returning Infinity where a
    JDK throws NumberFormatException.
  - **The floating-literal shapes the lexer did not know** (JLS §3.10.2):
    the fraction digits after the dot are OPTIONAL (`5.`, `5.d`, `5.e2`),
    and HEXADECIMAL literals (`0x1.fp3` = 15.5) have a mandatory binary
    exponent — `0x1.f` is javac's "malformed floating point literal". A hex
    float was previously read as `0x1` followed by a field access `.fp3`.
  - **A literal that does not fit its type is an error**: "floating point
    number too large" when it rounds to an infinity, "too small" when a
    NONZERO significand rounds all the way to zero — `1e400`, `1e-400` and
    `1e40f` were silently becoming Infinity/0.0. `0e-400` and `0.0e999`
    stay legal, as on javac, because their significand is zero.
- **Arrays check their stores** (2026-07-30, round 7) — the arrays cluster:
  - **`ArrayStoreException` exists** (JLS §10.5): a store through a widened
    array reference whose value does not fit the RUNTIME component type
    throws, naming the value's class. caturra had the check but it saw only
    `[L…;` elements of known classes, so an ARRAY-typed element
    (`String[][]` holding a row) and a `Number[]` both accepted anything.
    The check now walks array components recursively (covariance, one level
    at a time) and knows the wrappers' closed hierarchy under `Number`; an
    element type whose subtypes caturra cannot enumerate still accepts
    anything, so no legal program gets a spurious throw.
  - **`System.arraycopy` performs the JVM's checks, in the JVM's order,
    with the JVM's messages**: not-an-array (`arraycopy: source type
    java.lang.String is not an array`), kind mismatch (`type mismatch: can
    not copy int[] into double[]`), then source index, destination index,
    negative length, and the two last-index checks — `arraycopy: last
    source index 4 out of bounds for int[3]`, with "object array" naming
    every reference array as the JVM does. The bounds messages used to
    report a bare, in-bounds-looking index.
  - **A reference copy is checked element by element**, so the elements
    that fit are copied and the first that does not throws — a partially
    written destination, observable exactly as on a JDK.
  - **`arraycopy`'s array parameters are `Object`**, as javac's are: a
    non-array argument compiles and throws at run time. (This was a
    documented deliberate strictness; the runtime check it stood in for now
    exists, so the strictness is gone.)
  - **`Class` knows about arrays**: `isArray()`, `getComponentType()`
    (`null` for a non-array, a primitive `Class` for `int[]`), and
    `getSimpleName()` spelling `int[]` / `String[][]` instead of the raw
    descriptor. `Class.toString()` now gives the JDK's `class
    java.lang.String` / `interface Marker` / bare `int` — it used to print
    the INTERNAL `class java/lang/String`, a name Java never produces.
- **`finally` runs, whatever discards what** (2026-07-30, round 7) — the
  finally cluster, all silent-wrong: when an inner finally's `return` or
  `break` discarded a pending return, every ENCLOSING finally was skipped —
  including a try-with-resources `close()` — and an outer finally's
  overriding `return` never executed. One mechanism bug:
  `emit_pending_finallys` took the whole pending stack while emitting, so an
  abrupt exit INSIDE a finally body saw no enclosing entries; it now pops
  one entry at a time, leaving the outer entries visible (JLS §14.20.2).
  Pinned alongside: return-value capture before the finally reassigns,
  `continue` through nested finallys, and an exception thrown in a finally
  replacing the original.
- **Class initialization, four ways truer** (2026-07-30, round 7):
  - **The entry class initializes at startup** (JVMS §5.5): its static
    blocks and static field initializers (superclass first) run before
    `main`'s first statement — they used to be skipped entirely.
  - **A failed `<clinit>` poisons the class** on every later active use —
    field read, static call, AND `new` (the pre-decoded `new` fast path
    cached the site and re-ran constructors; it now honors the failure) —
    with the JDK's `NoClassDefFoundError: Could not initialize class X`.
  - **Superinterfaces that declare default methods initialize with their
    implementor**, before it (JVMS §5.5); an interface without defaults
    still waits for a direct use of its own non-constant field.
  - **An escaping `<clinit>` exception is ExceptionInInitializerError**
    even when UNCAUGHT — the wrap used to fire only on the caught path, so
    the raw cause leaked to the top. The uncaught trace matches the JDK's:
    EIIE at the triggering use site, `Caused by:` with the in-clinit frames
    and `... N more` elision.
- **The inheritance tail** (2026-07-30) — round 6's last two findings, and
  with them ROUND 6 FULLY CLOSED (71/71):
  - **Object methods resolve through an interface-typed reference** (JLS
    §9.2): every interface implicitly declares a public abstract member for
    each public `Object` method, so `i.toString()`, `i.equals(x)`,
    `i.hashCode()`, and `i.getClass()` all compile on a variable,
    parameter, or collection element typed by a user interface. The method
    walk simply never reached `Object` from an interface (interfaces have
    no superclass chain); resolution now falls back to the synthetic
    Object entry when an interface walk comes up empty.
  - **`super.m()` naming a HIDDEN static method is legal** (JLS §15.12):
    it resolves statically to the superclass's method — including one the
    superclass inherited — and emits a plain static call; `super` there is
    a type qualifier, not a receiver. The old refusal ("it has no body
    there") was factually wrong. `super.m()` to an ABSTRACT method still
    errors, now with javac's wording ("abstract method m() cannot be
    accessed directly").
- **Exception traces, filled at construction** (2026-07-30) — the round-6
  trace cluster:
  - **A throwable's stack trace is captured when it is CONSTRUCTED** (as
    `fillInStackTrace` in the Throwable constructor does), kept per object,
    so `catch (E e) { throw e; }` reports the ORIGINAL throw site, an
    exception built by a factory traces to the `new` site, and the
    exception's own constructor chain is hidden (a user subclass's
    `<init>` frames don't appear — the JDK hides them too). A caught
    VM-raised exception (an AIOOBE from the array machinery) adopts the
    frames from its error text, so rethrowing IT preserves them as well.
  - **The uncaught rendering is the JDK's, in full**: header, `\tat`
    frames, every `Suppressed:` block (tab-indented, from
    try-with-resources), the `Caused by:` chain — recursively, with the
    frames each enclosed trace shares with its enclosing one elided as
    "... N more" (`Throwable.printEnclosedStackTrace`, faithfully) and a
    `[CIRCULAR REFERENCE: …]` guard.
  - **`printStackTrace(System.out)` (and `System.err`)** compiles and
    prints that same full rendering — the argument names a standard
    stream, encoded for the VM to route (an honest refusal for any other
    PrintStream expression); the no-argument form now prints the full
    trace to stderr instead of just the header line. A user override of
    `printStackTrace` still wins.
  - **`getSuppressed()` returns a real `Throwable[]`** (new
    `ElemType::Throwable` carrying the exception id): assignable to its
    declared type, indexable, iterable, elements reaching `getMessage()`
    without casts — `Throwable[]`/`Exception[]` declarations now resolve
    as throwable arrays generally (they were `Object[]`).
  - Known cosmetic gap: a NESTED class in a trace frame prints its
    flattened name (`R.close`) where the JVM writes `Outer$R.close` —
    caturra flattens nested classes at compile time.
- **StringBuilder sub-ranges** (2026-07-30) — five round-6 findings:
  - **The Java-5 sub-range overloads exist**: `append(CharSequence, start,
    end)` (null appends the sub-range of "null"), `insert(dst, CharSequence)`,
    `insert(dst, char[], offset, LEN)` and `insert(dst, CharSequence, start,
    END)` — note the char[] form takes a length where the CharSequence form
    takes an end index, as in the JDK. A plain String argument picks the
    CharSequence overload (a new `BParam::CharSeq`, most-specific against
    the Object overload), and `new StringBuilder((CharSequence) s)` seeds.
  - **Bounds failures are the JDK's, class and message**: the sub-range
    checks throw the plain `IndexOutOfBoundsException` saying "start S, end
    E, length L" (unclamped; the char[] append's end is offset + len with
    int wrap-around) — except the char[] INSERT form, which throws
    `StringIndexOutOfBoundsException` with the same text, the JDK's own
    split — and the destination-offset check ("offset D,length C") fires
    before the sub-range check. `append(char[], -1, 1)` used to throw the
    wrong class (ArrayIndexOutOfBounds) with a Rust usize underflow
    (18446744073709551615) leaking into the message.
  - **Self-insert aliases, as the JDK's does**: `insert(dst, CharSequence)`
    copies in place AFTER shifting the tail, so inserting a builder into
    itself reads the already-shifted chars (`new StringBuilder("abab")
    .insert(1, self)` is "aaaaabab") — while the `insert(dst, Object)`
    overload snapshots via `String.valueOf` ("cdcd"), both pinned.
  - **`(CharSequence) null` — and null cast to ANY reference type — is
    legal** (JLS §5.5): the overload-selection idiom compiles, interface
    targets included (`(Comparable<String>) null`); a non-null Object
    downcast to CharSequence still `checkcast`s at runtime.
- **Switch labels and the switch scope** (2026-07-30) — four round-6 findings:
  - **Any constant expression is a case label** (JLS §15.29): arithmetic,
    shifts, char arithmetic (`case 'a' + 1:`), compile-time string
    concatenation (`case "he" + "llo":`), and `final` constant variables
    (JLS §4.12.4 — the textbook `final int MENU_QUIT = 3; … case
    MENU_QUIT:`). caturra accepted only bare literals and static-final
    field names, and its refusal ("case labels must be constants") was
    factually wrong about labels that WERE constants; the messages are now
    javac's ("constant expression required" / "constant string expression
    required"). Folded labels join the duplicate check, so `case 13:` and
    `case 2 * 3 + 7:` collide, exactly as on javac.
  - **Shifts fold** in the shared constant folder, with exact int
    semantics (count masked to five bits) — safe because a `long` literal
    or `final long` variable is a distinct `Literal::Long` the int folder
    never matches, so `1L << 40` stays a runtime computation. Bonus:
    `byte b = 1 << 3;` (constant narrowing through a shift) now compiles.
  - **The switch block is ONE scope** (JLS §6.3): a variable declared in
    one case group is in scope in every later group — `case 1: int v; …
    case 2: v = 20;` compiles, and redeclaring in a later group is
    "variable 'v' is already defined", not "cannot find variable".
  - **Definite assignment per case group** (JLS §16.2.9): every group can
    be jumped into directly, so a variable declared (even initialized) in
    an earlier group is NOT definitely assigned at the next group's start —
    reading it there is javac's "variable 'v' might not have been
    initialized" (it used to be the wrong claim that v did not exist).
    Assign-then-read within the fallen-into group stays legal, and the
    existing all-ways-out merge for DA after the switch is unchanged.
- **Interface defaults, resolved right** (2026-07-30) — four round-6 findings:
  - **The most specific default wins** (JLS §9.4.1): a sub-interface's
    redeclaration overrides the default it inherits, no matter where the
    interfaces sit in the implements list. The VM's interface fallback used
    to answer with whichever interface its traversal reached first, so
    `class P implements Left, Right` (Left overriding Top's default, Right
    inheriting it) silently flipped answers with the implements order. Now
    every providing interface becomes a candidate and any candidate that a
    more specific candidate's interface extends is dropped. (Two UNRELATED
    interfaces defaulting the same signature is a compile error, already
    enforced.)
  - **A lambda body is lexically scoped** (JLS §15.27.2): a bare `apply(y)`
    inside `y -> apply(y) * 2`, written in the interface's own default
    method, means the ENCLOSING instance's method — never the SAM the
    lambda is defining. codegen resolved the name against the synthesized
    `Lambda$` class first, which turned the standard decorator/combinator
    idiom into a StackOverflowError. Bare calls in a lambda class now skip
    self-resolution and ride the captured-outer chain, exactly as bare
    field reads already did (the capture pass was already right).
  - **Abstract redeclarations of public Object methods don't count** toward
    the single abstract method (JLS §9.8): an interface declaring
    `describe()` plus abstract `toString()`/`equals(Object)`/`hashCode()`
    is functional (`java.util.Comparator` redeclares `equals` this way);
    caturra counted them and refused the lambda. An interface with two REAL
    abstract methods still rejects.
  - **An ambiguously inherited constant is a compile error** (JLS
    §6.5.6.1): `implements CA, CB` with a constant `K` in each made a bare
    `K` silently resolve to whichever the field walk hit last; now the
    reference errors with javac's wording ("reference to K is ambiguous:
    both variable K in CA and variable K in CB match"), including across a
    superclass/interface pair. Not ambiguous, as in Java: one declaration
    reached along two diamond paths, a declaration in the class itself
    (which hides everything it would inherit), a private field in a
    supertype (not inherited), and qualified access (`CA.K`).
- **The Map API tail** (2026-07-30) — seven round-6 findings:
  - **The compute family inserts at the bucket HEAD**: a NEW key from
    `computeIfAbsent`/`compute`/`merge` links at the front of its bucket
    chain (`tab[i] = newNode(hash, key, v, first)` in the JDK), unlike
    `put`'s tail append — observable in iteration order whenever keys share
    a bucket (`put("a",1); computeIfAbsent("q",…)` prints `{q=…, a=1}`).
    The map model gained a per-entry chain sequence (tail-appends ascending,
    head-inserts descending), which survives resizes because two keys in the
    same final bucket were in the same bucket at every smaller table size.
  - **A mistyped remapper VARIABLE is rejected at the call site**: `merge`
    wants `BiFunction<? super V, ? super V, ? extends V>`, but the declared
    type arguments erase before codegen, so `m.merge(k, v, g)` with a
    `BiFunction<K, V, V>` compiled and CCE'd at run time. The lambda pass —
    the last place the declaration is visible — now checks it (and
    `computeIfAbsent`'s `Function`), rejecting only PROVABLE mismatches
    (both sides concrete final library types), with javac's exact message.
    This gave the lambda pass its first diagnostics channel.
  - **Method references reach the map lambda methods**: `m.merge(k, v,
    Integer::sum)` and `m.computeIfAbsent(s, String::length)` desugar like
    the equivalent lambdas (they were refused with "only allowed where a
    functional-interface type is expected").
  - **No import needed**: the map lambda methods work without any
    `java.util.function` import — the bundled function library's injection
    triggers now include `.merge(`/`.compute`.
  - **`entrySet()` assigns to its own type**: `Set<Map.Entry<K, V>>` now
    RESOLVES as the entry-set view type (it resolved as `Set<Object>` and
    the assignment was refused). Since that spelling also names a real
    `HashSet` of entries, the view's method table gained the mutating Set
    surface: `add` (UnsupportedOperationException on an actual view, a real
    add on a `HashSet`), `remove(entry)` (removes the mapping when key AND
    value match, like `contains`), and `clear` (writes through, like
    `keySet().clear()`).
  - **`AbstractMap.SimpleEntry`** is a standalone `Map.Entry` — modelled as
    an entry view over a hidden one-mapping map, so `getKey`/`getValue`/
    `setValue`/`toString`/`hashCode` flow through the existing entry
    machinery (it was refused with the wrong diagnosis "package AbstractMap
    does not exist"). `Map.Entry.equals` arrived with it: key-and-value
    equality against any other entry, live or standalone.
- **The String API tail** (2026-07-30) — eight round-6 findings:
  - **`strip()`/`stripLeading`/`stripTrailing`/`isBlank` use JAVA's
    whitespace** (`Character.isWhitespace`), not Unicode's: the two differ on
    U+001C–1F (Java: whitespace) and the non-breaking spaces (Java: not).
    Rust's `char::is_whitespace` had silently substituted the Unicode set, so
    the string methods disagreed with caturra's own classifier.
  - **`equalsIgnoreCase(null)` is `false`** (as the Javadoc says), not an NPE.
  - **`replaceAll`'s out-of-range group reference** (`"$5"` against a
    groupless pattern) throws `Matcher.group`'s IndexOutOfBoundsException
    ("No group 5") — the JDK reads the first digit unconditionally and only
    extends while the number stays a valid group; `$x` remains
    IllegalArgumentException.
  - **`concat("")` returns `this`** — `a.concat("") == a` is observably true
    on a JDK (while `a + ""` mints a new string).
  - **`String.valueOf(char[], int, int)`** and the `copyValueOf` subrange
    (with StringIndexOutOfBounds checks).
  - **`lines()` and `chars()` are real streams** — a `Stream<String>` split
    on `\n`/`\r\n`/`\r` (no trailing empty line) and an `IntStream` of the
    UTF-16 units; both had refused with the FALSE reason "streams are not
    supported". The lambda pass knows them as stream heads, so
    `lines().forEach(...)` and `chars().map(...).sum()` desugar.
  - **The dangling-metacharacter message names the character** ("Dangling
    meta character '+' near index 0"), as the JDK's does.
- **The round-6 mechanical batch** (2026-07-30) — fifteen findings, seven
  small fixes:
  - **Unary plus promotes** (JLS §15.15.3): `+aChar` is an int, so `char r =
    +c` is javac's lossy-conversion error, `println(+c)` prints 65, and
    `f(+c)` picks `f(int)` over `f(char)`. A new `UnaryOp::Plus` — the parser
    used to drop the `+` entirely.
  - **`var` is contextual** (JLS §3.9): a variable, method, field or
    parameter may be NAMED `var` (`int var = 7; var = var + 1; var()`), and a
    declaration is recognized only when `var` is followed by a name. And
    **`var l = new ArrayList<>()` infers the `Object` form** through the
    raw-type resolution — the diamond usually adopts its element from the
    declaration context, but `var` IS the context.
  - **A duplicate nested label rejects** (JLS §14.7, accepts-invalid): `lab:`
    inside `lab:` compiled and silently bound `break lab` to the INNER loop.
  - **A comma-separated statement-expression list in a for INITIALIZER
    parses** (`for (i = 0, j = 3; ...)`, JLS §14.14.1) — the update list
    already did.
  - **Compound assignment and `++`/`--` on a wrapper-array element** work
    (`Integer[] arr; arr[0] += 2; arr[0]++`): unbox on load, re-box on store,
    a fresh box per write with neighbours' identity untouched.
  - **The Character int-codepoint overloads** (`isDigit(int)` family,
    `isAlphabetic(int)` — the JDK's only signature — `toUpperCase(int)`,
    `getNumericValue(int)`, `digit(int,int)`) and `Character.SIZE`/`BYTES`.
  - **`ListIterator` is importable**; **`StrictMath`** (and any unmodeled
    static receiver, e.g. `Thread`) now refuses with its honest reason
    instead of "cannot find symbol".
- **Override validation in full** (2026-07-30, JLS §8.4.8.3 / §9.6.4.4) —
  round 6's remaining accepts-invalid trio, one seam:
  - An override may not **weaken access** below the overridden method's level
    (public > protected > package > private; only the private case was
    caught). `MethodDecl` now records `protected`, and a side table carries
    each user method's level; javac's detail is appended ("attempting to
    assign weaker access privileges; was public").
  - An override may not **broaden checked exceptions**: each checked
    exception it declares must be covered by one the overridden method
    declares (unchecked additions stay free) — reusing the §11.2 pass's
    exception identity ("overridden method does not throw Exception").
  - **`@Override` must override or implement something** — it was retained
    for the JUnit runner and never validated. The match is
    erasure-TOLERANT: an ancestor parameter that is `Object` or an erased
    type variable accepts any declared parameter, so `compare(Card, Card)`
    implementing the erased `Comparator` (and `set(String)` overriding a
    generic `set(T)`) are overrides, exactly as the erasure bridge
    dispatches them; classes under an unmodeled library superclass are
    exempt. The whole corpus — thousands of `@Override`s on interface
    implementations, JUnit tests and Swing subclasses — compiles unchanged.
  - Bonus adjacent fix: a BARE inherited-throwable call inside a user
    exception (`getMessage()` in a `toString()` override) resolves — the
    receiver path had the exception-table fallback, the implicit-`this` path
    did not. And the round-6 "wrong-method blame" finding resolved as the
    documented covariant-return strictness being reported first in a
    two-error program: the sibling's genuine error is also flagged.
- **`String.format` nulls, boxes, and flag corners** (2026-07-30) — the
  round-6 format cluster, 8 findings.
  - **Null arguments render per conversion**, as the JDK's Formatter does:
    `%b` says "false", `%h` says "null" (`%H` "NULL"), and every other
    conversion the width/precision-treated string "null" — `%d`/`%x`/`%c` of
    null used to throw. The compiler's argument coercion is now null-guarded,
    so a null Object rides through to the formatter instead of being
    pre-stringified into the four-character `"null"` (which `%b` then called
    true).
  - **Boxed wrappers pass through as references** and the formatter unwraps
    them — so `%d` of an Integer still formats the number, and a NULL
    `Boolean` reaches `%b` as null (it used to NPE at the call-site unbox).
  - **`%c` of an impossible codepoint** (negative, or past U+10FFFF) throws
    `IllegalFormatCodePointException` with the JDK's hex message ("Code point
    = 0x110000") — a NEW exception class, registered and importable, where a
    replacement character was silently printed.
  - **`%,g` groups** its fixed-notation integer part; **zero-padding is
    ignored for NaN/Infinity** (spaces, as the JDK pads them).
- **The conditional operator joins every pair of types** (2026-07-30, JLS
  §15.25) — round 6's only crashes, plus four rejects. One `conditional_join`
  is now shared by `type_of` and the emitter (so the two can never disagree):
  - **boolean|Boolean is boolean, in either order** — the one non-numeric
    primitive pairing in the JLS table. `t ? aBoolean : true` used to
    VerifyError: `widens` picked the type but the branch coercion never
    unboxed the Boolean branch.
  - **A char/byte/short branch pairs with a fitting constant EXPRESSION int**
    (`t ? 'a' : 'b' + 1` is char), folded via `const_int` — only literals
    worked before.
  - **Unrelated pairs join at Object**, the erased least upper bound: String
    vs StringBuilder, Boolean vs Integer, int vs String ("count: " + (t ? n :
    "none")), even boolean vs char — each branch boxes or passes through as a
    reference. javac never type-rejects a conditional once primitives box,
    and now neither does caturra; the join still refuses to narrow silently
    (`String s = t ? "s" : 1` stays an error, as on javac).
  - Wrapper pairs promote numerically as javac does: `t ? anInteger : aLong`
    is `long` (unbox, promote), so the joined Object is a `Long`.
- **CHECKED EXCEPTIONS ARE ENFORCED** (2026-07-30, JLS §11.2) — round 6's
  headline: five audit rounds never probed it, and caturra enforced nothing.
  A new `thrown.rs` pass (beside `flow.rs`) performs both javac rejections:
  - **Unreported exception** — a `throw` of a checked exception, or a call to
    a method/constructor that declares one, must be caught by an enclosing
    `try` or declared by the enclosing method: "unreported exception X; must
    be caught or declared to be thrown" (javac's sentence). Enforced at throw
    sites, user method/ctor calls (`throws` clauses are now RECORDED, not
    discarded), chained `this(...)`/`super(...)` ctors, static initializers,
    and the modeled library's checked throwers — a CLOSED-WORLD enumeration:
    reader `read`/`readLine`/`close` (IOException), `FileReader`/`PrintWriter`
    ctors and `new Scanner(file)` (FileNotFoundException),
    `File.createNewFile`, `Files.*` content operations, and the reflective
    surface (`Class.forName`, `getMethod`/`getField`, `newInstance`,
    `Method.invoke`, `Field.get`/`set`).
  - **Never thrown** (JLS §11.2.3) — a `catch` of a checked exception the try
    body cannot throw: "exception X is never thrown in body of corresponding
    try statement". `Exception` and `Throwable` are exempt, as javac exempts
    them.
  - **Conservative by construction**, like `flow.rs`: anything the syntactic
    pass cannot resolve (a chained receiver, an unresolvable overload)
    contributes an *unknown* marker that suppresses BOTH checks around it —
    a missed error is the status quo, a spurious one rejects a valid program.
    JLS §11.2.2 precise rethrow is modeled (an effectively-final catch
    parameter rethrows only what its clause caught), so
    `try {...} catch (Exception e) { throw e; }` keeps compiling in a method
    that declares only the specific exception. Bundled library units and
    synthesized lambda/anonymous/local classes are exempt (a lambda's
    contract lives on its erased functional interface). The whole
    2,600-level corpus compiles unchanged under the new rule.
- **BOXED AT REST** (2026-07-30) — the representation change, closing the last
  audit-round findings. Wrapper values now live BOXED in every container, so an
  element read through the `Object` boundary keeps its identity instead of
  minting a fresh box per read.
  - **`Integer[]` is a reference array**, distinct from `int[]` at last: a new
    `ElemType::Wrapper(Prim)` element. Slots default to NULL (the old `int[]`
    model zero-filled), reads are typed `Integer`, unboxing a null slot throws
    NPE, and `arr[0] == arr[1]` answers correctly in both directions (two 200s
    are two boxes; the same box twice is one). Covariant into `Object[]`,
    `Comparable[]`, and — numeric only — `Number[]`. `Arrays.sort/toString/
    equals/fill/copyOf/binarySearch/setAll` all work on the reference array
    (`setAll` boxes its generator results in), and **`Arrays.asList(arr)` is a
    write-through view of the caller's array** now that there is a shared
    reference array to back it.
  - **`f(int...)` and `f(Integer...)` are distinct signatures** (`[I` vs
    `[Ljava/lang/Integer;`) — the pair used to be rejected as duplicates. A
    mixed call (`f(1, 2)`) is ambiguous exactly as javac says; an array
    argument picks its own overload. An `Integer[]` also spreads into an
    `Object...`.
  - **The list family stores boxed references** — `ArrayList`, `LinkedList`,
    `Stack`, `ArrayDeque`, plus `Optional` and the `singletonList`/`singleton`/
    `singletonMap`/`nCopies`/`asList` construction boundaries — the convention
    maps and sets always used. A wrapper TYPE ARGUMENT is now
    `ElemType::Wrapper` (and `List<Long>`/`Float`/`Short`/`Byte` resolve,
    which they never did), `add` boxes at the boundary through the `valueOf`
    cache (so `l.add(5); l.add(5)` shares the cached box, exactly as the JDK's
    autoboxing does), and `get` returns the STORED reference. `IntStream`
    stays primitive; `boxed()` produces a `Stream<Integer>` of references.
  - **`Boolean.TRUE`/`FALSE` are the cached singletons**, not bare primitives:
    `Boolean.TRUE == Boolean.TRUE` is true, `new Boolean(true) == Boolean.TRUE`
    is false (JLS §15.9.4), and a `Boolean` operand of `&&`/`||` auto-unboxes.
  - **One documented strictness fell out:** `Collections.addAll(list,
    integerArray)` is legal Java that now WORKS (the wrapper array IS the
    varargs `T[]`); it was refused while `Integer[]` meant `int[]`. Reject
    wordings naming a list's element say `Integer` where they said `int`.
  - Verified: the whole 2,600-level grading corpus is byte-identical under the
    new representation, and identity now round-trips through every container
    (list/set/map/deque/Optional/array) in the pinned differential tests.
- **Round-5 tail: object methods, numeric corners, nested classes**
  (2026-07-30): the last fourteen round-5 findings outside the boxed-at-rest
  representation change.
  - **Library statics:** `String.valueOf(String)` and `valueOf(Object)` (the
    latter interpreter-answered so a user `toString` runs);
    `System.identityHashCode` (stable per object — the heap reference — 0 for
    null; only the identity PROPERTIES match a real JVM, whose values are
    address bits); `Objects.requireNonNullElse` (NPE message `defaultObj` when
    both are null); the LONG `Math.*Exact` overloads (`addExact(long,long)` &
    co., throwing the JDK's "long overflow" where the int ones say "integer
    overflow"); `Byte`/`Short.toUnsignedInt`/`toUnsignedLong` (zero-extension);
    and a `type_of` mirror for the `Objects` emitter, so `Objects.hash(...)` is
    usable in expression position (`== x` used to be "bad operand types").
  - **`java.lang.CharSequence` is a type** (`JType::CharSequence`): `String`
    and `StringBuilder` widen to it (JLS §4.10.2), a `CharSequence` parameter
    accepts either, and `length`/`charAt`/`toString`/`subSequence` dispatch on
    the actual heap object. `CharSequence -> String` still needs the cast, as
    on javac.
  - **`Collection.equals` takes `Object`**, as Java declares it: a list of a
    DIFFERENT element type is a legal argument (usually false, but `[]` equals
    `[]` across element types). It used to demand the receiver's own list type.
  - **A conditional of constants is a constant** (JLS §15.28): `byte b = flag ?
    1 : 2;` with a `final boolean flag` narrows (JLS §5.2). A new `const_bool`
    folds constant boolean conditions (literals, constant variables, `!`,
    comparisons of constant ints, `&&`/`||`/`&`/`|`/`^`); a non-constant
    condition still rejects exactly as javac does.
  - **`(Integer) null`** is a legal reference cast (unboxing the result throws
    NPE at runtime); it was "cannot cast null to Integer".
  - **`-9223372036854775808L` parses** (JLS §3.10.1): one-past-`Long.MAX_VALUE`
    is legal exactly as the operand of unary minus, where the pair spells
    `Long.MIN_VALUE` — the lexer folds the two tokens using the standard
    unary-vs-binary minus test. Without the minus, and as `1 - 9223…L`, it is
    still the error javac gives.
  - **Lossy-conversion wording matches javac:** "incompatible types: possible
    lossy conversion from int to char" (and double→int/char), dropping
    caturra's friendly "; add a cast" suffix, which read as our wording.
  - **`new Object() { ... }`** — an anonymous class may extend the synthetic
    top type (it was "cannot find symbol: class Object").
  - **Grand-enclosing capture** (JLS §6.5.6.1): a doubly-nested inner class
    reads, writes, and calls members TWO (or more) enclosing levels up, through
    a CHAIN of `__caturraOuter` hops — `enclosing_instance_field` and the bare-
    call fallback both walk the chain now, and the nearest enclosing class
    declaring the name wins (Java's shadowing rule).
- **Comparators and collection algorithms** (2026-07-24): the round-5
  comparator + collections cluster, 21 findings.
  - **Comparator combinators.** A `comparing`/`comparingInt` key extractor
    written as a LAMBDA (`Comparator.comparingInt(p -> p.a)`) now types its
    parameter from the sort/declaration context — the erased SAM's parameter is
    `Object`, so it used to report "cannot find symbol: field 'a' in class
    Object" and only method references worked. Added `thenComparingInt`/`Long`/
    `Double`, `nullsFirst`/`nullsLast`, and the two-argument
    `comparing(keyExtractor, keyComparator)` (new `ComparatorSpec::ByKeyWith`
    and `Nulls`).
  - **Comparator-taking static overloads:** `Arrays.sort(T[], cmp)` (VM-native,
    so the bundled `Arrays` need not name `__Comparator`), `Collections.max`/
    `min`/`binarySearch` with a comparator, and `list.sort(null)` /
    `Arrays.sort(a, null)` meaning natural ordering rather than an NPE.
  - **Collections list algorithms:** `rotate`, `fill`, `copy` (with the JDK's
    IndexOutOfBoundsException on a short destination), `disjoint`.
  - **Map lambda methods (JDK 8):** `merge`, `compute`, `computeIfPresent`,
    `computeIfAbsent`, `replaceAll` — including the remove-on-null semantics
    that make `merge` a counter that can also delete. They were refused with an
    honest "lambdas are not supported"; a new `__BiFunction` erased interface
    and `BParam::BiFunction` carry the two-argument remappers.
  - **`ListIterator`** — a bidirectional cursor (`hasPrevious`/`previous`/
    `nextIndex`/`previousIndex`/`set`/`add` on top of the `Iterator` surface),
    a new `JType::ListIterator(E)` reusing the VM's existing cursor object. Was
    refused as "iterators are not supported".
  - **`entrySet().contains(entry)`** asks for KEY-AND-VALUE equality (it used to
    answer a flat `false`).
  - **`forEach` is FAIL-FAST:** a consumer that structurally changes the
    collection is a ConcurrentModificationException, on lists, maps and sets —
    it used to walk a snapshot to the end and finish silently. Matches the JDK's
    two shapes: `ArrayList.forEach` re-checks in the loop (so one element is
    visited before it throws), `HashMap`/`HashSet` check once at the end.
  - **Two gaps left open.** A chained comparator combinator whose FIRST
    extractor is a fully-implicit lambda (`Comparator.comparingInt(p -> p.a)
    .thenComparing(...)`) is accepted where javac cannot infer `p`'s type
    through the chain and rejects it — caturra is slightly LOOSER on this
    inference corner (give the first lambda an explicit `(P p)` type and both
    accept it). And a collection lambda called directly on a diamond
    constructor (`new ArrayList<>(...).forEach(x -> ...)`) still cannot resolve
    the element type from the constructor argument — assign to a variable
    first. Both are element-inference limits, not new to this change.
- **A `Class` object is a singleton** (2026-07-23): every `Foo.class` and every
  `getClass()` on a `Foo` now returns the SAME reference, as on a real JVM
  (a new `class_pool` beside the string pool). Minting a fresh `Class` per query
  made the textbook `equals` idiom — `if (getClass() != o.getClass()) return
  false;` — always take the `false` branch, so a class written that way was
  never equal to anything: `equals` said false, `List.contains` said false, and
  a `HashSet` never deduplicated. Found while probing enum reflection, not by an
  audit round. `Class.equals`/`hashCode` now resolve too (identity, which is
  what interning makes meaningful).
- **Enum semantics, the silent/invalid batch** (2026-07-23):
  - `==` between two DIFFERENT enums, or an enum and a `String`, is javac's
    "incomparable types" (JLS §15.21.3 — an enum is implicitly `final` and
    extends only `java.lang.Enum`). caturra compiled both into an identity test
    that was always `false`, which reads as a legitimate answer. `null`,
    `Object`, an interface the enum implements, and the same enum are unaffected.
  - `A.X.compareTo(B.Y)` across two enums, and `card.compareTo(2)` on any
    `Comparable<Card>`, are now rejected. The erased `Comparable.compareTo(T)`
    inherited from a generic supertype is OVERRIDDEN by a nearer same-arity
    declaration, not overloaded by it — offering both made EVERY argument
    applicable, so the cross-enum compare ran and `A.X.compareTo("s")` reached
    the enum body with a String and died on "unknown field `__ordinal`".
  - A case label naming a constant of a DIFFERENT enum, a qualified name
    (`case A.X:`), or a typo is javac's "an enum switch case label must be the
    unqualified name of an enumeration constant" — it used to abort at class
    load with "malformed class: unknown static field A.B.P".
  - `Enum.valueOf`'s message names the enum canonically (`No enum constant
    Outer.Level.LOW`); the enclosing chain is applied when nested classes are
    hoisted, which is where it becomes known.
  - `getDeclaringClass()` is modelled — the enum TYPE, which for a constant WITH
    A BODY is not `getClass()`, that being the constant's anonymous subclass.
  - `getSimpleName()` of a synthesized ANONYMOUS class is `""`, as the JDK's is
    (it was reporting caturra's internal `Anon$N`).
- **An assignment used as an expression evaluates its target once**
  (2026-07-23, JLS §15.26.1/§15.26): `int v = (a[next()] = 5);` called `next()`
  TWICE and left `v` holding the element at the SECOND index; `(get().f = 7)`
  called `get()` twice and yielded `0`. The expression form stored and then READ
  THE TARGET BACK; it now duplicates the stored value under the store's operands
  (`dup_x2` / `dup2_x2` for an array, `dup_x1` / `dup2_x1` for a field, `dup` for
  a static), which is both the correct value and a single evaluation. Every
  target shape and every element width is pinned, including the compound and
  `String +=` forms.
- **`Arrays.asList` is fixed-size for every argument shape** (2026-07-23):
  `Arrays.asList(1, 2, 3).add(4)` used to succeed here and throw
  `UnsupportedOperationException` on a real JDK — only the inline
  reference-varargs form got the fixed-size view. A primitive-backed array is
  now copied into a private reference array that still backs a view, so
  `add`/`remove`/`clear` throw and `set` works. **Gap:** `set` on
  `Arrays.asList(anIntegerArray)` does not write through to the caller's array —
  caturra stores a wrapper array unboxed, so there is no `Vec<JValue>` to share
  (the `Integer[]` vs `int[]` representation limit again).
- **Generics, the deep round** (2026-07-23): seven round-5 findings, all of them
  ordinary generic Java that caturra REJECTED.
  1. **`java.lang.Number` is a type.** It used to answer only `instanceof`
     ("there is nothing to declare a variable of"), which made `<T extends
     Number>` — the most common bounded type parameter — unusable, because `T`
     erases to its bound. It is now a synthetic abstract class with the six
     conversion accessors (`intValue`/`longValue`/`doubleValue`/`floatValue`/
     `shortValue`/`byteValue`), the supertype of the six NUMERIC wrappers only:
     `Number n = true;` and `Number n = "x";` are still javac's errors, verbatim.
     A user class named `Number` shadows it, as it shadows every library name.
     `WildcardBound::NumberUpper` is gone with it — `? extends Number` is now an
     ordinary class bound, which is what lets `for (Number n : List<? extends
     Number>)` type its loop variable.
  2. **A generic method may take a PARAMETERIZED parameter** (`<T> void
     dump(List<T>)`, `<K, V> void show(Map<K, V>)`). The type argument erases to
     a type-variable wildcard: any element matches, as inference would — and
     unlike a `? extends` capture it may still be WRITTEN to, because a `T` is a
     real type. Reading such an element goes through a new `System.__box`, since
     the collection stores its primitives unboxed and only the VM knows which
     wrapper the wildcard hid.
  3. **A primitive boxes into a type variable** (JLS §5.3): `new Box<Integer>(42)`
     and `id(3)` were "cannot be applied to given types (int)".
  4. **INTERSECTION bounds** (`<T extends A & B>`) may call both bounds' methods.
     The JVM erasure is the leftmost bound, which hid `B` entirely; the compiler
     now works against an interface synthesized per intersection (`__And$A$B`),
     which a class satisfies exactly when it satisfies every bound. A class
     implementing only one is still not applicable.
  5. **An explicit type witness** (`Collections.<String>emptyList()`,
     `this.<T>id(x)`, JLS §15.12) parses. The arguments erase away, so skipping
     them is the whole fix; a `<` that is really a comparison is unaffected,
     since a witness can only sit directly after the `.`.
  6. **RAW types** (JLS §4.8): `List l = new ArrayList();`, `Map`, `Set`,
     `Collection`, `Iterator` and the rest resolve as their `Object`-argument
     form, which is what their members read and write. javac only warns here, and
     `java.util.HashMap x = null;` used to be refused as "not supported".
  7. **A user generic class in a signature** (`String join(Pair<A, B> p)`) —
     the descriptor writer had no arm for one, so every such method was rejected
     as an "unknown generic type". `Box<T>` only worked because a
     single-parameter class is described through its tracked form.

  **Gap left open:** a MULTI-parameter generic class still does not track its
  type arguments — `p.getB()` on a `Pair<String, Integer>` reads out `Object`,
  so `p.getB() + 1` is a compile error where javac infers `Integer`. Tracking
  needs one erasure sentinel per position (`JType::TypeVar` is singular by
  construction), which is its own change.
- **Inherited static field write** (2026-07-20): `Sub.f = v` on a field
  declared in a superclass resolves to the DECLARING class — it used to abort
  ("malformed class: unknown static field Sub.f") because the write emitted the
  referenced subclass in the field ref while the READ path already emitted the
  owner. Now consistent: the write finds the slot and initializes only the
  declaring class, not the subclass named (JLS §12.4.1). (The analogous inherited
  static METHOD call still over-initializes the subclass — a separate open item.)
- **String.intern() identity** (2026-07-20): `intern()` returns the CANONICAL
  pooled reference (the `string_pool` every literal shares, which `ldc`
  populates), so `new String("x").intern() == "x"` is true. It used to return the
  first heap string of equal content, which — because a `new String` object is
  allocated before its `ldc`'d argument — was the receiver's own copy. Handled at
  the interpreter level, where the pool is reachable. Separately, `new String(...)`
  now types as `String` in `type_of` (it already did in the emitter), so
  `new String("x") == "x"` compiles instead of "bad operand types".
- **JDK 11 NullPointerException messages** (2026-07-20): a VM-raised NPE in JDK
  11 has a NULL detail message — the "helpful" JEP-358 text ("cannot invoke
  \"String.length()\" because ... is null") is JDK 14+, enabled by default only
  in 15. caturra had synthesized the JDK-14 messages for a null method receiver,
  field read/write, array access, unbox, and `throw null`; all are now null, so
  `getMessage()` matches JDK 11.
- **Boolean logical operators and shifts** (2026-07-20): `&`/`|`/`^` on boxed
  `Boolean` operands now UNBOX (they reached `IAND` on two references — a runtime
  VerifyError that leaked a Rust value); `&=`/`|=`/`^=` on a boolean are the
  logical compound assignments (JLS §15.26.2), the only compound forms a boolean
  accepts, wired on local, field, static, and array targets; and a compound
  shift (`x <<= n`) treats its count as an independent integral operand
  (JLS §15.19), so `x <<= 1.5` is a COMPILE error rather than a VerifyError, and
  a wrapper shift target that cannot take the int/long result (`Byte b; b <<= 2`)
  is refused as javac refuses it.
- **Deep equals/hashCode** (2026-07-20): every collection compares and hashes
  STRUCTURALLY all the way down, so a nested `List<List<...>>`, a `Set` of
  `List`s, and a `Map` with collection values are deep-equal (and hash equal) —
  before, `equals`/`hashCode` recursed only into a user object's override, so a
  collection ELEMENT was compared by identity and two structurally-equal nested
  lists were unequal. The recursion lives in the central `java_equals`/
  `java_hash_code`, so it also fixes `list.contains(aNestedList)`,
  `Objects.equals(list1, list2)`, and collection keys/values in a map. An
  `ArrayDeque`/`PriorityQueue` (a Queue, not a `List`) keeps identity equality,
  as on a JDK. Each boxed wrapper also hashes as its own type — `Boolean` is
  1231/1237 (not its 0/1 value), `Long`/`Double`/`Float` fold their bits — where
  the boxed `hashCode` had returned the raw value. And `String.equals(Object)`
  accepts any argument (`"1".equals(1)` compiles and is false, the int
  autoboxing); it had demanded a `String`.
- **Wrapper dispatch and identity** (2026-07-19): `list.remove(Integer.valueOf(2))`
  removes the VALUE, not the element at index 2. Both `remove(int)` and
  `remove(Object)` exist and caturra stores list elements UNBOXED, so both
  modelled as `int` and the table order decided it — silently, and it is the
  classic Java trap. `pick_builtin` now applies JLS §15.12.2's rule that an
  overload applicable WITHOUT boxing beats one that needs it, reading the real
  parameter kinds off the descriptors. `Integer.valueOf(int)` accordingly
  answers a REFERENCE (through the autoboxing cache, so `valueOf(100) ==
  valueOf(100)` is true and `valueOf(200)` is not), while the deprecated
  `new Integer(5)` mints a fresh object every time (JLS §15.9.4) and is never
  `==` to another. Two more silent-vanish paths closed alongside: a comparison
  whose operand does not type, and `new Integer("7")`, which the emitter
  modelled and `type_of` did not.
- **Integer/Long radix and decode API** (2026-07-21): the radix overloads —
  `Long.toString(long, int)`, `parseLong`/`parseInt`/`valueOf(String, int)`,
  `parseUnsignedInt`/`parseUnsignedLong(String[, int])`, and
  `Integer.decode`/`Long.decode` (sign + `0x`/`0X`/`#` hex, leading-`0` octal,
  decimal) — were absent (or `decode` blamed "system properties"). Added across
  the compiler tables and the VM. An out-of-range radix now throws
  `radix N less than/greater than Character.MIN_RADIX/MAX_RADIX`, and a
  `parseUnsignedInt` overflow `String value N exceeds range of unsigned int.` —
  the JDK's own wording, not the generic "For input string".
- **`Short`/`Byte` wrappers** (2026-07-20): `Short.valueOf((short) 5)` and
  `Byte.valueOf((byte) 3)` box instead of crashing with a "not a String"
  `ClassCastException` — the descriptor returned the wrapper (so the VM
  autoboxed) but the compiler typed the result as a primitive and re-boxed it,
  and the second `valueOf` reached the String-parsing arm. Typed `BRet::Wrapper`
  now, like `Integer.valueOf(int)`. Two more divergences fell out: `Short.compare`
  and `Byte.compare` return `x - y` (the JDK source), NOT the -1/0/1 sign
  `Integer.compare` gives; and the `parseShort`/`parseByte` range
  `NumberFormatException` message is `Value out of range. Value:"99999"
  Radix:10` with NO trailing `(Short)` — caturra had appended a spurious class
  suffix. `Short.valueOf(Object)` remains a compile error, as on javac.
- **A call can no longer vanish** (2026-07-19): when an argument's type could
  not be determined, `builtin_instance_call` emitted the arguments "for nested
  diagnostics" and bailed — and when there were none, the call disappeared
  entirely: no code, no value, no diagnostic. `l.addAll(Arrays.asList("d"))`
  left the list untouched and `println(l.containsAll(...))` printed nothing at
  all. The trigger was an inline `Arrays.asList(...)`, which the EMITTER knew
  and `type_of` did not (the same divergence as the type-variable values
  above). `type_of` now mirrors it, and the bail reports a caturra-limitation
  diagnostic when nothing else did, so the failure mode cannot recur silently
  through any other unmodelled argument.
- **Generics, the dangerous direction** (2026-07-19): four programs javac
  refuses no longer compile — a type argument in `instanceof`
  (JLS §15.20.2; a wildcard `List<?>` is still fine), a STATIC member using the
  class's type parameters (JLS §8.4.1; a method-level `<U>` is fine), two
  methods of one class sharing an ERASURE (JLS §8.4.2), and writing to a
  `? extends` collection (JLS §4.5.1 — reads such as `get`/`size`/`contains`
  stay legal, and a `? super` collection is still writable). Enforcing the
  `instanceof` rule found TWO places in caturra's own tree relying on it being
  accepted: the bundled JUnit and a test program, both since corrected.
  Also fixed: a value whose declared type is a TYPE VARIABLE
  (`box.get()`, `box.v` on a `Box<String>`) typed as nothing in the pure
  `type_of` path though the emitter substituted the tracked argument — so
  `box.get() + box.get()` was not seen as a concatenation and the whole
  `println` produced NO OUTPUT AT ALL, with no diagnostic. A binary operator
  whose operands type as errors now says so rather than vanishing.
- **Parameterized supertypes and bridge methods** (2026-07-18): a subclass
  stands in for its parameterized supertype — `Box<String> b = new SBox()`,
  `F<String> f = new SF()` — and the call REACHES THE OVERRIDE. Erasure gives
  `Box` a `set(Object)` and `SBox` a `set(String)`: two descriptors, so
  nothing overrode anything and the call reached the superclass. A pass
  (`bridges.rs`) synthesizes the bridge javac emits — a `set(Object)` that
  casts and delegates. The two halves are inseparable: allowing the assignment
  without the bridge would have traded a compile error for a silent wrong
  answer. The type argument a subclass writes on its supertype is now RECORDED
  (the parser used to skip it), so `Box<String> b = new IntBox()` is still
  refused; a raw `extends Box` records nothing and passes as unchecked, as
  javac has it. `new Pair<String, Integer>(...)` constructs too — only
  single-parameter classes track their argument, but any class that declares
  type parameters may be parameterized.
  **Still refused, deliberately:** a COVARIANT return (`String f()` overriding
  `Object f()`). Dispatch is by descriptor, so it needs a bridge, and a bridge
  differing only in return type cannot be written in source — its body would
  resolve back to itself. Accepting it without one made `((A) new B()).f()`
  answer A's method, a silent wrong answer; the refusal is the safe direction
  and is pinned by `stricter_than_javac!`.
- **Flow analysis** (`caturra-compiler/src/flow.rs`, 2026-07-18): statement
  reachability (JLS §14.21) and blank-final definite assignment (JLS §8.3.1.2,
  §16.9). Code after `return`/`throw`/`break`/`continue`, and the body of a
  constant-false loop, are rejected as `unreachable statement` — while
  `if (false) { ... }` stays legal, the conditional-compilation carve-out that
  applies to `if` and nothing else. A blank `final` field must be definitely
  assigned by the end of every constructor, may not be assigned twice, and may
  not be read before it is assigned; before this a blank final silently read 0
  and `final` meant nothing for a field. Definite assignment for LOCALS lives
  in the codegen tracker and honours constant conditions (`if (true) x = 1;`),
  abruptly-completing branches (`if (c) x = 1; else return;`) and JLS §16.2.10
  (`while (true) { x = 5; break; }` — assigned before every exiting break).
  Deliberately conservative: only boolean *literals* count as constant
  conditions, not the wider constant expressions of JLS §15.28, since a missed
  error costs only strictness while a spurious one rejects a valid program.
  **Not implemented:** checked-exception analysis, so an unreported checked
  exception and a `catch` for one the body cannot throw are both still accepted.
- User-defined **static methods**: parameters, `return` (with javac-style
  missing-return and unexpected-return checks via the same analysis,
  so `while (true) { ... return ...; }` needs no trailing return),
  recursion and mutual recursion (guarded by `StackOverflowError`, see
  RUNTIME.md), bare same-class calls and `ClassName.method(...)` across
  classes/files, results usable in any expression or discarded as a
  statement. Overload resolution follows JLS §15.12.2 without boxing:
  applicable-by-widening, exact match preferred, unique most-specific
  method, javac-worded ambiguity/cannot-find-symbol/non-static errors.
- Callable surface: the above, plus `System.out` / `System.err`
  `print` / `println` with zero arguments or one argument of any supported
  expression type.
- Literals: exponent notation (`1e10`, `2.5E-3`) lexes as double.
- **Arrays** (any element of `int`/`long`/`short`/`byte`/`double`/`float`/
  `boolean`/`char`/`String`/a class, any dimension count — every primitive
  leaf allocates correctly through `multianewarray`, which `long[][]`,
  `float[][]`, `short[][]` and `byte[][]` did not until 2026-07-09): `new T[n]` / `new T[n][m]` / `new T[n][]` (via
  `newarray`/`anewarray`/`multianewarray`), `{...}` initializers in
  declarations and `new T[] {...}` (nested for 2D, ragged rows fine, and a
  trailing comma is legal — JLS §10.6),
  `a[i]` reads/writes with compound assignment and `++`/`--` on elements,
  `a.length` / `m[i].length`, arrays as parameters and returns (rows are
  references — aliasing behaves like Java), the C-style declarator spelling
  where the brackets follow the name — `String args[]`, `int grid[][]`,
  fields, and locals — binding to that declarator alone, so `int a[], b;`
  makes only `a` an array (JLS §10.2); the legacy method form `int m()[]`
  is not accepted. Reference `==`, an array widened to `Object` (`(Object)
arr`, or passed to an `Object` parameter) and cast back down (`(int[])
obj`, `(String[]) obj`, `(int[][]) obj` — 2026-07-09; the primitive-array
  forms did not parse before, the reference forms were rejected in codegen)
  with a runtime `checkcast` that throws `ClassCastException` exactly as
  the JVM does (a primitive array is invariant and is not `Object[]`; a
  reference or nested array is), and
  `for (T x : array)` desugared to an indexed loop (element widening
  applies; `break`/`continue` work). Runtime exceptions use Java 11's
  wording: `ArrayIndexOutOfBoundsException: Index 5 out of bounds for
length 3`, `NegativeArraySizeException`, `NullPointerException`.
  `main`'s `String[] args` is now fully usable. Printing or
  concatenating a whole array gives Java's `Object.toString()` — the class
  descriptor, `@`, and the identity hash in hex: `[I@1b6d3586`,
  `[Ljava.lang.String;@4554617c`, `[[I@7f31245a` (2026-07-09; caturra used
  to refuse to compile it). Useless to read, and exactly what a student
  sees on a JDK. `getClass().getName()` gives the same name, spelled with
  dots. `println(char[])` is a real overload and prints the **characters**,
  while `"" + chars` is `String.valueOf(Object)` and prints `[C@hash` —
  the Java trap is reproduced rather than smoothed over. The heap carries
  a reference array's class descriptor, because once the static type is
  gone it is the only thing that still knows the element type. Pinned
  against a real JDK by `diff_array_default_to_string`. `Arrays.toString`
  remains the way to see the elements.

- **Classes with state** (stage 5): instance and static fields (with
  initializers, `final`, and Java's default values), constructors with
  overloading and the synthesized default constructor (JLS §8.8.9),
  instance methods with virtual dispatch, `this` (explicit and implicit
  — bare names resolve local → field → static field), `new ClassName(args)`,
  cross-class member access with `private` enforced using javac's wording
  ("x has private access in A", "non-static variable x cannot be referenced
  from a static context"), static fields initialized by a synthesized
  `<clinit>` run lazily on first class use, objects in arrays and as
  parameters (reference semantics, `==` identity), and
  `println(obj)`/concatenation calling `toString()` — null-safely, with the
  VM supplying Java's `ClassName@hex` default when a class doesn't define
  one. `this(...)` constructor chaining, `super`, `extends`, and
  `instanceof` wait for stage 6; String methods (`s.equals(t)`,
  `s.length()`) wait for the class library.

- **Inheritance** (stage 6): `extends` (single), `implements` (multiple),
  `interface` and `abstract class`/`abstract` methods with completeness
  checks in javac's wording ("B is not abstract and does not override
  abstract method go() in A"), method overriding with compatibility checks
  ("f() in B cannot override f() in A") and true polymorphic dispatch
  (including calls made from inherited method bodies), `super.method(...)`,
  `super(...)`/`this(...)` constructor chaining with Java's first-statement
  and field-initializer rules, inherited fields, subtype reference widening
  in assignments/arguments/overloads, `instanceof` (null is false), class
  casts with upcast elision and runtime `ClassCastException` ("class A
  cannot be cast to class B") — including a cast of a `null` literal,
  `(String) null`, which names an overload without turning `(x) - 1`
  into one — and superclass-first static initialization.
  **Casts completed 2026-07-18** (JLS §5.5): every reference widens to
  `Object` (`(Object) "hi"`, the ordinary way to pick an `Object` overload
  — previously rejected as an incompatible type); boxing casts
  (`(Integer) 5`, `(Double) d`) and the unboxing direction
  (`(int) (Integer) x`); a cast operand may now be a literal or itself
  parenthesized (`(Integer) 5` used to be a PARSE error, `(Short) (short) 3`
  too). A cast performs at most ONE boxing conversion and no numeric
  conversion beside it, so `(Long) 5`, `(Integer) 5.0` and `(Character) 65`
  stay errors, as javac has them. `T[]` erases to `Object[]`, so the
  unchecked `(T[]) new Object[n]` every generic container needs compiles.
  Still ambiguous by design: `(Integer) -5`, since `(x) - 1` cannot be told
  from a cast without type information.
  **Field hiding** (JLS §8.3, 2026-07-09): a subclass may declare a field
  with the same name as one in a superclass, of any type. The two are
  distinct slots, and which one an access means is fixed by the **static
  type** at the access site — `((Sup) sub).n` and `sub.n` read different
  fields, and each class's own methods see their own. Hiding is not
  overriding. The heap keys an instance field by its declaring class,
  because keying by name alone merged the two slots; that is why caturra
  rejected hiding outright until the storage could tell them apart. The
  bytecode already carried the owner — `getfield`/`putfield` resolve it
  as the JVMS does, walking the owner's superclasses. Reflection reads the
  slot of the class that declared the `Field`, and the debugger names a
  field by its declaring class only when the name is actually hidden.
  Pinned against a real JDK by `diff_field_hiding` and
  `diff_field_hiding_with_reflection`.
  **`super.field`** (2026-07-09) reads and writes the slot the superclass
  sees, including `super.n = v`, `super.n += 3` and `super.n++`. Fields do
  not dispatch, so that is all it means: the receiver is `this` and only
  the lookup moves up — from the superclass, walking further only if it
  must, so in `C extends B extends A` where both `A` and `B` declare `n`,
  `super.n` in `C` is `B`'s. javac's wording for the three refusals is
  matched: `n has private access in A`, `non-static variable super cannot
be referenced from a static context`, and `cannot find symbol` when the
  class has no superclass. Pinned by `diff_super_field_access`.
  **A static member reached through an instance** — `obj.staticField` and,
  since 2026-07-09, `obj.staticMethod(...)` — is legal if discouraged, as
  javac has it (it only warns under `-Xlint:static`). The receiver
  expression is evaluated for its side effects and then discarded, so
  `make().twice(3)` runs `make()`, and a `null` receiver does **not** throw:
  nothing is dereferenced. A static method inherited from a superclass
  resolves through it (JVMS §5.4.3.3), whether named by the subclass or
  reached through a subclass instance — `Derived.who()` used to compile and
  then die with `MalformedClass`. An **interface's static method is not
  inherited** (JLS §8.4.8): only `I.hi()` names it, never `C.hi()` or
  `c.hi()`, which caturra used to compile and crash on; `default` methods
  are inherited as usual. Pinned by
  `diff_static_method_through_an_instance`,
  `diff_static_interface_method_through_the_interface` and
  `reject_static_interface_method_through_a_class`.
  `protected` currently behaves like public (no packages).

- **The class library** (stage 7, intrinsics per SCOPE.md):
  - The full Java 11 `String` API over UTF-16 (2026-07-03), because
    students look methods up in documentation and expect them to exist:
    every instance method except the stream family
    (`chars`/`codePoints`/`lines`) and `getBytes` — those report an
    honest "String.chars exists in Java, but streams are not supported
    by caturra" rather than a misleading cannot-find-symbol.
    Statics: `valueOf` (all overloads incl. `char[]`), `copyValueOf`,
    and `String.format` (2026-07-03) — plus `System.out.printf` and
    `PrintWriter.printf` — via a compiler special-case that synthesizes
    the call descriptor from the actual argument types (the one shape
    the fixed signature tables can't express). The Formatter subset:
    conversions `b B s S c C d o x X e E f g G n % h H`, flags
    `- + 0 , ( #` and space, argument indexes (`%2$s`), width, and
    precision, with Java's exception types and messages
    (`IllegalFormatConversionException: d != java.lang.String`, ...).
    **Validation tightened 2026-07-19**: a duplicate flag
    (`DuplicateFormatFlagsException`), `-`/`0` with no width
    (`MissingFormatWidthException`), a width on `%n` or precision on `%%`
    (`IllegalFormatWidth`/`PrecisionException`), `#` on `%g`
    (`FormatFlagsConversionMismatchException`), and a leading `%<` with no
    preceding argument (`MissingFormatArgumentException`) are all thrown as a
    JDK throws them — previously rendered silently. Rendering corners fixed the
    same day: the `#` radix prefix (`0x`) precedes the zero-pad (`%#010x` of 255
    is `0x000000ff`), `%(f` parenthesizes a negative infinity, precision
    truncates `%b`, `%#.0f` keeps the trailing dot, `%g` of zero uses
    `precision-1` fraction digits, and `%c` widens a byte or short.
    Float conversions round HALF_UP over the shortest-round-trip
    decimal digits, matching Java's `BigDecimal.valueOf` path exactly
    (`%.2f` of `2.675` is `2.68`). **Known limit:** `%b`/`%h` of a NULL
    Object-typed argument (and `%b`/`%d`/`%f` of a non-null one) — the argument
    is coerced to a string at the CALL site because the formatter cannot invoke
    `toString` at render time, so a null becomes `"null"` where `%b` should be
    `false` and `%h` should be `null`. `join` still reports the varargs
    limitation. The regex family (`split`/`matches`/`replaceAll`/
    `replaceFirst`) is backed by caturra's own backtracking engine over
    UTF-16 units (`caturra-vm/src/regex.rs`, 2026-07-18): character
    classes, the predefined classes with Java's ASCII definitions of
    `\d`/`\w`/`\s`, greedy/reluctant/possessive quantifiers, groups,
    backreferences, alternation and anchors, with `split`'s exact JDK
    algorithm (limit semantics, the zero-width-at-zero rule, and the
    no-match case that makes `"".split(",")` one element). A malformed
    pattern is a real `PatternSyntaxException`. Before that `split` took
    its argument LITERALLY, which was silently wrong for any delimiter
    with regex meaning and silently right for the `","` case that
    dominates real code. `intern` preserves reference identity; `hashCode`
    is Java's exact algorithm. With Java 11 exception wording
    (`StringIndexOutOfBoundsException: String index out of range: 5`).
    `equals` accepts only strings/null (a documented narrowing of
    `equals(Object)`).
  - The full Java 11 `StringBuilder` API (2026-07-09), over the same
    UTF-16 code units `String` uses, so indices agree even across
    supplementary characters: `append` (every primitive, `char[]`,
    `String`, `Object`), `appendCodePoint`, `insert` (the same
    overloads, at any offset), `delete`/`deleteCharAt`/`replace`
    (whose `end` is clamped to the length, unlike `substring`'s),
    `reverse` (which keeps a surrogate pair together, as Java's does),
    `setCharAt`/`setLength` (which pads with the null
    character, as Java does), `indexOf`/
    `lastIndexOf` (both with and without a start), `substring`/
    `subSequence`, `charAt`/`length`/`getChars`, `compareTo`, the
    code-point family (`codePointAt`/`codePointBefore`/`codePointCount`/
    `offsetByCodePoints`), and `toString`. Pinned against a real JDK by
    `diff_string_builder_mutators`, `_search_and_extract` and
    `_code_points_and_errors`. `new StringBuilder(int)` accepts the
    capacity hint and ignores it: caturra models a builder's contents,
    not its backing array, so `ensureCapacity`/`trimToSize` are the
    no-ops they observably are, and `capacity()` — having no honest
    answer — reports a reason rather than a cannot-find-symbol
    (but `new StringBuilder(-1)` throws `NegativeArraySizeException`, as it
    allocates a `char[]`). **Rounded out 2026-07-19:**
    `new StringBuilder(otherBuilder)`, `equals` (Object IDENTITY — the classic
    trap), `append(char[], offset, len)`, `%s` of a builder, `(StringBuilder)
    null`, and a null builder printing as `null` all work; `append(null)` is
    rejected as ambiguous and `sb == aString` as incomparable types, both as
    javac rejects them; and the out-of-range exception messages match JDK 11
    exactly — `start`/`end` (not `begin`) for a builder, no space in
    `index N,length M`, a clamped `end` in `replace`, the JDK 11 `setLength`
    wording, and `Not a valid Unicode code point: 0x…` for `appendCodePoint`.
    **Still gaps** (all safe-direction, near-zero corpus demand):
    `CharSequence` as a declarable type; `String.valueOf(aBuilder)`;
    `StringBuffer` (report an honest reason); and an UNPAIRED-surrogate char or
    string LITERAL (`'\uD83D'`), which the lexer's `char`/`String` token types
    cannot hold, so it becomes U+FFFD — a core token-representation limit, not a
    StringBuilder one.
    `StringBuilder` is a full type wherever a type is written — a field, a
    parameter, a return type, or a captured lambda local (2026-07-09; the
    JVM descriptor builder handled `String`/`Scanner` but not
    `StringBuilder`, so those gave "unknown type 'StringBuilder'"),
    unblocking `list.forEach(x -> sb.append(x))`. Pinned by
    `diff_string_builder_as_a_type`.
  - The full Java 11 `Math` API for int/double (2026-07-03): trig,
    hyperbolic, log/exp families, `cbrt`/`hypot`/`rint`/`signum`/
    `toDegrees`/`toRadians`/`copySign`/`ulp`/`nextUp`/`nextDown`/
    `nextAfter`/`fma`/`IEEEremainder`/`getExponent`, `floorDiv`/
    `floorMod` with Java's negative semantics, the `xxxExact` family
    throwing `ArithmeticException: integer overflow` (`absExact` is Java
    15 and so is absent), `multiplyHigh`/`multiplyFull` (2026-07-09,
    the whole 64-bit product `int * int` would wrap), `scalb` for both
    `double` and `float` (2026-07-09 — scaled in stages, as the JDK
    does, so a result that underflows into the subnormals is rounded
    once rather than twice; an `int` argument picks the more specific
    `float` overload, in caturra as in javac), plus `PI`/`E`
    (`round` returns int, not long — the classroom idiom is
    `(int) Math.round(x)` anyway; transcendentals may differ from a
    given JVM by 1 ulp on irrational results, as JVMs differ among
    themselves). `random()` uses Java's LCG,
    seeded deterministically by default (tests) and from host entropy in
    the browser (`VmOptions::random_seed`).
  - `java.util.Random` is Java's exact 48-bit LCG, so a seed replays the
    JVM's sequence: `new Random(42).nextInt()` is `-1170105035` here as
    it is there, and two `Random`s with the same seed agree. `setSeed`,
    `nextInt`/`nextInt(bound)` (both the power-of-two and rejection
    branches), `nextLong`, `nextDouble`, `nextFloat`, `nextBoolean`,
    `nextBytes` (2026-07-09 — four bytes per draw, low byte first, so
    the generator lands where Java's does) and `nextGaussian` (the
    polar method, caching its second value) all match, pinned by
    `diff_random_seeded_sequences` and `diff_random_next_bytes` against
    a real JDK.
    `nextGaussian` may differ in the last ulp because it goes through
    `Math.log` (above). An unseeded `new Random()` draws its seed from
    `Math.random()`, so it is reproducible in tests and entropic in the
    browser, rather than following the JVM's uniquifier. Java 17's two-arg
    `nextInt(origin, bound)` is deliberately absent: caturra models the
    Java 11 surface, and offering a later API would let code compile here
    that a real JDK 11 rejects.
  - The full Java 11 int/double surfaces of `Integer` (radix parsing
    and formatting, the bit-twiddling family, the unsigned family,
    `compare`/`min`/`max`/`sum`/`signum`/`hashCode`, `SIZE`/`BYTES`),
    `Double` (all constants incl. infinities and `NaN`,
    `isNaN`/`isInfinite`/`isFinite`, `compare` with Java's `-0.0 < 0.0`
    and NaN-greatest ordering, bit-exact `hashCode`, `toHexString`),
    `Character` (classification and conversion families with Java's
    own `isWhitespace`-vs-`isSpaceChar` distinction, `digit`/`forDigit`/
    `getNumericValue`, surrogates, radix constants), and `Boolean`
    (including the 1231/1237 hash codes). `valueOf` returns primitive
    values (no boxed identity or caching — a documented deviation).
    Long/float/byte-dependent members report honest "the long type is
    not supported" style errors, with
    `NumberFormatException: For input string: "x"`.
  - `Scanner` over `System.in`, a `File`, **or a `String`** (`new
    Scanner("10 20 hi")`, 2026-07-20 — the JDK constructor that tokenizes a
    literal source, fully present so `hasNextLine` is exact) — `next`/`nextLine`,
    the full numeric set
    (`nextInt`/`nextLong`/`nextShort`/`nextByte`/`nextFloat`/`nextDouble`/
    `nextBoolean`), every matching `hasNextX`, and `close` — tokenizing
    like Java, fed by the host console; in the browser this is the
    SharedArrayBuffer blocking-stdin path. `nextLine` excludes the line
    terminator and matches `\r\n` (or a lone `\r`) as one, so a CRLF source
    yields `a`, not `a\r` (2026-07-20 — the buffer path returned the carriage
    return; fixed for the file and string sources alike). A failed `nextX` throws
    `InputMismatchException` and **does not consume the token**, as the
    JDK documents, so `catch (InputMismatchException e) { in.next(); }`
    skips the offending word rather than the one after it. The float
    grammar is Java's, not Rust's: `[-+]?(NaN|Infinity)` exactly, so
    `nan`, `inf`, `infinity`, `1.5f` and `0x10` are not numbers to
    `hasNextDouble`. **`close()` closes the underlying stream** (2026-07-09,
    it used to be a no-op): the closed `Scanner` throws
    `IllegalStateException: Scanner closed` from every method but `close`,
    which is idempotent — and closing a `System.in` scanner closes standard
    input, so a _later_ `new Scanner(System.in)` reads nothing and throws
    `NoSuchElementException`. That is the JDK's behaviour and a classic
    student trap; a no-op `close` let such a program work in the playground
    and die on a real JVM. A `Scanner` over a file closes only itself.
    Every `hasNextX` classification, the mismatch
    behaviour and `close` are pinned against a real JDK by the differential
    suite (`diff_scanner_close_closes_standard_in`).
    `Pattern`/`Matcher` and the stream members report honest reasons
    (the regex ENGINE exists, but caturra compiles patterns inside the
    `String` methods rather than exposing the objects).
  - `ArrayList<E>` with the CSA generics surface: wrapper/String/class
    element types, the diamond, and the full Java 11 method set
    (2026-07-03): `size/add/get/set/remove` (by index and by value),
    `clear/contains/indexOf/lastIndexOf/addAll` (both)/`equals`/
    `hashCode` (Java's 31-fold)/`toString/isEmpty`, plus the capacity
    hints as no-ops; `contains`/`indexOf`/`lastIndexOf`/`remove(Object)`/
    `equals`/`hashCode` compare elements with **their own `equals` and
    `hashCode`** (2026-07-09), overrides included, asking the probe as
    Java does. So do `containsAll`/`removeAll`/`retainAll`
    (2026-07-09), which ask the side Java asks — `containsAll` the
    _other_ collection's element, `removeAll` and `retainAll` this
    list's — and return whether the list changed. Their argument must be
    a list of the same element type, where javac takes any
    `Collection<?>`: stricter, so anything compiling here compiles on a
    JDK. **`forEach(x -> ...)`** (a `Consumer<E>`) and **`removeIf(x ->
...)`** (a `Predicate<E>`) run their lambda over the elements
    (2026-07-09), the element-typed single-parameter counterpart of
    `Map.forEach`; `removeIf` reports whether any element went.
    **`replaceAll(x -> ...)`** (a `UnaryOperator<E>`) transforms each element
    in place, and **`sort((a, b) -> ...)`** (a `Comparator<E>`) runs a stable
    sort — as does `Collections.sort(list, (a, b) -> ...)`, which previously
    compiled and silently ignored the comparator. Iterators, streams,
    `toArray`, and `subList` still report honest reasons. Autoboxing (a no-op in this VM — boxed-equality caching
    semantics are not modeled), `println(list)` printing `[a, b]`,
    for-each, and `IndexOutOfBoundsException` in Java 11's wording.
    Nested generics (`ArrayList<ArrayList<...>>`) are rejected kindly.
  - Compound assignment unboxes a wrapper on either side and boxes the
    result back (JLS §15.26.2, fixed 2026-07-09), so `total +=
map.get(key)` and `Integer count; count += 1;` behave as javac
    compiles them — the latter used to leave a raw `int` in a reference
    slot and fail only when something later read it. The implicit
    narrowing cast still applies (`char c; c += anInteger;`).
  - A wrapper **auto-unboxes in every context that demands a primitive**
    (JLS §5.1.8, completed 2026-07-20), not only in assignments and
    arithmetic: an `if`/`while`/`for`/ternary CONDITION on a `Boolean`, a
    unary `-`/`~` on a numeric wrapper or `!` on a `Boolean`, a wrapper as an
    array INDEX (`arr[anInteger]`, also `Character`/`Short`/`Byte`), and
    `++`/`--` on a wrapper local or field. The increment reboxes through
    `valueOf` (JLS §15.14.2) — postfix yields the old wrapper, prefix the new
    — and a `null` wrapper in any of these throws `NullPointerException` on
    the unbox. Each of these was previously a spurious compile error
    ("cannot be converted to boolean/int", "++/-- needs a numeric variable").
  - `HashMap<K, V>` / `Map<K, V>` (2026-07-09), **with the JDK's own
    iteration order**. A real map's order looks arbitrary but is a pure
    function of the keys' hash codes, the table length, and insertion
    order, and students see it whenever they print a map — so caturra
    reproduces it (bucket index `(n - 1) & (h ^ h >>> 16)`, insertion
    order within a bucket, the table doubling past a 0.75 load factor,
    and `new HashMap<>(capacity)` / `new HashMap<>(map)` sizing their
    tables as Java does). The one deviation left: at a table of 64 or
    more, a bin of 8+ colliding keys really does become a red-black
    tree, which iterates in tree order rather than chain order; caturra
    keeps the chain order. Reaching it takes deliberately-crafted keys.
    Methods: `size/isEmpty/containsKey/containsValue/get/getOrDefault/
put/putIfAbsent/remove` (by key, and by key+value)/`replace` (both)/
    `clear/putAll/equals/hashCode/toString`, and the three views
    `keySet()` -> `Set<K>`, `values()` -> `Collection<V>`, `entrySet()`
    -> `Set<Map.Entry<K, V>>`. The views are live, as Java's are: a
    later `put` shows through, and `entry.setValue(...)` writes back.
    for-each walks all three (an index loop over a synthetic accessor
    rather than a real iterator) and is **fail-fast** since 2026-07-18:
    mutating a map inside such a loop throws
    `ConcurrentModificationException` exactly where a JDK does. **`keySet().forEach(k -> ...)`** and
    **`values().forEach(v -> ...)`** (2026-07-09), and the same on a
    `Set`/`Collection` variable holding a view, run a `Consumer` over the
    view's elements in the map's iteration order; the lambda's parameter is
    typed from the map's key or value type. `entrySet().forEach` (a
    `Map.Entry` parameter) is not yet target-typed — a compile error, the
    safe direction. Keys hash and compare with
    **their own `hashCode` and `equals`** (2026-07-09), overrides
    included — so a map's iteration order follows a user's `hashCode`,
    and a class that overrides `equals` without `hashCode` loses its
    keys, exactly as on a real JVM. A bin of 8 in a table shorter than
    64 makes Java resize rather than treeify, reshuffling every bucket;
    that is modelled too. `Double.equals` compares raw bits, so `-0.0`
    and `0.0` are distinct keys and `NaN` equals itself (`==` says the
    opposite of both); `Boolean` hashes to 1231/1237; a `null` key is
    legal and lands in bucket 0.
    Unlike `ArrayList`'s elements, a map's keys and values are **boxed**
    at the boundary: `map.put(k, v);` as a statement must not throw on a
    new key while `int old = map.put(k, v);` must, and only a boxed
    return models both. So `map.get(missing)` is `null`, `map.get(k) ==
null` is the way to test for absence, and unboxing an absent value
    throws `NullPointerException` exactly where Java does. `get`,
    `containsKey` and `remove` are typed to `K` rather than Java's
    `Object`, which only rejects programs a JDK would accept (never the
    reverse). **`forEach((k, v) -> ...)`** works (2026-07-09): the lambda's
    parameter types come from the receiver's declared type arguments — no
    other target type in caturra is instantiated from its receiver — and
    the VM walks the entries in the map's own iteration order. A receiver
    with no declaration to read (`getMap().forEach(...)`) is refused, as
    is `Map.of`; both report honest reasons. `merge`/`compute*`/
    `replaceAll` work — with method references, no-import injection, and
    the compute family's bucket-HEAD insertion position (2026-07-30). `keySet`/`values`/`entrySet` and the core methods are
    pinned against a real JDK by `diff_hash_map_iteration_order`,
    `_core_methods`, `_null_and_unboxing` and `_views`.
  - `HashSet<E>` / `Set<E>` (2026-07-10), **with the JDK's own iteration
    order** — which comes for free, because a real `HashSet` is backed by a
    `HashMap` whose keys are the elements, and caturra reuses exactly that
    bucket-order machinery. `new HashSet<>()`, `new HashSet<>(capacity)`, and
    `new HashSet<>(collection)` (which deduplicates and pre-sizes its table to
    `max((int)(c.size()/.75f)+1, 16)`, as Java's does, so a large source
    lands on a bigger table and iterates accordingly). Methods:
    `add/remove/contains/size/isEmpty/clear/addAll/removeAll/retainAll/
containsAll/forEach/equals/hashCode/toString`, and for-each (the same
    synthetic index-loop accessor the map views use). Elements hash and
    compare with **their own `hashCode`/`equals`**, so a user type governs
    both membership and order exactly as on a JVM; `equals`/`hashCode` are
    `AbstractSet`'s (order-independent). `Set<E>` is the mutable interface
    for both a standalone `HashSet` and a map's `keySet()`: at runtime a
    `keySet()` view throws `UnsupportedOperationException` on `add` (Java's
    behaviour — accepting it silently would be the dangerous direction) and
    writes through to its map on `remove`/`clear`, while a real `HashSet`
    performs them. `removeIf(x -> ...)` drops the matching elements (2026-07-11);
    `iterator`/`stream` on a `Set` report honest reasons. Pinned against a real
    JDK by `diff_hash_set_core`, `_integer_order`, `_bulk_ops` and
    `_keyset_bridge`.
  - `TreeSet<E>` / `SortedSet<E>` / `NavigableSet<E>` (2026-07-10) — a **sorted**
    set. Its elements are kept in an ordered vector, so iteration, `first`/
    `last`, and the navigation methods read straight off it. Ordering is the
    elements' **natural (`Comparable`) ordering**, a user `compareTo` included
    — which also decides equality, so `compareTo == 0` deduplicates (two
    people of the same age collapse to one, exactly as on a JVM). Methods:
    `add`/`remove`/`contains`/`size`/`isEmpty`/`clear`/`addAll`/`containsAll`/
    `forEach`, `first`/`last` (throw `NoSuchElementException` when empty),
    `floor`(≤)/`ceiling`(≥)/`lower`(<)/`higher`(>) and `pollFirst`/`pollLast`
    (all boxed, `null`/empty-safe), and for-each in sorted order. A `TreeSet`
    widens to `Set`/`Collection`, and `new TreeSet<>(collection)` copies and
    sorts. `new TreeSet<>(comparator)` orders by a `Comparator` (a class or a
    lambda — see below) instead of the natural ordering. `removeIf(x -> ...)`
    drops the matching elements (2026-07-11); `iterator`/`stream` and the range
    views (`headSet`/`tailSet`/`subSet`/`descendingSet`) report honest reasons.
    Pinned against a real JDK by `diff_tree_set_core`, `_strings_and_copy` and
    `_user_comparable`.
  - `TreeMap<K, V>` / `SortedMap<K, V>` / `NavigableMap<K, V>` (2026-07-10) — a
    **sorted map**, the map analogue of `TreeSet`. Entries are kept in an
    ordered vector by key, so iteration, the three views, and the key
    navigation all read straight off it — and it flows through the same code
    as `HashMap` (`put`/`get`/`containsKey`/`remove`/`keySet`/`values`/
    `entrySet`/`forEach`/…), because the shared map helpers route a TreeMap to
    comparison-based key lookup and its sorted vector. Keys order by their
    natural (`Comparable`) ordering, a user `compareTo` included, which decides
    key identity too (`compareTo == 0` replaces rather than adds). It adds
    `firstKey`/`lastKey` (throw when empty) and `floorKey`/`ceilingKey`/
    `lowerKey`/`higherKey` (boxed, `null` when absent). `new TreeMap<>(map)`
    copies and re-sorts; a `TreeMap` widens to `Map`. `new TreeMap<>(comparator)`
    orders keys by a `Comparator` (see below) rather than naturally.
    The entry-view and range methods (`firstEntry`/`headMap`/`tailMap`/
    `subMap`/`descendingMap`/…) report honest reasons. Pinned against a real
    JDK by `diff_tree_map_core`, `_int_keys_and_views` and
    `_user_comparable_keys`.
  - `java.util.Comparator<T>` (2026-07-11) — students can now write their own,
    as a **class** (`class ByAge implements Comparator<Person> { public int
compare(Person a, Person b) {...} }`), a **variable** (`Comparator<Person>
c = ...`), or a **lambda** (`(a, b) -> a.age - b.age`), and use it to order
    `list.sort(cmp)`, `Collections.sort(list, cmp)`, `new TreeSet<>(cmp)` and
    `new TreeMap<>(cmp)`. It is a student-facing alias for the erased
    `__Comparator` caturra already used internally for sort lambdas: `Comparator`
    resolves to it, `implements Comparator<T>` implements it, and the SAM
    `compare(Object, Object)` reaches a user `compare(T, T)` through the same
    erasure bridge as `Comparable.compareTo` (matched by name and arity). A
    comparator lambda casts both parameters back to the element type, read from
    the `Comparator<E>` target or the `TreeSet<E>`/`TreeMap<K, V>` being
    constructed. The static factories and combinators are also modelled
    (2026-07-11): `Comparator.naturalOrder()`/`reverseOrder()` order
    `Comparable`s; `Comparator.comparing`/`comparingInt`/`comparingDouble`/
    `comparingLong(keyExtractor)` build a key-extractor comparator; and
    `.reversed()`/`.thenComparing(other)`/`.thenComparing(keyExtractor)` chain
    them. The key extractor must be a **method reference** (`Person::getAge`,
    `String::length`) — its qualifier class types the extracted key — because a
    bare `Comparator.comparingInt(p -> p.age)` has no receiver to flow the
    element type from and is an honest compile error (write the method reference,
    or a full `(a, b) -> …` comparator lambda, instead). Pinned against a real
    JDK by `diff_comparator_classes`, `_lambdas`, and `_factories`.
  - `PriorityQueue<E>` (2026-07-11) — a real **binary min-heap**, so `peek`/
    `poll`/`element`/`remove()` return the _least_ element, but `toString`,
    for-each, and the iterator show the **heap-array order, not sorted** — and
    that array is replicated exactly (Java's `siftUp`/`siftDown`/`heapify`) so
    both match a real JVM byte for byte, the same fidelity as `HashMap`'s
    iteration order. Ordering is natural (`Comparable`) or a `Comparator` (a
    class or a lambda): `new PriorityQueue<>()`, `(int capacity)`,
    `(Comparator)`, `(int, Comparator)`, and `(Collection)` (which `heapify`s a
    plain collection, or copies a `PriorityQueue`/`TreeSet` whose order is
    already a valid heap). `add`/`offer`/`poll`/`peek`/`element`/`remove()`/
    `remove(Object)`/`contains`/`size`/`isEmpty`/`clear`/`forEach`, and
    for-each; `contains` and `remove(Object)` compare by `equals`, not the
    ordering, exactly as Java's do. It **is** a `Queue`, so it reuses that
    interface (`Queue<E> q = new PriorityQueue<>()`), differing only in the
    heap object behind it. Pinned against a real JDK by
    `diff_priority_queue_core` and `_comparator_and_heapify`.
  - `java.util.Stack<E>` (2026-07-11) — the `Vector`-backed LIFO. `push`/`pop`/
    `peek` act on the **top** (the end of the list, unlike a `Deque`'s
    head-based `push`/`pop`); `empty()` mirrors `isEmpty()`; `search(o)` is the
    1-based distance from the top (via `lastIndexOf`, so a duplicate reports the
    position nearest the top), or `-1`. An empty `pop()`/`peek()` throws
    `EmptyStackException` (not the `NoSuchElementException` a `Deque` throws).
    Because a `Stack` **is** a `List` (it extends `Vector`), every list method —
    `get`/`set`/`add(i, e)`/`remove(i)`/`indexOf`/`contains`/`size`/`sort`/
    `forEach`/… — and for-each work on it, and it widens to `List<E>` and
    `Collection<E>` (`List<E> l = new Stack<>()`). Only the no-argument
    constructor is offered, matching javac (Stack declares no copy constructor of
    its own). Pinned against a real JDK by `diff_stack_lifo_and_list`.
  - `java.util.stream.Stream<E>` (2026-07-11; made **LAZY** 2026-07-19), the
    `collection.stream()` pipeline. A stream holds its source plus the pending
    intermediate ops; nothing runs until a terminal PULLS elements one at a
    time through the op chain. So side effects INTERLEAVE
    (`peek(a); filter(a); out(a); peek(bb); …`, not staged), and a
    short-circuit terminal (`findFirst`/`anyMatch`/`allMatch`/`noneMatch`, or a
    downstream `limit`) stops the source early — a pipeline whose upstream would
    throw for a skipped element now completes exactly as a JDK does, where the
    eager model crashed. `sorted` is a barrier: it materializes the upstream
    (running its side effects in order) and re-sources. Intermediate:
    `filter`/`map`/`sorted`/`sorted(cmp)`/`distinct`/`limit`/`skip`/`peek`.
    Terminal: `collect(Collectors.toList()/toSet()/joining(...))`,
    `forEach`/`forEachOrdered`, `count`, `anyMatch`/`allMatch`/`noneMatch`. A
    lambda's parameter type flows **syntactically** from the source collection
    through the element-preserving ops (`list.stream().filter(p -> p.age > 18)`
    types `p` from the list) — the lambda is desugared before types are known,
    so the one gap is a lambda **after** a `map`, whose element is erased to
    `Object` (fine when it is only printed or matched; an element-specific use
    is an honest error). `collect` reads its result type from the collector:
    `joining()` is a `String`; `toList()`/`toSet()` a `List`/`Set` of the
    element, or — once `map` has erased it — a `null` that adopts the
    assignment context (`List<R> r = ...map(...).collect(toList())` works; an
    inline bare-iteration over such a result does not, the one documented
    limitation). Pinned against a real JDK by `diff_stream_pipeline` and
    `_joining_limit_skip`.
  - `java.util.stream.IntStream` (2026-07-11) — a primitive `int` stream,
    modelled as a `Stream` of unboxed ints. Sources: `collection.stream().mapToInt(e -> ...)`
    and `IntStream.range(a, b)` / `rangeClosed(a, b)`. Intermediate:
    `map`/`filter`/`sorted`/`distinct`/`limit`/`skip` (their lambdas take a
    single `int`), `mapToObj(i -> ...)` → `Stream<R>`, `boxed()` →
    `Stream<Integer>`. Terminal: **`sum()`**, `count()`, `toArray()` → `int[]`,
    `forEach`, `anyMatch`/`allMatch`/`noneMatch`. So the ubiquitous
    `list.stream().mapToInt(x -> x).sum()` and `IntStream.range(0, n).forEach(...)`
    both work. Pinned against a real JDK by `diff_int_stream`.
  - `java.util.Optional<E>` / `OptionalInt` / `OptionalDouble` (2026-07-11) — the
    present-or-absent results of the stream terminals: `Stream.findFirst()`/
    `findAny()` and `max(cmp)`/`min(cmp)` give an `Optional<E>`; `IntStream.max()`/
    `min()` an `OptionalInt`; `IntStream.average()` an `OptionalDouble`. Methods:
    `isPresent`/`isEmpty`, `get`/`getAsInt`/`getAsDouble` and `orElseThrow`
    (throw `NoSuchElementException` when absent), `orElse(default)`,
    `ifPresent(consumer)`. `toString` matches Java exactly — `Optional[x]`,
    `Optional.empty`, `OptionalInt[9]`, `OptionalDouble.empty`. This closes the
    `average`/`min`/`max`/`findFirst` gap the streams left. An Optional can also
    be **constructed directly** (2026-07-11): `Optional.of(x)` and
    `Optional.ofNullable(x)` build a present Optional of the argument's type,
    `Optional.empty()` an absent one that adopts its assignment context (like
    `null`), so a method can return `Optional<T>`. Two lambda methods work too
    (2026-07-11): `ifPresent(x -> ...)` runs its consumer only when present, and
    `filter(x -> ...)` keeps a present value only if the predicate matches (else
    an empty Optional); `map(x -> ...)` transforms a present value (its result
    element erased to `Object`, like a stream's `map`, so a function returning
    `null` yields an empty Optional); and `orElseGet(() -> ...)` computes a
    fallback only when absent (a zero-argument supplier). Their lambda parameter
    is typed by the Optional's element, resolved from a variable/field receiver
    — a chained `opt.filter(..).filter(..)` on a call receiver remains
    unsupported. Pinned against a real JDK by `diff_optional`,
    `diff_optional_factories`, `diff_optional_lambdas`, and `diff_optional_map`.
  - `LinkedList<E>`, and the `Queue<E>`/`Deque<E>` interfaces it implements
    (2026-07-10). The storage is the same ordered-element vector an
    `ArrayList` uses — this VM models no node links or their cost — kept a
    distinct type only so `getClass()` stays honest and so each interface face
    exposes the right methods. As a **`List`** it has the full list surface
    (`get`/`set`/`add(i,e)`/`remove(i)`/`indexOf`/…); as a **`Queue`**,
    `offer`/`poll`/`peek`/`element`; as a **`Deque`**, those plus
    `push`/`pop`/`addFirst`/`addLast`/`offerFirst`/`offerLast`/`removeFirst`/
    `removeLast`/`pollFirst`/`pollLast`/`getFirst`/`getLast`/`peekFirst`/
    `peekLast`. The **interface typing restricts the method set exactly as
    javac does**: `Queue<E> q = new LinkedList<>()` cannot call `q.get(0)`
    (a `Queue` is not a `List`), and accepting it would be the dangerous
    direction. The nullable ends match Java: `poll`/`peek` (and the
    `*First`/`*Last` polls/peeks) return the **boxed** element so an empty
    collection yields `null`, while `remove()`/`element()`/`getFirst()` throw
    `NoSuchElementException`. `new LinkedList<>(c)` copies any collection in
    order; a `LinkedList` widens to `List`/`Collection` (and a `Deque` to
    `Queue`), and for-each walks it by index. Pinned against a real JDK by
    `diff_linked_list_queue`, `_deque` and `_as_list`.
  - `ArrayDeque<E>` (2026-07-11) — the array-backed `Deque`/`Queue` (**not** a
    `List`). It shares the `LinkedList` Deque semantics — head-based `push`/
    `pop`/`peekFirst`/`pollFirst`, tail-based `offer`/`addLast`/`peekLast` — so
    it is used interchangeably as a **LIFO stack** (`push`/`pop`) or a **FIFO
    queue** (`offer`/`poll`), and `toString`/for-each read `[head, …, tail]`.
    Two things keep it a distinct type from `LinkedList`: it **forbids null
    elements** (every insertion throws `NullPointerException`, matching the JDK
    and preserving the one-directional rule the null-tolerant `LinkedList`
    could not), and `getClass()` reports `java.util.ArrayDeque`. Constructors:
    `new ArrayDeque<>()`, `(int numElements)` (a capacity hint), and
    `(Collection)` (copies in order, rejecting null). It widens to `Deque<E>`
    and `Queue<E>` but is unrelated to the concrete `LinkedList` (neither
    assigns to the other, exactly as javac enforces). Pinned against a real JDK
    by `diff_array_deque`.
  - **`toString` is honoured wherever a value becomes text** (2026-07-09).
    A container renders its elements by calling their `toString()`, as
    `AbstractCollection` and `AbstractMap` do — `println(list)`,
    `"" + map`, `map.values()`, `sb.append(obj)` and `%s` all agree with
    the JDK. This needs an intrinsic to call back into Java, so the
    coercion points live in the interpreter (which can run a nested
    frame stack) rather than the intrinsic layer (which sees only the
    heap). Faithful to the corners: a collection holding itself renders
    as `(this Collection)` rather than recursing; a `toString()` that
    returns `null` renders as `"null"`; one that throws propagates to
    the caller's `catch`, with the calling frames in the stack trace;
    and a cycle between two collections, or a `toString()` that renders
    itself, throws `StackOverflowError` instead of exhausting the host
    stack. A class that declares no `toString` keeps Java's default
    `Class@hash`. Pinned against a real JDK by
    `diff_to_string_inside_collections` and `diff_to_string_edge_cases`.
    One consequence for the debugger: pausing on a breakpoint inside a
    `toString()` that a container is rendering shows only that call's
    frames, because the frames beneath it are suspended.
  - `Collections.sort(list)` is a stable natural-ordering sort, and a
    user class sorts by its own `compareTo` (2026-07-09) — it reaches
    that through the same nested-call machinery `toString` uses. Before,
    it compared every pair of user objects as equal and so silently left
    the list alone. A list whose element type declares no `compareTo` is
    rejected at compile time (2026-07-09), as javac rejects it: `sort`,
    `max`, `min` and `binarySearch` are declared over
    `T extends Comparable<? super T>`. Pinned by
    `diff_collections_sort_uses_compare_to` and
    `reject_sorting_a_list_of_non_comparables`.
  - `Collections.reverse`/`swap`/`shuffle`/`max`/`min`/`frequency`/
    `nCopies` (2026-07-09). `reverse`, `swap` and `shuffle` are bundled
    Java; `shuffle(list, random)` is Java's own Fisher-Yates over
    caturra's exact `Random`, so a seeded one replays the JDK's
    permutation. The VM answers `max`/`min`/`frequency`/`nCopies`,
    because a list stores unboxed primitives (a bundled version could
    not compare them) and `max`/`min` must hand back the element's own
    type. `max`/`min` compare with the element's `compareTo`, keep the
    first of equal elements, throw `NoSuchElementException` on an empty
    collection and `NullPointerException` on a null element (a lone
    element is never compared, so it does not); `frequency` asks the
    probe's own `equals`; `nCopies` throws
    `IllegalArgumentException` on a negative count and returns a
    **mutable** `ArrayList`, where Java's is immutable — the same
    deviation `Arrays.asList` has. `frequency(list, wrongType)` is
    rejected at compile time, where javac takes its `Object` parameter
    and answers 0; and `shuffle`'s second argument must be a `Random`.
    The `Comparator` overloads are absent. Pinned by
    `diff_collections_helpers` and `_errors`.
  - `Collections.binarySearch`/`addAll`/`unmodifiableList`/`emptyList`
    (2026-07-09). `binarySearch` compares with the element's own
    `compareTo`; `addAll(list, e1, e2, ...)` is variadic and returns
    whether the list changed — a lone `T[]` is the varargs array, but
    only for a reference `T`, since javac has no `Integer[]`/`int[]`
    conflation and rejects `addAll(List<Integer>, int[])`.
    `unmodifiableList` is a real **view**, as Java's is: a later change
    to the backing list shows through, and every mutator throws
    `UnsupportedOperationException`, whether reached through the list
    (`add`, `set`, `remove`, `clear`) or through `Collections`
    (`sort`, `reverse`, `shuffle`, `swap`, `addAll`). `emptyList()` is
    an unmodifiable empty view; it types as `null` does — assignable to
    any `List<T>` — because caturra has no target typing, so
    `System.out.println(Collections.emptyList())` is a compile error
    where javac infers `List<Object>`. Pinned by
    `diff_collections_binary_search_and_add_all` and
    `_unmodifiable_and_empty`.
  - `Collections.singletonList`/`reverseOrder` (2026-07-11). `singletonList(e)`
    is an immutable one-element list (an `UnmodifiableList` view, so `add`
    throws `UnsupportedOperationException`) of the argument's type.
    `reverseOrder()` reverses natural ordering and `reverseOrder(cmp)` reverses
    a given comparator — the same reversed `__Comparator` value
    `Comparator.reverseOrder()`/`reversed()` produce, so it orders a
    `list.sort(...)` or a `new TreeSet<>(...)`. Pinned by
    `diff_collections_singleton_and_reverse_order`.
  - `Collections.emptySet`/`emptyMap`/`singleton`/`singletonMap`/
    `unmodifiableSet`/`unmodifiableMap` (2026-07-11) — the **immutable Set and
    Map** family, mirroring the list ones. `emptySet()`/`emptyMap()` are empty
    (typed like `null`, adopting context); `singleton(e)`/`singletonMap(k, v)`
    hold one element/entry; `unmodifiableSet`/`unmodifiableMap` view an existing
    collection. All are real views (`UnmodifiableSet`/`UnmodifiableMap` wrapping
    a backing `HashSet`/`TreeSet`/`HashMap`/`TreeMap`): reads and iteration
    delegate to the backing (so an `unmodifiableSet(treeSet)` still prints
    sorted), and **every mutator throws `UnsupportedOperationException`** — as do
    the `keySet()`/`values()`/`entrySet()` of an unmodifiable map, which are
    themselves unmodifiable, so a `keySet().remove(k)` cannot write through.
    Pinned by `diff_immutable_set_and_map`.
  - `java.util.Arrays` is bundled Java rather than a native intrinsic,
    so every element operation dispatches. `toString` renders elements
    through their own `toString`; `equals` and `hashCode` (2026-07-09,
    nine overloads each) ask each element's own `equals`/`hashCode`,
    null-safely, so two arrays of user objects compare by value; and
    `sort` of a reference array (2026-07-09) orders elements by their
    `compareTo`, stably; a `null` element throws, since the comparison
    calls `compareTo` (so does `Collections.sort`).
    `Arrays.equals(double[], double[])` compares
    raw bits, so `NaN` equals itself and `-0.0` does not equal `0.0` —
    `==` says the opposite of both. An element that is itself an array
    compares by identity, as Java's does. `Arrays.sort` of an array
    whose element type is not `Comparable` is rejected at compile time,
    where javac accepts it and throws `ClassCastException` at run time —
    stricter, so anything compiling here still compiles on a JDK.
    `asList` returns a mutable `ArrayList` (a documented deviation) and
    keeps its elements unboxed, so `Arrays.asList(1, 2).get(0)` reads
    back as an `int` — it used to box them and fail at run time. Pinned
    against a real JDK by `diff_arrays_equals_and_hash_code` and
    `diff_arrays_sort_uses_compare_to`.
  - `Arrays.copyOf`/`copyOfRange`/`fill`/`binarySearch` (2026-07-09),
    every Java 11 overload except the `Comparator` and `Class` ones.
    The VM answers these too — 50 Java overloads are one method each
    here, because the heap already knows an array's element kind, which
    `copyOf` must reproduce. Arity tells the ranged forms apart, as in
    Java. `copyOf`/`copyOfRange` return an array of the source's own
    type (so `String[] s = Arrays.copyOf(strings, 3)` type-checks) and
    pad with the element default. `binarySearch` compares a reference
    element with its own `compareTo`, so a class without one throws
    `ClassCastException` and a `null` key throws
    `NullPointerException`; `-0.0` sorts below `0.0` and `NaN` above
    everything, as `Double.compare` orders them. There is no
    `binarySearch(boolean[], boolean)` — booleans have no order — and
    caturra rejects it as javac does. Java's bounds checks are exact:
    `NegativeArraySizeException`, `IllegalArgumentException` for
    `from > to`, `ArrayIndexOutOfBoundsException` past the end.
    `Arrays.fill(String[], 5)` is rejected at compile time, where javac
    accepts it (its erased `fill(Object[], Object)`) and throws
    `ArrayStoreException` — stricter, the safe direction. Pinned by
    `diff_arrays_copy_fill_and_binary_search` and `_errors`.
  - `Arrays.setAll(array, i -> ...)` (2026-07-11) — fills each slot from a
    generator of its index, for `int[]`, `double[]`, and object arrays. The
    generator's parameter is the `int` index and its result the array's element
    type (resolved from the array variable's declared type); it runs in index
    order, so a generator may read already-filled slots (`Arrays.setAll(fib, i
-> i < 2 ? i : fib[i-1] + fib[i-2])`). Pinned by `diff_arrays_set_all`.
  - `Arrays.deepToString`/`deepEquals`/`deepHashCode` (2026-07-09),
    for 2D arrays. These three the VM answers rather than the bundled
    Java, because only it can see an element array's kind once the
    static type is gone — so `deepToString(boolean[][])` prints `true`
    where `int[][]` prints `1`, and `deepHashCode` folds 1231/1237
    rather than the value. Elements that are not arrays go through
    their own `toString`/`equals`/`hashCode`. `deepEquals` compares
    rows element-wise, where plain `equals` compares them by identity
    and so reports `false` for equal-but-distinct rows — as Java's
    does. A `boolean[]` never equals an `int[]` of the same shape.
    `deepToString` of an array holding itself renders `[...]` rather
    than recursing; `deepEquals` and `deepHashCode` have no such guard
    and throw `StackOverflowError`, exactly as Java's do. A
    multi-dimensional array widens to `Object[]` (its rows are
    references), so `Object[] rows = grid;` and
    `Arrays.equals(int[][], int[][])` compile; `int[]` does not.
    Pinned by `diff_arrays_deep_operations`.
  - `System.arraycopy` and `System.lineSeparator` (2026-07-09).
    `arraycopy` copies between arrays of every element kind, checks the
    component types exactly (a `boolean[]` never copies into an
    `int[]`, though both hold their elements as 32-bit words), and a
    copy within one array behaves as if it went through a temporary, as
    Java's does. Its parameters are `Object`, as javac's are; every
    check the JVM makes — not-an-array, kind mismatch, the four bounds
    cases, and the per-element check of a reference copy — throws with
    the JVM's own message (2026-07-30, see the round-7 entry above).
    A null array is a `NullPointerException`. `lineSeparator()` is always `"\n"`: the
    JVM's is system-dependent, and caturra runs where a line ends with a
    newline. Pinned by `diff_system_arraycopy_and_line_separator`.
  - A real Java 11 class caturra does not model (`Stack`, `ArrayDeque`,
    `Iterator`, `Optional`, `Hashtable`, `BufferedReader`, ...)
    reports an honest "java.util.Stack is not supported by caturra"
    **wherever it is written** (2026-07-09): a local or field declaration,
    a parameter, an array element, a type argument, `new`, `extends` and
    `implements`, qualified or not. Until then only `import` and `new`
    said so, and a declaration said "this type cannot be used for a
    variable" — which reads as a typo for a class the student can see in
    the documentation. The reason is only consulted once the name has
    failed to resolve, so a user class named `Stack` still shadows the
    library one, and a genuine typo still gets `unknown type 'Frobnicator'`
    (blaming the type argument, not the `ArrayList` around it). javac
    accepts all of these, so this is deliberate strictness, pinned by
    `strict_stack_is_refused_by_name` and
    `unmodeled_library_classes_explain_themselves_in_every_position`.
  - `import` statements are real (2026-07-03): declarations are
    validated (unknown class in a known package / unknown package get
    javac's wording; real-but-unmodeled Java classes and packages get
    an honest "not supported by caturra" instead), and using `Scanner`,
    `ArrayList`, `HashMap`, `Map`, `TreeMap`, `Set`, `HashSet`, `TreeSet`,
    `LinkedList`, `Queue`, `Deque`, `PriorityQueue`, `Collection`, `File`, or
    `PrintWriter` without the matching import
    (or a `java.util.*` / `java.io.*` wildcard) is javac's "cannot find
    symbol: class Scanner". `java.lang` is implicit; exception-class
    imports (`IOException`, ...) are accepted for `throws` clauses;
    user-defined classes shadow library names. Fully qualified names
    work without imports (2026-07-03): `java.util.Scanner` in type
    positions (declarations, `new`, generics, `throws`) and
    `java.lang.Math.abs(...)` / `java.lang.System.out.println(...)` /
    `java.lang.Integer.MAX_VALUE` in expressions, with Java's obscuring
    rule (a variable named `java` wins over the package). Every modeled
    class of `java.util`, `java.io` and `java.lang` resolves qualified,
    in both positions — until 2026-07-09 the resolver kept a second,
    hand-maintained class list that had drifted from the real one, so
    `java.util.Arrays.fill(...)` and `java.lang.StringBuilder` did not
    resolve while `java.lang.Math.abs(...)` did, and `java.util.Random`
    resolved only by falling through the `Outer.Inner` nested-class
    path. Unknown qualified names get the same javac/honest wording as
    imports, in expressions too: `java.util.Nope.f()` is "cannot find
    symbol: class Nope in package java.util" rather than a complaint
    about a missing variable named `java`. Pinned by
    `diff_fully_qualified_names_in_expression_position`.
    `package` declarations remain unsupported.
  - User classes shadow intrinsic names (a class called `Scanner` wins).

- **File IO over the virtual filesystem** (stage 8): `new File(path)`
  with `exists/isFile/isDirectory/delete/mkdir/createNewFile/getName/`
  `getPath` (`length()` returns int, not long — virtual files are
  small), `new Scanner(file)` slurping VFS content with
  `FileNotFoundException: path (No such file or directory)`,
  `new PrintWriter(path|file)` truncating on open and **writing
  through** (a kind deviation: output is durable even without
  `close()`). A `PrintWriter` supports `print`/`println`/`printf`, and
  (2026-07-11) `write(String)`/`write(int)` (the int is a single
  character, not its decimal), `append(char)`/`append(CharSequence)`
  and `format(...)` — `append` and `format` return the writer, so they
  chain (`out.append('[').append(name).append(']')`). Also `throws`
  clauses parsed and ignored (checked
  exceptions are not enforced). The host seeds and inspects files via
  the `JvmSession` VFS API, so JS ⇄ Java file exchange works.

- **Exceptions** (2026-07-03): `try` / multi-`catch` with JVMS
  exception tables, `throw`, and `new SomeException("message")` over the
  closed library hierarchy in `caturra-classfile::exceptions`
  (`Throwable` down through `Exception`/`Error`/`RuntimeException` to
  the concrete classes the runtime throws — `StackOverflowError` is
  catchable via `Error`, matching Java). Subtype catching, cross-frame
  unwinding, rethrow, `getMessage()`/`toString()`, javac's "exception X
  has already been caught" for masked clauses, JLS definite-assignment
  and completes-normally rules (a method whose try and catch both
  return has no missing-return error). `finally` works via javac's
  duplication strategy (2026-07-03): copies on the normal path, each
  catch, every `return`/`break`/`continue` that exits the try, and a
  catch-all rethrow handler; loop-depth tagging keeps `break` from
  running guards outside its loop, and copies are never covered by
  their own catch-all. User exception classes extend the library
  throwables (`class TooSmall extends Exception` with `super(message)`
  chaining, own fields and methods, inherited `getMessage`/`toString`);
  thrown user objects keep their identity through catch and rethrow.
  Uncaught behavior is unchanged:
  Java-formatted stderr with a line-numbered stack trace captured at
  the throw point.

- **Expressions & switch** (2026-07-03): the conditional operator
  `?:` (with numeric promotion and reference joining), `switch` over
  int/char/String (fall-through, stacked labels, default anywhere,
  `break` binding to the switch while `continue` skips past it to the
  enclosing loop; javac's "duplicate case label"; lowered to an
  evaluate-once compare chain — semantically identical to
  tableswitch), expression-position `++`/`--` on variables and array
  elements (JVMS dup/dup_x sequences; field targets remain
  statement-only for now), the bitwise and shift operator family
  `& | ^ ~ << >> >>>` with Java precedence, compound forms, shift-count
  masking, and non-short-circuit `& | ^` on booleans, plus hex
  (`0x1F`), binary (`0b1010`), octal (`0755`), and underscore
  (`1_000_000`) integer literals and `\uXXXX` escapes.

- **The `long` type** (2026-07-03): literals with `L` (decimal and
  hex/binary up to 64 bits) and `d`/`D` double suffixes, JLS numeric
  promotion (int op long → long, long op double → double), implicit
  int→long and long→double widening with javac's lossy-conversion
  errors the other way, explicit casts in every direction, the full
  arithmetic/bitwise/shift set (six-bit shift-count masking, `LCMP`
  comparisons, wrapping overflow), `long[]` arrays, `++`/`--`,
  compound-assignment narrowing (`int += long`), switch-selector
  rejection in javac's wording, `%d`/`%x` formatting, concat and
  println, the `Long` wrapper class, `Math`/`Double` long members
  (`toIntExact`, `multiplyHigh`, `doubleToLongBits` family — formerly
  honest-error stubs, now real), `Scanner.nextLong`, and
  `System.currentTimeMillis`/`nanoTime` via a host clock
  (`ConsoleIo::now_millis`; JS `Date.now()` in the browser).

- **The `float` type** (2026-07-03): `f`/`F` literals, JLS promotion
  (`long op float → float`, `float op double → double`), widening
  int/char/long→float→double with lossy-conversion errors otherwise,
  casts in every direction, f32-precision arithmetic (`0.1f + 0.2f`
  is `0.3`, not `0.30000000000000004`), `FCMPL`/`FCMPG` NaN
  comparisons, `float[]`, the `Float` wrapper class with the bits
  functions and Java's total-order `compare`, `Math` float overloads,
  `Scanner.nextFloat`, and `%f`-family formatting via `doubleValue()`
  widening. Rendering matches OpenJDK 11's FloatingDecimal — which is
  NOT shortest-round-trip (pre-Ryū): a clean-room implementation
  (`crates/caturra-vm/src/floatdec.rs`) validated against a 25k-value
  reference corpus (`tools/FloatCorpus.java`), byte-identical on
  99.94% including every value ordinary programs produce; the residue
  is exotic subnormal bit patterns, documented in the corpus test.

- **The `short` and `byte` types** (2026-07-03): stored as ints (the
  JVM way), with the full conversion discipline — JLS §5.2 constant
  narrowing (`byte b = 5;` compiles, `byte b = 200;` is javac's lossy
  error), `I2B`/`I2S` truncating casts from every numeric type,
  compound-assignment and `++`/`--` implicit narrow-back with
  wraparound (`byte b = 126; b++; b++;` is -128), arithmetic promoting
  to int, `byte[]`/`short[]` with true element semantics (`bastore`
  truncates; byte arrays are distinct from boolean arrays on the
  heap), switch selectors, `Byte`/`Short` wrapper classes with javac's
  out-of-range `NumberFormatException`, `Scanner.nextByte/nextShort`
  (range-checked `InputMismatchException`), and `%x` formatting that
  masks to the argument width (`%x` of `(byte) -1` is `ff`, not
  `ffffffff`).

- **Labeled `break`/`continue`** (2026-07-03): `label: statement`
  prefixes, with `break label;` jumping past the labeled loop, switch,
  or block, and `continue label;` continuing the named enclosing loop.
  The loop-target stack carries the source label; targeting searches
  it innermost-out. javac's exact diagnostics for the error cases
  (`undefined label: x`, `not a loop label: x`). Sibling scopes may
  reuse a label independently.

- **Initializer blocks** (2026-07-03): `static { ... }` blocks run in
  the synthesized `<clinit>`, and instance `{ ... }` blocks run in
  every constructor after `super(...)` — both interleaved with their
  field initializers in source order (JLS §12.4.2 / §12.5), tracked by
  a per-class textual-order counter. `this(...)` delegation runs the
  instance initializers exactly once (in the delegated-to
  constructor); static blocks run exactly once at class init. Each
  block is its own local scope.

- **`enum` types** (2026-07-03): desugared to an ordinary class whose
  constants are `static final` singletons instantiated in source order
  in `<clinit>`. Supports constants with constructor arguments, enum
  fields/methods/constructors (user constructors gain the two implicit
  leading `name`/`ordinal` parameters), and the standard members
  `values()` (fresh array each call), `valueOf(String)` (javac's `No
enum constant E.X` `IllegalArgumentException` on miss), `ordinal()`,
  `name()`, and a default `toString()` returning the name (only
  `toString` is overridable — `name`/`ordinal`/`equals`/`hashCode`/
  `compareTo`/`getDeclaringClass` are FINAL in `Enum`, so overriding one is
  a compile error, JLS §8.9). `switch` on an enum matches unqualified
  constant names by reference identity; enum constants are singletons so
  `==` works, and a switch on a NULL selector throws
  `NullPointerException` (it dereferences the selector), while
  `valueOf(null)` throws `NullPointerException` ("Name is null") — both
  fixed 2026-07-19. Every enum is `Comparable` (compareTo by ordinal), so
  `Comparable<C> c = C.X` and `Collections.sort(enumList)` work, and
  `String.valueOf(anEnum)` gives its name (the general
  `String.valueOf(Object)` now coerces any reference via a null-safe
  `toString`). An enum whose EVERY constant has a body is implicitly
  abstract, so `enum E implements I { A { m(){…} }, … }` — the interface
  method supplied per constant — compiles. `@Override` and other
  annotations are parsed and ignored. **Gaps** (safe-direction, near-zero
  demand): `getDeclaringClass()`; `java.lang.Enum` as a written type;
  an enum nested inside another enum; and the not-found `valueOf` message
  omits the enclosing-class prefix for a NESTED enum (`No enum constant
  C.X`, not `Outer.C.X`) — the exception type and behaviour are right.

- **Varargs declarations** (2026-07-03): `Type... name` as the last
  parameter (an array at runtime). Overload resolution follows JLS
  §15.12.2 phase ordering — fixed-arity applicability is tried first,
  so `pick(int, int)` beats `pick(int...)` at arity two. Callers may
  spread any number of trailing arguments (packed into a fresh array,
  including zero), pass an assignable array directly (array form), and
  trailing arguments widen to the element type. Works for methods,
  static methods, constructors, and `super`/`this` calls.

- **Nested classes** (2026-07-03): `class`/`interface`/`enum`
  declarations inside a class body are hoisted to top level with their
  simple name (caturra shares one flat namespace). Static nested classes
  work fully — the dominant AP CS A pattern of a `private static class
Node` inside a linked structure. They are referenced by simple name
  inside the enclosing class and as `Outer.Inner` elsewhere (including
  `new Outer.Inner(...)`). Nested enums are supported. Non-static inner
  classes flatten the same way but cannot reach the enclosing
  instance's state (no `Outer.this` capture yet). This work also fixed
  multi-level field-access chains generally (`head.next.next.value`,
  reads and writes, and chains rooted at an implicit `this` field like
  `top.value`).

- **Interface default methods** (2026-07-03): `default` instance
  methods with bodies in an interface are inherited by implementers
  (no override required) and callable on instances and through
  interface-typed references. A default method may call the
  interface's abstract methods (resolved virtually on the actual
  instance) and is itself overridable. Virtual dispatch searches the
  superclass chain first, then implemented interfaces breadth-first
  (including super-interfaces) for an inherited default. Interface
  concrete methods are implicitly public.

- **User-defined generics** (2026-07-03): generic classes (`class
Box<T>`, `class Pair<A, B>`) and generic methods (`<T> T identity(T
x)`) with type-parameter erasure — every type variable is rewritten
  to `Object` at parse time, so the runtime is generics-unaware (as on
  a real JVM). Type bounds (`<T extends Comparable<T>>`) parse and
  erase. This also makes **`Object`** a usable top type: a synthetic
  `Object` class is the supertype of every reference type, with
  `toString`/`equals`/`hashCode`, and any reference widens to it.
  A **single-type-parameter** generic class (`Box<T>`, `Stack<E>`,
  `Node<T>`) tracks its type argument, so reads of a type-variable
  member are **cast-free**: `String s = box.get();` and `cell.value`
  yield `String` directly (the compiler inserts the `checkcast`).
  Multi-parameter generics (`Pair<A, B>`) and generic-method returns
  use raw type arguments — reads there still need a cast. Not yet:
  nested type arguments (`Box<Pair<A, B>>`).
  **Much widened 2026-07-23** — see "Generics, the deep round" above:
  bounded type variables can call their bound's methods (including
  `<T extends Number>` and intersection bounds `<T extends A & B>`), a
  generic method may take a `List<T>`/`Map<K, V>`, a primitive boxes
  into a `T`, explicit type witnesses parse, and raw types resolve.

- **Autoboxing** (2026-07-03): the wrapper types `Integer`, `Double`,
  `Long`, `Float`, `Short`, `Byte`, `Character`, and `Boolean` are
  storable reference types (a boxed primitive on the heap). Boxing
  (`Integer i = 5;`, `Object o = 42;`) and unboxing (`int x = i;`,
  `double d = box;`) happen automatically at assignments, method
  arguments, and returns (JLS §5.1.7/§5.1.8), via `Wrapper.valueOf` /
  `wrapper.xValue()`. Arithmetic and comparison auto-unbox their
  operands (`Integer a = 10, b = 3; int s = a + b;`), and the wrapper
  instance methods work (`intValue`, `doubleValue`, `compareTo`,
  `equals`, `hashCode`, `toString`). This also makes single-parameter
  generics over primitives usable in the collection sense (values box
  on the way in).

- **Anonymous classes** (2026-07-03): `new Interface() { ... }` and
  `new AbstractClass() { ... }` desugar to a synthesized top-level
  class (`Anon$N`) that implements the interface or extends the class,
  hoisted alongside the program. The body may declare its own fields
  and methods, override abstract methods, and inherit concrete and
  interface-default methods; the instance is used through its
  supertype.
  **The enclosing instance is captured since 2026-07-18**: an anonymous
  or local class in an instance method reads the enclosing object's
  fields and calls its methods by simple name, through the same
  `__caturraOuter` capture lambdas already used. A member the class
  provides ITSELF — declared or inherited from a user superclass —
  shadows the enclosing one and captures nothing, which is the
  difference from a lambda (a lambda has no members, so a bare name
  there is unambiguous). A library supertype's members are invisible to
  that check, so an anonymous subclass of a library class whose
  INHERITED member shadows an enclosing field would still capture the
  outer instance; nothing in the corpus does that, and assuming a shadow
  whenever the supertype is unknown would silently drop legitimate
  enclosing access, the worse failure.
  **`Outer.this`** (JLS §15.8.4) parses and resolves since the same
  date, walking out one `__caturraOuter` hop at a time so a
  doubly-nested class can name either enclosing instance.
  **Private members cross the nesting boundary** (JLS §6.6.1): they are
  accessible throughout the body of the enclosing TOP-LEVEL class, in
  both directions, so an inner class may call the outer's private method
  and the outer may read the inner's private field.
  **Rejected** since 2026-07-18, as javac does: a static member declared
  in an inner class (JLS §8.1.3 — constant variables excepted) and a
  qualified `new` of a static nested class.

- **Closure capture** (2026-07-03): an anonymous class body may
  reference effectively-final local variables of the enclosing method
  (`int t = 5; new Predicate() { boolean test(int x) { return x > t; }
}`). A capture pass runs after parsing: it walks each method with a
  local-variable scope, computes each anonymous class's captured free
  names and their types, synthesizes a private field and a constructor
  per capture, and passes the captured locals at the `new` site. Bare
  references in the body then resolve to the implicit `this`-field.
  Captures of any type work (primitives, objects, arrays), each `new`
  in a loop captures the value at that iteration, and captures combine
  with the anonymous class's own fields and the overriding method's
  parameters. (This is the machinery lambdas reuse.)
  A captured local must be **final or effectively final** (JLS §4.12.4),
  enforced since 2026-07-09 in javac's wording — `local variables
referenced from a lambda expression must be final or effectively
final`, or `from an inner class` for an anonymous class. Copying a
  capture into a field hides a later write, so accepting one would run a
  program a JDK refuses. A local with an initializer (or a parameter) is
  effectively final only if never reassigned; a blank local may be
  assigned once. caturra counts assignments rather than tracking definite
  assignment, so a blank local assigned on both arms of an `if` is
  refused where javac accepts it — stricter, the safe direction. Pinned
  by `reject_lambda_captures_a_reassigned_local` and
  `diff_effectively_final_capture_is_accepted`.

- **Lambda expressions** (2026-07-04): `x -> expr`, `(a, b) -> expr`,
  `() -> { ... }`, and typed-parameter forms. A lambda is target-typed
  against a _functional interface_ (an interface with a single abstract
  method) and desugared to an anonymous class implementing that method,
  reusing the anonymous-class and capture machinery — so lambdas
  capture effectively-final locals exactly as anonymous classes do.
  The target type is read from a declaration or field type (`Fn f = x
-> ...`), an assignment target (including array elements), a `return`
  statement, a method-call parameter (single-candidate resolution), or
  the **element type of a collection** for `list.add(() -> ...)`,
  `list.set(i, () -> ...)`, `list.forEach(x -> ...)`, `list.removeIf(x ->
...)` and `list.replaceAll(x -> ...)` — the element argument is typed
  against the receiver's `ArrayList`/`List`/`Set`/`Collection<E>` argument,
  the same receiver-driven typing `Map.forEach` uses.
  `list.replaceAll(x -> ...)`, `list.sort((a, b) -> ...)` and
  `Collections.sort(list, (a, b) -> ...)` bind the erased
  `__Consumer`/`__Predicate`/`__UnaryOperator`/`__Comparator`; `sort`'s
  lambda is typed from the receiver's (or the first argument's) element
  type, and `replaceAll`'s result is checked against the element type
  (which the erased `Object` return would otherwise drop). Expression bodies
  become `return e;` (or `e;` for a void SAM); block bodies are used
  directly. A lambda body may also be a **statement expression** — an
  assignment or `++`/`--` (`n -> sum[0] += n`, `() -> count++`), which Java
  treats as an expression and caturra lowers to a one-statement block
  (2026-07-09). A lambda in a position with no functional target type is
  reported. Pinned by `diff_lambda_as_a_list_element` and
  `diff_lambda_statement_expression_body`.
  `Map.forEach` is target-typed from its **receiver** (2026-07-09): the
  synthesized class implements the bundled erased `__BiConsumer`, whose
  `accept(Object, Object)` opens with the two casts javac puts in a bridge
  method, so `(key, value)` have the map's declared types. Pinned by
  `diff_map_for_each_lambda`.
  A lambda's body sees the members of the class that created it
  (2026-07-09): its **static fields** by the enclosing class, and its
  **instance** members — a bare field, `this.field`, a bare method call,
  `this` itself — through a captured enclosing `this`. Java captures the
  enclosing instance as `this$0`; caturra captures it as a synthetic
  `__caturraOuter` field and resolves instance references through it, live
  on the real object, so a lambda mutating `field` writes the enclosing
  object. Scoped to a lambda **directly** in an instance method: a nested
  lambda reaching an instance field two levels up needs transitive capture
  and is a compile error (javac accepts it — the safe direction), as is
  capturing a `StringBuilder`. Pinned by
  `diff_lambda_captures_enclosing_instance`. A captured local must be
  effectively final, enforced as for anonymous classes above.

- **Method references** (2026-07-04): all four kinds — static
  (`Integer::parseInt`), unbound instance (`String::length`, where the
  first SAM parameter is the receiver), bound instance
  (`System.out::println`, on a value), and constructor (`Point::new`).
  Each is target-typed against a functional interface and desugared to
  the equivalent lambda, then handled by the lambda pipeline.
  Static-vs-instance is resolved precisely for user classes and via a
  curated static-method set for library types.

- **`Comparable<T>` and generic supertype clauses** (2026-07-04):
  `extends`/`implements` clauses now accept generic type arguments
  (`class Foo implements Comparable<Foo>`, `class Bar extends
Base<String>`), erased like everywhere else. `Comparable<T>` is
  modeled as a built-in interface, so a class implementing it with
  `int compareTo(Foo)` works — called directly on a concrete receiver
  and through a `Comparable`-typed one (e.g. a selection sort over
  `Comparable[]`). Array covariance (`Card[]` to `Comparable[]`) and
  the erasure bridge (a `compareTo(Object)` interface call dispatching
  to `compareTo(Card)`) are handled. This completes the AP CS A subset.

- **Code.org CSA console patterns** (2026-07-04): fixes surfaced by
  surveying the Code.org CSA curriculum corpus. Leading-dot double
  literals (`.5` is `0.5`); wrapper constructors (`new Integer(5)`,
  `new Double(3.5)`, `new Integer("42")`) that box the argument; a
  user class shadowing a wrapper name (`ArrayList<Character>` where
  `Character` is user-defined); reading and assigning a static field
  through an instance (`obj.staticField`, `obj.staticField = v`,
  legal if discouraged); and `super.method(...)` as an expression
  statement. Lifted console-solution compile coverage from 82.6% to
  85.3% (the remainder is dominated by `java.lang.reflect`).

Everything else parses into a not-yet-supported diagnostic with recovery, so a
file full of future-Java still reports one clear message per construct.
Value-position `++`/`--` (e.g. `y = x++`) is parsed and rejected with a
friendly message for now.

## Divergences from javac

The one-directional rule: **anything that compiles in caturra must also
compile on a real JDK 11.** Being stricter is safe — a student sees the
error here instead of later. Being more permissive is not: that code runs
in the playground and fails on a JDK. Every known case in both directions
is enumerated below, and each is pinned by a test in
`crates/caturra-vm/tests/differential.rs` that runs a live `javac` — so a
case cannot silently change direction, and neither list can grow unnoticed.

**Stricter than javac** (caturra rejects; javac accepts). Each
`stricter_than_javac!` test fails if javac ever starts rejecting the
program, which would mean it is a shared rule rather than a strictness:

- `new StringBuilder().capacity()` — capacity is an implementation detail
  of a growable buffer caturra does not model.
- `Arrays.fill(new String[1], 5)` — javac erases to `fill(Object[], Object)`
  and throws `ArrayStoreException` at run time.
- `Arrays.sort(new Plain[2])` where `Plain` is not `Comparable` — javac
  throws `ClassCastException` at run time.
- `Collections.frequency(list, wrongType)` — javac's parameter is `Object`
  and it answers 0.
- `list.containsAll(otherOfADifferentElementType)` — likewise `Collection<?>`.
- `Collections.addAll(List<Integer>, new Integer[] {1})` — caturra reads a
  lone array as the varargs array only for a reference element type.
- `LinkedList<Integer> l;`, `HashSet`, `TreeMap`, `TreeSet` and the rest of
  the unmodeled library — a scope limit, reported by name wherever written
  rather than as a missing symbol.

**More permissive than javac** (caturra accepts; javac rejects). **This
list is empty**, and the `looser_than_javac!` macro exists to keep it
that way — a case asserted there is a case that cannot be forgotten.

Until 2026-07-09 it held three: `Collections.sort`, `max`/`min` and
`binarySearch` over a list whose element type is not `Comparable`, which
caturra accepted and failed on at run time with `ClassCastException`.
Those four methods are declared over `T extends Comparable<? super T>`,
so the bound is now checked at compile time, and `ArrayList<Object>` is
refused with it. `Arrays.sort` always refused a non-`Comparable` element,
because it is bundled Java whose parameter is `Comparable[]`; only the
native `Collections` had to be taught to ask. The bound reaches exactly
the four methods that declare one — `reverse`, `shuffle`, `swap`,
`frequency` and `nCopies` still take any element type, as javac's do.
The last runtime-only permissiveness (`Scanner.close`) and the last
compile-time one (a lambda capturing a non-effectively-final local) were
both closed on 2026-07-09, so caturra has no known divergence from a real
JDK 11 in either direction across the corpus.

**Reject wording.** `reject_wording_tracks_javac` pins both javac's
headline and caturra's message for 19 rejected programs, so the two are
compared rather than remembered. Most of caturra's messages are javac's
headline plus the detail javac prints on its `symbol:`/`location:`
continuation lines. The deliberate exceptions, each recorded with its
reason in the table:

- Where javac has a **single** candidate it says `cannot be applied to
given types` or reports converting the argument; caturra says
  `no suitable method found` uniformly (`Math.multiplyFull`,
  `System.arraycopy` arity, `Collections.unmodifiableList`).
- Where javac reports **overload resolution** failing across many
  overloads, caturra names the offending argument, which is what a
  student needs (`Arrays.fill(int[], String)`, `Collections.addAll`,
  `Collections.binarySearch(List<Integer>, String)`).
- `Arrays.copyOf(String[], 2)` assigned to `int[]`: javac explains its
  generic inference; caturra names the two array types.
- `int[] c = {1,,2}`: javac says `illegal start of expression`, caturra
  `expected an expression` — a parser message, not a library one.

## Codegen choices

- Class file format: major version 55 (Java 11), as fixed in
  `caturra-classfile`.
- String concatenation compiles to `StringBuilder` chains (javac-8 style), not
  `invokedynamic`/`StringConcatFactory` — keeps the VM free of `indy` support.
- No `StackMapTable` emission (see [RUNTIME.md](RUNTIME.md) — our VM is the
  only verifier).
- Constructors emit the implicit `Object.<init>` super call followed by
  instance-field initializers, and a default constructor is synthesized when
  none is declared — matching javac's shape.

## Staging (each stage flips its diagnostics to real support)

1. ~~Local variables, assignment, arithmetic/comparison/logical operators,
   string concatenation.~~ **Done (2026-07-02).**
2. ~~Control flow: `if`/`else`, `while`, `for`, `break`/`continue`.~~
   **Done (2026-07-02)** — plus `do`/`while`. Labeled break/continue and
   `switch` remain out for now; for-each arrives with arrays (stage 4).
3. ~~Static methods with parameters and returns (user-defined), recursion.~~
   **Done (2026-07-02)** — verified against OpenJDK 11 by the differential
   suite (`crates/caturra-vm/tests/differential.rs`), which runs identical
   programs through `javac`+`java` and caturra and requires byte-identical
   stdout.
4. ~~Arrays (1D, then 2D), `for-each`.~~ **Done (2026-07-02)** — both
   dimensions at once, differential-verified against OpenJDK 11.
5. ~~Objects: fields, constructors, `new`, instance methods, `this`.~~
   **Done (2026-07-02)** — plus static fields/`<clinit>` and private-access
   enforcement, differential-verified against OpenJDK 11.
6. ~~Inheritance: `extends`, `super`, overriding, polymorphic dispatch;
   `interface` / `abstract`.~~ **Done (2026-07-02)** — plus `this(...)`
   chaining and `instanceof`, differential-verified against OpenJDK 11.
7. ~~Generics as far as `ArrayList<E>` requires (erasure, autoboxing).~~
   **Done (2026-07-02)** together with the intrinsic class library
   (String/Math/Integer/Double/Scanner/ArrayList), differential-verified
   against OpenJDK 11 including Scanner over piped stdin.
8. ~~`java.io.File` + readers/writers over the virtual filesystem.~~
   **Done (2026-07-02).** The SCOPE.md surface is fully covered.

The staging order optimizes for what CSA course units need earliest.
