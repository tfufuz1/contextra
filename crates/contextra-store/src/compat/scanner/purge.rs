//! # Purge-Implementierung (Ring 1)
//!
//! Bereinigt endgueltig physische Spuren geloeschter Daten auf Platten- und Speicher-Ebene.

// FILE-CONTEXT
// ZWECK: Physischer Purger für LSM Storage (§12.7.4, Verträge W1-P01).
// INVARIANTEN: Nullt und unlinkt verwaiste SSTable- und WAL-Segmente; atomarer Memtable-Flush.
// HOTSPOTS: [1-100]

#![forbid(unsafe_code)]

use crate::compat::scanner::runtime::ScannerRuntime;
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
                let entries = reader.iter().await.map_err(|e| {
                    ContextraError::Storage(format!(
                        "Purge: SSTable {} nicht lesbar: {e}",
                        reader.file_path().display()
                    ))
                })?;
                let total_entries = entries.len();
                let mut retained = Vec::new();

                for (k, v, seq) in entries {
                    if !k.starts_with(&target_prefix) {
                        retained.push((k, v, seq));
                    } else {
                        purged_units += 1;
                    }
                }

                if retained.len() == total_entries {
                    // Keine Target-Einträge in dieser SSTable -> behalten
                    new_readers.push(Arc::clone(reader));
                } else if retained.is_empty() {
                    // Alle Einträge gehörten zum Target -> Datei entfernen
                    sst_files_to_purge.push(reader.file_path().to_path_buf());
                } else {
                    // Datei filtern und neu schreiben
                    let old_path = reader.file_path();
                    let new_path = old_path.with_extension("purged.sst");
                    let mut builder = SstableBuilder::create_with_vfs(
                        &new_path,
                        storage.key_manager(),
                        storage.vfs().clone(),
                    )
                    .await
                    .map_err(|e| {
                        ContextraError::Storage(format!(
                            "Purge: Builder {} nicht erstellbar: {e}",
                            new_path.display()
                        ))
                    })?;
                    for (rk, rv, rseq) in retained {
                        builder.add(&rk, &rv, rseq, 1).await.map_err(|e| {
                            ContextraError::Storage(format!("Purge: Schreibfehler: {e}"))
                        })?;
                    }
                    builder.finish().await.map_err(|e| {
                        ContextraError::Storage(format!(
                            "Purge: Finalisierung {} fehlgeschlagen: {e}",
                            new_path.display()
                        ))
                    })?;
                    let new_reader = SstableReader::open_with_key_manager(
                        &new_path,
                        block_cache.clone(),
                        storage.key_manager(),
                    )
                    .await
                    .map_err(|e| {
                        ContextraError::Storage(format!(
                            "Purge: Oeffnen {} fehlgeschlagen: {e}",
                            new_path.display()
                        ))
                    })?;
                    new_readers.push(Arc::new(new_reader));
                    sst_files_to_purge.push(old_path.to_path_buf());
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

            let vfs = self.storage.vfs();
            if let Ok(dir_entries) = vfs.list_dir(data_dir) {
                for name_str in dir_entries {
                    let path = data_dir.join(&name_str);
                    if path.extension().and_then(|s| s.to_str()) == Some("sst")
                        && !active_paths.contains(&path)
                    {
                        sst_files_to_purge.push(path);
                    }
                }
            }

            for old_path in sst_files_to_purge {
                let len = match tokio::fs::metadata(&old_path).await {
                    Ok(m) => m.len(),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(e) => return Err(ContextraError::Io(e)),
                };
                zero_and_unlink(&old_path, len).await?;
                bytes_freed += len;
            }
            vfs.sync_dir(data_dir).map_err(|e| {
                ContextraError::Storage(format!("Purge: Dir-Fsync nach SST-Unlink: {e}"))
            })?;

            // 4. WAL-Segmente filtern / bereinigen
            let mut wal_files = Vec::new();
            if let Ok(dir_entries) = vfs.list_dir(data_dir) {
                for name_str in dir_entries {
                    let path = data_dir.join(&name_str);
                    let is_wal = (name_str.starts_with("wal-") && name_str.ends_with(".log"))
                        || name_str == "wal.log"
                        || name_str.ends_with(".wal");
                    if is_wal {
                        wal_files.push(path);
                    }
                }
            }

            for wal_path in wal_files {
                let wal = Wal::open_read_only(&wal_path, None).await
                    .map_err(|e| ContextraError::Storage(format!("Failed to open WAL {}: {e}", wal_path.display())))?;
                let entries = wal.replay().await
                    .map_err(|e| ContextraError::Storage(format!("Failed to replay WAL {}: {e}", wal_path.display())))?;
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
                        let metadata = tokio::fs::metadata(&wal_path)
                            .await
                            .map_err(ContextraError::Io)?;
                        let len = metadata.len();
                        zero_and_unlink(&wal_path, len).await?;
                        bytes_freed += len;
                    } else {
                        // WAL neu schreiben ohne Target-Records
                        let tmp_wal_path = wal_path.with_extension("purged.log");
                        let integrity_key = Wal::load_or_create_integrity_key(&wal_path)
                            .await
                            .map_err(|e| ContextraError::Storage(format!("Failed to load integrity key for {}: {e}", wal_path.display())))?;
                        let mut file_bytes = Vec::new();
                        file_bytes.extend_from_slice(&WAL_V3_HEADER);
                        let mut prev_hmac = [0u8; 32];
                        for (op, seq_no) in retained_wal {
                            let entry = WalEntry::try_new(op, seq_no, &integrity_key, prev_hmac)
                                .map_err(|e| ContextraError::Storage(format!("Failed to construct WalEntry during purge: {e}")))?;
                            prev_hmac = entry.checksum;
                            let ebytes = entry.to_bytes()
                                .map_err(|e| ContextraError::Storage(format!("Failed to serialize WalEntry during purge: {e}")))?;
                            file_bytes.extend_from_slice(&ebytes);
                        }

                        // Temp-Datei schreiben und per fsync sichern
                        {
                            use tokio::io::AsyncWriteExt;
                            let mut tmp_file = tokio::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(&tmp_wal_path)
                                .await
                                .map_err(ContextraError::Io)?;
                            tmp_file.write_all(&file_bytes)
                                .await
                                .map_err(ContextraError::Io)?;
                            tmp_file.sync_all()
                                .await
                                .map_err(ContextraError::Io)?;
                        }

                        // Atomar ersetzen: Erst alt nach .trash umbenennen, tmp nach wal_path, dann .trash nullen und löschen
                        let trash_wal_path = wal_path.with_extension("trash.log");
                        tokio::fs::rename(&wal_path, &trash_wal_path)
                            .await
                            .map_err(ContextraError::Io)?;
                        tokio::fs::rename(&tmp_wal_path, &wal_path)
                            .await
                            .map_err(ContextraError::Io)?;

                        let metadata = tokio::fs::metadata(&trash_wal_path)
                            .await
                            .map_err(ContextraError::Io)?;
                        let len = metadata.len();
                        zero_and_unlink(&trash_wal_path, len).await?;
                        bytes_freed += len.saturating_sub(file_bytes.len() as u64);
                    }
                }
            }
            vfs.sync_dir(data_dir).map_err(|e| {
                ContextraError::Storage(format!("Purge: Dir-Fsync nach WAL-Bereinigung: {e}"))
            })?;

            Ok::<(), ContextraError>(())
        })?;

        Ok(PurgeStats {
            purged_units,
            bytes_freed,
        })
    }
}

/// Ueberschreibt `path` in Chunks mit Nullen, fsynct und entfernt die Datei.
///
/// Kein Fehler wird unterdrueckt; der Aufrufer fsynct das Verzeichnis.
async fn zero_and_unlink(path: &std::path::Path, len: u64) -> Result<()> {
    use tokio::io::AsyncWriteExt;
    const CHUNK: usize = 1 << 20;
    let zeros = vec![0u8; CHUNK];
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .await
        .map_err(ContextraError::Io)?;
    let mut remaining = len;
    while remaining > 0 {
        let n = usize::try_from(remaining).map_or(CHUNK, |r| r.min(CHUNK));
        file.write_all(&zeros[..n]).await.map_err(ContextraError::Io)?;
        remaining -= n as u64;
    }
    file.sync_all().await.map_err(ContextraError::Io)?;
    drop(file);
    tokio::fs::remove_file(path).await.map_err(ContextraError::Io)
}
