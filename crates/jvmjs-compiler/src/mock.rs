//! `EasyMock` partial-mock support.
//!
//! The CSA neighborhood validators use `EasyMock` to check that a student's
//! method makes the right internal calls (e.g. `moveFast()` calls `move()`
//! three times). Real `EasyMock` generates a subclass at runtime; jvmjs has no
//! runtime bytecode generation, but the mocked method names are compile-time
//! string literals, so the compiler can do it instead.
//!
//! This pass rewrites each
//! `partialMockBuilder(T.class).addMockedMethod("m")….createMock()` chain into
//! `new __EMock_T_m()` and generates that subclass — overriding the mocked
//! methods to route through the bundled `__EMockEngine`
//! (record / replay / verify lives in `stdlib/easymock.java`).

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::ast::{CompilationUnit, Expr, Literal, Stmt, TypeRef};

/// A partial mock to generate: a subclass of `target` overriding `methods`.
/// `ctor_arity` is the number of `withConstructor(...)` arguments (0 = default
/// constructor).
pub struct MockSpec {
    target: String,
    methods: Vec<String>,
    ctor_arity: usize,
    /// `createMock(T.class)` — mock *every* method of `T`, not a named subset.
    full: bool,
}

fn mock_class_name(target: &str, methods: &[String], ctor_arity: usize, full: bool) -> String {
    let base = if full {
        format!("__EMock_{target}_all")
    } else {
        format!("__EMock_{}_{}", target, methods.join("_"))
    };
    if ctor_arity == 0 {
        base
    } else {
        format!("{base}_c{ctor_arity}")
    }
}

/// Rewrite mock-builder chains in the non-bundle units and return the source
/// of the mock subclasses to compile (empty if no mocks are used).
pub fn rewrite(units: &mut [(String, CompilationUnit)]) -> Option<String> {
    // Method signatures across every class, for generating overrides that
    // match (return type + arity) — the mocked method may be inherited from a
    // library base class such as the bundled `Painter`.
    let sigs = collect_signatures(units);

    let mut specs: Vec<MockSpec> = Vec::new();
    for (path, unit) in units.iter_mut() {
        if path.starts_with('<') {
            continue; // never rewrite injected bundles
        }
        for class in &mut unit.classes {
            for method in &mut class.methods {
                for stmt in &mut method.body {
                    rewrite_stmt(stmt, &mut specs);
                }
            }
        }
    }
    if specs.is_empty() {
        return None;
    }

    // Deduplicate by generated class name.
    let mut seen = BTreeSet::new();
    let mut source = String::new();
    for spec in &specs {
        let name = mock_class_name(&spec.target, &spec.methods, spec.ctor_arity, spec.full);
        if !seen.insert(name.clone()) {
            continue;
        }
        emit_mock_class(&mut source, &name, spec, &sigs);
    }
    Some(source)
}

/// One class's methods plus its superclass, so a mocked method can be looked
/// up in the *target's* hierarchy (a same-named method on an unrelated class,
/// e.g. an enum's `turnLeft`, must not be picked).
struct ClassSigs {
    superclass: Option<String>,
    methods: Vec<Sig>,
    /// Parameter-type lists for each declared constructor (for
    /// `withConstructor` forwarding).
    constructors: Vec<Vec<TypeRef>>,
}

/// (method, return type, parameter types).
struct Sig {
    method: String,
    ret: TypeRef,
    params: Vec<TypeRef>,
}

fn collect_signatures(units: &[(String, CompilationUnit)]) -> Vec<(String, ClassSigs)> {
    let mut classes = Vec::new();
    for (_, unit) in units {
        for class in &unit.classes {
            let methods = class
                .methods
                .iter()
                .filter(|m| !m.is_constructor)
                .map(|m| Sig {
                    method: m.name.clone(),
                    ret: m.return_type.clone(),
                    params: m.params.iter().map(|p| p.ty.clone()).collect(),
                })
                .collect();
            let constructors = class
                .methods
                .iter()
                .filter(|m| m.is_constructor)
                .map(|m| m.params.iter().map(|p| p.ty.clone()).collect())
                .collect();
            classes.push((
                class.name.clone(),
                ClassSigs {
                    superclass: class.superclass.clone(),
                    methods,
                    constructors,
                },
            ));
        }
    }
    classes
}

