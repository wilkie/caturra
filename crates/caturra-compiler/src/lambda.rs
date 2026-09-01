//! Lambda desugaring.
//!
//! A lambda `x -> body` has no type on its own; its type comes from the
//! context (a "target type"), which must be a *functional interface* —
//! an interface with a single abstract method (the SAM). This pass runs
//! right after parsing and rewrites each lambda whose target type is
//! syntactically available into an anonymous class implementing that
//! interface, reusing the anonymous-class and capture machinery.
//!
//! Target types are read from declaration/field types (`Fn f = x ->
//! ...`), assignment targets, `return` statements, and method-call
//! parameters (single-candidate resolution). A lambda in any other
//! position is left in place and reported by codegen.

use std::collections::HashMap;

use crate::ast::{ClassDecl, CompilationUnit, Expr, LambdaBody, MethodDecl, Param, Stmt, TypeRef};

/// A functional interface's single abstract method.
#[derive(Clone)]
struct Sam {
    method: String,
    params: Vec<TypeRef>,
    ret: TypeRef,
}

/// Rewrite every target-typed lambda into an anonymous class.
///
/// The returned diagnostics are the few call-shape errors only this pass can
/// see: generic type arguments are erased before codegen, so a pre-built
/// function variable whose declared arguments don't fit the call is checked
/// here, where the declaration is still visible.
#[allow(clippy::too_many_lines)] // one context build per walked scope
pub fn desugar_lambdas(
    units: &mut [(String, CompilationUnit)],
) -> Vec<crate::diagnostics::Diagnostic> {
    let mut diags = Vec::new();
    let sams = functional_interfaces(units);
    let sam_owners = functional_interface_owners(units);
    // Signatures for single-candidate method-argument target typing.
    let methods = method_signatures(units);
    let methods_in_class = method_signatures_by_class(units);
    let generics = generic_signatures(units);
    // Constructor signatures per class, for `new T(…, lambda)` target typing.
    let constructors = constructor_signatures(units);
    let static_methods = static_method_names(units);
    let class_names = class_name_set(units);
    let shapes = method_shapes(units);
    // Each class's DIRECT supertypes, for joining two elements at what they
    // have in common — the same reading codegen does for a ternary's branches.
    let supers: HashMap<String, Vec<String>> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .map(|class| {
            let mut parents: Vec<String> = class.superclass.iter().cloned().collect();
            parents.extend(class.interfaces.iter().cloned());
            (class.name.clone(), parents)
        })
        .collect();
    let hierarchy = class_hierarchy(units);
    let enums = enum_names(units);
    let field_types: HashMap<(String, String), TypeRef> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .flat_map(|class| {
            class
                .fields
                .iter()
                .map(|field| ((class.name.clone(), field.name.clone()), field.ty.clone()))
        })
        .collect();

    // Lambda-class names stay globally unique (one counter), but each
    // synthesized class is appended to the SAME unit its lambda came from, so
    // its `SourceFile` matches the line numbers it carries — otherwise a
    // lambda in a non-first file would get the first file's SourceFile and
    // its breakpoints/stack traces would point at the wrong file.
    // The classes the PARSER hoisted out of an expression — an anonymous class
    // body, a local class. They are walked like any other class here, which
    // loses the scope they were written in: a lambda inside one could not see
    // the enclosing class's fields or the locals the body captures, so
    // `new Supplier<Object>() { … list.stream().map(String::length) … }` was
    // refused for having no functional-interface position, though the same
    // pipeline one line outside compiled. The `new` site records what was in
    // scope; the class body reads it back.
    let hoisted: std::collections::HashSet<String> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .filter(|class| class.is_anonymous || class.is_local)
        .map(|class| class.name.clone())
        .collect();
    let mut captured_scopes: HashMap<String, HashMap<String, TypeRef>> = HashMap::new();
    let mut counter = 0usize;
    for (path, unit) in units.iter_mut() {
        let mut new_classes: Vec<ClassDecl> = Vec::new();
        // The classes a program WROTE first, then the hoisted bodies — and
        // those in reverse, because the parser appends the innermost one
        // first. A body's `new` site records what is in scope there, so the
        // class it creates has to be walked after it: an anonymous class
        // inside an anonymous class was walked BEFORE the one that creates it,
        // and saw nothing.
        let order: Vec<usize> = {
            let mut plain: Vec<usize> = Vec::with_capacity(unit.classes.len());
            let mut nested: Vec<usize> = Vec::new();
            for (index, class) in unit.classes.iter().enumerate() {
                if hoisted.contains(&class.name) {
                    nested.push(index);
                } else {
                    plain.push(index);
                }
            }
            nested.reverse();
            plain.extend(nested);
            plain
        };
        for index in order {
            let class = &mut unit.classes[index];
            let class_name = class.name.clone();
            // The name a FRAME in this class carries: the binary one, so a
            // lambda in a nested class is `Outer$Inner.lambda$go$0`.
            let owner = class
                .binary_name
                .clone()
                .unwrap_or_else(|| class_name.clone());
            let mut lambda_order: Vec<(usize, usize, String, String)> = Vec::new();
            let mut bridges: Vec<MethodDecl> = Vec::new();
            let return_types: Vec<Option<TypeRef>> = class
                .methods
                .iter()
                .map(|m| match &m.return_type {
                    TypeRef::Void => None,
                    other => Some(other.clone()),
                })
                .collect();
            // The enclosing class's fields are in scope for target typing:
            // `vocab.forEach(...)` needs `vocab`'s declared type args, and
            // `vocab` is usually a field.
            let mut fields: HashMap<String, TypeRef> = class
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect();
            // A field a class INHERITS is in scope by its simple name too, and
            // only the class's own were collected — so `cards.stream().map(…)`
            // inside a subclass of the class that declares `cards` had no
            // element and the lambda was refused, though the identical method
            // one class up compiled. Walk the chain outward, keeping the
            // nearest declaration of each name (a subclass field HIDES the
            // one above it).
            {
                let mut above = class.superclass.clone();
                let mut seen: Vec<String> = Vec::new();
                while let Some(parent) = above {
                    if seen.contains(&parent) {
                        break;
                    }
                    for ((owner, name), ty) in &field_types {
                        if *owner == parent {
                            fields.entry(name.clone()).or_insert_with(|| ty.clone());
                        }
                    }
                    seen.push(parent.clone());
                    above = supers
                        .get(&parent)
                        .and_then(|parents| parents.first().cloned());
                }
            }
            // A hoisted body sees what its `new` site saw: the enclosing
            // class's own fields, then the locals in scope there — each only
            // where the body does not shadow it.
            if class.is_anonymous || class.is_local {
                let enclosing = class.enclosing.clone().unwrap_or_default();
                let outer = field_types
                    .iter()
                    .filter(|((owner, _), _)| *owner == enclosing)
                    .map(|((_, name), ty)| (name.clone(), ty.clone()));
                let locals = captured_scopes
                    .get(&class_name)
                    .into_iter()
                    .flatten()
                    .map(|(name, ty)| (name.clone(), ty.clone()));
                for (name, ty) in outer.chain(locals) {
                    fields.entry(name).or_insert(ty);
                }
            }
            for (method, ret) in class.methods.iter_mut().zip(return_types) {
                let params: HashMap<String, TypeRef> = method
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect();
                let mut ctx = Ctx {
                    sams: &sams,
                    sam_owners: &sam_owners,
                    methods: &methods,
                    methods_in_class: &methods_in_class,
                    generics: &generics,
                    constructors: &constructors,
                    static_methods: &static_methods,
                    class_names: &class_names,
                    ret: ret.as_ref(),
                    new_classes: &mut new_classes,
                    counter: &mut counter,
                    scope: vec![fields.clone(), params],
                    fields: &field_types,
                    current_class: Some(class_name.as_str()),
                    frame_owner: (owner.as_str(), &frame_method_name(method)),
                    lambda_order: &mut lambda_order,
                    member_start: source_position(method.span),
                    bridges: &mut bridges,
                    shapes: &shapes,
                    supers: &supers,
                    hierarchy: &hierarchy,
                    enums: &enums,
                    hoisted: &hoisted,
                    captured_scopes: &mut captured_scopes,
                    class_prefix: crate::LAMBDA_CLASS_PREFIX,
                    path,
                    diags: &mut diags,
                };
                for stmt in &mut method.body {
                    desugar_stmt(stmt, &mut ctx);
                }
            }
            // An INITIALIZER BLOCK holds statements like any method body, and
            // a lambda written in one had nowhere to be desugared: this pass
            // walked methods and field initializers only, so
            // `static { list.forEach(x -> …); }` was refused for having no
            // functional-interface position. The capture pass already walked
            // them, which is why the failure looked like a target-typing one.
            for block in &mut class.init_blocks {
                let mut ctx = Ctx {
                    sams: &sams,
                    sam_owners: &sam_owners,
                    methods: &methods,
                    methods_in_class: &methods_in_class,
                    generics: &generics,
                    constructors: &constructors,
                    static_methods: &static_methods,
                    class_names: &class_names,
                    ret: None,
                    new_classes: &mut new_classes,
                    counter: &mut counter,
                    scope: vec![fields.clone()],
                    fields: &field_types,
                    current_class: Some(class_name.as_str()),
                    frame_owner: (
                        owner.as_str(),
                        if block.is_static { "static" } else { "new" },
                    ),
                    lambda_order: &mut lambda_order,
                    member_start: source_position(block.span),
                    bridges: &mut bridges,
                    shapes: &shapes,
                    supers: &supers,
                    hierarchy: &hierarchy,
                    enums: &enums,
                    hoisted: &hoisted,
                    captured_scopes: &mut captured_scopes,
                    class_prefix: crate::LAMBDA_CLASS_PREFIX,
                    path,
                    diags: &mut diags,
                };
                for stmt in &mut block.body {
                    desugar_stmt(stmt, &mut ctx);
                }
            }
            for field in &mut class.fields {
                if let Some(init) = &mut field.init {
                    let mut ctx = Ctx {
                        sams: &sams,
                        sam_owners: &sam_owners,
                        methods: &methods,
                        methods_in_class: &methods_in_class,
                        generics: &generics,
                        constructors: &constructors,
                        static_methods: &static_methods,
                        class_names: &class_names,
                        ret: None,
                        new_classes: &mut new_classes,
                        counter: &mut counter,
                        scope: vec![HashMap::new()],
                        fields: &field_types,
                        current_class: Some(class_name.as_str()),
                        frame_owner: (
                            owner.as_str(),
                            if field.is_static { "static" } else { "new" },
                        ),
                        lambda_order: &mut lambda_order,
                        member_start: source_position(field.span),
                        bridges: &mut bridges,
                        shapes: &shapes,
                        supers: &supers,
                        hierarchy: &hierarchy,
                        enums: &enums,
                        hoisted: &hoisted,
                        captured_scopes: &mut captured_scopes,
                        class_prefix: crate::LAMBDA_CLASS_PREFIX,
                        path,
                        diags: &mut diags,
                    };
                    desugar_expr(init, Some(&field.ty), &mut ctx);
                }
            }
            class.methods.append(&mut bridges);
            // Hand out javac's numbers now that the whole class is walked:
            // members in SOURCE order, and within one, the order each
            // translation finished (innermost lambda first).
            lambda_order.sort_by_key(|(member, seq, _, _)| (*member, *seq));
            for (index, (_, _, synthesized, method)) in lambda_order.iter().enumerate() {
                if let Some(decl) = new_classes.iter_mut().find(|c| &c.name == synthesized) {
                    decl.trace_name = Some(format!("{owner}.lambda${method}${index}"));
                }
            }
        }
        // The class iteration's borrow of `unit.classes` is released here, so
        // this unit's synthesized lambda classes can be appended to it.
        unit.classes.append(&mut new_classes);
    }
    diags
}

struct Ctx<'a> {
    sams: &'a HashMap<String, Sam>,
    methods: &'a HashMap<String, Vec<Vec<TypeRef>>>,
    /// The same, keyed by (class, method), for when the receiver's class is
    /// known — the name-only map cannot separate two interfaces that declare
    /// one method name with different parameter types.
    methods_in_class: &'a HashMap<(String, String), Vec<Vec<TypeRef>>>,
    /// Generic methods' parameter types as WRITTEN, for putting a type
    /// variable back into a lambda argument's target type.
    generics: &'a HashMap<String, Vec<GenericSig>>,
    /// Class name -> its constructors' parameter-type lists, for target
    /// typing a lambda passed to `new T(…)`.
    constructors: &'a HashMap<String, Vec<Vec<TypeRef>>>,
    /// Class name -> its static method names, for method-reference
    /// static-vs-instance disambiguation.
    static_methods: &'a HashMap<String, std::collections::HashSet<String>>,
    /// All class/interface names (user + known library types).
    class_names: &'a std::collections::HashSet<String>,
    ret: Option<&'a TypeRef>,
    new_classes: &'a mut Vec<ClassDecl>,
    counter: &'a mut usize,
    /// Local-variable types, for assignment-target typing.
    scope: Vec<HashMap<String, TypeRef>>,
    /// Every class's fields, by (class, field) — a method reference assigned
    /// to ANOTHER object's field (`q.stored = q::add`) needs the declared
    /// type to target-type it.
    fields: &'a HashMap<(String, String), TypeRef>,
    /// The class whose body is being walked, for `this.field` targets.
    current_class: Option<&'a str>,
    /// How a stack-trace frame in the member being walked is written by javac:
    /// the enclosing class's BINARY name and the synthetic method's middle
    /// part — a method's own name, `new` for a constructor or instance
    /// initializer, `static` for a static one.
    frame_owner: (&'a str, &'a str),
    /// Which interface each [`Ctx::sams`] key names — see
    /// [`functional_interface_owners`].
    sam_owners: &'a HashMap<String, String>,
    /// Where that member starts in the source, and a counter of the lambdas
    /// synthesized so far in this class. javac numbers a class's lambdas in
    /// SOURCE order of the members, and within one, innermost FIRST (it
    /// numbers as each translation finishes) — so the pair is recorded here
    /// and the numbers handed out once the class is walked.
    lambda_order: &'a mut Vec<(usize, usize, String, String)>,
    member_start: usize,
    /// Declared methods per class, for method-reference validation.
    shapes: &'a HashMap<String, Vec<MethodShape>>,
    /// Each class's DIRECT supertypes, for joining two element types.
    supers: &'a HashMap<String, Vec<String>>,
    /// Each class's type parameters and the ARGUMENTS it writes on its own
    /// supertypes, for reading a type variable a supertype owns off a
    /// subclass receiver (`class SBox implements Box<String>` answers `T`).
    hierarchy: &'a HashMap<String, ClassGenerics>,
    /// The `enum` classes, which have no accessible constructor.
    enums: &'a std::collections::HashSet<String>,
    /// The classes the parser HOISTED out of an expression (an anonymous class
    /// body, a local class), and what was in scope at each one's `new` site.
    /// A hoisted body is walked like any other class, which loses the scope it
    /// was written in.
    hoisted: &'a std::collections::HashSet<String>,
    captured_scopes: &'a mut HashMap<String, HashMap<String, TypeRef>>,
    /// The prefix for the next synthesized class — a method REFERENCE gets
    /// its own, because its captures follow different rules (see
    /// `crate::METHOD_REF_CLASS_PREFIX`).
    class_prefix: &'static str,
    /// Methods to append to the enclosing class: a `super::m` reference
    /// needs one, because the synthesized lambda class cannot itself make a
    /// non-virtual call on another object's superclass (javac writes the
    /// same kind of bridge).
    bridges: &'a mut Vec<MethodDecl>,
    /// The unit's source path, for diagnostics.
    path: &'a str,
    /// Call-shape errors only this pass can see (declared generic arguments
    /// are erased before codegen).
    diags: &'a mut Vec<crate::diagnostics::Diagnostic>,
}

impl Ctx<'_> {
    fn lookup(&self, name: &str) -> Option<TypeRef> {
        self.scope.iter().rev().find_map(|f| f.get(name).cloned())
    }
}

/// An abstract interface method whose signature matches a PUBLIC method of
/// `java.lang.Object` — `toString()`, `hashCode()`, `equals(Object)`. Any
/// implementation inherits these from Object, so JLS §9.8 excludes them when
/// deciding whether an interface is functional.
fn is_object_method_redeclaration(m: &MethodDecl) -> bool {
    match (m.name.as_str(), m.params.as_slice()) {
        ("toString" | "hashCode", []) => true,
        ("equals", [p]) => matches!(
            &p.ty,
            TypeRef::Named(n) if n == "Object" || n == "java.lang.Object"
        ),
        _ => false,
    }
}

/// The interface each [`functional_interfaces`] key belongs to, by BINARY
/// name. Two nested interfaces may share a simple name, and the map holds one
/// SAM per SPELLING — this is what says whether two spellings are the same
/// interface, which is the difference between a scoped lookup that must
/// override the written name and one that must leave it alone.
fn functional_interface_owners(units: &[(String, CompilationUnit)]) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            if !class.is_interface {
                continue;
            }
            let binary = class
                .binary_name
                .clone()
                .unwrap_or_else(|| class.name.clone());
            if let Some(dotted) = class.binary_name.as_deref().map(|b| b.replace('$', ".")) {
                let mut rest = dotted.as_str();
                loop {
                    out.insert(String::from(rest), binary.clone());
                    match rest.split_once('.') {
                        Some((_, tail)) if tail.contains('.') => rest = tail,
                        _ => break,
                    }
                }
                out.entry(class.name.clone())
                    .or_insert_with(|| binary.clone());
            } else {
                out.insert(class.name.clone(), binary);
            }
        }
    }
    out
}

/// The functional interfaces in the program: interface name -> its SAM.
fn functional_interfaces(units: &[(String, CompilationUnit)]) -> HashMap<String, Sam> {
    let mut out = HashMap::new();
    // `AutoCloseable`/`Closeable` are library interfaces, so they are not among
    // the units — but they ARE functional (one abstract `close()`), and
    // `try (AutoCloseable a = () -> ...)` is the shape that notices. A user
    // interface of the same name overwrites this below, as it should.
    for name in ["AutoCloseable", "Closeable"] {
        out.insert(
            String::from(name),
            Sam {
                method: String::from("close"),
                params: Vec::new(),
                ret: TypeRef::Void,
            },
        );
    }
    for (_, unit) in units {
        for class in &unit.classes {
            if !class.is_interface {
                continue;
            }
            let abstract_methods: Vec<&MethodDecl> = class
                .methods
                .iter()
                .filter(|m| m.is_abstract && !m.is_static)
                // JLS §9.8: an abstract redeclaration of a public Object
                // method (Comparator redeclares equals this way) does not
                // count toward the single abstract method.
                .filter(|m| !is_object_method_redeclaration(m))
                .collect();
            if abstract_methods.len() == 1 {
                let m = abstract_methods[0];
                let sam = Sam {
                    method: m.name.clone(),
                    params: m.params.iter().map(|p| p.ty.clone()).collect(),
                    ret: m.return_type.clone(),
                };
                // A NESTED interface is hoisted to the top level under its
                // simple name, but source names it through its enclosing type
                // — `Outer.Inner`, which javac REQUIRES from outside `Outer`.
                // So it is keyed under every suffix of its binary name:
                // `Main$H$Inner` answers to `Main.H.Inner`, `H.Inner` and
                // `Inner`. Keying it only by the simple name refused every
                // lambda whose target was a nested interface — while the
                // anonymous-class form of the same target compiled, which is
                // what made the gap look like a rule about lambdas.
                //
                // The qualified keys are inserted outright; the SIMPLE name is
                // only filled in if nothing holds it, so a top-level interface
                // keeps its own name when a nested one shares it (codegen
                // already tells those two apart — this map was the one place
                // that could not).
                let dotted = class
                    .binary_name
                    .as_deref()
                    .map(|binary| binary.replace('$', "."));
                if let Some(dotted) = &dotted {
                    let mut rest = dotted.as_str();
                    loop {
                        out.insert(String::from(rest), sam.clone());
                        match rest.split_once('.') {
                            Some((_, tail)) if tail.contains('.') => rest = tail,
                            _ => break,
                        }
                    }
                    out.entry(class.name.clone()).or_insert_with(|| sam.clone());
                } else {
                    out.insert(class.name.clone(), sam);
                }
            }
        }
    }
    // An interface may declare NO abstract method of its own and still be
    // functional by INHERITING one: `interface Sub extends Op { }` is a target
    // for `Sub f = x -> x + 1`, and caturra refused every such lambda for
    // having no functional-interface position. Repeated to a fixed point, so a
    // chain of extending interfaces all resolve; an interface that inherits
    // two DIFFERENT SAMs is not functional and is left out.
    loop {
        let mut added = false;
        for (_, unit) in units {
            for class in &unit.classes {
                if !class.is_interface || out.contains_key(&class.name) {
                    continue;
                }
                if class
                    .methods
                    .iter()
                    .any(|m| m.is_abstract && !m.is_static && !is_object_method_redeclaration(m))
                {
                    continue;
                }
                let mut inherited: Vec<&Sam> = class
                    .interfaces
                    .iter()
                    .filter_map(|name| out.get(name))
                    .collect();
                inherited.dedup_by(|a, b| a.method == b.method && a.params.len() == b.params.len());
                if let [only] = inherited[..] {
                    let sam = only.clone();
                    out.insert(class.name.clone(), sam);
                    added = true;
                }
            }
        }
        if !added {
            break;
        }
    }
    out
}

/// Method name -> the parameter-type lists of each declaration, for
/// single-candidate target typing of a lambda argument.
/// Class name -> its declared static method names.
/// What method-reference validation needs to know about one declared method.
struct MethodShape {
    name: String,
    is_static: bool,
    arity: usize,
    /// Declared with a trailing `T...`. A varargs method applies at EVERY
    /// arity from `arity - 1` upward (JLS §15.12.2.4), so a reference to one
    /// fits a functional interface of any matching shape — checking `arity`
    /// alone rejected `Q::pack` for a two-argument SAM.
    is_varargs: bool,
    /// The checked exceptions the method declares, as written.
    throws: Vec<String>,
    /// The declared RETURN type. A call to a method that returns a collection
    /// is a receiver like any other — `make().forEach(v -> …)` — and without
    /// this the pass had no element for the lambda.
    return_type: TypeRef,
    /// The method's OWN type parameters, in order, so an explicit witness
    /// (`W.<String>box(x)`) can be substituted into the return type. Without
    /// them the return stays `List<T>`, and a lambda written against it has
    /// no element.
    type_params: Vec<String>,
}

impl MethodShape {
    /// Whether the method can be called with exactly `count` arguments.
    fn takes(&self, count: usize) -> bool {
        self.arity == count || (self.is_varargs && count + 1 >= self.arity)
    }
}

/// Every class's declared methods, for the JLS §15.13.1 checks a method
/// reference needs: which of the four forms it is, whether that form is
/// ambiguous, and whether its thrown types fit the functional interface.
fn method_shapes(units: &[(String, CompilationUnit)]) -> HashMap<String, Vec<MethodShape>> {
    let mut out: HashMap<String, Vec<MethodShape>> = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            let entry = out.entry(class.name.clone()).or_default();
            for method in &class.methods {
                if method.is_constructor {
                    continue;
                }
                entry.push(MethodShape {
                    name: method.name.clone(),
                    is_static: method.is_static,
                    arity: method.params.len(),
                    is_varargs: method.params.last().is_some_and(|p| p.is_varargs),
                    throws: method.throws.clone(),
                    return_type: method.return_type.clone(),
                    type_params: method
                        .type_params
                        .iter()
                        .map(|param| param.name.clone())
                        .collect(),
                });
            }
        }
    }
    out
}

/// The classes declared as `enum` — `E::new` is "enum types may not be
/// instantiated", not an arity complaint.
fn enum_names(units: &[(String, CompilationUnit)]) -> std::collections::HashSet<String> {
    units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .filter(|class| class.is_enum)
        .map(|class| class.name.clone())
        .collect()
}

fn static_method_names(
    units: &[(String, CompilationUnit)],
) -> HashMap<String, std::collections::HashSet<String>> {
    let mut out: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            let entry = out.entry(class.name.clone()).or_default();
            for method in &class.methods {
                if method.is_static {
                    entry.insert(method.name.clone());
                }
            }
        }
    }
    out
}

/// Every class/interface name, plus the library types that can qualify
/// a method reference.
/// The library CONTAINER types a method reference may name as its qualifier —
/// `List::stream`, `Map::size`, `Optional::get`. Each is generic, so the
/// receiver parameter is deliberately left UNTYPED where a `String::length`
/// gets `String`: the functional interface supplies the element type with its
/// type arguments intact, and the bare name would be the raw type, whose every
/// method answers `Object`.
const LIBRARY_CONTAINERS: [&str; 16] = [
    "ArrayList",
    "LinkedList",
    "List",
    "Collection",
    "Set",
    "HashSet",
    "LinkedHashSet",
    "TreeSet",
    "Map",
    "HashMap",
    "LinkedHashMap",
    "TreeMap",
    "Queue",
    "Deque",
    "Optional",
    // `Enum::name` / `Enum::ordinal` — the supertype every enum constant has.
    // A method reference through it is the ordinary way to stream an enum's
    // names, and without the name here it was not a class at all: "cannot find
    // symbol: 'Enum'".
    "Enum",
];

/// A member's start, as one sortable number — javac numbers a class's lambdas
/// in the order the members are WRITTEN, and this pass walks methods, then
/// initializer blocks, then field initializers.
fn source_position(span: crate::diagnostics::SourceSpan) -> usize {
    span.start.line as usize * 100_000 + span.start.column as usize
}

/// The middle part of javac's synthetic lambda name for a method: its own
/// name, `new` for a constructor (and an instance initializer), `static` for a
/// static one.
fn frame_method_name(method: &MethodDecl) -> String {
    if method.is_constructor {
        String::from("new")
    } else {
        method.name.clone()
    }
}

fn class_name_set(units: &[(String, CompilationUnit)]) -> std::collections::HashSet<String> {
    let mut set: std::collections::HashSet<String> = units
        .iter()
        .flat_map(|(_, u)| u.classes.iter().map(|c| c.name.clone()))
        .collect();
    for lib in [
        "String",
        "Integer",
        "Double",
        "Long",
        "Float",
        "Short",
        "Byte",
        "Character",
        "Boolean",
        "Math",
        "System",
        "Object",
        "StringBuilder",
        // The interface `String` and `StringBuilder` share: a program writes
        // `CharSequence::length` in a stream over a `List<CharSequence>`, and
        // a qualifier that names no class here fell to the BOUND form, where
        // the name itself does not resolve.
        "CharSequence",
        // The regex objects. `MatchResult::group` is how a program reads a
        // `results()` stream, and a qualifier missing from here is not a class
        // at all: "cannot find symbol: 'MatchResult'".
        "Pattern",
        "Matcher",
        "MatchResult",
        // The filesystem objects: `File::getName` is how a program turns a
        // directory listing into names, and the qualifier has to name a class
        // here or the reference is not one.
        "File",
        "Path",
        // `Collections::unmodifiableSet` and friends — the algorithms class,
        // which is a method reference's qualifier as readily as any other and
        // is exactly what a `collectingAndThen` finisher usually names.
        "Collections",
        "Objects",
    ]
    .into_iter()
    .chain(LIBRARY_CONTAINERS)
    {
        set.insert(String::from(lib));
    }
    set
}

/// Whether `method` is a static method of the library type `class`
/// (a curated set; used only for method-reference disambiguation).
fn is_library_static(class: &str, method: &str) -> bool {
    // The statics of the classes a method REFERENCE is written on. Judged by
    // NAME alone below, which is why `Arrays::stream` — the ordinary way to
    // flatten a grid — read as an instance call on a row and was "cannot find
    // symbol: method stream() in variable __p0 of type String[]".
    let by_class = match class {
        "Arrays" => matches!(
            method,
            "stream" | "asList" | "toString" | "deepToString" | "hashCode" | "deepHashCode"
        ),
        "Character" => matches!(
            method,
            "isDigit"
                | "isLetter"
                | "isLetterOrDigit"
                | "isUpperCase"
                | "isLowerCase"
                | "isWhitespace"
                | "isSpaceChar"
                | "isAlphabetic"
                | "toUpperCase"
                | "toLowerCase"
                | "getNumericValue"
        ),
        "String" => matches!(method, "join" | "format" | "copyValueOf"),
        "Objects" => matches!(
            method,
            "isNull"
                | "nonNull"
                | "requireNonNull"
                | "requireNonNullElse"
                | "requireNonNullElseGet"
                | "hash"
                | "toString"
        ),
        "Integer" | "Long" | "Short" | "Byte" => matches!(
            method,
            "toBinaryString" | "toHexString" | "toOctalString" | "bitCount" | "signum"
        ),
        "Boolean" => matches!(method, "logicalAnd" | "logicalOr" | "logicalXor"),
        // The read-only WRAPPERS are here because they are what a
        // `collectingAndThen` finisher is: `collectingAndThen(toSet(),
        // Collections::unmodifiableSet)` is how a stream gathers into a set
        // nobody can change.
        "Collections" => matches!(
            method,
            "reverseOrder"
                | "emptyList"
                | "emptySet"
                | "emptyMap"
                | "unmodifiableList"
                | "unmodifiableSet"
                | "unmodifiableMap"
                | "unmodifiableCollection"
                | "unmodifiableSortedSet"
                | "unmodifiableSortedMap"
                | "max"
                | "min"
                | "nCopies"
                | "singleton"
                | "singletonList"
                | "frequency"
        ),
        // `Pattern::compile` and `Pattern::quote` are statics; `matcher`,
        // `split` and the predicates beside them are not.
        "Pattern" => matches!(method, "compile" | "quote" | "matches"),
        "Matcher" => method == "quoteReplacement",
        _ => false,
    };
    by_class
        || matches!(
            method,
            "parseInt"
                | "parseLong"
                | "parseDouble"
                | "parseFloat"
                | "parseShort"
                | "parseByte"
                | "parseBoolean"
                | "valueOf"
                | "abs"
                | "max"
                | "min"
                | "sqrt"
                | "cbrt"
                | "pow"
                | "floor"
                | "ceil"
                | "round"
                | "random"
                | "signum"
                | "sum"
                | "compare"
                | "sin"
                | "cos"
                | "tan"
                | "log"
                | "exp"
        )
}

/// Class name -> each constructor's parameter-type list, for target typing
/// a lambda passed to `new T(…)`.
fn constructor_signatures(
    units: &[(String, CompilationUnit)],
) -> HashMap<String, Vec<Vec<TypeRef>>> {
    let mut out: HashMap<String, Vec<Vec<TypeRef>>> = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            for method in &class.methods {
                if method.is_constructor {
                    out.entry(class.name.clone())
                        .or_default()
                        .push(method.params.iter().map(|p| p.ty.clone()).collect());
                }
            }
        }
    }
    out
}

/// The same signatures, keyed by (class, method). The name-only map cannot
/// tell `__UnaryOperator.andThen` from `__Consumer.andThen` — three interfaces
/// declare that name with different parameter types, so they disagree and the
/// argument goes untyped. When the receiver's class is known, ask this first.
/// The target type for a combinator's single lambda argument, read off the
/// RECEIVER's declared type arguments. `Function<A, B>.andThen(g)` gives `g`
/// the parameter type `B`; `Predicate<T>.and(p)` gives `p` the type `T`.
/// `None` for anything else, which keeps the ordinary lookup.
/// The declared functional type an expression answers, when it is knowable.
/// A local variable gives its own; a PREDICATE COMBINATOR gives its receiver's,
/// because `negate`, `and` and `or` all answer a `Predicate<T>` over the same
/// element — as does `Predicate.not(p)`. Reading only a bare name meant that
/// `p.negate().and(s -> s.length() == 1)` typed the second lambda's parameter
/// `Object`, and calling anything on it was then refused.
///
/// Deliberately NOT extended to `Function.andThen`, whose result type is the
/// composed function's, not the receiver's.
fn functional_type_of(expr: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    match expr {
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0]),
        Expr::Call {
            receiver: Some(inner),
            method,
            args,
            ..
        } => {
            if is_negated_predicate(expr) {
                return functional_type_of(&args[0], ctx);
            }
            let preserving = (method == "negate" && args.is_empty())
                || (matches!(method.as_str(), "and" | "or") && args.len() == 1);
            // The receiver's own type decides: only a `Predicate` has these,
            // so a user class that happens to declare `and` answers its own
            // type here and falls out of the caller's match.
            preserving
                .then(|| functional_type_of(inner, ctx))
                .flatten()
                .filter(|ty| matches!(ty, TypeRef::Generic { base, .. } if simple_base(base) == "Predicate"))
        }
        // The receiver may already have been rewritten: the desugaring turns
        // `Predicate.not(p)` into the bundled `__Negate`, and it still stands
        // for a predicate over `p`'s element.
        Expr::NewObject { class, args, .. } if class == "__Negate" && args.len() == 1 => {
            functional_type_of(&args[0], ctx)
        }
        _ => None,
    }
}

fn combinator_argument_type(
    receiver: &Expr,
    method: &str,
    arity: usize,
    ctx: &Ctx,
) -> Option<TypeRef> {
    if arity != 1 {
        return None;
    }
    let receiver_ty = functional_type_of(receiver, ctx)?;
    // The PRIMITIVE specializations take no type arguments, so their
    // combinators are the simplest case of all: `IntPredicate.and`,
    // `IntConsumer.andThen` and `IntUnaryOperator.andThen`/`compose` each take
    // the very interface they are called on. Reading only a PARAMETERIZED
    // receiver skipped the whole family, and an inline lambda argument was then
    // refused for having no functional target.
    if let TypeRef::Named(name) = &receiver_ty {
        let simple = name.rsplit('.').next().unwrap_or(name);
        let same_shape = match method {
            "and" | "or" => simple.ends_with("Predicate"),
            "andThen" => simple.ends_with("Consumer") || simple.ends_with("UnaryOperator"),
            "compose" => simple.ends_with("UnaryOperator"),
            _ => false,
        };
        return same_shape.then(|| receiver_ty.clone());
    }
    let TypeRef::Generic { base, args } = receiver_ty else {
        return None;
    };
    let object = || TypeRef::Named(String::from("Object"));
    match (simple_base(&base), method) {
        // The result of `this` is what the next function receives.
        ("Function" | "UnaryOperator" | "BiFunction" | "BinaryOperator", "andThen") => {
            let result = args.last()?.clone();
            Some(TypeRef::Generic {
                base: String::from("Function"),
                args: vec![result, object()],
            })
        }
        // Both halves of a predicate see the same element.
        ("Predicate", "and" | "or") => Some(TypeRef::Generic {
            base: String::from("Predicate"),
            args: vec![args.first()?.clone()],
        }),
        // Both consumers see the same element.
        ("Consumer", "andThen") => Some(TypeRef::Generic {
            base: String::from("Consumer"),
            args: vec![args.first()?.clone()],
        }),
        // The two-argument shapes: both halves see the same PAIR.
        ("BiPredicate", "and" | "or") | ("BiConsumer", "andThen") => Some(TypeRef::Generic {
            base: String::from(simple_base(&base)),
            args: args.clone(),
        }),
        _ => None,
    }
}

