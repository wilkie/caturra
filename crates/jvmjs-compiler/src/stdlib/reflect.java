// Bundled subset of java.lang.reflect.Modifier, injected when a source uses
// `Modifier.` (and defines no own Modifier). The methods decode the JVM
// access-flag bits carried by a Field/Method/Constructor's getModifiers().

class Modifier {
  public static boolean isPublic(int m) { return (m & 0x0001) != 0; }
  public static boolean isPrivate(int m) { return (m & 0x0002) != 0; }
  public static boolean isProtected(int m) { return (m & 0x0004) != 0; }
  public static boolean isStatic(int m) { return (m & 0x0008) != 0; }
  public static boolean isFinal(int m) { return (m & 0x0010) != 0; }
  public static boolean isSynchronized(int m) { return (m & 0x0020) != 0; }
  public static boolean isVolatile(int m) { return (m & 0x0040) != 0; }
  public static boolean isTransient(int m) { return (m & 0x0080) != 0; }
  public static boolean isNative(int m) { return (m & 0x0100) != 0; }
  public static boolean isInterface(int m) { return (m & 0x0200) != 0; }
  public static boolean isAbstract(int m) { return (m & 0x0400) != 0; }
  public static boolean isStrict(int m) { return (m & 0x0800) != 0; }

  // Canonical modifier order (java.lang.reflect.Modifier.toString).
  public static String toString(int m) {
    String s = "";
    if (isPublic(m)) s = s + "public ";
    if (isProtected(m)) s = s + "protected ";
    if (isPrivate(m)) s = s + "private ";
    if (isAbstract(m)) s = s + "abstract ";
    if (isStatic(m)) s = s + "static ";
    if (isFinal(m)) s = s + "final ";
    if (isTransient(m)) s = s + "transient ";
    if (isVolatile(m)) s = s + "volatile ";
    if (isSynchronized(m)) s = s + "synchronized ";
    if (isNative(m)) s = s + "native ";
    if (isStrict(m)) s = s + "strictfp ";
    if (isInterface(m)) s = s + "interface ";
    return s.trim();
  }
}
