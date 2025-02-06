#![allow(unused)]

use std::{path::Path, sync::Arc};

use dashmap::DashMap;
use echolysis_core::{
    engine::Engine, languages::SupportedLanguage, utils::language_id::get_language_id_by_path,
};

pub struct Router {
    // K: language_id, V: Engine
    engines: DashMap<String, Arc<Engine>, ahash::RandomState>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            engines: DashMap::with_hasher(ahash::RandomState::default()),
        }
    }

    pub fn engines(&self) -> &DashMap<String, Arc<Engine>, ahash::RandomState> {
        &self.engines
    }

    pub fn get_engine_by_path(&self, path: &Path) -> Option<Arc<Engine>> {
        let language_id = get_language_id_by_path(path).to_string();
        self.get_engine_by_language_id(&language_id)
    }

    pub fn get_engine_by_language_id(&self, language_id: &str) -> Option<Arc<Engine>> {
        let language = SupportedLanguage::from_language_id(language_id)?;
        Some(
            self.engines
                .entry(language_id.to_string())
                .or_insert_with(|| Arc::new(Engine::new(language)))
                .value()
                .clone(),
        )
    }

    pub fn remove_engine(&self, language_id: &str) {
        self.engines.remove(language_id);
    }

    pub fn clear(&self) {
        self.engines.clear();
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
