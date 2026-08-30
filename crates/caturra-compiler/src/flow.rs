//! Statement reachability (JLS §14.21) and blank-final definite assignment
//! (JLS §8.3.1.2, §16.9).
//!
//! Both are *rejection* analyses: they exist to refuse programs a real javac
//! refuses. caturra had neither, so code after a `return` compiled, and a
//! blank `final` field could go unassigned (reading 0), be assigned twice, or
//! be read before it was assigned — `final` was not enforced for fields at
//! all. Every one of those compiles in the playground and fails on a JDK,
//! which is the direction that hurts.
//!
//! **Conservative by construction.** A missed error is a nuisance; a spurious
//! one rejects a valid program, so every rule here errs toward saying nothing.
//! `constant_bool` asks the SHARED constant folder — the one codegen uses — so
//! it recognises the full constant expressions of JLS §15.28: `while (DEBUG)`
//! over a `static final boolean`, and `while (FOUR < TWO)` over two constant
//! ints alike. A second, narrower folder lived here once, and the loops it
//! could not see through were ACCEPTED where javac reports the body
//! unreachable.

use crate::ast::{ClassDecl, Expr, FieldDecl, MethodDecl, Stmt};
use crate::diagnostics::{Diagnostic, SourceSpan};

/// Report unreachable statements and blank-final violations in `decl`.
pub(crate) fn check(
    decl: &ClassDecl,
    path: &str,
    table: &crate::codegen::MethodTable,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let constants = table.constant_literals(&decl.name);
    for method in &decl.methods {
        if method.is_abstract {
            continue;
        }
        let mut reporter = Reporter {
            path,
            diagnostics,
            constants: constants.clone(),
        };
        // The body of a method is reachable (JLS §14.21).
        reachability(&method.body, &mut reporter);
    }
    // An initializer block is a block like any other, and used to escape every
    // check here — its unreachable code, its `return`, and (for an instance
    // one) its inability to finish all went unreported.
    for block in &decl.init_blocks {
        let mut reporter = Reporter {
            path,
            diagnostics,
            constants: constants.clone(),
        };
        reachability(&block.body, &mut reporter);
        // JLS §14.17: `return` belongs to a method or constructor. An
        // initializer is neither, whether it is static or not.
        if let Some(span) = first_return(&block.body) {
            reporter.error(span, "return outside method");
        }
        // JLS §8.6: an INSTANCE initializer must be able to complete normally.
        // A static one need not (§8.7 forbids only a checked exception), so
        // `static { throw new RuntimeException(); }` stays legal.
        if !block.is_static && !block_completes_normally(&block.body) {
            reporter.error(block.span, "initializer must be able to complete normally");
        }
    }
    blank_finals(decl, path, diagnostics);
    recursive_constructors(decl, path, diagnostics);
}

/// JLS §8.8.7.1: a constructor may not invoke itself, directly or around a
/// chain of `this(...)` delegations. Left unchecked, `C() { this(1); }
/// C(int a) { this(); }` compiled and blew the stack at run time.
///
/// The delegation target is matched by ARITY, which is all this pass can see
/// (overload resolution lives in codegen), so the graph is ambiguous wherever
/// two constructors share one. A constructor is therefore reported only when NO
/// resolution can terminate: `terminates` is the least fixpoint over "declares
/// no `this(...)`" and "some candidate terminates". A delegation that matches
/// nothing counts as terminating too — the arity may simply be one this pass
/// does not model (a varargs constructor, say).
fn recursive_constructors(decl: &ClassDecl, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let ctors: Vec<&MethodDecl> = decl.methods.iter().filter(|m| m.is_constructor).collect();
    let targets: Vec<Option<Vec<usize>>> = ctors
        .iter()
        .map(|ctor| {
            ctor.body
                .iter()
                .find_map(|stmt| match stmt {
                    Stmt::ThisCall { args, .. } => Some(args.len()),
                    _ => None,
                })
                .map(|arity| {
                    (0..ctors.len())
                        .filter(|i| ctors[*i].params.len() == arity)
                        .collect()
                })
        })
        .collect();
    let mut terminates: Vec<bool> = targets
        .iter()
        .map(|target| match target {
            None => true,
            Some(candidates) => candidates.is_empty(),
        })
        .collect();
    loop {
        let mut changed = false;
        for (i, target) in targets.iter().enumerate() {
            if terminates[i] {
                continue;
            }
            if let Some(candidates) = target
                && candidates.iter().any(|c| terminates[*c])
            {
                terminates[i] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for (ctor, ok) in ctors.iter().zip(terminates) {
        if !ok {
            diagnostics.push(Diagnostic::error(
                path,
                "recursive constructor invocation",
                ctor.span,
            ));
        }
    }
}

/// The first `return` in this block, at any depth of nested STATEMENT. Lambda
/// bodies and local classes are hoisted into classes of their own long before
/// this runs, so a `return` reached here really does belong to the block.
fn first_return(statements: &[Stmt]) -> Option<SourceSpan> {
    statements.iter().find_map(return_span)
}

fn return_span(statement: &Stmt) -> Option<SourceSpan> {
    match statement {
        Stmt::Return { span, .. } => Some(*span),
        Stmt::Block(body) => first_return(body),
        Stmt::If { then, els, .. } => return_span(then).or_else(|| {
            els.as_deref()
                .and_then(|els| -> Option<SourceSpan> { return_span(els) })
        }),
        Stmt::While { body, .. }
        | Stmt::DoWhile { body, .. }
        | Stmt::For { body, .. }
        | Stmt::ForEach { body, .. }
        | Stmt::Labeled { body, .. } => return_span(body),
        Stmt::Switch { arms, .. } => arms.iter().find_map(|arm| first_return(&arm.body)),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => first_return(body)
            .or_else(|| catches.iter().find_map(|c| first_return(&c.body)))
            .or_else(|| finally_body.as_deref().and_then(first_return)),
        _ => None,
    }
}

struct Reporter<'a> {
    path: &'a str,
    diagnostics: &'a mut Vec<Diagnostic>,
    /// The class's own `static final boolean` CONSTANT VARIABLES with a
    /// literal initializer — `while (FLAG)` is as constant as `while (true)`
    /// when `FLAG` is one (JLS §15.28), so what follows is unreachable.
    constants: std::collections::HashMap<String, crate::ast::Literal>,
}

impl Reporter<'_> {
    fn error(&mut self, span: SourceSpan, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::error(self.path, message, span));
    }
}

