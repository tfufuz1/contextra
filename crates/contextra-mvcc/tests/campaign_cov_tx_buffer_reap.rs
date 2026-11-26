// ANCHOR[contextra-mvcc:TEST-CAMPAIGN-COV-TX-REAP-ROOT]
//! # Kampagnen-Test für TxBuffer Orphan-Reaping und Obergrenzen
//!
//! Prüft die gezielte Testabdeckung von `reap_orphans_bounded` unter deterministischer
//! Zeiteinspeisung über `ManualClock`.

// FILE-CONTEXT
// ZWECK: Campaign coverage test for TxBuffer reap_orphans_bounded and deterministic time injection.
// INVARIANTEN: reap_orphans_bounded caps reaped orphan count to max and leaves remaining unexpired/unreaped transactions intact.
// NICHT-OFFENSICHTLICH: Uses independent oracle (R4) validating set difference of staged vs reaped TxIDs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_mvcc::tx_buffer::{TxBuffer, TxBufferConfig};
use contextra_testkit::ManualClock;
use contextra_types::TxId;
use std::sync::Arc;
use std::time::Duration;

// Prüft die exakte Begrenzung der abgeräumten verwaisten Transaktionen auf die vorgegebene Obergrenze.
#[test]
fn test_tx_buffer_reap_orphans_bounded_exact_capping_and_time_injection() {
    let clock = Arc::new(ManualClock::new(100_000_000_000));
    let timeout = Duration::from_secs(10);
    let config = TxBufferConfig {
        tx_timeout: timeout,
        ..Default::default()
    };
    let buffer = TxBuffer::<Vec<u8>>::new_with_clock_and_config(clock.clone(), 4, config);

    let start_nanos = clock.now_nanos();
    let past_nanos = start_nanos.saturating_sub(20_000_000_000);

    // Stage 10 orphan transactions in the past
    let staged_txs: Vec<TxId> = (1..=10).map(TxId::new).collect();
    for &tx in &staged_txs {
        buffer.begin_at(tx, past_nanos);
    }

    assert_eq!(buffer.len(), 10);

    // 1. Reap bounded to max = 4
    let reaped_batch_1 = buffer.reap_orphans_bounded_at(4, start_nanos);
    assert_eq!(
        reaped_batch_1.len(),
        4,
        "Bounded reap MUST return exactly max = 4 transactions"
    );
    assert_eq!(
        buffer.len(),
        6,
        "Remaining 6 transactions MUST stay in buffer"
    );

    // 2. Reap bounded to max = 10 (reaps remaining 6)
    let reaped_batch_2 = buffer.reap_orphans_bounded(10);
    assert_eq!(reaped_batch_2.len(), 6);
    assert!(
        buffer.is_empty(),
        "Buffer MUST be empty after reaping all remaining orphans"
    );

    // 3. Independent Oracle Check (R4): Combine reaped batches and verify total union set equals staged_txs
    let mut all_reaped = reaped_batch_1;
    all_reaped.extend(reaped_batch_2);
    all_reaped.sort();
    assert_eq!(
        all_reaped, staged_txs,
        "Union of reaped batches MUST equal exact set of staged transactions"
    );
}
