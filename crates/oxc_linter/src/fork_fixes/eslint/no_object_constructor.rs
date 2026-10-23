//! `eslint/no-object-constructor`: replace `Object()` and `new Object()` with `{}`.
//!
//! The rule only reports calls to the global `Object` without arguments, which always return a new
//! empty plain object, so this is a safe fix. ESLint offers it as a suggestion only. Where `{` would
//! start a statement or an arrow body, the literal is wrapped as `({})`, and the fix is skipped when
//! the previous line might continue into it.

use oxc_ast::AstKind;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{starts_arrow_expression_body, starts_statement, statement_boundary_before},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| {
        matches!(node.kind(), AstKind::CallExpression(_) | AstKind::NewExpression(_))
    }) else {
        return vec![];
    };
    let at_statement_start = starts_statement(fcx, node.id());
    if at_statement_start && !statement_boundary_before(fcx, span.start) {
        return vec![];
    }
    let replacement = if at_statement_start || starts_arrow_expression_body(fcx, node.id()) {
        "({})"
    } else {
        "{}"
    };
    vec![
        fcx.fixer(FixKind::SafeFix)
            .replace(span, replacement)
            .with_message("Replace with an object literal"),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_object_constructor::NoObjectConstructor, tester::Tester,
    };

    let pass = vec!["new Object(x)"];
    let fail = vec!["new Object()"];

    let fix = vec![
        ("var foo = new Object()", "var foo = {}", None, FixKind::SafeFix),
        ("var foo = new Object", "var foo = {}", None, FixKind::SafeFix),
        ("var foo = Object()", "var foo = {}", None, FixKind::SafeFix),
        ("f(new Object())", "f({})", None, FixKind::SafeFix),
        ("new Object().x", "({}).x", None, FixKind::SafeFix),
        ("new Object();", "({});", None, FixKind::SafeFix),
        ("const f = () => new Object()", "const f = () => ({})", None, FixKind::SafeFix),
        ("const f = () => new Object().x", "const f = () => ({}).x", None, FixKind::SafeFix),
        ("foo();\nnew Object().x", "foo();\n({}).x", None, FixKind::SafeFix),
        // The previous line could continue into `({})`.
        ("foo()\nnew Object().x", "foo()\nnew Object().x", None, FixKind::SafeFix),
        // A `}` that ends an object literal does not end the statement.
        ("x = {}\nObject().y", "x = {}\nObject().y", None, FixKind::SafeFix),
        ("if (a) {}\nObject().y", "if (a) {}\n({}).y", None, FixKind::SafeFix),
    ];

    Tester::new(NoObjectConstructor::NAME, NoObjectConstructor::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
