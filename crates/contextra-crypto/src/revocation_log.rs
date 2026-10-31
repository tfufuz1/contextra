// FILE-CONTEXT
// ZWECK: Persistierter, Ed25519-signierter, manipulationssicherer Append-Only Widerrufslog (v17 Teil 16.1).
// INVARIANTEN: Monotone Sequence Numbers (0, 1, ...). Tamper-proof SHA-256 Hash-Verkettung. Ed25519 Signatur-Verifikation pro Eintrag.
// ABGRENZUNG PROMPT 10: Prompt 10 definiert KEK/DEK Envelope-Datenstrukturen; revocation_log stellt die persistierte Widerrufs-Engine bereit.
// ABGRENZUNG PROMPT 12: Prompt 12 betrifft die Audit-Hash-Kette, die diesen Revocation-Log als Eingabequelle referenzieren kann, aber ein eigenständiges, separates System-Artefakt ist.
// HOTSPOTS: [RevocationLog::open_or_create, RevocationLog::append, RevocationLog::verify_chain]

#![forbid(unsafe_code)]

//! Persistierter Ed25519-signierter Append-Only Widerrufslog (v17 Teil 16.1).
//!
//! # Architektur & v17 Teil 16.1
//! Dieser Log speichert jeden Widerruf eines KEKs, DEKs, einer Shredding-Gruppe oder eines Records
//! als dauerhaften, Ed25519-signierten, manipulationssicheren Hash-Ketten-Eintrag.
//!
//! # Abgrenzung zu Prompt 10 (Envelope-Schema)
//! Prompt 10 spezifiziert die KEK/DEK Envelope-Datenstrukturen und Hierarchien.
//! `revocation_log` stellt die kryptographische Widerrufs-Engine und Durchsetzung für Schlüssel und Gruppen bereit.
//!
//! # Abgrenzung zu Prompt 12 (Audit-Hash-Kette)
//! Prompt 12 regelt die globale Audit-Hash-Kette von Contextra. Der Widerrufslog dient der
//! Audit-Kette als optionale Eingabequelle, bleibt aber als eigenständiges, isoliertes Artefakt bestehen.

use crate::error::{CryptoError, Result};
use ahash::AHashSet;
use contextra_ports::{Clock, Vfs};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Das Ziel eines Widerruf-Eintrags im Log (Group ID, KEK ID, DEK ID oder Record ID).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RevocationTarget {
    /// Shredding Group ID (z. B. für `KeyRegistry`)
    Group(u64),
    /// Key Encryption Key (KEK) Identifier
    Kek(String),
    /// Data Encryption Key (DEK) Identifier
    Dek(String),
    /// Specific Record Identifier
    Record(String),
}

impl RevocationTarget {
    /// Gibt eine kanonische Byte-Repräsentation des Targets für das Hashing zurück.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match self {
            Self::Group(id) => format!("group:{id}").into_bytes(),
            Self::Kek(id) => format!("kek:{id}").into_bytes(),
            Self::Dek(id) => format!("dek:{id}").into_bytes(),
            Self::Record(id) => format!("record:{id}").into_bytes(),
        }
    }
}

/// Ein einzelner Eintrag im manipulationssicheren Widerrufslog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationEntry {
    /// Monotone Sequenznummer im Log (0, 1, 2, ...).
    pub sequence_number: u64,
    /// Das widerrufene Target.
    pub target: RevocationTarget,
    /// Widerrufszeitpunkt in Nanosekunden seit UNIX_EPOCH (aus dem Clock-Port).
    pub revoked_at_nanos: u64,
    /// SHA-256-Hash des vorherigen Logeintrags (`[0u8; 32]` für Genesis-Eintrag 0).
    pub prev_hash: [u8; 32],
    /// SHA-256-Hash dieses Eintrags (über Sequenznummer, timestamp, prev_hash und canonical target bytes).
    pub entry_hash: [u8; 32],
    /// Ed25519-Signatur über `entry_hash` (64 Bytes).
    #[serde(with = "serde_64_bytes")]
    pub signature: [u8; 64],
}

mod serde_64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes for Ed25519 signature"))
    }
}

/// Berechnet den SHA-256-Hash eines Eintrags aus seinen kanonischen Feldern.
pub fn compute_entry_hash(
    sequence_number: u64,
    target: &RevocationTarget,
    revoked_at_nanos: u64,
    prev_hash: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(sequence_number.to_le_bytes());
    hasher.update(revoked_at_nanos.to_le_bytes());
    hasher.update(prev_hash);
    hasher.update(target.canonical_bytes());
    hasher.finalize().into()
}

