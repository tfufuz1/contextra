//! Umschlag v1 (Spec 4.5): XChaCha20-Poly1305 mit internem Nonce und Größenklassen-Padding.
//!
//! Layout: `ver u8 · alg u8 · key_class u8 · nonce (24) · ciphertext · tag (16)`.
//! Das Ciphertext trägt den gepolsterten, gerahmten Klartext. Die AAD bindet Ebene,
//! Mandant, Keyspace, Datensatzkennung und Version.
//!
//! Die Nonce entsteht im Umschlag aus dem Port `CryptoRng`. Scheitert er, entsteht kein
//! Umschlag (Z10). Ebenen sind Typen (`Sealed<P>`), damit eine Ebene nicht mit einer
//! anderen verwechselt werden kann.

use crate::domain;
use crate::error::CryptoError;
use crate::key::private;
use crate::pad;
use acta_base::CryptoRng;
use chacha20poly1305::aead::AeadInPlace;
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use std::marker::PhantomData;
use zeroize::Zeroizing;

/// Versionsbyte des Umschlags.
pub const ENVELOPE_VERSION: u8 = 0x01;
/// Algorithmusbyte für XChaCha20-Poly1305. Das Byte `0x02` ist für AES-256-GCM reserviert.
pub const ALG_XCHACHA20_POLY1305: u8 = 0x01;
/// Nonce-Länge in Byte.
pub const NONCE_LEN: usize = 24;
/// Tag-Länge in Byte.
pub const TAG_LEN: usize = 16;
const HEADER_LEN: usize = 3 + NONCE_LEN;

/// Schlüssel, mit dem ein Umschlag versiegelt wird. Nur Schlüsseltypen aus `crate::key`
/// implementieren ihn. Der Trait liefert nur den Cipher und nie das Schlüsselmaterial.
pub trait SealKey: private::Sealed {
    /// AEAD-Cipher unter diesem Schlüssel.
    ///
    /// # Errors
    /// [`CryptoError::Internal`] nur bei internem Fehler (Schlüssellänge ist fest).
    fn cipher(&self) -> Result<XChaCha20Poly1305, CryptoError>;
}

/// Ebene eines Umschlags. Die Konstante `CLASS` steht im Umschlag und in der AAD.
pub trait Plane: Send + Sync + 'static {
    /// Klassenbyte: P = 0x01, K = 0x02, D = 0x03.
    const CLASS: u8;
}

/// Ebene P: Personendaten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Personal;
/// Ebene K: versioniertes Wissen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Knowledge;
/// Ebene D: Ableitungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Derived;

impl Plane for Personal {
    const CLASS: u8 = 0x01;
}
impl Plane for Knowledge {
    const CLASS: u8 = 0x02;
}
impl Plane for Derived {
    const CLASS: u8 = 0x03;
}

/// Ein versiegelter Umschlag der Ebene `P`. Die Bytes sind Chiffrat, kein Geheimnis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sealed<P: Plane> {
    bytes: Vec<u8>,
    plane: PhantomData<P>,
}

impl<P: Plane> Sealed<P> {
    /// Übernimmt geladene Bytes. Prüft Version, Verfahren, Ebene und Mindestlänge.
    ///
    /// # Errors
    /// [`CryptoError::Malformed`] bei zu kurzen Bytes, [`CryptoError::Unsupported`] bei
    /// unbekannter Version, unbekanntem Verfahren oder anderer Ebene.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        if bytes.len() < HEADER_LEN + TAG_LEN {
            return Err(CryptoError::Malformed);
        }
        let version = bytes.first().copied().ok_or(CryptoError::Malformed)?;
        let alg = bytes.get(1).copied().ok_or(CryptoError::Malformed)?;
        let class = bytes.get(2).copied().ok_or(CryptoError::Malformed)?;
        if version != ENVELOPE_VERSION || alg != ALG_XCHACHA20_POLY1305 || class != P::CLASS {
            return Err(CryptoError::Unsupported);
        }
        Ok(Self {
            bytes,
            plane: PhantomData,
        })
    }

    /// Die Umschlagbytes, wie sie gespeichert werden.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Kontext der AAD: Mandant, Keyspace, Datensatzkennung und Version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealCtx {
    /// Mandantenkennung (16 Byte).
    pub tenant: [u8; 16],
    /// Keyspace-Kennung (Spec 4.5).
    pub keyspace: u16,
    /// Datensatzkennung (16 Byte).
    pub record_ref: [u8; 16],
    /// Version des Datensatzes.
    pub version: u64,
}

