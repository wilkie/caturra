//! Bridge methods for generic overrides (JVMS §4.6, JLS §8.4.8.3).
//!
//! `class SBox extends Box<String>` overriding `void set(T)` with
//! `void set(String)` does NOT override it on the JVM: erasure gives the
//! superclass `set(Object)` and the subclass `set(String)`, two different
//! descriptors, so a call through a `Box` reference reaches the SUPERCLASS.
//! javac hides this by synthesizing a *bridge* — a `set(Object)` in the
//! subclass that casts its argument and delegates to `set(String)`.
//!
//! Without it, `Box<String> b = new SBox(); b.set("hi")` printed `Box.set hi`
//! where a real JVM prints `SBox.set hi`: a silent wrong answer, and the reason
//! this pass exists rather than merely letting the assignment compile.
//!
//! Covariant returns need the same treatment (`String f()` overriding
//! `Object f()`), and get it here too.

use std::collections::HashMap;

use crate::ast::{ClassDecl, CompilationUnit, Expr, MethodDecl, Param, Stmt, TypeRef};
use crate::diagnostics::SourceSpan;

/// Add the bridge methods every generic override needs.
pub fn add_bridge_methods(units: &mut [(String, CompilationUnit)]) {
    // Every class by name, for walking supertype chains.
    let classes: HashMap<String, ClassDecl> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .map(|c| (c.name.clone(), c.clone()))
        .collect();

    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            let bridges = bridges_for(class, &classes);
            class.methods.extend(bridges);
        }
    }
}

/// The bridges `class` needs: one per method that overrides an inherited
/// method whose ERASED signature differs.
fn bridges_for(class: &ClassDecl, classes: &HashMap<String, ClassDecl>) -> Vec<MethodDecl> {
    let mut bridges: Vec<MethodDecl> = Vec::new();
    for method in &class.methods {
        if method.is_static || method.is_constructor || method.is_abstract {
            continue;
        }
        // A bundled FUNCTIONAL interface (`IntBinaryOperator` is the erased
        // `__IntBinaryOperator { int applyAsInt(Object, Object); }`) is matched
        // by the JDK's own declaration, which the erased one stands for: its
        // parameters may be primitives (`applyAsInt(int, int)`) or a narrowed
        // type argument (`ObjIntConsumer<String>.accept(String, int)`).
        let functional = inherited_signature(class, &method.name, method.params.len(), classes)
            .is_none()
            .then(|| bundled_functional_signature(class, method, classes))
            .flatten();
        let Some(inherited) = functional.clone().or_else(|| {
            inherited_signature(class, &method.name, method.params.len(), classes)
                .or_else(|| library_erased_signature(class, method, classes))
        }) else {
            continue;
        };
        // Same erasure already: the override works without help.
        if same_erasure(&inherited, method) {
            continue;
        }
        // A difference in RETURN type alone cannot be bridged from source: the
        // bridge would have the same name and arity as the method it delegates
        // to, so the call in its body would resolve back to itself. javac emits
        // such a bridge directly as bytecode; caturra relies instead on its
        // dispatch matching by name and arity, which reaches the override.
        if erased_params(&inherited) == erased_params(method) {
            continue;
        }
        // The subclass's parameters must each be assignable FROM the
        // inherited ones — otherwise this is an overload, not an override,
        // and bridging it would hijack a legitimately different method.
        if functional.is_none() && !is_override_of(&inherited, method) {
            continue;
        }
        // Do not add a bridge whose signature the class already declares.
        let already_declared = class
            .methods
            .iter()
            .any(|m| m.name == method.name && erased_params(m) == erased_params(&inherited))
            || bridges
                .iter()
                .any(|m| m.name == method.name && erased_params(m) == erased_params(&inherited));
        if already_declared {
            continue;
        }
        bridges.push(build_bridge(method, &inherited, class.span));
    }
    bridges
}

