//! Integrationstests für KV-Bridge-Fail-Open-Verhalten, GASP-Grounding und Stresstests unter Last.

#[cfg(feature = "kv-bridge")]
mod tests {
    use candle_core::Device;
    use contextra_infer_candle::inference::CandleModelInner;
    use contextra_infer_candle::kv_bridge::{create_test_kv_bridge, KvCacheKey};
    use contextra_infer_candle::{CandleLlmClient, GaspValidator};
    use contextra_ports::ResponseGroundingValidator;
    use contextra_types::{ModelFingerprint, Result, TenantId};
    use tokenizers::Tokenizer;

    struct DummyModel;
    impl CandleModelInner for DummyModel {
        fn generate(
            &mut self,
            prompt: &str,
            _tokenizer: &Tokenizer,
            _device: &Device,
        ) -> Result<String> {
            Ok(format!("Dummy response to: {prompt}"))
        }

        fn generate_stream(
            &mut self,
            _prompt: &str,
            _tokenizer: &Tokenizer,
            _device: &Device,
            _on_token: &mut dyn FnMut(String) -> bool,
        ) -> Result<String> {
            Ok("Dummy response".to_string())
        }
    }

    fn dummy_tokenizer() -> tokenizers::Tokenizer {
        let json = r#"{
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
        tokenizers::Tokenizer::from_bytes(json.as_bytes()).ok().unwrap()
    }

    fn dummy_fingerprint(quant: &str) -> ModelFingerprint {
        ModelFingerprint::new([0x55u8; 32], "llama-3.2-1b.gguf", quant)
    }

    #[test]
    fn kv_bridge_fallback_on_error() {
        let adapter = create_test_kv_bridge();
        let tenant = TenantId::try_new(777).ok().unwrap();
        let fp = dummy_fingerprint("Q4_K_M");
        let key = KvCacheKey::new(9999, fp.clone(), None);

        let cached = adapter.try_get_cached_segment(tenant, &key);
        assert!(
            cached.is_none(),
            "KV-Bridge lookup for non-existent segment MUST fall back to None (prefill)"
        );

        let corrupt_key = KvCacheKey::new(8888, fp, None);
        adapter.store.insert_segment(tenant, 8888, b"invalid corrupt payload bytes".to_vec());

        let failed_decrypt = adapter.try_get_cached_segment(tenant, &corrupt_key);
        assert!(
            failed_decrypt.is_none(),
            "KV-Bridge corrupt payload failure MUST return None (fail-open to prefill) without panicking"
        );
    }

    #[test]
    fn kv_bridge_fingerprint_mismatch() {
        let adapter = create_test_kv_bridge();
        let tenant = TenantId::try_new(888).ok().unwrap();
        let fp_q4 = dummy_fingerprint("Q4_K_M");
        let fp_q8 = dummy_fingerprint("Q8_0");
        let chunk_id = 101;
        let payload = b"cached KV activation layer weights for Q4_K_M".to_vec();

        let key_q4 = KvCacheKey::new(chunk_id, fp_q4, Some(64));
        adapter.store_segment(tenant, key_q4.clone(), payload.clone());

        let hit = adapter.try_get_cached_segment(tenant, &key_q4);
        assert_eq!(hit, Some(payload));

        let key_q8 = KvCacheKey::new(chunk_id, fp_q8, Some(64));
        let result = adapter.try_get_cached_segment(tenant, &key_q8);
        assert!(
            result.is_none(),
            "Requesting a segment with a mismatched fingerprint MUST return None (cache miss)"
        );
    }

    #[test]
    fn gasp_grounding() {
        let validator = GaspValidator::new();

        let response = "Berlin is the capital of Germany with a population of 3.8 million.";
        let source1 = "Berlin is the capital city of Germany.";
        let source2 = "The population of Berlin is approximately 3.8 million people.";

        let score = validator
            .score_grounding(response, &[source1, source2])
            .ok()
            .unwrap();

        assert!(
            score >= 0.5,
            "GASP grounding score ({score}) should meet threshold for supported claims"
        );

        let unsupported_response = "Berlin is located in South America near the Amazon river.";
        let unsup_score = validator
            .score_grounding(unsupported_response, &[source1, source2])
            .ok()
            .unwrap();

        assert!(
            unsup_score < score,
            "Unsupported response score ({unsup_score}) must be lower than grounded score ({score})"
        );
    }

    #[tokio::test]
    async fn inference_backpressure_saturation() {
        let fp = dummy_fingerprint("Q4_K_M");
        let client = CandleLlmClient::new(Device::Cpu, Box::new(DummyModel), fp, dummy_tokenizer())
            .with_max_concurrent_inferences(4);

        assert_eq!(client.max_concurrent_inferences, 4);

        let permit1 = client.semaphore.acquire().await.ok().unwrap();
        let permit2 = client.semaphore.acquire().await.ok().unwrap();
        assert_eq!(client.semaphore.available_permits(), 2);

        drop(permit1);
        drop(permit2);
        assert_eq!(client.semaphore.available_permits(), 4);
    }
}
