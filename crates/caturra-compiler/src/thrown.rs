//! Checked-exception enforcement (JLS §11.2).
//!
//! Two *rejection* analyses javac performs that caturra did not, found by
//! audit round 6:
//!
//!  - **Unreported exception.** A `throw` of a checked exception, or a call
//!    to a method/constructor that declares one, must sit inside a `try`
//!    whose `catch` covers it or inside a method that declares it —
//!    "unreported exception X; must be caught or declared to be thrown".
//!  - **Never thrown.** A `catch` of a checked exception the try body cannot
//!    throw is dead — "exception X is never thrown in body of corresponding
//!    try statement" (Exception and Throwable are exempt, JLS §11.2.3).
//!
//! **Conservative by construction**, like `flow.rs`: caturra's modeled
//! library is a closed world, so the checked throwers in it can be enumerated
//! exactly — but a call this pass cannot resolve (a chained receiver, an
//! overload it cannot pick) contributes an *unknown* marker instead of a
//! guess. Unknown suppresses BOTH checks around it: no "unreported" error is
//! raised for it, and any enclosing catch is assumed reachable. A missed
//! error is the status quo; a spurious one rejects a valid program.
//!
//! Bundled library units (paths in angle brackets) and synthesized
//! anonymous/local classes are exempt: the bundled Java is trusted, and a
//! lambda's checked-exception contract lives on its functional interface,
//! which caturra erases.

use crate::ast::{CatchClause, ClassDecl, Expr, MethodDecl, Stmt, TypeRef};
use crate::codegen::MethodTable;
use crate::diagnostics::{Diagnostic, SourceSpan};
use caturra_classfile::exceptions as exc;
use std::collections::HashMap;

/// An exception type as this pass tracks it: a library throwable by internal
/// name, or a user class (whose checked-ness comes from its library ancestor).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Exc {
    Lib(&'static str),
    User(String),
}

impl Exc {
    /// The name to print — the simple name, as javac prints an imported type.
    fn display(&self) -> String {
        match self {
            Exc::Lib(internal) => internal.rsplit('/').next().unwrap_or(internal).to_owned(),
            Exc::User(name) => name.clone(),
        }
    }
}

/// What a construct can throw, as far as this pass can see. `unknown` is the
/// safety valve: set whenever something unresolvable might throw.
#[derive(Debug, Default, Clone)]
struct ThrownSet {
    list: Vec<Exc>,
    unknown: bool,
}

impl ThrownSet {
    fn push(&mut self, e: Exc) {
        if !self.list.contains(&e) {
            self.list.push(e);
        }
    }
    fn absorb(&mut self, other: ThrownSet) {
        for e in other.list {
            self.push(e);
        }
        self.unknown |= other.unknown;
    }
}

/// What a name in scope can mean to a `throw` statement.
#[derive(Debug, Clone)]
enum Binding {
    /// A local/parameter with this declared type.
    Declared(TypeRef),
    /// A catch parameter under JLS §11.2.2 precise rethrow: `throw e` throws
    /// only these (the try-body exceptions its clause can catch). `None` when
    /// the body's throw set was unknown — then `throw e` is unknown too.
    Precise(Option<Vec<Exc>>),
}

pub(crate) fn check(
    class: &ClassDecl,
    path: &str,
    table: &MethodTable,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // Bundled units are trusted.
    //
    // A synthesized lambda class is NOT: its body is source a program wrote,
    // and a checked exception escaping it is javac's "unreported exception".
    // Skipping every anonymous class hid that — the class carries the
    // interface's own `throws` on its method, which is exactly what decides
    // whether the body may throw.
    if path.starts_with('<') {
        return;
    }
    // JLS §8.1.2: a generic class may not extend Throwable — the catch clause
    // that would name it cannot check a type argument at run time.
    if !class.type_params.is_empty()
        && let Some(parent) = &class.superclass
        && resolve_exc(parent, table).is_some()
    {
        diagnostics.push(Diagnostic::error(
            path,
            String::from("a generic class may not extend java.lang.Throwable"),
            class.span,
        ));
    }
    for method in &class.methods {
        // JLS §8.4.6: every name in a `throws` clause must be a Throwable.
        for thrown in &method.throws {
            if resolve_exc(thrown, table).is_none() && table.names_a_type(thrown) {
                diagnostics.push(Diagnostic::error(
                    path,
                    format!("incompatible types: {thrown} cannot be converted to Throwable"),
                    method.span,
                ));
            }
        }
        if method.is_abstract {
            continue;
        }
        check_method(class, method, path, table, diagnostics);
    }
    // JLS §11.2.3. A STATIC initializer can declare nothing, so every checked
    // exception in one is unreported. An INSTANCE initializer — a block or a
    // field's initializer expression — runs inside every constructor, so it may
    // throw only what EVERY constructor declares. Both were exempt before, and
    // an initializer was the one place a checked exception could hide.
    let instance_declared = constructor_common_throws(class, table);
    for block in &class.init_blocks {
        let declared = if block.is_static {
            Vec::new()
        } else {
            instance_declared.clone()
        };
        let mut ctx = Ctx {
            class,
            path,
            table,
            diagnostics,
            declared,
            locals: HashMap::new(),
        };
        thrown_of_block(&block.body, &mut Vec::new(), &mut ctx);
    }
    for field in &class.fields {
        let Some(init) = &field.init else {
            continue;
        };
        let declared = if field.is_static {
            Vec::new()
        } else {
            instance_declared.clone()
        };
        let mut ctx = Ctx {
            class,
            path,
            table,
            diagnostics,
            declared,
            locals: HashMap::new(),
        };
        thrown_of_expr(init, &mut Vec::new(), &mut ctx);
    }
}

