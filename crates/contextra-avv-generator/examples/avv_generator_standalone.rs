//! Standalone-Beispiel für `contextra-avv-generator` (M10).
//! Demonstriert die Erzeugung eines strukturierten AVV-Vertragsentwurfs gemäß Art. 28 DSGVO.

use contextra_avv_generator::{default_technical_measures, render_avv_markdown, AvvContext};
use contextra_types::TenantId;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AvvContext {
        controller_name: "Beispiel-Mandant Kanzlei Schmidt".to_string(),
        processor_name: "Contextra Embedded Systems".to_string(),
        tenant_id: TenantId(101),
        technical_measures: default_technical_measures(),
        subprocessors: vec!["Lokaler In-Memory Provider".to_string()],
        deletion_sla_days: 14,
        contract_date: Some("2026-11-07".to_string()),
        subprocessors_allowed: Some(true),
    };

    let markdown = render_avv_markdown(&ctx)?;
    println!("AVV erfolgreich generiert ({} Zeichen):\n", markdown.len());
    println!("{}", &markdown[..markdown.len().min(400)]);

    Ok(())
}
