// FILE-CONTEXT
// ZWECK: Durable, scope-granular Keyring implementing ADR-N27 for S4 Crypto-Erase across Collections and Tenants.
// INVARIANTEN: Zeroize-on-drop for all DEK handles, atomic disk writes with fsync (file + parent dir), RevocationLog integration before return, zero raw key material in Debug/logs/errors.
// STAND: TS:2026-10-18T00:00:00Z

#![forbid(unsafe_code)]

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use crate::revocation_log::{RevocationLog, RevocationTarget};
use aes_gcm_siv::{
    aead::{Aead, KeyInit, Payload},
    Aes256GcmSiv, Nonce,
};
use contextra_ports::{Clock, EntropyRng};
use contextra_types::{CollectionId, TenantId};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zeroize::Zeroizing;

const KEYRING_FILE_MAGIC: &[u8; 4] = b"CTKR";
const KEYRING_FILE_VERSION: u8 = 1;
const KEYRING_FILE_AAD: &[u8] = b"contextra-keyring-file-v1";

/// Identifies a key's cryptographic scope according to ADR-N27.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyScope {
    /// Tenant-level Key Encryption Key (KEK) scope bound to a specific epoch.
    TenantEpoch {
        /// Tenant identifier.
        tenant: TenantId,
        /// Epoch counter for key rotation/reset.
        epoch: u64,
    },
    /// Collection-level Data Encryption Key (DEK) scope bound to a specific incarnation.
    Collection {
        /// Parent tenant identifier.
        tenant: TenantId,
        /// Collection identifier.
        collection: CollectionId,
        /// Incarnation counter for collection reset/crypto-erase.
        incarnation: u64,
    },
}

impl KeyScope {
    /// Maps this scope to a canonical `RevocationTarget` for `RevocationLog` entries.
    pub fn to_revocation_target(&self) -> RevocationTarget {
        match self {
            Self::TenantEpoch { tenant, epoch } => {
                RevocationTarget::Kek(format!("tenant:{}:epoch:{}", tenant.inner(), epoch))
            }
            Self::Collection { tenant, collection, incarnation } => {
                RevocationTarget::Dek(format!(
                    "tenant:{}:col:{}:inc:{}",
                    tenant.inner(),
                    collection.inner(),
                    incarnation
                ))
            }
        }
    }

    /// Returns the tenant identifier bound to this scope.
    pub fn tenant_id(&self) -> TenantId {
        match self {
            Self::TenantEpoch { tenant, .. } => *tenant,
            Self::Collection { tenant, .. } => *tenant,
        }
    }
}

/// 16-byte random identifier for a key within the keyring.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId(pub [u8; 16]);

impl KeyId {
    /// Generates a new random `KeyId` using the provided `EntropyRng`.
    pub fn generate(rng: &dyn EntropyRng) -> Self {
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Returns a reference to the raw 16-byte slice.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl std::fmt::Debug for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyId(")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        write!(f, ")")
    }
}

/// Lifecycle state of a scope key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyState {
    /// Key is active and available for encryption/decryption.
    Active,
    /// Key is scheduled for destruction after `not_before` nanoseconds (grace period).
    PendingDestruction {
        /// Earliest timestamp (unix nanoseconds) at which destruction can be finalized.
        not_before: u64,
    },
    /// Key has been permanently scrubbed and destroyed at `at` timestamp (unix nanoseconds).
    Destroyed {
        /// Timestamp (unix nanoseconds) when key destruction occurred.
        at: u64,
    },
}

impl KeyState {
    /// Returns `true` if this key is in the `Destroyed` state.
    pub fn is_destroyed(&self) -> bool {
        matches!(self, Self::Destroyed { .. })
    }
}

/// Handle holding a DEK in zeroized memory.
///
/// Uses injected `EntropyRng` port for nonces and does NOT implement `Clone` or `Debug` that exposes raw key material.
pub struct DekHandle {
    key_id: KeyId,
    scope: KeyScope,
    raw_key: Zeroizing<[u8; 32]>,
    rng: Arc<dyn EntropyRng>,
}

impl DekHandle {
    /// Constructs a new `DekHandle`.
    pub fn new(
        key_id: KeyId,
        scope: KeyScope,
        raw_key: [u8; 32],
        rng: Arc<dyn EntropyRng>,
    ) -> Self {
        Self {
            key_id,
            scope,
            raw_key: Zeroizing::new(raw_key),
            rng,
        }
    }

