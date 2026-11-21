//! # Physische Löschung: Schritte 4c & 4d (SPEC §5.3)
//!
//! Dieses Modul bereinigt physische Spuren eines gelöschten Schlüsselbereichs
//! (Prefix) im LSM-Kern. Die Schlüsselvernichtung (Schritt 3) erfolgt außerhalb
//! dieses Moduls; hier werden die verbleibenden Klartext-Schlüssel und
//! Ciphertexte aus MemTable, WAL und SSTables entfernt.
//!
//! ## Ablauf von [`LsmStorage::purge_prefix`]
//! 1. **Snapshot-Gate:** Offene Snapshots blockieren den Purge (fail-closed).
//! 2. **4c.1 Flush:** Aktive MemTable wird als L0-SSTable persistiert (ohne Sperren gehalten).
//! 3. **4c.2 Schreib-Barriere:** `commit_mutex` (Commits) und `flush_mutex` (Flush/Rotation/Kompaktion-Swap)
//!    werden gehalten. Eine nicht leere MemTable bricht ab (fail-closed, Aufrufer wiederholt).
//! 4. **4d.1/4d.2 Target-Prüfung & Filter:** Jede SSTable mit Target-Einträgen wird neu geschrieben
//!    ohne diese Einträge (TX-IDs bleiben erhalten). Vollständig betroffene SSTables werden entfernt.
//! 5. **4d.3 Schreiben neuer SSTables:** Ausschließlich mit den erhaltenen Einträgen.
//! 6. **4d.4 Manifest:** Atomarer Commit aller `Replace`/`Remove`-Einträge per `append_batch`,
//!    erst danach wird die In-Memory-Liste getauscht.
//! 7. **4d.5 Scrubbing:** Alte SSTable-Dateien werden über [`scrub_if_safe`] mit Nullen überschrieben
//!    und entfernt. Liegen noch Leser auf einer Datei, verweigert `scrub_if_safe` (fail-closed).
//!
//! ## Invarianten
//! - INV-6: Das Manifest-Format bleibt unverändert; es werden nur bestehende Eintragsarten genutzt.
//! - INV-11/T-DEL-1: Neu geschriebene SSTables nutzen denselben Key-Manager wie das Original.
//! - INV-12: Fehlermeldungen enthalten Pfade und Gründe, aber keine Klartextinhalte.
//!
//! ## Bekannte Grenzen (dokumentiert, nicht still verschwiegen)
//! - Ein Crash zwischen Manifest-Commit (4d.4) und Scrubbing (4d.5) hinterlässt eine nicht genullte
//!   Alt-Datei, die nicht mehr im Manifest steht. Der Schlüssel ist bereits vernichtet (Schritt 3),
//!   der Inhalt ist daher nicht lesbar, aber physisch nicht überschrieben.
//! - Der Purge arbeitet nur auf dem echten Dateisystem; `scrub_if_safe` nutzt `std::fs` direkt.

use crate::error::{StoreError, StoreResult};
use crate::lsm::LsmStorage;
use crate::manifest::ManifestEntry;
use crate::io_seam::Vfs;
use crate::sstable::{SstableBuilder, SstableReader};
use contextra_durable_fs::{scrub_if_safe, ScrubOutcome, ScrubPolicy};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Dateiendung für neu geschriebene (gefilterte) SSTables.
pub const PURGED_SST_SUFFIX: &str = "purged.sst";

/// Ergebnis eines physischen Löschlaufs. Dient der Messung (INV-7: nur gemessene Stufen sind „gelöscht").
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PurgeReport {
    /// Anzahl der entfernten Einträge (Target-Records) aus SSTables.
    pub purged_records: u64,
    /// Anzahl der neu geschriebenen SSTables.
    pub rewritten_sstables: u64,
    /// Anzahl der vollständig entfernten SSTables.
    pub removed_sstables: u64,
    /// Anzahl der über `scrub_if_safe` genullten und entfernten Dateien.
    pub scrubbed_files: u64,
    /// Summe der genullten Bytes.
    pub scrubbed_bytes: u64,
}

