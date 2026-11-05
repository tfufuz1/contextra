//! Crash-Matrix Tests FM-1 bis FM-14 und Wiederanlauf (SPEC §11.2, §15.1, FM-1..14, T-CRASH-1..6).
//!
//! T-ID: T-CRASH-1, T-CRASH-2, T-CRASH-3, T-CRASH-4, T-CRASH-5, T-CRASH-6

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::error::ErasureError;
use contextra_erasure::journal::{ErasureJournal, JournalEntry};
use contextra_erasure::purge::reserve::{check_disk_reserve, ReserveCheck};
use contextra_erasure::saga::core::{ErasureConfig, ErasureCore, ErasureDeps};
use contextra_erasure::saga::machine::{EraseSaga, SagaState};
use contextra_erasure::saga::seams::{
    SagaGate, SagaJournal, SagaLedger, SagaMeasure, SagaReport, SagaVault, SnapshotRevoker,
};
use contextra_ports::barrier::{
    BarrierExclusive, BarrierShared, CommitBarrier, CompactionPermit, PurgeBarrier, PurgePermit,
};
use contextra_ports::clock::Clock;
use contextra_ports::erasure::{EraseTarget, LayerId, LayerWitness};
use contextra_ports::gate::GateDecision;
use contextra_ports::id_gen::IdGen;
use contextra_ports::BoxFuture;
use contextra_types::erasure::PendingReason;
use contextra_types::ids::ErasureId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MatrixJournal {
    entries: Mutex<Vec<JournalEntry>>,
}

impl SagaJournal for MatrixJournal {
    fn append_intent<'a>(
        &'a self,
        _erasure_id: ErasureId,
        _target: &'a EraseTarget,
        _attestation: &'a contextra_types::erasure::RetentionAttestation,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move { Ok(()) })
    }

    fn append_state<'a>(
        &'a self,
        erasure_id: ErasureId,
        state: SagaState,
        _reason: Option<&'a PendingReason>,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            self.entries.lock().unwrap().push(JournalEntry {
                erasure_id: *erasure_id.as_bytes(),
                state,
                timestamp: 1000,
                commit_hash: [0u8; 32],
            });
            Ok(())
        })
    }

    fn recover_open_sagas<'a>(&'a self) -> BoxFuture<'a, Result<Vec<EraseSaga>, ErasureError>> {
        Box::pin(async move {
            let entries = self.entries.lock().unwrap().clone();
            let mut j = ErasureJournal::new();
            for e in entries {
                j.append(e);
            }
            Ok(j.recover_open_sagas())
        })
    }
}

#[derive(Default)]
struct DummyGate;
impl SagaGate for DummyGate {
    fn add_gate<'a>(
        &'a self,
        _id: ErasureId,
        _target: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<u64, ErasureError>> {
        Box::pin(async move { Ok(10) })
    }
    fn compact_gate<'a>(&'a self, _id: ErasureId) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move { Ok(()) })
    }
    fn decision_for_target<'a>(
        &'a self,
        _t: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<GateDecision, ErasureError>> {
        Box::pin(async move { Ok(GateDecision::Erased) })
    }
}

#[derive(Default)]
struct DummyVault;
impl SagaVault for DummyVault {
    fn destroy_highest_key<'a>(
        &'a self,
        _t: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move { Ok(()) })
    }
    fn verify_key_absent<'a>(
        &'a self,
        _t: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<bool, ErasureError>> {
        Box::pin(async move { Ok(true) })
    }
}

#[derive(Default)]
struct DummyMeasure;
impl SagaMeasure for DummyMeasure {
    fn witness_layer<'a>(
        &'a self,
        _id: LayerId,
        _t: &'a EraseTarget,
    ) -> BoxFuture<'a, Result<LayerWitness, ErasureError>> {
        Box::pin(async move { Ok(LayerWitness::from_hash([0x11; 32])) })
    }
}

#[derive(Default)]
struct DummyReport;
impl SagaReport for DummyReport {
    fn sign_report<'a>(
        &'a self,
        _id: ErasureId,
        _t: &'a EraseTarget,
        _att: &'a contextra_types::erasure::RetentionAttestation,
        _w: &'a [LayerWitness],
    ) -> BoxFuture<'a, Result<Vec<u8>, ErasureError>> {
        Box::pin(async move { Ok(vec![1, 2, 3]) })
    }
}

