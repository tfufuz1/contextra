#![cfg(not(loom))]

use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::TenantId;
use serde_json::json;
use tempfile::TempDir;

/// Test (a) Determinismus: Gleicher Collection-Name liefert deterministisch dieselbe `CollectionId`.
#[test]
fn test_derive_collection_id_determinism() -> contextra_types::Result<()> {
    let id1 = Contextra::derive_collection_id("analytics_events")?;
    let id2 = Contextra::derive_collection_id("analytics_events")?;
    let id3 = Contextra::derive_collection_id("user_profiles")?;

    assert_eq!(
        id1, id2,
        "Gleicher Collection-Name muss identische CollectionId erzeugen"
    );
    assert_ne!(
        id1, id3,
        "Unterschiedliche Collection-Namen muessen unterschiedliche CollectionIds erzeugen"
    );
    assert_ne!(id1.inner(), 0, "Abgeleitete CollectionId darf nicht 0 sein");

    Ok(())
}

/// Test (b) Fehlerpfad: Ein Hash-Praefix 0 wird von `CollectionId::try_new` abgelehnt und liefert Err (kein Fallback auf ID 1).
#[test]
fn test_collection_id_from_hash_prefix_rejects_zero() -> contextra_types::Result<()> {
    let err_res = Contextra::collection_id_from_hash_prefix(0);
    assert!(
        err_res.is_err(),
        "collection_id_from_hash_prefix(0) muss Err liefern"
    );

    let err = err_res.unwrap_err();
    assert!(
        matches!(err, contextra_types::ContextraError::InvalidInput(_)),
        "Fehler bei Praefix 0 muss InvalidInput sein, got: {:?}",
        err
    );

    let valid_id = Contextra::collection_id_from_hash_prefix(42)?;
    assert_eq!(valid_id.inner(), 42);

    Ok(())
}

/// Test (c) Reihenfolge / Fail-Closed: Ein Ableitungsfehler in drop_collection bricht ab VOR destruktiven Operationen.
#[tokio::test]
async fn test_drop_collection_derivation_order_fail_closed() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let tenant_id = TenantId::try_new(101).expect("Valid tenant ID");
    let proof_key = b"secret_proof_key_32bytes_1234567";

    // 1. Erstelle eine gueltige Collection und fuege Daten ein
    let col = db
        .collection_for_tenant("test_order_col", tenant_id)
        .await?;
    col.insert(
        "doc_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({"text": "data"})),
    )
    .await?;

    // 2. Versuch, eine ungueltige Collection (z. B. leerer Name) zu droppen
    let drop_res = db.drop_collection("", tenant_id, proof_key).await;
    assert!(
        drop_res.is_err(),
        "drop_collection mit ungueltigem Namen muss umgehend mit Err fehlschlagen"
    );

    // 3. Verifiziere, dass die urspruengliche Collection unberuehrt existiert und Daten noch vorhanden sind
    let doc = col.get("doc_1").await?;
    assert!(
        doc.is_some(),
        "Daten muessen unberuehrt bleiben, wenn drop_collection fehlgeschlagen ist"
    );

    db.close().await?;
    Ok(())
}
