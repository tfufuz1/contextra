// FILE-CONTEXT
// ZWECK: Campaign coverage test for TxBuffer reap_orphans_bounded and deterministic time injection.
// INVARIANTEN: reap_orphans_bounded caps reaped orphan count to max and leaves remaining unexpired/unreaped transactions intact.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_mvcc::tx_buffer::TxBuffer;
use contextra_types::TxId;
use std::time::Duration;

#[test]
fn test_tx_buffer_reap_orphans_bounded_exact_capping_and_time_injection() {
    let timeout = Duration::from_secs(10);
    let buffer = TxBuffer::<Vec<u8>>::new_with_config(4, timeout);

    let t_start = 1_000_000_000u64;
    let now_ticks = t_start + 20_000_000_000; // 20s later (> 10s timeout)

    let staged_txs: Vec<TxId> = (1..=10).map(TxId::new).collect();
    for &tx in &staged_txs {
        buffer.begin_at(tx, t_start);
    }

    assert_eq!(buffer.len(), 10);

    let reaped_batch_1 = buffer.reap_orphans_bounded_at(4, now_ticks);
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

    let reaped_batch_2 = buffer.reap_orphans_bounded_at(10, now_ticks);
    assert_eq!(reaped_batch_2.len(), 6);
    assert!(
        buffer.is_empty(),
        "Buffer MUST be empty after reaping all remaining orphans"
    );

    let mut all_reaped = reaped_batch_1;
    all_reaped.extend(reaped_batch_2);
    all_reaped.sort();
    assert_eq!(
        all_reaped, staged_txs,
        "Union of reaped batches MUST equal exact set of staged transactions"
    );
}
