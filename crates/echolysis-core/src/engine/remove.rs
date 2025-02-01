use std::sync::Arc;

use super::Engine;

use dashmap::Entry;
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use crate::{utils::tree::Traverse, Id};

impl Engine {
    pub fn remove(&self, path: Arc<String>) {
        if let Entry::Occupied(tree) = self.tree_map.entry(path) {
            let mut ids = FxHashSet::default();
            tree.get().preorder_traverse(|node| {
                ids.insert(Id::from(node.id()));
            });
            self.remove_by_ids(ids);
            tree.remove();
        }
    }

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