/// All overloads of `method` in `target`'s hierarchy, deduplicated by
/// parameter signature (a subclass override shadows the superclass one). Used
/// so mocking a name covers every overload — `canMove()` and `canMove(String)`
/// are mocked separately, matching per-signature `addMockedMethod` calls.
#[allow(clippy::assigning_clones)] // `current` is `None` after `take`
fn find_all_sigs<'a>(
    classes: &'a [(String, ClassSigs)],
    target: &str,
    method: &str,
) -> Vec<&'a Sig> {
    let mut result: Vec<&Sig> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut current = Some(target.to_owned());
    while let Some(name) = current.take() {
        let Some(entry) = classes.iter().find(|(n, _)| *n == name) else {
            break;
        };
        for sig in entry.1.methods.iter().filter(|s| s.method == method) {
            let key: String = sig
                .params
                .iter()
                .map(type_src)
                .collect::<Vec<_>>()
                .join(",");
            if seen.insert(key) {
                result.push(sig);
            }
        }
        current = entry.1.superclass.clone();
    }
    result
}

/// Every distinct method name in `target`'s hierarchy (for a full mock).
#[allow(clippy::assigning_clones)] // `current` is `None` after `take`
fn all_method_names(classes: &[(String, ClassSigs)], target: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut current = Some(target.to_owned());
    while let Some(name) = current.take() {
        let Some(entry) = classes.iter().find(|(n, _)| *n == name) else {
            break;
        };
        for sig in &entry.1.methods {
            if seen.insert(sig.method.clone()) {
                names.push(sig.method.clone());
            }
        }
        current = entry.1.superclass.clone();
    }
    names
}

/// The target's constructor with the fewest parameters (walking superclasses),
/// for a full mock's forwarding no-arg constructor. `None` means no declared
/// constructor (an implicit `super()` suffices).
#[allow(clippy::assigning_clones)] // `current` is `None` after `take`
fn simplest_ctor<'a>(classes: &'a [(String, ClassSigs)], target: &str) -> Option<&'a Vec<TypeRef>> {
    let mut current = Some(target.to_owned());
    while let Some(name) = current.take() {
        let entry = classes.iter().find(|(n, _)| *n == name)?;
        if let Some(params) = entry.1.constructors.iter().min_by_key(|p| p.len()) {
            return Some(params);
        }
        current = entry.1.superclass.clone();
    }
    None
}

/// A default literal for a parameter type, for a forwarded `super(...)` call.
fn default_value(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Int | TypeRef::Short | TypeRef::Byte | TypeRef::Long => "0".into(),
        TypeRef::Double | TypeRef::Float => "0.0".into(),
        TypeRef::Boolean => "false".into(),
        TypeRef::Char => "(char) 0".into(),
        // Reference types (including arrays): null.
        _ => "null".into(),
    }
}

/// Find a constructor of `target` (walking superclasses) with `arity`
/// parameters, returning its parameter types.
#[allow(clippy::assigning_clones)] // `current` is `None` after `take`
fn find_ctor<'a>(
    classes: &'a [(String, ClassSigs)],
    target: &str,
    arity: usize,
) -> Option<&'a Vec<TypeRef>> {
    let mut current = Some(target.to_owned());
    while let Some(name) = current.take() {
        let entry = classes.iter().find(|(n, _)| *n == name)?;
        if let Some(params) = entry.1.constructors.iter().find(|p| p.len() == arity) {
            return Some(params);
        }
        current = entry.1.superclass.clone();
    }
    None
}

