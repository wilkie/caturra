#!/usr/bin/env python3
"""The same expression in every syntactic POSITION — a MIRROR sweep.

One expression, twelve places to write it: a `println` argument, a `var`, a
concatenation, an argument to a method, a return, a field initializer, an array
element, a ternary branch, a lambda body, an ANONYMOUS CLASS body,
`String.valueOf`, a plain assignment. Every position prints the same thing, so
any position that differs is a position the compiler types differently from the
others — which is exactly the recurring defect in this codebase, one fact read
by two paths.

It found a lambda inside an anonymous class body refused for having no
functional-interface position, in a program whose identical pipeline one line
outside compiled.

    scripts/fuzz/positions.py [--out DIR]
    scripts/fuzz/run.py <DIR>
"""
import os
import sys

# (name, prelude statements, expression)
EXPRS = [
 ("diamond", 'Node<Integer> n = new Node<>(5);', 'n.get() + 1'),
 ("factory", '', 'Box.of("hi").get()'),
 ("entrySet", 'Map<String, Integer> m = new LinkedHashMap<>(); m.put("k", 1);', 'm.entrySet()'),
 ("subList", 'List<String> l = new ArrayList<>(Arrays.asList("a", "b", "c"));', 'l.subList(0, 2)'),
 ("asList", '', 'Arrays.asList(1, 2)'),
 ("listOf", '', 'List.of("a", "b")'),
 ("keySet", 'Map<String, Integer> m = new LinkedHashMap<>(); m.put("k", 1);', 'm.keySet()'),
 ("collect", 'List<String> l = new ArrayList<>(Arrays.asList("aa", "b"));',
  'l.stream().filter(s -> s.length() > 1).collect(Collectors.toList())'),
 ("mapped", 'List<String> l = new ArrayList<>(Arrays.asList("aa", "b"));',
  'l.stream().map(String::length).collect(Collectors.toList())'),
 ("optional", '', 'Optional.of("x").map(String::toUpperCase).orElse("-")'),
 ("entryCmp", 'List<Map.Entry<String, Integer>> es = new ArrayList<>(new LinkedHashMap<String, Integer>().entrySet());',
  'Map.Entry.comparingByKey().getClass().getSimpleName().isEmpty()'),
 ("array", '', 'Arrays.toString(new int[] {1, 2})'),
 ("concat", 'int k = 3;', '"a" + k + 1.5'),
 ("cast", 'Object o = "s";', '((String) o).length()'),
 ("boxed", '', 'Integer.valueOf(3).compareTo(4)'),
 ("poll", 'Queue<String> q = new PriorityQueue<>(Arrays.asList("b", "a"));', 'q.poll()'),
 ("cursor", 'List<String> l = new ArrayList<>(Arrays.asList("a", "b"));', 'l.iterator().next()'),
 ("chars", '', '"hello".chars().filter(c -> c == \'l\').count()'),
 ("ternary", 'boolean flag = true;', '(flag ? 1 : 2.5)'),
 ("emptyList", '', 'Collections.emptyList()'),
 ("builder", '', 'new StringBuilder("abc").reverse().toString()'),
 ("mathmax", '', 'Math.max(1, 2L)'),
 ("clone", 'int[] a = {1, 2};', 'Arrays.toString(a.clone())'),
 ("comparator", '', 'Comparator.comparing(String::length).compare("aa", "b")'),
 ("nested", '', 'new ArrayList<>(Arrays.asList(new ArrayList<>(Arrays.asList(1)))).get(0).get(0)'),
 ("stringFmt", '', 'String.format("%s-%d", "x", 5)'),
 # A parameterized library INTERFACE as a declared type. Its type ARGUMENT
 # used to be dropped on the way in, so `Comparable<Integer> c = "x";`
 # compiled; every position has to read the same argument now.
 ("comparableInt", 'Comparable<Integer> c = 5;', 'c.compareTo(3)'),
 ("comparableStr", 'Comparable<String> c = "b";', 'c.compareTo("a")'),
 ("comparableChar", 'Comparable<Character> c = \'z\';', 'c.compareTo(\'a\')'),
 ("comparableEnum", '', 'java.time.Month.MAY.compareTo(java.time.Month.JUNE)'),
 ("comparableUnbox", 'Comparable<Integer> c = 5;', '(int) c + 1'),
 ("enumFace", '', '((Enum<java.time.DayOfWeek>) java.time.DayOfWeek.MONDAY).ordinal()'),
 ("cloneableArray", 'Cloneable c = new int[] {1, 2};', '((int[]) c).length'),
 # A generic FACTORY's type argument comes from the TARGET, so the same call
 # has a different element in every position it is written into.
 ("wideList", 'List<Number> w = List.of(1, 2.5);', 'w.get(0).intValue() + w.size()'),
 ("wideCopy", 'List<Object> w = new ArrayList<>(List.of("a"));', 'w.size() + "" + w.get(0)'),
 ("wideMap", 'Map<String, Number> w = Map.of("k", 1);', 'w.get("k").intValue()'),
 ("wideOptional", 'Optional<Number> w = Optional.of(1);', 'w.get().doubleValue()'),
 ("wideCollected", 'List<Number> w = Stream.of(1, 2).collect(Collectors.toList());',
  'w.size() + w.get(1).intValue()'),
 # A generic METHOD's variable comes from the target too, and one written with
 # no argument at all has nowhere else to get it.
 ("genericWide", 'List<Number> w = pairOf(1, 2.5);', 'w.get(0).intValue() + w.size()'),
 ("genericEmpty", 'List<Number> w = none(); w.add(3);', 'w.get(0).intValue()'),
 ("genericExact", '', 'pairOf("a", "b").get(0).length()'),
]

