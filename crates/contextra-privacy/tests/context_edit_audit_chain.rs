// FILE-CONTEXT
// ZWECK: Integrationstest für ContextEditAuditRecord-Verkettung, Hash-Integrität, Clock-Port-Injektion und Manipulationserkennung.
// INVARIANTEN: Zeitstempel wird ausschließlich über Clock bezogen;
//              Kein Klartext des Scratchpad-Inhalts in den Records;
//              Verifizierung liefert exakt den ersten fehlerhaften Index bei Manipulation.

#![forbid(unsafe_code)]
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use contextra_ports::Clock;
use contextra_privacy::context_edit_audit::{
    build_context_edit_audit_record, render_audit_line, verify_audit_chain, ContextEditAuditError,
    ContextEditInput, ContextEditKind,
};
use contextra_types::TenantId;

struct MockSequenceClock {
    timestamps: std::sync::atomic::AtomicU64,
}

impl MockSequenceClock {
    fn new(start_nanos: u64) -> Self {
        Self {
            timestamps: std::sync::atomic::AtomicU64::new(start_nanos),
        }
    }
}

impl Clock for MockSequenceClock {
    fn now_unix_nanos(&self) -> u64 {
        self.timestamps
            .fetch_add(1000, std::sync::atomic::Ordering::Relaxed)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.now_unix_nanos()
    }
}

#[test]
fn test_audit_chain_validity_and_no_plaintext_leak() {
    let tenant = TenantId(42);
    let clock = MockSequenceClock::new(1_700_000_000_000_000_000);

    let scratchpad_plaintext_chunk_01 = "GEHEIM: Interner Systemprompt / API Keyskj389h2d";
    let scratchpad_plaintext_chunk_02 = "GEHEIM: Zweiter vertraulicher Textabschnitt";

    // Record 0: Append chunk 01
    let input0 = ContextEditInput {
        task_id: "task-alpha".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Append,
        chunk_label: Some("chunk-01".to_string()),
        bytes_affected: scratchpad_plaintext_chunk_01.len(),
        subgoal_index: 0,
    };
    let rec0 = build_context_edit_audit_record([0u8; 32], input0, &clock);

    // Record 1: Replace chunk 02
    let input1 = ContextEditInput {
        task_id: "task-alpha".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Replace,
        chunk_label: Some("chunk-02".to_string()),
        bytes_affected: scratchpad_plaintext_chunk_02.len(),
        subgoal_index: 1,
    };
    let rec1 = build_context_edit_audit_record(rec0.record_hash, input1, &clock);

    // Record 2: CheckpointReset
    let input2 = ContextEditInput {
        task_id: "task-alpha".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::CheckpointReset,
        chunk_label: None,
        bytes_affected: 0,
        subgoal_index: 2,
    };
    let rec2 = build_context_edit_audit_record(rec1.record_hash, input2, &clock);

    let chain = vec![rec0.clone(), rec1.clone(), rec2.clone()];

    // 1. Valid chain verification
    assert!(
        verify_audit_chain(&chain).is_ok(),
        "Audit chain must verify successfully"
    );

    // 2. No plaintext leakage check
    for rec in &chain {
        let line = render_audit_line(rec);
        assert!(
            !line.contains(scratchpad_plaintext_chunk_01),
            "Audit line leaked plaintext chunk 1"
        );
        assert!(
            !line.contains(scratchpad_plaintext_chunk_02),
            "Audit line leaked plaintext chunk 2"
        );
    }
}

#[test]
fn test_audit_chain_tampering_at_index_i() {
    let tenant = TenantId(100);
    let clock = MockSequenceClock::new(10_000_000);

    let mut records = Vec::new();
    let mut prev_hash = [0u8; 32];

    for i in 0..5 {
        let input = ContextEditInput {
            task_id: "task-beta".to_string(),
            tenant_id: tenant,
            kind: ContextEditKind::Append,
            chunk_label: Some(format!("step-{}", i)),
            bytes_affected: 64 * (i + 1),
            subgoal_index: i as u64,
        };
        let rec = build_context_edit_audit_record(prev_hash, input, &clock);
        prev_hash = rec.record_hash;
        records.push(rec);
    }

    assert!(verify_audit_chain(&records).is_ok());

    // Tamper with record at index 2
    let tampered_index = 2;
    records[tampered_index].bytes_affected += 1;

    let res = verify_audit_chain(&records);
    assert_eq!(
        res,
        Err(ContextEditAuditError::ChainTampered {
            index: tampered_index
        })
    );
}
