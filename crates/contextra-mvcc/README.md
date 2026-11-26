# contextra-mvcc

Multi-Version Concurrency Control (MVCC) und Serializable Snapshot Isolation (SSI).

## Zweck
`contextra-mvcc` bietet deterministische Multi-Version Concurrency Control (SequenceLog, SnapshotRegistry, TxBuffer, ReadSet, SSI-Validator) für serielle Transaktionsisolation ohne Tokio-Laufzeitabhängigkeit.

## Reifegrad
- **Status:** Stabil (`stable`), autarker Kernbaustein.
- **Sicherheitsstufe:** `#![forbid(unsafe_code)]`, Zero-Panic in Nicht-Test-Code.

## Interne Abhängigkeiten
`contextra-mvcc` hängt ausschließlich von minimalen Basis-Crates ab:
- `contextra-types`: Domänentypen wie `TxId`, `DocId`, `TenantId`, `ContextraError`.
- `contextra-ports`: Zeitabstraktion (`Clock`).
- Externe Abhängigkeiten: `ahash`, `bytes`, `parking_lot`, `serde`, `thiserror`, `tracing`.
Keine Abhängigkeiten zu höheren Schichten (`store`, `vector`, `graph`, `engine`, `db`).

## Features
- `default = []`: Vollständig synchron nutzbar ohne Runtimes.
- Keine versteckten I/O- oder Async-Feature-Flags.

## Doctest
```rust
use std::sync::Arc;
use contextra_mvcc::{SnapshotRegistry, TxBuffer, ReadSet, SequenceLogSsiValidator, SsiValidator, IndexOp};
use contextra_types::types::{TxId, DocId};

let registry = Arc::new(SnapshotRegistry::new());
let ssi = SequenceLogSsiValidator::new();
let buffer = TxBuffer::<String>::new();

let tx = TxId::new(1);
let guard = registry.register(100);

let mut read_set = ReadSet::new();
read_set.record_read(b"account:42", 100);

buffer.begin(tx);
buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(1u64), data: "value".into() }).unwrap();

// Validierung vor Commit gegen Anti-Dependencies
assert!(ssi.validate(tx, &read_set).is_ok());
drop(guard);
```

## Entnahme-Rezept
Zur isolierten Nutzung in externen Projekten:
```bash
# Kopieren des Crates zusammen mit seinen Basis-Abhängigkeiten
cp -r crates/contextra-mvcc /path/to/target/
cp -r crates/contextra-types /path/to/target/
cp -r crates/contextra-ports /path/to/target/
```
In `Cargo.toml` des Zielprojekts:
```toml
[dependencies]
contextra-mvcc = { path = "./contextra-mvcc" }
```

## Bekannte Grenzen
- **In-Memory-Fokus:** `contextra-mvcc` speichert Transaktionszustände, ReadSets und Staging-Puffer flüchtig im Arbeitsspeicher (RAM). Persistente WAL- und SSTable-Verwaltung wird durch das Zusammenspiel mit dem Storage-Layer (`contextra-store`) sichergestellt.
- **Speicherbegrenzung:** Bei Erreichen von 80 % der maximal getrackten Commit-Schlüssel (`DEFAULT_MAX_TRACKED_COMMIT_KEYS = 1.000.000`) schaltet der SSI-Validator auf konservatives Präfix-Coarsening um, um den Arbeitsspeicher zu begrenzen (0 % False Negatives, erhöhte Abort-Rate bei hohem Durchsatz).
