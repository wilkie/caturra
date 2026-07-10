//! Closure-capture resolution for anonymous classes.
//!
//! An anonymous class body may reference effectively-final local
//! variables of the enclosing method (`int t = 5; new P() { boolean
//! test(int x) { return x > t; } }`). Java captures those by copying
//! them into synthetic fields set through a synthesized constructor.
//!
//! This pass runs after parsing (so every anonymous class is a hoisted
//! top-level [`ClassDecl`]) and before the method table is built (so the
//! synthesized fields and constructor are visible to type resolution).
//! It:
//!
//! 1. Walks every method body tracking the in-scope local variables and
//!    their declared types, and at each `new Anon$N(...)` computes which
//!    of the anonymous class's free names are enclosing locals — the
//!    captures.
//! 2. Adds a synthetic field per capture and a constructor that stores
//!    them. A bare reference in the body then resolves to the implicit
//!    `this`-field, so the body needs no rewriting.
//! 3. Passes the captured locals as arguments at the `new` site.

use std::collections::{HashMap, HashSet};

use crate::ast::{
    ClassDecl, CompilationUnit, Expr, FieldDecl, LambdaBody, MethodDecl, Param, Stmt, TypeRef,
};
use crate::diagnostics::SourceSpan;

/// Resolve captures for all anonymous classes in the compilation.
pub fn resolve_captures(
    units: &mut [(String, CompilationUnit)],
) -> Vec<crate::diagnostics::Diagnostic> {
    // Clone anonymous-class bodies for free-name analysis (read-only).
    let anon_bodies: HashMap<String, ClassDecl> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .filter(|c| c.is_anonymous)
        .map(|c| (c.name.clone(), c.clone()))
        .collect();
    if anon_bodies.is_empty() {
        return Vec::new();
    }

    // Phase 1: capture set per anonymous class (sorted for determinism), plus
    // the type of each super-constructor argument at its `new` site.
    let mut found = Found::default();
    let mut diagnostics: Vec<crate::diagnostics::Diagnostic> = Vec::new();
    for (path, unit) in units.iter() {
        for class in &unit.classes {
            for method in &class.methods {
                let mut scope: Vec<HashMap<String, TypeRef>> = vec![
                    method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                ];
                // A parameter arrives initialized, so any assignment to it
                // costs it its effective finality.
                let mut mutations = Mutations::default();
                mutations
                    .initialized
                    .extend(method.params.iter().map(|p| p.name.clone()));
                mutations_in_stmts(&method.body, &mut mutations);
                find_in_stmts(
                    &method.body,
                    &mut scope,
                    &anon_bodies,
                    &mut found,
                    &class.name,
                    &mutations,
                );
            }
            for block in &class.init_blocks {
                let mut scope: Vec<HashMap<String, TypeRef>> = vec![HashMap::new()];
                let mut mutations = Mutations::default();
                mutations_in_stmts(&block.body, &mut mutations);
                find_in_stmts(
                    &block.body,
                    &mut scope,
                    &anon_bodies,
                    &mut found,
                    &class.name,
                    &mutations,
                );
            }
        }
        diagnostics.extend(
            found
                .diagnostics
                .drain(..)
                .map(|(message, span)| crate::diagnostics::Diagnostic::error(path, message, span)),
        );
    }
    let mut captures = found.captures;
    let super_args = found.super_args;
    let owners = found.owner;

    inject_outer_captures(units, &anon_bodies, &owners, &mut captures);

    // Phase 2a: synthesize the fields and constructor on each anon class that
    // captures locals or forwards args to super.
    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            let caps = captures.get(&class.name).cloned().unwrap_or_default();
            let supers = super_args.get(&class.name).cloned().unwrap_or_default();
            if !caps.is_empty() || !supers.is_empty() {
                add_capture_members(class, &caps, &supers);
            }
        }
    }

    // An anonymous class is hoisted to the top level, so a bare name that
    // meant the ENCLOSING class's static field no longer resolves. Record the
    // enclosing class; codegen falls back to its statics when a name is
    // neither a local, a parameter, nor a member of the class itself.
    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            if let Some(owner) = owners.get(&class.name) {
                class.enclosing = Some(owner.clone());
            }
        }
    }

    // Phase 2b: pass the captured locals at every `new Anon$N()` site.
    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            for method in &mut class.methods {
                rewrite_stmts(&mut method.body, &captures);
            }
            for block in &mut class.init_blocks {
                rewrite_stmts(&mut block.body, &captures);
            }
        }
    }
    diagnostics
}

