#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;
use contextra::builder;
use contextra::{AgentMemory, TextEmbeddingEngine};
use contextra_core::{BoxFuture, ContextraError};
use contextra_ports::clock::ManualClock;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;
use serde_json::json;

struct MockEmbedder;

impl TextEmbeddingEngine for MockEmbedder {
    fn embed<'a>(&'a self, _text: &'a str) -> BoxFuture<'a, contextra_core::Result<Vec<f32>>> {
        Box::pin(async move { Ok(vec![0.1; 16]) })
    }

    fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, contextra_core::Result<Vec<Vec<f32>>>> {
        Box::pin(async move { Ok(vec![vec![0.1; 16]; texts.len()]) })
    }
}

#[tokio::test]
async fn test_agent_memory_forget_flow_facade() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_04;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!("e2e_wiring_deletion seed: {}, clock_nanos: {}", seed, clock.now_nanos());

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_deletion_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let db = builder(16)
        .with_storage_path(&tmp_dir)
        .with_embedder(Arc::new(MockEmbedder))
        .build()
        .await?;

    let memory_facade = AgentMemory::new(db);

    // 1. Remember memory
    let mem_id = memory_facade
        .remember("System architecture design for vector store", Some(json!({"topic": "architecture"})))
        .await?;

    // 2. Recall memory
    let recalled = memory_facade.recall("vector store architecture", 5).await?;
    assert_eq!(recalled.len(), 1);
    assert_eq!(recalled[0].id, mem_id.as_str());

    // 3. Forget memory
    memory_facade.forget(&mem_id).await?;

    // 4. Recall again => empty
    let recalled_after = memory_facade.recall("vector store architecture", 5).await?;
    assert!(
        recalled_after.is_empty(),
        "Forgotten memory must not appear in recall results"
    );

    // 5. Verify underlying document is deleted via engine reference
    let doc_check = memory_facade.engine().get(mem_id.as_str()).await?;
    assert!(doc_check.is_none(), "Underlying document must be None after forget");

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
#[ignore = "blockiert durch PROMPT-DELETION-01: Purge driver reverse sweep and KV segment shredding is not wired through Contextra facade"]
async fn test_forget_document_purge_driver_and_deletion_proof() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_05;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!("e2e_wiring_deletion proof seed: {}, clock_nanos: {}", seed, clock.now_nanos());

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_deletion_proof_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    // Purge driver reverse sweep & DeletionProof generation require facade wiring for
    // deep KV segment shredding and cryptographic deletion proof export on forget_document.

    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}
