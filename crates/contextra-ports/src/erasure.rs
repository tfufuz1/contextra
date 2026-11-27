//! Erasure chain port definitions, scope resolution, budgets, and kernel contracts (WP-02 §4.2–4.6).
//!
//! Defines [`KeyScope`], [`DocScope`], [`ResolvedScope`], [`IoBudget`], [`ProgressSink`],
//! [`Cancel`], [`PurgeCx`], [`PrivilegedRead`], [`ErasureLayer`], and [`ErasureKernel`].

use crate::clock::Clock;
use crate::gate::ReadGate;
use crate::scanner::{LayerScanner, LayerWitness, WitnessLayer};
use crate::BoxFuture;
use bytes::Bytes;
pub use contextra_types::{
    Cursor, DocId, EraseTarget, ErasureError, ErasureException, ErasureId, ErasurePolicy,
    ErasureReason, ErasureState, ErasureStatus, Guarantee, LayerErrorKind, LayerProgress, Seq,
    TargetKind, TenantId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::sync::Arc;

/// Pattern matching raw storage keys in a [`KeyScope`].
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KeyPattern {
    /// Matches any key starting with the given byte prefix.
    Prefix(Vec<u8>),
    /// Matches only the exact byte sequence.
    Exact(Vec<u8>),
}

impl KeyPattern {
    /// Returns the underlying pattern bytes.
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Prefix(b) | Self::Exact(b) => b.as_slice(),
        }
    }

    /// Returns `true` if `raw_key` matches this pattern.
    pub fn matches(&self, raw_key: &[u8]) -> bool {
        match self {
            Self::Prefix(p) => raw_key.starts_with(p),
            Self::Exact(e) => raw_key == e.as_slice(),
        }
    }

    fn kind_tag(&self) -> u8 {
        match self {
            Self::Prefix(_) => 1,
            Self::Exact(_) => 2,
        }
    }
}

impl fmt::Debug for KeyPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact key bytes (TST-PORTS-07)
        match self {
            Self::Prefix(b) => write!(f, "KeyPattern::Prefix(len={})", b.len()),
            Self::Exact(b) => write!(f, "KeyPattern::Exact(len={})", b.len()),
        }
    }
}

/// Scope inclusion verdict for a [`KeyPattern`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Verdict {
    /// Keys matching this pattern are included in the erasure scope.
    In,
    /// Keys matching this pattern are excluded from the erasure scope.
    Out,
}

impl Verdict {
    const fn tag(self) -> u8 {
        match self {
            Self::Out => 0,
            Self::In => 1,
        }
    }
}

/// Canonical rule-based key scope using longest-match-wins semantics (WP-02 §4.2, Index §3.1).
///
/// - Longest matching pattern wins.
/// - For equal byte length, [`KeyPattern::Exact`] beats [`KeyPattern::Prefix`].
/// - No matching rule means not in scope (`false`).
#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeyScope {
    rules: Vec<(KeyPattern, Verdict)>,
}

impl KeyScope {
    /// Maximum number of rules permitted in a single [`KeyScope`].
    pub const MAX_RULES: usize = 4096;

    /// Maximum total pattern byte length permitted in a single [`KeyScope`] (1 MiB).
    pub const MAX_BYTES: usize = 1 << 20;

    /// Constructs a validated, canonically sorted and deduplicated [`KeyScope`].
    ///
    /// # Fehler
    /// Liefert [`ErasureError::InvalidTarget`], wenn die Regelanzahl [`Self::MAX_RULES`] oder
    /// die Summe der Musterbytes [`Self::MAX_BYTES`] überschreitet, oder
    /// [`ErasureError::ScopeAmbiguous`], wenn dasselbe Muster widersprüchliche Urteile trägt.
    ///
    /// # Crash
    /// Reine Validierung und Sortierung im Arbeitsspeicher; panikfrei.
    pub fn new(mut rules: Vec<(KeyPattern, Verdict)>) -> Result<Self, ErasureError> {
        if rules.len() > Self::MAX_RULES {
            return Err(ErasureError::InvalidTarget(
                "key scope exceeds maximum rule count (4096)",
            ));
        }

        let mut total_bytes: usize = 0;
        for (pat, _) in &rules {
            total_bytes = total_bytes
                .checked_add(pat.bytes().len())
                .ok_or(ErasureError::InvalidTarget("key scope byte size overflow"))?;
            if total_bytes > Self::MAX_BYTES {
                return Err(ErasureError::InvalidTarget(
                    "key scope exceeds maximum pattern bytes (1 MiB)",
                ));
            }
        }

        rules.sort_by(|(pa, va), (pb, vb)| {
            pa.kind_tag()
                .cmp(&pb.kind_tag())
                .then_with(|| pa.bytes().cmp(pb.bytes()))
                .then_with(|| va.cmp(vb))
        });
        rules.dedup();

        // Reject conflicting verdicts for the exact same pattern.
        for pair in rules.windows(2) {
            if let [a, b] = pair {
                if a.0 == b.0 && a.1 != b.1 {
                    return Err(ErasureError::ScopeAmbiguous);
                }
            }
        }

        Ok(Self { rules })
    }

