// Bundled clean-room subset of java.util.Arrays, injected when a source
// imports java.util. toString concatenates the elements — each coerced via
// its own toString (unlike a native renderer) so arrays of user objects and
// reflection Field/Constructor print correctly. Primitive overloads cover
// the rest; a reference array widens to Object[] (array covariance).

class Arrays {
  public static String toString(Object[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  public static String toString(int[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  public static String toString(long[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  public static String toString(double[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  public static String toString(boolean[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  public static String toString(char[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
}
