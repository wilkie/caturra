// Bundled clean-room subset of org.junit.jupiter.api.Assertions, injected
// when a source imports org.junit. Failures throw RuntimeException; the
// test runner treats any throw as a failed test. Enough for the CSA FRQ
// validators (assertTrue / assertEquals / ...).

class Assertions {
  public static void assertTrue(boolean condition) {
    if (!condition) throw new RuntimeException("expected condition to be true");
  }
  public static void assertTrue(boolean condition, String message) {
    if (!condition) throw new RuntimeException(message);
  }
  public static void assertFalse(boolean condition) {
    if (condition) throw new RuntimeException("expected condition to be false");
  }
  public static void assertFalse(boolean condition, String message) {
    if (condition) throw new RuntimeException(message);
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
  public static void assertEquals(boolean expected, boolean actual) {
    if (expected != actual) throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  public static void assertEquals(String expected, String actual) {
    boolean same = expected == null ? actual == null : expected.equals(actual);
    if (!same) throw new RuntimeException("expected " + expected + " but was " + actual);
  }
  public static void assertEquals(Object expected, Object actual) {
    boolean same = expected == null ? actual == null : expected.equals(actual);
    if (!same) throw new RuntimeException("expected " + expected + " but was " + actual);
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
    boolean same = expected == null ? actual == null : expected.equals(actual);
    if (!same)
      throw new RuntimeException(message + " ==> expected: <" + expected + "> but was: <" + actual + ">");
  }
  public static void assertNotEquals(int unexpected, int actual) {
    if (unexpected == actual) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(int unexpected, int actual, String message) {
    if (unexpected == actual) throw new RuntimeException(message);
  }
  public static void assertNotEquals(double unexpected, double actual) {
    if (unexpected == actual) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(double unexpected, double actual, String message) {
    if (unexpected == actual) throw new RuntimeException(message);
  }
  public static void assertNotEquals(boolean unexpected, boolean actual, String message) {
    if (unexpected == actual) throw new RuntimeException(message);
  }
  public static void assertNotEquals(String unexpected, String actual) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(String unexpected, String actual, String message) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException(message);
  }
  public static void assertNotEquals(Object unexpected, Object actual) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException("expected not equal to " + actual);
  }
  public static void assertNotEquals(Object unexpected, Object actual, String message) {
    boolean same = unexpected == null ? actual == null : unexpected.equals(actual);
    if (same) throw new RuntimeException(message);
  }
  public static void assertArrayEquals(Object[] expected, Object[] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(Object[] expected, Object[] actual, String message) {
    if (expected == null || actual == null) { if (expected != actual) throw new RuntimeException(message); return; }
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) {
      boolean same = expected[i] == null ? actual[i] == null : expected[i].equals(actual[i]);
      if (!same) throw new RuntimeException(message + " ==> arrays first differed at element [" + i + "]");
    }
  }
  public static void assertArrayEquals(int[] expected, int[] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(int[] expected, int[] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) if (expected[i] != actual[i]) throw new RuntimeException(message + " ==> arrays first differed at element [" + i + "]");
  }
  public static void assertArrayEquals(double[] expected, double[] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(double[] expected, double[] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) if (expected[i] != actual[i]) throw new RuntimeException(message + " ==> arrays first differed at element [" + i + "]");
  }
  public static void assertArrayEquals(long[] expected, long[] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(long[] expected, long[] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) if (expected[i] != actual[i]) throw new RuntimeException(message + " ==> arrays first differed at element [" + i + "]");
  }
  public static void assertArrayEquals(boolean[] expected, boolean[] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(boolean[] expected, boolean[] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) if (expected[i] != actual[i]) throw new RuntimeException(message + " ==> arrays first differed at element [" + i + "]");
  }
  public static void assertArrayEquals(char[] expected, char[] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(char[] expected, char[] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) if (expected[i] != actual[i]) throw new RuntimeException(message + " ==> arrays first differed at element [" + i + "]");
  }
  // 2D array overloads: compare row by row (each row via the 1D form).
  public static void assertArrayEquals(int[][] expected, int[][] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(int[][] expected, int[][] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) assertArrayEquals(expected[i], actual[i], message);
  }
  public static void assertArrayEquals(double[][] expected, double[][] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(double[][] expected, double[][] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) assertArrayEquals(expected[i], actual[i], message);
  }
  public static void assertArrayEquals(long[][] expected, long[][] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(long[][] expected, long[][] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) assertArrayEquals(expected[i], actual[i], message);
  }
  public static void assertArrayEquals(boolean[][] expected, boolean[][] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(boolean[][] expected, boolean[][] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) assertArrayEquals(expected[i], actual[i], message);
  }
  public static void assertArrayEquals(char[][] expected, char[][] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(char[][] expected, char[][] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) assertArrayEquals(expected[i], actual[i], message);
  }
  public static void assertArrayEquals(Object[][] expected, Object[][] actual) { assertArrayEquals(expected, actual, "array contents differ"); }
  public static void assertArrayEquals(Object[][] expected, Object[][] actual, String message) {
    if (expected.length != actual.length) throw new RuntimeException(message + " ==> array lengths differ");
    for (int i = 0; i < expected.length; i++) assertArrayEquals(expected[i], actual[i], message);
  }
  public static void assertNull(Object value) {
    if (value != null) throw new RuntimeException("expected null but was " + value);
  }
  public static void assertNull(Object value, String message) {
    if (value != null) throw new RuntimeException(message);
  }
  public static void assertNotNull(Object value) {
    if (value == null) throw new RuntimeException("expected a non-null value");
  }
  public static void assertNotNull(Object value, String message) {
    if (value == null) throw new RuntimeException(message);
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
}
