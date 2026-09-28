//! Static imports of the JDK, resolved once, before any pass types a call.
//!
//! `import static java.util.Arrays.stream;` makes `stream(xs)` mean
//! `Arrays.stream(xs)`, and `import static java.lang.System.out;` makes `out`
//! mean `System.out`. caturra answered a few of these where codegen happened
//! to look (`Math.*`, some `Collectors`), and the lambda pass — which types
//! every stream, collector and comparator — knew none of them, so a program
//! written in that style was "cannot find symbol" or "a lambda … is only
//! allowed where a functional-interface type is expected". Every later pass
//! already understands the QUALIFIED spelling, so this pass writes it.
//!
//! JLS §6.4.1: a static import is SHADOWED by a member of the same name in
//! scope. A call `m(…)` is rewritten only when no method named `m` is declared
//! by the class, its program superclasses or its enclosing classes; a name is
//! rewritten only when no local, parameter or field of that name exists in the
//! class (a conservative reading — a name declared anywhere in the class keeps
//! the unqualified meaning). A class extending a LIBRARY class may inherit a
//! method caturra cannot see, so an on-demand import is not applied there.
//!
//! Only `java.*`/`javax.*` imports are resolved here; a static import of a
//! program or test library (`org.junit…Assertions.*`) keeps the resolution
//! codegen does for it.

use std::collections::{HashMap, HashSet};

use crate::ast::{
    AssignTarget, ClassDecl, CompilationUnit, Expr, LambdaBody, Stmt,
};
use crate::diagnostics::SourceSpan;

/// What a file's static imports bring into scope.
#[derive(Default)]
struct Imports {
    /// `import static X.member;` — the member's name, to its class.
    single: HashMap<String, String>,
    /// `import static X.*;` — the classes.
    wildcard: Vec<String>,
}

impl Imports {
    fn of(unit: &CompilationUnit) -> Self {
        let mut imports = Self::default();
        for import in unit.imports.iter().filter(|i| i.is_static) {
            if !matches!(import.path.first().map(String::as_str), Some("java" | "javax")) {
                continue;
            }
            if import.wildcard {
                if let Some(class) = import.path.last() {
                    imports.wildcard.push(class.clone());
                }
            } else if import.path.len() >= 2 {
                let class = import.path[import.path.len() - 2].clone();
                let member = import.path[import.path.len() - 1].clone();
                imports.single.insert(member, class);
            }
        }
        imports
    }

    fn is_empty(&self) -> bool {
        self.single.is_empty() && self.wildcard.is_empty()
    }

    /// The class a bare METHOD name resolves to.
    fn method_class(&self, name: &str, on_demand: bool) -> Option<&str> {
        if let Some(class) = self.single.get(name) {
            return Some(class);
        }
        if !on_demand {
            return None;
        }
        let answering: Vec<&String> = self
            .wildcard
            .iter()
            .filter(|class| crate::codegen::library_static_method(class, name))
            .collect();
        match answering.as_slice() {
            [only] => Some(only),
            _ => None,
        }
    }

    /// The class a bare NAME (a static field) resolves to.
    fn field_class(&self, name: &str, on_demand: bool) -> Option<&str> {
        if let Some(class) = self.single.get(name) {
            return Some(class);
        }
        if !on_demand {
            return None;
        }
        let answering: Vec<&String> = self
            .wildcard
            .iter()
            .filter(|class| crate::codegen::library_static_field(class, name))
            .collect();
        match answering.as_slice() {
            [only] => Some(only),
            _ => None,
        }
    }
}

