//! `eslint/no-lone-blocks`: remove the braces of an unnecessary block.
//!
//! An empty block is deleted. Otherwise the braces are removed and the statements stay. Only when the
//! block sits directly in a statement list. No fix when the block declares a function (block-level
//! function semantics differ), when a `let`, `const` or `class` would move into a function body
//! where it could clash with a parameter, or when removing the braces could join statements through
//! automatic semicolon insertion.

use oxc_ast::{AstKind, ast::Declaration};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{
            asi_hazard_after, can_delete_statement, is_statement_list, safe_statement_start,
            statement_boundary_before,
        },
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::BlockStatement(_)))
    else {
        return vec![];
    };
    let AstKind::BlockStatement(block) = node.kind() else { return vec![] };
    let parent = fcx.nodes().parent_kind(node.id());
    if !is_statement_list(&parent) {
        return vec![];
    }

    if block.body.is_empty() {
        if fcx.ctx.has_comments_between(span) || !can_delete_statement(fcx, span) {
            return vec![];
        }
        return vec![
            fcx.fixer(FixKind::SafeFix).delete_range(span).with_message("Remove the empty block"),
        ];
    }

    // Lexical declarations may move into a plain block or static block, which has no other bindings
    // (the reported block is its only statement). A catch body shares its scope with the catch
    // parameter, so a moved `let e` could clash with `catch (e)`.
    let parent_id = fcx.nodes().parent_id(node.id());
    let lexical_allowed = match parent {
        AstKind::BlockStatement(_) => {
            !matches!(fcx.nodes().parent_kind(parent_id), AstKind::CatchClause(_))
        }
        AstKind::StaticBlock(_) => true,
        _ => false,
    };
    for statement in &block.body {
        match statement.as_declaration() {
            Some(Declaration::FunctionDeclaration(_)) => return vec![],
            Some(Declaration::VariableDeclaration(decl))
                if !decl.kind.is_var() && !lexical_allowed =>
            {
                return vec![];
            }
            Some(Declaration::ClassDeclaration(_)) if !lexical_allowed => return vec![],
            _ => {}
        }
    }

    // Without `{`, the first statement could join the previous one; without `}`, the next statement
    // could join the last one.
    let first = &block.body[0];
    let last = &block.body[block.body.len() - 1];
    if !safe_statement_start(fcx, block.span.start, fcx.source_range(first.span())) {
        return vec![];
    }
    // A `}` ending a function expression or object literal does not end the statement.
    let last_ends_safely = statement_boundary_before(fcx, block.span.end - 1);
    if !last_ends_safely && asi_hazard_after(fcx, last.span().end, block.span.end) {
        return vec![];
    }

    let fixer = fcx.fixer(FixKind::SafeFix).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    fix.push(Fix::delete(Span::sized(block.span.start, 1)));
    fix.push(Fix::delete(Span::new(block.span.end - 1, block.span.end)));
    vec![fix.with_message("Remove the unnecessary braces")]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_lone_blocks::NoLoneBlocks, tester::Tester};

    let pass = vec!["if (foo) { bar(); }"];
    let fail = vec!["{}"];

    let fix = vec![
        ("{}", "", None, FixKind::SafeFix),
        ("foo(); {} bar();", "foo();  bar();", None, FixKind::SafeFix),
        ("{ foo(); }", " foo(); ", None, FixKind::SafeFix),
        ("{ var x = 1; }", " var x = 1; ", None, FixKind::SafeFix),
        ("function f() { { foo(); } }", "function f() {  foo();  }", None, FixKind::SafeFix),
        // The outer braces go first; the inner block then keeps `let` scoped and is no longer reported.
        ("{ { let x = 1; } }", " { let x = 1; } ", None, FixKind::SafeFix),
        ("foo();\n{ (a) }", "foo();\n (a) ", None, FixKind::SafeFix),
        // `let` would move into the function body, where it could clash with a parameter.
        (
            "function f(x) { { let x = 1; } }",
            "function f(x) { { let x = 1; } }",
            None,
            FixKind::SafeFix,
        ),
        // `let` would move into the catch body, where it could clash with the catch parameter.
        (
            "try {} catch (e) { { let e = 1; } }",
            "try {} catch (e) { { let e = 1; } }",
            None,
            FixKind::SafeFix,
        ),
        (
            "try {} catch (e) { { class e {} } }",
            "try {} catch (e) { { class e {} } }",
            None,
            FixKind::SafeFix,
        ),
        // The first statement could join the previous line.
        ("foo()\n{ (a) }", "foo()\n{ (a) }", None, FixKind::SafeFix),
        // The last statement could join the next one.
        ("{ foo() }\n(a)", "{ foo() }\n(a)", None, FixKind::SafeFix),
        // A `}` that ends a function expression does not end the statement.
        (
            "var cb = function () {}\n{ (init)(); }",
            "var cb = function () {}\n{ (init)(); }",
            None,
            FixKind::SafeFix,
        ),
        ("{ x = function () {} }\n(a)", "{ x = function () {} }\n(a)", None, FixKind::SafeFix),
        // After a block statement or a function declaration it does.
        ("if (x) {}\n{ (a)(); }", "if (x) {}\n (a)(); ", None, FixKind::SafeFix),
        ("function f() {}\n{ (a)(); }", "function f() {}\n (a)(); ", None, FixKind::SafeFix),
        // Deleting an empty block could join its neighbors.
        ("foo()\n{}\n(bar)()", "foo()\n{}\n(bar)()", None, FixKind::SafeFix),
    ];

    Tester::new(NoLoneBlocks::NAME, NoLoneBlocks::PLUGIN, pass, fail).expect_fix(fix).test();
}
