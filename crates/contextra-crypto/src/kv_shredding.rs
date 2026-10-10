// FILE-CONTEXT
// ZWECK: Envelope Encryption und Key-Registry für KV-Crypto-Shredding (AP-P0-06, Spec v17 §16.1).
// INVARIANTEN: CryptoShred vernichtet KEK (Gruppenlöschung) oder DEK-Wrap (Einzellöschung) in O(1).
// Fail-Closed bei Revokationsprüfungen und verifiziertem RevocationLog-Status.
// Zufälliges KEK/DEK-Material via OsRng (keine deterministische Ableitung rein aus group_id).
// HOTSPOTS: [SubKey, KeyRegistry, revoke_group, revoke_record, encrypt_record, decrypt_record]

//! Envelope encryption and thread-safe key registry for KV crypto-shredding.

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use crate::revocation_log::{RevocationLog, RevocationTarget};
use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Default size of a shred key group (number of records sharing one KEK).
pub const DEFAULT_SHRED_KEY_GROUP_SIZE: usize = 64;

/// Sub-key or KEK bytes (256 bits) zeroized on drop.
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

/// Group Key Encryption Key (KEK) - 256 bits, zeroized on drop.
#[derive(ZeroizeOnDrop)]
pub struct GroupKek(pub [u8; 32]);

impl std::fmt::Debug for GroupKek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupKek")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Record Data Encryption Key (DEK) - 256 bits, zeroized on drop.
#[derive(ZeroizeOnDrop)]
pub struct RecordDek(pub [u8; 32]);

impl std::fmt::Debug for RecordDek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordDek")
            .field("bytes", &"***REDACTED***")
            .finish()
    }
}

/// Container for an encrypted record payload under Envelope Encryption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct EncryptedRecordPayload {
    /// Unique identifier for the record.
    pub record_id: u64,
    /// Shred group identifier to which this record belongs.
    pub group_id: u64,
    /// AES-256-GCM-SIV encrypted record data.
    pub ciphertext: Vec<u8>,
    /// 12-byte initialization vector (nonce) for payload decryption.
    pub nonce: [u8; 12],
    /// DEK wrapped with Group KEK.
    pub wrapped_dek: Vec<u8>,
    /// 12-byte initialization vector (nonce) for DEK unwrapping.
    pub dek_nonce: [u8; 12],
}

#[derive(Debug)]
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
    record_deks: HashMap<u64, RecordDekEntry>,
}

/// Derives or retrieves a random sub-key (Group KEK) for a shred group via `KeyRegistry`.
pub fn derive_subkey(
    registry: &KeyRegistry,
    master_key: &KeyManager,
    group_id: u64,
) -> Result<SubKey> {
    registry.get_or_derive(master_key, group_id)
}

/// Listener trait for group revocation events.
pub trait GroupRevocationListener: Send + Sync {
    fn on_group_revoked(&self, group_id: u64);
}

/// In-memory thread-safe registry for envelope KV crypto-shredding keys.
#[derive(Default)]
pub struct KeyRegistry {
    groups: RwLock<HashMap<u64, GroupEntry>>,
    revoked_groups: RwLock<HashSet<u64>>,
    ever_registered_groups: RwLock<HashSet<u64>>,
    pub revocation_log: Option<Arc<RevocationLog>>,
    listeners: Vec<Arc<dyn GroupRevocationListener>>,
}

impl std::fmt::Debug for KeyRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyRegistry")
            .field("groups", &self.groups)
            .field("revoked_groups", &self.revoked_groups)
            .field("ever_registered_groups", &self.ever_registered_groups)
            .field("revocation_log", &self.revocation_log)
            .field("listeners_count", &self.listeners.len())
            .finish()
    }
}

