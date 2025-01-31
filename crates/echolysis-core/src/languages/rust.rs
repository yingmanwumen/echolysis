use phf::{phf_map, phf_set};
use tree_sitter::{Parser, Query};

use super::{Language, NodeTaste};

pub struct Rust {
    hash_builder: ahash::RandomState,
    query: Query,
    query_names: Vec<String>,
    language: tree_sitter::Language,
}

const QUERY_NOT_TO_OBFUSCATE: phf::Set<&str> = phf_set! {
    "type",
    "constant",
    "function",
    "constructor",
    "label"
};

const QUERY_TO_OBFUSCATE: phf::Set<&str> = phf_set! {
    "variable.parameter"
};

const NODES_TO_OBFUSCATE: phf::Set<&str> = phf_set! {
    "identifier"
};

const INTERESTING_NODES: phf::Set<&str> = phf_set! {
    "call_expression",
    "const_block",
    "for_expression",
    "if_expression",
    "loop_expression",
    "match_expression",
    "while_expression",
    "function_item",
    "impl_item",
    "trait_item",
    "closure_expression",
};

const IGNORED_NODES: phf::Set<&str> = phf_set! {
    "block_comment",
    "doc_comment",
    "line_comment",
    "inner_doc_comment_marker",
    "outer_doc_comment_marker",
    "empty_statement",
};

const COGNITIVE_COMPLEXITY_WEIGHT: phf::Map<&str, f64> = phf_map! {
    "block" => 1.0,
    "if_expression" => 1.0,
    "match_expression" => 1.0,
    "match_pattern" => 1.0,
    "loop_expression" => 1.0,
    "for_expression" => 1.0,
    "while_expression" => 1.0,
    "break_expression" => 1.0,
    "continue_expression" => 1.0,
    "try_expression" => 1.0,
    "try_block" => 1.0,
    "binary_expression" => 0.5,
    "unary_expression" => 0.5,
    "let_condition" => 0.5,
    "closure_expression" => 1.0,
    "async_block" => 1.0,
    "function_item" => 1.0,
    "unsafe_block" => 1.0,
    "await_expression" => 0.5,
    "type_cast_expression" => 0.5,
    "macro_invocation" => 1.0,
    "attribute_item" => 0.5,
    "or_pattern" => 0.5,
    "compound_assignment_expression" => 0.5,
    "range_expression" => 0.5,
    "lifetime" => 0.5,
    "const_block" => 1.0,
    "gen_block" => 1.0,
    "array_expression" => 1.0,
    "call_expression" => 1.0,
    "index_expression" => 1.0,
    "parenthesized_expression" => 0.5,
    "reference_expression" => 0.5,
    "return_expression" => 1.5,
    "yield_expression" => 1.5,
    "tuple_expression" => 1.0,
    "tuple_pattern" => 1.0,
    "type_arguments" => 0.5,
    "struct_pattern" => 1.0,
    "field_pattern" => 0.5,
    "remaining_field_pattern" => 0.5,
    "tuple_struct_pattern" => 1.0,
};

impl Default for Rust {
    fn default() -> Self {
        let language: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
        let query = Query::new(&language, tree_sitter_rust::HIGHLIGHTS_QUERY).unwrap();
        let query_names = query
            .capture_names()
            .iter()
            .map(|x| x.to_string())
            .collect();
        Self {
            hash_builder: ahash::RandomState::new(),
            query: Query::new(&language, tree_sitter_rust::HIGHLIGHTS_QUERY).unwrap(),
            query_names,
            language,
        }
    }
}

impl Language for Rust {
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
            if QUERY_NOT_TO_OBFUSCATE.contains(query) {
                return self.hash_builder.hash_one(node.utf8_text(source).unwrap());
            }
            if QUERY_TO_OBFUSCATE.contains(query) {
                return self.hash_builder.hash_one(query);
            }
        }
        if NODES_TO_OBFUSCATE.contains(node.kind()) {
            return self.hash_builder.hash_one(node.kind());
        }
        self.hash_builder.hash_one(node.utf8_text(source).unwrap())
    }

    fn node_taste(&self, node: &tree_sitter::Node<'_>) -> NodeTaste {
        if INTERESTING_NODES.contains(node.kind()) {
            NodeTaste::Interesting
        } else if IGNORED_NODES.contains(node.kind()) {
            NodeTaste::Ignored
        } else {
            NodeTaste::Normal
        }
    }

    fn cognitive_complexity(&self, node: tree_sitter::Node<'_>) -> f64 {
        let mut res = 0.0;
        let mut stack = vec![node];
        while let Some(node) = stack.pop() {
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
            if let Some(&weight) = COGNITIVE_COMPLEXITY_WEIGHT.get(node.kind()) {
                res += weight;
            }
        }
        res
    }
}
