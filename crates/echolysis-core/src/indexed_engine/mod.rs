mod insert;
mod merkle_hash;
mod remove;

use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;
use rayon::iter::IntoParallelIterator;
use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::{
    indexed_tree::{IndexedNode, IndexedTree},
    languages::SupportedLanguage,
};

pub struct IndexedEngine {
    language: SupportedLanguage,
    tree_map: DashMap<Arc<PathBuf>, IndexedTree, ahash::RandomState>,
    hash_map: DashMap<u64, FxHashSet<Arc<IndexedNode>>, FxBuildHasher>,
}

impl IndexedEngine {
    pub fn new(
        language: SupportedLanguage,
        sources: Option<impl IntoParallelIterator<Item = (Arc<PathBuf>, Arc<String>)>>,
    ) -> Self {
        let engine = Self {
            language,
            tree_map: DashMap::with_hasher(ahash::RandomState::default()),
            hash_map: DashMap::with_hasher(FxBuildHasher),
        };
        if let Some(sources) = sources {
            engine.insert_many(sources.into_par_iter());
        }
        engine
    }
}
