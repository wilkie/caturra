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
  public static void sort(float[] a) {
    for (int i = 1; i < a.length; i++) {
      float key = a[i];
      int j = i - 1;
      while (j >= 0 && a[j] > key) { a[j + 1] = a[j]; j--; }
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
    for (int i = 0; i < a.length; i++) if (Double.compare(a[i], b[i]) != 0) return false;
    return true;
  }
  public static boolean equals(float[] a, float[] b) {
    if (a == b) return true;
    if (a == null || b == null || a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) if (Float.compare(a[i], b[i]) != 0) return false;
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
    for (int i = 0; i < a.length; i++) result = 31 * result + Long.hashCode(a[i]);
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
    for (int i = 0; i < a.length; i++) result = 31 * result + Boolean.hashCode(a[i]);
    return result;
  }
  public static int hashCode(double[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + Double.hashCode(a[i]);
    return result;
  }
  public static int hashCode(float[] a) {
    if (a == null) return 0;
    int result = 1;
    for (int i = 0; i < a.length; i++) result = 31 * result + Float.hashCode(a[i]);
    return result;
  }

  // Natural-ordering sort of a reference array, by each element's compareTo.
  // Insertion sort, so equal elements keep their order — Arrays.sort of a
  // reference array is stable, unlike its primitive overloads.
  public static void sort(Comparable[] a) {
    for (int i = 1; i < a.length; i++) {
      Comparable key = a[i];
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
