//! # Mocks der Löschkette (`erasure_mocks`)
//!
//! - [`MockErasureKernel`]: Zustandsautomat-Contract des [`ErasureKernel`]-Ports.
//! - [`MockLayer`]: korrekte Referenzschicht über [`CrashVfs`] mit schaltbaren [`Defect`]s
//!   (Mutationsnachweis der Konformanzsuite).
//! - [`MockPrivilegedRead`], [`MockProgressSink`], [`ManualIoBudget`].

use crate::conformance::erasure_layer::{FaultSpec, LayerFixture};
use crate::crash_vfs::{CrashSeed, CrashVfs};
use bytes::Bytes;
use contextra_ports::erasure::{
    AcceptReceipt, EraseRequest, ErasureKernel, ErasureLayer, GateReceipt, IoBudget, IoClass,
    JobView, LayerError, PressureHint, PrivilegedRead, ProgressSink, PurgeCx, PurgeOutcome,
    ResolvedScope, SealReceipt,
};
use contextra_ports::gate::{DiscardRule, GateDecision, RangeGate, ReadGate};
use contextra_ports::scanner::{LayerScanner, LayerWitness, ScanSession, WitnessLayer};
use contextra_ports::{BoxFuture, DocScope, KeyPattern, KeyScope, Verdict, Vfs};
use contextra_types::{
    Cursor, DocId, EraseTarget, ErasureError, ErasureException, ErasureId, ErasureState,
    ErasureStatus, Guarantee, LayerErrorKind, LayerProgress, RequestContext, Seq, TenantId,
};
use parking_lot::Mutex;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

/// Gibt genau einmal die Kontrolle ab (echter `await`-Punkt ohne Laufzeitabhängigkeit).
pub(crate) fn yield_now() -> impl Future<Output = ()> + Send {
    struct Yield(bool);
    impl Future for Yield {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.0 {
                Poll::Ready(())
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
    Yield(false)
}

// ───────────────────────────────────── PrivilegedRead ─────────────────────────────────────

/// Scope-prüfender Reader über einer In-Memory-Tabelle.
pub struct MockPrivilegedRead {
    scope: KeyScope,
    data: Mutex<BTreeMap<Vec<u8>, Bytes>>,
    violations: AtomicU64,
}

/// Mandant 1 ohne den abgekündigten Konstruktor (`TenantId(0)` ist reserviert).
fn tenant() -> TenantId {
    TenantId::try_new(TENANT).unwrap_or(TenantId::SYSTEM)
}

fn scope_violation() -> ErasureError {
    ErasureError::Layer {
        layer: "store".to_string(),
        kind: LayerErrorKind::ScopeViolation,
    }
}

impl MockPrivilegedRead {
    /// Leerer Reader für `scope`.
    pub fn new(scope: KeyScope) -> Self {
        Self {
            scope,
            data: Mutex::new(BTreeMap::new()),
            violations: AtomicU64::new(0),
        }
    }

    /// Legt einen Eintrag an.
    pub fn insert(&self, key: impl Into<Vec<u8>>, value: impl Into<Bytes>) {
        self.data.lock().insert(key.into(), value.into());
    }

    /// Anzahl abgewiesener Zugriffe.
    pub fn violations(&self) -> u64 {
        self.violations.load(Ordering::SeqCst)
    }

    fn deny(&self) -> ErasureError {
        self.violations.fetch_add(1, Ordering::SeqCst);
        scope_violation()
    }
}

impl PrivilegedRead for MockPrivilegedRead {
    fn get<'a>(&'a self, raw_key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>, ErasureError>> {
        Box::pin(async move {
            if !self.scope.contains(raw_key) {
                return Err(self.deny());
            }
            Ok(self.data.lock().get(raw_key).cloned())
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
        after: Option<&'a [u8]>,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<(Bytes, Bytes)>, ErasureError>> {
        Box::pin(async move {
            let covered = self.scope.rules().iter().any(|(pat, v)| {
                matches!(pat, KeyPattern::Prefix(p) if prefix.starts_with(p)) && *v == Verdict::In
            });
            if !covered {
                return Err(self.deny());
            }
            Ok(self
                .data
                .lock()
                .range(prefix.to_vec()..)
                .take_while(|(k, _)| k.starts_with(prefix))
                .filter(|(k, _)| after.is_none_or(|a| k.as_slice() > a))
                .filter(|(k, _)| self.scope.contains(k))
                .take(limit.min(1024))
                .map(|(k, v)| (Bytes::copy_from_slice(k), v.clone()))
                .collect())
        })
    }
}

// ───────────────────────────────── ProgressSink, IoBudget ─────────────────────────────────

/// Zeichnet Fortschrittsmeldungen auf.
#[derive(Default)]
pub struct MockProgressSink {
    records: Mutex<Vec<(WitnessLayer, LayerProgress)>>,
    fail_after: Mutex<Option<usize>>,
}

impl MockProgressSink {
    /// Leerer Sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ab der `n+1`-ten Meldung schlägt `record` fehl.
    pub fn fail_after(&self, n: usize) {
        *self.fail_after.lock() = Some(n);
    }

    /// Angenommene Meldungen.
    pub fn records(&self) -> Vec<(WitnessLayer, LayerProgress)> {
        self.records.lock().clone()
    }
}

impl ProgressSink for MockProgressSink {
    fn record<'a>(
        &'a self,
        layer: &'a WitnessLayer,
        p: &'a LayerProgress,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            let mut records = self.records.lock();
            if self.fail_after.lock().is_some_and(|n| records.len() >= n) {
                return Err(ErasureError::Io("injected progress failure".to_string()));
            }
            records.push((layer.clone(), p.clone()));
            Ok(())
        })
    }
}

/// Manuell gesteuertes I/O-Budget; wartet nie, sondern meldet `Busy`.
pub struct ManualIoBudget {
    remaining: Mutex<Option<u64>>,
    pressure: Mutex<PressureHint>,
    log: Mutex<Vec<(IoClass, u64)>>,
}

impl Default for ManualIoBudget {
    fn default() -> Self {
        Self {
            remaining: Mutex::new(None),
            pressure: Mutex::new(PressureHint::Normal),
            log: Mutex::new(Vec::new()),
        }
    }
}

impl ManualIoBudget {
    /// Unbegrenztes Budget.
    pub fn unlimited() -> Self {
        Self::default()
    }

    /// Setzt die Restbytes (`None` = unbegrenzt).
    pub fn set_remaining(&self, bytes: Option<u64>) {
        *self.remaining.lock() = bytes;
    }

    /// Setzt den Druckhinweis.
    pub fn set_pressure(&self, hint: PressureHint) {
        *self.pressure.lock() = hint;
    }

    /// Bewilligte Anforderungen.
    pub fn log(&self) -> Vec<(IoClass, u64)> {
        self.log.lock().clone()
    }
}

impl IoBudget for ManualIoBudget {
    fn acquire<'a>(
        &'a self,
        class: IoClass,
        bytes: u64,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            let mut remaining = self.remaining.lock();
            if let Some(left) = *remaining {
                *remaining = Some(
                    left.checked_sub(bytes)
                        .ok_or(ErasureError::Busy { active: 0 })?,
                );
            }
            self.log.lock().push((class, bytes));
            Ok(())
        })
    }

    fn pressure(&self) -> PressureHint {
        *self.pressure.lock()
    }
}

