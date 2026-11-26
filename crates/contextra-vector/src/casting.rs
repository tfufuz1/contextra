//! Zero-panic checked numerical conversions for vector indices.
//!
//! Provides typed, checked bounds validation replacing unconstrained `as` casts.

use contextra_types::{ContextraError, Result};

/// Converts `usize` to `u32`, returning `ContextraError::invalid_input` if out of bounds.
#[inline]
pub fn to_u32(val: usize) -> Result<u32> {
    u32::try_from(val).map_err(|_| {
        ContextraError::invalid_input("value exceeds u32 capacity limit")
    })
}

/// Converts `u32` to `usize`. On supported architectures (32-bit and 64-bit), this is infallible.
#[inline]
pub fn u32_to_usize(val: u32) -> usize {
    val as usize
}

/// Converts `u64` to `usize`, returning `ContextraError::invalid_input` if out of bounds.
#[inline]
pub fn u64_to_usize(val: u64) -> Result<usize> {
    usize::try_from(val).map_err(|_| {
        ContextraError::invalid_input("u64 value exceeds pointer width")
    })
}

/// Converts `u64` to `u32`, returning `ContextraError::invalid_input` if out of bounds.
#[inline]
pub fn u64_to_u32(val: u64) -> Result<u32> {
    u32::try_from(val).map_err(|_| {
        ContextraError::invalid_input("u64 value exceeds u32 bounds")
    })
}

/// Converts `usize` to `u8`, returning `ContextraError::invalid_input` if out of bounds.
#[inline]
pub fn to_u8(val: usize) -> Result<u8> {
    u8::try_from(val).map_err(|_| {
        ContextraError::invalid_input("value exceeds u8 capacity limit")
    })
}

/// Converts `usize` to `u64`. Infallible on 32-bit and 64-bit platforms.
#[inline]
pub fn to_u64(val: usize) -> u64 {
    val as u64
}

/// Converts `f32` to `usize` defensively with bounds validation.
#[inline]
pub fn f32_to_usize(val: f32) -> Result<usize> {
    if !val.is_finite() || val < 0.0 {
        return Err(ContextraError::invalid_input(
            "floating point value cannot be converted to unsigned index",
        ));
    }
    let u = val as usize;
    Ok(u)
}

/// Converts `f64` to `usize` defensively with bounds validation.
#[inline]
pub fn f64_to_usize(val: f64) -> Result<usize> {
    if !val.is_finite() || val < 0.0 {
        return Err(ContextraError::invalid_input(
            "floating point value cannot be converted to unsigned index",
        ));
    }
    let u = val as usize;
    Ok(u)
}
