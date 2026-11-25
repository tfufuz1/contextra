// FILE-CONTEXT
// ZWECK: Skalare Fallback-Implementierung der Distanzmetriken (f32, f32_bytes, u8, f32_u8).
// INVARIANTEN: Mathematisch exakt und deterministisch; Null-Division-Schutz für Cosine.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CosineSimilarityPartsU8 {
    pub dot: u32,
    pub norm_a_sq: u32,
    pub norm_b_sq: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CosineSimilarityPartsF32U8 {
    pub dot: f32,
    pub norm_a_sq: f32,
    pub norm_b_sq: f32,
}

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

#[inline]
pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        let diff = x - y;
        sum += diff * diff;
    }
    sum.sqrt()
}

#[inline]
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
    }
    -dot
}

#[inline]
pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (i, &x) in a.iter().enumerate() {
        let b_val = f32::from_le_bytes([
            b_bytes[i * 4],
            b_bytes[i * 4 + 1],
            b_bytes[i * 4 + 2],
            b_bytes[i * 4 + 3],
        ]);
        dot += x * b_val;
        norm_a += x * x;
        norm_b += b_val * b_val;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

#[inline]
pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut sum = 0.0f32;
    for (i, &x) in a.iter().enumerate() {
        let b_val = f32::from_le_bytes([
            b_bytes[i * 4],
            b_bytes[i * 4 + 1],
            b_bytes[i * 4 + 2],
            b_bytes[i * 4 + 3],
        ]);
        let diff = x - b_val;
        sum += diff * diff;
    }
    sum.sqrt()
}

#[inline]
pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut dot = 0.0f32;
    for (i, &x) in a.iter().enumerate() {
        let b_val = f32::from_le_bytes([
            b_bytes[i * 4],
            b_bytes[i * 4 + 1],
            b_bytes[i * 4 + 2],
            b_bytes[i * 4 + 3],
        ]);
        dot += x * b_val;
    }
    -dot
}

#[inline]
pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32 {
    let min_len = a.len().min(b.len());
    let mut dot = 0u32;
    for i in 0..min_len {
        dot += (a[i] as u32) * (b[i] as u32);
    }
    dot
}

#[inline]
pub fn euclidean_distance_sq_u8_scalar(a: &[u8], b: &[u8]) -> u32 {
    let min_len = a.len().min(b.len());
    let mut sum = 0u32;
    for i in 0..min_len {
        let diff = (a[i] as i32) - (b[i] as i32);
        sum += (diff * diff) as u32;
    }
    sum
}

#[inline]
pub fn cosine_similarity_parts_u8_scalar(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    let min_len = a.len().min(b.len());
    let mut dot = 0u32;
    let mut norm_a_sq = 0u32;
    let mut norm_b_sq = 0u32;

    for i in 0..min_len {
        let x = a[i] as u32;
        let y = b[i] as u32;
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

#[inline]
pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32 {
    let min_len = a.len().min(b.len());
    let mut dot = 0.0f32;
    for i in 0..min_len {
        dot += a[i] * (b[i] as f32);
    }
    dot
}

#[inline]
pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32 {
    let min_len = a.len().min(b.len()).min(alphas.len()).min(mins.len());
    let mut sum = 0.0f32;
    for i in 0..min_len {
        let y_f32 = (b[i] as f32) * alphas[i] + mins[i];
        let diff = a[i] - y_f32;
        sum += diff * diff;
    }
    sum
}

#[inline]
pub fn cosine_similarity_parts_f32_u8(a: &[f32], b: &[u8]) -> CosineSimilarityPartsF32U8 {
    let min_len = a.len().min(b.len());
    let mut dot = 0.0f32;
    let mut norm_a_sq = 0.0f32;
    let mut norm_b_sq = 0.0f32;

    for i in 0..min_len {
        let x = a[i];
        let y = b[i] as f32;
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

pub fn normalize_inplace(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        let inv_norm = 1.0 / norm;
        for x in v.iter_mut() {
            *x *= inv_norm;
        }
    }
}
