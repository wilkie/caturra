/*
 * Erased target types of collection lambdas. These stay internal (the `__`
 * prefix a student cannot write) because `java.util.function.*` is outside
 * the AP CS A subset and caturra models one type parameter per class.
 *
 * Parameters are `Object` because generics are erased. The synthesized lambda
 * class casts them back to the collection's declared element types in its
 * first statements — exactly what javac's bridge method does.
 */
interface __BiConsumer {
  void accept(Object key, Object value);

  default __BiConsumer andThen(__BiConsumer after) { return new __BothBiConsumers(this, after); }
}

class __BothBiConsumers implements __BiConsumer {
  private final __BiConsumer first;
  private final __BiConsumer second;
  __BothBiConsumers(__BiConsumer first, __BiConsumer second) { this.first = first; this.second = second; }
  public void accept(Object key, Object value) { first.accept(key, value); second.accept(key, value); }
}

interface __Consumer {
  void accept(Object element);

  default __Consumer andThen(__Consumer after) { return new __BothConsumers(this, after); }
}

class __BothConsumers implements __Consumer {
  private final __Consumer first;
  private final __Consumer second;
  __BothConsumers(__Consumer first, __Consumer second) { this.first = first; this.second = second; }
  public void accept(Object element) { first.accept(element); second.accept(element); }
}

interface __Predicate {
  boolean test(Object element);

  // The default combinators. NAMED helper classes, not anonymous ones: an
  // anonymous class in a bundled source shares the `Anon$N` counter with the
  // program's own, and the two collided.
  default __Predicate negate() { return new __Negate(this); }

  default __Predicate and(__Predicate other) { return new __And(this, other); }

  default __Predicate or(__Predicate other) { return new __Or(this, other); }
}

class __Negate implements __Predicate {
  private final __Predicate inner;
  // `Predicate.not(null)` throws AT THE CALL (the JDK's `requireNonNull`),
  // not when the predicate is first tested. `negate()` never passes null.
  __Negate(__Predicate inner) {
    if (inner == null) { throw new NullPointerException(); }
    this.inner = inner;
  }
  public boolean test(Object element) { return !inner.test(element); }
}

// `Predicate.isEqual(target)` — the JDK compares TARGET-first
// (`targetRef.equals(object)`), and a null target tests for null instead.
class __IsEqual implements __Predicate {
  private final Object target;
  __IsEqual(Object target) { this.target = target; }
  public boolean test(Object element) {
    if (target == null) { return element == null; }
    return target.equals(element);
  }
}

class __And implements __Predicate {
  private final __Predicate left;
  private final __Predicate right;
  __And(__Predicate left, __Predicate right) { this.left = left; this.right = right; }
  public boolean test(Object element) { return left.test(element) && right.test(element); }
}

class __Or implements __Predicate {
  private final __Predicate left;
  private final __Predicate right;
  __Or(__Predicate left, __Predicate right) { this.left = left; this.right = right; }
  public boolean test(Object element) { return left.test(element) || right.test(element); }
}

interface __UnaryOperator {
  Object apply(Object element);

  default __UnaryOperator andThen(__UnaryOperator after) { return new __AndThen(this, after); }

  default __UnaryOperator compose(__UnaryOperator before) { return new __AndThen(before, this); }
}

class __AndThen implements __UnaryOperator {
  private final __UnaryOperator first;
  private final __UnaryOperator second;
  __AndThen(__UnaryOperator first, __UnaryOperator second) { this.first = first; this.second = second; }
  public Object apply(Object element) { return second.apply(first.apply(element)); }
}

interface __BiFunction {
  Object apply(Object left, Object right);

  default __BiFunction andThen(__UnaryOperator after) { return new __BiThen(this, after); }
}

class __BiThen implements __BiFunction {
  private final __BiFunction first;
  private final __UnaryOperator second;
  __BiThen(__BiFunction first, __UnaryOperator second) { this.first = first; this.second = second; }
  public Object apply(Object left, Object right) { return second.apply(first.apply(left, right)); }
}

// `BinaryOperator.minBy(cmp)` / `maxBy(cmp)` — the two-argument function that
// keeps one of its arguments. Ties go to the LEFT one in a JDK, which
// `<= 0` / `>= 0` preserve.
class __MinBy implements __BiFunction {
  private final __Comparator order;
  __MinBy(__Comparator order) { this.order = order; }
  public Object apply(Object left, Object right) {
    return order.compare(left, right) <= 0 ? left : right;
  }
}

class __MaxBy implements __BiFunction {
  private final __Comparator order;
  __MaxBy(__Comparator order) { this.order = order; }
  public Object apply(Object left, Object right) {
    return order.compare(left, right) >= 0 ? left : right;
  }
}

interface __Supplier {
  Object get();
}

interface __Runnable {
  void run();
}

interface __Comparator {
  int compare(Object left, Object right);
}

