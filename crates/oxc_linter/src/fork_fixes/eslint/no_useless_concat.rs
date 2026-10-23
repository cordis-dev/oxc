//! `eslint/no-useless-concat`: merge two adjacent literals into one.
//!
//! `'a' + 'b'` becomes `'ab'`, keeping the left literal's quote style and the original escapes.
//! When either side is a template literal, the result is a template literal. Concatenating strings
//! is associative, so `x + 'a' + 'b'` can become `x + 'ab'`. No fix when anything other than
//! whitespace and `+` sits between the literals, such as parentheses or comments.

use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{escape_unescaped, has_legacy_escape},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let is_literal =
        |kind: &AstKind| matches!(kind, AstKind::StringLiteral(_) | AstKind::TemplateLiteral(_));
    let Some(left) = fcx.innermost(Span::sized(span.start, 1), |node| {
        is_literal(&node.kind()) && node.kind().span().start == span.start
    }) else {
        return vec![];
    };
    let Some(right) = fcx.innermost(Span::new(span.end - 1, span.end), |node| {
        is_literal(&node.kind()) && node.kind().span().end == span.end
    }) else {
        return vec![];
    };
    let (left_span, right_span) = (left.kind().span(), right.kind().span());
    if fcx.source_range(Span::new(left_span.end, right_span.start)).trim() != "+" {
        return vec![];
    }

    let left_text = fcx.source_range(left_span);
    let right_text = fcx.source_range(right_span);
    let left_quote = left_text.chars().next().unwrap_or('\'');
    let right_quote = right_text.chars().next().unwrap_or('\'');
    let left_inner = &left_text[1..left_text.len() - 1];
    let right_inner = &right_text[1..right_text.len() - 1];

    let merged = if left_quote == '`' || right_quote == '`' {
        // Legacy octal escapes are allowed in sloppy-mode strings but not in templates.
        if (left_quote != '`' && has_legacy_escape(left_inner))
            || (right_quote != '`' && has_legacy_escape(right_inner))
        {
            return vec![];
        }
        let mut left_inner = if left_quote == '`' {
            left_inner.to_string()
        } else {
            escape_for_template(left_inner)
        };
        let right_inner = if right_quote == '`' {
            right_inner.to_string()
        } else {
            escape_for_template(right_inner)
        };
        // `$` + `{` across the boundary would start a substitution.
        if right_inner.starts_with('{') && ends_with_unescaped(&left_inner, '$') {
            left_inner.insert(left_inner.len() - 1, '\\');
        }
        format!("`{left_inner}{right_inner}`")
    } else if left_quote == right_quote {
        format!("{left_quote}{left_inner}{right_inner}{left_quote}")
    } else {
        format!("{left_quote}{left_inner}{}{left_quote}", escape_unescaped(right_inner, left_quote))
    };
    vec![fcx.fixer(FixKind::SafeFix).replace(span, merged).with_message("Merge into one literal")]
}

/// Whether `raw` ends with `ch` that is not escaped by a backslash.
fn ends_with_unescaped(raw: &str, ch: char) -> bool {
    let Some(rest) = raw.strip_suffix(ch) else { return false };
    rest.chars().rev().take_while(|&c| c == '\\').count() % 2 == 0
}

/// Turns the raw contents of a string literal into raw template literal contents with the same
/// value: unescaped backticks and `${` are escaped. Other escapes mean the same in templates.
fn escape_for_template(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut escaped = false;
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if !escaped && (ch == '`' || (ch == '$' && chars.peek() == Some(&'{'))) {
            out.push('\\');
        }
        escaped = !escaped && ch == '\\';
        out.push(ch);
    }
    out
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_useless_concat::NoUselessConcat, tester::Tester,
    };

    let pass = vec!["var a = 'a' + b"];
    let fail = vec!["'a' + 'b'"];

    let fix = vec![
        ("'a' + 'b'", "'ab'", None, FixKind::SafeFix),
        ("\"a\" + \"b\"", "\"ab\"", None, FixKind::SafeFix),
        ("'a' + \"b\"", "'ab'", None, FixKind::SafeFix),
        ("'a' + \"it's\"", "'ait\\'s'", None, FixKind::SafeFix),
        ("'a' + \"it\\'s\"", "'ait\\'s'", None, FixKind::SafeFix),
        ("'\\u00e9' + 'b'", "'\\u00e9b'", None, FixKind::SafeFix),
        ("foo + 'a' + 'b'", "foo + 'ab'", None, FixKind::SafeFix),
        ("'a' + `b`", "`ab`", None, FixKind::SafeFix),
        ("`a${x}` + 'b'", "`a${x}b`", None, FixKind::SafeFix),
        ("'a`$' + `{b}`", "`a\\`\\${b}`", None, FixKind::SafeFix),
        ("`a$` + '{b}'", "`a\\${b}`", None, FixKind::SafeFix),
        ("'a\\\\$' + `{b}`", "`a\\\\\\${b}`", None, FixKind::SafeFix),
        ("'a\\$' + `{b}`", "`a\\${b}`", None, FixKind::SafeFix),
        ("'${x}' + `b`", "`\\${x}b`", None, FixKind::SafeFix),
        // Not only `+` between the literals.
        ("('a') + 'b'", "('a') + 'b'", None, FixKind::SafeFix),
        ("'a' /* x */ + 'b'", "'a' /* x */ + 'b'", None, FixKind::SafeFix),
    ];

    Tester::new(NoUselessConcat::NAME, NoUselessConcat::PLUGIN, pass, fail).expect_fix(fix).test();

    // In a `.vue` file the script block is linted separately; fix spans must still land in the
    // right place in the whole file.
    let vue = vec![(
        "<template><div /></template>\n<script>\nvar a = 'a' + 'b';\n</script>",
        "<template><div /></template>\n<script>\nvar a = 'ab';\n</script>",
        None,
        Some(std::path::PathBuf::from("test.vue")),
    )];
    Tester::new(
        NoUselessConcat::NAME,
        NoUselessConcat::PLUGIN,
        vec!["var a = b"],
        vec!["'a' + 'b'"],
    )
    .expect_fix(vue)
    .test();

    // Legacy octal escapes (sloppy-mode scripts only) are not allowed in templates. Plain strings
    // still merge.
    let script = || Some(std::path::PathBuf::from("a.cjs"));
    let octal = vec![
        ("'\\033[31m' + `${msg}`", "'\\033[31m' + `${msg}`", None, script()),
        ("'\\033[31m' + 'b'", "'\\033[31mb'", None, script()),
    ];
    Tester::new(
        NoUselessConcat::NAME,
        NoUselessConcat::PLUGIN,
        vec!["var a = b"],
        vec!["'a' + 'b'"],
    )
    .expect_fix(octal)
    .test();
}
