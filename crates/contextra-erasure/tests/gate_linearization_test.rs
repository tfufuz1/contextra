//! Integrationstests für Lesesperre/Gate (SPEC §11.3, D-57, D-58, D-78, D-120, D-123, I-1, I-2, I-21, I-22).
//!
//! Test-IDs:
//! - T-GATE-1: Sofortige Sperre nach persistiertem Gate, Schreib- und Leseabweisung.
//! - T-GATE-2: Lese-Linearisierung vor und nach Entschlüsselung via `gate_epoch`; Wiederholung bei Epochenänderung.
//! - T-GATE-3: Zustandstabelle je Lesepfad (Erased vs Restricted vs Open).
//! - T-GATEGROUP-1: Mehrere nebenläufige Gate-Anfragen teilen sich dieselbe exklusive Barrieren-Phase und einen gemeinsamen fsync.
//! - T-VER-1: Replay-Filter / Zeitreise-Prüfung (`ReadGate::covers`).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::gate::batch::{
    add_batch, bind_global_erased_set, BatchGateRequest, GateBatchCoordinator,
};
use contextra_erasure::gate::erased_set::ErasedSet;
use contextra_erasure::gate::log::GateLog;
use contextra_erasure::gate::read_gate::GateGuard;
use contextra_erasure::types_local::{BarrierExclusive, BarrierShared, CommitBarrier};
use contextra_erasure::types_local::{GateDecision, KeyBuf, ReadGate, Seq};
use contextra_ports::BoxFuture;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Mock-CommitBarrier für Tests.
struct TestCommitBarrier {
    exclusive_count: AtomicUsize,
    shared_count: AtomicUsize,
}

impl TestCommitBarrier {
    fn new() -> Self {
        Self {
            exclusive_count: AtomicUsize::new(0),
            shared_count: AtomicUsize::new(0),
        }
    }
}

impl CommitBarrier for TestCommitBarrier {
    fn shared(&self) -> BoxFuture<'_, BarrierShared> {
        self.shared_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { BarrierShared::new(None) })
    }

    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive> {
        self.exclusive_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { BarrierExclusive::new(None) })
    }
}

/// Hilfsfunktion zur Erzeugung eines 78-Byte-Schlüssels mit spezifischer `commit_seq`.
fn make_78_byte_key(prefix_byte: u8, commit_seq: u64) -> [u8; 78] {
    let mut key = [0u8; 78];
    // Region (1) + Tenant(16) + Coll(16) + Cluster(16) + Doc(16) = 65 Bytes Präfix
    key[0..65].fill(prefix_byte);
    key[65] = 0x01; // Family::Doc
    key[66..70].copy_from_slice(&0u32.to_be_bytes()); // sub
    key[70..78].copy_from_slice(&(!commit_seq).to_be_bytes()); // !commit_seq
    key
}

#[tokio::test]
async fn t_gate_1_immediate_rejection_after_persist() {
    let set = Arc::new(RwLock::new(ErasedSet::new()));
    bind_global_erased_set(Arc::clone(&set));
    let guard = GateGuard::new(Arc::clone(&set));

    let key_buf = KeyBuf::new([0x11; 16]);
    assert_eq!(guard.check(&key_buf), GateDecision::Open);
    assert!(!guard.covers(&key_buf));

    let req = BatchGateRequest::new(
        [0xaa; 16],
        KeyBuf::new([0x10; 16]),
        KeyBuf::new([0x20; 16]),
        true,
        [0xbb; 16],
    );

    let outcome = add_batch(vec![req], Seq::new(42)).expect("add_batch failed");
    assert_eq!(outcome.gated_seq, Seq::new(42));
    assert!(outcome.applied_to_set, "must be applied to ErasedSet");

    // Nach dem Gate muss der Key sofort Erased sein (I-1, I-2)
    assert_eq!(guard.check(&key_buf), GateDecision::Erased);
    assert!(guard.covers(&key_buf), "Replay-Filter covers must be true");
}

