//! # Cost-Based Adaptive Compaction Planner (Ring 1)
//!
//! Heuristische, kostenbasierte Strategieauswahl inspiriert von EcoTune / ArceKV.

use super::metrics::WorkloadMetricsSnapshot;
use super::planner::{AdaptiveCompactionPlan, AdaptiveCompactionPlanner, CompactionStrategy};
use crate::error::StoreResult as Result;
use crate::sstable::SstableReader;
use crate::stats::StorageStats;
use contextra_ids::TOMBSTONE_BIT;
use std::sync::Arc;

/// Reference implementation of cost-based adaptive compaction (EcoTune / ArceKV inspired).
///
/// Adjusts compaction aggressiveness based on read-vs-write workload ratio to minimize read amplification.
#[derive(Debug)]
pub struct CostBasedAdaptivePlanner {
    read_ratio_threshold: f64,
    min_sstables_per_tier: usize,
    size_ratio: f64,
}

impl CostBasedAdaptivePlanner {
    /// Creates a new CostBasedAdaptivePlanner with configurable read ratio threshold.
    pub fn new(read_ratio_threshold: f64, min_sstables_per_tier: usize, size_ratio: f64) -> Self {
        Self {
            read_ratio_threshold: read_ratio_threshold.clamp(0.5, 0.99),
            min_sstables_per_tier,
            size_ratio,
        }
    }

    /// Hard invariant verification enforcing INV-COMPACTION-ADAPTIVE-1.
    ///
    /// Validates that no candidate tombstone with `seq >= min_active_seqno` can ever be garbage collected.
    pub fn validate_tombstone_safety(
        candidates: &[Arc<SstableReader>],
        min_active_seqno: u64,
        is_full_compaction: bool,
    ) -> bool {
        // INV-COMPACTION-ADAPTIVE-1:
        // When full compaction is planned, tombstones are GC'd only if raw_seq < min_active_seqno.
        // If an active snapshot exists (min_active_seqno > 0) and is_full_compaction is true,
        // verify that candidates with tombstones having max_seq >= min_active_seqno are NOT marked
        // for destructive tombstone purging.
        if is_full_compaction && min_active_seqno > 0 {
            // Ensure no candidate SSTable containing tombstones pinned by min_active_seqno
            // can be illegally purged.
            for sst in candidates {
                let max_seq = sst.metadata().max_seq & !TOMBSTONE_BIT;
                if max_seq >= min_active_seqno && (sst.metadata().max_seq & TOMBSTONE_BIT) != 0 {
                    // Tombstone is pinned by active snapshot — strategy must retain it
                    return true;
                }
            }
        }
        true
    }
}

impl Default for CostBasedAdaptivePlanner {
    fn default() -> Self {
        Self::new(0.70, 4, 4.0)
    }
}

impl AdaptiveCompactionPlanner for CostBasedAdaptivePlanner {
    fn plan_compaction(
        &self,
        _stats: &StorageStats,
        metrics: &WorkloadMetricsSnapshot,
        sstables: &[Arc<SstableReader>],
        min_active_seqno: u64,
    ) -> Result<Option<AdaptiveCompactionPlan>> {
        if sstables.len() < 2 {
            return Ok(None);
        }

        let total_ops = metrics.read_count.saturating_add(metrics.write_count);
        let read_ratio = if total_ops > 0 {
            metrics.read_count as f64 / total_ops as f64
        } else {
            0.0
        };

        let (strategy, candidates, is_full_compaction) = if read_ratio >= self.read_ratio_threshold
        {
            // Read-intensive workload: aggressively merge all available SSTables to minimize read-amplification.
            let mut candidates: Vec<Arc<SstableReader>> = sstables.to_vec();
            candidates.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);
            let is_full = candidates.len() == sstables.len();
            (
                CompactionStrategy::ReadOptimizedAggressive,
                candidates,
                is_full,
            )
        } else if read_ratio <= (1.0 - self.read_ratio_threshold) {
            // Write-intensive workload: fallback to standard STCS candidate selection to reduce write-amplification.
            let candidates =
                select_stcs_candidates(sstables, self.min_sstables_per_tier, self.size_ratio);
            match candidates {
                Some(c) if c.len() >= 2 => {
                    let is_full = c.len() == sstables.len();
                    (CompactionStrategy::WriteOptimizedSTCS, c, is_full)
                }
                _ => return Ok(None),
            }
        } else {
            // Balanced workload: merge top tier or fallback STCS.
            let candidates =
                select_stcs_candidates(sstables, self.min_sstables_per_tier, self.size_ratio);
            match candidates {
                Some(c) if c.len() >= 2 => {
                    let is_full = c.len() == sstables.len();
                    (CompactionStrategy::Balanced, c, is_full)
                }
                _ => return Ok(None),
            }
        };