// ───────────────────────────────────── ErasureKernel ─────────────────────────────────────

/// Versuche, nach denen ein Auftrag in Quarantäne geht.
pub const MOCK_QUARANTINE_ATTEMPTS: u32 = 3;

#[derive(Clone)]
struct Job {
    target: EraseTarget,
    scope: Arc<ResolvedScope>,
    state: ErasureState,
    gated_seq: Option<Seq>,
    layers_done: Vec<WitnessLayer>,
    attempt: u32,
}

#[derive(Default)]
struct KernelState {
    jobs: BTreeMap<ErasureId, Job>,
    next_seq: u64,
    epoch: u64,
}

fn advance(job: &mut Job, next: ErasureState) -> Result<(), ErasureError> {
    if !job.state.can_transition_to(&next) {
        return Err(ErasureError::InvalidTransition);
    }
    job.state = next;
    Ok(())
}

/// In-Memory-Kernel, der ausschließlich erlaubte Zustandsübergänge zulässt.
///
/// Das Gate bleibt ab `publish_gate` bis `Completed` gesetzt, auch über `Failed` und
/// `Quarantined` hinweg (I-9).
#[derive(Clone, Default)]
pub struct MockErasureKernel {
    state: Arc<Mutex<KernelState>>,
    expected_layers: Vec<WitnessLayer>,
    reader_data: Arc<Mutex<BTreeMap<Vec<u8>, Bytes>>>,
}

impl MockErasureKernel {
    /// Kernel, der bei `seal` für jede Schicht in `expected_layers` einen Witness verlangt.
    pub fn new(expected_layers: Vec<WitnessLayer>) -> Self {
        Self {
            expected_layers,
            ..Self::default()
        }
    }

