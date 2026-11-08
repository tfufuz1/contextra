//! Standalone-Beispiel für `contextra-audit-export` (M10).
//! Demonstriert den Export eines Verzeichnisses von Verarbeitungstätigkeiten (Art. 30 DSGVO)
//! sowie das kryptografische BSI-Mapping ohne externe Systemabhängigkeiten.

use contextra_audit_export::{
    bsi_mapping_table, render_bsi_mapping_markdown, render_register_json, render_register_markdown,
    testkit::InMemoryProcessingRegisterSource, ProcessingRegisterSource, TenantId,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(1001)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id)?;

    println!("Gefundene Verarbeitungstätigkeiten: {}", entries.len());

    // 1. JSON-Export
    let json = render_register_json(&entries)?;
    println!("--- JSON Export (Länge: {} Bytes) ---", json.len());

    // 2. Markdown-Export
    let md = render_register_markdown(&entries, "2026-11-07")?;
    println!(
        "--- Markdown Export (Auszug) ---\n{}",
        &md[..md.len().min(300)]
    );

    // 3. BSI Mapping Table
    let bsi = bsi_mapping_table();
    let bsi_md = render_bsi_mapping_markdown(&bsi);
    println!(
        "--- BSI TR-02102 Mapping (Auszug) ---\n{}",
        &bsi_md[..bsi_md.len().min(250)]
    );

    Ok(())
}
