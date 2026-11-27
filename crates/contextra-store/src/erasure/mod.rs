//! # Erasure-Kernel (`contextra-store`, Ring 1)
//!
//! Implementiert das nicht umgehbare Lösch-Subsystem im primären Speicher:
//! - Durables Auftragsjournal (`journal.log`) mit Pre-Allokation, CRC32C und Vergiftungsschutz
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
    decode_resolved_scope, encode_resolved_scope, encode_reason, layer_sort_key, Journal,
    JournalRecord,
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
    Cursor, ErasureError, ErasureException, ErasureId, ErasurePolicy, ErasureReason,
    ErasureState, ErasureStatus, Guarantee, LayerErrorKind, LayerProgress, LayerState,
    LayerStatus, Seq, TargetKind, TenantId, Timestamp,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct BootReport {
    /// `true`, wenn das Journal korrupt war und der Store im Fail-Closed-Modus gestartet ist.
    pub fail_closed: bool,
    /// Anzahl offener (nicht-terminaler) Aufträge nach Recovery.
    pub jobs_open: u32,
    /// `true`, wenn beim Boot ein unvollständiges Tail-Frame abgeschnitten wurde (CP-13).
    pub torn_tail_truncated: bool,
    /// Anzahl der beim Boot über Pending-Delete entfernten Dateien.
    pub pending_deleted: u32,
}

