// FILE-CONTEXT
// STAND: 2026-09-11T10:21:34Z (SESSION: 31ada253)
// ZWECK: Size-Tiered Compaction Strategy (STCS) für SSTables
// INVARIANTEN: Compaction löscht nur Tombstones, die von keinem gepinnten Snapshot mehr benötigt werden
// NICHT-OFFENSICHTLICH: Multi-Version-Merging behält die höchste Sequence Number
// SIEHE AUCH: lsm.rs, sstable.rs, DECISIONS.md

// Background compaction engine for the LSM-Tree.
//
// Implements a Size-Tiered Compaction Strategy (STCS):
// Groups SSTables by size class and merges groups that exceed a threshold.
// Tombstones are garbage-collected during merge when no active snapshot
// references them.

// INVARIANT: Background Compaction (STCS — Size-Tiered Compaction Strategy).
// ALGORITHMUS: Gruppiere SSTables nach Größenklasse → Merge wenn >= min_sstables_per_tier.
// TOMBSTONE-GC: Tombstones werden NUR gelöscht wenn seq < min_active_seqno (MVCC-SAFE).
// ATOMARER SWAP: Merge unter read-lock, SSTable-Liste swap unter write-lock.
// INVARIANTE-COMP-1: Merge-Iterator liest alle Kandidaten-SSTables vollständig. Kein Key-Value-Paar geht verloren.
// INVARIANTE-COMP-2: Tombstone-GC löscht Tombstones nur wenn seq < min_active_snapshot.
// INVARIANTE-COMP-3: Atomarer SSTable-Swap: Alte SSTables bleiben lesbar (über Arc) bis swap, neue Datei ist vollständig fsynced.
// LIFECYCLE: run_loop() -> maybe_compact() -> select_candidates() -> merge_sstables()
//
// Implements a Size-Tiered Compaction Strategy (STCS):
// Groups SSTables by size class and merges groups that exceed a threshold.
// Tombstones are garbage-collected during merge when no active snapshot
// references them.

// FILE-CONTEXT
// STAND: 2026-08-30T21:49:55Z (SESSION: 283abf0f)
// ZWECK:       STCS-Compaction-Engine (Size-Tiered Compaction Strategy)
// INVARIANTEN: Compaction must not block concurrent reads, tombstone GC safe with active snapshot min_seqno, atomic SSTable swap
// HOTSPOTS:    compact_sstables(), merge_sorted_iters()
// SIEHE AUCH:  crates/memfuse-store/AGENTS.md

use std::time::Duration;

/// Configuration for the compaction engine.
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Minimum number of SSTables in a size tier to trigger compaction.
    pub min_sstables_per_tier: usize,
    /// Size ratio between adjacent tiers (e.g., 4.0 means each tier is ~4x the previous).
    pub size_ratio: f64,
    /// Interval between compaction checks.
    pub check_interval: Duration,
    /// Yield execution after this many entries during merge.
    pub yield_threshold: usize,
    /// Maximum memory (in bytes) to use for in-memory buffering during merge.
    pub max_memory_bytes: Option<u64>,
    /// I/O-Rate-Limit für Compaction-Merge-Writes (Token-Bucket, plattformneutral).
    /// `None` = unbegrenzt (Standard).
    /// Bei Aktivierung: Token-Bucket-Delay nach jedem Merge-Block AUSSERHALB
    /// aller MVCC-Write-Locks. Max. 100 ms Delay pro Iteration.
    /// Verhindert NVMe-Queue-Depth-Sättigung die P95-Lese-Latenz von hybrid_search() erhöht.
    pub max_io_bytes_per_second: Option<u64>,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            min_sstables_per_tier: 4,
            size_ratio: 4.0,
            check_interval: Duration::from_secs(30),
            yield_threshold: 1000,
            max_memory_bytes: Some(128 * 1024 * 1024), // 128MB budget by default
            max_io_bytes_per_second: None,
        }
    }
}
