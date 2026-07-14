"""The programs behind the compatibility page.

Each one is a whole, runnable Java program that PROVES something: run it on a real
JDK 11 and on caturra and compare. Nothing here is a claim — `compat-record.py`
asks both engines what actually happens and writes the answer into
`features.json`, and `tests/compat_manifest.rs` re-asks on every CI run.

Keep the output small and deterministic (no times, no hashes, no `Math.random`) —
it is compared byte for byte.
"""

FEATURES = [
    # ----- the language -----
    dict(
        id="classes",
        category="Language",
        title="Classes, constructors, encapsulation",
        summary="Fields, a constructor, private state behind accessors, and toString().",
        main="Book",
        source='''
public class Book {
    private final String title;
    private int pages;

    public Book(String title, int pages) {
        this.title = title;
        this.pages = pages;
    }

    public String getTitle() { return title; }
    public void read(int howMany) { pages -= howMany; }
    public int getPagesLeft() { return pages; }

    @Override
    public String toString() { return title + " (" + pages + " pages left)"; }

    public static void main(String[] args) {
        Book book = new Book("Dune", 412);
        book.read(12);
        System.out.println(book);
        System.out.println(book.getTitle() + " -> " + book.getPagesLeft());
    }
}
''',
    ),
    dict(
        id="inheritance",
        category="Language",
        title="Inheritance and polymorphism",
        summary="A subclass overrides a method; the call dispatches on the runtime type.",
        main="Shapes",
        source='''
public class Shapes {
    static class Shape {
        String name() { return "shape"; }
        double area() { return 0; }
        @Override public String toString() { return name() + " area=" + area(); }
    }

    static class Circle extends Shape {
        private final double r;
        Circle(double r) { this.r = r; }
        @Override String name() { return "circle"; }
        @Override double area() { return Math.round(Math.PI * r * r * 100) / 100.0; }
    }

    static class Square extends Shape {
        private final double side;
        Square(double side) { this.side = side; }
        @Override String name() { return "square"; }
        @Override double area() { return side * side; }
    }

    public static void main(String[] args) {
        Shape[] shapes = { new Circle(2), new Square(3), new Shape() };
        for (Shape shape : shapes) {
            System.out.println(shape);
        }
    }
}
''',
    ),
    dict(
        id="interfaces",
        category="Language",
        title="Interfaces and abstract classes",
        summary="An abstract base, an interface, and a default method.",
        main="Speak",
        source='''
public class Speak {
    interface Greeter {
        String greet();
        default String greetTwice() { return greet() + " " + greet(); }
    }

    abstract static class Animal implements Greeter {
        abstract String sound();
        public String greet() { return sound() + "!"; }
    }

    static class Dog extends Animal {
        String sound() { return "woof"; }
    }

    public static void main(String[] args) {
        Greeter dog = new Dog();
        System.out.println(dog.greet());
        System.out.println(dog.greetTwice());
    }
}
''',
    ),
    dict(
        id="generics",
        category="Language",
        title="Generics",
        summary="A generic class and a generic method, erased as Java erases them.",
        main="Pair",
        source='''
import java.util.ArrayList;
import java.util.List;

public class Pair<T> {
    private final T left;
    private final T right;

    public Pair(T left, T right) {
        this.left = left;
        this.right = right;
    }

    public T getLeft() { return left; }

    public static void main(String[] args) {
        Pair<String> words = new Pair<>("hello", "world");
        System.out.println(words.getLeft());

        List<Integer> numbers = new ArrayList<>();
        numbers.add(3);
        numbers.add(1);
        numbers.add(2);
        System.out.println(numbers);
    }
}
''',
    ),
    dict(
        id="lambdas",
        category="Language",
        title="Lambdas and functional interfaces",
        summary="A lambda passed to a method, and one stored in a variable.",
        main="Lambdas",
        source='''
import java.util.ArrayList;
import java.util.List;

public class Lambdas {
    interface Transform {
        String apply(String value);
    }

    static String shout(String word, Transform transform) {
        return transform.apply(word);
    }

    public static void main(String[] args) {
        System.out.println(shout("hello", w -> w.toUpperCase() + "!"));

        Transform reverse = w -> new StringBuilder(w).reverse().toString();
        System.out.println(reverse.apply("caturra"));

        List<String> names = new ArrayList<>();
        names.add("ada");
        names.add("grace");
        names.forEach(name -> System.out.println("hi " + name));
    }
}
''',
    ),
    dict(
        id="anonymous-classes",
        category="Language",
        title="Anonymous inner classes",
        summary="An interface implemented inline, capturing a local variable.",
        main="Anon",
        source='''
public class Anon {
    interface Counter {
        int next();
    }

    public static void main(String[] args) {
        final int start = 10;
        Counter counter = new Counter() {
            private int seen = 0;
            @Override public int next() { return start + seen++; }
        };
        System.out.println(counter.next());
        System.out.println(counter.next());
        System.out.println(counter.next());
    }
}
''',
    ),
    dict(
        id="control-flow",
        category="Language",
        title="Control flow",
        summary="if/else, switch, for, while, do-while, and the ternary operator.",
        main="Flow",
        source='''
public class Flow {
    public static void main(String[] args) {
        for (int i = 1; i <= 5; i++) {
            String kind;
            switch (i % 3) {
                case 0: kind = "fizz"; break;
                case 1: kind = "one"; break;
                default: kind = "two";
            }
            System.out.println(i + " " + kind + (i % 2 == 0 ? " even" : " odd"));
        }

        int countdown = 3;
        while (countdown > 0) {
            System.out.print(countdown + " ");
            countdown--;
        }
        System.out.println();

        int doubling = 1;
        do {
            doubling *= 2;
        } while (doubling < 20);
        System.out.println(doubling);
    }
}
''',
    ),
    dict(
        id="arrays",
        category="Language",
        title="Arrays, 2D arrays and varargs",
        summary="Array literals, a nested loop over a grid, and a varargs method.",
        main="Grids",
        source='''
import java.util.Arrays;

public class Grids {
    static int total(int... values) {
        int sum = 0;
        for (int value : values) {
            sum += value;
        }
        return sum;
    }

    public static void main(String[] args) {
        int[] row = { 5, 3, 8, 1 };
        Arrays.sort(row);
        System.out.println(Arrays.toString(row));

        int[][] grid = new int[3][3];
        for (int r = 0; r < grid.length; r++) {
            for (int c = 0; c < grid[r].length; c++) {
                grid[r][c] = r * 3 + c;
            }
        }
        System.out.println(Arrays.deepToString(grid));
        System.out.println(total(1, 2, 3) + " " + total());
    }
}
''',
    ),
    dict(
        id="exceptions",
        category="Language",
        title="Exceptions",
        summary="try/catch/finally, a custom exception, and a stack unwind.",
        main="Boom",
        source='''
public class Boom {
    static class TooLoudException extends RuntimeException {
        TooLoudException(String message) { super(message); }
    }

    static void amplify(int volume) {
        if (volume > 11) {
            throw new TooLoudException("volume " + volume + " is too loud");
        }
        System.out.println("playing at " + volume);
    }

    public static void main(String[] args) {
        try {
            amplify(11);
            amplify(12);
        } catch (TooLoudException e) {
            System.out.println("caught: " + e.getMessage());
        } finally {
            System.out.println("finally");
        }

        try {
            int[] small = new int[2];
            small[5] = 1;
        } catch (ArrayIndexOutOfBoundsException e) {
            System.out.println("caught: " + e.getMessage());
        }

        try {
            Object text = "not a number";
            Integer number = (Integer) text;
        } catch (ClassCastException e) {
            System.out.println("caught a ClassCastException");
        }
    }
}
''',
    ),
    dict(
        id="boxing",
        category="Language",
        title="Autoboxing, wrappers and instanceof",
        summary="Primitives boxed into Object keep their identity; a bad cast throws.",
        main="Boxing",
        source='''
import java.util.ArrayList;
import java.util.List;

public class Boxing {
    public static void main(String[] args) {
        List<Object> values = new ArrayList<>();
        values.add(5);
        values.add(2.5);
        values.add("hi");
        values.add(true);
        values.add('x');

        for (Object value : values) {
            System.out.println(value.getClass().getName()
                + " Number=" + (value instanceof Number)
                + " Comparable=" + (value instanceof Comparable));
        }

        Object first = values.get(0);
        System.out.println("unboxed: " + ((Integer) first + 1));
        System.out.println(Integer.parseInt("42") + Integer.valueOf(8));
    }
}
''',
    ),
    dict(
        id="recursion",
        category="Language",
        title="Recursion",
        summary="A recursive factorial and a recursive binary search.",
        main="Recur",
        source='''
public class Recur {
    static long factorial(int n) {
        return n <= 1 ? 1 : n * factorial(n - 1);
    }

    static int search(int[] sorted, int target, int lo, int hi) {
        if (lo > hi) {
            return -1;
        }
        int mid = (lo + hi) / 2;
        if (sorted[mid] == target) {
            return mid;
        }
        return sorted[mid] < target
            ? search(sorted, target, mid + 1, hi)
            : search(sorted, target, lo, mid - 1);
    }

    public static void main(String[] args) {
        System.out.println(factorial(10));
        int[] sorted = { 1, 3, 5, 7, 9, 11 };
        System.out.println(search(sorted, 9, 0, sorted.length - 1));
        System.out.println(search(sorted, 4, 0, sorted.length - 1));
    }
}
''',
    ),
    dict(
        id="enums",
        category="Language",
        title="Enums",
        summary="An enum with a field, a constructor and values().",
        main="Enums",
        source='''
public class Enums {
    enum Planet {
        MERCURY(3.3), EARTH(59.7), JUPITER(1898.0);

        private final double mass;

        Planet(double mass) { this.mass = mass; }

        double getMass() { return mass; }
    }

    public static void main(String[] args) {
        for (Planet planet : Planet.values()) {
            System.out.println(planet + " " + planet.ordinal() + " " + planet.getMass());
        }
        Planet home = Planet.EARTH;
        System.out.println(home == Planet.valueOf("EARTH"));
    }
}
''',
    ),
    # ----- the library -----
    dict(
        id="strings",
        category="Library",
        title="String",
        summary="The everyday String surface, plus String.format.",
        main="Strings",
        source='''
public class Strings {
    public static void main(String[] args) {
        String text = "The quick brown fox";
        System.out.println(text.length() + " " + text.toUpperCase());
        System.out.println(text.substring(4, 9) + "|" + text.indexOf("brown"));
        System.out.println(text.replace("quick", "slow"));
        String[] words = text.split(" ");
        System.out.println(words.length + " words, last=" + words[words.length - 1]);
        System.out.println("  padded  ".trim() + "|");
        System.out.println(String.format("%s scored %d (%.1f%%)", "Ada", 92, 91.75));
        System.out.println("abc".compareTo("abd") + " " + "abc".equals("ABC")
            + " " + "abc".equalsIgnoreCase("ABC"));
    }
}
''',
    ),
    dict(
        id="stringbuilder",
        category="Library",
        title="StringBuilder",
        summary="Building a string in place: append, insert, reverse, delete.",
        main="Builder",
        source='''
public class Builder {
    public static void main(String[] args) {
        StringBuilder sb = new StringBuilder("caturra");
        sb.append(" runs Java");
        sb.insert(0, ">> ");
        System.out.println(sb);
        System.out.println(sb.reverse());
        sb.reverse();
        sb.delete(0, 3);
        sb.setCharAt(0, 'C');
        System.out.println(sb + " (" + sb.length() + ")");
        System.out.println(sb.indexOf("Java"));
    }
}
''',
    ),
    dict(
        id="arraylist",
        category="Collections",
        title="ArrayList",
        summary="The workhorse list: add, get, set, remove, contains, iterate.",
        main="Lists",
        source='''
import java.util.ArrayList;
import java.util.List;

public class Lists {
    public static void main(String[] args) {
        List<String> queue = new ArrayList<>();
        queue.add("ada");
        queue.add("grace");
        queue.add(1, "alan");
        System.out.println(queue + " size=" + queue.size());
        System.out.println(queue.get(0) + " " + queue.contains("grace")
            + " " + queue.indexOf("alan"));
        queue.set(0, "ADA");
        queue.remove("grace");
        for (String name : queue) {
            System.out.println(name);
        }
        System.out.println(queue.isEmpty());
    }
}
''',
    ),
    dict(
        id="hashmap",
        category="Collections",
        title="HashMap",
        summary="Keys to values — and the JDK's own iteration order, not an approximation.",
        main="Maps",
        source='''
import java.util.HashMap;
import java.util.Map;

public class Maps {
    public static void main(String[] args) {
        Map<String, Integer> votes = new HashMap<>();
        votes.put("ada", 3);
        votes.put("grace", 5);
        votes.put("alan", 2);
        votes.put("ada", votes.get("ada") + 1);

        System.out.println(votes);
        System.out.println(votes.containsKey("alan") + " " + votes.getOrDefault("nobody", 0));

        for (Map.Entry<String, Integer> entry : votes.entrySet()) {
            System.out.println(entry.getKey() + " -> " + entry.getValue());
        }
        System.out.println(votes.keySet());
    }
}
''',
    ),
    dict(
        id="sets",
        category="Collections",
        title="HashSet and TreeSet",
        summary="Uniqueness, and a set that keeps itself sorted.",
        main="Sets",
        source='''
import java.util.HashSet;
import java.util.Set;
import java.util.TreeSet;

public class Sets {
    public static void main(String[] args) {
        Set<String> seen = new HashSet<>();
        System.out.println(seen.add("ada") + " " + seen.add("ada"));
        seen.add("grace");
        System.out.println(seen.contains("ada") + " size=" + seen.size());

        TreeSet<Integer> sorted = new TreeSet<>();
        sorted.add(5);
        sorted.add(1);
        sorted.add(3);
        System.out.println(sorted);
        System.out.println(sorted.first() + " " + sorted.last() + " " + sorted.ceiling(2));
    }
}
''',
    ),
    dict(
        id="deques",
        category="Collections",
        title="Stack, Queue and Deque",
        summary="LinkedList as a queue, ArrayDeque, java.util.Stack, and a PriorityQueue.",
        main="Deques",
        source='''
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.LinkedList;
import java.util.PriorityQueue;
import java.util.Queue;
import java.util.Stack;

public class Deques {
    public static void main(String[] args) {
        Queue<String> queue = new LinkedList<>();
        queue.add("first");
        queue.add("second");
        System.out.println(queue.poll() + " then " + queue.peek());

        Deque<Integer> deque = new ArrayDeque<>();
        deque.addFirst(1);
        deque.addLast(2);
        System.out.println(deque.pollFirst() + " " + deque.pollLast());

        Stack<String> stack = new Stack<>();
        stack.push("bottom");
        stack.push("top");
        System.out.println(stack.pop() + " " + stack.peek());

        PriorityQueue<Integer> heap = new PriorityQueue<>();
        heap.add(5);
        heap.add(1);
        heap.add(3);
        System.out.println(heap.poll() + " " + heap.poll() + " " + heap.poll());
    }
}
''',
    ),
    dict(
        id="sorting",
        category="Collections",
        title="Sorting with a Comparator",
        summary="Collections.sort, Comparable, and comparator combinators.",
        main="Sorting",
        source='''
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;

public class Sorting {
    static class Runner implements Comparable<Runner> {
        final String name;
        final int time;

        Runner(String name, int time) {
            this.name = name;
            this.time = time;
        }

        String getName() { return name; }
        @Override public int compareTo(Runner other) { return time - other.time; }
        @Override public String toString() { return name + "(" + time + ")"; }
    }

    public static void main(String[] args) {
        List<Runner> runners = new ArrayList<>();
        runners.add(new Runner("ada", 31));
        runners.add(new Runner("grace", 29));
        runners.add(new Runner("alan", 35));

        Collections.sort(runners);
        System.out.println(runners);

        runners.sort(Comparator.comparing(Runner::getName));
        System.out.println(runners);

        List<Integer> numbers = new ArrayList<>();
        numbers.add(3);
        numbers.add(1);
        numbers.add(2);
        Collections.sort(numbers, Collections.reverseOrder());
        System.out.println(numbers + " max=" + Collections.max(numbers));
    }
}
''',
    ),
    dict(
        id="streams",
        category="Library",
        title="Streams and Optional",
        summary="filter/map/collect, IntStream, and an Optional result.",
        main="Streams",
        source='''
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public class Streams {
    public static void main(String[] args) {
        List<String> names = new ArrayList<>();
        names.add("ada");
        names.add("grace");
        names.add("alan");

        List<String> shouted = names.stream()
            .filter(name -> name.length() > 3)
            .map(name -> name.toUpperCase())
            .collect(Collectors.toList());
        System.out.println(shouted);

        System.out.println(names.stream().count() + " " + names.stream().anyMatch(n -> n.startsWith("a")));
        System.out.println(IntStream.range(1, 6).sum());

        Optional<String> first = names.stream().filter(n -> n.startsWith("g")).findFirst();
        System.out.println(first.isPresent() + " " + first.get());
        System.out.println(names.stream().filter(n -> n.isEmpty()).findFirst().orElse("none"));
    }
}
''',
    ),
    dict(
        id="math",
        category="Library",
        title="Math, Integer and Character",
        summary="Numeric helpers, parsing, and character classification.",
        main="Numbers",
        source='''
public class Numbers {
    public static void main(String[] args) {
        System.out.println(Math.max(3, 7) + " " + Math.abs(-4) + " " + Math.pow(2, 10));
        System.out.println(Math.sqrt(144) + " " + Math.round(2.6) + " " + Math.min(0.5, 0.25));
        System.out.println(Integer.parseInt("123") + 1);
        System.out.println(Integer.MAX_VALUE + " " + Integer.toBinaryString(10));
        System.out.println(Character.isDigit('7') + " " + Character.isLetter('7')
            + " " + Character.toUpperCase('a'));
        System.out.println(Double.parseDouble("2.5") * 2);
        System.out.println(7 / 2 + " " + 7 % 2 + " " + 7.0 / 2);
    }
}
''',
    ),
    dict(
        id="file-io",
        category="Library",
        title="File I/O",
        summary="Write a file with PrintWriter, read it back with Scanner and File.",
        main="Files",
        source='''
import java.io.File;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Scanner;

public class Files {
    public static void main(String[] args) throws IOException {
        PrintWriter writer = new PrintWriter("notes.txt");
        writer.println("first line");
        writer.println("second line");
        writer.close();

        File file = new File("notes.txt");
        System.out.println(file.exists() + " " + file.getName());

        Scanner scanner = new Scanner(file);
        while (scanner.hasNextLine()) {
            System.out.println("read: " + scanner.nextLine());
        }
        scanner.close();
    }
}
''',
    ),
    dict(
        id="reflection",
        category="Library",
        title="Reflection",
        summary="Class, Method and Field — the machinery Code.org's own graders run on.",
        main="Reflect",
        source='''
import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class Reflect {
    static class Student {
        private String name = "ada";
        public int score(int bonus) { return 90 + bonus; }
    }

    public static void main(String[] args) throws Exception {
        Class<?> type = Student.class;
        System.out.println(type.getSimpleName());

        Method score = type.getDeclaredMethod("score", int.class);
        System.out.println(score.getName() + " -> " + score.invoke(new Student(), 5));

        Field name = type.getDeclaredField("name");
        name.setAccessible(true);
        System.out.println(name.getName() + " = " + name.get(new Student()));

        try {
            type.getDeclaredMethod("missing");
        } catch (NoSuchMethodException e) {
            System.out.println("no such method, as Java says");
        }
    }
}
''',
    ),
]

