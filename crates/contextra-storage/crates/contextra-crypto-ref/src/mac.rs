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

#[cfg(test)]
mod tests {
    use super::{HmacSha256, Mac};
    use contextra_storage_ports::Key32;

    #[test]
    fn hmac_sha256_known_answer_for_fixed_width_key() {
        let key = Key32::from_array([0x0b; 32]);
        let actual = HmacSha256.tag(&key, b"Hi There");
        let expected = [
            0x19, 0x8a, 0x60, 0x7e, 0xb4, 0x4b, 0xfb, 0xc6,
            0x99, 0x03, 0xa0, 0xf1, 0xcf, 0x2b, 0xbd, 0xc5,
            0xba, 0x0a, 0xa3, 0xf3, 0xd9, 0xae, 0x3c, 0x1c,
            0x7a, 0x3b, 0x16, 0x96, 0xa0, 0xb6, 0x8c, 0xf7,
        ];
        assert_eq!(actual, expected);
    }
}
