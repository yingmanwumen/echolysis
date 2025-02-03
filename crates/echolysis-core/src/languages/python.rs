use phf::phf_set;
use tree_sitter::{Parser, Query};

use crate::{indexed_engine::indexed_node::IndexedNode, utils::tree::NodeExt};

use super::{Language, NodeTaste};

pub struct Python {
    hash_builder: ahash::RandomState,
    query: Query,
    query_names: Vec<String>,
    language: tree_sitter::Language,
}

const QUERY_TO_OBFUSCATE: phf::Set<&str> = phf_set! {
    "variable"
};

const PY_INTERESTING_NODES: phf::Set<&str> = phf_set! {
    "for_statement",
    "if_statement",
    "match_statement",
    "try_statement",
    "while_statement",
    "with_statement",
    "call",
    "function_definition",
    "class_definition",
};

const PY_IGNORED_NODES: phf::Set<&str> = phf_set! {
    "comment",
};

impl Default for Python {
    fn default() -> Self {
        let language: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
        let query = Query::new(&language, tree_sitter_python::HIGHLIGHTS_QUERY).unwrap();
        let query_names = query
            .capture_names()
            .iter()
            .map(|x| x.to_string())
            .collect();
        Self {
            hash_builder: ahash::RandomState::new(),
            query,
            query_names,
            language,
        }
    }
}

impl Language for Python {
    fn language(&self) -> &tree_sitter::Language {
        &self.language
    }

    fn parser(&self) -> Parser {
        let mut parser = Parser::new();
        // SAFETY: We know the language is valid
        parser.set_language(self.language()).unwrap();
        parser
    }

    fn query(&self) -> &Query {
        &self.query
    }

    fn simple_hash_node(
        &self,
        node: tree_sitter::Node<'_>,
        query_index: Option<usize>,
        source: &[u8],
    ) -> u64 {
        if let Some(index) = query_index {
            let query = &self.query_names[index];
            if QUERY_TO_OBFUSCATE.contains(query) {
                return self.hash_builder.hash_one(query);
            }
        }
        self.hash_builder.hash_one(node.text(source))
    }

    fn simple_hash_indexed_node(&self, node: &IndexedNode) -> u64 {
        if let Some(index) = node.query_index() {
            let query = &self.query_names[index];
            if QUERY_TO_OBFUSCATE.contains(query) {
                return self.hash_builder.hash_one(query);
            }
        }
        self.hash_builder.hash_one(node.text())
    }

    fn node_taste(&self, node: &tree_sitter::Node<'_>) -> NodeTaste {
        if PY_INTERESTING_NODES.contains(node.kind()) {
            NodeTaste::Interesting
        } else if PY_IGNORED_NODES.contains(node.kind()) {
            NodeTaste::Ignored
        } else {
            NodeTaste::Normal
        }
    }

    fn indexed_node_taste(&self, node: &IndexedNode) -> NodeTaste {
        if PY_INTERESTING_NODES.contains(node.kind()) {
            NodeTaste::Interesting
        } else if PY_IGNORED_NODES.contains(node.kind()) {
            NodeTaste::Ignored
        } else {
            NodeTaste::Normal
        }
    }

    fn cognitive_complexity(&self, node: tree_sitter::Node<'_>) -> f64 {
        let mut stack = vec![node];
        let mut res = 0.0;
        while let Some(node) = stack.pop() {
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
            let node_kind = node.kind();
            if node_kind.contains("statement")
                || node_kind.contains("call")
                || node_kind.contains("function_definition")
            {
                res += 1.0;
            }
        }
        res
    }

    fn indexed_node_cognitive_complexity(&self, node: &IndexedNode) -> f64 {
        let mut stack = vec![node];
        let mut res = 0.0;
        while let Some(node) = stack.pop() {
            for child in node.children() {
                stack.push(child);
            }
            let node_kind = node.kind();
            if node_kind.contains("statement")
                || node_kind.contains("call")
                || node_kind.contains("function_definition")
            {
                res += 1.0;
            }
        }
        res
    }
}