/// Strikte Bereinigung für den Purge-Pfad (fail-closed).
///
/// Jede Verweigerung von `scrub_if_safe` wird als Fehler gemeldet, weil der Purge sonst eine
/// Löschung behaupten würde, die physisch nicht stattgefunden hat.
fn scrub_strict(path: &Path, reader_alive: bool, report: &mut PurgeReport) -> StoreResult<()> {
    match scrub_if_safe(path, ScrubPolicy::OverwriteZeros, reader_alive, false) {
        Ok(ScrubOutcome::Scrubbed(receipt)) => {
            report.scrubbed_files += 1;
            report.scrubbed_bytes = report.scrubbed_bytes.saturating_add(receipt.bytes_scrubbed);
            Ok(())
        }
        Ok(ScrubOutcome::RemovedUnscrubbed) => {
            report.scrubbed_files += 1;
            Ok(())
        }
        Ok(ScrubOutcome::Refused(reason)) => Err(StoreError::Erasure(format!(
            "scrubbing refused for {}: {reason:?}",
            path.display()
        ))),
        Err(e) => Err(StoreError::Erasure(format!(
            "scrubbing failed for {}: {e}",
            path.display()
        ))),
    }
}

/// Best-Effort-Bereinigung für den regulären Flush (versiegelte WAL-Segmente).
///
/// Gelingt das Scrubbing nicht (z. B. Hardlinks oder Copy-on-Write-Dateisystem), wird die Datei
/// über das VFS entfernt und die Verweigerung protokolliert. Der Flush selbst darf daran nicht
/// scheitern; die strikte Prüfung liegt im Purge-Pfad.
pub(crate) fn scrub_or_remove_best_effort(vfs: &dyn Vfs, path: &Path) -> StoreResult<()> {
    match scrub_if_safe(path, ScrubPolicy::OverwriteZeros, false, false) {
        Ok(ScrubOutcome::Scrubbed(_)) | Ok(ScrubOutcome::RemovedUnscrubbed) => Ok(()),
        Ok(ScrubOutcome::Refused(reason)) => {
            tracing::warn!(
                path = %path.display(),
                ?reason,
                "WAL scrubbing refused during flush cleanup; falling back to plain removal"
            );
            vfs.remove_file(path)
        }
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "WAL scrubbing failed during flush cleanup; falling back to plain removal"
            );
            vfs.remove_file(path)
        }
    }
}

/// Entfernt neu geschriebene Dateien nach einem Abbruch (best effort, Fehler unterdrückt
/// nur für bereits nicht vorhandene Dateien).
fn cleanup_created(vfs: &dyn Vfs, created: &[PathBuf]) {
    for path in created {
        if let Err(e) = vfs.remove_file(path) {
            tracing::debug!(path = %path.display(), error = %e, "purge cleanup of new SSTable failed");
        }
    }
}

