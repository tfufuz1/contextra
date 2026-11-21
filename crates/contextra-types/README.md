# contextra-types

**Zweck:** Kanonische Domänentypen: IDs (teilweise), Budgets, Filter, Fehlertypen. Laut Spec **nicht in der Zielstruktur**: REM-30 entfernt das Crate, Inhalte gehen nach `contextra-ids` und `contextra-ports`.

**Ebene:** keine Zielebene (REM-30, wird aufgelöst)

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod error
pub mod error_dto
pub mod model_fingerprint
pub mod retrieval_strategy
pub mod schema
pub mod tenant_scope
pub mod tombstone
pub mod types
pub mod erasure
pub mod ids
pub mod refs
pub mod witness
pub use erasure::*
pub use ids::{ErasureId, FileId, GateId, Seq, StoreId, Timestamp}
pub use refs::{CollectionRef, DocKey, OperatorRef, SubjectRef, TenantRef, MAX_REF_LEN}
pub use error::{ContextraError, HnswDeletionError, Result}
pub use error_dto::ContextraErrorDto
pub use model_fingerprint::ModelFingerprint
pub use retrieval_strategy::RetrievalStrategy
pub use schema::{DocIdWidth, ManifestSchemaVersion}
pub use tenant_scope::{TenantScopeViolation, TenantScoped}
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck}
pub use types::*
pub use witness::{RequestContext, SagaRecord, ScanMethod}
```

## Features (`Cargo.toml`)

`default`, `docid-128`, `auto-extraction-opt-out`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

