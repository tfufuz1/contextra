#![cfg(feature = "volatile-vault")]

// FILE-CONTEXT
// ZWECK: Integrationstest für VolatileContextVault preview_metadata() und Produktions-purge()-Lebenszyklus.

use contextra_db::volatile_vault::{SignalModality, VaultChunk, VaultConfig, VolatileContextVault};
use contextra_types::{DocId, TxId};

#[test]
fn test_j22closure_volatile_vault_preview_metadata_and_purge() {
    let config = VaultConfig {
        max_capacity_bytes: 1024 * 1024,
        attempt_mlock: false,
    };
    let mut vault = VolatileContextVault::open(config);

    assert!(vault.is_empty());
    assert_eq!(vault.preview_metadata().len(), 0);

    let chunk1 = VaultChunk::new(
        DocId::new(201),
        b"confidential information text".to_vec(),
        SignalModality::TextInput,
        TxId::new(10),
    )
    .with_label("doc_201");

    let chunk2 = VaultChunk::new(
        DocId::new(202),
        b"structured record data".to_vec(),
        SignalModality::StructuredData,
        TxId::new(11),
    )
    .with_label("doc_202");

    vault.ingest(chunk1).expect("ingest chunk 1");
    vault.ingest(chunk2).expect("ingest chunk 2");

    // Preview metadata directly before purge
    let preview = vault.preview_metadata();
    assert_eq!(preview.len(), 2);
    assert_eq!(preview[0].id, DocId::new(201));
    assert_eq!(preview[0].modality, SignalModality::TextInput);
    assert_eq!(preview[0].size_bytes, b"confidential information text".len());
    assert_eq!(preview[0].label.as_deref(), Some("doc_201"));

    assert_eq!(preview[1].id, DocId::new(202));
    assert_eq!(preview[1].modality, SignalModality::StructuredData);
    assert_eq!(preview[1].size_bytes, b"structured record data".len());
    assert_eq!(preview[1].label.as_deref(), Some("doc_202"));

    // Call purge() which internally invokes preview_metadata() prior to zeroizing data
    let receipt = vault.purge();
    assert_eq!(receipt.chunks_purged, 2);
    assert_eq!(
        receipt.bytes_zeroed,
        b"confidential information text".len() + b"structured record data".len()
    );
}
