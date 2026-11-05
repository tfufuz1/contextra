// FILE-CONTEXT
// ZWECK: Runtime Hardware Feature Detection & Dispatcher.
// INVARIANTEN: Zero-Panic, sicherer Fallback auf Skalar, wenn CPU-Features fehlen.

//! # Modul `dispatch`
//!
//! **Ring-Zugehörigkeit:** Ring 1 (Unsafe-Insel).
//!
//! **Zweck:**
//! Laufzeit-Erkennung von Hardware-Eigenschaften (CPU Feature Detection) und dynamische Weiterleitung (Dispatch)
//! an den optimalen Kernel.

use crate::kernels::*;
use contextra_types::ContextraError;
use std::sync::OnceLock;

/// SIMD execution path selected at runtime based on hardware capabilities.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdPath {
    Avx512,
    Avx2Fma,
    Neon,
    Scalar,
}

static CACHED_SIMD_PATH: OnceLock<SimdPath> = OnceLock::new();

/// Ermittelt den aktiv gewählten SIMD-Pfad für Diagnose und Berichte (SI-04).
/// Der Pfad wird beim ersten Aufruf bestimmt und gecacht.
#[inline]
pub fn active_path() -> SimdPath {
    *CACHED_SIMD_PATH.get_or_init(detect_hardware_path)
}

fn detect_hardware_path() -> SimdPath {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return SimdPath::Avx512;
        }
        if is_x86_feature_detected!("avx2") {
            return SimdPath::Avx2Fma;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return SimdPath::Neon;
        }
    }

    SimdPath::Scalar
}

/// SIMD path override options for testing purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdPathOverride {
    Scalar,
    Avx2,
    Avx512,
    Neon,
}

std::thread_local! {
    static OVERRIDE_PATH: std::cell::Cell<Option<SimdPathOverride>> = const { std::cell::Cell::new(None) };
}

/// Sets thread-local SIMD dispatch path override for testing and verification.
pub fn set_dispatch_override(path: Option<SimdPathOverride>) {
    OVERRIDE_PATH.with(|p| p.set(path));
}

/// Retrieves the current thread-local SIMD dispatch path override.
pub fn get_dispatch_override() -> Option<SimdPathOverride> {
    OVERRIDE_PATH.with(|p| p.get())
}

#[inline]
fn resolve_dispatch_path() -> Result<SimdPath, ContextraError> {
    match get_dispatch_override() {
        Some(SimdPathOverride::Scalar) => Ok(SimdPath::Scalar),
        #[cfg(target_arch = "x86_64")]
        Some(SimdPathOverride::Avx512) if is_x86_feature_detected!("avx512f") => {
            Ok(SimdPath::Avx512)
        }
        #[cfg(target_arch = "x86_64")]
        Some(SimdPathOverride::Avx2) if is_x86_feature_detected!("avx2") => Ok(SimdPath::Avx2Fma),
        #[cfg(target_arch = "aarch64")]
        Some(SimdPathOverride::Neon) if std::arch::is_aarch64_feature_detected!("neon") => {
            Ok(SimdPath::Neon)
        }
        Some(_) => Err(ContextraError::invalid_input(
            "Unsupported SIMD path override",
        )),
        None => Ok(active_path()),
    }
}

/// Computes cosine distance ($1.0 - \text{cosine\_similarity}$) between two `f32` vectors.
#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    match resolve_dispatch_path()? {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Längen geprüft, avx512f verifiziert.
        SimdPath::Avx512 => Ok(unsafe { avx512::cosine_distance_avx512(a, b) }),
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Längen geprüft, avx2 verifiziert.
        SimdPath::Avx2Fma => Ok(unsafe { avx2::cosine_distance_avx2(a, b) }),
        #[cfg(target_arch = "aarch64")]
        // SAFETY: Längen geprüft, neon verifiziert.
        SimdPath::Neon => Ok(unsafe { neon::cosine_distance_neon(a, b) }),
        _ => Ok(scalar::cosine_distance_scalar(a, b)),
    }
}

