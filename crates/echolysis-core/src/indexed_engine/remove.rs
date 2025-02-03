use std::{path::PathBuf, sync::Arc};

use dashmap::Entry;
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use crate::{languages::NodeTaste, utils::hash::merge_structure_hash};

use super::{indexed_node::IndexedNode, IndexedEngine};

impl IndexedEngine {
    pub fn remove_many(&self, paths: impl ParallelIterator<Item = Arc<PathBuf>>) {
        paths.for_each(|path| {
            self.remove(path);
        })
    }

    pub fn remove(&self, path: Arc<PathBuf>) {
        if let Entry::Occupied(entry) = self.tree_map.entry(path) {
            let tree = entry.get();
            // First remove all hashes related to this tree
            self.remove_merkle_hashes(tree.root_node());
            // Then remove the tree itself
            entry.remove();
        }
    }

    fn remove_merkle_hashes(&self, node: Arc<IndexedNode>) -> u64 {
        if node.is_extra_or_missing_or_error()
            || self.language.indexed_node_taste(&node) == NodeTaste::Ignored
        {
            return 0;
        }
        if node.children().is_empty() {
            return self.language.simple_hash_indexed_node(&node);
        }
        let mut combined_hash: u64 = 0;
        for child in node.children() {
            combined_hash =
                merge_structure_hash(combined_hash, self.remove_merkle_hashes(child.clone()));
        }
        if self.language.indexed_node_taste(&node) == NodeTaste::Interesting
            && self.language.indexed_node_cognitive_complexity(&node)
                >= self.language.complexity_threshold()
        {
            if let Some(mut set) = self.hash_map.get_mut(&combined_hash) {
                set.remove(&node);
                // If the set becomes empty, remove it from the hash_map
                if set.is_empty() {
                    drop(set); // 先释放可变引用
                    self.hash_map.remove(&combined_hash);
                }
            }
        }
        combined_hash
    }
}
