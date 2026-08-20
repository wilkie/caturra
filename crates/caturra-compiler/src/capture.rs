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
#[allow(clippy::too_many_lines)] // one linear pass with clearly labelled phases
pub fn resolve_captures(
    units: &mut [(String, CompilationUnit)],
) -> Vec<crate::diagnostics::Diagnostic> {
    // Clone anonymous-class bodies for free-name analysis (read-only).
    let anon_bodies: HashMap<String, ClassDecl> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        // Local classes join too: they capture enclosing locals and read the
        // enclosing statics just as anonymous classes do.
        .filter(|c| c.is_anonymous || c.is_local)
        .map(|c| (c.name.clone(), c.clone()))
        .collect();
    if anon_bodies.is_empty() {
        return Vec::new();
    }

    // Phase 1: capture set per anonymous class (sorted for determinism), plus
    // the type of each super-constructor argument at its `new` site.
    //
    // Run TWICE. The first pass learns which classes each body creates (the
    // owner map); the second uses that to make a capture set TRANSITIVE, so an
    // outer lambda captures what a lambda nested inside it needs. One pass
    // cannot do both: the relation is only known once every body has been
    // walked, and it is what the walk needs.
    let mut created_in: HashMap<String, Vec<String>> = HashMap::new();
    let mut outer_caps: HashMap<String, Vec<(String, TypeRef)>> = HashMap::new();
    let mut found = Found::default();
    let mut diagnostics: Vec<crate::diagnostics::Diagnostic> = Vec::new();
    // Phase 1 runs to a fixed point: each pass learns which classes are
    // created inside which (so a capture can be pulled DOWN transitively) and
    // what the level above ended up holding (so a capture can be seen from
    // below). One pass only ever reaches one level out.
    for round in 0..6 {
        if round > 0 {
            created_in.clear();
            for (inner, owner) in &found.owner {
                created_in
                    .entry(owner.clone())
                    .or_default()
                    .push(inner.clone());
            }
            let next = found.captures.clone();
            if round > 1 && next == outer_caps {
                break;
            }
            outer_caps = next;
            found = Found::default();
            diagnostics.clear();
        }
        collect_captures(
            units,
            &anon_bodies,
            &created_in,
            &outer_caps,
            &mut found,
            &mut diagnostics,
        );
    }
    let mut captures = found.captures;
    let mut super_args = found.super_args;
    let owners = found.owner;

    // The super-args a `new Base(expr){…}` site passes are forwarded VERBATIM
    // to `super(…)`, so the constructor that carries them is typed by the
    // superclass constructor they reach — guessing from the argument
    // expression only works for a literal or a plain local, which left
    // `new Base(field){}` (or any arithmetic, call or ternary) refusing to
    // compile at all.
    let super_ctors: HashMap<String, Vec<Vec<TypeRef>>> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .map(|c| {
            (
                c.name.clone(),
                c.methods
                    .iter()
                    .filter(|m| m.is_constructor)
                    .map(|m| m.params.iter().map(|p| p.ty.clone()).collect())
                    .collect(),
            )
        })
        .collect();
    for (name, types) in &mut super_args {
        let Some(body) = anon_bodies.get(name) else {
            continue;
        };
        let Some(base) = body.superclass.as_ref().and_then(|s| super_ctors.get(s)) else {
            continue;
        };
        // Only when ONE constructor can be the one called: with several of the
        // same arity the site's own argument types are what picks between
        // them, and that is overload resolution, which happens in codegen.
        let mut matching = base.iter().filter(|params| params.len() == types.len());
        if let Some(params) = matching.next()
            && matching.next().is_none()
        {
            types.clone_from(params);
        }
    }

    inject_outer_captures(units, &anon_bodies, &owners, &created_in, &mut captures);

    // Phase 2a: synthesize the fields and constructor on each anon class that
    // captures locals or forwards args to super.
    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            let caps = captures.get(&class.name).cloned().unwrap_or_default();
            if class.is_local {
                // A local class keeps its own constructors (its `new` args are
                // its own, not a superclass's). Thread the captures through
                // them as trailing parameters instead of forwarding to super.
                if !caps.is_empty() {
                    augment_local_ctors(class, &caps);
                }
                continue;
            }
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
            // A FIELD INITIALIZER holds `new Anon(){...}` sites too, and phase
            // 1 already collects their captures from one — but the arguments
            // were never passed here, so `Runnable r = new Runnable(){ … tag … };`
            // (an anonymous class in a field, capturing the enclosing instance)
            // failed with "constructor Anon$1 cannot be applied to given types".
            for field in &mut class.fields {
                if let Some(init) = &mut field.init {
                    rewrite_expr(init, &captures);
                }
            }
        }
    }
    diagnostics
}

