use std::{path::PathBuf, sync::Arc};

use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::server::Server;

impl Server {
    pub async fn on_remove(&self, paths: Vec<PathBuf>) {
        let mut lang_map = FxHashMap::default();
        for path in &paths {
            if let Some((_, lang)) = self.file_map.remove(path) {
                lang_map
                    .entry(lang)
                    .or_insert_with(Vec::new)
                    .push(path.clone());
            }
        }
        if lang_map.is_empty() {
            return;
        }
        for (lang, files) in &lang_map {
            self.log_info(format!("remove {} files: {:?}", lang, files))
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
