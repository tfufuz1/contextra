use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer, TxBufferConfig, STAGING_ENTRY_OVERHEAD_BYTES};
use contextra_types::{ContextraError, DocId, TxId};
use std::time::Duration;

#[test]
fn tx_buffer_byte_budget_exceeded() {
    assert_eq!(STAGING_ENTRY_OVERHEAD_BYTES, 32);

    let config = TxBufferConfig {
        max_tx_staged_bytes: 1024, // 1 KiB limit per tx
        max_total_staged_bytes: 4096,
        ..Default::default()
    };
    let buffer = TxBuffer::<Vec<u8>>::new_with_config_ext(4, Duration::from_secs(30), config);
    let tx = TxId::new(1);
    buffer.begin(tx);

    // Payload of 500 bytes + STAGING_ENTRY_OVERHEAD_BYTES (32 bytes) = 532 bytes
    let payload = vec![0u8; 500];
    let res1 = buffer.stage(
        tx,
        IndexOp::Insert {
            doc_id: DocId::from(100u64),
            data: payload.clone(),
        },
    );
    assert!(res1.is_ok());

    // Second payload of 500 bytes + 32 = 532 bytes (Total: 1064 bytes > 1024 max_tx_staged_bytes)
    let res2 = buffer.stage(
        tx,
        IndexOp::Insert {
            doc_id: DocId::from(101u64),
            data: payload,
        },
    );

    assert!(matches!(res2, Err(ContextraError::Storage(_))));
}

#[test]
fn tx_buffer_byte_budget_freed_on_drain_discard_reap() {
    let config = TxBufferConfig {
        max_tx_staged_bytes: 1024,
        max_total_staged_bytes: 2048,
        ..Default::default()
    };
    let buffer = TxBuffer::<Vec<u8>>::new_with_config_ext(4, Duration::from_millis(10), config);

    let tx1 = TxId::new(1);
    let tx2 = TxId::new(2);

    buffer.begin(tx1);
    buffer.begin(tx2);

    let payload = vec![0u8; 500]; // 500 + 32 = 532 bytes
    assert!(buffer
        .stage(
            tx1,
            IndexOp::Insert {
                doc_id: DocId::from(1u64),
                data: payload.clone()
            }
        )
        .is_ok());

    assert_eq!(buffer.staged_bytes(), 532);

    // Drain tx1
    buffer.drain(tx1);
    assert_eq!(buffer.staged_bytes(), 0);

    // Stage tx2
    assert!(buffer
        .stage(
            tx2,
            IndexOp::Insert {
                doc_id: DocId::from(2u64),
                data: payload
            }
        )
        .is_ok());

    assert_eq!(buffer.staged_bytes(), 532);

    // Discard tx2
    buffer.discard(tx2);
    assert_eq!(buffer.staged_bytes(), 0);
}
