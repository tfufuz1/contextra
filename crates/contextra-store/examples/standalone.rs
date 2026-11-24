//! Standalone Example für `contextra-store`.
//!
//! Demonstriert die autarke Verwendung des Speicherkerns (LsmStorage)
//! ohne direkte Kopplung an fremde Crates.

use contextra_store::lsm::{LsmConfig, LsmStorage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join(format!(
        "contextra_store_standalone_{}",
        std::process::id()
    ));
    let config = LsmConfig {
        path: tmp_dir.clone(),
        ..Default::default()
    };

    println!("Initialisiere autarken LsmStorage unter {:?}", tmp_dir);
    let storage = LsmStorage::new(config).await?;

    let seq = storage.last_applied_seq();
    println!("Initial seq_no: {}", seq);

    storage.close().await?;
    let _ = std::fs::remove_dir_all(&tmp_dir);
    println!("Bereinigung abgeschlossen.");
    Ok(())
}
