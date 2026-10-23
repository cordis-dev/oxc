//! Fork-only autofixes (cordis-dev/oxc).
//!
//! This module adds fixes to upstream rules **without editing the upstream rule files**, so that
//! merging upstream `oxc-project/oxc` stays conflict-free. Only these marked hunks touch upstream
//! files (search for `fork(cordis-dev)`):
//!
//! - `lib.rs`: `mod fork_fixes;`
//! - `context/mod.rs` `LintContext::add_diagnostic`: calls [`apply`] before the diagnostic is
//!   pushed to the host.
//! - `table.rs`, `apps/oxlint/src/output_formatter/{json,sarif}.rs`: use
//!   [`RuleEnum::effective_fix`] so `oxlint --rules`, JSON/SARIF output and generated docs report
//!   the fork's fixes.
//!
//! Fix levels in the `github` and `json2` output formats (see [`report`]) add these marked hunks:
//!
//! - `options/mod.rs`: the `report_fixes` field of `LintOptions`.
//! - `context/host.rs`: builds fixes of the kinds [`report::build_kind`] returns.
//! - `service/runtime.rs`: calls [`report::before_fixing`] before fixes are applied and
//!   [`report::wrap_diagnostics`] when reporting.
//! - `fixer/mod.rs`: the `fix_level` field of `Message`.
//! - `oxc_diagnostics` `protocol.rs`: the `Diagnostic::fix_level` method.
//! - `apps/oxlint/src/lint.rs`: enables it for `--format=github` and `json2`;
//!   `output_formatter/github.rs`: the `fix=` property; `output_formatter/mod.rs`: registers the
//!   fork-only `json2` format (`output_formatter/json2.rs`).
//!
//! ## How it works
//!
//! Every native diagnostic passes through `LintContext::add_diagnostic`. When fixes are being built
//! (CLI `--fix*`, the language server, or any `--format=github` or `json2` run, which builds every kind to
//! report fix levels), the hook looks up the reporting rule in [`ENTRIES`] and runs its fixer, which
//! inspects the diagnostic and the AST and returns zero or more [`RuleFix`]es. Fixes of kinds that
//! are not being built are dropped, exactly like upstream's own fixes; [`report::before_fixing`]
//! later drops the kinds that should not be applied.
//!
//! - [`Mode::Fill`]: the fixer runs only when the diagnostic carries no fix. If upstream later
//!   implements its own fix for a rule, upstream's fix wins automatically. When upstream adds only
//!   a suggestion to a covered rule, the fork's safe fix stops being built: re-check the rule (the
//!   `upstream_fix_meta_unchanged` test flags it) and switch it to [`Mode::Upgrade`].
//! - [`Mode::Upgrade`]: the rule already offers a suggestion. The fixer returns a safe fix only when
//!   one is provably correct; that fix is placed first and upstream's suggestions are kept after it.
//!
//! The hook never modifies `message.error`, so upstream snapshot files are unaffected. Spans are
//! relative to the current source block (Vue/Svelte/Astro `<script>`), the same as for rules, because
//! the hook runs before the host shifts them.
//!
//! The hook cannot see a rule's configuration. Fixers that depend on an option infer it from the
//! diagnostic text, which their tests pin down.
//!
//! ## Adding a fixer
//!
//! 1. Add `fork_fixes/<plugin>/<rule>.rs` with `pub fn fix(fcx: &FixCtx) -> Vec<RuleFix>`
//!    and a `#[test]` that uses `Tester::new(..).expect_fix(..).test()`. Never use
//!    `test_and_snapshot`, which would overwrite the upstream rule's snapshot.
//! 2. Declare the module in `fork_fixes/<plugin>/mod.rs`.
//! 3. Add a line to the `fork_fixers!` registry below.
//!
//! ## On upstream sync
//!
//! `upstream_fix_meta_unchanged` fails when upstream changes a covered rule's fix metadata, for
//! example because it implemented its own fix. Compare the two, then delete the fork fixer or switch
//! it to [`Mode::Upgrade`].
//!
//! ## Rules deliberately not covered
//!
//! - Rules that cannot be fixed in principle: size limits (`max-*`, `complexity`), naming
//!   (`id-*`, `new-cap`, `no-underscore-dangle`), `no-restricted-*`, scope and hoisting problems
//!   (`no-undef`, `no-use-before-define`, ...), and logic bugs whose intent is unknown
//!   (`no-constant-condition`, `no-dupe-else-if`, `no-unsafe-finally`, ...).
//! - `default-case`: adding `default: break;` only silences the warning.
//! - Suggestion-only rules where ESLint also only suggests, because the rewrite changes behavior:
//!   `no-case-declarations`, `no-console`, `no-empty`, `no-empty-function`,
//!   `no-empty-static-block`, `no-misleading-character-class`, `no-nonoctal-decimal-escape`,
//!   `no-useless-constructor`, `no-debugger`, `no-iterator`, `no-throw-literal`, and `no-plusplus`
//!   (`i += 1` throws on BigInt).

