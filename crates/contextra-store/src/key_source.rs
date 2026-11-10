//! Kryptografische Schlüsselquellen-Implementierungen für contextra-store.
//!
//! Der Port-Trait [`KeyMaterialSource`] lebt in `contextra-ports::key_material` und
//! wird hier für Rückwärtskompatibilität re-exportiert.
//! Diese Datei enthält ausschließlich die konkreten Implementierungen:
//! - [`OsKeyMaterialSource`]: Produktive Implementierung via OS-CSPRNG.
//! - [`StaticKeyMaterialSource`]: Deterministische Test-Implementierung.

use contextra_types::{ContextraError, Result};

// Re-Export des Port-Traits aus contextra-ports — keine Duplizierung der Trait-Definition.
pub use contextra_ports::KeyMaterialSource;

/// Default production implementation using OS CSPRNG (`rand::rngs::OsRng`).
/// Fail-closed: returns error on entropy exhaustion or failure without fallback.
#[derive(Debug, Clone, Copy, Default)]
pub struct OsKeyMaterialSource;

impl KeyMaterialSource for OsKeyMaterialSource {
    fn fill_key_bytes(&self, dest: &mut [u8]) -> Result<()> {
        use rand::rngs::OsRng;
        use rand::RngCore;

        OsRng
            .try_fill_bytes(dest)
            .map_err(|e| ContextraError::Storage(format!("OS entropy source error: {e}")))
    }
}

/// Fixed deterministic key material source for reproducible testing and fault simulation.
#[derive(Debug, Clone)]
pub struct StaticKeyMaterialSource {
    key_stream: std::sync::Arc<parking_lot::Mutex<Vec<u8>>>,
}

impl StaticKeyMaterialSource {
    /// Creates a static key material source repeating or providing fixed byte slices.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            key_stream: std::sync::Arc::new(parking_lot::Mutex::new(bytes)),
        }
    }
}

impl KeyMaterialSource for StaticKeyMaterialSource {
    fn fill_key_bytes(&self, dest: &mut [u8]) -> Result<()> {
        let mut stream = self.key_stream.lock();
        if stream.is_empty() {
            // Fill with deterministic repeating pattern if exhausted
            for (i, b) in dest.iter_mut().enumerate() {
                *b = (i as u8).wrapping_add(0x42);
            }
            return Ok(());
        }
        let take = dest.len().min(stream.len());
        if let Some(src) = stream.get(..take) {
            if let Some(dst) = dest.get_mut(..take) {
                dst.copy_from_slice(src);
            }
        }
        stream.drain(..take);
        if take < dest.len() {
            if let Some(rest) = dest.get_mut(take..) {
                for (i, b) in rest.iter_mut().enumerate() {
                    *b = (i as u8).wrapping_add(0x42);
                }
            }
        }
        Ok(())
    }
}
