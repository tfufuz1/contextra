#![forbid(unsafe_code)]

//! Integration test verifying attention producer flow to CandleAttentionExporter.

use candle_core::Device;
use contextra_infer_candle::{
    CandleAttentionExporter, CandleLlmClient, CandleModelInner, ModelFingerprint,
};
use contextra_ports::{AttentionExporter, LlmTextGenerator, RequestId};
use contextra_types::{ContextraError, Result};
use std::sync::Arc;

struct MockAttentionModel {
    scores: Vec<Vec<f32>>,
}

impl MockAttentionModel {
    fn new(scores: Vec<Vec<f32>>) -> Self {
        Self { scores }
    }
}

impl CandleModelInner for MockAttentionModel {
    fn generate(
        &mut self,
        prompt: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
    ) -> Result<String> {
        Ok(format!("[MockAttentionModel] Response for: {prompt}"))
    }

    fn last_attention_scores(&self) -> Option<Vec<Vec<f32>>> {
        Some(self.scores.clone())
    }
}

fn create_dummy_tokenizer() -> Result<tokenizers::Tokenizer> {
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
    tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
        .map_err(|e| ContextraError::Internal(format!("Tokenizer error: {e}")))
}

#[tokio::test]
async fn test_attention_producer_flow_with_and_without_exporter() -> Result<()> {
    let scores = vec![vec![0.1, 0.2, 0.7], vec![0.3, 0.4, 0.3]];
    let mock_model = Box::new(MockAttentionModel::new(scores));
    let fingerprint = ModelFingerprint {
        hash: [1u8; 32],
        model_id: "mock-attn".to_string(),
        quantization: "Q4_0".to_string(),
    };
    let tokenizer = create_dummy_tokenizer()?;

    let client_without_exporter =
        CandleLlmClient::new(Device::Cpu, mock_model, fingerprint.clone(), tokenizer.clone());

    let res_without = client_without_exporter.generate("Hello World").await?;
    assert_eq!(
        res_without,
        "[MockAttentionModel] Response for: Hello World"
    );

    let exporter = Arc::new(CandleAttentionExporter::new());
    assert_eq!(exporter.tracked_request_count(), 0);

    let mock_model_with_exp = Box::new(MockAttentionModel::new(vec![
        vec![0.1, 0.2, 0.7],
        vec![0.3, 0.4, 0.3],
    ]));

    let client_with_exporter = CandleLlmClient::new(
        Device::Cpu,
        mock_model_with_exp,
        fingerprint,
        tokenizer,
    )
    .with_attention_exporter(exporter.clone());

    let res_with = client_with_exporter.generate("Hello World").await?;
    assert_eq!(res_with, "[MockAttentionModel] Response for: Hello World");

    assert_eq!(exporter.tracked_request_count(), 1);

    // Request ID starts at 1 via default SequentialIdGen
    let exported = exporter.export_attention_weights(RequestId(1));
    assert!(exported.is_some());
    if let Some(exported_weights) = exported {
        assert_eq!(exported_weights.len(), 3);
        assert!((exported_weights[0] - 0.4).abs() < 1e-5);
        assert!((exported_weights[1] - 0.6).abs() < 1e-5);
        assert!((exported_weights[2] - 1.0).abs() < 1e-5);
    }

    Ok(())
}

#[tokio::test]
async fn test_attention_producer_flow_concurrent_requests_distinct_ids() -> Result<()> {
    let scores = vec![vec![0.5, 0.5]];
    let mock_model = Box::new(MockAttentionModel::new(scores));
    let fingerprint = ModelFingerprint {
        hash: [2u8; 32],
        model_id: "mock-attn-parallel".to_string(),
        quantization: "Q4_0".to_string(),
    };
    let tokenizer = create_dummy_tokenizer()?;

    let exporter = Arc::new(CandleAttentionExporter::new());
    let client = Arc::new(
        CandleLlmClient::new(Device::Cpu, mock_model, fingerprint, tokenizer)
            .with_attention_exporter(exporter.clone()),
    );

    let client1 = Arc::clone(&client);
    let client2 = Arc::clone(&client);

    let task1 = tokio::spawn(async move { client1.generate("Prompt 1").await });
    let task2 = tokio::spawn(async move { client2.generate("Prompt 2").await });

    let res1 = task1
        .await
        .map_err(|e| ContextraError::Internal(format!("Task 1 join error: {e}")))??;
    let res2 = task2
        .await
        .map_err(|e| ContextraError::Internal(format!("Task 2 join error: {e}")))??;

    assert_eq!(res1, "[MockAttentionModel] Response for: Prompt 1");
    assert_eq!(res2, "[MockAttentionModel] Response for: Prompt 2");

    assert_eq!(exporter.tracked_request_count(), 2);

    let req1_weights = exporter.export_attention_weights(RequestId(1));
    let req2_weights = exporter.export_attention_weights(RequestId(2));

    assert!(req1_weights.is_some());
    assert!(req2_weights.is_some());
    if let (Some(w1), Some(w2)) = (req1_weights, req2_weights) {
        assert_eq!(w1, vec![0.5, 0.5]);
        assert_eq!(w2, vec![0.5, 0.5]);
    }

    Ok(())
}
