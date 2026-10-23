//! Fork-only: with `--format=github` or `--format=json2`, the `oxlint` CLI reports how each issue
//! can be fixed.
//!
//! Upstream only builds a fix when the run may apply it (`--fix`, `--fix-suggestions`,
//! `--fix-dangerously`), so a plain run cannot tell which issues are fixable. When reporting fix
//! levels, the linter builds every kind of fix (see [`build_kind`], used by `ContextHost`), and
//! before fixing records the least risky one on each message ([`before_fixing`]) and drops the
//! kinds the `--fix*` flags did not ask for (all of them without a flag). `LintOptions::fix` keeps
//! upstream's meaning: the kinds to apply. [`wrap_diagnostics`] attaches the level to the reported
//! error, where the `github` format prints it as `fix=<level>` and `json2` as `"fix"`. Other
//! formats leave this off and behave as upstream. Building the extra fixes costs under 1% of lint time (measured
//! 2026-10-23 on the React and ESLint repos).
//!
//! The level lives on `Message` and on a wrapper around the reported error, not on
//! `OxcDiagnostic`, whose `Debug` output upstream snapshots record.
//!
//! Levels, from least to most risky, with the flag that applies them:
//!
//! - `safe`: `--fix`
//! - `suggestion`: `--fix-suggestions` (a fix that mixes both kinds needs `--fix --fix-suggestions`)
//! - `dangerous`: `--fix-dangerously`
//! - `dangerous-suggestion`: `--fix-dangerously`
//!
//! Differences from upstream when this is on, all deliberate: a fix is applied only if its kind was
//! asked for. Upstream applies a few fixes it builds regardless of the flags: the suggestion that
//! removes an unused disable directive in Vue, Astro and Svelte files (applied by plain `--fix`
//! upstream), and JS plugin fixes (also applied by `--fix-suggestions` alone). JS plugins are
//! disabled in this fork.
//!
//! Not covered: type-aware (tsgolint) diagnostics, which this fork disables, the language server,
//! which builds and offers fixes on its own, and unused disable directives in plain JS/TS files,
//! which upstream reports without a fix.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
    path::Path,
    sync::Arc,
};

use oxc_diagnostics::{Diagnostic, DiagnosticService, Error, LabeledSpan, Severity, SourceCode};

use crate::{
    Linter,
    fixer::{Fix, FixKind, Message, PossibleFixes},
    options::LintOptions,
};

impl Linter {
    /// Build every kind of fix and record each diagnostic's fix level. The kinds to apply stay as
    /// set by [`Linter::with_fix`].
    #[must_use]
    pub fn with_report_fixes(mut self, enabled: bool) -> Self {
        self.options.report_fixes = enabled;
        self
    }
}

/// The kinds of fix rules should build: all of them when reporting fix levels, otherwise the kinds
/// to apply.
pub fn build_kind(options: LintOptions) -> FixKind {
    if options.report_fixes { FixKind::All } else { options.fix }
}

/// Called right before fixes are applied to a file's messages. Records each message's fix level
/// and drops the fixes that should not be applied.
pub fn before_fixing(options: LintOptions, messages: &mut [Message]) {
    if !options.report_fixes {
        return;
    }
    let apply = options.fix;
    for message in messages {
        message.fix_level = fix_level(&message.fixes);
        message.fixes = match std::mem::replace(&mut message.fixes, PossibleFixes::None) {
            _ if apply.is_none() => PossibleFixes::None,
            fixes if apply == FixKind::All => fixes,
            PossibleFixes::Single(fix) if !apply.can_apply(fix.kind) => PossibleFixes::None,
            PossibleFixes::Multiple(fixes) => {
                fixes.into_iter().filter(|fix| apply.can_apply(fix.kind)).collect()
            }
            fixes => fixes,
        };
    }
}

/// Like `DiagnosticService::wrap_diagnostics`, but keeps each message's fix level on its error.
pub fn wrap_diagnostics(
    cwd: &Path,
    path: &Path,
    source_text: &str,
    messages: Vec<Message>,
) -> Vec<Error> {
    let source = DiagnosticService::named_source(cwd, path, source_text);
    messages
        .into_iter()
        .map(|message| {
            let fix_level = message.fix_level;
            let error = message.error.with_source_code(Arc::clone(&source));
            match fix_level {
                Some(fix_level) => Box::new(WithFixLevel { error, fix_level }),
                None => error,
            }
        })
        .collect()
}

/// A reported error with its fix level.
struct WithFixLevel {
    error: Error,
    fix_level: &'static str,
}

impl Debug for WithFixLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl Display for WithFixLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.error, f)
    }
}

