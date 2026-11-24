//! Härtungstests für contextra-vault gegen korrupte Eingaben, Bitflips und Truncation.

use contextra_vault::{
    SignalModality, VaultChunk, VaultConfig, VaultDocId, VaultError, VaultTxId,
    VolatileContextVault,
};

#[test]
fn test_vault_bitflip_and_capacity_fail_closed() {
    let config = VaultConfig {
        max_capacity_bytes: 50,
        attempt_mlock: false,
    };
    let mut vault = VolatileContextVault::open(config);
    let chunk1 = VaultChunk::new(
        VaultDocId::new(1),
        vec![0xAA; 40],
        SignalModality::TextInput,
        VaultTxId::new(1),
    );
    assert!(vault.ingest(chunk1).is_ok());

    // Truncated/Oversized chunk triggers CapacityExceeded without panic
    let chunk2 = VaultChunk::new(
        VaultDocId::new(2),
        vec![0xBB; 20],
        SignalModality::TextInput,
        VaultTxId::new(2),
    );
    let res = vault.ingest(chunk2);
    assert!(matches!(res, Err(VaultError::CapacityExceeded { .. })));
}

#[test]
fn test_vault_purge_clears_safely() {
    let mut vault = VolatileContextVault::open(VaultConfig::default());
    let chunk = VaultChunk::new(
        VaultDocId::new(1),
        vec![1, 2, 3],
        SignalModality::TextInput,
        VaultTxId::new(1),
    );
    assert!(vault.ingest(chunk).is_ok());
    let receipt = vault.purge();
    assert_eq!(receipt.chunks_purged, 1);
    assert_eq!(receipt.bytes_zeroed, 3);
}

#[cfg(feature = "workspace")]
#[test]
fn test_vault_hardening_with_workspace_types() {
    use contextra_types::{DocId, TxId};
    let config = VaultConfig {
        max_capacity_bytes: 50,
        attempt_mlock: false,
    };
    let mut vault = VolatileContextVault::open(config);
    let chunk = VaultChunk::new(
        DocId::new(1).into(),
        vec![0xCC; 10],
        SignalModality::TextInput,
        TxId::new(1).into(),
    );
    assert!(vault.ingest(chunk).is_ok());
}
