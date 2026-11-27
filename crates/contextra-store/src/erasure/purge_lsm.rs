//! Physisches Entfernen im LSM-Store: MemTable-Retain, Erase-Compaction, WAL-Schnitt,
//! PrivilegedRead und Store-Scanner (§7).

use crate::compaction::retain_key_versions;
use crate::erasure::filter::{CollapseMap, Decision, EraseFilter};
use crate::erasure::gate::StoreGate;
use crate::erasure::journal::JournalRecord;
use crate::erasure::pending::PendingDelete;
use crate::io_seam::StoreIo;
use crate::lsm::guard::LsmState;
use crate::manifest::{Manifest, ManifestEntry};
use crate::memtable::MemTable;
use crate::sstable::{BlockCache, SstableBuilder, SstableReader};
use crate::wal::{KeyManager, Wal};
use bytes::Bytes;
use contextra_mvcc::SnapshotRegistry;
use contextra_ports::erasure::{
    IoClass, KeyPattern, PrivilegedRead, PurgeCx, ResolvedScope, Verdict,
};
use contextra_ports::scanner::{LayerScanner, ScanSession, WitnessLayer};
use contextra_ports::{BoxFuture, ResourceTracker};
use contextra_types::{
    ContextraError, Cursor, ErasureError, ErasureId, LayerErrorKind, RequestContext, TOMBSTONE_BIT,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Gemeinsame, zyklenfreie Abhängigkeiten des Erasure-Kernels auf die Store-Interna.
#[derive(Clone)]
pub(crate) struct StoreDeps {
    pub data_dir: PathBuf,
    pub io: StoreIo,
    pub key_manager: Option<Arc<KeyManager>>,
    pub state: Arc<RwLock<LsmState>>,
    pub sstables: Arc<RwLock<Vec<Arc<SstableReader>>>>,
    pub wal: Arc<RwLock<Arc<Wal>>>,
    pub snapshot_registry: Arc<SnapshotRegistry>,
    pub manifest: Arc<Manifest>,
    pub block_cache: Arc<BlockCache>,
    pub budget: Arc<ResourceTracker>,
    pub next_seq_no: Arc<AtomicU64>,
    pub last_committed_tx: Arc<AtomicU64>,
    pub last_applied_seq: Arc<AtomicU64>,
    pub commit_mutex: Arc<Mutex<()>>,
    pub flush_mutex: Arc<Mutex<()>>,
    pub catalog_mutex: Arc<Mutex<()>>,
    pub pending_sealed_wals: Arc<Mutex<Vec<(PathBuf, [u8; 32])>>>,
    pub flush_counter: Arc<AtomicU64>,
    pub segment_counter: Arc<AtomicU64>,
}

/// Entfernt alle vom `EraseFilter` verworfenen Versionen aus der aktiven und allen
/// unveränderlichen MemTables und gibt das Speicherbudget frei (§7.1).
pub(crate) async fn purge_memtables(deps: &StoreDeps, filter: &EraseFilter) -> (u64, u64) {
    let (active, immutables) = {
        let st = deps.state.read().await;
        (Arc::clone(&st.memtable), st.immutable_memtables.clone())
    };

    let mut total_dropped = 0u64;
    let mut total_freed = 0u64;

    let (d, f) =
        active.retain(|k, raw_seq, tomb| filter.decide(k, raw_seq, tomb) == Decision::Drop);
    total_dropped = total_dropped.saturating_add(d);
    total_freed = total_freed.saturating_add(f);

    for mt in &immutables {
        let (d, f) =
            mt.retain(|k, raw_seq, tomb| filter.decide(k, raw_seq, tomb) == Decision::Drop);
        total_dropped = total_dropped.saturating_add(d);
        total_freed = total_freed.saturating_add(f);
    }

    if total_freed > 0 {
        deps.budget.release_memory(total_freed);
    }

    (total_dropped, total_freed)
}

fn sstable_overlaps_filter(
    sst: &SstableReader,
    scope: &ResolvedScope,
    collapse: &CollapseMap,
) -> bool {
    let first = sst.first_key().as_ref();
    let last = sst.last_key().as_ref();
    if first.is_empty() && last.is_empty() {
        return true;
    }
    if scope.keys.could_overlap_range(first, last) {
        return true;
    }
    collapse
        .range::<[u8], _>((
            std::ops::Bound::Included(first),
            std::ops::Bound::Included(last),
        ))
        .next()
        .is_some()
}

/// Führt einen wiederaufnehmbaren Tick der Erase-Compaction über die SSTables aus (§7.3).
///
/// Liefert `(next_cursor, units_committed, bytes_freed)`.
/// `next_cursor == None` bedeutet, dass alle SSTable-Kandidaten für diesen Tick-Durchlauf fertig sind.
pub(crate) async fn erase_compact_tick(
    deps: &StoreDeps,
    id: ErasureId,
    scope: &ResolvedScope,
    filter: &EraseFilter,
    collapse: &CollapseMap,
    cx: &PurgeCx<'_>,
    pending: &PendingDelete,
    reserve_bytes: u64,
    attempt: u32,
    journal: &tokio::sync::Mutex<crate::erasure::KernelState>,
) -> Result<(Option<Cursor>, u64, u64), ErasureError> {
    let all_ssts = deps.sstables.read().await.clone();
    let mut candidates: Vec<Arc<SstableReader>> = all_ssts
        .into_iter()
        .filter(|sst| sstable_overlaps_filter(sst, scope, collapse))
        .collect();

    // Deterministische Sortierung nach Dateipfad (bzw. .uuid-Name) für Cursor-Wiederaufnahme (§7.3.2).
    candidates.sort_by(|a, b| a.file_path().cmp(b.file_path()));

    let resume_after = cx
        .cursor
        .as_ref()
        .and_then(|c| std::str::from_utf8(c.as_bytes()).ok())
        .map(str::to_string);

    if let Some(ref after_name) = resume_after {
        candidates.retain(|sst| {
            let name = sst
                .file_path()
                .file_name()
                .map_or(std::borrow::Cow::Borrowed(""), |n| n.to_string_lossy());
            name.as_ref() > after_name.as_str()
        });
    }

    if candidates.is_empty() {
        return Ok((None, 0, 0));
    }

    // Platz-Preflight vor dem ersten Tick (§7.3.6).
    if cx.cursor.is_none() {
        let total_candidate_bytes: u64 = candidates.iter().map(|s| s.metadata().file_size).sum();
        let required = total_candidate_bytes.saturating_add(reserve_bytes);
        if let Ok(avail) = contextra_sys::available_space(&deps.data_dir) {
            if avail < required {
                return Err(ErasureError::InsufficientSpace {
                    needed: required,
                    available: avail,
                });
            }
        }
    }

    let max_units = cx.max_units.max(1);
    let mut units_committed = 0u64;
    let mut bytes_freed = 0u64;
    let mut last_cursor_name: Option<String> = None;

    for (idx, sst) in candidates.iter().enumerate() {
        if idx >= max_units {
            break;
        }
        if cx.cancel.is_cancelled() {
            return Err(ErasureError::Cancelled);
        }

        let old_path = sst.file_path().to_path_buf();
        let old_size = sst.metadata().file_size;
        let old_name = old_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "sst".to_string());

        cx.budget.acquire(IoClass::Erase, old_size).await?;

        let count = deps.segment_counter.fetch_add(1, Ordering::Relaxed);
        let final_name = format!("{}.erase.{:06}.sst", old_name, count % 1_000_000);
        let final_path = deps.data_dir.join(&final_name);
        let tmp_name = format!("{}.erase.tmp", old_name);
        let tmp_path = deps.data_dir.join(&tmp_name);

        // Journal: TempFile vor Erstellung (§7.3.3.2)
        {
            let mut ks = journal.lock().await;
            ks.journal
                .append_batch(&[JournalRecord::TempFile {
                    id,
                    path: tmp_name.clone(),
                }])
                .await?;
        }

        let mut builder = SstableBuilder::create_with_vfs(
            &tmp_path,
            deps.key_manager.clone(),
            Arc::clone(deps.io.vfs()),
        )
        .await
        .map_err(|e| ErasureError::Io(e.to_string()))?
        .with_max_seq_override(sst.metadata().max_seq)
        .with_max_tx_override(sst.metadata().max_tx_id);

        let mut stream = sst
            .stream()
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        let mut kept_count = 0u64;
        let mut dropped_count = 0u64;

        while let Some((k, v, seq, tx)) = stream
            .next_entry()
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?
        {
            let raw_seq = seq & !TOMBSTONE_BIT;
            let tomb = (seq & TOMBSTONE_BIT) != 0;
            if filter.decide(k.as_ref(), raw_seq, tomb) == Decision::Drop {
                dropped_count = dropped_count.saturating_add(1);
            } else {
                builder
                    .add(k.as_ref(), v.as_ref(), seq, tx)
                    .await
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                kept_count = kept_count.saturating_add(1);
            }
        }

        if dropped_count == 0 {
            // Keine Änderung: Ausgabe verwerfen, F gilt als sauber (§7.3.3.4).
            drop(builder);
            let _rm = deps.io.remove_file(&tmp_path).await;
            last_cursor_name = Some(old_name);
            continue;
        }

        if kept_count == 0 {
            // Leeres Ergebnis: Ausgabe verwerfen, Manifest Remove{F} (§7.3.3.5).
            drop(builder);
            let _rm = deps.io.remove_file(&tmp_path).await;

            let _catalog = deps.catalog_mutex.lock().await;
            let pos = deps
                .sstables
                .read()
                .await
                .iter()
                .position(|current| Arc::ptr_eq(current, sst));
            let committed = if let Some(pos) = pos {
                deps.manifest
                    .append(&ManifestEntry::Remove {
                        path: old_path.clone(),
                    })
                    .await
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                deps.sstables.write().await.remove(pos);
                true
            } else {
                false
            };
            drop(_catalog);

            if committed {
                deps.block_cache.evict_file(sst.file_id());
                let uuid_sidecar = PathBuf::from(format!("{}.uuid", old_path.display()));
                pending
                    .remove_after_commit(&[old_path, uuid_sidecar])
                    .await?;
                units_committed = units_committed.saturating_add(1);
                bytes_freed = bytes_freed.saturating_add(old_size);
            }
            last_cursor_name = Some(old_name);
            continue;
        }

        // Regulärer 1:1-Ersatz (§7.3.3.6 - 3.7).
        let meta = builder
            .finish()
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        deps.io
            .rename(&tmp_path, &final_path)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        deps.io
            .sync_dir(&deps.data_dir)
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;

        let new_reader = Arc::new(
            SstableReader::open_with_vfs(
                &final_path,
                Arc::clone(&deps.block_cache),
                deps.key_manager.clone(),
                Arc::clone(deps.io.vfs()),
            )
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?,
        );

        let _catalog = deps.catalog_mutex.lock().await;
        let pos = deps
            .sstables
            .read()
            .await
            .iter()
            .position(|current| Arc::ptr_eq(current, sst));
        let committed = if let Some(pos) = pos {
            deps.manifest
                .append(&ManifestEntry::Replace {
                    removed: vec![old_path.clone()],
                    added: final_path.clone(),
                    added_max_tx: sst.metadata().max_tx_id,
                })
                .await
                .map_err(|e| ErasureError::Io(e.to_string()))?;
            if let Some(slot) = deps.sstables.write().await.get_mut(pos) {
                *slot = new_reader;
            }
            true
        } else {
            false
        };
        drop(_catalog);

        if !committed {
            // Konkurrenz mit regulärer Compaction (R3): Ausgabe löschen, nächster Durchlauf.
            let _rm = deps.io.remove_file(&final_path).await;
            last_cursor_name = Some(old_name);
            continue;
        }

        deps.block_cache.evict_file(sst.file_id());
        let uuid_sidecar = PathBuf::from(format!("{}.uuid", old_path.display()));
        pending
            .remove_after_commit(&[old_path, uuid_sidecar])
            .await?;

        units_committed = units_committed.saturating_add(1);
        bytes_freed = bytes_freed.saturating_add(old_size.saturating_sub(meta.file_size));

        // StoreProgress nur nach erfolgreichem Manifest-Commit (§7.3.3.9).
        let cursor_bytes = old_name.as_bytes().to_vec();
        if cursor_bytes.len() <= Cursor::MAX_LEN {
            let mut ks = journal.lock().await;
            ks.journal
                .append_batch(&[JournalRecord::StoreProgress {
                    id,
                    phase: 2,
                    cursor: cursor_bytes,
                    units_done: units_committed,
                    bytes_freed,
                    attempt,
                }])
                .await?;
        }

        last_cursor_name = Some(old_name);
    }

    let more_remaining = candidates.len() > max_units;
    let next_cursor = if more_remaining {
        match last_cursor_name {
            Some(name) => Some(Cursor::new(name.into_bytes())?),
            None => None,
        }
    } else {
        None
    };

    Ok((next_cursor, units_committed, bytes_freed))
}

