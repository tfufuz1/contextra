#![forbid(unsafe_code)]

//! KIVI 2-Bit Quantization Metadata, Asymmetric Quantization, and Unpacking Infrastructure.
//!
//! Provides metadata representations, validation, asymmetric 2-bit quantization
//! (per-channel for keys, per-token for values), and dequantization for KV-cache byte blocks.

use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

/// Configuration for KIVI quantization parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KiviQuantizeConfig {
    /// Group size for key channel quantization (default: 16).
    pub key_group_size: usize,
    /// Whether value quantization is enabled (default: true).
    pub quantize_values: bool,
}

impl Default for KiviQuantizeConfig {
    fn default() -> Self {
        Self {
            key_group_size: 16,
            quantize_values: true,
        }
    }
}

/// A view over unquantized raw key and value tensors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KvTensorView {
    /// Key tensor floats in [num_tokens, num_channels] row-major order.
    pub keys: Vec<f32>,
    /// Value tensor floats in [num_tokens, num_channels] row-major order.
    pub values: Vec<f32>,
    /// Number of tokens encoded.
    pub num_tokens: usize,
    /// Number of channels per head/token.
    pub num_channels: usize,
}

impl KvTensorView {
    /// Creates a new `KvTensorView` validating element count dimensions.
    pub fn new(
        keys: Vec<f32>,
        values: Vec<f32>,
        num_tokens: usize,
        num_channels: usize,
    ) -> Result<Self> {
        let expected = num_tokens * num_channels;
        if keys.len() != expected {
            return Err(ContextraError::InvalidInput(format!(
                "Keys length {} does not match num_tokens ({}) * num_channels ({}) = {}",
                keys.len(),
                num_tokens,
                num_channels,
                expected
            )));
        }
        if values.len() != expected {
            return Err(ContextraError::InvalidInput(format!(
                "Values length {} does not match num_tokens ({}) * num_channels ({}) = {}",
                values.len(),
                num_tokens,
                num_channels,
                expected
            )));
        }
        Ok(Self {
            keys,
            values,
            num_tokens,
            num_channels,
        })
    }

    /// Serializes tensor view to bytes using bincode.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| ContextraError::Serialization(e.to_string()))
    }

    /// Deserializes tensor view from bytes using bincode.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| ContextraError::Serialization(e.to_string()))
    }
}

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

    /// Number of tokens.
    #[serde(default)]
    pub num_tokens: usize,
    /// Number of channels.
    #[serde(default)]
    pub num_channels: usize,
    /// Key quantization group size.
    #[serde(default)]
    pub key_group_size: usize,
    /// Whether values are quantized.
    #[serde(default)]
    pub quantize_values: bool,
    /// Per-channel key scaling factors.
    #[serde(default)]
    pub key_scales: Vec<f32>,
    /// Per-channel key zero points.
    #[serde(default)]
    pub key_zero_points: Vec<f32>,
    /// Per-token value scaling factors.
    #[serde(default)]
    pub value_scales: Vec<f32>,
    /// Per-token value zero points.
    #[serde(default)]
    pub value_zero_points: Vec<f32>,
}

/// A 2-bit quantized byte block with accompanying metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KiviQuantizedBlock {
    /// Quantization metadata.
    pub meta: KiviBlockMeta,
    /// Packed 2-bit scalar values (4 values packed per byte, LSB-first).
    pub packed: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct KiviPackedPayload {
    packed_keys: Vec<u8>,
    packed_values: Vec<u8>,
}

