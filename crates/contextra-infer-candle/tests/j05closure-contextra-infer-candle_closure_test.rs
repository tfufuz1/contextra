// FILE-CONTEXT
// STAND: 2026-10-05
// ZWECK: Integration closure test verifying all 7 J05 contextra-infer-candle symbols.

use candle_core::Device;
use contextra_infer_candle::attention_exporter::CandleAttentionExporter;
use contextra_infer_candle::embedding::CandleEmbedClient;
use contextra_infer_candle::gasp::GaspValidator;
use contextra_infer_candle::gguf_loader::parse_gguf_metadata;
use contextra_infer_candle::inference::{CandleLlmClient, DefaultCandleLlmModel};
#[cfg(feature = "kv-stage-b")]
use contextra_infer_candle::kv_state::{KvState, LayerKv};
use contextra_infer_candle::model_registry::{CandleQuantization, ModelFingerprint};
use contextra_ports::RequestId;
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_symbol_1_tracked_request_count() {
    let exporter = Arc::new(CandleAttentionExporter::new());
    assert_eq!(exporter.tracked_request_count(), 0);

    exporter.record_attention_weights(RequestId(100), vec![0.5, 0.5]);
    assert_eq!(exporter.tracked_request_count(), 1);

    let mock_model = Box::new(DefaultCandleLlmModel);
    let fp = ModelFingerprint {
        hash: [1u8; 32],
        model_id: "closure-model".to_string(),
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
        .with_attention_exporter(exporter);

    assert_eq!(client.tracked_request_count(), 1);
}

#[test]
fn test_symbol_2_set_threshold() {
    let mut validator = GaspValidator::new();
    assert_eq!(validator.threshold(), 0.70);

    validator.set_threshold(0.92);
    assert_eq!(validator.threshold(), 0.92);
    assert_eq!(validator.config().fingerprint.threshold(), 0.92);
}

#[cfg(feature = "kv-stage-b")]
#[test]
fn test_symbol_3_with_prefix_store() {
    struct MockPrefixStore;
    impl contextra_ports::kv::KvPrefixStore for MockPrefixStore {
        fn lookup(
            &self,
            _tenant: contextra_types::TenantId,
            _key: &contextra_ports::kv::PrefixKey,
            _prompt_tokens: &[u32],
        ) -> Option<contextra_ports::kv::KvPrefixHit> {
            None
        }
        fn insert(
            &self,
            _tenant: contextra_types::TenantId,
            _key: &contextra_ports::kv::PrefixKey,
            _prompt_tokens: &[u32],
            _blocks: Vec<contextra_ports::kv::KvBlock>,
        ) -> Result<(), contextra_types::ContextraError> {
            Ok(())
        }
        fn evict(
            &self,
            _tenant: contextra_types::TenantId,
            _key: &contextra_ports::kv::PrefixKey,
        ) -> Result<u64, contextra_types::ContextraError> {
            Ok(0)
        }
    }

    let mock_model = Box::new(DefaultCandleLlmModel);
    let fp = ModelFingerprint {
        hash: [2u8; 32],
        model_id: "prefix-model".to_string(),
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

    let store = Arc::new(MockPrefixStore);
    let client = CandleLlmClient::new(Device::Cpu, mock_model, fp, tokenizer)
        .with_prefix_store(store);

    assert!(client.prefix_store.is_some());
}

#[cfg(feature = "kv-stage-b")]
#[test]
fn test_symbols_4_and_5_import_block_at_and_layer_count(
) -> Result<(), Box<dyn std::error::Error>> {
    use candle_core::Tensor;

    let k = Tensor::zeros((1, 2, 4, 8), candle_core::DType::F32, &Device::Cpu)?;
    let v = Tensor::zeros((1, 2, 4, 8), candle_core::DType::F32, &Device::Cpu)?;
    let layer = LayerKv::new(k, v);

    let state = KvState::new(vec![layer], 4);
    assert_eq!(state.layer_count(), 1);

    let block = state.export_block(0..2)?;
    let mut target = KvState::new(Vec::new(), 0);
    target.import_block_at(&block, 0)?;

    assert_eq!(target.layer_count(), 1);
    assert_eq!(target.pos(), 2);
    Ok(())
}

#[test]
fn test_symbol_6_with_max_concurrent_embeddings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let client = CandleEmbedClient::from_dir_with_concurrency(
        temp_dir.path(),
        CandleQuantization::Q4KM,
        12,
    )
    .expect("from_dir_with_concurrency should succeed");

    assert_eq!(client.max_concurrent_embeddings, 12);

    let client_configured = client.with_max_concurrent_embeddings(24);
    assert_eq!(client_configured.max_concurrent_embeddings, 24);
}

#[test]
fn test_symbol_7_parse_gguf_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_file = NamedTempFile::new()?;
    tmp_file.write_all(b"CORRUPT_OR_INVALID_GGUF_HEADER_STREAM_TEST")?;

    let res = parse_gguf_metadata(tmp_file.path());
    assert!(res.is_err());
    if let Err(err) = res {
        let msg = err.to_string();
        assert!(msg.contains("Failed to parse GGUF container header") || msg.contains("GGUF"));
    }
    Ok(())
}
