// FILE-CONTEXT
// ZWECK: Validierungsfunktionen für Vektoren und Bytes (INV-SIMD-FINITE-1, INV-SIMD-ALIGN-1).
// INVARIANTEN: Zero unsafe code (#![forbid(unsafe_code)]); NaN und Inf werden zuverlässig abgewiesen.
// HOTSPOTS: validate_vector, validate_slices, validate_bytes
// STAND: 2026-11-12

//! # Vector Validation Module
//!
//! Funktionen zur Prüfung von Vektordimensionen, Ausrichtung und Endlichkeit.

use crate::error::{Result, VectorError};

// ANCHOR[contextra-vector:DIST-VALIDATE-VECTOR]
/// Prüft, ob ein Vektor frei von NaN oder Unendlichkeitswerten ist (INV-SIMD-FINITE-1).
#[inline]
pub fn validate_vector(vec: &[f32]) -> Result<()> {
    if vec.iter().any(|v| !v.is_finite()) {
        return Err(VectorError::invalid_input(
            "NaN or Infinity detected in vector",
        ));
    }
    Ok(())
}

/// Prüft zwei Vektor-Slices auf übereinstimmende Länge und Endlichkeit.
#[inline]
pub(crate) fn validate_slices(a: &[f32], b: &[f32]) -> Result<()> {
    if a.len() != b.len() {
        return Err(VectorError::invalid_input(format!(
            "Vector dimensions must match: {} != {}",
            a.len(),
            b.len()
        )));
    }
    validate_vector(a)?;
    validate_vector(b)?;
    Ok(())
}

/// Prüft, ob Byte-Slices für Little-Endian f32 ausgerichtet und dimensional passend sind.
#[inline]
pub(crate) fn validate_bytes(a: &[f32], b_bytes: &[u8]) -> Result<()> {
    if b_bytes.len() % 4 != 0 {
        return Err(VectorError::invalid_input(format!(
            "Byte slice length {} is not aligned to 4 bytes",
            b_bytes.len()
        )));
    }
    if b_bytes.len() / 4 != a.len() {
        return Err(VectorError::invalid_input(format!(
            "Dimension mismatch: f32 slice len {} != byte float count {}",
            a.len(),
            b_bytes.len() / 4
        )));
    }
    validate_vector(a)?;
    for chunk in b_bytes.chunks_exact(4) {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(chunk);
        let val = f32::from_le_bytes(arr);
        if !val.is_finite() {
            return Err(VectorError::invalid_input(
                "NaN or Infinity detected in vector bytes",
            ));
        }
    }
    Ok(())
}