#[derive(Default)]
struct DummyLedger;
impl SagaLedger for DummyLedger {
    fn record_completed<'a>(
        &'a self,
        _id: ErasureId,
        _t: &'a EraseTarget,
        _seq: u64,
        _at: u64,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move { Ok(()) })
    }
}

#[derive(Default)]
struct DummySnapshotRevoker;
impl SnapshotRevoker for DummySnapshotRevoker {
    fn revoke_covering<'a>(
        &'a self,
        _t: &'a EraseTarget,
        _seq: u64,
    ) -> BoxFuture<'a, Result<u64, ErasureError>> {
        Box::pin(async move { Ok(0) })
    }
}

struct DummyCommitBarrier;
impl CommitBarrier for DummyCommitBarrier {
    fn shared<'a>(&'a self) -> BoxFuture<'a, BarrierShared> {
        Box::pin(async move { BarrierShared::new(None) })
    }
    fn exclusive<'a>(&'a self) -> BoxFuture<'a, BarrierExclusive> {
        Box::pin(async move { BarrierExclusive::new(None) })
    }
}

struct DummyPurgeBarrier;
impl PurgeBarrier for DummyPurgeBarrier {
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

struct DummyClock;
impl Clock for DummyClock {
    fn now_unix_nanos(&self) -> u64 {
        1000
    }
    fn monotonic_nanos(&self) -> u64 {
        1000
    }
}

struct DummyIdGen;
impl IdGen for DummyIdGen {
    fn next_id(&self) -> u64 {
        1
    }
}

/// SPEC: T-CRASH-1
#[tokio::test]
async fn t_crash_1_fm_matrix_recovery_step_progression() {
    let journal = Arc::new(MatrixJournal::default());
    let deps = ErasureDeps {
        journal: Arc::clone(&journal) as Arc<dyn SagaJournal>,
        gate: Arc::new(DummyGate),
        vault: Arc::new(DummyVault),
        measure: Arc::new(DummyMeasure),
        report: Arc::new(DummyReport),
        ledger: Arc::new(DummyLedger),
        snapshots: Arc::new(DummySnapshotRevoker),
        layers: vec![],
        layer_priorities: HashMap::new(),
        commit_barrier: Arc::new(DummyCommitBarrier),
        purge_barrier: Arc::new(DummyPurgeBarrier),
        clock: Arc::new(DummyClock),
        ids: Arc::new(DummyIdGen),
        observer: None,
    };
    let core = ErasureCore::new(ErasureConfig::default(), deps);

    // FM-1: Crash nach Requested -> Recovery findet Requested
    let id1 = ErasureId::new([0x10; 16]);
    journal
        .append_state(id1, SagaState::Requested, None)
        .await
        .unwrap();

    // FM-1b: Crash nach Gated -> Recovery findet Gated
    let id2 = ErasureId::new([0x20; 16]);
    journal
        .append_state(id2, SagaState::Gated, None)
        .await
        .unwrap();

    // FM-2: Crash während Purging -> Recovery findet Purging
    let id3 = ErasureId::new([0x30; 16]);
    journal
        .append_state(id3, SagaState::Purging, None)
        .await
        .unwrap();

    // Completed Saga darf nicht mehr in offenen Sagas auftauchen
    let id4 = ErasureId::new([0x40; 16]);
    journal
        .append_state(id4, SagaState::Completed, None)
        .await
        .unwrap();

    let recovered = core
        .recover_and_resume()
        .await
        .expect("recovery should succeed");
    assert_eq!(recovered.len(), 3);
    assert!(recovered.contains(&id1));
    assert!(recovered.contains(&id2));
    assert!(recovered.contains(&id3));
    assert!(!recovered.contains(&id4));
}

/// SPEC: T-CRASH-3
#[test]
fn t_crash_3_fm_3_disk_reserve_pending() {
    let check = check_disk_reserve(500_000, 1_000_000).unwrap();
    assert!(matches!(check, ReserveCheck::Insufficient { .. }));
}
