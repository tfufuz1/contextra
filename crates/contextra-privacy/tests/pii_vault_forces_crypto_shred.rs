// FILE-CONTEXT
// ZWECK: Test PII vault forces CryptoShred under DeploymentTier::EdgeMinimal (INV-COLLECTION-PROFILE-3)

use contextra_privacy::egress_gateway::pii_vault_forces_crypto_shred;

#[test]
fn test_pii_vault_forces_crypto_shred_edge_minimal() {
    let is_pii_match = true;

    // When is_memory_only is false (e.g. non-MemoryOnly durability), PII forces CryptoShred
    let is_memory_only_false = false;
    assert_eq!(
        pii_vault_forces_crypto_shred(is_pii_match, is_memory_only_false),
        true
    );

    // When is_memory_only is true (BareMetal/MemoryOnly), physical CryptoShred is not forced/required on disk
    let is_memory_only_true = true;
    assert_eq!(
        pii_vault_forces_crypto_shred(is_pii_match, is_memory_only_true),
        false
    );

    // When not a PII match, CryptoShred is not forced regardless of memory-only status
    assert_eq!(pii_vault_forces_crypto_shred(false, false), false);
    assert_eq!(pii_vault_forces_crypto_shred(false, true), false);
}
