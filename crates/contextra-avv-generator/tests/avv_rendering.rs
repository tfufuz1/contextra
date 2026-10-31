//! Integrationstests für das Rendering und die Validierung von AVV-Dokumenten.
//!
//! # Zweck & Testabdeckung
//! Diese Test-Suite verifiziert das funktionale Verhalten von `contextra-avv-generator`:
//! - Vollständigkeit aller geforderten Abschnitte gemäß Art. 28 DSGVO (Abschnitte 1 bis 11).
//! - Abgleich der gerenderten Ausgabe gegen eine gespeicherte Referenzdatei (Golden File).
//! - Validierungsregeln für Eingabedaten ([`AvvContext::validate`]), einschließlich Randwerten für Löschfristen.
//! - Sicherheit gegen Markdown-Injections (Maskierung von Sonderzeichen und Neutralisierung von Zeilenumbrüchen).
//! - Deterministische Ausgabe bei festem Vertragsdatum sowie fehlerfreie Unicode-Unterstützung.

use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, render_avv_markdown_with_date, AvvContext,
    AvvError, TechnicalMeasure,
};
use contextra_types::TenantId;

// ANCHOR[contextra-avv-generator:TEST-CTX-001]
fn sample_context() -> AvvContext {
    AvvContext {
        controller_name: "Kanzlei Dr. Mustermann".to_string(),
        processor_name: "Contextra Cloud GmbH".to_string(),
        tenant_id: TenantId(42),
        technical_measures: default_technical_measures(),
        subprocessors: vec!["Hetzner Online GmbH (Hosting Germany)".to_string()],
        deletion_sla_days: 14,
        contract_date: None,
        subprocessors_allowed: None,
    }
}

// ANCHOR[contextra-avv-generator:TEST-001]
#[test]
fn test_render_avv_markdown_contains_required_sections() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let markdown = render_avv_markdown(&ctx)?;

    // Verify required Art. 28 DSGVO headings
    assert!(markdown.contains("# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO"));
    assert!(markdown.contains("## 1. Gegenstand und Dauer der Verarbeitung"));
    assert!(markdown.contains("## 2. Art und Zweck der Verarbeitung"));
    assert!(markdown
        .contains("## 3. Art der personenbezogenen Daten und Kategorien betroffener Personen"));
    assert!(markdown.contains("## 4. Pflichten und Rechte des Verantwortlichen"));
    assert!(markdown.contains("## 6. Technische und organisatorische Maßnahmen (TOM)"));
    assert!(markdown.contains("## 7. Unterauftragsverarbeiter"));
    assert!(markdown.contains("## 8. Löschung von Daten und SLA"));

    // Verify technical measures are present
    assert!(markdown.contains("Kryptografischer Löschbeweis"));
    assert!(markdown.contains("Zero"));
    assert!(markdown.contains("Kryptografische Mandantentrennung"));
    assert!(markdown.contains("Verschlüsselte Segmente im KV"));

    // Verify parties and SLA are present
    assert!(markdown.contains("Kanzlei Dr. Mustermann"));
    assert!(markdown.contains("Contextra Cloud GmbH"));
    assert!(markdown.contains("Hetzner Online GmbH"));
    assert!(markdown.contains("14 Tagen"));

    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-002]
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
        contract_date: None,
        subprocessors_allowed: None,
    };

    let markdown = render_avv_markdown(&ctx)?;

    assert!(markdown.contains("## 7. Unterauftragsverarbeiter"));
    assert!(markdown.contains("Keine Unterauftragsverarbeiter"));
    assert!(markdown.contains("Die Verarbeitung erfolgt ausschließlich auf eigenen Systemen des Auftragsverarbeiters ohne Einbindung Dritter."));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-003]
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
        contract_date: None,
        subprocessors_allowed: None,
    };

    let markdown = render_avv_markdown(&ctx)?;

    let expected_substrings = [
        "# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO",
        "**Verantwortlicher (Auftraggeber):** Test AG",
        "**Auftragsverarbeiter (Auftragnehmer):** Contextra GmbH",
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

// ANCHOR[contextra-avv-generator:TEST-004]
#[test]
fn test_art28_dsgvo_completeness_checklist() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md = render_avv_markdown(&ctx)?;

    // Check every mandatory element under Art. 28 Abs. 3 lit. a–h DSGVO:
    assert!(md.contains("Gegenstand und Dauer"), "Art. 28 Abs. 3 lit. a");
    assert!(md.contains("Art und Zweck"), "Art. 28 Abs. 3 lit. b");
    assert!(
        md.contains("Art der personenbezogenen Daten"),
        "Art. 28 Abs. 3 lit. c"
    );
    assert!(
        md.contains("Kategorien betroffener Personen"),
        "Art. 28 Abs. 3 lit. c"
    );
    assert!(
        md.contains("ausschließlich auf dokumentierte Weisung"),
        "Art. 28 Abs. 3 lit. a (Weisung)"
    );
    assert!(
        md.contains("Vertraulichkeitsverpflichtung"),
        "Art. 28 Abs. 3 lit. b (Vertraulichkeit)"
    );
    assert!(
        md.contains("Technische und organisatorische Maßnahmen"),
        "Art. 28 Abs. 3 lit. c (TOM)"
    );
    assert!(
        md.contains("Unterauftragsverarbeiter"),
        "Art. 28 Abs. 3 lit. d (Subprozessoren)"
    );
    assert!(
        md.contains("beabsichtigte Änderung bezüglich der Hinzuziehung oder Ersetzung"),
        "Art. 28 Abs. 2 (Informationspflicht)"
    );
    assert!(
        md.contains("Unterstützung bei Betroffenenrechten"),
        "Art. 28 Abs. 3 lit. e"
    );
    assert!(
        md.contains("Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO"),
        "Art. 28 Abs. 3 lit. f"
    );
    assert!(
        md.contains("Löschung von Daten und SLA"),
        "Art. 28 Abs. 3 lit. g"
    );
    assert!(
        md.contains("Nachweis- und Kontrollrechte"),
        "Art. 28 Abs. 3 lit. h"
    );

    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-005]
