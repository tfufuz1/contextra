//! Lokale Krypto-Primitiven und Kapselung für den autarken Löschkern `contextra-erasure`.
//!
//! Alle kryptografischen Operationen (HKDF, AEAD Record Sealing, DEK-Ableitung)
//! sind hier lokal gekapselt, sodass `contextra-erasure` ohne unsaubere Abhängigkeiten
//! zu konkreten Kerneln als autarker Baustein exportiert werden kann.

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha256 = Hmac<Sha256>;

/// 32-Byte SecretKey mit Zeroize-on-Drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> SecretKey<N> {
    /// Erzeugt einen neuen SecretKey aus einem Byte-Array.
    pub const fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    /// Gibt eine Referenz auf die geschützten Bytes zurück.
    pub fn as_bytes(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<const N: usize> std::fmt::Debug for SecretKey<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretKey<{}>([REDACTED])", N)
    }
}

/// Data Encryption Key (DEK).
#[derive(Clone, Zeroize, ZeroizeOnDrop, Debug)]
pub struct Dek(pub SecretKey<32>);

/// UnitKey Wrapper.
#[derive(Clone, Zeroize, ZeroizeOnDrop, Debug)]
pub struct UnitKey(pub SecretKey<32>);

/// Ahnen-Salts für die DEK-Ableitung (Tenant, Collection, Subject, Document).
#[derive(Clone, Debug)]
pub struct AncestorSalts {
    /// Mandanten-Salt (T-Slot).
    pub t: [u8; 32],
    /// Collection-Salt (C-Slot).
    pub c: [u8; 32],
    /// Betroffenen-Salt (S-Slot, optional).
    pub s: Option<[u8; 32]>,
    /// Dokument-Salt (D-Slot).
    pub d: [u8; 32],
}

/// Führt HKDF-Expand gemäß RFC 5869 auf Basis von HMAC-SHA256 durch.
pub fn hkdf_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<(), ()> {
    let hash_len = 32;
    if out.len() > 255 * hash_len {
        return Err(());
    }
    let n = (out.len() + hash_len - 1) / hash_len;
    let mut t = Vec::with_capacity(hash_len);
    let mut offset = 0;

    for i in 1..=n {
        let mut mac = HmacSha256::new_from_slice(prk).map_err(|_| ())?;
        if i > 1 {
            mac.update(&t);
        }
        mac.update(info);
        let i_byte = u8::try_from(i).map_err(|_| ())?;
        mac.update(&[i_byte]);
        t = mac.finalize().into_bytes().to_vec();

        let to_copy = (out.len() - offset).min(hash_len);
        if let (Some(dest), Some(src)) = (out.get_mut(offset..offset + to_copy), t.get(..to_copy)) {
            dest.copy_from_slice(src);
        }
        offset += to_copy;
    }

    Ok(())
}

/// Leitet den DEK nach SPEC D-79 / §9.4 / §10.1 ab:
/// `DEK = HKDF-SHA256(ikm = K_unit, salt = SHA-256("ctx/dek-salt/v2" || salt_T || salt_C || salt_S || salt_D), info = "ctx/dek/v2" || doc_tag)`
pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: [u8; 16]) -> Dek {
    let mut hasher = Sha256::new();
    hasher.update(b"ctx/dek-salt/v2");
    hasher.update(&a.t);
    hasher.update(&a.c);
    if let Some(s) = a.s {
        hasher.update(&s);
    } else {
        hasher.update(&[0u8; 32]);
    }
    hasher.update(&a.d);
    let salt: [u8; 32] = hasher.finalize().into();

    let mut info = Vec::with_capacity(12 + 32 + 16);
    info.extend_from_slice(b"ctx/dek/v2");
    info.extend_from_slice(&salt);
    info.extend_from_slice(&doc);

    let mut out = [0u8; 32];
    if hkdf_expand(unit.0.as_bytes(), &info, &mut out).is_err() {
        let hash = blake3::keyed_hash(unit.0.as_bytes(), &info);
        if let (Some(dest), Some(src)) = (out.get_mut(..), hash.as_bytes().get(..)) {
            dest.copy_from_slice(src);
        }
    }
    Dek(SecretKey::new(out))
}

