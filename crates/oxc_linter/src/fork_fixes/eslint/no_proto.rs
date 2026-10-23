//! `eslint/no-proto`: replace `__proto__` with the `Object` prototype functions.
//!
//! - A read, `o.__proto__`, becomes `Object.getPrototypeOf(o)`.
//! - A statement `o.__proto__ = p;` becomes `Object.setPrototypeOf(o, p);`.
//!
//! A dangerous fix: objects without `Object.prototype` (for example from `Object.create(null)`) have
//! no `__proto__` accessor, and an object may have an own property named `__proto__`. No fix for
//! calls (`this` would change), updates, `delete`, destructuring, optional chains, or when `Object` is
//! shadowed.

use oxc_ast::{AstKind, ast::AssignmentOperator};
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{FixCtx, util::is_unshadowed_global},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| {
        matches!(
            node.kind(),
            AstKind::StaticMemberExpression(_) | AstKind::ComputedMemberExpression(_)
        )
    }) else {
        return vec![];
    };
    if !is_unshadowed_global(fcx, node.id(), "Object") {
        return vec![];
    }
    let (object_span, optional) = match node.kind() {
        AstKind::StaticMemberExpression(member) => (member.object.span(), member.optional),
        AstKind::ComputedMemberExpression(member) => (member.object.span(), member.optional),
        _ => return vec![],
    };
    if optional {
        return vec![];
    }
    let object = fcx.source_range(object_span);
    let fixer = fcx.fixer(FixKind::DangerousFix);
    let nodes = fcx.nodes();
    let parent_id = nodes.parent_id(node.id());

    match nodes.kind(parent_id) {
        AstKind::AssignmentExpression(assignment) if assignment.left.span() == span => {
            let is_statement = matches!(nodes.parent_kind(parent_id), AstKind::ExpressionStatement(stmt) if stmt.expression.span() == assignment.span);
            if assignment.operator != AssignmentOperator::Assign || !is_statement {
                return vec![];
            }
            let value = fcx.source_range(assignment.right.span());
            vec![
                fixer
                    .replace(assignment.span, format!("Object.setPrototypeOf({object}, {value})"))
                    .with_message("Use `Object.setPrototypeOf`"),
            ]
        }
        AstKind::CallExpression(call) if call.callee.span() == span => vec![],
        AstKind::ChainExpression(_)
        | AstKind::UpdateExpression(_)
        | AstKind::UnaryExpression(_) => {
            vec![]
        }
        _ if is_written(fcx, node.id(), span) => vec![],
        _ => vec![
            fixer
                .replace(span, format!("Object.getPrototypeOf({object})"))
                .with_message("Use `Object.getPrototypeOf`"),
        ],
    }
}

/// Whether the member at `node_id` is (part of) an assignment target, including destructuring
/// targets and `for (.. in/of ..)` left sides.
fn is_written(fcx: &FixCtx, node_id: oxc_semantic::NodeId, span: oxc_span::Span) -> bool {
    fcx.nodes().ancestor_kinds(node_id).any(|kind| match kind {
        AstKind::AssignmentExpression(assignment) => {
            assignment.left.span().contains_inclusive(span)
        }
        AstKind::ForInStatement(stmt) => stmt.left.span().contains_inclusive(span),
        AstKind::ForOfStatement(stmt) => stmt.left.span().contains_inclusive(span),
        _ => false,
    })
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_proto::NoProto, tester::Tester};

    let pass = vec!["var a = Object.getPrototypeOf(obj);"];
    let fail = vec!["var a = obj.__proto__;"];

    let fix = vec![
        (
            "var a = obj.__proto__;",
            "var a = Object.getPrototypeOf(obj);",
            None,
            FixKind::DangerousFix,
        ),
        ("var a = obj.__proto__;", "var a = obj.__proto__;", None, FixKind::SafeFix),
        (
            "var a = obj['__proto__'];",
            "var a = Object.getPrototypeOf(obj);",
            None,
            FixKind::DangerousFix,
        ),
        (
            "var a = obj.__proto__.x;",
            "var a = Object.getPrototypeOf(obj).x;",
            None,
            FixKind::DangerousFix,
        ),
        ("obj.__proto__ = b;", "Object.setPrototypeOf(obj, b);", None, FixKind::DangerousFix),
        // Not a plain read or a statement-level assignment.
        ("x = obj.__proto__ = b;", "x = obj.__proto__ = b;", None, FixKind::DangerousFix),
        ("obj.__proto__();", "obj.__proto__();", None, FixKind::DangerousFix),
        ("var a = obj?.__proto__;", "var a = obj?.__proto__;", None, FixKind::DangerousFix),
    ];

    Tester::new(NoProto::NAME, NoProto::PLUGIN, pass, fail).expect_fix(fix).test();
}
