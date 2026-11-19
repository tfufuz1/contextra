//! AEAD Record Envelopes (RFC 8452, AES-256-GCM-SIV).
//!
//! Provides authenticated encryption with associated data (AEAD) ensuring
//! strict nonce discipline, constant-time verification, and uniform error reporting.
//! All header components (version, algorithm, doc id) are cryptographically bound into the AAD.

// SECTION [epic=crypto-aead,seq=1] Imports & Konstanten
use crate::dek::Dek;
use crate::error::{CryptoError, Result};
use crate::secret::SecretKey32;
use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};
use contextra_ids::{CollectionId, DocId, TenantId};
use rand::RngCore;
use zeroize::Zeroizing;

pub const REC_AAD: &[u8] = b"ctx/rec/v2";

// !SECTION

// SECTION [epic=crypto-aead,seq=2] Rekord-Versiegelung (Seal)
// ANCHOR[contextra-crypto:AEAD-001] Authentifizierte Umschlag-Erzeugung mit Nonce-Bindung
// NOTE: Bindet Dokument-ID und Schlüssel in die AAD ein zur Replay-Vermeidung (TS:2026-11-11)
/// Seals a plaintext record into an envelope using an explicit, caller-provided 12-byte nonce.
/// Binds `doc` and `key_bytes` into the Associated Data (AAD) preventing replay/reordering across documents.
pub fn seal_record_with_nonce(
    dek: &Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
    nonce: [u8; 12],
) -> Result<Vec<u8>> {
    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(REC_AAD.len() + 16 + key_bytes.len());
    aad.extend_from_slice(REC_AAD);
    aad.extend_from_slice(&doc);
    aad.extend_from_slice(key_bytes);

    let payload = Payload {
        msg: plain,
        aad: &aad,
    };
    let ct = cipher
        .encrypt(Nonce::from_slice(&nonce), payload)
        .map_err(|e| CryptoError::Aead(format!("AEAD encryption failed: {e}")))?;

    let mut env = Vec::with_capacity(1 + 1 + 16 + 12 + ct.len());
    env.push(2); // version = 2
    env.push(1); // alg_id = 1
    env.extend_from_slice(&doc);
    env.extend_from_slice(&nonce);
    env.extend_from_slice(&ct);
    Ok(env)
}

/// Seals a plaintext record into an envelope using an injected RNG for nonce generation.
// ANCHOR[contextra-crypto:AEAD-002] Rekord-Versiegelung mit injiziertem RNG
pub fn seal_record_with_rng<R: RngCore>(
    dek: &Dek,
    doc: [u8; 16],
    key_bytes: &[u8],
    plain: &[u8],
    rng: &mut R,
) -> Result<Vec<u8>> {
    let mut nonce = [0u8; 12];
    // Fail-Closed: fehlbarer RNG-Pfad, kein Panic und kein deterministischer Fallback.
    rng.try_fill_bytes(&mut nonce)
        .map_err(|e| CryptoError::RngFailure(e.to_string()))?;
    seal_record_with_nonce(dek, doc, key_bytes, plain, nonce)
}

/// Seals a record using OsRng (convenience wrapper).
///
/// Fail-Closed: RNG- oder AEAD-Fehler werden als `Err` zurückgegeben. Es gibt
/// keinen stillen Fallback auf leere Umschläge.
// ANCHOR[contextra-crypto:AEAD-003] Standard-Versiegelung via OsRng
pub fn seal_record(dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8]) -> Result<Vec<u8>> {
    seal_record_with_rng(dek, doc, key_bytes, plain, &mut rand::rngs::OsRng)
}

// !SECTION

// SECTION [epic=crypto-aead,seq=3] Rekord-Entsiegelung (Open)
// ANCHOR[contextra-crypto:AEAD-004] Sichere Umschlag-Prüfung und Entschlüsselung
// NOTE: Fail-Closed Rückgabe ohne Orakel-Leckage (TS:2026-11-11)
/// Opens an encrypted record envelope, verifying the associated data and authentication tag.
/// Fails with a uniform `CryptoError::Aead` on any authentication or corruption issue without leaking oracle details.
pub fn open_record(dek: &Dek, key_bytes: &[u8], env: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    if env.len() < 1 + 1 + 16 + 12 + 16 {
        return Err(CryptoError::Aead("record envelope too short".into()));
    }
    if env.first().copied() != Some(2) {
        return Err(CryptoError::Aead("unsupported envelope version".into()));
    }
    if env.get(1).copied() != Some(1) {
        return Err(CryptoError::Aead("unsupported cipher algorithm".into()));
    }

    let doc = match env.get(2..18) {
        Some(d) => d,
        None => return Err(CryptoError::Aead("missing doc id".into())),
    };
    let nonce = match env.get(18..30) {
        Some(n) => n,
        None => return Err(CryptoError::Aead("missing nonce".into())),
    };
    let ct = match env.get(30..) {
        Some(c) => c,
        None => return Err(CryptoError::Aead("missing ciphertext".into())),
    };

    let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(dek.0.as_bytes()));
    let mut aad = Vec::with_capacity(REC_AAD.len() + 16 + key_bytes.len());
    aad.extend_from_slice(REC_AAD);
    aad.extend_from_slice(doc);
    aad.extend_from_slice(key_bytes);

    let payload = Payload { msg: ct, aad: &aad };
    let pt = cipher
        .decrypt(Nonce::from_slice(nonce), payload)
        .map_err(|_| CryptoError::Aead("AEAD authentication failed".into()))?;
    Ok(Zeroizing::new(pt))
}

