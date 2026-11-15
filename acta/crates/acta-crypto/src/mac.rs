//! HMAC-SHA256 (RFC 2104) für Integritäts- und Commitment-Werte (Spec 4.1).

use crate::error::CryptoError;
use hmac::digest::KeyInit;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// HMAC-SHA256 über die konkatenierten `parts`.
///
/// # Errors
/// [`CryptoError::Internal`] nur, wenn die Schlüssellänge verweigert wird (bei HMAC nicht möglich).
pub fn hmac_sha256(key: &[u8], parts: &[&[u8]]) -> Result<[u8; 32], CryptoError> {
    let mut mac =
        <HmacSha256 as KeyInit>::new_from_slice(key).map_err(|_| CryptoError::Internal)?;
    for part in parts {
        mac.update(part);
    }
    Ok(mac.finalize().into_bytes().into())
}

/// Prüft ein HMAC-Tag in konstanter Zeit. Ungleiche Längen ergeben `false`.
///
/// # Errors
/// Wie [`hmac_sha256`].
pub fn verify_hmac_sha256(key: &[u8], parts: &[&[u8]], tag: &[u8]) -> Result<bool, CryptoError> {
    let expected = hmac_sha256(key, parts)?;
    Ok(expected.as_slice().ct_eq(tag).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc4231_case2_matches() -> Result<(), CryptoError> {
        let tag = hmac_sha256(b"Jefe", &[b"what do ya want for nothing?"])?;
        let hex: String = tag.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        Ok(())
    }

    #[test]
    fn verify_accepts_correct_and_rejects_wrong_or_short_tags() -> Result<(), CryptoError> {
        let tag = hmac_sha256(b"k", &[b"m"])?;
        assert!(verify_hmac_sha256(b"k", &[b"m"], &tag)?);
        let mut bad = tag;
        if let Some(first) = bad.get_mut(0) {
            *first ^= 1;
        }
        assert!(!verify_hmac_sha256(b"k", &[b"m"], &bad)?);
        assert!(!verify_hmac_sha256(
            b"k",
            &[b"m"],
            tag.get(..31).unwrap_or(&[])
        )?);
        Ok(())
    }
}