#[tokio::test]
async fn t_gate_2_linearized_read_and_gate_epoch() {
    let set = Arc::new(RwLock::new(ErasedSet::new()));
    let guard = GateGuard::new(Arc::clone(&set));

    let initial_epoch = guard.gate_epoch();

    // Linearisierter Lesevorgang: Wenn Epoche gleich bleibt, ist alles konsistent.
    let (epoch_before, decision) = {
        let ep = guard.gate_epoch();
        let dec = guard.check(&KeyBuf::new([0x05; 16]));
        (ep, dec)
    };
    assert_eq!(epoch_before, initial_epoch);
    assert_eq!(decision, GateDecision::Open);

    // Jetzt fügen wir ein Gate hinzu -> gate_epoch muss steigen
    set.write().add_erased(
        KeyBuf::new([0x04; 16]),
        KeyBuf::new([0x06; 16]),
        Seq::new(100),
    );

    let new_epoch = guard.gate_epoch();
    assert!(
        new_epoch > initial_epoch,
        "gate_epoch must increment on gate addition"
    );

    // Nach Entschlüsselung wird Epoche verglichen; bei Änderung muss wiederholt werden (I-21)
    let decision_after = guard.check(&KeyBuf::new([0x05; 16]));
    assert_eq!(decision_after, GateDecision::Erased);

    // Test des Helfer-Linearized-Reads
    let res = guard.linearized_read(&KeyBuf::new([0x05; 16]), || Ok("test_data"));
    assert_eq!(res, Err(GateDecision::Erased));
}

#[test]
fn t_gate_3_state_table_per_read_path() {
    let mut set = ErasedSet::new();

    let k_open = KeyBuf::new([0x01; 16]);
    let k_erased = KeyBuf::new([0x05; 16]);
    let k_restr = KeyBuf::new([0x09; 16]);

    set.add_erased(
        KeyBuf::new([0x04; 16]),
        KeyBuf::new([0x06; 16]),
        Seq::new(10),
    );
    set.add_restricted(
        KeyBuf::new([0x08; 16]),
        KeyBuf::new([0x0a; 16]),
        Seq::new(20),
    );

    assert_eq!(set.check(&k_open), GateDecision::Open);
    assert_eq!(set.check(&k_erased), GateDecision::Erased);
    assert_eq!(set.check(&k_restr), GateDecision::Restricted);

    // covers gibt true für Erased und Restricted zurück (Records gegateter Bereiche dürfen nicht in Memtable)
    assert!(!set.covers(&k_open));
    assert!(set.covers(&k_erased));
    assert!(set.covers(&k_restr));
}

#[tokio::test]
async fn t_gategroup_1_concurrent_erases_share_barrier_phase() {
    let barrier = Arc::new(TestCommitBarrier::new());
    let erased_set = Arc::new(RwLock::new(ErasedSet::new()));
    let gate_log = Arc::new(RwLock::new(GateLog::new()));

    let coordinator = GateBatchCoordinator::new(
        Arc::clone(&barrier) as Arc<dyn CommitBarrier>,
        Arc::clone(&erased_set),
        Arc::clone(&gate_log),
    );

    let req1 = BatchGateRequest::new(
        [0x01; 16],
        KeyBuf::new([0x10; 16]),
        KeyBuf::new([0x20; 16]),
        true,
        [0x11; 16],
    );

    let req2 = BatchGateRequest::new(
        [0x02; 16],
        KeyBuf::new([0x30; 16]),
        KeyBuf::new([0x40; 16]),
        false,
        [0x22; 16],
    );

    let outcome = coordinator
        .execute_batch(vec![req1, req2], Seq::new(50))
        .await
        .expect("batch execution failed");

    assert_eq!(outcome.gated_seq, Seq::new(50));
    assert!(outcome.applied_to_set);
    assert_eq!(outcome.gates_count, 2);

    // Genau 1 exklusiver Barrieren-Schritt für beide Anfragen zusammen! (D-78)
    assert_eq!(barrier.exclusive_count.load(Ordering::SeqCst), 1);

    // Beide Gates im Log
    assert_eq!(gate_log.read().len(), 2);

    // Beide Gates im ErasedSet
    assert_eq!(
        erased_set.read().check(&KeyBuf::new([0x15; 16])),
        GateDecision::Erased
    );
    assert_eq!(
        erased_set.read().check(&KeyBuf::new([0x35; 16])),
        GateDecision::Restricted
    );
}

#[test]
fn t_ver_1_replay_filter_and_78_byte_matcher() {
    let mut set = ErasedSet::new();

    // 78-Byte Key Matcher: Matcht Präfix und prüft commit_seq < seq_lt (PrefixSeq, D-117)
    let raw_key_old = make_78_byte_key(0xAA, 100);
    let raw_key_new = make_78_byte_key(0xAA, 200);

    set.add_prefix_seq([0xAA; 65], Seq::new(150), Seq::new(500));

    // raw_key_old (seq 100 < 150) ist gegated -> covers liefert true
    assert!(set.covers_raw(&raw_key_old));
    assert_eq!(set.check_raw(&raw_key_old), GateDecision::Erased);

    // raw_key_new (seq 200 >= 150) ist offen -> covers liefert false
    assert!(!set.covers_raw(&raw_key_new));
    assert_eq!(set.check_raw(&raw_key_new), GateDecision::Open);
}
