#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Generator für den Golden-Korpus von `contextra-kvcache` (KV3.1).

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use contextra_kvcache::attention_score::{rank_for_eviction_weighted, NullAttentionScoreSource};
#[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
use contextra_kvcache::quantize_kivi::{
    kivi_dequantize, kivi_quantize, KiviQuantizeConfig, KvTensorView,
};
use contextra_kvcache::radix::{KvReusePolicy, PrefixRadixTree};
use contextra_types::TenantId;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

#[test]
#[ignore]
fn generate_all_golden() {
    let dir = golden_dir();
    fs::create_dir_all(&dir).expect("create golden dir");

    // 1. Radix-Matching deterministischer Output
    let tenant = TenantId::try_new(42).unwrap();
    let mut tree = PrefixRadixTree::new(tenant);
    let tokens_a = vec![101, 102, 103, 104, 105];
    let tokens_b = vec![101, 102, 103, 204, 205];
    tree.insert(&tokens_a, 10).unwrap();
    tree.insert(&tokens_b, 20).unwrap();

    let match_a = tree
        .find_longest_prefix(&tokens_a, KvReusePolicy::Always)
        .unwrap();
    let radix_json = serde_json::to_string_pretty(&serde_json::json!({
        "matched_tokens": match_a.matched_tokens,
        "matched_len": match_a.matched_len,
        "block_id": match_a.block_id,
    }))
    .unwrap();
    fs::write(dir.join("radix_match.json"), radix_json).unwrap();

    // 2. KIVI Quantisierung & Packing
    #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
    {
        let keys = vec![
            -1.5, -0.5, 0.5, 1.5, -2.0, -1.0, 1.0, 2.0, 0.0, 0.25, 0.5, 0.75, -0.75, -0.25, 0.25,
            0.75,
        ];
        let values = vec![
            1.0, 2.0, 3.0, 4.0, -1.0, -2.0, -3.0, -4.0, 0.5, 0.5, 0.5, 0.5, -0.5, -0.5, -0.5, -0.5,
        ];
        let raw = KvTensorView::new(keys, values, 4, 4).unwrap();
        let config = KiviQuantizeConfig {
            key_group_size: 16,
            quantize_values: true,
        };
        let block = kivi_quantize(&raw, config).unwrap();
        let recon = kivi_dequantize(&block.packed, &block.meta).unwrap();

        fs::write(dir.join("kivi_packed.bin"), &block.packed).unwrap();
        fs::write(
            dir.join("kivi_meta.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "num_tokens": block.meta.num_tokens,
                "num_channels": block.meta.num_channels,
                "bits_per_value": block.meta.bits_per_value,
                "original_len": block.meta.original_len,
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            dir.join("kivi_dequantized.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "num_tokens": recon.num_tokens,
                "num_channels": recon.num_channels,
                "keys": recon.keys,
                "values": recon.values,
            }))
            .unwrap(),
        )
        .unwrap();
    }

    // 3. Eviction-Rangfolge (deterministischer LRU-Ablauf)
    let base_instant = Instant::now();
    let candidates = vec![
        (10u64, base_instant + Duration::from_millis(50)),
        (5u64, base_instant + Duration::from_millis(80)),
        (20u64, base_instant + Duration::from_millis(50)),
        (2u64, base_instant + Duration::from_millis(10)),
    ];
    let null_source = NullAttentionScoreSource;
    let ranked = rank_for_eviction_weighted(&candidates, &null_source, 0.0);
    let ranked_json = serde_json::to_string_pretty(&ranked).unwrap();
    fs::write(dir.join("eviction_ranking.json"), ranked_json).unwrap();

    println!("Golden files successfully generated in {:?}", dir);
}