/// Quantizes raw `KvTensorView` into 2-bit `KiviQuantizedBlock`.
///
/// Implements asymmetric 2-bit quantization (per-channel for keys, per-token for values, arXiv:2402.02750).
pub fn kivi_quantize(raw: &KvTensorView, config: KiviQuantizeConfig) -> Result<KiviQuantizedBlock> {
    let num_tokens = raw.num_tokens;
    let num_channels = raw.num_channels;
    let group_size = if config.key_group_size == 0 {
        16
    } else {
        config.key_group_size
    };

    if num_tokens == 0 || num_channels == 0 {
        let meta = KiviBlockMeta {
            bits_per_value: 2,
            scale: 0.0,
            zero_point: 0.0,
            original_len: 0,
            num_tokens,
            num_channels,
            key_group_size: group_size,
            quantize_values: config.quantize_values,
            key_scales: Vec::new(),
            key_zero_points: Vec::new(),
            value_scales: Vec::new(),
            value_zero_points: Vec::new(),
        };
        let payload = KiviPackedPayload {
            packed_keys: Vec::new(),
            packed_values: Vec::new(),
        };
        let packed_bytes =
            bincode::serialize(&payload).map_err(|e| ContextraError::Serialization(e.to_string()))?;
        return Ok(KiviQuantizedBlock {
            meta,
            packed: packed_bytes,
        });
    }

    let num_groups = num_tokens.div_ceil(group_size);
    let key_entries = num_groups * num_channels;
    let mut key_scales = Vec::with_capacity(key_entries);
    let mut key_zero_points = Vec::with_capacity(key_entries);

    let key_packed_len = (num_tokens * num_channels).div_ceil(4);
    let mut packed_keys = vec![0u8; key_packed_len];

    // Compute key group scales and zero points
    for g in 0..num_groups {
        let t_start = g * group_size;
        let t_end = (t_start + group_size).min(num_tokens);

        for c in 0..num_channels {
            let mut min_v = f32::INFINITY;
            let mut max_v = f32::NEG_INFINITY;

            for t in t_start..t_end {
                let idx = t * num_channels + c;
                let v = raw.keys[idx];
                if v.is_finite() {
                    min_v = min_v.min(v);
                    max_v = max_v.max(v);
                }
            }

            if !min_v.is_finite() {
                min_v = 0.0;
                max_v = 0.0;
            }

            let (sc, zp) = if (max_v - min_v).abs() < f32::EPSILON {
                (0.0, min_v)
            } else {
                ((max_v - min_v) / 3.0, min_v)
            };

            key_scales.push(sc);
            key_zero_points.push(zp);
        }
    }

    // Quantize keys into row-major 2-bit packed array
    for t in 0..num_tokens {
        let g = t / group_size;
        for c in 0..num_channels {
            let orig_idx = t * num_channels + c;
            let v = raw.keys[orig_idx];
            let meta_idx = g * num_channels + c;
            let sc = key_scales[meta_idx];
            let zp = key_zero_points[meta_idx];

            let code = if sc > 0.0 && v.is_finite() {
                let norm = ((v - zp) / sc).round();
                norm.clamp(0.0, 3.0) as u8
            } else {
                0u8
            };

            let byte_idx = orig_idx / 4;
            let bit_shift = (orig_idx % 4) * 2;
            if let Some(byte) = packed_keys.get_mut(byte_idx) {
                *byte |= (code & 0x03) << bit_shift;
            }
        }
    }

    // Quantize Values per-token
    let mut value_scales = Vec::new();
    let mut value_zero_points = Vec::new();
    let packed_values = if config.quantize_values {
        value_scales.reserve(num_tokens);
        value_zero_points.reserve(num_tokens);

        let val_packed_len = (num_tokens * num_channels).div_ceil(4);
        let mut p_vals = vec![0u8; val_packed_len];

        for t in 0..num_tokens {
            let mut min_v = f32::INFINITY;
            let mut max_v = f32::NEG_INFINITY;

            for c in 0..num_channels {
                let idx = t * num_channels + c;
                let v = raw.values[idx];
                if v.is_finite() {
                    min_v = min_v.min(v);
                    max_v = max_v.max(v);
                }
            }

            if !min_v.is_finite() {
                min_v = 0.0;
                max_v = 0.0;
            }

            let (sc, zp) = if (max_v - min_v).abs() < f32::EPSILON {
                (0.0, min_v)
            } else {
                ((max_v - min_v) / 3.0, min_v)
            };

            value_scales.push(sc);
            value_zero_points.push(zp);

            for c in 0..num_channels {
                let elem_idx = t * num_channels + c;
                let v = raw.values[elem_idx];

                let code = if sc > 0.0 && v.is_finite() {
                    let norm = ((v - zp) / sc).round();
                    norm.clamp(0.0, 3.0) as u8
                } else {
                    0u8
                };

                let byte_idx = elem_idx / 4;
                let bit_shift = (elem_idx % 4) * 2;
                if let Some(byte) = p_vals.get_mut(byte_idx) {
                    *byte |= (code & 0x03) << bit_shift;
                }
            }
        }
        p_vals
    } else {
        bincode::serialize(&raw.values).map_err(|e| ContextraError::Serialization(e.to_string()))?
    };

    let meta = KiviBlockMeta {
        bits_per_value: 2,
        scale: key_scales.first().copied().unwrap_or(0.0),
        zero_point: key_zero_points.first().copied().unwrap_or(0.0),
        original_len: num_tokens * num_channels * 2,
        num_tokens,
        num_channels,
        key_group_size: group_size,
        quantize_values: config.quantize_values,
        key_scales,
        key_zero_points,
        value_scales,
        value_zero_points,
    };

    let payload = KiviPackedPayload {
        packed_keys,
        packed_values,
    };
    let packed = bincode::serialize(&payload)
        .map_err(|e| ContextraError::Serialization(e.to_string()))?;

    Ok(KiviQuantizedBlock { meta, packed })
}

