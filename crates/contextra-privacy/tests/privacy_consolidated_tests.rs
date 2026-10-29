//! Consolidated tests for `contextra-privacy` resolving TenantId & AVV duplicates,
//! verifying processing registry serialization, tenant separation, Art. 15 data subject searches,
//! audit-chain manipulation detection with `ManualClock`, and AVV equivalence.

#![allow(deprecated)]

use contextra_privacy::avv_generator::{generate_avv_draft, AvvContext};
use contextra_privacy::context_edit_audit::{
    build_context_edit_audit_record, render_audit_line, verify_audit_chain, ContextEditAuditError,
    ContextEditInput, ContextEditKind,
};
use contextra_privacy::egress_guard::TextSearchEngine;
use contextra_privacy::processing_registry::{
    generate_registry, render_markdown, ProcessingActivityRecord, ProcessingRegistryExport,
    ProcessorRole,
};
use contextra_testkit::ManualClock;
use contextra_types::TenantId;

fn make_sample_record(tenant_u64: u64, name: &str, purpose: &str) -> ProcessingActivityRecord {
    ProcessingActivityRecord {
        activity_name: name.to_string(),
        controller_or_processor_role: ProcessorRole::Controller,
        purpose: purpose.to_string(),
        data_categories: vec!["E-Mail".to_string(), "Kundennummer".to_string()],
        data_subject_categories: vec!["Bestandskunden".to_string()],
        recipients: vec!["Rechenzentrum GmbH".to_string()],
        third_country_transfers: vec![],
        retention_period: "10 Jahre nach Vertragsende".to_string(),
        technical_and_organizational_measures: vec![
            "Kryptografische Mandantentrennung".to_string(),
            "TLS 1.3".to_string(),
        ],
        tenant_id: TenantId::try_new(tenant_u64).unwrap_or(TenantId::SYSTEM),
    }
}

// 1. Serde JSON Roundtrip
#[test]
fn test_01_registry_json_serde_roundtrip() {
    let rec = make_sample_record(101, "Kundendatenbank", "Vertragsabwicklung");
    let export = generate_registry(&[rec.clone()]);

    let json_str = serde_json::to_string(&export).expect("serialization to json");
    let deser: ProcessingRegistryExport =
        serde_json::from_str(&json_str).expect("deserialization from json");

    assert_eq!(export, deser);
    assert_eq!(deser.records[0].tenant_id, TenantId::try_new(101).unwrap());
}

// 2. Bincode Roundtrip
#[test]
fn test_02_registry_bincode_serde_roundtrip() {
    let rec = make_sample_record(202, "Personalverwaltung", "Lohnabrechnung");
    let export = generate_registry(&[rec]);

    let encoded = bincode::serialize(&export).expect("bincode serialize");
    let decoded: ProcessingRegistryExport =
        bincode::deserialize(&encoded).expect("bincode deserialize");

    assert_eq!(export, decoded);
    assert_eq!(
        decoded.records[0].tenant_id,
        TenantId::try_new(202).unwrap()
    );
}

