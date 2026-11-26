//! Minimal standalone example demonstrating isolated usage of `contextra-store`.
//!
//! SPEC: M10 (Baukasten-Vertrag: eigenständiges Beispiel unter examples/ das die Crate allein benutzt).

use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_types::TxId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_dir = std::env::temp_dir().join(format!(
        "contextra_store_standalone_{}",
        rand::random::<u32>()
    ));
    let config = LsmConfig {
        path: tmp_dir.clone(),
        ..Default::default()
    };

    println!("Initializing standalone LsmStorage at: {:?}", tmp_dir);
    let storage = LsmStorage::new(config).await?;

    let tx = TxId::new(1);
    let key = b"standalone_key";
    let val = b"building_block_data";

    println!("Writing key-value pair under transaction 1...");
    storage.put(tx, key, val).await?;
    storage.commit(tx).await?;

    println!("Reading key-value pair...");
    let read_val = storage.get(key).await?;
    assert_eq!(read_val.as_deref(), Some(&val[..]));
    println!(
        "Read successfully: {:?}",
        read_val.map(|v| String::from_utf8_lossy(&v).into_owned())
    );

    storage.close().await?;
    let _ = std::fs::remove_dir_all(&tmp_dir);
    println!("Storage closed and temporary directory cleaned up.");

    Ok(())
}