/// What every constructor of the class declares it may throw — the set an
/// instance initializer is allowed to throw (JLS §11.2.3). A class with no
/// explicit constructor gets the default one, which declares nothing.
fn constructor_common_throws(class: &ClassDecl, table: &MethodTable) -> Vec<Exc> {
    let mut ctors = class.methods.iter().filter(|m| m.is_constructor);
    let Some(first) = ctors.next() else {
        return Vec::new();
    };
    let rest: Vec<&MethodDecl> = ctors.collect();
    first
        .throws
        .iter()
        .filter_map(|name| resolve_exc(name, table))
        .filter(|exc| {
            rest.iter().all(|ctor| {
                ctor.throws
                    .iter()
                    .filter_map(|name| resolve_exc(name, table))
                    .any(|declared| exc_covers(&declared, exc, table))
            })
        })
        .collect()
}

fn check_method(
    class: &ClassDecl,
    method: &MethodDecl,
    path: &str,
    table: &MethodTable,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let declared: Vec<Exc> = method
        .throws
        .iter()
        .filter_map(|name| resolve_exc(name, table))
        .collect();
    let mut locals = HashMap::new();
    // The class's own FIELDS are in scope for every method, and a bare
    // `reader.readLine()` on a field is as much a typed receiver as one on a
    // local. Without them the name fell through to the STATIC path, was not a
    // class either, and the call was taken to throw nothing — so a `catch`
    // around it read as "never thrown in body". A captured variable is a field
    // of the synthesized class, which is how a lambda hit this too.
    for field in &class.fields {
        locals.insert(field.name.clone(), Binding::Declared(field.ty.clone()));
    }
    // A parameter shadows a field of the same name.
    for param in &method.params {
        locals.insert(param.name.clone(), Binding::Declared(param.ty.clone()));
    }
    let mut ctx = Ctx {
        class,
        path,
        table,
        diagnostics,
        declared,
        locals,
    };
    thrown_of_block(&method.body, &mut Vec::new(), &mut ctx);
}

struct Ctx<'a> {
    class: &'a ClassDecl,
    path: &'a str,
    table: &'a MethodTable,
    diagnostics: &'a mut Vec<Diagnostic>,
    declared: Vec<Exc>,
    /// Declared types of names in scope (parameters, locals, catch params).
    /// Flat, last-write-wins: good enough for typing `throw e` and receivers.
    locals: HashMap<String, Binding>,
}

