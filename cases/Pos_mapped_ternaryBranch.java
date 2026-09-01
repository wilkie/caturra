import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class Pos_mapped_ternaryBranch {
    static class Node<T> {
        private final T v;
        Node(T v) { this.v = v; }
        T get() { return v; }
    }
    static class Box<T> {
        private final T v;
        Box(T v) { this.v = v; }
        T get() { return v; }
        static <T> Box<T> of(T v) { return new Box<>(v); }
    }
    static void take(Object o) { System.out.println(o); }

    public static void main(String[] args) {
        List<String> l = new ArrayList<>(Arrays.asList("aa", "b"));
        Object t = args.length == 0 ? l.stream().map(String::length).collect(Collectors.toList()) : null;
        System.out.println(t);
    }
}
