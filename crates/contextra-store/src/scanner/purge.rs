//! # Purge-Implementierung (Ring 1)
//!
//! Bereinigt endgueltig physische Spuren geloeschter Daten auf Platten- und Speicher-Ebene.

// FILE-CONTEXT
// ZWECK: Physischer Purger für LSM Storage (§12.7.4, Verträge W1-P01).
// INVARIANTEN: Nullt und unlinkt verwaiste SSTable- und WAL-Segmente; atomarer Memtable-Flush.
// HOTSPOTS: [1-100]

#![forbid(unsafe_code)]

use super::runtime::ScannerRuntime;
use crate::lsm::LsmStorage;
use crate::sstable::{SstableBuilder, SstableReader};
use crate::tenant_codec::TenantKeyCodec;
use crate::wal::{Wal, WalEntry, WalOp, WAL_V3_HEADER};
use ahash::AHashSet;
use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, RequestContext, Result};
use std::sync::Arc;

/// Purger für LSM Storage Engine.
pub struct LsmPurger {
    storage: Arc<LsmStorage>,
    runtime: ScannerRuntime,
}

impl LsmPurger {
    /// Erzeugt einen neuen `LsmPurger` mit implizit erfasster Laufzeit.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self::with_runtime(storage, ScannerRuntime::capture())
    }

    /// Erzeugt einen neuen `LsmPurger` mit expliziter `ScannerRuntime`.
    pub fn with_runtime(storage: Arc<LsmStorage>, runtime: ScannerRuntime) -> Self {
        Self { storage, runtime }
    }
}