impl Ctx<'_> {
    fn error(&mut self, span: SourceSpan, message: String) {
        self.diagnostics
            .push(Diagnostic::error(self.path, message, span));
    }

    /// Report every checked exception in `set` that no enclosing catch frame
    /// covers and the method does not declare.
    fn report_escapes(&mut self, set: &ThrownSet, handlers: &[Vec<Exc>], span: SourceSpan) {
        let escaped: Vec<Exc> = set
            .list
            .iter()
            .filter(|e| exc_is_checked(e, self.table))
            .filter(|e| {
                !handlers
                    .iter()
                    .any(|frame| frame.iter().any(|c| exc_covers(c, e, self.table)))
            })
            .filter(|e| !self.declared.iter().any(|d| exc_covers(d, e, self.table)))
            .cloned()
            .collect();
        for e in escaped {
            self.error(
                span,
                format!(
                    "unreported exception {}; must be caught or declared to be thrown",
                    e.display()
                ),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

fn thrown_of_block(body: &[Stmt], handlers: &mut Vec<Vec<Exc>>, ctx: &mut Ctx) -> ThrownSet {
    let mut out = ThrownSet::default();
    for stmt in body {
        out.absorb(thrown_of_stmt(stmt, handlers, ctx));
    }
    out
}

#[allow(clippy::too_many_lines)] // one arm per statement kind
fn thrown_of_stmt(stmt: &Stmt, handlers: &mut Vec<Vec<Exc>>, ctx: &mut Ctx) -> ThrownSet {
    match stmt {
        Stmt::Block(body) => thrown_of_block(body, handlers, ctx),
        Stmt::LocalDecl {
            ty, declarators, ..
        } => {
            let mut out = ThrownSet::default();
            for d in declarators {
                ctx.locals
                    .insert(d.name.clone(), Binding::Declared(ty.clone()));
                if let Some(init) = &d.init {
                    out.absorb(thrown_of_expr(init, handlers, ctx));
                }
            }
            out
        }
        Stmt::Expr(e) | Stmt::Throw { value: e, .. } => {
            let mut out = thrown_of_expr(e, handlers, ctx);
            if let Stmt::Throw { value, span } = stmt {
                let mut thrown = throw_types(value, ctx);
                // Report at the throw, then propagate.
                ctx.report_escapes(&thrown, handlers, *span);
                out.absorb(std::mem::take(&mut thrown));
            }
            out
        }
        Stmt::Assign {
            target: _, value, ..
        } => thrown_of_expr(value, handlers, ctx),
        Stmt::If {
            cond, then, els, ..
        } => {
            let mut out = thrown_of_expr(cond, handlers, ctx);
            out.absorb(thrown_of_stmt(then, handlers, ctx));
            if let Some(els) = els {
                out.absorb(thrown_of_stmt(els, handlers, ctx));
            }
            out
        }
        Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
            let mut out = thrown_of_expr(cond, handlers, ctx);
            out.absorb(thrown_of_stmt(body, handlers, ctx));
            out
        }
        Stmt::For {
            init,
            cond,
            update,
            body,
            ..
        } => {
            let mut out = ThrownSet::default();
            if let Some(s) = init {
                out.absorb(thrown_of_stmt(s, handlers, ctx));
            }
            if let Some(cond) = cond {
                out.absorb(thrown_of_expr(cond, handlers, ctx));
            }
            for s in update {
                out.absorb(thrown_of_stmt(s, handlers, ctx));
            }
            out.absorb(thrown_of_stmt(body, handlers, ctx));
            out
        }
        Stmt::ForEach {
            ty,
            name,
            iterable,
            body,
            ..
        } => {
            let mut out = thrown_of_expr(iterable, handlers, ctx);
            ctx.locals
                .insert(name.clone(), Binding::Declared(ty.clone()));
            out.absorb(thrown_of_stmt(body, handlers, ctx));
            out
        }
        Stmt::Labeled { body, .. } => thrown_of_stmt(body, handlers, ctx),
        Stmt::Return { value, .. } => value
            .as_ref()
            .map_or_else(ThrownSet::default, |v| thrown_of_expr(v, handlers, ctx)),
        Stmt::SuperCall { args, span, .. } | Stmt::ThisCall { args, span, .. } => {
            let mut out = ThrownSet::default();
            for a in args {
                out.absorb(thrown_of_expr(a, handlers, ctx));
            }
            // The chained constructor's own throws: superclass or this class.
            if let Stmt::SuperCall { .. } = stmt {
                if let Some(sup) = &ctx.class.superclass {
                    let sup = sup.clone();
                    out.absorb(callee_throws_named(
                        &sup,
                        "<init>",
                        args.len(),
                        *span,
                        handlers,
                        ctx,
                    ));
                }
            } else {
                let own = ctx.class.name.clone();
                out.absorb(callee_throws_named(
                    &own,
                    "<init>",
                    args.len(),
                    *span,
                    handlers,
                    ctx,
                ));
            }
            out
        }
        Stmt::Switch { selector, arms, .. } => {
            let mut out = thrown_of_expr(selector, handlers, ctx);
            for arm in arms {
                out.absorb(thrown_of_block(&arm.body, handlers, ctx));
            }
            out
        }
        // An empty statement throws nothing, like a break or a continue.
        Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Empty(_) => ThrownSet::default(),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => thrown_of_try(body, catches, finally_body.as_deref(), handlers, ctx),
    }
}

fn thrown_of_try(
    body: &[Stmt],
    catches: &[CatchClause],
    finally_body: Option<&[Stmt]>,
    handlers: &mut Vec<Vec<Exc>>,
    ctx: &mut Ctx,
) -> ThrownSet {
    // The body runs under this try's own catch frame.
    let frame: Vec<Exc> = catches
        .iter()
        .flat_map(|c| c.types.iter())
        .filter_map(|t| resolve_typeref(t, ctx.table))
        .collect();
    handlers.push(frame);
    let body_thrown = thrown_of_block(body, handlers, ctx);
    handlers.pop();

    // JLS §11.2.3: a catch of a checked exception the body cannot throw is an
    // error — unless the type is Exception or Throwable, or something in the
    // body was beyond this analysis (`unknown`).
    for clause in catches {
        for t in &clause.types {
            let Some(e) = resolve_typeref(t, ctx.table) else {
                continue;
            };
            if !exc_is_checked(&e, ctx.table)
                || matches!(e, Exc::Lib("java/lang/Exception" | "java/lang/Throwable"))
            {
                continue;
            }
            let reachable = body_thrown.unknown
                || body_thrown.list.iter().any(|thrown| {
                    exc_covers(&e, thrown, ctx.table) || exc_covers(thrown, &e, ctx.table)
                });
            if !reachable {
                ctx.error(
                    clause.span,
                    format!(
                        "exception {} is never thrown in body of corresponding try statement",
                        e.display()
                    ),
                );
            }
        }
    }

    // What escapes the whole statement: the body's exceptions its catches do
    // not cover, plus whatever the catch bodies and finally throw.
    let mut out = ThrownSet {
        unknown: body_thrown.unknown,
        ..ThrownSet::default()
    };
    for thrown in &body_thrown.list {
        let caught = catches.iter().any(|clause| {
            clause
                .types
                .iter()
                .filter_map(|t| resolve_typeref(t, ctx.table))
                .any(|c| exc_covers(&c, thrown, ctx.table))
        });
        if !caught {
            out.push(thrown.clone());
        }
    }
    for clause in catches {
        // JLS §11.2.2 precise rethrow: an (effectively final) catch parameter
        // rethrows only what the body can throw that this clause catches.
        // A REASSIGNED parameter forfeits precise rethrow, and then `throw e`
        // throws its DECLARED type — which the enclosing method must report.
        // Recording "unknown" instead let an under-declared `throws` compile.
        if !body_thrown.unknown
            && is_reassigned(&clause.name, &clause.body)
            && let [declared] = clause.types.as_slice()
        {
            ctx.locals
                .insert(clause.name.clone(), Binding::Declared(declared.clone()));
            out.absorb(thrown_of_block(&clause.body, handlers, ctx));
            continue;
        }
        let precise = if body_thrown.unknown || is_reassigned(&clause.name, &clause.body) {
            None
        } else {
            Some(
                body_thrown
                    .list
                    .iter()
                    .filter(|thrown| {
                        clause
                            .types
                            .iter()
                            .filter_map(|t| resolve_typeref(t, ctx.table))
                            .any(|c| exc_covers(&c, thrown, ctx.table))
                    })
                    .cloned()
                    .collect(),
            )
        };
        ctx.locals
            .insert(clause.name.clone(), Binding::Precise(precise));
        out.absorb(thrown_of_block(&clause.body, handlers, ctx));
    }
    if let Some(finally_body) = finally_body {
        out.absorb(thrown_of_block(finally_body, handlers, ctx));
    }
    out
}

/// Whether the catch parameter is assigned anywhere in the clause body —
/// which forfeits precise rethrow (JLS §11.2.2 requires effectively final).
fn is_reassigned(name: &str, body: &[Stmt]) -> bool {
    fn in_stmt(name: &str, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Assign { target, .. } => {
                matches!(target, crate::ast::AssignTarget::Var(v) if v == name)
            }
            Stmt::Block(body) => body.iter().any(|s| in_stmt(name, s)),
            Stmt::If { then, els, .. } => {
                in_stmt(name, then) || els.as_deref().is_some_and(|e| in_stmt(name, e))
            }
            Stmt::While { body, .. }
            | Stmt::DoWhile { body, .. }
            | Stmt::For { body, .. }
            | Stmt::ForEach { body, .. }
            | Stmt::Labeled { body, .. } => in_stmt(name, body),
            Stmt::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                body.iter().any(|s| in_stmt(name, s))
                    || catches
                        .iter()
                        .any(|c| c.body.iter().any(|s| in_stmt(name, s)))
                    || finally_body
                        .as_ref()
                        .is_some_and(|f| f.iter().any(|s| in_stmt(name, s)))
            }
            Stmt::Switch { arms, .. } => {
                arms.iter().any(|a| a.body.iter().any(|s| in_stmt(name, s)))
            }
            _ => false,
        }
    }
    body.iter().any(|s| in_stmt(name, s))
}

