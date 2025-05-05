use std::path::{Path, PathBuf};
use std::sync::Arc;

use globset::Glob;
use ignore::gitignore::Gitignore;
use log::warn;
use rustc_hash::{FxBuildHasher, FxHashMap};
use tower_lsp_server::lsp_types::Uri;

use oxc_linter::{ConfigStore, ConfigStoreBuilder, LintOptions, Linter, Oxlintrc};
use tower_lsp_server::UriExt;

use crate::linter::{
    error_with_position::DiagnosticReport,
    isolated_lint_handler::{IsolatedLintHandler, IsolatedLintHandlerOptions},
};
use crate::{ConcurrentHashMap, Options};

use super::config_walker::ConfigWalker;

#[derive(Clone)]
pub struct ServerLinter {
    isolated_linter: Arc<IsolatedLintHandler>,
}

impl ServerLinter {
    /// Searches inside root_uri recursively for the default oxlint config files
    /// and insert them inside the nested configuration
    pub fn create_nested_configs(
        root_uri: &Uri,
        options: &Options,
    ) -> ConcurrentHashMap<PathBuf, ConfigStore> {
        // nested config is disabled, no need to search for configs
        if !options.use_nested_configs() {
            return ConcurrentHashMap::default();
        }

        let root_path = root_uri.to_file_path().expect("Failed to convert URI to file path");

        let paths = ConfigWalker::new(&root_path).paths();
        let nested_configs =
            ConcurrentHashMap::with_capacity_and_hasher(paths.capacity(), FxBuildHasher);

        for path in paths {
            let file_path = Path::new(&path);
            let Some(dir_path) = file_path.parent() else {
                continue;
            };

            let Ok(oxlintrc) = Oxlintrc::from_file(file_path) else {
                warn!("Skipping invalid config file: {}", file_path.display());
                continue;
            };
            let Ok(config_store_builder) = ConfigStoreBuilder::from_oxlintrc(false, oxlintrc)
            else {
                warn!("Skipping config (builder failed): {}", file_path.display());
                continue;
            };
            let Ok(config_store) = config_store_builder.build() else {
                warn!("Skipping config (builder failed): {}", file_path.display());
                continue;
            };

            nested_configs.pin().insert(dir_path.to_path_buf(), config_store);
        }

        nested_configs
    }

    pub fn create_ignore_glob(root_uri: &Uri, oxlintrc: &Oxlintrc) -> Vec<Gitignore> {
        let mut builder = globset::GlobSetBuilder::new();
        // Collecting all ignore files
        builder.add(Glob::new("**/.eslintignore").unwrap());
        builder.add(Glob::new("**/.gitignore").unwrap());

        let ignore_file_glob_set = builder.build().unwrap();

        let walk = ignore::WalkBuilder::new(root_uri.to_file_path().unwrap())
            .ignore(true)
            .hidden(false)
            .git_global(false)
            .build()
            .flatten();

        let mut gitignore_globs = vec![];
        for entry in walk {
            let ignore_file_path = entry.path();
            if !ignore_file_glob_set.is_match(ignore_file_path) {
                continue;
            }

            if let Some(ignore_file_dir) = ignore_file_path.parent() {
                let mut builder = ignore::gitignore::GitignoreBuilder::new(ignore_file_dir);
                builder.add(ignore_file_path);
                if let Ok(gitignore) = builder.build() {
                    gitignore_globs.push(gitignore);
                }
            }
        }

        if oxlintrc.ignore_patterns.is_empty() {
            return gitignore_globs;
        }

        let Some(oxlintrc_dir) = oxlintrc.path.parent() else {
            warn!("Oxlintrc path has no parent, skipping inline ignore patterns");
            return gitignore_globs;
        };

        let mut builder = ignore::gitignore::GitignoreBuilder::new(oxlintrc_dir);
        for entry in &oxlintrc.ignore_patterns {
            builder.add_line(None, entry).expect("Failed to add ignore line");
        }
        gitignore_globs.push(builder.build().unwrap());
        gitignore_globs
    }

