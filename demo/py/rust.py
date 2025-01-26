#!/usr/bin/env python

from collections import defaultdict
import sys
import time
from typing import Dict, List, Optional, Set, Tuple
from tree_sitter import Language, Node, Parser, Tree, Query
import tree_sitter_rust

NATIVE_ENCODING = sys.getfilesystemencoding()

RUST_COMPLEX_STATEMENT = {
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
}

RUST_COGNITIVE_NODES = {
    "block": 1,
    "if_expression": 1,
    "match_expression": 1,
    "match_pattern": 1,
    "loop_expression": 1,
    "for_expression": 1,
    "while_expression": 1,
    "break_expression": 1,
    "continue_expression": 1,
    "try_expression": 1,
    "try_block": 1,
    "binary_expression": 0.5,
    "unary_expression": 0.5,
    "let_condition": 0.5,
    "closure_expression": 1,
    "async_block": 1,
    "function_item": 1,
    "unsafe_block": 1,
    "await_expression": 0.5,
    "type_cast_expression": 0.5,
    "macro_invocation": 1,
    "attribute_item": 0.5,
    "or_pattern": 0.5,
    "compound_assignment_expression": 0.5,
    "range_expression": 0.5,
    "lifetime": 0.5,
    "const_block": 1,
    "gen_block": 1,
    "array_expression": 1,
    "call_expression": 1,
    "index_expression": 1,
    "parenthesized_expression": 0.5,
    "range_expression": 0.5,
    "reference_expression": 0.5,
    "return_expression": 1.5,
    "yield_expression": 1.5,
    "tuple_expression": 1,
    "tuple_pattern": 1,
    "type_arguments": 0.5,
    "struct_pattern": 1,
    "field_pattern": 0.5,
    "remaining_field_pattern": 0.5,
    "tuple_struct_pattern": 1,
}

RUST_NODES_TO_SKIP = {
    "block_comment",
    "doc_comment",
    "line_comment",
    "inner_doc_comment_marker",
    "outer_doc_comment_marker",
    "empty_statement",
}

RUST_QUERY_TO_OBFUSCATE = {"variable.parameter"}
RUST_QUERY_NOT_TO_OBFUSCATE = {"type", "constant", "function", "constructor", "label"}

RUST_NODES_TO_OBFUSCATE = {"identifier"}

DUPLICATED_THRESHOLD = 10


def parse_file(path: str, parser: Parser) -> Tree:
    try:
        with open(path) as f:
            return parser.parse(bytes(f.read(), NATIVE_ENCODING))
    except Exception as e:
        print("Failed to parse: " + path)
        raise e


def hash_node(node: Node, query: Optional[str]) -> int:
    res = hash(node.text)
    if query:
        if query in RUST_QUERY_NOT_TO_OBFUSCATE:
            return res
        if query in RUST_QUERY_TO_OBFUSCATE:
            return hash(query)
    if node.type in RUST_NODES_TO_OBFUSCATE:
        return hash(node.type)
    return res


def merkle_hash(
    trees: Dict[str, Tree], query_of_node: Dict[str, Dict[int, str]]
) -> Dict[int, int]:
    res = {}

    def do_merkle_hash(
        node: Node, record: Dict[int, int], query_of_node: Dict[int, str]
    ) -> int:
        # Skip comment
        if node.type in RUST_NODES_TO_SKIP:
            return 0

        if node.child_count == 0:
            return hash_node(node, query_of_node.get(node.id))

        combined_hash = 0
        for child in node.children:
            child_hash = do_merkle_hash(child, record, query_of_node)
            combined_hash = hash((combined_hash, child_hash))

        if node.type in RUST_COMPLEX_STATEMENT:
            record[node.id] = combined_hash

        return combined_hash

    for path, tree in trees.items():
        root_query = query_of_node.get(path, {})
        do_merkle_hash(tree.root_node, res, root_query)

    return res


def child_set(node: Node) -> Set[Node]:
    res = set()
    res.update(node.children)
    for child in node.children:
        res.update(child_set(child))
    return res


def cognitive_complexity(node: Node) -> float:
    res = 0.0
    stack = [node]
    while stack:
        n = stack.pop()
        stack.extend(n.children)
        res += 0 if n.type not in RUST_COGNITIVE_NODES else RUST_COGNITIVE_NODES[n.type]
        # if n.type in RUST_COGNITIVE_NODES:
        #     print(n.type, RUST_COGNITIVE_NODES[n.type])
    return res


