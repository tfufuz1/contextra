#![forbid(unsafe_code)]

use contextra_avv_generator::{AvvContext, TechnicalMeasure};
use contextra_types::TenantId;

#[test]
fn test_avv_context_new_production_path_wires_default_measures_and_renders() {
    let tenant_id = TenantId(777);
    // Exercise production path via AvvContext::new and AvvContext::render
    // Note: default_technical_measures and render_avv_markdown are NOT called directly here.
    let ctx = AvvContext::new("Kanzlei Dr. Weber", "Contextra Cloud Platform", tenant_id);

    // Verify default technical measures were automatically wired
    assert_eq!(ctx.technical_measures.len(), 4);
    assert_eq!(
        ctx.technical_measures[0].name,
        "Kryptografischer Löschbeweis (Ed25519)"
    );

    let markdown = ctx
        .render()
        .expect("Rendering via AvvContext::render succeeds");

    // Assert rendered document contains required sections and default technical measures
    assert!(markdown.contains("# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO"));
    assert!(markdown.contains("**Verantwortlicher (Auftraggeber):** Kanzlei Dr. Weber"));
    assert!(markdown.contains("**Auftragsverarbeiter (Auftragnehmer):** Contextra Cloud Platform"));
    assert!(markdown.contains("TenantId(777)"));
    assert!(markdown.contains("Kryptografischer Löschbeweis (Ed25519)"));
    assert!(markdown.contains("Zero-Egress-Default / Egress-Gateway"));
    assert!(markdown.contains("Kryptografische Mandantentrennung"));
    assert!(markdown.contains("Verschlüsselte Segmente im KV-Cache"));
}

#[test]
fn test_avv_context_builder_production_path_wires_customizations_and_renders() {
    let tenant_id = TenantId(888);
    // Exercise production path via AvvContext::builder
    let markdown = AvvContext::builder(
        "Praxis Dr. Schmidt",
        "Contextra On-Premise",
        tenant_id,
    )
    .add_subprocessor("Hetzner Online GmbH")
    .with_deletion_sla_days(7)
    .add_technical_measure(TechnicalMeasure {
        name: "Spezifisches Backup-Konzept".to_string(),
        description: "Tägliches verschlüsseltes Offsite-Backup.".to_string(),
        reference_article: "Art. 32 Abs. 1 lit. c DSGVO".to_string(),
    })
    .render()
    .expect("Rendering via AvvContextBuilder::render succeeds");

    assert!(markdown.contains("**Verantwortlicher (Auftraggeber):** Praxis Dr. Schmidt"));
    assert!(markdown.contains("**Auftragsverarbeiter (Auftragnehmer):** Contextra On-Premise"));
    assert!(markdown.contains("- Hetzner Online GmbH"));
    assert!(markdown.contains("7 Tagen"));
    assert!(markdown.contains("Spezifisches Backup-Konzept"));
    assert!(markdown.contains("Kryptografischer Löschbeweis (Ed25519)"));
}
