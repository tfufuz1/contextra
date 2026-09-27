#![forbid(unsafe_code)]

use contextra_kvcache::{
    kivi_dequantize, kivi_quantize, KiviBlockMeta, KiviQuantizeConfig, KiviQuantizedBlock,
    KvTensorView,
};
use contextra_types::ContextraError;
use proptest::prelude::*;

#[test]
fn test_kivi_empty_and_zero_tensors_roundtrip() {
    let empty_raw = KvTensorView::new(Vec::new(), Vec::new(), 0, 0).unwrap();
    let config = KiviQuantizeConfig::default();

    let block = kivi_quantize(&empty_raw, config).unwrap();
    let recon = kivi_dequantize(&block.packed, &block.meta).unwrap();
    assert_eq!(recon.num_tokens, 0);
    assert_eq!(recon.num_channels, 0);
    assert!(recon.keys.is_empty());
    assert!(recon.values.is_empty());

    let zeros_raw = KvTensorView::new(vec![0.0; 16], vec![0.0; 16], 4, 4).unwrap();
    let block_zeros = kivi_quantize(&zeros_raw, config).unwrap();
    let recon_zeros = kivi_dequantize(&block_zeros.packed, &block_zeros.meta).unwrap();
    assert_eq!(recon_zeros.num_tokens, 4);
    assert_eq!(recon_zeros.num_channels, 4);
    for k in recon_zeros.keys {
        assert_eq!(k, 0.0);
    }
    for v in recon_zeros.values {
        assert_eq!(v, 0.0);
    }
}

#[test]
fn test_kivi_dequantize_invalid_metadata_returns_kv_quantization_error() {
    let invalid_bits_meta = KiviBlockMeta {
        bits_per_value: 4, // Invalid for KIVI (must be 2)
        scale: 1.0,
        zero_point: 0.0,
        original_len: 16,
        num_tokens: 2,
        num_channels: 4,
        key_group_size: 16,
        quantize_values: true,
        key_scales: vec![1.0],
        key_zero_points: vec![0.0],
        value_scales: vec![1.0, 1.0],
        value_zero_points: vec![0.0, 0.0],
    };

    let res = kivi_dequantize(&[0x00], &invalid_bits_meta);
    assert!(res.is_err());
    match res.unwrap_err() {
        ContextraError::KvQuantization(msg) => {
            assert!(msg.contains("bits_per_value == 2"));
        }
        err => panic!("Expected KvQuantization error, got {:?}", err),
    }

    let non_finite_meta = KiviBlockMeta {
        bits_per_value: 2,
        scale: f32::NAN,
        zero_point: 0.0,
        original_len: 16,
        num_tokens: 2,
        num_channels: 4,
        key_group_size: 16,
        quantize_values: true,
        key_scales: vec![f32::NAN],
        key_zero_points: vec![0.0],
        value_scales: vec![1.0, 1.0],
        value_zero_points: vec![0.0, 0.0],
    };

    let block = KiviQuantizedBlock {
        meta: non_finite_meta.clone(),
        packed: vec![0x00; 4],
    };
    let serialized_block = bincode::serialize(&block).unwrap();
    let res_finite = kivi_dequantize(&serialized_block, &non_finite_meta);
    assert!(res_finite.is_err());
    match res_finite.unwrap_err() {
        ContextraError::KvQuantization(msg) => {
            assert!(msg.contains("Non-finite"));
        }
        err => panic!("Expected KvQuantization error, got {:?}", err),
    }
}

proptest! {
    /// Property test: Quantization error bound verification for KIVI 2-bit asymmetric quantization.
    ///
    /// Theoretical error bound reasoning:
    /// For a 2-bit linear uniform quantizer mapping [min_val, max_val] into 4 discrete levels {0, 1, 2, 3},
    /// quantization step size delta = (max_val - min_val) / 3.
    /// Maximum distance from any scalar value to its nearest quantization level is delta / 2 = (max_val - min_val) / 6.0.
    /// With floating point rounding tolerance, error |v_orig - v_recon| <= (max_val - min_val) / 6.0 + 1e-3.
    #[test]
    fn prop_kivi_asymmetric_quantize_dequantize_roundtrip_error_bound(
        num_tokens in 1usize..8,
        num_channels in 1usize..8,
        keys in proptest::collection::vec(-500.0f32..500.0f32, 1..64),
        values in proptest::collection::vec(-500.0f32..500.0f32, 1..64),
    ) {
        let total_len = num_tokens * num_channels;
        let mut keys_adj = keys;
        keys_adj.resize(total_len, 1.0);
        let mut values_adj = values;
        values_adj.resize(total_len, 2.0);

        let raw = KvTensorView::new(keys_adj.clone(), values_adj.clone(), num_tokens, num_channels).unwrap();
        let config = KiviQuantizeConfig {
            key_group_size: 16,
            quantize_values: true,
        };

        let block = kivi_quantize(&raw, config).unwrap();
        let recon = kivi_dequantize(&block.packed, &block.meta).unwrap();

        prop_assert_eq!(recon.num_tokens, num_tokens);
        prop_assert_eq!(recon.num_channels, num_channels);

        // Verify keys error bound per channel group
        for g in 0..num_tokens.div_ceil(16) {
            let t_start = g * 16;
            let t_end = (t_start + 16).min(num_tokens);

            for c in 0..num_channels {
                let mut min_k = f32::INFINITY;
                let mut max_k = f32::NEG_INFINITY;
                for t in t_start..t_end {
                    let idx = t * num_channels + c;
                    min_k = min_k.min(keys_adj[idx]);
                    max_k = max_k.max(keys_adj[idx]);
                }
                let delta_k = max_k - min_k;
                let allowed_err_k = delta_k / 6.0 + 1e-3;

                for t in t_start..t_end {
                    let idx = t * num_channels + c;
                    let err = (keys_adj[idx] - recon.keys[idx]).abs();
                    prop_assert!(
                        err <= allowed_err_k,
                        "Key error {} exceeded allowed bound {} for orig={} recon={}",
                        err, allowed_err_k, keys_adj[idx], recon.keys[idx]
                    );
                }
            }
        }

        // Verify values error bound per token
        for t in 0..num_tokens {
            let mut min_v = f32::INFINITY;
            let mut max_v = f32::NEG_INFINITY;
            for c in 0..num_channels {
                let idx = t * num_channels + c;
                min_v = min_v.min(values_adj[idx]);
                max_v = max_v.max(values_adj[idx]);
            }
            let delta_v = max_v - min_v;
            let allowed_err_v = delta_v / 6.0 + 1e-3;

            for c in 0..num_channels {
                let idx = t * num_channels + c;
                let err = (values_adj[idx] - recon.values[idx]).abs();
                prop_assert!(
                    err <= allowed_err_v,
                    "Value error {} exceeded allowed bound {} for orig={} recon={}",
                    err, allowed_err_v, values_adj[idx], recon.values[idx]
                );
            }
        }
    }
}
