// FILE-CONTEXT
// STAND: 2026-10-05
// ZWECK: Integration test verifying production path reachability for J05 contextra-infer-candle symbols.

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
async fn test_tracked_request_count_wiring_via_llm_client() {
    let mock_model = Box::new(DefaultCandleLlmModel);
    let fingerprint = ModelFingerprint {
        hash: [1u8; 32],
        model_id: "test-model".to_string(),
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

    let exporter = Arc::new(CandleAttentionExporter::new());
    exporter.record_attention_weights(RequestId(1), vec![0.1, 0.2]);

    let client = CandleLlmClient::new(Device::Cpu, mock_model, fingerprint, tokenizer)
        .with_attention_exporter(exporter);

    assert_eq!(client.tracked_request_count(), 1);
}

#[test]
fn test_set_threshold_gasp_validator_wiring() {
    let mut validator = GaspValidator::new();
    assert_eq!(validator.threshold(), 0.70);

    validator.set_threshold(0.85);
    assert_eq!(validator.threshold(), 0.85);
    assert_eq!(validator.config().fingerprint.threshold(), 0.85);
}

#[cfg(feature = "kv-stage-b")]
#[test]
fn test_kv_state_layer_count_and_import_block_at_wiring() -> Result<(), Box<dyn std::error::Error>>
{
    use candle_core::Tensor;

    let k1 = Tensor::zeros((1, 4, 2, 8), candle_core::DType::F32, &Device::Cpu)?;
    let v1 = Tensor::zeros((1, 4, 2, 8), candle_core::DType::F32, &Device::Cpu)?;
    let layer1 = LayerKv::new(k1, v1);

    let state = KvState::new(vec![layer1], 4);
    assert_eq!(state.layer_count(), 1);

    let block = state.export_block(0..2)?;
    let mut target_state = KvState::new(Vec::new(), 0);
    target_state.import_block_at(&block, 0)?;

    assert_eq!(target_state.layer_count(), 1);
    assert_eq!(target_state.pos(), 2);
    Ok(())
}

#[test]
fn test_embed_client_with_max_concurrent_embeddings_wiring() {
    let temp_dir = tempfile::tempdir().unwrap();
    let client_res =
        CandleEmbedClient::from_dir_with_concurrency(temp_dir.path(), CandleQuantization::Q4KM, 16);
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    assert_eq!(client.max_concurrent_embeddings, 16);
}

#[test]
fn test_parse_gguf_metadata_invalid_header_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_file = NamedTempFile::new()?;
    tmp_file.write_all(b"INVALID_GGUF_HEADER_BYTES")?;

    let res = parse_gguf_metadata(tmp_file.path());
    assert!(res.is_err());
    Ok(())
}

#[cfg(feature = "kv-stage-b")]
#[test]
fn test_llm_client_from_dir_with_prefix_store_wiring() {
    struct DummyPrefixStore;
    impl contextra_ports::kv::KvPrefixStore for DummyPrefixStore {
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

    let temp_dir = tempfile::tempdir().unwrap();
    let store = Arc::new(DummyPrefixStore);
    let client_res = CandleLlmClient::from_dir_with_prefix_store(
        temp_dir.path(),
        CandleQuantization::Q4KM,
        store,
    );
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    assert!(client.prefix_store.is_some());
}
