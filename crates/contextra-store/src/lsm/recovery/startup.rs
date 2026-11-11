//! # Recovery & Engine-Initialisierung (Ring 1)
//!
//! ZWECK: Wiederherstellung des LSM-Zustands beim Hochfahren:
//! 1. DirLock akquirieren & Salt laden/schreiben
//! 2. WAL-Dateien auflisten und chronologisch sortieren
//! 3. Manifest laden & gueltige/tote SSTables ermitteln
//! 4. WAL-Replay in MemTable und Verifikation der HMAC-Kette
//! 5. Bereinigung unvollstaendiger temporaerer Dateien (.tmp, SALT.tmp)
//! 6. SSTables oeffnen und mit BlockCache verknuepfen
//! 7. Hintergrund-Tasks starten (Compaction, Pressure-Monitor)
//! 8. Anhaengige Rollback-Intents (`rollback-*.intent`) ausfuehren

use crate::compaction::{CompactionEngine, MergeOperator};
use crate::error::{StoreError as ContextraError, StoreResult as Result};
use crate::io_seam::{RealFs, Vfs};
use crate::lsm::config::LsmConfig;
use crate::lsm::engine::{LsmStorage, StorageHealth};
use crate::lsm::guard::LsmState;
use crate::lsm::recovery::salt::{directory_is_pristine, generate_crypto_salt_with_source, write_salt_atomically};
use crate::memtable::MemTable;
use crate::sstable::{create_block_cache_with_shards, SstableReader};
use crate::util::DirLock;
use crate::wal::{KeyManager, Wal, WalOp};
use bytes::Bytes;
use contextra_ids::{TxId, TOMBSTONE_BIT};
use contextra_mvcc::{SnapshotRegistry, TxBuffer};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

struct PendingTxOp {
    lsn: u64,
    op: WalOp,
}

impl LsmStorage {
    /// Oeffnet oder erzeugt den LSM-Speicher mit der uebergebenen Konfiguration.
    pub async fn new(config: LsmConfig) -> Result<Self> {
        Self::new_with_merge_operator(config, None).await
    }

