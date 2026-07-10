/*
 * The erased target type of a `Map.forEach` lambda.
 *
 * `java.util.function.BiConsumer<K,V>` is outside the AP CS A subset and
 * caturra models only one type parameter per class, so this stays internal:
 * the `__` prefix is the established convention (`__NbhdWorld`,
 * `System.__uiAwait`) for a name a student cannot write.
 *
 * The parameters are `Object` because generics are erased. The synthesized
 * lambda class casts them back to the map's declared key and value types in
 * its first two statements — exactly what javac's bridge method does.
 */
interface __BiConsumer {
  void accept(Object key, Object value);
}
