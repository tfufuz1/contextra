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

        crate::util::fsync_parent_dir(&config.path).await?;

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
                wal_files.push((0, entry.path()));
                max_wal_id = Some(max_wal_id.unwrap_or(0));
            }
        }

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

        let mut sst_files = Vec::new();
        if let Ok(mut entries) = tokio::fs::read_dir(&config.path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.ends_with(".tmp")
                    || path.extension().is_some_and(|ext| ext == "tmp")
                    || file_name.starts_with("SALT.tmp.")
                    || file_name.starts_with("MANIFEST.new.")
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

        let block_cache = create_block_cache(64);

        let mut sstables = Vec::new();
        for path in sst_files {
            let reader = SstableReader::open_with_key_manager(
                path,
                Arc::clone(&block_cache),
                key_manager.clone(),
            )
            .await?;

            let raw_sst_max_seq = reader.metadata().max_seq & !TOMBSTONE_BIT;
            if raw_sst_max_seq > max_seq {
                max_seq = raw_sst_max_seq;
            }
            if reader.metadata().max_tx_id > max_tx {
                if reader.metadata().max_tx_id < TxId::INTERNAL_BASE {
                    max_tx = reader.metadata().max_tx_id;
                }
            }

            sstables.push(Arc::new(reader));
        }

        sstables.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);
        let sstables = Arc::new(RwLock::new(sstables));

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

        let snapshot_registry = Arc::new(SnapshotRegistry::new());

        let cancel_token = tokio_util::sync::CancellationToken::new();
        let task_tracker = tokio_util::task::TaskTracker::new();

        let wal_queue_depth = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let wal_queue_depth_clone = Arc::clone(&wal_queue_depth);

        let pressure_monitor = crate::system_pressure::SystemPressureMonitor::new(
            std::time::Duration::from_millis(100),
        );
        let pressure_rx = pressure_monitor.pressure_rx.clone();
        let ct_pressure = cancel_token.clone();
        task_tracker.spawn(async move {
            pressure_monitor
                .run(
                    ct_pressure,
                    move || wal_queue_depth_clone.load(Ordering::Relaxed),
                    || 0,
                    0,
                )
                .await;
        });

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
            compaction_engine,
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

        if replayed_size > 0 && !wal_files.is_empty() {
            tracing::info!(
                replayed_bytes = replayed_size,
                "Forcing startup flush to persist replayed WAL entries before old WAL cleanup"
            );
            storage.flush().await.map_err(|e| {
                MemFuseError::Storage(format!("Startup flush after WAL replay failed: {e}"))
            })?;
        }

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

    pub async fn rollback_to_tx(&self, target_tx: TxId) -> Result<()> {
        let _commit_lock = self.commit_mutex.lock().await;
        let commit_guard = CommitGuard {
            _lock: &_commit_lock,
        };
        self.rollback_to_tx_locked(target_tx, &commit_guard).await
    }

    pub(super) async fn rollback_to_tx_locked(
        &self,
        target_tx: TxId,
        _guard: &CommitGuard<'_>,
    ) -> Result<()> {
        self.clear_intent_locks_above_tx(target_tx);

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

        let (target_offset, target_hmac) = wal.find_tx_offset(target_tx).await?;
        wal.truncate(target_offset, target_hmac).await?;

        state.memtable = Arc::new(MemTable::new());
        state.immutable_memtables.clear();

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

                self.manifest
                    .append(&crate::manifest::ManifestEntry::Add {
                        path: new_sst_path.clone(),
                        max_tx: new_reader.metadata().max_tx_id,
                    })
                    .await?;

                sstables_lock.push(Arc::new(new_reader));
            } else if !surviving_entries.is_empty() {
                for (k, v, seq, tx) in surviving_entries {
                    state.memtable.put(k, v, seq, tx);
                }
            }

            sst_to_remove.push(spanning.file_path().to_path_buf());
        }

        sstables_lock.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        let mut max_seq = 0;
        for sst in sstables_lock.iter() {
            max_seq = max_seq.max(sst.metadata().max_seq & !TOMBSTONE_BIT);
        }
        drop(sstables_lock);

        for path in sst_to_remove {
            tracing::info!("Removing SSTable during rollback: {:?}", path);
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
            if let Err(e) = tokio::fs::remove_file(&path).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    tracing::error!(
                        path = ?path,
                        "Orphaned SSTable konnte nicht entfernt werden: {e}. Manuelles Cleanup nötig."
                    );
                }
            }
        }

        self.manifest
            .append(&crate::manifest::ManifestEntry::RollbackComplete {
                target_tx: target_tx.inner(),
            })
            .await?;

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

        let entries = wal.replay().await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn test_storage() -> (LsmStorage, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");
        (storage, tmp)
    }

    #[tokio::test]
    async fn test_lsm_rollback_persistence() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };

        {
            let storage = LsmStorage::new(config.clone())
                .await
                .expect("create storage");

            let tx1 = TxId::new(1);
            storage.put(tx1, b"k1", b"v1").await.unwrap();
            storage.commit(tx1).await.unwrap();

            let tx2 = TxId::new(2);
            storage.put(tx2, b"k2", b"v2").await.unwrap();
            storage.commit(tx2).await.unwrap();

            assert_eq!(storage.get(b"k1").await.unwrap(), Some(b"v1".to_vec()));
            assert_eq!(storage.get(b"k2").await.unwrap(), Some(b"v2".to_vec()));

            storage.rollback_to_tx(tx1).await.expect("rollback");

            assert_eq!(storage.get(b"k1").await.unwrap(), Some(b"v1".to_vec()));
            assert_eq!(storage.get(b"k2").await.unwrap(), None);
        }

        {
            let storage = LsmStorage::new(config).await.expect("restart storage");
            assert_eq!(storage.get(b"k1").await.unwrap(), Some(b"v1".to_vec()));
            assert_eq!(
                storage.get(b"k2").await.unwrap(),
                None,
                "k2 should NOT be replayed after rollback"
            );

            let tx3 = TxId::new(3);
            storage.put(tx3, b"k3", b"v3").await.unwrap();
            storage.commit(tx3).await.unwrap();
            assert_eq!(storage.get(b"k3").await.unwrap(), Some(b"v3".to_vec()));
        }
    }

    #[tokio::test]
    async fn test_rollback_with_sstables() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, b"k1", b"v1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k2", b"v2").await.unwrap();
        storage.commit(tx2).await.unwrap();

        storage.force_flush().await.unwrap();

        let tx3 = TxId::new(3);
        storage.put(tx3, b"k3", b"v3").await.unwrap();
        storage.commit(tx3).await.unwrap();

        let tx4 = TxId::new(4);
        storage.put(tx4, b"k4", b"v4").await.unwrap();
        storage.commit(tx4).await.unwrap();

        storage.force_flush().await.unwrap();

        {
            let sstables = storage.sstables.read().await;
            assert_eq!(sstables.len(), 2);
        }

        storage.rollback_to_tx(tx2).await.expect("rollback");

        {
            let sstables = storage.sstables.read().await;
            assert_eq!(sstables.len(), 1, "SSTable 2 should be deleted");
            assert_eq!(sstables[0].metadata().max_tx_id, 2);
        }

        assert_eq!(storage.get(b"k1").await.unwrap(), Some(b"v1".to_vec()));
        let val2 = storage.get(b"k2").await.unwrap();
        let ssts = storage.sstables.read().await;
        let sst_meta = if !ssts.is_empty() {
            format!(
                "min_tx: {}, max_tx: {}, range: [{:?}, {:?}]",
                ssts[0].metadata().min_tx_id,
                ssts[0].metadata().max_tx_id,
                ssts[0].metadata().first_key,
                ssts[0].metadata().last_key
            )
        } else {
            "NO SSTABLES".into()
        };
        assert_eq!(
            val2,
            Some(b"v2".to_vec()),
            "k2 should be found. SST 0 meta: {}",
            sst_meta
        );
        assert_eq!(storage.get(b"k3").await.unwrap(), None);
        assert_eq!(storage.get(b"k4").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_rollback_recompacts_spanning_sstable() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        for i in 1..=15u64 {
            let tx = TxId::new(i);
            let key = format!("k{:02}", i);
            let val = format!("v{:02}", i);
            storage
                .put(tx, key.as_bytes(), val.as_bytes())
                .await
                .unwrap();
            storage.commit(tx).await.unwrap();
        }
        storage.force_flush().await.unwrap();

        {
            let sstables = storage.sstables.read().await;
            assert_eq!(sstables.len(), 1);
            assert_eq!(sstables[0].metadata().min_tx_id, 1);
            assert_eq!(sstables[0].metadata().max_tx_id, 15);
        }

        storage
            .rollback_to_tx(TxId::new(10))
            .await
            .expect("rollback");

        {
            let sstables = storage.sstables.read().await;
            assert_eq!(
                sstables.len(),
                1,
                "Spanning SSTable should be recompacted into 1 new SSTable"
            );
            assert_eq!(sstables[0].metadata().max_tx_id, 10);

            let mut count = 0;
            let mut stream = sstables[0].stream().await.unwrap();
            while let Some((_k, _v, _seq, tx)) = stream.next_entry().await.unwrap() {
                assert!(
                    tx <= 10,
                    "SSTable on disk must not contain entries with tx_id > 10"
                );
                count += 1;
            }
            assert_eq!(
                count, 10,
                "Surviving on-disk entry count must equal exactly 10"
            );
        }

        for i in 1..=10u64 {
            let key = format!("k{:02}", i);
            let expected = format!("v{:02}", i);
            let val = storage.get(key.as_bytes()).await.unwrap();
            assert_eq!(val, Some(expected.into_bytes()));
        }

        for i in 11..=15u64 {
            let key = format!("k{:02}", i);
            let val = storage.get(key.as_bytes()).await.unwrap();
            assert_eq!(val, None);
        }
    }

    #[tokio::test]
    async fn test_rollback_drops_sstable_fully_stale_after_recompaction() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        for i in 10..=15u64 {
            let tx = TxId::new(i);
            let key = format!("k{:02}", i);
            let val = format!("v{:02}", i);
            storage
                .put(tx, key.as_bytes(), val.as_bytes())
                .await
                .unwrap();
            storage.commit(tx).await.unwrap();
        }
        storage.force_flush().await.unwrap();

        storage
            .rollback_to_tx(TxId::new(5))
            .await
            .expect("rollback");

        {
            let sstables = storage.sstables.read().await;
            assert!(sstables.is_empty(), "Fully stale SSTable must be dropped");
        }
    }

    #[tokio::test]
    async fn test_wal_survives_process_restart() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };

        {
            let storage = LsmStorage::new(config.clone())
                .await
                .expect("create storage");
            let tx = TxId::new(1);
            storage
                .put(tx, b"persistent_key", b"persistent_val")
                .await
                .expect("put");
            storage.commit(tx).await.expect("commit");
        }

        {
            let storage = LsmStorage::new(config).await.expect("reopen storage");
            let val = storage.get(b"persistent_key").await.expect("get");
            assert_eq!(val, Some(b"persistent_val".to_vec()));
        }
    }

    #[tokio::test]
    async fn test_rollback_to_tx_edge_cases() {
        let (storage, _tmp) = test_storage().await;

        let res = storage.rollback_to_tx(TxId::new(999)).await;
        assert!(res.is_ok());

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.expect("put");
        storage.commit(tx1).await.expect("commit");

        storage
            .rollback_to_tx(TxId::new(0))
            .await
            .expect("rollback to 0");
        assert_eq!(storage.get(b"key1").await.expect("get"), None);
    }

    #[tokio::test]
    async fn test_rollback_tombstone_sstable() {
        let (storage, _tmp) = test_storage().await;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key2", b"val2").await.unwrap();
        storage.commit(tx2).await.unwrap();

        let tx3 = TxId::new(3);
        storage.delete(tx3, b"key2").await.unwrap();
        storage.commit(tx3).await.unwrap();
        storage.force_flush().await.unwrap();

        storage.rollback_to_tx(tx3).await.unwrap();

        let tx4 = TxId::new(4);
        storage.put(tx4, b"key3", b"val3").await.unwrap();
        storage.commit(tx4).await.unwrap();

        let current_max_seq = storage.next_seq_no.load(Ordering::Acquire);
        let val = storage.get_at_seq(b"key3", current_max_seq).await.unwrap();
        assert_eq!(val, Some(b"val3".to_vec()));

        let last_seq = storage.last_seq_no().await.unwrap();
        assert_eq!(
            last_seq & TOMBSTONE_BIT,
            0,
            "Sequence number of new insert must not have TOMBSTONE_BIT set"
        );
    }

    #[tokio::test]
    async fn test_rollback_tombstone_wal() {
        let (storage, _tmp) = test_storage().await;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"k1", b"v1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k2", b"v2").await.unwrap();
        storage.commit(tx2).await.unwrap();

        let tx3 = TxId::new(3);
        storage.delete(tx3, b"k2").await.unwrap();
        storage.commit(tx3).await.unwrap();

        storage.rollback_to_tx(tx3).await.unwrap();

        let tx4 = TxId::new(4);
        storage.put(tx4, b"k3", b"v3").await.unwrap();
        storage.commit(tx4).await.unwrap();

        let current_max_seq = storage.next_seq_no.load(Ordering::Acquire);
        let val = storage.get_at_seq(b"k3", current_max_seq).await.unwrap();
        assert_eq!(val, Some(b"v3".to_vec()));

        let last_seq = storage.last_seq_no().await.unwrap();
        assert_eq!(
            last_seq & TOMBSTONE_BIT,
            0,
            "Sequence number of new insert must not have TOMBSTONE_BIT set"
        );
    }

    #[tokio::test]
    async fn test_rollback_tombstone_subsequent_ops() {
        let (storage, _tmp) = test_storage().await;

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.delete(tx2, b"key1").await.unwrap();
        storage.commit(tx2).await.unwrap();

        storage.rollback_to_tx(tx2).await.unwrap();

        let tx3 = TxId::new(3);
        storage.put(tx3, b"key2", b"val2").await.unwrap();
        storage.commit(tx3).await.unwrap();

        let tx4 = TxId::new(4);
        storage.delete(tx4, b"key2").await.unwrap();
        storage.commit(tx4).await.unwrap();

        let tx5 = TxId::new(5);
        storage.put(tx5, b"key3", b"val3").await.unwrap();
        storage.commit(tx5).await.unwrap();

        let state = storage.state.read().await;
        for (k, _v, seq, _tx) in state.memtable.iter_latest() {
            if k.as_ref() == b"key1" || k.as_ref() == b"key2" {
                assert_ne!(
                    seq & TOMBSTONE_BIT,
                    0,
                    "Latest entry for deleted key {:?} must have TOMBSTONE_BIT set",
                    String::from_utf8_lossy(&k)
                );
            } else if k.as_ref() == b"key3" {
                assert_eq!(
                    seq & TOMBSTONE_BIT,
                    0,
                    "Latest entry for inserted key {:?} must NOT have TOMBSTONE_BIT set",
                    String::from_utf8_lossy(&k)
                );
            }
        }
        drop(state);

        assert_eq!(storage.get(b"key1").await.unwrap(), None);
        assert_eq!(storage.get(b"key2").await.unwrap(), None);
        assert_eq!(storage.get(b"key3").await.unwrap(), Some(b"val3".to_vec()));
    }

    #[tokio::test]
    async fn test_recovery_scan_ignores_and_removes_tmp_files() {
        let tmp = tempfile::TempDir::new().unwrap();
        let corrupt_tmp_path = tmp.path().join("sst-compact-corrupt.sst.tmp");
        tokio::fs::write(&corrupt_tmp_path, b"invalid sst data from crash")
            .await
            .unwrap();

        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: std::time::Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };

        let storage = LsmStorage::new(config)
            .await
            .expect("LsmStorage startup must succeed");
        assert_eq!(storage.sstables.read().await.len(), 0);

        assert!(
            !corrupt_tmp_path.exists(),
            "Leftover .tmp file must be removed during startup recovery scan"
        );
    }

    #[tokio::test]
    async fn test_startup_flush_before_wal_cleanup() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };

        {
            let storage = LsmStorage::new(config.clone())
                .await
                .expect("create initial storage");
            let tx1 = TxId::new(1);
            storage.put(tx1, b"key1", b"val1").await.unwrap();
            storage.commit(tx1).await.unwrap();

            let tx2 = TxId::new(2);
            storage.put(tx2, b"key2", b"val2").await.unwrap();
            storage.commit(tx2).await.unwrap();
        }

        {
            let storage = LsmStorage::new(config.clone())
                .await
                .expect("reopen storage after crash/restart");

            assert_eq!(storage.get(b"key1").await.unwrap(), Some(b"val1".to_vec()));
            assert_eq!(storage.get(b"key2").await.unwrap(), Some(b"val2".to_vec()));

            let stats = storage.stats().await.unwrap();
            assert!(
                stats.num_segments >= 1,
                "Startup flush must persist replayed WAL entries into SSTable"
            );
        }

        {
            let storage = LsmStorage::new(config)
                .await
                .expect("reopen storage after second crash");
            assert_eq!(storage.get(b"key1").await.unwrap(), Some(b"val1".to_vec()));
            assert_eq!(storage.get(b"key2").await.unwrap(), Some(b"val2".to_vec()));
        }
    }

    #[tokio::test]
    async fn test_rollback_small_tx_inline_no_sstable() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            group_commit_window_micros: 0,
        };

        let storage = LsmStorage::new(config.clone())
            .await
            .expect("create storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.unwrap();
        storage.commit(tx1).await.unwrap();
        storage.force_flush().await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key2", b"val2").await.unwrap();
        storage.commit(tx2).await.unwrap();

        let sst_count_before = storage.sstables.read().await.len();
        assert_eq!(sst_count_before, 1);

        storage.rollback_to_tx(tx1).await.unwrap();

        assert_eq!(storage.get(b"key1").await.unwrap(), Some(b"val1".to_vec()));
        assert_eq!(storage.get(b"key2").await.unwrap(), None);

        let sst_count_after = storage.sstables.read().await.len();
        assert!(
            sst_count_after <= sst_count_before,
            "Small transaction rollback must not produce new SSTables"
        );
    }

    #[tokio::test]
    async fn test_wal_uuid_sidecar_cleaned_up_on_startup() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };

        {
            let storage = LsmStorage::new(config.clone())
                .await
                .expect("create storage");
            let tx1 = TxId::new(1);
            storage.put(tx1, b"key1", b"val1").await.unwrap();
            storage.commit(tx1).await.unwrap();
            storage.force_flush().await.unwrap();

            let tx2 = TxId::new(2);
            storage.put(tx2, b"key2", b"val2").await.unwrap();
            storage.commit(tx2).await.unwrap();
        }

        let uuid_path = tmp.path().join("wal-00000000000000000000.log.uuid");
        tokio::fs::write(&uuid_path, b"test-uuid-content")
            .await
            .unwrap();
        let wal1_path = tmp.path().join("wal-00000000000000000001.log");
        tokio::fs::write(&wal1_path, b"").await.unwrap();
        assert!(
            uuid_path.exists(),
            "Dummy .uuid file must exist before startup cleanup"
        );

        {
            let _storage = LsmStorage::new(config).await.expect("reopen storage");

            assert!(
                !uuid_path.exists(),
                "WAL .uuid sidecar file must be cleaned up during startup recovery"
            );
        }
    }

    #[tokio::test]
    async fn test_rollback_crash_recovery_startup() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            group_commit_window_micros: 0,
        };

        let tx1 = TxId::new(1);
        let tx2 = TxId::new(2);
        let tx3 = TxId::new(3);

        {
            let storage = LsmStorage::new(config.clone())
                .await
                .expect("create storage");

            storage.put(tx1, b"key1", b"val1").await.unwrap();
            storage.commit(tx1).await.unwrap();
            storage.force_flush().await.unwrap();

            storage.put(tx2, b"key2", b"val2").await.unwrap();
            storage.commit(tx2).await.unwrap();
            storage.force_flush().await.unwrap();

            storage.put(tx3, b"key3", b"val3").await.unwrap();
            storage.commit(tx3).await.unwrap();
            storage.close().await.unwrap();
        }

        let intent_path = tmp
            .path()
            .join(format!("rollback-{:016x}.intent", tx1.inner()));
        const INTENT_MAGIC: &[u8] = b"MFRLBK\0\0";
        let mut intent_bytes = Vec::with_capacity(16);
        intent_bytes.extend_from_slice(INTENT_MAGIC);
        intent_bytes.extend_from_slice(&tx1.inner().to_le_bytes());
        tokio::fs::write(&intent_path, &intent_bytes).await.unwrap();

        assert!(
            intent_path.exists(),
            "Rollback intent file must exist before startup recovery"
        );

        let storage = LsmStorage::new(config.clone())
            .await
            .expect("reopen storage after simulated rollback crash");

        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            Some(b"val1".to_vec()),
            "Data committed at target_tx must remain visible"
        );
        assert_eq!(
            storage.get(b"key2").await.unwrap(),
            None,
            "Data committed after target_tx (tx2) must be rolled back"
        );
        assert_eq!(
            storage.get(b"key3").await.unwrap(),
            None,
            "Data committed after target_tx (tx3) must be rolled back"
        );
        assert!(
            !intent_path.exists(),
            "Rollback intent file must be deleted after successful startup recovery"
        );
    }

    #[tokio::test]
    async fn test_wal_discovery_mixed_filenames() {
        let tmp = TempDir::new().expect("temp dir");

        let legacy_wal_path = tmp.path().join("wal.log");
        let wal = Wal::open_with_key_manager(&legacy_wal_path, None)
            .await
            .unwrap();
        let tx1 = TxId::new(1);
        let (entries1, _) = wal
            .prepare_batch(vec![(
                WalOp::Put {
                    tx_id: tx1,
                    key: b"k1".to_vec(),
                    value: b"v1".to_vec(),
                },
                1,
            )])
            .await
            .unwrap();
        wal.append_batch(entries1).await.unwrap();
        drop(wal);

        let wal5_path = tmp.path().join("wal-5.log");
        let wal5 = Wal::open_with_key_manager(&wal5_path, None).await.unwrap();
        let tx2 = TxId::new(2);
        let (entries2, _) = wal5
            .prepare_batch(vec![(
                WalOp::Put {
                    tx_id: tx2,
                    key: b"k2".to_vec(),
                    value: b"v2".to_vec(),
                },
                2,
            )])
            .await
            .unwrap();
        wal5.append_batch(entries2).await.unwrap();
        drop(wal5);

        let overflow_wal_path = tmp.path().join(format!("wal-{}.log", u128::MAX));
        let wal_overflow = Wal::open_with_key_manager(&overflow_wal_path, None)
            .await
            .unwrap();
        let tx3 = TxId::new(3);
        let (entries3, _) = wal_overflow
            .prepare_batch(vec![(
                WalOp::Put {
                    tx_id: tx3,
                    key: b"k3".to_vec(),
                    value: b"v3".to_vec(),
                },
                3,
            )])
            .await
            .unwrap();
        wal_overflow.append_batch(entries3).await.unwrap();
        drop(wal_overflow);

        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            ..Default::default()
        };
        let storage = LsmStorage::new(config)
            .await
            .expect("LsmStorage startup must succeed with mixed WAL files");

        assert_eq!(
            storage.flush_counter.load(Ordering::Relaxed),
            7,
            "flush_counter should be 7 (initialized to 6 + 1 for startup flush)"
        );
        assert_eq!(storage.get(b"k1").await.unwrap(), Some(b"v1".to_vec()));
        assert_eq!(storage.get(b"k2").await.unwrap(), Some(b"v2".to_vec()));
        assert_eq!(storage.get(b"k3").await.unwrap(), Some(b"v3".to_vec()));
    }

    #[tokio::test]
    async fn test_rollback_spanning_sstable_below_min_entries_threshold() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        let storage = LsmStorage::new(config).await.expect("create storage");

        let tx1 = TxId::new(1);
        storage.put(tx1, b"k1", b"v1").await.unwrap();
        storage.commit(tx1).await.unwrap();

        let tx2 = TxId::new(2);
        storage.put(tx2, b"k2", b"v2").await.unwrap();
        storage.commit(tx2).await.unwrap();

        storage.force_flush().await.unwrap();

        {
            let ssts = storage.sstables.read().await;
            assert_eq!(
                ssts.len(),
                1,
                "Should have 1 spanning SSTable before rollback"
            );
        }

        storage.rollback_to_tx(tx1).await.unwrap();

        {
            let ssts = storage.sstables.read().await;
            assert_eq!(
                ssts.len(),
                0,
                "No new SSTable should be created when surviving entries < 8"
            );
        }

        assert_eq!(storage.get(b"k1").await.unwrap(), Some(b"v1".to_vec()));
        assert_eq!(storage.get(b"k2").await.unwrap(), None);
    }
}
