use super::*;
use crate::compaction::CompactionEngine;
use crate::memtable::MemTable;
use crate::sstable::{create_block_cache, SstableReader};
use crate::wal::{Wal, WalOp};
use memfuse_core::{
    MemFuseError, ResourceBudget, ResourceTracker, Result, SnapshotRegistry, TxBuffer, TxId,
    TOMBSTONE_BIT,
};
use memfuse_security::crypto::KeyManager;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(super) async fn write_salt_atomically(
    salt_path: &std::path::Path,
    buf: &[u8; 32],
) -> Result<()> {
    let parent = salt_path
        .parent()
        .ok_or_else(|| MemFuseError::Storage("Invalid salt path parent".into()))?;

    let pid = std::process::id();
    let rand_val: u64 = rand::random();
    let tmp_path = parent.join(format!("SALT.tmp.{}.{}", pid, rand_val));

    let buf_copy = *buf;
    let write_res: Result<()> = async {
        let file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .await
            .map_err(|e| {
                MemFuseError::Storage(format!("Failed to create temp SALT file: {}", e))
            })?;

        let mut std_file = file.into_std().await;

        tokio::task::spawn_blocking(move || -> Result<()> {
            use std::io::Write;
            std_file
                .write_all(&buf_copy)
                .map_err(|e| MemFuseError::Storage(format!("Failed to write SALT bytes: {}", e)))?;
            std_file.sync_all().map_err(|e| {
                MemFuseError::Storage(format!("Failed to sync temp SALT file: {}", e))
            })?;
            Ok(())
        })
        .await
        .map_err(|e| MemFuseError::Storage(format!("Join error during SALT write: {}", e)))??;

        tokio::fs::rename(&tmp_path, salt_path).await.map_err(|e| {
            MemFuseError::Storage(format!("Failed to rename temp SALT file: {}", e))
        })?;

        crate::util::fsync_parent_dir(salt_path).await?;
        Ok(())
    }
    .await;

    if write_res.is_err() {
        if let Err(e) = tokio::fs::remove_file(&tmp_path).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("Failed to remove temporary SALT file {:?}: {}", tmp_path, e);
            }
        }
    }

    write_res
}

impl LsmStorage {
    pub async fn new(config: LsmConfig) -> Result<Self> {
        tokio::fs::create_dir_all(&config.path)
            .await
            .map_err(|e| MemFuseError::Storage(format!("Failed to create dir: {}", e)))?;

        // 🛡️ SICHERUNG: Directory FSync (FIND-STO-004)
        crate::util::fsync_parent_dir(&config.path).await?;

        // Persistent Salt Management (FIND-CRY-001)
        let salt_path = config.path.join("SALT");
        let salt = if let Ok(buf) = tokio::fs::read(&salt_path).await {
            if buf.len() != 32 {
                return Err(MemFuseError::Storage(format!(
                    "Invalid SALT length: expected 32, got {}",
                    buf.len()
                )));
            }
            buf
        } else {
            let mut buf = [0u8; 32];
            use rand::Rng;
            rand::thread_rng().fill(&mut buf);
            write_salt_atomically(&salt_path, &buf).await?;
            buf.to_vec()
        };

        let key_manager = config
            .encryption_passphrase
            .as_ref()
            .map(|p| KeyManager::try_new(p, &salt).map(Arc::new))
            .transpose()?;

        // AI-TAG[SMELL][MINOR] RESOLVED(audit-NC-5/u64 try_from overflow safety): Verified safe u128 -> u64 sequence parsing with try_from and fallback warning logging to ensure monotonic flush_counter initialization. (ID: AGT-STORE-5a195b0b) (TS: 2026-09-12T12:00:00Z) (SESSION: c16d73e9)
        // Discover and sort all WAL files for replay
        let mut max_wal_id: Option<u64> = None;
        let mut wal_files = Vec::new();
        let mut entries = tokio::fs::read_dir(&config.path)
            .await
            .map_err(|e| MemFuseError::Storage(format!("Failed to read data dir: {}", e)))?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("wal-") && name_str.ends_with(".log") {
                if let Ok(seq_component) = name_str[4..name_str.len() - 4].parse::<u128>() {
                    wal_files.push((seq_component, entry.path()));
                    match u64::try_from(seq_component) {
                        Ok(id) => {
                            max_wal_id = Some(max_wal_id.unwrap_or(id).max(id));
                        }
                        Err(_) => {
                            tracing::warn!(
                                path = ?entry.path(),
                                seq_component = seq_component,
                                "WAL filename sequence component overflows u64; ignoring from max_wal_id calculation"
                            );
                        }
                    }
                }
            } else if name_str == "wal.log" {
                // NC-5: Assign seq_component=0 (oldest sentinel) and ensure flush_counter starts at ≥ 1
                // to prevent wal-0.log collision on next flush after legacy migration.
                wal_files.push((0, entry.path()));
                // Update max_wal_id to at least 0 so flush_counter initializes to 1
                // AI-TAG[SMELL][MINOR] RESOLVED(clippy::map_or_identity): Simplified map_or(0, |m| m) to unwrap_or(0). (ID: AGT-STORE-cbd72ab9) (TS: 2026-09-12T12:00:00Z) (SESSION: c16d73e9)
                max_wal_id = Some(max_wal_id.unwrap_or(0));
            }
        }
        // NC-5: Stable sort with path tiebreaker prevents wal.log/wal-0.log ordering ambiguity
        wal_files.sort_by(|(ts_a, path_a), (ts_b, path_b)| {
            ts_a.cmp(ts_b).then_with(|| path_a.cmp(path_b))
        });

