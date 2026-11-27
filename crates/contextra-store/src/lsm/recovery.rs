//! # Crash-Recovery des LSM-Trees (Ring 1)
//!
//! Stellt den letzten konsistenten Zustand durch Manifest- und WAL-Replay wieder her.

use super::*;
use crate::compaction::{CompactionEngine, MergeOperator};
use crate::io_seam::RealFs;
use crate::memtable::MemTable;
use crate::sstable::{create_block_cache_with_shards, SstableReader};
use crate::util::DirLock;
use crate::wal::KeyManager;
use crate::wal::{Wal, WalOp};
use contextra_mvcc::{SnapshotRegistry, TxBuffer};
use contextra_ports::gate::ReadGate;
use contextra_ports::Vfs;
use contextra_ports::{ResourceBudget, ResourceTracker};
use contextra_types::{ContextraError, Result, TxId, TOMBSTONE_BIT};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Process-wide counter for deterministic temporary file naming.
/// `Ordering::Relaxed` is sufficient because process ID + atomic sequence guarantees
/// intra-process collision freedom, while process ID guarantees inter-process collision freedom.
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_temp_file_counter() -> u64 {
    TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Helper to generate a 32-byte cryptographic salt using the provided [`KeyMaterialSource`].
pub(crate) fn generate_crypto_salt_with_source(
    key_source: &dyn crate::key_source::KeyMaterialSource,
) -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    key_source.fill_key_bytes(&mut buf)?;
    Ok(buf)
}

/// Helper to generate a 32-byte cryptographic salt with default OS-CSPRNG.
pub(crate) fn generate_crypto_salt() -> Result<[u8; 32]> {
    let source = crate::key_source::OsKeyMaterialSource;
    generate_crypto_salt_with_source(&source)
}

pub(crate) async fn write_salt_atomically(
    salt_path: &std::path::Path,
    buf: &[u8; 32],
) -> Result<()> {
    let io = crate::io_seam::StoreIo::default();
    write_salt_atomically_with_io(&io, salt_path, buf).await
}

pub(crate) async fn write_salt_atomically_with_io(
    store_io: &crate::io_seam::StoreIo,
    salt_path: &std::path::Path,
    buf: &[u8; 32],
) -> Result<()> {
    store_io
        .write_atomic(salt_path, buf.to_vec())
        .await
        .map_err(|e| ContextraError::Storage(format!("Failed to write SALT file: {e}")))?;
    if let Some(parent) = salt_path.parent() {
        store_io.sync_dir(parent).await?;
    }
    Ok(())
}

pub(super) async fn directory_is_pristine(
    store_io: &crate::io_seam::StoreIo,
    path: &std::path::Path,
) -> Result<bool> {
    let entries = store_io.read_dir(path).await.map_err(|e| {
        ContextraError::Storage(format!("Failed to read data dir for pristine check: {e}"))
    })?;

    for entry_path in entries {
        let name_str = entry_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if name_str.starts_with("wal-")
            || name_str == "wal.log"
            || name_str.ends_with(".sst")
            || name_str == "MANIFEST"
        {
            return Ok(false);
        }
    }

    Ok(true)
}

struct PendingTxOp {
    lsn: u64,
    op: WalOp,
}

impl LsmStorage {
    pub async fn new(config: LsmConfig) -> Result<Self> {
        Self::new_with_merge_operator(config, None).await
    }

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

        let store_io = config
            .vfs
            .clone()
            .unwrap_or_else(|| crate::io_seam::StoreIo::new(Arc::clone(&vfs)));