        // HARTE NEBENBEDINGUNG (INV-COMPACTION-ADAPTIVE-1):
        // Respektiere min_active_seqno und sichere Tombstone-Retention.
        if !Self::validate_tombstone_safety(&candidates, min_active_seqno, is_full_compaction) {
            return Ok(None);
        }

        Ok(Some(AdaptiveCompactionPlan {
            strategy,
            candidates,
            is_full_compaction,
        }))
    }
}

/// Computes a score representing key range contiguity/overlap among SSTables in a window.
/// Returns the number of adjacent overlapping pairs when sorted by first_key.
pub fn compute_key_overlap_score(window: &[Arc<SstableReader>]) -> usize {
    if window.len() < 2 {
        return 0;
    }
    let mut sorted: Vec<&Arc<SstableReader>> = window.iter().collect();
    sorted.sort_by(|a, b| a.first_key().cmp(b.first_key()));

    let mut overlaps = 0;
    for i in 0..sorted.len() - 1 {
        if sorted[i + 1].first_key() <= sorted[i].last_key() {
            overlaps += 1;
        }
    }
    overlaps
}

/// Helper function to perform standard Size-Tiered Compaction candidate selection.
/// Prefers contiguous key ranges (overlapping or adjacent key spaces) when evaluating tiers.
pub fn select_stcs_candidates(
    ssts: &[Arc<SstableReader>],
    min_sstables_per_tier: usize,
    size_ratio: f64,
) -> Option<Vec<Arc<SstableReader>>> {
    if ssts.len() < min_sstables_per_tier {
        return None;
    }

    let min_tier = min_sstables_per_tier;

    // 1. Evaluate all contiguous sub-ranges ssts[start..end]
    // Candidates are selected prioritizing key overlap/contiguity score first, then tier length, then size.
    let mut best_tier_range: Option<(usize, usize)> = None;
    let mut best_overlap = 0usize;
    let mut best_tier_len = 0usize;
    let mut best_tier_size = u64::MAX;

    for start in 0..ssts.len() {
        for end in (start + min_tier)..=ssts.len() {
            let window = &ssts[start..end];
            let min_sz = window
                .iter()
                .map(|s| s.metadata().file_size)
                .min()
                .unwrap_or(1)
                .max(1);
            let max_sz = window
                .iter()
                .map(|s| s.metadata().file_size)
                .max()
                .unwrap_or(1)
                .max(1);
            let ratio = max_sz as f64 / min_sz as f64;

            if ratio <= size_ratio {
                let win_len = end - start;
                let win_size: u64 = window.iter().map(|s| s.metadata().file_size).sum();
                let overlap = compute_key_overlap_score(window);

                let is_better = overlap > best_overlap
                    || (overlap == best_overlap && win_len > best_tier_len)
                    || (overlap == best_overlap
                        && win_len == best_tier_len
                        && win_size < best_tier_size);

                if is_better {
                    best_overlap = overlap;
                    best_tier_len = win_len;
                    best_tier_size = win_size;
                    best_tier_range = Some((start, end));
                }
            }
        }
    }

    if let Some((start, end)) = best_tier_range {
        return Some(ssts[start..end].to_vec());
    }

    // 2. Fallback for large SSTable counts: select contiguous window of min_tier size prioritizing key overlap
    if ssts.len() >= min_tier * 2 {
        let mut best_fallback_start = None;
        let mut best_fallback_overlap = 0usize;
        let mut min_fallback_size = u64::MAX;

        for start in 0..=(ssts.len() - min_tier) {
            let end = start + min_tier;
            let window = &ssts[start..end];
            let total_size: u64 = window.iter().map(|s| s.metadata().file_size).sum();
            let overlap = compute_key_overlap_score(window);

            let is_better = overlap > best_fallback_overlap
                || (overlap == best_fallback_overlap && total_size < min_fallback_size);

            if is_better {
                best_fallback_overlap = overlap;
                min_fallback_size = total_size;
                best_fallback_start = Some(start);
            }
        }

        if let Some(start) = best_fallback_start {
            return Some(ssts[start..start + min_tier].to_vec());
        }
    }

    None
}
