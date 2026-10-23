//! `eslint/logical-assignment-operators`: rewrite to or from the `||=`, `&&=` and `??=` shorthand.
//!
//! - `a = a || b` and `a = a || b || c` become `a ||= b` and `a ||= b || c`.
//! - `a || (a = b)` becomes `a ||= b`, parenthesized where an assignment needs it.
//! - `if (!a) a = b;` becomes `a ||= b;`.
//! - With the `never` option, `a ||= b` becomes `a = a || b`.
//!
//! These are safe fixes only when the target is a local variable. For a global or a property, the
//! shorthand calls the setter (and evaluates the object) a different number of times, so the fix is
//! a suggestion, as it is for a `const` binding: in `never` mode `a = a || b` throws on a `const` even
//! when `a` is truthy, while `a ||= b` does not. The hook cannot see the rule's mode,
//! so the fixer reads the transform from the diagnostic message.
//! No fix when the code contains comments.

use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, Expression, LogicalExpression, Statement},
};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{Binding, assignment_fits_at, binding_of},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    if fcx.ctx.has_comments_between(span) {
        return vec![];
    }
    let text = fcx.text();
    let fix = if text.starts_with("Assignment (=) can be replaced") {
        fix_assignment(fcx, span)
    } else if text.starts_with("Logical expression can be replaced") {
        fix_logical(fcx, span)
    } else if text.starts_with("`if` statement can be replaced") {
        fix_if(fcx, span)
    } else if text.starts_with("Unexpected logical operator assignment") {
        fix_never(fcx, span)
    } else {
        None
    };
    fix.into_iter().collect()
}

/// `a = a || b` → `a ||= b`.
fn fix_assignment(fcx: &FixCtx, span: Span) -> Option<RuleFix> {
    let node = fcx.exact(span, |node| matches!(node.kind(), AstKind::AssignmentExpression(_)))?;
    let AstKind::AssignmentExpression(assignment) = node.kind() else { return None };
    let Expression::LogicalExpression(logical) = assignment.right.without_parentheses() else {
        return None;
    };
    // Find the innermost logical expression of the same operator whose left side is the target.
    let mut inner: &LogicalExpression = logical;
    while let Expression::LogicalExpression(left) = &inner.left
        && left.operator == logical.operator
    {
        inner = left;
    }
    let rest = fcx.source_range(Span::new(inner.right.span().start, logical.span.end));
    let target = fcx.source_range(assignment.left.span());
    let operator = logical.operator.to_assignment_operator().as_str();
    let kind = kind_for_target(fcx, &assignment.left);
    Some(
        fcx.fixer(kind)
            .replace(span, format!("{target} {operator} {rest}"))
            .with_message(format!("Use `{operator}`")),
    )
}

/// `a || (a = b)` → `a ||= b`.
fn fix_logical(fcx: &FixCtx, span: Span) -> Option<RuleFix> {
    let node = fcx.exact(span, |node| matches!(node.kind(), AstKind::LogicalExpression(_)))?;
    let AstKind::LogicalExpression(logical) = node.kind() else { return None };
    let Expression::AssignmentExpression(assignment) = logical.right.get_inner_expression() else {
        return None;
    };
    let target = fcx.source_range(assignment.left.span());
    let value = fcx.source_range(assignment.right.span());
    let operator = logical.operator.to_assignment_operator().as_str();
    let replacement = if assignment_fits_at(fcx, node.id()) {
        format!("{target} {operator} {value}")
    } else {
        format!("({target} {operator} {value})")
    };
    let kind = kind_for_target(fcx, &assignment.left);
    Some(fcx.fixer(kind).replace(span, replacement).with_message(format!("Use `{operator}`")))
}

/// `if (!a) a = b;` → `a ||= b;`.
fn fix_if(fcx: &FixCtx, span: Span) -> Option<RuleFix> {
    let node = fcx.exact(span, |node| matches!(node.kind(), AstKind::IfStatement(_)))?;
    let AstKind::IfStatement(if_statement) = node.kind() else { return None };
    let body = match &if_statement.consequent {
        Statement::BlockStatement(block) => block.body.first()?,
        statement => statement,
    };
    let Statement::ExpressionStatement(statement) = body else { return None };
    let Expression::AssignmentExpression(assignment) = statement.expression.get_inner_expression()
    else {
        return None;
    };
    let operator = fcx.text().split('`').find(|part| matches!(*part, "||=" | "&&=" | "??="))?;
    let target = fcx.source_range(assignment.left.span());
    // The statement now starts with the target. A target starting with one of these characters
    // could join the previous statement if that one relies on automatic semicolon insertion.
    if target.starts_with(crate::fork_fixes::util::ASI_HAZARD_START) {
        return None;
    }
    let value = fcx.source_range(assignment.right.span());
    let kind = kind_for_target(fcx, &assignment.left);
    Some(
        fcx.fixer(kind)
            .replace(span, format!("{target} {operator} {value};"))
            .with_message(format!("Use `{operator}`")),
    )
}

