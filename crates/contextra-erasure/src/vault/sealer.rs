//! RecordSealer-Implementierung (SPEC §4.2, D-53, D-79).

use contextra_ports::gate::{KeyBuf, KeyRange};
use contextra_ports::keys::{Envelope, RecordSealer, SealCtx};
use contextra_ports::BoxFuture;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

/// In-Memory RecordSealer zur Kapselung von Datensätzen in Envelopes.
#[derive(Clone, Default)]
pub struct VaultSealer {
    dek_cache: Arc<RwLock<BTreeMap<KeyBuf, [u8; 32]>>>,
}

impl VaultSealer {
    /// Erstellt einen neuen `VaultSealer`.
    pub fn new_in_memory() -> Self {
        Self {
            dek_cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl RecordSealer for VaultSealer {
    fn seal<'a>(
        &'a self,
        _ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>> {
        Box::pin(async move {
            let dek = {
                let mut cache = self.dek_cache.write();
                if let Some(existing) = cache.get(key) {
                    *existing
                } else {
                    let hash = blake3::hash(key.as_bytes());
                    let d = *hash.as_bytes();
                    cache.insert(*key, d);
                    d
                }
            };

            let mut payload = Vec::with_capacity(plain.len());
            let stream_key = blake3::keyed_hash(&dek, b"ctx/envelope/stream");
            let mask = stream_key.as_bytes();
            for (i, &b) in plain.iter().enumerate() {
                let mask_byte = mask.get(i % 32).copied().unwrap_or(0);
                payload.push(b ^ mask_byte);
            }

            Ok(Envelope::new(payload))
        })
    }

    fn open<'a>(
        &'a self,
        key: &'a KeyBuf,
        env: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<u8>, String>> {
        Box::pin(async move {
            let dek = {
                let cache = self.dek_cache.read();
                match cache.get(key) {
                    Some(&k) => k,
                    None => {
                        let hash = blake3::hash(key.as_bytes());
                        *hash.as_bytes()
                    }
                }
            };

            let mut plain = Vec::with_capacity(env.len());
            let stream_key = blake3::keyed_hash(&dek, b"ctx/envelope/stream");
            let mask = stream_key.as_bytes();
            for (i, &b) in env.iter().enumerate() {
                let mask_byte = mask.get(i % 32).copied().unwrap_or(0);
                plain.push(b ^ mask_byte);
            }

            Ok(plain)
        })
    }

    fn evict(&self, scope: &KeyRange) {
        let mut cache = self.dek_cache.write();
        cache.retain(|k, v| {
            if k.as_bytes() >= scope.start.as_bytes() && k.as_bytes() < scope.end.as_bytes() {
                v.fill(0);
                false
            } else {
                true
            }
        });
    }

    fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move { Ok(()) })
    }
}
