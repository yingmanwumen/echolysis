#!/usr/bin/env python

from collections import defaultdict, deque
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
    """Parse a Rust source file into an AST using tree-sitter

    Args:
        path: Path to the Rust source file
        parser: Configured tree-sitter parser

    Returns:
        Tree: The parsed syntax tree

    Raises:
        Exception: If parsing fails
    """
    try:
        with open(path) as f:
            return parser.parse(bytes(f.read(), NATIVE_ENCODING))
    except Exception as e:
        print("Failed to parse: " + path)
        raise e


def hash_node(node: Node, query: Optional[str]) -> int:
    """Generate a hash for an AST node considering its type and context

    Args:
        node: The AST node to hash
        query: Optional query pattern that matched this node

    Returns:
        int: A hash value representing the node's content and context
    """
    # Directly hash the node's byte content without creating new bytes object
    if query:
        if query in RUST_QUERY_NOT_TO_OBFUSCATE:
            return hash(node.text)
        if query in RUST_QUERY_TO_OBFUSCATE:
            return hash(query)

    if node.type in RUST_NODES_TO_OBFUSCATE:
        return hash(node.type)

    return hash(node.text)


def child_set(node: Node) -> Set[Node]:
    """Get all descendant nodes of a given node

    Args:
        node: The root node to collect children from

    Returns:
        Set[Node]: All descendant nodes including immediate children
    """
    res = set()
    stack = deque([node])
    while stack:
        n = stack.pop()
        res.update(n.children)
        stack.extend(n.children)
    return res


def cognitive_complexity(node: Node) -> float:
    """Calculate the cognitive complexity of a code segment

    Args:
        node: The root node of the code segment

    Returns:
        float: The computed complexity score based on node types
    """
    res = 0.0
    stack = deque([node])
    while stack:
        n = stack.pop()
        stack.extend(n.children)
        res += RUST_COGNITIVE_NODES.get(n.type, 0)
    return res


def detect_duplicated_tree(
    hash_of_node: Dict[int, int], id_map: Dict[int, Node]
) -> List[List[int]]:
    """Detect duplicated code segments based on node hashes and complexity

    Args:
        hash_of_node: Mapping of node IDs to their hashes
        id_map: Mapping of node IDs to their corresponding nodes

    Returns:
        List[List[int]]: Groups of node IDs representing duplicate code segments
    """
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


def compute_hash_and_collect_data(
    trees: Dict[str, Tree], query_of_node: Dict[str, Dict[int, str]]
) -> Tuple[Dict[int, int], Dict[int, Node], Dict[str, Set[int]]]:
    hash_of_node = {}
    id_map = {}
    path_map = defaultdict(set)

    def do_merkle_hash(
        node: Node, record: Dict[int, int], query_of_node: Dict[int, str], path: str
    ) -> int:
        if node.type in RUST_NODES_TO_SKIP or node.is_missing:
            return 0

        id_map[node.id] = node
        path_map[path].add(node.id)

        if node.child_count == 0:
            return hash_node(node, query_of_node.get(node.id))

        combined_hash = 0
        for child in node.children:
            child_hash = do_merkle_hash(child, record, query_of_node, path)
            combined_hash = hash((combined_hash, child_hash))

        if node.type in RUST_COMPLEX_STATEMENT:
            record[node.id] = combined_hash

        return combined_hash

    for path, tree in trees.items():
        do_merkle_hash(tree.root_node, hash_of_node, query_of_node.get(path, {}), path)

    return hash_of_node, id_map, path_map


def main():
    """Main entry point for the Rust code analysis tool

    Processes Rust source files to detect duplicate code segments and calculate complexity metrics
    """
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
    hash_of_node, id_map_of_tree, path_map = compute_hash_and_collect_data(
        trees, query_of_node
    )
    loading_cost = time.time() - now

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
            node_path = next(
                path for path, node_ids in path_map.items() if node.id in node_ids
            )
            print(
                f"{node_path}:{start} {end - start + 1} lines long, cognitive complexity: {cognitive_complexity(node)}"
            )
            print(" " * node.start_point.column + node.text.decode(NATIVE_ENCODING))  # type: ignore
            if i != nodes_count - 1:
                print("-------------------------------------------------------")
    total_files = len(paths)
    total_lines = sum(
        x.root_node.end_point.row - x.root_node.start_point.row for x in trees.values()
    )
    print("#######################################################")
    print(f"Language:\t\t\tRust")
    print(f"Complexity threshold:\t\t{DUPLICATED_THRESHOLD}")
    print(f"Passed files:\t\t\t{total_files}")
    print(f"Checked lines: \t\t\t{total_lines}")
    print(f"Loaded AST nodes:\t\t{len(id_map_of_tree)}")
    print(f"Duplicated code segment groups:\t{len(duplicated_trees)}")
    print(f"Duplicated code segment places:\t{sum(len(x) for x in duplicated_trees)}")

    print("-------------------------------------------------------")

    print(f"Parsing cost:\t{format(parsing_cost, '.6f')} s")
    print(f"Loading cost:\t{format(loading_cost, '.6f')} s")
    print(f"Detecting cost:\t{format(detecting_cost, '.6f')} s")
    print(
        f"Total cost:\t{format(parsing_cost + loading_cost + detecting_cost, '.6f')} s"
    )
    files_per_second = total_files / (parsing_cost + loading_cost + detecting_cost)
    lines_per_second = total_lines / (parsing_cost + loading_cost + detecting_cost)
    print(f"Files per sec:\t{format(files_per_second, '.6f')}")
    print(f"Lines per sec:\t{format(lines_per_second, '.6f')}")


if __name__ == "__main__":
    main()
