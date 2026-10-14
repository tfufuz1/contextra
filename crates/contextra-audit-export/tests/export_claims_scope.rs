#![forbid(unsafe_code)]
//! FILE-CONTEXT (STAND 2026-10-13, AUFTRAG B07: Audit-Export Geltungsbereich & Nachweisdarstellung)
//!
//! Verification test suite for Audit-Export:
//! - Exact scoping of layer statuses (`verified`, `not_covered`, `observed_only`)
//! - Secondary copies statuses (`deleted`, `unreachable`, `out_of_scope` with reason)
//! - Mandatory "Grenzen" section (Garg et al. tolerance, Wei et al. SSD/Flash limits, observed vs guaranteed)
//! - Dual timestamps (Anforderungszeitpunkt and Vernichtungszeitpunkt) and no cleartext IDs
//! - Correct standards references (DIN EN ISO/IEC 27555:2025-09 and NIST SP 800-88 Rev. 2, no DIN 66398)

use contextra_audit_export::{
    bsi_mapping_table, render_art30_register_markdown, render_bsi_mapping_markdown,
    render_register_markdown, Art30ProcessingRegisterEntry, DeletionProofSummary,
    EgressEventSummary, ProcessingRegisterEntry,
};
use contextra_types::TenantId;

#[test]
fn test_report_without_layer_proof_contains_no_geloescht() {
    let tenant_id = TenantId::try_new(42).unwrap();
    let entry_without_proofs = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Testzweck ohne Löschnachweis".to_string(),
        data_categories: vec!["Vertragsdaten".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. b DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![],
        generated_at: 1_000_000,
    };

    let md = render_register_markdown(&[entry_without_proofs]).unwrap();

    // Extract the Layer Status section
    let layer_section = md
        .split("## Status der Speicher-Layer")
        .nth(1)
        .unwrap_or("")
        .split("## Status sekundärer Kopien")
        .next()
        .unwrap_or("");

    // Without verified layer proof, no layer may claim "gelöscht" or "verified"
    assert!(
        !layer_section.contains("gelöscht"),
        "Report without layer proof MUST NOT contain 'gelöscht' in Layer Status section"
    );
    assert!(
        !layer_section.contains("Status: `verified`"),
        "Report without layer proof MUST NOT claim status 'verified'"
    );
    assert!(
        layer_section.contains("not_covered") || layer_section.contains("observed_only"),
        "Layer status without proof MUST report not_covered or observed_only"
    );
}

#[test]
fn test_mandatory_grenzen_section_always_present() {
    let tenant_id = TenantId::try_new(100).unwrap();
    let entry = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Test Grenzensicherung".to_string(),
        data_categories: vec!["Protokolldaten".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![],
        generated_at: 2_000_000,
    };

    let md_reg = render_register_markdown(&[entry]).unwrap();
    let md_art30 = render_art30_register_markdown(&[]).unwrap();
    let md_bsi = render_bsi_mapping_markdown(&bsi_mapping_table());

    for (name, markdown) in [
        ("render_register_markdown", md_reg),
        ("render_art30_register_markdown", md_art30),
        ("render_bsi_mapping_markdown", md_bsi),
    ] {
        assert!(
            markdown.contains("## Grenzen der Nachweisbarkeit"),
            "{} MUST contain section '## Grenzen der Nachweisbarkeit'",
            name
        );
        assert!(
            markdown.contains("Garg et al."),
            "{} MUST mention Garg et al. (no proof of absence)",
            name
        );
        assert!(
            markdown.contains("Wei et al."),
            "{} MUST mention Wei et al. (SSD/Flash overwrite non-guarantee)",
            name
        );
        assert!(
            markdown.contains("beobachtet") && markdown.contains("garantiert"),
            "{} MUST distinguish observed vs guaranteed environment values",
            name
        );
    }
}

