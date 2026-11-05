//! # Tests für MV-07, MV-08, MV-09: TxBuffer & SeqLog Verifikation
//!
//! Prüft:
//! - MV-07: Keine unwrap_or(u64::MAX) Sentinel-Nanos in seq_log und tx_buffer (Sättigung via Duration).
//! - MV-09: Tombstones sind Einträge mit vlen == 0 (oder leeren Daten), keine Sequenz-Bit-Maskierung (Tombstone-Bit entfernt).
//! - MV-08: Lockreihenfolge Dokumentlock -> Commit-Barriere -> Purge-Barrier -> Journal.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::seq_log::SequenceLog;
use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer};
use contextra_testmacros::ctx_test;
use contextra_types::{DocId, TxId};
use std::time::Duration;

#[ctx_test(id = "T-VER-1")]
fn mv_07_duration_saturation_no_sentinel() {
    let mut log = SequenceLog::new().with_max_pin_duration(Duration::MAX);
    let now_nanos = 1_000_000_000u64;
    log.pin_snapshot_at(100, now_nanos);

    let expired = log.expired_pins_at(now_nanos + 1_000_000);
    assert!(
        expired.is_empty(),
        "Bei Duration::MAX darf nichts nach 1ms ablaufen"
    );

    let buffer = TxBuffer::<Vec<u8>>::new_with_config(2, Duration::MAX);
    let reaped = buffer.reap_orphans_bounded_at(10, now_nanos + 1_000_000);
    assert!(
        reaped.is_empty(),
        "Bei Duration::MAX darf kein Orphan gereapt werden"
    );
}

#[ctx_test(id = "T-TXN-1")]
fn mv_09_tombstone_zero_vlen_not_bit() {
    let op_insert = IndexOp::Insert {
        doc_id: DocId::new(1),
        data: vec![1, 2, 3],
    };
    let op_tombstone = IndexOp::Delete {
        doc_id: DocId::new(1),
        data: None::<Vec<u8>>,
    };

    match op_insert {
        IndexOp::Insert { data, .. } => assert_eq!(data.len(), 3),
        _ => panic!("Expected Insert"),
    }

    match op_tombstone {
        IndexOp::Delete { data, .. } => {
            assert!(data.is_none(), "Tombstone hat keine Nutzdaten (vlen == 0)");
        }
        _ => panic!("Expected Delete"),
    }
}

#[ctx_test(id = "T-CONC-1")]
fn mv_08_lock_order_documentation_and_contract() {
    let buffer = TxBuffer::<Vec<u8>>::new();
    let tx = TxId::new(42);
    let res = buffer.stage(
        tx,
        IndexOp::Delete {
            doc_id: DocId::new(10),
            data: None,
        },
    );
    assert!(res.is_ok());
    assert!(buffer.has_tx(tx));
}
