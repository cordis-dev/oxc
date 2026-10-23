//! `eslint/no-multi-assign`: split a chained assignment into separate statements.
//!
//! - `a = b = c;` becomes `b = c; a = b;`.
//! - `var a = b = c;` becomes `b = c; var a = b;`.
//!
//! Longer chains are split one link per pass. Only plain `=` with identifier targets, as a statement
//! in a statement list. A dangerous fix: reading `b` back runs a getter if `b` is a global accessor,
//! and a `const` `b` now throws before `a` is assigned.

use oxc_ast::{
    AstKind,
    ast::{AssignmentExpression, AssignmentOperator, AssignmentTarget, BindingPattern},
};
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_statement_list},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(inner) =
        fcx.exact(span, |node| matches!(node.kind(), AstKind::AssignmentExpression(_)))
    else {
        return vec![];
    };
    let AstKind::AssignmentExpression(inner_assignment) = inner.kind() else { return vec![] };
    let Some(inner_target) = simple_target(inner_assignment) else { return vec![] };
    let inner_text = fcx.source_range(inner_assignment.span);
    let nodes = fcx.nodes();
    let parent_id = nodes.parent_id(inner.id());

    let (statement_span, replacement) = match nodes.kind(parent_id) {
        AstKind::AssignmentExpression(outer) => {
            let Some(outer_target) = simple_target(outer) else { return vec![] };
            let statement_id = nodes.parent_id(parent_id);
            let AstKind::ExpressionStatement(statement) = nodes.kind(statement_id) else {
                return vec![];
            };
            if statement.expression.span() != outer.span
                || !is_statement_list(&nodes.parent_kind(statement_id))
            {
                return vec![];
            }
            (statement.span, format!("{inner_text}; {outer_target} = {inner_target};"))
        }
        AstKind::VariableDeclarator(declarator) => {
            let BindingPattern::BindingIdentifier(ident) = &declarator.id else { return vec![] };
            let declaration_id = nodes.parent_id(parent_id);
            let AstKind::VariableDeclaration(declaration) = nodes.kind(declaration_id) else {
                return vec![];
            };
            if declaration.declarations.len() != 1
                || declarator.type_annotation.is_some()
                || !is_statement_list(&nodes.parent_kind(declaration_id))
            {
                return vec![];
            }
            let kind = declaration.kind.as_str();
            (declaration.span, format!("{inner_text}; {kind} {} = {inner_target};", ident.name))
        }
        _ => return vec![],
    };
    if fcx.ctx.has_comments_between(statement_span) {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::DangerousFix)
            .replace(statement_span, replacement)
            .with_message("Split the chained assignment"),
    ]
}

/// The identifier name of a plain `name = value` assignment.
fn simple_target<'a>(assignment: &'a AssignmentExpression<'a>) -> Option<&'a str> {
    match &assignment.left {
        AssignmentTarget::AssignmentTargetIdentifier(ident)
            if assignment.operator == AssignmentOperator::Assign =>
        {
            Some(ident.name.as_str())
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_multi_assign::NoMultiAssign, tester::Tester};

    let pass = vec!["var a = 1; var b = a;"];
    let fail = vec!["var a = b = c;"];

    let fix = vec![
        ("var a = b = c;", "b = c; var a = b;", None, FixKind::DangerousFix),
        ("var a = b = c;", "var a = b = c;", None, FixKind::SafeFix),
        ("let a = b = c", "b = c; let a = b;", None, FixKind::DangerousFix),
        ("a = b = c;", "b = c; a = b;", None, FixKind::DangerousFix),
        // Property targets and non-statement positions are left alone.
        ("a.x = b = c;", "a.x = b = c;", None, FixKind::DangerousFix),
        ("f(a = b = c);", "f(a = b = c);", None, FixKind::DangerousFix),
    ];

    Tester::new(NoMultiAssign::NAME, NoMultiAssign::PLUGIN, pass, fail).expect_fix(fix).test();
}
