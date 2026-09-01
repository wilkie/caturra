import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class Pos_factory_returned {
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
    static Object make() {
        
        return Box.of("hi").get();
    }

    public static void main(String[] args) {
        System.out.println(make());
    }
}
