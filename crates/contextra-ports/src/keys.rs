//! Key and encryption sealing port contracts (SPEC §4.2, §8.2, PO-04, PO-06, PO-08).
//!
//! Defines [`RecordSealer`], [`FileKeyring`], and the zeroizing [`FileKey`] type.

use crate::gate::{KeyBuf, KeyRange};
use crate::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a durable storage file or SSTable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub u64);

impl FileId {
    /// Creates a new `FileId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying numeric ID.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Kind of file managed by [`FileKeyring`] (SPEC §8.2, D-85).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    /// SSTable data file.
    SSTable,
    /// Write-Ahead-Log (WAL) file.
    Wal,
    /// Inverted index segment.
    TextSegment,
    /// Vector HNSW segment.
    VectorSegment,
    /// General auxiliary file.
    Auxiliary,
}

/// Wrapper around a cryptographic file key with zeroizing on drop and masked debug display (PO-08).
pub struct FileKey {
    raw: Vec<u8>,
    file_id: FileId,
}

impl FileKey {
    /// Creates a new `FileKey`.
    pub fn new(raw: Vec<u8>, file_id: FileId) -> Self {
        Self { raw, file_id }
    }

    /// Returns a reference to the raw secret bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// Associated file ID.
    pub fn file_id(&self) -> FileId {
        self.file_id
    }
}

impl Drop for FileKey {
    fn drop(&mut self) {
        // Zeroize without unsafe
        self.raw.fill(0);
    }
}

impl fmt::Debug for FileKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Mask secret material completely (PO-08, HR-4)
        write!(
            f,
            "FileKey {{ file_id: {:?}, key: [REDACTED] }}",
            self.file_id
        )
    }
}

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

/// Record sealer contract for crypto envelope management (SPEC §4.2, D-53).
pub trait RecordSealer: Send + Sync {
    /// Encrypts and seals plaintext with slot allocation if missing.
    fn seal<'a>(
        &'a self,
        ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>>;

    /// Decrypts envelope. If slot was destroyed/missing, returns error indicating erasure.
    fn open<'a>(
        &'a self,
        key: &'a KeyBuf,
        env: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<u8>, String>>;

    /// Evicts key range from DEK cache, zeroizing RAM keys (D-79).
    fn evict(&self, scope: &KeyRange);

    /// Forces persistence of vault slots (`sync_data`) before WAL frame flush (I-9, L-9).
    fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>>;
}

/// File keyring interface managing per-file cryptographic keys (SPEC §4.1, §8.2, D-85, PO-06).
///
/// # Contract (PO-06)
/// `destroy` zeroes the key slot, executes `sync_data`, and verifies the key's non-presence
/// via a read-check; failure leaves the slot in `Pending` requiring a retry upon reopen (D-55).
pub trait FileKeyring: Send + Sync {
    /// Allocates and records a new file key for the given kind and ID.
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>;

    /// Opens and retrieves the file key for the given ID.
    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>;

    /// Securely destroys the file key slot (PO-06).
    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>>;
}
