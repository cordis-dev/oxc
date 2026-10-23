//! `eslint/no-debugger`: upgrade upstream's suggestion to a safe fix.
//!
//! Upstream turned its fix into a suggestion so that editors do not remove `debugger` on save while
//! someone is debugging. Removing it never changes what the program does, so `--fix` may apply it.
//! In a statement list the statement is deleted, unless automatic semicolon insertion would then
//! join the code around it. As the body of `if`, a loop or a label it becomes `{}`, which also
//! covers `a: debugger;`, where upstream's deletion leaves a label without a statement.

use oxc_ast::AstKind;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{can_remove_statement, is_statement_list},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::DebuggerStatement(_)))
    else {
        return vec![];
    };
    let fixer = fcx.fixer(FixKind::SafeFix);
    let fix = if !is_statement_list(&fcx.nodes().parent_kind(node.id())) {
        fixer.replace(span, "{}")
    } else if can_remove_statement(fcx, node.id(), span) {
        fixer.delete_range(span)
    } else {
        return vec![];
    };
    vec![fix.with_message("Remove the debugger statement")]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_debugger::NoDebugger, tester::Tester};

    let pass = vec!["var test = { debugger: 1 }; test.debugger;"];
    let fail = vec!["debugger;"];

    let fix = vec![
        ("debugger;", "", None, FixKind::SafeFix),
        ("let x; debugger; let y;", "let x;  let y;", None, FixKind::SafeFix),
        (
            "switch (a) {\n  case 1:\n    debugger;\n}",
            "switch (a) {\n  case 1:\n    \n}",
            None,
            FixKind::SafeFix,
        ),
        ("if (foo) debugger", "if (foo) {}", None, FixKind::SafeFix),
        ("if (foo) debugger; else bar()", "if (foo) {} else bar()", None, FixKind::SafeFix),
        ("do debugger; while (x)", "do {} while (x)", None, FixKind::SafeFix),
        ("a: debugger;", "a: {}", None, FixKind::SafeFix),
        // Deleting it would join `a` and `(b)` into the call `a(b)`: only upstream's suggestion.
        ("a\ndebugger\n(b)", "a\ndebugger\n(b)", None, FixKind::SafeFix),
    ];

    Tester::new(NoDebugger::NAME, NoDebugger::PLUGIN, pass, fail).expect_fix(fix).test();
}
