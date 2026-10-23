//! `eslint/no-inline-comments`: suggest moving a trailing line comment onto its own line above.
//!
//! `foo(); // note` becomes `// note` on the line above, with the same indentation. Only single-line
//! `//` comments that end a line of code are moved. Comments that tools read in place are left alone:
//! `@ts-` directives, `prettier-ignore`, coverage hints, and anything that disables a rule.
//! A suggestion, because it is a layout change.

use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::{FixCtx, util::line_indent},
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    let span = fcx.span();
    let source = fcx.source_text();
    let comment = fcx.source_range(span);
    if !comment.starts_with("//") {
        return vec![];
    }
    let content = comment[2..].trim();
    let positional = content.starts_with('@')
        || content.starts_with("#__")
        || ["ignore", "disable", "istanbul", "c8 ", "prettier", "noqa", "nolint", "NOSONAR"]
            .iter()
            .any(|marker| content.contains(marker));
    if positional {
        return vec![];
    }

    let (line_start, indent) = line_indent(source, span.start);
    let before = &source[line_start as usize..span.start as usize];
    if before.trim().is_empty() {
        return vec![];
    }
    // The line may start inside a multi-line token, where the inserted comment would become text:
    // a template or string continued over lines, JSX children, or a block comment.
    let in_token = fcx.ctx.get_comment_at(line_start).is_some_and(|c| c.span.start < line_start)
        || fcx
            .innermost(Span::empty(line_start), |node| {
                node.kind().span().start < line_start
                    && matches!(
                        node.kind(),
                        AstKind::TemplateLiteral(_)
                            | AstKind::TSTemplateLiteralType(_)
                            | AstKind::StringLiteral(_)
                            | AstKind::JSXElement(_)
                            | AstKind::JSXFragment(_)
                    )
            })
            .is_some();
    if in_token {
        return vec![];
    }
    let Ok(code_len) = u32::try_from(before.trim_end().len()) else { return vec![] };
    let code_end = line_start + code_len;

    let fixer = fcx.fixer(FixKind::Suggestion).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    fix.push(Fix::new(format!("{indent}{comment}\n"), Span::empty(line_start)));
    fix.push(Fix::delete(Span::new(code_end, span.end)));
    vec![fix.with_message("Move the comment to its own line")]
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_inline_comments::NoInlineComments, tester::Tester,
    };

    let pass = vec!["// comment\nfoo();"];
    let fail = vec!["foo(); // comment"];

    let fix = vec![
        ("foo(); // comment", "// comment\nfoo();", None, FixKind::Suggestion),
        ("foo(); // comment", "foo(); // comment", None, FixKind::SafeFix),
        (
            "function f() {\n  foo(); // note\n}",
            "function f() {\n  // note\n  foo();\n}",
            None,
            FixKind::Suggestion,
        ),
        // Comments that tools read in place stay where they are.
        ("foo(); // @ts-ignore", "foo(); // @ts-ignore", None, FixKind::Suggestion),
        ("foo(); // prettier-ignore", "foo(); // prettier-ignore", None, FixKind::Suggestion),
        // The line starts inside a template or block comment; the comment would land inside it.
        ("const s = `a\nb`; // note", "const s = `a\nb`; // note", None, FixKind::Suggestion),
        ("/* a\nb */ foo(); // note", "/* a\nb */ foo(); // note", None, FixKind::Suggestion),
        // A block comment that starts the line is fine.
        ("/* a */ foo(); // note", "// note\n/* a */ foo();", None, FixKind::Suggestion),
        // Block comments are not moved.
        ("foo(/* a */ 1);", "foo(/* a */ 1);", None, FixKind::Suggestion),
    ];

    Tester::new(NoInlineComments::NAME, NoInlineComments::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
