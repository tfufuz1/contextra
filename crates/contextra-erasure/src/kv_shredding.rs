//! # Kryptographisches Shredding für KV-Daten (`kv_shredding.rs`)
//!
//! Ermöglicht kryptografische Bereinigung einzelner Datensätze oder ganzer Mandantengruppen
//! durch gezielte Zerstörung des zugehörigen kryptographischen Schlüssels (Crypto-Erase).
//!
//! ## Invarianten
//! - Nach Zerstörung des Schlüssels ist der Chiffretext unentschlüsselbar.
//! - Gruppe und Einzelschlüssel-Hierarchie erlauben feingranulares Shredding.
//! - Grenzen: Physische Speicherabbilder, RAM-Reste vor Zeroization, externe Backups
//!   sind außerhalb des kryptographischen Einflussbereichs dieses Bausteins.

// FILE-CONTEXT
// ZWECK: Envelope Encryption und Key-Registry für KV-Crypto-Shredding (AP-P0-06, Spec v17 §16.1).
// INVARIANTEN: CryptoShred vernichtet KEK (Gruppenbereinigung) oder DEK-Wrap (Einzelbereinigung) in O(1).
// Zero-Panic, deterministische Fehlerpfade, ZeroizeOnDrop für alle Schlüsseltypen.

#![forbid(unsafe_code)]

use crate::error::ErasureError;
#[cfg(feature = "signing")]
use crate::revocation_log::{RevocationLog, RevocationTarget};
use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "signing")]
use std::sync::Arc;
use std::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Standardgröße einer Shredding-Schlüsselgruppe (Anzahl Records je KEK).
pub const DEFAULT_SHRED_KEY_GROUP_SIZE: usize = 64;

/// Sub-Key bzw. KEK-Bytes (256 Bit) mit automatischer Nullung beim Verwerfen (`ZeroizeOnDrop`).
#[derive(ZeroizeOnDrop)]
pub struct SubKey(pub [u8; 32]);

impl Clone for SubKey {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl std::fmt::Debug for SubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubKey")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Group Key Encryption Key (KEK) - 256 Bit, mit automatischer Nullung beim Verwerfen (`ZeroizeOnDrop`).
#[derive(ZeroizeOnDrop)]
pub struct GroupKek(pub [u8; 32]);

impl Clone for GroupKek {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl std::fmt::Debug for GroupKek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupKek")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Record Data Encryption Key (DEK) - 256 Bit, mit automatischer Nullung beim Verwerfen (`ZeroizeOnDrop`).
#[derive(ZeroizeOnDrop)]
pub struct RecordDek(pub [u8; 32]);

impl Clone for RecordDek {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl std::fmt::Debug for RecordDek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordDek")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Serialisierbarer Container für einen mit Envelope Encryption geschützten Datensatz.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct EncryptedRecordPayload {
    /// Numerische ID des Datensatzes.
    pub record_id: u64,
    /// Shredding-Gruppen-ID, zu der der Datensatz gehört.
    pub group_id: u64,
    /// Mit AES-256-GCM-SIV verschlüsselte Nutzdaten.
    pub ciphertext: Vec<u8>,
    /// 12-Byte Nonce für die Nutzdatenentschlüsselung.
    pub nonce: [u8; 12],
    /// Mit dem Gruppen-KEK gewrappter Datensatzschlüssel (DEK).
    pub wrapped_dek: Vec<u8>,
    /// 12-Byte Nonce für das DEK-Unwrapping.
    pub dek_nonce: [u8; 12],
}

#[derive(Debug, Clone)]
struct RecordDekEntry {
    wrapped_dek: Vec<u8>,
    dek_nonce: [u8; 12],
    revoked: bool,
}

#[derive(Debug)]
struct GroupEntry {
    kek: GroupKek,
    wrapped_kek: Vec<u8>,
    kek_nonce: [u8; 12],
    record_deks: BTreeMap<u64, RecordDekEntry>,
}

/// Leitet einen Sub-Key aus einem Masterschlüssel deterministisch via BLAKE3 KDF ab.
pub fn derive_subkey(master_key: &[u8; 32], group_id: u64, epoch: u64) -> SubKey {
    let mut hasher = blake3::Hasher::new_derive_key("contextra-kv-shredding-subkey-v1");
    hasher.update(master_key);
    hasher.update(&group_id.to_le_bytes());
    hasher.update(&epoch.to_le_bytes());
    let mut subkey_bytes = [0u8; 32];
    hasher.finalize_xof().fill(&mut subkey_bytes);
    SubKey(subkey_bytes)
}

/// Thread-sichere Schlüssel-Registry für Envelope Encryption und O(1) Krypto-Shredding.
/// Schlüsselbereich für Gruppen und Epochen (Entscheidung E5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GroupEpochKey {
    /// Gruppen- bzw. Mandanten-ID.
    pub group_id: u64,
    /// Epochen-Nummer für Rotation und isolierte Schlüsselvernichtung.
    pub epoch: u64,
}

impl GroupEpochKey {
    /// Erzeugt einen neuen `GroupEpochKey`.
    pub const fn new(group_id: u64, epoch: u64) -> Self {
        Self { group_id, epoch }
    }
}

/// Vernichtungsergebnis nach `purge_keys` zur Verankerung im Ledger durch den Orchestrator.
/// Herkunft: ffeb79555:crates/contextra-crypto/src/scope_keyring.rs:220 (DestructionReceipt)
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

/// Lokale Fehlertaxonomie für Schlüsseloperationen und Krypto-Shredding.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum KeyShredError {
    /// Gruppe nicht gefunden.
    #[error("Gruppe nicht gefunden: {group_id}")]
    GroupNotFound {
        /// Betroffene Gruppen-ID.
        group_id: u64,
    },
    /// Epoche nicht gefunden.
    #[error("Epoche nicht gefunden für Gruppe {group_id}, Epoche {epoch}")]
    EpochNotFound {
        /// Betroffene Gruppen-ID.
        group_id: u64,
        /// Betroffene Epoche.
        epoch: u64,
    },
    /// KEK wurde bereits vernichtet (Fail-Closed).
    #[error("KEK für Gruppe {group_id}, Epoche {epoch} ist bereits vernichtet")]
    KekAlreadyDestroyed {
        /// Betroffene Gruppen-ID.
        group_id: u64,
        /// Betroffene Epoche.
        epoch: u64,
    },
    /// Kryptographischer Fehler im Verschlüsselungs- oder Entschlüsselungspfad.
    #[error("Kryptographischer Fehler: {0}")]
    Crypto(String),
    /// Ungültige Eingabeparameter.
    #[error("Ungültige Eingabe: {0}")]
    InvalidInput(String),
}

