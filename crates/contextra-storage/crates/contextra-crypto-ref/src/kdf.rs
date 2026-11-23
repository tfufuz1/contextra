//! HKDF-SHA256 labels and HChaCha20 fast per-nonce subkeys.
use chacha20::cipher::{consts::{U10, U16}, generic_array::GenericArray};
use contextra_storage_ports::{traits::Kdf, Error, Key32};
use hkdf::Hkdf;
use sha2::Sha256;

#[derive(Debug, Default, Clone, Copy)]
pub struct HkdfSha256;

impl Kdf for HkdfSha256 {
    fn derive(&self, master: &Key32, label: &[u8]) -> Result<Key32, Error> {
        let hkdf = Hkdf::<Sha256>::new(Some(b"contextra-storage-kdf-v1"), master.expose());
        let mut output = [0u8; 32];
        hkdf.expand(label, &mut output).map_err(|_| Error::Limit { limit: "KDF output length" })?;
        Ok(Key32::from_array(output))
    }

    fn derive_subkey_fast(&self, master: &Key32, label: &[u8], nonce: &[u8; 24]) -> Result<Key32, Error> {
        let base = self.derive(master, label)?;
        let key = chacha20::Key::from_slice(base.expose());
        let input = GenericArray::<u8, U16>::from_slice(&nonce[..16]);
        let derived = chacha20::hchacha::<U10>(key, input);
        let mut output = [0u8; 32];
        output.copy_from_slice(derived.as_slice());
        Ok(Key32::from_array(output))
    }
}
