#![cfg(not(loom))]

use contextra_engine::{Contextra, ContextraConfig};
use serde_json::json;
use std::ops::Bound;
use tempfile::TempDir;

#[tokio::test]
async fn test_scan_internal_keys_leakage() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("default").await?;

    // Insert user documents
    col.insert("doc1", &[0.1, 0.2, 0.3, 0.4], Some(json!({"type": "user"})))
        .await?;
    col.insert("doc2", &[0.1, 0.2, 0.3, 0.4], Some(json!({"type": "user"})))
        .await?;
    col.insert(
        "_underscore_user_doc",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"type": "user_underscore"})),
    )
    .await?;

    // Create internal entries via relate API
    col.relate("doc1", "doc2", "knows").await?;

    // Create internal entries via put_kv
    col.put_kv("__internal_test_key", &json!({"internal": true}))
        .await?;
    col.put_kv("__graph:internal_node", &json!({"node": true}))
        .await?;

    // 1. scan_prefix("") must NOT contain internal keys starting with "__"
    let prefix_empty = col.scan_prefix("", None).await?;
    let internal_keys_empty: Vec<_> = prefix_empty
        .iter()
        .filter(|(k, _)| k.starts_with("__"))
        .collect();

    // 2. scan_prefix("__") must NOT contain internal keys
    let prefix_double = col.scan_prefix("__", None).await?;
    let internal_keys_double: Vec<_> = prefix_double
        .iter()
        .filter(|(k, _)| k.starts_with("__"))
        .collect();

    // 3. scan(Unbounded, Unbounded) must NOT contain internal keys
    let range_scan = col.scan(Bound::Unbounded, Bound::Unbounded, None).await?;
    let internal_keys_range: Vec<_> = range_scan
        .iter()
        .filter(|(k, _)| k.starts_with("__"))
        .collect();

    // Assert underscore user doc is included
    let has_underscore_doc = prefix_empty
        .iter()
        .any(|(k, _)| k == "_underscore_user_doc");
    assert!(
        has_underscore_doc,
        "User document with single underscore prefix '_underscore_user_doc' MUST NOT be filtered"
    );

    // Verify internal key leaks
    assert!(
        internal_keys_empty.is_empty(),
        "scan_prefix(\"\") leaked internal keys: {:?}",
        internal_keys_empty
    );
    assert!(
        internal_keys_double.is_empty(),
        "scan_prefix(\"__\") leaked internal keys: {:?}",
        internal_keys_double
    );
    assert!(
        internal_keys_range.is_empty(),
        "scan(Unbounded, Unbounded) leaked internal keys: {:?}",
        internal_keys_range
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_scan_rel_prefix_explicit_access() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("default").await?;

    col.insert("doc1", &[0.1, 0.2, 0.3, 0.4], Some(json!({"type": "user"})))
        .await?;
    col.insert("doc2", &[0.1, 0.2, 0.3, 0.4], Some(json!({"type": "user"})))
        .await?;
    col.relate("doc1", "doc2", "knows").await?;

    // Explicit scan_prefix("__rel:") should return relationship entries
    let rel_results = col.scan_prefix("__rel:", None).await?;
    assert!(
        !rel_results.is_empty(),
        "Explicit scan_prefix('__rel:') must return relationship entries"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_scan_non_default_collection_behavior() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("custom_col").await?;

    col.insert("docA", &[0.1, 0.2, 0.3, 0.4], Some(json!({"v": 1})))
        .await?;
    col.insert("docB", &[0.1, 0.2, 0.3, 0.4], Some(json!({"v": 2})))
        .await?;
    col.relate("docA", "docB", "linked").await?;

    let res = col.scan_prefix("", None).await?;
    let keys: Vec<_> = res.iter().map(|(k, _)| k.as_str()).collect();
    assert!(keys.contains(&"docA"));
    assert!(keys.contains(&"docB"));
    assert!(!keys.iter().any(|k| k.starts_with("__")));

    db.close().await?;
    Ok(())
}
