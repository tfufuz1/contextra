# Port Contract & Facade Leak Audit (M3 / M7)

> Generiert durch `tools/port_contract_auditor.py`.
> M3: Kein Fremd-Reexport (`pub use contextra_*`). M7: Reine mechanische Bereinigung in Datenhaltern.

## Crate Übersicht

| Crate | Rolle | M3 API-Leaks | Implementierte Ports | M7 Domain-Leaks (Holders) |
|---|---|---|---|---|
| `contextra` | Baustein | 0 | - | 0 |
| `contextra-adapt` | Baustein | 0 | - | 0 |
| `contextra-agent` | Baustein | 0 | `StorageEngine` | 0 |
| `contextra-audit-export` | Baustein | 0 | - | 0 |
| `contextra-avv-generator` | Baustein | 0 | - | 0 |
| `contextra-checkpoint` | Datenhalter | 0 | `StorageEngine` | 0 |
| `contextra-cognition` | Baustein | 0 | `Clock`, `StorageEngine` | 0 |
| `contextra-core` | Baustein | 7 | - | 0 |
| `contextra-crypto` | Baustein | 0 | `Vfs` | 0 |
| `contextra-db` | Baustein | 7 | - | 0 |
| `contextra-durable-fs` | Baustein | 0 | `Vfs` | 0 |
| `contextra-engine` | Baustein | 4 | `StorageEngine`, `VectorIndex` | 0 |
| `contextra-erasure` | Baustein | 0 | - | 0 |
| `contextra-graph` | Datenhalter | 0 | `Purger` | 0 |
| `contextra-infer-candle` | Baustein | 0 | - | 0 |
| `contextra-infer-ollama` | Baustein | 0 | - | 0 |
| `contextra-infer-onnx` | Baustein | 0 | `Vfs` | 0 |
| `contextra-ingest` | Baustein | 0 | - | 0 |
| `contextra-kvcache` | Datenhalter | 0 | `LayerScanner` | 0 |
| `contextra-license` | Baustein | 0 | - | 0 |
| `contextra-mcp` | Baustein | 0 | `Clock` | 0 |
| `contextra-mvcc` | Baustein | 0 | - | 0 |
| `contextra-ports` | Baustein | 0 | `Clock`, `IdGen`, `LayerScanner`, `MetricsSink`, `Rng`, `TextIndex`, `VectorIndex`, `Vfs` | 0 |
| `contextra-privacy` | Baustein | 0 | `Clock` | 0 |
| `contextra-py` | Baustein | 0 | - | 0 |
| `contextra-rank` | Baustein | 0 | `Clock` | 0 |
| `contextra-router` | Baustein | 6 | `Clock` | 0 |
| `contextra-sandbox` | Baustein | 0 | - | 0 |
| `contextra-simd` | Baustein | 0 | - | 0 |
| `contextra-store` | Datenhalter | 0 | `LayerScanner`, `Purger`, `StorageEngine` | 0 |
| `contextra-sys` | Baustein | 0 | - | 0 |
| `contextra-testkit` | Baustein | 0 | `Clock`, `MetricsSink`, `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs` | 0 |
| `contextra-text` | Datenhalter | 0 | `LayerScanner`, `Purger`, `StorageEngine`, `TextIndex` | 0 |
| `contextra-types` | Baustein | 0 | - | 0 |
| `contextra-vector` | Datenhalter | 0 | `Clock`, `LayerScanner`, `MetricsSink`, `Purger`, `VectorIndex` | 0 |
| `contextra-wire` | Baustein | 0 | - | 0 |

## Befunde & Verstöße

### M3 Leaks in `contextra-core`

- `/app/crates/contextra-core/src/lib.rs:135`: `pub use contextra_mvcc::seq_log;`
- `/app/crates/contextra-core/src/lib.rs:144`: `pub use contextra_mvcc::snapshot;`
- `/app/crates/contextra-core/src/lib.rs:153`: `pub use contextra_mvcc::tx_buffer;`
- `/app/crates/contextra-core/src/lib.rs:164`: `pub use contextra_wire::*;`
- `/app/crates/contextra-core/src/lib.rs:179`: `pub use contextra_mvcc::seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};`
- `/app/crates/contextra-core/src/lib.rs:191`: `pub use contextra_mvcc::snapshot::{SnapshotGuard, SnapshotRegistry};`
- `/app/crates/contextra-core/src/lib.rs:202`: `pub use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer};`
### M3 Leaks in `contextra-db`

- `/app/crates/contextra-db/src/lib.rs:44`: `pub use contextra_engine::collection::{self, Collection, CollectionConfig};`
- `/app/crates/contextra-db/src/lib.rs:45`: `pub use contextra_engine::{`
- `/app/crates/contextra-db/src/lib.rs:73`: `pub use contextra_engine::DurabilityMode;`
- `/app/crates/contextra-db/src/lib.rs:75`: `pub use contextra_engine::{`
- `/app/crates/contextra-db/src/lib.rs:82`: `pub use contextra_engine::SandboxBridge;`
- `/app/crates/contextra-db/src/lib.rs:92`: `pub use contextra_engine::decay_controller::*;`
- `/app/crates/contextra-db/src/lib.rs:94`: `pub use contextra_engine::fusion;`
### M3 Leaks in `contextra-engine`

- `/app/crates/contextra-engine/src/lib.rs:20`: `pub use contextra_crypto::deletion_proof::{`
- `/app/crates/contextra-engine/src/lib.rs:30`: `pub use contextra_store::lsm::DurabilityMode;`
- `/app/crates/contextra-engine/src/lib.rs:102`: `pub use contextra_text::Language;`
- `/app/crates/contextra-engine/src/lib.rs:396`: `pub use contextra_impl::erasure::{`
### M3 Leaks in `contextra-router`

- `/app/crates/contextra-router/src/lib.rs:35`: `pub use contextra_adapt::bandit;`
- `/app/crates/contextra-router/src/lib.rs:40`: `pub use contextra_adapt::lyapunov;`
- `/app/crates/contextra-router/src/lib.rs:45`: `pub use contextra_adapt::offpolicy;`
- `/app/crates/contextra-router/src/lib.rs:60`: `pub use contextra_adapt::{DriftReason, LyapunovDriftWatcher, LyapunovResult};`
- `/app/crates/contextra-router/src/lib.rs:61`: `pub use contextra_adapt::{OffPolicyEvaluator, OffPolicyStats};`
- `/app/crates/contextra-router/src/lib.rs:69`: `pub use contextra_adapt::{BanditImplementation, BanditPolicy, BanditProfileState};`