    pub fn create_server_linter(
        root_uri: &Uri,
        options: &Options,
        nested_configs: &ConcurrentHashMap<PathBuf, ConfigStore>,
    ) -> (Self, Oxlintrc) {
        let root_path = root_uri.to_file_path().unwrap();
        let relative_config_path = options.config_path.clone();
        let oxlintrc = if relative_config_path.is_some() {
            let config = root_path.join(relative_config_path.unwrap());
            if config.try_exists().expect("Could not get fs metadata for config") {
                if let Ok(oxlintrc) = Oxlintrc::from_file(&config) {
                    oxlintrc
                } else {
                    warn!("Failed to initialize oxlintrc config: {}", config.to_string_lossy());
                    Oxlintrc::default()
                }
            } else {
                warn!(
                    "Config file not found: {}, fallback to default config",
                    config.to_string_lossy()
                );
                Oxlintrc::default()
            }
        } else {
            Oxlintrc::default()
        };

        // clone because we are returning it for ignore builder
        let config_builder =
            ConfigStoreBuilder::from_oxlintrc(false, oxlintrc.clone()).unwrap_or_default();

        // TODO(refactor): pull this into a shared function, because in oxlint we have the same functionality.
        let use_nested_config = options.use_nested_configs();

        let use_cross_module = if use_nested_config {
            nested_configs.pin().values().any(|config| config.plugins().has_import())
        } else {
            config_builder.plugins().has_import()
        };

        let config_store = config_builder.build().expect("Failed to build config store");

        let lint_options = LintOptions { fix: options.fix_kind(), ..Default::default() };

        let linter = if use_nested_config {
            let nested_configs = nested_configs.pin();
            let nested_configs_copy: FxHashMap<PathBuf, ConfigStore> = nested_configs
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<FxHashMap<_, _>>();

            Linter::new_with_nested_configs(lint_options, config_store, nested_configs_copy)
        } else {
            Linter::new(lint_options, config_store)
        };

        let server_linter = ServerLinter::new_with_linter(
            linter,
            IsolatedLintHandlerOptions { use_cross_module, root_path: root_path.to_path_buf() },
        );

        (server_linter, oxlintrc)
    }

    fn new_with_linter(linter: Linter, options: IsolatedLintHandlerOptions) -> Self {
        let isolated_linter = Arc::new(IsolatedLintHandler::new(linter, options));

        Self { isolated_linter }
    }

    pub fn run_single(&self, uri: &Uri, content: Option<String>) -> Option<Vec<DiagnosticReport>> {
        self.isolated_linter.run_single(uri, content)
    }
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use rustc_hash::FxHashMap;
    use tower_lsp_server::lsp_types::Uri;

    use crate::{
        Options,
        linter::server_linter::ServerLinter,
        tester::{Tester, get_file_uri},
    };

    #[test]
    fn test_create_nested_configs_with_disabled_nested_configs() {
        let mut flags = FxHashMap::default();
        flags.insert("disable_nested_configs".to_string(), "true".to_string());

        let configs = ServerLinter::create_nested_configs(
            &Uri::from_str("file:///root/").unwrap(),
            &Options { flags, ..Options::default() },
        );

        assert!(configs.is_empty());
    }

    #[test]
    fn test_create_nested_configs() {
        let configs = ServerLinter::create_nested_configs(
            &get_file_uri("fixtures/linter/init_nested_configs"),
            &Options::default(),
        );
        let configs = configs.pin();
        let mut configs_dirs = configs.keys().collect::<Vec<&PathBuf>>();
        // sorting the key because for consistent tests results
        configs_dirs.sort();

        assert!(configs_dirs.len() == 3);
        assert!(configs_dirs[2].ends_with("deep2"));
        assert!(configs_dirs[1].ends_with("deep1"));
        assert!(configs_dirs[0].ends_with("init_nested_configs"));
    }

    #[test]
    fn test_no_errors() {
        Tester::new("fixtures/linter/no_errors", None)
            .test_and_snapshot_single_file("hello_world.js");
    }

    #[test]
    fn test_no_console() {
        Tester::new("fixtures/linter/deny_no_console", None)
            .test_and_snapshot_single_file("hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        Tester::new("fixtures/linter/issue_9958", None).test_and_snapshot_single_file("issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        Tester::new("fixtures/linter/regexp_feature", None)
            .test_and_snapshot_single_file("index.ts");
    }

    #[test]
    fn test_frameworks() {
        Tester::new("fixtures/linter/astro", None).test_and_snapshot_single_file("debugger.astro");
        Tester::new("fixtures/linter/vue", None).test_and_snapshot_single_file("debugger.vue");
        Tester::new("fixtures/linter/svelte", None)
            .test_and_snapshot_single_file("debugger.svelte");
        // ToDo: fix Tester to work only with Uris and do not access the file system
        // Tester::new("fixtures/linter/nextjs").test_and_snapshot_single_file("%5B%5B..rest%5D%5D/debugger.ts");
    }

    #[test]
    fn test_invalid_syntax_file() {
        Tester::new("fixtures/linter/invalid_syntax", None)
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_debugger() {
        Tester::new("fixtures/linter/cross_module", None)
            .test_and_snapshot_single_file("debugger.ts");
    }

    #[test]
    fn test_cross_module_no_cycle() {
        Tester::new("fixtures/linter/cross_module", None).test_and_snapshot_single_file("dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_nested_config() {
        Tester::new("fixtures/linter/cross_module_nested_config", None)
            .test_and_snapshot_single_file("dep-a.ts");
        Tester::new("fixtures/linter/cross_module_nested_config", None)
            .test_and_snapshot_single_file("folder/folder-dep-a.ts");
    }

    #[test]
    fn test_cross_module_no_cycle_extended_config() {
        Tester::new("fixtures/linter/cross_module_extended_config", None)
            .test_and_snapshot_single_file("dep-a.ts");
    }
}