#[test]
fn test_disclaimer_present_in_header() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md = render_avv_markdown(&ctx)?;
    assert!(md.contains("> **Hinweis:** Entwurf — ersetzt keine Rechtsberatung."));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-006]
#[test]
fn test_golden_file_output_matches() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let rendered = render_avv_markdown_with_date(&ctx, "2026-10-29")?;

    let golden_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/avv_default.md");
    let golden_content = std::fs::read_to_string(&golden_path)?;

    assert_eq!(rendered.trim(), golden_content.trim());
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-007]
#[test]
fn test_validation_empty_controller_name() {
    let mut ctx = sample_context();
    ctx.controller_name = "   ".to_string();
    assert_eq!(ctx.validate(), Err(AvvError::EmptyControllerName));
}

// ANCHOR[contextra-avv-generator:TEST-008]
#[test]
fn test_validation_empty_processor_name() {
    let mut ctx = sample_context();
    ctx.processor_name = "".to_string();
    assert_eq!(ctx.validate(), Err(AvvError::EmptyProcessorName));
}

// ANCHOR[contextra-avv-generator:TEST-009]
#[test]
fn test_validation_empty_contract_date() {
    let mut ctx = sample_context();
    ctx.contract_date = Some("   ".to_string());
    assert_eq!(ctx.validate(), Err(AvvError::EmptyContractDate));
}

// ANCHOR[contextra-avv-generator:TEST-010]
#[test]
fn test_validation_invalid_deletion_sla_zero() {
    let mut ctx = sample_context();
    ctx.deletion_sla_days = 0;
    assert_eq!(
        ctx.validate(),
        Err(AvvError::InvalidDeletionSla {
            days: 0,
            reason: "Löschfrist muss mindestens 1 Tag betragen.",
        })
    );
}

// ANCHOR[contextra-avv-generator:TEST-011]
#[test]
fn test_validation_invalid_deletion_sla_too_high() {
    let mut ctx = sample_context();
    ctx.deletion_sla_days = 366;
    assert_eq!(
        ctx.validate(),
        Err(AvvError::InvalidDeletionSla {
            days: 366,
            reason: "Löschfrist darf maximal 365 Tage betragen.",
        })
    );
}

// ANCHOR[contextra-avv-generator:TEST-012]
#[test]
fn test_validation_missing_subprocessors_when_allowed() {
    let mut ctx = sample_context();
    ctx.subprocessors_allowed = Some(true);
    ctx.subprocessors = vec![];
    assert_eq!(ctx.validate(), Err(AvvError::MissingSubprocessors));
}

// ANCHOR[contextra-avv-generator:TEST-013]
#[test]
fn test_validation_invalid_subprocessor_empty_string() {
    let mut ctx = sample_context();
    ctx.subprocessors = vec!["  ".to_string()];
    assert_eq!(
        ctx.validate(),
        Err(AvvError::InvalidSubprocessor(
            "Name des Unterauftragsverarbeiters darf nicht leer sein.".to_string()
        ))
    );
}

