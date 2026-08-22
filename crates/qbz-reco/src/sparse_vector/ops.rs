//! Pure arithmetic operations on `SparseVector` (add/sub/scale/dot/etc).

use super::SparseVector;
use std::collections::HashMap;

impl SparseVector {
    /// Add two sparse vectors element-wise
    pub fn add(&self, other: &SparseVector) -> SparseVector {
        let mut result = HashMap::new();

        // Add all elements from self
        for (idx, val) in self.iter() {
            *result.entry(idx).or_insert(0.0) += val;
        }

        // Add all elements from other
        for (idx, val) in other.iter() {
            *result.entry(idx).or_insert(0.0) += val;
        }

        // Convert back to sparse vector
        let mut indices: Vec<u32> = result.keys().copied().collect();
        indices.sort_unstable();

        let values: Vec<f32> = indices.iter().map(|idx| result[idx]).collect();

        SparseVector { indices, values }
    }

    /// Subtract another vector from this one (self - other)
    pub fn sub(&self, other: &SparseVector) -> SparseVector {
        let mut result = HashMap::new();

        for (idx, val) in self.iter() {
            *result.entry(idx).or_insert(0.0) += val;
        }

        for (idx, val) in other.iter() {
            *result.entry(idx).or_insert(0.0) -= val;
        }

        let mut indices: Vec<u32> = result
            .iter()
            .filter(|(_, v)| v.abs() > 1e-9)
            .map(|(k, _)| *k)
            .collect();
        indices.sort_unstable();

        let values: Vec<f32> = indices.iter().map(|idx| result[idx]).collect();

        SparseVector { indices, values }
    }

    /// Scale vector by a scalar
    pub fn scale(&self, scalar: f32) -> SparseVector {
        SparseVector {
            indices: self.indices().to_vec(),
            values: self.values().iter().map(|v| v * scalar).collect(),
        }
    }

    /// Compute dot product with another sparse vector
    pub fn dot(&self, other: &SparseVector) -> f32 {
        let mut sum = 0.0;
        let mut i = 0;
        let mut j = 0;

        let a = self.indices();
        let b = other.indices();

        // Merge-style iteration (both vectors are sorted by index)
        while i < a.len() && j < b.len() {
            match a[i].cmp(&b[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => {
                    sum += self.values()[i] * other.values()[j];
                    i += 1;
                    j += 1;
                }
            }
        }

        sum
    }

    /// Compute L2 norm (magnitude) of the vector
    pub fn magnitude(&self) -> f32 {
        self.values().iter().map(|v| v * v).sum::<f32>().sqrt()
    }

    /// Normalize the vector to unit length
    ///
    /// Returns a zero vector if magnitude is zero.
    pub fn normalize(&self) -> SparseVector {
        let mag = self.magnitude();
        if mag < 1e-9 {
            return SparseVector::new();
        }

        SparseVector {
            indices: self.indices().to_vec(),
            values: self.values().iter().map(|v| v / mag).collect(),
        }
    }

    /// Compute cosine similarity with another vector
    ///
    /// Returns 0 if either vector has zero magnitude.
    /// Result is in range [-1, 1], where 1 means identical direction.
    ///
    /// NOTE: ported for parity + test coverage, but NOT used for ranking
    /// (production ranks by summed relationship weight; epic decision D3).
    pub fn cosine_similarity(&self, other: &SparseVector) -> f32 {
        let dot = self.dot(other);
        let mag_self = self.magnitude();
        let mag_other = other.magnitude();

        if mag_self < 1e-9 || mag_other < 1e-9 {
            return 0.0;
        }

        dot / (mag_self * mag_other)
    }
}
