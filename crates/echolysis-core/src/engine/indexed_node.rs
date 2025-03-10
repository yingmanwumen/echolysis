use std::{
    hash::Hash,
    path::{Path, PathBuf},
    sync::Arc,
};

use tree_sitter::Node;

pub type Id = usize;

#[derive(Eq)]
pub struct IndexedNode {
    id: Id,
    path: Arc<PathBuf>,
    query_index: Option<usize>,
    children: Vec<Arc<IndexedNode>>,
    source: Arc<String>,
    kind: u16,
    language: Arc<tree_sitter::Language>,
    start: tree_sitter::Point,
    end: tree_sitter::Point,
    start_byte: usize,
    end_byte: usize,
    is_extra_or_missing_or_error: bool,
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

impl IndexedNode {
    pub fn new(
        node: Node<'_>,
        path: Arc<PathBuf>,
        query_index: Option<usize>,
        children: Vec<Arc<IndexedNode>>,
        source: Arc<String>,
        language: Arc<tree_sitter::Language>,
    ) -> Self {
        Self {
            id: node.id(),
            path,
            query_index,
            children,
            source,
            kind: node.kind_id(),
            start: node.start_position(),
            end: node.end_position(),
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            is_extra_or_missing_or_error: node.is_extra() || node.is_missing() || node.is_error(),
            language,
        }
    }

    pub fn is_extra_or_missing_or_error(&self) -> bool {
        self.is_extra_or_missing_or_error
    }

    pub fn kind(&self) -> &str {
        self.language
            .node_kind_for_id(self.kind)
            .unwrap_or_default()
    }

    pub fn text(&self) -> &str {
        self.source
            .get(self.start_byte..self.end_byte)
            .unwrap_or_default()
    }

    pub fn position_range(&self) -> (tree_sitter::Point, tree_sitter::Point) {
        (self.start, self.end)
    }

    pub fn byte_range(&self) -> (usize, usize) {
        (self.start_byte, self.end_byte)
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn children(&self) -> &Vec<Arc<IndexedNode>> {
        &self.children
    }

    pub fn query_index(&self) -> Option<usize> {
        self.query_index
    }

    pub fn preorder_traverse(&self, mut f: impl FnMut(&IndexedNode)) {
        let mut stack = vec![self];

        while let Some(node) = stack.pop() {
            f(node);
            for child in node.children.iter().rev() {
                stack.push(child);
            }
        }
    }

    pub fn all_children(node: Arc<Self>) -> Vec<Arc<IndexedNode>> {
        let mut stack = vec![node];
        let mut res = vec![];

        while let Some(node) = stack.pop() {
            for child in node.children.iter().rev() {
                res.push(child.clone());
                stack.push(child.clone());
            }
        }
        res
    }
}
