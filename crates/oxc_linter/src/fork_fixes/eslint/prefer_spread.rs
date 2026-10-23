//! `eslint/prefer-spread`: replace `.apply()` with spread arguments.
//!
//! `f.apply(undefined, args)` becomes `f(...args)` and `o.f.apply(o, args)` becomes `o.f(...args)`.
//! The rule only reports calls whose `this` argument matches what a direct call would use. A
//! dangerous fix: `apply` accepts any array-like object, while spreading requires an iterable, and
//! `f` might define its own `apply`. No fix for optional chains.

use oxc_ast::AstKind;
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::CallExpression(_)))
    else {
        return vec![];
    };
    let AstKind::CallExpression(call) = node.kind() else { return vec![] };
    if call.optional || matches!(fcx.nodes().parent_kind(node.id()), AstKind::ChainExpression(_)) {
        return vec![];
    }
    let Some(member) = call.callee.get_inner_expression().as_member_expression() else {
        return vec![];
    };
    let Some(arguments) = call.arguments.get(1) else { return vec![] };
    if member.optional() || call.arguments.len() != 2 || arguments.is_spread() {
        return vec![];
    }
    let function = fcx.source_range(member.object().span());
    let arguments = fcx.source_range(arguments.span());
    vec![
        fcx.fixer(FixKind::DangerousFix)
            .replace(span, format!("{function}(...{arguments})"))
            .with_message("Use spread arguments"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::prefer_spread::PreferSpread, tester::Tester};

    let pass = vec!["foo(...args)"];
    let fail = vec!["foo.apply(undefined, args)"];

    let fix = vec![
        ("foo.apply(undefined, args)", "foo(...args)", None, FixKind::DangerousFix),
        ("foo.apply(undefined, args)", "foo.apply(undefined, args)", None, FixKind::SafeFix),
        ("foo.apply(null, args)", "foo(...args)", None, FixKind::DangerousFix),
        ("obj.foo.apply(obj, args)", "obj.foo(...args)", None, FixKind::DangerousFix),
    ];

    Tester::new(PreferSpread::NAME, PreferSpread::PLUGIN, pass, fail).expect_fix(fix).test();
}
