# AGENTS.md — contextra-simd — Hardware-Vektorkerne & SIMD-Dispatch

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-simd.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-simd.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2.3 (R5 Unsafe-Inseln), §13 (INV-3, INV-5 Determinismus)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E0 (Autarkes Fundament / Ring 0 — Genehmigte Unsafe-Insel)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Unsafe-Status:** Erlaubt (Genehmigte Unsafe-Insel nach SPEC §2.3 R5 & INV-3)  

---

## 1. Zweck & Systemrolle

Hardware-beschleunigte SIMD-Distanzkerne (Cosine, L2, Dot) für Gleitkomma- und SQ8-Vektoren.

---

## 2. Kern-Richtlinien für Agenten

- **Unsafe-Beschränkung (INV-3):** `unsafe` ist ausschließlich für SIMD-Intrinsics in `kernels/avx2.rs`, `kernels/avx512.rs` und `kernels/neon.rs` erlaubt.
- **Sichere API:** Alle öffentlichen Funktionen sind vollständig sichere Rust-Funktionen mit Validierung vor dem Intrinsic-Aufruf.
- **Dynamischer Dispatch:** Laufzeiterkennung via CPUID/auxval; automatischer Fallback auf die mathematische Referenz in `kernels/scalar.rs`.
- **Eingabe-Validierung & NaN-Schutz:** Jeder Vektor wird geprüft. `NaN` oder `Infinity` führt sofort zu `Err(SimdError::InvalidFloat)`. Dimensions-Mismatches führen zu `Err(SimdError::DimensionMismatch)`.
- **Bitidentischer Determinismus (INV-5):** SIMD-Pfade müssen auf derselben Architektur bitidentische Ergebnisse zur skalaren Referenz liefern.

## 3. Test-Pflichten
- T-SIMD1 bis T-SIMD5 (Differenzialtest AVX2/AVX-512/NEON gegen Skalar, NaN-Schutz, Dimensionsmismatch, Alignment-Resilienz, SQ8-Quantisierung).

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-simd` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz.
- **src:** 11 Dateien, 1959 LOC · **unsafe:** 286 Vorkommen, Policy `keiner`
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

<!-- END GENERATED: crate_inventory.py -->