        let memtable = MemTable::new();
        let mut max_seq = 0u64;
        let mut max_tx = 0u64;
        let mut replayed_size = 0u64;
        let mut last_wal = None;

        for (_ts, wal_path) in &wal_files {
            let wal = Wal::open_with_key_manager(wal_path, key_manager.clone()).await?;
            let wal_entries = wal.replay().await?;

            for (lsn, entry, _offset) in &wal_entries {
                let raw_lsn = *lsn & !TOMBSTONE_BIT;
                if raw_lsn > max_seq {
                    max_seq = raw_lsn;
                }
                if entry.tx_id().inner() > max_tx && entry.tx_id().inner() < TxId::INTERNAL_BASE {
                    max_tx = entry.tx_id().inner();
                }
                match &entry.op {
                    WalOp::Put { key, value, tx_id } => {
                        replayed_size += (key.len() + value.len()) as u64;
                        memtable.put(
                            Bytes::from(key.clone()),
                            Bytes::from(value.clone()),
                            *lsn,
                            tx_id.inner(),
                        );
                    }
                    WalOp::Delete { key, tx_id } => {
                        replayed_size += key.len() as u64;
                        memtable.put(
                            Bytes::from(key.clone()),
                            Bytes::new(),
                            *lsn | TOMBSTONE_BIT,
                            tx_id.inner(),
                        );
                    }
                }
            }
            last_wal = Some(wal);
        }

        let wal = if let Some(w) = last_wal {
            w
        } else {
            // No WAL found, create a new one
            Wal::open_with_key_manager(config.path.join("wal.log"), key_manager.clone()).await?
        };

        let budget_config = ResourceBudget {
            memory_limit: config.max_ram_mb * 1024 * 1024,
        };
        let resource_tracker = Arc::new(ResourceTracker::new(budget_config));
        if replayed_size > 0 {
            if let Err(e) = resource_tracker.consume_memory(replayed_size) {
                tracing::warn!(
                    replayed_bytes = replayed_size,
                    "Memory budget tracking nach WAL-Replay fehlgeschlagen: {e}. \
                     Budget-Accounting unpräzise bis zum nächsten Flush."
                );
            }
        }

        let tx_buffer = TxBuffer::new_with_config(16, config.tx_timeout);

