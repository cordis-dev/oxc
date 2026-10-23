//! `eslint/grouped-accessor-pairs`: group a getter and setter for the same key.
//!
//! - "should be grouped": the later accessor moves to right after the earlier one.
//! - "Expected .. to be before ..": two adjacent accessors swap places.
//!
//! The property keeps its position (it is created by the first accessor) and both accessors are the
//! same functions, so the object behaves the same. A dangerous fix anyway, because it reorders
//! members and their evaluation. No fix when any member involved or in between has a computed key
//! (its evaluation would move), when a member in between has the same name, or when comments would
//! move.

use oxc_ast::{
    AstKind,
    ast::{ClassElement, ObjectPropertyKind, PropertyKey},
};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::FixCtx,
};

/// The static name of a member key, if it has one.
fn member_name(key: &PropertyKey) -> Option<String> {
    key.static_name()?.as_js_str().as_str().map(ToString::to_string)
}

/// One member of an object literal or class body.
struct Member {
    span: Span,
    computed: bool,
    name: Option<String>,
}

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let labels = fcx.label_spans();
    let [first_label, second_label] = labels.as_slice() else { return vec![] };
    let Some(container) = fcx.innermost(*first_label, |node| {
        let span = node.kind().span();
        matches!(node.kind(), AstKind::ObjectExpression(_) | AstKind::ClassBody(_))
            && span.contains_inclusive(*second_label)
    }) else {
        return vec![];
    };
    let (members, is_object) = match container.kind() {
        AstKind::ObjectExpression(object) => (
            object
                .properties
                .iter()
                .map(|prop| match prop {
                    ObjectPropertyKind::ObjectProperty(p) => {
                        Member { span: p.span, computed: p.computed, name: member_name(&p.key) }
                    }
                    ObjectPropertyKind::SpreadProperty(p) => {
                        Member { span: p.span, computed: true, name: None }
                    }
                })
                .collect::<Vec<_>>(),
            true,
        ),
        AstKind::ClassBody(body) => (
            body.body
                .iter()
                .map(|element| match element {
                    ClassElement::MethodDefinition(m) => {
                        Member { span: m.span, computed: m.computed, name: member_name(&m.key) }
                    }
                    ClassElement::PropertyDefinition(p) => {
                        Member { span: p.span, computed: p.computed, name: member_name(&p.key) }
                    }
                    other => Member { span: other.span(), computed: true, name: None },
                })
                .collect::<Vec<_>>(),
            false,
        ),
        _ => return vec![],
    };
    let index_of =
        |label: &Span| members.iter().position(|member| member.span.contains_inclusive(*label));
    let (Some(a), Some(b)) = (index_of(first_label), index_of(second_label)) else { return vec![] };
    let (earlier, later) = if a < b { (a, b) } else { (b, a) };
    let name = &members[earlier].name;
    if name.is_none()
        || members[earlier..=later].iter().any(|member| member.computed)
        || members[earlier + 1..later].iter().any(|member| &member.name == name)
    {
        return vec![];
    }
    let source = fcx.source_text();
    let text = |member: &Member| &source[member.span.start as usize..member.span.end as usize];

    let fixer = fcx.fixer(FixKind::DangerousFix).for_multifix();
    if fcx.text().ends_with("should be grouped.") {
        if later == earlier + 1 {
            return vec![];
        }
        // Remove the later member with the separator before it, and insert it after the earlier one.
        let removed = Span::new(members[later - 1].span.end, members[later].span.end);
        if fcx.ctx.has_comments_between(removed) {
            return vec![];
        }
        let separator = if is_object { ", " } else { " " };
        let mut fix = fixer.new_fix_with_capacity(2);
        fix.push(Fix::new(
            format!("{separator}{}", text(&members[later])),
            Span::empty(members[earlier].span.end),
        ));
        fix.push(Fix::delete(removed));
        return vec![fix.with_message("Group the accessor pair")];
    }

    // Order: swap two adjacent accessors.
    if later != earlier + 1 {
        return vec![];
    }
    let whole = Span::new(members[earlier].span.start, members[later].span.end);
    if fcx.ctx.has_comments_between(whole) {
        return vec![];
    }
    let between = &source[members[earlier].span.end as usize..members[later].span.start as usize];
    let swapped = format!("{}{between}{}", text(&members[later]), text(&members[earlier]));
    vec![fixer.replace(whole, swapped).with_message("Swap the accessors")]
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{
        rule::RuleMeta, rules::eslint::grouped_accessor_pairs::GroupedAccessorPairs, tester::Tester,
    };

    let pass = vec![("({ get a() {}, set a(v) {} })", None)];
    let fail = vec![("({ get a() {}, b: 1, set a(v) {} })", None)];

    let get_before_set = Some(json!(["getBeforeSet"]));
    let fix = vec![
        (
            "({ get a() {}, b: 1, set a(v) {} })",
            "({ get a() {}, set a(v) {}, b: 1 })",
            None,
            FixKind::DangerousFix,
        ),
        (
            "({ get a() {}, b: 1, set a(v) {} })",
            "({ get a() {}, b: 1, set a(v) {} })",
            None,
            FixKind::SafeFix,
        ),
        (
            "class A { get a() {} b() {} set a(v) {} }",
            "class A { get a() {} set a(v) {} b() {} }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "({ set a(v) {}, get a() {} })",
            "({ get a() {}, set a(v) {} })",
            get_before_set,
            FixKind::DangerousFix,
        ),
        // A member in between has the same name, or a key is computed.
        (
            "({ get a() {}, a: 1, set a(v) {} })",
            "({ get a() {}, a: 1, set a(v) {} })",
            None,
            FixKind::DangerousFix,
        ),
        (
            "({ get a() {}, [b]: 1, set a(v) {} })",
            "({ get a() {}, [b]: 1, set a(v) {} })",
            None,
            FixKind::DangerousFix,
        ),
    ];

    Tester::new(GroupedAccessorPairs::NAME, GroupedAccessorPairs::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
