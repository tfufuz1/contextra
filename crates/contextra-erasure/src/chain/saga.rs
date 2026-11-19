//! Die 6-Schritte-Löschkette nach SPEC §5.3 mit Recovery nach §5.5.
//!
//! Reihenfolge (normativ): Validierung & Intent (fsync #1) → Gate (fsync #2) →
//! Schlüsselvernichtung (fsync #3, `KeyRevoked`) → Purge (asynchron) →
//! Messung (Positivkontrolle, Residuen) → Bericht & Ledger (fsync #4, `Completed`).
//!
//! Fehlerpolitik (SPEC §11.1): Transiente I/O-Fehler setzen `Pending(IoBackoff)` mit
//! Resume-Ziel. Logische Fehler (Verifikation, Rückstände, fehlende Gate-Unterstützung)
//! lassen den Zustand unverändert und geben den Fehler zurück. Der Zustand wird erst
//! nach erfolgreichem Journal-Append (fsync) weitergeschaltet.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use contextra_ids::ErasureId;
use parking_lot::Mutex;

use crate::error::ErasureError;
use crate::gate::saga_gate::GateDecision;
use crate::types_local::{ControlEvidence, LayerWitness, MeasureClass, Scanner, Unarmed};

use super::journal::JournalRecord;
use super::layer::{ErasureLayer, LayerScope, PurgeCx};
use super::ports::{
    CommitBarrier, DiskReserve, GateMode, GateSink, Journal, KeyVault, PositiveControl, ReportSink,
};
use super::retention::RetentionAttestation;
use super::state::{PendingReason, SagaState};
use super::target::EraseTarget;

/// Abhängigkeiten der Kette. Schichten werden nach `priority` aufsteigend gepurgt (SPEC §5.3 Schritt 4).
pub struct ChainDeps {
    /// Durables Journal.
    pub journal: Arc<dyn Journal>,
    /// ReadGate-Port.
    pub gate: Arc<dyn GateSink>,
    /// Schlüsseltresor.
    pub keys: Arc<dyn KeyVault>,
    /// Schichten mit Purge-Priorität (kleiner zuerst).
    pub layers: Vec<(u16, Arc<dyn ErasureLayer>)>,
    /// Plattenreserve.
    pub disk: Arc<dyn DiskReserve>,
    /// Mindestreserve in Bytes (SPEC: 1 GB).
    pub disk_required_bytes: u64,
    /// Positivkontrolle für die Messung.
    pub canary: Arc<dyn PositiveControl>,
    /// Bericht und Ledger.
    pub report: Arc<dyn ReportSink>,
    /// Schlüssel für die keyed-BLAKE3-Duplikaterkennung (SPEC §5.3 Schritt 1.2).
    pub dedupe_key: [u8; 32],
}

/// Rückgabe an den Aufrufer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// Saga-Kennung.
    pub id: ErasureId,
    /// Aktueller Zustand.
    pub state: SagaState,
}

#[derive(Debug, Clone)]
struct Saga {
    id: ErasureId,
    target: EraseTarget,
    attestation: RetentionAttestation,
    state: SagaState,
    resume: Option<SagaState>,
    digest: [u8; 32],
}

impl Saga {
    fn receipt(&self) -> Receipt {
        Receipt {
            id: self.id,
            state: self.state,
        }
    }
}

struct Inner {
    sagas: BTreeMap<[u8; 16], Saga>,
    next_seq: u64,
}

/// Die Löschkette.
pub struct Chain {
    deps: ChainDeps,
    barrier: CommitBarrier,
    inner: Mutex<Inner>,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn invalid(from: SagaState, to: SagaState) -> ErasureError {
    ErasureError::InvalidTransition {
        from: format!("{from:?}"),
        to: format!("{to:?}"),
    }
}

impl Chain {
    /// Erstellt die Kette. Schichten werden nach Priorität sortiert.
    pub fn new(mut deps: ChainDeps) -> Self {
        deps.layers.sort_by_key(|(p, _)| *p);
        Self {
            deps,
            barrier: CommitBarrier::new(),
            inner: Mutex::new(Inner {
                sagas: BTreeMap::new(),
                next_seq: 1,
            }),
        }
    }

