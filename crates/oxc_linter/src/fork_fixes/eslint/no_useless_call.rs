//! `eslint/no-useless-call`: suggest calling the function directly.
//!
//! `f.call(undefined, a)` becomes `f(a)`, `o.f.call(o, a)` becomes `o.f(a)`, and
//! `f.apply(undefined, [a, b])` becomes `f(a, b)`. A suggestion: in strict mode `this` becomes
//! `undefined` instead of `null` for `.call(null)`, and `f` might define its own `call`. For
//! `.apply`, only an array literal without holes can be spread into the arguments. No suggestion for
//! optional chains.

use oxc_ast::{
    AstKind,
    ast::{ArrayExpressionElement, Expression},
};
use oxc_span::GetSpan;
use oxc_str::JSStr;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::arguments_text},
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
    if member.optional() {
        return vec![];
    }
    let Some(method) = member.static_property_name().and_then(JSStr::as_str) else {
        return vec![];
    };
    let function = fcx.source_range(member.object().span());

    let arguments = match method {
        "call" => arguments_text(fcx, call.arguments.get(1..).unwrap_or_default()).to_string(),
        "apply" => match call.arguments.get(1) {
            None => String::new(),
            Some(argument) => {
                let Some(Expression::ArrayExpression(array)) = argument.as_expression() else {
                    return vec![];
                };
                if array
                    .elements
                    .iter()
                    .any(|element| matches!(element, ArrayExpressionElement::Elision(_)))
                {
                    return vec![];
                }
                array
                    .elements
                    .iter()
                    .map(|element| fcx.source_range(element.span()))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        },
        _ => return vec![],
    };
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(span, format!("{function}({arguments})"))
            .with_message("Call the function directly"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_useless_call::NoUselessCall, tester::Tester};

    let pass = vec!["foo.call(obj, 1)"];
    let fail = vec!["foo.call(undefined, 1)"];

    let fix = vec![
        ("foo.call(undefined, 1, 2)", "foo(1, 2)", None, FixKind::Suggestion),
        ("foo.call(undefined, 1, 2)", "foo.call(undefined, 1, 2)", None, FixKind::SafeFix),
        ("foo.call(null)", "foo()", None, FixKind::Suggestion),
        ("obj.foo.call(obj, 1)", "obj.foo(1)", None, FixKind::Suggestion),
        ("foo.apply(undefined, [1, 2])", "foo(1, 2)", None, FixKind::Suggestion),
        ("obj.foo.apply(obj, [])", "obj.foo()", None, FixKind::Suggestion),
        // An array literal with holes cannot be spread into arguments.
        (
            "foo.apply(undefined, [1, , 2])",
            "foo.apply(undefined, [1, , 2])",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoUselessCall::NAME, NoUselessCall::PLUGIN, pass, fail).expect_fix(fix).test();
}