/// Emit `class <name> extends <target> implements __Mockable { … }` overriding
/// each mocked method to route through the engine.
fn emit_mock_class(out: &mut String, name: &str, spec: &MockSpec, sigs: &[(String, ClassSigs)]) {
    let _ = writeln!(
        out,
        "class {name} extends {} implements __Mockable {{",
        spec.target
    );
    out.push_str("  __EMockEngine __e = new __EMockEngine();\n");
    out.push_str("  public __EMockEngine __engine() { return __e; }\n");
    // A full `createMock(T.class)` is constructed with no arguments, but T may
    // have no no-arg constructor (real EasyMock bypasses it). Forward to T's
    // simplest constructor with default argument values.
    if spec.full
        && let Some(params) = simplest_ctor(sigs, &spec.target)
        && !params.is_empty()
    {
        let args: Vec<String> = params.iter().map(default_value).collect();
        let _ = writeln!(out, "  public {name}() {{ super({}); }}", args.join(", "));
    }
    // `withConstructor(args)` — forward to the matching super constructor.
    if spec.ctor_arity > 0 {
        let types = find_ctor(sigs, &spec.target, spec.ctor_arity).cloned();
        let params: Vec<String> = (0..spec.ctor_arity)
            .map(|i| {
                let ty = types.as_ref().and_then(|t| t.get(i));
                let src = ty.map_or_else(|| String::from("Object"), type_src);
                format!("{src} __c{i}")
            })
            .collect();
        let args: Vec<String> = (0..spec.ctor_arity).map(|i| format!("__c{i}")).collect();
        let _ = writeln!(
            out,
            "  public {name}({}) {{ super({}); }}",
            params.join(", "),
            args.join(", ")
        );
    }
    // A full `createMock(T.class)` mocks every method name in T's hierarchy.
    let method_names: Vec<String> = if spec.full {
        all_method_names(sigs, &spec.target)
    } else {
        spec.methods.clone()
    };
    for method in &method_names {
        // Mock every overload of the name (a library method not in `sigs`
        // falls back to a single no-arg override).
        let overloads = find_all_sigs(sigs, &spec.target, method);
        let signatures: Vec<(TypeRef, Vec<TypeRef>)> = if overloads.is_empty() {
            vec![(TypeRef::Void, Vec::new())]
        } else {
            overloads
                .iter()
                .map(|s| (s.ret.clone(), s.params.clone()))
                .collect()
        };
        // Overloaded names need a per-signature engine key so each overload
        // keeps its own record/replay queue; a lone overload uses the plain
        // name (unchanged behavior for the single-overload common case).
        let multi = signatures.len() > 1;
        for (ret, param_types) in signatures {
            let params: Vec<String> = param_types
                .iter()
                .enumerate()
                .map(|(i, ty)| format!("{} __a{i}", type_src(ty)))
                .collect();
            let key = if multi {
                format!(
                    "{method}({})",
                    param_types
                        .iter()
                        .map(type_src)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            } else {
                method.clone()
            };
            let ret_src = type_src(&ret);
            let body = mock_body(&key, &ret);
            let _ = writeln!(
                out,
                "  public {ret_src} {method}({}) {{ {body} }}",
                params.join(", ")
            );
        }
    }
    out.push_str("}\n");
}

fn mock_body(key: &str, ret: &TypeRef) -> String {
    match ret {
        TypeRef::Void => format!("__e.consume(\"{key}\");"),
        TypeRef::Boolean => format!("return __e.consume(\"{key}\") != 0;"),
        TypeRef::Int | TypeRef::Short | TypeRef::Byte => {
            format!("return __e.consume(\"{key}\");")
        }
        TypeRef::Char => format!("return (char) __e.consume(\"{key}\");"),
        TypeRef::Long => format!("return (long) __e.consume(\"{key}\");"),
        TypeRef::Double | TypeRef::Float => format!("__e.consume(\"{key}\"); return 0;"),
        // Reference return: count the call, return null.
        _ => format!("__e.consume(\"{key}\"); return null;"),
    }
}

fn type_src(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Void => "void".into(),
        TypeRef::Int => "int".into(),
        TypeRef::Double => "double".into(),
        TypeRef::Boolean => "boolean".into(),
        TypeRef::Char => "char".into(),
        TypeRef::Long => "long".into(),
        TypeRef::Float => "float".into(),
        TypeRef::Short => "short".into(),
        TypeRef::Byte => "byte".into(),
        TypeRef::Named(name) => name.clone(),
        TypeRef::Generic { base, .. } => base.clone(),
        TypeRef::Array(inner) => format!("{}[]", type_src(inner)),
    }
}

// ----- chain extraction + AST rewrite -----

/// If `expr` is a `partialMockBuilder(T.class)….createMock()` chain, return
/// its `MockSpec` and the `withConstructor(...)` arguments (empty if none).
fn extract_spec(expr: &Expr) -> Option<(MockSpec, Vec<Expr>)> {
    let Expr::Call {
        method, receiver, ..
    } = expr
    else {
        return None;
    };
    if method != "createMock" {
        return None;
    }
    // Full mock: `createMock(T.class)` — no builder chain, mock all methods.
    if receiver.is_none() {
        let Expr::Call { args, .. } = expr else {
            return None;
        };
        let target = class_literal_name(args.first()?)?;
        return Some((
            MockSpec {
                target,
                methods: Vec::new(),
                ctor_arity: 0,
                full: true,
            },
            Vec::new(),
        ));
    }
    let mut methods = Vec::new();
    let mut ctor_args: Vec<Expr> = Vec::new();
    let mut current = receiver.as_deref()?;
    loop {
        match current {
            // `withConstructor(args)` — the mock runs this super constructor.
            Expr::Call {
                method,
                receiver: Some(inner),
                args,
                ..
            } if method == "withConstructor" => {
                ctor_args.clone_from(args);
                current = inner;
            }
            // `addMockedMethod("m")` (one name; may be followed by a Class[]
            // signature) or `addMockedMethods("m1", "m2", …)` (varargs).
            Expr::Call {
                method,
                receiver: Some(inner),
                args,
                ..
            } if method == "addMockedMethod" || method == "addMockedMethods" => {
                let take = if method == "addMockedMethod" {
                    1
                } else {
                    args.len()
                };
                for arg in args.iter().take(take) {
                    if let Expr::Literal {
                        value: Literal::Str(name),
                        ..
                    } = arg
                    {
                        methods.push(name.clone());
                    }
                }
                current = inner;
            }
            Expr::Call {
                method,
                receiver: None,
                args,
                ..
            } if method == "partialMockBuilder" => {
                let target = class_literal_name(args.first()?)?;
                methods.sort();
                methods.dedup();
                let ctor_arity = ctor_args.len();
                return Some((
                    MockSpec {
                        target,
                        methods,
                        ctor_arity,
                        full: false,
                    },
                    ctor_args,
                ));
            }
            _ => return None,
        }
    }
}

