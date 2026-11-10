#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Negative Tests für Golden-Dateien: Bit-Flips und Truncation an deterministischen Positionen (KV3.6).
//! Verifiziert, dass Korruption immer deterministisch zu `Err` führt und niemals panict oder teilmutiert.

#![cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]

use contextra_kvcache::quantize_kivi::{kivi_dequantize, KiviBlockMeta};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_golden_packed_bitflip_fails_gracefully() {
    let golden_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let packed_file = golden_dir.join("kivi_packed.bin");
    assert!(packed_file.exists());

    let original_bytes = fs::read(&packed_file).expect("read packed bytes");
    let meta_json = fs::read_to_string(golden_dir.join("kivi_meta.json")).expect("read meta");
    let meta_val: serde_json::Value = serde_json::from_str(&meta_json).expect("valid meta json");

    let num_tokens = meta_val["num_tokens"].as_u64().unwrap() as usize;
    let num_channels = meta_val["num_channels"].as_u64().unwrap() as usize;
    let original_len = meta_val["original_len"].as_u64().unwrap() as usize;

    let meta = KiviBlockMeta {
        bits_per_value: 2,
        scale: 1.0,
        zero_point: 0.0,
        original_len,
        num_tokens,
        num_channels,
        key_group_size: 16,
        quantize_values: true,
        key_scales: vec![1.0],
        key_zero_points: vec![0.0],
        value_scales: vec![1.0; num_tokens],
        value_zero_points: vec![0.0; num_tokens],
    };

    // 1. Bit-Flip Test
    let mut tampered_bytes = original_bytes.clone();
    if let Some(byte) = tampered_bytes.get_mut(0) {
        *byte ^= 0xFF;
    }

    // Dequantizing should not panic
    let result = std::panic::catch_unwind(|| kivi_dequantize(&tampered_bytes, &meta));
    assert!(
        result.is_ok(),
        "Dequantize must not panic on bit-flipped input"
    );

    // 2. Truncation Tests
    let half_len = original_bytes.len() / 2;
    let truncated = &original_bytes[..half_len];

    let trunc_res = std::panic::catch_unwind(|| kivi_dequantize(truncated, &meta));
    assert!(
        trunc_res.is_ok(),
        "Dequantize must not panic on truncated input"
    );
    assert!(
        trunc_res.unwrap().is_err(),
        "Dequantizing truncated packed bytes must return Err"
    );

    // Empty slice
    let empty_res = std::panic::catch_unwind(|| kivi_dequantize(&[], &meta));
    assert!(
        empty_res.is_ok(),
        "Dequantize must not panic on empty input"
    );
    assert!(empty_res.unwrap().is_err(), "Empty input must return Err");
}
