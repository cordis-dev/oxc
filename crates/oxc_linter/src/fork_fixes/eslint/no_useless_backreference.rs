//! `eslint/no-useless-backreference`: remove a backreference that always matches the empty string.
//!
//! The rule reports backreferences that can never refer to a captured group, so they always match
//! the empty string and removing them leaves the regex's behavior unchanged. A quantifier on the
//! backreference (`\1*`, `\1{2}`) is removed with it. Only regex literals are fixed, not
//! `RegExp("...")` strings. No fix when the removal would glue neighbors into a different escape,
//! such as `\0` followed by a digit becoming an octal escape.

use oxc_ast::AstKind;
use oxc_span::Span;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    if fcx.innermost(span, |node| matches!(node.kind(), AstKind::RegExpLiteral(_))).is_none() {
        return vec![];
    }
    let source = fcx.source_text();
    let after = &source[span.end as usize..];
    let quantifier_len = quantifier_len(after);
    let Ok(quantifier_len) = u32::try_from(quantifier_len) else { return vec![] };
    let removed = Span::new(span.start, span.end + quantifier_len);

    let before = &source[..span.start as usize];
    let next = &source[removed.end as usize..];
    if next.starts_with(|ch: char| ch.is_ascii_digit()) || ends_with_open_escape(before) {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::SafeFix)
            .delete_range(removed)
            .with_message("Remove the useless backreference"),
    ]
}

/// Length of a quantifier at the start of `text`: `*`, `+`, `?`, `{n}`, `{n,}` or `{n,m}`, each
/// optionally followed by a lazy `?`.
fn quantifier_len(text: &str) -> usize {
    let len = match text.as_bytes().first() {
        Some(b'*' | b'+' | b'?') => 1,
        Some(b'{') => {
            let Some(close) = text.find('}') else { return 0 };
            let inner = &text[1..close];
            let valid = !inner.is_empty()
                && inner.split(',').count() <= 2
                && inner.split(',').next().is_some_and(|min| !min.is_empty())
                && inner.chars().all(|ch| ch.is_ascii_digit() || ch == ',');
            if !valid {
                return 0;
            }
            close + 1
        }
        _ => return 0,
    };
    if text[len..].starts_with('?') { len + 1 } else { len }
}

/// Whether `text` ends inside an escape that the next characters would complete: `\c`, a partial
/// `\x` or `\u` escape, or `\` followed by digits.
fn ends_with_open_escape(text: &str) -> bool {
    let bytes = text.as_bytes();
    let tail_start = bytes.iter().rposition(|&b| b == b'\\');
    let Some(backslash) = tail_start else { return false };
    let escaped = bytes[..backslash].iter().rev().take_while(|&&b| b == b'\\').count() % 2 == 1;
    if escaped {
        return false;
    }
    let tail = &text[backslash + 1..];
    tail == "c"
        || (tail.starts_with('x')
            && tail.len() < 3
            && tail[1..].chars().all(|ch| ch.is_ascii_hexdigit()))
        || (tail.starts_with('u')
            && tail.len() < 5
            && tail[1..].chars().all(|ch| ch.is_ascii_hexdigit()))
        || (!tail.is_empty() && tail.chars().all(|ch| ch.is_ascii_digit()))
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_useless_backreference::NoUselessBackreference,
        tester::Tester,
    };

    let pass = vec!["/(a)\\1/"];
    let fail = vec!["/\\1(a)/"];

    let fix = vec![
        ("/\\1(a)/", "/(a)/", None, FixKind::SafeFix),
        ("/(a|\\1b)/", "/(a|b)/", None, FixKind::SafeFix),
        ("/(a\\1)/", "/(a)/", None, FixKind::SafeFix),
        ("/\\1*(a)/", "/(a)/", None, FixKind::SafeFix),
        ("/\\1{2,3}?(a)/", "/(a)/", None, FixKind::SafeFix),
        ("/\\k<n>(?<n>a)/", "/(?<n>a)/", None, FixKind::SafeFix),
        // A digit after the removal could join the previous escape.
        ("/\\0\\1(a)2/", "/\\0\\1(a)2/", None, FixKind::SafeFix),
        // `RegExp` strings are not fixed.
        ("new RegExp('\\\\1(a)')", "new RegExp('\\\\1(a)')", None, FixKind::SafeFix),
    ];

    Tester::new(NoUselessBackreference::NAME, NoUselessBackreference::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
