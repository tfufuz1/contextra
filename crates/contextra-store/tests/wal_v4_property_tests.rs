use contextra_core::TxId;
use contextra_store::wal::{WalEntry, WalOp};

#[test]
fn test_wal_v4_encode_decode_identity_all_ops() {
    let integrity_key = [7u8; 32];
    let prev_hmac = [12u8; 32];

    let ops = vec![
        WalOp::Put {
            tx_id: TxId::new(100),
            key: b"sample_key".to_vec(),
            value: b"sample_value".to_vec(),
        },
        WalOp::Delete {
            tx_id: TxId::new(101),
            key: b"deleted_key".to_vec(),
        },
        WalOp::TxEnd {
            tx_id: TxId::new(102),
            committed: true,
        },
        WalOp::TxEnd {
            tx_id: TxId::new(103),
            committed: false,
        },
    ];

    for (seq_no, op) in ops.into_iter().enumerate() {
        let entry = WalEntry::try_new(op.clone(), seq_no as u64 + 1, &integrity_key, prev_hmac)
            .expect("create entry");

        let serialized = entry.to_bytes().expect("serialize entry");
        let deserialized =
            WalEntry::from_bytes_classified(&serialized[4..]).expect("deserialize entry");

        assert_eq!(deserialized.seq_no, entry.seq_no);
        assert_eq!(deserialized.checksum, entry.checksum);
        assert_eq!(deserialized.prev_hmac, entry.prev_hmac);
        assert_eq!(deserialized.op, op);
    }
}
