//! Proptest / drift test for MemTable budget accounting alignment (WP-16.3).

use bytes::Bytes;
use contextra_mvcc::tx_buffer::STAGING_ENTRY_OVERHEAD_BYTES;
use contextra_store::memtable::MemTable;
use proptest::prelude::*;

#[test]
fn test_memtable_size_matches_sum_of_living_entries_after_rollbacks() {
    let mt = MemTable::new();

    // Insert Tx 1
    mt.put(Bytes::from("k1"), Bytes::from("v1"), 1, 10);
    mt.put(Bytes::from("k2"), Bytes::from("v2"), 2, 10);

    // Insert Tx 2
    mt.put(Bytes::from("k3"), Bytes::from("v3"), 3, 20);

    let expected_size_tx1 = (2 + 2 + STAGING_ENTRY_OVERHEAD_BYTES) * 2;
    let expected_size_tx2 = 2 + 2 + STAGING_ENTRY_OVERHEAD_BYTES;

    assert_eq!(mt.size(), expected_size_tx1 + expected_size_tx2);

    // Rollback Tx 1
    mt.rollback(10);

    assert_eq!(mt.size(), expected_size_tx2);

    // Rollback Tx 2
    mt.rollback(20);

    assert_eq!(mt.size(), 0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn prop_memtable_budget_drift_is_zero(
        tx1_keys in prop::collection::vec((any::<u8>(), any::<u8>()), 1..20),
        tx2_keys in prop::collection::vec((any::<u8>(), any::<u8>()), 1..20),
    ) {
        let mt = MemTable::new();

        let mut tx1_expected_size = 0usize;
        for (i, (k, v)) in tx1_keys.iter().enumerate() {
            let key = Bytes::from(vec![*k]);
            let val = Bytes::from(vec![*v]);
            tx1_expected_size += key.len() + val.len() + STAGING_ENTRY_OVERHEAD_BYTES;
            mt.put(key, val, (i + 1) as u64, 100);
        }

        let mut tx2_expected_size = 0usize;
        for (i, (k, v)) in tx2_keys.iter().enumerate() {
            let key = Bytes::from(vec![*k]);
            let val = Bytes::from(vec![*v]);
            tx2_expected_size += key.len() + val.len() + STAGING_ENTRY_OVERHEAD_BYTES;
            mt.put(key, val, (i + 100) as u64, 200);
        }

        prop_assert_eq!(mt.size(), tx1_expected_size + tx2_expected_size);

        // Rollback Tx 1
        mt.rollback(100);

        prop_assert_eq!(mt.size(), tx2_expected_size);

        // Rollback Tx 2
        mt.rollback(200);

        prop_assert_eq!(mt.size(), 0);
    }
}
