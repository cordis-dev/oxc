//! `eslint/no-extra-bind`: remove the unnecessary `.bind(x)` call, as ESLint does.
//!
//! No fix when the argument may have side effects (`b()`, `b++`, `b.c`) or when a comment would be
//! removed.

use oxc_ast::{AstKind, ast::Expression};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{is_side_effect_free, skip_trivia},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    // The diagnostic labels the `bind` property. The innermost call around it is the `.bind(..)` call.
    let label = fcx.span();
    let Some(node) = fcx.innermost(label, |node| matches!(node.kind(), AstKind::CallExpression(_)))
    else {
        return vec![];
    };
    let AstKind::CallExpression(call) = node.kind() else { return vec![] };

    let member = match call.callee.get_inner_expression() {
        Expression::ChainExpression(chain) => chain.expression.as_member_expression(),
        callee => callee.as_member_expression(),
    };
    let Some(member) = member else { return vec![] };
    if !member.span().contains_inclusive(label) {
        return vec![];
    }
    // A spread argument (`as_expression` is `None`) runs an iterator.
    if call.arguments.len() != 1
        || !call.arguments[0].as_expression().is_some_and(is_side_effect_free)
    {
        return vec![];
    }

    // Remove the accessor (`.bind`, `?.bind`, `['bind']`) but keep parentheses around the function,
    // then remove the argument list.
    let accessor = Span::new(skip_trivia(fcx, member.object().span().end, true), member.span().end);
    let arguments = Span::new(call.callee.span().end, call.span.end);
    if fcx.ctx.has_comments_between(accessor) || fcx.ctx.has_comments_between(arguments) {
        return vec![];
    }

    let fixer = fcx.fixer(FixKind::SafeFix).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    if accessor.end == arguments.start {
        fix.push(Fix::delete(Span::new(accessor.start, arguments.end)));
    } else {
        fix.push(Fix::delete(accessor));
        fix.push(Fix::delete(arguments));
    }
    vec![fix.with_message("Remove the unnecessary `.bind` call")]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_extra_bind::NoExtraBind, tester::Tester};

    let pass = vec!["var a = function() { return this.b }.bind(c)"];
    let fail = vec!["var a = function() { return 1; }.bind(b)"];

    // From the fix cases upstream left commented out in `rules/eslint/no_extra_bind.rs`.
    let fix = vec![
        (
            "var a = function() { return 1; }.bind(b)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { return 1; }['bind'](b)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { return 1; }[`bind`](b)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = (() => { return 1; }).bind(b)",
            "var a = (() => { return 1; })",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = (() => { return this; }).bind(b)",
            "var a = (() => { return this; })",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { (function(){ this.c }) }.bind(b)",
            "var a = function() { (function(){ this.c }) }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { function c(){ this.d } }.bind(b)",
            "var a = function() { function c(){ this.d } }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { return 1; }.bind(this)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { (function(){ (function(){ this.d }.bind(c)) }) }.bind(b)",
            "var a = function() { (function(){ (function(){ this.d }.bind(c)) }) }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = (function() { return 1; }).bind(this)",
            "var a = (function() { return 1; })",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = (function() { return 1; }.bind)(this)",
            "var a = (function() { return 1; })",
            None,
            FixKind::SafeFix,
        ),
        ("var a = function() {}/**/.bind(b)", "var a = function() {}/**/", None, FixKind::SafeFix),
        (
            "var a = function() {}/**/['bind'](b)",
            "var a = function() {}/**/",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}//comment\n\t\t.bind(b)",
            "var a = function() {}//comment\n\t\t",
            None,
            FixKind::SafeFix,
        ),
        ("var a = function() {}.bind(b)/**/", "var a = function() {}/**/", None, FixKind::SafeFix),
        (
            "var a = function() { return 1; }.bind?.(b)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { return 1; }?.bind(b)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = (function() { return 1; }?.bind)(b)",
            "var a = (function() { return 1; })",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { return 1; }['bind']?.(b)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() { return 1; }?.['bind'](b)",
            "var a = function() { return 1; }",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = (function() { return 1; }?.['bind'])(b)",
            "var a = (function() { return 1; })",
            None,
            FixKind::SafeFix,
        ),
        // The argument may have side effects: reported, not fixed.
        (
            "var a = function() {}.bind(b++)",
            "var a = function() {}.bind(b++)",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}.bind(b())",
            "var a = function() {}.bind(b())",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}.bind(b.c)",
            "var a = function() {}.bind(b.c)",
            None,
            FixKind::SafeFix,
        ),
        // A comment would be removed: reported, not fixed.
        (
            "var a = function() {}./**/bind(b)",
            "var a = function() {}./**/bind(b)",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}[/**/'bind'](b)",
            "var a = function() {}[/**/'bind'](b)",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}.//\n\t\t\tbind(b)",
            "var a = function() {}.//\n\t\t\tbind(b)",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}.bind/**/(b)",
            "var a = function() {}.bind/**/(b)",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}.bind(\n\t\t\t/**/b)",
            "var a = function() {}.bind(\n\t\t\t/**/b)",
            None,
            FixKind::SafeFix,
        ),
        (
            "var a = function() {}.bind(b/**/)",
            "var a = function() {}.bind(b/**/)",
            None,
            FixKind::SafeFix,
        ),
        // Fixes are not applied when only suggestions were requested.
        (
            "var a = function() { return 1; }.bind(b)",
            "var a = function() { return 1; }.bind(b)",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoExtraBind::NAME, NoExtraBind::PLUGIN, pass, fail).expect_fix(fix).test();
}