    /// Commit-Barriere (für Writer-Seite des Stores).
    pub fn barrier(&self) -> &CommitBarrier {
        &self.barrier
    }

    /// Aktueller Zustand einer Saga.
    pub fn status(&self, id: ErasureId) -> Result<SagaState, ErasureError> {
        Ok(self.snapshot(id)?.state)
    }

    fn digest(&self, target: &EraseTarget) -> Result<[u8; 32], ErasureError> {
        Ok(*blake3::keyed_hash(&self.deps.dedupe_key, &target.encode()?).as_bytes())
    }

    fn snapshot(&self, id: ErasureId) -> Result<Saga, ErasureError> {
        self.inner
            .lock()
            .sagas
            .get(&id.0)
            .cloned()
            .ok_or(ErasureError::Internal("unknown erasure id".into()))
    }

    fn layers(&self) -> Vec<Arc<dyn ErasureLayer>> {
        self.deps
            .layers
            .iter()
            .map(|(_, l)| Arc::clone(l))
            .collect()
    }

    fn open_saga_for(&self, digest: [u8; 32]) -> Option<Saga> {
        self.inner
            .lock()
            .sagas
            .values()
            .find(|s| s.digest == digest && !s.state.is_terminal())
            .cloned()
    }

    /// Schreibt einen Zustandsübergang durabel (fsync) und schaltet erst danach um.
    fn persist(
        &self,
        id: ErasureId,
        next: SagaState,
        resume: Option<SagaState>,
    ) -> Result<(), ErasureError> {
        let mut inner = self.inner.lock();
        let saga = inner
            .sagas
            .get(&id.0)
            .cloned()
            .ok_or(ErasureError::Internal("unknown erasure id".into()))?;
        if !saga.state.can_transition_to(next) {
            return Err(invalid(saga.state, next));
        }
        let resume = match next {
            SagaState::Pending(_) => Some(resume.ok_or(ErasureError::Corrupt)?),
            _ => None,
        };
        let record = JournalRecord {
            seq: inner.next_seq,
            erasure_id: id,
            target: saga.target.clone(),
            attestation: saga.attestation.clone(),
            state: next,
            resume,
        };
        self.deps.journal.append(&record)?;
        inner.next_seq = inner.next_seq.saturating_add(1);
        if let Some(s) = inner.sagas.get_mut(&id.0) {
            s.state = next;
            s.resume = resume;
        }
        Ok(())
    }

    /// Fehlerpolitik (SPEC §11.1): transiente I/O-Fehler werden als `Pending(IoBackoff)` festgehalten.
    fn fail(&self, id: ErasureId, err: ErasureError) -> ErasureError {
        if matches!(err, ErasureError::Io(_) | ErasureError::Storage(_)) {
            if let Ok(saga) = self.snapshot(id) {
                if !saga.state.is_terminal() && !matches!(saga.state, SagaState::Pending(_)) {
                    if let Err(journal_err) = self.persist(
                        id,
                        SagaState::Pending(PendingReason::IoBackoff),
                        Some(saga.state),
                    ) {
                        tracing::error!(error = %journal_err, "journal write failed while recording pending state");
                    }
                }
            }
        }
        err
    }

    /// Erfasst eine neue Saga und schreibt den Intent (fsync #1).
    fn register(
        &self,
        id: ErasureId,
        target: EraseTarget,
        attestation: RetentionAttestation,
    ) -> Result<Saga, ErasureError> {
        let digest = self.digest(&target)?;
        let saga = Saga {
            id,
            target,
            attestation,
            state: SagaState::Requested,
            resume: None,
            digest,
        };
        {
            let mut inner = self.inner.lock();
            if inner.sagas.contains_key(&id.0) {
                return Err(ErasureError::Internal(
                    "erasure id already registered".into(),
                ));
            }
            let record = JournalRecord {
                seq: inner.next_seq,
                erasure_id: id,
                target: saga.target.clone(),
                attestation: saga.attestation.clone(),
                state: SagaState::Requested,
                resume: None,
            };
            self.deps.journal.append(&record)?;
            inner.next_seq = inner.next_seq.saturating_add(1);
            inner.sagas.insert(id.0, saga.clone());
        }
        self.snapshot(id)
    }

