//! Keyed BLAKE3 MAC (keine Verwendung als Inhalts-Hash).
use contextra_storage_ports::{traits::Mac, Key32};

#[derive(Debug, Default, Clone, Copy)]
pub struct Blake3Mac;

impl Mac for Blake3Mac {
    fn tag(&self, key: &Key32, data: &[u8]) -> [u8; 32] {
        *blake3::keyed_hash(key.expose(), data).as_bytes()
    }
}
