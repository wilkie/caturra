import java.lang.reflect.*;
import java.util.*;

/** Public methods a JDK offers on each named class, one
 * `class<TAB>name<TAB>static<TAB>arity` per overload — and with
 * `--signatures`, `class<TAB>name<TAB>static<TAB>returnType<TAB>paramTypes`
 * instead, which is what a probe needs to write a call with real ARGUMENTS
 * rather than nulls. */
public class ApiList {
    public static void main(String[] args) throws Exception {
        Set<String> objectMethods = new HashSet<>();
        for (Method m : Object.class.getMethods()) objectMethods.add(m.getName());
        // `--constructors` lists the public constructors instead, as
        // `class<TAB><init><TAB>false<TAB>arity` — what `scripts/fuzz/panics.py`
        // needs to write `new X(...)`, which is a code path of its own and one
        // the method walk never reaches.
        boolean constructors = false;
        boolean signatures = false;
        for (String arg : args) {
            constructors |= arg.equals("--constructors");
            signatures |= arg.equals("--signatures");
        }
        for (String name : args) {
            if (name.startsWith("--")) continue;
            Class<?> c = Class.forName(name);
            if (constructors) {
                Set<String> arities = new TreeSet<>();
                for (Constructor<?> k : c.getConstructors()) {
                    arities.add("<init>\tfalse\t" + k.getParameterCount());
                }
                for (String s : arities) System.out.println(name + "\t" + s);
                continue;
            }
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
                if (signatures) {
                    StringBuilder params = new StringBuilder();
                    for (Class<?> p : m.getParameterTypes()) {
                        if (params.length() > 0) params.append(',');
                        params.append(p.getName());
                    }
                    seen.add(m.getName() + "\t" + Modifier.isStatic(m.getModifiers())
                            + "\t" + m.getReturnType().getName() + "\t" + params);
                    continue;
                }
                seen.add(m.getName() + "\t" + Modifier.isStatic(m.getModifiers())
                        + "\t" + m.getParameterCount());
            }
            for (String s : seen) System.out.println(name + "\t" + s);
        }
    }
}
