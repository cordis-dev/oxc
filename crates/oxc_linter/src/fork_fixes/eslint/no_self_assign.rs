//! `eslint/no-self-assign`: suggest removing a statement that assigns a variable to itself.
//!
//! `a = a;` does nothing for a plain variable. Only whole statements of that form in a statement
//! list are removed; destructuring and property self-assignments (which may call setters) are not.
//! A suggestion, because the author may have meant a different right-hand side.

use oxc_ast::{
    AstKind,
    ast::{AssignmentOperator, AssignmentTarget},
};
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::can_remove_statement},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.innermost(span, |node| {
        matches!(node.kind(), AstKind::AssignmentExpression(assignment) if assignment.right.span() == span)
    }) else {
        return vec![];
    };
    let AstKind::AssignmentExpression(assignment) = node.kind() else { return vec![] };
    if assignment.operator != AssignmentOperator::Assign
        || !matches!(assignment.left, AssignmentTarget::AssignmentTargetIdentifier(_))
    {
        return vec![];
    }
    let statement_id = fcx.nodes().parent_id(node.id());
    let AstKind::ExpressionStatement(statement) = fcx.nodes().kind(statement_id) else {
        return vec![];
    };
    if !can_remove_statement(fcx, statement_id, statement.span) {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::Suggestion)
            .delete_range(statement.span)
            .with_message("Remove the self-assignment"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_self_assign::NoSelfAssign, tester::Tester};

    let pass = vec!["a = b"];
    let fail = vec!["a = a"];

    let fix = vec![
        ("a = a;", "", None, FixKind::Suggestion),
        ("a = a;", "a = a;", None, FixKind::SafeFix),
        ("f(); a = a; g();", "f();  g();", None, FixKind::Suggestion),
        // Not a statement of its own, or a property that may have a setter.
        ("if (x) a = a;", "if (x) a = a;", None, FixKind::Suggestion),
        ("o.a = o.a;", "o.a = o.a;", None, FixKind::Suggestion),
    ];

    Tester::new(NoSelfAssign::NAME, NoSelfAssign::PLUGIN, pass, fail).expect_fix(fix).test();
}
