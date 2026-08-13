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
  __Negate(__Predicate inner) { this.inner = inner; }
  public boolean test(Object element) { return !inner.test(element); }
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

interface __Supplier {
  Object get();
}

interface __Runnable {
  void run();
}

interface __Comparator {
  int compare(Object left, Object right);
}