/// One walk over every body, recording each anonymous class's capture set,
/// its owner, and its super-constructor argument types.
fn collect_captures(
    units: &[(String, CompilationUnit)],
    anon_bodies: &HashMap<String, ClassDecl>,
    created_in: &HashMap<String, Vec<String>>,
    outer_caps: &HashMap<String, Vec<(String, TypeRef)>>,
    found: &mut Found,
    diagnostics: &mut Vec<crate::diagnostics::Diagnostic>,
) {
    for (path, unit) in units {
        for class in &unit.classes {
            // What a `new Base(expr){…}` site inside this class can name
            // without a qualifier — the super-argument types are read off the
            // expression, and a bare field or call is the common shape.
            let members = MemberTypes {
                fields: class
                    .fields
                    .iter()
                    .map(|f| (f.name.clone(), f.ty.clone()))
                    .collect(),
                methods: class
                    .methods
                    .iter()
                    .filter(|m| !m.is_constructor)
                    .map(|m| (m.name.clone(), m.return_type.clone()))
                    .collect(),
            };

            // A synthesized class's own captures are FIELDS, not locals — but
            // to a class nested inside it they stand for the enclosing
            // method's locals, so a name two levels up has to find them here
            // or it looks free and gets captured by nobody.
            let inherited: HashMap<String, TypeRef> = outer_caps
                .get(&class.name)
                .into_iter()
                .flatten()
                .filter(|(name, _)| name != OUTER_FIELD)
                .cloned()
                .collect();
            for method in &class.methods {
                let mut scope: Vec<HashMap<String, TypeRef>> = vec![
                    inherited.clone(),
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
                    found,
                    &Walk {
                        anon: anon_bodies,
                        owner: &class.name,
                        mutations: &mutations,
                        created_in,
                        members: &members,
                    },
                );
            }
            for block in &class.init_blocks {
                let mut scope: Vec<HashMap<String, TypeRef>> = vec![inherited.clone()];
                let mut mutations = Mutations::default();
                mutations_in_stmts(&block.body, &mut mutations);
                find_in_stmts(
                    &block.body,
                    &mut scope,
                    found,
                    &Walk {
                        anon: anon_bodies,
                        owner: &class.name,
                        mutations: &mutations,
                        created_in,
                        members: &members,
                    },
                );
            }
            // Field initializers hold `new Anon(){...}` too — both a field like
            // `Runnable r = new Runnable(){...};` and, crucially, an enum
            // constant's synthesized `new Anon$N("NAME", ordinal, ...)`. Without
            // this the anon class never gets its super-forwarding constructor.
            for field in &class.fields {
                if let Some(init) = &field.init {
                    let mut scope: Vec<HashMap<String, TypeRef>> = vec![inherited.clone()];
                    find_in_expr(
                        init,
                        &mut scope,
                        found,
                        &Walk {
                            anon: anon_bodies,
                            owner: &class.name,
                            mutations: &Mutations::default(),
                            created_in,
                            members: &members,
                        },
                    );
                }
            }
        }
        diagnostics.extend(
            found
                .diagnostics
                .drain(..)
                .map(|(message, span)| crate::diagnostics::Diagnostic::error(path, message, span)),
        );
    }
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
    created_in: &HashMap<String, Vec<String>>,
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
    // Every user class's superclass, for walking what a body inherits.
    let supers: HashMap<String, String> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .filter_map(|c| c.superclass.clone().map(|s| (c.name.clone(), s)))
        .collect();

    // Which classes want the enclosing instance, propagated to a FIXED POINT:
    // a class three levels down reaches its outer instance through every
    // level in between, so each of them has to hold it too.
    let mut wants: HashMap<String, String> = HashMap::new();
    for (name, body) in anon_bodies {
        // The enclosing INSTANCE is the first owner that is a real class: a
        // lambda nested in a lambda is owned by a synthesized class, whose
        // `this` is the outer lambda rather than the object whose members the
        // body is reading. Walking to the first real owner is what makes the
        // inner reach the same instance the outer one did.
        let Some(owner) = enclosing_instance(name, owners) else {
            continue;
        };
        // What an enclosing name can mean is everything declared ANYWHERE up
        // the chain, not just in the nearest owner: `field` inside a lambda
        // inside an anonymous class is the top-level class's, reached by one
        // hop per level. Lambda levels declare nothing of their own.
        let (fields, methods) = enclosing_members(name, owners, &instance_members);
        let (fields, methods) = (&fields, &methods);
        let needs = if crate::is_lambda_class(name) {
            lambda_needs_outer(body, fields, methods)
        } else {
            // An anonymous or local class HAS members of its own, and may
            // inherit more, so a bare name means the enclosing instance only
            // when nothing nearer provides it. `this` never counts either — it
            // is the class's own instance, unlike a lambda's.
            let (own_fields, own_methods) = provided_members(body, &instance_members, &supers);
            class_needs_outer(body, fields, methods, &own_fields, &own_methods)
        };
        if needs {
            wants.insert(name.clone(), owner);
        }
    }
    loop {
        let mut grew = false;
        for (name, inners) in created_in {
            if wants.contains_key(name) || !anon_bodies.contains_key(name) {
                continue;
            }
            if inners.iter().any(|inner| wants.contains_key(inner))
                && let Some(owner) = enclosing_instance(name, owners)
            {
                wants.insert(name.clone(), owner);
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    for (name, owner) in wants {
        // The outer instance is captured first, so its constructor
        // parameter precedes the local captures.
        let caps = captures.entry(name).or_default();
        caps.insert(0, (String::from(OUTER_FIELD), TypeRef::Named(owner)));
    }
}

/// Every instance member reachable by simple name from a class body: the
/// union over the owner chain. A lambda level contributes nothing — its only
/// fields are its own captures, which a nested body reaches as locals.
fn enclosing_members(
    name: &str,
    owners: &HashMap<String, String>,
    instance_members: &HashMap<String, (HashSet<String>, HashSet<String>)>,
) -> (HashSet<String>, HashSet<String>) {
    let mut fields = HashSet::new();
    let mut methods = HashSet::new();
    let mut current = name.to_owned();
    for _ in 0..=owners.len() {
        let Some(owner) = owners.get(&current) else {
            break;
        };
        if !crate::is_lambda_class(owner)
            && let Some((f, m)) = instance_members.get(owner)
        {
            fields.extend(f.iter().cloned());
            methods.extend(m.iter().cloned());
        }
        current = owner.clone();
    }
    (fields, methods)
}

/// The instance an enclosing name is reached through — the first owner that
/// HAS a `this` of its own. A lambda does not (JLS §15.27.2: its `this` is
/// the enclosing instance), so a lambda inside a lambda reaches the same
/// object the outer one did; an anonymous or local class does, so the walk
/// stops there.
fn enclosing_instance(name: &str, owners: &HashMap<String, String>) -> Option<String> {
    let mut current = owners.get(name)?.clone();
    for _ in 0..=owners.len() {
        if !crate::is_lambda_class(&current) {
            return Some(current);
        }
        current.clone_from(owners.get(&current)?);
    }
    None
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
            is_final: false,
        })
        .collect();
    params.extend(caps.iter().map(|(name, ty)| Param {
        ty: ty.clone(),
        name: name.clone(),
        is_varargs: false,
        is_final: false,
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
        is_final: false,
        is_constructor: true,
        is_abstract: false,
        type_params: Vec::new(),
        infer_return: None,
        declared_params: Vec::new(),
        type_var_sources: Vec::new(),
        return_type: TypeRef::Void,
        params,
        body,
        annotations: Vec::new(),
        throws: Vec::new(),
        is_protected: false,
        span: zero,
        // The capture stores are javac's `val$x = x`: they run BEFORE the
        // class's field initializers, so one may read a captured local. (The
        // leading `super(...)` is not counted — codegen strips it first.)
        pre_init: caps.len(),
        declared_return: None,
    });
}

/// Thread captured locals through a LOCAL class's constructors. Unlike an
/// anonymous class, a local class has its own constructor(s) and its own
/// `new` arguments, so the captures become trailing parameters on each
/// constructor (matching the values appended at every `new` site) rather than
/// a fresh super-forwarding constructor. The stores go right after a leading
/// `super(...)`/`this(...)` so the body can already read the captured fields.
fn augment_local_ctors(class: &mut ClassDecl, caps: &[(String, TypeRef)]) {
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
    let cap_params: Vec<Param> = caps
        .iter()
        .map(|(name, ty)| Param {
            ty: ty.clone(),
            name: name.clone(),
            is_varargs: false,
            is_final: false,
        })
        .collect();
    let stores = || -> Vec<Stmt> {
        caps.iter()
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
            .collect()
    };

    let ctors: Vec<&mut MethodDecl> = class
        .methods
        .iter_mut()
        .filter(|m| m.is_constructor)
        .collect();
    if ctors.is_empty() {
        // No explicit constructor: synthesize one taking just the captures.
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
            params: cap_params,
            body: stores(),
            annotations: Vec::new(),
            throws: Vec::new(),
            is_protected: false,
            span: zero,
            // The capture stores are javac's `val$x = x`: they run BEFORE the
            // class's field initializers, so one may read a captured local.
            pre_init: caps.len(),
            declared_return: None,
        });
        return;
    }
    for ctor in ctors {
        ctor.params.extend(cap_params.iter().cloned());
        // A constructor that DELEGATES (`C() { this(6); }`) threads the
        // captured values through to the one it calls, and does not store them
        // itself — the delegate does. Without the extra arguments the
        // delegation matched the constructor's OWN new signature and recursed
        // until the stack blew.
        if let Some(Stmt::ThisCall { args, span }) = ctor.body.first_mut() {
            let span = *span;
            args.extend(caps.iter().map(|(name, _)| Expr::Name {
                path: vec![name.clone()],
                span,
            }));
            continue;
        }
        // `super(...)` must stay first; the stores follow it.
        let after = usize::from(matches!(ctor.body.first(), Some(Stmt::SuperCall { .. })));
        let mut rest = ctor.body.split_off(after);
        ctor.body.append(&mut stores());
        ctor.body.append(&mut rest);
        ctor.pre_init = caps.len();
    }
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
/// The types of the members a body can name without a qualifier: the field
/// types and the method return types, kept apart because Java lets a field
/// and a method share a name.
#[derive(Default)]
struct MemberTypes {
    fields: HashMap<String, TypeRef>,
    methods: HashMap<String, TypeRef>,
}

fn infer_type(expr: &Expr, scope: &Scope, members: &MemberTypes) -> Option<TypeRef> {
    match expr {
        Expr::Name { path, .. } if path.len() == 1 => {
            scope_lookup(scope, &path[0]).or_else(|| members.fields.get(&path[0]).cloned())
        }
        Expr::Field { object, name, .. } if matches!(**object, Expr::This { .. }) => {
            members.fields.get(name).cloned()
        }
        Expr::Call {
            receiver: None,
            method,
            ..
        } => members.methods.get(method).cloned(),
        Expr::NewObject { class, .. } => Some(TypeRef::Named(class.clone())),
        Expr::NewArray { elem, dims, .. } => {
            let mut ty = elem.clone();
            for _ in 0..dims.len() {
                ty = TypeRef::Array(Box::new(ty));
            }
            Some(ty)
        }
        Expr::Cast { ty, .. } => Some(ty.clone()),
        // JLS §15.18/§15.20/§15.22: an arithmetic operator promotes, a
        // comparison answers a boolean, and `+` with a String operand
        // concatenates.
        Expr::Binary { op, lhs, rhs, .. } => {
            use crate::ast::BinaryOp as B;
            match op {
                B::Lt | B::Le | B::Gt | B::Ge | B::Eq | B::Ne | B::And | B::Or => {
                    Some(TypeRef::Boolean)
                }
                B::Shl | B::Shr | B::Ushr => {
                    promote(&infer_type(lhs, scope, members)?, &TypeRef::Int)
                }
                _ => {
                    let (l, r) = (
                        infer_type(lhs, scope, members)?,
                        infer_type(rhs, scope, members)?,
                    );
                    let string = TypeRef::Named(String::from("String"));
                    if *op == B::Add && (l == string || r == string) {
                        return Some(string);
                    }
                    if matches!(l, TypeRef::Boolean) && matches!(r, TypeRef::Boolean) {
                        return Some(TypeRef::Boolean);
                    }
                    promote(&l, &r)
                }
            }
        }
        Expr::Unary { op, operand, .. } => match op {
            crate::ast::UnaryOp::Not => Some(TypeRef::Boolean),
            _ => promote(&infer_type(operand, scope, members)?, &TypeRef::Int),
        },
        // Only when both arms agree: the full conditional-expression rules
        // (JLS §15.25) are codegen's, and a wrong guess here would be worse
        // than no guess.
        Expr::Ternary { then, els, .. } => {
            let then = infer_type(then, scope, members)?;
            (then == infer_type(els, scope, members)?).then_some(then)
        }
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

/// Binary numeric promotion (JLS §5.6.2), for the primitive operands only —
/// anything else (a boxed type, an unknown name) declines to guess.
fn promote(lhs: &TypeRef, rhs: &TypeRef) -> Option<TypeRef> {
    for wide in [TypeRef::Double, TypeRef::Float, TypeRef::Long] {
        if *lhs == wide || *rhs == wide {
            return Some(wide);
        }
    }
    let numeric = |t: &TypeRef| {
        matches!(
            t,
            TypeRef::Int | TypeRef::Short | TypeRef::Byte | TypeRef::Char
        )
    };
    (numeric(lhs) && numeric(rhs)).then_some(TypeRef::Int)
}

/// The context a capture walk carries: what the synthesized classes are, whose
/// body is being walked, and what a name in it can turn out to mean.
struct Walk<'a> {
    anon: &'a HashMap<String, ClassDecl>,
    owner: &'a str,
    mutations: &'a Mutations,
    created_in: &'a HashMap<String, Vec<String>>,
    members: &'a MemberTypes,
}

fn find_in_stmts(stmts: &[Stmt], scope: &mut Scope, out: &mut Found, walk: &Walk) {
    scope.push(HashMap::new());
    for stmt in stmts {
        find_in_stmt(stmt, scope, out, walk);
    }
    scope.pop();
}

#[allow(clippy::match_same_arms)]
#[allow(clippy::too_many_lines)] // capture walk, one arm per statement kind
fn find_in_stmt(stmt: &Stmt, scope: &mut Scope, out: &mut Found, walk: &Walk) {
    match stmt {
        Stmt::Block(body) => find_in_stmts(body, scope, out, walk),
        Stmt::LocalDecl {
            ty, declarators, ..
        } => {
            for d in declarators {
                if let Some(init) = &d.init {
                    find_in_expr(init, scope, out, walk);
                }
                // The variable is in scope for later statements.
                if let Some(frame) = scope.last_mut() {
                    frame.insert(d.name.clone(), ty.clone());
                }
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => {
            find_in_expr(e, scope, out, walk);
        }
        Stmt::Assign { value, .. } => find_in_expr(value, scope, out, walk),
        Stmt::Return { value: Some(e), .. } => find_in_expr(e, scope, out, walk),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            find_in_expr(cond, scope, out, walk);
            find_in_stmt(then, scope, out, walk);
            if let Some(e) = els {
                find_in_stmt(e, scope, out, walk);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            find_in_expr(cond, scope, out, walk);
            find_in_stmt(body, scope, out, walk);
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
                find_in_stmt(s, scope, out, walk);
            }
            if let Some(c) = cond {
                find_in_expr(c, scope, out, walk);
            }
            for s in update {
                find_in_stmt(s, scope, out, walk);
            }
            find_in_stmt(body, scope, out, walk);
            scope.pop();
        }
        Stmt::ForEach {
            ty,
            name,
            iterable,
            body,
            ..
        } => {
            find_in_expr(iterable, scope, out, walk);
            scope.push(HashMap::new());
            if let Some(frame) = scope.last_mut() {
                frame.insert(name.clone(), ty.clone());
            }
            find_in_stmt(body, scope, out, walk);
            scope.pop();
        }
        Stmt::Switch { selector, arms, .. } => {
            find_in_expr(selector, scope, out, walk);
            for arm in arms {
                for stmt in &arm.body {
                    find_in_stmt(stmt, scope, out, walk);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            find_in_stmts(body, scope, out, walk);
            for c in catches {
                scope.push(HashMap::new());
                if let Some(frame) = scope.last_mut() {
                    // A multi-catch variable is typed by its first alternative here;
                    // capture only needs to know the name is bound, not its exact type.
                    if let Some(ty) = c.types.first() {
                        frame.insert(c.name.clone(), ty.clone());
                    }
                }
                find_in_stmts(&c.body, scope, out, walk);
                scope.pop();
            }
            if let Some(fin) = finally_body {
                find_in_stmts(fin, scope, out, walk);
            }
        }
        Stmt::Labeled { body, .. } => find_in_stmt(body, scope, out, walk),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                find_in_expr(a, scope, out, walk);
            }
        }
    }
}

#[allow(clippy::too_many_lines)] // capture walk, one arm per expression kind
fn find_in_expr(expr: &Expr, scope: &mut Scope, out: &mut Found, walk: &Walk) {
    match expr {
        Expr::NewObject {
            class,
            args,
            outer,
            span,
            ..
        } => {
            // The QUALIFIER of `o.new Inner()` is an expression like any
            // other: every walk here used to visit only the arguments, so a
            // local named in one was never seen — and a lambda body doing
            // `o.new Inner()` failed with "cannot find variable 'o'" because
            // `o` was never captured.
            if let Some(outer) = outer {
                find_in_expr(outer, scope, out, walk);
            }
            for a in args {
                find_in_expr(a, scope, out, walk);
            }
            if let Some(body) = walk.anon.get(class)
                && !out.captures.contains_key(class)
            {
                let caps = captures_of(body, scope, walk.anon, walk.created_in);
                // JLS §4.12.4: a captured local must be final or effectively
                // final. Copying it into a synthetic field hides a later
                // write, so the program would quietly disagree with a JDK
                // that refuses to compile it.
                let written_inside = assigned_in_class(body);
                // JLS §15.13.3: a METHOD REFERENCE evaluates its receiver
                // when the reference is created, so a later assignment to the
                // variable cannot be observed — javac imposes no
                // effective-finality requirement there.
                let from_method_ref = class.starts_with(crate::METHOD_REF_CLASS_PREFIX);
                for (name, _) in &caps {
                    if from_method_ref {
                        break;
                    }
                    if !walk.mutations.effectively_final(name) || written_inside.contains(name) {
                        let what = if crate::is_lambda_class(class) {
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
                out.owner.insert(class.clone(), walk.owner.to_owned());
                // These args (the ones the parser recorded) are the super-args;
                // captured locals get appended later, in phase 2b.
                if !args.is_empty() {
                    let types = args
                        .iter()
                        .map(|a| infer_type(a, scope, walk.members))
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
                find_in_expr(r, scope, out, walk);
            }
            for a in args {
                find_in_expr(a, scope, out, walk);
            }
        }
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                find_in_expr(a, scope, out, walk);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            find_in_expr(lhs, scope, out, walk);
            find_in_expr(rhs, scope, out, walk);
        }
        Expr::Unary { operand, .. }
        | Expr::Cast { operand, .. }
        | Expr::Field {
            object: operand, ..
        }
        | Expr::InstanceOf { value: operand, .. } => {
            find_in_expr(operand, scope, out, walk);
        }
        Expr::Index { array, index, .. } => {
            find_in_expr(array, scope, out, walk);
            find_in_expr(index, scope, out, walk);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            find_in_expr(cond, scope, out, walk);
            find_in_expr(then, scope, out, walk);
            find_in_expr(els, scope, out, walk);
        }
        Expr::IncDec { target, .. } => find_in_expr(target, scope, out, walk),
        Expr::NewArray { dims, init, .. } => {
            for d in dims.iter().flatten() {
                find_in_expr(d, scope, out, walk);
            }
            if let Some(elems) = init {
                for e in elems {
                    find_in_expr(e, scope, out, walk);
                }
            }
        }
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                find_in_expr(e, scope, out, walk);
            }
        }
        Expr::MethodRef { qualifier, .. } => {
            find_in_expr(qualifier, scope, out, walk);
        }
        Expr::Lambda { body, .. } => match body {
            LambdaBody::Expr(e) => find_in_expr(e, scope, out, walk),
            LambdaBody::Block(stmts) => {
                for s in stmts {
                    find_in_stmt(s, scope, out, walk);
                }
            }
        },
        Expr::Assign { target, value, .. } => {
            match target {
                crate::ast::AssignTarget::Index { array, index } => {
                    find_in_expr(array, scope, out, walk);
                    find_in_expr(index, scope, out, walk);
                }
                crate::ast::AssignTarget::Field { object, .. } => {
                    find_in_expr(object, scope, out, walk);
                }
                crate::ast::AssignTarget::Var(_) => {}
            }
            find_in_expr(value, scope, out, walk);
        }
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } | Expr::Super { .. } => {}
    }
}

/// The captures of an anonymous class: its free simple names that are
/// bound as locals in the enclosing scope, with their declared types,
/// sorted by name.
fn captures_of(
    body: &ClassDecl,
    enclosing: &Scope,
    anon: &HashMap<String, ClassDecl>,
    created_in: &HashMap<String, Vec<String>>,
) -> Vec<(String, TypeRef)> {
    let free = free_names_deep(body, anon, created_in);
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
/// The free names of a class AND of every anonymous or lambda class created
/// inside it, transitively.
///
/// A capture set is computed where the class is CREATED, and by then a nested
/// lambda is already a class of its own — its body is not part of the enclosing
/// one, so its free names were invisible. The outer class then captured nothing
/// on the inner's behalf and the inner had nowhere to read the name from:
/// `outer = () -> { inner = () -> field; … }` could not see `field` two levels
/// up, though one level worked. Every level must capture what the levels below
/// it need.
///
/// `created_in` says which classes each body instantiates — the inverse of the
/// owner map, which a first pass over the same bodies already produces.
fn free_names_deep(
    class: &ClassDecl,
    anon: &HashMap<String, ClassDecl>,
    created_in: &HashMap<String, Vec<String>>,
) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut stack = vec![class.name.clone()];
    let mut seen: HashSet<String> = HashSet::new();
    while let Some(name) = stack.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let decl = if name == class.name {
            Some(class)
        } else {
            anon.get(&name)
        };
        let Some(decl) = decl else { continue };
        out.extend(free_names(decl));
        for inner in created_in.get(&name).into_iter().flatten() {
            stack.push(inner.clone());
        }
    }
    out
}

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
    // A FIELD INITIALIZER and an INSTANCE INITIALIZER BLOCK capture too —
    // `new Runner() { int w = captured; { z = captured + 1; } }` is ordinary
    // Java. Walking only the methods left both reading a name that no longer
    // existed, so the class was rejected outright.
    for field in &class.fields {
        if let Some(init) = &field.init {
            free_in_expr(init, &mut fields.clone(), &mut free);
        }
    }
    for block in &class.init_blocks {
        let mut bound = fields.clone();
        for stmt in &block.body {
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
        // (`node.value` captures `node`). A path CONTAINING `this` is an
        // enclosing-instance reference (`Outer.this`, `Outer.this.field`), and
        // its head is a class name — capturing it would give the synthesized
        // class a parameter for a variable that does not exist.
        Expr::Name { path, .. } => {
            if !bound.contains(&path[0]) && !path.iter().any(|segment| segment == "this") {
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
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                free_in_expr(a, bound, free);
            }
        }
        Expr::NewObject { args, outer, .. } => {
            if let Some(outer) = outer {
                free_in_expr(outer, bound, free);
            }
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
        Expr::Assign { target, value, .. } => {
            free_in_target(target, bound, free);
            free_in_expr(value, bound, free);
        }
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
        class,
        args,
        outer,
        span,
        ..
    } = expr
    {
        if let Some(outer) = outer.as_deref_mut() {
            rewrite_expr(outer, captures);
        }
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
                    // `this` is already the right object at either kind of
                    // site: a real class's own instance, and — inside a
                    // lambda, which has none — the instance it captured,
                    // which is exactly what a lambda nested in it reaches.
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
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                f(a);
            }
        }
        Expr::NewObject { args, outer, .. } => {
            if let Some(outer) = outer.as_deref_mut() {
                f(outer);
            }
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
        Expr::Assign { target, value, .. } => {
            match target {
                crate::ast::AssignTarget::Index { array, index } => {
                    f(array);
                    f(index);
                }
                crate::ast::AssignTarget::Field { object, .. } => f(object),
                crate::ast::AssignTarget::Var(_) => {}
            }
            f(value);
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

/// Every assignment target and initialized declaration in `stmts`. Assignment
/// is an EXPRESSION in Java (`while ((n = next()) != null)`, `f(x = 5)`), so
/// this descends into expressions too — otherwise a captured local mutated
/// inside an expression would look effectively final and be wrongly accepted.
/// (Lambda bodies are already hoisted into their own classes by this point.)
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
                if let Some(init) = &d.init {
                    out.initialized.insert(d.name.clone());
                    mutations_in_expr(init, out);
                }
            }
        }
        Stmt::Assign { target, value, .. } => {
            if let crate::ast::AssignTarget::Var(name) = target {
                *out.assigns.entry(name.clone()).or_default() += 1;
            } else {
                mutations_in_target(target, out);
            }
            mutations_in_expr(value, out);
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } | Stmt::Return { value: Some(e), .. } => {
            mutations_in_expr(e, out);
        }
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            mutations_in_expr(cond, out);
            mutations_in_stmt(then, out);
            if let Some(e) = els {
                mutations_in_stmt(e, out);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            mutations_in_expr(cond, out);
            mutations_in_stmt(body, out);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            if let Some(s) = init {
                mutations_in_stmt(s, out);
            }
            if let Some(c) = cond {
                mutations_in_expr(c, out);
            }
            for s in update {
                mutations_in_stmt(s, out);
            }
            mutations_in_stmt(body, out);
        }
        Stmt::ForEach {
            name,
            iterable,
            body,
            ..
        } => {
            // The loop variable arrives INITIALIZED on every pass, exactly
            // like a parameter, so an assignment to it costs it its effective
            // finality and a class capturing it is a compile error. Without
            // this the write looked like the variable's own initializer.
            out.initialized.insert(name.clone());
            mutations_in_expr(iterable, out);
            mutations_in_stmt(body, out);
        }
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
        Stmt::Switch { selector, arms, .. } => {
            mutations_in_expr(selector, out);
            for arm in arms {
                for label in arm.labels.iter().flatten() {
                    mutations_in_expr(label, out);
                }
                mutations_in_stmts(&arm.body, out);
            }
        }
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args {
                mutations_in_expr(a, out);
            }
        }
    }
}

