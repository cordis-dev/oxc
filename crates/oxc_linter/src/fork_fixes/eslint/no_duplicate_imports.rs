//! `eslint/no-duplicate-imports`: merge a duplicated import into the earlier one.
//!
//! The earlier import decides when the module is evaluated, so specifiers are always moved into the
//! earlier import and the later one is deleted; imported bindings are hoisted, so moving them up
//! changes nothing. Handled cases:
//!
//! - A later side-effect import (`import 'x'`) is deleted.
//! - Named specifiers join the earlier import's braces, or are added after its default import.
//! - A default-only import merges into an earlier import without a default.
//! - An earlier side-effect import becomes the merged import.
//!
//! No fix for namespace imports, differing `import type` kinds, import attributes or phases,
//! exports, or when comments would be lost.

use oxc_ast::{
    AstKind,
    ast::{ImportDeclaration, ImportDeclarationSpecifier},
};
use oxc_span::GetSpan;

use crate::{
    fixer::{Fix, FixKind, RuleFix},
    fork_fixes::FixCtx,
};

pub fn fix(fcx: &FixCtx) -> Vec<RuleFix> {
    if !fcx.text().ends_with("import is duplicated") {
        return vec![];
    }
    let spans = fcx.label_spans();
    let [later, earlier] = spans.as_slice() else { return vec![] };
    let find = |span| {
        fcx.innermost(span, |node| matches!(node.kind(), AstKind::ImportDeclaration(_)))
            .and_then(|node| node.kind().as_import_declaration())
    };
    let (Some(later), Some(earlier)) = (find(*later), find(*earlier)) else { return vec![] };
    if later.span == earlier.span
        || later.import_kind != earlier.import_kind
        || later.phase.is_some()
        || earlier.phase.is_some()
        || later.with_clause.is_some()
        || earlier.with_clause.is_some()
        || fcx.ctx.has_comments_between(later.span)
        || fcx.ctx.has_comments_between(earlier.span)
    {
        return vec![];
    }

    let fixer = fcx.fixer(FixKind::SafeFix).for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    let delete_later = Fix::delete(later.span);
    let later_specifiers = later.specifiers.as_ref().map_or(&[][..], |specifiers| &specifiers[..]);
    if later_specifiers.is_empty() {
        fix.push(delete_later);
        return vec![fix.with_message("Remove the duplicated import")];
    }

    let Some(insertion) = merge(fcx, earlier, later_specifiers) else { return vec![] };
    fix.push(insertion);
    fix.push(delete_later);
    vec![fix.with_message("Merge into the earlier import")]
}