/// The erased signature a LIBRARY generic interface declares, for a method
/// that implements one. caturra synthesizes `Comparable` and `Comparator`
/// straight into the method table rather than parsing them, so the walk below
/// cannot find their signatures at all — and a `Shape extends
/// Comparable<Shape>` whose `compareTo(Shape)` is a DEFAULT got no bridge, so
/// nothing on the class answered `compareTo(Object)` and sorting one threw
/// `ClassCastException: … cannot be cast to java.lang.Comparable`.
///
/// javac puts that bridge on the INTERFACE (an interface may carry one since
/// Java 8), which is where this one goes too.
fn library_erased_signature(
    class: &ClassDecl,
    method: &MethodDecl,
    classes: &HashMap<String, ClassDecl>,
) -> Option<MethodDecl> {
    let params = match (method.name.as_str(), method.params.len()) {
        ("compareTo", 1) => 1,
        ("compare", 2) => 2,
        _ => return None,
    };
    let wanted = if params == 1 {
        "Comparable"
    } else {
        "Comparator"
    };
    if !implements_library_interface(class, wanted, classes) {
        return None;
    }
    // The interface's own declaration is `int compareTo(T)`, and it is the
    // TYPE VARIABLE that makes this an override rather than an overload — the
    // same sentinel a parsed generic supertype would carry, since plain
    // `Object` deliberately does not count (it would hijack ordinary
    // overloads).
    let object = |index: usize| Param {
        ty: TypeRef::Named(crate::parser::typevar_sentinel(0)),
        name: format!("__erased{index}"),
        is_varargs: false,
        is_final: false,
    };
    let mut erased = method.clone();
    erased.params = (0..params).map(object).collect();
    erased.return_type = TypeRef::Int;
    erased.body = Vec::new();
    Some(erased)
}

/// The erased method of a bundled FUNCTIONAL interface that `class` implements
/// — through its own `implements` or a superclass's (an `enum Op implements
/// IntBinaryOperator` whose constants each declare `applyAsInt`) — with the
/// name and arity of `method`. The program wrote the JDK name; the bundled
/// declaration is `__` plus it.
fn bundled_functional_signature(
    class: &ClassDecl,
    method: &MethodDecl,
    classes: &HashMap<String, ClassDecl>,
) -> Option<MethodDecl> {
    let mut interfaces: Vec<String> = class.interfaces.clone();
    let mut current = class.superclass.clone();
    for _ in 0..=classes.len() {
        let Some(name) = current else {
            break;
        };
        let Some(parent) = classes.get(&name) else {
            // Not a program class: an ANONYMOUS class records the one type it
            // names as its superclass until codegen learns whether that is an
            // interface, so `new Comparator<String>() {…}` is found here.
            interfaces.push(name);
            break;
        };
        interfaces.extend(parent.interfaces.iter().cloned());
        current = parent.superclass.clone();
    }
    let mut seen: Vec<String> = Vec::new();
    while let Some(name) = interfaces.pop() {
        let simple = name.split('<').next().unwrap_or(&name);
        let simple = simple.rsplit('.').next().unwrap_or(simple).to_owned();
        if seen.contains(&simple) || seen.len() > classes.len() * 2 + 2 {
            continue;
        }
        seen.push(simple.clone());
        if let Some(own) = classes.get(&simple) {
            interfaces.extend(own.interfaces.iter().cloned());
            continue;
        }
        let Some(bundled) = classes.get(&format!("__{simple}")) else {
            continue;
        };
        if !bundled.is_interface {
            continue;
        }
        if let Some(found) = bundled.methods.iter().find(|m| {
            m.name == method.name
                && m.params.len() == method.params.len()
                && m.is_abstract
                && m.params
                    .iter()
                    .any(|p| matches!(&p.ty, TypeRef::Named(n) if n == "Object"))
        }) {
            return Some(found.clone());
        }
    }
    None
}

