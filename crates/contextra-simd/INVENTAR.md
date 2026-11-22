
## contextra-simd

Pfad: `crates/contextra-simd`

Abhängigkeiten (direkt): thiserror, bytemuck, tracing, serde, contextra-types

### Module

- `/src/compat/mod.rs`
- `/src/contract.rs`
- `/src/dispatch.rs`
- `/src/error.rs`
- `/src/kernels/avx2.rs`
- `/src/kernels/avx512.rs`
- `/src/kernels/mod.rs`
- `/src/kernels/neon.rs`
- `/src/kernels/scalar.rs`
- `/src/lib.rs`
- `/src/metric.rs`

### Öffentliche Typen

- **CosineSimilarityPartsF32U8** (struct) – src/kernels/scalar.rs:40
- **CosineSimilarityPartsU8** (struct) – src/kernels/scalar.rs:28
- **Dispatcher** (struct) – src/dispatch.rs:80
- **DistanceFn** (trait) – src/contract.rs:21
- **DistanceMetric** (enum) – src/metric.rs:14
- **Error** (type) – src/error.rs:41
- **MetricDistanceFn** (struct) – src/contract.rs:31
- **Result** (type) – src/error.rs:38
- **SimdError** (enum) – src/error.rs:15
- **SimdPathOverride** (type) – src/dispatch.rs:51
- **SimdTarget** (enum) – src/dispatch.rs:32

### Öffentliche Funktionen

- `pub fn auto() -> Self `  – src/dispatch.rs:148
- `pub fn compute_distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32> `  – src/lib.rs:137
- `pub fn compute_distance_f32_bytes_trusted(`  – src/lib.rs:190
- `pub fn compute_distance_trusted(a: &[f32], b: &[f32], metric: DistanceMetric) -> Result<f32> `  – src/lib.rs:162
- `pub fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> `  – src/dispatch.rs:176
- `pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32> `  – src/dispatch.rs:662
- `pub fn cosine_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32> `  – src/dispatch.rs:351
- `pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32> `  – src/dispatch.rs:755
- `pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 `  – src/kernels/scalar.rs:138
- `pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 `  – src/kernels/scalar.rs:63
- `pub fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8 `  – src/kernels/scalar.rs:342
- `pub fn cosine_similarity_parts_u8(`  – src/dispatch.rs:583
- `pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> Result<CosineSimilarityPartsU8> `  – src/dispatch.rs:910
- `pub fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 `  – src/kernels/scalar.rs:267
- `pub fn detect_best_target() -> SimdTarget `  – src/dispatch.rs:89
- `pub fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> `  – src/dispatch.rs:293
- `pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32> `  – src/dispatch.rs:724
- `pub fn dot_product_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32> `  – src/dispatch.rs:443
- `pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32> `  – src/dispatch.rs:817
- `pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 `  – src/kernels/scalar.rs:201
- `pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32 `  – src/kernels/scalar.rs:300
- `pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 `  – src/kernels/scalar.rs:116
- `pub fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> Result<u32> `  – src/dispatch.rs:488
- `pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32> `  – src/dispatch.rs:848
- `pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32 `  – src/kernels/scalar.rs:227
- `pub fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> `  – src/dispatch.rs:235
- `pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32> `  – src/dispatch.rs:693
- `pub fn euclidean_distance_f32_bytes(&self, a: &[f32], b_bytes: &[u8]) -> Result<f32> `  – src/dispatch.rs:396
- `pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32> `  – src/dispatch.rs:786
- `pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 `  – src/kernels/scalar.rs:175
- `pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 `  – src/kernels/scalar.rs:96
- `pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32 `  – src/kernels/scalar.rs:319
- `pub fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> Result<u32> `  – src/dispatch.rs:538
- `pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32> `  – src/dispatch.rs:879
- `pub fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32 `  – src/kernels/scalar.rs:246
- `pub fn get_dispatch_override() -> Option<SimdPathOverride> `  – src/dispatch.rs:68
- `pub fn new(metric: DistanceMetric) -> Self `  – src/contract.rs:41
- `pub fn normalize_inplace(v: &mut [f32]) `  – src/kernels/scalar.rs:371
- `pub fn set_dispatch_override(path: Option<SimdPathOverride>) `  – src/dispatch.rs:61
- `pub fn validate_vector(vec: &[f32]) -> Result<()> `  – src/lib.rs:98

---
