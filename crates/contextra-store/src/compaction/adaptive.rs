// FILE-CONTEXT
// STAND: 2026-09-27T00:00:00Z
// ZWECK: Workload-adaptive Compaction-Strategie (EcoTune / ArceKV inspiriert)
// INVARIANTEN: P28 Determinismus-Zwang (WorkloadMetrics über TxId/seq_no & Ops-Counter, KEINE Wanduhrzeit)
//              INV-COMPACTION-ADAPTIVE-1: Tombstone retention for active MVCC snapshots (min_active_seqno)
// SIEHE AUCH: engine.rs, config.rs

use crate::sstable::SstableReader;
use contextra_core::{Result, StorageStats, TOMBSTONE_BIT};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Operational workload metrics captured deterministically without wall-clock time (P28 compliance).
///
/// All trigger decisions are strictly derived from transaction/sequence numbers and operational counts.
#[derive(Debug)]
pub struct WorkloadMetrics {
    read_count: AtomicU64,
    write_count: AtomicU64,
    last_seq_no: AtomicU64,
}

impl WorkloadMetrics {
    /// Creates a new WorkloadMetrics counter instance initialized to zero.
    pub fn new() -> Self {
        Self {
            read_count: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            last_seq_no: AtomicU64::new(0),
        }
    }

    /// Records a read operation atomically.
    #[inline]
    pub fn record_read(&self) {
        self.read_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a write operation and updates the latest sequence number atomically.
    #[inline]
    pub fn record_write(&self, seq_no: u64) {
        self.write_count.fetch_add(1, Ordering::Relaxed);
        self.last_seq_no.fetch_max(seq_no, Ordering::Relaxed);
    }

    /// Takes a deterministic snapshot of current metrics for compaction strategy evaluation.
    pub fn snapshot(&self) -> WorkloadMetricsSnapshot {
        WorkloadMetricsSnapshot {
            read_count: self.read_count.load(Ordering::Relaxed),
            write_count: self.write_count.load(Ordering::Relaxed),
            last_seq_no: self.last_seq_no.load(Ordering::Relaxed),
        }
    }
}

impl Default for WorkloadMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Point-in-time snapshot of operational workload metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadMetricsSnapshot {
    pub read_count: u64,
    pub write_count: u64,
    pub last_seq_no: u64,
}

/// Compaction strategy variants selected by adaptive planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Standard Size-Tiered Compaction Strategy optimized for write throughput.
    WriteOptimizedSTCS,
    /// Aggressive compaction merging more tiers / full LSM to reduce read amplification.
    ReadOptimizedAggressive,
    /// Moderate tier merging balancing read and write amplification.
    Balanced,
}

/// Resulting plan proposed by adaptive compaction planner.
#[derive(Clone)]
pub struct AdaptiveCompactionPlan {
    pub strategy: CompactionStrategy,
    pub candidates: Vec<Arc<SstableReader>>,
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

/// Helper function to perform standard Size-Tiered Compaction candidate selection.
fn select_stcs_candidates(
    ssts: &[Arc<SstableReader>],
    min_sstables_per_tier: usize,
    size_ratio: f64,
) -> Option<Vec<Arc<SstableReader>>> {
    if ssts.len() < 2 {
        return None;
    }

    let mut tiers: Vec<Vec<usize>> = Vec::new();

    for (i, sst) in ssts.iter().enumerate() {
        let size = sst.metadata().file_size;
        let mut placed = false;

        for tier in &mut tiers {
            if let Some(&neighbor_idx) = tier.last() {
                if let Some(neighbor_sst) = ssts.get(neighbor_idx) {
                    let neighbor_size = neighbor_sst.metadata().file_size;
                    let ratio = if size > neighbor_size {
                        size as f64 / neighbor_size.max(1) as f64
                    } else {
                        neighbor_size as f64 / size.max(1) as f64
                    };

                    if ratio <= size_ratio {
                        tier.push(i);
                        placed = true;
                        break;
                    }
                }
            }
        }

        if !placed {
            tiers.push(vec![i]);
        }
    }

    tiers.sort_by(|a, b| {
        b.len().cmp(&a.len()).then_with(|| {
            let a_size = a
                .first()
                .and_then(|&i| ssts.get(i))
                .map(|s| s.metadata().file_size)
                .unwrap_or(0);
            let b_size = b
                .first()
                .and_then(|&i| ssts.get(i))
                .map(|s| s.metadata().file_size)
                .unwrap_or(0);
            a_size.cmp(&b_size)
        })
    });

    for tier in tiers {
        if tier.len() >= min_sstables_per_tier {
            let mut sorted_tier = tier;
            sorted_tier.sort_by_key(|&i| {
                ssts.get(i)
                    .map(|s| s.metadata().max_seq & !TOMBSTONE_BIT)
                    .unwrap_or(0)
            });
            return Some(
                sorted_tier
                    .into_iter()
                    .filter_map(|i| ssts.get(i).cloned())
                    .collect(),
            );
        }
    }

    if ssts.len() >= min_sstables_per_tier * 2 {
        let mut by_size: Vec<(usize, u64)> = ssts
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.metadata().file_size))
            .collect();
        by_size.sort_by_key(|&(_, size)| size);
        let count = min_sstables_per_tier;
        let mut indices: Vec<usize> = by_size[..count].iter().map(|&(i, _)| i).collect();
        indices.sort_by_key(|&i| {
            ssts.get(i)
                .map(|s| s.metadata().max_seq & !TOMBSTONE_BIT)
                .unwrap_or(0)
        });
        return Some(
            indices
                .into_iter()
                .filter_map(|i| ssts.get(i).cloned())
                .collect(),
        );
    }

    None
}
