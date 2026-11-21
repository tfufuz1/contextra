# contextra-erasure

**Zweck:** Löschkern: Löschkette (Saga), ReadGate, Tresor, Messung, Schichtschnittstelle, Scrub-Planung. Spec: **Kapitel 7** (Löschkette v4.1) und Kapitel 4 (Entkopplung von Löschung und Versionierung, Seal-Modell).

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod chain
pub mod crypto_local
pub mod error
pub mod gate
pub mod io
pub mod types_local
pub mod vault
pub use gate::saga_gate::{GateDecision, RangeGate, ReadGate, SagaGate}
```

## Features (`Cargo.toml`)

`default`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

