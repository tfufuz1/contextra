// FILE-CONTEXT
// STAND: 2026-10-06T00:00:00Z (SESSION: p02a-fix)
// ZWECK: Tiered/Leveled CompactionEngine & Multi-Way SSTable Merging mit MVCC-Snapshot-Watermark-Retention.
// INVARIANTEN: GC/Tombstone-Entfernung muss bound = min(snapshot_registry.min_active_seqno(), tx_buffer.min_read_snapshot()) einhalten (Invariante I-2).
// HOTSPOTS: 150-250, 400-750

use super::adaptive::{AdaptiveCompactionPlanner, CostBasedAdaptivePlanner, WorkloadMetrics};
use super::config::CompactionConfig;
use super::merge_operator::MergeOperator;
use crate::sstable::{BlockCache, SstableBuilder, SstableReader};
use crate::wal::KeyManager;
use contextra_core::{Result, StorageStats, TOMBSTONE_BIT};
use contextra_mvcc::snapshot::SnapshotFloor;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;

static COMPACTION_SST_COUNTER: AtomicU64 = AtomicU64::new(1);
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

pub struct CompactionEngine {
    config: CompactionConfig,
    pub floor: Arc<dyn SnapshotFloor>,
    block_cache: Arc<BlockCache>,
    key_manager: Option<Arc<KeyManager>>,
    budget: Arc<contextra_core::ResourceTracker>,
    manifest: Option<Arc<crate::manifest::Manifest>>,
    compaction_counter: AtomicU64,
    pressure_rx: Option<tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>>,
    workload_metrics: WorkloadMetrics,
    adaptive_planner: Option<Arc<dyn AdaptiveCompactionPlanner>>,
    merge_operator: Option<Arc<dyn MergeOperator>>,
    clock: Arc<dyn contextra_ports::Clock>,
}

impl CompactionEngine {
    /// Creates a new compaction engine.
    pub fn new(
        config: CompactionConfig,
        floor: Arc<dyn SnapshotFloor>,
        block_cache: Arc<BlockCache>,
        key_manager: Option<Arc<KeyManager>>,
        budget: Arc<contextra_core::ResourceTracker>,
        manifest: Option<Arc<crate::manifest::Manifest>>,
    ) -> Self {
        let adaptive_planner: Option<Arc<dyn AdaptiveCompactionPlanner>> =
            if config.enable_adaptive_compaction {
                Some(Arc::new(CostBasedAdaptivePlanner::new(
                    config.adaptive_read_ratio_threshold,
                    config.min_sstables_per_tier,
                    config.size_ratio,
                )))
            } else {
                None
            };

        Self {
            config,
            floor,
            block_cache,
            key_manager,
            budget,
            manifest,
            compaction_counter: AtomicU64::new(0),
            pressure_rx: None,
            workload_metrics: WorkloadMetrics::new(),
            adaptive_planner,
            merge_operator: None,
            clock: Arc::new(contextra_ports::SystemClock::new()),
        }
    }

    /// Attaches a custom merge operator for compaction value merging (§4.12).
    pub fn with_merge_operator(mut self, merge_operator: Arc<dyn MergeOperator>) -> Self {
        self.merge_operator = Some(merge_operator);
        self
    }

