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

use crate::ast::{ClassDecl, CompilationUnit, Expr, FieldDecl, MethodDecl, Param, Stmt, TypeRef};

/// Resolve captures for all anonymous classes in the compilation.
pub fn resolve_captures(units: &mut [(String, CompilationUnit)]) {
    // Clone anonymous-class bodies for free-name analysis (read-only).
    let anon_bodies: HashMap<String, ClassDecl> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .filter(|c| c.is_anonymous)
        .map(|c| (c.name.clone(), c.clone()))
        .collect();
    if anon_bodies.is_empty() {
        return;
    }

    // Phase 1: capture set per anonymous class (sorted for determinism).
    let mut captures: HashMap<String, Vec<(String, TypeRef)>> = HashMap::new();
    for (_, unit) in units.iter() {
        for class in &unit.classes {
            for method in &class.methods {
                let mut scope: Vec<HashMap<String, TypeRef>> = vec![
                    method
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone()))
                        .collect(),
                ];
                find_in_stmts(&method.body, &mut scope, &anon_bodies, &mut captures);
            }
            for block in &class.init_blocks {
                let mut scope: Vec<HashMap<String, TypeRef>> = vec![HashMap::new()];
                find_in_stmts(&block.body, &mut scope, &anon_bodies, &mut captures);
            }
        }
    }

    // Phase 2a: synthesize the fields and constructor on each anon class.
    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            if let Some(caps) = captures.get(&class.name)
                && !caps.is_empty()
            {
                add_capture_members(class, caps);
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
}

/// Add a field per capture and a constructor that stores them.
fn add_capture_members(class: &mut ClassDecl, caps: &[(String, TypeRef)]) {
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
    let params: Vec<Param> = caps
        .iter()
        .map(|(name, ty)| Param {
            ty: ty.clone(),
            name: name.clone(),
            is_varargs: false,
        })
        .collect();
    let body: Vec<Stmt> = caps
        .iter()
        .map(|(name, _)| Stmt::Assign {
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
        })
        .collect();
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
        span: zero,
    });
}

// ----- Phase 1: find captures -----

type Scope = Vec<HashMap<String, TypeRef>>;

fn scope_lookup(scope: &Scope, name: &str) -> Option<TypeRef> {
    scope
        .iter()
        .rev()
        .find_map(|frame| frame.get(name).cloned())
}

fn find_in_stmts(
    stmts: &[Stmt],
    scope: &mut Scope,
    anon: &HashMap<String, ClassDecl>,
    out: &mut HashMap<String, Vec<(String, TypeRef)>>,
) {
    scope.push(HashMap::new());
    for stmt in stmts {
        find_in_stmt(stmt, scope, anon, out);
    }
    scope.pop();
}

#[allow(clippy::match_same_arms)]
fn find_in_stmt(
    stmt: &Stmt,
    scope: &mut Scope,
    anon: &HashMap<String, ClassDecl>,
    out: &mut HashMap<String, Vec<(String, TypeRef)>>,
) {
    match stmt {
        Stmt::Block(body) => find_in_stmts(body, scope, anon, out),
        Stmt::LocalDecl {
            ty, declarators, ..
        } => {
            for d in declarators {
                if let Some(init) = &d.init {
                    find_in_expr(init, scope, anon, out);
                }
                // The variable is in scope for later statements.
                if let Some(frame) = scope.last_mut() {
                    frame.insert(d.name.clone(), ty.clone());
                }
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => find_in_expr(e, scope, anon, out),
        Stmt::Assign { value, .. } => find_in_expr(value, scope, anon, out),
        Stmt::Return { value: Some(e), .. } => find_in_expr(e, scope, anon, out),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            find_in_expr(cond, scope, anon, out);
            find_in_stmt(then, scope, anon, out);
            if let Some(e) = els {
                find_in_stmt(e, scope, anon, out);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            find_in_expr(cond, scope, anon, out);
            find_in_stmt(body, scope, anon, out);
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
                find_in_stmt(s, scope, anon, out);
            }
            if let Some(c) = cond {
                find_in_expr(c, scope, anon, out);
            }
            for s in update {
                find_in_stmt(s, scope, anon, out);
            }
            find_in_stmt(body, scope, anon, out);
            scope.pop();
        }
        Stmt::ForEach {
            ty,
            name,
            iterable,
            body,
            ..
        } => {
            find_in_expr(iterable, scope, anon, out);
            scope.push(HashMap::new());
            if let Some(frame) = scope.last_mut() {
                frame.insert(name.clone(), ty.clone());
            }
            find_in_stmt(body, scope, anon, out);
            scope.pop();
        }
        Stmt::Switch { selector, arms, .. } => {
            find_in_expr(selector, scope, anon, out);
            for arm in arms {
                for stmt in &arm.body {
                    find_in_stmt(stmt, scope, anon, out);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            find_in_stmts(body, scope, anon, out);
            for c in catches {
                scope.push(HashMap::new());
                if let Some(frame) = scope.last_mut() {
                    frame.insert(c.name.clone(), c.ty.clone());
                }
                find_in_stmts(&c.body, scope, anon, out);
                scope.pop();
            }
            if let Some(fin) = finally_body {
                find_in_stmts(fin, scope, anon, out);
            }
        }
        Stmt::Labeled { body, .. } => find_in_stmt(body, scope, anon, out),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                find_in_expr(a, scope, anon, out);
            }
        }
    }
}

fn find_in_expr(
    expr: &Expr,
    scope: &mut Scope,
    anon: &HashMap<String, ClassDecl>,
    out: &mut HashMap<String, Vec<(String, TypeRef)>>,
) {
    match expr {
        Expr::NewObject { class, args, .. } => {
            for a in args {
                find_in_expr(a, scope, anon, out);
            }
            if let Some(body) = anon.get(class)
                && !out.contains_key(class)
            {
                let caps = captures_of(body, scope);
                out.insert(class.clone(), caps);
            }
        }
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                find_in_expr(r, scope, anon, out);
            }
            for a in args {
                find_in_expr(a, scope, anon, out);
            }
        }
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                find_in_expr(a, scope, anon, out);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            find_in_expr(lhs, scope, anon, out);
            find_in_expr(rhs, scope, anon, out);
        }
        Expr::Unary { operand, .. }
        | Expr::Cast { operand, .. }
        | Expr::Field {
            object: operand, ..
        }
        | Expr::InstanceOf { value: operand, .. } => find_in_expr(operand, scope, anon, out),
        Expr::Index { array, index, .. } => {
            find_in_expr(array, scope, anon, out);
            find_in_expr(index, scope, anon, out);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            find_in_expr(cond, scope, anon, out);
            find_in_expr(then, scope, anon, out);
            find_in_expr(els, scope, anon, out);
        }
        Expr::IncDec { target, .. } => find_in_expr(target, scope, anon, out),
        Expr::NewArray { dims, init, .. } => {
            for d in dims.iter().flatten() {
                find_in_expr(d, scope, anon, out);
            }
            if let Some(elems) = init {
                for e in elems {
                    find_in_expr(e, scope, anon, out);
                }
            }
        }
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                find_in_expr(e, scope, anon, out);
            }
        }
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } => {}
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
        Expr::Literal { .. } | Expr::This { .. } => {}
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
            *args = caps
                .iter()
                .map(|(name, _)| Expr::Name {
                    path: vec![name.clone()],
                    span,
                })
                .collect();
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
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } => {}
    }
}