/// A lambda that reads or writes an enclosing INSTANCE member captures the
/// enclosing `this`, as Java does (its `this$0`). Prepend a synthetic
/// `__caturraOuter` capture to each such lambda. Scoped to lambdas: they have
/// no members of their own, so a bare field, a bare method call, or `this`
/// unambiguously means the enclosing instance. Anonymous classes, whose own
/// members can shadow, keep their existing behaviour.
fn inject_outer_captures(
    units: &[(String, CompilationUnit)],
    anon_bodies: &HashMap<String, ClassDecl>,
    owners: &HashMap<String, String>,
    captures: &mut HashMap<String, Vec<(String, TypeRef)>>,
) {
    let instance_members: HashMap<String, (HashSet<String>, HashSet<String>)> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .map(|c| {
            let fields = c
                .fields
                .iter()
                .filter(|f| !f.is_static)
                .map(|f| f.name.clone())
                .collect();
            let methods = c
                .methods
                .iter()
                .filter(|m| !m.is_static && !m.is_constructor)
                .map(|m| m.name.clone())
                .collect();
            (c.name.clone(), (fields, methods))
        })
        .collect();
    for (name, body) in anon_bodies {
        if !name.starts_with("Lambda$") {
            continue;
        }
        let Some(owner) = owners.get(name) else {
            continue;
        };
        let Some((fields, methods)) = instance_members.get(owner) else {
            continue;
        };
        if lambda_needs_outer(body, fields, methods) {
            // The outer instance is captured first, so its constructor
            // parameter precedes the local captures.
            let caps = captures.entry(name.clone()).or_default();
            caps.insert(
                0,
                (String::from(OUTER_FIELD), TypeRef::Named(owner.clone())),
            );
        }
    }
}

/// Add a field per capture and a constructor that forwards `supers` to
/// `super(...)` and stores the captured locals. The constructor's parameters
/// are the super-args first (`__super0`, …) then one per capture.
fn add_capture_members(class: &mut ClassDecl, caps: &[(String, TypeRef)], supers: &[TypeRef]) {
    let zero = class.span;
    for (name, ty) in caps {
        class.fields.push(FieldDecl {
            name: name.clone(),
            ty: ty.clone(),
            is_static: false,
            is_private: true,
            is_final: true,
            init: None,
            order: 0,
            span: zero,
        });
    }
    let super_names: Vec<String> = (0..supers.len()).map(|i| format!("__super{i}")).collect();
    let mut params: Vec<Param> = supers
        .iter()
        .zip(&super_names)
        .map(|(ty, name)| Param {
            ty: ty.clone(),
            name: name.clone(),
            is_varargs: false,
        })
        .collect();
    params.extend(caps.iter().map(|(name, ty)| Param {
        ty: ty.clone(),
        name: name.clone(),
        is_varargs: false,
    }));
    let mut body: Vec<Stmt> = Vec::new();
    if !supers.is_empty() {
        // super(...) must be the first statement in the constructor.
        body.push(Stmt::SuperCall {
            args: super_names
                .iter()
                .map(|name| Expr::Name {
                    path: vec![name.clone()],
                    span: zero,
                })
                .collect(),
            span: zero,
        });
    }
    body.extend(caps.iter().map(|(name, _)| Stmt::Assign {
        target: crate::ast::AssignTarget::Field {
            object: Box::new(Expr::This { span: zero }),
            name: name.clone(),
        },
        op: None,
        value: Expr::Name {
            path: vec![name.clone()],
            span: zero,
        },
        span: zero,
    }));
    class.methods.push(MethodDecl {
        name: class.name.clone(),
        is_static: false,
        is_public: false,
        is_private: false,
        is_constructor: true,
        is_abstract: false,
        type_params: Vec::new(),
        return_type: TypeRef::Void,
        params,
        body,
        annotations: Vec::new(),
        span: zero,
    });
}

// ----- Phase 1: find captures -----

type Scope = Vec<HashMap<String, TypeRef>>;

/// What phase 1 collects per anonymous class: the captured enclosing locals and
/// the types of the super-constructor arguments at its `new` site.
#[derive(Default)]
struct Found {
    captures: HashMap<String, Vec<(String, TypeRef)>>,
    super_args: HashMap<String, Vec<TypeRef>>,
    /// Anonymous class name -> the class whose method instantiates it. Its
    /// static fields are in scope inside the body, but the hoisted class
    /// cannot see them by bare name.
    owner: HashMap<String, String>,
    /// `local variables referenced from a lambda expression must be final or
    /// effectively final`, one per offending `new Anon$N(...)` site.
    diagnostics: Vec<(String, SourceSpan)>,
}

