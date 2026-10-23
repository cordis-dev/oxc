//! `eslint/no-void`: upgrade upstream's suggestion to a safe fix where it is provably correct.
//!
//! Upstream suggests replacing any `void x` with `undefined`. That drops the side effects of `x`
//! and is wrong where `undefined` is shadowed, so it stays a suggestion in general. When the operand
//! is a literal and `undefined` is the unshadowed global, `void 0` and `undefined` are the same value,
//! so the fork applies it as a safe fix. Upstream's suggestion is kept as the second option.

use oxc_ast::{AstKind, ast::Expression};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_unshadowed_global},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::UnaryExpression(_)))
    else {
        return vec![];
    };
    let AstKind::UnaryExpression(unary) = node.kind() else { return vec![] };
    let literal_operand = matches!(
        unary.argument.get_inner_expression(),
        Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
    );
    if !unary.operator.is_void()
        || !literal_operand
        || !is_unshadowed_global(fcx, node.id(), "undefined")
    {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::SafeFix)
            .replace(span, "undefined")
            .with_message("Replace with `undefined`"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_void::NoVoid, tester::Tester};

    let pass = vec!["var foo = bar()"];
    let fail = vec!["void 0"];

    let fix = vec![
        ("var a = void 0", "var a = undefined", None, FixKind::SafeFix),
        ("f(void 0)", "f(undefined)", None, FixKind::SafeFix),
        ("var a = void (0)", "var a = undefined", None, FixKind::SafeFix),
        ("var a = void 'x'", "var a = undefined", None, FixKind::SafeFix),
        // The operand has side effects: only upstream's suggestion.
        ("var a = void foo()", "var a = void foo()", None, FixKind::SafeFix),
        ("var a = void foo()", "var a = undefined", None, FixKind::Suggestion),
        // `undefined` is shadowed.
        (
            "function f(undefined) { return void 0 }",
            "function f(undefined) { return void 0 }",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoVoid::NAME, NoVoid::PLUGIN, pass, fail).expect_fix(fix).test();
}
