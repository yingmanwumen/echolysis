#[allow(unused)]
pub mod indexed_node;
#[allow(unused)]
pub mod indexed_tree;

mod insert;
mod merkle_hash;
mod remove;

use std::{path::PathBuf, sync::Arc};

use dashmap::{DashMap, DashSet};
use indexed_node::IndexedNode;
use indexed_tree::IndexedTree;
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashSet};

use crate::languages::SupportedLanguage;

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

    pub fn detect_duplicates(&self) -> Vec<Vec<Arc<IndexedNode>>> {
        let mut children = DashSet::with_hasher(FxBuildHasher);
        self.hash_map.par_iter().for_each(|nodes| {
            if nodes.len() < 2 {
                return;
            }
            for node in nodes.value() {
                for child in node.children() {
                    children.insert(child.clone());
                }
            }
        });
        self.hash_map
            .par_iter()
            .filter_map(|nodes| {
                let group: Vec<_> = nodes
                    .iter()
                    .filter_map(|node| {
                        if children.contains(node) {
                            None
                        } else {
                            Some(node.clone())
                        }
                    })
                    .collect();
                if group.len() > 1 {
                    Some(group)
                } else {
                    None
                }
            })
            .collect()
    }
}