// ---------------------------------------------------------------------------
// Expressions: calls and constructors are the throwers
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_lines)] // one arm per expression kind
fn thrown_of_expr(expr: &Expr, handlers: &mut Vec<Vec<Exc>>, ctx: &mut Ctx) -> ThrownSet {
    match expr {
        Expr::Call {
            receiver,
            method,
            args,
            span,
            ..
        } => {
            let mut out = ThrownSet::default();
            if let Some(r) = receiver {
                out.absorb(thrown_of_expr(r, handlers, ctx));
            }
            for a in args {
                out.absorb(thrown_of_expr(a, handlers, ctx));
            }
            out.absorb(call_throws(
                receiver.as_deref(),
                method,
                args,
                *span,
                handlers,
                ctx,
            ));
            out
        }
        Expr::NewObject {
            class, args, span, ..
        } => {
            let mut out = ThrownSet::default();
            for a in args {
                out.absorb(thrown_of_expr(a, handlers, ctx));
            }
            out.absorb(ctor_throws(class, args, *span, handlers, ctx));
            out
        }
        Expr::SuperMethodCall { args, .. } => {
            let mut out = ThrownSet::default();
            for a in args {
                out.absorb(thrown_of_expr(a, handlers, ctx));
            }
            // A super.m() call: the superclass method's clause.
            out.unknown = true;
            out
        }
        Expr::Binary { lhs, rhs, .. } => {
            let mut out = thrown_of_expr(lhs, handlers, ctx);
            out.absorb(thrown_of_expr(rhs, handlers, ctx));
            out
        }
        Expr::Unary { operand, .. } | Expr::Cast { operand, .. } => {
            thrown_of_expr(operand, handlers, ctx)
        }
        Expr::Ternary {
            cond, then, els, ..
        } => {
            let mut out = thrown_of_expr(cond, handlers, ctx);
            out.absorb(thrown_of_expr(then, handlers, ctx));
            out.absorb(thrown_of_expr(els, handlers, ctx));
            out
        }
        Expr::Index { array, index, .. } => {
            let mut out = thrown_of_expr(array, handlers, ctx);
            out.absorb(thrown_of_expr(index, handlers, ctx));
            out
        }
        Expr::Field { object, .. } => thrown_of_expr(object, handlers, ctx),
        Expr::NewArray { dims, init, .. } => {
            let mut out = ThrownSet::default();
            for d in dims.iter().flatten() {
                out.absorb(thrown_of_expr(d, handlers, ctx));
            }
            for e in init.iter().flatten() {
                out.absorb(thrown_of_expr(e, handlers, ctx));
            }
            out
        }
        Expr::ArrayLiteral { elements, .. } => {
            let mut out = ThrownSet::default();
            for e in elements {
                out.absorb(thrown_of_expr(e, handlers, ctx));
            }
            out
        }
        Expr::InstanceOf { value, .. } | Expr::Assign { value, .. } => {
            thrown_of_expr(value, handlers, ctx)
        }
        Expr::IncDec { .. }
        | Expr::Literal { .. }
        | Expr::Name { .. }
        | Expr::This { .. }
        | Expr::Super { .. }
        | Expr::MethodRef { .. }
        | Expr::Lambda { .. } => ThrownSet::default(),
    }
}