    /// Evaluates whether `raw_key` belongs to this scope.
    pub fn contains(&self, raw_key: &[u8]) -> bool {
        let mut best: Option<((usize, bool), Verdict)> = None;
        for (pattern, verdict) in &self.rules {
            if pattern.matches(raw_key) {
                let is_exact = matches!(pattern, KeyPattern::Exact(_));
                let spec = (pattern.bytes().len(), is_exact);
                match best {
                    None => best = Some((spec, *verdict)),
                    Some((best_spec, _)) if spec > best_spec => best = Some((spec, *verdict)),
                    _ => {}
                }
            }
        }
        matches!(best, Some((_, Verdict::In)))
    }

    /// Returns the canonically sorted rules slice.
    pub fn rules(&self) -> &[(KeyPattern, Verdict)] {
        &self.rules
    }

    /// Conservative range overlap test for SSTable `[min, max]` key envelopes.
    ///
    /// False positives are permitted; false negatives are forbidden.
    pub fn could_overlap_range(&self, min: &[u8], max: &[u8]) -> bool {
        if min > max {
            return false;
        }
        for (pattern, verdict) in &self.rules {
            if *verdict != Verdict::In {
                continue;
            }
            match pattern {
                KeyPattern::Exact(e) => {
                    let s = e.as_slice();
                    if s >= min && s <= max {
                        return true;
                    }
                }
                KeyPattern::Prefix(p) => {
                    let s = p.as_slice();
                    if s <= max && (min < s || min.starts_with(s)) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl fmt::Debug for KeyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact key bytes (TST-PORTS-07)
        write!(f, "KeyScope {{ rules: {} }}", self.rules.len())
    }
}

/// Document-level scope for secondary indices (text, vector, graph).
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocScope {
    /// Explicit set of document IDs within a collection.
    Set {
        /// Tenant ID.
        tenant: TenantId,
        /// Collection name without internal prefix.
        collection: String,
        /// Sorted set of document IDs.
        docs: BTreeSet<DocId>,
    },
    /// Entire collection of a tenant.
    Collection {
        /// Tenant ID.
        tenant: TenantId,
        /// Collection name without internal prefix.
        collection: String,
    },
    /// Entire tenant.
    Tenant {
        /// Tenant ID.
        tenant: TenantId,
    },
}

impl DocScope {
    fn tag(&self) -> u8 {
        match self {
            Self::Set { .. } => 1,
            Self::Collection { .. } => 2,
            Self::Tenant { .. } => 3,
        }
    }
}

impl fmt::Debug for DocScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Redact collection name and key material (TST-PORTS-07)
        match self {
            Self::Set { tenant, docs, .. } => f
                .debug_struct("DocScope::Set")
                .field("tenant", &tenant.as_u64())
                .field("collection", &"[REDACTED]")
                .field("docs_count", &docs.len())
                .finish(),
            Self::Collection { tenant, .. } => f
                .debug_struct("DocScope::Collection")
                .field("tenant", &tenant.as_u64())
                .field("collection", &"[REDACTED]")
                .finish(),
            Self::Tenant { tenant } => f
                .debug_struct("DocScope::Tenant")
                .field("tenant", &tenant.as_u64())
                .finish(),
        }
    }
}

fn push_u32_len_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    hasher.update(&len.to_le_bytes());
    hasher.update(bytes);
}

