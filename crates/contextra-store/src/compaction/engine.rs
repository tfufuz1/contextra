//! # Kompaktierungs-Engine (Ring 1)
//!
//! Fuehrt das Zusammenfuehren ueberlappender SSTables durch und bereinigt alte Versionen.

use super::adaptive::{AdaptiveCompactionPlanner, CostBasedAdaptivePlanner, WorkloadMetrics};
use super::config::CompactionConfig;
use super::merge_operator::MergeOperator;
use crate::io_seam::RealFs;
use crate::sstable::{BlockCache, SstableBuilder, SstableReader};
use crate::wal::KeyManager;
use contextra_mvcc::SnapshotRegistry;
use contextra_ports::gate::ReadGate;
use contextra_ports::StorageStats;
use contextra_ports::Vfs;
use contextra_types::{Result, TOMBSTONE_BIT};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;

static COMPACTION_SST_COUNTER: AtomicU64 = AtomicU64::new(1);
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing;

struct OutputGuard {
    vfs: Arc<dyn Vfs>,
    path: PathBuf,
    armed: bool,
}

impl OutputGuard {
    fn new(vfs: Arc<dyn Vfs>, path: PathBuf) -> Self {
        Self {
            vfs,
            path,
            armed: true,
        }
    }

    fn commit(&mut self) {
        self.armed = false;
    }
}

impl Drop for OutputGuard {
    fn drop(&mut self) {
        if self.armed {
            if let Err(error) = self.vfs.remove_file(&self.path) {
                tracing::warn!(path = ?self.path, error = %error, "Failed to clean up uncommitted compaction output");
            }
        }
    }
}

// ANCHOR[contextra-store:COMP-ENG-001]
/// Orchestriert die periodische Verdichtung und Bereinigung von SSTables.
pub struct CompactionEngine {
    config: CompactionConfig,
    pub(super) snapshot_registry: Arc<SnapshotRegistry>,
    block_cache: Arc<BlockCache>,
    key_manager: Option<Arc<KeyManager>>,
    budget: Arc<contextra_ports::ResourceTracker>,
    manifest: Option<Arc<crate::manifest::Manifest>>,
    compaction_counter: AtomicU64,
    pressure_rx: Option<tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>>,
    workload_metrics: WorkloadMetrics,
    adaptive_planner: Option<Arc<dyn AdaptiveCompactionPlanner>>,
    merge_operator: Option<Arc<dyn MergeOperator>>,
    clock: Arc<dyn contextra_ports::Clock>,
    pub(crate) vfs: Arc<dyn Vfs>,
    erase_filter: Option<(
        Arc<crate::erasure::gate::StoreGate>,
        Arc<parking_lot::RwLock<Arc<crate::erasure::filter::CollapseMap>>>,
    )>,
    pending_delete: Option<Arc<crate::erasure::pending::PendingDelete>>,
    catalog_mutex: Arc<tokio::sync::Mutex<()>>,
}

