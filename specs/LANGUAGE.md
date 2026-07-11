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
- User-defined **static methods**: parameters, `return` (with javac-style
  missing-return and unexpected-return checks via a reachability-lite
  analysis, so `while (true) { ... return ...; }` needs no trailing return),
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
    every instance method except the regex family
    (`matches`/`replaceAll`/`replaceFirst`), the stream family
    (`chars`/`codePoints`/`lines`), and `getBytes` — those report an
    honest "String.matches exists in Java, but regular expressions are
    not supported by caturra" rather than a misleading cannot-find-symbol.
    Statics: `valueOf` (all overloads incl. `char[]`), `copyValueOf`,
    and `String.format` (2026-07-03) — plus `System.out.printf` and
    `PrintWriter.printf` — via a compiler special-case that synthesizes
    the call descriptor from the actual argument types (the one shape
    the fixed signature tables can't express). The Formatter subset:
    conversions `b B s S c C d o x X e E f g G n % h H`, flags
    `- + 0 , ( #` and space, argument indexes (`%2$s`), width, and
    precision, with Java's exception types and messages
    (`IllegalFormatConversionException: d != java.lang.String`, ...).
    Float conversions round HALF_UP over the shortest-round-trip
    decimal digits, matching Java's `BigDecimal.valueOf` path exactly
    (`%.2f` of `2.675` is `2.68`). `join` still reports the varargs
    limitation. `split` treats its
    delimiter literally, not as a regex (a documented deviation visible
    only for metacharacter delimiters like `"."`), with Java's full
    limit semantics; `intern` preserves reference identity; `hashCode`
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
    answer — reports a reason rather than a cannot-find-symbol.
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
  - `Scanner` over `System.in` — `next`/`nextLine`, the full numeric set
    (`nextInt`/`nextLong`/`nextShort`/`nextByte`/`nextFloat`/`nextDouble`/
    `nextBoolean`), every matching `hasNextX`, and `close` — tokenizing
    like Java, fed by the host console; in the browser this is the
    SharedArrayBuffer blocking-stdin path. A failed `nextX` throws
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
    Pattern/regex members and streams report honest reasons.
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
    for-each walks all three (an index loop over a synthetic accessor,
    since caturra has no iterators — so mutating a map inside such a
    loop silently sees the change where a real JDK throws
    `ConcurrentModificationException`). **`keySet().forEach(k -> ...)`** and
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
    is `merge`/`compute*`/`replaceAll` and `Map.of`; all report honest
    reasons. `keySet`/`values`/`entrySet` and the core methods are
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
    performs them. `iterator`/`stream`/`removeIf` on a `Set` report honest
    reasons. Pinned against a real JDK by `diff_hash_set_core`,
    `_integer_order`, `_bulk_ops` and `_keyset_bridge`.
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
    lambda — see below) instead of the natural ordering.
    `iterator`/`stream`/`removeIf` and the
    range views (`headSet`/`tailSet`/`subSet`/`descendingSet`) report honest
    reasons. Pinned against a real JDK by `diff_tree_set_core`,
    `_strings_and_copy` and `_user_comparable`.
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
    constructed. `Comparator`'s static factories (`comparing`, `naturalOrder`,
    `reversed`, …) are not modelled — write the comparison directly. Pinned
    against a real JDK by `diff_comparator_classes` and `_lambdas`.
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
  - `java.util.stream.Stream<E>` (2026-07-11), the `collection.stream()`
    pipeline, modelled **eagerly** — each intermediate op transforms a vector
    and each terminal op consumes it, which gives identical results to a lazy
    JDK stream for the finite streams student code builds. Intermediate:
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
    `average`/`min`/`max`/`findFirst` gap the streams left. `Optional.of`/
    `empty`/`ofNullable` (constructing one directly) and `map`/`filter`/`orElseGet`
    (which take lambdas) are not yet reachable from source — students almost
    always get an Optional from a stream and immediately unwrap it. Pinned
    against a real JDK by `diff_optional`.
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
    `Queue`), and for-each walks it by index. `iterator`/`stream`/`removeIf`
    report honest reasons; `TreeMap`/`TreeSet`/`ArrayDeque`/`Stack` stay
    unsupported. Pinned against a real JDK by `diff_linked_list_queue`,
    `_deque` and `_as_list`.
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
    Java's does. Its parameters are typed to arrays, where javac takes
    `Object` and throws `ArrayStoreException` for anything else —
    stricter, so anything compiling here still compiles on a JDK.
    Bounds errors are `ArrayIndexOutOfBoundsException`, a null array is
    a `NullPointerException`. `lineSeparator()` is always `"\n"`: the
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
  `close()`), and `throws` clauses parsed and ignored (checked
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
  `name()`, and a default `toString()` returning the name (all
  overridable). `switch` on an enum matches unqualified constant names
  by reference identity; enum constants are singletons so `==` works.
  `@Override` and other annotations are parsed and ignored.

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
  autoboxing of primitives into `Object`, and nested type arguments
  (`Box<Pair<A, B>>`).

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
  supertype. Not yet: constructor arguments (`new Base(args) {...}`)
  and capture of `Outer.this` (the enclosing instance).

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

- `System.arraycopy("a", 0, intArray, 0, 1)` — javac's parameters are
  `Object`; caturra's are arrays.
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
