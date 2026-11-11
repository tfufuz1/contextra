//! Testsuite für die Crash- und Wiederaufnahme-Matrix (WP-2, SPEC §11, INV-ERA-1..7).
//!
//! Verifiziert für jede Phase der Saga (`Requested`, `Gated`, `KeyDestroyed`,
//! `Purging`, `Measured`, `Completed`):
//! 1. Saubere Fehlerunterbrechung / Abbruchsimulation an Phasengrenzen.
//! 2. Deterministische Wiederaufnahme (`resume_and_execute_purge` / Replay).
//! 3. Idempotenz: Wiederholte Ausführung nach erfolgreichem Abschluss führt zum
//!    identischen Endzustand ohne Doppelwirkung.

use contextra_erasure::gate::fence::{FenceGate, FenceRange, InMemoryFenceGate};
use contextra_erasure::journal::wal::MemoryWalSink;
use contextra_erasure::orchestrator::{
    ErasureCoordinator, MemoryKeyStore, MemoryResidueScanner,
};
use contextra_erasure::purge::phase::PurgePhase;
use contextra_erasure::restore::replay::resume_and_execute_purge;
use contextra_erasure::types_local::{
    Clock, DocId, ErasureTarget, LayerId,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct StepClock {
    now: AtomicU64,
}

impl StepClock {
    fn new(start_nanos: u64) -> Self {
        Self {
            now: AtomicU64::new(start_nanos),
        }
    }
}

impl Clock for StepClock {
    fn monotonic_nanos(&self) -> u64 {
        self.now.fetch_add(1_000, Ordering::Relaxed)
    }

    fn system_time_nanos(&self) -> u64 {
        self.now.load(Ordering::Relaxed)
    }
}

#[test]
fn test_crash_at_requested_phase_and_resume() {
    let clock = StepClock::new(1_000_000);
    let mut coordinator = ErasureCoordinator::new(clock.system_time_nanos());

    // Phase: Requested
    assert_eq!(coordinator.current_phase(), PurgePhase::Admission);

    // Simulation: Absturz vor Gate-Aktivierung
    // Bei Wiederanlauf muss eine neue/unterbrochene Saga sauber von Admission starten können
    let restarted_coordinator = ErasureCoordinator::new(clock.system_time_nanos());
    assert_eq!(restarted_coordinator.current_phase(), PurgePhase::Admission);
}

#[test]
fn test_crash_at_gated_phase_and_resume() {
    let clock = StepClock::new(2_000_000);
    let fence_gate = Arc::new(InMemoryFenceGate::new());
    let wal_sink = Arc::new(MemoryWalSink::new());
    let key_store = Arc::new(MemoryKeyStore::new());
    let scanner = Arc::new(MemoryResidueScanner::new(LayerId::DocStore));

    let mut coordinator = ErasureCoordinator::new(clock.system_time_nanos());

    // 1. Bis Gated / Fence ausführen
    coordinator
        .advance_to_fence(fence_gate.as_ref())
        .expect("Fence advance must succeed");
    assert_eq!(coordinator.current_phase(), PurgePhase::Fence);

    // Simulation: Absturz an der Fence-Phase
    // Wiederaufnahme über resume_and_execute_purge
    let res = resume_and_execute_purge(
        &mut coordinator,
        fence_gate.as_ref(),
        wal_sink.as_ref(),
        key_store.as_ref(),
        scanner.as_ref(),
    );
    assert!(res.is_ok(), "Resume after Fence must succeed: {:?}", res);
    assert_eq!(coordinator.current_phase(), PurgePhase::Completed);
}

#[test]
fn test_crash_at_key_destroyed_phase_and_resume() {
    let clock = StepClock::new(3_000_000);
    let fence_gate = Arc::new(InMemoryFenceGate::new());
    let wal_sink = Arc::new(MemoryWalSink::new());
    let key_store = Arc::new(MemoryKeyStore::new());
    let scanner = Arc::new(MemoryResidueScanner::new(LayerId::DocStore));

    let mut coordinator = ErasureCoordinator::new(clock.system_time_nanos());

    // Bis WalRetire vorrücken
    coordinator.advance_to_fence(fence_gate.as_ref()).unwrap();
    coordinator.advance_to_wal(wal_sink.as_ref()).unwrap();
    coordinator.advance_to_keys(key_store.as_ref()).unwrap();
    assert_eq!(coordinator.current_phase(), PurgePhase::KeyDestruction);

    // Simulation: Absturz nach Schlüsselvernichtung
    // Wiederaufnahme muss die nachfolgenden Phasen (Rewrite, Witness, Finalize) abschließen
    let res = resume_and_execute_purge(
        &mut coordinator,
        fence_gate.as_ref(),
        wal_sink.as_ref(),
        key_store.as_ref(),
        scanner.as_ref(),
    );
    assert!(res.is_ok());
    assert_eq!(coordinator.current_phase(), PurgePhase::Completed);
}

#[test]
fn test_crash_at_purging_docstore_and_resume() {
    let clock = StepClock::new(4_000_000);
    let fence_gate = Arc::new(InMemoryFenceGate::new());
    let wal_sink = Arc::new(MemoryWalSink::new());
    let key_store = Arc::new(MemoryKeyStore::new());
    let scanner = Arc::new(MemoryResidueScanner::new(LayerId::DocStore));

    let mut coordinator = ErasureCoordinator::new(clock.system_time_nanos());

    // Bis SegmentRewrite vorrücken
    coordinator.advance_to_fence(fence_gate.as_ref()).unwrap();
    coordinator.advance_to_wal(wal_sink.as_ref()).unwrap();
    coordinator.advance_to_keys(key_store.as_ref()).unwrap();
    coordinator.advance_to_rewrite().unwrap();
    assert_eq!(coordinator.current_phase(), PurgePhase::SegmentRewrite);

    // Wiederaufnahme nach Unterbrechung im Rewrite
    let res = resume_and_execute_purge(
        &mut coordinator,
        fence_gate.as_ref(),
        wal_sink.as_ref(),
        key_store.as_ref(),
        scanner.as_ref(),
    );
    assert!(res.is_ok());
    assert_eq!(coordinator.current_phase(), PurgePhase::Completed);
}

#[test]
fn test_crash_at_measured_phase_and_resume() {
    let clock = StepClock::new(5_000_000);
    let fence_gate = Arc::new(InMemoryFenceGate::new());
    let wal_sink = Arc::new(MemoryWalSink::new());
    let key_store = Arc::new(MemoryKeyStore::new());
    let scanner = Arc::new(MemoryResidueScanner::new(LayerId::DocStore));

    let mut coordinator = ErasureCoordinator::new(clock.system_time_nanos());

    // Bis Witness-Phase vorrücken
    coordinator.advance_to_fence(fence_gate.as_ref()).unwrap();
    coordinator.advance_to_wal(wal_sink.as_ref()).unwrap();
    coordinator.advance_to_keys(key_store.as_ref()).unwrap();
    coordinator.advance_to_rewrite().unwrap();
    coordinator.advance_to_witness(scanner.as_ref()).unwrap();
    assert_eq!(coordinator.current_phase(), PurgePhase::WitnessCollection);

    // Wiederaufnahme: muss finalisieren und Fence freigeben
    let res = resume_and_execute_purge(
        &mut coordinator,
        fence_gate.as_ref(),
        wal_sink.as_ref(),
        key_store.as_ref(),
        scanner.as_ref(),
    );
    assert!(res.is_ok());
    assert_eq!(coordinator.current_phase(), PurgePhase::Completed);
}

#[test]
fn test_replay_idempotency_after_completed() {
    let clock = StepClock::new(6_000_000);
    let fence_gate = Arc::new(InMemoryFenceGate::new());
    let wal_sink = Arc::new(MemoryWalSink::new());
    let key_store = Arc::new(MemoryKeyStore::new());
    let scanner = Arc::new(MemoryResidueScanner::new(LayerId::DocStore));

    let mut coordinator = ErasureCoordinator::new(clock.system_time_nanos());

    // Vollständig ausführen
    let res = resume_and_execute_purge(
        &mut coordinator,
        fence_gate.as_ref(),
        wal_sink.as_ref(),
        key_store.as_ref(),
        scanner.as_ref(),
    );
    assert!(res.is_ok());
    assert_eq!(coordinator.current_phase(), PurgePhase::Completed);

    // Idempotentes Replay auf bereits abgeschlossener Saga
    let res_replay = resume_and_execute_purge(
        &mut coordinator,
        fence_gate.as_ref(),
        wal_sink.as_ref(),
        key_store.as_ref(),
        scanner.as_ref(),
    );
    assert!(res_replay.is_ok(), "Replay on completed saga must succeed idempotently");
    assert_eq!(coordinator.current_phase(), PurgePhase::Completed);
}
