# contextra-types

Kanonische Domänenmodelle, Identifikatoren, Budgets, Filter und Fehlerstrukturen.

## Rolle & Zweck
`contextra-types` ist das typisierte Fundament des Contextra-Baukastens (Gruppe `contracts`, Ebene 2). Es enthält speichermodell-neutrale Domänentypen wie Identifikatoren (`DocId`, `TxId`, `TenantId`), Token-Budgets, Tags und den zentralen kanonischen Fehlerbaum (`ContextraError`).

Der Baustein ist vollständig autark von I/O, deterministisch und enthält keine Annahmen über konkrete Speichermodelle (wie LSM, HNSW oder BM25).

## Autarke Entnahme
```bash
cargo build -p contextra-types --no-default-features --locked
```

## Baukasten-Eigenschaften
- **Gruppe / Ebene:** `contracts` (Ebene 2)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten:** Ausschließlich `contextra-ids` (`allowed_internal = ["contextra-ids"]`).
  - Externe Dependencies: `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` (dev: `proptest`, `criterion`).
- **Features:** Minimal per Default (`default = []`), additiv:
  - `docid-128`: Schaltet `DocId` von 64-Bit (`u64`) auf 128-Bit (`u128`) um (weitergeleitet an `contextra-ids`).
  - `auto-extraction-opt-out`: Kompatibilitätsfeature für Metadaten-Extraktion.
- **Determinismus:** Keine Systemzeit (`SystemTime::now`), kein Thread-Zufall (`thread_rng`), keine I/O-Operationen (INV-DETERMINISM-1).
- **Zero-Panic:** Panics und unvollständige Fehlerbehandlung in Konstruktoren und Konvertierungen sind eliminiert.
- **Löschentkopplung:** Löschdomänen-Typen (`erasure.rs`, `witness.rs`) sind als deprecated markiert (Migration nach `contextra-erasure`).

## Nicht-Ziele
- Keine Implementierung von Speicher-Engines, Indizes oder Löschprotokollen.
- Kein I/O, keine Netzwerkaufrufe, keine Abhängigkeit von Async-Runtimes.

## Minimalbeispiel
```rust
use contextra_types::{DocId, TenantId, TxId};

let doc = DocId::from(42u64);
let tenant = TenantId::system();
let tx = TxId::new(1);

assert_eq!(doc.as_u64(), 42);
assert!(tenant.is_system());
assert_eq!(tx.as_u64(), 1);
```
