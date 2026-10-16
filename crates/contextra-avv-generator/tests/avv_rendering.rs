use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, AvvContext, TechnicalMeasure,
};
use contextra_types::TenantId;

#[test]
fn test_render_avv_markdown_contains_required_sections() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(42)?;
    let ctx = AvvContext {
        controller_name: "Kanzlei Dr. Mustermann".to_string(),
        processor_name: "Contextra Cloud GmbH".to_string(),
        tenant_id,
        technical_measures: default_technical_measures(),
        subprocessors: vec!["Hetzner Online GmbH (Hosting Germany)".to_string()],
        deletion_sla_days: 14,
    };

    let markdown = render_avv_markdown(&ctx)?;

    // Verify required Art. 28 DSGVO headings
    assert!(markdown.contains("# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO"));
    assert!(markdown.contains("## 1. Gegenstand und Dauer der Verarbeitung"));
    assert!(markdown.contains("## 2. Art und Zweck der Verarbeitung"));
    assert!(markdown
        .contains("## 3. Art der personenbezogenen Daten und Kategorien betroffener Personen"));
    assert!(markdown.contains("## 4. Pflichten und Rechte des Verantwortlichen"));
    assert!(markdown.contains("## 5. Technische und organisatorische Maßnahmen (TOM)"));
    assert!(markdown.contains("## 6. Unterauftragsverarbeiter"));
    assert!(markdown.contains("## 7. Löschung von Daten und SLA"));

    // Verify technical measures are present
    assert!(markdown.contains("Kryptografischer Löschbeweis (Ed25519)"));
    assert!(markdown.contains("Zero-Egress-Default / Egress-Gateway"));
    assert!(markdown.contains("Kryptografische Mandantentrennung"));
    assert!(markdown.contains("Verschlüsselte Segmente im KV-Cache"));

    // Verify personalized data
    assert!(markdown.contains("Kanzlei Dr. Mustermann"));
    assert!(markdown.contains("Contextra Cloud GmbH"));
    assert!(markdown.contains("TenantId(42)"));
    assert!(markdown.contains("Hetzner Online GmbH (Hosting Germany)"));
    assert!(markdown.contains("14 Tagen"));
    Ok(())
}

#[test]
fn test_empty_subprocessors_handling() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(100)?;
    let ctx = AvvContext {
        controller_name: "Praxis Musterfrau".to_string(),
        processor_name: "Contextra On-Premise".to_string(),
        tenant_id,
        technical_measures: default_technical_measures(),
        subprocessors: vec![],
        deletion_sla_days: 7,
    };

    let markdown = render_avv_markdown(&ctx)?;

    assert!(markdown.contains("## 6. Unterauftragsverarbeiter"));
    assert!(markdown.contains("Keine Unterauftragsverarbeiter"));
    assert!(markdown.contains("Die Verarbeitung erfolgt ausschließlich auf eigenen Systemen des Auftragsverarbeiters ohne Einbindung Dritter."));
    Ok(())
}

#[test]
fn test_snapshot_rendering_output() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(999)?;
    let ctx = AvvContext {
        controller_name: "Test AG".to_string(),
        processor_name: "Contextra GmbH".to_string(),
        tenant_id,
        technical_measures: vec![TechnicalMeasure {
            name: "Custom Security Measure".to_string(),
            description: "Description of custom measure.".to_string(),
            reference_article: "Art. 32 Abs. 1 DSGVO".to_string(),
        }],
        subprocessors: vec!["Cloud Provider X".to_string()],
        deletion_sla_days: 30,
    };

    let markdown = render_avv_markdown(&ctx)?;

    let expected_substrings = [
        "# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO",
        "**Verantwortlicher (Auftraggeber):** Test AG",
        "**Auftragsverarbeiter (Auftragnehmer):** Contextra GmbH",
        "**Mandanten-ID (TenantId):** TenantId(999)",
        "### Custom Security Measure (Art. 32 Abs. 1 DSGVO)",
        "Description of custom measure.",
        "- Cloud Provider X",
        "innerhalb des vereinbarten Deletion-SLA von **30 Tagen**.",
    ];

    for expected in &expected_substrings {
        assert!(
            markdown.contains(expected),
            "Expected output to contain substring: '{}'",
            expected
        );
    }
    Ok(())
}
