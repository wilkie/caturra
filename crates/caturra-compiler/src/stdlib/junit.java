// Bundled clean-room subset of org.junit.jupiter.api.Assertions, injected
// when a source imports org.junit. Failures throw RuntimeException; the
// test runner treats any throw as a failed test. Enough for the CSA FRQ
// validators (assertTrue / assertEquals / ...).

class Assertions {
  // JUnit puts its own account of the failure after the message, and a student
  // reads both. The exact wording is pinned against real JUnit 5 in
  // differential_validation_orgcode.rs.
  public static void assertTrue(boolean condition) {
    if (!condition) throw new RuntimeException("expected: <true> but was: <false>");
  }
  public static void assertTrue(boolean condition, String message) {
    if (!condition) throw new RuntimeException(message + " ==> expected: <true> but was: <false>");
  }
  public static void assertFalse(boolean condition) {
    if (condition) throw new RuntimeException("expected: <false> but was: <true>");
  }
  public static void assertFalse(boolean condition, String message) {
    if (condition) throw new RuntimeException(message + " ==> expected: <false> but was: <true>");
  }
  public static void assertEquals(int expected, int actual) {
    if (expected != actual) throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  public static void assertEquals(long expected, long actual) {
    if (expected != actual) throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  public static void assertEquals(double expected, double actual) {
    if (expected != actual) throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  // Comparing doubles within a tolerance: JUnit's delta overloads, which the
  // money/measurement levels use to avoid asserting on exact binary fractions
  // (`assertEquals(0.02, cost, 0.0001, "...")`). Equal within delta passes;
  // JUnit treats a NaN delta, or a negative one, as a failure of the assertion
  // itself, and exactly equal values pass whatever the delta.
  public static void assertEquals(double expected, double actual, double delta) {
    if (!__within(expected, actual, delta))
      throw new RuntimeException("expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertEquals(double expected, double actual, double delta, String message) {
    if (!__within(expected, actual, delta))
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  private static boolean __within(double expected, double actual, double delta) {
    if (expected == actual) return true;
    double difference = expected - actual;
    if (difference < 0) difference = -difference;
    return difference <= delta;
  }
  public static void assertEquals(float expected, float actual, float delta) {
    assertEquals((double) expected, (double) actual, (double) delta);
  }
  public static void assertEquals(float expected, float actual, float delta, String message) {
    assertEquals((double) expected, (double) actual, (double) delta, message);
  }
  public static void assertEquals(boolean expected, boolean actual) {
    if (expected != actual) throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  public static void assertEquals(String expected, String actual) {
    boolean same = expected == null ? actual == null : expected.equals(actual);
    if (!same) throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  // Value-equality for assertEquals: two lists are equal when their elements
  // are pairwise equal via each element's own equals (JUnit / AbstractList
  // semantics), which for user objects dispatches their equals override.
  static boolean __objEquals(Object expected, Object actual) {
    if (expected == null) return actual == null;
    if (expected instanceof java.util.ArrayList<Object> && actual instanceof java.util.ArrayList<Object>) {
      java.util.ArrayList<Object> a = (java.util.ArrayList<Object>) expected;
      java.util.ArrayList<Object> b = (java.util.ArrayList<Object>) actual;
      if (a.size() != b.size()) return false;
      // Compare element-by-element via each element's own equals. No null
      // check on the element: caturra stores primitives unboxed, so `get(i)` may
      // be a bare int (a null check would verify-fail); equals handles it.
      for (int i = 0; i < a.size(); i++) {
        if (!a.get(i).equals(b.get(i))) return false;
      }
      return true;
    }
    return expected.equals(actual);
  }

  public static void assertEquals(Object expected, Object actual) {
    if (!__objEquals(expected, actual))
      throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  // The `message` overloads (JUnit's assertX(expected, actual, message)) —
  // validators pass a message explaining the failure to the student.
  public static void assertEquals(int expected, int actual, String message) {
    if (expected != actual)
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertEquals(long expected, long actual, String message) {
    if (expected != actual)
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertEquals(double expected, double actual, String message) {
    if (expected != actual)
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertEquals(boolean expected, boolean actual, String message) {
    if (expected != actual)
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertEquals(String expected, String actual, String message) {
    boolean same = expected == null ? actual == null : expected.equals(actual);
    if (!same)
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertEquals(Object expected, Object actual, String message) {
    if (!__objEquals(expected, actual))
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertNotEquals(int unexpected, int actual) {
    if (unexpected == actual) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(int unexpected, int actual, String message) {
    if (unexpected == actual)
      throw new RuntimeException(message + " ==> expected: not equal but was: <" + actual + ">");
  }
  public static void assertNotEquals(double unexpected, double actual) {
    if (unexpected == actual) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(double unexpected, double actual, String message) {
    if (unexpected == actual)
      throw new RuntimeException(message + " ==> expected: not equal but was: <" + actual + ">");
  }
  public static void assertNotEquals(boolean unexpected, boolean actual, String message) {
    if (unexpected == actual)
      throw new RuntimeException(message + " ==> expected: not equal but was: <" + actual + ">");
  }
  public static void assertNotEquals(String unexpected, String actual) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(String unexpected, String actual, String message) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException(message + " ==> expected: not equal but was: <" + actual + ">");
  }
  public static void assertNotEquals(Object unexpected, Object actual) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(Object unexpected, Object actual, String message) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException(message + " ==> expected: not equal but was: <" + actual + ">");
  }
  // JUnit's array failures name the exact index that differs, and for a 2D array
  // they name the whole PATH to it (`at index [0][1]`) — so the row's compare
  // carries the enclosing index down. `path` is that prefix: "" at the top.
  // Measured against real JUnit 5, not guessed: the wording below is its own.
  private static String __arrayLengths(String message, String path, int expected, int actual) {
    String prefix = message == null ? "" : message + " ==> ";
    String at = path.isEmpty() ? "" : " at index " + path;
    return prefix + "array lengths differ" + at + ", expected: <" + expected + "> but was: <" + actual + ">";
  }
  private static String __arrayDiffers(String message, String path, int index, String expected, String actual) {
    String prefix = message == null ? "" : message + " ==> ";
    return prefix + "array contents differ at index " + path + "[" + index + "], expected: <"
        + expected + "> but was: <" + actual + ">";
  }
  private static String __arrayNull(String message, String path, boolean expectedWasNull) {
    String prefix = message == null ? "" : message + " ==> ";
    String at = path.isEmpty() ? "" : " at index " + path;
    return prefix + (expectedWasNull ? "expected" : "actual") + " array was <null>" + at;
  }

  public static void assertArrayEquals(Object[] expected, Object[] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(Object[] expected, Object[] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(Object[] expected, Object[] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) {
      boolean same = expected[i] == null ? actual[i] == null : expected[i].equals(actual[i]);
      if (!same) throw new RuntimeException(__arrayDiffers(message, path, i, "" + expected[i], "" + actual[i]));
    }
  }
  public static void assertArrayEquals(int[] expected, int[] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(int[] expected, int[] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(int[] expected, int[] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) {
      if (expected[i] != actual[i]) throw new RuntimeException(__arrayDiffers(message, path, i, "" + expected[i], "" + actual[i]));
    }
  }
  public static void assertArrayEquals(double[] expected, double[] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(double[] expected, double[] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(double[] expected, double[] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) {
      if (expected[i] != actual[i]) throw new RuntimeException(__arrayDiffers(message, path, i, "" + expected[i], "" + actual[i]));
    }
  }
  // The same tolerance, elementwise: the sound levels compare sample arrays.
  public static void assertArrayEquals(double[] expected, double[] actual, double delta) { __arrayEquals(expected, actual, delta, null); }
  public static void assertArrayEquals(double[] expected, double[] actual, double delta, String message) { __arrayEquals(expected, actual, delta, message); }
  private static void __arrayEquals(double[] expected, double[] actual, double delta, String message) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, "", true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, "", false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, "", expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) {
      double difference = expected[i] - actual[i];
      if (difference < 0) difference = -difference;
      if (difference > delta) throw new RuntimeException(__arrayDiffers(message, "", i, "" + expected[i], "" + actual[i]));
    }
  }
  public static void assertArrayEquals(long[] expected, long[] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(long[] expected, long[] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(long[] expected, long[] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) {
      if (expected[i] != actual[i]) throw new RuntimeException(__arrayDiffers(message, path, i, "" + expected[i], "" + actual[i]));
    }
  }
  public static void assertArrayEquals(boolean[] expected, boolean[] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(boolean[] expected, boolean[] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(boolean[] expected, boolean[] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) {
      if (expected[i] != actual[i]) throw new RuntimeException(__arrayDiffers(message, path, i, "" + expected[i], "" + actual[i]));
    }
  }
  public static void assertArrayEquals(char[] expected, char[] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(char[] expected, char[] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(char[] expected, char[] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) {
      if (expected[i] != actual[i]) throw new RuntimeException(__arrayDiffers(message, path, i, "" + expected[i], "" + actual[i]));
    }
  }
  // 2D: each row compares through the 1D form, carrying `[i]` down so the index
  // a failure names is the full path to the element.
  public static void assertArrayEquals(int[][] expected, int[][] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(int[][] expected, int[][] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(int[][] expected, int[][] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) __arrayEquals(expected[i], actual[i], message, path + "[" + i + "]");
  }
  public static void assertArrayEquals(double[][] expected, double[][] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(double[][] expected, double[][] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(double[][] expected, double[][] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) __arrayEquals(expected[i], actual[i], message, path + "[" + i + "]");
  }
  public static void assertArrayEquals(long[][] expected, long[][] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(long[][] expected, long[][] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(long[][] expected, long[][] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) __arrayEquals(expected[i], actual[i], message, path + "[" + i + "]");
  }
  public static void assertArrayEquals(boolean[][] expected, boolean[][] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(boolean[][] expected, boolean[][] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(boolean[][] expected, boolean[][] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) __arrayEquals(expected[i], actual[i], message, path + "[" + i + "]");
  }
  public static void assertArrayEquals(char[][] expected, char[][] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(char[][] expected, char[][] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(char[][] expected, char[][] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) __arrayEquals(expected[i], actual[i], message, path + "[" + i + "]");
  }
  public static void assertArrayEquals(Object[][] expected, Object[][] actual) { __arrayEquals(expected, actual, null, ""); }
  public static void assertArrayEquals(Object[][] expected, Object[][] actual, String message) { __arrayEquals(expected, actual, message, ""); }
  private static void __arrayEquals(Object[][] expected, Object[][] actual, String message, String path) {
    if (expected == null && actual == null) return;
    if (expected == null) throw new RuntimeException(__arrayNull(message, path, true));
    if (actual == null) throw new RuntimeException(__arrayNull(message, path, false));
    if (expected.length != actual.length) throw new RuntimeException(__arrayLengths(message, path, expected.length, actual.length));
    for (int i = 0; i < expected.length; i++) __arrayEquals(expected[i], actual[i], message, path + "[" + i + "]");
  }
  public static void assertNull(Object value) {
    if (value != null) throw new RuntimeException("expected: <null> but was: <" + value + ">");
  }
  public static void assertNull(Object value, String message) {
    if (value != null) throw new RuntimeException(message + " ==> expected: <null> but was: <" + value + ">");
  }
  public static void assertNotNull(Object value) {
    if (value == null) throw new RuntimeException("expected: not <null>");
  }
  public static void assertNotNull(Object value, String message) {
    if (value == null) throw new RuntimeException(message + " ==> expected: not <null>");
  }
  public static void assertSame(Object expected, Object actual) {
    if (expected != actual) throw new RuntimeException("expected the same object");
  }
  public static void assertSame(Object expected, Object actual, String message) {
    if (expected != actual) throw new RuntimeException(message);
  }
  public static void assertNotSame(Object unexpected, Object actual) {
    if (unexpected == actual) throw new RuntimeException("expected a different object");
  }
  public static void assertNotSame(Object unexpected, Object actual, String message) {
    if (unexpected == actual) throw new RuntimeException(message);
  }
  public static void fail() {
    throw new RuntimeException("test failed");
  }
  public static void fail(String message) {
    throw new RuntimeException(message);
  }
  // `assertDoesNotThrow(() -> student.method(), message)`: run the body and
  // fail if it throws. The wording is JUnit's own, verified against real JUnit 5
  // in differential_validation.rs — a Throwable's toString() is already the
  // fully-qualified `java.lang.IllegalStateException: kaboom` JUnit prints.
  public static void assertDoesNotThrow(Executable executable) {
    try {
      executable.execute();
    } catch (Throwable e) {
      throw new RuntimeException("Unexpected exception thrown: " + e);
    }
  }
  public static void assertDoesNotThrow(Executable executable, String message) {
    try {
      executable.execute();
    } catch (Throwable e) {
      throw new RuntimeException(message + " ==> Unexpected exception thrown: " + e);
    }
  }
  // `assertThrows(IllegalStateException.class, () -> student.method())`: the body
  // must throw, and throw something the expected type accepts — a SUBCLASS
  // counts, so this is a runtime type test (`isInstance`) and not a name
  // comparison. Returns the throwable, as JUnit does; caturra erases the
  // generic, so it comes back as a Throwable and a test wanting the exact type
  // must cast. The wording is JUnit's own.
  public static Throwable assertThrows(Class expected, Executable executable) {
    return __assertThrows(expected, executable, null);
  }
  public static Throwable assertThrows(Class expected, Executable executable, String message) {
    return __assertThrows(expected, executable, message);
  }
  private static Throwable __assertThrows(Class expected, Executable executable, String message) {
    String prefix = message == null ? "" : message + " ==> ";
    try {
      executable.execute();
    } catch (Throwable actual) {
      if (expected.isInstance(actual)) {
        return actual;
      }
      throw new RuntimeException(prefix + "Unexpected exception type thrown ==> expected: <"
          + expected.getName() + "> but was: <" + actual.getClass().getName() + ">");
    }
    throw new RuntimeException(
        prefix + "Expected " + expected.getName() + " to be thrown, but nothing was thrown.");
  }
}

// JUnit's `Executable`: the body of an `assertDoesNotThrow`. A functional
// interface, so a lambda binds to it.
interface Executable {
  void execute();
}
