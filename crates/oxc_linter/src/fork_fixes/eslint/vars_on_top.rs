//! `eslint/vars-on-top`: move `var` declarations to the top of their scope.
//!
//! `var` is hoisted to the top of its function (or program, or static block) anyway, so the binding
//! is declared there as `var a, b;` and the original declaration keeps only its initializers as
//! assignments: `var a = 1;` becomes `a = 1;`, and a bare `var a;` is removed (or left as `;` when
//! removing it would join its neighbors). The top is after directives and imports. Declarations on
//! lines where the rule is disabled are not moved.
//!
//! All misplaced declarations of one scope are moved together by the fix on the first of them, so
//! one pass is enough; in an editor, that first declaration carries the quick fix. Only plain
//! identifier declarators directly in a statement list are moved; destructuring, `for (var ..)`,
//! `declare` and declarations containing comments stay where they are. A dangerous fix, because it
//! moves code around.

use std::cell::RefCell;

use rustc_hash::FxHashMap;

use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, Declaration, Expression, Statement, VariableDeclaration,
        VariableDeclarationKind,
    },
};
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        nodes::{FileKey, file_key},
        util::{can_delete_statement, descendants, is_statement_list, line_indent},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::VariableDeclaration(_)))
    else {
        return vec![];
    };
    let nodes = fcx.nodes();
    let Some(scope) = scope_of(fcx, node.id()) else { return vec![] };
    let (statements, is_static_block) = match nodes.kind(scope) {
        AstKind::Program(program) => (&program.body, false),
        AstKind::FunctionBody(body) => (&body.statements, false),
        AstKind::StaticBlock(block) => (&block.body, true),
        _ => return vec![],
    };

    // The top, as upstream defines it: after directive-like strings and imports (not in a static
    // block), the leading run of variable declarations, exported ones included.
    let first = if is_static_block {
        0
    } else {
        statements
            .iter()
            .position(|statement| !looks_like_directive_or_import(statement))
            .unwrap_or(statements.len())
    };
    let mut top_end = first;
    while statements.get(top_end).is_some_and(is_variable_declaration) {
        top_end += 1;
    }
    // Right after the leading declarations, before the first other statement.
    let Some(insert_at) = statements.get(top_end).map(|statement| statement.span().start) else {
        return vec![];
    };

    let AstKind::VariableDeclaration(declaration) = node.kind() else { return vec![] };
    if !movable(fcx, node.id(), declaration) {
        return vec![];
    }

    // Only the first misplaced declaration of the scope gets a fix, which moves all of them.
    // Fixes for the others would also insert at the top, and the fixer prefers the shortest
    // overlapping fix, so one declaration would move per pass. The scope is scanned once.
    if first_misplaced(fcx, scope, insert_at) != Some(span) {
        return vec![];
    }
    let mut names = vec![];
    let mut edits = vec![];
    for candidate in misplaced(fcx, scope, insert_at) {
        edits.push(hoist(fcx, candidate, &mut names));
    }

    let fixer = fcx.fixer(FixKind::DangerousFix).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(edits.len() + 1);
    let (_, indent) = line_indent(fcx.source_text(), insert_at);
    fix.push(Fix::new(format!("var {};\n{indent}", names.join(", ")), Span::empty(insert_at)));
    fix.push(edits);
    vec![fix.with_message("Move the `var` declarations to the top")]
}

/// The edit that leaves only the initializers of `declaration`, adding its names to `names`.
fn hoist(fcx: &FixCtx, declaration: &VariableDeclaration, names: &mut Vec<String>) -> Fix {
    let mut assignments = vec![];
    for declarator in &declaration.declarations {
        let BindingPattern::BindingIdentifier(ident) = &declarator.id else { continue };
        let annotation =
            declarator.type_annotation.as_ref().map_or("", |t| fcx.source_range(t.span));
        names.push(format!("{}{annotation}", ident.name));
        if let Some(init) = &declarator.init {
            assignments.push(format!("{} = {};", ident.name, fcx.source_range(init.span())));
        }
    }
    // Removing a bare `var a` without a semicolon could join its neighbors through automatic
    // semicolon insertion (`foo()\nvar a\n(bar)()`), so an empty statement stays in its place.
    if assignments.is_empty() && !can_delete_statement(fcx, declaration.span) {
        return Fix::new(";", declaration.span);
    }
    Fix::new(assignments.join(" "), declaration.span)
}

/// Movable `var` declarations of `scope` after its leading block of declarations, which ends at
/// `insert_at`. Declarations of `scope` that start earlier are in that block. Declarations whose
/// diagnostic is suppressed by a disable directive stay where they are.
fn misplaced<'c, 'a>(
    fcx: &'c FixCtx<'c, 'a>,
    scope: NodeId,
    insert_at: u32,
) -> impl Iterator<Item = &'c VariableDeclaration<'a>> + 'c {
    descendants(fcx, scope).filter_map(move |candidate| {
        let AstKind::VariableDeclaration(declaration) = candidate.kind() else { return None };
        (declaration.span.start >= insert_at
            && !fcx.ctx.disable_directives().contains("vars-on-top", declaration.span)
            && movable(fcx, candidate.id(), declaration)
            && scope_of(fcx, candidate.id()) == Some(scope))
        .then_some(declaration)
    })
}

/// The file the cache belongs to, and the first misplaced declaration of each scope.
type FirstMisplacedCache = (Option<FileKey>, FxHashMap<NodeId, Option<Span>>);

thread_local! {
    /// First misplaced declaration per scope, for the file being linted.
    static FIRST_MISPLACED: RefCell<FirstMisplacedCache> =
        const { RefCell::new((None, FxHashMap::with_hasher(rustc_hash::FxBuildHasher))) };
}