fn aad<P: Plane>(ctx: &SealCtx) -> Vec<u8> {
    let mut out = Vec::with_capacity(domain::REC.len() + 16 + 1 + 2 + 16 + 8);
    out.extend_from_slice(domain::REC);
    out.extend_from_slice(&ctx.tenant);
    out.push(P::CLASS);
    out.extend_from_slice(&ctx.keyspace.to_be_bytes());
    out.extend_from_slice(&ctx.record_ref);
    out.extend_from_slice(&ctx.version.to_be_bytes());
    out
}

/// Versiegelt `plaintext` unter `key`. Die Nonce kommt aus `rng`.
///
/// # Errors
/// [`CryptoError::Entropy`], wenn `rng` versagt (dann entsteht kein Umschlag),
/// [`CryptoError::TooLarge`] bei übergroßer Nutzlast.
pub fn seal<P: Plane, K: SealKey>(
    key: &K,
    ctx: &SealCtx,
    plaintext: &[u8],
    rng: &dyn CryptoRng,
) -> Result<Sealed<P>, CryptoError> {
    let mut nonce = [0_u8; NONCE_LEN];
    rng.try_fill(&mut nonce)?;
    let mut buf = pad::pad(plaintext)?;
    let tag = key
        .cipher()?
        .encrypt_in_place_detached(&XNonce::from(nonce), &aad::<P>(ctx), buf.as_mut_slice())
        .map_err(|_| CryptoError::Internal)?;

    let mut bytes = Vec::with_capacity(HEADER_LEN + buf.len() + TAG_LEN);
    bytes.push(ENVELOPE_VERSION);
    bytes.push(ALG_XCHACHA20_POLY1305);
    bytes.push(P::CLASS);
    bytes.extend_from_slice(&nonce);
    bytes.extend_from_slice(&buf);
    bytes.extend_from_slice(tag.as_slice());
    Ok(Sealed {
        bytes,
        plane: PhantomData,
    })
}

/// Öffnet einen Umschlag. Der Klartext liegt in `Zeroizing`.
///
/// # Errors
/// [`CryptoError::DecryptionFailed`] bei falschem Schlüssel, falscher AAD oder Manipulation.
/// Ohne Unterscheidung nach Ursache. [`CryptoError::Malformed`] bei fehlerhaftem Padding
/// nach erfolgreicher Authentisierung.
pub fn open<P: Plane, K: SealKey>(
    key: &K,
    ctx: &SealCtx,
    sealed: &Sealed<P>,
) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let bytes = sealed.as_bytes();
    let nonce_part = bytes.get(3..HEADER_LEN).ok_or(CryptoError::Malformed)?;
    let nonce = <[u8; NONCE_LEN]>::try_from(nonce_part).map_err(|_| CryptoError::Malformed)?;
    let body = bytes.get(HEADER_LEN..).ok_or(CryptoError::Malformed)?;
    let ct_len = body
        .len()
        .checked_sub(TAG_LEN)
        .ok_or(CryptoError::Malformed)?;
    let ct = body.get(..ct_len).ok_or(CryptoError::Malformed)?;
    let tag_part = body.get(ct_len..).ok_or(CryptoError::Malformed)?;
    let tag = <[u8; TAG_LEN]>::try_from(tag_part).map_err(|_| CryptoError::Malformed)?;

    let mut buf = Zeroizing::new(ct.to_vec());
    key.cipher()?
        .decrypt_in_place_detached(
            &XNonce::from(nonce),
            &aad::<P>(ctx),
            buf.as_mut_slice(),
            &chacha20poly1305::Tag::from(tag),
        )
        .map_err(|_| CryptoError::DecryptionFailed)?;
    let payload = pad::unpad(&buf)?;
    Ok(Zeroizing::new(payload.to_vec()))
}
