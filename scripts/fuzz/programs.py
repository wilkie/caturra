#!/usr/bin/env python3
"""A TYPED random-program generator: every program it writes compiles, prints,
and ends.

The generator tracks the type of every variable it declares, so an expression is
only ever built where its type fits — which is what keeps javac's rejection rate
at zero and makes every case a real comparison rather than a syntax check. It
mixes the shapes a student program is made of (arithmetic, strings, control
flow, collections, arrays, try/catch) with the ones that have historically
broken: a generic class, a map walked by its entries, method references, nested
lambdas, a `subList` handed to an algorithm.

    scripts/fuzz/programs.py [count] [seed] [--out DIR]
    scripts/fuzz/run.py <DIR>

A seed is the whole state: the same seed writes the same programs, so a
divergence can be reproduced exactly.
"""
import random, os, sys

PRELUDE = """import java.util.*;
import java.util.stream.*;

public class %s {
    static class Animal implements Comparable<Animal> {
        final String name; final int size;
        Animal(String name, int size) { this.name = name; this.size = size; }
        public int compareTo(Animal o) { return Integer.compare(size, o.size); }
        public String toString() { return name + "(" + size + ")"; }
        String describe() { return "animal:" + name; }
    }
    static class Cat extends Animal {
        Cat(String name, int size) { super(name, size); }
        @Override String describe() { return "cat:" + name; }
    }
    static class Box<T> {
        private final T value;
        Box(T value) { this.value = value; }
        T get() { return value; }
        <R> Box<R> map(java.util.function.Function<T, R> f) { return new Box<>(f.apply(value)); }
        public String toString() { return "Box[" + value + "]"; }
    }
    static int twice(int n) { return n * 2; }
    static long fact(int n) { return n <= 1 ? 1 : n * fact(n - 1); }
    static long fib(int n) { return n < 2 ? n : fib(n - 1) + fib(n - 2); }
    static String tag(String s) { return "<" + s + ">"; }
    static <T extends Comparable<T>> T biggest(List<T> xs) {
        T best = xs.get(0);
        for (T x : xs) { if (x.compareTo(best) > 0) best = x; }
        return best;
    }

    public static void main(String[] args) {
"""

