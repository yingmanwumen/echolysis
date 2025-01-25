#!/usr/bin/env python

import sys
import time
from typing import Dict, List, Set, Tuple
from tree_sitter import Language, Node, Parser, Tree, Query
import tree_sitter_python

NATIVE_ENCODING = sys.getfilesystemencoding()


def parse_file(path: str, parser: Parser) -> Tree:
    try:
        with open(path) as f:
            return parser.parse(bytes(f.read(), NATIVE_ENCODING))
    except Exception as e:
        print("Failed to parse: " + path)
        raise e


def merkle_hash(
    trees: Dict[str, "Tree"], query_of_node: Dict[str, Dict[int, str]]
) -> Dict[int, int]:
    res = {}

    control_keywords = {"if_statement", "for_statement", "while_statement", "with"}

    def do_merkle_hash(
        node: "Node", record: Dict[int, int], query_of_node: Dict[int, str]
    ) -> int:
        # Skip comment
        if node.grammar_name == "comment":
            return 0

        if node.child_count == 0:
            node_hash = hash(node.text)
            if node.id in query_of_node and query_of_node[node.id] == "variable":
                node_hash = hash(query_of_node[node.id])
            return node_hash

        combined_hash = 0
        for child in node.children:
            child_hash = do_merkle_hash(child, record, query_of_node)
            combined_hash = hash((combined_hash, child_hash))  # 避免简单加法哈希冲突

        # Only record control sentences , because we don't care about normal statements
        if node.grammar_name in control_keywords:
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


def statement_count_of_children(node: Node) -> int:
    count = 0
    for child in node.children:
        if "statement" in child.grammar_name:
            count += 1
        count += statement_count_of_children(child)
    return count


def detect_duplicated_tree(
    hash_of_node: Dict[int, int], id_map: Dict[int, Node]
) -> List[List[int]]:
    hash_to_nodes: Dict[int, List[int]] = {}
    for node_id, hash in hash_of_node.items():
        name = id_map[node_id].grammar_name
        if name == "comment":
            continue
        if hash not in hash_to_nodes:
            hash_to_nodes[hash] = []
        if statement_count_of_children(id_map[node_id]) < 3:
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
    py_language = Language(tree_sitter_python.language())
    py_parser = Parser(py_language)
    py_query = Query(py_language, tree_sitter_python.HIGHLIGHTS_QUERY)
    trees: Dict[str, Tree] = {}
    query_to_nodes: Dict[str, Dict[str, List[Node]]] = {}
    query_of_node: Dict[str, Dict[int, str]] = {}
    for path in paths:
        trees[path] = parse_file(path, py_parser)
        query_to_nodes[path] = py_query.captures(trees[path].root_node)
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
        for node in nodes:
            node = id_map_of_tree[node]
            print(
                f"{path_of_nodes[node.id]}({node.start_point.row + 1}~{node.end_point.row + 1})"
            )
            print(" " * node.start_point.column + node.text.decode())  # type: ignore
            print("-------------------------------------------------------")
    print("#######################################################")
    print(f"{len(paths)} files passed")
    print(
        f"{sum(x.root_node.end_point.row - x.root_node.start_point.row for x in trees.values())} lines checked"
    )
    print(len(duplicated_trees), "duplicated code segments found")

    print("Parsing cost:", parsing_cost)
    print("Loading cost:", loading_cost)
    print("Hashing cost:", hashing_cost)
    print("Detecting cost:", detecting_cost)
    print("Total cost:", parsing_cost + loading_cost + hashing_cost + detecting_cost)


if __name__ == "__main__":
    main()
