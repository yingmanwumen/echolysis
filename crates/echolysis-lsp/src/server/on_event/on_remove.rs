use std::{path::PathBuf, sync::Arc};

use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::server::Server;

impl Server {
    /// Handles the removal of files from the LSP server
    ///
    /// # Arguments
    /// * `paths` - Vector of file paths to be removed
    pub async fn on_remove(&self, paths: Vec<PathBuf>) {
        // Clear diagnostics for files being removed
        self.clear_diagnostic(&paths).await;

        // Group files by language for batch processing
        let mut lang_map = FxHashMap::default();
        for path in &paths {
            if let Some((_, lang)) = self.file_map.remove(path) {
                lang_map
                    .entry(lang)
                    .or_insert_with(Vec::new)
                    .push(path.clone());
            }
        }

        // Early return if no files need processing
        if lang_map.is_empty() {
            return;
        }

        // Log removal operations
        for (lang, files) in &lang_map {
            self.log_info(format!("Removing {} files: {:?}", lang, files))
                .await;
        }

        lang_map.into_par_iter().for_each(|(lang, files)| {
            if let Some(engine) = self.router.get_engine_by_language_id(lang) {
                engine.remove_many(
                    files
                        .into_iter()
                        .filter_map(|file| {
                            let file = file.to_str()?.to_string();
                            Some(Arc::new(file))
                        })
                        .collect(),
                );
            }
        });
        self.push_diagnostic().await;
    }

    pub async fn on_remove_all(&self) {
        self.router
            .engines()
            .iter()
            .for_each(|engine| engine.remove_all());
        self.file_map.clear();
        self.push_diagnostic().await;
    }
}