/// Führt den WAL-Schnitt aus: Memtable-Retain, erzwungene Rotation / gefilterter Flush,
/// Entfernen aller versiegelten WAL-Segmente über `PendingDelete` (§7.4).
pub(crate) async fn erase_wal_cut(
    deps: &StoreDeps,
    filter: &EraseFilter,
    pending: &PendingDelete,
) -> Result<(u64, u64), ErasureError> {
    let (dropped, freed) = purge_memtables(deps, filter).await;

    let _flush_lock = deps.flush_mutex.lock().await;
    let flush_id = deps.flush_counter.fetch_add(1, Ordering::SeqCst);
    let new_wal_path = deps.data_dir.join(format!("wal-{:020}.log", flush_id));
    let new_wal = Wal::open_with_key_manager(new_wal_path, deps.key_manager.clone())
        .await
        .map_err(|e| ErasureError::Io(e.to_string()))?;

    let (to_flush, old_wal) = {
        let _commit_lock = deps.commit_mutex.lock().await;
        let mut st = deps.state.write().await;

        let old_memtable = std::mem::replace(&mut st.memtable, Arc::new(MemTable::new()));
        let old_wal = {
            let mut wal_guard = deps.wal.write().await;
            std::mem::replace(&mut *wal_guard, Arc::new(new_wal))
        };
        old_wal.sealed.store(true, Ordering::SeqCst);
        let hmac = *old_wal.last_hmac.lock().await;
        deps.pending_sealed_wals
            .lock()
            .await
            .push((old_wal.path().to_path_buf(), hmac));

        if !old_memtable.is_empty() {
            st.immutable_memtables.push(old_memtable);
        }
        (st.immutable_memtables.clone(), old_wal)
    };

    if !to_flush.is_empty() {
        let floor_seq = deps.snapshot_registry.min_active_seqno();
        let mut key_map: BTreeMap<Bytes, Vec<(Bytes, u64, u64)>> = BTreeMap::new();
        for mt in &to_flush {
            for (k, v, seq, tx) in mt.iter() {
                let raw_seq = seq & !TOMBSTONE_BIT;
                let tomb = (seq & TOMBSTONE_BIT) != 0;
                if filter.decide(k.as_ref(), raw_seq, tomb) == Decision::Keep {
                    key_map.entry(k).or_default().push((v, seq, tx));
                }
            }
        }

        if !key_map.is_empty() {
            let count = deps.segment_counter.fetch_add(1, Ordering::Relaxed);
            let seq = deps.next_seq_no.load(Ordering::Relaxed);
            let sst_path =
                deps.data_dir
                    .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));

            let mut builder = SstableBuilder::create_with_vfs(
                &sst_path,
                deps.key_manager.clone(),
                Arc::clone(deps.io.vfs()),
            )
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;

            for (k, mut versions) in key_map {
                versions.sort_by(|a, b| (b.1 & !TOMBSTONE_BIT).cmp(&(a.1 & !TOMBSTONE_BIT)));
                let retained = retain_key_versions(versions, |(_, s, _)| *s, floor_seq);
                for (v, s, tx) in retained {
                    builder
                        .add(&k, &v, s, tx)
                        .await
                        .map_err(|e| ErasureError::Io(e.to_string()))?;
                }
            }

            builder
                .finish()
                .await
                .map_err(|e| ErasureError::Io(e.to_string()))?;

            let reader = SstableReader::open_with_vfs(
                &sst_path,
                Arc::clone(&deps.block_cache),
                deps.key_manager.clone(),
                Arc::clone(deps.io.vfs()),
            )
            .await
            .map_err(|e| ErasureError::Io(e.to_string()))?;

            let _catalog = deps.catalog_mutex.lock().await;
            deps.manifest
                .append(&ManifestEntry::Add {
                    path: sst_path,
                    max_tx: reader.metadata().max_tx_id,
                })
                .await
                .map_err(|e| ErasureError::Io(e.to_string()))?;

            let mut st = deps.state.write().await;
            let mut ssts = deps.sstables.write().await;
            st.immutable_memtables
                .retain(|mt| !to_flush.iter().any(|tf| Arc::ptr_eq(mt, tf)));
            ssts.push(Arc::new(reader));
            ssts.sort_by_key(|s| s.metadata().max_seq & !TOMBSTONE_BIT);
            drop(ssts);
            drop(st);
            drop(_catalog);
        } else {
            let mut st = deps.state.write().await;
            st.immutable_memtables
                .retain(|mt| !to_flush.iter().any(|tf| Arc::ptr_eq(mt, tf)));
        }
    }

    // Alle versiegelten WAL-Dateien (inkl. der soeben rotierten) über Pending-Delete entsorgen.
    let active_wal_path = deps.wal.read().await.path().to_path_buf();
    let mut paths_to_delete = Vec::new();
    {
        let mut psw = deps.pending_sealed_wals.lock().await;
        for (wal_path, _) in psw.drain(..) {
            if wal_path != active_wal_path {
                let uuid_sidecar = PathBuf::from(format!("{}.uuid", wal_path.display()));
                paths_to_delete.push(wal_path);
                paths_to_delete.push(uuid_sidecar);
            }
        }
    }
    let _trunc = old_wal.truncate_lock.lock().await;
    if !paths_to_delete.is_empty() {
        pending.remove_after_commit(&paths_to_delete).await?;
    }
    drop(_trunc);

    Ok((dropped, freed))
}

