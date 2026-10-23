//! `eslint/default-case-last`: move the `default` clause to the end of the `switch`.
//!
//! Where the `default` clause sits does not change which clause is selected, only what falls through
//! into what. So the clause is moved only when nothing falls through into it, out of it, or out of
//! the last clause into it after the move: the clause before it, the `default` clause itself and the
//! last clause must all end with `break`, `return`, `throw` or `continue`. A dangerous fix, because it
//! reorders code. No fix when comments would move.

use oxc_ast::{
    AstKind,
    ast::{Statement, SwitchCase},
};
use oxc_span::Span;

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.innermost(span, |node| matches!(node.kind(), AstKind::SwitchStatement(_)))
    else {
        return vec![];
    };
    let AstKind::SwitchStatement(switch) = node.kind() else { return vec![] };
    // The label covers the `default` keyword.
    let Some(index) = switch.cases.iter().position(|case| case.span.start == span.start) else {
        return vec![];
    };
    let default = &switch.cases[index];
    let Some(last) = switch.cases.last() else { return vec![] };
    if index + 1 == switch.cases.len()
        || !terminates(default)
        || !terminates(last)
        || (index > 0 && !terminates(&switch.cases[index - 1]))
    {
        return vec![];
    }
    let next = &switch.cases[index + 1];
    // From the start of the `default` clause to the start of the next clause.
    let moved = Span::new(default.span.start, next.span.start);
    if fcx.ctx.has_comments_between(moved) {
        return vec![];
    }
    let separator = &fcx.source_text()[default.span.end as usize..next.span.start as usize];
    let default_text = fcx.source_range(default.span);

    let fixer = fcx.fixer(FixKind::DangerousFix).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    fix.push(Fix::delete(moved));
    fix.push(Fix::new(format!("{separator}{default_text}"), Span::empty(last.span.end)));
    vec![fix.with_message("Move the `default` clause to the end")]
}

/// Whether control never falls out of the end of `case`.
fn terminates(case: &SwitchCase) -> bool {
    fn ends(statement: &Statement) -> bool {
        match statement {
            Statement::BreakStatement(_)
            | Statement::ReturnStatement(_)
            | Statement::ThrowStatement(_)
            | Statement::ContinueStatement(_) => true,
            Statement::BlockStatement(block) => block.body.last().is_some_and(ends),
            _ => false,
        }
    }
    case.consequent.last().is_some_and(ends)
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::default_case_last::DefaultCaseLast, tester::Tester,
    };

    let pass = vec!["switch (foo) { case 1: bar(); break; default: baz(); }"];
    let fail = vec!["switch (foo) { default: bar(); break; case 1: baz(); break; }"];

    let fix = vec![
        (
            "switch (foo) { default: bar(); break; case 1: baz(); break; }",
            "switch (foo) { case 1: baz(); break; default: bar(); break; }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "switch (foo) { default: bar(); break; case 1: baz(); break; }",
            "switch (foo) { default: bar(); break; case 1: baz(); break; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "switch (foo) { case 0: a(); break; default: b(); break; case 1: c(); break; }",
            "switch (foo) { case 0: a(); break; case 1: c(); break; default: b(); break; }",
            None,
            FixKind::DangerousFix,
        ),
        // Code falls through into or out of the `default` clause.
        (
            "switch (foo) { default: bar(); case 1: baz(); break; }",
            "switch (foo) { default: bar(); case 1: baz(); break; }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "switch (foo) { case 0: a(); default: b(); break; case 1: c(); break; }",
            "switch (foo) { case 0: a(); default: b(); break; case 1: c(); break; }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "switch (foo) { default: b(); break; case 1: c(); }",
            "switch (foo) { default: b(); break; case 1: c(); }",
            None,
            FixKind::DangerousFix,
        ),
    ];

    Tester::new(DefaultCaseLast::NAME, DefaultCaseLast::PLUGIN, pass, fail).expect_fix(fix).test();
}