mod ctx;
mod eslint;
mod nodes;
pub mod report;
mod util;

#[cfg(test)]
mod corpus;

pub use ctx::FixCtx;

use crate::{
    context::LintContext,
    fixer::{Fix, FixKind, Message, PossibleFixes, RuleFix},
    rule::RuleFixMeta,
    rules::RuleEnum,
};

/// When a fork fixer runs, relative to fixes the upstream rule already provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Run only when the diagnostic carries no fix (of the kinds being built).
    Fill,
    /// Run when the diagnostic carries no safe fix, and put the fork's fix first.
    Upgrade,
}

/// One fork fixer.
pub struct Entry {
    pub plugin: &'static str,
    /// Kebab-case rule name, as reported in diagnostics.
    pub rule: &'static str,
    pub mode: Mode,
    /// Every fix kind this fixer can emit, exactly.
    pub kinds: &'static [FixKind],
    /// The fix metadata upstream declares for the rule. Checked by `upstream_fix_meta_unchanged`.
    #[cfg(test)]
    pub upstream_meta: RuleFixMeta,
    pub fixer: for<'c, 'a> fn(&FixCtx<'c, 'a>) -> Vec<RuleFix>,
    #[cfg(test)]
    pub src: &'static str,
}

macro_rules! fork_fixers {
    ($( $plugin:ident :: $module:ident => $name:literal, $mode:ident, [$($kind:ident),+], $meta:expr; )*) => {
        /// Every fork fixer, sorted by plugin and rule name.
        pub static ENTRIES: &[Entry] = &[
            $(
                Entry {
                    plugin: stringify!($plugin),
                    rule: $name,
                    mode: Mode::$mode,
                    kinds: &[$(FixKind::$kind),+],
                    #[cfg(test)]
                    upstream_meta: $meta,
                    fixer: $plugin::$module::fix,
                    #[cfg(test)]
                    src: include_str!(concat!(stringify!($plugin), "/", stringify!($module), ".rs")),
                },
            )*
        ];
    };
}

#[cfg(test)]
use RuleFixMeta as M;

