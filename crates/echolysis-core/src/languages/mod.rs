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
    fn language(&self) -> &tree_sitter::Language;
    fn query(&self) -> &Query;
    fn parser(&self) -> Parser;
    fn simple_hash_node(&self, node: Node<'_>, query_index: Option<usize>, source: &[u8]) -> u64;
    fn node_taste(&self, node: &tree_sitter::Node<'_>) -> NodeTaste;
    fn cognitive_complexity(&self, node: Node<'_>) -> f64;

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
