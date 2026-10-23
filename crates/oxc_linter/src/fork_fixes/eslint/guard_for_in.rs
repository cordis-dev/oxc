//! `eslint/guard-for-in`: suggest guarding the loop body with an own-property check.
//!
//! `for (k in o) body` becomes
//! `for (k in o) { if (Object.prototype.hasOwnProperty.call(o, k)) body }`; the braces keep a
//! following `else` with its own `if`.
//! A suggestion, because it skips inherited properties the loop used to visit. Only when the key is
//! a plain identifier and the object is an identifier, `this` or a member chain of those, which can
//! be evaluated again without side effects (getters aside). No suggestion when `Object` is shadowed.

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression, ForStatementLeft},
};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_unshadowed_global},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(node) =
        fcx.innermost(fcx.span(), |node| matches!(node.kind(), AstKind::ForInStatement(_)))
    else {
        return vec![];
    };
    let AstKind::ForInStatement(for_in) = node.kind() else { return vec![] };
    if !is_unshadowed_global(fcx, node.id(), "Object") || !is_reevaluable(&for_in.right) {
        return vec![];
    }
    let key = match &for_in.left {
        ForStatementLeft::VariableDeclaration(declaration) => {
            match declaration.declarations.first().map(|declarator| &declarator.id) {
                Some(BindingPattern::BindingIdentifier(ident))
                    if declaration.declarations.len() == 1 =>
                {
                    ident.name.as_str()
                }
                _ => return vec![],
            }
        }
        ForStatementLeft::AssignmentTargetIdentifier(ident) => ident.name.as_str(),
        _ => return vec![],
    };
    let object = fcx.source_range(for_in.right.span());
    let guard = format!("if (Object.prototype.hasOwnProperty.call({object}, {key})) ");
    let body = for_in.body.span();
    // Braces around the new `if` keep an `else` after the loop (`if (c) for (..) f(); else g();`)
    // from binding to it.
    let fixer = fcx.fixer(FixKind::Suggestion).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    fix.push(Fix::new(format!("{{ {guard}"), Span::empty(body.start)));
    fix.push(Fix::new(" }", Span::empty(body.end)));
    vec![fix.with_message("Skip inherited properties")]
}

/// An identifier, `this`, or a non-computed member chain of those.
fn is_reevaluable(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(_) | Expression::ThisExpression(_) => true,
        Expression::StaticMemberExpression(member) => is_reevaluable(&member.object),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::guard_for_in::GuardForIn, tester::Tester};

    let pass = vec!["for (const k in o) { if (Object.hasOwn(o, k)) f(k) }"];
    let fail = vec!["for (const k in o) { f(k) }"];

    let fix = vec![
        (
            "for (const k in o) { f(k) }",
            "for (const k in o) { if (Object.prototype.hasOwnProperty.call(o, k)) { f(k) } }",
            None,
            FixKind::Suggestion,
        ),
        ("for (const k in o) { f(k) }", "for (const k in o) { f(k) }", None, FixKind::SafeFix),
        (
            "for (k in this.o) f(k);",
            "for (k in this.o) { if (Object.prototype.hasOwnProperty.call(this.o, k)) f(k); }",
            None,
            FixKind::Suggestion,
        ),
        // The braces keep the outer `else` with the outer `if`.
        (
            "if (c) for (k in o) f(); else g();",
            "if (c) for (k in o) { if (Object.prototype.hasOwnProperty.call(o, k)) f(); } else g();",
            None,
            FixKind::Suggestion,
        ),
        (
            "if (c) for (k in o) { f() } else g();",
            "if (c) for (k in o) { if (Object.prototype.hasOwnProperty.call(o, k)) { f() } } else g();",
            None,
            FixKind::Suggestion,
        ),
        // The object would be evaluated again.
        (
            "for (const k in get()) { f(k) }",
            "for (const k in get()) { f(k) }",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(GuardForIn::NAME, GuardForIn::PLUGIN, pass, fail).expect_fix(fix).test();
}
