//! `eslint/init-declarations`: add or move the initializer.
//!
//! - `always`: `let x;` becomes `let x = undefined;`, which is the same value, so this is a safe fix
//!   when `undefined` is the global. Not for `var`, which inside a loop keeps its value between
//!   iterations where `= undefined` would reset it, and not with a type annotation, where
//!   `= undefined` is a type error under `strictNullChecks`.
//! - `never`: `let x = 1;` becomes `let x; x = 1;`. A suggestion, because `let x = x` (a TDZ error)
//!   would stop throwing. Only for a single declarator in a statement list.
//!
//! The mode is read from the diagnostic message.

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, VariableDeclarationKind},
};
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{is_statement_list, is_unshadowed_global},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::VariableDeclarator(_)))
    else {
        return vec![];
    };
    let AstKind::VariableDeclarator(declarator) = node.kind() else { return vec![] };
    let declaration_id = fcx.nodes().parent_id(node.id());
    let AstKind::VariableDeclaration(declaration) = fcx.nodes().kind(declaration_id) else {
        return vec![];
    };
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else { return vec![] };

    if fcx.text().ends_with("should be initialized on declaration.") {
        // With a type annotation, `= undefined` is a type error under `strictNullChecks`.
        if declaration.kind != VariableDeclarationKind::Let
            || declaration.declare
            || declarator.type_annotation.is_some()
            || !is_unshadowed_global(fcx, node.id(), "undefined")
        {
            return vec![];
        }
        return vec![
            fcx.fixer(FixKind::SafeFix)
                .insert_text_after_range(Span::empty(span.end), " = undefined")
                .with_message("Initialize with `undefined`"),
        ];
    }

    // `never`: split the initializer into an assignment. Only `var` and `let` can be declared
    // without a value and assigned later; `using` declarations need their initializer.
    let Some(init) = &declarator.init else { return vec![] };
    if !matches!(declaration.kind, VariableDeclarationKind::Var | VariableDeclarationKind::Let)
        || declaration.declarations.len() != 1
        || !is_statement_list(&fcx.nodes().parent_kind(declaration_id))
        || fcx.ctx.has_comments_between(declaration.span)
    {
        return vec![];
    }
    let head = fcx.source_range(Span::new(declaration.span.start, declarator.id.span().end));
    let type_annotation = declarator
        .type_annotation
        .as_ref()
        .map_or("", |annotation| fcx.source_range(annotation.span));
    let name = ident.name.as_str();
    let value = fcx.source_range(init.span());
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(declaration.span, format!("{head}{type_annotation}; {name} = {value};"))
            .with_message("Move the initializer to an assignment"),
    ]
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{
        rule::RuleMeta, rules::eslint::init_declarations::InitDeclarations, tester::Tester,
    };

    let pass = vec![("let a = 1;", None)];
    let fail = vec![("let a;", None)];

    let never = Some(json!(["never"]));
    let fix = vec![
        ("let a;", "let a = undefined;", None, FixKind::SafeFix),
        ("function f() { let a; }", "function f() { let a = undefined; }", None, FixKind::SafeFix),
        // `var` would be reset in each loop iteration.
        ("var a;", "var a;", None, FixKind::SafeFix),
        // `= undefined` would be a type error for an annotated variable under `strictNullChecks`.
        ("let a: number;", "let a: number;", None, FixKind::SafeFix),
        // `undefined` is shadowed.
        (
            "function f(undefined) { let a; }",
            "function f(undefined) { let a; }",
            None,
            FixKind::SafeFix,
        ),
        ("let a = 1;", "let a; a = 1;", never.clone(), FixKind::Suggestion),
        ("let a = 1;", "let a = 1;", never.clone(), FixKind::SafeFix),
        ("var a = f();", "var a; a = f();", never.clone(), FixKind::Suggestion),
        ("let a = 1, b = 2;", "let a = 1, b = 2;", never.clone(), FixKind::Suggestion),
        // `using` needs its initializer.
        ("{ using res = open(); }", "{ using res = open(); }", never, FixKind::Suggestion),
    ];

    Tester::new(InitDeclarations::NAME, InitDeclarations::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
