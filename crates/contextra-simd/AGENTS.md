# AGENTS.md — `contextra-simd`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-simd` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz.
- **src:** 11 Dateien, 1942 LOC · **unsafe:** 269 Vorkommen, Policy `keiner`
- **Interne Abhängigkeiten (normal, 1):** `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 4):** `bytemuck`, `serde`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (1):** `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend `contextra-types`; eingehend –
- **Features (6):** `default`, `serde`, `simd-avx512`, `std`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (4): `CosineSimilarityPartsF32U8`, `CosineSimilarityPartsU8`, `Dispatcher`, `MetricDistanceFn`
- enum (3): `DistanceMetric`, `SimdError`, `SimdTarget`
- trait (1): `DistanceFn`
- fn (34): `auto`, `compute_distance`, `compute_distance_f32_bytes_trusted`, `compute_distance_trusted`, `cosine_distance`, `cosine_distance_f32_bytes`, `cosine_distance_f32_bytes_scalar`, `cosine_distance_scalar`, `cosine_similarity_parts_f32_u8`, `cosine_similarity_parts_u8`, `cosine_similarity_parts_u8_scalar`, `detect_best_target`, `dot_product_distance`, `dot_product_distance_f32_bytes`, `dot_product_f32_bytes_scalar` … (+19)
- type (3): `Error`, `Result`, `SimdPathOverride`
- mod (7): `compat`, `contract`, `dispatch`, `error`, `kernels`, `metric`, `scalar`
- pub use: `contract::{DistanceFn, MetricDistanceFn}`, `crate::error::{Error, Result, SimdError}`, `crate::metric::DistanceMetric`, `dispatch::{ cosine_distance, cosine_distance_f32_bytes, cosine_similarity_parts_u8, dot_product_distance, dot_product_distance_f32_bytes, dot_product_u8, euclidean_distance, euclidean_distance_f32_bytes, euclidean_distance_sq_u8, Dispatcher, SimdTarget, }`, `dispatch::{get_dispatch_override, set_dispatch_override, SimdPathOverride}`, `error::{Error, Result, SimdError}`, `kernels::scalar::{ cosine_distance_f32_bytes_scalar, cosine_distance_scalar, cosine_similarity_parts_f32_u8, cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_f32_u8, dot_product_scalar, dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar, euclidean_distance_sq_f32_u8, euclidean_distance_sq_u8_scalar, normalize_inplace, CosineSimilarityPartsF32U8, CosineSimilarityPartsU8, }`, `metric::DistanceMetric`, `scalar::*`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->
