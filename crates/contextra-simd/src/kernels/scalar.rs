// FILE-CONTEXT
// ZWECK: Skalare kanonische Referenz-Implementierung der Distanzmetriken (f32, f32_bytes).
// INVARIANTEN: Kanonische Distanz nach SPEC D-122 (5b): 16 Teilsummen (Lane = Index mod 16),
//             kein FMA, feste paarweise Reduktion 16 -> 8 -> 4 -> 2 -> 1. Alle Kernel sind bitgleich.

//! # Modul `kernels::scalar`
//!
//! **Ring-Zugehörigkeit:** Ring 1.
//!
//! **Zweck:**
//! Plattformunabhängige kanonische Referenzimplementierung aller Distanzmetriken in reinem, sicherem Rust.

/// Feste paarweise Reduktion von 16 Teilsummen nach SPEC D-122 (5b).
#[inline(always)]
pub fn canonical_reduce_16(lanes: &[f32; 16]) -> f32 {
    let [l0, l1, l2, l3, l4, l5, l6, l7, l8, l9, l10, l11, l12, l13, l14, l15] = *lanes;
    let r8_0 = l0 + l1;
    let r8_1 = l2 + l3;
    let r8_2 = l4 + l5;
    let r8_3 = l6 + l7;
    let r8_4 = l8 + l9;
    let r8_5 = l10 + l11;
    let r8_6 = l12 + l13;
    let r8_7 = l14 + l15;

    let r4_0 = r8_0 + r8_1;
    let r4_1 = r8_2 + r8_3;
    let r4_2 = r8_4 + r8_5;
    let r4_3 = r8_6 + r8_7;

    let r2_0 = r4_0 + r4_1;
    let r2_1 = r4_2 + r4_3;

    r2_0 + r2_1
}

/// Skalare kanonische Kosinus-Distanz auf `f32`-Slices.
#[inline]
pub fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_lanes = [0.0f32; 16];
    let mut norm_a_lanes = [0.0f32; 16];
    let mut norm_b_lanes = [0.0f32; 16];

    for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
        let lane = i % 16;
        if let Some(slot) = dot_lanes.get_mut(lane) {
            *slot += x * y;
        }
        if let Some(slot) = norm_a_lanes.get_mut(lane) {
            *slot += x * x;
        }
        if let Some(slot) = norm_b_lanes.get_mut(lane) {
            *slot += y * y;
        }
    }

    let dot = canonical_reduce_16(&dot_lanes);
    let norm_a = canonical_reduce_16(&norm_a_lanes);
    let norm_b = canonical_reduce_16(&norm_b_lanes);

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Skalare kanonische Euklidische Distanz auf `f32`-Slices.
#[inline]
pub fn euclidean_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut sum_lanes = [0.0f32; 16];

    for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
        let lane = i % 16;
        let diff = x - y;
        if let Some(slot) = sum_lanes.get_mut(lane) {
            *slot += diff * diff;
        }
    }

    let sum = canonical_reduce_16(&sum_lanes);
    sum.sqrt()
}

/// Skalare kanonische Skalarprodukt-Distanz (negiertes Dot Product) auf `f32`-Slices.
#[inline]
pub fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    let mut dot_lanes = [0.0f32; 16];

    for (i, (&x, &y)) in a.iter().zip(b.iter()).enumerate() {
        let lane = i % 16;
        if let Some(slot) = dot_lanes.get_mut(lane) {
            *slot += x * y;
        }
    }

    let dot = canonical_reduce_16(&dot_lanes);
    -dot
}

