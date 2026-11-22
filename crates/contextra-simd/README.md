# contextra-simd

**Zweck:** Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz. **Zweite erlaubte `unsafe`-Insel** (neben `contextra-sys`). Die SIMD-Varianten müssen identische Ergebnisse liefern wie die Skalar-Referenz (Selbstprüfung).

**Ebene:** E0 (unsafe-Insel)

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod contract
pub mod dispatch
pub mod error
pub mod kernels
pub mod metric
pub mod compat
pub use contract::{DistanceFn, MetricDistanceFn}
pub use dispatch::
pub use dispatch::{get_dispatch_override, set_dispatch_override, SimdPathOverride}
pub use error::{Error, Result, SimdError}
pub use kernels::scalar::
pub use metric::DistanceMetric
pub fn validate_vector(vec: &[f32]) -> Result<()>
pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
pub fn compute_distance_f32_bytes_trusted(
```

## Features (`Cargo.toml`)

`default`, `std`, `serde`, `workspace`, `types-compat`, `simd-avx512`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

