use super::adaptive::{AdaptiveCompactionPlanner, CostBasedAdaptivePlanner, WorkloadMetrics};
use super::config::CompactionConfig;
use super::merge_operator::MergeOperator;
use crate::sstable::{BlockCache, SstableBuilder, SstableReader};
use crate::wal::KeyManager;
use contextra_core::{Result, SnapshotRegistry, StorageStats, TOMBSTONE_BIT};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

pub struct CompactionEngine {
    config: CompactionConfig,
    pub(super) snapshot_registry: Arc<SnapshotRegistry>,
    block_cache: Arc<BlockCache>,
    key_manager: Option<Arc<KeyManager>>,
    budget: Arc<contextra_core::ResourceTracker>,
    manifest: Option<Arc<crate::manifest::Manifest>>,
    compaction_counter: AtomicU64,
    pressure_rx: Option<tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>>,
    workload_metrics: WorkloadMetrics,
    adaptive_planner: Option<Arc<dyn AdaptiveCompactionPlanner>>,
    merge_operator: Option<Arc<dyn MergeOperator>>,
}

impl CompactionEngine {
    /// Creates a new compaction engine.
    pub fn new(
        config: CompactionConfig,
        snapshot_registry: Arc<SnapshotRegistry>,
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
            snapshot_registry,
            block_cache,
            key_manager,
            budget,
            manifest,
            compaction_counter: AtomicU64::new(0),
            pressure_rx: None,
            workload_metrics: WorkloadMetrics::new(),
            adaptive_planner,
            merge_operator: None,
        }
    }

    /// Attaches a custom merge operator for compaction value merging (§4.12).
    pub fn with_merge_operator(mut self, merge_operator: Arc<dyn MergeOperator>) -> Self {
        self.merge_operator = Some(merge_operator);
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
                    let min_seq = self.snapshot_registry.min_active_seqno();

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
        let min_snapshot_seq = self.snapshot_registry.min_active_seqno();
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
                if let Err(e) = tokio::fs::remove_file(&output_path).await {
                    tracing::warn!(
                        "Failed to clean up aborted compaction output {:?}: {}",
                        output_path,
                        e
                    );
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
            if let Err(e) = tokio::fs::remove_file(path).await {
                tracing::warn!("Failed to delete compacted SSTable {:?}: {}", path, e);
            }
            let uuid_sidecar = PathBuf::from(format!("{}.uuid", path.display()));
            if matches!(tokio::fs::try_exists(&uuid_sidecar).await, Ok(true)) {
                if let Err(e) = tokio::fs::remove_file(&uuid_sidecar).await {
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
    /// Groups by size class and returns the first group that meets the threshold.
    pub(super) fn select_compaction_candidates(
        &self,
        ssts: &[Arc<SstableReader>],
    ) -> Option<Vec<Arc<SstableReader>>> {
        if ssts.len() < self.config.min_sstables_per_tier {
            return None;
        }

        let min_tier = self.config.min_sstables_per_tier;

        // 1. Evaluate all contiguous sub-ranges ssts[start..end]
        // Candidates MUST form a contiguous subslice in `ssts`.
        let mut best_tier_range: Option<(usize, usize)> = None;
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

                if ratio <= self.config.size_ratio {
                    let win_len = end - start;
                    let win_size: u64 = window.iter().map(|s| s.metadata().file_size).sum();

                    if win_len > best_tier_len
                        || (win_len == best_tier_len && win_size < best_tier_size)
                    {
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

        // 2. Fallback for large SSTable counts: select contiguous window of min_tier size with smallest total size
        if ssts.len() >= min_tier * 2 {
            let mut best_fallback_start = None;
            let mut min_fallback_size = u64::MAX;

            for start in 0..=(ssts.len() - min_tier) {
                let end = start + min_tier;
                let total_size: u64 = ssts[start..end]
                    .iter()
                    .map(|s| s.metadata().file_size)
                    .sum();
                if total_size < min_fallback_size {
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
            if let Err(e) = tokio::fs::remove_file(output_path).await {
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

        while let Some(item) = heap.pop() {
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
                // Check pressure_rx at batch boundaries between merge iterations.
                // Critical pressure level triggers a 50ms backpressure sleep delay to reduce NVMe / CPU contention.
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
                // Apply memory backpressure to prevent Compaction from OOMing the system
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
                    // Yield after blocking to give other tasks a fair chance
                    tokio::task::yield_now().await;
                }
                // When budget is fine: no yield, just continue — the budget check above
                // already provided cooperative scheduling opportunities via the sleep loop.
            }

            let is_tombstone = (item.seq & TOMBSTONE_BIT) != 0;
            let raw_seq = item.seq & !TOMBSTONE_BIT;

            if last_key.as_ref() != Some(&item.key) {
                floor_emitted = false;
            }

            let mut merged = false;
            let mut failed_merge_keep = false;

            // MergeOperator logic:
            // 1. key_manager MUST be None (encrypted values cannot be merged as raw bytes).
            //    Note: If encryption is enabled (key_manager.is_some()), values are encrypted
            //    ciphertexts and cannot be merged by a raw MergeOperator. Merging is skipped,
            //    preserving existing versions according to standard retention rules.
            // 2. Both current item and next item must be Put entries (not tombstones).
            // 3. Both sequence numbers must be strictly below min_snapshot_seq (no active snapshot
            //    can view the older version independently).
            // 4. No tombstone exists for this key in inputs (tombstone keys are never merged to avoid resurrecting deleted data).
            if let Some(ref merge_op) = self.merge_operator {
                if self.key_manager.is_none() && !is_tombstone && raw_seq < min_snapshot_seq {
                    if let Some(next_item) = heap.peek() {
                        let next_is_same_key = next_item.key == item.key;
                        let next_is_tombstone = (next_item.seq & TOMBSTONE_BIT) != 0;
                        let next_raw_seq = next_item.seq & !TOMBSTONE_BIT;

                        if next_is_same_key
                            && !next_is_tombstone
                            && next_raw_seq < min_snapshot_seq
                            && !heap
                                .iter()
                                .any(|h| h.key == item.key && (h.seq & TOMBSTONE_BIT) != 0)
                        {
                            // existing_val is older version (next_item), new_val is newer version (item)
                            match merge_op.merge(&next_item.value, &item.value) {
                                Ok(merged_val) => {
                                    // Consume next_item from heap and advance its stream
                                    if let Some(popped_next) = heap.pop() {
                                        if let Some((key, value, seq, tx)) =
                                            streams[popped_next.source_idx].next_entry().await?
                                        {
                                            heap.push(HeapItem {
                                                key,
                                                value,
                                                seq,
                                                tx,
                                                source_idx: popped_next.source_idx,
                                            });
                                        }
                                    }

                                    let merged_bytes = bytes::Bytes::from(merged_val);
                                    let entry_bytes =
                                        (item.key.len() + merged_bytes.len() + 16) as u64;
                                    builder
                                        .add(&item.key, &merged_bytes, item.seq, item.tx)
                                        .await?;

                                    // Token-Bucket I/O Rate Limiting
                                    if let Some(max_bps) = self.config.max_io_bytes_per_second {
                                        if max_bps > 0 {
                                            io_token_bytes_written += entry_bytes;
                                            let elapsed = io_token_last_reset.elapsed();
                                            let target = std::time::Duration::from_secs_f64(
                                                io_token_bytes_written as f64 / max_bps as f64,
                                            );
                                            if target > elapsed {
                                                let delay = (target - elapsed)
                                                    .min(std::time::Duration::from_millis(100));
                                                tokio::time::sleep(delay).await;
                                                let total_elapsed = io_token_last_reset.elapsed();
                                                let allowed_bytes = (total_elapsed.as_secs_f64()
                                                    * max_bps as f64)
                                                    as u64;
                                                io_token_bytes_written = io_token_bytes_written
                                                    .saturating_sub(allowed_bytes);
                                                io_token_last_reset = std::time::Instant::now();
                                            }
                                        }
                                    }

                                    last_key = Some(item.key.clone());
                                    floor_emitted = true;
                                    merged = true;
                                }
                                Err(err) => {
                                    tracing::warn!(
                                        "Merge operator error for key {:?}: {}, retaining both versions (fail-safe)",
                                        item.key,
                                        err
                                    );
                                    failed_merge_keep = true;
                                }
                            }
                        }
                    }
                }
            }

            if !merged {
                // LSM Retention Rule:
                // Keep all versions with raw_seq >= min_snapshot_seq (visible to active or future snapshots)
                // PLUS the newest version with raw_seq < min_snapshot_seq (the "floor" version).
                // All further, older versions for the key below min_snapshot_seq are discarded.
                let keep = if raw_seq >= min_snapshot_seq {
                    true
                } else if !floor_emitted {
                    if !failed_merge_keep {
                        floor_emitted = true;
                    }
                    true
                } else {
                    false
                };

                if keep {
                    // O(1): Bytes::clone is an Arc refcount increment
                    last_key = Some(item.key.clone());

                    // FIND-STO-001: Tombstone-Retention
                    // Only GC tombstones during FULL compaction when no snapshot references them
                    // and no older SSTables outside this compaction round can contain older values.
                    // NOTE: If raw_seq < min_snapshot_seq and is_full_compaction is true, this tombstone
                    // is the floor version below min_snapshot_seq. Being a tombstone below min_snapshot_seq
                    // during full compaction, no active snapshot references a non-deleted version below it
                    // and no older SSTables exist, so GC'ing it is safe.
                    let should_gc_tombstone =
                        is_tombstone && is_full_compaction && raw_seq < min_snapshot_seq;
                    if !should_gc_tombstone {
                        let entry_bytes = (item.key.len() + item.value.len() + 16) as u64; // +16 für Overhead
                        builder
                            .add(&item.key, &item.value, item.seq, item.tx)
                            .await?;

                        // Token-Bucket I/O Rate Limiting (plattformneutral, außerhalb aller Write-Locks)
                        if let Some(max_bps) = self.config.max_io_bytes_per_second {
                            if max_bps > 0 {
                                io_token_bytes_written += entry_bytes;
                                let elapsed = io_token_last_reset.elapsed();
                                let target = std::time::Duration::from_secs_f64(
                                    io_token_bytes_written as f64 / max_bps as f64,
                                );
                                if target > elapsed {
                                    let delay = (target - elapsed)
                                        .min(std::time::Duration::from_millis(100));
                                    // INVARIANTE: Kein MVCC-Write-Lock aktiv an dieser Stelle (merge läuft lock-frei).
                                    // Verifiziert durch Lektüre von merge_sstables() — kein RwLock::write() im Merge-Loop.
                                    tokio::time::sleep(delay).await;
                                    // Deduct allowed bytes based on actual elapsed time instead of wiping to 0
                                    let total_elapsed = io_token_last_reset.elapsed();
                                    let allowed_bytes =
                                        (total_elapsed.as_secs_f64() * max_bps as f64) as u64;
                                    io_token_bytes_written =
                                        io_token_bytes_written.saturating_sub(allowed_bytes);
                                    io_token_last_reset = std::time::Instant::now();
                                }
                            }
                        }
                    }
                }
            }

            // Immediately fetch the next item from the source stream
            if let Some((key, value, seq, tx)) = streams[item.source_idx].next_entry().await? {
                heap.push(HeapItem {
                    key,
                    value,
                    seq,
                    tx,
                    source_idx: item.source_idx,
                });
            }
        }
        builder.finish().await?;
        Ok(())
    }

    /// Generates a unique SSTable file path using microsecond timestamp.
    pub(super) fn generate_sst_path(&self, data_path: &std::path::Path) -> Result<PathBuf> {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                contextra_core::ContextraError::Storage(format!("System clock error: {}", e))
            })?
            .as_micros();
        let count = self
            .compaction_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(data_path.join(format!("sst-compact-{:020}-{:04}.sst", id, count % 10000)))
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