// ---------------------------------------------------------------------------
// Reachability (JLS §14.21)
// ---------------------------------------------------------------------------

/// Walk a block, reporting the FIRST statement that cannot be reached.
///
/// Only the first: javac reports one unreachable statement per block, and
/// every statement after it is unreachable for the same reason, so repeating
/// the message would bury the cause.
fn reachability(statements: &[Stmt], reporter: &mut Reporter) {
    let mut reachable = true;
    for statement in statements {
        if !reachable && let Some(span) = stmt_span(statement) {
            reporter.error(span, "unreachable statement");
        }
        descend(statement, reporter);
        // Reachability RESTARTS after a reported statement: everything below
        // it is unreachable for the same already-reported reason, and javac
        // reports only the first.
        //
        // A loop whose condition is a constant VARIABLE (`while (FLAG)`) is
        // recognized here, where the enclosing class's constants are in hand;
        // `completes_normally` itself folds only the literal forms, since
        // codegen calls it without a class.
        reachable = match statement {
            Stmt::While { cond, body, .. }
                if constant_bool_in(cond, &reporter.constants) == Some(true) =>
            {
                has_escaping_break(body, None)
            }
            other => completes_normally(other),
        };
    }
}

/// Recurse into a statement's own sub-blocks.
fn descend(statement: &Stmt, reporter: &mut Reporter) {
    match statement {
        Stmt::Block(body) => reachability(body, reporter),
        // JLS §14.21 deliberately exempts `if` from the constant-condition
        // rule so that `if (DEBUG) { ... }` compiles either way — the
        // "conditional compilation" carve-out. BOTH branches are treated as
        // reachable no matter what the condition is, which is why
        // `if (false) S;` is legal while `while (false) S;` is not.
        Stmt::If { then, els, .. } => {
            descend_stmt(then, reporter);
            if let Some(els) = els {
                descend_stmt(els, reporter);
            }
        }
        Stmt::While { cond, body, .. } => {
            if constant_bool_in(cond, &reporter.constants) == Some(false)
                && let Some(span) = stmt_span(body)
            {
                reporter.error(span, "unreachable statement");
            }
            descend_stmt(body, reporter);
        }
        Stmt::For { cond, body, .. } => {
            if cond
                .as_ref()
                .is_some_and(|c| constant_bool_in(c, &reporter.constants) == Some(false))
                && let Some(span) = stmt_span(body)
            {
                reporter.error(span, "unreachable statement");
            }
            descend_stmt(body, reporter);
        }
        // A `do` body always runs at least once, so it is reachable
        // whatever the condition says.
        Stmt::DoWhile { body, .. } | Stmt::ForEach { body, .. } | Stmt::Labeled { body, .. } => {
            descend_stmt(body, reporter);
        }
        Stmt::Switch { arms, .. } => {
            // Each group is entered by its own label, so reachability
            // restarts at every arm rather than flowing in from the last.
            for arm in arms {
                reachability(&arm.body, reporter);
            }
        }
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            reachability(body, reporter);
            for clause in catches {
                reachability(&clause.body, reporter);
            }
            if let Some(finally_body) = finally_body {
                reachability(finally_body, reporter);
            }
        }
        _ => {}
    }
}

fn descend_stmt(statement: &Stmt, reporter: &mut Reporter) {
    descend(statement, reporter);
}

