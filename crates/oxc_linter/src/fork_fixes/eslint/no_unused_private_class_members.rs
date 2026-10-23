//! `eslint/no-unused-private-class-members`: suggest removing the unused member, as ESLint does.
//!
//! A suggestion only: a property's initializer may have side effects. No suggestion when the
//! private name appears anywhere else in the class (for example a property that is only written),
//! because removing the declaration would turn those uses into syntax errors.

use oxc_ast::AstKind;
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_word_char},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(member) = fcx.innermost(span, |node| {
        matches!(
            node.kind(),
            AstKind::PropertyDefinition(_)
                | AstKind::MethodDefinition(_)
                | AstKind::AccessorProperty(_)
        )
    }) else {
        return vec![];
    };
    let Some(class_body) = fcx.innermost(span, |node| matches!(node.kind(), AstKind::ClassBody(_)))
    else {
        return vec![];
    };
    let name = fcx.source_range(span);
    if !name.starts_with('#') {
        return vec![];
    }
    let body = fcx.source_range(class_body.kind().span());
    let uses = body
        .match_indices(name)
        .filter(|(index, _)| !body[index + name.len()..].starts_with(is_word_char))
        .count();
    let member_span = member.kind().span();
    if uses != 1 || fcx.ctx.has_comments_between(member_span) {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::Suggestion)
            .delete_range(member_span)
            .with_message(format!("Remove the unused member `{name}`")),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta,
        rules::eslint::no_unused_private_class_members::NoUnusedPrivateClassMembers,
        tester::Tester,
    };

    let pass = vec!["class C { #a = 1; f() { return this.#a } }"];
    let fail = vec!["class C { #a = 1; }"];

    let fix = vec![
        ("class C { #a = 1; }", "class C {  }", None, FixKind::Suggestion),
        ("class C { #a = 1; }", "class C { #a = 1; }", None, FixKind::SafeFix),
        ("class C { #m() {} f() {} }", "class C {  f() {} }", None, FixKind::Suggestion),
        ("class C { static #a = 1; }", "class C {  }", None, FixKind::Suggestion),
        // Written but never read: removing the declaration would break `this.#a = 2`.
        (
            "class C { #a = 1; f() { this.#a = 2 } }",
            "class C { #a = 1; f() { this.#a = 2 } }",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoUnusedPrivateClassMembers::NAME, NoUnusedPrivateClassMembers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
