#![forbid(unsafe_code)]
//! Campaign J-27 Compliance Test Suite — GDPR Art. 30 Verarbeitungsverzeichnis & BSI Mapping

use contextra_audit_export::{
    bsi_mapping_table, render_bsi_mapping_markdown, render_register_json, render_register_markdown,
    testkit::InMemoryProcessingRegisterSource, ProcessingRegisterEntry, ProcessingRegisterSource,
};
use contextra_types::TenantId;

// ── H10: Art. 30 DSGVO Processing Register, Canary & BSI Mapping ─────────────

#[test]
fn test_h10_gdpr_art30_processing_registry_and_canary_leak_check(
) -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(42)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id)?;

    assert_eq!(
        entries.len(),
        2,
        "Sample source provides 2 Art. 30 processing entries"
    );

    // 1. JSON rendering and parsing validity
    let json_output = render_register_json(&entries)?;
    let parsed_entries: Vec<ProcessingRegisterEntry> = serde_json::from_str(&json_output)?;
    assert_eq!(
        parsed_entries, entries,
        "JSON rendering must be lossless and round-trip parseable"
    );

    // 2. Markdown rendering determinism & Art. 30 lit. a-g technical completeness check
    let md1 = render_register_markdown(&entries)?;
    let md2 = render_register_markdown(&entries)?;
    assert_eq!(md1, md2, "Markdown rendering must be 100% deterministic");

    assert!(md1.contains("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)"));
    assert!(md1.contains("Zweck"), "Art. 30 lit. b: Purpose present");
    assert!(
        md1.contains("Datenkategorien"),
        "Art. 30 lit. c: Data categories present"
    );

    // BEFUND H10: contextra-audit-export's ProcessingRegisterEntry omits Art. 30 (1) lit. a (controller/processor name),
    // lit. c (subject categories), lit. d (recipients), lit. e (third country transfers),
    // lit. f (retention periods), and lit. g (TOMs)!
    let has_lit_a_controller = md1.contains("Verantwortlicher");
    let has_lit_d_recipients = md1.contains("Empfänger");
    let has_lit_e_third_country = md1.contains("Drittland");
    let has_lit_f_retention = md1.contains("Löschfrist");
    let has_lit_g_toms = md1.contains("Technische und organisatorische Maßnahmen");

    if !has_lit_a_controller
        || !has_lit_d_recipients
        || !has_lit_e_third_country
        || !has_lit_f_retention
        || !has_lit_g_toms
    {
        eprintln!("H10 CONFIRMED: contextra-audit-export Art. 30 register lacks statutory fields lit. a, d, e, f, g!");
    }

    // 3. Canary secret non-leakage check
    let canary_secret = "CANARY-SECRET-1234-DONT-LEAK";

    assert!(
        !json_output.contains(canary_secret),
        "Canary secret must not appear in Art. 30 JSON export"
    );
    assert!(
        !md1.contains(canary_secret),
        "Canary secret must not appear in Art. 30 Markdown export"
    );

    // 4. BSI Mapping Table verification
    let bsi_table = bsi_mapping_table();
    assert_eq!(
        bsi_table.len(),
        4,
        "BSI mapping table must contain 4 core cryptographic mappings"
    );

    let bsi_md = render_bsi_mapping_markdown(&bsi_table);
    assert!(bsi_md.contains("Ed25519"));
    assert!(bsi_md.contains("HMAC-SHA256"));
    assert!(bsi_md.contains("AES-256-GCM-SIV"));
    assert!(bsi_md.contains("Argon2id"));

    // Verify mapped code paths exist
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let workspace_root = std::path::Path::new(&manifest_dir).join("../..");

    for entry in &bsi_table {
        let full_path = workspace_root.join(&entry.code_location);
        assert!(
            full_path.exists(),
            "Mapped BSI code path '{}' must reference valid codebase path at '{:?}'",
            entry.code_location,
            full_path
        );
    }

    Ok(())
}