    /// Attaches a custom clock port for deterministic execution timing (P28).
    pub fn with_clock(mut self, clock: Arc<dyn contextra_ports::Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Attaches a system pressure watch receiver to enable pressure-aware compaction backpressure.
    pub fn with_pressure_rx(
        mut self,
        rx: tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>,
    ) -> Self {
        self.pressure_rx = Some(rx);
        self
    }

    /// Attaches a custom adaptive compaction planner.
    // TODO(wiring): Facade-Anbindung in contextra/src/builder.rs folgt in separatem Task
    pub fn with_adaptive_planner(mut self, planner: Arc<dyn AdaptiveCompactionPlanner>) -> Self {
        self.adaptive_planner = Some(planner);
        self
    }

    /// Records a read operation for workload tracking.
    #[inline]
    pub fn record_read_op(&self) {
        self.workload_metrics.record_read();
    }

    /// Records a write operation for workload tracking.
    #[inline]
    pub fn record_write_op(&self, seq_no: u64) {
        self.workload_metrics.record_write(seq_no);
    }

    /// Calculates the effective lower sequence bound for MVCC tombstone/version GC (Invariante I-2).
    #[inline]
    pub fn min_snapshot_seq_bound(&self) -> u64 {
        self.floor.floor()
    }

    /// Evaluates whether compaction should run and performs it if needed.
    ///
    /// Takes a write-lock on the SSTable list to atomically swap old SSTables
    /// for the compacted result.
    // AI-TAG[SMELL][RESOLVED] audit-H-3: LsmStorage hält eine persistente CompactionEngine-Instanz (LsmStorage.compaction_engine), wodurch Compaction-State & Zähler erhalten bleiben.
    pub async fn maybe_compact(
        &self,
        sstables: &RwLock<Vec<Arc<SstableReader>>>,
        data_path: &std::path::Path,
    ) -> Result<bool> {
        self.maybe_compact_with_cancel(sstables, data_path, None)
            .await
    }

    /// Evaluates whether compaction should run and performs it with an optional cancellation token.
    pub async fn maybe_compact_with_cancel(
        &self,
        sstables: &RwLock<Vec<Arc<SstableReader>>>,
        data_path: &std::path::Path,
        cancel_token: Option<&tokio_util::sync::CancellationToken>,
    ) -> Result<bool> {
        // 1. Select candidates under a single read-lock window.
        // Both candidate decision, Arc cloning, and full-compaction determination
        // happen atomically under one lock acquisition to prevent TOCTOU race conditions.
        let (mut input_ssts, is_full_compaction) = {
            let ssts = sstables.read().await;
            if ssts.len() < self.config.min_sstables_per_tier {
                return Ok(false);
            }

            if self.config.enable_adaptive_compaction {
                if let Some(ref planner) = self.adaptive_planner {
                    let total_size_bytes = ssts.iter().map(|s| s.metadata().file_size).sum();
                    let stats = StorageStats {
                        num_segments: ssts.len(),
                        total_size_bytes,
                        memtable_size_bytes: 0,
                    };
                    let metrics_snap = self.workload_metrics.snapshot();
                    let min_seq = self.min_snapshot_seq_bound();

                    if let Some(plan) =
                        planner.plan_compaction(&stats, &metrics_snap, &ssts, min_seq)?
                    {
                        if plan.candidates.len() >= 2 {
                            (plan.candidates, plan.is_full_compaction)
                        } else {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                } else {
                    match self.select_compaction_candidates(&ssts) {
                        Some(candidates) if candidates.len() >= 2 => {
                            let is_full = candidates.len() == ssts.len();
                            (candidates, is_full)
                        }
                        _ => return Ok(false),
                    }
                }
            } else {
                match self.select_compaction_candidates(&ssts) {
                    Some(candidates) if candidates.len() >= 2 => {
                        let is_full = candidates.len() == ssts.len();
                        (candidates, is_full)
                    }
                    _ => return Ok(false),
                }
            }
        };

        input_ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        tracing::info!(
            "Compaction triggered: merging {} SSTables",
            input_ssts.len()
        );

        // 1b. Peak Memory Estimation Check
        if let Some(limit_bytes) = self.config.max_peak_memory_bytes {
            let estimated_peak = self.estimate_compaction_peak_bytes(&input_ssts);
            if estimated_peak > limit_bytes {
                let used_mb = estimated_peak.div_ceil(1024 * 1024);
                let limit_mb = limit_bytes.div_ceil(1024 * 1024);
                tracing::warn!(
                    "Compaction aborted: estimated peak memory {} bytes ({}MB) exceeds max_peak_memory_bytes limit {} bytes ({}MB)",
                    estimated_peak,
                    used_mb,
                    limit_bytes,
                    limit_mb
                );
                return Err(contextra_core::ContextraError::MemoryBudgetExceeded {
                    used_mb,
                    limit_mb,
                });
            }
        }

        // 2. Perform the merge (no lock held — this is the expensive part)
        // Invariante I-2: Minimum aus snapshot_registry.min_active_seqno() UND tx_buffer.min_read_snapshot()
        let min_snapshot_seq = self.min_snapshot_seq_bound();
        let output_path = self.generate_sst_path(data_path)?;
        self.merge_sstables_with_cancel(
            &input_ssts,
            &output_path,
            min_snapshot_seq,
            is_full_compaction,
            cancel_token,
        )
        .await?;

        // Explicit fsync of output file and parent directory
        crate::util::fsync_parent_dir(&output_path).await?;

        // Open the new SSTable reader
        let new_reader = Arc::new(
            SstableReader::open_with_key_manager(
                &output_path,
                Arc::clone(&self.block_cache),
                self.key_manager.clone(),
            )
            .await?,
        );

        // 4. In-memory SSTable swap under write-lock with MANIFEST durability
        let old_paths = {
            let mut ssts = sstables.write().await;

            let start_opt = ssts.iter().position(|s| Arc::ptr_eq(s, &input_ssts[0]));
            let is_contiguous = match start_opt {
                Some(start) => {
                    if start + input_ssts.len() <= ssts.len() {
                        input_ssts
                            .iter()
                            .enumerate()
                            .all(|(idx, inp)| Arc::ptr_eq(inp, &ssts[start + idx]))
                    } else {
                        false
                    }
                }
                None => false,
            };

            if !is_contiguous {
                tracing::warn!(
                    "Compaction aborted under write lock: input SSTables are no longer contiguous or modified \
                     (concurrent flush or rollback detected)"
                );
                let p = output_path.clone();
                let res =
                    tokio::task::spawn_blocking(move || contextra_durable_fs::durable_remove(&p))
                        .await;
                if let Ok(Err(e)) = res {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        tracing::warn!(
                            "Failed to clean up aborted compaction output {:?}: {}",
                            output_path,
                            e
                        );
                    }
                }
                return Ok(false);
            }

            let insertion_point = start_opt.unwrap_or(0);
            let old_paths: Vec<PathBuf> = input_ssts
                .iter()
                .map(|s| s.file_path().to_path_buf())
                .collect();

            // 5. Write EXACTLY ONE atomic `Replace` entry to MANIFEST before swapping in-memory state
            if let Some(ref manifest) = self.manifest {
                manifest
                    .append(&crate::manifest::ManifestEntry::Replace {
                        removed: old_paths.clone(),
                        added: output_path.clone(),
                        added_max_tx: new_reader.metadata().max_tx_id,
                        rank: 0,
                    })
                    .await?;
            }

            // Remove input SSTables by identity (Arc::ptr_eq), which form a contiguous window
            ssts.retain(|sst| !input_ssts.iter().any(|inp| Arc::ptr_eq(inp, sst)));

            // Add new SSTable at insertion point
            let insert_idx = insertion_point.min(ssts.len());
            ssts.insert(insert_idx, new_reader);

            // Re-sort SSTable list by max_seq to guarantee shadowing/visibility order.
            ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

            debug_assert!(
                ssts.windows(2)
                    .all(|w| (w[0].metadata().max_seq & !TOMBSTONE_BIT)
                        <= (w[1].metadata().max_seq & !TOMBSTONE_BIT)),
                "SSTable list must be sorted by max_seq in ascending order after compaction swap"
            );

            old_paths
        };

        // 7. Delete old SSTable files (best-effort cleanup outside lock)
        for path in &old_paths {
            let p = path.clone();
            let res =
                tokio::task::spawn_blocking(move || contextra_durable_fs::durable_remove(&p))
                    .await;
            if let Ok(Err(e)) = res {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!("Failed to delete compacted SSTable {:?}: {}", path, e);
                }
            }
            let uuid_sidecar = PathBuf::from(format!("{}.uuid", path.display()));
            let sidecar = uuid_sidecar.clone();
            let sidecar_res = tokio::task::spawn_blocking(move || {
                contextra_durable_fs::durable_remove(&sidecar)
            })
            .await;
            if let Ok(Err(e)) = sidecar_res {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::debug!(
                        "Could not remove SSTable UUID sidecar {:?}: {} (non-critical)",
                        uuid_sidecar,
                        e
                    );
                }
            }
        }

        tracing::info!(
            "Compaction complete: merged {} SSTables into {:?}",
            input_ssts.len(),
            output_path
        );

        if let Some(ref manifest) = self.manifest {
            let current_live_entries: Vec<crate::manifest::ManifestEntry> = {
                let ssts = sstables.read().await;
                ssts.iter()
                    .map(|sst| crate::manifest::ManifestEntry::Add {
                        path: sst.file_path().to_path_buf(),
                        max_tx: sst.metadata().max_tx_id,
                    })
                    .collect()
            };
            if let Err(e) = manifest
                .maybe_rollover(
                    &current_live_entries,
                    crate::manifest::DEFAULT_ROLLOVER_THRESHOLD_BYTES,
                )
                .await
            {
                tracing::warn!("Periodic MANIFEST rollover failed after compaction: {e}");
            }
        }

        Ok(true)
    }