class Gen:
    def __init__(self, rng):
        self.rng = rng
        self.vars = {}   # name -> type
        self.counter = 0
        self.lines = []

    def name(self, prefix="v"):
        self.counter += 1
        return f"{prefix}{self.counter}"

    def of_type(self, ty):
        return [n for n, t in self.vars.items() if t == ty]

    def lit(self, ty):
        r = self.rng
        if ty == "int": return str(r.randint(-20, 100))
        if ty == "long": return f"{r.randint(-1000, 100000)}L"
        if ty == "double": return f"{round(r.uniform(-50, 50), 3)}"
        if ty == "boolean": return r.choice(["true", "false"])
        if ty == "char": return "'" + r.choice("abcxyz") + "'"
        if ty == "String": return '"' + r.choice(["alpha", "b", "cc", "", "Zed", "mixed Case"]) + '"'
        if ty == "int[]": return "new int[] {" + ", ".join(str(r.randint(0, 30)) for _ in range(r.randint(1, 4))) + "}"
        if ty == "String[]": return "new String[] {" + ", ".join('"' + r.choice(["a", "bb", "ccc"]) + '"' for _ in range(r.randint(1, 3))) + "}"
        if ty == "List<Integer>": return "new ArrayList<>(Arrays.asList(" + ", ".join(str(r.randint(0, 20)) for _ in range(r.randint(1, 4))) + "))"
        if ty == "List<String>": return 'new ArrayList<>(Arrays.asList(' + ", ".join('"' + r.choice(["p", "qq", "rrr"]) + '"' for _ in range(r.randint(1, 4))) + '))'
        if ty == "Map<String,Integer>":
            pairs = r.randint(1, 3)
            m = self.name("m")
            self.lines.append(f"        Map<String, Integer> {m} = new LinkedHashMap<>();")
            for i in range(pairs):
                self.lines.append(f'        {m}.put("k{i}", {r.randint(0, 9)});')
            return m
        if ty == "Animal": return f'new {r.choice(["Animal", "Cat"])}("{r.choice(["ann", "bo", "cy"])}", {r.randint(1, 9)})'
        if ty == "Box<String>": return f'new Box<>({self.expr("String", 0)})'
        raise ValueError(ty)

    def expr(self, ty, depth):
        r = self.rng
        pool = self.of_type(ty)
        choices = ["lit"]
        if pool: choices += ["var", "var"]
        if depth < 2:
            if ty == "int": choices += ["arith", "len", "call", "size", "index", "ternary"]
            if ty == "double": choices += ["arith", "ternary"]
            if ty == "long": choices += ["arith"]
            if ty == "boolean": choices += ["cmp", "contains", "ternary"]
            if ty == "String": choices += ["concat", "call", "upper", "ternary", "join"]
        pick = r.choice(choices)
        if pick == "var": return r.choice(pool)
        if pick == "lit": return self.lit(ty)
        if pick == "arith":
            op = r.choice(["+", "-", "*"])
            return f"({self.expr(ty, depth + 1)} {op} {self.expr(ty, depth + 1)})"
        if pick == "len":
            s = self.expr("String", depth + 1)
            return f"{s}.length()"
        if pick == "size":
            src = self.of_type("List<Integer>") + self.of_type("List<String>")
            if not src: return self.lit("int")
            return f"{r.choice(src)}.size()"
        if pick == "index":
            src = self.of_type("int[]")
            if not src: return self.lit("int")
            a = r.choice(src)
            return f"({a}.length > 0 ? {a}[0] : -1)"
        if pick == "call": 
            if ty == "int": return f"twice({self.expr('int', depth + 1)})"
            return f"tag({self.expr('String', depth + 1)})"
        if pick == "cmp":
            t = r.choice(["int", "double", "String"])
            if t == "String":
                return f"{self.expr('String', depth + 1)}.equals({self.expr('String', depth + 1)})"
            op = r.choice(["<", "<=", ">", ">=", "==", "!="])
            return f"({self.expr(t, depth + 1)} {op} {self.expr(t, depth + 1)})"
        if pick == "contains":
            src = self.of_type("List<String>")
            if not src: return self.lit("boolean")
            return f"{r.choice(src)}.contains({self.expr('String', depth + 1)})"
        if pick == "concat":
            return f"({self.expr('String', depth + 1)} + {self.expr(r.choice(['int', 'String', 'boolean', 'char']), depth + 1)})"
        if pick == "upper":
            return f"{self.expr('String', depth + 1)}.toUpperCase()"
        if pick == "join":
            src = self.of_type("List<String>")
            if not src: return self.lit("String")
            return f'String.join("-", {r.choice(src)})'
        if pick == "ternary":
            return f"({self.expr('boolean', depth + 1)} ? {self.expr(ty, depth + 1)} : {self.expr(ty, depth + 1)})"
        raise ValueError(pick)

    def decl(self):
        ty = self.rng.choice(["int", "long", "double", "boolean", "char", "String", "int[]",
                              "String[]", "List<Integer>", "List<String>", "Map<String,Integer>",
                              "Animal", "Box<String>"])
        value = self.expr(ty, 0) if ty in ("int", "long", "double", "boolean", "char", "String") else self.lit(ty)
        n = self.name()
        written = ty.replace("Map<String,Integer>", "Map<String, Integer>")
        if ty == "Map<String,Integer>":
            self.vars[n] = ty
            self.lines.append(f"        Map<String, Integer> {n} = {value};")
            return
        self.lines.append(f"        {written} {n} = {value};")
        self.vars[n] = ty

    def stmt(self, depth=0):
        r = self.rng
        kinds = ["decl", "print", "print", "if", "for", "foreach", "while", "assign", "try",
                 "stream", "sort", "generic", "map", "methodref", "arrays", "nested", "sublist",
                 "switch", "dowhile", "labeled", "format", "builder", "cast", "anon", "recurse"]
        kind = r.choice(kinds)
        if kind == "decl": return self.decl()
        if kind == "print":
            ty = r.choice(["int", "String", "boolean", "double"])
            self.lines.append(f"        System.out.println({self.expr(ty, 0)});")
            return
        if kind == "assign":
            targets = [n for n, t in self.vars.items() if t in ("int", "double", "String", "boolean", "long", "char")]
            if not targets: return self.decl()
            n = r.choice(targets)
            self.lines.append(f"        {n} = {self.expr(self.vars[n], 0)};")
            return
        if kind == "if":
            self.lines.append(f"        if ({self.expr('boolean', 0)}) {{")
            self.lines.append(f"            System.out.println({self.expr('String', 0)});")
            self.lines.append("        } else {")
            self.lines.append(f"            System.out.println({self.expr('int', 0)});")
            self.lines.append("        }")
            return
        if kind == "for":
            i = self.name("i")
            self.lines.append(f"        for (int {i} = 0; {i} < {r.randint(1, 4)}; {i}++) {{")
            self.lines.append(f"            System.out.println({i} + \":\" + {self.expr('String', 1)});")
            self.lines.append("        }")
            return
        if kind == "foreach":
            src = self.of_type("List<String>") + self.of_type("List<Integer>") + self.of_type("int[]")
            if not src: return self.decl()
            s = r.choice(src)
            ty = self.vars.get(s, "int[]")
            elem = {"List<String>": "String", "List<Integer>": "Integer", "int[]": "int"}[ty]
            e = self.name("e")
            self.lines.append(f"        for ({elem} {e} : {s}) {{")
            self.lines.append(f"            System.out.println({e});")
            self.lines.append("        }")
            return
        if kind == "while":
            c = self.name("c")
            self.lines.append(f"        int {c} = {r.randint(1, 3)};")
            self.lines.append(f"        while ({c} > 0) {{")
            self.lines.append(f"            System.out.println(\"w\" + {c});")
            self.lines.append(f"            {c}--;")
            self.lines.append("        }")
            return
        if kind == "try":
            self.lines.append("        try {")
            self.lines.append(f"            System.out.println({self.expr('int', 0)} / ({self.expr('int', 1)} % 3));")
            self.lines.append("        } catch (ArithmeticException ex) {")
            self.lines.append("            System.out.println(\"ae \" + ex.getMessage());")
            self.lines.append("        }")
            return
        if kind == "stream":
            src = self.of_type("List<String>")
            if not src: return self.decl()
            s = r.choice(src)
            what = r.choice([
                f'{s}.stream().map(String::toUpperCase).collect(Collectors.toList())',
                f'{s}.stream().filter(x -> x.length() > 1).count()',
                f'{s}.stream().mapToInt(String::length).sum()',
                f'{s}.stream().sorted().collect(Collectors.joining("|"))',
                f'{s}.stream().collect(Collectors.groupingBy(String::length)).size()',
            ])
            self.lines.append(f"        System.out.println({what});")
            return
        if kind == "switch":
            n = self.name("s")
            self.lines.append(f"        int {n} = {self.expr('int', 1)};")
            self.lines.append(f"        switch (Math.floorMod({n}, 4)) {{")
            self.lines.append("            case 0:")
            self.lines.append(f"                System.out.println(\"zero\" + {n});")
            self.lines.append("                break;")
            self.lines.append("            case 1:")
            self.lines.append("            case 2:")
            self.lines.append(f"                System.out.println(\"low\" + {n});")
            self.lines.append("                break;")
            self.lines.append("            default:")
            self.lines.append(f"                System.out.println(\"other\" + {n});")
            self.lines.append("        }")
            src = self.of_type("String")
            if src:
                v = r.choice(src)
                self.lines.append(f"        switch ({v}) {{")
                self.lines.append('            case "alpha": System.out.println("A"); break;')
                self.lines.append('            case "": System.out.println("empty"); break;')
                self.lines.append('            default: System.out.println("d" + ' + v + '.length());')
                self.lines.append("        }")
            return
        if kind == "dowhile":
            c = self.name("d")
            self.lines.append(f"        int {c} = {r.randint(0, 2)};")
            self.lines.append("        do {")
            self.lines.append(f"            System.out.println(\"do\" + {c});")
            self.lines.append(f"            {c}++;")
            self.lines.append(f"        }} while ({c} < {r.randint(1, 3)});")
            return
        if kind == "labeled":
            self.lines.append("        outer" + str(self.counter) + ":")
            label = "outer" + str(self.counter)
            self.counter += 1
            i, j = self.name("i"), self.name("j")
            self.lines.append(f"        for (int {i} = 0; {i} < 3; {i}++) {{")
            self.lines.append(f"            for (int {j} = 0; {j} < 3; {j}++) {{")
            self.lines.append(f"                if ({i} * {j} > 2) break {label};")
            self.lines.append(f"                if ({j} > {i}) continue {label};")
            self.lines.append(f"                System.out.println({i} + \"x\" + {j});")
            self.lines.append("            }")
            self.lines.append("        }")
            return
        if kind == "format":
            self.lines.append(
                f'        System.out.println(String.format("%d|%s|%.2f|%b|%c",'
                f' {self.expr("int", 1)}, {self.expr("String", 1)}, {self.expr("double", 1)},'
                f' {self.expr("boolean", 1)}, {self.expr("char", 1)}));'
            )
            return
        if kind == "builder":
            b = self.name("sb")
            self.lines.append(f"        StringBuilder {b} = new StringBuilder({self.expr('String', 1)});")
            self.lines.append(f"        {b}.append({self.expr('int', 1)}).append('|').append({self.expr('boolean', 1)});")
            if r.random() < 0.5:
                self.lines.append(f"        {b}.reverse();")
            self.lines.append(f"        System.out.println({b} + \":\" + {b}.length());")
            return
        if kind == "cast":
            what = r.choice([
                f"(int) {self.expr('double', 1)}",
                f"(double) {self.expr('int', 1)}",
                f"(char) ({self.expr('char', 1)} + 1)",
                f"(long) {self.expr('int', 1)} * 100000L",
                f"(int) {self.expr('char', 1)}",
            ])
            self.lines.append(f"        System.out.println({what});")
            src = self.of_type("Animal")
            if src:
                a = r.choice(src)
                self.lines.append(f"        System.out.println({a} instanceof Cat ? \"cat\" : \"animal\");")
                self.lines.append(f"        System.out.println({a}.describe() + ((Animal) {a}).size);")
            return
        if kind == "anon":
            n = self.name("anon")
            self.lines.append(f"        Comparator<String> {n} = new Comparator<String>() {{")
            self.lines.append("            public int compare(String a, String b) { return b.length() - a.length(); }")
            self.lines.append("        };")
            src = self.of_type("List<String>")
            if src:
                s2 = r.choice(src)
                self.lines.append(f"        {s2}.sort({n});")
                self.lines.append(f"        System.out.println({s2});")
            else:
                self.lines.append(f'        System.out.println({n}.compare("aa", "b"));')
            return
        if kind == "recurse":
            self.lines.append(f"        System.out.println(fact({r.randint(0, 8)}) + \"/\" + fib({r.randint(0, 12)}));")
            return
        if kind == "generic":
            src = self.of_type("Box<String>")
            if not src:
                b = self.name("b")
                self.lines.append(f"        Box<String> {b} = new Box<>({self.expr('String', 1)});")
                self.vars[b] = "Box<String>"
                src = [b]
            b = r.choice(src)
            what = r.choice([
                f"{b}.get().length()",
                f"{b}.map(s -> s.length()).get()",
                f"{b}.map(String::toUpperCase).get()",
                f"{b}.toString()",
            ])
            self.lines.append(f"        System.out.println({what});")
            return
        if kind == "map":
            src = self.of_type("Map<String,Integer>")
            if not src: return self.decl()
            m = r.choice(src)
            what = r.choice([
                f'{m}.getOrDefault("k0", -1)',
                f'{m}.size() + {m}.keySet().size()',
                f'new TreeMap<>({m}).toString()',
                f'{m}.values().stream().mapToInt(Integer::intValue).sum()',
                f'{m}.entrySet().stream().map(Map.Entry::getKey).sorted().collect(Collectors.joining(","))',
            ])
            self.lines.append(f"        System.out.println({what});")
            if r.random() < 0.5:
                self.lines.append(f'        {m}.merge("k0", 1, Integer::sum);')
                self.lines.append(f"        for (Map.Entry<String, Integer> e : {m}.entrySet()) {{")
                self.lines.append("            System.out.println(e.getKey() + \"=\" + e.getValue());")
                self.lines.append("        }")
            return
        if kind == "methodref":
            src = self.of_type("List<String>")
            if not src: return self.decl()
            s = r.choice(src)
            what = r.choice([
                f"{s}.stream().map(String::length).collect(Collectors.toList())",
                f"{s}.stream().max(Comparator.comparing(String::length)).orElse(\"-\")",
                f"biggest({s})",
                f"{s}.stream().sorted(Comparator.comparingInt(String::length)).collect(Collectors.toList())",
            ])
            self.lines.append(f"        System.out.println({what});")
            return
        if kind == "arrays":
            src = self.of_type("int[]")
            if not src: return self.decl()
            a = r.choice(src)
            what = r.choice([
                f"Arrays.toString({a})",
                f"Arrays.stream({a}).sum()",
                f"Arrays.stream({a}).max().orElse(-1)",
            ])
            self.lines.append(f"        System.out.println({what});")
            if r.random() < 0.4:
                self.lines.append(f"        Arrays.sort({a});")
                self.lines.append(f"        System.out.println(Arrays.toString({a}));")
            return
        if kind == "nested":
            src = self.of_type("List<String>")
            if not src: return self.decl()
            s = r.choice(src)
            self.lines.append(f"        {s}.forEach(outer -> {s}.forEach(inner -> System.out.println(outer + \"/\" + inner.length())));")
            return
        if kind == "sublist":
            src = self.of_type("List<Integer>")
            if not src: return self.decl()
            s = r.choice(src)
            self.lines.append(f"        if ({s}.size() >= 2) {{")
            self.lines.append(f"            List<Integer> part = {s}.subList(0, 2);")
            self.lines.append("            Collections.sort(part);")
            self.lines.append(f"            System.out.println(part + \"|\" + {s});")
            self.lines.append("        }")
            return
        if kind == "sort":
            src = self.of_type("List<Integer>") + self.of_type("List<String>")
            if not src: return self.decl()
            s = r.choice(src)
            self.lines.append(f"        Collections.sort({s});")
            self.lines.append(f"        System.out.println({s});")
            return

def program(name, seed):
    rng = random.Random(seed)
    g = Gen(rng)
    for _ in range(rng.randint(6, 14)):
        g.stmt()
    body = "\n".join(g.lines)
    return (PRELUDE % name) + body + "\n    }\n}\n"

def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    count = int(args[0]) if args else 100
    base = int(args[1]) if len(args) > 1 else 900
    out = "cases"
    if "--out" in sys.argv:
        out = sys.argv[sys.argv.index("--out") + 1]
    os.makedirs(out, exist_ok=True)
    for i in range(count):
        name = f"Fz{i}"
        with open(os.path.join(out, name + ".java"), "w") as handle:
            handle.write(program(name, base + i))
    print(f"wrote {count} programs to {out}/ (seed {base})")


if __name__ == "__main__":
    main()