fn scope_lookup(scope: &Scope, name: &str) -> Option<TypeRef> {
    scope
        .iter()
        .rev()
        .find_map(|frame| frame.get(name).cloned())
}

/// Best-effort static type of a super-constructor argument, from the forms
/// students actually pass: a local/param name, a `new T(...)`, a cast, an
/// array creation, or a literal. `super(...)` resolution then matches it (with
/// assignability) against the real superclass constructor.
fn infer_type(expr: &Expr, scope: &Scope) -> Option<TypeRef> {
    match expr {
        Expr::Name { path, .. } if path.len() == 1 => scope_lookup(scope, &path[0]),
        Expr::NewObject { class, .. } => Some(TypeRef::Named(class.clone())),
        Expr::NewArray { elem, dims, .. } => {
            let mut ty = elem.clone();
            for _ in 0..dims.len() {
                ty = TypeRef::Array(Box::new(ty));
            }
            Some(ty)
        }
        Expr::Cast { ty, .. } => Some(ty.clone()),
        Expr::Literal { value, .. } => Some(match value {
            crate::ast::Literal::Int(_) => TypeRef::Int,
            crate::ast::Literal::Long(_) => TypeRef::Long,
            crate::ast::Literal::Float(_) => TypeRef::Float,
            crate::ast::Literal::Double(_) => TypeRef::Double,
            crate::ast::Literal::Char(_) => TypeRef::Char,
            crate::ast::Literal::Bool(_) => TypeRef::Boolean,
            crate::ast::Literal::Str(_) => TypeRef::Named(String::from("String")),
            crate::ast::Literal::Null => return None,
        }),
        _ => None,
    }
}

fn find_in_stmts(
    stmts: &[Stmt],
    scope: &mut Scope,
    anon: &HashMap<String, ClassDecl>,
    out: &mut Found,
    owner: &str,
    mutations: &Mutations,
) {
    scope.push(HashMap::new());
    for stmt in stmts {
        find_in_stmt(stmt, scope, anon, out, owner, mutations);
    }
    scope.pop();
}

