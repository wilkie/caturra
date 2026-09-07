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
  // `parallelSort` IS `sort` here: caturra runs on one thread, so the only
  // difference a JDK's parallel form has is how it divides the work, and
  // dividing it one way gives the same array back. Written as a delegation
  // rather than a second implementation, so the two cannot drift apart.
  public static void parallelSort(int[] a) { sort(a, 0, a.length); }
  public static void parallelSort(int[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  public static void parallelSort(double[] a) { sort(a, 0, a.length); }
  public static void parallelSort(double[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  public static void parallelSort(long[] a) { sort(a, 0, a.length); }
  public static void parallelSort(long[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  public static void parallelSort(char[] a) { sort(a, 0, a.length); }
  public static void parallelSort(char[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  public static void parallelSort(float[] a) { sort(a, 0, a.length); }
  public static void parallelSort(float[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  public static void parallelSort(short[] a) { sort(a, 0, a.length); }
  public static void parallelSort(short[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  public static void parallelSort(byte[] a) { sort(a, 0, a.length); }
  public static void parallelSort(byte[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  public static void parallelSort(String[] a) { sort(a, 0, a.length); }
  public static void parallelSort(String[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
  // ...and any OTHER reference array, which sorts by each element's own
  // `compareTo` exactly as `sort(Object[])` does. Only the `String[]` pair was
  // written, so `Comparable[]` — or any array of a program's own comparable
  // class — had no parallel form at all.
  public static void parallelSort(Object[] a) { sort(a, 0, a.length); }
  public static void parallelSort(Object[] a, int fromIndex, int toIndex) { sort(a, fromIndex, toIndex); }
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
  public static int mismatch(short[] a, short[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) if (a[i] != b[i]) return i;
    return a.length == b.length ? -1 : shared;
  }
  public static int mismatch(byte[] a, byte[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) if (a[i] != b[i]) return i;
    return a.length == b.length ? -1 : shared;
  }
  public static int mismatch(boolean[] a, boolean[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) if (a[i] != b[i]) return i;
    return a.length == b.length ? -1 : shared;
  }
  public static int mismatch(float[] a, float[] b) {
    int shared = a.length < b.length ? a.length : b.length;
    // `Float.compare`, as the double form has it: two NaNs MATCH and 0.0f does
    // not match -0.0f.
    for (int i = 0; i < shared; i++) if (__Float.compare(a[i], b[i]) != 0) return i;
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
  // The parameter is `Object[]`, not `Comparable[]`: a JDK declares
  // `sort(Object[])` and casts each element as it compares, so sorting an
  // array of a class that does not implement Comparable COMPILES and throws
  // ClassCastException — where a `Comparable[]` parameter refused the program
  // javac accepts. (`Collections.sort` is the other rule: it really is
  // declared over `T extends Comparable<? super T>`, so its refusal stands.)
  public static void sort(Object[] a) {
    sort(a, 0, a.length);
  }
  public static void sort(Object[] a, int fromIndex, int toIndex) {
    rangeCheck(a.length, fromIndex, toIndex);
    for (int i = fromIndex + 1; i < toIndex; i++) {
      Comparable key = (Comparable) a[i];
      int j = i - 1;
      while (j >= fromIndex && ((Comparable) a[j]).compareTo(key) > 0) { a[j + 1] = a[j]; j--; }
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
  // `compareUnsigned` reads every element as if it had no sign, so
  // `{-1}` sorts ABOVE `{1}` where `compare` puts it below. The two integer
  // widths answer a SIGN (their `compareUnsigned` does); the two narrow ones
  // answer the DIFFERENCE of the unsigned values, because that is what
  // `Byte.compareUnsigned` and `Short.compareUnsigned` return — measured, not
  // assumed, and the reason `{-1}` against `{1}` is 254 for bytes and 1 for
  // ints.
  public static int compareUnsigned(int[] a, int[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return __Integer.compareUnsigned(a[i], b[i]);
    }
    return a.length - b.length;
  }
  public static int compareUnsigned(long[] a, long[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return __Long.compareUnsigned(a[i], b[i]);
    }
    return a.length - b.length;
  }
  public static int compareUnsigned(byte[] a, byte[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return (a[i] & 0xff) - (b[i] & 0xff);
    }
    return a.length - b.length;
  }
  public static int compareUnsigned(short[] a, short[] b) {
    if (a == b) return 0;
    if (a == null || b == null) return a == null ? -1 : 1;
    int shared = a.length < b.length ? a.length : b.length;
    for (int i = 0; i < shared; i++) {
      if (a[i] != b[i]) return (a[i] & 0xffff) - (b[i] & 0xffff);
    }
    return a.length - b.length;
  }
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

  // ...and the RANGE form of each, which compares two SLICES: the ranges are
  // checked first (a bad one throws before anything is read, and names the
  // index a JDK names), and when one slice is a prefix of the other the answer
  // is the difference of their LENGTHS — not of the arrays'.
  public static int compare(int[] a, int aFromIndex, int aToIndex, int[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return __Integer.compare(a[ai], b[bi]);
    }
    return aLength - bLength;
  }
  public static int compare(long[] a, int aFromIndex, int aToIndex, long[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return __Long.compare(a[ai], b[bi]);
    }
    return aLength - bLength;
  }
  public static int compare(double[] a, int aFromIndex, int aToIndex, double[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      int c = __Double.compare(a[ai], b[bi]);
      if (c != 0) return c;
    }
    return aLength - bLength;
  }
  public static int compare(float[] a, int aFromIndex, int aToIndex, float[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      int c = __Float.compare(a[ai], b[bi]);
      if (c != 0) return c;
    }
    return aLength - bLength;
  }
  public static int compare(char[] a, int aFromIndex, int aToIndex, char[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return a[ai] - b[bi];
    }
    return aLength - bLength;
  }
  public static int compare(short[] a, int aFromIndex, int aToIndex, short[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return __Short.compare(a[ai], b[bi]);
    }
    return aLength - bLength;
  }
  public static int compare(byte[] a, int aFromIndex, int aToIndex, byte[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return __Byte.compare(a[ai], b[bi]);
    }
    return aLength - bLength;
  }
  public static int compare(boolean[] a, int aFromIndex, int aToIndex, boolean[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return __Boolean.compare(a[ai], b[bi]);
    }
    return aLength - bLength;
  }
  public static int compare(String[] a, int aFromIndex, int aToIndex, String[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      String x = a[ai];
      String y = b[bi];
      if (x == null || y == null) {
        if (x != y) return x == null ? -1 : 1;
      } else {
        int c = x.compareTo(y);
        if (c != 0) return c;
      }
    }
    return aLength - bLength;
  }
  public static int compare(Comparable[] a, int aFromIndex, int aToIndex, Comparable[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      Comparable x = a[ai];
      Comparable y = b[bi];
      if (x == null || y == null) {
        if (x != y) return x == null ? -1 : 1;
      } else {
        int c = x.compareTo(y);
        if (c != 0) return c;
      }
    }
    return aLength - bLength;
  }

  // ...and the unsigned reading of the same slices.
  public static int compareUnsigned(int[] a, int aFromIndex, int aToIndex, int[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return __Integer.compareUnsigned(a[ai], b[bi]);
    }
    return aLength - bLength;
  }
  public static int compareUnsigned(long[] a, int aFromIndex, int aToIndex, long[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return __Long.compareUnsigned(a[ai], b[bi]);
    }
    return aLength - bLength;
  }
  public static int compareUnsigned(byte[] a, int aFromIndex, int aToIndex, byte[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return (a[ai] & 0xff) - (b[bi] & 0xff);
    }
    return aLength - bLength;
  }
  public static int compareUnsigned(short[] a, int aFromIndex, int aToIndex, short[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      int ai = aFromIndex + i;
      int bi = bFromIndex + i;
      if (a[ai] != b[bi]) return (a[ai] & 0xffff) - (b[bi] & 0xffff);
    }
    return aLength - bLength;
  }

  // The RANGE forms of `mismatch` and `equals`: the same questions asked of two
  // SLICES. `mismatch` answers an index RELATIVE to each slice's own start, so
  // a difference at the same offset in both reads the same however far into the
  // arrays the slices sit.
  public static int mismatch(int[] a, int aFromIndex, int aToIndex, int[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) if (a[aFromIndex + i] != b[bFromIndex + i]) return i;
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(long[] a, int aFromIndex, int aToIndex, long[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) if (a[aFromIndex + i] != b[bFromIndex + i]) return i;
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(char[] a, int aFromIndex, int aToIndex, char[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) if (a[aFromIndex + i] != b[bFromIndex + i]) return i;
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(short[] a, int aFromIndex, int aToIndex, short[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) if (a[aFromIndex + i] != b[bFromIndex + i]) return i;
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(byte[] a, int aFromIndex, int aToIndex, byte[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) if (a[aFromIndex + i] != b[bFromIndex + i]) return i;
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(boolean[] a, int aFromIndex, int aToIndex, boolean[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) if (a[aFromIndex + i] != b[bFromIndex + i]) return i;
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(double[] a, int aFromIndex, int aToIndex, double[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      if (__Double.compare(a[aFromIndex + i], b[bFromIndex + i]) != 0) return i;
    }
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(float[] a, int aFromIndex, int aToIndex, float[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      if (__Float.compare(a[aFromIndex + i], b[bFromIndex + i]) != 0) return i;
    }
    return aLength == bLength ? -1 : shared;
  }
  public static int mismatch(Object[] a, int aFromIndex, int aToIndex, Object[] b, int bFromIndex, int bToIndex) {
    rangeCheck(a.length, aFromIndex, aToIndex);
    rangeCheck(b.length, bFromIndex, bToIndex);
    int aLength = aToIndex - aFromIndex;
    int bLength = bToIndex - bFromIndex;
    int shared = aLength < bLength ? aLength : bLength;
    for (int i = 0; i < shared; i++) {
      Object x = a[aFromIndex + i];
      Object y = b[bFromIndex + i];
      boolean same = x == null ? y == null : x.equals(y);
      if (!same) return i;
    }
    return aLength == bLength ? -1 : shared;
  }
  public static boolean equals(int[] a, int aFromIndex, int aToIndex, int[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(long[] a, int aFromIndex, int aToIndex, long[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(char[] a, int aFromIndex, int aToIndex, char[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(short[] a, int aFromIndex, int aToIndex, short[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(byte[] a, int aFromIndex, int aToIndex, byte[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(boolean[] a, int aFromIndex, int aToIndex, boolean[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(double[] a, int aFromIndex, int aToIndex, double[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(float[] a, int aFromIndex, int aToIndex, float[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
  public static boolean equals(Object[] a, int aFromIndex, int aToIndex, Object[] b, int bFromIndex, int bToIndex) {
    return mismatch(a, aFromIndex, aToIndex, b, bFromIndex, bToIndex) < 0;
  }
}