/// Rewrite every file's JDK static-import uses into their qualified form.
pub fn qualify_static_imports(units: &mut [(String, CompilationUnit)]) {
    let classes: HashMap<String, ClassDecl> = units
        .iter()
        .flat_map(|(_, unit)| unit.classes.iter())
        .map(|class| (class.name.clone(), class.clone()))
        .collect();
    for (_, unit) in units.iter_mut() {
        let imports = Imports::of(unit);
        if imports.is_empty() {
            continue;
        }
        for class in &mut unit.classes {
            let scope = Scope::of(class, &classes);
            let mut rewriter = Rewriter {
                imports: &imports,
                scope: &scope,
            };
            for field in &mut class.fields {
                if let Some(init) = &mut field.init {
                    rewriter.expr(init);
                }
            }
            for block in &mut class.init_blocks {
                rewriter.stmts(&mut block.body);
            }
            for method in &mut class.methods {
                rewriter.stmts(&mut method.body);
            }
        }
    }
}

/// What shadows an import inside one class.
struct Scope {
    methods: HashSet<String>,
    names: HashSet<String>,
    /// Whether the class (or one it is nested in) extends a LIBRARY class,
    /// whose members caturra cannot list.
    library_parent: bool,
}

impl Scope {
    fn of(class: &ClassDecl, classes: &HashMap<String, ClassDecl>) -> Self {
        let mut scope = Self {
            methods: HashSet::new(),
            names: HashSet::new(),
            library_parent: false,
        };
        // The class, the classes enclosing it, and each one's superclass
        // chain among the program's classes.
        let mut roots = vec![class.name.clone()];
        let mut enclosing = class.enclosing.clone();
        while let Some(name) = enclosing {
            enclosing = classes.get(&name).and_then(|c| c.enclosing.clone());
            roots.push(name);
            if roots.len() > classes.len() + 1 {
                break;
            }
        }
        let mut seen = HashSet::new();
        for root in roots {
            let mut current = Some(root);
            while let Some(name) = current {
                if !seen.insert(name.clone()) {
                    break;
                }
                let Some(decl) = classes.get(&name) else {
                    scope.library_parent |= name != "Object";
                    break;
                };
                // An anonymous class records the one type it names as its
                // superclass; for `new Comparator<…>() {…}` that is an
                // INTERFACE, which hides no static import.
                let parent = decl
                    .superclass
                    .clone()
                    .filter(|parent| !decl.is_anonymous || classes.contains_key(parent));
                scope.methods.extend(decl.methods.iter().map(|m| m.name.clone()));
                scope.names.extend(decl.fields.iter().map(|f| f.name.clone()));
                for method in &decl.methods {
                    scope.names.extend(method.params.iter().map(|p| p.name.clone()));
                    collect_names(&method.body, &mut scope.names);
                }
                for block in &decl.init_blocks {
                    collect_names(&block.body, &mut scope.names);
                }
                current = parent;
            }
        }
        // Class names are never static members.
        scope.names.extend(classes.keys().cloned());
        scope
    }
}

/// Every local, loop, catch and lambda parameter name declared in `stmts`.
fn collect_names(stmts: &[Stmt], out: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::LocalDecl { declarators, .. } => {
                out.extend(declarators.iter().map(|d| d.name.clone()));
                for d in declarators {
                    if let Some(init) = &d.init {
                        collect_expr_names(init, out);
                    }
                }
            }
            Stmt::Block(body) => collect_names(body, out),
            Stmt::ForEach { name, body, iterable, .. } => {
                out.insert(name.clone());
                collect_expr_names(iterable, out);
                collect_names(std::slice::from_ref(body), out);
            }
            Stmt::For { init, update, body, cond, .. } => {
                if let Some(init) = init {
                    collect_names(std::slice::from_ref(init), out);
                }
                if let Some(cond) = cond {
                    collect_expr_names(cond, out);
                }
                collect_names(update, out);
                collect_names(std::slice::from_ref(body), out);
            }
            Stmt::If { cond, then, els, .. } => {
                collect_expr_names(cond, out);
                collect_names(std::slice::from_ref(then), out);
                if let Some(els) = els {
                    collect_names(std::slice::from_ref(els), out);
                }
            }
            Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
                collect_expr_names(cond, out);
                collect_names(std::slice::from_ref(body), out);
            }
            Stmt::Switch { selector, arms, .. } => {
                collect_expr_names(selector, out);
                for arm in arms {
                    collect_names(&arm.body, out);
                }
            }
            Stmt::Try { body, catches, finally_body, .. } => {
                collect_names(body, out);
                for catch in catches {
                    out.insert(catch.name.clone());
                    collect_names(&catch.body, out);
                }
                if let Some(fin) = finally_body {
                    collect_names(fin, out);
                }
            }
            Stmt::Labeled { body, .. } => collect_names(std::slice::from_ref(body), out),
            Stmt::Expr(e) | Stmt::Throw { value: e, .. } | Stmt::Assign { value: e, .. } => {
                collect_expr_names(e, out);
            }
            Stmt::Return { value: Some(e), .. } => collect_expr_names(e, out),
            _ => {}
        }
    }
}

