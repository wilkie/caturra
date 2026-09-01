import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class Pos_clone_anonBody {
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
        int[] a = {1, 2};
        Supplier<Object> anon = new Supplier<Object>() {
            public Object get() { return Arrays.toString(a.clone()); }
        };
        System.out.println(anon.get());
    }
}
