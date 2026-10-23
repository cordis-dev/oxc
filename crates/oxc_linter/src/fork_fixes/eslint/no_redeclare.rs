//! `eslint/no-redeclare`: turn a repeated `var` declaration into what it actually does.
//!
//! A `var` that redeclares a binding in the same function scope refers to the same variable, so
//! `var x = 1;` is just the assignment `x = 1;` and a bare `var x;` does nothing. Fixed only for a
//! `var` with a single identifier declarator: the initialized form becomes an assignment (also in
//! `for (var x = ..;;)`), `for (var x in/of ..)` becomes `for (x in/of ..)`, and a bare `var x;` in a
//! statement list is deleted. No fix for `let`, `const`, functions, classes, destructuring, `declare`,
//! or redeclarations of built-in globals.

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, VariableDeclarationKind},
};
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{can_delete_statement, is_statement_list},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    if !fcx.text().ends_with("is already defined.") {
        return vec![];
    }
    // The labels are the first declaration, then the redeclaration.
    let Some(redeclaration) = fcx.label_span(1) else { return vec![] };
    let Some(ident) =
        fcx.exact(redeclaration, |node| matches!(node.kind(), AstKind::BindingIdentifier(_)))
    else {
        return vec![];
    };
    let nodes = fcx.nodes();
    let declarator_id = nodes.parent_id(ident.id());
    let AstKind::VariableDeclarator(declarator) = nodes.kind(declarator_id) else { return vec![] };
    let declaration_id = nodes.parent_id(declarator_id);
    let AstKind::VariableDeclaration(declaration) = nodes.kind(declaration_id) else {
        return vec![];
    };
    if declaration.kind != VariableDeclarationKind::Var
        || declaration.declare
        || declaration.declarations.len() != 1
        || declarator.type_annotation.is_some()
        || !matches!(declarator.id, BindingPattern::BindingIdentifier(_))
        || fcx.ctx.has_comments_between(declaration.span)
    {
        return vec![];
    }

    let name = fcx.source_range(redeclaration);
    let parent = nodes.parent_kind(declaration_id);
    let fixer = fcx.fixer(FixKind::SafeFix);
    let fix = match (&declarator.init, parent) {
        // `var x = 1;` → `x = 1;`, also in `for (var x = 1;;)`.
        (Some(init), _) => {
            let in_for = matches!(parent, AstKind::ForStatement(_));
            if !in_for && !is_statement_list(&parent) {
                return vec![];
            }
            let statement_end =
                if !in_for && fcx.source_range(declaration.span).ends_with(';') { ";" } else { "" };
            fixer.replace(
                declaration.span,
                format!("{name} = {}{statement_end}", fcx.source_range(init.span())),
            )
        }
        // `for (var x in o)` → `for (x in o)`.
        (None, AstKind::ForInStatement(_) | AstKind::ForOfStatement(_)) => {
            fixer.replace(declaration.span, name.to_string())
        }
        // `var x;` does nothing.
        (None, parent)
            if is_statement_list(&parent) && can_delete_statement(fcx, declaration.span) =>
        {
            fixer.delete_range(declaration.span)
        }
        (None, _) => return vec![],
    };
    vec![fix.with_message("Remove the redundant declaration")]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_redeclare::NoRedeclare, tester::Tester};

    let pass = vec!["var a = 3; var b = function() { var a = 10; };"];
    let fail = vec!["var a = 3; var a = 10;"];

    let fix = vec![
        ("var a = 3; var a = 10;", "var a = 3; a = 10;", None, FixKind::SafeFix),
        ("var a = 3; var a = 10", "var a = 3; a = 10", None, FixKind::SafeFix),
        ("var a = 3; var a;", "var a = 3; ", None, FixKind::SafeFix),
        ("function f(a) { var a = 1; }", "function f(a) { a = 1; }", None, FixKind::SafeFix),
        (
            "var i; for (var i = 0; i < 3; i++) {}",
            "var i; for (i = 0; i < 3; i++) {}",
            None,
            FixKind::SafeFix,
        ),
        ("var k; for (var k in o) {}", "var k; for (k in o) {}", None, FixKind::SafeFix),
        ("var a = 1; { var a = 2; }", "var a = 1; { a = 2; }", None, FixKind::SafeFix),
        // Not a single plain `var` declarator.
        (
            "var a = 1, b = 2; var a = 3, c;",
            "var a = 1, b = 2; var a = 3, c;",
            None,
            FixKind::SafeFix,
        ),
        ("var a = 1; var { a } = o;", "var a = 1; var { a } = o;", None, FixKind::SafeFix),
        // A comment would be lost.
        (
            "var a = 1; var /* again */ a = 2;",
            "var a = 1; var /* again */ a = 2;",
            None,
            FixKind::SafeFix,
        ),
        // Deleting `var x;` would join `foo()` with `(bar || baz)()`.
        (
            "var x; foo()\nvar x;\n(bar || baz)()",
            "var x; foo()\nvar x;\n(bar || baz)()",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, pass, fail).expect_fix(fix).test();
}
