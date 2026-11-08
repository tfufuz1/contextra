# Unsafe Boundary & Island Compliance Audit

> Generiert durch `tools/unsafe_boundary_auditor.py`.
> Erlaubte Unsafe-Inseln: `['contextra-simd', 'contextra-sys', 'contextra-wire']`

## Crate Übersicht

| Crate | Status | Forbid Attribute | Unsafe Total | Fehlende `// SAFETY:` |
|---|---|---|---|---|
| `contextra` | REIN | Ja | 0 | 0 |
| `contextra-adapt` | REIN | Ja | 0 | 0 |
| `contextra-agent` | REIN | Ja | 0 | 0 |
| `contextra-audit-export` | REIN | Ja | 0 | 0 |
| `contextra-avv-generator` | REIN | Ja | 0 | 0 |
| `contextra-checkpoint` | REIN | Ja | 0 | 0 |
| `contextra-cognition` | REIN | Ja | 0 | 0 |
| `contextra-core` | REIN | Ja | 0 | 0 |
| `contextra-crypto` | REIN | Ja | 0 | 0 |
| `contextra-db` | REIN | Ja | 0 | 0 |
| `contextra-durable-fs` | REIN | Ja | 0 | 0 |
| `contextra-engine` | REIN | Ja | 0 | 0 |
| `contextra-erasure` | REIN | Ja | 0 | 0 |
| `contextra-graph` | **VERBOTEN** | Ja | 5 | 0 |
| `contextra-infer-candle` | REIN | Ja | 0 | 0 |
| `contextra-infer-ollama` | REIN | Ja | 0 | 0 |
| `contextra-infer-onnx` | REIN | Ja | 0 | 0 |
| `contextra-ingest` | REIN | Ja | 0 | 0 |
| `contextra-kvcache` | REIN | Ja | 0 | 0 |
| `contextra-license` | REIN | Ja | 0 | 0 |
| `contextra-mcp` | REIN | Ja | 0 | 0 |
| `contextra-mvcc` | REIN | Ja | 0 | 0 |
| `contextra-ports` | REIN | Ja | 0 | 0 |
| `contextra-privacy` | REIN | Ja | 0 | 0 |
| `contextra-py` | REIN | Ja | 0 | 0 |
| `contextra-rank` | REIN | Ja | 0 | 0 |
| `contextra-router` | REIN | Ja | 0 | 0 |
| `contextra-sandbox` | REIN | Ja | 0 | 0 |
| `contextra-simd` | Insel (Erlaubt) | Nein | 272 | 18 |
| `contextra-store` | REIN | Ja | 0 | 0 |
| `contextra-sys` | Insel (Erlaubt) | Nein | 21 | 0 |
| `contextra-testkit` | REIN | Ja | 0 | 0 |
| `contextra-text` | **VERBOTEN** | Ja | 5 | 1 |
| `contextra-types` | REIN | Ja | 0 | 0 |
| `contextra-vector` | REIN | Ja | 0 | 0 |
| `contextra-wire` | Insel (Erlaubt) | Ja | 54 | 9 |

## Auffälligkeiten & Details

### `contextra-graph`

- `/app/crates/contextra-graph/tests/ppr_alloc_test.rs:22` [unsafe_impl] (OK): `unsafe impl GlobalAlloc for CountingAllocator {`
- `/app/crates/contextra-graph/tests/ppr_alloc_test.rs:25` [unsafe_fn] (OK): `unsafe fn alloc(&self, layout: Layout) -> *mut u8 {`
- `/app/crates/contextra-graph/tests/ppr_alloc_test.rs:30` [unsafe_block] (OK): `unsafe { System.alloc(layout) }`
- `/app/crates/contextra-graph/tests/ppr_alloc_test.rs:34` [unsafe_fn] (OK): `unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {`
- `/app/crates/contextra-graph/tests/ppr_alloc_test.rs:37` [unsafe_block] (OK): `unsafe { System.dealloc(ptr, layout) }; // SAFETY: Forward ptr and layout to System allocator.`
### `contextra-simd`