// ANCHOR[contextra-avv-generator:TEST-014]
#[test]
fn test_validation_invalid_technical_measure_empty_name() {
    let mut ctx = sample_context();
    ctx.technical_measures = vec![TechnicalMeasure {
        name: "".to_string(),
        description: "Valid description".to_string(),
        reference_article: "Art. 32".to_string(),
    }];
    assert_eq!(
        ctx.validate(),
        Err(AvvError::InvalidTechnicalMeasure(
            "Name der technischen Maßnahme darf nicht leer sein.".to_string()
        ))
    );
}

// ANCHOR[contextra-avv-generator:TEST-015]
#[test]
fn test_validation_valid_context() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    assert_eq!(ctx.validate(), Ok(()));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-016]
#[test]
fn test_markdown_escaping_user_inputs() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = sample_context();
    ctx.controller_name = "Company *Bold* & _Italic_ <Inc>".to_string();
    ctx.processor_name = "Cloud`Code`#Heading".to_string();

    let md = render_avv_markdown(&ctx)?;

    assert!(md.contains(r"Company \*Bold\* & \_Italic\_ \<Inc\>"));
    assert!(md.contains(r"Cloud\`Code\`\#Heading"));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-017]
#[test]
fn test_markdown_escaping_injection_prevention() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = sample_context();
    ctx.controller_name = "Hacker\n## Fake Heading\n* Malicious".to_string();

    let md = render_avv_markdown(&ctx)?;

    // Verify newline was converted to space so fake header is neutralized
    assert!(!md.contains("\n## Fake Heading"));
    assert!(md.contains(r"Hacker \#\# Fake Heading \* Malicious"));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-018]
#[test]
fn test_deterministic_rendering_with_fixed_date() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md1 = render_avv_markdown_with_date(&ctx, "2026-10-29")?;
    let md2 = render_avv_markdown_with_date(&ctx, "2026-10-29")?;

    assert_eq!(md1, md2);
    assert!(md1.contains("**Vertragsdatum:** 2026\\-10\\-29"));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-019]
#[test]
fn test_unicode_support_in_names_and_measures() -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = sample_context();
    ctx.controller_name = "Müller & Söhne ⚖️ Legal AI (東京)".to_string();
    ctx.processor_name = "Münchner Datenverarbeitung GmbH 🛡️".to_string();

    let md = render_avv_markdown(&ctx)?;

    assert!(md.contains(r"Müller & Söhne ⚖️ Legal AI \(東京\)"));
    assert!(md.contains("Münchner Datenverarbeitung GmbH 🛡️"));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-020]
#[test]
fn test_subprocessor_change_notification_clause() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md = render_avv_markdown(&ctx)?;

    assert!(md.contains("Der Auftragsverarbeiter informiert den Verantwortlichen immer über jede beabsichtigte Änderung bezüglich der Hinzuziehung oder Ersetzung anderer Unterauftragsverarbeiter."));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-021]
#[test]
fn test_audit_and_control_rights_clause() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md = render_avv_markdown(&ctx)?;

    assert!(md.contains("## 11. Nachweis- und Kontrollrechte"));
    assert!(md.contains(
        "Der Auftragsverarbeiter ermöglicht Überprüfungen — einschließlich Inspektionen —"
    ));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-022]
#[test]
fn test_data_subject_rights_and_assistance_clauses() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md = render_avv_markdown(&ctx)?;

    assert!(md.contains("## 9. Unterstützung bei Betroffenenrechten"));
    assert!(md.contains("## 10. Unterstützung bei Einhaltung der Pflichten gemäß Art. 32–36 DSGVO"));
    assert!(md.contains("Datenschutz-Folgenabschätzungen"));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-023]
#[test]
fn test_confidentiality_clause_present() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md = render_avv_markdown(&ctx)?;

    assert!(md.contains("## 5. Vertraulichkeitsverpflichtung"));
    assert!(md.contains("zur Vertraulichkeit verpflichtet haben"));
    Ok(())
}

// ANCHOR[contextra-avv-generator:TEST-024]
#[test]
fn test_instructions_clause_present() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = sample_context();
    let md = render_avv_markdown(&ctx)?;

    assert!(md.contains("ausschließlich auf dokumentierte Weisung des Verantwortlichen"));
    Ok(())
}
