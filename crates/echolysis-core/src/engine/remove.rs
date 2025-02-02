use std::sync::Arc;

use super::Engine;

use dashmap::Entry;
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use crate::{utils::tree::Traverse, Id};

impl Engine {
    /// Removes a tree and all its associated data from the engine using the given path.
    ///
    /// This method performs the following operations:
    /// 1. Looks up the tree in the tree map using the provided path
    /// 2. If found, collects all node IDs in the tree
    /// 3. Removes all associated data for these IDs
    /// 4. Removes the tree itself from the tree map
    ///
    /// # Arguments
    /// * `path` - An `Arc<String>` representing the path to the tree to be removed
    pub fn remove(&self, path: Arc<String>) {
        if let Entry::Occupied(tree) = self.tree_map.entry(path) {
            let mut ids = FxHashSet::default();
            tree.get().preorder_traverse(|node| {
                ids.insert(Id::from(node.id()));
            });
            self.remove_by_ids(ids);
            // NOTE: This guard is essential!!!
            let _guard = self.protecting_guard.lock().unwrap();
            tree.remove();
        }
    }

    pub fn remove_many(&self, paths: Vec<Arc<String>>) {
        paths.into_par_iter().for_each(|path| {
            self.remove(path);
        });
    }

    pub fn remove_all(&self) {
        self.tree_map.clear();
        self.hash_map.clear();
        self.id_map.clear();
        self.path_map.clear();
        self.query_map.clear();
    }

    /// Removes all data associated with the given set of IDs from the engine's internal maps.
    ///
    /// This method cleans up:
    /// - Entries in the hash map (removing IDs from value sets)
    /// - Empty entries from the hash map
    /// - Entries from the ID map
    /// - Entries from the path map
    /// - Entries from the query map
    ///
    /// The cleanup of ID, path, and query maps is performed in parallel using rayon.
    ///
    /// # Arguments
    /// * `ids` - A `FxHashSet<Id>` containing all IDs to be removed
    fn remove_by_ids(&self, ids: FxHashSet<Id>) {
        for mut entry in self.hash_map.iter_mut() {
            entry.value_mut().retain(|id| !ids.contains(id));
        }
        self.hash_map.retain(|_, v| !v.is_empty());
        ids.into_par_iter().for_each(|id| {
            self.id_map.remove(&id);
            self.path_map.remove(&id);
            self.query_map.remove(&id);
        });
    }
}
