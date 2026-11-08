//! Tests für ErasureCore Orchestrierung, Schritte 1–6, Cancel-Safety, Observer und Retention-Matrix.
//!
//! T-ID: T-CRASH-1, T-CANCEL-1, T-SLA-1, T-OBS-1, T-OBS-2, T-OBS-3, T-STATUS-1

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::error::ErasureError;
use contextra_erasure::purge::order::PurgePriority;
use contextra_erasure::saga::core::{
    EraseExecutionRequest, ErasureConfig, ErasureCore, ErasureDeps,
};
use contextra_erasure::saga::machine::{EraseSaga, SagaState};
use contextra_erasure::saga::seams::{
    SagaGate, SagaJournal, SagaLedger, SagaMeasure, SagaReport, SagaVault, SnapshotRevoker,
};
use contextra_erasure::types_local::Clock;
use contextra_erasure::types_local::GateDecision;
use contextra_erasure::types_local::IdGen;
use contextra_erasure::types_local::OperatorRef;
use contextra_erasure::types_local::{
    Armed, EraseTarget, ErasureLayer, KeyCoverage, LayerId, LayerScope, LayerWitness, PurgeCx,
    PurgeOutcome, Scanner,
};
use contextra_erasure::types_local::{
    BarrierExclusive, BarrierShared, CommitBarrier, CompactionPermit, PurgeBarrier, PurgePermit,
};
use contextra_erasure::types_local::{
    EraseMode, ErasureStatus, PendingReason, Phase, RefusalReason, RetentionAttestation,
    RetentionCode, Timing,
};
use contextra_erasure::types_local::{ErasureEvent, ErasureObserver, ObservedPhase, ObserverError};
use contextra_erasure::types_local::{ErasureId, Timestamp};
use futures_util::future::BoxFuture;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// --- IN-MEMORY DOUBLES FÜR TESTS ---

#[derive(Default)]
struct MockJournal {
    states: Mutex<Vec<(ErasureId, SagaState, Option<PendingReason>)>>,
}

impl SagaJournal for MockJournal {
    fn append_intent<'a>(
        &'a self,
        _erasure_id: ErasureId,
        _target: &'a EraseTarget,
        _attestation: &'a RetentionAttestation,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move { Ok(()) })
    }

    fn append_state<'a>(
        &'a self,
        erasure_id: ErasureId,
        state: SagaState,
        reason: Option<&'a PendingReason>,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            self.states
                .lock()
                .unwrap()
                .push((erasure_id, state, reason.cloned()));
            Ok(())
        })
    }

    fn recover_open_sagas<'a>(&'a self) -> BoxFuture<'a, Result<Vec<EraseSaga>, ErasureError>> {
        Box::pin(async move {
            let guard = self.states.lock().unwrap();
            let mut map: HashMap<ErasureId, (SagaState, Option<PendingReason>)> = HashMap::new();
            for (id, state, reason) in guard.iter() {
                map.insert(*id, (*state, reason.clone()));
            }
            let mut list = Vec::new();
            for (id, (state, reason)) in map {
                if !state.is_terminal() {
                    let mut saga = EraseSaga::with_state(id, state);
                    if let Some(r) = reason {
                        let _ = saga.transition_with_reason(state, Some(r));
                    }
                    list.push(saga);
                }
            }
            Ok(list)
        })
    }
}

#[derive(Default)]
struct MockGate {
    gated_seq: AtomicU64,
    compacted: AtomicBool,
}

impl SagaGate for MockGate {
    fn add_gate<'a>(
        &'a self,
        _erasure_id: ErasureId,
        _target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<u64, ErasureError>> {
        Box::pin(async move {
            let seq = self.gated_seq.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(seq)
        })
    }

    fn compact_gate<'a>(
        &'a self,
        _erasure_id: ErasureId,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            self.compacted.store(true, Ordering::SeqCst);
            Ok(())
        })
    }

    fn decision_for_target<'a>(
        &'a self,
        _target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<GateDecision, ErasureError>> {
        Box::pin(async move { Ok(GateDecision::Erased) })
    }
}

#[derive(Default)]
struct MockVault {
    destroyed: AtomicBool,
    fail_destroy: AtomicBool,
}

