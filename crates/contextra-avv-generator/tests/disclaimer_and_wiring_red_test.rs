use contextra_avv_generator::{render_avv_markdown, AvvContext};
use contextra_types::TenantId;

fn assert_disclaimer_first_and_last_line(markdown: &str, path_label: &str) {
    let lines: Vec<&str> = markdown.trim().lines().map(|l| l.trim()).collect();
    assert!(
        !lines.is_empty(),
        "Rendered markdown for {} must not be empty",
        path_label
    );

    let first = lines.first().unwrap();
    let last = lines.last().unwrap();

    assert!(
        first.contains("Vorlage, keine Rechtsberatung"),
        "First line of {} rendered output must contain 'Vorlage, keine Rechtsberatung', got: '{}'",
        path_label,
        first
    );

    assert!(
        last.contains("Vorlage, keine Rechtsberatung"),
        "Last line of {} rendered output must contain 'Vorlage, keine Rechtsberatung', got: '{}'",
        path_label,
        last
    );
}

#[test]
fn test_avv_generator_render_has_legal_disclaimer() {
    let tenant_id = TenantId::try_new(100).unwrap();
    let ctx = AvvContext::new("Test Controller Kanzlei", "Test Processor GmbH", tenant_id);
    let md = render_avv_markdown(&ctx).unwrap();

    assert_disclaimer_first_and_last_line(&md, "render_avv_markdown");
}

#[test]
fn test_avv_wiring_exists() {
    use contextra_ports::clock::ManualClock;
    use std::sync::Arc;

    let tenant_id = TenantId::try_new(100).unwrap();
    let clock: Arc<dyn contextra_ports::clock::Clock> = Arc::new(ManualClock::new(2_000_000));

    let wiring = contextra_avv_generator::wiring::AvvWiring::build(
        "Test Controller Kanzlei",
        "Test Processor GmbH",
        tenant_id,
        clock,
    )
    .expect("AvvWiring::build failed");

    let md = wiring.render().unwrap();
    assert_disclaimer_first_and_last_line(&md, "AvvWiring::render");
}
