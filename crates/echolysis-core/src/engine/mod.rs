mod merkle_hash;

use std::sync::Arc;

use ahash::AHashMap;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, QueryCursor, Tree};

use crate::{
    utils::{
        hash::{ADashMap, FxDashMap, FxDashSet},
        tree::{children_set, preorder_traverse},
    },
    SupportedLanguage,
};

pub struct Engine {
    language: SupportedLanguage,
    #[allow(unused)]
    tree_map: ADashMap<Arc<String>, Tree>,
    id_map: FxDashMap<usize, Node<'static>>,
    path_map: FxDashMap<usize, Arc<String>>,
    hash_map: FxDashMap<u64, FxHashSet<usize>>,
}

impl Engine {
    pub fn new(language: SupportedLanguage, sources: AHashMap<String, &str>) -> Self {
        let id_map = FxDashMap::default();
        let path_map = FxDashMap::default();
        let tree_map = ADashMap::default();
        let query = language.query();
        let query_of_node = FxDashMap::default();

        sources.par_iter().for_each(|(path, source)| {
            let mut parser = language.parser();
            let path = Arc::new(path.to_string());
            if let Some(tree) = parser.parse(source, None) {
                analyze_tree(
                    &tree,
                    path.clone(),
                    source,
                    &id_map,
                    &path_map,
                    query,
                    &query_of_node,
                );
                tree_map.insert(path.clone(), tree);
            }
        });

        let hash_map = Self::merkle_hash(&language, &tree_map, query_of_node, sources);

        Self {
            language,
            tree_map,
            id_map,
            hash_map,
            path_map,
        }
    }

    pub fn detect_duplicates(&self) -> Vec<Vec<usize>> {
        let children = FxDashSet::default();
        self.hash_map.par_iter().for_each(|nodes| {
            if nodes.value().len() < 2 {
                return;
            }
            for &node in nodes.value() {
                if let Some(node) = self.get_node_by_id(node) {
                    children_set(node).into_iter().for_each(|child| {
                        children.insert(child);
                    });
                }
            }
        });
        self.hash_map
            .par_iter()
            .filter_map(|nodes| {
                let group: Vec<_> = nodes
                    .iter()
                    .filter(|node| !children.contains(node))
                    .copied()
                    .collect();
                if group.len() > 1 {
                    Some(group)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_node_by_id(&self, id: usize) -> Option<Node<'static>> {
        self.id_map.get(&id).map(|x| *x.value())
    }

    pub fn get_path_by_id(&self, id: usize) -> Option<Arc<String>> {
        self.path_map.get(&id).map(|x| x.value().clone())
    }

    pub fn language(&self) -> &SupportedLanguage {
        &self.language
    }
}

fn analyze_tree(
    tree: &tree_sitter::Tree,
    path: Arc<String>,
    source: &str,
    id_map: &FxDashMap<usize, Node<'static>>,
    path_map: &FxDashMap<usize, Arc<String>>,
    query: &tree_sitter::Query,
    query_of_node: &FxDashMap<usize, usize>,
) {
    preorder_traverse(tree.root_node(), |node| {
        let node: Node<'static> = unsafe { std::mem::transmute(node) };
        path_map.insert(node.id(), path.clone());
        id_map.insert(node.id(), node);
    });
    QueryCursor::new()
        .captures(query, tree.root_node(), source.as_bytes())
        .for_each(|(x, _)| {
            if let Some(capture) = x.captures.last() {
                query_of_node.insert(capture.node.id(), capture.index as usize);
            }
        });
}
