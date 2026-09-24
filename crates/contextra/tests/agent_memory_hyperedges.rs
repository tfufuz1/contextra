#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra::agent_memory::{MemoryId, RelationId};
use contextra::{builder, AgentMemory, ContextraError, TextEmbeddingEngine};
use contextra_core::{BoxFuture, Result};
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

async fn setup_agent_memory() -> (AgentMemory, tempfile::TempDir) {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let db_path = tmp_dir.path().join("agent_memory_hyperedges_db");

    let engine = builder(16)
        .with_storage_path(&db_path)
        .with_embedder(Arc::new(MockEmbedder))
        .build()
        .await
        .expect("build engine");

    (AgentMemory::new(engine), tmp_dir)
}

#[tokio::test]
async fn test_relate_n_ary_success_and_unique_ids() {
    let (memory, _tmp) = setup_agent_memory().await;

    let mem1 = memory
        .remember("Alice wrote the spec", None)
        .await
        .expect("remember 1");
    let mem2 = memory
        .remember("Bob reviewed the code", None)
        .await
        .expect("remember 2");
    let mem3 = memory
        .remember("Charlie tested the release", None)
        .await
        .expect("remember 3");

    let participants1 = vec![(&mem1, "author"), (&mem2, "reviewer")];
    let rel_id1: RelationId = memory
        .relate_n_ary("collaborated", &participants1, Some(&mem3))
        .await
        .expect("relate_n_ary 1");

    let participants2 = vec![
        (&mem1, "author"),
        (&mem2, "reviewer"),
        (&mem3, "tester"),
    ];
    let rel_id2: RelationId = memory
        .relate_n_ary("project_x", &participants2, None)
        .await
        .expect("relate_n_ary 2");

    assert_ne!(rel_id1, rel_id2, "Two relations must have distinct IDs");
}

#[tokio::test]
async fn test_relate_n_ary_insufficient_participants() {
    let (memory, _tmp) = setup_agent_memory().await;

    let mem1 = memory
        .remember("Solo memory", None)
        .await
        .expect("remember");

    let participants = vec![(&mem1, "author")];
    let res = memory.relate_n_ary("solo", &participants, None).await;

    assert!(res.is_err(), "relate_n_ary with < 2 participants must fail");
    let err = res.unwrap_err();
    assert!(
        matches!(err, ContextraError::InvalidInput(ref msg) if msg.contains("at least 2 participants")),
        "Expected InvalidInput error for <2 participants, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_relate_n_ary_empty_predicate_or_role() {
    let (memory, _tmp) = setup_agent_memory().await;

    let mem1 = memory.remember("Mem 1", None).await.expect("remember 1");
    let mem2 = memory.remember("Mem 2", None).await.expect("remember 2");

    // Empty predicate
    let participants = vec![(&mem1, "author"), (&mem2, "reviewer")];
    let res_pred = memory.relate_n_ary("", &participants, None).await;
    assert!(res_pred.is_err());
    assert!(
        matches!(res_pred.unwrap_err(), ContextraError::InvalidInput(ref msg) if msg.contains("predicate must not be empty"))
    );

    // Whitespace predicate
    let res_pred_ws = memory.relate_n_ary("   ", &participants, None).await;
    assert!(res_pred_ws.is_err());

    // Empty role
    let invalid_participants = vec![(&mem1, "author"), (&mem2, "")];
    let res_role = memory
        .relate_n_ary("collaborated", &invalid_participants, None)
        .await;
    assert!(res_role.is_err());
    assert!(
        matches!(res_role.unwrap_err(), ContextraError::InvalidInput(ref msg) if msg.contains("role must not be empty"))
    );
}

#[tokio::test]
async fn test_relate_n_ary_nonexistent_memory_id() {
    let (memory, _tmp) = setup_agent_memory().await;

    let mem1 = memory
        .remember("Existing memory", None)
        .await
        .expect("remember");
    let nonexistent_mem = MemoryId::new("nonexistent-doc-id-1234");

    let participants = vec![(&mem1, "author"), (&nonexistent_mem, "reviewer")];
    let res = memory
        .relate_n_ary("collaborated", &participants, None)
        .await;

    // The engine creates entity IDs from key strings and persists hyperedges regardless of prior doc insertion
    assert!(
        res.is_ok(),
        "relate_n_ary with valid string MemoryId succeeds in engine: {:?}",
        res
    );
}

#[tokio::test]
async fn test_relate_n_ary_forget_participant_does_not_panic() {
    let (memory, _tmp) = setup_agent_memory().await;

    let mem1 = memory.remember("Mem 1", None).await.expect("remember 1");
    let mem2 = memory.remember("Mem 2", None).await.expect("remember 2");

    let participants = vec![(&mem1, "author"), (&mem2, "reviewer")];
    let _rel_id = memory
        .relate_n_ary("collaborated", &participants, None)
        .await
        .expect("relate_n_ary");

    // Forgetting mem1 should not panic or fail
    let forget_res = memory.forget(&mem1).await;
    assert!(
        forget_res.is_ok(),
        "Forget participant should succeed without panicking"
    );
}