#[allow(clippy::match_same_arms)]
#[allow(clippy::too_many_lines)] // capture walk, one arm per statement kind
fn find_in_stmt(
    stmt: &Stmt,
    scope: &mut Scope,
    anon: &HashMap<String, ClassDecl>,
    out: &mut Found,
    owner: &str,
    mutations: &Mutations,
) {
    match stmt {
        Stmt::Block(body) => find_in_stmts(body, scope, anon, out, owner, mutations),
        Stmt::LocalDecl {
            ty, declarators, ..
        } => {
            for d in declarators {
                if let Some(init) = &d.init {
                    find_in_expr(init, scope, anon, out, owner, mutations);
                }
                // The variable is in scope for later statements.
                if let Some(frame) = scope.last_mut() {
                    frame.insert(d.name.clone(), ty.clone());
                }
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => {
            find_in_expr(e, scope, anon, out, owner, mutations);
        }
        Stmt::Assign { value, .. } => find_in_expr(value, scope, anon, out, owner, mutations),
        Stmt::Return { value: Some(e), .. } => find_in_expr(e, scope, anon, out, owner, mutations),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            find_in_expr(cond, scope, anon, out, owner, mutations);
            find_in_stmt(then, scope, anon, out, owner, mutations);
            if let Some(e) = els {
                find_in_stmt(e, scope, anon, out, owner, mutations);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            find_in_expr(cond, scope, anon, out, owner, mutations);
            find_in_stmt(body, scope, anon, out, owner, mutations);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            scope.push(HashMap::new());
            if let Some(s) = init {
                find_in_stmt(s, scope, anon, out, owner, mutations);
            }
            if let Some(c) = cond {
                find_in_expr(c, scope, anon, out, owner, mutations);
            }
            for s in update {
                find_in_stmt(s, scope, anon, out, owner, mutations);
            }
            find_in_stmt(body, scope, anon, out, owner, mutations);
            scope.pop();
        }
        Stmt::ForEach {
            ty,
            name,
            iterable,
            body,
            ..
        } => {
            find_in_expr(iterable, scope, anon, out, owner, mutations);
            scope.push(HashMap::new());
            if let Some(frame) = scope.last_mut() {
                frame.insert(name.clone(), ty.clone());
            }
            find_in_stmt(body, scope, anon, out, owner, mutations);
            scope.pop();
        }
        Stmt::Switch { selector, arms, .. } => {
            find_in_expr(selector, scope, anon, out, owner, mutations);
            for arm in arms {
                for stmt in &arm.body {
                    find_in_stmt(stmt, scope, anon, out, owner, mutations);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            find_in_stmts(body, scope, anon, out, owner, mutations);
            for c in catches {
                scope.push(HashMap::new());
                if let Some(frame) = scope.last_mut() {
                    frame.insert(c.name.clone(), c.ty.clone());
                }
                find_in_stmts(&c.body, scope, anon, out, owner, mutations);
                scope.pop();
            }
            if let Some(fin) = finally_body {
                find_in_stmts(fin, scope, anon, out, owner, mutations);
            }
        }
        Stmt::Labeled { body, .. } => find_in_stmt(body, scope, anon, out, owner, mutations),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                find_in_expr(a, scope, anon, out, owner, mutations);
            }
        }
    }
}

#[allow(clippy::too_many_lines)] // capture walk, one arm per expression kind
fn find_in_expr(
    expr: &Expr,
    scope: &mut Scope,
    anon: &HashMap<String, ClassDecl>,
    out: &mut Found,
    owner: &str,
    mutations: &Mutations,
) {
    match expr {
        Expr::NewObject {
            class, args, span, ..
        } => {
            for a in args {
                find_in_expr(a, scope, anon, out, owner, mutations);
            }
            if let Some(body) = anon.get(class)
                && !out.captures.contains_key(class)
            {
                let caps = captures_of(body, scope);
                // JLS §4.12.4: a captured local must be final or effectively
                // final. Copying it into a synthetic field hides a later
                // write, so the program would quietly disagree with a JDK
                // that refuses to compile it.
                let written_inside = assigned_in_class(body);
                for (name, _) in &caps {
                    if !mutations.effectively_final(name) || written_inside.contains(name) {
                        let what = if class.starts_with("Lambda$") {
                            "a lambda expression"
                        } else {
                            "an inner class"
                        };
                        out.diagnostics.push((
                            format!(
                                "local variables referenced from {what} must be final or \
                                 effectively final"
                            ),
                            *span,
                        ));
                    }
                }
                out.captures.insert(class.clone(), caps);
                out.owner.insert(class.clone(), owner.to_owned());
                // These args (the ones the parser recorded) are the super-args;
                // captured locals get appended later, in phase 2b.
                if !args.is_empty() {
                    let types = args
                        .iter()
                        .map(|a| infer_type(a, scope))
                        .collect::<Vec<_>>();
                    out.super_args.insert(
                        class.clone(),
                        types
                            .into_iter()
                            .map(|t| t.unwrap_or(TypeRef::Named(String::from("Object"))))
                            .collect(),
                    );
                }
            }
        }
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                find_in_expr(r, scope, anon, out, owner, mutations);
            }
            for a in args {
                find_in_expr(a, scope, anon, out, owner, mutations);
            }
        }
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                find_in_expr(a, scope, anon, out, owner, mutations);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            find_in_expr(lhs, scope, anon, out, owner, mutations);
            find_in_expr(rhs, scope, anon, out, owner, mutations);
        }
        Expr::Unary { operand, .. }
        | Expr::Cast { operand, .. }
        | Expr::Field {
            object: operand, ..
        }
        | Expr::InstanceOf { value: operand, .. } => {
            find_in_expr(operand, scope, anon, out, owner, mutations);
        }
        Expr::Index { array, index, .. } => {
            find_in_expr(array, scope, anon, out, owner, mutations);
            find_in_expr(index, scope, anon, out, owner, mutations);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            find_in_expr(cond, scope, anon, out, owner, mutations);
            find_in_expr(then, scope, anon, out, owner, mutations);
            find_in_expr(els, scope, anon, out, owner, mutations);
        }
        Expr::IncDec { target, .. } => find_in_expr(target, scope, anon, out, owner, mutations),
        Expr::NewArray { dims, init, .. } => {
            for d in dims.iter().flatten() {
                find_in_expr(d, scope, anon, out, owner, mutations);
            }
            if let Some(elems) = init {
                for e in elems {
                    find_in_expr(e, scope, anon, out, owner, mutations);
                }
            }
        }
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                find_in_expr(e, scope, anon, out, owner, mutations);
            }
        }
        Expr::MethodRef { qualifier, .. } => {
            find_in_expr(qualifier, scope, anon, out, owner, mutations);
        }
        Expr::Lambda { body, .. } => match body {
            LambdaBody::Expr(e) => find_in_expr(e, scope, anon, out, owner, mutations),
            LambdaBody::Block(stmts) => {
                for s in stmts {
                    find_in_stmt(s, scope, anon, out, owner, mutations);
                }
            }
        },
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } | Expr::Super { .. } => {}
    }
}

