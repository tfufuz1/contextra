//! WAL deletion receipt creation and verification.

use contextra_types::{ContextraError, Result};

/// Erzeugt eine WAL-Löschquittung H(hmac_prev || delete_event) für den Nachweis
/// auf WAL-Ebene gemäß DSGVO Art. 17.
pub fn compute_wal_delete_receipt(
    prev_hmac: &[u8; 32],
    delete_event_payload: &[u8],
    integrity_key: &[u8],
) -> Result<[u8; 32]> {
    compute_hmac_sha256(
        integrity_key,
        &[b"contextra-wal-delete-v1", prev_hmac, delete_event_payload],
    )
}

/// Verifiziert eine WAL-Löschquittung in O(1) konstanter Zeit ohne Klartextzugang.
pub fn verify_wal_delete_receipt(
    receipt: &[u8; 32],
    prev_hmac: &[u8; 32],
    delete_event_payload: &[u8],
    integrity_key: &[u8],
) -> Result<bool> {
    use subtle::ConstantTimeEq;
    let expected = compute_wal_delete_receipt(prev_hmac, delete_event_payload, integrity_key)?;
    Ok(expected.ct_eq(receipt).into())
}

pub(super) fn compute_hmac_sha256(key: &[u8], data_parts: &[&[u8]]) -> Result<[u8; 32]> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|e| ContextraError::Internal(format!("HMAC key error: {e}")))?;
    for part in data_parts {
        mac.update(part);
    }
    Ok(mac.finalize().into_bytes().into())
}