#[test]
fn test_din_66398_not_present_in_code_or_rendered_output() {
    let table = bsi_mapping_table();
    let md_bsi = render_bsi_mapping_markdown(&table);

    assert!(
        !md_bsi.contains("DIN 66398"),
        "Withdrawn DIN 66398 MUST NOT appear in rendered output"
    );

    for entry in &table {
        assert!(
            !entry.note.contains("DIN 66398"),
            "Withdrawn DIN 66398 MUST NOT appear in mapping entry notes"
        );
    }

    assert!(
        md_bsi.contains("DIN EN ISO/IEC 27555:2025-09"),
        "DIN EN ISO/IEC 27555:2025-09 MUST appear as active deletion standard"
    );
    assert!(
        md_bsi.contains("NIST SP 800-88 Rev. 2"),
        "NIST SP 800-88 Rev. 2 MUST appear as media sanitization reference"
    );
}

#[test]
fn test_layer_status_and_copies_status_formatting() {
    let tenant_id = TenantId::try_new(200).unwrap();
    let verified_proof = DeletionProofSummary {
        proof_id: "a1b2c3d4e5f60718".to_string(),
        scope: "/var/data/tenants/t200/doc_secret.bin".to_string(),
        timestamp_nanos: 1_700_000_100_000_000_000,
        verified: true,
    };

    let entry_with_proof = ProcessingRegisterEntry {
        tenant_id,
        processing_purpose: "Test mit verifiziertem Beleg".to_string(),
        data_categories: vec!["Geheimdaten".to_string()],
        legal_basis: "Art. 17 DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![verified_proof],
        generated_at: 3_000_000,
    };

    let md = render_register_markdown(&[entry_with_proof]).unwrap();

    // Verify presence of layer statuses
    assert!(md.contains("Status: `verified`"));
    assert!(md.contains("Layer 1 (Ring-0 In-Memory)"));
    assert!(md.contains("Layer 4 (SSTable / Persistent Disk)"));
    assert!(md.contains("Layer 7 (Key Vault / DEK Registry)"));

    // Verify copies status section
    assert!(md.contains("## Status sekundärer Kopien"));
    assert!(md.contains("Checkpoints & Hardlinks"));
    assert!(md.contains("out_of_scope"));
    assert!(md.contains("Grund:"));
}

#[test]
fn test_two_timestamps_and_no_cleartext_ids() {
    let tenant_id = TenantId::try_new(300).unwrap();
    let sensitive_path = "/secret/customer_data/file_001.txt";
    let verified_proof = DeletionProofSummary {
        proof_id: "deadbeef12345678".to_string(),
        scope: sensitive_path.to_string(),
        timestamp_nanos: 1_700_000_500_000_000_000,
        verified: true,
    };

    let entry = Art30ProcessingRegisterEntry {
        tenant_id,
        controller_details: "Datenschutzbeauftragter Test".to_string(),
        processing_purpose: "Audit Dual Timestamps".to_string(),
        data_subject_categories: vec!["Nutzer".to_string()],
        data_categories: vec!["Nutzer-ID".to_string()],
        recipient_categories: vec!["Interne Verarbeitung".to_string()],
        third_country_transfers: None,
        erasure_deadlines: "Sofort".to_string(),
        technical_organizational_measures: vec!["Crypto-Erase".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. c DSGVO".to_string(),
        egress_events: vec![EgressEventSummary {
            event_id: "evt-999".to_string(),
            destination: "https://audit.local".to_string(),
            timestamp_nanos: 1_000,
            detail: "Audit event".to_string(),
        }],
        deletion_proofs: vec![verified_proof],
        generated_at: 4_000_000,
    };

    let md = render_art30_register_markdown(&[entry]).unwrap();

    // 1. Check dual timestamps
    assert!(
        md.contains("Anforderungszeitpunkt (Wartefrist)"),
        "Markdown MUST display Anforderungszeitpunkt (request time)"
    );
    assert!(
        md.contains("Vernichtungszeitpunkt (Vernichtung)"),
        "Markdown MUST display Vernichtungszeitpunkt (destruction time)"
    );

    // 2. Check no raw sensitive scope in cleartext
    assert!(
        !md.contains(sensitive_path),
        "Raw sensitive path MUST NOT appear in cleartext"
    );
    assert!(
        md.contains("anonymisiert"),
        "Scope MUST be masked / anonymized in output"
    );
}
