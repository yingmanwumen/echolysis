mod python;
mod rust;

use std::ops::Deref;

use python::Python;
use rust::Rust;
use tree_sitter::{InputEdit, Parser, Query};

use crate::engine::indexed_node::IndexedNode;

pub enum SupportedLanguage {
    Python(Python),
    Rust(Rust),
}

impl SupportedLanguage {
    pub fn supported_languages() -> Vec<&'static str> {
        vec!["python", "rust"]
    }

    pub fn from_language_id<T: AsRef<str>>(language_id: T) -> Option<SupportedLanguage> {
        match language_id.as_ref() {
            "python" => Some(SupportedLanguage::Python(Python::default())),
            "rust" => Some(SupportedLanguage::Rust(Rust::default())),
            _ => None,
        }
    }
}

impl Deref for SupportedLanguage {
    type Target = dyn Language;
    fn deref(&self) -> &Self::Target {
        match self {
            SupportedLanguage::Python(python) => python,
            SupportedLanguage::Rust(rust) => rust,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum NodeTaste {
    Ignored,
    Interesting,
    Normal,
}

pub trait Language {
    /// Returns the tree-sitter Language definition for this programming language
    fn language(&self) -> &tree_sitter::Language;

    /// Returns the syntax highlighting query used to identify language constructs
    fn query(&self) -> &Query;

    /// Creates and configures a new tree-sitter Parser instance for this language
    fn parser(&self) -> Parser;

    /// Computes a hash value for a single syntax node
    ///
    /// # Arguments
    /// * `node` - The syntax node to hash
    /// * `query_index` - Optional index into the highlighting query results
    /// * `source` - The source code bytes
    ///
    /// # Returns
    /// A 64-bit hash value representing the node's content
    fn simple_hash_indexed_node(&self, node: &IndexedNode) -> u64;

    /// Determines the importance level of a syntax node for analysis
    ///
    /// # Arguments
    /// * `node` - The syntax node to evaluate
    ///
    /// # Returns
    /// A NodeTaste value indicating if the node should be:
    /// - Ignored: Excluded from analysis (like comments)
    /// - Interesting: Given special attention (like function definitions)
    /// - Normal: Processed normally
    fn indexed_node_taste(&self, node: &IndexedNode) -> NodeTaste;

    fn indexed_node_cognitive_complexity(&self, node: &IndexedNode) -> f64;

    /// Returns the minimum cognitive complexity threshold for considering a node interesting
    ///
    /// Nodes with complexity scores above this threshold are candidates for duplication detection
    fn complexity_threshold(&self) -> f64 {
        10.0
    }

    fn parse(&self, text: &str) -> Option<tree_sitter::Tree> {
        self.parser().parse(text, None)
    }

    fn incremental_parse(
        &self,
        new_source: &str,
        edit: &InputEdit,
        old: &mut tree_sitter::Tree,
    ) -> Option<tree_sitter::Tree> {
        old.edit(edit);
        self.parser().parse(new_source, Some(old))
    }
}