    /// Returns the `KeyId` associated with this handle.
    pub fn key_id(&self) -> KeyId {
        self.key_id
    }

    /// Returns a reference to the bound `KeyScope`.
    pub fn scope(&self) -> &KeyScope {
        &self.scope
    }

    /// Exposes references to raw key bytes for internal cipher operations.
    pub fn raw_key_bytes(&self) -> &[u8; 32] {
        &self.raw_key
    }

    /// Encrypts plaintext with AES-256-GCM-SIV using this DEK and Additional Authenticated Data (AAD).
    /// Generates nonces strictly via the injected `EntropyRng`.
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<(Vec<u8>, [u8; 12])> {
        let cipher = Aes256GcmSiv::new_from_slice(&*self.raw_key)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv init failed: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        self.rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: plaintext,
            aad,
        };

        let ciphertext = cipher
            .encrypt(nonce, payload)
            .map_err(|e| CryptoError::Encryption(format!("DekHandle encrypt failed: {e}")))?;

        Ok((ciphertext, nonce_bytes))
    }

    /// Decrypts ciphertext with AES-256-GCM-SIV using this DEK and Additional Authenticated Data (AAD).
    pub fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12], aad: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256GcmSiv::new_from_slice(&*self.raw_key)
            .map_err(|e| CryptoError::Crypto(format!("Aes256GcmSiv init failed: {e}")))?;

        let nonce = Nonce::from_slice(nonce_bytes);
        let payload = Payload {
            msg: ciphertext,
            aad,
        };

        cipher
            .decrypt(nonce, payload)
            .map_err(|e| CryptoError::Decryption(format!("DekHandle decrypt failed: {e}")))
    }
}

impl std::fmt::Debug for DekHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DekHandle")
            .field("key_id", &self.key_id)
            .field("scope", &self.scope)
            .field("raw_key", &"***REDACTED***")
            .finish()
    }
}

/// Receipt confirming key destruction and `RevocationLog` persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestructionReceipt {
    /// List of destroyed `KeyId`s associated with the scope.
    pub key_ids: Vec<KeyId>,
    /// Timestamp (unix nanoseconds) when key destruction occurred.
    pub destroyed_at: u64,
    /// Entry hash recorded in the persistent `RevocationLog`.
    pub revocation_entry_hash: [u8; 32],
}

/// Persistent keyring entry recorded on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyringEntry {
    /// Key identifier.
    pub id: KeyId,
    /// Associated cryptographic scope.
    pub scope: KeyScope,
    /// Current lifecycle state.
    pub state: KeyState,
    /// Wrapped (encrypted) DEK bytes.
    pub wrapped_dek: Vec<u8>,
    /// Nonce used for wrapping the DEK.
    pub nonce: [u8; 12],
    /// Creation timestamp (unix nanoseconds).
    pub created_at: u64,
}

#[derive(Serialize, Deserialize)]
struct KeyringFilePayload {
    entries: Vec<KeyringEntry>,
}

/// Scope-granular keyring managing DEKs and KEKs with durable S4 Crypto-Erase capabilities.
pub struct ScopeKeyring {
    dir: PathBuf,
    keyring_file: PathBuf,
    key_manager: KeyManager,
    clock: Arc<dyn Clock>,
    rng: Arc<dyn EntropyRng>,
    revocation_log: Arc<RevocationLog>,
    entries: RwLock<HashMap<KeyId, KeyringEntry>>,
    scope_index: RwLock<HashMap<KeyScope, Vec<KeyId>>>,
}

fn derive_wrapping_key(key_manager: &KeyManager, scope: &KeyScope) -> Result<KeyManager> {
    let scope_info = bincode::serialize(scope)
        .map_err(|e| CryptoError::Crypto(format!("Failed to serialize KeyScope: {e}")))?;
    let prefix = b"contextra-scope-kek-v1:";
    let cap = prefix.len().saturating_add(scope_info.len());
    let mut info = Vec::with_capacity(cap);
    info.extend_from_slice(prefix);
    info.extend_from_slice(&scope_info);
    key_manager.derive_segment_key_bytes(&info)
}

fn derive_file_cipher(key_manager: &KeyManager) -> Result<KeyManager> {
    key_manager.derive_segment_key_bytes(b"contextra-scope-keyring-file-v1")
}

