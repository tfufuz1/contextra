// FILE-CONTEXT
// ZWECK: Persistierter, Ed25519-signierter, manipulationssicherer Append-Only Widerrufslog (v17 Teil 16.1).
// INVARIANTEN: Monotone Sequence Numbers (0, 1, ...). Tamper-proof SHA-256 Hash-Verkettung. Ed25519 Signatur-Verifikation pro Eintrag. Atomare Initialisierungs- & Truncation-Prüfung via Marker-Datei (<path>.initialized).
// ABGRENZUNG PROMPT 10: Prompt 10 definiert KEK/DEK Envelope-Datenstrukturen; revocation_log stellt die persistierte Widerrufs-Engine bereit.
// ABGRENZUNG PROMPT 12: Prompt 12 betrifft die Audit-Hash-Kette, die diesen Revocation-Log als Eingabequelle referenzieren kann, aber ein eigenständiges, separates System-Artefakt ist.
// HOTSPOTS: [RevocationLog::open_or_create, RevocationLog::open_existing, RevocationLog::create_new, RevocationLog::append, RevocationLog::verify_chain]

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
//!
//! # Limitierung Rollback-Schutz
//! Ein simultaner Rollback sowohl der Logdatei als auch der Marker-Datei auf ein älteres,
//! in sich konsistentes und gültiges Paar bleibt lokal unerkannt, da beide Dateien die
//! gültige Signatur des jeweiligen Standes tragen. Für vollständigen Rollback-Schutz
//! über Prozess-Neustarts hinweg ist ein externer Anker erforderlich (z. B. AuditChain-Anchor).

use crate::error::{CryptoError, Result};
use contextra_ports::Clock;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const MARKER_MAGIC: &[u8; 4] = b"RVMK";
const MARKER_VERSION_1: u8 = 1;

/// Marker structure stored in `<log-path>.initialized` for atomic deletion & truncation protection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationMarker {
    /// Minimum expected entry count in the log.
    pub count: u64,
    /// Head hash of the latest log entry (`[0u8; 32]` if count == 0).
    pub head_hash: [u8; 32],
}

/// Helper to serialize a signed marker file.
fn serialize_signed_marker(
    count: u64,
    head_hash: [u8; 32],
    signing_key: Option<&SigningKey>,
) -> Vec<u8> {
    if let Some(sk) = signing_key {
        let mut msg = Vec::with_capacity(40);
        msg.extend_from_slice(&count.to_le_bytes());
        msg.extend_from_slice(&head_hash);
        let sig = sk.sign(&msg);

        let mut buf = Vec::with_capacity(4 + 1 + 8 + 32 + 64);
        buf.extend_from_slice(MARKER_MAGIC);
        buf.push(MARKER_VERSION_1);
        buf.extend_from_slice(&count.to_le_bytes());
        buf.extend_from_slice(&head_hash);
        buf.extend_from_slice(&sig.to_bytes());
        buf
    } else {
        let marker = RevocationMarker { count, head_hash };
        bincode::serialize(&marker).unwrap_or_default()
    }
}

/// Helper to deserialize and verify a marker file.
fn parse_and_verify_marker(
    marker_bytes: &[u8],
    verifying_key: &VerifyingKey,
) -> Result<RevocationMarker> {
    if marker_bytes.starts_with(MARKER_MAGIC) {
        if marker_bytes.len() < 4 + 1 + 8 + 32 + 64 {
            return Err(CryptoError::IntegrityViolation);
        }
        let version = marker_bytes[4];
        if version != MARKER_VERSION_1 {
            return Err(CryptoError::IntegrityViolation);
        }

        let mut count_bytes = [0u8; 8];
        count_bytes.copy_from_slice(&marker_bytes[5..13]);
        let count = u64::from_le_bytes(count_bytes);

        let mut head_hash = [0u8; 32];
        head_hash.copy_from_slice(&marker_bytes[13..45]);

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&marker_bytes[45..109]);

        let mut msg = Vec::with_capacity(40);
        msg.extend_from_slice(&count_bytes);
        msg.extend_from_slice(&head_hash);

        let sig = Signature::from_bytes(&sig_bytes);
        verifying_key
            .verify(&msg, &sig)
            .map_err(|_| CryptoError::IntegrityViolation)?;

        Ok(RevocationMarker { count, head_hash })
    } else {
        let marker: RevocationMarker =
            bincode::deserialize(marker_bytes).map_err(|_| CryptoError::IntegrityViolation)?;
        tracing::warn!("Loaded legacy unsigned revocation marker; will upgrade on next write");
        Ok(marker)
    }
}

