//! `eslint/no-async-promise-executor`: remove `async` from an executor that does not use `await`.
//!
//! Without `await`, `async` only changes how errors thrown in the executor behave: they become an
//! unhandled rejection instead of rejecting the new promise. A dangerous fix for that reason. No fix
//! when the executor's source contains `await` anywhere, or a comment follows `async`.

use oxc_span::Span;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::next_char},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    if fcx.source_range(span) != "async" {
        return vec![];
    }
    let Some(function) = fcx.innermost(span, |node| {
        matches!(
            node.kind(),
            oxc_ast::AstKind::Function(_) | oxc_ast::AstKind::ArrowFunctionExpression(_)
        )
    }) else {
        return vec![];
    };
    let function_text = fcx.source_range(oxc_span::GetSpan::span(&function.kind()));
    if function_text.contains("await") {
        return vec![];
    }
    let Some((next, _)) = next_char(fcx, span.end) else { return vec![] };
    let removed = Span::new(span.start, next);
    if fcx.ctx.has_comments_between(removed) {
        return vec![];
    }
    vec![fcx.fixer(FixKind::DangerousFix).delete_range(removed).with_message("Remove `async`")]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_async_promise_executor::NoAsyncPromiseExecutor,
        tester::Tester,
    };

    let pass = vec!["new Promise((resolve, reject) => {})"];
    let fail = vec!["new Promise(async function foo(resolve, reject) {})"];

    let fix = vec![
        (
            "new Promise(async function foo(resolve, reject) {})",
            "new Promise(function foo(resolve, reject) {})",
            None,
            FixKind::DangerousFix,
        ),
        (
            "new Promise(async function foo(resolve, reject) {})",
            "new Promise(async function foo(resolve, reject) {})",
            None,
            FixKind::SafeFix,
        ),
        (
            "new Promise(async (resolve, reject) => {})",
            "new Promise((resolve, reject) => {})",
            None,
            FixKind::DangerousFix,
        ),
        (
            "new Promise(async resolve => resolve(1))",
            "new Promise(resolve => resolve(1))",
            None,
            FixKind::DangerousFix,
        ),
        // The executor uses `await`.
        (
            "new Promise(async (resolve) => { await x; resolve(); })",
            "new Promise(async (resolve) => { await x; resolve(); })",
            None,
            FixKind::DangerousFix,
        ),
    ];

    Tester::new(NoAsyncPromiseExecutor::NAME, NoAsyncPromiseExecutor::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
