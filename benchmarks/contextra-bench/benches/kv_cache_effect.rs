// FILE-CONTEXT
// ZWECK: Benchmark-Suite zur Messung des KV-Cache-Effekts (Latenz, Throughput, Hit-Rate, Quantisierung, Encryption, Evictions).
// INVARIANTEN: Deterministic Seed, Zero-Panic, Criterion Benchmark Harness.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::inconsistent_digit_grouping
)]

use contextra_core::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_core::model_fingerprint::ModelFingerprint;
use contextra_core::TenantId;
use contextra_crypto::kv_cipher::KvSegmentCipher;
use contextra_crypto::KeyManager;
use contextra_kvcache::{kivi_quantize, KiviQuantizeConfig, KvTensorView, TenantPrefixKvStore};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::{Duration, Instant};

/// Synthetic Dummy LLM model simulator (`DummyLlmModel`).
/// Simulates prefill transformer matrix operations per unhit token using active CPU math
/// (~100 ns per token for matrix MAC simulation).
struct DummyLlmModel {
    per_token_macs: usize,
}

impl DummyLlmModel {
    fn new() -> Self {
        Self {
            per_token_macs: 250,
        }
    }

    #[inline(never)]
    fn simulate_prefill_compute(&self, unhit_tokens: usize) {
        let total_ops = unhit_tokens * self.per_token_macs;
        let mut acc = 1.0f32;
        for i in 0..total_ops {
            acc = acc.mul_add(1.00001f32, (i as f32) * 0.0001f32);
        }
        black_box(acc);
    }
}

fn sample_prefix_key() -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0xcd; 32], "dummy-llm-7b", "F16"),
        tokenizer_hash: [0x12; 32],
        layout: KvLayout {
            n_layer: 32,
            n_kv_head: 8,
            head_dim: 128,
            dtype: "f16".into(),
        },
        rope: RopeConfig {
            base: 10000.0,
            scaling: None,
        },
    }
}

