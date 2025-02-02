use std::{path::PathBuf, sync::Arc};

use echolysis_core::{languages::SupportedLanguage, utils::language_id::path_to_language_id};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::server::Server;

impl Server {
    /// Handles insertion/modification of files in the LSP server
    ///
    /// # Arguments
    /// * `files` - Vector of (file path, optional source content) pairs to process
    ///
    /// This function:
    /// 1. Clears existing diagnostics for modified files
    /// 2. Groups files by language
    /// 3. Updates the file map with language associations
    /// 4. Processes file contents in parallel using the appropriate language engines
    pub async fn on_insert(&self, files: Vec<(PathBuf, Option<&str>)>) {
        // Clear diagnostics for modified files first
        self.clear_diagnostic(&files.iter().map(|x| x.0.clone()).collect::<Vec<_>>())
            .await;

        // Group files by language ID
        let mut lang_map = FxHashMap::default();
        let supported = SupportedLanguage::supported_languages();
        for (file, source) in files {
            let language_id = path_to_language_id(&file);
            if !supported.contains(&language_id) {
                continue;
            }
            // Group files by language and store language association
            lang_map
                .entry(language_id)
                .or_insert_with(Vec::new)
                .push((file.clone(), source));
            self.file_map.insert(file, language_id);
        }

        // Early return if no supported files found
        if lang_map.is_empty() {
            return;
        }

        for (lang, files) in &lang_map {
            let paths: Vec<_> = files.iter().map(|x| x.0.clone()).collect();
            self.log_info(format!("insert {} files: {:?}", lang, paths))
                .await;
        }

        lang_map.into_par_iter().for_each(|(lang, files)| {
            if let Some(engine) = self.router.get_engine_by_language_id(lang) {
                engine.insert_many(
                    files
                        .into_iter()
                        .filter_map(|(path, source)| {
                            let source = source
                                .map(|x| x.to_string())
                                .unwrap_or(std::fs::read_to_string(&path).ok()?);
                            let file = path.to_str()?.to_string();
                            Some((Arc::new(file), source.to_string()))
                        })
                        .collect(),
                );
            }
        });

        self.push_diagnostic().await;
    }
}
