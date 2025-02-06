use std::{path::PathBuf, sync::Arc};

use echolysis_core::utils::language_id::get_language_id_by_path;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use tower_lsp::lsp_types;

use crate::server::{utils::should_ignore, Server};

type LangGroup<'a> = FxHashMap<&'static str, Vec<(PathBuf, &'a Option<Arc<String>>)>>;

impl Server {
    fn filter_and_group_paths<'a>(
        &self,
        sources: &'a [(lsp_types::Url, Option<Arc<String>>)],
    ) -> LangGroup<'a> {
        // Group files by language ID
        let mut lang_map = FxHashMap::default();
        for (uri, source) in sources {
            if let Ok(path) = uri.to_file_path() {
                if should_ignore(&path) {
                    continue;
                }
                let language_id = get_language_id_by_path(&path);
                // Group files by language and store language association
                lang_map
                    .entry(language_id)
                    .or_insert_with(Vec::new)
                    .push((path.clone(), source));
                self.file_map.insert(uri.clone(), language_id);
            }
        }
        lang_map
    }

    pub async fn on_insert(&self, sources: &[(lsp_types::Url, Option<Arc<String>>)]) {
        if self.is_stopped() {
            return;
        }

        // Clear diagnostics for modified files first
        self.clear_diagnostic(
            &sources.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
            None,
        )
        .await;

        let lang_map = self.filter_and_group_paths(sources);
        if lang_map.is_empty() {
            return;
        }

        lang_map.into_iter().for_each(|(lang, sources)| {
            if let Some(engine) = self.router.get_engine_by_language_id(lang) {
                let sources = sources
                    .into_par_iter()
                    .filter_map(|(path, source)| {
                        let source = source
                            .clone()
                            .unwrap_or(Arc::new(std::fs::read_to_string(&path).ok()?));
                        Some((Arc::new(path), source))
                    })
                    .collect::<Vec<_>>();
                engine.insert_many(sources);
            }
        });

        self.push_diagnostic().await;
    }
}
