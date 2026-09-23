use contextra::{builder, AgentMemory, TextEmbeddingEngine};
use contextra_core::{BoxFuture, Result};
use serde_json::json;
use std::sync::Arc;

struct MockEmbedder;

impl TextEmbeddingEngine for MockEmbedder {
    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>> {
        Box::pin(async move { Ok(vec![0.1; 16]) })
    }

    fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, Result<Vec<Vec<f32>>>> {
        Box::pin(async move { Ok(vec![vec![0.1; 16]; texts.len()]) })
    }
}

#[tokio::test]
async fn test_agent_memory_facade_lifecycle() {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = tmp_dir.path().join("agent_memory_db");

    let engine = builder(16)
        .with_storage_path(&db_path)
        .with_embedder(Arc::new(MockEmbedder))
        .build()
        .await
        .expect("build engine");

    let memory_facade = AgentMemory::new(engine);

    // 1. Remember
    let metadata = json!({ "category": "user_preference", "source": "conversation" });
    let mem_id1 = memory_facade
        .remember(
            "User prefers dark mode and Rust programming language",
            Some(metadata.clone()),
        )
        .await
        .expect("remember mem1");

    let mem_id2 = memory_facade
        .remember("User works as a software engineer in Berlin", None)
        .await
        .expect("remember mem2");

    assert_ne!(mem_id1, mem_id2);

    // 2. Recall (finds entry)
    let recalled = memory_facade
        .recall("programming language dark mode", 5)
        .await
        .expect("recall");

    assert!(
        !recalled.is_empty(),
        "Recall should return matching memories"
    );
    let found = recalled.iter().any(|m| m.id == mem_id1.as_str());
    assert!(found, "Recalled memories should contain mem_id1");

    // 3. Relate
    memory_facade
        .relate(&mem_id1, &mem_id2, "PREFERS_LOCATION")
        .await
        .expect("relate memories");

    // 4. Forget
    memory_facade.forget(&mem_id1).await.expect("forget mem1");

    // 5. Recall (entry is gone)
    let recalled_after_forget = memory_facade
        .recall("programming language dark mode", 5)
        .await
        .expect("recall after forget");

    let found_after_forget = recalled_after_forget
        .iter()
        .any(|m| m.id == mem_id1.as_str());
    assert!(
        !found_after_forget,
        "Forgotten memory should no longer be recalled"
    );
}
