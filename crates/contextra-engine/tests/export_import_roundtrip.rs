use contextra_engine::{
    Contextra, ContextraConfig, ExportDocumentV1, SCHEMA_VERSION_V1,
};
use contextra_types::ContextraError;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_export_import_collection_roundtrip_and_schema_validation() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config.clone()).await?;
    let col = db.collection("export_src_col").await?;

    // 1. Populate source collection
    col.insert(
        "doc_exp_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "title": "Export Test Doc 1", "category": "A" })),
    )
    .await?;

    col.insert(
        "doc_exp_2",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({ "title": "Export Test Doc 2", "category": "B" })),
    )
    .await?;

    col.relate("doc_exp_1", "doc_exp_2", "LINKED_TO").await?;

    // 2. Export memories via Contextra::export_memories()
    let export_doc = db.export_memories().await?;

    assert_eq!(export_doc.schema_version, SCHEMA_VERSION_V1);
    assert!(
        !export_doc.collections.is_empty(),
        "Exported collections list must not be empty"
    );

    // Serialize to JSON string and deserialize
    let json_str = serde_json::to_string_pretty(&export_doc)
        .map_err(|e| ContextraError::Serialization(e.to_string()))?;

    let deserialized_export: ExportDocumentV1 = serde_json::from_str(&json_str)
        .map_err(|e| ContextraError::Serialization(e.to_string()))?;

    // 3. Import into clean database instance
    let tmp_dest = TempDir::new().expect("Failed to create dest temp directory");
    let db_dest = Contextra::open_with_config(tmp_dest.path(), config).await?;

    let summary = db_dest.import_memories(deserialized_export.clone()).await?;

    assert!(
        summary.imported_memories >= 2,
        "Import summary imported_memories must match exported memory count"
    );

    // Verify imported data in destination database
    let col_dest = db_dest.collection("export_src_col").await?;
    let doc1 = col_dest.get("doc_exp_1").await?;
    assert!(doc1.is_some(), "Imported doc_exp_1 must exist in col_dest");
    assert_eq!(
        doc1.unwrap()
            .metadata
            .as_ref()
            .and_then(|m| m.get("title"))
            .and_then(|v| v.as_str()),
        Some("Export Test Doc 1")
    );

    // 4. Error path test: Importing invalid schema version MUST fail with typed error
    let mut invalid_export = deserialized_export;
    invalid_export.schema_version = "v99.0".to_string();

    let import_res = db_dest.import_memories(invalid_export).await;
    match import_res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("Incompatible export schema version") || msg.contains("v99.0"),
                "Error message should mention incompatible export schema version"
            );
        }
        other => {
            assert!(
                false,
                "Expected ContextraError::InvalidInput on schema version mismatch, got: {:?}",
                other
            );
        }
    }

    db.close().await?;
    db_dest.close().await?;
    Ok(())
}