fn mutations_in_target(target: &crate::ast::AssignTarget, out: &mut Mutations) {
    match target {
        crate::ast::AssignTarget::Index { array, index } => {
            mutations_in_expr(array, out);
            mutations_in_expr(index, out);
        }
        crate::ast::AssignTarget::Field { object, .. } => mutations_in_expr(object, out),
        crate::ast::AssignTarget::Var(_) => {}
    }
}

/// Count assignments to bare locals that appear inside an EXPRESSION — a nested
/// assignment (`f(x = 5)`) or an increment used for its value (`a[i++]`).
fn mutations_in_expr(expr: &Expr, out: &mut Mutations) {
    match expr {
        Expr::Assign { target, value, .. } => {
            if let crate::ast::AssignTarget::Var(name) = target {
                *out.assigns.entry(name.clone()).or_default() += 1;
            }
            mutations_in_target(target, out);
            mutations_in_expr(value, out);
        }
        Expr::IncDec { target, .. } => {
            if let Expr::Name { path, .. } = target.as_ref()
                && path.len() == 1
            {
                *out.assigns.entry(path[0].clone()).or_default() += 1;
            }
            mutations_in_expr(target, out);
        }
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                mutations_in_expr(r, out);
            }
            for a in args {
                mutations_in_expr(a, out);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            mutations_in_expr(lhs, out);
            mutations_in_expr(rhs, out);
        }
        Expr::Unary { operand, .. }
        | Expr::Cast { operand, .. }
        | Expr::Field {
            object: operand, ..
        }
        | Expr::InstanceOf { value: operand, .. } => mutations_in_expr(operand, out),
        Expr::Index { array, index, .. } => {
            mutations_in_expr(array, out);
            mutations_in_expr(index, out);
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            mutations_in_expr(cond, out);
            mutations_in_expr(then, out);
            mutations_in_expr(els, out);
        }
        Expr::NewArray { dims, init, .. } => {
            for d in dims.iter().flatten() {
                mutations_in_expr(d, out);
            }
            if let Some(elems) = init {
                for e in elems {
                    mutations_in_expr(e, out);
                }
            }
        }
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                mutations_in_expr(e, out);
            }
        }
        Expr::SuperMethodCall { args, .. } => {
            for a in args {
                mutations_in_expr(a, out);
            }
        }
        Expr::NewObject { args, outer, .. } => {
            if let Some(outer) = outer {
                mutations_in_expr(outer, out);
            }
            for a in args {
                mutations_in_expr(a, out);
            }
        }
        Expr::MethodRef { qualifier, .. } => mutations_in_expr(qualifier, out),
        // A lambda here would be un-desugared, which does not happen at this
        // point; its body is a separate scope regardless.
        Expr::Lambda { .. }
        | Expr::Literal { .. }
        | Expr::Name { .. }
        | Expr::This { .. }
        | Expr::Super { .. } => {}
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
    stmts.iter().any(|s| stmt_uses_outer(s, methods, true))
}

