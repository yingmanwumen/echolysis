use std::{path::PathBuf, sync::Arc};

use dashmap::Entry;

use super::IndexedEngine;

impl IndexedEngine {
    pub fn remove(&self, path: Arc<PathBuf>) {
        if let Entry::Occupied(tree) = self.tree_map.entry(path) {
            tree.remove();
            todo!();
        }
    }
}
