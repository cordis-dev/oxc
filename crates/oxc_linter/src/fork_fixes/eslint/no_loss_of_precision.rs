//! `eslint/no-loss-of-precision`: suggest the value the literal actually has at runtime.
//!
//! `9007199254740993` becomes `9007199254740992`. A suggestion, because the author probably wanted
//! the digits they wrote, which a `Number` cannot hold (a `BigInt` may be the real fix).

use oxc_ast::AstKind;
use oxc_syntax::number::ToJsString;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::NumericLiteral(_)))
    else {
        return vec![];
    };
    let AstKind::NumericLiteral(literal) = node.kind() else { return vec![] };
    if !literal.value.is_finite() {
        return vec![];
    }
    let value = literal.value.to_js_string();
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(span, value.clone())
            .with_message(format!("Replace with the actual value `{value}`")),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_loss_of_precision::NoLossOfPrecision, tester::Tester,
    };

    let pass = vec!["var x = 12345"];
    let fail = vec!["var x = 9007199254740993"];

    let fix = vec![
        ("var x = 9007199254740993", "var x = 9007199254740992", None, FixKind::Suggestion),
        ("var x = 9007199254740993", "var x = 9007199254740993", None, FixKind::SafeFix),
        ("var x = 5123000000000000000000000000001", "var x = 5.123e+30", None, FixKind::Suggestion),
        ("var x = 1.0000000000000000000000123", "var x = 1", None, FixKind::Suggestion),
    ];

    Tester::new(NoLossOfPrecision::NAME, NoLossOfPrecision::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
