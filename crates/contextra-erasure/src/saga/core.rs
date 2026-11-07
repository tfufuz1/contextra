//! Orchestrierung des Löschkerns via `ErasureCore` (SPEC §11.1, §11.2, §11.4, D-15, D-58, D-78, D-118, D-119, ER-18..24).
//!
//! Exakte Schritte 1–6 aus SPEC §11.2:
//! 1. Validierung & RetentionAttestation (Matrix D-124 Abs. 6), resolve je Schicht, Journal Requested+Intent (fsync #1).
//! 2. Gate setzen unter CommitBarrier::exclusive -> add_gate mit gated_seq (fsync #2).
//! 3. on_gate je Schicht, Schlüsselvernichtung höchster Ebene via SagaVault (fsync #3), Fehlen prüfen.
//! 4. Journal KeyDestroyed (fsync #4) -> Observer-Benachrichtigung Gated -> Rückgabe Gated.
//! 5. Purge-Fenster in Reihenfolge a–g:
//!    a) Snapshots widerrufen (SnapshotRevoker)
//!    b) Textsegment-Rewrite
//!    c) Memtable-Drain / WAL
//!    d) SSTable-Rewrite
//!    e) Vektor-Reparatur
//!    f) D-Slots / Slot-Bereinigung
//!    g) Caches
//!    Exklusiv gegen Kompaktion (PurgeBarrier), Plattenreserve vorab.
//! 6. Messung via SagaMeasure (matches != 0 oder Zeuge fehlt -> Pending(WitnessMissing)),
//!    signierter Bericht, Ledger, Journal Completed, Gate-Kompaktion -> Observer Completed.
//!
//! Cancel-Safety (ER-24): Jede mutierende Operation läuft in einer vom Aufrufer unabhängigen
//! Task (`tokio::spawn` + `oneshot`). Drop blockiert nie. Keine Locks über .await fremder Tasks.

use crate::error::ErasureError;
use crate::purge::order::PurgePriority;
use crate::purge::reserve::{check_disk_reserve, ReserveCheck};
use crate::saga::idempotency::derive_open_key;
use crate::saga::machine::{EraseSaga, SagaState};
use crate::saga::seams::{
    SagaGate, SagaJournal, SagaLedger, SagaMeasure, SagaReport, SagaVault, SnapshotRevoker,
};
use crate::types_local::{CommitBarrier, PurgeBarrier};
use crate::types_local::{
    EraseMode, EraseReceipt, ErasureStatus, LayerId as TypesLayerId, PendingReason, Phase,
    RefusalReason, RetentionAttestation, RetentionCode, Timing,
};
use crate::types_local::{EraseTarget, ErasureLayer, LayerId, LayerScope, PurgeCx};
use crate::types_local::{ErasureEvent, ErasureObserver, ObservedPhase, TargetKind};
use crate::types_local::{ErasureId, Timestamp};
use contextra_ports::clock::Clock;
use contextra_ports::id_gen::IdGen;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, RwLock};

/// Konfiguration für den Löschkern (SPEC §4, §11.2).
#[derive(Clone, Debug)]
pub struct ErasureConfig {
    /// Maximale Dauer bis zum Erreichen von Gated oder Pending.
    pub erase_return_timeout: Duration,
    /// SLA-Frist für die gesamte Löschung.
    pub erase_sla: Duration,
    /// Batch-Fenster für gebündelte Purge-Läufe.
    pub batch_window: Duration,
    /// Mindestanzahl freier Bytes auf dem Speichermedium vor dem Purge.
    pub min_free_bytes: u64,
    /// Index-Schlüssel für Open-Key Ableitung nach D-119.
    pub k_idx: [u8; 32],
}

impl Default for ErasureConfig {
    fn default() -> Self {
        Self {
            erase_return_timeout: Duration::from_millis(500),
            erase_sla: Duration::from_secs(86400),
            batch_window: Duration::from_secs(60),
            min_free_bytes: 1024 * 1024 * 1024, // 1 GiB
            k_idx: [0u8; 32],
        }
    }
}

