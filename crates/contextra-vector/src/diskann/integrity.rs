//! Standalone DiskANN integrity hasher replacing the external `contextra-crypto` edge.

use contextra_types::{ContextraError, Result};

/// Standalone keyed BLAKE3 hasher for DiskANN file integrity.
pub struct DiskAnnHmac {
    hasher: blake3::Hasher,
}

impl DiskAnnHmac {
    /// Creates a new `DiskAnnHmac` using the provided integrity key.
    pub fn new(integrity_key: &[u8]) -> Result<Self> {
        if integrity_key.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Key cannot be empty".to_string(),
            ));
        }
        if integrity_key.len() > 10_000 {
            return Err(ContextraError::InvalidInput(format!(
                "integrity_key length {} exceeds maximum allowed bound of 10000 bytes",
                integrity_key.len()
            )));
        }
        let key_bytes: [u8; 32] = if integrity_key.len() == 32 {
            let mut k = [0u8; 32];
            k.copy_from_slice(integrity_key);
            k
        } else {
            *blake3::hash(integrity_key).as_bytes()
        };
        let mut hasher = blake3::Hasher::new_keyed(&key_bytes);
        hasher.update(b"contextra-diskann-v1");
        Ok(Self { hasher })
    }

    /// Feeds new data into the integrity calculation.
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalizes the calculation and yields the 32-byte digest.
    pub fn finalize(self) -> [u8; 32] {
        *self.hasher.finalize().as_bytes()
    }
}
