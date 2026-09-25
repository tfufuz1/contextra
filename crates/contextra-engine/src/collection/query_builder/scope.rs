// FILE-CONTEXT
// ZWECK: Scoping-Typen und Query-Builder Ergänzung für ACORN-Hard-Boundary (Spec §5.3, §8.5, §10.4).
// INVARIANTEN: allowed_doc_ids als BTreeSet für deterministische Iteration (P-Systeminvariante 3).

use std::collections::BTreeSet;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::DocId;
use super::builder::HybridQueryBuilder;

/// Scoping constraint defining an explicit hard-boundary set of allowed document IDs
/// for predicate-agnostic vector search (ACORN-Hard-Boundary, Spec §5.3, §8.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeConstraint {
    /// Explicit set of allowed document IDs (BTreeSet guarantees deterministic ordering).
    pub allowed_doc_ids: BTreeSet<DocId>,
    /// Gamma augmentation factor for edge budget during ACORN graph traversal.
    /// Default: 2 (per Spec §8.5 example "γ=2").
    pub gamma: u32,
}

impl ScopeConstraint {
    /// Creates a new `ScopeConstraint` with a custom `gamma` factor.
    pub fn new(allowed_doc_ids: BTreeSet<DocId>, gamma: u32) -> Self {
        Self {
            allowed_doc_ids,
            gamma,
        }
    }

    /// Creates a new `ScopeConstraint` with default `gamma = 2` (Spec §8.5).
    pub fn from_allowed_ids(allowed_doc_ids: BTreeSet<DocId>) -> Self {
        Self {
            allowed_doc_ids,
            gamma: 2,
        }
    }
}

impl Default for ScopeConstraint {
    /// Default `ScopeConstraint` uses empty document set and default `gamma = 2` (Spec §8.5 example "γ=2").
    fn default() -> Self {
        Self {
            allowed_doc_ids: BTreeSet::new(),
            gamma: 2,
        }
    }
}

impl<'a, S: StorageEngine, V: VectorIndex> HybridQueryBuilder<'a, S, V> {
    /// Sets an explicit ACORN hard-boundary scope constraint for vector retrieval.
    ///
    /// Spec §5.3: `.scope(ScopeConstraint)` enforces strict document boundary scoping
    /// via ACORN graph traversal, preventing zero-cross-contamination leaks.
    pub fn scope(mut self, constraint: ScopeConstraint) -> Self {
        self.hard_scope = Some(constraint);
        self
    }
}
