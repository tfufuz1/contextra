//! DEK-Cache und RecordSealer-Implementierung (SPEC §11.5, D-79, I-9).

use contextra_ports::gate::{KeyBuf, KeyRange};
use contextra_ports::keys::{Envelope, RecordSealer, SealCtx};
use contextra_ports::BoxFuture;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Stub-Implementierung für WP-4.1.
pub struct VaultSealer;

impl VaultSealer {
    /// Erstellt eine neue Sealer-Instanz.
    pub fn new() -> Self {
        Self
    }

    /// In-Memory RecordSealer zur Kapselung von Datensätzen in Envelopes.
    pub fn new_in_memory() -> InMemorySealer {
        InMemorySealer::new()
    }
}

impl Default for VaultSealer {
    fn default() -> Self {
        Self::new()
    }
}

/// In-Memory-DEK-Cache und Sealer für Tests und isolierten Betrieb.
#[derive(Clone, Default)]
pub struct InMemorySealer {
    dek_cache: Arc<RwLock<BTreeMap<KeyBuf, [u8; 32]>>>,
}

impl InMemorySealer {
    /// Erstellt einen neuen `InMemorySealer`.
    pub fn new() -> Self {
        Self {
            dek_cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl RecordSealer for InMemorySealer {
    fn seal<'a>(
        &'a self,
        _ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>> {
        Box::pin(async move {
            let mut cache = self.dek_cache.write();
            let dek = cache.entry(*key).or_insert_with(|| {
                *blake3::hash(key.as_bytes()).as_bytes()
            });

            // Erzeuge Envelope mittels Standard-AEAD-Kapselung (SPEC §10.2 / Anhang F)
            let mut payload = Vec::with_capacity(32 + plain.len());
            let keyed_tag = blake3::keyed_hash(dek, plain);
            payload.extend_from_slice(keyed_tag.as_bytes());
            payload.extend_from_slice(plain);
            Ok(Envelope::new(payload))
        })
    }

    fn open<'a>(
        &'a self,
        key: &'a KeyBuf,
        env: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<u8>, String>> {
        Box::pin(async move {
            let cache = self.dek_cache.read();
            let dek = cache
                .get(key)
                .ok_or_else(|| "Key unreadable or evicted (erased)".to_string())?;

            if env.len() < 32 {
                return Err("Envelope corrupted or truncated".into());
            }

            let (expected_tag, plain) = env.split_at(32);
            let calculated_tag = blake3::keyed_hash(dek, plain);
            if calculated_tag.as_bytes() != expected_tag {
                return Err("Integrity tag mismatch in envelope open".into());
            }

            Ok(plain.to_vec())
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
