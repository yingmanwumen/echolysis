mod python;
mod rust;

use std::ops::Deref;

use python::Python;
use rust::Rust;
use tree_sitter::{InputEdit, Node, Parser, Query};

pub enum SupportedLanguage {
    Python(Python),
    Rust(Rust),
}

impl TryFrom<&str> for SupportedLanguage {
    type Error = ();
    fn try_from(value: &str) -> Result<SupportedLanguage, ()> {
        Ok(match value {
            "rust" => SupportedLanguage::Rust(Rust::default()),
            "python" => SupportedLanguage::Python(Python::default()),
            _ => return Err(()),
        })
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
    fn simple_hash_node(&self, node: Node<'_>, query_index: Option<usize>, source: &[u8]) -> u64;

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
    fn node_taste(&self, node: &Node<'_>) -> NodeTaste;

    /// Calculates a cognitive complexity score for a syntax node and its sub-tree
    ///
    /// # Arguments
    /// * `node` - The root node to analyze
    ///
    /// # Returns
    /// A floating point score where higher values indicate more complex code
    fn cognitive_complexity(&self, node: Node<'_>) -> f64;

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