// !SECTION

// SECTION [epic=crypto-aead,seq=4] Unit-Tests & Invarianten
#[cfg(test)]
mod tests {
    use super::*;
    use crate::secret::SecretKey;

    fn test_dek() -> Dek {
        let secret = SecretKey::new([0x55u8; 32]);
        Dek(secret)
    }

    #[test]
    fn test_aead_seal_open_roundtrip() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"cryptographic building block envelope content";

        let env = seal_record(&dek, doc, key_bytes, plaintext).expect("seal");
        let opened = open_record(&dek, key_bytes, &env).expect("open record");
        assert_eq!(opened.as_slice(), plaintext);
    }

    #[test]
    fn test_aead_seal_with_nonce() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"cryptographic building block envelope content";
        let nonce = [0x77u8; 12];

        let env = seal_record_with_nonce(&dek, doc, key_bytes, plaintext, nonce).expect("seal");
        let opened = open_record(&dek, key_bytes, &env).expect("open record");
        assert_eq!(opened.as_slice(), plaintext);
    }

    #[test]
    fn test_aead_negative_wrong_aad_fails() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let wrong_key_bytes = b"wrong_record_key";
        let plaintext = b"authenticated payload";

        let env = seal_record(&dek, doc, key_bytes, plaintext).expect("seal");
        let res = open_record(&dek, wrong_key_bytes, &env);
        assert!(
            matches!(res, Err(CryptoError::Aead(_))),
            "Wrong AAD must fail AEAD verification"
        );
    }

    #[test]
    fn test_aead_negative_bit_flip_fails() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"authenticated payload";

        let mut env = seal_record(&dek, doc, key_bytes, plaintext).expect("seal");
        let last_idx = env.len() - 1;
        env[last_idx] ^= 0x01; // flip 1 bit in tag/ciphertext

        let res = open_record(&dek, key_bytes, &env);
        assert!(
            matches!(res, Err(CryptoError::Aead(_))),
            "Bit-flip must fail AEAD verification"
        );
    }

    #[test]
    fn test_aead_negative_truncated_tag_fails() {
        let dek = test_dek();
        let doc = [0xAAu8; 16];
        let key_bytes = b"sample_record_key";
        let plaintext = b"authenticated payload";

        let env = seal_record(&dek, doc, key_bytes, plaintext).expect("seal");
        let truncated = &env[..env.len() - 8]; // truncate tag

        let res = open_record(&dek, key_bytes, truncated);
        assert!(
            matches!(res, Err(CryptoError::Aead(_))),
            "Truncated tag must fail AEAD verification"
        );
    }
}
// !SECTION

// SECTION [epic=crypto-hierarchy,seq=3] AEAD nach Spec §3.4 (AES-256-GCM-SIV)
// NOTE: Wire-Format `Nonce (12 Byte) || Ciphertext || Tag (16 Byte)`. Nonce ausschließlich aus
// dem OS-CSPRNG (fehlbar, kein deterministischer Fallback). Das bestehende `seal_record`-Format
// bleibt unverändert.

/// Baut den AAD-Kontext `tenant || collection || doc || version` (je 8 Byte, little-endian).
#[inline]
pub fn build_aad(tenant: TenantId, collection: CollectionId, doc: DocId, version: u64) -> [u8; 32] {
    let mut aad = [0u8; 32];
    let parts = [
        tenant.as_u64().to_le_bytes(),
        collection.as_u64().to_le_bytes(),
        doc.as_u64().to_le_bytes(),
        version.to_le_bytes(),
    ];
    for (chunk, part) in aad.chunks_exact_mut(8).zip(parts.iter()) {
        chunk.copy_from_slice(part);
    }
    aad
}

/// Verschlüsselt `plaintext` mit AES-256-GCM-SIV und bindet `aad` ein.
pub fn encrypt_aead(key: &SecretKey32, aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256GcmSiv::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::CipherInit(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng
        .try_fill_bytes(&mut nonce_bytes)
        .map_err(|e| CryptoError::RngFailure(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let mut out = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Entschlüsselt einen Blob aus `encrypt_aead` unter Validierung von `aad`.
pub fn decrypt_aead(key: &SecretKey32, aad: &[u8], blob: &[u8]) -> Result<Vec<u8>> {
    const NONCE_LEN: usize = 12;
    const TAG_LEN: usize = 16;

    let (nonce_bytes, ciphertext) = blob
        .split_at_checked(NONCE_LEN)
        .filter(|(_, ct)| ct.len() >= TAG_LEN)
        .ok_or(CryptoError::CiphertextTooShort)?;

    let cipher = Aes256GcmSiv::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::CipherInit(e.to_string()))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map_err(|_| CryptoError::DecryptionFailed)
}
// !SECTION
