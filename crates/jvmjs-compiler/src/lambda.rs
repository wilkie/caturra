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

    let mut new_classes: Vec<ClassDecl> = Vec::new();
    let mut counter = 0usize;
    for (_, unit) in units.iter_mut() {
        for class in &mut unit.classes {
            let return_types: Vec<Option<TypeRef>> = class
                .methods
                .iter()
                .map(|m| match &m.return_type {
                    TypeRef::Void => None,
                    other => Some(other.clone()),
                })
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
                    ret: ret.as_ref(),
                    new_classes: &mut new_classes,
                    counter: &mut counter,
                    scope: vec![params],
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
                        ret: None,
                        new_classes: &mut new_classes,
                        counter: &mut counter,
                        scope: vec![HashMap::new()],
                    };
                    desugar_expr(init, Some(&field.ty), &mut ctx);
                }
            }
        }
    }

    if !new_classes.is_empty() {
        // Append synthesized lambda classes to the first unit.
        if let Some((_, unit)) = units.first_mut() {
            unit.classes.append(&mut new_classes);
        }
    }
}

struct Ctx<'a> {
    sams: &'a HashMap<String, Sam>,
    methods: &'a HashMap<String, Vec<Vec<TypeRef>>>,
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
fn method_signatures(units: &[(String, CompilationUnit)]) -> HashMap<String, Vec<Vec<TypeRef>>> {
    let mut out: HashMap<String, Vec<Vec<TypeRef>>> = HashMap::new();
    for (_, unit) in units {
        for class in &unit.classes {
            for method in &class.methods {
                if method.is_constructor {
                    continue;
                }
                out.entry(method.name.clone())
                    .or_default()
                    .push(method.params.iter().map(|p| p.ty.clone()).collect());
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

fn desugar_expr(expr: &mut Expr, expected: Option<&TypeRef>, ctx: &mut Ctx) {
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
        Expr::NewObject { args, .. } | Expr::SuperMethodCall { args, .. } => {
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
        Expr::Lambda { .. } | Expr::Literal { .. } | Expr::Name { .. } | Expr::This { .. } => {}
    }
}

/// The simple interface name of a target type (`Named`/`Generic`).
fn interface_name(ty: &TypeRef) -> Option<&str> {
    match ty {
        TypeRef::Named(name) | TypeRef::Generic { base: name, .. } => Some(name.as_str()),
        _ => None,
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
        span,
    };

    ctx.new_classes.push(ClassDecl {
        name: name.clone(),
        superclass: Some(interface.to_owned()),
        interfaces: Vec::new(),
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