/// The class a call's receiver denotes, when that is knowable from its
/// declared type — a local, or `this`. Library and computed receivers stay
/// unknown and fall back to the name-only lookup.
/// Whether `expr` names the library class `want`, written plainly (`Arrays`)
/// or FULLY QUALIFIED (`java.util.Arrays`).
///
/// Every receiver test here matched a one-segment path, so a qualified
/// `java.util.Arrays.asList(1, 2).stream().filter(v -> ...)` lost the element
/// type and then refused the lambda for having no target.
fn names_library_class(expr: &Expr, want: &str) -> bool {
    let Expr::Name { path, .. } = expr else {
        return false;
    };
    match path.split_last() {
        Some((last, [])) => last == want,
        Some((last, prefix)) => {
            last == want && matches!(prefix.first().map(String::as_str), Some("java" | "javax"))
        }
        None => false,
    }
}

fn receiver_class_name(receiver: &Expr, ctx: &Ctx) -> Option<String> {
    let Expr::Name { path, .. } = receiver else {
        return None;
    };
    if path.len() != 1 {
        return None;
    }
    let declared = ctx.lookup(&path[0])?;
    let base = match &declared {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Generic { base, .. } => base.clone(),
        _ => return None,
    };
    let simple = simple_base(&base).to_owned();
    Some(functional_erased_name(&simple).unwrap_or(simple))
}

/// The bundled interface a `java.util.function` name aliases, mirroring
/// codegen's own mapping.
fn functional_erased_name(simple: &str) -> Option<String> {
    Some(String::from(match simple {
        "Comparator" => "__Comparator",
        "Function" | "UnaryOperator" => "__UnaryOperator",
        "BiFunction" | "BinaryOperator" => "__BiFunction",
        "Predicate" => "__Predicate",
        "Consumer" => "__Consumer",
        "BiConsumer" => "__BiConsumer",
        "Supplier" => "__Supplier",
        "Runnable" => "__Runnable",
        _ => return None,
    }))
}

/// A generic method's parameter types AS WRITTEN, with the plan for pinning
/// each of its type variables from the arguments. Keyed by (class, name) and
/// by name, like the erased signatures beside it.
#[derive(Clone)]
struct GenericSig {
    params: Vec<TypeRef>,
    /// Every type variable in scope for the declaration. A parameter that
    /// mentions one this call could not pin is left to the ERASED signature:
    /// a half-substituted target reaches codegen as a name nothing declares
    /// ("unknown type 'R'"), which is a worse answer than the erasure.
    vars: Vec<String>,
    sources: Vec<(String, Vec<crate::ast::InferSource>)>,
    /// The type parameters of the DECLARING class, so a variable the class
    /// owns can be pinned from the receiver's own type arguments instead.
    class_params: Vec<String>,
    /// That class's NAME. A receiver is often a SUBTYPE of it
    /// (`SBox implements Box<String>`, calling a `default` method of `Box`),
    /// and then the arguments are read off the subclass's own `implements`
    /// clause rather than off the receiver's written type.
    owner: String,
    /// The method's OWN type parameters, in the order an explicit witness
    /// gives them (`W.<String>box(v)`), and the return type they appear in.
    own_params: Vec<String>,
    /// The parameter types after ERASURE, beside `params` as written. A
    /// variable this call cannot pin takes its erased form from here rather
    /// than abandoning the whole target: `<R> List<R> mapped(Function<T, R>)`
    /// pins `T` from the receiver and can never pin `R`, whose only source is
    /// the lambda's own body — and the target that types the lambda's
    /// PARAMETER does not need it.
    erased: Vec<TypeRef>,
    return_type: TypeRef,
}

fn generic_signatures(units: &[(String, CompilationUnit)]) -> HashMap<String, Vec<GenericSig>> {
    let mut out: HashMap<String, Vec<GenericSig>> = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            let class_params: Vec<String> =
                class.type_params.iter().map(|tp| tp.name.clone()).collect();
            for method in &class.methods {
                if method.is_constructor || method.declared_params.is_empty() {
                    continue;
                }
                let mut vars: Vec<String> = method
                    .type_params
                    .iter()
                    .map(|tp| tp.name.clone())
                    .collect();
                vars.extend(class_params.iter().cloned());
                out.entry(method.name.clone())
                    .or_default()
                    .push(GenericSig {
                        params: method.declared_params.clone(),
                        erased: method.params.iter().map(|p| p.ty.clone()).collect(),
                        sources: method.type_var_sources.clone(),
                        class_params: class_params.clone(),
                        owner: class.name.clone(),
                        own_params: method
                            .type_params
                            .iter()
                            .map(|tp| tp.name.clone())
                            .collect(),
                        return_type: method
                            .declared_return
                            .clone()
                            .unwrap_or_else(|| method.return_type.clone()),
                        vars,
                    });
            }
            // A method of a GENERIC CLASS that mentions the class's own
            // variable — `class Holder<T> { void each(Consumer<T> c) }`. It
            // declares no type parameters of its own, so nothing above records
            // it, and the receiver is what pins `T`.
            if class_params.is_empty() {
                continue;
            }
            for method in &class.methods {
                if method.is_constructor || !method.declared_params.is_empty() {
                    continue;
                }
                let written: Vec<TypeRef> = method.params.iter().map(|p| p.ty.clone()).collect();
                if written.iter().any(|ty| mentions_any(ty, &class_params)) {
                    out.entry(method.name.clone())
                        .or_default()
                        .push(GenericSig {
                            params: written.clone(),
                            erased: written,
                            sources: Vec::new(),
                            class_params: class_params.clone(),
                            owner: class.name.clone(),
                            own_params: Vec::new(),
                            return_type: method
                                .declared_return
                                .clone()
                                .unwrap_or_else(|| method.return_type.clone()),
                            vars: class_params.clone(),
                        });
                }
            }
        }
    }
    out
}

/// Whether a written type mentions any of `names` as a type argument or as
/// itself.
fn mentions_any(ty: &TypeRef, names: &[String]) -> bool {
    match ty {
        TypeRef::Named(name) => names.iter().any(|n| n == name),
        TypeRef::Generic { base, args } => {
            names.iter().any(|n| n == base) || args.iter().any(|a| mentions_any(a, names))
        }
        TypeRef::Array(inner) => mentions_any(inner, names),
        _ => false,
    }
}

/// A class's own type parameters and the type arguments it writes on each of
/// its direct supertypes.
struct ClassGenerics {
    params: Vec<String>,
    supers: Vec<(String, Vec<TypeRef>)>,
}

fn class_hierarchy(units: &[(String, CompilationUnit)]) -> HashMap<String, ClassGenerics> {
    let mut out = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            let supers = class
                .superclass
                .iter()
                .chain(class.interfaces.iter())
                .map(|parent| {
                    let args = class
                        .supertype_args
                        .iter()
                        .find(|(name, _)| name == parent)
                        .map_or_else(Vec::new, |(_, args)| args.clone());
                    (parent.clone(), args)
                })
                .collect();
            out.insert(
                class.name.clone(),
                ClassGenerics {
                    params: class.type_params.iter().map(|tp| tp.name.clone()).collect(),
                    supers,
                },
            );
        }
    }
    out
}

/// The type arguments `owner` is given, seen from `class` parameterized with
/// `args` — the same walk codegen makes for an inherited RETURN type, here for
/// a lambda PARAMETER. `SBox implements Box<String>` answers `[String]` for
/// `Box`, and an intermediate class substitutes as it goes, so
/// `SBox extends Mid<String>` where `Mid<T> implements Box<T>` answers the
/// same. `None` when the supertype is reached raw, or not reached at all.
fn inherited_arguments(
    class: &str,
    args: &[TypeRef],
    owner: &str,
    ctx: &Ctx,
    depth: usize,
) -> Option<Vec<TypeRef>> {
    if class == owner {
        return (!args.is_empty()).then(|| args.to_vec());
    }
    // A cyclic `extends` is a program error resolution reports; this pass only
    // has to not hang on one.
    if depth > 16 {
        return None;
    }
    let info = ctx.hierarchy.get(class)?;
    let bound: HashMap<String, TypeRef> = info
        .params
        .iter()
        .cloned()
        .zip(args.iter().cloned())
        .collect();
    info.supers.iter().find_map(|(parent, written)| {
        let substituted: Vec<TypeRef> = written
            .iter()
            .map(|ty| substitute_vars(ty, &bound, &info.params).unwrap_or_else(|| ty.clone()))
            .collect();
        inherited_arguments(parent, &substituted, owner, ctx, depth + 1)
    })
}

/// The type arguments the RECEIVER gives the class that declares the method
/// being called: written on the receiver's own type where it names that class,
/// and inherited from its `extends`/`implements` clause where it does not.
fn receiver_class_arguments(receiver: &Expr, owner: &str, ctx: &Ctx) -> Option<Vec<TypeRef>> {
    match static_type_of(receiver, ctx)? {
        TypeRef::Generic { base, args } if base == owner => Some(args),
        TypeRef::Generic { base, args } => inherited_arguments(&base, &args, owner, ctx, 0)
            // A receiver written with arguments for a class this pass cannot
            // place still pins by POSITION, which is what it did before the
            // walk existed.
            .or(Some(args)),
        TypeRef::Named(name) => inherited_arguments(&name, &[], owner, ctx, 0),
        _ => None,
    }
}

/// Bind a generic method's type variables for one call: from an explicit
/// WITNESS where the call gives one, from the arguments where it does not, and
/// from the receiver's own type arguments for a variable the declaring class
/// owns (`Holder<String> h; h.each(c)` pins `T` with no argument mentioning
/// it). A variable nothing pins is simply absent, and the caller falls back to
/// the erased signature.
fn pinned_vars(
    sig: &GenericSig,
    receiver: Option<&Expr>,
    args: &[Expr],
    witness: &[TypeRef],
    ctx: &Ctx,
) -> HashMap<String, TypeRef> {
    use crate::ast::InferSource;
    let mut bound: HashMap<String, TypeRef> = HashMap::new();
    // The witness is the programmer saying it outright, so it wins over what
    // the arguments would have inferred (JLS §15.12.2.1 does not infer at all
    // when one is written).
    if witness.len() == sig.own_params.len() {
        for (name, actual) in sig.own_params.iter().zip(witness) {
            bound.insert(name.clone(), actual.clone());
        }
    }
    for (var, sources) in &sig.sources {
        if bound.contains_key(var) {
            continue;
        }
        let pinned = sources.iter().find_map(|source| match source {
            InferSource::Direct(index) => static_type_of(args.get(*index)?, ctx),
            InferSource::Element(index) => list_elem_type(args.get(*index)?, ctx),
            // What a lambda's BODY answers is read in codegen, off the class
            // this pass synthesizes — by the time it exists, this pass has
            // already handed out the target types it was asked for.
            InferSource::LambdaResult(_) => None,
        });
        if let Some(pinned) = pinned {
            bound.insert(var.clone(), pinned);
        }
    }
    if !sig.class_params.is_empty()
        && let Some(written) = receiver.and_then(|r| receiver_class_arguments(r, &sig.owner, ctx))
    {
        for (name, actual) in sig.class_params.iter().zip(written) {
            bound.entry(name.clone()).or_insert(actual);
        }
    }
    bound
}

/// What a call to a GENERIC method of the program returns, with its type
/// variables pinned. `box("ab")` returns `List<String>`, not `List<T>` — and
/// the erased `T` reaching codegen is what made a lambda over the result
/// "a functional interface parameterized on a method\'s own type variable".
fn generic_call_return(call: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    let Expr::Call {
        receiver,
        method,
        args,
        type_args,
        ..
    } = call
    else {
        return None;
    };
    let sigs = ctx.generics.get(method)?;
    let mut matching = sigs.iter().filter(|sig| sig.params.len() == args.len());
    let sig = matching.next()?;
    if matching.next().is_some() {
        return None;
    }
    let bound = pinned_vars(sig, receiver.as_deref(), args, type_args, ctx);
    if bound.is_empty() {
        return None;
    }
    substitute_vars(&sig.return_type, &bound, &sig.vars)
}

/// The target type for each argument of a call to a GENERIC method, with the
/// method's type variables pinned from the arguments (and, for a variable the
/// declaring class owns, from the receiver's own type arguments). `None` where
/// a variable could not be pinned, so the caller falls back to the erased
/// signature.
fn generic_argument_targets(
    method: &str,
    receiver: Option<&Expr>,
    args: &[Expr],
    witness: &[TypeRef],
    ctx: &Ctx,
) -> Option<Vec<Option<TypeRef>>> {
    let sigs = ctx.generics.get(method)?;
    let mut matching = sigs.iter().filter(|sig| sig.params.len() == args.len());
    let sig = matching.next()?;
    // More than one generic method of this name and arity: which one applies
    // is an overload question this pass cannot answer, so it answers none.
    if matching.next().is_some() {
        return None;
    }
    let bound = pinned_vars(sig, receiver, args, witness, ctx);
    if bound.is_empty() {
        return None;
    }
    Some(
        sig.params
            .iter()
            .zip(&sig.erased)
            .map(|(declared, erased)| Some(substitute_partly(declared, erased, &bound, &sig.vars)))
            .collect(),
    )
}

/// A written type with every PINNED variable replaced, and every unpinned one
/// taken from the same position of the ERASED type — the answer the call site
/// would have fallen back to anyway.
///
/// Substituting nothing at all where one variable is missing is what made
/// `src.mapped(s -> s.length())` refuse: `Function<T, R>` could pin `T` from
/// the receiver but never `R`, and dropping the whole target left `s` an
/// Object. An unpinned name must not reach codegen, which is why the erased
/// type is merged in rather than the variable left standing.
fn substitute_partly(
    declared: &TypeRef,
    erased: &TypeRef,
    bound: &HashMap<String, TypeRef>,
    vars: &[String],
) -> TypeRef {
    match (declared, erased) {
        (
            TypeRef::Generic { base, args },
            TypeRef::Generic {
                args: erased_args, ..
            },
        ) if args.len() == erased_args.len() && !vars.iter().any(|v| v == base) => {
            TypeRef::Generic {
                base: base.clone(),
                args: args
                    .iter()
                    .zip(erased_args)
                    .map(|(a, e)| substitute_partly(a, e, bound, vars))
                    .collect(),
            }
        }
        (TypeRef::Array(inner), TypeRef::Array(erased_inner)) => TypeRef::Array(Box::new(
            substitute_partly(inner, erased_inner, bound, vars),
        )),
        _ => substitute_vars(declared, bound, vars).unwrap_or_else(|| erased.clone()),
    }
}

/// A written type with every bound variable replaced. `None` when a variable
/// it mentions was not pinned — a partly-substituted target is worse than none,
/// since the unbound half would reach codegen as a name nothing declares.
fn substitute_vars(
    ty: &TypeRef,
    bound: &HashMap<String, TypeRef>,
    vars: &[String],
) -> Option<TypeRef> {
    match ty {
        TypeRef::Named(name) => {
            if let Some(actual) = bound.get(name) {
                return Some(actual.clone());
            }
            // A wildcard's bound may itself be the variable
            // (`Consumer<? super T>`); the wildcard is an encoded NAME, so
            // rewriting it is a name rewrite.
            if let Some((_, inner)) = crate::ast::wildcard_parts(name) {
                return match bound.get(inner) {
                    Some(actual) => Some(actual.clone()),
                    None if vars.iter().any(|v| v == inner) => None,
                    None => Some(ty.clone()),
                };
            }
            if vars.iter().any(|v| v == name) {
                return None;
            }
            Some(ty.clone())
        }
        TypeRef::Generic { base, args } => {
            let args: Option<Vec<TypeRef>> = args
                .iter()
                .map(|a| substitute_vars(a, bound, vars))
                .collect();
            Some(TypeRef::Generic {
                base: base.clone(),
                args: args?,
            })
        }
        TypeRef::Array(inner) => Some(TypeRef::Array(Box::new(substitute_vars(
            inner, bound, vars,
        )?))),
        other => Some(other.clone()),
    }
}

/// What a `Scanner`'s reader answers, by name. `hasNext…` is the question and
/// `next…` the value, which is the whole of the class a corpus program uses.
fn scanner_answer(read: &str) -> Option<TypeRef> {
    if read.starts_with("hasNext") {
        return Some(TypeRef::Boolean);
    }
    Some(match read {
        "nextLine" | "next" => TypeRef::Named(String::from("String")),
        "nextInt" => TypeRef::Int,
        "nextLong" => TypeRef::Long,
        "nextDouble" | "nextFloat" => TypeRef::Double,
        "nextBoolean" => TypeRef::Boolean,
        _ => return None,
    })
}

/// What a LIBRARY call answers: the factories that are typed from their
/// ARGUMENT (`Optional.of(x)`), and everything else from the receiver's own
/// type, which is the reading [`library_return`] already holds.
fn library_call_type(owner: &Expr, method: &str, args: &[Expr], ctx: &Ctx) -> Option<TypeRef> {
    if matches!(method, "of" | "ofNullable")
        && args.len() == 1
        && names_library_class(owner, "Optional")
    {
        return Some(TypeRef::Generic {
            base: String::from("Optional"),
            args: vec![boxed_element(static_type_of(&args[0], ctx)?)],
        });
    }
    library_return(&static_type_of(owner, ctx)?, method, args.len())
}

/// What a method of the PROGRAM answers, on a receiver whose class the pass can
/// name. The declared return as written — a type VARIABLE is left alone here,
/// since the receiver's own argument is what would replace it and that is
/// [`call_body_type`]'s reading, with the lambda's bound parameters in hand.
fn user_method_return(owner: &Expr, method: &str, argc: usize, ctx: &Ctx) -> Option<TypeRef> {
    // The receiver is either a VALUE whose class can be named, or the CLASS
    // itself — `Deck.shuffledFake()`, the static factory a `var` most often
    // holds. Only the first was read, so the factory form had no type.
    let class = declared_class_name(owner, ctx).or_else(|| match owner {
        Expr::Name { path, .. }
            if path.len() == 1
                && ctx.lookup(&path[0]).is_none()
                && ctx.class_names.contains(&path[0]) =>
        {
            Some(path[0].clone())
        }
        _ => None,
    })?;
    let answered = ctx
        .shapes
        .get(&class)?
        .iter()
        .find(|shape| shape.name == method && shape.takes(argc))
        .map(|shape| shape.return_type.clone())?;
    // A bare type VARIABLE says nothing without the receiver's argument.
    match &answered {
        TypeRef::Named(name) if crate::parser::typevar_index(name).is_some() => None,
        TypeRef::Void => None,
        _ => Some(answered),
    }
}

/// The type of a literal collection factory — `List.of(…)`, `Set.of(…)`,
/// `Arrays.asList(…)`. They are how a collection is written inline, so without
/// them `var items = new ArrayList<>(List.of(item))` had no element and the
/// lambda in the stream after it had no target.
fn literal_collection_type(expr: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        ..
    } = expr
    else {
        return None;
    };
    let Expr::Name { path, .. } = owner.as_ref() else {
        return None;
    };
    let owner = path.last().map(String::as_str)?;
    if args.is_empty()
        || !matches!(
            (owner, method.as_str()),
            ("List" | "Set", "of") | ("Arrays", "asList")
        )
    {
        return None;
    }
    let first = static_type_of(&args[0], ctx)?;
    // A LONE reference array SPREADS: `Arrays.asList(Kind.values())` is a list
    // of the constants, not a one-element list holding the array. (A primitive
    // array does not — `List.of(new int[2])` is a `List<int[]>` — which is why
    // the component's kind decides.)
    let elem = match (&first, args.len()) {
        (TypeRef::Array(component), 1)
            if !matches!(
                **component,
                TypeRef::Int
                    | TypeRef::Long
                    | TypeRef::Double
                    | TypeRef::Float
                    | TypeRef::Short
                    | TypeRef::Byte
                    | TypeRef::Char
                    | TypeRef::Boolean
            ) =>
        {
            (**component).clone()
        }
        _ => boxed_element(first),
    };
    Some(TypeRef::Generic {
        base: String::from(if owner == "Set" { "Set" } else { "List" }),
        args: vec![elem],
    })
}

/// The declared type of an expression, for the shapes this pass can see. Used
/// to pin a type variable from an argument at a call.
#[allow(clippy::too_many_lines)] // one arm per expression shape
fn static_type_of(expr: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    use crate::ast::Literal;
    match expr {
        // An ARRAY written inline says its own type. Without this a stream
        // whose element is an array — `Stream.iterate(new long[] {0, 1}, p ->
        // …)`, the fold every Fibonacci one-liner uses — had no element, and
        // the lambda after it was refused for having no functional-interface
        // position.
        Expr::NewArray { elem, dims, .. } => {
            let mut ty = elem.clone();
            for _ in 0..dims.len().max(1) {
                ty = TypeRef::Array(Box::new(ty));
            }
            Some(ty)
        }
        Expr::Literal { value, .. } => Some(TypeRef::Named(String::from(match value {
            Literal::Str(_) => "String",
            Literal::Int(_) => "Integer",
            Literal::Long(_) => "Long",
            Literal::Double(_) => "Double",
            Literal::Float(_) => "Float",
            Literal::Char(_) => "Character",
            Literal::Bool(_) => "Boolean",
            Literal::Null => return None,
        }))),
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0]),
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            ctx.lookup(name)
        }
        // A field read through ANOTHER object — `deck.cards` — whose type the
        // pass can name. Only `this.field` and the bare name were read, so the
        // identical field reached through a reference had no type and the
        // lambda after `deck.cards.stream()` was refused for having no
        // functional-interface position. The parser keeps a dotted read as a
        // NAME, so both spellings arrive here.
        Expr::Field { object, name, .. } => field_of_object(object, name, ctx),
        Expr::Name { path, span } if path.len() == 2 => field_of_object(
            &Expr::Name {
                path: vec![path[0].clone()],
                span: *span,
            },
            &path[1],
            ctx,
        ),
        Expr::Cast { ty, .. } => Some(ty.clone()),
        Expr::NewObject {
            class, type_args, ..
        } if !type_args.is_empty() => Some(TypeRef::Generic {
            base: class.clone(),
            args: type_args.clone(),
        }),
        // A DIAMOND over a collection — `new ArrayList<>(List.of(item))` —
        // takes its argument from the collection it copies. Written out this
        // is what `var` needs: the declaration says nothing, so the
        // initializer is the only thing that can say what the element is, and
        // without it the lambda in `items.stream().map(…)` had no target.
        Expr::NewObject {
            class,
            type_args,
            args,
            ..
        } if type_args.is_empty() && args.len() == 1 => {
            let source = static_type_of(&args[0], ctx);
            match source {
                Some(TypeRef::Generic { args: from, .. }) if !from.is_empty() => {
                    Some(TypeRef::Generic {
                        base: class.clone(),
                        args: from,
                    })
                }
                _ => Some(TypeRef::Named(class.clone())),
            }
        }
        // A diamond or argument-less `new T()` still says WHICH class — which
        // is the whole answer for a supplier: `toCollection(TreeSet::new)`
        // gathers into a `TreeSet`, and reading the class as nothing made it
        // the default list.
        Expr::NewObject { class, .. } => Some(TypeRef::Named(class.clone())),
        // The literal collection factories, read by a helper: they are how a
        // collection is written inline, and the spread rule is a paragraph of
        // its own.
        Expr::Call { .. } if literal_collection_type(expr, ctx).is_some() => {
            literal_collection_type(expr, ctx)
        }
        // A method of the PROGRAM, on a receiver whose class can be named:
        // `new Roster().add(s)` answers a `Roster`, which is what a `var`
        // holding a builder chain needs — and without it the lambda in
        // `roster.stream().map(…)` had no element, though the same chain
        // assigned to a DECLARED variable compiled.
        Expr::Call {
            receiver: Some(owner),
            method,
            args,
            ..
        } if user_method_return(owner, method, args.len(), ctx).is_some() => {
            user_method_return(owner, method, args.len(), ctx)
        }
        // A LIBRARY call whose answer is written on its receiver —
        // `line.split(",")` is a `String[]`, `text.toUpperCase()` a String.
        // A `var` holding one had no type, so the stream over it had no
        // element: the same gap as the user-method one, on the other half of
        // the world.
        Expr::Call {
            receiver: Some(owner),
            method,
            args,
            ..
        } if library_call_type(owner, method, args, ctx).is_some() => {
            library_call_type(owner, method, args, ctx)
        }
        // An enum's two synthetic statics: `values()` answers an ARRAY of the
        // enum, `valueOf(String)` one constant. Without them a stream, a list
        // or a `Stream.of` over `Kind.values()` had an `Object` element and
        // every lambda after it lost the constant's own methods.
        Expr::Call {
            receiver: Some(owner),
            method,
            args,
            ..
        } => {
            // `EnumSet`'s factories answer a `Set<E>` of the enum they name —
            // by a class literal (`allOf(Day.class)`), by the constants
            // passed, or by the collection copied. Without it a stream over
            // one had no element and every lambda after it was refused.
            if names_library_class(owner, "EnumSet")
                && matches!(
                    method.as_str(),
                    "noneOf" | "allOf" | "of" | "range" | "complementOf" | "copyOf"
                )
            {
                let elem = args.first().and_then(|arg| match arg {
                    Expr::Field { object, name, .. } if name == "class" => match object.as_ref() {
                        Expr::Name { path, .. } => path.last().cloned().map(TypeRef::Named),
                        _ => None,
                    },
                    other => match static_type_of(other, ctx) {
                        Some(TypeRef::Generic { args, .. }) => args.first().cloned(),
                        Some(named @ TypeRef::Named(_)) => Some(named),
                        _ => None,
                    },
                });
                if let Some(elem) = elem {
                    return Some(TypeRef::Generic {
                        base: String::from("Set"),
                        args: vec![elem],
                    });
                }
            }
            // The regex objects, whose types this pass can name. A stream op
            // is written straight onto them
            // (`Pattern.compile(p).matcher(s).results().map(…)`), and inline
            // the chain has no variable to read a declared type from — so
            // without these the lambda had no target type at all while the
            // very same chain through a variable compiled.
            match (method.as_str(), args.len()) {
                ("compile", 1 | 2) if names_library_class(owner, "Pattern") => {
                    return Some(TypeRef::Named(String::from("Pattern")));
                }
                ("matcher", 1)
                    if matches!(static_type_of(owner, ctx),
                        Some(TypeRef::Named(name)) if name == "Pattern") =>
                {
                    return Some(TypeRef::Named(String::from("Matcher")));
                }
                ("toMatchResult", 0)
                    if matches!(static_type_of(owner, ctx),
                        Some(TypeRef::Named(name)) if name == "Matcher") =>
                {
                    return Some(TypeRef::Named(String::from("MatchResult")));
                }
                _ => {}
            }
            let name = enum_owner_name(owner, ctx)?;
            match (method.as_str(), args.len()) {
                ("values", 0) => Some(TypeRef::Array(Box::new(TypeRef::Named(name)))),
                ("valueOf", 1) => Some(TypeRef::Named(name)),
                _ => None,
            }
        }
        _ => None,
    }
}

fn method_signatures_by_class(
    units: &[(String, CompilationUnit)],
) -> HashMap<(String, String), Vec<Vec<TypeRef>>> {
    let mut out: HashMap<(String, String), Vec<Vec<TypeRef>>> = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            for method in &class.methods {
                if method.is_constructor {
                    continue;
                }
                let sig: Vec<TypeRef> = method.params.iter().map(|p| p.ty.clone()).collect();
                let entry = out
                    .entry((class.name.clone(), method.name.clone()))
                    .or_default();
                if !entry.contains(&sig) {
                    entry.push(sig);
                }
            }
        }
    }
    out
}

fn method_signatures(units: &[(String, CompilationUnit)]) -> HashMap<String, Vec<Vec<TypeRef>>> {
    let mut out: HashMap<String, Vec<Vec<TypeRef>>> = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            for method in &class.methods {
                if method.is_constructor {
                    continue;
                }
                // De-duplicate identical signatures so a method declared with
                // the same parameter types on several classes (e.g.
                // `addActionListener(ActionListener)` on JButton/JCheckBox/…)
                // still counts as a single target-typing candidate.
                let sig: Vec<TypeRef> = method.params.iter().map(|p| p.ty.clone()).collect();
                let entry = out.entry(method.name.clone()).or_default();
                if !entry.contains(&sig) {
                    entry.push(sig);
                }
            }
        }
    }
    out
}

