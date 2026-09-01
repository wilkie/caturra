import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class Pos_array_field {
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
    static final Object FIELD = Arrays.toString(new int[] {1, 2});

    public static void main(String[] args) {
        System.out.println(FIELD);
    }
}