    /// Oeffnet oder erzeugt den LSM-Speicher mit optionalem MergeOperator fuer Compaction.
    pub async fn new_with_merge_operator(
        config: LsmConfig,
        merge_operator: Option<Arc<dyn MergeOperator>>,
    ) -> Result<Self> {
        if config.block_cache_shards == 0 || !config.block_cache_shards.is_power_of_two() {
            return Err(ContextraError::InvalidInput(format!(
                "block_cache_shards must be > 0 and a power of two, got {}",
                config.block_cache_shards
            )));
        }

        let vfs: Arc<dyn Vfs> = config
            .vfs
            .clone()
            .map(|s| s.vfs().clone())
            .unwrap_or_else(|| Arc::new(RealFs));

        vfs.create_dir_all(&config.path)?;
        if let Some(parent) = config.path.parent() {
            vfs.sync_dir(parent)?;
        }

        let dir_lock = DirLock::acquire(&config.path, config.durability_mode)?;

        let salt_path = config.path.join("SALT");
        let salt = match vfs.read(&salt_path) {
            Ok(buf) => {
                if buf.len() != 32 {
                    return Err(ContextraError::Storage(format!(
                        "Invalid SALT length: expected 32, got {}",
                        buf.len()
                    )));
                }
                buf
            }
            Err(crate::error::StoreError::NotFound(_)) => {
                if !directory_is_pristine(&config.path).await? {
                    return Err(ContextraError::Storage(
                        "SALT file missing in non-pristine database directory".into(),
                    ));
                }
                let buf = generate_crypto_salt_with_source(config.key_source.as_ref())?;
                write_salt_atomically(&salt_path, &buf).await?;
                buf.to_vec()
            }
            Err(e) => {
                return Err(ContextraError::Storage(format!("SALT nicht lesbar: {e}")));
            }
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
            .map_err(|e| ContextraError::Storage(format!("Failed to read data dir: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read directory entry: {e}")))?
        {
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

        let manifest_path = config.path.join("MANIFEST");
        let manifest_exists = manifest_path.exists();
        #[allow(clippy::type_complexity)]
        let (valid_manifest_sstables, dead_manifest_sstables, manifest_hwm): (
            Option<std::collections::BTreeSet<std::path::PathBuf>>,
            Option<std::collections::HashSet<std::path::PathBuf>>,
            Option<[u8; 32]>,
        ) = if manifest_exists {
            let entries = crate::manifest::Manifest::load(&manifest_path).await?;
            let valid_set = crate::manifest::Manifest::reconstruct_valid_sstables(&entries)
                .into_iter()
                .map(|(path, _rank)| path)
                .collect();
            let dead_set = crate::manifest::Manifest::reconstruct_dead_sstables(&entries);
            let hwm = crate::manifest::Manifest::extract_high_water_mark(&entries);
            (Some(valid_set), Some(dead_set), hwm)
        } else {
            (None, None, None)
        };

        let memtable = MemTable::new();
        let mut max_seq = 0u64;
        let mut max_tx = 0u64;
        let mut replayed_size = 0u64;
        let mut pending_tx_map: std::collections::BTreeMap<u64, Vec<PendingTxOp>> =
            std::collections::BTreeMap::new();

        let mut last_replayed_hmac: Option<[u8; 32]> = None;
        let mut replayed_hmac_set = std::collections::BTreeSet::new();
        replayed_hmac_set.insert([0u8; 32]);

        for (_ts, wal_path) in &wal_files {
            let wal = Wal::open_read_only(wal_path, key_manager.clone()).await?;
            let wal_entries = wal.replay().await?;

            if let Some((_, last_entry, _)) = wal_entries.last() {
                last_replayed_hmac = Some(last_entry.checksum);
            }

            for (lsn, entry, _offset) in &wal_entries {
                replayed_hmac_set.insert(entry.checksum);
                let raw_lsn = *lsn & !TOMBSTONE_BIT;
                if raw_lsn > max_seq {
                    max_seq = raw_lsn;
                }

                match &entry.op {
                    WalOp::Put { .. } | WalOp::Delete { .. } => {
                        let tx_id = entry.tx_id().inner();
                        if tx_id > max_tx && tx_id < TxId::INTERNAL_BASE {
                            max_tx = tx_id;
                        }
                        pending_tx_map.entry(tx_id).or_default().push(PendingTxOp {
                            lsn: *lsn,
                            op: entry.op.clone(),
                        });
                    }
                    WalOp::TxEnd { tx_id, committed } => {
                        let tx_raw = tx_id.inner();
                        if tx_raw > max_tx && tx_raw < TxId::INTERNAL_BASE {
                            max_tx = tx_raw;
                        }
                        if *committed {
                            if let Some(ops) = pending_tx_map.remove(&tx_raw) {
                                for op in ops {
                                    match op.op {
                                        WalOp::Put { key, value, tx_id } => {
                                            replayed_size += (key.len() + value.len()) as u64;
                                            memtable.put(
                                                Bytes::from(key),
                                                Bytes::from(value),
                                                op.lsn,
                                                tx_id.inner(),
                                            );
                                        }
                                        WalOp::Delete { key, tx_id } => {
                                            replayed_size += key.len() as u64;
                                            memtable.put(
                                                Bytes::from(key),
                                                Bytes::new(),
                                                op.lsn | TOMBSTONE_BIT,
                                                tx_id.inner(),
                                            );
                                        }
                                        WalOp::TxEnd { .. } => {}
                                    }
                                }
                            }
                        } else {
                            pending_tx_map.remove(&tx_raw);
                        }
                    }
                }
            }

            let orphan_ops: usize = pending_tx_map.values().map(|v| v.len()).sum();
            if orphan_ops > 0 {
                tracing::warn!(
                    orphan_ops = orphan_ops,
                    wal_path = ?wal_path,
                    "Discarded uncommitted orphan transaction operations from WAL segment"
                );
                pending_tx_map.clear();
            }

            drop(wal);
        }

        if let Some(expected_hwm) = manifest_hwm {
            let actual_tail_hmac = last_replayed_hmac.unwrap_or([0u8; 32]);
            let check_hmac = if replayed_hmac_set.contains(&expected_hwm)
                || valid_manifest_sstables
                    .as_ref()
                    .is_some_and(|s| !s.is_empty())
            {
                expected_hwm
            } else {
                actual_tail_hmac
            };
            if contextra_crypto::verify_wal_chain_completeness(&check_hmac, &expected_hwm).is_err()
            {
                return Err(ContextraError::wal_truncation_detected(
                    expected_hwm,
                    actual_tail_hmac,
                ));
            }
        }

        let active_wal_path = if let Some((_, last_path)) = wal_files.last() {
            last_path.clone()
        } else {
            config.path.join("wal.log")
        };
        let wal = Wal::open_with_key_manager(&active_wal_path, key_manager.clone()).await?;

        let budget_config = crate::budget::ResourceBudget {
            memory_limit: (config.max_ram_mb * 1024 * 1024) as usize,
        };
        let resource_tracker = Arc::new(crate::budget::ResourceTracker::new(budget_config));
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
        let mut entries = tokio::fs::read_dir(&config.path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read data dir: {e}")))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read directory entry: {e}")))?
        {
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
        pending_rollbacks.sort_unstable();

        let mut sst_files = Vec::new();
        let mut entries = tokio::fs::read_dir(&config.path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read data dir: {e}")))?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read directory entry: {e}")))?
        {
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
                if let Some(ref valid_set) = valid_manifest_sstables {
                    let path_key = std::path::Path::new(file_name);
                    if valid_set.contains(path_key) {
                        sst_files.push(path);
                    } else if dead_manifest_sstables
                        .as_ref()
                        .is_some_and(|dead_set| dead_set.contains(path_key))
                    {
                        tracing::info!("Removing dead SSTable file proven by MANIFEST: {:?}", path);
                        if let Err(e) = tokio::fs::remove_file(&path).await {
                            tracing::warn!("Failed to remove dead SSTable {:?}: {}", path, e);
                        }
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
        sst_files.sort();

        let block_cache = create_block_cache_with_shards(64, config.block_cache_shards);

        let mut sstables = Vec::new();
        for path in sst_files {
            let reader = SstableReader::open_with_vfs(
                path,
                Arc::clone(&block_cache),
                key_manager.clone(),
                Arc::clone(&vfs),
            )
            .await?;

            let raw_sst_max_seq = reader.metadata().max_seq & !TOMBSTONE_BIT;
            if raw_sst_max_seq > max_seq {
                max_seq = raw_sst_max_seq;
            }
            if reader.metadata().max_tx_id > max_tx
                && reader.metadata().max_tx_id < TxId::INTERNAL_BASE
            {
                max_tx = reader.metadata().max_tx_id;
            }

            sstables.push(Arc::new(reader));
        }

        sstables.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);
        let sstables = Arc::new(RwLock::new(sstables));

        let manifest = Arc::new(
            crate::manifest::Manifest::open_with_dir_lock(&manifest_path, Some(dir_lock)).await?,
        );
        if !manifest_exists {
            let ssts_read = sstables.read().await;
            let mut add_entries = Vec::with_capacity(ssts_read.len() + 1);
            for sst in ssts_read.iter() {
                add_entries.push(crate::manifest::ManifestEntry::Add {
                    path: sst.file_path().to_path_buf(),
                    max_tx: sst.metadata().max_tx_id,
                });
            }
            let initial_wal_hmac = *wal.last_hmac.lock().await;
            add_entries.push(crate::manifest::ManifestEntry::WalCheckpoint {
                hmac: initial_wal_hmac,
            });
            manifest.append_batch(&add_entries).await?;
        }

        let snapshot_registry = Arc::new(SnapshotRegistry::new());

        let cancel_token = tokio_util::sync::CancellationToken::new();
        let task_tracker = tokio_util::task::TaskTracker::new();

        let wal_queue_depth = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let wal_queue_depth_clone = Arc::clone(&wal_queue_depth);

        let pressure_monitor = crate::system_pressure::SystemPressureMonitor::new_with_clock(
            std::time::Duration::from_millis(100),
            Arc::clone(&config.clock),
        );
        let pressure_rx = pressure_monitor.pressure_rx.clone();
        let ct_pressure = cancel_token.clone();
        let metrics_sink_container = Arc::new(parking_lot::RwLock::new(Arc::new(
            contextra_env::NoopMetricsSink,
        )
            as Arc<dyn contextra_env::MetricsSink>));
        let metrics_sink_for_pressure = Arc::clone(&metrics_sink_container);

        task_tracker.spawn(async move {
            pressure_monitor
                .run_with_metrics_sink(
                    ct_pressure,
                    move || wal_queue_depth_clone.load(Ordering::Relaxed),
                    || 0,
                    0,
                    Some(metrics_sink_for_pressure),
                )
                .await;
        });

        let mut compaction_engine_builder = CompactionEngine::new(
            config.compaction.clone(),
            Arc::clone(&snapshot_registry),
            Arc::clone(&block_cache),
            key_manager.clone(),
            Arc::clone(&resource_tracker),
            Some(Arc::clone(&manifest)),
        )
        .with_pressure_rx(pressure_rx.clone())
        .with_vfs(Arc::clone(&vfs));

        if let Some(mo) = merge_operator {
            compaction_engine_builder = compaction_engine_builder.with_merge_operator(mo);
        }

        let compaction_engine = Arc::new(compaction_engine_builder);

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

        let flush_notify = Arc::new(tokio::sync::Notify::new());
        let health = Arc::new(parking_lot::RwLock::new(StorageHealth::Healthy));

        let ssi_max_tracked_keys = config.ssi_max_tracked_keys;

        let default_ssi_val = Arc::new(
            contextra_mvcc::SequenceLogSsiValidator::new_with_bounds(ssi_max_tracked_keys)
                .with_metrics_sink(Arc::clone(&metrics_sink_container)),
        );

        let clock_arc = Arc::clone(&config.clock);
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
            last_applied_seq: AtomicU64::new(max_seq),
            flush_notify,
            health,
            commit_mutex: tokio::sync::Mutex::new(()),
            flush_mutex: tokio::sync::Mutex::new(()),
            flush_purge_fence: crate::lsm::fence::FlushPurgeFence::new(),
            pending_sealed_wals: tokio::sync::Mutex::new(Vec::new()),
            cancel_token,
            task_tracker,
            flush_counter: AtomicU64::new(max_wal_id.map(|m| m.saturating_add(1)).unwrap_or(0)),
            segment_counter: AtomicU64::new(0),
            budget_tracking_drift_bytes: std::sync::atomic::AtomicU64::new(0),
            pending_commit_queue: tokio::sync::Mutex::new(None),
            wal_queue_depth,
            pressure_rx,
            intent_locks: std::sync::Mutex::new(std::collections::HashMap::new()),
            observer_registry: crate::lsm::observer::ObserverRegistry::new(),
            clock: parking_lot::RwLock::new(clock_arc),
            ssi_validator: Arc::clone(&default_ssi_val)
                as Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>,
            sequence_log_ssi_validator: Some(default_ssi_val),
            metrics_sink: metrics_sink_container,
        };

        if replayed_size > 0 && !wal_files.is_empty() {
            tracing::info!(
                replayed_bytes = replayed_size,
                "Forcing startup flush to persist replayed WAL entries before old WAL cleanup"
            );
            storage.flush().await.map_err(|e| {
                ContextraError::Storage(format!("Startup flush after WAL replay failed: {e}"))
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
                    ContextraError::Storage(format!(
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
}
