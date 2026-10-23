//! `eslint/no-prototype-builtins`: suggest calling the method from `Object.prototype`.
//!
//! `foo.hasOwnProperty(bar)` becomes `Object.prototype.hasOwnProperty.call(foo, bar)`, as ESLint
//! suggests. A suggestion only, because it changes behavior for objects that override the method,
//! which is the point of the rule. No suggestion when `Object` is shadowed, for optional chains, or
//! for `super`.

use oxc_ast::{AstKind, ast::Expression};
use oxc_span::GetSpan;
use oxc_str::JSStr;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{arguments_text, is_unshadowed_global},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.innermost(
        span,
        |node| matches!(node.kind(), AstKind::CallExpression(call) if call.callee.span() == span),
    ) else {
        return vec![];
    };
    let AstKind::CallExpression(call) = node.kind() else { return vec![] };
    if call.optional
        || matches!(fcx.nodes().parent_kind(node.id()), AstKind::ChainExpression(_))
        || !is_unshadowed_global(fcx, node.id(), "Object")
    {
        return vec![];
    }
    let Some(member) = call.callee.as_member_expression() else { return vec![] };
    if member.optional() || matches!(member.object(), Expression::Super(_)) {
        return vec![];
    }
    let Some(method) = member.static_property_name().and_then(JSStr::as_str) else {
        return vec![];
    };

    let object = fcx.source_range(member.object().span());
    let arguments = arguments_text(fcx, &call.arguments);
    let arguments = if arguments.is_empty() { String::new() } else { format!(", {arguments}") };
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(call.span, format!("Object.prototype.{method}.call({object}{arguments})"))
            .with_message(format!("Call `Object.prototype.{method}` instead")),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_prototype_builtins::NoPrototypeBuiltins, tester::Tester,
    };

    let pass = vec!["Object.prototype.hasOwnProperty.call(foo, 'bar')"];
    let fail = vec!["foo.hasOwnProperty('bar')"];

    let fix = vec![
        (
            "foo.hasOwnProperty('bar')",
            "Object.prototype.hasOwnProperty.call(foo, 'bar')",
            None,
            FixKind::Suggestion,
        ),
        ("foo.hasOwnProperty('bar')", "foo.hasOwnProperty('bar')", None, FixKind::SafeFix),
        (
            "foo.bar.isPrototypeOf(baz)",
            "Object.prototype.isPrototypeOf.call(foo.bar, baz)",
            None,
            FixKind::Suggestion,
        ),
        (
            "foo['propertyIsEnumerable']('bar')",
            "Object.prototype.propertyIsEnumerable.call(foo, 'bar')",
            None,
            FixKind::Suggestion,
        ),
        (
            "(a, b).hasOwnProperty(c)",
            "Object.prototype.hasOwnProperty.call((a, b), c)",
            None,
            FixKind::Suggestion,
        ),
        // Optional chains and a shadowed `Object` get no suggestion.
        ("foo?.hasOwnProperty('bar')", "foo?.hasOwnProperty('bar')", None, FixKind::Suggestion),
        (
            "function f(Object) { foo.hasOwnProperty('bar') }",
            "function f(Object) { foo.hasOwnProperty('bar') }",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoPrototypeBuiltins::NAME, NoPrototypeBuiltins::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
