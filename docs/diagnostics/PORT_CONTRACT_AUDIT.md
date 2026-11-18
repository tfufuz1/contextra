# Port Contract & Facade Leak Audit (M3 / M7)

> Generiert durch `tools/port_contract_auditor.py`.
> M3: Kein Fremd-Reexport (`pub use contextra_*`). M7: Reine mechanische Bereinigung in Datenhaltern.

## Crate Übersicht

| Crate | Rolle | M3 API-Leaks | Implementierte Ports | M7 Domain-Leaks (Holders) |
|---|---|---|---|---|
| `contextra` | Baustein | 0 | - | 0 |
| `contextra-checkpoint` | Datenhalter | 0 | `StorageEngine` | 0 |
| `contextra-crypto` | Baustein | 0 | - | 0 |
| `contextra-db` | Baustein | 2 | `StorageEngine`, `VectorIndex` | 0 |
| `contextra-durable-fs` | Baustein | 0 | `Vfs` | 0 |
| `contextra-engine` | Baustein | 1 | - | 0 |
| `contextra-env` | Baustein | 0 | `Clock`, `IdGen`, `MetricsSink`, `Rng` | 0 |
| `contextra-erasure` | Baustein | 0 | `Clock`, `CommitBarrier`, `IdGen`, `LayerScanner`, `PurgeBarrier`, `ReadGate`, `Vfs` | 0 |
| `contextra-graph` | Datenhalter | 0 | `Purger`, `StorageEngine` | 0 |
| `contextra-ids` | Baustein | 0 | - | 0 |
| `contextra-mcp` | Baustein | 0 | `Clock` | 0 |
| `contextra-mvcc` | Baustein | 2 | `Clock` | 0 |
| `contextra-ports` | Baustein | 0 | `CommitBarrier`, `LayerScanner`, `PurgeBarrier`, `Purger`, `TextIndex`, `VectorIndex`, `Vfs` | 0 |
| `contextra-privacy` | Baustein | 0 | `Clock` | 0 |
| `contextra-rank` | Baustein | 0 | `Clock` | 0 |
| `contextra-simd` | Baustein | 0 | - | 0 |
| `contextra-store` | Datenhalter | 0 | `LayerScanner`, `Purger`, `StorageEngine`, `Vfs` | 0 |
| `contextra-sys` | Baustein | 0 | - | 0 |
| `contextra-testkit` | Baustein | 0 | `Clock`, `MetricsSink`, `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs` | 0 |
| `contextra-text` | Datenhalter | 0 | `Clock`, `LayerScanner`, `Purger`, `StorageEngine`, `TextIndex` | 0 |
| `contextra-types` | Baustein | 0 | - | 0 |
| `contextra-vault` | Baustein | 0 | - | 0 |
| `contextra-vector` | Datenhalter | 0 | `Clock`, `LayerScanner`, `Purger`, `VectorIndex` | 0 |

## Befunde & Verstöße

### M3 Leaks in `contextra-db`

- `/home/freddy/Projekte/contextra/crates/contextra-db/src/lib.rs:58`: `pub use contextra_store::lsm::DurabilityMode;`
- `/home/freddy/Projekte/contextra/crates/contextra-db/src/lib.rs:101`: `pub use contextra_text::Language;`
### M3 Leaks in `contextra-engine`

- `/home/freddy/Projekte/contextra/crates/contextra-engine/src/lib.rs:15`: `pub use contextra_db::*;`
### M3 Leaks in `contextra-mvcc`

- `/home/freddy/Projekte/contextra/crates/contextra-mvcc/src/lib.rs:82`: `pub use contextra_ids::{DocId, TenantId, TxId, TOMBSTONE_BIT};`
- `/home/freddy/Projekte/contextra/crates/contextra-mvcc/src/lib.rs:86`: `pub use contextra_ids::{DocId, TenantId, TxId, TOMBSTONE_BIT};`
