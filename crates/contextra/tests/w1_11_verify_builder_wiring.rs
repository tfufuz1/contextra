// FILE-CONTEXT
// ZWECK: Test verify_builder_wiring security ports and no unverifiable entries (W1-11 Stage 4).

use contextra::builder::ContextraBuilder;
use contextra::wiring_selftest::{verify_builder_wiring, generate_report_for_builder};

#[test]
fn verify_builder_wiring_no_unverifiable_entries() {
    let builder = ContextraBuilder::new(768);
    let report = generate_report_for_builder(&builder);

    for m in &report.missing_ports {
        assert!(
            !m.contains("nicht prüfbar"),
            "WiringReport missing_ports should not contain unverifiable entries, got: {m}"
        );
    }
}

#[test]
fn verify_builder_wiring_rejects_stub_port() {
    let builder = ContextraBuilder::new(768)
        .with_integrity_license(b"payload", &[0u8; 64], &[0u8; 32]);

    let res = verify_builder_wiring(&builder);
    assert!(res.is_err(), "Invalid/stub license gate must be rejected by verify_builder_wiring");
}
