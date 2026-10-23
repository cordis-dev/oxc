//! `eslint/symbol-description`: suggest describing the symbol with its variable's name.
//!
//! `const foo = Symbol()` becomes `const foo = Symbol('foo')`. Only when the symbol initializes a
//! variable, which gives a sensible description. A suggestion, because the description is visible
//! through `description` and `toString()`.

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, Expression},
};
use oxc_span::Span;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::CallExpression(_)))
    else {
        return vec![];
    };
    let AstKind::VariableDeclarator(declarator) = fcx.nodes().parent_kind(node.id()) else {
        return vec![];
    };
    let (BindingPattern::BindingIdentifier(ident), Some(Expression::CallExpression(init))) =
        (&declarator.id, &declarator.init)
    else {
        return vec![];
    };
    if init.span != span || !init.arguments.is_empty() || !fcx.source_range(span).ends_with("()") {
        return vec![];
    }
    let name = ident.name.as_str();
    vec![
        fcx.fixer(FixKind::Suggestion)
            .insert_text_after_range(Span::empty(span.end - 1), format!("'{name}'"))
            .with_message(format!("Describe the symbol as '{name}'")),
    ]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::symbol_description::SymbolDescription, tester::Tester,
    };

    let pass = vec!["Symbol('foo');"];
    let fail = vec!["Symbol();"];

    let fix = vec![
        ("const foo = Symbol();", "const foo = Symbol('foo');", None, FixKind::Suggestion),
        ("const foo = Symbol();", "const foo = Symbol();", None, FixKind::SafeFix),
        ("Symbol();", "Symbol();", None, FixKind::Suggestion),
    ];

    Tester::new(SymbolDescription::NAME, SymbolDescription::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
