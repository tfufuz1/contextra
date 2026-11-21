# contextra-vector

**Zweck:** Vektorindex: HNSW, SQ8, exakte Suche, ACORN-artige Traversierung, Filter (SPEC §6.3). Die Entscheidung „DiskANN und RaBitQ raus“ (ADR-04) steht im Soll-Zustand.

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod casting
pub mod compat
pub mod contract
pub mod purge
pub mod vector_index
pub mod error
pub mod types
pub mod diskann
pub mod distance
pub mod hnsw
pub mod persistence
pub mod quantize
pub mod acorn
pub mod compute_pool
pub mod fixed_hasher
pub mod partial_rebuild
pub mod quantize_rabitq
pub use compat::candidate_stream::{VectorCandidateStream, DEFAULT_VECTOR_STREAM_BATCH_SIZE}
pub use compute_pool::ComputePool
pub use purge::{VectorPurgeOutcome, VectorPurger}
pub use vector_index::{VectorIndex, VectorSearchHit}
pub use contract::
pub use diskann::{DiskAnnConfig, DiskAnnFallbackPolicy, DiskAnnIndex}
pub use hnsw::{hnsw_purger, hnsw_scanner, HnswPurger, HnswScanner}
pub use hnsw::{HnswConfig, HnswIndex, RebuildStatus}
pub use persistence::{HnswHeader, MmapIndex}
pub use quantize::ScalarQuantizer
pub use quantize_rabitq::RaBitQQuantizer
pub use types::{Clock, DistanceMetric, DocId, Maintain, MetricsSink, Rng, ScoredDocument, TxId}
```

## Features (`Cargo.toml`)

`default`, `std`, `simd`, `serde`, `tracing`, `inspect`, `experimental`, `mmap`, `diskann`, `quantize-rabitq`, `workspace`, `ports-compat`, `docid-128`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`, `experimental-all`, `partial-index-rebuild`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

