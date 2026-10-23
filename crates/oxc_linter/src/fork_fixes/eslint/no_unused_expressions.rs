//! `eslint/no-unused-expressions`: suggest removing the statement.
//!
//! A suggestion, because evaluating the expression may still matter (a property read can run a
//! getter or throw) or the author may have meant to use its value. Only for statements in a
//! statement list, where removing them leaves valid code.

use oxc_ast::AstKind;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::can_remove_statement},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::ExpressionStatement(_)))
    else {
        return vec![];
    };
    if !can_remove_statement(fcx, node.id(), span) {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::Suggestion)
            .delete_range(span)
            .with_message("Remove the unused expression"),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_unused_expressions::NoUnusedExpressions, tester::Tester,
    };

    let pass = vec!["f()"];
    let fail = vec!["0"];

    let fix = vec![
        ("f(); a.b; g();", "f();  g();", None, FixKind::Suggestion),
        ("f(); a.b; g();", "f(); a.b; g();", None, FixKind::SafeFix),
        ("if (x) a;", "if (x) a;", None, FixKind::Suggestion),
    ];

    Tester::new(NoUnusedExpressions::NAME, NoUnusedExpressions::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
