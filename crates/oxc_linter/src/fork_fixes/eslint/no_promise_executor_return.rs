//! `eslint/no-promise-executor-return`: suggestions for a Promise executor that returns a value.
//!
//! - A concise arrow body is wrapped in braces, as ESLint suggests.
//! - `return value;` becomes `{ value; return; }`. The executor's return value is ignored, so this
//!   keeps the code's behavior while making it explicit.
//!
//! Suggestions only, because a returned value there is usually a mistake the author should look at.
//! The `allowVoid` option cannot be seen from the diagnostic, so no `void` suggestion is offered.

use oxc_ast::AstKind;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{as_statement, concise_body, wrap_in_block},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    if concise_body(fcx, span).is_some() {
        return vec![wrap_in_block(fcx, span)];
    }

    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::ReturnStatement(_)))
    else {
        return vec![];
    };
    let AstKind::ReturnStatement(statement) = node.kind() else { return vec![] };
    let Some(argument) = &statement.argument else { return vec![] };
    let value = as_statement(fcx.source_range(oxc_span::GetSpan::span(argument)));
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(span, format!("{{ {value}; return; }}"))
            .with_message("Do not return the value"),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_promise_executor_return::NoPromiseExecutorReturn,
        tester::Tester,
    };

    let pass = vec!["new Promise((resolve) => { resolve(1) })"];
    let fail = vec!["new Promise((resolve) => resolve(1))"];

    let fix = vec![
        (
            "new Promise((resolve) => resolve(1))",
            "new Promise((resolve) => {resolve(1)})",
            None,
            FixKind::Suggestion,
        ),
        (
            "new Promise((resolve) => resolve(1))",
            "new Promise((resolve) => resolve(1))",
            None,
            FixKind::SafeFix,
        ),
        (
            "new Promise(function (resolve) { return resolve(1); })",
            "new Promise(function (resolve) { { resolve(1); return; } })",
            None,
            FixKind::Suggestion,
        ),
        (
            "new Promise(function (resolve) { if (a) return { b }; })",
            "new Promise(function (resolve) { if (a) { ({ b }); return; } })",
            None,
            FixKind::Suggestion,
        ),
        // A function expression is wrapped so the block does not start with a declaration.
        (
            "new Promise(() => function () {})",
            "new Promise(() => {(function () {})})",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoPromiseExecutorReturn::NAME, NoPromiseExecutorReturn::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