    /// Daten, die [`ErasureKernel::privileged_reader`] sichtbar macht.
    pub fn insert_data(&self, key: impl Into<Vec<u8>>, value: impl Into<Bytes>) {
        self.reader_data.lock().insert(key.into(), value.into());
    }

    fn with_job<T>(
        &self,
        id: ErasureId,
        f: impl FnOnce(&mut Job, &mut u64, &mut u64) -> Result<T, ErasureError>,
    ) -> Result<T, ErasureError> {
        let mut s = self.state.lock();
        let KernelState {
            jobs,
            next_seq,
            epoch,
        } = &mut *s;
        let job = jobs.get_mut(&id).ok_or(ErasureError::NotFound)?;
        f(job, next_seq, epoch)
    }
}

struct MockKernelGate(Arc<Mutex<KernelState>>);

impl MockKernelGate {
    fn blocked_seq(&self, matches: impl Fn(&ResolvedScope) -> bool) -> Option<Seq> {
        self.0
            .lock()
            .jobs
            .values()
            .filter(|j| j.state != ErasureState::Completed && matches(&j.scope))
            .filter_map(|j| j.gated_seq)
            .max()
    }
}

fn doc_in_scope(s: &ResolvedScope, tenant: TenantId, collection: &str, doc: DocId) -> bool {
    s.docs.iter().any(|d| match d {
        DocScope::Set {
            tenant: t,
            collection: c,
            docs,
        } => *t == tenant && c == collection && docs.contains(&doc),
        DocScope::Collection {
            tenant: t,
            collection: c,
        } => *t == tenant && c == collection,
        DocScope::Tenant { tenant: t } => *t == tenant,
    })
}

impl ReadGate for MockKernelGate {
    fn is_active(&self) -> bool {
        self.blocked_seq(|_| true).is_some()
    }
    fn epoch(&self) -> u64 {
        self.0.lock().epoch
    }
    fn check(&self, raw_key: &[u8]) -> GateDecision {
        match self.gated_seq(raw_key) {
            Some(_) => GateDecision::Restricted,
            None => GateDecision::Open,
        }
    }
    fn check_version(&self, raw_key: &[u8], _version_seq: Seq) -> GateDecision {
        // Das Gate gilt für alle Sequenzen (auch Zeitreise).
        self.check(raw_key)
    }
    fn check_prefix(&self, prefix: &[u8]) -> RangeGate {
        let mut upper = prefix.to_vec();
        upper.extend_from_slice(&[0xFF; 64]);
        if self
            .blocked_seq(|s| s.keys.could_overlap_range(prefix, &upper))
            .is_some()
        {
            RangeGate::Partial
        } else {
            RangeGate::EntirelyOpen
        }
    }
    fn discard_rule(&self, raw_key: &[u8], version_seq: Seq) -> DiscardRule {
        match self.gated_seq(raw_key) {
            Some(up_to) if version_seq <= up_to => DiscardRule::MayDrop,
            _ => DiscardRule::Keep,
        }
    }
    fn check_doc(&self, tenant: TenantId, collection: &str, doc: DocId) -> GateDecision {
        match self.blocked_seq(|s| doc_in_scope(s, tenant, collection, doc)) {
            Some(_) => GateDecision::Restricted,
            None => GateDecision::Open,
        }
    }
    fn gated_seq(&self, raw_key: &[u8]) -> Option<Seq> {
        self.blocked_seq(|s| s.keys.contains(raw_key))
    }
}

