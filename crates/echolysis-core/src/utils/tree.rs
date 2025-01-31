use rustc_hash::FxHashSet;
use tree_sitter::Tree;

use crate::Id;

pub fn preorder_traverse(node: tree_sitter::Node, mut f: impl FnMut(tree_sitter::Node)) {
    let mut cursor = node.walk();
    loop {
        f(cursor.node());
        if cursor.goto_first_child() {
            continue;
        }
        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return;
            }
        }
    }
}

pub fn postorder_traverse(node: tree_sitter::Node, mut f: impl FnMut(tree_sitter::Node)) {
    let mut cursor = node.walk();
    let mut visited = Vec::new();
    loop {
        if visited.last() == Some(&cursor.node().id()) {
            visited.pop();
        } else if cursor.goto_first_child() {
            // SAFETY: We know the parent exists
            visited.push(cursor.node().parent().unwrap().id());
            continue;
        }
        // Visited or no child
        f(cursor.node());
        if !cursor.goto_next_sibling() {
            // no sibling, go to the parent
            if !cursor.goto_parent() {
                // no parent, we're done
                return;
            }
        }
    }
}

pub fn children_set(node: tree_sitter::Node) -> FxHashSet<Id> {
    let mut res: FxHashSet<Id> = FxHashSet::default();
    let mut stack = Vec::from_iter([node]);
    while let Some(node) = stack.pop() {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        res.extend(children.iter().map(|x| Id::from(x.id())));
        stack.extend(children);
    }
    res
}

pub fn tree_diff(lhs: &Tree, rhs: &Tree) -> FxHashSet<Id> {
    let mut lhs_set = FxHashSet::default();
    let mut rhs_set = FxHashSet::default();
    preorder_traverse(lhs.root_node(), |node| {
        lhs_set.insert(Id::from(node.id()));
    });
    preorder_traverse(rhs.root_node(), |node| {
        rhs_set.insert(Id::from(node.id()));
    });
    lhs_set.difference(&rhs_set).copied().collect()
}