/// Cursor for amortized incremental verification of log entries.
#[derive(Debug, Clone, Copy)]
struct VerifiedCursor {
    verified_len: usize,
    last_verified_hash: [u8; 32],
}

/// Mode for initializing or opening a `RevocationLog`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitMode {
    /// Explicit fresh creation. Fails if log or marker file already exists with contents.
    CreateNew,
    /// Explicit reopen of existing log. Fails with `IntegrityViolation` if log or marker file is missing or truncated.
    OpenExisting,
    /// Auto-detect: Fresh creation if no marker and empty/missing log; otherwise open existing.
    /// Fails closed if marker exists without log, log exists without marker, or if `has_existing_keys` is true when log and marker are missing.
    OpenOrCreateIfFresh {
        /// Whether existing wrapped key traces or registry entries are present.
        has_existing_keys: bool,
    },
}

/// Computes the path of the `.initialized` marker file corresponding to a given log file path.
pub fn marker_path_for(log_path: &Path) -> PathBuf {
    let mut os_str = log_path.as_os_str().to_os_string();
    os_str.push(".initialized");
    PathBuf::from(os_str)
}

fn write_atomic_file(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };

    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let count = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_path = parent.join(format!(
        "{}.tmp-{}-{}",
        file_name,
        std::process::id(),
        count
    ));

    let write_tmp = || -> Result<()> {
        let mut tmp_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
            .map_err(|e| CryptoError::Crypto(format!("Failed to create temp file: {e}")))?;

        tmp_file
            .write_all(content)
            .map_err(|e| CryptoError::Crypto(format!("Failed to write temp file: {e}")))?;

        tmp_file
            .sync_all()
            .map_err(|e| CryptoError::Crypto(format!("Failed to sync temp file: {e}")))?;

        Ok(())
    };

    if let Err(err) = write_tmp() {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(err);
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(CryptoError::Crypto(format!(
            "Failed to rename temp file: {e}"
        )));
    }

    if let Ok(dir_file) = File::open(parent) {
        dir_file
            .sync_all()
            .map_err(|e| CryptoError::Crypto(format!("Failed to sync parent directory: {e}")))?;
    }

    Ok(())
}

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
    file_path: Option<PathBuf>,
    signing_key: Option<SigningKey>,
    verifying_key: VerifyingKey,
    clock: Arc<dyn Clock>,
    entries: RwLock<Vec<RevocationEntry>>,
    revoked_targets: RwLock<HashSet<RevocationTarget>>,
    verified_cursor: RwLock<VerifiedCursor>,
}

impl RevocationLog {
    /// Erstellt einen neuen In-Memory-Widerrufslog ohne Dateipersistenz.
    pub fn new_in_memory(
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
    ) -> Self {
        Self {
            file_path: None,
            signing_key,
            verifying_key,
            clock,
            entries: RwLock::new(Vec::new()),
            revoked_targets: RwLock::new(HashSet::new()),
            verified_cursor: RwLock::new(VerifiedCursor {
                verified_len: 0,
                last_verified_hash: [0u8; 32],
            }),
        }
    }

    /// Erzeugt einen neuen Widerrufslog unter `path` (Erstinitialisierung).
    /// Fails mit `CryptoError::IntegrityViolation`, falls die Datei oder die Marker-Datei bereits existiert.
    pub fn create_new(
        path: impl AsRef<Path>,
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
    ) -> Result<Self> {
        Self::open_or_create_with_mode(path, clock, signing_key, verifying_key, InitMode::CreateNew)
    }

    /// Öffnet einen bestehenden Widerrufslog unter `path`.
    /// Fails mit `CryptoError::IntegrityViolation`, falls die Log-Datei oder die Marker-Datei fehlt oder gekürzt ist.
    pub fn open_existing(
        path: impl AsRef<Path>,
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
    ) -> Result<Self> {
        Self::open_or_create_with_mode(
            path,
            clock,
            signing_key,
            verifying_key,
            InitMode::OpenExisting,
        )
    }