/// Whether `class` implements a library interface of this name, directly or
/// through the interfaces it extends. The type ARGUMENT does not matter: what
/// the bridge stands for is the erasure, which is `Object` either way.
fn implements_library_interface(
    class: &ClassDecl,
    wanted: &str,
    classes: &HashMap<String, ClassDecl>,
) -> bool {
    let mut queue: Vec<String> = class.interfaces.clone();
    // An anonymous class's one supertype is recorded as its superclass (see
    // `bundled_functional_signature`); a library name there may be the
    // interface.
    queue.extend(
        class
            .superclass
            .iter()
            .filter(|name| !classes.contains_key(name.as_str()))
            .cloned(),
    );
    let mut steps = 0usize;
    while let Some(name) = queue.pop() {
        steps += 1;
        if steps > classes.len() * 2 + 2 {
            return false;
        }
        let simple = name.split('<').next().unwrap_or(&name);
        let simple = simple.rsplit('.').next().unwrap_or(simple);
        if simple == wanted && !classes.contains_key(simple) {
            return true;
        }
        if let Some(parent) = classes.get(simple) {
            queue.extend(parent.interfaces.iter().cloned());
        }
    }
    false
}

/// The nearest inherited method of this name and arity: up the extends chain,
/// then through the INTERFACES those classes implement.
///
/// The interfaces were left out on the theory that their methods erase to the
/// same descriptor as the implementing class's — which is true only while the
/// interface is not generic. `class Named implements Sink<String>` declaring
/// `twice(String)` overrides `twice(T)`, whose erasure takes an `Object`, so
/// without a bridge the call through `Sink<String>` found no `twice(Object)`
/// on the class and ran the interface's DEFAULT instead: the override silently
/// did not happen. (The abstract methods of the same interface were reached
/// anyway, which is why only a defaulted one showed it.)
#[allow(clippy::assigning_clones)] // `current` is moved by the `while let`
fn inherited_signature(
    class: &ClassDecl,
    name: &str,
    arity: usize,
    classes: &HashMap<String, ClassDecl>,
) -> Option<MethodDecl> {
    let matches = |m: &&MethodDecl| {
        m.name == name && m.params.len() == arity && !m.is_static && !m.is_constructor
    };
    let mut interfaces: Vec<String> = class.interfaces.clone();
    let mut current = class.superclass.clone();
    let mut steps = 0usize;
    while let Some(super_name) = current {
        steps += 1;
        if steps > classes.len() + 1 {
            return None; // a cycle would be a compiler bug; do not hang
        }
        let Some(parent) = classes.get(&super_name) else {
            break;
        };
        if let Some(found) = parent.methods.iter().find(matches) {
            return Some(found.clone());
        }
        interfaces.extend(parent.interfaces.iter().cloned());
        current = parent.superclass.clone();
    }
    // The interfaces, and the interfaces they extend.
    let mut steps = 0usize;
    while let Some(name) = interfaces.pop() {
        steps += 1;
        if steps > classes.len() * 2 + 2 {
            break;
        }
        let Some(iface) = classes.get(&name) else {
            continue;
        };
        if let Some(found) = iface.methods.iter().find(matches) {
            return Some(found.clone());
        }
        interfaces.extend(iface.interfaces.iter().cloned());
    }
    None
}

fn erased_params(method: &MethodDecl) -> Vec<String> {
    method.params.iter().map(|p| type_key(&p.ty)).collect()
}

/// A type's identity after erasure, as a string — enough to compare
/// descriptors without resolving anything.
fn type_key(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Named(name) => name.clone(),
        TypeRef::Generic { base, .. } => base.clone(),
        TypeRef::Array(inner) => format!("{}[]", type_key(inner)),
        other => format!("{other:?}"),
    }
}

fn same_erasure(inherited: &MethodDecl, method: &MethodDecl) -> bool {
    erased_params(inherited) == erased_params(method)
        && type_key(&inherited.return_type) == type_key(&method.return_type)
}