/// The captures of an anonymous class: its free simple names that are
/// bound as locals in the enclosing scope, with their declared types,
/// sorted by name.
fn captures_of(body: &ClassDecl, enclosing: &Scope) -> Vec<(String, TypeRef)> {
    let free = free_names(body);
    let mut caps: Vec<(String, TypeRef)> = free
        .into_iter()
        .filter_map(|name| scope_lookup(enclosing, &name).map(|ty| (name, ty)))
        .collect();
    caps.sort_by(|a, b| a.0.cmp(&b.0));
    caps
}

// ----- free-name analysis of an anonymous class body -----

/// Simple names referenced in the body that are not bound within it
/// (its own fields, method parameters, or locals).
fn free_names(class: &ClassDecl) -> HashSet<String> {
    let fields: HashSet<String> = class.fields.iter().map(|f| f.name.clone()).collect();
    let mut free = HashSet::new();
    for method in &class.methods {
        let mut bound: HashSet<String> = fields.clone();
        for p in &method.params {
            bound.insert(p.name.clone());
        }
        for stmt in &method.body {
            free_in_stmt(stmt, &mut bound, &mut free);
        }
    }
    free
}

#[allow(clippy::match_same_arms, clippy::too_many_lines)]
fn free_in_stmt(stmt: &Stmt, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    match stmt {
        Stmt::Block(body) => {
            let snapshot = bound.clone();
            for s in body {
                free_in_stmt(s, bound, free);
            }
            *bound = snapshot;
        }
        Stmt::LocalDecl { declarators, .. } => {
            for d in declarators {
                if let Some(init) = &d.init {
                    free_in_expr(init, bound, free);
                }
                bound.insert(d.name.clone());
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => free_in_expr(e, bound, free),
        Stmt::Assign { target, value, .. } => {
            free_in_target(target, bound, free);
            free_in_expr(value, bound, free);
        }
        Stmt::Return { value: Some(e), .. } => free_in_expr(e, bound, free),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            free_in_expr(cond, bound, free);
            free_in_stmt(then, bound, free);
            if let Some(e) = els {
                free_in_stmt(e, bound, free);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            free_in_expr(cond, bound, free);
            free_in_stmt(body, bound, free);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            let snapshot = bound.clone();
            if let Some(s) = init {
                free_in_stmt(s, bound, free);
            }
            if let Some(c) = cond {
                free_in_expr(c, bound, free);
            }
            for s in update {
                free_in_stmt(s, bound, free);
            }
            free_in_stmt(body, bound, free);
            *bound = snapshot;
        }
        Stmt::ForEach {
            name,
            iterable,
            body,
            ..
        } => {
            free_in_expr(iterable, bound, free);
            let snapshot = bound.clone();
            bound.insert(name.clone());
            free_in_stmt(body, bound, free);
            *bound = snapshot;
        }
        Stmt::Switch { selector, arms, .. } => {
            free_in_expr(selector, bound, free);
            for arm in arms {
                for label in arm.labels.iter().flatten() {
                    free_in_expr(label, bound, free);
                }
                for s in &arm.body {
                    free_in_stmt(s, bound, free);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            let snapshot = bound.clone();
            for s in body {
                free_in_stmt(s, bound, free);
            }
            *bound = snapshot.clone();
            for c in catches {
                let inner = bound.clone();
                bound.insert(c.name.clone());
                for s in &c.body {
                    free_in_stmt(s, bound, free);
                }
                *bound = inner;
            }
            if let Some(fin) = finally_body {
                for s in fin {
                    free_in_stmt(s, bound, free);
                }
            }
            *bound = snapshot;
        }
        Stmt::Labeled { body, .. } => free_in_stmt(body, bound, free),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                free_in_expr(a, bound, free);
            }
        }
    }
}

fn free_in_target(
    target: &crate::ast::AssignTarget,
    bound: &HashSet<String>,
    free: &mut HashSet<String>,
) {
    match target {
        crate::ast::AssignTarget::Var(name) => {
            if !bound.contains(name) {
                free.insert(name.clone());
            }
        }
        crate::ast::AssignTarget::Index { array, index } => {
            free_in_expr(array, &mut bound.clone(), free);
            free_in_expr(index, &mut bound.clone(), free);
        }
        crate::ast::AssignTarget::Field { object, .. } => {
            free_in_expr(object, &mut bound.clone(), free);
        }
    }
}

fn free_in_expr(expr: &Expr, bound: &mut HashSet<String>, free: &mut HashSet<String>) {
    match expr {
        // A bare simple name that isn't locally bound is free.
        Expr::Name { path, .. } if path.len() == 1 => {
            if !bound.contains(&path[0]) {
                free.insert(path[0].clone());
            }
        }
        // Longer dotted paths: only the head can be a captured local
        // (`node.value` captures `node`).
        Expr::Name { path, .. } => {
            if !bound.contains(&path[0]) {
                free.insert(path[0].clone());
            }
        }
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                free_in_expr(r, bound, free);
            }
            for a in args {
                free_in_expr(a, bound, free);
            }
        }
        Expr::SuperMethodCall { args, .. } | Expr::NewObject { args, .. } => {
            for a in args {
                free_in_expr(a, bound, free);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            free_in_expr(lhs, bound, free);
            free_in_expr(rhs, bound, free);
        }
        Expr::Unary { operand, .. }
        | Expr::Cast { operand, .. }
        | Expr::Field {
            object: operand, ..
        }
        | Expr::InstanceOf { value: operand, .. } => free_in_expr(operand, bound, free),
        Expr::Index { array, index, .. } => {
            free_in_expr(array, bound, free);
            free_in_expr(index, bound, free);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            free_in_expr(cond, bound, free);
            free_in_expr(then, bound, free);
            free_in_expr(els, bound, free);
        }
        Expr::IncDec { target, .. } => free_in_expr(target, bound, free),
        Expr::NewArray { dims, init, .. } => {
            for d in dims.iter().flatten() {
                free_in_expr(d, bound, free);
            }
            if let Some(elems) = init {
                for e in elems {
                    free_in_expr(e, bound, free);
                }
            }
        }
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                free_in_expr(e, bound, free);
            }
        }
        Expr::MethodRef { qualifier, .. } => free_in_expr(qualifier, bound, free),
        Expr::Lambda { body, .. } => match body {
            LambdaBody::Expr(e) => free_in_expr(e, bound, free),
            LambdaBody::Block(stmts) => {
                for s in stmts {
                    free_in_stmt(s, bound, free);
                }
            }
        },
        Expr::Literal { .. } | Expr::This { .. } | Expr::Super { .. } => {}
    }
}

