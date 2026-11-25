// FILE-CONTEXT
// ZWECK: Tests für CLM Scratchpad Reset-Flow, RAM-Zeroization, Audit-/Invalidierungscounter und PinnedRegion.
// INVARIANTEN: RAM nach Reset geleert (0 Bytes), PurgeReceipt vorhanden, Audit/Invalidator genau 1, PinnedRegion unverändert.

#![forbid(unsafe_code)]

use contextra_agent::{
    ClmScratchpad, CountingContextEditAuditSink, CountingScratchpadCacheInvalidator,
    PinnedRegionId, ScratchpadEditOp,
};
use contextra_db::volatile_vault::VaultConfig;
use contextra_ports::SystemClock;
use contextra_types::{TenantId, TxId};
use std::sync::Arc;

#[test]
fn test_scratchpad_reset_flow_and_invariants() {
    let clock = Arc::new(SystemClock::new());
    let audit_sink = Arc::new(CountingContextEditAuditSink::new());
    let cache_invalidator = Arc::new(CountingScratchpadCacheInvalidator::new());

    let vault_config = VaultConfig {
        max_capacity_bytes: 1024 * 1024,
        attempt_mlock: false,
    };

    let tenant_id = TenantId::try_new(42).unwrap();
    let pinned = PinnedRegionId::new("sys_prompt_v1");

    let mut scratchpad = ClmScratchpad::new("task-reset-flow".to_string(), vault_config)
        .with_tenant_id(tenant_id)
        .with_clock(clock.clone())
        .with_audit_sink(audit_sink.clone())
        .with_cache_invalidator(cache_invalidator.clone())
        .with_pinned_region(pinned.clone());

    // Verify initial setup
    assert_eq!(scratchpad.tenant_id(), tenant_id);
    assert_eq!(scratchpad.pinned_regions().len(), 1);
    assert_eq!(scratchpad.pinned_regions()[0], pinned);
    assert_eq!(scratchpad.total_bytes(), 0);
    assert_eq!(scratchpad.current_subgoal_index(), 0);

    // Apply 2 edits
    scratchpad
        .apply_edit(
            ScratchpadEditOp::Append {
                content: b"First chunk of subgoal 0".to_vec(),
                label: Some("chunk1".to_string()),
            },
            TxId(10),
        )
        .unwrap();

    scratchpad
        .apply_edit(
            ScratchpadEditOp::Append {
                content: b"Second chunk of subgoal 0".to_vec(),
                label: Some("chunk2".to_string()),
            },
            TxId(11),
        )
        .unwrap();

    let bytes_before = b"First chunk of subgoal 0".len() + b"Second chunk of subgoal 0".len();
    assert_eq!(scratchpad.total_bytes(), bytes_before);
    assert_eq!(audit_sink.count(), 2);

    // Execute checkpoint_and_reset()
    let checkpoint = scratchpad.checkpoint_and_reset().unwrap();

    // Verification 1: Checkpoint data & PurgeReceipt
    assert_eq!(checkpoint.task_id, "task-reset-flow");
    assert_eq!(checkpoint.completed_subgoal_index, 0);
    assert_eq!(checkpoint.chunk_count_at_checkpoint, 2);
    assert_eq!(checkpoint.purge_receipt.chunks_purged, 2);
    assert_eq!(checkpoint.purge_receipt.bytes_zeroed, bytes_before);

    // Verification 2: RAM zeroized/emptied (total_bytes is 0) and subgoal_index incremented
    assert_eq!(scratchpad.total_bytes(), 0);
    assert_eq!(scratchpad.current_subgoal_index(), 1);

    // Verification 3: Audit sink and Cache invalidator received reset notifications
    // Edits count (2) + reset audit count (1) = 3 total edits
    assert_eq!(audit_sink.count(), 3);
    // Cache invalidator called EXACTLY 1 time for the reset
    assert_eq!(cache_invalidator.count(), 1);

    // Verification 4: PinnedRegion remains completely unchanged and protected
    assert_eq!(scratchpad.pinned_regions().len(), 1);
    assert_eq!(scratchpad.pinned_regions()[0], pinned);

    // Attempting to modify or remove pinned region still yields Err
    let err = scratchpad.apply_edit(
        ScratchpadEditOp::Remove {
            chunk_label: "sys_prompt_v1".to_string(),
        },
        TxId(12),
    );
    assert!(err.is_err());
}
