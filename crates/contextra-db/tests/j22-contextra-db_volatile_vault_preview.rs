// FILE-CONTEXT
// ZWECK: Integrationstest für VolatileContextVault-Produktionslebenszyklus (open -> ingest -> purge).
// Belegt die Erreichbarkeit von preview_metadata() über den umschließenden Produktionspfad purge().

use contextra_db::volatile_vault::{SignalModality, VaultChunk, VaultConfig, VolatileContextVault};
use contextra_types::{DocId, TxId};

#[test]
fn test_volatile_vault_enclosing_production_lifecycle_purge() {
    let config = VaultConfig {
        max_capacity_bytes: 1024 * 1024,
        attempt_mlock: false,
    };
    let mut vault = VolatileContextVault::open(config);

    assert!(vault.is_empty());
    assert_eq!(vault.len(), 0);

    let chunk1 = VaultChunk::new(
        DocId::new(101),
        b"sensitive payload alpha".to_vec(),
        SignalModality::TextInput,
        TxId::new(1),
    )
    .with_label("alpha");

    let chunk2 = VaultChunk::new(
        DocId::new(102),
        b"sensitive payload beta audio".to_vec(),
        SignalModality::AudioTranscript,
        TxId::new(2),
    )
    .with_label("beta");

    vault.ingest(chunk1).expect("ingest chunk 1");
    vault.ingest(chunk2).expect("ingest chunk 2");

    assert_eq!(vault.len(), 2);
    assert_eq!(
        vault.current_size_bytes(),
        b"sensitive payload alpha".len() + b"sensitive payload beta audio".len()
    );

    // Aufruf des umschließenden öffentlichen Produktionspfades purge(),
    // der preview_metadata() intern vor der Nullisierung der Daten aufruft.
    let receipt = vault.purge();
    assert_eq!(receipt.chunks_purged, 2);
    assert_eq!(
        receipt.bytes_zeroed,
        b"sensitive payload alpha".len() + b"sensitive payload beta audio".len()
    );
}