# Real Java 11 that caturra does NOT model. javac must ACCEPT these — that is what
# makes them an honest gap rather than an invented one — and caturra must reject
# them with a reason that says so.
GAPS = [
    dict(
        id="varargs-library",
        category="Library",
        title="Varargs library methods (String.join)",
        summary="A varargs method YOU write works; the library's own varargs (String.join) are not modelled.",
        main="Join",
        source='''
public class Join {
    public static void main(String[] args) {
        String[] parts = { "a", "b", "c" };
        System.out.println(String.join(",", parts));
    }
}
''',
    ),
    dict(
        id="method-ref-stream",
        category="Library",
        title="Method references in a stream",
        summary="A lambda works (`map(s -> s.toUpperCase())`), and a Comparator key extractor must BE a method reference — but `map(String::toUpperCase)` is not modelled.",
        main="MethodRef",
        source='''
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

public class MethodRef {
    public static void main(String[] args) {
        List<String> names = new ArrayList<>();
        names.add("ada");
        List<String> shouted = names.stream()
            .map(String::toUpperCase)
            .collect(Collectors.toList());
        System.out.println(shouted);
    }
}
''',
    ),
    dict(
        id="iterator",
        category="Collections",
        title="Iterator",
        summary="An explicit iterator. The for-each loop it desugars to works; the interface itself is not modelled.",
        main="Iterate",
        source='''
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

public class Iterate {
    public static void main(String[] args) {
        List<String> names = new ArrayList<>();
        names.add("ada");
        Iterator<String> it = names.iterator();
        while (it.hasNext()) {
            System.out.println(it.next());
        }
    }
}
''',
    ),
    dict(
        id="vector",
        category="Collections",
        title="Vector and Hashtable",
        summary="The legacy synchronized collections. ArrayList and HashMap replace them.",
        main="Legacy",
        source='''
import java.util.Vector;

public class Legacy {
    public static void main(String[] args) {
        Vector<String> items = new Vector<>();
        items.add("one");
        System.out.println(items);
    }
}
''',
    ),
    dict(
        id="threads",
        category="Library",
        title="Threads",
        summary="Concurrency. caturra runs a program on one thread, in one WASM instance.",
        main="Threads",
        source='''
public class Threads {
    public static void main(String[] args) throws InterruptedException {
        Thread worker = new Thread(() -> System.out.println("working"));
        worker.start();
        worker.join();
    }
}
''',
    ),
    dict(
        id="buffered-reader",
        category="Library",
        title="BufferedReader",
        summary="The java.io reader stack. Scanner and File cover the same ground.",
        main="Reader",
        source='''
import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;

public class Reader {
    public static void main(String[] args) throws IOException {
        BufferedReader reader = new BufferedReader(new FileReader("notes.txt"));
        System.out.println(reader.readLine());
        reader.close();
    }
}
''',
    ),
    dict(
        id="nio-files",
        category="Library",
        title="java.nio.file",
        summary="Files/Paths, including Java 11's Files.readString.",
        main="Nio",
        source='''
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

public class Nio {
    public static void main(String[] args) throws IOException {
        Path path = Path.of("notes.txt");
        System.out.println(Files.readString(path));
    }
}
''',
    ),
]