#[allow(clippy::too_many_lines)] // one arm per statement kind
fn desugar_stmt(stmt: &mut Stmt, ctx: &mut Ctx) {
    match stmt {
        Stmt::Block(body) => {
            ctx.scope.push(HashMap::new());
            for s in body {
                desugar_stmt(s, ctx);
            }
            ctx.scope.pop();
        }
        Stmt::LocalDecl {
            ty, declarators, ..
        } => {
            for d in declarators {
                if let Some(init) = &mut d.init {
                    desugar_expr(init, Some(ty), ctx);
                }
                // `var` says nothing on its own: the INITIALIZER does.
                // Recording the placeholder left `var items = new
                // ArrayList<>(…)` with no element, so the lambda in
                // `items.stream().map(…)` had no target and was refused as
                // though the position were not a functional-interface one.
                let declared = if matches!(ty, TypeRef::Var) {
                    d.init
                        .as_ref()
                        .and_then(|init| static_type_of(init, ctx))
                        .unwrap_or(TypeRef::Var)
                } else {
                    ty.clone()
                };
                if let Some(frame) = ctx.scope.last_mut() {
                    frame.insert(d.name.clone(), declared);
                }
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => desugar_expr(e, None, ctx),
        Stmt::Assign { target, value, .. } => {
            let expected = assign_target_type(target, ctx);
            desugar_expr(value, expected.as_ref(), ctx);
        }
        Stmt::Return { value: Some(e), .. } => {
            let expected = ctx.ret.cloned();
            desugar_expr(e, expected.as_ref(), ctx);
        }
        // An empty statement holds nothing, like a bare return or a break.
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Empty(_) => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            desugar_expr(cond, None, ctx);
            desugar_stmt(then, ctx);
            if let Some(e) = els {
                desugar_stmt(e, ctx);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            desugar_expr(cond, None, ctx);
            desugar_stmt(body, ctx);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            ctx.scope.push(HashMap::new());
            if let Some(s) = init {
                desugar_stmt(s, ctx);
            }
            if let Some(c) = cond {
                desugar_expr(c, None, ctx);
            }
            for s in update {
                desugar_stmt(s, ctx);
            }
            desugar_stmt(body, ctx);
            ctx.scope.pop();
        }
        Stmt::ForEach {
            ty,
            name,
            iterable,
            body,
            ..
        } => {
            desugar_expr(iterable, None, ctx);
            ctx.scope.push(HashMap::new());
            if let Some(frame) = ctx.scope.last_mut() {
                frame.insert(name.clone(), ty.clone());
            }
            desugar_stmt(body, ctx);
            ctx.scope.pop();
        }
        Stmt::Switch { selector, arms, .. } => {
            desugar_expr(selector, None, ctx);
            for arm in arms {
                for s in &mut arm.body {
                    desugar_stmt(s, ctx);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            for s in body {
                desugar_stmt(s, ctx);
            }
            for c in catches {
                for s in &mut c.body {
                    desugar_stmt(s, ctx);
                }
            }
            if let Some(fin) = finally_body {
                for s in fin {
                    desugar_stmt(s, ctx);
                }
            }
        }
        Stmt::Labeled { body, .. } => desugar_stmt(body, ctx),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                desugar_expr(a, None, ctx);
            }
        }
    }
}

/// The declared type of an assignment target, for target typing.
fn assign_target_type(target: &crate::ast::AssignTarget, ctx: &Ctx) -> Option<TypeRef> {
    match target {
        crate::ast::AssignTarget::Var(name) => ctx.lookup(name),
        crate::ast::AssignTarget::Index { array, .. } => {
            // `arr[i] = ...`: the element type is one array dimension down.
            if let Expr::Name { path, .. } = array.as_ref()
                && path.len() == 1
                && let Some(TypeRef::Array(elem)) = ctx.lookup(&path[0])
            {
                Some(*elem)
            } else {
                None
            }
        }
        // `obj.f = P::m` / `this.f = ...`: the field's declared type is the
        // target type, looked up on the class the object names.
        crate::ast::AssignTarget::Field { object, name } => {
            let owner = match object.as_ref() {
                Expr::This { .. } => ctx.current_class.map(str::to_owned),
                Expr::Name { path, .. } if path.len() == 1 => match ctx.lookup(&path[0]) {
                    Some(TypeRef::Named(class) | TypeRef::Generic { base: class, .. }) => {
                        Some(class)
                    }
                    _ => ctx.class_names.contains(&path[0]).then(|| path[0].clone()),
                },
                _ => None,
            }?;
            ctx.fields.get(&(owner, name.clone())).cloned()
        }
    }
}

#[allow(clippy::too_many_lines)] // one arm per expression kind
fn desugar_expr(expr: &mut Expr, expected: Option<&TypeRef>, ctx: &mut Ctx) {
    // A target the identity arm below synthesizes for itself; see there.
    let mut identity_target: Option<TypeRef> = None;
    // `Function.identity()` IS the lambda `x -> x`, and saying so here is the
    // whole implementation: everything below — target typing, the erased SAM,
    // the synthesized class — then treats it as one. Written out by hand it
    // always worked; only the named factory was missing.
    if let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        span,
        ..
    } = expr
        && method == "identity"
        && args.is_empty()
        && matches!(owner.as_ref(), Expr::Name { path, .. }
            if path.last().is_some_and(|name| matches!(name.as_str(),
                "Function" | "UnaryOperator"
                // The PRIMITIVE unary operators declare `identity()` too, and
                // it means the same lambda.
                | "IntUnaryOperator" | "LongUnaryOperator" | "DoubleUnaryOperator")))
    {
        let span = *span;
        // `IntUnaryOperator.identity().applyAsInt(7)` — an identity used
        // STRAIGHT, with no variable to read a target type from. The owner
        // names the interface, so the arm supplies the target itself rather
        // than leaving a bare lambda that "is only allowed where a
        // functional-interface type is expected". Where javac has to infer
        // (`Function.identity()`), it infers `Object`, and so does this.
        if expected.is_none()
            && let Expr::Name { path, .. } = owner.as_ref()
            && let Some(simple) = path.last()
        {
            let object = || TypeRef::Named(String::from("Object"));
            identity_target = Some(match simple.as_str() {
                "Function" => TypeRef::Generic {
                    base: String::from("Function"),
                    args: vec![object(), object()],
                },
                "UnaryOperator" => TypeRef::Generic {
                    base: String::from("UnaryOperator"),
                    args: vec![object()],
                },
                other => TypeRef::Named(other.to_owned()),
            });
        }
        let name = String::from("__identity");
        *expr = Expr::Lambda {
            params: vec![crate::ast::LambdaParam {
                name: name.clone(),
                ty: None,
            }],
            body: LambdaBody::Expr(Box::new(Expr::Name {
                path: vec![name],
                span,
            })),
            span,
        };
    }
    let expected = identity_target.as_ref().or(expected);
    // `Predicate.not(p)` (Java 11) IS `p.negate()`, and `negate()` already
    // builds the bundled `__Negate`. Rewriting to that CLASS rather than to a
    // `.negate()` call keeps the argument in a target-typed position, so a
    // lambda written inline (`Predicate.not(s -> s.isEmpty())`) still knows
    // what interface it implements.
    if let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        span,
        ..
    } = expr
        && matches!(method.as_str(), "not" | "isEqual")
        && args.len() == 1
        && matches!(owner.as_ref(), Expr::Name { path, .. }
            if path.last().is_some_and(|name| name == "Predicate"))
    {
        // `not`'s argument IS the predicate, so it inherits the whole
        // expression's target type — desugaring it here is what tells an
        // inline lambda its element is a `String` and not an `Object`. The
        // constructor parameter is the ERASED `__Predicate`, which carries no
        // element, so leaving this to the generic recursion typed it `Object`.
        if method == "not" {
            desugar_expr(&mut args[0], expected, ctx);
        }
        *expr = Expr::NewObject {
            class: String::from(if method == "not" {
                "__Negate"
            } else {
                "__IsEqual"
            }),
            type_args: Vec::new(),
            args: std::mem::take(args),
            outer: None,
            span: *span,
        };
    }
    // `toArray(String[]::new)` (Java 11) IS `toArray(new String[0])`: the JDK's
    // default method is `toArray(generator.apply(0))`, so applying the
    // generator here — the parser already models `String[]::new` as the lambda
    // `n -> new String[n]` — is the whole implementation, and the array
    // overload below does the rest.
    if let Expr::Call {
        receiver,
        method,
        args,
        ..
    } = expr
        && method == "toArray"
        && let [Expr::Lambda { params, body, span }] = &args[..]
        && let [param] = &params[..]
        && let LambdaBody::Expr(body) = body
        && let Expr::NewArray {
            elem,
            dims,
            init: None,
            ..
        } = body.as_ref()
        && let [Some(Expr::Name { path, .. })] = &dims[..]
        && path == std::slice::from_ref(&param.name)
    {
        let span = *span;
        let elem = elem.clone();
        // A STREAM has only the generator overload, so its call is renamed to
        // an internal one; a COLLECTION has `toArray(T[])` itself and keeps it.
        let stream = receiver
            .as_deref()
            .is_some_and(|r| stream_elem_type(r, ctx).is_some());
        args[0] = Expr::NewArray {
            elem,
            dims: vec![Some(Expr::Literal {
                value: crate::ast::Literal::Int(0),
                span,
            })],
            init: None,
            span,
        };
        if stream {
            *method = String::from("__toArrayTyped");
        }
    }
    // The same rewrite for a generator held in a VARIABLE. `toArray(gen)` is
    // `toArray(gen.apply(0))` by the JDK's own definition, and only the
    // written-out `String[]::new` was recognised — the shape that compiles
    // inline and is refused one line later, through a name.
    if let Expr::Call {
        receiver,
        method,
        args,
        span,
        ..
    } = expr
        && method == "toArray"
        && let [only] = &args[..]
        && !matches!(
            only,
            Expr::Lambda { .. } | Expr::NewArray { .. } | Expr::MethodRef { .. }
        )
        && let Some(TypeRef::Generic {
            base,
            args: written,
        }) = static_type_of(only, ctx)
        && simple_base(&base) == "IntFunction"
        && matches!(written.first(), Some(TypeRef::Array(_)))
    {
        let span = *span;
        let stream = receiver
            .as_deref()
            .is_some_and(|r| stream_elem_type(r, ctx).is_some());
        args[0] = Expr::Call {
            receiver: Some(Box::new(only.clone())),
            method: String::from("apply"),
            args: vec![Expr::Literal {
                value: crate::ast::Literal::Int(0),
                span,
            }],
            type_args: Vec::new(),
            span,
        };
        if stream {
            *method = String::from("__toArrayTyped");
        }
    }
    // `BinaryOperator.minBy(cmp)` / `maxBy(cmp)` — a two-argument function that
    // keeps one side, built from the comparator. Like `Predicate.not`, the
    // named CLASS is what the argument lands in, so an inline comparator lambda
    // still has a functional target; the target it gets is a `Comparator` over
    // the operator's own element.
    if let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        span,
        ..
    } = expr
        && matches!(method.as_str(), "minBy" | "maxBy")
        && args.len() == 1
        && matches!(owner.as_ref(), Expr::Name { path, .. }
            if path.last().is_some_and(|name| name == "BinaryOperator"))
    {
        let comparator = match expected {
            Some(TypeRef::Generic { base, args: over })
                if simple_base(base) == "BinaryOperator" =>
            {
                Some(TypeRef::Generic {
                    base: String::from("Comparator"),
                    args: over.clone(),
                })
            }
            _ => None,
        };
        desugar_expr(&mut args[0], comparator.as_ref(), ctx);
        *expr = Expr::NewObject {
            class: String::from(if method == "minBy" {
                "__MinBy"
            } else {
                "__MaxBy"
            }),
            type_args: Vec::new(),
            args: std::mem::take(args),
            outer: None,
            span: *span,
        };
    }
    // A method reference in a target-typed position becomes a lambda.
    if matches!(expr, Expr::MethodRef { .. }) {
        // A `java.util.function` target (`Function<String, Integer> len =
        // String::length`): synthesize the SAM from the type arguments, turn the
        // reference into a lambda, and fall through to the erased-lambda branch.
        if let Some(target) = expected
            && !user_defined_functional(target, ctx)
            && let Some(spec) = functional_lambda_spec(target)
        {
            let sam = Sam {
                method: spec.method.to_owned(),
                params: spec.params.clone(),
                ret: spec.result.clone().unwrap_or(spec.ret),
            };
            validate_method_ref(expr, &sam, ctx);
            *expr = method_ref_to_lambda(expr, &sam, ctx);
            ctx.class_prefix = crate::METHOD_REF_CLASS_PREFIX;
        } else if let Some(target) = expected
            && let Some((_, sam)) = sam_target(target, ctx)
        {
            validate_method_ref(expr, &sam, ctx);
            *expr = method_ref_to_lambda(expr, &sam, ctx);
            ctx.class_prefix = crate::METHOD_REF_CLASS_PREFIX;
            // Fall through to lambda handling below.
        } else {
            if let Expr::MethodRef { qualifier, .. } = expr {
                desugar_expr(qualifier, None, ctx);
            }
            return;
        }
    }
    // A comparator lambda in a `Comparator<E>` position (a variable, field,
    // parameter, or return): the erased SAM is `__Comparator.compare`, and both
    // parameters cast back to `E` — like `list.sort`, but with `E` read from the
    // target type's argument rather than a receiver's element.
    if let Expr::Lambda { params, .. } = expr
        && params.len() == 2
        && let Some(target) = expected
        && let Some(elem) = comparator_target_elem(target)
    {
        *expr = build_erased_lambda(
            expr,
            "__Comparator",
            "compare",
            &TypeRef::Int,
            &[elem.clone(), elem],
            None,
            ctx,
        );
        return;
    }
    // A lambda in a `java.util.function` position (`Function<A, B>`,
    // `Predicate<A>`, `Supplier<A>`, ...): the same erased-SAM treatment as
    // `Comparator`, with the parameter and result types read from the target's
    // type arguments rather than a receiver's element.
    if let Expr::Lambda { .. } = expr
        && let Some(target) = expected
        && !user_defined_functional(target, ctx)
        && let Some(spec) = functional_lambda_spec(target)
    {
        *expr = build_erased_lambda(
            expr,
            spec.interface,
            spec.method,
            &spec.ret,
            &spec.params,
            spec.result.as_ref(),
            ctx,
        );
        return;
    }
    // A lambda in a target-typed position: rewrite it.
    if let Expr::Lambda { params, span, .. } = expr {
        let (arity, span) = (params.len(), *span);
        if let Some(target) = expected
            && let Some((name, sam)) = sam_target(target, ctx)
        {
            // ...unless the lambda's shape does not FIT the target, which is a
            // mistake with a name: javac says which of the two it is, where
            // the fallback ("only allowed where a functional-interface type is
            // expected") blames the position for the program's arity.
            if sam.params.len() != arity {
                ctx.diags.push(crate::diagnostics::Diagnostic::error(
                    ctx.path,
                    String::from(
                        "incompatible types: incompatible parameter types in lambda expression",
                    ),
                    span,
                ));
                return;
            }
            let specialized = specialize_sam(&sam, target);
            let replacement = build_lambda_class(expr, &name, &sam, specialized.as_ref(), ctx);
            *expr = replacement;
            return;
        }
        // A target that is a real TYPE and not a functional interface at all —
        // `String s = () -> "x";`. javac names it; the fallback below does not.
        if let Some(target) = expected
            && let TypeRef::Named(name) = target
            && ctx.class_names.contains(name)
        {
            ctx.diags.push(crate::diagnostics::Diagnostic::error(
                ctx.path,
                format!("incompatible types: {name} is not a functional interface"),
                span,
            ));
        }
        return;
    }
    match expr {
        Expr::Call {
            receiver,
            method,
            args,
            type_args,
            span,
        } => {
            // A comparator FACTORY or COMBINATOR: the whole chain shares one
            // element type, and the generic receiver walk below would throw
            // away the target type that carries it.
            if desugar_comparator_chain(receiver, method, args, expected, ctx) {
                return;
            }
            // `Map.Entry.comparingByValue((a, b) -> ...)`: the lambda compares
            // two VALUES (or two KEYS), which nothing in the call itself says
            // — the entry type comes from the `Comparator<Map.Entry<K, V>>`
            // position the factory sits in, which the sort site hands down.
            if let ("comparingByKey" | "comparingByValue", [Expr::Lambda { params, .. }]) =
                (method.as_str(), &args[..])
                && params.len() == 2
                && matches!(receiver.as_deref(), Some(Expr::Name { path, .. })
                        if path.last().is_some_and(|last| last == "Entry"))
                && let Some(side) = expected
                    .and_then(comparator_target_elem)
                    .and_then(|entry| entry_side_type(&entry, method == "comparingByValue"))
            {
                args[0] = build_erased_lambda(
                    &mut args[0],
                    "__Comparator",
                    "compare",
                    &TypeRef::Int,
                    &[side.clone(), side],
                    None,
                    ctx,
                );
                return;
            }
            if let Some(r) = receiver {
                desugar_expr(r, None, ctx);
            }
            // `map.forEach((k, v) -> ...)`: the SAM is the erased
            // `__BiConsumer`, and the lambda's parameter types come from the
            // RECEIVER's declared type arguments. No other target type in
            // caturra is instantiated from its receiver, so this is its own
            // rule rather than a case of the one below.
            if method == "forEach"
                && args.len() == 1
                && matches!(&args[0], Expr::Lambda { params, .. } if params.len() == 2)
                && let Some(r) = receiver.as_deref()
                && let Some((key, value)) = map_type_args(r, ctx)
            {
                args[0] = build_bi_consumer_class(&mut args[0], &key, &value, ctx);
                return;
            }
            // The lambda-taking map methods (JDK 8). Like `forEach`, the
            // lambda's parameter types come from the RECEIVER's type arguments
            // — and they differ per method: `merge`'s remapper sees two VALUES,
            // `compute`/`computeIfPresent`/`replaceAll` see a key and a value,
            // and `computeIfAbsent`'s mapping sees only the key. Each returns a
            // value, which the erased SAM types as `Object` and `result_type`
            // checks back against `V`.
            if matches!(
                method.as_str(),
                "merge" | "compute" | "computeIfPresent" | "computeIfAbsent" | "replaceAll"
            ) && let Some(r) = receiver.as_deref()
                && let Some((key, value)) = map_type_args(r, ctx)
                && !args.is_empty()
            {
                let params: Vec<TypeRef> = match method.as_str() {
                    "merge" => vec![value.clone(), value.clone()],
                    "computeIfAbsent" => vec![key.clone()],
                    _ => vec![key.clone(), value.clone()],
                };
                // A pre-built function VARIABLE: its type arguments erase
                // before codegen, so `merge` taking `(V, V)` would silently
                // accept a `BiFunction<K, V, V>` and CCE at run time. Check
                // the declaration here, where it is still visible.
                if let Some(Expr::Name { path, span }) = args.last()
                    && path.len() == 1
                    && let Some(declared) = ctx.lookup(&path[0])
                {
                    check_map_function_variable(&declared, &params, &value, *span, ctx);
                }
                if let Some(function) = args.last_mut()
                    && matches!(function, Expr::Lambda { .. } | Expr::MethodRef { .. })
                {
                    // `m.merge(k, v, Integer::sum)` — the reference first
                    // becomes the equivalent lambda, then erases like one.
                    if matches!(function, Expr::MethodRef { .. }) {
                        let synth = Sam {
                            method: String::from("apply"),
                            params: params.clone(),
                            ret: value.clone(),
                        };
                        *function = method_ref_to_lambda(function, &synth, ctx);
                    }
                    let expected = match function {
                        Expr::Lambda { params, .. } => params.len(),
                        _ => 0,
                    };
                    if expected == params.len() {
                        let object = TypeRef::Named(String::from("Object"));
                        let interface = if params.len() == 2 {
                            "__BiFunction"
                        } else {
                            "__UnaryOperator"
                        };
                        let last = args.len() - 1;
                        let (leading, tail) = args.split_at_mut(last);
                        for arg in leading {
                            desugar_expr(arg, None, ctx);
                        }
                        tail[0] = build_erased_lambda(
                            &mut tail[0],
                            interface,
                            "apply",
                            &object,
                            &params,
                            Some(&value),
                            ctx,
                        );
                        return;
                    }
                }
            }
            // `matcher.replaceAll(mr -> …)` (Java 9) — a `Function` whose
            // parameter is the MATCH, not an element of anything, so the
            // element-typed arm below cannot reach it.
            if matches!(method.as_str(), "replaceAll" | "replaceFirst")
                && args.len() == 1
                && matches!(&args[0], Expr::Lambda { .. } | Expr::MethodRef { .. })
                && matches!(receiver.as_deref().and_then(|r| static_type_of(r, ctx)),
                    Some(TypeRef::Named(name)) if name == "Matcher")
            {
                let result = TypeRef::Named(String::from("String"));
                let param = TypeRef::Named(String::from("MatchResult"));
                if matches!(&args[0], Expr::MethodRef { .. }) {
                    let synth = Sam {
                        method: String::from("apply"),
                        params: vec![param.clone()],
                        ret: result.clone(),
                    };
                    args[0] = method_ref_to_lambda(&args[0], &synth, ctx);
                }
                args[0] = build_erased_lambda(
                    &mut args[0],
                    "__UnaryOperator",
                    "apply",
                    &TypeRef::Named(String::from("Object")),
                    &[param],
                    Some(&result),
                    ctx,
                );
                return;
            }
            // `list.forEach(x -> ...)` / `list.removeIf(x -> ...)`: a single
            // lambda whose parameter type is the receiver's element type. The
            // erased SAM is `__Consumer` (void) or `__Predicate` (boolean).
            // A METHOD REFERENCE stands here too (`list.forEach(this::show)`),
            // and was refused with the false claim that this is not a
            // functional-interface position.
            let one_argument_function = matches!(&args[0..], [Expr::MethodRef { .. }])
                || matches!(&args[0..], [Expr::Lambda { params, .. }] if params.len() == 1);
            if matches!(
                method.as_str(),
                "forEach" | "forEachRemaining" | "removeIf" | "replaceAll"
            ) && args.len() == 1
                && let Some(elem) = receiver.as_deref().and_then(|r| list_elem_type(r, ctx))
                && let Some(()) = {
                    // A lambda of the WRONG arity for the one-argument callback
                    // these take is a mistake with a name — javac's
                    // "incompatible parameter types in lambda expression" —
                    // where falling through blamed the position instead.
                    if let Expr::Lambda { params, span, .. } = &args[0]
                        && params.len() != 1
                    {
                        ctx.diags.push(crate::diagnostics::Diagnostic::error(
                            ctx.path,
                            String::from(
                                "incompatible types: incompatible parameter types in lambda \
                                 expression",
                            ),
                            *span,
                        ));
                        return;
                    }
                    (one_argument_function
                        || (method == "removeIf" && is_negated_predicate(&args[0])))
                    .then_some(())
                }
            {
                let object = TypeRef::Named(String::from("Object"));
                let (iface, sam, ret) = match method.as_str() {
                    "forEach" | "forEachRemaining" => ("__Consumer", "accept", TypeRef::Void),
                    "removeIf" => ("__Predicate", "test", TypeRef::Boolean),
                    // `UnaryOperator<E>` — `E apply(E)`, erased to `Object
                    // apply(Object)`. The result is boxed on return.
                    _ => ("__UnaryOperator", "apply", object),
                };
                // `list.forEach(System.out::println)` — a method reference is
                // as good as a lambda here, and was refused with the false
                // claim that this is not a functional-interface position. It
                // becomes the equivalent lambda first, then erases like one.
                if matches!(&args[0], Expr::MethodRef { .. }) {
                    let synth = Sam {
                        method: String::from(sam),
                        params: vec![elem.clone()],
                        ret: ret.clone(),
                    };
                    args[0] = method_ref_to_lambda(&args[0], &synth, ctx);
                }
                let result_type = (method == "replaceAll").then(|| elem.clone());
                args[0] = build_erased_lambda(
                    &mut args[0],
                    iface,
                    sam,
                    &ret,
                    &[elem],
                    result_type.as_ref(),
                    ctx,
                );
                return;
            }
            // `optional.ifPresent(x -> ...)` / `filter(x -> ...)` / `map(x ->
            // ...)`: a single lambda whose parameter is the Optional's element.
            // `ifPresent` erases to `__Consumer` (void), `filter` to
            // `__Predicate` (boolean), `map` to `__UnaryOperator` (its result
            // erased to `Object`, like a stream's `map`).
            // A METHOD REFERENCE stands in every one of these positions too
            // (`optional.map(String::toUpperCase)`), and was refused with the
            // false claim that this is not a functional-interface position —
            // while the equivalent lambda compiled. `flatMap` takes the same
            // shape as `map`: a function whose result the Optional adopts.
            let one_argument_function = matches!(&args[0..], [Expr::MethodRef { .. }])
                || matches!(&args[0..], [Expr::Lambda { params, .. }] if params.len() == 1);
            if matches!(method.as_str(), "ifPresent" | "filter" | "map" | "flatMap")
                && (one_argument_function
                    || (method == "filter" && args.len() == 1 && is_negated_predicate(&args[0])))
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = optional_elem_type(r, ctx)
            {
                let object = TypeRef::Named(String::from("Object"));
                let (iface, sam, ret) = match method.as_str() {
                    "ifPresent" => ("__Consumer", "accept", TypeRef::Void),
                    "filter" => ("__Predicate", "test", TypeRef::Boolean),
                    _ => ("__UnaryOperator", "apply", object),
                };
                if matches!(&args[0], Expr::MethodRef { .. }) {
                    let synth = Sam {
                        method: String::from(sam),
                        params: vec![elem.clone()],
                        ret: ret.clone(),
                    };
                    args[0] = method_ref_to_lambda(&args[0], &synth, ctx);
                }
                args[0] = build_erased_lambda(&mut args[0], iface, sam, &ret, &[elem], None, ctx);
                return;
            }
            // `optional.ifPresentOrElse(action, empty)`: a consumer of the
            // element and a Runnable that takes nothing — the two arms erase
            // to different interfaces, which is why this cannot ride the
            // single-function path above.
            if method == "ifPresentOrElse"
                && args.len() == 2
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = optional_elem_type(r, ctx)
            {
                if matches!(&args[0], Expr::MethodRef { .. }) {
                    let synth = Sam {
                        method: String::from("accept"),
                        params: vec![elem.clone()],
                        ret: TypeRef::Void,
                    };
                    args[0] = method_ref_to_lambda(&args[0], &synth, ctx);
                }
                if matches!(&args[0], Expr::Lambda { .. }) {
                    args[0] = build_erased_lambda(
                        &mut args[0],
                        "__Consumer",
                        "accept",
                        &TypeRef::Void,
                        &[elem],
                        None,
                        ctx,
                    );
                }
                if matches!(&args[1], Expr::MethodRef { .. }) {
                    let synth = Sam {
                        method: String::from("run"),
                        params: Vec::new(),
                        ret: TypeRef::Void,
                    };
                    args[1] = method_ref_to_lambda(&args[1], &synth, ctx);
                }
                if matches!(&args[1], Expr::Lambda { .. }) {
                    args[1] = build_erased_lambda(
                        &mut args[1],
                        "__Runnable",
                        "run",
                        &TypeRef::Void,
                        &[],
                        None,
                        ctx,
                    );
                }
                return;
            }
            // `optional.or(() -> ...)`: a supplier of another OPTIONAL, not of
            // the element — the one place these two are told apart.
            if method == "or"
                && args.len() == 1
                && matches!(&args[0], Expr::Lambda { params, .. } if params.is_empty())
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = optional_elem_type(r, ctx)
            {
                let object = TypeRef::Named(String::from("Object"));
                // The supplier must answer another OPTIONAL, not an element —
                // that is the whole difference from `orElseGet`. Erased with
                // no result type the check was gone, and `o.or(() -> "x")`
                // compiled where javac says "bad return type in lambda
                // expression".
                let answers = TypeRef::Generic {
                    base: String::from("Optional"),
                    args: vec![elem],
                };
                args[0] = build_erased_lambda(
                    &mut args[0],
                    "__Supplier",
                    "get",
                    &object,
                    &[],
                    Some(&answers),
                    ctx,
                );
                return;
            }
            // `optional.orElseGet(() -> ...)`: a zero-parameter supplier whose
            // result is the Optional's element type.
            if method == "orElseGet"
                && args.len() == 1
                && matches!(&args[0], Expr::Lambda { params, .. } if params.is_empty())
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = optional_elem_type(r, ctx)
            {
                let object = TypeRef::Named(String::from("Object"));
                args[0] = build_erased_lambda(
                    &mut args[0],
                    "__Supplier",
                    "get",
                    &object,
                    &[],
                    Some(&elem),
                    ctx,
                );
                return;
            }
            // `Objects.requireNonNullElseGet(value, () -> ...)`: a
            // zero-parameter supplier of the value's own type, asked only when
            // the value is null. Without this the lambda had no target and the
            // message blamed the LAMBDA for a method caturra had not modelled.
            let supplier_argument = matches!(&args[..], [_, Expr::MethodRef { .. }])
                || matches!(&args[..], [_, Expr::Lambda { params, .. }] if params.is_empty());
            if method == "requireNonNullElseGet"
                && supplier_argument
                && matches!(receiver.as_deref(), Some(Expr::Name { path, .. })
                        if path.len() == 1 && path[0] == "Objects")
            {
                desugar_expr(&mut args[0], None, ctx);
                let object = TypeRef::Named(String::from("Object"));
                if matches!(&args[1], Expr::MethodRef { .. }) {
                    let synth = Sam {
                        method: String::from("get"),
                        params: Vec::new(),
                        ret: object.clone(),
                    };
                    args[1] = method_ref_to_lambda(&args[1], &synth, ctx);
                }
                args[1] =
                    build_erased_lambda(&mut args[1], "__Supplier", "get", &object, &[], None, ctx);
                return;
            }
            // `Arrays.sort(array, cmp)`: the comparator is over the ARRAY's
            // element type — the same rule as `list.sort`, read from the first
            // argument rather than the receiver.
            if method == "sort"
                && args.len() == 2
                && matches!(receiver.as_deref(), Some(Expr::Name { path, .. })
                        if path.len() == 1 && path[0] == "Arrays")
                && let Some(elem) = array_elem_type(&args[0], ctx)
            {
                desugar_expr(&mut args[0], None, ctx);
                if matches!(&args[1], Expr::Lambda { params, .. } if params.len() == 2) {
                    args[1] = build_erased_lambda(
                        &mut args[1],
                        "__Comparator",
                        "compare",
                        &TypeRef::Int,
                        &[elem.clone(), elem],
                        None,
                        ctx,
                    );
                } else {
                    let target = TypeRef::Generic {
                        base: String::from("Comparator"),
                        args: vec![elem],
                    };
                    desugar_expr(&mut args[1], Some(&target), ctx);
                }
                return;
            }
            // `list.sort(Comparator.comparing(p -> ...))`: the receiver's
            // element type is what the key extractor's parameter is, and only
            // this call site knows it. Handed down as the `Comparator<E>`
            // position the argument sits in.
            // A METHOD REFERENCE takes the same route: it is a comparator
            // expression like any other, and `list.sort(Cls::byX)` was refused
            // as if the position were not a functional-interface one.
            if method == "sort"
                && args.len() == 1
                && !matches!(&args[0], Expr::Lambda { .. })
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = list_elem_type(r, ctx)
            {
                let target = TypeRef::Generic {
                    base: String::from("Comparator"),
                    args: vec![elem],
                };
                desugar_expr(&mut args[0], Some(&target), ctx);
                return;
            }
            // `list.sort((a, b) -> ...)`: a two-parameter comparator whose
            // parameters are both the receiver's element type, returning int.
            if method == "sort"
                && args.len() == 1
                && matches!(&args[0], Expr::Lambda { params, .. } if params.len() == 2)
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = list_elem_type(r, ctx)
            {
                args[0] = build_erased_lambda(
                    &mut args[0],
                    "__Comparator",
                    "compare",
                    &TypeRef::Int,
                    &[elem.clone(), elem],
                    None,
                    ctx,
                );
                return;
            }
            // `list.add(() -> ...)` / `list.set(i, ...)` / `map.put(k, ...)`:
            // a lambda STORED in a collection whose element is a functional
            // interface — the callback registry and the strategy table. The
            // element type IS the target type, and only this call site knows
            // it; without it the lambda had no functional-interface position
            // to sit in, though a `Runnable` VARIABLE assigned the same lambda
            // worked. Handed down rather than built here, so a method
            // reference and a user-declared interface take the same route.
            let stored_at = match (method.as_str(), args.len()) {
                ("add" | "addFirst" | "addLast" | "push" | "offer", 1) => Some(0),
                ("add" | "set" | "put" | "putIfAbsent", 2) => Some(1),
                _ => None,
            };
            if let Some(at) = stored_at
                && matches!(&args[at], Expr::Lambda { .. } | Expr::MethodRef { .. })
                && let Some(r) = receiver.as_deref()
                && let Some(target) = if matches!(method.as_str(), "put" | "putIfAbsent") {
                    map_type_args(r, ctx).map(|(_, value)| value)
                } else {
                    list_elem_type(r, ctx)
                }
            {
                let (leading, tail) = args.split_at_mut(at);
                for arg in leading {
                    desugar_expr(arg, None, ctx);
                }
                desugar_expr(&mut tail[0], Some(&target), ctx);
                return;
            }
            // `Collections.sort(list, (a, b) -> ...)`: the comparator is the
            // second argument, its parameters the FIRST argument's element type.
            if method == "sort"
                && args.len() == 2
                && matches!(&args[1], Expr::Lambda { params, .. } if params.len() == 2)
                && let Some(elem) = list_elem_type(&args[0], ctx)
            {
                desugar_expr(&mut args[0], None, ctx);
                args[1] = build_erased_lambda(
                    &mut args[1],
                    "__Comparator",
                    "compare",
                    &TypeRef::Int,
                    &[elem.clone(), elem],
                    None,
                    ctx,
                );
                return;
            }
            // `Collections.max(c, cmp)` / `min(c, cmp)` / `binarySearch(l, key,
            // cmp)`: the comparator is the LAST argument and its parameters are
            // the FIRST argument's element type — the same rule `sort` above
            // has, which these three never got. Without it the lambda reached
            // overload resolution untyped, the two-argument form did not
            // apply, and the call silently resolved to the natural-ordering
            // one — leaving the comparator on the stack, which the verifier
            // reported as malformed bytecode at whatever ran next.
            if matches!(method.as_str(), "max" | "min" | "binarySearch")
                && receiver
                    .as_deref()
                    .is_some_and(|r| names_library_class(r, "Collections"))
                && matches!(args.last(), Some(Expr::Lambda { params, .. }) if params.len() == 2)
                && let Some(elem) = list_elem_type(&args[0], ctx)
            {
                let last = args.len() - 1;
                for arg in &mut args[..last] {
                    desugar_expr(arg, None, ctx);
                }
                args[last] = build_erased_lambda(
                    &mut args[last],
                    "__Comparator",
                    "compare",
                    &TypeRef::Int,
                    &[elem.clone(), elem],
                    None,
                    ctx,
                );
                return;
            }
            // `Arrays.setAll(array, i -> ...)`: the generator's parameter is the
            // int index, its result the array's element type.
            if method == "setAll"
                && args.len() == 2
                && receiver
                    .as_deref()
                    .is_some_and(|r| names_library_class(r, "Arrays"))
                && matches!(&args[1], Expr::Lambda { params, .. } if params.len() == 1)
                && let Some(elem) = array_elem_type(&args[0], ctx)
            {
                desugar_expr(&mut args[0], None, ctx);
                let object = TypeRef::Named(String::from("Object"));
                args[1] = build_erased_lambda(
                    &mut args[1],
                    "__UnaryOperator",
                    "apply",
                    &object,
                    &[TypeRef::Int],
                    Some(&elem),
                    ctx,
                );
                return;
            }
            // `stream.collect(supplier, accumulator, combiner)` — the form
            // that gathers without a `Collector`. The supplier takes nothing;
            // both consumers take the CONTAINER and one element, and the
            // container's type is what the supplier answers.
            if method == "collect"
                && args.len() == 3
                && let Some(stream_receiver) = receiver.as_deref()
            {
                let object = TypeRef::Named(String::from("Object"));
                let elem = stream_elem_type(stream_receiver, ctx).unwrap_or_else(|| object.clone());
                // The container is what the SUPPLIER makes. A constructor
                // reference says it outright (`ArrayList::new`), and it has to
                // be read: the accumulator is an UNBOUND reference on that
                // very type (`ArrayList::add`), so an `Object` container makes
                // the reference impossible rather than merely imprecise.
                let container = match &args[0] {
                    Expr::MethodRef {
                        qualifier, method, ..
                    } if method == "new" => {
                        let base = qualifier_type_name(qualifier);
                        if LIBRARY_CONTAINERS.contains(&base.as_str()) {
                            // A container holds REFERENCES, so a primitive
                            // stream's element is the wrapper: an
                            // `ArrayList<int>` is not a type, and the
                            // accumulator reference on one cannot be made.
                            TypeRef::Generic {
                                base,
                                args: vec![boxed_name(elem.clone())],
                            }
                        } else {
                            TypeRef::Named(base)
                        }
                    }
                    other => mapped_element_type(std::slice::from_ref(other), ctx)
                        .unwrap_or_else(|| object.clone()),
                };
                let shapes: [(&str, Vec<TypeRef>); 3] = [
                    ("get", Vec::new()),
                    ("accept", vec![container.clone(), elem.clone()]),
                    ("accept", vec![container.clone(), container]),
                ];
                for (index, (sam, params)) in shapes.into_iter().enumerate() {
                    if !matches!(&args[index], Expr::Lambda { .. } | Expr::MethodRef { .. }) {
                        desugar_expr(&mut args[index], None, ctx);
                        continue;
                    }
                    if matches!(&args[index], Expr::MethodRef { .. }) {
                        let synth = Sam {
                            method: String::from(sam),
                            params: params.clone(),
                            ret: object.clone(),
                        };
                        args[index] = method_ref_to_lambda(&args[index], &synth, ctx);
                    }
                    let interface = if index == 0 {
                        "__Supplier"
                    } else {
                        "__BiConsumer"
                    };
                    let ret = if index == 0 {
                        object.clone()
                    } else {
                        TypeRef::Void
                    };
                    args[index] = build_erased_lambda(
                        &mut args[index],
                        interface,
                        sam,
                        &ret,
                        &params,
                        None,
                        ctx,
                    );
                }
                return;
            }
            // `stream.collect(Collectors.groupingBy(e -> ...))` — a collector's
            // own lambdas see the STREAM's element, which only the enclosing
            // `collect` knows. This is the one place in caturra where a
            // lambda's target type comes from two levels up.
            if method == "collect"
                && args.len() == 1
                && let Some(stream_receiver) = receiver.as_deref()
                && let Some(elem) = stream_elem_type(stream_receiver, ctx)
                && is_collectors_call(&args[0])
            {
                desugar_collector(&mut args[0], &elem, ctx);
                return;
            }
            // The two INFINITE stream sources take their lambdas as a STATIC
            // call, so there is no receiver whose element could type them:
            // `Stream.iterate(seed, x -> x + 1)` steps from the seed's own
            // type, and `Stream.generate(() -> …)` takes a supplier.
            if matches!(method.as_str(), "iterate" | "generate")
                && let Some(owner) = receiver.as_deref()
                && (names_library_class(owner, "Stream")
                    || names_library_class(owner, "IntStream")
                    || names_library_class(owner, "LongStream")
                    || names_library_class(owner, "DoubleStream"))
            {
                let object = TypeRef::Named(String::from("Object"));
                if method == "generate" && args.len() == 1 {
                    let sam = Sam {
                        method: String::from("get"),
                        params: Vec::new(),
                        ret: object.clone(),
                    };
                    if matches!(args[0], Expr::MethodRef { .. }) {
                        args[0] = method_ref_to_lambda(&args[0], &sam, ctx);
                    }
                    args[0] = build_erased_lambda(
                        &mut args[0],
                        "__Supplier",
                        "get",
                        &object,
                        &[],
                        None,
                        ctx,
                    );
                    return;
                }
                if method == "iterate" && args.len() == 2 {
                    let elem = static_type_of(&args[0], ctx).unwrap_or_else(|| object.clone());
                    desugar_expr(&mut args[0], None, ctx);
                    // A method REFERENCE first becomes the equivalent lambda,
                    // as every other library callback does — handing one
                    // straight to the erased-lambda builder was a compiler
                    // PANIC, which is the one answer a program may never get.
                    let sam = Sam {
                        method: String::from("apply"),
                        params: vec![elem.clone()],
                        ret: object.clone(),
                    };
                    if matches!(args[1], Expr::MethodRef { .. }) {
                        args[1] = method_ref_to_lambda(&args[1], &sam, ctx);
                    }
                    args[1] = build_erased_lambda(
                        &mut args[1],
                        "__UnaryOperator",
                        "apply",
                        &object,
                        &[elem],
                        None,
                        ctx,
                    );
                    return;
                }
            }
            // A stream op with a lambda: the parameter type is the stream's
            // current element, walked back through the pipeline to `.stream()`.
            // (After a `map` the element is erased to `Object`.)
            if let Some(stream_receiver) = receiver.as_deref()
                && let Some(elem) = stream_elem_type(stream_receiver, ctx)
            {
                let object = TypeRef::Named(String::from("Object"));
                let single = match method.as_str() {
                    "filter" | "anyMatch" | "allMatch" | "noneMatch"
                    // `takeWhile`/`dropWhile` (Java 9) take the same predicate
                    // over the element that `filter` does.
                    | "takeWhile" | "dropWhile" => {
                        Some(("__Predicate", "test", TypeRef::Boolean))
                    }
                    "map" | "mapToObj" | "mapToInt" | "mapToLong" | "mapToDouble"
                    // `flatMap(f)` returns a STREAM per element, which the
                    // pipeline splices in; the SAM shape is `map`'s. The
                    // primitive forms take the same function and differ only in
                    // which pipeline they splice into — left out, a
                    // `flatMapToInt(Arrays::stream)` over a grid had no
                    // functional-interface position at all.
                    | "flatMap" | "flatMapToInt" | "flatMapToLong" | "flatMapToDouble" => {
                        Some(("__UnaryOperator", "apply", object.clone()))
                    }
                    "forEach" | "forEachOrdered" | "peek" => {
                        Some(("__Consumer", "accept", TypeRef::Void))
                    }
                    _ => None,
                };
                // `onClose(() -> …)` takes a RUNNABLE: no element, no result.
                // It is the one stream callback that is not a function of the
                // element, so it cannot ride the table above.
                if method == "onClose"
                    && matches!(&args[0..], [Expr::Lambda { params, .. }] if params.is_empty())
                {
                    args[0] = build_erased_lambda(
                        &mut args[0],
                        "__Runnable",
                        "run",
                        &TypeRef::Void,
                        &[],
                        None,
                        ctx,
                    );
                    return;
                }
                // `reduce(identity, (a, b) -> ...)` / `reduce((a, b) -> ...)`:
                // a two-parameter fold over the stream's own element type,
                // erased to the bundled `__BiFunction` like a map remapper.
                // A METHOD REFERENCE is the same fold written shorter
                // (`reduce(0, Integer::sum)`), and every other stream op
                // converts one to the equivalent lambda before erasing it —
                // this arm did not, so the reference had no functional
                // position and the whole program was refused.
                if method == "reduce"
                    && matches!(args.last(), Some(Expr::MethodRef { .. }))
                    && !args.is_empty()
                {
                    let synth = Sam {
                        method: String::from("apply"),
                        params: vec![elem.clone(), elem.clone()],
                        ret: object.clone(),
                    };
                    let last = args.len() - 1;
                    args[last] = method_ref_to_lambda(&args[last], &synth, ctx);
                }
                if method == "reduce"
                    && matches!(args.last(), Some(Expr::Lambda { params, .. }) if params.len() == 2)
                {
                    let last = args.len() - 1;
                    let (leading, tail) = args.split_at_mut(last);
                    for arg in leading {
                        desugar_expr(arg, None, ctx);
                    }
                    tail[0] = build_erased_lambda(
                        &mut tail[0],
                        "__BiFunction",
                        "apply",
                        &object,
                        &[elem.clone(), elem],
                        None,
                        ctx,
                    );
                    return;
                }
                // The argument is either a single-parameter lambda or a method
                // reference (`map(String::toUpperCase)`). A reference first
                // becomes the equivalent one-parameter lambda — its SAM element
                // is the stream's current element — then goes through the same
                // erasure as a written lambda.
                let is_lambda =
                    matches!(&args[0..], [Expr::Lambda { params, .. }] if params.len() == 1);
                let is_method_ref = matches!(&args[0..], [Expr::MethodRef { .. }]);
                if let Some((iface, sam, ret)) = single
                    && (is_lambda
                        || is_method_ref
                        || (iface == "__Predicate"
                            && args.len() == 1
                            && is_negated_predicate(&args[0])))
                {
                    if is_method_ref {
                        let synth = Sam {
                            method: sam.to_owned(),
                            params: vec![elem.clone()],
                            ret: ret.clone(),
                        };
                        args[0] = method_ref_to_lambda(&args[0], &synth, ctx);
                    }
                    args[0] =
                        build_erased_lambda(&mut args[0], iface, sam, &ret, &[elem], None, ctx);
                    return;
                }
                if matches!(method.as_str(), "sorted" | "max" | "min")
                    && args.len() == 1
                    && matches!(&args[0], Expr::Lambda { params, .. } if params.len() == 2)
                {
                    args[0] = build_erased_lambda(
                        &mut args[0],
                        "__Comparator",
                        "compare",
                        &TypeRef::Int,
                        &[elem.clone(), elem],
                        None,
                        ctx,
                    );
                    return;
                }
                // `sorted(Comparator.comparing(s -> s.length()))` — the key
                // extractor's parameter is the STREAM's element, which only
                // this op knows. Without it the lambda parameter typed as
                // `Object` and `s.length()` was "cannot find symbol".
                if matches!(method.as_str(), "sorted" | "max" | "min") && args.len() == 1 {
                    let target = TypeRef::Generic {
                        base: String::from("Comparator"),
                        args: vec![elem],
                    };
                    desugar_expr(&mut args[0], Some(&target), ctx);
                    return;
                }
            }
            // `list.add(() -> ...)` / `list.set(i, () -> ...)`: the element
            // argument's target type is the receiver's declared element type,
            // not a user method signature. `add`/`set` take the element last.
            if matches!(method.as_str(), "add" | "set")
                && !args.is_empty()
                && matches!(
                    args.last(),
                    Some(Expr::Lambda { .. } | Expr::MethodRef { .. })
                )
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = list_elem_type(r, ctx)
                && interface_name(&elem).is_some_and(|n| ctx.sams.contains_key(n))
            {
                let last = args.len() - 1;
                for (index, arg) in args.iter_mut().enumerate() {
                    let expected = (index == last).then(|| elem.clone());
                    desugar_expr(arg, expected.as_ref(), ctx);
                }
                return;
            }
            // Target typing for a lambda passed as an argument. A lambda has no
            // type of its own; it takes the one the parameter it lands on
            // declares, so this has to know which declaration is being called.
            //
            // Only the overloads whose arity matches can be, and they only pin
            // a type if they agree on it. That is what `assertDoesNotThrow`
            // needs — it is declared twice, `(Executable)` and
            // `(Executable, String)`, and the argument count says which — and
            // it was previously out of reach, because this insisted the method
            // be declared exactly once. Overloads that disagree at a position
            // leave it untyped, as before.
            // A COMBINATOR takes its lambda's parameter from the receiver's
            // own type arguments: `f.andThen(n -> n + 1)` on a
            // `Function<String, Integer>` gives `n` the type Integer. Without
            // it the bundled SAM's `Object` parameter reached the body and
            // `n + 1` was "operator '+' cannot be applied to Object and int".
            if let Some(target) = receiver
                .as_deref()
                .and_then(|r| combinator_argument_type(r, method, args.len(), ctx))
            {
                for arg in args.iter_mut() {
                    desugar_expr(arg, Some(&target), ctx);
                }
                return;
            }
            let owner = receiver
                .as_deref()
                .and_then(|r| receiver_class_name(r, ctx));
            let sigs_for_call = owner
                .and_then(|class| ctx.methods_in_class.get(&(class, method.clone())))
                .or_else(|| ctx.methods.get(method));
            let param_types = sigs_for_call.and_then(|sigs| {
                let mut matching = sigs.iter().filter(|params| params.len() == args.len());
                let first = matching.next()?;
                matching
                    .all(|params| params == first)
                    .then(|| first.clone())
            });
            // A GENERIC method's lambda argument is target-typed by its
            // declared parameter with the type variables PUT BACK: erasure
            // turned `Box<T>` into a wildcard that no longer says which
            // variable it held, so `pick("abc", s -> s.length())` had no
            // element for `s` and was refused outright.
            let witness = type_args.clone();
            let substituted =
                generic_argument_targets(method, receiver.as_deref(), args, &witness, ctx);
            // A witness is a claim about the arguments, not just about the
            // result: `W.<String>id(5)` states that `5` is a String, and javac
            // says so. Without the witness the same call INFERS `T` from the
            // argument and cannot be wrong, which is why nothing checked it.
            if !witness.is_empty() {
                let span = *span;
                check_witnessed_arguments(substituted.as_deref(), args, span, ctx);
            }
            for (index, arg) in args.iter_mut().enumerate() {
                let expected = substituted
                    .as_ref()
                    .and_then(|types| types[index].clone())
                    .or_else(|| param_types.as_ref().map(|types| types[index].clone()));
                desugar_expr(arg, expected.as_ref(), ctx);
            }
        }
        Expr::NewObject {
            class,
            type_args,
            args,
            ..
        } => {
            // A HOISTED body (an anonymous class, a local class) is walked as
            // a class of its own, long after this expression. What is in scope
            // HERE is what it was written inside, so record it for that walk.
            if ctx.hoisted.contains(class) && !ctx.captured_scopes.contains_key(class) {
                let mut visible: HashMap<String, TypeRef> = HashMap::new();
                for frame in &ctx.scope {
                    for (name, ty) in frame {
                        visible.insert(name.clone(), ty.clone());
                    }
                }
                ctx.captured_scopes.insert(class.clone(), visible);
            }
            // A comparator lambda argument to a sorted collection's constructor
            // (`TreeSet`/`TreeMap`/`PriorityQueue`): its parameters are the
            // element / key type, read from the `new`'s own type arguments or,
            // for a diamond, the declaration target it initializes. For a
            // `PriorityQueue(capacity, cmp)` the lambda is the last argument.
            if matches!(
                simple_base(class.as_str()),
                "TreeSet" | "TreeMap" | "PriorityQueue"
            ) && (matches!(args.last(), Some(Expr::Lambda { params, .. }) if params.len() == 2)
                || matches!(args.last(), Some(Expr::MethodRef { .. })))
                && let Some(elem) = type_args
                    .first()
                    .cloned()
                    .or_else(|| expected.and_then(sorted_ctor_elem))
            {
                let last = args.len() - 1;
                // A method REFERENCE is the same comparator written shorter —
                // `new TreeMap<>(String::compareTo)` — and was refused as "not
                // a functional-interface position" beside the lambda that is.
                if matches!(&args[last], Expr::MethodRef { .. }) {
                    let synth = Sam {
                        method: String::from("compare"),
                        params: vec![elem.clone(), elem.clone()],
                        ret: TypeRef::Int,
                    };
                    args[last] = method_ref_to_lambda(&args[last], &synth, ctx);
                }
                args[last] = build_erased_lambda(
                    &mut args[last],
                    "__Comparator",
                    "compare",
                    &TypeRef::Int,
                    &[elem.clone(), elem],
                    None,
                    ctx,
                );
                return;
            }
            // Target-type a lambda constructor argument (`new Timer(40, e -> …)`)
            // when exactly one constructor of the class takes this many args.
            let param_types = ctx.constructors.get(class).and_then(|sigs| {
                let mut matching = sigs.iter().filter(|s| s.len() == args.len());
                let first = matching.next()?;
                match matching.next() {
                    None => Some(first.clone()),
                    Some(_) => None, // ambiguous arity — leave untyped
                }
            });
            for (index, arg) in args.iter_mut().enumerate() {
                let expected = param_types.as_ref().map(|types| types[index].clone());
                desugar_expr(arg, expected.as_ref(), ctx);
            }
        }
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                desugar_expr(a, None, ctx);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            desugar_expr(lhs, None, ctx);
            desugar_expr(rhs, None, ctx);
        }
        // A CAST is a target type (`(Op) P::m` is the standard way to give a
        // reference one where nothing else would).
        Expr::Cast { ty, operand, .. } => {
            let ty = ty.clone();
            desugar_expr(operand, Some(&ty), ctx);
        }
        Expr::Unary { operand, .. }
        | Expr::Field {
            object: operand, ..
        }
        | Expr::InstanceOf { value: operand, .. } => desugar_expr(operand, None, ctx),
        Expr::Index { array, index, .. } => {
            desugar_expr(array, None, ctx);
            desugar_expr(index, None, ctx);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            desugar_expr(cond, None, ctx);
            // A ternary yielding a lambda inherits the outer target type.
            desugar_expr(then, expected, ctx);
            desugar_expr(els, expected, ctx);
        }
        Expr::IncDec { target, .. } => desugar_expr(target, None, ctx),
        Expr::NewArray {
            elem, dims, init, ..
        } => {
            let element = elem.clone();
            for d in dims.iter_mut().flatten() {
                desugar_expr(d, None, ctx);
            }
            if let Some(elems) = init {
                for e in elems {
                    desugar_expr(e, Some(&element), ctx);
                }
            }
        }
        // `Op[] ops = { P::m }`: each element is target-typed by the array's
        // ELEMENT type, one dimension down.
        Expr::ArrayLiteral { elements, .. } => {
            let element = match expected {
                Some(TypeRef::Array(inner)) => Some((**inner).clone()),
                _ => None,
            };
            for e in elements {
                desugar_expr(e, element.as_ref(), ctx);
            }
        }
        Expr::Assign { target, value, .. } => {
            match target {
                crate::ast::AssignTarget::Index { array, index } => {
                    desugar_expr(array, None, ctx);
                    desugar_expr(index, None, ctx);
                }
                crate::ast::AssignTarget::Field { object, .. } => desugar_expr(object, None, ctx),
                crate::ast::AssignTarget::Var(_) => {}
            }
            // The value takes the target's type, so `f = () -> ...` finds its
            // functional interface just as `f = ...` in a declaration does.
            let expected = assign_target_type(target, ctx);
            desugar_expr(value, expected.as_ref(), ctx);
        }
        Expr::Lambda { .. }
        | Expr::MethodRef { .. }
        | Expr::Literal { .. }
        | Expr::Name { .. }
        | Expr::This { .. }
        | Expr::Super { .. } => {}
    }
}

