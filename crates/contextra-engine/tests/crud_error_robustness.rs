use contextra_engine::{Contextra, ContextraConfig, MAX_SCAN_RESULTS};
use contextra_types::ContextraError;
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_crud_error_path_robustness_returns_typed_errors() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("crud_robustness_col").await?;

    // 1. Vector dimension mismatch on insert: passing 8 floats into dimension 4 collection
    let insert_mismatch_res = col
        .insert(
            "doc_valid_id",
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            Some(json!({ "test": "mismatch" })),
        )
        .await;

    match insert_mismatch_res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("Dimension mismatch"),
                "Expected dimension mismatch message, got: {}",
                msg
            );
        }
        other => assert!(
            false,
            "Expected ContextraError::InvalidInput on dimension mismatch insert, got: {:?}",
            other
        ),
    }

    // 2. Vector dimension mismatch on search
    let search_mismatch_res = col.query().embedding(&[1.0, 2.0]).execute().await;

    match search_mismatch_res {
        Err(ContextraError::EmbeddingDimensionMismatch { expected, got }) => {
            assert_eq!(expected, 4);
            assert_eq!(got, 2);
        }
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(msg.contains("Dimension mismatch"));
        }
        other => assert!(
            false,
            "Expected dimension mismatch error on search, got: {:?}",
            other
        ),
    }

    // 3. Invalid document ID validation (empty string or control characters)
    let invalid_id_get = col.get("").await;
    match invalid_id_get {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("Document ID") || msg.contains("empty"),
                "Expected document ID error message"
            );
        }
        other => assert!(
            false,
            "Expected ContextraError::InvalidInput on empty doc ID get, got: {:?}",
            other
        ),
    }

    let invalid_id_insert = col
        .insert("doc\0invalid_null_byte", &[0.1, 0.2, 0.3, 0.4], None)
        .await;

    match invalid_id_insert {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("Document ID") || msg.contains("null") || msg.contains("invalid"),
                "Expected invalid doc ID error message"
            );
        }
        other => assert!(
            false,
            "Expected ContextraError::InvalidInput on doc ID with null byte, got: {:?}",
            other
        ),
    }

    // 4. Scan prefix with requested limit exceeding MAX_SCAN_RESULTS
    let excessive_limit = MAX_SCAN_RESULTS + 100;
    let scan_limit_res = col.scan_prefix("test", Some(excessive_limit)).await;

    match scan_limit_res {
        Err(ContextraError::InvalidInput(msg)) => {
            assert!(
                msg.contains("requested limit") || msg.contains("exceeds MAX_SCAN_RESULTS"),
                "Expected scan limit error message"
            );
        }
        other => assert!(
            false,
            "Expected ContextraError::InvalidInput on excessive scan_prefix limit, got: {:?}",
            other
        ),
    }

    db.close().await?;
    Ok(())
}
