//! `eslint/no-dupe-class-members`: suggest removing the member that is overwritten.
//!
//! The later declaration wins, so the earlier one is dead code. A suggestion, because the author may
//! have meant to rename one of them, and a removed field initializer may have had side effects.

use oxc_ast::AstKind;
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(first) = fcx.label_span(0) else { return vec![] };
    let Some(member) = fcx.innermost(first, |node| {
        matches!(
            node.kind(),
            AstKind::MethodDefinition(_)
                | AstKind::PropertyDefinition(_)
                | AstKind::AccessorProperty(_)
        )
    }) else {
        return vec![];
    };
    let span = member.kind().span();
    if fcx.ctx.has_comments_between(span) {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::Suggestion)
            .delete_range(span)
            .with_message("Remove the overwritten member"),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_dupe_class_members::NoDupeClassMembers, tester::Tester,
    };

    let pass = vec!["class A { foo() {} bar() {} }"];
    let fail = vec!["class A { foo() {} foo() {} }"];

    let fix = vec![
        ("class A { foo() {} foo() {} }", "class A {  foo() {} }", None, FixKind::Suggestion),
        ("class A { foo() {} foo() {} }", "class A { foo() {} foo() {} }", None, FixKind::SafeFix),
        ("class A { a = 1; a = 2; }", "class A {  a = 2; }", None, FixKind::Suggestion),
        (
            "class A { static foo() {} static foo() {} }",
            "class A {  static foo() {} }",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoDupeClassMembers::NAME, NoDupeClassMembers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
