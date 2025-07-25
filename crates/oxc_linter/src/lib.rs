#![expect(clippy::self_named_module_files)] // for rules.rs
#![allow(clippy::literal_string_with_formatting_args)]

use std::{path::Path, rc::Rc, sync::Arc};

use oxc_semantic::{AstNode, Semantic};

#[cfg(test)]
mod tester;

mod ast_util;
mod config;
mod context;
mod disable_directives;
mod external_linter;
mod external_plugin_store;
mod fixer;
mod frameworks;
mod globals;
mod module_graph_visitor;
mod module_record;
mod options;
mod rule;
mod service;
mod utils;

pub mod loader;
pub mod rules;
pub mod table;

pub use crate::{
    config::{
        BuiltinLintPlugins, Config, ConfigBuilderError, ConfigStore, ConfigStoreBuilder,
        ESLintRule, LintPlugins, Oxlintrc,
    },
    context::LintContext,
    external_linter::{
        ExternalLinter, ExternalLinterCb, ExternalLinterLoadPluginCb, LintResult, PluginLoadResult,
    },
    external_plugin_store::{ExternalPluginStore, ExternalRuleId},
    fixer::FixKind,
    frameworks::FrameworkFlags,
    loader::LINTABLE_EXTENSIONS,
    module_record::ModuleRecord,
    options::LintOptions,
    options::{AllowWarnDeny, InvalidFilterKind, LintFilter, LintFilterKind},
    rule::{RuleCategory, RuleFixMeta, RuleMeta},
    service::{LintService, LintServiceOptions, RuntimeFileSystem},
    utils::read_to_arena_str,
    utils::read_to_string,
};
use crate::{
    config::{LintConfig, OxlintEnv, OxlintGlobals, OxlintSettings, ResolvedLinterState},
    context::ContextHost,
    fixer::{Fixer, Message},
    rules::RuleEnum,
    utils::iter_possible_jest_call_node,
};

#[cfg(feature = "language_server")]
pub use crate::fixer::{FixWithPosition, MessageWithPosition, PossibleFixesWithPosition};

#[cfg(target_pointer_width = "64")]
#[test]
fn size_asserts() {
    // `RuleEnum` runs in a really tight loop, make sure it is small for CPU cache.
    // A reduction from 168 bytes to 16 results 15% performance improvement.
    // See codspeed in https://github.com/oxc-project/oxc/pull/1783
    assert_eq!(size_of::<RuleEnum>(), 16);
}

#[derive(Debug, Clone)]
#[expect(clippy::struct_field_names)]
pub struct Linter {
    options: LintOptions,
    config: ConfigStore,
    #[cfg_attr(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))), expect(dead_code))]
    external_linter: Option<ExternalLinter>,
}

impl Linter {
    pub fn new(
        options: LintOptions,
        config: ConfigStore,
        external_linter: Option<ExternalLinter>,
    ) -> Self {
        Self { options, config, external_linter }
    }

    /// Set the kind of auto fixes to apply.
    #[must_use]
    pub fn with_fix(mut self, kind: FixKind) -> Self {
        self.options.fix = kind;
        self
    }

    #[must_use]
    pub fn with_report_unused_directives(mut self, report_config: Option<AllowWarnDeny>) -> Self {
        self.options.report_unused_directive = report_config;
        self
    }

    pub(crate) fn options(&self) -> &LintOptions {
        &self.options
    }

    /// Returns the number of rules that will are being used, unless there
    /// nested configurations in use, in which case it returns `None` since the
    /// number of rules depends on which file is being linted.
    pub fn number_of_rules(&self) -> Option<usize> {
        self.config.number_of_rules()
    }