impl SagaVault for MockVault {
    fn destroy_highest_key<'a>(
        &'a self,
        _target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            if self.fail_destroy.load(Ordering::SeqCst) {
                return Err(ErasureError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "vault destroy failure",
                )));
            }
            self.destroyed.store(true, Ordering::SeqCst);
            Ok(())
        })
    }

    fn verify_key_absent<'a>(
        &'a self,
        _target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<bool, ErasureError>> {
        Box::pin(async move { Ok(self.destroyed.load(Ordering::SeqCst)) })
    }
}

#[derive(Default)]
struct MockMeasure {
    fail_residue: AtomicBool,
}

impl SagaMeasure for MockMeasure {
    fn witness_layer<'a>(
        &'a self,
        _layer_id: LayerId,
        _target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<LayerWitness, ErasureError>> {
        Box::pin(async move {
            if self.fail_residue.load(Ordering::SeqCst) {
                return Err(ErasureError::Residue(1));
            }
            Ok(LayerWitness::from_hash([0x77; 32]))
        })
    }
}

#[derive(Default)]
struct MockReport;

impl SagaReport for MockReport {
    fn sign_report<'a>(
        &'a self,
        _erasure_id: ErasureId,
        _target: &'a EraseTarget,
        _attestation: &'a RetentionAttestation,
        _witnesses: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<Vec<u8>, ErasureError>> {
        Box::pin(async move { Ok(b"canonical-report-bytes".to_vec()) })
    }
}

#[derive(Default)]
struct MockLedger {
    recorded: AtomicBool,
}

impl SagaLedger for MockLedger {
    fn record_completed<'a>(
        &'a self,
        _erasure_id: ErasureId,
        _target: &'a EraseTarget,
        _gated_seq: u64,
        _completed_at: u64,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            self.recorded.store(true, Ordering::SeqCst);
            Ok(())
        })
    }
}

#[derive(Default)]
struct MockSnapshotRevoker {
    revoked_count: AtomicU64,
}

impl SnapshotRevoker for MockSnapshotRevoker {
    fn revoke_covering<'a>(
        &'a self,
        _target: &'a EraseTarget,
        _gated_seq: u64,
    ) -> BoxFuture<'a, Result<u64, ErasureError>> {
        Box::pin(async move {
            self.revoked_count.fetch_add(1, Ordering::SeqCst);
            Ok(1)
        })
    }
}

struct MockLayer {
    id: [u8; 16],
    purged: AtomicBool,
    fail_purge: AtomicBool,
}

impl ErasureLayer for MockLayer {
    fn id(&self) -> LayerId {
        LayerId::new(self.id)
    }

    fn resolve(&self, _t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        Box::pin(async move { Ok(LayerScope::new(self.id, vec![1, 2, 3])) })
    }

    fn on_gate(&self, _s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge(&self, _s: &LayerScope, _cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        Box::pin(async move {
            if self.fail_purge.load(Ordering::SeqCst) {
                return Err("purge failed".into());
            }
            self.purged.store(true, Ordering::SeqCst);
            Ok(PurgeOutcome::new(10, 1024))
        })
    }

    fn witness(
        &self,
        _s: &LayerScope,
        _sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        Box::pin(async move { Ok(LayerWitness::from_hash([0x88; 32])) })
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }
}

struct MockCommitBarrier;
impl CommitBarrier for MockCommitBarrier {
    fn shared<'a>(&'a self) -> BoxFuture<'a, BarrierShared> {
        Box::pin(async move { BarrierShared::new(None) })
    }

    fn exclusive<'a>(&'a self) -> BoxFuture<'a, BarrierExclusive> {
        Box::pin(async move { BarrierExclusive::new(None) })
    }
}

struct MockPurgeBarrier;
impl PurgeBarrier for MockPurgeBarrier {
    fn compaction<'a>(&'a self) -> BoxFuture<'a, CompactionPermit> {
        Box::pin(async move { CompactionPermit::new(None) })
    }

    fn purge<'a>(&'a self) -> BoxFuture<'a, PurgePermit> {
        Box::pin(async move { PurgePermit::new(None) })
    }

    fn purge_waiting(&self) -> bool {
        false
    }
}

