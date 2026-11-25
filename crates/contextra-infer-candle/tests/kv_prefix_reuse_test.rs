// FILE-CONTEXT
// STAND: 2026-09-22T00:00:00Z
// ZWECK: Integration test suite for KV cache stage B prefix reuse, counters, and tenant isolation (Spec §9.2).
// INVARIANTEN: No .unwrap() / .expect() in production code; `#![cfg(feature = "kv-stage-b")]` test gates.

#![cfg(feature = "kv-stage-b")]

use candle_core::Device;
use contextra_infer_candle::{
    CandleLlmClient, CandleModelInner, KvState, LayerKv, PrefixRun, PrefixSeed, QuantizedLlamaModel,
};
use contextra_ports::kv::{KvBlock, KvLayout, KvPrefixHit, KvPrefixStore, RopeConfig};
use contextra_ports::ContextSegment;
use contextra_ports::LlmTextGenerator;
use contextra_types::{ContextraError, ModelFingerprint, Result, TenantId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type CacheKey = (TenantId, contextra_ports::kv::PrefixKey);
type CacheValue = (Vec<u32>, Vec<KvBlock>);

/// In-memory test-double for `KvPrefixStore` with exact longest prefix matching per tenant.
#[derive(Default)]
struct InMemoryKvPrefixStore {
    entries: Mutex<HashMap<CacheKey, CacheValue>>,
}

impl KvPrefixStore for InMemoryKvPrefixStore {
    fn lookup(
        &self,
        tenant: TenantId,
        key: &contextra_ports::kv::PrefixKey,
        tokens: &[u32],
    ) -> Option<KvPrefixHit> {
        let guard = self.entries.lock().ok()?;
        let (cached_tokens, blocks) = guard.get(&(tenant, key.clone()))?;

        let match_len = tokens
            .iter()
            .zip(cached_tokens.iter())
            .take_while(|(a, b)| a == b)
            .count();

        if match_len == 0 {
            None
        } else {
            Some(KvPrefixHit {
                matched_tokens: match_len,
                blocks: blocks.clone(),
            })
        }
    }

    fn insert(
        &self,
        tenant: TenantId,
        key: &contextra_ports::kv::PrefixKey,
        tokens: &[u32],
        blocks: Vec<KvBlock>,
    ) -> Result<()> {
        let mut guard = self
            .entries
            .lock()
            .map_err(|_| ContextraError::Internal("Lock poisoned".into()))?;
        guard.insert((tenant, key.clone()), (tokens.to_vec(), blocks));
        Ok(())
    }

    fn evict(&self, tenant: TenantId, key: &contextra_ports::kv::PrefixKey) -> Result<u64> {
        let mut guard = self
            .entries
            .lock()
            .map_err(|_| ContextraError::Internal("Lock poisoned".into()))?;
        if let Some((_, blocks)) = guard.remove(&(tenant, key.clone())) {
            Ok(blocks.len() as u64)
        } else {
            Ok(0)
        }
    }
}

/// Mock model tracking prefix seeds and returning deterministic tokens.
struct MockPrefixModel {
    last_seed: Option<PrefixSeed>,
    fail_on_import: bool,
}

impl MockPrefixModel {
    fn new(fail_on_import: bool) -> Self {
        Self {
            last_seed: None,
            fail_on_import,
        }
    }
}

impl CandleModelInner for MockPrefixModel {
    fn generate(
        &mut self,
        prompt: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
    ) -> Result<String> {
        Ok(format!("[MockPrefixModel] Response for: {prompt}"))
    }

    fn kv_layout(&self) -> Option<(KvLayout, RopeConfig)> {
        Some((
            KvLayout {
                n_layer: 1,
                n_kv_head: 1,
                head_dim: 4,
                dtype: "f16".to_string(),
            },
            RopeConfig {
                base: 10000.0,
                scaling: None,
            },
        ))
    }

    fn generate_stream_with_prefix(
        &mut self,
        prompt: &str,
        tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
        seed: Option<PrefixSeed>,
        _on_token: &mut dyn FnMut(String) -> bool,
    ) -> Result<PrefixRun> {
        self.last_seed = seed.clone();
        let tokens = tokenizer
            .encode(prompt, true)
            .map_err(|e| ContextraError::InvalidInput(format!("Tokenizer error: {e}")))?;
        let prompt_tokens = tokens.get_ids().to_vec();

        let mut reused_tokens = 0;
        if let Some(s) = seed {
            if self.fail_on_import {
                reused_tokens = 0;
            } else {
                reused_tokens = s.matched_len;
            }
        }

        let len = prompt_tokens.len();
        let dummy_state = KvState::new(
            vec![LayerKv::new(
                candle_core::Tensor::zeros((1, 1, len, 4), candle_core::DType::F16, &Device::Cpu)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?,
                candle_core::Tensor::zeros((1, 1, len, 4), candle_core::DType::F16, &Device::Cpu)
                    .map_err(|e| ContextraError::Internal(e.to_string()))?,
            )],
            len,
        );

        let exported_prefix = dummy_state.export_block(0..prompt_tokens.len()).ok();

        Ok(PrefixRun {
            output: format!(
                "[MockPrefixModel] Generated text for prompt of len {}",
                prompt_tokens.len()
            ),
            prompt_tokens,
            reused_tokens,
            exported_prefix,
        })
    }
}

fn create_test_client(
    model: MockPrefixModel,
    store: Option<Arc<dyn KvPrefixStore>>,
) -> Result<CandleLlmClient> {
    let device = Device::Cpu;
    let fingerprint = ModelFingerprint {
        hash: [1u8; 32],
        model_id: "test-llama".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": [],
        "normalizer": null,
        "pre_tokenizer": { "type": "Whitespace" },
        "post_processor": null,
        "decoder": null,
        "model": { "type": "WordLevel", "vocab": {"Hello": 0, "world": 1, "test": 2, "prefix": 3, "reuse": 4}, "unk_token": "[UNK]" }
    }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
        .map_err(|e| ContextraError::Internal(e.to_string()))?;

    let mut client = CandleLlmClient::new(device, Box::new(model), fingerprint, tokenizer);
    if let Some(s) = store {
        client = client.with_prefix_store(s);
    }
    Ok(client)
}

#[tokio::test]
async fn test_prefix_reuse_cache_miss_and_hit_cycle() -> Result<()> {
    let store = Arc::new(InMemoryKvPrefixStore::default());
    let model = MockPrefixModel::new(false);
    let client = create_test_client(model, Some(store.clone()))?;

    let tenant = TenantId::try_new(100).map_err(|e| ContextraError::InvalidInput(e.to_string()))?;
    let segs = vec![ContextSegment {
        text: "Hello world test",
        chunk_id: 1,
        model_fingerprint: None,
        rope_offset: Some(0),
    }];

    // First call: Miss -> Full prefill -> Store insert
    let out1 = client.generate_with_context(tenant, &segs).await?;
    assert!(!out1.is_empty());
    assert_eq!(client.prefill_count(), 1);
    assert_eq!(client.prefill_skip_count(), 0);
    assert_eq!(client.prefill_skipped_tokens(), 0);

    // Second call with same prompt: Hit -> Skip prefill
    let out2 = client.generate_with_context(tenant, &segs).await?;
    assert!(!out2.is_empty());
    assert_eq!(client.prefill_count(), 2);
    assert_eq!(client.prefill_skip_count(), 1);
    assert!(client.prefill_skipped_tokens() > 0);

    Ok(())
}

#[tokio::test]
async fn test_prefix_reuse_longer_suffix_continuation() -> Result<()> {
    let store = Arc::new(InMemoryKvPrefixStore::default());
    let model = MockPrefixModel::new(false);
    let client = create_test_client(model, Some(store.clone()))?;

    let tenant = TenantId::try_new(101).map_err(|e| ContextraError::InvalidInput(e.to_string()))?;
    let segs_short = vec![ContextSegment {
        text: "Hello world",
        chunk_id: 1,
        model_fingerprint: None,
        rope_offset: Some(0),
    }];

    client.generate_with_context(tenant, &segs_short).await?;

    let segs_long = vec![
        ContextSegment {
            text: "Hello world",
            chunk_id: 1,
            model_fingerprint: None,
            rope_offset: Some(0),
        },
        ContextSegment {
            text: "test prefix reuse",
            chunk_id: 2,
            model_fingerprint: None,
            rope_offset: Some(0),
        },
    ];

    client.generate_with_context(tenant, &segs_long).await?;
    assert_eq!(client.prefill_skip_count(), 1);

    Ok(())
}

#[tokio::test]
async fn test_prefix_reuse_tenant_isolation() -> Result<()> {
    let store = Arc::new(InMemoryKvPrefixStore::default());
    let model = MockPrefixModel::new(false);
    let client = create_test_client(model, Some(store.clone()))?;

    let tenant_a =
        TenantId::try_new(10).map_err(|e| ContextraError::InvalidInput(e.to_string()))?;
    let tenant_b =
        TenantId::try_new(20).map_err(|e| ContextraError::InvalidInput(e.to_string()))?;

    let segs = vec![ContextSegment {
        text: "Hello world test",
        chunk_id: 1,
        model_fingerprint: None,
        rope_offset: Some(0),
    }];

    // Tenant A populates cache
    client.generate_with_context(tenant_a, &segs).await?;
    assert_eq!(client.prefill_skip_count(), 0);

    // Tenant B queries same prompt -> Must be Miss!
    client.generate_with_context(tenant_b, &segs).await?;
    assert_eq!(client.prefill_skip_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_prefix_reuse_fail_safe_fallback_on_import_error() -> Result<()> {
    let store = Arc::new(InMemoryKvPrefixStore::default());
    let model = MockPrefixModel::new(true); // Fails on import
    let client = create_test_client(model, Some(store.clone()))?;

    let tenant = TenantId::try_new(300).map_err(|e| ContextraError::InvalidInput(e.to_string()))?;
    let segs = vec![ContextSegment {
        text: "Hello world test",
        chunk_id: 1,
        model_fingerprint: None,
        rope_offset: Some(0),
    }];

    client.generate_with_context(tenant, &segs).await?;
    let out = client.generate_with_context(tenant, &segs).await?;
    assert!(!out.is_empty());
    // Skipped count remains 0 because import failed and fell back to full prefill
    assert_eq!(client.prefill_skip_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_prefix_reuse_no_store_counter_invariance() -> Result<()> {
    let model = MockPrefixModel::new(false);
    let client = create_test_client(model, None)?; // No store configured

    let tenant = TenantId::try_new(400).map_err(|e| ContextraError::InvalidInput(e.to_string()))?;
    let segs = vec![ContextSegment {
        text: "Hello world test",
        chunk_id: 1,
        model_fingerprint: None,
        rope_offset: Some(0),
    }];

    client.generate_with_context(tenant, &segs).await?;
    client.generate_with_context(tenant, &segs).await?;

    assert_eq!(client.prefill_count(), 2);
    assert_eq!(client.prefill_skip_count(), 0);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_real_gguf_greedy_identic_with_and_without_reuse() -> Result<()> {
    let gguf_env = std::env::var("LLAMA_GGUF_PATH");
    if let Ok(model_path) = gguf_env {
        let path = std::path::Path::new(&model_path);
        if path.exists() {
            let device = Device::Cpu;
            let model = QuantizedLlamaModel::load(path, &device)?;
            let fingerprint = contextra_infer_candle::compute_fingerprint(
                path,
                &contextra_infer_candle::CandleQuantization::Q4KM,
            )?;
            let tokenizer_path = path
                .parent()
                .ok_or_else(|| ContextraError::InvalidInput("Missing parent directory".into()))?
                .join("tokenizer.json");
            let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
                .map_err(|e| ContextraError::InvalidInput(e.to_string()))?;

            let store = Arc::new(InMemoryKvPrefixStore::default());

            let client_plain = CandleLlmClient::new(
                device.clone(),
                Box::new(QuantizedLlamaModel::load(path, &device)?),
                fingerprint.clone(),
                tokenizer.clone(),
            );

            let client_reuse =
                CandleLlmClient::new(device, Box::new(model), fingerprint, tokenizer)
                    .with_prefix_store(store);

            let tenant =
                TenantId::try_new(1).map_err(|e| ContextraError::InvalidInput(e.to_string()))?;
            let segs = vec![ContextSegment {
                text: "The capital of France is Paris.",
                chunk_id: 1,
                model_fingerprint: None,
                rope_offset: Some(0),
            }];

            let res_plain = client_plain.generate_with_context(tenant, &segs).await?;
            client_reuse.generate_with_context(tenant, &segs).await?;
            let res_reuse = client_reuse.generate_with_context(tenant, &segs).await?;

            assert_eq!(res_plain, res_reuse);
        }
    }
    Ok(())
}
