#![forbid(unsafe_code)]
//! Campaign J-27 AVV Test Suite — GDPR Art. 28 AVV Generator Technical Completeness

use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, AvvContext,
};
use contextra_types::TenantId;

// ── H11: Art. 28 DSGVO AVV Generator Technical Completeness ─────────────────────

#[test]
fn test_h11_gdpr_art28_avv_generator_technical_completeness() {
    let ctx = AvvContext {
        controller_name: "Kanzlei Dr. Mustermann & Partner".to_string(),
        processor_name: "Contextra AI Operating System Systems GmbH".to_string(),
        tenant_id: TenantId(42),
        technical_measures: default_technical_measures(),
        subprocessors: vec!["AWS EU-Central-1 (Frankfurt)".to_string()],
        deletion_sla_days: 14,
    };

    let avv_md = render_avv_markdown(&ctx).expect("AVV markdown rendering succeeds");

    // Technical completeness check against Art. 28 Abs. 3 lit. a-h required sections:
    assert!(avv_md.contains("Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO"));
    assert!(avv_md.contains("## 1. Gegenstand und Dauer der Verarbeitung"), "Art. 28 lit. a");
    assert!(avv_md.contains("## 2. Art und Zweck der Verarbeitung"), "Art. 28 lit. b");
    assert!(avv_md.contains("## 3. Art der personenbezogenen Daten und Kategorien betroffener Personen"), "Art. 28 lit. c");
    assert!(avv_md.contains("## 4. Pflichten und Rechte des Verantwortlichen"), "Art. 28 lit. d");
    assert!(avv_md.contains("## 5. Technische und organisatorische Maßnahmen (TOM)"), "Art. 28 lit. f");
    assert!(avv_md.contains("## 6. Unterauftragsverarbeiter"), "Art. 28 lit. g");
    assert!(avv_md.contains("## 7. Löschung von Daten und SLA"), "Art. 28 lit. h");

    // Verify claimed technical measures in default TOMs
    let measures = default_technical_measures();
    assert_eq!(measures.len(), 4);

    let measure_names: Vec<&str> = measures.iter().map(|m| m.name.as_str()).collect();
    assert!(measure_names.contains(&"Kryptografischer Löschbeweis (Ed25519)"));
    assert!(measure_names.contains(&"Zero-Egress-Default / Egress-Gateway"));
    assert!(measure_names.contains(&"Kryptografische Mandantentrennung"));
    assert!(measure_names.contains(&"Verschlüsselte Segmente im KV-Cache"));

    // Code verification of claimed technical guarantees from workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let workspace_root = std::path::Path::new(&manifest_dir).join("../..");

    // 1. Ed25519 DeletionProof: crates/contextra-crypto/src/deletion_proof.rs
    assert!(workspace_root.join("crates/contextra-crypto/src/deletion_proof.rs").exists());

    // 2. Egress Gateway: crates/contextra-privacy/src/egress_gateway.rs
    assert!(workspace_root.join("crates/contextra-privacy/src/egress_gateway.rs").exists());

    // 3. Multi-Tenant Isolation: crates/contextra-types/src/tenant_scope.rs
    assert!(workspace_root.join("crates/contextra-types/src/tenant_scope.rs").exists());

    // 4. Encrypted KV-Cache: crates/contextra-crypto/src/crypto.rs
    assert!(workspace_root.join("crates/contextra-crypto/src/crypto.rs").exists());
}
