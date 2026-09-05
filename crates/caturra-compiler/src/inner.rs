//! Inner (non-static nested) class binding.
//!
//! An inner class is bound to an enclosing instance. This pass gives each inner
//! class a synthetic `__caturraOuter` field of its enclosing type and threads
//! that instance through its constructors as a leading parameter — synthesizing
//! a constructor when the class declares none. The enclosing instance is then
//! supplied at every `new Inner(...)` site by codegen (the qualifier of
//! `outer.new Inner()`, the current `this`, or a sibling's `this.__caturraOuter`),
//! and enclosing-member access reuses codegen's existing `__caturraOuter`
//! routing — the same field an enclosing-instance-capturing lambda gets.

use std::collections::HashMap;

use crate::ast::{
    AssignTarget, ClassDecl, CompilationUnit, Expr, FieldDecl, MethodDecl, Param, Stmt, TypeRef,
};
use crate::capture::OUTER_FIELD;

pub fn bind_inner_classes(units: &mut [(String, CompilationUnit)]) {
    // Inner class name -> its enclosing class name.
    let enclosing: HashMap<String, String> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .filter(|c| c.is_inner)
        .filter_map(|c| c.enclosing.clone().map(|e| (c.name.clone(), e)))
        .collect();
    if enclosing.is_empty() {
        return;
    }
    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            // A class whose SUPERCLASS is an inner class must pass an enclosing
            // instance to it — `class B extends A` inside one outer builds an
            // `A` too. Its own `__caturraOuter` is that instance (they share the
            // outer), and it is in scope as the leading constructor parameter
            // this pass is about to add.
            let super_is_inner = class
                .superclass
                .as_deref()
                .is_some_and(|name| enclosing.contains_key(name));
            if class.is_inner
                && let Some(outer) = enclosing.get(&class.name)
            {
                add_outer_binding(class, outer);
            }
            if super_is_inner && class.is_inner {
                thread_outer_to_super(class);
            }
        }
    }
}

/// Give every explicit `super(...)` the enclosing instance as its leading
/// argument. The implicit one is synthesized by codegen, which is told the
/// same thing by the superclass's signature.
fn thread_outer_to_super(class: &mut ClassDecl) {
    let zero = class.span;
    for ctor in class.methods.iter_mut().filter(|m| m.is_constructor) {
        let outer_arg = |span| Expr::Name {
            path: vec![String::from(OUTER_FIELD)],
            span,
        };
        match ctor.body.first_mut() {
            Some(Stmt::SuperCall { args, span }) => {
                let span = *span;
                args.insert(0, outer_arg(span));
            }
            // Delegating to another of this class's constructors: that one
            // passes the instance on.
            Some(Stmt::ThisCall { .. }) => {}
            // No explicit chain call: codegen would synthesize a `super()`
            // with no arguments, which the inner superclass cannot accept.
            _ => ctor.body.insert(
                0,
                Stmt::SuperCall {
                    args: vec![outer_arg(zero)],
                    span: zero,
                },
            ),
        }
    }
}

/// Add the `__caturraOuter` field of the enclosing type and thread it through
/// the class's constructors as a leading parameter, stored right after any
/// leading `super(...)`/`this(...)` so the body can already read it.
fn add_outer_binding(class: &mut ClassDecl, outer: &str) {
    let zero = class.span;
    let outer_ty = TypeRef::Named(outer.to_owned());
    class.fields.push(FieldDecl {
        name: String::from(OUTER_FIELD),
        ty: outer_ty.clone(),
        is_static: false,
        is_private: true,
        is_public: false,
        is_protected: false,
        is_enum_constant: false,
        is_final: true,
        init: None,
        order: 0,
        span: zero,
    });
    let param = || Param {
        ty: outer_ty.clone(),
        name: String::from(OUTER_FIELD),
        is_varargs: false,
        is_final: false,
    };
    let store = || Stmt::Assign {
        target: AssignTarget::Field {
            object: Box::new(Expr::This { span: zero }),
            name: String::from(OUTER_FIELD),
        },
        op: None,
        value: Expr::Name {
            path: vec![String::from(OUTER_FIELD)],
            span: zero,
        },
        span: zero,
    };

    let has_ctor = class.methods.iter().any(|m| m.is_constructor);
    if !has_ctor {
        // codegen prepends the implicit `super()`.
        class.methods.push(MethodDecl {
            name: class.name.clone(),
            is_static: false,
            is_public: false,
            is_private: false,
            is_final: false,
            is_constructor: true,
            is_abstract: false,
            type_params: Vec::new(),
            infer_return: None,
            declared_params: Vec::new(),
            type_var_sources: Vec::new(),
            return_type: TypeRef::Void,
            params: vec![param()],
            body: vec![store()],
            annotations: Vec::new(),
            throws: Vec::new(),
            is_protected: false,
            span: zero,
            pre_init: 0,
            declared_return: None,
        });
        return;
    }
    for ctor in class.methods.iter_mut().filter(|m| m.is_constructor) {
        ctor.params.insert(0, param());
        // A constructor that DELEGATES (`Inner() { this(99); }`) threads the
        // enclosing instance through to the one it calls, and does not store it
        // itself — the delegate does. Without the extra argument the delegation
        // matched the constructor's OWN new signature, which the recursion
        // check then reported as a recursive constructor invocation (the same
        // trap the capture pass records for its own captured values).
        if let Some(Stmt::ThisCall { args, span }) = ctor.body.first_mut() {
            let span = *span;
            args.insert(
                0,
                Expr::Name {
                    path: vec![String::from(OUTER_FIELD)],
                    span,
                },
            );
            continue;
        }
        let after = usize::from(matches!(ctor.body.first(), Some(Stmt::SuperCall { .. })));
        ctor.body.insert(after, store());
    }
}
