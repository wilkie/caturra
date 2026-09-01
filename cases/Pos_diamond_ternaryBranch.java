import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class Pos_diamond_ternaryBranch {
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
        Node<Integer> n = new Node<>(5);
        Object t = args.length == 0 ? n.get() + 1 : null;
        System.out.println(t);
    }
}
