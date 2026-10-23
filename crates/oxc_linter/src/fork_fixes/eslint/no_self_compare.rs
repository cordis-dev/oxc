//! `eslint/no-self-compare`: suggest `Number.isNaN` for a NaN check.
//!
//! `x !== x` and `x != x` are true only for NaN, so they become `Number.isNaN(x)`; `x === x` and
//! `x == x` become `!Number.isNaN(x)`. A suggestion, because a getter would now run once instead of
//! twice, and the author may have meant to compare two different values. No suggestion for other
//! operators or when `Number` is shadowed.

use oxc_ast::AstKind;
use oxc_span::GetSpan;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_unshadowed_global},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(left) = fcx.label_span(0) else { return vec![] };
    let Some(node) = fcx.innermost(left, |node| {
        matches!(node.kind(), AstKind::BinaryExpression(binary) if binary.left.span() == left)
    }) else {
        return vec![];
    };
    let AstKind::BinaryExpression(binary) = node.kind() else { return vec![] };
    let negate = match binary.operator {
        BinaryOperator::StrictInequality | BinaryOperator::Inequality => false,
        BinaryOperator::StrictEquality | BinaryOperator::Equality => true,
        _ => return vec![],
    };
    if !is_unshadowed_global(fcx, node.id(), "Number") {
        return vec![];
    }
    let operand = fcx.source_range(binary.left.span());
    let check = format!("{}Number.isNaN({operand})", if negate { "!" } else { "" });
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(binary.span, check)
            .with_message("Use `Number.isNaN`"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_self_compare::NoSelfCompare, tester::Tester};

    let pass = vec!["if (x === y) {}"];
    let fail = vec!["if (x === x) {}"];

    let fix = vec![
        ("if (x !== x) {}", "if (Number.isNaN(x)) {}", None, FixKind::Suggestion),
        ("if (x !== x) {}", "if (x !== x) {}", None, FixKind::SafeFix),
        ("if (x === x) {}", "if (!Number.isNaN(x)) {}", None, FixKind::Suggestion),
        ("if (a.b != a.b) {}", "if (Number.isNaN(a.b)) {}", None, FixKind::Suggestion),
        ("if (x < x) {}", "if (x < x) {}", None, FixKind::Suggestion),
        (
            "function f(Number) { return x !== x }",
            "function f(Number) { return x !== x }",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoSelfCompare::NAME, NoSelfCompare::PLUGIN, pass, fail).expect_fix(fix).test();
}
