//! Integration tests for `MockEmbedder::with_fixed_output` and `ContextSegment::with_rope_offset`.

use contextra_ports::embedding::{
    ContextSegment, EmbeddingProvider, LlmTextGenerator, MockEmbedder, TextEmbeddingEngine,
};
use contextra_ports::{BoxFuture, TenantId};

struct SimpleLlm;

impl LlmTextGenerator for SimpleLlm {
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, contextra_ports::Result<String>> {
        let text = prompt.to_string();
        Box::pin(async move { Ok(format!("LLM Output: {}", text)) })
    }
}

#[tokio::test]
async fn test_mock_embedder_with_fixed_output_via_trait_path() {
    let fixed_vec = vec![0.5f32, 1.5f32, 2.5f32, 3.5f32];
    let embedder = MockEmbedder::with_fixed_output(fixed_vec.clone());

    // Test through EmbeddingProvider trait
    assert_eq!(embedder.provider_name(), "mock");
    assert_eq!(embedder.embedding_dim(), 4);
    let output = EmbeddingProvider::embed(&embedder, "sample query")
        .await
        .unwrap();
    assert_eq!(output, fixed_vec);

    // Test through blanket implementation of TextEmbeddingEngine
    let engine: &dyn TextEmbeddingEngine = &embedder;
    let engine_output = engine.embed("another query").await.unwrap();
    assert_eq!(engine_output, fixed_vec);

    let batch_output = engine.embed_batch(&["q1", "q2"]).await.unwrap();
    assert_eq!(batch_output, vec![fixed_vec.clone(), fixed_vec]);
}

#[tokio::test]
async fn test_context_segment_with_rope_offset_via_llm_generator() {
    let seg1 = ContextSegment::new(1, "First segment").with_rope_offset(0);
    let seg2 = ContextSegment::new(2, "Second segment with RoPE").with_rope_offset(128);

    assert_eq!(seg1.rope_offset, Some(0));
    assert_eq!(seg2.rope_offset, Some(128));

    let llm = SimpleLlm;
    let tenant = TenantId::SYSTEM;
    let result = llm
        .generate_with_context(tenant, &[seg1, seg2])
        .await
        .unwrap();

    assert_eq!(
        result,
        "LLM Output: First segment\n\nSecond segment with RoPE"
    );
}
