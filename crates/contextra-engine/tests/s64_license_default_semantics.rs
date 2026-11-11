//! # Lizenz-Default-Semantik Test (S-64B)
//!
//! Verifiziert, dass ohne explizite Angabe eines Lizenz-Gates bei Initialisierung
//! der `Contextra`-Instanz die bisherige `OpenFastGate`-Semantik aktiv ist.

use contextra_engine::{Contextra, ContextraConfig};
use tempfile::tempdir;

#[tokio::test]
async fn test_license_default_open_fast_gate_semantics() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let config = ContextraConfig::default();
    let engine = Contextra::open_with_config(dir.path(), config).await?;

    // Instanz öffnet ohne Fehler und bietet uneingeschränkten Zugriff
    let col = engine.collection("test_lic_col").await?;
    col.insert("doc1", &[0.1; 768], None).await?;
    let res = col.get("doc1").await?;
    assert!(res.is_some());

    Ok(())
}
