use std::{path::PathBuf, sync::Arc};

use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};

use super::indexed_node::IndexedNode;

pub struct IndexedTree {
    root: Arc<IndexedNode>,
}

impl IndexedTree {
    pub fn new(path: Arc<PathBuf>, source: Arc<String>, tree: Tree, query: &Query) -> Self {
        let root_node = Self::build_index_nodes(tree, path, source, query);
        Self { root: root_node }
    }

    pub fn root_node(&self) -> Arc<IndexedNode> {
        self.root.clone()
    }

    fn build_index_nodes(
        tree: Tree,
        path: Arc<PathBuf>,
        source: Arc<String>,
        query: &Query,
    ) -> Arc<IndexedNode> {
        let tsnode = tree.root_node();
        let language = Arc::new(tree.language().to_owned());
        // Get all matches first using streaming iterator
        let mut query_cursor = QueryCursor::new();
        let mut captures = query_cursor.captures(query, tsnode, source.as_bytes());
        let mut match_map = std::collections::HashMap::new();
        while let Some((m, _)) = captures.next() {
            if let Some(capture) = m.captures.last() {
                match_map.insert(capture.node.id(), capture.index as usize);
            }
        }

        // Stack for traversal: (node, processed)
        let mut stack = vec![(tsnode, false)];
        // Map to store node's children
        let mut children_map = std::collections::HashMap::new();

        let mut result = None;
        while let Some((node, processed)) = stack.pop() {
            if !processed {
                // Push back the node as processed
                stack.push((node, true));

                // Push all children in reverse order (so they pop in correct order)
                let mut cursor = node.walk();
                if cursor.goto_first_child() {
                    let mut children = vec![];
                    loop {
                        children.push(cursor.node());
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    // Push children in reverse order
                    for child in children.into_iter().rev() {
                        stack.push((child, false));
                    }
                }
            } else {
                // All children have been processed, create the node
                let children = children_map.remove(&node.id()).unwrap_or_default();
                let query_index = match_map.get(&node.id()).copied();
                let indexed_node = Arc::new(IndexedNode::new(
                    node,
                    path.clone(),
                    query_index,
                    children,
                    source.clone(),
                    language.clone(),
                ));

                // Store this node in its parent's children list if it's not the root
                if let Some(parent_id) = node.parent().map(|n| n.id()) {
                    children_map
                        .entry(parent_id)
                        .or_insert_with(Vec::new)
                        .push(indexed_node.clone());
                }

                result = Some(indexed_node);
            }
        }

        // SAFETY: We know that the root node is always present
        result.unwrap()
    }
}
