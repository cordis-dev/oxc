//! `eslint/prefer-regex-literals`: suggest the equivalent regex literal.
//!
//! - `new RegExp('a/b', 'g')` becomes `/a\/b/g`: the string's value becomes the pattern, with `/`
//!   and line terminators escaped.
//! - `new RegExp(/a/)` becomes `/a/`, and `new RegExp(/a/, 'g')` becomes `/a/g` (the new flags
//!   replace the old ones, as at runtime).
//!
//! The result is checked with the regex parser and dropped when invalid. A suggestion, as in ESLint.
//! No suggestion when the call starts a statement after a line that might continue into `/`.

use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{
            is_valid_regex_literal, regex_source_from_string, starts_statement,
            statement_boundary_before, static_string,
        },
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| {
        matches!(node.kind(), AstKind::CallExpression(_) | AstKind::NewExpression(_))
    }) else {
        return vec![];
    };
    let arguments = match node.kind() {
        AstKind::CallExpression(call) => &call.arguments,
        AstKind::NewExpression(new) => &new.arguments,
        _ => return vec![],
    };
    let flags_argument = match arguments.get(1).map(static_string) {
        Some(None) => return vec![],
        flags => flags.flatten(),
    };

    let (pattern, flags) = match arguments
        .first()
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
    {
        Some(Expression::RegExpLiteral(literal)) => {
            let source = fcx.source_range(literal.span);
            let slash = source.rfind('/').unwrap_or_default();
            (source[1..slash].to_string(), flags_argument.unwrap_or(&source[slash + 1..]))
        }
        Some(_) => {
            let Some(value) = arguments.first().and_then(static_string) else { return vec![] };
            (regex_source_from_string(value), flags_argument.unwrap_or_default())
        }
        None => return vec![],
    };
    if !is_valid_regex_literal(&pattern, flags)
        || (starts_statement(fcx, node.id()) && !statement_boundary_before(fcx, span.start))
    {
        return vec![];
    }
    vec![
        fcx.fixer(FixKind::Suggestion)
            .replace(span, format!("/{pattern}/{flags}"))
            .with_message("Use a regular expression literal"),
    ]
}

#[test]
fn test() {
    use serde_json::json;

    use crate::{
        rule::RuleMeta, rules::eslint::prefer_regex_literals::PreferRegexLiterals, tester::Tester,
    };

    let pass = vec![("new RegExp(pattern)", None)];
    let fail = vec![("new RegExp('abc')", None)];

    let redundant = Some(json!([{ "disallowRedundantWrapping": true }]));
    let fix = vec![
        ("new RegExp('abc')", "/abc/", None, FixKind::Suggestion),
        ("new RegExp('abc')", "new RegExp('abc')", None, FixKind::SafeFix),
        ("RegExp('abc', 'gi')", "/abc/gi", None, FixKind::Suggestion),
        ("new RegExp('a/b')", "/a\\/b/", None, FixKind::Suggestion),
        ("new RegExp('\\\\d+')", "/\\d+/", None, FixKind::Suggestion),
        ("new RegExp(`a`)", "/a/", None, FixKind::Suggestion),
        ("new RegExp('')", "/(?:)/", None, FixKind::Suggestion),
        ("x = new RegExp('a').test(y)", "x = /a/.test(y)", None, FixKind::Suggestion),
        ("new RegExp(/a/)", "/a/", redundant.clone(), FixKind::Suggestion),
        ("new RegExp(/a/i, 'g')", "/a/g", redundant, FixKind::Suggestion),
        // Invalid as a literal: no suggestion.
        ("new RegExp('(')", "new RegExp('(')", None, FixKind::Suggestion),
        // `/` could continue the previous line as a division.
        (
            "foo()\nnew RegExp('a').test(b)",
            "foo()\nnew RegExp('a').test(b)",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(PreferRegexLiterals::NAME, PreferRegexLiterals::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
