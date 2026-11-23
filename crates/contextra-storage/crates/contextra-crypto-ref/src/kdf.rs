//! Stable HKDF-SHA256 key-role derivation.
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
}