fn fsync_file_and_dir(file_path: &Path) -> Result<()> {
    if file_path.exists() {
        let file = std::fs::File::open(file_path)
            .map_err(|e| CryptoError::Crypto(format!("Failed to open file for fsync: {e}")))?;
        file.sync_all()
            .map_err(|e| CryptoError::Crypto(format!("Failed to fsync file: {e}")))?;
    }

    if let Some(parent) = file_path.parent() {
        if parent.exists() {
            if let Ok(dir_file) = std::fs::File::open(parent) {
                let _ = dir_file.sync_all();
            }
        }
    }
    Ok(())
}

impl ScopeKeyring {
    /// Opens or creates a durable `ScopeKeyring` at `dir`.
    pub fn open(
        dir: impl AsRef<Path>,
        key_manager: &KeyManager,
        clock: Arc<dyn Clock>,
        rng: Arc<dyn EntropyRng>,
    ) -> Result<Self> {
        let dir_buf = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir_buf)
            .map_err(|e| CryptoError::Crypto(format!("Failed to create keyring dir: {e}")))?;

        let keyring_file = dir_buf.join("scope_keyring.dat");
        let rev_log_path = dir_buf.join("revocation.log");

        let signing_key = key_manager.derive_revocation_signing_key()?;
        let verifying_key = signing_key.verifying_key();

        let has_existing = keyring_file.exists();
        let revocation_log = Arc::new(RevocationLog::open_or_create_if_fresh(
            &rev_log_path,
            clock.clone(),
            Some(signing_key),
            verifying_key,
            has_existing,
        )?);

        let file_cipher = derive_file_cipher(key_manager)?;

        let mut entries = HashMap::new();
        let mut scope_index: HashMap<KeyScope, Vec<KeyId>> = HashMap::new();

        if keyring_file.exists() {
            let raw_bytes = std::fs::read(&keyring_file)
                .map_err(|e| CryptoError::Crypto(format!("Failed to read keyring file: {e}")))?;

            if raw_bytes.len() < 17 {
                return Err(CryptoError::IntegrityViolation);
            }

            let magic = raw_bytes.get(0..4).ok_or(CryptoError::IntegrityViolation)?;
            let version = raw_bytes.get(4).ok_or(CryptoError::IntegrityViolation)?;

            if magic != KEYRING_FILE_MAGIC || *version != KEYRING_FILE_VERSION {
                return Err(CryptoError::IntegrityViolation);
            }

            let nonce_slice = raw_bytes.get(5..17).ok_or(CryptoError::IntegrityViolation)?;
            let mut nonce_bytes = [0u8; 12];
            nonce_bytes.copy_from_slice(nonce_slice);

            let ciphertext = raw_bytes.get(17..).ok_or(CryptoError::IntegrityViolation)?;

            // AAD-bound file decryption preventing block-swapping/substitution attacks
            let cipher = Aes256GcmSiv::new_from_slice(file_cipher.raw_key_bytes())
                .map_err(|e| CryptoError::Crypto(format!("File cipher init failed: {e}")))?;
            let nonce = Nonce::from_slice(&nonce_bytes);
            let payload = Payload {
                msg: ciphertext,
                aad: KEYRING_FILE_AAD,
            };

            let decrypted = cipher
                .decrypt(nonce, payload)
                .map_err(|_| CryptoError::IntegrityViolation)?;

            let keyring_payload: KeyringFilePayload = bincode::deserialize(&decrypted)
                .map_err(|_| CryptoError::IntegrityViolation)?;

            entries = keyring_payload
                .entries
                .into_iter()
                .map(|e| (e.id, e))
                .collect::<HashMap<_, _>>();

            for entry in entries.values() {
                scope_index
                    .entry(entry.scope.clone())
                    .or_default()
                    .push(entry.id);
            }
        }

        // Crash recovery cross-check: Ensure alignment between RevocationLog and KeyringEntry states
        let mut modified = false;
        let now_nanos = clock.now_unix_nanos();

        for entry in entries.values_mut() {
            let rev_target = entry.scope.to_revocation_target();
            let is_revoked = revocation_log.is_revoked(&rev_target);

            if is_revoked && !entry.state.is_destroyed() {
                // Recover from crash after RevocationLog append but before keyring file sync
                entry.state = KeyState::Destroyed { at: now_nanos };
                entry.wrapped_dek = vec![0u8; entry.wrapped_dek.len()];
                modified = true;
            } else if entry.state.is_destroyed() && !is_revoked {
                // Recover from crash after keyring file sync but before RevocationLog append
                revocation_log.append(rev_target)?;
            }
        }