/// Thread-sicherer, persistierter, Ed25519-signierter Widerrufslog.
pub struct RevocationLog {
    storage: Option<(Arc<dyn Vfs>, PathBuf)>,
    signing_key: Option<SigningKey>,
    verifying_key: VerifyingKey,
    clock: Arc<dyn Clock>,
    entries: RwLock<Vec<RevocationEntry>>,
    revoked_targets: RwLock<AHashSet<RevocationTarget>>,
}

impl RevocationLog {
    /// Erstellt einen neuen In-Memory-Widerrufslog ohne Dateipersistenz.
    pub fn new_in_memory(
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
    ) -> Self {
        Self {
            storage: None,
            signing_key,
            verifying_key,
            clock,
            entries: RwLock::new(Vec::new()),
            revoked_targets: RwLock::new(AHashSet::new()),
        }
    }

    /// Öffnet einen bestehenden Widerrufslog oder erzeugt einen neuen unter `path` über den Vfs-Port.
    ///
    /// Beim Öffnen einer bestehenden Datei wird die komplette Hash-Kette sowie jede
    /// Ed25519-Signatur anhand von `verifying_key` verifiziert.
    /// Bei Integritätsverletzungen bricht das Öffnen mit `CryptoError::IntegrityViolation` ab.
    pub fn open_or_create(
        vfs: Arc<dyn Vfs>,
        path: impl AsRef<Path>,
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
    ) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let mut entries = Vec::new();
        let mut revoked_targets = AHashSet::new();

        let exists = vfs.exists(&path_buf).map_err(CryptoError::Storage)?;

        if exists {
            let contents = vfs.read(&path_buf).map_err(CryptoError::Storage)?;

            if !contents.is_empty() {
                let parsed_entries: Vec<RevocationEntry> =
                    bincode::deserialize(&contents).map_err(|_| CryptoError::IntegrityViolation)?;

                Self::verify_chain_entries(&parsed_entries, &verifying_key)?;

                for entry in &parsed_entries {
                    revoked_targets.insert(entry.target.clone());
                }
                entries = parsed_entries;
            }
        }

        Ok(Self {
            storage: Some((vfs, path_buf)),
            signing_key,
            verifying_key,
            clock,
            entries: RwLock::new(entries),
            revoked_targets: RwLock::new(revoked_targets),
        })
    }

    /// Verifiziert die Integrität der gesamten Logeinträge-Kette (Sequenz, Hashes, Ed25519-Signaturen).
    pub fn verify_integrity(&self) -> Result<()> {
        let guard = self.entries.read();
        Self::verify_chain_entries(&guard, &self.verifying_key)
    }

    /// Hilfsfunktion zur Verifikation einer Liste von Eintrags-Ketten.
    fn verify_chain_entries(
        entries: &[RevocationEntry],
        verifying_key: &VerifyingKey,
    ) -> Result<()> {
        let mut expected_prev_hash = [0u8; 32];

        for (idx, entry) in entries.iter().enumerate() {
            if entry.sequence_number != idx as u64 {
                return Err(CryptoError::IntegrityViolation);
            }

            if entry.prev_hash != expected_prev_hash {
                return Err(CryptoError::IntegrityViolation);
            }

            let recomputed_hash = compute_entry_hash(
                entry.sequence_number,
                &entry.target,
                entry.revoked_at_nanos,
                &entry.prev_hash,
            );

            if entry.entry_hash != recomputed_hash {
                return Err(CryptoError::IntegrityViolation);
            }

            let sig = Signature::from_bytes(&entry.signature);
            verifying_key
                .verify(&entry.entry_hash, &sig)
                .map_err(|_| CryptoError::IntegrityViolation)?;

            expected_prev_hash = entry.entry_hash;
        }

        Ok(())
    }

    /// Fügt einen neuen Widerrufs-Eintrag für `target` an den Log an.
    ///
    /// Nutzt den injizierten Clock-Port zur Generierung des Zeitstempels und den Signierschlüssel
    /// zur Erstellung der Ed25519-Signatur. Persistiert die aktualisierte Kette sofort auf Disk.
    pub fn append(&self, target: RevocationTarget) -> Result<RevocationEntry> {
        let signing_key = self.signing_key.as_ref().ok_or_else(|| {
            CryptoError::Crypto("Signing key not configured for RevocationLog append".to_string())
        })?;

        let mut entries_guard = self.entries.write();
        let mut revoked_guard = self.revoked_targets.write();

        let sequence_number = entries_guard.len() as u64;
        let prev_hash = entries_guard
            .last()
            .map(|e| e.entry_hash)
            .unwrap_or([0u8; 32]);

        let revoked_at_nanos = self.clock.now_unix_nanos();
        let entry_hash = compute_entry_hash(sequence_number, &target, revoked_at_nanos, &prev_hash);
        let signature_bytes = signing_key.sign(&entry_hash).to_bytes();

        let new_entry = RevocationEntry {
            sequence_number,
            target: target.clone(),
            revoked_at_nanos,
            prev_hash,
            entry_hash,
            signature: signature_bytes,
        };

        // Fall 1: Persistenter Log über VFS.
        // Crash-Konsistenz / Fail-Closed: Zuerst den neuen Gesamtzustand serialisieren
        // und via `write_atomic` auf Platte schreiben. Erst bei Erfolg In-Memory-Zustand updaten.
        if let Some((ref vfs, ref path)) = self.storage {
            let mut persisted_entries = entries_guard.clone();
            persisted_entries.push(new_entry.clone());

            let serialized = bincode::serialize(&persisted_entries).map_err(|e| {
                CryptoError::Crypto(format!("Failed to serialize revocation log: {e}"))
            })?;

            vfs.write_atomic(path, &serialized)
                .map_err(CryptoError::Storage)?;
        }

        // Erst NACH erfolgreicher Persistierung (oder wenn rein in-memory):
        entries_guard.push(new_entry.clone());
        revoked_guard.insert(target);

        Ok(new_entry)
    }

    /// Gibt eine Kopie aller aktuell im Log widerrufenen Targets zurück.
    pub fn revoked_targets(&self) -> AHashSet<RevocationTarget> {
        self.revoked_targets.read().clone()
    }

    /// Prüft, ob ein gegebenes Target (KEK, DEK, Group oder Record) widerrufen wurde.
    pub fn is_revoked(&self, target: &RevocationTarget) -> bool {
        let guard = self.revoked_targets.read();
        guard.contains(target)
    }

    /// Gibt die Anzahl der Widerrufe im Log zurück.
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Prüft, ob der Log leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }
}

