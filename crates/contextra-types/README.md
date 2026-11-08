# contextra-types

Kanonische Domänenmodelle, Identifikatoren, Budgets, Filter und Fehlerstrukturen für Contextra.

Normativitätsklasse: **Klasse B (Storage-Bundle)** (Ring 0; SPEC v4.0 §2, §5.2).

## Rolle & Zweck
`contextra-types` ist die Basis aller Typdefinitionen im gesamten Contextra-Ökosystem. Es enthält produkt- und I/O-unabhängige Domänentypen wie Identifikatoren (`DocId`, `TxId`, `TenantId`), Token-Budgets, Tags und den zentralen kanonischen Fehlerbaum (`ContextraError`).

Der Crate ist vollständig autark, I/O-frei, deterministisch und enthält keine externen Laufzeit-Abhängigkeiten wie `tokio`.

## Baukasten-Einordnung & Garantien (B1–B12)
- **Klasse:** B (Storage-Bundle)
- **Ring:** 0 (Typen-Kern, Fundament)
- **Status:** 🟢 Vollständig autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Zero-Panic (B9):** 0 unhandled Panics oder unwrap im Produktionscode.
- **Abhängigkeiten (B1, B2):** Keine internen Workspace-Abhängigkeiten (0 Workspace-Deps).
  - Externe Crates: `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` (dev: `proptest`).
- **Features (B6):** Minimal per Default (`default = []`), additiv:
  - `docid-128`: Schaltet `DocId` von 64-Bit (`u64`) auf 128-Bit (`u128`) um (Workspace-weit gespiegelt).
  - `auto-extraction-opt-out`: Kompatibilitätsfeature für Metadaten-Extraktion.
- **Determinismus (B11):** Keine Systemzeit (`SystemTime::now`), kein Thread-Zufall (`thread_rng`), keine I/O-Operationen.
- **Löschentkopplung (§6, Welle 3):** Löschtypen in `erasure.rs` und `witness.rs` sind als `#[deprecated(note = "moved to contextra-erasure")]` markiert.

## Export & Standalone-Nutzung
Der Baustein kann über das Export-Werkzeug isoliert exportiert werden:

```bash
python3 tools/export_crate.py contextra-types --out /tmp/export-contextra-types
```

## Einbettungsbeispiel
Ein vollständiges, lauffähiges Beispiel findet sich unter `examples/standalone.rs`:

```bash
cargo run -p contextra-types --example standalone
```

```rust
use contextra_types::{DocId, RequestContext, TenantId, TxId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tenant = TenantId::try_new(1)?;
    let tx = TxId::new(100);
    let ctx = RequestContext::new(tenant, tx);
    let doc_id = DocId::from_key("document_123")?;
    println!("Context: Tenant={}, Tx={}, DocId={}", ctx.tenant_id, ctx.tx_id, doc_id);
    Ok(())
}
```
