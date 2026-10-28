// FILE-CONTEXT
// ZWECK: Physical Purger implementation für LSM-Storage-Schichten (§12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: Physikalische Entfernung durch Flush, Range Compaction, File Zeroing und Unlink.
// HOTSPOTS: [1-120]

#![forbid(unsafe_code)]

use super::sstable::block_on_future;
use crate::lsm::LsmStorage;
use crate::sstable::{SstableBuilder, SstableReader};
use crate::tenant_codec::TenantKeyCodec;
use crate::wal::{Wal, WalConfig, WalOp};
use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, RequestContext, Result};
use std::collections::HashSet;
use std::sync::Arc;

/// Purger für LSM Storage Engine.
pub struct LsmPurger {
    storage: Arc<LsmStorage>,
}

impl LsmPurger {
    /// Erzeugt einen neuen `LsmPurger`.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self { storage }
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
        let target_prefix = codec.scan_prefix();

        let mut purged_units = 0u64;
        let mut bytes_freed = 0u64;

        block_on_future(async {
            // 2. Erzwungener Flush der Memtables in SSTables
            let _ = self.storage.flush().await;

            // 3. SSTables filtern / neu schreiben ohne Target-Records
            let readers = self.storage.sstables_for_scanner().await;
            let block_cache = self.storage.block_cache();

            let mut new_readers = Vec::new();
            let mut sst_files_to_purge = Vec::new();

            for reader in &readers {
                let entries = reader.iter().await.unwrap_or_default();
                let mut retained = Vec::new();

                for (k, v, seq) in entries {
                    if !k.starts_with(target_prefix) {
                        retained.push((k, v, seq));
                    } else {
                        purged_units += 1;
                    }
                }

                if retained.len() == reader.iter().await.unwrap_or_default().len() {
                    // Keine Target-Einträge in dieser SSTable -> behalten
                    new_readers.push(Arc::clone(reader));
                } else if retained.is_empty() {
                    // Alle Einträge gehörten zum Target -> Datei löschen
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
                            if let Ok(new_reader) =
                                SstableReader::open_with_key_manager(&new_path, block_cache.clone(), None).await
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
                let sst_arc = self.storage.sstables_arc();
                let mut sst_guard = sst_arc.write().await;
                *sst_guard = new_readers;
            }

            // Altdateien (.sst) Nullen und Löschen
            let data_dir = self.storage.data_path();
            let mut active_paths = HashSet::new();
            for r in &self.storage.sstables_for_scanner().await {
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
                    let zeros = vec![0u8; len as usize];
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
                        let has_target = entries.iter().any(|(_seq, entry, _pos)| match &entry.op {
                            WalOp::Put { key, .. } | WalOp::Delete { key, .. } => key.starts_with(target_prefix),
                            WalOp::TxEnd { .. } => false,
                        });

                        if has_target {
                            let retained_wal: Vec<_> = entries
                                .into_iter()
                                .filter_map(|(_seq, entry, _pos)| match entry.op {
                                    WalOp::Put { key, value, tx_id } if !key.starts_with(target_prefix) => {
                                        Some((WalOp::Put { key: key.clone(), value: value.clone(), tx_id }, entry.seq_no))
                                    }
                                    WalOp::Delete { key, tx_id } if !key.starts_with(target_prefix) => {
                                        Some((WalOp::Delete { key: key.clone(), tx_id }, entry.seq_no))
                                    }
                                    WalOp::TxEnd { committed, tx_id } => {
                                        Some((WalOp::TxEnd { committed, tx_id }, entry.seq_no))
                                    }
                                    _ => None,
                                })
                                .collect();

                            if retained_wal.is_empty() {
                                // WAL enthielt nur Target -> Nullen und Löschen
                                if let Ok(metadata) = tokio::fs::metadata(&wal_path).await {
                                    let len = metadata.len();
                                    let zeros = vec![0u8; len as usize];
                                    let _ = tokio::fs::write(&wal_path, zeros).await;
                                    let _ = tokio::fs::remove_file(&wal_path).await;
                                    bytes_freed += len;
                                }
                            } else {
                                // WAL neu schreiben ohne Target-Records
                                let tmp_wal_path = wal_path.with_extension("purged.log");
                                if let Ok(new_wal) = Wal::open_with_config(
                                    &tmp_wal_path,
                                    WalConfig::default(),
                                )
                                .await
                                {
                                    if let Ok((batch, _)) = new_wal.prepare_batch(retained_wal).await {
                                        let _ = new_wal.append_batch(batch).await;
                                    }

                                    if let Ok(metadata) = tokio::fs::metadata(&wal_path).await {
                                        let len = metadata.len();
                                        let zeros = vec![0u8; len as usize];
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
