//! `eslint/no-const-assign`: suggest declaring the variable with `let`.
//!
//! The reassignment throws at runtime. Changing `const` to `let` makes it work, which may or may not
//! be what the author meant, so this is a suggestion. Every declarator in the same declaration
//! becomes `let`. No suggestion for `using` declarations.

use oxc_ast::{AstKind, ast::VariableDeclarationKind};
use oxc_span::Span;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(declared) = fcx.label_span(0) else { return vec![] };
    let Some(node) =
        fcx.innermost(declared, |node| matches!(node.kind(), AstKind::VariableDeclaration(_)))
    else {
        return vec![];
    };
    let AstKind::VariableDeclaration(declaration) = node.kind() else { return vec![] };
    if declaration.kind != VariableDeclarationKind::Const
        || !fcx.source_range(declaration.span).starts_with("const")
    {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(Span::sized(declaration.span.start, 5), "let")
            .with_message("Declare it with `let`"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_const_assign::NoConstAssign, tester::Tester};

    let pass = vec!["let x = 0; x = 1;"];
    let fail = vec!["const x = 0; x = 1;"];

    let fix = vec![
        ("const x = 0; x = 1;", "let x = 0; x = 1;", None, FixKind::Suggestion),
        ("const x = 0; x = 1;", "const x = 0; x = 1;", None, FixKind::SafeFix),
        ("const { a } = o; a = 1;", "let { a } = o; a = 1;", None, FixKind::Suggestion),
        ("for (const x of y) { x = 1; }", "for (let x of y) { x = 1; }", None, FixKind::Suggestion),
    ];

    Tester::new(NoConstAssign::NAME, NoConstAssign::PLUGIN, pass, fail).expect_fix(fix).test();
}
