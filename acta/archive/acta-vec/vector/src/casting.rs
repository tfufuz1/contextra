// Zero-Panic geprüfte numerische Typkonvertierungen für acta-vec.
// Invarianten: Zero-Panic, kein ungeprüftes Truncation-Casting (`as`), strikte Fehlerbehandlung.
// HOTSPOTS: casting.rs:18-24 (to_u32), casting.rs:32-38 (u64_to_usize), casting.rs:60-78 (float casts)

//! Zero-Panic geprüfte numerische Typkonvertierungen für Vektorindizes.
//!
//! Stellt typsichere, validierte Bereichsprüfungen bereit, die unüberprüfte `as`-Casts
//! in Graph-, Block- und Slot-Offsets ersetzen.

use crate::error::{Result, VectorError};

// SECTION Integer-Konvertierungen (Ganzzahlen)

/// Konvertiert `usize` zu `u32`, liefert `VectorError::InvalidInput` bei Überlauf.
#[inline]
pub fn to_u32(val: usize) -> Result<u32> {
    u32::try_from(val).map_err(|_| VectorError::invalid_input("value exceeds u32 capacity limit"))
}

/// Konvertiert `u32` zu `usize`. Unfehlbar auf 32-Bit und 64-Bit Zielarchitekturen.
#[inline]
pub fn u32_to_usize(val: u32) -> usize {
    val as usize
}

/// Konvertiert `u64` zu `usize`, liefert `VectorError::InvalidInput` bei Überlauf auf 32-Bit.
#[inline]
pub fn u64_to_usize(val: u64) -> Result<usize> {
    usize::try_from(val).map_err(|_| VectorError::invalid_input("u64 value exceeds pointer width"))
}

/// Konvertiert `u64` zu `u32`, liefert `VectorError::InvalidInput` bei Überschreitung von `u32::MAX`.
#[inline]
pub fn u64_to_u32(val: u64) -> Result<u32> {
    u32::try_from(val).map_err(|_| VectorError::invalid_input("u64 value exceeds u32 bounds"))
}

/// Konvertiert `usize` zu `u8`, liefert `VectorError::InvalidInput` bei Überschreitung von 255.
#[inline]
pub fn to_u8(val: usize) -> Result<u8> {
    u8::try_from(val).map_err(|_| VectorError::invalid_input("value exceeds u8 capacity limit"))
}

/// Konvertiert `usize` zu `u64`. Unfehlbar auf 32-Bit und 64-Bit Systemen.
#[inline]
pub fn to_u64(val: usize) -> u64 {
    val as u64
}
// !SECTION

// SECTION Floating-Point Konvertierungen

/// Konvertiert `f32` defensiv zu `usize` mit Validierung gegen Nicht-Endlichkeit und negative Werte.
#[inline]
pub fn f32_to_usize(val: f32) -> Result<usize> {
    if !val.is_finite() || val < 0.0 {
        return Err(VectorError::invalid_input(
            "floating point value cannot be converted to unsigned index",
        ));
    }
    let u = val as usize;
    Ok(u)
}

/// Konvertiert `f64` defensiv zu `usize` mit Validierung gegen Nicht-Endlichkeit und negative Werte.
#[inline]
pub fn f64_to_usize(val: f64) -> Result<usize> {
    if !val.is_finite() || val < 0.0 {
        return Err(VectorError::invalid_input(
            "floating point value cannot be converted to unsigned index",
        ));
    }
    let u = val as usize;
    Ok(u)
}
// !SECTION
