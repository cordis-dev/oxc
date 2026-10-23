//! `eslint/no-unreachable`: delete an unreachable statement.
//!
//! Unreachable code never runs, but its declarations still take effect: `var` and function
//! declarations are hoisted, and `let`, `const` and `class` shadow outer names from the start of their
//! scope. So the statement is deleted only when it declares nothing anywhere inside it, outside of
//! nested functions. A dangerous fix, because unreachable code is often a bug worth reading. Only for
//! statements directly in a statement list, and not when comments would be lost.

use oxc_ast::AstKind;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{can_remove_statement, descendants},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| node.kind().is_statement()) else { return vec![] };
    if !can_remove_statement(fcx, node.id(), span) {
        return vec![];
    }
    let declares = std::iter::once(node).chain(descendants(fcx, node.id())).any(|node| {
        matches!(
            node.kind(),
            AstKind::VariableDeclaration(_)
                | AstKind::Function(_)
                | AstKind::Class(_)
                | AstKind::TSEnumDeclaration(_)
                | AstKind::TSNamespaceDeclaration(_)
                | AstKind::TSGlobalDeclaration(_)
                | AstKind::TSImportEqualsDeclaration(_)
        )
    });
    if declares {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::DangerousFix)
            .delete_range(span)
            .with_message("Remove the unreachable code"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_unreachable::NoUnreachable, tester::Tester};

    let pass = vec!["function foo() { return x; }"];
    let fail = vec!["function foo() { return x; var x = 1; }"];

    let fix = vec![
        (
            "function foo() { return; x(); }",
            "function foo() { return;  }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "function foo() { return; x(); }",
            "function foo() { return; x(); }",
            None,
            FixKind::SafeFix,
        ),
        ("while (true) { break; x(); }", "while (true) { break;  }", None, FixKind::DangerousFix),
        (
            "function foo() { throw e; if (a) { b(); } }",
            "function foo() { throw e;  }",
            None,
            FixKind::DangerousFix,
        ),
        // Declarations take effect even when unreachable.
        (
            "function foo() { return x; var x = 1; }",
            "function foo() { return x; var x = 1; }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "function foo() { return x; let y = 1; }",
            "function foo() { return x; let y = 1; }",
            None,
            FixKind::DangerousFix,
        ),
        (
            "function foo() { return; if (a) { var b; } }",
            "function foo() { return; if (a) { var b; } }",
            None,
            FixKind::DangerousFix,
        ),
    ];

    Tester::new(NoUnreachable::NAME, NoUnreachable::PLUGIN, pass, fail).expect_fix(fix).test();
}