/// The same walk for an anonymous/local class, where a bare `this` is the
/// class's OWN instance and so proves nothing about the enclosing one — only
/// an unqualified call to a method it does not provide itself does.
fn stmts_use_outer_methods(stmts: &[Stmt], methods: &HashSet<String>) -> bool {
    stmts.iter().any(|s| stmt_uses_outer(s, methods, false))
}

fn stmt_uses_outer(stmt: &Stmt, methods: &HashSet<String>, this_counts: bool) -> bool {
    let e = |x: &Expr| expr_uses_outer(x, methods, this_counts);
    let sub = |x: &Stmt| stmt_uses_outer(x, methods, this_counts);
    match stmt {
        Stmt::Block(body) => body
            .iter()
            .any(|s| stmt_uses_outer(s, methods, this_counts)),
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

fn expr_uses_outer(expr: &Expr, methods: &HashSet<String>, this_counts: bool) -> bool {
    if this_counts && matches!(expr, Expr::This { .. }) {
        return true;
    }
    // `Outer.this` (JLS §15.8.4) parses as a name path with `this` in it —
    // `Outer.this.field` continues into `["Outer", "this", "field"]`, so the
    // segment is not always last. It can only mean the enclosing instance, so
    // the class needs the outer link
    // whatever else it references. Missing this is why `Outer.this.field`
    // inside an anonymous or local class was reported as a static context: the
    // `__caturraOuter` chain the walk follows had never been built.
    if let Expr::Name { path, .. } = expr
        && path.iter().skip(1).any(|segment| segment == "this")
    {
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
        hit = hit || expr_uses_outer(e, methods, this_counts);
    });
    hit
}

/// The instance members an anonymous/local class body provides for itself:
/// its own fields and methods, plus everything it inherits from a USER
/// superclass. A name these cover is not a reference to the enclosing
/// instance, whatever the enclosing class happens to declare.
///
/// A library supertype's members are invisible here, so an anonymous subclass
/// of a library class that shadows an enclosing field by inheritance would
/// still capture the outer instance. Nothing in the corpus does that, and the
/// alternative — assuming a shadow whenever the supertype is unknown — would
/// silently drop legitimate enclosing access, the worse failure.
fn provided_members(
    body: &ClassDecl,
    all: &HashMap<String, (HashSet<String>, HashSet<String>)>,
    supers: &HashMap<String, String>,
) -> (HashSet<String>, HashSet<String>) {
    let mut fields: HashSet<String> = body.fields.iter().map(|f| f.name.clone()).collect();
    let mut methods: HashSet<String> = body.methods.iter().map(|m| m.name.clone()).collect();
    // An anonymous class records its supertype in `superclass` whether that is
    // a class or an interface; either way its members are nearer than the
    // enclosing instance's.
    let mut current = body.superclass.clone();
    let mut seen = 0usize;
    while let Some(name) = current {
        seen += 1;
        if seen > all.len() + 1 {
            break; // a cycle would be a compiler bug; do not hang on it
        }
        if let Some((f, m)) = all.get(&name) {
            fields.extend(f.iter().cloned());
            methods.extend(m.iter().cloned());
        }
        current = supers.get(&name).cloned();
    }
    (fields, methods)
}

/// Whether an anonymous or local class body reads a field or calls a method of
/// its ENCLOSING instance — the condition for capturing that instance.
///
/// Unlike the lambda rule this ignores `this` (the class has its own) and
/// subtracts everything the class provides for itself, so an own or inherited
/// member shadows the enclosing one rather than capturing it.
fn class_needs_outer(
    body: &ClassDecl,
    owner_fields: &HashSet<String>,
    owner_methods: &HashSet<String>,
    own_fields: &HashSet<String>,
    own_methods: &HashSet<String>,
) -> bool {
    // `free_names` already excludes the body's own fields and every local.
    if free_names(body)
        .iter()
        .any(|n| owner_fields.contains(n) && !own_fields.contains(n))
    {
        return true;
    }
    // Only the enclosing methods this class does NOT provide itself.
    let reachable: HashSet<String> = owner_methods.difference(own_methods).cloned().collect();
    body.methods
        .iter()
        .any(|m| stmts_use_outer_methods(&m.body, &reachable))
        // A field initializer and an instance initializer block reach the
        // enclosing instance exactly as a method body does.
        || body
            .fields
            .iter()
            .filter_map(|f| f.init.as_ref())
            .any(|init| expr_uses_outer(init, &reachable, false))
        || body
            .init_blocks
            .iter()
            .any(|block| stmts_use_outer_methods(&block.body, &reachable))
}
