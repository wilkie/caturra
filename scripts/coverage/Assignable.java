/** Answers, for each `from<TAB>to` pair on stdin, whether a value of `from`
 * could stand where `to` is wanted — the question that separates a descriptor
 * that NARROWS a JDK's parameter (caturra models one concrete type where a JDK
 * names the interface) from one that simply disagrees with it. */
import java.io.*;

public class Assignable {
    public static void main(String[] a) throws Exception {
        BufferedReader in = new BufferedReader(new InputStreamReader(System.in));
        String line;
        while ((line = in.readLine()) != null) {
            String[] pair = line.split("\t");
            if (pair.length != 2) continue;
            boolean yes;
            try {
                yes = load(pair[1]).isAssignableFrom(load(pair[0]));
            } catch (Throwable ignored) {
                yes = false;
            }
            System.out.println(line + "\t" + yes);
        }
    }

    private static Class<?> load(String name) throws Exception {
        switch (name) {
            case "int": return int.class;
            case "long": return long.class;
            case "double": return double.class;
            case "float": return float.class;
            case "short": return short.class;
            case "byte": return byte.class;
            case "char": return char.class;
            case "boolean": return boolean.class;
            default: return Class.forName(name);
        }
    }
}
