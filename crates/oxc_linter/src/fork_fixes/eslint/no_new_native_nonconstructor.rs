//! `eslint/no-new-native-nonconstructor`: suggest calling `Symbol` or `BigInt` without `new`.
//!
//! `new Symbol('a')` always throws a `TypeError`; `Symbol('a')` creates the symbol. A suggestion,
//! because the code goes from throwing to working. `new Symbol` without parentheses becomes
//! `Symbol()`. No suggestion when a comment sits between `new` and the callee.

use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let label = fcx.span();
    let Some(node) = fcx.innermost(
        label,
        |node| matches!(node.kind(), AstKind::NewExpression(new) if new.span.start == label.start),
    ) else {
        return vec![];
    };
    let AstKind::NewExpression(new) = node.kind() else { return vec![] };
    let keyword = Span::new(new.span.start, new.callee.span().start);
    if fcx.ctx.has_comments_between(keyword) {
        return vec![];
    }
    let fixer = fcx.fixer(FixKind::Suggestion).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    fix.push(Fix::delete(keyword));
    if new.span.end == new.callee.span().end {
        fix.push(Fix::new("()", Span::empty(new.span.end)));
    }
    vec![fix.with_message("Call it without `new`")]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_new_native_nonconstructor::NoNewNativeNonconstructor,
        tester::Tester,
    };

    let pass = vec!["var foo = Symbol('foo');"];
    let fail = vec!["var foo = new Symbol('foo');"];

    let fix = vec![
        ("var foo = new Symbol('foo');", "var foo = Symbol('foo');", None, FixKind::Suggestion),
        ("var foo = new Symbol('foo');", "var foo = new Symbol('foo');", None, FixKind::SafeFix),
        (
            "var foo = new BigInt(9007199254740991);",
            "var foo = BigInt(9007199254740991);",
            None,
            FixKind::Suggestion,
        ),
        ("var foo = new Symbol;", "var foo = Symbol();", None, FixKind::Suggestion),
    ];

    Tester::new(NoNewNativeNonconstructor::NAME, NoNewNativeNonconstructor::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