    /// Schritt 1: Validierung und Intent. Liefert idempotent eine laufende Saga zurück.
    pub async fn erase(
        &self,
        id: ErasureId,
        target: EraseTarget,
        attestation: RetentionAttestation,
    ) -> Result<Receipt, ErasureError> {
        let digest = self.digest(&target)?;
        if let Some(open) = self.open_saga_for(digest) {
            return match open.state {
                // Art.-18-Einschränkung wird zur Löschung eskaliert (Restricted → Gated).
                SagaState::Restricted => self.escalate(open.id).await,
                _ => Ok(open.receipt()),
            };
        }

        // Schritt 1.1: Aufbewahrungspflicht prüfen.
        if attestation.code.blocks_erasure() {
            let saga = self.register(id, target, attestation)?;
            self.persist(saga.id, SagaState::Refused, None)?;
            return Ok(self.snapshot(id)?.receipt());
        }

        // Schritt 1.4: Requested schreiben (fsync #1).
        self.register(id, target, attestation)?;

        // Schritt 1.3: Layer-Geltungsbereiche ermitteln (Fehler ⇒ noch nichts gesperrt).
        if let Err(e) = self.resolve_all(id).await {
            return Err(self.fail(id, e));
        }

        self.gate_step(id)?;
        self.shred_step(id).await
    }

    /// Schritt 2: Gate setzen unter exklusiver Commit-Barriere, dann `Gated` durabel schreiben.
    fn gate_step(&self, id: ErasureId) -> Result<(), ErasureError> {
        let saga = self.snapshot(id)?;
        let from_restricted = saga.state == SagaState::Restricted;
        let _excl = self.barrier.exclusive();
        let activated = self
            .deps
            .gate
            .activate(&saga.target, GateMode::Erased)
            .and_then(|()| {
                if from_restricted {
                    self.deps.gate.lift_restricted(&saga.target)
                } else {
                    Ok(())
                }
            });
        if let Err(e) = activated {
            return Err(self.fail(id, e));
        }
        self.persist(id, SagaState::Gated, None)
    }

    /// Schritt 3: `on_gate`, Schlüsselvernichtung, Verifikation, dann `KeyRevoked`.
    async fn shred_step(&self, id: ErasureId) -> Result<Receipt, ErasureError> {
        let saga = self.snapshot(id)?;
        let scopes = match self.resolve_all(id).await {
            Ok(s) => s,
            Err(e) => return Err(self.fail(id, e)),
        };
        for (layer, scope) in &scopes {
            if let Err(e) = layer.on_gate(scope).await {
                return Err(self.fail(id, e));
            }
        }
        if let Err(e) = self.deps.keys.destroy(&saga.target) {
            return Err(self.fail(id, e));
        }
        // Bei Fehlschlag bleibt der Zustand `Gated` (SPEC §11.1).
        match self.deps.keys.verify_absent(&saga.target) {
            Ok(true) => {}
            Ok(false) => return Err(ErasureError::ShredVerificationFailed),
            Err(e) => return Err(self.fail(id, e)),
        }
        self.persist(id, SagaState::KeyRevoked, None)?;
        Ok(self.snapshot(id)?.receipt())
    }

    /// Eskalation einer Einschränkung zur Löschung.
    async fn escalate(&self, id: ErasureId) -> Result<Receipt, ErasureError> {
        self.gate_step(id)?;
        self.shred_step(id).await
    }

