use rustc_hash::FxHashSet;
use tree_sitter::{Node, Tree};

use crate::Id;

pub trait NodeExt: Traverse {
    fn text<'a>(&self, source: &'a [u8]) -> &'a str;
    fn all_children(&self) -> FxHashSet<Id>;
}

impl NodeExt for Node<'_> {
    fn text<'a>(&self, source: &'a [u8]) -> &'a str {
        self.utf8_text(source).unwrap()
    }

    fn all_children(&self) -> FxHashSet<Id> {
        let mut res: FxHashSet<Id> = FxHashSet::default();
        let mut stack = vec![*self];
        while let Some(node) = stack.pop() {
            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();
            res.extend(children.iter().map(|x| Id::from(x.id())));
            stack.extend(children);
        }
        res
    }
}

pub trait Traverse {
    fn preorder_traverse(&self, f: impl FnMut(Node));
    fn postorder_traverse(&self, f: impl FnMut(Node));
}

impl Traverse for Tree {
    fn preorder_traverse(&self, f: impl FnMut(Node)) {
        self.root_node().preorder_traverse(f);
    }

    fn postorder_traverse(&self, f: impl FnMut(Node)) {
        self.root_node().postorder_traverse(f);
    }
}

impl Traverse for Node<'_> {
    fn preorder_traverse(&self, mut f: impl FnMut(Node)) {
        let mut cursor = self.walk();
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

    fn postorder_traverse(&self, mut f: impl FnMut(Node)) {
        let mut cursor = self.walk();
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
}

pub trait TreeExt: Traverse {
    fn diff(&self, rhs: &Tree) -> FxHashSet<Id>;
}

impl TreeExt for Tree {
    fn diff(&self, rhs: &Tree) -> FxHashSet<Id> {
        let mut lhs_set = FxHashSet::default();
        let mut rhs_set = FxHashSet::default();
        self.preorder_traverse(|node| {
            lhs_set.insert(Id::from(node.id()));
        });
        rhs.postorder_traverse(|node| {
            rhs_set.insert(Id::from(node.id()));
        });
        lhs_set.difference(&rhs_set).copied().collect()
    }
}