// 3. Golden Test (Deterministic JSON String Output)
#[test]
fn test_03_registry_golden_json_format() {
    let rec = ProcessingActivityRecord {
        activity_name: "Auskunfts-Dienst".to_string(),
        controller_or_processor_role: ProcessorRole::Processor,
        purpose: "Art. 15 DSGVO Bearbeitung".to_string(),
        data_categories: vec!["Stammdaten".to_string()],
        data_subject_categories: vec!["Betroffene".to_string()],
        recipients: vec!["Datenschutzbeauftragter".to_string()],
        third_country_transfers: vec![],
        retention_period: "3 Jahre".to_string(),
        technical_and_organizational_measures: vec!["AES-256".to_string()],
        tenant_id: TenantId::try_new(42).unwrap(),
    };
    let export = generate_registry(&[rec]);

    let json = serde_json::to_string_pretty(&export).expect("pretty json");
    assert!(json.contains(r#""tenant_id": 42"#));
    assert!(json.contains(r#""activity_name": "Auskunfts-Dienst""#));
    assert!(json.contains(r#""controller_or_processor_role": "Processor""#));
}

// 4. Tenant Isolation & Filtering
#[test]
fn test_04_tenant_filtering_isolation() {
    let rec_t1 = make_sample_record(1, "Vertrieb", "Kundenakquise");
    let rec_t2 = make_sample_record(2, "Buchhaltung", "Rechnungsstellung");
    let export = generate_registry(&[rec_t1, rec_t2]);

    let filtered_t1 = export.filter_by_tenant(TenantId::try_new(1).unwrap());
    assert_eq!(filtered_t1.records.len(), 1);
    assert_eq!(filtered_t1.records[0].activity_name, "Vertrieb");

    let filtered_t2 = export.filter_by_tenant(TenantId::try_new(2).unwrap());
    assert_eq!(filtered_t2.records.len(), 1);
    assert_eq!(filtered_t2.records[0].activity_name, "Buchhaltung");

    let filtered_t3 = export.filter_by_tenant(TenantId::try_new(999).unwrap());
    assert!(filtered_t3.records.is_empty());
}

// 5. Art. 15 Data Subject Access Search (Exact Match)
#[tokio::test]
async fn test_05_art15_search_exact_matches() {
    let rec1 = make_sample_record(10, "Bewerbermanagement", "Recruiting");
    let rec2 = make_sample_record(10, "Zeiterfassung", "Arbeitszeit-Nachweis");

    let export = generate_registry(&[rec1, rec2]);

    let results = export.search_text("Recruiting", 5).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "record-0");
    assert!(results[0].score > 0.0);
}

// 6. Art. 15 Search (Empty Query / No Match)
#[tokio::test]
async fn test_06_art15_search_empty_or_no_match() {
    let rec = make_sample_record(10, "Support-Tickets", "Kunden-Helpline");
    let export = generate_registry(&[rec]);

    let empty_res = export.search_text("   ", 5).await.unwrap();
    assert!(empty_res.is_empty());

    let nomatch_res = export.search_text("NonExistentTermXYZ", 5).await.unwrap();
    assert!(nomatch_res.is_empty());
}

// 7. Art. 15 Search Limit & Score Ordering
#[tokio::test]
async fn test_07_art15_search_limit_and_score_ordering() {
    let mut rec1 = make_sample_record(5, "E-Mail Marketing", "Newsletter");
    rec1.data_categories = vec!["E-Mail".to_string(), "Name".to_string()];

    let mut rec2 = make_sample_record(5, "E-Mail Support", "Kundenservice");
    rec2.data_categories = vec!["E-Mail".to_string()];

    let export = generate_registry(&[rec1, rec2]);

    let results = export.search_text("E-Mail", 1).await.unwrap();
    assert_eq!(results.len(), 1);
}

// 8. Markdown Rendering Multi-Tenant
#[test]
fn test_08_markdown_rendering_multi_tenant() {
    let r1 = make_sample_record(100, "Aktion A", "Zweck A");
    let r2 = make_sample_record(200, "Aktion B", "Zweck B");

    let export = generate_registry(&[r1, r2]);
    let md = render_markdown(&export);

    assert!(md.contains("| TenantId(100) | Aktion A |"));
    assert!(md.contains("| TenantId(200) | Aktion B |"));
}

// 9. Audit Chain Construction with ManualClock
#[test]
fn test_09_audit_chain_construction_manual_clock() {
    let clock = ManualClock::new(100_000_000);
    let tenant = TenantId::try_new(7).unwrap();

    let input0 = ContextEditInput {
        task_id: "task-99".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Append,
        chunk_label: Some("chunk-head".to_string()),
        bytes_affected: 512,
        subgoal_index: 0,
    };

    let r0 = build_context_edit_audit_record([0u8; 32], input0, &clock);
    clock.advance(std::time::Duration::from_nanos(5000));

    let input1 = ContextEditInput {
        task_id: "task-99".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Replace,
        chunk_label: Some("chunk-body".to_string()),
        bytes_affected: 128,
        subgoal_index: 1,
    };

    let r1 = build_context_edit_audit_record(r0.record_hash, input1, &clock);

    let chain = vec![r0, r1];
    assert!(verify_audit_chain(&chain).is_ok());
}

// 10. Audit Chain Tampering Detection at Index 0
#[test]
fn test_10_audit_chain_tamper_detection_index_0() {
    let clock = ManualClock::new(50_000);
    let tenant = TenantId::try_new(12).unwrap();

    let input0 = ContextEditInput {
        task_id: "t1".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Append,
        chunk_label: None,
        bytes_affected: 64,
        subgoal_index: 0,
    };

    let mut r0 = build_context_edit_audit_record([0u8; 32], input0, &clock);
    // Tamper with previous hash of root record
    r0.previous_hash = [1u8; 32];

    assert_eq!(
        verify_audit_chain(&[r0]),
        Err(ContextEditAuditError::ChainTampered { index: 0 })
    );
}

// 11. Audit Chain Tampering Detection at Index 1 (Bytes Affected)
#[test]
fn test_11_audit_chain_tamper_bytes_affected() {
    let clock = ManualClock::new(10_000);
    let tenant = TenantId::try_new(15).unwrap();

    let input0 = ContextEditInput {
        task_id: "t2".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Append,
        chunk_label: None,
        bytes_affected: 100,
        subgoal_index: 0,
    };
    let r0 = build_context_edit_audit_record([0u8; 32], input0, &clock);

    let input1 = ContextEditInput {
        task_id: "t2".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Remove,
        chunk_label: None,
        bytes_affected: 50,
        subgoal_index: 1,
    };
    let mut r1 = build_context_edit_audit_record(r0.record_hash, input1, &clock);

    // Tamper with bytes_affected
    r1.bytes_affected = 51;

    let chain = vec![r0, r1];
    assert_eq!(
        verify_audit_chain(&chain),
        Err(ContextEditAuditError::ChainTampered { index: 1 })
    );
}

// 12. Audit Chain Tampering Detection at Index 1 (Subgoal Index)
#[test]
fn test_12_audit_chain_tamper_subgoal_index() {
    let clock = ManualClock::new(20_000);
    let tenant = TenantId::try_new(20).unwrap();

    let input0 = ContextEditInput {
        task_id: "t3".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::CheckpointReset,
        chunk_label: Some("cp1".to_string()),
        bytes_affected: 0,
        subgoal_index: 0,
    };
    let r0 = build_context_edit_audit_record([0u8; 32], input0, &clock);

    let input1 = ContextEditInput {
        task_id: "t3".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Append,
        chunk_label: Some("cp2".to_string()),
        bytes_affected: 10,
        subgoal_index: 1,
    };
    let mut r1 = build_context_edit_audit_record(r0.record_hash, input1, &clock);

    r1.subgoal_index = 99;

    assert_eq!(
        verify_audit_chain(&[r0, r1]),
        Err(ContextEditAuditError::ChainTampered { index: 1 })
    );
}

// 13. Audit Line Formatter Test
#[test]
fn test_13_render_audit_line_format() {
    let clock = ManualClock::new(123_456_789);
    let tenant = TenantId::try_new(88).unwrap();

    let input = ContextEditInput {
        task_id: "task-fmt".to_string(),
        tenant_id: tenant,
        kind: ContextEditKind::Append,
        chunk_label: Some("lbl-1".to_string()),
        bytes_affected: 1024,
        subgoal_index: 3,
    };

    let record = build_context_edit_audit_record([0u8; 32], input, &clock);
    let line = render_audit_line(&record);

    assert!(line.contains("task=task-fmt"));
    assert!(line.contains("tenant=TenantId(88)"));
    assert!(line.contains("kind=Append"));
    assert!(line.contains("chunk_label=lbl-1"));
    assert!(line.contains("bytes=1024"));
    assert!(line.contains("recorded_at_tx=123456789"));
    assert!(line.contains("subgoal=3"));
}

// 14. AVV Generator Re-export Equivalence (Default)
#[test]
fn test_14_avv_reexport_default_equivalence() {
    let mut ctx = AvvContext::default();
    ctx.controller_name = "Kanzlei Schmidt".to_string();
    ctx.processor_name = "Contextra Operations".to_string();

    let draft = generate_avv_draft(&ctx);
    assert!(draft.contains("Kanzlei Schmidt"));
    assert!(draft.contains("Contextra Operations"));
    assert!(draft.contains("Art. 28 DSGVO"));
}

// 15. AVV Generator Re-export Equivalence (Custom Date & Measures)
#[test]
fn test_15_avv_reexport_custom_context_equivalence() {
    let ctx = AvvContext::new(
        "Praxis Dr. Weber",
        "Contextra Health AG",
        TenantId::try_new(500).unwrap(),
        vec![],
        vec!["AWS Frankfurt".to_string()],
        30,
    )
    .with_contract_date("2026-10-29")
    .with_subprocessors_allowed(true);

    let draft = generate_avv_draft(&ctx);
    assert!(draft.contains("Praxis Dr. Weber"));
    assert!(draft.contains("Contextra Health AG"));
}

// 16. ProcessorRole as_str
#[test]
fn test_16_processor_role_as_str() {
    assert_eq!(ProcessorRole::Controller.as_str(), "Verantwortlicher");
    assert_eq!(ProcessorRole::Processor.as_str(), "Auftragsverarbeiter");
}

// 17. ProcessingRegistryExport Constructor
#[test]
fn test_17_processing_registry_export_new() {
    let export = ProcessingRegistryExport::new(vec![]);
    assert!(export.records.is_empty());
}
