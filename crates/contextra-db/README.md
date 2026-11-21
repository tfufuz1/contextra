# contextra-db

**Zweck:** Orchestrierung der Hybridsuche: Schreibpfad, Lesepfad, Scheduler, Wiederanlauf, Verdrahtung der Ports. SPEC §13.1 Meilenstein M2. **Hier fehlt am meisten gegenüber der Spec**: Ist 412 Zeilen, Ziel 6.000.

**Ebene:** E2

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub use erasure::ErasureWiring
pub use lineage::{DerivedRef, LineageIndex, LineageRef, NonEmpty, SourceId, SourceRef}
pub use storage::{Document, GateAudit, HybridCandidate, TenantScopedStorage}
```

## Features (`Cargo.toml`)

`default`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

