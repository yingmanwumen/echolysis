use std::{path::PathBuf, sync::Arc};

use rayon::prelude::*;

use super::{indexed_node::IndexedNode, Engine};

impl Engine {
    pub fn remove_many(&self, paths: impl IntoParallelIterator<Item = Arc<PathBuf>>) {
        let trees_to_remove: Vec<_> = paths
            .into_par_iter()
            .filter_map(|path| self.tree_map.remove(&path).map(|(_, tree)| tree))
            .collect();

        trees_to_remove.into_par_iter().for_each(|tree| {
            self.remove_merkle_hashes(tree.root_node());
        });
    }

    pub fn remove_all(&self) {
        self.remove_many(
            self.tree_map
                .iter()
                .map(|entry| entry.key().clone())
                .collect::<Vec<_>>(),
        );
    }

    pub fn remove(&self, path: Arc<PathBuf>) {
        if let dashmap::Entry::Occupied(entry) = self.tree_map.entry(path) {
            self.remove_merkle_hashes(entry.get().root_node());
            entry.remove();
        }
    }

    pub(super) fn remove_merkle_hashes(&self, node: Arc<IndexedNode>) {
        node.preorder_traverse(|x| {
            if let Some((_, h)) = self.node_hash_map.remove(&x.id()) {
                if let dashmap::Entry::Occupied(mut entry) = self.hash_map.entry(h) {
                    let set = entry.get_mut();
                    set.remove(x);
                    if set.is_empty() {
                        entry.remove();
                    }
                }
            }
        });
    }
}