    /// Abwärtskompatibles Öffnen oder Erzeugen unter `path` (nutzt `InitMode::OpenOrCreateIfFresh`).
    ///
    /// # Warnung / Security Note
    /// Diese Methode setzt fest `has_existing_keys: false` ein, was den Rollback-Schutz
    /// bei Neu-Initialisierung über einem gelöschten Log abschwächt.
    /// Produktionscode MUSS `open_or_create_if_fresh` mit korrekter Flag-Übergabe nutzen!
    pub fn open_or_create(
        path: impl AsRef<Path>,
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
    ) -> Result<Self> {
        Self::open_or_create_with_mode(
            path,
            clock,
            signing_key,
            verifying_key,
            InitMode::OpenOrCreateIfFresh {
                has_existing_keys: false,
            },
        )
    }

    /// Öffnet oder erzeugt einen Widerrufslog mit Schutz vor Stale/Missing Log wenn Schlüsselspuren existieren.
    pub fn open_or_create_if_fresh(
        path: impl AsRef<Path>,
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
        has_existing_keys: bool,
    ) -> Result<Self> {
        Self::open_or_create_with_mode(
            path,
            clock,
            signing_key,
            verifying_key,
            InitMode::OpenOrCreateIfFresh { has_existing_keys },
        )
    }

    /// Öffnet oder erzeugt einen Widerrufslog mit explizitem `InitMode`.
    pub fn open_or_create_with_mode(
        path: impl AsRef<Path>,
        clock: Arc<dyn Clock>,
        signing_key: Option<SigningKey>,
        verifying_key: VerifyingKey,
        mode: InitMode,
    ) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let m_path = marker_path_for(&path_buf);

        let log_exists = path_buf.exists();
        let log_has_content = log_exists
            && std::fs::metadata(&path_buf)
                .map(|m| m.len() > 0)
                .unwrap_or(false);
        let marker_exists = m_path.exists();

        let effective_mode = match mode {
            InitMode::CreateNew => {
                if log_has_content || marker_exists {
                    return Err(CryptoError::IntegrityViolation);
                }
                InitMode::CreateNew
            }
            InitMode::OpenExisting => {
                if !log_exists || !marker_exists {
                    return Err(CryptoError::IntegrityViolation);
                }
                InitMode::OpenExisting
            }
            InitMode::OpenOrCreateIfFresh { has_existing_keys } => {
                if !log_has_content && !marker_exists {
                    if has_existing_keys {
                        return Err(CryptoError::IntegrityViolation);
                    }
                    InitMode::CreateNew
                } else if log_exists && marker_exists {
                    InitMode::OpenExisting
                } else {
                    // One exists but the other does not -> Fail Closed!
                    return Err(CryptoError::IntegrityViolation);
                }
            }
        };

        if effective_mode == InitMode::CreateNew {
            // Write initial empty log and marker
            let empty_entries: Vec<RevocationEntry> = Vec::new();
            let serialized_empty = bincode::serialize(&empty_entries).map_err(|e| {
                CryptoError::Crypto(format!("Failed to serialize empty revocation log: {e}"))
            })?;
            write_atomic_file(&path_buf, &serialized_empty)?;

            let serialized_marker = serialize_signed_marker(0, [0u8; 32], signing_key.as_ref());
            write_atomic_file(&m_path, &serialized_marker)?;

            return Ok(Self {
                file_path: Some(path_buf),
                signing_key,
                verifying_key,
                clock,
                entries: RwLock::new(Vec::new()),
                revoked_targets: RwLock::new(HashSet::new()),
                verified_cursor: RwLock::new(VerifiedCursor {
                    verified_len: 0,
                    last_verified_hash: [0u8; 32],
                }),
            });
        }

        // Open existing path
        let marker_bytes = std::fs::read(&m_path).map_err(|_| CryptoError::IntegrityViolation)?;
        let marker = parse_and_verify_marker(&marker_bytes, &verifying_key)?;

        let log_bytes = std::fs::read(&path_buf).map_err(|_| CryptoError::IntegrityViolation)?;
        let parsed_entries: Vec<RevocationEntry> =
            bincode::deserialize(&log_bytes).map_err(|_| CryptoError::IntegrityViolation)?;

        Self::verify_chain_entries(&parsed_entries, &verifying_key)?;