/// The `T` in a `T.class` expression (`Field { object: Name[T], "class" }`).
fn class_literal_name(expr: &Expr) -> Option<String> {
    let Expr::Field { object, name, .. } = expr else {
        return None;
    };
    if name != "class" {
        return None;
    }
    match object.as_ref() {
        Expr::Name { path, .. } => path.last().cloned(),
        _ => None,
    }
}

fn rewrite_expr(expr: &mut Expr, specs: &mut Vec<MockSpec>) {
    if let Some((spec, ctor_args)) = extract_spec(expr) {
        let span = expr.span();
        let class = mock_class_name(&spec.target, &spec.methods, spec.ctor_arity, spec.full);
        specs.push(spec);
        *expr = Expr::NewObject {
            class,
            type_args: Vec::new(),
            args: ctor_args,
            span,
        };
        return;
    }
    walk_expr_children(expr, &mut |e| rewrite_expr(e, specs));
}

#[allow(clippy::match_same_arms)]
fn rewrite_stmt(stmt: &mut Stmt, specs: &mut Vec<MockSpec>) {
    match stmt {
        Stmt::Block(body) => body.iter_mut().for_each(|s| rewrite_stmt(s, specs)),
        Stmt::LocalDecl { declarators, .. } => {
            for d in declarators {
                if let Some(init) = &mut d.init {
                    rewrite_expr(init, specs);
                }
            }
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => rewrite_expr(e, specs),
        Stmt::Assign { value, .. } => rewrite_expr(value, specs),
        Stmt::Return { value: Some(e), .. } => rewrite_expr(e, specs),
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
        Stmt::If {
            cond, then, els, ..
        } => {
            rewrite_expr(cond, specs);
            rewrite_stmt(then, specs);
            if let Some(e) = els {
                rewrite_stmt(e, specs);
            }
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            rewrite_expr(cond, specs);
            rewrite_stmt(body, specs);
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            if let Some(s) = init {
                rewrite_stmt(s, specs);
            }
            if let Some(c) = cond {
                rewrite_expr(c, specs);
            }
            for s in update {
                rewrite_stmt(s, specs);
            }
            rewrite_stmt(body, specs);
        }
        Stmt::ForEach { iterable, body, .. } => {
            rewrite_expr(iterable, specs);
            rewrite_stmt(body, specs);
        }
        Stmt::Switch { selector, arms, .. } => {
            rewrite_expr(selector, specs);
            for arm in arms {
                for s in &mut arm.body {
                    rewrite_stmt(s, specs);
                }
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            for s in body.iter_mut() {
                rewrite_stmt(s, specs);
            }
            for c in catches {
                for s in &mut c.body {
                    rewrite_stmt(s, specs);
                }
            }
            if let Some(fin) = finally_body {
                for s in fin.iter_mut() {
                    rewrite_stmt(s, specs);
                }
            }
        }
        Stmt::Labeled { body, .. } => rewrite_stmt(body, specs),
        Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
            for a in args.iter_mut() {
                rewrite_expr(a, specs);
            }
        }
    }
}

/// Apply `f` to each direct sub-expression (mirrors the walker in `capture`).
#[allow(clippy::match_same_arms)]
fn walk_expr_children(expr: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    use crate::ast::LambdaBody;
    match expr {
        Expr::Call { receiver, args, .. } => {
            if let Some(r) = receiver {
                f(r);
            }
            args.iter_mut().for_each(&mut *f);
        }
        Expr::SuperMethodCall { args, .. } | Expr::NewObject { args, .. } => {
            args.iter_mut().for_each(&mut *f);
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
                elems.iter_mut().for_each(&mut *f);
            }
        }
        Expr::ArrayLiteral { elements, .. } => elements.iter_mut().for_each(&mut *f),
        Expr::MethodRef { qualifier, .. } => f(qualifier),
        Expr::Lambda { body, .. } => {
            if let LambdaBody::Expr(e) = body {
                f(e);
            }
        }
        Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } => {}
    }
}