/// Serialisierbarer Container für einen mit Envelope Encryption und Epochenbindung geschützten Datensatz.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct EncryptedEpochRecordPayload {
    /// Numerische ID des Datensatzes.
    pub record_id: u64,
    /// Shredding-Gruppen-ID, zu der der Datensatz gehört.
    pub group_id: u64,
    /// Epoche, zu der der KEK gehört.
    pub epoch: u64,
    /// Mit AES-256-GCM-SIV verschlüsselte Nutzdaten.
    pub ciphertext: Vec<u8>,
    /// 12-Byte Nonce für die Nutzdatenentschlüsselung.
    pub nonce: [u8; 12],
    /// Mit dem Gruppen-Epochen-KEK gewrappter Datensatzschlüssel (DEK).
    pub wrapped_dek: Vec<u8>,
    /// 12-Byte Nonce für das DEK-Unwrapping.
    pub dek_nonce: [u8; 12],
}

/// Thread-sichere Schlüssel-Registry für Envelope Encryption und O(1) Krypto-Shredding.
pub struct KeyRegistry {
    groups: RwLock<BTreeMap<u64, GroupEntry>>,
    epoch_groups: RwLock<BTreeMap<(u64, u64), GroupEntry>>,
    group_keks: RwLock<BTreeMap<u64, Vec<u8>>>,
    record_deks: RwLock<BTreeMap<String, Vec<u8>>>,
    revoked_groups: RwLock<BTreeSet<u64>>,
    revoked_epoch_groups: RwLock<BTreeSet<(u64, u64)>>,
    #[allow(dead_code)]
    revoked_subkeys: RwLock<BTreeSet<String>>,
    revoked_records: RwLock<BTreeSet<String>>,
    #[cfg(feature = "signing")]
    revocation_log: Option<Arc<RevocationLog>>,
    master_kek: [u8; 32],
}

