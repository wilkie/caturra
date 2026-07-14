package org.code.protocol;
import java.util.HashMap;
public class JavabuilderContext {
  private static final JavabuilderContext INSTANCE = new JavabuilderContext();
  private final GlobalProtocol protocol = new GlobalProtocol();
  private final HashMap<Class, JavabuilderSharedObject> objects = new HashMap<>();
  public static JavabuilderContext getInstance() { return INSTANCE; }
  public GlobalProtocol getGlobalProtocol() { return this.protocol; }
  public JavabuilderSharedObject get(Class key) { return this.objects.get(key); }
  public void register(Class key, JavabuilderSharedObject value) { this.objects.put(key, value); }
}
