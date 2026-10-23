//! End-to-end check of every fork fixer on real code.
//!
//! Lints the benchmark files cached in `target/` (downloaded by `cargo lint-timings`) with each
//! covered rule on its own, applies the fixes until nothing changes, and checks that no pass adds
//! parse or semantic errors and that fixing converges. The runs are parallel (about 20s in a debug
//! build). Run it with:
//!
//! ```text
//! cargo test -p oxc_linter --lib fork_fixes::corpus -- --ignored --nocapture
//! ```
//!
//! Set `FORK_FIX_RULE=<rule-name>` to check a single fixer.

use std::{fmt::Write as _, path::Path, sync::Arc, time::Instant};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;

use super::{ENTRIES, Entry};
use crate::{
    AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions,
    LintPlugins, Linter, ModuleRecord,
    external_plugin_store::ExternalPluginStore,
    fixer::{FixKind, Fixer, Message},
    options::LintOptions,
    rules::RULES,
};

const FILES: &[&str] = &[
    "checker.ts",
    "binder.ts",
    "antd.js",
    "pdf.mjs",
    "App.tsx",
    "kitchen-sink.tsx",
    "RadixUIAdoptionSection.jsx",
];

const MAX_PASSES: usize = 10;

fn build_linter(kind: FixKind, entry: &Entry) -> Linter {
    let rule = RULES
        .iter()
        .find(|rule| rule.plugin_name() == entry.plugin && rule.name() == entry.rule)
        .unwrap()
        .from_configuration(serde_json::Value::Null)
        .unwrap();
    let plugins = LintPlugins::try_from(entry.plugin).unwrap();
    let mut external_plugin_store = ExternalPluginStore::default();
    let builder = ConfigStoreBuilder::empty()
        .with_builtin_plugins(plugins)
        .with_rule(rule, AllowWarnDeny::Warn);
    let config = builder.build(&mut external_plugin_store).unwrap();
    Linter::new(
        LintOptions::default(),
        ConfigStore::new(config, FxHashMap::default(), external_plugin_store),
        None,
    )
    .with_fix(kind)
}

fn lint(linter: &Linter, path: &Path, source: &str, source_type: SourceType) -> Vec<Message> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new_linter().build(&ret.program).semantic;
    let module_record = Arc::new(ModuleRecord::new(path, &ret.module_record, &semantic));
    linter.run(
        path,
        vec![ContextSubHost::new(semantic, module_record, 0, ContextSubHostOptions::default())],
        &allocator,
    )
}

/// Parse errors plus semantic syntax errors (redeclarations, invalid assignment targets, ...).
fn errors(source: &str, source_type: SourceType) -> Vec<String> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new().with_check_syntax_error(true).build(&ret.program);
    ret.diagnostics
        .errors()
        .chain(semantic.diagnostics.errors())
        .map(|error| error.message.to_string())
        .collect()
}

#[test]
#[ignore = "slow end-to-end check; run explicitly"]
#[expect(clippy::print_stdout, reason = "the corpus check reports what each fixer did")]
fn corpus() {
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    let only = std::env::var("FORK_FIX_RULE").ok();
    let files: Vec<_> = FILES
        .iter()
        .filter_map(|file| {
            let path = target.join(file);
            let source = std::fs::read_to_string(&path).ok()?;
            let source_type = SourceType::from_path(&path).unwrap();
            let baseline_errors = errors(&source, source_type);
            Some((path, source, source_type, baseline_errors))
        })
        .collect();
    assert!(
        !files.is_empty(),
        "no corpus files found in {}; run `cargo lint-timings` once",
        target.display()
    );

    // Each fixer runs on its own: some rules contradict each other (`no-undefined` writes
    // `void 0`, `no-void` writes `undefined`), and a failure is then attributed to one rule. The
    // runs are independent, so every (fixer, fix level, file) runs in parallel. The safe-fix-only
    // level is skipped for fixers that emit no safe fix, because `apply` never calls them there.
    let jobs: Vec<(&Entry, FixKind, &CorpusFile)> = ENTRIES
        .iter()
        .filter(|entry| only.as_deref().is_none_or(|rule| rule == entry.rule))
        .flat_map(|entry| {
            let safe = entry.kinds.contains(&FixKind::SafeFix).then_some(FixKind::SafeFix);
            safe.into_iter().chain([FixKind::All]).map(move |kind| (entry, kind))
        })
        .flat_map(|(entry, kind)| files.iter().map(move |file| (entry, kind, file)))
        .collect();
    let start = Instant::now();
    let results: Vec<Result<usize, String>> =
        jobs.par_iter().map(|&(entry, kind, file)| fix_file(entry, kind, file)).collect();

    // Fix counts per fixer and level; jobs of one pair are adjacent.
    let mut totals: Vec<(&str, FixKind, usize)> = vec![];
    let mut failures = vec![];
    for (&(entry, kind, _), result) in jobs.iter().zip(results) {
        match result {
            Ok(fixed) => match totals.last_mut() {
                Some((rule, last_kind, total)) if *rule == entry.rule && *last_kind == kind => {
                    *total += fixed;
                }
                _ => totals.push((entry.rule, kind, fixed)),
            },
            Err(failure) => failures.push(failure),
        }
    }
    let mut report = String::new();
    for (rule, kind, fixed) in totals {
        writeln!(report, "{rule:<36} {kind:?}: {fixed} fixes").unwrap();
    }
    writeln!(report, "{} runs in {:?}", jobs.len(), start.elapsed()).unwrap();
    println!("{report}");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

type CorpusFile = (std::path::PathBuf, String, SourceType, Vec<String>);

/// Fixes one corpus file with one fixer at one fix level until nothing changes. Returns the number
/// of diagnostics that carried a fix, or why the file gained errors or did not converge.
fn fix_file(entry: &Entry, kind: FixKind, file: &CorpusFile) -> Result<usize, String> {
    let (path, original, source_type, baseline_errors) = file;
    let name = path.file_name().unwrap().to_string_lossy();
    let linter = build_linter(kind, entry);
    let mut fixed = 0;
    let mut text = original.clone();
    for pass in 1..=MAX_PASSES {
        let messages = lint(&linter, path, &text, *source_type);
        fixed += messages.iter().filter(|message| !message.fixes.is_empty()).count();
        let result = Fixer::new(&text, messages, Some(*source_type)).fix();
        if !result.fixed {
            return Ok(fixed);
        }
        text = result.fixed_code.into_owned();

        let new_errors: Vec<_> = errors(&text, *source_type)
            .into_iter()
            .filter(|error| !baseline_errors.contains(error))
            .collect();
        if !new_errors.is_empty() {
            return Err(format!(
                "{}: {name} ({kind:?}, pass {pass}): fixes introduced errors: {new_errors:#?}",
                entry.rule
            ));
        }
    }
    Err(format!("{}: {name} ({kind:?}): still fixing after {MAX_PASSES} passes", entry.rule))
}
