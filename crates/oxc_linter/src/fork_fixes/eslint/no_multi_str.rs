//! `eslint/no-multi-str`: turn a string with line continuations into a concatenation.
//!
//! A line continuation (`\` followed by a line break) adds nothing to the string's value, so
//! `'a \<newline>b'` becomes `'a ' +<newline>'b'` with the same value. Leading whitespace of the next
//! line stays inside the second literal because it is part of the value. The concatenation is
//! wrapped in parentheses where `+` would bind differently. No fix where a concatenation is not
//! allowed, such as import sources, property keys and directives.

use oxc_ast::AstKind;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{concatenation_fits_at, in_expression_slot},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let Some(node) =
        fcx.innermost(fcx.span(), |node| matches!(node.kind(), AstKind::StringLiteral(_)))
    else {
        return vec![];
    };
    let AstKind::StringLiteral(literal) = node.kind() else { return vec![] };
    if !in_expression_slot(fcx, node.id()) {
        return vec![];
    }

    let raw = fcx.source_range(literal.span);
    let quote = &raw[..1];
    let inner = &raw[1..raw.len() - 1];

    // Split the contents at each line continuation, keeping the line break that followed it.
    let mut parts: Vec<(&str, &str)> = vec![];
    let mut part_start = 0;
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < inner.len() {
        if bytes[i] == b'\\' {
            let rest = &inner[i + 1..];
            let line_break = ["\r\n", "\n", "\r", "\u{2028}", "\u{2029}"]
                .into_iter()
                .find(|line_break| rest.starts_with(line_break));
            if let Some(line_break) = line_break {
                parts.push((&inner[part_start..i], line_break));
                i += 1 + line_break.len();
                part_start = i;
                continue;
            }
            // Skip the escaped character.
            i += 1 + rest.chars().next().map_or(0, char::len_utf8);
            continue;
        }
        i += inner[i..].chars().next().map_or(1, char::len_utf8);
    }
    if parts.is_empty() {
        return vec![];
    }

    let mut out = String::with_capacity(raw.len() + parts.len() * 4);
    for (part, line_break) in &parts {
        out.push_str(quote);
        out.push_str(part);
        out.push_str(quote);
        out.push_str(" +");
        out.push_str(line_break);
    }
    out.push_str(quote);
    out.push_str(&inner[part_start..]);
    out.push_str(quote);
    if !concatenation_fits_at(fcx, node.id()) {
        out = format!("({out})");
    }
    vec![
        fcx.fixer(FixKind::SafeFix)
            .replace(literal.span, out)
            .with_message("Replace the line continuation with a concatenation"),
    ]
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, rules::eslint::no_multi_str::NoMultiStr, tester::Tester};

    let pass = vec!["var a = 'Line 1 Line 2';"];
    let fail = vec!["var x = 'Line 1 \\\n Line 2'"];

    let fix = vec![
        ("var x = 'Line 1 \\\n Line 2'", "var x = 'Line 1 ' +\n' Line 2'", None, FixKind::SafeFix),
        ("var x = \"a\\\nb\\\nc\"", "var x = \"a\" +\n\"b\" +\n\"c\"", None, FixKind::SafeFix),
        ("var x = 'a\\\r\nb'", "var x = 'a' +\r\n'b'", None, FixKind::SafeFix),
        ("f('a\\\nb')", "f('a' +\n'b')", None, FixKind::SafeFix),
        ("x + 'a\\\nb'", "x + 'a' +\n'b'", None, FixKind::SafeFix),
        // `+` binds differently here, so the concatenation gets parentheses.
        ("var n = 'a\\\nb'.length", "var n = ('a' +\n'b').length", None, FixKind::SafeFix),
        ("var n = 'a\\\nb' * 2", "var n = ('a' +\n'b') * 2", None, FixKind::SafeFix),
        // An escaped backslash before the line break is not a continuation; the next one is.
        ("var x = 'a\\\\\\\nb'", "var x = 'a\\\\' +\n'b'", None, FixKind::SafeFix),
        // A concatenation is not allowed here.
        ("import x from 'a\\\nb'", "import x from 'a\\\nb'", None, FixKind::SafeFix),
        ("var o = { 'a\\\nb': 1 }", "var o = { 'a\\\nb': 1 }", None, FixKind::SafeFix),
    ];

    Tester::new(NoMultiStr::NAME, NoMultiStr::PLUGIN, pass, fail).expect_fix(fix).test();
}