/// Abhängigkeiten des Löschkerns (SPEC §4 Schnittstellen, Ports & Seams).
pub struct ErasureDeps {
    /// Intent-Journal Naht.
    pub journal: Arc<dyn SagaJournal>,
    /// Gate Naht.
    pub gate: Arc<dyn SagaGate>,
    /// Schlüsseltresor Naht.
    pub vault: Arc<dyn SagaVault>,
    /// Messung / Residuen-Scan Naht.
    pub measure: Arc<dyn SagaMeasure>,
    /// Bericht Naht.
    pub report: Arc<dyn SagaReport>,
    /// Ledger Naht.
    pub ledger: Arc<dyn SagaLedger>,
    /// Snapshot-Widerruf Naht.
    pub snapshots: Arc<dyn SnapshotRevoker>,
    /// Schichten (Storage, Text, Vektor).
    pub layers: Vec<Arc<dyn ErasureLayer>>,
    /// Feste Prioritäten der Schichten.
    pub layer_priorities: HashMap<[u8; 16], PurgePriority>,
    /// Commit-Barriere für Gate-Gruppen-Commit.
    pub commit_barrier: Arc<dyn CommitBarrier>,
    /// Purge-Barriere für exklusiven Purge gegen Kompaktion.
    pub purge_barrier: Arc<dyn PurgeBarrier>,
    /// System- oder Simulations-Uhr.
    pub clock: Arc<dyn Clock>,
    /// ID-Generator.
    pub ids: Arc<dyn IdGen>,
    /// Optionaler Beobachter für Phase-Übergänge (D-118).
    pub observer: Option<Arc<dyn ErasureObserver>>,
}

/// Antrag an den Löschkern.
#[derive(Clone, Debug)]
pub struct EraseExecutionRequest {
    /// Eindeutige ID (oder abgeleitet).
    pub erasure_id: ErasureId,
    /// Ziel des Löschvorgangs.
    pub target: EraseTarget,
    /// Modus (Erase oder Restrict).
    pub mode: EraseMode,
    /// Aufbewahrungs-Attestierung.
    pub attestation: RetentionAttestation,
    /// Zeitverhalten für das Purge-Fenster.
    pub timing: Timing,
}

/// Status eines laufenden oder abgeschlossenen Löschauftrags.
#[derive(Clone, Debug)]
pub struct ErasureCoreStatus {
    /// Die ID.
    pub id: ErasureId,
    /// Der aktuelle Zustand.
    pub state: SagaState,
    /// Detaillierter Status gemäß SPEC §7.4.
    pub status: ErasureStatus,
    /// Gated-Sequenz (falls erreicht).
    pub gated_seq: Option<u64>,
}

#[derive(Clone)]
struct ActiveEntry {
    saga: EraseSaga,
    target: EraseTarget,
    attestation: RetentionAttestation,
    scopes: Vec<(LayerId, LayerScope)>,
    gated_seq: u64,
}

impl ErasureCore {
    /// Hilfsfunktion zur Ermittlung des aktuellen Zeitstempels in Millisekunden.
    fn now_millis(clock: &dyn Clock) -> u64 {
        clock.now_unix_nanos() / 1_000_000
    }

    /// Kanonische Bytes für ein Ziel zwecks Open-Key-Ableitung (D-119).
    fn canonical_target_bytes(target: &EraseTarget) -> Vec<u8> {
        match target {
            EraseTarget::ExactKeys(keys) => {
                let mut out = Vec::new();
                for k in keys {
                    out.extend_from_slice(k);
                }
                out
            }
            EraseTarget::KeyRange { start, end } => {
                let mut out = Vec::new();
                out.extend_from_slice(start);
                out.extend_from_slice(end);
                out
            }
        }
    }
}

/// Haupt-Orchestrator des Löschkerns (SPEC §11.2, AUFTRAG J-07).
pub struct ErasureCore {
    cfg: ErasureConfig,
    deps: Arc<ErasureDeps>,
    active_sagas: Arc<RwLock<HashMap<ErasureId, ActiveEntry>>>,
    open_keys: Arc<RwLock<HashMap<[u8; 32], ErasureId>>>,
}