/*
 * The PRIMITIVE specializations, and `BiPredicate`. Each is its own interface
 * rather than another name for one of the SAMs above, because a shared one
 * would make `aFunction.applyAsInt(x)` and `anIntSupplier.get()` compile —
 * both are javac errors, and permissiveness is the direction that must not
 * grow quietly.
 *
 * Their PARAMETERS are `Object` like every other erased SAM here: the
 * synthesized lambda class casts them back to the declared primitive in its
 * first statements, and a caller's `int` boxes on the way in. Only the method
 * NAME and the RETURN are specialized, which is all a program can observe.
 */
interface __IntUnaryOperator {
  int applyAsInt(Object value);

  default __IntUnaryOperator andThen(__IntUnaryOperator after) { return new __IntChain(this, after); }

  default __IntUnaryOperator compose(__IntUnaryOperator before) { return new __IntChain(before, this); }
}

class __IntChain implements __IntUnaryOperator {
  private final __IntUnaryOperator first;
  private final __IntUnaryOperator second;
  __IntChain(__IntUnaryOperator first, __IntUnaryOperator second) { this.first = first; this.second = second; }
  public int applyAsInt(Object value) { return second.applyAsInt(first.applyAsInt(value)); }
}

interface __IntBinaryOperator { int applyAsInt(Object left, Object right); }

interface __IntPredicate {
  boolean test(Object value);

  default __IntPredicate negate() { return new __IntNegate(this); }

  default __IntPredicate and(__IntPredicate other) { return new __IntAnd(this, other); }

  default __IntPredicate or(__IntPredicate other) { return new __IntOr(this, other); }
}

class __IntNegate implements __IntPredicate {
  private final __IntPredicate inner;
  __IntNegate(__IntPredicate inner) { this.inner = inner; }
  public boolean test(Object value) { return !inner.test(value); }
}

class __IntAnd implements __IntPredicate {
  private final __IntPredicate left;
  private final __IntPredicate right;
  __IntAnd(__IntPredicate left, __IntPredicate right) { this.left = left; this.right = right; }
  public boolean test(Object value) { return left.test(value) && right.test(value); }
}

class __IntOr implements __IntPredicate {
  private final __IntPredicate left;
  private final __IntPredicate right;
  __IntOr(__IntPredicate left, __IntPredicate right) { this.left = left; this.right = right; }
  public boolean test(Object value) { return left.test(value) || right.test(value); }
}

interface __IntSupplier { int getAsInt(); }

interface __IntConsumer {
  void accept(Object value);

  default __IntConsumer andThen(__IntConsumer after) { return new __BothIntConsumers(this, after); }
}

class __BothIntConsumers implements __IntConsumer {
  private final __IntConsumer first;
  private final __IntConsumer second;
  __BothIntConsumers(__IntConsumer first, __IntConsumer second) { this.first = first; this.second = second; }
  public void accept(Object value) { first.accept(value); second.accept(value); }
}

interface __IntFunction { Object apply(Object value); }

interface __ToIntFunction { int applyAsInt(Object value); }

interface __DoubleUnaryOperator {
  double applyAsDouble(Object value);

  default __DoubleUnaryOperator andThen(__DoubleUnaryOperator after) { return new __DoubleChain(this, after); }

  default __DoubleUnaryOperator compose(__DoubleUnaryOperator before) { return new __DoubleChain(before, this); }
}

class __DoubleChain implements __DoubleUnaryOperator {
  private final __DoubleUnaryOperator first;
  private final __DoubleUnaryOperator second;
  __DoubleChain(__DoubleUnaryOperator first, __DoubleUnaryOperator second) { this.first = first; this.second = second; }
  public double applyAsDouble(Object value) { return second.applyAsDouble(first.applyAsDouble(value)); }
}

interface __DoubleBinaryOperator { double applyAsDouble(Object left, Object right); }

interface __DoublePredicate {
  boolean test(Object value);

  default __DoublePredicate negate() { return new __DoubleNegate(this); }

  default __DoublePredicate and(__DoublePredicate other) { return new __DoubleAnd(this, other); }

  default __DoublePredicate or(__DoublePredicate other) { return new __DoubleOr(this, other); }
}

class __DoubleNegate implements __DoublePredicate {
  private final __DoublePredicate inner;
  __DoubleNegate(__DoublePredicate inner) { this.inner = inner; }
  public boolean test(Object value) { return !inner.test(value); }
}

class __DoubleAnd implements __DoublePredicate {
  private final __DoublePredicate left;
  private final __DoublePredicate right;
  __DoubleAnd(__DoublePredicate left, __DoublePredicate right) { this.left = left; this.right = right; }
  public boolean test(Object value) { return left.test(value) && right.test(value); }
}

class __DoubleOr implements __DoublePredicate {
  private final __DoublePredicate left;
  private final __DoublePredicate right;
  __DoubleOr(__DoublePredicate left, __DoublePredicate right) { this.left = left; this.right = right; }
  public boolean test(Object value) { return left.test(value) || right.test(value); }
}

interface __DoubleSupplier { double getAsDouble(); }

interface __DoubleConsumer {
  void accept(Object value);

  default __DoubleConsumer andThen(__DoubleConsumer after) { return new __BothDoubleConsumers(this, after); }
}

