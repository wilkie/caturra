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
  // In-place ascending sort (insertion sort — stable, small inputs).
  public static void sort(int[] a) {
    for (int i = 1; i < a.length; i++) {
      int key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(double[] a) {
    for (int i = 1; i < a.length; i++) {
      double key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(long[] a) {
    for (int i = 1; i < a.length; i++) {
      long key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(char[] a) {
    for (int i = 1; i < a.length; i++) {
      char key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(String[] a) {
    for (int i = 1; i < a.length; i++) {
      String key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j].compareTo(key) > 0) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }

  // A List view of the array (a reference array widens to Object[]).
  public static java.util.ArrayList<Object> asList(Object[] a) {
    java.util.ArrayList<Object> list = new java.util.ArrayList<Object>();
    for (int i = 0; i < a.length; i++) list.add(a[i]);
    return list;
  }
}
