//! AEAD-Umhüllung von RootKey und Space-DEK (SP-KEYS, INV-KEY-1).

use contextra_storage_ports::traits::{Cipher, Entropy};
use contextra_storage_ports::{Error, Key32, SpaceId};
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"KWR1";
const NONCE_BYTES: usize = 24;
const KEY_BYTES: usize = 32;
const TAG_BYTES: usize = 16;
const WRAPPED_LEN: usize = 4 + NONCE_BYTES + KEY_BYTES + TAG_BYTES;

pub fn wrap_root(
    cipher: &dyn Cipher,
    kek: &Key32,
    root: &Key32,
    entropy: &dyn Entropy,
) -> Result<Vec<u8>, Error> {
    wrap_key(cipher, kek, root, entropy, b"contextra root key v1")
}

pub fn unwrap_root(cipher: &dyn Cipher, kek: &Key32, wrapped: &[u8]) -> Result<Key32, Error> {
    unwrap_key(cipher, kek, wrapped, b"contextra root key v1")
}

pub fn wrap_space_dek(
    cipher: &dyn Cipher,
    root: &Key32,
    dek: &Key32,
    entropy: &dyn Entropy,
    space: SpaceId,
) -> Result<Vec<u8>, Error> {
    let mut label = b"contextra space dek v1".to_vec();
    label.extend_from_slice(&space.0.to_be_bytes());
    wrap_key(cipher, root, dek, entropy, &label)
}

pub fn unwrap_space_dek(
    cipher: &dyn Cipher,
    root: &Key32,
    wrapped: &[u8],
    space: SpaceId,
) -> Result<Key32, Error> {
    let mut label = b"contextra space dek v1".to_vec();
    label.extend_from_slice(&space.0.to_be_bytes());
    unwrap_key(cipher, root, wrapped, &label)
}

fn wrap_key(
    cipher: &dyn Cipher,
    wrapping_key: &Key32,
    plaintext: &Key32,
    entropy: &dyn Entropy,
    domain: &[u8],
) -> Result<Vec<u8>, Error> {
    let mut nonce = [0u8; NONCE_BYTES];
    entropy.fill(&mut nonce)?;
    let mut aad = Vec::with_capacity(MAGIC.len() + domain.len());
    aad.extend_from_slice(MAGIC);
    aad.extend_from_slice(domain);
    let mut encrypted = Zeroizing::new(Vec::with_capacity(KEY_BYTES + TAG_BYTES));
    cipher.seal(wrapping_key, &nonce, &aad, plaintext.expose(), &mut encrypted)?;
    if encrypted.len() != KEY_BYTES + TAG_BYTES {
        return Err(Error::Corrupt { place: "wrapped key length" });
    }
    let mut output = Vec::with_capacity(WRAPPED_LEN);
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&encrypted);
    Ok(output)
}

fn unwrap_key(
    cipher: &dyn Cipher,
    wrapping_key: &Key32,
    wrapped: &[u8],
    domain: &[u8],
) -> Result<Key32, Error> {
    if wrapped.len() != WRAPPED_LEN || wrapped.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
        return Err(Error::Corrupt { place: "wrapped key header" });
    }
    let nonce_start = MAGIC.len();
    let nonce_end = nonce_start + NONCE_BYTES;
    let nonce: &[u8; NONCE_BYTES] = wrapped.get(nonce_start..nonce_end)
        .ok_or(Error::Corrupt { place: "wrapped key nonce" })?
        .try_into().map_err(|_| Error::Corrupt { place: "wrapped key nonce" })?;
    let mut aad = Vec::with_capacity(MAGIC.len() + domain.len());
    aad.extend_from_slice(MAGIC);
    aad.extend_from_slice(domain);
    let mut plaintext = Zeroizing::new(Vec::with_capacity(KEY_BYTES));
    cipher.open(wrapping_key, nonce, &aad, wrapped.get(nonce_end..)
        .ok_or(Error::Corrupt { place: "wrapped key payload" })?, &mut plaintext)?;
    let key: [u8; KEY_BYTES] = plaintext.as_slice().try_into()
        .map_err(|_| Error::Corrupt { place: "unwrapped key size" })?;
    Ok(Key32::from_array(key))
}