def detect_duplicated_tree(
    hash_of_node: Dict[int, int], id_map: Dict[int, Node]
) -> List[List[int]]:
    hash_to_nodes: Dict[int, List[int]] = defaultdict(list)
    for node_id, hash in hash_of_node.items():
        if cognitive_complexity(id_map[node_id]) < DUPLICATED_THRESHOLD:
            continue
        hash_to_nodes[hash].append(node_id)
    children = set()
    for nodes in hash_to_nodes.values():
        if len(nodes) > 1:
            for node in nodes:
                children.update(x.id for x in child_set(id_map[node]))
    res: List[List[int]] = []
    for node_ids in hash_to_nodes.values():
        group: List[int] = [node for node in node_ids if node not in children]
        if len(group) > 1:
            res.append(group)
    return res


def get_extra_data_of_tree(
    trees: Dict[str, Tree]
) -> Tuple[Dict[int, Node], Dict[int, str]]:
    id_map = {}
    path_map = {}

    for path, tree in trees.items():
        stack = [tree.root_node]
        while stack:
            node = stack.pop()
            id_map[node.id] = node
            path_map[node.id] = path
            stack.extend(node.children)
    return id_map, path_map


def main():
    paths = sys.argv[1:]

    now = time.time()
    rust_language = Language(tree_sitter_rust.language())
    rust_parser = Parser(rust_language)
    rust_query = Query(rust_language, tree_sitter_rust.HIGHLIGHTS_QUERY)
    trees: Dict[str, Tree] = {}
    query_to_nodes: Dict[str, Dict[str, List[Node]]] = {}
    query_of_node: Dict[str, Dict[int, str]] = {}
    for path in paths:
        trees[path] = parse_file(path, rust_parser)
        query_to_nodes[path] = rust_query.captures(trees[path].root_node)
        query_of_node[path] = {}
        for query, nodes in query_to_nodes[path].items():
            for node in nodes:
                query_of_node[path][node.id] = query
    parsing_cost = time.time() - now

    now = time.time()
    id_map_of_tree, path_of_nodes = get_extra_data_of_tree(trees)
    loading_cost = time.time() - now

    now = time.time()
    hash_of_node = merkle_hash(trees, query_of_node)
    hashing_cost = time.time() - now

    now = time.time()
    duplicated_trees = detect_duplicated_tree(hash_of_node, id_map_of_tree)
    detecting_cost = time.time() - now

    for nodes in duplicated_trees:
        print("=======================================================")
        nodes_count = len(nodes)
        for i, node in enumerate(nodes):
            node = id_map_of_tree[node]
            start = node.start_point.row + 1
            end = node.end_point.row + 1
            print(
                f"{path_of_nodes[node.id]}:{start} {end - start + 1} lines long, cognitive complexity: {cognitive_complexity(node)}"
            )
            print(" " * node.start_point.column + node.text.decode())  # type: ignore
            if i != nodes_count - 1:
                print("-------------------------------------------------------")
    print("#######################################################")
    print(f"Language:\t\t\tRust")
    print(f"Complexity threshold:\t\t{DUPLICATED_THRESHOLD}")
    print(f"Passed files:\t\t\t{len(paths)}")
    print(
        f"Checked lines: \t\t\t{sum(x.root_node.end_point.row - x.root_node.start_point.row for x in trees.values())}"
    )
    print(f"Loaded AST nodes:\t\t{len(id_map_of_tree)}")
    print(f"Duplicated code segment groups:\t{len(duplicated_trees)}")
    print(f"Duplicated code segment places:\t{sum(len(x) for x in duplicated_trees)}")

    print("-------------------------------------------------------")

    print(f"Parsing cost:\t{format(parsing_cost, '.6f')} s")
    print(f"Loading cost:\t{format(loading_cost, '.6f')} s")
    print(f"Hashing cost:\t{format(hashing_cost, '.6f')} s")
    print(f"Detecting cost:\t{format(detecting_cost, '.6f')} s")
    print(
        f"Total cost:\t{format(parsing_cost + loading_cost + hashing_cost + detecting_cost, '.6f')} s"
    )


if __name__ == "__main__":
    main()
