# contextra-audit-export

Export von Verarbeitungsverzeichnissen (DSGVO Art. 30), Audit-Logs und BSI TR-02102 Zuordnungen.

Normativitätsklasse: **Klasse A (Autarker Baustein)** · Reifegrad: `experimental` · Ring: Extension.

## Rolle in einem Satz
`contextra-audit-export` liest neutrale Verarbeitungs- und Audit-Einträge und exportiert sie deterministisch in typsichere JSON-, Markdown- und CSV-Formate sowie BSI-Grundschutz-Zuordnungstabellen.

## Nicht-Ziele (Abgrenzung)
- **Keine Löschkette / Löschbeweise:** Enthält keine Lösch-, Saga- oder Crypto-Shredding-Logik (§6); diese liegt ausschließlich in `contextra-erasure`.
- **Keine Rechtsberatung:** Die Ausgaben stellen rein technische Dokumentationen dar.
- **Keine Netzwerk-/Speicher-Kernkomponenten:** Keine I/O- oder Datenbank-Verbindungen.

## Baustein-Eigenschaften (Klasse A)
- **Keine internen Abhängigkeiten:** Kompiliert ohne interne Workspace-Crates (`--no-default-features`).
- **Feature `types-compat`:** Optionales Kompatibilitätsfeature für die nahtlose Konvertierung zwischen lokalem `TenantId` und `contextra-types::TenantId`.
- **Zero-Panic:** Alle Export- und Validierungspfade liefern `Result<T, AuditExportError>`.
- **Standard-Typen:** Öffentliche API nutzt Standard-Typen (`u64`, `&str`, `Vec`) und bausteineigene Typen.

## Standalone-Nutzung & Export
Dieser Baustein kann autark in andere Projekte kopiert werden:
```bash
python3 tools/export_crate.py contextra-audit-export --out /tmp/contextra-audit-export
```

## Externe Abhängigkeiten
- `serde` (v1.0)
- `serde_json` (v1.0)
- `thiserror` (v2.0)

## Minimalbeispiel
```rust
use contextra_audit_export::{
    render_register_json, render_register_markdown, render_register_csv,
    testkit::InMemoryProcessingRegisterSource, ProcessingRegisterSource, TenantId,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(1001)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id)?;

    // 1. JSON
    let json = render_register_json(&entries)?;

    // 2. Markdown
    let md = render_register_markdown(&entries, "2026-11-08")?;

    // 3. CSV (RFC 4180)
    let csv = render_register_csv(&entries)?;

    println!("Exportiert: {} Einträge", entries.len());
    Ok(())
}
```

## Benchmarks
Criterion-Benchmarks befinden sich in `benches/audit_export_bench.rs` und können ausgeführt werden mit:
```bash
cargo bench -p contextra-audit-export
```
Workloads: JSON-Export, Markdown-Export, RFC-4180 CSV-Export, BSI-Mapping.
