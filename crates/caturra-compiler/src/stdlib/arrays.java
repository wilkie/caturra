// Bundled clean-room subset of java.util.Arrays, injected when a source
// imports java.util. Written in Java rather than as a native intrinsic so
// that every element operation dispatches: toString concatenates elements
// through their own toString, equals and hashCode call theirs, and sort
// calls their compareTo. Primitive overloads cover the rest; a reference
// array widens to Object[] (array covariance).

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
  public static String toString(float[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  public static String toString(short[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  public static String toString(byte[] a) {
    if (a == null) return "null";
    if (a.length == 0) return "[]";
    String s = "[" + a[0];
    for (int i = 1; i < a.length; i++) s = s + ", " + a[i];
    return s + "]";
  }
  // java.util.Arrays.rangeCheck: the SAME order of checks, so a bad range
  // throws before anything is sorted and names the index the JDK names.
  private static void rangeCheck(int length, int fromIndex, int toIndex) {
    if (fromIndex > toIndex)
      throw new IllegalArgumentException("fromIndex(" + fromIndex + ") > toIndex(" + toIndex + ")");
    if (fromIndex < 0) throw new ArrayIndexOutOfBoundsException(fromIndex);
    if (toIndex > length) throw new ArrayIndexOutOfBoundsException(toIndex);
  }

  // In-place ascending sort (insertion sort — stable, small inputs).
  public static void sort(int[] a) {
    sort(a, 0, a.length);
  }
  // Sort a[fromIndex..toIndex).
  public static void sort(int[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      int key = a[i];
      int j = i - 1;
      while (j >= fromIndex && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(double[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      double key = a[i];
      int j = i - 1;
      while (j >= fromIndex && __Double.compare(a[j], key) > 0) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(long[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      long key = a[i];
      int j = i - 1;
      while (j >= fromIndex && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(char[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      char key = a[i];
      int j = i - 1;
      while (j >= fromIndex && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(float[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      float key = a[i];
      int j = i - 1;
      while (j >= fromIndex && __Float.compare(a[j], key) > 0) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(short[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      short key = a[i];
      int j = i - 1;
      while (j >= fromIndex && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(byte[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      byte key = a[i];
      int j = i - 1;
      while (j >= fromIndex && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(double[] a) {
    // Double.compare, not `>`: NaN sorts last (NaN > x is always false) and
    // -0.0 sorts before 0.0, matching java.util.Arrays.
    for (int i = 1; i < a.length; i++) {
      double key = a[i];
      int j = i - 1;
      while (j >= 0 && __Double.compare(a[j], key) > 0) { a[j + 1] = a[j]; j--; }
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
  public static void sort(float[] a) {
    // Float.compare, not `>`: NaN sorts last and -0.0f before 0.0f.
    for (int i = 1; i < a.length; i++) {
      float key = a[i];
      int j = i - 1;
      while (j >= 0 && __Float.compare(a[j], key) > 0) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(short[] a) {
    for (int i = 1; i < a.length; i++) {
      short key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(byte[] a) {
    for (int i = 1; i < a.length; i++) {
      byte key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j] > key) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }
  public static void sort(String[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      String key = a[i];
      int j = i - 1;
      while (j >= fromIndex && a[j].compareTo(key) > 0) { a[j + 1] = a[j]; j--; }
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

  // Element-wise equality. A reference array asks each element's own equals,
  // null-safely, as Objects.equals does. A double or float array compares
  // raw bits, so NaN equals itself and -0.0 does not equal 0.0 — which is
  // what Double.compare reports, and the opposite of what == would say.
  // `mismatch` (Java 9): the first index where the two differ, or -1 when one
  // is a prefix of the other and they are the same length. A SHORTER array
  // that matches so far mismatches at its own length, which is what makes
  // `mismatch([1], [1, 2])` answer 1 rather than -1.
  public static int mismatch(int[] a, int[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) if (a[i] != b[i]) return i;
    return a.length == b.length ? -1 : shared;
  }
  public static int mismatch(long[] a, long[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) if (a[i] != b[i]) return i;
    return a.length == b.length ? -1 : shared;
  }
  public static int mismatch(double[] a, double[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    // The DOUBLE comparison is `Double.compare`, not `==`: two NaNs match here
    // and 0.0 does not match -0.0, exactly as `Arrays.equals` has it.
    for (int i = 0; i < shared; i++) if (__Double.compare(a[i], b[i]) != 0) return i;
    return a.length == b.length ? -1 : shared;
  }
  public static int mismatch(char[] a, char[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) if (a[i] != b[i]) return i;
    return a.length == b.length ? -1 : shared;
  }
  public static int mismatch(Object[] a, Object[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      Object x = a[i];
      Object y = b[i];
      boolean same = x == null ? y == null : x.equals(y);
      if (!same) return i;
    }
    return a.length == b.length ? -1 : shared;
  }

  public static boolean equals(Object[] a, Object[] b) {
    if (a == b) return true;
    if (a == null || b == null) return false;
    if (a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) {
      Object x = a[i];
      Object y = b[i];
      if (x == null) {
        if (y != null) return false;
      } else if (!x.equals(y)) {
        return false;
      }
    }
    return true;
  }
  public static boolean equals(int[] a, int[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (a[i] != b[i]) return false;
    return true;
  }
  public static boolean equals(long[] a, long[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (a[i] != b[i]) return false;
    return true;
  }
  public static boolean equals(short[] a, short[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (a[i] != b[i]) return false;
    return true;
  }
  public static boolean equals(byte[] a, byte[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (a[i] != b[i]) return false;
    return true;
  }
  public static boolean equals(char[] a, char[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (a[i] != b[i]) return false;
    return true;
  }
  public static boolean equals(boolean[] a, boolean[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (a[i] != b[i]) return false;
    return true;
  }
  public static boolean equals(double[] a, double[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (__Double.compare(a[i], b[i]) != 0) return false;
    return true;
  }
  public static boolean equals(float[] a, float[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (__Float.compare(a[i], b[i]) != 0) return false;
    return true;
  }

  // The 31-fold of the elements' own hash codes, so that two arrays which
  // are Arrays.equals have the same Arrays.hashCode.
  public static int hashCode(Object[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) {
      Object e = a[i];
      result = 31 * result + (e == null ? 0 : e.hashCode());
    }
    return result;
  }
  public static int hashCode(int[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + a[i];
    return result;
  }
  public static int hashCode(long[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + __Long.hashCode(a[i]);
    return result;
  }
  public static int hashCode(short[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + a[i];
    return result;
  }
  public static int hashCode(byte[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + a[i];
    return result;
  }
  public static int hashCode(char[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + a[i];
    return result;
  }
  public static int hashCode(boolean[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + __Boolean.hashCode(a[i]);
    return result;
  }
  public static int hashCode(double[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + __Double.hashCode(a[i]);
    return result;
  }
  public static int hashCode(float[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + __Float.hashCode(a[i]);
    return result;
  }

  // Natural-ordering sort of a reference array, by each element's compareTo.
  // Insertion sort, so equal elements keep their order — Arrays.sort of a
  // reference array is stable, unlike its primitive overloads.
  public static void sort(Comparable[] a) {
    sort(a, 0, a.length);
  }
  public static void sort(Comparable[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      Comparable key = a[i];
      int j = i - 1;
      while (j >= fromIndex && a[j].compareTo(key) > 0) { a[j + 1] = a[j]; j--; }
      a[j + 1] = key;
    }
  }

  // `compare` (Java 9): LEXICOGRAPHIC order over two arrays — the first index
  // where they differ decides, and when one is a prefix of the other the
  // SHORTER one is smaller (the JDK returns the length difference, so the
  // magnitude is that difference and not just its sign). A null array sorts
  // before a non-null one, and each element pair is compared the way the
  // wrapper's own `compare` does, so `Double.NaN` is greater than everything
  // and `-0.0` is less than `0.0` — the same total order `sort` imposes.
  public static int compare(int[] a, int[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return __Integer.compare(a[i], b[i]);
    }
    return a.length - b.length;
  }
  public static int compare(long[] a, long[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return __Long.compare(a[i], b[i]);
    }
    return a.length - b.length;
  }
  public static int compare(double[] a, double[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      int c = __Double.compare(a[i], b[i]);
      if (c != 0) return c;
    }
    return a.length - b.length;
  }
  public static int compare(float[] a, float[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      int c = __Float.compare(a[i], b[i]);
      if (c != 0) return c;
    }
    return a.length - b.length;
  }
  public static int compare(char[] a, char[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      // Subtracted rather than routed through `Character.compare`: a program
      // may declare its own class named `Character` (a play's cast, in the
      // corpus), and a bundled library must not depend on a name a program is
      // free to take. `Character.compare` IS this subtraction.
      if (a[i] != b[i]) return a[i] - b[i];
    }
    return a.length - b.length;
  }
  public static int compare(short[] a, short[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return __Short.compare(a[i], b[i]);
    }
    return a.length - b.length;
  }
  public static int compare(byte[] a, byte[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return __Byte.compare(a[i], b[i]);
    }
    return a.length - b.length;
  }
  public static int compare(boolean[] a, boolean[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return __Boolean.compare(a[i], b[i]);
    }
    return a.length - b.length;
  }
  public static int compare(String[] a, String[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      String x = a[i];
      String y = b[i];
      // A null ELEMENT is smaller than any value, and two nulls are equal —
      // this is where `compare` differs from a plain `compareTo` chain, which
      // would throw.
      if (x == null || y == null) {
        if (x != y) return x == null ? -1 : 1;
      } else {
        int c = x.compareTo(y);
        if (c != 0) return c;
      }
    }
    return a.length - b.length;
  }
  public static int compare(Comparable[] a, Comparable[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      Comparable x = a[i];
      Comparable y = b[i];
      if (x == null || y == null) {
        if (x != y) return x == null ? -1 : 1;
      } else {
        int c = x.compareTo(y);
        if (c != 0) return c;
      }
    }
    return a.length - b.length;
  }
}