struct MockClock;
impl Clock for MockClock {
    fn now_unix_nanos(&self) -> u64 {
        1_700_000_000_000_000_000
    }

    fn monotonic_nanos(&self) -> u64 {
        1_000_000
    }
}

struct MockIdGen;
impl IdGen for MockIdGen {
    fn next_id(&self) -> u64 {
        1
    }
}

#[derive(Default)]
struct MockObserver {
    events: Mutex<Vec<ErasureEvent>>,
}

impl ErasureObserver for MockObserver {
    fn on_erased(&self, ev: ErasureEvent) -> BoxFuture<'static, Result<(), ObserverError>> {
        self.events.lock().unwrap().push(ev);
        Box::pin(async move { Ok(()) })
    }
}

fn build_test_core() -> (
    ErasureCore,
    Arc<MockObserver>,
    Arc<MockVault>,
    Arc<MockMeasure>,
) {
    let journal = Arc::new(MockJournal::default());
    let gate = Arc::new(MockGate::default());
    let vault = Arc::new(MockVault::default());
    let measure = Arc::new(MockMeasure::default());
    let report = Arc::new(MockReport::default());
    let ledger = Arc::new(MockLedger::default());
    let snapshots = Arc::new(MockSnapshotRevoker::default());
    let observer = Arc::new(MockObserver::default());

    let layer1 = Arc::new(MockLayer {
        id: [1u8; 16],
        purged: AtomicBool::new(false),
        fail_purge: AtomicBool::new(false),
    });
    let layer2 = Arc::new(MockLayer {
        id: [2u8; 16],
        purged: AtomicBool::new(false),
        fail_purge: AtomicBool::new(false),
    });

    let mut priorities = HashMap::new();
    priorities.insert([1u8; 16], PurgePriority::TEXT_REWRITE);
    priorities.insert([2u8; 16], PurgePriority::SSTABLE_REWRITE);

    let deps = ErasureDeps {
        journal,
        gate,
        vault: Arc::clone(&vault) as Arc<dyn SagaVault>,
        measure: Arc::clone(&measure) as Arc<dyn SagaMeasure>,
        report,
        ledger,
        snapshots,
        layers: vec![layer1, layer2],
        layer_priorities: priorities,
        commit_barrier: Arc::new(MockCommitBarrier),
        purge_barrier: Arc::new(MockPurgeBarrier),
        clock: Arc::new(MockClock),
        ids: Arc::new(MockIdGen),
        observer: Some(Arc::clone(&observer) as Arc<dyn ErasureObserver>),
    };

    let cfg = ErasureConfig {
        min_free_bytes: 0, // In Memory: immer ausreichend
        ..Default::default()
    };

    let core = ErasureCore::new(cfg, deps);
    (core, observer, vault, measure)
}

fn sample_attestation(code: RetentionCode) -> RetentionAttestation {
    RetentionAttestation {
        code,
        attested_by: OperatorRef::try_new(b"op_1".to_vec()).unwrap(),
        attested_at: Timestamp(1700000000),
    }
}

/// SPEC: T-CRASH-1, T-SLA-1
#[tokio::test]
async fn t_crash_1_orchestration_full_flow_steps_1_to_6() {
    let (core, observer, vault, _) = build_test_core();

    let erasure_id = ErasureId::new([0x33; 16]);
    let req = EraseExecutionRequest {
        erasure_id,
        target: EraseTarget::ExactKeys(vec![b"doc_1".to_vec()]),
        mode: EraseMode::Erase,
        attestation: sample_attestation(RetentionCode::NoObligation),
        timing: Timing::Now,
    };

    let receipt = core.erase(req).await.expect("erase should succeed");
    assert_eq!(receipt.id, erasure_id);
    assert_eq!(receipt.status, ErasureStatus::Completed);
    assert!(vault.destroyed.load(Ordering::SeqCst));

    // Observer muss Gated und Completed erhalten haben
    let events = observer.events.lock().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].phase, ObservedPhase::Gated);
    assert_eq!(events[1].phase, ObservedPhase::Completed);
}

