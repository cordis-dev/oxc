#![expect(clippy::print_stderr)]
mod doc_page;
mod html;
mod table;

use std::{
    borrow::Cow,
    env, fs,
    path::{Path, PathBuf},
};

use doc_page::Context;
use html::HtmlWriter;
use oxc_linter::{Oxlintrc, table::RuleTable};
use pico_args::Arguments;
use schemars::{SchemaGenerator, r#gen::SchemaSettings};
use table::render_rules_table;

const HELP: &str = "
usage: linter-rules [args]

Arguments:
    -t,--table <path>     Path to file where rule markdown table will be saved.
    -r,--rule-docs <path> Path to directory where rule doc pages will be saved.
                          A directory will be created if one doesn't exist.
    --git-ref <ref>       Git commit, branch, or tag to be used in the generated links.
                          If not supplied, `main` will be used.
    -h,--help             Show this help message.

";

/// `print_rules`
///
/// `cargo run -p website linter-rules --table
/// /path/to/oxc/oxc-project.github.io/src/docs/guide/usage/linter/generated-rules.md
/// --rule-docs /path/to/oxc/oxc-project.github.io/src/docs/guide/usage/linter/rules
/// --git-ref dc9dc03872101c15b0d02f05ce45705565665829
/// `
/// <https://oxc.rs/docs/guide/usage/linter/rules.html>
pub fn print_rules(mut args: Arguments) {
    let pwd = PathBuf::from(env::var("PWD").unwrap());
    if args.contains(["-h", "--help"]) {
        println!("{HELP}");
        return;
    }

    let git_ref: Option<String> = args.opt_value_from_str("--git-ref").unwrap();
    let table_path = args.opt_value_from_str::<_, PathBuf>(["-t", "--table"]).unwrap();
    let rules_dir = args.opt_value_from_str::<_, PathBuf>(["-r", "--rule-docs"]).unwrap();

    let prefix =
        rules_dir.as_ref().and_then(|p| p.as_os_str().to_str()).map_or(Cow::Borrowed(""), |p| {
            if p.contains("src/docs") {
                let split = p.split("src/docs").collect::<Vec<_>>();
                assert!(split.len() > 1);
                Cow::Owned("/docs".to_string() + split.last().unwrap())
            } else {
                Cow::Borrowed(p)
            }
        });

    let mut generator = SchemaGenerator::new(SchemaSettings::default());
    let table = RuleTable::new(Some(&mut generator));

    if let Some(table_path) = table_path {
        let table_path = pwd.join(table_path).canonicalize().unwrap();

        eprintln!("Rendering rules table...");
        let rules_table = render_rules_table(&table, prefix.as_ref());
        fs::write(table_path, rules_table).unwrap();
    }

    if let Some(rules_dir) = &rules_dir {
        eprintln!("Rendering rule doc pages...");
        let rules_dir = pwd.join(rules_dir);
        if !rules_dir.exists() {
            fs::create_dir_all(&rules_dir).unwrap();
        }
        let rules_dir = rules_dir.canonicalize().unwrap();
        assert!(
            !rules_dir.is_file(),
            "Cannot write rule docs to a file. Please specify a directory."
        );
        write_rule_doc_pages(generator, &table, &rules_dir);
        write_version_data(&rules_dir, git_ref.unwrap_or("main".to_string()).as_str());
    }

    eprintln!(
        "Done. Written rules to {}",
        rules_dir.as_ref().map_or_else(|| "none".to_string(), |p| format!("{}", p.display()))
    );
}

fn write_rule_doc_pages(g: SchemaGenerator, table: &RuleTable, outdir: &Path) {
    let mut ctx = Context::new::<Oxlintrc>(g);
    for rule in table.sections.iter().flat_map(|section| &section.rows) {
        let plugin_path = outdir.join(&rule.plugin);
        fs::create_dir_all(&plugin_path).unwrap();
        let page_path = plugin_path.join(format!("{}.md", rule.name));
        if page_path.exists() {
            fs::remove_file(&page_path).unwrap();
        }
        let docs = ctx.render_rule_docs_page(rule).unwrap();
        fs::write(&page_path, docs).unwrap();
    }
}

fn write_version_data(outdir: &Path, git_ref: &str) {
    let data = format!(r#"export default {{ load() {{ return "{git_ref}" }} }} "#);
    fs::write(outdir.join("version.data.js"), data).unwrap();
}