impl ErasureKernel for MockErasureKernel {
    fn accept<'a>(
        &'a self,
        req: &'a EraseRequest,
        plan: &'a ResolvedScope,
    ) -> BoxFuture<'a, Result<AcceptReceipt, ErasureError>> {
        Box::pin(async move {
            req.target.validate()?;
            if plan.id != req.id {
                return Err(ErasureError::InvalidTarget("plan id differs from request"));
            }
            let mut s = self.state.lock();
            if let Some(job) = s.jobs.get(&req.id) {
                if job.target != req.target || job.scope.plan_hash != plan.plan_hash {
                    return Err(ErasureError::IdempotencyConflict);
                }
                return Ok(AcceptReceipt {
                    id: req.id,
                    existing: true,
                });
            }
            s.jobs.insert(
                req.id,
                Job {
                    target: req.target.clone(),
                    scope: Arc::new(plan.clone()),
                    state: ErasureState::Requested,
                    gated_seq: None,
                    layers_done: Vec::new(),
                    attempt: 0,
                },
            );
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
            self.with_job(id, |job, next_seq, epoch| {
                let gated_seq = match job.gated_seq {
                    Some(seq) => seq,
                    None => {
                        advance(job, ErasureState::Planned)?;
                        advance(job, ErasureState::Gating)?;
                        *next_seq += 1;
                        *epoch += 1;
                        job.gated_seq = Some(Seq(*next_seq));
                        advance(job, ErasureState::Gated)?;
                        Seq(*next_seq)
                    }
                };
                Ok(GateReceipt {
                    id,
                    gated_seq,
                    epoch: *epoch,
                })
            })
        })
    }

    fn begin_purge<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            self.with_job(id, |job, _, _| {
                if job.state != ErasureState::Purging {
                    advance(job, ErasureState::Purging)?;
                }
                Ok(())
            })
        })
    }

    fn purge_store<'a>(
        &'a self,
        id: ErasureId,
        _cx: &'a PurgeCx<'a>,
    ) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>> {
        Box::pin(async move {
            self.with_job(id, |job, _, _| {
                if job.state != ErasureState::Purging {
                    advance(job, ErasureState::Purging)?;
                }
                let mut data = self.reader_data.lock();
                let before = data.len();
                data.retain(|k, _| !job.scope.keys.contains(k));
                let units = (before - data.len()) as u64;
                Ok(PurgeOutcome {
                    progress: LayerProgress {
                        cursor: None,
                        units_done: units,
                        bytes_freed: 0,
                    },
                    units_committed: units,
                    bytes_freed: 0,
                })
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
            self.with_job(id, |job, _, _| {
                if job.state != ErasureState::Purging {
                    return Err(ErasureError::InvalidTransition);
                }
                if p.cursor.is_none() && !job.layers_done.contains(layer) {
                    job.layers_done.push(layer.clone());
                }
                Ok(())
            })
        })
    }

    fn seal<'a>(
        &'a self,
        id: ErasureId,
        w: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<SealReceipt, ErasureError>> {
        Box::pin(async move {
            self.with_job(id, |job, _, _| {
                if job.state != ErasureState::Verifying {
                    advance(job, ErasureState::Verifying)?;
                }
                let mut exceptions = vec![ErasureException::OutOfScopeExternalCopies];
                let mut purged = true;
                for layer in &self.expected_layers {
                    let of_layer: Vec<_> =
                        w.iter().filter(|w| w.witness_layer() == layer).collect();
                    if of_layer.iter().any(|w| w.remaining() > 0) {
                        // Rest gefunden: zurück nach `Purging`, kein Siegel.
                        advance(job, ErasureState::Purging)?;
                        job.layers_done.retain(|l| l != layer);
                        return Err(ErasureError::Layer {
                            layer: layer.as_str().to_string(),
                            kind: LayerErrorKind::Retryable,
                        });
                    }
                    let measured = !of_layer.is_empty()
                        && of_layer.iter().all(|w| {
                            matches!(w.method(), contextra_types::witness::ScanMethod::Structural)
                        });
                    if !measured {
                        purged = false;
                        exceptions.push(ErasureException::NotMeasured {
                            layer: layer.as_str().to_string(),
                        });
                    }
                }
                if job.scope.unprovenanced > 0 {
                    exceptions.push(ErasureException::Unprovenanced {
                        objects: job.scope.unprovenanced,
                    });
                }
                advance(job, ErasureState::Sealed)?;
                Ok(SealReceipt {
                    id,
                    guarantee: Guarantee::new(true, purged, false)?,
                    exceptions,
                    ledger_index: None,
                })
            })
        })
    }

    fn complete<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            self.with_job(id, |job, _, epoch| {
                advance(job, ErasureState::Completed)?;
                *epoch += 1;
                Ok(())
            })
        })
    }

    fn fail<'a>(
        &'a self,
        id: ErasureId,
        layer: Option<&'a WitnessLayer>,
        _kind: LayerErrorKind,
    ) -> BoxFuture<'a, Result<ErasureState, ErasureError>> {
        Box::pin(async move {
            self.with_job(id, |job, _, _| {
                let name = layer.map(|l| l.as_str().to_string());
                let attempt = job.attempt + 1;
                advance(
                    job,
                    ErasureState::Failed {
                        layer: name.clone(),
                        attempt,
                    },
                )?;
                job.attempt = attempt;
                if attempt >= MOCK_QUARANTINE_ATTEMPTS {
                    advance(
                        job,
                        ErasureState::Quarantined {
                            layer: name.unwrap_or_else(|| "store".to_string()),
                        },
                    )?;
                }
                Ok(job.state.clone())
            })
        })
    }

    fn open_jobs(&self) -> Vec<JobView> {
        self.state
            .lock()
            .jobs
            .iter()
            .filter(|(_, j)| !j.state.is_terminal())
            .map(|(id, j)| JobView {
                id: *id,
                state: j.state.clone(),
                scope: Arc::clone(&j.scope),
                layers_done: j.layers_done.clone(),
                attempt: j.attempt,
            })
            .collect()
    }

    fn status(&self, id: ErasureId) -> Option<ErasureStatus> {
        self.state.lock().jobs.get(&id).map(|j| {
            let mut reached = j.state.reached();
            reached.gated |= j.gated_seq.is_some();
            ErasureStatus {
                id,
                state: j.state.clone(),
                reached,
                layers: Vec::new(),
                exceptions: Vec::new(),
                requested_at: 0,
                gated_at: None,
                sealed_at: None,
                ledger_index: None,
            }
        })
    }

    fn privileged_reader(&self, id: ErasureId) -> Result<Arc<dyn PrivilegedRead>, ErasureError> {
        self.with_job(id, |job, _, _| {
            if matches!(job.state, ErasureState::Sealed | ErasureState::Completed) {
                return Err(ErasureError::InvalidTransition);
            }
            let reader = MockPrivilegedRead::new(job.scope.keys.clone());
            for (k, v) in self.reader_data.lock().iter() {
                reader.insert(k.clone(), v.clone());
            }
            Ok(Arc::new(reader) as Arc<dyn PrivilegedRead>)
        })
    }

    fn gate(&self) -> Arc<dyn ReadGate> {
        Arc::new(MockKernelGate(Arc::clone(&self.state)))
    }

    fn store_scanners(&self) -> Vec<Arc<dyn LayerScanner>> {
        Vec::new()
    }
}

