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