    pub fn run<'a>(
        &self,
        path: &Path,
        semantic: Rc<Semantic<'a>>,
        module_record: Arc<ModuleRecord>,
    ) -> Vec<Message<'a>> {
        let ResolvedLinterState { rules, config, external_rules } = self.config.resolve(path);

        #[cfg(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))))]
        let _ = external_rules;

        let ctx_host =
            Rc::new(ContextHost::new(path, semantic, module_record, self.options, config));

        let rules = rules
            .iter()
            .filter(|(rule, _)| rule.should_run(&ctx_host))
            .map(|(rule, severity)| (rule, Rc::clone(&ctx_host).spawn(rule, *severity)));

        let semantic = ctx_host.semantic();

        let should_run_on_jest_node =
            ctx_host.plugins().has_test() && ctx_host.frameworks().is_test();

        if path.to_str().is_some_and(|str| str.ends_with(".d.ts")) {
            return ctx_host.take_diagnostics();
        }

        // IMPORTANT: We have two branches here for performance reasons:
        //
        // 1) Branch where we iterate over each node, then each rule
        // 2) Branch where we iterate over each rule, then each node
        //
        // When the number of nodes is relatively small, most of them can fit
        // in the cache and we can save iterating over the rules multiple times.
        // But for large files, the number of nodes can be so large that it
        // starts to not fit into the cache and pushes out other data, like the rules.
        // So we end up thrashing the cache with each rule iteration. In this case,
        // it's better to put rules in the inner loop, as the rules data is smaller
        // and is more likely to fit in the cache.
        //
        // The threshold here is chosen to balance between performance improvement
        // from not iterating over rules multiple times, but also ensuring that we
        // don't thrash the cache too much. Feel free to tweak based on benchmarking.
        //
        // See https://github.com/oxc-project/oxc/pull/6600 for more context.
        if semantic.nodes().len() > 200_000 {
            // Collect rules into a Vec so that we can iterate over the rules multiple times
            let rules = rules.collect::<Vec<_>>();

            for (rule, ctx) in &rules {
                rule.run_once(ctx);
            }

            for symbol in semantic.scoping().symbol_ids() {
                for (rule, ctx) in &rules {
                    rule.run_on_symbol(symbol, ctx);
                }
            }

            for node in semantic.nodes() {
                for (rule, ctx) in &rules {
                    rule.run(node, ctx);
                }
            }

            if should_run_on_jest_node {
                for jest_node in iter_possible_jest_call_node(semantic) {
                    for (rule, ctx) in &rules {
                        rule.run_on_jest_node(&jest_node, ctx);
                    }
                }
            }
        } else {
            for (rule, ref ctx) in rules {
                rule.run_once(ctx);

                for symbol in semantic.scoping().symbol_ids() {
                    rule.run_on_symbol(symbol, ctx);
                }

                for node in semantic.nodes() {
                    rule.run(node, ctx);
                }

                if should_run_on_jest_node {
                    for jest_node in iter_possible_jest_call_node(semantic) {
                        rule.run_on_jest_node(&jest_node, ctx);
                    }
                }
            }
        }

        #[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
        self.run_external_rules(&external_rules, path, &ctx_host);

        if let Some(severity) = self.options.report_unused_directive {
            if severity.is_warn_deny() {
                ctx_host.report_unused_directives(severity.into());
            }
        }

        ctx_host.take_diagnostics()
    }

    #[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
    fn run_external_rules(
        &self,
        external_rules: &[(ExternalRuleId, AllowWarnDeny)],
        path: &Path,
        ctx_host: &ContextHost,
    ) {
        use oxc_diagnostics::OxcDiagnostic;
        use oxc_span::Span;

        use crate::fixer::PossibleFixes;

        if external_rules.is_empty() {
            return;
        }

        // `external_linter` always exists when `oxlint2` feature is enabled
        let external_linter = self.external_linter.as_ref().unwrap();

        let result = (external_linter.run)(
            path.to_str().unwrap().to_string(),
            external_rules.iter().map(|(rule_id, _)| rule_id.raw()).collect(),
        );
        match result {
            Ok(diagnostics) => {
                for diagnostic in diagnostics {
                    match self.config.resolve_plugin_rule_names(diagnostic.external_rule_id) {
                        Some((plugin_name, rule_name)) => {
                            ctx_host.push_diagnostic(Message::new(
                                // TODO: `error` isn't right, we need to get the severity from `external_rules`
                                OxcDiagnostic::error(diagnostic.message)
                                    .with_label(Span::new(diagnostic.loc.start, diagnostic.loc.end))
                                    .with_error_code(
                                        plugin_name.to_string(),
                                        rule_name.to_string(),
                                    ),
                                PossibleFixes::None,
                            ));
                        }
                        None => {
                            // TODO: report diagnostic, this should be unreachable
                            debug_assert!(false);
                        }
                    }
                }
            }
            Err(_err) => {
                // TODO: report diagnostic
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Oxlintrc;

    #[test]
    fn test_schema_json() {
        use std::fs;

        use project_root::get_project_root;
        let path = get_project_root().unwrap().join("npm/oxlint/configuration_schema.json");
        let schema = schemars::schema_for!(Oxlintrc);
        let json = serde_json::to_string_pretty(&schema).unwrap();
        let existing_json = fs::read_to_string(&path).unwrap_or_default();
        if existing_json.trim() != json.trim() {
            std::fs::write(&path, &json).unwrap();
        }
        insta::with_settings!({ prepend_module_to_snapshot => false }, {
            insta::assert_snapshot!(json);
        });
    }
}
