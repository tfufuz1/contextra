//! # Erasure-Kernel (`contextra-store`, Ring 1)
//!
//! Implementiert das nicht umgehbare Lösch-Subsystem im primären Speicher:
//! - Durables Auftragsjournal (`journal.log`) mit Pre-Allokation, CRC-32 (IEEE) und Vergiftungsschutz
//! - Synchrones, I/O-freies Read-Gate (`StoreGate`) mit copy-on-write Snapshot und Epoche
//! - Zustandsloser Löschfilter (`EraseFilter`) für MemTable, Flush, Compaction und Replay
//! - Physisches Entfernen in MemTable (`MemTable::retain`), SSTables (1:1-Rewrite) und WAL-Schnitt
//! - Atomare Pending-Delete-Liste (`pending-delete.list`) für crashsichere Datei-Entsorgung
//! - Tamper-evidentes Audit-Ledger (`ledger.log`) außerhalb des Datenverzeichnisses
//!
//! ## Öffentliche Fläche
//! Ausschließlich [`StoreErasureKernel`], [`StoreErasureConfig`] und [`BootReport`] sind `pub`.

pub(crate) mod filter;
pub(crate) mod gate;
pub(crate) mod journal;
pub(crate) mod ledger;
pub(crate) mod pending;
pub(crate) mod purge_lsm;