        // Scan for pending rollback intent files resulting from a crash during rollback_to_tx_locked
        let mut pending_rollbacks = Vec::new();
        if let Ok(mut entries) = tokio::fs::read_dir(&config.path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.starts_with("rollback-") && file_name.ends_with(".intent") {
                    let hex_part = &file_name[9..file_name.len() - 7];
                    if hex_part.len() == 16 {
                        if let Ok(target_tx) = u64::from_str_radix(hex_part, 16) {
                            tracing::error!(
                                target_tx = target_tx,
                                intent_path = ?path,
                                "Unfinished rollback intent file detected during LsmStorage startup! Recovery required."
                            );
                            pending_rollbacks.push(target_tx);
                        }
                    }
                }
            }
        }
        pending_rollbacks.sort_unstable();

        // Authoritative manifest load for SSTable verification during data directory scanning in `new()`.
        // This is the sole authorized manifest loading call during `LsmStorage::new()`. The resulting
        // `HashSet<PathBuf>` is used below to filter out unmanifested/orphaned SSTable files during data directory scanning.
        let manifest_path = config.path.join("MANIFEST");
        let manifest_exists = manifest_path.exists();
        let _valid_manifest_sstables: Option<std::collections::HashSet<std::path::PathBuf>> =
            if manifest_exists {
                if let Ok(entries) = crate::manifest::Manifest::load(&manifest_path).await {
                    Some(
                        crate::manifest::Manifest::reconstruct_valid_sstables(&entries)
                            .into_iter()
                            .collect(),
                    )
                } else {
                    None
                }
            } else {
                None
            };

        // Load existing SSTables and sort by filename (which includes seq_no)
        let mut sst_files = Vec::new();
        if let Ok(mut entries) = tokio::fs::read_dir(&config.path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.ends_with(".tmp")
                    || path.extension().is_some_and(|ext| ext == "tmp")
                    || file_name.starts_with("SALT.tmp.")
                {
                    tracing::warn!("Removing leftover un-renamed temp file: {:?}", path);
                    if let Err(e) = tokio::fs::remove_file(&path).await {
                        tracing::warn!("Failed to remove leftover temp file {:?}: {}", path, e);
                    }
                } else if path.extension().is_some_and(|ext| ext == "sst") {
                    if let Some(ref valid_set) = _valid_manifest_sstables {
                        let path_key = std::path::Path::new(file_name);
                        if valid_set.contains(path_key) {
                            sst_files.push(path);
                        } else {
                            tracing::warn!(
                                "Unmanifested or orphaned SSTable file found in data directory (skipping): {:?}",
                                path
                            );
                        }
                    } else {
                        sst_files.push(path);
                    }
                }
            }
        }
        sst_files.sort();

        let block_cache = create_block_cache(64); // 64MB block cache for SSTables

        let mut sstables = Vec::new();
        for path in sst_files {
            let reader = SstableReader::open_with_key_manager(
                path,
                Arc::clone(&block_cache),
                key_manager.clone(),
            )
            .await?;

            // Recover max_seq and max_tx from SSTables
            let raw_sst_max_seq = reader.metadata().max_seq & !TOMBSTONE_BIT;
            if raw_sst_max_seq > max_seq {
                max_seq = raw_sst_max_seq;
            }
            if reader.metadata().max_tx_id > max_tx {
                // Ignore internal TxIds during recovery
                if reader.metadata().max_tx_id < TxId::INTERNAL_BASE {
                    max_tx = reader.metadata().max_tx_id;
                }
            }

            sstables.push(Arc::new(reader));
        }
        // Explicitly sort SSTables by metadata().max_seq for guaranteed read-path ordering
        sstables.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);
        let sstables = Arc::new(RwLock::new(sstables));

        // Open or migrate manifest
        let manifest = Arc::new(crate::manifest::Manifest::open(&manifest_path).await?);
        if !manifest_exists {
            let ssts_read = sstables.read().await;
            for sst in ssts_read.iter() {
                manifest
                    .append(&crate::manifest::ManifestEntry::Add {
                        path: sst.file_path().to_path_buf(),
                        max_tx: sst.metadata().max_tx_id,
                    })
                    .await?;
            }
        }
        // === SSTABLE MANIFEST INTEGRATION END ===

        let snapshot_registry = Arc::new(SnapshotRegistry::new());

        let cancel_token = tokio_util::sync::CancellationToken::new();
        let task_tracker = tokio_util::task::TaskTracker::new();

        // INV-LSM-2: SystemPressureMonitor must be spawned as a background task before LsmStorage accepts its first insert.
        let wal_queue_depth = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let wal_queue_depth_clone = Arc::clone(&wal_queue_depth);

        let pressure_monitor = crate::system_pressure::SystemPressureMonitor::new(
            std::time::Duration::from_millis(100),
        );
        let pressure_rx = pressure_monitor.pressure_rx.clone();
        let ct_pressure = cancel_token.clone();
        task_tracker.spawn(async move {
            // Note: embedding_permits_fn and max_embedding_permits are passed as || 0, 0 because memfuse-store
            // is a standalone KV storage engine crate decoupled from vector embedding subsystems. Embedding backpressure
            // is managed at higher layers (e.g. Collection/TextEmbedder in memfuse-embed / memfuse-db).
            pressure_monitor
                .run(
                    ct_pressure,
                    move || wal_queue_depth_clone.load(Ordering::Relaxed),
                    || 0,
                    0,
                )
                .await;
        });

        // Spawn background compaction task
        // COMP-001 — Implementiere CompactionEngine::run_loop.
        // TEST: cargo test -p memfuse-store test_concurrent_reads_during_compaction
        // DONE: Triple-Test grün, keine Deadlocks in tokio::spawn.
        // AI-TAG[SMELL][RESOLVED] audit-C-1: Startup-Flush erzwungen vor Löschung alter WAL-Dateien in LsmStorage::new (Zeilen ~480-490).
        let compaction_engine = Arc::new(
            CompactionEngine::new(
                config.compaction.clone(),
                Arc::clone(&snapshot_registry),
                Arc::clone(&block_cache),
                key_manager.clone(),
                Arc::clone(&resource_tracker),
                Some(Arc::clone(&manifest)),
            )
            .with_pressure_rx(pressure_rx.clone()),
        );
        // Clone für den Hintergrund-Task, original bleibt als Struct-Feld
        let compaction_engine_for_loop = Arc::clone(&compaction_engine);
        let compaction_sstables = Arc::clone(&sstables);
        let compaction_path = config.path.clone();

        let ct_clone = cancel_token.clone();
        task_tracker.spawn(async move {
            compaction_engine_for_loop
                .run_loop(compaction_sstables, compaction_path, ct_clone)
                .await;
        });
        task_tracker.close();

        let storage = Self {
            config,
            key_manager,
            state: RwLock::new(LsmState {
                memtable: Arc::new(memtable),
                immutable_memtables: Vec::new(),
            }),
            sstables,
            tx_buffer,
            budget: resource_tracker,
            block_cache,
            wal: RwLock::new(Arc::new(wal)),
            snapshot_registry,
            compaction_engine, // H-3: persistent field
            manifest,
            next_seq_no: AtomicU64::new(max_seq.saturating_add(1)),
            last_committed_tx: AtomicU64::new(max_tx),
            commit_mutex: tokio::sync::Mutex::new(()),
            cancel_token,
            task_tracker,
            flush_counter: AtomicU64::new(max_wal_id.map(|m| m.saturating_add(1)).unwrap_or(0)),
            segment_counter: AtomicU64::new(0),
            budget_tracking_drift_bytes: std::sync::atomic::AtomicU64::new(0),
            pending_commit_queue: tokio::sync::Mutex::new(None),
            wal_queue_depth,
            pressure_rx,
            intent_locks: std::sync::Mutex::new(std::collections::HashMap::new()),
        };

        // Flush after WAL replay regardless of WAL count — ensures replayed data is persisted to SSTable before any WAL rotation/deletion can occur.
        if replayed_size > 0 && !wal_files.is_empty() {
            tracing::info!(
                replayed_bytes = replayed_size,
                "Forcing startup flush to persist replayed WAL entries before old WAL cleanup"
            );
            storage.flush().await.map_err(|e| {
                MemFuseError::Storage(format!("Startup flush after WAL replay failed: {e}"))
            })?;
        }

        // WAL cleanup: now safe, replayed data is on disk in SSTable
        if wal_files.len() > 1 {
            let active_wal_path = {
                let wal = storage.wal.read().await;
                wal.path().to_path_buf()
            };
            for (_ts, old_wal_path) in &wal_files[..wal_files.len() - 1] {
                if old_wal_path != &active_wal_path {
                    if let Err(e) = tokio::fs::remove_file(old_wal_path).await {
                        tracing::warn!("Failed to remove old WAL file {:?}: {}", old_wal_path, e);
                    } else {
                        tracing::info!("Removed old replayed WAL file: {:?}", old_wal_path);
                        // NC-4: Remove .uuid sidecar file alongside WAL
                        let uuid_sidecar =
                            PathBuf::from(format!("{}.uuid", old_wal_path.display()));
                        if let Err(e) = tokio::fs::remove_file(&uuid_sidecar).await {
                            tracing::debug!(
                                "Could not remove WAL UUID sidecar {:?}: {} (non-critical)",
                                uuid_sidecar,
                                e
                            );
                        }
                    }
                }
            }
        }

        // Replay any pending rollbacks detected during startup.
        // APM-PARTIAL-ORDER-VIOLATION Note:
        // SSTables and WAL MUST be loaded first into `storage` before calling `rollback_to_tx`,
        // because `rollback_to_tx_locked()` inspects, filters, and recompacts the loaded SSTables
        // and WAL in memory to physically remove rolled-back entries.
        for target_tx in pending_rollbacks {
            tracing::info!(
                target_tx = target_tx,
                "Executing pending rollback recovery for TxId({})",
                target_tx
            );
            storage
                .rollback_to_tx(TxId::new(target_tx))
                .await
                .map_err(|e| {
                    MemFuseError::Storage(format!(
                        "Startup rollback recovery failed for target_tx {}: {}. Please inspect data directory '{:?}' manually.",
                        target_tx,
                        e,
                        storage.config.path
                    ))
                })?;
            tracing::info!(
                target_tx = target_tx,
                "Successfully completed pending rollback recovery for TxId({})",
                target_tx
            );
        }

        Ok(storage)
    }

    /// Removes all intent lock registrations associated with the specified transaction ID.

    /// Rolls back the entire storage state to a specific transaction ID.
    /// This is a destructive operation that removes all data after the target TX.
    // AI-TAG[SMELL][MINOR] RESOLVED(audit-M-4): Below MIN_ENTRIES_FOR_SSTABLE_REBUILD (8), surviving entries from spanning SSTables during rollback are inserted directly into active MemTable rather than writing a new SSTable file. (ID: AGT-STORE-1f3c3709) (TS: 2026-09-12T12:00:00Z) (SESSION: c16d73e9)
    pub async fn rollback_to_tx(&self, target_tx: TxId) -> Result<()> {
        let _commit_lock = self.commit_mutex.lock().await;
        let commit_guard = CommitGuard {
            _lock: &_commit_lock,
        };
        self.rollback_to_tx_locked(target_tx, &commit_guard).await
    }

    /// Internal rollback implementation.
    ///
    /// # Safety / Concurrency Invariant
    /// **MUST ONLY** be called while holding `commit_mutex`. Calling this function without
    /// holding `commit_mutex` violates lock ordering and leads to state corruption and race conditions.
    // AI-TAG[SMELL][MINOR] RESOLVED(audit-NC-3/C-4): Rollback transaction crash-atomicity via rollback-{txid}.intent file and startup recovery confirmed fully operational in LsmStorage::new() and verified by test_rollback_crash_recovery_startup. (ID: AGT-STORE-27a11909) (TS: 2026-09-11T19:30:00Z) (SESSION: 4a9ccf21)
    pub(super) async fn rollback_to_tx_locked(
        &self,
        target_tx: TxId,
        _guard: &CommitGuard<'_>,
    ) -> Result<()> {
        self.clear_intent_locks_above_tx(target_tx);

        // NC-3-RECOVERY-NOTE: Implement recovery in P1 fix/lsm-startup-recovery
        // NC-3: Write crash-atomic rollback intent file before any mutation.
        // On recovery in new(), this file signals that rollback must be completed.
        let intent_path = self
            .config
            .path
            .join(format!("rollback-{:016x}.intent", target_tx.inner()));
        {
            const INTENT_MAGIC: &[u8] = b"MFRLBK\0\0";
            let mut intent_bytes = Vec::with_capacity(16);
            intent_bytes.extend_from_slice(INTENT_MAGIC);
            intent_bytes.extend_from_slice(&target_tx.inner().to_le_bytes());
            tokio::fs::write(&intent_path, &intent_bytes)
                .await
                .map_err(|e| {
                    MemFuseError::Storage(format!("Failed to write rollback intent file: {e}"))
                })?;
            // fsync parent directory to persist the intent file entry
            let parent = self.config.path.clone();
            tokio::task::spawn_blocking(move || {
                std::fs::File::open(&parent)
                    .and_then(|f| f.sync_all())
                    .map_err(|e| {
                        MemFuseError::Storage(format!("Failed to fsync dir after intent file: {e}"))
                    })
            })
            .await
            .map_err(|e| MemFuseError::Internal(e.to_string()))??;
        }

        let mut state = self.state.write().await;
        let wal = self.wal.read().await.clone();

        // 1. Truncate WAL to the position after target_tx
        let (target_offset, target_hmac) = wal.find_tx_offset(target_tx).await?;
        wal.truncate(target_offset, target_hmac).await?;

        // 2. Clear current memtable (it might have data > target_tx)
        state.memtable = Arc::new(MemTable::new());
        state.immutable_memtables.clear();

        // 3. Handle SSTables: Remove SSTables that are entirely newer than target_tx,
        // and recompact SSTables that span target_tx to physically delete post-rollback entries.
        let mut sstables_lock = self.sstables.write().await;
        let mut sst_to_remove = Vec::new();
        let mut spanning_sstables = Vec::new();

        sstables_lock.retain(|sst| {
            let meta = sst.metadata();
            if meta.min_tx_id > target_tx.inner() {
                sst_to_remove.push(sst.file_path().to_path_buf());
                false
            } else if meta.min_tx_id <= target_tx.inner() && meta.max_tx_id > target_tx.inner() {
                spanning_sstables.push(Arc::clone(sst));
                false
            } else {
                true
            }
        });

        for spanning in spanning_sstables {
            let mut surviving_entries = Vec::new();
            let mut stream = spanning.stream().await?;
            while let Some((k, v, seq, tx)) = stream.next_entry().await? {
                if tx <= target_tx.inner() {
                    surviving_entries.push((k, v, seq, tx));
                }
            }

            if surviving_entries.len() >= MIN_ENTRIES_FOR_SSTABLE_REBUILD {
                let count = self.segment_counter.fetch_add(1, Ordering::Relaxed);
                let seq = self.next_seq_no.load(Ordering::Relaxed);
                let new_sst_path =
                    self.config
                        .path
                        .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));

                let mut builder = SstableBuilder::create_with_key_manager(
                    &new_sst_path,
                    self.key_manager.clone(),
                )
                .await?;

                for (k, v, seq, tx) in surviving_entries {
                    builder.add(&k, &v, seq, tx).await?;
                }
                builder.finish().await?;

                let new_reader = SstableReader::open_with_key_manager(
                    &new_sst_path,
                    Arc::clone(&self.block_cache),
                    self.key_manager.clone(),
                )
                .await?;

                // === SSTABLE MANIFEST INTEGRATION START ===
                self.manifest
                    .append(&crate::manifest::ManifestEntry::Add {
                        path: new_sst_path.clone(),
                        max_tx: new_reader.metadata().max_tx_id,
                    })
                    .await?;
                // === SSTABLE MANIFEST INTEGRATION END ===

                sstables_lock.push(Arc::new(new_reader));
            } else if !surviving_entries.is_empty() {
                // Below MIN_ENTRIES_FOR_SSTABLE_REBUILD: insert surviving entries directly into memtable
                for (k, v, seq, tx) in surviving_entries {
                    state.memtable.put(k, v, seq, tx);
                }
            }
            // If surviving_entries <= ROLLBACK_INLINE_THRESHOLD_ENTRIES (e.g. small/single-entry),
            // we do not create a new SSTable. The surviving entries will be re-populated
            // directly into the memtable via replay of the truncated WAL in step 5.

            sst_to_remove.push(spanning.file_path().to_path_buf());
        }

        sstables_lock.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        // 4. Update next_seq_no and last_committed_tx
        // Find max_seq from kept SSTables to avoid regressing next_seq_no
        // TOMBSTONE_BIT-Disziplin (DECISIONS.md): Bit 63 darf niemals in next_seq_no einfließen.
        let mut max_seq = 0;
        for sst in sstables_lock.iter() {
            max_seq = max_seq.max(sst.metadata().max_seq & !TOMBSTONE_BIT);
        }
        drop(sstables_lock);

        for path in sst_to_remove {
            tracing::info!("Removing SSTable during rollback: {:?}", path);
            // === SSTABLE MANIFEST INTEGRATION START ===
            if let Err(e) = self
                .manifest
                .append(&crate::manifest::ManifestEntry::Remove { path: path.clone() })
                .await
            {
                tracing::warn!(
                    "Failed to write Manifest Remove entry during rollback: {}",
                    e
                );
            }
            // === SSTABLE MANIFEST INTEGRATION END ===
            // Best-effort cleanup: do not abort rollback recovery if file removal fails.
            // The SSTable is superseded by restored WAL replay state, so its orphaned presence is safe but wastes disk space.
            if let Err(e) = tokio::fs::remove_file(&path).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::error!(
                        path = ?path,
                        "Orphaned SSTable konnte nicht entfernt werden: {e}. Manuelles Cleanup nötig."
                    );
                }
            }
        }

        // === SSTABLE MANIFEST INTEGRATION START ===
        self.manifest
            .append(&crate::manifest::ManifestEntry::RollbackComplete {
                target_tx: target_tx.inner(),
            })
            .await?;
        // === SSTABLE MANIFEST INTEGRATION END ===

        // NC-3: Remove rollback intent file after all SST cleanup is complete.
        // If this removal fails, recovery on next startup will re-execute the (idempotent) rollback.
        if let Err(e) = tokio::fs::remove_file(&intent_path).await {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    "Could not remove rollback intent file {:?}: {} \
                     (non-fatal — recovery will re-run on next startup)",
                    intent_path,
                    e
                );
            }
        }

        // 5. Re-populate memtable from truncated WAL
        let entries = wal.replay().await?;
        // TOMBSTONE_BIT-Disziplin (DECISIONS.md): Bit 63 darf niemals in next_seq_no einfließen.
        for (seq, entry, _offset) in entries {
            if (seq & !TOMBSTONE_BIT) > max_seq {
                max_seq = seq & !TOMBSTONE_BIT;
            }
            match entry.op {
                WalOp::Put { key, value, tx_id } => {
                    state.memtable.put(
                        bytes::Bytes::from(key),
                        bytes::Bytes::from(value),
                        seq,
                        tx_id.inner(),
                    );
                }
                WalOp::Delete { key, tx_id } => {
                    state.memtable.put(
                        bytes::Bytes::from(key),
                        bytes::Bytes::new(),
                        seq | TOMBSTONE_BIT,
                        tx_id.inner(),
                    );
                }
            }
        }

        self.next_seq_no.store(max_seq + 1, Ordering::SeqCst);
        self.last_committed_tx
            .store(target_tx.inner(), Ordering::SeqCst);

        {
            let mut locks = self.intent_locks.lock().unwrap_or_else(|e| e.into_inner());
            locks.retain(|_, v| v.inner() <= target_tx.inner());
        }

        tracing::info!(
            "Rollback to TX {} successful. Max seq: {}, WAL offset: {}",
            target_tx.inner(),
            max_seq,
            target_offset
        );

        Ok(())
    }
}