fn first_misplaced(fcx: &FixCtx, scope: NodeId, insert_at: u32) -> Option<Span> {
    let key = file_key(fcx.nodes());
    let cached = FIRST_MISPLACED.with_borrow_mut(|(cached_key, cache)| {
        if *cached_key != Some(key) {
            *cached_key = Some(key);
            cache.clear();
        }
        cache.get(&scope).copied()
    });
    // A cached span from an earlier file at the same address is caught by checking it is a
    // movable declaration of this scope.
    if let Some(cached) = cached
        && cached.is_none_or(|span| {
            fcx.exact(span, |node| matches!(node.kind(), AstKind::VariableDeclaration(_)))
                .is_some_and(|node| scope_of(fcx, node.id()) == Some(scope))
        })
    {
        return cached;
    }
    let first = misplaced(fcx, scope, insert_at).next().map(|declaration| declaration.span);
    FIRST_MISPLACED.with_borrow_mut(|(_, cache)| cache.insert(scope, first));
    first
}

fn looks_like_directive_or_import(statement: &Statement) -> bool {
    match statement {
        Statement::ImportDeclaration(_) => true,
        Statement::ExpressionStatement(statement) => {
            matches!(statement.expression, Expression::StringLiteral(_))
        }
        _ => false,
    }
}

fn is_variable_declaration(statement: &Statement) -> bool {
    match statement {
        Statement::VariableDeclaration(_) => true,
        Statement::ExportDeclaration(export) => {
            matches!(export.declaration, Declaration::VariableDeclaration(_))
        }
        _ => false,
    }
}

/// The function body, program or static block whose `var` scope contains `node_id`. A TypeScript
/// namespace block is its own scope, which this fixer does not handle (the caller gets no match).
fn scope_of(fcx: &FixCtx, node_id: NodeId) -> Option<NodeId> {
    fcx.nodes().ancestor_ids(node_id).find(|&id| {
        matches!(
            fcx.nodes().kind(id),
            AstKind::FunctionBody(_)
                | AstKind::Program(_)
                | AstKind::StaticBlock(_)
                | AstKind::TSModuleBlock(_)
        )
    })
}

/// A plain `var` with identifier declarators, directly in a statement list, without comments.
fn movable(fcx: &FixCtx, node_id: NodeId, declaration: &VariableDeclaration) -> bool {
    declaration.kind == VariableDeclarationKind::Var
        && !declaration.declare
        && is_statement_list(&fcx.nodes().parent_kind(node_id))
        && declaration
            .declarations
            .iter()
            .all(|declarator| matches!(declarator.id, BindingPattern::BindingIdentifier(_)))
        && !fcx.ctx.has_comments_between(declaration.span)
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::vars_on_top::VarsOnTop, tester::Tester};

    let pass = vec!["var a; a = 1;"];
    let fail = vec!["foo(); var a = 1;"];

    let fix = vec![
        ("foo(); var a = 1;", "var a;\nfoo(); a = 1;", None, FixKind::DangerousFix),
        ("foo(); var a = 1;", "foo(); var a = 1;", None, FixKind::SafeFix),
        (
            "foo(); var a = 1, b; bar(); var c;",
            "var a, b, c;\nfoo(); a = 1; bar(); ",
            None,
            FixKind::DangerousFix,
        ),
        (
            "function f() {\n  foo();\n  if (x) { var a = 1; }\n}",
            "function f() {\n  var a;\n  foo();\n  if (x) { a = 1; }\n}",
            None,
            FixKind::DangerousFix,
        ),
        (
            "var z = 0; foo(); var a = 1;",
            "var z = 0; var a;\nfoo(); a = 1;",
            None,
            FixKind::DangerousFix,
        ),
        // Exported declarations belong to the top, as upstream counts them.
        (
            "export const k = 1; var a; foo(); if (x) { var b; }",
            "export const k = 1; var a; var b;\nfoo(); if (x) {  }",
            None,
            FixKind::DangerousFix,
        ),
        // A namespace is its own scope: its `var`s are not moved out of it.
        (
            "foo(); var a = 1; namespace N { if (c) { var b = 2; } }",
            "var a;\nfoo(); a = 1; namespace N { if (c) { var b = 2; } }",
            None,
            FixKind::DangerousFix,
        ),
        // Removing `var a` would join `foo()` and `(bar)()`, so an empty statement stays.
        ("foo()\nvar a\n(bar)()", "var a;\nfoo()\n;\n(bar)()", None, FixKind::DangerousFix),
        // Declarations on suppressed lines stay where they are, whichever comes first.
        (
            "foo();\n// eslint-disable-next-line vars-on-top\nvar a = 1;\nbar();\nvar b = 2;",
            "var b;\nfoo();\n// eslint-disable-next-line vars-on-top\nvar a = 1;\nbar();\nb = 2;",
            None,
            FixKind::DangerousFix,
        ),
        (
            "foo();\nvar a = 1;\nbar();\n// eslint-disable-next-line vars-on-top\nvar b = 2;",
            "var a;\nfoo();\na = 1;\nbar();\n// eslint-disable-next-line vars-on-top\nvar b = 2;",
            None,
            FixKind::DangerousFix,
        ),
        // Destructuring stays where it is.
        ("foo(); var { a } = o;", "foo(); var { a } = o;", None, FixKind::DangerousFix),
    ];

    Tester::new(VarsOnTop::NAME, VarsOnTop::PLUGIN, pass, fail).expect_fix(fix).test();
}