- `/app/crates/contextra-simd/src/dispatch.rs:122` [unsafe_block] (OK): `Ok(unsafe { avx2::cosine_distance_avx2(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:127` [unsafe_block] (OK): `Ok(unsafe { avx512::cosine_distance_avx512(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:132` [unsafe_block] (OK): `return Ok(unsafe { neon::cosine_distance_neon(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:139` [unsafe_block] (OK): `return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:143` [unsafe_block] (OK): `return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:151` [unsafe_block] (OK): `return Ok(unsafe { neon::cosine_distance_neon(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:177` [unsafe_block] (OK): `Ok(unsafe { avx2::euclidean_distance_avx2(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:182` [unsafe_block] (OK): `Ok(unsafe { avx512::euclidean_distance_avx512(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:187` [unsafe_block] (OK): `return Ok(unsafe { neon::euclidean_distance_neon(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:194` [unsafe_block] (OK): `return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:198` [unsafe_block] (OK): `return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:206` [unsafe_block] (OK): `return Ok(unsafe { neon::euclidean_distance_neon(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:232` [unsafe_block] (OK): `Ok(unsafe { avx2::dot_product_avx2(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:237` [unsafe_block] (OK): `Ok(unsafe { avx512::dot_product_avx512(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:242` [unsafe_block] (OK): `return Ok(unsafe { neon::dot_product_neon(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:249` [unsafe_block] (OK): `return Ok(unsafe { avx512::dot_product_avx512(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:253` [unsafe_block] (OK): `return Ok(unsafe { avx2::dot_product_avx2(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:261` [unsafe_block] (OK): `return Ok(unsafe { neon::dot_product_neon(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:291` [unsafe_block] (OK): `Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) })`
- `/app/crates/contextra-simd/src/dispatch.rs:296` [unsafe_block] (OK): `Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) })`
- `/app/crates/contextra-simd/src/dispatch.rs:303` [unsafe_block] (OK): `return Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) });`
- `/app/crates/contextra-simd/src/dispatch.rs:307` [unsafe_block] (OK): `return Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) });`
- `/app/crates/contextra-simd/src/dispatch.rs:337` [unsafe_block] (OK): `Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) })`
- `/app/crates/contextra-simd/src/dispatch.rs:342` [unsafe_block] (OK): `Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) })`
- `/app/crates/contextra-simd/src/dispatch.rs:349` [unsafe_block] (OK): `return Ok(unsafe {`
- `/app/crates/contextra-simd/src/dispatch.rs:355` [unsafe_block] (OK): `return Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) });`
- `/app/crates/contextra-simd/src/dispatch.rs:385` [unsafe_block] (OK): `Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) })`
- `/app/crates/contextra-simd/src/dispatch.rs:390` [unsafe_block] (OK): `Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) })`
- `/app/crates/contextra-simd/src/dispatch.rs:397` [unsafe_block] (OK): `return Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) });`
- `/app/crates/contextra-simd/src/dispatch.rs:401` [unsafe_block] (OK): `return Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) });`
- `/app/crates/contextra-simd/src/dispatch.rs:425` [unsafe_block] (OK): `Ok(unsafe { avx2::dot_product_u8_avx2(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:434` [unsafe_block] (OK): `Ok(unsafe { avx512::dot_product_u8_avx512vnni(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:444` [unsafe_block] (OK): `return Ok(unsafe { avx512::dot_product_u8_avx512vnni(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:448` [unsafe_block] (OK): `return Ok(unsafe { avx2::dot_product_u8_avx2(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:472` [unsafe_block] (OK): `Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:479` [unsafe_block] (OK): `Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:486` [unsafe_block] (OK): `return Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:490` [unsafe_block] (OK): `return Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:518` [unsafe_block] (OK): `Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:527` [unsafe_block] (OK): `Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) })`
- `/app/crates/contextra-simd/src/dispatch.rs:537` [unsafe_block] (OK): `return Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) });`
- `/app/crates/contextra-simd/src/dispatch.rs:541` [unsafe_block] (OK): `return Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) });`
- `/app/crates/contextra-simd/src/kernels/neon.rs:16` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `//! - Alle Kernel sind `unsafe fn`, da der Aufrufer sicherstellen muss, dass die NEON-Erweiterung`
- `/app/crates/contextra-simd/src/kernels/neon.rs:33` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn cosine_distance_neon(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/neon.rs:38` [unsafe_block] (OK): `let mut dot_v = unsafe { vdupq_n_f32(0.0) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:40` [unsafe_block] (OK): `let mut norm_a_v = unsafe { vdupq_n_f32(0.0) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:42` [unsafe_block] (OK): `let mut norm_b_v = unsafe { vdupq_n_f32(0.0) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:49` [unsafe_block] (OK): `let (va, vb) = unsafe { (vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i))) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:52` [unsafe_block] (OK): `dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:54` [unsafe_block] (OK): `norm_a_v = unsafe { vfmaq_f32(norm_a_v, va, va) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:56` [unsafe_block] (OK): `norm_b_v = unsafe { vfmaq_f32(norm_b_v, vb, vb) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:62` [unsafe_block] (OK): `let mut dot = unsafe { vaddvq_f32(dot_v) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:64` [unsafe_block] (OK): `let mut norm_a = unsafe { vaddvq_f32(norm_a_v) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:66` [unsafe_block] (OK): `let mut norm_b = unsafe { vaddvq_f32(norm_b_v) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:70` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:95` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn euclidean_distance_neon(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/neon.rs:100` [unsafe_block] (OK): `let mut sum_v = unsafe { vdupq_n_f32(0.0) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:107` [unsafe_block] (OK): `let (va, vb) = unsafe { (vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i))) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:109` [unsafe_block] (OK): `let diff = unsafe { vsubq_f32(va, vb) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:112` [unsafe_block] (OK): `sum_v = unsafe { vfmaq_f32(sum_v, diff, diff) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:118` [unsafe_block] (OK): `let mut sum = unsafe { vaddvq_f32(sum_v) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:122` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:140` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/neon.rs:145` [unsafe_block] (OK): `let mut dot_v = unsafe { vdupq_n_f32(0.0) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:152` [unsafe_block] (OK): `let (va, vb) = unsafe { (vld1q_f32(a.as_ptr().add(i)), vld1q_f32(b.as_ptr().add(i))) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:155` [unsafe_block] (OK): `dot_v = unsafe { vfmaq_f32(dot_v, va, vb) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:161` [unsafe_block] (OK): `let mut dot = unsafe { vaddvq_f32(dot_v) };`
- `/app/crates/contextra-simd/src/kernels/neon.rs:165` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:17` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `//! Alle Funktionen in diesem Modul sind als `pub(crate) unsafe fn` deklariert und mit`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:35` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:40` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:42` [unsafe_block] (OK): `let mut norm_a_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:44` [unsafe_block] (OK): `let mut norm_b_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:51` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:59` [unsafe_block] (OK): `dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:61` [unsafe_block] (OK): `norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:63` [unsafe_block] (OK): `norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:69` [unsafe_block] (OK): `let (mut dot, mut norm_a, mut norm_b) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:80` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:105` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:110` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:117` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:124` [unsafe_block] (OK): `let diff = unsafe { _mm256_sub_ps(va, vb) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:127` [unsafe_block] (OK): `sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:133` [unsafe_block] (OK): `let mut sum = unsafe { hsum256_ps_avx(sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:137` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:155` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:160` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:167` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:175` [unsafe_block] (OK): `dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:181` [unsafe_block] (OK): `let mut dot = unsafe { hsum256_ps_avx(dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:185` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:202` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn cosine_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:207` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:209` [unsafe_block] (OK): `let mut norm_a_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:211` [unsafe_block] (OK): `let mut norm_b_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:218` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:226` [unsafe_block] (OK): `dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:228` [unsafe_block] (OK): `norm_a_v = unsafe { _mm256_fmadd_ps(va, va, norm_a_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:230` [unsafe_block] (OK): `norm_b_v = unsafe { _mm256_fmadd_ps(vb, vb, norm_b_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:236` [unsafe_block] (OK): `let (mut dot, mut norm_a, mut norm_b) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:248` [unsafe_block] (OK): `let (x, b_val) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:283` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn euclidean_distance_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:288` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:295` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:302` [unsafe_block] (OK): `let diff = unsafe { _mm256_sub_ps(va, vb) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:305` [unsafe_block] (OK): `sum_v = unsafe { _mm256_fmadd_ps(diff, diff, sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:311` [unsafe_block] (OK): `let mut sum = unsafe { hsum256_ps_avx(sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:317` [unsafe_block] (OK): `let (x, b_val) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:345` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn dot_product_f32_bytes_avx2(a: &[f32], b_bytes: &[u8]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:350` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm256_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:357` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:365` [unsafe_block] (OK): `dot_v = unsafe { _mm256_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:371` [unsafe_block] (OK): `let mut dot = unsafe { hsum256_ps_avx(dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:377` [unsafe_block] (OK): `let (x, b_val) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:402` [unsafe_fn] (OK): `pub(crate) unsafe fn hsum256_ps_avx(v: __m256) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:404` [unsafe_block] (OK): `let vlow = unsafe { _mm256_castps256_ps128(v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:406` [unsafe_block] (OK): `let vhigh = unsafe { _mm256_extractf128_ps(v, 1) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:408` [unsafe_block] (OK): `let v128 = unsafe { _mm_add_ps(vlow, vhigh) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:410` [unsafe_block] (OK): `let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:412` [unsafe_block] (OK): `let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:414` [unsafe_block] (OK): `unsafe { _mm_cvtss_f32(v32) }`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:424` [unsafe_fn] (OK): `pub(crate) unsafe fn dot_product_u8_avx2(a: &[u8], b: &[u8]) -> u32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:429` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:431` [unsafe_block] (OK): `let zero = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:438` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:446` [unsafe_block] (OK): `let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:448` [unsafe_block] (OK): `let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:450` [unsafe_block] (OK): `let prod_lo = unsafe { _mm256_madd_epi16(va_lo, vb_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:453` [unsafe_block] (OK): `let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:455` [unsafe_block] (OK): `let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:457` [unsafe_block] (OK): `let prod_hi = unsafe { _mm256_madd_epi16(va_hi, vb_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:460` [unsafe_block] (OK): `sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:462` [unsafe_block] (OK): `sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:468` [unsafe_block] (OK): `let mut sum = u32::from_ne_bytes(unsafe { hsum256_epi32_avx2(sum_v) }.to_ne_bytes());`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:472` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i) as u32, *b.get_unchecked(i) as u32) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:487` [unsafe_fn] (OK): `pub(crate) unsafe fn euclidean_distance_sq_u8_avx2(a: &[u8], b: &[u8]) -> u32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:492` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:494` [unsafe_block] (OK): `let zero = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:501` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:509` [unsafe_block] (OK): `let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:511` [unsafe_block] (OK): `let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:513` [unsafe_block] (OK): `let diff_lo = unsafe { _mm256_sub_epi16(va_lo, vb_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:515` [unsafe_block] (OK): `let prod_lo = unsafe { _mm256_madd_epi16(diff_lo, diff_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:518` [unsafe_block] (OK): `let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:520` [unsafe_block] (OK): `let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:522` [unsafe_block] (OK): `let diff_hi = unsafe { _mm256_sub_epi16(va_hi, vb_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:524` [unsafe_block] (OK): `let prod_hi = unsafe { _mm256_madd_epi16(diff_hi, diff_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:527` [unsafe_block] (OK): `sum_v = unsafe { _mm256_add_epi32(sum_v, prod_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:529` [unsafe_block] (OK): `sum_v = unsafe { _mm256_add_epi32(sum_v, prod_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:535` [unsafe_block] (OK): `let mut sum = u32::from_ne_bytes(unsafe { hsum256_epi32_avx2(sum_v) }.to_ne_bytes());`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:539` [unsafe_block] (OK): `let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:555` [unsafe_fn] (OK): `pub(crate) unsafe fn cosine_similarity_parts_u8_avx2(`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:563` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:565` [unsafe_block] (OK): `let mut norm_a_v = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:567` [unsafe_block] (OK): `let mut norm_b_v = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:569` [unsafe_block] (OK): `let zero = unsafe { _mm256_setzero_si256() };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:576` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:584` [unsafe_block] (OK): `let va_lo = unsafe { _mm256_unpacklo_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:586` [unsafe_block] (OK): `let vb_lo = unsafe { _mm256_unpacklo_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:588` [unsafe_block] (OK): `dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_lo, vb_lo)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:590` [unsafe_block] (OK): `norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_lo, va_lo)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:592` [unsafe_block] (OK): `norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_lo, vb_lo)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:595` [unsafe_block] (OK): `let va_hi = unsafe { _mm256_unpackhi_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:597` [unsafe_block] (OK): `let vb_hi = unsafe { _mm256_unpackhi_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:599` [unsafe_block] (OK): `dot_v = unsafe { _mm256_add_epi32(dot_v, _mm256_madd_epi16(va_hi, vb_hi)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:601` [unsafe_block] (OK): `norm_a_v = unsafe { _mm256_add_epi32(norm_a_v, _mm256_madd_epi16(va_hi, va_hi)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:603` [unsafe_block] (OK): `norm_b_v = unsafe { _mm256_add_epi32(norm_b_v, _mm256_madd_epi16(vb_hi, vb_hi)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:609` [unsafe_block] (OK): `let (mut dot, mut norm_a_sq, mut norm_b_sq) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:619` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i) as u32, *b.get_unchecked(i) as u32) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:640` [unsafe_fn] (OK): `pub(crate) unsafe fn hsum256_epi32_avx2(v: __m256i) -> i32 {`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:642` [unsafe_block] (OK): `let vlow = unsafe { _mm256_castsi256_si128(v) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:644` [unsafe_block] (OK): `let vhigh = unsafe { _mm256_extracti128_si256(v, 1) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:646` [unsafe_block] (OK): `let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:648` [unsafe_block] (OK): `let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:650` [unsafe_block] (OK): `let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };`
- `/app/crates/contextra-simd/src/kernels/avx2.rs:652` [unsafe_block] (OK): `unsafe { _mm_cvtsi128_si32(v32) }`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:17` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `//! Alle Funktionen in diesem Modul sind als `pub(crate) unsafe fn` deklariert und erfordern`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:35` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn cosine_distance_avx512(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:40` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:42` [unsafe_block] (OK): `let mut norm_a_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:44` [unsafe_block] (OK): `let mut norm_b_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:51` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:59` [unsafe_block] (OK): `dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:61` [unsafe_block] (OK): `norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:63` [unsafe_block] (OK): `norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:69` [unsafe_block] (OK): `let (mut dot, mut norm_a, mut norm_b) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:79` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:104` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn euclidean_distance_avx512(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:109` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:116` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:123` [unsafe_block] (OK): `let diff = unsafe { _mm512_sub_ps(va, vb) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:126` [unsafe_block] (OK): `sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:132` [unsafe_block] (OK): `let mut sum = unsafe { hsum512_ps_avx(sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:136` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:154` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn dot_product_avx512(a: &[f32], b: &[f32]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:159` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:166` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:174` [unsafe_block] (OK): `dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:180` [unsafe_block] (OK): `let mut dot = unsafe { hsum512_ps_avx(dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:184` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i), *b.get_unchecked(i)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:201` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn cosine_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:206` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:208` [unsafe_block] (OK): `let mut norm_a_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:210` [unsafe_block] (OK): `let mut norm_b_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:217` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:225` [unsafe_block] (OK): `dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:227` [unsafe_block] (OK): `norm_a_v = unsafe { _mm512_fmadd_ps(va, va, norm_a_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:229` [unsafe_block] (OK): `norm_b_v = unsafe { _mm512_fmadd_ps(vb, vb, norm_b_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:235` [unsafe_block] (OK): `let (mut dot, mut norm_a, mut norm_b) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:247` [unsafe_block] (OK): `let (x, b_val) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:282` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn euclidean_distance_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:287` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:294` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:301` [unsafe_block] (OK): `let diff = unsafe { _mm512_sub_ps(va, vb) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:304` [unsafe_block] (OK): `sum_v = unsafe { _mm512_fmadd_ps(diff, diff, sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:310` [unsafe_block] (OK): `let mut sum = unsafe { hsum512_ps_avx(sum_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:316` [unsafe_block] (OK): `let (x, b_val) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:344` [unsafe_fn] (**KEIN SAFETY KOMMENTAR**): `pub(crate) unsafe fn dot_product_f32_bytes_avx512(a: &[f32], b_bytes: &[u8]) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:349` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm512_setzero_ps() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:356` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:364` [unsafe_block] (OK): `dot_v = unsafe { _mm512_fmadd_ps(va, vb, dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:370` [unsafe_block] (OK): `let mut dot = unsafe { hsum512_ps_avx(dot_v) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:376` [unsafe_block] (OK): `let (x, b_val) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:401` [unsafe_fn] (OK): `pub(crate) unsafe fn hsum512_ps_avx(v: __m512) -> f32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:403` [unsafe_block] (OK): `let v256 = unsafe { _mm256_add_ps(_mm512_extractf32x8_ps(v, 0), _mm512_extractf32x8_ps(v, 1)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:405` [unsafe_block] (OK): `let vlow = unsafe { _mm256_castps256_ps128(v256) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:407` [unsafe_block] (OK): `let vhigh = unsafe { _mm256_extractf128_ps(v256, 1) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:409` [unsafe_block] (OK): `let v128 = unsafe { _mm_add_ps(vlow, vhigh) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:411` [unsafe_block] (OK): `let v64 = unsafe { _mm_add_ps(v128, _mm_movehl_ps(v128, v128)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:413` [unsafe_block] (OK): `let v32 = unsafe { _mm_add_ss(v64, _mm_shuffle_ps(v64, v64, 0x55)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:415` [unsafe_block] (OK): `unsafe { _mm_cvtss_f32(v32) }`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:425` [unsafe_fn] (OK): `pub(crate) unsafe fn dot_product_u8_avx512vnni(a: &[u8], b: &[u8]) -> u32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:430` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm512_setzero_si512() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:437` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:445` [unsafe_block] (OK): `sum_v = unsafe { _mm512_dpbusd_epi32(sum_v, va, vb) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:451` [unsafe_block] (OK): `let mut sum = u32::from_ne_bytes(unsafe { hsum512_epi32_avx512(sum_v) }.to_ne_bytes());`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:455` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i) as u32, *b.get_unchecked(i) as u32) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:470` [unsafe_fn] (OK): `pub(crate) unsafe fn euclidean_distance_sq_u8_avx512(a: &[u8], b: &[u8]) -> u32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:475` [unsafe_block] (OK): `let mut sum_v = unsafe { _mm512_setzero_si512() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:477` [unsafe_block] (OK): `let zero = unsafe { _mm512_setzero_si512() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:484` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:492` [unsafe_block] (OK): `let va_lo = unsafe { _mm512_unpacklo_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:494` [unsafe_block] (OK): `let vb_lo = unsafe { _mm512_unpacklo_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:496` [unsafe_block] (OK): `let diff_lo = unsafe { _mm512_sub_epi16(va_lo, vb_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:498` [unsafe_block] (OK): `let prod_lo = unsafe { _mm512_madd_epi16(diff_lo, diff_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:501` [unsafe_block] (OK): `let va_hi = unsafe { _mm512_unpackhi_epi8(va, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:503` [unsafe_block] (OK): `let vb_hi = unsafe { _mm512_unpackhi_epi8(vb, zero) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:505` [unsafe_block] (OK): `let diff_hi = unsafe { _mm512_sub_epi16(va_hi, vb_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:507` [unsafe_block] (OK): `let prod_hi = unsafe { _mm512_madd_epi16(diff_hi, diff_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:510` [unsafe_block] (OK): `sum_v = unsafe { _mm512_add_epi32(sum_v, prod_lo) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:512` [unsafe_block] (OK): `sum_v = unsafe { _mm512_add_epi32(sum_v, prod_hi) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:518` [unsafe_block] (OK): `let mut sum = u32::from_ne_bytes(unsafe { hsum512_epi32_avx512(sum_v) }.to_ne_bytes());`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:522` [unsafe_block] (OK): `let diff = unsafe { (*a.get_unchecked(i) as i32) - (*b.get_unchecked(i) as i32) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:538` [unsafe_fn] (OK): `pub(crate) unsafe fn cosine_similarity_parts_u8_avx512(`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:546` [unsafe_block] (OK): `let mut dot_v = unsafe { _mm512_setzero_si512() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:548` [unsafe_block] (OK): `let mut norm_a_v = unsafe { _mm512_setzero_si512() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:550` [unsafe_block] (OK): `let mut norm_b_v = unsafe { _mm512_setzero_si512() };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:557` [unsafe_block] (OK): `let (va, vb) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:565` [unsafe_block] (OK): `dot_v = unsafe { _mm512_dpbusd_epi32(dot_v, va, vb) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:567` [unsafe_block] (OK): `norm_a_v = unsafe { _mm512_dpbusd_epi32(norm_a_v, va, va) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:569` [unsafe_block] (OK): `norm_b_v = unsafe { _mm512_dpbusd_epi32(norm_b_v, vb, vb) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:575` [unsafe_block] (OK): `let (mut dot, mut norm_a_sq, mut norm_b_sq) = unsafe {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:585` [unsafe_block] (OK): `let (x, y) = unsafe { (*a.get_unchecked(i) as u32, *b.get_unchecked(i) as u32) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:606` [unsafe_fn] (OK): `pub(crate) unsafe fn hsum512_epi32_avx512(v: __m512i) -> i32 {`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:609` [unsafe_block] (OK): `unsafe { _mm256_add_epi32(_mm512_castsi512_si256(v), _mm512_extracti32x8_epi32(v, 1)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:611` [unsafe_block] (OK): `let vlow = unsafe { _mm256_castsi256_si128(v256) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:613` [unsafe_block] (OK): `let vhigh = unsafe { _mm256_extracti128_si256(v256, 1) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:615` [unsafe_block] (OK): `let v128 = unsafe { _mm_add_epi32(vlow, vhigh) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:617` [unsafe_block] (OK): `let v64 = unsafe { _mm_add_epi32(v128, _mm_srli_si128(v128, 8)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:619` [unsafe_block] (OK): `let v32 = unsafe { _mm_add_epi32(v64, _mm_srli_si128(v64, 4)) };`
- `/app/crates/contextra-simd/src/kernels/avx512.rs:621` [unsafe_block] (OK): `unsafe { _mm_cvtsi128_si32(v32) }`
### `contextra-text`

- `/app/crates/contextra-text/tests/alloc_profiler.rs:27` [unsafe_impl] (**KEIN SAFETY KOMMENTAR**): `unsafe impl GlobalAlloc for CountingAllocator {`
- `/app/crates/contextra-text/tests/alloc_profiler.rs:31` [unsafe_fn] (OK): `unsafe fn alloc(&self, layout: Layout) -> *mut u8 {`
- `/app/crates/contextra-text/tests/alloc_profiler.rs:37` [unsafe_block] (OK): `let ptr = unsafe { System.alloc(layout) }; // SAFETY: Forward layout to System allocator.`
- `/app/crates/contextra-text/tests/alloc_profiler.rs:43` [unsafe_fn] (OK): `unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {`
- `/app/crates/contextra-text/tests/alloc_profiler.rs:48` [unsafe_block] (OK): `unsafe {`
### `contextra-wire`

- `/app/crates/contextra-wire/build.rs:59` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `/// Kapselt Aufrufe von `self._tab.get` in `unsafe { ... }`, falls sie nicht bereits gewrappt sind.`
- `/app/crates/contextra-wire/build.rs:67` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `/// 3. Voranstellen von `unsafe { ` und Anhängen von ` }`.`
- `/app/crates/contextra-wire/build.rs:82` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `if prefix.trim_end().ends_with("unsafe {") || prefix.contains("unsafe") {`
- `/app/crates/contextra-wire/build.rs:110` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `result.push_str("unsafe { ");`
- `/app/crates/contextra-wire/build.rs:168` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `let expected1 = "    unsafe { self._tab.get::<u32>(RoleId::VT_ID, Some(0)) }.unwrap()";`
- `/app/crates/contextra-wire/build.rs:173` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `let expected2 = "    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ScoredDocument::VT_ID, None) }";`
- `/app/crates/contextra-wire/build.rs:176` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `let input3 = "    unsafe { self._tab.get::<i8>(Embedding::VT_METRIC, Some(0)).unwrap()}";`
- `/app/crates/contextra-wire/src/contextra_generated.rs:59` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:81` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:110` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:126` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:142` [unsafe_block] (OK): `unsafe { self._tab.get::<i8>(Embedding::VT_METRIC, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:263` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:288` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:320` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:333` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:347` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:362` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:500` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:522` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:552` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:566` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:580` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:709` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:731` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:762` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:775` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:790` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:917` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:933` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:956` [unsafe_block] (OK): `unsafe { self._tab.get::<u32>(RoleId::VT_ID, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1038` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1054` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1077` [unsafe_block] (OK): `unsafe { self._tab.get::<u64>(HyperEdgeId::VT_ID, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1161` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1180` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1204` [unsafe_block] (OK): `unsafe { self._tab.get::<u32>(RoleBinding::VT_ROLE, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1214` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1312` [unsafe_fn] (OK): `unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1355` [unsafe_fn] (OK): `pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1391` [unsafe_block] (OK): `unsafe { self._tab.get::<u64>(HyperEdge::VT_ID, Some(0)).unwrap() }`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1401` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1418` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1432` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1446` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1460` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1474` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1488` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1502` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1516` [unsafe_block] (OK): `unsafe {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1783` [unsafe_fn] (OK): `pub unsafe fn root_as_search_response_unchecked(buf: &[u8]) -> SearchResponse {`
- `/app/crates/contextra-wire/src/contextra_generated.rs:1793` [unsafe_fn] (OK): `pub unsafe fn size_prefixed_root_as_search_response_unchecked(`
- `/app/crates/contextra-wire/tests/schema_drift.rs:191` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `if prefix.trim_end().ends_with("unsafe {") || prefix.contains("unsafe") {`
- `/app/crates/contextra-wire/tests/schema_drift.rs:216` [unsafe_block] (**KEIN SAFETY KOMMENTAR**): `result.push_str("unsafe { ");`