impl Default for KeyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyRegistry {
    /// Erzeugt eine neue `KeyRegistry` mit zufälligem Master-KEK.
    pub fn new() -> Self {
        let mut master_kek = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut master_kek);
        Self {
            groups: RwLock::new(BTreeMap::new()),
            epoch_groups: RwLock::new(BTreeMap::new()),
            group_keks: RwLock::new(BTreeMap::new()),
            record_deks: RwLock::new(BTreeMap::new()),
            revoked_groups: RwLock::new(BTreeSet::new()),
            revoked_epoch_groups: RwLock::new(BTreeSet::new()),
            revoked_subkeys: RwLock::new(BTreeSet::new()),
            revoked_records: RwLock::new(BTreeSet::new()),
            #[cfg(feature = "signing")]
            revocation_log: None,
            master_kek,
        }
    }

    /// Bindet ein `RevocationLog` an und synchronisiert bereits protokollierte Widerrufe.
    #[cfg(feature = "signing")]
    pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self {
        if let Ok(mut revoked_guard) = self.revoked_groups.write() {
            if let Ok(mut groups_guard) = self.groups.write() {
                for target in log.revoked_targets() {
                    if let RevocationTarget::Group(group_id) = target {
                        revoked_guard.insert(group_id);
                        if let Some(mut entry) = groups_guard.remove(&group_id) {
                            entry.kek.0.zeroize();
                            entry.wrapped_kek.zeroize();
                            for (_, rec) in entry.record_deks.iter_mut() {
                                rec.wrapped_dek.zeroize();
                            }
                        }
                    }
                }
            }
        }
        self.revocation_log = Some(log);
        self
    }

    /// Liefert den gewrappten KEK und die Nonce einer Gruppe, falls vorhanden und aktiv.
    pub fn get_wrapped_kek(&self, group_id: u64) -> Option<(Vec<u8>, [u8; 12])> {
        if self.is_group_revoked(group_id) {
            return None;
        }
        let read_guard = match self.groups.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        let entry = read_guard.get(&group_id)?;
        Some((entry.wrapped_kek.clone(), entry.kek_nonce))
    }

    /// Liefert den gewrappten DEK und die Nonce eines Records, falls vorhanden und aktiv.
    pub fn get_wrapped_dek(&self, group_id: u64, record_id: u64) -> Option<(Vec<u8>, [u8; 12])> {
        if self.is_group_revoked(group_id) {
            return None;
        }
        let read_guard = match self.groups.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        let entry = read_guard.get(&group_id)?;
        let rec = entry.record_deks.get(&record_id)?;
        if rec.revoked {
            return None;
        }
        Some((rec.wrapped_dek.clone(), rec.dek_nonce))
    }

    /// Prüft ob eine Gruppe aktiv ist.
    pub fn is_group_active(&self, group_id: u64) -> bool {
        !self.is_group_revoked(group_id)
    }

    /// Alias für `is_group_active`.
    pub fn is_key_active(&self, group_id: u64) -> bool {
        self.is_group_active(group_id)
    }

    /// Prüft ob eine Gruppe widerrufen ist.
    pub fn is_group_revoked(&self, group_id: u64) -> bool {
        #[cfg(feature = "signing")]
        if let Some(ref log) = self.revocation_log {
            if log.is_revoked(&RevocationTarget::Group(group_id)) {
                return true;
            }
        }
        let lock = match self.revoked_groups.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        lock.contains(&group_id)
    }

    /// Ermittelt oder erzeugt den Group KEK für `group_id`.
    pub fn get_or_derive(&self, group_id: u64) -> Result<SubKey, ErasureError> {
        if self.is_group_revoked(group_id) {
            return Err(ErasureError::Refused);
        }

        {
            let read_guard = match self.groups.read() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(entry) = read_guard.get(&group_id) {
                return Ok(SubKey(entry.kek.0));
            }
        }

        let mut write_guard = match self.groups.write() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };

        if self.is_group_revoked(group_id) {
            return Err(ErasureError::Refused);
        }

        if let Some(entry) = write_guard.get(&group_id) {
            return Ok(SubKey(entry.kek.0));
        }

        let mut kek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut kek_bytes);

        let master_cipher = Aes256GcmSiv::new_from_slice(&self.master_kek)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        let mut kek_nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut kek_nonce);
        let wrapped_kek = master_cipher
            .encrypt(Nonce::from_slice(&kek_nonce), kek_bytes.as_slice())
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        let entry = GroupEntry {
            kek: GroupKek(kek_bytes),
            wrapped_kek,
            kek_nonce,
            record_deks: BTreeMap::new(),
        };

        write_guard.insert(group_id, entry);
        Ok(SubKey(kek_bytes))
    }

    /// Widerruft eine gesamte Shredding-Gruppe in O(1) und nullt das Schlüsselmaterial im RAM.
    pub fn revoke_group(&self, group_id: u64) -> Result<(), ErasureError> {
        if self.is_group_revoked(group_id) {
            return Ok(());
        }

        #[cfg(feature = "signing")]
        if let Some(ref log) = self.revocation_log {
            log.append(RevocationTarget::Group(group_id))?;
        }

        {
            let mut lock = match self.revoked_groups.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            lock.insert(group_id);
        }

        {
            let mut groups = match self.groups.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(mut entry) = groups.remove(&group_id) {
                entry.kek.0.zeroize();
                entry.wrapped_kek.zeroize();
                for (_, rec) in entry.record_deks.iter_mut() {
                    rec.wrapped_dek.zeroize();
                }
            }
        }

        {
            let mut keks = match self.group_keks.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(mut wrapped) = keks.remove(&group_id) {
                wrapped.zeroize();
            }
        }

        Ok(())
    }

    /// Alias für `revoke_group`.
    pub fn revoke_subkey(&self, group_id: u64) -> Result<(), ErasureError> {
        self.revoke_group(group_id)
    }

    /// Widerruft einen einzelnen Datensatz anhand seiner numerischen ID.
    pub fn revoke_record_num(&self, group_id: u64, record_id: u64) -> bool {
        if self.is_group_revoked(group_id) {
            return false;
        }

        let mut groups = match self.groups.write() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };

        if let Some(entry) = groups.get_mut(&group_id) {
            if let Some(rec) = entry.record_deks.get_mut(&record_id) {
                if !rec.revoked {
                    rec.revoked = true;
                    rec.wrapped_dek.zeroize();
                    return true;
                }
            }
        }
        false
    }

    /// Widerruft einen einzelnen Datensatz anhand seines String-Schlüssels.
    pub fn revoke_record(&self, record_id: &str) -> Result<(), ErasureError> {
        {
            let mut lock = match self.revoked_records.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            lock.insert(record_id.to_string());
        }
        {
            let mut deks = match self.record_deks.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(mut wrapped) = deks.remove(record_id) {
                wrapped.zeroize();
            }
        }
        #[cfg(feature = "signing")]
        if let Some(ref log) = self.revocation_log {
            log.append(RevocationTarget::Record(record_id.to_string()))?;
        }
        Ok(())
    }

    /// Prüft ob ein numerischer Datensatz aktiv ist.
    pub fn is_record_num_active(&self, group_id: u64, record_id: u64) -> bool {
        if self.is_group_revoked(group_id) {
            return false;
        }
        let groups = match self.groups.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        if let Some(entry) = groups.get(&group_id) {
            if let Some(rec) = entry.record_deks.get(&record_id) {
                return !rec.revoked;
            }
        }
        false
    }

    /// Prüft ob ein Datensatz anhand seines String-Schlüssels aktiv ist.
    pub fn is_record_active(&self, record_id: &str) -> bool {
        let lock = match self.revoked_records.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        #[cfg(feature = "signing")]
        let log_revoked = self
            .revocation_log
            .as_ref()
            .map(|log| log.is_revoked(&RevocationTarget::Record(record_id.to_string())))
            .unwrap_or(false);
        #[cfg(not(feature = "signing"))]
        let log_revoked = false;

        !lock.contains(record_id) && !log_revoked
    }

    /// Verschlüsselt einen Datensatz mittels Envelope Encryption.
    pub fn encrypt_envelope_record(
        &self,
        group_id: u64,
        record_id: u64,
        plaintext: &[u8],
    ) -> Result<EncryptedRecordPayload, ErasureError> {
        let subkey = self.get_or_derive(group_id)?;
        let kek_cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| ErasureError::Crypto(format!("KEK init: {e}")))?;

        let mut dek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut dek_bytes);

        let mut dek_nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut dek_nonce);
        let wrapped_dek = kek_cipher
            .encrypt(Nonce::from_slice(&dek_nonce), dek_bytes.as_slice())
            .map_err(|e| ErasureError::Crypto(format!("DEK wrap: {e}")))?;

        let dek_cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| ErasureError::Crypto(format!("DEK cipher init: {e}")))?;
        let mut nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ciphertext = dek_cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| ErasureError::Crypto(format!("Payload encrypt: {e}")))?;

        let mut groups = match self.groups.write() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        if let Some(entry) = groups.get_mut(&group_id) {
            entry.record_deks.insert(
                record_id,
                RecordDekEntry {
                    wrapped_dek: wrapped_dek.clone(),
                    dek_nonce,
                    revoked: false,
                },
            );
        }

        dek_bytes.zeroize();

        Ok(EncryptedRecordPayload {
            record_id,
            group_id,
            ciphertext,
            nonce,
            wrapped_dek,
            dek_nonce,
        })
    }

    /// Entschlüsselt einen Datensatz mittels Envelope Encryption.
    pub fn decrypt_envelope_record(
        &self,
        payload: &EncryptedRecordPayload,
    ) -> Result<Vec<u8>, ErasureError> {
        if self.is_group_revoked(payload.group_id) {
            return Err(ErasureError::Refused);
        }

        let (wrapped_dek, dek_nonce) = {
            let groups = match self.groups.read() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            let entry = groups.get(&payload.group_id).ok_or(ErasureError::Refused)?;
            let rec = entry
                .record_deks
                .get(&payload.record_id)
                .ok_or(ErasureError::Refused)?;
            if rec.revoked || rec.wrapped_dek != payload.wrapped_dek {
                return Err(ErasureError::Refused);
            }
            (rec.wrapped_dek.clone(), rec.dek_nonce)
        };

        let subkey = self.get_or_derive(payload.group_id)?;
        let kek_cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| ErasureError::Crypto(format!("KEK init: {e}")))?;

        let dek_bytes = kek_cipher
            .decrypt(Nonce::from_slice(&dek_nonce), wrapped_dek.as_slice())
            .map_err(|e| ErasureError::Crypto(format!("DEK unwrap: {e}")))?;

        let dek_cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| ErasureError::Crypto(format!("DEK init: {e}")))?;

        dek_cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                payload.ciphertext.as_slice(),
            )
            .map_err(|e| ErasureError::Crypto(format!("Payload decrypt: {e}")))
    }

    /// Verschlüsselt direkt unter dem Gruppen-KEK für `group_id`.
    pub fn encrypt_with_group(
        &self,
        group_id: u64,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, [u8; 12]), ErasureError> {
        if self.is_group_revoked(group_id) {
            return Err(ErasureError::Refused);
        }
        let subkey = self.get_or_derive(group_id)?;
        let cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| ErasureError::Crypto(format!("Aes256GcmSiv init: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| ErasureError::Crypto(format!("Group encrypt: {e}")))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Entschlüsselt Chiffretext direkt unter dem Gruppen-KEK für `group_id`.
    pub fn decrypt_with_group(
        &self,
        group_id: u64,
        ciphertext: &[u8],
        nonce_bytes: &[u8; 12],
    ) -> Result<Vec<u8>, ErasureError> {
        if self.is_group_revoked(group_id) {
            return Err(ErasureError::Refused);
        }

        let groups = match self.groups.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };

        let entry = groups.get(&group_id).ok_or(ErasureError::Refused)?;
        let cipher = Aes256GcmSiv::new_from_slice(&entry.kek.0)
            .map_err(|e| ErasureError::Crypto(format!("Aes256GcmSiv init: {e}")))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ErasureError::Crypto(format!("Group decrypt: {e}")))
    }

    /// Verschlüsselt Daten für einen Record mit AES-256-GCM-SIV.
    pub fn encrypt_record(
        &self,
        group_id: u64,
        record_id: &str,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ErasureError> {
        if self.is_group_revoked(group_id) || !self.is_record_active(record_id) {
            return Err(ErasureError::Refused);
        }

        let mut dek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut dek_bytes);
        let cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        dek_bytes.zeroize();

        let mut payload = Vec::with_capacity(12 + ciphertext.len());
        payload.extend_from_slice(&nonce_bytes);
        payload.extend_from_slice(&ciphertext);
        Ok(payload)
    }

    /// Entschlüsselt Daten für einen Record mit AES-256-GCM-SIV.
    pub fn decrypt_record(
        &self,
        group_id: u64,
        record_id: &str,
        dek_bytes: &[u8; 32],
        payload: &[u8],
    ) -> Result<Vec<u8>, ErasureError> {
        if self.is_group_revoked(group_id) || !self.is_record_active(record_id) {
            return Err(ErasureError::Refused);
        }
        if payload.len() < 12 {
            return Err(ErasureError::Corrupt);
        }

        let nonce_slice = payload.get(..12).ok_or(ErasureError::Corrupt)?;
        let nonce = Nonce::from_slice(nonce_slice);
        let ciphertext = payload.get(12..).ok_or(ErasureError::Corrupt)?;

        let cipher = Aes256GcmSiv::new_from_slice(dek_bytes)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ErasureError::Crypto(e.to_string()))
    }

    /// Prüft ob eine Gruppe in einer bestimmten Epoche widerrufen ist.
    pub fn is_group_epoch_revoked(&self, group_id: u64, epoch: u64) -> bool {
        if self.is_group_revoked(group_id) {
            return true;
        }
        let lock = match self.revoked_epoch_groups.read() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        lock.contains(&(group_id, epoch))
    }

    /// Prüft ob eine Gruppe in einer bestimmten Epoche aktiv ist.
    pub fn is_group_epoch_active(&self, group_id: u64, epoch: u64) -> bool {
        !self.is_group_epoch_revoked(group_id, epoch)
    }

    /// Ermittelt oder erzeugt den Group-KEK für ein spezifisches `(group_id, epoch)`-Paar.
    pub fn get_or_derive_epoch(&self, group_id: u64, epoch: u64) -> Result<SubKey, ErasureError> {
        if self.is_group_epoch_revoked(group_id, epoch) {
            return Err(ErasureError::Refused);
        }

        let key = (group_id, epoch);
        {
            let read_guard = match self.epoch_groups.read() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(entry) = read_guard.get(&key) {
                return Ok(SubKey(entry.kek.0));
            }
        }

        let mut write_guard = match self.epoch_groups.write() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };

        if self.is_group_epoch_revoked(group_id, epoch) {
            return Err(ErasureError::Refused);
        }

        if let Some(entry) = write_guard.get(&key) {
            return Ok(SubKey(entry.kek.0));
        }

        let mut kek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut kek_bytes);

        let master_cipher = Aes256GcmSiv::new_from_slice(&self.master_kek)
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        let mut kek_nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut kek_nonce);
        let wrapped_kek = master_cipher
            .encrypt(Nonce::from_slice(&kek_nonce), kek_bytes.as_slice())
            .map_err(|e| ErasureError::Crypto(e.to_string()))?;

        let entry = GroupEntry {
            kek: GroupKek(kek_bytes),
            wrapped_kek,
            kek_nonce,
            record_deks: BTreeMap::new(),
        };

        write_guard.insert(key, entry);
        Ok(SubKey(kek_bytes))
    }

    /// Verschlüsselt einen Datensatz unter Envelope Encryption gebunden an Gruppe und Epoche.
    pub fn encrypt_envelope_record_epoch(
        &self,
        group_id: u64,
        epoch: u64,
        record_id: u64,
        plaintext: &[u8],
    ) -> Result<EncryptedEpochRecordPayload, ErasureError> {
        if self.is_group_epoch_revoked(group_id, epoch) {
            return Err(ErasureError::Refused);
        }

        let subkey = self.get_or_derive_epoch(group_id, epoch)?;
        let kek_cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| ErasureError::Crypto(format!("KEK init: {e}")))?;

        let mut dek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut dek_bytes);

        let mut dek_nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut dek_nonce);
        let wrapped_dek = kek_cipher
            .encrypt(Nonce::from_slice(&dek_nonce), dek_bytes.as_slice())
            .map_err(|e| ErasureError::Crypto(format!("DEK wrap: {e}")))?;

        let dek_cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| ErasureError::Crypto(format!("DEK cipher init: {e}")))?;
        let mut nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ciphertext = dek_cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| ErasureError::Crypto(format!("Payload encrypt: {e}")))?;

        let mut groups = match self.epoch_groups.write() {
            Ok(guard) => guard,
            Err(poison) => poison.into_inner(),
        };
        if let Some(entry) = groups.get_mut(&(group_id, epoch)) {
            entry.record_deks.insert(
                record_id,
                RecordDekEntry {
                    wrapped_dek: wrapped_dek.clone(),
                    dek_nonce,
                    revoked: false,
                },
            );
        }

        dek_bytes.zeroize();

        Ok(EncryptedEpochRecordPayload {
            record_id,
            group_id,
            epoch,
            ciphertext,
            nonce,
            wrapped_dek,
            dek_nonce,
        })
    }

    /// Entschlüsselt einen Datensatz unter Envelope Encryption mit Epochenprüfung.
    pub fn decrypt_envelope_record_epoch(
        &self,
        payload: &EncryptedEpochRecordPayload,
    ) -> Result<Vec<u8>, ErasureError> {
        if self.is_group_epoch_revoked(payload.group_id, payload.epoch) {
            return Err(ErasureError::Refused);
        }

        let (wrapped_dek, dek_nonce) = {
            let groups = match self.epoch_groups.read() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            let entry = groups
                .get(&(payload.group_id, payload.epoch))
                .ok_or(ErasureError::Refused)?;
            let rec = entry
                .record_deks
                .get(&payload.record_id)
                .ok_or(ErasureError::Refused)?;
            if rec.revoked || rec.wrapped_dek != payload.wrapped_dek {
                return Err(ErasureError::Refused);
            }
            (rec.wrapped_dek.clone(), rec.dek_nonce)
        };

        let subkey = self.get_or_derive_epoch(payload.group_id, payload.epoch)?;
        let kek_cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| ErasureError::Crypto(format!("KEK init: {e}")))?;

        let dek_bytes = kek_cipher
            .decrypt(Nonce::from_slice(&dek_nonce), wrapped_dek.as_slice())
            .map_err(|e| ErasureError::Crypto(format!("DEK unwrap: {e}")))?;

        let dek_cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| ErasureError::Crypto(format!("DEK init: {e}")))?;

        dek_cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                payload.ciphertext.as_slice(),
            )
            .map_err(|e| ErasureError::Crypto(format!("Payload decrypt: {e}")))
    }

    /// Physische und kryptographische Schlüsselvernichtung (`purge_keys`) für eine Gruppe und Epoche.
    ///
    /// Herkunft: ffeb79555:crates/contextra-crypto/src/scope_keyring.rs:600 (ScopeKeyring::destroy)
    ///
    /// Vernichtet den KEK, nullt alle Rohschlüssel und gewrappten DEKs via Zeroize,
    /// markiert die Epoche unwiderruflich als widerrufen (INV-NO-RESURRECTION-1)
    /// und liefert ein `KeyPurgeReceipt` zur späteren Ledger-Verankerung zurück.
    /// Bei wiederholtem Aufruf verhält sich die Funktion idempotent und liefert ein Quittungsobjekt
    /// mit 0 betroffenen DEKs.
    pub fn purge_keys(
        &self,
        group_id: u64,
        epoch: u64,
        clock: Option<&dyn crate::types_local::Clock>,
    ) -> Result<KeyPurgeReceipt, KeyShredError> {
        let now_nanos = clock.map(|c| c.now_unix_nanos()).unwrap_or(0);

        let key = (group_id, epoch);

        // Idempotenz-Prüfung nach INV-NO-RESURRECTION-1: Bereits widerrufen?
        let is_already_revoked = {
            let lock = match self.revoked_epoch_groups.read() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            lock.contains(&key)
        };

        if is_already_revoked {
            return Ok(KeyPurgeReceipt {
                group_id,
                epoch,
                affected_deks: 0,
                purged_at_nanos: now_nanos,
            });
        }

        // Epoche in revoked_epoch_groups eintragen (Fail-Closed Vormerkung)
        {
            let mut lock = match self.revoked_epoch_groups.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            lock.insert(key);
        }

        // KEK und alle DEKs physisch aus dem Speicher entfernen und nullen
        let affected_deks = {
            let mut groups = match self.epoch_groups.write() {
                Ok(guard) => guard,
                Err(poison) => poison.into_inner(),
            };
            if let Some(mut entry) = groups.remove(&key) {
                entry.kek.0.zeroize();
                entry.wrapped_kek.zeroize();
                let count = entry.record_deks.len();
                for (_, rec) in entry.record_deks.iter_mut() {
                    rec.wrapped_dek.zeroize();
                }
                count
            } else {
                0
            }
        };

        Ok(KeyPurgeReceipt {
            group_id,
            epoch,
            affected_deks,
            purged_at_nanos: now_nanos,
        })
    }
}