/// Computes the canonical BLAKE3 `plan_hash` for an erasure plan (WP-02 §5).
///
/// Format:
/// `BLAKE3(b"ctx-erase-plan-v1" || target.canonical_bytes() || u32 n_rules || sorted rules || u32 n_docs || sorted docs || u64 unprovenanced)`
pub fn compute_plan_hash(
    target: &EraseTarget,
    keys: &KeyScope,
    docs: &[DocScope],
    unprovenanced: u64,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"ctx-erase-plan-v1");
    hasher.update(&target.canonical_bytes());

    let rules = keys.rules();
    let n_rules = u32::try_from(rules.len()).unwrap_or(u32::MAX);
    hasher.update(&n_rules.to_le_bytes());
    for (pattern, verdict) in rules {
        hasher.update(&[verdict.tag(), pattern.kind_tag()]);
        push_u32_len_bytes(&mut hasher, pattern.bytes());
    }

    let mut sorted_docs: Vec<&DocScope> = docs.iter().collect();
    sorted_docs.sort();
    let n_docs = u32::try_from(sorted_docs.len()).unwrap_or(u32::MAX);
    hasher.update(&n_docs.to_le_bytes());
    for doc_scope in sorted_docs {
        hasher.update(&[doc_scope.tag()]);
        match doc_scope {
            DocScope::Set {
                tenant,
                collection,
                docs: doc_ids,
            } => {
                hasher.update(&tenant.as_u64().to_le_bytes());
                push_u32_len_bytes(&mut hasher, collection.as_bytes());
                let n = u32::try_from(doc_ids.len()).unwrap_or(u32::MAX);
                hasher.update(&n.to_le_bytes());
                for doc_id in doc_ids {
                    hasher.update(&doc_id.0.to_le_bytes());
                }
            }
            DocScope::Collection { tenant, collection } => {
                hasher.update(&tenant.as_u64().to_le_bytes());
                push_u32_len_bytes(&mut hasher, collection.as_bytes());
            }
            DocScope::Tenant { tenant } => {
                hasher.update(&tenant.as_u64().to_le_bytes());
            }
        }
    }

    hasher.update(&unprovenanced.to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// Fully resolved erasure plan shared across the kernel and all storage layers.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedScope {
    /// Unique erasure job identifier.
    pub id: ErasureId,
    /// Target kind classification.
    pub target_kind: TargetKind,
    /// Canonical key scope rules.
    pub keys: KeyScope,
    /// Document-level scopes for secondary indices.
    pub docs: Vec<DocScope>,
    /// BLAKE3 hash over the canonical plan serialization.
    pub plan_hash: [u8; 32],
    /// Count of legacy objects lacking `derived_from` provenance.
    pub unprovenanced: u64,
}

impl ResolvedScope {
    /// Creates a [`ResolvedScope`] and computes its canonical [`Self::plan_hash`].
    pub fn new(
        id: ErasureId,
        target_kind: TargetKind,
        target: &EraseTarget,
        keys: KeyScope,
        mut docs: Vec<DocScope>,
        unprovenanced: u64,
    ) -> Self {
        docs.sort();
        docs.dedup();
        let plan_hash = compute_plan_hash(target, &keys, &docs, unprovenanced);
        Self {
            id,
            target_kind,
            keys,
            docs,
            plan_hash,
            unprovenanced,
        }
    }
}

impl fmt::Debug for ResolvedScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print key bytes (TST-PORTS-07)
        f.debug_struct("ResolvedScope")
            .field("id", &self.id)
            .field("target_kind", &self.target_kind)
            .field("keys_rules", &self.keys.rules().len())
            .field("docs_count", &self.docs.len())
            .field("unprovenanced", &self.unprovenanced)
            .finish()
    }
}

/// I/O scheduling class for budget arbitration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IoClass {
    /// Foreground user queries and writes (highest priority).
    Interactive,
    /// Background physical erasure.
    Erase,
    /// Background LSM compaction.
    Compaction,
}

/// System I/O pressure signal for cooperative throttling.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PressureHint {
    /// Normal load.
    Normal,
    /// Elevated load; background work should throttle.
    Elevated,
    /// Critical load; `Erase` and `Compaction` pause while `Interactive` continues.
    Critical,
}

/// Cooperative I/O budget port for throttling background purge and compaction.
pub trait IoBudget: Send + Sync {
    /// Waits cooperatively (clock-based) until `bytes` are granted. Must be cancel-safe.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::Cancelled`] bei Abbruch oder Budget-Fehler.
    /// # Crash
    /// Reine Ratenbegrenzung im RAM; keine persistente Mutation.
    fn acquire<'a>(&'a self, class: IoClass, bytes: u64)
        -> BoxFuture<'a, Result<(), ErasureError>>;

    /// Returns the current I/O pressure hint.
    ///
    /// # Fehler
    /// Keine; synchroner Statusabruf.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn pressure(&self) -> PressureHint;
}

