use std::{fs, str::FromStr};

use quote::quote;
use serde::Deserialize;
use syn::Ident;

use oxc_tasks_common::project_root;
use oxc_transformer::EngineTargets;

#[derive(Debug, Deserialize)]
struct Item {
    name: String,
    es: String,
    targets: EngineTargets,
}

impl Item {
    fn es_name(&self) -> Ident {
        quote::format_ident!("{}{}", self.es, self.name)
    }
}

/// # Panics
pub fn generate() {
    let path = project_root().join("tasks/compat_data/data.json");
    let content = fs::read_to_string(path).unwrap();
    let items = serde_json::from_str::<Vec<Item>>(&content).unwrap();

    let es_features = items.iter().map(Item::es_name).collect::<Vec<_>>();

    let features = items.iter().map(|item| {
        let key = item.es_name();
        let es_version = u32::from_str(item.es.trim_start_matches("ES")).unwrap();
        let targets = item
            .targets
            .iter()
            .map(|(engine, version)| {
                let engine = quote::format_ident!("{engine:?}");
                let (a, b, c) = (version.0, version.1, version.2);
                quote! {
                    (#engine, Version(#a, #b, #c))
                }
            })
            .chain([quote! { (Es, Version(#es_version, 0, 0)) }]);
        quote! {
            (#key, EngineTargets::new(FxHashMap::from_iter([#(#targets),*])))
        }
    });

    let code = quote! {
        #![allow(clippy::enum_glob_use, clippy::match_same_arms)]
        use std::sync::OnceLock;

        use browserslist::Version;
        use rustc_hash::FxHashMap;

        use super::{Engine, EngineTargets};

        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub enum ESFeature {
            #(#es_features,)*
        }

        pub fn features() -> &'static FxHashMap<ESFeature, EngineTargets> {
            use ESFeature::*;
            use Engine::*;
            static FEATURES: OnceLock<FxHashMap<ESFeature, EngineTargets>> = OnceLock::new();
            FEATURES.get_or_init(|| {
                FxHashMap::from_iter([#(#features),*])
            })
        }
    };

    generate_file("crates/oxc_transformer/src/options/es_features.rs", code);
}

fn generate_file(file: &str, token_stream: proc_macro2::TokenStream) {
    let syntax_tree = syn::parse2(token_stream).unwrap();
    let code = format!(
        "// Auto generated by `tasks/compat_data/src/lib.rs`.\n{}",
        prettyplease::unparse(&syntax_tree)
    );
    fs::write(project_root().join(file), code).unwrap();
}
