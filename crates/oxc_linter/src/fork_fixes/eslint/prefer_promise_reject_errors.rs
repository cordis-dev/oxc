//! `eslint/prefer-promise-reject-errors`: suggest rejecting with an `Error`.
//!
//! `reject(x)` becomes `reject(new Error(x))`, and `reject()` or `reject(undefined)` becomes
//! `reject(new Error())`. A suggestion, because code that handles the rejection may rely on the
//! original value. No suggestion for spread arguments or when `Error` is shadowed.

use oxc_ast::{AstKind, ast::Argument};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_unshadowed_global},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::CallExpression(_)))
    else {
        return vec![];
    };
    let AstKind::CallExpression(call) = node.kind() else { return vec![] };
    if !is_unshadowed_global(fcx, node.id(), "Error") {
        return vec![];
    }
    let fixer = fcx.fixer(FixKind::Suggestion);
    let message = "Reject with an `Error`";
    match call.arguments.first() {
        None => {
            let close = span.end - 1;
            vec![
                fixer
                    .insert_text_after_range(Span::empty(close), "new Error()")
                    .with_message(message),
            ]
        }
        Some(Argument::SpreadElement(_)) => vec![],
        Some(argument) => {
            let is_undefined =
                argument.as_expression().is_some_and(|e| e.is_undefined() || e.is_void());
            let replacement = if is_undefined {
                "new Error()".to_string()
            } else {
                format!("new Error({})", fcx.source_range(argument.span()))
            };
            vec![fixer.replace(argument.span(), replacement).with_message(message)]
        }
    }
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::prefer_promise_reject_errors::PreferPromiseRejectErrors,
        tester::Tester,
    };

    let pass = vec!["Promise.reject(new Error('a'))"];
    let fail = vec!["Promise.reject('a')"];

    let fix = vec![
        ("Promise.reject('a')", "Promise.reject(new Error('a'))", None, FixKind::Suggestion),
        ("Promise.reject('a')", "Promise.reject('a')", None, FixKind::SafeFix),
        ("Promise.reject()", "Promise.reject(new Error())", None, FixKind::Suggestion),
        ("Promise.reject(undefined)", "Promise.reject(new Error())", None, FixKind::Suggestion),
        (
            "new Promise((resolve, reject) => reject(5))",
            "new Promise((resolve, reject) => reject(new Error(5)))",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f(Error) { Promise.reject('a') }",
            "function f(Error) { Promise.reject('a') }",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(PreferPromiseRejectErrors::NAME, PreferPromiseRejectErrors::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
