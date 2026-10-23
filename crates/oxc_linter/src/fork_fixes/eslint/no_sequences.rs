//! `eslint/no-sequences`: split a sequence used as a statement into separate statements.
//!
//! `a(), b();` becomes `a(); b();`, which evaluates the same expressions in the same order and
//! discards their values. Only when the statement sits directly in a statement list: in
//! `if (x) a(), b();` splitting would move `b()` out of the `if`. No fix when a later expression would
//! start a statement with `{`, `function`, `class` or `let`, which would parse differently, or when the
//! sequence starts with a string, which would become a directive.

use oxc_ast::{AstKind, ast::Expression};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{is_statement_list, needs_parens_as_statement, next_char},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(node) =
        fcx.innermost(fcx.span(), |node| matches!(node.kind(), AstKind::SequenceExpression(_)))
    else {
        return vec![];
    };
    let AstKind::SequenceExpression(sequence) = node.kind() else { return vec![] };
    let statement_id = fcx.nodes().parent_id(node.id());
    let AstKind::ExpressionStatement(statement) = fcx.nodes().kind(statement_id) else {
        return vec![];
    };
    if statement.expression.span() != sequence.span
        || !is_statement_list(&fcx.nodes().parent_kind(statement_id))
        || matches!(sequence.expressions.first(), Some(Expression::StringLiteral(_)))
        || sequence
            .expressions
            .iter()
            .skip(1)
            .any(|expr| needs_parens_as_statement(fcx.source_range(expr.span())))
    {
        return vec![];
    }

    let fixer = fcx.fixer(FixKind::SafeFix).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(sequence.expressions.len() - 1);
    for pair in sequence.expressions.windows(2) {
        let Some((comma, ',')) = next_char(fcx, pair[0].span().end) else { return vec![] };
        fix.push(Fix::new(";", Span::sized(comma, 1)));
    }
    vec![fix.with_message("Split into separate statements")]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_sequences::NoSequences, tester::Tester};

    let pass = vec!["a(); b();"];
    let fail = vec!["a(), b();"];

    let fix = vec![
        ("a(), b();", "a(); b();", None, FixKind::SafeFix),
        ("a = 1, b = 2, c()", "a = 1; b = 2; c()", None, FixKind::SafeFix),
        ("function f() { a(), b(); }", "function f() { a(); b(); }", None, FixKind::SafeFix),
        (
            "switch (x) { case 1: a(), b(); }",
            "switch (x) { case 1: a(); b(); }",
            None,
            FixKind::SafeFix,
        ),
        ("a() /* c */, b();", "a() /* c */; b();", None, FixKind::SafeFix),
        // Not in a statement list: `b()` would leave the `if`.
        ("if (x) a(), b();", "if (x) a(), b();", None, FixKind::SafeFix),
        // Not a statement.
        (
            "var x = (a(), b());",
            "var x = (a(), b());",
            Some(serde_json::json!([{ "allowInParentheses": false }])),
            FixKind::SafeFix,
        ),
        // A later expression would parse differently at the start of a statement.
        ("a(), function () {}();", "a(), function () {}();", None, FixKind::SafeFix),
        ("a(), { b } = c;", "a(), { b } = c;", None, FixKind::SafeFix),
        // The string would become a directive.
        (
            "function f() { 'use strict', a(); }",
            "function f() { 'use strict', a(); }",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoSequences::NAME, NoSequences::PLUGIN, pass, fail).expect_fix(fix).test();
}
