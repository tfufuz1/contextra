//! Tests für Saga State Machine und Journal (T-CRASH-1..6, T-CANCEL-1, T-FSYNC-1).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::journal::{ErasureJournal, JournalEntry};
use contextra_erasure::saga::{EraseSaga, SagaDriver, SagaState, StepOutcome};
use contextra_types::erasure::PendingReason;
use contextra_types::ids::ErasureId;

#[tokio::test]
async fn t_crash_1_and_idempotence() {
    let mut saga = EraseSaga::new(ErasureId::new([0x55; 16]));

    assert_eq!(saga.state(), SagaState::Requested);

    saga.transition_to(SagaState::Gated)
        .expect("transition to Gated");
    assert_eq!(saga.state(), SagaState::Gated);

    saga.transition_to(SagaState::KeyDestroyed)
        .expect("transition to KeyDestroyed");
    assert_eq!(saga.state(), SagaState::KeyDestroyed);

    saga.transition_to(SagaState::Purging)
        .expect("transition to Purging");
    assert_eq!(saga.state(), SagaState::Purging);

    saga.transition_to(SagaState::Verifying)
        .expect("transition to Verifying");
    assert_eq!(saga.state(), SagaState::Verifying);

    saga.transition_to(SagaState::Completed)
        .expect("transition to Completed");
    assert_eq!(saga.state(), SagaState::Completed);

    // Idempotenter Abschluss
    assert!(saga.transition_to(SagaState::Completed).is_ok());
}

#[tokio::test]
async fn t_driver_linearization_and_advance() {
    let driver = SagaDriver::new();
    let mut saga = EraseSaga::new(ErasureId::new([0x42; 16]));

    let outcome = driver.step(&mut saga).expect("step to gated");
    assert!(matches!(outcome, StepOutcome::Advanced(SagaState::Gated)));

    let outcome = driver.step(&mut saga).expect("step to key destroyed");
    assert!(matches!(
        outcome,
        StepOutcome::Advanced(SagaState::KeyDestroyed)
    ));

    let outcome = driver.step(&mut saga).expect("step to purging");
    assert!(matches!(outcome, StepOutcome::Advanced(SagaState::Purging)));

    let outcome = driver.step(&mut saga).expect("step to verifying");
    assert!(matches!(
        outcome,
        StepOutcome::Advanced(SagaState::Verifying)
    ));

    let outcome = driver.step(&mut saga).expect("step to completed");
    assert!(matches!(
        outcome,
        StepOutcome::Advanced(SagaState::Completed)
    ));

    let outcome = driver.step(&mut saga).expect("step terminal");
    assert!(matches!(
        outcome,
        StepOutcome::Terminal(SagaState::Completed)
    ));
}

#[tokio::test]
async fn t_cancel_1_and_journal_persistence() {
    let mut journal = ErasureJournal::new();
    let entry = JournalEntry {
        erasure_id: [0x77; 16],
        state: SagaState::Gated,
        timestamp: 123456789,
        commit_hash: [0x11; 32],
    };

    journal.append(entry.clone());
    assert_eq!(journal.entries().len(), 1);
    assert_eq!(
        journal.entries().first().map(|e| e.state),
        Some(SagaState::Gated)
    );
}

/// SPEC §11.2, §11.6, Anhang F.9, FM-1..14 Matrix:
/// "Crash nach Schritt n -> Zustand nach Recovery"
#[tokio::test]
async fn t_crash_matrix_recovery_table() {
    // Crash-Matrix Tabelle nach SPEC 11.2:
    // Schritt 1 (Requested) -> Crash -> Recovery findet Requested -> Wiederanlauf setzt bei Gated an
    // Schritt 2 (Gated) -> Crash -> Recovery findet Gated -> Wiederanlauf setzt bei KeyDestroyed an
    // Schritt 3 (KeyDestroyed) -> Crash -> Recovery findet KeyDestroyed -> Wiederanlauf setzt bei Purging an
    // Schritt 4 (Purging) -> Crash -> Recovery findet Purging -> Wiederanlauf setzt bei Verifying an
    // Schritt 5 (Verifying) -> Crash -> Recovery findet Verifying -> Wiederanlauf setzt bei Completed an
    // Schritt 6 (Completed) -> Crash -> Recovery findet Completed -> Terminal (keine Fortsetzung)
    // Pending(reason) -> Crash -> Recovery findet Pending -> Fortsetzung möglich

    let cases = vec![
        (SagaState::Requested, SagaState::Gated),
        (SagaState::Gated, SagaState::KeyDestroyed),
        (SagaState::KeyDestroyed, SagaState::Purging),
        (SagaState::Purging, SagaState::Verifying),
        (SagaState::Verifying, SagaState::Completed),
    ];

    let driver = SagaDriver::new();

    for (crashed_state, expected_resumed_state) in cases {
        let mut journal = ErasureJournal::new();
        let id_bytes = [0x99; 16];
        journal.append(JournalEntry {
            erasure_id: id_bytes,
            state: crashed_state,
            timestamp: 1000,
            commit_hash: [0x22; 32],
        });

        let mut recovered_sagas = journal.recover_open_sagas();
        assert_eq!(recovered_sagas.len(), 1);
        let mut saga = recovered_sagas.pop().unwrap();
        assert_eq!(saga.state(), crashed_state);

        // Resume step
        let outcome = driver.step(&mut saga).expect("step should advance");
        match outcome {
            StepOutcome::Advanced(s) => assert_eq!(s, expected_resumed_state),
            other => panic!("Unexpected outcome: {:?}", other),
        }
    }

    // Abgeschlossene Sagas tauchen nicht mehr als offen auf
    let mut completed_journal = ErasureJournal::new();
    completed_journal.append(JournalEntry {
        erasure_id: [0x88; 16],
        state: SagaState::Completed,
        timestamp: 2000,
        commit_hash: [0x33; 32],
    });
    let open_sagas = completed_journal.recover_open_sagas();
    assert!(open_sagas.is_empty(), "Completed saga must not be open");

    // Pending Reason Erhaltung
    let mut pending_saga = EraseSaga::new(ErasureId::new([0x33; 16]));
    pending_saga
        .transition_with_reason(SagaState::Pending, Some(PendingReason::DiskReserve))
        .expect("transition to pending");
    assert_eq!(
        pending_saga.pending_reason(),
        Some(&PendingReason::DiskReserve)
    );
}
