use contextra_avv_generator::{wiring::AvvWiring, TechnicalMeasure};
use contextra_ports::clock::ManualClock;
use contextra_types::TenantId;
use std::sync::Arc;

#[test]
fn test_avv_wiring_end_to_end() {
    let tenant_id = TenantId::try_new(300).unwrap();
    let clock = Arc::new(ManualClock::new(5_000_000));

    let wiring = AvvWiring::build(
        "Kanzlei Dr. Schmidt",
        "Contextra SaaS Corp",
        tenant_id,
        clock,
    )
    .expect("AvvWiring build");

    assert_eq!(wiring.context().controller_name, "Kanzlei Dr. Schmidt");
    assert_eq!(wiring.context().processor_name, "Contextra SaaS Corp");
    assert_eq!(wiring.context().tenant_id, tenant_id);
    assert!(!wiring.context().technical_measures.is_empty());

    let wiring_custom = wiring
        .add_subprocessor("AWS Europe (Frankfurt)")
        .with_technical_measures(vec![TechnicalMeasure {
            name: "Mandantengrundschutz".to_string(),
            description: "Isolierter Vektorspeicher".to_string(),
            reference_article: "Art. 32 DSGVO".to_string(),
        }]);

    let md = wiring_custom.render().expect("render AVV");
    assert!(md.contains("Kanzlei Dr. Schmidt"));
    assert!(md.contains("Contextra SaaS Corp"));
    assert!(md.contains("AWS Europe (Frankfurt)"));
    assert!(md.contains("Mandantengrundschutz"));

    let lines: Vec<&str> = md.trim().lines().map(|l| l.trim()).collect();
    assert!(lines
        .first()
        .unwrap()
        .contains("Vorlage, keine Rechtsberatung"));
    assert!(lines
        .last()
        .unwrap()
        .contains("Vorlage, keine Rechtsberatung"));
}
