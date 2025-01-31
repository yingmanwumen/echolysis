use std::ops::{Deref, DerefMut};

use dashmap::{DashMap, DashSet};
use rustc_hash::FxBuildHasher;

pub struct ADashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    inner: DashMap<K, V, ahash::RandomState>,
}

impl<K, V> Deref for ADashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    type Target = DashMap<K, V, ahash::RandomState>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V> DerefMut for ADashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K, V> Default for ADashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self {
            inner: DashMap::with_hasher(ahash::RandomState::default()),
        }
    }
}

impl<K, V> From<DashMap<K, V, ahash::RandomState>> for ADashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn from(value: DashMap<K, V, ahash::RandomState>) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug)]
pub struct FxDashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    inner: DashMap<K, V, FxBuildHasher>,
}

impl<K, V> Deref for FxDashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    type Target = DashMap<K, V, FxBuildHasher>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V> DerefMut for FxDashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K, V> Default for FxDashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self {
            inner: DashMap::with_hasher(FxBuildHasher),
        }
    }
}

pub struct FxDashSet<K>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    inner: DashSet<K, FxBuildHasher>,
}

impl<K> Deref for FxDashSet<K>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    type Target = DashSet<K, FxBuildHasher>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K> DerefMut for FxDashSet<K>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K> Default for FxDashSet<K>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self {
            inner: DashSet::with_hasher(FxBuildHasher),
        }
    }
}

/// Merges two 64-bit hash values into a single 64-bit hash value.
///
/// This function takes two 64-bit hash values, `lhs` and `rhs`, and combines them
/// using a specific algorithm to produce a single 64-bit hash value. The algorithm
/// involves using a seed value, multiplying and XORing the input values, and applying
/// a final adjustment to ensure the result is not equal to `u64::MAX`.
///
/// # Arguments
///
/// * `lhs` - The first 64-bit hash value.
/// * `rhs` - The second 64-bit hash value.
///
/// # Returns
///
/// A 64-bit hash value that represents the combined hash of `lhs` and `rhs`.
pub fn merge_structure_hash(lhs: u64, rhs: u64) -> u64 {
    let seed: u64 = 0x12345678;
    let mut value = seed.wrapping_mul(1000003) ^ (lhs + 0x01);
    value = value.wrapping_mul(1000003) ^ (rhs + 0x02);
    value ^= 2;
    if value == u64::MAX {
        u64::MAX - 1
    } else {
        value
    }
}
