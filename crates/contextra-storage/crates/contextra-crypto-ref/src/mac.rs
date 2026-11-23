//! MAC reference implementations; keyed hashes are never content identifiers.
use contextra_storage_ports::{traits::Mac, Key32};
use sha2::{Digest, Sha256};

#[derive(Debug, Default, Clone, Copy)]
pub struct Blake3Mac;

impl Mac for Blake3Mac {
    fn tag(&self, key: &Key32, data: &[u8]) -> [u8; 32] {
        *blake3::keyed_hash(key.expose(), data).as_bytes()
    }
}

/// HMAC-SHA256 required by the Authenticated SealFrame mode (E-68).
#[derive(Debug, Default, Clone, Copy)]
pub struct HmacSha256;

impl Mac for HmacSha256 {
    fn tag(&self, key: &Key32, data: &[u8]) -> [u8; 32] {
        let mut inner_pad = [0x36u8; 64];
        let mut outer_pad = [0x5cu8; 64];
        for (index, byte) in key.expose().iter().copied().enumerate() {
            if let Some(inner) = inner_pad.get_mut(index) { *inner ^= byte; }
            if let Some(outer) = outer_pad.get_mut(index) { *outer ^= byte; }
        }
        let mut inner = Sha256::new();
        inner.update(inner_pad);
        inner.update(data);
        let inner_digest = inner.finalize();
        let mut outer = Sha256::new();
        outer.update(outer_pad);
        outer.update(inner_digest);
        outer.finalize().into()
    }
}