use crate::erasure::filter::CollapseMap;
use crate::erasure::gate::{GatePhase, JobGate, StoreGate};
use crate::erasure::journal::{
    decode_resolved_scope, encode_resolved_scope, layer_sort_key, Journal, JournalRecord,
};
use crate::erasure::ledger::{validate_external_ledger_path, StoreLedger};
use crate::erasure::pending::PendingDelete;
use crate::erasure::purge_lsm::{
    build_filter, collect_temp_file_paths, erase_compact_tick, erase_wal_cut, purge_memtables,
    KernelStoreScanner, StoreDeps, StorePrivilegedReader,
};
use crate::key_source::KeyMaterialSource;
use contextra_ports::erasure::{
    AcceptReceipt, DocScope, EraseRequest, ErasureKernel, GateReceipt, JobView, PrivilegedRead,
    PurgeCx, PurgeOutcome, ResolvedScope, SealReceipt,
};
use contextra_ports::gate::ReadGate;
use contextra_ports::ledger::{LedgerAnchor, LedgerDraft, LedgerSink};
use contextra_ports::scanner::{LayerScanner, LayerWitness, ScanTechnique, WitnessLayer};
use contextra_ports::BoxFuture;
use contextra_types::{
    Cursor, ErasureError, ErasureException, ErasureId, ErasurePolicy, ErasureState, ErasureStatus,
    Guarantee, LayerErrorKind, LayerProgress, LayerState, LayerStatus, Seq, TargetKind, Timestamp,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

const DEFAULT_RESERVE_BYTES: u64 = 8 * 1024 * 1024;
const DEFAULT_MAX_OPEN_JOBS: u32 = 64;
const DEFAULT_MAX_PLAN_BYTES: u32 = 16 * 1024 * 1024;
const DEFAULT_ERASE_IO_SHARE: f32 = 0.5;
const DEFAULT_PENDING_RETRY_LIMIT: u32 = 10;
const JOURNAL_COMPACT_SIZE_THRESHOLD: u64 = 1024 * 1024;
const JOURNAL_COMPACT_COMPLETED_THRESHOLD: u64 = 64;

/// Konfiguration für den [`StoreErasureKernel`] (§3.1).
///
/// Besitzt bewusst kein `Default`, da `dir` explizit vom Datenpfad abgeleitet werden muss.
#[derive(Clone, Debug)]
pub struct StoreErasureConfig {
    /// Ablageverzeichnis für Journal und Pending-Delete-Liste (`<data_dir>/erasure`).
    pub dir: PathBuf,
    /// Optionaler Pfad für das Audit-Ledger (`None` = Profil Standard, `Some(p)` muss außerhalb von `<data_dir>` liegen).
    pub ledger_path: Option<PathBuf>,
    /// Vorab reservierter Speicherplatz für das Journal (Default: 8 MiB).
    pub reserve_bytes: u64,
    /// Maximale Anzahl gleichzeitiger nicht-terminaler Aufträge (Default: 64).
    pub max_open_jobs: u32,
    /// Maximale Bytegröße eines einzelnen Journal-Frames (Default: 16 MiB).
    pub max_plan_bytes: u32,
    /// Anteil der I/O-Bandbreite für physisches Löschen gegenüber Compaction (Default: 0.5).
    pub erase_io_share: f32,
    /// Schwellwert für Wiederholungsversuche bei Pending-Delete vor Alarmierung (Default: 10).
    pub pending_retry_limit: u32,
    /// Erzwingt externes Ledger außerhalb des Datenverzeichnisses (Profil Regulated).
    pub require_external_ledger: bool,
}

impl StoreErasureConfig {
    /// Erzeugt eine Standardkonfiguration für das angegebene Store-Datenverzeichnis.
    pub fn for_data_dir(data_dir: &Path) -> Self {
        Self {
            dir: data_dir.join("erasure"),
            ledger_path: None,
            reserve_bytes: DEFAULT_RESERVE_BYTES,
            max_open_jobs: DEFAULT_MAX_OPEN_JOBS,
            max_plan_bytes: DEFAULT_MAX_PLAN_BYTES,
            erase_io_share: DEFAULT_ERASE_IO_SHARE,
            pending_retry_limit: DEFAULT_PENDING_RETRY_LIMIT,
            require_external_ledger: false,
        }
    }
}

/// Startbericht des Erasure-Kernels nach Recovery (§5.4).
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct BootReport {
    /// `true`, wenn das Journal korrupt war und der Store im Fail-Closed-Modus gestartet ist.
    pub fail_closed: bool,
    /// Anzahl offener (nicht-terminaler) Aufträge nach Recovery.
    pub jobs_open: u32,
    /// `true`, wenn beim Boot ein unvollständiges Tail-Frame abgeschnitten wurde (CP-13).
    pub torn_tail_truncated: bool,
    /// Anzahl der beim Boot über Pending-Delete entfernten Dateien.
    pub pending_deleted: u32,
    /// SSTable-Pfade, die beim Start als unmanifestiert erkannt wurden.
    pub orphans: Vec<PathBuf>,
}

#[inline]
fn layer_map_key(layer: &WitnessLayer) -> (u8, String) {
    let (tag, name) = layer_sort_key(layer);
    (tag, name.to_string())
}

#[derive(Clone)]
pub(crate) struct Job {
    pub id: ErasureId,
    pub state: ErasureState,
    pub plan: Arc<ResolvedScope>,
    pub policy: ErasurePolicy,
    pub expected_layers: Vec<WitnessLayer>,
    pub gated_seq: Option<Seq>,
    pub layer_cursor: BTreeMap<(u8, String), LayerProgress>,
    pub layer_attempts: BTreeMap<(u8, String), u32>,
    pub layer_remaining: BTreeMap<(u8, String), u64>,
    pub collapse: CollapseMap,
    pub attempt: u32,
    pub guarantee_reached: Guarantee,
    pub exceptions: Vec<ErasureException>,
    pub holds: u32,
    pub requested_at: u64,
    pub gated_at: Option<u64>,
    pub sealed_at: Option<u64>,
    pub ledger_index: Option<u64>,
    pub purge_started: bool,
    pub keys_destroyed: bool,
    pub sealed_flag: Arc<AtomicBool>,
    pub raw_records: Vec<JournalRecord>,
}

impl Job {
    fn transition_to(&mut self, next: ErasureState) -> Result<(), ErasureError> {
        if !self.state.can_transition_to(&next) {
            return Err(ErasureError::InvalidTransition);
        }
        self.state = next;
        Ok(())
    }

    fn to_job_gate(&self) -> JobGate {
        JobGate {
            id: self.id,
            scope: Arc::new(self.plan.keys.clone()),
            docs: self.plan.docs.clone(),
            up_to: self.gated_seq.unwrap_or(Seq(u64::MAX)),
            phase: if self.purge_started {
                GatePhase::Erased
            } else {
                GatePhase::Restricted
            },
            reinsert_after_sealed: matches!(
                self.policy.reinsert,
                contextra_types::ReinsertPolicy::AllowAfterSealed
            ),
            sealed: matches!(self.state, ErasureState::Sealed | ErasureState::Completed),
        }
    }

    fn to_status(&self) -> ErasureStatus {
        let mut layers = Vec::with_capacity(self.expected_layers.len());
        for layer in &self.expected_layers {
            let lkey = layer_map_key(layer);
            let progress = self.layer_cursor.get(&lkey);
            let remaining = self.layer_remaining.get(&lkey).copied();
            let attempt = self.layer_attempts.get(&lkey).copied().unwrap_or(0);
            let lstate = match &self.state {
                ErasureState::Requested | ErasureState::Planned | ErasureState::Gating => {
                    LayerState::Pending
                }
                ErasureState::Gated | ErasureState::KeysDestroying => LayerState::Gated,
                ErasureState::Purging => {
                    if let Some(p) = progress {
                        if p.cursor.is_none() {
                            LayerState::Purged
                        } else {
                            LayerState::Purging
                        }
                    } else {
                        LayerState::Gated
                    }
                }
                ErasureState::Verifying => LayerState::Purged,
                ErasureState::Sealed | ErasureState::Completed => LayerState::Verified,
                ErasureState::Failed {
                    layer: failed_layer,
                    ..
                } => {
                    if failed_layer.as_deref() == Some(layer.as_str()) {
                        LayerState::Failed
                    } else {
                        LayerState::Purging
                    }
                }
                ErasureState::Quarantined { layer: q_layer } => {
                    if q_layer == layer.as_str() {
                        LayerState::Quarantined
                    } else {
                        LayerState::Purging
                    }
                }
                _ => LayerState::Pending,
            };
            layers.push(LayerStatus {
                layer: layer.as_str().to_string(),
                state: lstate,
                attempt,
                remaining,
            });
        }

        ErasureStatus {
            id: self.id,
            state: self.state.clone(),
            reached: self.guarantee_reached,
            layers,
            exceptions: self.exceptions.clone(),
            requested_at: self.requested_at,
            gated_at: self.gated_at,
            sealed_at: self.sealed_at,
            ledger_index: self.ledger_index,
        }
    }

    fn to_job_view(&self) -> JobView {
        let mut layers_done = Vec::new();
        for layer in &self.expected_layers {
            if let Some(p) = self.layer_cursor.get(&layer_map_key(layer)) {
                if p.cursor.is_none() {
                    layers_done.push(layer.clone());
                }
            }
        }
        JobView {
            id: self.id,
            state: self.state.clone(),
            scope: Arc::clone(&self.plan),
            layers_done,
            attempt: self.attempt,
        }
    }
}

pub(crate) struct KernelState {
    pub jobs: BTreeMap<ErasureId, Job>,
    pub completed_summaries: BTreeMap<ErasureId, ErasureStatus>,
    pub summary_records: Vec<JournalRecord>,
    pub journal: Journal,
}

impl KernelState {
    fn rebuild_collapse_map(&self) -> Arc<CollapseMap> {
        let mut merged = CollapseMap::new();
        for job in self.jobs.values() {
            for (k, seq) in &job.collapse {
                merged
                    .entry(k.clone())
                    .and_modify(|s| {
                        if *seq > *s {
                            *s = *seq;
                        }
                    })
                    .or_insert(*seq);
            }
        }
        Arc::new(merged)
    }

    fn open_job_count(&self) -> usize {
        self.jobs
            .values()
            .filter(|j| !j.state.is_terminal())
            .count()
    }
}

/// Zentraler Erasure-Kernel des `contextra-store` (Ring 1).
///
/// Implementiert [`ports::ErasureKernel`].
pub struct StoreErasureKernel {
    cfg: StoreErasureConfig,
    health: Arc<parking_lot::RwLock<crate::lsm::engine::StorageHealth>>,
    clock: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::Clock>>>,
    deps: OnceLock<StoreDeps>,
    gate: Arc<StoreGate>,
    pending: Arc<PendingDelete>,
    ledger: Option<Arc<StoreLedger>>,
    state: Arc<Mutex<KernelState>>,
    /// Synchroner Snapshot für `open_jobs()` und `status()` ohne `.await` auf Rang 150.
    status_cache: Arc<parking_lot::RwLock<BTreeMap<ErasureId, ErasureStatus>>>,
    open_views_cache: Arc<parking_lot::RwLock<Vec<JobView>>>,
    sealed_flags_cache: Arc<parking_lot::RwLock<BTreeMap<ErasureId, Arc<AtomicBool>>>>,
    global_collapse: Arc<parking_lot::RwLock<Arc<CollapseMap>>>,
    erasure_log_key: [u8; 32],
}

impl StoreErasureKernel {
    /// A-4: idempotenter Übergang nach `Purging`, extrahiert aus `purge_store`.
    /// Liefert Scope und Versuchszähler aus derselben Sperre zurück.
    pub(crate) async fn begin_purge_inner(
        &self,
        id: ErasureId,
    ) -> Result<(Arc<ResolvedScope>, u32), ErasureError> {
        let mut ks = self.state.lock().await;
        if ks.journal.is_poisoned() {
            return Err(ErasureError::JournalPoisoned);
        }
        let job = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
        let valid_state = matches!(
            job.state,
            ErasureState::Gated
                | ErasureState::KeysDestroying
                | ErasureState::Purging
                | ErasureState::Verifying
                | ErasureState::Failed { .. }
                | ErasureState::Quarantined { .. }
        );
        if !valid_state || job.gated_seq.is_none() {
            return Err(ErasureError::InvalidTransition);
        }
        let resume_transition = !matches!(&job.state, ErasureState::Purging);
        let scope = Arc::clone(&job.plan);
        let attempt = job.attempt;

        if !job.purge_started || resume_transition {
            let previous = job.state.clone();
            if !previous.can_transition_to(&ErasureState::Purging) {
                return Err(ErasureError::InvalidTransition);
            }
            let now = self.clock.read().now_unix_nanos();
            let rec = JournalRecord::PurgeStart { id, ts: now };
            if let Err(e) = ks.journal.append_batch(&[rec.clone()]).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }
            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.purge_started = true;
                job_mut.transition_to(ErasureState::Purging)?;
                job_mut.raw_records.push(rec);
                let jg = job_mut.to_job_gate();
                self.gate
                    .install_snapshot(|s, ep| s.with_upserted_job(ep, jg.clone()));
            }
            self.refresh_caches(&ks);
        }
        Ok((scope, attempt))
    }
    /// Öffnet den Erasure-Kernel, validiert Journal & Ledger und rekonstruiert aktive Gates (§5.4).
    ///
    /// # Fehler
    /// Liefert [`ErasureError::InvalidTarget`] bei verletzter `require_external_ledger`-Regel
    /// oder [`ErasureError::Io`] bei Basis-Verzeichnisfehlern.
    /// Bei korruptem Journal startet der Kernel im Fail-Closed-Modus (`BootReport::fail_closed = true`).
    ///
    /// # Crash
    /// Schneidet unvollständige Tail-Frames ab (CP-13) und bereinigt verwaiste Temp-Dateien (CP-15)
    /// sowie ausstehende Löschungen aus `pending-delete.list` (CP-17, CP-18).
    pub(crate) async fn open_early(
        cfg: StoreErasureConfig,
        data_dir: PathBuf,
        io: crate::io_seam::StoreIo,
        health: Arc<parking_lot::RwLock<crate::lsm::engine::StorageHealth>>,
        clock: Arc<parking_lot::RwLock<Arc<dyn contextra_ports::Clock>>>,
        key_source: &dyn KeyMaterialSource,
        anchor: Option<Arc<dyn LedgerAnchor>>,
    ) -> Result<(Self, BootReport), ErasureError> {
        if cfg.require_external_ledger {
            let Some(ref lp) = cfg.ledger_path else {
                return Err(ErasureError::InvalidTarget(
                    "require_external_ledger requires ledger_path to be set",
                ));
            };
            validate_external_ledger_path(lp, &data_dir)?;
        } else if let Some(ref lp) = cfg.ledger_path {
            validate_external_ledger_path(lp, &data_dir)?;
        }

        // Schlüsselmaterial ableiten (§11.1) — niemals aus Konstanten.
        let mut prk = [0u8; 32];
        key_source
            .fill_key_bytes(&mut prk)
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        let mut ledger_key = [0u8; 32];
        let mut erasure_log_key = [0u8; 32];
        contextra_crypto::hkdf_expand(&prk, b"ctx-ledger-mac-v1", &mut ledger_key)
            .map_err(|e| ErasureError::Io(e.to_string()))?;
        contextra_crypto::hkdf_expand(&prk, b"ctx-erasure-log-v1", &mut erasure_log_key)
            .map_err(|e| ErasureError::Io(e.to_string()))?;

        let ledger = if let Some(ref ledger_dir) = cfg.ledger_path {
            Some(Arc::new(
                StoreLedger::open(io.clone(), ledger_dir.clone(), ledger_key, anchor).await?,
            ))
        } else {
            None
        };

        let pending = Arc::new(
            PendingDelete::open(
                io.clone(),
                &cfg.dir,
                data_dir.clone(),
                cfg.pending_retry_limit,
            )
            .await?,
        );

        let j_open = Journal::open(
            io.clone(),
            cfg.dir.clone(),
            cfg.reserve_bytes,
            cfg.max_plan_bytes,
        )
        .await?;

        let gate = Arc::new(StoreGate::new());

        // Zustandsautomat aus den gelesenen Journal-Records rekonstruieren (§5.4.3)
        let mut jobs: BTreeMap<ErasureId, Job> = BTreeMap::new();
        let mut summary_records = Vec::new();
        let mut invalid_state_history = false;
        for rec in &j_open.records {
            if Self::apply_record_to_jobs(&mut jobs, &mut summary_records, rec).is_err() {
                invalid_state_history = true;
                break;
            }
        }

        if j_open.corrupt || invalid_state_history {
            // I-7: Unlesbare oder unzulässige Journal-Historie fail-closed behandeln.
            j_open.journal.poison();
            gate.set_fail_closed();
            *health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
            let ks = KernelState {
                jobs: BTreeMap::new(),
                completed_summaries: BTreeMap::new(),
                summary_records: Vec::new(),
                journal: j_open.journal,
            };
            let kernel = Self {
                cfg,
                health,
                clock,
                deps: OnceLock::new(),
                gate,
                pending,
                ledger,
                state: Arc::new(Mutex::new(ks)),
                status_cache: Arc::new(parking_lot::RwLock::new(BTreeMap::new())),
                open_views_cache: Arc::new(parking_lot::RwLock::new(Vec::new())),
                sealed_flags_cache: Arc::new(parking_lot::RwLock::new(BTreeMap::new())),
                global_collapse: Arc::new(parking_lot::RwLock::new(Arc::new(CollapseMap::new()))),
                erasure_log_key,
            };
            let report = BootReport {
                fail_closed: true,
                jobs_open: 0,
                torn_tail_truncated: false,
                pending_deleted: 0,
                orphans: Vec::new(),
            };
            return Ok((kernel, report));
        }

        // Temp-Dateien aus TempFile-Frames über Pending-Delete bereinigen (§5.4.7, CP-15)
        let temp_paths = collect_temp_file_paths(&data_dir, &j_open.records);
        let mut pending_deleted = pending.drain_pending().await?;
        if !temp_paths.is_empty() {
            let rm_tmp = pending.remove_after_commit(&temp_paths).await?;
            pending_deleted = pending_deleted.saturating_add(rm_tmp);
        }

        // Gate für jeden aktiven Auftrag ab GateIntent laden (§5.4.4)
        let mut gate_jobs = Vec::new();
        for job in jobs.values() {
            if job.state.gate_active() && !job.state.is_terminal() {
                gate_jobs.push(Arc::new(job.to_job_gate()));
            }
        }
        if !gate_jobs.is_empty() {
            gate.install_snapshot(|_old, ep| GateSnapshot::from_jobs(ep, gate_jobs.clone()));
        }

        let jobs_open = jobs.values().filter(|j| !j.state.is_terminal()).count() as u32;
        let ks = KernelState {
            jobs,
            completed_summaries: BTreeMap::new(),
            summary_records,
            journal: j_open.journal,
        };
        let collapse_arc = ks.rebuild_collapse_map();

        let status_map: BTreeMap<ErasureId, ErasureStatus> =
            ks.jobs.iter().map(|(id, j)| (*id, j.to_status())).collect();
        let open_views: Vec<JobView> = ks
            .jobs
            .values()
            .filter(|j| !j.state.is_terminal())
            .map(Job::to_job_view)
            .collect();
        let sealed_flags: BTreeMap<ErasureId, Arc<AtomicBool>> = ks
            .jobs
            .iter()
            .map(|(id, j)| (*id, Arc::clone(&j.sealed_flag)))
            .collect();

        let kernel = Self {
            cfg,
            health,
            clock,
            deps: OnceLock::new(),
            gate,
            pending,
            ledger,
            state: Arc::new(Mutex::new(ks)),
            status_cache: Arc::new(parking_lot::RwLock::new(status_map)),
            open_views_cache: Arc::new(parking_lot::RwLock::new(open_views)),
            sealed_flags_cache: Arc::new(parking_lot::RwLock::new(sealed_flags)),
            global_collapse: Arc::new(parking_lot::RwLock::new(collapse_arc)),
            erasure_log_key,
        };

        let report = BootReport {
            fail_closed: false,
            jobs_open,
            torn_tail_truncated: j_open.torn_tail_truncated,
            pending_deleted,
            orphans: Vec::new(),
        };
        Ok((kernel, report))
    }

    pub(crate) fn attach_deps(&self, deps: StoreDeps) {
        if self.deps.set(deps).is_err() {
            tracing::warn!("erasure kernel store deps already attached");
        }
    }

    fn deps_ref(&self) -> Result<&StoreDeps, ErasureError> {
        self.deps
            .get()
            .ok_or_else(|| ErasureError::Io("erasure kernel store deps not yet attached".into()))
    }

    fn apply_record_to_jobs(
        jobs: &mut BTreeMap<ErasureId, Job>,
        summary_records: &mut Vec<JournalRecord>,
        rec: &JournalRecord,
    ) -> Result<(), ErasureError> {
        match rec {
            JournalRecord::Requested {
                id,
                target_hash: _,
                reason: _,
                policy,
                ts,
            } => {
                if jobs.contains_key(id) {
                    return Err(ErasureError::InvalidTransition);
                }
                let job = Job {
                    id: *id,
                    state: ErasureState::Requested,
                    plan: Arc::new(ResolvedScope {
                        id: *id,
                        target_kind: TargetKind::Subject,
                        keys: contextra_ports::erasure::KeyScope::default(),
                        docs: Vec::new(),
                        plan_hash: [0u8; 32],
                        unprovenanced: 0,
                    }),
                    policy: policy.clone(),
                    expected_layers: Vec::new(),
                    gated_seq: None,
                    layer_cursor: BTreeMap::new(),
                    layer_attempts: BTreeMap::new(),
                    layer_remaining: BTreeMap::new(),
                    collapse: CollapseMap::new(),
                    attempt: 0,
                    guarantee_reached: Guarantee::NONE,
                    exceptions: Vec::new(),
                    holds: 0,
                    requested_at: *ts,
                    gated_at: None,
                    sealed_at: None,
                    ledger_index: None,
                    purge_started: false,
                    keys_destroyed: false,
                    sealed_flag: Arc::new(AtomicBool::new(false)),
                    raw_records: vec![rec.clone()],
                };
                jobs.insert(*id, job);
            }
            JournalRecord::Planned {
                id,
                expected_layers,
                plan_blob,
                ..
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    let scope = decode_resolved_scope(plan_blob)?;
                    job.transition_to(ErasureState::Planned)?;
                    job.plan = Arc::new(scope);
                    job.expected_layers = expected_layers.clone();
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::GateIntent { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Gating)?;
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::Gated {
                id, gated_seq, ts, ..
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Gated)?;
                    job.gated_seq = Some(Seq(*gated_seq));
                    job.gated_at = Some(*ts);
                    job.guarantee_reached = Guarantee {
                        gated: true,
                        purged: false,
                        crypto_erased: false,
                    };
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::KeysDestroying { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::KeysDestroying)?;
                    job.keys_destroyed = true;
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::PurgeStart { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Purging)?;
                    job.purge_started = true;
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::LayerProgress {
                id,
                layer,
                cursor,
                units_done,
                bytes_freed,
                attempt,
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    let c = if cursor.is_empty() {
                        None
                    } else {
                        Some(Cursor::new(cursor.clone())?)
                    };
                    job.layer_cursor.insert(
                        layer_map_key(layer),
                        LayerProgress {
                            cursor: c,
                            units_done: *units_done,
                            bytes_freed: *bytes_freed,
                        },
                    );
                    job.layer_attempts.insert(layer_map_key(layer), *attempt);
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::StoreProgress { id, .. } | JournalRecord::TempFile { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::CollapseKeys { id, entries } => {
                if let Some(job) = jobs.get_mut(id) {
                    for (k, keep_from) in entries {
                        job.collapse
                            .entry(k.clone())
                            .and_modify(|s| {
                                if Seq(*keep_from) > *s {
                                    *s = Seq(*keep_from);
                                }
                            })
                            .or_insert(Seq(*keep_from));
                    }
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::Verified { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Verifying)?;
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::Sealed {
                id,
                ledger_index,
                guarantee,
                exceptions,
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Sealed)?;
                    job.sealed_flag.store(true, Ordering::Release);
                    job.ledger_index = if *ledger_index >= 0 {
                        Some(*ledger_index as u64)
                    } else {
                        None
                    };
                    job.guarantee_reached = Guarantee::new(
                        (guarantee & 1) != 0,
                        (guarantee & 2) != 0,
                        (guarantee & 4) != 0,
                    )
                    .unwrap_or(Guarantee::NONE);
                    job.exceptions = exceptions.clone();
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::Completed { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Completed)?;
                    job.sealed_flag.store(true, Ordering::Release);
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::Failed {
                id, layer, attempt, ..
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Failed {
                        layer: layer.as_ref().map(|l| l.as_str().to_string()),
                        attempt: *attempt,
                    })?;
                    job.attempt = *attempt;
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::Quarantined { id, layer } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.transition_to(ErasureState::Quarantined {
                        layer: layer.as_str().to_string(),
                    })?;
                    job.raw_records.push(rec.clone());
                } else {
                    return Err(ErasureError::InvalidTransition);
                }
            }
            JournalRecord::Summary { .. } => {
                summary_records.push(rec.clone());
            }
        }
        Ok(())
    }

    /// Nach dem WAL-Replay + Start-Flush: für Aufträge mit `GateIntent` ohne `Gated`
    /// das `Gated{gated_seq = last_applied_seq}` nachtragen (§5.4.6, CP-3/CP-4).
    pub(crate) async fn finalize_boot_gates(
        &self,
        last_applied_seq: u64,
    ) -> Result<(), ErasureError> {
        if self.gate.is_fail_closed() {
            return Ok(());
        }
        let mut ks = self.state.lock().await;
        let now = self.clock.read().now_unix_nanos();
        let mut to_append = Vec::new();
        let mut updated_ids = Vec::new();

        for (id, job) in ks.jobs.iter() {
            if matches!(job.state, ErasureState::Gating) && job.gated_seq.is_none() {
                let next_ep = self.gate.epoch().saturating_add(1);
                to_append.push(JournalRecord::Gated {
                    id: *id,
                    gated_seq: last_applied_seq,
                    epoch: next_ep,
                    ts: now,
                });
                updated_ids.push(*id);
            }
        }

        if !to_append.is_empty() {
            if let Err(e) = ks.journal.append_batch(&to_append).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }
            for (idx, id) in updated_ids.iter().enumerate() {
                if let Some(rec) = to_append.get(idx) {
                    if let Some(job) = ks.jobs.get_mut(id) {
                        job.gated_seq = Some(Seq(last_applied_seq));
                        job.gated_at = Some(now);
                        job.transition_to(ErasureState::Gated)?;
                        job.guarantee_reached = Guarantee {
                            gated: true,
                            purged: false,
                            crypto_erased: false,
                        };
                        job.raw_records.push(rec.clone());
                        let jg = job.to_job_gate();
                        self.gate
                            .install_snapshot(|s, ep| s.with_upserted_job(ep, jg.clone()));
                    }
                }
            }
            self.refresh_caches(&ks);
        }
        Ok(())
    }

    pub(crate) fn gate_ref(&self) -> &Arc<StoreGate> {
        &self.gate
    }

    pub(crate) fn pending_ref(&self) -> &Arc<PendingDelete> {
        &self.pending
    }

    pub(crate) fn collapse_snapshot(&self) -> Arc<CollapseMap> {
        Arc::clone(&self.global_collapse.read())
    }

    pub(crate) fn global_collapse_handle(&self) -> Arc<parking_lot::RwLock<Arc<CollapseMap>>> {
        Arc::clone(&self.global_collapse)
    }

    /// Liefert ein `Notify`-Handle, das benachrichtigt wird, wenn Pending-Delete-Einträge verbleiben (§8).
    ///
    /// # Fehler
    /// Keine; reiner Handle-Zugriff.
    /// # Crash
    /// Keine Seiteneffekte.
    pub fn pending_delete_notify(&self) -> Arc<tokio::sync::Notify> {
        self.pending.notify_handle()
    }

    /// Liefert die aktuelle Anzahl ausstehender Pending-Delete-Pfade (§8).
    ///
    /// # Fehler
    /// Keine; atomarer Lesezugriff (`Acquire`).
    /// # Crash
    /// Keine Seiteneffekte.
    pub fn pending_delete_remaining(&self) -> u64 {
        self.pending.remaining()
    }

    /// Registriert Version-Collapse-Einträge (A-1) durabel im Journal und RAM (§7.2).
    ///
    /// # Fehler
    /// Liefert [`ErasureError::JournalPoisoned`], [`ErasureError::NotFound`] oder
    /// [`ErasureError::InvalidTransition`] bzw. [`ErasureError::Io`] bei Schreibfehlern.
    /// # Crash
    /// Write-Ahead-Protokollierung; unvollständige Batches bleiben ohne RAM-Übernahme.
    pub async fn register_collapse(
        &self,
        id: ErasureId,
        entries: Vec<(Vec<u8>, u64)>,
    ) -> Result<(), ErasureError> {
        if entries.is_empty() {
            return Ok(());
        }
        let mut ks = self.state.lock().await;
        if ks.journal.is_poisoned() {
            return Err(ErasureError::JournalPoisoned);
        }
        let job = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
        if job.state.is_terminal() {
            return Err(ErasureError::InvalidTransition);
        }

        for chunk in entries.chunks(1024) {
            let rec = JournalRecord::CollapseKeys {
                id,
                entries: chunk.to_vec(),
            };
            if let Err(e) = ks.journal.append_batch(&[rec.clone()]).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }
            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                for (k, keep_from) in chunk {
                    job_mut
                        .collapse
                        .entry(k.clone())
                        .and_modify(|s| {
                            if Seq(*keep_from) > *s {
                                *s = Seq(*keep_from);
                            }
                        })
                        .or_insert(Seq(*keep_from));
                }
                job_mut.raw_records.push(rec);
            }
        }
        let new_collapse = ks.rebuild_collapse_map();
        *self.global_collapse.write() = new_collapse;
        Ok(())
    }

    fn refresh_caches(&self, ks: &KernelState) {
        let mut status_map = self.status_cache.write();
        *status_map = ks.completed_summaries.clone();
        for (id, job) in &ks.jobs {
            status_map.insert(*id, job.to_status());
        }
        let views: Vec<JobView> = ks
            .jobs
            .values()
            .filter(|j| !j.state.is_terminal())
            .map(Job::to_job_view)
            .collect();
        *self.open_views_cache.write() = views;
        let flags: BTreeMap<ErasureId, Arc<AtomicBool>> = ks
            .jobs
            .iter()
            .map(|(id, j)| (*id, Arc::clone(&j.sealed_flag)))
            .collect();
        *self.sealed_flags_cache.write() = flags;
    }

    fn default_expected_layers() -> Vec<WitnessLayer> {
        vec![
            WitnessLayer::LsmMemtable,
            WitnessLayer::SsTableAllLevels,
            WitnessLayer::WalAllSegments,
        ]
    }
}