// ─────────────────────────────────────── MockLayer ───────────────────────────────────────

/// Absichtlicher Fehler in [`MockLayer`]; jede Variante muss genau von der Suite erkannt werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Defect {
    /// Korrekte Schicht.
    #[default]
    None,
    /// C1: committet bei jedem Lauf erneut Einheiten.
    RecountsOnRerun,
    /// C2: überspringt beim Fortsetzen mit Cursor ein Dokument.
    SkipsOnResume,
    /// C3: verändert Nachbardaten.
    TouchesNeighbor,
    /// C4: Scanner meldet immer `remaining == 0`.
    BlindScanner,
    /// C5: meldet bei Fehlern `Ok` mit Teilzähler.
    PartialOkOnFault,
    /// C6: markiert Fortschritt vor der Wirkung (nicht abbruchsicher).
    ProgressBeforeEffect,
    /// C7: `on_gate` blendet nichts aus.
    IgnoresGate,
    /// C7: liest am privilegierten Reader vorbei.
    DirectRead,
    /// K1: zweiter `on_gate`-Aufruf verändert den Zustand.
    NonIdempotentGate,
    /// K2: Ergebnisbytes hängen von der Instanz ab.
    NondeterministicOutput,
}

const TENANT: u64 = 1;
const COLLECTION: &str = "c";
const DIR: &str = "/layer";

/// Baut eine `DocId`; `.into()` ist nur ohne Feature `docid-128` eine Identität.
#[allow(clippy::useless_conversion)]
fn doc_id(n: u64) -> DocId {
    DocId(n.into())
}

/// Der Cursor ist die Dezimaldarstellung der zuletzt erledigten Mock-ID.
fn parse_cursor(bytes: &[u8]) -> Option<DocId> {
    std::str::from_utf8(bytes).ok()?.parse().ok().map(doc_id)
}

fn key_of(doc: DocId) -> Vec<u8> {
    format!("doc/{:020}", doc.0).into_bytes()
}

fn file_of(doc: DocId) -> PathBuf {
    PathBuf::from(format!("{DIR}/{:020}", doc.0))
}

struct Shared {
    vfs: CrashVfs,
    scope: ResolvedScope,
    defect: Defect,
    hidden: Mutex<BTreeSet<DocId>>,
    scope_docs: Mutex<Vec<DocId>>,
    neighbors: Mutex<Vec<DocId>>,
    fault: Mutex<Option<FaultSpec>>,
    direct_blocked_reads: AtomicU64,
    gate_calls: AtomicU64,
    instance: u64,
}

/// Referenzschicht: ein Dokument je Datei in [`CrashVfs`].
#[derive(Clone)]
pub struct MockLayer(Arc<Shared>);

impl MockLayer {
    fn error(&self, kind: LayerErrorKind, detail: &'static str) -> LayerError {
        LayerError {
            kind,
            layer: ErasureLayer::layer(self),
            detail,
        }
    }

