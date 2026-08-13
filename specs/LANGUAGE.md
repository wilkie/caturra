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
  - Deferred: `HashMap` iteration order once a bucket holds ≥8 colliding keys
    (a JDK treeifies the bin, ordering by hash then `compareTo`), and a live
    `Map.Entry` whose hash changed while it sat in a `HashSet` — the same
    hazard for a USER class already matches the JDK exactly.
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
  - Deferred: `subList` is not a live view (it is refused, by that name, on
    every list face), and `java.util.Enumeration` — with it
    `Collections.enumeration`/`list` — is refused by name rather than
    reported as a missing symbol.
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

The other two accepts-invalid are recorded but not fixed: assigning to a
multi-catch parameter (`catch (A | B e) { e = null; }`), and a repeated
modifier (`public public void f()`).

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
- `Math m;`, `Collectors c;`, `Arrays a;` — a class caturra models only as a
  namespace for its static members cannot name a variable, though javac
  accepts the declaration (they are ordinary class types). Nobody writes one,
  but the refusal has to say so: written in full it gave the honest reason,
  written simply it read as a typo — "unknown type 'Math'", about a class
  every program has used.

**More permissive than javac** (caturra accepts; javac rejects). **This
list is empty**, and the `looser_than_javac!` macro exists to keep it
that way — a case asserted there is a case that cannot be forgotten.

It was not empty on 2026-08-13: a cast to `String` accepted ANY reference
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
