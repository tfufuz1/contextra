// FILE-CONTEXT
// ZWECK: Skalare Fallback-Implementierung der Distanzmetriken (f32, f32_bytes, u8, f32_u8).
// INVARIANTEN: Mathematisch exakt und deterministisch; Null-Division-Schutz für Cosine.

/// Struct holding raw integer sum components for u8 cosine similarity calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CosineSimilarityPartsU8 {
    /// Accumulated dot product $\sum a_i b_i$.
    pub dot: u32,
    /// Accumulated squared L2 norm $\sum a_i^2$.
    pub norm_a_sq: u32,
    /// Accumulated squared L2 norm $\sum b_i^2$.
    pub norm_b_sq: u32,
}

/// Struct holding raw float sum components for f32/u8 mixed cosine similarity calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CosineSimilarityPartsF32U8 {
    /// Accumulated dot product $\sum a_i (b_i \text{ as } f32)$.
    pub dot: f32,
    /// Accumulated squared L2 norm $\sum a_i^2$.
    pub norm_a_sq: f32,
    /// Accumulated squared L2 norm $\sum (b_i \text{ as } f32)^2$.
    pub norm_b_sq: f32,
}

/// Scalar reference implementation for cosine distance on `f32` slices.
///
/// # Semantics
///
/// Calculates $1.0 - \text{cosine\_similarity}(a, b)$. Cosine similarity is clamped to $[-1.0, 1.0]$.
/// Zero-norm vectors safely return `1.0`.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Scalar reference implementation for Euclidean distance on `f32` slices.
///
/// # Semantics
///
/// Calculates $\sqrt{\sum (a_i - b_i)^2}$.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum.sqrt()
}

/// Scalar reference implementation for dot product distance on `f32` slices.
///
/// # Semantics
///
/// Calculates negated dot product $- \sum a_i b_i$.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
    }
    -dot
}

/// Scalar reference implementation for cosine distance between `f32` slice and raw LE `f32` byte slice.
///
/// # Semantics
///
/// Decodes `f32` values from `b_bytes` via little-endian bytes and computes cosine distance against `a`.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
#[inline]
#[allow(clippy::indexing_slicing)]
pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (i, &x) in a.iter().enumerate() {
        if let Some(chunk) = b_bytes.get(i * 4..i * 4 + 4) {
            if let Ok(bytes) = chunk.try_into() {
                let b_val = f32::from_le_bytes(bytes);
                dot += x * b_val;
                norm_a += x * x;
                norm_b += b_val * b_val;
            }
        }
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Scalar reference implementation for Euclidean distance between `f32` slice and raw LE `f32` byte slice.
///
/// # Semantics
///
/// Decodes `f32` values from `b_bytes` via little-endian bytes and computes Euclidean distance against `a`.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
#[inline]
#[allow(clippy::indexing_slicing)]
pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut sum = 0.0f32;
    for (i, &x) in a.iter().enumerate() {
        if let Some(chunk) = b_bytes.get(i * 4..i * 4 + 4) {
            if let Ok(bytes) = chunk.try_into() {
                let b_val = f32::from_le_bytes(bytes);
                let diff = x - b_val;
                sum += diff * diff;
            }
        }
    }
    sum.sqrt()
}

/// Scalar reference implementation for dot product distance between `f32` slice and raw LE `f32` byte slice.
///
/// # Semantics
///
/// Decodes `f32` values from `b_bytes` via little-endian bytes and computes negated dot product against `a`.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
#[inline]
#[allow(clippy::indexing_slicing)]
pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut dot = 0.0f32;
    for (i, &x) in a.iter().enumerate() {
        if let Some(chunk) = b_bytes.get(i * 4..i * 4 + 4) {
            if let Ok(bytes) = chunk.try_into() {
                let b_val = f32::from_le_bytes(bytes);
                dot += x * b_val;
            }
        }
    }
    -dot
}

/// Scalar reference implementation for unsigned integer dot product between two `u8` byte slices.
///
/// # Semantics
///
/// Calculates $\sum a_i b_i$ with 32-bit unsigned accumulation.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32 {
    let mut dot = 0u32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += (x as u32) * (y as u32);
    }
    dot
}

/// Scalar reference implementation for squared Euclidean distance between two `u8` byte slices.
///
/// # Semantics
///
/// Calculates $\sum (a_i - b_i)^2$ with 32-bit unsigned accumulation.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32 {
    let mut sum = 0u32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let diff = (x as i32) - (y as i32);
        let abs_diff = diff.unsigned_abs();
        sum += abs_diff * abs_diff;
    }
    sum
}

/// Scalar reference implementation for `u8` cosine similarity component parts.
///
/// # Semantics
///
/// Returns [`CosineSimilarityPartsU8`] containing dot product and squared norms.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let mut dot = 0u32;
    let mut norm_a_sq = 0u32;
    let mut norm_b_sq = 0u32;

    for (&x_u8, &y_u8) in a.iter().zip(b.iter()) {
        let x = x_u8 as u32;
        let y = y_u8 as u32;
        dot += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    CosineSimilarityPartsU8 {
        dot,
        norm_a_sq,
        norm_b_sq,
    }
}

/// Scalar reference implementation for mixed `f32`/`u8` dot product.
///
/// # Semantics
///
/// Calculates $\sum a_i (b_i \text{ as } f32)$.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32 {
    let mut dot = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * (y as f32);
    }
    dot
}

/// Scalar reference implementation for mixed `f32`/`u8` scalar dequantized squared Euclidean distance.
///
/// # Semantics
///
/// Calculates $\sum (a_i - (b_i \cdot \alpha_i + \min_i))^2$.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len(), \alpha.len(), \min.len())$.
#[inline]
pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for (((&x, &y_u8), &alpha), &min_val) in
        a.iter().zip(b.iter()).zip(alphas.iter()).zip(mins.iter())
    {
        let y_f32 = (y_u8 as f32) * alpha + min_val;
        let diff = x - y_f32;
        sum += diff * diff;
    }
    sum
}

/// Scalar reference implementation for mixed `f32`/`u8` cosine similarity component parts.
///
/// # Semantics
///
/// Returns [`CosineSimilarityPartsF32U8`] containing dot product and squared norms.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = \min(a.len(), b.len())$.
#[inline]
pub fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8 {
    let mut dot = 0.0f32;
    let mut norm_a_sq = 0.0f32;
    let mut norm_b_sq = 0.0f32;

    for (&x, &y_u8) in a.iter().zip(b.iter()) {
        let y = y_u8 as f32;
        dot += x * y;
        norm_a_sq += x * x;
        norm_b_sq += y * y;
    }

    CosineSimilarityPartsF32U8 {
        dot,
        norm_a_sq,
        norm_b_sq,
    }
}

/// Normalizes a vector in-place to unit L2 length.
///
/// # Semantics
///
/// Divides each element by $\sqrt{\sum v_i^2}$. Zero-norm vectors remain unmodified.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = v.len()$.
pub fn normalize_inplace(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        let inv_norm = 1.0 / norm;
        for x in v.iter_mut() {
            *x *= inv_norm;
        }
    }
}