/// Durable write-ahead progress sink implemented by the erasure kernel.
pub trait ProgressSink: Send + Sync {
    /// Records durable progress for a layer during purge execution.
    ///
    /// # Fehler
    /// Liefert [`ErasureError`] bei Journal- oder Persistierungsfehlern.
    /// # Crash
    /// Write-Ahead-Protokollierung; ein Crash vor Rückkehr lässt den vorherigen Cursor aktiv.
    fn record<'a>(
        &'a self,
        layer: &'a WitnessLayer,
        p: &'a LayerProgress,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;
}

/// Cancellation token port decoupled from `tokio_util` (WP-02 §4.3, O-P1).
pub trait Cancel: Send + Sync {
    /// Returns `true` if cancellation has been requested.
    ///
    /// # Fehler
    /// Keine; synchroner Zustandscheck.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn is_cancelled(&self) -> bool;

    /// Completes when cancellation is requested.
    ///
    /// # Fehler
    /// Keine; wartet kooperativ bis zum Abbruch.
    /// # Crash
    /// Keine persistente Mutation.
    fn cancelled(&self) -> BoxFuture<'_, ()>;
}

/// Execution context passed to layer purgers.
pub struct PurgeCx<'a> {
    /// Identifier of the active erasure job.
    pub erasure_id: ErasureId,
    /// Deterministic clock source.
    pub clock: &'a dyn Clock,
    /// Cooperative cancellation signal.
    pub cancel: &'a dyn Cancel,
    /// Cooperative I/O budget.
    pub budget: &'a dyn IoBudget,
    /// Optional resume cursor from the last durable checkpoint.
    pub cursor: Option<Cursor>,
    /// Gate-bypassing reader scoped strictly to this job.
    pub reader: &'a dyn PrivilegedRead,
    /// Durable progress sink.
    pub progress: &'a dyn ProgressSink,
    /// Maximum units to process per invocation (tick).
    pub max_units: usize,
}

/// Privileged read port bypassing the active [`ReadGate`] exclusively for the active purger.
///
/// Implementations must enforce that every accessed key satisfies `ResolvedScope.keys.contains`,
/// returning `ErasureError::Layer { kind: LayerErrorKind::ScopeViolation, .. }` otherwise (I-8),
/// and must reject all calls once the job reaches `Sealed`.
pub trait PrivilegedRead: Send + Sync {
    /// Reads a raw key bypassing the gate, subject to scope verification.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::Layer`] mit [`LayerErrorKind::ScopeViolation`] bei Scope-Verstoß
    /// oder nach `Sealed`, sowie I/O-Fehler bei Speicherproblemen.
    /// # Crash
    /// Reiner Lesezugriff ohne Seiteneffekte.
    fn get<'a>(&'a self, raw_key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>, ErasureError>>;

    /// Scans up to `limit` (capped at 1024) key-value pairs under `prefix` after `after`.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::Layer`] mit [`LayerErrorKind::ScopeViolation`] bei Scope-Verstoß
    /// oder nach `Sealed`, sowie I/O-Fehler bei Speicherproblemen.
    /// # Crash
    /// Reiner Lesezugriff ohne Seiteneffekte.
    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
        after: Option<&'a [u8]>,
        limit: usize,
    ) -> BoxFuture<'a, Result<Vec<(Bytes, Bytes)>, ErasureError>>;
}

/// Result of a resumable layer purge tick.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PurgeOutcome {
    /// Progress snapshot (`cursor == None` indicates layer completion).
    pub progress: LayerProgress,
    /// Units durably committed during this invocation.
    pub units_committed: u64,
    /// Physical bytes freed during this invocation.
    pub bytes_freed: u64,
}

/// Structured error returned by an [`ErasureLayer`] implementation.
#[derive(Clone, PartialEq, Eq, Debug, thiserror::Error)]
#[error("layer {layer:?} failed ({kind:?}): {detail}")]
pub struct LayerError {
    /// Retry classification.
    pub kind: LayerErrorKind,
    /// Layer that produced the error.
    pub layer: WitnessLayer,
    /// Static diagnostic detail (must never contain user key bytes).
    pub detail: &'static str,
}