/// `never` mode: `a ||= b` → `a = a || b`.
fn fix_never(fcx: &FixCtx, span: Span) -> Option<RuleFix> {
    let node = fcx.exact(span, |node| matches!(node.kind(), AstKind::AssignmentExpression(_)))?;
    let AstKind::AssignmentExpression(assignment) = node.kind() else { return None };
    let operator = assignment.operator.to_logical_operator()?.as_str();
    let target = fcx.source_range(assignment.left.span());
    let value = fcx.source_range(assignment.right.span());
    let value = if binds_tighter_than_logical(&assignment.right) {
        value.to_string()
    } else {
        format!("({value})")
    };
    let kind = kind_for_target(fcx, &assignment.left);
    Some(
        fcx.fixer(kind)
            .replace(span, format!("{target} = {target} {operator} {value}"))
            .with_message(format!("Use `= {target} {operator}`")),
    )
}

/// Safe only for a local variable that may be reassigned. A suggestion for globals, properties,
/// `const` bindings and imports.
fn kind_for_target(fcx: &FixCtx, target: &AssignmentTarget) -> FixKind {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident)
            if binding_of(fcx, ident) == Binding::Mutable =>
        {
            FixKind::SafeFix
        }
        _ => FixKind::Suggestion,
    }
}

/// Whether `expr` can be the right operand of `||`, `&&` or `??` without parentheses.
fn binds_tighter_than_logical(expr: &Expression) -> bool {
    !matches!(
        expr,
        Expression::LogicalExpression(_)
            | Expression::ConditionalExpression(_)
            | Expression::AssignmentExpression(_)
            | Expression::ArrowFunctionExpression(_)
            | Expression::YieldExpression(_)
            | Expression::SequenceExpression(_)
    )
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{
        rule::RuleMeta, rules::eslint::logical_assignment_operators::LogicalAssignmentOperators,
        tester::Tester,
    };

    let pass = vec!["let a; a ||= b"];
    let fail = vec!["let a; a = a || b"];

    let never = Some(json!(["never"]));
    let ifs = Some(json!(["always", { "enforceForIfStatements": true }]));
    let fix = vec![
        // `a = a || b`
        ("let a; a = a || b", "let a; a ||= b", None, FixKind::SafeFix),
        ("let a; a = a && b", "let a; a &&= b", None, FixKind::SafeFix),
        ("let a; a = a ?? b", "let a; a ??= b", None, FixKind::SafeFix),
        ("let a; a = a || b || c", "let a; a ||= b || c", None, FixKind::SafeFix),
        ("let a; a = (a || b)", "let a; a ||= b", None, FixKind::SafeFix),
        // A global or a property may have a setter: only a suggestion.
        ("a = a || b", "a = a || b", None, FixKind::SafeFix),
        ("a = a || b", "a ||= b", None, FixKind::Suggestion),
        ("o.p = o.p || b", "o.p = o.p || b", None, FixKind::SafeFix),
        ("o.p = o.p || b", "o.p ||= b", None, FixKind::Suggestion),
        // `a || (a = b)`
        ("let a; a || (a = b)", "let a; a ||= b", None, FixKind::SafeFix),
        ("let a; a ?? (a = b)", "let a; a ??= b", None, FixKind::SafeFix),
        ("let a; f(a || (a = b))", "let a; f(a ||= b)", None, FixKind::SafeFix),
        ("let a; x = a && (a = b)", "let a; x = a &&= b", None, FixKind::SafeFix),
        ("let a; (a || (a = b)) ? c : d", "let a; (a ||= b) ? c : d", None, FixKind::SafeFix),
        ("let a; !(a || (a = b))", "let a; !(a ||= b)", None, FixKind::SafeFix),
        // `if`
        ("let a; if (!a) a = b;", "let a; a ||= b;", ifs.clone(), FixKind::SafeFix),
        ("let a; if (!a) { a = b }", "let a; a ||= b;", ifs.clone(), FixKind::SafeFix),
        ("let a; if (a) a = b", "let a; a &&= b;", ifs.clone(), FixKind::SafeFix),
        ("let a; if (a == null) a = b", "let a; a ??= b;", ifs.clone(), FixKind::SafeFix),
        (
            "let a; if (x) {} else if (!a) a = b",
            "let a; if (x) {} else a ||= b;",
            ifs,
            FixKind::SafeFix,
        ),
        // `never`
        ("let a; a ||= b", "let a; a = a || b", never.clone(), FixKind::SafeFix),
        ("let a; a &&= b", "let a; a = a && b", never.clone(), FixKind::SafeFix),
        ("let a; a ??= b || c", "let a; a = a ?? (b || c)", never.clone(), FixKind::SafeFix),
        ("let a; a ||= b ? c : d", "let a; a = a || (b ? c : d)", never.clone(), FixKind::SafeFix),
        ("let a; a ||= b + 1", "let a; a = a || b + 1", never.clone(), FixKind::SafeFix),
        // `a = a || b` throws on a const even when `a` is truthy.
        ("const a = 1; a ||= b", "const a = 1; a ||= b", never.clone(), FixKind::SafeFix),
        ("const a = 1; a ||= b", "const a = 1; a = a || b", never.clone(), FixKind::Suggestion),
        ("o.p ||= b", "o.p = o.p || b", never, FixKind::Suggestion),
        // Comments would be lost.
        ("let a; a = a /* x */ || b", "let a; a = a /* x */ || b", None, FixKind::SafeFix),
    ];

    Tester::new(LogicalAssignmentOperators::NAME, LogicalAssignmentOperators::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