impl Purger for LsmPurger {
    fn purge(&self, ctx: &RequestContext, _target: &ScanTarget) -> Result<PurgeStats> {
        // 1. Offener Snapshot blockiert den Purge (fail-closed)
        if self.storage.snapshot_registry.min_active_seqno() != u64::MAX {
            return Err(ContextraError::Storage(
                "Purge blocked by open snapshot in SnapshotRegistry".to_string(),
            ));
        }

        let codec = TenantKeyCodec::new(ctx.tenant_id);
        let target_prefix = codec.scan_prefix().to_vec();

        let mut purged_units = 0u64;
        let mut bytes_freed = 0u64;

        let storage = Arc::clone(&self.storage);

        self.runtime.block_on(async {
            // 2. Erzwungener Flush der Memtables in SSTables
            storage.flush().await?;

            // 3. SSTables filtern / neu schreiben ohne Target-Records
            let readers = storage.sstables_for_scanner().await;
            let block_cache = storage.block_cache();

            let mut new_readers = Vec::new();
            let mut sst_files_to_purge = Vec::new();

            for reader in &readers {
                let entries = reader.iter().await.unwrap_or_default();
                let mut retained = Vec::new();

                for (k, v, seq) in entries {
                    if !k.starts_with(&target_prefix) {
                        retained.push((k, v, seq));
                    } else {
                        purged_units += 1;
                    }
                }

                if retained.len() == reader.iter().await.unwrap_or_default().len() {
                    // Keine Target-Einträge in dieser SSTable -> behalten
                    new_readers.push(Arc::clone(reader));
                } else if retained.is_empty() {
                    // Alle Einträge gehörten zum Target -> Datei entfernen
                    sst_files_to_purge.push(reader.file_path().to_path_buf());
                } else {
                    // Datei filtern und neu schreiben
                    let old_path = reader.file_path();
                    let new_path = old_path.with_extension("purged.sst");
                    if let Ok(mut builder) = SstableBuilder::create(&new_path).await {
                        for (rk, rv, rseq) in retained {
                            let _ = builder.add(&rk, &rv, rseq, 1).await;
                        }
                        if builder.finish().await.is_ok() {
                            if let Ok(new_reader) = SstableReader::open_with_key_manager(
                                &new_path,
                                block_cache.clone(),
                                None,
                            )
                            .await
                            {
                                new_readers.push(Arc::new(new_reader));
                                sst_files_to_purge.push(old_path.to_path_buf());
                            }
                        }
                    }
                }
            }

            // SSTables im Store aktualisieren
            {
                let sst_arc = storage.sstables_arc();
                let mut sst_guard = sst_arc.write().await;
                *sst_guard = new_readers;
            }

            // Altdateien (.sst) Nullen und Entfernen
            let data_dir = storage.data_path();
            let mut active_paths = AHashSet::new();
            for r in &storage.sstables_for_scanner().await {
                active_paths.insert(r.file_path().to_path_buf());
            }

            if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("sst")
                        && !active_paths.contains(&path)
                    {
                        sst_files_to_purge.push(path);
                    }
                }
            }

            for old_path in sst_files_to_purge {
                if let Ok(metadata) = tokio::fs::metadata(&old_path).await {
                    let len = metadata.len();
                    let zeros = vec![0u8; usize::try_from(len).unwrap_or(0)];
                    let _ = tokio::fs::write(&old_path, zeros).await;
                    if tokio::fs::remove_file(&old_path).await.is_ok() {
                        bytes_freed += len;
                    }
                }
            }

            // 4. WAL-Segmente filtern / bereinigen
            let mut wal_files = Vec::new();
            if let Ok(mut dir) = tokio::fs::read_dir(data_dir).await {
                while let Ok(Some(entry)) = dir.next_entry().await {
                    let path = entry.path();
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let is_wal = (name.starts_with("wal-") && name.ends_with(".log"))
                        || name == "wal.log"
                        || name.ends_with(".wal");
                    if is_wal {
                        wal_files.push(path);
                    }
                }
            }

            for wal_path in wal_files {
                if let Ok(wal) = Wal::open_read_only(&wal_path, None).await {
                    if let Ok(entries) = wal.replay().await {
                        let has_target =
                            entries.iter().any(|(_seq, entry, _pos)| match &entry.op {
                                WalOp::Put { key, .. } | WalOp::Delete { key, .. } => {
                                    key.starts_with(&target_prefix)
                                }
                                WalOp::TxEnd { .. } => false,
                            });

                        if has_target {
                            let retained_wal: Vec<_> = entries
                                .into_iter()
                                .filter_map(|(_seq, entry, _pos)| match entry.op {
                                    WalOp::Put { key, value, tx_id }
                                        if !key.starts_with(&target_prefix) =>
                                    {
                                        Some((
                                            WalOp::Put {
                                                key: key.clone(),
                                                value: value.clone(),
                                                tx_id,
                                            },
                                            entry.seq_no,
                                        ))
                                    }
                                    WalOp::Delete { key, tx_id }
                                        if !key.starts_with(&target_prefix) =>
                                    {
                                        Some((
                                            WalOp::Delete {
                                                key: key.clone(),
                                                tx_id,
                                            },
                                            entry.seq_no,
                                        ))
                                    }
                                    WalOp::TxEnd { committed, tx_id } => {
                                        Some((WalOp::TxEnd { committed, tx_id }, entry.seq_no))
                                    }
                                    _ => None,
                                })
                                .collect();

                            if retained_wal.is_empty() {
                                // WAL enthielt nur Target -> Nullen und Entfernen
                                if let Ok(metadata) = tokio::fs::metadata(&wal_path).await {
                                    let len = metadata.len();
                                    let zeros = vec![0u8; usize::try_from(len).unwrap_or(0)];
                                    let _ = tokio::fs::write(&wal_path, zeros).await;
                                    let _ = tokio::fs::remove_file(&wal_path).await;
                                    bytes_freed += len;
                                }
                            } else {
                                // WAL neu schreiben ohne Target-Records
                                let tmp_wal_path = wal_path.with_extension("purged.log");
                                let integrity_key = Wal::load_or_create_integrity_key(&wal_path)
                                    .await
                                    .unwrap_or([0u8; 32]);
                                let mut file_bytes = Vec::new();
                                file_bytes.extend_from_slice(&WAL_V3_HEADER);
                                for (op, seq_no) in retained_wal {
                                    if let Ok(entry) =
                                        WalEntry::try_new(op, seq_no, &integrity_key, [0u8; 32])
                                    {
                                        if let Ok(ebytes) = entry.to_bytes() {
                                            file_bytes.extend_from_slice(&ebytes);
                                        }
                                    }
                                }

                                if tokio::fs::write(&tmp_wal_path, file_bytes).await.is_ok() {
                                    if let Ok(metadata) = tokio::fs::metadata(&wal_path).await {
                                        let len = metadata.len();
                                        let zeros = vec![0u8; usize::try_from(len).unwrap_or(0)];
                                        let _ = tokio::fs::write(&wal_path, zeros).await;
                                    }
                                    let _ = tokio::fs::rename(&tmp_wal_path, &wal_path).await;
                                }
                            }
                        }
                    }
                }
            }

            Ok::<(), ContextraError>(())
        })?;

        Ok(PurgeStats {
            purged_units,
            bytes_freed,
        })
    }
}