        let dir_lock = match DirLock::acquire(&config.path, config.durability_mode) {
            Ok(lock) => lock,
            Err(ContextraError::Storage(ref msg))
                if msg == "Datenverzeichnis bereits in Benutzung" =>
            {
                let mut acquired = None;
                for _ in 0..10 {
                    tokio::task::yield_now().await;
                    if let Ok(lock) = DirLock::acquire(&config.path, config.durability_mode) {
                        acquired = Some(lock);
                        break;
                    }
                }
                match acquired {
                    Some(lock) => lock,
                    None => DirLock::acquire(&config.path, config.durability_mode)?,
                }
            }
            Err(e) => return Err(e),
        };

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
            Err(ContextraError::NotFound(_)) => {
                if !directory_is_pristine(&store_io, &config.path).await? {
                    return Err(ContextraError::Storage(
                        "SALT file missing in non-pristine database directory".into(),
                    ));
                }
                let buf = generate_crypto_salt()?;
                write_salt_atomically_with_io(&store_io, &salt_path, &buf).await?;
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
        let dir_entries = store_io
            .read_dir(&config.path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read data dir: {e}")))?;

        for entry_path in dir_entries {
            let name_str = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if name_str.starts_with("wal-") && name_str.ends_with(".log") {
                if let Ok(seq_component) = name_str[4..name_str.len() - 4].parse::<u128>() {
                    wal_files.push((seq_component, entry_path.clone()));
                    match u64::try_from(seq_component) {
                        Ok(id) => {
                            max_wal_id = Some(max_wal_id.unwrap_or(id).max(id));
                        }
                        Err(_) => {
                            tracing::warn!(
                                path = ?entry_path,
                                seq_component = seq_component,
                                "WAL filename sequence component overflows u64; ignoring from max_wal_id calculation"
                            );
                        }
                    }
                }
            } else if name_str == "wal.log" {
                wal_files.push((0, entry_path));
                max_wal_id = Some(max_wal_id.unwrap_or(0));
            }
        }

        wal_files.sort_by(|(ts_a, path_a), (ts_b, path_b)| {
            ts_a.cmp(ts_b).then_with(|| path_a.cmp(path_b))
        });

        let manifest_path = config.path.join("MANIFEST");
        let manifest_exists = store_io.exists(&manifest_path).await?;
        #[allow(clippy::type_complexity)]
        let (valid_manifest_sstables, dead_manifest_sstables, manifest_hwm): (
            Option<std::collections::HashSet<std::path::PathBuf>>,
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

        let health = Arc::new(parking_lot::RwLock::new(StorageHealth::Healthy));
        let clock_arc: Arc<dyn contextra_ports::Clock> =
            Arc::new(contextra_ports::SystemClock::new());
        let clock_lock = Arc::new(parking_lot::RwLock::new(Arc::clone(&clock_arc)));
        let erasure_cfg = crate::erasure::StoreErasureConfig::for_data_dir(&config.path);
        let key_source = crate::key_source::OsKeyMaterialSource;
        let (erasure_kernel, erasure_boot_report) =
            crate::erasure::StoreErasureKernel::open_early(
                erasure_cfg,
                config.path.clone(),
                store_io.clone(),
                Arc::clone(&health),
                Arc::clone(&clock_lock),
                &key_source,
                None,
            )
            .await
            .map_err(|e| ContextraError::Storage(format!("Erasure kernel open failed: {e}")))?;
        let erasure = Arc::new(erasure_kernel);

        let memtable = MemTable::new();
        let mut max_seq = 0u64;
        let mut max_tx = 0u64;
        let mut replayed_size = 0u64;
        let mut pending_tx_map: std::collections::HashMap<u64, Vec<PendingTxOp>> =
            std::collections::HashMap::new();

        let mut last_replayed_hmac: Option<[u8; 32]> = None;
        let mut replayed_hmac_set = std::collections::HashSet::new();
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
                            let gate = erasure.gate_ref();
                            let gate_active = gate.is_active();
                            if let Some(ops) = pending_tx_map.remove(&tx_raw) {
                                for op in ops {
                                    match op.op {
                                        WalOp::Put { key, value, tx_id } => {
                                            if gate_active
                                                && gate.discard_rule(
                                                    &key,
                                                    contextra_types::Seq(op.lsn & !TOMBSTONE_BIT),
                                                ) == contextra_ports::gate::DiscardRule::MayDrop
                                            {
                                                continue;
                                            }
                                            replayed_size += (key.len() + value.len()) as u64;
                                            memtable.put(
                                                Bytes::from(key),
                                                Bytes::from(value),
                                                op.lsn,
                                                tx_id.inner(),
                                            );
                                        }
                                        WalOp::Delete { key, tx_id } => {
                                            if gate_active
                                                && gate.discard_rule(
                                                    &key,
                                                    contextra_types::Seq(op.lsn & !TOMBSTONE_BIT),
                                                ) == contextra_ports::gate::DiscardRule::MayDrop
                                            {
                                                continue;
                                            }
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
        let mut sst_files = Vec::new();
        let dir_entries = store_io
            .read_dir(&config.path)
            .await
            .map_err(|e| ContextraError::Storage(format!("Failed to read data dir: {e}")))?;

        for path in dir_entries {
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
            } else if file_name.ends_with(".tmp")
                || path.extension().is_some_and(|ext| ext == "tmp")
                || file_name.starts_with("SALT.tmp.")
                || file_name.starts_with("MANIFEST.new.")
            {
                tracing::warn!("Removing leftover un-renamed temp file: {:?}", path);
                if let Err(e) = store_io.remove_file(&path).await {
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
                        if let Err(e) = store_io.remove_file(&path).await {
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
        pending_rollbacks.sort_unstable();
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
            Arc::clone(&clock_arc),
        );
        let pressure_rx = pressure_monitor.pressure_rx.clone();
        let ct_pressure = cancel_token.clone();
        let metrics_sink_container = Arc::new(parking_lot::RwLock::new(Arc::new(
            contextra_ports::NoopMetricsSink,
        )
            as Arc<dyn contextra_ports::MetricsSink>));
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
        .with_vfs(Arc::clone(&vfs))
        .with_erase_filter(
            Arc::clone(erasure.gate_ref()),
            erasure.global_collapse_handle(),
        )
        .with_pending_delete(Arc::clone(erasure.pending_ref()));

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

        let ssi_max_tracked_keys = config.ssi_max_tracked_keys;

        let default_ssi_val = Arc::new(
            contextra_mvcc::SequenceLogSsiValidator::new_with_bounds(ssi_max_tracked_keys)
                .with_metrics_sink(Arc::clone(&metrics_sink_container)),
        );

        let state = Arc::new(RwLock::new(LsmState {
            memtable: Arc::new(memtable),
            immutable_memtables: Vec::new(),
        }));
        let wal_lock = Arc::new(RwLock::new(Arc::new(wal)));
        let next_seq_no = Arc::new(AtomicU64::new(max_seq.saturating_add(1)));
        let last_committed_tx = Arc::new(AtomicU64::new(max_tx));
        let last_applied_seq = Arc::new(AtomicU64::new(max_seq));
        let commit_mutex = Arc::new(tokio::sync::Mutex::new(()));
        let flush_mutex = Arc::new(tokio::sync::Mutex::new(()));
        let pending_sealed_wals = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let flush_counter = Arc::new(AtomicU64::new(max_wal_id.map(|m| m.saturating_add(1)).unwrap_or(0)));
        let segment_counter = Arc::new(AtomicU64::new(0));

        erasure.attach_deps(crate::erasure::purge_lsm::StoreDeps {
            data_dir: config.path.clone(),
            io: store_io.clone(),
            key_manager: key_manager.clone(),
            state: Arc::clone(&state),
            sstables: Arc::clone(&sstables),
            wal: Arc::clone(&wal_lock),
            snapshot_registry: Arc::clone(&snapshot_registry),
            manifest: Arc::clone(&manifest),
            block_cache: Arc::clone(&block_cache),
            budget: Arc::clone(&resource_tracker),
            next_seq_no: Arc::clone(&next_seq_no),
            last_committed_tx: Arc::clone(&last_committed_tx),
            last_applied_seq: Arc::clone(&last_applied_seq),
            commit_mutex: Arc::clone(&commit_mutex),
            flush_mutex: Arc::clone(&flush_mutex),
            pending_sealed_wals: Arc::clone(&pending_sealed_wals),
            flush_counter: Arc::clone(&flush_counter),
            segment_counter: Arc::clone(&segment_counter),
        });

        let storage = Self {
            config,
            key_manager,
            state,
            sstables,
            tx_buffer,
            budget: resource_tracker,
            block_cache,
            wal: wal_lock,
            snapshot_registry,
            compaction_engine,
            manifest,
            next_seq_no,
            last_committed_tx,
            last_applied_seq,
            flush_notify,
            health,
            commit_mutex,
            flush_mutex,
            flush_purge_fence: super::fence::FlushPurgeFence::new(),
            pending_sealed_wals,
            cancel_token,
            task_tracker,
            flush_counter,
            segment_counter,
            budget_tracking_drift_bytes: std::sync::atomic::AtomicU64::new(0),
            pending_commit_queue: tokio::sync::Mutex::new(None),
            wal_queue_depth,
            pressure_rx,
            intent_locks: std::sync::Mutex::new(std::collections::HashMap::new()),
            observer_registry: super::observer::ObserverRegistry::new(),
            clock: clock_lock,
            ssi_validator: Arc::clone(&default_ssi_val)
                as Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>,
            sequence_log_ssi_validator: Some(default_ssi_val),
            metrics_sink: metrics_sink_container,
            erasure,
            erasure_boot_report,
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
                    let uuid_sidecar =
                        PathBuf::from(format!("{}.uuid", old_wal_path.display()));
                    if let Err(e) = storage
                        .erasure
                        .pending_ref()
                        .remove_after_commit(&[old_wal_path.clone(), uuid_sidecar])
                        .await
                    {
                        tracing::warn!("Failed to remove old WAL file {:?}: {}", old_wal_path, e);
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

        let boot_seq = storage
            .last_applied_seq
            .load(std::sync::atomic::Ordering::Acquire);
        storage
            .erasure
            .finalize_boot_gates(boot_seq)
            .await
            .map_err(|e| {
                ContextraError::Storage(format!("Failed to finalize erasure boot gates: {e}"))
            })?;

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
            let store_io = self
                .config
                .vfs
                .clone()
                .unwrap_or_else(crate::io_seam::StoreIo::default);
            store_io
                .write_atomic(&intent_path, intent_bytes)
                .await
                .map_err(|e| {
                    ContextraError::Storage(format!("Failed to write rollback intent file: {e}"))
                })?;
            store_io.sync_dir(&self.config.path).await?;
        }

        let state = self.state.write().await;
        let wal = self.wal.read().await.clone();

        let (target_offset, target_hmac) = wal.find_tx_offset(target_tx).await?;
        wal.truncate(target_offset, target_hmac).await?;

        fn prune_memtable_above_tx(memtable: &crate::memtable::MemTable, target_tx: u64) {
            let txs_to_rollback: std::collections::HashSet<u64> = memtable
                .iter()
                .into_iter()
                .map(|(_, _, _, tx)| tx)
                .filter(|&tx| tx > target_tx)
                .collect();
            for tx_id in txs_to_rollback {
                memtable.rollback(tx_id);
            }
        }

        prune_memtable_above_tx(&state.memtable, target_tx.inner());
        for imm in &state.immutable_memtables {
            prune_memtable_above_tx(imm, target_tx.inner());
        }

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

            const MIN_ENTRIES_FOR_SSTABLE_REBUILD: usize = 8;
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
                    state
                        .memtable
                        .put(bytes::Bytes::from(k), bytes::Bytes::from(v), seq, tx);
                }
            }

            sst_to_remove.push(spanning.file_path().to_path_buf());
        }

        sstables_lock.sort_by_key(|sst| sst.metadata().max_seq & !TOMBSTONE_BIT);

        let mut max_seq = 0u64;
        for sst in sstables_lock.iter() {
            max_seq = max_seq.max(sst.metadata().max_seq & !TOMBSTONE_BIT);
        }
        for (_, _, seq, _) in state.memtable.iter() {
            max_seq = max_seq.max(seq & !TOMBSTONE_BIT);
        }
        for imm in &state.immutable_memtables {
            for (_, _, seq, _) in imm.iter() {
                max_seq = max_seq.max(seq & !TOMBSTONE_BIT);
            }
        }
        drop(sstables_lock);

        let store_io = self
            .config
            .vfs
            .clone()
            .unwrap_or_else(crate::io_seam::StoreIo::default);
        let mut manifest_batch = Vec::new();
        for path in sst_to_remove {
            tracing::info!("Removing SSTable during rollback: {:?}", path);
            manifest_batch.push(crate::manifest::ManifestEntry::Remove { path: path.clone() });
            if let Err(e) = store_io.remove_file(&path).await {
                if !matches!(e, ContextraError::NotFound(_)) {
                    tracing::error!(
                        path = ?path,
                        "Orphaned SSTable konnte nicht entfernt werden: {e}. Manuelles Cleanup nötig."
                    );
                }
            }
        }

        manifest_batch.push(crate::manifest::ManifestEntry::RollbackComplete {
            target_tx: target_tx.inner(),
        });

        if let Err(e) = self.manifest.append_batch(&manifest_batch).await {
            tracing::warn!("Failed to write Manifest batch during rollback: {}", e);
        }

        if let Err(e) = store_io.remove_file(&intent_path).await {
            if !matches!(e, ContextraError::NotFound(_)) {
                tracing::warn!(
                    "Could not remove rollback intent file {:?}: {} \
                     (non-fatal — recovery will re-run on next startup)",
                    intent_path,
                    e
                );
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
