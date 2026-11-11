//! Port `KeyStore` für die Schlüsselhierarchie und Schlüsselvernichtung (WP-K).
//!
//! Entkoppelte Schnittstelle zum Erzeugen, Wrappen, Entschlüsseln und physischen
//! Vernichten von Schlüsseln (KEK/DEK) mit Zeroize und Fail-Closed-Semantik.
//!
//! Herkunft: ffeb79555:crates/contextra-crypto/src/scope_keyring.rs

#![forbid(unsafe_code)]

#[cfg(feature = "vault")]
use crate::kv_shredding::{
    EncryptedEpochRecordPayload, GroupEpochKey, KeyPurgeReceipt, KeyRegistry,
};
use crate::types_local::Clock;
#[cfg(not(feature = "vault"))]
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

/// Schlüsselbereich für Gruppen und Epochen (Entscheidung E5), falls Feature vault nicht aktiv ist.
#[cfg(not(feature = "vault"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GroupEpochKey {
    /// Gruppen- bzw. Mandanten-ID.
    pub group_id: u64,
    /// Epochen-Nummer für Rotation und isolierte Schlüsselvernichtung.
    pub epoch: u64,
}

#[cfg(not(feature = "vault"))]
impl GroupEpochKey {
    /// Erzeugt einen neuen `GroupEpochKey`.
    pub const fn new(group_id: u64, epoch: u64) -> Self {
        Self { group_id, epoch }
    }
}

/// Vernichtungsergebnis nach `purge_keys` zur Verankerung im Ledger durch den Orchestrator.
/// Herkunft: ffeb79555:crates/contextra-crypto/src/scope_keyring.rs:220 (DestructionReceipt)
#[cfg(not(feature = "vault"))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyPurgeReceipt {
    /// Bereinigte Gruppen-ID.
    pub group_id: u64,
    /// Bereinigte Epoche (Entscheidung E5).
    pub epoch: u64,
    /// Anzahl physisch genullter DEK-Schlüssel-Handles dieser Epoche.
    pub affected_deks: usize,
    /// Zeitstempel der Vernichtung (Nanosekunden seit UNIX_EPOCH).
    pub purged_at_nanos: u64,
}

/// Stark typisierter Daten-Schlüssel (DEK) mit automatischer Nullung im Speicher.
/// Herkunft: 1364a54d5:crates/contextra-crypto/src/hierarchy.rs:16 (MasterKey)
#[derive(ZeroizeOnDrop)]
pub struct LocalDek(pub [u8; 32]);

impl LocalDek {
    /// Erzeugt einen neuen `LocalDek` aus 32 Byte Schlüsselmaterial.
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Zugriff auf die rohen Schlüssel-Bytes als Referenz.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for LocalDek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalDek")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Lokale Fehlertaxonomie für den synchronen `KeyStore`-Port.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum KeyStoreError {
    /// Gruppe oder Epoche wurde nicht gefunden.
    #[error("Schlüsselbereich nicht gefunden: Gruppe {group_id}, Epoche {epoch}")]
    NotFound {
        /// Betroffene Gruppen-ID.
        group_id: u64,
        /// Betroffene Epoche.
        epoch: u64,
    },
    /// Schlüssel wurde bereits vernichtet (Fail-Closed).
    #[error("Schlüssel für Gruppe {group_id}, Epoche {epoch} wurde bereits vernichtet")]
    KeyDestroyed {
        /// Betroffene Gruppen-ID.
        group_id: u64,
        /// Betroffene Epoche.
        epoch: u64,
    },
    /// Kryptographischer Fehler im Verschlüsselungs- oder Wrap-Vorgang.
    #[error("Kryptographischer Fehler: {0}")]
    Crypto(String),
    /// Ungültige Parameter oder fehlerhafte Nonce/Länge.
    #[error("Ungültige Eingabe: {0}")]
    InvalidInput(String),
}

/// Synchroner Port-Trait für Schlüsselverwaltung und Vernichtung (WP-K).
///
/// Folgt der synchronen Kernregel: Keine Async-Runtime im Löschpfad.
/// Zeit und Zufall werden ausschließlich über Ports (`Clock`) injiziert.
pub trait KeyStore: Send + Sync {
    /// Erzeugt oder lädt den Gruppen-KEK für einen Bereich `(group_id, epoch)`.
    fn get_or_derive_kek(&self, scope: GroupEpochKey) -> Result<[u8; 32], KeyStoreError>;

