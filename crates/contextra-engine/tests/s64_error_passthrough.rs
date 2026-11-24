//! # Fehler-Durchreichungstest (S-64F)
//!
//! Belegt, dass Backend-Fehler aus den Speicher- und Index-Ports unverändert
//! (Variante und Display-Nachricht) an der Engine-Fassade ankommen.

use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::ContextraError;
use tempfile::tempdir;

#[tokio::test]
async fn test_invalid_dimension_error_passthrough() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = ContextraConfig {
        dimension: 0, // Ungültige Dimension
        ..Default::default()
    };

    let result = Contextra::open_with_config(dir.path(), config).await;
    match result {
        Ok(_) => panic!("Erwartet Fehler bei Dimension 0"),
        Err(err) => match err {
            ContextraError::InvalidInput(msg) => {
                assert!(
                    msg.contains("dimension must be > 0"),
                    "Unerwartete Fehlermeldung: {msg}"
                );
            }
            other => panic!("Erwartet ContextraError::InvalidInput, erhalten: {:?}", other),
        },
    }

    Ok(())
}

#[tokio::test]
async fn test_collection_vector_dimension_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = ContextraConfig {
        dimension: 128,
        ..Default::default()
    };
    let engine = Contextra::open_with_config(dir.path(), config).await?;

    let col = engine.collection("mismatch_col").await?;

    // Einfügen eines Vektors mit falscher Dimension (64 statt 128)
    let bad_vector = vec![0.1f32; 64];
    let insert_result = col.insert("doc1", &bad_vector, None).await;

    assert!(insert_result.is_err());
    let err = insert_result.err().expect("Error expected");

    match err {
        ContextraError::InvalidInput(msg) => {
            assert!(
                msg.to_lowercase().contains("dimension mismatch"),
                "Unerwartete Fehlermeldung: {msg}"
            );
        }
        other => panic!("Erwartet InvalidInput, erhalten: {:?}", other),
    }

    Ok(())
}
