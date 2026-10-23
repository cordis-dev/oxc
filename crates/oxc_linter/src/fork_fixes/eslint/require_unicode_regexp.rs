//! `eslint/require-unicode-regexp`: suggest adding the `u` (or `v`) flag, as ESLint does.
//!
//! For a regex literal the flag is added (`v` replaces `u` when the `v` flag is required). For
//! `RegExp(..)` with a static pattern, the flag is added to a static flags string, or a flags
//! argument is added. The result is checked with the regex parser, because the `u` and `v` flags
//! make pattern syntax stricter. A suggestion, because the flags also change what the regex matches.

use oxc_ast::{AstKind, ast::Argument};
use oxc_span::GetSpan;

use crate::fork_fixes::util::static_string;
use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{is_valid_regex_literal, regex_source_from_string},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let flag = if fcx.text().contains("'v' flag") { 'v' } else { 'u' };
    let add_flag = |flags: &str| {
        let mut flags: String = flags.chars().filter(|&ch| !(flag == 'v' && ch == 'u')).collect();
        flags.push(flag);
        flags
    };
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| {
        matches!(
            node.kind(),
            AstKind::RegExpLiteral(_) | AstKind::CallExpression(_) | AstKind::NewExpression(_)
        )
    }) else {
        return vec![];
    };
    let fixer = fcx.fixer(FixKind::Suggestion);
    let message = format!("Add the `{flag}` flag");

    let arguments = match node.kind() {
        AstKind::RegExpLiteral(_) => {
            let source = fcx.source_range(span);
            let slash = source.rfind('/').unwrap_or_default();
            let pattern = &source[1..slash];
            let flags = add_flag(&source[slash + 1..]);
            if !is_valid_regex_literal(pattern, &flags) {
                return vec![];
            }
            return vec![fixer.replace(span, format!("/{pattern}/{flags}")).with_message(message)];
        }
        AstKind::CallExpression(call) => &call.arguments,
        AstKind::NewExpression(new) => &new.arguments,
        _ => return vec![],
    };

    let Some(pattern) = arguments.first().and_then(static_string) else { return vec![] };
    let pattern = regex_source_from_string(pattern);
    match arguments.get(1) {
        None => {
            let flags = add_flag("");
            if !is_valid_regex_literal(&pattern, &flags) {
                return vec![];
            }
            let end = arguments[0].span().end;
            vec![
                fixer
                    .insert_text_after_range(oxc_span::Span::empty(end), format!(", '{flags}'"))
                    .with_message(message),
            ]
        }
        Some(argument @ Argument::StringLiteral(literal)) => {
            let Some(value) = literal.value.as_str() else { return vec![] };
            let flags = add_flag(value);
            if !is_valid_regex_literal(&pattern, &flags) {
                return vec![];
            }
            let quote = &fcx.source_range(argument.span())[..1];
            vec![
                fixer.replace(literal.span, format!("{quote}{flags}{quote}")).with_message(message),
            ]
        }
        Some(_) => vec![],
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{
        rule::RuleMeta, rules::eslint::require_unicode_regexp::RequireUnicodeRegexp, tester::Tester,
    };

    let pass = vec![("/foo/u", None)];
    let fail = vec![("/foo/", None)];

    let require_v = Some(json!([{ "requireFlag": "v" }]));
    let fix = vec![
        ("/foo/", "/foo/u", None, FixKind::Suggestion),
        ("/foo/", "/foo/", None, FixKind::SafeFix),
        ("/foo/gi", "/foo/giu", None, FixKind::Suggestion),
        ("new RegExp('foo')", "new RegExp('foo', 'u')", None, FixKind::Suggestion),
        ("RegExp('foo', \"g\")", "RegExp('foo', \"gu\")", None, FixKind::Suggestion),
        ("/foo/u", "/foo/v", require_v.clone(), FixKind::Suggestion),
        ("/foo/", "/foo/v", require_v, FixKind::Suggestion),
        // Invalid with the `u` flag: no suggestion.
        ("/\\a/", "/\\a/", None, FixKind::Suggestion),
        ("new RegExp(pattern, 'g')", "new RegExp(pattern, 'g')", None, FixKind::Suggestion),
    ];

    Tester::new(RequireUnicodeRegexp::NAME, RequireUnicodeRegexp::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
