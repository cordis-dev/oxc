//! `eslint/no-useless-assignment`: suggest dropping the value that is never read.
//!
//! - `x = value;` as a statement is removed when `value` has no side effects, and otherwise becomes
//!   `value;` so its side effects stay.
//! - `let x = value;` loses its initializer when `value` has no side effects.
//!
//! A suggestion, because the assignment may point at a bug (a later read of the wrong variable).
//! Compound assignments, updates and destructuring are left alone.

use oxc_ast::{
    AstKind,
    ast::{AssignmentOperator, AssignmentTarget},
};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{
            as_statement, can_delete_statement, is_side_effect_free, is_statement_list,
            safe_statement_start,
        },
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let fixer = fcx.fixer(FixKind::Suggestion);
    let Some(node) = fcx.innermost(span, |node| {
        matches!(node.kind(), AstKind::AssignmentExpression(_) | AstKind::VariableDeclarator(_))
    }) else {
        return vec![];
    };

    match node.kind() {
        AstKind::VariableDeclarator(declarator) if declarator.id.span() == span => {
            let Some(init) = &declarator.init else { return vec![] };
            if !is_side_effect_free(init) {
                return vec![];
            }
            let start = declarator
                .type_annotation
                .as_ref()
                .map_or(declarator.id.span().end, |t| t.span.end);
            let removed = Span::new(start, init.span().end);
            if fcx.ctx.has_comments_between(removed) {
                return vec![];
            }
            vec![fixer.delete_range(removed).with_message("Remove the unused initial value")]
        }
        AstKind::AssignmentExpression(assignment)
            if (assignment.left.span() == span || assignment.span == span)
                && assignment.operator == AssignmentOperator::Assign
                && matches!(assignment.left, AssignmentTarget::AssignmentTargetIdentifier(_)) =>
        {
            let statement_id = fcx.nodes().parent_id(node.id());
            let AstKind::ExpressionStatement(statement) = fcx.nodes().kind(statement_id) else {
                return vec![];
            };
            if !is_statement_list(&fcx.nodes().parent_kind(statement_id))
                || fcx.ctx.has_comments_between(statement.span)
            {
                return vec![];
            }
            if is_side_effect_free(&assignment.right) {
                if !can_delete_statement(fcx, statement.span) {
                    return vec![];
                }
                return vec![
                    fixer.delete_range(statement.span).with_message("Remove the unused assignment"),
                ];
            }
            let value = as_statement(fcx.source_range(assignment.right.span())).into_owned();
            if !safe_statement_start(fcx, assignment.span.start, &value) {
                return vec![];
            }
            vec![
                fixer
                    .replace(assignment.span, value)
                    .with_message("Keep only the assigned expression"),
            ]
        }
        _ => vec![],
    }
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_useless_assignment::NoUselessAssignment, tester::Tester,
    };

    let pass = vec!["function f() { let v = 'used'; console.log(v); }"];
    let fail = vec!["let v = 'used'; console.log(v); v = 'unused'"];

    let fix = vec![
        (
            "function f() { let v = 'used'; console.log(v); v = 'unused'; }",
            "function f() { let v = 'used'; console.log(v);  }",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() { let v = 'used'; console.log(v); v = g(); }",
            "function f() { let v = 'used'; console.log(v); g(); }",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() { let v = 'unused'; v = 'used'; console.log(v); }",
            "function f() { let v; v = 'used'; console.log(v); }",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() { let v = 'unused'; v = 'used'; console.log(v); }",
            "function f() { let v = 'unused'; v = 'used'; console.log(v); }",
            None,
            FixKind::SafeFix,
        ),
        // `[f()]` at the start of the line would continue `foo()`.
        (
            "function g() { let v; foo()\nv = [f()]; v = 2; return v; }",
            "function g() { let v; foo()\nv = [f()]; v = 2; return v; }",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
