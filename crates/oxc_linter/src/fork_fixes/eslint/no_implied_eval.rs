//! `eslint/no-implied-eval`: suggest passing a function instead of a code string.
//!
//! `setTimeout("a()", 10)` becomes `setTimeout(() => { a() }, 10)`. A suggestion, because the string
//! was evaluated in the global scope while the function sees the local scope. Only for a string
//! literal (or substitution-free template) whose contents parse as a script.

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use crate::fork_fixes::util::static_string;
use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    if !fcx.text().starts_with("Implied eval. Consider passing a function") {
        return vec![];
    }
    let Some(node) =
        fcx.exact(fcx.span(), |node| matches!(node.kind(), AstKind::CallExpression(_)))
    else {
        return vec![];
    };
    let AstKind::CallExpression(call) = node.kind() else { return vec![] };
    let Some(argument) = call.arguments.first() else { return vec![] };
    let Some(code) = static_string(argument) else { return vec![] };
    let allocator = Allocator::default();
    if Parser::new(&allocator, code, SourceType::cjs()).parse().diagnostics.has_errors() {
        return vec![];
    }
    // The line break before `}` keeps a trailing line comment in the code from swallowing it.
    let replacement = if code.contains("//") {
        format!("() => {{ {code}\n}}")
    } else {
        format!("() => {{ {code} }}")
    };
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(argument.span(), replacement)
            .with_message("Pass a function instead of a string"),
    ]
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{rule::RuleMeta, rules::eslint::no_implied_eval::NoImpliedEval, tester::Tester};

    let browser = || Some(json!({ "env": { "browser": true } }));
    let pass = vec![("setTimeout(function() { x = 1; }, 100);", None, browser(), None)];
    let fail = vec![("setTimeout('x = 1;', 100);", None, browser(), None)];

    // Cases with an ESLint config apply every fix kind, which includes suggestions.
    let fix = vec![
        ("setTimeout('x = 1;', 100);", "setTimeout(() => { x = 1; }, 100);", None, browser()),
        ("setInterval(\"foo()\");", "setInterval(() => { foo() });", None, browser()),
        ("setTimeout('a() // run', 1);", "setTimeout(() => { a() // run\n}, 1);", None, browser()),
        // The string does not parse.
        ("setTimeout('x = ', 100);", "setTimeout('x = ', 100);", None, browser()),
    ];

    Tester::new(NoImpliedEval::NAME, NoImpliedEval::PLUGIN, pass, fail).expect_fix(fix).test();
}