/// Convert a method reference into an equivalent lambda, choosing the
/// call form (static, unbound-instance, bound-instance, or
/// constructor) from the qualifier and the SAM's arity.
/// JLS §15.13.1: decide which of the four method-reference forms applies and
/// report the ones javac rejects — a static method named through an
/// INSTANCE, a name that fits both the static and the unbound-instance form
/// (ambiguous), an arity that fits neither, and a referenced method whose
/// checked exceptions the functional interface does not allow.
///
/// Silent when the qualifier's class is unknown (a library type, a generic
/// position): the checks must never fire on something caturra cannot see.
#[allow(clippy::too_many_lines)] // one arm per reference form
fn validate_method_ref(expr: &Expr, sam: &Sam, ctx: &mut Ctx) {
    let Expr::MethodRef {
        qualifier,
        method,
        span,
    } = expr
    else {
        return;
    };
    let arity = sam.params.len();
    // `Type::new` on an enum: there is no constructor to reference.
    if method == "new"
        && let Expr::Name { path, .. } = qualifier.as_ref()
        && path.len() == 1
        && ctx.enums.contains(&path[0])
    {
        ctx.diags.push(crate::diagnostics::Diagnostic::error(
            ctx.path,
            "enum types may not be instantiated",
            *span,
        ));
        return;
    }
    if method == "new" {
        return;
    }
    // Which class is being searched, and whether the qualifier names a TYPE
    // (static or unbound-instance forms) or a VALUE (bound form).
    let (class, by_type) = match qualifier.as_ref() {
        Expr::Name { path, .. } if path.len() == 1 => {
            if let Some(TypeRef::Named(named) | TypeRef::Generic { base: named, .. }) =
                ctx.lookup(&path[0])
            {
                (named, false)
            } else if ctx.class_names.contains(&path[0]) {
                (path[0].clone(), true)
            } else {
                return;
            }
        }
        _ => return,
    };
    let Some(shapes) = ctx.shapes.get(&class) else {
        return; // a library class: caturra cannot enumerate its overloads
    };
    let named: Vec<&MethodShape> = shapes.iter().filter(|m| m.name == *method).collect();
    if named.is_empty() {
        return; // resolution reports the missing method itself
    }
    let describe = |m: &MethodShape| format!("{}({} args)", m.name, m.arity);
    if by_type {
        // A static method takes the SAM's arguments; an unbound instance one
        // takes the receiver FIRST, so it fits one argument fewer.
        let statics: Vec<&&MethodShape> = named
            .iter()
            .filter(|m| m.is_static && m.takes(arity))
            .collect();
        let unbound: Vec<&&MethodShape> = named
            .iter()
            .filter(|m| !m.is_static && arity > 0 && m.takes(arity - 1))
            .collect();
        if !statics.is_empty() && !unbound.is_empty() {
            ctx.diags.push(crate::diagnostics::Diagnostic::error(
                ctx.path,
                format!(
                    // javac's headline for a reference it cannot make sense
                    // of; what caturra has to add rides after it, the way
                    // javac's own continuation line does.
                    "incompatible types: invalid method reference\n  \
                     reference to {method} is ambiguous"
                ),
                *span,
            ));
            return;
        }
        if statics.is_empty() && unbound.is_empty() {
            // Whichever kind exists is the one to blame, in javac's words.
            let instance = named.iter().find(|m| !m.is_static);
            let message = match instance {
                Some(m) => format!(
                    "incompatible types: invalid method reference\n  \
                     unexpected instance method {}",
                    describe(m)
                ),
                None => format!(
                    "incompatible types: invalid method reference\n  \
                     unexpected static method {}",
                    named.first().map_or_else(String::new, |m| describe(m))
                ),
            };
            ctx.diags.push(crate::diagnostics::Diagnostic::error(
                ctx.path, message, *span,
            ));
            return;
        }
        check_thrown(statics.first().or(unbound.first()).map(|m| **m), ctx, *span);
        return;
    }
    // A BOUND reference: the method must be an instance method of the
    // receiver's class taking exactly the SAM's arguments.
    if named.iter().all(|m| m.is_static) {
        ctx.diags.push(crate::diagnostics::Diagnostic::error(
            ctx.path,
            format!(
                "incompatible types: invalid method reference\n  \
                 unexpected static method {}",
                named.first().map_or_else(String::new, |m| describe(m))
            ),
            *span,
        ));
        return;
    }
    let chosen = named.iter().find(|m| !m.is_static && m.takes(arity));
    check_thrown(chosen.copied(), ctx, *span);
}

/// A method reference may not throw checked exceptions its functional
/// interface does not declare (JLS §15.13.2). caturra's user functional
/// interfaces are the ones it can see; a SAM that declares nothing allows
/// nothing.
fn check_thrown(chosen: Option<&MethodShape>, ctx: &mut Ctx, span: crate::diagnostics::SourceSpan) {
    let Some(chosen) = chosen else {
        return;
    };
    // Only the library throwables can be judged here (a user exception's
    // ancestry needs the method table, which this pass does not build); an
    // unknown name is left alone rather than reported wrongly.
    let unchecked = |name: &String| {
        let simple = name.rsplit(['.', '/']).next().unwrap_or(name);
        match caturra_classfile::exceptions::internal_name_of(simple) {
            Some(internal) => {
                caturra_classfile::exceptions::is_exception_subclass(
                    internal,
                    "java/lang/RuntimeException",
                ) || caturra_classfile::exceptions::is_exception_subclass(
                    internal,
                    "java/lang/Error",
                )
            }
            None => true,
        }
    };
    if let Some(thrown) = chosen.throws.iter().find(|name| !unchecked(name)) {
        ctx.diags.push(crate::diagnostics::Diagnostic::error(
            ctx.path,
            format!("incompatible thrown types {thrown} in functional expression"),
            span,
        ));
    }
}

/// The bridge a `super::m` reference calls: an ordinary instance method on
/// the enclosing class whose body makes the non-virtual super call.
fn super_bridge(
    name: &str,
    params: Vec<Param>,
    return_type: TypeRef,
    body: Vec<Stmt>,
    span: crate::diagnostics::SourceSpan,
) -> MethodDecl {
    MethodDecl {
        name: name.to_owned(),
        is_static: false,
        is_public: false,
        is_private: false,
        is_protected: false,
        is_final: false,
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        infer_return: None,
        declared_params: Vec::new(),
        type_var_sources: Vec::new(),
        return_type,
        params,
        body,
        annotations: Vec::new(),
        throws: Vec::new(),
        span,
        pre_init: 0,
        declared_return: None,
    }
}

#[allow(clippy::too_many_lines)] // one arm per reference form
fn method_ref_to_lambda(expr: &Expr, sam: &Sam, ctx: &mut Ctx) -> Expr {
    let Expr::MethodRef {
        qualifier,
        method,
        span,
    } = expr
    else {
        unreachable!("guarded by caller");
    };
    // The class built for THIS lambda is a method-reference class: its captures
    // follow the reference's rules, and — since a JDK's `invokedynamic` calls
    // the target directly — it has no stack-trace frame and takes no number in
    // javac's per-class lambda sequence. Marked here rather than at each call
    // site: the STREAM ops converted a reference without marking it, so
    // `map(Item::score)` was counted as a lambda and every lambda after it in
    // the same class was numbered one too high.
    ctx.class_prefix = crate::METHOD_REF_CLASS_PREFIX;
    let span = *span;
    let arity = sam.params.len();
    let param_names: Vec<String> = (0..arity).map(|i| format!("__p{i}")).collect();
    let name_expr = |name: &str| Expr::Name {
        path: vec![name.to_owned()],
        span,
    };

    // `super::m` — the synthesized lambda class cannot make a non-virtual
    // call on the enclosing object's superclass, so the enclosing class gets
    // a bridge that does it and the reference targets THAT (javac emits the
    // same shape). The SAM gives the bridge its signature.
    if matches!(qualifier.as_ref(), Expr::Super { .. }) {
        let bridge = format!("__caturraSuper${method}");
        if !ctx.bridges.iter().any(|m| m.name == bridge) {
            let params: Vec<Param> = sam
                .params
                .iter()
                .enumerate()
                .map(|(i, ty)| Param {
                    ty: ty.clone(),
                    name: format!("__s{i}"),
                    is_varargs: false,
                    is_final: false,
                })
                .collect();
            let args: Vec<Expr> = params
                .iter()
                .map(|p| Expr::Name {
                    path: vec![p.name.clone()],
                    span,
                })
                .collect();
            let call = Expr::SuperMethodCall {
                owner: None,
                method: method.clone(),
                args,
                span,
            };
            let body = if matches!(sam.ret, TypeRef::Void) {
                vec![Stmt::Expr(call)]
            } else {
                vec![Stmt::Return {
                    value: Some(call),
                    span,
                }]
            };
            ctx.bridges
                .push(super_bridge(&bridge, params, sam.ret.clone(), body, span));
        }
        return Expr::Lambda {
            params: param_names
                .iter()
                .map(|name| crate::ast::LambdaParam {
                    name: name.clone(),
                    ty: None,
                })
                .collect(),
            body: LambdaBody::Expr(Box::new(Expr::Call {
                receiver: Some(Box::new(Expr::This { span })),
                method: bridge,
                args: param_names.iter().map(|n| name_expr(n)).collect(),
                span,
                type_args: Vec::new(),
            })),
            span,
        };
    }
    // Is the qualifier a class name? A QUALIFIED library class names the same
    // type as its simple spelling and must take the same route — the
    // one-segment test saw only the simple one, so `java.lang.String::length`
    // and `Map.Entry::getKey` fell to the BOUND form and compiled to
    // `java.lang.String.length(p0)` and `Map.Entry.getKey(entry)`: calls on a
    // type rather than through it.
    let qualifier_class = match qualifier.as_ref() {
        Expr::Name { path, .. } if path.len() == 1 && ctx.class_names.contains(&path[0]) => {
            Some(path[0].clone())
        }
        Expr::Name { path, .. } if path.len() > 1 => {
            crate::imports::nested_library_class(&path.join(".")).map(String::from)
        }
        _ => None,
    };
    // A NESTED LIBRARY type as the qualifier — `Map.Entry::getKey`, the
    // reference every stream over an `entrySet()` reaches for. The test above
    // sees one segment only, so this fell through to the BOUND form and
    // compiled to `Map.Entry.getKey(entry)`: a static call on a type that has
    // no such static. The receiver parameter keeps the type the SAM gives it
    // (the stream's element), which is more precise than the raw qualifier.
    // For an unbound-instance ref (`Person::getAge`), the first parameter IS
    // the receiver and must be typed as the qualifier class, so the call
    // resolves against it rather than `Object`.
    let mut receiver_param_type: Option<TypeRef> = None;
    let call: Expr = if method == "new" {
        // Constructor reference: `new Type(p0, ...)`.
        let class = qualifier_type_name(qualifier);
        Expr::NewObject {
            class,
            type_args: Vec::new(),
            args: param_names.iter().map(|n| name_expr(n)).collect(),
            outer: None,
            span,
        }
    } else if let Some(class) = qualifier_class {
        let is_static = ctx
            .static_methods
            .get(&class)
            .is_some_and(|set| set.contains(method))
            // ...or a LIBRARY static the VM answers, which no bundled class
            // declares. The guard was "not a class the program (or the bundle)
            // declares", and caturra bundles an `Arrays` — so `Arrays::stream`
            // fell through to the unbound-INSTANCE form and compiled to
            // `row.stream()`. A bundled class that declares the name itself
            // still wins, above.
            || (!declares_instance_method(ctx, &class, method)
                && is_library_static(&class, method));
        if is_static {
            // `Type.method(p0, ...)`.
            Expr::Call {
                receiver: Some(Box::new(name_expr(&class))),
                method: method.clone(),
                args: param_names.iter().map(|n| name_expr(n)).collect(),
                span,
                type_args: Vec::new(),
            }
        } else if param_names.is_empty() {
            // An unbound instance reference needs the SAM to supply a receiver
            // as its first parameter, and this SAM has none — `S::getV` for a
            // `Supplier<Integer>`. `validate_method_ref` has already reported
            // it ("unexpected instance method"); building the call anyway
            // indexed an empty parameter list and PANICKED the compiler, which
            // is the one failure mode worse than a wrong answer.
            Expr::Call {
                receiver: Some(qualifier.clone()),
                method: method.clone(),
                args: Vec::new(),
                span,
                type_args: Vec::new(),
            }
        } else {
            // Unbound instance: `p0.method(p1, ...)`.
            //
            // A NESTED library type (`Map.Entry`) is left untyped: the SAM
            // gives the receiver the stream's element, which carries its type
            // arguments, where the bare qualifier would be the raw type and
            // `getKey()` would answer `Object`.
            if !class.contains('.') && !LIBRARY_CONTAINERS.contains(&class.as_str()) {
                receiver_param_type = Some(TypeRef::Named(class));
            }
            Expr::Call {
                receiver: Some(Box::new(name_expr(&param_names[0]))),
                method: method.clone(),
                args: param_names[1..].iter().map(|n| name_expr(n)).collect(),
                span,
                type_args: Vec::new(),
            }
        }
    } else {
        // Bound instance: `qualifier.method(p0, ...)`.
        Expr::Call {
            receiver: Some(qualifier.clone()),
            method: method.clone(),
            args: param_names.iter().map(|n| name_expr(n)).collect(),
            span,
            type_args: Vec::new(),
        }
    };

    let params = param_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| crate::ast::LambdaParam {
            name,
            ty: if i == 0 {
                receiver_param_type.clone()
            } else {
                None
            },
        })
        .collect();
    Expr::Lambda {
        params,
        body: LambdaBody::Expr(Box::new(call)),
        span,
    }
}

/// Whether the program (or a bundled class) declares `method` on `class` as an
/// INSTANCE method — the one thing that must beat the library-static table,
/// since a class of one's own shadows a library name entirely.
fn declares_instance_method(ctx: &Ctx, class: &str, method: &str) -> bool {
    ctx.shapes.get(class).is_some_and(|shapes| {
        shapes
            .iter()
            .any(|shape| shape.name == method && !shape.is_static)
    })
}

/// The type name a constructor-reference qualifier denotes.
fn qualifier_type_name(qualifier: &Expr) -> String {
    match qualifier {
        Expr::Name { path, .. } => path.join("."),
        _ => String::from("Object"),
    }
}

/// The simple interface name of a target type (`Named`/`Generic`).
fn interface_name(ty: &TypeRef) -> Option<&str> {
    match ty {
        TypeRef::Named(name) | TypeRef::Generic { base: name, .. } => Some(name.as_str()),
        _ => None,
    }
}

/// The functional interface a target type names, and its SAM.
///
/// A NESTED interface is hoisted to the top level under its SIMPLE name, so
/// `Outer.Inner` — the spelling javac REQUIRES from outside `Outer` — missed a
/// map keyed by `Inner`, and a lambda there was refused as though the position
/// were not a functional-interface one. The anonymous-class form of the same
/// target compiled, which is what made the gap look like a lambda rule.
///
/// The written name is tried first, so a top-level interface of the same name
/// still wins, and the name RETURNED is the one that matched — the
/// synthesized lambda class implements it, so it has to be a name codegen can
/// resolve.
fn sam_target(target: &TypeRef, ctx: &Ctx) -> Option<(String, Sam)> {
    let name = interface_name(target)?;
    // An UNQUALIFIED name is resolved in the SCOPE it was written in. Two
    // enclosing classes may each declare a `Go`, and this map — one entry per
    // spelling — can only hold one of them under the simple name, so a lambda
    // in the other class targeted the wrong interface and was refused as
    // "cannot be converted". The enclosing chain is walked innermost-first,
    // the way codegen resolves the same name.
    if !name.contains('.') {
        let mut chain = ctx.frame_owner.0.replace('$', ".");
        loop {
            let key = format!("{chain}.{name}");
            if let Some(sam) = ctx.sams.get(&key) {
                // The name as WRITTEN is returned whenever it names the same
                // interface: it is the spelling every later pass already
                // handles, and only the ambiguous case needs the other one.
                let same_interface = ctx.sam_owners.get(name) == ctx.sam_owners.get(&key);
                return match ctx.sams.get(name).filter(|_| same_interface) {
                    Some(plain) => Some((name.to_owned(), plain.clone())),
                    None => Some((key, sam.clone())),
                };
            }
            match chain.rsplit_once('.') {
                Some((head, _)) => chain = head.to_owned(),
                None => break,
            }
        }
    }
    if let Some(sam) = ctx.sams.get(name) {
        return Some((name.to_owned(), sam.clone()));
    }
    let (_, last) = name.rsplit_once('.')?;
    ctx.sams.get(last).map(|sam| (last.to_owned(), sam.clone()))
}

/// `m.merge(k, v, g)` with `g` a declared function variable: javac checks the
/// variable's type arguments against the map's — `merge` wants
/// `BiFunction<? super V, ? super V, ? extends V>` — but they erase before
/// codegen, so the mismatch would otherwise compile and CCE at run time.
/// Only a PROVABLE mismatch is rejected: both sides concrete library types,
/// which are final, so the wildcards collapse to equality (an `Object` input
/// stays accepted — it satisfies any `? super`).
fn check_map_function_variable(
    declared: &TypeRef,
    inputs: &[TypeRef],
    result: &TypeRef,
    span: crate::diagnostics::SourceSpan,
    ctx: &mut Ctx,
) {
    let TypeRef::Generic { base, args } = declared else {
        return;
    };
    let simple = base.rsplit('.').next().unwrap_or(base);
    let (declared_inputs, declared_result): (Vec<&TypeRef>, &TypeRef) =
        match (simple, args.as_slice()) {
            ("BiFunction", [a, b, c]) => (vec![a, b], c),
            ("BinaryOperator", [t]) => (vec![t, t], t),
            ("Function", [a, b]) => (vec![a], b),
            ("UnaryOperator", [t]) => (vec![t], t),
            _ => return,
        };
    if declared_inputs.len() != inputs.len() {
        return;
    }
    let input_wrong = inputs.iter().zip(&declared_inputs).any(
        |(need, have)| matches!((concrete(need), concrete(have)), (Some(n), Some(h)) if n != h),
    );
    let result_wrong =
        matches!((concrete(result), concrete(declared_result)), (Some(n), Some(h)) if n != h);
    if !(input_wrong || result_wrong) {
        return;
    }
    let iface = if inputs.len() == 2 {
        "BiFunction"
    } else {
        "Function"
    };
    let supers: Vec<String> = inputs
        .iter()
        .map(|t| format!("? super {}", render_type(t)))
        .collect();
    let required = format!(
        "{iface}<{},? extends {}>",
        supers.join(","),
        render_type(result)
    );
    ctx.diags.push(crate::diagnostics::Diagnostic::error(
        ctx.path,
        format!(
            "incompatible types: {} cannot be converted to {required}",
            render_type(declared)
        ),
        span,
    ));
}

