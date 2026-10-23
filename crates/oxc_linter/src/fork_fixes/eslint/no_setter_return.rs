//! `eslint/no-setter-return`: stop returning a value from a setter.
//!
//! A setter's return value is always ignored, so `return value;` becomes `{ value; return; }`
//! (or `return;` when the value has no side effects), and a property descriptor's concise
//! `set: v => value` becomes `set: v => { value; }`. Behavior is unchanged, so this is a safe fix.

use oxc_ast::AstKind;
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{as_statement, is_side_effect_free},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    if fcx.ctx.has_comments_between(span) {
        return vec![];
    }
    let fixer = fcx.fixer(FixKind::SafeFix);

    if let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::ReturnStatement(_))) {
        let AstKind::ReturnStatement(statement) = node.kind() else { return vec![] };
        let Some(argument) = &statement.argument else { return vec![] };
        let replacement = if is_side_effect_free(argument) {
            "return;".to_string()
        } else {
            format!("{{ {}; return; }}", as_statement(fcx.source_range(argument.span())))
        };
        return vec![fixer.replace(span, replacement).with_message("Do not return a value")];
    }

    // A concise arrow body; the label excludes its parentheses. Match the arrow whose body is the
    // label: when the setter returns an arrow (`set: v => w => x`), the inner arrow also contains it.
    let Some(arrow) = fcx.innermost(span, |node| {
        matches!(node.kind(), AstKind::ArrowFunctionExpression(arrow)
            if arrow.get_expression().is_some_and(|body| body.without_parentheses().span() == span))
    }) else {
        return vec![];
    };
    let AstKind::ArrowFunctionExpression(arrow) = arrow.kind() else { return vec![] };
    let Some(body) = arrow.get_expression() else { return vec![] };
    let body_span = body.span();
    vec![
        fixer
            .replace(body_span, format!("{{ {}; }}", as_statement(fcx.source_range(body_span))))
            .with_message("Do not return a value"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_setter_return::NoSetterReturn, tester::Tester};

    let pass = vec!["({ set a(val) { return; } })"];
    let fail = vec!["({ set a(val) { return val; } })"];

    let fix = vec![
        (
            "({ set a(val) { return val; } })",
            "({ set a(val) { return; } })",
            None,
            FixKind::SafeFix,
        ),
        (
            "({ set a(val) { return f(val); } })",
            "({ set a(val) { { f(val); return; } } })",
            None,
            FixKind::SafeFix,
        ),
        (
            "class A { set a(val) { if (val) return 1; } }",
            "class A { set a(val) { if (val) return; } }",
            None,
            FixKind::SafeFix,
        ),
        (
            "Object.defineProperty(foo, 'bar', { set: v => w => x })",
            "Object.defineProperty(foo, 'bar', { set: v => { w => x; } })",
            None,
            FixKind::SafeFix,
        ),
        (
            "Object.defineProperty(foo, 'bar', { set: val => f(val) })",
            "Object.defineProperty(foo, 'bar', { set: val => { f(val); } })",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoSetterReturn::NAME, NoSetterReturn::PLUGIN, pass, fail).expect_fix(fix).test();
}
