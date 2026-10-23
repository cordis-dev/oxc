//! `eslint/array-callback-return`: ESLint's suggestions for a `forEach` callback that returns a value.
//!
//! For an arrow function with a concise body, suggest wrapping the body in braces so nothing is
//! returned, and, when the rule's help asks for `void` (the `allowVoid` option), suggest prepending
//! `void`. Suggestions only: either one changes what the callback returns. No suggestion for
//! `return` statements in block bodies, as in ESLint.

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{binds_as_unary_operand, concise_body, wrap_in_block},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    if !fcx.text().starts_with("Unexpected return value in callback") {
        return vec![];
    }
    let Some(returned) = fcx.label_span(1) else { return vec![] };
    let Some(body) = concise_body(fcx, returned) else { return vec![] };

    let text = fcx.source_range(returned);
    let mut fixes = vec![];
    if fcx.help().is_some_and(|help| help.contains("`void`")) {
        let operand =
            if binds_as_unary_operand(body) { text.to_string() } else { format!("({text})") };
        fixes.push(
            fcx.fixer(FixKind::Suggestion)
                .replace(returned, format!("void {operand}"))
                .with_message("Prepend `void` to the expression"),
        );
    }
    fixes.push(wrap_in_block(fcx, returned));
    fixes
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{
        rule::RuleMeta, rules::eslint::array_callback_return::ArrayCallbackReturn, tester::Tester,
    };

    let pass = vec![("foo.forEach(x => { bar(x) })", Some(json!([{ "checkForEach": true }])))];
    let fail = vec![("foo.forEach(x => bar(x))", Some(json!([{ "checkForEach": true }])))];

    let check = Some(json!([{ "checkForEach": true }]));
    let allow_void = Some(json!([{ "checkForEach": true, "allowVoid": true }]));
    let fix = vec![
        (
            "foo.forEach(x => bar(x))",
            "foo.forEach(x => {bar(x)})",
            check.clone(),
            FixKind::Suggestion,
        ),
        ("foo.forEach(x => bar(x))", "foo.forEach(x => bar(x))", check.clone(), FixKind::SafeFix),
        (
            "foo.forEach(x => ({ a: x }))",
            "foo.forEach(x => {({ a: x })})",
            check.clone(),
            FixKind::Suggestion,
        ),
        // A function expression is wrapped so the block does not start with a declaration.
        (
            "foo.forEach(() => function () {})",
            "foo.forEach(() => {(function () {})})",
            check.clone(),
            FixKind::Suggestion,
        ),
        // No suggestion for a `return` in a block body.
        (
            "foo.forEach(x => { return bar(x) })",
            "foo.forEach(x => { return bar(x) })",
            check,
            FixKind::Suggestion,
        ),
    ];
    let fix_void = vec![
        (
            "foo.forEach(x => bar(x))",
            ("foo.forEach(x => void bar(x))", "foo.forEach(x => {bar(x)})"),
            allow_void.clone(),
        ),
        (
            "foo.forEach(x => a || b)",
            ("foo.forEach(x => void (a || b))", "foo.forEach(x => {a || b})"),
            allow_void,
        ),
    ];

    Tester::new(ArrayCallbackReturn::NAME, ArrayCallbackReturn::PLUGIN, pass.clone(), fail.clone())
        .expect_fix(fix)
        .test();
    Tester::new(ArrayCallbackReturn::NAME, ArrayCallbackReturn::PLUGIN, pass, fail)
        .expect_fix(fix_void)
        .test();
}