/// The declared throws of a call, resolved as far as a syntactic pass can:
/// bare calls against the current class, `Type.m()` statics, and receivers
/// whose declared type is in the local map. Anything else is `unknown`.
fn call_throws(
    receiver: Option<&Expr>,
    method: &str,
    args: &[Expr],
    span: SourceSpan,
    handlers: &[Vec<Exc>],
    ctx: &mut Ctx,
) -> ThrownSet {
    let arity = args.len();
    match receiver {
        None | Some(Expr::This { .. }) => {
            let own = ctx.class.name.clone();
            callee_throws_named(&own, method, arity, span, handlers, ctx)
        }
        Some(Expr::Name { path, .. }) if path.len() == 1 => {
            let head = &path[0];
            if let Some(binding) = ctx.locals.get(head).cloned() {
                // A typed receiver: a user class method, or a library kind.
                if let Binding::Declared(ty) = binding {
                    receiver_type_throws(&ty, method, args, span, handlers, ctx)
                } else {
                    // A catch parameter as a receiver (e.getMessage()):
                    // throwable methods throw nothing checked.
                    ThrownSet::default()
                }
            } else if ctx.table.has_class(head) {
                let head = head.clone();
                callee_throws_named(&head, method, arity, span, handlers, ctx)
            } else {
                library_static_throws(head, method, span, handlers, ctx)
            }
        }
        // A dotted static path (`java.nio.file.Files.readString`).
        Some(Expr::Name { path, .. }) if path.len() > 1 => {
            let last = path[path.len() - 1].clone();
            library_static_throws(&last, method, span, handlers, ctx)
        }
        // A literal receiver — `"x".getBytes("UTF-8")`, which is how the one
        // checked exception on a String is usually reached.
        Some(Expr::Literal {
            value: crate::ast::Literal::Str(_),
            ..
        }) => library_kind_throws(Some("String"), method, args, span, handlers, ctx),
        // A directly-constructed receiver: `new Foo().m()`.
        Some(Expr::NewObject { class, .. }) => {
            let class = class.clone();
            if ctx.table.has_class(&class) {
                callee_throws_named(&class, method, arity, span, handlers, ctx)
            } else {
                library_kind_throws(
                    library_kind_of_class(&class),
                    method,
                    args,
                    span,
                    handlers,
                    ctx,
                )
            }
        }
        _ => ThrownSet {
            list: Vec::new(),
            unknown: true,
        },
    }
}

/// Throws of `class.method/arity` (user classes), reported at `span`.
fn callee_throws_named(
    class: &str,
    method: &str,
    arity: usize,
    span: SourceSpan,
    handlers: &[Vec<Exc>],
    ctx: &mut Ctx,
) -> ThrownSet {
    let names: Vec<String> = ctx.table.declared_throws(class, method, arity).to_vec();
    let mut out = ThrownSet::default();
    for name in names {
        match resolve_exc(&name, ctx.table) {
            Some(e) => out.push(e),
            None => out.unknown = true,
        }
    }
    ctx.report_escapes(&out, handlers, span);
    out
}

