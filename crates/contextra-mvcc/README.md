# contextra-mvcc

Multi-Version Concurrency Control (MVCC) und Serializable Snapshot Isolation (SSI).

Normativitätsklasse: **Klasse B (Storage-Bundle)** (Ring 0, `stable`; SPEC §5.2, `capabilities.toml`).

## Rolle in einem Satz
`contextra-mvcc` bietet deterministische Multi-Version Concurrency Control (SequenceLog, SnapshotRegistry, TxBuffer, ReadSet, SSI-Validator) für serielle Transaktionsisolation ohne Tokio-Laufzeitabhängigkeit.

## Baukasten-Export (Copy-Paste)
```bash
python3 tools/export_crate.py contextra-mvcc --out /tmp/export-contextra-mvcc
```

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Transaktionsisolation)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)

## Garantien & Invarianten
- **Baukasten-Reinheit (M1, M8):** Keine asynchronen Abhängigkeiten (kein Tokio in `[dependencies]` oder `[dev-dependencies]`).
- **Sperrenordnung (INV-LOCK-ORDER-1):** Totale, dokumentierte Sperrenhierarchie:
  `Document -> TxBuffer (Shards 0..N-1) -> SnapshotRegistry -> SequenceLog / SSI (CommittedWrites)`. Keine Locks über `.await`.
- **SSI-Validierung (INV-SSI-ACTIVE-1):** Erkennung von Anti-Dependency-Zyklen (Write-Skew) und Phantomschutz über Prefix-Tracking.
- **Fail-Closed & Sentinel-Schutz:** Sentinel `u64::MAX` oder Sequenzen mit `TOMBSTONE_BIT` / `>= i64::MAX as u64` werden in `prune_through` strikt abgewiesen, um eine vorzeitige Invalidierung von Snapshots zu verhindern.
- **Zero-Unsafe (M9):** `#![forbid(unsafe_code)]` strikt durchgesetzt.

## Einbettungsbeispiel
Ein vollständiges, autarkes Verwendungsbeispiel findet sich unter `examples/mvcc_basic.rs`.

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
read_set.record_read(b"key:1", 100);

buffer.begin(tx);
buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(1u64), data: "val".into() }).unwrap();

// Validierung vor Commit
assert!(ssi.validate(tx, &read_set).is_ok());
drop(guard);
```
