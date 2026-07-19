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
        let Some(inherited) =
            inherited_signature(class, &method.name, method.params.len(), classes)
        else {
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
        if !is_override_of(&inherited, method) {
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
        bridges.push(build_bridge(method, &inherited));
    }
    bridges
}

/// The nearest inherited method of this name and arity, walking the extends
/// chain. Interfaces are not walked: their methods erase to the same
/// descriptor as the implementing class's unless the interface is generic,
/// and an interface's own generic method is reached through the class chain
/// that implements it.
#[allow(clippy::assigning_clones)] // `current` is moved by the `while let`
fn inherited_signature(
    class: &ClassDecl,
    name: &str,
    arity: usize,
    classes: &HashMap<String, ClassDecl>,
) -> Option<MethodDecl> {
    let mut current = class.superclass.clone();
    let mut steps = 0usize;
    while let Some(super_name) = current {
        steps += 1;
        if steps > classes.len() + 1 {
            return None; // a cycle would be a compiler bug; do not hang
        }
        let parent = classes.get(&super_name)?;
        if let Some(found) = parent.methods.iter().find(|m| {
            m.name == name && m.params.len() == arity && !m.is_static && !m.is_constructor
        }) {
            return Some(found.clone());
        }
        current = parent.superclass.clone();
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
    key == crate::parser::TYPEVAR_SENTINEL
}

/// `void set(Object t) { set((String) t); }` — cast each narrowed parameter
/// and delegate to the real override.
fn build_bridge(method: &MethodDecl, inherited: &MethodDecl) -> MethodDecl {
    let zero = method.span;
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
        // The bridge keeps the INHERITED return type, so a call through the
        // supertype sees the descriptor it expects.
        return_type: inherited.return_type.clone(),
        params,
        body,
        annotations: Vec::new(),
        span: zero,
    }
}