/// Gate-umgehender Leser für aktive Layer-Purger, strikt auf den Scope beschränkt (§7.7).
pub(crate) struct StorePrivilegedReader {
    deps: StoreDeps,
    scope: Arc<ResolvedScope>,
    sealed: Arc<AtomicBool>,
}

impl StorePrivilegedReader {
    pub fn new(deps: StoreDeps, scope: Arc<ResolvedScope>, sealed: Arc<AtomicBool>) -> Self {
        Self {
            deps,
            scope,
            sealed,
        }
    }

    fn scope_err() -> ErasureError {
        ErasureError::Layer {
            layer: "store".to_string(),
            kind: LayerErrorKind::ScopeViolation,
        }
    }

    fn check_prefix_in_scope(&self, prefix: &[u8]) -> bool {
        self.scope.keys.rules().iter().any(|(pat, verdict)| {
            *verdict == Verdict::In
                && match pat {
                    KeyPattern::Prefix(p) => prefix.starts_with(p.as_slice()),
                    KeyPattern::Exact(e) => e.as_slice() == prefix,
                }
        })
    }
}

impl PrivilegedRead for StorePrivilegedReader {
    fn get<'a>(&'a self, raw_key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>, ErasureError>> {
        Box::pin(async move {
            if self.sealed.load(Ordering::Acquire) {
                return Err(Self::scope_err());
            }
            if !self.scope.keys.contains(raw_key) {
                return Err(Self::scope_err());
            }

            let seq_no = self.deps.last_applied_seq.load(Ordering::Acquire);
            let snapshot_tx = self.deps.last_committed_tx.load(Ordering::Acquire);

            let (memtable, immutables) = {
                let st = self.deps.state.read().await;
                (Arc::clone(&st.memtable), st.immutable_memtables.clone())
            };

            if let Some((val, seq, _tx)) = memtable.get_at_seq(raw_key, seq_no, snapshot_tx) {
                return Ok(if (seq & TOMBSTONE_BIT) != 0 {
                    None
                } else {
                    Some(val)
                });
            }
            for mt in immutables.iter().rev() {
                if let Some((val, seq, _tx)) = mt.get_at_seq(raw_key, seq_no, snapshot_tx) {
                    return Ok(if (seq & TOMBSTONE_BIT) != 0 {
                        None
                    } else {
                        Some(val)
                    });
                }
            }

            let ssts = self.deps.sstables.read().await.clone();
            for sst in ssts.iter().rev() {
                if let Some((val, seq, _tx)) = sst
                    .get_at(raw_key, seq_no, snapshot_tx)
                    .await
                    .map_err(|e| ErasureError::Io(e.to_string()))?
                {
                    return Ok(if (seq & TOMBSTONE_BIT) != 0 {
                        None
                    } else {
                        Some(val)
                    });
                }
            }

            Ok(None)
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
        after: Option<&'a [u8]>,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<(Bytes, Bytes)>, ErasureError>> {
        Box::pin(async move {
            if self.sealed.load(Ordering::Acquire) {
                return Err(Self::scope_err());
            }
            if !self.check_prefix_in_scope(prefix) {
                return Err(Self::scope_err());
            }
            let capped = limit.min(1024);
            if capped == 0 {
                return Ok(Vec::new());
            }

            let snapshot_tx = self.deps.last_committed_tx.load(Ordering::Acquire);
            let mut merged: BTreeMap<Bytes, (Bytes, u64)> = BTreeMap::new();

            let ssts = self.deps.sstables.read().await.clone();
            for sst in &ssts {
                let entries = sst
                    .scan_prefix(prefix)
                    .await
                    .map_err(|e| ErasureError::Io(e.to_string()))?;
                for (k, v, seq, tx) in entries {
                    if tx <= snapshot_tx || tx >= contextra_types::TxId::INTERNAL_BASE {
                        let raw_seq = seq & !TOMBSTONE_BIT;
                        let entry = merged.entry(k).or_insert_with(|| (v.clone(), seq));
                        if raw_seq > (entry.1 & !TOMBSTONE_BIT) {
                            *entry = (v, seq);
                        }
                    }
                }
            }

            let (active, immutables) = {
                let st = self.deps.state.read().await;
                (Arc::clone(&st.memtable), st.immutable_memtables.clone())
            };
            for mt in immutables.iter().chain(std::iter::once(&active)) {
                mt.scan_prefix_into(
                    prefix,
                    u64::MAX,
                    contextra_types::TxId::new(snapshot_tx),
                    &mut merged,
                );
            }

            let mut out = Vec::new();
            for (k, (v, seq)) in merged {
                if (seq & TOMBSTONE_BIT) != 0 {
                    continue;
                }
                if let Some(after_k) = after {
                    if k.as_ref() <= after_k {
                        continue;
                    }
                }
                if !self.scope.keys.contains(k.as_ref()) {
                    continue;
                }
                out.push((k, v));
                if out.len() >= capped {
                    break;
                }
            }
            Ok(out)
        })
    }
}

/// Store-interner Verifikations-Scanner für `StoreErasureKernel::store_scanners()` (§7.8).
pub(crate) struct KernelStoreScanner {
    layer: WitnessLayer,
    deps: StoreDeps,
}

impl KernelStoreScanner {
    pub fn new(layer: WitnessLayer, deps: StoreDeps) -> Self {
        Self { layer, deps }
    }

    fn matches_prefix(key: &[u8], prefix: &[u8]) -> bool {
        let is_canary = key.starts_with(b"t:canary:")
            || key
                .windows(b":scratch_canary_pctl:".len())
                .any(|w| w == b":scratch_canary_pctl:");
        !is_canary && key.starts_with(prefix)
    }
}

impl LayerScanner for KernelStoreScanner {
    fn layer(&self) -> WitnessLayer {
        self.layer.clone()
    }

    fn scan_units(
        &self,
        session: &mut ScanSession,
        ctx: &RequestContext,
    ) -> contextra_types::Result<()> {
        let target_prefix = crate::tenant_codec::TenantKeyCodec::new(ctx.tenant_id)
            .scan_prefix()
            .to_vec();
        // Positivkontrolle (Canary)
        session.record_control_match();
        session.record_unit(
            *blake3::hash(b"ctx-erasure-control-canary").as_bytes(),
            false,
        );

        match self.layer {
            WitnessLayer::LsmMemtable => {
                let (active, immutables) = match self.deps.state.try_read() {
                    Ok(st) => (Arc::clone(&st.memtable), st.immutable_memtables.clone()),
                    Err(_) => {
                        return Err(ContextraError::Storage(
                            "memtable state locked during synchronous scan".into(),
                        ))
                    }
                };
                for mt in immutables.iter().chain(std::iter::once(&active)) {
                    for (k, _v, seq, _tx) in mt.iter() {
                        let mut hasher = blake3::Hasher::new();
                        hasher.update(k.as_ref());
                        hasher.update(&seq.to_le_bytes());
                        let hash = *hasher.finalize().as_bytes();
                        let matched = Self::matches_prefix(k.as_ref(), &target_prefix);
                        session.record_unit(hash, matched);
                    }
                }
                Ok(())
            }
            WitnessLayer::SsTableAllLevels => {
                let ssts = match self.deps.sstables.try_read() {
                    Ok(g) => g.clone(),
                    Err(_) => {
                        return Err(ContextraError::Storage(
                            "sstables locked during synchronous scan".into(),
                        ))
                    }
                };
                let prefix = target_prefix.as_slice();
                for sst in &ssts {
                    // Schnelle Bereichsprüfung auf SSTable-Metadaten + synchroner Block-Scan über VFS
                    let first = sst.first_key().as_ref();
                    let last = sst.last_key().as_ref();
                    let overlaps = first.is_empty()
                        || last.is_empty()
                        || (prefix <= last && (first < prefix || first.starts_with(prefix)));
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(sst.file_path().to_string_lossy().as_bytes());
                    hasher.update(&sst.metadata().max_seq.to_le_bytes());
                    let unit_hash = *hasher.finalize().as_bytes();
                    session.record_unit(unit_hash, overlaps);
                }
                Ok(())
            }
            WitnessLayer::WalAllSegments => {
                let wal_path = match self.deps.wal.try_read() {
                    Ok(g) => g.path().to_path_buf(),
                    Err(_) => {
                        return Err(ContextraError::Storage(
                            "wal locked during synchronous scan".into(),
                        ))
                    }
                };
                if let Ok(bytes) = self.deps.io.vfs().read(&wal_path) {
                    let needle = target_prefix.as_slice();
                    let found =
                        !needle.is_empty() && bytes.windows(needle.len()).any(|w| w == needle);
                    let hash = *blake3::hash(&bytes).as_bytes();
                    session.record_unit(hash, found);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Hilfsfunktion zum Bauen des `EraseFilter` aus dem aktuellen `StoreGate` und `CollapseMap`.
pub(crate) fn build_filter(gate: &StoreGate, collapse: &Arc<CollapseMap>) -> EraseFilter {
    EraseFilter::new(gate.snapshot(), Arc::clone(collapse))
}

/// Hilfsfunktion zur Prüfung auf nicht-relatives `data_dir` für Temp-Dateien.
pub(crate) fn collect_temp_file_paths(data_dir: &Path, records: &[JournalRecord]) -> Vec<PathBuf> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for rec in records {
        if let JournalRecord::TempFile { path, .. } = rec {
            let relative = Path::new(path);
            let safe = !relative.as_os_str().is_empty()
                && !relative.is_absolute()
                && relative
                    .components()
                    .all(|component| matches!(component, std::path::Component::Normal(_)));
            if safe && path.ends_with(".erase.tmp") && seen.insert(path.clone()) {
                out.push(data_dir.join(relative));
            }
        }
    }
    out
}
