//! `eslint/no-dupe-keys`: suggest removing the property that is overwritten.
//!
//! The later property wins, so the earlier one only matters for its side effects. Suggested only
//! when the earlier value has none, and the property's comma is removed with it. A suggestion,
//! because the author may have meant a different key.

use oxc_ast::{AstKind, ast::ObjectPropertyKind};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{is_side_effect_free, next_char},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(first_key) = fcx.label_span(0) else { return vec![] };
    let Some(object) =
        fcx.innermost(first_key, |node| matches!(node.kind(), AstKind::ObjectExpression(_)))
    else {
        return vec![];
    };
    let AstKind::ObjectExpression(object) = object.kind() else { return vec![] };
    let Some(index) = object.properties.iter().position(
        |prop| matches!(prop, ObjectPropertyKind::ObjectProperty(p) if p.key.span() == first_key),
    ) else {
        return vec![];
    };
    let ObjectPropertyKind::ObjectProperty(prop) = &object.properties[index] else { return vec![] };
    if prop.kind != oxc_ast::ast::PropertyKind::Init
        || prop.computed
        || !is_side_effect_free(&prop.value)
    {
        return vec![];
    }

    // Remove the property and the comma after it.
    let removed = match object.properties.get(index + 1) {
        Some(next) => Span::new(prop.span.start, next.span().start),
        None => match next_char(fcx, prop.span.end) {
            Some((comma, ',')) => Span::new(prop.span.start, comma + 1),
            _ => prop.span,
        },
    };
    if fcx.ctx.has_comments_between(removed) {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::Suggestion)
            .delete_range(removed)
            .with_message("Remove the overwritten property"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_dupe_keys::NoDupeKeys, tester::Tester};

    let pass = vec!["var x = { a: 1, b: 2 };"];
    let fail = vec!["var x = { a: 1, a: 2 };"];

    let fix = vec![
        ("var x = { a: 1, a: 2 };", "var x = { a: 2 };", None, FixKind::Suggestion),
        ("var x = { a: 1, a: 2 };", "var x = { a: 1, a: 2 };", None, FixKind::SafeFix),
        ("var x = { a: b, c: 1, a: 2 };", "var x = { c: 1, a: 2 };", None, FixKind::Suggestion),
        ("var x = { 'a': 1, a: 2 };", "var x = { a: 2 };", None, FixKind::Suggestion),
        // The earlier value has side effects.
        ("var x = { a: f(), a: 2 };", "var x = { a: f(), a: 2 };", None, FixKind::Suggestion),
    ];

    Tester::new(NoDupeKeys::NAME, NoDupeKeys::PLUGIN, pass, fail).expect_fix(fix).test();
}
