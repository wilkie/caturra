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
pub fn desugar_lambdas(units: &mut [(String, CompilationUnit)]) {
    let sams = functional_interfaces(units);
    // Signatures for single-candidate method-argument target typing.
    let methods = method_signatures(units);
    // Constructor signatures per class, for `new T(…, lambda)` target typing.
    let constructors = constructor_signatures(units);
    let static_methods = static_method_names(units);
    let class_names = class_name_set(units);

    // Lambda-class names stay globally unique (one counter), but each
    // synthesized class is appended to the SAME unit its lambda came from, so
    // its `SourceFile` matches the line numbers it carries — otherwise a
    // lambda in a non-first file would get the first file's SourceFile and
    // its breakpoints/stack traces would point at the wrong file.
    let mut counter = 0usize;
    for (_, unit) in units.iter_mut() {
        let mut new_classes: Vec<ClassDecl> = Vec::new();
        for class in &mut unit.classes {
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
            let fields: HashMap<String, TypeRef> = class
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.ty.clone()))
                .collect();
            for (method, ret) in class.methods.iter_mut().zip(return_types) {
                let params: HashMap<String, TypeRef> = method
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone()))
                    .collect();
                let mut ctx = Ctx {
                    sams: &sams,
                    methods: &methods,
                    constructors: &constructors,
                    static_methods: &static_methods,
                    class_names: &class_names,
                    ret: ret.as_ref(),
                    new_classes: &mut new_classes,
                    counter: &mut counter,
                    scope: vec![fields.clone(), params],
                };
                for stmt in &mut method.body {
                    desugar_stmt(stmt, &mut ctx);
                }
            }
            for field in &mut class.fields {
                if let Some(init) = &mut field.init {
                    let mut ctx = Ctx {
                        sams: &sams,
                        methods: &methods,
                        constructors: &constructors,
                        static_methods: &static_methods,
                        class_names: &class_names,
                        ret: None,
                        new_classes: &mut new_classes,
                        counter: &mut counter,
                        scope: vec![HashMap::new()],
                    };
                    desugar_expr(init, Some(&field.ty), &mut ctx);
                }
            }
        }
        // The class iteration's borrow of `unit.classes` is released here, so
        // this unit's synthesized lambda classes can be appended to it.
        unit.classes.append(&mut new_classes);
    }
}

struct Ctx<'a> {
    sams: &'a HashMap<String, Sam>,
    methods: &'a HashMap<String, Vec<Vec<TypeRef>>>,
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
}

impl Ctx<'_> {
    fn lookup(&self, name: &str) -> Option<TypeRef> {
        self.scope.iter().rev().find_map(|f| f.get(name).cloned())
    }
}

/// The functional interfaces in the program: interface name -> its SAM.
fn functional_interfaces(units: &[(String, CompilationUnit)]) -> HashMap<String, Sam> {
    let mut out = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            if !class.is_interface {
                continue;
            }
            let abstract_methods: Vec<&MethodDecl> = class
                .methods
                .iter()
                .filter(|m| m.is_abstract && !m.is_static)
                .collect();
            if abstract_methods.len() == 1 {
                let m = abstract_methods[0];
                out.insert(
                    class.name.clone(),
                    Sam {
                        method: m.name.clone(),
                        params: m.params.iter().map(|p| p.ty.clone()).collect(),
                        ret: m.return_type.clone(),
                    },
                );
            }
        }
    }
    out
}

/// Method name -> the parameter-type lists of each declaration, for
/// single-candidate target typing of a lambda argument.
/// Class name -> its declared static method names.
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
        "ArrayList",
    ] {
        set.insert(String::from(lib));
    }
    set
}