/// Skalare kanonische Kosinus-Distanz zwischen `f32`-Slice und LE `f32`-Byte-Slice.
#[inline]
pub fn cosine_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut dot_lanes = [0.0f32; 16];
    let mut norm_a_lanes = [0.0f32; 16];
    let mut norm_b_lanes = [0.0f32; 16];

    for (i, &x) in a.iter().enumerate() {
        let offset = i * 4;
        let Some(chunk) = b_bytes.get(offset..offset + 4) else {
            break;
        };
        let Ok(bytes) = chunk.try_into() else {
            break;
        };
        let b_val = f32::from_le_bytes(bytes);
        let lane = i % 16;
        if let Some(slot) = dot_lanes.get_mut(lane) {
            *slot += x * b_val;
        }
        if let Some(slot) = norm_a_lanes.get_mut(lane) {
            *slot += x * x;
        }
        if let Some(slot) = norm_b_lanes.get_mut(lane) {
            *slot += b_val * b_val;
        }
    }

    let dot = canonical_reduce_16(&dot_lanes);
    let norm_a = canonical_reduce_16(&norm_a_lanes);
    let norm_b = canonical_reduce_16(&norm_b_lanes);

    if norm_a == 0.0 || norm_b == 0.0 {
        return 1.0;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    let sim = (dot / denom).clamp(-1.0, 1.0);
    1.0 - sim
}

/// Skalare kanonische Euklidische Distanz zwischen `f32`-Slice und LE `f32`-Byte-Slice.
#[inline]
pub fn euclidean_distance_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut sum_lanes = [0.0f32; 16];

    for (i, &x) in a.iter().enumerate() {
        let offset = i * 4;
        let Some(chunk) = b_bytes.get(offset..offset + 4) else {
            break;
        };
        let Ok(bytes) = chunk.try_into() else {
            break;
        };
        let b_val = f32::from_le_bytes(bytes);
        let lane = i % 16;
        let diff = x - b_val;
        if let Some(slot) = sum_lanes.get_mut(lane) {
            *slot += diff * diff;
        }
    }

    let sum = canonical_reduce_16(&sum_lanes);
    sum.sqrt()
}

/// Skalare kanonische Skalarprodukt-Distanz zwischen `f32`-Slice und LE `f32`-Byte-Slice.
#[inline]
pub fn dot_product_f32_bytes_scalar(a: &[f32], b_bytes: &[u8]) -> f32 {
    let mut dot_lanes = [0.0f32; 16];

    for (i, &x) in a.iter().enumerate() {
        let offset = i * 4;
        let Some(chunk) = b_bytes.get(offset..offset + 4) else {
            break;
        };
        let Ok(bytes) = chunk.try_into() else {
            break;
        };
        let b_val = f32::from_le_bytes(bytes);
        let lane = i % 16;
        if let Some(slot) = dot_lanes.get_mut(lane) {
            *slot += x * b_val;
        }
    }

    let dot = canonical_reduce_16(&dot_lanes);
    -dot
}

/// Normalisiert einen `f32`-Vektor in-place auf Einheitslänge.
pub fn normalize_inplace(v: &mut [f32]) {
    let mut norm_lanes = [0.0f32; 16];
    for (i, &x) in v.iter().enumerate() {
        let lane = i % 16;
        if let Some(slot) = norm_lanes.get_mut(lane) {
            *slot += x * x;
        }
    }
    let norm_sq = canonical_reduce_16(&norm_lanes);
    if norm_sq > 0.0 {
        let inv_norm = 1.0 / norm_sq.sqrt();
        for x in v.iter_mut() {
            *x *= inv_norm;
        }
    }
}

/// Komponenten für ganzzahlige Kosinus-Ähnlichkeit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CosineSimilarityPartsU8 {
    pub dot: u32,
    pub norm_a_sq: u32,
    pub norm_b_sq: u32,
}

/// Skalare Referenz für ganzzahliges Skalarprodukt.
#[inline]
pub fn dot_product_u8_scalar(a: &[u8], b: &[u8]) -> u32 {
    let mut dot = 0u32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += (x as u32) * (y as u32);
    }
    dot
}

/// Skalare Referenz für quadrierte euklidische Distanz auf u8.
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

/// Skalare Referenz für Kosinus-Komponenten auf u8.
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

/// Kompatibilitäts-Funktion für gemischte f32/u8 Skalarprodukte.
#[inline]
pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32 {
    let mut dot = 0.0f32;
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * (y as f32);
    }
    dot
}

/// Kompatibilitäts-Funktion für dequantisierte quadrierte euklidische Distanz.
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
