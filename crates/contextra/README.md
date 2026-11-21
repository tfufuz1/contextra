# contextra

**Zweck:** Öffentliche Fassade und API: Konfigurationsladen, Builder, Agent-Gedächtnis, Query-Rewriter-Anbindung, Performance-Profile. SPEC §6.8 (Fassade, Fehler, Schnittstellen). Die Crate-Verträge sind in SPEC §15.

**Ebene:** E2 (Fassade)

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod agent_memory
pub mod boot
pub mod builder
pub mod collection_profile
pub mod performance_profile
pub mod query_rewriter
pub use agent_memory::{AgentMemory, Memory, MemoryId}
pub use builder::{ContextraBuilder, ContextraConfigExt, EncryptionMode}
pub use collection_profile::{CollectionProfile, DeploymentTier, KvDeleteMode, LsmTuning}
pub use contextra_db::
pub use contextra_types::error::ContextraError
pub use contextra_types::types::domain::{DocId, ScoredDocument}
pub use contextra_types::DistanceMetric
pub use performance_profile::{DurabilityConfigError, DurabilityMode, PerformanceProfile}
pub use query_rewriter::LlmQueryRewriter
pub fn builder(dimension: usize) -> ContextraBuilder
```

## Features (`Cargo.toml`)

`default`, `embedded`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