    /// Selects SSTables to compact using Size-Tiered strategy.
    ///
    /// Groups by size class and returns the group that meets the threshold,
    /// prioritizing key range overlap and contiguity.
    pub fn select_compaction_candidates(
        &self,
        ssts: &[Arc<SstableReader>],
    ) -> Option<Vec<Arc<SstableReader>>> {
        super::adaptive::select_stcs_candidates(
            ssts,
            self.config.min_sstables_per_tier,
            self.config.size_ratio,
        )
    }

    /// Estimates the peak memory consumption (in bytes) during a multi-way merge of candidates.
    ///
    /// Computes a conservative estimate taking into account candidate file sizes, block stream
    /// buffers, heap node structures, and in-memory deserialization overhead.
    pub(super) fn estimate_compaction_peak_bytes(&self, candidates: &[Arc<SstableReader>]) -> u64 {
        let mut total_file_size: u64 = 0;
        for candidate in candidates {
            total_file_size += candidate.metadata().file_size;
        }

        // Base memory estimate for stream buffers and multi-way merge priority queue:
        // Each stream buffers at least 1 block (BLOCK_SIZE = 4KB), plus heap structures and
        // deserialized entry overhead (estimated as ~1.5x raw candidate file size).
        let raw_overhead = (total_file_size as f64 * 1.5) as u64;
        let min_stream_buffer = (candidates.len() as u64) * 4096 * 4;

        raw_overhead + min_stream_buffer
    }