/// Report an argument a WITNESS says is something it is not. Only on footing
/// where the mismatch is provable — a concrete final type against a primitive
/// or another concrete final type — because a wrong REJECTION is worse than
/// the missing check it replaces.
fn check_witnessed_arguments(
    targets: Option<&[Option<TypeRef>]>,
    args: &[Expr],
    span: crate::diagnostics::SourceSpan,
    ctx: &mut Ctx,
) {
    let Some(targets) = targets else {
        return;
    };
    for (target, arg) in targets.iter().zip(args) {
        let Some(wanted) = target.as_ref().and_then(concrete) else {
            continue;
        };
        let Some(actual) = static_type_of(arg, ctx) else {
            continue;
        };
        // A primitive argument boxes to exactly one wrapper (JLS §5.1.7), so
        // any other wrapper is a mismatch — `<Long>` does not accept an `int`,
        // as no boxing-then-widening conversion exists.
        let (described, boxes_to) = match actual {
            TypeRef::Int => ("int", "Integer"),
            TypeRef::Long => ("long", "Long"),
            TypeRef::Double => ("double", "Double"),
            TypeRef::Float => ("float", "Float"),
            TypeRef::Short => ("short", "Short"),
            TypeRef::Byte => ("byte", "Byte"),
            TypeRef::Char => ("char", "Character"),
            TypeRef::Boolean => ("boolean", "Boolean"),
            ref other => match concrete(other) {
                Some(name) => (name, name),
                None => continue,
            },
        };
        if boxes_to == wanted {
            continue;
        }
        ctx.diags.push(crate::diagnostics::Diagnostic::error(
            ctx.path,
            format!("incompatible types: {described} cannot be converted to {wanted}"),
            span,
        ));
    }
}

/// A concrete FINAL library type, for which `? super`/`? extends` collapse
/// to equality — the only footing on which a mismatch is provable.
fn concrete(t: &TypeRef) -> Option<&str> {
    match t {
        TypeRef::Named(n)
            if matches!(
                n.as_str(),
                "String"
                    | "Integer"
                    | "Double"
                    | "Boolean"
                    | "Character"
                    | "Long"
                    | "Short"
                    | "Byte"
                    | "Float"
            ) =>
        {
            Some(n.as_str())
        }
        _ => None,
    }
}

/// A source-level rendering of a type reference, for diagnostics.
fn render_type(t: &TypeRef) -> String {
    match t {
        TypeRef::Named(n) => n.clone(),
        TypeRef::Generic { base, args } => {
            let rendered: Vec<String> = args.iter().map(render_type).collect();
            format!("{base}<{}>", rendered.join(","))
        }
        _ => String::from("Object"),
    }
}

/// The type a call with an explicit type WITNESS returns
/// (`Collections.<String>emptyList()` is a `List<String>`, JLS §15.12.2.1).
///
/// A witness is the only thing an argument-less generic call can be typed by,
/// so without this a lambda written against the result had no element type and
/// was refused for having no functional-interface position — while the same
/// call assigned to a declared variable first compiled.
fn witnessed_return(call: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    let Expr::Call {
        receiver,
        method,
        args,
        type_args,
        ..
    } = call
    else {
        return None;
    };
    if type_args.is_empty() {
        return None;
    }
    // A method of the program: substitute the witness into the return type it
    // declares, positionally, which is what the witness means.
    let own = matches!(receiver.as_deref(), None | Some(Expr::This { .. }));
    let class = if own {
        ctx.current_class.map(ToOwned::to_owned)
    } else {
        match receiver.as_deref() {
            Some(Expr::Name { path, .. }) if path.len() == 1 => Some(path[0].clone()),
            _ => None,
        }
    };
    if let Some(class) = class.as_deref()
        && let Some(shapes) = ctx.shapes.get(class)
        && let Some(shape) = shapes
            .iter()
            .find(|shape| shape.name == *method && shape.takes(args.len()))
        && shape.type_params.len() == type_args.len()
    {
        let bound: HashMap<String, TypeRef> = shape
            .type_params
            .iter()
            .cloned()
            .zip(type_args.iter().cloned())
            .collect();
        return substitute_vars(&shape.return_type, &bound, &shape.type_params);
    }
    // A LIBRARY factory whose element type comes from nowhere else. Only the
    // ones whose return shape is written down here: guessing that any
    // one-argument witness means a collection of it would mistype
    // `Collections.<String>max(…)`, which returns the element itself.
    let owner = receiver.as_deref()?;
    let base = match (method.as_str(), type_args.len()) {
        ("emptyList" | "singletonList" | "nCopies", 1)
            if names_library_class(owner, "Collections") =>
        {
            "List"
        }
        ("emptySet" | "singleton", 1) if names_library_class(owner, "Collections") => "Set",
        ("emptyMap", 2) if names_library_class(owner, "Collections") => "Map",
        ("asList", 1) if names_library_class(owner, "Arrays") => "List",
        ("of" | "copyOf", 1) if names_library_class(owner, "List") => "List",
        ("of" | "copyOf", 1) if names_library_class(owner, "Set") => "Set",
        ("of" | "copyOf", 2) if names_library_class(owner, "Map") => "Map",
        ("of" | "empty", 1) if names_library_class(owner, "Stream") => "Stream",
        _ => return None,
    };
    Some(TypeRef::Generic {
        base: String::from(base),
        args: type_args.clone(),
    })
}

/// The declared key and value types of a `Map`/`HashMap` receiver, read
/// syntactically from the local, parameter or field it names. `getMap()
/// .forEach(...)` has no declaration to read, so its lambda is left in place
/// and codegen reports the honest "lambdas are not supported" it always did.
/// The key and value of a MAP-building collector — `stream.collect(
/// Collectors.groupingBy(f))` and its family. The classifier's answer is the
/// key, and the value is the downstream collector's result (a list of the
/// stream's own element when there is none). By the time an outer call asks,
/// the classifier is already a synthesized CLASS, which is what says what it
/// answers.
fn collector_map_types(collector: &Expr, source: &Expr, ctx: &Ctx) -> Option<(TypeRef, TypeRef)> {
    let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        ..
    } = collector
    else {
        return None;
    };
    if !matches!(owner.as_ref(), Expr::Name { path, .. }
        if path.last().is_some_and(|name| name == "Collectors"))
    {
        return None;
    }
    let object = || TypeRef::Named(String::from("Object"));
    let list_of = |elem: TypeRef| TypeRef::Generic {
        base: String::from("List"),
        args: vec![elem],
    };
    let elem = || stream_elem_type(source, ctx).unwrap_or_else(object);
    let key = match (method.as_str(), args.first()) {
        ("partitioningBy", _) => TypeRef::Named(String::from("Boolean")),
        ("groupingBy" | "toMap", Some(classifier)) => {
            boxed_name(mapped_element_type(std::slice::from_ref(classifier), ctx)?)
        }
        _ => return None,
    };
    let value = match (method.as_str(), args.get(1)) {
        ("toMap", Some(value)) => {
            boxed_name(mapped_element_type(std::slice::from_ref(value), ctx)?)
        }
        ("toMap", None) => return None,
        (_, Some(downstream)) => collector_value_type(downstream, &elem(), ctx)?,
        (_, None) => list_of(elem()),
    };
    Some((key, value))
}

/// The collector callbacks whose shape is fixed by the FACTORY rather than by
/// the element: a supplier that takes nothing, a predicate of the element, a
/// merge of two values, a finisher of what was gathered. `true` when this
/// argument was one of them and has been erased.
fn desugar_collector_shape(
    method: &str,
    args: &mut [Expr],
    index: usize,
    is_lambda: bool,
    elem: &TypeRef,
    ctx: &mut Ctx,
) -> bool {
    if !is_lambda {
        return false;
    }
    let object = TypeRef::Named(String::from("Object"));
    // `toCollection(ArrayList::new)` — a SUPPLIER, which takes no
    // element at all. Without this arm the reference was refused as "only
    // allowed where a functional-interface type is expected".
    if method == "toCollection" && index == 0 {
        if matches!(&args[index], Expr::MethodRef { .. }) {
            let synth = Sam {
                method: String::from("get"),
                params: Vec::new(),
                ret: object.clone(),
            };
            args[index] = method_ref_to_lambda(&args[index], &synth, ctx);
        }
        args[index] = build_erased_lambda(
            &mut args[index],
            "__Supplier",
            "get",
            &object,
            &[],
            None,
            ctx,
        );
        return true;
    }
    // `filtering(p, downstream)` — a predicate of the element, the same
    // shape `partitioningBy` takes in the same position.
    if matches!(method, "partitioningBy" | "filtering") && index == 0 {
        args[index] = build_erased_lambda(
            &mut args[index],
            "__Predicate",
            "test",
            &TypeRef::Boolean,
            std::slice::from_ref(elem),
            None,
            ctx,
        );
        return true;
    }
    // `toMap`'s third argument merges two VALUES, whose type this pass
    // cannot see, so both parameters erase to `Object`.
    if matches!(method, "toMap" | "toUnmodifiableMap") && index == 2 {
        args[index] = build_erased_lambda(
            &mut args[index],
            "__BiFunction",
            "apply",
            &object,
            &[object.clone(), object.clone()],
            None,
            ctx,
        );
        return true;
    }
    // `collectingAndThen(downstream, finisher)` — the finisher takes what
    // the collector below GATHERED, not an element, so its parameter
    // erases to `Object` rather than being typed from the stream.
    if method == "collectingAndThen" && index == 1 {
        // …and that is a COLLECTION, not an element: `List::size` has to
        // see a `List<String>` or the reference cannot be made at all.
        let gathered = collector_value_type(&args[0], elem, ctx).unwrap_or_else(|| object.clone());
        if matches!(&args[index], Expr::MethodRef { .. }) {
            let synth = Sam {
                method: String::from("apply"),
                params: vec![gathered.clone()],
                ret: object.clone(),
            };
            args[index] = method_ref_to_lambda(&args[index], &synth, ctx);
        }
        args[index] = build_erased_lambda(
            &mut args[index],
            "__UnaryOperator",
            "apply",
            &object,
            std::slice::from_ref(&gathered),
            None,
            ctx,
        );
        return true;
    }
    false
}

/// The `maxBy`/`minBy`/`reducing` arguments, which are the collector callbacks
/// that are NOT one function of one element: a comparator takes two, and a
/// reduction's operator folds two of whatever the mapper before it answered.
/// `true` when this argument was one and has been erased.
fn desugar_extreme_or_reducing(
    method: &str,
    args: &mut [Expr],
    index: usize,
    is_lambda: bool,
    elem: &TypeRef,
    ctx: &mut Ctx,
) -> bool {
    if !is_lambda {
        return false;
    }
    let object = TypeRef::Named(String::from("Object"));
    // `maxBy`/`minBy` take a COMPARATOR of the element, and `reducing`'s
    // operator two of them; neither is a function of one element.
    if matches!(method, "maxBy" | "minBy") && index == 0 {
        args[index] = build_erased_lambda(
            &mut args[index],
            "__Comparator",
            "compare",
            &TypeRef::Int,
            &[elem.clone(), elem.clone()],
            None,
            ctx,
        );
        return true;
    }
    // `reducing`'s LAST argument is always the binary operator; the
    // three-argument form puts a mapper of the element before it.
    if method == "reducing" && index + 1 == args.len() {
        // The three-argument form folds what the MAPPER answered, not the
        // element — and this pass can read that off the mapper.
        let folded = if args.len() == 3 {
            mapped_element_type(&args[1..2], ctx).unwrap_or_else(|| elem.clone())
        } else {
            elem.clone()
        };
        if matches!(&args[index], Expr::MethodRef { .. }) {
            let synth = Sam {
                method: String::from("apply"),
                params: vec![folded.clone(), folded.clone()],
                ret: object.clone(),
            };
            args[index] = method_ref_to_lambda(&args[index], &synth, ctx);
        }
        args[index] = build_erased_lambda(
            &mut args[index],
            "__BiFunction",
            "apply",
            &object,
            &[folded.clone(), folded],
            None,
            ctx,
        );
        return true;
    }
    if method == "reducing" && args.len() == 3 && index == 1 {
        if matches!(&args[index], Expr::MethodRef { .. }) {
            let synth = Sam {
                method: String::from("apply"),
                params: vec![elem.clone()],
                ret: object.clone(),
            };
            args[index] = method_ref_to_lambda(&args[index], &synth, ctx);
        }
        args[index] = build_erased_lambda(
            &mut args[index],
            "__UnaryOperator",
            "apply",
            &object,
            std::slice::from_ref(elem),
            None,
            ctx,
        );
        return true;
    }
    false
}

/// What a DOWNSTREAM collector produces, as a written type.
fn collector_value_type(collector: &Expr, element: &TypeRef, ctx: &Ctx) -> Option<TypeRef> {
    let Expr::Call { method, args, .. } = collector else {
        return None;
    };
    let elem = || element.clone();
    Some(match method.as_str() {
        "toList" | "toUnmodifiableList" => TypeRef::Generic {
            base: String::from("List"),
            args: vec![elem()],
        },
        "toSet" | "toUnmodifiableSet" => TypeRef::Generic {
            base: String::from("Set"),
            args: vec![elem()],
        },
        "joining" => TypeRef::Named(String::from("String")),
        "counting" => TypeRef::Named(String::from("Long")),
        // `mapping(f, downstream)` produces whatever the DOWNSTREAM does, over
        // the mapped element — not a list. Answering `List<mapped>` was right
        // for the usual `mapping(f, toList())` and wrong for every other
        // downstream, which showed the moment a `collectingAndThen` finisher
        // had to be typed from one.
        "mapping" if args.len() == 2 => {
            let mapped = boxed_name(mapped_element_type(std::slice::from_ref(&args[0]), ctx)?);
            collector_value_type(&args[1], &mapped, ctx)?
        }
        // `filtering` keeps the element; the collector under either of these
        // is what really decides, so ask it.
        "filtering" if args.len() == 2 => collector_value_type(&args[1], element, ctx)?,
        "collectingAndThen" if args.len() == 2 => {
            mapped_element_type(std::slice::from_ref(&args[1]), ctx)?
        }
        "maxBy" | "minBy" | "reducing" if args.len() == 1 => TypeRef::Generic {
            base: String::from("Optional"),
            args: vec![elem()],
        },
        "summarizingInt" => TypeRef::Named(String::from("IntSummaryStatistics")),
        "summarizingLong" => TypeRef::Named(String::from("LongSummaryStatistics")),
        "summarizingDouble" => TypeRef::Named(String::from("DoubleSummaryStatistics")),
        _ => return None,
    })
}

/// A primitive answer becomes its WRAPPER: a map holds references, so a
/// classifier that answers an `int` keys the map by `Integer`.
fn boxed_name(ty: TypeRef) -> TypeRef {
    // A primitive written as its own VARIANT boxes too. Only the `Named`
    // spelling was handled, so a `mapping(String::length, …)` whose element
    // this decides answered a `Set<int>` — a type argument that is not a
    // reference, which the position below it reported as "required:
    // reference, found: int" about a program that says neither.
    let name = match &ty {
        TypeRef::Int => "int",
        TypeRef::Long => "long",
        TypeRef::Double => "double",
        TypeRef::Float => "float",
        TypeRef::Short => "short",
        TypeRef::Byte => "byte",
        TypeRef::Char => "char",
        TypeRef::Boolean => "boolean",
        TypeRef::Named(name) => name.as_str(),
        _ => return ty,
    };
    let name = &String::from(name);
    TypeRef::Named(String::from(match name.as_str() {
        "int" => "Integer",
        "long" => "Long",
        "double" => "Double",
        "float" => "Float",
        "short" => "Short",
        "byte" => "Byte",
        "char" => "Character",
        "boolean" => "Boolean",
        _ => return ty,
    }))
}

fn map_type_args(receiver: &Expr, ctx: &Ctx) -> Option<(TypeRef, TypeRef)> {
    let witnessed = generic_call_return(receiver, ctx).or_else(|| witnessed_return(receiver, ctx));
    let ty = match witnessed {
        Some(ty) => ty,
        None => match receiver {
            Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0])?,
            // `this.vocab`
            Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
                ctx.lookup(name)?
            }
            // `new HashMap<String, Integer>().forEach(...)` — the arguments are
            // written right there.
            Expr::NewObject {
                class, type_args, ..
            } if type_args.len() == 2 => TypeRef::Generic {
                base: class.clone(),
                args: type_args.clone(),
            },
            // A MAP-building collector used straight as a receiver:
            // `stream.collect(groupingBy(f)).forEach((k, v) -> …)`. Through a
            // declared variable this worked; inline it had no key or value,
            // and the lambda was refused for having no functional-interface
            // position — the same tell every missing recogniser leaves.
            Expr::Call {
                receiver: Some(stream),
                method,
                args,
                ..
            } if method == "collect" && args.len() == 1 => {
                return collector_map_types(&args[0], stream, ctx);
            }
            // `Collections.unmodifiableMap(m).keySet()` — a read-only view keeps
            // the map's key and value types.
            Expr::Call { method, args, .. }
                if matches!(method.as_str(), "unmodifiableMap" | "unmodifiableSortedMap")
                    && args.len() == 1 =>
            {
                return map_type_args(&args[0], ctx);
            }
            // A RANGE or DESCENDING view of a sorted map holds the same keys
            // and values it does: `m.headMap(k).forEach((key, value) -> …)`.
            Expr::Call {
                receiver: Some(inner),
                method,
                ..
            } if matches!(
                method.as_str(),
                "headMap" | "tailMap" | "subMap" | "descendingMap"
            ) =>
            {
                return map_type_args(inner, ctx);
            }
            // `Map.of(k, v, …)` used STRAIGHT as a receiver: its key and value
            // are what the arguments look like, the same reading `List.of`
            // already gets. Without it a lambda over `Map.of(…).entrySet()
            // .stream()` had no element, though the identical chain over a
            // DECLARED map compiled.
            Expr::Call {
                receiver: Some(owner),
                method,
                args,
                ..
            } if method == "of"
                && args.len() >= 2
                && args.len().is_multiple_of(2)
                && names_library_class(owner, "Map") =>
            {
                let keys: Vec<Expr> = args.iter().step_by(2).cloned().collect();
                let values: Vec<Expr> = args.iter().skip(1).step_by(2).cloned().collect();
                return Some((
                    literal_element_type(&keys, ctx),
                    literal_element_type(&values, ctx),
                ));
            }
            // The same receiver shapes a LIST's element is read from: a cast, a
            // ternary, an array element, and a call to a method whose declared
            // return says what it gives back. A map returned by a method
            // (`config().forEach((k, v) -> …)`) had no key or value type, so the
            // lambda over it was refused for having no functional target — while
            // the identical call on a declared variable compiled.
            Expr::Cast { ty, .. } => ty.clone(),
            Expr::Ternary { then, els, .. } => {
                return map_type_args(then, ctx).or_else(|| map_type_args(els, ctx));
            }
            Expr::Index { array, .. } => match declared_array_type(array, ctx) {
                Some(TypeRef::Array(elem)) => *elem,
                _ => return None,
            },
            Expr::Call {
                receiver: owner,
                method,
                args,
                ..
            } if matches!(owner.as_deref(), None | Some(Expr::This { .. })) => {
                let class = ctx.current_class?;
                let shape = ctx
                    .shapes
                    .get(class)?
                    .iter()
                    .find(|shape| shape.name == *method && shape.arity == args.len())?;
                shape.return_type.clone()
            }
            // ...and the same call on ANOTHER object, or on the class itself:
            // `store.grouped().forEach((k, v) -> …)` and
            // `Store.of().map().values()`. Only the implicit-`this` form was
            // read, so a map handed back by another class had no key or value.
            Expr::Call {
                receiver: Some(owner),
                method,
                args,
                ..
            } => user_method_return(owner, method, args.len(), ctx)?,
            _ => return None,
        },
    };
    let TypeRef::Generic { base, args } = ty else {
        return None;
    };
    if !matches!(
        simple_base(base.as_str()),
        "Map" | "HashMap" | "TreeMap" | "SortedMap" | "NavigableMap" | "EnumMap"
    ) || args.len() != 2
    {
        return None;
    }
    Some((args[0].clone(), args[1].clone()))
}

/// The simple name of a library type written in full: `java.util.List` is
/// `List`. This pass matches container and functional-interface names by
/// spelling, and a QUALIFIED one missed every list — `l.sort((a, b) -> …)`
/// where `l` was declared `java.util.List<String>` found no comparator target.
/// Only `java.*` is stripped: a nested user type (`Outer.Inner`) keeps its
/// qualifier, since flattening that could match a library name by accident.
fn simple_base(base: &str) -> &str {
    match base.strip_prefix("java.") {
        Some(rest) => rest.rsplit('.').next().unwrap_or(base),
        None => base,
    }
}

/// The comparator element of a `TreeSet<E>` (`E`) or `TreeMap<K, V>` (`K`, the
/// key it orders) declaration target — for a diamond `new TreeSet<>(cmp)`
/// whose element type lives on the left-hand side.
fn sorted_ctor_elem(target: &TypeRef) -> Option<TypeRef> {
    let TypeRef::Generic { base, args } = target else {
        return None;
    };
    matches!(
        simple_base(base.as_str()),
        "TreeSet"
            | "SortedSet"
            | "NavigableSet"
            | "TreeMap"
            | "SortedMap"
            | "NavigableMap"
            | "PriorityQueue"
            // …and the INTERFACE faces a sorted collection is usually held by:
            // `Map<String, Integer> m = new TreeMap<>(cmp)` is how one is
            // written, and the target said nothing here, so the comparator
            // lambda had no parameter types and the whole declaration was
            // refused.
            | "Map"
            | "Set"
            | "Collection"
            | "Queue"
            | "Deque"
            | "List"
    )
    .then(|| args.first().cloned())
    .flatten()
}

/// How a lambda targeting a `java.util.function` interface desugars: which erased
/// bundled interface and SAM it implements, and — read from the target's type
/// arguments — the parameter types to cast to and the result type to coerce to.
pub(crate) struct FunctionalSpec {
    interface: &'static str,
    method: &'static str,
    /// The synthesized method's return type: the erased SAM's own return
    /// (`Object`, `boolean`, or `void`).
    ret: TypeRef,
    /// The parameter types the erased `Object` arguments cast back to.
    params: Vec<TypeRef>,
    /// The declared result type the body must coerce to, when the SAM returns a
    /// value that erases to `Object` (`apply`/`get`). `None` for `test`/`accept`,
    /// whose return is concrete.
    result: Option<TypeRef>,
}

/// Whether the user defined a class/interface of the target's name — in which
/// case theirs wins, and the `java.util.function` aliasing must stand aside (a
/// user `interface Function<A, B>` is common). The bundled interfaces are all
/// `__`-prefixed, so a bare `Function` in the name set is the user's.
fn user_defined_functional(target: &TypeRef, ctx: &Ctx) -> bool {
    let name = match target {
        TypeRef::Named(name) | TypeRef::Generic { base: name, .. } => name.as_str(),
        _ => return false,
    };
    let simple = name.rsplit('.').next().unwrap_or(name);
    ctx.class_names.contains(simple)
}

/// The functional interfaces named WITHOUT type arguments: `Runnable`, and the
/// primitive specializations, whose parameter and result types their name
/// fixes. Each names its own erased interface — see `stdlib/function.java` for
/// why they are not the shared SAM of matching shape.
#[allow(clippy::too_many_lines)] // one flat table, one line per interface
fn unparameterized_spec(simple: &str) -> Option<FunctionalSpec> {
    let (interface, method, ret, params, result): (_, _, _, Vec<TypeRef>, Option<TypeRef>) =
        match simple {
            "Runnable" => ("__Runnable", "run", TypeRef::Void, vec![], None),
            "IntUnaryOperator" => (
                "__IntUnaryOperator",
                "applyAsInt",
                TypeRef::Int,
                vec![TypeRef::Int],
                None,
            ),
            "IntBinaryOperator" => (
                "__IntBinaryOperator",
                "applyAsInt",
                TypeRef::Int,
                vec![TypeRef::Int, TypeRef::Int],
                None,
            ),
            "IntPredicate" => (
                "__IntPredicate",
                "test",
                TypeRef::Boolean,
                vec![TypeRef::Int],
                None,
            ),
            "IntSupplier" => ("__IntSupplier", "getAsInt", TypeRef::Int, vec![], None),
            // The primitive-to-primitive conversions: both ends fixed by the
            // name, so neither is written and neither takes a type argument.
            "IntToLongFunction" => (
                "__IntToLongFunction",
                "applyAsLong",
                TypeRef::Long,
                vec![TypeRef::Int],
                None,
            ),
            "IntToDoubleFunction" => (
                "__IntToDoubleFunction",
                "applyAsDouble",
                TypeRef::Double,
                vec![TypeRef::Int],
                None,
            ),
            "LongToIntFunction" => (
                "__LongToIntFunction",
                "applyAsInt",
                TypeRef::Int,
                vec![TypeRef::Long],
                None,
            ),
            "LongToDoubleFunction" => (
                "__LongToDoubleFunction",
                "applyAsDouble",
                TypeRef::Double,
                vec![TypeRef::Long],
                None,
            ),
            "DoubleToIntFunction" => (
                "__DoubleToIntFunction",
                "applyAsInt",
                TypeRef::Int,
                vec![TypeRef::Double],
                None,
            ),
            "DoubleToLongFunction" => (
                "__DoubleToLongFunction",
                "applyAsLong",
                TypeRef::Long,
                vec![TypeRef::Double],
                None,
            ),
            "IntConsumer" => (
                "__IntConsumer",
                "accept",
                TypeRef::Void,
                vec![TypeRef::Int],
                None,
            ),
            "DoubleUnaryOperator" => (
                "__DoubleUnaryOperator",
                "applyAsDouble",
                TypeRef::Double,
                vec![TypeRef::Double],
                None,
            ),
            "DoubleBinaryOperator" => (
                "__DoubleBinaryOperator",
                "applyAsDouble",
                TypeRef::Double,
                vec![TypeRef::Double, TypeRef::Double],
                None,
            ),
            "DoublePredicate" => (
                "__DoublePredicate",
                "test",
                TypeRef::Boolean,
                vec![TypeRef::Double],
                None,
            ),
            "DoubleSupplier" => (
                "__DoubleSupplier",
                "getAsDouble",
                TypeRef::Double,
                vec![],
                None,
            ),
            "DoubleConsumer" => (
                "__DoubleConsumer",
                "accept",
                TypeRef::Void,
                vec![TypeRef::Double],
                None,
            ),
            "LongUnaryOperator" => (
                "__LongUnaryOperator",
                "applyAsLong",
                TypeRef::Long,
                vec![TypeRef::Long],
                None,
            ),
            "LongBinaryOperator" => (
                "__LongBinaryOperator",
                "applyAsLong",
                TypeRef::Long,
                vec![TypeRef::Long, TypeRef::Long],
                None,
            ),
            "LongPredicate" => (
                "__LongPredicate",
                "test",
                TypeRef::Boolean,
                vec![TypeRef::Long],
                None,
            ),
            "LongSupplier" => ("__LongSupplier", "getAsLong", TypeRef::Long, vec![], None),
            "LongConsumer" => (
                "__LongConsumer",
                "accept",
                TypeRef::Void,
                vec![TypeRef::Long],
                None,
            ),
            "BooleanSupplier" => (
                "__BooleanSupplier",
                "getAsBoolean",
                TypeRef::Boolean,
                vec![],
                None,
            ),
            _ => return None,
        };
    Some(FunctionalSpec {
        interface,
        method,
        ret,
        params,
        result,
    })
}

#[allow(clippy::too_many_lines)] // one flat table, one line per interface
pub(crate) fn functional_lambda_spec(target: &TypeRef) -> Option<FunctionalSpec> {
    // `Runnable` takes no type arguments, so it arrives as a plain NAMED type
    // rather than a parameterized one — which is why a `Runnable r = () -> …`
    // found no functional target at all.
    // The specializations that take NO type arguments — `Runnable`, and every
    // primitive one, whose parameter and result types are fixed by the name.
    if let TypeRef::Named(name) = target {
        return unparameterized_spec(name.rsplit('.').next().unwrap_or(name));
    }
    let TypeRef::Generic { base, args } = target else {
        return None;
    };
    let object = || TypeRef::Named(String::from("Object"));
    let simple = base.rsplit('.').next().unwrap_or(base);
    let (interface, method, ret, params, result): (_, _, _, Vec<TypeRef>, Option<TypeRef>) =
        match (simple, args.as_slice()) {
            ("Function", [a, b]) => (
                "__UnaryOperator",
                "apply",
                object(),
                vec![a.clone()],
                Some(b.clone()),
            ),
            ("UnaryOperator", [a]) => (
                "__UnaryOperator",
                "apply",
                object(),
                vec![a.clone()],
                Some(a.clone()),
            ),
            ("Predicate", [a]) => (
                "__Predicate",
                "test",
                TypeRef::Boolean,
                vec![a.clone()],
                None,
            ),
            ("Consumer", [a]) => ("__Consumer", "accept", TypeRef::Void, vec![a.clone()], None),
            // `Comparator<E>` is a functional interface too, so a METHOD
            // REFERENCE stands where its `compare(E, E)` is expected —
            // `Comparator<Integer> c = Cls::byDescending;` was refused as if
            // the position were not one.
            ("Comparator", [a]) => (
                "__Comparator",
                "compare",
                TypeRef::Int,
                vec![a.clone(), a.clone()],
                None,
            ),
            ("Supplier", [a]) => ("__Supplier", "get", object(), Vec::new(), Some(a.clone())),
            ("BiFunction", [a, b, c]) => (
                "__BiFunction",
                "apply",
                object(),
                vec![a.clone(), b.clone()],
                Some(c.clone()),
            ),
            ("BinaryOperator", [a]) => (
                "__BiFunction",
                "apply",
                object(),
                vec![a.clone(), a.clone()],
                Some(a.clone()),
            ),
            ("BiConsumer", [a, b]) => (
                "__BiConsumer",
                "accept",
                TypeRef::Void,
                vec![a.clone(), b.clone()],
                None,
            ),
            // The specializations that DO take a type argument: one end of the
            // function is a primitive fixed by the name, the other is written.
            ("IntFunction", [r]) => (
                "__IntFunction",
                "apply",
                object(),
                vec![TypeRef::Int],
                Some(r.clone()),
            ),
            ("ToIntFunction", [t]) => (
                "__ToIntFunction",
                "applyAsInt",
                TypeRef::Int,
                vec![t.clone()],
                None,
            ),
            ("ToDoubleFunction", [t]) => (
                "__ToDoubleFunction",
                "applyAsDouble",
                TypeRef::Double,
                vec![t.clone()],
                None,
            ),
            ("ToLongFunction", [t]) => (
                "__ToLongFunction",
                "applyAsLong",
                TypeRef::Long,
                vec![t.clone()],
                None,
            ),
            ("DoubleFunction", [r]) => (
                "__DoubleFunction",
                "apply",
                object(),
                vec![TypeRef::Double],
                Some(r.clone()),
            ),
            ("LongFunction", [r]) => (
                "__LongFunction",
                "apply",
                object(),
                vec![TypeRef::Long],
                Some(r.clone()),
            ),
            ("ObjIntConsumer", [t]) => (
                "__ObjIntConsumer",
                "accept",
                TypeRef::Void,
                vec![t.clone(), TypeRef::Int],
                None,
            ),
            ("ObjLongConsumer", [t]) => (
                "__ObjLongConsumer",
                "accept",
                TypeRef::Void,
                vec![t.clone(), TypeRef::Long],
                None,
            ),
            ("ObjDoubleConsumer", [t]) => (
                "__ObjDoubleConsumer",
                "accept",
                TypeRef::Void,
                vec![t.clone(), TypeRef::Double],
                None,
            ),
            ("ToIntBiFunction", [t, u]) => (
                "__ToIntBiFunction",
                "applyAsInt",
                TypeRef::Int,
                vec![t.clone(), u.clone()],
                None,
            ),
            ("ToLongBiFunction", [t, u]) => (
                "__ToLongBiFunction",
                "applyAsLong",
                TypeRef::Long,
                vec![t.clone(), u.clone()],
                None,
            ),
            ("ToDoubleBiFunction", [t, u]) => (
                "__ToDoubleBiFunction",
                "applyAsDouble",
                TypeRef::Double,
                vec![t.clone(), u.clone()],
                None,
            ),
            ("BiPredicate", [t, u]) => (
                "__BiPredicate",
                "test",
                TypeRef::Boolean,
                vec![t.clone(), u.clone()],
                None,
            ),
            _ => return None,
        };
    Some(FunctionalSpec {
        interface,
        method,
        ret,
        params,
        result,
    })
}

/// The element type `E` of a `Comparator<E>` target type, for casting a
/// comparator lambda's two parameters. `null` for anything else.
/// The KEY or VALUE type of a written `Map.Entry<K, V>`.
fn entry_side_type(entry: &TypeRef, value: bool) -> Option<TypeRef> {
    let TypeRef::Generic { base, args } = entry else {
        return None;
    };
    (matches!(
        simple_base(base),
        "Map.Entry" | "Entry" | "java.util.Map.Entry"
    ) && args.len() == 2)
        .then(|| args[usize::from(value)].clone())
}

fn comparator_target_elem(target: &TypeRef) -> Option<TypeRef> {
    let TypeRef::Generic { base, args } = target else {
        return None;
    };
    (matches!(base.as_str(), "Comparator" | "java.util.Comparator") && args.len() == 1)
        .then(|| args[0].clone())
}

/// The element type for a comparator key extractor: the lambda's OWN declared
/// parameter wins (`(P p) -> p.a`), otherwise the element of the `Comparator<E>`
/// position the call sits in (a declaration, or the list being sorted).
///
/// A generic factory has no other way to see it — the erased SAM's parameter is
/// `Object`, so `Comparator.comparingInt(p -> p.a)` reported "cannot find
/// symbol: field 'a' in class Object" and only METHOD REFERENCES worked.
fn key_extractor_elem(arg: &Expr, expected: Option<&TypeRef>) -> Option<TypeRef> {
    if let Expr::Lambda { params, .. } = arg
        && let [param] = params.as_slice()
        && let Some(ty) = &param.ty
    {
        return Some(ty.clone());
    }
    expected.and_then(comparator_target_elem)
}