        let keyring = Self {
            dir: dir_buf,
            keyring_file,
            key_manager: key_manager.clone(),
            clock,
            rng,
            revocation_log,
            entries: RwLock::new(entries),
            scope_index: RwLock::new(scope_index),
        };

        if modified {
            keyring.persist_and_sync()?;
        }

        Ok(keyring)
    }

    fn persist_and_sync(&self) -> Result<()> {
        let entries_guard = self.entries.read();
        let payload = KeyringFilePayload {
            entries: entries_guard.values().cloned().collect(),
        };

        let serialized = bincode::serialize(&payload)
            .map_err(|e| CryptoError::Crypto(format!("Failed to serialize keyring payload: {e}")))?;

        let file_cipher = derive_file_cipher(&self.key_manager)?;
        let cipher = Aes256GcmSiv::new_from_slice(file_cipher.raw_key_bytes())
            .map_err(|e| CryptoError::Crypto(format!("File cipher init failed: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        self.rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // AAD-bound payload encryption protecting keyring file from block-swapping
        let aead_payload = Payload {
            msg: &serialized,
            aad: KEYRING_FILE_AAD,
        };

        let ciphertext = cipher
            .encrypt(nonce, aead_payload)
            .map_err(|e| CryptoError::Encryption(format!("Keyring file encrypt failed: {e}")))?;

        let header_len = 17usize;
        let cap = header_len.saturating_add(ciphertext.len());
        let mut buf = Vec::with_capacity(cap);
        buf.extend_from_slice(KEYRING_FILE_MAGIC);
        buf.push(KEYRING_FILE_VERSION);
        buf.extend_from_slice(&nonce_bytes);
        buf.extend_from_slice(&ciphertext);

        contextra_durable_fs::atomic_replace(&self.keyring_file, &buf)
            .map_err(|e| CryptoError::Crypto(format!("Atomic write of keyring file failed: {e}")))?;

        fsync_file_and_dir(&self.keyring_file)?;
        Ok(())
    }

    /// Creates a new DEK bound to `scope` and persists it to disk.
    pub fn create_dek(&self, scope: KeyScope) -> Result<(KeyId, DekHandle)> {
        let rev_target = scope.to_revocation_target();
        if self.revocation_log.is_revoked(&rev_target) {
            return Err(CryptoError::KeyRevoked(format!(
                "Cannot create key for revoked scope {:?}",
                scope
            )));
        }

        let key_id = KeyId::generate(self.rng.as_ref());
        let mut raw_dek = [0u8; 32];
        self.rng.fill_bytes(&mut raw_dek);

        let wrapping_km = derive_wrapping_key(&self.key_manager, &scope)?;
        let aad = bincode::serialize(&(&scope, &key_id))
            .map_err(|e| CryptoError::Crypto(format!("Failed to serialize AAD: {e}")))?;

        let cipher = Aes256GcmSiv::new_from_slice(wrapping_km.raw_key_bytes())
            .map_err(|e| CryptoError::Crypto(format!("Wrapping cipher init failed: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        self.rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            msg: &raw_dek,
            aad: &aad,
        };

        let wrapped_dek = cipher
            .encrypt(nonce, payload)
            .map_err(|e| CryptoError::Encryption(format!("Failed to wrap DEK: {e}")))?;

        let created_at = self.clock.now_unix_nanos();
        let entry = KeyringEntry {
            id: key_id,
            scope: scope.clone(),
            state: KeyState::Active,
            wrapped_dek,
            nonce: nonce_bytes,
            created_at,
        };

        {
            let mut entries_guard = self.entries.write();
            let mut scope_guard = self.scope_index.write();
            entries_guard.insert(key_id, entry);
            scope_guard.entry(scope.clone()).or_default().push(key_id);
        }

        self.persist_and_sync()?;

        let handle = DekHandle::new(key_id, scope, raw_dek, self.rng.clone());
        Ok((key_id, handle))
    }

    /// Retrieves an active DEK handle by `key_id`.
    pub fn dek(&self, key_id: KeyId) -> Result<DekHandle> {
        let entry = {
            let entries_guard = self.entries.read();
            entries_guard
                .get(&key_id)
                .cloned()
                .ok_or_else(|| CryptoError::InvalidInput(format!("KeyId {:?} not found", key_id)))?
        };

        if entry.state.is_destroyed() {
            return Err(CryptoError::KeyRevoked(format!(
                "Key {:?} has been destroyed",
                key_id
            )));
        }

        let rev_target = entry.scope.to_revocation_target();
        if self.revocation_log.is_revoked(&rev_target) {
            return Err(CryptoError::KeyRevoked(format!(
                "Scope {:?} for key {:?} has been revoked",
                entry.scope, key_id
            )));
        }

        let wrapping_km = derive_wrapping_key(&self.key_manager, &entry.scope)?;
        let aad = bincode::serialize(&(&entry.scope, &key_id))
            .map_err(|e| CryptoError::Crypto(format!("Failed to serialize AAD: {e}")))?;

        let cipher = Aes256GcmSiv::new_from_slice(wrapping_km.raw_key_bytes())
            .map_err(|e| CryptoError::Crypto(format!("Wrapping cipher init failed: {e}")))?;

        let nonce = Nonce::from_slice(&entry.nonce);
        let payload = Payload {
            msg: &entry.wrapped_dek,
            aad: &aad,
        };

        let raw_dek_vec = cipher
            .decrypt(nonce, payload)
            .map_err(|_| CryptoError::IntegrityViolation)?;

        if raw_dek_vec.len() != 32 {
            return Err(CryptoError::IntegrityViolation);
        }

        let mut raw_dek = [0u8; 32];
        raw_dek.copy_from_slice(&raw_dek_vec);

        Ok(DekHandle::new(key_id, entry.scope, raw_dek, self.rng.clone()))
    }

    /// Schedules key destruction for `scope` after `grace_period_nanos`.
    pub fn begin_destruction(&self, scope: &KeyScope, grace_period_nanos: u64) -> Result<()> {
        let now_nanos = self.clock.now_unix_nanos();
        let not_before = now_nanos.saturating_add(grace_period_nanos);

        let key_ids = {
            let scope_guard = self.scope_index.read();
            scope_guard.get(scope).cloned().unwrap_or_default()
        };

        if key_ids.is_empty() {
            return Ok(());
        }

        {
            let mut entries_guard = self.entries.write();
            for key_id in &key_ids {
                if let Some(entry) = entries_guard.get_mut(key_id) {
                    if entry.state == KeyState::Active {
                        entry.state = KeyState::PendingDestruction { not_before };
                    }
                }
            }
        }

        self.persist_and_sync()
    }

    /// Permanently destroys all keys associated with `scope`, scrubs file bytes via durable-fs, and writes to `RevocationLog`.
    pub fn destroy(&self, scope: &KeyScope) -> Result<DestructionReceipt> {
        let now_nanos = self.clock.now_unix_nanos();

        let key_ids = {
            let scope_guard = self.scope_index.read();
            scope_guard.get(scope).cloned().unwrap_or_default()
        };

        {
            let entries_guard = self.entries.read();
            for key_id in &key_ids {
                if let Some(entry) = entries_guard.get(key_id) {
                    if let KeyState::PendingDestruction { not_before } = entry.state {
                        if now_nanos < not_before {
                            return Err(CryptoError::InvalidInput(format!(
                                "Grace period not expired for key {:?}",
                                key_id
                            )));
                        }
                    }
                }
            }
        }

        {
            let mut entries_guard = self.entries.write();
            for key_id in &key_ids {
                if let Some(entry) = entries_guard.get_mut(key_id) {
                    entry.state = KeyState::Destroyed { at: now_nanos };
                    // Overwrite/scrub wrapped DEK bytes in memory before persisting
                    entry.wrapped_dek = vec![0u8; entry.wrapped_dek.len()];
                }
            }
        }

        // Persist updated keyring entries (with zeroed wrapped_dek for destroyed keys)
        // to disk via atomic replace and fsync.
        self.persist_and_sync()?;

        // Append revocation entry to RevocationLog and fsync
        let rev_target = scope.to_revocation_target();
        let rev_entry = self.revocation_log.append(rev_target)?;

        fsync_file_and_dir(&self.keyring_file)?;

        Ok(DestructionReceipt {
            key_ids,
            destroyed_at: now_nanos,
            revocation_entry_hash: rev_entry.entry_hash,
        })
    }
}

impl std::fmt::Debug for ScopeKeyring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScopeKeyring")
            .field("dir", &self.dir)
            .field("keyring_file", &self.keyring_file)
            .field("key_manager", &"***REDACTED***")
            .field("entries_count", &self.entries.read().len())
            .finish()
    }
}
