//! `eslint/no-fallthrough`: two suggestions for a `case` that falls through into the next one.
//!
//! - Insert `break;` at the end of the falling-through case. A dangerous suggestion: it changes what
//!   runs.
//! - Insert a `/* falls through */` comment, which marks the fallthrough as intended and keeps the
//!   behavior. A block comment, so it cannot comment out code on the same line.

use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::statement_boundary_before},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.innermost(span, |node| matches!(node.kind(), AstKind::SwitchStatement(_)))
    else {
        return vec![];
    };
    let AstKind::SwitchStatement(switch) = node.kind() else { return vec![] };
    let Some(index) = switch.cases.iter().position(|case| case.span == span) else { return vec![] };
    let Some(previous) = index.checked_sub(1).map(|index| &switch.cases[index]) else {
        return vec![];
    };
    let end =
        previous.consequent.last().map_or(previous.span.end, |statement| statement.span().end);
    let at = Span::empty(end);
    // A last statement that relies on automatic semicolon insertion (`foo()`) would run into
    // `break` on the same line, so it gets an explicit `;` first.
    let brk = if statement_boundary_before(fcx, end) { " break;" } else { "; break;" };
    vec![
        fcx.fixer(FixKind::DangerousSuggestion)
            .insert_text_after_range(at, brk)
            .with_message("Add `break;`"),
        fcx.fixer(FixKind::Suggestion)
            .insert_text_after_range(at, " /* falls through */")
            .with_message("Mark the fallthrough as intended"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_fallthrough::NoFallthrough, tester::Tester};

    let pass = vec!["switch (foo) { case 0: a(); break; case 1: b(); }"];
    let fail = vec!["switch (foo) { case 0: a(); case 1: b(); }"];

    let fix = vec![
        (
            "switch (foo) { case 0: a(); case 1: b(); }",
            (
                "switch (foo) { case 0: a(); break; case 1: b(); }",
                "switch (foo) { case 0: a(); /* falls through */ case 1: b(); }",
            ),
        ),
        (
            "switch (foo) { case 0: a(); default: b(); }",
            (
                "switch (foo) { case 0: a(); break; default: b(); }",
                "switch (foo) { case 0: a(); /* falls through */ default: b(); }",
            ),
        ),
        // Without a semicolon, `break` would join the call on the same line.
        (
            "switch (foo) { case 0:\n  a()\ncase 1: b(); }",
            (
                "switch (foo) { case 0:\n  a(); break;\ncase 1: b(); }",
                "switch (foo) { case 0:\n  a() /* falls through */\ncase 1: b(); }",
            ),
        ),
        // A `}` that ends a statement needs no semicolon.
        (
            "switch (foo) { case 0: if (a) {} case 1: b(); }",
            (
                "switch (foo) { case 0: if (a) {} break; case 1: b(); }",
                "switch (foo) { case 0: if (a) {} /* falls through */ case 1: b(); }",
            ),
        ),
    ];
    let not_safe = vec![(
        "switch (foo) { case 0: a(); case 1: b(); }",
        "switch (foo) { case 0: a(); case 1: b(); }",
        None,
        FixKind::SafeFix,
    )];

    Tester::new(NoFallthrough::NAME, NoFallthrough::PLUGIN, pass.clone(), fail.clone())
        .expect_fix(fix)
        .test();
    Tester::new(NoFallthrough::NAME, NoFallthrough::PLUGIN, pass, fail).expect_fix(not_safe).test();
}
