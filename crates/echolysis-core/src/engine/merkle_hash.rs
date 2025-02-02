use std::sync::{Arc, Mutex};

use ahash::AHashMap;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use tree_sitter::{Node, Tree};

use crate::{
    languages::NodeTaste,
    languages::SupportedLanguage,
    utils::hash::{merge_structure_hash, ADashMap, FxDashMap},
    Id,
};

use super::Engine;

impl Engine {
    /// Computes the Merkle hash for a set of syntax trees.
    ///
    /// # Arguments
    ///
    /// * `language` - The programming language of the syntax trees.
    /// * `trees` - A map of source code identifiers to their corresponding syntax trees.
    /// * `query_of_node` - A map of node IDs to query IDs.
    /// * `sources` - A map of source code identifiers to their corresponding source code as strings.
    ///
    /// # Returns
    ///
    /// A map of Merkle hashes to sets of node IDs that have that hash.
    pub(super) fn merkle_hash_trees(
        language: &SupportedLanguage,
        trees: &ADashMap<Arc<String>, Tree>,
        query_map: &FxDashMap<Id, usize>,
        sources: &AHashMap<Arc<String>, &str>,
        protecting: Arc<Mutex<()>>,
    ) -> FxDashMap<u64, FxHashSet<Id>> {
        let hashmap = FxDashMap::default();

        trees.par_iter().for_each(|tree| {
            if let Some(source) = sources.get(tree.key()) {
                Self::merkle_hash_without_recursion(
                    language,
                    tree.value().root_node(),
                    query_map,
                    &hashmap,
                    source.as_bytes(),
                    protecting.clone(),
                );
            }
        });
        hashmap
    }

    pub(super) fn merkle_hash_without_recursion(
        language: &SupportedLanguage,
        root: Node<'_>,
        query_map: &FxDashMap<Id, usize>,
        hash_map: &FxDashMap<u64, FxHashSet<Id>>,
        source: &[u8],
        protecting: Arc<Mutex<()>>,
    ) -> u64 {
        let _guard = protecting.lock().unwrap();
        // (node, combined_hash, visited_children_count)
        let mut stack = vec![(root, 0u64, 0usize)];
        let result_map = FxDashMap::default();

        while let Some((node, combined_hash, visited_count)) = stack.pop() {
            if node.is_extra()
                || node.is_missing()
                || node.is_error()
                || language.node_taste(&node) == NodeTaste::Ignored
            {
                result_map.insert(node.id(), 0);
                continue;
            }

            if node.child_count() == 0 {
                let hash = language.simple_hash_node(
                    node,
                    query_map.get(&node.id().into()).map(|x| *x.value()),
                    source,
                );
                result_map.insert(node.id(), hash);
                continue;
            }

            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();

            if visited_count == 0 {
                stack.push((node, combined_hash, children.len()));
                for child in children.into_iter().rev() {
                    stack.push((child, 0, 0));
                }
                continue;
            }

            let mut final_hash = combined_hash;
            for child in children {
                if let Some(child_hash) = result_map.get(&child.id()) {
                    final_hash = merge_structure_hash(final_hash, *child_hash);
                }
            }

            if language.node_taste(&node) == NodeTaste::Interesting
                && language.cognitive_complexity(node) >= language.complexity_threshold()
            {
                hash_map
                    .entry(final_hash)
                    .or_default()
                    .insert(node.id().into());
            }

            result_map.insert(node.id(), final_hash);
        }

        result_map.get(&root.id()).map(|x| *x.value()).unwrap_or(0)
    }
}
