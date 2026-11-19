//! ReadGate-Durchsetzung an den Lesepfaden (SPEC §6.2, INV-GATE-COMPLETENESS, T-DEL-11, T-DEL-13).
//!
//! Jeder Test aktiviert ein Löschziel im Gate und prüft, dass der jeweilige Lesepfad
//! den Zugriff verweigert (`Erased`), das Ziel still auslässt bzw. bei `Restricted`
//! ohne Audit mit `RestrictedReadAuditRequired` abbricht.

#![allow(deprecated)]

use contextra_db::{Contextra, ContextraConfig, DistanceMetric};
use contextra_erasure::SagaGate;
use contextra_types::ContextraError;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

async fn setup() -> (Contextra, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");
    (db, tmp)
}

/// Dokumente: `doc-live` (offen) und `doc-erased` (später gesperrt).
async fn seed(db: &Contextra) {
    let col = db.collection("default").await.expect("collection");
    col.insert("doc-live", &[1.0, 0.0, 0.0, 0.0], Some(json!({"text": "live"})))
        .await
        .expect("insert live");
    col.insert("doc-erased", &[0.9, 0.1, 0.0, 0.0], Some(json!({"text": "erased"})))
        .await
        .expect("insert erased");
}

/// Liefert eine Collection-Sicht, die gegen das angegebene Gate prüft.
async fn view_with_gate(db: &Contextra, gate: Arc<SagaGate>) -> contextra_db::collection::Collection {
    let col = db.collection("default").await.expect("collection");
    (*col).clone().with_read_gate(gate)
}

#[tokio::test]
async fn get_on_erased_target_returns_erased_error() {
    let (db, _tmp) = setup().await;
    seed(&db).await;
    let gate = Arc::new(SagaGate::new());
    gate.activate_key(b"doc-erased").expect("activate");
    let view = view_with_gate(&db, gate).await;

    let err = view.get("doc-erased").await.expect_err("must be gated");
    assert!(matches!(err, ContextraError::Erased), "got {err:?}");
    assert!(view.get("doc-live").await.expect("ok").is_some());
}

#[tokio::test]
async fn get_on_restricted_target_requires_audit() {
    let (db, _tmp) = setup().await;
    seed(&db).await;
    let gate = Arc::new(SagaGate::new());
    gate.activate_restricted_key(b"doc-erased").expect("activate restricted");
    let view = view_with_gate(&db, gate).await;

    let err = view.get("doc-erased").await.expect_err("restricted read");
    assert!(
        matches!(err, ContextraError::RestrictedReadAuditRequired),
        "got {err:?}"
    );
}

#[tokio::test]
async fn scan_prefix_silently_omits_erased_entries() {
    let (db, _tmp) = setup().await;
    seed(&db).await;
    let gate = Arc::new(SagaGate::new());
    gate.activate_key(b"doc-erased").expect("activate");
    let view = view_with_gate(&db, gate).await;

    let rows = view.scan_prefix("doc-", None).await.expect("scan");
    let ids: Vec<&str> = rows.iter().map(|(k, _)| k.as_str()).collect();
    assert!(ids.contains(&"doc-live"), "live doc missing: {ids:?}");
    assert!(!ids.contains(&"doc-erased"), "erased doc leaked: {ids:?}");
}

#[tokio::test]
async fn vector_search_drops_erased_candidates() {
    let (db, _tmp) = setup().await;
    seed(&db).await;
    let gate = Arc::new(SagaGate::new());
    gate.activate_key(b"doc-erased").expect("activate");
    let view = view_with_gate(&db, gate).await;

    let hits = view
        .search(&[1.0, 0.0, 0.0, 0.0], 10)
        .await
        .expect("search");
    let ids: Vec<&str> = hits.iter().map(|h| h.id.as_str()).collect();
    assert!(ids.contains(&"doc-live"), "live hit missing: {ids:?}");
    assert!(!ids.contains(&"doc-erased"), "erased hit leaked: {ids:?}");
}

#[tokio::test]
async fn ungated_collection_is_unaffected() {
    let (db, _tmp) = setup().await;
    seed(&db).await;
    // Standard-Gate (leer) darf keine Ergebnisse unterdrücken.
    let col = db.collection("default").await.expect("collection");
    assert!(col.get("doc-erased").await.expect("get").is_some());
    let rows = col.scan_prefix("doc-", None).await.expect("scan");
    assert_eq!(rows.len(), 2);
}
