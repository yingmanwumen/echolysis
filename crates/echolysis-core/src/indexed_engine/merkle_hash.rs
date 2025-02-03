use std::sync::Arc;

use crate::{languages::NodeTaste, utils::hash::merge_structure_hash};

use super::{indexed_node::IndexedNode, indexed_tree::IndexedTree, IndexedEngine};

impl IndexedEngine {
    pub(super) fn merkle_hash(&self, indexed_tree: &IndexedTree) {
        self.calculate_merkle_hash(indexed_tree.root_node());
    }

    fn calculate_merkle_hash(&self, node: Arc<IndexedNode>) -> u64 {
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
                merge_structure_hash(combined_hash, self.calculate_merkle_hash(child.clone()));
        }
        if self.language.indexed_node_taste(&node) == NodeTaste::Interesting
            && self.language.indexed_node_cognitive_complexity(&node)
                >= self.language.complexity_threshold()
        {
            self.hash_map.entry(combined_hash).or_default().insert(node);
        }
        combined_hash
    }
}
