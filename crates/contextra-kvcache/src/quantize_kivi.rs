#![forbid(unsafe_code)]

//! KIVI 2-Bit Quantization Metadata and Unpacking Infrastructure.
//!
//! Provides metadata representations, validation, and dequantization for 2-bit
//! quantized KV-cache byte blocks. Tensor-level quantization occurs upstream
//! before serialization into `KvSegment`.

use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

/// Metadata required to dequantize a 2-bit KIVI block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KiviBlockMeta {
    /// Number of bits allocated per scalar value (must be 2 for KIVI).
    pub bits_per_value: u8,
    /// Scaling multiplier for linear quantization.
    pub scale: f32,
    /// Zero-point baseline shift for linear quantization.
    pub zero_point: f32,
    /// Number of floating point values encoded in the block.
    pub original_len: usize,
}

/// A 2-bit quantized byte block with accompanying metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KiviQuantizedBlock {
    /// Quantization metadata.
    pub meta: KiviBlockMeta,
    /// Packed 2-bit scalar values (4 values packed per byte, LSB-first).
    pub packed: Vec<u8>,
}

/// Dequantizes a 2-bit packed block back into floating point `Vec<f32>`.
///
/// # Errors
/// Returns `ContextraError::InvalidInput` if:
/// - `meta.bits_per_value != 2`
/// - `packed.len()` does not match `meta.original_len.div_ceil(4)`
/// - `meta.scale` or `meta.zero_point` is not finite
pub fn unpack_kivi_block(block: &KiviQuantizedBlock) -> Result<Vec<f32>> {
    if block.meta.bits_per_value != 2 {
        return Err(ContextraError::InvalidInput(format!(
            "KIVI dequantization requires bits_per_value == 2, got {}",
            block.meta.bits_per_value
        )));
    }

    let expected_packed_len = block.meta.original_len.div_ceil(4);
    if block.packed.len() != expected_packed_len {
        return Err(ContextraError::InvalidInput(format!(
            "Packed byte length mismatch: expected {} bytes for {} original elements, got {}",
            expected_packed_len,
            block.meta.original_len,
            block.packed.len()
        )));
    }

    if !block.meta.scale.is_finite() || !block.meta.zero_point.is_finite() {
        return Err(ContextraError::InvalidInput(
            "KIVI metadata scale and zero_point must be finite floats".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(block.meta.original_len);
    for i in 0..block.meta.original_len {
        let byte_idx = i / 4;
        let bit_shift = (i % 4) * 2;
        let byte = match block.packed.get(byte_idx) {
            Some(&b) => b,
            None => {
                return Err(ContextraError::InvalidInput(format!(
                    "Unexpected out-of-bounds byte access at index {}",
                    byte_idx
                )));
            }
        };

        let code = (byte >> bit_shift) & 0x03;
        let val = block.meta.zero_point + (code as f32) * block.meta.scale;
        result.push(val);
    }

    Ok(result)
}

/// Reference implementation for 2-bit linear min/max quantization.
///
/// Encodes a slice of floats into a 2-bit packed block using uniform scale and zero-point.
/// Used primarily for testing, validation, and reference round-trips.
pub fn pack_kivi_block(values: &[f32]) -> KiviQuantizedBlock {
    if values.is_empty() {
        return KiviQuantizedBlock {
            meta: KiviBlockMeta {
                bits_per_value: 2,
                scale: 0.0,
                zero_point: 0.0,
                original_len: 0,
            },
            packed: Vec::new(),
        };
    }

    let finite_vals: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let (min_v, max_v) = if !finite_vals.is_empty() {
        let min_val = finite_vals.iter().copied().fold(f32::INFINITY, f32::min);
        let max_val = finite_vals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (min_val, max_val)
    } else {
        (0.0, 0.0)
    };

    let (scale, zero_point) = if (max_v - min_v).abs() < f32::EPSILON {
        (0.0, min_v)
    } else {
        let sc = (max_v - min_v) / 3.0;
        (sc, min_v)
    };

    let packed_len = values.len().div_ceil(4);
    let mut packed = vec![0u8; packed_len];

    for (i, &v) in values.iter().enumerate() {
        let byte_idx = i / 4;
        let bit_shift = (i % 4) * 2;

        let code = if scale > 0.0 && v.is_finite() {
            let norm = ((v - zero_point) / scale).round();
            norm.clamp(0.0, 3.0) as u8
        } else {
            0u8
        };

        if let Some(byte) = packed.get_mut(byte_idx) {
            *byte |= (code & 0x03) << bit_shift;
        }
    }

    KiviQuantizedBlock {
        meta: KiviBlockMeta {
            bits_per_value: 2,
            scale,
            zero_point,
            original_len: values.len(),
        },
        packed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_kivi_metadata_validation_errors_no_panic() {
        let meta_wrong_bits = KiviBlockMeta {
            bits_per_value: 4, // Invalid for KIVI
            scale: 1.0,
            zero_point: 0.0,
            original_len: 4,
        };
        let block_wrong_bits = KiviQuantizedBlock {
            meta: meta_wrong_bits,
            packed: vec![0x00],
        };
        assert!(unpack_kivi_block(&block_wrong_bits).is_err());

        let meta_len_mismatch = KiviBlockMeta {
            bits_per_value: 2,
            scale: 1.0,
            zero_point: 0.0,
            original_len: 8, // Requires 2 packed bytes
        };
        let block_len_mismatch = KiviQuantizedBlock {
            meta: meta_len_mismatch,
            packed: vec![0x00], // Only 1 byte provided
        };
        assert!(unpack_kivi_block(&block_len_mismatch).is_err());

        let meta_nan = KiviBlockMeta {
            bits_per_value: 2,
            scale: f32::NAN,
            zero_point: 0.0,
            original_len: 0,
        };
        let block_nan = KiviQuantizedBlock {
            meta: meta_nan,
            packed: vec![],
        };
        assert!(unpack_kivi_block(&block_nan).is_err());
    }

    #[test]
    fn test_kivi_pack_unpack_roundtrip_basic() {
        let input = vec![0.0, 1.0, 2.0, 3.0];
        let block = pack_kivi_block(&input);
        assert_eq!(block.meta.bits_per_value, 2);
        assert_eq!(block.meta.original_len, 4);
        assert_eq!(block.packed.len(), 1);

        let reconstructed = unpack_kivi_block(&block).unwrap();
        assert_eq!(reconstructed.len(), 4);
        for (orig, recon) in input.iter().zip(reconstructed.iter()) {
            assert!((orig - recon).abs() < 1e-4);
        }
    }

    proptest! {
        /// Property test: Quantization error bound verification.
        ///
        /// Theoretical error bound reasoning:
        /// For a 2-bit linear uniform quantizer mapping [min_val, max_val] into 4 discrete levels,
        /// step size delta = (max_val - min_val) / 3.
        /// The maximum distance from any value to its nearest quantization step is delta / 2 = (max_val - min_val) / 6.
        /// Accounting for floating point precision in scale calculation, the absolute reconstruction error
        /// |v_orig - v_recon| is bounded by (max_val - min_val) / 6.0 + epsilon.
        #[test]
        fn prop_kivi_quantization_error_within_theoretical_bound(
            vec in proptest::collection::vec(-1000.0f32..1000.0f32, 1..64)
        ) {
            let block = pack_kivi_block(&vec);
            let recon = unpack_kivi_block(&block).unwrap();

            let min_v = vec.iter().copied().fold(f32::INFINITY, f32::min);
            let max_v = vec.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let delta = max_v - min_v;

            // Theoretical max discretization error + small epsilon for f32 round-off
            let max_allowed_error = delta / 6.0 + 1e-4;

            for (orig, rec) in vec.iter().zip(recon.iter()) {
                let err = (orig - rec).abs();
                prop_assert!(
                    err <= max_allowed_error,
                    "Reconstruction error {} exceeded bound {} for orig={} rec={}",
                    err,
                    max_allowed_error,
                    orig,
                    rec
                );
            }
        }
    }
}