POSITIONS = {
 "println": 'System.out.println({E});',
 "var": 'var v = {E};\n        System.out.println(v);',
 "concatArg": 'System.out.println("" + {E});',
 "argument": 'take({E});',
 "returned": 'System.out.println(make{N}());',
 "field": 'System.out.println(FIELD{N});',
 "arrayElem": 'Object[] cell = { {E} };\n        System.out.println(cell[0]);',
 "ternaryBranch": 'Object t = args.length == 0 ? {E} : null;\n        System.out.println(t);',
 "lambdaBody": 'Supplier<Object> sup = () -> {E};\n        System.out.println(sup.get());',
 "anonBody": 'Supplier<Object> anon = new Supplier<Object>() {\n            public Object get() { return {E}; }\n        };\n        System.out.println(anon.get());',
 "valueOf": 'System.out.println(String.valueOf({E}));',
 "assigned": 'Object slot;\n        slot = {E};\n        System.out.println(slot);',
}

HELPERS = """    static class Node<T> {
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
    static <T> List<T> pairOf(T a, T b) { return new ArrayList<>(Arrays.asList(a, b)); }
    static <T> List<T> none() { return new ArrayList<>(); }
"""

out_dir = "cases"
if "--out" in sys.argv:
    out_dir = sys.argv[sys.argv.index("--out") + 1]
os.makedirs(out_dir, exist_ok=True)
n = 0
for ename, prelude, expr in EXPRS:
    for pname, shape in POSITIONS.items():
        name = f"Pos_{ename}_{pname}"
        body = shape.replace("{E}", expr).replace("{N}", "")
        extra = ""
        if pname == "returned":
            extra = f"    static Object make() {{\n        {prelude}\n        return {expr};\n    }}\n"
        if pname == "field":
            # a static field initializer cannot see the prelude locals: only
            # emit for expressions that need none
            if prelude:
                continue
            extra = f"    static final Object FIELD = {expr};\n"
        pre = "" if pname in ("returned", "field") else f"        {prelude}\n"
        open(os.path.join(out_dir, name + ".java"), "w").write(f"""import java.util.*;
import java.util.function.*;
import java.util.stream.*;

public class {name} {{
{HELPERS}{extra}
    public static void main(String[] args) {{
{pre}        {body}
    }}
}}
""")
        n += 1
print("wrote", n)
