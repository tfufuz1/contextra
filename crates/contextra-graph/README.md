# contextra-graph

**Zweck:** Optionaler CSR-Graph für Entitäten-Beziehungen, Traversierung, PathRAG, Seal-Provenienz. **Optional** in der Zielarchitektur (ADR-21 bis ADR-27, SPEC §6.10). Die Reduktion (§12.4) passiert nur, wenn Gate G-GRAPH grün ist.

**Ebene:** E1, optional

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod apprh
pub mod arc_slice
pub mod cascade
pub mod community
pub mod compat
pub mod consistency_enforcement
pub mod csr
pub mod edge_reinforcement
pub mod edge_reinforcement_buffer
pub mod entity_extraction
pub mod error
pub mod hyperedge
pub mod hyperedge_suggest
pub mod path_rag
pub mod percolation
pub mod ppr
pub mod ppr_stream
pub mod provenance
pub mod provider
pub mod purger
pub mod session_dag
pub mod test_doubles
pub mod types
pub mod tl_hfd
pub mod storage
pub mod graph_view
pub mod text_gen
pub use apprh::
pub use arc_slice::ArcSlice
pub use cascade::
pub use community::
pub use consistency_enforcement::
pub use csr::CsrGraph
pub use edge_reinforcement::
pub use edge_reinforcement_buffer::{CooccurrenceSignal, EdgeReinforcementBuffer, TraversalSignal}
pub use entity_extraction::
pub use error::GraphError as Error
pub use error::{GraphError, GraphMutationError, Result}
pub use hyperedge::
pub use hyperedge_suggest::
pub use hyperedge_suggest::validate_candidates_with_llm
pub use path_rag::{GraphPath, PathGraph, PathRAGConfig, PathRAGEngine}
pub use path_rag::{KPathConfig, KPathDiffusion, KPathResult}
pub use percolation::
pub use ppr::compute_ppr_with_apprh_selector
pub use ppr::{compute_ppr, compute_ppr_at, DeletedView, PprContext}
pub use ppr_stream::{PprCandidateStream, DEFAULT_PPR_STREAM_BATCH_SIZE}
pub use provenance::{DocEdgeIndex, EdgeProvenance}
pub use provider::
pub use purger::{graph_purger, GraphPurger}
pub use session_dag::
pub use tl_hfd::
pub use compat::graph_index_ext::GraphIndexExt
pub use storage::{GraphFuture, GraphStorage, InMemoryGraphStorage}
pub use graph_view::GraphView
pub use text_gen::TextGenerator
pub use types::{EntityId, PprAlgorithm, PprConfig}
```

## Features (`Cargo.toml`)

`default`, `workspace`, `session-dag`, `entity-extraction`, `hyperedge-suggest`, `apprh-diffusion`, `k-path-diffusion`, `graph-connectivity-health`, `edge-reinforcement-learning`, `docid-128`, `test-support`, `types-compat`, `ports-compat`, `experimental-graph`, `physio-percolation`, `ppr-forward-push`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

