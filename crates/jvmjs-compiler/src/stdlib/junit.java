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
  public static void assertNull(Object value) {
    if (value != null) throw new RuntimeException("expected null but was " + value);
  }
  public static void assertNotNull(Object value) {
    if (value == null) throw new RuntimeException("expected a non-null value");
  }
  public static void assertSame(Object expected, Object actual) {
    if (expected != actual) throw new RuntimeException("expected the same object");
  }
  public static void assertNotSame(Object unexpected, Object actual) {
    if (unexpected == actual) throw new RuntimeException("expected a different object");
  }
  public static void fail() {
    throw new RuntimeException("test failed");
  }
  public static void fail(String message) {
    throw new RuntimeException(message);
  }
}
