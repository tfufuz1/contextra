# contextra-types

Kanonische Domänenmodelle, Identifikatoren, Budgets, Filter und Fehlerstrukturen.

Normativitätsklasse: **Klasse B (Storage-Bundle-Kerne)** gemäß SPEC v4.0.

## Rolle & Zweck
`contextra-types` ist das typisierte Fundament des Storage-Bundles im Contextra-Baukasten. Es enthält speichermodell-neutrale Domänentypen wie Identifikatoren (`DocId`, `TxId`, `TenantId`), Token-Budgets, Tags und den zentralen kanonischen Fehlerbaum (`ContextraError`).

Der Baustein ist vollständig autark, I/O-frei, deterministisch und enthält keine Laufzeit- oder Speichermodell-Annahmen (LSM, HNSW, BM25).

## Autarke Entnahme (B5 / SPEC §7)
```bash
python3 tools/export_crate.py contextra-types --out /tmp/export-contextra-types
```

## Baukasten-Eigenschaften (SPEC v4.0 Klasse B, B3–B12)
- **Klasse:** Klasse B (Fundament des Storage-Bundles)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten:** Keine internen Workspace-Abhängigkeiten (`may_depend_on = []`).
  - Externe Dependencies: `thiserror`, `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes` (dev: `proptest`, `criterion`).
- **Features:** Minimal per Default (`default = []`), additiv:
  - `docid-128`: Schaltet `DocId` von 64-Bit (`u64`) auf 128-Bit (`u128`) um.
  - `auto-extraction-opt-out`: Kompatibilitätsfeature für Metadaten-Extraktion.
- **Determinismus:** Keine Systemzeit (`SystemTime::now`), kein Thread-Zufall (`thread_rng`), keine I/O-Operationen.
- **Zero-Panic:** Panics und unvollständige Fehlerbehandlung in Konstruktoren und Konvertierungen sind eliminiert.
- **Löschentkopplung (§6 / D-E1):** Löschdomänen-Typen (`erasure.rs`, `witness.rs`) sind als deprecated markiert (Migration nach `contextra-erasure`).

## Nicht-Ziele
- Keine Implementierung von Speicher-Engines, Indizes oder Löschprotokollen.
- Kein I/O, keine Netzwerkaufrufe, keine Abhängigkeit von Async-Runtimes.

## Minimalbeispiel
```rust
use contextra_types::{DocId, TenantId, TxId};

let doc = DocId::from_u64(42);
let tenant = TenantId::system();
let tx = TxId::from_u64(1);

assert_eq!(doc.as_u64(), 42);
assert!(tenant.is_system());
assert_eq!(tx.as_u64(), 1);
```
