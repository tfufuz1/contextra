// FILE-CONTEXT
// ZWECK: Integer-Kernel für quantisierte Operationen (u8 Distanzen und Skalarprodukte).
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); Keine Panics bei ungleichen Längen.
// HOTSPOTS: euclidean_distance_sq_u8, dot_product_u8, cosine_similarity_parts_u8
// STAND: 2026-11-12

//! # Quantized Integer Ops Module
//!
//! Freistehende Operationen auf quantisierten u8-Vektoren.

use super::kernel::CosineSimilarityPartsU8;
use super::scalar::ScalarKernel;

/// Quadratische Euklidische Distanz zwischen zwei u8-Vektoren.
#[inline]
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> u32 {
    ScalarKernel.euclidean_distance_sq_u8(a, b)
}

/// Gemischte quadratische Euklidische Distanz zwischen f32-Query und quantisiertem u8-Vektor.
#[inline]
pub fn euclidean_distance_sq_f32_u8(a: &[f32], b: &[u8], alphas: &[f32], mins: &[f32]) -> f32 {
    ScalarKernel.euclidean_distance_sq_f32_u8(a, b, alphas, mins)
}

/// Ganzzahliges Skalarprodukt zwischen zwei u8-Vektoren.
#[inline]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> u32 {
    ScalarKernel.dot_product_u8(a, b)
}

/// Gemischtes Skalarprodukt zwischen f32-Query und quantisiertem u8-Vektor.
#[inline]
pub fn dot_product_f32_u8(a: &[f32], b: &[u8]) -> f32 {
    ScalarKernel.dot_product_f32_u8(a, b)
}

/// Berechnet die ganzzahligen Summen-Komponenten für Kosinus-Ähnlichkeit auf u8-Vektoren.
#[inline]
pub fn cosine_similarity_parts_u8(a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
    ScalarKernel.cosine_similarity_parts_u8(a, b)
}

/// In-place Normalisierung eines Vektors auf Einheitslänge.
#[inline]
pub fn normalize_inplace(v: &mut [f32]) {
    ScalarKernel.normalize_inplace(v);
}
