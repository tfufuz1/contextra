# contextra-rank

**Zweck:** Ergebnis-Fusion und Ranking: RRF (Reciprocal Rank Fusion), Top-k, Zeitgewichtung, Erklärung, Isotonic- und Platt-Kalibrierung (Wahrscheinlichkeiten sind kalibriert, README §1), Drift-Erkennung. SPEC §6.6, REM-18.

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod compat
pub mod calibration
pub mod contract
pub mod drift
pub mod error
pub mod explain
pub mod fusion
pub mod local_types
pub mod metrics
pub mod temporal
pub use error::{RankError, Result}
pub use local_types::
pub mod dibud
pub use calibration::
pub use dibud::
pub use drift::{DriftDetector, DriftStatus}
pub use explain::{explain, ExplanationEntry, RetrievalExplanation}
pub use fusion::
pub use metrics::{mean_reciprocal_rank, ndcg_at_k, precision_at_k, recall_at_k, reciprocal_rank}
pub use temporal::
pub use fusion::{apply_resonance_bonus, fuse_signals, ResonanceConfig}
pub use fusion::ResonanceModifier
```

## Features (`Cargo.toml`)

`default`, `std`, `workspace`, `types-compat`, `ports-compat`, `dibud`, `exp-dibud`, `exp-resonance`, `experimental-all`, `experimental-rank`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