impl CompactionEngine {
    /// Creates a new compaction engine.
    pub fn new(
        config: CompactionConfig,
        snapshot_registry: Arc<SnapshotRegistry>,
        block_cache: Arc<BlockCache>,
        key_manager: Option<Arc<KeyManager>>,
        budget: Arc<contextra_ports::ResourceTracker>,
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
            clock: Arc::new(contextra_ports::SystemClock::new()),
            vfs: Arc::new(RealFs),
            erase_filter: None,
            pending_delete: None,
            catalog_mutex: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    /// Shares the store's serialized catalog-publication lock.
    pub(crate) fn with_catalog_mutex(mut self, mutex: Arc<tokio::sync::Mutex<()>>) -> Self {
        self.catalog_mutex = mutex;
        self
    }

    /// Configures a custom VFS implementation for compaction file operations.
    pub fn with_vfs(mut self, vfs: Arc<dyn Vfs>) -> Self {
        self.vfs = vfs;
        self
    }

    /// Verknüpft das Erasure-Gate und die Collapse-Map für H-7-Filterung beim Merge.
    pub(crate) fn with_erase_filter(
        mut self,
        gate: Arc<crate::erasure::gate::StoreGate>,
        collapse: Arc<parking_lot::RwLock<Arc<crate::erasure::filter::CollapseMap>>>,
    ) -> Self {
        self.erase_filter = Some((gate, collapse));
        self
    }

    /// Verknüpft die Pending-Delete-Liste für sichere Datei-Entsorgung (WP-4.1).
    pub(crate) fn with_pending_delete(
        mut self,
        pending: Arc<crate::erasure::pending::PendingDelete>,
    ) -> Self {
        self.pending_delete = Some(pending);
        self
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
            let trigger_threshold = self
                .config
                .min_sstables_per_tier
                .min(self.config.l0_compaction_trigger);
            if ssts.len() < trigger_threshold {
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

        self.execute_compaction_for_candidates(
            input_ssts,
            is_full_compaction,
            sstables,
            data_path,
            cancel_token,
        )
        .await
    }

    /// Forces a full compaction over all existing SSTables.
    ///
    /// This maintenance API merges all currently registered SSTables into a single
    /// consolidated SSTable, evaluating tombstones against the active snapshot watermark.
    pub async fn compact_all(
        &self,
        sstables: &RwLock<Vec<Arc<SstableReader>>>,
        data_path: &std::path::Path,
    ) -> Result<bool> {
        let (mut input_ssts, is_full) = {
            let ssts = sstables.read().await;
            if ssts.len() < 2 {
                return Ok(false);
            }
            (ssts.clone(), true)
        };

        input_ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        self.execute_compaction_for_candidates(input_ssts, is_full, sstables, data_path, None)
            .await
    }

    /// Compacts all SSTables that overlap with the specified key range `[start, end]`.
    ///
    /// Neutral maintenance API without domain-specific erasure/shredding semantics.
    pub async fn compact_range(
        &self,
        range: std::ops::RangeInclusive<&[u8]>,
        sstables: &RwLock<Vec<Arc<SstableReader>>>,
        data_path: &std::path::Path,
    ) -> Result<bool> {
        let (start_key, end_key) = range.into_inner();
        let (mut input_ssts, is_full) = {
            let ssts = sstables.read().await;
            if ssts.len() < 2 {
                return Ok(false);
            }

            let mut overlapping = Vec::new();
            for sst in ssts.iter() {
                // If SSTable has key bounds, check overlap.
                // An SSTable overlaps if its range intersects [start_key, end_key].
                let meta = sst.metadata();
                let first = meta.first_key.as_ref();
                let last = meta.last_key.as_ref();
                if !(last < start_key || first > end_key) {
                    overlapping.push(Arc::clone(sst));
                }
            }

            if overlapping.len() < 2 {
                return Ok(false);
            }

            let is_full = overlapping.len() == ssts.len();
            (overlapping, is_full)
        };

        input_ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        self.execute_compaction_for_candidates(input_ssts, is_full, sstables, data_path, None)
            .await
    }

    /// Helper executing candidate merge, swap and manifest recording for a determined set of SSTables.
    async fn execute_compaction_for_candidates(
        &self,
        input_ssts: Vec<Arc<SstableReader>>,
        is_full_compaction: bool,
        sstables: &RwLock<Vec<Arc<SstableReader>>>,
        data_path: &std::path::Path,
        cancel_token: Option<&tokio_util::sync::CancellationToken>,
    ) -> Result<bool> {
        // Peak Memory Estimation Check
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
                return Err(contextra_types::ContextraError::MemoryBudgetExceeded {
                    used_mb,
                    limit_mb,
                });
            }
        }

        // Perform the merge
        let min_snapshot_seq = self.snapshot_registry.min_active_seqno();
        let output_path = self.generate_sst_path(data_path)?;
        // Hard guard (F-04): the output must never be one of the inputs. Otherwise the
        // merge overwrites its own input, and the old-path cleanup later deletes the output.
        if input_ssts
            .iter()
            .any(|sst| sst.file_path() == output_path.as_path())
        {
            return Err(contextra_types::ContextraError::invalid_input(
                "compaction output path collides with an input SSTable",
            ));
        }
        self.merge_sstables_with_cancel(
            &input_ssts,
            &output_path,
            min_snapshot_seq,
            is_full_compaction,
            cancel_token,
        )
        .await?;
        let mut output_guard = OutputGuard::new(Arc::clone(&self.vfs), output_path.clone());

        // Explicit fsync of output file and parent directory
        if let Some(parent) = output_path.parent() {
            self.vfs.sync_dir(parent)?;
        }

        // Open the new SSTable reader
        let new_reader = Arc::new(
            SstableReader::open_with_vfs(
                &output_path,
                Arc::clone(&self.block_cache),
                self.key_manager.clone(),
                Arc::clone(&self.vfs),
            )
            .await?,
        );

        // Catalog mutations serialize publication, while the SSTables write lock covers only RAM work.
        let _catalog_guard = self.catalog_mutex.lock().await;
        let start_opt = {
            let ssts = sstables.read().await;
            ssts.iter().position(|s| Arc::ptr_eq(s, &input_ssts[0]))
        };
        let is_contiguous = if let Some(start) = start_opt {
            let ssts = sstables.read().await;
            start + input_ssts.len() <= ssts.len()
                && input_ssts
                    .iter()
                    .enumerate()
                    .all(|(idx, input)| Arc::ptr_eq(input, &ssts[start + idx]))
        } else {
            false
        };
        if !is_contiguous {
            tracing::warn!("Compaction aborted: input SSTables are no longer contiguous");
            return Ok(false);
        }
        let insertion_point = start_opt.unwrap_or(0);
        let old_paths: Vec<PathBuf> = input_ssts
            .iter()
            .map(|sst| sst.file_path().to_path_buf())
            .collect();

        if let Some(ref manifest) = self.manifest {
            if let Err(error) = manifest
                .append(&crate::manifest::ManifestEntry::Replace {
                    removed: old_paths.clone(),
                    added: output_path.clone(),
                    added_max_tx: new_reader.metadata().max_tx_id,
                })
                .await
            {
                return Err(error);
            }
        }
        output_guard.commit();

        let mut ssts = sstables.write().await;
        debug_assert!(
            input_ssts.iter().enumerate().all(|(idx, input)| {
                ssts.get(insertion_point + idx)
                    .is_some_and(|current| Arc::ptr_eq(input, current))
            }),
            "catalog changed without catalog_mutex"
        );
        ssts.retain(|sst| !input_ssts.iter().any(|input| Arc::ptr_eq(input, sst)));
        let insert_idx = insertion_point.min(ssts.len());
        ssts.insert(insert_idx, new_reader);
        ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);
        debug_assert!(
            ssts.windows(2)
                .all(|w| (w[0].metadata().max_seq & !TOMBSTONE_BIT)
                    <= (w[1].metadata().max_seq & !TOMBSTONE_BIT)),
            "SSTable list must be sorted by max_seq in ascending order after compaction swap"
        );
        drop(ssts);
        drop(_catalog_guard);

