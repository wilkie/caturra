import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class Pos_boxed_anonBody {
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
        
        Supplier<Object> anon = new Supplier<Object>() {
            public Object get() { return Integer.valueOf(3).compareTo(4); }
        };
        System.out.println(anon.get());
    }
}
