//! Stage-2 cryptographic record sealing port contracts (WP-11).
//!
//! Retained behind `cfg(feature = "stage2")` until Stage-2 implementation.

use crate::erasure::KeyScope;
use crate::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Sealing context containing metadata and epoch tokens for envelope creation.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealCtx {
    /// Epoch sequence or slot identifier.
    pub epoch: u64,
}

impl SealCtx {
    /// Creates a new sealing context.
    pub const fn new(epoch: u64) -> Self {
        Self { epoch }
    }
}

impl fmt::Debug for SealCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SealCtx {{ epoch: {} }}", self.epoch)
    }
}

/// Sealed record envelope containing encrypted ciphertext and tag.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    payload: Vec<u8>,
}

impl Envelope {
    /// Creates a new envelope.
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    /// Returns payload bytes.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl fmt::Debug for Envelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Mask ciphertext / plaintext (PO-08)
        write!(f, "Envelope([REDACTED, len={}])", self.payload.len())
    }
}

/// Record sealer contract for crypto envelope management (Stage 2, WP-11).
pub trait RecordSealer: Send + Sync {
    /// Encrypts and seals plaintext with slot allocation if missing.
    ///
    /// # Fehler
    /// Liefert Fehler falls Verschlüsselung oder Slot-Allokation fehlschlägt.
    ///
    /// # Crash
    /// Vorübergehende Daten im RAM; Persistierung der Slots erfolgt vor dem Commit.
    fn seal<'a>(
        &'a self,
        ctx: &'a SealCtx,
        key: &'a [u8],
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>>;

    /// Decrypts envelope. If slot was destroyed/missing, returns error indicating erasure.
    ///
    /// # Fehler
    /// Liefert Fehler falls Schlüssel vernichtet wurde oder Entschlüsselung fehlschlägt.
    ///
    /// # Crash
    /// Reiner kryptografischer Entschlüsselungszugriff ohne Seiteneffekte.
    fn open<'a>(&'a self, key: &'a [u8], env: &'a [u8]) -> BoxFuture<'a, Result<Vec<u8>, String>>;

    /// Evicts key scope from DEK cache, zeroizing RAM keys.
    ///
    /// # Fehler
    /// Synchrone Operation ohne Fehlerzustände.
    ///
    /// # Crash
    /// Entfernt und nullt Schlüsselmaterial flüchtig im RAM.
    fn evict(&self, scope: &KeyScope);

    /// Forces persistence of vault slots (`sync_data`) before WAL frame flush.
    ///
    /// # Fehler
    /// Liefert Fehler bei I/O- oder fsync-Fehlern.
    ///
    /// # Crash
    /// Erzwingt Slot-Dauerhaftigkeit via sync_data; bei fsync-Fehler tritt Poisoning ein.
    fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>>;
}