/// Lambda parameters (and the locals of lambda block bodies) inside `expr`.
fn collect_expr_names(expr: &Expr, out: &mut HashSet<String>) {
    let mut stack = vec![expr];
    while let Some(expr) = stack.pop() {
        if let Expr::Lambda { params, body, .. } = expr {
            out.extend(params.iter().map(|p| p.name.clone()));
            match body {
                LambdaBody::Expr(e) => stack.push(e),
                LambdaBody::Block(stmts) => collect_names(stmts, out),
            }
            continue;
        }
        for child in children(expr) {
            stack.push(child);
        }
    }
}

/// The direct sub-expressions of `expr` (lambda bodies excluded).
fn children(expr: &Expr) -> Vec<&Expr> {
    match expr {
        Expr::Call { receiver, args, .. } => receiver.as_deref().into_iter().chain(args).collect(),
        Expr::SuperMethodCall { args, .. } => args.iter().collect(),
        Expr::NewObject { args, outer, .. } => outer.as_deref().into_iter().chain(args).collect(),
        Expr::Binary { lhs, rhs, .. } => vec![lhs, rhs],
        Expr::Unary { operand, .. } | Expr::Cast { operand, .. } => vec![operand],
        Expr::Field { object, .. } => vec![object],
        Expr::InstanceOf { value, .. } => vec![value],
        Expr::Index { array, index, .. } => vec![array, index],
        Expr::Ternary { cond, then, els, .. } => vec![cond, then, els],
        Expr::IncDec { target, .. } => vec![target],
        Expr::NewArray { dims, init, .. } => dims
            .iter()
            .flatten()
            .chain(init.iter().flatten())
            .collect(),
        Expr::ArrayLiteral { elements, .. } => elements.iter().collect(),
        Expr::MethodRef { qualifier, .. } => vec![qualifier],
        Expr::Assign { target, value, .. } => {
            let mut out: Vec<&Expr> = match target {
                AssignTarget::Var(_) => Vec::new(),
                AssignTarget::Index { array, index } => vec![array, index],
                AssignTarget::Field { object, .. } => vec![object],
            };
            out.push(value);
            out
        }
        _ => Vec::new(),
    }
}

struct Rewriter<'a> {
    imports: &'a Imports,
    scope: &'a Scope,
}