// ----- Phase 2b: pass captured args at the `new` sites -----

fn rewrite_stmts(stmts: &mut [Stmt], captures: &HashMap<String, Vec<(String, TypeRef)>>) {
    for stmt in stmts {
        rewrite_stmt(stmt, captures);
    }
}

#[allow(clippy::match_same_arms)]
fn rewrite_stmt(stmt: &mut Stmt, captures: &HashMap<String, Vec<(String, TypeRef)>>) {
    match stmt {
        Stmt::Block(body) => rewrite_stmts(body, captures),
        Stmt::LocalDecl { declarators, .. } => {
            for d in declarators {
                if let Some(init) = &mut d.init {
                    rewrite_expr(init, captures);
                }
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => rewrite_expr(e, captures),
        Stmt::Assign { value, .. } => rewrite_expr(value, captures),
        Stmt::Return { value: Some(e), .. } => rewrite_expr(e, captures),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            rewrite_expr(cond, captures);
            rewrite_stmt(then, captures);
            if let Some(e) = els {
                rewrite_stmt(e, captures);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            rewrite_expr(cond, captures);
            rewrite_stmt(body, captures);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            if let Some(s) = init {
                rewrite_stmt(s, captures);
            }
            if let Some(c) = cond {
                rewrite_expr(c, captures);
            }
            for s in update {
                rewrite_stmt(s, captures);
            }
            rewrite_stmt(body, captures);
        }
        Stmt::ForEach { iterable, body, .. } => {
            rewrite_expr(iterable, captures);
            rewrite_stmt(body, captures);
        }
        Stmt::Switch { selector, arms, .. } => {
            rewrite_expr(selector, captures);
            for arm in arms {
                for s in &mut arm.body {
                    rewrite_stmt(s, captures);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            rewrite_stmts(body, captures);
            for c in catches {
                rewrite_stmts(&mut c.body, captures);
            }
            if let Some(fin) = finally_body {
                rewrite_stmts(fin, captures);
            }
        }
        Stmt::Labeled { body, .. } => rewrite_stmt(body, captures),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                rewrite_expr(a, captures);
            }
        }
    }
}

fn rewrite_expr(expr: &mut Expr, captures: &HashMap<String, Vec<(String, TypeRef)>>) {
    if let Expr::NewObject {
        class, args, span, ..
    } = expr
    {
        for a in args.iter_mut() {
            rewrite_expr(a, captures);
        }
        if let Some(caps) = captures.get(class.as_str())
            && !caps.is_empty()
        {
            let span = *span;
            // Append captured values after the super-args already at the
            // site. `__caturraOuter` is the enclosing instance itself.
            args.extend(caps.iter().map(|(name, _)| {
                if name == OUTER_FIELD {
                    Expr::This { span }
                } else {
                    Expr::Name {
                        path: vec![name.clone()],
                        span,
                    }
                }
            }));
        }
        return;
    }
    walk_expr_children(expr, &mut |e| rewrite_expr(e, captures));
}

/// Apply `f` to each direct sub-expression.
#[allow(clippy::match_same_arms)]
fn walk_expr_children(expr: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    match expr {
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                f(r);
            }
            for a in args {
                f(a);
            }
        }
        Expr::SuperMethodCall { args, .. } | Expr::NewObject { args, .. } => {
            for a in args {
                f(a);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            f(lhs);
            f(rhs);
        }
        Expr::Unary { operand, .. }
        | Expr::Cast { operand, .. }
        | Expr::Field {
            object: operand, ..
        }
        | Expr::InstanceOf { value: operand, .. } => f(operand),
        Expr::Index { array, index, .. } => {
            f(array);
            f(index);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            f(cond);
            f(then);
            f(els);
        }
        Expr::IncDec { target, .. } => f(target),
        Expr::NewArray { dims, init, .. } => {
            for d in dims.iter_mut().flatten() {
                f(d);
            }
            if let Some(elems) = init {
                for e in elems {
                    f(e);
                }
            }
        }
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                f(e);
            }
        }
        Expr::MethodRef { qualifier, .. } => f(qualifier),
        Expr::Lambda { body, .. } => {
            if let LambdaBody::Expr(e) = body {
                f(e);
            }
        }
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } | Expr::Super { .. } => {}
    }
}