    fn io(&self) -> impl Fn(contextra_types::ContextraError) -> LayerError + '_ {
        |_| self.error(LayerErrorKind::Retryable, "vfs_io")
    }

    fn present(&self, doc: DocId) -> bool {
        matches!(self.0.vfs.exists(&file_of(doc)), Ok(true))
    }

    fn scope_docs(s: &ResolvedScope) -> Vec<DocId> {
        s.docs
            .iter()
            .filter_map(|d| match d {
                DocScope::Set { docs, .. } => Some(docs.iter().copied()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    async fn run_purge(
        &self,
        s: &ResolvedScope,
        cx: &PurgeCx<'_>,
    ) -> Result<PurgeOutcome, LayerError> {
        let defect = self.0.defect;
        let mut after = match &cx.cursor {
            Some(c) => Some(
                parse_cursor(c.as_bytes())
                    .ok_or_else(|| self.error(LayerErrorKind::Permanent, "bad_cursor"))?,
            ),
            None => None,
        };
        let mut skip = defect == Defect::SkipsOnResume && after.is_some();
        let docs = Self::scope_docs(s);
        let mut committed = 0u64;
        let mut freed = 0u64;
        let progress_file = PathBuf::from(format!("{DIR}/progress"));
        if defect == Defect::ProgressBeforeEffect && cx.cursor.is_none() {
            // Fehlerhaft: eigener Fortschritt überschreibt den Cursor des Aufrufers.
            if let Ok(b) = self.0.vfs.read(&progress_file) {
                after = parse_cursor(&b);
            }
        }
        let todo: Vec<DocId> = docs
            .iter()
            .copied()
            .filter(|d| after.is_none_or(|a| *d > a))
            .collect();
        let mut last = after;
        for doc in todo.iter().take(cx.max_units.max(1)) {
            if cx.cancel.is_cancelled() {
                return Err(self.error(LayerErrorKind::Cancelled, "cancelled"));
            }
            if let Some(fault) = *self.0.fault.lock() {
                if defect == Defect::PartialOkOnFault {
                    break;
                }
                let (kind, detail) = fault.expected_error();
                return Err(self.error(kind, detail));
            }
            if defect == Defect::DirectRead && self.0.vfs.read(&file_of(*doc)).is_ok() {
                self.0.direct_blocked_reads.fetch_add(1, Ordering::SeqCst);
            }
            let body = cx
                .reader
                .get(&key_of(*doc))
                .await
                .map_err(|e| self.error(e.kind(), "reader"))?;
            if std::mem::take(&mut skip) {
                last = Some(*doc);
                continue;
            }
            if defect == Defect::ProgressBeforeEffect {
                self.0
                    .vfs
                    .write_atomic(&progress_file, doc.0.to_string().as_bytes())
                    .map_err(self.io())?;
            }
            cx.budget
                .acquire(IoClass::Erase, body.as_ref().map_or(0, |b| b.len() as u64))
                .await
                .map_err(|e| self.error(e.kind(), "budget"))?;
            yield_now().await;
            if self.present(*doc) {
                let size = self.0.vfs.len(&file_of(*doc)).map_err(self.io())?;
                self.0.vfs.remove_file(&file_of(*doc)).map_err(self.io())?;
                self.0
                    .vfs
                    .sync_dir(std::path::Path::new(DIR))
                    .map_err(self.io())?;
                committed += 1;
                freed += size;
            } else if defect == Defect::RecountsOnRerun {
                committed += 1;
            }
            last = Some(*doc);
        }
        if defect == Defect::TouchesNeighbor {
            let first = self.0.neighbors.lock().first().copied();
            if let Some(n) = first {
                self.0
                    .vfs
                    .write_atomic(&file_of(n), b"damaged")
                    .map_err(self.io())?;
            }
        }
        let done = docs.iter().all(|d| last.is_some_and(|l| *d <= l));
        let cursor = match (done, last) {
            (false, Some(l)) => Some(
                Cursor::new(l.0.to_string().into_bytes())
                    .map_err(|e| self.error(e.kind(), "cursor"))?,
            ),
            // Fehler vor der ersten Einheit: Cursor des Aufrufers behalten.
            (false, None) => cx.cursor.clone(),
            (true, _) => None,
        };
        let progress = LayerProgress {
            cursor,
            units_done: committed,
            bytes_freed: freed,
        };
        cx.progress
            .record(&ErasureLayer::layer(self), &progress)
            .await
            .map_err(|e| self.error(e.kind(), "progress"))?;
        Ok(PurgeOutcome {
            progress,
            units_committed: committed,
            bytes_freed: freed,
        })
    }
}

impl ErasureLayer for MockLayer {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::Custom("mock".to_string())
    }

    fn on_gate<'a>(&'a self, s: &'a ResolvedScope) -> BoxFuture<'a, Result<(), LayerError>> {
        Box::pin(async move {
            let calls = self.0.gate_calls.fetch_add(1, Ordering::SeqCst);
            match self.0.defect {
                Defect::IgnoresGate => {}
                Defect::NonIdempotentGate if calls > 0 => {
                    self.0
                        .vfs
                        .write_atomic(&PathBuf::from(format!("{DIR}/gate-{calls}")), b"x")
                        .map_err(self.io())?;
                }
                _ => self.0.hidden.lock().extend(Self::scope_docs(s)),
            }
            Ok(())
        })
    }

    fn purge<'a>(
        &'a self,
        s: &'a ResolvedScope,
        cx: &'a PurgeCx<'a>,
    ) -> BoxFuture<'a, Result<PurgeOutcome, LayerError>> {
        Box::pin(self.run_purge(s, cx))
    }

    fn scanner(&self) -> &dyn LayerScanner {
        self
    }
}

