use super::*;

#[test]
fn test_deprecated_global_orphan_path_warns() {
    #[allow(deprecated)]
    {
        clear_all_orphaned_checkpoints();
        register_pinned_seq_no_orphan(PinnedSeqNoOrphan {
            seq_no: 99999,
            timestamp_ms: 1000,
        });
        let _orphans = get_orphaned_checkpoints();
        clear_all_orphaned_checkpoints();
    }
}

#[test]
fn timestamp_ms_is_monotonic() {
    let clock = SystemClock::new();
    let t1 = clock_timestamp_ms(&clock);
    let t2 = clock_timestamp_ms(&clock);
    assert!(t2 >= t1, "Timestamp must be monotonic");
}

#[test]
fn test_instance_orphan_registry_drain_pins_and_checkpoints() {
    let registry = InstanceOrphanRegistry::new("");
    registry.register_orphan_sync(PinnedSeqNoOrphan {
        seq_no: 100,
        timestamp_ms: 1000,
    });
    registry.register_checkpoint_sync(StateCheckpoint {
        tx_id: TxId::new(200),
        timestamp_ms: 2000,
        namespace: Some("test_drain".to_string()),
    });

    assert_eq!(registry.get_orphan_pins().len(), 1);
    assert_eq!(registry.get_orphaned_checkpoints().len(), 1);

    let drained_pins = registry.drain_orphan_pins();
    assert_eq!(drained_pins.len(), 1);
    assert_eq!(drained_pins[0].seq_no, 100);
    assert!(registry.get_orphan_pins().is_empty());

    let drained_cps = registry.drain_orphaned_checkpoints();
    assert_eq!(drained_cps.len(), 1);
    assert_eq!(drained_cps[0].tx_id, TxId::new(200));
    assert!(registry.get_orphaned_checkpoints().is_empty());
}

#[test]
fn test_instance_orphan_registry_clear_nonexistent() {
    let registry = InstanceOrphanRegistry::new("");
    registry.register_orphan_sync(PinnedSeqNoOrphan {
        seq_no: 50,
        timestamp_ms: 100,
    });
    registry.register_checkpoint_sync(StateCheckpoint {
        tx_id: TxId::new(60),
        timestamp_ms: 200,
        namespace: None,
    });

    // Clearing non-existent pin or checkpoint should preserve existing ones
    registry.clear_orphan_pin(999);
    registry.clear_orphaned_checkpoint(TxId::new(999));

    assert_eq!(registry.get_orphan_pins().len(), 1);
    assert_eq!(registry.get_orphaned_checkpoints().len(), 1);
}

#[test]
fn test_instance_orphan_registry_empty_path_persistence_noop() {
    let registry = InstanceOrphanRegistry::new("");
    assert!(registry.persist_sync().is_ok());

    let state = OrphanState::default();
    assert!(state.persist_sync().is_ok());
}

#[test]
fn test_two_instance_registries_independent() {
    let dir1 = tempfile::tempdir().expect("tempdir 1");
    let dir2 = tempfile::tempdir().expect("tempdir 2");

    let path1 = dir1.path().join("orphans1.json");
    let path2 = dir2.path().join("orphans2.json");

    let reg1 = InstanceOrphanRegistry::new(&path1);
    let reg2 = InstanceOrphanRegistry::new(&path2);

    reg1.register_orphan_sync(PinnedSeqNoOrphan {
        seq_no: 101,
        timestamp_ms: 1000,
    });

    reg2.register_checkpoint_sync(StateCheckpoint {
        tx_id: TxId::new(202),
        timestamp_ms: 2000,
        namespace: Some("ns2".to_string()),
    });

    assert_eq!(reg1.get_orphan_pins().len(), 1);
    assert_eq!(reg1.get_orphaned_checkpoints().len(), 0);

    assert_eq!(reg2.get_orphan_pins().len(), 0);
    assert_eq!(reg2.get_orphaned_checkpoints().len(), 1);
}

proptest::proptest! {
    #[test]
    fn prop_monotonic_timestamp_ms_increases_or_equals(_n: u8) {
        let clock = SystemClock::new();
        let ts1 = clock_timestamp_ms(&clock);
        let ts2 = clock_timestamp_ms(&clock);
        proptest::prop_assert!(ts2 >= ts1);
    }
}
