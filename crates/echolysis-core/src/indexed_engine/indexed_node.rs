use std::{
    hash::Hash,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub type Id = usize;

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