/// Whether `method` is a static method of the library type `class`
/// (a curated set; used only for method-reference disambiguation).
fn is_library_static(method: &str) -> bool {
    matches!(
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
                if let Some(frame) = ctx.scope.last_mut() {
                    frame.insert(d.name.clone(), ty.clone());
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
        Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {}
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
        crate::ast::AssignTarget::Field { .. } => None,
    }
}

#[allow(clippy::too_many_lines)] // one arm per expression kind
fn desugar_expr(expr: &mut Expr, expected: Option<&TypeRef>, ctx: &mut Ctx) {
    // A method reference in a target-typed position becomes a lambda.
    if matches!(expr, Expr::MethodRef { .. }) {
        if let Some(target) = expected
            && let Some(name) = interface_name(target)
            && let Some(sam) = ctx.sams.get(name).cloned()
        {
            *expr = method_ref_to_lambda(expr, &sam, ctx);
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
    // A lambda in a target-typed position: rewrite it.
    if matches!(expr, Expr::Lambda { .. }) {
        if let Some(target) = expected
            && let Some(name) = interface_name(target)
            && let Some(sam) = ctx.sams.get(name).cloned()
        {
            let replacement = build_lambda_class(expr, name, &sam, ctx);
            *expr = replacement;
        }
        return;
    }
    match expr {
        Expr::Call {
            receiver,
            method,
            args,
            ..
        } => {
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
            // `list.forEach(x -> ...)` / `list.removeIf(x -> ...)`: a single
            // lambda whose parameter type is the receiver's element type. The
            // erased SAM is `__Consumer` (void) or `__Predicate` (boolean).
            if matches!(method.as_str(), "forEach" | "removeIf" | "replaceAll")
                && args.len() == 1
                && matches!(&args[0], Expr::Lambda { params, .. } if params.len() == 1)
                && let Some(r) = receiver.as_deref()
                && let Some(elem) = list_elem_type(r, ctx)
            {
                let object = TypeRef::Named(String::from("Object"));
                let (iface, sam, ret) = match method.as_str() {
                    "forEach" => ("__Consumer", "accept", TypeRef::Void),
                    "removeIf" => ("__Predicate", "test", TypeRef::Boolean),
                    // `UnaryOperator<E>` — `E apply(E)`, erased to `Object
                    // apply(Object)`. The result is boxed on return.
                    _ => ("__UnaryOperator", "apply", object),
                };
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
            // A stream op with a lambda: the parameter type is the stream's
            // current element, walked back through the pipeline to `.stream()`.
            // (After a `map` the element is erased to `Object`.)
            if let Some(stream_receiver) = receiver.as_deref()
                && let Some(elem) = stream_elem_type(stream_receiver, ctx)
            {
                let object = TypeRef::Named(String::from("Object"));
                let single = match method.as_str() {
                    "filter" | "anyMatch" | "allMatch" | "noneMatch" => {
                        Some(("__Predicate", "test", TypeRef::Boolean))
                    }
                    "map" | "mapToObj" | "mapToInt" | "mapToLong" | "mapToDouble" => {
                        Some(("__UnaryOperator", "apply", object))
                    }
                    "forEach" | "forEachOrdered" | "peek" => {
                        Some(("__Consumer", "accept", TypeRef::Void))
                    }
                    _ => None,
                };
                if let Some((iface, sam, ret)) = single
                    && args.len() == 1
                    && matches!(&args[0], Expr::Lambda { params, .. } if params.len() == 1)
                {
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
            // Single-candidate method argument target typing.
            let param_types = ctx
                .methods
                .get(method)
                .filter(|sigs| sigs.len() == 1 && sigs[0].len() == args.len())
                .map(|sigs| sigs[0].clone());
            for (index, arg) in args.iter_mut().enumerate() {
                let expected = param_types.as_ref().map(|types| types[index].clone());
                desugar_expr(arg, expected.as_ref(), ctx);
            }
        }
        Expr::NewObject {
            class,
            type_args,
            args,
            ..
        } => {
            // A comparator lambda argument to a sorted collection's constructor
            // (`TreeSet`/`TreeMap`/`PriorityQueue`): its parameters are the
            // element / key type, read from the `new`'s own type arguments or,
            // for a diamond, the declaration target it initializes. For a
            // `PriorityQueue(capacity, cmp)` the lambda is the last argument.
            if matches!(class.as_str(), "TreeSet" | "TreeMap" | "PriorityQueue")
                && matches!(args.last(), Some(Expr::Lambda { params, .. }) if params.len() == 2)
                && let Some(elem) = type_args
                    .first()
                    .cloned()
                    .or_else(|| expected.and_then(sorted_ctor_elem))
            {
                let last = args.len() - 1;
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
        Expr::Unary { operand, .. }
        | Expr::Cast { operand, .. }
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
        Expr::NewArray { dims, init, .. } => {
            for d in dims.iter_mut().flatten() {
                desugar_expr(d, None, ctx);
            }
            if let Some(elems) = init {
                for e in elems {
                    desugar_expr(e, None, ctx);
                }
            }
        }
        Expr::ArrayLiteral { elements, .. } => {
            for e in elements {
                desugar_expr(e, None, ctx);
            }
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
fn method_ref_to_lambda(expr: &Expr, sam: &Sam, ctx: &Ctx) -> Expr {
    let Expr::MethodRef {
        qualifier,
        method,
        span,
    } = expr
    else {
        unreachable!("guarded by caller");
    };
    let span = *span;
    let arity = sam.params.len();
    let param_names: Vec<String> = (0..arity).map(|i| format!("__p{i}")).collect();
    let name_expr = |name: &str| Expr::Name {
        path: vec![name.to_owned()],
        span,
    };

    // Is the qualifier a bare class name?
    let qualifier_class = match qualifier.as_ref() {
        Expr::Name { path, .. } if path.len() == 1 && ctx.class_names.contains(&path[0]) => {
            Some(path[0].clone())
        }
        _ => None,
    };

    let call: Expr = if method == "new" {
        // Constructor reference: `new Type(p0, ...)`.
        let class = qualifier_type_name(qualifier);
        Expr::NewObject {
            class,
            type_args: Vec::new(),
            args: param_names.iter().map(|n| name_expr(n)).collect(),
            span,
        }
    } else if let Some(class) = qualifier_class {
        let is_static = ctx
            .static_methods
            .get(&class)
            .is_some_and(|set| set.contains(method))
            || (!ctx_user_class(ctx, &class) && is_library_static(method));
        if is_static {
            // `Type.method(p0, ...)`.
            Expr::Call {
                receiver: Some(Box::new(name_expr(&class))),
                method: method.clone(),
                args: param_names.iter().map(|n| name_expr(n)).collect(),
                span,
            }
        } else {
            // Unbound instance: `p0.method(p1, ...)`.
            Expr::Call {
                receiver: Some(Box::new(name_expr(&param_names[0]))),
                method: method.clone(),
                args: param_names[1..].iter().map(|n| name_expr(n)).collect(),
                span,
            }
        }
    } else {
        // Bound instance: `qualifier.method(p0, ...)`.
        Expr::Call {
            receiver: Some(qualifier.clone()),
            method: method.clone(),
            args: param_names.iter().map(|n| name_expr(n)).collect(),
            span,
        }
    };

    let params = param_names
        .into_iter()
        .map(|name| crate::ast::LambdaParam { name, ty: None })
        .collect();
    Expr::Lambda {
        params,
        body: LambdaBody::Expr(Box::new(call)),
        span,
    }
}

fn ctx_user_class(ctx: &Ctx, class: &str) -> bool {
    ctx.static_methods.contains_key(class)
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

/// The declared key and value types of a `Map`/`HashMap` receiver, read
/// syntactically from the local, parameter or field it names. `getMap()
/// .forEach(...)` has no declaration to read, so its lambda is left in place
/// and codegen reports the honest "lambdas are not supported" it always did.
fn map_type_args(receiver: &Expr, ctx: &Ctx) -> Option<(TypeRef, TypeRef)> {
    let ty = match receiver {
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
        _ => return None,
    };
    let TypeRef::Generic { base, args } = ty else {
        return None;
    };
    if !matches!(
        base.as_str(),
        "Map" | "HashMap" | "TreeMap" | "SortedMap" | "NavigableMap"
    ) || args.len() != 2
    {
        return None;
    }
    Some((args[0].clone(), args[1].clone()))
}

/// The comparator element of a `TreeSet<E>` (`E`) or `TreeMap<K, V>` (`K`, the
/// key it orders) declaration target — for a diamond `new TreeSet<>(cmp)`
/// whose element type lives on the left-hand side.
fn sorted_ctor_elem(target: &TypeRef) -> Option<TypeRef> {
    let TypeRef::Generic { base, args } = target else {
        return None;
    };
    matches!(
        base.as_str(),
        "TreeSet"
            | "SortedSet"
            | "NavigableSet"
            | "TreeMap"
            | "SortedMap"
            | "NavigableMap"
            | "PriorityQueue"
    )
    .then(|| args.first().cloned())
    .flatten()
}

/// The element type `E` of a `Comparator<E>` target type, for casting a
/// comparator lambda's two parameters. `null` for anything else.
fn comparator_target_elem(target: &TypeRef) -> Option<TypeRef> {
    let TypeRef::Generic { base, args } = target else {
        return None;
    };
    (matches!(base.as_str(), "Comparator" | "java.util.Comparator") && args.len() == 1)
        .then(|| args[0].clone())
}

/// The current element type of a stream-pipeline receiver, for typing a stream
/// lambda's parameter. `X.stream()` yields the collection `X`'s element; the
/// element-preserving ops (`filter`/`sorted`/`distinct`/`limit`/`skip`/`peek`)
/// recurse into the prior stage; `map` erases it to `Object`.
fn stream_elem_type(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    let Expr::Call {
        receiver: Some(prev),
        method,
        args,
        ..
    } = receiver
    else {
        return None;
    };
    if method == "stream" && args.is_empty() {
        return list_elem_type(prev, ctx);
    }
    // `IntStream.range(a, b)` / `rangeClosed(a, b)` — a source of `int`s.
    if matches!(method.as_str(), "range" | "rangeClosed")
        && matches!(prev.as_ref(), Expr::Name { path, .. } if path.len() == 1 && path[0] == "IntStream")
    {
        return Some(TypeRef::Int);
    }
    match method.as_str() {
        "filter" | "sorted" | "distinct" | "limit" | "skip" | "peek" => stream_elem_type(prev, ctx),
        // `mapToInt` produces an int stream; `map`/`mapToObj` an erased one.
        "mapToInt" => Some(TypeRef::Int),
        "map" | "mapToObj" => Some(TypeRef::Named(String::from("Object"))),
        _ => None,
    }
}

/// The declared element type of a `List`/`ArrayList`/`Set`/`Collection`
/// receiver, read syntactically from the local, parameter, or field it names
/// — the same shape as `map_type_args`, for a single type argument.
fn list_elem_type(receiver: &Expr, ctx: &Ctx) -> Option<TypeRef> {
    // A `map.keySet()`/`values()` view: the element is the map's key or value
    // type. `entrySet()` yields `Map.Entry`, whose lambda parameter typing
    // differs, so it is not handled here.
    if let Expr::Call {
        receiver: Some(map),
        method,
        args,
        ..
    } = receiver
        && args.is_empty()
        && let Some((key, value)) = map_type_args(map, ctx)
    {
        return match method.as_str() {
            "keySet" => Some(key),
            "values" => Some(value),
            _ => None,
        };
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
        _ => return None,
    };
    let TypeRef::Generic { base, args } = ty else {
        return None;
    };
    let is_collection = matches!(
        base.as_str(),
        "ArrayList"
            | "List"
            | "Set"
            | "HashSet"
            | "TreeSet"
            | "SortedSet"
            | "NavigableSet"
            | "LinkedList"
            | "Queue"
            | "Deque"
            | "PriorityQueue"
            | "Collection"
    );
    (is_collection && args.len() == 1).then(|| args[0].clone())
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
    let Expr::Lambda { params, body, span } = lambda else {
        unreachable!("guarded by caller");
    };
    let span = *span;
    *ctx.counter += 1;
    let name = format!("Lambda${}", ctx.counter);

    let object = || TypeRef::Named(String::from("Object"));
    let erased: Vec<Param> = (0..elem_types.len())
        .map(|i| Param {
            ty: object(),
            name: format!("__caturraArg{i}"),
            is_varargs: false,
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
    let mut method_body: Vec<Stmt> = elem_types
        .iter()
        .enumerate()
        .map(|(i, ty)| unwrap(ty, format!("__caturraArg{i}"), &params[i].name))
        .collect();

    let is_void = matches!(ret, TypeRef::Void);
    match std::mem::replace(body, LambdaBody::Block(Vec::new())) {
        LambdaBody::Expr(mut e) => {
            desugar_expr(&mut e, None, ctx);
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

    ctx.new_classes.push(ClassDecl {
        name: name.clone(),
        is_public: false,
        is_nested: false,
        enclosing: None,
        superclass: None,
        interfaces: vec![interface.to_owned()],
        is_abstract: false,
        is_interface: false,
        is_enum: false,
        is_anonymous: true,
        type_params: Vec::new(),
        fields: Vec::new(),
        methods: vec![MethodDecl {
            name: method.to_owned(),
            is_static: false,
            is_public: true,
            is_private: false,
            is_constructor: false,
            is_abstract: false,
            type_params: Vec::new(),
            return_type: ret.clone(),
            params: erased,
            body: method_body,
            annotations: Vec::new(),
            span,
        }],
        init_blocks: Vec::new(),
        nested: Vec::new(),
        span,
    });

    Expr::NewObject {
        class: name,
        type_args: Vec::new(),
        args: Vec::new(),
        span,
    }
}

/// Build the anonymous class for a lambda and the `new` expression that
/// replaces it. Also recurses into the lambda body first so nested
/// lambdas are handled.
fn build_lambda_class(lambda: &mut Expr, interface: &str, sam: &Sam, ctx: &mut Ctx) -> Expr {
    let Expr::Lambda { params, body, span } = lambda else {
        unreachable!("guarded by caller");
    };
    let span = *span;
    *ctx.counter += 1;
    let name = format!("Lambda${}", ctx.counter);

    // The synthesized method takes the SAM's parameter types with the
    // lambda's parameter names.
    let method_params: Vec<Param> = params
        .iter()
        .zip(&sam.params)
        .map(|(p, ty)| Param {
            ty: ty.clone(),
            name: p.name.clone(),
            is_varargs: false,
        })
        .collect();

    // The body: an expression lambda becomes `return e;` (or `e;` when
    // the SAM is void); a block lambda's statements are used directly.
    let method_body = match std::mem::replace(body, LambdaBody::Block(Vec::new())) {
        LambdaBody::Expr(mut e) => {
            desugar_expr(&mut e, None, ctx);
            if matches!(sam.ret, TypeRef::Void) {
                vec![Stmt::Expr(*e)]
            } else {
                vec![Stmt::Return {
                    value: Some(*e),
                    span,
                }]
            }
        }
        LambdaBody::Block(mut stmts) => {
            for s in &mut stmts {
                desugar_stmt(s, ctx);
            }
            stmts
        }
    };

    let method = MethodDecl {
        name: sam.method.clone(),
        is_static: false,
        is_public: true,
        is_private: false,
        is_constructor: false,
        is_abstract: false,
        type_params: Vec::new(),
        return_type: sam.ret.clone(),
        params: method_params,
        body: method_body,
        annotations: Vec::new(),
        span,
    };

    // The target is known to be a functional interface (it is in `sams`),
    // so record it in the `implements` slot directly. Parking it in
    // `superclass` for codegen to reclassify relies on the interface's
    // `is_interface` flag already being set, which fails when the interface
    // lives in a later compilation unit (e.g. the injected `ActionListener`)
    // than the synthesized lambda class.
    ctx.new_classes.push(ClassDecl {
        name: name.clone(),
        is_public: false,
        is_nested: false,
        enclosing: None,
        superclass: None,
        interfaces: vec![interface.to_owned()],
        is_abstract: false,
        is_interface: false,
        is_enum: false,
        is_anonymous: true,
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
            .find(|c| c.binary_name.starts_with("Lambda$"))
            .expect("a synthesized lambda class");
        assert_eq!(
            source_file_of(&lambda.class_file).as_deref(),
            Some("Helper.java"),
        );
    }
}