/// Desugar a `Comparator.comparing*(f)` factory or a `cmp.thenComparing*(f)`
/// combinator. Returns whether it took the call.
///
/// The chain shares ONE element type: `Comparator.comparingInt(P::getA)
/// .thenComparing(p -> p.n)` compares two `P`s throughout. So the target type
/// travels down the receiver as well as into the argument — which is why this
/// runs before the generic receiver walk, which passes `None`.
#[allow(clippy::too_many_lines)] // one arm per comparator factory/combinator
fn desugar_comparator_chain(
    receiver: &mut Option<Box<Expr>>,
    method: &str,
    args: &mut [Expr],
    expected: Option<&TypeRef>,
    ctx: &mut Ctx,
) -> bool {
    let is_factory = matches!(
        method,
        "comparing" | "comparingInt" | "comparingDouble" | "comparingLong"
    ) && matches!(receiver.as_deref(), Some(Expr::Name { path, .. })
            if path.last().is_some_and(|last| last == "Comparator")
                && (path.len() == 1 || path[0] == "java"));
    let is_combinator = matches!(
        method,
        "thenComparing" | "thenComparingInt" | "thenComparingLong" | "thenComparingDouble"
    );
    // `Comparator.comparing(keyExtractor, keyComparator)`: the extractor is
    // typed exactly as in the one-argument form; the second argument compares
    // the KEYS, whose type nothing here knows, so it desugars on its own.
    // `cmp.thenComparing(keyExtractor, keyComparator)` — the two-argument
    // combinator, typed exactly like the two-argument factory: the extractor
    // takes the element, and the second argument orders the KEYS.
    if is_combinator && method == "thenComparing" && args.len() == 2 {
        if let Some(r) = receiver.as_deref_mut() {
            desugar_expr(r, expected, ctx);
        }
        let (extractor, rest) = args.split_at_mut(1);
        if !desugar_key_extractor(&mut extractor[0], ctx) {
            let single = matches!(&extractor[0], Expr::Lambda { params, .. } if params.len() == 1);
            if let (true, Some(elem)) = (single, key_extractor_elem(&extractor[0], expected)) {
                let object = TypeRef::Named(String::from("Object"));
                extractor[0] = build_erased_lambda(
                    &mut extractor[0],
                    "__UnaryOperator",
                    "apply",
                    &object,
                    &[elem],
                    None,
                    ctx,
                );
            } else {
                let unary = TypeRef::Named(String::from("__UnaryOperator"));
                desugar_expr(&mut extractor[0], Some(&unary), ctx);
            }
        }
        desugar_expr(&mut rest[0], None, ctx);
        return true;
    }
    if is_factory && method == "comparing" && args.len() == 2 {
        let (extractor, rest) = args.split_at_mut(1);
        if !desugar_key_extractor(&mut extractor[0], ctx) {
            let single = matches!(&extractor[0], Expr::Lambda { params, .. } if params.len() == 1);
            if let (true, Some(elem)) = (single, key_extractor_elem(&extractor[0], expected)) {
                let object = TypeRef::Named(String::from("Object"));
                extractor[0] = build_erased_lambda(
                    &mut extractor[0],
                    "__UnaryOperator",
                    "apply",
                    &object,
                    &[elem],
                    None,
                    ctx,
                );
            } else {
                let unary = TypeRef::Named(String::from("__UnaryOperator"));
                desugar_expr(&mut extractor[0], Some(&unary), ctx);
            }
        }
        desugar_expr(&mut rest[0], None, ctx);
        return true;
    }
    if args.len() != 1 || !(is_factory || is_combinator) {
        return false;
    }
    if is_combinator {
        // The receiver is a comparator over the same element.
        let Some(r) = receiver.as_deref_mut() else {
            return false;
        };
        desugar_expr(r, expected, ctx);
    }
    // A method reference self-types from its qualifier.
    if desugar_key_extractor(&mut args[0], ctx) {
        return true;
    }
    let params = match &args[0] {
        Expr::Lambda { params, .. } => params.len(),
        _ => usize::MAX,
    };
    // `thenComparing(anotherComparator)` — including a two-parameter lambda,
    // whose parameters are both the element.
    if method == "thenComparing" && params != 1 {
        let target = expected.and_then(comparator_target_elem).map(|elem| {
            (
                TypeRef::Generic {
                    base: String::from("Comparator"),
                    args: vec![elem.clone()],
                },
                elem,
            )
        });
        match (params, target) {
            (2, Some((_, elem))) => {
                args[0] = build_erased_lambda(
                    &mut args[0],
                    "__Comparator",
                    "compare",
                    &TypeRef::Int,
                    &[elem.clone(), elem],
                    None,
                    ctx,
                );
            }
            (_, target) => desugar_expr(&mut args[0], target.as_ref().map(|(t, _)| t), ctx),
        }
        return true;
    }
    // A one-parameter key extractor.
    if params == 1
        && let Some(elem) = key_extractor_elem(&args[0], expected)
    {
        let object = TypeRef::Named(String::from("Object"));
        args[0] = build_erased_lambda(
            &mut args[0],
            "__UnaryOperator",
            "apply",
            &object,
            &[elem],
            None,
            ctx,
        );
        return true;
    }
    // A method REFERENCE whose qualifier is not a plain class name — the one
    // that matters is `Map.Entry::getKey`, written on the list being sorted.
    // `desugar_key_extractor` types such a reference from its QUALIFIER, and a
    // nested library type is not a name it can read; the element the
    // surrounding `Comparator<E>` position gives is the same answer, and the
    // sort site hands it down. Without this the reference resolved `getKey()`
    // against `Object` — a refusal for one of the two spellings a program
    // uses to sort a map's entries.
    if matches!(&args[0], Expr::MethodRef { .. })
        && let Some(elem) = expected.and_then(comparator_target_elem)
        && let Some(sam) = ctx.sams.get("__UnaryOperator").cloned()
    {
        args[0] = method_ref_to_lambda(&args[0], &sam, ctx);
        args[0] = build_erased_lambda(
            &mut args[0],
            "__UnaryOperator",
            "apply",
            &TypeRef::Named(String::from("Object")),
            &[elem],
            None,
            ctx,
        );
        return true;
    }
    // Nothing pinned the element: fall back to the erased `Object` parameter,
    // which is right for a key extractor that never touches the element's own
    // members (`Comparator.comparing(s -> s)`).
    let unary = TypeRef::Named(String::from("__UnaryOperator"));
    desugar_expr(&mut args[0], Some(&unary), ctx);
    true
}

/// Desugar a `Comparator.comparing*`/`thenComparing` **method-reference** key
/// extractor (`Person::getAge`) into an erased `__UnaryOperator` whose parameter
/// is cast to the qualifier class. Returns whether it handled `arg` (false for a
/// lambda or a non-extractor, which the caller types some other way).
fn desugar_key_extractor(arg: &mut Expr, ctx: &mut Ctx) -> bool {
    let Expr::MethodRef {
        qualifier, method, ..
    } = &*arg
    else {
        return false;
    };
    let Expr::Name { path, .. } = qualifier.as_ref() else {
        return false;
    };
    if path.len() != 1 {
        return false;
    }
    // The key extractor's parameter is the ELEMENT being sorted.
    //
    // For an unbound instance reference (`Runner::getName`) that element IS the
    // qualifier class. For a STATIC one (`G::key`) it is not — the element is
    // whatever the static method takes, and typing it as the qualifier produced
    // `no suitable method found for key(G)`: caturra had read `G::key` as
    // "call key() ON a G".
    let is_static = ctx
        .static_methods
        .get(&path[0])
        .is_some_and(|names| names.contains(method));
    let elem = if is_static {
        let mut one_param = ctx
            .methods
            .get(method)
            .into_iter()
            .flatten()
            .filter(|params| params.len() == 1);
        let first = one_param.next().cloned();
        // Overloads that disagree cannot pin the element type; leave it alone.
        match first {
            Some(param) if one_param.all(|other| other[..] == param[..]) => param[0].clone(),
            _ => return false,
        }
    } else {
        TypeRef::Named(path[0].clone())
    };
    let Some(sam) = ctx.sams.get("__UnaryOperator").cloned() else {
        return false;
    };
    *arg = method_ref_to_lambda(arg, &sam, ctx);
    *arg = build_erased_lambda(
        arg,
        "__UnaryOperator",
        "apply",
        &TypeRef::Named(String::from("Object")),
        &[elem],
        None,
        ctx,
    );
    true
}
/// The synthetic static field recording what a lambda class answers, so the
/// element of a mapped stream survives erasure into codegen.
pub(crate) const PRODUCES_FIELD: &str = "__caturraProduces";

/// The synthesized local a lambda's body is assigned to when its target
/// declares a RESULT type. It is how that type gets checked at all, and
/// codegen reads the name to word an error about it as javac does — a bad
/// return type in a lambda expression, rather than a mismatch in code the
/// program never wrote.
pub(crate) const RESULT_LOCAL: &str = "__caturraResult";

/// What a `map`'s lambda ANSWERS, so the mapped stream keeps an element type.
///
/// `map` erased its result to `Object`, which is faithful to erasure and
/// useless downstream: `map(s -> s.length())` could not be filtered on, added
/// to, collected into a `List<Integer>` or assigned to an `int`, though javac
/// types every one of them. Only the shapes whose type is written down
/// somewhere are read; anything else stays `Object`, which is what it was.
fn mapped_element_type(args: &[Expr], ctx: &Ctx) -> Option<TypeRef> {
    // By the time an outer call asks, the lambda is already a synthesized
    // CLASS — the pass desugars a receiver before it types what the receiver
    // yields. The class says everything needed: its body opens by unwrapping
    // each erased argument into the parameter's declared type, and ends in the
    // expression whose type is the answer.
    let [only] = args else {
        return None;
    };
    if let Expr::NewObject { class, .. } = only
        && let Some(decl) = ctx.new_classes.iter().find(|decl| decl.name == *class)
    {
        return produced_type(decl, ctx);
    }
    // …or a `Function` the program factored out — a variable, a field, a
    // method's return. There is no synthesized class to read, and its DECLARED
    // type says the same thing: a `Function<String, Integer>` produces an
    // `Integer`. Without this, `stream.map(f)` — the first thing anyone does
    // after pulling a lambda out into a name — left the element unknown, so
    // the NEXT operation's lambda had an untyped parameter.
    declared_functional_result(&body_type(only, &HashMap::new(), ctx)?)
}

/// The RESULT type argument of a functional interface as written:
/// `Function<String, Integer>` answers `Integer`, `Supplier<Pet>` a `Pet`.
/// `None` for an interface whose result is not a type argument at all — a
/// `Predicate` answers `boolean` and a `Consumer` nothing, so the variable in
/// their last position is a PARAMETER and says nothing about a result.
fn declared_functional_result(ty: &TypeRef) -> Option<TypeRef> {
    let TypeRef::Generic { base, args } = ty else {
        return None;
    };
    let simple = base.rsplit('.').next().unwrap_or(base);
    let simple = simple.strip_prefix("__").unwrap_or(simple);
    let arity = crate::ast::functional_result_arity(simple)?;
    if args.len() != arity {
        return None;
    }
    args.last().cloned()
}

/// What a synthesized lambda class ANSWERS, read off the class itself: its
/// body opens by unwrapping each erased argument into the parameter's declared
/// type, and ends in the expression whose type is the answer.
fn produced_type(decl: &ClassDecl, ctx: &Ctx) -> Option<TypeRef> {
    let (answer, bound) = lambda_answer(decl)?;
    body_type(answer, &bound, ctx)
}

/// The expression a synthesized lambda class ANSWERS, with the declared type
/// of each parameter it unwrapped on the way in. `produced_type` reads the
/// type of that expression; `flat_element_type` reads the ELEMENT of the
/// stream it denotes — the same answer, asked two different questions.
fn lambda_answer(decl: &ClassDecl) -> Option<(&Expr, HashMap<String, TypeRef>)> {
    let body = decl.methods.iter().find(|method| !method.is_constructor)?;
    let mut bound: HashMap<String, TypeRef> = HashMap::new();
    let mut answer = None;
    // The body's value may sit in the synthesized `__caturraResult` local,
    // which is DECLARED as the target's result type — `Object` when the target
    // is a functional interface parameterized on a type variable. Reading the
    // name gives that erasure and nothing else, so the initializer is the
    // answer: `conv("abc", s -> s.length())` produces an `int`, not an Object.
    let mut result_init = None;
    for stmt in &body.body {
        match stmt {
            Stmt::LocalDecl {
                ty, declarators, ..
            } => {
                for declarator in declarators {
                    let unwraps = matches!(
                        &declarator.init,
                        Some(Expr::Cast { operand, .. })
                            if matches!(
                                operand.as_ref(),
                                Expr::Name { path, .. }
                                    if path.len() == 1 && path[0].starts_with("__caturraArg")
                            )
                    );
                    if unwraps {
                        bound.insert(declarator.name.clone(), ty.clone());
                    } else if declarator.name == "__caturraResult" {
                        result_init = declarator.init.as_ref();
                    }
                }
            }
            Stmt::Return {
                value: Some(value), ..
            } => answer = Some(value),
            _ => {}
        }
    }
    let answer = match (answer?, result_init) {
        (Expr::Name { path, .. }, Some(init)) if path == &[String::from("__caturraResult")] => init,
        (other, _) => other,
    };
    Some((answer, bound))
}

/// The element of the stream a `flatMap` lambda answers. Its parameter is not
/// in `ctx` — it exists only inside the synthesized class — so the ordinary
/// receiver walk cannot type `s -> Stream.of(s, s.substring(0, 1))`; the two
/// shapes that mention the parameter are read here against `bound`, and
/// everything else (a field, a local, `Files.lines(p)`) is left to the walk,
/// which already knows those.
fn flat_element_type(args: &[Expr], ctx: &Ctx) -> Option<TypeRef> {
    let [Expr::NewObject { class, .. }] = args else {
        return None;
    };
    let decl = ctx.new_classes.iter().find(|decl| decl.name == *class)?;
    let (answer, bound) = lambda_answer(decl)?;
    let Expr::Call {
        receiver: Some(prev),
        method,
        args,
        ..
    } = answer
    else {
        return stream_elem_type(answer, ctx);
    };
    // `Stream.of(a, b)` over the parameter: the element joins the arguments,
    // exactly as a literal stream's does, but typed against `bound`.
    if method == "of" && names_library_class(prev.as_ref(), "Stream") && !args.is_empty() {
        let mut kind: Option<TypeRef> = None;
        for arg in args {
            let this = boxed_element(body_type(arg, &bound, ctx)?);
            kind = Some(match &kind {
                Some(seen) => join_element_types(seen, &this, ctx.supers),
                None => this,
            });
        }
        return kind;
    }
    // `x.stream()` / `Arrays.stream(x)` over a parameter whose declared type
    // carries the element. `parallelStream()` is the same source under
    // another name, and every one of these readings has to know that or the
    // lambda after it has no element and is refused for having no target.
    if matches!(method.as_str(), "stream" | "parallelStream") {
        let source = if args.is_empty() {
            body_type(prev, &bound, ctx)?
        } else if names_library_class(prev.as_ref(), "Arrays") && args.len() == 1 {
            body_type(&args[0], &bound, ctx)?
        } else {
            return stream_elem_type(answer, ctx);
        };
        return match source {
            TypeRef::Array(elem) => Some(boxed_element(*elem)),
            TypeRef::Generic { args, .. } if args.len() == 1 => Some(args[0].clone()),
            _ => None,
        };
    }
    stream_elem_type(answer, ctx)
}

/// [`body_type`] for a CALL — the shape a lambda body most often ends in, and
/// the only one that has to ask every table: the program's own methods, the
/// library's, a static factory that answers a stream.
fn call_body_type(
    expr: &Expr,
    receiver: Option<&Expr>,
    method: &str,
    args: &[Expr],
    bound: &HashMap<String, TypeRef>,
    ctx: &Ctx,
) -> Option<TypeRef> {
    // A method of the program: its declared return, with a generic
    // one pinned the way any other call to it would be.
    if let Some(ty) = generic_call_return(expr, ctx) {
        return Some(ty);
    }
    if matches!(receiver, None | Some(Expr::This { .. }))
        && let Some(class) = ctx.current_class
        && let Some(shape) = ctx
            .shapes
            .get(class)?
            .iter()
            .find(|shape| shape.name == method && shape.takes(args.len()))
    {
        return Some(shape.return_type.clone());
    }
    let receiver = receiver?;
    // A library STATIC, whose receiver is a class name rather than a
    // value: `String.valueOf(c)` is a `String`, and reading its type
    // through the receiver (as an instance call would) answered
    // nothing — so `mapToObj(c -> String.valueOf(c))` produced an
    // `Object` element and the `String::concat` after it could not
    // resolve.
    // The `Collections` wrappers pass their ARGUMENT's type through, which a
    // table keyed by (class, method) cannot say — it never sees the argument.
    // `collectingAndThen(toList(), Collections::unmodifiableList)` is the
    // ordinary way to freeze a gathered list, and with no type for the body
    // the whole `collect` typed as nothing: `.size()` on it was "<null> cannot
    // be dereferenced", while the same collector through a VARIABLE worked.
    if let Expr::Name { path, .. } = receiver
        && path.last().is_some_and(|name| name == "Collections")
        && let [only] = args
    {
        match method {
            "unmodifiableList"
            | "unmodifiableSet"
            | "unmodifiableMap"
            | "unmodifiableCollection"
            | "unmodifiableSortedSet"
            | "unmodifiableSortedMap"
            | "unmodifiableNavigableSet"
            | "unmodifiableNavigableMap" => return body_type(only, bound, ctx),
            "singletonList" | "singleton" => {
                let elem = body_type(only, bound, ctx)?;
                return Some(TypeRef::Generic {
                    base: String::from(if method == "singleton" { "Set" } else { "List" }),
                    args: vec![boxed_name(elem)],
                });
            }
            _ => {}
        }
    }
    if let Expr::Name { path, .. } = receiver
        && let Some(ty) = library_static_type(path.last()?, method, args.len())
    {
        return Some(ty);
    }
    // An ENUM's own statics, which have no receiver VALUE to type:
    // `Kind.values()` is a `Kind[]` and `Kind.valueOf(s)` a `Kind`.
    if let Some(name) = enum_owner_name(receiver, ctx) {
        return match (method, args.len()) {
            ("values", 0) => Some(TypeRef::Array(Box::new(TypeRef::Named(name)))),
            ("valueOf", 1) => Some(TypeRef::Named(name)),
            _ => None,
        };
    }
    // A lambda that ANSWERS a stream built from a static factory:
    // `flatMap(x -> Stream.of(x.first()))` and
    // `flatMap(row -> Arrays.stream(row))`. Both are typed from an
    // ARGUMENT, which no receiver-keyed table can see.
    let stream_of = |elem: TypeRef| TypeRef::Generic {
        base: String::from("Stream"),
        args: vec![boxed_element(elem)],
    };
    if method == "of"
        && names_library_class(receiver, "Stream")
        && let Some(first) = args.first()
        && let Some(elem) = body_type(first, bound, ctx)
    {
        return Some(stream_of(elem));
    }
    if method == "stream"
        && names_library_class(receiver, "Arrays")
        && let [only] = args
        && let Some(TypeRef::Array(elem)) = body_type(only, bound, ctx)
    {
        return Some(stream_of(*elem));
    }
    // A receiver that is a CLASS rather than a value — `Store.of()`, the static
    // factory. `body_type` types VALUES, so the chain stopped at the class name
    // and every reading after it was lost.
    let Some(on) = body_type(receiver, bound, ctx) else {
        return user_method_return(receiver, method, args.len(), ctx);
    };
    if let Some(ty) = library_return(&on, method, args.len()) {
        return Some(ty);
    }
    // A method of a USER class, on a receiver whose type is known —
    // the lambda's own parameter, usually. `pets.stream().map(p ->
    // p.name())` is as ordinary as a stream gets, and the mapped
    // element was `Object`: the shapes were consulted for an implicit
    // `this` receiver and for library types, and for nothing else.
    // Asked AFTER the library table, since caturra's own bundled Java
    // declares classes of those names and its erased `Optional.get()`
    // answers an `Object` the real one does not.
    let (TypeRef::Named(name) | TypeRef::Generic { base: name, .. }) = &on else {
        return None;
    };
    let answered = ctx
        .shapes
        .get(name)?
        .iter()
        .find(|shape| shape.name == method && shape.takes(args.len()))
        .map(|shape| shape.return_type.clone())?;
    // A method that answers its class's own TYPE VARIABLE — `Box<T>`'s
    // `get()` — answers the RECEIVER's argument. Erasure has already
    // replaced the variable with its positional sentinel, which is
    // exactly the index to read: without this
    // `boxes.stream().map(Box::get)` mapped to `Object`, though
    // `boxes.get(0).get()` beside it did not.
    if let TypeRef::Named(sentinel) = &answered
        && let Some(index) = crate::parser::typevar_index(sentinel)
        && let TypeRef::Generic { args: written, .. } = &on
        && let Some(argument) = written.get(usize::from(index))
    {
        return Some(argument.clone());
    }
    Some(answered)
}

/// The reference form of a primitive. An OBJECT stream's element is always a
/// reference, so a supplier answering `2` makes a `Stream<Integer>` — reading
/// the element as a bare `int` typed the fold's parameters as primitives and
/// its result went back unboxed, which is a `VerifyError`, not a diagnostic.
fn boxed_element(ty: TypeRef) -> TypeRef {
    let name = match ty {
        TypeRef::Int => "Integer",
        TypeRef::Long => "Long",
        TypeRef::Double => "Double",
        TypeRef::Float => "Float",
        TypeRef::Short => "Short",
        TypeRef::Byte => "Byte",
        TypeRef::Char => "Character",
        TypeRef::Boolean => "Boolean",
        other => return other,
    };
    TypeRef::Named(String::from(name))
}

/// The type of a lambda BODY, given what its parameter is. Deliberately a
/// subset: a name, a literal, an operator, a call to a method of the program,
/// and the handful of library methods whose return is a scalar or a String.
/// Anything unrecognized answers `None`, which leaves the caller exactly where
/// it was before this existed.
fn body_type(expr: &Expr, bound: &HashMap<String, TypeRef>, ctx: &Ctx) -> Option<TypeRef> {
    use crate::ast::BinaryOp as B;
    match expr {
        Expr::Name { path, .. } if path.len() == 1 => bound
            .get(&path[0])
            .cloned()
            .or_else(|| static_type_of(expr, ctx)),
        // A field read through the lambda's OWN parameter — `d.cards` inside
        // `decks.stream().map(d -> d.cards.get(0))`. The general reader knows
        // the pass's scope; only `bound` knows the parameter. (The parser
        // keeps a dotted read as a NAME, so both spellings arrive here.)
        Expr::Name { path, span } if path.len() == 2 => {
            let object = Expr::Name {
                path: vec![path[0].clone()],
                span: *span,
            };
            match body_type(&object, bound, ctx)? {
                TypeRef::Named(class) | TypeRef::Generic { base: class, .. } => {
                    field_of_class(&class, &path[1], ctx)
                }
                _ => None,
            }
        }
        Expr::Field { object, name, .. } if !matches!(**object, Expr::This { .. }) => {
            match body_type(object, bound, ctx)? {
                TypeRef::Named(class) | TypeRef::Generic { base: class, .. } => {
                    field_of_class(&class, name, ctx)
                }
                _ => None,
            }
        }
        Expr::Cast { ty, .. } => Some(ty.clone()),
        Expr::Ternary { then, els, .. } => {
            let then = body_type(then, bound, ctx)?;
            (then == body_type(els, bound, ctx)?).then_some(then)
        }
        Expr::Unary { op, operand, .. } => match op {
            crate::ast::UnaryOp::Not => Some(TypeRef::Boolean),
            _ => body_type(operand, bound, ctx),
        },
        Expr::Binary { op, lhs, rhs, .. } => match op {
            B::Lt | B::Le | B::Gt | B::Ge | B::Eq | B::Ne | B::And | B::Or => {
                Some(TypeRef::Boolean)
            }
            _ => {
                let (l, r) = (body_type(lhs, bound, ctx), body_type(rhs, bound, ctx));
                let string = TypeRef::Named(String::from("String"));
                if *op == B::Add && (l.as_ref() == Some(&string) || r.as_ref() == Some(&string)) {
                    return Some(string);
                }
                numeric_join(&l?, &r?)
            }
        },
        Expr::Call {
            receiver,
            method,
            args,
            ..
        } => call_body_type(expr, receiver.as_deref(), method, args, bound, ctx),
        _ => static_type_of(expr, ctx),
    }
}

/// The result of the library methods a lambda body commonly ends in. A SUBSET,
/// kept here only to type a mapped element: a method missing from it leaves the
/// element `Object`, which is where every one of them stood before.
#[allow(clippy::too_many_lines)] // one arm per library shape
fn library_return(receiver: &TypeRef, method: &str, argc: usize) -> Option<TypeRef> {
    let base = match receiver {
        TypeRef::Named(name) => name.rsplit('.').next().unwrap_or(name),
        TypeRef::Generic { base, .. } => base.rsplit('.').next().unwrap_or(base),
        _ => return None,
    };
    let string = || TypeRef::Named(String::from("String"));
    // A collection face, for the methods every one of them shares.
    let collection = matches!(
        base,
        "List"
            | "ArrayList"
            | "LinkedList"
            | "Set"
            | "HashSet"
            | "TreeSet"
            | "Map"
            | "HashMap"
            | "TreeMap"
            | "EnumMap"
            | "EnumSet"
            | "Collection"
    );
    match (base, method, argc) {
        // A SCANNER's accessors, all of them at once. A `var` holding
        // `in.nextLine()` — the first line of half the corpus's programs — had
        // no type, so the split or the stream after it had no element.
        ("Scanner", read, _) if scanner_answer(read).is_some() => scanner_answer(read),
        ("String", "charAt", 1) | ("Character", "charValue", 0) => Some(TypeRef::Char),
        (
            "String",
            "length" | "indexOf" | "lastIndexOf" | "compareTo" | "compareToIgnoreCase",
            _,
        )
        | ("Integer" | "Short" | "Byte", "intValue", 0)
        // A match's own spans, so a lambda over `results()` can chain.
        | ("Matcher" | "MatchResult", "start" | "end" | "groupCount", _)
        | (_, "hashCode", 0) => Some(TypeRef::Int),
        ("Long", "longValue", 0) => Some(TypeRef::Long),
        ("Double" | "Float", "doubleValue", 0) => Some(TypeRef::Double),
        (
            "String",
            "isEmpty" | "isBlank" | "contains" | "startsWith" | "endsWith" | "equalsIgnoreCase"
            | "matches",
            _,
        )
        | (_, "equals", 1) => Some(TypeRef::Boolean),
        (
            "String",
            "toUpperCase" | "toLowerCase" | "trim" | "strip" | "substring" | "replace" | "concat"
            | "repeat",
            _,
        )
        // A match's own text, likewise.
        | ("Matcher" | "MatchResult", "group", _)
        | (_, "toString", 0) => Some(string()),
        (_, "size", 0) if collection => Some(TypeRef::Int),
        (_, "isEmpty" | "contains" | "containsKey" | "containsValue", _) if collection => {
            Some(TypeRef::Boolean)
        }
        // The two String methods that answer an ARRAY. Without them
        // `Arrays.stream(s.split(","))` had no element type, so a lambda over
        // that stream took an `Object` and could not call a `String` method.
        ("String", "split", _) => Some(TypeRef::Array(Box::new(string()))),
        ("String", "toCharArray", 0) => Some(TypeRef::Array(Box::new(TypeRef::Char))),
        ("List" | "ArrayList" | "LinkedList", "get", 1) => element_of_declared(receiver),
        // A collection's own `stream()`, so a lambda that ANSWERS one carries
        // its element: `flatMap(inner -> inner.stream())` is the whole reason
        // `flatMap` exists, and codegen reads the answer off this class.
        (_, "stream" | "parallelStream", 0) => match receiver {
            // An `Optional<E>.stream()` is a stream of at most one `E`
            // (Java 9); a collection's is a stream of its element.
            TypeRef::Generic { base, args } if simple_base(base) == "Optional" && args.len() == 1 => {
                Some(TypeRef::Generic {
                    base: String::from("Stream"),
                    args: vec![args[0].clone()],
                })
            }
            _ => element_of_declared(receiver).map(|elem| TypeRef::Generic {
                base: String::from("Stream"),
                args: vec![elem],
            }),
        },
        // An `Optional<E>` inside a container: `stream.map(Optional::get)` over
        // a `Stream<Optional<Pet>>` is the shape that noticed — the mapped
        // element was `Object`, so the `Pet` method after it was "cannot find
        // symbol". `orElse`/`orElseThrow` answer the same element.
        ("Optional", "get" | "orElse" | "orElseThrow", _) => match receiver {
            TypeRef::Generic { args, .. } if args.len() == 1 => Some(args[0].clone()),
            _ => None,
        },
        // A standalone entry's two halves, for the same reason.
        ("Entry" | "Map.Entry", "getKey", 0) => match receiver {
            TypeRef::Generic { args, .. } if args.len() == 2 => Some(args[0].clone()),
            _ => None,
        },
        ("Entry" | "Map.Entry", "getValue", 0) => match receiver {
            TypeRef::Generic { args, .. } if args.len() == 2 => Some(args[1].clone()),
            _ => None,
        },
        ("TreeSet" | "SortedSet" | "NavigableSet", "first" | "last", 0) => {
            element_of_declared(receiver)
        }
        ("Map" | "HashMap" | "TreeMap", "get", 1) => match receiver {
            TypeRef::Generic { args, .. } if args.len() == 2 => Some(args[1].clone()),
            _ => None,
        },
        // The VIEWS and derived containers, whose element is the receiver's.
        // `var` reads this table now, so a local holding one of them — a
        // `subList`, a `keySet`, a `toArray`, a stream's `findFirst` — had no
        // type, though the same expression inline had always worked.
        (_, "subList", 2) if collection => Some(receiver.clone()),
        (_, "keySet", 0) => map_half(receiver, 0).map(|key| TypeRef::Generic {
            base: String::from("Set"),
            args: vec![key],
        }),
        (_, "values", 0) => map_half(receiver, 1).map(|value| TypeRef::Generic {
            base: String::from("Collection"),
            args: vec![value],
        }),
        (_, "toArray", _) => element_of_declared(receiver).map(|e| TypeRef::Array(Box::new(e))),
        (
            "Stream",
            "filter" | "sorted" | "distinct" | "limit" | "skip" | "peek" | "onClose" | "parallel"
            | "sequential" | "unordered",
            _,
        ) => Some(receiver.clone()),
        // A `Stream` is not in the collection set (its element is not read the
        // same way), so its single argument is taken directly.
        ("Stream", "findFirst" | "findAny", 0) => match receiver {
            TypeRef::Generic { args, .. } if args.len() == 1 => Some(TypeRef::Generic {
                base: String::from("Optional"),
                args: vec![args[0].clone()],
            }),
            _ => None,
        },
        _ => None,
    }
}

/// The KEY (0) or VALUE (1) of a written map type.
fn map_half(receiver: &TypeRef, at: usize) -> Option<TypeRef> {
    match receiver {
        TypeRef::Generic { base, args }
            if args.len() == 2
                && matches!(
                    simple_base(base),
                    "Map"
                        | "HashMap"
                        | "LinkedHashMap"
                        | "TreeMap"
                        | "SortedMap"
                        | "NavigableMap"
                        | "EnumMap"
                ) =>
        {
            args.get(at).cloned()
        }
        _ => None,
    }
}

/// The type binary numeric promotion gives two operands (JLS §5.6.2), over the
/// BOXED spellings a stream element wears as well as the primitives.
fn numeric_join(lhs: &TypeRef, rhs: &TypeRef) -> Option<TypeRef> {
    let rank = |t: &TypeRef| match t {
        TypeRef::Double | TypeRef::Float => Some(3),
        TypeRef::Long => Some(2),
        TypeRef::Int | TypeRef::Short | TypeRef::Byte | TypeRef::Char => Some(1),
        TypeRef::Named(name) => match name.rsplit('.').next().unwrap_or(name) {
            "Double" | "Float" => Some(3),
            "Long" => Some(2),
            "Integer" | "Short" | "Byte" | "Character" => Some(1),
            _ => None,
        },
        _ => None,
    };
    Some(match rank(lhs)?.max(rank(rhs)?) {
        3 => TypeRef::Double,
        2 => TypeRef::Long,
        _ => TypeRef::Int,
    })
}

/// `matcher.results()` — a stream of frozen matches — and
/// `pattern.splitAsStream(text)`, which is `split` handed over as a pipeline.
/// Both are typed from the RECEIVER, so a user method of either name keeps its
/// own meaning.
fn regex_stream_elem(prev: &Expr, method: &str, ctx: &Ctx) -> Option<TypeRef> {
    if !matches!(method, "results" | "splitAsStream") {
        return None;
    }
    let Some(TypeRef::Named(name)) = static_type_of(prev, ctx) else {
        return None;
    };
    match name.rsplit('.').next().unwrap_or(&name) {
        "Matcher" if method == "results" => Some(TypeRef::Named(String::from("MatchResult"))),
        "Pattern" if method == "splitAsStream" => Some(TypeRef::Named(String::from("String"))),
        _ => None,
    }
}

/// The current element type of a stream-pipeline receiver, for typing a stream
/// lambda's parameter. `X.stream()` yields the collection `X`'s element; the
/// element-preserving ops (`filter`/`sorted`/`distinct`/`limit`/`skip`/`peek`)
/// recurse into the prior stage; `map` erases it to `Object`.
#[allow(clippy::too_many_lines)] // one arm per stream source
/// The DECLARED type of a variable, parameter or `this` field — the two
/// shapes every reading here starts from.
fn declared_type_of(expr: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    match expr {
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0]),
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            ctx.lookup(name)
        }
        _ => None,
    }
}

/// The element of a stream a SUPPLIER answers: `Supplier<Stream<Pet>> s =
/// pets::stream; s.get().collect(...)`. The declared type says it, and reading
/// it is what lets the collector after it be typed — every other arm walks a
/// stream-producing CALL, and `get()` is not one.
fn supplied_stream_elem(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        ..
    } = receiver
    else {
        return None;
    };
    if method != "get" || !args.is_empty() {
        return None;
    }
    let TypeRef::Generic { base, args: held } = declared_type_of(owner, ctx)? else {
        return None;
    };
    if base.rsplit('.').next().unwrap_or(&base) != "Supplier" {
        return None;
    }
    let [
        TypeRef::Generic {
            base: inner,
            args: elem,
        },
    ] = &held[..]
    else {
        return None;
    };
    (inner.rsplit('.').next().unwrap_or(inner) == "Stream" && elem.len() == 1)
        .then(|| elem[0].clone())
}

/// The element of a stream that has been given a NAME — a variable, a
/// parameter, a `this` field. Every other reading walks a chain of calls, so a
/// stream with a name took a lambda nowhere: `Stream<String> s = ...;
/// s.filter(x -> ...)` was refused though the identical inline chain compiled,
/// which undercut the point of `Stream<T>` being a nameable type at all.
fn named_stream_elem(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    let declared = declared_type_of(receiver, ctx)?;
    match &declared {
        TypeRef::Generic { base, args } if args.len() == 1 => {
            (base.rsplit('.').next().unwrap_or(base) == "Stream").then(|| args[0].clone())
        }
        // The primitive pipelines carry their element in their name.
        TypeRef::Named(name) => match name.rsplit('.').next().unwrap_or(name) {
            "IntStream" => Some(TypeRef::Int),
            "DoubleStream" => Some(TypeRef::Double),
            "LongStream" => Some(TypeRef::Long),
            _ => None,
        },
        _ => None,
    }
}

