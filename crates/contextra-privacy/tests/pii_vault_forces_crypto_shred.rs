// FILE-CONTEXT
// ZWECK: Test PII vault forces CryptoShred under DeploymentTier::EdgeMinimal (INV-COLLECTION-PROFILE-3)

use contextra_privacy::egress_gateway::{
    pii_vault_forces_crypto_shred, resolve_effective_kv_delete_mode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DummyDurability {
    Full,
    MemoryOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DummyKvDeleteMode {
    TombstoneOnly,
    CryptoShred,
}

#[test]
fn test_pii_vault_forces_crypto_shred_edge_minimal() {
    let is_pii_match = true;

    // When is_memory_only is false (e.g. non-MemoryOnly durability), PII forces CryptoShred
    let is_memory_only_false = false;
    assert!(pii_vault_forces_crypto_shred(
        is_pii_match,
        is_memory_only_false
    ));

    // When is_memory_only is true (BareMetal/MemoryOnly), physical CryptoShred is not forced/required on disk
    let is_memory_only_true = true;
    assert!(!pii_vault_forces_crypto_shred(
        is_pii_match,
        is_memory_only_true
    ));

    // When not a PII match, CryptoShred is not forced regardless of memory-only status
    assert!(!pii_vault_forces_crypto_shred(false, false));
    assert!(!pii_vault_forces_crypto_shred(false, true));
}

#[test]
fn test_resolve_effective_kv_delete_mode_matrix() {
    let tombstone = DummyKvDeleteMode::TombstoneOnly;
    let shred = DummyKvDeleteMode::CryptoShred;

    // PII + Full durability -> forces CryptoShred even if preset is TombstoneOnly
    assert_eq!(
        resolve_effective_kv_delete_mode(true, DummyDurability::Full, tombstone, shred),
        shred
    );

    // PII + MemoryOnly -> does NOT force CryptoShred, preserves preset
    assert_eq!(
        resolve_effective_kv_delete_mode(true, DummyDurability::MemoryOnly, tombstone, shred),
        tombstone
    );

    // No PII + Full durability -> preserves preset TombstoneOnly
    assert_eq!(
        resolve_effective_kv_delete_mode(false, DummyDurability::Full, tombstone, shred),
        tombstone
    );

    // No PII + preset is already CryptoShred -> stays CryptoShred
    assert_eq!(
        resolve_effective_kv_delete_mode(false, DummyDurability::Full, shred, shred),
        shred
    );
}