    /// Performs a multi-way merge of input SSTables into a single output SSTable.
    ///
    /// During merge:
    /// - Duplicate keys: newest sequence number wins
    /// - Tombstones: removed if `seq_no < min_snapshot_seq` (no snapshot references them)
    pub async fn merge_sstables(
        &self,
        inputs: &[Arc<SstableReader>],
        output_path: &std::path::Path,
        min_snapshot_seq: u64,
        is_full_compaction: bool,
    ) -> Result<()> {
        self.merge_sstables_with_cancel(
            inputs,
            output_path,
            min_snapshot_seq,
            is_full_compaction,
            None,
        )
        .await
    }

    /// Helper to fetch next item from source stream and push to heap.
    async fn fetch_next_entry<T: Send>(
        source_idx: usize,
        streams: &mut [crate::sstable::SstableStream],
        heap: &mut std::collections::BinaryHeap<T>,
    ) -> Result<()>
    where
        T: Ord,
        T: From<(bytes::Bytes, bytes::Bytes, u64, u64, usize)>,
    {
        if let Some((key, value, seq, tx)) = streams[source_idx].next_entry().await? {
            heap.push(T::from((key, value, seq, tx, source_idx)));
        }
        Ok(())
    }

    /// Writes a single SSTable entry and applies token-bucket I/O rate limiting.
    async fn write_entry_with_rate_limit(
        &self,
        builder: &mut SstableBuilder,
        key: &bytes::Bytes,
        value: &bytes::Bytes,
        seq: u64,
        tx: u64,
        io_bytes_written: &mut u64,
        io_last_reset: &mut std::time::Instant,
    ) -> Result<()> {
        let entry_bytes = (key.len() + value.len() + 16) as u64;
        builder.add(key, value, seq, tx).await?;

        if let Some(max_bps) = self.config.max_io_bytes_per_second {
            if max_bps > 0 {
                *io_bytes_written += entry_bytes;
                let elapsed = io_last_reset.elapsed();
                let target =
                    std::time::Duration::from_secs_f64(*io_bytes_written as f64 / max_bps as f64);
                if target > elapsed {
                    let delay = (target - elapsed).min(std::time::Duration::from_millis(100));
                    tokio::time::sleep(delay).await;
                    let total_elapsed = io_last_reset.elapsed();
                    let allowed_bytes = (total_elapsed.as_secs_f64() * max_bps as f64) as u64;
                    *io_bytes_written = io_bytes_written.saturating_sub(allowed_bytes);
                    *io_last_reset = std::time::Instant::now();
                }
            }
        }
        Ok(())
    }