impl From<LayerError> for ErasureError {
    fn from(err: LayerError) -> Self {
        ErasureError::Layer {
            layer: err.layer.as_str().to_string(),
            kind: err.kind,
        }
    }
}

/// Contract implemented by every secondary storage/index layer participating in erasure.
pub trait ErasureLayer: Send + Sync {
    /// Returns the layer identifier.
    ///
    /// # Fehler
    /// Keine; statische Schichtabfrage.
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn layer(&self) -> WitnessLayer;

    /// Performs fast (< 10 ms), idempotent RAM masking and cache eviction without file I/O.
    ///
    /// # Fehler
    /// Liefert [`LayerError`] falls In-Memory-Maskierung fehlschlägt.
    /// # Crash
    /// Reine RAM-Operation; bei Neustart wird `on_gate` aus dem durablen Journal erneut angestoßen.
    fn on_gate<'a>(&'a self, s: &'a ResolvedScope) -> BoxFuture<'a, Result<(), LayerError>>;

    /// Performs resumable physical purge using `cx.cursor`. Partial failures must return `Err`.
    /// Must never call `block_on`.
    ///
    /// # Fehler
    /// Liefert [`LayerError`] bei I/O-, Abbruch- oder Teilfehlern.
    /// # Crash
    /// Idempotent und über den zuletzt persistierten `Cursor` wiederaufnehmbar.
    fn purge<'a>(
        &'a self,
        s: &'a ResolvedScope,
        cx: &'a PurgeCx<'a>,
    ) -> BoxFuture<'a, Result<PurgeOutcome, LayerError>>;

    /// Returns the layer's verification scanner.
    ///
    /// # Fehler
    /// Keine; Referenzabruf.
    /// # Crash
    /// Reiner Zugriffsabruf ohne Seiteneffekte.
    fn scanner(&self) -> &dyn LayerScanner;
}

/// Request to initiate an erasure job on the kernel.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EraseRequest {
    /// Unique job identifier.
    pub id: ErasureId,
    /// Target specification.
    pub target: EraseTarget,
    /// Structured reason without PII.
    pub reason: ErasureReason,
    /// Execution and taint policy.
    pub policy: ErasurePolicy,
}

/// Receipt returned by [`ErasureKernel::accept`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AcceptReceipt {
    /// Job identifier.
    pub id: ErasureId,
    /// `true` if an identical `(id, plan_hash)` job already existed.
    pub existing: bool,
}

/// Receipt returned by [`ErasureKernel::publish_gate`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GateReceipt {
    /// Job identifier.
    pub id: ErasureId,
    /// Sequence watermark at which the gate became active.
    pub gated_seq: Seq,
    /// Monotonic gate epoch after publication.
    pub epoch: u64,
}

/// Receipt returned by [`ErasureKernel::seal`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SealReceipt {
    /// Job identifier.
    pub id: ErasureId,
    /// Achieved guarantee level (downgraded if any layer is `NotMeasured`).
    pub guarantee: Guarantee,
    /// Documented exceptions affecting the guarantee.
    pub exceptions: Vec<ErasureException>,
    /// Optional audit ledger entry index.
    pub ledger_index: Option<u64>,
}

/// Snapshot view of an open erasure job for recovery and orchestration.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct JobView {
    /// Job identifier.
    pub id: ErasureId,
    /// Current state machine state.
    pub state: ErasureState,
    /// Resolved erasure scope.
    pub scope: Arc<ResolvedScope>,
    /// Layers that have already completed purging.
    pub layers_done: Vec<WitnessLayer>,
    /// Current retry attempt counter.
    pub attempt: u32,
}