fn stream_elem_type(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    if let Some(elem) = supplied_stream_elem(receiver, ctx) {
        return Some(elem);
    }
    let Expr::Call {
        receiver: Some(prev),
        method,
        args,
        ..
    } = receiver
    else {
        return named_stream_elem(receiver, ctx);
    };
    // A COLLECTION's own `stream()`. A receiver that is not one falls through
    // — a class of the program may declare a `stream()` of its own, and its
    // element is in its return type, which the last arm reads.
    if matches!(method.as_str(), "stream" | "parallelStream")
        && args.is_empty()
        && let Some(elem) = list_elem_type(prev, ctx)
    {
        return Some(elem);
    }
    // `Files.lines(path)` — a stream of the file's lines, like `"text".lines()`
    // below. Without this a lambda over one had no element and was refused.
    if method == "lines" && args.len() == 1 && names_library_class(prev.as_ref(), "Files") {
        return Some(TypeRef::Named(String::from("String")));
    }
    if let Some(elem) = regex_stream_elem(prev, method, ctx) {
        return Some(elem);
    }
    // `"text".lines()` — a Stream<String>; `chars()` — an IntStream.
    if method == "lines" && args.is_empty() {
        return Some(TypeRef::Named(String::from("String")));
    }
    if matches!(method.as_str(), "chars" | "codePoints") && args.is_empty() {
        return Some(TypeRef::Int);
    }
    // `IntStream.range(a, b)` / `rangeClosed(a, b)` / `IntStream.of(...)` —
    // sources of `int`s, and the same factories on the other two primitive
    // pipelines, whose elements carry their own width.
    if matches!(
        method.as_str(),
        "range" | "rangeClosed" | "of" | "empty" | "iterate" | "generate"
    ) {
        for (class, elem) in [
            ("IntStream", TypeRef::Int),
            ("LongStream", TypeRef::Long),
            ("DoubleStream", TypeRef::Double),
        ] {
            if names_library_class(prev.as_ref(), class) {
                return Some(elem);
            }
        }
    }
    // `Stream.iterate(seed, next)` — every element is a `next` of the seed, so
    // the seed's type is the element's. `generate`'s supplier answers a type
    // this pass cannot read, so that one erases.
    if method == "iterate" && args.len() == 2 && names_library_class(prev.as_ref(), "Stream") {
        return static_type_of(&args[0], ctx);
    }
    // `Stream.generate(supplier)` — the element is what the supplier ANSWERS,
    // and that is readable by the same route `map` uses: by the time an outer
    // op asks, the supplier is already a synthesized class whose body ends in
    // the value. A supplier this pass cannot read erases to `Object` rather
    // than to NOTHING — an element of NOTHING left every downstream lambda in
    // the chain with no target at all, so
    // `generate(() -> 2).limit(2).reduce(0, (a, b) -> a + b)` was refused
    // outright while the same chain over `iterate` compiled.
    if method == "generate" && args.len() == 1 && names_library_class(prev.as_ref(), "Stream") {
        return Some(
            mapped_element_type(args, ctx)
                .map_or_else(|| TypeRef::Named(String::from("Object")), boxed_element),
        );
    }
    // `Stream.of(...)` — the element is what the arguments agree on, which is
    // all this syntactic pass can see; a mixed or computed list erases to
    // `Object`, as it does after `map`.
    if method == "of" && names_library_class(prev.as_ref(), "Stream") {
        return Some(literal_element_type(args, ctx));
    }
    // `Stream.concat(a, b)` — both halves have the same element, so the first
    // one that answers is it. Missing here, a lambda applied INLINE to a
    // concatenation had no target type at all ("a lambda is only allowed where
    // a functional-interface type is expected"), while the very same chain
    // through a `Stream<String>` variable compiled: the tell that a stream
    // source is absent from this pass.
    if method == "concat" && args.len() == 2 && names_library_class(prev.as_ref(), "Stream") {
        return args
            .iter()
            .find_map(|half| stream_elem_type(half, ctx))
            .or_else(|| Some(TypeRef::Named(String::from("Object"))));
    }
    // `Arrays.stream(array)` — the array's element type.
    if method == "stream" && args.len() == 1 && names_library_class(prev.as_ref(), "Arrays") {
        return array_elem_type(&args[0], ctx);
    }
    match method.as_str() {
        // `boxed` retypes without changing what the element IS here (the VM
        // stores it unboxed either way), so it passes the element through too.
        "filter" | "sorted" | "distinct" | "limit" | "skip" | "peek" | "boxed"
        // `takeWhile`/`dropWhile` pass the element through unchanged, as
        // `filter` does, so a chain after one keeps its type.
        | "takeWhile" | "dropWhile"
        // …and so do the ops that change nothing about the ELEMENTS at all:
        // the parallel toggles, and registering a close handler. A chained
        // `onClose(…).onClose(…)` had no element for the second lambda.
        | "onClose" | "parallel" | "sequential" | "unordered" => {
            stream_elem_type(prev, ctx)
        }
        // `mapToInt` produces an int stream; `mapToObj` an erased one.
        // `as…Stream` widens every element to that primitive.
        "mapToInt" => Some(TypeRef::Int),
        "asLongStream" | "mapToLong" => Some(TypeRef::Long),
        "asDoubleStream" | "mapToDouble" => Some(TypeRef::Double),
        // `map` on a PRIMITIVE pipeline is an `IntUnaryOperator` and friends:
        // the element keeps its width. Only an object stream's `map` erases.
        "map" => Some(match stream_elem_type(prev, ctx) {
            Some(prim @ (TypeRef::Int | TypeRef::Long | TypeRef::Double)) => prim,
            _ => mapped_element_type(args, ctx)
                .unwrap_or_else(|| TypeRef::Named(String::from("Object"))),
        }),
        // `mapToObj` leaves the primitive pipeline for an object one, and
        // what the lambda answers is that stream's element — the same reading
        // `map` gets. `flatMap`'s lambda answers a STREAM, so its element is
        // that stream's element, one step further in.
        "mapToObj" => Some(
            mapped_element_type(args, ctx).unwrap_or_else(|| TypeRef::Named(String::from("Object"))),
        ),
        "flatMap" => Some(
            flat_element_type(args, ctx).unwrap_or_else(|| TypeRef::Named(String::from("Object"))),
        ),
        // A stream a USER method answers — a class whose `stream()` is
        // declared `Stream<Leaf>`. Every case above reads a LIBRARY shape, and
        // a method of the program says its element in its own return type; the
        // lambda after `tree.stream()` had no target without this.
        _ => {
            let answered = body_type(receiver, &HashMap::new(), ctx);
            if std::env::var("CATURRA_DEBUG_STREAM").is_ok() {
                eprintln!("stream_elem_type last arm: method={method} answered={answered:?}");
            }
            match answered {
                Some(TypeRef::Generic { base, args }) if args.len() == 1 => {
                    (base.rsplit('.').next().unwrap_or(&base) == "Stream").then(|| args[0].clone())
                }
                _ => None,
            }
        }
    }
}

/// The declared element type of a `List`/`ArrayList`/`Set`/`Collection`
/// receiver, read syntactically from the local, parameter, or field it names
/// — the same shape as `map_type_args`, for a single type argument.
/// Whether this expression is a `Collectors.xxx(...)` factory call.
fn is_collectors_call(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call {
            receiver: Some(owner),
            ..
        } if matches!(owner.as_ref(), Expr::Name { path, .. }
            if path.split_last().is_some_and(|(last, prefix)| last == "Collectors"
                && (prefix.is_empty()
                    || matches!(prefix.first().map(String::as_str), Some("java" | "javax")))))
    )
}

/// Target-type the lambdas inside a `Collectors.xxx(...)` factory against the
/// stream element they will be handed. Each factory says which of its arguments
/// is a function of the element, which is a predicate, and which is a nested
/// collector to recurse into.
fn desugar_collector(expr: &mut Expr, elem: &TypeRef, ctx: &mut Ctx) {
    let Expr::Call { method, args, .. } = expr else {
        return;
    };
    let object = TypeRef::Named(String::from("Object"));
    // How many leading arguments are one-parameter functions OF THE ELEMENT.
    let element_functions = match method.as_str() {
        "groupingBy" | "mapping" | "summingInt" | "summingLong" | "summingDouble"
        | "averagingInt" | "averagingLong" | "averagingDouble" | "flatMapping"
        | "summarizingInt" | "summarizingLong" | "summarizingDouble" => 1,
        "toMap" | "toUnmodifiableMap" => 2,
        _ => 0,
    };
    #[allow(clippy::needless_range_loop)] // each arm REPLACES args[index]
    for index in 0..args.len() {
        let is_lambda = matches!(&args[index], Expr::Lambda { .. } | Expr::MethodRef { .. });
        if index < element_functions && is_lambda {
            if matches!(&args[index], Expr::MethodRef { .. }) {
                let synth = Sam {
                    method: String::from("apply"),
                    params: vec![elem.clone()],
                    ret: object.clone(),
                };
                args[index] = method_ref_to_lambda(&args[index], &synth, ctx);
            }
            args[index] = build_erased_lambda(
                &mut args[index],
                "__UnaryOperator",
                "apply",
                &object,
                std::slice::from_ref(elem),
                None,
                ctx,
            );
            continue;
        }
        if desugar_collector_shape(method, args, index, is_lambda, elem, ctx) {
            continue;
        }
        if desugar_extreme_or_reducing(method, args, index, is_lambda, elem, ctx) {
            continue;
        }
        // A nested collector (`groupingBy(f, counting())`) sees the same
        // element as the outer one.
        if is_collectors_call(&args[index]) {
            desugar_collector(&mut args[index], elem, ctx);
            continue;
        }
        // `maxBy(Comparator.comparingInt(f -> …))` — the comparator is not a
        // lambda but a FACTORY CALL, and the lambda inside it needs the
        // element as much as a bare one would. The chain reads that from its
        // target type, so the target has to be handed down: with `None` the
        // inner lambda had no parameter type and `f.getName()` was "cannot
        // find symbol", in a collector whose bare-lambda form worked.
        if matches!(method.as_str(), "maxBy" | "minBy") && index == 0 {
            let target = TypeRef::Generic {
                base: String::from("Comparator"),
                args: vec![elem.clone()],
            };
            desugar_expr(&mut args[index], Some(&target), ctx);
            continue;
        }
        desugar_expr(&mut args[index], None, ctx);
    }
}

/// What `Stream.of(...)`'s arguments agree on, read from their literal forms —
/// the only inference available to a syntactic pass. Anything mixed or computed
/// erases to `Object`, exactly as a stream's element does after `map`.
/// What two element types have in common: the same type, the more general of
/// the two, or a supertype both reach. `Object` when nothing nearer is
/// written down — the answer this pass gave for EVERY pair before.
fn join_element_types(
    left: &TypeRef,
    right: &TypeRef,
    supers: &HashMap<String, Vec<String>>,
) -> TypeRef {
    let object = || TypeRef::Named(String::from("Object"));
    if left == right {
        return left.clone();
    }
    let (TypeRef::Named(a), TypeRef::Named(b)) = (left, right) else {
        return object();
    };
    // Every supertype of a name, nearest first (breadth first, so a class's
    // own `extends`/`implements` clause is read before what those extend).
    let ancestry = |start: &str| -> Vec<String> {
        let mut found: Vec<String> = vec![start.to_owned()];
        let mut queue: std::collections::VecDeque<String> =
            std::collections::VecDeque::from(vec![start.to_owned()]);
        while let Some(current) = queue.pop_front() {
            if found.len() > supers.len() + 2 {
                break;
            }
            for parent in supers.get(&current).into_iter().flatten() {
                if !found.contains(parent) {
                    found.push(parent.clone());
                    queue.push_back(parent.clone());
                }
            }
        }
        found
    };
    let theirs = ancestry(b);
    ancestry(a)
        .into_iter()
        .find(|name| theirs.contains(name))
        .map_or_else(object, TypeRef::Named)
}

fn literal_element_type(args: &[Expr], ctx: &Ctx) -> TypeRef {
    let supers = ctx.supers;
    let object = TypeRef::Named(String::from("Object"));
    // A LONE reference array is the varargs array itself, so the element is
    // the array's — `Stream.of(Kind.values())` is a stream of constants, not a
    // stream holding one array. (A primitive array is one element, which is
    // the varargs gotcha the emit side already models; there is no element
    // type to give it here either way.)
    if let [single] = args
        && let Some(TypeRef::Array(elem)) = body_type(single, &HashMap::new(), ctx)
        && !matches!(
            *elem,
            TypeRef::Int
                | TypeRef::Long
                | TypeRef::Double
                | TypeRef::Float
                | TypeRef::Short
                | TypeRef::Byte
                | TypeRef::Char
                | TypeRef::Boolean
        )
    {
        return *elem;
    }
    let mut kind: Option<TypeRef> = None;
    for arg in args {
        // A `new Point(…)` argument says its type outright, and reading only
        // LITERALS left `Stream.of(new Point(1, 2)).map(p -> p.x)` with an
        // `Object` for `p` — while the same stream taken from a declared
        // `List<Point>` had the element all along.
        // ...and an ARRAY written inline, for the same reason: `Stream.of(new
        // int[] {1, 2}, new int[] {3})` is a stream of two arrays, and its
        // lambda could not read `a.length` off an `Object`.
        if let Expr::NewArray { .. } = arg
            && let Some(this) = static_type_of(arg, ctx)
        {
            kind = Some(match &kind {
                Some(seen) if *seen == this => this,
                Some(_) => return object,
                None => this,
            });
            continue;
        }
        if let Expr::NewObject { class, .. } = arg {
            let this = TypeRef::Named(class.clone());
            kind = Some(match &kind {
                Some(seen) => join_element_types(seen, &this, supers),
                None => this,
            });
            continue;
        }
        // Anything else a lambda body could be typed by: a local, a field, a
        // call whose return is known. Reading only LITERALS left
        // `Stream.of(first, second)` over two declared `String`s with an
        // `Object` element, though the same two spelled out inline were
        // `String`s.
        if let Some(known) = body_type(arg, &HashMap::new(), ctx) {
            let this = boxed_element(known);
            kind = Some(match &kind {
                Some(seen) => join_element_types(seen, &this, supers),
                None => this,
            });
            continue;
        }
        let Expr::Literal { value, .. } = arg else {
            return object;
        };
        let this = match value {
            crate::ast::Literal::Int(_) => TypeRef::Named(String::from("Integer")),
            crate::ast::Literal::Str(_) => TypeRef::Named(String::from("String")),
            crate::ast::Literal::Double(_) => TypeRef::Named(String::from("Double")),
            crate::ast::Literal::Long(_) => TypeRef::Named(String::from("Long")),
            crate::ast::Literal::Char(_) => TypeRef::Named(String::from("Character")),
            crate::ast::Literal::Bool(_) => TypeRef::Named(String::from("Boolean")),
            crate::ast::Literal::Null | crate::ast::Literal::Float(_) => return object,
        };
        match &kind {
            Some(seen) if *seen != this => return object,
            _ => kind = Some(this),
        }
    }
    kind.unwrap_or(object)
}

/// The element type of a receiver declared as an array (`int[]` → `int`,
/// `String[]` → `String`) — for typing the generator of `Arrays.setAll`.
/// What a LIBRARY STATIC answers, for the handful whose return is a scalar or
/// a String. The instance table beside this one is keyed by the receiver's
/// TYPE, which a static call has no value to give — the receiver is a class
/// name. Kept to the calls a lambda body actually makes; anything else stays
/// unknown, which is where it was.
fn library_static_type(class: &str, method: &str, argc: usize) -> Option<TypeRef> {
    // Written as a sequence of questions rather than one table: the families
    // cut across classes (every wrapper's `toString` is a String, every
    // `parse` a scalar of its own width), and a match keyed by the pair says
    // that far less clearly.
    let wrapper = matches!(
        class,
        "Integer" | "Long" | "Double" | "Float" | "Short" | "Byte" | "Boolean" | "Character"
    );
    if class == "String" && matches!(method, "valueOf" | "copyValueOf" | "format" | "join") {
        return Some(TypeRef::Named(String::from("String")));
    }
    if (wrapper && method == "toString")
        || (matches!(class, "Integer" | "Long")
            && matches!(method, "toBinaryString" | "toHexString" | "toOctalString"))
        || (class == "Objects" && method == "toString")
    {
        return Some(TypeRef::Named(String::from("String")));
    }
    if class == "Math" {
        // `abs`/`max`/`min`/`round` and the exact-arithmetic helpers answer the
        // ARGUMENT's own width, which nothing here knows; the rest are doubles.
        let by_argument = matches!(
            method,
            "abs"
                | "max"
                | "min"
                | "round"
                | "floorDiv"
                | "floorMod"
                | "addExact"
                | "subtractExact"
                | "multiplyExact"
                | "toIntExact"
                | "negateExact"
                | "incrementExact"
                | "decrementExact"
        );
        return (!by_argument).then_some(TypeRef::Double);
    }
    let answer = match class {
        "Integer" => match method {
            "parseInt" | "compare" | "signum" | "bitCount" | "max" | "min" | "sum" => TypeRef::Int,
            _ => return None,
        },
        "Long" => match method {
            "parseLong" | "max" | "min" | "sum" => TypeRef::Long,
            "compare" | "signum" | "bitCount" => TypeRef::Int,
            _ => return None,
        },
        "Double" | "Float" => match (method, argc) {
            ("parseDouble" | "parseFloat", 1) | ("max" | "min" | "sum", 2) => TypeRef::Double,
            ("compare" | "signum", _) => TypeRef::Int,
            _ => return None,
        },
        "Boolean" => match method {
            "parseBoolean" | "logicalAnd" | "logicalOr" | "logicalXor" => TypeRef::Boolean,
            "compare" => TypeRef::Int,
            _ => return None,
        },
        "Character" => match method {
            "toUpperCase" | "toLowerCase" | "forDigit" => TypeRef::Char,
            "getNumericValue" | "compare" | "digit" => TypeRef::Int,
            "isDigit" | "isLetter" | "isLetterOrDigit" | "isUpperCase" | "isLowerCase"
            | "isWhitespace" | "isAlphabetic" | "isSpaceChar" => TypeRef::Boolean,
            _ => return None,
        },
        "Objects" => match method {
            "equals" | "isNull" | "nonNull" => TypeRef::Boolean,
            "hash" | "hashCode" => TypeRef::Int,
            _ => return None,
        },
        _ => return None,
    };
    Some(answer)
}

/// The user CLASS an expression's declared type names — a local, a `this`
/// field, a `new`, or `this` itself. Enough to look up that class's own method
/// shapes; the sibling above answers the same question for a FUNCTIONAL
/// receiver, where the name is aliased to a bundled interface.
fn declared_class_name(owner: &Expr, ctx: &Ctx) -> Option<String> {
    let ty = match owner {
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0])?,
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            ctx.lookup(name)?
        }
        Expr::NewObject { class, .. } => TypeRef::Named(class.clone()),
        Expr::This { .. } => TypeRef::Named(ctx.current_class?.to_owned()),
        // A BUILDER chain: `new Roster().add(x).add(y)` is a `Roster`, and
        // each link is named by the one before it. Recursing on the receiver
        // rather than on the call itself is what terminates — at the `new`.
        Expr::Call {
            receiver: Some(inner),
            method,
            args,
            ..
        } => user_method_return(inner, method, args.len(), ctx)?,
        _ => return None,
    };
    match ty {
        TypeRef::Named(name) => Some(name),
        TypeRef::Generic { base, .. } => Some(base),
        _ => None,
    }
}

/// The declared type of `object.name`, where `object`'s class the pass can
/// name. Walked OUTWARD, since a field may be inherited.
fn field_of_object(object: &Expr, name: &str, ctx: &Ctx) -> Option<TypeRef> {
    field_of_class(&declared_class_name(object, ctx)?, name, ctx)
}

/// The declared type of a field on `class` or on any class above it — a field
/// a class INHERITS is read by its simple name like any other.
fn field_of_class(class: &str, name: &str, ctx: &Ctx) -> Option<TypeRef> {
    let mut owner = class.to_owned();
    let mut seen: Vec<String> = Vec::new();
    loop {
        if let Some(ty) = ctx.fields.get(&(owner.clone(), name.to_owned())) {
            return Some(ty.clone());
        }
        if seen.contains(&owner) {
            return None;
        }
        seen.push(owner.clone());
        owner = ctx.supers.get(&owner)?.first()?.clone();
    }
}

/// The ENUM an expression names, when it names one: `Kind.values()`'s owner.
fn enum_owner_name(owner: &Expr, ctx: &Ctx) -> Option<String> {
    let Expr::Name { path, .. } = owner else {
        return None;
    };
    let name = path.last()?;
    ctx.enums.contains(name).then(|| name.clone())
}

fn array_elem_type(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    let ty = match receiver {
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0])?,
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            ctx.lookup(name)?
        }
        // `Kind.values()` — an ENUM's own array of its constants. It has no
        // declaration to look up, so a stream over it had no element type and
        // the lambda after it was refused as if the position were not a
        // functional-interface one, though the same array in a VARIABLE worked.
        Expr::Call {
            receiver: Some(owner),
            method,
            args,
            ..
        } if method == "values" && args.is_empty() => {
            let name = enum_owner_name(owner, ctx)?;
            return Some(TypeRef::Named(name));
        }
        // A library call that ANSWERS an array — `csv.split(",")`,
        // `word.toCharArray()`, `list.toArray(new String[0])`. Each is the
        // ordinary way to get one, and a stream over any of them had no
        // element type inline while the same array through a VARIABLE worked.
        Expr::Call {
            receiver: Some(owner),
            method,
            args,
            ..
        } if matches!(
            method.as_str(),
            "split"
                | "toCharArray"
                | "getBytes"
                | "toArray"
                | "copyOf"
                | "copyOfRange"
                | "listFiles"
        ) =>
        {
            return match method.as_str() {
                "split" => Some(TypeRef::Named(String::from("String"))),
                "toCharArray" => Some(TypeRef::Char),
                "getBytes" => Some(TypeRef::Byte),
                // `dir.listFiles()` — the other ordinary way to get an array,
                // and the one a program streams over to read a directory.
                "listFiles" => Some(TypeRef::Named(String::from("File"))),
                // `Arrays.copyOf(source, n)` keeps the SOURCE's element.
                "copyOf" | "copyOfRange" => array_elem_type(args.first()?, ctx).filter(|_| {
                    matches!(owner.as_ref(), Expr::Name { path, .. }
                            if path.last().is_some_and(|name| name == "Arrays"))
                }),
                // `toArray(new String[0])` says its element in the MODEL it is
                // given; the no-argument form answers `Object[]`.
                _ => match args.first() {
                    Some(model) => array_elem_type(model, ctx),
                    None => Some(TypeRef::Named(String::from("Object"))),
                },
            };
        }
        // An array a method of the PROGRAM hands back —
        // `Arrays.stream(store.array())`. Every arm above reads a LIBRARY call
        // or a declaration; a class's own method says the array in its return
        // type, and without it the inline form was refused while the same
        // array through a variable compiled.
        Expr::Call {
            receiver: Some(owner),
            method,
            args,
            ..
        } if user_method_return(owner, method, args.len(), ctx).is_some() => {
            user_method_return(owner, method, args.len(), ctx)?
        }
        // An array written INLINE — `Arrays.stream(new int[]{1, 2, 3})` — is
        // its own declaration. Only a variable was looked up, so the identical
        // call on a literal array had no element type and the lambda after it
        // was refused as if the position were not a functional-interface one.
        Expr::NewArray { elem, dims, .. } => {
            let mut ty = elem.clone();
            for _ in 0..dims.len().max(1).saturating_sub(1) {
                ty = TypeRef::Array(Box::new(ty));
            }
            return Some(ty);
        }
        _ => return None,
    };
    match ty {
        TypeRef::Array(elem) => Some(*elem),
        _ => None,
    }
}

/// The element type `E` of a receiver declared `Optional<E>` — for typing the
/// lambda parameter of `ifPresent`/`filter`. Resolves a variable or a `this`
/// field, and walks a CHAINED receiver: `filter` keeps the element, `map`
/// erases it to `Object`, and a stream's `findFirst`/`max`/`min` terminal
/// carries the stream's element — so
/// `list.stream().filter(p).findFirst().ifPresent(x -> ...)` types its lambda.
fn optional_elem_type(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    // A call to one of the PROGRAM's own generic methods that answers an
    // `Optional<T>` — with `T` pinned from the arguments or an explicit
    // witness, the same reading a collection-returning one gets. Asked BEFORE
    // the shapes below, which all want a receiver: a bare `opt("ab")` in the
    // same class has none.
    if let Some(TypeRef::Generic { base, args }) = generic_call_return(receiver, ctx)
        && base.rsplit('.').next().unwrap_or(base.as_str()) == "Optional"
        && args.len() == 1
    {
        return Some(args[0].clone());
    }
    if let Expr::Call {
        receiver: Some(prev),
        method,
        args,
        ..
    } = receiver
    {
        // `Optional.of(x)` / `ofNullable(x)` used STRAIGHT as a receiver: the
        // element is what the argument looks like, the same reading
        // `List.of(...)` already gets. Without it the identical chain over a
        // DECLARED Optional worked and `Optional.ofNullable("v").map(...)` did
        // not.
        if matches!(method.as_str(), "of" | "ofNullable")
            && args.len() == 1
            && names_library_class(prev, "Optional")
        {
            return Some(literal_element_type(args, ctx));
        }
        // `Optional.<String>empty()` — a WITNESS says the element where there
        // is no argument to read it from, and an empty Optional is exactly the
        // receiver a program writes `ifPresentOrElse` on.
        if method == "empty" && args.is_empty() && names_library_class(prev, "Optional") {
            if let Expr::Call { type_args, .. } = receiver
                && let [witness] = &type_args[..]
            {
                return Some(witness.clone());
            }
            return Some(TypeRef::Named(String::from("Object")));
        }
        return match method.as_str() {
            // Optional.filter: same element. (A STREAM's filter resolves to
            // None here — its chain never bottoms out in an Optional.)
            "filter" => optional_elem_type(prev, ctx),
            // `Optional.map` answers what its LAMBDA answers, the same reading
            // a stream's `map` gets — `Optional.of(s).map(String::toUpperCase)`
            // is an `Optional<String>`, and erasing it to `Object` refused the
            // `filter` after it and the assignment to an `Optional<String>`.
            // `flatMap` answers an Optional, whose own element this does not
            // chase, so it stays erased.
            "map" => optional_elem_type(prev, ctx).map(|_| {
                mapped_element_type(args, ctx)
                    .unwrap_or_else(|| TypeRef::Named(String::from("Object")))
            }),
            "flatMap" => {
                optional_elem_type(prev, ctx).map(|_| TypeRef::Named(String::from("Object")))
            }
            "findFirst" | "findAny" | "max" | "min" => stream_elem_type(prev, ctx),
            // `reduce(accumulator)` — the ONE-argument form answers an
            // `Optional` of the stream's own element (the two-argument form
            // answers the element itself, and is not an Optional at all). So
            // `stream.reduce((a, b) -> …).map(Item::label)` had no element for
            // the `map`'s lambda, and was refused for having no
            // functional-interface position.
            "reduce" if args.len() == 1 => stream_elem_type(prev, ctx),
            // A method of the PROGRAM that answers an `Optional<E>` —
            // `shelf.longest().map(Book::title)`, which is how a class hands
            // back "maybe one". Every arm above reads a LIBRARY chain; the
            // program's own method says its element in its return type.
            _ => match user_method_return(prev, method, args.len(), ctx) {
                Some(TypeRef::Generic {
                    base,
                    args: written,
                }) if simple_base(&base) == "Optional" && written.len() == 1 => {
                    Some(written[0].clone())
                }
                _ => None,
            },
        };
    }
    let ty = match receiver {
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0])?,
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            ctx.lookup(name)?
        }
        _ => return None,
    };
    let TypeRef::Generic { base, args } = ty else {
        return None;
    };
    // The SIMPLE name: `java.util.Optional<String>` is the same declared type
    // as `Optional<String>`, and comparing the written base against "Optional"
    // meant a qualified declaration lost its element — so a lambda on one had
    // no target type, though the list, map and stream lookups beside it all
    // normalize.
    (base.rsplit('.').next().unwrap_or(base.as_str()) == "Optional" && args.len() == 1)
        .then(|| args[0].clone())
}

/// Whether a simple class name is one of the collections whose single type
/// argument IS its element — the set two places need, and the reason a
/// constructed collection cannot be recognised by "not a user class": the
/// name-disambiguation set deliberately holds `ArrayList`, so testing against
/// it excluded exactly the most common collection there is.
fn is_collection_class(simple: &str) -> bool {
    matches!(
        simple,
        "ArrayList"
            | "List"
            | "Set"
            | "HashSet"
            | "LinkedHashSet"
            | "TreeSet"
            | "SortedSet"
            | "NavigableSet"
            | "LinkedList"
            | "ArrayDeque"
            | "Stack"
            | "Queue"
            | "Deque"
            | "PriorityQueue"
            | "Collection"
            // An `EnumSet<E>` is a set of its enum, like any other.
            | "EnumSet"
            // A cursor declared `Iterator<E>` walks `E`s, for
            // `forEachRemaining`.
            | "Iterator"
            | "ListIterator"
    )
}

/// The ELEMENT of a written collection type: `List<String>` → `String`.
fn element_of_declared(ty: &TypeRef) -> Option<TypeRef> {
    match ty {
        TypeRef::Generic { base, args }
            if is_collection_class(simple_base(base.as_str())) && args.len() == 1 =>
        {
            Some(args[0].clone())
        }
        _ => None,
    }
}

/// The declared type of an expression that names an ARRAY, when the pass can
/// see one — a local, or a `this` field.
fn declared_array_type(expr: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    match expr {
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0]),
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            ctx.lookup(name)
        }
        // An array a METHOD hands back — `Arrays.stream(store.array())`, where
        // the array is built inside the class that owns it. Only a variable
        // and a `this` field were read.
        Expr::Call {
            receiver: Some(owner),
            method,
            args,
            ..
        } => user_method_return(owner, method, args.len(), ctx),
        _ => None,
    }
}

