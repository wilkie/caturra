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
}

interface __Predicate {
  boolean test(Object element);
}

interface __UnaryOperator {
  Object apply(Object element);
}
