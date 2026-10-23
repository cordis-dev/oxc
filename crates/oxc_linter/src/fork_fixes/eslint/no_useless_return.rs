//! `eslint/no-useless-return`: delete the redundant `return;`, as ESLint does.
//!
//! Only when the `return` sits directly in a statement list (a block, function body, static block or
//! `case`), because elsewhere, as in `if (a) return;`, deleting it would leave invalid code. No fix
//! when the statement contains a comment.

use oxc_ast::AstKind;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::can_delete_statement},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::ReturnStatement(_)))
    else {
        return vec![];
    };
    let in_statement_list = matches!(
        fcx.nodes().parent_kind(node.id()),
        AstKind::BlockStatement(_)
            | AstKind::FunctionBody(_)
            | AstKind::StaticBlock(_)
            | AstKind::SwitchCase(_)
    );
    if !in_statement_list || fcx.ctx.has_comments_between(span) || !can_delete_statement(fcx, span)
    {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::SafeFix)
            .delete_range(span)
            .with_message("Remove the redundant `return`"),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_useless_return::NoUselessReturn, tester::Tester,
    };

    let pass = vec!["function foo() { return 5; }"];
    let fail = vec!["function foo() { return; }"];

    let fix = vec![
        ("function foo() { return; }", "function foo() {  }", None, FixKind::SafeFix),
        (
            "function foo() { doSomething(); return; }",
            "function foo() { doSomething();  }",
            None,
            FixKind::SafeFix,
        ),
        (
            "function foo() { if (condition) { bar(); return; } else { baz(); } }",
            "function foo() { if (condition) { bar();  } else { baz(); } }",
            None,
            FixKind::SafeFix,
        ),
        (
            "function foo() { switch (bar) { case 1: doSomething(); default: doSomethingElse(); return; } }",
            "function foo() { switch (bar) { case 1: doSomething(); default: doSomethingElse();  } }",
            None,
            FixKind::SafeFix,
        ),
        (
            "function foo() { try {} catch (err) { return; } }",
            "function foo() { try {} catch (err) {  } }",
            None,
            FixKind::SafeFix,
        ),
        ("const foo = () => { return; }", "const foo = () => {  }", None, FixKind::SafeFix),
        // Deleting it would leave `if (foo)` without a body.
        (
            "function foo() { if (foo) return; }",
            "function foo() { if (foo) return; }",
            None,
            FixKind::SafeFix,
        ),
        // A comment would be lost.
        (
            "function foo() { return /* why */; }",
            "function foo() { return /* why */; }",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoUselessReturn::NAME, NoUselessReturn::PLUGIN, pass, fail).expect_fix(fix).test();
}
