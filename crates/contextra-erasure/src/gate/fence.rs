//! # Fence-Protokoll (INV-ERA-5 / WP-6).
//!
//! Koordiniert die Sperrung von Schreib- und Lesezugriffen während eines physischen Purge-Laufs.

use crate::error::ErasureError;
use crate::types_local::{BoxFuture, FenceGate, FencePermit, KeyBuf, KeyScope};
use parking_lot::RwLock;
use std::collections::BTreeSet;
use std::sync::Arc;

/// In-Memory-Referenzimplementierung von [`FenceGate`].
#[derive(Default)]
pub struct MemoryFenceGate {
    fenced_keys: RwLock<BTreeSet<[u8; 16]>>,
}

impl MemoryFenceGate {
    /// Erzeugt ein neues `MemoryFenceGate`.
    pub fn new() -> Self {
        Self {
            fenced_keys: RwLock::new(BTreeSet::new()),
        }
    }

    /// Fügt einen Schlüssel manuell zum Fence hinzu.
    pub fn add_fenced_key(&self, key: [u8; 16]) {
        self.fenced_keys.write().insert(key);
    }

    /// Entfernt einen Schlüssel aus dem Fence.
    pub fn remove_fenced_key(&self, key: &[u8; 16]) {
        self.fenced_keys.write().remove(key);
    }
}

impl FenceGate for MemoryFenceGate {
    fn acquire_fence<'a>(
        &'a self,
        _scope: &'a KeyScope,
    ) -> BoxFuture<'a, Result<FencePermit, ErasureError>> {
        Box::pin(async move {
            // INV-ERA-5: Erzeugt ein RAII-Token für den aktiven Fence
            Ok(FencePermit::new(Some(Arc::new(()))))
        })
    }

    fn is_fenced(&self, key: &KeyBuf) -> bool {
        self.fenced_keys.read().contains(key.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_fence_gate_basic() {
        let gate = MemoryFenceGate::new();
        let key = KeyBuf::new([0xAA; 16]);
        assert!(!gate.is_fenced(&key));

        gate.add_fenced_key([0xAA; 16]);
        assert!(gate.is_fenced(&key));

        gate.remove_fenced_key(&[0xAA; 16]);
        assert!(!gate.is_fenced(&key));
    }
}