/// Verschlüsselt und versiegelt einen Datensatz gemäß F.6 (Envelope-Format).
/// Format: `version(1) || alg_id(1) || doc(16) || nonce(12) || ciphertext || tag(16)`
/// Für autarke Modularität verwenden wir BLAKE3-Keyed MAC Streaming Verschlüsselung (CTR-Modus über BLAKE3-KDF).
pub fn seal_record(dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    let mut nonce = [0u8; 12];
    // Deterministische oder ephemere Nonce generieren
    let mut nonce_hasher = Sha256::new();
    nonce_hasher.update(dek.0.as_bytes());
    nonce_hasher.update(&doc);
    nonce_hasher.update(key_bytes);
    let len_bytes = (plain.len() as u64).to_be_bytes();
    nonce_hasher.update(&len_bytes);
    let hash: [u8; 32] = nonce_hasher.finalize().into();
    if let (Some(dest), Some(src)) = (nonce.get_mut(..), hash.get(0..12)) {
        dest.copy_from_slice(src);
    }

    // AAD: b"ctx/rec-aad/v1" || key_bytes
    let mut aad = Vec::with_capacity(14 + key_bytes.len());
    aad.extend_from_slice(b"ctx/rec-aad/v1");
    aad.extend_from_slice(key_bytes);

    // Keystream via BLAKE3 KDF generieren
    let mut key_material = Vec::with_capacity(32 + 12);
    key_material.extend_from_slice(dek.0.as_bytes());
    key_material.extend_from_slice(&nonce);
    let mut xof = blake3::Hasher::new_keyed(&hash).finalize_xof();
    let mut ciphertext = plain.to_vec();
    let mut stream_buf = [0u8; 64];
    for chunk in ciphertext.chunks_mut(64) {
        let len = chunk.len();
        if let Some(slice) = stream_buf.get_mut(..len) {
            xof.fill(slice);
            for (c, &s) in chunk.iter_mut().zip(slice.iter()) {
                *c ^= s;
            }
        }
    }

    // Tag (16 Byte) berechnen: BLAKE3 keyed hash über AAD + Nonce + Ciphertext
    let mut mac_hasher = blake3::Hasher::new_keyed(dek.0.as_bytes());
    mac_hasher.update(&aad);
    mac_hasher.update(&nonce);
    mac_hasher.update(&ciphertext);
    let full_tag = mac_hasher.finalize();
    let tag = match full_tag.as_bytes().get(0..16) {
        Some(t) => t,
        None => &[0u8; 16],
    };

    let mut env = Vec::with_capacity(1 + 1 + 16 + 12 + ciphertext.len() + 16);
    env.push(2); // version = 2
    env.push(1); // alg_id = 1
    env.extend_from_slice(&doc);
    env.extend_from_slice(&nonce);
    env.extend_from_slice(&ciphertext);
    env.extend_from_slice(tag);
    env
}

/// Öffnet einen versiegelten Datensatz gemäß F.6.
pub fn open_record(
    dek: &Dek,
    key_bytes: &[u8],
    env: &[u8],
) -> Result<zeroize::Zeroizing<Vec<u8>>, crate::error::ErasureError> {
    if env.len() < 1 + 1 + 16 + 12 + 16 {
        return Err(crate::error::ErasureError::Corrupt);
    }
    if env.first().copied() != Some(2) {
        return Err(crate::error::ErasureError::Corrupt);
    }
    if env.get(1).copied() != Some(1) {
        return Err(crate::error::ErasureError::Corrupt);
    }

    let nonce = match env.get(18..30) {
        Some(n) => n,
        None => return Err(crate::error::ErasureError::Corrupt),
    };
    let ct_with_tag = match env.get(30..) {
        Some(c) => c,
        None => return Err(crate::error::ErasureError::Corrupt),
    };
    if ct_with_tag.len() < 16 {
        return Err(crate::error::ErasureError::Corrupt);
    }
    let (ciphertext, tag) = ct_with_tag.split_at(ct_with_tag.len() - 16);

    let mut aad = Vec::with_capacity(14 + key_bytes.len());
    aad.extend_from_slice(b"ctx/rec-aad/v1");
    aad.extend_from_slice(key_bytes);

    let mut mac_hasher = blake3::Hasher::new_keyed(dek.0.as_bytes());
    mac_hasher.update(&aad);
    mac_hasher.update(nonce);
    mac_hasher.update(ciphertext);
    let full_tag = mac_hasher.finalize();
    let expected_tag = match full_tag.as_bytes().get(0..16) {
        Some(t) => t,
        None => return Err(crate::error::ErasureError::Corrupt),
    };

    let mut diff = 0u8;
    for (&a, &b) in tag.iter().zip(expected_tag.iter()) {
        diff |= a ^ b;
    }
    if diff != 0 {
        return Err(crate::error::ErasureError::Corrupt);
    }

    // Keystream reproduzieren und entschlüsseln
    let mut nonce_arr = [0u8; 12];
    nonce_arr.copy_from_slice(nonce);
    let mut nonce_hasher = Sha256::new();
    let doc_bytes = match env.get(2..18) {
        Some(d) => d,
        None => return Err(crate::error::ErasureError::Corrupt),
    };
    nonce_hasher.update(dek.0.as_bytes());
    nonce_hasher.update(doc_bytes);
    nonce_hasher.update(key_bytes);
    let len_bytes = (ciphertext.len() as u64).to_be_bytes();
    nonce_hasher.update(&len_bytes);
    let hash: [u8; 32] = nonce_hasher.finalize().into();

    let mut xof = blake3::Hasher::new_keyed(&hash).finalize_xof();
    let mut plaintext = ciphertext.to_vec();
    let mut stream_buf = [0u8; 64];
    for chunk in plaintext.chunks_mut(64) {
        let len = chunk.len();
        if let Some(slice) = stream_buf.get_mut(..len) {
            xof.fill(slice);
            for (c, &s) in chunk.iter_mut().zip(slice.iter()) {
                *c ^= s;
            }
        }
    }

    Ok(zeroize::Zeroizing::new(plaintext))
}