/// Dequantizes packed 2-bit bytes back into `KvTensorView`.
pub fn kivi_dequantize(bytes: &[u8], meta: &KiviBlockMeta) -> Result<KvTensorView> {
    if meta.bits_per_value != 2 {
        return Err(ContextraError::kv_quantization(format!(
            "KIVI dequantization requires bits_per_value == 2, got {}",
            meta.bits_per_value
        )));
    }

    let (active_meta, packed_bytes) = match bincode::deserialize::<KiviQuantizedBlock>(bytes) {
        Ok(block) => (block.meta, block.packed),
        Err(_) => (meta.clone(), bytes.to_vec()),
    };

    if !active_meta.scale.is_finite() || !active_meta.zero_point.is_finite() {
        return Err(ContextraError::kv_quantization(
            "Non-finite key scale or zero_point encountered in metadata".to_string(),
        ));
    }

    let num_tokens = active_meta.num_tokens;
    let num_channels = active_meta.num_channels;

    if num_tokens == 0 || num_channels == 0 {
        return KvTensorView::new(Vec::new(), Vec::new(), num_tokens, num_channels);
    }

    let payload: KiviPackedPayload = match bincode::deserialize(&packed_bytes) {
        Ok(p) => p,
        Err(e) => {
            return Err(ContextraError::kv_quantization(format!(
                "Failed to deserialize KIVI payload bytes: {}",
                e
            )));
        }
    };

    let group_size = if active_meta.key_group_size == 0 {
        16
    } else {
        active_meta.key_group_size
    };

    for sc in &active_meta.key_scales {
        if !sc.is_finite() {
            return Err(ContextraError::kv_quantization(
                "Non-finite key scale encountered in metadata".to_string(),
            ));
        }
    }
    for zp in &active_meta.key_zero_points {
        if !zp.is_finite() {
            return Err(ContextraError::kv_quantization(
                "Non-finite key zero_point encountered in metadata".to_string(),
            ));
        }
    }

    let total_elements = num_tokens * num_channels;
    let mut keys = vec![0.0f32; total_elements];

    for t in 0..num_tokens {
        let g = t / group_size;
        for c in 0..num_channels {
            let orig_idx = t * num_channels + c;
            let meta_idx = g * num_channels + c;
            let sc = active_meta
                .key_scales
                .get(meta_idx)
                .copied()
                .unwrap_or(active_meta.scale);
            let zp = active_meta
                .key_zero_points
                .get(meta_idx)
                .copied()
                .unwrap_or(active_meta.zero_point);

            let byte_idx = orig_idx / 4;
            let bit_shift = (orig_idx % 4) * 2;

            let byte = match payload.packed_keys.get(byte_idx) {
                Some(&b) => b,
                None => {
                    return Err(ContextraError::kv_quantization(format!(
                        "Packed key byte index {} out of bounds (len {})",
                        byte_idx,
                        payload.packed_keys.len()
                    )));
                }
            };

            let code = (byte >> bit_shift) & 0x03;
            keys[orig_idx] = zp + (code as f32) * sc;
        }
    }

    let values = if active_meta.quantize_values {
        let mut vals = vec![0.0f32; total_elements];
        for t in 0..num_tokens {
            let sc = active_meta
                .value_scales
                .get(t)
                .copied()
                .unwrap_or(active_meta.scale);
            let zp = active_meta
                .value_zero_points
                .get(t)
                .copied()
                .unwrap_or(active_meta.zero_point);

            for c in 0..num_channels {
                let elem_idx = t * num_channels + c;
                let byte_idx = elem_idx / 4;
                let bit_shift = (elem_idx % 4) * 2;

                let byte = match payload.packed_values.get(byte_idx) {
                    Some(&b) => b,
                    None => {
                        return Err(ContextraError::kv_quantization(format!(
                            "Packed value byte index {} out of bounds (len {})",
                            byte_idx,
                            payload.packed_values.len()
                        )));
                    }
                };

                let code = (byte >> bit_shift) & 0x03;
                let v = zp + (code as f32) * sc;
                vals[elem_idx] = v;
            }
        }
        vals
    } else {
        match bincode::deserialize::<Vec<f32>>(&payload.packed_values) {
            Ok(v) => v,
            Err(e) => {
                return Err(ContextraError::kv_quantization(format!(
                    "Failed to deserialize raw value floats: {}",
                    e
                )));
            }
        }
    };

    KvTensorView::new(keys, values, num_tokens, num_channels)
}