impl Rewriter<'_> {
    fn stmts(&mut self, stmts: &mut [Stmt]) {
        for stmt in stmts {
            self.stmt(stmt);
        }
    }

    fn stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Block(body) => self.stmts(body),
            Stmt::LocalDecl { declarators, .. } => {
                for d in declarators {
                    if let Some(init) = &mut d.init {
                        self.expr(init);
                    }
                }
            }
            Stmt::Expr(e) | Stmt::Throw { value: e, .. } => self.expr(e),
            Stmt::Assign { target, value, .. } => {
                self.target(target);
                self.expr(value);
            }
            Stmt::Return { value: Some(e), .. } => self.expr(e),
            Stmt::If { cond, then, els, .. } => {
                self.expr(cond);
                self.stmt(then);
                if let Some(els) = els {
                    self.stmt(els);
                }
            }
            Stmt::While { cond, body, .. } | Stmt::DoWhile { cond, body, .. } => {
                self.expr(cond);
                self.stmt(body);
            }
            Stmt::For { init, cond, update, body, .. } => {
                if let Some(init) = init {
                    self.stmt(init);
                }
                if let Some(cond) = cond {
                    self.expr(cond);
                }
                self.stmts(update);
                self.stmt(body);
            }
            Stmt::ForEach { iterable, body, .. } => {
                self.expr(iterable);
                self.stmt(body);
            }
            Stmt::Switch { selector, arms, .. } => {
                self.expr(selector);
                for arm in arms {
                    self.stmts(&mut arm.body);
                }
            }
            Stmt::Try { body, catches, finally_body, .. } => {
                self.stmts(body);
                for catch in catches {
                    self.stmts(&mut catch.body);
                }
                if let Some(fin) = finally_body {
                    self.stmts(fin);
                }
            }
            Stmt::Labeled { body, .. } => self.stmt(body),
            Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
                for a in args {
                    self.expr(a);
                }
            }
            _ => {}
        }
    }

    fn target(&mut self, target: &mut AssignTarget) {
        match target {
            AssignTarget::Var(_) => {}
            AssignTarget::Index { array, index } => {
                self.expr(array);
                self.expr(index);
            }
            AssignTarget::Field { object, .. } => self.expr(object),
        }
    }

    fn qualifier(class: &str, span: SourceSpan) -> Box<Expr> {
        Box::new(Expr::Name {
            path: vec![class.to_owned()],
            span,
        })
    }

    fn expr(&mut self, expr: &mut Expr) {
        let on_demand = !self.scope.library_parent;
        match expr {
            Expr::Call {
                receiver: receiver @ None,
                method,
                span,
                ..
            } if !self.scope.methods.contains(method.as_str()) => {
                if let Some(class) = self.imports.method_class(method, on_demand) {
                    *receiver = Some(Self::qualifier(class, *span));
                }
            }
            Expr::Name { path, .. }
                if !path.is_empty() && !self.scope.names.contains(&path[0]) =>
            {
                if let Some(class) = self.imports.field_class(&path[0], on_demand) {
                    path.insert(0, class.to_owned());
                }
            }
            _ => {}
        }
        match expr {
            Expr::Lambda { body, .. } => match body {
                LambdaBody::Expr(e) => self.expr(e),
                LambdaBody::Block(stmts) => self.stmts(stmts),
            },
            Expr::Call { receiver, args, .. } => {
                if let Some(r) = receiver {
                    self.expr(r);
                }
                for a in args {
                    self.expr(a);
                }
            }
            Expr::SuperMethodCall { args, .. } => {
                for a in args {
                    self.expr(a);
                }
            }
            Expr::NewObject { args, outer, .. } => {
                if let Some(o) = outer {
                    self.expr(o);
                }
                for a in args {
                    self.expr(a);
                }
            }
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Unary { operand, .. } | Expr::Cast { operand, .. } => self.expr(operand),
            Expr::Field { object, .. } => self.expr(object),
            Expr::InstanceOf { value, .. } => self.expr(value),
            Expr::Index { array, index, .. } => {
                self.expr(array);
                self.expr(index);
            }
            Expr::Ternary { cond, then, els, .. } => {
                self.expr(cond);
                self.expr(then);
                self.expr(els);
            }
            Expr::IncDec { target, .. } => self.expr(target),
            Expr::NewArray { dims, init, .. } => {
                for d in dims.iter_mut().flatten() {
                    self.expr(d);
                }
                if let Some(elems) = init {
                    for e in elems {
                        self.expr(e);
                    }
                }
            }
            Expr::ArrayLiteral { elements, .. } => {
                for e in elements {
                    self.expr(e);
                }
            }
            Expr::MethodRef { qualifier, .. } => self.expr(qualifier),
            Expr::Assign { target, value, .. } => {
                self.target(target);
                self.expr(value);
            }
            _ => {}
        }
    }
}
