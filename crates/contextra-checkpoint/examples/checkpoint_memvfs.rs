//! Eigenständiges Beispiel für `contextra-checkpoint` auf `InMemoryStorageEngine`.
//!
//! Demonstriert die Erstellung, Abfrage und Wiederherstellung eines Checkpoints
//! ohne Abhängigkeit zu `contextra-store`.

use contextra_checkpoint::PersistentCheckpointStore;
use contextra_testkit::InMemoryStorageEngine;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let store = PersistentCheckpointStore::open(storage, "demo_namespace").await?;

    // Checkpoint mit Ziel-Tx erzeugen
    let tx = contextra_types::TxId::new(contextra_types::TxId::INTERNAL_BASE + 100);
    let cp_meta = store
        .create_checkpoint(
            "demo_cp",
            "demo_collection",
            10,
            tx,
            json!({"model": "gpt-4", "status": "checkpointed"}),
        )
        .await?;

    println!("Checkpoint erstellt: {:?}", cp_meta.name);

    let listed = store.list_checkpoints().await?;
    assert_eq!(listed.len(), 1);
    println!("Gefundene Checkpoints: {}", listed.len());

    if let Some(cp) = store.get_checkpoint("demo_cp").await? {
        println!("Checkpoint erfolgreich geladen: {:?}", cp.name);
    } else {
        return Err("Checkpoint nicht gefunden".into());
    }

    Ok(())
}