#[derive(Clone)]
pub(crate) struct Job {
    pub id: ErasureId,
    pub state: ErasureState,
    pub plan: Arc<ResolvedScope>,
    pub target_hash: [u8; 32],
    pub reason: ErasureReason,
    pub policy: ErasurePolicy,
    pub expected_layers: Vec<WitnessLayer>,
    pub gated_seq: Option<Seq>,
    pub layer_cursor: BTreeMap<WitnessLayer, LayerProgress>,
    pub layer_attempts: BTreeMap<WitnessLayer, u32>,
    pub layer_remaining: BTreeMap<WitnessLayer, u64>,
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
    fn to_job_gate(&self) -> JobGate {
        JobGate {
            id: self.id,
            scope: Arc::new(self.plan.keys.clone()),
            docs: self.plan.docs.clone(),
            up_to: self.gated_seq.unwrap_or(Seq::MAX),
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
            let progress = self.layer_cursor.get(layer);
            let remaining = self.layer_remaining.get(layer).copied();
            let attempt = self.layer_attempts.get(layer).copied().unwrap_or(0);
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
                ErasureState::Quarantined {
                    layer: q_layer,
                } => {
                    if q_layer == layer.as_str() {
                        LayerState::Quarantined
                    } else {
                        LayerState::Purging
                    }
                }
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
            if let Some(p) = self.layer_cursor.get(layer) {
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
    deps: StoreDeps,
    gate: Arc<StoreGate>,
    pending: Arc<PendingDelete>,
    ledger: Option<Arc<StoreLedger>>,
    state: Arc<Mutex<KernelState>>,
    /// Synchroner Snapshot für `open_jobs()` und `status()` ohne `.await` auf Rang 150.
    status_cache: Arc<parking_lot::RwLock<BTreeMap<ErasureId, ErasureStatus>>>,
    open_views_cache: Arc<parking_lot::RwLock<Vec<JobView>>>,
    global_collapse: Arc<parking_lot::RwLock<Arc<CollapseMap>>>,
    erasure_log_key: [u8; 32],
}

impl StoreErasureKernel {
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
    pub(crate) async fn open(
        cfg: StoreErasureConfig,
        deps: StoreDeps,
        key_source: &dyn KeyMaterialSource,
        anchor: Option<Arc<dyn LedgerAnchor>>,
    ) -> Result<(Self, BootReport), ErasureError> {
        if cfg.require_external_ledger {
            let Some(ref lp) = cfg.ledger_path else {
                return Err(ErasureError::InvalidTarget(
                    "require_external_ledger requires ledger_path to be set",
                ));
            };
            validate_external_ledger_path(lp, &deps.data_dir)?;
        } else if let Some(ref lp) = cfg.ledger_path {
            validate_external_ledger_path(lp, &deps.data_dir)?;
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
                StoreLedger::open(deps.io.clone(), ledger_dir.clone(), ledger_key, anchor).await?,
            ))
        } else {
            None
        };

        let pending = Arc::new(
            PendingDelete::open(
                deps.io.clone(),
                &cfg.dir,
                deps.data_dir.clone(),
                cfg.pending_retry_limit,
            )
            .await?,
        );

        let j_open = Journal::open(
            deps.io.clone(),
            cfg.dir.clone(),
            cfg.reserve_bytes,
            cfg.max_plan_bytes,
        )
        .await?;

        let gate = Arc::new(StoreGate::new());

        if j_open.corrupt {
            // I-7: Fail-Closed-Modus!
            gate.set_fail_closed();
            *deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
            let ks = KernelState {
                jobs: BTreeMap::new(),
                completed_summaries: BTreeMap::new(),
                summary_records: Vec::new(),
                journal: j_open.journal,
            };
            let kernel = Self {
                cfg,
                deps,
                gate,
                pending,
                ledger,
                state: Arc::new(Mutex::new(ks)),
                status_cache: Arc::new(parking_lot::RwLock::new(BTreeMap::new())),
                open_views_cache: Arc::new(parking_lot::RwLock::new(Vec::new())),
                global_collapse: Arc::new(parking_lot::RwLock::new(Arc::new(CollapseMap::new()))),
                erasure_log_key,
            };
            let report = BootReport {
                fail_closed: true,
                jobs_open: 0,
                torn_tail_truncated: false,
                pending_deleted: 0,
            };
            return Ok((kernel, report));
        }

        // Zustandsautomat aus den gelesenen Journal-Records rekonstruieren (§5.4.3)
        let mut jobs: BTreeMap<ErasureId, Job> = BTreeMap::new();
        let mut summary_records = Vec::new();
        for rec in &j_open.records {
            Self::apply_record_to_jobs(&mut jobs, &mut summary_records, rec)?;
        }

        // Temp-Dateien aus TempFile-Frames über Pending-Delete bereinigen (§5.4.7, CP-15)
        let temp_paths = collect_temp_file_paths(&deps.data_dir, &j_open.records);
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
        let mut ks = KernelState {
            jobs,
            completed_summaries: BTreeMap::new(),
            summary_records,
            journal: j_open.journal,
        };
        let collapse_arc = ks.rebuild_collapse_map();

        let status_map: BTreeMap<ErasureId, ErasureStatus> = ks
            .jobs
            .iter()
            .map(|(id, j)| (*id, j.to_status()))
            .collect();
        let open_views: Vec<JobView> = ks
            .jobs
            .values()
            .filter(|j| !j.state.is_terminal())
            .map(Job::to_job_view)
            .collect();

        // Entferne bereits abgeschlossene Jobs aus `jobs`? Behalten für status(id) bis Verdichtung.
        let _ = &mut ks;

        let kernel = Self {
            cfg,
            deps,
            gate,
            pending,
            ledger,
            state: Arc::new(Mutex::new(ks)),
            status_cache: Arc::new(parking_lot::RwLock::new(status_map)),
            open_views_cache: Arc::new(parking_lot::RwLock::new(open_views)),
            global_collapse: Arc::new(parking_lot::RwLock::new(collapse_arc)),
            erasure_log_key,
        };

        let report = BootReport {
            fail_closed: false,
            jobs_open,
            torn_tail_truncated: j_open.torn_tail_truncated,
            pending_deleted,
        };
        Ok((kernel, report))
    }

    fn apply_record_to_jobs(
        jobs: &mut BTreeMap<ErasureId, Job>,
        summary_records: &mut Vec<JournalRecord>,
        rec: &JournalRecord,
    ) -> Result<(), ErasureError> {
        match rec {
            JournalRecord::Requested {
                id,
                target_hash,
                reason,
                policy,
                ts,
            } => {
                let job = Job {
                    id: *id,
                    state: ErasureState::Requested,
                    plan: Arc::new(ResolvedScope {
                        id: *id,
                        target_kind: TargetKind::KeyPrefix,
                        keys: contextra_ports::erasure::KeyScope::default(),
                        docs: Vec::new(),
                        plan_hash: [0u8; 32],
                        unprovenanced: 0,
                    }),
                    target_hash: *target_hash,
                    reason: reason.clone(),
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
                    job.plan = Arc::new(scope);
                    job.expected_layers = expected_layers.clone();
                    job.state = ErasureState::Planned;
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::GateIntent { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.state = ErasureState::Gating;
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::Gated {
                id, gated_seq, ts, ..
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.gated_seq = Some(Seq(*gated_seq));
                    job.gated_at = Some(*ts);
                    job.state = ErasureState::Gated;
                    job.guarantee_reached = Guarantee {
                        gated: true,
                        purged: false,
                        crypto_erased: false,
                    };
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::KeysDestroying { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.keys_destroyed = true;
                    job.state = ErasureState::KeysDestroying;
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::PurgeStart { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.purge_started = true;
                    job.state = ErasureState::Purging;
                    job.raw_records.push(rec.clone());
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
                        layer.clone(),
                        LayerProgress {
                            cursor: c,
                            units_done: *units_done,
                            bytes_freed: *bytes_freed,
                        },
                    );
                    job.layer_attempts.insert(layer.clone(), *attempt);
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
                    job.state = ErasureState::Verifying;
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::Sealed {
                id,
                ledger_index,
                guarantee,
                exceptions,
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.state = ErasureState::Sealed;
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
                }
            }
            JournalRecord::Completed { id, .. } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.state = ErasureState::Completed;
                    job.sealed_flag.store(true, Ordering::Release);
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::Failed {
                id, layer, attempt, ..
            } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.attempt = *attempt;
                    job.state = ErasureState::Failed {
                        layer: layer.as_ref().map(|l| l.as_str().to_string()),
                        attempt: *attempt,
                    };
                    job.raw_records.push(rec.clone());
                }
            }
            JournalRecord::Quarantined { id, layer } => {
                if let Some(job) = jobs.get_mut(id) {
                    job.state = ErasureState::Quarantined {
                        layer: layer.as_str().to_string(),
                    };
                    job.raw_records.push(rec.clone());
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
        let now = self.deps.clock.read().unix_nanos();
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
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }
            for (idx, id) in updated_ids.iter().enumerate() {
                if let Some(rec) = to_append.get(idx) {
                    if let Some(job) = ks.jobs.get_mut(id) {
                        job.gated_seq = Some(Seq(last_applied_seq));
                        job.gated_at = Some(now);
                        job.state = ErasureState::Gated;
                        job.guarantee_reached = Guarantee {
                            gated: true,
                            purged: false,
                            crypto_erased: false,
                        };
                        job.raw_records.push(rec.clone());
                        let jg = job.to_job_gate();
                        self.gate.install_snapshot(|s, ep| s.with_upserted_job(ep, jg.clone()));
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

    /// Registriert Version-Collapse-Einträge (A-1) durabel im Journal und RAM (§7.2).
    pub(crate) async fn register_collapse(
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
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
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
            let now = self.deps.clock.read().unix_nanos();
            let target_hash = *blake3::hash(&req.target.canonical_bytes()).as_bytes();

            let r_req = JournalRecord::Requested {
                id: req.id,
                target_hash,
                reason: req.reason.clone(),
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
                + r_plan.encode_frame(self.cfg.max_plan_bytes)?.len()) as u64;

            // (4) Platzbedarf prüfen
            if !ks.journal.has_space_for(est_bytes.saturating_add(self.cfg.reserve_bytes)) {
                return Err(ErasureError::InsufficientSpace {
                    required: est_bytes.saturating_add(self.cfg.reserve_bytes),
                    available: 0,
                });
            }

            // (5) Requested + Planned in einem append_batch + sync_data
            if let Err(e) = ks.journal.append_batch(&[r_req.clone(), r_plan.clone()]).await {
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            let job = Job {
                id: req.id,
                state: ErasureState::Planned,
                plan: Arc::new(plan.clone()),
                target_hash,
                reason: req.reason.clone(),
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
            ks.jobs.insert(req.id, job);
            self.refresh_caches(&ks);

            Ok(AcceptReceipt {
                id: req.id,
                existing: false,
            })
        })
    }

    fn publish_gate<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<GateReceipt, ErasureError>> {
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

            let now = self.deps.clock.read().unix_nanos();
            let intent_rec = JournalRecord::GateIntent { id, ts: now };
            if let Err(e) = ks.journal.append_batch(&[intent_rec.clone()]).await {
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.state = ErasureState::Gating;
                job_mut.raw_records.push(intent_rec);
            }

            // Protokoll §6.2 Schritte 4..10 unter commit_mutex (Rang 200) -> snap (Rang 960)
            let (gated_seq, epoch, target_kind, docs) = {
                let _commit_lock = self.deps.commit_mutex.lock().await;
                let seq_val = self.deps.next_seq_no.load(Ordering::SeqCst).saturating_sub(1);
                let job_ref = ks.jobs.get(&id).ok_or(ErasureError::NotFound)?;
                let mut jg = job_ref.to_job_gate();
                jg.up_to = Seq(seq_val);
                jg.phase = GatePhase::Restricted;
                let ep = self
                    .gate
                    .install_snapshot(|s, new_ep| s.with_upserted_job(new_ep, jg.clone()));
                (seq_val, ep, job_ref.plan.target_kind, job_ref.plan.docs.clone())
            };

            let gated_rec = JournalRecord::Gated {
                id,
                gated_seq,
                epoch,
                ts: now,
            };
            if let Err(e) = ks.journal.append_batch(&[gated_rec.clone()]).await {
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.gated_seq = Some(Seq(gated_seq));
                job_mut.gated_at = Some(now);
                job_mut.state = ErasureState::Gated;
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
                        let _ = self.deps.snapshot_registry.revoke_covering(
                            tenant,
                            0..gated_seq.saturating_add(1),
                            gated_seq,
                        );
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

    fn purge_store<'a>(
        &'a self,
        id: ErasureId,
        cx: &'a PurgeCx<'a>,
    ) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>> {
        Box::pin(async move {
            if cx.cancel.is_cancelled() {
                return Err(ErasureError::Cancelled);
            }

            let (scope, attempt, need_purge_start) = {
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
                );
                if !valid_state || job.gated_seq.is_none() {
                    return Err(ErasureError::InvalidTransition);
                }
                let need_start = !job.purge_started;
                let scope = Arc::clone(&job.plan);
                let attempt = job.attempt;

                if need_start {
                    let now = self.deps.clock.read().unix_nanos();
                    let rec = JournalRecord::PurgeStart { id, ts: now };
                    if let Err(e) = ks.journal.append_batch(&[rec.clone()]).await {
                        *self.deps.health.write() =
                            crate::lsm::engine::StorageHealth::JournalPoisoned;
                        return Err(e);
                    }
                    if let Some(job_mut) = ks.jobs.get_mut(&id) {
                        job_mut.purge_started = true;
                        job_mut.state = ErasureState::Purging;
                        job_mut.raw_records.push(rec);
                        let jg = job_mut.to_job_gate();
                        self.gate
                            .install_snapshot(|s, ep| s.with_upserted_job(ep, jg.clone()));
                    }
                    self.refresh_caches(&ks);
                }
                (scope, attempt, need_start)
            };
            let _ = need_purge_start;

            let collapse = self.collapse_snapshot();
            let filter = build_filter(&self.gate, &collapse);

            // Phase 1: MemTable-Retain (§7.1)
            let (mem_dropped, mem_freed) = purge_memtables(&self.deps, &filter).await;

            // Phase 2: Erase-Compaction über SSTables (§7.3)
            let (next_cursor, sst_units, sst_freed) = erase_compact_tick(
                &self.deps,
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
                erase_wal_cut(&self.deps, &filter, &self.pending).await?
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
            if !matches!(
                job.state,
                ErasureState::Purging | ErasureState::Verifying | ErasureState::Gated
            ) {
                return Err(ErasureError::InvalidTransition);
            }
            let attempt = job.layer_attempts.get(layer).copied().unwrap_or(0);
            let cursor_bytes = p
                .cursor
                .as_ref()
                .map(|c| c.as_bytes().to_vec())
                .unwrap_or_default();
            let rec = JournalRecord::LayerProgress {
                id,
                layer: layer.clone(),
                cursor: cursor_bytes,
                units_done: p.units_done,
                bytes_freed: p.bytes_freed,
                attempt,
            };
            if let Err(e) = ks.journal.append_batch(&[rec.clone()]).await {
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }
            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.layer_cursor.insert(layer.clone(), p.clone());
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
                _existing_state,
            ) = {
                let mut ks = self.state.lock().await;
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
            let mut remaining_map: BTreeMap<WitnessLayer, u64> = BTreeMap::new();

            let mut sorted_expected = expected_layers.clone();
            sorted_expected.sort_by(|a, b| layer_sort_key(a).cmp(&layer_sort_key(b)));

            let mut root_hasher = blake3::Hasher::new();
            root_hasher.update(b"ctx-layers-v1");

            for layer in &sorted_expected {
                let (tag, name) = layer_sort_key(layer);
                if let Some(wit) = witness_by_layer.get(&(tag, name.to_string())) {
                    root_hasher.update(&[tag]);
                    root_hasher.update(wit.digest());
                    remaining_map.insert(layer.clone(), wit.remaining());
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

            if let Some(gs) = gated_seq {
                if let Some(min_snap) = self.deps.snapshot_registry.min_read_snapshot() {
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

            let now = self.deps.clock.read().unix_nanos();
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
                    completed_at: Timestamp::from_nanos(now),
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
            if let Err(e) = ks.journal.append_batch(&[r_ver.clone(), r_seal.clone()]).await {
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.state = ErasureState::Sealed;
                job_mut.sealed_at = Some(now);
                job_mut.sealed_flag.store(true, Ordering::Release);
                job_mut.ledger_index = ledger_index;
                job_mut.guarantee_reached = guarantee;
                job_mut.exceptions = exceptions.clone();
                job_mut.layer_remaining = remaining_map;
                job_mut.raw_records.push(r_ver);
                job_mut.raw_records.push(r_seal);
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
                match job.layer_remaining.get(layer) {
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
                if let Some(min_snap) = self.deps.snapshot_registry.min_read_snapshot() {
                    let has_pinned_exc = job
                        .exceptions
                        .iter()
                        .any(|e| matches!(e, ErasureException::OverlappingSnapshotPinned { .. }));
                    if min_snap <= gs.0 && !has_pinned_exc {
                        return Err(ErasureError::InvalidTransition);
                    }
                }
            }

            let now = self.deps.clock.read().unix_nanos();
            let comp_rec = JournalRecord::Completed { id, ts: now };
            if let Err(e) = ks.journal.append_batch(&[comp_rec.clone()]).await {
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.state = ErasureState::Completed;
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
                for (jid, j) in &ks.jobs {
                    if j.state.is_terminal() {
                        let id_hmac = contextra_crypto::ledger_mac(&self.erasure_log_key, &jid.0)
                            .map_err(|e| ErasureError::Io(e.to_string()))?;
                        let plan_hmac = contextra_crypto::ledger_mac(
                            &self.erasure_log_key,
                            &j.plan.plan_hash,
                        )
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
                        ks.summary_records.push(sum.clone());
                        ks.completed_summaries.insert(*jid, j.to_status());
                        completed_ids.push(*jid);
                    } else {
                        compact_records.extend(j.raw_records.clone());
                    }
                }
                for cid in completed_ids {
                    ks.jobs.remove(&cid);
                }
                compact_records.extend(ks.summary_records.clone());
                let _ = ks.journal.compact(&compact_records).await;
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

            let rec = match &new_state {
                ErasureState::Quarantined { .. } => {
                    if let Some(l) = layer {
                        JournalRecord::Quarantined {
                            id,
                            layer: l.clone(),
                        }
                    } else {
                        JournalRecord::Failed {
                            id,
                            layer: None,
                            kind,
                            attempt: next_attempt,
                        }
                    }
                }
                _ => JournalRecord::Failed {
                    id,
                    layer: layer.cloned(),
                    kind,
                    attempt: next_attempt,
                },
            };

            if let Err(e) = ks.journal.append_batch(&[rec.clone()]).await {
                *self.deps.health.write() = crate::lsm::engine::StorageHealth::JournalPoisoned;
                return Err(e);
            }

            if let Some(job_mut) = ks.jobs.get_mut(&id) {
                job_mut.attempt = next_attempt;
                job_mut.state = new_state.clone();
                if let Some(l) = layer {
                    job_mut.layer_attempts.insert(l.clone(), next_attempt);
                }
                job_mut.raw_records.push(rec);
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
        let sealed_flag = Arc::new(AtomicBool::new(false));
        Ok(Arc::new(StorePrivilegedReader::new(
            self.deps.clone(),
            Arc::clone(&view.scope),
            sealed_flag,
        )))
    }

    fn gate(&self) -> Arc<dyn ReadGate> {
        Arc::clone(&self.gate) as Arc<dyn ReadGate>
    }

    fn store_scanners(&self) -> Vec<Arc<dyn LayerScanner>> {
        vec![
            Arc::new(KernelStoreScanner::new(
                WitnessLayer::LsmMemtable,
                self.deps.clone(),
            )),
            Arc::new(KernelStoreScanner::new(
                WitnessLayer::SsTableAllLevels,
                self.deps.clone(),
            )),
            Arc::new(KernelStoreScanner::new(
                WitnessLayer::WalAllSegments,
                self.deps.clone(),
            )),
        ]
    }
}
