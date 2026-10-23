//! `eslint/no-return-assign`: suggestions for an assignment in a `return`.
//!
//! - `except-parens` mode (the default): wrap the assignment in parentheses to mark it as intended,
//!   `return (a = b)`.
//! - `always` mode: compute first, then return. `return a = b;` becomes `{ a = b; return a; }` and
//!   `() => a = b` becomes `() => { a = b; return a; }`. Only for a local variable, where reading it
//!   back gives the assigned value.
//!
//! Suggestions, because the assignment may be a typo for `===`. Only when the returned expression is
//! itself the assignment. The mode is read from the diagnostic's help text.

use oxc_ast::{
    AstKind,
    ast::{AssignmentTarget, Expression},
};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{Binding, binding_of},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    if fcx.ctx.has_comments_between(span) {
        return vec![];
    }
    let Some(node) = fcx.exact(span, |node| {
        matches!(node.kind(), AstKind::ReturnStatement(_) | AstKind::ArrowFunctionExpression(_))
    }) else {
        return vec![];
    };
    let (value, is_arrow) = match node.kind() {
        AstKind::ReturnStatement(statement) => (statement.argument.as_ref(), false),
        AstKind::ArrowFunctionExpression(arrow) => (arrow.get_expression(), true),
        _ => return vec![],
    };
    let Some(Expression::AssignmentExpression(assignment)) = value else { return vec![] };
    let assignment_text = fcx.source_range(assignment.span);
    let fixer = fcx.fixer(FixKind::Suggestion);

    if fcx.help().is_some_and(|help| help.contains("wrap the assignment in parentheses")) {
        return vec![
            fixer
                .replace(assignment.span, format!("({assignment_text})"))
                .with_message("Wrap the assignment in parentheses"),
        ];
    }

    let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment.left else {
        return vec![];
    };
    if binding_of(fcx, ident) != Binding::Mutable {
        return vec![];
    }
    let name = ident.name.as_str();
    let block = format!("{{ {assignment_text}; return {name}; }}");
    let replaced = if is_arrow { assignment.span } else { span };
    vec![fixer.replace(replaced, block).with_message("Assign before returning")]
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{rule::RuleMeta, rules::eslint::no_return_assign::NoReturnAssign, tester::Tester};

    let pass = vec![("function f() { return (a = b); }", None)];
    let fail = vec![("function f() { return a = b; }", None)];

    let always = Some(json!(["always"]));
    let fix = vec![
        (
            "function f() { return a = b; }",
            "function f() { return (a = b); }",
            None,
            FixKind::Suggestion,
        ),
        (
            "function f() { return a = b; }",
            "function f() { return a = b; }",
            None,
            FixKind::SafeFix,
        ),
        ("const f = () => a = b;", "const f = () => (a = b);", None, FixKind::Suggestion),
        (
            "function f() { let a; return a = b; }",
            "function f() { let a; { a = b; return a; } }",
            always.clone(),
            FixKind::Suggestion,
        ),
        (
            "function f() { let a; return a += 1; }",
            "function f() { let a; { a += 1; return a; } }",
            always.clone(),
            FixKind::Suggestion,
        ),
        (
            "let a; const f = () => a = b;",
            "let a; const f = () => { a = b; return a; };",
            always.clone(),
            FixKind::Suggestion,
        ),
        // A property may have a getter that returns something else.
        (
            "function f() { return o.a = b; }",
            "function f() { return o.a = b; }",
            always,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(NoReturnAssign::NAME, NoReturnAssign::PLUGIN, pass, fail).expect_fix(fix).test();
}