impl LayerScanner for MockLayer {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::Custom("mock".to_string())
    }

    fn scan_units(
        &self,
        session: &mut ScanSession,
        _ctx: &RequestContext,
    ) -> contextra_types::Result<()> {
        let broken = *self.0.fault.lock() == Some(FaultSpec::BrokenScanner);
        let names = self.0.vfs.list_dir(std::path::Path::new(DIR))?;
        let in_scope: BTreeSet<DocId> = self.0.scope_docs.lock().iter().copied().collect();
        for name in names {
            let Ok(id) = name.parse::<u64>() else {
                continue;
            };
            let residual = self.0.defect != Defect::BlindScanner && in_scope.contains(&doc_id(id));
            let mut unit = [0u8; 32];
            for (dst, src) in unit.iter_mut().zip(id.to_le_bytes()) {
                *dst = src;
            }
            session.record_unit(unit, residual);
            // Positivkontrolle: ein bekannter Nachbar muss gefunden werden.
            if !broken && self.0.neighbors.lock().contains(&doc_id(id)) {
                session.record_control_match();
            }
        }
        Ok(())
    }
}

/// [`LayerFixture`] um [`MockLayer`]; `D` wählt den Defekt zur Übersetzungszeit.
pub struct MockLayerFixture {
    layer: MockLayer,
    reader: Arc<MockPrivilegedRead>,
}

static INSTANCES: AtomicU64 = AtomicU64::new(1);

impl MockLayerFixture {
    /// Fixture mit gewähltem Defekt.
    pub fn with_defect(vfs: CrashVfs, defect: Defect) -> Self {
        let keys = KeyScope::new(vec![(KeyPattern::Prefix(b"doc/".to_vec()), Verdict::In)])
            .unwrap_or_default();
        let target = EraseTarget::Collection {
            tenant: tenant(),
            collection: COLLECTION.to_string(),
        };
        let scope = ResolvedScope::new(
            ErasureId::from_u128(1),
            target.kind(),
            &target,
            keys,
            Vec::new(),
            0,
        );
        let layer = MockLayer(Arc::new(Shared {
            vfs,
            scope,
            defect,
            hidden: Mutex::new(BTreeSet::new()),
            scope_docs: Mutex::new(Vec::new()),
            neighbors: Mutex::new(Vec::new()),
            fault: Mutex::new(None),
            direct_blocked_reads: AtomicU64::new(0),
            gate_calls: AtomicU64::new(0),
            instance: INSTANCES.fetch_add(1, Ordering::SeqCst),
        }));
        Self {
            reader: Arc::new(MockPrivilegedRead::new(KeyScope::default())),
            layer,
        }
    }
}

/// Fixtures können Fehler nicht zurückgeben; ein Setup-Fehler macht sie sichtbar leer,
/// woraufhin die Vorbedingungsprüfungen der Suite fehlschlagen.
fn write_doc(vfs: &CrashVfs, doc: DocId) -> bool {
    let body = format!("body-{}", doc.0);
    vfs.create_dir_all(std::path::Path::new(DIR))
        .and_then(|()| vfs.sync_dir(std::path::Path::new("/")))
        .and_then(|()| vfs.write_atomic(&file_of(doc), body.as_bytes()))
        .is_ok()
}

impl LayerFixture for MockLayerFixture {
    type Layer = MockLayer;

    fn new(vfs: CrashVfs) -> Self {
        Self::with_defect(vfs, Defect::None)
    }

