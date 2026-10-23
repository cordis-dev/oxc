//! `eslint/no-template-curly-in-string`: suggest turning the string into a template literal.
//!
//! `'Hello ${name}'` becomes `` `Hello ${name}` ``, so the placeholder interpolates. A suggestion,
//! because it changes the string's value. Unescaped backticks are escaped; other escapes mean the
//! same in templates. No suggestion where a template literal is not allowed, such as import sources,
//! property keys and directives, or when a placeholder is not a valid expression.

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_parser::Parser;

use crate::{
    fixer::{FixKind, RuleFix},
    fork_fixes::{
        FixCtx,
        util::{escape_unescaped, has_legacy_escape, in_expression_slot},
    },
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let Some(node) = fcx.exact(span, |node| matches!(node.kind(), AstKind::StringLiteral(_)))
    else {
        return vec![];
    };
    if !in_expression_slot(fcx, node.id()) {
        return vec![];
    }
    let raw = fcx.source_range(span);
    let inner = &raw[1..raw.len() - 1];
    // Legacy octal escapes are allowed in sloppy-mode strings but not in templates.
    if has_legacy_escape(inner) {
        return vec![];
    }
    let out = format!("`{}`", escape_unescaped(inner, '`'));
    // Upstream reports any `${` followed by `}`, so the placeholders may not be valid code: `'${}'`
    // or `'${a b}'` would become a syntax error.
    let allocator = Allocator::default();
    if Parser::new(&allocator, &out, *fcx.ctx.source_type()).parse().diagnostics.has_errors() {
        return vec![];
    }
    vec![fcx.fixer(FixKind::Suggestion).replace(span, out).with_message("Use a template literal")]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_template_curly_in_string::NoTemplateCurlyInString,
        tester::Tester,
    };

    let pass = vec!["`Hello, ${name}`"];
    let fail = vec!["'Hello, ${name}'"];

    let fix = vec![
        ("var a = 'Hello, ${name}'", "var a = `Hello, ${name}`", None, FixKind::Suggestion),
        ("var a = 'Hello, ${name}'", "var a = 'Hello, ${name}'", None, FixKind::SafeFix),
        ("var a = \"Hi `${x}`\"", "var a = `Hi \\`${x}\\``", None, FixKind::Suggestion),
        ("var a = 'it\\'s ${x}'", "var a = `it\\'s ${x}`", None, FixKind::Suggestion),
        ("var o = { 'a${b}': 1 }", "var o = { 'a${b}': 1 }", None, FixKind::Suggestion),
        // The placeholder would not be a valid expression.
        ("var a = '${}'", "var a = '${}'", None, FixKind::Suggestion),
        ("var a = '${a b}'", "var a = '${a b}'", None, FixKind::Suggestion),
    ];

    Tester::new(
        NoTemplateCurlyInString::NAME,
        NoTemplateCurlyInString::PLUGIN,
        pass.clone(),
        fail.clone(),
    )
    .expect_fix(fix)
    .test();

    // Legacy octal escapes (sloppy-mode scripts only) are not allowed in templates.
    let octal = vec![(
        "var a = '\\033[1m${name}'",
        "var a = '\\033[1m${name}'",
        None,
        Some(std::path::PathBuf::from("a.cjs")),
    )];
    Tester::new(NoTemplateCurlyInString::NAME, NoTemplateCurlyInString::PLUGIN, pass, fail)
        .expect_fix(octal)
        .test();
}
