// FILE-CONTEXT
// ZWECK: Runtime Hardware Feature Detection & Dispatcher.
// INVARIANTEN: Zero-Panic, sicherer Fallback auf Skalar, wenn CPU-Features fehlen.

//! # Modul `dispatch`
//!
//! **Ring-Zugehörigkeit:** Ring 0 (Unsafe-Insel).
//!
//! **Zweck:**
//! Laufzeit-Erkennung von Hardware-Eigenschaften (CPU Feature Detection) und dynamische Weiterleitung (Dispatch)
//! an den optimalen Kernel.
//!
//! **Funktionsweise:**
//! 1. Bei jedem Funktionsaufruf werden mittels Compiler-Makros (`is_x86_feature_detected!`,
//!    `is_aarch64_feature_detected!`) die verfügbaren Prozessorbefehle abgefragt.
//! 2. Es wird die höchstwertige verfügbare Vektorebene gewählt:
//!    - x86_64: AVX-512 (falls `avx512f` bzw. `avx512vnni`) -> AVX2+FMA -> Skalarer Fallback.
//!    - AArch64: NEON -> Skalarer Fallback.
//! 3. Sämtliche Vorbedingungen (z. B. identische Vektorlängen, Puffergrößen bei Bytes) werden
//!    vor dem Betreten von `unsafe`-Blöcken strikt validiert.
//! 4. Für Testläufe und Benchmarks existiert ein Thread-lokaler Überschreibungsmechanismus
//!    (SimdPathOverride), um gezielt einzelne Pfade zu isolieren.

use crate::kernels::*;
use contextra_types::ContextraError;

// ANCHOR[contextra-simd:DISP-OVERRIDE-ENUM]
/// SIMD path override options for testing purposes.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdPathOverride {
    /// Force scalar fallback execution.
    Scalar,
    /// Force AVX2 SIMD execution path.
    Avx2,
    /// Force AVX-512 SIMD execution path.
    Avx512,
    /// Force ARM NEON SIMD execution path.
    Neon,
}

#[cfg(test)]
std::thread_local! {
    static OVERRIDE_PATH: std::cell::Cell<Option<SimdPathOverride>> = const { std::cell::Cell::new(None) };
}

// ANCHOR[contextra-simd:DISP-SET-OVERRIDE]
/// Sets thread-local SIMD dispatch path override for testing.
#[cfg(test)]
pub fn set_dispatch_override(path: Option<SimdPathOverride>) {
    OVERRIDE_PATH.with(|p| p.set(path));
}

// ANCHOR[contextra-simd:DISP-GET-OVERRIDE]
/// Retrieves the current thread-local SIMD dispatch path override.
#[cfg(test)]
pub fn get_dispatch_override() -> Option<SimdPathOverride> {
    OVERRIDE_PATH.with(|p| p.get())
}

