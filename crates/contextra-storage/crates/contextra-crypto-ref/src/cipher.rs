//! XChaCha20-Poly1305 adapter for the storage crypto port.
use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use contextra_storage_ports::{traits::Cipher, Error, Key32};
use zeroize::Zeroizing;

#[derive(Debug, Default, Clone, Copy)]
pub struct XChaCha20Poly1305Cipher;

impl Cipher for XChaCha20Poly1305Cipher {
    fn seal(&self, key: &Key32, nonce: &[u8; 24], aad: &[u8], plaintext: &[u8], out: &mut Vec<u8>) -> Result<(), Error> {
        let cipher = XChaCha20Poly1305::new_from_slice(key.expose())
            .map_err(|_| Error::Limit { limit: "XChaCha20-Poly1305 key length" })?;
        let mut ciphertext = Zeroizing::new(plaintext.to_vec());
        let tag = cipher.encrypt_in_place_detached(XNonce::from_slice(nonce), aad, &mut ciphertext)
            .map_err(|_| Error::Corrupt { place: "AEAD seal" })?;
        out.reserve(ciphertext.len().saturating_add(tag.len()));
        out.extend_from_slice(&ciphertext);
        out.extend_from_slice(&tag);
        Ok(())
    }

    fn open(&self, key: &Key32, nonce: &[u8; 24], aad: &[u8], ciphertext_and_tag: &[u8], out: &mut Vec<u8>) -> Result<(), Error> {
        let tag_start = ciphertext_and_tag.len().checked_sub(16)
            .ok_or(Error::Corrupt { place: "AEAD frame" })?;
        let cipher = XChaCha20Poly1305::new_from_slice(key.expose())
            .map_err(|_| Error::Limit { limit: "XChaCha20-Poly1305 key length" })?;
        let ciphertext = ciphertext_and_tag.get(..tag_start)
            .ok_or(Error::Corrupt { place: "AEAD frame" })?;
        let tag_bytes = ciphertext_and_tag.get(tag_start..)
            .ok_or(Error::Corrupt { place: "AEAD frame" })?;
        let mut plaintext = Zeroizing::new(ciphertext.to_vec());
        let tag = chacha20poly1305::Tag::from_slice(tag_bytes);
        cipher.decrypt_in_place_detached(XNonce::from_slice(nonce), aad, &mut plaintext, tag)
            .map_err(|_| Error::Corrupt { place: "AEAD authentication" })?;
        out.extend_from_slice(&plaintext);
        Ok(())
    }
}