/// Whether `method` overrides `inherited` rather than overloading it: every
/// parameter the subclass narrows must correspond to an `Object` (an erased
/// type variable) in the superclass.
///
/// Conservative on purpose. A parameter pair that differs any other way means
/// these are two distinct methods, and synthesizing a bridge would make the
/// subclass answer a call meant for the superclass.
fn is_override_of(inherited: &MethodDecl, method: &MethodDecl) -> bool {
    inherited
        .params
        .iter()
        .zip(&method.params)
        .all(|(from, to)| {
            let (from, to) = (type_key(&from.ty), type_key(&to.ty));
            from == to || is_erased_variable(&from)
        })
}

/// Whether an erased parameter type came from a TYPE VARIABLE.
///
/// ONLY the sentinel counts. Accepting plain `Object` as well — which an
/// earlier version did, to catch multi-parameter generic classes whose `T`
/// erases to `Object` — hijacked ordinary OVERLOADS: given
/// `class A { m(Object) }` and `class B extends A { m(String) }`, which are
/// two distinct methods, it synthesized a `m(Object)` bridge in B, and a call
/// through an `A` reference then reached `B.m(String)` where a JDK reaches
/// `A.m(Object)`. Erasure loses the difference between a `T` that became
/// `Object` and an `Object` written by hand, so the only safe reading is the
/// unambiguous one.
///
/// The cost is that a generic class with SEVERAL type parameters gets no
/// bridge (its `T` erases to `Object`, not the sentinel), leaving the
/// pre-existing missing-bridge gap. Missing a bridge loses a dispatch; adding
/// a wrong one steals a call that was never generic. The second is worse.
fn is_erased_variable(key: &str) -> bool {
    crate::parser::typevar_index(key).is_some()
}

/// `void set(Object t) { set((String) t); }` — cast each narrowed parameter
/// and delegate to the real override.
/// javac positions a bridge at its CLASS's declaration — for an anonymous
/// class the `new` that declares it — which is the line the bridge's frame
/// names in a trace through it.
fn build_bridge(method: &MethodDecl, inherited: &MethodDecl, class_span: SourceSpan) -> MethodDecl {
    let zero = class_span;
    let params: Vec<Param> = inherited
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| Param {
            ty: p.ty.clone(),
            name: format!("__bridge{i}"),
            is_varargs: false,
            is_final: false,
        })
        .collect();
    // Each argument is cast to the overriding method's parameter type, which
    // is also what makes the call resolve to that method rather than back to
    // this bridge: the cast type is strictly more specific.
    let args: Vec<Expr> = method
        .params
        .iter()
        .enumerate()
        .map(|(i, p)| Expr::Cast {
            ty: p.ty.clone(),
            operand: Box::new(Expr::Name {
                path: vec![format!("__bridge{i}")],
                span: zero,
            }),
            span: zero,
        })
        .collect();
    let call = Expr::Call {
        receiver: Some(Box::new(Expr::This { span: zero })),
        method: method.name.clone(),
        args,
        span: zero,
        type_args: Vec::new(),
        paren_line: 0,
    };
    let body = if matches!(inherited.return_type, TypeRef::Void) {
        vec![Stmt::Expr(call)]
    } else {
        vec![Stmt::Return {
            value: Some(call),
            span: zero,
        }]
    };
    MethodDecl {
        name: method.name.clone(),
        is_static: false,
        is_public: method.is_public,
        is_private: false,
        is_final: false,
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        infer_return: None,
        declared_params: Vec::new(),
        type_var_sources: Vec::new(),
        // The bridge keeps the INHERITED return type, so a call through the
        // supertype sees the descriptor it expects.
        return_type: inherited.return_type.clone(),
        params,
        body,
        annotations: Vec::new(),
        throws: Vec::new(),
        is_protected: false,
        span: zero,
        pre_init: 0,
        declared_return: None,
        body_end: None,
    }
}