        for old_sst in &input_ssts {
            self.block_cache.evict_file(old_sst.file_id());
        }

        // Delete old SSTable files (WP-4.1 PendingDelete wenn konfiguriert)
        if let Some(ref pending) = self.pending_delete {
            let mut to_remove = Vec::with_capacity(old_paths.len() * 2);
            for path in &old_paths {
                to_remove.push(path.clone());
                to_remove.push(PathBuf::from(format!("{}.uuid", path.display())));
            }
            if let Err(e) = pending.remove_after_commit(&to_remove).await {
                tracing::warn!("Pending-delete for compacted SSTables deferred: {}", e);
            }
        } else {
            for path in &old_paths {
                if let Err(e) = self.vfs.remove_file(path) {
                    tracing::warn!("Failed to delete compacted SSTable {:?}: {}", path, e);
                }
                let uuid_sidecar = PathBuf::from(format!("{}.uuid", path.display()));
                if matches!(self.vfs.exists(&uuid_sidecar), Ok(true)) {
                    if let Err(e) = self.vfs.remove_file(&uuid_sidecar) {
                        tracing::debug!(
                            "Could not remove SSTable UUID sidecar {:?}: {} (non-critical)",
                            uuid_sidecar,
                            e
                        );
                    }
                }
            }
        }

