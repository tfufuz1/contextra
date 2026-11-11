# contextra-mvcc

Multi-Version Concurrency Control (MVCC), Sequence Log, Snapshot-Registry und Serializable Snapshot Isolation (SSI).

## Zweck
`contextra-mvcc` bietet deterministische Multi-Version Concurrency Control (`SequenceLog`, `SnapshotRegistry`, `TxBuffer`, `ReadSet`, `SequenceLogSsiValidator`) für serielle Transaktionsisolation ohne Async- oder Tokio-Laufzeitabhängigkeit.

## Reifegrad & Qualität
- **Status:** Stabil (`stable`), autarker Kernbaustein (Reifegrad R3).
- **Sicherheitsstufe:** `#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`, Zero-Panic in Nicht-Test-Code.

## Interne Abhängigkeiten (Kern)
Laut `cargo tree -p contextra-mvcc -e normal --no-default-features`:
- `contextra-ids`: Kanonische ID-Typen wie `TxId`, `DocId`, `TenantId`, `TOMBSTONE_BIT`.
- `contextra-env`: Abstrakte Umgebungs-Schnittstellen (`Clock`, `SystemClock`, `MetricsSink`).
- Optionale Workspace-Brücke (`workspace`, `ports-compat`, `types-compat`): `contextra-types`, `contextra-ports`.
- Externe Abhängigkeiten: `ahash`, `bytes`, `parking_lot`, `serde`, `thiserror`, `tracing`.
Keine Abhängigkeiten zu höheren Schichten (`store`, `vector`, `graph`, `engine`, `db`).

## Features
- `default = ["workspace"]`: Kompatibilitätsbrücke für bestehende Workspace-Konsumenten (`store`, `vector`, `engine`, `checkpoint`).
- `workspace`: Aktiviert `contextra-types` und `contextra-ports` unter `compat/`.
- `types-compat` / `ports-compat`: Abwärtskompatible Aliase für `workspace`.
- `docid-128`: Propagiert 128-Bit DocId-Support an `contextra-ids`.
- Für Standalone-Nutzung: `--no-default-features`.

## Quickstart (Doctest)
```rust
use std::sync::Arc;
use contextra_mvcc::{SnapshotRegistry, TxBuffer, ReadSet, SequenceLogSsiValidator, SsiValidator, IndexOp};
use contextra_mvcc::types::{TxId, DocId};

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
cp -r crates/contextra-ids /path/to/target/
cp -r crates/contextra-env /path/to/target/
```
In `Cargo.toml` des Zielprojekts:
```toml
[dependencies]
contextra-mvcc = { path = "./contextra-mvcc", default-features = false }
```

## Bekannte Grenzen & IST-Verhalten
- **In-Memory-Fokus:** `contextra-mvcc` speichert Transaktionszustände, ReadSets und Staging-Puffer flüchtig im Arbeitsspeicher (RAM). Persistente WAL- und SSTable-Verwaltung wird durch das Zusammenspiel mit dem Storage-Layer (`contextra-store`) sichergestellt.
- **Speicherbegrenzung:** Bei Erreichen von 80 % der maximal getrackten Commit-Schlüssel (`DEFAULT_MAX_TRACKED_COMMIT_KEYS = 1.000.000`) schaltet der SSI-Validator auf konservatives Präfix-Coarsening um, um den Arbeitsspeicher zu begrenzen (0 % False Negatives, konservative Aborts bei hohem Durchsatz).
