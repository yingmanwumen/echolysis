use std::{path::PathBuf, sync::Arc};

use rustc_hash::FxHashMap;
use tower_lsp::lsp_types;

use crate::server::Server;

impl Server {
    pub async fn on_remove(&self, uris: &[lsp_types::Url]) {
        // Clear diagnostics for files being removed
        self.clear_diagnostic(uris, None).await;

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

    pub async fn on_remove_all(&self) {
        self.file_map.clear();
        self.router
            .engines()
            .iter()
            .for_each(|engine| engine.remove_all());
        self.push_diagnostic().await;
    }
}