impl std::fmt::Debug for RevocationLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RevocationLog")
            .field("storage_path", &self.storage.as_ref().map(|(_, p)| p))
            .field("verifying_key", &self.verifying_key)
            .field("entries_count", &self.entries.read().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyManager;
    use crate::kv_shredding::KeyRegistry;
    use contextra_ports::SystemClock;
    use contextra_testkit::FaultVfs;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use std::io::ErrorKind;

    struct TestVfs {
        inner: FaultVfs,
    }

    impl TestVfs {
        fn new() -> Self {
            Self {
                inner: FaultVfs::new(),
            }
        }
    }

    impl Vfs for TestVfs {
        fn read(&self, path: &Path) -> contextra_ports::Result<Vec<u8>> {
            self.inner.read_file(path).map_err(|e| {
                if e.kind() == ErrorKind::NotFound {
                    contextra_types::ContextraError::NotFound(path.display().to_string())
                } else {
                    contextra_types::ContextraError::Io(e)
                }
            })
        }

        fn write_atomic(&self, path: &Path, data: &[u8]) -> contextra_ports::Result<()> {
            self.inner
                .write_file(path, data)
                .map_err(contextra_types::ContextraError::Io)
        }

        fn create_dir_all(&self, _path: &Path) -> contextra_ports::Result<()> {
            Ok(())
        }

        fn remove_file(&self, path: &Path) -> contextra_ports::Result<()> {
            let _ = path;
            Ok(())
        }

        fn exists(&self, path: &Path) -> contextra_ports::Result<bool> {
            match self.inner.read_file(path) {
                Ok(_) => Ok(true),
                Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
                Err(e) => Err(contextra_types::ContextraError::Io(e)),
            }
        }

        fn len(&self, path: &Path) -> contextra_ports::Result<u64> {
            let data = self.read(path)?;
            Ok(data.len() as u64)
        }

        fn list_dir(&self, _path: &Path) -> contextra_ports::Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn sync_dir(&self, _path: &Path) -> contextra_ports::Result<()> {
            self.inner
                .sync()
                .map_err(contextra_types::ContextraError::Io)
        }
    }

    fn test_keypair() -> (SigningKey, VerifyingKey) {
        let sk = SigningKey::generate(&mut OsRng);
        let vk = sk.verifying_key();
        (sk, vk)
    }

    #[test]
    fn test_in_memory_revocation_log_append_and_query() -> Result<()> {
        let (sk, vk) = test_keypair();
        let clock = Arc::new(SystemClock::new());
        let log = RevocationLog::new_in_memory(clock, Some(sk), vk);

        let kek_target = RevocationTarget::Kek("kek-101".into());
        let group_target = RevocationTarget::Group(42);

        assert!(!log.is_revoked(&kek_target));
        assert!(!log.is_revoked(&group_target));

        log.append(kek_target.clone())?;
        log.append(group_target.clone())?;

        assert!(log.is_revoked(&kek_target));
        assert!(log.is_revoked(&group_target));
        assert!(!log.is_revoked(&RevocationTarget::Dek("dek-999".into())));

        assert_eq!(log.len(), 2);
        assert!(log.verify_integrity().is_ok());
        Ok(())
    }

    #[test]
    fn test_persistence_and_simulated_restart() -> Result<()> {
        let vfs: Arc<dyn Vfs> = Arc::new(TestVfs::new());
        let log_path = PathBuf::from("/revocation.log");
        let (sk, vk) = test_keypair();
        let clock = Arc::new(SystemClock::new());

        let kek_target = RevocationTarget::Kek("kek-v17".into());
        let group_target = RevocationTarget::Group(1001);

        {
            let log1 =
                RevocationLog::open_or_create(vfs.clone(), &log_path, clock.clone(), Some(sk), vk)?;
            log1.append(kek_target.clone())?;
            log1.append(group_target.clone())?;
            assert_eq!(log1.len(), 2);
            // Drop log1 simulating process shutdown
        }

        // Simulate process restart by reopening log file
        let log2 = RevocationLog::open_or_create(vfs, &log_path, clock, None, vk)?;
        assert_eq!(log2.len(), 2);
        assert!(log2.is_revoked(&kek_target));
        assert!(log2.is_revoked(&group_target));
        assert!(log2.verify_integrity().is_ok());

        Ok(())
    }

    #[test]
    fn test_get_or_derive_fails_with_dedicated_error_on_revoked_group() -> Result<()> {
        let (sk, vk) = test_keypair();
        let clock = Arc::new(SystemClock::new());
        let log = Arc::new(RevocationLog::new_in_memory(clock, Some(sk), vk));

        let master_km = KeyManager::try_new("master-passphrase", b"salt-123")?;
        let registry = KeyRegistry::new().with_revocation_log(log.clone());
        let group_id = 777;

        // Precondition: key retrieval and encryption succeed before revocation
        let (ct, nonce) = registry.encrypt_with_group(&master_km, group_id, b"sensitive data")?;
        let decrypted = registry.decrypt_with_group(group_id, &ct, &nonce)?;
        assert_eq!(decrypted, b"sensitive data");

        // Action: Revoke group in log
        log.append(RevocationTarget::Group(group_id))?;

        // Proof (a): get_or_derive fails with dedicated CryptoError::KeyRevoked
        let res_derive = registry.get_or_derive(&master_km, group_id);
        assert!(matches!(res_derive, Err(CryptoError::KeyRevoked(ref msg)) if msg.contains("777")));

        // Proof (b): encrypt_with_group fails with dedicated CryptoError::KeyRevoked
        let res_enc = registry.encrypt_with_group(&master_km, group_id, b"new payload");
        assert!(matches!(res_enc, Err(CryptoError::KeyRevoked(_))));

        // Proof (c): decrypt_with_group fails with dedicated CryptoError::KeyRevoked
        let res_dec = registry.decrypt_with_group(group_id, &ct, &nonce);
        assert!(matches!(res_dec, Err(CryptoError::KeyRevoked(_))));

        Ok(())
    }

    #[test]
    fn test_persisted_log_tamper_detection_fails_open() -> Result<()> {
        let vfs: Arc<dyn Vfs> = Arc::new(TestVfs::new());
        let log_path = PathBuf::from("/revocation.log");
        let (sk, vk) = test_keypair();
        let clock = Arc::new(SystemClock::new());

        {
            let log =
                RevocationLog::open_or_create(vfs.clone(), &log_path, clock.clone(), Some(sk), vk)?;
            log.append(RevocationTarget::Group(1))?;
            log.append(RevocationTarget::Group(2))?;
        }

        // Tamper with 1 byte in the file via Vfs
        let mut raw_bytes = vfs
            .read(&log_path)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;
        assert!(!raw_bytes.is_empty());
        let last_idx = raw_bytes.len() - 1;
        raw_bytes[last_idx] ^= 0xFF; // Flip bits of last byte
        vfs.write_atomic(&log_path, &raw_bytes)
            .map_err(|e| CryptoError::Crypto(e.to_string()))?;

        // Attempting to open tampered file MUST fail with IntegrityViolation
        let open_res = RevocationLog::open_or_create(vfs, &log_path, clock, None, vk);
        assert!(
            matches!(open_res, Err(CryptoError::IntegrityViolation)),
            "Opening tampered log file MUST return CryptoError::IntegrityViolation"
        );

        Ok(())
    }
}
