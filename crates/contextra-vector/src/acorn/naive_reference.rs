// FILE-CONTEXT
// ZWECK: Naive O(n) Referenzimplementierung für ACORN Recall-Baseline-Vergleiche.
// INVARIANTEN: NICHT für Produktion — O(n) Referenzimplementierung.

// NICHT für Produktion — O(n) Referenzimplementierung

use contextra_core::{DistanceMetric, DocId};

use crate::distance::compute_distance;

use super::{AcornError, FilteredIndex};

/// Naive reference index performing brute-force search over all stored points.
///
/// **NICHT für Produktion — O(n) Referenzimplementierung**
///
/// Serves exclusively as a ground-truth baseline for ACORN graph traversal correctness
/// and recall benchmarking.
#[derive(Debug, Clone)]
pub struct NaiveReferenceIndex {
    vectors: Vec<(DocId, Vec<f32>)>,
    metric: DistanceMetric,
}

impl NaiveReferenceIndex {
    /// Creates a new empty `NaiveReferenceIndex` with the specified distance metric.
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            vectors: Vec::new(),
            metric,
        }
    }

    /// Creates a `NaiveReferenceIndex` pre-populated with vectors using Euclidean distance.
    pub fn from_vectors(vectors: Vec<(DocId, Vec<f32>)>) -> Self {
        Self {
            vectors,
            metric: DistanceMetric::Euclidean,
        }
    }

    /// Inserts a document vector into the index.
    pub fn insert(&mut self, id: DocId, vector: Vec<f32>) {
        self.vectors.push((id, vector));
    }
}

impl FilteredIndex for NaiveReferenceIndex {
    type Predicate = dyn Fn(DocId) -> bool;

    fn search_knn_acorn(
        &self,
        query: &[f32],
        k: usize,
        predicate: &Self::Predicate,
        _gamma: u32,
    ) -> Result<Vec<(DocId, f32)>, AcornError> {
        if k == 0 {
            return Err(AcornError::InvalidK { k });
        }

        let mut matches = Vec::new();

        for (id, vec) in &self.vectors {
            if vec.len() != query.len() {
                return Err(AcornError::DimensionMismatch {
                    expected: query.len(),
                    got: vec.len(),
                });
            }

            if predicate(*id) {
                let dist = compute_distance(query, vec, self.metric).map_err(|e| {
                    AcornError::Internal(format!("Distance computation failed: {e}"))
                })?;
                matches.push((*id, dist));
            }
        }

        matches.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        matches.truncate(k);
        Ok(matches)
    }
}