/// Computes Euclidean distance between two `f32` vectors.
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    match resolve_dispatch_path()? {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Längen geprüft, avx512f verifiziert.
        SimdPath::Avx512 => Ok(unsafe { avx512::euclidean_distance_avx512(a, b) }),
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Längen geprüft, avx2 verifiziert.
        SimdPath::Avx2Fma => Ok(unsafe { avx2::euclidean_distance_avx2(a, b) }),
        #[cfg(target_arch = "aarch64")]
        // SAFETY: Längen geprüft, neon verifiziert.
        SimdPath::Neon => Ok(unsafe { neon::euclidean_distance_neon(a, b) }),
        _ => Ok(scalar::euclidean_distance_scalar(a, b)),
    }
}

/// Computes dot product distance ($- \sum a_i b_i$) between two `f32` vectors.
#[inline]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    match resolve_dispatch_path()? {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Längen geprüft, avx512f verifiziert.
        SimdPath::Avx512 => Ok(unsafe { avx512::dot_product_avx512(a, b) }),
        #[cfg(target_arch = "x86_64")]
        // SAFETY: Längen geprüft, avx2 verifiziert.
        SimdPath::Avx2Fma => Ok(unsafe { avx2::dot_product_avx2(a, b) }),
        #[cfg(target_arch = "aarch64")]
        // SAFETY: Längen geprüft, neon verifiziert.
        SimdPath::Neon => Ok(unsafe { neon::dot_product_neon(a, b) }),
        _ => Ok(scalar::dot_product_scalar(a, b)),
    }
}

/// Computes cosine distance between an `f32` vector and an LE raw `f32` byte slice.
#[inline]
pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    match resolve_dispatch_path()? {
        #[cfg(target_arch = "x86_64")]
        SimdPath::Avx512 => {
            // SAFETY: Byte-Länge gegen a.len() * 4 geprüft, avx512f aktiv.
            Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) })
        }
        #[cfg(target_arch = "x86_64")]
        SimdPath::Avx2Fma => {
            // SAFETY: Byte-Länge gegen a.len() * 4 geprüft, avx2 aktiv.
            Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) })
        }
        _ => Ok(scalar::cosine_distance_f32_bytes_scalar(a, b_bytes)),
    }
}

/// Computes Euclidean distance between an `f32` vector and an LE raw `f32` byte slice.
#[inline]
pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    match resolve_dispatch_path()? {
        #[cfg(target_arch = "x86_64")]
        SimdPath::Avx512 => {
            // SAFETY: Byte-Länge gegen a.len() * 4 geprüft, avx512f aktiv.
            Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) })
        }
        #[cfg(target_arch = "x86_64")]
        SimdPath::Avx2Fma => {
            // SAFETY: Byte-Länge gegen a.len() * 4 geprüft, avx2 aktiv.
            Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) })
        }
        _ => Ok(scalar::euclidean_distance_f32_bytes_scalar(a, b_bytes)),
    }
}

/// Computes dot product distance between an `f32` vector and an LE raw `f32` byte slice.
#[inline]
pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    match resolve_dispatch_path()? {
        #[cfg(target_arch = "x86_64")]
        SimdPath::Avx512 => {
            // SAFETY: Byte-Länge gegen a.len() * 4 geprüft, avx512f aktiv.
            Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) })
        }
        #[cfg(target_arch = "x86_64")]
        SimdPath::Avx2Fma => {
            // SAFETY: Byte-Länge gegen a.len() * 4 geprüft, avx2 aktiv.
            Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) })
        }
        _ => Ok(scalar::dot_product_f32_bytes_scalar(a, b_bytes)),
    }
}

/// Computes integer dot product between two `u8` byte slices.
#[inline]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }
    Ok(scalar::dot_product_u8_scalar(a, b))
}

/// Computes squared Euclidean distance between two `u8` byte slices.
#[inline]
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }
    Ok(scalar::euclidean_distance_sq_u8_scalar(a, b))
}

/// Computes cosine similarity component parts for two `u8` byte slices.
#[inline]
pub fn cosine_similarity_parts_u8(
    a: &[u8],
    b: &[u8],
) -> Result<scalar::CosineSimilarityPartsU8, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }
    Ok(scalar::cosine_similarity_parts_u8_scalar(a, b))
}

pub use crate::kernels::scalar::dot_product_f32_u8;
pub use crate::kernels::scalar::euclidean_distance_sq_f32_u8;
