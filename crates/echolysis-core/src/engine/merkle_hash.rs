use std::sync::Arc;

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
    ) -> FxDashMap<u64, FxHashSet<Id>> {
        let hashmap = FxDashMap::default();

        trees.par_iter().for_each(|tree| {
            if let Some(source) = sources.get(tree.key()) {
                Self::merkle_hash(
                    language,
                    tree.value().root_node(),
                    query_map,
                    &hashmap,
                    source.as_bytes(),
                );
            }
        });
        hashmap
    }

    pub(super) fn merkle_hash(
        language: &SupportedLanguage,
        node: Node<'_>,
        query_map: &FxDashMap<Id, usize>,
        hash_map: &FxDashMap<u64, FxHashSet<Id>>,
        source: &[u8],
    ) -> u64 {
        if node.is_extra()
            || node.is_missing()
            || node.is_error()
            || language.node_taste(&node) == NodeTaste::Ignored
        {
            return 0;
        }
        if node.child_count() == 0 {
            return language.simple_hash_node(
                node,
                query_map.get(&node.id().into()).map(|x| *x.value()),
                source,
            );
        }
        let mut combined_hash: u64 = 0;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            combined_hash = merge_structure_hash(
                combined_hash,
                Self::merkle_hash(language, child, query_map, hash_map, source),
            )
        }
        if language.node_taste(&node) == NodeTaste::Interesting
            && language.cognitive_complexity(node) >= language.complexity_threshold()
        {
            hash_map
                .entry(combined_hash)
                .or_default()
                .insert(node.id().into());
        }
        combined_hash
    }
}