/// Throws of a call through a receiver with a DECLARED type.
fn receiver_type_throws(
    ty: &TypeRef,
    method: &str,
    args: &[Expr],
    span: SourceSpan,
    handlers: &[Vec<Exc>],
    ctx: &mut Ctx,
) -> ThrownSet {
    let name = match ty {
        TypeRef::Named(n) => n.clone(),
        TypeRef::Generic { base, .. } => base.clone(),
        _ => return ThrownSet::default(),
    };
    if ctx.table.has_class(&name) {
        return callee_throws_named(&name, method, args.len(), span, handlers, ctx);
    }
    library_kind_throws(
        library_kind_of_class(&name),
        method,
        args,
        span,
        handlers,
        ctx,
    )
}

/// Whether an expression names a `Charset` VALUE rather than a charset's name
/// as text — `StandardCharsets.UTF_8`, a `Charset`-typed variable, or what
/// `Charset.forName` answers. Only the text form can name a charset that does
/// not exist, so only the text form declares the checked exception.
fn names_a_charset(expr: &Expr, ctx: &Ctx) -> bool {
    match expr {
        Expr::Name { path, .. } => match path.as_slice() {
            [only] => matches!(
                ctx.locals.get(only),
                Some(Binding::Declared(TypeRef::Named(name))) if name == "Charset"
                    || name.ends_with(".Charset")
            ),
            // ...written plainly or in full: `java.nio.charset.StandardCharsets
            // .UTF_8` names the same constant.
            [.., owner, _] => owner == "StandardCharsets",
            _ => false,
        },
        Expr::Call {
            receiver: Some(owner),
            method,
            ..
        } => {
            method == "forName"
                && matches!(owner.as_ref(), Expr::Name { path, .. }
                    if path.last().is_some_and(|last| last == "Charset"))
        }
        _ => false,
    }
}

/// The library "kind" a declared type name maps to, for the closed-world
/// thrower table. Only kinds with checked throwers are named.
fn library_kind_of_class(name: &str) -> Option<&'static str> {
    let simple = name.rsplit('.').next().unwrap_or(name);
    match simple {
        "BufferedReader" | "FileReader" | "InputStreamReader" | "StringReader" | "Reader" => {
            Some("Reader")
        }
        // A `StringWriter`'s writes do NOT declare one — it narrows them away —
        // but `close()` still does, which is what a try-with-resources catches.
        "StringWriter" => Some("StringWriter"),
        // A `Writer`'s `write`/`close`/`flush` all declare `IOException` —
        // which is what makes `try (FileWriter w = …) … catch (IOException e)`
        // legal, the shape every program that writes a file is written in.
        // `PrintWriter` is the exception: it swallows, and none of its methods
        // declares one.
        "FileWriter" | "Writer" | "BufferedWriter" | "OutputStreamWriter" => Some("Writer"),
        "File" => Some("File"),
        // A String's `getBytes(name)` — the one method of a type nobody thinks
        // of as I/O that declares a CHECKED exception, because the name may be
        // one no charset answers to.
        "String" => Some("String"),
        // A formatter's `parse` declares `ParseException`, and nothing else in
        // `java.text` declares anything checked.
        "DecimalFormat" | "NumberFormat" => Some("NumberFormat"),
        "Class" => Some("Class"),
        "Method" => Some("Method"),
        "Field" => Some("Field"),
        "Constructor" => Some("Constructor"),
        _ => None,
    }
}

