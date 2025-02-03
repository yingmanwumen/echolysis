use std::{
    hash::Hash,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tree_sitter::{Node, Query, QueryCursor, StreamingIterator, Tree};

use super::indexed_node::IndexedNode;

pub struct IndexedTree {
    ts_tree: Mutex<Option<Tree>>,
    root: Arc<IndexedNode>,
}

impl IndexedTree {
    pub fn new(path: Arc<PathBuf>, source: Arc<String>, tree: Tree, query: &Query) -> Self {
        todo!()
    }

    pub fn root_node(&self) -> Arc<IndexedNode> {
        self.root.clone()
    }

    fn build_index_nodes(
        tsnode: Node<'_>,
        path: Arc<String>,
        source: Arc<String>,
        query: &Query,
    ) -> Arc<IndexedNode> {
        todo!()
    }
}