/// SPEC: T-SLA-1
#[tokio::test]
async fn t_sla_1_orchestration_batched_returns_gated() {
    let (core, observer, vault, _) = build_test_core();

    let erasure_id = ErasureId::new([0x44; 16]);
    let req = EraseExecutionRequest {
        erasure_id,
        target: EraseTarget::ExactKeys(vec![b"doc_2".to_vec()]),
        mode: EraseMode::Erase,
        attestation: sample_attestation(RetentionCode::NoObligation),
        timing: Timing::Batched,
    };

    let receipt = core.erase(req).await.expect("erase should succeed");
    assert_eq!(receipt.id, erasure_id);
    assert_eq!(receipt.status, ErasureStatus::Gated);
    assert!(vault.destroyed.load(Ordering::SeqCst));

    let events = observer.events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].phase, ObservedPhase::Gated);
}

/// SPEC: T-STATUS-1
#[tokio::test]
async fn t_status_1_retention_matrix_refusal() {
    let (core, _, _, _) = build_test_core();

    let erasure_id = ErasureId::new([0x55; 16]);
    let req = EraseExecutionRequest {
        erasure_id,
        target: EraseTarget::ExactKeys(vec![b"doc_3".to_vec()]),
        mode: EraseMode::Erase,
        attestation: sample_attestation(RetentionCode::ObligationActive),
        timing: Timing::Now,
    };

    let receipt = core.erase(req).await.expect("erase should return receipt");
    assert_eq!(receipt.id, erasure_id);
    assert_eq!(
        receipt.status,
        ErasureStatus::Refused(RefusalReason::RetentionObligation)
    );
}

/// SPEC: T-STATUS-1, T-OBS-1
#[tokio::test]
async fn t_restr_1_retention_matrix_restricted_mode() {
    let (core, observer, vault, _) = build_test_core();

    let erasure_id = ErasureId::new([0x66; 16]);
    let req = EraseExecutionRequest {
        erasure_id,
        target: EraseTarget::ExactKeys(vec![b"doc_4".to_vec()]),
        mode: EraseMode::Restrict,
        attestation: sample_attestation(RetentionCode::DataSubjectRequestedRestriction),
        timing: Timing::Now,
    };

    let receipt = core.erase(req).await.expect("erase should return receipt");
    assert_eq!(receipt.id, erasure_id);
    assert_eq!(receipt.status, ErasureStatus::Restricted);
    // Bei Restrict darf KEIN Schlüssel vernichtet werden
    assert!(!vault.destroyed.load(Ordering::SeqCst));

    let events = observer.events.lock().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].phase, ObservedPhase::Restricted);
}

/// SPEC: T-CANCEL-1
#[tokio::test]
async fn t_cancel_1_caller_abort() {
    let (core, _, vault, _) = build_test_core();

    let erasure_id = ErasureId::new([0x77; 16]);
    let req = EraseExecutionRequest {
        erasure_id,
        target: EraseTarget::ExactKeys(vec![b"doc_5".to_vec()]),
        mode: EraseMode::Erase,
        attestation: sample_attestation(RetentionCode::NoObligation),
        timing: Timing::Now,
    };

    // Erzeuge ein Future und breche es sofort per Timeout ab
    let erase_fut = core.erase(req);
    let _ = tokio::time::timeout(Duration::from_millis(1), erase_fut).await;

    // Gib dem Hintergrund-Task Zeit zum Ausführen
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Task lief im Hintergrund weiter (ER-24)
    assert!(vault.destroyed.load(Ordering::SeqCst));
}

/// SPEC: T-CRASH-1
#[tokio::test]
async fn t_crash_1_witness_missing_residue_pending() {
    let (core, _, _, measure) = build_test_core();
    measure.fail_residue.store(true, Ordering::SeqCst);

    let erasure_id = ErasureId::new([0x88; 16]);
    let req = EraseExecutionRequest {
        erasure_id,
        target: EraseTarget::ExactKeys(vec![b"doc_6".to_vec()]),
        mode: EraseMode::Erase,
        attestation: sample_attestation(RetentionCode::NoObligation),
        timing: Timing::Now,
    };

    let receipt = core.erase(req).await.expect("erase should succeed");
    assert_eq!(receipt.id, erasure_id);
    assert_eq!(
        receipt.status,
        ErasureStatus::Pending {
            phase: Phase::Verify,
            reason: PendingReason::WitnessMissing,
        }
    );
}