/// How a local is assigned in a method body, for the effectively-final rule.
#[derive(Default)]
struct Mutations {
    /// Locals declared with an initializer, and parameters.
    initialized: HashSet<String>,
    /// How many times each name appears as an assignment target. Statement
    /// `++`/`--` already lowered to `+= 1`, so they land here too.
    assigns: HashMap<String, usize>,
}

impl Mutations {
    /// JLS §4.12.4. A local with an initializer (or a parameter) is
    /// effectively final only if never assigned again. A blank local may be
    /// assigned once — javac allows one assignment per path, so a blank local
    /// set on both arms of an `if` is effectively final; caturra counts
    /// assignments instead of tracking definite assignment, so it refuses
    /// that. Rejecting valid Java is the safe direction.
    fn effectively_final(&self, name: &str) -> bool {
        let assigns = self.assigns.get(name).copied().unwrap_or(0);
        if self.initialized.contains(name) {
            assigns == 0
        } else {
            assigns <= 1
        }
    }
}

/// Every assignment target and initialized declaration in `stmts`. Assignments
/// are statements in caturra (value-position `++` is rejected by the parser),
/// and lambda bodies are already hoisted into their own classes, so statement
/// recursion sees everything without descending into expressions.
fn mutations_in_stmts(stmts: &[Stmt], out: &mut Mutations) {
    for stmt in stmts {
        mutations_in_stmt(stmt, out);
    }
}

#[allow(clippy::match_same_arms)]
fn mutations_in_stmt(stmt: &Stmt, out: &mut Mutations) {
    match stmt {
        Stmt::Block(body) => mutations_in_stmts(body, out),
        Stmt::LocalDecl { declarators, .. } => {
            for d in declarators {
                if d.init.is_some() {
                    out.initialized.insert(d.name.clone());
                }
            }
        }
        Stmt::Assign {
            target: crate::ast::AssignTarget::Var(name),
            ..
        } => {
            *out.assigns.entry(name.clone()).or_default() += 1;
        }
        Stmt::If { then, els, .. } => {
            mutations_in_stmt(then, out);
            if let Some(e) = els {
                mutations_in_stmt(e, out);
            }
        }
        Stmt::While { body, .. } | Stmt::DoWhile { body, .. } => mutations_in_stmt(body, out),
        Stmt::For {
            init, update, body, ..
        } => {
            if let Some(s) = init {
                mutations_in_stmt(s, out);
            }
            for s in update {
                mutations_in_stmt(s, out);
            }
            mutations_in_stmt(body, out);
        }
        Stmt::ForEach { body, .. } => mutations_in_stmt(body, out),
        Stmt::Labeled { body, .. } => mutations_in_stmt(body, out),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            mutations_in_stmts(body, out);
            for c in catches {
                mutations_in_stmts(&c.body, out);
            }
            if let Some(f) = finally_body {
                mutations_in_stmts(f, out);
            }
        }
        Stmt::Switch { arms, .. } => {
            for arm in arms {
                mutations_in_stmts(&arm.body, out);
            }
        }
        _ => {}
    }
}

