use std::sync::Arc;

use crate::indexed_tree::{IndexedNode, IndexedTree};

use super::IndexedEngine;

impl IndexedEngine {
    pub(super) fn merkle_hash(&self, indexed_tree: &IndexedTree) {
        self.calculate_merkle_hash(indexed_tree.root_node());
    }

    fn calculate_merkle_hash(&self, node: Arc<IndexedNode>) -> u64 {
        todo!()
    }
}
