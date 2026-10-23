//! `eslint/prefer-named-capture-group`: suggest naming the capture group, as ESLint does.
//!
//! `(a)` becomes `(?<temp1>a)`. Each unnamed group gets a name from its position among the unnamed
//! groups, so applying several suggestions at once cannot produce duplicate names, and the prefix
//! is chosen so it cannot clash with existing names. Group numbers, and so backreferences, do not
//! change. Regex literals only. A suggestion, because the names are placeholders.

use oxc_ast::AstKind;
use oxc_span::GetSpan;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(literal) =
        fcx.innermost(span, |node| matches!(node.kind(), AstKind::RegExpLiteral(_)))
    else {
        return vec![];
    };
    let literal_span = literal.kind().span();
    let source = fcx.source_text();
    if !source[span.start as usize..].starts_with('(') {
        return vec![];
    }
    let pattern_start = literal_span.start + 1;
    let Some(Ok(pattern_end)) = source[..literal_span.end as usize].rfind('/').map(u32::try_from)
    else {
        return vec![];
    };
    let pattern = &source[pattern_start as usize..pattern_end as usize];

    let unnamed = unnamed_group_offsets(pattern);
    let Some(index) = unnamed.iter().position(|&offset| pattern_start + offset == span.start)
    else {
        return vec![];
    };
    let Some(prefix) = ["temp", "group", "capture", "_group"]
        .into_iter()
        .find(|prefix| !pattern.contains(&format!("(?<{prefix}")))
    else {
        return vec![];
    };
    let name = format!("{prefix}{}", index + 1);
    vec![
        fcx.fixer(FixKind::Suggestion)
            .insert_text_after_range(oxc_span::Span::sized(span.start, 1), format!("?<{name}>"))
            .with_message(format!("Name the group `{name}`")),
    ]
}

/// Offsets of every unnamed capture group's `(` in a regex pattern, skipping escapes and character
/// classes.
fn unnamed_group_offsets(pattern: &str) -> Vec<u32> {
    let bytes = pattern.as_bytes();
    let mut offsets = vec![];
    let mut in_class = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'[' => in_class = true,
            b']' => in_class = false,
            b'(' if !in_class && bytes.get(i + 1) != Some(&b'?') => {
                offsets.extend(u32::try_from(i).ok());
            }
            _ => {}
        }
        i += 1;
    }
    offsets
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::prefer_named_capture_group::PreferNamedCaptureGroup,
        tester::Tester,
    };

    let pass = vec!["/(?<year>[0-9]{4})/"];
    let fail = vec!["/([0-9]{4})/"];

    let fix = vec![
        ("/([0-9]{4})/", "/(?<temp1>[0-9]{4})/", None, FixKind::Suggestion),
        ("/([0-9]{4})/", "/([0-9]{4})/", None, FixKind::SafeFix),
        ("/(a)(?:b)(c)/", "/(?<temp1>a)(?:b)(?<temp2>c)/", None, FixKind::Suggestion),
        ("/[(](a)\\(/", "/[(](?<temp1>a)\\(/", None, FixKind::Suggestion),
        ("/(?<temp1>x)(a)/", "/(?<temp1>x)(?<group1>a)/", None, FixKind::Suggestion),
    ];

    Tester::new(PreferNamedCaptureGroup::NAME, PreferNamedCaptureGroup::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
