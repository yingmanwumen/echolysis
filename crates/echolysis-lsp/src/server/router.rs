#![allow(unused)]

use std::{path::Path, sync::Arc};

use echolysis_core::{
    languages::SupportedLanguage,
    utils::{hash::ADashMap, language_id::path_to_language_id},
    Engine,
};

/// Router manages language-specific engines for code analysis
///
/// The Router is responsible for:
/// - Managing language-specific analysis engines
/// - Mapping file paths to appropriate language engines
/// - Determining language support for files
pub struct Router {
    /// Maps language IDs to their corresponding analysis engines
    engines: ADashMap<String, Arc<Engine>>,
}

impl Router {
    /// Creates a new Router instance
    pub fn new() -> Self {
        Self {
            engines: ADashMap::default(),
        }
    }

    pub fn engines(&self) -> &ADashMap<String, Arc<Engine>> {
        &self.engines
    }

    /// Gets or creates an engine instance for the given file path
    ///
    /// # Arguments
    /// * `path` - File path to get engine for
    pub fn get_engine_by_path(&self, path: &Path) -> Option<Arc<Engine>> {
        let language_id = path_to_language_id(path).to_string();
        self.get_engine_by_language_id(&language_id)
    }

    /// Gets or creates an engine instance for the given language ID
    ///
    /// # Arguments
    /// * `language_id` - Language identifier (e.g. "rust", "python")
    ///
    /// # Returns
    /// * `Some(Engine)` if language is supported
    /// * `None` if language is not supported
    pub fn get_engine_by_language_id(&self, language_id: &str) -> Option<Arc<Engine>> {
        let language = SupportedLanguage::from_language_id(language_id)?;
        Some(
            self.engines
                .entry(language_id.to_string())
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
