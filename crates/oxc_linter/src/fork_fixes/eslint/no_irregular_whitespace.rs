//! `eslint/no-irregular-whitespace`: replace irregular whitespace without changing behavior.
//!
//! - In code and comments, the character becomes a normal space. U+2028 and U+2029 are line
//!   terminators, so they become `\n` to keep automatic semicolon insertion and line comments the
//!   same.
//! - In string, template and regex literals, the character becomes a `\uXXXX` escape with the same
//!   value.
//! - No fix in JSX text and attributes (no escapes there), in tagged templates (the tag sees the raw
//!   text), or for U+180E in code, which is not whitespace.

use std::borrow::Cow;

use oxc_ast::AstKind;
use oxc_span::Span;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let text = fcx.source_range(span);
    let Some(ch) = text.chars().next() else { return vec![] };
    if text.len() != ch.len_utf8() {
        return vec![];
    }

    if fcx.ctx.is_inside_comment(span.start) {
        return vec![
            fcx.fixer(FixKind::SafeFix)
                .replace(span, whitespace_replacement(ch))
                .with_message("Replace the irregular whitespace"),
        ];
    }
    let container = fcx.innermost(span, |node| {
        matches!(
            node.kind(),
            AstKind::StringLiteral(_)
                | AstKind::TemplateLiteral(_)
                | AstKind::RegExpLiteral(_)
                | AstKind::JSXText(_)
                | AstKind::JSXAttribute(_)
        )
    });

    // In a literal the character becomes a `\uXXXX` escape. An identity escape `\<ch>` has the same
    // value as `ch`, so its backslash is replaced too; left in place it would escape the new `\`. An
    // escaped line separator is a line continuation with no value, so it is left alone.
    let escape = || {
        let escape = format!("\\u{:04X}", u32::from(ch));
        let backslashes = fcx.source_text()[..span.start as usize]
            .bytes()
            .rev()
            .take_while(|&b| b == b'\\')
            .count();
        if backslashes % 2 == 0 {
            Some((span, Cow::Owned(escape)))
        } else if matches!(ch, '\u{2028}' | '\u{2029}') {
            None
        } else {
            Some((Span::new(span.start - 1, span.end), Cow::Owned(escape)))
        }
    };
    let replacement = match container.map(|node| (node.id(), node.kind())) {
        Some((_, AstKind::JSXText(_) | AstKind::JSXAttribute(_))) => None,
        // A JSX attribute value (`title="…"`) is a string literal, but escapes are not processed.
        Some((id, AstKind::StringLiteral(_)))
            if matches!(fcx.nodes().parent_kind(id), AstKind::JSXAttribute(_)) =>
        {
            None
        }
        Some((id, AstKind::TemplateLiteral(_)))
            if matches!(fcx.nodes().parent_kind(id), AstKind::TaggedTemplateExpression(_)) =>
        {
            None
        }
        Some((_, AstKind::StringLiteral(_) | AstKind::RegExpLiteral(_))) => escape(),
        // Only the literal's contents, not code inside `${..}`.
        Some((_, AstKind::TemplateLiteral(template)))
            if template.quasis.iter().any(|quasi| quasi.span.contains_inclusive(span)) =>
        {
            escape()
        }
        _ if ch == '\u{180e}' => None,
        _ => Some((span, whitespace_replacement(ch))),
    };
    let Some((span, replacement)) = replacement else { return vec![] };
    vec![
        fcx.fixer(FixKind::SafeFix)
            .replace(span, replacement)
            .with_message("Replace the irregular whitespace"),
    ]
}

fn whitespace_replacement(ch: char) -> Cow<'static, str> {
    Cow::Borrowed(if matches!(ch, '\u{2028}' | '\u{2029}') { "\n" } else { " " })
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{
        rule::RuleMeta, rules::eslint::no_irregular_whitespace::NoIrregularWhitespace,
        tester::Tester,
    };

    let pass = vec!["var a = 1;"];
    let fail = vec!["var\u{a0}a = 1;"];

    let in_strings = Some(
        json!([{ "skipStrings": false, "skipTemplates": false, "skipRegExps": false, "skipComments": false }]),
    );
    let fix = vec![
        ("var\u{a0}a = 1;", "var a = 1;", None, FixKind::SafeFix),
        ("var a =\u{3000}1;", "var a = 1;", None, FixKind::SafeFix),
        ("var a = 1;\u{feff}", "var a = 1; ", None, FixKind::SafeFix),
        ("var a = 1\u{2028}var b = 2", "var a = 1\nvar b = 2", None, FixKind::SafeFix),
        ("var a = '\u{a0}';", "var a = '\\u00A0';", in_strings.clone(), FixKind::SafeFix),
        ("var a = `x\u{a0}`;", "var a = `x\\u00A0`;", in_strings.clone(), FixKind::SafeFix),
        ("var a = /\u{a0}/;", "var a = /\\u00A0/;", in_strings.clone(), FixKind::SafeFix),
        ("// a\u{a0}b", "// a b", in_strings.clone(), FixKind::SafeFix),
        // An identity escape keeps its value: the backslash is replaced too.
        ("var a = '\\\u{a0}';", "var a = '\\u00A0';", in_strings.clone(), FixKind::SafeFix),
        ("var a = /a\\\u{a0}b/;", "var a = /a\\u00A0b/;", in_strings.clone(), FixKind::SafeFix),
        // An escaped backslash does not escape the character.
        ("var a = '\\\\\u{a0}';", "var a = '\\\\\\u00A0';", in_strings.clone(), FixKind::SafeFix),
        // The tag sees the raw text.
        ("var a = tag`x\u{a0}`;", "var a = tag`x\u{a0}`;", in_strings.clone(), FixKind::SafeFix),
        // JSX attribute values do not process escapes.
        ("<a title=\"x\u{a0}y\" />", "<a title=\"x\u{a0}y\" />", in_strings, FixKind::SafeFix),
    ];

    Tester::new(NoIrregularWhitespace::NAME, NoIrregularWhitespace::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
