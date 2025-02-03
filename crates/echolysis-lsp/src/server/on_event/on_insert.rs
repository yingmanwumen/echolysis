use std::sync::Arc;

use echolysis_core::{languages::SupportedLanguage, utils::language_id::get_language_id_by_path};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tower_lsp::lsp_types;

use crate::server::Server;

impl Server {
    pub async fn on_insert(&self, sources: &[(lsp_types::Url, Option<Arc<String>>)]) {
        // Clear diagnostics for modified files first
        self.clear_diagnostic(
            &sources.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
            None,
        )
        .await;

        // Group files by language ID
        let mut lang_map = FxHashMap::default();
        let supported = SupportedLanguage::supported_languages();
        for (uri, source) in sources {
            if let Ok(path) = uri.to_file_path() {
                let language_id = get_language_id_by_path(&path);
                if !supported.contains(&language_id) {
                    continue;
                }
                // Group files by language and store language association
                lang_map
                    .entry(language_id)
                    .or_insert_with(Vec::new)
                    .push((path.clone(), source));
                self.file_map.insert(uri.clone(), language_id);
            }
        }

        // Early return if no supported files found
        if lang_map.is_empty() {
            return;
        }

        lang_map.into_iter().for_each(|(lang, sources)| {
            if let Some(engine) = self.router.get_engine_by_language_id(lang) {
                engine.insert_many(
                    sources
                        .into_par_iter()
                        .filter_map(|(path, source)| {
                            let source = source
                                .clone()
                                .unwrap_or(Arc::new(std::fs::read_to_string(&path).ok()?));
                            Some((Arc::new(path), source))
                        })
                        .collect::<Vec<_>>(),
                );
            }
        });

        self.push_diagnostic().await;
    }
}
