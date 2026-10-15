use contextra_ports::{DeleteOutcome, PurgeState, TxId, PurgeId};

#[test]
fn test_purge_state_u8_encoding_golden() {
    assert_eq!(PurgeState::Begun.to_u8(), 0);
    assert_eq!(PurgeState::LedgerAppended.to_u8(), 1);
    assert_eq!(PurgeState::Tombstoned.to_u8(), 2);
    assert_eq!(PurgeState::PurgePending.to_u8(), 3);
    assert_eq!(PurgeState::Purging.to_u8(), 4);
    assert_eq!(PurgeState::Purged.to_u8(), 5);
    assert_eq!(PurgeState::KeyQuarantined.to_u8(), 6);
    assert_eq!(PurgeState::Completed.to_u8(), 7);
    assert_eq!(PurgeState::Attested.to_u8(), 8);

    for code in 0..=8u8 {
        let state = PurgeState::from_u8(code).expect("valid code");
        assert_eq!(state.to_u8(), code);
    }
    assert_eq!(PurgeState::from_u8(9), None);
}

#[test]
fn test_delete_outcome_variants() {
    let tx = TxId::try_new(42).unwrap();
    let purge_id = PurgeId::new([1u8; 16]);

    let logical = DeleteOutcome::Logical { tx };
    match logical {
        DeleteOutcome::Logical { tx: t } => assert_eq!(t, tx),
        _ => panic!("Expected Logical variant"),
    }

    let assured = DeleteOutcome::Assured { tx, purge_id };
    match assured {
        DeleteOutcome::Assured { tx: t, purge_id: pid } => {
            assert_eq!(t, tx);
            assert_eq!(pid, purge_id);
        }
        _ => panic!("Expected Assured variant"),
    }
}