    /// Schritt 4: Physische Bereinigung in Prioritätsreihenfolge (idempotent).
    pub async fn purge(&self, id: ErasureId, cx: PurgeCx) -> Result<Receipt, ErasureError> {
        let saga = self.snapshot(id)?;
        match saga.state {
            SagaState::KeyRevoked => self.persist(id, SagaState::Purging, None)?,
            SagaState::Purging => {}
            // Zweiter Aufruf nach Abschluss der Bereinigung ist ein No-op (T-DEL-14).
            SagaState::Verifying | SagaState::Completed => return Ok(saga.receipt()),
            other => return Err(invalid(other, SagaState::Purging)),
        }

        if self.deps.disk.available_bytes() < self.deps.disk_required_bytes {
            self.persist(
                id,
                SagaState::Pending(PendingReason::DiskReserveLow),
                Some(SagaState::Purging),
            )?;
            return Err(ErasureError::DiskReserve);
        }

        for layer in self.layers() {
            let scope = match layer.resolve(&saga.target).await {
                Ok(s) => s,
                Err(e) => return Err(self.fail(id, e)),
            };
            if let Err(e) = layer.purge(&scope, &cx).await {
                return Err(self.fail(id, e));
            }
        }
        self.persist(id, SagaState::Verifying, None)?;
        Ok(self.snapshot(id)?.receipt())
    }

    /// Schritt 5 und 6: Positivkontrolle, Residuenmessung, Lesepfad-Test, Bericht, Abschluss.
    pub async fn verify(&self, id: ErasureId) -> Result<Receipt, ErasureError> {
        let saga = self.snapshot(id)?;
        if saga.state != SagaState::Verifying {
            return Err(invalid(saga.state, SagaState::Completed));
        }

        // Schritt 5.2: Positivkontrolle. Ohne gefundenen Canary ist die Messung ungültig.
        let observed = match self.deps.canary.canary_observed() {
            Ok(v) => v,
            Err(e) => return Err(self.fail(id, e)),
        };
        if !observed {
            return Err(ErasureError::Internal(
                "positive control: canary not observed".into(),
            ));
        }

        // Schritt 5.5: Witnesses je Schicht, jede muss `residue_matches == 0` melden.
        let mut witnesses: Vec<LayerWitness> = Vec::new();
        for layer in self.layers() {
            let scope: LayerScope = match layer.resolve(&saga.target).await {
                Ok(s) => s,
                Err(e) => return Err(self.fail(id, e)),
            };
            let mut armed = Scanner::<Unarmed>::new(layer.id(), MeasureClass::Mk1, 0)
                .arm(ControlEvidence::observed());
            let started = Instant::now();
            let residue = match layer.witness(&scope, &mut armed).await {
                Ok(r) => r,
                Err(e) => return Err(self.fail(id, e)),
            };
            let duration: Duration = started.elapsed();
            let hash = witness_hash(&layer.id().0, &scope.scope_id, residue);
            match armed.finish(residue, duration, hash) {
                Ok(w) => witnesses.push(w),
                Err(r) => return Err(ErasureError::Residue(r.matches)),
            }
        }

        // Schritt 5.4: Lesepfad muss `Erased` liefern.
        if self.deps.gate.decision(&saga.target) != GateDecision::Erased {
            return Err(ErasureError::Internal(
                "read path does not report Erased".into(),
            ));
        }

        // Schritt 6: Bericht signieren und Ledger schreiben (fsync #4), dann Abschluss.
        if let Err(e) = self
            .deps
            .report
            .publish(id, &saga.target, &saga.attestation, &witnesses)
        {
            return Err(self.fail(id, e));
        }
        self.persist(id, SagaState::Completed, None)?;
        Ok(self.snapshot(id)?.receipt())
    }

    /// Restrict-Modus (Art. 18): Gate als `Restricted`, keine Vernichtung, kein Purge.
    pub fn restrict(
        &self,
        id: ErasureId,
        target: EraseTarget,
        attestation: RetentionAttestation,
    ) -> Result<Receipt, ErasureError> {
        let digest = self.digest(&target)?;
        if let Some(open) = self.open_saga_for(digest) {
            return Ok(open.receipt());
        }
        self.register(id, target, attestation)?;
        let saga = self.snapshot(id)?;
        {
            let _excl = self.barrier.exclusive();
            if let Err(e) = self.deps.gate.activate(&saga.target, GateMode::Restricted) {
                return Err(self.fail(id, e));
            }
            self.persist(id, SagaState::Restricted, None)?;
        }
        Ok(self.snapshot(id)?.receipt())
    }

