//! # Adaptive Compaction Planner Interface (Ring 1)
//!
//! Definition des Planner-Traits, der Strategien und des Ergebnisplans.

use super::metrics::WorkloadMetricsSnapshot;
use crate::error::StoreResult as Result;
use crate::sstable::SstableReader;
use crate::stats::StorageStats;
use std::sync::Arc;

/// Dynamic compaction strategy chosen by the adaptive planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Strict leveled/tiered size-tiered compaction (fast writes, higher space/read amplification).
    WriteOptimizedSTCS,
    /// Aggressive compaction merging more tiers / full LSM to reduce read amplification.
    ReadOptimizedAggressive,
    /// Moderate tier merging balancing read and write amplification.
    Balanced,
}

/// Resulting plan proposed by adaptive compaction planner.
#[derive(Clone)]
pub struct AdaptiveCompactionPlan {
    /// The chosen compaction strategy.
    pub strategy: CompactionStrategy,
    /// Candidates SSTables to compact.
    pub candidates: Vec<Arc<SstableReader>>,
    /// Whether this run constitutes a full compaction.
    pub is_full_compaction: bool,
}

impl PartialEq for AdaptiveCompactionPlan {
    fn eq(&self, other: &Self) -> bool {
        self.strategy == other.strategy
            && self.is_full_compaction == other.is_full_compaction
            && self.candidates.len() == other.candidates.len()
            && self
                .candidates
                .iter()
                .zip(other.candidates.iter())
                .all(|(a, b)| Arc::ptr_eq(a, b))
    }
}

impl Eq for AdaptiveCompactionPlan {}

impl std::fmt::Debug for AdaptiveCompactionPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdaptiveCompactionPlan")
            .field("strategy", &self.strategy)
            .field("candidates_count", &self.candidates.len())
            .field("is_full_compaction", &self.is_full_compaction)
            .finish()
    }
}

/// Trait defining adaptive compaction strategy selection.
pub trait AdaptiveCompactionPlanner: Send + Sync {
    /// Derives a compaction decision based on storage stats and operational workload metrics.
    fn plan_compaction(
        &self,
        stats: &StorageStats,
        metrics: &WorkloadMetricsSnapshot,
        sstables: &[Arc<SstableReader>],
        min_active_seqno: u64,
    ) -> Result<Option<AdaptiveCompactionPlan>>;
}