impl ErasureCore {
    /// Erstellt eine neue Instanz des Orchestrators.
    pub fn new(cfg: ErasureConfig, deps: ErasureDeps) -> Self {
        Self {
            cfg,
            deps: Arc::new(deps),
            active_sagas: Arc::new(RwLock::new(HashMap::new())),
            open_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gibt die Konfiguration zurück.
    pub fn config(&self) -> &ErasureConfig {
        &self.cfg
    }

    /// Gibt die Abhängigkeiten zurück.
    pub fn deps(&self) -> &Arc<ErasureDeps> {
        &self.deps
    }

    /// Benachrichtigt den Beobachter at-least-once ohne PII (D-118).
    /// Fehler des Observers stoppen die Saga nicht.
    async fn notify_observer(
        deps: &ErasureDeps,
        id: ErasureId,
        target: &EraseTarget,
        phase: ObservedPhase,
    ) {
        if let Some(obs) = &deps.observer {
            let target_kind = match target {
                EraseTarget::ExactKeys(k) if k.len() <= 1 => TargetKind::Document,
                _ => TargetKind::Collection,
            };
            let at = Self::now_millis(deps.clock.as_ref());
            let event = ErasureEvent {
                id,
                target_kind,
                phase,
                at,
            };
            let res = obs.on_erased(event).await;
            if let Err(err) = res {
                tracing::warn!(
                    code = "OBSERVER_DISPATCH_FAILED",
                    phase = ?phase,
                    error = %err,
                    "Observer notification failed without interrupting saga"
                );
            }
        }
    }

    /// Führt die Validierung der Retention-Attestierung gemäß Matrix D-124 Abs. 6 durch.
    pub fn evaluate_retention(
        mode: EraseMode,
        attestation: &RetentionAttestation,
    ) -> Result<RetentionDecision, ErasureError> {
        match attestation.code {
            RetentionCode::NoObligation
            | RetentionCode::ObligationExpired
            | RetentionCode::OperatorOverride => match mode {
                EraseMode::Erase => Ok(RetentionDecision::ProceedErase),
                EraseMode::Restrict => Ok(RetentionDecision::ProceedRestrict),
            },
            RetentionCode::ObligationActive | RetentionCode::LegalClaimsPending => match mode {
                EraseMode::Erase => Ok(RetentionDecision::Refuse(
                    RefusalReason::RetentionObligation,
                )),
                EraseMode::Restrict => Ok(RetentionDecision::ProceedRestrict),
            },
            RetentionCode::DataSubjectRequestedRestriction => match mode {
                EraseMode::Erase => Ok(RetentionDecision::Refuse(RefusalReason::InvalidParameters)),
                EraseMode::Restrict => Ok(RetentionDecision::ProceedRestrict),
            },
        }
    }

    /// Nimmt eine Löschung an und führt sie bis mindestens `Gated` aus (SPEC §11.2, ER-23).
    ///
    /// Cancel-Safety (ER-24): Die Ausführung läuft in einer unabhängigen Tokio-Task.
    /// Bricht das Aufrufer-Future ab, läuft die Löschung unbeeinflusst weiter.
    pub async fn erase(&self, req: EraseExecutionRequest) -> Result<EraseReceipt, ErasureError> {
        // Idempotenz offener Anträge nach D-119
        let mode_byte = match req.mode {
            EraseMode::Erase => 0x01,
            EraseMode::Restrict => 0x02,
        };
        let target_bytes = Self::canonical_target_bytes(&req.target);
        let open_key = derive_open_key(&self.cfg.k_idx, &target_bytes, mode_byte);

        {
            let keys = self.open_keys.read().await;
            if let Some(existing_id) = keys.get(&open_key) {
                let map = self.active_sagas.read().await;
                if let Some(entry) = map.get(existing_id) {
                    let status = match entry.saga.state() {
                        SagaState::Requested => ErasureStatus::Requested,
                        SagaState::Gated => ErasureStatus::Gated,
                        SagaState::KeyDestroyed => ErasureStatus::KeyDestroyed,
                        SagaState::Purging => ErasureStatus::Purging,
                        SagaState::Verifying => ErasureStatus::Verifying,
                        SagaState::Completed => ErasureStatus::Completed,
                        SagaState::Restricted => ErasureStatus::Restricted,
                        SagaState::Lifted => ErasureStatus::Lifted,
                        SagaState::Pending => ErasureStatus::Pending {
                            phase: Phase::Purge,
                            reason: entry
                                .saga
                                .pending_reason()
                                .cloned()
                                .unwrap_or(PendingReason::Io),
                        },
                        SagaState::Refused => {
                            ErasureStatus::Refused(RefusalReason::RetentionObligation)
                        }
                    };
                    return Ok(EraseReceipt {
                        id: *existing_id,
                        status,
                        gated_at: Timestamp(Self::now_millis(self.deps.clock.as_ref())),
                    });
                }
            }
        }

        let (tx, rx) = oneshot::channel();
        let deps = Arc::clone(&self.deps);
        let cfg = self.cfg.clone();
        let active_sagas = Arc::clone(&self.active_sagas);
        let open_keys = Arc::clone(&self.open_keys);

        // Spawn einer entkoppelten Task für Cancel-Safety (ER-24)
        tokio::spawn(async move {
            let res =
                Self::execute_erase_flow(req, deps, cfg, active_sagas, open_keys, open_key).await;
            if tx.send(res).is_err() {
                tracing::debug!("erase caller receiver dropped before response");
            }
        });

        match rx.await {
            Ok(res) => res,
            Err(_) => Err(ErasureError::Internal(
                "erasure-core task panicked or aborted".into(),
            )),
        }
    }

    /// Interner Ablauf der Schritte 1–6.
    async fn execute_erase_flow(
        req: EraseExecutionRequest,
        deps: Arc<ErasureDeps>,
        cfg: ErasureConfig,
        active_sagas: Arc<RwLock<HashMap<ErasureId, ActiveEntry>>>,
        open_keys: Arc<RwLock<HashMap<[u8; 32], ErasureId>>>,
        open_key: [u8; 32],
    ) -> Result<EraseReceipt, ErasureError> {
        let erasure_id = req.erasure_id;

        // Retention-Matrix prüfen (D-124 Abs. 6)
        let decision = Self::evaluate_retention(req.mode, &req.attestation)?;
        match decision {
            RetentionDecision::Refuse(reason) => {
                let mut saga = EraseSaga::with_id(erasure_id);
                saga.transition_to(SagaState::Refused)?;
                let mut map = active_sagas.write().await;
                map.insert(
                    erasure_id,
                    ActiveEntry {
                        saga,
                        target: req.target,
                        attestation: req.attestation,
                        scopes: vec![],
                        gated_seq: 0,
                    },
                );
                return Ok(EraseReceipt {
                    id: erasure_id,
                    status: ErasureStatus::Refused(reason),
                    gated_at: Timestamp(Self::now_millis(deps.clock.as_ref())),
                });
            }
            RetentionDecision::ProceedRestrict => {
                // Modus Restrict: Gate setzen als Restricted, keine Vernichtung (D-15, D-116)
                let mut saga = EraseSaga::with_id(erasure_id);
                saga.transition_to(SagaState::Restricted)?;
                deps.journal
                    .append_intent(erasure_id, &req.target, &req.attestation)
                    .await?;
                let gated_seq = deps.gate.add_gate(erasure_id, &req.target).await?;
                deps.journal
                    .append_state(erasure_id, SagaState::Restricted, None)
                    .await?;

                Self::notify_observer(&deps, erasure_id, &req.target, ObservedPhase::Restricted)
                    .await;

                {
                    let mut map = active_sagas.write().await;
                    map.insert(
                        erasure_id,
                        ActiveEntry {
                            saga,
                            target: req.target,
                            attestation: req.attestation,
                            scopes: vec![],
                            gated_seq,
                        },
                    );
                }
                {
                    let mut keys = open_keys.write().await;
                    keys.insert(open_key, erasure_id);
                }

                return Ok(EraseReceipt {
                    id: erasure_id,
                    status: ErasureStatus::Restricted,
                    gated_at: Timestamp(Self::now_millis(deps.clock.as_ref())),
                });
            }
            RetentionDecision::ProceedErase => {
                // Weiter im normalen Löschablauf
            }
        }

        // --- SCHRITT 1: Validierung, Resolve je Schicht vor dem Gate, Journal Requested+Intent (fsync #1) ---
        let mut saga = EraseSaga::with_id(erasure_id);
        let mut scopes = Vec::new();
        for layer in &deps.layers {
            let scope = match layer.resolve(&req.target).await {
                Ok(s) => s,
                Err(_err) => {
                    let reason = PendingReason::LayerFailed(TypesLayerId(layer.id().0));
                    saga.transition_with_reason(SagaState::Pending, Some(reason.clone()))?;
                    deps.journal
                        .append_state(erasure_id, SagaState::Pending, Some(&reason))
                        .await?;
                    let mut map = active_sagas.write().await;
                    map.insert(
                        erasure_id,
                        ActiveEntry {
                            saga,
                            target: req.target,
                            attestation: req.attestation,
                            scopes: vec![],
                            gated_seq: 0,
                        },
                    );
                    return Ok(EraseReceipt {
                        id: erasure_id,
                        status: ErasureStatus::Pending {
                            phase: Phase::Intent,
                            reason,
                        },
                        gated_at: Timestamp(Self::now_millis(deps.clock.as_ref())),
                    });
                }
            };
            scopes.push((layer.id(), scope));
        }

        deps.journal
            .append_intent(erasure_id, &req.target, &req.attestation)
            .await?;

        // --- SCHRITT 2: Gate setzen unter CommitBarrier::exclusive (fsync #2) ---
        let _permit = deps.commit_barrier.exclusive().await;
        let gated_seq = deps.gate.add_gate(erasure_id, &req.target).await?;
        saga.transition_to(SagaState::Gated)?;
        deps.journal
            .append_state(erasure_id, SagaState::Gated, None)
            .await?;
        drop(_permit);

        // --- SCHRITT 3: on_gate je Schicht & Schlüsselvernichtung (fsync #3) ---
        for layer in &deps.layers {
            if let Some((_, scope)) = scopes.iter().find(|(id, _)| *id == layer.id()) {
                if let Err(_err) = layer.on_gate(scope).await {
                    let reason = PendingReason::LayerFailed(TypesLayerId(layer.id().0));
                    saga.transition_with_reason(SagaState::Pending, Some(reason.clone()))?;
                    deps.journal
                        .append_state(erasure_id, SagaState::Pending, Some(&reason))
                        .await?;
                    let mut map = active_sagas.write().await;
                    map.insert(
                        erasure_id,
                        ActiveEntry {
                            saga,
                            target: req.target,
                            attestation: req.attestation,
                            scopes,
                            gated_seq,
                        },
                    );
                    return Ok(EraseReceipt {
                        id: erasure_id,
                        status: ErasureStatus::Pending {
                            phase: Phase::Gate,
                            reason,
                        },
                        gated_at: Timestamp(Self::now_millis(deps.clock.as_ref())),
                    });
                }
            }
        }

        // Schlüsselvernichtung höchster Ebene via SagaVault (fsync #3)
        deps.vault.destroy_highest_key(&req.target).await?;

        // Fehlen verifizieren
        let absent = deps.vault.verify_key_absent(&req.target).await?;
        if !absent {
            let reason = PendingReason::Io;
            saga.transition_with_reason(SagaState::Pending, Some(reason.clone()))?;
            deps.journal
                .append_state(erasure_id, SagaState::Pending, Some(&reason))
                .await?;
            let mut map = active_sagas.write().await;
            map.insert(
                erasure_id,
                ActiveEntry {
                    saga,
                    target: req.target,
                    attestation: req.attestation,
                    scopes,
                    gated_seq,
                },
            );
            return Ok(EraseReceipt {
                id: erasure_id,
                status: ErasureStatus::Pending {
                    phase: Phase::KeyDestroy,
                    reason,
                },
                gated_at: Timestamp(Self::now_millis(deps.clock.as_ref())),
            });
        }

        // --- SCHRITT 4: Journal KeyDestroyed (fsync #4) & Rückgabe Gated ---
        saga.transition_to(SagaState::KeyDestroyed)?;
        deps.journal
            .append_state(erasure_id, SagaState::KeyDestroyed, None)
            .await?;

        Self::notify_observer(&deps, erasure_id, &req.target, ObservedPhase::Gated).await;

        let gated_at = Timestamp(Self::now_millis(deps.clock.as_ref()));

        // Saga in die aktiven Sagas aufnehmen
        {
            let mut map = active_sagas.write().await;
            map.insert(
                erasure_id,
                ActiveEntry {
                    saga: saga.clone(),
                    target: req.target.clone(),
                    attestation: req.attestation.clone(),
                    scopes: scopes.clone(),
                    gated_seq,
                },
            );
        }
        {
            let mut keys = open_keys.write().await;
            keys.insert(open_key, erasure_id);
        }

        // Falls Timing::Now gewählt ist, Purge direkt ausführen, ansonsten übernimmt der Hintergrundläufer
        if req.timing == Timing::Now {
            let purge_res = Self::execute_purge_and_verify(
                erasure_id,
                &req.target,
                &req.attestation,
                gated_seq,
                scopes,
                Arc::clone(&deps),
                &cfg,
                Arc::clone(&active_sagas),
                Arc::clone(&open_keys),
                open_key,
            )
            .await;

            if let Ok(status) = purge_res {
                return Ok(EraseReceipt {
                    id: erasure_id,
                    status,
                    gated_at,
                });
            }
        }

        // Bis hierher sind 4 fsyncs vollbracht (D-58), O(1) bis Gated
        Ok(EraseReceipt {
            id: erasure_id,
            status: ErasureStatus::Gated,
            gated_at,
        })
    }

    /// Führt die Schritte 5 (Purge) und 6 (Messung & Abschluss) durch.
    async fn execute_purge_and_verify(
        erasure_id: ErasureId,
        target: &EraseTarget,
        attestation: &RetentionAttestation,
        gated_seq: u64,
        scopes: Vec<(LayerId, LayerScope)>,
        deps: Arc<ErasureDeps>,
        cfg: &ErasureConfig,
        active_sagas: Arc<RwLock<HashMap<ErasureId, ActiveEntry>>>,
        open_keys: Arc<RwLock<HashMap<[u8; 32], ErasureId>>>,
        open_key: [u8; 32],
    ) -> Result<ErasureStatus, ErasureError> {
        let mut saga = {
            let map = active_sagas.read().await;
            map.get(&erasure_id)
                .map(|e| e.saga.clone())
                .unwrap_or_else(|| EraseSaga::with_id(erasure_id))
        };

        // --- SCHRITT 5: Purge-Fenster (Reihenfolge a–g) ---
        // PurgeBarrier exklusiv nehmen gegen Kompaktion (ER-21)
        let _purge_permit = deps.purge_barrier.purge().await;

        // Plattenreserve vorab prüfen (ER-22)
        let reserve = check_disk_reserve(cfg.min_free_bytes, 0)?;
        if let ReserveCheck::Insufficient { .. } = reserve {
            let reason = PendingReason::DiskReserve;
            saga.transition_with_reason(SagaState::Pending, Some(reason.clone()))?;
            deps.journal
                .append_state(erasure_id, SagaState::Pending, Some(&reason))
                .await?;
            let mut map = active_sagas.write().await;
            map.insert(
                erasure_id,
                ActiveEntry {
                    saga,
                    target: target.clone(),
                    attestation: attestation.clone(),
                    scopes,
                    gated_seq,
                },
            );
            return Ok(ErasureStatus::Pending {
                phase: Phase::Purge,
                reason,
            });
        }

        saga.transition_to(SagaState::Purging)?;
        deps.journal
            .append_state(erasure_id, SagaState::Purging, None)
            .await?;

        // 5a. Snapshots des Mandanten widerrufen (D-61, L-21)
        deps.snapshots.revoke_covering(target, gated_seq).await?;

        // 5b - 5f: Sortierung der Schichten nach Priorität a–g (PurgePriority)
        let mut sorted_layers = deps.layers.clone();
        sorted_layers.sort_by_key(|l| {
            deps.layer_priorities
                .get(&l.id().0)
                .copied()
                .unwrap_or(PurgePriority::SLOT_CLEANUP)
        });

        let purge_cx = PurgeCx::new(cfg.erase_sla.as_millis() as u64);

        for layer in &sorted_layers {
            if let Some((_, scope)) = scopes.iter().find(|(id, _)| *id == layer.id()) {
                if let Err(_err) = layer.purge(scope, &purge_cx).await {
                    let reason = PendingReason::LayerFailed(TypesLayerId(layer.id().0));
                    saga.transition_with_reason(SagaState::Pending, Some(reason.clone()))?;
                    deps.journal
                        .append_state(erasure_id, SagaState::Pending, Some(&reason))
                        .await?;
                    let mut map = active_sagas.write().await;
                    map.insert(
                        erasure_id,
                        ActiveEntry {
                            saga,
                            target: target.clone(),
                            attestation: attestation.clone(),
                            scopes,
                            gated_seq,
                        },
                    );
                    return Ok(ErasureStatus::Pending {
                        phase: Phase::Purge,
                        reason,
                    });
                }
            }
        }

        // --- SCHRITT 6: Messung & Abschluss (SPEC §11.2, §12) ---
        saga.transition_to(SagaState::Verifying)?;
        deps.journal
            .append_state(erasure_id, SagaState::Verifying, None)
            .await?;

        let mut witnesses = Vec::new();
        for layer in &deps.layers {
            match deps.measure.witness_layer(layer.id(), target).await {
                Ok(w) => witnesses.push(w),
                Err(ErasureError::Residue(_)) | Err(ErasureError::Corrupt) => {
                    let reason = PendingReason::WitnessMissing;
                    saga.transition_with_reason(SagaState::Pending, Some(reason.clone()))?;
                    deps.journal
                        .append_state(erasure_id, SagaState::Pending, Some(&reason))
                        .await?;
                    let mut map = active_sagas.write().await;
                    map.insert(
                        erasure_id,
                        ActiveEntry {
                            saga,
                            target: target.clone(),
                            attestation: attestation.clone(),
                            scopes,
                            gated_seq,
                        },
                    );
                    return Ok(ErasureStatus::Pending {
                        phase: Phase::Verify,
                        reason,
                    });
                }
                Err(err) => return Err(err),
            }
        }

        // Bericht signieren
        let _report_bytes = deps
            .report
            .sign_report(erasure_id, target, attestation, &witnesses)
            .await?;

        // Ledger-Eintrag schreiben
        let completed_at = Self::now_millis(deps.clock.as_ref());
        deps.ledger
            .record_completed(erasure_id, target, gated_seq, completed_at)
            .await?;

        // Journal Completed
        saga.transition_to(SagaState::Completed)?;
        deps.journal
            .append_state(erasure_id, SagaState::Completed, None)
            .await?;

        // Gate-Kompaktion
        deps.gate.compact_gate(erasure_id).await?;

        // Observer Completed
        Self::notify_observer(&deps, erasure_id, target, ObservedPhase::Completed).await;

        {
            let mut map = active_sagas.write().await;
            map.insert(
                erasure_id,
                ActiveEntry {
                    saga,
                    target: target.clone(),
                    attestation: attestation.clone(),
                    scopes,
                    gated_seq,
                },
            );
        }
        {
            let mut keys = open_keys.write().await;
            keys.remove(&open_key);
        }

        drop(_purge_permit);

        Ok(ErasureStatus::Completed)
    }

    /// Führt ein Purge-Fenster über alle offenen Sagas im Zustand `KeyDestroyed` oder `Purging` aus.
    pub async fn run_purge_window(
        &self,
    ) -> Result<crate::purge::window::WindowOutcome, ErasureError> {
        let mut outcome = crate::purge::window::WindowOutcome::empty();

        let candidates: Vec<(
            ErasureId,
            EraseTarget,
            RetentionAttestation,
            u64,
            Vec<(LayerId, LayerScope)>,
        )> = {
            let map = self.active_sagas.read().await;
            map.iter()
                .filter(|(_, entry)| {
                    entry.saga.state() == SagaState::KeyDestroyed
                        || entry.saga.state() == SagaState::Purging
                })
                .map(|(id, entry)| {
                    (
                        *id,
                        entry.target.clone(),
                        entry.attestation.clone(),
                        entry.gated_seq,
                        entry.scopes.clone(),
                    )
                })
                .collect()
        };

        for (id, target, attestation, gated_seq, scopes) in candidates {
            let mode_byte = 0x01;
            let target_bytes = Self::canonical_target_bytes(&target);
            let open_key = derive_open_key(&self.cfg.k_idx, &target_bytes, mode_byte);

            let res = Self::execute_purge_and_verify(
                id,
                &target,
                &attestation,
                gated_seq,
                scopes,
                Arc::clone(&self.deps),
                &self.cfg,
                Arc::clone(&self.active_sagas),
                Arc::clone(&self.open_keys),
                open_key,
            )
            .await;

            match res {
                Ok(ErasureStatus::Completed) => {
                    outcome.purged_units = outcome.purged_units.saturating_add(1);
                }
                Ok(ErasureStatus::Pending { .. }) => {
                    outcome.pending_count = outcome.pending_count.saturating_add(1);
                }
                _ => {}
            }
        }

        Ok(outcome)
    }

    /// Wiederanlauf und Recovery unfertiger Sagas (SPEC §11.2 Recovery, I-15, FM-1..14).
    pub async fn recover_and_resume(&self) -> Result<Vec<ErasureId>, ErasureError> {
        let open_sagas = self.deps.journal.recover_open_sagas().await?;
        let mut resumed = Vec::new();

        for mut saga in open_sagas {
            let id = saga.erasure_id();
            let next_step = match saga.state() {
                SagaState::Requested => SagaState::Gated,
                SagaState::Gated => SagaState::KeyDestroyed,
                SagaState::KeyDestroyed => SagaState::Purging,
                SagaState::Purging => SagaState::Verifying,
                SagaState::Verifying => SagaState::Completed,
                SagaState::Pending => SagaState::Purging,
                _ => continue,
            };

            saga.transition_to(next_step)?;
            self.deps.journal.append_state(id, next_step, None).await?;

            let mut active = self.active_sagas.write().await;
            active.insert(
                id,
                ActiveEntry {
                    saga,
                    target: EraseTarget::ExactKeys(vec![id.as_bytes().to_vec()]),
                    attestation: RetentionAttestation {
                        code: RetentionCode::NoObligation,
                        attested_by: crate::types_local::OperatorRef::try_new(b"recovery".to_vec())
                            .map_err(|e| ErasureError::Internal(format!("{:?}", e)))?,
                        attested_at: Timestamp(Self::now_millis(self.deps.clock.as_ref())),
                    },
                    scopes: vec![],
                    gated_seq: 0,
                },
            );
            resumed.push(id);
        }

        Ok(resumed)
    }

    /// Fragt den Status einer Löschung ab.
    pub async fn status(&self, id: ErasureId) -> Result<ErasureCoreStatus, ErasureError> {
        let map = self.active_sagas.read().await;
        if let Some(entry) = map.get(&id) {
            let status = match entry.saga.state() {
                SagaState::Requested => ErasureStatus::Requested,
                SagaState::Gated => ErasureStatus::Gated,
                SagaState::KeyDestroyed => ErasureStatus::KeyDestroyed,
                SagaState::Purging => ErasureStatus::Purging,
                SagaState::Verifying => ErasureStatus::Verifying,
                SagaState::Completed => ErasureStatus::Completed,
                SagaState::Restricted => ErasureStatus::Restricted,
                SagaState::Lifted => ErasureStatus::Lifted,
                SagaState::Pending => ErasureStatus::Pending {
                    phase: Phase::Purge,
                    reason: entry
                        .saga
                        .pending_reason()
                        .cloned()
                        .unwrap_or(PendingReason::Io),
                },
                SagaState::Refused => ErasureStatus::Refused(RefusalReason::RetentionObligation),
            };

            Ok(ErasureCoreStatus {
                id,
                state: entry.saga.state(),
                status,
                gated_seq: Some(entry.gated_seq),
            })
        } else {
            Err(ErasureError::SlotMissing)
        }
    }
}

/// Ergebnis der Retention-Prüfung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionDecision {
    /// Löschung wird fortgesetzt.
    ProceedErase,
    /// Zugriffsbeschränkung wird gesetzt.
    ProceedRestrict,
    /// Antrag wird verweigert.
    Refuse(RefusalReason),
}
