import java.lang.reflect.*;
import java.util.*;

/** Public methods a JDK offers on each named class, one
 * `class<TAB>name<TAB>static<TAB>arity` per overload. */
public class ApiList {
    public static void main(String[] args) throws Exception {
        Set<String> objectMethods = new HashSet<>();
        for (Method m : Object.class.getMethods()) objectMethods.add(m.getName());
        for (String name : args) {
            Class<?> c = Class.forName(name);
            Set<String> seen = new TreeSet<>();
            for (Method m : c.getMethods()) {
                if (m.isSynthetic() || m.isBridge()) continue;
                if (!Modifier.isPublic(m.getModifiers())) continue;
                // Object's own methods are answered generically, not per class.
                if (objectMethods.contains(m.getName()) && !c.equals(Object.class)) continue;
                // Deprecated-for-removal and thread/monitor plumbing are not
                // part of what a program written for this course reaches for.
                // The ARITY too: a probe has to call with a number of
                // arguments some overload really takes, or a compiler that
                // reports a wrong arity as a missing symbol reads as not
                // knowing the name at all.
                seen.add(m.getName() + "\t" + Modifier.isStatic(m.getModifiers())
                        + "\t" + m.getParameterCount());
            }
            for (String s : seen) System.out.println(name + "\t" + s);
        }
    }
}
