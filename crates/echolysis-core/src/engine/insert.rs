use std::sync::Arc;

use rayon::prelude::*;

use super::Engine;

impl Engine {
    pub fn insert(&self, path: Arc<String>, source: &str) {
        self.remove(path.clone());

        let mut parser = self.language.parser();
        let query = self.language.query();
        if let Some(tree) = parser.parse(source, None) {
            Self::collect_data(
                &tree,
                path.clone(),
                source,
                &self.id_map,
                &self.path_map,
                query,
                &self.query_map,
            );
            Self::merkle_hash_without_recursion(
                self.language(),
                tree.root_node(),
                &self.query_map,
                &self.hash_map,
                source.as_bytes(),
                self.protecting_guard.clone(),
            );
            self.tree_map.insert(path, tree);
        }
    }

    pub fn insert_many(&self, paths: Vec<(Arc<String>, String)>) {
        paths.into_par_iter().for_each(|(path, source)| {
            self.insert(path, &source);
        });
    }
}
