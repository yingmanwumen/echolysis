pub mod indexed_node;
pub mod indexed_tree;

mod detect;
mod insert;
mod merkle_hash;
mod remove;

use std::{path::PathBuf, sync::Arc};

use dashmap::DashMap;
use indexed_node::{Id, IndexedNode};
use indexed_tree::IndexedTree;
use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::languages::SupportedLanguage;

pub struct Engine {
    language: SupportedLanguage,
    tree_map: DashMap<Arc<PathBuf>, IndexedTree, ahash::RandomState>,
    hash_map: DashMap<u64, FxHashSet<Arc<IndexedNode>>, FxBuildHasher>,
    node_hash_map: DashMap<Id, u64, FxBuildHasher>,
}

impl Engine {
    pub fn new(language: SupportedLanguage) -> Self {
        Self {
            language,
            tree_map: DashMap::with_hasher(ahash::RandomState::default()),
            hash_map: DashMap::with_hasher(FxBuildHasher),
            node_hash_map: DashMap::with_hasher(FxBuildHasher),
        }
    }
}
