//! `eslint/no-cond-assign`: suggest marking the assignment as intended with parentheses.
//!
//! `if (a = b)` becomes `if ((a = b))`, which the default `except-parens` mode accepts. A
//! suggestion, because the assignment may be a typo for `===`. Both modes report the same message, so
//! when the assignment is already parenthesized (which only the `always` mode reports) nothing is
//! suggested; adding more parentheses would not help there.

use oxc_ast::AstKind;
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(node) =
        fcx.innermost(fcx.span(), |node| matches!(node.kind(), AstKind::AssignmentExpression(_)))
    else {
        return vec![];
    };
    if matches!(fcx.nodes().parent_kind(node.id()), AstKind::ParenthesizedExpression(_)) {
        return vec![];
    }
    let span = node.kind().span();
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(span, format!("({})", fcx.source_range(span)))
            .with_message("Wrap the assignment in parentheses"),
    ]
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{rule::RuleMeta, rules::eslint::no_cond_assign::NoCondAssign, tester::Tester};

    let pass = vec![("if ((a = b)) {}", None)];
    let fail = vec![("if (a = b) {}", None)];

    let fix = vec![
        ("if (a = b) {}", "if ((a = b)) {}", None, FixKind::Suggestion),
        ("if (a = b) {}", "if (a = b) {}", None, FixKind::SafeFix),
        ("while (a = next()) {}", "while ((a = next())) {}", None, FixKind::Suggestion),
        ("do {} while (a = b)", "do {} while ((a = b))", None, FixKind::Suggestion),
        // `always` mode reports parenthesized assignments too; more parentheses would not help.
        ("if ((a = b)) {}", "if ((a = b)) {}", Some(json!(["always"])), FixKind::Suggestion),
    ];

    Tester::new(NoCondAssign::NAME, NoCondAssign::PLUGIN, pass, fail).expect_fix(fix).test();
}
