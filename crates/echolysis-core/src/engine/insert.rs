use std::{path::PathBuf, sync::Arc};

use super::{indexed_tree::IndexedTree, Engine};
use rayon::prelude::*;

impl Engine {
    pub fn insert_many(
        &self,
        sources: impl IntoParallelIterator<Item = (Arc<PathBuf>, Arc<String>)>,
    ) {
        sources.into_par_iter().for_each(|(path, source)| {
            self.insert(path, source);
        });
    }

    pub fn insert(&self, path: Arc<PathBuf>, source: Arc<String>) -> Option<()> {
        self.remove(path.clone());
        let mut parser = self.language.parser();
        let query = self.language.query();
        let tree = parser.parse(source.as_str(), None)?;
        let indexed_tree = IndexedTree::new(path.clone(), source, tree, query);
        self.merkle_hash(&indexed_tree);
        self.tree_map.insert(path, indexed_tree);
        Some(())
    }
}
