use super::config::CompactionConfig;
use crate::sstable::{BlockCache, SstableBuilder, SstableReader};
use contextra_core::{Result, SnapshotRegistry, TOMBSTONE_BIT};
use crate::wal::KeyManager;
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
        Self {
            config,
            snapshot_registry,
            block_cache,
            key_manager,
            budget,
            manifest,
            compaction_counter: AtomicU64::new(0),
            pressure_rx: None,
        }
    }

    /// Attaches a system pressure watch receiver to enable pressure-aware compaction backpressure.
    pub fn with_pressure_rx(
        mut self,
        rx: tokio::sync::watch::Receiver<crate::system_pressure::SystemPressure>,
    ) -> Self {
        self.pressure_rx = Some(rx);
        self
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
            match self.select_compaction_candidates(&ssts) {
                Some(candidates) if candidates.len() >= 2 => {
                    let is_full = candidates.len() == ssts.len();
                    (candidates, is_full)
                }
                _ => return Ok(false),
            }
        };

        input_ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        tracing::info!(
            "Compaction triggered: merging {} SSTables",
            input_ssts.len()
        );

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

        // 4. Check consistency under read-lock before writing MANIFEST
        let (all_present, insertion_point, old_paths) = {
            let ssts = sstables.read().await;

            let all_present = input_ssts
                .iter()
                .all(|inp| ssts.iter().any(|sst| Arc::ptr_eq(inp, sst)));

            if !all_present {
                (false, 0, Vec::new())
            } else {
                let insertion_point = ssts
                    .iter()
                    .position(|sst| input_ssts.iter().any(|inp| Arc::ptr_eq(inp, sst)))
                    .unwrap_or(ssts.len());

                let old_paths: Vec<PathBuf> = input_ssts
                    .iter()
                    .filter_map(|inp| {
                        ssts.iter()
                            .find(|sst| Arc::ptr_eq(inp, sst))
                            .map(|sst| sst.file_path().to_path_buf())
                    })
                    .collect();

                (true, insertion_point as u64, old_paths)
            }
        };

        if !all_present {
            // Concurrent modification detected — abort compaction, clean up output file without writing MANIFEST entry
            tracing::warn!(
                "Compaction aborted: input SSTables modified during merge \
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

        // Open the new SSTable reader
        let new_reader = Arc::new(
            SstableReader::open_with_key_manager(
                &output_path,
                Arc::clone(&self.block_cache),
                self.key_manager.clone(),
            )
            .await?,
        );

        // 5. Write EXACTLY ONE atomic `Replace` entry to MANIFEST and fsync
        if let Some(ref manifest) = self.manifest {
            manifest
                .append(&crate::manifest::ManifestEntry::Replace {
                    removed: old_paths.clone(),
                    added: output_path.clone(),
                    added_max_tx: new_reader.metadata().max_tx_id,
                    rank: insertion_point,
                })
                .await?;
        }

        // 6. In-memory SSTable swap under write-lock — mirroring already-persisted MANIFEST state
        {
            let mut ssts = sstables.write().await;

            // Remove input SSTables by identity (Arc::ptr_eq), not by raw index
            ssts.retain(|sst| !input_ssts.iter().any(|inp| Arc::ptr_eq(inp, sst)));

            // Add new SSTable at insertion point
            let insert_idx = (insertion_point as usize).min(ssts.len());
            ssts.insert(insert_idx, new_reader);

            // Re-sort SSTable list by max_seq to guarantee shadowing/visibility order.
            // Non-input SSTables might lie between the oldest and newest input SSTables.
            ssts.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

            debug_assert!(
                ssts.windows(2)
                    .all(|w| (w[0].metadata().max_seq & !TOMBSTONE_BIT)
                        <= (w[1].metadata().max_seq & !TOMBSTONE_BIT)),
                "SSTable list must be sorted by max_seq in ascending order after compaction swap"
            );
        }

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
        if ssts.len() < 2 {
            return None;
        }

        // Group SSTables by size tier
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

                        if ratio <= self.config.size_ratio {
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

        // Return the most-filled tier with enough candidates (FIND-STO-002)
        // Tie-breaker: prefer tiers with smaller files (likely closer to L0)
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
            if tier.len() >= self.config.min_sstables_per_tier {
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

        // Fallback: if total SSTable count is very high, compact the smallest ones
        if ssts.len() >= self.config.min_sstables_per_tier * 2 {
            let mut by_size: Vec<(usize, u64)> = ssts
                .iter()
                .enumerate()
                .map(|(i, s)| (i, s.metadata().file_size))
                .collect();
            by_size.sort_by_key(|&(_, size)| size);
            let count = self.config.min_sstables_per_tier;
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
                    while !self.budget.has_memory_capacity() {
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

            // LSM Retention Rule:
            // Keep all versions with raw_seq >= min_snapshot_seq (visible to active or future snapshots)
            // PLUS the newest version with raw_seq < min_snapshot_seq (the "floor" version).
            // All further, older versions for the key below min_snapshot_seq are discarded.
            let keep = if raw_seq >= min_snapshot_seq {
                true
            } else if !floor_emitted {
                floor_emitted = true;
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
                                let delay =
                                    (target - elapsed).min(std::time::Duration::from_millis(100));
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