/// Checked exceptions of instance methods on modeled library kinds — the
/// CLOSED WORLD enumeration. Everything not listed throws nothing checked,
/// which is true for caturra's whole modeled surface outside I/O and
/// reflection.
fn library_kind_throws(
    kind: Option<&'static str>,
    method: &str,
    args: &[Expr],
    span: SourceSpan,
    handlers: &[Vec<Exc>],
    ctx: &mut Ctx,
) -> ThrownSet {
    let thrown: &[&'static str] = match (kind, method) {
        // `getBytes(String)` declares it; `getBytes()` and `getBytes(Charset)`
        // do not — a charset OBJECT cannot name an unknown charset, since the
        // only way to build one refuses an unknown name.
        (Some("String"), "getBytes") if args.len() == 1 && !names_a_charset(&args[0], ctx) => {
            &["java/io/UnsupportedEncodingException"]
        }
        // Every I/O method that declares the same one exception, in one arm:
        // a reader's reads, a writer's writes, and `File.createNewFile`.
        (Some("StringWriter"), "close") => &["java/io/IOException"],
        (Some("Reader"), "read" | "readLine" | "ready" | "close" | "lines" | "skip")
        | (Some("Writer"), "write" | "append" | "close" | "flush" | "newLine")
        | (Some("File"), "createNewFile" | "getCanonicalPath" | "getCanonicalFile") => {
            &["java/io/IOException"]
        }
        // Reading a number back is the one `java.text` method that declares a
        // checked exception, and a program has to catch it — but only the
        // form that has no other way to report a failure. `parse(text,
        // position)` writes the error into the POSITION and answers null, so
        // it declares nothing.
        // `parseObject(text)` declares it too, inherited from `Format`, so
        // the one-argument form of either name is the pair.
        (Some("NumberFormat"), "parse" | "parseObject") if args.len() < 2 => {
            &["java/text/ParseException"]
        }
        (
            Some("Class"),
            "getMethod" | "getDeclaredMethod" | "getConstructor" | "getDeclaredConstructor",
        ) => &["java/lang/NoSuchMethodException"],
        (Some("Class"), "getField" | "getDeclaredField") => &["java/lang/NoSuchFieldException"],
        (Some("Class"), "newInstance") => &[
            "java/lang/InstantiationException",
            "java/lang/IllegalAccessException",
        ],
        (Some("Method"), "invoke") => &[
            "java/lang/IllegalAccessException",
            "java/lang/reflect/InvocationTargetException",
        ],
        (
            Some("Field"),
            "get" | "set" | "getInt" | "setInt" | "getDouble" | "setDouble" | "getBoolean"
            | "setBoolean" | "getLong" | "setLong",
        ) => &["java/lang/IllegalAccessException"],
        (Some("Constructor"), "newInstance") => &[
            "java/lang/InstantiationException",
            "java/lang/IllegalAccessException",
            "java/lang/reflect/InvocationTargetException",
        ],
        _ => &[],
    };
    let mut out = ThrownSet::default();
    for internal in thrown {
        out.push(Exc::Lib(internal));
    }
    ctx.report_escapes(&out, handlers, span);
    out
}

/// Whether a `new PrintWriter(...)` wraps another writer rather than naming a
/// file — the one form of it that declares no checked exception.
fn wraps_a_writer(args: &[Expr], ctx: &Ctx) -> bool {
    let [only] = args else {
        return false;
    };
    match only {
        // `new PrintWriter(new StringWriter())`, written inline.
        Expr::NewObject { class, .. } => matches!(
            class.rsplit('.').next().unwrap_or(class),
            "StringWriter" | "CharArrayWriter"
        ),
        // ...or through a variable the program declared as one.
        Expr::Name { path, .. } => matches!(path.as_slice(), [only] if matches!(
            ctx.locals.get(only),
            Some(Binding::Declared(TypeRef::Named(name)))
                if matches!(
                    name.rsplit('.').next().unwrap_or(name),
                    "StringWriter" | "Writer"
                )
        )),
        _ => false,
    }
}

/// Checked exceptions of modeled library STATICS.
fn library_static_throws(
    class: &str,
    method: &str,
    span: SourceSpan,
    handlers: &[Vec<Exc>],
    ctx: &mut Ctx,
) -> ThrownSet {
    let thrown: &[&'static str] = match (class, method) {
        // Every modeled Files operation that touches content throws
        // IOException; the pure predicates do not.
        ("Files", "exists" | "notExists" | "isDirectory" | "isRegularFile") => &[],
        ("Files", _) => &["java/io/IOException"],
        ("Class", "forName") => &["java/lang/ClassNotFoundException"],
        // `Thread.sleep` throws `InterruptedException` in Java, and this table
        // is a model of JAVA, not of what caturra runs: `java.lang.Thread` is
        // refused (one thread, so a sleep would be a lie), and the refusal
        // says so — but only if the CATCH clause around it is legal first.
        // Without this entry the try was "exception InterruptedException is
        // never thrown", which blames the one part of the program that is
        // right.
        ("Thread", "sleep" | "join") => &["java/lang/InterruptedException"],
        _ => &[],
    };
    let mut out = ThrownSet::default();
    for internal in thrown {
        out.push(Exc::Lib(internal));
    }
    ctx.report_escapes(&out, handlers, span);
    out
}