impl LsmStorage {
    /// Führt die physische Löschung eines Schlüssel-Prefixes aus (SPEC §5.3, Schritte 4c und 4d).
    ///
    /// Der Aufrufer ist für die Schlüsselvernichtung (Schritt 3) verantwortlich und MUSS sie
    /// vor diesem Aufruf abgeschlossen haben. Der Purge ist fail-closed: Jeder Fehler lässt den
    /// Store in einem konsistenten Zustand zurück, und der Aufruf kann wiederholt werden.
    pub async fn purge_prefix(&self, target_prefix: &[u8]) -> StoreResult<PurgeReport> {
        if !self.flush_purge_fence.enter_purge() {
            return Err(StoreError::PendingDeleteBlocked(
                "a flush currently owns the purge fence; retry purge".into(),
            ));
        }
        let _purge_fence = PurgeFenceGuard(&self.flush_purge_fence);

        if self.snapshot_registry.min_active_seqno() != u64::MAX {
            return Err(StoreError::Erasure(
                "purge blocked by open snapshot in SnapshotPins".into(),
            ));
        }

        // 4c.1: Flush ohne gehaltene Sperren (flush() nimmt selbst flush_mutex/commit_mutex).
        self.flush().await?;

        // 4c.2: Schreib-Barriere. Commits und Flush/Rotation sind bis zum Ende blockiert.
        let _commit_guard = self.commit_mutex.lock().await;
        let _flush_guard = self.flush_mutex.lock().await;

        {
            let state = self.state.read().await;
            if !state.memtable.is_empty() || !state.immutable_memtables.is_empty() {
                return Err(StoreError::Erasure(
                    "purge barrier: memtable not drained (concurrent writes), retry".into(),
                ));
            }
        }
        {
            let pending = self.pending_sealed_wals.lock().await;
            if !pending.is_empty() {
                return Err(StoreError::Erasure(
                    "purge barrier: sealed WAL segments pending, retry".into(),
                ));
            }
        }

        let mut report = PurgeReport::default();
        let vfs = Arc::clone(self.vfs());
        let sst_arc = self.sstables_arc();

        // 4d.1 / 4d.2 / 4d.3: Plan erstellen und neue SSTables schreiben.
        let mut created: Vec<PathBuf> = Vec::new();
        let mut manifest_entries: Vec<ManifestEntry> = Vec::new();
        // (alter Reader, optionaler neuer Reader). `None` bedeutet vollständig entfernt.
        let mut plan: Vec<(Arc<SstableReader>, Option<Arc<SstableReader>>)> = Vec::new();

        let planned: StoreResult<()> = async {
            let snapshot: Vec<Arc<SstableReader>> = sst_arc.read().await.clone();
            for reader in &snapshot {
                let entries = reader.iter_with_tx().await?;
                let total = entries.len();
                let retained: Vec<_> = entries
                    .into_iter()
                    .filter(|(k, _, _, _)| !k.starts_with(target_prefix))
                    .collect();
                let purged = total - retained.len();
                if purged == 0 {
                    continue;
                }
                report.purged_records = report.purged_records.saturating_add(purged as u64);
                let old_path = reader.file_path().to_path_buf();

                if retained.is_empty() {
                    manifest_entries.push(ManifestEntry::Remove {
                        path: old_path.clone(),
                    });
                    plan.push((Arc::clone(reader), None));
                    report.removed_sstables += 1;
                } else {
                    let new_path = old_path.with_extension(PURGED_SST_SUFFIX);
                    let mut builder = SstableBuilder::create_with_vfs(
                        &new_path,
                        self.key_manager(),
                        Arc::clone(&vfs),
                    )
                    .await?;
                    created.push(new_path.clone());
                    for (k, v, seq, tx) in &retained {
                        builder.add(k, v, *seq, *tx).await?;
                    }
                    let _meta = builder.finish().await?;
                    let new_reader = SstableReader::open_with_vfs(
                        &new_path,
                        self.block_cache(),
                        self.key_manager(),
                        Arc::clone(&vfs),
                    )
                    .await?;
                    manifest_entries.push(ManifestEntry::Replace {
                        removed: vec![old_path],
                        added: new_path,
                        added_max_tx: new_reader.metadata().max_tx_id,
                        rank: 0,
                    });
                    plan.push((Arc::clone(reader), Some(Arc::new(new_reader))));
                    report.rewritten_sstables += 1;
                }
            }
            Ok(())
        }
        .await;

        if let Err(e) = planned {
            cleanup_created(vfs.as_ref(), &created);
            return Err(e);
        }

        if plan.is_empty() {
            return Ok(report);
        }

        // 4d.4: Konsistenz prüfen, Manifest atomar committen, dann In-Memory-Liste tauschen.
        // Der SSTable-Schreibzugriff bleibt über Commit und Swap gehalten, damit keine
        // parallele Kompaktion dazwischenfunkt.
        {
            let mut guard = sst_arc.write().await;
            let still_present = plan
                .iter()
                .all(|(old, _)| guard.iter().any(|r| Arc::ptr_eq(r, old)));
            if !still_present {
                cleanup_created(vfs.as_ref(), &created);
                return Err(StoreError::Erasure(
                    "concurrent SSTable change during purge, retry".into(),
                ));
            }

            if let Err(e) = self.manifest.append_batch(&manifest_entries).await {
                cleanup_created(vfs.as_ref(), &created);
                return Err(e);
            }

            guard.retain(|r| !plan.iter().any(|(old, _)| Arc::ptr_eq(r, old)));
            for (_, new) in &plan {
                if let Some(new_reader) = new {
                    guard.push(Arc::clone(new_reader));
                }
            }
            guard.sort_by_key(|sst| sst.metadata().max_seq & !contextra_ids::TOMBSTONE_BIT);
        }

        // 4d.5: Alte Dateien über scrub_if_safe genullen und entfernen.
        // `plan` hält je Alt-Reader genau eine Referenz; weitere Referenzen stammen von Lesern.
        let data_dir = self.data_path().to_path_buf();
        for (old, _new) in plan {
            let reader_alive = Arc::strong_count(&old) > 1;
            let path = old.file_path().to_path_buf();
            drop(old);
            scrub_strict(&path, reader_alive, &mut report)?;
        }

        // A cache entry does not carry enough scope information to evict just
        // one record safely. Invalidating the shared cache is conservative and
        // prevents replaced table blocks from surviving a successful purge.
        self.block_cache().clear();

        vfs.sync_dir(&data_dir).map_err(|e| {
            StoreError::Erasure(format!("directory sync after SSTable scrub failed: {e}"))
        })?;

        Ok(report)
    }
}

/// Releases the purge side of the flush/purge fence on every return path.
struct PurgeFenceGuard<'a>(&'a super::fence::FlushPurgeFence);