class __BothDoubleConsumers implements __DoubleConsumer {
  private final __DoubleConsumer first;
  private final __DoubleConsumer second;
  __BothDoubleConsumers(__DoubleConsumer first, __DoubleConsumer second) { this.first = first; this.second = second; }
  public void accept(Object value) { first.accept(value); second.accept(value); }
}

interface __ToDoubleFunction { double applyAsDouble(Object value); }

interface __LongUnaryOperator {
  long applyAsLong(Object value);

  default __LongUnaryOperator andThen(__LongUnaryOperator after) { return new __LongChain(this, after); }

  default __LongUnaryOperator compose(__LongUnaryOperator before) { return new __LongChain(before, this); }
}

class __LongChain implements __LongUnaryOperator {
  private final __LongUnaryOperator first;
  private final __LongUnaryOperator second;
  __LongChain(__LongUnaryOperator first, __LongUnaryOperator second) { this.first = first; this.second = second; }
  public long applyAsLong(Object value) { return second.applyAsLong(first.applyAsLong(value)); }
}

interface __LongBinaryOperator { long applyAsLong(Object left, Object right); }

interface __LongPredicate {
  boolean test(Object value);

  default __LongPredicate negate() { return new __LongNegate(this); }

  default __LongPredicate and(__LongPredicate other) { return new __LongAnd(this, other); }

  default __LongPredicate or(__LongPredicate other) { return new __LongOr(this, other); }
}

class __LongNegate implements __LongPredicate {
  private final __LongPredicate inner;
  __LongNegate(__LongPredicate inner) { this.inner = inner; }
  public boolean test(Object value) { return !inner.test(value); }
}

class __LongAnd implements __LongPredicate {
  private final __LongPredicate left;
  private final __LongPredicate right;
  __LongAnd(__LongPredicate left, __LongPredicate right) { this.left = left; this.right = right; }
  public boolean test(Object value) { return left.test(value) && right.test(value); }
}

class __LongOr implements __LongPredicate {
  private final __LongPredicate left;
  private final __LongPredicate right;
  __LongOr(__LongPredicate left, __LongPredicate right) { this.left = left; this.right = right; }
  public boolean test(Object value) { return left.test(value) || right.test(value); }
}

interface __LongSupplier { long getAsLong(); }

interface __LongConsumer {
  void accept(Object value);

  default __LongConsumer andThen(__LongConsumer after) { return new __BothLongConsumers(this, after); }
}

class __BothLongConsumers implements __LongConsumer {
  private final __LongConsumer first;
  private final __LongConsumer second;
  __BothLongConsumers(__LongConsumer first, __LongConsumer second) { this.first = first; this.second = second; }
  public void accept(Object value) { first.accept(value); second.accept(value); }
}

interface __ToLongFunction { long applyAsLong(Object value); }

interface __BooleanSupplier { boolean getAsBoolean(); }

/*
 * The remaining primitive shapes. Same rule as above: each is its own
 * interface, even where two have the identical erased signature
 * (`ToLongFunction` and `IntToLongFunction` are both `long applyAsLong(Object)`
 * here), so that calling one's method on the other stays the compile error it
 * is in Java.
 */
interface __DoubleFunction { Object apply(Object value); }

interface __LongFunction { Object apply(Object value); }

interface __IntToLongFunction { long applyAsLong(Object value); }

interface __IntToDoubleFunction { double applyAsDouble(Object value); }

interface __LongToIntFunction { int applyAsInt(Object value); }

interface __LongToDoubleFunction { double applyAsDouble(Object value); }

interface __DoubleToIntFunction { int applyAsInt(Object value); }

interface __DoubleToLongFunction { long applyAsLong(Object value); }

interface __ObjIntConsumer { void accept(Object left, Object right); }

interface __ObjLongConsumer { void accept(Object left, Object right); }

interface __ObjDoubleConsumer { void accept(Object left, Object right); }

interface __ToIntBiFunction { int applyAsInt(Object left, Object right); }

interface __ToLongBiFunction { long applyAsLong(Object left, Object right); }

interface __ToDoubleBiFunction { double applyAsDouble(Object left, Object right); }

interface __BiPredicate {
  boolean test(Object left, Object right);

  default __BiPredicate negate() { return new __BiNegate(this); }

  default __BiPredicate and(__BiPredicate other) { return new __BiAnd(this, other); }

  default __BiPredicate or(__BiPredicate other) { return new __BiOr(this, other); }
}

class __BiNegate implements __BiPredicate {
  private final __BiPredicate inner;
  __BiNegate(__BiPredicate inner) { this.inner = inner; }
  public boolean test(Object left, Object right) { return !inner.test(left, right); }
}

class __BiAnd implements __BiPredicate {
  private final __BiPredicate left;
  private final __BiPredicate right;
  __BiAnd(__BiPredicate left, __BiPredicate right) { this.left = left; this.right = right; }
  public boolean test(Object a, Object b) { return left.test(a, b) && right.test(a, b); }
}

class __BiOr implements __BiPredicate {
  private final __BiPredicate left;
  private final __BiPredicate right;
  __BiOr(__BiPredicate left, __BiPredicate right) { this.left = left; this.right = right; }
  public boolean test(Object a, Object b) { return left.test(a, b) || right.test(a, b); }
}