/// Checked exceptions a constructor call can throw.
fn ctor_throws(
    class: &str,
    args: &[Expr],
    span: SourceSpan,
    handlers: &[Vec<Exc>],
    ctx: &mut Ctx,
) -> ThrownSet {
    if ctx.table.has_class(class) {
        let class = class.to_owned();
        return callee_throws_named(&class, "<init>", args.len(), span, handlers, ctx);
    }
    let simple = class.rsplit('.').next().unwrap_or(class);
    let mut out = ThrownSet::default();
    match simple {
        // `new FileReader(...)` / `new PrintWriter(file-or-name)`. A
        // `PrintWriter` over a WRITER declares nothing — there is no file to be
        // missing — which is exactly the form that wraps a `StringWriter`.
        "FileReader" => out.push(Exc::Lib("java/io/FileNotFoundException")),
        "PrintWriter" if !wraps_a_writer(args, ctx) => {
            out.push(Exc::Lib("java/io/FileNotFoundException"));
        }
        // `new String(bytes, "UTF-8")` — the charset NAMED as text may name no
        // charset, so this constructor declares the checked exception where
        // the `Charset` form (and every other String constructor) does not.
        "String"
            if matches!(
                args.last(),
                Some(Expr::Literal {
                    value: crate::ast::Literal::Str(_),
                    ..
                })
            ) && args.len() >= 2 =>
        {
            out.push(Exc::Lib("java/io/UnsupportedEncodingException"));
        }
        // `new FileWriter(...)` declares the broader `IOException` — the
        // directory may be missing, not just the file.
        "FileWriter" | "BufferedWriter" | "OutputStreamWriter" => {
            out.push(Exc::Lib("java/io/IOException"));
        }
        // `new Scanner(file)` throws; `new Scanner("text")` does not. The
        // argument's kind decides, and only a File-typed argument is certain.
        "Scanner" => match args.first() {
            Some(Expr::NewObject { class, .. }) if class == "File" || class.ends_with(".File") => {
                out.push(Exc::Lib("java/io/FileNotFoundException"));
            }
            Some(Expr::Name { path, .. }) if path.len() == 1 => {
                match ctx.locals.get(&path[0]) {
                    Some(Binding::Declared(TypeRef::Named(n)))
                        if n == "File" || n == "java.io.File" =>
                    {
                        out.push(Exc::Lib("java/io/FileNotFoundException"));
                    }
                    Some(Binding::Declared(_)) => {}
                    // Unknown argument: might be a File from elsewhere.
                    _ => out.unknown = true,
                }
            }
            Some(Expr::Literal { .. }) => {}
            _ => out.unknown = true,
        },
        _ => {}
    }
    ctx.report_escapes(&out, handlers, span);
    out
}

// ---------------------------------------------------------------------------
// Exception identity
// ---------------------------------------------------------------------------

/// The types a `throw` statement can throw.
fn throw_types(value: &Expr, ctx: &Ctx) -> ThrownSet {
    let mut out = ThrownSet::default();
    match value {
        Expr::NewObject { class, .. } => match resolve_exc(class, ctx.table) {
            Some(e) => out.push(e),
            None => out.unknown = true,
        },
        Expr::Name { path, .. } if path.len() == 1 => match ctx.locals.get(&path[0]) {
            Some(Binding::Precise(Some(list))) => {
                for e in list {
                    out.push(e.clone());
                }
            }
            Some(Binding::Declared(ty)) => match typeref_exc(ty, ctx.table) {
                Some(e) => out.push(e),
                None => out.unknown = true,
            },
            Some(Binding::Precise(None)) | None => out.unknown = true,
        },
        Expr::Cast { ty, .. } => match resolve_typeref(ty, ctx.table) {
            Some(e) => out.push(e),
            None => out.unknown = true,
        },
        _ => out.unknown = true,
    }
    out
}

fn typeref_exc(ty: &TypeRef, table: &MethodTable) -> Option<Exc> {
    match ty {
        TypeRef::Named(name) => resolve_exc(name, table),
        _ => None,
    }
}

fn resolve_typeref(ty: &TypeRef, table: &MethodTable) -> Option<Exc> {
    typeref_exc(ty, table)
}

/// Resolve an exception NAME (simple or dotted) to a tracked type.
pub(crate) fn resolve_exc(name: &str, table: &MethodTable) -> Option<Exc> {
    let simple = name.rsplit('.').next().unwrap_or(name);
    // A user class shadows a library name.
    if table.has_class(simple) && table.is_user_throwable(simple) {
        return Some(Exc::User(simple.to_owned()));
    }
    if let Some(internal) = exc::internal_name_of(simple) {
        return Some(Exc::Lib(internal));
    }
    None
}

/// Checked = a throwable NOT under `RuntimeException` or `Error` (JLS §11.1.1).
pub(crate) fn exc_is_checked(e: &Exc, table: &MethodTable) -> bool {
    let internal = match e {
        Exc::Lib(internal) => internal,
        Exc::User(name) => match table.user_throwable_ancestor(name) {
            Some(ancestor) => ancestor,
            None => return false,
        },
    };
    !exc::is_exception_subclass(internal, "java/lang/RuntimeException")
        && !exc::is_exception_subclass(internal, "java/lang/Error")
}

/// Whether `thrown` is `catch_t` or a subclass of it.
pub(crate) fn exc_covers(catch_t: &Exc, thrown: &Exc, table: &MethodTable) -> bool {
    match (catch_t, thrown) {
        (Exc::Lib(c), Exc::Lib(t)) => exc::is_exception_subclass(t, c),
        // A user exception is caught by a library catch when its library
        // ancestor is under the catch type.
        (Exc::Lib(c), Exc::User(t)) => table
            .user_throwable_ancestor(t)
            .is_some_and(|ancestor| exc::is_exception_subclass(ancestor, c)),
        // A library exception is never a subclass of a user class.
        (Exc::User(_), Exc::Lib(_)) => false,
        (Exc::User(c), Exc::User(t)) => table.user_class_is_subtype(t, c),
    }
}
