# API: contextra-simd

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 62 öffentliche Items, 28 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- keine

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/dispatch.rs:31  pub enum SimdTarget
src/dispatch.rs:48  pub type SimdPathOverride = SimdTarget
src/dispatch.rs:58  pub fn set_dispatch_override(path: Option<SimdPathOverride>)
src/dispatch.rs:65  pub fn get_dispatch_override() -> Option<SimdPathOverride>
src/dispatch.rs:75  pub struct Dispatcher
src/dispatch.rs:84  pub fn detect_best_target() -> SimdTarget
src/dispatch.rs:142  pub fn auto() -> Self
src/dispatch.rs:150  pub const fn scalar() -> Self
src/dispatch.rs:158  pub const fn new(target: SimdTarget) -> Self
src/dispatch.rs:164  pub const fn target(&self) -> SimdTarget
src/dispatch.rs:170  pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>
src/dispatch.rs:228  pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>
src/dispatch.rs:286  pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>
src/dispatch.rs:344  pub fn cosine_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>
src/dispatch.rs:389  pub fn euclidean_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>
src/dispatch.rs:436  pub fn dot_product_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32>
src/dispatch.rs:481  pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>
src/dispatch.rs:531  pub fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> Result<u32>
src/dispatch.rs:576  pub fn cosine_similarity_parts_u8(
src/dispatch.rs:653  pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32>
src/dispatch.rs:684  pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32>
src/dispatch.rs:715  pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32>
src/dispatch.rs:746  pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>
src/dispatch.rs:777  pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>
src/dispatch.rs:808  pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32>
src/dispatch.rs:839  pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32>
src/dispatch.rs:870  pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32>
src/dispatch.rs:901  pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> Result<CosineSimilarityPartsU8>
src/error.rs:7  pub enum SimdError
src/error.rs:18  pub type Result<T> = core::result::Result<T, SimdError>
src/error.rs:21  pub type Error = SimdError
src/kernels/mod.rs:41  pub mod scalar
src/kernels/mod.rs:43  pub use scalar::*
src/kernels/scalar.rs:27  pub struct CosineSimilarityPartsU8
src/kernels/scalar.rs:39  pub struct CosineSimilarityPartsF32U8
src/kernels/scalar.rs:60  pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32
src/kernels/scalar.rs:91  pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32
src/kernels/scalar.rs:111  pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32
src/kernels/scalar.rs:131  pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/scalar.rs:168  pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/scalar.rs:194  pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/scalar.rs:218  pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32
src/kernels/scalar.rs:237  pub fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32
src/kernels/scalar.rs:258  pub fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8
src/kernels/scalar.rs:289  pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32
src/kernels/scalar.rs:308  pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32
src/kernels/scalar.rs:331  pub fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8
src/kernels/scalar.rs:360  pub fn normalize_inplace(v: &mut [f32])
src/lib.rs:33  pub mod dispatch
src/lib.rs:34  pub mod error
src/lib.rs:37  pub mod kernels
src/lib.rs:38  pub mod metric
src/lib.rs:40  pub use dispatch::
src/lib.rs:46  pub use dispatch::{get_dispatch_override, set_dispatch_override, SimdPathOverride}
src/lib.rs:47  pub use error::{Error, Result, SimdError}
src/lib.rs:48  pub use kernels::scalar::
src/lib.rs:55  pub use metric::DistanceMetric
src/lib.rs:86  pub fn validate_vector(vec: &[f32]) -> Result<()>
src/lib.rs:124  pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
src/lib.rs:147  pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32>
src/lib.rs:174  pub fn compute_distance_f32_bytes_trusted(
src/metric.rs:5  pub enum DistanceMetric
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/kernels/avx2.rs:36  pub(crate) unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32
src/kernels/avx2.rs:107  pub(crate) unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32
src/kernels/avx2.rs:158  pub(crate) unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32
src/kernels/avx2.rs:206  pub(crate) unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/avx2.rs:288  pub(crate) unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/avx2.rs:351  pub(crate) unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/avx2.rs:409  pub(crate) unsafe fn hsum256_ps_avx(v: __m256) -> f32
src/kernels/avx2.rs:432  pub(crate) unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32
src/kernels/avx2.rs:496  pub(crate) unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32
src/kernels/avx2.rs:565  pub(crate) unsafe fn cosine_similarity_parts_u8_avx2(
src/kernels/avx2.rs:651  pub(crate) unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32
src/kernels/avx512.rs:36  pub(crate) unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32
src/kernels/avx512.rs:106  pub(crate) unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32
src/kernels/avx512.rs:157  pub(crate) unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32
src/kernels/avx512.rs:205  pub(crate) unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/avx512.rs:287  pub(crate) unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/avx512.rs:350  pub(crate) unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32
src/kernels/avx512.rs:408  pub(crate) unsafe fn hsum512_ps_avx(v: __m512) -> f32
src/kernels/avx512.rs:433  pub(crate) unsafe fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32
src/kernels/avx512.rs:479  pub(crate) unsafe fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32
src/kernels/avx512.rs:548  pub(crate) unsafe fn cosine_similarity_parts_u8_avx512(
src/kernels/avx512.rs:617  pub(crate) unsafe fn hsum512_epi32_avx512(v: __m512i) -> i32
src/kernels/mod.rs:29  pub(crate) mod avx2
src/kernels/mod.rs:33  pub(crate) mod avx512
src/kernels/mod.rs:37  pub(crate) mod neon
src/kernels/neon.rs:34  pub(crate) unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32
src/kernels/neon.rs:97  pub(crate) unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32
src/kernels/neon.rs:143  pub(crate) unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32
```