    /// Hebt eine Einschränkung auf (`Restricted → Lifted`, terminal).
    pub fn lift(&self, id: ErasureId) -> Result<Receipt, ErasureError> {
        let saga = self.snapshot(id)?;
        if saga.state != SagaState::Restricted {
            return Err(invalid(saga.state, SagaState::Lifted));
        }
        let _excl = self.barrier.exclusive();
        if let Err(e) = self.deps.gate.lift_restricted(&saga.target) {
            return Err(self.fail(id, e));
        }
        self.persist(id, SagaState::Lifted, None)?;
        Ok(self.snapshot(id)?.receipt())
    }

    /// Setzt eine Saga nach Unterbrechung fort (SPEC §5.5). Pending wird zuerst in den Resume-Zustand überführt.
    pub async fn resume(&self, id: ErasureId) -> Result<Receipt, ErasureError> {
        let saga = self.snapshot(id)?;
        let state = match saga.state {
            SagaState::Pending(_) => {
                let target = saga.resume.ok_or(ErasureError::Corrupt)?;
                self.persist(id, target, None)?;
                target
            }
            other => other,
        };
        match state {
            SagaState::Requested => {
                self.gate_step(id)?;
                self.shred_step(id).await
            }
            SagaState::Gated => self.shred_step(id).await,
            SagaState::KeyRevoked | SagaState::Purging => {
                self.purge(
                    id,
                    PurgeCx {
                        deadline_ms: now_ms().saturating_add(60_000),
                    },
                )
                .await?;
                self.verify(id).await
            }
            SagaState::Verifying => self.verify(id).await,
            _ => Ok(self.snapshot(id)?.receipt()),
        }
    }

    async fn resolve_all(
        &self,
        id: ErasureId,
    ) -> Result<Vec<(Arc<dyn ErasureLayer>, LayerScope)>, ErasureError> {
        let saga = self.snapshot(id)?;
        let mut out = Vec::new();
        for layer in self.layers() {
            let scope = layer.resolve(&saga.target).await?;
            out.push((layer, scope));
        }
        Ok(out)
    }

    /// Stellt nach einem Neustart den ReadGate wieder her und liefert alle offenen Sagas (SPEC §5.5).
    ///
    /// Gates für `Gated`, `KeyRevoked`, `Purging`, `Verifying` und `Completed` werden sofort aktiviert,
    /// damit kein gelöschtes Datum während der Wiederherstellung lesbar ist. `Restricted` wird
    /// als Einschränkung wiederhergestellt.
    pub fn recover(&self) -> Result<Vec<ErasureId>, ErasureError> {
        let records = self.deps.journal.load()?;
        let mut latest: BTreeMap<[u8; 16], JournalRecord> = BTreeMap::new();
        let mut max_seq = 0u64;
        for r in records {
            max_seq = max_seq.max(r.seq);
            latest.insert(r.erasure_id.0, r);
        }

        let mut open = Vec::new();
        let mut inner = self.inner.lock();
        inner.next_seq = max_seq.saturating_add(1);
        inner.sagas.clear();
        for (raw_id, r) in latest {
            let digest = self.digest(&r.target)?;
            let saga = Saga {
                id: ErasureId(raw_id),
                target: r.target,
                attestation: r.attestation,
                state: r.state,
                resume: r.resume,
                digest,
            };
            let effective = match saga.state {
                SagaState::Pending(_) => saga.resume,
                other => Some(other),
            };
            {
                let _excl = self.barrier.exclusive();
                match effective {
                    Some(
                        SagaState::Gated
                        | SagaState::KeyRevoked
                        | SagaState::Purging
                        | SagaState::Verifying
                        | SagaState::Completed,
                    ) => self.deps.gate.activate(&saga.target, GateMode::Erased)?,
                    Some(SagaState::Restricted) => self
                        .deps
                        .gate
                        .activate(&saga.target, GateMode::Restricted)?,
                    _ => {}
                }
            }
            if !saga.state.is_terminal() {
                open.push(saga.id);
            }
            inner.sagas.insert(raw_id, saga);
        }
        Ok(open)
    }
}

fn witness_hash(layer: &[u8; 16], scope: &[u8; 16], residue: u64) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(b"contextra.erasure.witness.v1");
    h.update(layer);
    h.update(scope);
    h.update(&residue.to_be_bytes());
    *h.finalize().as_bytes()
}