use crate::erasure::gate::GateSnapshot;

impl ErasureKernel for StoreErasureKernel {
    fn accept<'a>(
        &'a self,
        req: &'a EraseRequest,
        plan: &'a ResolvedScope,
    ) -> BoxFuture<'a, Result<AcceptReceipt, ErasureError>> {
        Box::pin(async move {
            req.target.validate()?;
            if let Some(ref ledger) = self.ledger {
                if ledger.is_corrupt() {
                    return Err(ErasureError::LedgerCorrupt);
                }
            }

            let mut ks = self.state.lock().await;
            if ks.journal.is_poisoned() {
                return Err(ErasureError::JournalPoisoned);
            }

            // (1) Idempotenzprüfung
            if let Some(existing) = ks.jobs.get(&req.id) {
                if existing.plan.plan_hash == plan.plan_hash {
                    return Ok(AcceptReceipt {
                        id: req.id,
                        existing: true,
                    });
                }
                return Err(ErasureError::IdempotencyConflict);
            }

            // (2) max_open_jobs
            if ks.open_job_count() >= self.cfg.max_open_jobs as usize {
                return Err(ErasureError::Busy {
                    active: ks.open_job_count() as u32,
                });
            }

            // (3) Plan-Größe <= max_plan_bytes
            let plan_blob = encode_resolved_scope(plan)?;
            if plan_blob.len() > self.cfg.max_plan_bytes as usize {
                return Err(ErasureError::InvalidTarget(
                    "resolved plan exceeds max_plan_bytes",
                ));
            }

            let expected_layers = Self::default_expected_layers();
            let now = self.clock.read().now_unix_nanos();
            let target_hash = *blake3::hash(&req.target.canonical_bytes()).as_bytes();

            let r_req = JournalRecord::Requested {
                id: req.id,
                target_hash,
                reason: req.reason,
                policy: req.policy.clone(),
                ts: now,
            };
            let r_plan = JournalRecord::Planned {
                id: req.id,
                plan_hash: plan.plan_hash,
                expected_layers: expected_layers.clone(),
                plan_blob,
                unprovenanced: plan.unprovenanced,
            };

            let est_bytes = (r_req.encode_frame(self.cfg.max_plan_bytes)?.len()
                + r_plan.encode_frame(self.cfg.max_plan_bytes)?.len())
                as u64;

            // (4) Platzbedarf prüfen
            if !ks
                .journal
                .has_space_for(est_bytes.saturating_add(self.cfg.reserve_bytes))
            {
                return Err(ErasureError::InsufficientSpace {
                    needed: est_bytes.saturating_add(self.cfg.reserve_bytes),
                    available: 0,
                });
            }

            // (5) Requested + Planned in einem append_batch + sync_data
            if let Err(e) = ks
                .journal
                .append_batch(&[r_req.clone(), r_plan.clone()])
                .await
            {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            let mut job = Job {
                id: req.id,
                state: ErasureState::Requested,
                plan: Arc::new(plan.clone()),
                policy: req.policy.clone(),
                expected_layers,
                gated_seq: None,
                layer_cursor: BTreeMap::new(),
                layer_attempts: BTreeMap::new(),
                layer_remaining: BTreeMap::new(),
                collapse: CollapseMap::new(),
                attempt: 0,
                guarantee_reached: Guarantee::NONE,
                exceptions: Vec::new(),
                holds: 0,
                requested_at: now,
                gated_at: None,
                sealed_at: None,
                ledger_index: None,
                purge_started: false,
                keys_destroyed: false,
                sealed_flag: Arc::new(AtomicBool::new(false)),
                raw_records: vec![r_req, r_plan],
            };
            job.transition_to(ErasureState::Planned)?;
            ks.jobs.insert(req.id, job);
            self.refresh_caches(&ks);

            Ok(AcceptReceipt {
                id: req.id,
                existing: false,
            })
        })
    }

    fn publish_gate<'a>(
        &'a self,
        id: ErasureId,
    ) -> BoxFuture<'a, Result<GateReceipt, ErasureError>> {
        Box::pin(async move {
            let mut ks = self.state.lock().await;
            if ks.journal.is_poisoned() {
                return Err(ErasureError::JournalPoisoned);
            }

            let job = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
            if !matches!(job.state, ErasureState::Planned | ErasureState::Gating) {
                return Err(ErasureError::InvalidTransition);
            }
            if !job.state.can_transition_to(&ErasureState::Gating)
                && !matches!(job.state, ErasureState::Gating)
            {
                return Err(ErasureError::InvalidTransition);
            }

            let deps = self.deps_ref()?;
            let now = self.clock.read().now_unix_nanos();
            if matches!(&job.state, ErasureState::Planned) {
                let intent_rec = JournalRecord::GateIntent { id, ts: now };
                if let Err(e) = ks.journal.append_batch(&[intent_rec.clone()]).await {
                    *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                    return Err(e);
                }

                if let Some(job_mut) = ks.jobs.get_mut(&id) {
                    job_mut.transition_to(ErasureState::Gating)?;
                    job_mut.raw_records.push(intent_rec);
                }
            }

            // Protokoll §6.2 Schritte 4..10 unter commit_mutex (Rang 200) -> snap (Rang 960)
            let (gated_seq, epoch, target_kind, docs) = {
                let _commit_lock = deps.commit_mutex.lock().await;
                let seq_val = deps.next_seq_no.load(Ordering::SeqCst).saturating_sub(1);
                let job_ref = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
                let mut jg = job_ref.to_job_gate();
                jg.up_to = Seq(seq_val);
                jg.phase = GatePhase::Restricted;
                let ep = self
                    .gate
                    .install_snapshot(|s, new_ep| s.with_upserted_job(new_ep, jg.clone()));
                (
                    seq_val,
                    ep,
                    job_ref.plan.target_kind,
                    job_ref.plan.docs.clone(),
                )
            };

            let gated_rec = JournalRecord::Gated {
                id,
                gated_seq,
                epoch,
                ts: now,
            };
            if let Err(e) = ks.journal.append_batch(&[gated_rec.clone()]).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.gated_seq = Some(Seq(gated_seq));
                job_mut.gated_at = Some(now);
                job_mut.transition_to(ErasureState::Gated)?;
                job_mut.guarantee_reached = Guarantee {
                    gated: true,
                    purged: false,
                    crypto_erased: false,
                };
                job_mut.raw_records.push(gated_rec);
            }

            // Schritt 12: Snapshot-Entzug nur für Tenant-Aufträge (§6.2, K-21)
            if target_kind == TargetKind::Tenant {
                for ds in docs {
                    if let DocScope::Tenant { tenant } = ds {
                        let revoked = deps.snapshot_registry.revoke_covering(
                            tenant,
                            0..gated_seq.saturating_add(1),
                            gated_seq,
                        );
                        tracing::debug!(%tenant, revoked, "revoked tenant snapshots on gate publish");
                    }
                }
            }

            self.refresh_caches(&ks);

            Ok(GateReceipt {
                id,
                gated_seq: Seq(gated_seq),
                epoch,
            })
        })
    }

    fn begin_purge<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move { self.begin_purge_inner(id).await.map(|_| ()) })
    }

    fn purge_store<'a>(
        &'a self,
        id: ErasureId,
        cx: &'a PurgeCx<'a>,
    ) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>> {
        Box::pin(async move {
            if cx.cancel.is_cancelled() {
                return Err(ErasureError::Cancelled);
            }

            let (scope, attempt) = self.begin_purge_inner(id).await?;

            let deps = self.deps_ref()?;
            let collapse = self.collapse_snapshot();
            let filter = build_filter(&self.gate, &collapse);

            // Phase 1: MemTable-Retain (§7.1)
            let (mem_dropped, mem_freed) = purge_memtables(deps, &filter).await;

            // Phase 2: Erase-Compaction über SSTables (§7.3)
            let (next_cursor, sst_units, sst_freed) = erase_compact_tick(
                deps,
                id,
                &scope,
                &filter,
                &collapse,
                cx,
                &self.pending,
                self.cfg.reserve_bytes,
                attempt,
                &self.state,
            )
            .await?;

            // Phase 3: WAL-Schnitt nur, wenn alle SSTables durch sind (§7.4)
            let (wal_dropped, wal_freed) = if next_cursor.is_none() {
                erase_wal_cut(deps, &filter, &self.pending).await?
            } else {
                (0, 0)
            };

            let total_units = mem_dropped
                .saturating_add(sst_units)
                .saturating_add(wal_dropped);
            let total_freed = mem_freed
                .saturating_add(sst_freed)
                .saturating_add(wal_freed);

            Ok(PurgeOutcome {
                progress: LayerProgress {
                    cursor: next_cursor,
                    units_done: total_units,
                    bytes_freed: total_freed,
                },
                units_committed: total_units,
                bytes_freed: total_freed,
            })
        })
    }

    fn record_layer<'a>(
        &'a self,
        id: ErasureId,
        layer: &'a WitnessLayer,
        p: &'a LayerProgress,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            let mut ks = self.state.lock().await;
            if ks.journal.is_poisoned() {
                return Err(ErasureError::JournalPoisoned);
            }
            let job = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
            if !matches!(&job.state, ErasureState::Purging) {
                return Err(ErasureError::InvalidTransition);
            }
            let attempt = job
                .layer_attempts
                .get(&layer_map_key(layer))
                .copied()
                .unwrap_or(0);
            let cursor_bytes = p
                .cursor
                .as_ref()
                .map_or_else(Vec::new, |c| c.as_bytes().to_vec());
            let rec = JournalRecord::LayerProgress {
                id,
                layer: layer.clone(),
                cursor: cursor_bytes,
                units_done: p.units_done,
                bytes_freed: p.bytes_freed,
                attempt,
            };
            if let Err(e) = ks.journal.append_batch(&[rec.clone()]).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }
            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.layer_cursor.insert(layer_map_key(layer), p.clone());
                job_mut.raw_records.push(rec);
            }
            self.refresh_caches(&ks);
            Ok(())
        })
    }

    fn seal<'a>(
        &'a self,
        id: ErasureId,
        w: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<SealReceipt, ErasureError>> {
        Box::pin(async move {
            // Vorprüfung & Daten aus KernelState extrahieren, ohne Rang 150 über Ledger-Append zu halten?
            // Ledger-Append benötigt fsync auf externer Datei, daher zuerst Zustand prüfen:
            let (
                expected_layers,
                plan_hash,
                unprovenanced,
                gated_seq,
                keys_destroyed,
                holds,
                attempt,
                current_state,
            ) = {
                let ks = self.state.lock().await;
                if ks.journal.is_poisoned() {
                    return Err(ErasureError::JournalPoisoned);
                }
                let job = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
                if !matches!(job.state, ErasureState::Purging | ErasureState::Verifying) {
                    return Err(ErasureError::InvalidTransition);
                }
                (
                    job.expected_layers.clone(),
                    job.plan.plan_hash,
                    job.plan.unprovenanced,
                    job.gated_seq,
                    job.keys_destroyed,
                    job.holds,
                    job.attempt,
                    job.state.clone(),
                )
            };

            let mut witness_by_layer: BTreeMap<(u8, String), &LayerWitness> = BTreeMap::new();
            for wit in w {
                let (tag, name) = layer_sort_key(wit.witness_layer());
                witness_by_layer.insert((tag, name.to_string()), wit);
            }

            let mut exceptions = vec![ErasureException::OutOfScopeExternalCopies];
            if unprovenanced > 0 {
                exceptions.push(ErasureException::Unprovenanced {
                    objects: unprovenanced,
                });
            }
            if holds > 0 {
                exceptions.push(ErasureException::LegalHold {
                    checkpoint: "active".to_string(),
                });
            }

            let mut all_zero_remaining = true;
            let mut missing_or_disabled = false;
            let mut remaining_map: BTreeMap<(u8, String), u64> = BTreeMap::new();

            let mut sorted_expected = expected_layers.clone();
            sorted_expected.sort_by(|a, b| layer_sort_key(a).cmp(&layer_sort_key(b)));

            let mut root_hasher = blake3::Hasher::new();
            root_hasher.update(b"ctx-layers-v1");

            for layer in &sorted_expected {
                let (tag, name) = layer_sort_key(layer);
                if let Some(wit) = witness_by_layer.get(&(tag, name.to_string())) {
                    root_hasher.update(&[tag]);
                    root_hasher.update(wit.digest());
                    remaining_map.insert(layer_map_key(layer), wit.remaining());
                    if wit.remaining() > 0 {
                        all_zero_remaining = false;
                    }
                    match wit.technique() {
                        ScanTechnique::NotMeasured(_) => {
                            missing_or_disabled = true;
                            exceptions.push(ErasureException::NotMeasured {
                                layer: layer.as_str().to_string(),
                            });
                        }
                        ScanTechnique::FeatureDisabled => {
                            missing_or_disabled = true;
                            exceptions.push(ErasureException::FeatureDisabled {
                                layer: layer.as_str().to_string(),
                            });
                        }
                        _ => {}
                    }
                } else {
                    missing_or_disabled = true;
                    all_zero_remaining = false;
                    exceptions.push(ErasureException::NotMeasured {
                        layer: layer.as_str().to_string(),
                    });
                }
            }

            let deps = self.deps_ref()?;
            if let Some(gs) = gated_seq {
                if let Some(min_snap) = deps.snapshot_registry.min_read_snapshot() {
                    if min_snap <= gs.0 {
                        exceptions.push(ErasureException::OverlappingSnapshotPinned {
                            min_seq: min_snap,
                        });
                    }
                }
            }

            let layers_root = *root_hasher.finalize().as_bytes();
            let gated = gated_seq.is_some();
            let purged = gated && all_zero_remaining && !missing_or_disabled;
            let crypto_erased = gated && keys_destroyed;
            let guarantee = Guarantee::new(gated, purged, crypto_erased)?;

            let now = self.clock.read().now_unix_nanos();
            let id_hmac = contextra_crypto::ledger_mac(&self.erasure_log_key, &id.0)
                .map_err(|e| ErasureError::Io(e.to_string()))?;

            // (5) Zuerst Ledger-Append, danach Sealed ins Journal (§11.2.5, CP-10)
            let ledger_index = if let Some(ref ledger) = self.ledger {
                let draft = LedgerDraft {
                    erasure_id_hmac: id_hmac,
                    guarantee,
                    layers_root,
                    exceptions: exceptions.clone(),
                    plan_hash,
                    policy_version: 1,
                    completed_at: Timestamp::new(now / 1_000),
                };
                let entry = ledger.append(draft).await?;
                if let Ok(head) = ledger.head().await {
                    ledger.publish_anchor_best_effort(head);
                }
                Some(entry.index)
            } else {
                None
            };

            let mut g_byte = 0u8;
            if guarantee.gated {
                g_byte |= 1;
            }
            if guarantee.purged {
                g_byte |= 2;
            }
            if guarantee.crypto_erased {
                g_byte |= 4;
            }

            let r_ver = JournalRecord::Verified {
                id,
                witness_root: layers_root,
                attempt,
            };
            let r_seal = JournalRecord::Sealed {
                id,
                ledger_index: ledger_index.map(|i| i as i64).unwrap_or(-1),
                guarantee: g_byte,
                exceptions: exceptions.clone(),
            };

            let mut ks = self.state.lock().await;
            if ks.journal.is_poisoned() {
                return Err(ErasureError::JournalPoisoned);
            }
            if ks.jobs.get(&id).map(|job| &job.state) != Some(&current_state) {
                return Err(ErasureError::InvalidTransition);
            }
            let records = if matches!(current_state, ErasureState::Purging) {
                vec![r_ver.clone(), r_seal.clone()]
            } else {
                vec![r_seal.clone()]
            };
            if let Err(e) = ks.journal.append_batch(&records).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.transition_to(ErasureState::Sealed)?;
                job_mut.sealed_at = Some(now);
                job_mut.sealed_flag.store(true, Ordering::Release);
                job_mut.ledger_index = ledger_index;
                job_mut.guarantee_reached = guarantee;
                job_mut.exceptions = exceptions.clone();
                job_mut.layer_remaining = remaining_map;
                job_mut.raw_records.extend(records);
                let jg = job_mut.to_job_gate();
                self.gate
                    .install_snapshot(|s, ep| s.with_upserted_job(ep, jg.clone()));
            }
            self.refresh_caches(&ks);

            Ok(SealReceipt {
                id,
                guarantee,
                exceptions,
                ledger_index,
            })
        })
    }

    fn complete<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            let deps = self.deps_ref()?;
            let mut ks = self.state.lock().await;
            if ks.journal.is_poisoned() {
                return Err(ErasureError::JournalPoisoned);
            }
            let job = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
            if !matches!(job.state, ErasureState::Sealed) {
                return Err(ErasureError::InvalidTransition);
            }

            // §6.5 Freigabebedingungen (I-9)
            // 1. Alle erwarteten Layer remaining == 0
            for layer in &job.expected_layers {
                match job.layer_remaining.get(&layer_map_key(layer)) {
                    Some(0) => {}
                    _ => return Err(ErasureError::InvalidTransition),
                }
            }
            // 2. Kein LegalHold
            if job.holds > 0
                || job
                    .exceptions
                    .iter()
                    .any(|e| matches!(e, ErasureException::LegalHold { .. }))
            {
                return Err(ErasureError::InvalidTransition);
            }
            // 3. Snapshot-Freigabe: min_read_snapshot > gated_seq oder OverlappingSnapshotPinned
            if let Some(gs) = job.gated_seq {
                if let Some(min_snap) = deps.snapshot_registry.min_read_snapshot() {
                    let has_pinned_exc = job
                        .exceptions
                        .iter()
                        .any(|e| matches!(e, ErasureException::OverlappingSnapshotPinned { .. }));
                    if min_snap <= gs.0 && !has_pinned_exc {
                        return Err(ErasureError::InvalidTransition);
                    }
                }
            }

            let now = self.clock.read().now_unix_nanos();
            let comp_rec = JournalRecord::Completed { id, ts: now };
            if let Err(e) = ks.journal.append_batch(&[comp_rec.clone()]).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.transition_to(ErasureState::Completed)?;
                job_mut.raw_records.push(comp_rec);
            }

            // Gate-Eintrag entfernen und Epoche erhöhen (§4.2)
            self.gate.install_snapshot(|s, ep| s.without_job(ep, id));

            // Verdichtung prüfen (§5.5)
            let should_compact = ks.journal.offset() > JOURNAL_COMPACT_SIZE_THRESHOLD
                || ks.journal.completed_since_compact() > JOURNAL_COMPACT_COMPLETED_THRESHOLD;
            if should_compact {
                let mut compact_records = Vec::new();
                let mut completed_ids = Vec::new();
                let mut new_summaries = Vec::new();
                let mut new_completed = Vec::new();
                for (jid, j) in &ks.jobs {
                    if j.state.is_terminal() {
                        let id_hmac = contextra_crypto::ledger_mac(&self.erasure_log_key, &jid.0)
                            .map_err(|e| ErasureError::Io(e.to_string()))?;
                        let plan_hmac =
                            contextra_crypto::ledger_mac(&self.erasure_log_key, &j.plan.plan_hash)
                                .map_err(|e| ErasureError::Io(e.to_string()))?;
                        let mut g = 0u8;
                        if j.guarantee_reached.gated {
                            g |= 1;
                        }
                        if j.guarantee_reached.purged {
                            g |= 2;
                        }
                        if j.guarantee_reached.crypto_erased {
                            g |= 4;
                        }
                        let sum = JournalRecord::Summary {
                            id_hmac,
                            plan_hmac,
                            guarantee: g,
                            completed_at: now,
                        };
                        new_summaries.push(sum);
                        new_completed.push((*jid, j.to_status()));
                        completed_ids.push(*jid);
                    } else {
                        compact_records.extend(j.raw_records.clone());
                    }
                }
                ks.summary_records.extend(new_summaries);
                for (jid, st) in new_completed {
                    ks.completed_summaries.insert(jid, st);
                }
                for cid in completed_ids {
                    ks.jobs.remove(&cid);
                }
                compact_records.extend(ks.summary_records.clone());
                if let Err(e) = ks.journal.compact(&compact_records).await {
                    tracing::warn!(error = %e, "erasure journal compaction failed");
                }
            }

            let new_collapse = ks.rebuild_collapse_map();
            *self.global_collapse.write() = new_collapse;
            self.refresh_caches(&ks);
            Ok(())
        })
    }

    fn fail<'a>(
        &'a self,
        id: ErasureId,
        layer: Option<&'a WitnessLayer>,
        kind: LayerErrorKind,
    ) -> BoxFuture<'a, Result<ErasureState, ErasureError>> {
        Box::pin(async move {
            let mut ks = self.state.lock().await;
            if ks.journal.is_poisoned() {
                return Err(ErasureError::JournalPoisoned);
            }
            let job = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
            if job.state.is_terminal() {
                return Err(ErasureError::InvalidTransition);
            }

            let next_attempt = job.attempt.saturating_add(1);
            let failed_state = ErasureState::Failed {
                layer: layer.map(|l| l.as_str().to_string()),
                attempt: next_attempt,
            };
            if !job.state.can_transition_to(&failed_state) {
                return Err(ErasureError::InvalidTransition);
            }
            let new_state = if next_attempt >= 10 {
                if let Some(l) = layer {
                    ErasureState::Quarantined {
                        layer: l.as_str().to_string(),
                    }
                } else {
                    ErasureState::Failed {
                        layer: None,
                        attempt: next_attempt,
                    }
                }
            } else {
                ErasureState::Failed {
                    layer: layer.map(|l| l.as_str().to_string()),
                    attempt: next_attempt,
                }
            };

            let failed_rec = JournalRecord::Failed {
                id,
                layer: layer.cloned(),
                kind,
                attempt: next_attempt,
            };
            let mut records = vec![failed_rec.clone()];
            if let (true, Some(l)) = (next_attempt >= 10, layer) {
                let quarantined = ErasureState::Quarantined {
                    layer: l.as_str().to_string(),
                };
                if !failed_state.can_transition_to(&quarantined) {
                    return Err(ErasureError::InvalidTransition);
                }
                records.push(JournalRecord::Quarantined {
                    id,
                    layer: l.clone(),
                });
            }

            if let Err(e) = ks.journal.append_batch(&records).await {
                *self.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.attempt = next_attempt;
                job_mut.transition_to(failed_state)?;
                if matches!(&new_state, ErasureState::Quarantined { .. }) {
                    job_mut.transition_to(new_state.clone())?;
                }
                if let Some(l) = layer {
                    job_mut
                        .layer_attempts
                        .insert(layer_map_key(l), next_attempt);
                }
                job_mut.raw_records.extend(records);
            }
            self.refresh_caches(&ks);
            Ok(new_state)
        })
    }

    fn open_jobs(&self) -> Vec<JobView> {
        self.open_views_cache.read().clone()
    }

    fn status(&self, id: ErasureId) -> Option<ErasureStatus> {
        self.status_cache.read().get(&id).cloned()
    }

    fn privileged_reader(&self, id: ErasureId) -> Result<Arc<dyn PrivilegedRead>, ErasureError> {
        let deps = self.deps_ref()?.clone();
        let views = self.open_views_cache.read();
        let view = views
            .iter()
            .find(|v| v.id == id)
            .ok_or(ErasureError::NotFound)?;
        if !matches!(
            view.state,
            ErasureState::Gated
                | ErasureState::KeysDestroying
                | ErasureState::Purging
                | ErasureState::Verifying
        ) {
            return Err(ErasureError::InvalidTransition);
        }
        let sealed_flag = self
            .sealed_flags_cache
            .read()
            .get(&id)
            .cloned()
            .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
        Ok(Arc::new(StorePrivilegedReader::new(
            deps,
            Arc::clone(&view.scope),
            sealed_flag,
        )))
    }

    fn gate(&self) -> Arc<dyn ReadGate> {
        Arc::clone(&self.gate) as Arc<dyn ReadGate>
    }

    fn store_scanners(&self) -> Vec<Arc<dyn LayerScanner>> {
        let Some(deps) = self.deps.get().cloned() else {
            return Vec::new();
        };
        vec![
            Arc::new(KernelStoreScanner::new(
                WitnessLayer::LsmMemtable,
                deps.clone(),
            )),
            Arc::new(KernelStoreScanner::new(
                WitnessLayer::SsTableAllLevels,
                deps.clone(),
            )),
            Arc::new(KernelStoreScanner::new(WitnessLayer::WalAllSegments, deps)),
        ]
    }
}
