// FILE-CONTEXT
// ZWECK: InjectedDistanceKernel zur Anbindung externer DistanceFn-Typen.
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); Propagiert NaN als Fehler.
// HOTSPOTS: InjectedDistanceKernel
// STAND: 2026-11-12

//! # Injected Distance Kernel Module
//!
//! Kapselt eine benutzerdefinierte Implementierung von [`DistanceFn`] in einen [`DistanceKernel`].

use super::kernel::{CosineSimilarityPartsU8, DistanceKernel};
use super::validate::validate_slices;
use crate::contract::DistanceFn;
use crate::error::{Result, VectorError};
use crate::types::DistanceMetric;

// ANCHOR[contextra-vector:DIST-INJECTED-KERNEL]
/// Adapter, der einen [`DistanceFn`]-Typ in das [`DistanceKernel`]-Interface einbettet.
#[derive(Debug, Clone)]
pub struct InjectedDistanceKernel<D: DistanceFn> {
    pub distance_fn: D,
}

impl<D: DistanceFn> InjectedDistanceKernel<D> {
    /// Erzeugt einen neuen injizierten Kernel aus einer [`DistanceFn`].
    pub fn new(distance_fn: D) -> Self {
        Self { distance_fn }
    }
}

impl<D: DistanceFn + 'static> DistanceKernel for InjectedDistanceKernel<D> {
    #[inline]
    fn distance(&self, _metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32> {
        validate_slices(a, b)?;
        let d = self.distance_fn.distance(a, b);
        if d.is_nan() {
            Err(VectorError::invalid_input(
                "Distance computation produced NaN",
            ))
        } else {
            Ok(d)
        }
    }

    #[inline]
    fn distance_f32_bytes(
        &self,
        _metric: DistanceMetric,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32> {
        let mut b_vec = Vec::with_capacity(a.len());
        for chunk in b_bytes.chunks_exact(4) {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(chunk);
            b_vec.push(f32::from_le_bytes(arr));
        }
        self.distance(_metric, a, &b_vec)
    }

    #[inline]
    fn distance_trusted(&self, metric: DistanceMetric, a: &[f32], b: &[f32]) -> Result<f32> {
        self.distance(metric, a, b)
    }

    #[inline]
    fn distance_f32_bytes_trusted(
        &self,
        metric: DistanceMetric,
        a: &[f32],
        b_bytes: &[u8],
    ) -> Result<f32> {
        self.distance_f32_bytes(metric, a, b_bytes)
    }

    #[inline]
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        self.distance(DistanceMetric::Cosine, a, b)
    }

    #[inline]
    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        self.distance(DistanceMetric::DotProduct, a, b)
    }

    #[inline]
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32> {
        self.distance(DistanceMetric::Euclidean, a, b)
    }

    #[inline]
    fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        let mut sum = 0u32;
        let len = a.len().min(b.len());
        for i in 0..len {
            let diff = a[i] as i32 - b[i] as i32;
            sum += (diff * diff) as u32;
        }
        sum
    }

    #[inline]
    fn euclidean_distance_sq_f32_u8(
        &self,
        a: &[f32],
        b: &[u8],
        alphas: &[f32],
        mins: &[f32],
    ) -> f32 {
        let mut sum = 0.0f32;
        let len = a.len().min(b.len()).min(alphas.len()).min(mins.len());
        for i in 0..len {
            let dequant = mins[i] + (b[i] as f32) * alphas[i];
            let diff = a[i] - dequant;
            sum += diff * diff;
        }
        sum
    }

    #[inline]
    fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32 {
        let mut dot = 0u32;
        let len = a.len().min(b.len());
        for i in 0..len {
            dot += (a[i] as u32) * (b[i] as u32);
        }
        dot
    }

    #[inline]
    fn dot_product_f32_u8(&self, a: &[f32], b: &[u8]) -> f32 {
        let mut dot = 0.0f32;
        let len = a.len().min(b.len());
        for i in 0..len {
            dot += a[i] * (b[i] as f32);
        }
        dot
    }

    #[inline]
    fn cosine_similarity_parts_u8(&self, a: &[u8], b: &[u8]) -> CosineSimilarityPartsU8 {
        let mut dot = 0u32;
        let mut norm_a = 0u32;
        let mut norm_b = 0u32;
        let len = a.len().min(b.len());

        for i in 0..len {
            let val_a = a[i] as u32;
            let val_b = b[i] as u32;
            dot += val_a * val_b;
            norm_a += val_a * val_a;
            norm_b += val_b * val_b;
        }

        CosineSimilarityPartsU8 {
            dot,
            norm_a_sq: norm_a,
            norm_b_sq: norm_b,
        }
    }

    #[inline]
    fn normalize_inplace(&self, v: &mut [f32]) {
        let norm_sq: f32 = v.iter().map(|x| x * x).sum();
        if norm_sq > 0.0 {
            let inv_norm = 1.0 / norm_sq.sqrt();
            for x in v.iter_mut() {
                *x *= inv_norm;
            }
        }
    }
}