fork_fixers! {
    eslint::array_callback_return => "array-callback-return", Fill, [Suggestion], M::FixPending;
    eslint::default_case_last => "default-case-last", Fill, [DangerousFix], M::None;
    eslint::func_style => "func-style", Fill, [DangerousFix], M::FixPending;
    eslint::grouped_accessor_pairs => "grouped-accessor-pairs", Fill, [DangerousFix], M::FixPending;
    eslint::guard_for_in => "guard-for-in", Fill, [Suggestion], M::None;
    eslint::init_declarations => "init-declarations", Fill, [SafeFix, Suggestion], M::None;
    eslint::logical_assignment_operators => "logical-assignment-operators", Fill, [SafeFix, Suggestion], M::FixPending;
    eslint::no_async_promise_executor => "no-async-promise-executor", Fill, [DangerousFix], M::None;
    eslint::no_cond_assign => "no-cond-assign", Fill, [Suggestion], M::None;
    eslint::no_const_assign => "no-const-assign", Fill, [Suggestion], M::None;
    eslint::no_dupe_class_members => "no-dupe-class-members", Fill, [Suggestion], M::None;
    eslint::no_dupe_keys => "no-dupe-keys", Fill, [Suggestion], M::None;
    eslint::no_duplicate_imports => "no-duplicate-imports", Fill, [SafeFix], M::FixPending;
    eslint::no_extra_bind => "no-extra-bind", Fill, [SafeFix], M::FixPending;
    eslint::no_fallthrough => "no-fallthrough", Fill, [Suggestion, DangerousSuggestion], M::FixPending;
    eslint::no_implied_eval => "no-implied-eval", Fill, [Suggestion], M::None;
    eslint::no_inline_comments => "no-inline-comments", Fill, [Suggestion], M::None;
    eslint::no_irregular_whitespace => "no-irregular-whitespace", Fill, [SafeFix], M::None;
    eslint::no_lone_blocks => "no-lone-blocks", Fill, [SafeFix], M::None;
    eslint::no_lonely_if => "no-lonely-if", Fill, [SafeFix], M::FixPending;
    eslint::no_loss_of_precision => "no-loss-of-precision", Fill, [Suggestion], M::None;
    eslint::no_multi_assign => "no-multi-assign", Fill, [DangerousFix], M::None;
    eslint::no_multi_str => "no-multi-str", Fill, [SafeFix], M::None;
    eslint::no_new_native_nonconstructor => "no-new-native-nonconstructor", Fill, [Suggestion], M::None;
    eslint::no_object_constructor => "no-object-constructor", Fill, [SafeFix], M::FixPending;
    eslint::no_promise_executor_return => "no-promise-executor-return", Fill, [Suggestion], M::FixPending;
    eslint::no_proto => "no-proto", Fill, [DangerousFix], M::FixPending;
    eslint::no_prototype_builtins => "no-prototype-builtins", Fill, [Suggestion], M::FixPending;
    eslint::no_redeclare => "no-redeclare", Fill, [SafeFix], M::None;
    eslint::no_return_assign => "no-return-assign", Fill, [Suggestion], M::None;
    eslint::no_self_assign => "no-self-assign", Fill, [Suggestion], M::None;
    eslint::no_self_compare => "no-self-compare", Fill, [Suggestion], M::None;
    eslint::no_sequences => "no-sequences", Fill, [SafeFix], M::None;
    eslint::no_setter_return => "no-setter-return", Fill, [SafeFix], M::None;
    eslint::no_template_curly_in_string => "no-template-curly-in-string", Fill, [Suggestion], M::None;
    eslint::no_undefined => "no-undefined", Fill, [SafeFix], M::None;
    eslint::no_unreachable => "no-unreachable", Fill, [DangerousFix], M::None;
    eslint::no_unused_expressions => "no-unused-expressions", Fill, [Suggestion], M::None;
    eslint::no_unused_private_class_members => "no-unused-private-class-members", Fill, [Suggestion], M::FixPending;
    eslint::no_useless_assignment => "no-useless-assignment", Fill, [Suggestion], M::None;
    eslint::no_useless_backreference => "no-useless-backreference", Fill, [SafeFix], M::None;
    eslint::no_useless_call => "no-useless-call", Fill, [Suggestion], M::None;
    eslint::no_useless_catch => "no-useless-catch", Fill, [SafeFix], M::None;
    eslint::no_useless_concat => "no-useless-concat", Fill, [SafeFix], M::FixPending;
    eslint::no_useless_return => "no-useless-return", Fill, [SafeFix], M::FixPending;
    eslint::no_void => "no-void", Upgrade, [SafeFix], M::Fixable(FixKind::Suggestion);
    eslint::prefer_named_capture_group => "prefer-named-capture-group", Fill, [Suggestion], M::FixPending;
    eslint::prefer_promise_reject_errors => "prefer-promise-reject-errors", Fill, [Suggestion], M::None;
    eslint::prefer_regex_literals => "prefer-regex-literals", Fill, [Suggestion], M::FixPending;
    eslint::prefer_spread => "prefer-spread", Fill, [DangerousFix], M::None;
    eslint::require_unicode_regexp => "require-unicode-regexp", Fill, [Suggestion], M::FixPending;
    eslint::symbol_description => "symbol-description", Fill, [Suggestion], M::None;
    eslint::vars_on_top => "vars-on-top", Fill, [DangerousFix], M::None;
}

fn find_entry(plugin: &str, rule: &str) -> Option<&'static Entry> {
    ENTRIES
        .binary_search_by(|entry| (entry.plugin, entry.rule).cmp(&(plugin, rule)))
        .ok()
        .map(|index| &ENTRIES[index])
}

/// Hook called from `LintContext::add_diagnostic` when fixing is enabled.
///
/// `allowed` is the fix kind the caller asked for. Fixes of other kinds are dropped.
#[inline(never)]
pub fn apply(
    ctx: &LintContext<'_>,
    plugin: &str,
    rule: &str,
    allowed: FixKind,
    message: &mut Message,
) {
    let Some(entry) = find_entry(plugin, rule) else { return };
    match entry.mode {
        Mode::Fill if !message.fixes.is_empty() => return,
        Mode::Upgrade if has_safe_fix(&message.fixes) => return,
        Mode::Fill | Mode::Upgrade => {}
    }
    // Skip the fixer when the caller would drop everything it can produce.
    if !entry.kinds.iter().any(|&kind| allowed.can_apply(kind)) {
        return;
    }

    let rule_fixes = (entry.fixer)(&FixCtx::new(ctx, message));
    let mut fixes = Vec::with_capacity(rule_fixes.len());
    for rule_fix in rule_fixes {
        debug_assert!(
            entry.kinds.contains(&rule_fix.kind()),
            "fork fixer {plugin}/{rule} emitted {:?}, which is not in its declared kinds {:?}",
            rule_fix.kind(),
            entry.kinds
        );
        if rule_fix.is_empty() || !allowed.can_apply(rule_fix.kind()) {
            continue;
        }
        let fix = rule_fix.into_fix(ctx.source_text());
        debug_assert!(
            fix.span.size() <= 1 || fix.message.as_ref().is_some_and(|m| !m.is_empty()),
            "fork fixer {plugin}/{rule} must give a message for a fix spanning more than one character"
        );
        fixes.push(fix);
    }
    if fixes.is_empty() {
        return;
    }

    if entry.mode == Mode::Upgrade {
        match std::mem::replace(&mut message.fixes, PossibleFixes::None) {
            PossibleFixes::None => {}
            PossibleFixes::Single(fix) => fixes.push(fix),
            PossibleFixes::Multiple(upstream) => fixes.extend(upstream),
        }
    }
    message.fixes = fixes.into_iter().collect();
}

