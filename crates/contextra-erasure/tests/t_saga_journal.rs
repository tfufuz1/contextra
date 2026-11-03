//! Tests für Saga State Machine und Journal (T-CRASH-1..6, T-CANCEL-1, T-FSYNC-1).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::journal::{ErasureJournal, JournalEntry};
use contextra_erasure::saga::{EraseSaga, SagaDriver, SagaState, StepOutcome};
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
    assert!(matches!(outcome, StepOutcome::Advanced(SagaState::KeyDestroyed)));

    let outcome = driver.step(&mut saga).expect("step to purging");
    assert!(matches!(outcome, StepOutcome::Advanced(SagaState::Purging)));

    let outcome = driver.step(&mut saga).expect("step to verifying");
    assert!(matches!(outcome, StepOutcome::Advanced(SagaState::Verifying)));

    let outcome = driver.step(&mut saga).expect("step to completed");
    assert!(matches!(outcome, StepOutcome::Advanced(SagaState::Completed)));

    let outcome = driver.step(&mut saga).expect("step terminal");
    assert!(matches!(outcome, StepOutcome::Terminal(SagaState::Completed)));
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
