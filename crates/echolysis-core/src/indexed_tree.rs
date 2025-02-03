use std::{
    hash::Hash,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tree_sitter::{Node, Query, QueryCursor, StreamingIterator, Tree};

pub type Id = usize;

pub struct IndexedTree {
    ts_tree: Mutex<Option<Tree>>,
    root: Arc<IndexedNode>,
}

#[derive(Eq)]
pub struct IndexedNode {
    id: Id,
    path: Arc<String>,
    query_index: Option<usize>,
    children: Vec<Arc<IndexedNode>>,
    source: Arc<String>,
    kind: String,
}

impl Hash for IndexedNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for IndexedNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