        // Truncation check: Log count cannot be smaller than marker count
        let log_count = parsed_entries.len() as u64;
        if log_count < marker.count {
            return Err(CryptoError::IntegrityViolation);
        }

        // If log_count == marker.count and count > 0, verify head hash match
        if log_count == marker.count && marker.count > 0 {
            if let Some(last) = parsed_entries.last() {
                if last.entry_hash != marker.head_hash {
                    return Err(CryptoError::IntegrityViolation);
                }
            }
        }

        let mut revoked_targets = HashSet::new();
        for entry in &parsed_entries {
            revoked_targets.insert(entry.target.clone());
        }

        let last_verified_hash = parsed_entries
            .last()
            .map(|e| e.entry_hash)
            .unwrap_or([0u8; 32]);

        Ok(Self {
            file_path: Some(path_buf),
            signing_key,
            verifying_key,
            clock,
            entries: RwLock::new(parsed_entries.clone()),
            revoked_targets: RwLock::new(revoked_targets),
            verified_cursor: RwLock::new(VerifiedCursor {
                verified_len: parsed_entries.len(),
                last_verified_hash,
            }),
        })
    }

    /// Gibt `true` zurück, wenn dieser Log datei-persistiert ist (`file_path` gesetzt ist).
    pub fn is_durable(&self) -> bool {
        self.file_path.is_some()
    }

    /// Verifiziert inkrementell die Integrität unverifizierter neuer Einträge im Log.
    pub fn verify_integrity(&self) -> Result<()> {
        let entries_guard = self.entries.read();
        let cursor = *self.verified_cursor.read();

        let total_len = entries_guard.len();
        if cursor.verified_len > total_len {
            return Err(CryptoError::IntegrityViolation);
        }

        if cursor.verified_len == total_len {
            if total_len == 0 {
                return Ok(());
            }
            if entries_guard[total_len - 1].entry_hash != cursor.last_verified_hash {
                return Err(CryptoError::IntegrityViolation);
            }
            return Ok(());
        }

        let unverified_slice = &entries_guard[cursor.verified_len..total_len];
        let initial_prev_hash = if cursor.verified_len == 0 {
            [0u8; 32]
        } else {
            if entries_guard[cursor.verified_len - 1].entry_hash != cursor.last_verified_hash {
                return Err(CryptoError::IntegrityViolation);
            }
            cursor.last_verified_hash
        };

        Self::verify_chain_slice(
            unverified_slice,
            cursor.verified_len as u64,
            initial_prev_hash,
            &self.verifying_key,
        )?;

        let new_last_hash = unverified_slice
            .last()
            .map(|e| e.entry_hash)
            .unwrap_or(initial_prev_hash);

        drop(entries_guard);

        let mut cursor_guard = self.verified_cursor.write();
        if total_len > cursor_guard.verified_len {
            cursor_guard.verified_len = total_len;
            cursor_guard.last_verified_hash = new_last_hash;
        }

        Ok(())
    }

    /// Verifiziert bedingungslos die gesamte Eintrags-Kette neu von Index 0 an.
    pub fn verify_integrity_full(&self) -> Result<()> {
        let entries_guard = self.entries.read();
        Self::verify_chain_entries(&entries_guard, &self.verifying_key)?;

        let total_len = entries_guard.len();
        let last_hash = entries_guard
            .last()
            .map(|e| e.entry_hash)
            .unwrap_or([0u8; 32]);

        drop(entries_guard);

        let mut cursor_guard = self.verified_cursor.write();
        cursor_guard.verified_len = total_len;
        cursor_guard.last_verified_hash = last_hash;

        Ok(())
    }

    /// Hilfsfunktion zur Verifikation einer Liste von Eintrags-Ketten ab Index 0.
    fn verify_chain_entries(
        entries: &[RevocationEntry],
        verifying_key: &VerifyingKey,
    ) -> Result<()> {
        Self::verify_chain_slice(entries, 0, [0u8; 32], verifying_key)
    }

    /// Hilfsfunktion zur Verifikation eines Slice von Eintrags-Ketten ab `start_seq`.
    fn verify_chain_slice(
        slice: &[RevocationEntry],
        start_seq: u64,
        mut expected_prev_hash: [u8; 32],
        verifying_key: &VerifyingKey,
    ) -> Result<()> {
        for (offset, entry) in slice.iter().enumerate() {
            let expected_seq = start_seq + offset as u64;
            if entry.sequence_number != expected_seq {
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
    /// PERSIST-BEFORE-MUTATE: Serialisiert und schreibt die neue Logdatei und den signierten Marker
    /// vor der Mutation des RAM-Zustands (`entries`, `revoked_targets`). Bei Fehler bleibt der
    /// RAM-Zustand exakt unberührt.
    pub fn append(&self, target: RevocationTarget) -> Result<RevocationEntry> {
        let signing_key = self.signing_key.as_ref().ok_or_else(|| {
            CryptoError::Crypto("Signing key not configured for RevocationLog append".to_string())
        })?;

        let entries_guard = self.entries.read();

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

        let mut candidate_entries = entries_guard.clone();
        candidate_entries.push(new_entry.clone());
        drop(entries_guard);

        if let Some(ref path) = self.file_path {
            let serialized_log = bincode::serialize(&candidate_entries).map_err(|e| {
                CryptoError::Crypto(format!("Failed to serialize revocation log: {e}"))
            })?;

            write_atomic_file(path, &serialized_log)?;

            let m_path = marker_path_for(path);
            let serialized_marker = serialize_signed_marker(
                candidate_entries.len() as u64,
                new_entry.entry_hash,
                Some(signing_key),
            );

            write_atomic_file(&m_path, &serialized_marker)?;
        }

        let mut entries_write_guard = self.entries.write();
        let mut revoked_write_guard = self.revoked_targets.write();

        entries_write_guard.push(new_entry.clone());
        revoked_write_guard.insert(target);

        let mut cursor_guard = self.verified_cursor.write();
        cursor_guard.verified_len = entries_write_guard.len();
        cursor_guard.last_verified_hash = new_entry.entry_hash;

        drop(entries_write_guard);
        drop(revoked_write_guard);

        Ok(new_entry)
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
            .field("file_path", &self.file_path)
            .field("verifying_key", &self.verifying_key)
            .field("entries_count", &self.entries.read().len())
            .field("is_durable", &self.is_durable())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyManager;
    use crate::kv_shredding::KeyRegistry;
    use contextra_ports::SystemClock;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use tempfile::NamedTempFile;

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
        assert!(log.verify_integrity_full().is_ok());
        Ok(())
    }

    #[test]
    fn test_persistence_and_simulated_restart() -> Result<()> {
        let tmp_file = NamedTempFile::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let log_path = tmp_file.path().to_path_buf();
        let (sk, vk) = test_keypair();
        let clock = Arc::new(SystemClock::new());

        let kek_target = RevocationTarget::Kek("kek-v17".into());
        let group_target = RevocationTarget::Group(1001);

        {
            let log1 = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)?;
            log1.append(kek_target.clone())?;
            log1.append(group_target.clone())?;
            assert_eq!(log1.len(), 2);
            // Drop log1 simulating process shutdown
        }

        // Simulate process restart by reopening log file
        let log2 = RevocationLog::open_or_create(&log_path, clock, None, vk)?;
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
        let tmp_file = NamedTempFile::new().map_err(|e| CryptoError::Crypto(e.to_string()))?;
        let log_path = tmp_file.path().to_path_buf();
        let (sk, vk) = test_keypair();
        let clock = Arc::new(SystemClock::new());

        {
            let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)?;
            log.append(RevocationTarget::Group(1))?;
            log.append(RevocationTarget::Group(2))?;
        }

        // Tamper with 1 byte in the file
        let mut raw_bytes =
            std::fs::read(&log_path).map_err(|e| CryptoError::Crypto(e.to_string()))?;
        assert!(!raw_bytes.is_empty());
        let last_idx = raw_bytes.len() - 1;
        raw_bytes[last_idx] ^= 0xFF; // Flip bits of last byte
        std::fs::write(&log_path, &raw_bytes).map_err(|e| CryptoError::Crypto(e.to_string()))?;

        // Attempting to open tampered file MUST fail with IntegrityViolation
        let open_res = RevocationLog::open_or_create(&log_path, clock, None, vk);
        assert!(
            matches!(open_res, Err(CryptoError::IntegrityViolation)),
            "Opening tampered log file MUST return CryptoError::IntegrityViolation"
        );

        Ok(())
    }
}
