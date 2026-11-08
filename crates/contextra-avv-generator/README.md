# contextra-avv-generator

Auftragsverarbeitungsvertrag (AVV) Template Generator.

Normativitätsklasse: **Klasse A (Isolierter Baustein, SPEC v4.0 §2, §5.1)**.

## Rolle & Zweck
`contextra-avv-generator` ist eine leichtgewichtige, autarke Template-Engine zur automatisierten Generierung strukturierter AVV-Dokumentenvorlagen (Art. 28 DSGVO) im Markdown-Format basierend auf System- und Sicherheitskonfigurationen.

## Modulare Architektur & Entkopplung (Klasse A)
- **Vollkommen autark (B1, B2):** Keine internen Abhängigkeiten (`contextra-*`). Der Typ `TenantId` ist ein lokaler 64-Bit-Identifier.
- **Minimale Standard-Abhängigkeiten (B6):** Hängt standardmäßig nur an `thiserror`. `serde` ist optional hinter dem Feature-Flag `serde` verfügbar (`default = ["std"]`).
- **Reine Rendering-Engine:** Keine Kryptografie-, Transaktions-, Beweis- oder physische Löschlogik im Crate (§6).
- **Rechtsprüfungsvorbehalt:** Sämtliche generierten Dokumente enthalten verbindlich den Disclaimer:
  `> **Hinweis:** Entwurf — ersetzt keine Rechtsberatung.`
- **Speichersicherheit & Zero-Panic:** Die Crate erzwingt `#![forbid(unsafe_code)]` und enthält keinerlei `.unwrap()`, `.expect()` oder Panics in Produktion.

## Export & Standalone-Nutzung
Der Baustein kann per `tools/export_crate.py` in beliebige externe Rust-Projekte exportiert werden:
```bash
python3 tools/export_crate.py contextra-avv-generator --out /tmp/contextra-avv-generator
```

## Minimalbeispiel
```rust
use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, AvvContext, TenantId,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AvvContext::new(
        "Kanzlei Dr. Mustermann",
        "Contextra Hosting GmbH",
        TenantId(42),
        default_technical_measures(),
        vec!["EU-Cloud-Region-Frankfurt".to_string()],
        14,
    );

    let markdown = render_avv_markdown(&ctx)?;
    println!("{}", markdown);
    Ok(())
}
```

## Nicht-Ziele
- Keine Rechtsberatung oder rechtsverbindliche Erklärung.
- Keine Ausführung von Löschaufträgen oder Sagas.
- Keine Netzwerk- oder Dateisystem-I/O.
