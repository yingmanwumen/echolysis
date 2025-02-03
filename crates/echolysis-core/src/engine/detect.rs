use std::sync::Arc;

use super::{indexed_node::IndexedNode, Engine};

use dashmap::DashSet;
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashSet};

impl Engine {
    /// Detect duplicate code blocks in the parsed source files.
    ///
    /// # Arguments
    /// * `limitation` - Optional maximum number of duplicate groups to return
    ///
    /// # Returns
    /// A vector of duplicate node groups, where each group contains identical code blocks
    pub fn detect_duplicates(&self, limitation: Option<usize>) -> Vec<Vec<Arc<IndexedNode>>> {
        // First collect all child nodes that are part of larger nodes
        let child_nodes = self.collect_child_nodes();

        // Then find groups of identical nodes that aren't children of other nodes
        self.hash_map
            .par_iter()
            .filter_map(|nodes| {
                if nodes.len() < 2 {
                    None
                } else {
                    self.extract_non_child_nodes(&nodes, &child_nodes)
                }
            })
            .take_any(limitation.unwrap_or(usize::MAX))
            .collect()
    }

    /// Collects all child nodes from node groups that have duplicates
    fn collect_child_nodes(&self) -> DashSet<Arc<IndexedNode>, FxBuildHasher> {
        let child_nodes = DashSet::with_hasher(FxBuildHasher);
        self.hash_map
            .par_iter()
            .filter(|nodes| nodes.len() > 1)
            .for_each(|nodes| {
                for node in nodes.value() {
                    for child in node.children() {
                        child_nodes.insert(child.clone());
                    }
                }
            });
        child_nodes
    }

    /// Extracts nodes that aren't children of other nodes from a group
    fn extract_non_child_nodes(
        &self,
        nodes: &FxHashSet<Arc<IndexedNode>>,
        child_nodes: &DashSet<Arc<IndexedNode>, FxBuildHasher>,
    ) -> Option<Vec<Arc<IndexedNode>>> {
        let group: Vec<_> = nodes
            .iter()
            .filter(|node| !child_nodes.contains(*node))
            .cloned()
            .collect();

        if group.len() > 1 {
            Some(group)
        } else {
            None
        }
    }
}
