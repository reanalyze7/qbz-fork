//! Sparse vector implementation for artist similarity
//!
//! Efficient representation for high-dimensional vectors where most values are zero.
//! Used to represent artist relationships where each dimension is another artist.

mod ops;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

/// A sparse vector storing only non-zero values
///
/// Internally uses parallel vectors for indices and values for memory efficiency.
/// For operations, temporarily converts to HashMap for O(1) lookups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SparseVector {
    /// Sorted indices of non-zero elements
    indices: Vec<u32>,
    /// Values corresponding to indices
    values: Vec<f32>,
}

impl SparseVector {
    /// Create an empty sparse vector
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Create a sparse vector with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            indices: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Create from parallel vectors (must be same length, indices should be sorted)
    pub fn from_parts(indices: Vec<u32>, values: Vec<f32>) -> Self {
        debug_assert_eq!(indices.len(), values.len());
        Self { indices, values }
    }

    /// Set a value at the given index
    ///
    /// If the value is 0 (or very close), removes the entry.
    /// If the index exists, updates the value.
    /// If the index doesn't exist, inserts in sorted order.
    pub fn set(&mut self, idx: u32, value: f32) {
        // Ignore near-zero values
        if value.abs() < 1e-9 {
            self.remove(idx);
            return;
        }

        match self.indices.binary_search(&idx) {
            Ok(pos) => {
                // Index exists, update value
                self.values[pos] = value;
            }
            Err(pos) => {
                // Index doesn't exist, insert at sorted position
                self.indices.insert(pos, idx);
                self.values.insert(pos, value);
            }
        }
    }

    /// Get the value at the given index (returns 0 if not present)
    pub fn get(&self, idx: u32) -> f32 {
        match self.indices.binary_search(&idx) {
            Ok(pos) => self.values[pos],
            Err(_) => 0.0,
        }
    }

    /// Remove an entry by index
    pub fn remove(&mut self, idx: u32) {
        if let Ok(pos) = self.indices.binary_search(&idx) {
            self.indices.remove(pos);
            self.values.remove(pos);
        }
    }

    /// Number of non-zero elements
    pub fn nnz(&self) -> usize {
        self.indices.len()
    }

    /// Check if vector is empty (all zeros)
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Get indices of non-zero elements
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Get values of non-zero elements
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// Iterate over (index, value) pairs
    pub fn iter(&self) -> impl Iterator<Item = (u32, f32)> + '_ {
        self.indices
            .iter()
            .copied()
            .zip(self.values.iter().copied())
    }
}
