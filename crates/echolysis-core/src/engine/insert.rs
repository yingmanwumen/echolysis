use std::sync::Arc;

use rayon::prelude::*;

use super::Engine;

impl Engine {
    // pub fn update(&self, path: Arc<String>, new_source: &str, edit: Option<InputEdit>) {
    //     match (self.tree_map.get(&path).map(|x| x.value().clone()), edit) {
    //         (Some(mut old), Some(edit)) => {
    //             if let Some(new_tree) = self
    //                 .language()
    //                 .incremental_parse(new_source, &edit, &mut old)
    //             {
    //                 let diff = tree_diff(&old, &new_tree);
    //                 self.remove_by_ids(diff);
    //                 Self::collect_data(
    //                     &new_tree,
    //                     path.clone(),
    //                     new_source,
    //                     &self.id_map,
    //                     &self.path_map,
    //                     self.language.query(),
    //                     &self.query_map,
    //                 );
    //                 Self::merkle_hash(
    //                     self.language(),
    //                     new_tree.root_node(),
    //                     &self.query_map,
    //                     &self.hash_map,
    //                     new_source.as_bytes(),
    //                 );
    //                 self.tree_map.entry(path).and_modify(|x| *x = new_tree);
    //             }
    //         }
    //         (None, _) | (_, None) => {
    //             self.insert(path, new_source);
    //         }
    //     }
    // }

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
            Self::merkle_hash(
                self.language(),
                tree.root_node(),
                &self.query_map,
                &self.hash_map,
                source.as_bytes(),
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
