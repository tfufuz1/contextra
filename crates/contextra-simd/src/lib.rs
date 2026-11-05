// FILE-CONTEXT
// ZWECK: Ring-1 Unsafe-Insel für SIMD-Distanzkernel und Laufzeit-Dispatch.
// INVARIANTEN: Safe Public API, zero panic, bytemuck/slice alignments.
// HOTSPOTS: lib.rs, kernels/, dispatch.rs

//! # Crate `contextra-simd`
//! Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz (Ring 1).
//!
//!

#![allow(unsafe_code)]
#![allow(unused_unsafe)]

// Dynamische CPU-Erkennung und Funktionszeiger-Auswahl.
pub mod dispatch;
// Hardwarespezifische Rechenkerne (Scalar, AVX2, AVX-512, NEON).
pub mod kernels;

pub use dispatch::*;
pub use kernels::scalar::{
    canonical_reduce_16, cosine_distance_f32_bytes_scalar, cosine_distance_scalar,
    cosine_similarity_parts_u8_scalar, dot_product_f32_bytes_scalar, dot_product_scalar,
    dot_product_u8_scalar, euclidean_distance_f32_bytes_scalar, euclidean_distance_scalar,
    euclidean_distance_sq_u8_scalar, normalize_inplace, CosineSimilarityPartsU8,
};

use contextra_types::{ContextraError, DistanceMetric};

pub const MAX_VECTOR_DIMENSION: usize = 4096;

/// Prüft, ob ein Vektor gültige endliche Zahlen enthält und eine zulässige Dimension besitzt (1..=4096, D-87).
///
/// 1. Prüfung der Vektorlänge: Wenn leer, ist der Vektor leer (erlaubt für triviale Abgleiche, sonst DimensionTooLarge / InvalidInput).
/// 2. Prüfung der Maximaldimension (D-87): Wenn `vec.len() > 4096`, Abbruch mit Fehler.
/// 3. Iteration über alle Elemente der `f32`-Slice und Prüfung mittels `v.is_finite()`.
/// 4. Abbruch mit Fehler (`ContextraError::invalid_input`), falls mindestens ein Wert ungültig ist.
/// 5. Rückgabe von `Ok(())`, wenn alle Werte endlich sind.
///
///
///
#[inline]
pub fn validate_vector(vec: &[f32]) -> contextra_types::Result<()> {
    if vec.len() > MAX_VECTOR_DIMENSION {
        return Err(ContextraError::invalid_input(
            "Vector dimension exceeds maximum allowed limit of 4096 (D-87)",
        ));
    }
    if vec.iter().any(|v| !v.is_finite()) {
        return Err(ContextraError::invalid_input(
            "NaN or Infinity detected in vector",
        ));
    }
    Ok(())
}

/// Berechnet die Distanz zwischen zwei Vektoren inklusive Validierung (SPEC §8.4 / SI-05).
#[inline]
pub fn distance(metric: DistanceMetric, a: &[f32], b: &[f32]) -> contextra_types::Result<f32> {
    validate_vector(a)?;
    validate_vector(b)?;
    distance_prevalidated(metric, a, b)
}

/// Berechnet die Distanz zwischen zwei bereits vorvalidierten Vektoren (SPEC §8.4 / SI-05).
#[inline]
pub fn distance_prevalidated(
    metric: DistanceMetric,
    a: &[f32],
    b: &[f32],
) -> contextra_types::Result<f32> {
    compute_distance_trusted(a, b, metric)
}

/// Berechnet die Distanz zwischen zwei Vektoren unter Verwendung der angegebenen Metrik inklusive Validierung.
///
/// 1. Vektor `a` auf Nicht-Endlichkeit (NaN / Unendlich) validieren.
/// 2. Vektor `b` auf Nicht-Endlichkeit (NaN / Unendlich) validieren.
/// 3. An [`compute_distance_trusted`] delegieren.
///
///
///
#[inline]
pub fn compute_distance(
    a: &[f32],
    b: &[f32],
    metric: DistanceMetric,
) -> contextra_types::Result<f32> {
    distance(metric, a, b)
}

/// Berechnet die Distanz zwischen zwei bereits validierten `f32`-Vektoren ohne erneute NaN-Prüfung.
///
/// 1. Fallunterscheidung anhand der `DistanceMetric`.
/// 2. Aufruf der passenden SIMD-Dispatch-Funktion (`cosine_distance`, `euclidean_distance`, `dot_product_distance`).
/// 3. Längenabgleich und Ausführung im gewählten Hardware-Pfad.
///
///
#[inline]
pub fn compute_distance_trusted(
    a: &[f32],
    b: &[f32],
    metric: DistanceMetric,
) -> contextra_types::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::DotProduct => dot_product_distance(a, b),
        _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
    }
}

/// Berechnet die Distanz zwischen einem `f32`-Vektor und einem rohen LE-Byte-Puffer (`&[u8]`).
///
/// 1. Fallunterscheidung nach `DistanceMetric`.
/// 2. Aufruf der byte-kompatiblen Dispatch-Funktion (`cosine_distance_f32_bytes`, etc.).
/// 3. Konvertierung/Reinterpretation der Bytes als `f32` im Ziel-Kernel.
///
///   oder die Dimensionen nicht übereinstimmen.
///
#[inline]
pub fn compute_distance_f32_bytes_trusted(
    a: &[f32],
    b_bytes: &[u8],
    metric: DistanceMetric,
) -> contextra_types::Result<f32> {
    match metric {
        DistanceMetric::Cosine => cosine_distance_f32_bytes(a, b_bytes),
        DistanceMetric::Euclidean => euclidean_distance_f32_bytes(a, b_bytes),
        DistanceMetric::DotProduct => dot_product_distance_f32_bytes(a, b_bytes),
        _ => Err(ContextraError::invalid_input("Unsupported distance metric")),
    }
}
