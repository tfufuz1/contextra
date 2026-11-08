# contextra-types

Kanonische Domänenmodelle, Identifikatoren, Budgets, Filter und Fehlerstrukturen.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, `capabilities.toml`).

## Rolle & Zweck
`contextra-types` ist die Basis aller Typdefinitionen im gesamten Contextra-Ökosystem. Es enthält produkt- und I/O-unabhängige Domänentypen wie Identifikatoren (`DocId`, `TxId`, `TenantId`), Token-Budgets, Tags und den zentralen kanonischen Fehlerbaum (`ContextraError`).

Der Crate ist vollständig autark, I/O-frei, deterministisch und enthält keine externen Laufzeit-Abhängigkeiten wie `tokio`.

## Baukasten-Einordnung & Garantien (M1-M11)
- **Ring:** Ring 0 (Typen-Kern, Fundament)
- **Status:** 🟢 Vollständig autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten:** Keine internen Workspace-Abhängigkeiten (0 Workspace-Deps).
  - Externe Crates: `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` (dev: `proptest`).
- **Features:** Minimal per Default (`default = []`), additiv:
  - `docid-128`: Schaltet `DocId` von 64-Bit (`u64`) auf 128-Bit (`u128`) um (Workspace-weit gespiegelt).
  - `auto-extraction-opt-out`: Kompatibilitätsfeature für Metadaten-Extraktion.
- **Determinismus:** Keine Systemzeit (`SystemTime::now`), kein Thread-Zufall (`thread_rng`), keine I/O-Operationen.
- **Löschentkopplung (D-E1):** Löschtypen sind isoliert und als deprecated markiert (Übergang zu `contextra-erasure-spi`). Es verbleiben nur neutrale Löschmarker (`SeqBitTombstone`, `TOMBSTONE_BIT`).

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