    #[cfg(feature = "vault")]
    /// Verschlüsselt und wrappt einen Datensatz unter Envelope Encryption.
    fn encrypt_record(
        &self,
        scope: GroupEpochKey,
        record_id: u64,
        plaintext: &[u8],
    ) -> Result<EncryptedEpochRecordPayload, KeyStoreError>;

    #[cfg(feature = "vault")]
    /// Entschlüsselt einen Datensatz unter Envelope Encryption.
    fn decrypt_record(
        &self,
        payload: &EncryptedEpochRecordPayload,
    ) -> Result<Vec<u8>, KeyStoreError>;

    /// Prüft ob ein Schlüsselbereich aktiv (nicht widerrufen) ist.
    fn is_active(&self, scope: GroupEpochKey) -> bool;

    /// Vernichtet alle Schlüssel eines Bereichs unwiderruflich (`purge_keys`).
    ///
    /// Nullt das Schlüsselmaterial im RAM mit Zeroize, markiert die Epoche
    /// als vernichtet und liefert ein Quittungsobjekt zur Ledger-Verankerung.
    /// Idempotent: Mehrfachaufrufe führen nicht zu Fehlern oder Reaktivierung (INV-NO-RESURRECTION-1).
    fn purge_keys(
        &self,
        scope: GroupEpochKey,
        clock: Option<&dyn Clock>,
    ) -> Result<KeyPurgeReceipt, KeyStoreError>;
}

#[cfg(feature = "vault")]
/// Adapter-Implementierung von `KeyStore` für `KeyRegistry`.
impl KeyStore for KeyRegistry {
    fn get_or_derive_kek(&self, scope: GroupEpochKey) -> Result<[u8; 32], KeyStoreError> {
        let subkey = self
            .get_or_derive_epoch(scope.group_id, scope.epoch)
            .map_err(|e| match e {
                crate::error::ErasureError::Refused => KeyStoreError::KeyDestroyed {
                    group_id: scope.group_id,
                    epoch: scope.epoch,
                },
                other => KeyStoreError::Crypto(other.to_string()),
            })?;
        Ok(subkey.0)
    }

    fn encrypt_record(
        &self,
        scope: GroupEpochKey,
        record_id: u64,
        plaintext: &[u8],
    ) -> Result<EncryptedEpochRecordPayload, KeyStoreError> {
        self.encrypt_envelope_record_epoch(scope.group_id, scope.epoch, record_id, plaintext)
            .map_err(|e| match e {
                crate::error::ErasureError::Refused => KeyStoreError::KeyDestroyed {
                    group_id: scope.group_id,
                    epoch: scope.epoch,
                },
                other => KeyStoreError::Crypto(other.to_string()),
            })
    }

    fn decrypt_record(
        &self,
        payload: &EncryptedEpochRecordPayload,
    ) -> Result<Vec<u8>, KeyStoreError> {
        self.decrypt_envelope_record_epoch(payload)
            .map_err(|e| match e {
                crate::error::ErasureError::Refused => KeyStoreError::KeyDestroyed {
                    group_id: payload.group_id,
                    epoch: payload.epoch,
                },
                other => KeyStoreError::Crypto(other.to_string()),
            })
    }

    fn is_active(&self, scope: GroupEpochKey) -> bool {
        self.is_group_epoch_active(scope.group_id, scope.epoch)
    }

    fn purge_keys(
        &self,
        scope: GroupEpochKey,
        clock: Option<&dyn Clock>,
    ) -> Result<KeyPurgeReceipt, KeyStoreError> {
        self.purge_keys(scope.group_id, scope.epoch, clock)
            .map_err(|e| match e {
                crate::kv_shredding::KeyShredError::GroupNotFound { group_id } => {
                    KeyStoreError::NotFound {
                        group_id,
                        epoch: scope.epoch,
                    }
                }
                crate::kv_shredding::KeyShredError::EpochNotFound { group_id, epoch } => {
                    KeyStoreError::NotFound { group_id, epoch }
                }
                crate::kv_shredding::KeyShredError::KekAlreadyDestroyed { group_id, epoch } => {
                    KeyStoreError::KeyDestroyed { group_id, epoch }
                }
                crate::kv_shredding::KeyShredError::Crypto(s) => KeyStoreError::Crypto(s),
                crate::kv_shredding::KeyShredError::InvalidInput(s) => {
                    KeyStoreError::InvalidInput(s)
                }
            })
    }
}
