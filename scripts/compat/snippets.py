"""The programs behind the compatibility page.

Each one is a whole, runnable Java program that PROVES something: run it on a real
JDK 11 and on caturra and compare. Nothing here is a claim — `compat-record.py`
asks both engines what actually happens and writes the answer into
`features.json`, and `tests/compat_manifest.rs` re-asks on every CI run.

Keep the output small and deterministic (no times, no hashes, no `Math.random`) —
it is compared byte for byte.
"""

FEATURES = [
    dict(
        id="element-types",
        category="Collections",
        title="Any object as a collection element",
        summary="A collection holds any modelled type — a Scanner, a File, a CharSequence — and reads it back as itself.",
        main="Elements",
        source="""
import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Scanner;

public class Elements {
    public static void main(String[] args) {
        List<Scanner> scanners = new ArrayList<>();
        scanners.add(new Scanner("7"));
        System.out.println(scanners.get(0).nextInt());

        List<File> files = new ArrayList<>();
        files.add(new File("notes.txt"));
        System.out.println(files.get(0).getName() + " " + files.size());

        List<CharSequence> texts = new ArrayList<>();
        texts.add("abc");
        texts.add(new StringBuilder("de"));
        System.out.println(texts.get(0).length() + texts.get(1).length());
    }
}
""",
    ),
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
        id="nested-containers",
        category="Collections",
        title="A collection inside another type's argument",
        summary="An Optional, a Map.Entry or a Stream holds a whole collection — and reads it back as one.",
        main="Nested",
        source="""
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

public class Nested {
    public static void main(String[] args) {
        Optional<List<String>> names = Optional.of(new ArrayList<>(List.of("ada")));
        names.get().add("grace");
        System.out.println(names.get().size() + " " + names.get().get(1));
        System.out.println(names.map(List::size).get());

        Map.Entry<String, List<String>> entry = Map.entry("team", names.get());
        System.out.println(entry.getKey() + " " + entry.getValue().get(0));

        Map<String, List<String>> byTeam = new HashMap<>();
        byTeam.put("red", new ArrayList<>(List.of("alan")));
        System.out.println(byTeam.get("red").get(0).length());

        Optional<int[]> scores = Optional.of(new int[] {3, 4});
        System.out.println(scores.get().length + scores.get()[1]);
    }
}
""",
    ),
    dict(
        id="stream-gathering",
        category="Library",
        title="Gathering a stream into any collection",
        summary="toCollection builds the container you name; flatMap flattens a stream of collections; a stream hands itself to a loop.",
        main="Gather",
        source="""
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.TreeSet;
import java.util.stream.Collectors;

public class Gather {
    public static void main(String[] args) {
        List<String> names = new ArrayList<>(List.of("grace", "ada", "alan"));

        TreeSet<String> sorted = names.stream().collect(Collectors.toCollection(TreeSet::new));
        System.out.println(sorted.first() + " " + sorted.last());

        List<List<String>> teams = new ArrayList<>();
        teams.add(new ArrayList<>(List.of("ada")));
        teams.add(new ArrayList<>(List.of("alan", "grace")));
        System.out.println(teams.stream().flatMap(List::stream).collect(Collectors.toList()));

        Iterator<String> cursor = names.stream().sorted().iterator();
        while (cursor.hasNext()) {
            System.out.print(cursor.next().charAt(0));
        }
        System.out.println();

        String[] copy = names.toArray(String[]::new);
        System.out.println(copy.length + copy[0]);
    }
}
""",
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
        id="charsets",
        category="Library",
        title="Text to bytes and back",
        summary="getBytes and new String(bytes), in the charset the program names.",
        main="Bytes",
        source='''
import java.nio.charset.Charset;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;

public class Bytes {
    public static void main(String[] args) {
        String text = "hi";
        System.out.println(Arrays.toString(text.getBytes()));
        System.out.println(Arrays.toString(text.getBytes(StandardCharsets.UTF_8)));
        System.out.println(new String(text.getBytes(), StandardCharsets.UTF_8));
        System.out.println(new String(new byte[] {104, 101, 108, 108, 111}));
        Charset charset = Charset.forName("US-ASCII");
        System.out.println(charset + " " + charset.name());
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
    dict(
        id="iterator",
        category="Collections",
        title="Iterator",
        summary="An explicit iterator over a list or set, including remove() while iterating.",
        main="Iterate",
        source='''
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeSet;

public class Iterate {
    public static void main(String[] args) {
        List<String> names = new ArrayList<>();
        names.add("ada");
        names.add("grace");
        names.add("alan");

        // remove() takes the last element next() returned out of the collection.
        Iterator<String> it = names.iterator();
        while (it.hasNext()) {
            if (it.next().equals("grace")) {
                it.remove();
            }
        }
        System.out.println(names);

        Set<Integer> scores = new TreeSet<>();
        scores.add(3);
        scores.add(1);
        scores.add(2);
        Iterator<Integer> nums = scores.iterator();
        int sum = 0;
        while (nums.hasNext()) {
            sum += nums.next();
        }
        System.out.println("sum=" + sum);

        // entrySet iteration, via var for brevity (the explicit
        // Iterator<Map.Entry<K, V>> declaration also works).
        Map<String, Integer> ages = new TreeMap<>();
        ages.put("ada", 36);
        ages.put("bea", 41);
        var entries = ages.entrySet().iterator();
        while (entries.hasNext()) {
            var e = entries.next();
            System.out.println(e.getKey() + " is " + e.getValue());
        }
    }
}
''',
    ),
    dict(
        id="method-ref-stream",
        category="Library",
        title="Method references in a stream",
        summary="A method reference stands in for a stream lambda: `map(String::toUpperCase)`, `mapToInt(String::length)`, `filter(String::isEmpty)`. Each desugars to the one-parameter lambda it denotes.",
        main="MethodRef",
        source='''
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

public class MethodRef {
    public static void main(String[] args) {
        List<String> names = new ArrayList<>();
        names.add("ada");
        names.add("grace");
        List<String> shouted = names.stream()
            .map(String::toUpperCase)
            .collect(Collectors.toList());
        System.out.println(shouted);
        System.out.println(names.stream().mapToInt(String::length).sum());
    }
}
''',
    ),
    dict(
        id="varargs-library",
        category="Library",
        title="Varargs library methods (String.join)",
        summary="`String.join` — a String array, a List, or individual strings. Modelled like `String.format`: the argument shape rides in the descriptor.",
        main="Join",
        source='''
import java.util.Arrays;
import java.util.List;

public class Join {
    public static void main(String[] args) {
        String[] parts = { "a", "b", "c" };
        System.out.println(String.join(",", parts));
        System.out.println(String.join("-", "x", "y", "z"));
        List<String> words = Arrays.asList("one", "two", "three");
        System.out.println(String.join(" ", words));
    }
}
''',
    ),
    dict(
        id="buffered-reader",
        category="Library",
        title="BufferedReader",
        summary="`new BufferedReader(new FileReader(path))` and the `while ((line = reader.readLine()) != null)` read loop — the java.io reader stack, over a file or `System.in`.",
        main="Reader",
        source='''
import java.io.BufferedReader;
import java.io.FileReader;
import java.io.PrintWriter;
import java.io.IOException;

public class Reader {
    public static void main(String[] args) throws IOException {
        PrintWriter writer = new PrintWriter("notes.txt");
        writer.println("first line");
        writer.println("second line");
        writer.close();

        BufferedReader reader = new BufferedReader(new FileReader("notes.txt"));
        String line;
        while ((line = reader.readLine()) != null) {
            System.out.println(line);
        }
        reader.close();
    }
}
''',
    ),
    dict(
        id="nio-files",
        category="Library",
        title="java.nio.file",
        summary="`Path.of`, `Files.writeString`/`readString`/`readAllLines` — the java.nio.file slice, including Java 11's string helpers.",
        main="Nio",
        source='''
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

public class Nio {
    public static void main(String[] args) throws IOException {
        Path path = Path.of("notes.txt");
        Files.writeString(path, "first line\\nsecond line\\n");
        System.out.println(Files.readString(path));
        System.out.println(Files.readAllLines(path));
    }
}
''',
    ),
    dict(
        id="iterable-type",
        category="Collections",
        title="Iterable as a type",
        summary="Hold any collection by the interface a for-each actually uses — and implement it yourself.",
        main="Iterables",
        source="""
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Set;
import java.util.TreeSet;

public class Iterables {
    static class Bag<T> implements Iterable<T> {
        private final List<T> items = new ArrayList<>();
        void add(T item) { items.add(item); }
        public Iterator<T> iterator() { return items.iterator(); }
    }

    static <T> int count(Iterable<T> things) {
        int n = 0;
        for (T thing : things) {
            n++;
        }
        return n;
    }

    public static void main(String[] args) {
        List<String> list = new ArrayList<>(List.of("b", "a"));
        Set<String> set = new TreeSet<>(list);

        Iterable<String> held = list;
        for (String s : held) {
            System.out.print(s);
        }
        System.out.println();
        System.out.println(held.iterator().next().toUpperCase());

        for (String s : (Iterable<String>) list) {
            System.out.print(s);
        }
        System.out.println();

        Bag<String> bag = new Bag<>();
        bag.add("q");
        Iterable<String> asIterable = bag;
        for (String s : asIterable) {
            System.out.println(s);
        }

        System.out.println(count(list) + " " + count(set) + " " + count(bag));
    }
}
""",
    ),
    dict(
        id="object-contract",
        category="Language",
        title="What every object inherits",
        summary="getClass, equals and hashCode on every reference — and which classes compare by VALUE.",
        main="ObjectContract",
        source="""
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.PriorityQueue;

public class ObjectContract {
    public static void main(String[] args) {
        // Value-based: two of these are equal when their contents are.
        java.io.File one = new java.io.File("dir/f.txt");
        java.io.File same = new java.io.File("dir/f.txt");
        System.out.println(one.equals(same) + " " + (one.hashCode() == same.hashCode()));
        System.out.println(one.getClass().getName());

        Optional<String> present = Optional.of("x");
        System.out.println(present.equals(Optional.of("x")));
        System.out.println(present.hashCode() == "x".hashCode());
        System.out.println(Optional.empty().equals(Optional.empty()));

        List<String> list = new ArrayList<>(List.of("a"));
        System.out.println(list.equals(new ArrayList<>(List.of("a"))) + " " + list.hashCode());
        Map<String, Integer> map = new HashMap<>();
        map.put("k", 1);
        System.out.println(map.equals(new HashMap<>(map)));

        // Identity: these override neither, so only the object itself is equal.
        StringBuilder builder = new StringBuilder("a");
        System.out.println(builder.equals(new StringBuilder("a")) + " " + builder.equals(builder));
        PriorityQueue<String> queue = new PriorityQueue<>();
        System.out.println(queue.equals(new PriorityQueue<String>()));
        RuntimeException failure = new IllegalStateException("z");
        System.out.println(failure.equals(new IllegalStateException("z")));

        System.out.println(list.getClass().getName() + " " + builder.getClass().getName());
    }
}
""",
    ),
    dict(
        id="class-literals",
        category="Types and literals",
        title="Class literals and what a class knows",
        summary="int[].class and String[].class, and the four names a class answers to.",
        main="ClassLiterals",
        source="""
public class ClassLiterals {
    static class Nested { }

    public static void main(String[] args) {
        System.out.println(String.class + " " + String.class.getSimpleName());
        System.out.println(String[].class);
        System.out.println(String[][].class.getName());
        System.out.println(int[].class + " " + int[].class.getTypeName());
        System.out.println(String[].class.getComponentType());
        System.out.println(String[].class == new String[0].getClass());

        System.out.println(Nested.class.getName());
        System.out.println(Nested.class.getCanonicalName());
        System.out.println(Nested.class.getEnclosingClass());
        System.out.println(String.class.getEnclosingClass());
        System.out.println(Nested.class.isAnonymousClass() + " " + int[].class.isArray());
    }
}
""",
    ),
    dict(
        id="qualified-names",
        category="Declarations",
        title="Fully qualified names, without an import",
        summary="A qualified name needs no import — that is what it is for.",
        main="Qualified",
        source="""
public class Qualified {
    public static void main(String[] args) {
        java.util.List<String> list = new java.util.ArrayList<>();
        list.add("b");
        list.add("a");
        java.util.Collections.sort(list);
        System.out.println(list);

        java.util.stream.Stream<String> stream = list.stream();
        System.out.println(stream.collect(java.util.stream.Collectors.joining("-")));
        System.out.println(java.util.stream.IntStream.range(0, 4).sum());

        java.util.Map<String, Integer> counts = new java.util.HashMap<>();
        counts.put("a", 1);
        for (java.util.Map.Entry<String, Integer> entry : counts.entrySet()) {
            System.out.println(entry.getKey() + "=" + entry.getValue());
        }

        System.out.println(java.lang.Math.abs(-2));
        java.util.Scanner scanner = new java.util.Scanner("7");
        System.out.println(scanner.nextInt());
    }
}
""",
    ),
    dict(
        id="stack-traces",
        category="Library",
        title="Reading a stack trace",
        summary="getStackTrace, the frames it hands back, and fillInStackTrace.",
        main="Traces",
        source="""
public class Traces {
    static Throwable deep() {
        return new RuntimeException("x");
    }

    public static void main(String[] args) {
        Throwable failure = deep();
        StackTraceElement[] frames = failure.getStackTrace();
        System.out.println(frames.length > 0);
        System.out.println(frames[0].getMethodName() + " in " + frames[0].getClassName());
        System.out.println(frames[0].getFileName() + " " + (frames[0].getLineNumber() > 0));
        System.out.println(frames[0].equals(frames[0]));

        Throwable same = failure.fillInStackTrace();
        System.out.println(same == failure);
        System.out.println(failure.getStackTrace()[0].getMethodName());

        try {
            throw new IllegalStateException("boom", failure);
        } catch (IllegalStateException caught) {
            System.out.println(caught.getMessage() + " <- " + caught.getCause().getMessage());
        }
    }
}
""",
    ),
    dict(
        id="functional-runnable",
        category="Language",
        title="Runnable and Function",
        summary="The no-argument callback every example uses, and Function.identity().",
        main="Callbacks",
        source="""
import java.util.function.Function;
import java.util.function.UnaryOperator;

public class Callbacks {
    static void twice(Runnable job) {
        job.run();
        job.run();
    }

    public static void main(String[] args) {
        Runnable lambda = () -> System.out.println("ran");
        lambda.run();
        twice(() -> System.out.println("again"));

        Runnable anonymous = new Runnable() {
            public void run() { System.out.println("anon"); }
        };
        anonymous.run();

        int[] count = {0};
        Runnable bump = () -> count[0]++;
        bump.run();
        bump.run();
        System.out.println(count[0]);

        Function<String, String> same = Function.identity();
        System.out.println(same.apply("z"));
        UnaryOperator<String> shout = s -> s.toUpperCase();
        System.out.println(shout.apply("hi"));
    }
}
""",
    ),
    dict(
        id="factory-streams",
        category="Collections",
        title="List.of as a stream source",
        summary="The immutable factories, straight into a pipeline and a collector.",
        main="Factories",
        source="""
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;

public class Factories {
    public static void main(String[] args) {
        System.out.println(List.of("a", "bb").stream()
            .map(String::toUpperCase).collect(Collectors.toList()));
        System.out.println(List.of("a", "bb").stream()
            .collect(Collectors.toMap(x -> x, String::length)));
        System.out.println(List.of("a", "bb").stream()
            .collect(Collectors.groupingBy(String::length)));
        System.out.println(Arrays.asList("a", "bb").stream()
            .filter(s -> s.length() > 1).count());
        System.out.println(List.of(3, 1, 2).stream()
            .sorted().map(n -> n * 2).collect(Collectors.toList()));
        System.out.println(List.of("x", "y").stream().collect(Collectors.joining("-")));
        System.out.println(Arrays.asList(5, 6).stream().mapToInt(n -> n).sum());
        System.out.println(List.of("a", "bb").get(0).toUpperCase());
    }
}
""",
    ),
    dict(
        id="map-entry",
        category="Collections",
        title="Map.Entry as a value",
        summary="An entry from a map, one built by Map.entry, and a standalone SimpleEntry.",
        main="Entries",
        source="""
import java.util.AbstractMap;
import java.util.HashMap;
import java.util.Map;

public class Entries {
    public static void main(String[] args) {
        Map<String, Integer> counts = new HashMap<>();
        counts.put("a", 1);
        counts.put("b", 2);

        Map.Entry<String, Integer> first = counts.entrySet().iterator().next();
        System.out.println(first + " " + first.getKey() + " " + first.getValue());

        Map.Entry<String, Integer> made = Map.entry("k", 1);
        System.out.println(made + " " + made.getKey());
        System.out.println(made.equals(Map.entry("k", 1)));
        System.out.println(made.equals(new AbstractMap.SimpleEntry<>("k", 1)));

        int total = 0;
        for (Map.Entry<String, Integer> entry : counts.entrySet()) {
            total += entry.getValue();
        }
        System.out.println(total);

        var standalone = new AbstractMap.SimpleEntry<>("c", 3);
        System.out.println(standalone.getKey() + "=" + (standalone.getValue() + 1));
    }
}
""",
    ),
    dict(
        id="lambda-after-map",
        category="Collections",
        title="A lambda after map()",
        summary=(
            "A stream's element type survives `map`: the lambda's body is typed, "
            "so a later filter, map or collect sees its parameter as what the "
            "map produced rather than as Object."
        ),
        main="AfterMap",
        source="""
import java.util.List;
import java.util.stream.Collectors;

public class AfterMap {
    public static void main(String[] args) {
        List<String> words = List.of("a", "bb");
        System.out.println(words.stream()
            .map(String::toUpperCase)
            .filter(w -> w.length() > 1)
            .count());
        List<Integer> lengths = words.stream()
            .map(w -> w.length())
            .collect(Collectors.toList());
        System.out.println(lengths);
        int first = words.stream().map(w -> w.length()).findFirst().get();
        System.out.println(first + 1);
    }
}
""",
    ),
    dict(
        id="sublist-view",
        category="Collections",
        title="subList as a live view",
        summary=(
            "A window onto the list itself: writing through it writes through "
            "to the original, clearing it removes the range, and changing the "
            "list around the view invalidates it."
        ),
        main="SubList",
        source="""
import java.util.ArrayList;
import java.util.ConcurrentModificationException;
import java.util.List;

public class SubList {
    public static void main(String[] args) {
        List<String> letters = new ArrayList<>(List.of("a", "b", "c", "d"));
        List<String> middle = letters.subList(1, 3);
        middle.set(0, "B");
        System.out.println(letters);
        System.out.println(middle);

        letters.set(2, "C");
        System.out.println(middle);

        middle.clear();
        System.out.println(letters);

        List<Integer> numbers = new ArrayList<>(List.of(0, 1, 2, 3, 4));
        List<Integer> view = numbers.subList(1, 4);
        numbers.add(5);
        try {
            System.out.println(view);
        } catch (ConcurrentModificationException e) {
            System.out.println("the view noticed");
        }
    }
}
""",
    ),
    dict(
        id="sorted-views",
        category="Collections",
        title="Range and descending views of a sorted collection",
        summary=(
            "headSet, subMap, descendingSet and their kin are live windows onto "
            "the tree: a key added inside the range shows through, and clearing "
            "the view deletes that range from the map."
        ),
        main="SortedViews",
        source="""
import java.util.NavigableMap;
import java.util.NavigableSet;
import java.util.TreeMap;
import java.util.TreeSet;

public class SortedViews {
    public static void main(String[] args) {
        TreeSet<Integer> numbers = new TreeSet<>();
        for (int n : new int[] { 1, 3, 5, 7, 9 }) {
            numbers.add(n);
        }
        System.out.println(numbers.headSet(5) + " " + numbers.tailSet(5));
        System.out.println(numbers.subSet(3, true, 7, true));
        System.out.println(numbers.descendingSet());

        NavigableSet<Integer> upper = numbers.tailSet(5, false);
        numbers.add(6);
        System.out.println(upper + " " + upper.first());

        TreeMap<String, Integer> scores = new TreeMap<>();
        scores.put("ana", 3);
        scores.put("bo", 1);
        scores.put("cy", 4);
        System.out.println(scores.headMap("cy") + " " + scores.descendingMap());
        System.out.println(scores.firstEntry() + " " + scores.ceilingKey("b"));

        NavigableMap<String, Integer> tail = scores.tailMap("bo", true);
        tail.clear();
        System.out.println(scores);
    }
}
""",
    ),
    dict(
        id="string-joiner",
        category="Library",
        title="StringJoiner",
        summary=(
            "A delimiter, a prefix and a suffix, with an empty value of its own "
            "and merge() splicing another joiner's elements in."
        ),
        main="Joining",
        source="""
import java.util.StringJoiner;

public class Joining {
    public static void main(String[] args) {
        StringJoiner list = new StringJoiner(", ", "[", "]");
        list.add("ana").add("bo").add("cy");
        System.out.println(list + " length=" + list.length());

        StringJoiner empty = new StringJoiner(", ", "[", "]").setEmptyValue("none");
        System.out.println(empty);

        StringJoiner left = new StringJoiner("+", "(", ")");
        left.add("1");
        StringJoiner right = new StringJoiner("-", "{", "}");
        right.add("x").add("y");
        System.out.println(left.merge(right));
    }
}
""",
    ),
    dict(
        id="raw-types",
        category="Language",
        title="Raw types and the unchecked conversion",
        summary=(
            "A collection written without type arguments, and the conversion "
            "between it and a parameterized one that javac only warns about."
        ),
        main="RawTypes",
        source="""
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

public class RawTypes {
    public static void main(String[] args) {
        List raw = new ArrayList();
        raw.add("a");
        raw.add(1);
        System.out.println(raw + " " + raw.size());

        List<String> typed = raw;
        System.out.println(typed.get(0).toUpperCase());

        List<String> words = new ArrayList<>();
        words.add("hello");
        List back = words;
        back.add("also raw");
        System.out.println(words);

        Iterator cursor = raw.iterator();
        System.out.println(cursor.next());
    }
}
""",
    ),
    dict(
        id="iface-super",
        category="Language",
        title="Interface.super.method()",
        summary=(
            "A class that inherits two default methods picks one by name — the "
            "only way to call a default a class has overridden."
        ),
        main="Defaults",
        source="""
public class Defaults {
    interface Greeter {
        default String greet() { return "hello"; }
    }

    interface Shouter {
        default String greet() { return "HELLO"; }
    }

    static class Polite implements Greeter, Shouter {
        @Override
        public String greet() {
            return Greeter.super.greet() + " / " + Shouter.super.greet();
        }
    }

    public static void main(String[] args) {
        System.out.println(new Polite().greet());
    }
}
""",
    ),
    dict(
        id="file-paths",
        category="Library",
        title="A file's path, and a directory's contents",
        summary="getParent, getAbsolutePath and getCanonicalPath, mkdirs beside mkdir, list and listFiles, renameTo — the half of java.io.File that is not reading and writing.",
        main="Paths",
        source="""
import java.io.File;
import java.io.IOException;
import java.util.Arrays;

public class Paths {
    public static void main(String[] args) throws IOException {
        File nested = new File("shelf/deep");
        System.out.println(nested.mkdirs() + " " + nested.isDirectory());
        // mkdir makes ONE directory: with the parent missing it answers false.
        System.out.println(new File("gone/deeper").mkdir());

        new File("shelf/a.txt").createNewFile();
        new File("shelf/b.txt").createNewFile();
        File shelf = new File("shelf");
        String[] names = shelf.list();
        Arrays.sort(names);
        System.out.println(Arrays.toString(names));
        File[] files = shelf.listFiles();
        Arrays.sort(files);
        System.out.println(files[0].getPath() + " " + files[0].getName());

        File one = new File("shelf/a.txt");
        System.out.println(one.getParent() + " " + one.getParentFile().getName());
        System.out.println(new File("a.txt").getParent());
        System.out.println(one.isAbsolute() + " " + one.getAbsolutePath().endsWith("/shelf/a.txt"));
        System.out.println(new File("./shelf/../shelf/a.txt").getCanonicalPath()
                .equals(one.getAbsolutePath()));
        System.out.println(one.compareTo(new File("shelf/b.txt")) < 0);

        System.out.println(one.renameTo(new File("shelf/moved.txt")));
        System.out.println(new File("shelf/moved.txt").exists() + " " + one.exists());
        System.out.println(new File("ghost").list());

        // Tidy up, so running this a second time starts where the first did:
        // the page keeps one filesystem for the whole visit, and `mkdirs` on a
        // directory that is already there answers false.
        new File("shelf/moved.txt").delete();
        new File("shelf/b.txt").delete();
        nested.delete();
        shelf.delete();
        System.out.println(shelf.exists());
    }
}
""",
    ),
    dict(
        id="collector-downstream",
        category="Collections",
        title="Collectors that wrap another collector",
        summary="filtering, flatMapping, mapping and collectingAndThen hand elements to the collector below; maxBy/minBy and reducing gather without one. Under a groupingBy, a group that keeps nothing still exists.",
        main="Gathering",
        source="""
import java.util.*;
import java.util.stream.*;

public class Gathering {
    public static void main(String[] args) {
        List<String> words = Arrays.asList("pear", "fig", "apple", "kiwi", "fig");

        System.out.println(words.stream().collect(
                Collectors.maxBy(Comparator.comparingInt(String::length))));
        System.out.println(words.stream().collect(Collectors.reducing("", w -> w.substring(0, 1),
                String::concat)));
        int size = words.stream().collect(
                Collectors.collectingAndThen(Collectors.toList(), List::size));
        System.out.println(size);

        Map<Integer, List<String>> byLength = words.stream().collect(
                Collectors.groupingBy(String::length,
                        Collectors.filtering(w -> w.startsWith("f"), Collectors.toList())));
        System.out.println(new TreeMap<>(byLength));

        Map<Integer, Long> counts = words.stream().collect(
                Collectors.groupingBy(String::length, Collectors.counting()));
        System.out.println(new TreeMap<>(counts));
        System.out.println(words.stream().collect(Collectors.flatMapping(
                w -> Stream.of(w.charAt(0)), Collectors.toSet())).size());
        List<String> gathered = words.stream()
                .collect(ArrayList::new, ArrayList::add, ArrayList::addAll);
        System.out.println(gathered);
    }
}
""",
    ),
    dict(
        id="summary-statistics",
        category="Collections",
        title="IntSummaryStatistics and its siblings",
        summary="One pass gathers count, sum, min, max and average. The three classes differ in the width of what they report and in the identity values an empty summary keeps.",
        main="Summaries",
        source="""
import java.util.*;
import java.util.stream.*;

public class Summaries {
    public static void main(String[] args) {
        List<String> words = Arrays.asList("pear", "fig", "apple");

        IntSummaryStatistics lengths = words.stream()
                .collect(Collectors.summarizingInt(String::length));
        System.out.println(lengths);
        System.out.println(lengths.getCount() + " " + lengths.getSum() + " "
                + lengths.getMin() + " " + lengths.getMax() + " " + lengths.getAverage());

        System.out.println(words.stream().collect(Collectors.summarizingLong(String::length)));
        System.out.println(words.stream().collect(Collectors.summarizingDouble(String::length)));

        System.out.println(IntStream.rangeClosed(1, 4).summaryStatistics());
        System.out.println(DoubleStream.of(1.5, 2.5).summaryStatistics());
        System.out.println(LongStream.of(10L, 20L).summaryStatistics());

        // An EMPTY summary keeps the identity values its accumulator started from.
        System.out.println(IntStream.of().summaryStatistics());
        System.out.println(DoubleStream.of().summaryStatistics());
    }
}
""",
    ),
    dict(
        id="parallel-streams",
        category="Collections",
        title="parallelStream, on one thread",
        summary="caturra runs a single thread, and a JDK is allowed to answer a sequential stream from parallelStream() — so the pipeline is the same one, and isParallel() reports what a JDK reports.",
        main="Parallel",
        source="""
import java.util.*;
import java.util.stream.*;

public class Parallel {
    public static void main(String[] args) {
        List<String> words = List.of("pear", "fig", "apple");
        System.out.println(words.parallelStream().map(String::length)
                .collect(Collectors.toList()));
        System.out.println(words.parallelStream().isParallel() + " "
                + words.stream().isParallel());
        System.out.println(words.stream().parallel().isParallel() + " "
                + words.parallelStream().sequential().isParallel());
        System.out.println(words.parallelStream().map(String::length).isParallel());
        System.out.println(new TreeSet<>(words).parallelStream()
                .collect(Collectors.joining("-")));
        System.out.println(Map.of("k", 1).values().parallelStream().count());
        System.out.println(IntStream.of(1, 2, 3).parallel().sum());
    }
}
""",
    ),
    dict(
        id="optional-arms",
        category="Library",
        title="Optional's two-armed forms",
        summary="ifPresentOrElse runs one arm or the other; or() answers this Optional, or the one a supplier makes. Java 9 additions, and the only place a Runnable is a functional parameter here.",
        main="Arms",
        source="""
import java.util.*;

public class Arms {
    public static void main(String[] args) {
        List<String> words = Arrays.asList("pear", "fig", "kiwi");
        Optional<String> found = words.stream().filter(w -> w.startsWith("k")).findFirst();
        found.ifPresentOrElse(w -> System.out.println("found " + w),
                () -> System.out.println("none"));
        Optional.<String>empty().ifPresentOrElse(w -> System.out.println("found " + w),
                () -> System.out.println("none"));

        System.out.println(found.or(() -> Optional.of("fallback")).get());
        System.out.println(Optional.<String>empty().or(() -> Optional.of("fallback")).get());
        System.out.println(Optional.<String>empty().orElse("x").length());
        System.out.println(Optional.of("kept").or(() -> Optional.of("other")));
    }
}
""",
    ),
    dict(
        id="collection-bulk",
        category="Collections",
        title="The bulk operations every collection has",
        summary="containsAll, removeAll and retainAll on a deque and a queue as much as on a list; sort and replaceAll on a LinkedList; a priority queue's own comparator.",
        main="Bulk",
        source="""
import java.util.*;

public class Bulk {
    public static void main(String[] args) {
        ArrayDeque<String> deque = new ArrayDeque<>(Arrays.asList("a", "b", "c"));
        System.out.println(deque.containsAll(Arrays.asList("a", "c")));
        System.out.println(deque.removeAll(Arrays.asList("b")) + " " + deque);
        System.out.println(deque.retainAll(Arrays.asList("a")) + " " + deque);

        LinkedList<String> list = new LinkedList<>(Arrays.asList("pear", "fig", "apple"));
        list.sort(Comparator.naturalOrder());
        list.replaceAll(String::toUpperCase);
        System.out.println(list);
        System.out.println(list.removeAll(Arrays.asList("FIG")) + " " + list);

        PriorityQueue<String> queue = new PriorityQueue<>(Comparator.reverseOrder());
        queue.addAll(Arrays.asList("a", "b", "c"));
        System.out.println((queue.comparator() != null) + " "
                + queue.containsAll(Arrays.asList("a", "b")));
        System.out.println(new PriorityQueue<String>().comparator());
    }
}
""",
    ),
]

# Real Java 11 that caturra does NOT model. javac must ACCEPT these — that is what
# makes them an honest gap rather than an invented one — and caturra must reject
# them with a reason that says so.
GAPS = [
    dict(
        id="vector",
        category="Collections",
        title="Vector, Hashtable, EnumMap, EnumSet",
        summary=(
            "The legacy synchronized collections, and the enum-keyed ones. "
            "ArrayList and HashMap replace the first two; a TreeMap or TreeSet "
            "keyed by the enum iterates in the very same order as the others, "
            "an enum's natural ordering being its ordinal."
        ),
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
        id="class-modifiers",
        category="Library",
        title="Class.getModifiers",
        summary="A class's access flags. caturra answers them for a field, a method and a constructor — not for a class.",
        main="Modifiers",
        source="""
import java.lang.reflect.Modifier;

public class Modifiers {
    public static void main(String[] args) {
        System.out.println(Modifier.toString(String.class.getModifiers()));
    }
}
""",
    ),
    dict(
        id="scanner-text",
        category="Library",
        title="A Scanner's own toString",
        summary="The JDK prints its delimiters, position and locale separators — internal state caturra does not model.",
        main="ScannerText",
        source="""
import java.util.Scanner;

public class ScannerText {
    public static void main(String[] args) {
        Scanner scanner = new Scanner("7 x");
        System.out.println(scanner.toString().startsWith("java.util.Scanner"));
    }
}
""",
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


def _prog(body, members="", imports=""):
    """A whole program around a statement body."""
    return f"""{imports}
public class G {{
{members}
    public static void main(String[] args) throws Exception {{
{body}
    }}
}}
"""


# The GRAMMAR, walked construct by construct (JLS ch. 4, 6-10, 14, 15) rather than
# cherry-picked. This is what answers "how much of the LANGUAGE is there", which is
# a different question from "which features did someone choose to demo" — and the
# answer is only worth having if it is measured. Statuses come from the engines.
GRAMMAR = [
    # ----- types, literals -----
    dict(id="g-primitives", category="Types and literals", title="The eight primitive types",
         summary="byte, short, int, long, float, double, char, boolean.", main="G",
         source=_prog('byte b = 1; short s = 2; int i = 3; long l = 4L; float f = 5.0f;\n'
                      '        double d = 6.5; char c = \'x\'; boolean z = true;\n'
                      '        System.out.println(b + " " + s + " " + i + " " + l + " " + f + " " + d + " " + c + " " + z);')),
    dict(id="g-literals", category="Types and literals", title="Number literals: hex, octal, binary, underscores",
         summary="0xFF, 010, 0b1010, 1_000_000, and the L/f/e suffixes.", main="G",
         source=_prog('int hex = 0xFF; int oct = 010; int bin = 0b1010; int big = 1_000_000;\n'
                      '        long l = 10L; float f = 1.5f; double sci = 1.5e3;\n'
                      '        System.out.println(hex + " " + oct + " " + bin + " " + big + " " + l + " " + f + " " + sci);')),
    dict(id="g-char-escapes", category="Types and literals", title="Character escapes and \\u literals",
         summary="\\t, \\n, \\\\, and a unicode escape.", main="G",
         source=_prog("char tab = '\\t'; char u = '\\u0041';\n"
                      "        System.out.println(\"[\" + tab + \"]\" + u + \"\\\\\");")),
    dict(id="g-arrays", category="Types and literals", title="Arrays: jagged, covariant, C-style brackets",
         summary="new int[2][], an Object[] holding a String[], and `int a[]`.", main="G",
         source=_prog('int[][] jagged = new int[2][];\n        jagged[0] = new int[] { 1, 2 };\n'
                      '        jagged[1] = new int[] { 3 };\n'
                      '        Object[] objects = new String[] { "a" };\n'
                      '        int old[] = { 7 };\n'
                      '        System.out.println(jagged[0][1] + " " + jagged[1][0] + " " + objects[0] + " " + old[0]);')),
    dict(id="g-var", category="Types and literals", title="var (Java 10 local type inference)",
         summary="`var count = 3;` — the initializer's type, inferred.", main="G",
         source=_prog('var count = 3;\n        var name = "ada";\n        System.out.println(count + name);')),

    # ----- declarations -----
    dict(id="g-nested-class", category="Declarations", title="Static nested class",
         summary="A class inside a class.", main="G",
         source=_prog('System.out.println(new Inner().hi());',
                      '    static class Inner { String hi() { return "inner"; } }')),
    dict(id="g-inner-class", category="Declarations", title="Inner (non-static) class",
         summary="`outer.new Inner()` — an instance bound to an enclosing one, reading the enclosing instance's fields by simple name.", main="G",
         source=_prog('G outer = new G();\n        G.Inner inner = outer.new Inner(5);\n'
                      '        System.out.println(inner.total());',
                      '    int base = 10;\n'
                      '    class Inner {\n        int bonus;\n        Inner(int b) { bonus = b; }\n'
                      '        int total() { return base + bonus; }\n    }')),
    dict(id="g-local-class", category="Declarations", title="Local class (declared inside a method)",
         summary="A named class in a method body — captures an effectively-final local and reads the enclosing statics.", main="G",
         source=_prog('int factor = 3;\n'
                      '        class Scaler { int of(int n) { return n * factor; } }\n'
                      '        System.out.println(new Scaler().of(4));')),
    dict(id="g-init-blocks", category="Declarations", title="Static and instance initializer blocks",
         summary="`static { ... }` and `{ ... }`.", main="G",
         source=_prog('System.out.println(VALUE + " " + new G().value);',
                      '    static int VALUE;\n    static { VALUE = 7; }\n'
                      '    int value;\n    { value = 9; }')),
    dict(id="g-ctor-chain", category="Declarations", title="Constructor overloading and this(...)",
         summary="One constructor delegating to another.", main="G",
         source=_prog('System.out.println(new G(2).value + " " + new G().value);',
                      '    int value;\n    G() { this(1); }\n    G(int v) { value = v; }')),
    dict(id="g-iface-static", category="Declarations", title="Interface with a static method",
         summary="`interface H { static ... }` (Java 8).", main="G",
         source=_prog('System.out.println(Helper.of());',
                      '    interface Helper { static String of() { return "static"; } }')),
    dict(id="g-iface-private", category="Declarations", title="Interface with a private method (Java 9)",
         summary="A private helper behind a default method.", main="G",
         source=_prog('System.out.println(new H() {}.pub());',
                      '    interface H {\n        private String secret() { return "priv"; }\n'
                      '        default String pub() { return secret(); }\n    }')),
    dict(id="g-enum-body", category="Declarations", title="Enum with a body per constant",
         summary="Constant-specific class bodies implementing an abstract method.", main="G",
         source=_prog('System.out.println(Op.PLUS.apply(2, 3));',
                      '    enum Op {\n        PLUS { int apply(int a, int b) { return a + b; } };\n'
                      '        abstract int apply(int a, int b);\n    }')),
    dict(id="g-annotation", category="Declarations", title="Declaring an annotation type (@interface)",
         summary="`@interface Marker { }` and using it.", main="G",
         source=_prog('System.out.println("annotated");',
                      '    @interface Marker { }\n    @Marker static class Thing { }')),
    dict(id="g-generic-method", category="Declarations", title="Generic method (with return-type inference)",
         summary="`static <T> T firstOf(T a, T b)`. The return recovers its type argument from the arguments, so `String s = firstOf(\"a\", \"b\")` narrows back from the erased Object with no cast.", main="G",
         source=_prog('String s = firstOf("a", "b");\n        System.out.println(s.length());',
                      '    static <T> T firstOf(T a, T b) { return a; }')),
    dict(id="g-bounded-type", category="Declarations", title="Bounded type parameter (<T extends Comparable<T>>)",
         summary="The type variable erases to its bound, so `compareTo()` resolves on it; `max(3, 5)` and `max(\"a\", \"b\")` work, and the inferred return assigns straight back to the argument's own type.", main="G",
         source=_prog('String longer = max("a", "bb");\n        System.out.println(longer);',
                      '    static <T extends Comparable<T>> T max(T a, T b) {\n'
                      '        return a.compareTo(b) >= 0 ? a : b;\n    }')),
    dict(id="g-wildcard", category="Declarations", title="Wildcard generics (? extends)",
         summary="A `List<Integer>` passes to a `List<? extends Number>` parameter — the use-site covariance a wildcard grants. The bound is a real constraint: a `List<String>` is still refused, and a plain `List<Object>` stays invariant.", main="G",
         source=_prog('List<Integer> n = new ArrayList<>();\n        n.add(2);\n        n.add(9);\n'
                      '        System.out.println(count(n));',
                      '    static int count(List<? extends Number> values) { return values.size(); }',
                      'import java.util.ArrayList;\nimport java.util.List;')),
    dict(id="g-nested-generics", category="Declarations", title="Nested generic collections (List<List<Integer>>)",
         summary="A collection's parameterized element keeps its type, so `grid.get(0)` is a `List<Integer>` and `grid.get(0).get(1)` type-checks — at any depth, and through a `Map<K, List<V>>` value. The element is a real constraint: `grid.add(\"x\")` is refused.", main="G",
         source=_prog('List<List<Integer>> grid = new ArrayList<>();\n'
                      '        List<Integer> row = new ArrayList<>();\n'
                      '        row.add(10);\n        row.add(20);\n        grid.add(row);\n'
                      '        System.out.println(grid.get(0).get(1));',
                      imports='import java.util.ArrayList;\nimport java.util.List;')),
    dict(id="g-final-param", category="Declarations", title="final parameters",
         summary="`static int twice(final int v)`. A final LOCAL works; a final parameter does not.", main="G",
         source=_prog('System.out.println(twice(2));',
                      '    static int twice(final int v) { return v * 2; }')),
    dict(id="g-package", category="Declarations", title="package declaration",
         summary="caturra puts every class in one namespace.", main="G",
         source='package demo;\n\npublic class G {\n    public static void main(String[] args) {\n'
                '        System.out.println("packaged");\n    }\n}\n'),
    dict(id="g-static-import", category="Declarations", title="Static import of a library member",
         summary="`import static java.lang.Math.max;` (a static import of a USER class works — this is the library one).",
         main="G",
         source='import static java.lang.Math.max;\n\npublic class G {\n'
                '    public static void main(String[] args) {\n        System.out.println(max(2, 3));\n    }\n}\n'),

    # ----- statements -----
    dict(id="g-labeled", category="Statements", title="Labeled break and continue",
         summary="`outer: for (...) { ... continue outer; }`.", main="G",
         source=_prog('outer:\n        for (int i = 0; i < 3; i++) {\n'
                      '            for (int j = 0; j < 3; j++) {\n'
                      '                if (j == 1) continue outer;\n'
                      '                if (i == 2) break outer;\n'
                      '                System.out.println(i + "," + j);\n            }\n        }')),
    dict(id="g-switch-kinds", category="Statements", title="switch on String, char and enum",
         summary="All three selector types, with fall-through and default.", main="G",
         source=_prog('String day = "tue";\n        switch (day) {\n'
                      '            case "mon": System.out.println("start"); break;\n'
                      '            case "tue": System.out.println("second"); break;\n'
                      '            default: System.out.println("other");\n        }\n'
                      "        char grade = 'B';\n        switch (grade) {\n"
                      "            case 'A': System.out.println(4); break;\n"
                      "            case 'B': System.out.println(3); break;\n"
                      '            default: System.out.println(0);\n        }\n'
                      '        Color c = Color.RED;\n        switch (c) {\n'
                      '            case RED: System.out.println("red"); break;\n'
                      '            default: System.out.println("other");\n        }',
                      '    enum Color { RED, BLUE }')),
    dict(id="g-try-finally", category="Statements", title="try / catch / finally, and checked exceptions",
         summary="`throws`, a caught checked exception, and finally running on the way out of a return.",
         main="G",
         source=_prog('try {\n            risky();\n        } catch (Exception e) {\n'
                      '            System.out.println("caught " + e.getMessage());\n        }\n'
                      '        System.out.println(f());',
                      '    static void risky() throws Exception { throw new Exception("checked"); }\n'
                      '    static int f() {\n        try { return 1; } finally { System.out.println("fin"); }\n    }')),
    dict(id="g-try-resources", category="Statements", title="try-with-resources",
         summary="`try (PrintWriter w = ...) { }` — the resource closes itself.", main="G",
         source=_prog('try (java.io.PrintWriter w = new java.io.PrintWriter("g.txt")) {\n'
                      '            w.println("x");\n        }\n        System.out.println("closed");')),
    dict(id="g-multicatch", category="Statements", title="Multi-catch (catch A | B)",
         summary="One clause, two exception types.", main="G",
         source=_prog('try {\n            throw new IllegalStateException("boom");\n'
                      '        } catch (IllegalStateException | IllegalArgumentException e) {\n'
                      '            System.out.println("caught " + e.getMessage());\n        }')),
    dict(id="g-assert", category="Statements", title="assert",
         summary="A runtime no-op — assertions are off by default (only `-ea` enables them) — but the condition is still type-checked, as javac does.", main="G",
         source=_prog('assert 1 + 1 == 2 : "math";\n        System.out.println("asserted");')),
    dict(id="g-synchronized", category="Statements", title="synchronized",
         summary=("A program here runs on one thread, so a monitor is never contended: "
                  "the lock is evaluated (a null one still throws) and the body runs."),
         main="G",
         source=_prog('Object lock = new Object();\n        synchronized (lock) {\n'
                      '            System.out.println("locked");\n        }\n'
                      '        Object missing = null;\n        try {\n'
                      '            synchronized (missing) {\n'
                      '                System.out.println("never");\n            }\n'
                      '        } catch (NullPointerException e) {\n'
                      '            System.out.println("null lock throws");\n        }')),
    dict(id="g-for-forms", category="Statements", title="Every for loop",
         summary="Comma-separated init/update, `for(;;)` with break, and the enhanced for.", main="G",
         source=_prog('for (int i = 0, j = 3; i < j; i++, j--) {\n'
                      '            System.out.println(i + " " + j);\n        }\n'
                      '        int n = 0;\n        for (;;) {\n            if (n++ > 1) break;\n        }\n'
                      '        System.out.println(n);\n'
                      '        for (String s : new String[] { "a", "b" }) {\n'
                      '            System.out.println(s);\n        }')),

    # ----- expressions -----
    dict(id="g-bitwise", category="Expressions", title="Bitwise operators and shifts",
         summary="& | ^ ~ << >> and the unsigned >>>.", main="G",
         source=_prog('int a = 0b1100, b = 0b1010;\n'
                      '        System.out.println((a & b) + " " + (a | b) + " " + (a ^ b) + " " + (~a));\n'
                      '        System.out.println((a << 2) + " " + (a >> 1) + " " + (-8 >>> 28));')),
    dict(id="g-compound", category="Expressions", title="Compound assignment, every form",
         summary="+= -= *= /= %= <<= >>= &= |= ^=.", main="G",
         source=_prog('int x = 8;\n        x += 2; x -= 1; x *= 3; x /= 2; x %= 7; x <<= 1; x >>= 1;\n'
                      '        x &= 6; x |= 1; x ^= 2;\n        System.out.println(x);')),
    dict(id="g-numeric-edges", category="Expressions", title="Integer overflow, division, NaN",
         summary="Java's exact wrap-around, truncating division, and 1/0.0.", main="G",
         source=_prog('System.out.println(Integer.MAX_VALUE + 1);\n'
                      '        System.out.println(-7 / 2 + " " + -7 % 2);\n'
                      '        System.out.println(1 / 0.0 + " " + 0.0 / 0.0);\n'
                      "        char c = 'a';\n        System.out.println((c + 1) + \" \" + (char) (c - 32));")),
    dict(id="g-method-ref-instance", category="Expressions", title="Method references: obj::method and Class::new",
         summary="A bound instance reference, and a constructor reference.", main="G",
         source=_prog('String prefix = "hi ";\n        Greeter g = prefix::concat;\n'
                      '        System.out.println(g.greet("ada"));\n'
                      '        Maker m = G::new;\n        System.out.println(m.make() != null);',
                      '    interface Greeter { String greet(String name); }\n'
                      '    interface Maker { G make(); }')),
    dict(id="g-method-ref-static", category="Expressions", title="Method reference to a STATIC method (Class::method)",
         summary="`Comparator.comparing(G::key)` — caturra reads Class::method as an instance reference.",
         main="G",
         source=_prog('List<String> l = new ArrayList<>();\n        l.add("b");\n        l.add("a");\n'
                      '        l.sort(Comparator.comparing(G::key));\n        System.out.println(l);',
                      '    static String key(String s) { return s; }',
                      'import java.util.ArrayList;\nimport java.util.Comparator;\nimport java.util.List;')),
    dict(id="g-instanceof-iface", category="Expressions", title="instanceof a library interface (List, Map)",
         summary="`o instanceof List` — instanceof against a collection interface.", main="G",
         source=_prog('Object o = new ArrayList<String>();\n'
                      '        if (o instanceof List) {\n'
                      '            System.out.println("a list");\n        }',
                      '', 'import java.util.ArrayList;\nimport java.util.List;')),
    dict(id="g-anon-diamond", category="Expressions", title="Anonymous class with a diamond (Java 9)",
         summary="`new Comparator<>() { ... }` — the diamond on an anonymous class body.", main="G",
         source=_prog('Comparator<String> c = new Comparator<>() {\n'
                      '            @Override public int compare(String a, String b) { return a.compareTo(b); }\n'
                      '        };\n        System.out.println(c.compare("a", "b"));',
                      '', 'import java.util.Comparator;')),
    dict(id="g-function-package", category="Expressions",
         title="java.util.function (Function, Predicate, Supplier, Consumer)",
         summary="The JDK's standard functional interfaces, from a lambda or a method reference.",
         main="G",
         source=_prog('Function<Integer, Integer> twice = n -> n * 2;\n'
                      '        System.out.println(twice.apply(4));\n'
                      '        Function<String, Integer> len = String::length;\n'
                      '        System.out.println(len.apply("hello"));\n'
                      '        Predicate<String> nonEmpty = s -> !s.isEmpty();\n'
                      '        System.out.println(nonEmpty.test("hi") + " " + nonEmpty.test(""));\n'
                      '        Supplier<String> greet = () -> "hello";\n'
                      '        System.out.println(greet.get());\n'
                      '        Consumer<String> shout = s -> System.out.println(s + "!");\n'
                      '        shout.accept("go");\n'
                      '        BiFunction<Integer, Integer, Integer> add = (a, b) -> a + b;\n'
                      '        System.out.println(add.apply(2, 3));',
                      '', 'import java.util.function.BiFunction;\n'
                      'import java.util.function.Consumer;\n'
                      'import java.util.function.Function;\n'
                      'import java.util.function.Predicate;\n'
                      'import java.util.function.Supplier;')),
    dict(id="g-shadowing", category="Expressions", title="Shadowing, this, and super",
         summary="A local shadowing a field, `this.x`, and `super.method()`.", main="G",
         source=_prog('System.out.println(new G(5).show());\n        System.out.println(new Child().describe());',
                      '    int x = 1;\n    G() { }\n    G(int x) { this.x = x; }\n'
                      '    String show() { int x = 99; return x + " " + this.x; }\n'
                      '    static class Parent { String describe() { return "parent"; } }\n'
                      '    static class Child extends Parent {\n'
                      '        @Override String describe() { return super.describe() + "+child"; }\n    }')),
    # ----- landed 2026-07-18/19: each of these was a gap the differential
    # audit found, so the page should show them working rather than leave the
    # grammar walk silent about them.
    dict(id="g-regex", category="Library", title="Regular expressions (split, matches, replaceAll)",
         summary="`String.split` takes a REGEX, not a literal — with groups, quantifiers and $1 in the replacement.",
         main="G",
         source=_prog('System.out.println(Arrays.toString("1.2.3".split("\\\\.")));\n'
                      '        System.out.println(Arrays.toString("a1b22c".split("[0-9]+")));\n'
                      '        System.out.println("2024-01-15".matches("\\\\d{4}-\\\\d{2}-\\\\d{2}"));\n'
                      '        System.out.println("John Smith".replaceAll("(\\\\w+) (\\\\w+)", "$2, $1"));\n'
                      '        System.out.println("a b  c".replaceAll("\\\\s+", "_"));',
                      '', 'import java.util.Arrays;')),
    dict(id="g-matcher", category="Library", title="Pattern and Matcher",
         summary="A compiled pattern, walked with find/group/start — the same engine String.matches uses.",
         main="G",
         source=_prog('Matcher m = Pattern.compile("(\\\\w+)@(\\\\w+)").matcher("a@b and c@d");\n'
                      '        while (m.find()) {\n'
                      '            System.out.println(m.group() + " " + m.group(1) + " " + m.start());\n'
                      '        }\n'
                      '        System.out.println(Pattern.compile("A", Pattern.CASE_INSENSITIVE).matcher("xax").find());\n'
                      '        System.out.println(Pattern.compile("(\\\\d)(\\\\d)").matcher("12").replaceAll("$2$1"));',
                      '', 'import java.util.regex.Matcher;\nimport java.util.regex.Pattern;')),
    dict(id="g-matcher-region", category="Library", title="Matcher: regions, rewriting, results",
         summary="The rest of the Matcher API — a region with anchoring and transparent bounds, the appendReplacement loop, and results() as a lazy stream of frozen matches.",
         main="G",
         source=_prog('Matcher m = Pattern.compile("\\\\d+").matcher("a12b345");\n'
                      '        StringBuilder sb = new StringBuilder();\n'
                      '        while (m.find()) {\n'
                      '            m.appendReplacement(sb, "<" + m.group().length() + ">");\n'
                      '        }\n'
                      '        System.out.println(m.appendTail(sb));\n'
                      '        Matcher r = Pattern.compile("\\\\bcat\\\\b").matcher("thecat here");\n'
                      '        r.region(3, 6);\n'
                      '        System.out.println(r.matches());\n'
                      '        r.useTransparentBounds(true);\n'
                      '        r.reset();\n'
                      '        r.region(3, 6);\n'
                      '        System.out.println(r.matches());\n'
                      '        Matcher w = Pattern.compile("\\\\w+").matcher("one two three");\n'
                      '        System.out.println(w.results().map(MatchResult::group).collect(Collectors.toList()));\n'
                      '        System.out.println(Pattern.compile("\\\\d").matcher("a1b2")\n'
                      '            .replaceAll(one -> "[" + one.group() + "]"));\n'
                      '        System.out.println(Pattern.compile("^a").asPredicate().test("abc"));',
                      '', 'import java.util.regex.MatchResult;\nimport java.util.regex.Matcher;\n'
                      'import java.util.regex.Pattern;\nimport java.util.stream.Collectors;')),
    dict(id="g-nio-path", category="Library", title="Path: building and taking apart a filename",
         summary="java.nio.file.Path — Path.of joins its segments, and every method works on the NAME ELEMENTS, not the text.",
         main="G",
         source=_prog('Path p = Path.of("home", "wilkie", "notes.txt");\n'
                      '        System.out.println(p + " " + p.getNameCount() + " " + p.getFileName());\n'
                      '        System.out.println(p.getParent() + " " + p.getName(0) + " " + p.isAbsolute());\n'
                      '        System.out.println(Path.of("a/./b/../c").normalize());\n'
                      '        System.out.println(p.resolveSibling("other.txt"));\n'
                      '        System.out.println(Path.of("a", "b").relativize(Path.of("a", "b", "c")));\n'
                      '        System.out.println(p.startsWith("home") + " " + Path.of("a", "bc").startsWith("a/b"));\n'
                      '        System.out.println(List.of(Path.of("x"), Path.of("y")));',
                      '', 'import java.nio.file.Path;\nimport java.util.List;')),
    dict(id="g-default-methods", category="Classes", title="Interfaces: default methods a class inherits",
         summary="A default method can implement a library interface's own (Comparable, Comparator, Iterable), and the class inherits it — including the erased bridge a JDK synthesizes.",
         main="G",
         source=_prog('List<Shape> shapes = new ArrayList<>(List.of(new Sq(3), new Sq(1)));\n'
                      '        Collections.sort(shapes);\n'
                      '        System.out.println(shapes);\n'
                      '        System.out.println(shapes.get(0) instanceof Comparable);\n'
                      '        Bag bag = new Bag();\n'
                      '        bag.add("x");\n        bag.add("y");\n'
                      '        bag.forEach(System.out::println);\n'
                      '        System.out.println(Comparator.<String>naturalOrder().compare("ab", "cd"));',
                      '    interface Shape extends Comparable<Shape> {\n'
                      '        double area();\n'
                      '        default int compareTo(Shape other) { return Double.compare(area(), other.area()); }\n'
                      '    }\n'
                      '    static class Sq implements Shape {\n'
                      '        final double s;\n'
                      '        Sq(double s) { this.s = s; }\n'
                      '        public double area() { return s * s; }\n'
                      '        public String toString() { return "Sq" + area(); }\n'
                      '    }\n'
                      '    static class Bag implements Iterable<String> {\n'
                      '        final List<String> items = new ArrayList<>();\n'
                      '        void add(String s) { items.add(s); }\n'
                      '        public Iterator<String> iterator() { return items.iterator(); }\n'
                      '    }\n',
                      'import java.util.ArrayList;\nimport java.util.Collections;\n'
                      'import java.util.Comparator;\nimport java.util.Iterator;\nimport java.util.List;')),
    dict(id="g-conditional", category="Expressions", title="The type of a conditional (?:)",
         summary="JLS 15.25: a char and a fitting int constant give a char, mixed numbers promote, and two references join at what they share — including a face neither wears.",
         main="G",
         source=_prog('boolean flag = args.length == 0;\n'
                      '        System.out.println((int) (flag ? \'a\' : 98));\n'
                      '        System.out.println(flag ? 1 : 2.0);\n'
                      '        System.out.println(flag ? 1L : 2);\n'
                      '        Integer boxed = 3;\n'
                      '        System.out.println(flag ? boxed : 4);\n'
                      '        Object either = flag ? Integer.valueOf(1) : Double.valueOf(2);\n'
                      '        System.out.println(either + " " + either.getClass().getSimpleName());\n'
                      '        List<String> list = flag ? new ArrayList<>() : new LinkedList<>();\n'
                      '        System.out.println(list.size());\n'
                      '        CharSequence text = flag ? new StringBuilder("sb") : "str";\n'
                      '        System.out.println(text.length());',
                      '', 'import java.util.ArrayList;\nimport java.util.LinkedList;\nimport java.util.List;')),
    dict(id="g-enum-collections", category="Collections", title="EnumMap and EnumSet",
         summary="The enum-keyed collections iterate in their constants' own order, and an EnumSet is built from the enum's universe rather than by hashing.",
         main="G",
         source=_prog('EnumMap<Day, Integer> hours = new EnumMap<>(Day.class);\n'
                      '        hours.put(Day.WED, 3);\n        hours.put(Day.MON, 1);\n'
                      '        System.out.println(hours + " " + hours.get(Day.MON));\n'
                      '        System.out.println(hours.keySet() + " " + hours.getClass().getName());\n'
                      '        System.out.println(hours.get(null));\n'
                      '        EnumSet<Day> some = EnumSet.of(Day.FRI, Day.MON);\n'
                      '        System.out.println(some + " " + some.contains(null));\n'
                      '        System.out.println(EnumSet.allOf(Day.class));\n'
                      '        System.out.println(EnumSet.range(Day.TUE, Day.THU));\n'
                      '        System.out.println(EnumSet.complementOf(some));',
                      '    enum Day { MON, TUE, WED, THU, FRI }\n',
                      'import java.util.EnumMap;\nimport java.util.EnumSet;')),
    dict(id="g-fail-fast", category="Collections", title="Fail-fast iterators (ConcurrentModificationException)",
         summary="Modifying a collection while iterating it throws, as on a real JVM — including the quirk where removing the second-to-last element does not.",
         main="G",
         source=_prog('List<String> l = new ArrayList<>(Arrays.asList("a", "b", "c", "d", "e"));\n'
                      '        try {\n'
                      '            for (String s : l) { if (s.equals("b")) l.remove(s); }\n'
                      '        } catch (ConcurrentModificationException e) {\n'
                      '            System.out.println("threw " + e.getClass().getSimpleName());\n        }\n'
                      '        Iterator<String> it = l.iterator();\n'
                      '        while (it.hasNext()) { if (it.next().equals("c")) it.remove(); }\n'
                      '        System.out.println(l);',
                      '', 'import java.util.ArrayList;\nimport java.util.Arrays;\n'
                      'import java.util.ConcurrentModificationException;\n'
                      'import java.util.Iterator;\nimport java.util.List;')),
    dict(id="g-casts", category="Expressions", title="Casts: to Object, boxing and unboxing",
         summary="Every reference widens to Object (which is how you pick an overload), and a primitive casts to its wrapper.",
         main="G",
         source=_prog('Object o = (Object) "hi";\n        System.out.println(o);\n'
                      '        System.out.println(which("x"));\n'
                      '        System.out.println(which((Object) "x"));\n'
                      '        int n = 5;\n'
                      '        System.out.println((Integer) n);\n'
                      '        System.out.println((int) (Integer) n);\n'
                      '        System.out.println((Double) 2.5);',
                      '    static String which(Object x) { return "Object"; }\n'
                      '    static String which(String x) { return "String"; }')),
    dict(id="g-qualified-this", category="Declarations", title="Qualified this (Outer.this) and enclosing access",
         summary="An inner class names either instance, and an anonymous class reads its enclosing object's members.",
         main="G",
         source=_prog('G outer = new G();\n        outer.new Inner().show();\n        outer.viaAnonymous();',
                      '    int x = 1;\n    int twice() { return x * 2; }\n'
                      '    class Inner {\n        int x = 2;\n'
                      '        void show() { System.out.println(G.this.x + " " + this.x + " " + G.this.twice()); }\n    }\n'
                      '    interface Src { int get(); }\n'
                      '    void viaAnonymous() {\n'
                      '        Src s = new Src() { public int get() { return x + twice(); } };\n'
                      '        System.out.println(s.get());\n    }')),
    dict(id="g-generic-supertype", category="Declarations", title="Parameterized supertype (with bridge methods)",
         summary="A subclass stands in for `Box<String>`, and a call through that reference reaches the OVERRIDE — which needs the bridge method erasure would otherwise lose.",
         main="G",
         source=_prog('Box<String> b = new SBox();\n        b.set("typed");\n'
                      '        F<String> f = new SF();\n        System.out.println(f.apply("y"));',
                      '    static class Box<T> { void set(T t) { System.out.println("Box.set " + t); } }\n'
                      '    static class SBox extends Box<String> {\n'
                      '        @Override void set(String s) { System.out.println("SBox.set " + s); }\n    }\n'
                      '    interface F<T> { String apply(T t); }\n'
                      '    static class SF implements F<String> {\n'
                      '        public String apply(String s) { return "SF:" + s; }\n    }')),
]
