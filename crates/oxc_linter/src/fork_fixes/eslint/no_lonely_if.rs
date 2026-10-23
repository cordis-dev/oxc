//! `eslint/no-lonely-if`: turn `else { if (..) .. }` into `else if (..) ..`, as ESLint does.
//!
//! No fix when anything other than whitespace (for example a comment) sits between the braces and
//! the inner `if`, when the inner `if` is wrapped in a further block, or when removing the braces
//! could change how semicolons are inserted.

use oxc_ast::{AstKind, ast::Statement};
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{asi_hazard_after, next_char, starts_with_word},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let label = fcx.span();
    let Some(node) = fcx.innermost(
        label,
        |node| matches!(node.kind(), AstKind::IfStatement(stmt) if stmt.span.start == label.start),
    ) else {
        return vec![];
    };
    let AstKind::IfStatement(lonely_if) = node.kind() else { return vec![] };

    // The lonely `if` must sit directly in the `else` block of its parent `if`.
    let AstKind::BlockStatement(block) = fcx.nodes().parent_kind(node.id()) else { return vec![] };
    let block_id = fcx.nodes().parent_id(node.id());
    let AstKind::IfStatement(parent_if) = fcx.nodes().parent_kind(block_id) else {
        return vec![];
    };
    if parent_if.alternate.as_ref().map(GetSpan::span) != Some(block.span) {
        return vec![];
    }

    let source = fcx.source_text();
    let before = &source[(block.span.start + 1) as usize..lonely_if.span.start as usize];
    let after = &source[lonely_if.span.end as usize..(block.span.end - 1) as usize];
    if !before.trim().is_empty() || !after.trim().is_empty() {
        return vec![];
    }

    // Without the braces, the inner `if` ends the statement. If its last statement relies on
    // automatic semicolon insertion, the following code could join it.
    let (last, has_final_else) = chain_tail(lonely_if);
    let ends_safely = matches!(last, Statement::BlockStatement(_))
        || fcx.source_range(last.span()).trim_end().ends_with(';');
    if !ends_safely && asi_hazard_after(fcx, lonely_if.span.end, block.span.end) {
        return vec![];
    }

    // Without the braces, an `else` right after the block would bind to the inner `if` when its
    // chain ends without one: `if (z) if (a) {} else { if (b) x(); } else y();`. Upstream does not
    // report that shape today, so this only guards against it starting to.
    if !has_final_else
        && next_char(fcx, block.span.end)
            .is_some_and(|(pos, _)| starts_with_word(&source[pos as usize..], "else"))
    {
        return vec![];
    }

    let needs_space = !source[..block.span.start as usize].ends_with(char::is_whitespace);
    let replacement =
        format!("{}{}", if needs_space { " " } else { "" }, fcx.source_range(lonely_if.span));
    vec![
        fcx.fixer(FixKind::SafeFix)
            .replace(block.span, replacement)
            .with_message("Replace the `else` block with `else if`"),
    ]
}

/// The last statement of the `if`/`else if` chain starting at `stmt`, and whether the chain ends
/// with a final `else`.
fn chain_tail<'s, 'a>(mut stmt: &'s oxc_ast::ast::IfStatement<'a>) -> (&'s Statement<'a>, bool) {
    loop {
        match &stmt.alternate {
            Some(Statement::IfStatement(next)) => stmt = next,
            Some(alternate) => return (alternate, true),
            None => return (&stmt.consequent, false),
        }
    }
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_lonely_if::NoLonelyIf, tester::Tester};

    let pass = vec!["if (a) {} else if (b) {}"];
    let fail = vec!["if (a) {} else { if (b) {} }"];

    let fix = vec![
        (
            "if (a) { foo(); } else { if (b) { bar(); } }",
            "if (a) { foo(); } else if (b) { bar(); }",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (a) {\n  foo();\n} else {\n  if (b) {\n    bar();\n  }\n}",
            "if (a) {\n  foo();\n} else if (b) {\n    bar();\n  }",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (a) {} else {\n  if ( /* this comment is ok */\n b) {}\n}",
            "if (a) {} else if ( /* this comment is ok */\n b) {}",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (foo) {} else { if (bar) baz(); }",
            "if (foo) {} else if (bar) baz();",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (foo) { } else { if (bar) baz(); } qux();",
            "if (foo) { } else if (bar) baz(); qux();",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (foo) { } else { if (bar) baz++; } foo;",
            "if (foo) { } else if (bar) baz++; foo;",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (a) {} else { if (b) { bar(); } else if (c) { baz(); } else { qux(); } }",
            "if (a) {} else if (b) { bar(); } else if (c) { baz(); } else { qux(); }",
            None,
            FixKind::SafeFix,
        ),
        ("if (a) {;} else { if (b) {;} }", "if (a) {;} else if (b) {;}", None, FixKind::SafeFix),
        ("if(a){}else{if(b){}}", "if(a){}else if(b){}", None, FixKind::SafeFix),
        (
            "if (a) {} else if (d) {} else { if (b) {} }",
            "if (a) {} else if (d) {} else if (b) {}",
            None,
            FixKind::SafeFix,
        ),
        // A comment would be lost.
        (
            "if (a) {} else { if (b) {} /* keep */ }",
            "if (a) {} else { if (b) {} /* keep */ }",
            None,
            FixKind::SafeFix,
        ),
        // The inner `if` sits in a further block.
        (
            "if (a) {} else { { if (b) {} } }",
            "if (a) {} else { { if (b) {} } }",
            None,
            FixKind::SafeFix,
        ),
        // Automatic semicolon insertion would change the meaning.
        (
            "if (foo) {} else { if (bar) baz() }\n[1, 2, 3].forEach(foo);",
            "if (foo) {} else { if (bar) baz() }\n[1, 2, 3].forEach(foo);",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (foo) {} else { if (bar) baz() } qux()",
            "if (foo) {} else { if (bar) baz() } qux()",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (foo) {} else { if (bar) baz++\n}\nfoo",
            "if (foo) {} else { if (bar) baz++\n}\nfoo",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (a) {} else { if (b) bar()\n}\n`template literal`;",
            "if (a) {} else { if (b) bar()\n}\n`template literal`;",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (a) {} else { if (b) x = {}\n}\n(foo)",
            "if (a) {} else { if (b) x = {}\n}\n(foo)",
            None,
            FixKind::SafeFix,
        ),
        // No ASI hazard: the next statement starts on a new line with an identifier.
        (
            "if (foo) {} else { if (bar) baz()\n}\nqux()",
            "if (foo) {} else if (bar) baz()\nqux()",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoLonelyIf::NAME, NoLonelyIf::PLUGIN, pass, fail).expect_fix(fix).test();
}