// ---------------------------------------------------------------------------
// (a) Latenz und Tokens/s der Vorbefüllung (Prefill) bei 1k/4k/16k Token
// ---------------------------------------------------------------------------
fn bench_prefill_latency_and_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("a_prefill_latency_and_throughput");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_secs(1));

    let tenant = TenantId::try_new(1).unwrap();
    let prefix_key = sample_prefix_key();
    let model = DummyLlmModel::new();

    let token_lengths = [1000usize, 4000usize, 16000usize];

    for &len in &token_lengths {
        group.throughput(Throughput::Elements(len as u64));

        let tokens: Vec<u32> = (1..=(len as u32)).collect();
        let block_bytes = vec![0u8; len * 32 * 8 * 128 * 2 / 100]; // Simulated KV tensor block
        let block = KvBlock {
            block_id: 1,
            data: bytes::Bytes::from(block_bytes),
        };

        // Cache Miss Scenario
        group.bench_function(BenchmarkId::new("Miss", len), |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::ZERO;
                for _ in 0..iters {
                    let store = TenantPrefixKvStore::new();
                    let start = Instant::now();
                    // Lookup misses
                    let hit = store.lookup(tenant, &prefix_key, &tokens);
                    assert!(hit.is_none());

                    // Compute prefill for all tokens
                    model.simulate_prefill_compute(len);

                    // Insert into cache
                    store
                        .insert(tenant, &prefix_key, &tokens, vec![block.clone()])
                        .unwrap();

                    total_duration += start.elapsed();
                }
                total_duration
            });
        });

        // Cache Hit Scenario (Warm Store with exact 100% prefix match)
        let warm_store = TenantPrefixKvStore::new();
        warm_store
            .insert(tenant, &prefix_key, &tokens, vec![block.clone()])
            .unwrap();

        group.bench_function(BenchmarkId::new("Hit", len), |b| {
            b.iter(|| {
                let start = Instant::now();
                let hit = warm_store.lookup(tenant, &prefix_key, &tokens);
                let hit_len = hit.as_ref().map(|h| h.matched_tokens).unwrap_or(0);
                let unhit_tokens = len - hit_len;

                if unhit_tokens > 0 {
                    model.simulate_prefill_compute(unhit_tokens);
                }
                let elapsed = start.elapsed();
                black_box(hit);
                elapsed
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// (b) Trefferquote bei einer Wiederholungs-Verteilung (50 Prompts, 70% teilen ein Präfix)
// ---------------------------------------------------------------------------
fn bench_prefix_hit_rate_distribution(c: &mut Criterion) {
    let mut group = c.benchmark_group("b_prefix_hit_rate_distribution");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_secs(1));

    let tenant = TenantId::try_new(2).unwrap();
    let prefix_key = sample_prefix_key();

    // 50 prompts, 70% (35 prompts) share a 500-token shared prefix
    let total_prompts = 50usize;
    let shared_count = 35usize;
    let shared_prefix: Vec<u32> = (1..=500).collect();

    let mut prompts: Vec<Vec<u32>> = Vec::with_capacity(total_prompts);
    let mut rng = StdRng::seed_from_u64(42);

    for i in 0..total_prompts {
        let mut prompt = if i < shared_count {
            shared_prefix.clone()
        } else {
            // Unique prefix
            (10000..10500).map(|x| x + (i as u32) * 500).collect()
        };
        // Add random suffix of 100 tokens per prompt
        for _ in 0..100 {
            prompt.push(rng.gen_range(20000..30000));
        }
        prompts.push(prompt);
    }

    group.bench_function("50_prompts_70pct_shared_prefix", |b| {
        b.iter(|| {
            let store = TenantPrefixKvStore::new();
            let mut total_hits = 0usize;
            let mut total_matched_tokens = 0usize;
            let mut total_prompt_tokens = 0usize;

            for (idx, prompt) in prompts.iter().enumerate() {
                total_prompt_tokens += prompt.len();
                if let Some(hit) = store.lookup(tenant, &prefix_key, prompt) {
                    if hit.matched_tokens > 0 {
                        total_hits += 1;
                        total_matched_tokens += hit.matched_tokens;
                    }
                }

                // Insert into cache post-lookup
                let block = KvBlock {
                    block_id: idx as u64,
                    data: bytes::Bytes::from(vec![0u8; 1024]),
                };
                let _ = store.insert(tenant, &prefix_key, prompt, vec![block]);
            }

            black_box((total_hits, total_matched_tokens, total_prompt_tokens));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// (c) Speicherbedarf je Segment vor/nach KIVI 2-Bit Quantisierung
// ---------------------------------------------------------------------------
fn bench_kivi_quantization_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("c_kivi_quantization_footprint");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_secs(1));

    // Simulated KV segment tensors (128 channels, 512 tokens)
    let num_tokens = 512;
    let num_channels = 128;
    let mut rng = StdRng::seed_from_u64(12345);
    let mut key_data: Vec<f32> = Vec::with_capacity(num_tokens * num_channels);
    for _ in 0..(num_tokens * num_channels) {
        key_data.push(rng.gen_range(-2.0f32..2.0f32));
    }
    let value_data = key_data.clone();

    let view = KvTensorView {
        keys: key_data,
        values: value_data,
        num_tokens,
        num_channels,
    };

    let cfg = KiviQuantizeConfig {
        key_group_size: 32,
        quantize_values: true,
    };

    group.bench_function("kivi_2bit_quantize_512x128", |b| {
        b.iter(|| {
            let quantized = kivi_quantize(&view, cfg).expect("kivi_quantize failed");
            black_box(quantized);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// (d) Overhead der AEAD-Verschlüsselung je MB
// ---------------------------------------------------------------------------
fn bench_aead_encryption_overhead_per_mb(c: &mut Criterion) {
    let mut group = c.benchmark_group("d_aead_encryption_overhead_per_mb");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_secs(1));

    let payload_1mb = vec![0x5a_u8; 1_024_1024]; // 1 MB payload
    let km = KeyManager::try_new("benchmark-secret-passphrase", b"salt-1234")
        .expect("KeyManager init failed");
    let cipher = KvSegmentCipher::new(km);
    let tenant_id = TenantId::try_new(42).unwrap();
    let model_fp = sample_prefix_key().model;

    group.throughput(Throughput::Bytes(1_024_1024));

    group.bench_function("encrypt_1mb", |b| {
        b.iter(|| {
            let layer = cipher
                .encrypt(tenant_id, model_fp.clone(), &payload_1mb)
                .expect("Encryption failed");
            black_box(layer);
        });
    });

    let layer = cipher
        .encrypt(tenant_id, model_fp, &payload_1mb)
        .expect("Encryption failed");

    group.bench_function("decrypt_1mb", |b| {
        b.iter(|| {
            let plaintext = cipher.decrypt(&layer).expect("Decryption failed");
            black_box(plaintext);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// (e) Evictions unter Budget
// ---------------------------------------------------------------------------
fn bench_eviction_under_budget(c: &mut Criterion) {
    let mut group = c.benchmark_group("e_eviction_under_budget");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(200));
    group.measurement_time(Duration::from_secs(1));

    let tenant = TenantId::try_new(99).unwrap();
    let prefix_key = sample_prefix_key();

    // Set tight budget: 1 MB
    let budget_bytes = 1024 * 1024;
    let block_size = 100 * 1024; // 100 KB per block
    let block = KvBlock {
        block_id: 1,
        data: bytes::Bytes::from(vec![0u8; block_size]),
    };

    group.bench_function("evict_lru_when_budget_exceeded", |b| {
        b.iter(|| {
            let store = TenantPrefixKvStore::new().with_byte_budget_per_tenant(budget_bytes);

            // Insert 15 blocks (1.5 MB total), triggering LRU eviction down to 1 MB limit (10 blocks max)
            for i in 0..15u32 {
                let tokens = vec![i + 1, i + 2, i + 3];
                store
                    .insert(tenant, &prefix_key, &tokens, vec![block.clone()])
                    .unwrap();
            }

            black_box(store);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_prefill_latency_and_throughput,
    bench_prefix_hit_rate_distribution,
    bench_kivi_quantization_footprint,
    bench_aead_encryption_overhead_per_mb,
    bench_eviction_under_budget
);
criterion_main!(benches);
