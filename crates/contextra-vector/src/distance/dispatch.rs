// FILE-CONTEXT
// ZWECK: Dispatcher für Distanzberechnung (SIMD vs. Scalar) mit flexiblen Metrik-Parametern.
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); Saubere Feature-Trennung von contextra-simd.
// HOTSPOTS: compute_distance, compute_distance_trusted, compute_distance_f32_bytes_trusted
// STAND: 2026-11-12

//! # Distance Dispatch Module
//!
//! Stellt die freistehenden Einstiegspunkte für Distanzberechnungen bereit,
//! die bei aktivierter `simd`-Feature auf `contextra_simd` dispatchen und andernfalls
//! die bitgleiche skalare Referenz verwenden.

#[cfg(not(feature = "simd"))]
use super::scalar::ScalarKernel;
#[cfg(feature = "simd")]
use super::validate::validate_vector;
use crate::error::Result;
#[cfg(feature = "simd")]
use crate::error::VectorError;
use crate::types::DistanceMetric;

// ANCHOR[contextra-vector:DIST-COMPUTE-DISTANCE]
/// Berechnet die Distanz zwischen zwei Vektoren nach Endlichkeitsvalidierung.
#[inline]
pub fn compute_distance<M: Into<DistanceMetric>>(a: &[f32], b: &[f32], metric: M) -> Result<f32> {
    let metric: DistanceMetric = metric.into();
    #[cfg(feature = "simd")]
    {
        validate_vector(a)?;
        validate_vector(b)?;
        compute_distance_trusted(a, b, metric)
    }
    #[cfg(not(feature = "simd"))]
    {
        ScalarKernel.distance(metric, a, b)
    }
}

// ANCHOR[contextra-vector:DIST-COMPUTE-TRUSTED]
/// Berechnet die Distanz zwischen zwei vorvalidierten f32-Slices.
#[inline]
pub fn compute_distance_trusted<M: Into<DistanceMetric>>(
    a: &[f32],
    b: &[f32],
    metric: M,
) -> Result<f32> {
    let metric: DistanceMetric = metric.into();
    #[cfg(feature = "simd")]
    {
        let disp = contextra_simd::Dispatcher::auto();
        match metric {
            DistanceMetric::Cosine => disp
                .cosine_distance(a, b)
                .map_err(|e| VectorError::invalid_input(e.to_string())),
            DistanceMetric::Euclidean => disp
                .euclidean_distance(a, b)
                .map_err(|e| VectorError::invalid_input(e.to_string())),
            DistanceMetric::DotProduct => disp
                .dot_product_distance(a, b)
                .map_err(|e| VectorError::invalid_input(e.to_string())),
        }
    }
    #[cfg(not(feature = "simd"))]
    {
        ScalarKernel.distance_trusted(metric, a, b)
    }
}

// ANCHOR[contextra-vector:DIST-COMPUTE-BYTES-TRUSTED]
/// Berechnet die Distanz zwischen vorvalidiertem f32-Slice und Little-Endian f32 Byte-Slice.
#[inline]
pub fn compute_distance_f32_bytes_trusted<M: Into<DistanceMetric>>(
    a: &[f32],
    b_bytes: &[u8],
    metric: M,
) -> Result<f32> {
    let metric: DistanceMetric = metric.into();
    #[cfg(feature = "simd")]
    {
        let disp = contextra_simd::Dispatcher::auto();
        match metric {
            DistanceMetric::Cosine => disp
                .cosine_distance_f32_bytes(a, b_bytes)
                .map_err(|e| VectorError::invalid_input(e.to_string())),
            DistanceMetric::Euclidean => disp
                .euclidean_distance_f32_bytes(a, b_bytes)
                .map_err(|e| VectorError::invalid_input(e.to_string())),
            DistanceMetric::DotProduct => disp
                .dot_product_distance_f32_bytes(a, b_bytes)
                .map_err(|e| VectorError::invalid_input(e.to_string())),
        }
    }
    #[cfg(not(feature = "simd"))]
    {
        ScalarKernel.distance_f32_bytes_trusted(metric, a, b_bytes)
    }
}

/// Bequemlichkeitsfunktion für Kosinus-Distanz mit automatischer Validierung.
#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    compute_distance(a, b, DistanceMetric::Cosine)
}

/// Bequemlichkeitsfunktion für Skalarprodukt-Distanz mit automatischer Validierung.
#[inline]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    compute_distance(a, b, DistanceMetric::DotProduct)
}

/// Bequemlichkeitsfunktion für Euklidische Distanz mit automatischer Validierung.
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32> {
    compute_distance(a, b, DistanceMetric::Euclidean)
}