/// The edit that adds `specifiers` to `earlier`.
fn merge(
    fcx: &FixCtx,
    earlier: &ImportDeclaration,
    specifiers: &[ImportDeclarationSpecifier],
) -> Option<Fix> {
    let mut default = None;
    let mut named = vec![];
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => default = Some(spec.span),
            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                named.push(fcx.source_range(spec.span));
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => return None,
        }
    }
    let default = default.map(|span| fcx.source_range(span));
    let named = named.join(", ");

    let earlier_specifiers =
        earlier.specifiers.as_ref().map_or(&[][..], |specifiers| &specifiers[..]);
    let earlier_default = earlier_specifiers
        .iter()
        .find(|spec| matches!(spec, ImportDeclarationSpecifier::ImportDefaultSpecifier(_)));
    let last_earlier_named = earlier_specifiers
        .iter()
        .rfind(|spec| matches!(spec, ImportDeclarationSpecifier::ImportSpecifier(_)));
    if earlier_specifiers
        .iter()
        .any(|spec| matches!(spec, ImportDeclarationSpecifier::ImportNamespaceSpecifier(_)))
        || (default.is_some() && earlier_default.is_some())
    {
        return None;
    }

    let source = fcx.source_range(earlier.source.span);
    let keyword = if earlier.import_kind.is_type() { "import type" } else { "import" };
    if earlier_specifiers.is_empty() {
        // `import 'x'` (no braces in the source) becomes the merged import.
        if fcx.source_range(earlier.span).contains('{') {
            return None;
        }
        let clause = match (default, named.is_empty()) {
            (Some(default), true) => default.to_string(),
            (Some(default), false) => format!("{default}, {{ {named} }}"),
            (None, _) => format!("{{ {named} }}"),
        };
        let semicolon = if fcx.source_range(earlier.span).ends_with(';') { ";" } else { "" };
        return Some(Fix::new(
            format!("{keyword} {clause} from {source}{semicolon}"),
            earlier.span,
        ));
    }

    match (default, last_earlier_named) {
        // Add to the earlier braces, and a default in front of them.
        (default, Some(last_named)) => {
            let last_named_end = last_named.span().end;
            let named = if named.is_empty() { String::new() } else { format!(", {named}") };
            let Some(default) = default else {
                return (!named.is_empty())
                    .then(|| Fix::new(named, oxc_span::Span::empty(last_named_end)));
            };
            let first = earlier_specifiers[0].span().start;
            let open_brace = u32::try_from(fcx.source_text()[..first as usize].rfind('{')?).ok()?;
            // A single fix covering both insertion points keeps the edit atomic.
            let end = if named.is_empty() { open_brace } else { last_named_end };
            let between = &fcx.source_text()[open_brace as usize..end as usize];
            Some(Fix::new(
                format!("{default}, {between}{named}"),
                oxc_span::Span::new(open_brace, end),
            ))
        }
        // `import d from 'x'` + named → `import d, { .. } from 'x'`.
        // Not when the earlier import already has empty braces (`import d, {} from 'x'`).
        (None, None) => {
            let default = earlier_default?;
            let after_default = oxc_span::Span::new(default.span().end, earlier.source.span.start);
            if fcx.source_range(after_default).contains('{') {
                return None;
            }
            Some(Fix::new(format!(", {{ {named} }}"), oxc_span::Span::empty(default.span().end)))
        }
        // Defaults on both sides were rejected above.
        (Some(_), None) => None,
    }
}

#[test]
fn test() {
    use crate::{
        rule::RuleMeta, rules::eslint::no_duplicate_imports::NoDuplicateImports, tester::Tester,
    };

    let pass = vec!["import { a, b } from 'x';"];
    let fail = vec!["import { a } from 'x'; import { b } from 'x';"];

    let fix = vec![
        (
            "import { a } from 'x';\nimport { b } from 'x';",
            "import { a, b } from 'x';\n",
            None,
            FixKind::SafeFix,
        ),
        (
            "import { a } from 'x';\nimport { b as c, d } from 'x';",
            "import { a, b as c, d } from 'x';\n",
            None,
            FixKind::SafeFix,
        ),
        (
            "import d from 'x';\nimport { b } from 'x';",
            "import d, { b } from 'x';\n",
            None,
            FixKind::SafeFix,
        ),
        (
            "import { a } from 'x';\nimport d from 'x';",
            "import d, { a } from 'x';\n",
            None,
            FixKind::SafeFix,
        ),
        // A default and named specifiers both move into the earlier braces.
        (
            "import { a } from 'x'; import d, { b } from 'x';",
            "import d, { a, b } from 'x'; ",
            None,
            FixKind::SafeFix,
        ),
        ("import 'x';\nimport 'x';", "import 'x';\n", None, FixKind::SafeFix),
        (
            "import d, {} from 'x';\nimport { b } from 'x';",
            "import d, {} from 'x';\nimport { b } from 'x';",
            None,
            FixKind::SafeFix,
        ),
        (
            "import { a, } from 'x';\nimport { b } from 'x';",
            "import { a, b, } from 'x';\n",
            None,
            FixKind::SafeFix,
        ),
        // A comment would be lost.
        (
            "import { a } from 'x';\nimport { b } /* why */ from 'x';",
            "import { a } from 'x';\nimport { b } /* why */ from 'x';",
            None,
            FixKind::SafeFix,
        ),
    ];

    Tester::new(NoDuplicateImports::NAME, NoDuplicateImports::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test();
}