/// Dequantizes a 2-bit packed block back into floating point `Vec<f32>`.
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
pub fn pack_kivi_block(values: &[f32]) -> KiviQuantizedBlock {
    if values.is_empty() {
        return KiviQuantizedBlock {
            meta: KiviBlockMeta {
                bits_per_value: 2,
                scale: 0.0,
                zero_point: 0.0,
                original_len: 0,
                num_tokens: 0,
                num_channels: 0,
                key_group_size: 16,
                quantize_values: true,
                key_scales: Vec::new(),
                key_zero_points: Vec::new(),
                value_scales: Vec::new(),
                value_zero_points: Vec::new(),
            },
            packed: Vec::new(),
        };
    }

    let finite_vals: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let (min_v, max_v) = if !finite_vals.is_empty() {
        let min_val = finite_vals.iter().copied().fold(f32::INFINITY, f32::min);
        let max_val = finite_vals
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
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
            num_tokens: 1,
            num_channels: values.len(),
            key_group_size: 16,
            quantize_values: true,
            key_scales: vec![scale],
            key_zero_points: vec![zero_point],
            value_scales: Vec::new(),
            value_zero_points: Vec::new(),
        },
        packed,
    }
}

/// Losslessly compresses byte slice for LSM spill (Quantization -> Compression -> AEAD).
pub fn compress_bytes(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return vec![0x55];
    }
    let mut compressed = Vec::with_capacity(input.len());
    compressed.push(0x43); // 'C' header

    let mut i = 0;
    while i < input.len() {
        let byte = input[i];
        let mut run_len = 1u8;
        while i + (run_len as usize) < input.len()
            && input[i + (run_len as usize)] == byte
            && run_len < 255
        {
            run_len += 1;
        }
        compressed.push(run_len);
        compressed.push(byte);
        i += run_len as usize;
    }

    if compressed.len() >= input.len() {
        let mut raw = Vec::with_capacity(1 + input.len());
        raw.push(0x55); // 'U' header
        raw.extend_from_slice(input);
        raw
    } else {
        compressed
    }
}

/// Losslessly decompresses byte slice produced by `compress_bytes`.
pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(ContextraError::kv_quantization(
            "Empty compression payload".to_string(),
        ));
    }
    match input[0] {
        0x55 => Ok(input[1..].to_vec()),
        0x43 => {
            let mut decompressed = Vec::new();
            let mut i = 1;
            while i + 1 < input.len() {
                let count = input[i] as usize;
                let byte = input[i + 1];
                decompressed.resize(decompressed.len() + count, byte);
                i += 2;
            }
            Ok(decompressed)
        }
        other => Err(ContextraError::kv_quantization(format!(
            "Invalid compression header byte 0x{:02x}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_kivi_metadata_validation_errors_no_panic() {
        let meta_wrong_bits = KiviBlockMeta {
            bits_per_value: 4,
            scale: 1.0,
            zero_point: 0.0,
            original_len: 4,
            num_tokens: 1,
            num_channels: 4,
            key_group_size: 16,
            quantize_values: true,
            key_scales: vec![1.0],
            key_zero_points: vec![0.0],
            value_scales: vec![],
            value_zero_points: vec![],
        };
        let block_wrong_bits = KiviQuantizedBlock {
            meta: meta_wrong_bits,
            packed: vec![0x00],
        };
        assert!(unpack_kivi_block(&block_wrong_bits).is_err());
    }

    #[test]
    fn test_kivi_quantize_dequantize_roundtrip_simple() {
        let raw = KvTensorView::new(
            vec![1.0, 2.0, 3.0, 4.0],
            vec![10.0, 20.0, 30.0, 40.0],
            2,
            2,
        )
        .unwrap();

        let config = KiviQuantizeConfig {
            key_group_size: 16,
            quantize_values: true,
        };

        let block = kivi_quantize(&raw, config).unwrap();
        let decompressed = kivi_dequantize(&block.packed, &block.meta).unwrap();

        assert_eq!(decompressed.num_tokens, 2);
        assert_eq!(decompressed.num_channels, 2);
        assert_eq!(decompressed.keys.len(), 4);
        assert_eq!(decompressed.values.len(), 4);
    }

    #[test]
    fn test_compress_decompress_bytes_roundtrip() {
        let sample = vec![0x00, 0x00, 0x00, 0x00, 0xAA, 0xBB, 0xCC, 0xDD];
        let compressed = compress_bytes(&sample);
        let decompressed = decompress_bytes(&compressed).unwrap();
        assert_eq!(sample, decompressed);
    }

    proptest! {
        #[test]
        fn prop_compression_roundtrip(vec in proptest::collection::vec(0u8..255u8, 0..100)) {
            let compressed = compress_bytes(&vec);
            let decompressed = decompress_bytes(&compressed).unwrap();
            prop_assert_eq!(vec, decompressed);
        }
    }
}
