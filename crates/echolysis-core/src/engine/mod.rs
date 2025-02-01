mod insert;
mod merkle_hash;
mod remove;

use std::sync::Arc;

use ahash::AHashMap;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Node, Query, QueryCursor, Tree};

use crate::{
    languages::SupportedLanguage,
    utils::{
        hash::{ADashMap, FxDashMap, FxDashSet},
        tree::{NodeExt, Traverse},
    },
    Id,
};

pub struct Engine {
    language: SupportedLanguage,
    tree_map: ADashMap<Arc<String>, Tree>,
    id_map: FxDashMap<Id, Node<'static>>,
    path_map: FxDashMap<Id, Arc<String>>,
    hash_map: FxDashMap<u64, FxHashSet<Id>>,
    query_map: FxDashMap<Id, usize>,
}

impl Engine {
    pub fn new(language: SupportedLanguage, sources: Option<AHashMap<Arc<String>, &str>>) -> Self {
        let id_map = FxDashMap::default();
        let path_map = FxDashMap::default();
        let tree_map = ADashMap::default();
        let query_map = FxDashMap::default();

        if let Some(sources) = sources {
            let query = language.query();
            sources.par_iter().for_each(|(path, source)| {
                let mut parser = language.parser();
                if let Some(tree) = parser.parse(source, None) {
                    Self::collect_data(
                        &tree,
                        path.clone(),
                        source,
                        &id_map,
                        &path_map,
                        query,
                        &query_map,
                    );
                    tree_map.insert(path.clone(), tree);
                }
            });

            let hash_map = Self::merkle_hash_trees(&language, &tree_map, &query_map, &sources);
            Self {
                language,
                tree_map,
                id_map,
                hash_map,
                path_map,
                query_map,
            }
        } else {
            Self {
                language,
                tree_map,
                id_map,
                path_map,
                query_map,
                hash_map: FxDashMap::default(),
            }
        }
    }

    pub fn detect_duplicates(&self) -> Vec<Vec<Id>> {
        let children = FxDashSet::default();
        self.hash_map.par_iter().for_each(|nodes| {
            if nodes.value().len() < 2 {
                return;
            }
            for &node in nodes.value() {
                if let Some(node) = self.get_node_by_id(node) {
                    node.all_children().into_iter().for_each(|child| {
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

    pub fn get_node_by_id(&self, id: Id) -> Option<Node<'static>> {
        self.id_map.get(&id).map(|x| *x.value())
    }

    pub fn get_path_by_id(&self, id: Id) -> Option<Arc<String>> {
        self.path_map.get(&id).map(|x| x.value().clone())
    }

    pub fn language(&self) -> &SupportedLanguage {
        &self.language
    }

    fn collect_data(
        tree: &Tree,
        path: Arc<String>,
        source: &str,
        id_map: &FxDashMap<Id, Node<'static>>,
        path_map: &FxDashMap<Id, Arc<String>>,
        query: &Query,
        query_map: &FxDashMap<Id, usize>,
    ) {
        tree.preorder_traverse(|node| {
            let node: Node<'static> = unsafe { std::mem::transmute(node) };
            path_map.insert(node.id().into(), path.clone());
            id_map.insert(node.id().into(), node);
        });
        QueryCursor::new()
            .captures(query, tree.root_node(), source.as_bytes())
            .for_each(|(x, _)| {
                if let Some(capture) = x.captures.last() {
                    query_map.insert(capture.node.id().into(), capture.index as usize);
                }
            });
    }
}