#[allow(clippy::too_many_lines)] // one arm per receiver shape
fn list_elem_type(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    // A RANGE or DESCENDING view of a sorted SET holds the same element it
    // does, the way `subList` and `unmodifiableList` below already do.
    if let Expr::Call {
        receiver: Some(inner),
        method,
        ..
    } = receiver
        && matches!(
            method.as_str(),
            "headSet" | "tailSet" | "subSet" | "descendingSet"
        )
    {
        return list_elem_type(inner, ctx);
    }
    // `list.subList(a, b)` is a live view OF that list, so its element is the
    // same one — as `unmodifiableList` below already knew.
    if let Expr::Call {
        receiver: Some(inner),
        method,
        args,
        ..
    } = receiver
        && method == "subList"
        && args.len() == 2
    {
        return list_elem_type(inner, ctx);
    }
    // A call to a generic method says what it returns once its variables are
    // pinned — by an explicit witness, or by the arguments. Both say more than
    // any of the shapes below: `Collections.<String>emptyList()` has no
    // argument to read, and `box("ab")` returns `List<T>` as written.
    if let Some(ty) = generic_call_return(receiver, ctx).or_else(|| witnessed_return(receiver, ctx))
        && let Some(elem) = element_of_declared(&ty)
    {
        return Some(elem);
    }
    // The shapes whose element is written down somewhere the pass can reach.
    // Reading only a NAME (and a `this` field) meant that a collection reached
    // through a CAST, a ternary, an array element or any library factory but
    // `of`/`asList` had no element type, and every lambda over one was refused
    // for having no functional-interface position — a cross-product of receiver
    // shapes against the four consumers that need an element found 44 such
    // cells and 0 among the shapes already handled.
    match receiver {
        // `((List<String>) o).forEach(…)` — the cast says what it is.
        Expr::Cast { ty, .. } => return element_of_declared(ty),
        // Both branches have the same type; either one answers.
        Expr::Ternary { then, els, .. } => {
            return list_elem_type(then, ctx).or_else(|| list_elem_type(els, ctx));
        }
        // `rows[0].forEach(…)` — the ARRAY's element is the collection.
        Expr::Index { array, .. } => {
            if let Some(TypeRef::Array(elem)) = declared_array_type(array, ctx) {
                return element_of_declared(&elem);
            }
        }
        _ => {}
    }
    // A call to a USER method that returns a collection. The receiver is
    // whatever the method says it gives back, which the pass knows for a bare
    // call in the current class and for one on `this`.
    if let Expr::Call {
        receiver: owner,
        method,
        args,
        ..
    } = receiver
        && matches!(owner.as_deref(), None | Some(Expr::This { .. }))
        && let Some(class) = ctx.current_class
        && let Some(shapes) = ctx.shapes.get(class)
        && let Some(shape) = shapes
            .iter()
            .find(|shape| shape.name == *method && shape.arity == args.len())
        && let Some(elem) = element_of_declared(&shape.return_type)
    {
        return Some(elem);
    }
    // A user method called on a TYPED receiver: `b.all()` where `b` is a
    // `Branch` and `Branch.all()` returns a `List<Node>`. Only the class being
    // WALKED had its shapes consulted, so the identical call on another
    // object's method had no element and the lambda after it was refused for
    // having no functional-interface position.
    if let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        ..
    } = receiver
        && let Some(class) = declared_class_name(owner, ctx)
        && let Some(shapes) = ctx.shapes.get(&class)
        && let Some(shape) = shapes
            .iter()
            .find(|shape| shape.name == *method && shape.arity == args.len())
        && let Some(elem) = element_of_declared(&shape.return_type)
    {
        return Some(elem);
    }
    // A USER type that implements `Iterable<E>` or `Iterator<E>`: the
    // `forEach`/`forEachRemaining` it inherits is a DEFAULT method of the
    // library interface, and the element its callback takes is the argument
    // the class writes on that interface. Without this a lambda over one had
    // no target type at all — "a lambda is only allowed where a
    // functional-interface type is expected", for the ordinary
    // `for`-loop-in-a-callback a JDK gives every Iterable.
    for owner in ["Iterable", "Iterator"] {
        if let Some(args) = receiver_class_arguments(receiver, owner, ctx)
            && let [elem] = args.as_slice()
        {
            return Some(elem.clone());
        }
    }
    // A library READ whose type is written on the RECEIVER: `map.get(k)` is
    // the map's value type, `list.get(i)` / `queue.poll()` / `opt.orElse(d)`
    // its element. Without it a collection stored INSIDE another collection
    // had no element of its own, so `byKey.get("k").stream().map(v -> …)` was
    // refused for having no functional-interface position — while the same
    // list through a variable compiled, the tell of a missing recogniser.
    if let Expr::Call {
        receiver: Some(owner),
        method,
        ..
    } = receiver
        && let Some(TypeRef::Generic {
            base,
            args: written,
        }) = static_type_of(owner, ctx)
    {
        let read = match (simple_base(&base), method.as_str(), written.len()) {
            (
                "Map" | "HashMap" | "LinkedHashMap" | "TreeMap" | "SortedMap" | "NavigableMap"
                | "EnumMap",
                "get" | "getOrDefault" | "remove" | "put" | "putIfAbsent" | "computeIfAbsent"
                | "compute" | "computeIfPresent" | "merge" | "replace",
                2,
            ) => written.get(1),
            (
                _,
                "get" | "getFirst" | "getLast" | "peek" | "peekFirst" | "peekLast" | "poll"
                | "pollFirst" | "pollLast" | "pop" | "element" | "remove" | "first" | "last"
                | "orElse" | "orElseThrow" | "floor" | "ceiling" | "higher" | "lower",
                1,
            ) => written.first(),
            _ => None,
        };
        if let Some(read) = read
            && let Some(elem) = element_of_declared(read)
        {
            return Some(elem);
        }
    }
    // The `Collections` factories and `copyOf`, whose element comes from what
    // they are given rather than from a type argument.
    if let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        ..
    } = receiver
    {
        let from_collections = names_library_class(owner.as_ref(), "Collections");
        let from_factory = names_library_class(owner.as_ref(), "List")
            || names_library_class(owner.as_ref(), "Set")
            || names_library_class(owner.as_ref(), "Collection");
        match (method.as_str(), &args[..]) {
            // One element, written as the argument.
            ("singletonList" | "singleton", [only]) if from_collections => {
                return Some(literal_element_type(std::slice::from_ref(only), ctx));
            }
            ("nCopies", [_, only]) if from_collections => {
                return Some(literal_element_type(std::slice::from_ref(only), ctx));
            }
            // A WRAPPER or a copy: the element is the source's.
            (
                "unmodifiableList"
                | "unmodifiableSet"
                | "unmodifiableCollection"
                | "unmodifiableSortedSet",
                [source],
            ) if from_collections => return list_elem_type(source, ctx),
            ("copyOf", [source]) if from_factory => return list_elem_type(source, ctx),
            _ => {}
        }
    }
    // A collection CONSTRUCTED in place — `new ArrayList<>(source).removeIf(x
    // -> …)`. Its element is written (`new ArrayList<String>()`) or comes from
    // what it copies (`new ArrayList<>(aStringList)`). Without this the
    // receiver had no element type and the lambda was refused for having no
    // functional-interface position, though the same call on a DECLARED
    // variable one line up compiled.
    if let Expr::NewObject {
        class,
        type_args,
        args,
        ..
    } = receiver
        && is_collection_class(simple_base(class))
    {
        if let [written] = &type_args[..] {
            return Some(written.clone());
        }
        // A diamond: read the element off the copy source.
        if let [source] = &args[..] {
            return list_elem_type(source, ctx);
        }
    }
    // `List.of("a", "b")` / `Set.of(...)` / `Arrays.asList(...)` used straight
    // as a source. The element is what the arguments agree on, which is all
    // this syntactic pass can see — the same reading `Stream.of(...)` already
    // gets. Without it `List.of("a").stream().map(String::toUpperCase)` had no
    // element for the lambda, though the identical pipeline over a DECLARED
    // list worked and `List.of("a").get(0).toUpperCase()` did too.
    if let Expr::Call {
        receiver: Some(owner),
        method,
        args,
        ..
    } = receiver
        && !args.is_empty()
        && matches!(method.as_str(), "of" | "asList")
        && (names_library_class(owner.as_ref(), "List")
            || names_library_class(owner.as_ref(), "Set")
            || names_library_class(owner.as_ref(), "Arrays"))
    {
        return Some(literal_element_type(args, ctx));
    }
    if let Expr::Call {
        receiver: Some(inner),
        method,
        args,
        ..
    } = receiver
        && args.is_empty()
    {
        // A `map.keySet()`/`values()`/`entrySet()` view: the element is the
        // map's key, its value, or a whole `Map.Entry` — which is what makes
        // `entrySet().removeIf(e -> ...)` the only form that can decide by key
        // AND value together.
        if let Some((key, value)) = map_type_args(inner, ctx) {
            return match method.as_str() {
                "keySet" => Some(key),
                "values" => Some(value),
                "entrySet" => Some(TypeRef::Generic {
                    base: String::from("Map.Entry"),
                    args: vec![key, value],
                }),
                _ => None,
            };
        }
        // A sorted map's KEYS as a navigable set.
        if matches!(method.as_str(), "navigableKeySet" | "descendingKeySet")
            && let Some((key, _)) = map_type_args(inner, ctx)
        {
            return Some(key);
        }
        // A cursor over a collection walks that collection's elements:
        // `list.iterator().forEachRemaining(x -> ...)`.
        if matches!(method.as_str(), "iterator" | "listIterator") {
            return list_elem_type(inner, ctx);
        }
        return None;
    }
    let ty = match receiver {
        Expr::Name { path, .. } if path.len() == 1 => ctx.lookup(&path[0])?,
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            ctx.lookup(name)?
        }
        Expr::NewObject {
            class, type_args, ..
        } if type_args.len() == 1 => TypeRef::Generic {
            base: class.clone(),
            args: type_args.clone(),
        },
        // Every other shape the general reader can name — a field reached
        // through ANOTHER object, most of all: `deck.cards.stream()` is as
        // ordinary as a field gets, and only `this.cards` was read.
        other => static_type_of(other, ctx)?,
    };
    let TypeRef::Generic { base, args } = ty else {
        return None;
    };
    (is_collection_class(simple_base(base.as_str())) && args.len() == 1).then(|| args[0].clone())
}

/// The lambda class for `map.forEach((k, v) -> ...)`. `__BiConsumer.accept`
/// is erased to `(Object, Object)`, so the body opens with the two casts
/// javac would put in a bridge method: `Double k = (Double) __k;`. The
/// lambda's own parameter names then have the map's declared types.
fn build_bi_consumer_class(
    lambda: &mut Expr,
    key: &TypeRef,
    value: &TypeRef,
    ctx: &mut Ctx,
) -> Expr {
    build_erased_lambda(
        lambda,
        "__BiConsumer",
        "accept",
        &TypeRef::Void,
        &[key.clone(), value.clone()],
        None,
        ctx,
    )
}

/// Rewrite every `return e;` in `stmts` to `{ T __caturraResult = e; return
/// __caturraResult; }`, so the body's result is checked against the element
/// type `ty`. Recurses into nested statements; a lambda has no inner method
/// or class to stop at.
fn coerce_returns(stmts: &mut [Stmt], ty: &TypeRef, span: crate::diagnostics::SourceSpan) {
    for stmt in stmts {
        coerce_return_in(stmt, ty, span);
    }
}

/// Desugar every `return` expression in a lambda's BLOCK body against the
/// lambda's own result type, before the block is walked normally.
///
/// An expression-bodied lambda gets that target from its caller; a block one
/// reaches its body through `desugar_stmt`, which knows no expected type — so
/// `x -> { return y -> x + y; }` was refused where `x -> y -> x + y` compiled.
/// Running first means the inner lambda is already a class by the time the
/// ordinary walk reaches it, and `coerce_returns` then wraps the result as it
/// does for any other body.
fn target_type_returns(stmts: &mut [Stmt], ty: &TypeRef, ctx: &mut Ctx) {
    for stmt in stmts {
        target_type_return_in(stmt, ty, ctx);
    }
}

fn target_type_return_in(stmt: &mut Stmt, ty: &TypeRef, ctx: &mut Ctx) {
    match stmt {
        Stmt::Return { value: Some(e), .. } => desugar_expr(e, Some(ty), ctx),
        Stmt::Block(body) => target_type_returns(body, ty, ctx),
        Stmt::If { then, els, .. } => {
            target_type_return_in(then, ty, ctx);
            if let Some(e) = els {
                target_type_return_in(e, ty, ctx);
            }
        }
        Stmt::While { body, .. }
        | Stmt::DoWhile { body, .. }
        | Stmt::For { body, .. }
        | Stmt::ForEach { body, .. }
        | Stmt::Labeled { body, .. } => target_type_return_in(body, ty, ctx),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            target_type_returns(body, ty, ctx);
            for c in catches {
                target_type_returns(&mut c.body, ty, ctx);
            }
            if let Some(f) = finally_body {
                target_type_returns(f, ty, ctx);
            }
        }
        Stmt::Switch { arms, .. } => {
            for arm in arms {
                target_type_returns(&mut arm.body, ty, ctx);
            }
        }
        _ => {}
    }
}

fn coerce_return_in(stmt: &mut Stmt, ty: &TypeRef, span: crate::diagnostics::SourceSpan) {
    match stmt {
        Stmt::Return { value: Some(_), .. } => {
            let Stmt::Return { value, .. } = stmt else {
                unreachable!("matched Return");
            };
            let e = value.take().expect("matched Some");
            *stmt = Stmt::Block(vec![
                Stmt::LocalDecl {
                    ty: ty.clone(),
                    is_final: false,
                    declarators: vec![crate::ast::LocalDeclarator {
                        name: String::from("__caturraResult"),
                        init: Some(e),
                        span,
                        extra_dims: 0,
                    }],
                    span,
                },
                Stmt::Return {
                    value: Some(Expr::Name {
                        path: vec![String::from("__caturraResult")],
                        span,
                    }),
                    span,
                },
            ]);
        }
        Stmt::Block(body) => coerce_returns(body, ty, span),
        Stmt::If { then, els, .. } => {
            coerce_return_in(then, ty, span);
            if let Some(e) = els {
                coerce_return_in(e, ty, span);
            }
        }
        Stmt::While { body, .. }
        | Stmt::DoWhile { body, .. }
        | Stmt::For { body, .. }
        | Stmt::ForEach { body, .. }
        | Stmt::Labeled { body, .. } => coerce_return_in(body, ty, span),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            coerce_returns(body, ty, span);
            for c in catches {
                coerce_returns(&mut c.body, ty, span);
            }
            if let Some(f) = finally_body {
                coerce_returns(f, ty, span);
            }
        }
        Stmt::Switch { arms, .. } => {
            for arm in arms {
                coerce_returns(&mut arm.body, ty, span);
            }
        }
        _ => {}
    }
}

/// Build the erased functional class for a collection lambda: `__Consumer`,
/// `__Predicate`, or `__BiConsumer`. Its SAM takes `Object` parameters (erased
/// generics) and opens with a cast of each back to the collection's declared
/// element type, the way javac's bridge method does, binding the lambda's own
/// parameter names. An expression body returns for a non-void SAM.
#[allow(clippy::too_many_lines)] // one class-assembly, linear
/// `Predicate.not(p)` — a predicate-shaped expression that is not itself a
/// lambda, so the guards that look for one have to admit it explicitly.
fn is_negated_predicate(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { receiver: Some(owner), method, args, .. }
        if method == "not"
            && args.len() == 1
            && matches!(owner.as_ref(), Expr::Name { path, .. }
                if path.last().is_some_and(|n| n == "Predicate")))
}

/// The type a lambda's PARAMETER takes for an element that may be a wildcard:
/// `? extends Number` reads as `Number`, `?` and an erased type variable as
/// `Object`. A wildcard is an encoded NAME, and no type of its own.
fn readable_elem(ty: TypeRef) -> TypeRef {
    let TypeRef::Named(name) = &ty else {
        return ty;
    };
    match crate::ast::wildcard_parts(name) {
        Some((_, bound)) if !bound.is_empty() => TypeRef::Named(bound.to_owned()),
        Some(_) => TypeRef::Named(String::from("Object")),
        None => ty,
    }
}

#[allow(clippy::too_many_lines)] // the erasure, plus one arm per lambda shape
fn build_erased_lambda(
    lambda: &mut Expr,
    interface: &str,
    method: &str,
    ret: &TypeRef,
    elem_types: &[TypeRef],
    // For `replaceAll`, the declared element type the result must convert to.
    // The erased SAM returns `Object`, which would otherwise accept any
    // reference; assigning the body to a local of this type restores the
    // check javac makes on `UnaryOperator<E>.apply`.
    result_type: Option<&TypeRef>,
    ctx: &mut Ctx,
) -> Expr {
    // `Predicate.not(inner)` is TRANSPARENT to target typing: the predicate
    // inside it takes the SAME element the position wants, so it erases here
    // and the negation wraps the result. Doing it at this one site is what
    // lets `filter(Predicate.not(String::isEmpty))` and
    // `removeIf(Predicate.not(s -> s.isEmpty()))` know their element type —
    // reading the target type from the enclosing expression cannot, because
    // an argument position supplies no `Predicate<T>` to read.
    if let Expr::Call {
        receiver: Some(owner),
        method: name,
        args,
        span,
        ..
    } = lambda
        && name == "not"
        && args.len() == 1
        && matches!(owner.as_ref(), Expr::Name { path, .. }
            if path.last().is_some_and(|n| n == "Predicate"))
    {
        let span = *span;
        if matches!(args[0], Expr::MethodRef { .. }) {
            let synth = Sam {
                method: method.to_owned(),
                params: elem_types.to_vec(),
                ret: ret.clone(),
            };
            args[0] = method_ref_to_lambda(&args[0], &synth, ctx);
        }
        let inner = if matches!(args[0], Expr::Lambda { .. }) {
            build_erased_lambda(
                &mut args[0],
                interface,
                method,
                ret,
                elem_types,
                result_type,
                ctx,
            )
        } else {
            // Already a predicate VALUE (a variable, a field, another `not`):
            // nothing to erase, only to wrap.
            desugar_expr(&mut args[0], None, ctx);
            args[0].clone()
        };
        return Expr::NewObject {
            class: String::from("__Negate"),
            type_args: Vec::new(),
            args: vec![inner],
            outer: None,
            span,
        };
    }
    let Expr::Lambda { params, body, span } = lambda else {
        // Every caller is SUPPOSED to check, and one did not: `Stream
        // .generate(null)` reached here and PANICKED the compiler, which is
        // the one answer a program may never get — and it took every other
        // diagnostic in the file with it, so a program with that call and two
        // ordinary mistakes reported nothing at all. An argument that is not a
        // lambda is already a VALUE (a variable, a field, `null`): there is
        // nothing to erase, only to walk, which is what the `Predicate.not`
        // arm above does with one. Codegen judges whether the value fits.
        desugar_expr(lambda, None, ctx);
        return lambda.clone();
    };
    // A WILDCARD element reads out as its bound, or as `Object` — which is
    // what a JDK gives the lambda too. Left as the wildcard, the synthesized
    // `(E) __caturraArg0` declared a local of a type nothing resolves, and
    // `iteratorOfWildcard.forEachRemaining(v -> …)` was refused as "a
    // functional interface parameterized on a method's own type variable".
    let readable: Vec<TypeRef> = elem_types.iter().cloned().map(readable_elem).collect();
    let elem_types = &readable[..];
    let span = *span;
    *ctx.counter += 1;
    // One-shot: the prefix applies to the class being built now, and reverts
    // so nested lambdas inside its body are named as lambdas.
    let prefix = std::mem::replace(&mut ctx.class_prefix, crate::LAMBDA_CLASS_PREFIX);
    let name = format!("{prefix}{}", ctx.counter);

    // The lambda's arity must match the SAM's. Zipping them silently dropped
    // the extra parameter, so `Function<String,Integer> f = (a, b) -> 1;`
    // compiled here and is a compile error on a real JDK.
    if params.len() != elem_types.len() {
        ctx.diags.push(crate::diagnostics::Diagnostic::error(
            ctx.path,
            String::from("incompatible types: incompatible parameter types in lambda expression"),
            span,
        ));
    }

    let object = || TypeRef::Named(String::from("Object"));
    let erased: Vec<Param> = (0..elem_types.len())
        .map(|i| Param {
            ty: object(),
            name: format!("__caturraArg{i}"),
            is_varargs: false,
            is_final: false,
        })
        .collect();

    // `E e = (E) __caturraArg0;` for each parameter.
    let unwrap = |declared: &TypeRef, from: String, to: &str| Stmt::LocalDecl {
        ty: declared.clone(),
        is_final: false,
        declarators: vec![crate::ast::LocalDeclarator {
            name: to.to_owned(),
            init: Some(Expr::Cast {
                ty: declared.clone(),
                operand: Box::new(Expr::Name {
                    path: vec![from],
                    span,
                }),
                span,
            }),
            span,
            extra_dims: 0,
        }],
        span,
    };
    // ZIPPED, not indexed. The arity check above reports a lambda that does
    // not match its SAM, and then this ran anyway: for a lambda with FEWER
    // parameters than the interface takes (`stream.max(x -> x)`, where a
    // comparator takes two) it indexed past the end and PANICKED the
    // compiler. Binding the parameters that exist builds a class nothing will
    // run — the program is already refused — and reports one mistake instead
    // of a crash.
    let mut method_body: Vec<Stmt> = elem_types
        .iter()
        .zip(params.iter())
        .enumerate()
        .map(|(i, (ty, param))| unwrap(ty, format!("__caturraArg{i}"), &param.name))
        .collect();

    // The lambda's OWN parameters are in scope for its body. Without them a
    // nested lambda whose receiver is one of them —
    // `grid.forEach(row -> row.forEach(v -> …))` — could not find the
    // receiver's element type, and the inner lambda was refused for having no
    // functional-interface position, in a program javac compiles.
    ctx.scope.push(
        params
            .iter()
            .zip(elem_types)
            .map(|(param, ty)| (param.name.clone(), ty.clone()))
            .collect(),
    );
    let is_void = matches!(ret, TypeRef::Void);
    match std::mem::replace(body, LambdaBody::Block(Vec::new())) {
        LambdaBody::Expr(mut e) => {
            // The body is desugared against the lambda's own RESULT type
            // when there is one, which is what lets a lambda RETURN a lambda:
            // `x -> y -> x + y` has `Function<Integer, Integer>` as its
            // result, and the inner lambda needs that as its target. With no
            // expected type the inner one had no functional-interface
            // position and was refused, though a METHOD returning the same
            // lambda compiled -- its return type supplied what this did not.
            desugar_expr(&mut e, result_type, ctx);
            if is_void {
                method_body.push(Stmt::Expr(*e));
            } else if let Some(declared) = result_type {
                // `E __caturraResult = (expr); return __caturraResult;` — the
                // assignment type-checks the body against the element type.
                method_body.push(Stmt::LocalDecl {
                    ty: declared.clone(),
                    is_final: false,
                    declarators: vec![crate::ast::LocalDeclarator {
                        name: String::from("__caturraResult"),
                        init: Some(*e),
                        span,
                        extra_dims: 0,
                    }],
                    span,
                });
                method_body.push(Stmt::Return {
                    value: Some(Expr::Name {
                        path: vec![String::from("__caturraResult")],
                        span,
                    }),
                    span,
                });
            } else {
                method_body.push(Stmt::Return {
                    value: Some(*e),
                    span,
                });
            }
        }
        LambdaBody::Block(mut stmts) => {
            if let Some(declared) = result_type {
                target_type_returns(&mut stmts, declared, ctx);
            }
            for stmt in &mut stmts {
                desugar_stmt(stmt, ctx);
            }
            // For `replaceAll`, each `return e;` must convert to the element
            // type — the erased `Object` return would otherwise accept any
            // reference, more permissive than javac.
            if let Some(declared) = result_type {
                coerce_returns(&mut stmts, declared, span);
            }
            method_body.extend(stmts);
        }
    }
    ctx.scope.pop();

    let mut decl = ClassDecl {
        name: name.clone(),
        outer_type_params: Vec::new(),
        is_public: false,
        is_nested: false,
        enclosing: None,
        trace_name: None,
        binary_name: None,
        superclass: None,
        supertype_args: Vec::new(),
        interfaces: vec![interface.to_owned()],
        is_abstract: false,
        is_final: false,
        is_interface: false,
        is_enum: false,
        is_anonymous: true,
        is_local: false,
        is_inner: false,
        type_params: Vec::new(),
        fields: Vec::new(),
        methods: vec![MethodDecl {
            name: method.to_owned(),
            is_static: false,
            is_public: true,
            is_private: false,
            is_final: false,
            is_constructor: false,
            is_abstract: false,
            type_params: Vec::new(),
            infer_return: None,
            declared_params: Vec::new(),
            type_var_sources: Vec::new(),
            return_type: ret.clone(),
            params: erased,
            body: method_body,
            annotations: Vec::new(),
            // What the lambda's body is ALLOWED to throw: whatever the
            // interface's own method declares. The bundled ones declare
            // nothing, so a checked exception escaping a `Runnable` body is
            // the error javac reports ("unreported exception"); a user
            // interface written `void run() throws IOException` permits it.
            throws: ctx
                .shapes
                .get(interface)
                .and_then(|shapes| shapes.iter().find(|shape| shape.name == method))
                .map(|shape| shape.throws.clone())
                .unwrap_or_default(),
            is_protected: false,
            span,
            pre_init: 0,
            declared_return: None,
        }],
        init_blocks: Vec::new(),
        nested: Vec::new(),
        span,
    };
    // What this lambda ANSWERS, recorded where the next pass can read it: the
    // element of `stream.map(f)` is the type of `f`'s body, and codegen sees
    // only the erased `Object` the synthesized method returns. A synthetic
    // static field carries the type across; nothing reads its value, and a
    // lambda whose body cannot be typed simply has no field — its stream keeps
    // the erased element it always had.
    if let Some(produces) = produced_type(&decl, ctx) {
        decl.fields.push(crate::ast::FieldDecl {
            name: String::from(PRODUCES_FIELD),
            ty: produces,
            is_static: true,
            is_private: false,
            is_final: false,
            init: None,
            order: 0,
            span,
        });
    }
    // Recorded for the numbering pass at the end of the class: this is where
    // the translation FINISHES, so an inner lambda is recorded before the one
    // that contains it, which is the order javac numbers them in.
    if prefix == crate::LAMBDA_CLASS_PREFIX {
        let seq = ctx.lambda_order.len();
        let (_, method) = ctx.frame_owner;
        ctx.lambda_order
            .push((ctx.member_start, seq, name.clone(), method.to_owned()));
    } else {
        // A method REFERENCE has no frame of its own: javac compiles it to an
        // `invokedynamic` that calls the target directly, so a trace goes
        // straight from the target to whoever ran the functional interface.
        decl.trace_name = Some(String::new());
    }
    ctx.new_classes.push(decl);

    Expr::NewObject {
        class: name,
        type_args: Vec::new(),
        args: Vec::new(),
        outer: None,
        span,
    }
}

/// Build the anonymous class for a lambda and the `new` expression that
/// replaces it. Also recurses into the lambda body first so nested
/// lambdas are handled.
/// Substitute a target type's written type arguments into a user functional
/// interface's SAM, per parameter POSITION.
///
/// The interface's own type parameters erased to indexed sentinels before this
/// pass ran, so a sentinel's index picks the argument: for
/// `Mapper<String, Integer>`, `apply`'s parameter becomes `String` and its
/// return `Integer`. Without it the lambda's parameter stayed a bare type
/// variable, and `Mapper<String, Integer> m = s -> s.length()` was "cannot
/// find symbol: method `length()` in class `java/lang/Object`".
///
/// Answers `None` for a RAW target (no arguments written), which keeps its
/// erased treatment.
fn specialize_sam(sam: &Sam, target: &TypeRef) -> Option<Sam> {
    let TypeRef::Generic { args, .. } = target else {
        return None;
    };
    let specialized = Sam {
        method: sam.method.clone(),
        params: sam
            .params
            .iter()
            .map(|ty| substitute_sentinels(ty, args))
            .collect(),
        ret: substitute_sentinels(&sam.ret, args),
    };
    (specialized.params != sam.params || specialized.ret != sam.ret).then_some(specialized)
}

/// Replace each indexed type-variable sentinel in `ty` with the type argument
/// written at that position. An index the target does not carry is left as it
/// is — a raw or mis-arity use keeps the erased type rather than inventing one.
fn substitute_sentinels(ty: &TypeRef, args: &[TypeRef]) -> TypeRef {
    match ty {
        TypeRef::Named(name) => match crate::parser::typevar_index(name) {
            Some(index) => args
                .get(usize::from(index))
                .cloned()
                .unwrap_or_else(|| ty.clone()),
            None => ty.clone(),
        },
        TypeRef::Array(inner) => TypeRef::Array(Box::new(substitute_sentinels(inner, args))),
        TypeRef::Generic { base, args: inner } => TypeRef::Generic {
            base: base.clone(),
            args: inner
                .iter()
                .map(|a| substitute_sentinels(a, args))
                .collect(),
        },
        other => other.clone(),
    }
}

#[allow(clippy::too_many_lines)] // the specialized and erased shapes, in one place
fn build_lambda_class(
    lambda: &mut Expr,
    interface: &str,
    sam: &Sam,
    // The same SAM with the target's type arguments substituted in, when the
    // target wrote any. The METHOD keeps the erased signature — it has to, or
    // it would not override the interface's — and each type-variable parameter
    // is cast to its real type at the top of the body instead.
    specialized: Option<&Sam>,
    ctx: &mut Ctx,
) -> Expr {
    let Expr::Lambda { params, body, span } = lambda else {
        unreachable!("guarded by caller");
    };
    let span = *span;
    *ctx.counter += 1;
    // One-shot: the prefix applies to the class being built now, and reverts
    // so nested lambdas inside its body are named as lambdas.
    let prefix = std::mem::replace(&mut ctx.class_prefix, crate::LAMBDA_CLASS_PREFIX);
    let name = format!("{prefix}{}", ctx.counter);

    // Which parameters need the cast: the ones the interface declared as a
    // type variable and the target gave a real type. Those take a synthetic
    // name on the method, so the lambda's own name can be the cast local.
    let casts: Vec<Option<TypeRef>> = sam
        .params
        .iter()
        .enumerate()
        .map(|(i, erased)| {
            let real = specialized?.params.get(i)?;
            (real != erased).then(|| real.clone())
        })
        .collect();

    // The synthesized method takes the SAM's ERASED parameter types (that is
    // what makes it an override) with the lambda's parameter names, except
    // where a cast will introduce the name itself.
    let method_params: Vec<Param> = params
        .iter()
        .zip(&sam.params)
        .enumerate()
        .map(|(i, (p, ty))| Param {
            ty: ty.clone(),
            name: if casts[i].is_some() {
                format!("__caturraArg{i}")
            } else {
                p.name.clone()
            },
            is_varargs: false,
            is_final: false,
        })
        .collect();

    // `String s = (String) __caturraArg0;` for each specialized parameter.
    let mut prelude: Vec<Stmt> = Vec::new();
    for (i, declared) in casts.iter().enumerate() {
        let Some(declared) = declared else { continue };
        prelude.push(Stmt::LocalDecl {
            ty: declared.clone(),
            is_final: false,
            declarators: vec![crate::ast::LocalDeclarator {
                name: params[i].name.clone(),
                init: Some(Expr::Cast {
                    ty: declared.clone(),
                    operand: Box::new(Expr::Name {
                        path: vec![format!("__caturraArg{i}")],
                        span,
                    }),
                    span,
                }),
                span,
                extra_dims: 0,
            }],
            span,
        });
    }

    // A type-variable RETURN is checked against the target's argument the
    // same way `build_erased_lambda` does it: the erased `Object` return
    // would otherwise accept any reference, more permissive than javac.
    let result_type = specialized
        .filter(|spec| spec.ret != sam.ret)
        .map(|spec| spec.ret.clone());

    // The lambda's OWN parameters are in scope for its body — the same rule
    // the erased builder beside this one applies, and it has to be written
    // here too because a USER functional interface takes this path. Without
    // it a lambda whose parameter is a collection could not have a lambda
    // inside it: `Box<List<String>> f = l -> l.stream().map(v -> …)` was
    // refused for the INNER lambda having no functional-interface position.
    ctx.scope.push(
        params
            .iter()
            .enumerate()
            .filter_map(|(i, param)| {
                // The SPECIALIZED type when the target pinned one (what the
                // prelude casts to), else the SAM's own.
                let declared = casts
                    .get(i)
                    .cloned()
                    .flatten()
                    .or_else(|| sam.params.get(i).cloned())?;
                Some((param.name.clone(), declared))
            })
            .collect(),
    );
    // The body: an expression lambda becomes `return e;` (or `e;` when
    // the SAM is void); a block lambda's statements are used directly.
    let mut method_body = prelude;
    match std::mem::replace(body, LambdaBody::Block(Vec::new())) {
        LambdaBody::Expr(mut e) => {
            // The body is desugared against the lambda's own RESULT type
            // when there is one, which is what lets a lambda RETURN a lambda:
            // `x -> y -> x + y` has `Function<Integer, Integer>` as its
            // result, and the inner lambda needs that as its target. With no
            // expected type the inner one had no functional-interface
            // position and was refused, though a METHOD returning the same
            // lambda compiled -- its return type supplied what this did not.
            desugar_expr(&mut e, result_type.as_ref(), ctx);
            if matches!(sam.ret, TypeRef::Void) {
                method_body.push(Stmt::Expr(*e));
            } else if let Some(declared) = &result_type {
                method_body.push(Stmt::LocalDecl {
                    ty: declared.clone(),
                    is_final: false,
                    declarators: vec![crate::ast::LocalDeclarator {
                        name: String::from("__caturraResult"),
                        init: Some(*e),
                        span,
                        extra_dims: 0,
                    }],
                    span,
                });
                method_body.push(Stmt::Return {
                    value: Some(Expr::Name {
                        path: vec![String::from("__caturraResult")],
                        span,
                    }),
                    span,
                });
            } else {
                method_body.push(Stmt::Return {
                    value: Some(*e),
                    span,
                });
            }
        }
        LambdaBody::Block(mut stmts) => {
            if let Some(declared) = &result_type {
                target_type_returns(&mut stmts, declared, ctx);
            }
            for s in &mut stmts {
                desugar_stmt(s, ctx);
            }
            if let Some(declared) = &result_type {
                coerce_returns(&mut stmts, declared, span);
            }
            method_body.extend(stmts);
        }
    }
    ctx.scope.pop();
    let method_body = method_body;

    let method = MethodDecl {
        name: sam.method.clone(),
        is_static: false,
        is_public: true,
        is_private: false,
        is_final: false,
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        infer_return: None,
        declared_params: Vec::new(),
        type_var_sources: Vec::new(),
        return_type: sam.ret.clone(),
        params: method_params,
        body: method_body,
        annotations: Vec::new(),
        // What the body may throw is what the interface's own method declares
        // — `void go() throws IOException` permits an IOException to escape,
        // and an interface that declares nothing does not.
        throws: ctx
            .shapes
            .get(interface)
            .and_then(|shapes| shapes.iter().find(|shape| shape.name == sam.method))
            .map(|shape| shape.throws.clone())
            .unwrap_or_default(),
        is_protected: false,
        span,
        pre_init: 0,
        declared_return: None,
    };

    // The target is known to be a functional interface (it is in `sams`),
    // so record it in the `implements` slot directly. Parking it in
    // `superclass` for codegen to reclassify relies on the interface's
    // `is_interface` flag already being set, which fails when the interface
    // lives in a later compilation unit (e.g. the injected `ActionListener`)
    // than the synthesized lambda class.
    // Recorded for the numbering pass, exactly as the erased builder does:
    // the two shapes of lambda class share javac's one sequence.
    let trace_name = if prefix == crate::LAMBDA_CLASS_PREFIX {
        let seq = ctx.lambda_order.len();
        let (_, method_part) = ctx.frame_owner;
        ctx.lambda_order
            .push((ctx.member_start, seq, name.clone(), method_part.to_owned()));
        None
    } else {
        Some(String::new())
    };
    ctx.new_classes.push(ClassDecl {
        name: name.clone(),
        outer_type_params: Vec::new(),
        is_public: false,
        is_nested: false,
        enclosing: None,
        trace_name,
        binary_name: None,
        superclass: None,
        supertype_args: Vec::new(),
        interfaces: vec![interface.to_owned()],
        is_abstract: false,
        is_final: false,
        is_interface: false,
        is_enum: false,
        is_anonymous: true,
        is_local: false,
        is_inner: false,
        type_params: Vec::new(),
        fields: Vec::new(),
        methods: vec![method],
        init_blocks: Vec::new(),
        nested: Vec::new(),
        span,
    });

    Expr::NewObject {
        class: name,
        type_args: Vec::new(),
        args: Vec::new(),
        outer: None,
        span,
    }
}

#[cfg(test)]
mod tests {
    use caturra_classfile::ClassFile;

    /// The `SourceFile` a class advertises (how the debugger keys it).
    fn source_file_of(cf: &ClassFile) -> Option<String> {
        cf.attributes
            .iter()
            .find(|a| {
                cf.constant_pool.get_utf8(a.name_index)
                    == Some(caturra_classfile::debug::SOURCE_FILE_ATTRIBUTE)
            })
            .and_then(|a| caturra_classfile::debug::decode_source_file(&a.info))
            .and_then(|index| cf.constant_pool.get_utf8(index))
            .map(str::to_owned)
    }

    #[test]
    fn lambda_in_a_non_first_file_keeps_its_own_source_file() {
        // The lambda lives in Helper.java (the second unit). Its synthesized
        // class must carry Helper.java's SourceFile so its line numbers — and
        // thus its breakpoints and stack traces — point at the right file.
        let compilation = crate::compile(&[
            crate::SourceFile {
                path: "Main.java".to_owned(),
                text: "public class Main { public static void main(String[] a) {} }".to_owned(),
            },
            crate::SourceFile {
                path: "Helper.java".to_owned(),
                text: "interface Task { void go(); } public class Helper { void run() { Task t = () -> {}; t.go(); } }"
                    .to_owned(),
            },
        ]);
        assert!(
            compilation.success(),
            "compile failed: {:?}",
            compilation.diagnostics
        );
        let lambda = compilation
            .classes
            .iter()
            .find(|c| crate::is_lambda_class(&c.binary_name))
            .expect("a synthesized lambda class");
        assert_eq!(
            source_file_of(&lambda.class_file).as_deref(),
            Some("Helper.java"),
        );
    }
}