/// The locals a synthesized class's own body assigns. `() -> c++` writes the
/// capture field, but javac reads it as writing `c`, and refuses.
fn assigned_in_class(class: &ClassDecl) -> HashSet<String> {
    let mut out = Mutations::default();
    for method in &class.methods {
        mutations_in_stmts(&method.body, &mut out);
    }
    out.assigns.into_keys().collect()
}

/// The synthetic field holding a lambda's captured enclosing instance.
pub(crate) const OUTER_FIELD: &str = "__caturraOuter";

/// Whether a lambda body references the enclosing instance: a free name that
/// is an enclosing instance field, `this`, or a bare call to an enclosing
/// instance method. `free_names` already discounts the lambda's own
/// parameters and locals; `this` and a bare call need no such tracking (a
/// lambda has no `this` of its own, and a bare call always names a method).
fn lambda_needs_outer(
    body: &ClassDecl,
    inst_fields: &HashSet<String>,
    inst_methods: &HashSet<String>,
) -> bool {
    if free_names(body).iter().any(|n| inst_fields.contains(n)) {
        return true;
    }
    body.methods
        .iter()
        .any(|m| stmts_use_outer(&m.body, inst_methods))
}

fn stmts_use_outer(stmts: &[Stmt], methods: &HashSet<String>) -> bool {
    stmts.iter().any(|s| stmt_uses_outer(s, methods))
}

fn stmt_uses_outer(stmt: &Stmt, methods: &HashSet<String>) -> bool {
    let e = |x: &Expr| expr_uses_outer(x, methods);
    let sub = |x: &Stmt| stmt_uses_outer(x, methods);
    match stmt {
        Stmt::Block(body) => stmts_use_outer(body, methods),
        Stmt::Expr(x) | Stmt::Throw { value: x, .. } => e(x),
        Stmt::LocalDecl { declarators, .. } => {
            declarators.iter().filter_map(|d| d.init.as_ref()).any(e)
        }
        Stmt::Assign { target, value, .. } => {
            let t = match target {
                crate::ast::AssignTarget::Var(_) => false,
                crate::ast::AssignTarget::Index { array, index } => e(array) || e(index),
                crate::ast::AssignTarget::Field { object, .. } => e(object),
            };
            t || e(value)
        }
        Stmt::ForEach { iterable, body, .. } => e(iterable) || sub(body),
        Stmt::If {
            cond, then, els, ..
        } => e(cond) || sub(then) || els.as_deref().is_some_and(sub),
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => e(cond) || sub(body),
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            init.as_deref().is_some_and(sub)
                || cond.as_ref().is_some_and(e)
                || update.iter().any(sub)
                || sub(body)
        }
        Stmt::Labeled { body, .. } => sub(body),
        Stmt::Return { value, .. } => value.as_ref().is_some_and(e),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => args.iter().any(e),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            stmts_use_outer(body, methods)
                || catches.iter().any(|c| stmts_use_outer(&c.body, methods))
                || finally_body
                    .as_ref()
                    .is_some_and(|f| stmts_use_outer(f, methods))
        }
        Stmt::Switch { selector, arms, .. } => {
            e(selector) || arms.iter().any(|a| stmts_use_outer(&a.body, methods))
        }
        Stmt::Break { .. } | Stmt::Continue { .. } => false,
    }
}

fn expr_uses_outer(expr: &Expr, methods: &HashSet<String>) -> bool {
    if matches!(expr, Expr::This { .. }) {
        return true;
    }
    if let Expr::Call {
        receiver: None,
        method,
        ..
    } = expr
        && methods.contains(method)
    {
        return true;
    }
    let mut hit = false;
    walk_expr_children(&mut expr.clone(), &mut |e| {
        hit = hit || expr_uses_outer(e, methods);
    });
    hit
}