fn has_safe_fix(fixes: &PossibleFixes) -> bool {
    let is_safe = |fix: &Fix| fix.kind == FixKind::SafeFix;
    match fixes {
        PossibleFixes::None => false,
        PossibleFixes::Single(fix) => is_safe(fix),
        PossibleFixes::Multiple(fixes) => fixes.iter().any(is_safe),
    }
}

impl RuleEnum {
    /// The rule's fix capabilities including fork-only fixes.
    ///
    /// Use this instead of [`RuleEnum::fix`] wherever fix capabilities are shown to users. The
    /// generated `fix()` stays upstream's declaration, because the test harness requires fix tests
    /// for every rule whose `fix()` reports a fix.
    pub fn effective_fix(&self) -> RuleFixMeta {
        let declared = self.fix();
        match find_entry(self.plugin_name(), self.name()) {
            Some(entry) => RuleFixMeta::Conditional(
                entry.kinds.iter().fold(declared.fix_kind(), |all, &kind| all | kind),
            ),
            None => declared,
        }
    }
}

#[cfg(test)]
mod test {
    use super::ENTRIES;
    use crate::rules::RULES;

    /// Fails when upstream changes a covered rule's fix metadata, for example because it added its
    /// own fix. Compare upstream's fix with the fork's, then delete the fork fixer or rework it.
    #[test]
    fn upstream_fix_meta_unchanged() {
        for entry in ENTRIES {
            let rule = RULES
                .iter()
                .find(|rule| rule.plugin_name() == entry.plugin && rule.name() == entry.rule)
                .unwrap_or_else(|| {
                    panic!(
                        "{}/{} no longer exists upstream; delete its fork fixer",
                        entry.plugin, entry.rule
                    )
                });
            assert_eq!(
                rule.fix(),
                entry.upstream_meta,
                "{}/{}: upstream changed its fix metadata. Compare upstream's fix with the fork fixer, \
                 then delete the fork fixer or switch it to Mode::Upgrade.",
                entry.plugin,
                entry.rule
            );
        }
    }

    #[test]
    fn every_fixer_has_tests() {
        for entry in ENTRIES {
            for needle in ["#[test]", "Tester::new(", ".expect_fix(", ".test()"] {
                assert!(
                    entry.src.contains(needle),
                    "fork fixer {}/{} has no `{needle}` in its tests",
                    entry.plugin,
                    entry.rule
                );
            }
            assert!(
                !entry.src.contains("test_and_snapshot"),
                "fork fixer {}/{} must not use test_and_snapshot; it would overwrite upstream snapshots",
                entry.plugin,
                entry.rule
            );
        }
    }

    /// `apply` checks each emitted fix against `kinds`, so `kinds` may only hold the four standard
    /// kinds.
    #[test]
    fn registry_kinds_standard() {
        use crate::fixer::FixKind;
        for entry in ENTRIES {
            for kind in entry.kinds {
                assert!(
                    matches!(
                        *kind,
                        FixKind::SafeFix
                            | FixKind::Suggestion
                            | FixKind::DangerousFix
                            | FixKind::DangerousSuggestion
                    ),
                    "fork fixer {}/{} declares a non-standard fix kind {kind:?}",
                    entry.plugin,
                    entry.rule
                );
            }
        }
    }

    /// `find_entry` binary-searches the registry, so it must be strictly sorted (which also rules
    /// out duplicates).
    #[test]
    fn registry_sorted() {
        for pair in ENTRIES.windows(2) {
            assert!(
                (pair[0].plugin, pair[0].rule) < (pair[1].plugin, pair[1].rule),
                "fork fixer registry must be sorted without duplicates: {} then {}",
                pair[0].rule,
                pair[1].rule
            );
        }
    }
}
