use super::*;
use crate::gasp::GaspValidator;
use crate::model_registry::ModelFingerprint;
use candle_core::Device;
use contextra_types::{ConfigFingerprint, Result};
use contextra_ports::{LlmTextGenerator, LlmTextGeneratorStreaming};
#[cfg(feature = "kv-bridge")]
use contextra_types::TenantId;
use std::sync::Arc;

struct MockCandleModel {
    response: String,
}

impl CandleModelInner for MockCandleModel {
    fn generate(
        &mut self,
        prompt: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
    ) -> Result<String> {
        Ok(format!("{prompt} -> {}", self.response))
    }
}

#[tokio::test]
async fn test_candle_llm_client_generate() {
    let mock_model = Box::new(MockCandleModel {
        response: "Generated Completion".to_string(),
    });
    let fingerprint = ModelFingerprint {
        hash: [1u8; 32],
        model_id: "mock_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
        .map_err(|e| e.to_string())
        .unwrap();

    let client = CandleLlmClient::new(Device::Cpu, mock_model, fingerprint.clone(), tokenizer);

    assert_eq!(client.fingerprint(), &fingerprint);

    let output = client.generate("Hello world").await.unwrap();
    assert_eq!(output, "Hello world -> Generated Completion");
}

#[tokio::test]
async fn test_candle_llm_client_generate_stream() {
    use futures_util::StreamExt;

    let mock_model = Box::new(MockCandleModel {
        response: "Generated Completion".to_string(),
    });
    let fingerprint = ModelFingerprint {
        hash: [1u8; 32],
        model_id: "mock_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
        .map_err(|e| e.to_string())
        .unwrap();

    let client = CandleLlmClient::new(Device::Cpu, mock_model, fingerprint.clone(), tokenizer);
    let cfg = ConfigFingerprint::new("mock_model.gguf", "Q4_K_M", "default", 0.0);

    let sync_resp = client.generate("Hello world").await.unwrap();

    let mut stream = client.generate_stream("Hello world", &cfg);
    let mut assembled = String::new();
    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.unwrap();
        assembled.push_str(&chunk);
    }

    assert_eq!(sync_resp, assembled);
}

#[cfg(feature = "kv-bridge")]
#[tokio::test]
async fn test_generate_with_context_text_identical_and_kv_bridge_consultation() {
    let mock_model = Box::new(MockCandleModel {
        response: "Unified Output".to_string(),
    });
    let fingerprint = ModelFingerprint {
        hash: [9u8; 32],
        model_id: "context_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let client_plain = CandleLlmClient::new(
        Device::Cpu,
        mock_model,
        fingerprint.clone(),
        tokenizer.clone(),
    );

    let seg1 = ContextSegment::new(101, "Chunk 1 content");
    let seg2 = ContextSegment::new(102, "Chunk 2 content");
    let segments = vec![seg1, seg2];

    let tenant = TenantId::try_new(1).unwrap();
    let direct_concat_res = client_plain
        .generate("Chunk 1 content\n\nChunk 2 content")
        .await
        .unwrap();
    let context_without_adapter_res = client_plain
        .generate_with_context(tenant, &segments)
        .await
        .unwrap();

    assert_eq!(
        direct_concat_res, context_without_adapter_res,
        "generate_with_context without adapter must produce text-identical result to generate"
    );
}

#[cfg(feature = "kv-bridge")]
#[tokio::test]
async fn test_generate_with_context_kv_bridge_consultation() {
    use crate::KvBridgeAdapter;
    use contextra_crypto::{CryptoKey, KvSegmentCipher, TenantIsolatedKvStore};

    let fingerprint = ModelFingerprint {
        hash: [9u8; 32],
        model_id: "context_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let master_km = CryptoKey::try_new("passphrase", b"salt12345").unwrap();
    let cipher = Arc::new(KvSegmentCipher::new(master_km));
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, cipher);

    let mock_model = Box::new(MockCandleModel {
        response: "Unified Output".to_string(),
    });
    let client_with_adapter = CandleLlmClient::new(Device::Cpu, mock_model, fingerprint, tokenizer)
        .with_kv_bridge(adapter.clone());

    let seg1 = ContextSegment::new(101, "Chunk 1 content");
    let seg2 = ContextSegment::new(102, "Chunk 2 content");
    let segments = vec![seg1, seg2];

    let tenant = TenantId::try_new(1).unwrap();
    let context_with_adapter_res = client_with_adapter
        .generate_with_context(tenant, &segments)
        .await
        .unwrap();

    assert_eq!(
        context_with_adapter_res,
        "Chunk 1 content\n\nChunk 2 content -> Unified Output"
    );
    assert_eq!(
        adapter.consultation_count(),
        2,
        "KvBridgeAdapter must record consultation for each segment"
    );
}

#[cfg(feature = "kv-bridge")]
#[tokio::test]
async fn test_generate_with_context_kv_bridge_cache_hit_skips_prefill() {
    use crate::kv_bridge::{KvBridgeAdapter, KvCacheKey};
    use contextra_crypto::{CryptoKey, KvSegmentCipher, TenantIsolatedKvStore};

    let fingerprint = ModelFingerprint {
        hash: [9u8; 32],
        model_id: "context_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let master_km = CryptoKey::try_new("passphrase", b"salt12345").unwrap();
    let cipher = Arc::new(KvSegmentCipher::new(master_km));
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, cipher);

    let mock_model = Box::new(MockCandleModel {
        response: "Unified Output".to_string(),
    });
    let client_with_adapter =
        CandleLlmClient::new(Device::Cpu, mock_model, fingerprint.clone(), tokenizer)
            .with_kv_bridge(adapter.clone());

    let seg1 = ContextSegment::new(101, "Chunk 1 content").with_fingerprint(&fingerprint);
    let seg2 = ContextSegment::new(102, "Chunk 2 content").with_fingerprint(&fingerprint);
    let segments = vec![seg1, seg2];
    let tenant = TenantId::try_new(42).unwrap();

    // Initial counters
    assert_eq!(client_with_adapter.prefill_count(), 0);
    assert_eq!(client_with_adapter.prefill_skip_count(), 0);

    // FIRST CALL: Cache miss on both segments. Stores fresh KV tensor bytes.
    let first_res = client_with_adapter
        .generate_with_context(tenant, &segments)
        .await
        .unwrap();

    assert_eq!(
        first_res,
        "Chunk 1 content\n\nChunk 2 content -> Unified Output"
    );
    assert_eq!(
        client_with_adapter.prefill_count(),
        2,
        "First call must execute full prefill for both segments"
    );
    assert_eq!(
        client_with_adapter.prefill_skip_count(),
        0,
        "First call must have 0 prefill skips"
    );
    assert_eq!(adapter.consultation_count(), 2);

    // Verify that cached bytes actually exist in store and match expected structure
    let key1 = KvCacheKey::new(101, fingerprint.clone(), None);
    let key2 = KvCacheKey::new(102, fingerprint.clone(), None);
    let cached_bytes1 = adapter
        .try_get_cached_segment(tenant, &key1)
        .expect("Segment 1 must be cached");
    let cached_bytes2 = adapter
        .try_get_cached_segment(tenant, &key2)
        .expect("Segment 2 must be cached");
    assert_eq!(
        cached_bytes1,
        b"kv_cache_tensor:101:context_model.gguf:Chunk 1 content".to_vec()
    );
    assert_eq!(
        cached_bytes2,
        b"kv_cache_tensor:102:context_model.gguf:Chunk 2 content".to_vec()
    );

    // SECOND CALL with identical chunk_id / tenant / fingerprint
    let second_res = client_with_adapter
        .generate_with_context(tenant, &segments)
        .await
        .unwrap();

    assert_eq!(second_res, first_res);
    assert_eq!(
        client_with_adapter.prefill_count(),
        4,
        "Prefill count must be 4 after second call (2 segments * 2 calls)"
    );
    assert_eq!(
        client_with_adapter.prefill_skip_count(),
        0,
        "prefill_skip_count must remain 0 for placeholder KV-cache hits (§9.2)"
    );
    assert_eq!(
        adapter.consultation_count(),
        4,
        "Consultation counter must be 4 after 2 calls"
    );
}

#[cfg(feature = "kv-bridge")]
#[tokio::test]
async fn test_generate_with_context_metrics_distinguishes_miss_and_placeholder_hit() {
    use crate::kv_bridge::KvBridgeAdapter;
    use contextra_crypto::{CryptoKey, KvSegmentCipher, TenantIsolatedKvStore};

    let fingerprint = ModelFingerprint {
        hash: [7u8; 32],
        model_id: "metrics_test_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let master_km = CryptoKey::try_new("passphrase", b"salt12345").unwrap();
    let cipher = Arc::new(KvSegmentCipher::new(master_km));
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, cipher);

    let mock_model = Box::new(MockCandleModel {
        response: "Metrics Completion".to_string(),
    });
    let client = CandleLlmClient::new(Device::Cpu, mock_model, fingerprint.clone(), tokenizer)
        .with_kv_bridge(adapter.clone());

    let seg1 = ContextSegment::new(201, "Segment 1 text").with_fingerprint(&fingerprint);
    let segments = vec![seg1];
    let tenant = TenantId::try_new(88).unwrap();

    // 1. Initial Call: Cache Miss -> stores placeholder in KV-store
    let _ = client
        .generate_with_context(tenant, &segments)
        .await
        .unwrap();
    assert_eq!(client.prefill_count(), 1);
    assert_eq!(client.prefill_skip_count(), 0);

    // 2. Subsequent Call with same segment: Placeholder Cache Hit -> full prefill still executed, skip count remains 0
    let _ = client
        .generate_with_context(tenant, &segments)
        .await
        .unwrap();
    assert_eq!(client.prefill_count(), 2);
    assert_eq!(
        client.prefill_skip_count(),
        0,
        "Placeholder hit must NOT increment prefill_skip_count"
    );
}

#[test]
fn test_llm_from_dir_nonexistent() {
    let res = CandleLlmClient::from_dir(
        std::path::Path::new("/nonexistent/model/dir"),
        crate::model_registry::CandleQuantization::Q4KM,
    );
    assert!(res.is_err());
}

#[test]
fn test_llm_from_dir_valid_temp_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let res = CandleLlmClient::from_dir(
        temp_dir.path(),
        crate::model_registry::CandleQuantization::Q4KM,
    );
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_candle_llm_client_swap_model_invalidates_validator_calibration() {
    use crate::gasp::GaspConfig;
    use contextra_ports::GroundingValidator;
    use contextra_types::ContextChunk;
    use contextra_types::DocId;

    let mock_model_1 = Box::new(MockCandleModel {
        response: "Model 1 Completion".to_string(),
    });
    let fp1 = ModelFingerprint {
        hash: [1u8; 32],
        model_id: "llama-3.2-1b.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let mut client = CandleLlmClient::new(Device::Cpu, mock_model_1, fp1, tokenizer);

    let initial_gasp_cfg = GaspConfig {
        fingerprint: ConfigFingerprint::new(
            &client.fingerprint().model_id,
            &client.fingerprint().quantization,
            "gasp-attribution",
            0.0,
        ),
        ..GaspConfig::default()
    };
    let mut validator = GaspValidator::with_config(initial_gasp_cfg);

    // Record a grounding observation on the validator via record_external_feedback (INV-CAL-3)
    let chunk = ContextChunk {
        doc_id: DocId::new(1),
        content: "Der Umsatz betrug im Jahr 2025 genau 50 Millionen Euro.".to_string(),
        relevance: 0.95,
        token_count: 20,
        metadata: None,
        contextual_prefix: None,
        links: Vec::new(),
    };
    let res = validator
        .validate_grounding(
            "Im Jahr 2025 betrug der Umsatz 50 Millionen Euro.",
            &[chunk],
        )
        .await;
    assert!(res.is_ok());
    validator.record_external_feedback(res.unwrap().score, true);
    assert_eq!(validator.observation_count(), 1);

    // Perform runtime model hot swap on CandleLlmClient with new ModelFingerprint
    let mock_model_2 = Box::new(MockCandleModel {
        response: "Model 2 Completion".to_string(),
    });
    let fp2 = ModelFingerprint {
        hash: [2u8; 32],
        model_id: "llama-3.2-3b.gguf".to_string(),
        quantization: "Q8_0".to_string(),
    };

    client.swap_model(mock_model_2, fp2.clone(), Some(&mut validator));

    assert_eq!(client.fingerprint(), &fp2);
    // Calibrator observations MUST be reset to 0 via the runtime model hot-swap path (INV-CAL-2)
    assert_eq!(
        validator.observation_count(),
        0,
        "validator observation_count must be reset to 0 after client.swap_model"
    );
}

struct SlowCandleModel {
    delay: std::time::Duration,
}

impl CandleModelInner for SlowCandleModel {
    fn generate(
        &mut self,
        prompt: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
    ) -> Result<String> {
        std::thread::sleep(self.delay);
        Ok(format!("Slow response to: {prompt}"))
    }
}

#[tokio::test]
async fn test_inference_backpressure_single_permit_awaits() {
    let slow_model = Box::new(SlowCandleModel {
        delay: std::time::Duration::from_millis(100),
    });
    let fp = ModelFingerprint {
        hash: [3u8; 32],
        model_id: "slow_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let client = Arc::new(
        CandleLlmClient::new(Device::Cpu, slow_model, fp, tokenizer)
            .with_max_concurrent_inferences(1),
    );

    assert_eq!(client.max_concurrent_inferences, 1);
    assert_eq!(client.semaphore.available_permits(), 1);

    let start = std::time::Instant::now();

    let c1 = Arc::clone(&client);
    let handle1 = tokio::spawn(async move { c1.generate("task 1").await });

    // Short sleep to guarantee task 1 acquires the single permit
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert_eq!(client.semaphore.available_permits(), 0);

    let c2 = Arc::clone(&client);
    let handle2 = tokio::spawn(async move { c2.generate("task 2").await });

    let res1 = handle1.await.unwrap().unwrap();
    let res2 = handle2.await.unwrap().unwrap();

    let elapsed = start.elapsed();
    assert!(
        elapsed >= std::time::Duration::from_millis(180),
        "Sequential execution under limit=1 expected ~200ms, took {:?}",
        elapsed
    );
    assert_eq!(res1, "Slow response to: task 1");
    assert_eq!(res2, "Slow response to: task 2");
    assert_eq!(client.semaphore.available_permits(), 1);
}

#[tokio::test]
async fn test_inference_configurable_concurrency_limit() {
    let mock_model = Box::new(MockCandleModel {
        response: "Fast".to_string(),
    });
    let fp = ModelFingerprint {
        hash: [4u8; 32],
        model_id: "fast_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let client = CandleLlmClient::new(Device::Cpu, mock_model, fp, tokenizer)
        .with_max_concurrent_inferences(3);

    assert_eq!(client.max_concurrent_inferences, 3);
    assert_eq!(client.semaphore.available_permits(), 3);
}

#[tokio::test]
async fn test_inference_zero_concurrency_limit_clamped_to_one() {
    let mock_model = Box::new(MockCandleModel {
        response: "Clamped".to_string(),
    });
    let fp = ModelFingerprint {
        hash: [6u8; 32],
        model_id: "clamped_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let client = CandleLlmClient::new(Device::Cpu, mock_model, fp, tokenizer)
        .with_max_concurrent_inferences(0);

    assert_eq!(client.max_concurrent_inferences, 1);
    assert_eq!(client.semaphore.available_permits(), 1);
}

#[cfg(feature = "kv-bridge")]
#[tokio::test]
async fn test_candle_llm_client_with_kv_bridge() {
    use crate::KvBridgeAdapter;
    use contextra_crypto::{CryptoKey, KvSegmentCipher, TenantIsolatedKvStore};

    let mock_model = Box::new(MockCandleModel {
        response: "KV Response".to_string(),
    });
    let fp = ModelFingerprint {
        hash: [9u8; 32],
        model_id: "kv_model.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let master_km = CryptoKey::try_new("passphrase", b"salt12345").unwrap();
    let cipher = Arc::new(KvSegmentCipher::new(master_km));
    let store = Arc::new(TenantIsolatedKvStore::new());
    let adapter = KvBridgeAdapter::new(store, cipher);

    let client =
        CandleLlmClient::new(Device::Cpu, mock_model, fp, tokenizer).with_kv_bridge(adapter);

    assert!(client.kv_bridge.is_some());
}

#[tokio::test]
async fn test_inference_timeout_cancellation_releases_no_permit_leak() {
    let slow_model = Box::new(SlowCandleModel {
        delay: std::time::Duration::from_millis(200),
    });
    let fp = ModelFingerprint {
        hash: [5u8; 32],
        model_id: "slow_timeout.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": [],
            "normalizer": null,
            "pre_tokenizer": null,
            "post_processor": null,
            "decoder": null,
            "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
        }"#;
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    let client = Arc::new(
        CandleLlmClient::new(Device::Cpu, slow_model, fp, tokenizer)
            .with_max_concurrent_inferences(1),
    );

    let c1 = Arc::clone(&client);
    let h1 = tokio::spawn(async move { c1.generate("task 1").await });

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    // Task 2 attempts to generate, but times out while waiting for permit (simulating McpSandbox timeout)
    let c2 = Arc::clone(&client);
    let timed_out =
        tokio::time::timeout(std::time::Duration::from_millis(30), c2.generate("task 2")).await;

    assert!(
        timed_out.is_err(),
        "Task 2 must time out while permit is held by Task 1"
    );

    let _ = h1.await.unwrap().unwrap();
    // Give tokio a tick to return permit
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    assert_eq!(
        client.semaphore.available_permits(),
        1,
        "Permit must be fully available after cancellation without leaks"
    );
}
