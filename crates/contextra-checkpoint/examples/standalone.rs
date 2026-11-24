use contextra_checkpoint::contract::InMemorySnapshotSource;
use contextra_checkpoint::PersistentCheckpointStore;
use contextra_ids::TxId;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Arc::new(InMemorySnapshotSource::new());
    let store = PersistentCheckpointStore::open(source, "standalone_demo").await?;
    let cp = store
        .create_checkpoint("cp1", "demo", 1, TxId::new(100), json!({}))
        .await?;
    println!("Checkpoint created: {}", cp.name);
    let restored = store.restore_checkpoint("cp1").await?;
    println!("Checkpoint restored: {}", restored.name);
    Ok(())
}
