// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Typdefinitionen und Datenstrukturen für den SSTable-Witness-Reader, Scanner und Evidenzberichte.
// INVARIANTEN: No Key/Value bytes in Debug/Display/Logs; Fail-Closed; V1 Contract Evidenzfelder.

use bytes::Bytes;
use std::fmt;
use std::path::PathBuf;

/// A single entry decoded by the WitnessReader.
#[derive(Clone, PartialEq, Eq)]
pub struct WitnessEntry {
    pub key: Bytes,
    pub value: Bytes,
    pub seq_no: u64,
    pub tx_id: u64,
    pub is_tombstone: bool,
}

impl WitnessEntry {
    /// Returns the raw sequence number with the tombstone bit masked out.
    #[inline]
    pub fn raw_seq(&self) -> u64 {
        self.seq_no & !contextra_types::TOMBSTONE_BIT
    }
}

impl fmt::Debug for WitnessEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WitnessEntry")
            .field("key_len", &self.key.len())
            .field("val_len", &self.value.len())
            .field("seq_no", &self.seq_no)
            .field("tx_id", &self.tx_id)
            .field("is_tombstone", &self.is_tombstone)
            .finish()
    }
}

/// Metadata extracted from SSTable trailer and blocks for witness auditing.
#[derive(Clone, PartialEq, Eq)]
pub struct WitnessSstMetadata {
    pub first_key: Bytes,
    pub last_key: Bytes,
    pub file_size: u64,
    pub min_tx_id: u64,
    pub max_tx_id: u64,
    pub min_seq: u64,
    pub max_seq: u64,
    pub format_version: u16,
}

impl fmt::Debug for WitnessSstMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WitnessSstMetadata")
            .field("first_key_len", &self.first_key.len())
            .field("last_key_len", &self.last_key.len())
            .field("file_size", &self.file_size)
            .field("min_tx_id", &self.min_tx_id)
            .field("max_tx_id", &self.max_tx_id)
            .field("min_seq", &self.min_seq)
            .field("max_seq", &self.max_seq)
            .field("format_version", &self.format_version)
            .finish()
    }
}

/// Error type specific to Witness operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitnessReadError {
    /// The file header or trailer is corrupt.
    CorruptFile(String),
    /// File cannot be classified or parsed according to SSTable specs.
    UnclassifiedFile(String),
    /// Encrypted file without available decryption key.
    NotWitnessable(String),
    /// Key was destroyed (Crypto-Erase).
    KeyDestroyed(String),
    /// I/O read failure.
    Io(String),
    /// Frame or payload decoding failure.
    Decode(String),
    /// Scan timeout reached.
    ScanTimeout(String),
}

impl fmt::Display for WitnessReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitnessReadError::CorruptFile(msg) => write!(f, "CorruptFile: {msg}"),
            WitnessReadError::UnclassifiedFile(msg) => write!(f, "UnclassifiedFile: {msg}"),
            WitnessReadError::NotWitnessable(msg) => write!(f, "NotWitnessable: {msg}"),
            WitnessReadError::KeyDestroyed(msg) => write!(f, "KeyDestroyed: {msg}"),
            WitnessReadError::Io(msg) => write!(f, "Io: {msg}"),
            WitnessReadError::Decode(msg) => write!(f, "Decode: {msg}"),
            WitnessReadError::ScanTimeout(msg) => write!(f, "ScanTimeout: {msg}"),
        }
    }
}

impl std::error::Error for WitnessReadError {}

/// Scope specification for witness scanning.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SstScopeSpec {
    /// Explicit target keys to audit.
    pub explicit: Vec<Vec<u8>>,
    /// Key prefixes to audit.
    pub prefixes: Vec<Vec<u8>>,
}

/// Input parameters for the SSTable witness scanner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SstScanInputs {
    pub data_dirs: Vec<PathBuf>,
    pub live_files: Vec<PathBuf>,
    pub pending_delete_files: Vec<PathBuf>,
    pub manifest_generation: u64,
    pub as_of_tx: u64,
    pub depth: u32,
}

/// File classification during inventory stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SstFileClass {
    Live,
    ObsoletePending,
    Orphan,
    Temp,
    OldManifest,
    Unknown,
}

/// Interface Contract V1 Evidence report structure.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SstScanEvidence {
    pub manifest_gen: u64,
    pub files_in_manifest: u64,
    pub files_scanned: u64,
    pub files_pruned_with_verified_range: u64,
    pub blocks_scanned: u64,
    pub bytes_scanned: u64,
    pub orphan_files_found: u64,
    pub unclassified_files: u64,
    pub versions_found: u64,
    pub identifier_hits_index: u64,
    pub tombstones_remaining: u64,

    // Manifest Evidence
    pub manifests_scanned: u64,
    pub range_hits: u64,
}

impl SstScanEvidence {
    /// Checks for invariant violations according to Interface Contract V1 based on whether key identifiers are claimed.
    /// Claim CT-S2V=3 (Value witness) or CT-S2K=4 (Key witness).
    pub fn invariant_violations_for_claim(&self, key_identifiers_claimed: bool) -> Vec<String> {
        let mut violations = Vec::new();
        if self.unclassified_files > 0 {
            violations.push(format!(
                "Fail-closed: {} unclassified file(s) found",
                self.unclassified_files
            ));
        }
        if self.versions_found > 0 {
            violations.push(format!(
                "Claim CT-S2V violation: {} live version(s) found in storage",
                self.versions_found
            ));
        }
        if key_identifiers_claimed && self.identifier_hits_index > 0 {
            violations.push(format!(
                "Claim CT-S2K violation: {} index separator/fence hit(s) found",
                self.identifier_hits_index
            ));
        }
        if key_identifiers_claimed && self.tombstones_remaining > 0 {
            violations.push(format!(
                "Claim CT-S2K violation: {} tombstone(s) remaining",
                self.tombstones_remaining
            ));
        }
        if self.range_hits > 0 {
            violations.push(format!(
                "Claim CT-S2K/CT-S2V violation: {} historical manifest range hit(s) found",
                self.range_hits
            ));
        }
        violations
    }

    /// Checks for invariant violations assuming key identifiers are claimed.
    pub fn invariant_violations(&self) -> Vec<String> {
        self.invariant_violations_for_claim(true)
    }
}
