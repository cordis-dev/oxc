//! `eslint/no-useless-catch`: remove a `catch` that only rethrows.
//!
//! `try { a(); } catch (e) { throw e; }` becomes the block `{ a(); }`, and with a `finally` clause
//! only the `catch` clause is removed. Exceptions propagate exactly as before. No fix when the
//! `catch` clause contains comments.

use oxc_ast::AstKind;
use oxc_span::Span;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(node) =
        fcx.innermost(fcx.span(), |node| matches!(node.kind(), AstKind::TryStatement(_)))
    else {
        return vec![];
    };
    let AstKind::TryStatement(try_stmt) = node.kind() else { return vec![] };
    let Some(handler) = &try_stmt.handler else { return vec![] };
    // Everything from the end of the `try` block to the end of the `catch` clause goes.
    let removed = Span::new(try_stmt.block.span.end, handler.span.end);
    if fcx.ctx.has_comments_between(removed) {
        return vec![];
    }

    let fixer = fcx.fixer(FixKind::SafeFix);
    let fix = if try_stmt.finalizer.is_some() {
        fixer.delete_range(removed).with_message("Remove the `catch` clause")
    } else {
        fixer
            .replace(try_stmt.span, fcx.source_range(try_stmt.block.span).to_string())
            .with_message("Remove the `try`/`catch` wrapper")
    };
    vec![fix]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_useless_catch::NoUselessCatch, tester::Tester};

    let pass = vec!["try { foo(); } catch (err) { console.error(err); }"];
    let fail = vec!["try { foo(); } catch (err) { throw err; }"];

    let fix = vec![
        ("try { foo(); } catch (err) { throw err; }", "{ foo(); }", None, FixKind::SafeFix),
        (
            "try { foo(); } catch (err) { throw err; } finally { bar(); }",
            "try { foo(); } finally { bar(); }",
            None,
            FixKind::SafeFix,
        ),
        (
            "if (a) try { foo(); } catch (e) { throw e; }",
            "if (a) { foo(); }",
            None,
            FixKind::SafeFix,
        ),
        (
            "async () => { try { await foo(); } catch (e) { throw e; } }",
            "async () => { { await foo(); } }",
            None,
            FixKind::SafeFix,
        ),
        // A comment would be lost.
        (
            "try { foo(); } catch (err) { /* why */ throw err; }",
            "try { foo(); } catch (err) { /* why */ throw err; }",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoUselessCatch::NAME, NoUselessCatch::PLUGIN, pass, fail).expect_fix(fix).test();
}
