import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class Pos_clone_println {
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
        System.out.println(Arrays.toString(a.clone()));
    }
}