/// Whether `statement` can complete normally (JLS §14.21) — i.e. whether the
/// statement after it is reachable.
pub(crate) fn completes_normally(statement: &Stmt) -> bool {
    match statement {
        Stmt::Return { .. } | Stmt::Throw { .. } | Stmt::Break { .. } | Stmt::Continue { .. } => {
            false
        }
        Stmt::Block(body) => block_completes_normally(body),
        Stmt::If {
            then,
            els: Some(els),
            ..
        } => completes_normally(then) || completes_normally(els),
        // An `if` with no `else` always completes normally: the condition may
        // be false (true even for `if (true)`, per the carve-out), and a
        // for-each iterates a collection that may be empty. Both are covered
        // by the catch-all below; spelling them out would only repeat it.
        // A constant-true loop completes only by breaking out of it.
        Stmt::While { cond, body, .. } => {
            if constant_bool(cond) == Some(true) {
                return has_escaping_break(body, None);
            }
            true
        }
        Stmt::DoWhile { body, cond, .. } => {
            if constant_bool(cond) == Some(true) {
                return has_escaping_break(body, None);
            }
            // The body runs first, so an abrupt body still ends the loop.
            // A `continue` counts too: JLS §14.21 makes a do's CONDITION
            // reachable when the loop contains a reachable continue, so a body
            // that only ever returns or continues still lets the loop finish.
            // Without this, `do { try { return a; } finally { continue; } }
            // while (c);` — javac accepts it — was refused as "unreachable
            // statement" at whatever followed the loop.
            block_or_stmt_completes(body)
                || has_escaping_break(body, None)
                || has_escaping_continue(body, None)
        }
        Stmt::For { cond, body, .. } => match cond {
            Some(cond) if constant_bool(cond) != Some(true) => true,
            // `for (;;)` and `for (; true;)`.
            _ => has_escaping_break(body, None),
        },
        Stmt::Labeled { label, body, .. } => {
            // A labeled statement also completes normally if some
            // `break label;` inside it targets this label — or, when it labels
            // a `do`, a `continue label;` that makes the loop's condition
            // reachable.
            completes_normally(body)
                || has_escaping_break(body, Some(label))
                || matches!(body.as_ref(), Stmt::DoWhile { cond, .. } if constant_bool(cond) != Some(true))
                    && has_escaping_continue(body, Some(label))
        }
        Stmt::Switch { arms, .. } => switch_completes_normally(arms),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            // A `finally` that cannot complete normally swallows everything.
            if let Some(finally_body) = finally_body
                && !block_completes_normally(finally_body)
            {
                return false;
            }
            block_completes_normally(body)
                || catches
                    .iter()
                    .any(|clause| block_completes_normally(&clause.body))
        }
        _ => true,
    }
}

pub(crate) fn block_completes_normally(statements: &[Stmt]) -> bool {
    statements.iter().all(completes_normally)
}

fn block_or_stmt_completes(statement: &Stmt) -> bool {
    completes_normally(statement)
}

fn switch_completes_normally(arms: &[crate::ast::SwitchArm]) -> bool {
    let has_default = arms
        .iter()
        .any(|arm| arm.labels.iter().any(Option::is_none));
    // Without a `default`, the selector may match nothing.
    if !has_default {
        return true;
    }
    // Any `break` leaves the switch, and so does falling out of the last arm.
    arms.iter()
        .any(|arm| arm.body.iter().any(|s| has_escaping_break(s, None)))
        || arms
            .last()
            .is_some_and(|arm| block_completes_normally(&arm.body))
}

/// Whether `statement` contains a `break` that would leave the construct being
/// asked about — used to decide whether a `while (true)` or a labeled
/// statement can complete normally.
///
/// `label` selects what counts: `None` asks about an unlabeled `break`, which
/// binds to the nearest enclosing loop or switch, so the search does NOT
/// descend into a nested loop or switch (that one would capture it).
/// `Some(name)` asks about `break name;`, which escapes any nesting, so the
/// search descends everywhere.
/// Whether `statement` contains a `continue` that reaches the loop being
/// asked about — the mirror of [`has_escaping_break`], with one difference
/// that matters: a nested SWITCH captures an unlabeled `break` but not an
/// unlabeled `continue`, which passes straight through it to the loop.
fn has_escaping_continue(statement: &Stmt, label: Option<&str>) -> bool {
    match statement {
        Stmt::Continue {
            label: continue_label,
            ..
        } => match label {
            Some(wanted) => continue_label.as_deref() == Some(wanted),
            None => continue_label.is_none(),
        },
        Stmt::Block(body) => body.iter().any(|s| has_escaping_continue(s, label)),
        Stmt::If { then, els, .. } => {
            has_escaping_continue(then, label)
                || els
                    .as_deref()
                    .is_some_and(|e| has_escaping_continue(e, label))
        }
        Stmt::Labeled { body, .. } => has_escaping_continue(body, label),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            body.iter().any(|s| has_escaping_continue(s, label))
                || catches
                    .iter()
                    .any(|c| c.body.iter().any(|s| has_escaping_continue(s, label)))
                || finally_body
                    .as_ref()
                    .is_some_and(|f| f.iter().any(|s| has_escaping_continue(s, label)))
        }
        // A nested loop captures an unlabeled continue; a labeled one passes
        // straight through it.
        Stmt::While { body, .. }
        | Stmt::DoWhile { body, .. }
        | Stmt::For { body, .. }
        | Stmt::ForEach { body, .. } => label.is_some() && has_escaping_continue(body, label),
        Stmt::Switch { arms, .. } => arms
            .iter()
            .any(|arm| arm.body.iter().any(|s| has_escaping_continue(s, label))),
        _ => false,
    }
}