// ANCHOR[contextra-simd:DISP-COSINE-F32]
/// Computes cosine distance ($1.0 - \text{cosine\_similarity}$) between two `f32` vectors.
///
/// Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtime CPU feature detection,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// $d(a, b) = 1.0 - \frac{a \cdot b}{\|a\| \|b\|}$. Cosine similarity is clamped to $[-1.0, 1.0]$, yielding
/// distance in $[0.0, 2.0]$. Zero-norm vectors safely return `1.0`.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f` or `avx2`+`fma` on `x86_64`, or `neon` on `aarch64`.
#[inline]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => return Ok(scalar::cosine_distance_scalar(a, b)),
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") =>
            {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx2` and `fma` are verified.
                return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512 if is_x86_feature_detected!("avx512f") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx512f` is verified.
                return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });
            }
            #[cfg(target_arch = "aarch64")]
            SimdPathOverride::Neon if std::arch::is_aarch64_feature_detected!("neon") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `neon` is verified.
                return Ok(unsafe { neon::cosine_distance_neon(a, b) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx512f` is confirmed at runtime.
            return Ok(unsafe { avx512::cosine_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx2` and `fma` are confirmed at runtime.
            return Ok(unsafe { avx2::cosine_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `neon` is confirmed at runtime.
            return Ok(unsafe { neon::cosine_distance_neon(a, b) });
        }
    }

    Ok(scalar::cosine_distance_scalar(a, b))
}

// ANCHOR[contextra-simd:DISP-EUCLIDEAN-F32]
/// Computes Euclidean distance ($\|a - b\|_2$) between two `f32` vectors.
///
/// Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtime CPU feature detection,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// $d(a, b) = \sqrt{\sum_{i=0}^{N-1} (a_i - b_i)^2}$. Always non-negative ($\ge 0.0$).
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f` or `avx2`+`fma` on `x86_64`, or `neon` on `aarch64`.
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => return Ok(scalar::euclidean_distance_scalar(a, b)),
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") =>
            {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx2` and `fma` are verified.
                return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512 if is_x86_feature_detected!("avx512f") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx512f` is verified.
                return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });
            }
            #[cfg(target_arch = "aarch64")]
            SimdPathOverride::Neon if std::arch::is_aarch64_feature_detected!("neon") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `neon` is verified.
                return Ok(unsafe { neon::euclidean_distance_neon(a, b) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx512f` is confirmed at runtime.
            return Ok(unsafe { avx512::euclidean_distance_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx2` and `fma` are confirmed at runtime.
            return Ok(unsafe { avx2::euclidean_distance_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `neon` is confirmed at runtime.
            return Ok(unsafe { neon::euclidean_distance_neon(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_scalar(a, b))
}

// ANCHOR[contextra-simd:DISP-DOT-F32]
/// Computes dot product distance (negated dot product $-a \cdot b$) between two `f32` vectors.
///
/// Dynamically dispatches to AVX-512, AVX2+FMA, or NEON based on runtime CPU feature detection,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// Returns $- \sum_{i=0}^{N-1} a_i b_i$ so higher dot products yield smaller distance values.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f` or `avx2`+`fma` on `x86_64`, or `neon` on `aarch64`.
#[inline]
pub fn dot_product_distance(a: &[f32], b: &[f32]) -> Result<f32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => return Ok(scalar::dot_product_scalar(a, b)),
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") =>
            {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx2` and `fma` are verified.
                return Ok(unsafe { avx2::dot_product_avx2(a, b) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512 if is_x86_feature_detected!("avx512f") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx512f` is verified.
                return Ok(unsafe { avx512::dot_product_avx512(a, b) });
            }
            #[cfg(target_arch = "aarch64")]
            SimdPathOverride::Neon if std::arch::is_aarch64_feature_detected!("neon") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `neon` is verified.
                return Ok(unsafe { neon::dot_product_neon(a, b) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx512f` is confirmed at runtime.
            return Ok(unsafe { avx512::dot_product_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx2` and `fma` are confirmed at runtime.
            return Ok(unsafe { avx2::dot_product_avx2(a, b) });
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `neon` is confirmed at runtime.
            return Ok(unsafe { neon::dot_product_neon(a, b) });
        }
    }

    Ok(scalar::dot_product_scalar(a, b))
}

// ANCHOR[contextra-simd:DISP-COSINE-F32-BYTES]
/// Computes cosine distance between an `f32` slice and raw little-endian `f32` byte slice.
///
/// Dynamically dispatches to AVX-512 or AVX2+FMA based on runtime CPU feature detection,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// Decodes 4-byte float chunks from `b_bytes` and computes cosine distance against `a`.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `b_bytes.len() < a.len() * 4`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f` or `avx2`+`fma` on `x86_64`.
#[inline]
pub fn cosine_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => {
                return Ok(scalar::cosine_distance_f32_bytes_scalar(a, b_bytes))
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") =>
            {
                // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU features `avx2` and `fma` are verified.
                return Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512 if is_x86_feature_detected!("avx512f") => {
                // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU feature `avx512f` is verified.
                return Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU feature `avx512f` is confirmed at runtime.
            return Ok(unsafe { avx512::cosine_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU features `avx2` and `fma` are confirmed at runtime.
            return Ok(unsafe { avx2::cosine_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::cosine_distance_f32_bytes_scalar(a, b_bytes))
}

// ANCHOR[contextra-simd:DISP-EUCLIDEAN-F32-BYTES]
/// Computes Euclidean distance between an `f32` slice and raw little-endian `f32` byte slice.
///
/// Dynamically dispatches to AVX-512 or AVX2+FMA based on runtime CPU feature detection,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// Decodes 4-byte float chunks from `b_bytes` and computes Euclidean distance against `a`.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `b_bytes.len() < a.len() * 4`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f` or `avx2`+`fma` on `x86_64`.
#[inline]
pub fn euclidean_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => {
                return Ok(scalar::euclidean_distance_f32_bytes_scalar(a, b_bytes))
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") =>
            {
                // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU features `avx2` and `fma` are verified.
                return Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512 if is_x86_feature_detected!("avx512f") => {
                // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU feature `avx512f` is verified.
                return Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU feature `avx512f` is confirmed at runtime.
            return Ok(unsafe { avx512::euclidean_distance_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU features `avx2` and `fma` are confirmed at runtime.
            return Ok(unsafe { avx2::euclidean_distance_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::euclidean_distance_f32_bytes_scalar(a, b_bytes))
}

// ANCHOR[contextra-simd:DISP-DOT-F32-BYTES]
/// Computes dot product distance between an `f32` slice and raw little-endian `f32` byte slice.
///
/// Dynamically dispatches to AVX-512 or AVX2+FMA based on runtime CPU feature detection,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// Decodes 4-byte float chunks from `b_bytes` and computes negated dot product against `a`.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `b_bytes.len() < a.len() * 4`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f` or `avx2`+`fma` on `x86_64`.
#[inline]
pub fn dot_product_distance_f32_bytes(a: &[f32], b_bytes: &[u8]) -> Result<f32, ContextraError> {
    if b_bytes.len() < a.len() * 4 {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b_bytes.len() / 4,
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => {
                return Ok(scalar::dot_product_f32_bytes_scalar(a, b_bytes))
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2
                if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") =>
            {
                // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU features `avx2` and `fma` are verified.
                return Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512 if is_x86_feature_detected!("avx512f") => {
                // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU feature `avx512f` is verified.
                return Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU feature `avx512f` is confirmed at runtime.
            return Ok(unsafe { avx512::dot_product_f32_bytes_avx512(a, b_bytes) });
        }
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            // SAFETY: `b_bytes` length is validated to contain at least `a.len() * 4` bytes above, and CPU features `avx2` and `fma` are confirmed at runtime.
            return Ok(unsafe { avx2::dot_product_f32_bytes_avx2(a, b_bytes) });
        }
    }

    Ok(scalar::dot_product_f32_bytes_scalar(a, b_bytes))
}

// ANCHOR[contextra-simd:DISP-DOT-U8]
/// Computes integer dot product between two `u8` byte slices.
///
/// Dynamically dispatches to AVX-512 VNNI (`dpbusd`) or AVX2 (`madd_epi16`),
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// Calculates $\sum_{i=0}^{N-1} a_i b_i$ using 32-bit unsigned accumulation to prevent integer overflow.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f`+`avx512bw`+`avx512vnni` or `avx2` on `x86_64`.
#[inline]
pub fn dot_product_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => return Ok(scalar::dot_product_u8_scalar(a, b)),
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2 if is_x86_feature_detected!("avx2") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx2` is verified.
                return Ok(unsafe { avx2::dot_product_u8_avx2(a, b) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512
                if is_x86_feature_detected!("avx512f")
                    && is_x86_feature_detected!("avx512bw")
                    && is_x86_feature_detected!("avx512vnni") =>
            {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx512f`, `avx512bw`, `avx512vnni` are verified.
                return Ok(unsafe { avx512::dot_product_u8_avx512vnni(a, b) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && is_x86_feature_detected!("avx512vnni")
        {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx512f`, `avx512bw`, `avx512vnni` are confirmed at runtime.
            return Ok(unsafe { avx512::dot_product_u8_avx512vnni(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx2` is confirmed at runtime.
            return Ok(unsafe { avx2::dot_product_u8_avx2(a, b) });
        }
    }

    Ok(scalar::dot_product_u8_scalar(a, b))
}

// ANCHOR[contextra-simd:DISP-EUCLIDEAN-SQ-U8]
/// Computes squared Euclidean distance ($\|a - b\|_2^2$) between two `u8` byte slices.
///
/// Dynamically dispatches to AVX-512 or AVX2 SIMD implementations,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// Calculates $\sum_{i=0}^{N-1} (a_i - b_i)^2$ using 32-bit unsigned accumulation.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f`+`avx512bw` or `avx2` on `x86_64`.
#[inline]
pub fn euclidean_distance_sq_u8(a: &[u8], b: &[u8]) -> Result<u32, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => return Ok(scalar::euclidean_distance_sq_u8_scalar(a, b)),
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2 if is_x86_feature_detected!("avx2") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx2` is verified.
                return Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512
                if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") =>
            {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx512f` and `avx512bw` are verified.
                return Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx512f` and `avx512bw` are confirmed at runtime.
            return Ok(unsafe { avx512::euclidean_distance_sq_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx2` is confirmed at runtime.
            return Ok(unsafe { avx2::euclidean_distance_sq_u8_avx2(a, b) });
        }
    }

    Ok(scalar::euclidean_distance_sq_u8_scalar(a, b))
}

// ANCHOR[contextra-simd:DISP-COSINE-PARTS-U8]
/// Computes cosine similarity component parts (dot product, norm_a_sq, norm_b_sq) for two `u8` byte slices.
///
/// Dynamically dispatches to AVX-512 VNNI or AVX2 SIMD implementations,
/// falling back to scalar execution if SIMD features are missing or overridden in tests.
///
/// # Semantics
///
/// Computes `CosineSimilarityPartsU8 { dot: sum(a_i * b_i), norm_a_sq: sum(a_i^2), norm_b_sq: sum(b_i^2) }`.
///
/// # Errors
///
/// Returns [`ContextraError::EmbeddingDimensionMismatch`] if `a.len() != b.len()`.
/// Returns [`ContextraError::InvalidInput`] if a test path override specifies an unsupported CPU feature.
///
/// # Complexity
///
/// $O(N)$ time, $O(1)$ space where $N = a.len()$.
///
/// # Features
///
/// Requires runtime detection of `avx512f`+`avx512bw`+`avx512vnni` or `avx2` on `x86_64`.
#[inline]
pub fn cosine_similarity_parts_u8(
    a: &[u8],
    b: &[u8],
) -> Result<CosineSimilarityPartsU8, ContextraError> {
    if a.len() != b.len() {
        return Err(ContextraError::EmbeddingDimensionMismatch {
            expected: a.len(),
            got: b.len(),
        });
    }

    #[cfg(test)]
    if let Some(forced) = get_dispatch_override() {
        match forced {
            SimdPathOverride::Scalar => return Ok(scalar::cosine_similarity_parts_u8_scalar(a, b)),
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx2 if is_x86_feature_detected!("avx2") => {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx2` is verified.
                return Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) });
            }
            #[cfg(target_arch = "x86_64")]
            SimdPathOverride::Avx512
                if is_x86_feature_detected!("avx512f")
                    && is_x86_feature_detected!("avx512bw")
                    && is_x86_feature_detected!("avx512vnni") =>
            {
                // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx512f`, `avx512bw`, `avx512vnni` are verified.
                return Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) });
            }
            _ => {
                return Err(ContextraError::invalid_input(
                    "Requested SIMD path override is not supported on this host CPU",
                ));
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("avx512bw")
            && is_x86_feature_detected!("avx512vnni")
        {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU features `avx512f`, `avx512bw`, `avx512vnni` are confirmed at runtime.
            return Ok(unsafe { avx512::cosine_similarity_parts_u8_avx512(a, b) });
        }
        if is_x86_feature_detected!("avx2") {
            // SAFETY: Slice lengths `a` and `b` are validated equal above, and CPU feature `avx2` is confirmed at runtime.
            return Ok(unsafe { avx2::cosine_similarity_parts_u8_avx2(a, b) });
        }
    }

    Ok(scalar::cosine_similarity_parts_u8_scalar(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::DistanceMetric;

    struct OverrideGuard;

    impl OverrideGuard {
        fn new() -> Self {
            Self
        }
    }

    impl Drop for OverrideGuard {
        fn drop(&mut self) {
            set_dispatch_override(None);
        }
    }

    #[test]
    fn test_dispatch_override_setter_and_getter() {
        let _guard = OverrideGuard::new();

        assert_eq!(get_dispatch_override(), None);
        set_dispatch_override(Some(SimdPathOverride::Scalar));
        assert_eq!(get_dispatch_override(), Some(SimdPathOverride::Scalar));

        set_dispatch_override(Some(SimdPathOverride::Avx2));
        assert_eq!(get_dispatch_override(), Some(SimdPathOverride::Avx2));

        set_dispatch_override(None);
        assert_eq!(get_dispatch_override(), None);
    }

    #[test]
    fn test_forced_scalar_path_cosine() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();
        set_dispatch_override(Some(SimdPathOverride::Scalar));

        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [2.0f32, 3.0, 4.0, 5.0];

        let res = cosine_distance(&a, &b)?;
        assert!((0.0..=2.0).contains(&res));
        Ok(())
    }

    #[test]
    fn test_forced_scalar_path_euclidean() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();
        set_dispatch_override(Some(SimdPathOverride::Scalar));

        let a = [3.0f32, 0.0];
        let b = [0.0f32, 4.0];

        let res = euclidean_distance(&a, &b)?;
        assert_eq!(res, 5.0);
        Ok(())
    }

    #[test]
    fn test_forced_scalar_path_dot_product() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();
        set_dispatch_override(Some(SimdPathOverride::Scalar));

        let a = [1.0f32, 2.0, 3.0];
        let b = [4.0f32, 5.0, 6.0];

        let res = dot_product_distance(&a, &b)?;
        assert_eq!(res, -32.0);
        Ok(())
    }

    #[test]
    fn test_forced_scalar_path_f32_bytes() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();
        set_dispatch_override(Some(SimdPathOverride::Scalar));

        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [2.0f32, 3.0, 4.0, 5.0];
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let cos = cosine_distance_f32_bytes(&a, &b_bytes)?;
        let euc = euclidean_distance_f32_bytes(&a, &b_bytes)?;
        let dot = dot_product_distance_f32_bytes(&a, &b_bytes)?;

        assert!(cos >= 0.0);
        assert!(euc >= 0.0);
        assert_eq!(dot, -40.0);
        Ok(())
    }

    #[test]
    fn test_forced_scalar_path_u8_metrics() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();
        set_dispatch_override(Some(SimdPathOverride::Scalar));

        let a = [10u8, 20, 30, 40];
        let b = [5u8, 15, 25, 35];

        let dot = dot_product_u8(&a, &b)?;
        let euc_sq = euclidean_distance_sq_u8(&a, &b)?;
        let parts = cosine_similarity_parts_u8(&a, &b)?;

        assert_eq!(dot, 10 * 5 + 20 * 15 + 30 * 25 + 40 * 35);
        assert_eq!(euc_sq, 25 + 25 + 25 + 25);
        assert_eq!(parts.dot, dot);
        Ok(())
    }

    #[test]
    fn test_forced_scalar_path_high_level_facades() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();
        set_dispatch_override(Some(SimdPathOverride::Scalar));

        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [2.0f32, 3.0, 4.0, 5.0];
        let b_bytes: Vec<u8> = b.iter().flat_map(|x| x.to_le_bytes()).collect();

        let cos = crate::compute_distance(&a, &b, DistanceMetric::Cosine)?;
        let euc = crate::compute_distance_trusted(&a, &b, DistanceMetric::Euclidean)?;
        let dot_bytes =
            crate::compute_distance_f32_bytes_trusted(&a, &b_bytes, DistanceMetric::DotProduct)?;

        assert!(cos >= 0.0);
        assert!(euc >= 0.0);
        assert_eq!(dot_bytes, -40.0);
        Ok(())
    }

    #[test]
    fn test_unsupported_forced_path_rejection() {
        let _guard = OverrideGuard::new();

        #[cfg(target_arch = "x86_64")]
        {
            // Forcing ARM NEON on x86_64 host must return an unsupported error.
            set_dispatch_override(Some(SimdPathOverride::Neon));

            let a = [1.0f32, 2.0, 3.0, 4.0];
            let b = [2.0f32, 3.0, 4.0, 5.0];

            let res = cosine_distance(&a, &b);
            assert!(matches!(res, Err(ContextraError::InvalidInput(_))));

            let euc_res = euclidean_distance(&a, &b);
            assert!(matches!(euc_res, Err(ContextraError::InvalidInput(_))));

            let dot_res = dot_product_distance(&a, &b);
            assert!(matches!(dot_res, Err(ContextraError::InvalidInput(_))));
        }

        #[cfg(target_arch = "aarch64")]
        {
            // Forcing AVX2 or AVX512 on aarch64 host must return an unsupported error.
            set_dispatch_override(Some(SimdPathOverride::Avx2));

            let a = [1.0f32, 2.0, 3.0, 4.0];
            let b = [2.0f32, 3.0, 4.0, 5.0];

            let res = cosine_distance(&a, &b);
            assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
        }
    }

    #[test]
    fn test_bitwise_reproducibility_fixed_path() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();
        set_dispatch_override(Some(SimdPathOverride::Scalar));

        let a: Vec<f32> = (0..256).map(|i| (i as f32) * 0.137 - 17.5).collect();
        let b: Vec<f32> = (0..256).map(|i| (i as f32) * 0.219 - 8.3).collect();

        let initial_cos = cosine_distance(&a, &b)?;
        let initial_euc = euclidean_distance(&a, &b)?;
        let initial_dot = dot_product_distance(&a, &b)?;

        for _ in 0..100 {
            let cos = cosine_distance(&a, &b)?;
            let euc = euclidean_distance(&a, &b)?;
            let dot = dot_product_distance(&a, &b)?;

            assert_eq!(initial_cos.to_bits(), cos.to_bits());
            assert_eq!(initial_euc.to_bits(), euc.to_bits());
            assert_eq!(initial_dot.to_bits(), dot.to_bits());
        }
        Ok(())
    }

    #[test]
    fn test_forced_avx2_if_available() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();

        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            set_dispatch_override(Some(SimdPathOverride::Avx2));

            let a: Vec<f32> = (0..128).map(|i| (i as f32) * 0.1).collect();
            let b: Vec<f32> = (0..128).map(|i| (i as f32) * 0.2).collect();

            let cos = cosine_distance(&a, &b)?;
            let euc = euclidean_distance(&a, &b)?;
            let dot = dot_product_distance(&a, &b)?;

            assert!(cos >= 0.0);
            assert!(euc >= 0.0);
            assert!(dot <= 0.0);
        }
        Ok(())
    }

    #[test]
    fn test_forced_avx512_if_available() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();

        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx512f") {
            set_dispatch_override(Some(SimdPathOverride::Avx512));

            let a: Vec<f32> = (0..128).map(|i| (i as f32) * 0.1).collect();
            let b: Vec<f32> = (0..128).map(|i| (i as f32) * 0.2).collect();

            let cos = cosine_distance(&a, &b)?;
            let euc = euclidean_distance(&a, &b)?;
            let dot = dot_product_distance(&a, &b)?;

            assert!(cos >= 0.0);
            assert!(euc >= 0.0);
            assert!(dot <= 0.0);
        }
        Ok(())
    }

    #[test]
    fn test_forced_neon_if_available() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();

        #[cfg(target_arch = "aarch64")]
        if std::arch::is_aarch64_feature_detected!("neon") {
            set_dispatch_override(Some(SimdPathOverride::Neon));

            let a: Vec<f32> = (0..128).map(|i| (i as f32) * 0.1).collect();
            let b: Vec<f32> = (0..128).map(|i| (i as f32) * 0.2).collect();

            let cos = cosine_distance(&a, &b)?;
            let euc = euclidean_distance(&a, &b)?;
            let dot = dot_product_distance(&a, &b)?;

            assert!(cos >= 0.0);
            assert!(euc >= 0.0);
            assert!(dot <= 0.0);
        }
        Ok(())
    }

    #[test]
    fn test_dispatch_override_toggle_cycle() -> Result<(), ContextraError> {
        let _guard = OverrideGuard::new();

        let a = [1.0f32, 2.0, 3.0, 4.0];
        let b = [5.0f32, 6.0, 7.0, 8.0];

        // Default dispatch
        set_dispatch_override(None);
        let default_val = cosine_distance(&a, &b)?;

        // Forced scalar dispatch
        set_dispatch_override(Some(SimdPathOverride::Scalar));
        let scalar_val = cosine_distance(&a, &b)?;

        // Reverted to default dispatch
        set_dispatch_override(None);
        let reverted_val = cosine_distance(&a, &b)?;

        assert_eq!(default_val.to_bits(), reverted_val.to_bits());
        assert!((scalar_val - default_val).abs() < 1e-4);
        Ok(())
    }
}
