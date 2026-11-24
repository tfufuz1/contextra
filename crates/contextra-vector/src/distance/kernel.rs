// FILE-CONTEXT
// ZWECK: Crate-lokale Distanz-Kernel-Schnittstelle und u8-Komponenten (WP-VEC-06).
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); INV-SIMD-FINITE-1.
// HOTSPOTS: DistanceKernel, CosineSimilarityPartsU8
// STAND: 2026-11-12

//! # Distance Kernel Interface
//!
//! Definiert den Trait [`DistanceKernel`] und Hilfstypen für u8-Akkumulatoren.

use crate::error::Result;
use crate::types::DistanceMetric;

// SECTION Datentypen für SIMD- und Quantisierungs-Distanz

// ANCHOR[contextra-vector:DIST-PARTS-U8]
/// Rohkomponenten zur Berechnung der ganzzahligen Kosinus-Ähnlichkeit auf u8-Vektoren.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CosineSimilarityPartsU8 {
    /// Akkumuliertes Skalarprodukt $\sum a_i b_i$.
    pub dot: u32,
    /// Akkumulierte quadratische L2-Norm $\sum a_i^2$.
    pub norm_a_sq: u32,
    /// Akkumulierte quadratische L2-Norm $\sum b_i^2$.
    pub norm_b_sq: u32,
}

// ANCHOR[contextra-vector:DIST-KERNEL-TRAIT]
/// Schnittstelle für Distanzberechnungskerne auf f32- und quantisierten Vektoren.
pub trait DistanceKernel: Send + Sync + 'static {
    /// Berechnet die Distanz zwischen zwei Slices nach Endlichkeitsvalidierung.
    fn distance(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Berechnet die Distanz zwischen einer f32-Query und einem Byte-Slice (LE f32).
    fn distance_f32_bytes(&self, metric: DistanceMetric, a: &[f32], b_bytes: &[u8]) -> Result<f32>;

    /// Berechnet die Distanz auf vorvalidierten f32-Slices.
    fn distance_trusted(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Berechnet die Distanz zwischen vorvalidiertem f32-Slice und LE f32 Byte-Slice.
    fn distance_f32_bytes_trusted(
        &self,
        metric: DistanceMetric,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32>;

    /// Kosinus-Distanz zwischen zwei f32-Slices.
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Skalarprodukt-Distanz (invertiertes Skalarprodukt) zwischen zwei f32-Slices.
    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Euklidische Distanz zwischen zwei f32-Slices.
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32>;

    /// Quadratische Euklidische Distanz zwischen zwei u8-Slices.
    fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32;

    /// Gemischte f32/u8 quadratische Euklidische Distanz mit Rekonstruktionsparametern.
    fn euclidean_distance_sq_f32_u8(
        &self,
        a: &[f32],
        b: &[u8],
        alphas: &[f32],
        mins: &[f32],
    ) -> f32;

    /// Ganzzahliges Skalarprodukt zwischen zwei u8-Byte-Slices.
    fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32;

    /// Gemischtes f32/u8 Skalarprodukt.
    fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32;

    /// Rohkomponenten für u8 Kosinus-Ähnlichkeit.
    fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8;

    /// Normalisiert einen Vektor in-place auf Einheitslänge.
    fn normalize_inplace(&self, v: &mut [f32]);
}
// !SECTION