impl Drop for PurgeFenceGuard<'_> {
    fn drop(&mut self) {
        self.0.leave_purge();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsm::LsmConfig;
    use crate::CompactionConfig;
    use contextra_ids::TxId;
    use std::time::Duration;
    use tempfile::TempDir;

    async fn open(path: &Path) -> LsmStorage {
        let config = LsmConfig {
            path: path.to_path_buf(),
            memtable_size_limit: 1024 * 1024,
            max_ram_mb: 64,
            tx_timeout: Duration::from_secs(60),
            compaction: CompactionConfig::default(),
            encryption_passphrase: None,
            ..Default::default()
        };
        LsmStorage::new(config).await.expect("open storage")
    }

    async fn write(storage: &LsmStorage, tx: u64, key: &[u8], value: &[u8]) {
        let tx = TxId::new(tx);
        storage.put(tx, key, value).await.expect("put");
        storage.commit(tx).await.expect("commit");
    }

    /// Liest alle Dateien unterhalb von `dir` rekursiv und liefert deren Inhalte.
    fn all_file_bytes(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                } else if let Ok(bytes) = std::fs::read(&p) {
                    out.push((p, bytes));
                }
            }
        }
        out
    }

    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
    }

    #[tokio::test]
    async fn purge_removes_target_prefix_and_keeps_others() {
        let tmp = TempDir::new().expect("tempdir");
        let storage = open(tmp.path()).await;

        // Mischung in einer SSTable: Target und Nicht-Target.
        write(&storage, 1, b"tenant-a/doc1", b"secret-a1").await;
        write(&storage, 2, b"tenant-b/doc1", b"keep-b1").await;
        storage.force_flush().await.expect("flush 1");

        // Zweite SSTable, nur Target.
        write(&storage, 3, b"tenant-a/doc2", b"secret-a2").await;
        storage.force_flush().await.expect("flush 2");

        let report = storage.purge_prefix(b"tenant-a/").await.expect("purge");

        assert_eq!(report.purged_records, 2, "two target records must be purged");
        assert_eq!(report.rewritten_sstables, 1, "mixed SSTable is rewritten");
        assert_eq!(report.removed_sstables, 1, "pure-target SSTable is removed");
        assert_eq!(report.scrubbed_files, 2, "both replaced files are scrubbed");
        assert!(report.scrubbed_bytes > 0);

        assert_eq!(storage.get(b"tenant-a/doc1").await.expect("get"), None);
        assert_eq!(storage.get(b"tenant-a/doc2").await.expect("get"), None);
        assert_eq!(
            storage.get(b"tenant-b/doc1").await.expect("get"),
            Some(bytes::Bytes::from_static(b"keep-b1"))
        );
    }

    #[tokio::test]
    async fn purge_leaves_no_residue_of_target_on_disk() {
        let tmp = TempDir::new().expect("tempdir");
        let storage = open(tmp.path()).await;

        write(&storage, 1, b"tenant-a/residue-probe", b"payload-probe").await;
        write(&storage, 2, b"tenant-b/other", b"other-payload").await;
        storage.force_flush().await.expect("flush");

        storage.purge_prefix(b"tenant-a/").await.expect("purge");

        for (path, bytes) in all_file_bytes(tmp.path()) {
            assert!(
                !contains(&bytes, b"tenant-a/residue-probe"),
                "target key residue found in {}",
                path.display()
            );
        }
    }

    #[tokio::test]
    async fn purged_state_survives_reopen() {
        let tmp = TempDir::new().expect("tempdir");
        {
            let storage = open(tmp.path()).await;
            write(&storage, 1, b"tenant-a/x", b"gone").await;
            write(&storage, 2, b"tenant-b/y", b"stays").await;
            storage.force_flush().await.expect("flush");
            storage.purge_prefix(b"tenant-a/").await.expect("purge");
            storage.close().await.expect("close");
        }

        let reopened = open(tmp.path()).await;
        assert_eq!(reopened.get(b"tenant-a/x").await.expect("get"), None);
        assert_eq!(
            reopened.get(b"tenant-b/y").await.expect("get"),
            Some(bytes::Bytes::from_static(b"stays"))
        );
    }

    #[tokio::test]
    async fn purge_without_matches_is_a_noop() {
        let tmp = TempDir::new().expect("tempdir");
        let storage = open(tmp.path()).await;
        write(&storage, 1, b"tenant-b/y", b"stays").await;
        storage.force_flush().await.expect("flush");

        let report = storage.purge_prefix(b"tenant-z/").await.expect("purge");
        assert_eq!(report, PurgeReport::default());
    }
}
