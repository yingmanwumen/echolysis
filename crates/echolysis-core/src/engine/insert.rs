use std::{path::PathBuf, sync::Arc};

use super::{indexed_tree::IndexedTree, Engine};
use rayon::prelude::*;

impl Engine {
    pub fn insert_many(
        &self,
        sources: impl IntoParallelIterator<Item = (Arc<PathBuf>, Arc<String>)>,
    ) {
        sources.into_par_iter().for_each(|(path, source)| {
            self.insert(path, source);
        });
    }

    pub fn insert(&self, path: Arc<PathBuf>, source: Arc<String>) -> Option<()> {
        let mut parser = self.language.parser();
        let query = self.language.query();
        let tree = match parser.parse(source.as_str(), None) {
            Some(tree) => tree,
            None => {
                self.remove(path);
                return None;
            }
        };
        let indexed_tree = IndexedTree::new(path.clone(), source, tree, query);

        match self.tree_map.entry(path) {
            dashmap::Entry::Occupied(mut entry) => {
                let old = entry.get();
                self.remove_merkle_hashes(old.root_node());
                self.merkle_hash(&indexed_tree);
                entry.insert(indexed_tree);
            }
            dashmap::Entry::Vacant(entry) => {
                self.merkle_hash(&indexed_tree);
                entry.insert(indexed_tree);
            }
        }

        Some(())
    }
}
