#![allow(unused)]

use std::{path::Path, sync::Arc};

use echolysis_core::{
    languages::SupportedLanguage,
    utils::{hash::ADashMap, language_id::file_extension_to_language_id},
    Engine,
};

pub struct Router {
    engines: ADashMap<String, Arc<Engine>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            engines: ADashMap::default(),
        }
    }

    pub fn get_engine(&self, path: &Path) -> Option<Arc<Engine>> {
        let language_id = file_extension_to_language_id(path).to_string();
        let language = SupportedLanguage::from_language_id(&language_id)?;

        Some(
            self.engines
                .entry(language_id)
                .or_insert_with(|| Arc::new(Engine::new(language, None)))
                .value()
                .clone(),
        )
    }

    pub fn is_language_supported(language_id: &str) -> bool {
        SupportedLanguage::supported_languages().contains(&language_id)
    }

    pub fn remove_engine(&self, language_id: &str) {
        self.engines.remove(language_id);
    }

    pub fn supported_languages() -> Vec<&'static str> {
        SupportedLanguage::supported_languages()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