    fn layer(&self) -> &MockLayer {
        &self.layer
    }

    fn seed(&self, n_scope: usize, n_neighbor: usize) {
        let s = &self.layer.0;
        // Verschränkte, deterministische IDs: Scope gerade, Nachbarn ungerade.
        let scoped: Vec<DocId> = (1..=n_scope as u64).map(|i| doc_id(i * 2)).collect();
        let neighbors: Vec<DocId> = (0..n_neighbor as u64).map(|i| doc_id(i * 2 + 1)).collect();
        for doc in scoped.iter().chain(&neighbors) {
            write_doc(&s.vfs, *doc);
        }
        for doc in &scoped {
            self.reader
                .data
                .lock()
                .insert(key_of(*doc), Bytes::from(format!("body-{}", doc.0)));
        }
        *s.scope_docs.lock() = scoped;
        *s.neighbors.lock() = neighbors;
    }

    fn scope(&self) -> ResolvedScope {
        let docs = self.layer.0.scope_docs.lock().iter().copied().collect();
        let mut scope = self.layer.0.scope.clone();
        scope.docs = vec![DocScope::Set {
            tenant: tenant(),
            collection: COLLECTION.to_string(),
            docs,
        }];
        scope
    }

    fn neighbor_docs(&self) -> Vec<DocId> {
        self.layer.0.neighbors.lock().clone()
    }

    fn neighbor_digest(&self) -> [u8; 32] {
        let neighbors = self.neighbor_docs();
        digest(
            neighbors
                .iter()
                .map(|d| self.layer.0.vfs.read(&file_of(*d)).ok()),
        )
    }

    fn state_digest(&self) -> [u8; 32] {
        let vfs = &self.layer.0.vfs;
        let mut parts: Vec<Option<Vec<u8>>> = Vec::new();
        for (path, state) in vfs.snapshot_tree().files {
            if path.ends_with("progress") {
                continue;
            }
            parts.push(Some(path.as_os_str().as_encoded_bytes().to_vec()));
            parts.push(state.volatile);
        }
        if self.layer.0.defect == Defect::NondeterministicOutput {
            parts.push(Some(self.layer.0.instance.to_le_bytes().to_vec()));
        }
        digest(parts.into_iter())
    }

    fn plant_residue(&self) {
        let first = self.layer.0.scope_docs.lock().first().copied();
        if let Some(doc) = first {
            write_doc(&self.layer.0.vfs, doc);
        }
    }

    fn inject_fault(&self, f: FaultSpec) {
        *self.layer.0.fault.lock() = Some(f);
    }

    fn reader(&self) -> Arc<dyn PrivilegedRead> {
        let reader = MockPrivilegedRead::new(self.layer.0.scope.keys.clone());
        *reader.data.lock() = self.reader.data.lock().clone();
        reader.insert(self.foreign_key(), Bytes::from_static(b"foreign"));
        Arc::new(reader)
    }

    fn foreign_key(&self) -> Vec<u8> {
        b"other/1".to_vec()
    }

    fn gate(&self) -> Arc<dyn ReadGate> {
        Arc::new(crate::gates::AlwaysRestrictedGate)
    }

    fn is_visible(&self, doc: DocId) -> bool {
        !self.layer.0.hidden.lock().contains(&doc) && self.layer.present(doc)
    }

    fn direct_blocked_reads(&self) -> u64 {
        self.layer.0.direct_blocked_reads.load(Ordering::SeqCst)
    }
}

/// 32-Byte-Digest ohne Kryptoabhängigkeit (vier unabhängige FNV-1a-Spuren, nur für Tests).
fn digest(parts: impl Iterator<Item = Option<Vec<u8>>>) -> [u8; 32] {
    let mut lanes = [0xCBF2_9CE4_8422_2325u64, 1, 2, 3];
    for part in parts {
        let (tag, bytes) = match part {
            Some(b) => (1u8, b),
            None => (0u8, Vec::new()),
        };
        let framed = std::iter::once(tag)
            .chain((bytes.len() as u64).to_le_bytes())
            .chain(bytes);
        for b in framed {
            for lane in &mut lanes {
                *lane = (*lane ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3);
            }
        }
    }
    let mut out = [0u8; 32];
    for (chunk, lane) in out.chunks_mut(8).zip(lanes) {
        chunk.copy_from_slice(&lane.to_le_bytes());
    }
    out
}

/// Frische Referenz-Fixture mit festem Seed.
pub fn mock_fixture(defect: Defect) -> MockLayerFixture {
    MockLayerFixture::with_defect(CrashVfs::new(CrashSeed(0x5EED)), defect)
}
