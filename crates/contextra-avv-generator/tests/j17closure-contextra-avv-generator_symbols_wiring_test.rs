#![forbid(unsafe_code)]

use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, AvvContext, AvvGeneratorError,
    TechnicalMeasure,
};
use contextra_types::TenantId;

#[test]
fn test_default_technical_measures_standalone_and_associated_wire() {
    let direct_measures = default_technical_measures();
    let assoc_measures = AvvContext::default_technical_measures();

    assert_eq!(direct_measures, assoc_measures);
    assert_eq!(direct_measures.len(), 4);

    let measure_names: Vec<&str> = direct_measures.iter().map(|m| m.name.as_str()).collect();
    assert!(measure_names.contains(&"Kryptografischer Löschbeweis (Ed25519)"));
    assert!(measure_names.contains(&"Zero-Egress-Default / Egress-Gateway"));
    assert!(measure_names.contains(&"Kryptografische Mandantentrennung"));
    assert!(measure_names.contains(&"Verschlüsselte Segmente im KV-Cache"));
}

#[test]
fn test_builder_with_default_technical_measures_rewires_toms() {
    let tenant = TenantId(999);
    let builder = AvvContext::builder("Custom Controller", "Custom Processor", tenant)
        .with_technical_measures(vec![TechnicalMeasure {
            name: "Custom TOM".to_string(),
            description: "Custom description".to_string(),
            reference_article: "Art. 32 DSGVO".to_string(),
        }])
        .with_default_technical_measures();

    let ctx = builder.build();
    assert_eq!(ctx.technical_measures.len(), 4);
    assert_eq!(
        ctx.technical_measures[0].name,
        "Kryptografischer Löschbeweis (Ed25519)"
    );
}

#[test]
fn test_render_avv_markdown_standalone_and_error_paths() -> Result<(), AvvGeneratorError> {
    let tenant = TenantId(12345);
    let valid_ctx = AvvContext::new("Musterkanzlei", "Contextra Systems", tenant);

    let markdown = render_avv_markdown(&valid_ctx)?;

    assert!(markdown.contains("# Vereinbarung zur Auftragsverarbeitung (AVV) gemäß Art. 28 DSGVO"));
    assert!(markdown.contains("**Verantwortlicher (Auftraggeber):** Musterkanzlei"));
    assert!(markdown.contains("**Auftragsverarbeiter (Auftragnehmer):** Contextra Systems"));

    let invalid_ctx = AvvContext {
        controller_name: "   ".to_string(),
        processor_name: "Contextra Systems".to_string(),
        tenant_id: tenant,
        technical_measures: vec![],
        subprocessors: vec![],
        deletion_sla_days: 14,
    };

    let err_result = render_avv_markdown(&invalid_ctx);
    assert!(matches!(
        err_result,
        Err(AvvGeneratorError::InvalidContext(_))
    ));

    if let Err(AvvGeneratorError::InvalidContext(msg)) = err_result {
        assert!(msg.contains("Verantwortlicher (controller_name) darf nicht leer sein."));
    }

    Ok(())
}
