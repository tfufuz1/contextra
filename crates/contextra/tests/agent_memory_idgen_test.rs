use contextra::{builder, AgentMemory, TextEmbeddingEngine};
use contextra_core::{BoxFuture, Result};
use contextra_ports::SequentialIdGen;
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
async fn test_agent_memory_deterministic_id_generation(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir_1 = tempfile::tempdir()?;
    let engine_1 = builder(16)
        .with_storage_path(temp_dir_1.path())
        .with_embedder(Arc::new(MockEmbedder))
        .build()
        .await?;
    let id_gen_1 = Arc::new(SequentialIdGen::new(100));
    let memory_1 = AgentMemory::new_with_id_gen(engine_1, id_gen_1);

    let id1_a = memory_1.remember("First memory", None).await?;
    let id1_b = memory_1.remember("Second memory", None).await?;
    let id1_c = memory_1.remember("Third memory", None).await?;

    let temp_dir_2 = tempfile::tempdir()?;
    let engine_2 = builder(16)
        .with_storage_path(temp_dir_2.path())
        .with_embedder(Arc::new(MockEmbedder))
        .build()
        .await?;
    let id_gen_2 = Arc::new(SequentialIdGen::new(100));
    let memory_2 = AgentMemory::new(engine_2).with_id_gen(id_gen_2);

    let id2_a = memory_2.remember("First memory", None).await?;
    let id2_b = memory_2.remember("Second memory", None).await?;
    let id2_c = memory_2.remember("Third memory", None).await?;

    // Verify deterministic ID generation
    assert_eq!(id1_a.as_str(), "100");
    assert_eq!(id1_b.as_str(), "101");
    assert_eq!(id1_c.as_str(), "102");

    assert_eq!(id2_a.as_str(), "100");
    assert_eq!(id2_b.as_str(), "101");
    assert_eq!(id2_c.as_str(), "102");

    assert_eq!(id1_a, id2_a);
    assert_eq!(id1_b, id2_b);
    assert_eq!(id1_c, id2_c);

    Ok(())
}

#[tokio::test]
async fn test_agent_memory_fallback_uuid_generation(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let engine = builder(16)
        .with_storage_path(temp_dir.path())
        .with_embedder(Arc::new(MockEmbedder))
        .build()
        .await?;
    let memory = AgentMemory::new(engine);

    let id_a = memory.remember("Fallback memory 1", None).await?;
    let id_b = memory.remember("Fallback memory 2", None).await?;

    // Verify non-identical IDs generated
    assert_ne!(id_a, id_b);

    // Verify valid v4 UUID format
    let parsed_a = uuid::Uuid::parse_str(id_a.as_str())?;
    let parsed_b = uuid::Uuid::parse_str(id_b.as_str())?;
    assert_eq!(parsed_a.get_version(), Some(uuid::Version::Random));
    assert_eq!(parsed_b.get_version(), Some(uuid::Version::Random));

    // Verify recall functionality works with fallback IDs
    let recalled = memory.recall("Fallback memory", 2).await?;
    assert!(!recalled.is_empty());

    Ok(())
}