# Java that is NEWER than 11. caturra rejects these, and so does javac 11 — which is
# the point: accepting them would mean a program that runs in the playground and
# fails on the JDK the course targets.
BEYOND = [
    dict(
        id="records",
        category="Beyond Java 11",
        title="Records (Java 16)",
        summary="A record is not Java 11. javac 11 rejects it too.",
        main="Rec",
        source='''
public class Rec {
    record Point(int x, int y) {}

    public static void main(String[] args) {
        System.out.println(new Point(1, 2));
    }
}
''',
    ),
    dict(
        id="text-blocks",
        category="Beyond Java 11",
        title="Text blocks (Java 15)",
        summary="Triple-quoted strings are not Java 11. javac 11 rejects them too.",
        main="Block",
        source='''
public class Block {
    public static void main(String[] args) {
        String html = """
            <p>hi</p>
            """;
        System.out.println(html);
    }
}
''',
    ),
    dict(
        id="switch-expressions",
        category="Beyond Java 11",
        title="Switch expressions (Java 14)",
        summary="`case ->` and switch-as-a-value are not Java 11. javac 11 rejects them too.",
        main="Switch",
        source='''
public class Switch {
    public static void main(String[] args) {
        int day = 3;
        String name = switch (day) {
            case 1 -> "monday";
            case 3 -> "wednesday";
            default -> "other";
        };
        System.out.println(name);
    }
}
''',
    ),
]
