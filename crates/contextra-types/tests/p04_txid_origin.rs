#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use contextra_types::{ContextraError, TxId};

#[test]
fn test_txid_try_new_valid_ranges() -> contextra_types::Result<()> {
    // 1. Collection sequence range [1, MAX_COLLECTION_SEQUENCE]
    let tx_col = TxId::try_new(100)?;
    assert_eq!(tx_col.inner(), 100);
    assert!(tx_col.is_valid_origin());

    let tx_col_max = TxId::try_new(TxId::MAX_COLLECTION_SEQUENCE)?;
    assert_eq!(tx_col_max.inner(), TxId::MAX_COLLECTION_SEQUENCE);
    assert!(tx_col_max.is_valid_origin());

    // 2. Internal system range [INTERNAL_BASE, u64::MAX]
    let tx_int_base = TxId::try_new(TxId::INTERNAL_BASE)?;
    assert_eq!(tx_int_base.inner(), TxId::INTERNAL_BASE);
    assert!(tx_int_base.is_valid_origin());

    let tx_int_offset = TxId::try_new(TxId::INTERNAL_BASE + 500)?;
    assert_eq!(tx_int_offset.inner(), TxId::INTERNAL_BASE + 500);
    assert!(tx_int_offset.is_valid_origin());

    let tx_max = TxId::try_new(u64::MAX)?;
    assert_eq!(tx_max.inner(), u64::MAX);
    assert!(tx_max.is_valid_origin());

    // 3. TxId::INVALID / 0
    let tx_zero = TxId::try_new(0)?;
    assert_eq!(tx_zero, TxId::INVALID);
    assert!(tx_zero.is_valid_origin());

    // 4. TryFrom<u64> conversion
    let tx_try_from: TxId = 42u64.try_into()?;
    assert_eq!(tx_try_from.inner(), 42);

    Ok(())
}

#[test]
fn test_txid_try_new_rejects_invalid_origin_gap() {
    // Wall-clock derived timestamp in nanoseconds (~1.7e18) falls into the unmanaged gap
    let wall_clock_nanos = 1_700_000_000_000_000_000u64;
    let res = TxId::try_new(wall_clock_nanos);
    assert!(
        res.is_err(),
        "TxId::try_new must reject wall-clock derived TxId"
    );

    match res {
        Err(ContextraError::Transaction(msg)) => {
            assert!(
                msg.contains("AGT-GRAPH-001"),
                "Error message must reference AGT-GRAPH-001, got: {msg}"
            );
        }
        other => panic!("Expected ContextraError::Transaction, got: {:?}", other),
    }

    // Boundary right above MAX_COLLECTION_SEQUENCE
    let gap_start = TxId::MAX_COLLECTION_SEQUENCE + 1;
    let res_start = TxId::try_new(gap_start);
    assert!(res_start.is_err(), "TxId::try_new must reject gap_start");

    // Boundary right below INTERNAL_BASE
    let gap_end = TxId::INTERNAL_BASE - 1;
    let res_end = TxId::try_new(gap_end);
    assert!(res_end.is_err(), "TxId::try_new must reject gap_end");
}
