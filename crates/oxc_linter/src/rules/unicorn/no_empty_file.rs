use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::{ContextHost, LintContext},
    loader::LINT_PARTIAL_LOADER_EXTENSIONS,
    rule::Rule,
    utils::is_empty_stmt,
};

fn no_empty_file_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty files are not allowed.")
        .with_help("Delete this file or add some code to it.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyFile;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows files that do not contain any meaningful code.
    ///
    /// This includes files that consist only of:
    /// - Whitespace
    /// - Comments
    /// - Directives (e.g., `"use strict"`)
    /// - Empty statements (`;`)
    /// - Empty blocks (`{}`)
    /// - Hashbangs (`#!/usr/bin/env node`)
    ///
    /// ### Why is this bad?
    ///
    /// Files with no executable or exportable content are typically unintentional
    /// or left over from refactoring. They clutter the codebase and may confuse
    /// tooling or developers by appearing to serve a purpose when they do not.
    NoEmptyFile,
    unicorn,
    correctness,
);

impl Rule for NoEmptyFile {
    fn run_once(&self, ctx: &LintContext) {
        let program = ctx.nodes().program().unwrap();
        if program.body.iter().any(|node| !is_empty_stmt(node)) {
            return;
        }

        if has_triple_slash_directive(ctx) {
            return;
        }

        let mut span = program.span;
        // only show diagnostic for the first 100 characters to avoid huge diagnostic messages with
        // empty programs containing a bunch of comments.
        // NOTE: if the enable/disable directives come after the first 100 characters they won't be
        // respected by this diagnostic.
        span.end = std::cmp::min(span.end, 100);
        ctx.diagnostic(no_empty_file_diagnostic(span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.file_path().extension().is_some_and(|ext| {
            !LINT_PARTIAL_LOADER_EXTENSIONS.contains(&ext.to_string_lossy().as_ref())
        })
    }
}

fn has_triple_slash_directive(ctx: &LintContext<'_>) -> bool {
    for comment in ctx.comments() {
        if !comment.is_line() {
            continue;
        }
        let text = ctx.source_range(comment.content_span());

        // `comment.content_span` doesn't include the leading `//` of the comment
        if text.starts_with('/') {
            return true;
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const x = 0;",
        r";; const x = 0;",
        r"{{{;;const x = 0;}}}",
        r"
            'use strict';
            const x = 0;
        ",
        ";;'use strict';",
        "{'use strict';}",
        r#"("use strict")"#,
        r"`use strict`",
        r"({})",
        r"#!/usr/bin/env node
            console.log('done');
        ",
        r"false",
        r#"("")"#,
        r"NaN",
        r"undefined",
        r"null",
        r"[]",
        r"(() => {})()",
        "(() => {})();",
        "/* eslint-disable no-empty-file */",
        r#"/// <reference types="vite/client" />"#,
    ];

    let fail = vec![
        r"",
        r" ",
        "\t",
        "\n",
        "\r",
        "\r\n",
        r"

        ",
        r"// comment",
        r"/* comment */",
        r"#!/usr/bin/env node",
        "'use asm';",
        "'use strict';",
        r#""use strict""#,
        r#""""#,
        r";",
        r";;",
        r"{}",
        r"{;;}",
        r"{{}}",
        r#""";"#,
        r#""use strict";"#,
    ];

    Tester::new(NoEmptyFile::NAME, NoEmptyFile::PLUGIN, pass, fail).test_and_snapshot();
}
