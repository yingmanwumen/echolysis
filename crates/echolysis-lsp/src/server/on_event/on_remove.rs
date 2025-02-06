use std::{path::PathBuf, sync::Arc};

use rustc_hash::FxHashMap;
use tower_lsp::lsp_types;

use crate::server::Server;

impl Server {
    fn related_uris_of_duplicated(&self, uris: &[lsp_types::Url]) -> Option<Vec<lsp_types::Url>> {
        Some(
            self.duplicate_locations
                .lock()
                .iter()
                .find(|locations| locations.iter().any(|loc| uris.contains(&loc.uri)))?
                .iter()
                .map(|loc| loc.uri.clone())
                .collect::<Vec<_>>(),
        )
    }

    pub async fn on_remove(&self, uris: &[lsp_types::Url]) {
        if self.is_stopped() {
            return;
        }

        // Clear diagnostics for files being removed and their duplicates
        self.clear_diagnostic(
            &[
                uris,
                &self.related_uris_of_duplicated(uris).unwrap_or_default(),
            ]
            .concat(),
            None,
        )
        .await;

        // Group files by language for batch processing
        let mut lang_map: FxHashMap<&str, Vec<PathBuf>> = FxHashMap::default();
        for uri in uris {
            if let Some((_, lang)) = self.file_map.remove(uri) {
                if let Ok(path) = uri.to_file_path() {
                    lang_map.entry(lang).or_default().push(path);
                }
            }
        }

        // Early return if no files need processing
        if lang_map.is_empty() {
            return;
        }

        lang_map.into_iter().for_each(|(lang, paths)| {
            if let Some(engine) = self.router.get_engine_by_language_id(lang) {
                engine.remove_many(paths.into_iter().map(Arc::new).collect::<Vec<_>>());
            }
        });

        self.push_diagnostic().await;
    }
}