    /// Performs a multi-way merge with an optional cancellation token.
    pub async fn merge_sstables_with_cancel(
        &self,
        inputs: &[Arc<SstableReader>],
        output_path: &std::path::Path,
        min_snapshot_seq: u64,
        is_full_compaction: bool,
        cancel_token: Option<&tokio_util::sync::CancellationToken>,
    ) -> Result<()> {
        let merge_res = self
            .merge_sstables_inner(
                inputs,
                output_path,
                min_snapshot_seq,
                is_full_compaction,
                cancel_token,
            )
            .await;

        if merge_res.is_err() {
            let p = output_path.to_path_buf();
            let res =
                tokio::task::spawn_blocking(move || contextra_durable_fs::durable_remove(&p))
                    .await;
            if let Ok(Err(e)) = res {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::warn!(
                        "Failed to clean up partial compaction output file {:?}: {}",
                        output_path,
                        e
                    );
                }
            }
        }

        merge_res
    }

    async fn merge_sstables_inner(
        &self,
        inputs: &[Arc<SstableReader>],
        output_path: &std::path::Path,
        min_snapshot_seq: u64,
        is_full_compaction: bool,
        cancel_token: Option<&tokio_util::sync::CancellationToken>,
    ) -> Result<()> {
        struct HeapItem {
            key: bytes::Bytes,
            value: bytes::Bytes,
            seq: u64,
            tx: u64,
            source_idx: usize,
        }

        impl From<(bytes::Bytes, bytes::Bytes, u64, u64, usize)> for HeapItem {
            fn from(tuple: (bytes::Bytes, bytes::Bytes, u64, u64, usize)) -> Self {
                Self {
                    key: tuple.0,
                    value: tuple.1,
                    seq: tuple.2,
                    tx: tuple.3,
                    source_idx: tuple.4,
                }
            }
        }

        impl PartialEq for HeapItem {
            fn eq(&self, other: &Self) -> bool {
                self.key == other.key
                    && (self.seq & !TOMBSTONE_BIT) == (other.seq & !TOMBSTONE_BIT)
                    && self.source_idx == other.source_idx
            }
        }

        impl Eq for HeapItem {}

        impl PartialOrd for HeapItem {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for HeapItem {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                // Reverse key comparison for min-heap (smallest key first)
                match other.key.cmp(&self.key) {
                    std::cmp::Ordering::Equal => {
                        // Max-heap for raw_seq (largest logical seq first)
                        let self_raw = self.seq & !TOMBSTONE_BIT;
                        let other_raw = other.seq & !TOMBSTONE_BIT;
                        self_raw
                            .cmp(&other_raw)
                            .then_with(|| self.source_idx.cmp(&other.source_idx))
                    }
                    ord => ord,
                }
            }
        }

        let mut streams = Vec::new();
        for sst in inputs {
            streams.push(sst.stream().await?);
        }

        let mut heap = std::collections::BinaryHeap::new();
        for (i, stream) in streams.iter_mut().enumerate() {
            if let Some((key, value, seq, tx)) = stream.next_entry().await? {
                heap.push(HeapItem {
                    key,
                    value,
                    seq,
                    tx,
                    source_idx: i,
                });
            }
        }

        let mut builder =
            SstableBuilder::create_with_key_manager(output_path, self.key_manager.clone()).await?;
        let mut last_key: Option<bytes::Bytes> = None;
        let mut floor_emitted = false;
        let mut processed_count = 0;

        // Token-Bucket-State für I/O-Rate-Limiting (§4.12 C-2)
        let mut io_token_bytes_written: u64 = 0;
        let mut io_token_last_reset = std::time::Instant::now();

        while let Some(current_item) = heap.pop() {
            if let Some(ct) = cancel_token {
                if ct.is_cancelled() {
                    return Err(contextra_core::ContextraError::Internal(
                        "Compaction cancelled during merge stream processing".into(),
                    ));
                }
            }

            processed_count += 1;
            if processed_count % self.config.yield_threshold == 0 {
                // PERF-3: System Pressure-Awareness
                if let Some(ref pressure_rx) = self.pressure_rx {
                    let level = pressure_rx.borrow().pressure_level;
                    if level == crate::system_pressure::PressureLevel::Critical {
                        tracing::warn!("Compaction merge delayed due to Critical system pressure.");
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    } else if level == crate::system_pressure::PressureLevel::Elevated {
                        tracing::debug!("Compaction merge active under Elevated system pressure.");
                    }
                }

                // FIND-STO-002: Budgeted Compaction
                if !self.budget.has_memory_capacity() {
                    let wait_start = std::time::Instant::now();
                    while !self.budget.has_memory_capacity() {
                        if wait_start.elapsed() >= self.config.max_backpressure_wait {
                            let limit_bytes =
                                self.config.max_memory_bytes.unwrap_or(128 * 1024 * 1024);
                            let used_mb = limit_bytes.div_ceil(1024 * 1024);
                            let limit_mb = limit_bytes.div_ceil(1024 * 1024);
                            tracing::error!(
                                "Compaction aborted: backpressure wait timeout exceeded ({:?}) while memory budget exhausted",
                                self.config.max_backpressure_wait
                            );
                            return Err(contextra_core::ContextraError::MemoryBudgetExceeded {
                                used_mb,
                                limit_mb,
                            });
                        }
                        if let Some(ct) = cancel_token {
                            if ct.is_cancelled() {
                                return Err(contextra_core::ContextraError::Internal(
                                    "Compaction cancelled during memory budget wait".into(),
                                ));
                            }
                        }
                        tracing::warn!("Compaction engine paused due to memory budget exhaustion.");
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                    tokio::task::yield_now().await;
                }
            }

            // Immediately advance stream for popped item to maintain global heap invariant
            Self::fetch_next_entry(current_item.source_idx, &mut streams, &mut heap).await?;

            let is_tombstone = (current_item.seq & TOMBSTONE_BIT) != 0;
            let raw_seq = current_item.seq & !TOMBSTONE_BIT;

            if last_key.as_ref() != Some(&current_item.key) {
                floor_emitted = false;
            }

            // AI-TAG[BOUND_UNIFICATION]
            // Standardize boundary check for version retention and multi-way merging to `<=` min_snapshot_seq.
            // Versions strictly greater than min_snapshot_seq are visible to active/future snapshots and must remain separate.
            // Versions `<= min_snapshot_seq` belong to the floor and are candidates for folding into a single entry.
            if raw_seq > min_snapshot_seq {
                // Version is above floor: keep as separate entry
                let should_gc_tombstone =
                    is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq;
                if !should_gc_tombstone {
                    self.write_entry_with_rate_limit(
                        &mut builder,
                        &current_item.key,
                        &current_item.value,
                        current_item.seq,
                        current_item.tx,
                        &mut io_token_bytes_written,
                        &mut io_token_last_reset,
                    )
                    .await?;
                }
                last_key = Some(current_item.key.clone());
            } else if !floor_emitted {
                // Version is at or below the floor (raw_seq <= min_snapshot_seq) AND floor has not been emitted yet.
                // Exhaust and fold ALL remaining versions for this key `<= min_snapshot_seq` into a single entry.
                let mut acc_val = current_item.value.clone();
                let acc_seq = current_item.seq;
                let acc_tx = current_item.tx;
                let is_acc_tombstone = is_tombstone;
                let mut merge_failed = false;

                // Inner loop: consume all older versions for current_item.key with raw_seq <= min_snapshot_seq
                while let Some(next_item) = heap.peek() {
                    if next_item.key != current_item.key {
                        break;
                    }
                    let next_raw_seq = next_item.seq & !TOMBSTONE_BIT;
                    if next_raw_seq > min_snapshot_seq {
                        // Older version is somehow above min_snapshot_seq (impossible given heap sort, but safe check)
                        break;
                    }

                    let next_is_tombstone = (next_item.seq & TOMBSTONE_BIT) != 0;

                    // If either current accumulated value or next value is a tombstone, stop folding
                    if is_acc_tombstone || next_is_tombstone {
                        break;
                    }

                    // Check if MergeOperator can fold these two values
                    if let Some(ref merge_op) = self.merge_operator {
                        if self.key_manager.is_none() {
                            let start_nanos = self.clock.monotonic_nanos();
                            // Folding order: older version (next_item) is existing, newer version (acc_val) is new
                            let merge_res = merge_op.merge(&next_item.value, &acc_val);
                            let elapsed_nanos =
                                self.clock.monotonic_nanos().saturating_sub(start_nanos);
                            let timeout_nanos =
                                u64::try_from(self.config.merge_wall_clock_timeout.as_nanos())
                                    .unwrap_or(u64::MAX);

                            let merge_res = if timeout_nanos > 0 && elapsed_nanos > timeout_nanos {
                                Err(contextra_core::ContextraError::Sandbox(
                                    "Merge operator wall-clock timeout exceeded".into(),
                                ))
                            } else {
                                merge_res
                            };

                            match merge_res {
                                Ok(merged_bytes) => {
                                    acc_val = bytes::Bytes::from(merged_bytes);
                                    // Pop next_item from heap and advance its stream
                                    if let Some(popped) = heap.pop() {
                                        Self::fetch_next_entry(
                                            popped.source_idx,
                                            &mut streams,
                                            &mut heap,
                                        )
                                        .await?;
                                    }
                                }
                                Err(err) => {
                                    tracing::warn!(
                                        "Merge operator error for key {:?}: {}, retaining remaining versions (fail-safe)",
                                        current_item.key,
                                        err
                                    );
                                    merge_failed = true;
                                    break;
                                }
                            }
                        } else {
                            // Encrypted tables cannot fold values via MergeOperator; break to retain current version as floor
                            break;
                        }
                    } else {
                        // Without MergeOperator, newest value (current_item) replaces all older versions;
                        // consume older versions without merging
                        if let Some(popped) = heap.pop() {
                            Self::fetch_next_entry(popped.source_idx, &mut streams, &mut heap)
                                .await?;
                        }
                    }
                }

                // Write the resulting folded floor entry
                let should_gc_tombstone = is_acc_tombstone
                    && is_full_compaction
                    && (acc_seq & !TOMBSTONE_BIT) < min_snapshot_seq;
                if !should_gc_tombstone {
                    self.write_entry_with_rate_limit(
                        &mut builder,
                        &current_item.key,
                        &acc_val,
                        acc_seq,
                        acc_tx,
                        &mut io_token_bytes_written,
                        &mut io_token_last_reset,
                    )
                    .await?;
                }

                last_key = Some(current_item.key.clone());
                if !merge_failed {
                    floor_emitted = true;
                }
            } else {
                // floor_emitted is true AND raw_seq <= min_snapshot_seq:
                // An older version for this key below floor watermark that was not folded (e.g. due to merge failure or tombstone).
                // Discard to prevent stale duplicates.
            }
        }
        builder.finish().await?;
        Ok(())
    }

    /// Generates a unique SSTable file path using a monotonic counter.
    pub fn generate_sst_path(&self, data_path: &std::path::Path) -> Result<PathBuf> {
        let seq = COMPACTION_SST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let count = self
            .compaction_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(data_path.join(format!("sst-compact-{:020}-{:04}.sst", seq, count % 10000)))
    }

    /// Runs the background compaction loop.
    ///
    /// Periodically checks if compaction is needed and performs it.
    /// Designed to be spawned via `tokio::spawn`.
    pub async fn run_loop(
        self: Arc<Self>,
        sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
        data_path: PathBuf,
        shutdown: tokio_util::sync::CancellationToken,
    ) {
        loop {
            // Check for shutdown signal
            if shutdown.is_cancelled() {
                tracing::info!("Compaction engine received shutdown signal");
                break;
            }

            // Wait for interval OR shutdown signal
            tokio::select! {
                _ = tokio::time::sleep(self.config.check_interval) => {
                    match self.maybe_compact_with_cancel(&sstables, &data_path, Some(&shutdown)).await {
                        Ok(true) => {
                            tracing::debug!("Background compaction cycle completed successfully");
                        }
                        Ok(false) => {
                            tracing::trace!("No compaction needed");
                            if let Some(ref manifest) = self.manifest {
                                let current_live_entries: Vec<crate::manifest::ManifestEntry> = {
                                    let ssts = sstables.read().await;
                                    ssts.iter()
                                        .map(|sst| crate::manifest::ManifestEntry::Add {
                                            path: sst.file_path().to_path_buf(),
                                            max_tx: sst.metadata().max_tx_id,
                                        })
                                        .collect()
                                };
                                if let Err(e) = manifest
                                    .maybe_rollover(
                                        &current_live_entries,
                                        crate::manifest::DEFAULT_ROLLOVER_THRESHOLD_BYTES,
                                    )
                                    .await
                                {
                                    tracing::warn!("Periodic MANIFEST rollover check failed: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Background compaction failed: {}", e);
                        }
                    }
                }
                _ = shutdown.cancelled() => {
                    tracing::info!("Compaction engine shutting down via signal");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sstable::create_block_cache;
    use contextra_mvcc::snapshot::SnapshotFloor;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_durable_remove_compaction_resilience_and_manifest_integrity() -> Result<()> {
        let tmp = TempDir::new().map_err(|e| contextra_core::ContextraError::Storage(e.to_string()))?;
        let registry = Arc::new(contextra_core::SnapshotRegistry::new());
        let bc = create_block_cache(1);
        let manifest_path = tmp.path().join("MANIFEST-000001");
        let manifest = Arc::new(crate::manifest::Manifest::create(&manifest_path).await?);

        let config = CompactionConfig {
            min_sstables_per_tier: 2,
            ..Default::default()
        };

        let engine = CompactionEngine::new(
            config,
            registry as Arc<dyn SnapshotFloor>,
            Arc::clone(&bc),
            None,
            Arc::new(contextra_core::ResourceTracker::new(
                contextra_core::ResourceBudget {
                    memory_limit: 1024 * 1024,
                },
            )),
            Some(manifest),
        );

        // Create 2 input SSTables with UUID sidecars
        let sstables = Arc::new(RwLock::new(Vec::new()));
        let mut old_sst_paths = Vec::new();
        let mut sidecar_paths = Vec::new();

        for i in 0..2u8 {
            let sst_name = format!("sst-{i}.sst");
            let sst_path = tmp.path().join(&sst_name);
            let mut builder = crate::sstable::SstableBuilder::create(&sst_path).await?;
            builder
                .add(format!("key-{i}").as_bytes(), b"val", i as u64 + 1, i as u64 + 1)
                .await?;
            builder.finish().await?;

            let reader = Arc::new(
                SstableReader::open_with_key_manager(&sst_path, Arc::clone(&bc), None).await?,
            );

            let sidecar_path = PathBuf::from(format!("{}.uuid", sst_path.display()));
            tokio::fs::write(&sidecar_path, b"uuid-data")
                .await
                .map_err(|e| contextra_core::ContextraError::Storage(e.to_string()))?;

            old_sst_paths.push(sst_path);
            sidecar_paths.push(sidecar_path);
            sstables.write().await.push(reader);
        }

        // Simulate a deletion error prior to compaction (e.g. sidecar_paths[0] already removed/missing)
        let s0 = sidecar_paths[0].clone();
        let _ = tokio::task::spawn_blocking(move || contextra_durable_fs::durable_remove(&s0)).await;

        // Execute compaction
        let compacted = engine.maybe_compact(&sstables, tmp.path()).await?;
        assert!(compacted, "Compaction should report success");

        // (a) All replaced input SSTables and UUID sidecars should not exist
        for sst_p in &old_sst_paths {
            assert!(!sst_p.exists(), "Replaced SSTable {:?} should be removed", sst_p);
        }
        for sidecar_p in &sidecar_paths {
            assert!(!sidecar_p.exists(), "UUID sidecar {:?} should be removed", sidecar_p);
        }

        // (b) No file referenced by the manifest should be removed
        let ssts_guard = sstables.read().await;
        assert_eq!(ssts_guard.len(), 1, "Compacted output SSTable present in list");
        let active_sst_path = ssts_guard[0].file_path();
        assert!(
            active_sst_path.exists(),
            "Output SSTable referenced by manifest {:?} must exist",
            active_sst_path
        );
        assert!(manifest_path.exists(), "Manifest file must exist");
        Ok(())
    }
}
