//! # Modul: link_tests.rs
//!
//! **Zweck:** Tests für Dokumentverlinkungen und Kantenverfolgung.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

use super::fixtures::*;

#[tokio::test]
async fn test_link_memories_cycle_prevention_for_all_relations() -> contextra_types::Result<()> {
    use contextra_store::LsmStorage;
    use contextra_types::DocId;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert("node_a", &[1.0, 0.0, 0.0, 0.0], None).await?;
    col.insert("node_b", &[0.0, 1.0, 0.0, 0.0], None).await?;
    col.insert("node_c", &[0.0, 0.0, 1.0, 0.0], None).await?;

    let a = DocId::from_key("node_a")?;
    let b = DocId::from_key("node_b")?;
    let c = DocId::from_key("node_c")?;

    let rel = contextra_types::domain::LinkRelation::Elaborates;

    // A -> B
    col.link_memories(a, b, rel).await?;
    // B -> C
    col.link_memories(b, c, rel).await?;

    // C -> A should fail with cycle detection error!
    let cycle_res = col.link_memories(c, a, rel).await;
    assert!(cycle_res.is_err(), "Cyclic link must be rejected");
    let err_str = cycle_res.unwrap_err().to_string();
    assert!(err_str.contains("Cyclic Elaborates relation detected"));

    Ok(())
}
