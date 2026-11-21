# contextra-testkit

**Zweck:** Prüfwerkzeuge für Tests: `ManualClock`, `InMemoryStorageEngine`, `FaultVfs`, Canary-Scanner, Mocks. **Nur Dev-Abhängigkeit** laut Spec (F-27). Hier soll `tests/layering.rs` liegen (G-03, **fehlt**).

**Ebene:** E3, nur Dev-Abhängigkeit

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod conformance
pub mod fault_vfs
pub mod in_memory_graph
pub mod in_memory_store
pub mod in_memory_text
pub mod in_memory_vector
pub mod manual_clock
pub mod mocks
pub mod reference_model
pub mod doubles
pub mod fault
pub mod model
pub use fault_vfs::{FaultConfig, FaultVfs}
pub use in_memory_graph::InMemoryGraphIndex
pub use in_memory_store::InMemoryStorageEngine
pub use in_memory_text::InMemoryTextIndex
pub use in_memory_vector::InMemoryVectorIndex
pub use manual_clock::ManualClock
pub use mocks::*
pub use reference_model::{RefOp, ReferenceModel}
pub fn mint_witness_for_tests(
```

## Features (`Cargo.toml`)

`default`, `testkit`, `erasure-conformance`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