        tracing::info!(
            "Compaction complete: merged {} SSTables into {:?}",
            input_ssts.len(),
            output_path
        );

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
            if let Err(e) = self.vfs.remove_file(output_path) {
                if !matches!(e, contextra_ports::ContextraError::NotFound(_)) {
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

        let mut builder = SstableBuilder::create_with_vfs(
            output_path,
            self.key_manager.clone(),
            Arc::clone(&self.vfs),
        )
        .await?;
        let mut last_key: Option<bytes::Bytes> = None;
        let mut floor_emitted = false;
        let mut processed_count = 0;

        // Token-Bucket-State für I/O-Rate-Limiting (WP-4.0 Extraktion)
        let mut io_bucket = crate::io_budget::TokenBucket::new(
            self.config.max_io_bytes_per_second.unwrap_or(0),
            Arc::clone(&self.clock),
        );
        let erase_filter_opt = self.erase_filter.as_ref().and_then(|(gate, collapse)| {
            if gate.is_active() {
                Some(crate::erasure::purge_lsm::build_filter(
                    gate,
                    &Arc::clone(&collapse.read()),
                ))
            } else {
                None
            }
        });

        while let Some(item) = heap.pop() {
            if let Some(ct) = cancel_token {
                if ct.is_cancelled() {
                    return Err(contextra_types::ContextraError::Internal(
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
                    let wait_start_nanos = self.clock.monotonic_nanos();
                    let max_wait_nanos =
                        u64::try_from(self.config.max_backpressure_wait.as_nanos())
                            .unwrap_or(u64::MAX);
                    while !self.budget.has_memory_capacity() {
                        let wait_elapsed_nanos = self
                            .clock
                            .monotonic_nanos()
                            .saturating_sub(wait_start_nanos);
                        if wait_elapsed_nanos >= max_wait_nanos {
                            let limit_bytes =
                                self.config.max_memory_bytes.unwrap_or(128 * 1024 * 1024);
                            let used_mb = limit_bytes.div_ceil(1024 * 1024);
                            let limit_mb = limit_bytes.div_ceil(1024 * 1024);
                            tracing::error!(
                                "Compaction aborted: backpressure wait timeout exceeded ({:?}) while memory budget exhausted",
                                self.config.max_backpressure_wait
                            );
                            return Err(contextra_types::ContextraError::MemoryBudgetExceeded {
                                used_mb,
                                limit_mb,
                            });
                        }
                        if let Some(ct) = cancel_token {
                            if ct.is_cancelled() {
                                return Err(contextra_types::ContextraError::Internal(
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
                            let start_nanos = self.clock.monotonic_nanos();
                            let merge_res = merge_op.merge(&next_item.value, &item.value);
                            let elapsed_nanos =
                                self.clock.monotonic_nanos().saturating_sub(start_nanos);
                            let timeout_nanos =
                                u64::try_from(self.config.merge_wall_clock_timeout.as_nanos())
                                    .unwrap_or(u64::MAX);

                            let merge_res = if timeout_nanos > 0 && elapsed_nanos > timeout_nanos {
                                Err(contextra_types::ContextraError::Sandbox(
                                    "Merge operator wall-clock timeout exceeded".into(),
                                ))
                            } else {
                                merge_res
                            };

                            // existing_val is older version (next_item), new_val is newer version (item)
                            match merge_res {
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
                                    let skip_erased = erase_filter_opt.as_ref().is_some_and(|f| {
                                        f.decide(item.key.as_ref(), raw_seq, is_tombstone)
                                            == crate::erasure::filter::Decision::Drop
                                    });
                                    if !skip_erased {
                                        builder
                                            .add(&item.key, &merged_bytes, item.seq, item.tx)
                                            .await?;
                                        io_bucket.charge(entry_bytes).await;
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
                // Keep all versions with raw_seq > min_snapshot_seq (visible to active or future snapshots)
                // PLUS the newest version with raw_seq <= min_snapshot_seq (the "floor" version).
                // All further, older versions for the key below min_snapshot_seq are discarded.
                let keep = if raw_seq > min_snapshot_seq {
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
                    let skip_erased = erase_filter_opt.as_ref().is_some_and(|f| {
                        f.decide(item.key.as_ref(), raw_seq, is_tombstone)
                            == crate::erasure::filter::Decision::Drop
                    });
                    if !should_gc_tombstone && !skip_erased {
                        let entry_bytes = (item.key.len() + item.value.len() + 16) as u64; // +16 für Overhead
                        builder
                            .add(&item.key, &item.value, item.seq, item.tx)
                            .await?;

                        io_bucket.charge(entry_bytes).await;
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

    /// Generates a compaction output path that does not name an existing file.
    ///
    /// The process-global counter restarts at 1 on every process start, so a fresh
    /// process can propose a name that a previous session already wrote. Such a name is
    /// skipped (never overwritten): each probe consumes one counter value, so the scan
    /// is amortised over the process lifetime (F-04).
    pub fn generate_sst_path(&self, data_path: &std::path::Path) -> Result<PathBuf> {
        // Bounded: a directory that answers "exists" for every candidate must not hang compaction.
        const MAX_NAME_PROBES: usize = 10_000;
        for _ in 0..MAX_NAME_PROBES {
            let seq = COMPACTION_SST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let count = self
                .compaction_counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let candidate =
                data_path.join(format!("sst-compact-{:020}-{:04}.sst", seq, count % 10000));
            let uuid_sidecar = PathBuf::from(format!("{}.uuid", candidate.display()));
            if !self.vfs.exists(&candidate)? && !self.vfs.exists(&uuid_sidecar)? {
                return Ok(candidate);
            }
        }
        Err(contextra_types::ContextraError::invalid_input(
            "no free compaction SSTable name found within the probe budget",
        ))
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
