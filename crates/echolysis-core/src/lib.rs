mod engine;

#[allow(unused)]
pub mod indexed_engine;

pub mod languages;
pub mod utils;

use std::{hash::Hash, ops::Deref};

pub use engine::Engine;
pub use tree_sitter;

/// # Purpose
///
/// This type exists to provide compile-time guarantees and type safety over raw `usize` values.
/// It helps prevent accidental misuse of numeric values where an identifier is expected.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Id {
    id: usize,
}

impl Deref for Id {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl Hash for Id {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<usize> for Id {
    fn from(id: usize) -> Self {
        Self { id }
    }
}

impl Id {
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}