/// Kernel port implemented by the primary storage engine (`contextra-store`).
pub trait ErasureKernel: Send + Sync {
    /// Accepts and durably plans an erasure job. Idempotent over `(req.id, plan.plan_hash)`.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::IdempotencyConflict`] bei abweichendem `plan_hash`,
    /// sowie [`ErasureError::InvalidTarget`] oder Journal-Fehler.
    /// # Crash
    /// Persistiert den Plan durabel im Journal; vor Persistierung kein Zustandswechsel.
    fn accept<'a>(
        &'a self,
        req: &'a EraseRequest,
        plan: &'a ResolvedScope,
    ) -> BoxFuture<'a, Result<AcceptReceipt, ErasureError>>;

    /// Durably records `Gated` and publishes the active [`ReadGate`] snapshot.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::NotFound`], [`ErasureError::InvalidTransition`] oder Journal-Fehler.
    /// # Crash
    /// Kehrt erst nach durabler `Gated`-Markierung und Gate-Veröffentlichung zurück.
    fn publish_gate<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<GateReceipt, ErasureError>>;

    /// Executes resumable physical purge on the primary store layers (Memtable, SSTables, WAL).
    ///
    /// # Fehler
    /// Liefert [`ErasureError`] bei Abbruch, I/O- oder Zustandsfehlern.
    /// # Crash
    /// Idempotent und über `cx.cursor` wiederaufnehmbar.
    fn purge_store<'a>(
        &'a self,
        id: ErasureId,
        cx: &'a PurgeCx<'a>,
    ) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>>;

    /// Durably records progress for a participating layer.
    ///
    /// # Fehler
    /// Liefert [`ErasureError`] bei ungültiger ID, Übergang oder Journal-Fehlern.
    /// # Crash
    /// Persistiert Fortschritt im Journal vor Rückkehr.
    fn record_layer<'a>(
        &'a self,
        id: ErasureId,
        layer: &'a WitnessLayer,
        p: &'a LayerProgress,
    ) -> BoxFuture<'a, Result<(), ErasureError>>;

    /// Seals the job after collecting witnesses from all expected layers.
    /// Missing layers are recorded as `NotMeasured` and downgrade the guarantee (I-6).
    ///
    /// # Fehler
    /// Liefert [`ErasureError`] bei ungültigem Zustand oder Journal-Fehlern.
    /// # Crash
    /// Persistiert `Sealed` und Garantienachweis durabel.
    fn seal<'a>(
        &'a self,
        id: ErasureId,
        w: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<SealReceipt, ErasureError>>;

    /// Transitions a `Sealed` job with zero remaining units across all witnesses to `Completed`,
    /// releasing the gate lifetime (I-9).
    ///
    /// # Fehler
    /// Liefert [`ErasureError::InvalidTransition`], falls nicht `Sealed` oder `remaining > 0`.
    /// # Crash
    /// Persistiert `Completed` durabel vor Gate-Freigabe.
    fn complete<'a>(&'a self, id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>>;

    /// Records a failure transition for a job or specific layer.
    ///
    /// # Fehler
    /// Liefert [`ErasureError`] bei unbekannter ID oder Journal-Fehlern.
    /// # Crash
    /// Persistiert den Fehler- bzw. Quarantänezustand durabel.
    fn fail<'a>(
        &'a self,
        id: ErasureId,
        layer: Option<&'a WitnessLayer>,
        kind: LayerErrorKind,
    ) -> BoxFuture<'a, Result<ErasureState, ErasureError>>;

    /// Returns views of all non-terminal jobs.
    ///
    /// # Fehler
    /// Keine; synchroner Statusabruf.
    /// # Crash
    /// Reiner Lesezugriff auf den Kernelzustand.
    fn open_jobs(&self) -> Vec<JobView>;

    /// Returns the public status of a job if known.
    ///
    /// # Fehler
    /// Keine; synchroner Statusabruf.
    /// # Crash
    /// Reiner Lesezugriff auf den Kernelzustand.
    fn status(&self, id: ErasureId) -> Option<ErasureStatus>;

    /// Returns a scoped [`PrivilegedRead`] handle for the active job.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::NotFound`] oder [`ErasureError::InvalidTransition`] nach `Sealed`.
    /// # Crash
    /// Reine Handle-Erzeugung ohne Seiteneffekte.
    fn privileged_reader(&self, id: ErasureId) -> Result<Arc<dyn PrivilegedRead>, ErasureError>;

    /// Returns the store's active [`ReadGate`].
    ///
    /// # Fehler
    /// Keine; synchroner Handle-Abruf.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn gate(&self) -> Arc<dyn ReadGate>;

    /// Returns the store-internal verification scanners (Memtable, SSTables, WAL).
    ///
    /// # Fehler
    /// Keine; synchroner Handle-Abruf.
    /// # Crash
    /// Reiner RAM-Zugriff ohne Mutation.
    fn store_scanners(&self) -> Vec<Arc<dyn LayerScanner>>;
}
