# contextra-ports

**Zweck:** Kanonische, dyn-kompatible **Port-Traits** und DTOs für alle Subsysteme. Ein Port-Trait ist genau hier definiert (R9); implementiert wird es in dem Crate, das den Zustand besitzt (SPEC §3.4).

**Ebene:** E0

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub use contextra_types::
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
pub type BoxStream<'a, T> = Pin<Box<dyn futures_util::stream::Stream<Item = T> + Send + 'a>>
pub mod attention
pub mod checkpoint
pub mod clock
pub mod conformance
pub mod embedding
pub mod graph
pub mod graph_index
pub mod id_gen
pub mod kv
pub mod kv_bridge_port
pub mod license
pub mod lifecycle
pub mod metrics
pub mod observability
pub mod plugin
pub mod query_rewriter
pub mod reranker
pub mod rng
pub mod scanner
pub mod storage
pub mod text_index
pub mod vector_index
pub mod vfs
pub mod key_material
pub mod env
pub mod erasure_legacy
pub mod fs
pub mod index
pub mod inference
pub mod policy
pub use attention::*
pub use checkpoint::*
pub use clock::*
pub use conformance::*
pub use embedding::*
pub use graph::*
pub use graph_index::*
pub use id_gen::*
pub use key_material::KeyMaterialSource
pub use kv::*
pub use kv_bridge_port::*
pub use license::*
pub use lifecycle::*
pub use metrics::*
pub use observability::*
pub use plugin::*
pub use query_rewriter::*
pub use reranker::*
pub use rng::*
pub use scanner::
pub use storage::{StorageEngine, StorageStats, MAX_SCAN_MERGE_ACCUMULATOR}
pub use text_index::*
pub use vector_index::*
pub use vfs::*
pub mod contract_tests
pub use metrics::
pub mod barrier
```

(weitere 13 Zeilen, siehe Quelle)

## Features (`Cargo.toml`)

`default`, `conformance`, `contract-tests`, `testkit`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