impl KeyRegistry {
    /// Creates a new empty `KeyRegistry`.
    pub fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
            revoked_groups: RwLock::new(HashSet::new()),
            ever_registered_groups: RwLock::new(HashSet::new()),
            revocation_log: None,
            listeners: Vec::new(),
        }
    }

    /// Creates a new `KeyRegistry` backed by an in-memory `RevocationLog`.
    pub fn new_in_memory(
        clock: Arc<dyn contextra_ports::Clock>,
        signing_key: Option<ed25519_dalek::SigningKey>,
        verifying_key: ed25519_dalek::VerifyingKey,
    ) -> Self {
        Self::new().with_revocation_log(Arc::new(RevocationLog::new_in_memory(
            clock,
            signing_key,
            verifying_key,
        )))
    }

    /// Attaches an optional `RevocationLog` to check for persisted key/group revocations.
    pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self {
        self.revocation_log = Some(log);
        self
    }

    /// Attaches a `GroupRevocationListener` to be notified on group revocation.
    pub fn with_revocation_listener(mut self, listener: Arc<dyn GroupRevocationListener>) -> Self {
        self.listeners.push(listener);
        self
    }

    /// Retrieves the wrapped KEK and nonce for a group, if available and active.
    pub fn get_wrapped_kek(&self, group_id: u64) -> Option<(Vec<u8>, [u8; 12])> {
        if self.is_group_revoked(group_id) {
            return None;
        }
        let read_guard = self.groups.read().ok()?;
        let entry = read_guard.get(&group_id)?;
        Some((entry.wrapped_kek.clone(), entry.kek_nonce))
    }

    /// Retrieves the wrapped DEK and nonce for a record, if available and active.
    pub fn get_wrapped_dek(&self, group_id: u64, record_id: u64) -> Option<(Vec<u8>, [u8; 12])> {
        if self.is_group_revoked(group_id) {
            return None;
        }
        let read_guard = self.groups.read().ok()?;
        let entry = read_guard.get(&group_id)?;
        let rec = entry.record_deks.get(&record_id)?;
        if rec.revoked {
            return None;
        }
        Some((rec.wrapped_dek.clone(), rec.dek_nonce))
    }

    /// Checks if a group is revoked.
    // TODO(#JULES-P04-1, Implementer): [P04 / F-3 / CRITICAL & Phase D]
    // 1. Durability/Fail-Closed: Sicherstellen, dass bei fehlgeschlagenem Log-Reopen oder korruptem
    //    RevocationLog keine Entschlüsselung von geshreddeten Datensätzen möglich ist (fail-closed).
    // 2. Phase D Optimierung: Lock-freie Prüfung per `ArcSwap<HashSet<u64>>` oder Atomic-Bitset erwägen,
    //    um RwLock-Contention im heißen Lesepfad von `is_group_revoked` zu eliminieren.
    pub fn is_group_revoked(&self, group_id: u64) -> bool {
        if let Some(ref log) = self.revocation_log {
            if log.is_revoked(&RevocationTarget::Group(group_id)) {
                return true;
            }
            if log.verify_integrity().is_err() {
                return true;
            }
        }
        if let Ok(guard) = self.revoked_groups.read() {
            guard.contains(&group_id)
        } else {
            true
        }
    }

    /// Checks if a group was ever registered in this registry.
    pub fn was_group_ever_registered(&self, group_id: u64) -> bool {
        if let Ok(guard) = self.ever_registered_groups.read() {
            guard.contains(&group_id)
        } else {
            false
        }
    }

    /// Retrieves or generates a random Group KEK for `group_id`.
    /// Returns `Err(CryptoError::KeyRevoked(...))` if `group_id` has been revoked/shredded.
    pub fn get_or_derive(&self, master_key: &KeyManager, group_id: u64) -> Result<SubKey> {
        if self.is_group_revoked(group_id) {
            return Err(CryptoError::KeyRevoked(format!(
                "Shred group {group_id} has been revoked and cannot be accessed"
            )));
        }

        {
            let read_guard = self
                .groups
                .read()
                .map_err(|_| CryptoError::Crypto("KeyRegistry read lock poisoned".to_string()))?;
            if let Some(entry) = read_guard.get(&group_id) {
                return Ok(SubKey(entry.kek.0));
            }
        }

        let mut write_guard = self
            .groups
            .write()
            .map_err(|_| CryptoError::Crypto("KeyRegistry write lock poisoned".to_string()))?;

        if self.is_group_revoked(group_id) {
            return Err(CryptoError::Crypto(format!(
                "Shred group {group_id} has been revoked and cannot be accessed"
            )));
        }

        if let Some(entry) = write_guard.get(&group_id) {
            return Ok(SubKey(entry.kek.0));
        }

        // Generate a random 32-byte Group KEK (never derived from group_id)
        let mut kek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut kek_bytes);

        // AEAD wrap the KEK using Master Key
        let (wrapped_kek, kek_nonce) = master_key.encrypt_auto_nonce(&kek_bytes)?;

        let entry = GroupEntry {
            kek: GroupKek(kek_bytes),
            wrapped_kek,
            kek_nonce,
            record_deks: HashMap::new(),
        };

        write_guard.insert(group_id, entry);
        if let Ok(mut ever_guard) = self.ever_registered_groups.write() {
            ever_guard.insert(group_id);
        }
        Ok(SubKey(kek_bytes))
    }

    /// Revokes (destroys) the Group KEK for `group_id` (Group Deletion / Group Crypto-Shredding).
    /// Returns `true` if the group was active and is now revoked.
    pub fn revoke_group(&self, group_id: u64) -> Result<bool> {
        if let Some(ref log) = self.revocation_log {
            log.append(RevocationTarget::Group(group_id))
                .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        }

        let mut revoked_guard = match self.revoked_groups.write() {
            Ok(g) => g,
            Err(_) => return Ok(false),
        };

        if revoked_guard.contains(&group_id) {
            return Ok(false);
        }

        revoked_guard.insert(group_id);

        if let Ok(mut groups_guard) = self.groups.write() {
            if let Some(mut entry) = groups_guard.remove(&group_id) {
                entry.kek.0.zeroize();
                entry.wrapped_kek.zeroize();
                for (_, mut rec) in entry.record_deks.drain() {
                    rec.wrapped_dek.zeroize();
                }
            }
        }

        drop(revoked_guard);

        for listener in &self.listeners {
            listener.on_group_revoked(group_id);
        }

        Ok(true)
    }

    /// Revokes (destroys) the Group KEK for `group_id` in O(1). Alias for `revoke_group`.
    pub fn revoke_subkey(&self, group_id: u64) -> Result<bool> {
        self.revoke_group(group_id)
    }

    /// Revokes (destroys) the DEK wrap for a single record within `group_id` (Single Record Deletion).
    /// Leaves neighbor records in the same group intact and decryptable.
    pub fn revoke_record(&self, group_id: u64, record_id: u64) -> Result<bool> {
        if self.is_group_revoked(group_id) {
            return Ok(false);
        }

        if let Some(ref log) = self.revocation_log {
            log.append(RevocationTarget::Record(format!("{group_id}:{record_id}")))
                .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        }

        if let Ok(mut groups_guard) = self.groups.write() {
            if let Some(entry) = groups_guard.get_mut(&group_id) {
                if let Some(rec) = entry.record_deks.get_mut(&record_id) {
                    if !rec.revoked {
                        rec.revoked = true;
                        rec.wrapped_dek.zeroize();
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Returns `true` if `group_id` has an active KEK and is not revoked.
    pub fn is_group_active(&self, group_id: u64) -> bool {
        !self.is_group_revoked(group_id)
            && self
                .groups
                .read()
                .map(|g| g.contains_key(&group_id))
                .unwrap_or(false)
    }

    /// Returns `true` if `group_id` has an active sub-key in the registry.
    pub fn is_key_active(&self, group_id: u64) -> bool {
        self.is_group_active(group_id)
    }

    /// Returns `true` if `record_id` in `group_id` is active and not revoked.
    pub fn is_record_active(&self, group_id: u64, record_id: u64) -> bool {
        if self.is_group_revoked(group_id) {
            return false;
        }
        if let Ok(groups_guard) = self.groups.read() {
            if let Some(entry) = groups_guard.get(&group_id) {
                if let Some(rec) = entry.record_deks.get(&record_id) {
                    return !rec.revoked;
                }
            }
        }
        false
    }

    /// Encrypts a record using Envelope Encryption (Random KEK per group, Random DEK per record).
    pub fn encrypt_record(
        &self,
        master_key: &KeyManager,
        group_id: u64,
        record_id: u64,
        plaintext: &[u8],
    ) -> Result<EncryptedRecordPayload> {
        let subkey = self.get_or_derive(master_key, group_id)?;
        let kek_cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv KEK init failed: {e}")))?;

        let mut dek_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut dek_bytes);

        let mut dek_nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut dek_nonce);
        let wrapped_dek = kek_cipher
            .encrypt(Nonce::from_slice(&dek_nonce), dek_bytes.as_slice())
            .map_err(|e| CryptoError::Crypto(format!("DEK wrapping failed: {e}")))?;

        let dek_cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv DEK init failed: {e}")))?;
        let mut nonce = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let ciphertext = dek_cipher
            .encrypt(Nonce::from_slice(&nonce), plaintext)
            .map_err(|e| CryptoError::Crypto(format!("Payload encryption failed: {e}")))?;

        if let Ok(mut groups_guard) = self.groups.write() {
            if let Some(entry) = groups_guard.get_mut(&group_id) {
                entry.record_deks.insert(
                    record_id,
                    RecordDekEntry {
                        wrapped_dek: wrapped_dek.clone(),
                        dek_nonce,
                        revoked: false,
                    },
                );
            }
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

    /// Decrypts an `EncryptedRecordPayload` using Envelope Encryption.
    /// Fails with `CryptoError::Crypto` if group or record has been revoked.
    pub fn decrypt_record(
        &self,
        master_key: &KeyManager,
        payload: &EncryptedRecordPayload,
    ) -> Result<Vec<u8>> {
        if self.is_group_revoked(payload.group_id) {
            return Err(CryptoError::KeyRevoked(format!(
                "Group {} has been revoked/shredded",
                payload.group_id
            )));
        }

        if !self.is_record_active(payload.group_id, payload.record_id) {
            return Err(CryptoError::Crypto(format!(
                "Record {} in group {} is not active or has been revoked/shredded",
                payload.record_id, payload.group_id
            )));
        }

        if let Some((expected_wrapped_dek, expected_dek_nonce)) =
            self.get_wrapped_dek(payload.group_id, payload.record_id)
        {
            if payload.wrapped_dek != expected_wrapped_dek
                || payload.dek_nonce != expected_dek_nonce
            {
                return Err(CryptoError::Crypto(format!(
                    "Wrapped DEK mismatch for record {} in group {}",
                    payload.record_id, payload.group_id
                )));
            }
        }

        let subkey = self.get_or_derive(master_key, payload.group_id)?;
        let kek_cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv KEK init failed: {e}")))?;

        let dek_bytes = kek_cipher
            .decrypt(
                Nonce::from_slice(&payload.dek_nonce),
                payload.wrapped_dek.as_slice(),
            )
            .map_err(|e| CryptoError::Crypto(format!("DEK unwrap failed (key revoked?): {e}")))?;

        let dek_cipher = Aes256GcmSiv::new_from_slice(&dek_bytes)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv DEK init failed: {e}")))?;

        let plaintext = dek_cipher
            .decrypt(
                Nonce::from_slice(&payload.nonce),
                payload.ciphertext.as_slice(),
            )
            .map_err(|e| CryptoError::Crypto(format!("Payload decryption failed: {e}")))?;

        Ok(plaintext)
    }

    /// Encrypts plaintext using the sub-key (KEK) for `group_id`.
    pub fn encrypt_with_group(
        &self,
        master_key: &KeyManager,
        group_id: u64,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, [u8; 12])> {
        if self.is_group_revoked(group_id) {
            return Err(CryptoError::KeyRevoked(format!(
                "Shred group {group_id} has been revoked and cannot be accessed"
            )));
        }
        let subkey = self.get_or_derive(master_key, group_id)?;
        let cipher = Aes256GcmSiv::new_from_slice(&subkey.0)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv init failed: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::Crypto(format!("Encryption failed: {e}")))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypts ciphertext using the sub-key (KEK) for `group_id`.
    /// Returns `Err(CryptoError::Crypto(...))` if key was revoked or decryption fails.
    pub fn decrypt_with_group(
        &self,
        group_id: u64,
        ciphertext: &[u8],
        nonce_bytes: &[u8; 12],
    ) -> Result<Vec<u8>> {
        if self.is_group_revoked(group_id) || self.get_wrapped_kek(group_id).is_none() {
            return Err(CryptoError::KeyRevoked(format!(
                "Sub-key for group {group_id} has been revoked or is missing"
            )));
        }

        let read_guard = self
            .groups
            .read()
            .map_err(|_| CryptoError::Crypto("KeyRegistry read lock poisoned".to_string()))?;

        let entry = read_guard.get(&group_id).ok_or_else(|| {
            CryptoError::Crypto(format!(
                "Sub-key for group {group_id} has been revoked or is missing"
            ))
        })?;

        let cipher = Aes256GcmSiv::new_from_slice(&entry.kek.0)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv init failed: {e}")))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::Crypto(format!("Decryption failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_single_record_deletion_64_records() -> Result<()> {
        let km = KeyManager::try_new("test-passphrase-envelope", b"salt1")?;
        let registry = KeyRegistry::new();
        let group_id = 42;

        let mut payloads = Vec::with_capacity(64);
        for record_id in 0..64 {
            let plaintext = format!("Payload for record {record_id}");
            let payload =
                registry.encrypt_record(&km, group_id, record_id, plaintext.as_bytes())?;
            payloads.push(payload);
        }

        // Verify all 64 records decrypt successfully
        for (i, payload) in payloads.iter().enumerate() {
            let decrypted = registry.decrypt_record(&km, payload)?;
            assert_eq!(decrypted, format!("Payload for record {i}").as_bytes());
        }

        // Perform single record deletion (revoke record 10)
        let revoked = registry.revoke_record(group_id, 10)?;
        assert!(revoked, "Record 10 must be successfully revoked");

        // Record 10 MUST fail decryption
        let res_10 = registry.decrypt_record(&km, &payloads[10]);
        assert!(res_10.is_err(), "Revoked record 10 MUST fail decryption");

        // All remaining 63 neighbor records MUST remain readable
        for (i, payload) in payloads.iter().enumerate() {
            if i == 10 {
                continue;
            }
            let decrypted = registry.decrypt_record(&km, payload)?;
            assert_eq!(
                decrypted,
                format!("Payload for record {i}").as_bytes(),
                "Neighbor record {i} MUST remain decryptable after single record 10 deletion"
            );
        }

        Ok(())
    }

    #[test]
    fn test_envelope_group_deletion_and_attack_simulation() -> Result<()> {
        let km = KeyManager::try_new("test-passphrase-envelope", b"salt1")?;
        let registry = KeyRegistry::new();
        let group_id = 100;

        let payload =
            registry.encrypt_record(&km, group_id, 1, b"Group confidential data block")?;
        assert!(registry.is_group_active(group_id));

        // Revoke entire group (Group Deletion)
        let revoked = registry.revoke_group(group_id)?;
        assert!(revoked, "Group 100 revocation must succeed");
        assert!(!registry.is_group_active(group_id));

        // Decryption via decrypt_record MUST fail
        let res_decrypt = registry.decrypt_record(&km, &payload);
        assert!(
            res_decrypt.is_err(),
            "Decryption of record in revoked group MUST fail"
        );

        // Attempting to re-derive key for group 100 via get_or_derive MUST fail
        let res_get_derive = registry.get_or_derive(&km, group_id);
        assert!(
            res_get_derive.is_err(),
            "Re-deriving key for revoked group MUST fail and be consistently rejected"
        );

        // Attempting to encrypt_with_group for revoked group MUST fail
        let res_encrypt = registry.encrypt_with_group(&km, group_id, b"New payload");
        assert!(
            res_encrypt.is_err(),
            "encrypt_with_group MUST consistently reject revoked group"
        );

        Ok(())
    }

    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestListener {
        call_count: AtomicUsize,
        last_revoked_group: std::sync::Mutex<Option<u64>>,
    }

    impl TestListener {
        fn new() -> Self {
            Self {
                call_count: AtomicUsize::new(0),
                last_revoked_group: std::sync::Mutex::new(None),
            }
        }
    }

    impl GroupRevocationListener for TestListener {
        fn on_group_revoked(&self, group_id: u64) {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if let Ok(mut guard) = self.last_revoked_group.lock() {
                *guard = Some(group_id);
            }
        }
    }

    #[test]
    fn test_group_revocation_listener_first_and_second_revocation() -> Result<()> {
        let km = KeyManager::try_new("test-passphrase-listener", b"salt1")?;
        let listener = Arc::new(TestListener::new());
        let registry = KeyRegistry::new().with_revocation_listener(listener.clone());

        let group_id = 200;
        let _subkey = registry.get_or_derive(&km, group_id)?;

        // (a) First revocation calls listener exactly once
        let res1 = registry.revoke_group(group_id)?;
        assert!(res1, "First revocation must return Ok(true)");
        assert_eq!(
            listener.call_count.load(Ordering::SeqCst),
            1,
            "Listener must be called exactly once on first revocation"
        );
        assert_eq!(
            *listener.last_revoked_group.lock().unwrap(),
            Some(group_id),
            "Listener must receive the revoked group_id"
        );

        // (b) Second revocation does NOT call listener again
        let res2 = registry.revoke_group(group_id)?;
        assert!(!res2, "Second revocation must return Ok(false)");
        assert_eq!(
            listener.call_count.load(Ordering::SeqCst),
            1,
            "Listener must not be called again on second revocation"
        );

        Ok(())
    }

    #[test]
    fn test_group_revocation_listener_unknown_group_id() -> Result<()> {
        let listener = Arc::new(TestListener::new());
        let registry = KeyRegistry::new().with_revocation_listener(listener.clone());

        let unknown_group_id = 999;
        assert!(!registry.was_group_ever_registered(unknown_group_id));

        // (c) Revoking an unknown group_id reports Ok(true) and calls listener once on first attempt
        let res1 = registry.revoke_group(unknown_group_id)?;
        assert!(res1, "Revoking unknown group_id returns Ok(true)");
        assert_eq!(
            listener.call_count.load(Ordering::SeqCst),
            1,
            "Listener is called once when revoking an unknown group_id for the first time"
        );
        assert_eq!(
            *listener.last_revoked_group.lock().unwrap(),
            Some(unknown_group_id)
        );

        // Second revocation attempt for same unknown group_id returns Ok(false) and does not call listener
        let res2 = registry.revoke_group(unknown_group_id)?;
        assert!(
            !res2,
            "Second revocation of unknown group_id returns Ok(false)"
        );
        assert_eq!(
            listener.call_count.load(Ordering::SeqCst),
            1,
            "Listener is not called again on second revocation of unknown group_id"
        );

        Ok(())
    }
}