fn has_escaping_break(statement: &Stmt, label: Option<&str>) -> bool {
    match statement {
        Stmt::Break {
            label: break_label, ..
        } => match label {
            Some(wanted) => break_label.as_deref() == Some(wanted),
            None => break_label.is_none(),
        },
        Stmt::Block(body) => body.iter().any(|s| has_escaping_break(s, label)),
        Stmt::If { then, els, .. } => {
            has_escaping_break(then, label)
                || els.as_deref().is_some_and(|e| has_escaping_break(e, label))
        }
        Stmt::Labeled { body, .. } => has_escaping_break(body, label),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            body.iter().any(|s| has_escaping_break(s, label))
                || catches
                    .iter()
                    .any(|c| c.body.iter().any(|s| has_escaping_break(s, label)))
                || finally_body
                    .as_ref()
                    .is_some_and(|f| f.iter().any(|s| has_escaping_break(s, label)))
        }
        // A nested loop or switch captures an unlabeled break, but a labeled
        // one passes straight through it.
        Stmt::While { body, .. }
        | Stmt::DoWhile { body, .. }
        | Stmt::For { body, .. }
        | Stmt::ForEach { body, .. } => label.is_some() && has_escaping_break(body, label),
        Stmt::Switch { arms, .. } => {
            label.is_some()
                && arms
                    .iter()
                    .any(|arm| arm.body.iter().any(|s| has_escaping_break(s, label)))
        }
        _ => false,
    }
}

/// Whether this condition is a compile-time CONSTANT EXPRESSION with a known
/// value (JLS §15.28). Reachability turns on it: `while (1 == 1) { }` makes
/// what follows unreachable exactly as `while (true)` does, and caturra saw
/// only the bare literal — so a folded condition ran forever where javac
/// refuses the program.
///
/// The folding here is deliberately narrow: literals, parentheses, `!`, the
/// comparisons and the boolean operators over INT/boolean literals. A constant
/// VARIABLE (`static final boolean FLAG = true`) is resolved by the caller,
/// which has the class in hand.
pub(crate) fn constant_bool(expr: &Expr) -> Option<bool> {
    constant_bool_in(expr, &std::collections::HashMap::new())
}

fn constant_bool_in(
    expr: &Expr,
    constants: &std::collections::HashMap<String, crate::ast::Literal>,
) -> Option<bool> {
    // The SAME folder codegen uses, rather than a second one here. Reachability
    // asks a JLS §15.28 question ("is this condition a constant expression, and
    // which one?"), and the miniature folder that used to answer it knew
    // literals and boolean constant variables but not, say, `FOUR < TWO` over
    // two constant ints — which javac rejects as an unreachable loop body and
    // caturra accepted.
    let mut resolve = |path: &[String]| {
        constants
            .get(&path.join("."))
            .and_then(crate::constfold::literal_const)
    };
    match crate::constfold::fold(expr, &mut resolve) {
        Some(crate::constfold::ConstValue::Bool(value)) => Some(value),
        _ => None,
    }
}