impl std::error::Error for WithFixLevel {}

impl Diagnostic for WithFixLevel {
    fn code(&self) -> Option<Cow<'_, str>> {
        self.error.code()
    }

    fn severity(&self) -> Option<Severity> {
        self.error.severity()
    }

    fn help(&self) -> Option<Cow<'_, str>> {
        self.error.help()
    }

    fn note(&self) -> Option<Cow<'_, str>> {
        self.error.note()
    }

    fn url(&self) -> Option<Cow<'_, str>> {
        self.error.url()
    }

    fn fix_level(&self) -> Option<&'static str> {
        Some(self.fix_level)
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.error.source_code()
    }

    fn labels(&self) -> &[LabeledSpan] {
        self.error.labels()
    }
}

/// The least risky level among `fixes`.
fn fix_level(fixes: &PossibleFixes) -> Option<&'static str> {
    let fixes: &[Fix] = match fixes {
        PossibleFixes::None => &[],
        PossibleFixes::Single(fix) => std::slice::from_ref(fix),
        PossibleFixes::Multiple(fixes) => fixes,
    };
    fixes.iter().filter_map(|fix| level(fix.kind)).min().map(|(_, name)| name)
}

/// The level of one fix kind, with its rank (lower is less risky): the least risky `--fix*` flag
/// that applies it. A kind that mixes fix and suggestion counts as a suggestion, because `--fix`
/// alone does not apply it.
fn level(kind: FixKind) -> Option<(u8, &'static str)> {
    if kind.is_none() || !FixKind::All.contains(kind) {
        None
    } else if FixKind::SafeFix.contains(kind) {
        Some((0, "safe"))
    } else if !kind.is_dangerous() {
        Some((1, "suggestion"))
    } else if !kind.contains(FixKind::Suggestion) {
        Some((2, "dangerous"))
    } else {
        Some((3, "dangerous-suggestion"))
    }
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::OxcDiagnostic;
    use oxc_span::Span;

    use super::{fix_level, level, wrap_diagnostics};
    use crate::fixer::{Fix, FixKind, Message, PossibleFixes};

    fn fix(kind: FixKind) -> Fix {
        Fix::empty().with_kind(kind)
    }

    #[test]
    fn levels() {
        assert_eq!(level(FixKind::SafeFix).map(|l| l.1), Some("safe"));
        assert_eq!(level(FixKind::Suggestion).map(|l| l.1), Some("suggestion"));
        assert_eq!(level(FixKind::SafeFixOrSuggestion).map(|l| l.1), Some("suggestion"));
        assert_eq!(level(FixKind::DangerousFix).map(|l| l.1), Some("dangerous"));
        assert_eq!(level(FixKind::DangerousSuggestion).map(|l| l.1), Some("dangerous-suggestion"));
        assert_eq!(level(FixKind::None), None);
        assert_eq!(level(FixKind::IgnoreFix), None);
    }

    #[test]
    fn least_risky_wins() {
        assert_eq!(fix_level(&PossibleFixes::None), None);
        assert_eq!(
            fix_level(&PossibleFixes::Single(fix(FixKind::DangerousFix))),
            Some("dangerous")
        );
        assert_eq!(
            fix_level(&PossibleFixes::Multiple(vec![
                fix(FixKind::DangerousSuggestion),
                fix(FixKind::Suggestion),
                fix(FixKind::SafeFix),
            ])),
            Some("safe")
        );
    }

    #[test]
    fn wrapped_errors_keep_the_level() {
        let message = |level| {
            let mut message = Message::new(
                OxcDiagnostic::warn("message")
                    .with_error_code("eslint", "no-var")
                    .with_help("help")
                    .with_note("note")
                    .with_url("https://example.com")
                    .with_label(Span::new(0, 1)),
                PossibleFixes::None,
            );
            message.fix_level = level;
            message
        };
        let errors = wrap_diagnostics(
            ".".as_ref(),
            "a.js".as_ref(),
            "a;",
            vec![message(Some("safe")), message(None)],
        );
        assert_eq!(errors[0].fix_level(), Some("safe"));
        assert_eq!(errors[1].fix_level(), None);
        assert!(errors[0].source_code().is_some());
        assert_eq!(errors[0].to_string(), "message");
        // The wrapper forwards everything the output formats print.
        assert_eq!(errors[0].code().as_deref(), Some("eslint(no-var)"));
        assert_eq!(errors[0].help().as_deref(), Some("help"));
        assert_eq!(errors[0].note().as_deref(), Some("note"));
        assert_eq!(errors[0].url().as_deref(), Some("https://example.com"));
        assert_eq!(errors[0].labels().len(), 1);
    }
}
