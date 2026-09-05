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
- **Numeric edges and evaluation** (2026-07-31, round 7) — the last five
  findings, closing the round:
  - **`2147483648` exists only as the DIRECT operand of unary minus**
    (JLS §3.10.1): the negation now folds a literal only when it follows
    the `-` immediately, so `-(2147483648)` is the out-of-range literal
    javac rejects, while `- -2147483648` compiles and wraps back to
    `Integer.MIN_VALUE` as it should.
  - **An underscore may appear only BETWEEN digits**: `1_`, `_1`-adjacent
    forms, `0x_FF`, `0xFF_`, `0b_101`, `10_L` and `1_.5` are errors, while
    `1_000`, `1__0`, `0x1_F`, `0b1010_1010`, `1.5_2` and `1_0e1_0` stay
    legal — the placement rule caturra had simply stripped.
  - **`i++` on a BOXED counter is a value**: the increment unboxes for the
    arithmetic (JLS §15.14.2), so `i++ + 1` and `i++ < n` are ordinary
    expressions. They had been "bad operand types" — the increment itself
    always worked, only its static type was wrong.
  - **A blank `static final` may be assigned in a static initializer**
    (JLS §8.3.1.2), exactly as a blank instance final may be in a
    constructor. Both final-assignment checks now ask one predicate.
- **Scanner and the print stream** (2026-07-31, round 7) — 8 of 9 findings:
  - **`System.out.write(int)`, `append(char)`/`append(CharSequence)` and
    `flush()`** exist (they were "cannot find symbol" for real JDK APIs).
    `append` answers the stream, which caturra discards — nothing here has
    a `PrintStream` value to chain from.
  - **`printf` writes as it goes**, so a specifier that throws leaves
    everything before it PRINTED: `printf("a%dz%s", 5)` shows `a5z` and
    then throws, as the JDK's Formatter does (it appends to its
    destination one specifier at a time).
  - **An unknown conversion is `UnknownFormatConversionException`**,
    reported before the argument is fetched — `printf("%q")` had been
    complaining that the argument was missing.
  - **Scanner's numeric messages are the JDK's**: a well-formed number that
    does not fit reports `For input string: "99999999999"`, and
    `nextByte`/`nextShort` report `Value out of range. Value:"200"
    Radix:10`. Both had been a bare `InputMismatchException` with no
    message. A token that is not a numeral at all still has none, as on a
    JDK.
  - **Locale grouping is read**: `1,234` and `1,234,567` are integers (and
    `1,234.5` a double), while `1,23`, `12,34` and `,123` are not — the
    JDK validates the group widths, and so does this.
  - **`nextInt(radix)` / `hasNextInt(radix)`** read a token in another
    base.
  - **One left open**: two Scanners over `System.in`. A JDK's first
    Scanner BUFFERS ahead, so a second sees nothing; caturra's read from
    the live stream, so the second still finds input.
- **Interface members** (2026-07-31, round 7) — 7 of 8 findings:
  - **A member type of an interface is implicitly static** (JLS §9.5), so
    `interface Shape { class Point {…} }` needs no enclosing instance;
    member INTERFACES and enums declare there too, and `implements
    Outer.Inner` parses (a member type is named through its enclosing one,
    which caturra drops since it flattens nested types to simple names).
  - **`Outer.Nested.MEMBER` resolves** — the constant of a nested enum, a
    static call on a nested type (`Holder.Kind.valueOf("BLUE")`), and `new
    Holder.Deep()`. This was never interface-specific: a nested type inside
    a CLASS could not be named through its enclosing one either.
  - **An interface has no constructor** (JLS §9.1.4): one is now refused
    where it used to be accepted and silently ignored.
  - **A PRIVATE static is reachable only from its own top-level type**
    (JLS §6.6.1). The instance-call path checked this; the STATIC path did
    not, so `Calc.base()` on a private interface static — or on any other
    class's private static — compiled from anywhere.
  - **A static method may not override or hide an inherited default**
    (JLS §8.4.8.1/§9.4.1): `class C implements A { static who() }` and
    `interface B extends A { static who() }` both ran, picking whichever
    the dispatch found. The override walk climbs superclasses only, so the
    interface case needed its own check.
  - **One left open**: `Interface.super.m()`, which does not parse — the
    way an overriding class reaches the default it overrode.
- **The library tail** (2026-07-31, round 7) — 6 of 9 findings:
  - **`Objects.deepEquals`, `checkIndex` and `compare(a, b, cmp)`** exist
    now. `deepEquals` compares two PRIMITIVE arrays element by element as
    well (the deep walk only knew reference arrays), `checkIndex` returns
    the index or throws with the JDK's "Index 5 out of bounds for length
    5", and `compare` answers 0 for identical arguments WITHOUT consulting
    the comparator — which is what makes two nulls equal.
  - **`Boolean.valueOf(String)`** — `parseBoolean`'s answer, which the
    table simply lacked beside `valueOf(boolean)`.
  - **A method called straight on `Collections.emptyList()`** (or
    `emptySet`/`emptyMap`) works: those type as `null` so they assign to a
    collection of ANY element type, and a receiver in that position now
    resolves against the general Object-element face instead of "cannot
    call methods on null".
  - **`Collections.unmodifiableList(new ArrayList<>())`** — a DIAMOND
    argument types as `null`, which no longer fails overload resolution;
    the result stays element-unknown so it assigns onward, exactly as
    `emptyList()` does.
  - **`Integer.decode`'s message names the stripped remnant**, as the
    JDK's does: `decode("0x")` is `For input string: ""`, not the whole
    input.
  - **Three left open**: `List.of`/`Set.of`/`Map.of` (JDK 9 factories,
    still refused — with a message that wrongly blames the type name), and
    `remove(Object)` of an ABSENT element on an empty or singleton list,
    which should answer `false` rather than throwing (only a removal that
    would really remove throws).
- **`Outer.this`, and the static-context diagnostics** (2026-07-31, round
  7) — 4 of the 10 inner-class findings:
  - **`Outer.this` in a STATIC context is an error**, checked before the
    same-class shortcut that used to load local slot 0 — which in `main` is
    the args array, so `P.this` printed `[Ljava.lang.String;@0` instead of
    being refused.
  - **`Outer.this.field` in an expression** compiles: the type of a path
    ending in `this` is the named enclosing class (it typed as an error, so
    `Outer.this.x + 1` reported "bad operand types" although the emitter
    read the field correctly), and a field chain hanging off a qualified
    `this` resolves through it.
  - **A static nested class gets javac's reason**, not a puzzle: reading
    an outer INSTANCE field says "non-static variable field cannot be
    referenced from a static context" rather than "cannot find variable",
    and `Outer.this` from one says the same rather than "not an enclosing
    class" — which was doubly wrong, since it plainly is one.
  - **Six left open**: subclassing an inner class, `p.new Inner() {…}`,
    `o.super()`, an anonymous class inside a LOCAL class reaching the outer
    instance, a blank local assigned once per branch being capturable, and
    member types (interface/enum/static class) declared inside an inner
    class, which javac forbids.
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
- **The regex engine: what a pattern means, and where it errors**
  (2026-07-31, round 8) — the regex-deep cluster, 12 of 13:
  - **A quantifier after `\Q…\E` binds the LAST quoted character.** A quoted
    run expands to a sequence of literals, so `\Qab\E+` is `a` then one-or-
    more `b` — caturra repeated the whole run, making `abb` fail and `abab`
    match, each the opposite of a JDK.
  - **A `&&` intersection binds tighter than the class's negation**:
    `[^a-c&&[^b]]` is "not (in a-c and not b)", which `b` satisfies. Negating
    first made every negated intersection answer false.
  - **`$` does not fire between a final CR and LF** — the pair is ONE line
    terminator, so `"a\r\n".replaceAll("$", "X")` gets one X, not two.
  - **`[a-[b]]` is a UNION, not a range** (a `-` before `[` does not open one),
    while `[a-&&b]` really is the illegal range it looks like.
  - **A backreference to a group that does not exist compiles** and simply
    never matches; only `\0` is an error.
  - **`\R` and `\x{…}`** are legal JDK-11 escapes and were refused.
  - **A `{` after a quantifiable atom must open a repetition**: `a{x` is the
    JDK's "Illegal repetition", not a literal brace (a `{` in ATOM position
    still is one).
  - **The replacement string is checked**: a lone trailing backslash is
    "character to be escaped is missing", and a bare trailing `$` has its own
    "group index is missing" wording.
  - **The error INDEX is where the parse stopped**, as the JDK's cursor
    reports it — not where the construct began, which put the caret under the
    wrong character in four different messages.
  - **A simple greedy repeat runs in a LOOP**, not one stack frame per
    repetition: `a*b` over a thousand characters is an ordinary pattern, and
    recursing per iteration exhausted the backtracking budget (silently
    reporting "no match") long before it finished. The budget is raised to
    match, and the behaviour is unchanged — match as far as the body goes,
    then try the continuation from the longest run down.
  - **One left open**: matching walks UTF-16 CODE UNITS, so `.` and `[^…]`
    treat a supplementary character as two. Fixing it means the matcher
    stepping by code point throughout.
  - Pinned by `diff_regex_semantics` and `diff_regex_errors`.
- **The String tail: identity, surrogates, and the API corners**
  (2026-07-31, round 8) — the string-tail-2 cluster, all 15:
  - **The JDK returns THIS string** when a range covers all of it, so
    `s.substring(0) == s`, `s.substring(0, s.length()) == s` and
    `s.subSequence(0, s.length()) == s` are all true, `repeat(1)` answers the
    receiver and `repeat(0)` the `""` LITERAL (interned, so `== ""` holds).
    Each allocated a copy and answered false.
  - **A `char[]` round-trip keeps SURROGATE PAIRS.** `String.valueOf(char[])`
    and `copyValueOf` rendered unit by unit through a code point, so every
    non-BMP character came back as two U+FFFDs and the string no longer
    equalled itself. `String.valueOf((char[]) null)` throws — it is not the
    `Object` overload, which a `char[]` never takes. And `indexOf(int)` on a
    lone SURROGATE now finds it: `char::from_u32` rejects one, so every search
    for a real code unit of the string answered -1.
  - **Case folding is per UNIT with the SIMPLE mapping**, as
    `Character.toUpperCase` is: Rust's full mapping expands `ß` to `SS`, which
    made `compareToIgnoreCase` call equal strings unequal.
  - **The missing API**: `regionMatches` (both forms), `contentEquals` over
    the JDK's real `CharSequence` parameter — two narrower overloads made a
    `CharSequence`-typed argument "no suitable method" and a `null` ambiguous
    between overloads Java does not have — `String.CASE_INSENSITIVE_ORDER`,
    the `Object` methods on a `subSequence` result, `join` over any `Iterable`
    (a `Set` was refused) and with a `StringBuilder` or `null` delimiter, and
    the BARE NullPointerExceptions `join`'s null checks really throw.
  - Pinned by `diff_string_identity_and_surrogates` and
    `diff_string_api_corners`.
- **Boxing identity, `Math`'s overloads, and the flow that decides definite
  assignment** (2026-07-31, round 8) — the boxing-identity (6),
  operator-order (5), compound-assignment (2), math-deep (8 of 10) and
  labeled-flow (7 of 8) clusters:
  - **`Wrapper.valueOf(...)` answers the wrapper OBJECT**, not the bare
    primitive — every one of them typed and returned the primitive, so
    `Long.valueOf(1000L) == Long.valueOf(1000L)` compared by VALUE and
    answered true where a JDK's two distinct objects answer false, and
    `valueOf(x) == someObject` was refused as "long and Object". The
    small-value CACHE now applies wherever caturra boxes, so
    `IntStream.boxed()` yields the very object a literal `Integer y = 1`
    holds. `boxed()` also had to genuinely BOX: it was a retyping, and a raw
    `int` reaching a collection is a VerifyError at the next reference use.
  - **`++` on a boxed `Character`/`Short`/`Byte` is legal** (JLS §15.14.2:
    unbox, add, NARROW, box) while `+= 1` on one is not (§15.26.2: boxing
    cannot narrow). A statement-form `x++` used to LOWER to `x += 1`, which
    conflated the two and refused both.
  - **`(int) someNumber`** — a cast from a wrapper supertype (`Number`,
    `Comparable`) to a primitive is a checked cast plus an unboxing
    conversion, and was "cannot cast Number to int".
  - **`o += "x"` on an `Object`- or `CharSequence`-typed variable** holding a
    String: §15.26.2's implicit cast makes it legal, since a String IS a T.
  - **The `Math` surface**: the `float` overloads of
    `copySign`/`fma`/`getExponent` (which had widened to `double`, and read a
    `float`'s exponent from the wrong bias), `floorMod(long, int)`'s `int`
    result, `IEEEremainder`'s SIGNED ZERO (it carries the dividend's sign, so
    `1.0 / IEEEremainder(-4.0, 2.0)` is `-Infinity`), the
    `BigInteger divide by zero` message the JDK's unsigned LONG division
    inherits, a statically imported `Math.PI`, and `Math.PI = 3.0` — which
    compiled and was silently discarded, since the constant is folded at its
    reads and there was nothing to assign to.
  - **Definite assignment now follows the control flow**: an assignment in
    the short-circuited operand of `||`/`&&`, in one branch of `?:`, or on
    the non-`break` path out of a labeled block does NOT definitely assign
    (JLS §16.1.1/§16.1.4/§16.2.6) — each used to leave the flag set and let a
    later read compile. A labeled BLOCK is no longer treated as a loop, so a
    blank final may be assigned in one.
  - **A constant CONDITION is recognized**: `while (1 == 1)` and
    `while (FLAG)` for a `static final boolean` make what follows unreachable,
    exactly as `while (true)` does — both used to loop forever. The folding
    covers the literal forms and, at statement level, the enclosing class's
    own constant variables.
  - **The parser**: `?:`'s middle operand is a full `Expression` and its third
    may be a lambda (`true ? x = 2 : 3`, `c ? () -> "a" : () -> "b"`), a label
    may sit on the empty statement, and `lab: int x = 5;` is the error JLS
    §14.7 makes it.
  - **Three left open**: the transcendentals differ from the JDK by one ULP
    (matching `StrictMath`'s fdlibm results bit-for-bit is its own project),
    `new Math()` reports the unresolved TYPE rather than the private
    constructor, and a blank final assigned once inside `while (true)` before
    a `break` is still refused.
  - Pinned by `diff_boxed_identity`, `diff_math_overloads`,
    `diff_labels_and_conditional_operands` and six `reject_*` tests.
- **What a nested class can see, and what it may not** (2026-07-31, round 8)
  — the local-scoping and field-hiding clusters, 18 of their 21 findings:
  - **`Outer.this` works from an anonymous OR a local class** (JLS §15.8.4).
    The capture pass did not recognize a qualified `this` as needing the
    `__caturraOuter` link, so all four spellings were reported as a static
    context. It parses as a NAME PATH containing `this`, and
    `Outer.this.field` continues into `["Outer", "this", "field"]` — so the
    segment is not always last, which is why matching only the end fixed the
    method call and not the field read.
  - **A local class threads its captures through a `this(...)` delegation.**
    Without them the delegation matched the constructor's own newly-extended
    signature and recursed until the stack blew.
  - **An anonymous class can extend a local class, and a local class in a
    nested block can extend one from the enclosing block.** Both were "cannot
    find symbol": the renaming that hoists a local class covered the
    statements of its own block, but those bodies had already been hoisted out
    of it. A qualified reference (`C.F`) was invisible to the rename too.
  - **A field initializer is NOT in the constructor's scope** (JLS §8.3.1),
    though it runs inside it: `int a = q;` silently read the constructor's
    parameter `q` instead of the field's default — and a field initializer
    naming a parameter that exists in only ONE constructor compiled.
  - **A private field of an ancestor is not inherited** (JLS §8.2): reading
    one by simple name from a subclass is an error, which the qualified
    `obj.f` path already checked and the bare-name path did not.
  - **Three more programs javac refuses**: two local classes of one name in a
    block; capturing a for-each variable that the body reassigns (the loop
    variable arrives INITIALIZED on every pass, so a write costs it its
    effective finality); and a `static` member of a local class that is not a
    constant variable (JLS §8.1.3) — while `static final int F = 3;` now
    compiles and is readable as `C.F`.
  - `getSimpleName()` on a local class is the name the source wrote, not
    caturra's hoisted `Name$LocalN`.
  - **Three left open**, all about a local class's DECLARATION-POINT scope:
    a local class may still reference a local declared after it (caturra
    computes captures at the `new` site, not at the declaration), a local
    class inside a `switch` case is refused (local classes are hoisted from
    block bodies, and a switch arm is not one), and a local class nested
    inside another cannot capture the outer method's locals.
  - Pinned by `diff_qualified_this_in_nested_classes`,
    `diff_local_class_shapes`, `diff_initializer_scope` and five `reject_*`
    tests.
- **The `Object` contract, on both sides** (2026-07-31, round 8) — the
  object-contract cluster:
  - **`getClass()` belongs to every reference** — a `String`, a collection, a
    `StringBuilder`, a boxed wrapper — and was missing from all of them.
  - **A `Class` handle is INTERNED, so identity is meaningful**, and
    `getSuperclass()` interned the SIMPLE name: that minted a second class
    object called "Object", so `getSuperclass() == Object.class` was false and
    `getName()` answered "Object" rather than `java.lang.Object`.
    `Class.toString()` (`class java.lang.Object`) was missing too.
  - **`Object.toString()` uses the object's OWN hash**: it is
    `getName() + "@" + Integer.toHexString(hashCode())`, so a class that
    overrides `hashCode` and not `toString` prints the overridden value.
    caturra used the identity hash, which made `f.toString()` disagree with
    the very expression the JDK documents it as.
  - **`super.hashCode()`/`super.toString()`/`super.equals(o)` reach `Object`'s
    own**, where they used to abort the run with "unknown native member" —
    `java.lang.Object` has no class file here, so the call found nothing.
  - **`Cloneable` exists, and `Object.clone()` consults it**: a class that
    implements the marker gets a field-by-field copy, and one that does not
    gets the checked `CloneNotSupportedException` named after itself.
  - **Three programs javac refuses** now stop here too: declaring one of
    `Object`'s FINAL methods (`getClass`/`notify`/`notifyAll`/`wait`, JLS
    §8.4.3.3); `@Override` on an `equals(SubType)` overload — the classic bug
    the annotation exists to catch, which caturra's erasure-tolerant matcher
    let through because `Object.equals`'s parameter really IS `Object` (the
    tolerance is for erased type VARIABLES, so it now stops at `Object`
    itself); and an `instanceof` between two unrelated FINAL types, which can
    never be true (JLS §15.20.2) and so is an error rather than a `false`.
  - **One gap left open**: a nested class's `getClass().getName()` is `Inner`,
    not `Outer$Inner`. caturra hoists nested classes to flat top-level names,
    so the enclosing prefix is gone by the time the VM sees one; restoring it
    means renaming them at the hoist, which reaches the class table,
    diagnostics and stack traces alike. Recorded since round 6.
  - Pinned by `diff_object_class_handles`, `diff_object_defaults`,
    `diff_object_clone` and three `reject_*` tests.
- **A stream is a TYPE and a value, not only a chained expression**
  (2026-07-31, round 8) — the stream-deep cluster, 23 findings and the round's
  largest structural gap:
  - **`Stream<T>` and `IntStream` name types**, so a pipeline can be held in a
    variable, a field or a parameter, and `var` infers one. Before this a
    stream existed only inside one chained expression.
  - **A stream can start somewhere other than a collection**: `Stream.of`,
    `Stream.empty`, `Stream.concat`, `IntStream.of` and `Arrays.stream` (over
    a reference array and over a primitive one, which yields an `IntStream`).
    Each is variadic or array-taking, so none fits a fixed method table; they
    lower to one array plus a call, as `Arrays.asList` does.
    `entrySet().stream()` streams whole `Map.Entry`s — it used to hand out
    KEYS, so the pipeline's lambda died on a cast.
  - **The operations refused with a FALSE reason** — "a lambda is only allowed
    where a functional-interface type is expected", said of methods that have
    taken one since Java 8 — now work: `flatMap`, all three `reduce` forms,
    `toArray`, `mapToDouble`/`mapToLong` (whose `sum()`/`toArray()` carry
    their own numeric width) and `summaryStatistics()`, with
    `IntSummaryStatistics` a nameable type.
  - **The collectors beyond `toList`/`toSet`/`joining`**: `counting`,
    `groupingBy` (with an optional downstream), `partitioningBy` (whose map
    always holds both keys), `toMap` (with a merge function, and the JDK's
    `Duplicate key` IllegalStateException without one), and the
    `summing*`/`averaging*` family. Their lambdas take the STREAM's element —
    a target type that comes from TWO levels up, through the enclosing
    `collect`, which is the only such case in caturra.
  - **A `char` or a `boolean` result stays its own wrapper.** Lambda results
    were unboxed on the way back into the pipeline, and a bare `Int` cannot
    say which of `Integer`/`Character`/`Boolean` it is — so
    `map(s -> s.charAt(0))` was a stream of 97s and `map(String::isEmpty)` a
    stream of 0s. The numeric wrappers still unbox: the primitive pipelines
    are built on that representation.
  - **The rules a stream obeys**: `limit`/`skip` reject a negative count
    (IllegalArgumentException naming it) rather than reading it as an empty or
    a full stream; a pipeline is **single-use**, so a second operation on one
    is "stream has already been operated upon or closed"; and a terminal
    **fails fast** when the collection it was opened over is modified while it
    runs — the check sits AFTER the traversal, exactly where
    `ArrayList$ArrayListSpliterator.forEachRemaining` checks its modCount, so
    every original element is still seen before the throw.
  - Also: `sorted(Comparator.comparing(s -> s.length()))` types its key
    extractor from the stream's element, and `println(...collect(toList()))`
    no longer reads as the ambiguous `println(null)` — only the LITERAL `null`
    is that overload.
  - Pinned by `diff_stream_types_and_sources`, `diff_stream_operations`,
    `diff_stream_collectors` and `diff_stream_rules`.
- **Varargs: the lone array, the forwarded one, and which overload wins**
  (2026-07-31, round 8) — the varargs-deep cluster:
  - **A lone REFERENCE array IS the varargs array; a PRIMITIVE one is not.**
    `T` cannot be `int`, so it infers as `int[]` and `Arrays.asList(int[])`
    is a ONE-element `List<int[]>` — the most famous varargs gotcha, which
    caturra spread, answering 3 where a JDK answers 1. `Arrays.asList((String[])
    null)` used to abort the whole run with an internal "malformed class
    Arrays" instead of throwing the JDK's plain NullPointerException.
  - **`Collections.addAll` takes any `Collection`**, not only a `List` —
    a `Set` receiver, its commonest use, was refused outright. Each element
    now goes through the collection's own `add`, which dedups a set and sifts
    a heap.
  - **A varargs parameter can be FORWARDED**: `printf(fmt, parts)` inside a
    `void log(String fmt, Object... parts)` passes the array, whose elements
    are the format arguments. caturra reported "cannot format Object[]"; the
    descriptor now carries a `[` tag and the VM spreads it, with a null array
    reading as one null argument as the JDK does.
  - **A method reference to a varargs method fits ANY arity**, not only its
    declared one (JLS §15.12.2.4): `Q::pack` for a two-argument functional
    interface was "unexpected static method pack(1 args)".
  - **Variable-arity SPECIFICITY** (JLS §15.12.2.5) now compares each
    method's declared parameter inside its fixed prefix and its varargs
    ELEMENT beyond it, so overloads of DIFFERENT arity are comparable and
    `s(String, Object...)` beats `s(Object...)` rather than clashing with it.
    And specificity is SUBTYPING, not method-invocation conversion: `int` is
    not a subtype of `Object`, so `b(int...)` and `b(Object...)` really are
    ambiguous for `b(1, 2)` — caturra had an "`Object...` always loses"
    shortcut that silently picked the primitive one.
  - Pinned by `diff_varargs_array_arguments`, `diff_varargs_forwarding`,
    `diff_varargs_specificity_across_arities` and
    `reject_ambiguous_primitive_and_reference_varargs`.
- **What runs before what, when an object is built** (2026-07-31, round 8) —
  the instance-init-order cluster: four wrong answers at run time, and eight
  programs javac refuses that caturra ran:
  - **A CONSTANT VARIABLE is inlined at its use site** (JLS §4.12.4/§13.1) —
    and an INSTANCE one counts, not only a `static` one. caturra inlined
    only statics, so `final int K = 5;` read through a superclass
    constructor's virtual call printed 0 (the field slot is genuinely still
    0 there; javac reads the constant instead). The receiver of an inlined
    instance read is evaluated and discarded, as javac leaves it.
  - **A null collection prints "null"**. Printing and concatenation are
    `String.valueOf(Object)` (JLS §5.1.11), which is null-safe, but caturra
    emitted a bare `toString()` call for a `List`/`Map`/`Set`/`File`/`Path`
    receiver — so an uninitialized field read from a superclass constructor
    threw NullPointerException. Three of the six types already guarded the
    call inline; one shared `emit_string_value_of` now does it for all.
  - **An enum constant's name and ordinal are set by the SUPER
    constructor**, so the enum's own field initializers can call `name()`:
    `String tag = name() + "-" + ordinal();` is `RED-0`, not `null-0`. The
    desugaring's two hidden stores are lifted to where `java.lang.Enum`'s
    constructor would have run them — a new `MethodDecl::pre_init` counts the
    leading statements that stand in for a super constructor.
  - **A field or instance initializer may read a CAPTURED LOCAL**, since
    javac's `val$x = x` stores also precede the initializers. caturra's
    capture pass looked only at an anonymous class's METHODS, so
    `new Job() { int w = captured; { z = captured + 1; } }` — ordinary Java —
    was rejected outright with "cannot find variable".
  - **The eight compile-time rules an initializer used to escape**, all in
    the accepts-invalid direction: `return` inside one (`return outside
    method`, JLS §14.17), an INSTANCE initializer that cannot complete
    normally (§8.6 — a static one may, and `static { throw ... }` stays
    legal under §8.7), an illegal forward reference from a block (§8.3.3,
    which only field initializers were checked for), a blank final read
    before the block assigns it or assigned again by a constructor after it
    (§16.9), a checked exception thrown by an instance initializer or field
    initializer (§11.2.3 — permitted only when EVERY constructor declares
    it), `this` (explicit, or implied by a bare instance field or method
    call) in a `super(...)`/`this(...)` argument (§8.8.7.1), and a recursive
    constructor invocation, which used to compile and blow the stack. The
    recursion check matches delegation targets by arity and reports only a
    constructor no resolution can terminate, so an ambiguous overload set
    never produces a false error.
  - Pinned by `diff_initialization_order`,
    `diff_initializers_read_captured_locals` and eight
    `reject_*` differential tests.
- **A generic class may have more than one type parameter** (2026-08-06,
  JLS §4.4/§4.5) — the largest of round 9's deferrals, and a rejects-valid
  gap rather than a wrong answer: `class Pair<K, V>` compiled, but every USE
  of one was raw, so `String k = p.getKey()` was "Object cannot be converted
  to String". Two things were single where Java has many:
  - **A type variable had no identity.** The parser erased every tracked
    parameter to ONE reserved name, so a `K` and a `V` were the same type to
    the compiler and only a class with exactly one parameter could be tracked
    at all. The reserved name now carries the parameter's DECLARED POSITION,
    and `JType::TypeVar` carries it too — that position is what selects the
    argument, so `Pair<String, Integer>.getKey()` reads a `String` and
    `.getValue()` an `Integer` on the same receiver.
  - **A parameterized type carried ONE argument.** `JType` is `Copy`, so the
    list cannot ride inline; the first argument stays inline (every generic
    tracked before had exactly one, and that case remains allocation-free)
    and the rest are interned. A parameterized ARGUMENT is carried too, so
    `Pair<String, Pair<String, Integer>>` reads its inner pair back as a pair
    rather than as `Object`.
  - Bounded and unbounded parameters mix (`<T extends Number, U>`): a bounded
    one still erases to its bound, so its methods resolve, while keeping its
    slot — the slot is the declared position, not a count of tracked ones.
    Generic interfaces implemented with concrete arguments, subclasses of a
    two-parameter class, and two-parameter generic methods all follow.
  - **And the arity is now CHECKED**: `Pair<String>` on a two-parameter class
    used to fall back to the raw type and run. javac's "wrong number of type
    arguments; required N" is reported for a declaration, a field, a nested
    argument (`List<Pair<String>>`) and a `new`. A genuinely RAW use — no
    arguments at all — stays legal, as it is in Java.
  - Pinned by `diff_two_parameter_generic_class`,
    `diff_generic_parameters_in_depth`,
    `diff_generic_interface_and_inheritance` and two `reject_*` tests.
- **A lambda for a USER functional interface** (2026-08-06, JLS §15.27.3) —
  the half of multi-parameter generics that lambdas needed. A user interface's
  own type parameters erase before the lambda pass runs, so
  `Mapper<String, Integer> m = s -> s.length()` typed `s` as a bare type
  variable: "cannot find symbol: method `length()`". The target's WRITTEN
  arguments are now substituted into the SAM by parameter position — which is
  possible only because the erasure sentinel carries that position. The
  synthesized method keeps the ERASED signature (it has to, or it would not
  override the interface's), and each type-variable parameter is cast to its
  real type at the top of the body, the same shape the `Comparator` and
  `java.util.function` lambdas already used. A type-variable RETURN is checked
  against the target's argument too, so `Mapper<String, Integer> m = s -> s`
  is refused as javac refuses it. Mixed concrete and variable parameters
  (`String f(String a, T b)`) specialize only the variable ones, a RAW target
  keeps its erased treatment, and method references, fields, parameters and
  return positions all follow. A cast from a type variable is now an ordinary
  reference cast — it erases to `Object`, so `(Integer) t` is what javac says
  it is. Pinned by `diff_lambda_for_user_generic_interface` and
  `reject_lambda_body_wrong_result_type`.
- **A constant variable may name another one** (2026-08-06, JLS §4.12.4) —
  the last narrowness left by round 9's constant-expressions cluster. The
  member pass folds each `final` initializer once, against the LIBRARY
  constants alone, because the user's own are still being collected as it
  runs; so `static final int B = A + 1;` stayed non-constant, and every rule
  that keys off constness (case labels, array dimensions, inlined reads,
  `==` on folded strings) was wrong about it. Constants now fold to a
  FIXPOINT: each round sees what the previous rounds established, in any
  declaration order and across classes, and the loop stops when a round
  establishes nothing new. A cyclic definition simply never folds — and the
  forward-reference check reports it, exactly as javac does.
  Pinned by `diff_constant_referring_to_constant`.
- **An inner class of a generic outer** (2026-08-06, JLS §4.5/§8.1.3) —
  round 9's last generics deferral, where an earlier erasure-inheritance
  attempt had been reverted as too speculative. An INNER (non-static) class is
  bound to an instance of a parameterized outer, so the outer's type variables
  are in scope in its body — but it has no parameters of its own to hold them,
  and a `T` written inside one was an unknown type name outright. It now
  INHERITS the outer's parameters, ahead of any it declares, so the positions
  line up and a `T` inside `Inner` erases to the same slot it does in `Outer`.
  `o.new Inner()` takes the qualifier's own arguments (in `type_of` as well as
  on the emit path, or a `var` reads the inner's members erased), and
  `Outer<String>.Inner` parses as a type — including `Outer<String>.Inner<U>`,
  whose two argument lists concatenate in exactly the order the parameters are
  inherited. A RAW `Outer.Inner` still reads the outer's variable as `Object`,
  as it does in Java. A static nested class is untouched: it has no enclosing
  instance, so no arguments to inherit.
  Found on the way: `Objects.requireNonNull` narrowed a parameterized argument
  all the way to `Object` — it answers `T`, and the qualifier of
  `o.new Inner()` goes through it. Pinned by
  `diff_inner_class_of_generic_outer`.
- **A generic method's return can be pinned by a CONTAINER argument**
  (2026-08-07, JLS §18) — the return-inference plan matched only a parameter
  that WAS the type variable (`<T> T max(T a, T b)`), so the commoner shape,
  `<T> T max(List<T> xs)`, came back erased: as `Object`, or as the bound for
  a bounded variable, which made `max(listOfStrings).toUpperCase()` a "cannot
  find symbol". The plan now records where each parameter mentions the
  variable — directly, or as a container's single type argument — and the
  call reads the argument's ELEMENT in the second case. A parameterized USER
  class answers its first type argument the same way, so `<T> T open(Box<T>)`
  works too. Arguments that pin different types still keep the erased return.
  Known gap: a DIAMOND in an argument position (`open(new Box<>("k"))`) pins
  nothing, because a diamond types as `Null` so that the declared target stays
  the authority. Pinned by `diff_generic_return_from_container_argument`.
- **A type variable in ELEMENT position** (2026-08-07, JLS §4.5.1) — what
  makes the generic data structures every course writes actually work.
  `List<T> items`, `Iterator<T> iterator()` and `Node<T> next` all erased
  their argument to an anonymous wildcard, so a read through a `Bag<String>`
  receiver came back as `Object`: `for (String s : bag)` would not take the
  element, `b.items.get(0)` had no `String` methods, and a linked node's
  `next` was raw. A new `ElemType::TypeVar` carries the parameter's declared
  POSITION into element slots — seven exhaustive matches, no more — and a
  member's type is substituted through on every read: field, method return,
  and the for-each cursor, on both the emit and the `type_of` path.
  - FIELDS, RETURNS and BODIES erase this way; PARAMETERS deliberately do
    not. Applicability compares a declared parameter against the argument,
    and `addAll(List<T>)` has to keep accepting a `List<String>` without the
    receiver's arguments being in reach there.
  - Three separate gaps had to close alongside it: a cast TO a type variable
    (`(T) items[--n]`, the unchecked cast at the heart of every array-backed
    container) was refused with "cannot cast Object to Object"; a field
    ASSIGNMENT through a parameterized receiver (`top.item = x`) accepted only
    a plain `Object` receiver; and `type_param_count` was filled in the pass
    that resolves members, so a class's own members were resolved while its
    count was still zero — a self-referential `Node<T>` always went raw, and
    whether anyone else's did depended on declaration ORDER.
  - Pinned by `diff_type_variable_in_element_position`, which walks a `Bag`,
    a linked node chain, a linked stack and an array-backed stack.
- **A nested class has a BINARY name** (2026-08-08, JLS §13.1) — the last
  round-8/9 naming deferral, in both its halves. caturra hoists a nested class
  to the top level, and used the SIMPLE name for everything, so every nested
  class lied about itself: `getClass().getName()` and a class literal answered
  `Inner`, a default `toString()` printed `Inner@…`, and a stack-trace frame
  read `Inner.method`. The class file now carries `Outer$Inner` (`A$B$C` for a
  deeper one), which is what all four report; `getSimpleName()` is the one
  that really is the simple name, and strips at the last `$`.
  - The method table is keyed by that binary name, with the SOURCE spellings —
    the simple name and the canonical `Outer.Inner` — as aliases, resolved
    through one `info()` chokepoint. That is what lets a nested `Holder.Node`
    coexist with a top-level `Node`, which used to be refused outright as
    "class 'Node' is already defined". Only the QUALIFIED spelling is answered
    from the alias map: the bare simple name is an alias too, and answering it
    there would let a nested class win over a top-level one.
  - Two resolution orders had to flip with it: `new Outer.Inner()` and a
    declared `Outer.Inner` read the WHOLE name first, where both used to take
    the last segment on sight and build the wrong class — silently, since two
    classes of the same simple name have the same members as often as not.
  - A `ClassId` answers with the binary name, so everything that starts from
    one and ends at a name — descriptors, `extends`/`implements` entries, the
    `throws` and access side tables, the enclosing-statics walk — had to agree.
    Pinned by `diff_nested_class_binary_names` and
    `diff_nested_and_top_level_share_a_name`.
  - Still open: `getStackTrace()` returns no `StackTraceElement[]`, so a frame
    can only be read through `printStackTrace`.
- **A user throwable's hierarchy, and a null message** (2026-08-09, JLS
  §11.1.1) — two more from the same probe:
  - **A user throwable was `instanceof` its parent but nothing above it.** The
    runtime subtype walk climbs class FILES, and a user exception's parent is
    a library throwable with no class file, so the walk stopped there: a user
    `RuntimeException` subclass answered false to `instanceof Exception` and
    to `instanceof Throwable`. The exceptions table knows the rest of that
    hierarchy and is consulted now.
  - **A null message crashed.** `new RuntimeException(null)` and
    `super(null, cause)` are ordinary Java — `getMessage()` answers null and
    `toString()` is the bare class name — but the compiler refused the
    argument ("null cannot be converted to String") and the VM threw an NPE
    from the constructor. Only a THROWABLE takes one: `new String(null)`,
    `new StringBuilder(null)`, `new File(null)` and `new Scanner(null)` still
    throw, which is why the null was being rejected wholesale.
  - Pinned by `diff_user_throwable_hierarchy_and_null_message`.
- **`IntStream.peek` and `new String(char[], int, int)`** (2026-08-09) — two
  library gaps a surface probe turned up. `peek` is an intermediate op the
  object `Stream` had and a primitive one did not; the three-argument String
  constructor takes the used PREFIX of a char buffer, and reports its range
  failure with the JDK's three numbers (`offset o, count c, length n`).
  Known limit, unchanged and honest: the element type is ERASED after `map`,
  so `stream().map(String::toUpperCase).findFirst().get()` answers `Object`
  and assigning it to a `String` is refused. `collect(toList())`, `forEach`
  and `mapToInt` all keep their types; only a pipeline that ends in an
  `Optional` loses it. The same shape refuses a `Supplier<List<String>>`'s
  `get().size()`. Rejects-valid, which is the safe direction.
- **`getStackTrace()` and `StackTraceElement`** (2026-08-11,
  `java.lang.Throwable`) — the trace was recorded and rendered but could not
  be READ: `e.getStackTrace()` was "cannot find symbol", which is what a probe
  hit three separate times. A trace is stored as the lines `printStackTrace`
  prints (`Cls.method(File.java:12)`); those are now taken apart into real
  frame objects answering `getClassName`, `getMethodName`, `getFileName`,
  `getLineNumber` and `toString` — including `(Unknown Source)` for a frame
  with no file, the JDK's wording. `StackTraceElement` is a nameable type, its
  array widens to `Object[]` for `Arrays.toString`, and a frame renders as its
  own text wherever a value is printed. Modelled as one `JType`/`ElemType`
  pair, the shape `Class`, `Field`, `Method` and `Constructor` already use.
  - Found alongside it: `type_of` named only `getMessage` and `toString` as
    the inherited Throwable members, where EMISSION resolves the whole
    exception table. Every other one — `getCause`, `getSuppressed`,
    `getStackTrace` — typed as an error in any position that consulted its
    type, so `e.getStackTrace().length > 0` was "bad operand types" for a
    comparison that emitted perfectly well. Both paths use the one table now.
  - Pinned by `diff_stack_trace_elements`.
- **`type_of` must answer what the emitter builds** (2026-08-11) — the
  divergence swept for as a CLASS, after it turned out to be the real defect
  three separate times. `var x = EXPR;` needs the initializer's type where
  `println(EXPR)` needs only its code, so "cannot infer type for 'var'" on an
  expression that prints fine is exactly the signature. A sweep of 78
  expressions over every receiver kind found seven:
  - `arr.clone()` typed as an error though the comment above the arm said it
    must mirror `array_object_call` — `int[].clone()` is an `int[]`.
  - `Arrays.stream(array)` had no `type_of` rule at all.
  - `new TreeMap<>(m)` / `new HashMap<>(m)` — a COPYING diamond takes its
    arguments from the source, as javac does and as the emitted code already
    did; `var` fell to the `Object` form and then refused the very value being
    assigned. A diamond with no source still takes `Object`.
  - `List.of` / `Set.of` / `Map.of` / `Collections.emptyList()` are typed by
    their context everywhere else, and `var` IS the context: settled from the
    arguments, or `Object` when there are none.
  - A `StackTraceElement` receiver had no `type_of` arm.
  Pinned by `diff_var_infers_what_the_emitter_builds`. Both catalogues now
  report zero divergences.
- **A constant must FOLD to what it evaluates to** (2026-08-11, JLS §5.1.3) —
  the third self-checking invariant, and the same technique as the `var`
  sweep: the folder and the interpreter are two implementations of one
  language, so `static final int C = EXPR;` and `int v = EXPR;` must print the
  same thing, with no external oracle needed. 109 expressions found one:
  `(int) (1.0 / 0.0)` folded to -1 where evaluating it gives
  `Integer.MAX_VALUE`. A floating value saturates STRAIGHT into `int` range;
  it does not pass through `long` first, and the folder's long-then-truncate
  route turned `long`'s saturated maximum into -1. `byte`/`short`/`char`
  narrow from that `int`, so `(byte) 1e10` is `(byte) Integer.MAX_VALUE`.
  This one is worth the invariant: the folder decides case labels, array
  dimensions and inlined reads, so a disagreement is a SILENT wrong answer in
  whichever of the two paths a program happens to take.
  Pinned by `diff_constant_folding_matches_evaluation`.
- **Every collection is an `Iterable`** (2026-08-11, JLS §14.14.2) — the
  fourth self-check, and a different shape from the other three: a Java
  program that asserts its OWN contracts (a view's size agrees with its map's,
  a copy equals its source, a sort is an ordered permutation), so any failure
  is a caturra bug and the JDK only has to confirm the contracts are right.
  Writing it found two:
  - **A generic `<T> int count(Iterable<T> it)` refused every collection.**
    `Iterable` is a synthesized interface here, registered for inheritance,
    and no builtin collection declared it — so the one signature that means
    "anything a for-each can walk" accepted nothing at all. Lists, sets,
    queues, stacks and a map's three views widen to it now, erased as any use
    of the synthesized form is. The map VIEWS also needed adding to the
    reference-conversion arm that lists already had.
  - **A call straight on a copying map diamond** — `new HashMap<>(m).size()` —
    had no receiver type, because the map constructors did not infer from
    their source the way the collection ones do. `new ArrayList<>(l).size()`
    always worked; the map form is now the same.
  Pinned by `diff_collection_contracts_hold`.
- **`Arrays.asList` spreads only a REFERENCE array** (2026-08-11, JLS
  §15.12.4.2) — the same call typed two ways. `T` is never `int`, so a
  primitive array cannot BE the varargs array and `Arrays.asList(int[])` is a
  one-element `List<int[]>`. Emission had that right; `type_of` spread it and
  answered `List<Integer>`, so the two disagreed about one expression — which
  is what the widened `var` catalogue surfaced. Pinned by
  `diff_as_list_primitive_array_is_one_element`.
  The catalogue now stands at 130 expressions with ONE known divergence left:
  `map`/`mapToObj` erases the element, so a collect after one adopts its
  assignment context and cannot be inferred from.
- **A VM-internal error is always a bug** (2026-08-12) — the fifth
  self-check, and the sharpest: if the compiler ACCEPTED a program, the VM may
  never answer "unknown native member", "malformed class" or a `VerifyError`.
  Those say the two halves disagree about what was emitted, which is never the
  program's fault. Sweeping 164 library calls that way found five:
  - **The map compute family returned an UNBOXED value.** `merge`, `compute`,
    `computeIfAbsent` and `computeIfPresent` hand back whatever their lambda
    produced, and a lambda computing `x + y` produces an `int` — returned
    where `Ljava/lang/Object;` is declared, so `Object r = m.merge(...)` died
    with a `VerifyError`. The map stored boxed at rest either way, which is
    why every form that ignored the result worked.
  - **`PriorityQueue.iterator()`** was missing, so a for-each over one was an
    internal error. It walks the HEAP ARRAY, in no particular order — the
    JDK's own words, and caturra keeps that array in the JDK's sift order.
  - **A throwable had no `hashCode`**, so it could not go in a `HashSet`; and
    a LIBRARY throwable could not be a collection element at all
    (`List<RuntimeException>` was "unknown type 'List'").
  - **`stream.toString()`** was an internal error rather than an object's
    text. `toString`/`hashCode` are Object's and do NOT consume the pipeline.
  Pinned by `diff_no_internal_errors_on_accepted_programs`. The sweep now
  reports zero internal errors; the eight remaining refusals are honest ones.
- **A `collect(...)` is typed by its collector, everywhere** (2026-08-11) —
  found by extending the contract self-check to streams and text. Emission
  reads the result type from the COLLECTOR (`joining()` is a String,
  `toList()` a List of the element); `type_of` did not, and fell through to
  the stream table for an `Error`. So a collect used as a RECEIVER or an
  operand had no type at all —
  `xs.stream().collect(toList()).size() == n` was "bad operand types" for an
  expression that printed perfectly well, and the same for `toSet().size()`
  and `joining(",").split(",").length`. Both paths read the collector now.
  The `var` sweep would have caught this had its catalogue included stream
  terminals; the technique was right and the list was short.
  Known limit, unchanged: after `map` the element is erased, so
  `xs.stream().map(f).collect(toList())` still adopts its ASSIGNMENT context —
  the common form — and cannot be used directly as a receiver.
  Pinned by `diff_stream_and_text_contracts`.
- **A sort's comparison SEQUENCE** (2026-08-09, `java.util.TimSort`) — the
  route, not just the destination. WHICH comparisons a sort performs is
  observable whenever the comparator is not a pure function of its arguments:
  one that prints, one that counts, one that throws for a particular pair —
  and the case that matters, an INCONSISTENT comparator, where two correct
  sorts genuinely leave the elements in different orders. caturra merge-sorted,
  which agreed with the JDK on the result and disagreed on the route.
  `Collections.sort`, `List.sort` and `Arrays.sort(T[], c)` now run TimSort's
  small-input path exactly — `countRunAndMakeAscending`, which reverses a
  descending front run in place, then `binarySort`, whose binary search takes
  the RIGHTMOST equal slot and so keeps the sort stable. Verified comparison-
  for-comparison against a real JDK over empty, singleton, sorted, reversed,
  duplicate-heavy and shuffled inputs.
  At 32 elements and up (`MIN_MERGE`) the JDK runs the full TimSort merge
  machinery — run stack, galloping — and caturra keeps its merge sort there,
  so a side-effecting comparator on a long list still sees a different
  sequence. The RESULT is the same for any consistent comparator.
  Pinned by `diff_sort_comparison_sequence`.
- **An exception's own words** (2026-08-09, JLS §11, `java.lang.Throwable`) —
  two bugs found by hand-probing the area the binary-name rename touched
  hardest, neither of them reported by any audit round:
  - **A user exception that overrides `toString()` could not be caught.** An
    in-flight exception is carried as TEXT, and its class is recovered by
    parsing the header — which is `toString()`. An override renders something
    that is not a class name at all, so the exception was renamed out of every
    `catch` clause and escaped the program. The thrown OBJECT knows what it
    is, and is asked now when the text does not name a throwable. The same
    shape as the reflective exceptions that were uncatchable before it: an
    exception's IDENTITY must never be recovered from its display.
  - **`Throwable.toString()` reads the message virtually.** Its body is
    `getLocalizedMessage()`, whose body is `getMessage()`, so a subclass that
    COMPUTES its message names it even when nothing was passed to
    `super(...)` — `new Quiet()` is `Quiet: quiet`, not a bare class name —
    and an override BEATS a stored message. caturra read the stored field, in
    the printed form, in the stack-trace header and in a `Caused by:` line.
  - Pinned by `diff_throwable_message_is_virtual` and
    `diff_custom_tostring_exception_is_catchable`.
  - Known divergence, unmatchable: a `HashMap` keyed by ENUMS iterates in a
    JVM-specific order, because `Enum.hashCode()` is the identity hash. The
    same class of thing as `Set.of`'s salted order — no engine can reproduce
    it, so caturra does not pretend to.
- **A refusal refuses in EVERY position** (2026-08-06, round 9) — the
  round's cross-cutting root cause, made an invariant rather than a fourth
  patch. `JType::Error` only ever arises from a problem, and every problem is
  meant to have been REPORTED by whoever produced it. A diagnostic that only
  the EMITTING path produces breaks that: `type_of` answers `Error` quietly,
  the enclosing call emits nothing at all, and the program compiles with a
  hole in it — `Collections.reverse(list.subList(1, 5))` printed an unchanged
  list, `box.get() + box.get()` printed nothing whatsoever, a diamond copy
  constructor in an argument emitted its argument and no call. Every place an
  expression gives up because a subexpression typed `Error` now checks that
  something HAS been reported, and says so if not. A wrong message is a
  nuisance; a missing one is a wrong answer with no way to notice it. The
  guard never fires across the 2,600-program grading corpus, and a matrix of
  every refused member (42) in nine positions, plus twelve unmodelled classes
  in eight, found no remaining hole. Pinned by three
  `stricter_*_position` tests.
- **A number's width is carried, not assumed** (2026-08-06, round 9, JLS
  §5.1.3) — the numeric-corners cluster, all ten findings:
  - **Which primitive pipeline a source produces is the ELEMENT's own
    width.** `Arrays.stream(new double[] {1.5, 2.5, 3.5}).sum()` printed
    `0`, a silent wrong answer: every primitive source was typed
    `IntStream`, so `sum()` got an `()I` descriptor and read a double
    pipeline as ints. The kind now follows the element, and the
    shape-preserving intermediates (`filter`, `sorted`, `distinct`,
    `limit`, `skip`) answer the RECEIVER's own kind rather than a fixed
    `IntStream` — the three primitive streams share one method table here.
  - **The primitive-to-primitive conversions** — `asLongStream`,
    `asDoubleStream`, `mapToInt`/`mapToLong`/`mapToDouble` — with the JDK's
    per-kind availability: an `IntStream` widens to long and double, a
    `LongStream` only to double, and a `DoubleStream` narrows to neither.
    `as…Stream` really WIDENS each element; treating it as a pure retyping
    left an `Integer` to reach a `long` lambda, a `ClassCastException`.
  - **A lambda after an inline array** was refused outright ("only allowed
    where a functional-interface type is expected"): the element type was
    read from a variable's declaration, and `Arrays.stream(new int[] {…})`
    names no variable. And `map` on a primitive pipeline keeps the
    element's width — only an object stream's `map` erases to `Object`.
  - **A wrapper's `compareTo` takes its OWN type.** Reached through a raw
    `Comparable`, comparing an `Integer` with a `Long` (or a `String`, or
    any `Object`) is a `ClassCastException` with the JDK's two-class
    wording, not a silent numeric comparison — which is also what a
    `TreeMap`/`Collections.sort` over mixed wrappers must throw.
  - **Every narrowing primitive conversion is "possible lossy
    conversion"**, including the pairs among `byte`/`short`/`char` and
    from `long`. `byte b = aLong;` used to say "long cannot be converted
    to byte", which is what javac says about unrelated REFERENCE types.
    The same wording now covers an ARGUMENT: with exactly one candidate of
    a name and arity, javac blames the argument, so `"abc".charAt(aLong)`
    reports the lossy conversion rather than "no suitable method found".
  - **`void.class`**, and every wrapper's `TYPE` as the PRIMITIVE class it
    wraps (`Integer.TYPE == int.class`, and not `Integer.class`); the
    exponent, code-point and surrogate boundary constants; and
    `Character.reverseBytes` plus `codePointCount` over any
    `CharSequence`.
  - Pinned by `diff_primitive_stream_kinds`, `diff_wrapper_compare_across_types`,
    `diff_wrapper_boundary_constants`, `diff_primitive_class_literals`,
    `diff_character_byte_and_code_point_api` and three `reject_*` tests.
- **Modifiers: the inert ones parse, the illegal ones are refused**
  (2026-08-05, round 9, JLS §8.1.1/§8.4.3) — the modifiers cluster, all nine
  findings:
  - **`transient`, `volatile`, `synchronized` and `strictfp` on a member**
    used to make the whole member UNPARSEABLE ("expected a type"). They are
    ordinary Java with nothing to do here — serialization and threading are
    not modelled, and `strictfp` is the default since Java 17 — so they parse
    and are ignored. So does `final` on an enhanced-`for` variable.
  - **Seven checks javac makes and caturra did not**: a class may not extend
    a `final` class; an `abstract` method has no body and cannot be `final`,
    `static` or `private`; a class cannot be both `abstract` and `final`; an
    interface field may not be `private` (it is implicitly public); and a
    blank `static final` assigned in two static initializers is assigned
    twice — the initializers are one program in source order.
  - An enum is deliberately NOT marked final, though JLS §8.9 makes it so
    unless a constant has a class body: caturra desugars such a body into a
    subclass of the enum, and marking it final would refuse the desugaring's
    own output.
  - Pinned by `diff_inert_modifiers` and seven `reject_*` tests.
- **A `StringBuilder` is a reference like any other** (2026-08-05, round 9)
  — the stringbuilder cluster:
  - **A builder could not be an ELEMENT.** `List<StringBuilder>`,
    `StringBuilder[]` and `Box<StringBuilder>` were each refused with a
    message that was false about the program — "unknown type 'List'",
    "arrays are not yet supported by caturra". The cause was structural: a
    collection or array element is an `ElemType`, and there was no builder
    kind. One new variant covers all three, and the array is covariant into
    `Object[]` like every other reference array.
  - **A builder is a `CharSequence`**, so `new String(sb)` and
    `String.join(d, sb, …)` take one; and it inherits `Object`'s
    `equals`/`hashCode`, so two builders holding the same text are UNEQUAL
    and a builder's hash is stable for its lifetime — `hashCode()` was simply
    a missing entry.
  - **The out-of-range messages**, each recorded from a real JDK: a builder's
    `getChars` words the SOURCE failure `start … end … length` and the
    DESTINATION one as a plain `IndexOutOfBoundsException` over the
    destination's range, while a STRING's words the destination
    `offset … count … length`; a builder's one-argument `substring` is
    `substring(start, count)`, so it reports the pair rather than String's
    "String index out of range". A null `getChars` destination is an ordinary
    NPE — it used to abort the run with an internal error.
  - Deferred: a lone (unpaired) surrogate written as `\uXXXX` still becomes
    U+FFFD — the lexer decodes an escape into a Rust `char`, which cannot
    hold one; the same representational limit as `%c` above.
    `java.lang.StringBuffer` is refused BY NAME rather than aliased to
    `StringBuilder`, which would make `getClass()` lie about which one the
    program built.
  - Pinned by `diff_string_builder_as_a_reference` and
    `diff_builder_range_messages`.
- **An implementation is checked against the INTERFACE it implements**
  (2026-08-05, round 9, JLS §9) — the interface-members cluster, all eleven
  findings:
  - **The override rules stopped at the superclass chain.** An implementation
    that contradicted an interface method — a different RETURN type, WEAKER
    access (including one inherited from a superclass, which javac blames on
    the superclass's method), a BROADER `throws` — compiled, and then either
    ran the wrong body or aborted the run with an internal error. All three
    are checked now, with javac's "cannot implement" wording. Two rules the
    check has to know: an interface member is implicitly `public` whether or
    not the word is written, and caturra's own SYNTHESIZED library interfaces
    (`Iterable`, `Comparable`, `AutoCloseable`) carry approximate signatures,
    so only the access rule applies to them — checking a return type against
    `Iterable.iterator()`'s erased one would refuse ordinary Java.
  - **`Iface.super.m()` did not parse at all** — the standard way to pick one
    of several inherited defaults. It is a name path followed by `super`, and
    emits a non-virtual call to that interface's default.
  - **A name inherited from BOTH a superclass and an interface is
    ambiguous.** The check existed, but constant FOLDING reached the
    interface's constant first and printed a value; the folder now declines a
    name with more than one declaration.
  - **`Class.isInterface()`/`isEnum()`/`isPrimitive()`/`getInterfaces()`**, a
    cast to a PARAMETERIZED user interface (which casts by its erasure, as
    javac's unchecked warning says), a duplicate member reported as being in
    "interface I" rather than "class I", and `super` inside a default method
    reported as javac's missing symbol rather than a missing superclass.
  - Pinned by `diff_interface_super_and_reflection` and five `reject_*` tests.
- **A lookup takes `Object`, and an immutable view is still a collection**
  (2026-08-03, round 9) — the equals/hashCode cluster:
  - **`Map.get`/`containsKey`/`remove`/`getOrDefault` and
    `Collection.contains`/`indexOf`/`lastIndexOf` take `Object` in Java**, not
    the collection's own element type: `map.get(somethingElse)` compiles and
    answers null. caturra demanded the element type — one of the few places
    it was stricter than javac in a way a student meets by accident. A new
    `BParam::Probe` accepts any reference.
  - **A `Queue`/`Deque` face has the `Object` methods** (`q.equals(q)` did not
    compile) and `removeFirstOccurrence`/`removeLastOccurrence`.
  - **A `String` and the wrappers are `Comparable`**, so they assign to a
    `Comparable<T>` variable, raw or parameterized.
  - **`AbstractSet.equals` asks THIS set.** It is `size == size &&
    containsAll(other)`, so a `TreeSet`'s own COMPARATOR decides — caturra
    asked the other set, so a case-insensitive `TreeSet` was unequal to a
    `HashSet` holding the same element in another case.
  - **`Objects.hash(null)` is 0**, not the empty-array 1.
  - **Java 9's `List.of`/`Set.of`/`Map.of`** build immutable collections that
    reject nulls (and duplicates, with the JDK's message). Reading THROUGH an
    immutable wrapper also had gaps that surfaced here: a wrapper compared
    unequal to an ordinary map because `equals` recognised only the concrete
    map kinds and read values without unwrapping. Documented divergence: a
    JDK randomizes the ITERATION ORDER of `Set.of`/`Map.of` per JVM run (they
    are salted — two runs of the same program disagree), so caturra iterates
    in the order written and no engine can match a JDK there.
  - Deferred: `HashMap` iteration order once a bucket is deeply collided, and
    a live `Map.Entry` whose hash changed while it sat in a `HashSet` — the
    same hazard for a USER class already matches the JDK exactly.

    Measured 2026-08-23, because the description above used to be wrong in
    both of its details. The divergence begins at the ELEVENTH key in one
    bucket, not the eighth: a bin reaching eight makes a JDK RESIZE while the
    table is smaller than 64 and treeify only once it is not, which with an
    empty map and nothing but collisions lands on the eleventh insertion. And
    the order is not "by hash then `compareTo`" — it is the red-black tree's
    own linked order, which depends on the insertion sequence that built it.
    caturra keeps the chain (insertion order) however deep the bucket gets.

    Reaching that needs crafted collisions or a constant `hashCode`. For a key
    class that is not `Comparable` the JDK's order there depends on identity
    hashes — addresses, which no other implementation can reproduce — so the
    part that COULD be matched is the `Comparable` case, and matching it means
    porting `HashMap$TreeNode` whole. The ten-key case, which is the one a
    program actually meets, is asserted against the JDK itself.
    (`a_collided_bucket_iterates_alike`,
    `a_deeply_collided_bucket_keeps_insertion_order`)
  - Pinned by `diff_lookup_takes_object`,
    `diff_tree_set_equals_uses_its_comparator` and
    `diff_immutable_factories`.
- **The sorting contracts** (2026-08-03, round 9) — the sorting cluster, all
  twelve findings:
  - **`Arrays.rangeCheck`, in the JDK's order.** A reversed range is an
    `IllegalArgumentException` naming both bounds; an out-of-range one throws
    BEFORE anything moves — caturra sorted first and complained afterwards,
    leaving a half-sorted array. The message is
    `new ArrayIndexOutOfBoundsException(index)`'s, which names the index
    alone ("Array index out of range: 9"), so the three JDK exceptions with
    an int constructor now have one here too.
  - **Every `Arrays.sort` range overload exists** (the primitives, a
    reference array, and both with a `Comparator`), and
    **`Arrays.sort(Object[])`** is accepted as javac accepts it — throwing
    `ClassCastException` at run time when an element is not `Comparable`,
    rather than refusing at compile time.
  - **`binarySearch` uses the JDK's own loop** — a CLOSED range with
    `mid = (low + high) >>> 1`. A half-open loop finds a different one of
    several EQUAL elements, and the index is observable.
  - **An immutable list of 0 or 1 elements can be sorted** (those classes
    override `sort` to do nothing, since nothing can move), and **a
    comparator that modifies the list being sorted is a
    `ConcurrentModificationException`** rather than a silent discard of what
    it added.
  - **A user `Comparator` inherits the interface's default combinators.**
    `myComparator.reversed()` looked for a `reversed` method on the user's own
    class and aborted the run; it now builds the same derived comparator the
    factories return. The cause ran deeper: a class file recorded
    `implements Comparator` under the SOURCE name while everything else used
    the aliased `__Comparator`, so `x instanceof Comparator` was false for a
    class that plainly implements it. Both spellings agree now.
  - **A method reference stands wherever a `Comparator` is expected** — a
    variable, a field, or `list.sort(Cls::byX)` — and
    `thenComparing(keyExtractor, keyComparator)` (the two-argument form) is
    accepted, desugaring like the two-argument `comparing` factory.
  - Pinned by `diff_array_sort_range_checks`, `diff_binary_search_duplicates`
    and `diff_sorting_contracts`.
- **The `Collections` utilities take the types Java declares**
  (2026-08-03, round 9) — the collections-utility cluster:
  - **`max`/`min`/`frequency`/`disjoint`/`addAll` are declared over
    `Collection`**, so a `Set` (or a `Collection`-typed variable) is a legal
    argument; the rest are declared over `List`, which a `LinkedList`- or
    `Stack`-typed variable also is. caturra demanded a statically-`List`
    argument for all of them, and the VM's own read of the argument saw only
    list-shaped objects — so a `Set` passed to `frequency` answered 0 rather
    than counting.
  - **Methods that were missing**: `replaceAll`, `indexOfSubList`,
    `lastIndexOfSubList`; and `nCopies(n, null)` (a list of nulls, typed from
    its context like the empty factories) was refused with a reason that is
    false about Java.
  - **The immutable views**: copy-constructing from one
    (`new ArrayList<>(Collections.emptyList())` was a `ClassCastException`,
    because the heap-only arm read a list's own vector), printing one in a
    concatenation (an inference-placeholder type appended through the
    `String` overload and aborted the run — `Null` now appends as an
    `Object`, which renders "null" for a real null and the text for the
    rest), and the out-of-range MESSAGE, which each JDK view words its own
    way: `nCopies`/`singletonList` are `AbstractList`s (`Index: 2, Size: 2`),
    the shared empty list reports the index alone (`Index: 0`), and
    `Arrays.asList` indexes its array, so the exception CLASS is
    `ArrayIndexOutOfBoundsException`. Each was recorded from a real JDK.
  - **An unsupported member named in an ARGUMENT is now reported.**
    `Collections.reverse(list.subList(1, 5))` compiled to NOTHING and printed
    an unchanged list: the refusal a direct `list.subList(...)` gives was
    produced only by the emitting path, so `type_of` answered `Error`
    silently, and the enclosing call assumed the argument had reported its
    own problem. This is the same silent-non-emission shape as the diamond
    argument recorded above.
  - Deferred at the time: `subList` was not a live view (it was refused, by
    that name, on every list face) — it is one now, see **subList as a live
    view** — and `java.util.Enumeration`, with it
    `Collections.enumeration`/`list`, was refused by name rather than
    reported as a missing symbol. All three are built now; see **The
    collections that came first**.
  - Pinned by `diff_collections_over_any_collection` and
    `diff_collection_view_messages`.
- **Inner classes: the qualifier, the chain, and the name**
  (2026-08-03, round 9, JLS §8.1.3/§15.9) — the inner-class cluster:
  - **`p.new Inner()` null-checks its qualifier BEFORE the arguments**
    (§15.9.4). caturra passed the qualifier through as the leading
    constructor argument with no check, so `null.new Inner(side())` built an
    object with a null enclosing instance AND ran the side effect a JDK never
    runs.
  - **A constructor that delegates threads the enclosing instance on.**
    `Inner() { this(99); }` did not pass `__caturraOuter` to the constructor
    it delegated to, so the call matched the constructor's OWN new signature —
    reported as a recursive constructor invocation. The capture pass records
    this exact trap for its captured values; the inner pass had it too. An
    inner class extending another inner class needs the same for `super`,
    including the implicit one codegen synthesizes.
  - **An INHERITED inner class** is instantiable with a bare `new Inner()`
    from the subclass — `this` is an instance of the enclosing type, so the
    binding rule is subtyping, not identity.
  - **A qualified nested type NAME** (`Host.Point`) works as a return type, a
    parameter, a type argument, an array element and a `new` — it used to read
    as a PACKAGE ("package Host does not exist"). Nested classes are hoisted
    under their simple name, so only the qualifier had to be checked against
    the recorded enclosing chain. The array form also needed the declaration
    lookahead, which could not tell `Host.Point[] a` from an index expression.
  - **The capture pass now walks the QUALIFIER** of `o.new Inner()`. Every
    walk visited only the arguments, so a local named in one was never
    captured and `() -> o.new Inner()` failed with "cannot find variable 'o'".
    A name path containing `this` (`Outer.this`) is not a variable and is not
    captured — its head is a class name.
  - Found while testing, not by the audit: **an anonymous class in a FIELD
    INITIALIZER never received its captured values.** Field initializers were
    visited when COLLECTING captures and not when passing them, so the
    commonest anonymous-class shape of all — `Act r = new Act(){ … tag … };`
    as a field — failed with "constructor Anon$1 cannot be applied to given
    types".
  - Deferred: two outer classes may not each declare a nested class of the
    same simple name (hoisting uses simple names — the same limit as the
    nested-class binary names recorded above), and an inner class of a GENERIC
    outer cannot name the outer's type variable.
  - Pinned by `diff_qualified_new_null_check`, `diff_inner_class_shapes` and
    `diff_anonymous_class_in_field_initializer`.
- **A wrapper type argument survives erasure** (2026-08-02, round 9,
  JLS §18) — the generic-inference cluster:
  - **`Node<Integer>.get()` typed as `Object`.** Only `String` and class-typed
    type arguments were tracked on a parameterized user type; a WRAPPER one
    was dropped, so the class went raw. That refused `Integer y = node.get()`
    outright and — the dangerous half — made `p(node.get())` silently pick
    the `p(Object)` overload where javac picks `p(Integer)`. Wrapper
    arguments are now tracked (fields and collections hold boxed references,
    so `T get()` really does hand back an `Integer`), and the substituted
    type is the BOXED one a declared `Integer` resolves to — handing back a
    differently-shaped element kind put a value in a local that printed
    `true` as `1`.
  - **A generic METHOD's inferred return applies in `type_of` too**, not just
    when emitting: `p(id(5))` for `<T> T id(T)` chose `p(Object)` because the
    enclosing overload resolution asked for the argument's type and got the
    erased one. The two paths disagreeing is the same trap this file records
    for `box.get() + box.get()`.
  - A synthesized library interface (`Comparable<Integer>`) stays ERASED, so
    a boxed value still assigns to it; only classes the program declares
    track a wrapper argument.
  - **Java 11 `(var s) -> …` lambda parameters** parse.
  - Two internal-looking refusals now say what they mean: a functional
    interface parameterized on a METHOD's own type variable
    (`<T> void run(T t, Consumer<T> c)`) leaked the erasure sentinel as
    `unknown type ' Wildcard ='`, and extending a builtin collection
    (`new ArrayList<>() { … }`) claimed the class did not exist rather than
    saying caturra's collections are VM objects with no class to inherit
    from.
  - Deferred: a generic type with TWO type parameters substitutes neither
    (`JType::Generic` carries one argument); inference from the assignment
    TARGET (`List<String> l = box("hi")`); a lambda whose target is a generic
    method's `Function<T,R>` parameter; an explicit type witness on a
    constructor (`new <Integer>Box<String>(…)`); and javac's rejection of a
    call whose inference variable has incompatible equality constraints.
  - Pinned by `diff_generic_wrapper_substitution` and
    `diff_var_lambda_parameters`.
- **Throwable's own contract, and what a catch parameter's type is**
  (2026-08-02, round 9, JLS §11) — the catch-semantics cluster:
  - **A cause is set ONCE.** `initCause` on an exception that already has one
    throws `IllegalStateException: Can't overwrite cause with …` rather than
    silently replacing it; self-causation is refused too.
  - **`getLocalizedMessage()` is `getMessage()`.** The JDK's default body IS
    `return getMessage();`, so an override of `getMessage` answers for both.
    caturra read the stored `__message` field, so a subclass that COMPUTES
    its message returned the raw one from `getLocalizedMessage`.
  - **`super.getMessage()` / `super.toString()` inside a user exception now
    compile.** Its superclass is a library throwable with no entry in the
    class table, so `super` was refused as "X has no superclass" — on the
    single most common thing to write in an exception subclass.
    `super.toString()` reads the message VIRTUALLY, as `Throwable.toString`
    does, so an overridden `getMessage` shows through it.
  - **A multi-catch parameter's type is the least upper bound of its
    alternatives**, not `Throwable`: `catch (A | B e)` over two subclasses of
    `Base` can call what `Base` declares. Every such call was refused.
  - **Three validations javac makes and caturra did not**: precise rethrow
    requires an effectively final parameter (assigned, `throw e` throws the
    DECLARED type, which the method must report — this let an under-declared
    `throws` compile); every name in a `throws` clause must be a Throwable;
    and a generic class may not extend Throwable. A `catch` of a real
    non-throwable type now names it as javac does rather than claiming the
    class does not exist.
  - Deferred: a lambda BODY may still throw a checked exception its
    functional interface does not declare — the synthesized lambda class
    carries no `throws` contract, so the check has nothing to compare
    against; the program fails at run time with that exception rather than
    at compile time.
  - Pinned by `diff_throwable_contract`, `diff_multi_catch_lub`,
    `reject_rethrow_of_assigned_parameter`, `reject_throws_non_throwable`,
    `reject_generic_throwable` and `reject_catch_non_throwable`.
- **A format argument is rendered by the VM, not the call site**
  (2026-08-02, round 9, `java.util.Formatter`) — the format-conversions
  cluster:
  - **A varargs RELAY formatted every heap object as the EMPTY STRING.**
    `static void log(String fmt, Object... a) { String.format(fmt, a); }` —
    the shape every logging helper has — printed `enum=`, `obj=`, `list=`.
    The compiler coerces a directly-written argument to text at the call
    site, which is why a plain `format("%s", obj)` was right; an argument
    arriving through an `Object...` array has no static type left to coerce,
    and the formatter sees only the heap, so it cannot call a user
    `toString()`. The VM now renders the objects among a format call's
    arguments before formatting (into a FRESH array — the caller's own must
    not be rewritten), leaving strings, wrappers and nulls alone. A primitive
    array and a `Class` are arguments too, and print as a JDK prints them
    (`[I@1b6d3586`, `class java.lang.String`).
  - **The specifier grammar and its validations**, each recorded from a real
    JDK: an out-of-range width is IGNORED, not clamped (clamping built a
    2-billion-character string); a `.` with no digits after it ENDS the
    specifier, and the JDK reports such a malformed specifier against the
    character right after the `%` (`%5.d` says `Conversion = '5'`); `,` is
    illegal for the scientific conversions and is reported with the
    LOWERCASE conversion (`%,E` says `Conversion = e`); `%n` and `%%` take no
    flags; a null argument ARRAY answers null for EVERY specifier, so
    `format("%s %s", (Object[]) null)` prints `null null`; and
    `format(null, …)` throws an NPE with no message.
  - **`Locale.US`** (and `ROOT`/`ENGLISH`/`UK`/`CANADA`) names the locale
    caturra always formats in, so the argument is accepted and dropped. A
    locale that formats DIFFERENTLY is refused rather than silently ignored,
    which would be a wrong answer.
  - Two adjacent bugs found while testing this, neither reported by the
    audit: a DIAMOND constructor passed to a varargs parameter was passed AS
    the array (caturra types a diamond it cannot infer as `Null`, and `null`
    is assignable to the array — only a real null takes the array form now);
    and a map COPY constructor understood only a real `HashMap` source, so an
    immutable view was a `ClassCastException` into a `HashMap` and silently
    NOTHING into a `TreeMap` — an empty map where the JDK gives the copy.
  - Deferred: `%c` of an unpaired surrogate yields U+FFFD, because the
    formatter is `String`-based end to end and Rust's `String` cannot hold
    one; `printf` chaining is refused with an honest reason (caturra models
    the print calls as statements and cannot carry the stream as a value).
  - Pinned by `diff_format_object_relay`, `diff_format_specifier_rules`,
    `diff_format_us_locale`, `diff_varargs_diamond_argument` and
    `diff_map_copy_constructors`.
- **A constant expression is more than a literal** (2026-08-02, round 9,
  JLS §15.29/§4.12.4) — the constant-expressions cluster, ten findings with
  one cause:
  - **A `final` variable initialized with a constant EXPRESSION is a
    constant variable**, whose reads javac inlines. caturra recognised only
    bare literals, so `static final int MODE = 1 + 1;` was not one — and
    three unrelated rules key off that. A `case MODE:` label was refused
    outright; READING the constant ran its class's static initializer, which
    a JDK never does (the value is inlined and the class is never touched);
    and `while (Cfg.DEBUG)` over a false constant in ANOTHER class missed the
    unreachable-statement error, because the flow pass sees one class at a
    time and could not resolve the name.
  - **One folder, in `constfold.rs`**, now answers for all of them: literals,
    the `java.lang` constants (`Integer.MIN_VALUE`, `Math.PI`), constant
    variables, unary and binary operators under Java's numeric promotion,
    shifts masked to 5 or 6 bits, casts, string concatenation, and constant
    conditionals. Name resolution is the CALLER's, because the folder runs
    both while the method table is being built (only library names resolve
    then) and from codegen (where a local, a field, or another class's
    constant does). Integer arithmetic folds in the exact Java width, so a
    wrapping `int` overflow gives the value a JDK gives.
  - **A case label of the wrong TYPE is a type error**, naming both types as
    javac does (`char cannot be converted to String`), not the previous
    complaint about constness.
  - Known gaps: `static final int B = A + 1;` referring to another user
    constant is still not folded (they are collected in one pass, so the
    dependency is not resolved), and a concatenation involving a floating
    value is left un-folded rather than rendered. Both make FEWER things
    constant, which is the safe direction. Deferred: a `switch` over a type
    VARIABLE (`<T extends E>`) is accepted where javac refuses it — caturra
    erases a type variable to its bound before codegen, so the selector is
    indistinguishable from one of the bound's own type.
  - Pinned by `diff_constant_expressions`,
    `reject_unreachable_over_other_class_constant` and
    `reject_case_label_type_mismatch`.
- **A program's own class can be iterated** (2026-08-02, round 9, JLS
  §14.14.2) — the custom-iterable cluster, open since round 4:
  - **`implements Iterable<T>` / `implements Iterator<T>` were refused**
    ("cannot find symbol: class Iterable"), so a user data structure could
    not be walked by a for-each at all. Both are now interfaces in the method
    table, registered for INHERITANCE only: the names still resolve to
    caturra's builtin cursor and collection types, because `list.iterator()`
    must keep yielding one, and only a class the PROGRAM declares takes a
    builtin name over. A for-each over an `Iterable` compiles to the JLS
    translation — `iterator()`, then `hasNext`/`next` — with the element type
    read from what the class's own `iterator()` returns, so
    `for (int x : range)` unboxes. The cursor calls are emitted against
    `java/util/Iterator` and the VM dispatches them on the receiver's actual
    class, so a user iterator and a builtin one both answer.
  - **A cursor is an object.** It assigns to `Object`, casts back down,
    answers `instanceof Iterator`, and has the `Object` methods —
    `getClass()` on any iterator used to claim `java.lang.Object`, and
    `toString()` aborted the run. The JDK's iterator class names are
    observable and differ per collection (`ArrayList$Itr` vs `ArrayList
    $ListItr` vs `HashMap$KeyIterator` vs `Arrays$ArrayItr`); each name
    caturra reports was recorded from OpenJDK 11. Known gap: a cursor built
    at a read-only wrapper keeps only the backing collection, so the four
    `Collections`/`AbstractList` cursor classes cannot be told apart.
    Likewise `x instanceof Iterable` is true for every collection.
  - **A method reference stands wherever a lambda does** in
    `forEach`/`forEachRemaining`/`removeIf`/`replaceAll`; these were refused
    with the false claim that the position is not a functional-interface one.
  - **A method call on a DIAMOND copy constructor, in an argument position,
    compiled to nothing.** `q(new HashSet<>(src).size())` emitted its
    argument and then no call — the enclosing call silently failed to
    resolve, with no diagnostic, because the diamond's type was known only
    while emitting and `type_of` answered `Null`, making the call on it an
    `Error`. A diamond now takes its element from its copy source, as Java's
    inference does. This was found while testing the cluster, not reported by
    the audit.
  - Known gap: `java.util.AbstractList` and the other `Abstract*` skeletons
    are refused BY NAME (extending one means inheriting a dozen concrete
    methods written in terms of the subclass's `get`/`size`), instead of the
    previous claim that a real java.util class does not exist.
  - Pinned by `diff_custom_iterable`, `diff_iterator_is_an_object`,
    `diff_diamond_copy_receiver_in_argument` and
    `diff_method_ref_to_collection_consumers`.
- **A resource is closed EXACTLY ONCE** (2026-08-02, round 9, JLS §14.20.3)
  — the try-with-resources cluster:
  - **Leaving the body by `return`/`break`/`continue` closed the resource
    TWICE.** The cause was not in the desugaring but in how every `finally`
    is compiled: the copy an abrupt exit inlines sat inside the protected
    range of the try it was leaving, so when that copy threw, the try's own
    handler caught it and ran the finally again — a second `close()`, whose
    failure was then self-suppressed into the first (`getSuppressed().length`
    1 where a JDK says 0). Silent until `close()` throws or is not
    idempotent, and it applied to any `try`/`finally`, not just resources. A
    protected region is now a LIST of intervals with the inlined copies cut
    out. The exclusion is per guard, not global: a copy is excluded from its
    OWN try and everything nested in it, but stays covered by the tries
    outside it — which is exactly where an exception from a `finally` does
    surface, and where the enclosing `catch` must still see it.
  - **A try-with-resources nested in another's BODY now compiles.** Its
    synthetic locals live in the enclosing method's scope, so the
    per-resource index was not enough to keep two statements apart and
    `__caturraPrimary$0` collided — a compile error naming an internal
    symbol, on the canonical two-resource idiom. Sequential statements were
    unaffected, which is why it went unnoticed.
  - **A resource must BE an `AutoCloseable`**, not merely have a `close()`
    method. Because the statement is desugared in the parser, before any
    type is known, the desugaring marks one of its two generated close calls
    with a reserved name and codegen checks the receiver's type there.
    Only class types are judged, so a builtin resource is never wrongly
    refused.
  - **The rest of the resource surface**: a FIELD ACCESS is a resource
    (`try (t.field)`, `try (this.out)`, Java 9), read once where it is
    named; `AutoCloseable` is a functional interface, so a lambda is a
    resource; `java.io.Closeable` exists and IS-A `AutoCloseable`; and a
    class may inherit `close()` as a DEFAULT from an interface instead of
    declaring it (the abstract-method search walked only the class chain,
    never the interfaces). Known narrowness: caturra does not require an
    existing-variable or field resource to be effectively final, which javac
    does.
  - **An exception cannot suppress itself** — `addSuppressed(this)` throws
    `IllegalArgumentException: Self-suppression not permitted` rather than
    building a cycle, and `addSuppressed(null)` compiles and throws NPE.
  - Pinned by `diff_twr_close_once_on_abrupt_exit`,
    `diff_finally_copy_is_not_self_caught`, `diff_twr_nested_in_body`,
    `diff_twr_field_resource`, `diff_self_suppression_refused`,
    `diff_closeable_shapes` and `reject_resource_not_autocloseable`.
- **A read-only view stays read-only THROUGH its cursor** (2026-07-31,
  round 8) — the collection-views cluster, and the round's most dangerous
  finding:
  - **`Collections.unmodifiable*` was not read-only.** Every direct mutator
    threw, but the ITERATOR was a hole: `unmodifiableList(data).iterator()
    .remove()`, a `ListIterator`'s `set`/`add`/`remove`, and a
    `Map.Entry.setValue()` reached through an `unmodifiableMap`'s entrySet
    all wrote STRAIGHT INTO the backing collection, with no error. So did
    `Arrays.asList(a).iterator().remove()` — which made a `String[3]` appear
    to become length 2 — and the `singletonList`/`singleton`/`singletonMap`
    cursors, which emptied collections that can never change size. A
    defensive `return Collections.unmodifiableList(data);` protected
    nothing. The cause was structural: a view UNWRAPPED to its backing
    collection before dispatch, so the cursor it built knew only the
    backing. Cursors now carry what they may write (`IteratorWrites`) and
    are built at the wrapper, not through it; entries from a read-only view
    carry the same flag, so `setValue` throws however the entry was reached
    (iterator, for-each, or `removeIf`).
  - **The JDK's four cursor shapes are distinguished**, because they refuse
    differently and the exception CLASS is observable. A view's own cursor
    (`unmodifiable*`, `singleton*`) throws `UnsupportedOperationException`
    whatever its state; a GENERIC cursor (`AbstractList`'s, behind
    `nCopies`; the shared `EmptyIterator`, behind `empty*`) checks its own
    state first, so a `remove()` with no `next()` is an
    `IllegalStateException`; `Arrays.asList(a).listIterator()` is that
    generic cursor over a FIXED-SIZE list, so its `set` writes through to
    the array while `add`/`remove` refuse; and `Arrays.asList(a).iterator()`
    is JDK 9's `ArrayItr`, which has no `remove` at all and so throws
    `UnsupportedOperationException: remove`, message included. Known gap:
    `singletonList().listIterator()` is the generic cursor in a JDK, so a
    misplaced `set` there is an `IllegalStateException` where caturra says
    `UnsupportedOperationException` — the wrapper does not record which
    factory built it.
  - **A map's three views are CACHED**, one per map per kind, as a JDK keeps
    them in fields. This is observable: `m.values() == m.values()` is true,
    and a `values()` view's `equals` is `AbstractCollection`'s identity, so
    allocating a fresh view per call made `m.values().equals(m.values())`
    wrongly false. `keySet()`/`entrySet()` compare and hash as Sets instead
    (`AbstractSet`: same size and every element held, the hash their sum,
    an entry's being `key.hashCode() ^ value.hashCode()`).
  - **The view and cursor faces carry their whole surface**, each of which
    had been refused with a reason that was false about the JDK:
    `iterator()` on a `Queue`/`Deque` (both extend `Collection`, and it is
    the only way to remove from the middle of one), `listIterator(int)`,
    `Iterator.forEachRemaining`, `entrySet().removeIf`/`forEach` (the only
    form that decides by key AND value together, so its lambda takes a live
    `Map.Entry`), `Collections.unmodifiableCollection`, and the
    `equals`/`hashCode` that every reference type has.
  - Pinned by `diff_unmodifiable_views_refuse_their_cursors`,
    `diff_immutable_factories_refuse_their_cursors`,
    `diff_fixed_size_list_cursors` and `diff_view_and_cursor_surface`.
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
  A COVARIANT return (`String f()` overriding `Object f()`) was refused here,
  on the reasoning that dispatch is by descriptor and so it needs a bridge that
  cannot be written in source. That was superseded on 2026-08-14: the VM
  resolves an override by name and arity, so no bridge is involved, and the
  dispatch corners are pinned by `differential_test!`. See "A covariant return,
  and a lambda after a qualified call".
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
  object. A lambda nested in another one reaches the same instance and the
  same locals (see **Nested capture**); capturing a `StringBuilder` is still
  a compile error (javac accepts it — the safe direction). Pinned by
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

### `fillInStackTrace`, and what a `Class` will not say (2026-08-12)

`Throwable.fillInStackTrace()` re-records the trace AT THE CALL and returns the
receiver itself, so a throwable rethrown from elsewhere can be made to point at
the rethrow rather than at its construction. Its own frame is hidden exactly as
a constructor's is.

`Class.getModifiers()` is **refused, on purpose**. It could not be answered
honestly: a LIBRARY class has no class file here, and a nested class is
flattened to the top level, so the `static` and `private` bits a JDK reports
from the InnerClasses attribute are gone. A number that is right for a
top-level user class and quietly wrong for the other two is worse than saying
so. (`Field`, `Method` and `Constructor` DO answer `getModifiers` — those flags
survive.) `Class.getPackage()` is refused too: `java.lang.Package` is not
modelled.

Both refusals needed `receiver_class_name` to know its own receiver kinds
first — it named eleven and not `Class`, `Throwable`, `Optional`, `Stream` or
`File`, so an honest reason written for any of those could never fire.

### A value that adopts its context, under `var` (2026-08-12)

Self-check #2 re-run after a session of typing changes — `var x = EXPR` must
infer what `println(EXPR)` emits — found eight disagreements over 74
expressions, and they share one shape.

A value that types as `null` so it **adopts its context** (an empty or
immutable-factory collection, a `collect` whose element the `map` erased, a
diamond) has no context to adopt under `var`. Worse, `type_of` had no rule for
a `null` RECEIVER at all, so every method called straight on one was untyped
though emission handled it: `List.of("a").size()`,
`Collections.emptyList().size()` and
`xs.stream().map(f).collect(toList()).size()` all printed fine and could not be
named, inferred, or passed as an argument.

The fix keeps the adopt-its-context typing — the assignment
`List<String> r = …collect(toList())` still works — and resolves a direct call
against the general face, which is what emission already did. A factory WITH
arguments recovers its element from them, so `List.of("a").get(0)` stays a
String.

The rest were members reachable only through their own emitter, never a method
table: `Stream.of(...)`, the `SimpleEntry` diamond, and `isNaN`/`isInfinite`
(added earlier the same day on the emitting path only — the check caught the
half that was missing). And a qualified factory (`java.util.List.of`) could not
infer, because the rule keyed on the first path segment and read it as a class
called `java`.

All 74 expressions now agree. The other three sweeps are clean too: 0
VM-internal errors over 570 library calls, 0 Object-method holes beyond the
deliberate `Scanner` refusal, and 458 qualified-name probes with one known
dependency-gated case.

### An immutable factory as a stream source (2026-08-12)

`List.of("a").stream().map(String::toUpperCase)` had no target type for its
lambda, though the identical pipeline over a declared `List<String>` worked and
`List.of("a").get(0).toUpperCase()` worked too. The lambda pass reads element
types SYNTACTICALLY — it runs before typing — and it knew a declared variable's
element, a `new ArrayList<String>()`, a map view and a cursor, but not
`List.of(...)`, `Set.of(...)` or `Arrays.asList(...)` used straight as a
source. The element is what the arguments agree on, which is the reading
`Stream.of(...)` already got.

It presented as a COLLECTOR gap — "toMap takes no lambda key extractor" — and
was nothing of the kind: every collector involved already worked over a
declared list. Worth remembering when a refusal names the operation furthest
from the actual cause.

Also fixed here: a type argument caturra models as a VARIABLE but not as a
collection element (`List<Scanner>`, `Set<java.io.File>`) had two different
false messages — one blamed the base (`unknown type 'List'`), the other called
a perfectly working class unsupported. Caturra stores elements in a closed
`ElemType` set, and the honest message says which of the two things is true.

### Reference-array class literals, and `Runnable` (2026-08-12)

**`String[].class`** did not parse though `int[].class` did. The primitive form
is read where a primitive type keyword is; after a class NAME a `[` starts an
array index, so the whole `[] … .class` tail has to be seen before committing
to a type — the same lookahead `String[]::new` already needed.

**`Runnable` is a functional interface, not a threading one.** It sat in the
unsupported list beside `Thread`, which also refused the lambda target every
callback example uses; `r.run()` runs on the spot and needs no thread.
`Thread` itself stays unsupported — a program here runs on one thread, in one
WASM instance.

Runnable has NO type arguments, so it arrives as a plain named type, and that
exposed two things the parameterized functional interfaces had hidden:

- **A bare functional-interface name had no descriptor.** `void f(Runnable r)`
  — and `int g(Comparator c)` — were "unknown type", though the same type
  resolved fine as a local. The parameterized form erases to the bundled
  interface; the bare form reached neither that arm nor the raw-collection one.
- **The erasure alias was unguarded.** `functional_erased`'s own comment says a
  class of that source name shadows it, "checked by the caller with
  `has_class`" — two of three callers checked. It went unnoticed until a
  bundled library (swing) declared its own `Runnable`: an anonymous class then
  implemented `__Runnable` while the method taking it expected the other one.

Found alongside: the bundled erased interfaces **leaked into diagnostics** —
"does not override abstract method compare() in `__Comparator`" names an
implementation detail, and reads as caturra's bug rather than the program's.
javac names the interface the source wrote.

### One member, two spellings (2026-08-12)

A batch of library members that existed under one name and not the other. Each
is small; what they share is that the missing half is the one a program reaches
for first — on the value it already has, or under the shorter name.

- **`d.isNaN()` / `d.isInfinite()`.** The statics (`Double.isNaN(d)`) were
  modelled and the instance forms were not, though asking a value about itself
  is the natural spelling.
- **`Map.entry(k, v)`.** The same standalone entry as
  `new AbstractMap.SimpleEntry<>(k, v)`, which caturra already built — only
  the spelling differed.
- **`chars()` / `codePoints()` on a `CharSequence` and a `StringBuilder`.**
  `CharSequence` declares them, so a builder answers them; the builder's were
  refused as "streams are not supported" while the identical call on a String
  worked. Adding them to the `CharSequence` face without implementing the
  builder's would have made one object answer through one spelling and abort
  through the other.
- **`Function.identity()`** is the lambda `x -> x`, which always worked
  written out. Saying so in the lambda pass is the whole implementation.
- **`Class.getTypeName` / `getCanonicalName` / `getEnclosingClass` /
  `isAnonymousClass`.** The canonical name spells a nested class the way
  source does (`Outer.Inner`, not `Outer$Inner`) and is NULL for an anonymous
  or local class, which have no canonical name at all (JLS §6.7).

Reaching these turned up two gaps left open rather than fixed: `String[].class`
does not parse though `int[].class` does, and `Collectors.toMap` takes no
lambda key extractor in either spelling.

### Object's methods on every reference (2026-08-12)

`getClass`, `hashCode` and `equals` are declared on `Object`, so every
reference has them — there is no type for which `x.getClass()` is "cannot find
symbol". Both halves of caturra made each receiver kind repeat them by hand,
and both drifted:

- **The compiler**: each builtin method table listed `getClass` itself, and a
  Scanner's listed none of the three. They come from one shared
  `OBJECT_METHODS`, consulted when a receiver's own table has no match — so a
  type that overrides one (a list's value-based `equals`) still wins.
- **The VM**: each receiver kind answered them in its own arms, and a
  `PriorityQueue`, a `Comparator`, a `Scanner` and a stream aborted the run
  with "unknown native member" for a call the compiler had accepted.

**Which semantics apply is per class, and that is the half a blanket default
gets wrong.** A `File`, an `Optional` and a `StackTraceElement` compare by
VALUE; a `PriorityQueue`, a `Scanner`, a `StringBuilder` and a stream compare
by IDENTITY. The identity kinds are listed explicitly rather than defaulted,
so a value-based class added later cannot silently fall in and compare by
identity behind the program's back.

Two things are refused rather than answered, because their text is genuinely
unmodelled:

- **A Scanner's `toString`**. The JDK's is a dump of its delimiters, position
  and locale separators. Concatenating one was already refused; saying "cannot
  find symbol" for the method was a false statement about the same thing, and
  the two spellings now give one honest reason.
- A `Comparator` built by `naturalOrder`/`comparing`/a lambda has no text a
  program can depend on — a real JDK prints its lambda class, which differs
  between runs — so caturra answers in `Object`'s shape and does not pretend to
  match. A comparator the program declared is an ordinary object and prints its
  own `toString`.

Found alongside: **`Optional.of(x)` emitted fine and had no type.** `Optional`
is a static-call class handled inline rather than through a table, so
`type_of` typed the name as an expression, failed, and gave up before reaching
its own `Optional.of` rule — which was therefore dead code. This is precisely
the `var` self-check's shape (an expression `println` accepts but `var` cannot
infer), and it survived because the catalogue had no `Optional` in it.

### `Iterable` as a type, and parameterization checks (2026-08-12)

`Iterable<T>` was fixed in the PARAMETER position (`<T> int f(Iterable<T>)`)
and stayed broken in every other one: a declared `Iterable<String> it = list;`
was "incompatible types", a cast to it was refused outright, and neither
carried the element, so `for (String s : it)` saw an `Object`. Three separate
gaps sat behind that one symptom:

- **A cast never consulted the widening rule.** JLS §5.5: every widening
  reference conversion is a casting conversion. The cast arms only understood
  a `JType::Object` source, so a redundant upcast that ASSIGNS fine was
  refused.
- **A cast returned its target erased.** `(Bag<String>) o` had type raw `Bag`,
  so every later use lost the argument. The cast expression has the written
  type; only the run-time check uses the erasure.
- **The synthesized `Iterable.iterator()` declared `Iterator<Object>`**, so
  there was no type VARIABLE for the receiver's argument to substitute into.

Widening a user generic class was missing outright: nothing matched a
`JType::Generic` on the LEFT, so a `Bag<String>` could be held only by its own
type or by `Object` — not by `Iterable<String>`, not even by a raw `Bag`.

The opposite direction was open too, and it is the one that hurts. Two
different parameterizations of one class erase alike, and **both** gates —
`widens` and the assignment matrix — keyed on the erasure alone, so
`Bag<String> b = bagOfIntegers;` compiled and the program ran with the wrong
static type throughout. javac rejects it. The builtin collections compared
their elements all along; only a user generic class fell through. Assigning
THROUGH a raw type still launders it, which is javac's rule too.

**Two gates, one rule.** An assignment is checked by `widens` AND by the
conversion matrix in `convert_for_assignment_const`, and a rule added to one
does nothing in the other. The matrix already carried a comment warning about
this; it caught the fix mid-flight again here.

### Fully qualified names (2026-08-12)

A fully qualified name needs no import — that is the whole point of writing
one. Caturra resolved the qualified spelling on a *different* path from the
simple one, and the two disagreed in four places:

- **A drifted package list.** `canonical_library_class` kept its own list of
  the packages holding modeled classes, beside the real one in
  `package_classes`. It had already drifted once (`java.util.Arrays.fill`
  did not resolve though `java.lang.Math.abs` did) and had drifted again:
  `java.util.stream` was missing, so `java.util.stream.Stream<String> s` was
  refused as unsupported while `import java.util.stream.*` compiled. Both
  lists are now one table.
- **A two-segment assumption.** The expression path collapsed exactly
  `java.X.Y`, so `java.util.stream.Stream.of(1)` read as a class `stream` in
  package `java.util`. It now tries the longest prefix first.
- **Qualified nested types.** `java.util.Map.Entry<K, V>` split into a
  package `java.util.Map`, which does not exist. What precedes the last dot
  may be an enclosing class.
- **Import-gated bundled libraries.** The clean-room `org.code.*`,
  `javax.swing.*` and `java.awt.*` sources are injected when a program
  reaches for their package, and reaching was read as *importing*. So
  `new org.code.neighborhood.Painter()` was refused with the false claim
  that Painter is not supported by caturra. A program now reaches for a
  package by naming it in full too — detected on the TOKEN stream, so a
  package named in a comment or a string literal still pulls nothing in.

The invariant is now checked exhaustively rather than case by case: every
modeled class in every modeled package, written both ways, in a type position
and in a static-member position. 456 probes, and the only remaining
disagreement is `org.code.validation.NeighborhoodTestRunner`, which is
injected only alongside `org.code.neighborhood` and refuses in both spellings
once that is present.

One divergence was found rather than fixed, and is listed below: a class
caturra models only as a namespace for its statics cannot name a variable.

### A constant `if` condition and definite assignment (2026-08-13)

javac accepts `if (false) { } else { v = 1; }` and reads `v` afterwards: the
then-branch cannot execute, so it imposes nothing on definite assignment
(JLS §16.2.7 — "definitely assigned after e when false" is vacuous for a
constant-true condition, and symmetrically for a false one). caturra
intersected both branches unconditionally and refused every shape where only
the taken branch assigns — including the `if (DEBUG) … else …` a program
actually writes.

The no-`else` form already had the carve-out, but tested for a **literal**
`true`, so `if (1 < 2) x = 1;` was refused while `if (true) x = 1;` was not.
Both forms now ask the same folder — codegen's own, which resolves a constant
EXPRESSION and a constant VARIABLE, local `final boolean` included.

An existing test had pinned the wrong behaviour here, commented "javac-style
error" for a program javac accepts. Worth remembering: **a test can encode the
bug**, and only asking the JDK finds that out.

Found by an evaluation-order/scope/definite-assignment audit (round 10,
dimensions 1–3): 85 probes, one divergence, and the other two dimensions were
clean — evaluation order including `a[i++] = i++` and compound-assignment
targets evaluated once, and shadowing including a private method not being
overridden.

### `Math`'s transcendentals are FDLIBM's, not the platform's (2026-08-13)

A program that prints `Math.atan2(y, x)` prints every digit, so these have to
agree with a JDK 11 bit for bit. Java's are FDLIBM-derived; Rust's are the
platform libm, and the two differ in the last ulp often enough to matter.

**Measured over 17 100 calls** (19 functions × 900 pseudo-random inputs):

| function | wrong | | function | wrong |
|---|---|---|---|---|
| `atan2` | 25.2% | | `asin` | 4.8% |
| `hypot` | 12.4% → **0** | | `log10` | 3.8% |
| `cosh` | 8.8% | | `exp` | 0.3% |
| `cbrt` | 8.3% → **0** | | `sin`, `pow` | 0.2% |
| `acos` | 8.1% | | `tan`, `atan` | 0.1% |

Exact already, and now pinned so they cannot drift: `sqrt`, `cos`, `log`,
`log1p`, `expm1`, `sinh`, `tanh`.

Seven are FDLIBM ports now — `cbrt`, `hypot`, `atan`, `atan2`, `asin`, `acos`,
`cosh` — each verified to **0 divergences** on the corpus that had them wrong,
taking the total from 652 to **42 (93.6% closed)**. They are transcribed with the
algorithm's own constant spellings and variable names so a reader can check
them line by line against the original; the module carries a `#![allow]` for
the clippy lints that would otherwise push toward a tidier transcription nobody
can verify.

**Which functions are portable at all is decided by HotSpot, and the JDK will
tell you.** Comparing `Math.f(x)` with `StrictMath.f(x)` over 4 000 inputs:

- **`Math` == `StrictMath`** for `cbrt`, `hypot`, `atan2`, `asin`, `acos`,
  `cosh` — no intrinsic, so these really are FDLIBM, and all seven ports
  landed at exactly 0.
- **`Math` != `StrictMath`** for `log`, `exp`, `log10`, `sin`, `cos`, `tan`,
  `pow` — HotSpot substitutes an x86 intrinsic, so FDLIBM is the WRONG target
  for them. Ports of `exp` and `log` made things worse (3 → 103 and 0 → 21)
  and were reverted; Rust's libm is much closer to the intrinsic than FDLIBM
  is.

The FDLIBM `exp` is kept, but only as `cosh`'s internal helper — which is the
whole reason an earlier `cosh` written over Rust's `exp` reproduced Rust's
answer instead of the JDK's. `StrictMath.cosh` calls FDLIBM's own `exp`, not
`Math.exp`.

The remaining 42 (`log10` 34, `exp` 3, `sin` 2, `pow` 2, `tan` 1) are all in
the intrinsic set. Closing them means matching an x86 intrinsic, not a
published algorithm.

### `printf` digits, and an `Object` argument (2026-08-13)

Two defects on the formatting path, found by probing the neighbours of the
`Double.toString` fix — `%f`/`%e`/`%g` render on their own path, so that fix
did not reach them.

- **The digits came from the shortest round-trip decimal.** OpenJDK derives
  `%f` from the same `FloatingDecimal` as `toString`, so `%f` of 1e23 is
  `99999999999999990000000.000000`; caturra printed
  `100000000000000000000000.000000` — a different NUMBER, padded with the
  wrong zeros. One helper now asks the JDK-11 renderer, and all four call
  sites follow.
- **An `Object`-typed argument was converted to a String at the call site.**
  The formatter decides by the RUNTIME class — `%d` of an `Object` holding an
  Integer formats the number, which the statically-boxed case already relied
  on. Coercing first made every such argument a String, so
  `String.format("%d", someObject)` threw `IllegalFormatConversionException`
  for a program the JDK runs. This is the shape a generic helper has:
  `String show(String format, Object argument)`.

Checked on 14 231 formatted fields — ten specifiers across random bit
patterns, simple ratios and powers of ten with their neighbours — all
identical to a real JDK 11.

### `Double.toString` is JDK 11's, not the shortest decimal (2026-08-13)

OpenJDK 11 does **not** print the shortest round-trip decimal — Ryū arrived in
JDK 19 — and its `FloatingDecimal` sometimes emits one digit more than needed.
`1e23` prints as `9.999999999999999E22`, and a value needing 16 digits can be
given 17. caturra rendered doubles with Rust's shortest formatting, so it
matched a MODERN JDK and not the one the course targets: about 0.7% of
arbitrary doubles came out differently.

The `float` path already had a clean-room `FloatingDecimal` (corpus-derived,
not the GPL source), and its rendering half is width-agnostic — only the
bit-field extraction and the slop constants are per-type. `Double.toString` now
runs the same algorithm with 53-bit significands.

**Measured, not asserted:** 24 082 values across random bit patterns, small
integers and simple ratios, powers of two and ten with their neighbours,
subnormals, and a drifting accumulation. **6 disagree (0.025%), all subnormals
below 1e-315.** Both alternative constant choices were tried and are far worse
(a full ulp costs 1671 values, a quarter costs 826), so what remains is not a
constant to fit but JDK's per-value significant-bit count for subnormals. The
entire normal range is exact.

### Case mapping, carried rather than derived (2026-08-14)

The classification entry ended by noting that the earlier 15-range
case-mapping patch was still keyed to where Rust and JDK 11 *happen* to
disagree, and so would rot the next time the Rust toolchain updated its tables.
That inconsistency is closed the same way: recorded, not derived.

`Character.toUpperCase`/`toLowerCase`/`toTitleCase` now read a JDK 11 table of
2235 simple mappings and 12 titlecase exceptions. `String.toUpperCase`/
`toLowerCase` read the FULL mappings, which differ from the simple ones for
exactly 103 units — `ß` to `SS`, `\u0130` to `i` and a combining dot, the `ﬁ`
ligature. Those were wrong too, on **3321 units against the JDK's 1203**, for
the same Unicode-version reason; the earlier patch had only ever covered
`Character`.

Two surrogate losses fell out of pinning it, both of them corrupted VALUES
rather than renderings:

- **Appending a string to a `StringBuilder` went through a Rust `String`**, so
  a string holding an unpaired surrogate had the unit REPLACED rather than
  copied. That is why `"a😀cd"` reversed char by char could not round-trip.
- **`StringBuilder.append(Object)` on a boxed `Character`** did the same.

Those bounded the last of it, and the rest is closed below.

### An unpaired surrogate is a char like any other (2026-08-14)

Following the bound stated above to its end. `String.valueOf(Object)`, and a
concatenation whose operand is statically `Object`, both lower to
`obj.toString()` — so the loss was in the boxed wrapper's `toString`, not in
`object_display` as the bound had guessed. A `Character` is its UNIT, and it is
now answered as one.

With that, **the 24,371-line String cross product is byte-identical**, and an
unpaired surrogate survives being stored, concatenated, appended, boxed and
re-read, reaching the console as the `?` a JDK's encoder substitutes rather
than as U+FFFD — a different character a program can also legitimately print,
which is why the two must never be conflated. Every path is pinned.

The char-literal case is closed below.

### StringBuilder swept; an enclosing static could not be typed (2026-08-14)

A twenty-fourth dimension put `StringBuilder` through a cross product — every
mutator against every seed and index, results AND exception messages, over an
alphabet carrying astral characters, a combining mark and an unpaired
surrogate. **3,588 lines, all identical.** The surface is pinned so it stays
that way.

The finding came from RUNNING the probe rather than from its results: it
stopped on caturra's own "cannot determine the type of an argument" — the net
added when a vanishing statement was found, doing its job. **`type_of` could
not type a call to an enclosing STATIC from inside a lambda.** The
enclosing-INSTANCE fallback was already there, but a lambda in a static method
captures no instance, so it never fired, and the lexical chain — the only route
to an enclosing static — was not tried. The emission path resolved the same
call perfectly well; `type_of` versus emit, for the third time this round.

Worth noting what the net bought: before it existed this would have been a
silently dropped statement rather than a refusal, and a probe that prints
nothing looks like a probe that passed.

That left `capacity`, `ensureCapacity` and `trimToSize` refused — closed below.

### Reflection: the surface the graders run on (2026-08-14)

Chosen because the corpus had just shown where reflection actually executes:
**103 sites, all inside grading harnesses.** A defect there changes a student's
mark and nothing prints. 48 probes; nine diverged, and the two worst were
access rules.

- **`getMethod` and `getField` returned PRIVATE members.** They shared one
  lookup with the `Declared` forms and never filtered by access — so a harness
  asking "is this method public?" was told yes about a private one, and a
  student passed a test they should have failed. (`getMethods`, the plural, had
  the filter all along.)
- **`Method.invoke` did not wrap what the target threw.** A harness writes
  `catch (InvocationTargetException e) { e.getCause() }`; caturra handed over
  the raw exception, so that catch never fired. Wrapping follows the
  `ExceptionInInitializerError` machinery already in the unwinder.
- **`Field.set` stored a WRAPPER into a primitive field.** `set` takes an
  `Object`, so an `int` field was handed an `Integer` and kept it; nothing
  failed until the next read of that field, somewhere else entirely.
- `Method.toString` printed a JVM descriptor rather than Java's signature
  format, and `NoSuchMethodException` named no parameter list, so a missing
  overload read the same as a missing name.

And a fourth instance of the round's recurring shape: the emitter packs
`getMethod`/`getDeclaredMethod`/`getConstructor`'s `Class...` arguments into a
`Class[]` before any table lookup, and `type_of` matched the table directly —
so a chain through the varargs form typed as unknown, refusing a `println` of
it and, inside a lambda, emitting bytecode that did not verify.

That last one is worth recording against the cross-check added just before it,
which did NOT catch it: **both paths failed, so there was nothing to
disagree.** The check finds a `type_of` that is wrong where the emitter is
right; it cannot find a hole they share.

47 of 48 matched at that point; the last one is below.

### Legal Java that caturra refused (2026-08-14)

Every dimension so far asked "does this run the same?", which can only reach
programs caturra ACCEPTS. The last two findings arrived the other way — a probe
that would not compile — so a thirty-fourth went looking on purpose: 48 small
programs, each accepted by javac, across generics, lambdas, control flow,
nested classes and the awkward corners of the language. **Six were refused.**

- **An annotation on a LOCAL declaration did not parse.** Members accepted
  them; locals did not, so `@SuppressWarnings("unchecked") List<String> l =
  (List<String>) o;` — the standard idiom for an unchecked cast — read as
  "expected an expression". They are skipped now, and dropped rather than left
  pending, so a local's annotation cannot attach itself to the next member.
- **`Arrays.asList` read its element type off the FIRST argument**, so
  `asList(1, 2.5)` was refused for the `2.5`. The element is the join of them
  all now: mixed numerics at `Number` — which is what a `List<? extends
  Number>` parameter needs, and `Object` would not satisfy — two references at
  their nearest common supertype, and anything else at `Object`.

That second fix is a good example of a coarse answer passing its own test and
failing someone else's: falling to `Object` for every mismatch satisfied
`asList(1, 2.5)` and broke `asList(new B(), new C())`, which the inheritance
dimension's pinned test caught immediately.

Recorded, not fixed: a `List<? super Integer>` parameter does not accept a
`List<Number>`; a lambda whose body is itself a lambda; an array of a generic
type (`new List[2]`, refused honestly); and a functional interface as a
collection element, which was already documented.

### A diamond adopts its target (2026-08-14)

A thirty-third dimension took inheritance and polymorphism — construction
order, virtual dispatch from a superclass constructor, field and static HIDING
against method overriding, `super` calls, abstract dispatch, two inherited
defaults, polymorphic arrays and collections, and the runtime failures.
**67 lines, all identical.**

The finding was that the probe would not COMPILE. `List<Shape> shapes = new
ArrayList<>(squares)` was refused as "ArrayList<Square> cannot be converted to
ArrayList<Shape>" — ordinary Java, and only in the diamond form: writing
`new ArrayList<Shape>(squares)` worked, as did an empty diamond followed by
`addAll`.

A diamond is a POLY expression (JLS §15.9.1): it takes its type argument from
the target, not from what it copies. caturra inferred it from the constructor
ARGUMENT, which is right for `var` — where the initializer IS the context, and
which is why the `var` path had already been special-cased — and wrong wherever
a type is written down.

The fix is deliberately narrow, because `List<Square>` really is not a
`List<Shape>`: only a diamond `new`, and only when the inferred type differs
from the target by an element that widens to it. Assigning one collection
variable to another still fails, and so does an unrelated element type. The
runtime object is the same either way — a collection's elements are not typed
at run time — so adopting the target changes what is ACCEPTED and nothing about
what is built.

### java.lang.reflect, and which classes are the library's (2026-08-14)

`import java.lang.reflect.*` worked while `java.lang.reflect.Method` written
out in full did not — the package was waved through at the import check but
never listed as a package, so a QUALIFIED use could not resolve, as a type or
as a receiver. It is listed now, which also gives those classes their
fully-qualified `getName()`.

That exposed the question the earlier class-name fix had dodged: **`Modifier`
the library class and `Modifier` the class a student wrote answer to the same
simple name.** Only the first is `java.lang.reflect.Modifier`, and nothing in
the table said which was which. It does now — `ClassInfo::is_bundled`, set for
a class synthesized here (`Object`, the wrappers, `Comparable`) or parsed from
a bundled source, whose units are lexed under an angle-bracketed path that
nothing a user can write collides with. A program's own declaration takes the
name back, which needed one more line: a user class of an already-synthesized
name was skipped as a duplicate, leaving the library entry to answer for it.

Both directions are pinned, because this is a rule that is easy to get right in
one direction and wrong in the other.

**The corpus caught a regression, and the shape is worth keeping.** Listing the
package let the old permissive import check go — and 12 levels stopped
compiling, because a harness imports `java.lang.reflect.Parameter`, which
caturra does not model, and never uses it. The import check is permissive again
for that package: an unused import is not the place to refuse a class, and the
use site still does. A narrower rule looked strictly better and was strictly
worse for real programs.

### Auditing the refusals themselves (2026-08-14)

Two fixes this round came from re-reading an old refusal and finding its reason
no longer true — `StringBuilder.capacity` and `Collection.toArray`. That is a
cheap, repeatable audit, so it was run over the whole table: 37 members, each
reason checked against what caturra can actually do today.

**Eight reasons were false.** They named capabilities the engine has since
gained — streams, regular expressions, iterators, varargs — and a refusal that
misdescribes the engine is worse than no refusal at all: it sends a student
looking for a workaround that does not exist, and tells the next maintainer the
wrong thing about their own system.

Two of the eight were only ever a table entry:

- **`LinkedList.listIterator`** was refused as "iterators are not supported",
  while the very same list machinery already answered `ArrayList.listIterator`.
  Both forms work now.
- **`String.getBytes()`** was refused as "byte arrays are not supported". They
  are, and it is the default charset's encoding — UTF-8 here — with an
  unpaired surrogate becoming `?`, the substitution the console already makes.

The rest name members that genuinely are not modelled, so the entry stays and
the WORDING was corrected to say what is actually missing: a `Scanner` that
splits on whitespace and takes no delimiter pattern; no descending view of a
`TreeSet` or `LinkedList`; the immutable factories living on `Map` rather than
`HashMap`; and a parallel stream that, on one thread, would only be a
sequential one under another name.

Nothing here was implemented on the strength of the audit alone — each
capability was probed against a real JDK first, which is how `descendingIterator`
stayed refused rather than being faked with a reversed snapshot whose `remove`
would have written to the wrong element.

### String.format swept; collections found two more (2026-08-14)

Two dimensions, one clean and one not.

**`String.format` is exact.** 44 specifiers against 25 argument shapes, plus
the malformed calls whose text a student reads — **1,134 lines, all
identical**. Flags, width, precision, argument index, the non-finite doubles,
negative zero, and every "wrong type for this conversion" message. Pinned as a
slice rather than left as a one-off measurement.

**The collections' own instance methods** ran 82 probes over two `List`
implementations plus the `Map`/`Set`/`Deque`/`Stack`/`PriorityQueue`
contracts, and found two things.

`toArray()` was refused because "Object arrays are not supported by caturra".
**That reason had expired** — `Object[]` is fully modelled, as a probe
confirmed before a line was changed. It is the SECOND stale honest-reason this
round, after `StringBuilder.capacity`, and both were found by reading the
refusal rather than trusting it. It is now implemented for every collection
kind, boxing primitive elements the way `Stream.toArray` already did.

The other is a message split worth recording, because a single format looked
obviously right and was wrong in two directions at once. The JDK words an
out-of-range list index TWO ways: `ArrayList` reaches the shared
`Objects.checkIndex` and says `Index N out of bounds for length L`, while
`LinkedList` writes its own `Index: N, Size: L` — and `ArrayList.add(index, …)`
uses the SECOND form, because `rangeCheckForAdd` predates the shared check.
caturra used the first everywhere. All 24 combinations of implementation,
operation and out-of-range index now agree.

`subList` was refused here — a list VIEW, which is a feature rather than a
message. It is a view now (see **subList as a live view**), and its own
out-of-range wording joined the 24 combinations above.

### A descending cursor (2026-08-15)

The last entry in the refusal table left unjudged, and the reason was a
misreading of what the method is: "caturra does not model a descending view of
a LinkedList". It is not a view. It is the same cursor started at the END,
walking toward the front, with `remove()` working through it — so one flag on
the cursor, and `next()` does what `previous()` does.

The flag is a field on the cursor rather than a separate object, which makes
the compiler name every place a cursor is built and each of those declare its
direction. Five sites, each an honest "this one is ascending".

Answered ONCE at the shared native dispatch rather than at each of the points
that build a forward cursor — a list's, a view's, a queue's, a tree's. Doing it
per-path first showed the cost immediately: `LinkedList` worked while `TreeSet`
and `ArrayDeque` still refused, because they reach the cursor by different
routes. The direction is the only difference and the receiver kind does not
change it, so it belongs where the receiver kind is not yet known.

With this, every entry in the refusal table has been judged: three reasons had
expired (the TreeMap entry accessors, collection `clone`, this), and the rest
hold.

### Two more surfaces, and a negative result (2026-08-15)

Continuing to write down the surfaces the cleared scratchpad used to cover,
choosing the two where a defect would be SILENT rather than a refusal — because
a refusal announces itself and a wrong answer does not.

**How every kind of value RENDERS**, in each context that renders one:
concatenation, `String.valueOf`, `%s`, inside a list, inside a map. Fifty rows,
byte-identical, including the numeric cases this project has been bitten by
before — `1e23`, whose shortest decimal is not JDK 11's, and `-0.0`, the NaNs
and infinities, `Double.MIN_VALUE`, `Float.MAX_VALUE`, `1.0/3`.

**What observes a mutation and what does not.** Every silent wrong answer found
this session lived on this surface: an `entrySet()` copied as its KEYS, a
stream that read its source when it was BUILT rather than when it ran, a
`LinkedHashMap` losing its ordering through a sizing constructor. Views are
live and write through; copies and clones are detached; `Arrays.asList` writes
through to its array in both directions; a stream is late-binding; and the
`Integer` cache makes `==` true at 127 and false at 128. Byte-identical.

Four surfaces asked, four clean. That is a negative result and worth recording
as one: the axes that were productive earlier today are productive no longer,
which is a statement about the engine rather than about the method. The method
still holds — every one of these was closed by a fix made this session, and
each is now gated instead of assumed.

### Two surfaces, written down (2026-08-15)

A checked invariant catches the paths disagreeing; only the oracle catches them
agreeing and both being wrong. The probe corpora were what asked the oracle,
and they lived in a scratchpad that has since been cleared — so the surfaces
worth keeping belong in the differential suite, where every program is compiled
AND diffed against a JDK on every run.

Two are written down now, and both came back clean:

**Every expression form, used as an ARGUMENT.** That position is what forces
`type_of` to answer, and it is exactly where the suite sweep's 82 disagreements
bit: each printed correctly, because `println` types itself, and each failed
with "cannot determine the type of an argument" here. Seventy expression forms
— literals, every operator, pre/post increment, names, fields, array access,
calls, `new`, enums, class literals, `instanceof`, lambdas behind a stream —
byte-identical to a JDK.

**Overload resolution across the argument forms.** The other question an
argument position asks, and the one where a wrong answer is SILENT: choosing
`f(long)` where javac chooses `f(int)` runs a different method. Widening versus
boxing versus varargs, `Number` against `Integer`, `Collection` against `List`,
`char` against `int`, `String.valueOf`'s and `StringBuilder.append`'s families
— and the trap every Java programmer meets once, `list.remove(1)` removing an
INDEX where `list.remove(Integer.valueOf(30))` removes a VALUE. Byte-identical.

Finding nothing is the useful part of both: the expression surface is where the
82 lived, and it is closed and now gated rather than closed and assumed.

### The third pair, and what it found (2026-08-15)

The source names a third pair of paths that answer one question, and records
being caught by it twice: "this matrix gates separately from `widens`, so both
need the arm — the same trap that once left List -> Collection widening
half-implemented". It caught a third case this session, for an array of
collections.

The matrix's refusal arm now asks `widens` and reports when the two disagree —
a check at exactly the point of failure, so it costs nothing until an
assignment is actually about to be refused. Perturbing it (inverting the guard)
makes `String s = 5;` report, so a clean sweep means the invariant holds rather
than that nothing ran.

**Writing the test for it found the bug the check could not.** The check fires
only when the two paths DISAGREE; when both are wrong together it says nothing.
Compiling a conversion surface for it to sweep — thirty-odd assignments that
javac accepts — turned up `List<String>[] b; Object[] o = b;` refused by both,
with the message "incompatible types: Object[] cannot be converted to
Object[]".

The rule "any reference array widens to `Object[]`" had been written as a LIST
of the element kinds that happen to be references, and the list left out the
nested (collection) one. It asks the element itself now. A primitive array is
still not an `Object[]`, which is the half that listing kinds was protecting,
and both directions are pinned.

**A checked invariant covers the paths disagreeing, never the paths agreeing
and both being wrong.** The oracle is still the only thing that catches the
second kind.

### The second pair of paths (2026-08-15)

With the `type_of`-versus-emit sweep closed, the question is which OTHER pair of
functions answers one question twice — because that was the cause of all 82.

There is one more, and it has already produced a bug: a method's descriptor is
built from the WRITTEN `TypeRef` for the signature the class file carries, and
from the RESOLVED `JType` by every call site. `Iterator<T>` in a signature
disagreed between them, and the symptom was "malformed class Main: no static
method f(Ljava/util/Iterator;)" — a call naming a method that was never
emitted. That was found by hand, from a probe.

It is checked now, under the same `CATURRA_VERIFY_TYPES` flag, at both points
where a method's descriptor is written. A type that does not resolve is skipped
rather than reported: a user generic or a type variable has no `JType` to
compare against, and the written form is the authority there.

The corpus and the differential suite report **none**. That is a real zero, not
an unrun check: perturbing the comparison to expect `Q` where the return
descriptor says `V` makes it report on every method of a two-line program,
including the implicit constructor. After the `--nocapture` lesson — where a
sweep reported zero because `cargo test` swallows stderr for passing tests — a
clean result is worth only as much as the proof that the check can fail.

### The type sweep, closed (2026-08-15)

The differential suite reports **zero** `type_of`-versus-emit disagreements,
from 82. The tail took nine more fixes sharing no cause beyond the shape of the
sweep itself — a fact the emitter had and its mirror did not:

- a nested type's statics named through the enclosing type
  (`Holder.Kind.values()`), which the emission path resolves by dropping the
  qualifier and the mirror only ever looked at one segment;
- `Collections.max`/`min` over a COLLECTION rather than a list, and
  `replaceAll`, `unmodifiableCollection`, `nCopies(n, null)`;
- `Objects.checkIndex` and `Objects.compare`, absent from a mirror that listed
  their neighbours;
- `Objects.requireNonNull(7)` — a type variable is a REFERENCE, so a primitive
  argument BOXES and javac infers `Integer`;
- `writer.format(...)`, variadic and so special-cased out of its own method
  table, returning the writer for chaining;
- `IntStream.empty()`.

**What the sweep was worth.** Every one of these printed correctly before it was
fixed, because `println` types itself — and every one REJECTED the same
expression the moment it was passed to a method or used to infer a `var`. The
invariant had been documented as sweepable over this suite for months; running
it took one command and found 82 real defects, of which the majority were
rejections of valid Java rather than cosmetic disagreements.

The recurring cause, across all 82: **one fact, written down twice.** The
element join lived in three places, the copy-source element in two, the
builtin-receiver set in a list and the table it gates. Where a fact had one
home, the paths agreed.

### Types the emitter knew alone (2026-08-15)

Three shapes from the tail of the suite sweep. Each one REJECTED legal Java the
moment the expression's type was actually needed — passed to a method, or used
to infer a `var` — and each printed fine before that, because `println` types
itself.

- **`Double p, q; p + q`.** Binary numeric promotion UNBOXES its operands
  (JLS §5.6.2). `type_of` handed the boxed types straight to `promote`, which
  fell back to `int`, so `var s = p + q` inferred `int` and then refused its own
  initializer as a lossy conversion. The bitwise operators three lines below
  already applied the unboxing view — the arithmetic ones never had.
- **`import static java.lang.Math.PI`, then a bare `PI`.** The emitter reads
  imported CONSTANTS; `type_of` knew only imported METHODS, so the constant
  printed and could not be passed anywhere.
- **`super.getMessage()` inside a class extending a LIBRARY throwable.** The
  superclass has no entry in the class table. The emitter carries an explicit
  Throwable fallback for exactly this, with a comment explaining why; `type_of`
  returned an error beside it.

All three are the same shape as the rest of this sweep: a fact the emitter had
and its mirror did not. 24 → 19.

### A list of the types that have a table (2026-08-15)

The largest remaining shape from the suite sweep, and one change closed 27 of
the 51.

`type_of` gated its builtin-receiver dispatch on a hand-written list of the
`JType`s that have a method table — and the body of that arm then asked
`builtin_instance_table` anyway, with an `.expect("matched builtin
receivers")`. So the list existed only as a gate on the same question the table
answers, and the two had drifted: `DoubleStream`, `LongStream` and
`IntSummaryStatistics` all had tables and were missing from the list.

`type_of` could therefore not type a single method on one.
`ns.stream().mapToDouble(x -> x).sum()` was "cannot determine the type of an
argument" when passed to a method, while `println` of it — which types itself —
printed the right number. The gate asks the table now, so a receiver that gains
a table gains its typing with it.

**Two lists that must agree is the shape behind most of this sweep.** The
element join was written three times; the copy-source element twice; this one
was a list and the table it gates. Each disagreed only where nothing had looked.

51 → 24, the rest a long tail of distinct causes.

### What a diamond takes its type from (2026-08-15)

The second and third shapes from the suite sweep, both diamonds, and they
disagreed in opposite directions — which is what kept either from looking
obviously wrong.

`new AbstractMap.SimpleEntry<>(k, v)`: the emitter read the key and value off
the ARGUMENTS, `type_of` answered the context-adopting `Null`.
`new ArrayList<>(m.keySet())`: the reverse — `type_of` asked a helper that
knows a set and a map view carry elements, while the emitter read the element
off a `List` alone and answered `Null` for anything else. That helper is now
one function both call, so a `var` declared from either infers what javac
infers.

Writing the test for it turned up a fourth thing: a `SimpleEntry` could be
CONSTRUCTED and not NAMED. `AbstractMap.SimpleEntry<K, V> e = …` was "package
AbstractMap does not exist" — about a package that is a class. A bare nested
name splits at its last dot into a package that does not exist, so the
canonical-name lookup could not see it; `nested_library_class` (added for
`Map.Entry::getKey`) can, and the type-resolution and descriptor paths both ask
it now. The type is nameable in every position a program can write.

82 → 51.

### Sweeping the differential suite for type divergence (2026-08-15)

This file has said for some time that "the corpus and the differential suite
can be swept the same way with `CATURRA_VERIFY_TYPES=1`". The corpus had been.
The suite had not, and it holds 777 real Java programs — every shape the
project has ever pinned. Swept, it reports **82 disagreements in 30 distinct
shapes**.

A measurement note that nearly hid them: `cargo test` captures stderr for
PASSING tests, so the first sweep reported zero. The check writes each
disagreement to stderr, so the suite has to be run with `--nocapture`.

The first shape, and the largest at 19 of the 82, is the type of `i++` on a
BOXED counter. JLS §15.14.2 is explicit — the type of a postfix increment is
the type of the VARIABLE — so `Integer i; i++` is an `Integer`, which javac
confirms by inferring `Integer` for `var x = i++`.

`type_of` answered the primitive, and its comment gave the reason: `i++ + 1`
and `i++ < n` have to stay legal. They do, but because binary numeric promotion
unboxes the operand (§5.6.2), not because the expression is primitive — the
comment was right about the consequence and wrong about the cause. The emitter
answered the wrapper, *except* for an array element, where it answered the
primitive and `type_of` the wrapper. **Each path was right about one case and
wrong about the other**, which is why the disagreement survived being looked at.

None of it changed any program's output, which is what a latent divergence
looks like until the day it does not.

Remaining after this unit: 63, in shapes that are being worked through in
order of frequency.

### A collection's clone (2026-08-15)

`ArrayList.clone()`, `HashMap.clone()` and `TreeMap.clone()` were refused as
"clone is not supported by caturra", while `new ArrayList<>(list)` beside them
did the same work: a SHALLOW copy is what both are. The reason expired when the
copy constructors landed, and it was the third expired refusal this round.

Cloning the heap object IS the operation, so it is one arm at the shared
dispatch rather than one per collection — and a `LinkedHashMap` keeps its
insertion ordering for free, because the flag rides along in the value being
cloned. The JDK returns `Object`, so a program casts the result, which is why
casting back to a collection had to work first.

Pinned: the copy is independent, its ELEMENTS are shared (`clone` is shallow),
and it keeps its class.

**A refusal pinned by a test outlives its reason exactly as quietly as one that
is not — and the test then argues FOR keeping the gap.** The test that pinned
this one already carried a note that `toArray` had been removed from it for the
same reason, which is the second time that file has recorded the pattern. Both
members are now exercised rather than explained.

### Casting back to a collection (2026-08-15)

Chasing the refusal table's `clone` entries — which need a cast to be usable —
found the cast itself. Only `List` had an arm, so `(Map<K, V>) o`,
`(Set<E>) o`, `(TreeMap<K, V>) o` and the rest were all "cannot cast Object to
…": the ordinary store-in-an-`Object`-and-cast-it-back, refused.

The `List` arm was worse than missing. It checked `java/util/ArrayList`, so
`(List<E>) o` THREW ClassCastException on a `LinkedList` that a JDK accepts —
a wrong answer, not a refusal. `List` and `ArrayList` are one `JType` here, so
the type cannot say which the program asked for; the class to check is the one
it WROTE. `raw_library_internal` is already that table, and its own comment
records this exact mistake being made once before, in `instanceof`: "an
INTERFACE must map to the interface's own name, never to a concrete class —
`Set` -> `java/util/HashSet` made a TreeSet answer `instanceof Set` with
false."

So the same table now serves both, and the other half is pinned with it: an
interface target accepts any implementation, a concrete one accepts only its
own class, and an unrelated object is no collection at all.

### A TreeMap's entry accessors (2026-08-15)

Three shapes recurred this session: a name with more than one segment, a fact
taught to one path and not its mirror, and a REFUSAL whose reason had stopped
being true. The first two now have sweeps. This is the third, and the refusal
table is the place to look, because it is machine-readable: 35 entries, each
naming a class, a member, and a reason.

`TreeMap.firstEntry`/`lastEntry`/`pollFirstEntry`/`pollLastEntry` were refused
as "TreeMap entry views are not supported by caturra". That expired when
`Map.Entry` became a type a program can hold — the entries an `entrySet()`
hands out already worked here, `setValue` write-through and all.

What the JDK returns is not a view but an immutable SNAPSHOT
(`AbstractMap$SimpleImmutableEntry`), which is a stricter thing to model and
the reason to check rather than assume: it must NOT follow a later `put` to the
same key, its `setValue` throws, and an empty map answers `null` rather than
throwing — unlike `firstKey` beside it, which throws. All four are modelled the
way a standalone entry already was, as a hidden one-mapping map with a
read-only entry over it, and every one of those behaviours is pinned.

The same audit found a comment claiming `IntStream.average`/`min`/`max` were
unsupported "because caturra does not model Optional", long after it did — all
three have worked for some time. A comment can expire as quietly as a refusal.

### Sweeping the probe corpus for type divergence (2026-08-15)

The `type_of`-versus-emit invariant is CHECKED, and the check is swept over
the grading corpus — which contains no maps, no interfaces, no streams and no
`Optional`s. Eight divergences this round were in exactly those APIs, so the
sweep could not have found any of them. The 395 probe programs written for the
legal-Java and qualified-name dimensions can, and they cost nothing to reuse:
compile each with `CATURRA_VERIFY_TYPES=1` and read stderr.

Three distinct divergences, none of which changed any program's OUTPUT — they
are the latent kind, which surfaces the moment such an expression is used where
its type matters (an argument to a user method, a `var` initializer):

- **The element JOIN of a factory literal was written THREE times** — the
  emitter, the `type_of` mirror and `var` inference — and the copies disagreed
  about `List.of(1, 2.5)`: one said `Integer`, another `Number`. Now one
  helper, called by all three, and `Map.of`'s keys and values each join their
  own half. The lone-array rule lives in it too: a lone REFERENCE array spreads
  and a PRIMITIVE one is a single element, which the join must not undo. That
  rule was pinned by a test, which is what caught the helper's first version.
- **`new List[n]` typed as an error while it emitted an `Object[]`.** The
  array-of-collections work taught the emitter and left the mirror.
- **`new LinkedHashMap<>()` typed as an error while it emitted a map.** The
  `LinkedHashMap` work remapped the emitter's table and left this one.

The last two are mine, from this session, and both are the same shape: a
feature taught to one path and not the other. The sweep found them within an
hour of being written, which is the argument for it.

Six programs from the sweep are now part of the checked invariant, so the
shapes cannot silently regress.

### The same program, written with qualified names (2026-08-14)

Seven separate defects this session were one assumption — that a type's name
has one segment — and every one was found by accident, from a probe that
happened to spell a name in full. This asked the question on purpose: 62
programs, each written twice, once with `List`/`Map`/`Optional` and once with
`java.util.List`/`java.util.Map`/`java.util.Optional`, requiring that the
qualified form compile, that both forms agree, and that both match a JDK.

**Zero qualified-only failures.** The axis is closed for everything the sweep
covers: declarations, type arguments, parameters, returns, fields, statics,
supertypes, casts, `instanceof`, `catch`, lambdas, method references,
anonymous classes, for-each, generics and wildcards.

Two failures it did surface were shared by both spellings, so they were
ordinary gaps rather than qualifier bugs:

- **`Optional.ofNullable(null)` typed as an Optional that adopts its context
  in the emission path only.** So the chain `ofNullable(null).orElse(d)` was
  fine as a `println` argument — which types itself — and "cannot determine the
  type of an argument" when passed to a method of one's own. The eighth
  `type_of`-versus-emit divergence of the round, and mine: the emission half
  landed two units earlier without its mirror.
- **A qualified method reference chose the wrong SHAPE.**
  `java.lang.String::length` compiled to `java.lang.String.length(p0)`, a call
  ON the type rather than THROUGH it. The qualifier is canonicalized now, so
  every spelling goes through the same static-versus-unbound decision.

That second fix has a trap worth recording: keying the choice on "the path has
more than one segment" makes every package-qualified reference unbound, which
breaks `java.lang.Integer::parseInt` — a STATIC reference. A nested type
(`Map.Entry`) and a package-qualified one (`java.lang.Integer`) are told apart
by whether the CANONICAL name still contains a dot. Both are pinned, because
the bug and the fix look alike.

### A method reference through a nested library type (2026-08-14)

`Map.Entry::getKey` — the reference every stream over an `entrySet()` reaches
for — compiled to `Map.Entry.getKey(entry)`, a STATIC call on a type that has
no such static. The qualifier test recognized a one-segment class name, so a
two-segment one fell through to the BOUND form, where the qualifier is the
receiver. `String::length` and `Person::getName` were fine either side of it.

The receiver parameter keeps the type the SAM gives it — the stream's element —
rather than the raw qualifier, which is what lets `collect(joining(...))` have
Strings to join.

`canonical_library_class` could not answer whether `Map.Entry` names a type: it
splits at the last dot and asks the PACKAGE table, so a bare `Map.Entry` asks
about a package called `Map` and gets nothing. `nested_library_class` asks the
nested table too, and accepts both spellings.

Still refused: `Comparator.comparing(Map.Entry::getKey)`. The key extractor's
parameter is typed by the surrounding factory, which has no element to give, so
the reference resolves `getKey()` against `Object`. Both spellings a program
actually writes — `Map.Entry.comparingByKey()` and the lambda `e -> e.getKey()`
— work, and so does the same reference in a stream, where the element is known.

### LinkedHashMap and LinkedHashSet (2026-08-14)

The last absent class the legal-Java sweep found, and it cost one flag.

`JavaHashMap` already STORES its entries in insertion order — the bucket order
a `HashMap` iterates in is DERIVED on top, so that the JDK's exact ordering can
be reproduced across resizes. A `LinkedHashMap` is that same structure with the
derivation skipped. Everything built on the iteration order — `toString`, the
three views, for-each, streams, the compute family, `equals`/`hashCode` across
kinds — followed without being touched.

What did need saying explicitly:

- **The class a program can see.** `getClass()` and `instanceof` are the only
  ways besides the order to tell the two apart, so the object reports
  `LinkedHashMap`, and a `LinkedHashMap` answers `instanceof HashMap` (it
  extends one) while a plain `HashMap` is no `LinkedHashMap`.
- **The sizing constructors start from `default()`**, so
  `new LinkedHashMap<>(32)` and the copy constructors lost the flag and
  silently iterated in bucket order. Each now carries it across — the
  int-capacity one by reading it back off the receiver, which was already built
  for its class.

The order rules pinned: re-putting an existing key keeps its original position,
removing then re-adding moves it to the end, and a copy constructor iterates
the SOURCE in the source's own order — so `new LinkedHashMap<>(aHashMap)` is in
bucket order and stays that way.

The compiler treats both as the TYPE they already had (`Map`, `Set`): the
distinction lives in the object, not the type, so only the class named by `new`
differs.

### An array of collections (2026-08-14)

`new List[n]` — the bucket array — was refused with "arrays are not yet
supported by caturra", which a program that had just declared a `String[]`
could only read as nonsense. That message is the last-resort arm for a type
that does not resolve, and it was the ELEMENT that did not resolve, not the
array.

Four gates in a row had to learn the same fact, each with its own symptom:

1. **The type.** An array's element type mapped from a resolved `JType`, with
   no arm for a collection. Now interned as a `Nested` element, the way
   `List<List<Integer>>` already was.
2. **The creation opcode** — a panic, "reference elements are ANEWARRAY". A
   collection element is a reference; its descriptor names the class.
3. **The assignment.** `List<String>[] a = new List[2]` is an unchecked
   conversion, and the only way to build such an array at all, since
   parameterized array creation is illegal. This is the gate the code had
   already been burned by: its comment reads "this matrix gates separately from
   `widens`, so both need the arm — the same trap that once left List ->
   Collection widening half-implemented". With only the `widens` half the
   message was the nonsense "Object[] cannot be converted to Object[]".
4. **The element READ**, both indexed and as a for-each variable, which came
   back as `Object` — so `buckets[0].add(x)` found no method.

Still refused: storing a PARAMETERIZED collection into a RAW-element slot
(`Map[] raw; raw[0] = new HashMap<String, Integer>()`). The raw element
resolves to `Map<Object, Object>` and this pass cannot tell that shape from a
written one; allowing it would allow `Map<Object, Object> m = new
HashMap<String, Integer>()`, which javac rejects. The whole program was refused
before this work, so the limit is narrower than the one it replaces.

### A wrapper lower bound (2026-08-14)

`List<? super Integer>` accepted only a `List<Object>`. The `? super`
machinery was there and correct for a user class, and it dropped the bound
entirely when the bound named a WRAPPER: wrappers are elements, not classes in
the table, so there was no `ClassId` for `Lower` to hold and the whole wildcard
fell back to a plain `Object` element. The `List<Number>` and `List<Integer>`
a JDK takes were refused — which is the entire point of writing `? super`.

The bound now carries the PRIMITIVE kind (an `ElemType` holds a wildcard, so
nesting one inside `WildcardBound` makes the two types recursive), and the test
is asked in the direction a lower bound means: does the BOUND fit the argument
element — the wrapper itself, or a face it widens to (`Number` for the numeric
ones, `Object`, `Comparable`). So `List<String>` is still no `List<? super
Integer>`, and `Character` still does not widen to `Number`; both are pinned.

### A lambda that returns a lambda (2026-08-14)

`x -> y -> x + y` was refused. A lambda's body was desugared with NO expected
type, so the inner lambda had no functional-interface position to sit in —
while a METHOD returning the same lambda compiled, its declared return type
supplying what the lambda's own result type did not. The body is now desugared
against that result type, in both lambda builders.

A BLOCK body (`x -> { return y -> x + y; }`) needed a second step: it reaches
its statements through the ordinary walk, which knows no expected type, so the
returns are target-typed first and the block is walked after — by which time
the inner lambda is already a class. Returns nested in an `if`, a loop, a
`try` or a `switch` are reached the same way `coerce_returns` reaches them.

Three levels (`Function<Integer, Function<Integer, Function<Integer,
Integer>>>`) still refuses: caturra models one type parameter per class, so a
doubly-nested type argument has nowhere to go. Two is what curried code
actually writes.

Also: `java.util.Optional<String> o` lost its element type, because that lookup
compared the written base against `"Optional"` while the list, map and stream
lookups beside it all take the last segment — so a lambda on a qualified
Optional had no target though the simple spelling worked. That is the sixth
place this session where a qualified name was not treated as the same type as
the simple one; the others were the lambda pass's library receivers, the
bundled interfaces in a supertype, a nested SAM, the `Map.Entry` static
receiver, and a nested type named through its top-level class.

### A stream with a name (2026-08-14)

`Stream<T>` became a nameable type in round 8. Almost nothing could be done
with one, and the pieces were separate defects that each looked small:

- **A lambda on a stream held in a VARIABLE, a parameter or a field was
  refused**, though the identical inline chain compiled. The element walk
  followed a chain of calls and had no base case for a name — so giving a
  stream a name took its lambdas away, which undercuts the point of the type
  being nameable.
- **`Stream.of(...)` used straight as an ARGUMENT had no type.** The stream
  sources are built by their own emitter, so their static table is empty and
  `type_of` found nothing; passing the same stream through a variable first
  worked. The fifth `type_of`-versus-emit divergence of the round.
- **`IntStream`, `DoubleStream` and `LongStream` resolved as types but had no
  DESCRIPTOR**, so a method taking or returning one was "unknown type" — a
  variable could hold one, a signature could not name one.
- **`Stream.empty()` typed as `Stream<Object>`**, so `Stream<String> s =
  Stream.empty()` was an incompatible assignment. It now adopts its context,
  as `Optional.empty()` and `Collections.emptyList()` do.

That last one is worth noting as a near-miss: fixing it in `type_of` alone
made the two paths disagree, and the disagreement surfaced immediately as
"incompatible types: Stream<Object>". The emission path and its mirror have to
move together — which is why the invariant is checked rather than remembered.

### A functional interface as a collection element (2026-08-14)

`List<Runnable>` and `Map<String, Function<Integer, Integer>>` — the callback
registry and the strategy table, which are most of why a program keeps a
collection of functions — were refused. The reason given was honest
("`Runnable` works as a variable, but caturra does not model it as a
collection element") and had stopped being necessary: a functional interface
erases to a bundled `__`-one, and that IS a class in the table, so the element
is an ordinary reference to it — the same thing a `Runnable` variable already
held. Both the plain (`Runnable`) and parameterized (`Function<A, B>`) spellings
now map to it.

Storing a LAMBDA there took a second half, and it was the half that mattered:
with the element type accepted, `rs.add(() -> ...)` still had no
functional-interface position to sit in. The element type IS the lambda's
target type, and only the call site knows it, so `add`/`set`/`put` hand it
down — which means a method reference and a user-declared interface take the
same route without further code. `Comparator.naturalOrder()` stored in a
`List<Comparator<String>>` needed only the first half, which is what showed
the two were separable.

Handing a type down must not turn every argument into a functional-interface
position: `List<String>.add(() -> "x")` and a stored lambda of the wrong arity
are both still rejected, and pinned.

### A stream reads its source when it runs (2026-08-14)

A sweep of the static utility surface — 64 programs across `Arrays`,
`Collections`, `String`, `StringBuilder`, the wrappers, `Math` and `Objects` —
came back 62 clean. The two that did not were small, and chasing one of them
found something that was not.

**A stream is LATE-BINDING** (`java.util.stream`, package documentation): its
source is read when the TERMINAL runs, not when the stream is built. caturra
captured the elements at construction, so anything done to the source in
between was invisible:

```java
List<Integer> l = new ArrayList<>(List.of(1, 2));
Stream<Integer> s = l.stream();
l.add(97);
s.count();          // 3 on a JDK; caturra answered 2
```

All three shapes were wrong — an element replaced, an element added, an array
written through — and each is a wrong answer with no error. The stream already
recorded its origin collection (for fail-fast), so the fix is to re-read
through that origin when the terminal pulls. Arrays are recorded as origins
too, which is what makes `Arrays.stream(a)` see a later write, and they are
exempt from the comodification check because an array's length cannot change.
The two sources that must NOT be re-read are the fresh stream `sorted`
materializes — its order is its own — and `Stream.of(...)` over a synthetic
varargs array; neither records an origin, so both keep their vector.

**`Collections.disjoint`, `indexOfSubList` and `lastIndexOfSubList` had no
type.** Each was missing from `type_of`'s mirror of the Collections statics,
so it typed as an ERROR while emitting fine —
`println(Collections.disjoint(a, b))` worked, because that overload is chosen
on the emitted type, and passing the same call to a method of one's own was
"cannot determine the type of an argument". The fourth `type_of`-versus-emit
divergence of the round, and the reason that invariant is now checked.

Still refused: `Arrays.stream(a, from, to)`. A stream's origin is a whole
collection or array with no room for a range, and lowering the range to a copy
would quietly drop the late binding just established. The refusal now gives
that reason and names the spelling that works
(`Arrays.stream(Arrays.copyOfRange(a, from, to))`) instead of reporting "no
suitable method", which reads as though the program were wrong.

### Optional's null contract (2026-08-14)

Surveying the type rather than the one method a probe named again — and most
of it was already right (`map` with a lambda, `filter`, `ifPresent`,
`orElseGet`, `orElseThrow`, `isEmpty`, the `OptionalInt` family). Three things
were not, and the null contract was wrong in all three directions at once:

- **`Optional.of(s)` with a null `s` built a PRESENT Optional holding null**,
  so `isPresent()` answered `true` where a JDK throws NPE from
  `Objects.requireNonNull`. A silent wrong answer, and the null would surface
  from a later `get()` with nothing left to blame it on.
- **`Optional.of(null)` was a COMPILE error**, though javac accepts it and the
  throw is a runtime one.
- **`Optional.ofNullable(null)` was a compile error too** — the empty Optional
  is the entire point of that method.

All three came from one place: a `null` literal has no element type, so the
factory reported "Optional.of cannot hold null", a message that describes
caturra's representation rather than the program. Both factories now accept
it, `ofNullable(null)` types like `empty()` (an Optional with no element
adopts its context, so `ofNullable(null).orElse(d)` resolves), and the VM
throws for `of`.

**A method reference was refused everywhere a lambda was taken.**
`optional.map(String::toUpperCase)`, `filter(String::isEmpty)`,
`ifPresent(System.out::println)` — each with the false claim that this is not
a functional-interface position, while the equivalent lambda compiled. The
list methods had already been given the gate that accepts both; the Optional
ones had not. An `Optional.of(x)` used STRAIGHT as a receiver also had no
element type, so the identical chain over a DECLARED Optional worked and the
inline one did not.

**`flatMap` was missing.** Its function already answers an Optional, so that
answer IS the result — wrapping it would give `Optional[Optional[x]]`, which
is the whole difference from `map`.

Recorded, not fixed: a `map` ERASES its result element, here and in a
`Stream` alike, so `opt.map(String::toUpperCase).get().length()` does not
compile. The element would have to be inferred from the function's own result.
It predates `flatMap`, which inherits it rather than adding it.

### Half of java.util.function (2026-08-14)

Surveying the package rather than the one interface a probe happened to name:
the eight OBJECT-typed interfaces all worked, and all fourteen PRIMITIVE
specializations were "cannot find symbol" — `IntUnaryOperator`, `IntPredicate`,
`IntSupplier`, `IntConsumer`, `IntBinaryOperator`, `IntFunction`,
`ToIntFunction`, their `Double` and `Long` counterparts, `BooleanSupplier`,
plus `BiPredicate`. Half a package being nameable is not a distinction a
program can be expected to keep track of, and the split was invisible from
inside: each of the missing ones erases onto a SAM that already existed.

**The design decision was where to put the method name.** Every specialization
has the same shape as one of the bundled SAMs — one argument and a result, two
and a result, a test, a sink, a source — and differs only in what its method is
CALLED (`applyAsInt` rather than `apply`, `getAsInt` rather than `get`). The
short version is to add those names to the shared interfaces as defaults. That
was written, and it worked, and it made five programs compile that javac
rejects: `aFunction.applyAsInt(x)`, `aSupplier.getAsInt()`,
`aBiFunction.test(x, y)`, `anIntSupplier.get()`, `anIntUnaryOperator.apply(x)`.
Sharing a type is sharing its whole surface. So each specialization gets its
own erased interface, and the five are pinned as rejections.

Their PARAMETERS are `Object` like every other erased SAM here. Declaring the
real primitive was tried first and every specialization that takes one failed
with "Lambda$1 is not abstract and does not override" — the lambda builder
emits `Object` parameters and casts them back in the body, which is why
`ToIntFunction` (whose parameter is already `Object`) was the one shape that
worked. Only the method name and the RETURN are specialized, which is all a
program can observe.

~~Not included: the default COMBINATORS on the primitive forms~~ — **added
later**; see "`java.util.function` as a set of real types" below. The reason
recorded here (a helper class per interface) was real and expired: the classes
are mechanical, and their absence left a whole family unusable.

### An entrySet you can keep (2026-08-14)

Chasing the missing `Map.Entry.comparingByKey()` found something worse beside
it, which is the usual reason to chase a small gap.

**Copying an `entrySet()` produced a list of its KEYS.** `new
ArrayList<>(map.entrySet())` printed `[a, b]` where a JDK prints `[a=3, b=2]`
— a wrong answer with no error. Iterating the view builds its entries on the
way past, so this was invisible until a program kept one. The element walk
behind every copy takes `&self` (most of its callers are comparing two
collections while holding a borrow) and an entry has to be ALLOCATED, so the
entries kind fell through to the arm that answers keys. The copying paths now
use a materializing walk and the comparing paths keep the cheap one. The
entries are live, so `setValue` through a copied one writes to the map.

**The element TYPE went with it.** `Map.Entry` was carved out of the type
argument mapping, as "not a value element — it only names an entrySet's
type". That was true when `Set<Map.Entry<K, V>>` was the only way to write it,
and stopped being true the moment a program could hold a `List<Map.Entry<K,
V>>`: every element read back as `Object`, so `es.get(0).getKey()` did not
compile. A third expired justification this round.

**Then the factories.** `Map.Entry.comparingByKey()` / `comparingByValue()`
order entries by key or value; the value is read through the entry's MAP, as
`getValue` does, so sorting after a `put` orders by the new value. Reaching
them needed the receiver `Map.Entry` to resolve as a static-call target — and
that resolver, in both the emission path and its `type_of` mirror, assumed a
one-segment receiver. The fourth single-segment assumption found today, after
the lambda pass's library receivers, the bundled interfaces, and the nested
SAM map.

This one adds the FIRST known entry to the "more permissive than javac" list:
`Map.Entry.comparingByValue().reversed()` without a type witness is a javac
error (inference fixes `Entry<Object, V>` before the target type can correct
it) and caturra accepts it, because the parser discards a type witness and the
two spellings are the same tree here. All three forms javac accepts do work,
so nothing legitimate is blocked. It is asserted by `looser_than_javac!`
rather than described, so it cannot be forgotten.

### The name a type is written under (2026-08-14)

Batch 3 of the legal-Java dimension (67 programs: initializers, interface
member kinds, enums with bodies, try-with-resources, inner-class forms, arrays,
numeric literals) found NOTHING — the constructs a CSA course reaches for are
covered. Batch 4 aimed at the thinner parts instead and found six, four of
which were one theme: **a type is refused when written under a name the
program is entitled to use.**

**`Iterator<T>` in a method signature.** A descriptor has two builders — one
from the written syntax (`push_type`), one from the inferred `JType`
(`descriptor_of`) — and they disagreed. `static String f(Iterator<String> i)`
was emitted under one descriptor and CALLED with `Ljava/util/Iterator;`:
"malformed class Main: no static method f(...)", for a signature javac
accepts. A FIELD of the same type worked, which is what kept it hidden. This
is the `type_of`-versus-emit divergence one layer down, and the same lesson:
two computations of one fact drift unless something makes them agree.

Fixing it needed a distinction the class table could not make. The guard was
`!has_class("Iterator")`, which reads as "the library owns this name" and
means "nobody does" — the bundled interfaces are in the table too. The
predicate that was missing, `declares_class`, asks whether the PROGRAM
declares it; a user interface named `Iterator` still shadows the library one.
Its doc comment was already in the file, orphaned above an unrelated method,
which is a fair sign it had been intended and lost.

**A qualified library supertype.** `implements java.util.Iterator<T>`,
`java.lang.Iterable<T>` and `java.lang.Comparable<T>` were all "cannot find
symbol", about classes the JDK has: the bundled interfaces are registered
under their SIMPLE name, and the lookup used the written spelling.
`java.lang.Object` had already been carved out by hand a few lines above the
lookup — the same problem, solved one name at a time.

**A lambda targeting a nested interface.** `Outer.Inner i = () -> 5` was
refused as though the position were not a functional-interface one, while the
anonymous-class form of the identical target compiled — which is what made the
gap look like a rule about lambdas. Nested interfaces hoist to the top level
under their simple name and the lambda pass keyed its SAM map to match, so the
qualified spelling javac REQUIRES from outside the enclosing type missed. The
map now holds every dotted suffix of the binary name, with the bare simple name
yielding to a top-level interface that owns it, so a top-level `Same` and a
nested `Holder.Same` each keep their own SAM.

**A nested type named through the top-level class.** `Main.H.Inner` did not
resolve, though `H.Inner` and `Main.C` did: only `{enclosing}.{name}` was
registered as a source spelling, which coincides with the full path exactly
one level down. Every dotted suffix of the binary name is now registered.

The two that were NOT this theme are recorded rather than fixed: `LinkedHashMap`
and `IntBinaryOperator` are absent from the library.

One correction to the batch-4 reading above: the `Map.Entry` failure was
reported as the type name, and it is not. `Map.Entry<K, V>` works as a
for-each variable, a local, a parameter and a `List` element — the shapes a
program actually writes. What is missing is the pair of STATIC factories,
`Map.Entry.comparingByKey()` and `comparingByValue()`. A refusal names the
line it happened on, not the feature that is absent, and the probe that
provoked it used both at once.

### A covariant return, and a lambda after a qualified call (2026-08-14)

Two refusals of legal Java, found by the dimension that writes ordinary
programs and asserts only that they COMPILE.

**A covariant return was refused.** `String f()` overriding `Object f()` is
JLS 8.4.8.3, and every CSA textbook has it. The recorded justification —
dispatch is by descriptor, so this needs a bridge method that cannot be written
in source — had expired: the VM resolves an override by name and arity, so
there is no bridge to write. The refusal outlived its cause, which is the same
failure mode as the three stale "honest reasons" found earlier this round.

Both override sites had to be fixed, and both were wrong for one reason. Each
tested covariance by matching `(JType::Object(sub), JType::Object(sup))`.
caturra gives `String`, arrays and the collections their own `JType` variants,
so that pattern only ever matched user-declared classes and fell straight
through on the textbook case. The test is now `widens` between any two
REFERENCE types, which also keeps the two directions that must stay errors: a
return that does not widen to the overridden one, and a primitive return
(`int` does not override `long`, though it widens as a value).

Dispatch was the thing to verify, not acceptance, since a wrong answer here
would be silent. Pinned: the call through a supertype reference, through the
interface, from an interface DEFAULT method, through a `List<I>` for-each, and
`super.f()` still reaching the overridden method.

**A lambda after a fully qualified call was refused.** This compiled:

```java
Arrays.asList(1, 2, 3).stream().filter(v -> v > 1)
```

and this did not:

```java
java.util.Arrays.asList(1, 2, 3).stream().filter(v -> v > 1)
```

with "a lambda or method reference is only allowed where a functional-interface
type is expected". The lambda pass recognizes a library receiver in order to
type the lambda's parameter, and every one of those tests compared a
single-segment path — so writing the package name, which is always legal, hid
the receiver and left the lambda with no target type. Now routed through one
`names_library_class` helper that accepts either spelling, with the qualifier
required to start at `java` or `javax` so a user class named `Arrays` is
unaffected.

### A library Class handle stops misreporting itself (2026-08-14)

Chasing the one remaining reflection divergence — `Class.forName` on a library
class — found three worse things beside it, which is why it was worth chasing.

- **`getName()` was the SIMPLE name for most library classes.** `Math.class`
  reported `Math` where a JDK reports `java.lang.Math`, and so did
  `StringBuilder`, `ArrayList` and the rest; only the wrappers, `String`,
  `Object` and `Number` were qualified. The class literal now takes its name
  from the import table, which is the same source the compiler already resolves
  imports against.
- **`getSuperclass()` answered null**, so the `getName()` a program writes next
  threw NullPointerException. A library class now answers its real parent: the
  six numeric wrappers extend `Number`, a throwable follows the exception
  table, and the collection hierarchy (`ArrayList` to `AbstractList`, and so
  on up) is recorded from a real JDK rather than flattened to `Object`.
- **An array reported no interfaces**, where every array implements exactly
  `Cloneable` and `Serializable`.

With the handle honest, `Class.forName` follows: a CONSTANT name resolves at
compile time through the import table, so there is one source of truth and no
registry to drift. A computed name still reaches the VM, which answers for user
classes and reports `ClassNotFoundException` otherwise — the safe direction.

**One attempt was reverted, and the corpus is why.** Enumerating the members of
a library class (`String.class.getDeclaredMethods()`) returns an empty array
where a JDK returns 90, and an empty array is the worst kind of wrong: a
harness that loops over it concludes the method is absent. Refusing outright
seemed obviously better — and broke seven org.code levels, because validators
call `getDeclaredFields()` on a superclass that is usually `java.lang.Object`,
where **empty is the correct answer**. The refusal traded a measured
correctness for a speculative one. It stays recorded rather than guessed at:
the fix is real member data for modelled library classes, not a blanket
refusal.


### type_of must agree with what is emitted (2026-08-14)

The most productive defect class of this round was not a missing feature. It
was **two resolution paths that had to agree with nothing making them**:
`type_of` predicts an expression's static type; `expr` emits it and returns
what it left on the stack. `type_of`'s own doc comment has said "must agree
with what `expr` leaves on the stack" for as long as it has existed, and
nothing ever checked it. Three defects came from that gap in this round alone,
including one where a whole statement VANISHED — no code, no value, no
diagnostic.

The invariant needs no oracle. `expr` now predicts, emits, and compares, behind
`CATURRA_VERIFY_TYPES` (off by default, one relaxed load otherwise), reporting
every disagreement rather than stopping at the first.

**The excuse rule is the part worth reading.** The obvious version excuses a
disagreement when either side is `Error` — and that would have made the check
blind to all three defects, because their shape is exactly `type_of` answering
`Error` where the emitter succeeds. Only an EMIT error is excused: that one was
reported to the user by whoever produced it. A `type_of` error is silent by
contract, which is what makes it dangerous — a caller consulting it bails, or
picks the wrong overload, and says nothing.

Swept over all 2,698 corpus levels and the 724 differential programs, it found
two more:

- **`Method.invoke` and `Constructor.newInstance` were not typed at all.** The
  emitter intercepts them before any method table and answers `Object`;
  `type_of` did not, so a legal program that merely PASSED an `invoke` result
  to a method was refused. 103 sites in the corpus, all in grading harnesses,
  which had survived only because they cast the result immediately.
- **The immutable factories (`List.of`, `Set.of`, `Arrays.asList`) typed as
  `null`.** Latent rather than live — the emitter drove the overloads that
  mattered — but the same shape.

The check is asserted as a test over the shapes that have broken it, so the
invariant now has to hold rather than merely be documented.

### A builder's capacity (2026-08-14)

The refusal said caturra "does not model a builder's capacity, only its
contents", and gave that as an honest reason. It had stopped being a reason:
capacity is fully determined — 16, or `n`, or the seed's `length + 16`, growing
by `(old << 1) + 2` or straight to what is needed, and shrinking only on
`trimToSize`. All verified against a real JDK.

It cannot be derived from the contents, though, because it depends on HOW the
builder was built: appending ten characters one at a time leaves the initial
16, while appending forty at once jumps to 40. So it is stored — in a side
table on the heap, following the exception traces, rather than as a field on
the `StringBuilder` variant. That variant is a bare `Vec<u16>` and shares
or-patterns with `JavaString` at a dozen sites; a struct field would have
churned every one of them for a number none of them care about.

Growth lives in `builder_store`, the single point every mutation already passes
through, so no operation can grow the contents without the capacity noticing.

Two smaller things came with it: `new StringBuilder(String)` was seeding
through a Rust `String` and so replacing an unpaired surrogate rather than
copying it — the last instance of that pattern — and the strictness list is one
shorter, since a program using `capacity()` is no longer refused.


### A char literal is a code unit (2026-08-14)

The last case, and the bound was wrong about where it lived. The pre-lex pass
(JLS §3.3) already did the right thing — it leaves an unpaired surrogate escape
in place, since a literal is the only context where one means anything. What
lost it was the TOKEN: `CharLiteral` and `Literal::Char` each held a Rust
`char`, so `'\uD83D'` became U+FFFD before the parser ever saw it. Both now
carry a `u16`, which is what a Java `char` is.

Most of the twenty-odd sites converted untouched, because `u32::from` accepts a
`u16` as readily as a `char`. Three did not, and each was a small lesson:
narrowing a constant `int` to a `char` no longer needs to be a valid scalar
value; the constant STRING form of a char has to stay `None` for a surrogate
rather than invent a rendering, which is what folding `"" + '\uD83D'` depends
on; and `Character.toString` takes a CODE POINT in its `int` overload, so a
supplementary one must become a surrogate PAIR — truncating it to one unit
broke `Character.toString(0x1F600)`, which the suite caught.

One case is left, and it is genuinely the last: a lone surrogate inside a
STRING literal (`"x\uD83Dy"`). `Literal::Str` is a Rust `String`, and unlike
the char literal that is not a one-field change — a string literal's text flows
into the constant pool, class names and diagnostics. Every other route by which
a surrogate can enter a program, from data or from a char literal, is exact.

### Unicode classification, carried rather than derived (2026-08-13)

The largest measured-open divergence left: `Character.isLetter`,
`isUpperCase`, `isLowerCase`, `isAlphabetic`, `isDefined`, `isWhitespace`,
`isSpaceChar`, `isDigit` and `isLetterOrDigit` disagreed with a real JDK on
**4761 BMP units**.

Two causes, both about DATA rather than logic. A JDK carries the Unicode
version it shipped with — 11 carries Unicode 10 — while Rust's tables track the
current one, so Rust knows Georgian Mtavruli and the other additions of 11
onwards. And Java's predicates are defined over general CATEGORIES plus the
`Other_Uppercase`/`Other_Lowercase`/`Other_Alphabetic` properties, which is not
the same partition as Rust's `is_alphabetic`/`is_uppercase` — chiefly over the
combining marks.

So the categories are now carried, not derived: `Character.getType` recorded
from a real JDK 11 over the whole BMP, as 2858 contiguous runs, with the three
`Other_*` property sets and the two whitespace adjustments beside them. Every
predicate is derived from that table exactly as the JDK derives it, which also
makes these answers **independent of the Rust toolchain's Unicode version** —
the previous fix, the 15-range case-mapping patch, was not.

All 4761 units now agree, and `Character.getType` itself became answerable, so
it is exposed. The whole table is pinned as a differential test rather than a
spot check: a regenerated or drifting table cannot pass it.

Everything below U+0295 had always been exact, which is why no student program
ever showed this — and why it took a probe whose alphabet went past ASCII to
find.

### The wrapper statics at volume (2026-08-13)

A nineteenth dimension: `Integer`/`Long`/`Double`/`Float`/`Character`/`Boolean`
statics — parsing, radix conversion, bit twiddling, comparison — **9,179
lines**, with an alphabet deliberately carrying unicode digits, an astral
character, signs and overflow, because the String sweep had just shown what a
narrow alphabet hides.

The probe stopped on a **`VerifyError`**, which by this project's rule is
always caturra's bug, and it was: **`Integer.decode` answered an `int` where
Java answers an `Integer`.** The compiler already types it as a wrapper and so
does not box, and a raw int left where a reference belongs stays invisible
until the value reaches an `Object` — `Object o = Integer.decode("7")` and then
using `o`, or a `Supplier<Object>` lambda returning one. `Long.decode` had the
same shape.

`decode` also **accepted `--1`**: the first `-` set the sign and the second was
read as part of the number, where the JDK complains "Sign character in wrong
position". Its messages were wrong in two more ways — an empty input is "Zero
length string", not the "For input string" wording, and the JDK strips the
RADIX PREFIX before parsing but puts the SIGN back when it reports, so
`decode("-1.5")` names `-1.5` and not the `1.5` a plain strip leaves.

**The float parsers were folding unicode digits.** Only the integer parsers go
through `Character.digit`; `Double.parseDouble` reads ASCII alone. Sharing one
helper made `Double.parseDouble("٣")` answer 3.0 where a JDK throws — an
accepts-invalid. The integer parsers keep the fold but now also keep the string
as WRITTEN, because the JDK quotes THAT back: reporting the folded form said
`For input string: "3"` about an input of `"٣"`.

Measured and open, both already-known categories:

- **Unicode CLASSIFICATION** diverged on 4761 BMP units over 416 ranges. Closed
  below.
- **Subnormal formatting** now known to affect `Float.toString` as well as
  `Double.toString` — `intBitsToFloat` disagrees on three of the probe's
  values, the same shortest-decimal tail already recorded.

### The String surface at volume (2026-08-13)

An eighteenth dimension put `String` through a cross product — every method
against every input, four entry points, exceptions recorded as class AND
message: **24,371 lines** against a real JDK 11. 253 diverged, in five groups,
and every group was a real defect.

**A statement could VANISH.** `p(java.util.Objects.toString("q"));` produced no
code, no value, no diagnostic — the method was never invoked. `type_of` answers
a call by matching a ONE-SEGMENT receiver path, so a fully qualified receiver
typed as `Error` while the emission path resolved the same call fine; as an
argument that `Error` made the call bail out. This is the `type_of`-versus-emit
divergence in its worst form, and the instance-call path already carried a
"never bail in silence" net for exactly this — the implicit-`this` path did
not. Both are fixed: qualified receivers are normalised once, and the net now
covers own calls too.

**The regex engine matched UTF-16 units, not CODE POINTS.** Java's matches a
whole astral character with `.`; caturra's matched half a surrogate pair, so
`"a😀".replaceAll("a.", "#")` returned a string containing a LONE surrogate —
corrupt output, not merely a wrong count. The 13,728-line regex sweep missed it
because no input carried an astral character.

**`Character.toUpperCase`/`toLowerCase` over-mapped 2188 BMP units.** A JDK
carries the Unicode version it shipped with — 11 carries Unicode 10 — while
Rust's tables track the current one, so Rust uppercases Georgian Mtavruli
(Unicode 11) and the later Cyrillic and Latin additions that JDK 11 leaves
alone. Every divergence ran that one direction, and compresses to 15 ranges;
the last is the surrogate block, where a lone surrogate had been answering as
the replacement character. `compareToIgnoreCase` needed the other half of the
rule: a multi-character FULL lowercase does not mean "no simple mapping" —
`İ` lowercases to two characters in full but to a plain `i` simply, which is
why `"İ".compareToIgnoreCase("i")` is zero.

**The bounds messages are not uniform, and COMPACT STRINGS make that
observable.** `charAt` on a Latin-1 string says "String index out of range: i";
on a UTF-16 one it says "index i,length n" — the same call, worded by how the
receiver happens to be stored. `codePointAt` always names the length,
`codePointBefore` never does, and `codePointCount`/`offsetByCodePoints` throw a
bare `IndexOutOfBoundsException` whose message is null.

Measured and open: a genuinely UNPAIRED surrogate rendered into text through a
boxed `Character` or a `char[]` element still prints U+FFFD where a JDK's
encoder substitutes `?`. Most of this is closed by the case-table work below;
what remains is noted there.

### Hash iteration order, measured (2026-08-13)

A seventeenth dimension took the other surface caturra MODELS rather than
wraps: `HashMap`/`HashSet` iteration order, which is a pure function of the
keys' hash codes, the table length and insertion order — and which a student
sees every time they print a map.

120 printed orders: word, short, duplicate and deliberately-colliding key
sets; every resize boundary from 0 to 100 entries; and the `Integer`,
`Character`, `Long`, `Boolean`, `Double`, negative and `MIN_VALUE` key paths.
**114 matched a real JDK exactly**, including the resize thresholds and the
rule that removal does not re-order what stays. Those 114 are now pinned.

The six that did not are one known gap, and the probe pins its shape: **nine or
more keys in ONE bin of a table of 64 or more**, which takes keys that collide
on purpose. There Java treeifies the bin, and it prints `[tree root]` followed
by the rest in chain order, because `treeify` ends with `moveRootToFront`.

It stays open, deliberately, and the reasons belong in the record rather than a
future re-discovery. A tree's shape depends on the ORDER puts, removes and
resizes happened in, so it cannot be derived from the final entry set the way
chain order can — and `map.rs` is built on exactly that derivation. Building
one also calls `compareTo` on the keys, which for a user class is user code
that module deliberately cannot run. Closing this is a rewrite of the module,
with the grading-corpus baseline riding on it, in exchange for key sets no
ordinary program produces.

### Regex inline flags and named groups (2026-08-13)

The last two families the 13,728-line sweep refused. With these the whole
cross product is byte-identical, and so are five smaller probes written while
closing them: **14,621 lines, zero divergences**.

**Inline flags.** `(?i)`, `(?s)`, `(?m)`, `(?x)`, scoped the way Java scopes
them: a flag runs to the end of the group it sits in, `(?i:X)` limits it to
`X`, and `(?-i)` turns it back off — so every group saves and restores the
flag set around its body. `(?i)` is ASCII-only folding, which is Java's rule
without `u`, and it is applied in ONE place by routing every literal through a
fold-aware node, so `\Q...\E` and the escapes get it for free. Folding has to
happen BEFORE negation: `(?i)[^a]` must reject `A`, and negating each case
separately accepts it.

`(?x)` changes LEXING, not matching, so it is a skip performed wherever a token
is about to be read — Java ignores whitespace inside a character class and
inside `{n, m}` bounds too, which is easy to miss and was the last case to
fall.

**The multiline anchor rule is the surprise.** `"".matches("(?m)^.*$")` is
FALSE, though `^`, `.*` and `$` all look satisfiable at position 0. Java's
`Caret` carries the reason as a comment — *"Perl does not match ^ at end of
input even after newline"* — so a multiline `^` never fires at the end of the
input, which for an empty string is position 0 as well. A CRLF pair is ONE
terminator, so neither anchor fires between the CR and the LF.

**Named groups.** `(?<name>X)`, `\k<name>`, and `${name}` in a replacement.
The four ways to write one wrong carry Java's messages and caret indices,
including that an EMPTY name (`(?<>a)`) reports "does not start with a Latin
letter" rather than a length complaint of its own.

### Regex lookaround (2026-08-13)

A sixteenth dimension took the regex engine at volume, because caturra MODELS
this surface rather than wrapping the JDK's: a defect here is a wrong ANSWER,
not a wrong word. 78 patterns x 44 inputs x 4 entry points (`matches`, `split`,
`replaceAll`, `replaceFirst`) = **13,728 lines**, diffed against a real JDK 11.

The headline is what did NOT diverge. **70 of 78 patterns were byte-identical
across every input and entry point — 12,320 lines with zero wrong answers**,
including the backtracking stress patterns (`(a*)*b`, `(a+)+b`, `(.*)*c`),
backreferences, the greedy/reluctant/possessive trio, and the `$`-before-final-
terminator rule. Every divergence was an honest refusal, in exactly two
families: lookaround and inline flags.

Lookaround is implemented here: `(?=X)`, `(?!X)`, `(?<=X)`, `(?<!X)`. Looking
backwards needs a continuation that succeeds only at a GIVEN position — the
body has to end where the lookaround sits, not merely somewhere after its start
— so `Cont::EndAt` joins `Cont::Done`. A positive lookaround keeps what its
body captured, a negative one matched nothing and so captures nothing.

The width rule is the part worth writing down, because the obvious reading of
it is wrong. Java does NOT refuse an unbounded lookbehind: `(?<=a*)b` compiles
and matches. What it refuses is a body whose width is not knowable at ALL — a
BACKREFERENCE, whose width is whatever another group captured at run time.
caturra first refused both, which is stricter than javac in a place where the
spec says it should not be, and the probe caught it. The bound now only
narrows the backwards search; it is not a rule. The caret index is the body's
last character, one before the closing paren, confirmed against five patterns.

The other two families — inline flags and named groups — are closed below.

### copyOfRange reports arraycopy's bounds (2026-08-13)

A fifteenth dimension swept the `Arrays`/`Collections` utility surface — the
two classes a student reaches for constantly, never checked as a unit: what
`binarySearch` returns for an ABSENT key, whether `copyOf` pads or truncates,
`deepToString` and `deepEquals`, `nCopies`/`frequency`/`swap`/`disjoint`, and
the throwing contracts. 22 of 23 programs matched a real JDK byte for byte.

The one divergence is a message. `Arrays.copyOfRange` does not range-check
itself: past the `from > to` guard it hands the copy to `System.arraycopy`, so
what the program catches is arraycopy's diagnostic, which names the array's
TYPE and length — `arraycopy: source index -1 out of bounds for int[3]`, and
`object array[3]` for a reference array. caturra reported its own
`Array index out of range: -1`, from a different check entirely.

The second shape had to be derived rather than copied: when `from` is past the
end, arraycopy is asked for `min(length - from, to - from)` elements, and
`from > length` makes the first term the smaller and negative — so the message
is `arraycopy: length -2 is negative`, naming a number that appears nowhere in
the call. The exception CLASS and which inputs throw were already right; only
the words were caturra's.

### Every reference type is a subtype of Object (2026-08-13)

A fourteenth dimension — `private`/`static` interface methods (JLS §9.4) and
array covariance depth (JLS §10.5) — came back clean in 26 of 27 programs. The
one divergence was worth the sweep: **an array with an INTERFACE element type
did not widen to `Object[]`**, in assignment and in argument position, while
`Impl[]` and even an abstract class's `Base[]` did.

The cause is one line below the covariance rule, not in it. `is_subtype` walks
a class upward through its superclass and interfaces — and an interface records
no superclass, so the walk can never reach `Object` from one. Every reference
type is a subtype of `Object`, including an interface; that is now answered
directly rather than searched for. `elem_widens_to_class` had already special
cased `Object` at its top, which is the same rule written in one place and
missing from the other.

The rest of the dimension is pinned rather than merely observed: private
interface methods backing defaults, a private *static* one behind a public
static, and the two rejections that keep them private — a `private` method
reached from outside, and a `static` one read through an implementor, which
does NOT inherit it.

### An implementor owes the SUBSTITUTED signature (2026-08-13)

JLS §8.4.8.1: `class Impl implements Box<String>` owes `unwrap(String)`, not
merely something named `unwrap`. caturra compiled an `Impl` declaring
`unwrap(Integer)` and ran it — the last accepts-invalid on the list, and the
one that closes the "more permissive than javac" invariant.

The cause is a rule that was right in the wrong place. `params_override` treats
an abstract type-variable parameter as satisfied by ANY reference — that is the
generic-interface bridge, and it is correct exactly while the type argument is
unknown. Once the class WRITES the argument, the variable is no longer unknown
and the bridge must not apply. The supertype's written arguments were already
recorded (`supertype_args`, for the parameterized-supertype assignment); this
threads them down the supertype walk and substitutes before matching.

Every step is permissive when an argument will not resolve — and one that was
not is what caught it. Filling an unresolved slot with a placeholder made the
placeholder a concrete type as far as matching goes, so it demanded an exact
match and refused `class C extends Mid<String>` where `Mid<T> implements B<T>`.
One unresolvable argument now abandons the WHOLE substitution and falls back to
erasure. A check of this shape fails by over-rejecting ordinary Java.

The diagnostic had a second defect the sweep surfaced: it named no parameters
at all (`compare()` where javac writes `compare(String,String)`), and a TEST
had encoded that as correct — the second time this round a test asserted the
bug. javac names the substituted parameters, which is how a reader tells "wrong
signature" from "nothing at all".

For a BUNDLED interface the substitution cannot help: `__Comparator` really
declares `compare(Object, Object)`, so there is no type variable left to
replace. When such an interface is given exactly ONE type argument, every
erased `Object` parameter renders as that argument — true for `Comparator<T>`,
`Comparable<T>`, `Consumer<T>` and `Predicate<T>`, while a two-argument
`Function<T, R>` is left alone rather than guessed at. That shapes the MESSAGE
only; matching still runs on the erased parameters, so the bridge that lets
`compare(String,String)` implement `compare(Object,Object)` is untouched.

### An impossible cast is refused, not deferred (2026-08-13)

JLS §5.5.1: casting a class to an interface is a **compile error** when the
class is `final` and does not implement it. No subtype could ever satisfy both,
so the cast is provably impossible and javac refuses it outright. caturra
compiled it and threw `ClassCastException` at run time — accepts-invalid, and
the last entry on that list.

The rule turns on `final`, and only on `final`. A non-final class stays legal:
some subclass could implement the interface, so javac defers the decision to
run time and caturra must too. The three casts that remain legal — a final
class that DOES implement the interface, a non-final class, and `Object` — are
pinned alongside the refusal, because a check like this fails by over-reaching.

The message needed the same treatment as the `String` cast: `describe` yields
binary names, so the diagnostic read `X$F cannot be converted to X$I` where
javac says `F cannot be converted to I`. Source spellings, both sides.

That was one of the two accepts-invalid left open by the previous entry; the
other is the one below. Only with BOTH fixed is **"more permissive than javac"
empty again** — the invariant the spec documents.

### A type argument must satisfy its bound (2026-08-13)

A fifth diagnostic catalogue (31 more programs, 206 in all) found three more
accepts-invalid. The one fixed here is the substantial one: **a written type
argument was never checked against its parameter's bound** (JLS §4.5), so
`Box<String>` for a `Box<T extends Number>` compiled. The class table recorded
each parameter's COUNT but not its bound; it now records both, and the check
sits beside the existing arity validation.

Both of the others are fixed below: a cast from a FINAL class to an interface
it cannot implement, and a class implementing `I<String>` without the right
method signature.

### A functional interface's result has a type (2026-08-13)

A fourth diagnostic catalogue found a **compiler panic** — the one failure mode
worse than a wrong answer — and chasing it uncovered a much larger gap.

- **`S::getV` for a zero-parameter `Supplier` panicked the compiler.** An
  unbound instance reference needs the SAM to supply a receiver as its first
  parameter; this SAM has none, and building the call anyway indexed an empty
  parameter list. The diagnostic was already correct; the pass then carried on
  and crashed.
- **Calling a `java.util.function` SAM returned `Object`.**
  `supplier.get().toUpperCase()` was "cannot find symbol" for a program the JDK
  runs. These interfaces worked as lambda TARGETS all along — it was their
  RESULT that had no type, because the source type erased to the bundled
  `__`-interface and dropped its arguments. The result is the LAST type
  argument for every one of them, so one rule covers `Supplier<R>`,
  `Function<T, R>` and `BiFunction<A, B, R>`; the bundled SAMs are retyped to
  return a type variable so the receiver's argument substitutes into it.
- **A constructor takes an access modifier and nothing else** (JLS §8.8.3):
  `abstract`, `static` and `final` were accepted and ignored.

Keeping the result argument made a parameterized functional value stop matching
the raw parameter the collection methods declare — the same both-spellings
lesson as `Iterable`, and the existing suite caught it.

The default combinators are implemented too: `Predicate.negate`/`and`/`or`,
`Function.andThen`/`compose`, `Consumer.andThen` and `BiFunction.andThen`.

They are written in the bundled interfaces as **named helper classes, not
anonymous ones** — an anonymous class in a bundled source shares the `Anon$N`
counter with the program's own, and the two collided: a program's
`new Runnable() { … }` stopped converting to the interface it plainly
implements. The compatibility-page manifest caught that within a run.

The lambda pass keyed its argument target-typing on the method NAME alone, so
three interfaces declaring `andThen` with different parameter types disagreed
and left an inline lambda untyped. It now asks a (class, method) map first,
using the receiver's declared type — which also makes every other same-name,
different-signature pair resolvable.

The lambda's own PARAMETER now comes from the receiver's type arguments too:
`f.andThen(n -> n + 1)` on a `Function<String, Integer>` gives `n` the type
`Integer`, `p.and(s -> s.length() == 0)` on a `Predicate<String>` gives `s` a
String, and the same for `Consumer.andThen` and `BiFunction.andThen`. The
combinator's own signature cannot say this — the bundled SAM takes `Object` —
so the target is synthesized from the receiver's declaration.

### An interface has no initializer block (2026-08-13)

A third diagnostic catalogue (42 more programs, 136 in all) found one more
accepts-invalid and one more leak:

- **An interface body accepted an initializer block.** JLS §9.1.4 has none —
  there is no instance to initialise and its fields are constants — so a block
  of statements could sit inside an interface and never run.
- **A nested class's BINARY name leaked into the override message** at a second
  site: "go() in B cannot override go() in `S$A`". The abstract-method message
  was fixed earlier; this one wasn't, which is what a sweep finds and a
  case-by-case assertion does not.

The rest of that catalogue's differences are caturra being more specific than
javac (naming the method, the class, the escape) and are kept.

### A lambda's arity, and lambdas against qualified declarations (2026-08-13)

Widening the diagnostic sweep by 44 more programs found three more
accepts-invalid, and the fix for one of them exposed a fourth defect.

- **A lambda's arity was never checked against the SAM's.** The two were
  zipped, so the extra parameter was silently dropped and
  `Function<String, Integer> f = (a, b) -> 1;` compiled here — a compile error
  on a real JDK.
- **The lambda pass matches container and functional-interface names BY
  SPELLING**, so a fully qualified declaration found no target at all:
  `l.sort((a, b) -> …)` where `l` is a `java.util.List<String>` was "a lambda
  is only allowed where a functional-interface type is expected". The same for
  a sorted collection's comparator constructor and for
  `java.util.Comparator.comparingInt(…)`. Only `java.*` is stripped when
  matching — a nested user type keeps its qualifier, since flattening that
  could collide with a library name.

Both remaining accepts-invalid are now **fixed**, and the first turned out to
be much wider than it looked:

- **A `catch` body was not type-checked when the guarded region could not
  throw.** caturra emits no handler in that case — correct for codegen — but it
  skipped the body entirely, so EVERY error inside such a catch was invisible:
  `try { } catch (Exception e) { int n = "text"; }` compiled and ran. The
  bodies are now emitted into a jumped-over region, the same shape `assert`
  desugars to. (The multi-catch assignment I was chasing was already refused;
  it only *looked* accepted because my probe's `try` body was empty, which is
  what exposed the real bug.)
- **A modifier may appear at most once** (JLS §8.1.1). Setting the flag twice
  made `public public void f()` compile.

### Diagnostic wording (2026-08-13)

caturra mirrors javac's wording deliberately, because the message is what a
student reads and a different one reads as caturra's bug. That agreement was
asserted case by case and never swept. Fifty broken programs, comparing
javac's first `error:` line with caturra's first diagnostic: **20 identical,
26 different, 0 accepts-invalid** after the fixes below.

Most of the 26 are caturra being MORE specific where javac splits its message
over following lines — javac's bare "cannot find symbol" against caturra's
"cannot find symbol: class Scanner". Those are kept. Three were defects:

- **A cast to `String` accepted any reference source** — the accepts-invalid
  above, found because the sweep expected an error message and got a running
  program.
- **A primitive receiver reported a missing wrapper method.** `int n; n.length()`
  said "cannot find symbol: method length in class `java/lang/Integer`" — the
  wrong diagnosis (javac: "int cannot be dereferenced") *and* a slashed
  internal class name, which a message must never show.
- **A nested class's BINARY name leaked**: "does not override abstract method
  go() in `S$A`" where javac names `A`.

### Queue halves, list iterators, collection algorithms (2026-08-13)

Audit round 10, dimension 12: 11 programs, **no divergence**. Pinned for the
same reason as the dimension above — a `Queue` has two of everything
(`add`/`offer`, `remove`/`poll`, `element`/`peek`), one half throwing where the
other returns null, so picking the wrong half is a silent behaviour change
rather than an error.

Also checked: both ends and both halves of a `Deque` plus its stack face,
`PriorityQueue` poll order and a comparator-ordered one, a `ListIterator`
inserting and overwriting mid-walk (and what the list looks like DURING the
walk), reverse iteration and removal, `removeIf`/`replaceAll` on a list, a set
and a map's values view, a stable sort, `Collections`'
max/min/frequency/reverse/sort/binarySearch/swap/fill/nCopies/disjoint/addAll,
`shuffle` with a seeded `Random`, and `ConcurrentModificationException` from
mutating during a for-each.

### Enums and class initialization (2026-08-13)

Audit round 10, dimension 11: 16 programs, **no divergence**. Pinned because
every rule here is observable only through a side effect in a static
initializer — a wrong answer would be silent, and this is the shape of code
that would show it.

What was checked: a class initializes on FIRST ACTIVE USE, so reading a
**constant variable** does not initialize its class (the value is inlined at
compile time) while reading any other static does; a static read through a
SUBCLASS initializes the class that declares it, not the subclass;
a superclass initializes before its subclass; instance initializers run in
declaration order, interleaved with field initializers, before the constructor
body. And for enums: declaration order for `ordinal`/`compareTo`, a FRESH
`values()` array each call, `valueOf` throwing `IllegalArgumentException` for
an unknown name and NPE for null, enums as `TreeMap` keys and as
`Collections.sort` elements. Five programs javac rejects — instantiating an
enum, assigning a constant, extending a class, a qualified `case` label, an
abstract method with no constant body — are refused too.

### A generic method returning a container (2026-08-13)

`<T> List<T> listOf(T value)` — how every generic factory is written — came
back as a `List<Object>` and would not assign to the `List<String>` the call
plainly produces. The return-inference plan only understood a **bare** `T`
return; a container of it was not recognised at all.

Which shape to apply the inference to is read off the declared return itself:
`List<T>` erases to a list whose element is the erased variable, and the
inferred type takes that element's place. Three erased spellings count — the
bare variable, the top type, and the wildcard the parameter form leaves behind
— while a REAL wildcard (`? extends Number`) is deliberately left alone, since
its bound is a written constraint rather than an erased variable.

Two gaps found alongside and left open, both refusals rather than wrong
answers:

- **An explicit type witness that DISAGREES with the arguments is ignored.**
  `S.<Object>listOf("y")` infers `List<String>` from the argument and then
  refuses the assignment to `List<Object>`. `Expr::Call` carries no type
  arguments at all, so the witness is parsed and discarded; a witness that
  agrees with the arguments (the ordinary case) works.
- ~~**`<T> Optional<T> maybe(T v) { return Optional.of(v); }` does not
  compile**~~ — **fixed.** `Optional` was simply left off the list of
  containers whose elements match by the variance rule, so a generic method
  could not hold its own result: "Optional<Object> cannot be converted to
  Optional<Object>", the two spellings of an erased element printing alike and
  comparing unequal. Both gates needed the arm — `widens` and the conversion
  matrix — which is the same two-gate trap recorded above for `Iterable`.

### The Java 9-11 additions a Java 11 engine owes a program

Probing the surface the target itself defines — the APIs Java 9, 10 and 11 add
— found five gaps, and following the first of them found a much older one.

**`takeWhile` / `dropWhile` (Java 9)** join `filter` on both stream shapes as
`StreamOp::TakeWhile` and `StreamOp::DropWhile`. Because the pipeline is lazy,
`takeWhile` says "stop" to the source at the first element that fails rather
than filtering the rest, which is the whole difference from `filter`:
`numbers.stream().takeWhile(n -> n < 5)` over `1, 2, 5, 1, 6` is `[1, 2]` while
`filter` gives `[1, 2, 1]`. `dropWhile` latches once its predicate first fails
and passes everything after.

**`Stream.ofNullable` (Java 9)** is a one- or zero-element stream.

**`List.copyOf` / `Set.copyOf` / `Map.copyOf` (Java 10)** are NOT copy
constructors: what they answer refuses every mutator and does not follow its
source. They route through the same VM factory as `List.of`, which is where
the null rejection already lives; the one difference is that a duplicate is
DROPPED here (a source list may legitimately hold two equal elements) where
`of` throws `IllegalArgumentException`. The element of the result comes from
the SOURCE, not from reading the argument as an element — treating
`List.copyOf(aStringList)` the way `List.of` treats its arguments made it a
`List<Object>`, and the checked `type_of`-vs-emit invariant caught the map half
of exactly that mistake before it reached a program.

**`Predicate.not` / `Predicate.isEqual`** are the bundled `__Negate` and a new
`__IsEqual` (which compares TARGET-first, and tests for null when the target is
null, as the JDK does). `not` is transparent to target typing, and that is the
subtle part: in an ARGUMENT position — `filter(Predicate.not(String::isEmpty))`
— there is no `Predicate<String>` anywhere to read the element from, so the
erasure that types the inner lambda has to reach THROUGH the negation. It does
so at one site, inside `build_erased_lambda`, with the guards that admit it
naming only the positions where a predicate is what is wanted; spelling it more
loosely would have accepted `forEach(Predicate.not(...))`, which javac rejects.

Following it turned up a gap that predates all of this: the element type did
not flow through a predicate COMBINATOR either, so `p.negate().and(s ->
s.length() == 1)` typed the second lambda's parameter `Object` and then refused
`length()` on it. The combinator's argument type was read off a bare NAME.
`functional_type_of` now walks the chain, since `negate`, `and`, `or` and
`Predicate.not` all answer a predicate over the same element.

**`Collection.toArray(IntFunction)` (Java 11)** — and, behind it,
**`toArray(T[])`, the idiom since 1.2**, which was not supported at all. The
model array carries the RUNTIME element type: it is filled and returned when it
is long enough (with one null written after the last element, which callers
rely on), and replaced by a fresh array of its class when it is not. The parser
already models `String[]::new` as the lambda `n -> new String[n]`, so the
generator overload needs no new machinery — the JDK defines it as
`toArray(generator.apply(0))`, and applying the generator at compile time is
the whole implementation.

A stream has ONLY the generator overload, so its call is renamed to an internal
`__toArrayTyped`: spelling it `toArray` in the stream method table would have
accepted `stream.toArray(new String[0])`, which javac rejects, and this engine
holds accepts-invalid at zero.

The `ArrayStoreException` a bad element raises has two different messages in a
JDK, and both are observable. An array-backed collection (`ArrayList`,
`Vector`/`Stack`, `Arrays.asList`, `ArrayDeque`, a `subList`) bulk-copies, so
what a program catches is `System.arraycopy`'s complaint naming the internal
`Object[]` and the destination component; an element-wise one (`LinkedList`,
the sets, a map view) names the offending element's own class. An
unmodifiable wrapper delegates, so which message it gives depends on what it
wraps.

**`Collectors.toUnmodifiableList` / `toUnmodifiableSet` / `toUnmodifiableMap`**
were found while checking that claim, and the first two were the worst kind of
defect: they ALIASED the plain collectors outright. A program that asked for an
immutable result got a mutable one that printed identically, so a defensive
`collect(toUnmodifiableList())` protected nothing — the same shape as the
round-8 finding that `Collections.unmodifiable*` views mutated through their
cursors. `toUnmodifiableMap` was refused entirely.

They differ from the plain collectors in the FINISH, not the gathering, so
`CollectorKind::Unmodifiable` wraps the collector it defers to and hands the
result to the same factory `List.of` uses — which is where both the refusal to
mutate and the rejection of a null element already live. (A null element IS an
NPE here where `toList` keeps it, which the JDK does for the same reason: it
finishes through `List.of`.) An unmodifiable map inherits the salted iteration
order of `Map.of`, so a program that prints one does not agree with itself
between two runs of a real JDK either.

The one gap left open, a refusal: `IntFunction` as a named variable type
(`IntFunction<String[]> gen = String[]::new; list.toArray(gen)`).

### `java.util.function` as a set of real types

The last unit left `IntFunction` as a named variable type open. Probing the
whole package instead of the one case found that it was not one gap but a
family of them: of the 43 interfaces `java.util.function` declares in Java 11,
**29 were nameable and 14 were not**, and of those that were, the PRIMITIVE
specializations carried no default combinators at all.

That second half had been recorded as a deliberate strictness ("each would need
its own helper class per interface, for combinators that are rare on the
primitive forms"). The reason had expired: the helper classes are mechanical,
and without them `IntPredicate`, `LongPredicate`, `DoublePredicate`, the three
primitive `UnaryOperator`s, the three primitive `Consumer`s, `BiPredicate` and
`BiConsumer` had no `negate`/`and`/`or`/`andThen`/`compose` between them. They
all have them now, each with its own named helper class (an anonymous class in
a bundled source shares the `Anon$N` counter with the program's own and
collides). The pinned strictness test is what reported the expiry — it fails
when caturra stops being stricter, which is exactly what it is for.

Typing the ARGUMENT of those combinators needed one more thing: the primitive
specializations take no type arguments, so a receiver read as a `TypeRef::Named`
never reached the parameterized-receiver rule and an inline lambda argument was
refused for having no functional target. They are the simplest case of all —
`IntPredicate.and` takes the very interface it is called on.

Also added: `BinaryOperator.minBy`/`maxBy` (ties go to the LEFT argument, as a
JDK's do), and `identity()` on the primitive unary operators. `identity()` used
STRAIGHT as a receiver — `IntUnaryOperator.identity().applyAsInt(7)` — has no
variable to read a target from, so the desugaring now names the interface from
the call's own owner; where javac has to infer (`Function.identity()`), it
infers `Object`, and so does this.

**The same fact in three files.** A `java.util.function` name has to be known in
three places: `imports.rs` (that the name exists), `functional_erased` (its
erased interface) and the lambda pass (its SAM shape). A name present in the
first and missing from the others is a type a program may WRITE in a
declaration and then cannot use — "java.util.function.ObjIntConsumer is not
supported by caturra", which reads as a deliberate gap rather than an oversight.
A unit test now walks the first list against the other two.

Two structural gaps fell out of the same probe:

- **A cast could not take a LAMBDA.** `(Runnable) () -> …` is the standard way
  to give a lambda a target type where the position implies none, and it read as
  "expected an expression": a lambda is parsed at assignment level, which the
  cast's operand parse (`unary()`) cannot reach.
- **A raw array would not fill a parameterized one.** `Supplier<String>[] a =
  new Supplier[2]` is the ONLY way to build such an array (generic array
  creation is illegal), so refusing the unchecked conversion refused the whole
  idea of an array of a parameterized type — collections included
  (`List<String>[] rows = new List[2]`). caturra already had the rule for two
  parameterized element types and simply not for the raw source.

~~One gap left open, a refusal with its own message: a functional interface
parameterized on a METHOD's own type variable~~ — **closed later**, see "A
generic method's lambda argument". ~~What remains of it is narrower: a return
variable pinned only by what the lambda BODY gives back.~~ — closed too, see
"A type variable the lambda's body pins".

### What a library object says it IS

`getClass()` on a library object answered `java.lang.Object` for every
collection VIEW (`keySet`, `values`, `entrySet`), every map ENTRY, every
immutable factory result (`List.of`, `Set.of`, `Map.of`, `copyOf`), every
`Collections` wrapper (`emptyList`, `singletonList`, `unmodifiableList` and
friends), every comparator built by a factory, and every collector. An
`Arrays.asList` said `java.util.ArrayList`, which is a different lie: those two
share a name and nothing else, and which one a program holds decides whether
`add` throws.

A program reads this three ways, and only the first is obvious:

1. `getClass().getName()` / `getSimpleName()`, and the class-identity idiom
   (`a.getClass() == b.getClass()`).
2. The `ArrayStoreException` a bad array store raises, which NAMES the value's
   class.
3. The `ClassCastException` a failed cast raises, which names it too — and
   whose module parenthetical depends on the name (`java.*` is "module
   java.base of loader 'bootstrap'").

The last two had their own namer, which knew about instances, strings and
arrays and nothing else — so casting any library object read "class <object>
cannot be cast to …". Both now go through the same function `getClass()` uses,
which is the half of the fix that keeps them from drifting apart again.

The names are JDK 11's own, and several are not constant for a given method:

- `List.of` is `ImmutableCollections$List12` for one or two elements and
  `$ListN` otherwise (an EMPTY one is `ListN`); `Set.of` likewise; a map has
  its own `Map1` only for a single pair.
- A `LinkedHashMap`'s views are `LinkedKeySet`/`LinkedValues`/`LinkedEntrySet`
  where a `HashMap`'s are `KeySet`/`Values`/`EntrySet`, and its entry is
  `Entry` where a `HashMap`'s is `Node`.
- `unmodifiableList` picks `UnmodifiableRandomAccessList` or
  `UnmodifiableList` by whether what it wraps is `RandomAccess`.

Since one `UnmodifiableList` wrapper stands for `emptyList`, `singletonList`,
`List.of` and `unmodifiableList` alike, which class each is gets recorded per
view when it is built — beside the index style, which is there for the same
reason.

Three left as recorded divergences rather than guesses:

- **A stream.** A JDK names a pipeline after its LAST operation and its element
  family (`ReferencePipeline$3` for a mapped object stream, `IntPipeline$9` for
  a filtered int one). caturra's single `Stream` object does not record which
  family it is, and inventing one would replace a known wrong answer with a new
  one.
- **A comparator built from a lambda.** A JDK names it after the lambda's
  ADDRESS (`Comparator$$Lambda$3/0x0000000840066040`), which differs between
  runs of the JDK itself. `naturalOrder`/`reverseOrder` have real classes and
  are modelled.
- **`new Random()`** reports `Random` rather than `java.util.Random`, because a
  bundled library class is compiled as an ordinary class under its simple name
  and the VM cannot tell it from a user class of that name — and a user class
  called `Random` reports `Random` correctly today. Renaming by name alone
  would break the case that works to fix the one that does not.

### Deciding equality once

A 484-pair cross-product of every collection kind against every other — each
pair asked for `a.equals(b)`, `b.equals(a)` and whether their hashes agree —
found 31 divergent pairs, and behind them two implementations of the same rule.

**Collection equality was written twice.** `java_equals`, which compares a
collection held AS AN ELEMENT, had it right: a List only equals a List, a Set
only a Set, a Map only a Map. The path a program's own `a.equals(b)` reached
decided for itself, and compared anything the element vector could hold. So
`aList.equals(anArrayDeque)` was true — an `ArrayDeque` shares this engine's
element vector with the lists, but it is not a `List` and overrides NEITHER
`equals` nor `hashCode`, so both are Object's, by identity:
`new ArrayDeque<>(a).equals(new ArrayDeque<>(a))` is false in Java. The direct
path now calls the same helpers, and a deque answers identity — which the
existing `is_list_like` predicate already said it should, in a doc comment
naming this exact case.

**`hashCode()` and the hash the engine BUCKETS BY disagreed.** A map view, a
map entry and an unmodifiable map all answered a structural `hashCode()` when
asked directly, but the internal hash did not know those kinds, so it fell back
to identity. A program could print `a.hashCode() == b.hashCode()` and see
`true`, then put `a` in a `HashSet` and watch `contains(b)` say `false`. The
public method and the engine now compute through the same functions.

**A sorted collection casts its probe to `Comparable`.** caturra compared two
references it could not order and answered EQUAL, so
`aTreeSet.contains(anEntry)` was true. Now a value with no natural ordering
raises the cast failure a JDK raises — and `AbstractSet.equals` SWALLOWS that
exception and answers false, which is why `aTreeSet.equals(anEntrySet)` is
false rather than fatal.

Three separate places built a `ClassCastException` message by hand, and each
was subtly wrong: one named the class in INTERNAL form ("class
java/lang/Object"), one left the module parenthetical off entirely, and the
module rule read a primitive array's leading `[` as a package name and put `[I`
in the application module. All three go through the shared builder now.

Two more found alongside:

- **`TreeSet.equals` compares with the SET'S COMPARATOR**, not with `equals` —
  it reaches `containsAll`, which on a sorted set uses the ordering. A
  case-insensitive `TreeSet` holding "Apple" does equal a `HashSet` holding
  "apple". Routing it through the plain set rule broke that, and the pinned
  test for it is what said so.
- **A bare diamond could not be a RECEIVER.** `new ArrayList<>().isEmpty()` was
  "cannot call methods on null": a diamond types as `null` here so it assigns
  to a collection of any element, and javac infers `Object` where there is
  nothing to infer from. Re-typing it as if `<Object>` had been written keeps
  the CONCRETE face, so a `new Stack<>()` still has `push`.

### Cross-producting the collections again

Three more relations run over the whole collection zoo the way equality was:
the BULK operations (`addAll`/`removeAll`/`retainAll`/`containsAll` between
every pair of kinds, 288 calls), the `Collections` ALGORITHMS (sort, reverse,
swap, fill, rotate, replaceAll, seeded shuffle, min/max, frequency,
indexOfSubList, disjoint, binarySearch, nCopies over four list kinds, 88 calls)
and the CURSOR operations (remove-first, remove-twice, remove-before-next,
next-past-end and a plain walk over fifteen kinds, 75 calls).

The first two came back **clean** — 288/288 and 88/88, the latter including
`Collections.shuffle(list, new Random(42))`, which needs both the JDK's exact
`Random` sequence and its exact shuffle. Two defects came out of the third and
out of trying to run the first at all.

**A closed scope did not release its locals.** The 288-call probe would not
compile: "too many local variables in one method". javac frees a block's slots
when the block ends, so sibling blocks share them; caturra allocated one slot
per declaration and ran out of the 255 a narrow `istore` can name. Every scope
push/pop now goes through one pair of helpers that saves and restores the
allocator, and `max_locals` became the PEAK rather than the value left at the
end — the two have to change together, or a method claims a frame smaller than
it used. A method with 255 *simultaneously live* locals is still refused, which
needs the `wide` prefix the VM does not decode; that is pathological where the
scope case is routine.

**A priority queue's cursor removed nothing.** The engine's element-vector
removal knows every collection except this one — a heap needs `removeAt`, which
repairs it and may run a user `compare`, so it cannot live in the heap-only
cursor code. The machinery was already there: `pq_remove_at`, whose own doc
says it returns the moved element "exactly the JDK's contract, which its
iterator uses to know what it will miss". The iterator had never used it.

That contract is the subtle half. `removeAt` fills the hole from the END, and
that element can sift UP past the cursor, into territory already walked. A JDK
keeps it aside (`forgetMeNot`) and yields it once the array is spent, so the
walk still sees every element exactly once; it steps the cursor back only in
the other case. Doing one of the two unconditionally both revisited an element
and skipped another — visible in `seen=1,10,2,11,10,12` where a JDK gives
`1,10,2,11,12,3`. 271 randomized removals now agree, including a full drain.

### When a collection refuses, and in what order

A 208-call cross-product — sixteen collection kinds against thirteen illegal or
edge-case operations, printing the exception CLASS and MESSAGE — found that
caturra decided "is this immutable?" too early, and then, once ordered the
other way, too early in the other direction. The JDK's answer is per WRAPPER
CLASS, and caturra models four of them with one object:

- `Collections.unmodifiable*` and the `List.of` family OVERRIDE every mutator
  and refuse at once, so `unmodifiableList(l).removeAll(null)` is an
  `UnsupportedOperationException`;
- `Arrays.asList`, `singletonList` and `emptyList` INHERIT
  `AbstractCollection`'s, which null-check first (so the same call is a
  `NullPointerException`), SCAN before removing (so removing what is absent
  answers false rather than throwing), and remove nothing from an empty
  collection (so `emptyList().clear()` is a no-op and
  `emptyList().removeIf(p)` is false);
- a `singleton*` wrapper overrides `removeIf` to refuse where its `empty*`
  neighbour keeps the null-checking default — one JDK class apart, and a
  program sees which.

The class recorded when each view is built (added for `getClass`, two units
ago) is what tells them apart. The `UnsupportedOperationException` MESSAGE
follows the CURSOR: `Iterator`'s default method carries "remove", a cursor that
overrides it throws the message-less form — three of the fifty-two refusals in
the matrix carry a message, and those are the three.

Two more found alongside:

- **Every collection NPEs on a null bulk argument**, and this was written per
  kind — so a list's `forEach(null)` threw while a priority queue's ended the
  run with "unknown native member: PriorityQueue.forEach", because a null
  matched no arm at all. One guard at the dispatch, skipped for the wrappers
  that refuse without looking.
- **`List.of`/`Set.of`/`Map.of` reject a null PROBE**: `contains(null)` is a
  `NullPointerException` there, not false.

Along the way the probe would not COMPILE, which found two more:

- **`removeIf(null)` was refused**, where javac compiles it and throws at the
  call. Six functional parameters each carried their own copy of the same
  "is it an instance of the bundled interface?" test, and the copies had
  drifted: `null` was a `Comparator` and a `Collector` and not a `Predicate`.
  They are one arm now. `null` satisfies an ARRAY parameter too
  (`toArray((String[]) null)`), and `Collection` had no `toArray` at all.
- **A collection CONSTRUCTED in place could not receive a lambda.**
  `new ArrayList<>(source).removeIf(x -> …)` had no element type, because the
  guard that recognises a collection tested "not a user class" — and the
  name-disambiguation set it tested against deliberately holds `ArrayList`, so
  it excluded exactly the most common collection there is. A lambda's own
  PARAMETERS are now in scope for its body too, which is what
  `grid.forEach(row -> row.forEach(v -> …))` needs.

### Where a collection's element comes from

The previous unit left three lambda-target gaps recorded. Cross-producting them
showed they were one gap with many faces: **nineteen receiver shapes against the
six consumers that need an element type, and 49 of the 114 cells diverged.**

The structure was stark. The two consumers that do NOT pass a lambda —
`iterator()` and a for-each loop — worked for every shape. The four that do
(`forEach`, `removeIf`, `stream().map`, `stream().filter`) failed for eleven of
the nineteen, always with the same message: "a lambda or method reference is
only allowed where a functional-interface type is expected".

The lambda pass read a NAME, a `this` field, and three factories
(`List.of`/`Set.of`/`Arrays.asList`). Everything else had no element type at
all: a CAST, a TERNARY, an ARRAY element, a call to a method whose declared
return says exactly what it gives back, and every `Collections` factory —
`singletonList`, `singleton`, `nCopies`, `unmodifiableList`,
`unmodifiableSet` — plus `List.copyOf`. Each now answers, from wherever its
element is written down: a cast from its own type, a ternary from either
branch, an array element from the array's declared type, a method call from its
return type (which needed the pass to carry one at all), a wrapper or a copy
from its SOURCE, and `singletonList(x)`/`nCopies(n, x)` from the argument.

49 divergent cells became 9, and none of the nine is a defect:

- **Three are the JDK disagreeing with itself.** `Set.of`'s iteration order is
  salted per JVM run — `Set.of("a","b","c","d","e")` printed three different
  orders across five runs of the same program on the same JDK. Nothing to
  match, and nothing pinned.
- **Six are the discarded type WITNESS.** `Collections.<String>emptyList()`
  parses its witness and throws it away, so the call types as the
  context-adopting `null` and a lambda over it has no element. The witness has
  nowhere to live — `Expr::Call` carries no type arguments — which is the same
  recorded gap that makes a DISAGREEING witness ignored. Assigning the factory
  to a declared variable first works, and is what the shape is normally
  written as.

Noted while probing, not chased: `(List<String>) Collections.emptyList()`
compiles here and is an "inconvertible types" error in javac, because a cast is
a context and caturra's `emptyList()` adopts one. Looser than javac, in the
narrow place where the alternative is refusing a program whose meaning is
unambiguous.

### The map surface

Maps had never been cross-producted: seven map kinds against thirty-eight
operations, including every null-key and null-value edge and the whole
`compute` family — 266 calls, **15 divergent**. Four causes, three of them
families already met on the list and set sides:

- **`merge(k, null, f)`** answered the existing value where a JDK throws. Its
  `Objects.requireNonNull(value)` runs before anything else: a null value is
  not "leave it alone", it is a programming error the map refuses to guess
  about.
- **`putAll(null)`** ended the run with "unknown native member:
  HashMap.putAll" — a null argument matched no arm at all. The shared
  null-argument guard added for collections now covers a map's `putAll`,
  `forEach` and `replaceAll` too.
- **`Map.of(...).getOrDefault(null, d)`** answered the default where a JDK
  rejects the null key. The null-probe rule was there; it matched the whole
  argument list, and this probe has a second argument.
- **The inherit-versus-override split**, again. `singletonMap` and `emptyMap`
  inherit `AbstractMap`'s mutators, so `remove(missing)` scans and answers null
  rather than refusing, and `clear`/`replaceAll` over an empty map do nothing.
  `unmodifiableMap` and `Map.of` override everything and refuse without
  looking.

One JDK detail is worth writing down because it cost a VerifyError to find:
`Collections.EmptyMap` inherits the ONE-argument `remove(key)` and OVERRIDES
the two-argument `remove(key, value)` (along with `replace` and the `compute`
family) to refuse whatever the map holds. Treating the two forms alike answered
`null` for a call whose return is a BOOLEAN — an int slot holding a reference,
which the verifier catches at the next use rather than at the call.

15 became 1, and the one is `Map.of`'s salted iteration order.

### A user-defined functional interface

`java.util.function` was probed as a package; the interfaces a PROGRAM declares
had not been. Twenty-five shapes — every position a lambda can occupy, plus
method references, default methods, captures and nesting — found twenty already
working and two families that were not.

**An interface can be functional by INHERITANCE.** `interface Sub extends Op {}`
declares no abstract method of its own and is a perfectly good target for
`Sub f = x -> x + 1`; caturra collected single abstract methods per interface
and never looked at supertypes, so every such lambda was refused. The
collection now closes over `extends` to a fixed point, so a chain of extending
interfaces all resolve — and an interface that would inherit two DIFFERENT
abstract methods is left out, because it is not functional.

**A lambda's own parameters are in scope for its body here too.** The previous
unit put them in scope in the erased builder, which is the path a
`java.util.function` target takes; a USER interface takes the other one, and
had the same hole. `Box<List<String>> f = l -> l.stream().map(v -> …)` was
refused for the INNER lambda having no functional-interface position. The same
rule, written in both places — which is how it was missed the first time.

One shape is still refused, and it has now come up three times (in the
`java.util.function` unit, while writing a test harness for the map surface,
and here): **a functional interface parameterized on a METHOD's own type
variable** — `static <T> int pick(T v, Box<T> f)`. Typing the lambda needs `T`
inferred from the OTHER argument at the call, which is machinery caturra has
for inferring a generic RETURN type and not for a parameter. It has its own
honest message, and three appearances is the argument for building it next.

### A generic method's lambda argument

`static <T> int pick(T v, Box<T> f)` had its own refusal message — "a
functional interface parameterized on a method's own type variable is not
supported by caturra" — and had come up in three separate units before this
one. Probing the shape directly found it was not a corner: **14 of 15 forms
failed**, including the library interfaces (`Consumer<T>`, `Predicate<T>`,
`Function<T, Integer>`, `UnaryOperator<T>`), a variable pinned by a collection
argument's ELEMENT, a bounded variable, a varargs one, a wildcard bound, a
method reference in the lambda's place, and a variable belonging to the
enclosing CLASS rather than the method.

The cause was an ordering one. A lambda argument is target-typed by its
declared parameter, and by the time the lambda pass runs, erasure has already
replaced `Box<T>` with a wildcard that no longer says WHICH variable it held —
so there was nothing left to type the lambda's parameter with. The information
was destroyed before the pass that needed it.

caturra already computes exactly the right thing for the RETURN type: an
`infer_return` plan recording where a returned type variable is mentioned among
the parameters, so a call can recover the argument. That plan is now computed
for EVERY type variable, not just the returned one, and the parameter types as
WRITTEN are kept beside the erased ones. At a call, each variable is pinned
from the arguments — directly, from a collection argument's element, or from a
varargs argument, which is a `T` itself rather than a container of them — and
a variable the declaring class owns is pinned from the receiver's own type
arguments instead.

Substitution is all-or-nothing. A parameter mentioning a variable this call
could not pin falls back to the erased signature, because a half-substituted
target reaches codegen as a name nothing declares: the first version of this
answered "unknown type 'R'", which is a worse reply than the honest erasure.

One form was still out, and it is the one that needed a different mechanism:
`<T, R> R conv(T v, Function<T, R> f)`, where `R` is pinned only by what the
lambda BODY returns. **Closed later**, see "A type variable the lambda's body
pins".*

### A constant conditional keeps its type

Probing BOXING contexts — the `Integer` cache boundaries per wrapper, identity
against a primitive, unboxing a null in every context that does it, `switch` on
a wrapper, overload resolution between a primitive and its box, round trips
through an array, a list and a map key — found **43 of 45 already right**. The
two that were not turned out not to be about boxing at all.

`true ? 1 : 2.0` printed `1` where Java prints `1.0`, and `false ? 'a' : 98`
printed `98` where Java prints `b`. A conditional whose condition is a CONSTANT
folds to the taken branch, and the fold returned that branch's value —
discarding the conditional's own type, which JLS §15.25 makes the promotion of
BOTH branches. The identical expression with a variable condition was already
right, so the two forms disagreed with each other; a 441-pair cross-product of
operand types with a variable condition came back clean, and the same 1152
pairs with a constant condition did not.

The `char` rule is the delicate one, and it is why the fold cannot work from
values alone. A `char` beside an int CONSTANT representable in a char stays a
char (`flag ? 'a' : 98` is `b`); beside one that does not fit, both promote to
int; and beside a `(byte)` cast — an operand of that TYPE rather than a
constant — both promote to int as well, so `true ? 'a' : (byte) 3` is `97`
where `true ? 'a' : 3` is `a`. caturra's constant values have no byte or short
of their own, so that last distinction is read off the EXPRESSION rather than
the value.

### One constant folder

The previous unit found a folded constant answering with a different type from
the same expression emitted. That generalises to a question worth asking
directly: **wherever caturra folds a constant, does the folded answer match the
unfolded one?** Every operator over every operand-type pair, each written twice
— once foldable and once behind variables — is 954 cells, and they agreed with
the JDK and with each other throughout. The arithmetic core is sound.

The disagreement was elsewhere. **Reachability analysis had its own miniature
constant folder**, which knew literals and boolean constant variables but not
`FOUR < TWO` over two constant ints. JLS §14.21 makes a loop with a
constant-false condition an error — its body is unreachable — and javac rejects
such a program; caturra accepted it, because its folder could not see through
the comparison. An accepts-invalid, which this engine holds at zero.

Reachability now asks the same folder codegen uses, so it recognises the full
constant expressions of §15.28: `while (FALSE)`, `while (1 > 2)`,
`while (FOUR < TWO)`, `while (FOUR - 4 != 0)` and `while (!(FOUR > TWO))` are
all rejected, `while (FOUR > TWO)` runs, and `if (FOUR < TWO)` stays legal —
the `if` carve-out is deliberate in the JLS, so that a `DEBUG` block compiles.
The second folder is gone rather than extended; the module that held it now
says so.

Checked alongside and already correct: a folded concatenation is INTERNED
(`"a" + "b" == "ab"` is true, `a + "b" == "ab"` is false, `.intern()` makes it
true again), a constant expression stands in a `case` label and an array size,
constant overflow wraps as the runtime does, and definite assignment through a
constant condition matches.

### Auditing the divergence lists again

The previous unit found a narrowness documented as deliberate that was in fact
an accepts-invalid. That is a reason to distrust the documentation generally,
so this unit ran every bullet of the two divergence lists rather than reading
them — the same audit the lists themselves record having had on 2026-08-14.

**Every bullet still held.** All ten strictnesses and the one recorded
permissiveness are exactly as described.

The lists had stopped being EXHAUSTIVE instead, which is the other half of what
they claim. Four strictnesses and one permissiveness had been recorded in the
PROSE of later entries and never added to either list or pinned by a test:
`IntFunction` as a named variable for `toArray`, a return variable pinned only
by a lambda's body, a discarded type WITNESS, `Arrays.stream(a, from, to)`, and
— the one that matters most, because the looser list said "one known case" —
`(List<String>) Collections.emptyList()`, which caturra accepts and javac calls
inconvertible. All five are now enumerated and pinned, so the count is honest
again.

One stale claim was corrected in the same pass: "a functional interface
parameterized on a METHOD's own type variable" was written down as an open gap
and closed two entries later.

Prose is where a divergence is explained; the list is where it is COUNTED. An
entry that only explains leaves the count wrong, and a list that says it is
exhaustive has to earn it every time it grows.

### A greedy repeat backs off by code point

Two API surfaces were cross-producted as matrices and came back almost clean:
`java.util.Arrays` — every method over all nine element types, 180 cells, no
divergence — and `String`, every method against ten subjects including an
empty one, whitespace, mixed case, accented characters and a SURROGATE PAIR:
479 cells, one divergence.

`"a😀b".matches(".*a.*")` answered false. The `.` itself was already
code-point aware — `"a😀b".matches("...")` was right, and so was
`replaceAll(".", "#")` — and only the BACKTRACKING was not.

A greedy repeat over a simple body runs as a loop rather than one stack frame
per repetition, which is what keeps `a*b` over a thousand characters off the
stack. The loop counted how many repetitions it had taken and assumed each
consumed one code UNIT, so backing off decremented the position by one and
landed BETWEEN the surrogates. From there the positions it tried were not the
ones the repetitions had actually reached, and it ran out of repetitions before
reaching the start of the string. Each repetition's END is now recorded, so
backing off returns to a position the body really stopped at, whatever its
width.

The shapes affected were exactly those that must give a repetition back across
the pair — `.*a.*`, `.*a.+`, `(.*)a(.*)`, `replaceAll(".*a", …)`. A lazy repeat
was unaffected, because it never backs off; so was a greedy one whose backtrack
lands after the pair rather than inside it.

### Counting code units, not characters

The previous unit ended with a rule worth applying rather than just recording:
put a SURROGATE PAIR in every text matrix. Doing that across the rest of the
text surface — `StringBuilder`'s index-taking methods, `Character`'s
code-point helpers, the special-case foldings, ordering, `String.format` —
found 34 of 35 already right, and one that was not.

**`String.format("%5s", "😀")` padded one space too far.** A JDK's `Formatter`
measures with `CharSequence.length()`, so a width counts UTF-16 code UNITS and
a supplementary code point is two of them; caturra counted code points. The
same rule governs precision. A 306-cell matrix of every conversion against
every argument kind confirms nothing else moved.

Already right, and pinned so they stay that way: `StringBuilder.reverse` keeps
a surrogate pair together (and reverses back to the original), the builder's
`insert`/`delete`/`replace`/`subSequence` work in code units,
`Character.charCount`/`toChars`/`isHighSurrogate` and
`String.offsetByCodePoints` agree, and the foldings that change LENGTH —
ß→SS, ﬁ→FI, final sigma — all match.

**One recorded runtime divergence.** A string holding an UNPAIRED surrogate
prints as U+FFFD here where a JDK writes `?`: its UTF-8 encoder cannot encode
one and substitutes, and caturra decodes lossily at the boundary between its
UTF-16 storage and Rust text. Distinguishing the two downstream is impossible —
a real U+FFFD in the program is by then the same character — so the fix belongs
at the decode, which 35 call sites share and which every string operation goes
through. Left alone deliberately: the input is degenerate (an unpaired
surrogate can only be built by splicing one), and moving that decode to satisfy
it would put every string operation at risk. `%.1s` of an emoji is the same
divergence from the other end: the JDK truncates to one code unit and prints
the half as `?`.

### Which implementation runs

Virtual dispatch had never been cross-producted: every declaration site (class,
abstract class, interface default, interface static, private) against every
static type it can be called through. **37 cells, all clean** — private methods
do not dispatch, fields hide by STATIC type, `super.m()` and `Iface.super.m()`
reach the right one, covariant returns work through the erased signature, and a
call site fed alternating receiver types answers correctly each time, which is
what an inline cache is easiest to get wrong. The corners were where it was not.

**A DIAMOND is resolved by whichever declaration is most specific**, and caturra
asked only whether the CLASS resolved it. `interface C extends A, B` that
declares the method resolves it for everything below, so
`class Impl implements C { }` is legal Java — and was refused, with the conflict
reported against a diamond that had already been closed one level up.

**RE-ABSTRACTING resolves it too, and then binds.** `interface C extends A { T
m(); }` deliberately discards `A`'s default and pushes the obligation onto
implementers. Two rules had to move in opposite directions for that to work: an
abstract declaration now COUNTS as resolving an inherited conflict, and a
default in an interface the re-abstracting one extends no longer counts as
implementing it. Fixing only the first made `class Impl implements C { }`
compile and silently run `A`'s discarded default — the mirror mistake, and the
matrix caught it on the next run.

Eleven diamond shapes now agree with javac, in both directions: resolved by a
default, by a re-abstraction, by the class itself, through separate parents,
through a longer chain, and by a superclass method — which wins over any
interface default — as well as the two that must still be REJECTED.

### count() may not run the pipeline

Exception selection and control flow were cross-producted: eleven throw kinds
against a nine-clause catch ladder, `finally` overriding a return, `finally`
with `continue`/`break`/labelled jumps, nested try, precise rethrow, cause
chains, a `finally` replacing the exception in flight, try-with-resources with
suppression from the body and from each close, and the return-value-before-
`finally` rule (`try { return x++; } finally { x = 99; }` is 1). **Thirty-nine
cells, all already correct.**

The one divergence was in the probe's control case: an exception thrown from a
`map` inside `count()`. **A JDK never threw it.** `Stream.count()` is specified
to skip execution of the pipeline when it can compute the count directly from
the source, so the side effects of a size-preserving operation — a `map`, a
`peek`, even a mapper that throws — do not happen at all. caturra ran them.

Which operations allow the skip is empirical, and worth writing down: `map`,
`peek`, `sorted`, `boxed` and the `mapToX` family do; `filter`, `limit`,
`skip`, `distinct`, `flatMap` and `takeWhile` do not, and neither does anything
after one of them.

Two properties survive the shortcut, and both took a pinned test to get right.
The stream stays LATE-BINDING: the size is the source's as it stands when
`count()` runs, not as it stood when the stream was opened. And it does NOT
fail fast — appending to the source and then counting answers the new size
rather than throwing, because the JDK's modCount check lives in the
spliterator's `forEachRemaining`, which a skipped pipeline never reaches. The
first attempt kept the fail-fast check and broke the late-binding test that had
already pinned exactly this.

### How often a library call runs your callback

The previous unit found a JDK optimization that is visible only through side
effects. That is a dimension of its own: for every library method that takes a
lambda, HOW MANY TIMES and in WHAT ORDER is it invoked? Thirty-nine calls —
collection, map, `Optional` and stream — each logging every invocation, compared
against a JDK's sequence rather than only its result.

Most already agreed, and the agreements are worth naming because each is a rule
someone could have got wrong: `removeIf` over an empty collection calls nothing,
`sort` of fewer than two elements calls nothing, `computeIfAbsent` on a present
key calls nothing, `merge` on an absent key calls nothing, `orElse` evaluates
its argument EAGERLY while `orElseGet` does not, and the short-circuiting stream
terminals stop exactly where a JDK stops.

**The comparator-taking `Collections` statics did not.** `max`, `min` and
`binarySearch` had no rule giving an INLINE lambda its target type — `sort` had
one and these three never got it. So the lambda reached overload resolution
untyped, the two-argument form did not apply, and the call silently resolved to
the NATURAL-ORDERING one: the comparator was never called, and a reversing one
gave the wrong answer. Assigning the result made it worse, because the unused
comparator stayed on the stack and the verifier reported malformed bytecode.
Two things were missing — the target-typing rule, and the trigger that pulls the
bundled `__Comparator` into a program that never names it.

**A map's key and value types now flow from every receiver shape** a list's
element already did: a method's declared return, a cast, a ternary, an array
element. `config().forEach((k, v) -> …)` was refused while the identical call on
a declared variable compiled. A test had pinned that refusal as intended
behaviour; javac accepts the program, so the pin is gone.

### Auditing the refusals

Three separate units this session found a TEST that had pinned a caturra
limitation as though it were the intended behaviour — the most recent being a
map reached through a method's return, which javac accepts. Three is a pattern
worth attacking directly: every test that asserts a program should NOT compile
is a claim about javac as much as about caturra, and it is only true while javac
agrees.

Twenty-seven such tests were found and their programs run through a live javac.
Most are honest — many say so in their names — and one strictness fell out that
had never been enumerated: **a class nested inside another inner class cannot
reach the enclosing instance, nor an enclosing method's local.**

Probing it pinned the boundary much more narrowly than the test that recorded it
("nested capture is not supported"). What fails is only the CAPTURE CHAIN. A
nested lambda reads static fields, static methods, constants and the outer
lambda's own parameter perfectly well, and nests three deep over any of them;
what it cannot do is reach a name that the first level would itself have had to
capture. All four nestings behave the same way — lambda in lambda, anonymous in
lambda, lambda in anonymous, anonymous in anonymous — which says the limit is
the chain and not the lambda desugaring.

It was a bullet in the strictness list with two pins, and the boundary was
pinned too, so a fix would show up as three failing tests rather than as a
silent widening — which is exactly how it went: the capture chain was made
transitive the same week (see **Nested capture**, below), and the three tests
came back to say so.

### Nested capture (2026-08-19)

A lambda or anonymous class nested inside another one now reaches the enclosing
instance and the enclosing method's locals, at any depth. The chain used to be
ONE level: `outer = () -> { inner = () -> field; … }` did not compile, and the
previous entry had just finished pinning that as a strictness.

The capture pass runs where a class is CREATED, and by then a nested lambda is
already a class of its own — so the outer class's body no longer contains the
inner one's names, and the outer captured nothing on its behalf. The fix has
three parts, and each is load-bearing on its own:

- **Pull the capture down.** A class's free names are collected over its own
  body AND over every class created inside it, transitively, so an outer lambda
  captures what an inner one needs. The creation relation is the inverse of the
  owner map, which the same walk produces, so phase 1 runs to a FIXED POINT: it
  learns who creates whom, then re-walks knowing it. A single extra pass would
  reach two levels and stop.
- **See it from below.** A synthesized class's captures are FIELDS, not locals,
  so when the body nested inside it is scanned they stand in for the enclosing
  method's locals — seeded into the scope for that walk. Without this the inner
  body's `local` looked free and got captured by nobody.
- **Walk out for the instance.** `__caturraOuter` is injected wherever a body
  names an enclosing instance member, and the WANT propagates upward to a fixed
  point: a class three levels down reaches its outer instance through every
  level in between, so each of them must hold one too.

The subtle part is `this`. A lambda has none of its own — JLS §15.27.2 says
`this` in a lambda is the enclosing instance — so the walk out skips lambda
levels. It must STOP at an anonymous class, which does have one: inside a lambda
written in an anonymous class, `this` is the anonymous object, and javac REJECTS
`this.field` there for exactly that reason. Getting this wrong is invisible in
every name-resolution probe and shows up only in identity (`got == this`) and in
what the compiler refuses; skipping the anonymous level made 46 of 47 probes
agree while quietly answering the wrong object.

Which member a name means is decided against the union of the members along the
chain, not the nearest owner — `field` inside a lambda inside an anonymous class
is the top-level class's, reached by one hop per level, and the existing
`__caturraOuter` chain walk in codegen resolves it once the fields exist.

Probing this found a second, unrelated defect in the same neighbourhood: an
anonymous class's SUPER-ARGUMENTS were typed by guessing at the argument
expression, which only worked for a literal or a plain local. `new Base(field){}`
— or any arithmetic, call, comparison or ternary — refused to compile at all.
The arguments are forwarded verbatim to `super(…)`, so the synthesized
constructor is now typed by the superclass constructor they reach whenever only
one has that arity; where the superclass is overloaded on arity, the argument
types still decide, so the guess was extended to the shapes that carry a type
(binary numeric promotion, string concatenation, comparisons, unary operators,
an agreeing ternary, and a bare field or call of the enclosing class).

Pinned by `a_nested_lambda_reaches_the_enclosing_instance_and_its_locals`,
`this_in_a_lambda_is_the_nearest_instance_with_one`,
`a_lambda_in_an_anonymous_class_cannot_reach_out_through_this` and
`an_anonymous_class_passes_any_expression_to_its_super_constructor`.

### Explicit type witnesses (2026-08-19)

`Collections.<String>emptyList()` (JLS §15.12.2.1) was parsed and DISCARDED —
`Expr::Call` carried no type arguments — so a call with nothing else to infer
from had no type at all, and a lambda written against its result was refused
for having no functional-interface position. The witness now rides on the call.

That alone fixed only half the shapes, because reading a call's return type
went through the ERASED signature: `<T> List<T> box(T v)` returns a list of a
wildcard sentinel, and a lambda over it was refused as **"a functional
interface parameterized on a method's own type variable"** — an item that had
been recorded as open across three audit rounds. The declared return is now
kept beside the declared parameters (both are erased at the same point, for the
same reason), and one binder pins a call's type variables from three sources in
priority order: an explicit witness first (JLS does not infer when one is
written), then the arguments, then the receiver's own type arguments for a
variable the declaring class owns. `box("ab").forEach(s -> …)` types `s`.

A witness is also a claim about the ARGUMENTS — `W.<String>id(5)` states that
`5` is a String, and javac refuses it. Nothing checked that, because without a
witness the same call INFERS `T` from the argument and cannot be wrong. The
check is deliberately narrow: a primitive boxes to exactly one wrapper
(JLS §5.1.7), so `<Long>id(5)` is provably wrong, and one concrete final
library type against another likewise — but a witness naming a USER class is
left alone, because this pass knows each class's members and not its ancestry,
and a wrong REJECTION would be worse than the missing check. That residue is
the third bullet in the permissiveness list, pinned rather than left implicit.

A generic method that answers an `Optional<T>` reads the same way (added
2026-08-20): `first(list).map(s -> s.length())` had no element, because every
shape the Optional reader knew wanted a RECEIVER and a bare call in the same
class has none — so the pinned return is asked for first now.

Pinned by `a_type_witness_types_the_call_it_is_written_on`,
`a_generic_methods_return_is_pinned_by_its_arguments`,
`a_witness_that_contradicts_its_argument`,
`a_witness_that_names_the_wrong_wrapper`,
`a_witness_wider_than_the_argument_is_accepted` (eleven shapes the check must
NOT refuse) and `a_witness_naming_the_wrong_user_class`.

### The Optional a stream answers with (2026-08-19)

Two defects behind one shape, found by probing what a stream terminal HANDS
BACK rather than what it computes.

**A reference Optional holds a reference.** `get()` is bytecode-typed to return
one, so a primitive stored raw came back off a terminal as an `Int` where the
verifier wanted an object: `list.stream().map(s -> s.length()).findFirst()
.get()` was a **VerifyError**, not an answer — and so were `orElse`, a mapped
`sorted` chain, and every other stream whose lambda answered a primitive.
`Optional.map` already boxed for exactly this reason, with a comment saying
why; every OTHER way a reference Optional is built did not. The rule now lives
in `alloc_optional`, where all of them pass. An `OptionalInt`/`OptionalLong`/
`OptionalDouble` is the primitive-typed one and keeps its value as it is.

**A primitive stream is not only an `IntStream`.** One builtin table serves all
three, and the Optional flavour was hardcoded, so `mapToDouble(…).max()`
printed `OptionalInt[3]` where the JDK prints `OptionalDouble[3.0]` — a wrong
answer, not a refusal. `BRet::OptionalElem` follows the element, `OptionalLong`
is now a type (with `getAsLong`, and a `toString` that says so), and the
descriptor rewrite that adapts a primitive stream's signature no longer edits
the letters INSIDE a class name — a blind `I`→`D` had been turning
`Ljava/util/OptionalInt;` into `Ljava/util/OptionalDnt;`, which is how the VM
lost track of which flavour it was building.

Two more, underneath: `mapToLong`/`mapToDouble` did not WIDEN what the lambda
answered, so a "DoubleStream" built from `x -> x` over `Integer` elements held
ints (`sum` looked right only because it accumulates in the wider type), and
`max`/`min`/`average` read only the `Int` elements, so a widened stream's max
came back EMPTY. The primitive fold `reduce(op)` — the form with no identity to
answer with, which needs an Optional — was missing from the primitive table
altogether.

Known gaps beside this, both refusals: `DoubleStream.of(…)`/`LongStream.of(…)`
as NAMED static factories (the streams themselves work, reached through
`mapToDouble`), and a lambda's RESULT type, which erases to `Object` — so
`map(s -> s.length()).findFirst().get()` prints correctly but cannot be
assigned to an `int` or added to one.

Pinned by `a_reference_optional_holds_a_reference`,
`a_primitive_streams_optional_follows_its_element` and
`a_primitive_stream_folds_without_an_identity`.

### The element a `map` produces (2026-08-19)

A stream's element type erased to `Object` at `map`, which is faithful to
erasure and useless downstream: a later `filter`, `map` or `collect` saw its
parameter as `Object`, and the result would not assign to an `int`, add to one,
or collect into a `List<Integer>`. Eleven of sixteen probed shapes were
refused, in programs javac compiles.

The type is read off the lambda's BODY, in a deliberate subset — a name, a
literal, an operator (with binary numeric promotion and string concatenation),
a call to one of the program's own methods, and the library methods whose
return is a scalar or a `String`. A body outside the subset answers nothing and
the element stays `Object`, exactly where it stood before.

Two passes need it, and the second cannot see the first's reasoning: by the
time an outer call asks, the lambda is already a synthesized CLASS (a receiver
is desugared before what it yields is typed), and codegen sees only the erased
`Object` that class's method returns. So the class carries the answer: the same
reading is done from the class itself — its body opens by unwrapping each
erased argument into the parameter's declared type and ends in the expression
whose type is the answer — and the result is recorded as a synthetic static
field, `__caturraProduces`, that nothing reads at run time. The element codegen
takes from it is the WRAPPER (`Stream<Integer>`, not a stream of `int`), since
the lambda's result is boxed to fit the erased `Function`.

This retired the compatibility page's `lambda-after-map` GAP, which is now a
supported feature recorded against a real JDK rather than a refusal.

Pinned by `a_mapped_stream_keeps_its_element_type` (sixteen shapes).

### The element a literal collection joins at (2026-08-19)

`Arrays.asList(new Circle(), new Square())` would not assign to the
`List<Shape>` javac gives it: the element join walked SUPERCLASSES only, so two
classes whose nearest common ancestor is an interface met at `Object`. The
identical pair in a TERNARY joined at `Shape` — the rule was implemented twice
and the copies disagreed, which is this session's recurring shape. The walk now
stops short of `Object` (which covers everything, and would end the search
before an interface is considered) and falls through to the same
`shared_interface` the ternary uses.

`Stream.of` had a THIRD reading: the first argument's type, which made the
second argument an incompatible one. It joins like the list factories now.

Two classes sharing SEVERAL interfaces is an intersection type (`Shape &
Drawable`), which caturra has no type for. It used to answer `Object`, which is
accepted nowhere; it now names the FIRST interface written, so
`class Circle implements Shape, Drawable` joins with `Square` at `Shape`. Code
that wanted `Drawable` is refused where javac accepts it — the safe direction,
and no worse than the `Object` that refused both. The interface walk is
breadth-first so "first" means what the program wrote.

Known residue: the LAMBDA pass has its own element reading for a
`Stream.of(...)` source and no class hierarchy to join with, so
`Stream.of(new Circle(), new Square()).map(s -> s.area())` still sees `Object`.

Pinned by `a_literal_collection_joins_at_a_shared_interface` (including seven
joins that already worked, which the new one must not disturb) and
`a_join_over_several_interfaces_names_the_first`.

### DoubleStream and LongStream have their own factories (2026-08-19)

Both were TYPES with no way to make one. The pipelines worked — reached through
`mapToDouble`/`mapToLong`, whose results are `DoubleStream` and `LongStream`
and whose terminals answer in the element's own width — but the NAMES resolved
nowhere as a static-call target, so `DoubleStream.of(…)`, `LongStream.of(…)`,
`LongStream.range(…)` and `empty()` were all "cannot find symbol": sixteen of
twenty probed shapes.

They are registered as static owners now, `LongStream` with the two range
factories the JDK gives it (a `DoubleStream` has none there either), and the
source emitter takes the element from the factory's own class rather than from
the arguments — a primitive pipeline's elements carry their width, and joining
`1.5, 2.5` would have boxed them.

The same call had a THIRD reading in `type_of`, which mirrors the emitter so
the two can be checked against each other: it knew only `Stream` and
`IntStream`, so `Arrays.toString(DoubleStream.of(d).toArray())` — a source used
straight as an argument — had no type, while the identical stream held in a
variable first passed. It also read `Stream.of`'s element from the FIRST
argument where the emitter joins them; both now join.

The primitive OPTIONALS had the same shape of gap, closed the same way
(2026-08-20): a stream's terminal answers an `OptionalInt`, and
`OptionalInt.of(5)` resolved nowhere. They have `of` and `empty` now, over a
value whose width the class name carries — found by a sweep of 46 expressions
whose printed TEXT is subtle (grouping and padding in `format`, the radix and
sign conversions, `Double.toString`'s thresholds, rounding halfway cases,
`floorDiv` against `/`, the case-folding pairs, `deepToString` of a cycle), of
which this and one documented refusal were the only two that differed.

Pinned by `the_primitive_streams_have_their_own_factories` and
`the_primitive_optionals_have_their_own_factories`.

### An entry set is a collection like any other (2026-08-19)

`new ArrayList<>(map.entrySet())` — how a map's entries get sorted — was
refused with a message about needing a Collection, and so were the `HashSet`,
`LinkedList` and `ArrayDeque` forms. A map's entry set IS a
`Set<Map.Entry<K, V>>`; what it was not was a shape any of the copy
constructors recognized.

Every one of them had listed the accepted shapes for itself — SIX copies of one
fact (`LinkedList`, `ArrayDeque`, `HashSet`, `TreeSet`, `PriorityQueue`, and the
`copy_element_of` that the `ArrayList` emitter and `type_of` share). None
listed the entry set, and each would have needed its own arm. They all read one
`collection_element_type` now, which is also what a stream source reads, so a
seventh shape can only ever be added once.

Pinned by `an_entry_set_is_a_collection_like_any_other`, which also runs the
sources that always worked, since collapsing six readings into one is exactly
the change that could lose one of them.

### A stream made one element at a time (2026-08-19)

`Stream.iterate(seed, next)` and `Stream.generate(supplier)` are INFINITE: the
elements do not exist until a terminal pulls them. A source modelled as a
vector of values cannot express that, so both factories were missing outright —
and the lambda passed to one had no functional-interface position, which is how
the gap announced itself.

A stream's source is now a `StreamSource`: a vector, or the rule for making the
next element. The driver pulls from it one element at a time either way, which
the lazy pipeline already did, so the short-circuiting operations end an
infinite traversal exactly as they end a finite one. Nothing bounds an
unbounded terminal, as nothing bounds a JDK's — it runs until the instruction
budget ends the program, which is the nearest thing this engine has to never
returning. Two readings had to learn the difference: `count()`'s shortcut
(which answers from the source's size without running the pipeline, and a
generated source HAS no size) and `type_of`'s mirror of the source factories.

Probing it found a compiler PANIC — a method reference handed straight to the
erased-lambda builder, which every other library callback converts first — and,
underneath, a parser workaround that had outlived its reason. A lambda body
that is a statement-EXPRESSION (`x -> count++`, `n -> total[0] += n`) fits a
void descriptor AND a value-returning one (JLS §15.27.2), so only the target
can say which it is. The parser decided for it, lowering the body to a
statement: right for a `Consumer`, and for a `Supplier` it threw the value away
and left "missing return statement". The lowering was there because an
assignment EXPRESSION did not compile yet; that was fixed in round 5, and this
outlived it.

Pinned by `a_stream_can_be_made_one_element_at_a_time` and
`a_statement_expression_lambda_answers_when_asked` (both directions — the void
side is what the lowering existed to serve).

### subList as a live view (2026-08-19)

The last of the "list views are not supported by caturra" refusals, and the
most used of them: `list.subList(from, to)` is a window onto a range of the
list itself. Reads see what the backing list holds NOW, writes go through in
both directions, `clear()` on the view removes the range, and a structural
change made AROUND the view invalidates it — that last part is what separates a
view from a copy, and copying would have been the silent wrong answer.

A view is a `(backing, from, len)` triple plus the backing length it last
agreed with, which is how a change around it is noticed (caturra models
modCount as the length, as the fail-fast cursors do). Reads need no new code:
`list_items` knows the range, and every reader — `contains`, `indexOf`,
`equals`, `hashCode`, the for-each cursor, `stream()`, the renderer — goes
through it. What could NOT be inherited is every method that WRITES, since each
must land in the backing list at a shifted index and resize the view, plus
`size`/`isEmpty`, which answer from a size only the view knows. A `subList` OF
a subList composes the offsets onto the same backing list, so a write still
lands in one place.

The exceptions are the JDK's, and it words them differently on purpose: an
index outside the list is `IndexOutOfBoundsException`, `from` after `to` is
`IllegalArgumentException`, and using a view after the list changed around it
is `ConcurrentModificationException`.

Three `stricter_than_javac!` pins recorded the refusal — one per position
(argument, concatenation, constructor argument), because a construct caturra
refuses must refuse in EVERY position. They are now one `differential_test!`
covering the same three positions and answering.

Pinned by `a_sublist_is_a_view_of_its_list` and
`a_sublist_refuses_a_range_that_is_not_one`. The compatibility page's
`sublist-view` GAP is now a supported feature, recorded against a real JDK.

**Where the new feature met the old ones** is where the rest of it was. A
24-shape sweep of a view against the collection surface found eight more, and
five of them were SILENT: `view.removeIf(…)` left the list alone,
`view.sort(…)` left it unsorted, and `addAll`/`replaceAll`/`remove(Object)`
likewise did nothing — each read the range correctly and wrote to no one. They
are the ordinary list operations now, run over a scratch copy of the range and
spliced back, so a view's `removeIf` IS the same `removeIf` rather than a
second implementation of it. A CURSOR over a view could not be made at all;
every step of the iterator machinery (read, `remove`, `set`, `add`) resolves
the range, so the one legal modification during iteration re-agrees the view
instead of invalidating it. Pinned by
`a_view_meets_the_rest_of_the_collection_surface`.

### A constructed element says what it is (2026-08-20)

`Stream.of(new Point(1, 2)).map(p -> p.x)` had no element type: the pass read
only LITERAL arguments, so the lambda after it saw `Object` — while the same
stream taken from a declared `List<Point>` had the element all along. A `new`
expression says its type outright, which is all this needed.

Two of them can say two types, and the pass had no class hierarchy to join them
with: `Stream.of(new Circle(), new Square()).map(s -> s.area())` stayed
`Object` even after codegen learned that join (see **The element a literal
collection joins at**). The pass carries each class's DIRECT supertypes now and
reads them nearest-first, so a class's own `extends`/`implements` clause wins
over what those extend — the same answer codegen gives, from the same shape of
walk.

Found by probing what a collection does with a USER class: `Comparable`,
`equals`, `hashCode` and `toString` through every container, sorted, hashed,
deduplicated, printed and compared. Twenty-five of twenty-six shapes were
already right — the surface where user code is called FROM native code is one
the earlier rounds built carefully — and this was the twenty-sixth.

Pinned by `a_constructed_element_says_what_it_is`, which also runs the shapes
that must STAY erased (a builder, a diamond, a mixed pair with no join).

### An array of a parameterized type (2026-08-20)

`new List<String>[2]` compiled. JLS §15.10.1 requires a created array's
component type to be REIFIABLE, and two shapes are not: a type VARIABLE and a
parameterized type. Only the first was refused — with javac's own wording,
"generic array creation" — so the rule was half there, and the half that was
missing is the one a program is likely to write.

The check lives in the PARSER, which is the last place the type ARGUMENTS
exist: a `new` expression flattens them (`new ArrayList<Integer>()` needs only
the raw class), so codegen — which reports the type-variable half from a marker
the parser leaves behind — never sees them. What it needs is not the arguments
themselves but whether every one was written as the unbounded `?`, since that
is the one form that leaves the array reifiable; the flag is carried beside
them.

Found by a sweep of 46 programs javac REJECTS, one per rule — type mismatches,
flow, access, overriding, exceptions, lambdas, switch labels — asking only
whether caturra rejects them too. Forty-five did; this was the exception, and
it is the dangerous direction: a program that compiles here and does not
compile on a JDK.

Pinned by `an_array_of_a_parameterized_type`, `an_array_of_a_bounded_wildcard`
and `the_arrays_of_generics_that_are_legal` (the reifiable neighbours the rule
must leave alone — an unbounded wildcard, a raw type, and the cast every
generic-array idiom is written with).

### Three rules a second rejection sweep found (2026-08-20)

The 46-program sweep that found the parameterized-array hole was cheap enough
to repeat over a different 46 rules — interfaces, enums, inner classes,
try-with-resources, constructors, varargs, generics, arrays, switch. Three more
came back, all in the dangerous direction.

**A private member is not inherited** (JLS §8.2). A bare `f()` in a subclass
did resolve to the superclass's private method, and a bare field read to its
private field. The rule was already written down twice — for an interface's
private methods, which are skipped when the interface is reached THROUGH an
implementor, and for a field whose owner shares no top-level class — and the
ordinary case fell between them: two nested classes in one file share a
top-level type, so the field check waved them through, and the method lookup
had no check at all. Inheritance now skips a private member outright, which is
where the rule belongs: it is not an ACCESS question (javac says "cannot find
symbol", not "has private access") but a question of what is there to find. A
private member reached through a receiver of its own class stays legal inside
the same top-level type, which is §6.6.1 and a separate check.

**A primitive cannot be dereferenced** (JLS §15.12). `x.toString()` on an `int`
compiled: the receiver was autoboxed and only a method the WRAPPER lacked was
refused. No method call on a primitive is legal in Java. The autoboxing is
still there for the calls this compiler SYNTHESIZES — a comparator body
compares two unboxed values — so it is kept for a synthesized class and refused
wherever a program is written.

**Two parameters cannot share a name** (JLS §8.4.1). `f(int x, int x)`
compiled; javac refuses the declaration, in those words.

Pinned by `a_private_method_is_not_inherited`,
`a_private_field_is_not_inherited`, `a_private_member_through_its_own_type`
(the neighbours the rule must leave alone), `a_primitive_cannot_be_dereferenced`,
`a_primitive_double_cannot_be_dereferenced` and `two_parameters_with_one_name`.

### Where a hash cursor stops (2026-08-20)

Adding to a one-entry map while iterating it threw `ConcurrentModificationException`
here and finishes quietly on a JDK. The fail-fast model was built on
`ArrayList`'s rule — `hasNext` is a bare `cursor != size`, which is what makes
removing the second-to-last element end a for-each silently — and a HASH or
TREE cursor's is different: `hasNext` is `next != null`, a pointer computed as
each element is handed out, before any later insertion. When the last element
has been returned that pointer is already null, so nothing the loop body does
afterwards can be noticed.

The size the loop STARTED with stands in for that pointer, both in the for-each
lowering (which is an indexed loop) and in an explicit cursor's `hasNext`. A
`Collection` face keeps the list rule, since it may be either. The removals
still throw — there the pointer was pointing AT an element, so the next step
reaches the modification-count check — and so does an insertion with elements
still to come.

Found by a sweep of 46 programs that THROW: null dereferences of every shape,
the arithmetic and index failures, bad casts and stores, the empty-collection
accessors, the fail-fast paths, and a user exception with a cause. Two
divergences came back — this, and a `switch` on a null String with only a
`default` label, which ran the default arm where javac's compiled
`selector.hashCode()` throws.

Pinned by `a_hash_cursor_stops_where_its_pointer_stopped` (including the
neighbours that must still throw) and `a_switch_on_a_null_string`.

### A second Scanner over standard input (2026-08-20)

A JDK's `Scanner` reads its source in BLOCKS, so the first one over
`System.in` takes what a second would have read: a program that makes two finds
the second at end of input, however much is left. caturra gave the second
scanner the rest of the stream, which is the friendlier answer and the wrong
one — a student who writes this here and runs it on a JDK gets the opposite
behaviour, which is the failure this engine exists to prevent.

The stream is a shared resource, and the engine already modelled one half of
that (`close()` on a `System.in` scanner closes the stream, so every later
scanner reads nothing). The other half is buffering: the first scanner to read
OWNS standard input, and a later one is spent from the start. Each still reads
a line at a time, which is what an interactive console can serve — draining the
stream at the first read would block a program that prompts.

Found by a sweep of 26 Scanner-driven programs — the `nextInt`-then-`nextLine`
trap, blank lines, tabs, `\r\n`, a missing final newline, `hasNextInt` over
non-numbers, an empty stream, the two exception paths, a string scanner, a
closed scanner. Only this and one documented refusal (`useDelimiter`) differed.
The probe harness learned to pass standard input for it.

Pinned by `a_second_scanner_over_standard_input`.

### A scanner that reads on its own delimiter (2026-08-20)

`useDelimiter(pattern)` was the last of the Scanner refusals, and the one a
program reaches for to read a comma-separated line. caturra has its own regex
engine, so the token scanner takes a pattern the way the JDK's does; what
needed care was the fine print.

A token is `delimiter? token delimiter?`: ONE delimiter match at the cursor is
skipped before reading, and the one after it is LEFT for the next call.
Consuming the trailing one instead looks identical until the edges — `",a"`
answers an empty first token where a JDK answers `a`, and a `useDelimiter`
BETWEEN two reads starts after a separator the new pattern no longer treats as
one. An empty token between two delimiters is a token like any other, which is
what makes `"a,,b"` three.

The pattern is compiled where the JDK compiles it — in `useDelimiter` — so a
malformed one is a `PatternSyntaxException` from that call rather than from the
first read.

Pinned by `a_scanner_reads_on_its_own_delimiter`, which also runs the default
whitespace behaviour it must not disturb.

### The corpus divergences, adjudicated by the tool (2026-08-20)

The grading sweep has sat at four divergences on the JUnit half and one on the
`org.code` half for months, with a note in `compare.py`'s docstring saying they
were all validators seeded with `Math.random()` — the reference itself flipping
between runs. Re-running the reference five times on one of them confirmed it
(FAIL, FAIL, PASS, PASS, FAIL against a stable caturra), which is the point:
the note was a claim, and a claim is worth re-running.

Half of it was stale. The tool now READS the staged sources and prints WHY each
divergence is one, and the `org.code` case turned out to be a different defect
entirely: two tests annotated `@Order(2)`, which JUnit runs in an unspecified
order, sharing a static counter that one resets and the other does not. The
same submission passes or fails depending on which runs first — the reference
picks one order (stably, on this machine), caturra picks the other, and neither
is wrong. The JVM's tie-break is `getDeclaredMethods` order, which is
unspecified, and caturra already declines to imitate it elsewhere.

The output ends with an UNEXPLAINED count, which is now zero on both halves and
is the number a future sweep should watch: a real regression shows up there
rather than in a total that has to be remembered.

### Files a program keeps beside itself (2026-08-20)

A sweep of 20 file-reading and file-writing programs — a `Scanner` over a
`File`, `BufferedReader`, the `Files` helpers, `PrintWriter`, `FileWriter`,
existence and length, a missing file — found four gaps, one of them the kind
that changes what a program DOES rather than whether it compiles.

**`catch (IOException e)` did not catch a missing file.** The
`java.nio.file` failures were not in the throwable table at all, so a
`NoSuchFileException` was caught by nothing: a program that handles a missing
file died instead of recovering. The table now carries
`FileSystemException` and its four subclasses under `IOException`, which is
where the JDK puts them, so `catch (IOException)`, `catch (Exception)` and a
multi-catch naming `NoSuchFileException` all behave.

**`FileWriter` was refused outright** though the engine already had the writer
it needs — what was missing is the APPEND flag, which is the reason a program
reaches for a `FileWriter` over a `PrintWriter`. Both forms work now, over a
path or a `File`, and appending to a file that does not exist creates it.

**`Files.lines(path)`** had no stream form of `readAllLines`, in either engine
or in the element the lambda after it needs.

The probe harness learned to stage data files (as did the differential suite,
which now clears a file test's directory first — it is keyed by the source, so
an APPEND test accumulated across runs and the JDK answered differently each
time. The re-run check added earlier caught that as "the reference gave two
different answers", which is exactly what it is for).

Pinned by `a_program_that_reads_and_writes_files` and
`a_missing_file_is_an_io_exception`.

**The whole table is now checked against a JDK.** A missing entry matches no
handler and a wrong parent matches the wrong one, and neither shows up in a
test that only throws — so `the_throwable_hierarchy_is_the_jdks` asks a real
JVM, by reflection, for the superclass of every class in the table and compares.
It found one more: `DuplicateFormatFlagsException` was recorded under
`IllegalFormatFlagsException`, and the two are siblings however alike their
names read. Every exception the engine names in its own sources was already in
the table; the 55 parents are now the JDK's.

### A checked exception escaping a lambda (2026-08-20)

`Runnable r = () -> { throw new Exception("x"); };` compiled. A lambda body is
source a program wrote, and a checked exception escaping one is javac's
"unreported exception; must be caught or declared to be thrown" — but the
checked-exception pass skipped every synthesized class, on the reasoning that
such a class "carries its contract on an erased functional interface".

It does, and that contract is the answer rather than a reason to skip: what a
body may throw is what the INTERFACE's own method declares. The synthesized
method now carries those `throws`, so a user interface written
`void go() throws Exception` permits the throw and the bundled ones — which
declare nothing — do not. Anonymous and local classes are checked the same way,
which they were not before either.

Lifting the skip immediately failed a pinned test, and the reason was a
separate hole: the pass had only PARAMETERS and locals in scope, so a receiver
whose type comes from a FIELD was not a typed receiver at all. Its call was
taken to throw nothing, and a `catch` around it read as "never thrown in body of
corresponding try statement". A captured variable is a field of the synthesized
class, which is how a lambda reached the same hole — but a plain
`reader.readLine()` on a field reached it too, and had all along.

Found by comparing the WORDING of 131 rejected programs against javac's. Two of
the differences were defects rather than style: this one (caturra accepted the
program and threw at run time) and a varargs parameter that is not last, where
breaking out of the parameter list left the `,` behind and the message blamed
the punctuation — "expected ')'" for a rule about parameters.

Two more messages named the wrong thing, and both are now javac's:
`Map<String>` said "unknown type 'Map'" — the base is perfectly well known and
what is wrong is the COUNT, which is not approximate for the containers however
their arguments are modelled — and `Long l = anInteger;` said "int cannot be
converted to Long", describing a type the program never wrote, because the
conversion unboxes the wrapper before failing and the diagnostic followed it
down. (Pinned by `a_container_with_the_wrong_number_of_arguments`,
`a_wrapper_that_does_not_convert` and `the_wrapper_conversions_that_are_legal`.)

Pinned by `a_checked_exception_escaping_a_lambda`,
`a_checked_exception_escaping_an_anonymous_class`,
`the_checked_exceptions_a_lambda_may_throw` (the shapes the rule must leave
alone, including the field receiver) and `a_varargs_parameter_that_is_not_last`.

### synchronized, on one thread (2026-08-20)

`synchronized (lock) { … }` was refused as "not supported by caturra; programs
run single-threaded" — which is the reason it needs no support. A monitor that
is never contended does nothing: the statement means evaluate the lock, throw
if it is null (`monitorenter` does, and a program can catch it), and run the
body. The modifier form on a method had been accepted and ignored for the same
reason all along; only the statement was refused, and a textbook's synchronized
counter would not compile.

It lowers to a block whose first statement dereferences the lock — the same
check the instruction performs, which also refuses a primitive lock the way
javac does ("int cannot be dereferenced" against javac's "unexpected type").

Three tests had pinned the refusal, two of them using `synchronized` as their
example of "valid Java caturra does not implement, worded so the corpus tooling
can tell an engine gap from a student's mistake". They use a different example
now — the fourth time this session that a test recorded a limitation as a
requirement.

Pinned by `a_synchronized_block_on_one_thread` and
`a_primitive_is_not_a_monitor`; the compatibility page's `g-synchronized` gap is
now a supported feature, recorded against a real JDK.

### A fold written as a method reference (2026-08-20)

`reduce(0, Integer::sum)` was refused: "a lambda or method reference is only
allowed where a functional-interface type is expected". Every other stream
operation converts a METHOD REFERENCE to the equivalent lambda before erasing
it — the two-argument fold's arm matched only a two-parameter LAMBDA, so the
reference reached codegen unconverted and took the whole program with it.

Found by a different shape of probe: 130 programs composed by drawing six to
nine independent, self-contained statements at random from a pool of forty and
running the mixture. A unit-shaped probe never writes `reduce(0, Integer::sum)`
next to a `synchronized` block and a `subList`, and this is the sort of gap that
survives a hundred single-feature tests — the pool covered `reduce` and covered
method references, and the pairing was what nothing had run.

Pinned by `a_fold_written_as_a_method_reference`, alongside the lambda
spellings that always worked.

Widening the pool to seventy snippets — everything added this session included
— found two more, both in the same shape of pairing:

- **`Stream.of(new int[] {1, 2})` answered 2.** The varargs gotcha again: a
  lone REFERENCE array IS the varargs array, and a primitive one is not, since
  `T` cannot be `int`. The list factories learned this long ago; the stream
  factory spread both alike. It still spreads for `IntStream.of`, whose element
  IS the primitive, and for `Arrays.stream`, which takes an array by
  definition.
- **`Map.of(k, v)` used straight as a receiver** had no key or value type, so a
  lambda over its `entrySet()` had no element — the reading `List.of` already
  got, missing for the map factory.

Pinned by `a_lone_array_handed_to_a_stream_factory`.

### A lambda in an initializer block (2026-08-20)

`static { list.forEach(x -> …); }` was refused for having no
functional-interface position. The lambda pass walked a class's METHODS and its
FIELD initializers, and not its initializer BLOCKS — the two places a class
runs code that is not a method, both of them missed. The capture pass had
walked them all along, which is why the failure surfaced as a target-typing
message rather than as a missing capture.

Found by the other cross-product: every statement from a pool of
twenty-five, placed in each of twenty-two syntactic CONTEXTS — a loop, an `if`,
a lambda, an anonymous class, a static and an instance method, a constructor, a
static and an instance initializer, an enum method, an interface default, a
generic method, a recursive method, a lambda in an anonymous class and the
reverse, nested blocks, a `finally`, a switch arm. 550 programs; the initializer
blocks accounted for twelve failures and a local class in a switch arm for one
(now enumerated as a strictness).

The same sweep found a second, older defect the initializer case had been
hiding behind: **`count++` on a field of the ENCLOSING class** — from a lambda
or an inner class — resolved only against the class it stands in and said
"cannot find symbol". The read path walks the captured-outer chain and the
ASSIGNMENT path does too, so `count = count + 1` compiled while `count++` did
not, for the same field in the same body. (It surfaced now because a lambda
body that is a bare `x++` became an EXPRESSION body earlier this session; the
statement form had always failed the same way.)

Pinned by `a_lambda_in_an_initializer_block` and
`an_increment_of_an_enclosing_field`.

### What a generated stream, and a primitive Optional, are (2026-08-20)

The context cross-product was run a second time with a different pool — the
statements stressing the features added this session rather than the language
at large — and this time nothing was context-specific: both failures happened
in all twelve contexts alike, which is its own result. The passes now agree
about WHERE; they disagreed about WHAT.

**`Stream.generate(supplier)` had no element.** A stream's element is what its
source yields, and every other source says so: `of` joins its arguments,
`iterate` reads its seed, `Arrays.stream` reads the array. `generate` erased —
to `Object` in codegen, and in the lambda pass to nothing at all, which is
worse: an element of NOTHING gives every downstream lambda in the chain no
functional-interface position, so
`Stream.generate(() -> 2).limit(2).reduce(0, (a, b) -> a + b)` was refused
outright while the identical fold over `Stream.iterate` compiled. The
supplier's answer was already being recorded — `map` reads it off the
synthesized lambda class through the synthetic `__caturraProduces` field — so
both sides now read the same field, and the element boxes on the way out
because an object stream's element is a reference (a supplier answering `2`
makes a `Stream<Integer>`, and reading it as a bare `int` sent the fold's
result back unboxed, which is a `VerifyError` rather than a diagnostic).

**A primitive Optional's factory had no type.** `OptionalInt.of` and its two
siblings are emitted inline, the way the stream sources are, and their static
table is deliberately EMPTY — so the mirror that types an expression found no
method and answered `Error`. `OptionalInt.of(3).getAsInt() + 1` was "bad
operand types for binary operator '+'"; the same Optional held in a variable
first added fine. That is the recurring shape of this defect class — the
emission path and the typing path are two readings of one fact, and only one of
them was updated when the factories were added.

Pinned by `a_generated_streams_element_is_its_suppliers` and
`a_primitive_optional_factory_has_a_type`.

### The mirror probe (2026-08-20)

Naming that defect class made the probe for it obvious: take every library
expression that is emitted INLINE — the stream sources, the `Optional` and
collection factories, `String.format`, the nio calls, `Map.entry` — and put
each one in eight positions that read its type by different routes: printed,
passed as an argument, passed through a generic method, joined in a ternary, in
a `var`, in a FIELD initializer, inside a lambda body, and as an array element.
Thirty-four expressions, 272 programs.

Three findings, and the first is the same shape as the last two units:

- **`LongStream` and `DoubleStream` were not references.** The is-a-reference
  list named `IntStream` and stopped. So a `LongStream` could not be passed as
  an `Object`, held in a `var`, stored in an `Object[]`, or joined in a
  ternary: "incompatible types: LongStream cannot be converted to Object", for
  a value as much an object as the `IntStream` beside it. All eight positions
  failed; the emitting position was fine, which is why nothing had noticed.
- **An EMPTY stream had no context-free type.** `var s = Stream.of(1)` inferred
  and `var s = Stream.empty()` did not, because a factory that adopts its
  context has none to adopt in a `var`. The table that already answers this for
  `Collections.emptyList()` gained the four stream empties.
- **`String.valueOf(Collections.emptyList())` threw a ClassCastException.** The
  JDK's most specific `valueOf` overload for a null-typed argument is
  `valueOf(char[])`, and javac binds it — that is why `String.valueOf(null)`
  throws. Right for the null LITERAL; wrong for the factories that type like
  null because they ADOPT their context, and an overload set is the one place
  where the context is what is being chosen.

The last one has a wider version that is NOT fixed, and the attempt is worth
recording: giving those factories their context-free type everywhere
(`List<Object>`) makes `two(Collections.emptyList())` pick the list overload as
javac does — and makes `list.addAll(Collections.emptyList())` a type error,
because then the element is checked and `Object` is not `String`. The lenient
null-like typing is load-bearing. It is now enumerated as a strictness with the
reason attached, rather than left as a surprise.

Everything the probe still disagrees about is a JDK-internal implementation
class name — `ReferencePipeline$Head`, `UnixPath`, `KeyValueHolder`, a lambda's
generated name. Those are platform detail (the `sun.nio.fs` one differs by
operating system), and caturra does not mimic them.

Pinned by `every_primitive_pipeline_is_a_reference`,
`a_context_adopting_factory_is_not_the_null_literal`, and
`stricter_a_context_free_factory_in_an_overload_set`.

### The sorted views (2026-08-20)

Ten methods refused outright — "TreeSet range views are not supported by
caturra", "TreeMap.descendingMap is not supported by caturra". They are one
object now: `headSet`/`tailSet`/`subSet`, `headMap`/`tailMap`/`subMap`,
`descendingSet`, `descendingMap`, `descendingKeySet` and `navigableKeySet`
differ only in their BOUNDS, their DIRECTION, and which face they present (the
set, the map, or a map's keys as a set).

**The bounds are values, never positions.** That is the whole design, and it is
what makes the view LIVE the way a JDK's is: a key put into the tree inside the
range shows through, one outside it does not, and `map.headMap(k).clear()`
deletes a range from the tree. A snapshot would have been far less code and a
silent wrong answer the first time a program mutated the backing.

Resolving a bound means COMPARING, which may run the program's own
`compareTo` — so the view resolves its range on every call made through it,
where the interpreter is at hand. The readers that hold nothing but the heap
(printing, a `for` each, hashing) re-resolve it themselves whenever the
ordering is native, which covers every tree keyed by a number, a character or
a String; only a tree ordered by user code reads the range the last call
resolved. The bound rule itself is written ONCE, taking the comparison as a
callback, so the two callers cannot drift apart.

Three things the probe caught that the design did not anticipate:

- **A view's cursor carries the TREE's `modCount`, not its own.** Adding a key
  OUTSIDE the range still ends a walk of the view with a
  `ConcurrentModificationException` — and the view's own length, which that add
  never touched, could not notice. So the expectation is stamped where a walk
  BEGINS (`size()`, which every enhanced-for calls first) and re-stamped by a
  mutation made through the view, which is exactly when a JDK cursor re-syncs.
  Stamping it on every range refresh — the obvious place — silently forgave the
  very change the check exists for.
- **"The wrong way round" is decided in the VIEW's order.** On a descending set
  `subSet(7, 3)` is an ordinary range; read in the backing's ascending terms it
  looked like `fromKey > toKey` and was refused.
- **A copy of a view adopts the ordering the view presents.**
  `new TreeSet<>(s.descendingSet())` is descending, because `TreeSet(SortedSet)`
  takes the source's comparator — so a descending view has to be able to name
  one, which it builds by reversing its tree's.

What is NOT done, and is enumerated as a divergence rather than hidden: the
sorted collections still have one compile-time FACE, so `SortedSet` offers the
`NavigableSet` methods. That predates the views (`SortedSet<Integer> s = …;
s.floor(3)` already compiled); they make it reachable from more places.

Pinned by `a_sorted_collection_has_range_and_descending_views`,
`a_sorted_map_view_writes_through` and
`a_view_orders_itself_the_way_its_tree_does`.

### The sorted faces (2026-08-21)

The divergence the views left behind, closed the next morning: a sorted
collection has THREE compile-time faces, and the JDK's split between them is
observable. `SortedSet` declares `comparator`, `first`, `last` and the three
range views and nothing else; the navigation — `floor`, `ceiling`, `lower`,
`higher`, `pollFirst`, `pollLast`, `descendingSet`, `descendingIterator` — is
`NavigableSet`'s; `clone` is the class's alone. `SortedMap`/`NavigableMap`
divide the same way, with the entry accessors and the key-set views on the
navigable side.

That split is exactly why the one-argument `headSet(E)` answers a `SortedSet`
and cannot be polled, while `headSet(E, boolean)` answers a `NavigableSet` and
can. Both compiled here until now.

`JType::TreeSet` and `JType::TreeMap` carry a [`SortedRole`], the way
`LinkedList` already carries its `Queue`/`Deque` role. Three decisions worth
recording:

- **One table, filtered by face.** The three faces are NESTED interfaces, not
  three unrelated types, so splitting the method table three ways would have
  been the same fact written three times. Each entry names the narrowest face
  that declares it, and a receiver's own face is measured against that.
- **The face rides in `TypeArgs`.** Every caller that resolves a member already
  threads those through; a separate parameter would have had to be added at
  each of them, and the one that was forgotten would silently offer the whole
  table.
- **A missing face-member is a missing SYMBOL.** Not a bad overload: javac says
  "cannot find symbol: method floor(int)" for a `SortedSet`, and so does this
  now — the name has to be invisible, not merely unmatched.

A face widens only outward: a `TreeSet` is a `NavigableSet` is a `SortedSet`,
and `TreeSet<Integer> t = s.descendingSet();` is now the error it always was.

Two things the sweep behind it found, neither about faces:
`sortedSet.comparator()` was missing from the tables entirely (and needed the
`Comparator` return to fall back to `Object` for a program that names no
comparator — a natural-ordering collection answers null); and `spliterator()`
now gives its honest reason rather than "cannot find symbol", since caturra
models no such type. The `Set` face also offered `descendingIterator`, which
neither `Set` nor `HashSet` declares.

What remained was the same shape one level over — the HASH collections had one
face, so `Set` and `Map` offered `clone` — and "Which face a collection wears"
below applies this same role to them.

Pinned by `each_sorted_face_offers_its_own_members`,
`a_sorted_face_has_no_navigation`,
`a_one_argument_head_set_answers_a_sorted_set`,
`a_sorted_map_face_has_no_entry_navigation` and
`a_narrow_face_does_not_widen_inward`.

### What a broad API sweep found (2026-08-21)

Eighty-two programs, one per commonly-used library call or language shape,
each run against a real JDK. Six of the ten disagreements were the honest
refusals working as intended (`java.time`, `java.math`, `java.lang.Thread`) or
the harness's own missing clock. The other four were real:

- **`Iface.super.method()` aborted the VM for a NESTED interface.** The method
  reference names the CLASS FILE, which for a nested type is `Outer$Inner`, and
  the written name was interned instead — so the identical call to a top-level
  interface, whose two names agree, worked all along. An abort on ordinary Java
  is the worst answer available, and this one had been reachable since nested
  types got their binary names.
- **`catch (InterruptedException e)` did not compile**, because the class was
  missing from the throwable table. `java.lang.Thread` is refused (one thread,
  so a sleep would be a lie) and the refusal says so — but the catch clause
  failed FIRST, with "cannot find symbol", which blames the one part of the
  program that is right. The closed-world thrower table is a model of JAVA, not
  of what caturra runs, so it now records that `Thread.sleep` throws it.
- **`StringJoiner` did not exist.** It is bundled Java now, and its details are
  observable: the builder holds the prefix and the elements but never the
  suffix, which is what makes `merge` splice another joiner's contents WITHOUT
  its prefix and `length()` answer before `toString()` ever runs.
- **`Random.ints`/`doubles` and `Arrays.mismatch` were missing.** The random
  streams are LAZY, as the JDK's are, so the generator advances once per
  element pulled — observable in the seed a later `nextInt()` draws from. They
  are built from lambdas, which uncovered a second thing: the trigger that
  injects the functional interfaces reads the USER's text, and says nothing
  about what an injected library itself needs.

`EnumMap`/`EnumSet` moved from "unknown type" — which reads as a typo about a
class the documentation shows — to the honest scope limit, with the substitute
named in the code: a `TreeMap`/`TreeSet` keyed by the enum iterates in the very
same order, an enum's natural ordering being its ordinal.

Pinned by `an_interface_super_call_names_the_class_file`,
`a_string_joiner_matches_the_jdk` and
`a_seeded_randoms_streams_replay_the_jdks`.

### Which method the message blames (2026-08-22)

A second sweep of the API surface came back clean — 80 of 82, both remaining
being deliberate refusals — so the probe turned to the other half of what this
engine owes a student: the DIAGNOSTICS. Fifty programs, each wrong in one
ordinary way (an undeclared variable, a lossy assignment, a missing return, an
unreachable statement, a catch out of order, a final override, `"5".length`),
compared with javac's own wording.

Every verdict agreed, and most messages already did — several are deliberately
more specific than javac's, which says "cannot find symbol" where caturra names
the symbol. Three did not, and they were one rule:

**With exactly ONE candidate of that name and ARITY, javac blames the
ARGUMENT** — "incompatible types: String cannot be converted to int" — because
there is no doubt which parameter the argument was meant for. With no candidate
of that arity and only one overall, it says the lists differ in length. Only
when several candidates share the arity does it report no suitable method,
there being no single culprit. (This is javac's default `-Xdiags:compact`
behaviour; `-Xdiags:verbose` prints the full overload set either way.)

caturra had a NARROW version of that rule on the builtin path — one candidate,
both types numeric, report the lossy conversion — and none at all on the
user-method path. So `f("s")` against a single `f(int)` reported an overload
failure, and `stringList.add(1)` said "no suitable method found for add(int)"
rather than naming the int. The rule is now written once and used by both;
`Resolution::NoneApplicable` carries the candidates so the caller can apply it.

One thing the shared rule needed: a functional parameter models as an erased
`Object` here, so the message would have read "int cannot be converted to
Object" — true of the model, and silent about the mistake. Those parameters
name their Java interface instead.

The change promoted a pinned wording divergence (`WordNextBytes`) to word-for-
word agreement, which is what those pins are for.

Pinned by `WordOneCandidateArg`, `WordOneCandidateArity`, `WordTwoCandidates`
and `WordAddElement` in `reject_wording_tracks_javac`.

### A raw type is not `<Object>` (2026-08-22)

A second batch of fifty wrong-in-one-way programs agreed with javac on every
verdict but one, and that one was not a diagnostic at all:

```java
List raw = new ArrayList();
List<String> typed = raw;      // javac: unchecked warning. caturra: error.
List back = typed;             // javac: nothing at all. caturra: error.
```

Raw types were modelled as their erasure with `Object` arguments — right about
what the members read and write, wrong about CONVERSION. `List` and
`List<Object>` are different types in the JLS: the unchecked conversion (§5.1.9)
runs between a raw type and any parameterization of it in both directions,
while `List<Object> l = aStringList;` is the error javac calls it. With one
shape for both, caturra had to refuse all three, and refused the two that are
legal.

A raw argument is now its own `WildcardBound::Raw` — the marker says "unknown,
and unchecked". It reads out as `Object` exactly as before, and unlike `?` it
may still be written to, which is what keeps `raw.add("a")` working. Three
places had to learn it: the element rule (a raw element matches anything, in
either direction), the widening rule (`List`/`Set`/`Collection`/`Iterator` were
missing from the family that compares elements at all — so a raw list could
convert to a raw map's cousin but not to a `List<String>`), and the conversion
MATRIX, which gates separately and had an arm for a wildcard TARGET and none
for a wildcard SOURCE. That last one is the trap this file already warns about
twice: both gates need the arm, and with only one of them the message is the
nonsense "Iterator<Object> cannot be converted to Iterator<Object>".

The marker also retired a documented strictness: `Map[] raw; raw[0] = new
HashMap<String, Integer>();` was refused because the pass could not tell a raw
element from a written `<Object>` one. It can now, so the unchecked store is
allowed and `Map<Object, Object> m = new HashMap<String, Integer>();` still is
not.

Pinned by `a_raw_element_array_takes_a_parameterized_value`.

### What a trace names (2026-08-22)

Fifty programs that FAIL at run time — every null dereference, bad index, bad
cast, bad parse, unmodifiable write, comodification and empty-collection read a
student meets — compared with a real JDK.

**Every exception class and message is byte-identical**, including the places
the JDK is inconsistent with itself: `Index 2 out of bounds for length 1` from
a list read but `Index: 3, Size: 0` from a list insert, `begin 2, end 9,
length 3` from a substring but `String index out of range: 5` from a charAt.
Matching that means matching each site where it throws rather than picking a
house style, and it already did. The frames match too, name for name and line
for line, `Outer$Inner.<init>` included.

Two things did differ.

**A trace named caturra's own library.** Most of the class library is native
and contributes no frame at all; the few classes that are bundled JAVA
contributed one — `at Random.nextInt(<util>:31)`, naming a file that does not
exist and a line in caturra's source. A JDK shows a library frame here too, but
as `java.base/java.util.Random.nextInt(Random.java:388)`, and inventing that is
worse than saying nothing. Saying nothing is what every natively-modelled call
already did, so a frame from an injected unit is now hidden and a trace names
the program's own calls only.

**An ambiguous call listed every candidate.** javac names exactly TWO — the
word is "both", and `Arrays.sort(null, 0, 1)` listing nine of them after it is
not a sentence.

Pinned by `the_runtime_failures_read_like_the_jdks`,
`a_trace_names_the_programs_calls_not_the_bundled_librarys` and
`stricter_a_null_literal_to_a_bounded_collections_method`.

### Reading input by pattern (2026-08-22)

Two more sweeps came back almost empty, which is worth recording as much as a
fix: every `printf`/`String.format` specifier, flag, width, precision and
malformed form — 75 programs, including the HALF_UP rounding corners
(`%.0f` of 2.5 and of 3.5), `%,d`, `%(d`, `%#x`, `%a` and argument indexes —
prints byte-identically. And 25 Scanner programs, the `nextInt()`-then-
`nextLine()` trap included, agree on all but two.

**`hasNext(pattern)` / `next(pattern)` did not exist.** The token has to match
the pattern in FULL, which is what the bundled regex engine already answers for
`String.matches`, and a token that does not match is LEFT WHERE IT IS — which
is the whole point of the overload, since a program tries one pattern and then
another. A mismatch is an `InputMismatchException`, the refusal `nextInt`
already gives a token that is not a number.

**Input that does not end with a newline** reads differently, and this one is
enumerated rather than fixed. A JDK's Scanner reads a byte stream, so after
`nextInt()` takes the last token there is no terminator left: `hasNextLine()`
is false and `nextLine()` throws "No line found". caturra's host contract hands
the VM one LINE at a time — `ConsoleIo::read_line`, which the browser fills
from the input box and the CLI from stdin — and a line-shaped stream cannot say
whether the last line was terminated, so every line reads as terminated. That
matches an interactive console, where the student presses Enter and the
terminator is real; it diverges only for input piped without a final newline.
Fixing it means teaching both hosts to report the terminator.

Pinned by `a_scanner_reads_by_pattern` and
`a_line_shaped_stream_treats_every_line_as_terminated`.

### The collector (2026-08-22)

The heap only ever grew. Every object a program allocated stayed allocated, so
a loop that builds a list and drops it — the shape of every animation frame,
every simulation step, every `+=` on a string — cost memory forever. Three
million iterations of a small loop took **2.37 GB**; the same loop takes **18
MB** now, and runs slightly faster for the reduced allocation pressure. In a
browser the difference is not a slow program but a dead tab.

It is mark-and-sweep, and the three decisions worth recording are all about
safety rather than speed.

**Where it runs.** Native code allocates freely — an intrinsic may build a
holder map, an entry and three strings before it returns — and those
intermediate references live in Rust locals no collector can see. Between two
bytecode instructions, at the base nesting level, no such local exists: every
reference the program holds is in a frame, a static, an interned pool or a side
table. So the collection point is there and nowhere else, and a nested run (a
`toString` called from native code while a container renders) suppresses it.
One case escaped that rule and had to be given a root of its own: `main`'s
argument array is built BEFORE the entry class's `<clinit>` runs, so until
main's frame exists nothing points at it.

**What it walks.** One `visit_refs` per heap kind, with NO wildcard arm — a
variant that holds a reference cannot compile until it is listed. The side
tables (a stream's origin, a map's cached views, a cursor's pending elements)
are traced when their subject survives and pruned when it does not: a swept
slot is handed out again, so an entry left behind would attach an old object's
state to a new object at the same index.

**How it is trusted.** `CATURRA_GC_STRESS=1` collects at every safepoint, and
CI runs the whole VM suite that way. That is what turns a forgotten reference
from a silent corruption into a failing test — and it is how the one that WAS
forgotten got found: a sorted view's bounds are values, and
`set.headSet(new Point(4))` holds the only pointer to that Point.

Nothing moves and nothing is compacted: a reference is a slot index, so a
swept slot is simply available again and every reference the program holds
keeps pointing at the object it named.

Pinned by `the_collector_reclaims_garbage_and_keeps_the_live_set`.

A collector makes two more things possible, and they arrived with it.

**A full heap is now a catchable Java error.** A program that really does hold
more than the budget — a gigabyte of LIVE objects by default — gets
`OutOfMemoryError: Java heap space` at a safepoint, raised after a collection
has already run, so only what is still reachable counts against it. The same
program used to grow until the host refused, which in a browser is a dead tab
rather than an error a student can read (and can catch: the pinned test catches
it, drops the list, and allocates again).

**`System.gc()` exists.** It is a REQUEST in Java — "the Java Virtual Machine
expends effort" — and one here too: the collector runs at the next safepoint.
A program cannot observe whether it collected, which is exactly why the JDK is
free to ignore the call and why this one can honour it.

`java.lang.Runtime` is refused by name rather than reading as a typo: every
number it could answer (`freeMemory`, `totalMemory`, `maxMemory`) would be
fiction about a heap the program cannot influence, and `exec` has nothing to
execute.

Pinned by `a_full_heap_is_a_catchable_java_error` and
`a_program_may_ask_for_a_collection`.


### Fuzzing the arithmetic (2026-08-23)

Random expression trees over `int`/`long`/`double`/`char`/`boolean` — mixed
operators, casts, shifts, comparisons, ternaries, edge literals — generated
from a seed, printed, and compared with a real JDK. Roughly two thousand
expressions; one disagreed, and it was a silent wrong answer:

```java
System.out.println("x=" + ((Double.MIN_VALUE <= 0.0) ? "then" : "else"));
```

A JDK takes the else. caturra took the then, because the constant FOLDER
spelled `Double.MIN_VALUE` as `MIN_POSITIVE * EPSILON / 2.0` — and that rounds
to ZERO. (`MIN_POSITIVE * EPSILON` is already the smallest subnormal; halving
it underflows.) So the folded constant was zero, `zero <= 0.0` was true, and a
conditional took a branch a JDK never takes.

Only through a FOLD: printing `Double.MIN_VALUE` reads the emitter's table,
which had the right value all along. Two tables of the same library constants,
and only one of them right — the shape this file has recorded four times now.
So the fix is not the value but the duplication: the folder reads the emitter's
table, and there is no second table to disagree with.

The rest of the fuzz agrees exactly, `Integer.MIN_VALUE / -1`, `>>>` masking,
`-0.0`, subnormals, NaN comparisons, integer overflow and `/ by zero` messages
included.

Pinned by `the_folded_wrapper_constants_are_the_jdks`.

### The assignment cross-product (2026-08-23)

Every assignment between thirty-seven types — the primitives, the wrappers,
`String`/`Object`/`CharSequence`/`Comparable`, three array shapes, seven
parameterizations of `List`, a raw one, `Set`/`Map`/`Collection`/`Iterable`, a
user class hierarchy — as its own program. 1369 of them, compiled by javac in
one invocation and by caturra one at a time, comparing only the VERDICT.

Nineteen were accepted here and refused by javac, and they were all one thing:
**a diamond typed as `null`**. `new ArrayList<>()` had no element to name, and
the type that means "no element" was `JType::Null` — which assigns to any
reference at all. So `Integer x = new ArrayList<>();` compiled, and `String s =
new ArrayList();` with it. A typo javac catches, run instead.

A diamond is not a null: it is a collection whose element the program did not
write, which is exactly what the RAW marker means — unknown, and unchecked in
either direction. Typed that way, `List<String> l = new ArrayList<>()` still
converts and `Integer x = new ArrayList<>()` is the error javac calls it.

Making the two agree took the emitter and the mirror TOGETHER, and the
compiler's own `CATURRA_VERIFY_TYPES` check is what insisted: it compares what
`type_of` says against what the emitter returns, and it failed on
`new HashSet<>()` the moment the two disagreed. Half a fix would have been
silent otherwise.

Four assignments went the other way — refused here, accepted by javac — and
they were two rules:

* **The constant-narrowing rule reaches the WRAPPER targets** (JLS §5.2), and
  from a constant of any integral type: `Character c = (short) 1;` is a
  narrowing followed by a boxing. Only `int` sources were listed, so
  `Character c = 1;` worked and `Character c = (short) 1;` did not.
* **The unchecked conversion applies to the wider FACES too.** A raw
  `ArrayList` converts to `Collection<String>` and to `Iterable<String>`, not
  only to `List<String>`; those arms compared elements with `==` where the rest
  of the family had moved to the element rule.

Both directions are now zero across all 1369.

Pinned by `a_diamond_is_a_collection_with_an_unwritten_element`,
`a_diamond_is_not_assignable_to_anything` and
`a_raw_collection_is_not_assignable_to_anything`.

### Which overload runs (2026-08-23)

The same cross-product, one level up: fourteen overload SETS against nineteen
argument shapes, each program printing which method ran — so the comparison is
the JDK's choice, not merely its verdict. 266 programs.

Eleven failed to compile at all, and for one reason: `f(double)` and
`f(Double)` were reported as a **name clash**. The erasure key for a parameter
was the type's `Debug` text, and `TypeRef::Double` debug-prints as `Double` —
the wrapper's name exactly. So the two collided, and with them `f(long)` and
`f(Long)`, `f(float)`/`f(Float)`, `f(short)`, `f(byte)` and `f(boolean)`; only
`int` and `char` were spelled differently enough to escape. A primitive's key
now says it is a primitive.

Two more chose the wrong method, and that was **unboxing followed by a widening
primitive conversion** (JLS §5.3): `f(double)` accepts an `Integer`, and
`double d = anInteger;` assigns. Only the exact-width unboxing was modelled.

Adding it needed care, and the sweep is what showed why: the conversion is a
phase-TWO one. Letting it into phase one put `f(long)` and `f(double)` among
the strictly-applicable candidates for an `Integer` argument — where `f(int)`
is NOT, since its unboxing is also phase two — so the call picked `f(long)`.
It compiled, it ran, and it called the wrong method. Unboxing of any width is
now excluded from phase one, which is what JLS §15.12.2 means by trying without
boxing first.

All 266 agree.

Pinned by `which_overload_runs`.

### What counts as an override (2026-08-23)

The third cross-product: ten superclass declarations against seventeen subclass
ones — access, finality, staticness, return type, parameter type, throws
clause — 170 programs, each dispatching through a supertype reference and
printing which body ran.

Twenty-six were accepted here and refused by javac, all the same shape:
`@Override` on a method that OVERLOADS rather than overrides. A subclass
writing `speak(String)` against a base's `speak(Object)` declares a second
method, and the annotation is there to say so.

The rule underneath was "an ancestor's `Object` parameter stands for an erased
type variable", which is true of the bundled functional interfaces
(`__Comparator.compare(Object, Object)`) and of nothing else. It was already
carved out for `java.lang.Object` itself, because `@Override boolean
equals(Bad o)` — the classic bug the annotation exists to catch — must fail;
every ordinary user class needed the same carve-out.

Tightening it exposed the opposite error in the same check: a BOUNDED type
variable erases to its BOUND, so `Bounded<T extends Comparable<T>>.take(T)`
reads as `take(Comparable)` while its implementor writes `take(String)`.
That is an override, and it was refused — the tolerance had been hiding it.
The rule now consults the type argument the subclass actually WROTE for the
supertype, which is the substitution the JLS describes rather than a guess
about `Object`.

And there were FOUR copies of "is this an overriding signature" — the
`@Override` check, the not-abstract check, the interface-default resolution and
the implementation lookup — each with its own tolerance, none agreeing. They
are one function now.

All 170 agree, and so do the 266 overload selections and the 1369 assignments.

Pinned by `what_counts_as_an_override` and
`an_override_annotation_needs_the_same_signature`.

### Which casts exist (2026-08-23)

The fourth cross-product, and the first where both compile-time legality and
run-time behaviour matter: 25 types cast to 25 types, 1100 programs, each
printing either the value it got or the `ClassCastException` it caught — once
directly and once through an erased `Object` so `instanceof` answers first.

**206 disagreed.** 147 of them in the accepts-invalid direction, and all one
cause: the rule was written once per TARGET family, and each of those arms
carried its own list of source types it would accept. A family whose arm nobody
had written accepted *everything*. `(int[]) "s"`, `(List<Object>) "s"` and
`(int) aParent` all compiled and threw at run time, where a JDK refuses to
compile them.

JLS §5.5 is now asked ONCE, before any family gets a say:

- **Primitive to primitive** — every numeric pair, and `boolean` with nothing
  else.
- **Primitive to reference** — a boxing conversion, then a widening reference
  one. `(Number) 5` is legal (the box's own supertype) and was refused.
- **Reference to primitive** — an unboxing conversion, optionally followed by a
  WIDENING primitive one. `(long) anInteger` is legal, `(int) aDouble` is not:
  a cast narrows a primitive it already has, never one it just unboxed. From
  anything that is not itself a wrapper the cast goes through the TARGET's own
  wrapper — `(int) aNumber` *is* `(Integer) aNumber` unboxed — and every
  wrapper is `final`, which is what makes `(int) aParent` an error rather than
  a run-time failure.
- **Reference to reference** (§5.5.1) — one a subtype of the other, or some
  class could still be both, which an INTERFACE always leaves open unless the
  other side is `final`. Arrays cast when their elements do; a primitive
  element must match exactly.
- **Provably distinct parameterizations** (§4.5) — no class is both a
  `List<String>` and a `List<Object>`, so that cast is an error, not the
  unchecked warning a raw or wildcard one earns. Every wrapper is a
  `Comparable`, but of ITSELF: `(Comparable<String>) Integer.valueOf(1)` is
  refused too. This has to be asked BEFORE the subtype tests — a wrapper *is* a
  `Comparable` and a `List<String>` *is* a `Collection`, so "related?" answers
  yes about a pair whose arguments make it impossible.

The rule answers "legal" for anything it cannot classify — the erased `Object`
a bridge method or a specialized lambda parameter casts from is the everyday
one — so it turns away only pairs it can prove unrelated.

`instanceof` is the same question. JLS §15.20.2 says so outright ("if a cast of
the operand to the type would be rejected as a compile-time error, then the
instanceof likewise produces one"), and asking it any other way was the same
fact written twice: the check here compared the two types for being
`String`-or-a-wrapper, which caught `"x" instanceof Integer` and let every
other impossible test — `"s" instanceof List`, `aSet instanceof Integer` —
answer a plain `false`. A second cross-product over the same types (378
programs) found 40 of those, of which 31 are now the error javac gives.

Three things the sweep found besides:

- The rejection MESSAGE was caturra's own — "cannot cast X to Y", a sentence
  javac has never written. 245 of the 284 rejects the two engines already
  agreed on differed in their wording. It is javac's now, with a nested class
  named as the source wrote it (`Child`, not `Outer$Child`) and type arguments
  spelled without the space after the comma that caturra had used.
- A diagnostic about a WRITTEN type now renders what the program wrote rather
  than the resolved `JType`: `List` and `ArrayList` are one type here, so a
  message about a cast to `List<Object>` named `ArrayList<Object>` — a class
  the program never mentioned.
- `Number` and `Comparable` are modelled under a BARE name, since caturra does
  not compile them from source. Every `ClassCastException` about one therefore
  put a JDK class "in unnamed module of loader 'app'" — unless the program
  really declares a class of that name, which the loaded classes are the record
  of.

**12 of 1100 remain**, all one shape, and enumerated under the divergences: a
cast from a CONCRETE collection to an unrelated class. `List` and `ArrayList`
are one type, so the rule answers for the interface — the choice that never
turns away a legal program. ("Which face a collection wears", below, gives them
two types and closes these twelve as well.)

All four cross-products now agree in both directions: 1369 assignments, 266
overload selections, 170 overrides, 1088 of 1100 casts.

Pinned by `diff_cast_conversions_that_exist` and the eleven cast and
`instanceof` rejections in `stage6_compile_errors_match_javac_wording`.

### Which face a collection wears (2026-08-23)

`List` and `ArrayList` were ONE type here, as were `Set`/`HashSet` and
`Map`/`HashMap` — they share every member, and the same heap object answers for
both. A program can tell them apart three ways, though, and caturra could not:

```java
List<String> face = new ArrayList<>();
ArrayList<String> back = face;   // javac: List<String> cannot be converted to ArrayList<String>
Set<String> keys = counts.keySet();
keys.clone();                    // javac: cannot find symbol — Set does not declare clone
Parent held = (Parent) anArrayList;   // javac: inconvertible; no class is both
```

All three compiled. The fix is the role `SortedRole` already gives the sorted
collections and `SeqRole` gives the queues, applied to the last three types
without one: a `CollFace` of `Iface` or `Concrete`, read from the name the
program wrote at the one place that still has it (`resolve_type`, just before
it normalizes `List` to `ArrayList`).

What the face decides:

- **Widening** — the class stands in for the interface and never the other way
  round, exactly as a `TreeSet` is a `NavigableSet`. `LinkedList`, `Stack`,
  `TreeSet`, `TreeMap` and `ArrayDeque` reach only the INTERFACE face, since
  none of them is an `ArrayList` or a `HashMap`. (Two copies of "a `TreeMap` is
  a `Map`" had to be collapsed for that to hold — the older one ignored the
  face, so it let a `TreeMap` through to a `HashMap` variable.)
- **Member lookup** — `clone` is the class's, not the interface's. It rides on
  the SAME `role` the sorted faces use, so nothing new had to be threaded
  through the lookup; a `Queue`/`Deque` face lost `clone` in the same move.
- **Casting** — the `cast_face` of a concrete collection is a class, so
  `(Parent) anArrayList` is now the error javac calls it, while
  `(Parent) aList` keeps compiling (some class really could be both).

Everything the library hands back is the INTERFACE, which is how javac declares
every one of them (`Arrays.asList`, `keySet`, `subList`, `List.of`,
`collect(toList())`, `Collections.unmodifiable*`). The one shape that makes a
class is `new`.

Two collections join at the interface they share, which is what `flag ? new
ArrayList<>() : new LinkedList<>()` means — with a face, neither branch widens
to the other, and without the rule the join fell all the way to `Object` and
the assignment after it was refused.

**The sweep.** The assignment cross-product ("What assigns to what") had built
every source as a fresh EXPRESSION, so a face — which only a declared variable
carries — was invisible to it: 1369 programs, and not one of them could see
this. Rebuilt through a variable (`S source = expr; T target = source;`) it
found the family at once, and now agrees on all 1369 verdicts. The cast and
`instanceof` cross-products went exact at the same time: their whole residue
(12 and 9 programs, plus 23 diagnostics naming `Set<String>` where javac says
`HashSet<String>`) was this one modelling limit.

**Diagnostics.** The same sweep compared 1256 rejection MESSAGES, where 463
differed. A diagnostic must name the type the program WROTE, and `describe` was
naming caturra's model of it:

- a nested class by its BINARY name (`Outer$Inner`), which had been fixed at
  individual sites before — it is fixed in `describe` now, where the other two
  hundred diagnostics get it too;
- a RAW collection as `List<Object>`, a parameterization javac reserves for the
  one that really is `Object`;
- a wildcard as its BOUND — `List<Number>` for a `List<? extends Number>`,
  which reads as a different type entirely, and one the refused assignment
  would have allowed.

Two neighbours the sweep turned up: `byte b = aDouble;` was reported as
"possible lossy conversion from double to byte" (the narrowing rule is about a
primitive the program HAS, and a wrapper is not one — javac says "Double cannot
be converted to byte"), and comparing a primitive with `null` was caturra's own
sentence rather than javac's headline plus `first type:`/`second type:` lines.

100 of the 1256 still differ, all one wording: javac spells a captured wildcard
`List<CAP#1>` (with a footnote naming the capture), where caturra prints the
wildcard the program wrote, `List<? extends Number>`. Both name the type; only
javac's names the capture.

Pinned by `diff_a_collection_wears_two_faces` and the twelve face, diagnostic
and operand rejections in `stage6_compile_errors_match_javac_wording`.

### Every operator against every type (2026-08-23)

Nineteen binary operators over seventeen types on each side — **5491 cells**,
each printing the value it produced *and the class it boxed to*, so the promoted
type is checked and not only the number. javac rejects 3671 of them; the other
1820 run.

**45 cells were a VerifyError.** Every shift with a `Long` operand, on either
side, killed the VM: `aLong >> 1`, `1 << Long.valueOf(2)`, `aLong <<= 2`. Each
side of a shift is promoted on its OWN (JLS §15.19) and a wrapper promotes by
UNBOXING first — which neither side did here: the left because a `long` shift
needs no conversion and so asked for none, the right because a `Long` count is
not a `long` and fell through the `L2I` case. The VM refused the whole method
(`expected a long on the stack, found Ref`) for an ordinary expression.

**4 cells compiled that javac refuses**: `"s" == anIntArray`. JLS §15.21.3 says
`==` between two references is legal exactly when a CASTING CONVERSION exists
between their types — the rule "Which casts exist" already answers for `(T) x`
and for `instanceof`. It was answered here twice more by hand, once as a table
of seven "scalar families" and once as a wrapper rule, and neither had heard of
an array. Both are deleted; the shared rule stands in their place, and the
enum-specific one stays beside it (an enum is implicitly final, which the class
table does not record).

**2787 of the 3671 rejections were worded differently.** javac has one shape for
operands an operator has no meaning for — the operator in the headline, the two
types on continuation lines — and caturra had invented three of its own
("operator '+' cannot be applied to int and boolean", "operator '&&' needs
boolean operands, got int", and the bare headline). Now one helper, used by all
eleven sites, and three details that the cross-product is what pins:

- a COMPOUND assignment names the BINARY operator: `x *= o` is "bad operand
  types for binary operator `*`", never `*=`;
- both operands are named, and named as the program WROTE them — `Boolean`, not
  the `boolean` it would have unboxed to (which meant reaching for the type
  before the promotion, and, in `&&`, for the right operand's type before it
  had been emitted);
- `==` has a SECOND shape, and javac picks between them by whether the operands
  are values: `int == boolean` and `Boolean == int` are "incomparable types: A
  and B" (two values of kinds with no common one), while `String == int` — a
  reference that does not unbox, against a primitive — is the operator's own
  complaint. A relational operator never has the first shape.

**All 5491 cells now agree**, verdict and wording, with four exceptions that are
the same deliberate difference: `"" + new Object()` prints caturra's
deterministic identity hash (`java.lang.Object@2`) where a JVM prints a random
one, which is what makes every other run reproducible.

Pinned by `diff_a_shift_with_a_wrapper_operand` and the four operator
rejections in `stage6_compile_errors_match_javac_wording`.

### Cannot find symbol (2026-08-24)

The commonest compile error in student Java, and caturra said it in its own
words on one line. javac says it in three:

```
cannot find symbol
  symbol:   method bark()
  location: variable p of type Pet
```

Twenty-five shapes of it — every kind of receiver against every kind of missing
member — and **24 of the 25 differed**. They agree now.

**The `location:` line is the work.** A receiver that NAMES a variable (a local,
a parameter, or a field) is `variable p of type Pet`; a class name, a literal,
an array element, or any other expression is `class Pet`; a bare `nope()` is
looked for in the enclosing class, which javac names — while `this.nope()`, the
same lookup written out, gets no location line at all, and javac drops the
padding after `symbol:` when it does. Only the two call dispatchers and the
field paths have the receiver EXPRESSION, and a missing member is reported from
twenty places reached by a dozen helpers, none of which is given more than a
type — which cannot say whether a variable was named. So the description is
computed where the expression is and read where the message is, with the
dispatchers restoring the outer one on the way out: `list.get(0).nope()` first
reported `variable list of type List<Pet>` for a method looked for on the
ELEMENT, because the nested call had established its own.

**The applicability messages** got javac's shape too: one candidate gets

```
method speak in class Pet cannot be applied to given types;
  required: no arguments
  found: int,int
  reason: actual and formal argument lists differ in length
```

and several get one line each, with the reason javac gives — including the
lossy/unrelated split it makes for a numeric argument, which the one-candidate
rule above already had. `Integer.valueOf()` used to say "no suitable method
found for valueOf() in class Integer", naming none of the three overloads it
does take, which for a library call is the only place the reader learns what it
DOES take.

Two divergences recorded in `REJECT_WORDING` closed as a result — a lone
mismatched candidate (`System.arraycopy` with four arguments) and the
one-candidate rule on `Math.multiplyFull` — and that table is what caught them,
by failing when caturra started agreeing with javac.

One thing still differs: the ORDER of the candidate list. javac lists overloads
in declaration order, and caturra's builtin tables are ordered for overload
SELECTION (`Math.scalb(float,int)` before `(double,int)`, so an `int` argument
picks the more specific one) — so the same set can come out in a different
order.

Pinned by `a_missing_symbol_reads_like_javac` (nineteen shapes) and
`reject_wording_tracks_javac`.

### What a conversion asks of its argument (2026-08-24)

Thirty format specifiers against sixteen argument types — 480 cells, each
printing the text or the exception. **29 differed**, in three ways that share
one cause and one that does not.

The formatter sees only the heap, so it cannot call a user `toString()`. Every
heap object among a format call's arguments was therefore rendered to text
BEFORE it ran. That is invisible while the conversion wants text, and wrong the
moment it does not:

- `String.format("%d", pet)` ran `Pet.toString()` — a method the JDK never
  calls for `%d`, visible the moment it throws or has a side effect — and then
  reported the mismatch against `java.lang.String`, a class the program never
  mentioned, instead of against `Pet`;
- `%h` hashed that TEXT rather than the object, so an overridden `hashCode()`
  was ignored and two distinct objects with equal text hashed alike.

The template is asked what it will want now (`argument_needs`), and only those
arguments are prepared: the text for `%s`, the `hashCode` for `%h` — each run
where a user method CAN run — and everything else is passed untouched, so the
formatter can name the object's class when it refuses it. The index rules that
answer "which argument does this conversion consume" (`%2$s`, `%<d`, and plain
order) are one piece of code shared with the render loop, because asking them a
second way is how the two would come to disagree.

The fourth: `%X` reported itself as `X`. The JDK's exception carries the
CONVERSION, and an uppercase specifier is the lowercase conversion with a flag
— `x != java.lang.String`, never `X`.

**And the class an anonymous class calls itself.** Java names one
`Enclosing$1`, numbered from one per enclosing class in source order —
`A1$1`, and `A1$Inner$1` for one written inside `Inner`. caturra named them all
`Anon$N` from a single counter, which is what `getClass().getName()`,
`getSimpleName()` (empty), `getCanonicalName()` (null), a default `toString()`,
a `ClassCastException` and a stack-trace frame all reported: a name no Java
program can show. The binary name is assigned where the enclosing class is
already known (the capture pass), and the VM tells an anonymous class apart the
way javac's own naming does — the last `$`-segment is digits, which no source
name can be. A LAMBDA is excluded: a JDK calls its class `Outer$$Lambda$1`, and
it is not an anonymous class.

479 of the 480 cells agree now; the last is `%h` of an object with no
`hashCode` override, where caturra's identity hash is deterministic and a JVM's
is not.

Pinned by `diff_a_format_conversion_asks_for_what_it_needs`.

### Math at its edges (2026-08-24)

Sixty-four programs over every `Math` method and every edge value it has —
**2131 cells**: NaN through each function, both zeros (told apart by dividing
into them, the only way a program can), both infinities, `MIN_VALUE` and
`MAX_VALUE`, the overflow of every `*Exact`, the sign rules of
`floorDiv`/`floorMod`, and what `round`/`rint`/`ceil`/`floor` do at a half and
below zero. Plus 1500 random doubles (bit patterns, not decimals) through
`Double.toString`, and 3200 in-range transcendental calls.

**The semantics are exact**, and are now pinned. Two families are not, and
neither is a bug to fix:

- **A NaN's raw bit pattern.** `Math.acos(2.5)`, `Math.log10(-1)` and
  `IEEEremainder(x, 0)` all answer NaN, and the JDK's is the negative quiet NaN
  (`0xFFF8…`) where caturra's is the canonical `0x7FF8…`. Only
  `doubleToRawLongBits` can see it — `doubleToLongBits` collapses every NaN to
  the canonical one, arithmetic and printing treat them alike, and the JDK's own
  answer depends on the hardware it runs on. 238 of the 2131 cells; unspecified,
  not wrong.
- **The last ulp of a transcendental.** 7 of 3200 in-range calls differ by
  exactly one ulp — `log10` 5 of 200, `cos` and `tan` 1 each, everything else
  (`sin`, `exp`, `log`, `atan`, `asin`, `acos`, `sqrt`, `cbrt`, `sinh`, `cosh`,
  `tanh`, `expm1`, `log1p`) 0 of 200. `Math` is specified to be within 1 ulp and
  semi-monotonic, so both answers are conformant Java.

  This one was scoped as a fix and **measured as a non-goal**. `Math.log` on
  JDK 11 is a HotSpot INTRINSIC, not fdlibm: on the same JDK, `Math` and
  `StrictMath` disagree on **45 of the same 3200 calls** — six times as often as
  caturra disagrees with `Math`. So the exact bits of `Math` are not a portable
  target at all (they are the host's hand-written stub, and differ between a
  JDK's own platforms), and porting fdlibm — the obvious way to become
  bit-exact — would make caturra reproduce `StrictMath` and therefore diverge
  from `Math` on those 45. A hand port of `__ieee754_log` was written and
  verified against the JDK to confirm exactly that: it matches `StrictMath` on
  both inputs where it "failed" against `Math`.

  The one thing worth having is not accuracy but AGREEMENT: caturra's own two
  builds differ on 56 of 3200 (up to 2 ulp), because each takes the transcendentals
  from its target's libm — so the browser differs from a JDK on 53 where the
  native build differs on 7. Closing that means one vendored implementation used
  by both, and the measurement above says fdlibm is the wrong one to vendor.

`Double.toString` is exact over all 1500, subnormals and `1e23` included — the
JDK 11 algorithm, which prints more digits than the shortest round-trip, is what
`floatdec.rs` reproduces.

Pinned by `diff_math_at_its_edges`.

### String at its edges (2026-08-24)

Twelve programs, **1049 cells**: every `String` method a course uses, against
every argument that makes it awkward — a negative index, one past the end, a
reversed `substring` range, an empty pattern, an invalid regex, a group
reference with no group, a `fromIndex` outside the string — over text that is
itself awkward: a surrogate pair, the Turkish dotted and dotless i, a sharp s, a
non-breaking space, an en quad, and a CRLF. Each cell prints the value or the
exception's CLASS and MESSAGE, so the failures are compared as closely as the
successes.

**All 1049 agree**, Java 11's own additions included (`isBlank`, `lines`,
`repeat`, `strip`/`stripLeading`/`stripTrailing`). `String.indent` is Java 12
and javac 11 refuses it, as caturra does.

Nothing to fix, which is the result. The sweep is worth recording for what it
took to run: two of its three "findings" were the harness translating newlines,
not the engine. Python's text mode rewrites `\r\n` to `\n` on read AND on
write, so a JDK captured with `text=True`, or a caturra output round-tripped
through a text-mode file, silently loses the carriage return that
`"a\nb\r\nc"` is there to test. Compare bytes.

Pinned by `diff_string_at_its_edges`.

### Programs nobody wrote (2026-08-24)

Every sweep above asks about a surface someone thought of. This one does not: a
generator composes arithmetic, control flow, mutation, exceptions and calls at
random, and a real JDK decides what the answer is.

**3200 random programs, 0 divergences** — 2000 from a first generator and 1200
from the one that now lives in the test suite. Every program javac accepted
(the generator only emits well-typed, definitely-assigned code), and every
program printed the same bytes on both engines.

What it composes: arithmetic with every promotion; compound assignment,
including the implicit narrowing cast `x += aDouble` hides; `++`/`--` in prefix
and postfix on each of the three storage kinds (a local, an array element, a
static field), since each compiles its own read-modify-write; `for`, `while`,
`do`/`while` and for-each; labelled `break` and `continue` out of nested loops;
`if`/`else`; `switch` on `int` and on `String`, with and without fallthrough;
`try`/`catch` over four kinds of thrower; calls into a recursion with a budget,
a `finally` that overrides a `return`, and a throw two frames down; `Integer`
identity either side of the cache boundary; narrowing casts to `byte`, `short`,
`char` and `float`; arrays, `ArrayList`, `StringBuilder` and `String`.

Everything it emits is DETERMINISTIC — bounded loops, a recursion budget, no
hashing order, no transcendentals, no identity hashes — because a difference has
to mean a difference rather than a coin landing differently in two engines. The
awkward part was not the grammar but the SCOPING: a generator that forgets a
block's declarations die with it writes a use of a name Java says is gone, and
one that lets a loop body assign its own counter writes a program that never
ends. Both were found by javac refusing the output, which is the generator's
own check.

`cargo test` fuzzes 40 programs; `CATURRA_FUZZ=2000` is the longer hunt. A
failure prints the seed and the program, so anything CI finds is one case here.

**A second generator writes class HIERARCHIES**, where the first wrote method
bodies — three classes and an interface, with a field the subclass may hide, a
`tag()` every class overrides, a `describe()` some do, an overload only the
subclass declares, initializer blocks, constructors that may or may not write
`super()`, and an interface default that may or may not be there. `main` then
holds each instance at each of its static types and prints what a method call,
a field read and an overload choice each answer.

**The first hundred of those found a real bug**, and a bad one: `super.greet()`
in a class whose chain implements an interface that DEFAULTS `greet` aborted the
whole program with "malformed class C: no method greet()". The JVM resolves an
`invokespecial` by searching the superclasses and then their SUPERINTERFACES
(JVMS §5.4.3.3); caturra's walked classes only. Every ingredient is ordinary
Java — it takes all three at once (a default, a chain that does not override it,
and a subclass calling `super`), which is exactly the shape a written test
misses and a random hierarchy does not. 820 hierarchies agree now.

**A third generator writes GENERIC hierarchies**: an interface parameterized on
`T` with an optional default, a `Box<T>` with a field and methods of that type,
a subclass that PINS the argument (`Pin extends Box<Integer>`) and overrides
some of them, a class implementing the interface at a fixed argument, an enum
implementing it too (with or without constant bodies, which are anonymous
subclasses), and a lambda and an anonymous class beside them.

**Its first hundred found two more bugs, both of them ordinary Java:**

- A subclass that pins the argument inherits `T value` as an `Integer`, and the
  DECLARED type is the erasure — which is what a read answered. So `Integer
  get() { return value; }` was refused ("Object cannot be converted to
  Integer"), and `value + 1` was "bad operand types: Object and int". The
  emitter and `type_of` each needed telling, which is the mirror this codebase
  keeps rediscovering: fixing the first left the second to reject an expression
  the first had already accepted.
- An override of a generic interface's DEFAULT needs a bridge like any other.
  `Named implements Sink<String>` declaring `twice(String)` overrides
  `twice(T)`, whose erasure takes an `Object`; the bridge pass walked the
  extends chain only, on the theory that an interface's methods erase like the
  class's — true only while the interface is not generic. The call through
  `Sink<String>` found no `twice(Object)` on the class and ran the interface's
  DEFAULT instead: a silent wrong answer, where the override simply did not
  happen. Only a defaulted method could show it; an abstract one had nowhere
  else to go.

**Random OVERLOAD SETS against random arguments** — two to four overloads drawn
from thirteen parameter shapes, called with one of twelve arguments, one program
per pair so the verdict is per-cell — found two more, both about the PHASES
(JLS §15.12.2):

- A primitive reaching ANY reference is a boxing conversion, so none of them
  belongs in phase one. Only the wrapper itself was excluded, so `int` to
  `Number` stayed phase-ONE applicable while `int` to `Integer` was phase two:
  the WIDER overload won an earlier phase, and `f(1)` against `f(Integer)` and
  `f(Number)` chose `f(Number)`.
- An `Integer` is a `Comparable<Integer>`, never a `Comparable<String>`.
  Applicability ignored the type argument, so `g(Comparable<String>)` was
  selected for an `Integer` and the call then REFUSED — where javac passes over
  that candidate and picks another. "Which casts exist" already knew this;
  applicability did not, and one fact in two places is how they disagreed.

Nested, inner, local and anonymous classes were fuzzed too — two levels of
enclosing instance, `Outer.this` against `Inner.this`, a local class shadowing
an outer field, an anonymous subclass of a user class with a constructor
argument — and 500 of those agree without a fix, which is the sorted-out state
"Nested-class scoping" left them in.

Pinned by `fuzz::random_programs_run_the_same_as_the_jdk`,
`fuzz::random_hierarchies_dispatch_like_the_jdk`,
`fuzz::random_generics_erase_like_the_jdk`,
`diff_super_reaches_an_inherited_default`,
`diff_a_parameterized_supertype_substitutes` and
`diff_overload_phases_and_specificity`.

### A user exception is a throwable too (2026-08-25)

Random USER exception hierarchies — a `RuntimeException` subclass and a subclass
of that, a checked one, catch clauses ordered so a subclass never follows its
superclass, multi-catch, causes, rethrows, and a resource whose `close()` may
throw — found that **suppression stopped at the library's own throwables**.

A library throwable carries its suppressed list on its own heap object. A user
one — `class AppException extends RuntimeException`, as ordinary as a course
exercise gets — had nowhere to put it, so `addSuppressed`, which the
try-with-resources desugaring CALLS whenever a resource's `close()` throws while
the body is already throwing, aborted the program: "unknown native member". The
instance layout reserves a `__suppressed` slot now, beside the `__message` and
`__cause` it already reserved.

Fixing that uncovered the next two, each hidden by the one before it:

- `getSuppressed()` answered an EMPTY array by design ("none are modelled"),
  which was true until the slot existed. So the try-with-resources reported the
  body's exception with nothing under it, where a JDK lists what closing threw.
- And the fallback that reaches the intrinsic layer treated an intrinsic that
  THREW as one that did not exist: `addSuppressed(this)` is the JDK's
  `IllegalArgumentException: Self-suppression not permitted`, and it came back
  as "not implemented". Only a MISSING intrinsic may fall through — one that
  threw is answering the call.

500 exception programs agree now, as do the 96 of the try-with-resources sweep.

Two neighbouring generators found nothing to fix, which is worth recording:
**class INITIALIZATION order** (static blocks, field initializers, constructors,
and what triggers them — a constant read does not, a `new` does, an inherited
static read initializes only the DECLARING class, a failing `<clinit>` gives
`ExceptionInInitializerError` and then `NoClassDefFoundError`, two classes whose
initializers read each other see defaults) over 120 programs, and **nested,
inner, local and anonymous classes** over 500.

Pinned by `diff_a_user_exception_can_be_suppressed_into`.

### Every traversal is one order (2026-08-25)

Random operation SEQUENCES over the collections — a list, a deque, a queue, a
hash set, a sorted set and two maps, mutated eight to sixteen times each and
then walked, viewed, sorted, streamed and iterated — found the four ways to
traverse a hash collection disagreeing with each other.

`toString`, a for-each and a stream all ask for the ORDERED position: the JDK's
bucket order, which caturra reproduces exactly. An explicit `iterator()` indexed
the STORAGE directly and walked insertion order instead. So one set printed
`[0, 7, 8]` and iterated `7 0 8`, and a `while (it.hasNext())` loop disagreed
with the for-each beside it over the very same collection. The same seam ran
through `keySet()`, `values()` and `entrySet()` cursors.

Only a history that separates the two orders shows it — a `removeIf` and an
`addAll` leave the storage in an order the buckets do not agree with — which is
why composing operation sequences found it where building a set and walking it
would not.

**One corner is left, and is deliberate.** An `ArrayDeque`'s iterator is
fail-fast *on a best-effort basis*, and the JDK's check depends on where `head`
and `tail` sit in its circular array: `addFirst` during an iteration does not
disturb them, so a JDK returns the element the cursor was already on, while
caturra — whose deque is a vector — reports the modification. caturra is the
STRICTER of the two here (it never misses one the JDK catches), and the JDK's
own documentation says this detection "cannot be guaranteed" and exists only to
find bugs. Modelling head/tail positions to reproduce the gap would be a
structural change in aid of an unspecified answer.

Pinned by `diff_every_traversal_is_one_order`.

### A barrier that still waits (2026-08-25)

Fuzzing random stream pipelines — sources, intermediate ops, terminals and
comparator combinators composed at random — found `sorted` running EAGERLY, at
the call that appends it. Everything else in the pipeline was lazy, so this one
op was enough to make a `peek` above it print before any terminal existed:

```java
Stream<String> s = Stream.of("b", "a").peek(x -> System.out.println(x)).sorted();
```

A JDK prints nothing there — no terminal has asked for an element. caturra
printed both.

`sorted` is a stateful BARRIER: it emits nothing until the upstream runs dry.
That is not the same as being eager, and modelling it as a `StreamOp` that
BUFFERS — flushed, sorted, into the ops below it once the source is exhausted —
is what separates the two. A second barrier downstream is flushed by the same
walk, since the first one's flush fills it.

The larger half is what a lazy barrier makes possible: `count()` may answer
without traversing at all. The JDK's does whenever the source size is known and
no operation can change it, and `Stream.count`'s javadoc says so outright — so
the side effects of a `map`, a `peek` or a comparator in such a pipeline never
happen. Sorting cannot change how MANY elements there are, so `sorted` belongs
in that set:

```java
Stream.of("c", "a", "b").peek(x -> System.out.println(x)).sorted().count();  // prints nothing, answers 3
Stream.of("c", "a").sorted((x, y) -> { throw new IllegalStateException(); }).count();  // answers 2
```

A `filter` or a `distinct` anywhere in the chain makes it traverse as usual.

Twenty programs — no terminal at all, short-circuit terminals below the
barrier, a `limit` above it, two barriers, a throwing comparator, comodification
through the barrier, a primitive pipeline, an infinite source bounded above it —
agree with a real JDK exactly. Pinned by `diff_a_sorted_barrier_stays_lazy`.

### What a library object says it is (2026-08-25)

The same sweep asked, of every library object caturra can make, what
`getClass()` prints and what `instanceof` accepts. The two answers came from
different places: one namer, and a dozen hand-written lists in the
`instanceof`/`checkcast` arm. Whatever was missing from the lists answered
FALSE — and a false `instanceof` is followed by a `ClassCastException` on the
cast:

```java
Object o = Arrays.asList("p");
System.out.println(o instanceof List);   // JDK: true.  caturra: false
List<String> back = (List<String>) o;    // and then this threw
```

`Arrays.asList`'s list, a `subList`, a map's `keySet`/`values`/`entrySet`, a
`TreeSet` range view and a `TreeMap` sub-map were all in that gap — ordinary
collections a program holds through an `Object` for a moment.

Both answers now come from ONE fact: the class `object_class_name` gives, and a
table of that class's supertypes keyed by it. Two things fell out of writing it
down that way. A view had no class name of its own, so naming one was the first
half of the fix — a sub-list is `ArrayList$SubList`, `AbstractList$SubList` or
`AbstractList$RandomAccessSubList` depending on which list it views, and a
`TreeSet`'s range view IS a `TreeSet`, which is why it answers `Cloneable` too.
And the bundled interfaces (`Comparable`, `Cloneable`, the closeables) reach the
VM under their SIMPLE name, since that is how caturra synthesizes them, so a
bare spelling is qualified on the way into the table — without that an
`ArrayList` said it was not `Cloneable`.

`Iterable` and `Iterator` stay out of the table on purpose: they cut across
object KINDS rather than classes, and a separate cross-cut answers them (a
sorted view of a map is not one, though a view of its keys is).

Forty-four objects × twelve faces, against a real JDK. Pinned by
`diff_a_library_object_wears_its_own_class`.

### The element and its source (2026-08-25)

A stream lambda's parameter type is read SYNTACTICALLY from the receiver, so
each new way of writing a source needs the recogniser to learn it — and until
it does, the element is `Object` and the next lambda in the chain is refused
with "cannot find symbol". The pipeline fuzz found four shapes at once:

- `Stream.concat(a, b)` — the element is either half's.
- `flatMap(s -> Stream.of(s, s.substring(0, 1)))` — the element is that of the
  stream the lambda ANSWERS, one step further in. The lambda's parameter is not
  in scope for the ordinary walk (it exists only inside the synthesized class),
  so the two shapes that mention it — a `Stream.of` over it, and a
  `stream()`/`Arrays.stream` of it — are read against the lambda's own bindings.
- `flatMap(List::stream)` — an unbound method reference whose qualifier is a
  library CONTAINER. Those names were not class names to the reference
  desugarer, so it compiled the bound form, `List.stream(p0)`: a static call on
  an interface. They are deliberately left untyped as a receiver, since the
  functional interface supplies the element type WITH its type arguments where
  the bare name would be the raw type.
- `Stream.of(first, second)` over declared variables — the literal reader saw
  literals and `new`, so two `String` locals made a stream of `Object`. Each
  argument is now typed the way a lambda body is.

`String.split` and `toCharArray` joined the small table of library returns while
this was written: `Arrays.stream(s.split(","))` needs an element too.

One shape is knowingly left: a `Stream.of` whose arguments are library FACTORY
calls (`Stream.of(Arrays.asList(1, 2), ...)`) still has an `Object` element, so
a `List::stream` after it is refused. It fails in the safe direction — a
refusal, not a wrong answer.

Pinned by `diff_a_stream_element_survives_its_source`.

### Where an abrupt exit really goes (2026-08-25)

Fuzzing random CONTROL FLOW — loops, labels, switches, try/catch/finally and
try-with-resources nested into each other, with a `return`, `break`, `continue`
or `throw` allowed wherever one is legal — found three defects in the same
seam: what happens to a `return` while the finallys between it and the method
exit run.

**The return value was parked on the operand stack.** A handler entered
anywhere inside a finally copy clears the stack to just its exception, so a
`finally` that caught anything at all — even its own throw, handled entirely
within itself — took the parked value with it, and the `ireturn` that followed
underflowed:

```java
static int m(int a) {
    try {
        return a + 1;
    } finally {
        try { throw new IllegalArgumentException("x"); } catch (RuntimeException e) { }
    }
}
```

That is `operand stack underflow (malformed bytecode)` out of ordinary Java —
an abort, the failure mode worse than a wrong answer. The value goes in a local
now, which is where javac has always put it.

**A `continue` inside a finally continued the wrong loop.** The copy of a
finally that a `return` drags along is emitted where the return is, inside
whatever loops surround THAT — but a `break` or `continue` written in a finally
belongs to the loops around its own `try`. The loops opened since are hidden
while the copy is emitted, so:

```java
do {
    try {
        while (w < 3) {
            for (int i = 0; i < 1; i++) { return a; }
            ...
        }
    } finally {
        if (first) { continue; }   // the DO, not the while
    }
} while (...);
```

discards the pending return and starts the next `do` iteration, where caturra
had resumed in the middle of the `while` it had already left.

**And the same program has to compile.** A `do` whose body only ever returns or
continues still completes normally: JLS §14.21 makes a do's CONDITION reachable
when the loop contains a reachable `continue`, so what follows the loop is
reachable too. caturra called it "unreachable statement" and refused a program
javac accepts. The rule needed the mirror of `has_escaping_break` — with the
difference that a nested switch captures an unlabeled `break` but not an
unlabeled `continue`, which passes straight through it.

420 generated programs, three shapes of nesting, agree with a real JDK
exactly — apart from the ones caturra now says are too big, below. Pinned by
`diff_a_return_waits_for_a_catching_finally` and
`diff_a_finally_continues_the_right_loop`.

**A fourth defect the same fuzz found: the compiler PANICKED** on a method
whose bytecode outgrew a 16-bit branch offset (`branch offset exceeds 16 bits`).
It reports javac's own `code too large` now — see the divergence list below,
since caturra's limit is reached sooner than javac's.

### What a format string really says (2026-08-26)

Random format strings — every conversion against every argument type, with
random flags, widths and precisions, and the mismatches in `try`/`catch` so the
EXCEPTION is compared too — agreed with a real JDK on 150 programs except for
one conversion, and then a second batch with argument indices, `%<`, `printf`
and deliberate arity errors found three more facts.

**`%a` ignored its precision.** A hexadecimal float rounds in BINARY:
`%.2a` of `1e23` is `0x1.53p76`, which no truncation of
`0x1.52d02c7e14af6p76` produces. The JDK rounds the significand half-even to
`1 + 4 × precision` bits (scaling a subnormal up by 2^54 so it HAS that many,
and answering `0x1.0p1024` when `MAX_VALUE` rounds out of range), lets
`Double.toHexString` render the result, and pads the fraction out with zeros.
`%+a` and `% a` did not sign it either, and a zero-pad went in front of the
`0x` rather than between it and the digits. Two JDK quirks are now reproduced
deliberately, both visible in the width: **the sign does not count** toward it
(so `%012.3a` of a negative is thirteen characters), and neither do the
trailing zeros the precision adds (so `%012.3a` of `0.0` is fourteen).

**A JDK parses the whole template before it renders any of it.** Every
`FormatSpecifier` validates its own flags as it is constructed, so
`printf("%c %-o", -1, 7)` reports the `%-o` that has no width — not the illegal
code point in the specifier BEFORE it, which is what caturra said, and nothing
is written to the stream first.

**And it reports a specifier as it REBUILDS it**, not as the program wrote it:
flags first, in their own canonical order, and only then the argument index. So
`%3$,.2f` comes back as `%,3$.2f` and `%2$012.4f` as `%02$12.4f`.

**The argument's class was lost twice over.** `String.format("%d", aList)` said
its argument was a `java.lang.String`: the compiler coerced a statically-typed
collection to text at the call site (the arm beside the one that had already
been fixed for `Object`), and the VM's pre-render — which exists because only
the interpreter can run a user `toString` — replaced the argument with its text.
The text is recorded BESIDE the object now, so `%s` still prints it and `%d`
still names `java.util.ArrayList`. The class itself comes from the same namer
`getClass()` uses, which is the third place that fact lives and the one that
had been answering `java.lang.Object` for every collection.

**`%t` is a known conversion again.** Its suffix is parsed (`%tw` is
`Conversion = 'tw'`, `%,tY` is a flags mismatch naming `Y`), and every argument
type a JDK rejects is rejected with a JDK's own message. The one type it
accepts — a `long` of milliseconds — is refused honestly instead: caturra has
no `Date`, `Calendar` or `java.time`, and a calendar reading would need the
default time zone, which in a browser is the reader's.

### The frames a trace really has (2026-08-26)

Generated call graphs that end in a throw — through lambdas, method
references, anonymous classes, constructors, nested classes, recursion,
`finally`, a wrapped cause and a stream — compared frame by frame with a real
JDK. Two defects, both in what a student sees the moment a program crashes.

**A lambda's frame named caturra's own machinery.** javac compiles a lambda to
a synthetic METHOD on the enclosing class, so its frame is
`Frames.lambda$main$0`; caturra printed `Lambda$1.run`. Reproducing the name
means reproducing javac's numbering, which is per class, in SOURCE order of the
members — a lambda in a static field initializer is `lambda$static$0`, one in a
constructor or an instance initializer is `lambda$new$…` — and, where one
lambda contains another, INNERMOST FIRST, because javac numbers as each
translation finishes. The synthesized class carries that name in a class-file
attribute now, so the VM can write the frame without knowing anything about
lambdas.

**And a method reference had a frame that does not exist.** javac's
`invokedynamic` calls the target directly, so a trace goes straight from the
target to whoever ran the functional interface; caturra's synthesized forwarder
sat in between. The same attribute hides it.

**The other defect was not about names at all.** An exception thrown inside
native-driven user code — a stream op, a comparator, a `forEach`, a map's
compute function — is caught one level out, past the native frame. The handler
search takes the thrown OBJECT off its register while looking, and when nothing
in the nested run handled it, it never put it back: the real catch, one level
out, re-materialized a copy from the error TEXT. Same class, same message, and
nothing else — no cause, no suppressed list, none of a user subclass's fields,
and a different identity:

```java
try {
    Stream.of(1).forEach(x -> { throw new IllegalStateException("s", new IOException("c")); });
} catch (RuntimeException e) {
    e.getCause();   // the JDK's IOException; caturra's null
}
```

120 generated programs agree with a real JDK exactly, comparing the frames a
program can see. **What is left is deliberate:** a JDK's trace also contains
frames INSIDE the JDK (`java.base/java.util.Spliterators$ArraySpliterator.forEachRemaining`,
`jdk.internal.util.Preconditions.outOfBounds`), and caturra's library is native
— it has no Java frames to show, and inventing them would be inventing line
numbers in a source file that does not exist here. Those frames also differ
between JDK versions, which is why the comparison filters them rather than
chasing them.

Pinned by `diff_a_trace_names_a_lambda_as_javac_does` and
`diff_a_thrown_object_survives_a_native_frame`.

### Two ways an array API was half there (2026-08-26)

Random array programs — every element type, one and two dimensions, built by
`new` and by initializer, then printed, sorted, filled, copied, searched,
cloned, hashed, streamed and indexed out of bounds — found two gaps and one
misleading message.

**`Arrays.compare` was missing entirely.** It is Java 9's, so a Java 11 program
may write it, and every element type needs its own overload. The order is
lexicographic, with two details worth stating: when one array is a PREFIX of
the other the shorter is smaller by the LENGTH DIFFERENCE (the JDK returns
`a.length - b.length`, so the magnitude is that difference and not just a
sign), and each pair is compared the way the wrapper's own `compare` does — so
`NaN` is greater than everything and `-0.0` is less than `0.0`, the same total
order `sort` imposes. A null ARRAY sorts before a non-null one, and so does a
null ELEMENT, where the `compareTo` chain a naive implementation would write
throws instead.

**`Arrays.setAll` had two of its four overloads.** `int[]` and `T[]` worked;
`long[]` threw `NullPointerException` for want of an arm in the VM, and
`char[]`, `short[]`, `byte[]`, `float[]` and `boolean[]` — which javac REJECTS,
since the JDK's four overloads are `int[]`, `long[]`, `double[]` and a generic
`T[]` — were accepted here and filled nothing in. Both directions are fixed:
the `long[]` fills, the rest are rejected the way javac rejects them.

**And the message that rejects them had to name the lambda.** A synthesized
lambda class has no source name, and splitting its binary name the way a
nested class's is split left the counter: `setAll(char[],1)` for an argument
the program wrote as `i -> 'a'`. It reads `lambda expression` now — javac
prints the lambda's own text, which is gone by the time this pass runs.

210 generated programs agree with a real JDK. Two things stay out of the
comparison, both deliberately: the identity hash in a default `toString`
(`[I@3764951d`), which is unspecified and differs between runs of the same
JDK, and `Arrays.stream(array, from, to)`, whose refusal is enumerated in the
divergence list — a stream over an array is late-binding, and caturra records a
stream's origin as a whole array with no room for a range.

Pinned by `diff_arrays_compare_and_fill_by_index` and
`reject_set_all_takes_no_char_array`.

### What an enum's constants are (2026-08-26)

Generated enum programs — plain constants, constants with fields, constants
with BODIES, an enum implementing an interface — put through `values`,
`valueOf`, `ordinal`, `name`, `toString`, `compareTo`, a `switch`, a set, a
map, a sort, a stream and an array write. Two defects, and the second is not
about enums at all.

**`Kind.values()` had no type.** An enum's two synthetic statics have no
receiver VALUE to read a type from, and the pass that types a stream's element
reads types off expressions — so a stream over `Kind.values()` refused the
lambda after it OUTRIGHT ("a lambda or method reference is only allowed where a
functional-interface type is expected"), while the same array in a VARIABLE one
line up worked. Every other source — `Arrays.asList`, `Stream.of`, a copy
constructor — gave the lambda an `Object` instead. `values()` is a `Kind[]` and
`valueOf(String)` a `Kind` now, wherever an expression's type is read.

**And a lone reference array is the varargs array.** `List.of(Kind.values())`
is a list of the constants; caturra built a list holding ONE array, which threw
`ClassCastException` at the first use of an element. The rule was already
written down twice — the stream factories spread a lone reference array, and
`Arrays.asList` does — and the immutable factories did not, on either side:
neither the element TYPING nor the EMIT. `List.of(someStringArray)` had a size
of one.

`Enum::name` and `Enum::ordinal` work in a `map` now (the functional interface
supplies the element), and 180 generated programs agree with a real JDK.

**Two things are knowingly left.** `Comparator.comparing(Enum::name)` is still
refused — a key extractor's parameter is typed as the qualifier CLASS, and
`Enum` is not a class caturra models, so the reference has nowhere to get its
element; the lambda spelling (`k -> k.name()`) and a reference through the
enum's own name both work. And `EnumSet`/`EnumMap` remain refused by name, as
the divergence list says — a `TreeSet`/`TreeMap` keyed by the enum iterates in
the very same order, since an enum's natural ordering is its ordinal.

Pinned by `diff_an_enums_constants_keep_their_type`.

### Three corners of the String surface (2026-08-26)

Random SEQUENCES of `String` and `StringBuilder` calls — every method against
edge arguments, indices from -1 to past the end, unicode, surrogate pairs and
regex specials, with the exceptions caught so their class and message are
compared too — found three defects that two hand-written sweeps of the same
surface had not.

**`intern()` adds the RECEIVER to the pool.** A JDK pools the string it is
given when that text has never been pooled, so
`String.valueOf(42).intern() == String.valueOf(42)`'s receiver is true. caturra
allocated a canonical copy instead, which made the identity false for every
string a program BUILT rather than wrote — and the pool was consulted by
scanning the heap for equal text, so an unrelated earlier string could be
handed back as "the canonical one".

**`String.join(delimiter, iterable)` reads any collection.** It falls back to
the general element reader for anything that is not a plain list, and that
reader had no arm for an unmodifiable LIST — so `String.join("-", List.of("a"))`
threw `NullPointerException` where the same call over an `ArrayList` worked.

**And a lambda body could not type a library STATIC.** The table beside it is
keyed by the receiver's TYPE, which a static call has no value to give: its
receiver is a class name. So `mapToObj(c -> String.valueOf(c))` produced an
`Object` element and the `String::concat` after it had no method to resolve.
The common statics — every wrapper's `toString`/`parse`/`compare`,
`Character`'s classification methods, `Math`, `Objects`, `String.valueOf` and
friends — answer their own types now; `Math.abs`/`max`/`min`/`round` stay
unknown on purpose, since they answer the ARGUMENT's width.

A fourth came out of the same batch: **`String.CASE_INSENSITIVE_ORDER` needs no
import.** Its type is a `Comparator<String>` the program never spells, so a
program that names `java.util` nowhere still needs the bundled comparator
interface — without it the constant's type was unknown and calling `compare` on
it was refused.

210 generated programs agree with a real JDK.

Pinned by `diff_string_identity_joining_and_statics` and
`diff_the_case_insensitive_order_needs_no_import`.

### Hashing before comparing, and where a failed scan stops (2026-08-26)

Two fuzzes in one round. The first put random USER CLASSES — with and without
`equals`, `hashCode`, `compareTo` — through the collections that consult them:
`contains`, `indexOf`, `remove`, `sort`, a `HashSet`, a `HashMap`, a `TreeSet`,
`binarySearch`, `distinct`. 200 programs, one defect, and it is the one a
lesson is built around.

**`distinct()` is a `HashSet`, not an `equals` scan.** A class with `equals`
and no `hashCode` gives every instance a different hash, so a JDK's
`distinct()` removes NOTHING — the duplicates land in different buckets and are
never compared. caturra compared with `equals` alone and removed them, which is
the answer the student EXPECTED and not the one Java gives. Every collection
beside it already hashed first; the stream was the odd one out.

The second fuzz drove random `Scanner` programs against random input —
`nextInt`, `next`, `nextLine`, `nextDouble`, `hasNext*` in random order over
tokens, blank lines and input that runs out mid-read. Two defects, both about
where the cursor STOPS when a read fails.

**A failed `nextInt()` leaves the cursor at the offending TOKEN.** A JDK "will
not pass the token that caused the exception" — but it does not put back the
delimiters it skipped on the way there either. So after a mismatch on
`   x y   `, a JDK's `nextLine()` answers `x y   ` and caturra's answered
`   x y   `, leading spaces and all.

**And a read that runs out of input leaves the cursor at the END.** The skipped
whitespace stays skipped, so the `hasNextLine()` after a failed `next()` is
false — where caturra, which left the cursor untouched, still saw the trailing
newline as a line of its own and answered an empty one.

240 generated programs across the two dimensions agree with a real JDK.

Pinned by `diff_distinct_hashes_before_it_compares` and
`diff_a_failed_scan_leaves_the_cursor_where_a_jdk_leaves_it`.

### A type argument a subclass fixed (2026-08-26)

Generated class hierarchies — an interface with a default and a static, an
abstract base, two levels of subclass, field shadowing, `super` calls,
anonymous implementations, polymorphic lists — agreed with a real JDK on 140
programs at the first run. The hand-written half that followed, aimed at the
corners a generator does not reach, found two generics defects.

**A subclass that FIXES a supertype's type argument.** `class IntBox extends
Box<Integer>` reads what it inherits as `Integer`: `get()` through a variable,
`get()` inside its own body, and `get()` after widening back to
`Box<Integer>`. caturra answered the erased type in the first two — the
receiver carries no arguments THERE, because the subclass wrote them on its
`extends` clause — so `new IntBox(5).get() + 1` was "bad operand types" for a
program that runs. The walk up the chain supplies them now, in the emit path,
the implicit-`this` path and `type_of` alike; the three had to be fixed
together, because a call typed one way and emitted another is the
`type_of`-versus-emit divergence this codebase keeps re-learning.

**A wildcard bounded by a type VARIABLE.** `<T extends Comparable<T>> T
max(List<? extends T> items)` — the shape every "write a generic max" exercise
uses — could not assign `items.get(0)` to a `T`: the parser records a
wildcard's bound as written, so `? extends T` arrived naming `T`, which is no
class, and the element read as `Object`. It takes T's own erasure now.

**One shape was left for the next entry**, and is closed there: a class type
parameter with a BOUND (`class Box<T extends Comparable<T>>`) erased to that
bound and lost its POSITION, so a `Box<String>` did not read `get()` as a
`String` where the unbounded `class Box<T>` did.

Pinned by `diff_a_fixed_type_argument_is_inherited`.

### A bounded type parameter keeps its position (2026-08-26)

The shape the entry above left open. `class Box<T extends Comparable<T>>` has
to do two things at once: inside the class, `value.compareTo(other)` must
resolve — that is what the bound buys — and outside it, a `Box<String>` must
read `get()` as a `String`. caturra could do either but not both: a bounded
parameter erased to its BOUND, which resolved the methods and lost the
position, so `new SortedBag<String>().smallest().toUpperCase()` was "cannot
find symbol" on a `Comparable`, and every generic class written the way the
exercises write them ran into it.

The two facts live in different places now. The POSITION is what the parameter
erases to, bounded or not — one rule for all of them, where there had been two
— and the BOUND is recorded on the class, which is where a type variable's
methods are looked up from: a call on a type-variable receiver resolves against
its bound, in the emit path and in `type_of` alike.

**A method-level bound whose name is not a class is still flat.**
`<T extends CharSequence> int longest(List<T>)` reads its elements as `Object`,
because `CharSequence` is a TYPE here and not a class in the table — where
`Number` and `Comparable` are. `<T extends Number>` and
`<T extends Comparable<T>>`, which is what an exercise asks for, both work.

Pinned by `diff_a_bounded_type_parameter_keeps_its_position`.

### What a mixed program crosses (2026-08-26)

Every dimension fuzzed so far has been one dimension. This one composes them:
a program with an interface and a default method, a `Comparable` class with
`equals`/`hashCode`, a bounded generic container, an enum with a method, and
then a random handful of statements that use all of it together — streams over
the collection, a map keyed by the enum, an iterator removal, a lambda that
throws, a cast that fails, an unboxed null. 220 programs, three defects, and
each one is a PAIRING that the single-feature sweeps had no way to reach.

**A method reference converted by a stream op was not marked as one.** Every
other conversion site marks the synthesized class as a method-reference class;
the stream ops (`map(Item::score)`, `mapToInt`, `forEach`) did not, so it was a
LAMBDA class — which meant it took a number in javac's per-class lambda
sequence and pushed every later lambda's frame name one too high, and grew a
stack-trace frame a JDK does not have. The mark now happens where the
conversion does, which is the one place that always knows.

**`Collectors` named through its package.** `collect(java.util.stream.Collectors
.toList())` typed as a null collection where the one-segment spelling typed as
a list, so `String.join("+", …)` took the whole list for a single element and
printed `null` — a silent wrong answer, and only for a program that writes the
qualified name.

**The one-argument `reduce`.** It answers an `Optional` of the stream's element
(the two-argument form answers the element itself), and nothing knew that: the
`map` after it was refused for having no functional-interface position. Through
a declared `Optional<Item>` variable the same chain worked, which is the tell
this codebase now recognises on sight.

Pinned by `diff_a_mixed_pipeline_keeps_its_types`.

### What a lab program crosses (2026-08-26)

A second mixed fuzz, shaped like the programs the corpus is full of: a custom
exception, an `Iterable` over the rows of a 2-D array, an inner class, a
memoized recursion, a try-with-resources. Four defects, none of which a
single-feature sweep could have reached.

**A 2-D array handed to a varargs factory spreads into its ROWS.** A `int[][]`
IS a reference array — its elements are `int[]` — so `Arrays.asList(grid)` is a
`List<int[]>`. caturra read only the one-dimensional case, so the whole array
became a single element, and the assignment that followed failed with a message
naming the SAME type on both sides ("`List<Object>` cannot be converted to
`List<Object>`") — the tell that two elements differ in something the printer
does not show. The rule is one function now, shared by `asList`, the immutable
factories and the stream sources; the varargs GOTCHA (a lone one-dimensional
PRIMITIVE array is one element) is the caller's rule, spelled out where it
applies.

**A `throws` on a NESTED class's method was invisible.** The table is keyed by
the BINARY name and the lookup had the name the source wrote, so
`static class Store { void save() throws IOException }` declared nothing as far
as the analysis could see: a legal `catch (IOException e)` was "never thrown in
body of corresponding try statement", and a caller that failed to declare it
compiled — both directions of the same missing lookup.

**A `Writer`'s methods declare `IOException`.** `write`, `close`, `flush` and
`append` all do, which is what makes `try (FileWriter w = …) … catch
(IOException e)` — the shape of every program that writes a file — legal at
all. `PrintWriter` stays out: it swallows, and none of its methods declares
one.

**And the compute family checks for co-modification.** A JDK's `HashMap`
records its `modCount` before calling the mapping function and throws
`ConcurrentModificationException` if the function changed the map — so the
memoized-fibonacci idiom, `memo.computeIfAbsent(n, k -> fib(k - 1) + fib(k -
2))`, THROWS on a real JDK. caturra walked straight past it and printed 55.
The check is on all four (`compute`, `computeIfAbsent`, `computeIfPresent`,
`merge`), against the same length-as-`modCount` every fail-fast cursor uses.

Pinned by `diff_a_lab_program_crosses_its_features`.

### A map that keeps what it made (2026-08-27)

A third mixed fuzz — a checked exception, a generic repository interface with a
default and a static, user varargs, a string `switch`, labeled loops, char
arithmetic — found two gaps, and closing the first RETIRED a divergence.

**`Optional.map` erased its element.** A stream's `map` learned to keep it
(the lambda pass types the body and leaves the answer on the synthesized class,
which is the only thing that still knows it downstream); the `Optional` half
was left behind, so `Optional.of("v").map(String::toUpperCase)` was an
`Optional<Object>` — the `filter` after it was refused, and so was the
assignment to an `Optional<String>`. It reads the same field now.

That was a PINNED strictness, and the pin is what reported it: converting it
from `stricter_than_javac!` to an ordinary differential test was the first
thing the change broke. The bullet is gone from the list below.

**`Arrays.binarySearch(a, key, comparator)` was missing.** An array sorted by a
comparator has to be SEARCHED by the same one — a fuzz that sorts before it
searches finds that at once. A null comparator means the elements' own order,
exactly as `sort(a, null)` does.

Pinned by `an_optional_map_keeps_its_element` and
`diff_binary_search_takes_a_comparator`.

### Generics as a program writes them (2026-08-27)

A fourth mixed fuzz — a tree of nodes behind an abstract class, a generic
source interface, a hand-written `Iterator`, boxing identity, bit and char
arithmetic, exception chaining — found three generics shapes that ordinary code
uses and caturra refused outright.

**A generic interface implemented by a class that fixes its argument.**
`interface Source<T> { List<T> all(); }` implemented by
`public List<Node> all()` — the override check compared the two return types
and found `List<TypeVar>` unequal to `List<Node>`, so the class "cannot
implement" its own interface. A type-variable ELEMENT accepts any element now,
the way the erased wildcard beside it already did: after erasure they are one
type.

**A generic method whose functional parameter names its own type variable.**
`<R> R produce(Supplier<R> s)` — the synthesized lambda declares a local of
that variable's ERASURE, and a bare erasure sentinel had no resolution at all,
so the whole method was refused as "a functional interface parameterized on a
method's own type variable is not supported". The sentinel IS a type — the
variable's bound, or `Object` — and resolving it that way is all the shape
needed. (The RESULT of such a call stays erased, which is the separate,
already-enumerated `stricter_return_variable_pinned_only_by_a_lambda`.)

**A lambda over a collection ANOTHER object's method returned.** Only the class
being walked had its own methods consulted, so `b.all().stream().map(…)` — with
`all()` declared on `b`'s class — had no element type and the lambda was
refused for having no functional-interface position, though the same call
inside that class compiled.

**A lambda whose parameter is typed by the RECEIVER'S class type variable.**
`default void each(Consumer<T> c)` on a `SBox implements Box<String>`: the
receiver's own written type says nothing — `SBox` takes no arguments — so `T`
is read by walking its `implements`/`extends` clause, substituting at each
step, the same walk codegen makes for an inherited RETURN type. A receiver
written as the interface itself (`Box<String> b`) already worked.

**A functional parameter mentioning a variable that cannot be pinned.**
`<R> List<R> mapped(Function<T, R> f)` called as `source.mapped(s ->
s.length())`. `T` comes from the receiver, but `R`'s only source is the
lambda's own body, and abandoning the whole target for the one missing
variable left the lambda's parameter an `Object` — so `s.length()` was
"cannot find symbol". A variable that stays unpinned now takes its ERASED
form from the same position of the erased signature, which is the answer the
call site fell back to anyway; the ones that ARE pinned survive.

Pinned by `diff_generics_as_a_program_writes_them` and
`a_lambda_parameter_typed_by_the_receivers_own_type_argument`.

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
program, which would mean it is a shared rule rather than a strictness.

This list was itself audited on 2026-08-14, by running every bullet rather
than reading it, after a covariant return turned out to be documented as a
deliberate refusal months after it should have been. Three bullets had gone
stale in the safe direction — `new StringBuilder().capacity()`,
`Collections.addAll(list, new Integer[] {1})` and the declarations
`LinkedList`/`HashSet`/`TreeMap`/`TreeSet` all compile now — and `subList`,
which has three pinned tests, had never been written down. A strictness that
stops being true is not a bug, but a list that says it is exhaustive has to
earn it.

Audited a third time on 2026-08-20, from the other end: every bullet was run
(all 13 still true — javac accepts each stricter one and rejects each looser
one), and then the PINS were counted against the bullets. Twelve
`stricter_than_javac!` tests, ten bullets: the entry-key method reference in a
comparator and the raw-element array store were pinned and never enumerated,
and are the last two bullets above now. Counting the pins is the cheaper half
of this audit and the half that finds the omissions — running a bullet only
tells you the bullets you have.

Audited again on 2026-08-17, the same way. Every bullet above still held —
but the list had stopped being EXHAUSTIVE, which is the other half of what it
claims. Four strictnesses and one permissiveness had been recorded in the prose
of later entries and never added here or pinned; they are the last four bullets
above and the second bullet below. Prose is where a divergence is explained;
this list is where it is counted, and an entry that only explains does not keep
the count honest. A stale claim was corrected in the same pass: a functional
interface parameterized on a method's own type variable had been fixed two
entries after it was written down.

Audited again on 2026-09-03, after six units of type-system work, by counting
the pins against the bullets — the half that finds omissions. Nineteen
divergence pins, fourteen bullets: three had been recorded in the prose of
later entries and never counted (a qualified library name a class shadows, a
primitive stream's cursor, and a nested argument widening between variables),
and this session's own work added five more, four of them found by re-running
the shapes the entries called "still open" rather than by reading them. Two of
those turned out to be CLOSED — `Comparable<Integer> c = "x";` and
`Comparable<Month> c = aLocalDate;` are refused now — which is the other thing
counting catches: a divergence that stopped being one.

- `Arrays.fill(new String[1], 5)` — javac erases to `fill(Object[], Object)`
  and throws `ArrayStoreException` at run time. (`strict_fill_checks_the_element_type_of_a_reference_array`)
- `Collections.frequency(list, wrongType)` — javac's parameter is `Object`
  and it answers 0. (`strict_frequency_demands_the_lists_element_type`)
- `list.containsAll(otherOfADifferentElementType)` — likewise `Collection<?>`. (`strict_contains_all_demands_the_lists_element_type`)
- `AbstractList<Integer> v;` and the rest of the unmodeled library — a scope
  limit, reported by name wherever written rather than as a missing symbol.
  This bullet used to name `LinkedList`, `HashSet`, `TreeMap` and `TreeSet`
  as well; all four are modeled now, and so are `EnumMap`/`EnumSet` and, since
  2026-09-04, `Vector`/`Hashtable`/`Enumeration`. What is left is the
  `Abstract*` skeletons. (`strict_abstract_list_is_refused_by_name`)
- `Properties p;` — and every other real `java.*` class in a modeled package
  that caturra does not implement. The whole surface was swept (2026-09-04, see
  **Every class this engine does not have**), so this is now exhaustive rather
  than whichever names somebody had happened to hit.
  (`strict_unmodelled_java_classes_name_themselves`)
- `Instant i;` — the `java.time` values that carry an INSTANT or a ZONE, which
  need a timezone database caturra does not vendor.
  (`strict_an_instant_names_itself`)
- `import java.security.*;` — and every other real JDK PACKAGE caturra does not
  model. "package java.security does not exist" is a false statement about the
  JDK; a package that really does not exist still gets javac's own wording.
  (`strict_unmodelled_java_packages_name_themselves`)
- `Collectors c;` — a class caturra models only as a namespace for its members
  cannot name a variable, and now says exactly that instead of claiming the
  class is unsupported. (`stricter_namespace_class_says_what_is_missing`)
- `aFile.getFreeSpace()` and its two siblings — caturra's filesystem is in
  memory and has no device under it, so every number it could answer would be
  fiction about a disk the program cannot fill. A made-up "total" is worse than
  a refusal, because a program that checks before writing would trust it.
  (`strict_a_file_has_no_free_space`)
- `aFile.toURI()` / `toURL()` — the answer would be a `java.net.URI`, a type
  the program could then do nothing with. (`strict_a_file_has_no_uri`)
- `scanner.locale()` / `useLocale(l)` — a `java.util.Locale` VALUE, and caturra
  models `Locale` only as a constant read where it is written. The default is
  host state besides. (`strict_a_scanner_has_no_locale`)
- `aDate.query(q)` and `adjustInto(t)` — the `TemporalAccessor`/
  `TemporalAdjuster` plumbing every `java.time` value declares, over interfaces
  caturra does not model. A program never writes one itself; they are how the
  JDK's own types talk to each other. (`strict_a_date_has_no_query`)
- `aDateTime.atZone(z)`, `atOffset(o)`, `toInstant()`, `toEpochSecond(o)`,
  `ofInstant(...)` and `getChronology()` — everything carrying an INSTANT, a
  ZONE or a calendar choice, which needs the timezone database caturra does not
  vendor. (`strict_a_date_time_has_no_zone`)
- `anObject.wait()`, `notify()`, `notifyAll()` — `Object`'s monitor methods,
  so EVERY receiver has them. caturra runs a program on one thread: there is no
  second thread to wake, and a `wait()` that returned immediately would be a
  lie about a program that deadlocks on a JDK.
  (`strict_no_thread_to_wait_for`)
- `aCollection.spliterator()`, and the same name on `Arrays`, `Stream` and
  `IntStream` — a `Spliterator` is a parallel-decomposition handle for a
  machine with threads. (`strict_no_spliterator`)
- `System.getProperty(...)` and the fifteen other `System` members that reach
  for the HOST — properties, the environment, a `Console`, a native library, a
  `SecurityManager`. caturra runs in a page with no process around it. The
  three wrapper readers whose names read like parsers (`Integer.getInteger`,
  `Long.getLong`, `Boolean.getBoolean`) read properties too, and go the same
  way. (`strict_no_system_properties`)
- `aClass.getAnnotations()` and the nine other annotation questions — caturra
  parses annotations and discards them, so none survives to be read back.
  Beside them, the four questions about a GENERIC signature
  (`getGenericSuperclass`, `getTypeParameters`, ...), which erasure has already
  removed. (`strict_no_annotations_at_runtime`)
- `formatter.parse(text)` and the rest of `DateTimeFormatter`'s parsing half —
  caturra's formatter is a pattern that RENDERS a value; text is read back
  through `LocalDate.parse(text, formatter)`, which is what the refusal says.
  (`strict_a_formatter_does_not_parse`)
- `Collections.checkedList(...)` and its nine siblings — a view that type-checks
  every write at RUNTIME against a `Class`. caturra's compiler is the check
  that runs here. `asLifoQueue` and `newSetFromMap` are refused beside them, as
  adapters over a collection caturra can already spell directly.
  (`strict_no_checked_views`)
- `Stream.builder()` / `IntStream.builder()` — a `Stream.Builder` is a mutable
  accumulator caturra models no type for; collect the elements and call
  `stream()`. (`strict_no_stream_builder`)
- `Collectors.toConcurrentMap(...)` / `groupingByConcurrent(...)` — they collect
  into a `java.util.concurrent` map. On one thread the ordinary ones do the
  same job. (`strict_no_concurrent_collectors`)
- `Character.getName(cp)`, `codePointOf(name)` and `getDirectionality(c)` —
  caturra carries Unicode's character CATEGORIES, which is what `isLetter` and
  its siblings need, not the character database of NAMES.
  (`strict_no_unicode_character_names`)
- `aFrame.getModuleName()`, `getModuleVersion()` and `getClassLoaderName()` —
  the three pieces Java 9 added to a `StackTraceElement` for the module system.
  (`strict_a_frame_names_no_module`)
- `stats.andThen(consumer)` — the summary statistics are `IntConsumer`s, so
  they inherit its composing default; a consumer built out of two others is not
  a value caturra models. Call them in turn.
  (`strict_no_composed_consumer`)
- `Math m;`, `Collectors c;`, `Arrays a;` — a class caturra models only as a
  namespace for its static members cannot name a variable, though javac
  accepts the declaration (they are ordinary class types). Nobody writes one,
  but the refusal has to say so: written in full it gave the honest reason,
  written simply it read as a typo — "unknown type 'Math'", about a class
  every program has used. (`stricter_namespace_class_as_a_variable_type`)
- `Arrays.stream(array, from, to)` — the RANGE overload; the whole-array form
  is modelled. (`stricter_arrays_stream_takes_no_range`)
- A factory that ADOPTS its context (`Collections.emptyList()`,
  `Optional.empty()`, `List.of()`) used as an argument where the OVERLOADS
  disagree about it: `two(Collections.emptyList())`, against
  `two(List<String>)` and `two(String)`, is "reference to two is ambiguous".
  These factories are typed like the null literal, which is what lets one be
  assigned to a `List<String>` with no element to check; in an overload set
  that leniency matches both candidates, where javac infers `List<String>` and
  matches one. Giving them their context-free type instead (`List<Object>`, the
  one `var` gets) was tried and is worse — the element is then CHECKED, and
  `list.addAll(Collections.emptyList())` becomes a type error. The lenient
  typing stays. (`stricter_a_context_free_factory_in_an_overload_set`)
- `Collections.sort(null)` — javac infers the type variable from the null, so
  `T extends Comparable<? super T>` is satisfied vacuously and the program
  compiles and throws at run time. A null argument reads here as the empty
  `List<Object>` a DIAMOND argument means, and javac refuses THAT ("no suitable
  method found for sort(ArrayList<Object>)"), so the bound really is
  unsatisfied. Only the bare literal differs.
  (`stricter_a_null_literal_to_a_bounded_collections_method`)
- A conditional over two classes that share SEVERAL interfaces, passed as an
  ARGUMENT: `d(flag ? new Sq() : new Ci())` where `Sq` and `Ci` implement both
  `Shape` and `Drawable` and `d` takes a `Drawable`. javac's type for a
  conditional is the INTERSECTION of everything both branches share (JLS
  §15.25); caturra's join has to pick ONE, and the conditional ADOPTS its
  target wherever the target is known — a declaration, an assignment, a return,
  an array store, a field initializer, through a nested conditional. An
  ARGUMENT is the position where it is not: the type is needed to CHOOSE the
  overload, before any parameter is known.
  (`stricter_a_conditional_as_an_argument_needs_one_shared_type`)
- A local class declared inside a SWITCH arm
  (`case 0: class Helper { … }`). The switch block is one scope and its arms
  hold block statements like any other block, but the arm parser reads
  STATEMENTS only, and a class declaration is not one. Every other block
  position takes it. (`stricter_local_class_in_a_switch_arm`)
- A method whose bytecode outgrows a 16-bit branch offset — `code too large`,
  javac's own wording, at about half the size javac allows. A class file's
  branches are signed 16-bit, and caturra reaches that before the 64K limit on
  the code array itself, because a string concatenation compiles to a
  `StringBuilder` chain where javac emits one `invokedynamic`. Around 1,400
  printing statements in a single method; javac takes about 3,000 of them.
  Until this was written it was a compiler PANIC.
  (`strict_a_method_that_outgrows_a_class_file`)
- A fully qualified LIBRARY name that a program's own class shadows —
  `java.util.List` in a program that declares a `List` of its own. javac
  resolves a qualified name without consulting what is in scope; caturra reads
  the simple name, so the qualified spelling is unusable. Written down in the
  prose of the scoping entry when it was measured and never counted here.
  (`strict_a_qualified_library_name_a_class_shadows`)
- `PrimitiveIterator.OfInt c = IntStream.range(0, 3).iterator();` — a nested
  type of a class caturra does not model. It is refused BY NAME (rather than
  as a missing package, which is what it used to say), and javac accepts it.
  The same omission: explained in prose, never counted here.
  (`strict_a_primitive_stream_cursor`)
- A locale that is not an English one: `Month.getDisplayName(TextStyle.FULL,
  Locale.FRANCE)`. caturra ships one text, en-US, and answering a French
  program in English would be a WRONG answer rather than a missing one — so the
  locale is checked where it is written. (`stricter_a_locale_that_is_not_english`)
- A class that EXTENDS a builtin collection — `class Counts extends
  HashMap<String, Integer>`. caturra's collections are the VM's own objects,
  not classes compiled from source, so there is nothing to inherit from; the
  refusal says exactly that rather than pretending the name is unknown. Every
  other way of holding one (a field, a wrapper, composition) works.
  (`stricter_extending_a_builtin_collection`)
- A DIAMOND of a class with more than one type parameter, used inline:
  `new Pair<>("ab", 2).first().length()`. The plan behind diamond inference
  joins its sources into a single answer, so a second variable has nowhere to
  go and the whole thing reads raw. Writing the arguments out
  (`new Pair<String, Integer>(…)`) or assigning to a declared variable first —
  which is how a pair is nearly always used — compiles in both.
  (`stricter_a_diamond_with_two_arguments`)
- A factory INSIDE a factory: `List<List<Number>> rows = List.of(List.of(1));`
  and `Map<String, List<Number>> named = Map.of("k", List.of(1));`. A poly
  expression takes its type argument from the target, and the rule does not
  recurse — reaching a level down needs to know the inner call is poly as
  well, and refusing is the safe answer. Assigning the inner list to a
  `List<Number>` variable first compiles in both.
  (`stricter_a_factory_inside_a_factory`)
- `Map.Entry.comparingByValue().reversed()` with no type witness. javac
  infers `Comparator<Entry<Object, V>>` for the bare factory call, and
  `.reversed()` freezes that before the target type can correct it, so javac
  demands `Map.Entry.<K, V>comparingByValue()`, a typed variable, or a
  wrapper like `Collections.reverseOrder(...)`. A call now CARRIES its witness
  (see **Explicit type witnesses**), so the two spellings are no longer the
  same tree — but caturra's generics are erased, and it is the ABSENCE of a
  witness that javac makes fatal here, which is a rule about inference this
  engine does not model. All three forms javac accepts do work, so nothing
  legitimate is blocked by leaving it permissive. Recorded when the two
  factories were added (2026-08-14) rather than left for a later sweep to
  find. (`entry_comparator_needs_a_witness_to_reverse`)
- `(List<String>) Collections.emptyList()`. The factory types as a `null` that
  adopts its context, and a CAST is a context — so caturra reads this as an
  identity cast, where javac infers `List<Object>` for the bare call and calls
  the cast inconvertible. Assigning the factory to a `List<String>` first is
  legal in both, and is the ordinary spelling. (`empty_factory_adopts_a_cast_as_its_context`)
- `W.<Dog>id(new Cat())` — a witness naming a USER class is not checked against
  the argument. The pass that reads witnesses knows each class's members but
  not its ANCESTRY, so it cannot tell a wrong class from a supertype, and a
  wrong REJECTION would be worse than the missing check. The provable cases —
  a primitive against the one wrapper it boxes to, and one concrete final
  library type against another — ARE refused. (`a_witness_naming_the_wrong_user_class`)

- `Optional<ArrayList<Pet>>` assigned to an `Optional<List<Pet>>` between two
  declared VARIABLES. Generics are invariant and javac refuses it; caturra's
  rule — the value written as the class where the variable says the interface
  — is stated on the types alone, and the types cannot tell an inference site
  from an assignment. Explained where it was introduced and never counted
  here. (`loose_a_nested_argument_widens_between_variables`)
- A generic method's variable pinned by a TWO-argument container:
  `static <K, V> void put(Map<K, V> into, K key, V value)` called as
  `put(mapOfStringInteger, 1, 2)`. A container parameter pins its variable
  exactly, and the check reads that pin — but the plan the parser records
  names a variable only for a container with ONE argument, so a map pins
  nothing and the key goes unchecked. Refusing on a guess would be worse than
  the missing check. (`loose_a_variable_pinned_by_a_map_parameter`)
- A type variable used INSIDE its own class: `class Bag<T> { void add(T v);
  void seed() { add(1); } }`. There is no receiver to read an argument from,
  so the variable stands for its BOUND, and a `T` whose bound is `Object`
  takes anything. javac checks against the variable itself, which admits only
  a `T`. Every call from OUTSIDE the class is checked against the receiver's
  own argument. (`loose_a_type_variable_inside_its_own_class`)

It held a worse one on 2026-08-13: a cast to `String` accepted ANY reference
source, so `(String) Integer.valueOf(1)`, `(String) aStringBuilder` and
`(String) aList` all compiled here and are compile errors on a real JDK
(JLS §5.5 — a reference cast needs one type to be a subtype of the other, and
`String` is final, so only a supertype casts down to it). Found by a sweep of
DIAGNOSTIC WORDING, not of behaviour: the program was in the sweep to compare
error text, and caturra produced no error at all.

Until 2026-07-09 it held three: `Collections.sort`, `max`/`min` and
`binarySearch` over a list whose element type is not `Comparable`, which
caturra accepted and failed on at run time with `ClassCastException`.
Those four methods are declared over `T extends Comparable<? super T>`,
so the bound is now checked at compile time, and `ArrayList<Object>` is
refused with it. `Arrays.sort` used to refuse one too, because its bundled
parameter was `Comparable[]` — a strictness that outlived its reason and was
corrected on 2026-08-30: a JDK declares `sort(Object[])`, so that program
compiles and throws at RUN time. The bound reaches exactly
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
  generic inference; caturra names the two array types. Same for a bounded
  type variable in a container parameter (`total(new ArrayList<String>())`
  for a `<T extends Number> double total(List<T>)`).
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

### A Map.Entry is a type of its own (2026-08-27)

Chasing the shape above through a generic class found a whole cluster around
one type. `Map.Entry` was modelled as the thing an `entrySet()` yields and
little else; a program that names it does more than iterate.

**A for-each over a generic class's own map.** Inside a `class Store<K, V>`,
`for (Map.Entry<K, V> e : map.entrySet())` — the ordinary way to walk the map
such a class holds — was refused with a type that could not be converted to
ITSELF: "Map.Entry<Object,Object> cannot be converted to Map.Entry<Object,
Object>". The entry set's erased element and the loop variable's written
`K`/`V` describe alike and differ only in an interned element, and only
IDENTITY was accepted between two entry types. They join by the same element
rule every collection uses, where a type variable on either side accepts.

**The raw spelling.** `for (Map.Entry e : m.entrySet())` — how a program that
predates generics walks a map, and how plenty of ordinary code still does —
resolved to no type at all ("unknown type for the for-each variable"), and
`Set<Map.Entry>` was "Entry works as a variable, but caturra does not model it
as a collection element". Only the parameterized form had an arm.

**A real set of entries.** `new LinkedHashSet<>(m.entrySet())` — the copy a
program makes to keep the entries past the map's next change — could not be
assigned to the `Set<Map.Entry<K, V>>` that names it, because that written type
IS the view type here. Both are a `java/util/Set` holding entries at run time,
so the widening is a no-op; what was missing was saying so, in both gates. The
bulk surface (`addAll`/`removeAll`/`retainAll`/`containsAll`) had to learn the
entry element too: `SelfCollection` reads the receiver's FIRST type argument as
its element, and an entry set's first argument is its KEY, so it was asking for
a collection of keys.

**...and one thing that should NOT compile did.** `HashSet<Map.Entry<K, V>> h =
m.entrySet();` — javac refuses it, because an `entrySet()` is a `Set` and not a
`HashSet`. `HashSet<E>` normalizes to the interface name a line before the
entry carve-out reads it, so the concrete spelling resolved as the view too.
The carve-out is gated on the interface spelling now, and the accepts-invalid
is pinned as a rejection.

Pinned by `a_map_entry_as_a_type_of_its_own` and
`an_entry_set_view_is_not_a_hash_set`.

### A generic class as a program uses one (2026-08-27)

Probing the shape above from the outside — a generic class used the way
ordinary code uses one, rather than declared and admired — found five refusals
in a row, and one accepts-invalid underneath them.

**A static factory dropped its own class.** `static <T> Box<T> of(T v)` called
as `Box.of("hi")` was typed `String`: the return-inference plan says which
arguments pin the variable, and its answer was handed back as the WHOLE return
type. That is right only when the declared return IS the variable
(`<T> T first(List<T>)`); for a container return the pinned type is the
container's ARGUMENT. Only the library containers were re-argumented, so a user
generic class fell through to the plan's own answer and the call was typed as
its own element — "String cannot be converted to Box<String>", about a factory
whose declared return says otherwise.

**A `new` answered the RAW class.** `new Node<Integer>(5)` and `new Node<>(5)`
both typed as a bare `Node`, so `Arrays.asList(new Node<>(3))` could not be
assigned to the `List<Node<Integer>>` on the same line. The written arguments
resolve now, and a DIAMOND infers its argument from the constructor's own
arguments the way javac does — the same plan the return inference uses, built
for constructors too. Both the emit path and `type_of` answer it, which is the
divergence this file keeps meeting: through a variable the mismatch was caught,
inline it was not.

**A literal collection of parameterized things lost them.** The join reads each
argument's ELEMENT form, and a parameterized user class has none of its own, so
it fell back to the top `Object` — `Arrays.asList(aNodeOfInt)` was a
`List<Object>` that would not assign to the `List<Node<Integer>>` beside it,
with a message that said `List<Object>` twice. Such a value INTERNS now,
exactly as it does when a collection holds one.

**`? super T` refused every write.** A `Collection<? super T> sink` — the drain
method every generic container has — reported "a '? extends' collection cannot
be written to", the exact opposite of what `? super` means. A type variable's
erasure leaves the wildcard's bound EMPTY, and an empty bound fell into the
unbounded arm; a LOWER wildcard keeps its variance, which is the writable one.

**A nested element did not satisfy a bound.** `biggest(listOfNodes)` for a
`<T extends Comparable<T>> T biggest(List<T>)`: the erased parameter carries
the bound, and an interned element was not asked about the class hierarchy at
all.

**The accepts-invalid underneath.** While every `new` answered the raw class,
`list.add(new Node<Integer>(6))` on a `List<Node<String>>` COMPILED — the
unchecked conversion a genuinely RAW type gets, applied to a type the program
had parameterized right there. It is refused now, and pinned. (A raw element
and a parameterized one still convert both ways: that IS unchecked Java, and
it is what a diamond whose argument cannot be pinned still leans on.)

The messages improved with the types: a nested element used to DESCRIBE as
`Object`, so a mismatch between two lists of parameterized things read as
"List<Object> cannot be converted to List<Object>" — a type that cannot convert
to itself. It describes as what it is, which is javac's wording exactly.

Pinned by `a_generic_class_as_a_program_uses_one` and
`a_parameterized_new_is_not_another_parameterization`.

### A type variable the lambda's body pins (2026-08-27)

`<T, R> R conv(T value, Function<T, R> f)` called as `conv("abc", s ->
s.length())`. Nothing at the call site names `R`: the lambda's own BODY is the
only thing that says what it is. That was a pinned STRICTNESS for four rounds —
the lambda's parameter typed correctly, its result stayed `Object`, and a call
assigned to an `Integer` was an incompatible type.

Everything needed was already in place and never joined up. The lambda pass
types the body and leaves the answer on the synthesized class as a synthetic
static field, which is how a mapped stream keeps its element. The inference
plan a generic method carries had two kinds of source — the parameter IS the
variable, or a container OF it — and a functional interface whose RESULT is the
variable is a third: `InferSource::LambdaResult`. `Predicate` and `Consumer`
are deliberately not among the interfaces that qualify; their result is
`boolean`/`void`, so a variable in their last position is a PARAMETER and pins
nothing.

Two things had to be fixed for the field to be readable at all.

**The lambda's answer sat in a synthesized local.** A body with a declared
result type compiles to `R __caturraResult = (expr); return __caturraResult;`,
and that local is declared as the target's result — `Object` when the target is
parameterized on a type variable. Reading the returned NAME gave the erasure
and nothing else, so no lambda in this position ever recorded what it produced.
The initializer is the answer.

**A parameterized receiver skipped the method's own inference.** `type_of`'s
arm for a call on a `Box<String>` read `sig.ret` straight and substituted the
receiver's argument into it — so `box.map(s -> s.length())` assigned to a
variable typed fine, while the same call CHAINED into `.get()` answered an
`Object`. The type_of/emit divergence, in the one arm that had not been
converted.

The strictness bullet is gone from the list below; the test that pinned it is
an ordinary differential test now, widened to the chained and container forms.

Pinned by `a_return_variable_pinned_only_by_a_lambda`.

### Sorting a map's entries (2026-08-27)

A 60-program mixed fuzz — random compositions of independent snippets — came
back with 14 failures that were all ONE missing overload, and pulling on it
found four more defects around it. The other 46 programs were byte-identical,
which is what a sweep is for after a week of typing changes.

**`Map.Entry.comparingByKey(cmp)` / `comparingByValue(cmp)` were missing.** The
no-argument forms were modelled; the overloads that take an ordering for the
key (or value) — how a program sorts entries by DESCENDING value — were not.
`ComparatorSpec::Entry` carries an optional inner comparator now. A null one is
not "natural ordering" the way `sort(null)` is: the JDK runs
`Objects.requireNonNull`, so the factory itself throws, and so does this.

**A program that sorts entries and names `Comparator` nowhere had no
comparator interface at all.** The bundled `__Comparator` is pulled in by
scanning the source for the words that imply it, and `Map.Entry.comparingByKey()`
spells none of them — so `new TreeSet<>(Map.Entry.comparingByKey())` was refused
as "takes a Collection or a Comparator", about a Comparator. `comparingBy` is
one of those words now.

**`addAll` from an `entrySet()` copied the KEYS.** The cheap element walk of a
map view answers keys for anything that is not the values view, and the copy
CONSTRUCTORS had already been fixed to materialize real entries — the paths
that ADD had not, so `list.addAll(m.entrySet())` printed `[a, b]` where a JDK
prints `[a=3, b=2]`. `containsAll`/`removeAll`/`retainAll` walk the argument the
same way and needed the same fix, which the first half of this exposed: with
`addAll` fixed and the others not, a set of entries did not contain the entries
it had just been given.

**`Comparator.comparing(Map.Entry::getKey)` — a pinned strictness — is closed.**
The extractor rule types a method reference from its QUALIFIER, and a nested
library type is not a name it can read. The element the surrounding
`Comparator<E>` position gives is the same answer, and the sort site hands it
down; the lambda form of the same thing
(`Map.Entry.comparingByValue((a, b) -> ...)`) takes its two parameters from the
entry's key or value the same way.

Pinned by `sorting_a_maps_entries`; the strictness bullet is gone from the list
below.

### The algorithms reach through a subList (2026-08-27)

A second mixed fuzz — 80 programs composed from 24 fresh snippets (abstract
classes, private interface methods, inner classes, a custom `Iterable`,
exception chaining, enum bodies, bit work, `Objects`, labelled loops) — came
back with 14 failures that were again ONE cause, and it was a silent wrong
answer.

**`Collections.fill(list.subList(0, 2), 9)` did nothing.** A `subList` view has
no vector of its own, and every writing algorithm looked for one: it found
nothing to write and wrote nothing, with no error. The same for `sort`,
`rotate`, `copy` and `replaceAll` — five of the family, silently. (`reverse`
and `swap` go through the view's own methods and always worked, which is why
this hid.) They run over a scratch copy of the range and splice it back now,
the way the view's own `sort`/`removeIf` already did.

Pulling on that found the read-only half. **A range of an UNMODIFIABLE list was
freely writable.** `subList` is not a mutator, so the wrapper forwarded it to
the list inside, and the plain view that came back wrote straight through the
wrapper that exists to forbid it: `unmodifiableList(l).subList(0, 2).set(0, 9)`
succeeded, and the algorithms handed the same range read it as EMPTY and
returned quietly, ahead of the `UnsupportedOperationException` they owed. A
range of a read-only list is read-only, as the JDK's is.

Pinned by `the_algorithms_reach_through_a_sublist`.

### Cursors and wildcards over every collection (2026-08-27)

Two cross-product sweeps — eleven collection shapes against a dozen cursor
sequences (walk-and-set, remove-before-next, double-remove, add-during,
backwards, exhausted, `forEachRemaining`, a change under the cursor), 182
programs — found four gaps and agreed on everything else, including the
exception CLASS and message of every misuse.

**`subList.listIterator(index)`** — how a program walks a range backwards —
was "unknown native member". The view had the no-argument cursor only, so the
call fell through to the plain list path.

**A map view and a `PriorityQueue` had no `toArray()`.** Every collection has
one, and both were "unknown native member" for it. It is answered once for
every kind now, beside `descendingIterator`, which was already shared for the
same reason. The elements are MATERIALIZED, so an `entrySet().toArray()`
answers real entries rather than the keys its cheap walk yields — the typed
`toArray(T[])` beside it had the same reading and needed the same fix.

**A lambda handed to a WILDCARD-typed receiver was refused.**
`Iterator<?> it = list.iterator(); it.forEachRemaining(v -> …)` — the element
reached the synthesized class as the wildcard's own encoded name, which is no
type, and the refusal said "a functional interface parameterized on a method's
own type variable". A wildcard READS OUT as its bound (or `Object`), which is
what a JDK gives the lambda too.

**A wildcard bounded by a FINAL type read as `Object`.** `? extends Integer`
IS `Integer` — nothing else can be one — and the same for `? extends String`,
so the invariant element is not an approximation but the same type. The
`Object` fallback made `ints.get(0) + 1` an error on a `List<? extends
Integer>`. (A bound caturra models as no CLASS at all — `CharSequence` — still
falls back, and a `List<CharSequence>` is refused outright with its own honest
message.)

Pinned by `cursors_and_wildcards_over_every_collection`.

### A lambda inside a hoisted body (2026-08-27)

A MIRROR sweep — the same expression written in twelve syntactic positions
(a `println` argument, a `var`, a concatenation, an argument, a return, a field
initializer, an array element, a ternary branch, a lambda body, an anonymous
class body, `String.valueOf`, a plain assignment) across 26 expressions, 299
programs — agreed everywhere but one position, which is exactly what such a
sweep is for.

**A lambda inside an anonymous class body was refused.** The parser lifts that
body out of the expression it was written in, and this pass walks it like any
other class — so it lost the enclosing class's fields and the locals the body
captures, and a stream pipeline inside one had no element type: "a lambda or
method reference is only allowed where a functional-interface type is
expected", for a program whose identical pipeline one line OUTSIDE compiles.
The `new` site records what it can see, and the hoisted body reads it back.

The bodies are walked AFTER the classes that create them, and in reverse of
their appearance: the parser appends the innermost first, so an anonymous class
inside an anonymous class was walked BEFORE the body that creates it, and
nothing had yet recorded a scope for it.

**And the same tell, one level down.** `byKey.get("k").stream().map(v -> …)` —
a pipeline over a collection stored INSIDE another collection — was refused
while the identical list through a variable compiled. A library READ takes its
type from the receiver's own written type: a map's `get` is its VALUE type, a
list's `get` (or a queue's `poll`, or an `Optional`'s `orElse`) its element.

Pinned by `a_lambda_inside_a_hoisted_body`.

### The map-building collectors (2026-08-27)

A sweep of one BEHAVIOUR written several ways — iterate, build a list, sort,
filter, look up, join, count, take the max, dedup, group — each spelling a
program actually chooses between, 49 in all. Every spelling agreed but one, and
the one that failed was `new TreeMap<>(stream.collect(groupingBy(f)))`, refused
as "takes a Map or a Comparator", about a Map.

**The map-building collectors had no result type.** `groupingBy`, `toMap` and
`partitioningBy` typed as nothing, so their result could not be copied, walked
with a two-argument lambda, or read from — inline. Through a declared variable
every one of them worked, which is the tell this file keeps recording.

The key is what the classifier ANSWERS: the lambda pass types the body and
leaves it on the synthesized class, the same field a mapped stream's element
comes from. The value is the downstream collector's result, or a list of the
stream's own element when there is no downstream. Both halves needed it — the
lambda pass, to type a `forEach((k, v) -> …)` over the result, and codegen, to
type the value everywhere else.

**`Collectors.mapping` was missing outright**, and it exists to be a
`groupingBy` downstream. The numeric collectors (`summingInt`, `averagingInt`
and their widths) gathered correctly and typed as nothing, so `+ 1` after one
was "bad operand types".

Pinned by `the_map_building_collectors`.

### A stream of arrays (2026-08-27)

A second spelling sweep — parse numbers out of a string, reverse one, sum a
grid, copy an array, count characters, repeat a string, search, Fibonacci,
palindrome, split lines, forty spellings — put four failures here, all around
the same idea: an ARRAY as a stream's element or source.

**`Arrays.stream(csv.split(","))`** had no element inline, though the same
array through a variable did — a library call that ANSWERS an array
(`split`, `toCharArray`, `toArray`, `copyOf`) is its own declaration, and the
pass read only variables and literals.

**`Arrays.stream(grid)` on an `int[][]`** was "no suitable method found for
stream(int[][])". A multi-dimensional array is a reference array whatever its
element, so it always spreads — into a stream of ROWS, which is how a program
walks a grid.

**`Arrays::stream` as a method reference compiled to `row.stream()`.** The
library-static table was consulted only for a class the program (or the
bundle) does not declare — and caturra BUNDLES an `Arrays`, so the reference
fell through to the unbound-instance form. The table is class-aware now, and
what beats it is a class that declares the name as an INSTANCE method, which is
the only case that should shadow.

**`flatMapToInt` was in no table at all**, though `flatMapToInt(Arrays::stream)`
is the ordinary way to sum a grid; the VM's `flatMap` already does exactly what
it needs.

**And a stream whose element IS an array erased it.** `Stream.iterate(new
long[] {0, 1}, p -> new long[] {p[1], p[0] + p[1]})` — the Fibonacci one-liner
— typed its element as `Object`, so `p[1]` inside the lambda and `[0]` after
the fold were both "array required, but Object found". An array is not an
element KIND of its own: it interns, as it does anywhere a container holds one.

Pinned by `a_stream_of_arrays`.

### The numerics that were known-hard (2026-08-27)

An audit round left two items as "known-hard, likely document rather than fix":
`Double.toString(1e23)`, where JDK 11's legacy `FloatingDecimal` is NOT
shortest-round-trip and prints `9.999999999999999E22`, and `Math.cosh(1.0)`,
off by one ULP as a transcendental may be.

Re-measured: a 765-value random sweep — doubles and floats from random BITS,
through `toString`, `String.valueOf`, concatenation, `%f`/`%e`/`%g` and
`parseDouble` round-trips — found **zero** divergences, and the transcendentals
agree to the last digit. Both were closed by later work and the note had gone
stale.

They are pinned now rather than documented, which is the point: a deferral that
says "likely document not fix" reads as "do not check again", and this one hid
work that was already done.

Pinned by `the_numerics_that_were_known_hard`.

### A hierarchy walked every way (2026-08-27)

Nine probes over inheritance and dispatch: overriding and hiding, `super` and
an interface's `X.super.m()`, a constructor calling an overridable method (the
trap where the subclass's fields are not set yet), static hiding and a static
called through a null reference, private methods that do NOT override, the
overload matrix (`Object`/`String`/`Integer`/`int`/`long`/varargs, and a `null`
argument), covariant returns, a generic repository behind an abstract class,
and a user `Iterable`.

Four of the nine agreed exactly — including every corner above. The five
failures were all in one place: where a type VARIABLE meets inheritance.

**A type variable's erasure is its BOUND, and applicability never knew it.**
`held.compareTo(other)` inside a `Box<T extends Animal>` could not find
`compareTo(Animal)` at all — the call was refused before anything could check
it. Resolution is lenient about a variable now (it has no way to know WHICH
variable it is), and the conversion is what checks the bound, which keeps the
mismatched `take(held)` for a `take(String)` refused.

**A parameter declared as the variable takes what the RECEIVER'S argument
says.** `new Box<>(animal)` and `box.compareHeld(animal)` were both "Object
cannot be converted to Animal". The receiver's arguments are recorded around
the call — including a `new`'s own, written or inferred.

**An inherited CONTAINER field did not substitute.** A subclass that fixes its
supertype's argument reads an inherited `T value` as that argument; the
`protected List<T> items` every generic base class keeps was left an Object
list, so `for (String s : items)` inside the subclass was an error.

**`Iterable<Integer>` dropped its argument.** A wrapper argument is not carried
on a synthesized interface — right for `Comparable<Integer>`, which is modelled
as the face a wrapper widens to, wrong for the iteration pair: a for-each over
the interface face saw an `Object` element.

**`Iterator.remove()` is a DEFAULT** since Java 8, and the synthesized
interface did not declare it, so the `@Override public void remove()` a cursor
writes to refuse removal was "does not override or implement a method from a
supertype".

Pinned by `a_hierarchy_walked_every_way` and `a_type_variable_is_only_its_bound`.

### What the messages say (2026-08-27)

A second diagnostic sweep — 36 programs, each wrong in ONE ordinary way, in the
areas this week's work touched: generics, collections, lambdas and streams,
inheritance, and the everyday mistakes beside them. Every first line was put
beside javac's. **Seventeen matched; twenty-nine do now.**

**A LOCAL class was named as it is hoisted.** `class A` written inside a method
becomes `A$Local1`, which is the reverse of a nested class's `Outer$Inner` —
the source name is BEFORE the `$`, not after — so splitting alike printed
`Local1`, a class the program never wrote. Five messages said it.

**A lambda that does not FIT its target was blamed on the position.** javac
names the mistake: "incompatible parameter types in lambda expression" when the
arity is wrong, "String is not a functional interface" when the target is not
one at all. caturra said "a lambda or method reference is only allowed where a
functional-interface type is expected" for both — true, and about the wrong
thing.

**A method reference to a missing member read as if the program had written the
desugaring's call**: "cannot find symbol … location: variable __p0 of type
String", about a parameter the compiler invented. javac's headline is "invalid
method reference", and its location is the qualifier CLASS.

**`String<Integer>` was "cannot find symbol"** — about `String`, which is
perfectly well known. javac: "type String does not take parameters". The arity
message beside it dropped its "in class Map" tail, which javac puts on the
caret line rather than in the text.

**A single constructor with one wrong argument** listed candidates where javac
names the argument: "incompatible types: String cannot be converted to int" —
the shape the METHOD path already used.

**A type variable printed as `Object`.** javac says "T cannot be converted to
String"; `Object` there names a type the program never wrote.

Two differences are left and both are deliberate: caturra says which class a
`no suitable method` belongs to (javac puts it on the candidate lines), and
`incompatible types: inference variable T has incompatible bounds` is reported
as the concrete mismatch instead, which is what the program can act on.

Pinned by `the_diagnostics_say_what_javac_says` and
`a_type_variable_is_named_in_a_diagnostic`.

### The fuzz toolkit is checked in (2026-08-27)

Six generated sweeps found this week's defects, and every one of them lived in
`/tmp`. Two are general enough to re-run unchanged, so they are in the tree now
as `scripts/fuzz/`:

**`programs.py`** writes typed random programs — it tracks the type of every
variable it declares and only builds an expression where its type fits, which
is what keeps javac's rejection rate at zero and makes each case a real
comparison rather than a syntax check. The statement mix is what a student
program is made of, plus the shapes that have historically broken: a generic
class, a map walked by its entries, method references, nested lambdas, a
`subList` handed to an algorithm. 300 programs over the newest engine: no
divergence.

**`positions.py`** writes the same expression in twelve syntactic positions,
each printing the same thing — the MIRROR sweep, which is how the
anonymous-class scope bug was found.

**`run.py`** compiles and runs each case on both engines. Two things it does
that a hand-rolled loop forgets, both of which cost an hour this week: it
compiles each case ON ITS OWN (a batch compile writes no class files when one
program fails, so a single syntax error reads as "everything diverged"), and it
compares the FAILURE as well as the output (a program that throws identically
on both engines is not a divergence).

The narrower generators — the receiver × operation matrix, "one behaviour
written five ways", the Scanner input matrix, one ordinary MISTAKE per program
against javac's wording — stay written-fresh: they are a few dozen lines each,
and the entries above record what each found.

### A CharSequence element (2026-08-27)

`List<CharSequence>` — the interface `String` and `StringBuilder` share — was
refused outright: "CharSequence works as a variable, but caturra does not model
it as a collection element". It was the last refusal of that shape a sweep
could still reach, and the model already had the mechanism: a type that is not
an element KIND rides INTERNED, exactly as a nested collection does. The two
implementors are accepted where one is wanted, which is the whole of what
`CharSequence` means here.

The wildcard half reads the same way. A bound that names no CLASS still names a
type: `? extends CharSequence` reads as a CharSequence — so the
`total(List<? extends CharSequence>)` every library-ish API declares can be
walked — and `? extends Integer` IS `Integer`, since a final class has no
subtypes, where before its elements read as `Object` and `xs.get(0) + 1` was an
error.

Three smaller things fell out of it. A `String` and a `StringBuilder` JOIN at
`CharSequence` now (javac's answer for `Arrays.asList("a", new
StringBuilder())`), where joining them at `Object` left the list unassignable
to the `List<CharSequence>` the program declared. A cursor and an `Optional`
widen by the same rule the collections do — left out of the assignment matrix,
an `Iterator<String>` could not be held by an `Iterator<? extends
CharSequence>` once its element stopped being a wildcard. And `CharSequence`
names a class for a method REFERENCE, so `CharSequence::length` resolves.

Pinned by `a_char_sequence_element`.

### Any modelled type as an element (2026-08-27)

`List<Scanner>`, `Set<File>`, `List<Path>`, a list of writers, a list of stack
frames — every one was refused with "works as a variable, but caturra does not
model it as a collection element". The reason was honest and the limit was
already lifted: a type with no element KIND of its own rides INTERNED, exactly
as `List<List<Integer>>` does, and reads back as itself. The rule replaces the
type-by-type list with "any modelled reference".

Two shapes had to follow it. A LITERAL of such values keeps its type —
`Arrays.asList(Paths.get("x"), Paths.get("y"))` was a list of `Object` and
would not assign to the `List<Path>` beside it — and a type VARIABLE is an
element POSITION rather than a nested type, which the general rule got wrong
first: `new Node<>(item)` inside a `GStack<E>` became a `Node<nested T>` that
could not be assigned to the `Node<E>` beside it, a type that cannot convert to
itself.

The compatibility page's "Any object as a collection element" moves from
unsupported to supported, recorded against a real JDK like every other claim
there. What is left in that list is a type caturra models NOWHERE (`Thread`),
which is a different message and a deliberate boundary.

Pinned by `any_modelled_type_as_an_element`.

### Text to bytes and back (2026-08-27)

A unicode sweep — accents, Greek, CJK, an emoji's surrogate pair, `codePointAt`,
`toUpperCase` on non-ASCII, `chars()` versus `codePoints()`, regex over wide
characters, sorting, `String.format` widths, `"ß".toUpperCase()` — agreed with a
JDK **byte for byte, everywhere**. The gap was next door: the byte round trip.

`getBytes()` was the whole of what caturra had. `java.nio.charset` did not
exist, `new String(byte[])` was "no String constructor takes byte[]", and the
`Files` readers took no charset — so the round trip every file-handling program
writes stopped halfway.

A `Charset` is a type now, carrying its canonical NAME and nothing else, which
is all `getBytes`, `new String(bytes, …)` and its own `toString` need. The six
standard charsets encode exactly as a JDK's do, down to the `?` an unmappable
character becomes in the byte charsets and the BOM that `UTF-16` writes and the
other two spellings do not. Decoding never throws: a malformed byte is U+FFFD,
which is what a program reading a file relies on.

The failures are the JDK's own, and they differ by how the program NAMED the
charset: as text it is the checked `UnsupportedEncodingException` — which
`String.getBytes(String)` and `new String(bytes, String)` now declare, so a
program that ignores it is refused exactly as javac refuses it — and through
`Charset.forName` it is the unchecked `UnsupportedCharsetException`.

`Files.readString`/`readAllLines`/`lines`/`write`/`writeString` take a trailing
charset. The virtual filesystem stores TEXT, so the charset picks no bytes
there; it is evaluated (an unknown name still fails) and dropped. A file
written in one charset and read back in another is the one shape this does not
model, and no program that uses a single charset can tell. `deleteIfExists`
joined them, since a program that writes a file tidies up after itself.

One correction the compat page caught within the hour: the checked exception
was keyed on the ARITY of `getBytes`, so `getBytes(StandardCharsets.UTF_8)` —
which cannot name a charset that does not exist — was refused for it. The
ARGUMENT is what tells the two apart, in either spelling of the constant.

Pinned by `text_to_bytes_and_back`, `the_file_readers_take_a_charset`,
`naming_a_charset_as_text_is_checked` and `only_a_named_charset_is_checked`;
the compatibility page gained a "Text to bytes and back" claim, recorded
against a real JDK like the rest.

### Pattern and Matcher (2026-08-27)

`java.util.regex.Pattern` was refused outright — "the class library covers the
AP CS A subset" — while the ENGINE behind `String.matches`, `replaceAll` and
`split` was caturra's own, and had every part a matcher needs: a search from an
offset, the group spans it captured, named groups, and the syntax errors. What
was missing was the object API a program writes when it wants more than one
match: `while (m.find()) { … m.group(1) … }`.

A `Pattern` keeps its source as WRITTEN — which is what `pattern()` answers —
beside the flags it was compiled with. The two fold into the inline `(?ims)`
prefix the engine already reads, and `LITERAL` folds into the `\Q…\E` that
`Pattern.quote` produces, so a flag never has to be modelled twice. A malformed
pattern fails at `compile`, where a JDK reports it, rather than at the first
`find`.

A `Matcher` remembers its last match, because `group`, `start` and `end` all
read it: asking before there is one is Java's `IllegalStateException("No match
found")`, a group index past the count is `IndexOutOfBoundsException("No group
9")`, and a group that took no part in the match is `null` with a span of `-1`
— not an empty string. `find` resumes where the last match ended and advances
one unit past a zero-width match, which is the rule that keeps
`while (m.find())` from stalling.

Everything the probes asked matched a real JDK byte for byte on the first run,
which is what a shared engine buys: the only new code is the object around it.

Pinned by `pattern_and_matcher`; the compatibility page gained a "Pattern and
Matcher" claim (87 supported / 5 unsupported / 3 beyond-11).

### The whole of Pattern and Matcher (2026-08-27)

The regex objects shipped as a SUBSET: compile, matcher, find/group/start/end,
the flags, split and quote. Everything a program reaches for once one match is
not enough was missing — the region, the append/tail rewriting loop, the frozen
match, the Java 9 computed replacements, the predicates — so this unit is the
rest of the Java 11 API, method for method.

The **engine** grew a region and two bound modes. A `Matcher` now carries
`start`/`end` (what `region(a, b)` sets), whether its edges ANCHOR (the
default: `^` and `$` treat the region's edges as the input's) and whether they
are TRANSPARENT (lookaround and `\b` may read outside, though the match may
not). Every read, every anchor and every boundary asks the region rather than
the input, which is what makes an opaque region behave like a substring: over
`[3, 6)` of `"thecat here"`, `\bcat\b` matches, and with transparent bounds it
does not, because the `the` before it is visible again.

The same pass made `hitEnd()` and `requireEnd()` real rather than
approximations — the engine records them as it reads, so a `\b` at the end of
the input reports both (it had to look past the last character to decide the
boundary was there, and one more word character would take it away). That is
how an editor tells "no match" from "not yet".

`appendReplacement`/`appendTail` are the rewriting loop, expanding `$1` and
`${name}` exactly as the string replacements do — and the string replacements
are now written as that same loop, which fixed what they LEFT BEHIND: a JDK
resets the matcher first and leaves it wherever the scan stopped, so after
`replaceFirst` the next `find` answers the SECOND match and after `replaceAll`
it answers nothing. Rewriting the whole string in one pass and resetting had
made both start over.

A `MatchResult` is one match, frozen — a real type here, with its own
`ElemType`, so `results().map(MatchResult::group)` types. `toMatchResult()`
hands one out; `results()` is a LAZY stream over the matcher itself (a new
`StreamSource::Matches`), so `results().limit(1)` consumes one match and leaves
the rest to `find()`, as a JDK's does. A `Matcher` IS a `MatchResult` — it
implements the interface — and both answer `instanceof`, which needed the VM's
one class-name table to learn all three regex classes (a frozen match is a
JDK's `Matcher$ImmutableMatchResult`, which is what `getClass()` and a default
`toString` print).

`asPredicate()` and `asMatchPredicate()` are a native predicate object: no user
code runs, but `filter(pattern.asPredicate())` works because the interpreter's
functional-call path answers for it directly. `replaceAll(Function)` is the
other direction — the intrinsic layer cannot call user code, so the interpreter
drives the JDK's own loop over the native pieces (`reset`, `find`,
`appendReplacement`, `appendTail`), which is why the two forms cannot drift
apart.

Three defects fell out of the fixture rather than the feature. Casting an
erased `Object` DOWN to a regex object was "incompatible types", and worse, the
cast fell through to the UNBOXING arm and emitted `intValue()` on the object —
the shape that noticed was the `(MatchResult) __p0` a lambda over `results()`
is desugared into. A method reference through a library class the desugaring
does not know is not a class at all ("cannot find symbol: 'MatchResult'"). And
`PatternSyntaxException` had no accessors, though its message is BUILT from the
three they answer.

Which turned into a sweep of what a malformed pattern reports. A JDK's index is
the cursor MINUS one — at the end of the input that is the pattern's length,
and for a `)` in first position it is `-1`, which the message then leaves out
entirely. Seven of eighteen probes differed: the index for an unclosed
character class, the negative one, the wording for a truncated counted closure
(`a{2` is an unclosed closure, not an illegal repetition), for a trailing
backslash (a JDK reports its own `Unexpected internal error`), for a truncated
inline modifier, and a trailing `-` in a class (an illegal RANGE, not an
unclosed class).

Known gaps, both refused rather than answered wrongly: `CANON_EQ` and
`UNICODE_CHARACTER_CLASS` change what matches for ordinary text, and the engine
models neither, so `compile` says so where a JDK would have compiled;
`\p{...}` (Unicode character properties) is likewise unimplemented, and its
message is caturra's own rather than the JDK's `Unknown character property
name`.

Pinned by `a_matchers_region_and_bounds`, `the_append_and_tail_loop`,
`a_frozen_match_and_the_results_stream`, `a_patterns_own_predicates`,
`the_computed_replacements`, `what_the_last_attempt_learned`,
`a_pattern_syntax_exception_reports_where` and `a_malformed_pattern_says_where`.

### A library type, wherever a program puts one (2026-08-27)

A `Pattern` could be held in a local and not be a method's PARAMETER —
"unknown type 'Pattern'" about a type that had worked a line earlier. The
descriptor for a signature was built by a hand-written chain of `else if`s that
every new library type had to be written into a second time, and five had not
been. It asks the resolver now, which is the same answer every other position
gets, so the two cannot drift again.

That was one of eight defects a cross-product found: sixteen modelled library
types (a `Scanner`, a `Path`, a `Charset`, the regex trio, a `Random`, a
summary) against every position a program can put one in — parameter, return,
field, array, collection element, type argument, varargs, ternary, `var`,
lambda parameter, cast back from `Object`, `instanceof`. 26 of the first 64
programs diverged; all 100 agree now.

- **The cast back was refused for every one of them**, and worse, it fell
  through to the UNBOXING arm and emitted `intValue()` on the object. Both the
  cast rule and `instanceof` now ask ONE question — is this a library type
  modelled as a single class? — so neither can accept what the other refuses,
  and a checkcast is only emitted to a class the VM's namer knows.
- **An ARRAY of an interned element in a signature** never worked at all: the
  declaration said `rows([Ljava/util/ArrayList;)` and every call site asked for
  `rows([Ljava/lang/Object;)`, so the method could not be found. A
  `List<String>[]` parameter and a `Scanner...` varargs are the same bug.
- **One element kind per type.** A type ARGUMENT and a VALUE of the same type
  disagreed: `List<Class> l = List.of(x.getClass())` was "incompatible types:
  List<Class> cannot be converted to List<Class>", the tell of one fact in two
  shapes. And a library generic written RAW now converts to a parameterized one
  (`List<Optional> l = List.of(Optional.of("v"))`), the unchecked conversion
  javac allows and a user class's raw form already got.
- **A library object inside a COLLECTION printed as `object@2a`** while the
  same object printed directly said `x`. One `toString` written twice, and only
  one of them reachable from a list.
- **A bundled class that stands for a JDK one now carries the JDK's binary
  name**: `new Random(1).getClass().getName()` is `java.util.Random`, and a
  default `toString` prints it — while a program's own `Random` stays its own,
  since the library is not injected beside one. The diagnostics follow: javac
  prints the SIMPLE name for an imported class, so the message renderer strips
  a package the way it already stripped an enclosing class.
- **The VM names three more kinds**: a `Path` is a `sun.nio.fs.UnixPath` (the
  Unix-shaped filesystem whose separator caturra already prints), a `Charset` is
  a class per charset (`sun.nio.cs.UTF_8`), and a summary is a
  `java.util.IntSummaryStatistics` — which was also not importable, so a program
  could chain through one and never name it.

`java.nio.file.Path` gained its surface in the same pass, since the sweep could
not put one anywhere until it had one: `Path.of` is VARARGS (joining its
segments, dropping the empty ones), and `getRoot`/`getName`/`getNameCount`/
`isAbsolute`/`normalize`/`resolve`/`resolveSibling`/`relativize`/`startsWith`/
`endsWith`/`subpath`/`toAbsolutePath`/`toFile`/`compareTo`/`equals`/`hashCode`
all work on the NAME ELEMENTS rather than the text — `a/bc` does not start with
`a/b`, and `getParent` of a single name is null.

Known gaps, both honest refusals: a `Path` is `Iterable` in a JDK and
`for (Path part : p)` says so rather than iterating; and `new
IntSummaryStatistics()` cannot be constructed directly (only
`IntStream.summaryStatistics()` makes one).

Pinned by `a_library_type_in_every_position`, `the_path_surface`,
`a_bundled_class_reports_its_jdk_name`, `an_array_of_a_library_type_in_a_signature`,
`one_element_kind_per_type` and `a_library_object_inside_a_collection`; the
compatibility page gained a "Path" claim (89 supported / 5 unsupported /
3 beyond-11).

### The default methods a class inherits (2026-08-27)

A default method that implements a LIBRARY interface's own — `interface Shape
extends Comparable<Shape>` whose `compareTo(Shape)` is a default — left nothing
on the class answering `compareTo(Object)`, so sorting one threw
`ClassCastException: Sq cannot be cast to java.lang.Comparable`. javac puts
that erased bridge on the INTERFACE (an interface may carry one since Java 8);
caturra's bridge pass could not see the signature to bridge, because
`Comparable` and `Comparator` are synthesized straight into the method table
rather than parsed. It knows their two erased signatures now, and the bridge
lands where javac puts it.

That was the first of five findings from sweeping INTERFACES as a dimension —
defaults, statics, constants, diamonds, `Iface.super`, private interface
methods, an interface redeclaring `Object`'s methods, an enum implementing one,
a generic default, an abstract class implementing one.

- **A transitive interface was not implemented.** The check walked a class's
  superclasses and their DIRECT interfaces only, so a
  `class ByLength implements Weighted` where `interface Weighted extends
  Comparator<String>` was not a `Comparator`: its inherited `reversed()` was an
  unknown native member, for a default method every comparator has.
- **A default reached through the PARAMETERIZED face** of its interface —
  `Visitor<String> v = u; v.visit("x")` asks for `(String)Object` where the
  default is declared `(String)String` — found nothing. The class chain already
  matched a covariant override that way; the interface search demanded the
  descriptor exactly.
- **The defaults a user type inherits from an interface caturra synthesizes**
  rather than parses: `Iterable.forEach`, `Iterator.forEachRemaining`, and
  `Iterator.remove`, whose default THROWS `UnsupportedOperationException`. A
  lambda over a user `Iterable` had no target type ("only allowed where a
  functional-interface type is expected") and the call itself no symbol. The
  loop runs through the object's own `iterator()`/`hasNext()`/`next()`, which
  is what the JDK's default body does. A `Pattern`'s own predicate is not a
  user instance either, and it composes with `negate`/`and`/`or` now — the
  bundled composition classes, which is what the interface's defaults build.
- **A class that FIXES its `Iterable` argument** (`interface Sized extends
  Iterable<String>`) left `iterator()` answering `Iterator<T>` with nothing to
  substitute from, so a for-each over `this` was "incompatible types: T cannot
  be converted to String".
- **A natural comparison answered the SIGN, not the difference.**
  `Comparator.<String>naturalOrder().compare("ab", "cd")` is
  `"ab".compareTo("cd")`, which is -2; `Character`, `Byte` and `Short` subtract
  likewise. A sort reads only the sign, so this sorted correctly and answered
  wrongly to any program that printed the comparison.

Pinned by `a_default_method_implements_a_library_interface`,
`the_defaults_a_user_type_inherits`, `a_default_through_a_parameterized_face`
and `a_natural_comparison_answers_the_difference`; the compatibility page
gained an "Interfaces: default methods" claim (90 supported / 5 unsupported /
3 beyond-11).

### The two branching expressions (2026-08-27)

`switch` and `?:`, swept together. `switch` came back clean over every shape
asked — fallthrough, a `default` that is not last, an empty case, a constant
label, a `String` selector (including the NPE a null one throws), an `enum`
selector, a boxed `Integer`, and `continue`/`break` with a label jumping out of
one. The conditional did not.

Its numeric half was already exact — JLS §15.25's table, where `flag ? 'a' : 98`
is a `char` because the int is a constant that fits, `flag ? 1 : 2.0` promotes
to `double`, and unboxing a `null` branch throws. What was wrong was the JOIN
of two references:

- **Two collections that share no face** — a list and a set — fell all the way
  to `Object`, where both are a `Collection<E>`; assigning one was then refused.
- **A `String` and a `StringBuilder`** join at `CharSequence`, an interface
  neither of them wears as a face here.
- **Two classes that share SEVERAL interfaces** have no single join at all:
  javac's type is the INTERSECTION of everything both branches have in common,
  and caturra's join has to pick one. So `Drawable d = flag ? sq : ci` was
  "incompatible types: Shape cannot be converted to Drawable", for a program
  javac compiles.

The last one is not a join that can be computed — the answer depends on where
the value is going. So a conditional now ADOPTS its target wherever the target
is known: a local declaration, an assignment, a return, an array store, a field
initializer, a builtin parameter, and recursively through a NESTED conditional
(whose own join is the one type the intersection had to give up). The same
shape a DIAMOND already had, for the same reason.

An ARGUMENT to a user method is the one position left, and it is left
deliberately: the argument's type is what CHOOSES the overload, so there is no
parameter to adopt yet. It is written down under **Divergences from javac**.

Pinned by `the_type_of_a_conditional`, `a_conditional_adopts_its_target`,
`what_a_switch_does` and
`stricter_a_conditional_as_an_argument_needs_one_shared_type`; the
compatibility page gained a "The type of a conditional" claim (91 supported /
5 unsupported / 3 beyond-11).

### The enum-keyed collections (2026-08-27)

Enums themselves came back clean over eight probes — constants with fields and
constructors, constant-specific bodies, an enum implementing an interface, a
static initializer, `values()` handing back a FRESH array each call,
`valueOf`'s exception, `ordinal`/`name`/`compareTo`, an enum as a `HashMap` key
and in a `TreeSet`, an overridden `toString`, and the empty enum. The one hole
was the pair that exists FOR enums, and which caturra refused by name:
`EnumMap` and `EnumSet`.

They are the sorted collections underneath. An enum's natural ordering is its
ordinal, so a `TreeMap` keyed by one already iterates in exactly the order a
JDK's `EnumMap` does — which is what the old refusal said, as an argument for
not modelling them. What it left out is that a program can SEE the difference
three ways, and each is small: the class the object reports (`java.util.EnumMap`,
`java.util.RegularEnumSet`), the null PROBE an enum collection tolerates
(`get(null)` is null and `contains(null)` false, where a TreeMap's compare
throws — only a null KEY still throws), and the METHODS: an EnumMap is a plain
`Map`, so `firstKey`/`headMap` must NOT be offered. Modelling the type as a
`Map`/`Set` and the object as the sorted one gives all three.

`EnumSet`'s factories need the enum's UNIVERSE — every constant, in order — and
the compiler is where that is known, so each call emits the enum's own
`values()` as its first argument. The VM walks those constants and keeps the
ones the call selects, which is why `noneOf` remembers nothing and
`complementOf` needs no reflection: `allOf`, `noneOf`, `of`, `range`
(inclusive), `complementOf` and `copyOf` are one loop over the universe with a
different test.

Pinned by `the_enum_keyed_collections`; the compatibility page gained an
"EnumMap and EnumSet" claim (92 supported / 5 unsupported / 3 beyond-11), and
the divergence list lost the bullet that named them.

### Four dimensions, and the escape that hid one (2026-08-28)

A sweep that found nothing is worth as much as one that finds something, but
only if it really ran. This one nearly did not.

`StringBuilder`, method by method — every `append` overload, `insert` in all
seven forms, `delete`/`deleteCharAt`/`replace`/`reverse`/`setCharAt`, the
searches, `compareTo`, `chars`, `codePointAt`, `subSequence`, appending a
builder TO ITSELF, a surrogate pair reversed, and the index error each method
throws — agreed with a real JDK everywhere. So did **initialization order**
(a constant is inlined and initializes nothing; a static field read triggers the
class; the static initializers and blocks run in source order; the instance ones
run before the constructor body; a superclass constructor calling an overridden
method sees the subclass's fields at their defaults; an interface's constant
initializes the interface and not its implementor; and a forward reference is
refused as javac refuses it), **deep recursion** (`StackOverflowError`, caught
and recovered from), and **arrays of arrays** (jagged rows are null, `clone` is
shallow, `deepToString`/`deepEquals`, and the `ArrayStoreException` a covariant
array throws, message included).

What the first of those found instead was a hole in the HARNESS. A program may
print any character, including a control one — `setLength` pads with NUL, and
`(char) 7` is an ordinary value — and `compatrun` hands its output to the
differential runner as JSON through a hand-written escaper that knew only
`"`, `\`, `\n`, `\r` and `\t`. An unescaped control character made that JSON
unparseable, so every reader of it reported "printed no JSON": a program that
could not be compared AT ALL, reported as neither a match nor a difference. The
playground was never affected (the WASM boundary serializes properly); the
thing that checks caturra against a JDK was.

Pinned by `the_string_builder_surface`, `when_a_class_is_initialized`,
`arrays_of_arrays_and_covariance` and `a_program_may_print_a_control_character`
— the last one being a program that could not have been pinned before.

### Auditing the refusals (2026-08-28)

The last unit found a harness hole that made programs unverifiable; this one
asks the same question of the pins themselves. All 694 `differential_test!`
programs were extracted, compiled with javac and RUN: every one compiles, and
every one prints something — no pin passes by comparing nothing. Then the 268
`differential_reject!` programs, which assert only that BOTH engines refuse:
javac's first error was captured for each and set beside caturra's.

Two of them were passing for the wrong reason. `a_reabstracted_method_must_be_
implemented` and `unrelated_defaults_still_conflict` each wrote `return 'A';`
where the method returns `String`, so javac stopped at the TYPO and never
reached the rule the test is named for — the pin would have held even if javac
had no such rule. Both are fixed, and javac now refuses them for the rule.

Comparing the two engines' wording across the rest (171 of 268 were already
byte-identical) found four messages that name the wrong cause, and one that
was not an error at all:

- **`List<int>`** was "cannot find symbol: class List" — about a class the
  program used correctly. javac: "unexpected type / required: reference /
  found: int".
- **A private field of a superclass**, read or assigned by simple name or
  through a receiver, was "cannot find symbol: variable secret". The field IS
  there; what the program cannot do is SEE it, which is what javac says:
  "secret has private access in Base".
- **`s++` on a String** was caturra's own "++/-- needs a numeric variable";
  javac names the operand type and the operator: "bad operand type String for
  unary operator '++'".
- **An integer literal too large** was "integer literal '9223372036854775808'
  is out of range"; javac says "integer number too large" and lets its caret
  point at the literal.
- **Assigning a FINAL variable** has four javac messages, one per kind — a
  local ("cannot assign a value to final variable x"), a parameter ("final
  parameter p may not be assigned"), a multi-catch parameter ("multi-catch
  parameter e may not be assigned") and a for-each variable ("variable s might
  already have been assigned"). caturra had one wording for all of them, and
  **the for-each case was not an error at all**: the parser dropped the `final`
  from `for (final String s : xs)`, so assigning it compiled here and fails on
  a JDK. That is the direction that must never be wrong, and it was found by
  asking what javac says rather than what caturra does.

Pinned by `a_final_for_each_variable_cannot_be_assigned` and
`a_final_parameter_cannot_be_assigned`; 176 of the 268 reject pins now match
javac's first message byte for byte.

### The import a program never wrote (2026-08-28)

The audit continued into the OTHER pin family, and this time it found the
engine, not the pins. All 301 Java sources embedded in `run_programs.rs` — the
tests that assert caturra's own output with no JDK to check them against — were
extracted and run on both engines: 201 identical, 4 explained (an unseeded
`Math.random`, a directory left behind by an earlier run, caturra's smaller
heap, and the known HashMap treeify gap), 65 that a headless JDK cannot run at
all (Swing), and 31 javac cannot compile (org.code, JUnit, EasyMock — and one
placeholder the Rust test substitutes into).

One of those 31 was neither: a Swing program using `DefaultTableModel` under
`import javax.swing.*`. javac refuses it — the class is in `javax.swing.table`
— and caturra compiled it. Pulling on that found the general hole.

**Every library name outside `java.lang` needs an import**, and caturra was
enforcing that for barely half of them:

- The use-check only looked at TYPE positions, so a class used as a STATIC
  RECEIVER was never checked at all: `Arrays.sort(x)`, `Collections.reverse(l)`,
  `Objects.equals(a, b)`, `IntStream.of(1)`, and a dotted constant like
  `Locale.US` or `StandardCharsets.UTF_8` all compiled with no import. This is
  the most ordinary shape in the whole audit — a student's file that works in
  the playground and fails on a real JDK.
- Thirty-seven names were missing from the list that requires an import at all
  (`Optional`, `Random`, `Pattern`, `Charset`, the reflection types, the format
  exceptions), and every primitive functional interface (`IntPredicate` and its
  thirty-four relatives).
- A BUNDLED class shadowed the rule entirely: `user_classes` — the set that
  lets a program's own `Scanner` shadow the library one — was built from every
  compilation unit including the injected libraries, so `Random` and
  `StringJoiner` counted as the program's own classes and needed no import.
- `javax.swing.table` and `javax.swing.tree` were folded into `javax.swing`, so
  the wildcard provided what it must not and the import that really provides
  them ("package javax.swing.table does not exist") was refused.

Ten of caturra's own Swing tests were written under the wildcard alone. They
are valid Java now.

Pinned by `a_static_receiver_needs_its_import`, `a_dotted_constant_needs_its_import`,
`a_bundled_class_needs_its_import_too`, `the_swing_table_package_is_its_own`
and `the_imports_that_provide_a_name`.

### A name the program is free to take (2026-08-28)

Thirty-three of the 2698 corpus levels do not compile here, and the last unit's
audit habit says to ask WHY rather than to assume they are the known gaps.
Eight of them were one bug, and it belongs to caturra: a level whose student
class is named `Character` — a play's cast, in a unit about constructors —
failed with "cannot find symbol: method compare(char,char) in class
Character", about a call the level does not contain.

The call is in caturra's own bundled `Arrays`: `compare(char[], char[])` reads
`Character.compare(a[i], b[i])`, and the bundled libraries are compiled in the
same flat namespace as the program, so the program's class captured it. Every
bundled reference to a wrapper — `Integer.compare`, `Double.compare`,
`Long.hashCode`, `Math.max`, `String.valueOf` — was the same latent bug waiting
for a program to take that name.

Writing them fully qualified does not fix it: a qualified receiver is stripped
to its simple name before the lookup, so `java.lang.Integer.compare` finds the
program's `Integer` too — which is also a bug in its own right, since javac
reads a fully-qualified name as the library class whatever else is in scope.

Both are fixed by one reserved spelling. `__Integer`, `__Character`, `__Math`
and the rest resolve to the library's statics and constants, and the `__`
prefix is one a program cannot write. The bundled sources use it; and a
qualified name whose simple form the program has SHADOWED is rewritten to it,
so `java.lang.Character.isLetter('x')` means what javac says it means even in a
program with its own `Character`.

Eight corpus levels compile now (41 compile failures to 33), and the corpus
comparison covers 1761 per-test verdicts with 4 divergences, all four on
validators that do not pin their own verdict down.

Pinned by `a_program_may_shadow_a_library_class`.

### What the corpus cannot compile (2026-08-28)

Thirty-three levels of 2698 fail to compile here, and this asks the same
question of each: does javac accept it? **javac rejects all thirty-three** —
they are starter code with the exercise still to do, FRQ excerpts that omit
their imports, and one file whose public class disagrees with its name. There
is no compile gap left in the corpus.

So the question becomes whether caturra says the SAME THING about them, and
comparing the two engines' first message found the last unit's kind of defect
again — a diagnostic naming a member that does not exist:

> constructor RealEstate.RealEstate() is not applicable

`RealEstate` declares one constructor, taking a `Home[]`. The phantom is the
SUPERCLASS's, and a constructor is not inherited (JLS §8.8) — which means
caturra was not merely describing it wrongly: `new Sub()` **compiled** for a
subclass that declares only `Sub(int)`, whenever the superclass had a
no-argument constructor. A program that compiles here and fails on a JDK, found
through the wording of an error about something else.

Three messages were also brought to javac's: the static-context one lost a
parenthetical caturra had added ("(instance methods arrive with objects)"), and
the import check — new in the last unit — now uses the three-line
symbol/location block every other "cannot find symbol" here already used.

Twenty-five of the thirty-three now match javac's first message exactly. The
eight that remain are the parser's, which is deliberately more explicit
("expected ';' to end the statement" where javac says "';' expected").

Pinned by `a_constructor_is_not_inherited` and
`the_constructors_a_class_declares`.

### What the grader says, not just what it decides (2026-08-29)

The corpus sweep compares VERDICTS. Its own README has said for months that
this is half the check — "it has MISSED bugs the hand-written tests caught,
because it only compares verdicts and not message text" — and both sides record
the message. So this unit compared them: 88 tests that both engines FAIL, and
25 messages that differ. Twenty-three are the levels whose validators use
unseeded `Math.random()`, where the reference disagrees with itself between
runs. Two were real, and they were the same bug:

> Exception while calling hasWatchedMinimum(): Should return true for 31
> seconds ==> expected: `<true>` but was: `<false>`

Code.org's validators wrap their assertions in
`try { … } catch (Exception e) { fail("Exception while calling m(): " + …); }`.
On real JUnit that catch does NOT run: `AssertionFailedError` extends
`java.lang.AssertionError` extends **`Error`**. caturra's bundled assertions
threw a `RuntimeException`, so the wrapper swallowed every assertion failure
and re-reported it as an exception. The verdict was the same either way, which
is exactly why comparing verdicts alone could not see it — but the student read
a different sentence, and a validator that recovers from a caught exception
could have read a different VERDICT.

`java.lang.AssertionError` is a modelled throwable now (under `Error`), the
eighty-five assertion throws raise it, and comparing the wording against real
JUnit found four more: the plain `assertEquals` overloads said "expected 1 but
was 2" where JUnit says "expected: `<1>` but was: `<2>`", `assertNotEquals`
likewise, and a `char` pair widened to `int` — "expected: `<97>`" for `'a'`
— because JUnit's `char` overload was missing.

The sweep compares messages from now on: `compare.py` prints a
`per-test FAIL messages` section beside the verdicts, with the same
unseeded-random adjudication. It reads **65 identical, 23 unseeded, 0
divergent** on the JUnit half and **12 / 8 / 0** on the org.code half.

Pinned by `diff_junit_assertion_failure_is_an_error`, which runs the same
validator on real JUnit 5.

### What a level PRINTS, at corpus scale (2026-08-30)

The grading sweep asks how a level is GRADED. It has never asked what the
level's `main` writes to the console — and the console is what a student
actually reads. A solution that prints the wrong number still passes every test
that does not look at that number.

So this unit ran the other sweep: all 2698 staged levels, `javac`/`java` beside
caturra, same stdin (none) and the same data files staged into both, comparing
the console byte for byte. **A failing level is compared too**, which matters
more than it sounds: with no keyboard, every `Scanner`-driven level ends in a
`NoSuchElementException`, and skipping the levels that do not complete would
have left the single largest category unchecked. The exception line a program
dies on is console text, so it is compared as console text — and it agreed,
class and message, on all 71.

The result is **0 unexplained**. Sixty-four levels differ, in two families the
sweep names rather than counts:

- **29 draw randomness.** `Math.random()` in the SOLUTION, not just in the
  test — a distinction `compare.py` does not have to make, because a verdict
  only depends on the validator. One of them (`U6L2-L8d`) indexes a 2-D table
  with `(int)(Math.random() * (rows * cols))`, so on a JDK it prints a word,
  or throws `ArrayIndexOutOfBoundsException` at a different index, run to run.
- **35 print a reflective listing.** `getDeclaredConstructors()` returns the
  same constructors in a different order. This was already known and is now
  measured rather than asserted: on OpenJDK 11 a class declaring
  `[(), (int), (String), (int,int)]` reports them as
  `[(), (int,int), (int), (String)]`, and methods `[alpha, beta, gamma, delta]`
  come back as `[delta, alpha, beta, gamma]` — not declaration order, not its
  reverse, and not constant-pool order either (`javap` shows the pool holds
  them as written). It is HotSpot's internal method array, ordered by symbol
  address: stable for a given class on a given build, and reproducible by no
  rule a compiler could implement. The JDK documents it as "not in any
  particular order"; caturra returns declaration order and continues to decline
  to imitate the rest.

The sweep is checked in as `scripts/sweep/stdout.py`, next to the grading one,
and it adjudicates both families itself so the number a reader watches is the
UNEXPLAINED count — the same shape `compare.py` settled on. `compatrun` grew
the argument it needed to be usable for this: a corpus level is several files,
and passing only the one with `main` reported every other class as "cannot find
symbol" — a harness failure that reads exactly like an engine failure.

### A name is read where it is written (2026-08-30)

Two probes into ordinary-but-unswept program SHAPES — a generic container with
its own cursor, two helper classes each with a `Node` — turned up one cluster,
and its first member is the worst kind of finding: **`new Node()` inside `B`
built `A`'s `Node`**. No diagnostic, the right class one line away
(`new B.Node()` was correct), and the program printed `A.Node` twice.

caturra hoists every nested class to one flat namespace and keys the table by
BINARY name (`Main$B$Node`), with an alias per source spelling. The alias table
holds one entry per spelling, so the SIMPLE name `Node` could only point at
whichever class registered first. A simple name is resolved in the scope it is
written in (JLS §6.5.5.1): the table now records every class answering to a
simple name, each pass announces the class whose body it is resolving, and the
lookup takes the candidate whose enclosing class most closely encloses that
scope. A synthesized class — a lambda, a method reference, an anonymous body —
announces its ENCLOSING class instead of its own hoisted name, because that is
where its code was written; giving it its own name made a lambda implement a
different interface from the one its target resolved to.

The rule cuts the other way too: a MEMBER type shadows a top-level class of
the same name, so `new Outside()` inside `C` is `C.Outside` — which means the
scoped lookup has to run BEFORE the direct one, not after it.

Three more members of the same cluster, each the same fact read somewhere
else:

- **The qualifier was DROPPED.** `A.Node.K` resolved by taking `[A, Node, K]`
  to `[Node, K]` — "nested types are flattened to their simple names, so the
  qualifier carries nothing" — which is true only while no two of them share a
  name. The static-field path and the static-call path each did it, so
  `A.Node.K + B.Node.K` printed one constant twice and `B.Node.get()` ran `A`'s
  method. The pair now resolves to the one class it names, by binary name.
- **The lambda pass keeps its own map**, name to single abstract method, with
  the same one-entry-per-spelling problem: a lambda inside `B` was typed
  against `A`'s interface and refused as "cannot be converted to Go". Its
  target is resolved in its own scope now. The name RETURNED is still the one
  the program wrote wherever both spellings name the same interface — the
  spelling every later pass already handles — and only the ambiguous case
  takes the qualified one.
- **An anonymous class inside an INNER class could not see the outer
  instance.** The capture walk stopped at the inner class, whose own fields are
  usually none, so `new Runnable() { … outerField … }` inside
  `class Outer { class View { … } }` was "cannot find symbol" — while `View`'s
  own methods read the same field. The walk continues through a declared inner
  class now, and codegen's existing multi-hop `__caturraOuter` chain does the
  rest.

**The other half of the cluster is type variables in a hoisted body.** A class
or method type parameter is in scope in an anonymous or local class written
there, and hoisting lost sight of it: `new Comparator<T>() {…}` inside a
`Box<T>` was "cannot find symbol: class T", `new Iterator<T>() {…}` was
"incompatible types: T cannot be converted to an unsupported type", and a
`class Pick { T of(List<T> from) {…} }` inside a generic method was refused the
same way. Every one of them is an ordinary way to write a container. The
parser now records the type parameters in scope where the class was WRITTEN,
and they erase to their bounds in its body — which is exactly what javac
compiles them to, since the hoisted class holds no type argument of its own.

One thing was measured and left open. A class the program declares under a
LIBRARY type's name makes the fully qualified spelling unusable:
`java.util.List` reads as the program's own `List` and is refused. javac
resolves a qualified name without consulting what is in scope. It is stricter
than javac — the safe direction — and the library-type resolution consults
"does the program declare this name?" in a dozen places, so the fix is a
larger change than the shape deserves. Pinned by
`strict_a_qualified_library_name_a_class_shadows`.

Pinned by `a_simple_name_resolves_in_its_own_class`,
`a_qualified_nested_type_names_that_one`,
`an_enclosing_type_variable_in_a_hoisted_body`,
`a_lambda_target_resolves_in_its_own_class` and
`a_hoisted_body_inside_an_inner_class`.

### A program is several files (2026-08-30)

Both differential harnesses could only ask about ONE file. `scripts/fuzz/run.py`
took a case as a single `.java`, and `differential_test_files!` staged DATA
beside the program but compiled only the program itself — while javac, given a
directory, finds the rest on its own sourcepath. So every check of what a name
MEANS was a check inside one file, and the scoping cluster above is precisely
about a name meaning different things in different places.

Both take several sources now: a fuzz case may be a DIRECTORY named for the
class holding `main`, and a staged file ending in `.java` is compiled as a
source rather than written into the virtual filesystem. Nine multi-file
programs — a top-level `Node` in one file against a nested one in another, two
nested interfaces of one name in two files, an inner class instantiated across
a file boundary, mutually recursive statics, a generic interface implemented in
another file, an anonymous subclass of an abstract class declared elsewhere —
agree with a real JDK, which is the answer this dimension had never been asked
for. Pinned by `a_name_means_what_its_own_file_says`.

### The array a function makes (2026-08-30)

Walking the method-reference taxonomy — static, bound, unbound, constructor,
array-constructor, `super::`, a reference to a varargs method — found one
member of it that did not work, and pulling on it found a rule underneath.

**`IntFunction<String[]>` did not answer its type argument.** It is what
`String[]::new` is written as; the interface exists for almost nothing else. A
functional interface's result is its LAST type argument, and the list of which
interfaces have one covered `Supplier`, `Function`, `BiFunction`,
`UnaryOperator` and `BinaryOperator` but not the primitive-INPUT
specializations — so `maker.apply(3).length` was "cannot find symbol: variable
length" while `Function<Integer, String[]>` one line above worked.
`IntFunction`, `LongFunction` and `DoubleFunction` answer their argument now.

**The multi-dimensional array-constructor reference built the wrong shape.**
`String[][]::new` is `n -> new String[n][]`, and the desugar wrote it by
NESTING the element (`elem: String[]`, one dimension) where every reader of an
array creation expects the base element and one entry per dimension, the
leading one sized. The same array, written a way nothing downstream reads: the
result typed as `Object[]`, so assigning it to a `String[][]` was refused. The
lambda form of the very same expression compiled, which is what made it look
like a rule about references.

**And an array DIMENSION did not unbox.** `new String[count]` for an
`Integer count` — or for a `list.get(0)`, or a `map.get(k)` — is ordinary Java
(JLS §15.10.1: the dimension undergoes unary numeric promotion) and was refused
outright, as were `byte` and `short` dimensions. This is the wider rule the
`IntFunction` probe walked into: the check asked for `int` or `char` and
nothing else. A `null` dimension now throws the NPE the unboxing implies, and a
negative one the `NegativeArraySizeException` with the JDK's message.

Pinned by `an_array_dimension_promotes` and
`a_primitive_input_function_answers_its_argument`.

### A container inside a type argument (2026-08-30)

A cross-product of forty cells — every parameterized library type, its argument
read back through a member — found two, and pulling on them opened a family.

**An `Optional` could not hold a collection.** `Optional<List<Pet>> pets =
Optional.of(new ArrayList<>(…))` was refused outright: "Optional.of cannot hold
ArrayList<Pet>". So was an array, and so was another `Optional`. The element of
a library container is read by a function that answers only for FLAT types, and
a whole container has no flat element — but caturra has held one for years, as
the interned NESTED type an array of collections already uses. `Map.entry(k, v)`
had the same hole and collapsed both halves to `Object`. Reading the argument
back is one helper now, used by the emit path and by `type_of` alike — the two
were separately wrong, which is how this family stays alive.

**A nested element widens.** The value is written as the CLASS
(`Optional.of(new ArrayList<>(…))`) and the variable declared as the interface
(`Optional<List<Pet>>`); javac reads that through inference, and the answer is
the one the face rule already gives a level up. `Stream` was missing from the
widening chain entirely, so `Stream<List<Pet>> s = Stream.of(new
ArrayList<>(…))` was "incompatible types" for the same reason.

**What a lambda ANSWERS**, when the answer is not a library scalar, was three
separate gaps:

- **An instance method of a USER class.** `pets.stream().map(p -> p.name())` is
  as ordinary as a stream gets, and the mapped element was `Object` — the
  method shapes were consulted for an implicit `this` receiver and for library
  types, and for nothing else. (Asked AFTER the library table: caturra's own
  bundled `Optional.get()` answers an erased `Object` the real one does not.)
- **An `Optional`'s value**, so `map(Optional::get)` over a
  `Stream<Optional<Pet>>` keeps the `Pet`.
- **The stream a `flatMap` is given.** `flatMap` had no element rule at all in
  codegen — the operation that exists to flatten produced a stream of `Object`,
  whether the lambda answered `inner.stream()`, `Stream.of(…)` or
  `Arrays.stream(row)`. The lambda pass had computed the element correctly all
  along; nothing carried it across.

`Optional.stream()` (Java 9) is modelled now as well — a stream of at most one
value, which is what makes `flatMap(Optional::stream)` the idiom it is.

One thing was measured and left open here — `Optional.empty()` answering an
Optional with no element, so a CHAINED `Optional.<String>empty().orElse(x)` had
nothing to adopt and the explicit type witness was not read on this factory —
and it is CLOSED now; see "The negative direction, over the new surface" below.
The judgement that threading the witness through the static-call path was
"larger than the shape deserves" was wrong twice over: it is one field carried
the way `void_target` already is, and the shape turned up in an ordinary
negative-probe program rather than being hypothetical.

Pinned by `a_container_inside_a_type_argument` and `what_a_lambda_answers`.

### What a pipeline answers (2026-08-30)

Thirty-six cells, one per stream operation, each calling a method on what the
operation ANSWERS so an erased element shows. Four failed, and every one was a
missing piece rather than a wrong one:

- **`Collectors.toCollection` did not exist.** It is how a stream is gathered
  into anything but the default `ArrayList`/`HashSet` — a `TreeSet`, a
  `LinkedList`, an `ArrayDeque` — and the supplier argument was refused as
  "only allowed where a functional-interface type is expected". Its result type
  is the collection the SUPPLIER builds, holding the stream's element, so
  `collect(toCollection(TreeSet::new)).first()` reads as a sorted set. (The
  gathering needed the one thing a collection's kind is not known at compile
  time for: an add that dispatches on the object at run time.)
- **`partitioningBy(predicate, downstream)` was missing** beside a `groupingBy`
  that has had its two-argument form all along.
- **`mapping(f, downstream)` handed the downstream the STREAM's element**
  rather than what `f` answers, so `groupingBy(k, mapping(Word::text,
  toList()))` read as a map of Words and the `String` method after it was
  "cannot find symbol".
- **`Stream.iterator()`** — the terminal that hands a pipeline to a loop — was
  not modelled at all.

A fifth cell exposed a MESSAGE rather than a gap. `PrimitiveIterator.OfInt` is
what a primitive stream's `iterator()` answers, and caturra does not model it;
the qualifier read as a package, so the program was told "package
PrimitiveIterator does not exist" about a type `java.util` really has. A nested
type of a class on the known-unsupported list now says so. Pinned by
`strict_a_primitive_stream_cursor`.

Pinned by `what_a_pipeline_answers`.

### A generic class of one's own (2026-08-30)

The same probe shape turned on a generic class the PROGRAM declares — twenty-two
cells, each calling a method on what the class answers. Three failed, all the
same fact one layer apart:

**A method that answers its class's own type VARIABLE.** `Box<T>.get()` reads
as a `T`, and erasure has already replaced that with its positional sentinel —
which is exactly the index of the receiver's own argument. The lambda pass did
not make the substitution, so `boxes.stream().map(Box::get)` mapped to `Object`
while `boxes.get(0).get()` on the line above did not. (Codegen has done this
substitution all along; the lambda pass, which types what a lambda ANSWERS, had
its own half of the same rule missing.)

**A collection as a user generic's argument.** `GenBox<List<Tag>> box = new
GenBox<>(new ArrayList<>(…))` — the value written as the class, the variable
declared as the interface — was "incompatible types". The arm that keeps two
DIFFERENT parameterizations of one class apart (which is a real rule:
`Bag<String> b = bagOfIntegers` is an error, and accepting it once let a
program run with the wrong static type throughout) compared the two arguments
for EQUALITY. It asks `elem_matches` now, the same reading the builtin
collections have always used, so a widening argument passes and a mismatched
one is still refused — in javac's words, checked both ways.

Pinned by `a_user_generic_answers_its_argument`.

### What `var` infers (2026-08-30)

Thirty initializer shapes, each declared with `var` and then read back through
a method on what it inferred. One failed, and it took three readings to close.

**`var` says nothing on its own; the initializer does** — and the pass that
types lambdas kept the placeholder. So `var items = new ArrayList<>(…)`
recorded a local of type `var`, and the lambda in `items.stream().map(…)` had
no element to be typed against: refused as though the position were not a
functional-interface one, in a program whose every other line compiled.

Reading the initializer needed two more shapes the pass could not see:

- **A DIAMOND takes its argument from what it copies.** `new
  ArrayList<>(List.of(item))` is an `ArrayList<Item>`, and the pass read only
  the class.
- **The literal collection factories.** `List.of`, `Set.of` and
  `Arrays.asList` are how a collection is written inline. A LONE reference
  array SPREADS there — `Arrays.asList(Kind.values())` is a list of the
  constants, not a one-element list holding the array — and a primitive one
  does not, which the pinned enum test caught the moment the spread was
  written without that rule.

Pinned by `what_var_infers`.

### What a method answers (2026-08-30)

Twenty-five shapes a method may return — a bare object, an array, a
`Map<String, List<Leaf>>`, a `Stream`, a `Function`, a 2-D array — each read
back through a call on the answer. Two failed:

**A class of the program may declare a `stream()` of its own.** Every reader of
a stream's element knew the LIBRARY shapes and nothing else: a collection's
`stream()`, `Files.lines`, `IntStream.range`, `Stream.of`. A method of the
program says its element in its own return type, and the lambda after
`tree.stream()` had no target without it — refused, again, as though the
position were not a functional-interface one.

**A mapped element may be a whole CONTAINER.**
`trees.stream().map(Tree::leaves)` is a stream of lists, and the reader that
turns what a lambda produced into an element answers only for FLAT types — so
the element erased and `.get(0)` on it was "cannot find symbol". That reader
is the same one an `Optional` needed a container for, so both sides now share
it: a whole container is an element too, interned.

Pinned by `what_a_method_answers`.

### What a field holds (2026-08-30)

The last of the "read it back" surfaces: a field, and a call on what it holds.
Twenty cells, and three readings were missing — each of them refusing an
ordinary program with "a lambda is only allowed where a functional-interface
type is expected", the sentence that means the pass could not type the
receiver.

- **A field a class INHERITS.** The scope the lambda pass builds held the
  class's own fields and nothing above them, so `cards.stream().map(…)` inside
  a subclass of the class that declares `cards` had no element — while the
  identical method one class up compiled.
- **A field reached through ANOTHER object.** Only `this.cards` and the bare
  name were read; `deck.cards.stream()` had no type at all. (The parser keeps
  a dotted read as a NAME rather than a field access, so both spellings had to
  be answered.)
- **A field reached through a lambda's own PARAMETER** —
  `decks.stream().map(d -> d.cards.get(0))`. The general reader knows the
  pass's scope; only the parameter map knows what `d` is.

Pinned by `what_a_field_holds`.

### The other direction (2026-08-30)

Every unit above WIDENED what caturra accepts. Twenty programs that must still
be REFUSED — a wrong element inside a container's type argument, a collector
handed a String, an array constructor reference of the wrong component, an
array dimension that is a `double` or a `String`, a `var` with nothing to infer
from — were run against both engines. **All twenty are refused by both**, which
is what the loosening had to leave standing.

Seven now say what javac says, up from one. What was fixed is the wording of
the paths this session touched:

- An array DIMENSION reported "cannot be converted to int (array size)", a
  parenthetical of caturra's own. javac has two sentences here: a numeric
  dimension that does not fit is "possible lossy conversion from double to
  int", anything else "X cannot be converted to int" — and a BOXED value names
  the WRAPPER (`Long cannot be converted to int`), because unboxing followed by
  narrowing is not an assignment conversion at all.
- `var` with nothing to infer from said "cannot infer type for 'var' from this
  initializer". javac names the VARIABLE and puts the reason on a second,
  parenthesized line — a different reason for each rule met: no initializer,
  a `null` initializer, a lambda or method reference that needs a target type.

The rest are javac's INFERENCE messages ("inference variable T has incompatible
bounds"), which caturra does not model; it names the two concrete types
instead, which locates the same mistake.

The suite gained the pin family this needed. `differential_reject!` asserts
only the SHAPE — both engines said no — and a pin that cannot check the reason
passes for the wrong one, which an audit of these caught twice.
`differential_wording!` compares the first line of javac's first error against
caturra's, so a message that drifts is a failing test rather than a thing
someone notices later. The three programs above are pinned with it.

**One thing this session made looser than javac.** Generics are invariant:
`Optional<ArrayList<Pet>>` is not an `Optional<List<Pet>>`, and javac refuses
the assignment between two declared variables. caturra accepts it, because the
rule it needs — the value written as the CLASS where the variable says the
interface, which javac reaches by inference at the `Optional.of(…)` call — is
stated on the types alone, and the types cannot tell an inference site from an
assignment. The looseness cannot lose type safety (both sides erase to one
class, and every read through the target still answers the target's element).
Pinned by `loose_a_nested_argument_widens_between_variables`.

**And one the reader/collector session made (2026-09-04).** A `Collector`
carries what it GATHERS and not what it CONSUMES, so
`Collector<String, ?, String> c = Collectors.joining();` is accepted where
javac infers `Collector<CharSequence, ?, String>` and calls the assignment
incompatible. The element is the one type argument caturra's collectors erase;
what each gathers is still exactly its factory's, and every read through the
result answers the result's own type. Writing the element javac infers compiles
in both. Pinned by `a_collectors_element_is_unchecked` — recorded here the same
day it was noticed, because a looseness that is only observed and not counted
is what the pin-versus-bullet audit exists to catch.

### What the refusals SAY (2026-08-30)

`differential_reject!` asserts that both engines refuse a program. It has never
asserted WHY. With `differential_wording!` in hand, every one of the 236 pins
in that family was asked: does caturra refuse it in javac's words?

**179 already did.** Those are tightened to `differential_wording!` now — a
message that drifts is a failing test rather than something noticed later. Of
the 57 that differed, five were caturra's own wording being wrong rather than
merely fuller, and are fixed:

- **The lexer's four literal messages were two.** javac distinguishes an
  unclosed string, an unclosed character literal, an EMPTY one (`''`) and a
  newline inside one ("illegal line end in character literal"); caturra had
  "unterminated string literal" and "empty or unterminated character literal"
  for all four. A student searches the sentence.
- **An array of collections described itself as `Object[]`**, so a mismatch
  between two of them read "incompatible types: Object[] cannot be converted to
  Object[]" — a type that cannot convert to itself, which is the same
  no-information shape a nested ELEMENT was fixed out of once already. The
  element describes as what it is (raw, as javac prints an array's element),
  and the message is javac's: "List[] cannot be converted to Map[]".
- **A field on a PRIMITIVE** said "cannot find field 'lang' on int" — but there
  are no members to miss. javac says the receiver cannot be dereferenced, which
  is what a student meets when a variable obscures a package name
  (`int java = 3; java.lang.Math.abs(-4)`).

The remaining 50 are caturra saying MORE than javac, on purpose: javac's
sentence plus the reason it leaves for the following lines ("reference to f is
ambiguous: both method f(int[]) and method f(Integer[]) match"), or a concrete
type where javac names an inference variable. Those stay as
`differential_reject!` — the shape is what they pin.

### What the divergence pins do beyond compiling (2026-08-30)

The other two pin families, asked the same way. `stricter_than_javac!` says
caturra refuses what javac accepts, and says nothing about what the refusal
SAYS; `looser_than_javac!` says caturra accepts what javac refuses, compiles
the program and stops — where what the program then DOES is the part that
matters.

**Every looser program runs, and prints something sensible.** A permissiveness
that compiled and then died would be worse than the refusal it replaced; none
does.

**One strictness was no longer true, and is retired.** `list.toArray(gen)` for
an `IntFunction<String[]> gen` was refused because the generator overload was
modelled by REDUCING the written-out `String[]::new` to the array it makes,
which a variable holding the same function cannot be. It is the JDK's own
definition that closes it: `toArray(generator)` IS
`toArray(generator.apply(0))`, so the variable form rewrites the same way the
inline one always did — and this was the familiar shape of a defect here, a
program that compiles inline and is refused one line later through a name. The
pin is now `a_to_array_generator_through_a_variable`, which RUNS it.

The refusals the rest of the strictness list gives were read too. They name the
reason — "java.util.Vector is not supported by caturra (the class library
covers the AP CS A subset)", "Arrays.stream(array, from, to) exists in Java,
but caturra streams a whole array" — except one: a chained
`Optional.<String>empty().orElse(x)` still ends in "cannot find symbol: method
length() in class Object", because the empty Optional adopts a context it has
not got. The strictness is documented above; the message is not what it should
be, and is written down here so it is not mistaken for a missing method.

### The half of the repo the Rust gate never runs (2026-08-30)

`cargo test --workspace` is what every unit here is gated on, and it does not
run the TypeScript. `pnpm -r test` — the session layer, the sandbox RPC, the
compile API the playground calls — had two failing tests, and both were the
suite drifting rather than the engine:

- One asserted the wording "unterminated string literal", which this session
  changed to javac's "unclosed string literal" three commits ago. The Rust
  suite pinned the new wording; the TypeScript one still expected the old.
- One compiled `"hi".matches("h.")` to prove caturra reports unsupported Java
  in a friendly way — a program caturra has SUPPORTED since the regex engine
  landed, so the refusal it asserted stopped happening. The sample is
  `java.util.Vector` now, which is the scope limit itself rather than a feature
  that might arrive.

`pnpm lint` was failing too, on `specs/LANGUAGE.md`: prettier reflows every
bullet's continuation lines and rewrites `*emphasis*` as `_emphasis_`, 300
lines of churn through hand-wrapped prose. The log is in `.prettierignore` now,
and the whole of `pnpm lint` passes.

**Run both gates.** The wasm engine is rebuilt from this tree too — the
freshness check said it was five days behind, which is exactly the trap
`scripts/check-wasm-fresh.mjs` exists to catch.

### What the browser gate found (2026-08-30)

`pnpm test:e2e` — the third gate, a real browser driving the playground — had
24 failures. Five were message assertions this session's wording work made
stale, and the rest were older drift, but two were the product being wrong:

**Six Swing demo levels did not compile, and javac agrees.** "Table cell
renderer", "Sortable table", "Tree (JTree)", "Editable table", "Edit cells" and
"Custom table model" use `DefaultTableModel`, `DefaultTreeModel` and
`AbstractTableModel` under `import javax.swing.*` alone — those classes live in
`javax.swing.table` and `javax.swing.tree`, so the starter a student is handed
is invalid Java. They have the import they use now.

**A class NAMES its supertype, and the import check never looked there.**
`class PeopleModel extends AbstractTableModel` compiled here while javac
refused it — the walk covered fields, parameters, returns and bodies,
everything but the one position a class is written in. That is why the demo
shipped broken: caturra accepted what a JDK would not. Pinned by
`a_supertype_needs_its_import`, in javac's words.

**A watch expression named caturra's own class in its error.** The watch is
compiled as a synthetic `__CaturraWatch`, and javac's "cannot find symbol"
block ends with a `location:` line — so a typo in a watch read "location: class
CaturraWatch", an internal name the student never wrote. The watch keeps the
first line and the `symbol:` half, which is the useful one.

The last failure was a LEVEL defect rather than an engine one: the AP FRQ
BoxOfCandy validator assigns `boc.box`, a PRIVATE field of the student's class,
which javac refuses in exactly the words caturra now uses. The test asserted
that the level runs; it asserts what a student sees.

125 browser tests pass, `pnpm -r test` is 47/47, `pnpm lint` is clean, and the
compat manifest is re-recorded — its `javac` line no longer carries the temp
directory of the run that recorded it, which had been rewriting the file on
every re-record and hiding real changes in the churn.

### Publishing what now works (2026-08-30)

The compatibility page is where a capability becomes a claim someone can check,
and this session's work was not on it. Two features added, each a program the
recorder ran on a real JDK and on caturra and found byte-identical, and each
re-checked by `tests/compat_manifest.rs` and by the browser on every run:

- **A collection inside another type's argument** — an `Optional<List<String>>`
  read back as a list, a `Map.Entry<String, List<String>>`, a
  `Map<String, List<String>>`, an `Optional<int[]>`. All of it was refused
  outright a day ago.
- **Gathering a stream into any collection** —
  `collect(toCollection(TreeSet::new))`, `flatMap(List::stream)`,
  `stream().iterator()` driving a `while` loop, and `toArray(String[]::new)`.

94 supported features now, 5 gaps, 3 beyond Java 11.

### Composing the session's fixes (2026-08-30)

Twelve programs that use several of this session's repairs at once — a `var`
holding a builder chain, a stream from a user method, a collector into a
`TreeSet`, an `Optional` holding a list, an inner-class cursor, `toArray` with
a generator. Four failed, and all four were one missing reading:

**A `var` holding a BUILDER CHAIN.** `new Roster().add(a).add(b)` is a
`Roster`, and the reader that types a `var`'s initializer knew a `new`, a name
and a literal factory — not a call. So the chain had no type, and every lambda
after `roster.stream()` was refused for having no functional-interface
position, in a program whose declared-variable form compiles. A method of the
program answers its declared return now, and each link of a chain is named by
the one before it — recursing on the RECEIVER, which is what terminates the
walk at the `new`.

Nothing in the twelve was findable from any single feature: each program needed
two of them to meet. Pinned by `a_var_holding_a_builder_chain`, which is all
twelve in one.

### A second composition round (2026-08-30)

Twelve more programs, different ingredients — an abstract class and its
subclasses, a custom exception wrapping a cause, a navigable map keyed by a
`double`, a collector into a `TreeSet`, an array of arrays built from a
collection. One failed, and it was the other half of `var`:

**A lambda that CAPTURES a `var` local.** The capture becomes a field of the
synthesized class, and `var` is not a type there — so
`names.forEach(n -> seen.put(…))` for a `var seen = new TreeMap<>()` was
refused with "TreeMap<Double,String> cannot be converted to an unsupported
type", a message about the variable it had just read correctly. The capture
pass has no type table (it only rewrites), so it reads the shapes an
initializer SPELLS OUT — a `new`, a cast, an array creation — and leaves
anything else as it was.

Pinned by `a_lambda_capturing_a_var_local`.

### A program in the shape a level takes (2026-08-30)

Four programs written the way a corpus level is — a `Main` beside its helper
classes, each in its own FILE, now that both harnesses can hold more than one.
Three failed, on two more readings a class hands back:

- **An `Optional<E>` from the program's own method.**
  `shelf.longest().map(Book::title)` is how a class says "maybe one", and every
  arm of the Optional-element reader walked a LIBRARY chain. The program's own
  method says its element in its return type.
- **A STATIC FACTORY held in a `var`.** `var deck = Deck.fixed()` — the
  receiver is the CLASS rather than a value, and the reader that types a user
  method's return asked only "what class is this value?".

Pinned by `a_program_written_across_files`, which is all four in one.

### What a class hands back (2026-08-30)

One method per container a class can return — a `List`, a `Set`, a `Map`, a
`Map` of lists, an `Optional`, a `Stream`, an `Iterator`, an array, a `Deque` —
each with a lambda over it. Four of sixteen failed, and all four were the same
shape one type apart: **every reader of an element walks LIBRARY chains, and
reaches the program's own method only through an implicit `this`.**

- A **Map** from another object's method: `store.grouped().forEach((k, v) → …)`
  had no key or value.
- An **array** from one: `Arrays.stream(store.array())` had no element, while
  the same array through a variable worked.
- A **stream whose receiver is a static FACTORY** rather than a value:
  `Store.of().stream().filter(…)`. The reader types VALUES, so the chain
  stopped at the class name.

Each is now answered by one helper — what a method of the program returns, on a
receiver that is either a value whose class can be named or the class itself.
Pinned by `what_a_class_hands_back`.

### A program that reads its input (2026-08-30)

Ten programs in the shape a level actually takes — a `Scanner` over stdin,
`var` for every local, and the collections and streams the rest of this session
taught to carry their element. Two failed, and both were the other half of the
reading just added:

**A `var` holding a LIBRARY call.** `var numbers = in.nextLine().split(",")`
had no type at all: the reader that types an initializer had just learned the
PROGRAM's own methods, and a library call's answer is written on its receiver —
which is a reading the pass already held for lambda bodies and had never used
here.

**A `Scanner`'s own accessors were in no table.** `in.nextLine()` is the first
line of half the corpus's programs, and nothing said it answers a `String`. The
readers and the questions are one function now (`hasNext…` is a boolean,
`next…` the value).

Pinned by `a_program_that_reads_its_input`.

### `var` holding a library call (2026-08-30)

Eighteen cells: `var` holding each ordinary library call, then a use that needs
the type. Four failed, and the reason is in the table's history — the reader
that says what a library call answers grew for LAMBDA BODIES, where the answer
is usually a scalar (`s.length()`, `map.get(k)`). `var` asks the same table
about the VIEWS: a `subList`, a `keySet`, a `values`, a `toArray`, a stream's
`findFirst`. Each of those answers a container whose element is the receiver's,
and none of them was written down.

They are now — and a `Stream` takes its argument directly rather than through
the collection-element reader, which does not know it.

Pinned by `a_var_holding_a_library_call`.

### `java.lang.Enum` as a type a program writes (2026-08-30)

Twenty-four programs over what an enum is asked to do — constant bodies,
`EnumMap`/`EnumSet`, a `switch`, `values()`, an interface, a per-constant
`toString`, a stream. All but one agreed with a real JDK, and the one that did
not failed on a single word: `o instanceof Enum`.

`Enum` was modelled as a MECHANISM. The parser desugars an enum into an
ordinary class and synthesizes `name`, `ordinal`, `values`, `valueOf`,
`compareTo` and `getDeclaringClass` onto it, which is why every ordinary use
already worked. What none of that produced is a TYPE with the name `Enum`, so
the supertype every enum has could not be named: `Enum<?> e = Kind.TWO`,
`static <E extends Enum<E>> E last(E[] all)` — the standard way to write a
helper over any enum — `List<Enum<?>>`, `Enum::name`, and
`Enum.valueOf(Kind.class, s)` were each "cannot find symbol". This is the same
shape as `Map.Entry` before it: modelled as what something YIELDS, not as a
type a program may write down.

`Enum` is registered as an interface (there is no other shape here for a
supertype a class also has) and the desugar hangs it on every enum beside
`Comparable`. The two things a real CLASS would give it are refused by name,
with javac's own words: `extends Enum` is "classes cannot directly extend
java.lang.Enum", `implements Enum` is "interface expected here".
`Enum.valueOf(K.class, s)` reads the enum from the class literal at compile
time and IS `K.valueOf(s)` — the same rewrite `EnumSet.allOf` already does —
in the emitter and in `type_of`, since `var` asks the second one.

Making the scaffolding nameable exposed that it was VISIBLE. `Kind.class
.getInterfaces()` reported `[interface Marker, class Comparable]` where a JDK
reports `[interface Marker]`: an enum inherits `Comparable` from
`java.lang.Enum`, its SUPERCLASS, and declares neither. `getSuperclass()` said
`java.lang.Object`. Both are fixed at the source: an enum's class file no
longer records the two implicit supertypes at all, the VM answers
`getSuperclass` with `java.lang.Enum` for an enum, and `instanceof Enum` /
`instanceof Comparable` are answered from the class's enum-ness rather than
from its interface list.

Along the way, `o instanceof Nope` said "unknown type in instanceof" — a
message no JDK has, naming nothing. It is the ordinary unresolved-type report
now, `symbol:`/`location:` lines and all, which also means a namespace-only
class (`instanceof Math`) gives its honest reason instead.

Pinned by `enum_is_a_type_a_program_can_name`,
`an_enums_supertypes_are_the_ones_it_declares`,
`reject_extending_java_lang_enum`, `reject_implementing_java_lang_enum` and
`reject_instanceof_an_unknown_type`.

### What a `Class` object says about a type (2026-08-30)

The `Enum` unit left one thing open: a synthesized library interface, seen as a
`Class`, lied about itself. Pulling on it found that every question a `Class`
answers was read out of the class FILE — so a LIBRARY name, which has none,
got the default answer to all of them.

- `String.class.getInterfaces()` was `[]`. A JDK says
  `[Serializable, Comparable, CharSequence]`. Empty is a confident wrong
  answer — "implements nothing" — not a missing one, which is the same reason
  `library_superclass` exists. There is a table now, recorded from a real JDK
  11, of the interfaces each modelled library class DECLARES. That is not the
  question `library_faces` answers (which is transitive, and about what a value
  IS): a `PriorityQueue` declares only `Serializable`, and a `Stack` declares
  nothing at all — it gets its whole face from the `Vector` it extends.
- `Comparable.class.isInterface()` was false, and `System.out.println(
  Runnable.class)` printed `class java.lang.Runnable`. Three places computed
  interface-ness from the access flags and none had a fallback; they are one
  method now, which asks what is recorded when there is no class file.
- `getSuperclass()` on an INTERFACE answered `java.lang.Object`. A JDK answers
  `null`, and did for both halves — a program's own interface and a library's.
- A program's own class reported the COMPILER's spelling of what it implements:
  `class Comparable` rather than `interface java.lang.Comparable`, and
  `interface __Runnable` — an internal alias — for a plain `implements
  Runnable`. `getInterfaces` qualifies and un-aliases each name on the way out.

Pinned by `what_a_class_object_says_about_a_type`, which asks all nine
questions of fifteen types: the program's own class, interface, abstract class,
subclass and enum, and the library's `String`, `Integer`, `Object`, `int`,
`int[]`, `ArrayList`, `Map`, `Comparable`, `Runnable` and `Iterable`.

### Where a JDK checks the `Comparable` bound (2026-08-30)

Sorting a list of a class that forgot `implements Comparable` is textbook CSA
code. Asking, of every collection that orders by nature, whether caturra
refuses and throws exactly where a JDK does found two answers that were wrong
in opposite directions.

**`Arrays.sort` refused a program javac accepts.** The bundled `sort` took a
`Comparable[]`, so an array of a class with no order was a compile error. A JDK
declares `sort(Object[])` and casts each element as it compares, so that
program COMPILES and throws at run time — an NPE for an array of nulls (the
cast of a null succeeds; the compare is what fails) and a ClassCastException
for two real elements. The parameter is `Object[]` now, with the casts written
inside, and the strictness pin that recorded this as deliberate becomes an
ordinary agreement pin. The spec had already claimed the JDK behaviour two
thousand lines earlier — the claim and the code disagreed, and the code was
wrong. `Collections.sort`/`max`/`min`/`binarySearch` are the other rule: they
really are declared over `T extends Comparable<? super T>`, so their
compile-time refusal stands.

**A `PriorityQueue` accepted an element a JDK rejects.** Its `siftUpComparable`
casts the new element to `Comparable` BEFORE it looks for a parent to compare
with, so even the first `add` to a naturally-ordered queue throws. caturra only
ever compared, so a one-element queue of an unorderable class was built and
printed. It casts on offer now — and only there: a queue built FROM a
one-element collection heapifies without comparing (a JDK does not throw
either), a comparator-ordered queue never casts, and `null` is the queue's own
NPE. `TreeSet`/`TreeMap` were already right: they compare a key with itself on
the first insert, which is how a JDK type-checks it.

Pinned by `sorting_an_array_of_a_class_with_no_order` and
`a_priority_queue_casts_when_it_is_offered`.

**Measured and open: an INFERRED type argument's bound is not checked.** The
written form is (`Box<String>` for a `Box<T extends Number>` is refused, JLS
§4.5), a bare parameter is (`<T extends Number> void n(T)` with a String), and
a CONTAINER parameter is as of the entry below. What is not is a bound
satisfied only through inference:

- `<T extends Comparable<T>> T max(T x, T y)` called `max("a", 1)` — javac
  unifies the two arguments and reports "inference variable T has incompatible
  bounds"; caturra checks each against the bound's ERASURE and accepts.
- `<T extends Comparable<T>> void sort(List<T>)` called with an
  `ArrayList<Object>` — the same rule the library's `Collections.sort` already
  applies, which a user's own method does not.
- `<T> void pair(Map<T, T> m)` called with a `HashMap<String, Integer>` — one
  variable in two invariant positions.

caturra models a generic method's parameters by ERASURE rather than inferring a
binding, so all three are accepts-invalid. They are listed here because the
count of what is known is the point; see the divergence lists.

### A bound reaches a container parameter (2026-08-30)

The entry above listed three shapes where a type variable's bound went
unchecked. One of them is closed, and closing it is a lesson about which
direction to be wrong in.

`<T extends Number> double total(List<T> items)` erases its type argument to a
wildcard that accepts ANY element. That is right for an UNBOUNDED variable —
`<T> void dump(List<T>)` really does take a `List<String>`, and erasing the
argument to `Object` would make every generic method over a collection
unreachable — and the same wildcard was used for a bounded one, so
`total(new ArrayList<String>())` compiled. The erasure already carried the
bound (a `List<T>` READS its elements as `Number`, which is how the method's
own body types); only applicability ignored it. It does not now.

The first attempt refused valid code, which is the thing that must not happen:

    biggest(listOf(3, 9, 2), 0)   // <T extends Comparable<T>> T biggest(List<? extends T>, T)

`listOf` is `<T> List<T> listOf(T...)`, whose own variable nothing pins, so its
answer types as a list of the TOP TYPE — and an `Object` element does not widen
to `Comparable`. The call is perfectly legal Java: its element really is
`Integer`. So an element that is the top type, or is itself an erased
type-variable wildcard, satisfies any bound: an erasure that lost the element
cannot be evidence that the element is wrong. What that leaves accepted is
`sort(new ArrayList<Object>())`, where the program MEANT `Object` — javac
refuses it, and caturra cannot tell the two `Object`s apart. A false refusal of
valid code is the worse of the two, and this is the direction the erasure model
can be honest about.

The wording is javac's for the ordinary case and not for this one: where a type
variable is involved javac reports overload resolution failing ("method total
in class T cannot be applied to given types; … reason: inference variable T has
incompatible bounds") and caturra names the two collection types, as it already
does for `Arrays.copyOf` assigned to the wrong array type.

Pinned by `reject_a_bounded_container_parameter_given_another_element`,
`reject_a_bounded_set_parameter_given_another_element`, and — for the direction
that matters more — `bounded_generics_that_must_still_compile`, which is every
valid shape the first attempt broke.

### javac's phases, and a write through `this` (2026-08-30)

Every differential pin here compares the FIRST error, and `compatrun` reports
only that one — so how many errors caturra reports, and in what ORDER, had
never been measured against a JDK. Ten programs each wrong in three or four
ordinary ways, compared as a LIST, found that the lists mostly agree and that
where they do not, the reason is javac's PHASE structure.

**Flow analysis runs only after attribution succeeded.** A program with a type
error and a missing return is ONE error to javac — the type error — and was two
here, with the missing return FIRST, which is the one thing a student reads.
The granularity is the whole compilation, not the method or the class: a type
error in one nested class hides a missing return in its sibling. All three of
the flow phase's diagnostics behave that way (`missing return statement`,
`unreachable statement`, `variable x might not have been initialized`).

**Attribution runs only on a tree that parsed.** javac reports a file with a
syntax error using its syntax errors alone; caturra's parser recovers and
carried on, adding errors javac never prints — and which a student cannot act
on, because the real mistake is the one above them.

Both are now gates in `compile`. Measured on the same ten programs, the lists
that differed fell from five to two, and both remaining are the parser's
deliberately friendlier wording (`expected an expression` for `illegal start of
expression`), which is already recorded above.

**The same sweep found an accepts-invalid.** `static void g() { this.v = 1; }`
compiled and RAN. The static-context check lived on the READ path — evaluating
`Expr::This` — which an assignment TARGET never takes, so a write through a
receiver that does not exist went through. `this.v++` reached it by another
road and complained about the operand ("bad operand type an unknown type for
unary operator '++'") because `type_of` answers `Error` for a `this` that is
not there without saying why. Both say javac's sentence now.

Pulling on the increment path found the mirror gap: `Outer.this.count++`, a
plain statement, reported "++/-- as an expression works on variables and array
elements". The parser encodes a qualified `this` as a name path holding `this`,
and the increment's target dispatch knew paths of one and two segments only —
so the qualified form fell through to the expression path, beside an
`Outer.this.count += 1` that had always worked.

**Two message shapes now match javac.** A duplicate local names the MEMBER it
is in — `method go(String,int...)`, `constructor Pet(int)`, `static initializer
of class X` — where all three said "in this method"; a lambda body keeps the
old vague form, since javac attributes it to the member the lambda appears IN
and the hoisting loses that. And a for-each over something that is not iterable
gives javac's three lines (`required: array or java.lang.Iterable`), where
caturra listed what it happens to iterate: "an array, an ArrayList, or a map
view", which is neither what Java requires nor what the program wrote.

The measurement is checked in as `scripts/fuzz/diaglist.py`, over the new
`diagnostics` example, and the pin family `differential_error_count!` compares
the COUNT where the words are deliberately not javac's.

The compatibility page caught the first of these on its own, which is what it
is for: the Records card recorded caturra's reason as "missing return
statement" — a flow-phase answer to `record Point(int x, int y) {}`, and
nonsense to a reader — and now records "unknown type 'record'". One line of
`features.json` changed, and the browser test that verifies every claim went
green with it.

One more message stopped naming an internal class: writing to a
`List<? extends Number>` said "no suitable method found for add(...) in
java/util/ArrayList" — slashes and all — where the receiver's own type is what
a program wrote and what it now says.

Pinned by `a_type_error_hides_the_missing_return`,
`a_type_error_hides_the_missing_return_is_one_error`,
`a_syntax_error_stops_attribution`,
`reject_writing_a_field_through_this_in_a_static_method`,
`reject_incrementing_a_field_through_this_in_a_static_method`,
`writing_a_field_through_this`, `reject_for_each_over_a_map`, and the three
`reject_a_duplicate_local_names_*`.

### The rest of the diagnostic list (2026-08-30)

The list comparison from the entry above, pointed at the 236-program reject
corpus. Sixty-nine of them differed. The first thing that told me was about the
TOOL: nineteen were caturra's documented convention — javac's headline plus the
detail javac prints on its indented continuation lines, which a first-line
comparison can only see half of. `diaglist.py` reads a caturra line that starts
with javac's as agreement now, and so does `differential_wording!`, which was
asserting equality against a contract the spec already described.

That left fifty, and four mechanical causes account for most of the drop to
thirty-nine:

- **The same complaint, twice.** A desugaring that expands one construct into
  several — a try-with-resources becomes a body plus two `close()` calls —
  checked its resource in every copy, so one mistake was reported three times.
  Identical message at an identical position is now reported once, which javac
  never fails to do.
- **A syntax error AT a lexical one is that lexical one, told twice.** The
  parser is handed a token the lexer has already complained about and says what
  it cannot do with it: `int x = 1_;` was "illegal underscore" AND "expected an
  expression" pointing at the same `;`. A parse error whose span starts at or
  before a lexical error's end is dropped — but a LATER one is a separate
  mistake and stays, which javac confirms (`1_;` on one line and `int y = ;` on
  another is two errors to javac, and is two here).
- **A variable whose type was refused is still a variable.** javac gives it an
  error type and says nothing more; caturra left it undeclared, so
  `Pair<String> p = new Pair<>();` was "wrong number of type arguments" AND, on
  the next line, "cannot find symbol: 'p.k'" — a second complaint about a
  variable that is right there.
- **The names in a message are the names the program wrote.** Four said
  otherwise: a nested class by its BINARY name (`cannot inherit from final
  Outer$E`, `g() in Outer$C cannot implement g() in Outer$I`),
  `java.lang.Object` where javac writes `Object`, an abstract `super` call that
  named no class at all where javac writes "abstract method m() in Abs", and a
  map's entry view as `Map.Entry` inside a type argument where javac writes
  `Entry` — the simple name the arm beside it already used.

What remains is nineteen count differences and twenty wordings, and they are
not one thing: javac's own cascades where caturra says less (a covariant return
is reported twice by javac, once here), the generic-inference wording already
recorded above, and the parser's deliberately friendlier messages.

Pinned by `a_bad_resource_is_reported_once`, `a_lexical_error_is_not_told_twice`,
`a_later_syntax_error_survives_a_lexical_one`,
`a_variable_with_a_bad_type_is_still_declared`, and the five
`reject_*` naming pins.

The TypeScript gate caught something of its own while this ran: the RPC
transport test resolved two in-flight requests after ONE `setTimeout(0)`, and a
MessagePort delivery is not guaranteed to land in one turn — about one run in
three failed with "no gate registered for 2". It waits for the condition now,
not for a fixed number of turns.

### What a bad method reference says (2026-08-30)

Twelve wrong method references, compared with javac. Three matched. javac has
exactly two headlines here and the difference between them is real: a NAME it
cannot resolve is `invalid method reference`, and a name it resolves and cannot
USE is `incompatible types: invalid method reference` — with the specific
reason on the indented line underneath, which is the very message caturra had
been reporting on its own. So
`BiFunction<String,String,Integer> f = String::length` said "method length in
class String cannot be applied to given types", true of a call the program
never wrote, and `Function<Box,Integer> f = Box::twice` said "Box cannot be
converted to int", which is javac's continuation line promoted to a headline.

A synthesized method-reference class holds nothing but the reference, so every
error inside one is a failure of that reference: they all carry the headline
now, and the two-way split follows javac's. Nine of the twelve match; the three
that do not are a library method used in an unbound position (caturra searched
for a static and said "cannot find symbol" about a method that plainly exists —
it cannot enumerate a library class's overloads), `int[]::length`, and an array
constructor reference.

**A redeclared local is still declared.** Reporting the redeclaration and
skipping the binding left the name bound to what it SHADOWS — a parameter,
usually — so every later use was typed against the wrong declaration: an
`ArrayList<Pet> a` beside a `String[] a` parameter went on to complain that a
`String[]` is not a `List<Pet>`, about a line whose types are fine. It binds
now, which also meant making a name lookup take the NEWEST binding within a
scope as well as across them: two bindings of one name in one scope is not
legal Java, so that only ever shows in recovery.

Pinned by `reject_a_static_method_as_an_unbound_reference`,
`reject_a_reference_with_the_wrong_arity`,
`reject_an_instance_method_as_a_supplier`,
`reject_a_reference_to_a_method_that_does_not_exist` and
`a_redeclared_local_is_still_declared`.

### What a library algorithm asks a comparator (2026-08-30)

A user class whose `compareTo`, `equals`, `hashCode` and `toString` all print,
run through eighteen library algorithms that call back into it: the ORDER and
the COUNT of those callbacks, against a real JDK. Thirteen agreed. Two of the
five that did not were wrong in a way that changes ANSWERS, not only traces.

**`Comparator.reversed()` negated where a JDK swaps.** `c.reversed().compare(a,
b)` is `c.compare(b, a)` in a JDK, and was `-c.compare(a, b)` here. The two
agree for a well-behaved comparator and not otherwise: negating
`Integer.MIN_VALUE` gives `Integer.MIN_VALUE` back, so a comparator that ever
returns it sorts the wrong way round — and a comparator with a side effect sees
every pair transposed.

**`Stream.max`/`min` asked in the other order.** A JDK's `max(c)` is
`reduce(BinaryOperator.maxBy(c))`, and `maxBy` is `c.compare(a, b) >= 0 ? a : b`
with the ACCUMULATED value first; caturra compared (next, accumulated). For an
asymmetric comparator — a student's `compare` that returns 1 or 0 and never -1
is the common shape — that is a different answer, not a different trace.
`Collections.max` really is the other order (`next.compareTo(candidate)`, JDK
source), which is why the two now read backwards from each other.

With those two fixed, twelve algorithms ask exactly what a JDK asks:
`Collections.sort`, `list.sort`, `Arrays.sort(T[], c)`, `stream().sorted`,
`stream().max/min`, `Collections.max/min`, `PriorityQueue`, `reversed`,
`thenComparing`, `binarySearch`, and the natural-ordering forms of each.

**Measured, and deliberately not chased:** three algorithms call back the right
number of times in a different ORDER, and one a different number of times.
Inserting into a `TreeSet`/`TreeMap` compares against the middle of a sorted
vector where a JDK descends a red-black tree from its root; `Arrays.sort` of a
reference array is an insertion sort where a JDK runs TimSort (three
comparisons against its four, in a different order); `stream().distinct()`
hashes each element once where a JDK hashes it twice. All four are observable
only through a callback with a side effect, and matching them would mean
reimplementing a JDK's data structures rather than its semantics — the same
call made for `getDeclaredConstructors()` order.

The same question of `equals` and `hashCode`, over twelve more algorithms —
`contains`, `indexOf`, `lastIndexOf`, `remove`, `containsAll`, `List.equals`,
`HashMap.get`/`put`/`containsKey`, `HashSet.contains`, a stream filter — is
exact, callback for callback. The one exception is `Map.merge`, which locates
the bin once in a JDK and does a get and then a put here: three hashes and two
equals where a JDK asks for two and one, with the same answer.

Pinned by `what_a_library_algorithm_asks_a_comparator`,
`what_a_library_algorithm_asks_compare_to`, and
`a_reversed_comparator_swaps_rather_than_negates` — a comparator that answers
`Integer.MIN_VALUE`, which is legal (any negative means "less") and which a
`reversed()` that negates does not reverse.

### What a library algorithm asks a lambda (2026-08-30)

The same probe as the entry above, pointed at the FUNCTIONAL arguments: which
library operations call the lambda they were given, and how many times.
Thirteen of fifteen were exact — `computeIfAbsent` does not ask for a key that
is present, `computeIfPresent` does not ask for one that is absent,
`orElseGet` does not ask when the value is there, `removeIf` and `replaceAll`
ask once per element in order, a short-circuiting `findFirst` stops the
pipeline, `Stream.iterate` asks n-1 times for n elements, and `reduce` folds
left. The two that were not are gaps, not protocols.

**A functional interface the program factored out had no result type.** Pulling
a lambda into a variable, a field, or a method is the first thing anyone does
with one, and `stream.map(f)` then produced a stream of nothing: the answer
rides on a synthesized class that only exists when the lambda is written AT the
call. A value says the same thing in its DECLARED type — a
`Function<String, Integer>` produces an `Integer` — which is now read on both
sides: the lambda pass, so the NEXT operation's lambda has a typed parameter,
and codegen, so the element survives into the collector. It is the same "one
fact, two readers" shape as every other element gap.

**`Objects.requireNonNullElseGet` was missing**, and the message blamed the
LAMBDA — "a lambda or method reference is only allowed where a
functional-interface type is expected" — for a position caturra had not
modelled. Its eager twin `requireNonNullElse` was already there. The supplier
is asked only when the value is null, which is the whole point of the method
and the reason it cannot be the eager one with a call in front of it; and a
supplier that answers `null` is an NPE naming `supplier.get()`, as a JDK's is.

Pinned by `a_stream_maps_through_a_function_value`,
`require_non_null_else_get_asks_only_when_null` and
`what_a_library_algorithm_asks_a_lambda`.

**The fuzzer asks it too, now.** `scripts/fuzz/programs.py` grew a dimension
that hands a PRINTING comparator (and a printing key extractor) to the
operations that differ in which pair they pass, how many times, and whether a
wrapper swaps or negates: `sort`, `sort(reversed)`, `sort(thenComparing)`,
`Collections.sort/max/min`, `stream().max/min/sorted`, and a `PriorityQueue`.
A silent comparator hides all of that, which is why the two bugs above survived
every earlier sweep. 1450 generated programs across three seeds agree with a
real JDK, callback for callback.

A `TreeSet` built with a traced comparator is left out on purpose, and the
generator says why: its insertion order is the measured red-black-tree
difference, so every such program would diverge on the trace while agreeing on
the answer — and a generator whose baseline is not zero hides the next real
find.

### A generic factory takes its target type (2026-08-30)

`<T> List<T> emptyish()` assigned to a `List<String>` is a `List<String>`:
nothing at the call pins `T`, so javac infers it from the TARGET. caturra
erased the variable to `Object` and refused the assignment —
`Collections.emptyList()` written by hand, and the shape every generic helper
class starts from. The same for `<T> Optional<T> none()`,
`<T> Supplier<List<T>> maker()`, `<T> Function<T, T> same()`.

The erasure was right and the comparison was not. A method's own type variable
erases to a type-variable WILDCARD, which already accepts any element as a
PARAMETER — `<T> void dump(List<T>)` takes a `List<String>` — and the same
wildcard as an ARGUMENT was compared as though it were `Object`. It accepts
now, in that direction too. A WRITTEN wildcard deliberately does not: `?
extends Number` is a constraint the program stated, and keeps its own rule.

The element still flows wherever a variable IS pinned, so `String s =
pick(listOfIntegers)` is refused as before — the inference that exists is not
weakened, only the case where there was none to weaken.

**One accepts-invalid follows, and it is the old modelling limit rather than a
new rule:** `Function<String, Integer> bad = same()` is refused by javac,
because `T` cannot be both. caturra models a `Function<A, B>` by its RESULT
alone — a `Function<String,Integer>` and a `Function<Object,Integer>` are one
type here — so it has nothing to contradict. Five valid programs compile for
the one invalid one that does; refusing a generic factory is the worse of the
two for a teaching product.

Pinned by `a_generic_factory_takes_its_target_type` and
`reject_a_pinned_type_variable_used_as_another`.

**A for-each is the one context that is NOT a target.** javac types the source
of a for-each on its own, so `for (String s : empty())` is an error — the
factory infers `List<Object>` with nothing to infer from — and the loop is
written `for (String s : Type.<String>empty())` instead. caturra dropped that
witness and left the element `Object`, so the form javac requires was the one
it refused. The witness is read now, and the form javac refuses is still
refused, which is what keeps the two apart.

`Collections.emptyList()` was a special case of the same thing, one step
worse: it typed as `null` — assignable to any list, as it should be, and
walkable as none, so `for (Object o : Collections.emptyList())` was "for-each
not applicable to expression type" about a list. It is a list whose element is
an erased type variable now, which is both.

### A generic return that names two variables (2026-08-31)

The return-inference plan tracked ONE type variable, so a generic method
returning a container of two — `<K, V> Map<K, V> pair(K key, V value)`, the
shape every "make me a little map" helper has — was not inferred at all:
`pair("k", 3).get("k") + 1` was "bad operand types" about a map whose value
type the call plainly gives. The plan carries a second source list now, and
each argument is pinned on its own.

A FUNCTIONAL return is the same question asked of a different position.
caturra models a `Function<A, B>` by its RESULT alone, so that is the argument
its erased return keeps and the one to pin — `<T, R> Function<T, R>
constant(R value)` answers a function of `Integer` for `constant(7)`, where
before it answered a function of nothing and the stream it mapped had no
element. (The parameter side is not modelled and cannot be, which is the same
limit recorded two entries above.)

Still open, and measured: a stream mapped through a generic function FACTORY
whose result variable nothing pins — `map(same())` for a `<T> Function<T, T>`,
or `map(lift(s -> s.length()))` where the lambda's own body pins it. The
element after such a map wants the receiver's element unified with the
function's parameter, which is the inference direction this engine does not
have.

Pinned by `a_generic_return_that_names_two_variables`.

### Where a diagnostic points (2026-08-31)

Every comparison so far has been of what a diagnostic SAYS. Where it points
went unmeasured — and an editor underlines the place, not the sentence.
`diaglist.py` compares the LINE now, and `differential_wording!` asserts it on
every pin in the suite.

Across the 236-program reject corpus the two engines agree on the line
everywhere but once, which is the answer worth recording: the spans are right.
The exception is a captured local. javac reports it at the REFERENCE — the
line inside the class body that reads the name — and caturra reported it at
the `new` that creates the class, so the underline fell on a line whose only
fault was mentioning it. A lambda already agreed, which is why this went
unseen for so long: a lambda's reference is usually on the same line as its
creation. It is asked statement by statement through the same free-name walk
that decides what is captured, so there is still one answer to the question.

Turning the assertion on found one more: a cycle of constructors was reported
once per constructor in it, where javac reports a cycle ONCE. It is one error
now. Which member javac blames is not a rule anyone states — a two-constructor
cycle is reported at the first, a three-constructor one at the second — so
that case is pinned as a shape rather than a position.

Pinned by `reject_a_captured_local_points_at_its_use`,
`reject_a_captured_local_in_a_local_class`,
`reject_recursive_constructor_invocation` (now a self-cycle, where the
position is not in doubt) and `reject_a_cycle_through_two_constructors`.

### Which token a diagnostic sits under (2026-08-31)

The line agrees; the COLUMN is a different question, and it had never been
asked. javac has a convention per diagnostic for which token its caret sits
under, and `diaglist.py --columns` counts how often caturra matches: 107 of
202 on the reject corpus.

One of those conventions is worth matching and is now matched: **javac blames
the ARGUMENT**. `take(numbers)` where the parameter is a `List<String>` is
reported under `numbers`, and `list.add(new Node<Integer>(6))` under the
`new` — where caturra reported the same message at the start of the
statement, so an editor underlined the whole line. The message and the
argument it blames come out of one walk now, so the two cannot disagree.

The rest are a long tail of javac's own choices — the dot of a missing member,
the `<` of a wrong type-argument list, the second declaration of a duplicate
parameter — and one of them is a trap worth writing down: a first attempt at
the DOT rule read the receiver's end from a field on the emitter, which an
INNER member access in the same expression had already overwritten. It moved
`IntStream.of(1).asDoubleStream().asDoubleStream()` from one wrong column to
another. A position that is confidently wrong is worse than one that is
plainly the statement's, so it was reverted; doing it properly means the error
site holding the receiver, not the emitter holding a field.

The count is in the tool to be moved, not to gate: it is reported by
`--columns` and is not part of the comparison.

**The dot rule, done the way the note above says.** The receiver's end is
recorded beside the receiver's LOCATION, in the one place that already knows
about the clobbering — `instance_call` takes the location, emits the receiver
(which compiles any nested access), and puts its own back. The dot goes with
it, and a bare call clears it so no stale one survives.

Getting there needed the spans to be right first: a dotted call's span stopped
after its NAME, leaving its own arguments outside the expression they belong
to, so every dot after one in a chain read two columns early. It runs through
the closing paren now, as the bare-call form already did — which is what an
editor underlines for a call, and was wrong for every one of them.

Columns agreeing on the reject corpus: 107 → 120 of 202.

Pinned by `reject_an_argument_points_at_the_argument`,
`reject_a_missing_member_points_at_the_dot` and
`reject_a_bare_call_points_at_itself`.

Chasing the last mispointed case found a wording rather than a position: a
void call used as a VALUE. javac has three sentences for it and they depend on
the position — "void cannot be dereferenced" for a receiver, "incompatible
types: void cannot be converted to int" for an assignment, and "'void' type
not allowed here" everywhere else. caturra said one sentence of its own
invention in all three, which was javac's for none of them.

All three are caturra's now, and the position is what tells them apart. The
context the call site could not see is handed to it: a value being ASSIGNED
carries its target, a RECEIVER carries a flag, and both are cleared for an
ARGUMENT — which is a position of its own, and javac words it as one. Each is
taken and restored around the evaluation that owns it, the discipline the dot
above needed for the same reason. `int n = take(go())` is the case that keeps
it honest: the inner call is an argument, though the outer one is assigned.

That path had to be found twice: a call whose argument does not type is
emitted by a different loop, so the first fix reached only half of them.

A staged test had been pinning caturra's invented sentence as though it were
javac's, in a file named for matching javac's wording. It pins the real one.

Pinned by `reject_a_void_call_as_an_argument`,
`reject_a_void_call_as_an_operand`, `reject_a_void_call_assigned`,
`reject_a_void_call_dereferenced` and
`reject_a_void_call_inside_an_assigned_call`.

### What a collection holds after a callback threw (2026-08-31)

Twelve programs that throw from inside a callback — a `compareTo` during a
sort, an `equals` during a `contains`, a `hashCode` during a `put`, a lambda
in `forEach`/`map`/`removeIf` — all propagate exactly as a JDK's do. Asking
the next question found the gap: not whether the exception escapes, but what
the collection LOOKS LIKE afterwards.

`List.replaceAll` writes each element back as it computes it, so an operator
that throws half way leaves the elements before it replaced. caturra collected
the whole list and stored it at the end, so the same program left the list
untouched. A subList VIEW had it twice over: a view delegates by copying its
range, running the operation on the copy, and splicing it back — and it
propagated the exception before splicing, discarding what the operation had
managed to do.

The neighbours say why one rule covers all three: for `sort` and `removeIf`
the scratch copy is UNCHANGED when the callback throws (a JDK sorts into an
array and writes back at the end; `removeIf` collects what to drop before
dropping any of it), so splicing on the error path is right for those too.

And an operator that MUTATES the list it is walking: a JDK's loop is
`for (i = 0; modCount == expected && i < size; i++)`, so it stops as soon as
the list changed — one call, not one per element — and then reports the
modification. caturra ran the operator over its snapshot and added an element
for each.

Pinned by `what_a_list_holds_after_a_callback_threw` and
`a_replace_all_that_adds_stops_at_once`.

**`removeIf` is two methods wearing one name**, and the same probe says which
a collection has. `ArrayList`, `Vector` and — since JDK 11 — `ArrayDeque`
override it with a two-pass scan, so a predicate that throws leaves the
collection untouched. Everything else — `LinkedList`, `HashSet`,
`LinkedHashSet`, `TreeSet` — inherits `Collection.removeIf`, which walks an
iterator and removes each match as it finds it, so the earlier matches are
already gone. caturra had every collection on the two-pass side, which was
right for three of them by accident and wrong for four.

The two now read differently a few hundred lines apart, and each says why.
Pinned by `which_remove_if_a_collection_has`, over six collection kinds.

### The insert, and a comparator with no arrow in it (2026-08-31)

Probing what a half-finished structural operation leaves behind found two
things that never got as far as leaving anything: both were refused outright.

**`list.addAll(index, collection)`** — the INSERT form, which a list has and a
collection does not — reached no arm at all, so an ordinary insert ended the
run with "ClassCastException: not a list". Its range check is `AbstractList`'s,
and that class words its own message ("Index: 5, Size: 1") rather than the
`Objects.checkIndex` one the accessors use.

**A comparator written as a METHOD REFERENCE** was refused for every sorted
collection: `new TreeMap<>(String::compareTo)` was "a lambda or method
reference is only allowed where a functional-interface type is expected". Two
causes, one behind the other. The constructor's comparator argument was
target-typed only when it was written as a lambda, and — once that was fixed —
the interface it desugars to did not exist, because the bundle that provides it
is pulled in by looking for an ARROW in the source. A method reference has no
arrow. It looks for `::` as well now.

The same target-typing looks through a sorted collection's INTERFACE face,
which is how one is usually written: `Map<String, Integer> m = new
TreeMap<>(cmp)` said nothing about the comparator's parameters, so the whole
declaration was refused.

And `List.of()` with no arguments is a generic factory like any other: its
element is an erased type VARIABLE, which the target pins. Answering `Object`
refused `l.addAll(0, List.of())` — an empty list added to a list of strings.

Pinned by `inserting_a_collection_at_an_index` and
`a_comparator_written_as_a_method_reference`.

## How much of a JDK 11 caturra knows, measured

Every claim about coverage in this document is about a behaviour that was
probed. There was no number for the SURFACE: how much of the library a program
can reach at all. `scripts/coverage/measure.py` computes one, from the two
things that can be counted rather than asserted.

**Classes: 286, read from the compiler's own import table.** `imports.rs` is
the single list every import is checked against, so counting it counts what a
program can name — 154 in core `java.*` (48 `java.util`, 43
`java.util.function`, 33 `java.lang`, 8 `java.io`, 6 `java.util.stream`, …),
the rest Swing/AWT and the bundled `org.code` course library.

**Method names: 1030 of 1213 = 84.9%, over the 45 core classes whose receiver
can be written as one expression.** The denominator comes from a real JDK by
reflection (`ApiList.java`), not from a checked-in list, so running the script
on a different JDK moves the number. For each overload it writes a call with
that overload's own arity and `null` for every argument, then reads caturra's
diagnostic: "cannot find symbol" means the name is unknown, and anything else
— including "no suitable method found" — means the name is known and only
these arguments are wrong.

This is NAME-level, deliberately: it says `substring` exists, not that both of
its overloads do. It is the coarsest question with a checkable answer. A probe
that CRASHES the engine is a measurement failure rather than a result — a
panicking compiler prints no diagnostics, which reads exactly like a clean
compile, and every method in that probe would count as known.
Semantics are measured separately and far more strictly — by the 1098
differential pins and the 2698-level corpus sweep, which compare what a
program PRINTS against a real JDK.

Exactly at 100%: `String`, `Math`, `StringBuilder`, `Double`, `ArrayList`,
`List`, `Map`, `HashMap`, `LinkedHashMap`, `TreeMap`, `Set`, `Collection`,
`Queue`, `Deque`, `Iterator`, `Comparator`, `StringJoiner`, `Pattern`,
`Matcher`, `Stream`, `IntStream`, `Iterable`, `Comparable`. The low ones are
low for a reason that is usually — not always — deliberate. `System` 7/25 is a
boundary: JVM plumbing with no analogue (`loadLibrary`, `SecurityManager`,
`inheritedChannel`), the properties table, and stream redirection. `File` was
9/40 when this was written, and only HALF of that was a boundary; the
answerable half is the section below, and it is 23/40 now. `Class` 28/67 is
reflection past what a grading
harness inspects, `Collections` 28/60 and `Stack` 27/46 are the synchronized
and checked wrappers and `Vector`'s inherited legacy half, `Character` 32/52 is
the Unicode code-point surface. Run with `--verbose` for the misses per class.

**Measuring found one defect, in the measurement's own oracle.** A name
`Objects` HAS, called with arguments no overload takes, said "cannot find
symbol" where javac says "no suitable method found for
requireNonNull(<null>,<null>,<null>)" — `Objects` is the one library class
whose statics are matched by shape rather than by a table, so its fall-through
arm could not tell a wrong call from an absent one. Reading the two messages
side by side also showed the null TYPE was named bare: javac writes `<null>`,
in angle brackets, wherever it names it — as an argument type, as the source
of an incompatible assignment, and as a receiver. And a member ON the null
literal is not a missing symbol, since there are no members to miss: javac
blames the receiver, "<null> cannot be dereferenced", for a method and for a
field alike.

Pinned by `reject_a_known_objects_method_with_the_wrong_arguments`,
`reject_assigning_the_null_literal_to_an_int`,
`reject_calling_a_method_on_the_null_literal` and
`reject_reading_a_field_on_the_null_literal`.

## The path and directory half of `java.io.File`

The coverage measurement above scored `java.io.File` at 9 of a JDK's 40 method
names, and the entry filed that under "surface a browser cannot answer". Half
of it was: permission bits, timestamps, device space and `toURI`/`toURL` (which
need `java.net`, a package `imports.rs` refuses by name) have nothing behind
them. The other half was reachable all along — `VirtualFileSystem` has had
`normalize` and `list_dir` since it was written, and `FILE_METHODS` never
asked. It answers 23 of 40 now.

**The paths.** `getAbsolutePath` hangs a relative path off `/`, because that is
where caturra's filesystem is rooted and there is no working directory for it
to be anywhere else — the reading `Path.toAbsolutePath` already took.
`getCanonicalPath` is that with `.` and `..` resolved, and declares
`IOException` like `createNewFile` beside it. `getParent` is `null` both for a
bare name and for the root, which is the pair that stops a walk up the tree.
`getAbsoluteFile`, `getCanonicalFile`, `getParentFile` and `toPath` are the
same answers as objects.

**The abstract pathname.** A `File` keeps what its constructor was given, with
repeated separators collapsed and a trailing one dropped, but `.` segments
left alone: `new File("d//a.txt").getPath()` is "d/a.txt" and
`new File("./a.txt").getPath()` is "./a.txt". `new File(parent, child)` is
`UnixFileSystem.resolve` rather than concatenation — an empty parent means the
root, a null one means the child stands alone, an empty child is the parent
itself, and `new File("d", "/")` really is "d/", trailing separator included.

**The directory.** `list` answers the names, `listFiles` the files, both `null`
when the receiver is not a directory — including when it does not exist, which
a program that skips the check meets as a NullPointerException. `mkdir` makes
ONE directory and answers false when the parent is missing; `mkdirs` makes the
chain. That distinction needed writing down here, because the VFS creates
parents implicitly (what a program writing a file wants) and `mkdir` had been
inheriting it. `renameTo` moves a whole subtree and OVERWRITES an existing
destination file rather than failing, which is the surprising half of the Unix
rule.

**A `File` is `Comparable`, and value-based, and both had to be written
twice.** Its order is its pathname's, magnitude included (`compareTo` is
`String.compareTo`), so a `List<File>` sorts. Its equality is its pathname's —
but a direct `a.equals(b)` reaches the intrinsic dispatch while a COLLECTION
asks the VM's own `native_equals`, which knew only strings: `files.contains(new
File("box/a.txt"))` was reference identity and answered false where a JDK says
true. The same two-places rule applies to the hash that has to agree with it
(`UnixFileSystem` xors the pathname's hash with a constant), and pulling on it
found `Charset` in the same state — value equality with an identity hash, and
`charset.hashCode()` not implemented at all.

**A method reference needs its qualifier to name a class.** `File::getName` —
the ordinary way to turn a listing into names — was "invalid method reference:
cannot find symbol 'File'", because the desugaring's list of library qualifiers
did not have it. `Path` was missing for the same reason.

Two things here are deliberately not a JDK's. A listing's ORDER is sorted where
a JDK's is the filesystem's ("no particular order" — unspecified, like the
reflective listings elsewhere in this document), and `getAbsolutePath` answers
from the root rather than from a working directory. The pins compare what does
not depend on either: sorted contents, and the invariants the absolute and
canonical forms promise wherever they run.

Pinned by `a_files_path_and_directory_methods`,
`a_files_two_argument_constructors`, `a_files_absolute_and_canonical_forms`,
`a_files_in_collections_and_streams`, `a_files_and_charsets_hash_as_their_text`
and `reject_an_uncaught_canonical_path`.

## The collectors that wrap another collector

The coverage measurement scored `java.util.stream.Collectors` at 18 of 29
method names. The eleven missing split cleanly: two are the concurrent forms
(`groupingByConcurrent`, `toConcurrentMap`), which have no meaning where there
is one thread, and nine are the ones a program reaches for AFTER `toList` —
now all present, at 27/29.

**Four of them gather nothing themselves.** `filtering(p, downstream)`,
`flatMapping(f, downstream)`, `mapping(f, downstream)` and
`collectingAndThen(downstream, finisher)` all pass elements to a collector
below. `filtering` is not the same as filtering the STREAM, and the difference
shows exactly where these are used: under a `groupingBy`, a group whose every
member fails the predicate still exists, holding nothing, where a filtered
stream would never have made the key.

**`maxBy`/`minBy` answer an `Optional`**, empty over no elements, comparing
with the accumulated value first (`BinaryOperator.maxBy`'s order, the one
`Stream.max` already uses). **`reducing` has three shapes**, told apart by
their arguments: an operator alone (an `Optional`), an identity and an
operator (a plain value, the identity itself over nothing), and a mapper
between the two.

**`summarizingInt`/`Long`/`Double` needed the two summary classes caturra had
never modelled.** `IntSummaryStatistics` was here; its siblings were not, and
one method table served the type — so a `DoubleStream`'s `summaryStatistics()`
answered an `IntSummaryStatistics` with every number truncated through a
`long`, and `getSum()` was typed `()J` for a value the VM produced as a
double. The three now differ where they really differ: the width of the
numbers they report, the identity values an empty summary keeps
(`Integer.MAX_VALUE`/`MIN_VALUE`, the 64-bit pair, `Infinity`/`-Infinity`), and
the `%f` formatting a double summary gives its sum and bounds. Which one a
`summaryStatistics()` call answers comes from the RETURN DESCRIPTOR, not from
the values — an empty double pipeline has no element to read a kind off, and
its `Infinity` bounds are the whole answer.

**`Optional.ifPresentOrElse` and `or`** finish that class at 16/16. The first
is the only place a `Runnable` is a functional PARAMETER here (its empty arm
takes nothing and answers nothing); the second's supplier answers another
`Optional` rather than an element, which is what tells it from `orElseGet`.

Three defects turned up on the way, each the same shape — one fact written in
two places, and only one of them updated:

* A summary inside a map printed as `object@2a`. Its `toString` lived in the
  intrinsic dispatch, and a collection renders its elements through a
  different function. Both call one writer now.
* `Collectors.mapping(f, downstream)` was typed as producing a `List` of the
  mapped element, which is right for the usual `mapping(f, toList())` and
  wrong for every other downstream. It asks the downstream now.
* `boxed_name` boxed a primitive written as `TypeRef::Named("int")` but not
  one written as `TypeRef::Int`, so a mapped element could reach a type
  argument as a bare `int` — reported as "required: reference, found: int"
  about a program that says neither.

And two spellings that were not recognised as naming a class:
`Collections::unmodifiableSet` (the bundle is pulled in by a text sniff for
`Collections.`, which a method reference does not write) and
`Optional.<String>empty()` as a receiver (a witness says the element where
there is no argument to read it from).

Pinned by `the_collectors_that_wrap_another_collector`,
`the_wrapping_collectors_under_a_grouping`,
`each_primitive_stream_has_its_own_summary` and
`an_optionals_two_armed_forms`.

## The methods that were missing only on some receivers

Reading the coverage misses down the collection classes showed one shape
repeated: a method that every `Collection` declares, present on the list and
set tables and on no other. `deque.removeAll(...)` was "cannot find symbol"
while the identical call on an `ArrayList` worked — and the VM had been
answering all three bulk operations from the shared element vector the whole
time, so nothing but the table was missing. `containsAll`/`removeAll`/
`retainAll` are on every collection now; `sort` and `replaceAll` are on
`LinkedList` (it is a `List`); `comparator()` is on `PriorityQueue` (the
question a sorted set has always answered, about the same kind of object); and
`IntStream` has the `iterator`/`forEachOrdered`/`flatMap` its object twin
declared. A primitive stream's `iterator()` is a `PrimitiveIterator.OfInt`,
which IS an `Iterator<Integer>` — its `next()` hands back a reference, so the
element boxes.

**`parallelStream()` runs now, rather than being refused.** It was in the
honest-refusal table with the reason "caturra runs on one thread, so a parallel
stream would only be a sequential one under another name". That reason is
sound and the conclusion was wrong: a JDK's own contract for
`Collection.parallelStream()` says "it is allowable for this method to return a
sequential stream", so a sequential answer is not an approximation of the
method — it IS one of its permitted answers, the same reading `synchronized`
already gets here. `parallel()`, `sequential()`, `unordered()` and
`isParallel()` come with it. The pipeline really is sequential; the only thing
a program can see is `isParallel()`, which answers what a JDK answers because
the flag is tracked beside the stream and travels its ops.

**`collect(supplier, accumulator, combiner)`** — the form that gathers with no
`Collector` at all — is on both streams. Its combiner is evaluated and never
called, since one thread never splits the work. Typing it needed the CONTAINER,
which a constructor reference states outright: `ArrayList::add` is an unbound
reference on that very type, so an `Object` container makes the reference
impossible rather than merely imprecise.

**A null callback throws where it is written.** Every stream op matched
`Ref(Some(f))` and a null simply missed the pattern, so the VM reported "not
yet implemented" about a method it implements. A JDK guards each one with
`Objects.requireNonNull` before it builds anything — including on an EMPTY
`Optional`, which throws for a null mapper rather than answering empty.
`sort(null)` is the exception that proves it: that one means the natural
ordering.

**And the compiler PANICKED on `Stream.generate(null)`.** `build_erased_lambda`
ends in `unreachable!("guarded by caller")`, and the `generate` arm did not
guard. The panic is the worst part of this entry: it took every other
diagnostic in the file with it, so a program with that call and two ordinary
mistakes reported NOTHING — not a wrong message, no message. An argument that
is not a lambda is already a value (a variable, a field, `null`); there is
nothing to erase, only to walk, so that is what the fall-through does now.

**The measurement was reading its own crash as a pass.** A panicking probe
prints no diagnostics, which is exactly what a clean compile prints, so every
method in it counted as known: `Stream` scored 44/44 while the same calls one
at a time were "cannot find symbol". `scripts/coverage/measure.py` treats a
non-zero exit as a measurement failure now, and the honest figure — which is
lower than what was written here before — is **1030 of 1213 = 84.9%**.

Pinned by `the_bulk_operations_every_collection_declares`,
`a_parallel_stream_is_a_stream_here`, `the_stream_ops_only_one_stream_had` and
`a_null_callback_throws_where_it_is_written`.

## The engine must not crash

A panic is the worst answer a compiler can give, and the reason is not that it
looks bad: it prints NO diagnostics, which reads exactly like a clean compile.
The `Stream.generate(null)` panic above took every other error in its file with
it, and it made the coverage measurement score a whole probe as supported.
Nothing in the sweep or fuzz tooling was watching for one — they all compare
what caturra says with what a JDK says, and a crash says nothing.

`scripts/fuzz/panics.py` asks only that the engine exit cleanly. Over the API
surface the coverage measurement walks, it calls every modelled method with
each of ten argument SHAPES — a null, a one-parameter lambda, a two-parameter
one, a supplier, a method reference, a constructor reference, a number, a
string, an object, an array — and bisects a failing batch to the single call
that did it. Whether the call is legal is beside the point: an illegal one must
be REFUSED, not crash.

**Its first run found 15 crashes at one site.** A lambda with FEWER parameters
than its interface takes — `stream.max(x -> x)`, where a comparator takes two —
was reported correctly ("incompatible types: incompatible parameter types in
lambda expression") and then the desugaring carried on and indexed past the end
of the lambda's own parameter list. Binding the parameters that exist builds a
class nothing will run (the program is already refused) and reports one mistake
instead of dying. 450 probes over 45 classes, 0 crashes.

**And an unimported class hid every other mistake in the file.** The snapshot
that decides "did this parse?" — the one that keeps a file with a syntax error
from being reported with the nonsense a recovered tree makes of the rest — was
taken AFTER the import check had pushed its errors. So `Pattern p =
Pattern.compile("a")` with no import made the whole file read as unparsed, and
everything attribution found was truncated away: javac reported four errors for
such a program and caturra reported the import's two. The snapshot is taken
before the import check now, which is attribution too.

With the truncation gone, the ORDER showed: caturra runs several passes over a
whole file, so the import check's error sorted ahead of an attribution error on
an earlier line, where javac attributes in source order and reports that way.
The diagnostics are sorted by position at the end, stably, so two complaints
about one place keep the order the phases found them in.

Pinned by `reject_a_lambda_with_too_few_parameters`,
`reject_a_comparator_that_takes_nothing`,
`an_import_error_does_not_hide_the_others` and
`the_first_error_is_the_first_mistake`.

**The hunt grew two dimensions and found one more.** A crash can hide in what
the compiler does with a call's RESULT rather than in the call, so each probe
is now written in one of five POSITIONS — a bare statement, an argument, an
`Object` local, a `var` local, a receiver for `.toString()` — and the
CONSTRUCTORS are probed too, since `new X(...)` reaches a different emitter for
every modelled type. 680 probes, and the new positions found
`Collectors.toList().toString()`: a `Collector` has no method table of its own,
and the lookup asserted that every library receiver does. `Object`'s methods
are the fallback now — which is what a JDK gives it, including the class it
really builds (`java.util.stream.Collectors$CollectorImpl`). It needed a table
of its OWN rather than a `toString` added to the shared `Object` one: that
table is mixed into every other receiver's, so adding a method there overrode
the honest refusals that name it, and a `Scanner`'s text — which the
compatibility page documents as not modelled — started compiling and failing at
run time instead.

**And every tool must notice a crash, not just this one.**
`scripts/fuzz/diaglist.py` compares diagnostic LISTS, so a panic on a program
javac ACCEPTS read as perfect agreement — no errors on either side. It fails on
a non-zero exit now. (`run.py` already caught one, as "caturra printed no
JSON".)

Running the expression-position mirror through that stricter tool then found a
real refusal: `Object o = pick ? Collections.emptyList() : null`. Both branches
adopt their context — the factory answers whatever is wanted, like a diamond
`new`, and so does the null literal — so the conditional joins at the null type,
and the branch that really makes a list was asked to convert INTO it. A null
join is "adopts the context", not a type to convert to.

Pinned by `a_collector_is_an_ordinary_object` and
`a_conditional_that_joins_at_null`.

## Where the new surface joins the old

Every feature added this session was probed on its own. Programs that COMBINE
them found three more defects, all the same shape: the element type stopped
travelling at a join that had never been crossed before.

**`dir.listFiles()` is an array a program streams over**, like `split` or
`toCharArray` — and the pass that reads an array's element from a library call
did not know it, so `Arrays.stream(dir.listFiles()).filter(File::isFile)` had
no element and the method reference was refused for having no
functional-interface position.

**A comparator inside a collector is usually a FACTORY CALL, not a bare
lambda.** `Collectors.maxBy(Comparator.comparingInt(f -> f.getName().length()))`
— the comparator chain reads its element from its TARGET type, and the
collector desugaring handed it `None`, so the inner lambda's parameter had no
type and `f.getName()` was "cannot find symbol". The bare-lambda form of the
same collector worked, which is the tell.

**The `Collections` wrappers pass their ARGUMENT's type through**, which a
table keyed by (class, method) cannot say — it never sees the argument.
`collectingAndThen(toList(), Collections::unmodifiableList)` is the ordinary
way to freeze a gathered list, and with no type for the finisher's body the
whole `collect` typed as nothing: `.size()` on it was "<null> cannot be
dereferenced", while the same collector assigned to a variable worked.

Also: a stream a `Supplier` answers (`Supplier<Stream<Pet>> s = pets::stream;
s.get()...`) now carries its element, read from the declared type.

Pinned by `a_directory_listing_through_the_collectors` and
`a_collector_whose_finisher_freezes_the_result`.

## The negative direction, over the new surface

Widening what compiles is half a change; the other half is what must still be
REFUSED, and with which words. Twenty-two programs that misuse this session's
additions — a `File` method given the wrong type, a collector whose lambda
answers the wrong thing, a stream op with too few arguments — were compared
with javac's whole diagnostic list. **Every one is refused by both engines**;
no accepts-invalid. Most of the wording differences are the standing inference
family (javac names an inference variable, caturra names the concrete
mismatch). Three were real defects.

**`Optional.or(supplier)` takes a supplier of another OPTIONAL** — that is the
whole difference from `orElseGet`, which takes a supplier of the element.
Erased with no result type the check was gone, so `o.or(() -> "x")` compiled.
It was hidden behind a second bug: the function bundle is pulled in by a text
sniff, and neither `.or(` nor `.ifPresentOrElse(` was in it, so the program was
refused for "cannot find symbol: class __Supplier" — a name it never wrote —
which looked like the refusal it deserved.

**javac's headline for a lambda whose BODY answers the wrong type names the
lambda**: "bad return type in lambda expression", with the mismatch on the line
underneath. caturra reported only that continuation line, which on its own
reads as a mistake somewhere in code the program never wrote. The synthesized
result local is where the check happens, and an error there is worded as
javac's now.

**An explicit WITNESS is the only thing that can say what an empty Optional
holds** — there is no argument to read it from and no assignment context.
`Optional.<String>empty().orElse("x").length()` was "cannot find symbol",
because the empty Optional typed as a bare null and `orElse` answered `Object`.
Both paths read the witness now, and both read it as a TYPE ARGUMENT rather
than as a type: `Integer` there is the wrapper element an Optional holds, where
resolving it as a type and asking for its element gives an interned `Object` —
so `Optional.<Integer>empty().orElse(7) + 1` was "bad operand types" while the
same expression assigned to an `int` compiled.

Pinned by `reject_an_or_supplier_that_answers_an_element`,
`reject_a_lambda_whose_body_answers_the_wrong_type` and
`a_witness_says_what_an_empty_optional_holds`.

## Publishing what this all added

The compatibility page is where a student finds out what works, and every claim
on it is a runnable program: `scripts/compat/record.py` asks a real JDK 11 and
caturra what each one actually does, `tests/compat_manifest.rs` re-asks on every
CI run, and `e2e/compat.spec.ts` makes the page prove all of them in a browser.
Everything this session added was reachable and pinned, and none of it was on
the page — so a visitor had no way to know it was there.

Seven programs now cover it: **a file's path and a directory's contents**
(`getParent`, the absolute and canonical forms, `mkdirs` beside `mkdir`, `list`
/`listFiles`, `renameTo`), **the collectors that wrap another collector**
(`filtering`/`flatMapping`/`collectingAndThen`, `maxBy`, `reducing`, and the
three-argument `collect`), **the summary statistics** in all three widths with
their empty-summary identity values, **`parallelStream` on one thread**,
**`Optional`'s two-armed forms**, and **the bulk operations every collection
has**. 101 supported features, 5 documented gaps, 3 beyond Java 11.

One of them had to be written to run TWICE. The page keeps one filesystem for
a whole visit, so a program that makes a directory and leaves it there answers
`false` the second time it is asked to make it — a legitimate program the page
would then report as failing. The file program tidies up after itself, and says
so in its last line.

## How much of the COURSE library can a program call?

The JDK-11 measurement asks how much of `java.*` is reachable. The library that
matters more to a student here is `org.code` — the one the Code.org curriculum
is written against — and nothing measured it. `scripts/coverage/course.py` does,
the same way: the denominator is Code.org's OWN source, by reflection over the
classes `build-reference.py` compiles from the vendored `javabuilder/` checkout;
the numerator is what caturra answers for a call to each name.

**118 of 118 method names, over 15 student-facing classes** — `Painter`,
`Image`, `Color`, `Pixel`, `Font`, `FontStyle`, `Theater`, `Scene`,
`Instrument`, and the six `org.code.validation` types a grading harness reads.
Three names are excluded and named rather than counted: `Image
.getBufferedImage`, `Image.getImageAssetFromFile` and `Color.convertToAWTColor`
hand back `java.awt` types, which have no meaning in a browser.

The measurement found one real gap: **`Pixel.getSourceImage()`**, the image a
pixel belongs to — what a program reaches for to ask the source its size while
walking it. It is the only student-facing name of the course library that was
missing, and it is pinned against the REAL library
(`media_a_pixel_knows_its_image`), not just against the model.

The number is checkable rather than asserted, but it is not CI-able: the
vendored checkout is gitignored — it is not ours to vendor — so this runs where
the media differential tests do, and says so when the checkout is absent.

## A stream is a resource

`try (Stream<String> lines = Files.lines(path))` is the documented way to read
a file with a stream, and it did not compile: a stream had no `close()`, so the
try-with-resources desugaring had no method to call and the RESOURCE was
"cannot find symbol" — about a declaration the program had written correctly.
The resource check itself was never the problem; it judges class types and
leaves a builtin alone.

`close()` and `onClose(Runnable)` are modelled now. The handlers travel the
pipeline, as a JDK's do — one registered before a `map` still runs when the
mapped stream is closed — and they run in registration order. **Closing is not
an OPERATION on the pipeline**: a JDK closes a stream the body already consumed
without complaint, which is exactly what try-with-resources does to every one
of them, so the close is answered ahead of the single-use check rather than
tripping it.

`onClose` is also the one stream callback that is not a function of the
element — a `Runnable` takes nothing and answers nothing — so it cannot ride
the table that types the rest, and the ops that change nothing about the
elements (the parallel toggles, and this) had to be added to the list a chain
walks to find its element: `onClose(…).onClose(…)` left the second lambda with
no target.

Pinned by `a_stream_closes_like_a_resource` and
`a_streams_close_handlers_run_in_order`. `Stream` is 42 of 44 method names now,
`IntStream` 46 of 48; what is left on both is `builder()` (a nested type
nothing else needs) and `spliterator()` (a parallel-decomposition handle, which
is refused with a reason for every collection already).

## Where a parse error points

Eighteen programs, each with one ordinary SYNTAX mistake — a missing
semicolon, an unbalanced brace, a hole in an array initializer, a `case`
without its colon — compared with javac's whole diagnostic list. The WORDING
was never in question: the parser's is deliberately more explicit than javac's
("expected ';' to end the declaration" where javac says "';' expected"), and
that is written down above. What was in question is where it POINTS, and how
much it says.

**javac's caret for a MISSING token sits at the end of the token before the
gap**, not on the token that surprised the parser — and the two are often on
different LINES. A statement missing its semicolon is a mistake on the line the
statement is on, and caturra pointed at the line after it, which is the one
thing an editor's underline gets wrong in a way a student cannot reason about.
`expect_symbol` and `expect_ident` report at the end of the previous token now.

**A statement that aborts part way may have opened braces it never closed.**
Recovering from a depth of zero stopped at the first `}` — which closes the
CONSTRUCT, not the block — so the block ended early and every line after it
read as a class member: `int[] a = {1, 2, 3,,};` reported three errors where
javac reports one, two of them about a class body the program does not have.
The recovery counts the braces the statement itself opened; an array
initializer whose element does not parse also closes itself rather than letting
the abort escape.

Fifteen of the eighteen now agree with javac on every error position, up from
two. Of the three that do not: an unclosed class blames the class ("class 'S'
is missing its closing '}'", at the declaration) where javac says "reached end
of file while parsing" at the last line — more useful, and one error rather
than one; and two report a single error where javac reports a second after it.
Fewer is the safe direction.

Pinned by `reject_a_statement_missing_its_semicolon` (the LINE, not the
wording), `an_unfinished_initializer_reports_one_mistake` and
`an_unfinished_case_label_reports_one_mistake`.

### Breaking a working program in one place

`scripts/fuzz/syntax.py` is the hand-written sweep above, asked of hundreds:
mutate ONE token of a program that compiles — delete it, type it twice,
transpose it with its neighbour, replace it with a symbol next to it in the
grammar — and compare the POSITION of the first error and the NUMBER of errors.
At 200 mutations, 172 of the 193 that broke something put the first error in
the same place as javac, and 58 of those say less after it (fewer is the safe
direction). Nothing was refused that javac accepts.

It found one **accepts-invalid** on its first run, and it is a nice one: `return
"<" + s + ">"; ;` — an empty statement IS a statement for reachability (JLS
§14.21), so the `;` after a return is unreachable and javac says so. caturra
dropped empty statements at parse time, because they do nothing at run time,
and compiled a program a JDK refuses. They are kept in the tree now
(`Stmt::Empty`), which costs one arm in each pass that walks statements and
buys the reachability rule.

It also found the ORDER problem the sort exposed: an unclosed class reported
"class 'X' is missing its closing '}'" at the class's NAME, which sorts ahead
of every real mistake in the file — a program with a stray `{` on line 19
reported the unclosed class first, about line 4, where javac's first error is
the stray brace. The message still names the class; it points at the end of the
file now, which is where the brace should have been and where javac's own
"reached end of file while parsing" points.

Pinned by `reject_an_unreachable_empty_statement` and
`a_stray_semicolon_is_an_empty_statement`.

### What the mutations found at a second seed

Running `syntax.py` on a different seed, and reading its counterexamples rather
than its summary, turned up three programs a JDK refuses and caturra compiled —
each one token away from a working one, and each the dangerous direction.

**`class Animal implements Comparable<>`** — an empty type ARGUMENT list. The
diamond is an argument list at a `new`, and the `new` path parses its own; every
position that reaches the shared parser (a declared type, an `implements`
clause, an explicit witness) needs a real argument.

**`(x, Integer y) -> x + y`** — a lambda's parameters are ALL inferred or ALL
declared (JLS §15.27.1). Mixing them compiled, and the declared one silently
took its type from a name the program meant as a parameter.

**`javautil.function.Function<String, Integer>`** — one deleted dot. The
functional interfaces are aliased to their bundled erased forms by the LAST
segment of the name, so any qualifier at all resolved: a typo'd package
compiled and would fail on a JDK. A qualified name has to name a real package's
class now; the unqualified spelling is judged by the name alone, as before.

The measurement also stopped counting a column as a wrong ANSWER. javac has a
convention per diagnostic for which token its caret sits under — the operator
of a binary expression, the name of a clashing method, the token after a gap —
and caturra matches many but not all; a column difference on the right line is
a much smaller thing than blaming the wrong line. The headline is the LINE: at
200 mutations, 192 of 193 agree, and the one that does not is javac reporting
two errors in an order neither engine's phases can be talked into matching.

Pinned by `reject_an_empty_type_argument_list`,
`reject_a_lambda_that_declares_only_some_parameters`,
`reject_a_qualified_name_whose_package_is_a_typo` and
`reject_an_empty_type_parameter_list`.

### Three more seeds

Running the mutator on three fresh seeds and reading its counterexamples found
two more divergences, one in each direction.

**A leading zero is only OCTAL for an INTEGER.** `0574.` and `08.5` are decimal
floating-point literals — the `.` decides it, and the digits are then read in
base ten, so `08.5` is legal Java where `08` is not. The lexer committed to
octal on seeing `0` followed by a digit, read `0574`, and left the `.` for the
parser to meet as a field access: a literal a JDK compiles was "expected a name
after '.'". One lookahead past the digit run fixes it, and `017`, `0777L`,
`0x1F` and `09` all still say what they said.

**An interface member is implicitly public**, so an implementation cannot be
package-private. The check existed — and looked the access up by the SOURCE
name while the table is keyed by the BINARY one, so it found nothing for every
nested class, and nothing defaults to public. A `compareTo` written without
`public` in a class implementing `Comparable` compiled here and is an error on
a JDK; that is the shape half the corpus's Comparable lessons have, one
`public` away.

The message names the PARAMETERS now, as javac's does: `compareTo(B) in B
cannot implement compareTo(T) in Comparable`. The implementation's are read
from the class's own declaration — `implementation_of` matches against the
interface's erased signature, so its parameters come back erased — and the
interface's are whatever the table holds, which for a synthesized library
interface is the erasure rather than the type variable javac prints.

Pinned by `a_leading_zero_before_a_decimal_point`,
`reject_an_implementation_that_weakens_access` and
`reject_a_package_private_compare_to`.

### Mutating what a program MEANS

The mutator only ever broke the grammar: delete a token, double it, transpose
two, swap one for a neighbour in the punctuation. Two more kinds of edit reach
past the parser into attribution, where most of a student's errors actually
live — rename an identifier to one the program never declares, and retype a
declaration (`int` for `boolean`, `String` for `int`, drop a `static`, drop a
`return`). The oracle is unchanged: javac decides, and the comparison is the
first error's line and the number of errors.

At 250 mutations on two seeds, 466 of the 474 that broke something put the
first error on the same line, and nothing is refused that a JDK accepts. Two
programs compiled that a JDK refuses, and neither was about meaning at all —
both were the grammar, from a corner the earlier seeds had not reached.

**`break; ;` in a switch arm.** An empty statement is a statement for
reachability, which is why they are kept in the tree — and the switch-arm loop
dropped them anyway. Two parsers read a block: `block_body` kept the `;` and
the arm loop threw it away, so the one place the rule could still be broken was
the one place a `break` is ordinary. The arm keeps it now, and all three of
`break; ;`, `continue; ;` and `return x; ;` report on the same lines javac
does.

**`@ @Override`.** An `@` must be followed by the annotation's name — or by
`interface`, which begins an annotation DECLARATION rather than a use. The
annotation skipper read the name as "identifiers while there are identifiers",
which is satisfied by none of them, so a stray `@` was skipped in silence.
javac says `<identifier> expected` just past the `@`, and so does this.

### An anonymous class implementing a library interface

Pulling on the second of those found a much bigger hole than the `@`. The
access rule from the seed before — an interface member is implicitly public, so
an implementation cannot be package-private — was reaching a NAMED class only
by luck. The check asks `implementation_of` for the method matching the
interface's signature, and a synthesized library interface is written ERASED
(`__Comparator.compare(Object, Object)`) while the class writes the real types
(`compare(String, String)`). Those match only through the erasure BRIDGE
synthesized beside them — and an anonymous class has no bridge. So

```java
Comparator<String> byLength = new Comparator<String>() {
    int compare(String left, String right) { ... }   // javac: cannot implement
};
```

compiled here, which is the shape every sorting lesson writes, one `public`
away from correct. The implementation is matched by name and ARITY when the
interface is one of the erased ones, which is how the bridge dispatches it
anyway. `approximate` — the flag that keeps the return-type and `throws` rules
off a signature caturra only approximates — never covered the `__`-prefixed
interfaces at all: they come from caturra's own Java source rather than from
the table of synthesized names, so `Comparator`, `Predicate`, `Function` and
the rest were being checked as though their erased signatures were the JDK's.

Two things the message says now. javac names an anonymous class
`<anonymous Outer$1>`, its binary name whole; `source_type_name` reads the tail
after a `$` as the simple name, so the message said "`1` cannot implement".
And the interface's parameters are printed as the JDK DECLARES them —
`compare(T,T)`, not `compare(Object,Object)` — for the handful of interfaces
caturra writes erased rather than with a type variable.

Pinned by `reject_an_empty_statement_after_a_break`, `reject_a_stray_at_sign`,
`the_annotations_a_program_writes`,
`reject_a_package_private_compare_in_an_anonymous_class`,
`an_anonymous_class_is_named_as_javac_names_it` and
`the_anonymous_implementations_that_are_legal`.

### Twenty ways a class can fail to implement its interface

The anonymous-class hole was found by a mutator, one token at a time. Asking
the question directly — twenty programs that each satisfy or fail an interface
in one specific way, half of them anonymous — found five more, four of them in
the dangerous direction.

**A broader `throws` was never checked through an interface.** The rule (JLS
§8.4.8.3) was written and then asked of the `throws` table by the SOURCE name
while the table is keyed by the BINARY one — the same mistake the access
lookup two lines above it made, found the same week. An empty clause reads as
"declares nothing", so an implementation could broaden freely: `public String
name() throws IOException` against an interface that throws nothing compiled.

**A `static` method could implement an abstract one.** The check existed and
read only the interface's DEFAULT methods, because a static hiding a default is
where the rule is usually met. JLS §8.4.8.1 is about an INSTANCE method,
abstract or not: `public static String name()` in a class implementing `Named`
compiled here, and then had no instance method to dispatch.

**The type argument was a comment on the message, not a rule.** Matching ran on
the erasure — `__Comparator.compare(Object, Object)` — while only the
diagnostic substituted `Comparator<String>`'s argument back in, a split the
code said out loud ("This shapes the MESSAGE only"). So `compare(Integer,
Integer)` implemented a `Comparator<String>`. The substitution is what
`missing_abstract_method` matches on now: where a class writes exactly one type
argument on one of the erased interfaces, that argument is what each erased
`Object` parameter stands for. A raw `implements Comparator` still matches by
erasure, which is what a raw type means.

That one needed a second fix underneath it: the arguments were being dropped
twice over. An anonymous class recorded no `supertype_args` at all — `new
Comparator<String>() { ... }` fixes the argument exactly as an `implements`
clause does — and `written_supertype_args` looked the parent up by
`class_id("Comparator")`, which does not reach the `__Comparator` it aliases.

**A PRIMITIVE return is exact.** `approximate` — the flag that keeps the return
and `throws` rules off a signature caturra only models — was waving through
`public String compare(String, String)`. It is right about a reference return
(`Supplier.get()` really does answer `Object`, and `Iterable` a raw `Iterator`)
and wrong about a primitive one: `compare` returns `int`, there is no
covariance to allow, and a JDK reports it.

**And javac's names, in the other message too.** `<anonymous Outer$1>` and the
substituted parameters now appear in "is not abstract and does not override
abstract method" as well as in "cannot implement" — it said `1 is not abstract
... compare(Object,Object)` where javac says `<anonymous MissingSamAnon$1> ...
compare(String,String)`. A synthesized lambda class ends in a counter too and
is deliberately not one of these: it stays "lambda expression".

Of the twenty, the two that still differ are both refused by both engines: a
wrong RETURN type makes javac report the missing override first and the clash
second, where caturra reports only the clash — and for a named class, the body's
own `incompatible types` gets there first. Fewer errors, in the safe direction.

Pinned by `reject_a_broader_throws_on_an_interface_method`,
`reject_a_broader_throws_in_an_anonymous_class`,
`reject_a_static_method_implementing_an_interface`,
`an_anonymous_class_that_implements_nothing`,
`reject_a_signature_that_ignores_the_type_argument`,
`reject_a_wrong_return_through_an_erased_interface` and
`the_interface_implementations_that_are_legal`.

### The class a message names

The same twenty-question shape, asked of CLASS inheritance — a `final` method
overridden, a `final` class extended, a private method "widened", a static over
an instance and an instance over a static, an incompatible and a covariant
return, an abstract class instantiated and one left unimplemented, a missing
`super(...)`, a `super` call that is not first, a broader and a narrower
`throws`, a hidden field, a hidden static, `@Override` on nothing, an
assignment to an inherited `final`. Twenty-five programs; twenty-one already
agreed with javac to the word, and nothing was accepted that a JDK refuses.

What the four disagreements had in common was the NAME in the message.

**A nested class was named by its BINARY name.** `class Thing extends Named`
said "cannot extend interface `N02$Named`", the ambiguous-default message said
"types `N19$A` and `N19$B` are incompatible", and a cycle said "cyclic
inheritance involving `Cyc3$B`" — three types no program ever wrote. The rule
is old and written down (a message showing a binary name reads as caturra's bug
rather than the program's); these three sites were simply missed, which is what
a probe set is for.

**A cycle said one thing per CLASS, in a HashMap's order, at line 0.** javac
says one thing per CYCLE, about the class that declares it, on that class's
line: `A extends B`, `B extends C`, `C extends A` is one error about `A`. The
walk now runs in SOURCE order and reports only where it returns to the class it
started from — so a class that merely REACHES a cycle above it is not blamed
for it, and every class on the cycle after the first is silent.

**`class Thing extends SomeInterface` said two things.** The bogus supertype
stayed in place, so the implicit `super()` then looked for a constructor on an
interface — the first mistake's consequence reported as a second mistake.
javac's own wording for this one is "no interface expected here", the
counterpart of the "interface expected here" caturra already had for a class in
an `implements` clause, and it replaces a friendlier sentence that named the
interface by a name the program never wrote.

**An inherited method was blamed on the wrong class.** `new Sub().go(1)` where
`go()` is declared in `Base` said "method go in class Sub cannot be applied" —
naming a class that does not declare the method the message is about. javac
names the DECLARING class, which is what a reader needs to go and look at.

Pinned by `reject_a_cycle_in_the_class_hierarchy`, `two_cycles_are_two_errors`,
`reject_a_class_that_extends_an_interface`,
`a_class_extending_an_interface_says_one_thing`,
`an_inherited_method_names_the_class_that_declares_it`,
`ambiguous_defaults_name_the_interfaces_as_written` and
`the_inheritance_chain_a_lesson_writes`.

### Access, and what a `switch` may switch on

Two more probe sets in the same shape, and both dimensions came back nearly
clean — which is worth writing down as plainly as a haul is.

**Twenty-five access and static-context programs**: a private field read from
`main`, an instance method called without a receiver, `this` in a static
method, a private member of a nested class read from the enclosing one (legal
in Java, and legal here), a private member read from a SUBCLASS (not legal), a
private static shared with a nested class, `protected` through inheritance, a
private field of ANOTHER instance of the same class, a captured local, a
non-effectively-final capture, a private constructor, a static method through
an instance, an instance field in a static initializer. **Twenty-four of the
twenty-five already said exactly what javac says.**

The one was a message that had never distinguished two different mistakes.
Inside the enclosing class, an inner class is IN SCOPE and what is missing is
the `this` a static context does not have — javac: "non-static variable this
cannot be referenced from a static context". Written from another class
entirely, the type is fine and what is missing is the instance to qualify it
with — javac: "an enclosing instance that contains Holder.Inner is required".
caturra said the second for both, so the commonest form of it (`new Inner()`
in `main`) got the rarer message. The two are told apart by whether the class
doing the writing is the enclosing class, is nested inside it, or extends it.

**Twenty-five `switch`, `enum` and label programs**: a duplicate case label, two
`default`s, a non-constant label, a selector of every type Java 11 does and does
not allow, a `case` whose type does not match the selector, an enum constant
written qualified (`case Kind.A:`, which Java forbids), a constant that is not
one, a label on a `String` that is `null` at run time, fall-through, an empty
switch body, a `switch` that is the whole body of a method returning a value,
`break`/`continue` with and without a label, a `break` outside any loop.
**Twenty-three agreed to the word**, and nothing was accepted that javac
refuses.

The two that did not were both wording caturra had written for itself:
`'break' can only be used inside a loop or switch` (javac: "break outside
switch or loop"), `'continue' can only be used inside a loop` (javac:
"continue outside of loop"), and a selector message that explained itself —
`double cannot be converted to int (or String) for switch`. javac says nothing
about switch there: a selector is CONVERTED to `int`, so the message is the
ordinary assignment one, lossy where the type is numeric and plain otherwise.
`long` had been special-cased into javac's exact words, which is what made the
general case look right.

Pinned by `an_inner_class_in_a_static_context`,
`an_inner_class_from_another_class`, `reject_a_double_switch_selector`,
`reject_an_object_switch_selector`, `reject_a_break_outside_a_loop`,
`reject_a_continue_outside_a_loop` and `the_switches_a_lesson_writes`.

### Capturing what a program prints

Two of Code.org's own validators are written the way every JUnit tutorial
writes an output test:

```java
private final PrintStream standardOut = System.out;
private final ByteArrayOutputStream outputStreamCaptor = new ByteArrayOutputStream();

@BeforeEach public void setUp()    { System.setOut(new PrintStream(outputStreamCaptor)); }
@AfterEach  public void tearDown() { System.setOut(standardOut); }
```

Not one line of that ran here. `PrintStream` and `OutputStream` were on the
honest-refusal list, `System.setOut` did not exist, and `System.out` was not a
VALUE at all — the compiler routed `System.out.println(...)` straight to the
stream, so the field itself never needed a type. (The two levels are not in the
grading sweep, because neither ships a `solution` for it to grade. They are
still levels a student is graded on.)

**`System.out` is a `PrintStream` value now**, `System.setOut` REPLACES the
singleton behind it — which is why every `System.out.println` after it follows,
routed or not — and a `PrintStream` carries a SINK: one of the two standard
streams, or a `ByteArrayOutputStream` the program owns. The buffer is reachable
through the stream, so a collection cycle in between does not take it away.

**And `@AfterEach` ran nowhere at all.** The generated runner did `@BeforeAll`,
`@BeforeEach`, the test — and stopped. JUnit runs the teardown whether the test
passed or threw, which is why it goes in both arms of the runner's `try` rather
than a `finally`: a `finally` runs after the verdict is printed, and the
teardown is what puts `System.out` back. `@AfterAll` runs at the end of the
class.

**The runner reports out of band.** Its `__VPLAN`/`__VTEST` lines went through
`System.out` — the same channel the program under test can redirect. A
validator that captures output would have collected the runner's own reporting
into the student's buffer, and every verdict after the first would have
vanished. The runner holds the stream `System.out` named at startup and prints
through that, which is what a real harness does for the same reason.

Two smaller things fell out of making `System.out` a value. `printf` and
`append` answer the stream in Java, so they CHAIN — `System.out.printf(...)`
`.println(...)` was refused in as many words ("chained print calls are not
supported"). And `System.out.close()` was "cannot find symbol" while
`held.close()` on the same object worked: the print calls keep their direct
route (it is where the `println` overloads are chosen) and everything else is
now an ordinary call on the value, so the two agree.

Pinned by `diff_junit_captures_standard_out` (against real JUnit, including a
test that FAILS while capturing), `capturing_what_a_program_prints`,
`a_captured_buffer_survives_a_collection`, `a_print_stream_is_a_value`, and the
`capture-standard-out` feature on the compatibility page.

### The validators nobody compiles

The grading sweep stages a level only when it ships a `solution` — there has to
be something to grade. **Eighty-five levels have a validator and no solution**,
so nothing had ever compiled their validators: they are what a student's
submission is graded against, and no tool here had looked at them.

Compiled beside their `start` files and compared with javac (the JUnit jars and
the real `org.code` classes on its classpath): 64 compile in both, 20 are
refused by both — they name a class the student has yet to write — and **one
was refused only by caturra**.

`partialMockBuilder(PainterPlus.class).addMockedMethod(...).createMock()`: the
target's only constructor takes four arguments, and the generated mock subclass
declared none of its own, so its implicit `super()` named a constructor
`PainterPlus` does not have. Real EasyMock never runs a constructor at all
(Objenesis), and caturra already synthesized a forwarding one with default
argument values — but only for a FULL `createMock(T.class)`. Both shapes need
it. (The level still cannot be graded to a PASS: its `@BeforeAll` runs the
student's unwritten `main`, and the real reference produces no verdicts for it
either. What changed is that it compiles, which is where the student's own work
has to start.)

### A `@BeforeAll` that fails

Reading that validator turned up a second thing. Its setup is the shape most of
the neighborhood validators use:

```java
@BeforeAll public static void setup() {
    try { ... } catch (Exception e) { fail(message); }
}
```

JUnit does not run the tests when `@BeforeAll` fails — a listener sees no
verdict for any of them. caturra's runner swallowed the failure and ran each
test anyway, so what a student read was whatever each test then tripped over
(here, an empty message) rather than the sentence the level wrote for exactly
this case. The tests stay in the runner's PLAN, so a host still counts them:
announced and unreported is a failure, which is what the plan lines are for.

### Every level's first file

The same question, asked of every level's START files — the 6675 sets a student
opens before typing anything, which the staged sweep never sees on their own
(it merges them with the solution). Both engines agreed on all but seven.

The one caturra refuses alone is a student example that uses `Thread`, an
honest refusal. The other six were caturra compiling what javac refuses.

**Five are one program.** A `Cupcake` whose constructor calls `super(newFlavor,
newPrice)` in a class with no `extends` clause — the student is meant to add
it. The implicit superclass is `Object`, whose only constructor takes nothing,
and the arguments were being DROPPED: the call compiled as `super()` and did
not even evaluate them.

**The sixth is an import.** `class PainterPlus extends Painter` with no import
of its own compiled, because a SIBLING file imported
`org.code.neighborhood.Painter` — and the bundle that import injects is global
(there is one class table), so the name was in scope everywhere after it. javac
scopes an import to its own compilation unit. The course libraries need one
exactly as `java.util` does, and turning the rule on immediately caught two
invalid programs of caturra's OWN: a unit test and an end-to-end test, both
reading `SoundLoader` (which lives in `org.code.media`) under an
`import org.code.theater.*`. The playground's shipped levels import both, which
is why nobody noticed.

After both fixes: 6675 start-file sets, and the only disagreement left is the
`Thread` refusal. Across the 2698 STAGED cases, caturra and javac now agree on
every one — 2657 compile, 41 are refused by both.

Pinned by `easymock_partial_mock_of_a_class_with_no_default_constructor`,
`diff_junit_a_failing_before_all_runs_no_test` (against real JUnit),
`reject_super_arguments_with_no_superclass` and
`an_org_code_class_needs_its_import_in_every_file`.

### The levels the playground ships

The corpus in `artifacts/` is the source; what a student actually opens is the
generated `apps/playground/src/csa-units/unit-*.ts` — 724 levels, checked in
rather than vendored, and the one population where a refusal means a level is
broken today. Nothing had ever compiled them either.

**724 levels: 591 compile in both engines, 133 are refused by both — they name
a class the student has yet to write — and neither engine disagrees with the
other about a single one.**

All four populations are one script now (`scripts/sweep/compile.py --what
starts|validators|staged|levels`), because the finding each time was not a bug
in a rule but a set of files no tool had looked at. It is the cheapest question
there is — does this compile, and does javac agree — and it was never asked of
anything except the staged cases.

### java.time, without a timezone database

`java.time` was an honestly-refused package: "package java.time is not
supported by caturra (the class library covers the AP CS A subset)". It is
also the first thing a student reaches for the moment a program has a date in
it, so the refusal is a wall rather than a boundary.

The part that is worth having is the part that is **pure arithmetic**: a
`LocalDate` has no clock, no locale and no zone behind it, so every answer it
gives is exactly comparable with a JDK's. `LocalDate`, `DayOfWeek` and `Month`
are modelled now — the calendar (`crates/caturra-vm/src/time.rs`) is the
proleptic Gregorian one, with the month-end clamping rule (`Jan 31` plus a
month is `Feb 29` in a leap year, `Feb 29` plus a year is `Feb 28`), the
epoch-day conversion both ways, and `java.time`'s own exception text —
including the two spellings of the same complaint, `Invalid date 'FEBRUARY 30'`
beside `Invalid date 'February 29' as '2023' is not a leap year`, which are the
JDK's inconsistency and not a slip.

**What needs a zone asks the HOST.** `LocalDate.now()` has to know what "today"
is, and that is a timezone question. The browser already has the IANA database
— `Date.getTimezoneOffset()` reads it — so nothing is vendored and there is no
second copy of the database to fall out of date. The hook is
`ConsoleIo::zone_offset_seconds`, beside `now_millis`; a host without a zone
(the CLI, the test harness) is UTC, which is why a program that asks what today
is cannot be compared against a JDK any more than one that asks for a random
number can. The browser half is checked where it is true: an end-to-end test
runs `LocalDate.now()` in the page and compares it with the browser's own
`new Date()`.

**Vendoring the database was considered and rejected**, and the measurement is
worth recording: OpenJDK does NOT use the system database — it ships its own
`$JAVA_HOME/lib/tzdb.dat` — and on the machine this was written on the JDK is
on tzdb **2026b** while the OS is on **2026c**. So a vendored copy would not
have bought JDK parity either; it would only have added a third version to
disagree with. Zone RULES (a `ZonedDateTime`, `ZoneId.of("Asia/Tokyo")`) stay
unmodelled for that reason, and the honest contract for anything zone-shaped
is "correct per the host", not "byte-identical to a JDK".

Four things a date has to be beyond its own methods, each found by writing the
program a student would write: it SORTS (`Collections.sort` asks an intrinsic
for `compareTo`, and the dispatch resolved against class files only, so a
`LocalDate` was "not Comparable"); it HASHES (a `HashSet` held two equal
dates); it compares by VALUE inside a collection (`contains` and `indexOf` said
no); and it renders the same inside a collection as it does alone (a list of
dates printed `object@1c`). The last three are the same defect the
collection-renderer and the equality path have had before — one fact, read
somewhere else.

Pinned by `the_calendar_arithmetic_of_local_date`, `local_dates_are_values`,
`what_java_time_says_when_a_date_is_wrong`, the `local-date` feature on the
compatibility page, the browser test above, and three unit tests over the
calendar itself — including one that round-trips every day from 1800 to 2200
through the epoch-day conversion.

### The rest of the time and date routines

`LocalDate` was the half of `java.time` a program can do arithmetic with; this
is the other half of what a student writes — **`LocalTime`, `LocalDateTime`,
`Duration`, `Period` and `ChronoUnit`** — and it is the same bargain: all of it
is pure computation, so all of it is exactly comparable with a JDK.

Four rules are the whole of what makes these types feel right, and each is
copied rather than invented:

- **A time WRAPS; a date-time CARRIES.** `LocalTime.of(23, 0).plusHours(2)` is
  `01:00` and remembers nothing; `LocalDateTime`'s is `2024-05-16T01:00`. That
  is one line of arithmetic (`overflow_days`) and the only thing a date-time
  does that its two halves do not.
- **`toString` leaves out what is zero, in groups of three.** `10:15`,
  `10:15:30`, `10:15:30.500`, `01:02:03.000000004` — a fraction prints as three
  digits, six or nine, never as written.
- **A `Duration` is an amount of time and a `Period` is a number of years,
  months and days.** `Duration.ofDays(2)` prints `PT48H` (a duration's text has
  no days in it); `Period.ofMonths(1)` is `P1M` and never becomes thirty days.
  The signs sit per-part (`PT-1M-30S`, `P-1M-30D`), and a negative second with
  a fraction reads as one number (`PT1.5S`, and `-0.5s` as `PT-0.5S`).
- **`Period.between` counts whole months first**, so `2024-01-15` to
  `2025-03-20` is `P1Y2M5D` — not a day count divided up. `ChronoUnit.MONTHS`
  and `YEARS` use the same walk; every other unit is nanoseconds on a line,
  truncated toward zero.

`ChronoUnit`'s constants are stored under **the JDK's own ordinals**, so
`ChronoUnit.DAYS.ordinal()` is 7 and the two units caturra does not count
(`MICROS`, `HALF_DAYS`) still hold their places. Its `name()` is `DAYS` and its
`toString()` is `Days`, which are different strings — an enum with a
description of its own.

`DayOfWeek.values()` and `Month.values()` answer a real array now (looping over
an enum's constants is how a program uses one), which needed an element type
for a library enum beside the one `File[]` has.

Pinned by `the_time_of_day_and_the_date_time`,
`how_long_something_took_and_how_far_apart_two_dates_are`,
`looping_over_the_days_and_the_months`, `times_are_values`,
`building_a_date_time_from_its_halves`, and the `times-and-spans` feature on
the compatibility page.

### Printing a date the way a pattern asks

`DateTimeFormatter.ofPattern` is the last of `java.time` a student writes, and
the one place its behaviour is a small language of its own. The letters
modelled are the ones a program uses — `y`/`u`, `M`, `d`, `D`, `E`, `H`, `h`,
`m`, `s`, `S`, `a` — with quoted text between them, formatting either way round
(`value.format(fmt)` and `fmt.format(value)`), parsing back through the same
pattern, and the ISO constants.

Four things had to be read off a JDK rather than assumed:

**A letter caturra does not model is not an invalid pattern.** `java.time`
knows `q` (quarter), `G` (era), `z` (zone) and a dozen more; refusing them as
"Unknown pattern letter" would be refusing valid Java. There are three answers
now: a letter `java.time` does not know gets its message
(`Unknown pattern letter: b` — and the set is exactly `BCIJPRTUbfijlort`), a
letter it knows but caturra does not model gets a refusal of ours that says so,
and the rest are formatted. Each letter also has its own LIMIT — `ddd` is
"Too many pattern letters: d", not a three-digit day.

**`toString` prints the printer, not the pattern.** A JDK's formatter describes
what it built: `Value(DayOfMonth,2)'/'Value(MonthOfYear,2)'/'Value(YearOfEra,4,19,EXCEEDS_PAD)`.
Every shape in that description is mechanical once the pattern is parsed — the
sign style changes at four letters, a two-letter year is a `ReducedValue`, a
day-of-week is always `Text` — so it is produced exactly rather than
approximated with the pattern text. It also showed that a quoted run is ONE
piece and each unquoted character is its own (`', '` is `','' '`).

**An ISO constant is not the value's `toString`.** `ISO_LOCAL_TIME` always
writes the seconds (`14:05:00` where `toString` gives `14:05`) and its fraction
carries only as many digits as it needs (`14:05:09.5` where `toString` gives
`.500`). Modelling the ISO formatters as "print what the value prints" was
wrong in exactly that way, and the compatibility page caught it — the recorded
JDK output and the live one disagreed on one line.

**`ofPattern` resolves SMARTLY.** `LocalDate.parse("2024-02-30", ofPattern("yyyy-MM-dd"))`
is not an error: the day is pulled back to the 29th. A field outside its range
altogether IS an error, and a differently-shaped one — "could not be parsed:
Invalid value for MonthOfYear (valid values 1 - 12): 13" rather than "could not
be parsed at index 4".

Pinned by `formatting_a_date_the_way_a_pattern_asks`,
`what_a_pattern_says_when_it_is_wrong`, and the `date-patterns` feature on the
compatibility page.

### Fuzzing the calendar

The `java.time` work was checked against a JDK battery by battery, and every
battery passed. `scripts/fuzz/time.py` asks the same question of a few hundred
dates instead of a dozen — random ones over four centuries, plus the edges a
calendar trips over (year zero, the far side of the epoch, every February
29th, the four-hundred-year cycle, five-digit years) — and found **five**
divergences on its first run. That is the argument for the tool in one line:
hand-written cases are the cases someone thought of.

**`ChronoUnit.DAYS.between` overflowed a long.** Two dates two millennia apart
are more nanoseconds than an `i64` holds, and reducing every temporal to
nanoseconds-on-a-line — which is right for hours and minutes — silently wrapped
for days. Days and weeks are counted as DAYS now, from the epoch day, with the
time of day deciding only whether the last one is whole.

**`HOURS.between` on two DATES.** A `LocalDate` has no hour, and `java.time`
says so ("Unsupported unit: Hours") rather than treating it as midnight.

**A negative `Duration` signs every part of its text.**
`Duration.between(23:59:59.999999999, NOON)` is `PT-11H-59M-59.999999999S`, not
`PT-12H0.000000001S` — the whole seconds are counted from `seconds + 1` and the
fraction is the complement, which is a rule you cannot guess at.

**`y` is the year OF THE ERA.** It is never negative: for the year -1 a `yyyy`
pattern writes `0002`, because that is the year 2 of the era before this one.
`u` is the proleptic year and carries its sign. Both pad DIGITS, with the sign
outside the width and a `+` when the number needs more room than the pattern
gave it — and a signed year is read back the same way, which is what makes a
five-digit year round-trip.

**The narrow text forms are one letter** (`EEEEE` is `M` for Monday), and a
literal quote inside a pattern describes as `''` rather than as a quote inside
quotes.

Pinned by `the_awkward_edges_of_a_calendar`, and the fuzzer is checked in.

### The rest of what a date is asked

The last of the `java.time` surface a program actually reaches for: `until` in
both its forms (a `Period`, or a count in one unit), `withDayOfYear` and
`ofYearDay`, `truncatedTo` on a time and on a date-time, a `Period`'s own
arithmetic (`plusDays`, `withMonths`, `multipliedBy`, `negated`,
`normalized` — which folds MONTHS into years and leaves the days alone,
because a day is not a fixed part of a month), a `Duration` read in parts
(`toHoursPart` is the hours of a duration that also has minutes in it, not the
whole of it in hours), and the two enums ROTATING — Saturday plus three days
is Tuesday.

That takes the measured surface to `LocalDate` 35/50, `LocalDateTime` 44/63,
`Duration` 43/52, `Period` 24/33 and `LocalTime` 28/43. What is left in each is
almost entirely `TemporalAccessor` plumbing — `adjustInto`, `query`, `getLong`,
`isSupported`, `range`, `from`, the generic `plus(TemporalAmount)` — which is
how the library talks to ITSELF rather than anything a program writes, plus
`datesUntil` (a stream of dates) and the formatter's locale and zone
configuration, which is the part deliberately not modelled.

Pinned by `the_rest_of_what_a_date_is_asked`.

### Fuzzing the regex engine

The calendar fuzz paid for itself, so the same question was put to the other
thing here that is a hand-written engine: `crates/caturra-vm/src/regex.rs`.
`scripts/fuzz/regex.py` builds patterns from a grammar (quantifiers greedy,
reluctant and possessive; classes; alternation; nested groups; anchors;
boundaries; backreferences; lookarounds), draws inputs from the pattern's OWN
characters so a fair share of them match, and compares every observable
answer — `matches`, each `find`'s span and groups, the group count, `split`,
`replaceAll`, and the exception a bad pattern throws.

**6300 probes, two divergences**, and both are the same fact about
`java.util.regex`: it keeps ONE group array for a whole `find()` and only ever
writes to it.

**A capture outlives the attempt that made it.** `Matcher.find()` clears the
groups once and then walks the start positions itself, so a group set while
trying (and failing) at an earlier position is still readable from the match
that eventually succeeds: `([abc])*+a*?1+` over `"bx xac 10 "` matches `"1"` at
7 and reports group 1 as `"c"`. caturra allocated fresh captures per start
position, which is tidier and answers `null` — a different string on the
screen. One set per search now.

**A negative lookahead's captures survive it.** The body matching is exactly
what makes a negative lookahead fail, and the groups it set on the way are not
taken back. `(?!(a)b)ab` against `"ab"` has group 1 set. The comment in the
engine said the opposite in as many words ("a negative one matched nothing, so
it captures nothing"), which is true of the MATCH and false of the group array.

**What is left is one probe in 6300, and it is honest to leave it.** A pattern
with two nested negative lookaheads reports a group caturra does not, because
what a failed branch leaves behind depends on WHICH branches were tried — and
that is the engine's search order, not a rule. Replicating it would mean
replicating `java.util.regex`'s backtracking order, which is a different
project from replicating its semantics.

Pinned by `a_capture_outlives_the_attempt_that_made_it`.

### Fuzzing the collections

Two engines fuzzed, so the third question: the collections, which is where a
student's program actually spends its time. `scripts/fuzz/collections.py`
builds one collection and applies a random sequence of calls, printing the
answer of EVERY call and the whole collection after it — so a divergence lands
on the operation that caused it rather than ten steps later — with each call
wrapped, so the operations that THROW are compared too. It also walks hash
collections through several resizes.

**Nine thousand lines over ten seeds, no divergence** across `ArrayList`,
`LinkedList`, `HashSet`, `LinkedHashSet`, `TreeSet`, `HashMap`,
`LinkedHashMap`, `TreeMap`, `ArrayDeque` and `PriorityQueue`: every return
value, every iteration order, every `toString`, every `hashCode` the interface
defines, and every exception.

Two things the fuzzer itself had to get right, and they are the same lesson in
two places. A `Deque`'s and a `Queue`'s `hashCode` is `Object`'s — an identity
hash, which differs between runs of the same JDK — so printing it compares
nothing; and an uncaught exception ends the program, which throws away every
comparison after it. **A fuzzer that compares a value the reference itself does
not reproduce is measuring noise, and one that stops at the first exception
measures a prefix.**

**The treeify boundary, measured exactly.** `HashMap` turns a bucket into a
tree at eight entries in one bin, and only once the table holds 64 — the two
constants are `TREEIFY_THRESHOLD` and `MIN_TREEIFY_CAPACITY`, and caturra
models neither. With deliberately colliding keys (`"Aa"` and `"BB"` hash alike,
and so does every concatenation of them) the boundary is where the JDK says it
is: **seven colliding keys agree, eight in a 64-entry table do not**, and eight
in a smaller table agree too, because a JDK resizes rather than treeifying
until the table is big enough. Below both thresholds the orders are identical
key for key. It stays open — reaching it needs keys chosen to collide, which no
program writes by accident, and closing it means a red-black tree bin with the
JDK's exact ordering rules.

Pinned by `the_bucket_order_of_a_hash_map`, which holds the agreeing side of
both thresholds.

## Which complaint a bad specifier gets

A format specifier can be wrong in more than one way at once — `% 1.0X` has a
flag that an uppercase hex does not take AND a precision that no integer
conversion takes — and a program only ever sees one exception. Which one is
not a matter of taste: `java.util.Formatter` checks in a fixed order, the
order differs per conversion family, and a student reading the message is
being told which mistake to fix first.

caturra checked in the order the fields are written — flags, then width, then
precision — which agreed with a JDK for the general conversions and disagreed
for the numeric ones. The rule, per family:

- general (`s`, `b`, `h`): the `0` flag is a MISMATCH for this conversion, not
  a missing width; `-` without a width is the missing-width error; then the
  contradictory pairs.
- character (`c`): a precision is illegal FIRST, then the flags, then a lone
  `-`.
- integer (`d`, `o`, `x`): `d` rejects `#`, and `o`/`x` reject `,`; then a
  precision is illegal; then a lone `-`; then the contradictions.
- floating point (`e`, `f`, `g`, `a`): `#` is illegal for `e`, `,` for `e` and
  `a`, precision for `a`; then a lone `-`; then the contradictions.

Two checks are not made while READING the specifier at all, and a program can
tell:

- `%#s` is legal until the argument arrives, because an argument that
  implements `Formattable` is handed the flag to interpret. So
  `String.format("%#s")` with NO argument raises
  `MissingFormatArgumentException`, and only a `%#s` that HAS a plain argument
  raises the flag mismatch.
- `%(20o` against a `Double` is `IllegalFormatConversionException`, not a
  flag error: a JDK dispatches on the argument's type first and reaches the
  `(`/`+`/space checks only once it is already printing an integer.

And a hash is text once it is written, so a precision truncates it the way it
truncates a string: `%.3h` of `"abcdef"` is the first three characters of the
hash, not the whole of it.

Pinned by `which_complaint_a_bad_specifier_gets` in
`crates/caturra-vm/tests/differential.rs`, and swept by `scripts/fuzz/format.py`
— 2783 random probes over six seeds, all agreeing.

## The atomic group, and what a group is left holding

`(?>X)` is real Java 11 syntax and this engine refused it as "Unsupported
group construct" while already having everything it needs: an atomic group
matches `X` once and never reconsiders, which is one possessive repetition.
`(?>a*)a` fails over "aaa" and `(?>a|ab)c` matches "abc" — the run is not
given back. It parses to `Repeat { min: 1, max: 1, Possessive }` over a
non-capturing group, so no node kind was added.

The rest of this section is about a question a program can ask and a JDK
answers with an implementation artifact: what is in a capturing group that
did not participate in the match it reports, or that participated more than
once. `java.util.regex` writes group boundaries into ONE array and puts them
back in exactly two places — a `GroupTail` whose continuation failed, and a
`GroupCurly`'s own restores — so everything else it wrote is still readable.
This engine used to snapshot the whole capture set around a branch, a
repetition and a lookaround, which is tidier and answers `null` where a JDK
answers text. It does not any more; the only restore left is the one on a
group's own tail, which is the JDK's.

Four rules follow from that, each measured:

- **An empty iteration ends a repetition** — and stands for every iteration
  the minimum still wanted, because repeating an empty match again would
  change nothing. So `(a??){3}b` over "aab" runs empty, backs into "a", runs
  empty, backs into "a", and is then DONE: group 1 is "", not the "a" a loop
  that filled its minimum with empty passes first would leave.
- **A fixed-width body writes its group again on the way out.** A quantified
  capturing group whose body always matches the same number of characters
  compiles to a `GroupCurly`, whose backing-off sets the group AFTER its
  continuation returned true. The pass that stopped EARLIEST therefore has
  the last word: `((.){1,3})*` over "abcde" ends with group 2 as "c" — the
  third character, from the first pass — and not the "e" the second pass
  wrote. A fixed COUNT (`((.){3})*`) never reaches that code, and neither
  does a variable-width body (`((.|xy){1,3})*`).
- **An optional repetition that consumes nothing leaves its group unset.**
  `((?!x))*y`, `()*y` and `(\b)*y` all report group 1 as `null`, where
  `((?!x))+y` reports "". `?` is not a counted closure — a JDK compiles `X?`
  to a branch and `X{0,1}` to a loop — so `((?!x))?$` says "" and
  `((?!x)){0,1}$` says `null`.
- **A capture a failed branch made is still readable.** `(?=(a))?b|a` over
  "ab" matches "a" and reports group 1 as "a": the lookahead ran, captured,
  and the branch it was in then failed. Same for a group inside a NEGATIVE
  lookahead whose body matched, and for the last, failed iteration of a
  possessive repetition.

Pinned by `an_atomic_group` and `what_a_group_is_left_holding` in
`crates/caturra-vm/tests/differential.rs`, and swept by `scripts/fuzz/regex.py`
— 40 seeds, roughly 26000 pattern-against-input probes, all agreeing.

## Every construct in the table

`java.util.regex.Pattern`'s javadoc opens with a table of every construct the
engine knows — 103 rows, from `\\0n` to `(?>X)`. One probe per row, compared
against a real JDK, found forty that caturra refused or answered differently.
All but one are now closed, and the table is pinned as
`every_construct_in_the_table`.

What was missing, and what it took:

- **`\\p{...}` and `\\P{...}`** — the whole named-property family, which is
  the largest thing here. The names resolve exactly as `Pattern.family` does:
  a `key=value` form (`gc`, `general_category`, `sc`, `script`, `blk`,
  `block`), an `In`-prefixed block, an `Is`-prefixed binary property or
  category or script, and otherwise a category or POSIX or `java...` name.
  The POSIX names are **US-ASCII** — `\\p{Alpha}` is `[a-zA-Z]` and nothing
  else — while `\\p{IsAlpha}` and, under `(?U)`, a bare `\\p{Alpha}` mean the
  Unicode set. Those names are case SENSITIVE where the `Is` ones are not.
- **`\\p{IsLatin}` and `\\p{InGreek}`** — scripts and blocks, recorded from a
  real JDK: 913 script runs over the BMP, every script the enum knows
  (including the ones with nothing in the BMP, since `\\p{IsDeseret}` still has
  to compile), the ISO 15924 aliases, and the 162 blocks with every name
  `UnicodeBlock.forName` answers to.
- **`\\h \\H \\v \\V`** — Perl's horizontal and vertical whitespace, which are
  not `\\s`.
- **`\\cX`** — the control character, and **`\\x{...}`** above the BMP, which
  used to become its high surrogate alone and match nothing.
- **`\\G`** — where the previous match ended, which is not always where the
  next search begins: after an EMPTY match the search moves on by one and
  `\\G` does not.
- **`(?d)`** — `UNIX_LINES`, where only `\\n` ends a line for `.`, `^`, `$`
  and `\\Z`. It parsed and was ignored, so `(?d)a$` matched "a\\r\\n".
- **`(?U)`** — `UNICODE_CHARACTER_CLASS`, where `\\w`, `\\d`, `\\s`, `\\b` and
  the POSIX names mean their Unicode sets. It parsed and was ignored too.
- **`\\X` and `\\b{g}`** — the extended grapheme cluster, which is what a
  reader calls one character however many code points it takes: a flag is two
  regional indicators, a family emoji is three people and two joiners, `\\X`
  takes each in one bite. Annex #29's rules are derived from the general
  category, so this needed no table of its own.

The one still open is `\\N{LATIN SMALL LETTER A}`, which needs the Unicode
NAME database — thirty thousand strings for a construct no program writes.
caturra refuses it, in the JDK's own words.

Underneath, `java.lang.Character` grew a supplementary half. Its category table
covered the BMP, so `Character.getType` and every predicate derived from it
answered for a `char` and guessed above it. The runs above the BMP are recorded
too now — 844 of them — and the predicates take a CODE POINT. Two properties no
category implies, `Bidi_Mirrored` and `Ideographic`, are recorded as ranges.
`isJavaIdentifierStart` and its four relatives were reading "alphabetic, or `$`
or `_`" where the rule is two whole categories (currency symbols and connecting
punctuation), and `isTitleCase` was inferred from the case mappings rather than
read from the category. All of it — every category and twelve predicates over
all 1114112 code points — is pinned as `every_category_over_the_whole_space`.

One JDK inconsistency is written down rather than smoothed over: U+9FEB..U+9FEF
are `OTHER_LETTER` by `Character.getType` and are refused by
`isJavaIdentifierStart`. They were assigned in Unicode 10, which JDK 11 carries,
and the separate table behind the identifier predicates was not extended to
cover them.

And a lone surrogate now survives `Matcher.group()`: the text was going through
a Rust `String`, where an unpaired surrogate becomes U+FFFD.

## A string of code units

A Java `String` is a sequence of `char` — UTF-16 code UNITS — and three layers
of caturra used a Rust `String` instead, which cannot hold one of them. An
unpaired surrogate is a legal `char`: `"\\uD83D"` is a one-character string a
JDK prints, hashes and reverses like any other, and a Rust `String` turns it
into U+FFFD on the way past. So did every answer downstream.

A sweep of the whole `String` surface over text that is not all Basic
Multilingual Plane — an emoji, a Deseret pair, a lone high surrogate, a lone
low one — diverged on 60 of 307 answers. They had five causes:

- **The string LITERAL** was carried as a Rust `String` from the lexer to the
  class file's constant pool. It is UTF-16 units now, all the way through:
  `TokenKind::StringLiteral`, `Literal::Str`, a new `Constant::Utf8Units` for
  the pool entries that are not text, and a string pool keyed by units. The
  class file's two long-standing TODOs went with it — the constant pool now
  reads and writes **modified UTF-8** (JVMS §4.4.7), where U+0000 takes two
  bytes and everything above the BMP is written as its surrogate pair, which
  is also what gives an unpaired one a spelling.
- **`String.format`** built its output as a Rust `String`. A precision counts
  UTF-16 units, so `"%.3s"` of "a😀b" keeps the whole surrogate pair and
  `"%.2s"` keeps the high half ALONE — a `char` that has to survive to the
  output. The formatter works in units now, and `%S` uppercases with the JDK
  11 tables rather than Rust's newer ones.
- **`toUpperCase` and `toLowerCase`** mapped unit by unit, so the five
  supplementary scripts with a case — Deseret, Osage, Warang Citi, Adlam and
  Medefaidrin — were left alone. They map by CODE POINT now.
- **`codePointBefore`** read forwards: at index 2 of "a😀b" it combined the
  pair that STARTS there and answered the whole emoji, where a JDK answers the
  high surrogate alone. The unit before an index is a unit unless it is the
  low half of a pair.
- **`trim`, `strip`, `join` and `Matcher.group`** each round-tripped through a
  Rust `String` for no reason. They work in units.

`Character` grew the code point half it was missing while this was measured:
`isValidCodePoint`, `isBmpCodePoint`, `isSupplementaryCodePoint`,
`highSurrogate`, `lowSurrogate`, `isSurrogatePair`, the `CharSequence` forms of
`codePointAt`, `codePointBefore` and `offsetByCodePoints`, and the three
methods deprecated in Java 1.1 that are still there in 11.

All 307 answers agree, pinned as `a_string_of_code_units`.

What is still a Rust `String`, and so still lossy for an unpaired surrogate:
constant FOLDING (`"a" + "\\uD83D"` written as one constant expression), and
the bytes a `PrintStream` writes — where a JDK's encoder substitutes `?` and
caturra now does too, so only `System.out.print` of a lone surrogate differs
from the string it was given.

## A date as a Temporal

A `LocalDate` is not only its own getters. Underneath, `java.time` is built on
two interfaces a program can name directly: a `TemporalField` — one number a
value holds — and a `TemporalUnit` — one step a value can be moved by. Both are
enums, `ChronoField` and `ChronoUnit`, and caturra knew only part of the second.

`ChronoField` is here now, all thirty constants, and with it the whole
`TemporalAccessor` surface on `LocalDate`, `LocalTime` and `LocalDateTime`:

- `isSupported(field)` — a date has no hour and a time has no month.
- `getLong(field)` and `get(field)`, which differ in more than a cast: four
  fields (`NANO_OF_DAY`, `MICRO_OF_DAY`, `EPOCH_DAY`, `PROLEPTIC_MONTH`) are
  too wide for an `int`, and a JDK refuses them from `get` even when the value
  would have fitted.
- `range(field)` — a `ValueRange`, which is four numbers and not two: February
  has 28 days some years and 29 others, so `DAY_OF_MONTH` has a smallest
  maximum of 28 and a maximum of 31, and prints as "1 - 28/31".
- `with(field, value)`, whose ORDER of complaints is its own rule. The value is
  checked against the FIELD's own range first, so
  `LocalDate.with(HOUR_OF_AMPM, 30)` complains about the 30 rather than about a
  date having no hour. Then the field must be one the value has. Only then is
  the date built, which is where "Invalid date 'FEBRUARY 30'" comes from.

`with` is not one operation either. A "-of-second" field REPLACES the whole
nano-of-second, so setting the microsecond clears the nanoseconds beneath it; a
"-of-day" field replaces the whole day; a week field moves by weeks and a day
field by days; and the era is a reflection about year one.

`ChronoUnit` gained the five constants above `YEARS` — `DECADES`, `CENTURIES`,
`MILLENNIA`, `ERAS` and `FOREVER` — and the questions a unit answers about
itself: `getDuration` (an estimate: a month is a twelfth of the average
Gregorian year, and `FOREVER` is as long as a `Duration` goes),
`isDateBased`, `isTimeBased`, `isDurationEstimated`, `isSupportedBy`,
`compareTo`, `values` and `valueOf`. With them, `plus(amount, unit)`,
`minus(amount, unit)` and `until(end, unit)` work on all three values, and
`truncatedTo` refuses a unit that does not divide a day — in the JDK's own
words, which name no unit at all.

Pinned as `a_date_as_a_temporal` (1637 answers) and `every_chrono_unit` (339).

## Adjusting a date

The other half of what a `LocalDate` is asked. `java.time.temporal.TemporalAdjusters`
is thirteen factories, each a RULE for moving a date, which `with` applies:
`firstDayOfMonth`, `lastDayOfMonth`, `firstDayOfNextMonth` and their year
counterparts, and the six that hunt for a weekday — `firstInMonth`,
`lastInMonth`, `dayOfWeekInMonth`, `next`, `nextOrSame`, `previous`,
`previousOrSame`. `next` is strictly forwards where `nextOrSame` stays put, and
`dayOfWeekInMonth` counts from the END of the month when its ordinal is
negative.

`with` also takes a VALUE rather than a rule: a `Month` or a `DayOfWeek` sets
its own field, and a `LocalDate` or `LocalTime` replaces that half of a
`LocalDateTime`. And `plus`/`minus` take a whole amount. A `Period` is not
three separate additions: a JDK adds `years * 12 + months` as ONE number of
months and then the days, and adding the years first would clamp February 29 to
the 28th on the way through — which is the difference between 2025-05-02 and
2025-05-01 for `2024-02-29.plus(P1Y2M3D)`.

The rest of the surface came with it: `LocalDate.getEra` (and so
`java.time.chrono.IsoEra`), `datesUntil`, `LocalDateTime.withDayOfYear`,
`LocalTime.ofNanoOfDay` and `toNanoOfDay`, `Month.firstMonthOfQuarter`, the
`from(temporal)` factory on all five types, `Duration.parse` and `Period.parse`
(where `P1W` is seven days, and a fraction belongs to the seconds and nothing
else), `Duration.of(amount, unit)` — which refuses a unit whose length is an
ESTIMATE, `DAYS` excepted — `Duration.truncatedTo`, `toDaysPart`, and `get`
and `getUnits` on both amounts, which hold exactly two and exactly three units
respectively.

Two JDK messages name no unit, where an obvious reading would: "Unit is too
large to be used for truncation" and "Unit must not have an estimated
duration". And `Duration.toHoursPart` is the hours WITHIN the day, not the
whole span, which `toHours` is.

Pinned as `adjusting_a_date` — 601 answers.

## Every date-time conversion

`String.format`'s third family. `%tY`, `%tB`, `%tR` and their twenty-eight
relatives take a moment and print one piece of it, and caturra refused every
one of them — with a comment saying it had "no Date, Calendar or java.time",
which stopped being true four units ago.

A `java.time` value answers every conversion whose FIELD it has, which is the
same question `getLong(field)` asks, so the two share one reader. The rest
falls out of that:

- A `LocalDate` has no hour, so `%tH` of one is
  `IllegalFormatConversionException: H != java.time.LocalDate`. For a
  COMPOSITE the letter named is the piece it stopped on, not the composite's
  own: `%tR` over a date reports `H`, and `%tc` over a date-time reports `Z` —
  the zone name, which a local value does not have. That is not a choice; a
  JDK builds a composite by recursion and catches the failure one frame in.
- `%tY`, `%ty` and `%tC` read the year of the ERA, so 45 BCE prints as `0045`,
  and `%tF` — which is `%tY-%tm-%td` — prints `0045-03-15`.
- `%T` uppercases the whole rendering, which only the NAME conversions notice,
  and it uppercases the suffix in its own error text too: `%-Tp` writes itself
  as `%-TP`.
- A `Duration` and a `Period` are not moments at all, so they never reach the
  printer — and a composite over one names its OWN letter, where a `Month`
  (which IS one field of a moment) names the piece.

The validation order is its own rule again, as it was for the other families:
a PRECISION is the first complaint — a date-time conversion has none — then
the flags in the JDK's order (`#`, `+`, space, `0`, `,`, `(`), then a lone `-`
without a width.

Still refused, and now for a stated reason: `%t` over a `long`. That means
milliseconds read in the DEFAULT TIME ZONE, and this engine vendors no time
zone database — the same reason `LocalDate.now()` is not compared against a
JDK either.

Pinned as `every_date_time_conversion` — 839 answers — and swept by
`scripts/fuzz/format.py`, which pairs a date-time conversion only with a
`java.time` value for exactly that reason.

## Every pattern letter

`DateTimeFormatter.ofPattern`'s javadoc opens with a table too — thirty-five
letters, each meaning something different at each count it is written. caturra
knew twelve of them and refused the rest with a message of its own.

Asking a JDK all thirty-five at counts one to five, over a date, a time, a
date-time and a year before the era, is 926 answers. caturra differed on 568.

What was missing, and is not now:

- **`G`** the era ("AD", "Anno Domini", "A" by count), **`K`** the hour within
  the half-day and **`k`** the clock hour of the day, **`A`** the millisecond
  of the day, **`n`** the nanosecond of the second and **`N`** of the day.
- **`L`** and **`q`**, the STANDALONE month and quarter, which in English read
  exactly as `M` and `Q` do — and `Q` itself, whose four-letter form is "1st
  quarter".
- **`W`** and **`F`**, the aligned week of the month and the aligned day of
  the week in it.
- **`e`** and **`c`**, the day of week counted the way the LOCALE counts it —
  this formatter is en-US throughout, where the week begins on Sunday, so
  Thursday is 5 — and **`Y`**/**`w`**, the week-based year and its week
  number under the same rules.
- **`p`**, which pads whatever comes after it: `ppppHH` is "  13".
- **`[...]`**, an optional section, which prints only when the value has every
  field inside it. caturra printed the brackets.
- `''` inside a quoted run is one quote, so `'It''s'` is "It's". caturra
  dropped it.
- `{`, `}` and `#` are RESERVED — held back for a future release — and a JDK
  refuses a pattern containing one rather than printing it.

The zone letters — `V`, `z`, `O`, `X`, `x`, `Z` — parse and then fail where a
JDK fails, in a JDK's own words, because no `LocalDate`, `LocalTime` or
`LocalDateTime` carries a zone: an offset letter is a missing FIELD
("Unsupported field: OffsetSeconds") and a zone letter is a missing zone
("Unable to extract ZoneId from temporal 2024-02-29T13:45:30.123456789").
That is the whole of what they can do without a time zone database.

Two smaller things fell out. `uu` of the year -44 is "44" — the last two
digits of the MAGNITUDE, not the 56 a modulo of a negative gives — and the
width of a signed year counts digits, so `YYYY` of -44 is "-0044". And when a
pattern parses but into the wrong kind, a JDK says which type it wanted and
what it resolved instead ("Unable to obtain LocalTime from TemporalAccessor:
{},ISO resolved to 2024-02-29 of type java.time.format.Parsed") rather than
pointing at an index.

Pinned as `every_pattern_letter`.

## A java.time value in every position

Every `java.time` type placed where a program can put a value — a switch
selector, an `EnumSet`, a stream's element, a `Comparable` bound, a sorted
collection — measured against a JDK one construct at a time so a refusal of one
did not hide the rest. Fifty-nine constructs; seventeen disagreed.

They were not seventeen faults. Four causes account for all of them, and each
is a fact that was written down twice.

**A library enum is an enum.** `switch (date.getMonth()) { case FEBRUARY: … }`
was "incompatible types: Month cannot be converted to int": the switch checked
its selector against the classes the PROGRAM declares, and `Month` is not one.
The same reading refused `EnumSet.of(DayOfWeek.SATURDAY, DayOfWeek.SUNDAY)` —
the line every EnumSet tutorial opens with — as "needs an enum type", and left
`Arrays.stream(ChronoUnit.values())` with no element, so every lambda after it
was "only allowed where a functional-interface type is expected". Which library
types are enums is now ONE list (`LIBRARY_ENUMS`), read by the switch, by the
`EnumSet` factories and by the lambda pass. The VM needed nothing: an enum set
is built from the universe `values()` answers and reference identity, and the
constants are interned.

**A library call answers a type, and the emit side already knew which.** The
lambda pass typed a library call from a hand-written table that stopped at
`String` and the collections, so `LocalDate.of(2024, 2, 29)` had no type at all
— and `Stream.of(LocalDate.of(…), …).map(LocalDate::getYear)` was "cannot find
symbol: method getYear(), location: class Object" while the same dates through
a declared `List<LocalDate>` had always worked. The emitter's own method tables
carry a DESCRIPTOR per entry, and its return half is the answer; the pass reads
that now, and the hand-written arms are only the generic answers a descriptor
has erased (a stream's element, a receiver passed through). Overloads of one
arity that disagree answer nothing, since the parameter types are what tell
them apart and this pass has not resolved those.

**An ordered value is `Comparable`.** `<T extends Comparable<T>> T biggest(
List<T>)` — the first generic method a course writes — refused a list of dates,
because no `java.time` type had the interface in caturra's type system, at the
value level or the ELEMENT level. Both now do, for the ones a JDK orders. An
amount does not: `Period` is not `Comparable` ("1 year 2 days" and "1 month 40
days" have no order between them), nor is a `ValueRange`, and a `TreeSet` of
either is a `ClassCastException` naming `java.lang.Comparable` — which caturra
used to report as a gap in its own library, "unknown native member
ValueRange.compareTo".

**A method reference's qualifier and a functional interface's name are
different questions.** Making every library class a legal method-reference
qualifier had also made `Runnable` and `Supplier` read as classes the PROGRAM
declared, so the erased `java.util.function` treatment was skipped for them and
`Runnable r = () -> {};` was refused as "not a functional interface". The two
sets are separate now.

Two things fell out that had nothing to do with dates. A seeded fold over an
object pipeline answered the UNBOXED number a functional call hands back, so
`Stream.of(1, 2).reduce(0, Integer::sum)` assigned to anything was a
`VerifyError` — an object stream's elements are references, and its fold
answers an element. And `EnumSet.range(JUNE, MARCH)` said "from > to", naming
the parameters of a method the program never wrote, where a JDK names the two
constants.

What still differs is three things, all understood. Extending `HashMap` is the
documented refusal it always was. `groupingBy(LocalDate::getEra, …)` gathers
into a `HashMap` keyed by an enum, whose iteration order is the identity hash
a JDK gives that run — not reproducible by anything. And `Comparable<Month> c =
aLocalDate;` is accepted where javac refuses it, which is the same hole
`Comparable<Integer> c = "x";` has always had: the type ARGUMENT of a
`Comparable` target is not read.

Pinned as `java_time_in_every_position`, `an_object_folds_answer_a_reference`
and `a_backwards_enum_range_names_its_ends`.

## The type argument a target names

`Comparable<X> c = value;` for every pair of thirteen arguments and eleven
values — 143 cells against a real JDK. Seventy-five agreed. The other
sixty-eight were one fact: **the argument was thrown away.**

`Comparable<Integer>` resolved to the RAW `Comparable`, on the reasoning
written beside the code that dropped it — that it is "the erased face a wrapper
widens to", and tracking the argument would refuse `Comparable<Integer> c = 5;`.
Dropping it made the target exactly that erased face, so every value in the
language assigned to it: `Comparable<Integer> c = "x";` compiled, and so did a
date, a `Duration`, an enum constant of any type. The two conversions the
erasure stood in for are written out now — a primitive BOXES into a
parameterized `Comparable`, and a value whose element rides interned
(`LocalTime`, `Duration`, which have no element kind of their own) reaches its
own — so the argument can be carried and every cell answers.

Two neighbours of that fact were unwritten in the same way. An ENUM's supertype
arguments are ITSELF — `enum Kind` is an `Enum<Kind>`, which is a
`Comparable<Kind>` — and neither appears in its source, so nothing recorded
them and an unrecorded argument reads as unchecked: `Comparable<String> c =
Kind.A;` and `Enum<Kind> e = Other.X;` both compiled. And a `java.time` enum was
not an `Enum` at all here, where a JDK's is: `Month.MAY instanceof Enum` was
false, `Month.class.isEnum()` was false, `getSuperclass()` said `Object`,
`Enum<Month> m = Month.MAY;` was refused, and `Enum.valueOf(Month.class, "MAY")`
had no overload — while the identical five lines about a program's own enum all
worked.

Then forty-four positions for the same target — a parameter, a field, an array
element, a type argument, a cast, a return, a bound, `instanceof`, the sibling
faces (`Iterable<E>`, `Iterator<E>`, `Comparator<E>`, `Collection<E>`) — and
twenty-six shapes for a tracked argument on a user generic. All agree.

Three things fell out. `(int) aComparableOfInteger` is a checked cast plus an
unboxing conversion, and the arm that knew that matched only the RAW spelling,
so tracking the argument turned an accepts-invalid into a false refusal.
`Character`, `Byte` and `Short` answer their `compareTo` as a DIFFERENCE — `'c'
.compareTo('a')` is 2, not 1 — where the static `compare` of each already did,
one fact on two paths. And every array is `Cloneable` (JLS 10.7, which is what
makes `arr.clone()` legal): `instanceof` said so and the assignment did not.

The remaining known gap in this direction is a generic FACTORY's argument,
which javac infers from the target: `List<Number> l = List.of(1, 2);` is refused
here, though `take(List.of(1, 2))` into a `List<Number>` parameter is not. That
is target-typed inference, and it is its own unit.

Pinned as `a_library_interfaces_type_argument`, plus eight refusals pinned by
their WORDING, which javac's matches exactly.

## A factory's argument comes from the target

`List<Number> numbers = List.of(1, 2);` was refused — "List<Integer> cannot be
converted to List<Number>" — and so was every other spelling of the same line.
Forty-four of them, measured against a JDK: `Set.of`, `Map.of`, `Arrays.asList`,
`Collections.singletonList`, `Optional.of`, `Stream.of`, a copy constructor, a
stream's `collect`, in a variable, a field, a return, an argument, an array
element, a conditional. Eleven agreed.

The cause is JLS §15.9.1 and §18.5.1: a diamond and a call to a generic METHOD
are POLY expressions, whose type argument comes from where the value is going
rather than from what it was given. caturra knew this for the diamond — it had
a rule for exactly that, added when `List<Shape> s = new ArrayList<>(squares)`
was refused — and the rule tested only a `new` and only at a local declaration.
It is one predicate now: an expression that MINTS a container takes the
target's argument, wherever it is written.

Minting is what makes the retype sound. javac reaches the same answer by
inference; caturra reaches it by noticing that nothing else holds a reference to
the value under the narrower type, so nothing can put a `Square` into what a
`List<Circle>` variable still reads. A collection held in a VARIABLE is not
minted, and `List<Shape> s = circles;` stays the error javac calls it — as do
`List<Object> o = aStringList;` and `Map<String, Number> w = anIntegerMap;`,
which are the whole reason the rule cannot simply be "elements widen".

The element test grew to match. It compared two USER classes and nothing else,
so `List<Shape> s = List.of(new Circle());` worked as soon as the predicate did,
while `List<Number> n = List.of(1, 2);` — the plainest spelling — still did not:
a wrapper reaching `Number`, a `java.time` value reaching `Comparable`, anything
reaching `Object` all go through the same "does this element widen to that
class" question, which was already written down elsewhere. And it now covers the
containers a factory actually mints — an `Optional`, a `Stream`, a `TreeMap`, a
`LinkedList`, and a USER generic (`Box<Shape> b = new Box<>(new Circle());`).

An ARGUMENT is the one position that needs more than retyping: the call is
chosen before the argument is converted, so applicability has to consider the
poly form too. When nothing is applicable, resolution is tried once more with
each minting argument retyped to what the parameter says — and only when that
reaches exactly one candidate, so an ambiguity stays one.

Two limits remain, both the same boundary. `List<List<Number>> l = List.of(
List.of(1));` needs the inference to recurse through the nested factory, and a
user generic METHOD (`static <T> Box<T> boxed(T v)`, `Box<Number> b = boxed(1);`)
needs it to run over a signature rather than over a container. Both are refused,
which is the safe direction.

Pinned as `a_factorys_argument_comes_from_the_target`, with eight refusals
beside it — five wrong arguments to a factory, and three collections held in a
variable, which is what keeps the rule from being unsound.

## A generic method's variable comes from the target too

The factory rule stopped at the LIBRARY factories and the diamond. A program's
own generic method is the same poly expression (JLS §18.5.2), and thirty-five
shapes of one against a JDK found nine that disagreed — every one of them the
target asking for something wider than the call was handed:
`Box<Number> b = boxed(1);`, `List<Shape> l = listOf(new Circle(), new
Square());`, `Map<String, Number> m = pairOf("k", 1);`, and the same call in a
field, an array element and a conditional.

What tells a generic call from an ordinary one is already recorded: a method
whose return is an inferable type variable carries a plan for reading it off
the arguments, and a method without one carries `None`. The call now leaves a
MARK saying which it was, and the reader immediately after — the same one that
handles a diamond and a factory — takes the type argument from the target when
it does. The peek path leaves the mark too, since a conditional's branches are
only ever peeked at.

The freshness argument that carries the factory rule is not needed here, and
would be wrong: `<T> Box<T> shared()` may hand back the same box every time.
javac does not ask. It infers `T` from the target and then checks the ARGUMENTS
against it, which is what the element test does — so `Box<String> b = boxed(1);`
is still the error javac calls it, and a method that returns a concrete
`Box<Integer>` still cannot fill a `Box<Number>`.

One limit remains, the same one the factory unit left: inference does not
recurse, so `List<List<Number>> l = listOf(List.of(1), List.of(2));` is refused.
Reaching a level down needs to know that the inner call is poly as well, and
the safe answer is the refusal.

Pinned as `a_generic_methods_variable_comes_from_the_target`, with six refusals
beside it — a concrete return, an argument the target contradicts, one poly
branch of a conditional and one not, and a result already held in a variable.

## A lambda's parameter, through a class the program wrote

`new Box<>("ab").map(s -> s.length())` was "bad operand types" — `s` was an
`Object` — while `Box<String> b = new Box<>("ab"); b.map(s -> s.length())` one
line above compiled. Twenty-four shapes of the same question against a JDK; half
of them disagreed, and every one was the receiver's TYPE ARGUMENT not surviving
the way the receiver was written.

The lambda pass reads a receiver's argument to type the lambda it is about to
be handed. It read a declared variable, and a `new Box<String>(…)` written out.
It did not read:

- a DIAMOND of a program class. `new Box<>("ab")` was a raw `Box`; the argument
  is what the constructor was handed, and the erasure has already renamed the
  variable to a sentinel that carries its INDEX, which is exactly what to match
  a parameter against.
- a call to one of the program's own GENERIC methods. `boxOf("ab")` was
  nothing at all — the general reader never asked the question `body_type` had
  been asking all along.
- a method a class INHERITS. `Names extends Bag<String>` declares no `all()`,
  and looking only at the class named left the call untyped. The search walks
  outward now, and which class DECLARES the method is what says whose variables
  its return mentions.
- a variable INSIDE a written return. `Registry<String> r; r.all()` is a
  `List<String>`, and keeping the sentinel made it a list of nothing — so the
  stream after it had no element, and the lambda saw an `Object`.

The last two are one substitution, written once and asked from the three places
that need it: the general reader, the element reader a collection chain uses,
and the body reader a lambda's own chain uses.

Two limits are left, both refusals. A diamond with MORE THAN ONE type argument
(`new Pair<>("ab", 2)`) infers none — the plan behind the inference joins its
sources into a single answer, so a second variable has nowhere to go. And a
class type variable in a PARAMETER position is still unchecked: `Bag<String> b;
b.add(1);` compiles, where javac says "int cannot be converted to String". The
erased signature says `Object`, and codegen cannot tell that parameter from one
the program really wrote as `Object`.

Pinned as `a_lambda_through_a_user_generic`; four expressions join the position
mirror.

## A type variable in a parameter position

`Bag<String> b = new Bag<>(); b.add(1);` compiled. So did `b.add(new Circle())`,
`Bag<Circle> b; b.add("x")`, `Bag<Double> b; b.add(1)` — twenty-eight shapes
against a JDK found fifteen of them accepted where javac refuses, all one fact:
a parameter that is the class's own TYPE VARIABLE was checked against its
ERASURE.

Two rules said yes. Any reference stores into a `T` — true of the erasure, and
of nothing else — and a primitive handed to a `T` boxes first, which made it a
reference and then the first rule finished the job. The receiver's own argument
was already recorded (a `T` RETURN has read it for a long time); the argument
side simply never asked. It asks now, on both paths: the boxing happens only
when the boxed value is what the receiver's argument accepts, and a value the
argument does not accept is the error javac words identically —
"incompatible types: int cannot be converted to String".

Only while a receiver's arguments are actually known. A RAW receiver is
unchecked, which is what raw means; a call inside the generic class itself has
only the variable's bound to go on, and javac checks against the variable there,
which this does not model. A SUBCLASS that fixed the argument records nothing on
the receiver's own type — the receiver IS a `Names` — so the argument is read
off its `extends` clause for the length of the call.

Recording it there turned up the other half of the same omission: an inherited
return was substituted only when it was a BARE variable, so `Names extends
Bag<String>` read `T pick()` as a `String` and `List<T> all()` as a list of a
variable no class declares ("cannot find symbol … location: class T"). The
container substitution the parameterized-receiver path already had is asked from
both now.

Two limits remain, both accepting what javac refuses. A variable inside a
PARAMETER's own arguments (`void addAll(List<T> more)`) is still erased, and a
generic METHOD's own variable pinned by one argument is not checked against the
others (`static <T> void give(Bag<T> bag, T value); give(bagOfStrings, 1)`).

Pinned as `a_type_variable_in_a_parameter` — the shapes that must still compile,
including a raw receiver and a call from inside the class — with eleven
refusals pinned by their WORDING, which javac's matches exactly.

## A type variable inside a parameter

`Bag<String> b; b.addAll(listOfIntegers);` compiled — the parameter is
`List<T>`, and the erasure it was checked against is `List<Object>`. Twenty-five
shapes against a JDK found eleven accepted where javac refuses; five of them are
this, and the substitution that fixes them is the one the RETURN side has always
made, asked of the parameter as well.

It needed one thing the return side did not. A bare `T` keeps the parser's
POSITIONAL sentinel, and the previous unit read the index straight off it. A
variable inside a parameter's own arguments erases to a WILDCARD whose bound
names the declaring class instead — no index at all — so only a receiver with a
single argument can say which variable it was. That is `Bag<T>.addAll(List<T>)`,
the shape this is about; a class with two variables keeps the erasure it had,
which is the safe direction.

The wording is javac's exactly, down to the receiver's own spelling:
"incompatible types: ArrayList<Integer> cannot be converted to List<String>",
and "List<Circle> cannot be converted to List<Shape>" for the narrower element,
which is the invariance rule a `? extends T` parameter exists to relax — and
`addAny(Collection<? extends T>)` takes the circles it refuses.

Two shapes carried no marker at all: a USER generic as the parameter
(`void addFrom(Bag<T> other)`) arrives as the RAW class, and an array of the
variable (`void addArray(T[] more)`) as `Object[]`. Neither erasure has anything
to substitute — and neither had to, because the parser kept the parameter list
AS WRITTEN all along, for every method of a generic class. The table records it
now, resolved with the class's variables renamed to the positional sentinels,
and the check reads that where it says more than the erasure does. `T[]` is the
one place the two must differ: it really IS an `Object[]` at run time, which is
right for the descriptor and useless for the check, so the written form keeps
the variable and the descriptor keeps the erasure.

Pinned as `a_type_variable_inside_a_parameter` — the shapes that must still
compile, a raw receiver, a two-variable class and a `? extends` parameter among
them — with nine refusals pinned by their wording.

## A generic method's variable answers every argument

The last of that matrix: `static <T> void give(Bag<T> bag, T value)` called as
`give(bagOfStrings, 1)`. Erasure reads both parameters as `Object`, so every
call fit. The plan the compiler keeps for a generic method reads its RETURN and
a `void` one has none — but the parser records a plan for EVERY variable the
parameters mention, beside the one for the return, and nothing had ever asked
for it.

Which parameter pins a variable decides what the others may be. A CONTAINER
parameter is invariant, so `Bag<T>` given a `Bag<String>` pins `T` to `String`
exactly, and the `T value` beside it has to fit — while a variable named only by
DIRECT parameters (`<T> T pick(T a, T b)`) is joined at the least upper bound,
which is why `pick("a", 1)` compiles and must keep compiling. Two containers
that disagree are the same contradiction from both sides.

A container that is itself a POLY expression pins nothing: its element is
whatever the call needs, so `firstOf(new ArrayList<>(List.of("a")), 1)` is
javac's answer too. It still has to REACH a pin another argument made — an
inline `List.of(1)` for a `Bag<String>` is refused — and an empty diamond,
having no element at all, reaches anything.

The diagnostic is javac's, through the line that names the mistake:

    method give in class R4 cannot be applied to given types;
      required: Bag<T>,T
      found: Bag<String>,int
      reason: inference variable T has incompatible bounds

which needed the parameter list as WRITTEN — the erased one says `Bag,Object`
and explains nothing — so the table keeps that too, for the message alone.

Pinned as `a_generic_methods_variable_answers_every_argument`, with five
refusals beside it.

## The name of a day, and the date a locale asks for

`month.getDisplayName(TextStyle.FULL, Locale.US)` and
`DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM)` were both "cannot find
symbol" — the two ways a program asks for a date in words rather than in
pattern letters. Everything they need was already here: the month and day text
the formatter prints, and a pattern engine exact enough that the four localized
date formats ARE patterns.

`ofLocalizedDate(FULL)` is `EEEE, MMMM d, y`; LONG is `MMMM d, y`, MEDIUM
`MMM d, y`, SHORT `M/d/yy`; a localized time is `h:mm:ss a` or `h:mm a`, and
FULL and LONG name a ZONE, which no `LocalTime` has — so a JDK fails there
rather than printing, and formatting through the equivalent pattern fails in
the same words. Every one of these was checked against a JDK before it was
written down, including the two shapes that ask a value for a field it has not
got (a date-time format over a `LocalDate` is "Unsupported field:
ClockHourOfAmPm", a date format over a `LocalTime` "Unsupported field:
MonthOfYear"). Two things a pattern cannot say are stored beside it: the
formatter's own `toString` is `Localized(FULL,)` rather than the pattern, and a
localized TIME reports its missing zone with the chronology beside it.

Neither style is a VALUE here. `TextStyle` and `FormatStyle` are read where
they are written — the call site is the only place either has ever appeared —
so the compiler resolves the constant while compiling and hands the VM an int,
the way a `printStackTrace(System.out)` stream is already handed one. A style
held in a variable is refused by name, which is what `Locale` beside them
already did.

And the locale is CHECKED. caturra ships one text, en-US, so
`Locale.FRANCE` is refused where it is written rather than answered in English:
a wrong answer is worse than a missing one, and every English spelling
(`US`, `ENGLISH`, `UK`, `CANADA`, `ROOT`, `getDefault()`) gives the text a JDK
gives.

Pinned as `the_name_of_a_day_and_a_month` — every month and day in every style,
the four localized formats over a date, a time and both, the two missing-field
failures, the four `toString`s and a parse back through one — with the two new
strictnesses beside it in the divergence list. The `java.time` fuzzer's
positions program asks for all twenty-four of them each run.

## What a longer fuzz run found

The fuzzers are usually run for a handful of seeds a session, which is enough
to catch a regression and not enough to search. Run properly — nine hundred
random programs, sixty collection sequences, forty regex seeds, thirty each of
format and time, and seven hundred one-token source mutations — they found
three things, in three different parts of the engine.

**A precision truncates `%h`'s null.** `String.format("%.1h", null)` is "n": a
null hashes to the WORD "null", and every conversion's text is cut to its
precision. `%h`'s null arm padded and never truncated, so it printed the whole
word — the one cell of the format cross-product that had never been asked with
both a null and a precision.

**A type ARGUMENT nothing declares was accepted.** `Box<R>` for an `R` no scope
declares read as the RAW `Box` and compiled; so did `Comparator<xyzzy>`, whose
arguments the functional erasure drops on purpose, and so did `Box<int>`, where
a library generic had said "unexpected type" all along. Three arms silently
answered the erasure where the argument was a mistake rather than something the
erasure declines to carry — and a fourth position, a method SIGNATURE, never
looked at its arguments at all, because the descriptor erases them away. The
mutation that found it renamed a method's own `<R>`, which is exactly how a
student meets it: a typo'd type variable, silently compiled.

**An inner class's enclosing instance was read as its own type argument.**
`outer.new Box<>("hi")` for a non-static `Box<T>` inferred `Box<Outer>`: the
enclosing instance is prepended as the leading constructor argument, and the
constructor's inference plan counts only the parameters the program WROTE. The
first thing done with the value — "Box<Outer> cannot be converted to
Box<String>" — named a type the program never wrote. Writing the argument out
(`new Box<String>(…)`) worked, which is why nothing had noticed.

Pinned as `what_a_longer_fuzz_run_found`, with five refusals beside it. The
lesson is the running, not the finding: three real defects sat behind seeds
nobody had drawn.

## An integer of any size

`java.math.BigInteger` — the first thing past `long`, and the first addition
under the widened scope (see `specs/SCOPE.md`). A CSA program reaches for it the
day `long` overflows: factorials, the running total of a big simulation, the
"why did my number go negative?" lesson.

The arithmetic is caturra's own, in `crates/caturra-vm/src/bigint.rs` —
sign-and-magnitude over base-2³² limbs, no dependency. That follows the repo's
existing practice (the regex engine, the float formatter and the Unicode tables
are all written here) and its two constraints: the release profile optimizes for
WASM SIZE, and every answer has to be a JDK's exactly anyway, which a general
library would not give for free. `mod` is not `remainder`, the bit operations
run over an infinitely sign-extended two's complement the magnitude does not
hold, and six exceptions have to say the words a JDK says.

**What was measured.** A twelve-value grid (zero, ±1, ±7, two twenty-digit
values, 10³⁰, 255, 2³², 2⁶⁴) crossed with eleven binary operations and twenty-two
unary ones — 2029 lines of a real JDK's answers, including 91 exceptions. The
core agreed on 2013 of them on its first run; the sixteen that differed were the
harness's own wording and its float formatting. Then 87,378 randomized checks
against Python's own bignums, extended to 118,186 once the primality and bit
setters were in.

**The one real defect the fuzz found.** `doubleValue` was folding limb by limb —
`value * 2³² + limb` — which rounds at every step. Past three limbs the
accumulated error shows up as a one-ulp answer, and 40 of 4000 random values
were wrong. A JDK rounds ONCE, half to even, from the top 54 bits with
everything below folded into a sticky bit; `floatValue` does the same from the
top 25. Both are exact now.

**What the wiring found.** Four things, each the same shape: a fact that lived
in a hand-written list rather than in the table that already knew it.

- The lambda pass judged "is this method static?" by NAME against a list that
  includes `signum`, `abs`, `max`, `min`, `pow` and `sqrt` — true of
  `Integer.signum`, false of `BigInteger::signum`, which desugared to a static
  call that does not exist. It asks the real method tables now, for every
  library value type.
- `descriptor_type` in the lambda pass admitted only `java.time` names, so a
  method reference could not answer a `BigInteger`. The list it consults is now
  "library value types", the same one `resolve_type` reads.
- A `BigInteger` is a `Number` and a `Comparable` — the only library value that
  wears both wrapper faces — in the widening rule, in the bound a `<T extends
  Comparable<T>>` parameter carries, and at runtime where a sort asks.
- `%d`, `%x`, `%X` and `%o` take one. It prints in SIGN-MAGNITUDE there, unlike
  an `int`'s two's complement: `%x` of -255 is `-ff`, and the signed flags
  (`+`, `(`, a space) that a `long`'s hex refuses are legal here.

Pinned as four programs: `big_integer_arithmetic_is_exact`,
`big_integer_bits_and_conversions`, `big_integer_text_and_formatting`, and
`a_big_integer_in_every_position` — the last of which puts one in a field, a
parameter, an array, a `List`, a `TreeMap` key, a `TreeSet`, a cast, an
`instanceof`, a bounded type variable, a stream, four method references, a
`Comparator.comparing`, a `StringBuilder`, and a `Number` and `Comparable`
variable. The bignum core carries seven unit tests of its own for the parts a
program cannot reach directly.

## A number with a scale

`java.math.BigDecimal`, and the two types it cannot be used without:
`RoundingMode` and `MathContext`. This is the answer to the first surprise
every programming course delivers — `0.1 + 0.2` is `0.30000000000000004` — and
the type every money exercise should be written in.

The core is `crates/caturra-vm/src/decimal.rs`, over the `BigInteger` written
beside it: an unscaled integer and a `scale`, so the value is
`unscaled x 10^-scale`. That is a JDK's model exactly, and it has to be,
because **the scale is observable**. `2.0` and `2.00` compare equal and are not
`equals`; they hash differently, so a `HashSet` holds both; and every operation
has a rule for the scale it answers with — the scales ADD on a multiply, take
the larger on a sum, and an exact division answers at the "preferred" scale
(`dividend.scale - divisor.scale`) when the exact quotient fits there.

**What was measured.** Three JDK captures, run before a line was written:
a twelve-value grid crossed with eleven binary operations, twenty-two unary
ones, seven rounding modes at three scales and four powers (2234 lines); a
seventy-two-shape format matrix over a positive, a negative and a value no
`double` can hold; and a ten-by-ten-by-five `MathContext` cross-product (614
lines). Then 90,000 randomized checks against Python's own `decimal`, which
follows the same IBM specification a JDK's `toString` does.

**What that found.** Four things, none of them guessable:

- **A JDK has no negative zero, and Python does.** `new BigDecimal("-0.000003")
  .setScale(5, DOWN)` is `0.00000`, not `-0.00000` — the oracle was wrong, not
  the core, and the JDK capture is what said so.
- **`toEngineeringString` does not write an exponent of zero.** `3E+1` prints
  as `30`. And a ZERO is a rule of its own: `0E+1` is `0.00E+3`, because the
  coefficient grows a fraction where a non-zero would move digits left.
- **A zero divisor says two different things.** `divide(x)` says `Division by
  zero`; `divide(x, scale, mode)` says `/ by zero` — and `BigInteger divide by
  zero` once the dividend is too big for the `long` path a JDK takes first.
  Three messages for one mistake, and a student reads them.
- **`divide(divisor, mc)` is not "round the quotient".** An EXACT quotient that
  fits the precision is answered at the preferred scale; the context only bites
  when it does not fit. `10 / 1` to one significant digit is `1E+1`, to two it
  is `10`.

`RoundingMode` is a real enum here, not an int read at compile time: it
switches, fills an `EnumSet`, keys an `EnumMap`, streams, sorts, and `==`
works on it — which meant giving the interning pool the `java.time` enums use a
second occupant, and renaming it from `temporal_pool` to what it now holds.
`MathContext` is an ordinary two-field value. The deprecated `BigDecimal.ROUND_*`
ints are the same ordinals, and are accepted wherever a `RoundingMode` is.

Reading a decimal as a `double` goes through its own text — Rust's parser
rounds once, as a JDK does — and `new BigDecimal(0.1)` takes the EXACT binary
value (`0.1000000000000000055511151231257827021181583404541015625`), which is
the lesson `BigDecimal.valueOf(0.1)` exists to teach. `%f`, `%e` and `%g` take
one too, with its OWN digits rather than a `double`'s: a value past a double's
range prints every digit it has.

Pinned as `big_decimal_keeps_its_scale`, `big_decimal_formats_exactly` and
`a_big_decimal_in_every_position` — the last of which puts one in a field, an
array, a `List`, a `TreeMap` key, a `TreeSet`, a cast, a bounded type variable,
four method references, a `Comparator.comparing(...).thenComparing(...)`, a
`Number` and a `Comparable` variable, and runs the `RoundingMode` enum through
a switch, an `EnumSet`, an `EnumMap`, a stream and a sort. Seven unit tests sit
in the core beside them.

## A number a person reads

`java.text.DecimalFormat` and `java.text.NumberFormat` — the third and last
piece of the numbers lane, and the one a program reaches for the moment a
`double` has to appear on a screen: `new DecimalFormat("#,##0.00").format(total)`.

The pattern is a small language, and what it means is a handful of counts:
how few integer digits to insist on, how many fraction digits to allow, where
the groups fall, what multiplies the value, and what sits either side of it.
The core is `crates/caturra-vm/src/numfmt.rs`, formatting through the exact
decimal arithmetic of `decimal.rs` rather than a second pass through a
`double`.

**What was measured.** The seventeen-pattern by thirteen-value grid a program
would actually write (289 lines of a JDK), then three randomized sweeps against
a real JDK: 700 patterns generated from the grammar, 900 wilder ones (nested
quoting, negative subpatterns, per-mille, currency), and 10,272 chosen to sit
ON a tie, which is the only place two engines can disagree. 12,161 cases, and
they agree exactly.

**Four rules that are a JDK's and not the javadoc's.** Each cost a fuzz round
to find:

- **In scientific notation the significant-digit count is `maximumInteger +
  maximumFraction`,** and the fraction limit does not cap what is printed:
  `##.00#E0` of -707326118 is `-7.0733E8`, five digits where the pattern names
  three fraction places. The total printed is at least
  `minimumInteger + minimumFraction`, at least the value's own digit count,
  and at least the integer places the exponent left in front.
- **A zero has no exponent at all,** and grows a fraction where a non-zero
  would move digits left: `###.#E00` of 0 is `0E00`, and `###.00#E0` is `0.0E0`.
- **`toPattern` writes one position above the minimum,** so `0` comes back as
  `#0` — and an affix is quoted as a WHOLE when it holds a literal special
  (`(#` becomes `'(#'`), never per character.
- **A tie is not simply HALF_EVEN.** Which way it goes depends on three things
  a JDK reads off `Double.toString`: whether the true value sat above or below
  the shortest decimal (`0.05` at one place is `0.1`, `1.005` at two is
  `1.00`), whether the value is WHOLE (the `.0` that `Double.toString` writes
  puts the rounding position one short of the last digit, so `12335.0` to four
  significant digits is `1.234E4` where an exact `12335` gives the same and
  `12345.0` gives `1.235E4` where the exact one gives `1.234E4`), and — for a
  tie that underflows to zero — whether the text was scientific: `0.005` at two
  places is `0.01`, `0.0005` at three is `0.000`.

A percent pattern multiplies **the double**, in double arithmetic, before any
of that: `0%` of 2.675 is `268%`, because `2.675 * 100` is exactly `267.5` and
the tie then rounds to the odd digit's even neighbour. Multiplying the decimal
instead gave `267%`.

The locale here has no COUNTRY — a sandbox has no way to know one — so its
currency sign is the generic `¤` and its ISO code `XXX`, which is exactly what
a JDK with `LANG=en` answers. `getCurrencyInstance(Locale.US)` is not modelled
rather than guessed at.

Pinned as `every_decimal_format_pattern`, `a_number_format_in_every_position`
and `a_decimal_format_breaks_a_tie_as_a_jdk_does` — the last of which is the
twelve-pattern by thirty-five-value tie grid, run once over doubles and once
over the same values as exact decimals, which round the plain half-even way.
Seven unit tests sit in the core.

**One deliberate divergence.** `new DecimalFormat("").toPattern()` asks a JDK
for an array of 2^31 digits and dies with an `OutOfMemoryError`; caturra
answers `#,##0.###`, the pattern the empty one leaves in place. Reproducing a
JDK's out-of-memory bug is not worth the fidelity.

## Text that looks like a file, and a word at a time

Four small utilities a program reaches for without thinking, and which had no
answer at all: `java.util.StringTokenizer`, `java.util.UUID`,
`java.io.StringReader` and `java.io.StringWriter`.

**`StringTokenizer`** is the pre-`split` way to walk words, and the one a
textbook still teaches first. It is four questions over a cursor. Two details
are its own: a JDK's default delimiters are exactly space, tab, newline,
carriage return and form feed — not every whitespace character — and with the
delimiters KEPT (`new StringTokenizer(text, ",", true)`) a single delimiter IS
a token.

**`UUID`** is two longs and a canonical spelling. `fromString` is lenient about
each group's WIDTH and strict about the count; `variant()` is the JDK's own
bit expression, in which the top bits of the low half are masked by the
sign-extended top bit — reading it as unsigned answers 0 where a JDK answers 2.
`randomUUID()` cannot match a JDK's VALUE (a JDK draws from a secure source),
but its shape is fixed and pinned: version 4, variant 2, and the canonical
36 characters.

**`StringReader` costs almost nothing**, because caturra's `Reader` already IS
one: a buffer and a cursor. **`StringWriter`** needed a new sink, so
`PrintSink` gained a `Text` arm and a `PrintWriter` its target — which is what
makes `new PrintWriter(new StringWriter())` collect `printf` output a program
can read back, the shape every "test what this prints" exercise takes.

Three gaps turned up beside them and are closed too: `Reader.read(char[])` and
its range form, `Reader.skip`, and `Reader.lines()`. And **reading a CLOSED
reader is now a JDK's `IOException: Stream closed`** rather than an end of
stream — which matters, because a program that closes early and then reads was
getting a silently empty result instead of a failure.

Pinned as `a_tokenizer_and_a_uuid`, `text_that_looks_like_a_file` and
`the_small_utilities_in_every_position`.

## Bytes as text, and a set of small numbers

`java.util.Base64`, `java.util.BitSet`, and the marker interface an algorithm
asks about before it walks a list: `java.util.RandomAccess`.

**Base64** is a namespace for two coders. Three alphabets (basic, URL-safe,
MIME), padding that can be turned off, and a MIME encoder that breaks the line
every 76 characters. A decoder takes the text either way round — a `String` or
the bytes of one — and an unrecognised character is a JDK's
`Illegal base64 character 21`, naming the character's CODE in hex. A MIME
decoder skips what it does not recognise instead.

`Base64.Encoder` and `Base64.Decoder` are NESTED types, so a program that keeps
one in a field has to be able to name them — which meant registering them
beside `Map.Entry` in the nested-library table, or `Base64.Encoder e;` was
"package Base64 does not exist", about a package that is a class.

**BitSet** is index arithmetic over 64-bit words. Two details are its own:
`length()` is one past the highest set bit while `size()` is the STORAGE (a
multiple of 64, never less than one word, so two equal sets can report the same
size while holding different words); and `set(bit, value)` and `set(from, to)`
have the same ARITY, so the compiler renames the first before the VM sees it.

**RandomAccess** is a marker with no methods, synthesized beside `Cloneable`.
An `ArrayList` and a `Stack` wear it and a `LinkedList` does not — which is the
whole point of it, and what `list instanceof RandomAccess` asks. Like
`Comparable`, a synthesized marker reaches the VM under its BARE name, so the
runtime face list carries both spellings.

Pinned as `base64_and_bitset` and `base64_and_bitset_in_every_position` — the
second of which writes the sieve of Eratosthenes over a `BitSet`, which is what
a course actually uses one for.

## A year, a month, and a day that has no year

`java.time.Year`, `java.time.YearMonth` and `java.time.MonthDay` — the three
PARTIAL dates, and the last item of the small-utilities lane. A year on its own
is what a program keeps when the month and day would be a lie; a year-month is
the unit a statement covers; a month-day is a birthday.

They are three more `Temporal` variants, and — unlike the `java.time` enums
beside them — they are VALUES: a JDK does not intern them, so `==` on two is
false and they allocate.

Three things had to be decided rather than assumed:

- **What each one supports.** A `Year` has `YEAR`, `YEAR_OF_ERA` and `ERA` and
  nothing else; a `YearMonth` adds the month; a `MonthDay` has only the month
  and the day. `Year.of(2024).get(ChronoField.MONTH_OF_YEAR)` is a refusal, not
  January.
- **How they format.** Each renders through a date filled out with defaults,
  and a pattern that reaches past the fields it really carries is the JDK's
  `UnsupportedTemporalTypeException: Unsupported field: MonthOfYear` — not a
  silently invented 1st of January. The guard walks the pattern's pieces
  (including the optional and padded ones) before anything is printed.
- **What a 29th of February does in a common year.** `MonthDay.of(2, 29)
  .atYear(2023)` is the 28th, which is what a JDK does rather than refusing;
  but `MonthDay.of(2, 30)` is refused outright, because no year has one.

Pinned as `a_year_a_month_and_a_day` and
`the_partial_dates_in_every_position` — the second putting all three in a
field, an array that sorts, a `List` that sorts, a `TreeMap` key, a `HashSet`,
a bounded type variable, a stream, a `StringBuilder` and a `Comparable`
variable, and running each one's text, fields and refusals.

## The collections that came first

`java.util.Vector`, `java.util.Hashtable` and `java.util.Enumeration` — the
legacy tour, and the last of the four lanes this session widened into. They are
still on every AP reading list, and half the Java written before 1998 is built
on them: a student who meets one in real code should be able to run it here.

Underneath, each shares its storage with the modern collection it was replaced
by: a `Vector` IS the element vector a `Stack` already used (`HeapObject::Stack`
reporting `java.util.Vector` through the view-class map), and a `Hashtable` is
the same `JavaHashMap` a `HashMap` uses with one extra flag. What makes them
worth having is precisely what is NOT shared, and every one of the following was
measured against a real JDK before a line was written:

- **A capacity a program can see.** `new Vector<>()` reports 10, `new Vector<>(2)`
  grows 2 → 4 → 8, and `new Vector<>(2, 5)` grows 2 → 7 → 12: a positive
  `capacityIncrement` STEPS where the default doubles. `trimToSize`,
  `ensureCapacity` and `setSize` move the figure, so it is stored rather than
  derived from the size alone. An `ArrayList` has a capacity too and no way to
  ask about it; a `Vector` does, and that is the difference a lesson uses.
- **The pre-`List` names.** `addElement`, `elementAt`, `insertElementAt`,
  `setElementAt`, `removeElement`, `removeElementAt`, `removeAllElements` — the
  same operations, and two of them take their arguments the other way round.
  They are renamed to their `List` spellings in ONE place
  (`legacy_vector_call`), because both dispatch layers need the mapping:
  `removeElement(o)` becomes `remove(Object)`, which compares elements and so is
  answered by the interpreter, while the rest are answered by the list
  intrinsics. Read in only one of the two, `removeElement` reached a list that
  had never heard of it.
- **A complaint per method.** A `Vector` indexes a bare array, so the class is
  `ArrayIndexOutOfBoundsException` throughout — but `elementAt(5)` on a vector of
  two says `5 >= 2`, `get(5)` says `Array index out of range: 5`,
  `insertElementAt(x, 9)` says `9 > 2`, and a NEGATIVE index reaches the array
  and reports the CAPACITY (`Index -1 out of bounds for length 4`) — after the
  insertion has already grown it. `new Vector<>(-1)` is
  `Illegal Capacity: -1`, where a `HashMap` says "Illegal initial capacity".
- **A `Hashtable` takes no null**, in either position, and `get`/`containsKey`/
  `remove`/`contains` refuse one too — it predates the null-tolerant `Map`
  contract, hashes its key without checking, and tests its value outright.
  `entry.setValue(null)` through its entry set is the one way back in that does
  not pass through `put`, and it refuses as well.
- **TWO traversal orders.** `keys()`, `elements()`, `toString`, the views and
  the streams walk the bucket array DOWNWARD; `forEach` and `replaceAll` walk it
  UPWARD. Within a bucket the chain reads head-first either way, so one is not
  the reverse of the other — two entries that share a bucket keep their order
  while the buckets swap ends. `action_order()` is that second order.
- **An `Enumeration` is not fail-fast.** `Vector.elements()` and
  `Hashtable.keys()` predate `modCount`: a change made mid-walk is simply seen,
  and running off the end names the collection —
  `NoSuchElementException: Vector Enumeration`,
  `NoSuchElementException: Hashtable Enumerator`. The very same walk through
  `Collections.enumeration(v)` DOES throw a `ConcurrentModificationException`,
  because that one wraps an iterator. Two enumerations over one vector, and only
  one of them checks.
- **`Collections.enumeration` and `Collections.list`** are the bridges between
  the two eras, and the only `Collections` pair whose argument and answer are
  different families.

A `Stack` IS a `Vector` (that is where its `List` face comes from), so
`st instanceof Vector` is true and `st.elementAt(0)` works — and the `Vector`
face HIDES the five LIFO methods, which is why `push`/`pop`/`peek`/`empty`/
`search` are marked at `TableFace::Concrete` while `clone` is marked at
`TableFace::Vector`.

Widening the three cost the usual toll of lists that had drifted: the
`Collections` algorithms' element gates (now one `list_like_elem`), the
`instanceof` target table, `is_reference`, `mark_raw`, the syntax-side
descriptor builder, the for-each accessor table, the cast-target families, the
copy-constructor sources, the runtime face table, and eight separate name lists
in the lambda pass. One of those turned up a defect with nothing to do with the
legacy collections at all: `iterated_get` had no arm for an unmodifiable
wrapper where `iterated_len` did, so a cursor built straight over a `List.of`
view knew its LENGTH and handed back a null for every element of it.

A fresh seed of the FORMAT fuzzer, run beside this unit's own, turned up one
more thing with nothing to do with the collections: the `#` flag forces a
decimal point at precision 0, and on a SCIENTIFIC result it lands BEFORE the
exponent — `%#.0e` of 1e-10 is `1.e-10` — where on a fixed one it lands at the
end. `%#.0f` had the rule and `%#.0e` did not (pinned as
`the_alternate_flag_at_precision_zero`).

Pinned as `the_legacy_collections`, `what_a_vector_says_when_it_refuses`,
`the_legacy_collections_in_every_position`, `which_way_a_hashtable_walks`,
`the_legacy_collections_at_their_edges` and `an_enumeration_is_not_fail_fast`.

**Deliberately not built: `Date`, `Calendar`, `SimpleDateFormat`.** They were
measured too, and the measurement is why they are out: `new Date().toString()`
prints a timezone abbreviation with daylight saving applied (`PST`/`PDT` on the
machine this was measured on), and `Calendar` and `SimpleDateFormat` are shaped
around the same default zone. Answering those honestly needs a real timezone
database, which caturra deliberately does not vendor (see "java.time" above).
They stay named refusals rather than approximations that would be wrong by an
hour twice a year — and named is the point: `Date`, `Calendar`,
`GregorianCalendar`, `TimeZone`, `SimpleTimeZone`, `SimpleDateFormat`,
`DateFormat` and `DateFormatSymbols` each say "java.util.Date is not supported
by caturra" wherever they are written, rather than "cannot find symbol", which
reads as a typo for a class the student can see in the documentation. Pinned as
`strict_the_old_date_classes_are_refused_by_name` and
`strict_simple_date_format_is_refused_by_name`.

## Every class this engine does not have

The last of the four lanes, and the one that is about the MESSAGE rather than
the code: a real Java class caturra does not model should say so by name.
"cannot find symbol" is a statement about the PROGRAM — the student mistyped
something — and about a real class it is simply false. The student spelled
`Properties` correctly; this engine is the thing that is missing, and only one
of the two of us knows that.

The sweep was measured, not guessed. A Java 11 module image was walked for
every class in the thirteen packages caturra models — `java.lang`, `java.util`,
`java.io`, `java.math`, `java.text`, `java.time` (+ `format`/`temporal`),
`java.util.regex`, `java.util.function`, `java.util.stream`,
`java.nio.charset`, `java.lang.reflect` — 571 names, of which javac itself
accepts 455 as a variable type. Each was compiled here and its answer bucketed.
Before: 208 supported, 46 honest, **201 "cannot find symbol"**. After: 212
supported, 227 honest refusals, 16 that name what is really missing, and
**zero** that read as a typo.

Then the same sweep one level up, over every `java.*`/`javax.*` package the
module image holds (166 of them): 138 said "package java.security does not
exist", which is a false statement about the JDK. `KNOWN_UNSUPPORTED_PACKAGES`
held four names — whichever ones somebody had happened to hit — and is the
whole recorded list now. A package that really does NOT exist (`java.foo`)
still gets javac's own wording, which is what makes the honest one honest.

Three things the sweep found that were not the thing it was looking for:

- **A class in both tables.** `java.io.Reader` and `java.io.FileWriter` were
  listed as unsupported AND modelled: each worked written under a wildcard
  import and was refused by its own single import — one fact answered two ways,
  the defect class this repo keeps meeting. Four tests in `imports.rs` now
  assert the tables cannot say both things at once.
- **A modelled class that could not be imported by name.** The package tables
  say what a package OFFERS, and twenty-two names were missing from theirs —
  so `import java.lang.InterruptedException;` was "cannot find symbol: class
  InterruptedException in package java.lang", about a class that works on the
  very next line. The format-exception family (`MissingFormatWidthException`
  and five siblings) was the same: thrown by name from `String.format`, and not
  importable to catch.
- **A refusal that was false.** `Math m;` said "java.lang.Math is not supported
  by caturra" — about the class every program has used, in the one place a
  student reads a message as authoritative. What is missing is a VALUE of the
  type, which is all a variable could hold, and the message says that now:
  "java.lang.Math cannot name a variable in caturra: its members work (written
  out, as Math.…), but no value of the type is modelled". Sixteen classes reach
  it — `Math`, `System`, `Arrays`, `Collections`, `Objects`, `Collectors`,
  `Locale`, `Base64`, `TextStyle`, `FormatStyle` and the rest of the
  namespaces.

Raw `LinkedHashMap`, `LinkedHashSet`, `EnumMap` and `EnumSet` were refused
outright as well — the raw-arity table had every other collection and not these
four, so `LinkedHashMap m = new LinkedHashMap();` (ordinary pre-generics Java)
did not compile.

Pinned as `real_java_classes_caturra_lacks_name_themselves`,
`real_jdk_packages_caturra_lacks_name_themselves` and
`a_modelled_class_can_be_imported_by_name`, plus the four strictness entries
listed above. The consistency of the tables themselves is guarded by
`no_class_is_both_offered_and_refused`,
`no_package_is_both_modelled_and_refused`,
`every_refused_class_sits_in_a_modelled_package` and
`every_refused_package_is_a_real_jdk_one` — because a hand-written list that
drifts is the thing this whole lane was cleaning up after.

## Four refusals of ordinary Java

The honest-refusal sweep found more than it went looking for: four bits of
perfectly ordinary Java that caturra turned away. A refusal of code javac
accepts is the same defect as a misleading message — worse, in fact, because
the student cannot work around it.

**`java.io.Writer` is a type a program names.** A `PrintWriter` and a
`StringWriter` are distinct types here (they answer different methods), and
`Writer` is the abstract class both wear — what a variable holding either is
declared as, what a parameter takes, what a `List<Writer>` holds. It was
"unknown type 'Writer'". It is a FACE now, offering exactly what
`java.io.Writer` declares: `write`, `append`, `flush`, `close`. Not a
`PrintWriter`'s `println`, and not a `StringWriter`'s `getBuffer` — making the
face wide enough to hold either must not make it wide enough to call both, and
both refusals are pinned.

**`TextStyle` and `FormatStyle` are enums.** They were modelled as constants
the compiler read where they were WRITTEN: `Month.MAY.getDisplayName(
TextStyle.SHORT, Locale.US)` worked and there was no value of the type to
hold, so a variable, a `values()`, a `valueOf`, a switch, a sort or a
`TreeMap` key was refused. They are real library enums now, interned like
`Month` and `DayOfWeek` beside them, and `getDisplayName` takes either form —
a written constant is still folded to its ordinal while compiling, and a value
is emitted and asked for its own. `TextStyle.isStandalone`/`asStandalone`/
`asNormal` came with them: a standalone style is the odd ordinal of each pair.

**The `Collectors` overloads that name a map.** `toMap(k, v, merge)` was
refused with a method reference (`Integer::sum`) and mistyped with a lambda —
the merge folds two VALUES, and both parameters were erased to `Object`, so
`(p, q) -> p + q` was "bad operand types". The value type is what the second
argument answers, read once that argument has been erased into its synthesized
class. `toMap(k, v, merge, TreeMap::new)` and `groupingBy(f, TreeMap::new,
downstream)` were not modelled at all; both gather into the map the program
asked for, which is how it gets its result in key order rather than a
`HashMap`'s.

**`Collector<T, A, R>` as a written type.** Factoring a collector out into a
variable is ordinary Java, and the accumulator nobody ever writes out — the
`?` in the middle — read as an unknown class: "cannot find symbol: class
Wildcard ?". The type carries what it GATHERS (`R`), because that is the only
place the answer survives once the factory call is behind a name; a
parameterized one rides as a nested type, so `collect(c).get(0)` finds `get`
on a `List<String>` rather than on an `Object`. Its own lambdas are typed from
the element the declared type names (`T`), which the enclosing `collect` used
to be the only source of.

Two things fell out on the way, neither about the four:

- **`new List<>()` said "cannot find symbol: class List".** Every library type
  caturra models as an abstract face — `List`, `Map`, `Reader`, `Writer`,
  `Stream`, `Iterator` — was reported as a class that does not exist, when the
  reason `new List<>()` will not do is that no such object can be made. It is
  javac's "List is abstract; cannot be instantiated" now.
- **`new TextStyle[2]` crashed the compiler.** The one-dimensional array
  emitter listed its reference element kinds by hand and fell through to the
  primitive match's `unreachable!` for anything new. There is a catch-all now:
  any reference element is the same instruction over the class its own
  descriptor names. A crash prints no diagnostic at all, which reads as a clean
  compile — see **Engine must not crash**.

`Collector<T, A, R>` and the `Writer` face are the seventh and eighth types to
need an arm in BOTH `widens` and the assignment matrix. The trap is in the
model, not in anyone's attention.

Pinned as `a_writer_is_a_type_a_program_names`,
`the_formatting_styles_are_enums`, `a_collector_of_ones_own`,
`the_four_refusals_in_every_position`, and four reject pins for the negative
direction.

Left refused, and honestly: `BufferedWriter`, `InputStream` and `Serializable`
as declared types, and `OutputStream o = System.out` (caturra models
`ByteArrayOutputStream` and the abstract name as one type, so the face cannot
tell them apart). Each says so by name.

## Which reader a reader is

The four readers — `BufferedReader`, `FileReader`, `InputStreamReader`,
`StringReader` — were ONE type over one heap object. They share their storage
honestly (a buffer and a cursor, and the same reads), but they differ in two
ways a program can see, and caturra was wrong about both:

- **`getClass()` named the wrong class for three of them.**
  `new StringReader(t).getClass().getName()` answered `java.io.BufferedReader`.
  That is a confidently wrong answer about a class the program named itself,
  which is worse than a missing one.
- **`readLine` was offered on all four.** `new StringReader(t).readLine()`
  compiled and ran; javac says "cannot find symbol", because `readLine` and
  `lines` are a `BufferedReader`'s alone. A student who writes that here and
  hands it in gets a compile error from their teacher's JDK.

`JType::Reader` carries a `ReaderFace` now — the same shape `CollFace`,
`TableFace` and `SeqRole` already give the collections — and there are two
method tables: what `java.io.Reader` declares, and that plus the two a
`BufferedReader` adds. The runtime class rides in the view-class map the
`Vector`/`Hashtable` work put there, set by each constructor.

Measured against a JDK first, and three answers were not guessable:

- **A `FileReader`'s superclass is `java.io.InputStreamReader`**, not `Reader`
  — the one place that class shows up in an ordinary program.
- **`markSupported()` is true** for a `StringReader` as well as a
  `BufferedReader`, so `mark`/`reset` work on both. caturra had none of the
  three; they are a cursor save and restore over the buffer it already keeps.
- **`getInterfaces()` is empty for every one of them.** The interfaces belong
  to the abstract `Reader` (`Readable`, `Closeable`) and `Writer`
  (`Appendable`, `Closeable`, `Flushable`) above them.

`Reader.transferTo(Writer)` came with them — Java 10's, and the shortest way to
copy a whole reader into a writer.

**`java.io.BufferedWriter`, and why it is a heap kind.** It was a named refusal
until now, and it is what every file-writing exercise wraps a `FileWriter` in.
It really BUFFERS: what a program writes reaches the writer underneath only on
a `flush` or a `close`, and a JDK lets it see that — `sw.toString()` is empty in
between. Modelling it as a pass-through would answer the wrong thing for the
one program that checks, and "my file is empty" is exactly the lesson a
forgotten `close()` is supposed to teach. `close()` flushes; a second `close()`
is not an error; writing after one is `IOException: Stream closed`.

The reader faces are the NINTH type to need an arm in both `widens` and the
assignment matrix.

Pinned as `which_reader_a_reader_is` and
`readers_and_writers_in_every_position`, with four reject pins for the
direction that matters: `readLine` and `lines` on a plain reader, `newLine` on
a plain writer, and one reader assigned to another.

## The methods a measurement found

Four units in a row were chosen from whatever a probe happened to stumble into.
This one was chosen by `scripts/coverage/measure.py`, which had not run since
seven units before it: **286 → 364 nameable classes, 995/1213 → 1354/1552
method names (87.2%)**.

The first thing it found was a refusal of caturra's own making.
`Scanner.nextBigInteger()` said "BigInteger is not supported by caturra" — true
when the message was written, and false from the moment the bignum core landed
hours earlier in the same session. A refusal that outlives its reason is a
worse answer than no method at all, and it is exactly the shape of the stale
"Vector is a gap" card the legacy-collections unit caught. Both big-number
readers and their `hasNext` twins work now.

The rest of the low scores split cleanly. `java.lang.System` (36%) and
`DateTimeFormatter` (20%) are almost entirely host facilities and
locale/chronology machinery caturra deliberately does not have. What was left
was real:

- **The `Collections` wrappers.** `synchronizedList`/`Set`/`Map`/`Collection`
  and their four sorted spellings: on one thread a monitor is never contended,
  so each IS the collection it wraps — the same argument `synchronized` itself
  already uses. It gets an object of its own only so `getClass()` names the
  wrapper and wrapping does not change what the original answers; everything
  else delegates, and the delegation is TRANSITIVE because wrapping a wrapper
  is legal. A JDK picks `SynchronizedRandomAccessList` over `SynchronizedList`
  by whether what it wraps is `RandomAccess`, exactly as `unmodifiableList`
  does.
- **The four sorted `unmodifiable*` wrappers**, which caturra had for the plain
  set and map and not for the sorted ones.
- **The seven empty factories.** `emptySortedSet`/`SortedMap` and their
  navigable twins are built from the unmodifiable navigable wrapper (their
  class names say so), and `emptyIterator`/`emptyListIterator`/
  `emptyEnumeration` are cursors over nothing.
- **`Arrays.parallelSort`/`parallelSetAll`/`parallelPrefix`.** The first two
  ARE their serial twins here — on one thread the only difference is how the
  work is divided, and dividing it one way gives the same array — so they are
  written as delegations rather than second implementations, in the bundled
  `Arrays` where the serial ones live. `parallelPrefix` is a running fold in
  place, which has no serial name to borrow.
- **`Objects.checkFromToIndex`/`checkFromIndexSize`.** Their messages name the
  whole range, and `checkFromIndexSize` prints the sum UNEVALUATED —
  "Range [3, 3 + 2) out of bounds for length 4" — which only a capture tells
  you.

Two more measured answers that could not have been guessed:
`new BigDecimal` from a `Scanner` keeps its scale (`1.500` has scale 3), and
`nextBigDecimal` on a non-number is a BARE `InputMismatchException` where
`nextInt` names the string it could not parse.

`Collections.synchronizedSet(s)` handed to a `Collection<?>` parameter was
"cannot determine the type of an argument": the emit path knew the wrapper
answers its argument's own type and the `type_of` mirror did not — the
emit/typing split again, in a unit that added fourteen wrapper names at once.

Pinned as `the_methods_a_measurement_found` and
`the_measured_methods_in_every_position`, with two reject pins for the sorted
wrappers that must still refuse an unsorted argument.

## What a file says about itself

`java.io.File` was the next cluster the coverage measurement pointed at — 23 of
40 method names, the lowest of any class a student actually uses. What was
missing was everything a file knows about itself besides its bytes.

**The measurement corrected a guess mid-unit.** The first version of this code
modelled the three permission bits as state a program could set and read back,
and said in as many words that they were advisory — that nothing would enforce
them, because a JDK's enforcement is the host operating system's. The capture
said otherwise: opening a read-only file for writing is
`FileNotFoundException: name (Permission denied)`, and so is reading one that
is not readable. The comment was rewritten and the enforcement built. What is
NOT gated is metadata — `exists`, `length` and `delete` all work on a file a
program has locked itself out of, and `delete` answers to the DIRECTORY's
permission rather than the file's.

The check lives in the filesystem rather than at the five places a file is
opened, for the reason this repo keeps rediscovering: five copies of one rule
is five chances to forget it. So is the modified time — every write stamps the
entry from a clock the interpreter sets once on the way into any intrinsic
call, rather than each write path asking for one.

Three more measured answers:

- A newly written file is readable and writable and NOT executable; a
  directory is all three.
- The four setters answer `false` for a path that is not there, and
  `lastModified()` on one is 0 — the question is about a file, and there is no
  file.
- `setLastModified(-1)` is `IllegalArgumentException: Negative time`.
- `File.createTempFile` demands a prefix of at least three characters, and says
  so with the prefix quoted.

**`lastModified` is a HOST question**, like `System.currentTimeMillis()` and
`LocalDate.now()` before it. A host with no clock — the CLI, the tests —
answers 0, and the browser answers a real time. So "the file I just wrote has a
modified time later than zero" is true in the playground and false under
`compatrun`, and cannot be pinned any more than `LocalDate.now()` can. What IS
pinned is everything relative: a time a program sets is the time it reads back,
and a file that is not there has none.

**Refused, and by name:** `getFreeSpace`/`getTotalSpace`/`getUsableSpace` (an
in-memory filesystem has no device to measure, and a made-up total is worse
than a refusal because a program that checks before writing would trust it),
and `toURI`/`toURL` (the answer would be a `java.net.URI`, a type the program
could then do nothing with).

Pinned as `what_a_file_says_about_itself` and
`file_permissions_at_their_edges`, with the two strictness entries listed
above.

## The rest of what a Scanner is

`Scanner` is the most-used class in the curriculum and was the lowest-scoring
one a student actually writes: 29 of 39 method names. The ten missing were not
exotic — they are the delimiter and radix a program reads back and sets, what
the last read matched, and the `Iterator<String>` face `Scanner` has always
implemented.

Measured first, and three answers were not guessable:

- **The CONFIGURATION methods work on a CLOSED scanner.** `sc.close();
  sc.delimiter()` is legal, and so are `radix()`, `useRadix`, `useDelimiter`,
  `reset()` and `match()`. Only the ones that READ refuse, with
  `IllegalStateException: Scanner closed`. caturra had one check covering
  everything.
- **The default delimiter prints as `\p{javaWhitespace}+`**, not `\s+` — a
  program that prints `sc.delimiter()` sees the JDK's own pattern.
- **`match()` after `nextLine()` includes the terminator**: the group for
  `"one\ntwo"` is `"one\n"`, spanning 0 to 4, where the line the program got
  has no newline in it.

`reset()` is a trap worth being exact about: it puts the delimiter and the
radix back to a new scanner's, and does NOT rewind the input, whatever the name
suggests.

The radix reaches the INTEGER reads only — a JDK's `useRadix(16)` leaves
`nextDouble` decimal — and an explicit `nextInt(radix)` still wins over it.
What `match()` answers is recorded in the one place every successful read
passes through, so no path can move the cursor without saying what it
consumed; a `hasNext()` does not come that way, which is why asking after one
alone is still "No match result available".

**A `Scanner` IS an `Iterator<String>`.** It implements the interface — that is
what `hasNext`/`next` on one are — so it holds as one, casts to one, and
answers `instanceof Iterator`. It is not a cursor OVER a collection, though,
so it is named in the runtime face test rather than caught by the kind check,
and its `forEachRemaining` drives itself by reading. That is the tenth type to
need an arm in both `widens` and the assignment matrix.

**One thing deliberately not copied.** `match().groupCount()` after a
`nextInt()` is 35 in a JDK — the group count of the internal regex its integer
parser uses. Reproducing that would be copying an implementation detail no
engine should have; caturra's answer is its own pattern's, and the pin covers
the token case, where both say 0.

Refused by name, and listed in the strictness table above: `locale()` and
`useLocale()` (a `Locale` VALUE, which caturra models only as a written
constant), and — swept from the same measurement — the `TemporalAccessor`
plumbing (`query`, `adjustInto`, `addTo`, `subtractFrom`) and everything
carrying an instant, a zone or a calendar choice (`atZone`, `atOffset`,
`toInstant`, `toEpochSecond`, `ofInstant`, `ofEpochSecond`, `getChronology`) on
all six `java.time` values that declare them. Those had been reading as "cannot
find symbol" — the class-level honest-refusal sweep never reached METHODS, and
the coverage script is what makes that list exhaustive rather than a guess.

Pinned as `the_rest_of_what_a_scanner_is` and `a_scanner_in_every_position`.

## The long tail, and the reason for every name that is missing

The coverage script names its own next unit: run
`scripts/coverage/measure.py --verbose` and it prints, per class, the method
names a real JDK 11 offers that caturra does not. This is that list, worked
through. The seven things it found worth building were captured from a JDK
first, and every one of them was something a JDK does differently from the
obvious guess.

**`ArrayDeque.clone()` answers an `ArrayDeque<E>`.** Not the `Object` that
`ArrayList.clone()` answers — the deque overrides the return type and the list
does not, so the two need different rows and `d.clone().add(x)` compiles for
one and not the other.

**`getDeclaringClass()` on an enum constant is the enum.** It sits beside
`ordinal()` on all eight library enum tables. For a LIBRARY enum it is the same
question as `getClass()` (none of them has a constant body), which is why the
two share an arm in the VM — but only for a value, never for a `Class`, whose
`getDeclaringClass()` asks something else entirely and was answering
`java.lang.Class` for every class in the program.

**`Arrays.compareUnsigned` answers two different things.** For `int[]` and
`long[]` it answers a SIGN (`Integer.compareUnsigned` of the first differing
pair); for `byte[]` and `short[]` it answers their unsigned DIFFERENCE — `254`,
not `1`. Both spellings are in the same javadoc paragraph, and only a capture
tells them apart.

**`Random.longs` is the JDK's reject-until-unbiased loop.** A bounded stream
of longs is not `nextLong() % range`: the JDK draws and redraws until the value
is in an unbiased window, and caturra runs the same loop, so the streams are
byte-for-byte identical for a given seed.

**`ByteArrayOutputStream.writeBytes`/`writeTo`**, and a new parameter kind for
a stream passed to a stream.

**`java.lang.Class`, as far as a class file goes.** `getPackageName`, `cast`,
`asSubclass`, `getEnumConstants`, `isMemberClass`, `getDeclaringClass`,
`isLocalClass`, `isSynthetic`, `getFields`, `getDeclaredClasses` and
`desiredAssertionStatus`. Three notes:

* `cast` and `asSubclass` ask the question `instanceof` asks, so they read one
  shared rule — written once precisely so that two calls and an opcode cannot
  disagree about what a `ClassCastException` is.
* `getEnumConstants` does not build the list: it CALLS the enum's own
  `values()` — the intrinsic for a library enum, the synthesized static for a
  user one — so this answer and a program's own `Day.values()` cannot diverge.
  Reaching a user `values()` from inside an instruction meant running the
  class's `<clinit>` chain nested, since the dispatch loop's way of
  initializing a class is to PUSH frames.
* `isMemberClass`, `isLocalClass` and `getDeclaringClass` are read off the
  NAME, which is where caturra records nesting (`Outer$Inner`, `Name$Local1`,
  `Lambda$1`). The two predicates that recognise the synthesized shapes already
  existed for `getCanonicalName`; a third copy of the naming convention would
  have been a third chance to forget one.
* `desiredAssertionStatus()` is `false`, and that is not a guess: caturra
  compiles `assert` to `if (false)`, which is exactly a JDK run without `-ea`.

**Everything else answers with a reason.** The other 146 names a JDK offers on
these classes are refused ON PURPOSE, and each now says why — "`X.y` exists in
Java, but …" — rather than "cannot find symbol", which reads as a typo about a
method the documentation shows. `measure.py --why` prints the complaint for
every missing name and FAILS if any of them is still a bare "cannot find
symbol", so the property is checked rather than asserted. The families are
enumerated in the strictness table above.

Three things had to change for that to be true everywhere:

* Two names — `wait`/`notify`/`notifyAll` and `spliterator` — are on every
  receiver in Java, so they are answered by a rule rather than by a row per
  class. `spliterator` had seven rows and still said "cannot find symbol" on a
  `Stack`, on `Arrays` and on both stream kinds.
* The BUNDLED library classes (`Collections`, `Arrays`, `Object`, `Character`)
  are real compiled Java here, so their calls resolve through the ordinary
  user-class lookup and never reached the refusal table at all.
* A static call on a modelled class with no statics of its own reported the
  CLASS as unknown — "cannot find symbol: 'PrintStream'" about a type the
  program can name and hold two lines earlier.

The reasons themselves are now shared constants, because the same sentence had
been written out three times for a `Locale`, seven times for a `Temporal` and
five for a `TemporalQuery`.

Pinned as `the_long_tail_of_a_modelled_class` and the nine `strict_no_*` pins
above. The measurement read 1406/1552 method names answered across 56 classes
— down from the 1449 it reported before, because an honest refusal now counts
as a name that is MISSING rather than one that is known. It is the same engine;
the number is just no longer flattered by its own good manners.

**And then the same question about the measurement itself.** Those 56 classes
were the ones somebody had written a receiver expression for — less than a
quarter of the 232 core `java.*` classes caturra names. The other 176 were not
measured, which is a different thing from being complete. All of them are
measured now (**2833/3268 across 214 classes**), and the script refuses to
score a class whose receiver expression does not compile: that reads exactly
like a class scoring zero, and it is not the same fact. Eleven did — the
exception constructors that take something other than a message, and the three
summary-statistics classes with no visible constructor.

## What a throwable carries

Widening the measurement to every class caturra names (see the section above)
put 46 throwables under it for the first time, and they came back with two
things in common and eleven with a problem of their own.

**Every throwable in Java has `setStackTrace`, and none of caturra's did.** The
trace is not stored in the heap object here — it lives beside it, as the
rendered lines `getStackTrace()` reads back and `printStackTrace()` prints — so
this writes those lines, through the one renderer both of those use. Three JDK
facts a guess would have missed, all captured: it COPIES the array (a caller
that goes on mutating its own does not reach inside the throwable), it rejects
a null array with a bare `NullPointerException`, and it rejects a null ELEMENT
by its index — `stackTrace[0]`.

Beside it, `new StackTraceElement(class, method, file, line)`, without which
there is nothing to hand `setStackTrace`, and `isNativeMethod()`, which is not
a fifth piece of state: a JDK writes **-2** as the line number of a native
frame, and that is the whole test.

**Three throwables call their cause something else.** `ClassNotFoundException`
and `ExceptionInInitializerError` have `getException()`; `InvocationTargetException`
has `getTargetException()`. All three predate `getCause` and all three answer
exactly what it answers, so they share its arm rather than reading the field a
second time.

**Eleven could not be measured at all.** Their constructors take something
other than a message — `new ParseException("bad", 4)`,
`new IllegalFormatConversionException('d', String.class)` — so the probe's
receiver did not compile and every method on the class read as missing. That is
not the same fact as a class scoring zero, which is why the script now says so
rather than counting it.

The rule for where the extra value lives is worth writing down, because it is
not "wherever is convenient":

> **It lives in the MESSAGE when the JDK's message carries it.** A JDK words
> `new UnknownFormatConversionException("q")` as `Conversion = 'q'` and reads
> `getConversion()` back out of that. Doing the same here is not a shortcut: it
> means one implementation answers both the exception a program CONSTRUCTS and
> the one caturra's own formatter THREW, and the two cannot drift apart. Only
> where the message does not carry the value — `ParseException`'s error offset,
> `DateTimeParseException`'s parsed text and index — is anything stored beside
> it.

That is checked in both directions: one pin constructs each of the twelve and
reads its accessor, another catches six of them from caturra's own
`String.format` and reads the same accessors. The second found a real
divergence next door — `Charset.forName` threw `UnsupportedCharsetException`
for a name that is not a legal charset name at all, where a JDK asks the two
questions separately and answers the first with `IllegalCharsetNameException`.

**And the summary statistics.** `new IntSummaryStatistics()` and its two
siblings were the last receivers the measurement could not compile. An empty
one keeps the identity values a JDK's accumulator starts from —
`Integer.MAX_VALUE` as the minimum — and `accept`/`combine` fold into it, which
is what makes a hand-made accumulator worth having.

Pinned as `what_a_throwable_carries`,
`the_throwables_that_carry_more_than_a_message`,
`a_caught_format_exception_answers_the_same` and
`a_summary_a_program_fills_itself`. Every class caturra names is now measured
— 225 of them, no receiver left that will not compile — at 3007/3385 method
names answered.

## Ordinary Java the measurement found missing

With every class measured, the gaps sort themselves by how ordinary they are.
This is the ordinary end of the list — what a student writes, not what a
framework calls.

**The three partial dates, finished.** `Year`, `YearMonth` and `MonthDay` had
their readers and a couple of `plusX` methods; they had no `plus(n, unit)`, no
`with(field, value)`, no `until`, no `range`, no `now()`, and a `YearMonth` had
no `withYear`/`withMonth`. The rule that closed all of it at once:

> Every question about MOVING a partial date is that question about the date it
> FILLS OUT TO, asked and then narrowed again. The defaults are the JDK's:
> January, the 1st, and a LEAP year for a month-day — which is why `--02-29` is
> a legal one.

Written that way the calendar arithmetic exists once, in `Date`, rather than
once per partial kind, and what each one supports falls out rather than being
listed: a `Year` moves by years and up, a `YearMonth` by months and up, and a
`MonthDay` by nothing at all. That last is not an omission — a `MonthDay` is a
`TemporalAccessor` and NOT a `Temporal`, so a JDK gives it no `plus`, no
`until` and no two-argument `with` either. It has no year to move in.

Four measured facts a guess would have got wrong:

* `YearMonth.range(DAY_OF_MONTH)` REFUSES — a year-month does not carry a day.
  (The tempting answer, `1 - 29`, is wrong twice over.)
* `MonthDay.range(DAY_OF_MONTH)` for February is `1 - 28/29`: the one range a
  JDK reports as VARIABLE, because a month-day has no year to settle it.
* `MonthDay.of(1, 31).with(FEBRUARY)` is `--02-29`, not the 28th — for the same
  reason.
* `Period.addTo` has two shapes and which one runs is OBSERVABLE. With no
  months in the period a JDK adds the years AS YEARS, so
  `Year.plus(Period.ofYears(2))` works and `Year.plus(Period.ofMonths(2))` is
  "Unsupported unit: Months". No `LocalDate` can show the difference; a `Year`
  can, and caturra was combining them.

And one plain bug the probe found beside them: `Year.toString()` is the year as
a NUMBER — `Year.of(5)` is "5" — where caturra padded it to four digits. The
padding belongs to the values that write a year as part of a longer date
(`0005-03`, `0005-03-14`), where it is what keeps the fields apart.

**The primitive Optionals.** `OptionalInt`, `OptionalLong` and `OptionalDouble`
lacked `ifPresentOrElse`, `orElseGet` and `stream` — the three the object
`Optional` has had. The runtime already answered them (all four kinds are one
heap object); what was missing was the tables, and a way for the lambda pass to
find the target type of `orElseGet(() -> 9)`. That element is not a type
argument here: it is in the class NAME.

**The wrappers' missing siblings.** `Byte.decode`, `Short.decode`,
`Byte.compareUnsigned`, `Short.compareUnsigned` and `Float.toHexString` — every
one of them present on `Integer`, `Long` or `Double` and absent from the
narrower wrapper beside it, because a method table is written once per class.
The narrow `compareUnsigned` answers the DIFFERENCE where the wide ones answer
a sign, and `decode`'s complaint about a value that does not fit
("Value 300 out of range from input 300") is not the one `parseByte` gives.

**And the rest of the ordinary list.** Java 9's `asIterator()` on an
`Enumeration` and a `StringTokenizer` (both are already cursors here, so the
bridge hands the receiver back); Java 11's `Reader.nullReader()`,
`Writer.nullWriter()` and `OutputStream.nullOutputStream()`;
`InputStreamReader.getEncoding()`, which answers the HISTORICAL charset name
("UTF8", not "UTF-8"); `BitSet.intersects`/`toByteArray`;
`BigInteger.toByteArray` and the `new BigInteger(byte[])` that reads it back.

`PrintWriter.checkError()` came with a real defect behind it. A JDK's
`PrintWriter` never throws: a write after `close()` is DROPPED and the error
flag goes up, which is the only way `checkError()` becomes true for a writer
over memory. caturra's `close()` was a no-op, so a closed writer went on
writing — a program's output silently landing somewhere it should not.

Pinned as `the_partial_dates_finished`, `the_edges_of_a_partial_date` and
`the_small_gaps_a_measurement_found`. The measurement reads 3057/3385 across
225 classes, with fourteen more of them at 100%.