/// A statement's span, for the error caret.
fn stmt_span(statement: &Stmt) -> Option<SourceSpan> {
    match statement {
        Stmt::Expr(expr) => Some(expr.span()),
        Stmt::LocalDecl { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::ForEach { span, .. }
        | Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::DoWhile { span, .. }
        | Stmt::For { span, .. }
        | Stmt::Break { span, .. }
        | Stmt::Continue { span, .. }
        | Stmt::Labeled { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::SuperCall { span, .. }
        | Stmt::ThisCall { span, .. }
        | Stmt::Try { span, .. }
        | Stmt::Throw { span, .. }
        | Stmt::Switch { span, .. } => Some(*span),
        // A bare block borrows its first statement's position.
        Stmt::Block(body) => body.first().and_then(stmt_span),
    }
}

// ---------------------------------------------------------------------------
// Blank finals (JLS §8.3.1.2, §16.9)
// ---------------------------------------------------------------------------

/// A `final` field with no initializer must be definitely assigned by the end
/// of every constructor, and definitely `UNassigned` before each assignment.
///
/// Without this a blank final silently read its type's default, and `final`
/// meant nothing for a field: it could be written twice in one constructor.
#[allow(clippy::too_many_lines)] // the instance and static rules, in one pass
fn blank_finals(decl: &ClassDecl, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    // A field a desugaring assigns in a constructor's PRE-INIT prologue (a
    // captured local, an enum constant's name and ordinal) is definitely
    // assigned before any initializer or constructor body runs, so it is not a
    // blank final in the user's sense at all. Counting it as one made an
    // anonymous class that reads a captured local from its instance
    // initializer — legal Java — report the capture as uninitialized.
    let assigned_by_prologue: Vec<&str> = decl
        .methods
        .iter()
        .filter(|method| method.is_constructor)
        .flat_map(|method| method.body.iter().take(method.pre_init))
        .filter_map(|stmt| match stmt {
            Stmt::Assign {
                target: crate::ast::AssignTarget::Field { object, name },
                ..
            } if matches!(**object, Expr::This { .. }) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    // A blank `static final` is the same rule over the STATIC initializers,
    // which are one program in source order: assigning in two of them is the
    // second assignment, and never assigning at all leaves it uninitialized.
    let static_blanks: Vec<&FieldDecl> = decl
        .fields
        .iter()
        .filter(|field| field.is_final && field.is_static && field.init.is_none())
        .collect();
    for field in &static_blanks {
        let mut assigned_in: Option<SourceSpan> = None;
        for block in decl.init_blocks.iter().filter(|b| b.is_static) {
            if let Some(span) = assigns_twice(&block.body, &field.name) {
                diagnostics.push(Diagnostic::error(
                    path,
                    format!("variable {} might already have been assigned", field.name),
                    span,
                ));
            }
            if assigns_definitely_block(&block.body, &field.name) {
                if let Some(_earlier) = assigned_in {
                    diagnostics.push(Diagnostic::error(
                        path,
                        format!("variable {} might already have been assigned", field.name),
                        block.span,
                    ));
                }
                assigned_in = Some(block.span);
            }
        }
        if assigned_in.is_none() {
            diagnostics.push(Diagnostic::error(
                path,
                format!("variable {} might not have been initialized", field.name),
                field.span,
            ));
        }
    }

    let blanks: Vec<&FieldDecl> = decl
        .fields
        .iter()
        .filter(|field| field.is_final && !field.is_static && field.init.is_none())
        .filter(|field| !assigned_by_prologue.contains(&field.name.as_str()))
        .collect();
    if blanks.is_empty() {
        return;
    }
    // An instance initializer block assigns before every constructor body, so
    // a field it assigns is already definitely assigned everywhere.
    let assigned_by_initializer: Vec<&str> = blanks
        .iter()
        .filter(|field| {
            decl.init_blocks
                .iter()
                .filter(|block| !block.is_static)
                .any(|block| assigns_definitely_block(&block.body, &field.name))
        })
        .map(|field| field.name.as_str())
        .collect();

    let constructors: Vec<&MethodDecl> = decl.methods.iter().filter(|m| m.is_constructor).collect();

    for field in &blanks {
        blank_final_in_initializers(decl, field, &constructors, path, diagnostics);
        if assigned_by_initializer.contains(&field.name.as_str()) {
            continue;
        }
        // With no constructor at all, the default one assigns nothing.
        if constructors.is_empty() {
            diagnostics.push(Diagnostic::error(
                path,
                format!("variable {} might not have been initialized", field.name),
                field.span,
            ));
            continue;
        }
        for constructor in &constructors {
            // A constructor delegating with `this(...)` relies on the one it
            // calls, which is checked on its own. The delegation is FIRST in
            // source, but an enum's constructors are rewritten with a
            // synthetic prologue (the constant's name and ordinal) ahead of
            // it — so look for it anywhere, which `this(...)`'s own
            // position rule makes safe.
            if constructor
                .body
                .iter()
                .any(|s| matches!(s, Stmt::ThisCall { .. }))
            {
                continue;
            }
            if !assigns_definitely_block(&constructor.body, &field.name) {
                diagnostics.push(Diagnostic::error(
                    path,
                    format!("variable {} might not have been initialized", field.name),
                    constructor.span,
                ));
            }
            // A local or parameter of the same name shadows the field, so the
            // "read" would be of that instead. Over-approximated across the
            // whole constructor, which only ever MISSES an error.
            let shadowed = constructor.params.iter().any(|p| p.name == field.name)
                || declares_local_named(&constructor.body, &field.name);
            if !shadowed && let Some(span) = read_before_assignment(&constructor.body, &field.name)
            {
                diagnostics.push(Diagnostic::error(
                    path,
                    format!("variable {} might not have been initialized", field.name),
                    span,
                ));
            }
            if let Some(span) = assigns_twice(&constructor.body, &field.name) {
                diagnostics.push(Diagnostic::error(
                    path,
                    format!("variable {} might already have been assigned", field.name),
                    span,
                ));
            }
        }
    }
}

/// A blank final in the INSTANCE INITIALIZERS: a read before the block assigns
/// it, a second assignment within the block, and — once a block has assigned it
/// — any assignment in a constructor, which is then the second one (JLS §16.9).
/// Initializers used to be exempt from all three.
fn blank_final_in_initializers(
    decl: &ClassDecl,
    field: &FieldDecl,
    constructors: &[&MethodDecl],
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut assigned = false;
    for block in decl.init_blocks.iter().filter(|b| !b.is_static) {
        if let Some(span) = read_before_assignment(&block.body, &field.name) {
            diagnostics.push(Diagnostic::error(
                path,
                format!("variable {} might not have been initialized", field.name),
                span,
            ));
        }
        if let Some(span) = assigns_twice(&block.body, &field.name) {
            diagnostics.push(Diagnostic::error(
                path,
                format!("variable {} might already have been assigned", field.name),
                span,
            ));
        }
        assigned |= assigns_definitely_block(&block.body, &field.name);
    }
    if !assigned {
        return;
    }
    // A delegating constructor is not checked here: the one it calls carries
    // the rule, exactly as for the non-initializer case.
    for constructor in constructors {
        if constructor
            .body
            .iter()
            .any(|s| matches!(s, Stmt::ThisCall { .. }))
        {
            continue;
        }
        if let Some(span) = first_assignment(&constructor.body, &field.name) {
            diagnostics.push(Diagnostic::error(
                path,
                format!("variable {} might already have been assigned", field.name),
                span,
            ));
        }
    }
}

/// Whether `statement` definitely assigns the field named `name`.
fn assigns_definitely(statement: &Stmt, name: &str) -> bool {
    match statement {
        Stmt::Assign { target, .. } => is_field_target(target, name),
        Stmt::Block(body) => assigns_definitely_block(body, name),
        // Both branches must assign; a branch that cannot complete normally
        // imposes no requirement (JLS §16.2.7).
        Stmt::If {
            then,
            els: Some(els),
            ..
        } => {
            let then_ok = assigns_definitely(then, name) || !completes_normally(then);
            let else_ok = assigns_definitely(els, name) || !completes_normally(els);
            then_ok && else_ok && (assigns_definitely(then, name) || assigns_definitely(els, name))
        }
        Stmt::Labeled { body, .. } => assigns_definitely(body, name),
        // A loop body may not run, and a `try` body may fault partway; both
        // are treated as not assigning. `do`/`while (true)` could be refined,
        // but staying conservative here only ever accepts more.
        _ => false,
    }
}

fn assigns_definitely_block(statements: &[Stmt], name: &str) -> bool {
    statements.iter().any(|s| assigns_definitely(s, name))
}

/// The span of a second unconditional assignment to `name`, if there is one.
///
/// Only straight-line assignments count: two writes on opposite branches of an
/// `if` are legal, and this must never invent an error.
/// Where this block first assigns the field named `name`, at the top level or
/// in a nested block. Used to place the "might already have been assigned"
/// error on a constructor's assignment when an instance initializer already
/// made it — the assignment is the SECOND one whether or not it is definite.
fn first_assignment(statements: &[Stmt], name: &str) -> Option<SourceSpan> {
    statements.iter().find_map(|statement| match statement {
        Stmt::Assign { target, span, .. } if is_field_target(target, name) => Some(*span),
        Stmt::Block(body) => first_assignment(body, name),
        _ => None,
    })
}

fn assigns_twice(statements: &[Stmt], name: &str) -> Option<SourceSpan> {
    let mut seen = false;
    for statement in statements {
        match statement {
            Stmt::Assign { target, span, .. } if is_field_target(target, name) => {
                if seen {
                    return Some(*span);
                }
                seen = true;
            }
            Stmt::Block(body) => {
                if let Some(span) = assigns_twice(body, name) {
                    return Some(span);
                }
                if assigns_definitely_block(body, name) {
                    if seen {
                        return body.iter().find_map(|s| match s {
                            Stmt::Assign { target, span, .. } if is_field_target(target, name) => {
                                Some(*span)
                            }
                            _ => None,
                        });
                    }
                    seen = true;
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether an assignment target is the bare field `name` or `this.name`.
fn is_field_target(target: &crate::ast::AssignTarget, name: &str) -> bool {
    match target {
        crate::ast::AssignTarget::Var(var) => var == name,
        // `this.x = ...`; any other qualifier writes some other object.
        crate::ast::AssignTarget::Field {
            object,
            name: field,
        } => field == name && matches!(**object, Expr::This { .. }),
        crate::ast::AssignTarget::Index { .. } => false,
    }
}

/// The span of a read of blank final `name` that happens before the field is
/// definitely assigned (JLS §8.3.3, §16.9) — `final int f; C() { print(f); f = 1; }`
/// silently read 0.
///
/// Walks in execution order so an assignment *inside* a branch still covers the
/// reads after it in that same branch; a read is only reported while the field
/// is not yet assigned on the path reaching it.
fn read_before_assignment(body: &[Stmt], name: &str) -> Option<SourceSpan> {
    let mut scan = Scan {
        name,
        assigned: false,
        found: None,
    };
    scan.block(body);
    scan.found
}

struct Scan<'a> {
    name: &'a str,
    assigned: bool,
    found: Option<SourceSpan>,
}

impl Scan<'_> {
    fn block(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.stmt(statement);
        }
    }

    #[allow(clippy::too_many_lines)] // one arm per statement kind
    fn stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign {
                target,
                op,
                value,
                span,
            } => {
                // The right-hand side is evaluated first, so `f = f + 1` reads
                // an unassigned field. A compound `f += 1` reads it too.
                self.expr(value);
                if is_field_target(target, self.name) {
                    if op.is_some() && !self.assigned {
                        self.report(*span);
                    }
                    self.assigned = true;
                }
            }
            Stmt::Block(body) => self.block(body),
            Stmt::If {
                cond, then, els, ..
            } => {
                self.expr(cond);
                let before = self.assigned;
                self.stmt(then);
                let after_then = self.assigned;
                self.assigned = before;
                if let Some(els) = els {
                    self.stmt(els);
                }
                // Assigned afterwards only if both paths assigned.
                self.assigned = after_then && self.assigned;
            }
            // A loop body may run zero times, so nothing it assigns counts
            // afterwards — but a read inside it still happens after whatever
            // came before.
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                let before = self.assigned;
                self.stmt(body);
                self.assigned = before;
            }
            Stmt::DoWhile { body, cond, .. } => {
                let before = self.assigned;
                self.stmt(body);
                let after_body = self.assigned;
                self.expr(cond);
                // The body always runs once, so its assignment does count.
                self.assigned = after_body || before;
            }
            Stmt::For {
                init,
                cond,
                update,
                body,
                ..
            } => {
                if let Some(init) = init {
                    self.stmt(init);
                }
                let before = self.assigned;
                if let Some(cond) = cond {
                    self.expr(cond);
                }
                self.stmt(body);
                self.block(update);
                self.assigned = before;
            }
            Stmt::ForEach { iterable, body, .. } => {
                self.expr(iterable);
                let before = self.assigned;
                self.stmt(body);
                self.assigned = before;
            }
            Stmt::Labeled { body, .. } => self.stmt(body),
            Stmt::Expr(expr) | Stmt::Throw { value: expr, .. } => self.expr(expr),
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.expr(value);
                }
            }
            Stmt::LocalDecl { declarators, .. } => {
                for declarator in declarators {
                    if let Some(init) = &declarator.init {
                        self.expr(init);
                    }
                }
            }
            Stmt::SuperCall { args, .. } | Stmt::ThisCall { args, .. } => {
                for arg in args {
                    self.expr(arg);
                }
            }
            Stmt::Switch { selector, arms, .. } => {
                self.expr(selector);
                let before = self.assigned;
                for arm in arms {
                    self.assigned = before;
                    self.block(&arm.body);
                }
                self.assigned = before;
            }
            Stmt::Try {
                body,
                catches,
                finally_body,
                ..
            } => {
                let before = self.assigned;
                self.block(body);
                // A catch runs from a partially-executed try, so treat the
                // try's assignments as not having happened.
                for clause in catches {
                    self.assigned = before;
                    self.block(&clause.body);
                }
                self.assigned = before;
                if let Some(finally_body) = finally_body {
                    self.block(finally_body);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn report(&mut self, span: SourceSpan) {
        if self.found.is_none() {
            self.found = Some(span);
        }
    }

    fn expr(&mut self, expr: &Expr) {
        // A read of the field before it is assigned is the error being hunted.
        if !self.assigned && reads_name(expr, self.name) {
            self.report(expr.span());
            return;
        }
        match expr {
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Unary { operand, .. }
            | Expr::Cast { operand, .. }
            | Expr::InstanceOf { value: operand, .. }
            | Expr::IncDec {
                target: operand, ..
            } => self.expr(operand),
            Expr::Index { array, index, .. } => {
                self.expr(array);
                self.expr(index);
            }
            Expr::Field { object, .. }
            | Expr::MethodRef {
                qualifier: object, ..
            } => {
                self.expr(object);
            }
            Expr::Call { receiver, args, .. } => {
                if let Some(receiver) = receiver {
                    self.expr(receiver);
                }
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::NewObject { args, outer, .. } => {
                if let Some(outer) = outer {
                    self.expr(outer);
                }
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::SuperMethodCall { args, .. } | Expr::ArrayLiteral { elements: args, .. } => {
                for arg in args {
                    self.expr(arg);
                }
            }
            Expr::NewArray { dims, init, .. } => {
                for dim in dims.iter().flatten() {
                    self.expr(dim);
                }
                for element in init.iter().flatten() {
                    self.expr(element);
                }
            }
            Expr::Ternary {
                cond, then, els, ..
            } => {
                self.expr(cond);
                self.expr(then);
                self.expr(els);
            }
            Expr::Assign { value, .. } => self.expr(value),
            // A lambda body runs later, not here.
            Expr::Literal { .. }
            | Expr::Name { .. }
            | Expr::This { .. }
            | Expr::Super { .. }
            | Expr::Lambda { .. } => {}
        }
    }
}

/// Whether this expression IS a read of the field `name` (a bare simple name,
/// or `this.name`).
fn reads_name(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Name { path, .. } => path.len() == 1 && path[0] == name,
        Expr::Field {
            object,
            name: field,
            ..
        } => field == name && matches!(**object, Expr::This { .. }),
        _ => false,
    }
}

/// Whether any local declaration, for-each variable or catch parameter
/// anywhere in `statements` uses `name`.
fn declares_local_named(statements: &[Stmt], name: &str) -> bool {
    statements.iter().any(|statement| match statement {
        Stmt::LocalDecl { declarators, .. } => declarators.iter().any(|d| d.name == name),
        Stmt::ForEach {
            name: each, body, ..
        } => each == name || declares_local_named(std::slice::from_ref(body), name),
        Stmt::Block(body) => declares_local_named(body, name),
        Stmt::If { then, els, .. } => {
            declares_local_named(std::slice::from_ref(then), name)
                || els
                    .as_deref()
                    .is_some_and(|e| declares_local_named(std::slice::from_ref(e), name))
        }
        Stmt::While { body, .. } | Stmt::DoWhile { body, .. } | Stmt::Labeled { body, .. } => {
            declares_local_named(std::slice::from_ref(body), name)
        }
        Stmt::For {
            init, update, body, ..
        } => {
            init.as_deref()
                .is_some_and(|i| declares_local_named(std::slice::from_ref(i), name))
                || declares_local_named(update, name)
                || declares_local_named(std::slice::from_ref(body), name)
        }
        Stmt::Switch { arms, .. } => arms.iter().any(|a| declares_local_named(&a.body, name)),
        Stmt::Try {
            body,
            catches,
            finally_body,
            ..
        } => {
            declares_local_named(body, name)
                || catches
                    .iter()
                    .any(|c| c.name == name || declares_local_named(&c.body, name))
                || finally_body
                    .as_ref()
                    .is_some_and(|f| declares_local_named(f, name))
        }
        _ => false,
    })
}

/// The four messages the FLOW phase produces. javac runs flow analysis only
/// after attribution finished without errors, so a program that has both a
/// type error and a missing return is reported with the type error ALONE —
/// which is not a detail: it is the FIRST error a student is shown, and
/// caturra was showing a different one. (`variable x might already have been
/// assigned` is the blank-final rule, reported by the same phase.)
fn is_flow_message(message: &str) -> bool {
    let first = message.lines().next().unwrap_or(message);
    first == "missing return statement"
        || first == "unreachable statement"
        || (first.starts_with("variable ")
            && (first.ends_with(" might not have been initialized")
                || first.ends_with(" might already have been assigned")))
}

/// Drop the flow phase's errors when any OTHER error was reported, whatever
/// class it came from — javac's granularity is the whole compilation, not the
/// method or the class (a type error in one nested class hides a missing
/// return in its sibling).
pub fn drop_flow_errors_after_other_errors(diagnostics: &mut Vec<Diagnostic>) {
    let other = diagnostics.iter().any(|d| {
        matches!(d.severity, crate::Severity::Error) && !is_flow_message(&d.message)
    });
    if other {
        diagnostics
            .retain(|d| !matches!(d.severity, crate::Severity::Error) || !is_flow_message(&d.message));
    }
}

#[cfg(test)]
mod tests {
    use crate::{SourceFile, compile};

    fn errors(source: &str) -> Vec<String> {
        compile(&[SourceFile {
            path: String::from("M.java"),
            text: source.to_owned(),
        }])
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect()
    }

    fn rejects_with(source: &str, message: &str) {
        let found = errors(source);
        assert!(
            found.iter().any(|m| m.contains(message)),
            "expected {message:?}, got {found:?}"
        );
    }

    fn accepts(source: &str) {
        let found = errors(source);
        assert!(found.is_empty(), "expected no errors, got {found:?}");
    }

    #[test]
    fn unreachable_after_abrupt_completion() {
        rejects_with(
            "class M { static void f() { return; int x = 1; } }",
            "unreachable statement",
        );
        rejects_with(
            "class M { static int f() { throw new RuntimeException(); int y = 1; } }",
            "unreachable statement",
        );
        // But a CALL to a method that always throws completes normally as far
        // as JLS §14.21 is concerned — reachability is syntactic, and only a
        // `throw` statement itself ends the flow.
        accepts(
            "class M { static int f() { throw new RuntimeException(); } static void g() { f(); int x = 1; } }",
        );
        rejects_with(
            "class M { static void f() { for (;;) { break; int x = 1; } } }",
            "unreachable statement",
        );
        rejects_with(
            "class M { static void f() { for (int i = 0; i < 1; i++) { continue; int x = 1; } } }",
            "unreachable statement",
        );
        // The body of a constant-false loop can never run.
        rejects_with(
            "class M { static void f() { while (false) { int x = 1; } } }",
            "unreachable statement",
        );
        rejects_with(
            "class M { static void f() { for (; false;) { int x = 1; } } }",
            "unreachable statement",
        );
    }

    #[test]
    fn if_false_is_legal_conditional_compilation() {
        // JLS §14.21 exempts `if` on purpose, so this is NOT an error even
        // though the body cannot run — unlike the `while (false)` above.
        accepts("class M { static void f() { if (false) { int x = 1; } } }");
        accepts("class M { static void f() { if (true) { } else { int x = 1; } } }");
    }

    #[test]
    fn reachable_code_after_loops_and_branches_is_accepted() {
        // A `while (true)` with a break DOES complete normally.
        accepts("class M { static void f() { while (true) { break; } int x = 1; } }");
        // Only one branch is abrupt, so the `if` completes normally.
        accepts("class M { static void f(int n) { if (n > 0) return; int x = 1; } }");
        // A do-while body always runs, whatever the condition.
        accepts("class M { static void f() { do { int x = 1; } while (false); } }");
        // A labeled break makes the labeled statement complete normally.
        accepts("class M { static void f() { outer: { break outer; } int x = 1; } }");
        // A nested loop's unlabeled break does not end the outer one.
        accepts(
            "class M { static void f() { while (true) { while (true) { break; } break; } int x = 1; } }",
        );
    }

    #[test]
    fn blank_final_fields_must_be_definitely_assigned() {
        rejects_with(
            "class M { final int f; M() { } }",
            "variable f might not have been initialized",
        );
        // No constructor at all: the default one assigns nothing.
        rejects_with(
            "class M { final int f; }",
            "variable f might not have been initialized",
        );
        // Assigned on only one branch.
        rejects_with(
            "class M { final int f; M(int n) { if (n > 0) f = n; } }",
            "variable f might not have been initialized",
        );
        // Read before it is assigned.
        rejects_with(
            "class M { final int f; M() { int y = f; f = 1; } }",
            "variable f might not have been initialized",
        );
        // Assigned twice — `final` must mean final.
        rejects_with(
            "class M { final int f; M() { f = 1; f = 2; } }",
            "variable f might already have been assigned",
        );
    }

    #[test]
    fn blank_final_fields_assigned_every_way_are_accepted() {
        accepts("class M { final int f; M() { f = 1; } }");
        accepts("class M { final int f; M(int n) { if (n > 0) { f = 1; } else { f = 2; } } }");
        accepts("class M { final int f; M() { this(1); } M(int n) { f = n; } }");
        // An instance initializer block assigns before every constructor.
        accepts("class M { final int f; { f = 1; } M() { } }");
        // A field with an initializer is not blank at all.
        accepts("class M { final int f = 1; M() { } }");
        // Reading AFTER the assignment is fine, including inside a branch.
        accepts("class M { final int f; M(int n) { f = n; int y = f; } }");
        // A static final is initialized elsewhere and is not this rule's business.
        accepts("class M { static final int F = 1; }");
        // A local named like the field shadows it, so no read is involved.
        accepts("class M { final int f; M() { int f = 2; this.f = f; } }");
    }
}
