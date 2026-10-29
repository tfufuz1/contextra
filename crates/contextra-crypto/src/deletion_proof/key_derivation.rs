// FILE-CONTEXT
// ZWECK: Key derivation helper methods specifically for deletion proof scope keys.
// INVARIANTEN: INV-KEYPURPOSE-1: HKDF domain separation tag mandatory.

use contextra_types::{Result, TenantId};

/// Domain separation tag for deletion proof key derivation.
pub const DELETION_PROOF_KEY_PURPOSE: &[u8] = b"Contextra-v1-DeletionProof-Key-Derivation";

/// Derives a 32-byte key specifically bound to a tenant and deletion scope.
pub fn derive_deletion_proof_key_for_tenant(
    master_key: &[u8; 32],
    tenant_id: TenantId,
) -> Result<[u8; 32]> {
    use hkdf::Hkdf;
    use sha2::Sha256;

    let hk = Hkdf::<Sha256>::new(Some(DELETION_PROOF_KEY_PURPOSE), master_key);
    let mut okm = [0u8; 32];
    let info = tenant_id.as_u64().to_be_bytes();
    hk.expand(&info, &mut okm)
        .map_err(|e| contextra_types::ContextraError::Internal(format!("HKDF expand failed: {e}")))?;
    Ok(okm)
}
